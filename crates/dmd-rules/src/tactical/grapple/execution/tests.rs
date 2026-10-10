//! Private continuity controls, not original replay or public activation.
//! The existing pre-Grapple source image supplies identities and physical items;
//! actual ordinary upgrade/turn reducers establish the eligible baseline. After
//! owner construction every positive transition uses its typed shared dispatcher.
use super::*;

fn meta(state: &CampaignState, actor: Option<EntityId>) -> CommandMeta {
    CommandMeta {
        id: CommandId::new(),
        campaign_id: state.campaign_id(),
        session_id: state.encounter.as_ref().unwrap().origin.session_id,
        issuer: actor
            .and_then(|a| controller(state, a))
            .map_or(CommandIssuer::Admin, CommandIssuer::Player),
        actor: actor.map(AgentRef::Entity),
        expected_event_sequence: state.applied_event_sequence,
    }
}

fn public(
    state: &mut CampaignState,
    pack: &RulesPack,
    actor: Option<EntityId>,
    action: TacticalAction,
) {
    let command = meta(state, actor);
    let mut next = super::super::super::resolve_tactical(state, &command, &action, pack)
        .unwrap()
        .next_state;
    next.applied_event_sequence = command.expected_event_sequence.checked_add(1).unwrap();
    *state = next;
}

fn baseline(pack: &RulesPack) -> (CampaignState, EntityId, EntityId) {
    let mut state = crate::tactical_hands::tests::source_state();
    crate::validate_state(&state, pack).unwrap();
    let human = crate::tactical_hands::tests::human(&state);
    let target = state
        .rules
        .as_ref()
        .unwrap()
        .tactical_creatures
        .as_ref()
        .unwrap()
        .profiles[0]
        .actor;
    public(
        &mut state,
        pack,
        None,
        TacticalAction::UpgradeExecutionTo {
            execution: TacticalExecutionVersion::EncounterReleaseV1,
        },
    );
    // Controller setup is the real source scheduling operation in this labeled
    // private baseline; it does not claim a table event or replayed new history.
    let command = meta(&state, None);
    let source = state
        .rules
        .as_ref()
        .unwrap()
        .tactical_creatures
        .as_ref()
        .unwrap();
    let changed = crate::tactical_creatures::apply_creature_schedule(
        &state,
        source,
        &command,
        &crate::tactical_creatures::CreatureScheduleOperation::SetContext {
            actor: target,
            controller: CreatureController::Host,
            in_lair: false,
        },
    )
    .unwrap();
    state.rules.as_mut().unwrap().tactical_creatures = Some(changed.next);
    state.applied_event_sequence = command.expected_event_sequence.checked_add(1).unwrap();
    // The capture already spent the Human's first Action. Never reset its budget
    // or cursor by hand to manufacture another attack opportunity.
    public(&mut state, pack, Some(human), TacticalAction::EndTurn);
    public(&mut state, pack, Some(target), TacticalAction::EndTurn);
    crate::validate_state(&state, pack).unwrap();
    super::super::super::validate_tactical_state(&state).unwrap();
    assert_eq!(active(&state).unwrap(), human);
    assert!(!has_unimplemented_grapple_records(&state));
    (state, human, target)
}

fn apply(owner: &mut GuardedGrappleExecution<'_>, actor: EntityId, action: TacticalAction) {
    let command = meta(owner.state(), Some(actor));
    let sequence = command.expected_event_sequence;
    let event = owner.apply(&command, &action).unwrap();
    assert_eq!(event.meta, command);
    assert_eq!(event.action, action);
    assert_eq!(owner.state().applied_event_sequence, sequence + 1);
}

fn begin(owner: &mut GuardedGrappleExecution<'_>, human: EntityId, target: EntityId) -> GrappleId {
    apply(
        owner,
        human,
        TacticalAction::Grapple {
            target,
            hand: Hand::Left,
            before_change: None,
        },
    );
    attempt(owner.state()).unwrap().declaration.id
}

