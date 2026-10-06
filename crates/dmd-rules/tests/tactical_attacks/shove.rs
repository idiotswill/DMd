//! Source-built reducer tests; real table/SQLite creation is tested separately.
use super::*;
use dmd_rules::tactical_creatures::*;

fn set_controller(f: &mut Fixture, index: usize, controller: CreatureController) {
    let current = f.rules().tactical_creatures.as_ref().unwrap();
    let changed = apply_creature_schedule(
        &f.state,
        current,
        &f.meta(None),
        &CreatureScheduleOperation::SetContext {
            actor: f.actors[index],
            controller,
            in_lair: false,
        },
    )
    .unwrap();
    f.state.rules.as_mut().unwrap().tactical_creatures = Some(changed.next);
    f.state.applied_event_sequence += 1;
}
fn fixture() -> Fixture {
    let mut f = Fixture::new();
    f.arm("greatsword", false, false);
    hit_shield::source_actor(&mut f, 1, "mage", CreatureSize::Medium);
    f.begin();
    set_controller(&mut f, 1, CreatureController::Host);
    f
}
fn attempt(f: &Fixture) -> TacticalAction {
    TacticalAction::Shove {
        target: f.actors[1],
    }
}
fn record(f: &Fixture) -> &TacticalShove {
    f.flow()
        .resolution
        .as_ref()
        .unwrap()
        .shove
        .as_deref()
        .unwrap()
}
fn fail(f: &mut Fixture) {
    f.run(Some(0), attempt(f));
    f.run(
        None,
        TacticalAction::ChooseShoveSave {
            ability: ShoveSaveAbility::Dexterity,
        },
    );
    f.run(None, TacticalAction::VoluntarilyFailSave);
    assert_eq!(record(f).stage, TacticalShoveStage::OutcomeChoice);
}

#[test]
fn full_hands_shove_uses_target_save_and_paid_attack_without_weapon_or_shield_work() {
    for (ability, face, succeeds) in [
        (ShoveSaveAbility::Strength, 1, false),
        (ShoveSaveAbility::Dexterity, 20, true),
    ] {
        let mut f = fixture();
        let hands = f.loadout().clone();
        assert!(!hands.hands.hands.contains(&HandAssignment::Free));
        let issued = f.run(Some(0), attempt(&f));
        assert_eq!(record(&f).difficulty, 13);
        assert_eq!(record(&f).window.id, issued.meta.id);
        assert!(f.rules().timing.as_ref().unwrap().action_spent);
        assert_eq!(f.flow().budget.attacks_remaining, 0);
        assert!(f.rules().pending.is_none());
        f.rejected(Some(0), TacticalAction::ChooseShoveSave { ability });
        f.run(None, TacticalAction::ChooseShoveSave { ability });
        assert_eq!(
            f.request().modifier,
            if ability == ShoveSaveAbility::Strength {
                -1
            } else {
                2
            }
        );
        assert_eq!(
            f.request().dice,
            vec![DieSpec {
                count: 1,
                sides: 20
            }]
        );
        assert_eq!(f.request().visibility, RollVisibility::Secret);
        f.rejected(
            Some(0),
            TacticalAction::SubmitRoll {
                result: f.raw(&[face]),
            },
        );
        f.run(
            None,
            TacticalAction::SubmitRoll {
                result: f.raw(&[face]),
            },
        );
        if !succeeds {
            f.rejected(
                None,
                TacticalAction::ChooseShoveOutcome {
                    choice: ShoveChoice::Prone,
                },
            );
            f.run(
                Some(0),
                TacticalAction::ChooseShoveOutcome {
                    choice: ShoveChoice::Prone,
                },
            );
        }
        assert!(f.flow().resolution.is_none());
        assert_eq!(f.rules().entities[&f.actors[1]].prone, !succeeds);
        assert_eq!(f.rules().entities[&f.actors[1]].hp, 81);
        assert_eq!(f.loadout(), &hands);
        assert!(f.flow().budget.weapon_history.is_empty());
        assert!(
            f.rules()
                .timing
                .as_ref()
                .unwrap()
                .reactions_spent
                .is_empty()
        );
        f.rejected(Some(0), attempt(&f));
    }
}

