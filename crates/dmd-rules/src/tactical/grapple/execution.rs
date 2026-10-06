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
    retired_flow: Option<RetiredFlow>,
}

/// Observed only at the shared Establish producer after the released encounter
/// has passed its existing replacement checks. Original replay keeps the old
/// decisions in their exact prior image; they are not copied into a new fight.
struct RetiredFlow {
    encounter: EncounterId,
    replacement: EncounterId,
    release: CommandId,
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
    closed: Option<&'a crate::table::execution::ClosedImage>,
}

impl<'a> ReadContext<'a> {
    pub(crate) fn closed(image: &'a crate::table::execution::ClosedImage) -> Self {
        Self {
            state: image.state(),
            guarded: None,
            closed: Some(image),
        }
    }

    pub(crate) fn ordinary(state: &'a CampaignState) -> Self {
        Self {
            state,
            guarded: None,
            closed: None,
        }
    }

    pub(crate) fn state(&self) -> &'a CampaignState {
        self.state
    }

    pub(crate) fn require_guarded(&self, message: &str) -> Result<(), RulesError> {
        if self.guarded.is_some() || self.closed.is_some() {
            Ok(())
        } else {
            Err(invalid(message))
        }
    }

    pub(in crate::tactical) fn attack_current(
        &self,
        actor: EntityId,
        target: EntityId,
        physical: bool,
    ) -> Result<Option<reads::AttackRead<'a>>, RulesError> {
        if self.guarded.is_some() || self.closed.is_some() {
            reads::AttackRead::current(self, actor, target, physical).map(Some)
        } else {
            Ok(None)
        }
    }

    pub(in crate::tactical) fn attack_retained(
        &self,
        attack: &TacticalAttack,
    ) -> Result<Option<reads::AttackRead<'a>>, RulesError> {
        if self.guarded.is_some() || self.closed.is_some() {
            reads::AttackRead::retained(self, attack).map(Some)
        } else {
            Ok(None)
        }
    }

    pub(crate) fn validate_recorded_grapple(&self, roll: &RecordedRoll) -> Result<(), RulesError> {
        if let Some(closed) = self.closed {
            return if closed
                .state()
                .rules
                .as_ref()
                .is_some_and(|rules| rules.rolls.contains(roll))
            {
                Ok(())
            } else {
                Err(invalid(
                    "Grapple raw is absent from its exact certified image",
                ))
            };
        }
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

    pub(in crate::tactical) fn opportunity_window(
        &self,
        attack: &TacticalAttack,
    ) -> Result<Option<reads::OpportunityWindowRead<'a>>, RulesError> {
        if crate::table::grapple_enabled(self.state) {
            reads::OpportunityWindowRead::retained(self, attack).map(Some)
        } else {
            Ok(None)
        }
    }

    pub(in crate::tactical) fn validate_retained_grapple_decision(
        &self,
        decision: &TacticalSaveDecision,
    ) -> Result<(), RulesError> {
        if !crate::table::grapple_enabled(self.state)
            || decision.key.role != TacticalRollRole::GrappleSave
        {
            return Err(invalid(
                "retained decision is not an activated Grapple save",
            ));
        }
        if let Some(closed) = self.closed {
            return if flow(closed.state())?.save_decisions.contains(decision) {
                Ok(())
            } else {
                Err(invalid(
                    "Grapple decision is absent from its exact certified image",
                ))
            };
        }
        let owned = self
            .guarded
            .ok_or_else(|| invalid("Grapple decision has no owned historical producer"))?;
        if flow(owned.predecessor)?.save_decisions.contains(decision)
            || owned.produced.decisions.contains(decision)
        {
            Ok(())
        } else {
            Err(invalid(
                "Grapple decision lacks its exact predecessor or observed producer",
            ))
        }
    }
}

impl<'owner> ExecutionContext<'owner> {
    pub(crate) fn for_table(
        predecessor: &'owner crate::table::execution::ClosedImage,
        candidate: &CampaignState,
        command: &'owner CommandMeta,
    ) -> Self {
        Self {
            guarded: Some(GuardedCommand {
                predecessor: predecessor.state(),
                candidate: std::ptr::from_ref(candidate),
                command,
                produced: ProducedEvidence::default(),
            }),
        }
    }

