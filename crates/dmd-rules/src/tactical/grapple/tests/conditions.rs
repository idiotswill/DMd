//! Real private producers on the parent's labeled executor scaffold. Queries on
//! altered relation topology are explicitly synthetic and never admitted play.
use super::*;
use crate::tactical_conditions::{AttackPerception, DodgeContext};

fn dodging_target(definition: Option<(&str, CreatureSize)>) -> Fixture {
    let mut f = Fixture::new();
    if let Some((definition, size)) = definition {
        f.replace_target(definition, size);
    }
    let (holder, target) = (f.human, f.target);
    f.run(Some(holder), |s, m| {
        super::super::super::turns::core_action(s, m, &TacticalAction::EndTurn)
    });
    assert_eq!(active(&f.state).unwrap(), target);
    let origin = f.run(Some(target), |s, m| {
        super::super::super::turns::core_action(s, m, &TacticalAction::Dodge)
    });
    assert!(
        flow(&f.state)
            .unwrap()
            .dodges
            .iter()
            .any(|d| d.origin == origin)
    );
    f.run(Some(target), |s, m| {
        super::super::super::turns::core_action(s, m, &TacticalAction::EndTurn)
    });
    assert_eq!(active(&f.state).unwrap(), holder);
    assert!(dodge_context(&f.state, target).unwrap().declared);
    f
}

fn submit_faces(f: &mut Fixture, faces: &[u16]) {
    let pending = f.state.rules.as_ref().unwrap().pending.as_ref().unwrap();
    let result = RollResult {
        request_id: pending.request.id,
        source: RollSource::Physical,
        dice: faces
            .iter()
            .map(|&value| DieResult { sides: 20, value })
            .collect(),
    };
    let target = f.target;
    f.run(Some(target), |s, m| {
        super::super::super::continuations::submit(s, m, &result, None)
    });
}

fn views(state: &CampaignState) -> Vec<ActiveEffect> {
    crate::tactical_effect_adapter::condition_effects(state.rules.as_ref().unwrap())
        .filter(|e| e.condition == Some(Condition::Grappled))
        .collect()
}

pub(super) fn assert_public_and_raw_guards(f: &Fixture, state: &CampaignState) {
    for raw_only in [false, true] {
        let mut image = state.clone();
        if raw_only {
            image.rules.as_mut().unwrap().tactical_grapples = None;
            flow_mut(&mut image).unwrap().resolution = None;
            assert!(views(&image).is_empty());
        }
        for version in 1..=5 {
            flow_mut(&mut image).unwrap().version = version;
            let before = serde_json::to_vec(&image).unwrap();
            assert!(
                crate::validate_state(&image, &f.pack)
                    .unwrap_err()
                    .to_string()
                    .contains("Grapple")
            );
            assert!(
                crate::tactical::validate_tactical_state(&image)
                    .unwrap_err()
                    .to_string()
                    .contains("Grapple")
            );
            for policy in [ExecutionPolicy::Live, ExecutionPolicy::Historical] {
                assert!(
                    crate::tactical::resolve_with_policy(
                        &image,
                        &f.meta(Some(f.human)),
                        &TacticalAction::EndTurn,
                        &f.pack,
                        policy,
                    )
                    .is_err()
                );
                assert_eq!(serde_json::to_vec(&image).unwrap(), before);
            }
        }
    }
}