#[test]
fn both_owned_actor_kinds_refuse_even_same_owner_and_host_cannot_substitute() {
    for source_a in [false, true] {
        for source_b in [false, true] {
            for same in [false, true] {
                let mut f = Fixture::new();
                if !source_a {
                    f.arm("greatsword", false, false);
                }
                if source_a {
                    hit_shield::source_actor(&mut f, 0, "night-hag", CreatureSize::Medium);
                }
                if source_b {
                    hit_shield::source_actor(&mut f, 1, "mage", CreatureSize::Medium);
                }
                let e = f.state.encounter.as_mut().unwrap();
                e.participants[1].position.x = 20;
                e.participants[0].enemies = vec![f.actors[1]];
                e.participants[1].enemies = vec![f.actors[0]];
                if same {
                    if source_b {
                        let player = f.players[0];
                        set_controller(&mut f, 1, CreatureController::Player(player));
                    } else {
                        f.state
                            .characters
                            .values_mut()
                            .find(|c| c.entity_id == f.actors[1])
                            .unwrap()
                            .controlling_player_id = Some(f.players[0]);
                    }
                    f.players[1] = f.players[0];
                }
                f.begin();
                f.rejected(Some(0), attempt(&f));
                f.rejected(None, attempt(&f));
                assert!(!f.rules().timing.as_ref().unwrap().action_spent);
                assert!(f.flow().resolution.is_none());
            }
        }
    }
}

#[test]
fn every_push_waits_for_host_rederivation_and_return_preserves_paid_choice() {
    let mut f = fixture();
    fail(&mut f);
    let before = f.state.encounter.as_ref().unwrap().participants[1].position;
    let to = SpatialPoint {
        x: before.x + 10,
        ..before
    };
    let intent = TacticalAction::ChooseShoveOutcome {
        choice: ShoveChoice::Push { destination: to },
    };
    f.run(Some(0), intent.clone());
    assert_eq!(record(&f).stage, TacticalShoveStage::PushReview);
    assert_eq!(
        f.state.encounter.as_ref().unwrap().participants[1].position,
        before
    );
    f.rejected(
        Some(0),
        TacticalAction::RuleShovePush {
            ruling: ShoveGeometryRuling::CommitExactPush,
        },
    );
    f.rejected(
        None,
        TacticalAction::RuleShovePush {
            ruling: ShoveGeometryRuling::ConfirmBlockedNoMovement,
        },
    );
    f.run(
        None,
        TacticalAction::RuleShovePush {
            ruling: ShoveGeometryRuling::ReturnToShover,
        },
    );
    assert_eq!(record(&f).save.as_ref().unwrap().final_success, Some(false));
    assert!(f.rules().timing.as_ref().unwrap().action_spent);
    f.run(Some(0), intent);
    f.run(
        None,
        TacticalAction::RuleShovePush {
            ruling: ShoveGeometryRuling::CommitExactPush,
        },
    );
    assert_eq!(
        f.state.encounter.as_ref().unwrap().participants[1].position,
        to
    );
    assert!(f.flow().resolution.is_none());
    assert_eq!(f.flow().budget.movement_spent, 0);
    assert!(
        f.rules()
            .timing
            .as_ref()
            .unwrap()
            .reactions_spent
            .is_empty()
    );
    assert!(!f.rules().entities[&f.actors[1]].prone);
}

#[test]
fn blocked_push_completes_without_shortening_damage_refund_or_automatic_prone() {
    let mut f = fixture();
    // Actual authored bounds are the cause; do not fabricate an arbitrary failure.
    f.state.encounter.as_mut().unwrap().battlefield.bounds.max.x = 30;
    fail(&mut f);
    let before = f.state.encounter.as_ref().unwrap().participants[1].position;
    f.run(
        Some(0),
        TacticalAction::ChooseShoveOutcome {
            choice: ShoveChoice::Push {
                destination: SpatialPoint { x: 30, ..before },
            },
        },
    );
    f.rejected(
        None,
        TacticalAction::RuleShovePush {
            ruling: ShoveGeometryRuling::CommitExactPush,
        },
    );
    f.run(
        None,
        TacticalAction::RuleShovePush {
            ruling: ShoveGeometryRuling::ConfirmBlockedNoMovement,
        },
    );
    assert_eq!(
        f.state.encounter.as_ref().unwrap().participants[1].position,
        before
    );
    assert_eq!(f.rules().entities[&f.actors[1]].hp, 81);
    assert!(!f.rules().entities[&f.actors[1]].prone);
    assert!(f.rules().timing.as_ref().unwrap().action_spent);
    assert!(f.flow().resolution.is_none());
}

