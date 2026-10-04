//! Private continuity over the ordinary dispatcher, never imported save authority.
use super::*;

/// A normally validated baseline can enter this chain once. There is deliberately
/// no Clone, Deserialize, mutable state accessor, or replacement-state operation.
#[allow(dead_code)] // Private producer controls until the complete activation review.
pub(crate) struct GuardedGrappleExecution<'pack> {
    state: CampaignState,
    pack: &'pack RulesPack,
}

#[allow(dead_code)] // No public constructor or current application admission.
impl<'pack> GuardedGrappleExecution<'pack> {
    pub(crate) fn new(state: CampaignState, pack: &'pack RulesPack) -> Result<Self, RulesError> {
        crate::validate_state(&state, pack)?;
        super::super::validate_tactical_state(&state)?;
        require_execution(&state)?;
        if has_unimplemented_grapple_records(&state) || flow(&state)?.phase != TacticalPhase::Active
        {
            return Err(invalid(
                "private Grapple execution requires an ordinary active baseline",
            ));
        }
        Ok(Self { state, pack })
    }

    pub(crate) fn state(&self) -> &CampaignState {
        &self.state
    }

    pub(crate) fn into_state(self) -> CampaignState {
        self.state
    }

    pub(crate) fn apply(
        &mut self,
        meta: &CommandMeta,
        action: &TacticalAction,
    ) -> Result<TacticalEvent, RulesError> {
        if meta.campaign_id != self.state.campaign_id() {
            return Err(RulesError::Unauthorized);
        }
        if meta.expected_event_sequence != self.state.applied_event_sequence {
            return Err(RulesError::Stale);
        }
        let sequence = meta
            .expected_event_sequence
            .checked_add(1)
            .ok_or_else(|| invalid("private command sequence overflow"))?;
        // The candidate's address stays fixed for this command. Only this owner
        // constructs guarded authority; neither a caller nor a state clone can.
        let mut candidate = Box::new(self.state.clone());
        let mut execution = ExecutionContext {
            guarded: Some(GuardedCommand {
                predecessor: &self.state,
                candidate: std::ptr::from_ref(candidate.as_ref()),
                command: meta,
                produced: ProducedEvidence::default(),
            }),
        };
        execution.admit(&candidate, action)?;
        crate::kernel::validate_state_with_read(&execution.read(&candidate)?, self.pack)?;
        super::super::validation::validate_tactical_state_with_read(&execution.read(&candidate)?)?;
        super::super::dispatch(
            &self.state,
            &mut candidate,
            meta,
            action,
            self.pack,
            &mut execution,
        )?;
        execution.validate_delta(&candidate)?;
        crate::kernel::validate_state_with_read(&execution.read(&candidate)?, self.pack)?;
        super::super::validation::validate_tactical_state_with_read(&execution.read(&candidate)?)?;
        let outcome = super::super::outcome(&candidate)?;
        drop(execution);
        candidate.applied_event_sequence = sequence;
        self.state = *candidate;
        Ok(TacticalEvent {
            meta: meta.clone(),
            action: action.clone(),
            outcome,
        })
    }
}

#[derive(Default)]
struct ProducedEvidence {
    rolls: Vec<RecordedRoll>,
    decisions: Vec<TacticalSaveDecision>,
    cancelled: Vec<RollRequestId>,
}

struct GuardedCommand<'a> {
    predecessor: &'a CampaignState,
    // Compared for identity only; never dereferenced. Read borrows below carry
    // the actual lifetime and prevent retaining a view across a mutation.
    candidate: *const CampaignState,
    command: &'a CommandMeta,
    produced: ProducedEvidence,
}

/// Public dispatch uses the same implementation with ordinary authority. This
/// type is not Clone, and guarded construction remains private to the owner.
pub(crate) struct ExecutionContext<'a> {
    guarded: Option<GuardedCommand<'a>>,
}

pub(crate) struct ReadContext<'a> {
    state: &'a CampaignState,
    guarded: Option<&'a GuardedCommand<'a>>,
}