#[test]
fn real_dodge_ends_before_after_equipment_and_release_cannot_restore_it() {
    for equip_after in [false, true] {
        let mut f = dodging_target(None);
        let sword = f.loose_greatsword();
        let id = f.begin(Hand::Left, None);
        assert!(views(&f.state).is_empty(), "a reservation is not Grappled");
        f.choose(id, GrappleSaveAbility::Dexterity);
        let issued = attempt(&f.state).unwrap().save.as_ref().unwrap().clone();
        assert_eq!(issued.request.as_ref().unwrap().mode, RollMode::Advantage);
        let mechanics = f.state.rules.as_ref().unwrap().entities[&f.target].clone();
        let items = f.state.items.clone();
        let effects = f.state.rules.as_ref().unwrap().effects.clone();
        let lifecycle_effects = f.state.rules.as_ref().unwrap().tactical_effects.clone();
        let participant = encounter(&f.state)
            .unwrap()
            .participant(f.target)
            .unwrap()
            .clone();
        let perception =
            crate::spatial::perceive(encounter(&f.state).unwrap(), &f.state, f.human, f.target)
                .unwrap();
        submit_faces(&mut f, &[1, 2]);
        let completed = attempt(&f.state).unwrap().save.as_ref().unwrap().clone();
        assert_eq!(completed.request, issued.request);
        assert!(!completed.proof.as_ref().unwrap().final_success);
        assert_eq!(
            attempt(&f.state).unwrap().stage,
            TacticalGrappleAttemptStage::AfterEquipment
        );
        let raw = f
            .state
            .rules
            .as_ref()
            .unwrap()
            .rolls
            .iter()
            .find(|r| r.request.id == issued.key.request_id())
            .unwrap()
            .clone();
        assert_eq!(
            raw.result.dice.iter().map(|d| d.value).collect::<Vec<_>>(),
            [1, 2]
        );
        assert_eq!(raw.resolved, raw.request.resolve(&raw.result).unwrap());
        assert!(!dodge_context(&f.state, f.target).unwrap().declared);
        assert_eq!(speeds(&f.state, f.target).unwrap().walk, 0);
        let projected = views(&f.state);
        assert_eq!(projected.len(), 1);
        assert_eq!(
            (projected[0].source, projected[0].target),
            (f.human, f.target)
        );
        assert_eq!(projected, views(&f.state), "query identity is stable");
        let rules = f.state.rules.as_ref().unwrap();
        assert_eq!(
            rules.entities[&f.target], mechanics,
            "no HP, Prone or concentration mutation"
        );
        assert_eq!(f.state.items, items);
        assert_eq!(rules.effects, effects);
        assert_eq!(rules.tactical_effects, lifecycle_effects);
        assert_eq!(
            encounter(&f.state).unwrap().participant(f.target).unwrap(),
            &participant
        );
        assert_eq!(
            crate::spatial::perceive(encounter(&f.state).unwrap(), &f.state, f.human, f.target)
                .unwrap(),
            perception
        );
        let conditions = crate::active_conditions(rules, f.target);
        assert!(conditions.contains(&Condition::Grappled));
        assert!(!conditions.contains(&Condition::Prone));
        assert!(!conditions.contains(&Condition::Incapacitated));
        assert!(crate::tactical_conditions::can_act(rules, f.target).unwrap());
        assert!(crate::tactical_conditions::can_speak(rules, f.target).unwrap());
        let work = work_key(
            &f.state,
            attempt(&f.state).unwrap().selected.as_ref().unwrap(),
        )
        .unwrap();
        let holder = f.human;
        f.run(Some(holder), |s, m| release(s, m, id));
        assert!(
            views(&f.state).is_empty(),
            "ended retained proof is not Grappled"
        );
        assert_eq!(context(&f.state).unwrap().proofs[0].save, completed);
        assert_eq!(
            attempt(&f.state).unwrap().outcome,
            Some(GrappleAttemptOutcome::Established { grip: id })
        );
        assert_eq!(
            context(&f.state).unwrap().ends[0].cause,
            GrappleEndCause::Released
        );
        assert!(!dodge_context(&f.state, f.target).unwrap().declared);
        assert!(speeds(&f.state, f.target).unwrap().walk > 0);
        // A current query can derive Normal after release, but may not replace
        // the issued Advantage request or claim new public request authority.
        let a = attempt(&f.state).unwrap();
        assert_eq!(
            saves::expected_save(&f.state, &a.declaration, completed.ability, completed.key)
                .unwrap()
                .unwrap()
                .mode,
            RollMode::Normal
        );
        if equip_after {
            f.run(Some(holder), |s, m| {
                apply_after_equipment(
                    s,
                    m,
                    id,
                    work,
                    AttackEquipmentOperation::Equip {
                        item: sword,
                        hand: Hand::Left,
                    },
                )
            });
        } else {
            f.decline(id);
        }
        assert!(flow(&f.state).unwrap().resolution.is_none());
        assert!(views(&f.state).is_empty());
        assert!(!dodge_context(&f.state, f.target).unwrap().declared);
        assert_eq!(
            f.state
                .rules
                .as_ref()
                .unwrap()
                .rolls
                .iter()
                .find(|r| r.request.id == issued.key.request_id())
                .unwrap(),
            &raw
        );
        assert_public_and_raw_guards(&f, &f.state);
    }
}