fn choose(owner: &mut GuardedGrappleExecution<'_>, target: EntityId, grip: GrappleId) {
    apply(
        owner,
        target,
        TacticalAction::ChooseGrappleSave {
            grip,
            ability: GrappleSaveAbility::Strength,
        },
    );
}

fn roll(owner: &GuardedGrappleExecution<'_>, face: u16) -> RollResult {
    let request = &owner
        .state()
        .rules
        .as_ref()
        .unwrap()
        .pending
        .as_ref()
        .unwrap()
        .request;
    RollResult {
        request_id: request.id,
        source: RollSource::Physical,
        dice: vec![
            DieResult {
                sides: 20,
                value: face
            };
            if request.mode == RollMode::Normal {
                1
            } else {
                2
            }
        ],
    }
}

fn decline(owner: &mut GuardedGrappleExecution<'_>, human: EntityId, grip: GrappleId) {
    let selected = attempt(owner.state()).unwrap().selected.as_ref().unwrap();
    let work = work_key(owner.state(), selected).unwrap();
    apply(
        owner,
        human,
        TacticalAction::DeclineGrappleAfterEquipment { grip, work },
    );
    assert!(flow(owner.state()).unwrap().resolution.is_none());
}

fn next_human_turn(owner: &mut GuardedGrappleExecution<'_>, human: EntityId, target: EntityId) {
    apply(owner, human, TacticalAction::EndTurn);
    apply(owner, target, TacticalAction::EndTurn);
    assert_eq!(active(owner.state()).unwrap(), human);
}

#[test]
fn ordinary_baseline_and_consuming_export_do_not_claim_history() {
    let pack = crate::tactical_hands::tests::pack();
    let (state, _, _) = baseline(&pack);
    let original = state.clone();
    let owner = GuardedGrappleExecution::new(state, &pack).unwrap();
    assert_eq!(owner.state(), &original);
    assert_eq!(owner.into_state(), original);
}

#[test]
fn typed_pending_withdrawal_preserves_exact_cancel_and_eligible_export() {
    let pack = crate::tactical_hands::tests::pack();
    let (state, human, target) = baseline(&pack);
    let mut owner = GuardedGrappleExecution::new(state, &pack).unwrap();
    let grip = begin(&mut owner, human, target);
    choose(&mut owner, target, grip);
    let id = owner
        .state()
        .rules
        .as_ref()
        .unwrap()
        .pending
        .as_ref()
        .unwrap()
        .request
        .id;
    let rolls = owner.state().rules.as_ref().unwrap().rolls.clone();
    let mut cancelled = owner
        .state()
        .rules
        .as_ref()
        .unwrap()
        .cancelled_roll_ids
        .clone();
    cancelled.push(id);
    apply(&mut owner, human, TacticalAction::WithdrawGrapple { grip });
    assert_eq!(
        attempt(owner.state()).unwrap().stage,
        TacticalGrappleAttemptStage::AfterEquipment
    );
    assert!(matches!(
        attempt(owner.state()).unwrap().outcome,
        Some(GrappleAttemptOutcome::Withdrawn { cancelled: Some(actual), .. }) if actual == id
    ));
    assert!(attempt(owner.state()).unwrap().reservation().is_none());
    let rules = owner.state().rules.as_ref().unwrap();
    assert!(rules.pending.is_none());
    assert!(rules.timing.as_ref().unwrap().action_spent);
    assert_eq!(rules.rolls, rolls);
    assert_eq!(rules.cancelled_roll_ids, cancelled);
    decline(&mut owner, human, grip);
    assert!(flow(owner.state()).unwrap().resolution.is_none());
    assert_eq!(
        owner.state().rules.as_ref().unwrap().cancelled_roll_ids,
        cancelled
    );
    assert_eq!(owner.state().rules.as_ref().unwrap().rolls, rolls);
    assert_eq!(
        owner
            .state()
            .rules
            .as_ref()
            .unwrap()
            .cancelled_roll_ids
            .last(),
        Some(&id)
    );
    assert!(!has_unimplemented_grapple_records(owner.state()));
    crate::validate_state(owner.state(), &pack).unwrap();
    let mut reentered = GuardedGrappleExecution::new(owner.into_state(), &pack).unwrap();
    next_human_turn(&mut reentered, human, target);
    begin(&mut reentered, human, target);
    assert_eq!(
        reentered
            .state()
            .rules
            .as_ref()
            .unwrap()
            .cancelled_roll_ids
            .last(),
        Some(&id)
    );
}

