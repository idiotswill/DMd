use super::*;

fn ledge(height: i32, water: bool) -> Fixture {
    let mut f = Fixture::new();
    f.entity_mut(0).max_hp = 50;
    f.entity_mut(0).hp = 50;
    let e = f.state.encounter.as_mut().unwrap();
    e.participants[0].position.z = height;
    e.participants[0].movement.fly = None;
    e.battlefield.obstacles.push(SpatialObstacle {
        id: "authored-platform".into(),
        volume: SpatialBox {
            min: point(0, 0, 0),
            max: point(20, 30, height),
        },
        blocks_movement: true,
        blocks_sight: true,
        observable: true,
        cover: CoverDegree::Total,
    });
    if water {
        e.battlefield.terrain.push(TerrainVolume {
            id: "private-landing-water".into(),
            volume: SpatialBox {
                min: point(20, 0, 0),
                max: point(90, 90, 10),
            },
            difficult: false,
            water: true,
            climbable: false,
            burrowable: false,
            supports_top: false,
            surface: None,
            obscuration: Obscuration::None,
            magical_darkness: false,
            observable: false,
        });
    }
    f.begin();
    f
}
fn step_off(f: &mut Fixture, height: i32) -> TacticalEvent {
    f.run(
        Some(0),
        walk(&[point(20, 10, height), point(30, 10, height)]),
    )
}
fn fall(f: &Fixture) -> &TacticalFall {
    &f.flow().resolution.as_ref().unwrap().falls[0]
}

#[test]
fn legacy_liquid_choice_keeps_actor_ordering_without_new_ancestry() {
    let mut f = ledge(30, true);
    let meta = f.meta(Some(0));
    let action = walk(&[point(20, 10, 30), point(30, 10, 30)]);
    let current = resolve_tactical(&f.state, &meta, &action, &f.pack).unwrap();
    // This settled legacy image predates the execution-version upgrade. Replay
    // must reconstruct its old pause, rather than adding current ancestry.
    f.state
        .encounter
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .version = 1;
    let legacy = replay_tactical(&f.state, &current.event, &f.pack).unwrap();
    f.state = serde_json::from_slice(&serde_json::to_vec(&legacy.next_state).unwrap()).unwrap();
    f.state.applied_event_sequence += 1;
    validate_tactical_state(&f.state).unwrap();
    let resolution = f.flow().resolution.as_ref().unwrap();
    assert!(resolution.work_trace.is_none());
    assert_eq!(fall(&f).stage, TacticalFallStage::LandingChoice);
    assert!(!tactical_frame_host_ordering(resolution).unwrap());
    f.run(
        Some(0),
        TacticalAction::ChooseLiquidLanding { choice: None },
    );
    assert_eq!(f.request().dice, [DieSpec { count: 1, sides: 6 }]);
}

#[test]
fn a_real_step_off_support_pauses_damage_before_later_travel_and_replays_landing() {
    let mut f = ledge(40, false);
    let original = step_off(&mut f, 40);
    assert_eq!(f.position(0), point(20, 10, 40));
    assert_eq!(f.flow().budget.movement_spent, 10);
    assert_eq!(
        f.flow()
            .resolution
            .as_ref()
            .unwrap()
            .movement
            .as_ref()
            .unwrap()
            .next_step,
        1
    );
    assert_eq!(f.request().dice, [DieSpec { count: 2, sides: 6 }]);
    assert!(matches!(
        f.flow()
            .resolution
            .as_ref()
            .unwrap()
            .pending
            .as_ref()
            .unwrap()
            .key
            .role,
        TacticalRollRole::FallDamage
    ));
    assert_eq!(fall(&f).origin, original.meta);
    assert_eq!(fall(&f).path.to, point(20, 10, 0));
    f.rejected(Some(0), TacticalAction::VoluntarilyFailSave);
    f.rejected(
        Some(1),
        TacticalAction::SubmitRoll {
            result: f.raw(&[3, 4]),
        },
    );
    let result = f.raw(&[3, 4]);
    let landed = f.run(Some(0), TacticalAction::SubmitRoll { result });
    assert_eq!(f.position(0), point(20, 10, 0));
    assert_eq!(f.rules().entities[&f.actors[0]].hp, 43);
    assert!(f.rules().entities[&f.actors[0]].prone);
    assert!(f.flow().resolution.is_none());
    let receipt = f.flow().last_movement.as_ref().unwrap();
    assert_eq!(receipt.original, original.meta);
    assert_eq!(receipt.cause, landed.meta);
    assert_eq!(receipt.reason, TacticalMovementEnd::Fell);
    assert_eq!((receipt.completed_steps, receipt.requested_steps), (1, 2));
    assert_eq!(receipt.spent_after, 10);
    assert_eq!(receipt.endpoint, point(20, 10, 0));
    assert!(matches!(
        resolve_tactical(&f.state, &landed.meta, &landed.action, &f.pack),
        Err(RulesError::Stale)
    ));
}