#[test]
fn before_used_dodge_save_survives_creating_resolution_retirement_and_escape() {
    let mut f = dodging_target(None);
    let dagger = f.item("dagger");
    let id = f.begin(
        Hand::Right,
        Some(AttackEquipmentOperation::Unequip { item: dagger }),
    );
    f.choose(id, GrappleSaveAbility::Dexterity);
    let issued = attempt(&f.state).unwrap().save.as_ref().unwrap().clone();
    assert_eq!(issued.request.as_ref().unwrap().mode, RollMode::Advantage);
    submit_faces(&mut f, &[1, 2]);
    assert!(flow(&f.state).unwrap().resolution.is_none());
    validate(&f.state).unwrap();
    let grip = lifecycle::live_grip(&f.state, id).unwrap();
    assert_eq!(grip.save.request, issued.request);
    assert!(grip.save.proof.is_some());
    assert_eq!(views(&f.state).len(), 1);
    assert!(!dodge_context(&f.state, f.target).unwrap().declared);
    let (holder, target) = (f.human, f.target);
    f.run(Some(holder), |s, m| {
        super::super::super::turns::core_action(s, m, &TacticalAction::EndTurn)
    });
    f.run(Some(target), |s, m| {
        begin_escape(s, m, id, GrappleEscapeChoice::Acrobatics)
    });
    f.submit(20);
    assert!(views(&f.state).is_empty());
    assert!(!dodge_context(&f.state, target).unwrap().declared);
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
    assert_public_and_raw_guards(&f, &f.state);
}

#[test]
fn actual_resistance_immunity_and_unfinished_withdrawal_keep_dodge() {
    for route in 0..4 {
        let mut f = dodging_target((route == 1).then_some(("air-elemental", CreatureSize::Large)));
        let dodges = flow(&f.state).unwrap().dodges.clone();
        let id = f.begin(Hand::Left, None);
        if route != 2 {
            f.choose(id, GrappleSaveAbility::Dexterity);
        }
        if route >= 2 {
            let holder = f.human;
            f.run(Some(holder), |s, m| withdraw(s, m, id));
        } else {
            submit_faces(&mut f, if route == 1 { &[1, 2] } else { &[19, 20] });
            assert!(
                matches!(
                    attempt(&f.state).unwrap().outcome,
                    Some(GrappleAttemptOutcome::Immune { .. })
                ) == (route == 1)
            );
        }
        assert!(views(&f.state).is_empty());
        assert_eq!(flow(&f.state).unwrap().dodges, dodges);
        f.decline(id);
        assert!(views(&f.state).is_empty());
        assert_eq!(flow(&f.state).unwrap().dodges, dodges);
    }
}

#[test]
fn unfinished_dodge_request_cannot_copy_a_different_current_mode() {
    let mut f = dodging_target(None);
    let id = f.begin(Hand::Left, None);
    f.choose(id, GrappleSaveAbility::Dexterity);
    let mut forged = f.state.clone();
    attempt_mut(&mut forged)
        .unwrap()
        .save
        .as_mut()
        .unwrap()
        .request
        .as_mut()
        .unwrap()
        .mode = RollMode::Normal;
    forged
        .rules
        .as_mut()
        .unwrap()
        .pending
        .as_mut()
        .unwrap()
        .request
        .mode = RollMode::Normal;
    let before = serde_json::to_vec(&forged).unwrap();
    assert!(validate(&forged).is_err());
    let meta = f.meta(Some(f.human));
    assert!(withdraw(&mut forged, &meta, id).is_err());
    assert_eq!(serde_json::to_vec(&forged).unwrap(), before);
}