#[test]
fn real_completed_save_release_and_next_attempt_keep_raw_without_occupying_hand() {
    let pack = crate::tactical_hands::tests::pack();
    let (state, human, target) = baseline(&pack);
    let mut owner = GuardedGrappleExecution::new(state, &pack).unwrap();
    let grip = begin(&mut owner, human, target);
    choose(&mut owner, target, grip);
    let result = roll(&owner, 1);
    apply(&mut owner, target, TacticalAction::SubmitRoll { result });
    decline(&mut owner, human, grip);
    let inherited = owner.state().rules.as_ref().unwrap().rolls.clone();
    apply(&mut owner, human, TacticalAction::ReleaseGrapple { grip });
    assert!(
        owner
            .state()
            .rules
            .as_ref()
            .unwrap()
            .tactical_grapples
            .is_none()
    );
    assert_eq!(owner.state().rules.as_ref().unwrap().rolls, inherited);
    assert!(
        crate::tactical_hands::EffectiveHands::current(
            owner.state(),
            owner.state().rules.as_ref().unwrap(),
            human
        )
        .is_err()
    );
    assert!(crate::validate_state(owner.state(), &pack).is_err());
    assert!(super::super::super::validate_tactical_state(owner.state()).is_err());
    assert!(GuardedGrappleExecution::new(owner.state().clone(), &pack).is_err());
    next_human_turn(&mut owner, human, target);
    begin(&mut owner, human, target);
    assert_eq!(owner.state().rules.as_ref().unwrap().rolls, inherited);
}

#[test]
fn voluntary_pair_survives_retirement_and_public_import_stays_closed() {
    let pack = crate::tactical_hands::tests::pack();
    let (state, human, target) = baseline(&pack);
    let mut owner = GuardedGrappleExecution::new(state, &pack).unwrap();
    let grip = begin(&mut owner, human, target);
    choose(&mut owner, target, grip);
    let request = owner
        .state()
        .rules
        .as_ref()
        .unwrap()
        .pending
        .as_ref()
        .unwrap()
        .request
        .id;
    let raws = owner.state().rules.as_ref().unwrap().rolls.clone();
    apply(&mut owner, target, TacticalAction::VoluntarilyFailSave);
    decline(&mut owner, human, grip);
    apply(&mut owner, human, TacticalAction::ReleaseGrapple { grip });
    let decisions = flow(owner.state()).unwrap().save_decisions.clone();
    assert_eq!(
        decisions.last().unwrap().failure,
        TacticalSaveFailure::Voluntary
    );
    assert_eq!(owner.state().rules.as_ref().unwrap().rolls, raws);
    assert_eq!(
        owner
            .state()
            .rules
            .as_ref()
            .unwrap()
            .cancelled_roll_ids
            .last(),
        Some(&request)
    );
    assert!(GuardedGrappleExecution::new(owner.state().clone(), &pack).is_err());
    next_human_turn(&mut owner, human, target);
    begin(&mut owner, human, target);
    assert_eq!(flow(owner.state()).unwrap().save_decisions, decisions);
}