#[test]
fn retained_choice_rejects_foreign_sources_stages_requests_and_paid_windows() {
    let mut f = fixture();
    f.run(Some(0), attempt(&f));
    for case in 0..7 {
        let mut bad = f.state.clone();
        let s = bad
            .encounter
            .as_mut()
            .unwrap()
            .flow
            .as_mut()
            .unwrap()
            .resolution
            .as_mut()
            .unwrap()
            .shove
            .as_mut()
            .unwrap();
        match case {
            0 => s.difficulty += 1,
            1 => s.actor_from.x += 10,
            2 => s
                .target_source
                .as_mut()
                .unwrap()
                .definition_fingerprint
                .push('x'),
            3 => s.selected.as_mut().unwrap().occurrence += 1,
            4 => s.stage = TacticalShoveStage::Saving,
            5 => s.window.id = CommandId::new(),
            _ => s.origin.actor = Some(AgentRef::Entity(f.actors[1])),
        }
        assert!(validate_tactical_state(&bad).is_err(), "mutation {case}");
    }
    f.run(
        None,
        TacticalAction::ChooseShoveSave {
            ability: ShoveSaveAbility::Strength,
        },
    );
    let mut bad = f.state.clone();
    bad.encounter
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .resolution
        .as_mut()
        .unwrap()
        .shove
        .as_mut()
        .unwrap()
        .save
        .as_mut()
        .unwrap()
        .request
        .as_mut()
        .unwrap()
        .modifier += 1;
    assert!(validate_tactical_state(&bad).is_err());
}

#[test]
fn old_execution_neutral_relationship_and_out_of_reach_refuse_before_payment() {
    for case in 0..4 {
        let mut f = fixture();
        match case {
            0 => {
                f.state
                    .encounter
                    .as_mut()
                    .unwrap()
                    .flow
                    .as_mut()
                    .unwrap()
                    .version = TacticalExecutionVersion::ShieldHitV1.flow_version()
            }
            1 => f.state.encounter.as_mut().unwrap().participants[1]
                .enemies
                .clear(),
            2 => {
                f.state.encounter.as_mut().unwrap().participants[1]
                    .position
                    .x = 50
            }
            _ => f.state.encounter.as_mut().unwrap().participants[1]
                .allies
                .push(f.actors[0]),
        }
        f.rejected(Some(0), attempt(&f));
        assert!(!f.rules().timing.as_ref().unwrap().action_spent);
    }
}

#[test]
fn automatic_failure_and_voluntary_failure_retain_distinct_no_dice_evidence() {
    for automatic in [false, true] {
        let mut f = fixture();
        if automatic {
            f.state.rules.as_mut().unwrap().effects.push(ActiveEffect {
                id: EffectId::new(),
                source: f.actors[0],
                target: f.actors[1],
                condition: Some(Condition::Paralyzed),
                label: "Imported physical condition".into(),
                expires: Expiry::Never,
                concentration_owner: None,
            });
        }
        let before = f.rules().rolls.len();
        f.run(Some(0), attempt(&f));
        f.run(
            None,
            TacticalAction::ChooseShoveSave {
                ability: ShoveSaveAbility::Strength,
            },
        );
        if !automatic {
            f.run(None, TacticalAction::VoluntarilyFailSave);
        }
        assert_eq!(record(&f).stage, TacticalShoveStage::OutcomeChoice);
        assert_eq!(f.rules().rolls.len(), before);
        assert_eq!(
            f.flow().save_decisions.last().unwrap().failure,
            if automatic {
                TacticalSaveFailure::Automatic
            } else {
                TacticalSaveFailure::Voluntary
            }
        );
        f.run(
            Some(0),
            TacticalAction::ChooseShoveOutcome {
                choice: ShoveChoice::Prone,
            },
        );
        assert!(f.flow().resolution.is_none());
    }
}

#[test]
fn source_physical_save_exhaustion_and_inspiration_keep_original_faces() {
    let mut f = fixture();
    f.entity_mut(0).exhaustion = 2;
    f.entity_mut(1).exhaustion = 1;
    f.entity_mut(1).heroic_inspiration = true;
    f.run(Some(0), attempt(&f));
    assert_eq!(record(&f).difficulty, 13, "Exhaustion is not a DC penalty");
    f.run(
        None,
        TacticalAction::ChooseShoveSave {
            ability: ShoveSaveAbility::Dexterity,
        },
    );
    assert_eq!(f.request().modifier, 0);
    f.run(
        None,
        TacticalAction::SubmitRollWithInspiration {
            result: f.raw(&[1]),
            die_index: 0,
            replacement: DieResult {
                sides: 20,
                value: 20,
            },
        },
    );
    assert!(f.flow().resolution.is_none());
    assert_eq!(
        f.rules()
            .rolls
            .last()
            .unwrap()
            .original_result
            .as_ref()
            .unwrap()
            .dice[0]
            .value,
        1
    );
    assert_eq!(f.rules().rolls.last().unwrap().result.dice[0].value, 20);
    assert!(!f.rules().entities[&f.actors[1]].heroic_inspiration);
}

