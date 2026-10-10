//! Independent-review regressions. Real private producers supply attainable
//! outcomes; explicitly reconstructed pre-retirement images are validator-only
//! counterexamples, never accepted history or public replay evidence.
use super::*;

fn pending_escape(player_owned: bool) -> (Fixture, GrappleId) {
    let mut f = Fixture::new();
    let (holder, target) = if player_owned {
        f.replace_target("goblin-warrior", CreatureSize::Small);
        let human = f.human;
        f.run(Some(human), |s, m| {
            super::super::super::turns::core_action(s, m, &TacticalAction::EndTurn)
        });
        (f.target, f.human)
    } else {
        (f.human, f.target)
    };
    let pack = f.pack.clone();
    f.run(Some(holder), |s, m| {
        begin(
            s,
            m,
            target,
            if player_owned {
                Hand::Right
            } else {
                Hand::Left
            },
            None,
            &pack,
        )
    });
    let id = attempt(&f.state).unwrap().declaration.id;
    f.run(Some(target), |s, m| {
        choose_save(s, m, id, GrappleSaveAbility::Dexterity)
    });
    f.submit_as(target, 1, RollSource::Physical);
    let work = work_key(
        &f.state,
        attempt(&f.state).unwrap().selected.as_ref().unwrap(),
    )
    .unwrap();
    f.run(Some(holder), |s, m| decline_after_equipment(s, m, id, work));
    f.run(Some(holder), |s, m| {
        super::super::super::turns::core_action(s, m, &TacticalAction::EndTurn)
    });
    f.run(Some(target), |s, m| {
        begin_escape(s, m, id, GrappleEscapeChoice::Acrobatics)
    });
    (f, id)
}

/// Restore only the discarded consumer for an isolated validation control. Its
/// paid origin/request/trace come from real private BeginEscape; accepted raw or
/// cancellation comes from the actual producer. This is not a durable pause.
fn retain_completed_escape(
    state: &CampaignState,
    mut paid: TacticalResolution,
    outcome: GrappleEscapeOutcome,
    end: Option<GrappleEndReceipt>,
) -> CampaignState {
    assert!(flow(state).unwrap().resolution.is_none());
    paid.pending = None;
    paid.frames.clear();
    let context = paid.grapple.as_mut().unwrap();
    let Some(GrappleActivity::Escape(escape)) = context.activity.as_mut() else {
        panic!("actual paid Escape required")
    };
    escape.stage = TacticalGrappleEscapeStage::Complete;
    escape.selected = None;
    escape.outcome = Some(outcome);
    if let Some(end) = end {
        context.ends.push(end);
    }
    let mut retained = state.clone();
    flow_mut(&mut retained).unwrap().resolution = Some(Box::new(paid));
    retained
}

#[test]
fn released_escape_rejects_an_end_between_establishment_and_escape_admission() {
    let (mut f, id) = pending_escape(false);
    let paid = resolution(&f.state).unwrap().clone();
    let escape_origin = escape(&f.state).unwrap().origin.clone();
    let request = escape(&f.state).unwrap().key.request_id();
    let established = context(&f.state).unwrap().proofs[0].established_by.clone();
    let holder = f.human;
    let released = f.run(Some(holder), |s, m| release(s, m, id));
    assert!(
        f.state
            .rules
            .as_ref()
            .unwrap()
            .timing
            .as_ref()
            .unwrap()
            .action_spent
    );
    assert_eq!(
        f.state
            .rules
            .as_ref()
            .unwrap()
            .cancelled_roll_ids
            .iter()
            .filter(|r| **r == request)
            .count(),
        1
    );
    let retained = retain_completed_escape(
        &f.state,
        paid,
        GrappleEscapeOutcome::Obsolete {
            ended_by: released.clone(),
            cancelled: Some(request),
        },
        Some(GrappleEndReceipt {
            grip: id,
            caused_by: released.clone(),
            cause: GrappleEndCause::Released,
        }),
    );
    validate(&retained).unwrap();

    // Deliberately forged pre-retirement metadata: both matching end references
    // still follow establishment, but cannot have canceled this later Escape.
    let mut forged = retained;
    let mut backdated = released;
    backdated.expected_event_sequence = established.expected_event_sequence + 1;
    assert!(backdated.expected_event_sequence < escape_origin.expected_event_sequence);
    context_mut(&mut forged).unwrap().ends[0].caused_by = backdated.clone();
    escape_mut(&mut forged).unwrap().outcome = Some(GrappleEscapeOutcome::Obsolete {
        ended_by: backdated,
        cancelled: Some(request),
    });
    let before = serde_json::to_vec(&forged).unwrap();
    assert!(
        validate(&forged)
            .unwrap_err()
            .to_string()
            .contains("chronology")
    );
    let meta = f.meta(Some(holder));
    assert!(
        release(&mut forged, &meta, id)
            .unwrap_err()
            .to_string()
            .contains("chronology")
    );
    assert_eq!(serde_json::to_vec(&forged).unwrap(), before);
}