#[test]
fn real_escape_cancellation_is_owned_by_holder_and_keeps_earlier_save() {
    let pack = crate::tactical_hands::tests::pack();
    let (state, human, target) = baseline(&pack);
    let mut owner = GuardedGrappleExecution::new(state, &pack).unwrap();
    let grip = begin(&mut owner, human, target);
    choose(&mut owner, target, grip);
    let result = roll(&owner, 1);
    apply(&mut owner, target, TacticalAction::SubmitRoll { result });
    decline(&mut owner, human, grip);
    apply(&mut owner, human, TacticalAction::EndTurn);
    apply(
        &mut owner,
        target,
        TacticalAction::EscapeGrapple {
            grip,
            choice: GrappleEscapeChoice::Acrobatics,
        },
    );
    let request = owner
        .state()
        .rules
        .as_ref()
        .unwrap()
        .pending
        .as_ref()
        .unwrap()
        .request
        .id;
    let prior = owner.state().rules.as_ref().unwrap().rolls.clone();
    let before = owner.state().clone();
    let foreign = meta(owner.state(), Some(target));
    assert!(
        owner
            .apply(&foreign, &TacticalAction::ReleaseGrapple { grip })
            .is_err()
    );
    assert_eq!(owner.state(), &before);
    apply(&mut owner, human, TacticalAction::ReleaseGrapple { grip });
    assert!(flow(owner.state()).unwrap().resolution.is_none());
    assert_eq!(owner.state().rules.as_ref().unwrap().rolls, prior);
    assert_eq!(
        owner
            .state()
            .rules
            .as_ref()
            .unwrap()
            .cancelled_roll_ids
            .last(),
        Some(&request)
    );
}

#[test]
fn actual_successful_escape_retires_both_raw_roles_before_a_new_attempt() {
    let pack = crate::tactical_hands::tests::pack();
    let (state, human, target) = baseline(&pack);
    let mut owner = GuardedGrappleExecution::new(state, &pack).unwrap();
    let grip = begin(&mut owner, human, target);
    choose(&mut owner, target, grip);
    let result = roll(&owner, 1);
    apply(&mut owner, target, TacticalAction::SubmitRoll { result });
    decline(&mut owner, human, grip);
    apply(&mut owner, human, TacticalAction::EndTurn);
    apply(
        &mut owner,
        target,
        TacticalAction::EscapeGrapple {
            grip,
            choice: GrappleEscapeChoice::Acrobatics,
        },
    );
    let escaped = owner
        .state()
        .rules
        .as_ref()
        .unwrap()
        .pending
        .as_ref()
        .unwrap()
        .request
        .id;
    let result = roll(&owner, 20);
    apply(&mut owner, target, TacticalAction::SubmitRoll { result });
    assert!(flow(owner.state()).unwrap().resolution.is_none());
    assert!(
        owner
            .state()
            .rules
            .as_ref()
            .unwrap()
            .tactical_grapples
            .is_none()
    );
    let inherited = owner.state().rules.as_ref().unwrap().rolls.clone();
    assert!(matches!(inherited.last().unwrap().purpose,
        PendingPurpose::TacticalResolution { key, .. }
        if key.role == TacticalRollRole::GrappleEscape && key.request_id() == escaped));
    assert!(
        !owner
            .state()
            .rules
            .as_ref()
            .unwrap()
            .cancelled_roll_ids
            .contains(&escaped)
    );
    assert!(GuardedGrappleExecution::new(owner.state().clone(), &pack).is_err());
    apply(&mut owner, target, TacticalAction::EndTurn);
    begin(&mut owner, human, target);
    assert_eq!(owner.state().rules.as_ref().unwrap().rolls, inherited);
}

#[test]
fn private_producer_does_not_lift_either_public_dispatch_policy() {
    let pack = crate::tactical_hands::tests::pack();
    let (state, human, target) = baseline(&pack);
    let command = meta(&state, Some(human));
    for policy in [ExecutionPolicy::Live, ExecutionPolicy::Historical] {
        assert!(
            crate::tactical::resolve_with_policy(
                &state,
                &command,
                &TacticalAction::Grapple {
                    target,
                    hand: Hand::Left,
                    before_change: None
                },
                &pack,
                policy,
            )
            .is_err()
        );
    }
    let mut owner = GuardedGrappleExecution::new(state, &pack).unwrap();
    let grip = begin(&mut owner, human, target);
    choose(&mut owner, target, grip);
    let before = owner.state().clone();
    let command = meta(owner.state(), Some(target));
    let result = roll(&owner, 1);
    for policy in [ExecutionPolicy::Live, ExecutionPolicy::Historical] {
        assert!(
            crate::tactical::resolve_with_policy(
                owner.state(),
                &command,
                &TacticalAction::SubmitRoll {
                    result: result.clone()
                },
                &pack,
                policy,
            )
            .is_err()
        );
        assert_eq!(owner.state(), &before);
    }
}