#[test]
fn healthy_source_flyer_is_displaced_without_falling_but_prone_keeps_the_shove_cause() {
    for prone in [false, true] {
        let mut f = Fixture::new();
        f.arm("greatsword", false, false);
        hit_shield::source_actor(&mut f, 1, "chimera", CreatureSize::Large);
        let e = f.state.encounter.as_mut().unwrap();
        e.participants[0].position.z = 40;
        e.participants[1].position.z = 40;
        f.begin();
        set_controller(&mut f, 1, CreatureController::Host);
        fail(&mut f);
        let origin = record(&f).origin.clone();
        if prone {
            let chosen = f.run(
                Some(0),
                TacticalAction::ChooseShoveOutcome {
                    choice: ShoveChoice::Prone,
                },
            );
            let fall = &f.flow().resolution.as_ref().unwrap().falls[0];
            assert!(
                matches!(&fall.cause, TacticalFallCause::Shove { shove, consequence, .. } if *shove == origin && *consequence == chosen.meta)
            );
            assert_eq!(f.request().dice, [DieSpec { count: 2, sides: 6 }]);
            f.run(
                None,
                TacticalAction::SubmitRoll {
                    result: f.raw(&[1, 1]),
                },
            );
            assert_eq!(
                f.state.encounter.as_ref().unwrap().participants[1]
                    .position
                    .z,
                0
            );
            assert!(f.rules().entities[&f.actors[1]].prone);
        } else {
            let from = f.state.encounter.as_ref().unwrap().participants[1].position;
            let to = SpatialPoint {
                x: from.x + 10,
                ..from
            };
            f.run(
                Some(0),
                TacticalAction::ChooseShoveOutcome {
                    choice: ShoveChoice::Push { destination: to },
                },
            );
            f.run(
                None,
                TacticalAction::RuleShovePush {
                    ruling: ShoveGeometryRuling::CommitExactPush,
                },
            );
            assert_eq!(
                f.state.encounter.as_ref().unwrap().participants[1].position,
                to
            );
        }
        assert!(f.flow().resolution.is_none());
        assert!(
            f.rules()
                .timing
                .as_ref()
                .unwrap()
                .reactions_spent
                .is_empty()
        );
    }
}

#[test]
fn size_compatible_real_dragon_resistance_precedes_any_shover_outcome() {
    for resist in [false, true] {
        let mut f = Fixture::new();
        hit_shield::source_actor(&mut f, 0, "warhorse", CreatureSize::Large);
        hit_shield::source_actor(&mut f, 1, "adult-red-dragon", CreatureSize::Huge);
        let e = f.state.encounter.as_mut().unwrap();
        e.participants[1].position.x = 30;
        e.participants[0].enemies = vec![f.actors[1]];
        e.participants[1].enemies = vec![f.actors[0]];
        f.begin();
        set_controller(&mut f, 1, CreatureController::Host);
        f.run(Some(0), attempt(&f));
        assert_eq!(record(&f).difficulty, 14);
        f.run(
            None,
            TacticalAction::ChooseShoveSave {
                ability: ShoveSaveAbility::Strength,
            },
        );
        let key = record(&f).save.as_ref().unwrap().key;
        f.run(None, TacticalAction::VoluntarilyFailSave);
        assert!(f.flow().resolution.as_ref().unwrap().failed_save.is_some());
        f.rejected(
            Some(0),
            TacticalAction::ChooseShoveOutcome {
                choice: ShoveChoice::Prone,
            },
        );
        f.rejected(Some(0), TacticalAction::UseLegendaryResistance);
        f.run(
            None,
            if resist {
                TacticalAction::UseLegendaryResistance
            } else {
                TacticalAction::DeclineLegendaryResistance
            },
        );
        if resist {
            assert!(f.flow().resolution.is_none());
            assert!(
                f.rules()
                    .tactical_creatures
                    .as_ref()
                    .unwrap()
                    .runtime(f.actors[1])
                    .unwrap()
                    .legendary_resistance_rolls
                    .contains(&key.request_id())
            );
            assert!(!f.rules().entities[&f.actors[1]].prone);
        } else {
            assert_eq!(record(&f).stage, TacticalShoveStage::OutcomeChoice);
            f.run(
                Some(0),
                TacticalAction::ChooseShoveOutcome {
                    choice: ShoveChoice::Prone,
                },
            );
            assert!(f.rules().entities[&f.actors[1]].prone);
        }
        assert!(f.rules().timing.as_ref().unwrap().action_spent);
    }
}