#[test]
fn short_fall_has_no_damage_dice_and_still_stops_the_remaining_move() {
    let mut f = ledge(10, false);
    let rolls = f.rules().rolls.clone();
    step_off(&mut f, 10);
    assert_eq!(f.position(0), point(20, 10, 0));
    assert_eq!(f.rules().rolls, rolls);
    assert!(f.rules().pending.is_none());
    assert!(!f.rules().entities[&f.actors[0]].prone);
    assert_eq!(f.rules().entities[&f.actors[0]].hp, 50);
    assert_eq!(
        f.flow().last_movement.as_ref().unwrap().reason,
        TacticalMovementEnd::Fell
    );
}

#[test]
fn water_reaction_is_owned_and_halves_before_vulnerability_and_temporary_hp() {
    let mut f = ledge(30, true);
    f.entity_mut(0)
        .vulnerabilities
        .insert(DamageType::Bludgeoning);
    f.entity_mut(0).temporary_hp = 3;
    step_off(&mut f, 30);
    assert_eq!(fall(&f).stage, TacticalFallStage::LandingChoice);
    assert!(f.rules().pending.is_none());
    assert_eq!(f.position(0).z, 30);
    f.rejected(
        Some(1),
        TacticalAction::ChooseLiquidLanding {
            choice: Some(LiquidLandingChoice::Acrobatics),
        },
    );
    f.rejected(Some(0), TacticalAction::EndTurn);
    let choice = f.run(
        Some(0),
        TacticalAction::ChooseLiquidLanding {
            choice: Some(LiquidLandingChoice::Acrobatics),
        },
    );
    assert!(
        f.rules()
            .timing
            .as_ref()
            .unwrap()
            .reactions_spent
            .contains(&f.actors[0])
    );
    assert_eq!(f.request().modifier, 0);
    assert_eq!(
        f.request().dice,
        [DieSpec {
            count: 1,
            sides: 20
        }]
    );
    assert!(f.request().reason.contains("Acrobatics"));
    f.rejected(Some(0), TacticalAction::VoluntarilyFailSave);
    f.rejected(Some(0), TacticalAction::UseLegendaryResistance);
    f.roll(0, &[15]);
    let TacticalFallStage::Damage {
        landing: Some(landing),
    } = &fall(&f).stage
    else {
        panic!()
    };
    assert_eq!(landing.accepted_by, choice.meta);
    assert_eq!(landing.result.dice[0].value, 15);
    assert_eq!(f.request().dice, [DieSpec { count: 1, sides: 6 }]);
    f.roll(0, &[5]); // floor(5/2)=2, then Vulnerability=4, then temporary HP absorbs3.
    assert_eq!(f.position(0), point(20, 10, 10));
    assert_eq!(f.rules().entities[&f.actors[0]].hp, 49);
    assert_eq!(f.rules().entities[&f.actors[0]].temporary_hp, 0);
    assert!(f.rules().entities[&f.actors[0]].prone);
}

#[test]
fn liquid_decline_does_not_spend_reaction_and_inspiration_keeps_original_check_faces() {
    let mut declined = ledge(30, true);
    step_off(&mut declined, 30);
    declined.run(
        Some(0),
        TacticalAction::ChooseLiquidLanding { choice: None },
    );
    assert!(
        !declined
            .rules()
            .timing
            .as_ref()
            .unwrap()
            .reactions_spent
            .contains(&declined.actors[0])
    );
    declined.roll(0, &[5]);
    assert_eq!(declined.rules().entities[&declined.actors[0]].hp, 45);

    let mut inspired = ledge(30, true);
    inspired.entity_mut(0).heroic_inspiration = true;
    step_off(&mut inspired, 30);
    inspired.run(
        Some(0),
        TacticalAction::ChooseLiquidLanding {
            choice: Some(LiquidLandingChoice::Athletics),
        },
    );
    let result = inspired.raw(&[1]);
    inspired.run(
        Some(0),
        TacticalAction::SubmitRollWithInspiration {
            result,
            die_index: 0,
            replacement: DieResult {
                sides: 20,
                value: 20,
            },
        },
    );
    let recorded = inspired.rules().rolls.last().unwrap();
    assert_eq!(recorded.original_result.as_ref().unwrap().dice[0].value, 1);
    assert_eq!(recorded.result.dice[0].value, 20);
    inspired.roll(0, &[5]);
    assert_eq!(inspired.rules().entities[&inspired.actors[0]].hp, 48);
    assert!(!inspired.rules().entities[&inspired.actors[0]].heroic_inspiration);
}

