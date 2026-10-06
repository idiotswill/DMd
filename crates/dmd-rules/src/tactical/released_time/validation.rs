use super::*;
use std::collections::{HashMap, HashSet};

/// Private constructor, exact candidate borrow. This is neither serialized nor
/// inferred from an ambient flag. Shared readers cannot apply it to another state.
pub(crate) struct ReleasedValidation<'a> {
    state: &'a CampaignState,
}
impl<'a> ReleasedValidation<'a> {
    pub(super) fn derive(state: &'a CampaignState) -> Result<Self, RulesError> {
        super::super::release::require_completion_history(state)?;
        if !state.validate().is_empty() {
            return Err(invalid("released candidate is structurally invalid"));
        }
        let f = flow(state)?;
        if f.version != TacticalExecutionVersion::ReleasedTimeV1.flow_version() {
            return Err(invalid("released validation requires exact flow 7"));
        }
        if f.phase != TacticalPhase::Finished {
            if f.released_time_upgrade.is_some()
                || f.resolution
                    .as_ref()
                    .is_some_and(|r| r.released_interval().is_some())
            {
                return Err(invalid("released context outside Finished"));
            }
            return Ok(Self { state });
        }
        state
            .encounter_history
            .as_ref()
            .ok_or_else(|| invalid("released validation lacks history"))?
            .validate(state)
            .map_err(|e| invalid(&e))?;
        if let Some(r) = f.resolution.as_deref() {
            validate_interval(state, r)?;
        }
        Ok(Self { state })
    }
    pub(crate) fn binds(&self, state: &CampaignState) -> bool {
        std::ptr::eq(self.state, state)
    }
    pub(crate) fn interval(&self, state: &CampaignState) -> bool {
        self.binds(state)
            && state
                .encounter
                .as_ref()
                .and_then(|e| e.flow.as_ref())
                .and_then(|f| f.resolution.as_ref())
                .is_some_and(|r| r.released_interval().is_some())
    }
    pub(crate) fn require_interval(&self, state: &CampaignState) -> Result<(), RulesError> {
        if self.interval(state) {
            Ok(())
        } else {
            Err(invalid("released proof belongs to another context"))
        }
    }
    fn binding(
        &self,
        state: &CampaignState,
        matches: impl Fn(&ReleasedDeadlineSource) -> bool,
    ) -> bool {
        if !self.interval(state) {
            return false;
        }
        let r = resolution(state).expect("candidate checked");
        r.released_interval()
            .and_then(|i| i.batches.last())
            .is_some_and(|batch| {
                batch.bindings.iter().any(|b| {
                    matches(&b.source)
                        && b.due_at == state.clock.now
                        && r.frames.iter().flatten().any(|w| *w == b.work)
                })
            })
    }
    pub(crate) fn legacy(&self, state: &CampaignState, id: EffectId) -> bool {
        self.binding(
            state,
            |s| matches!(s, ReleasedDeadlineSource::Legacy { effect, .. } if *effect == id),
        )
    }
    pub(crate) fn effect(&self, state: &CampaignState, id: EffectId, group: bool) -> bool {
        self.binding(state, |s| match s {
            ReleasedDeadlineSource::Group { group: key, .. } => group && *key == id,
            ReleasedDeadlineSource::Effect { effect, .. } => !group && *effect == id,
            _ => false,
        })
    }
    pub(crate) fn stable(&self, state: &CampaignState, actor: EntityId) -> bool {
        self.binding(
            state,
            |s| matches!(s, ReleasedDeadlineSource::Stable { actor: id, .. } if *id == actor),
        )
    }
    pub(crate) fn ticket(&self, state: &CampaignState, ticket: EffectTicketId) -> bool {
        self.interval(state)
            && resolution(state).is_ok_and(|r| {
                r.frames.iter().flatten().any(
                    |w| matches!(w.kind, TacticalWorkKind::Effect { ticket: id } if id == ticket),
                )
            })
    }
}

fn origin(state: &CampaignState, meta: &CommandMeta, root: &CommandMeta) -> Result<(), RulesError> {
    privileged(meta)?;
    if meta.id.0.is_nil()
        || meta.campaign_id != state.campaign_id()
        || meta.session_id.is_none_or(|id| id.0.is_nil())
        || meta.expected_event_sequence < root.expected_event_sequence
        || meta.expected_event_sequence > state.applied_event_sequence
        || (meta.expected_event_sequence == root.expected_event_sequence && *meta != *root)
    {
        return Err(invalid("released operation has invalid command provenance"));
    }
    Ok(())
}