#[test]
fn genuine_host_source_can_shove_owned_source_and_host_source_opponents() {
    for target_owned in [false, true] {
        let mut f = Fixture::new();
        hit_shield::source_actor(&mut f, 0, "night-hag", CreatureSize::Medium);
        hit_shield::source_actor(&mut f, 1, "mage", CreatureSize::Medium);
        let e = f.state.encounter.as_mut().unwrap();
        e.participants[1].position.x = 20;
        e.participants[0].enemies = vec![f.actors[1]];
        e.participants[1].enemies = vec![f.actors[0]];
        f.begin();
        set_controller(&mut f, 0, CreatureController::Host);
        if !target_owned {
            set_controller(&mut f, 1, CreatureController::Host);
        }
        f.run(None, attempt(&f));
        let target = target_owned.then_some(1);
        if target_owned {
            f.rejected(
                None,
                TacticalAction::ChooseShoveSave {
                    ability: ShoveSaveAbility::Dexterity,
                },
            );
        }
        f.run(
            target,
            TacticalAction::ChooseShoveSave {
                ability: ShoveSaveAbility::Dexterity,
            },
        );
        if target_owned {
            f.rejected(None, TacticalAction::VoluntarilyFailSave);
        }
        f.run(target, TacticalAction::VoluntarilyFailSave);
        f.rejected(
            Some(1),
            TacticalAction::ChooseShoveOutcome {
                choice: ShoveChoice::Prone,
            },
        );
        f.run(
            None,
            TacticalAction::ChooseShoveOutcome {
                choice: ShoveChoice::Prone,
            },
        );
        assert!(f.rules().entities[&f.actors[1]].prone);
        assert!(f.flow().resolution.is_none());
    }
}

#[test]
fn host_source_shoves_an_actual_character_without_substituting_the_characters_save_choice() {
    let mut f = Fixture::new();
    // Reuse the source-built character helper with its actor at index zero,
    // then restore participant/actor ordering before building the opposing source.
    f.actors.swap(0, 1);
    f.players.swap(0, 1);
    f.state.encounter.as_mut().unwrap().participants.swap(0, 1);
    f.arm("greatsword", false, false);
    f.actors.swap(0, 1);
    f.players.swap(0, 1);
    f.state.encounter.as_mut().unwrap().participants.swap(0, 1);
    hit_shield::source_actor(&mut f, 0, "night-hag", CreatureSize::Medium);
    let e = f.state.encounter.as_mut().unwrap();
    e.participants[0].position.x = 10;
    e.participants[1].position.x = 20;
    e.participants[0].enemies = vec![f.actors[1]];
    e.participants[1].enemies = vec![f.actors[0]];
    f.begin();
    set_controller(&mut f, 0, CreatureController::Host);
    f.run(None, attempt(&f));
    f.rejected(
        None,
        TacticalAction::ChooseShoveSave {
            ability: ShoveSaveAbility::Strength,
        },
    );
    f.run(
        Some(1),
        TacticalAction::ChooseShoveSave {
            ability: ShoveSaveAbility::Strength,
        },
    );
    assert_eq!(f.request().visibility, RollVisibility::Public);
    assert_eq!(f.request().modifier, 5, "actual Fighter STR proficiency");
    f.rejected(None, TacticalAction::VoluntarilyFailSave);
    f.run(Some(1), TacticalAction::VoluntarilyFailSave);
    f.run(
        None,
        TacticalAction::ChooseShoveOutcome {
            choice: ShoveChoice::Prone,
        },
    );
    assert!(f.rules().entities[&f.actors[1]].prone);
}