#[test]
fn airborne_reaction_knockout_lands_before_path_resumes_and_dropped_gear_reaches_ground() {
    let mut f = Fixture::new();
    let weapon = f.arm("longsword", false, false).weapon;
    f.entity_mut(0).hp = 1;
    let e = f.state.encounter.as_mut().unwrap();
    e.participants[0].position.z = 40;
    e.participants[1].position.z = 40;
    e.participants[1].enemies = vec![f.actors[0]];
    f.begin();
    let original = f.run(
        Some(0),
        TacticalAction::Move {
            path: vec![TacticalMoveStep {
                destination: point(0, 10, 40),
                mode: MovementMode::Fly,
            }],
        },
    );
    f.run(
        Some(1),
        TacticalAction::OpportunityAttack {
            choice: TacticalMeleeChoice::UnarmedDamage {
                ability: Ability::Strength,
            },
        },
    );
    f.roll(1, &[15]);
    assert_eq!(
        f.flow()
            .resolution
            .as_ref()
            .unwrap()
            .attack
            .as_ref()
            .unwrap()
            .stage,
        TacticalAttackStage::KnockoutChoice
    );
    let knockout = f.run(
        Some(1),
        TacticalAction::ChooseAttackKnockout {
            choice: KnockoutChoice::KnockOut,
        },
    );
    assert_eq!(fall(&f).origin, knockout.meta);
    assert_eq!(fall(&f).cause, TacticalFallCause::FlightLost);
    assert_eq!(f.position(0), point(10, 10, 40));
    assert_eq!(f.flow().budget.movement_spent, 0);
    assert!(f.flow().resolution.as_ref().unwrap().attack.is_none());
    assert_eq!(
        f.flow()
            .ground_items
            .iter()
            .find(|ground| ground.item == weapon)
            .unwrap()
            .position
            .z,
        0
    );
    assert!(matches!(
        f.state.items[&weapon].custody,
        Custody::Location(_)
    ));
    assert_eq!(f.request().roller, Some(f.actors[0]));
    assert_eq!(f.request().dice, [DieSpec { count: 2, sides: 6 }]);
    f.roll(0, &[1, 1]);
    assert_eq!(f.position(0), point(10, 10, 0));
    assert_eq!(f.rules().entities[&f.actors[0]].hp, 0);
    assert!(
        f.rules()
            .timing
            .as_ref()
            .unwrap()
            .reactions_spent
            .contains(&f.actors[1])
    );
    let receipt = f.flow().last_movement.as_ref().unwrap();
    assert_eq!(receipt.original, original.meta);
    assert_eq!(receipt.reason, TacticalMovementEnd::Fell);
    assert_eq!((receipt.completed_steps, receipt.spent_after), (0, 0));
    assert!(f.flow().resolution.is_none());
}

#[test]
fn a_fatal_reaction_lands_the_now_dead_hover_body_without_living_damage_dice() {
    let mut f = Fixture::new();
    f.entity_mut(0).hp = 1;
    f.entity_mut(0).max_hp = 1;
    f.entity_mut(1).ability_scores[Ability::Strength.index()] = 18;
    let e = f.state.encounter.as_mut().unwrap();
    e.participants[0].position.z = 40;
    e.participants[0].movement.hover = true;
    e.participants[1].position = point(20, 10, 40);
    e.participants[1].enemies = vec![f.actors[0]];
    f.begin();
    let original = f.run(
        Some(0),
        TacticalAction::Move {
            path: vec![TacticalMoveStep {
                destination: point(0, 10, 40),
                mode: MovementMode::Fly,
            }],
        },
    );
    f.run(
        Some(1),
        TacticalAction::OpportunityAttack {
            choice: TacticalMeleeChoice::UnarmedDamage {
                ability: Ability::Strength,
            },
        },
    );
    f.roll(1, &[15]);
    let fatal = f.run(
        Some(1),
        TacticalAction::ChooseAttackKnockout {
            choice: KnockoutChoice::NormalDamage,
        },
    );
    assert_eq!(f.position(0), point(10, 10, 0));
    assert_eq!(f.rules().rolls.len(), 4); // Initiative plus the actual attack; fixed damage and corpse fall need no dice.
    assert!(f.rules().pending.is_none());
    assert!(f.rules().entities[&f.actors[0]].death.dead);
    assert_eq!(
        f.flow().last_movement.as_ref().unwrap().original,
        original.meta
    );
    assert_eq!(f.flow().last_movement.as_ref().unwrap().cause, fatal.meta);
    assert_eq!(
        f.flow().last_movement.as_ref().unwrap().reason,
        TacticalMovementEnd::Fell
    );
}