#[test]
fn invalid_raw_and_stale_or_foreign_commands_leave_whole_owner_unchanged() {
    let pack = crate::tactical_hands::tests::pack();
    let (state, human, target) = baseline(&pack);
    let mut owner = GuardedGrappleExecution::new(state, &pack).unwrap();
    let grip = begin(&mut owner, human, target);
    choose(&mut owner, target, grip);
    let before = owner.state().clone();
    let command = meta(owner.state(), Some(target));
    let invalid = roll(&owner, 21);
    assert!(
        owner
            .apply(&command, &TacticalAction::SubmitRoll { result: invalid })
            .is_err()
    );
    assert_eq!(owner.state(), &before);
    let mut stale = command.clone();
    stale.expected_event_sequence -= 1;
    assert!(
        owner
            .apply(&stale, &TacticalAction::VoluntarilyFailSave)
            .is_err()
    );
    assert_eq!(owner.state(), &before);
    let mut foreign = command;
    foreign.campaign_id = CampaignId::new();
    assert!(
        owner
            .apply(&foreign, &TacticalAction::VoluntarilyFailSave)
            .is_err()
    );
    assert_eq!(owner.state(), &before);
    let holder = meta(owner.state(), Some(human));
    assert!(
        owner
            .apply(&holder, &TacticalAction::VoluntarilyFailSave)
            .is_err()
    );
    assert_eq!(owner.state(), &before);
}

#[test]
fn pending_and_live_exports_cannot_import_private_authority() {
    let pack = crate::tactical_hands::tests::pack();
    let (state, human, target) = baseline(&pack);
    let mut owner = GuardedGrappleExecution::new(state, &pack).unwrap();
    let grip = begin(&mut owner, human, target);
    assert!(GuardedGrappleExecution::new(owner.state().clone(), &pack).is_err());
    choose(&mut owner, target, grip);
    let exported = serde_json::to_string(owner.state()).unwrap();
    let restored: CampaignState = serde_json::from_str(&exported).unwrap();
    assert!(GuardedGrappleExecution::new(restored, &pack).is_err());
    let result = roll(&owner, 1);
    apply(&mut owner, target, TacticalAction::SubmitRoll { result });
    decline(&mut owner, human, grip);
    assert!(GuardedGrappleExecution::new(owner.into_state(), &pack).is_err());
}

#[test]
fn foreign_candidate_read_and_unobserved_ordered_deltas_are_rejected() {
    let pack = crate::tactical_hands::tests::pack();
    let (state, _, _) = baseline(&pack);
    let owner = GuardedGrappleExecution::new(state, &pack).unwrap();
    // Mechanism negatives only. This inaccessible command context is never used
    // to commit a state or certify synthetic producer evidence.
    let command = meta(owner.state(), None);
    let mut candidate = Box::new(owner.state().clone());
    let context = ExecutionContext {
        mass: None,
        guarded: Some(GuardedCommand {
            predecessor: owner.state(),
            candidate: std::ptr::from_ref(candidate.as_ref()),
            command: &command,
            produced: ProducedEvidence::default(),
        }),
    };
    let foreign = candidate.as_ref().clone();
    assert!(context.read(&foreign).is_err());
    assert!(context.read(&candidate).is_ok());
    let prior = candidate.rules.as_ref().unwrap().rolls.clone();
    assert!(prior.len() >= 2);
    candidate.rules.as_mut().unwrap().rolls.swap(0, 1);
    assert!(context.validate_delta(&candidate).is_err());
    candidate.rules.as_mut().unwrap().rolls = prior;
    candidate
        .rules
        .as_mut()
        .unwrap()
        .cancelled_roll_ids
        .push(RollRequestId::new());
    assert!(context.validate_delta(&candidate).is_err());
    assert_eq!(
        owner.state().applied_event_sequence,
        command.expected_event_sequence
    );
}