#[test]
fn shove_selected_source_nodes_reject_a_valid_but_foreign_generic_work_dag() {
    let mut f = fixture();
    fail(&mut f);
    for case in 0..5 {
        let mut bad = f.state.clone();
        let r = bad
            .encounter
            .as_mut()
            .unwrap()
            .flow
            .as_mut()
            .unwrap()
            .resolution
            .as_mut()
            .unwrap();
        let nodes = &mut r.work_trace.as_mut().unwrap().nodes;
        match case {
            0 => nodes[1].parent = None,
            1 => nodes[2].parent = Some(0),
            2 => nodes[0].work.kind = TacticalWorkKind::FinishShove,
            3 => {
                r.shove
                    .as_mut()
                    .unwrap()
                    .save
                    .as_mut()
                    .unwrap()
                    .final_success = Some(true)
            }
            _ => {
                let meta = r
                    .shove
                    .as_mut()
                    .unwrap()
                    .save
                    .as_mut()
                    .unwrap()
                    .resolved_by
                    .as_mut()
                    .unwrap();
                meta.issuer = CommandIssuer::Player(f.players[0]);
                meta.actor = Some(AgentRef::Entity(f.actors[0]));
            }
        }
        assert!(
            validate_tactical_state(&bad).is_err(),
            "hostile source ancestry {case}"
        );
    }
}

#[test]
fn ledge_push_landing_keeps_source_route_and_concentration_ancestry_until_final_child() {
    let mut f = fixture();
    let point = |x, y, z| SpatialPoint { x, y, z };
    let e = f.state.encounter.as_mut().unwrap();
    e.participants[0].position.z = 40;
    e.participants[0].movement.fly = None;
    e.participants[1].position.z = 40;
    e.battlefield.obstacles.push(SpatialObstacle {
        id: "actual-supporting-ledge".into(),
        volume: SpatialBox {
            min: point(0, 0, 0),
            max: point(30, 30, 40),
        },
        blocks_movement: true,
        blocks_sight: true,
        observable: true,
        cover: CoverDegree::Total,
    });
    let group = EffectId::new();
    let source = f.meta(None);
    // Valid source snapshot setup, followed only by actual reducer commands.
    f.state = tactical_effect_adapter::apply_effect_operation(
        &f.state,
        &source,
        &EffectLifecycleAction {
            step: 0,
            operation: EffectLifecycleOperation::BeginConcentration {
                group: ConcentrationGroup {
                    id: group,
                    source: EffectSource {
                        definition_id: "retained-source-focus".into(),
                        actor: f.actors[1],
                        command: source.clone(),
                        ordinal: 0,
                    },
                    expires: TacticalEffectExpiry::Never,
                    stage: ConcentrationStage::Casting,
                },
            },
        },
    )
    .unwrap()
    .0;
    f.state.applied_event_sequence += 1;
    fail(&mut f);
    let origin = record(&f).origin.clone();
    f.run(
        Some(0),
        TacticalAction::ChooseShoveOutcome {
            choice: ShoveChoice::Push {
                destination: point(30, 10, 40),
            },
        },
    );
    let accepted = f.run(
        None,
        TacticalAction::RuleShovePush {
            ruling: ShoveGeometryRuling::CommitExactPush,
        },
    );
    assert_eq!(f.request().dice, [DieSpec { count: 2, sides: 6 }]);
    f.run(
        None,
        TacticalAction::SubmitRoll {
            result: f.raw(&[3, 4]),
        },
    );
    let r = f.flow().resolution.as_ref().unwrap();
    assert_eq!(
        f.state.encounter.as_ref().unwrap().participants[1].position,
        point(30, 10, 0)
    );
    assert!(
        matches!(&r.falls[0].cause, TacticalFallCause::Shove { shove, consequence, .. } if *shove == origin && *consequence == accepted.meta)
    );
    assert!(matches!(
        r.falls[0].stage,
        TacticalFallStage::Complete {
            damage: Some(_),
            ..
        }
    ));
    assert!(
        matches!(r.pending.as_ref().unwrap().work.kind, TacticalWorkKind::ConcentrationSave { actor, group: g, damage_taken: 7 } if actor == f.actors[1] && g == group)
    );
    for case in 0..8 {
        let mut bad = f.state.clone();
        let e = bad.encounter.as_mut().unwrap();
        let r = e.flow.as_mut().unwrap().resolution.as_mut().unwrap();
        match case {
            0 => {
                if let Some(TacticalShoveEffect::Push { intent, .. }) =
                    &mut r.shove.as_mut().unwrap().effect
                {
                    intent.destination.x += 10;
                }
            }
            1 => {
                if let Some(TacticalShoveEffect::Push { intent, .. }) =
                    &mut r.shove.as_mut().unwrap().effect
                {
                    intent.from.y += 10;
                }
            }
            2 => {
                if let Some(TacticalShoveEffect::Push { intent, ruled_by }) =
                    r.shove.as_ref().unwrap().effect.clone()
                {
                    r.shove.as_mut().unwrap().effect =
                        Some(TacticalShoveEffect::BlockedPush { intent, ruled_by });
                }
            }
            3 => r.falls[0].path.to.z = 10,
            4 => {
                if let TacticalFallCause::Shove { work, .. } = &mut r.falls[0].cause {
                    work.occurrence = 0;
                }
            }
            5 => {
                r.work_trace
                    .as_mut()
                    .unwrap()
                    .nodes
                    .iter_mut()
                    .find(|n| matches!(n.work.kind, TacticalWorkKind::BeginFall { .. }))
                    .unwrap()
                    .parent = Some(0)
            }
            6 => e.participants[1].position.x += 10,
            _ => r.falls[0].path.from.z += 10,
        }
        assert!(
            validate_tactical_state(&bad).is_err(),
            "retained displacement mutation {case}"
        );
    }
    f.rejected(Some(0), TacticalAction::VoluntarilyFailSave);
    f.run(None, TacticalAction::VoluntarilyFailSave);
    assert_eq!(f.rules().entities[&f.actors[1]].concentration, None);
    assert!(f.flow().resolution.is_none());
    assert_eq!(f.rules().entities[&f.actors[1]].hp, 74);
    assert!(
        f.rules()
            .timing
            .as_ref()
            .unwrap()
            .reactions_spent
            .is_empty()
    );
}

