//! Guarded released-time producer. Public/restore admission stays closed until
//! the complete app/session/portable vertical slice is reviewed.
mod deadlines;
mod validation;
use super::turns::*;
use super::*;
use crate::tactical_effects::*;
pub(crate) use validation::ReleasedValidation;

pub(crate) fn deny_public(state: &CampaignState) -> Result<(), RulesError> {
    if state
        .encounter
        .as_ref()
        .and_then(|e| e.flow.as_ref())
        .is_some_and(|f| {
            f.version == TacticalExecutionVersion::ReleasedTimeV1.flow_version()
                || f.released_time_upgrade.is_some()
                || f.resolution
                    .as_ref()
                    .is_some_and(|r| r.released_interval().is_some())
        })
        || state.encounter_history.as_ref().is_some_and(|h| {
            !h.elapsed_intervals.is_empty()
                || h.completions
                    .iter()
                    .any(|c| c.execution.supports_released_time())
        })
    {
        return Err(prerequisite(
            "released time awaits complete application and restore admission",
        ));
    }
    Ok(())
}

fn authority(state: &CampaignState, meta: &CommandMeta) -> Result<(), RulesError> {
    privileged(meta)?;
    if meta.campaign_id != state.campaign_id() || meta.id.0.is_nil() {
        return Err(RulesError::Unauthorized);
    }
    if meta.expected_event_sequence != state.applied_event_sequence {
        return Err(RulesError::Stale);
    }
    if meta.session_id.is_none()
        || state
            .table
            .as_ref()
            .and_then(|t| t.active_session.as_ref())
            .is_none_or(|s| Some(s.session_id) != meta.session_id)
    {
        return Err(RulesError::Unauthorized);
    }
    Ok(())
}

// Deliberately crate-private and unused by the public reducer at this checkpoint.
// Controls invoke the actual producer; serialized receipts never authorize it.
#[allow(dead_code)]
pub(super) fn transition(
    state: &CampaignState,
    meta: &CommandMeta,
    action: &TacticalAction,
    pack: &RulesPack,
) -> Result<CampaignState, RulesError> {
    authority(state, meta)?;
    if !state.validate().is_empty() {
        return Err(invalid("released input is structurally invalid"));
    }
    if flow(state)?.version == TacticalExecutionVersion::EncounterReleaseV1.flow_version() {
        crate::validate_state(state, pack)?;
    } else {
        validate(state, pack)?;
    }
    let mut next = state.clone();
    match action {
        TacticalAction::UpgradeExecutionTo {
            execution: TacticalExecutionVersion::ReleasedTimeV1,
        } => upgrade(&mut next, meta)?,
        TacticalAction::ConcludeHostilities { cadence, ruling }
            if TacticalExecutionVersion::from_flow_version(flow(state)?.version)
                .is_some_and(TacticalExecutionVersion::supports_released_time) =>
        {
            super::aftermath::conclude(&mut next, meta, *cadence, ruling)?
        }
        TacticalAction::FinishEncounter
            if TacticalExecutionVersion::from_flow_version(flow(state)?.version)
                .is_some_and(TacticalExecutionVersion::supports_released_time) =>
        {
            super::release::finish(&mut next, meta)?
        }
        TacticalAction::AdvanceReleasedTime {
            seconds,
            ordering,
            ruling,
        } => begin(&mut next, meta, *seconds, *ordering, ruling)?,
        TacticalAction::ChooseTurnWork { occurrence }
            if resolution(state)?.released_interval().is_some() =>
        {
            choose(&mut next, meta, *occurrence)?
        }
        _ => {
            return Err(prerequisite(
                "action is outside the guarded released-time producer",
            ));
        }
    }
    validate(&next, pack)?;
    Ok(next)
}

pub(super) fn release_preflight(
    state: &CampaignState,
) -> Result<super::release::EncounterReleaseReadiness, RulesError> {
    let proof = ReleasedValidation::derive(state)?;
    super::release::encounter_release_preflight_with_released(state, Some(&proof))
}

fn validate(state: &CampaignState, pack: &RulesPack) -> Result<(), RulesError> {
    let proof = ReleasedValidation::derive(state)?;
    crate::kernel::validate_released_state(state, pack, &proof)
}