impl<'a> ReadContext<'a> {
    pub(crate) fn ordinary(state: &'a CampaignState) -> Self {
        Self {
            state,
            guarded: None,
        }
    }

    pub(crate) fn state(&self) -> &'a CampaignState {
        self.state
    }

    pub(crate) fn require_guarded(&self, message: &str) -> Result<(), RulesError> {
        self.guarded.ok_or_else(|| invalid(message)).map(|_| ())
    }

    pub(crate) fn validate_recorded_grapple(&self, roll: &RecordedRoll) -> Result<(), RulesError> {
        let owned = self
            .guarded
            .ok_or_else(|| invalid("Grapple raw history has no enabled producer"))?;
        let previous = owned
            .predecessor
            .rules
            .as_ref()
            .ok_or(RulesError::Uninitialized)?;
        if previous.rolls.contains(roll) || owned.produced.rolls.contains(roll) {
            Ok(())
        } else {
            Err(invalid(
                "Grapple raw lacks its owned predecessor or actual producer",
            ))
        }
    }
}

impl ExecutionContext<'_> {
    pub(crate) fn ordinary() -> Self {
        Self { guarded: None }
    }

    pub(crate) fn read<'a>(
        &'a self,
        state: &'a CampaignState,
    ) -> Result<ReadContext<'a>, RulesError> {
        self.check_state(state)?;
        Ok(ReadContext {
            state,
            guarded: self.guarded.as_ref(),
        })
    }

    fn check_state(&self, state: &CampaignState) -> Result<(), RulesError> {
        if self
            .guarded
            .as_ref()
            .is_some_and(|owned| !std::ptr::eq(owned.candidate, state))
        {
            return Err(invalid(
                "private execution context belongs to another candidate",
            ));
        }
        Ok(())
    }

    pub(in crate::tactical) fn check_live_constraints(
        &self,
        state: &CampaignState,
    ) -> Result<(), RulesError> {
        self.check_state(state)?;
        if self.guarded.is_some() {
            admission::live_constraints(state)?;
        }
        Ok(())
    }

    fn admit(&self, state: &CampaignState, action: &TacticalAction) -> Result<(), RulesError> {
        self.check_state(state)?;
        require_execution(state)?;
        admission::supported_context(state)?;
        // This intermediate owner checkpoint leaves attacks closed until their
        // complete read/producer/completion adapter is implemented and reviewed.
        if !matches!(
            action,
            TacticalAction::Grapple { .. }
                | TacticalAction::ChooseGrappleSave { .. }
                | TacticalAction::ApplyGrappleAfterEquipment { .. }
                | TacticalAction::DeclineGrappleAfterEquipment { .. }
                | TacticalAction::WithdrawGrapple { .. }
                | TacticalAction::EscapeGrapple { .. }
                | TacticalAction::ReleaseGrapple { .. }
                | TacticalAction::SubmitRoll { .. }
                | TacticalAction::SubmitRollWithInspiration { .. }
                | TacticalAction::VoluntarilyFailSave
                | TacticalAction::UseLegendaryResistance
                | TacticalAction::DeclineLegendaryResistance
                | TacticalAction::EndTurn
        ) {
            return Err(prerequisite(
                "private temporal attack/turn integration is not yet complete",
            ));
        }
        if state
            .rules
            .as_ref()
            .ok_or(RulesError::Uninitialized)?
            .pending
            .is_some()
            && !matches!(
                action,
                TacticalAction::SubmitRoll { .. }
                    | TacticalAction::SubmitRollWithInspiration { .. }
                    | TacticalAction::VoluntarilyFailSave
                    | TacticalAction::WithdrawGrapple { .. }
                    | TacticalAction::ReleaseGrapple { .. }
            )
        {
            return Err(RulesError::Pending);
        }
        Ok(())
    }

    /// Existing direct private-handler controls retain clone/validate/commit.
    /// Guarded dispatch already owns the candidate, so an inner clone must not
    /// acquire its continuity permission. Errors discard the outer candidate.
    pub(in crate::tactical) fn transaction(
        &mut self,
        state: &mut CampaignState,
        apply: impl FnOnce(&mut CampaignState, &mut Self) -> Result<(), RulesError>,
    ) -> Result<(), RulesError> {
        self.check_state(state)?;
        if self.guarded.is_some() {
            apply(state, self)?;
            validate(state)
        } else {
            let mut next = state.clone();
            apply(&mut next, self)?;
            validate(&next)?;
            *state = next;
            Ok(())
        }
    }

    fn producer(
        &mut self,
        state: &CampaignState,
        meta: &CommandMeta,
        pending: &TacticalPendingWork,
    ) -> Result<Option<&mut ProducedEvidence>, RulesError> {
        self.check_state(state)?;
        let Some(owned) = self.guarded.as_mut() else {
            return Ok(None);
        };
        if owned.command != meta
            || resolution(state)?.pending.as_ref() != Some(pending)
            || super::super::continuations::key(state, &pending.work)? != pending.key
        {
            return Err(invalid(
                "private evidence differs from its actual selected producer",
            ));
        }
        Ok(Some(&mut owned.produced))
    }

    pub(in crate::tactical) fn observe_roll(
        &mut self,
        state: &CampaignState,
        meta: &CommandMeta,
        pending: &TacticalPendingWork,
        roll: &RecordedRoll,
    ) -> Result<(), RulesError> {
        if let Some(produced) = self.producer(state, meta, pending)? {
            if roll.accepted_by != *meta
                || roll.request.id != pending.key.request_id()
                || !matches!(roll.purpose, PendingPurpose::TacticalResolution { key, encounter: id }
                    if key == pending.key && id == encounter(state)?.id)
            {
                return Err(invalid("private raw observation differs from its source"));
            }
            produced.rolls.push(roll.clone());
        }
        Ok(())
    }

    pub(in crate::tactical) fn observe_decision(
        &mut self,
        state: &CampaignState,
        meta: &CommandMeta,
        pending: &TacticalPendingWork,
        decision: &TacticalSaveDecision,
    ) -> Result<(), RulesError> {
        if let Some(produced) = self.producer(state, meta, pending)? {
            if decision.key != pending.key || decision.resolved_by != *meta {
                return Err(invalid("private save observation differs from its source"));
            }
            if decision.failure == TacticalSaveFailure::Voluntary {
                produced.cancelled.push(pending.key.request_id());
            }
            produced.decisions.push(decision.clone());
        }
        Ok(())
    }

    pub(in crate::tactical) fn observe_cancellation(
        &mut self,
        state: &CampaignState,
        meta: &CommandMeta,
        pending: &TacticalPendingWork,
    ) -> Result<(), RulesError> {
        if let Some(produced) = self.producer(state, meta, pending)? {
            produced.cancelled.push(pending.key.request_id());
        }
        Ok(())
    }

    fn validate_delta(&self, state: &CampaignState) -> Result<(), RulesError> {
        self.check_state(state)?;
        let Some(owned) = &self.guarded else {
            return Ok(());
        };
        let before = owned
            .predecessor
            .rules
            .as_ref()
            .ok_or(RulesError::Uninitialized)?;
        let after = state.rules.as_ref().ok_or(RulesError::Uninitialized)?;
        fn exact_suffix<T: PartialEq>(before: &[T], after: &[T], produced: &[T]) -> bool {
            after.len() == before.len() + produced.len()
                && after.starts_with(before)
                && &after[before.len()..] == produced
        }
        if !exact_suffix(&before.rolls, &after.rolls, &owned.produced.rolls)
            || !exact_suffix(
                &before.cancelled_roll_ids,
                &after.cancelled_roll_ids,
                &owned.produced.cancelled,
            )
            || !exact_suffix(
                &flow(owned.predecessor)?.save_decisions,
                &flow(state)?.save_decisions,
                &owned.produced.decisions,
            )
        {
            return Err(invalid(
                "private execution changed inherited or unobserved evidence",
            ));
        }
        validate(state)
    }
}

#[cfg(test)]
mod tests;