#[test]
fn completed_dodge_evidence_still_rejects_nonmatching_proof_raw_shape_and_ownership() {
    let mut f = dodging_target(None);
    let id = f.begin(Hand::Left, None);
    f.choose(id, GrappleSaveAbility::Dexterity);
    submit_faces(&mut f, &[1, 2]);
    for case in 0..16 {
        let mut forged = f.state.clone();
        let mut save = attempt(&forged).unwrap().save.as_ref().unwrap().clone();
        let request_id = save.key.request_id();
        match case {
            0 => save.proof = None,
            1 => save.proof.as_mut().unwrap().final_success = true,
            2 => save.key.role = TacticalRollRole::GrappleEscape,
            3 => save.request.as_mut().unwrap().dice[0].sides = 6,
            4 => save.request.as_mut().unwrap().roller = Some(f.human),
            5 => save.chosen_by.actor = Some(AgentRef::Entity(f.human)),
            6 => save.proof.as_mut().unwrap().finalized_by.actor = Some(AgentRef::Entity(f.human)),
            7 => forged
                .rules
                .as_mut()
                .unwrap()
                .cancelled_roll_ids
                .push(request_id),
            8 => {
                forged
                    .rules
                    .as_mut()
                    .unwrap()
                    .rolls
                    .iter_mut()
                    .find(|r| r.request.id == request_id)
                    .unwrap()
                    .result
                    .dice
                    .pop();
            }
            9 => {
                forged
                    .rules
                    .as_mut()
                    .unwrap()
                    .rolls
                    .iter_mut()
                    .find(|r| r.request.id == request_id)
                    .unwrap()
                    .request
                    .modifier += 1;
            }
            10 => {
                forged
                    .rules
                    .as_mut()
                    .unwrap()
                    .rolls
                    .iter_mut()
                    .find(|r| r.request.id == request_id)
                    .unwrap()
                    .issued_by
                    .id = CommandId::new();
            }
            11 => attempt_mut(&mut forged).unwrap().stage = TacticalGrappleAttemptStage::Saving,
            12 => {
                attempt_mut(&mut forged).unwrap().outcome = Some(GrappleAttemptOutcome::Resisted {
                    resolved_by: save.proof.as_ref().unwrap().finalized_by.clone(),
                })
            }
            13 => {
                let raw = forged
                    .rules
                    .as_mut()
                    .unwrap()
                    .rolls
                    .iter_mut()
                    .find(|r| r.request.id == request_id)
                    .unwrap();
                let PendingPurpose::TacticalResolution { key, .. } = &mut raw.purpose else {
                    panic!("actual tactical result required")
                };
                key.role = TacticalRollRole::GrappleEscape;
            }
            14 => {
                forged
                    .rules
                    .as_mut()
                    .unwrap()
                    .rolls
                    .iter_mut()
                    .find(|r| r.request.id == request_id)
                    .unwrap()
                    .accepted_by
                    .actor = Some(AgentRef::Entity(f.human));
            }
            15 => {
                forged
                    .rules
                    .as_mut()
                    .unwrap()
                    .rolls
                    .iter_mut()
                    .find(|r| r.request.id == request_id)
                    .unwrap()
                    .resolved
                    .total += 1;
            }
            _ => unreachable!(),
        }
        // Copying the altered save into every retained location cannot repair
        // its absent/raw-inconsistent/unauthorized evidence or wrong stage.
        attempt_mut(&mut forged).unwrap().save = Some(save.clone());
        context_mut(&mut forged).unwrap().proofs[0].save = save.clone();
        forged
            .rules
            .as_mut()
            .unwrap()
            .tactical_grapples
            .as_mut()
            .unwrap()
            .active[0]
            .save = save;
        let before = serde_json::to_vec(&forged).unwrap();
        assert!(validate(&forged).is_err(), "case {case}");
        let meta = f.meta(Some(f.human));
        assert!(release(&mut forged, &meta, id).is_err(), "case {case}");
        assert_eq!(serde_json::to_vec(&forged).unwrap(), before, "case {case}");
    }
}

#[test]
fn final_flight_refusal_preserves_the_entire_private_command_input() {
    let mut f = Fixture::new();
    f.replace_target("young-red-dragon", CreatureSize::Large);
    let id = f.begin(Hand::Left, None);
    f.choose(id, GrappleSaveAbility::Dexterity);
    // Explicitly synthetic pre-finalization image: lift the actual non-Hover
    // source a half-foot and align the retained position, without changing its
    // source profile. This probes the final guard, not legal motion/history.
    let participant = f
        .state
        .encounter
        .as_mut()
        .unwrap()
        .participants
        .iter_mut()
        .find(|p| p.entity_id == f.target)
        .unwrap();
    participant.position.z = 1;
    let lifted = participant.position;
    attempt_mut(&mut f.state).unwrap().declaration.target_from = lifted;
    let raw = f.state.rules.as_ref().unwrap().pending.as_ref().unwrap();
    let result = RollResult {
        request_id: raw.request.id,
        source: RollSource::Physical,
        dice: vec![DieResult {
            sides: 20,
            value: 1,
        }],
    };
    let before = serde_json::to_vec(&f.state).unwrap();
    let meta = f.meta(Some(f.target));
    // Same clone/validate transaction as guarded commands; submit itself is an
    // internal mutation owned by the outer resolver transaction.
    let error = transaction(&mut f.state, &meta, |next| {
        super::super::super::continuations::submit(next, &meta, &result, None)
    })
    .unwrap_err();
    assert!(error.to_string().contains("flight loss"), "{error}");
    assert_eq!(serde_json::to_vec(&f.state).unwrap(), before);
    assert!(views(&f.state).is_empty());
    assert!(resolution(&f.state).unwrap().falls.is_empty());
}