fn upgrade(state: &mut CampaignState, meta: &CommandMeta) -> Result<(), RulesError> {
    let f = flow(state)?;
    if f.version != TacticalExecutionVersion::EncounterReleaseV1.flow_version()
        || f.resolution.is_some()
        || !f.ready.is_empty()
        || state.rules.as_ref().is_none_or(|r| r.pending.is_some())
        || !matches!(f.phase, TacticalPhase::Active | TacticalPhase::Finished)
    {
        return Err(prerequisite(
            "released time requires an explicit settled 5-to-7 upgrade",
        ));
    }
    let receipt = if f.phase == TacticalPhase::Finished {
        super::release::require_finished_encounter(state)?;
        Some(ReleasedTimeUpgrade {
            origin: meta.clone(),
            release: state
                .encounter_history
                .as_ref()
                .and_then(|h| h.last())
                .ok_or_else(|| invalid("upgrade lacks release"))?
                .released_by
                .id,
            from: TacticalExecutionVersion::EncounterReleaseV1,
            to: TacticalExecutionVersion::ReleasedTimeV1,
        })
    } else {
        None
    };
    let f = flow_mut(state)?;
    f.version = TacticalExecutionVersion::ReleasedTimeV1.flow_version();
    f.released_time_upgrade = receipt;
    Ok(())
}

fn begin(
    state: &mut CampaignState,
    meta: &CommandMeta,
    seconds: u32,
    ordering: ReleasedTimeOrdering,
    ruling: &str,
) -> Result<(), RulesError> {
    if seconds == 0 || !valid_released_time_ruling(ruling) {
        return Err(invalid(
            "elapsed time requires positive seconds and a bounded single-line ruling",
        ));
    }
    if flow(state)?.version != TacticalExecutionVersion::ReleasedTimeV1.flow_version()
        || flow(state)?.phase != TacticalPhase::Finished
        || flow(state)?.resolution.is_some()
    {
        return Err(RulesError::Pending);
    }
    deadlines::preflight(state)?;
    let target_at = WorldInstant(
        state
            .clock
            .now
            .0
            .checked_add(i64::from(seconds))
            .ok_or_else(|| invalid("elapsed target overflows world time"))?,
    );
    let sources = deadlines::inventory(state)?;
    let mut instants = sources
        .iter()
        .filter(|(at, _)| *at <= target_at)
        .map(|(at, _)| *at)
        .collect::<Vec<_>>();
    instants.sort();
    instants.dedup();
    if instants.len() > 128 || sources.iter().filter(|(at, _)| *at <= target_at).count() > 32_768 {
        return Err(prerequisite(
            "elapsed interval exceeds bounded deadline capacity",
        ));
    }
    let history = state
        .encounter_history
        .as_ref()
        .ok_or_else(|| invalid("elapsed time lacks release history"))?;
    let context = ReleasedElapsedContext {
        release: history
            .last()
            .ok_or_else(|| invalid("elapsed time lacks release"))?
            .released_by
            .id,
        started_at: state.clock.now,
        target_at,
        progress_at: state.clock.now,
        ordering,
        ruling: ruling.into(),
        predecessor: history.elapsed_intervals.last().map(|r| r.origin.id),
        batches: vec![],
    };
    flow_mut(state)?.resolution = Some(Box::new(TacticalResolution {
        origin: meta.clone(),
        context: TacticalResolutionContext::ReleasedInterval(Box::new(context)),
        frames: vec![],
        pending: None,
        failed_save: None,
        legendary_window: None,
        attack: None,
        hit_review: None,
        movement: None,
        casts: vec![],
        missiles: vec![],
        falls: vec![],
        areas: vec![],
        work_trace: Some(TacticalWorkTrace::default()),
        next_occurrence: 0,
    }));
    pump(state, meta)
}