#[test]
fn retained_fall_rejects_forged_geometry_origin_work_roles_and_damage() {
    let mut f = ledge(40, false);
    step_off(&mut f, 40);
    for mutation in 0..11 {
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
        match mutation {
            0 => r.falls[0].origin.campaign_id = CampaignId::new(),
            1 => r.falls[0].origin.id = CommandId::new(),
            2 => r.falls[0].actor = f.actors[1],
            3 => r.falls[0].path.to.z = 10,
            4 => r.falls[0].path.to.x += 1,
            5 => {
                r.falls[0].path.surface = FallSurface::Liquid {
                    id: "invented".into(),
                }
            }
            6 => r.falls[0].cause = TacticalFallCause::FlightLost,
            7 => r.falls.clear(),
            8 => r.pending.as_mut().unwrap().key.role = TacticalRollRole::LiquidLandingCheck,
            9 => r.pending.as_mut().unwrap().work.kind = TacticalWorkKind::FallDamage { fall: 5 },
            10 => {
                bad.rules
                    .as_mut()
                    .unwrap()
                    .pending
                    .as_mut()
                    .unwrap()
                    .request
                    .dice[0]
                    .count += 1
            }
            _ => unreachable!(),
        }
        assert!(
            validate_tactical_state(&bad).is_err(),
            "fall mutation {mutation}"
        );
    }
}

#[test]
fn declared_jump_traverses_its_airborne_segments_then_falls_at_the_declared_end() {
    let mut f = ledge(40, false);
    f.entity_mut(0).ability_scores[Ability::Strength.index()] = 20;
    let original = f.run(
        Some(0),
        TacticalAction::Move {
            path: vec![
                TacticalMoveStep {
                    destination: point(20, 10, 40),
                    mode: MovementMode::Jump,
                },
                TacticalMoveStep {
                    destination: point(30, 10, 40),
                    mode: MovementMode::Jump,
                },
            ],
        },
    );
    assert_eq!(f.position(0), point(30, 10, 40));
    assert_eq!(f.flow().budget.movement_spent, 20);
    assert_eq!(
        f.flow()
            .resolution
            .as_ref()
            .unwrap()
            .movement
            .as_ref()
            .unwrap()
            .next_step,
        2
    );
    assert!(
        f.flow()
            .budget
            .movement_progress
            .as_ref()
            .unwrap()
            .jump
            .is_none()
    );
    assert_eq!(fall(&f).path.from, point(30, 10, 40));
    assert!(
        matches!(&fall(&f).cause, TacticalFallCause::MovementEnd { movement, step_index: 1 } if movement == &original.meta)
    );
    f.roll(0, &[1, 1]);
    assert_eq!(f.position(0), point(30, 10, 0));
    let receipt = f.flow().last_movement.as_ref().unwrap();
    assert_eq!(receipt.reason, TacticalMovementEnd::Fell);
    assert_eq!((receipt.completed_steps, receipt.requested_steps), (2, 2));
}