#[test]
fn live_condition_does_not_lift_holder_break_or_range_guards() {
    let mut f = dodging_target(None);
    let id = f.begin(Hand::Left, None);
    f.choose(id, GrappleSaveAbility::Dexterity);
    submit_faces(&mut f, &[1, 2]);
    for case in 0..3 {
        let mut image = f.state.clone();
        match case {
            0 => {
                image
                    .rules
                    .as_mut()
                    .unwrap()
                    .entities
                    .get_mut(&f.human)
                    .unwrap()
                    .death
                    .dead = true
            }
            1 => image.rules.as_mut().unwrap().effects.push(ActiveEffect {
                id: EffectId::new(),
                source: f.target,
                target: f.human,
                condition: Some(Condition::Incapacitated),
                label: "Synthetic holder-break refusal".into(),
                expires: Expiry::Never,
                concentration_owner: None,
            }),
            2 => {
                image
                    .encounter
                    .as_mut()
                    .unwrap()
                    .participants
                    .iter_mut()
                    .find(|p| p.entity_id == f.target)
                    .unwrap()
                    .position
                    .x += 100
            }
            _ => unreachable!(),
        }
        let before = serde_json::to_vec(&image).unwrap();
        let meta = f.meta(Some(f.human));
        let error = release(&mut image, &meta, id).unwrap_err();
        assert!(error.to_string().contains(if case == 2 {
            "Range breaks"
        } else {
            "Holder breaks"
        }));
        assert_eq!(serde_json::to_vec(&image).unwrap(), before);
    }
}

#[test]
fn source_grounded_chimera_has_zero_walk_and_fly_without_a_fall_or_new_effect_authority() {
    let mut f = Fixture::new();
    f.replace_target("chimera", CreatureSize::Large);
    let before = speeds(&f.state, f.target).unwrap();
    assert!(before.walk > 0 && before.fly.is_some_and(|v| v > 0));
    assert!(
        crate::spatial::fall_destination(encounter(&f.state).unwrap(), f.target)
            .unwrap()
            .is_none()
    );
    let id = f.begin(Hand::Left, None);
    f.choose(id, GrappleSaveAbility::Dexterity);
    f.submit(1);
    let current = speeds(&f.state, f.target).unwrap();
    assert_eq!(current.walk, 0);
    assert_eq!(current.fly, Some(0));
    assert!(resolution(&f.state).unwrap().falls.is_empty());
    let mut budget = TacticalTurnBudget::default();
    for mode in [DashSpeed::Speed, DashSpeed::Fly] {
        crate::tactical_budget::grant_dash(&mut budget, mode, CommandId::new(), &before).unwrap();
        assert_eq!(
            crate::tactical_budget::movement_remaining(&budget, mode, &current).unwrap(),
            0
        );
    }
    f.decline(id);
    assert!(flow(&f.state).unwrap().resolution.is_none());
    validate(&f.state).unwrap();
    assert_eq!(speeds(&f.state, f.target).unwrap().fly, Some(0));
    assert_eq!(views(&f.state).len(), 1);
}

fn attack_mode(rules: &RulesState, actor: EntityId, target: EntityId) -> RollMode {
    crate::tactical_conditions::attack_conditions(
        rules,
        actor,
        target,
        false,
        AttackPerception {
            attacker_sees_target: true,
            target_sees_attacker: true,
            fear_source_in_sight: false,
            within_five_feet: true,
            hostile_ranged_threat: false,
        },
        Circumstances::default(),
        DodgeContext::default(),
    )
    .unwrap()
    .mode
}