pub(super) fn pump(state: &mut CampaignState, meta: &CommandMeta) -> Result<(), RulesError> {
    for _ in 0..32_768 {
        while resolution(state)?.frames.last().is_some_and(Vec::is_empty) {
            resolution_mut(state)?.frames.pop();
        }
        if let Some(frame) = resolution(state)?.frames.last() {
            if frame.len() > 1 {
                return Ok(());
            }
            let work = resolution_mut(state)?
                .frames
                .last_mut()
                .and_then(Vec::pop)
                .ok_or_else(|| invalid("released work vanished"))?;
            super::continuations::start(state, meta, work)?;
            continue;
        }
        let target = resolution(state)?
            .released_interval()
            .ok_or_else(|| invalid("released context absent"))?
            .target_at;
        let sources = deadlines::inventory(state)?;
        let Some(at) = sources
            .iter()
            .map(|(at, _)| *at)
            .filter(|at| *at <= target)
            .min()
        else {
            state.clock.now = target;
            let r = flow_mut(state)?
                .resolution
                .take()
                .ok_or_else(|| invalid("released context vanished"))?;
            let TacticalResolutionContext::ReleasedInterval(interval) = r.context else {
                return Err(invalid("elapsed completion used turn context"));
            };
            state
                .encounter_history
                .as_mut()
                .ok_or_else(|| invalid("elapsed history absent"))?
                .elapsed_intervals
                .push(ReleasedElapsedReceipt {
                    origin: r.origin,
                    release: interval.release,
                    predecessor: interval.predecessor,
                    started_at: interval.started_at,
                    target_at: interval.target_at,
                    ordering: interval.ordering,
                    ruling: interval.ruling,
                    completed_by: meta.clone(),
                    completed_at: target,
                });
            return Ok(());
        };
        if at <= state.clock.now {
            return Err(invalid("unconsumed deadline survived its batch"));
        }
        let due = sources
            .into_iter()
            .filter(|(due, _)| *due == at)
            .collect::<Vec<_>>();
        state.clock.now = at;
        effect_operation(
            state,
            meta,
            EffectLifecycleOperation::Observe(EffectObservation::Time),
        )?;
        let observed = effects(state)?
            .last_operation
            .clone()
            .ok_or_else(|| invalid("Time observation lacks stamp"))?;
        let mut kinds = new_effect_work(state)?;
        for (_, source) in &due {
            match source {
                ReleasedDeadlineSource::Legacy { effect, .. } => {
                    kinds.push(TacticalWorkKind::ExpireLegacyEffect { effect: *effect })
                }
                ReleasedDeadlineSource::Stable { actor, .. } => {
                    kinds.push(TacticalWorkKind::RecoverStable { actor: *actor })
                }
                _ => {}
            }
        }
        push_frame(state, kinds)?;
        let frame = resolution(state)?
            .frames
            .last()
            .ok_or_else(|| invalid("deadline produced no work"))?;
        let mut bindings = Vec::new();
        for (_, source) in due {
            let mut matches = Vec::new();
            for work in frame {
                if deadlines::work_matches(state, &source, work)? {
                    matches.push(work);
                }
            }
            if matches.len() != 1 {
                return Err(invalid(
                    "deadline source does not own exactly one actual work item",
                ));
            }
            bindings.push(ReleasedDeadlineBinding {
                source,
                due_at: at,
                work: matches[0].clone(),
            });
        }
        if bindings.len() != frame.len() {
            return Err(invalid("Time observation produced unsupported work"));
        }
        let interval = resolution_mut(state)?
            .released_interval_mut()
            .ok_or_else(|| invalid("released context absent"))?;
        if interval.batches.len() >= 128 {
            return Err(invalid("deadline batch capacity"));
        }
        interval.progress_at = at;
        interval.batches.push(ReleasedDeadlineBatch {
            ordinal: interval.batches.len() as u16,
            at,
            observed,
            bindings,
            completions: vec![],
        });
    }
    Err(invalid("released consequence capacity exceeded"))
}