#[test]
fn retired_live_grip_rejects_physical_overlap_before_release_can_erase_it() {
    let mut f = Fixture::new();
    let sword = f.loose_greatsword();
    let id = f.begin(Hand::Left, None);
    f.choose(id, GrappleSaveAbility::Strength);
    f.submit(1);
    f.decline(id);
    assert!(flow(&f.state).unwrap().resolution.is_none());
    validate(&f.state).unwrap();

    let mut forged = f.state.clone();
    let loadout = forged
        .rules
        .as_mut()
        .unwrap()
        .tactical_inventory
        .as_mut()
        .unwrap()
        .loadouts
        .iter_mut()
        .find(|l| l.actor == f.human)
        .unwrap();
    loadout.hands.hands[Hand::Left.index()] = HandAssignment::Item(sword);
    // This actual carried item is physically valid; the missing composition was
    // specifically the live reserved hand after the creating Attempt retired.
    crate::tactical_inventory::validate_loadout(
        &forged,
        admission::loadout(&forged, f.human).unwrap(),
    )
    .unwrap();
    let before = serde_json::to_vec(&forged).unwrap();
    assert!(
        validate(&forged)
            .unwrap_err()
            .to_string()
            .contains("overlaps a reserved hand")
    );
    let meta = f.meta(Some(f.human));
    assert!(
        release(&mut forged, &meta, id)
            .unwrap_err()
            .to_string()
            .contains("overlaps a reserved hand")
    );
    assert_eq!(serde_json::to_vec(&forged).unwrap(), before);
    assert_eq!(
        lifecycle::live_grip(&forged, id).unwrap(),
        lifecycle::live_grip(&f.state, id).unwrap()
    );
    f.run(Some(f.human), |s, m| release(s, m, id));
    assert!(f.state.rules.as_ref().unwrap().tactical_grapples.is_none());
}