#[test]
fn synthetic_multiple_holder_query_retains_sources_and_never_persists_its_views() {
    let mut f = Fixture::new();
    let id = f.begin(Hand::Left, None);
    f.choose(id, GrappleSaveAbility::Strength);
    f.submit(1);
    let mut rules = f.state.rules.as_ref().unwrap().clone();
    let third = EntityId::new();
    let mut mechanics = rules.entities[&f.human].clone();
    mechanics.entity_id = third;
    rules.entities.insert(third, mechanics);
    assert_eq!(attack_mode(&rules, f.target, f.human), RollMode::Normal);
    assert_eq!(attack_mode(&rules, f.target, third), RollMode::Disadvantage);
    // Synthetic read-only topology: no actual second source/turn admission is
    // claimed. The existing real two-hand producer control covers hand release.
    let mut second = rules.tactical_grapples.as_ref().unwrap().active[0].clone();
    second.declaration.id =
        GrappleId::from_declaration(CommandId::new(), third, f.target, Hand::Left);
    second.declaration.grappler = third;
    rules
        .tactical_grapples
        .as_mut()
        .unwrap()
        .active
        .push(second);
    assert_eq!(
        attack_mode(&rules, f.target, f.human),
        RollMode::Disadvantage
    );
    assert_eq!(attack_mode(&rules, f.target, third), RollMode::Disadvantage);
    let projected = crate::tactical_effect_adapter::condition_effects(&rules).collect::<Vec<_>>();
    let live_views = projected
        .iter()
        .filter(|v| v.condition == Some(Condition::Grappled))
        .collect::<Vec<_>>();
    assert_eq!(live_views.len(), 2);
    assert_ne!(live_views[0].id, live_views[1].id);
    assert!(
        live_views
            .iter()
            .all(|v| v.expires == Expiry::Never && v.concentration_owner.is_none())
    );
    let persisted = rules.effects.clone();
    rules
        .tactical_grapples
        .as_mut()
        .unwrap()
        .active
        .retain(|g| g.declaration.id != id);
    assert_eq!(
        attack_mode(&rules, f.target, f.human),
        RollMode::Disadvantage
    );
    assert_eq!(attack_mode(&rules, f.target, third), RollMode::Normal);
    assert_eq!(rules.effects, persisted);
    // Synthetic all-mode query does not alter source definitions or admit motion.
    for base in [1, 60, 120, 1_000] {
        assert_eq!(
            crate::tactical_conditions::effective_speed(&rules, f.target, base).unwrap(),
            0
        );
    }
    let mut query = f.state.clone();
    query.rules = Some(rules);
    let movement = &mut query
        .encounter
        .as_mut()
        .unwrap()
        .participants
        .iter_mut()
        .find(|p| p.entity_id == f.target)
        .unwrap()
        .movement;
    movement.climb = Some(40);
    movement.swim = Some(80);
    movement.burrow = Some(20);
    movement.fly = Some(120);
    let all_modes = speeds(&query, f.target).unwrap();
    let mut budget = TacticalTurnBudget::default();
    for mode in [
        DashSpeed::Speed,
        DashSpeed::Climb,
        DashSpeed::Swim,
        DashSpeed::Burrow,
        DashSpeed::Fly,
    ] {
        crate::tactical_budget::grant_dash(&mut budget, mode, CommandId::new(), &all_modes)
            .unwrap();
        assert_eq!(
            crate::tactical_budget::movement_remaining(&budget, mode, &all_modes).unwrap(),
            0
        );
    }
}

#[test]
fn absence_and_legacy_grapple_effects_keep_their_exact_adapter_sequence() {
    let f = Fixture::new();
    let mut rules = f.state.rules.as_ref().unwrap().clone();
    rules.effects.push(ActiveEffect {
        id: EffectId::new(),
        source: f.human,
        target: f.target,
        condition: Some(Condition::Grappled),
        label: "Existing legacy source".into(),
        expires: Expiry::Never,
        concentration_owner: None,
    });
    let expected = rules
        .effects
        .iter()
        .cloned()
        .chain(
            rules
                .tactical_effects
                .as_ref()
                .into_iter()
                .flat_map(crate::tactical_effects::active_effect_views),
        )
        .collect::<Vec<_>>();
    assert_eq!(
        crate::tactical_effect_adapter::condition_effects(&rules).collect::<Vec<_>>(),
        expected
    );
    assert_eq!(attack_mode(&rules, f.target, f.human), RollMode::Normal);
    assert_eq!(
        crate::tactical_conditions::effective_speed(&rules, f.target, 60).unwrap(),
        0
    );
    rules.tactical_grapples = Some(TacticalGrapples {
        schema_version: TACTICAL_GRAPPLES_SCHEMA_VERSION,
        active: vec![],
    });
    assert_eq!(
        crate::tactical_effect_adapter::condition_effects(&rules).collect::<Vec<_>>(),
        expected
    );
}