pub(super) fn choose(
    state: &mut CampaignState,
    meta: &CommandMeta,
    occurrence: u16,
) -> Result<(), RulesError> {
    authority(state, meta)?;
    let proof = ReleasedValidation::derive(state)?;
    proof.require_interval(state)?;
    let frame = resolution_mut(state)?
        .frames
        .last_mut()
        .ok_or_else(|| invalid("released siblings absent"))?;
    if frame.len() < 2 {
        return Err(prerequisite("no released ordering choice is due"));
    }
    let index = frame
        .iter()
        .position(|work| work.occurrence == occurrence)
        .ok_or_else(|| invalid("unknown released occurrence"))?;
    let work = frame.remove(index);
    super::continuations::start(state, meta, work)?;
    pump(state, meta)
}

pub(super) fn execute(
    state: &mut CampaignState,
    meta: &CommandMeta,
    work: &TacticalWorkItem,
) -> Result<(), RulesError> {
    let r = resolution(state)?;
    let interval = r
        .released_interval()
        .ok_or_else(|| invalid("released work lacks interval"))?;
    let batch = interval
        .batches
        .last()
        .ok_or_else(|| invalid("released work lacks batch"))?;
    let binding = batch
        .bindings
        .iter()
        .find(|binding| binding.work == *work)
        .ok_or_else(|| invalid("executed work lacks exact deadline source"))?;
    if r.work_trace
        .as_ref()
        .is_none_or(|trace| trace.active != Some(work.occurrence))
        || batch
            .completions
            .iter()
            .any(|c| c.work.occurrence == work.occurrence)
        || !deadlines::source_present(state, binding)?
    {
        return Err(invalid("released source is stale or not actually entered"));
    }
    let source = binding.source.clone();
    let key = TacticalWorkKey {
        resolution: r.origin.id,
        occurrence: work.occurrence,
    };
    match work.kind {
        TacticalWorkKind::Effect { ticket } => {
            let trigger = super::continuations::ticket(state, ticket)?;
            if !matches!(
                trigger.payload,
                EffectTriggerPayload::ExpireTargetEffect
                    | EffectTriggerPayload::ExpireConcentrationGroup
            ) {
                return Err(invalid("unsupported released effect payload"));
            }
            acknowledge(state, meta, ticket, EffectTriggerResolution::Apply)?;
        }
        TacticalWorkKind::RecoverStable { actor } => super::continuations::apply_vitality(
            state,
            meta,
            actor,
            work.occurrence,
            VitalityOperation::RecoverStable,
            None,
        )?,
        TacticalWorkKind::ExpireLegacyEffect { effect } => {
            let rules = state.rules.as_mut().ok_or(RulesError::Uninitialized)?;
            rules.effects.retain(|row| row.id != effect);
            for entity in rules.entities.values_mut() {
                if entity.concentration == Some(effect) {
                    entity.concentration = None;
                }
            }
        }
        _ => return Err(invalid("unsupported released work")),
    }
    let pending = &effects(state)?.pending;
    let cancelled = resolution(state)?.frames.iter().flatten().filter(|other| matches!(other.kind, TacticalWorkKind::Effect { ticket } if !pending.iter().any(|t| t.id == ticket))).cloned().collect::<Vec<_>>();
    for other in &cancelled {
        let binding = resolution(state)?
            .released_interval()
            .and_then(|i| i.batches.last())
            .and_then(|b| b.bindings.iter().find(|b| b.work == *other))
            .ok_or_else(|| invalid("cancelled sibling lacks source"))?;
        if !deadlines::cleanup_cancels(&source, &binding.source) {
            return Err(invalid("removed sibling has no exact cleanup cause"));
        }
    }
    let r = resolution_mut(state)?;
    for frame in &mut r.frames {
        frame.retain(|other| !cancelled.contains(other));
    }
    let batch = r
        .released_interval_mut()
        .and_then(|i| i.batches.last_mut())
        .ok_or_else(|| invalid("released batch vanished"))?;
    batch.completions.push(ReleasedWorkCompletion {
        work: key,
        completed_by: meta.clone(),
        outcome: ReleasedWorkOutcome::Applied,
    });
    for other in cancelled {
        batch.completions.push(ReleasedWorkCompletion {
            work: TacticalWorkKey {
                resolution: key.resolution,
                occurrence: other.occurrence,
            },
            completed_by: meta.clone(),
            outcome: ReleasedWorkOutcome::CancelledBy { work: key },
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests;