#[test]
fn an_unconscious_hovering_holder_drops_real_gear_without_falling_itself() {
    let mut f = Fixture::new();
    let weapon = f.arm("longsword", false, false).weapon;
    f.entity_mut(0).hp = 1;
    let e = f.state.encounter.as_mut().unwrap();
    e.participants[0].position.z = 40;
    e.participants[0].movement.hover = true;
    e.participants[1].position.z = 40;
    f.begin();
    f.run(
        Some(0),
        TacticalAction::Move {
            path: vec![TacticalMoveStep {
                destination: point(0, 10, 40),
                mode: MovementMode::Fly,
            }],
        },
    );
    f.run(
        Some(1),
        TacticalAction::OpportunityAttack {
            choice: TacticalMeleeChoice::UnarmedDamage {
                ability: Ability::Strength,
            },
        },
    );
    f.roll(1, &[15]);
    let knockout = f.run(
        Some(1),
        TacticalAction::ChooseAttackKnockout {
            choice: KnockoutChoice::KnockOut,
        },
    );
    assert_eq!(f.position(0), point(10, 10, 40));
    assert!(active_conditions(f.rules(), f.actors[0]).contains(&Condition::Unconscious));
    assert!(f.rules().pending.is_none());
    assert!(f.flow().resolution.is_none());
    let dropped = f
        .flow()
        .ground_items
        .iter()
        .find(|entry| entry.item == weapon)
        .unwrap();
    assert_eq!(dropped.position, point(10, 10, 0));
    assert_eq!(dropped.origin, knockout.meta);
    assert!(matches!(
        f.state.items[&weapon].custody,
        Custody::Location(_)
    ));
    assert_eq!(
        f.flow().last_movement.as_ref().unwrap().reason,
        TacticalMovementEnd::Interrupted
    );
}

#[test]
fn landing_finishes_only_the_move_and_preserves_its_concentration_child_and_completed_proof() {
    let mut f = ledge(40, false);
    let group = EffectId::new();
    let source = f.meta(None);
    // Explicit valid source snapshot setup; actual movement/fall/concentration
    // commands below all pass the public reducer and exact semantic replay.
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
                        actor: f.actors[0],
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
    let original = step_off(&mut f, 40);
    let result = f.raw(&[3, 4]);
    let landing = f.run(Some(0), TacticalAction::SubmitRoll { result });
    assert_eq!(f.position(0), point(20, 10, 0));
    assert!(f.flow().resolution.as_ref().unwrap().movement.is_none());
    assert_eq!(
        f.flow().last_movement.as_ref().unwrap().reason,
        TacticalMovementEnd::Fell
    );
    assert_eq!(
        f.flow().last_movement.as_ref().unwrap().original,
        original.meta
    );
    assert!(
        matches!(&fall(&f).stage, TacticalFallStage::Complete { damage: Some(_), resolved_by, .. } if resolved_by == &landing.meta)
    );
    assert!(
        matches!(f.flow().resolution.as_ref().unwrap().pending.as_ref().unwrap().work.kind,
        TacticalWorkKind::ConcentrationSave { actor, group: g, damage_taken: 7 } if actor == f.actors[0] && g == group)
    );
    for mutation in 0..4 {
        let mut bad = f.state.clone();
        let retained = &mut bad
            .encounter
            .as_mut()
            .unwrap()
            .flow
            .as_mut()
            .unwrap()
            .resolution
            .as_mut()
            .unwrap()
            .falls[0];
        match mutation {
            0 => retained.path.from.z = 80,
            1 => {
                if let TacticalFallStage::Complete { damage, .. } = &mut retained.stage {
                    *damage = None;
                }
            }
            2 => {
                if let TacticalFallStage::Complete { resolved_by, .. } = &mut retained.stage {
                    resolved_by.id = CommandId::new();
                }
            }
            3 => {
                if let TacticalFallStage::Complete {
                    damage: Some(key), ..
                } = &mut retained.stage
                {
                    key.occurrence += 1;
                }
            }
            _ => unreachable!(),
        }
        assert!(
            validate_tactical_state(&bad).is_err(),
            "completed fall mutation {mutation}"
        );
    }
    f.rejected(Some(1), TacticalAction::VoluntarilyFailSave);
    f.run(Some(0), TacticalAction::VoluntarilyFailSave);
    assert_eq!(f.rules().entities[&f.actors[0]].concentration, None);
    assert!(f.flow().resolution.is_none());
    assert_eq!(f.position(0), point(20, 10, 0));
}