#[test]
fn current_and_synthetic_retained_shove_keep_paid_work_through_all_three_choices() {
    for execution in [
        TacticalExecutionVersion::ReleasedTimeV1,
        TacticalExecutionVersion::EncounterReleaseV1,
        TacticalExecutionVersion::ShieldMissileV1,
    ] {
        let mut f = fixture();
        assert_eq!(
            f.flow().version,
            TacticalExecutionVersion::ReleasedTimeV1.flow_version()
        );
        let admitted = f.run(Some(0), attempt(&f));
        // Deliberately synthetic reducer coverage: only switch the executor of
        // actual paid work. This is not a capture from the historical producer.
        f.state
            .encounter
            .as_mut()
            .unwrap()
            .flow
            .as_mut()
            .unwrap()
            .version = execution.flow_version();
        validate_tactical_state(&f.state).unwrap();
        let paid_window = record(&f).window;
        let original_position = f.state.encounter.as_ref().unwrap().participants[1].position;
        let destination = SpatialPoint {
            x: original_position.x + 10,
            ..original_position
        };
        assert_eq!(record(&f).origin, admitted.meta);
        assert!(f.rules().timing.as_ref().unwrap().action_spent);
        assert_eq!(f.flow().budget.attacks_remaining, 0);
        f.rejected(Some(0), attempt(&f));
        let save = TacticalAction::ChooseShoveSave {
            ability: ShoveSaveAbility::Dexterity,
        };
        f.rejected(Some(0), save.clone());
        let mut stale = f.meta(None);
        stale.expected_event_sequence -= 1;
        assert_eq!(
            resolve_tactical(&f.state, &stale, &save, &f.pack),
            Err(RulesError::Stale)
        );
        let mut missing_work = f.state.clone();
        missing_work
            .encounter
            .as_mut()
            .unwrap()
            .flow
            .as_mut()
            .unwrap()
            .resolution
            .as_mut()
            .unwrap()
            .shove
            .as_mut()
            .unwrap()
            .selected = None;
        assert!(resolve_tactical(&missing_work, &f.meta(None), &save, &f.pack).is_err());
        // Fixture::run validates state, round-trips bytes and independently
        // replays each newly admitted continuation before accepting it.
        f.run(None, save);
        f.run(
            None,
            TacticalAction::SubmitRoll {
                result: f.raw(&[1]),
            },
        );
        assert_eq!(record(&f).stage, TacticalShoveStage::OutcomeChoice);
        let outcome = TacticalAction::ChooseShoveOutcome {
            choice: ShoveChoice::Push { destination },
        };
        f.rejected(None, outcome.clone());
        f.run(Some(0), outcome);
        assert_eq!(record(&f).stage, TacticalShoveStage::PushReview);
        assert_eq!(record(&f).window, paid_window);
        assert_eq!(record(&f).origin, admitted.meta);
        let ruling = TacticalAction::RuleShovePush {
            ruling: ShoveGeometryRuling::CommitExactPush,
        };
        f.rejected(Some(0), ruling.clone());
        f.run(None, ruling);
        assert!(f.flow().resolution.is_none());
        assert_eq!(f.flow().version, execution.flow_version());
        assert_eq!(f.flow().budget.attack_window, Some(paid_window));
        assert_eq!(f.flow().budget.attacks_remaining, 0);
        assert!(f.rules().timing.as_ref().unwrap().action_spent);
        assert_eq!(
            f.state.encounter.as_ref().unwrap().participants[1].position,
            destination
        );
        assert_eq!(f.rules().entities[&f.actors[1]].hp, 81);
        assert!(!f.rules().entities[&f.actors[1]].prone);
        assert!(
            f.rules()
                .timing
                .as_ref()
                .unwrap()
                .reactions_spent
                .is_empty()
        );
    }
}