#[test]
fn completed_escape_preserves_the_producers_physical_digital_and_secret_issuer_rule() {
    for (player_owned, source) in [
        (true, RollSource::Physical),
        (false, RollSource::Physical),
        (false, RollSource::Digital),
    ] {
        let (mut f, _) = pending_escape(player_owned);
        let paid = resolution(&f.state).unwrap().clone();
        let e = escape(&f.state).unwrap().clone();
        assert_eq!(
            e.request.as_ref().unwrap().visibility,
            if player_owned {
                RollVisibility::Public
            } else {
                RollVisibility::Secret
            }
        );
        let accepted = f.submit_as(e.actor, 1, source);
        assert_eq!(
            matches!(accepted.issuer, CommandIssuer::Player(_)),
            player_owned
        );
        let retained = retain_completed_escape(
            &f.state,
            paid,
            GrappleEscapeOutcome::Checked {
                resolved_by: accepted,
                succeeded: false,
            },
            None,
        );
        validate(&retained).unwrap();
        let accepted_raw = retained
            .rules
            .as_ref()
            .unwrap()
            .rolls
            .iter()
            .find(|r| r.request.id == e.key.request_id())
            .unwrap();
        assert_eq!(accepted_raw.result.source, source);
        assert_eq!(accepted_raw.resolved.source, source);
        if player_owned {
            // Same faces, request, arithmetic, owner and outcome. Forge both
            // source fields consistently so the issuer rule, not an arithmetic
            // or provenance mismatch, must reject this malformed snapshot.
            let mut forged = retained;
            let raw = forged
                .rules
                .as_mut()
                .unwrap()
                .rolls
                .iter_mut()
                .find(|r| r.request.id == e.key.request_id())
                .unwrap();
            let mut expected = raw.resolved.clone();
            assert_eq!(expected.source, RollSource::Physical);
            raw.result.source = RollSource::Digital;
            raw.resolved = raw.request.resolve(&raw.result).unwrap();
            expected.source = RollSource::Digital;
            assert_eq!(raw.resolved, expected);
            let before = serde_json::to_vec(&forged).unwrap();
            assert!(matches!(validate(&forged), Err(RulesError::Unauthorized)));
            let holder = context(&forged).unwrap().proofs[0].declaration.grappler;
            let meta = f.meta(Some(holder));
            assert!(matches!(
                release(&mut forged, &meta, e.grip),
                Err(RulesError::Unauthorized)
            ));
            assert_eq!(serde_json::to_vec(&forged).unwrap(), before);
        }
    }
}

#[test]
fn synthetic_final_evidence_cannot_omit_the_real_sources_required_lr_decision() {
    let mut f = Fixture::new();
    let id = f.begin(Hand::Left, None);
    f.choose(id, GrappleSaveAbility::Strength);
    f.submit(1);
    // A deliberately manufactured evidence-only counterexample. Replace the
    // target with the actual immutable Huge dragon and mutually adapt the
    // retained request/raw/proof source. Human->Huge admission is illegal and
    // this is NOT a positive LR pump, accepted save or replay fixture.
    f.replace_target("adult-red-dragon", CreatureSize::Huge);
    let pin = f
        .state
        .rules
        .as_ref()
        .unwrap()
        .tactical_creatures
        .as_ref()
        .unwrap()
        .profile(f.target)
        .unwrap()
        .source
        .clone();
    let mut grip = lifecycle::live_grip(&f.state, id).unwrap().clone();
    grip.declaration.target_source = Some(pin);
    let request = saves::expected_save(
        &f.state,
        &grip.declaration,
        grip.save.ability,
        grip.save.key,
    )
    .unwrap()
    .unwrap();
    grip.save.request = Some(request.clone());
    assert!(grip.save.proof.as_ref().unwrap().legendary.is_none());
    let raw = f
        .state
        .rules
        .as_mut()
        .unwrap()
        .rolls
        .iter_mut()
        .find(|r| r.request.id == request.id)
        .unwrap();
    raw.request = request;
    raw.resolved = raw.request.resolve(&raw.result).unwrap();
    assert!(raw.resolved.total < grip.declaration.escape_dc);
    let attempt = attempt_mut(&mut f.state).unwrap();
    attempt.declaration = grip.declaration.clone();
    attempt.save = Some(grip.save.clone());
    context_mut(&mut f.state).unwrap().proofs[0] = grip.clone();
    f.state
        .rules
        .as_mut()
        .unwrap()
        .tactical_grapples
        .as_mut()
        .unwrap()
        .active[0] = grip;
    assert!(super::super::super::failed_save::available(&f.state, f.target).unwrap());
    let before = serde_json::to_vec(&f.state).unwrap();
    assert!(
        validate(&f.state)
            .unwrap_err()
            .to_string()
            .contains("omitted its required LR decision")
    );
    assert_eq!(serde_json::to_vec(&f.state).unwrap(), before);
}