fn validate_interval(state: &CampaignState, r: &TacticalResolution) -> Result<(), RulesError> {
    let i = r
        .released_interval()
        .ok_or_else(|| invalid("Finished resolution requires explicit released context"))?;
    let history = state
        .encounter_history
        .as_ref()
        .ok_or_else(|| invalid("interval lacks history"))?;
    let release = history
        .last()
        .ok_or_else(|| invalid("interval lacks release"))?;
    let prior = history.elapsed_intervals.last();
    if flow(state)?
        .released_time_upgrade
        .as_ref()
        .is_some_and(|upgrade| {
            upgrade.origin.expected_event_sequence >= r.origin.expected_event_sequence
        })
    {
        return Err(invalid("interval predates explicit release upgrade"));
    }
    origin(state, &r.origin, &r.origin)?;
    if i.release != release.released_by.id
        || r.origin.expected_event_sequence <= release.released_by.expected_event_sequence
        || i.predecessor != prior.map(|p| p.origin.id)
        || prior.is_some_and(|p| {
            p.completed_by.expected_event_sequence >= r.origin.expected_event_sequence
                || p.target_at > i.started_at
        })
        || i.started_at < release.released_at
        || i.target_at <= i.started_at
        || i.progress_at != state.clock.now
        || i.progress_at <= i.started_at
        || i.progress_at > i.target_at
        || !valid_released_time_ruling(&i.ruling)
        || i.batches.is_empty()
        || i.batches.len() > 128
        || r.pending.is_some()
        || r.failed_save.is_some()
        || r.legendary_window.is_some()
        || r.attack.is_some()
        || r.hit_review.is_some()
        || r.movement.is_some()
        || !r.casts.is_empty()
        || !r.missiles.is_empty()
        || !r.falls.is_empty()
        || !r.areas.is_empty()
        || r.frames.len() != 1
        || r.frames[0].len() < 2
        || r.next_occurrence > 32_768
    {
        return Err(invalid("invalid persisted released interval boundary"));
    }
    deadlines::preflight(state)?;
    let trace = r
        .work_trace
        .as_ref()
        .ok_or_else(|| invalid("released work lacks ancestry"))?;
    if trace.active.is_some() || trace.nodes.len() != usize::from(r.next_occurrence) {
        return Err(invalid("released work trace is unfinished or incomplete"));
    }
    let inventory = deadlines::inventory(state)?;
    let mut seen = HashSet::new();
    let mut live = HashSet::new();
    let mut last_at = i.started_at;
    let mut last_seq = r.origin.expected_event_sequence;
    let mut commands = HashMap::from([(last_seq, &r.origin)]);
    let mut previous_observation: Option<&EffectOperationStamp> = None;
    for (index, batch) in i.batches.iter().enumerate() {
        origin(state, &batch.observed.command, &r.origin)?;
        register_command(&mut commands, &batch.observed.command)?;
        if previous_observation.is_some_and(|previous| {
            (previous.command.expected_event_sequence, previous.step)
                >= (
                    batch.observed.command.expected_event_sequence,
                    batch.observed.step,
                )
        }) {
            return Err(invalid(
                "deadline Time stamps are not in actual operation order",
            ));
        }
        previous_observation = Some(&batch.observed);
        if usize::from(batch.ordinal) != index
            || batch.at <= last_at
            || batch.at > i.target_at
            || batch.bindings.is_empty()
            || batch.observed.command.expected_event_sequence < last_seq
        {
            return Err(invalid("invalid ordered deadline batch"));
        }
        last_seq = batch.observed.command.expected_event_sequence;
        last_at = batch.at;
        let mut completed = HashSet::new();
        for (completion_index, completion) in batch.completions.iter().enumerate() {
            origin(state, &completion.completed_by, &r.origin)?;
            register_command(&mut commands, &completion.completed_by)?;
            let binding = batch
                .bindings
                .iter()
                .find(|b| b.work.occurrence == completion.work.occurrence)
                .ok_or_else(|| invalid("completion has no exact source binding"))?;
            if completion.work.resolution != r.origin.id
                || !completed.insert(completion.work.occurrence)
                || completion.completed_by.expected_event_sequence < last_seq
            {
                return Err(invalid(
                    "duplicate, reversed or unrelated released completion",
                ));
            }
            last_seq = completion.completed_by.expected_event_sequence;
            if inventory
                .iter()
                .any(|(at, source)| *at == binding.due_at && *source == binding.source)
            {
                return Err(invalid("completed deadline retains its source"));
            }
            if let ReleasedWorkOutcome::CancelledBy { work } = completion.outcome {
                let cause = batch
                    .completions
                    .iter()
                    .take(completion_index)
                    .find(|c| c.work == work && c.outcome == ReleasedWorkOutcome::Applied)
                    .ok_or_else(|| invalid("cancellation lacks applied sibling cause"))?;
                let source = batch
                    .bindings
                    .iter()
                    .find(|b| b.work.occurrence == cause.work.occurrence)
                    .ok_or_else(|| invalid("cleanup source absent"))?;
                if !deadlines::cleanup_cancels(&source.source, &binding.source)
                    || cause.completed_by != completion.completed_by
                {
                    return Err(invalid("cancelled deadline is unrelated to actual cleanup"));
                }
            }
        }
        let mut effect_ordinal = 0;
        for binding in &batch.bindings {
            let occurrence = binding.work.occurrence;
            if !deadlines::binding_shape(batch, binding, effect_ordinal)
                || binding.due_at != batch.at
                || !seen.insert(occurrence)
                || trace
                    .nodes
                    .get(usize::from(occurrence))
                    .is_none_or(|n| n.work != binding.work || n.parent.is_some())
            {
                return Err(invalid(
                    "deadline does not partition exact allocated ancestry",
                ));
            }
            if matches!(binding.work.kind, TacticalWorkKind::Effect { .. }) {
                effect_ordinal = effect_ordinal
                    .checked_add(1)
                    .ok_or_else(|| invalid("released effect ticket capacity"))?;
            }
            if !completed.contains(&occurrence) {
                if index + 1 != i.batches.len()
                    || !deadlines::source_present(state, binding)?
                    || !r.frames[0].contains(&binding.work)
                    || !live.insert(occurrence)
                {
                    return Err(invalid(
                        "unconsumed source is absent, stale or outside current batch",
                    ));
                }
                if let TacticalWorkKind::Effect { ticket } = binding.work.kind
                    && effects(state)?
                        .pending
                        .iter()
                        .find(|t| t.id == ticket)
                        .is_none_or(|t| t.origin != batch.observed || t.target.0.is_nil())
                {
                    return Err(invalid("expiry ticket differs from exact Time observation"));
                }
            } else if r.frames[0].contains(&binding.work) {
                return Err(invalid("completed deadline remains executable"));
            }
        }
    }
    if last_at != i.progress_at
        || seen.len() != trace.nodes.len()
        || live.len() != r.frames[0].len()
        || r.frames[0].iter().any(|w| !live.contains(&w.occurrence))
        || inventory.iter().any(|(at, source)| {
            *at <= state.clock.now
                && !i.batches.last().is_some_and(|batch| {
                    batch.bindings.iter().any(|b| {
                        b.due_at == *at && b.source == *source && live.contains(&b.work.occurrence)
                    })
                })
        })
        || effects(state)?.pending.iter().any(|ticket| {
            !r.frames[0].iter().any(
                |w| matches!(w.kind, TacticalWorkKind::Effect { ticket: id } if id == ticket.id),
            )
        })
    {
        return Err(invalid(
            "released work is not an exhaustive current source partition",
        ));
    }
    Ok(())
}

fn register_command<'a>(
    commands: &mut HashMap<u64, &'a CommandMeta>,
    meta: &'a CommandMeta,
) -> Result<(), RulesError> {
    if commands
        .get(&meta.expected_event_sequence)
        .is_some_and(|old| **old != *meta)
        || commands
            .values()
            .any(|old| old.id == meta.id && **old != *meta)
    {
        return Err(invalid(
            "released command identity or sequence is reused inconsistently",
        ));
    }
    commands.insert(meta.expected_event_sequence, meta);
    Ok(())
}