#[test]
fn synthetic_idle_flow4_refuses_fresh_shove_and_unpaid_choices_until_explicit_upgrade() {
    let mut f = fixture();
    // Synthetic old idle state isolates admission; it is not historical evidence.
    f.state
        .encounter
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .version = TacticalExecutionVersion::ShieldMissileV1.flow_version();
    let before = f.state.clone();
    f.rejected(Some(0), attempt(&f));
    f.rejected(
        None,
        TacticalAction::ChooseShoveSave {
            ability: ShoveSaveAbility::Strength,
        },
    );
    f.rejected(
        Some(0),
        TacticalAction::ChooseShoveOutcome {
            choice: ShoveChoice::Prone,
        },
    );
    f.rejected(
        None,
        TacticalAction::RuleShovePush {
            ruling: ShoveGeometryRuling::ReturnToShover,
        },
    );
    assert_eq!(f.state, before);
    assert!(!f.rules().timing.as_ref().unwrap().action_spent);
    f.run(
        None,
        TacticalAction::UpgradeExecutionTo {
            execution: TacticalExecutionVersion::EncounterReleaseV1,
        },
    );
    f.run(Some(0), attempt(&f));
    assert_eq!(record(&f).stage, TacticalShoveStage::SaveChoice);
    assert!(f.rules().timing.as_ref().unwrap().action_spent);
    assert_eq!(f.flow().budget.attacks_remaining, 0);
}

#[test]
fn fresh_seven_shove_uses_the_actual_turn_and_refuses_elapsed_work_until_completion() {
    let mut f = fixture();
    assert_eq!(
        f.flow().version,
        TacticalExecutionVersion::ReleasedTimeV1.flow_version()
    );
    let admitted = f.run(Some(0), attempt(&f));
    let r = f.flow().resolution.as_ref().unwrap();
    assert_eq!(r.origin, admitted.meta);
    assert_eq!(
        r.turn_context().unwrap(),
        &TacticalTurnContext {
            actor: f.actors[0],
            number: f.rules().timing.as_ref().unwrap().turn_number,
            boundary: TurnBoundary::Start,
        }
    );
    assert!(r.released_interval().is_none());
    let wire = serde_json::to_string(r).unwrap();
    assert!(wire.contains("\"turn_actor\":"));
    assert!(wire.contains("\"shove\":"));
    assert!(!wire.contains("released_interval"));
    assert_eq!(
        serde_json::from_str::<TacticalResolution>(&wire).unwrap(),
        **r
    );
    f.rejected(
        None,
        TacticalAction::AdvanceReleasedTime {
            seconds: 1,
            ordering: ReleasedTimeOrdering::HostSelect,
            ruling: "Time cannot bypass an admitted paid body action.".into(),
        },
    );
    f.rejected(None, TacticalAction::FinishEncounter);
    f.run(
        None,
        TacticalAction::ChooseShoveSave {
            ability: ShoveSaveAbility::Strength,
        },
    );
    f.run(None, TacticalAction::VoluntarilyFailSave);
    f.run(
        Some(0),
        TacticalAction::ChooseShoveOutcome {
            choice: ShoveChoice::Prone,
        },
    );
    assert!(f.flow().resolution.is_none());
    assert!(f.rules().timing.as_ref().unwrap().action_spent);
    assert!(f.rules().entities[&f.actors[1]].prone);
    assert!(
        f.state
            .encounter_history
            .as_ref()
            .is_none_or(|history| history.elapsed_intervals.is_empty())
    );
}