    pub(crate) fn is_owned(&self) -> bool {
        self.guarded.is_some()
    }
    pub(in crate::tactical) fn settle_work(
        &self,
        state: &mut CampaignState,
        meta: &CommandMeta,
    ) -> Result<(), RulesError> {
        self.check_state(state)?;
        if self.guarded.is_some() && crate::table::grapple_enabled(state) {
            modern_lifecycle::settle(state, meta)?;
        }
        Ok(())
    }

    pub(crate) fn admit_table(
        &self,
        state: &CampaignState,
        action: &TacticalAction,
    ) -> Result<(), RulesError> {
        self.check_state(state)?;
        if self.guarded.is_none() || !crate::table::grapple_enabled(state) {
            super::guard_action(state, action)?;
        } else if matches!(
            action,
            TacticalAction::Grapple { .. }
                | TacticalAction::MoveSelfOnly { .. }
                | TacticalAction::MoveGrappled { .. }
                | TacticalAction::ChooseGrappleSave { .. }
                | TacticalAction::ApplyGrappleAfterEquipment { .. }
                | TacticalAction::DeclineGrappleAfterEquipment { .. }
                | TacticalAction::WithdrawGrapple { .. }
                | TacticalAction::EscapeGrapple { .. }
                | TacticalAction::ReleaseGrapple { .. }
        ) {
            require_execution(state)?;
        }
        Ok(())
    }

    pub(crate) fn ordinary() -> Self {
        Self { guarded: None }
    }

    pub(in crate::tactical) fn observe_encounter_replacement(
        &mut self,
        state: &CampaignState,
        meta: &CommandMeta,
    ) -> Result<(), RulesError> {
        self.check_state(state)?;
        let Some(owned) = self.guarded.as_mut() else {
            return Ok(());
        };
        let old = encounter(owned.predecessor)?;
        let new = encounter(state)?;
        let receipt = owned
            .predecessor
            .encounter_history
            .as_deref()
            .and_then(TacticalEncounterHistory::last)
            .ok_or_else(|| invalid("replacement has no original completion"))?;
        if owned.command != meta
            || owned.produced.retired_flow.is_some()
            || old
                .flow
                .as_ref()
                .is_none_or(|flow| flow.phase != TacticalPhase::Finished)
            || receipt.encounter_id != old.id
            || receipt.encounter_origin != old.origin
            || new.id == old.id
            || new.origin != *meta
            || new.flow.is_some()
            || state.encounter_history != owned.predecessor.encounter_history
        {
            return Err(invalid("replacement differs from its released predecessor"));
        }
        owned.produced.retired_flow = Some(RetiredFlow {
            encounter: old.id,
            replacement: new.id,
            release: receipt.released_by.id,
        });
        Ok(())
    }

    /// Called only after the shared kernel reducer produced its fixed candidate.
    /// A table envelope cannot submit evidence or replace this observed source.
    pub(crate) fn observe_kernel_result(
        &mut self,
        state: &CampaignState,
        meta: &CommandMeta,
        action: &crate::RulesAction,
    ) -> Result<(), RulesError> {
        self.check_state(state)?;
        let Some(owned) = self.guarded.as_mut() else {
            return Ok(());
        };
        if owned.command != meta {
            return Err(invalid("kernel producer belongs to another command"));
        }
        let before = owned
            .predecessor
            .rules
            .as_ref()
            .ok_or(RulesError::Uninitialized)?;
        let after = state.rules.as_ref().ok_or(RulesError::Uninitialized)?;
        if let crate::RulesAction::SubmitRoll { result } = action {
            let pending = before.pending.as_ref().ok_or(RulesError::NoPending)?;
            if matches!(
                pending.purpose,
                PendingPurpose::TacticalInitiative { .. }
                    | PendingPurpose::TacticalResolution { .. }
            ) {
                return Err(invalid(
                    "kernel observation cannot attest tactical evidence",
                ));
            }
            let raw = after
                .rolls
                .last()
                .ok_or_else(|| invalid("kernel accepted raw absent"))?;
            if after.rolls.len() != before.rolls.len() + 1
                || !after.rolls.starts_with(&before.rolls)
                || raw.request != pending.request
                || raw.purpose != pending.purpose
                || raw.result != *result
                || raw.accepted_by != *meta
            {
                return Err(invalid(
                    "kernel evidence differs from its selected producer",
                ));
            }
            owned.produced.rolls.push(raw.clone());
        }
        Ok(())
    }