#[test]
fn impossible_embedded_setup_and_restore_reject_before_any_cost_or_damage() {
    let mut f = Fixture::new();
    let mut proposed = f.state.encounter.clone().unwrap();
    proposed.origin = f.meta(None);
    proposed.battlefield.obstacles.push(SpatialObstacle {
        id: "solid-setup-error".into(),
        volume: SpatialBox {
            min: point(15, 10, 0),
            max: point(25, 20, 20),
        },
        blocks_movement: true,
        blocks_sight: false,
        observable: false,
        cover: CoverDegree::None,
    });
    f.rejected(
        None,
        TacticalAction::Establish {
            encounter: Box::new(proposed.clone()),
        },
    );
    let mut corrupt = f.state.clone();
    corrupt.encounter = Some(proposed);
    assert!(validate_tactical_state(&corrupt).is_err());
    assert!(
        resolve_tactical(
            &corrupt,
            &f.meta(None),
            &TacticalAction::Establish {
                encounter: Box::new(f.state.encounter.clone().unwrap())
            },
            &f.pack
        )
        .is_err()
    );
    f.begin();
    let before = f.state.clone();
    let e = f.state.encounter.as_mut().unwrap();
    e.participants[0].movement.burrow = Some(60);
    e.battlefield.terrain.push(TerrainVolume {
        id: "side-clipped-ground".into(),
        volume: SpatialBox {
            min: point(19, 10, 0),
            max: point(25, 20, 20),
        },
        difficult: false,
        water: false,
        climbable: false,
        burrowable: true,
        supports_top: false,
        surface: None,
        obscuration: Obscuration::None,
        magical_darkness: false,
        observable: false,
    });
    assert!(validate_tactical_state(&f.state).is_err());
    f.rejected(
        Some(0),
        TacticalAction::Dash {
            speed: DashSpeed::Speed,
        },
    );
    assert_eq!(
        f.flow().budget,
        before.encounter.unwrap().flow.unwrap().budget
    );
}

#[test]
fn knockout_during_an_airborne_jump_without_fly_speed_falls_before_the_next_step() {
    let mut f = Fixture::new();
    f.entity_mut(0).hp = 1;
    f.entity_mut(0).max_hp = 50;
    f.entity_mut(0).ability_scores[Ability::Strength.index()] = 20;
    let e = f.state.encounter.as_mut().unwrap();
    e.participants[0].position.z = 40;
    e.participants[0].movement.fly = None;
    e.participants[1].position = point(10, 20, 40);
    e.participants[1].enemies = vec![f.actors[0]];
    e.battlefield.obstacles.push(SpatialObstacle {
        id: "jump-platform".into(),
        volume: SpatialBox {
            min: point(0, 0, 0),
            max: point(20, 30, 40),
        },
        blocks_movement: true,
        blocks_sight: true,
        observable: true,
        cover: CoverDegree::Total,
    });
    e.battlefield.terrain.push(TerrainVolume {
        id: "climbable-but-not-grasped".into(),
        volume: SpatialBox {
            min: point(20, 10, 40),
            max: point(40, 20, 60),
        },
        difficult: false,
        water: false,
        climbable: true,
        burrowable: false,
        supports_top: false,
        surface: None,
        obscuration: Obscuration::None,
        magical_darkness: false,
        observable: true,
    });
    f.begin();
    let original = f.run(
        Some(0),
        TacticalAction::Move {
            path: vec![
                TacticalMoveStep {
                    destination: point(20, 10, 40),
                    mode: MovementMode::Jump,
                },
                TacticalMoveStep {
                    destination: point(30, 10, 40),
                    mode: MovementMode::Jump,
                },
            ],
        },
    );
    assert_eq!(f.position(0), point(20, 10, 40));
    assert_eq!(f.flow().budget.movement_spent, 10);
    f.run(
        Some(1),
        TacticalAction::OpportunityAttack {
            choice: TacticalMeleeChoice::UnarmedDamage {
                ability: Ability::Strength,
            },
        },
    );
    f.roll(1, &[15]);
    let knockout = f.run(
        Some(1),
        TacticalAction::ChooseAttackKnockout {
            choice: KnockoutChoice::KnockOut,
        },
    );
    assert_eq!(fall(&f).cause, TacticalFallCause::Unsupported);
    assert_eq!(fall(&f).origin, knockout.meta);
    assert_eq!(fall(&f).path.from, point(20, 10, 40));
    assert_eq!(f.request().dice, [DieSpec { count: 2, sides: 6 }]);
    f.roll(0, &[1, 1]);
    assert_eq!(f.position(0), point(20, 10, 0));
    let receipt = f.flow().last_movement.as_ref().unwrap();
    assert_eq!(receipt.original, original.meta);
    assert_eq!(receipt.reason, TacticalMovementEnd::Fell);
    assert_eq!((receipt.completed_steps, receipt.spent_after), (1, 10));
    assert!(f.flow().resolution.is_none());
}