#[test]
fn before_weapon_use_resolves_all_outcomes_and_retires_without_an_after_allowance() {
    for (air, face, establishes) in [(false, 1, true), (false, 20, false), (true, 1, false)] {
        let mut f = Fixture::new();
        if air {
            f.replace_target("air-elemental", CreatureSize::Large);
        }
        let sword = f.loose_greatsword();
        let loadout = f
            .state
            .rules
            .as_mut()
            .unwrap()
            .tactical_inventory
            .as_mut()
            .unwrap()
            .loadouts
            .iter_mut()
            .find(|l| l.actor == f.human)
            .unwrap();
        loadout.hands.hands = [HandAssignment::Item(sword); 2];
        let before_equipment = admission::loadout(&f.state, f.human).unwrap().clone();
        let items = f.state.items.clone();
        let id = f.begin(
            Hand::Left,
            Some(AttackEquipmentOperation::Unequip { item: sword }),
        );
        let admitted = attempt(&f.state).unwrap().clone();
        assert_eq!(admitted.equipment.equipment_before, before_equipment);
        assert!(admitted.equipment.after.is_none());
        let paid_budget = flow(&f.state).unwrap().budget.clone();
        let raw_count = f.state.rules.as_ref().unwrap().rolls.len();
        f.choose(id, GrappleSaveAbility::Strength);
        let issued = f
            .state
            .rules
            .as_ref()
            .unwrap()
            .pending
            .as_ref()
            .unwrap()
            .clone();
        f.submit(face);
        // The real shared finish/pump validates Complete before immediately
        // retiring it. No synthetic after-choice is inserted to retain it.
        validate(&f.state).unwrap();
        assert!(flow(&f.state).unwrap().resolution.is_none());
        assert_eq!(flow(&f.state).unwrap().budget, paid_budget);
        assert_eq!(paid_budget.attacks_remaining, 0);
        assert_eq!(paid_budget.attack_window, Some(admitted.declaration.window));
        assert!(
            paid_budget.weapon_history.is_empty(),
            "no fake weapon receipt"
        );
        let rules = f.state.rules.as_ref().unwrap();
        assert!(rules.timing.as_ref().unwrap().action_spent);
        assert!(rules.pending.is_none());
        assert_eq!(rules.rolls.len(), raw_count + 1);
        let raw = rules
            .rolls
            .iter()
            .find(|r| r.request.id == issued.request.id)
            .unwrap();
        assert_eq!(raw.request, issued.request);
        assert_eq!(raw.issued_by, issued.issued_by);
        assert_eq!(raw.purpose, issued.purpose);
        assert_eq!(raw.result.dice[0].value, face);
        assert_eq!(
            crate::test_outcome::ability_test_success(
                &raw.resolved,
                admitted.declaration.escape_dc,
                &rules.house_rules
            )
            .unwrap(),
            face == 20
        );
        assert_eq!(
            rules.entities[&f.target]
                .condition_immunities
                .contains(&Condition::Grappled),
            air
        );
        assert_eq!(rules.tactical_grapples.is_some(), establishes);
        if establishes {
            let grip = lifecycle::live_grip(&f.state, id).unwrap();
            assert_eq!(grip.declaration, admitted.declaration);
            assert_eq!(grip.save.key.request_id(), issued.request.id);
            assert_eq!(grip.established_by, raw.accepted_by);
        }
        let after = admission::loadout(&f.state, f.human).unwrap();
        assert_eq!(after.hands.hands, [HandAssignment::Free; 2]);
        assert_eq!(after.command, admitted.declaration.origin);
        assert_eq!(after.worn_armor, before_equipment.worn_armor);
        assert_eq!(after.shield, before_equipment.shield);
        assert_eq!(f.state.items, items);
        let meta = f.meta(Some(f.human));
        let before = serde_json::to_vec(&f.state).unwrap();
        let invented_after = TacticalWorkKey {
            resolution: admitted.declaration.origin.id,
            occurrence: 2,
        };
        assert!(
            apply_after_equipment(
                &mut f.state,
                &meta,
                id,
                invented_after,
                AttackEquipmentOperation::Equip {
                    item: sword,
                    hand: Hand::Right
                }
            )
            .is_err()
        );
        assert_eq!(serde_json::to_vec(&f.state).unwrap(), before);
        assert!(decline_after_equipment(&mut f.state, &meta, id, invented_after).is_err());
        assert_eq!(serde_json::to_vec(&f.state).unwrap(), before);
    }
}