    pub(in crate::tactical) fn observe_initiative(
        &mut self,
        state: &CampaignState,
        meta: &CommandMeta,
        pending: &PendingRoll,
        roll: &RecordedRoll,
    ) -> Result<(), RulesError> {
        self.check_state(state)?;
        let Some(owned) = self.guarded.as_mut() else {
            return Ok(());
        };
        if owned.command != meta
            || state
                .rules
                .as_ref()
                .and_then(|rules| rules.pending.as_ref())
                != Some(pending)
            || !matches!(pending.purpose, PendingPurpose::TacticalInitiative { encounter: id, .. } if id == encounter(state)?.id)
            || roll.accepted_by != *meta
            || roll.issued_by != pending.issued_by
            || roll.request != pending.request
            || roll.purpose != pending.purpose
        {
            return Err(invalid(
                "initiative evidence differs from its selected producer",
            ));
        }
        owned.produced.rolls.push(roll.clone());
        Ok(())
    }

    pub(crate) fn read<'a>(
        &'a self,
        state: &'a CampaignState,
    ) -> Result<ReadContext<'a>, RulesError> {
        self.check_state(state)?;
        Ok(ReadContext {
            state,
            guarded: self.guarded.as_ref(),
            closed: None,
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
            admission::supported_context(state)?;
        }
        Ok(())
    }

    fn admit(&self, state: &CampaignState, action: &TacticalAction) -> Result<(), RulesError> {
        self.check_state(state)?;
        require_execution(state)?;
        admission::supported_context(state)?;
        // Only these shared producers participate in this private continuity
        // chain. Public dispatch and all unsupported nested families stay closed.
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
                | TacticalAction::ChooseTurnWork { .. }
                | TacticalAction::Attack { .. }
                | TacticalAction::CreatureWeaponAttack { .. }
                | TacticalAction::CreatureAttack { .. }
                | TacticalAction::UnarmedStrike { .. }
                | TacticalAction::SubmitSavageAttacker { .. }
                | TacticalAction::ChooseAttackKnockout { .. }
                | TacticalAction::ChooseAttackMastery { .. }
                | TacticalAction::RespondToHit { .. }
                | TacticalAction::OrderHitResponses { .. }
                | TacticalAction::DelegateHitResponses { .. }
                | TacticalAction::CastHitShield { .. }
                | TacticalAction::DeclineSelectedHitShield { .. }
        ) {
            return Err(prerequisite(
                "action is outside the private Grapple temporal profile",
            ));
        }
        if matches!(action, TacticalAction::Attack { choice } if choice.purpose != WeaponAttackPurpose::Normal)
        {
            return Err(prerequisite(
                "only normal own-turn weapon attacks are supported",
            ));
        }
        if matches!(
            action,
            TacticalAction::Attack { .. }
                | TacticalAction::CreatureWeaponAttack { .. }
                | TacticalAction::CreatureAttack { .. }
                | TacticalAction::UnarmedStrike { .. }
        ) && state
            .rules
            .as_ref()
            .and_then(|r| r.tactical_creatures.as_ref())
            .and_then(|c| c.runtime(active(state).ok()?))
            .is_some_and(|r| r.routine.is_some())
        {
            return Err(prerequisite(
                "source routines need their complete temporal adapter",
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
                    | TacticalAction::SubmitSavageAttacker { .. }
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

    pub(in crate::tactical) fn observe_savage_completion(
        &mut self,
        state: &CampaignState,
        meta: &CommandMeta,
        id: RollRequestId,
        savage: &SavageAttackerRoll,
    ) -> Result<(), RulesError> {
        self.check_state(state)?;
        let Some(owned) = self.guarded.as_mut() else {
            return Ok(());
        };
        let matches: Vec<_> = owned
            .produced
            .rolls
            .iter_mut()
            .filter(|raw| raw.request.id == id)
            .collect();
        if owned.command != meta || matches.len() != 1 {
            return Err(invalid("Savage completion does not own a new observed raw"));
        }
        let observed = matches
            .into_iter()
            .next()
            .ok_or_else(|| invalid("Savage observation absent"))?;
        let actual = state
            .rules
            .as_ref()
            .and_then(|r| r.rolls.iter().find(|raw| raw.request.id == id))
            .ok_or_else(|| invalid("new Savage raw absent"))?;
        if &*observed != actual
            || observed.accepted_by != *meta
            || observed.savage_attacker.is_some()
            || observed.original_result.is_some()
            || savage.weapon_dice.is_none()
            || !matches!(observed.purpose, PendingPurpose::TacticalResolution { key, .. } if key.role == TacticalRollRole::AttackDamage)
            || crate::kernel::savage_result(&observed.request, savage)? != observed.result
        {
            return Err(invalid(
                "Savage completion changes inherited or non-damage evidence",
            ));
        }
        observed.savage_attacker = Some(savage.clone());
        Ok(())
    }

    pub(crate) fn validate_delta(&self, state: &CampaignState) -> Result<(), RulesError> {
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
        let before_flow = owned
            .predecessor
            .encounter
            .as_ref()
            .and_then(|e| e.flow.as_ref());
        let after_flow = state.encounter.as_ref().and_then(|e| e.flow.as_ref());
        if let (Some(before_resolution), Some(after_resolution)) = (
            before_flow.and_then(|f| f.resolution.as_ref()),
            after_flow.and_then(|f| f.resolution.as_ref()),
        ) && before_resolution.origin == after_resolution.origin
        {
            let old_transport = before_resolution.grapple.as_ref().and_then(|c| c.transport.as_ref());
            let new_transport = after_resolution.grapple.as_ref().and_then(|c| c.transport.as_ref());
            match (old_transport, new_transport) {
                (None, None) => (),
                (Some(old), Some(new)) if old.admission == new.admission && new.steps.starts_with(&old.steps)
                    && old.stop.as_ref().is_none_or(|stop| new.stop.as_ref() == Some(stop)) => (),
                _ => return Err(invalid("ongoing resolution rewrote ground drag admission or history")),
            }
            if let Some(old) = &before_resolution.grapple {
                let new = after_resolution
                    .grapple
                    .as_ref()
                    .ok_or_else(|| invalid("ongoing resolution dropped its Grapple evidence"))?;
                if !new.cuts.starts_with(&old.cuts)
                    || !new.ends.starts_with(&old.ends)
                    || old.proofs.iter().any(|proof| !new.proofs.contains(proof))
                {
                    return Err(invalid(
                        "ongoing resolution rewrote inherited Grapple evidence",
                    ));
                }
            }
            if let Some(old) = &before_resolution.work_trace
                && after_resolution
                    .work_trace
                    .as_ref()
                    .is_none_or(|new| !new.nodes.starts_with(&old.nodes))
            {
                return Err(invalid("ongoing resolution rewrote actual work ancestry"));
            }
        }
        fn exact_suffix<T: PartialEq>(before: &[T], after: &[T], produced: &[T]) -> bool {
            after.len() == before.len() + produced.len()
                && after.starts_with(before)
                && &after[before.len()..] == produced
        }
        let decisions_match = if let Some(retired) = &owned.produced.retired_flow {
            owned
                .predecessor
                .encounter
                .as_ref()
                .is_some_and(|old| old.id == retired.encounter)
                && state.encounter.as_ref().is_some_and(|new| {
                    new.id == retired.replacement && new.origin == *owned.command
                })
                && before_flow.is_some_and(|flow| flow.phase == TacticalPhase::Finished)
                && after_flow.is_none()
                && owned.produced.decisions.is_empty()
                && state.encounter_history == owned.predecessor.encounter_history
                && state
                    .encounter_history
                    .as_deref()
                    .and_then(TacticalEncounterHistory::last)
                    .is_some_and(|receipt| {
                        receipt.encounter_id == retired.encounter
                            && receipt.released_by.id == retired.release
                    })
        } else {
            exact_suffix(
                before_flow.map_or(&[], |f| f.save_decisions.as_slice()),
                after_flow.map_or(&[], |f| f.save_decisions.as_slice()),
                &owned.produced.decisions,
            )
        };
        if !exact_suffix(&before.rolls, &after.rolls, &owned.produced.rolls)
            || !exact_suffix(
                &before.cancelled_roll_ids,
                &after.cancelled_roll_ids,
                &owned.produced.cancelled,
            )
            || !decisions_match
        {
            return Err(invalid(
                "private execution changed inherited or unobserved evidence",
            ));
        }
        if has_unimplemented_grapple_records(state) {
            validate(state)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod attack_tests;
#[cfg(test)]
mod tests;
