//! Real source throws produce inaccessible Items; no imported positive ground record.
use super::*;

#[derive(Clone, Copy, Debug)]
pub(super) enum MapCase {
    Distant,
    Hidden,
    Transparent,
}

pub(super) async fn inaccessible_ground(
    f: &mut Fixture,
    path: &Path,
    case: MapCase,
) -> (EntityId, ItemId, ItemId) {
    let ogre = Box::pin(current_source(f, path, "ogre", CreatureSize::Large)).await;
    let near = Box::pin(current_source(
        f,
        path,
        "goblin-warrior",
        CreatureSize::Small,
    ))
    .await;
    let far = Box::pin(current_source(
        f,
        path,
        "goblin-warrior",
        CreatureSize::Small,
    ))
    .await;
    let source = Box::pin(control_source(f, path, ogre)).await;
    let shown = view(f, &TableTransportChannel::Host).await;
    let count = shown
        .characters
        .iter()
        .find(|character| character.character_id == f.characters[0])
        .unwrap()
        .equipment
        .as_ref()
        .unwrap()
        .initial_item_count;
    Box::pin(step(
        f,
        path,
        TableTransportChannel::Host,
        TableTransportInput::Action(Box::new(TableAction::PrepareEquipment {
            character_id: f.characters[0],
            item_ids: (0..count).map(|_| ItemId::new()).collect(),
        })),
    ))
    .await;
    let point = |x, y, z| SpatialPoint { x, y, z };
    let ledge_z = if matches!(case, MapCase::Distant) {
        0
    } else {
        5
    };
    let far_point = point(
        if matches!(case, MapCase::Distant) {
            80
        } else {
            20
        },
        10,
        0,
    );
    let obstacles = if matches!(case, MapCase::Distant) {
        vec![]
    } else {
        vec![SpatialObstacle {
            id: "supporting-ledge".into(),
            volume: SpatialBox {
                min: point(0, 0, 0),
                max: point(20, 100, 5),
            },
            blocks_movement: true,
            blocks_sight: matches!(case, MapCase::Hidden),
            observable: true,
            cover: if matches!(case, MapCase::Hidden) {
                CoverDegree::Total
            } else {
                CoverDegree::None
            },
        }]
    };
    let battlefield = TableBattlefieldSetup {
        encounter_id: EncounterId::new(), scene_id: SceneId::new(), location_id: LocationId::new(), name: "Accepted floor and supporting ledge".into(),
        battlefield: Battlefield { bounds: SpatialBox { min: point(0, 0, 0), max: point(160, 100, 50) }, floor_z: 0, floor_surface: "stone".into(), ambient_light: LightLevel::Bright, terrain: vec![], obstacles, lights: vec![] },
        characters: vec![TableCharacterPlacement { character_id: f.characters[0], position: point(10, 10, ledge_z), height: 12, allies: vec![], enemies: vec![near, ogre, far] }],
        creatures: vec![
            TableCreaturePlacement { actor: ogre, public_label: "Large armed figure".into(), position: point(far_point.x + 30, 10, 0), height: 20, allies: vec![], enemies: vec![far, f.actors[0]] },
            TableCreaturePlacement { actor: near, public_label: "Nearby small figure".into(), position: point(10, 20, ledge_z), height: 8, allies: vec![], enemies: vec![f.actors[0]] },
            TableCreaturePlacement { actor: far, public_label: "Far-side small figure".into(), position: far_point, height: 8, allies: vec![], enemies: vec![ogre, f.actors[0]] },
        ], area_grid_policy: None, geometry_ruling: Ruling { basis: RulingBasis::GmAdjudication, reason: "Aligned occupied cells on a bright floor. In the nearby cases a low solid ledge supports the PC and its visible target; the source throw stays outside it, and the ground Item below the edge has an obstructed hand path.".into() },
    };
    Box::pin(step(
        f,
        path,
        TableTransportChannel::Host,
        TableTransportInput::Action(Box::new(TableAction::PrepareBattlefield {
            setup: Box::new(battlefield),
        })),
    ))
    .await;
    let initial = state(f).await;
    let map = initial.encounter.as_ref().unwrap();
    dmd_rules::spatial::validate_encounter(map, &initial).unwrap();
    dmd_rules::spatial::validate_physical_positions(map).unwrap();
    for actor in &map.participants {
        assert_eq!(actor.position.x.rem_euclid(GRID_SQUARE_UNITS), 0);
        assert_eq!(actor.position.y.rem_euclid(GRID_SQUARE_UNITS), 0);
        assert!(
            map.battlefield
                .obstacles
                .iter()
                .all(|obstacle| !actor.volume().unwrap().intersects(obstacle.volume))
        );
        assert!(
            dmd_rules::spatial::fall_destination(map, actor.entity_id)
                .unwrap()
                .is_none()
        );
    }
    assert_eq!(
        map.participant(f.actors[0]).unwrap().position,
        point(10, 10, ledge_z)
    );
    assert_eq!(
        map.participant(near).unwrap().position,
        point(10, 20, ledge_z)
    );
    assert_eq!(map.participant(far).unwrap().position, far_point);
    assert_eq!(
        map.participant(ogre).unwrap().position,
        point(far_point.x + 30, 10, 0)
    );
    assert!(
        dmd_rules::spatial::perceive(map, &initial, ogre, far)
            .unwrap()
            .precisely_located
    );
    assert!(
        dmd_rules::spatial::perceive(map, &initial, f.actors[0], near)
            .unwrap()
            .sees
    );
    let mut javelins = initial
        .items
        .values()
        .filter(|item| item.definition_id == "javelin" && item.custody == Custody::Entity(ogre))
        .map(|item| item.id)
        .collect::<Vec<_>>();
    javelins.sort_by_key(|id| id.0);
    assert_eq!(javelins.len(), 3);
    let foreign = initial
        .items
        .values()
        .find(|item| item.custody == Custody::Entity(near))
        .unwrap()
        .id;
    let actors = [f.actors[0], ogre, near, far];
    let groups = vec![
        InitiativeGroup {
            actors: vec![f.actors[0]],
            request_id: RollRequestId::new(),
        },
        InitiativeGroup {
            actors: vec![ogre],
            request_id: RollRequestId::new(),
        },
        InitiativeGroup {
            actors: vec![near, far],
            request_id: RollRequestId::new(),
        },
    ];
    Box::pin(step(
        f,
        path,
        TableTransportChannel::Host,
        action(TacticalAction::Begin {
            execution: TacticalExecutionVersion::EncounterReleaseV1,
            combatants: actors
                .iter()
                .enumerate()
                .map(|(index, actor)| TacticalCombatant {
                    actor: *actor,
                    source: if index == 0 {
                        TacticalSource::Character
                    } else {
                        TacticalSource::Creature {
                            definition_id: if index == 1 { "ogre" } else { "goblin-warrior" }
                                .into(),
                        }
                    },
                    surprised: false,
                })
                .collect(),
            groups: groups.clone(),
        }),
    ))
    .await;
    let begun = state(f).await;
    assert_eq!(
        begun
            .encounter
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .initiative_groups,
        groups
    );
    assert!(begun.rules.as_ref().unwrap().rolls.is_empty());
    let pending = begun.rules.as_ref().unwrap().pending.as_ref().unwrap();
    assert_eq!(pending.request.id, groups[0].request_id);
    assert_eq!(pending.request.roller, Some(f.actors[0]));
    Box::pin(player_raw(f, path, 18)).await;
    let after_pc = state(f).await;
    let pending = after_pc.rules.as_ref().unwrap().pending.as_ref().unwrap();
    assert_eq!(pending.request.id, groups[1].request_id);
    assert_eq!(pending.request.roller, Some(ogre));
    Box::pin(raw(f, path, source.clone(), 12)).await;
    let after_ogre = state(f).await;
    let pending = after_ogre.rules.as_ref().unwrap().pending.as_ref().unwrap();
    assert_eq!(pending.request.id, groups[2].request_id);
    assert_eq!(pending.request.roller, Some(near));
    Box::pin(raw(f, path, TableTransportChannel::Host, 6)).await;
    let tied = state(f).await;
    let rules = tied.rules.as_ref().unwrap();
    assert_eq!(rules.rolls.len(), 3);
    assert!(rules.pending.is_none());
    for (index, actor, face, modifier, total) in [
        (0, f.actors[0], 18, 2, 20),
        (1, ogre, 12, -1, 11),
        (2, near, 6, 2, 8),
    ] {
        let raw = &rules.rolls[index];
        assert_eq!(raw.request.id, groups[index].request_id);
        assert_eq!(raw.request.roller, Some(actor));
        assert_eq!(raw.request.mode, RollMode::Normal);
        assert_eq!(
            raw.request.dice,
            vec![DieSpec {
                count: 1,
                sides: 20
            }]
        );
        assert_eq!(raw.request.modifier, modifier);
        assert_eq!(raw.result.source, RollSource::Physical);
        assert_eq!(
            raw.result.dice,
            vec![DieResult {
                sides: 20,
                value: face
            }]
        );
        assert_eq!(raw.resolved.total, total);
    }
    assert_eq!(
        tied.encounter
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .phase,
        TacticalPhase::InitiativeTies {
            ties: vec![InitiativeTie {
                total: 8,
                actors: vec![near, far],
                proposed_order: None,
                accepted_by: vec![],
                host_decided: false,
            }]
        }
    );
    Box::pin(step(
        f,
        path,
        TableTransportChannel::Host,
        action(TacticalAction::ProposeInitiativeTie {
            order: vec![near, far],
        }),
    ))
    .await;
    let ready = state(f).await;
    let final_rules = ready.rules.as_ref().unwrap();
    assert_eq!(final_rules.rolls, rules.rolls);
    let flow = ready.encounter.as_ref().unwrap().flow.as_ref().unwrap();
    assert_eq!(flow.initiative_groups, groups);
    assert_eq!(flow.phase, TacticalPhase::Active);
    assert_eq!(
        flow.initiative_decisions,
        vec![InitiativeTie {
            total: 8,
            actors: vec![near, far],
            proposed_order: Some(vec![near, far]),
            accepted_by: vec![],
            host_decided: true,
        }]
    );
    let timing = final_rules.timing.as_ref().unwrap();
    assert_eq!(timing.index, 0);
    assert_eq!(
        timing
            .order
            .iter()
            .map(|entry| (entry.actor, entry.total, entry.tie_break))
            .collect::<Vec<_>>(),
        vec![
            (f.actors[0], 20, 0),
            (ogre, 11, 0),
            (near, 8, 0),
            (far, 8, 1)
        ]
    );
    Box::pin(step(
        f,
        path,
        TableTransportChannel::Host,
        action(TacticalAction::ActivateAttackEquipment),
    ))
    .await;
    Box::pin(player_step(f, path, action(TacticalAction::EndTurn))).await;
    let throw = Box::pin(step(
        f,
        path,
        source.clone(),
        action(TacticalAction::CreatureWeaponAttack {
            feature_id: "javelin-thrown".into(),
            choice: CreatureWeaponUseChoice {
                weapon: javelins[0],
                target: far,
                grip: WeaponGrip::OneHand(Hand::Right),
                ammunition: None,
                equipment_change: None,
                after_equipment: None,
            },
        }),
    ))
    .await;
    let roll = request(f, source.clone(), face_input(view(f, &source).await, 1)).await;
    Box::pin(cold_step(f, path, roll.clone())).await;
    let thrown = state(f).await;
    let encounter = thrown.encounter.as_ref().unwrap();
    assert!(encounter.participant(ogre).unwrap().senses.darkvision > 0);
    assert_eq!(
        thrown
            .rules
            .as_ref()
            .unwrap()
            .tactical_creatures
            .as_ref()
            .unwrap()
            .profile(ogre),
        initial
            .rules
            .as_ref()
            .unwrap()
            .tactical_creatures
            .as_ref()
            .unwrap()
            .profile(ogre)
    );
    let ground = encounter
        .flow
        .as_ref()
        .unwrap()
        .ground_items
        .iter()
        .find(|ground| ground.item == javelins[0])
        .unwrap();
    assert_eq!(ground.position, far_point);
    assert_eq!(ground.origin.id, roll.command_id);
    assert_eq!(
        thrown.items[&javelins[0]].custody,
        Custody::Location(thrown.scenes[&encounter.scene_id].location_id)
    );
    assert_eq!(
        thrown.items[&javelins[0]].owner,
        initial.items[&javelins[0]].owner
    );
    assert_eq!(thrown.items[&javelins[0]].quantity, 1);
    assert_eq!(
        encounter
            .flow
            .as_ref()
            .unwrap()
            .budget
            .weapon_history
            .iter()
            .find(|receipt| receipt.origin.id == throw.command_id)
            .unwrap()
            .outcome,
        WeaponAttackOutcome::Miss
    );
    let pc = encounter.participant(f.actors[0]).unwrap();
    assert_eq!(
        pc.volume().unwrap(),
        SpatialBox {
            min: point(10, 10, ledge_z),
            max: point(20, 20, ledge_z + 12),
        }
    );
    assert_eq!(encounter.battlefield.ambient_light, LightLevel::Bright);
    assert!(encounter.battlefield.terrain.is_empty());
    assert_eq!(pc.senses.blindsight, 0);
    assert_eq!(pc.senses.truesight, 0);
    if matches!(case, MapCase::Distant) {
        assert!(encounter.battlefield.obstacles.is_empty());
        assert!(dmd_rules::spatial::grid_distance(point(15, 15, 5), ground.position).unwrap() > 10);
        assert!(dmd_rules::spatial::grid_distance(point(15, 15, 7), ground.position).unwrap() > 10);
    } else {
        // Actual occupied-cell hand centers of this12-unit-high body are z10/z12.
        // The nearer one is exactly in range but crosses the solid ledge interior.
        let lower_hand = point(15, 15, 10);
        let upper_hand = point(15, 15, 12);
        assert_eq!(pc.center().unwrap(), point(15, 15, 11));
        assert_eq!(encounter.battlefield.obstacles.len(), 1);
        let ledge = &encounter.battlefield.obstacles[0];
        assert_eq!(
            ledge.volume,
            SpatialBox {
                min: point(0, 0, 0),
                max: point(20, 100, 5)
            }
        );
        assert!(ledge.blocks_movement);
        assert_eq!(ledge.blocks_sight, matches!(case, MapCase::Hidden));
        assert_eq!(
            ledge.cover,
            if matches!(case, MapCase::Hidden) {
                CoverDegree::Total
            } else {
                CoverDegree::None
            }
        );
        assert_eq!(
            dmd_rules::spatial::grid_distance(lower_hand, ground.position).unwrap(),
            10
        );
        assert_eq!(
            dmd_rules::spatial::grid_distance(upper_hand, ground.position).unwrap(),
            12
        );
        assert!(
            dmd_rules::spatial::segment_intersects(lower_hand, ground.position, ledge.volume)
                .unwrap()
        );
        assert!(
            dmd_rules::spatial::segment_intersects(
                pc.center().unwrap(),
                ground.position,
                ledge.volume
            )
            .unwrap()
        );
        // Hidden concerns this actual Item point; the Goblin's upper body is visible
        // above either ledge. Transparent has the same geometry without sight cover.
        let nearby_point = encounter.participant(near).unwrap().position;
        assert_eq!(
            dmd_rules::spatial::grid_distance(lower_hand, nearby_point).unwrap(),
            5
        );
        assert!(
            !dmd_rules::spatial::segment_intersects(lower_hand, nearby_point, ledge.volume)
                .unwrap()
        );
    }
    assert!(
        dmd_rules::spatial::perceive(encounter, &thrown, f.actors[0], far)
            .unwrap()
            .sees
    );
    if matches!(case, MapCase::Transparent) {
        assert_eq!(encounter.battlefield.obstacles[0].cover, CoverDegree::None);
        assert!(!encounter.battlefield.obstacles[0].blocks_sight);
        assert!(encounter.battlefield.obstacles[0].blocks_movement);
    }
    Box::pin(step(f, path, source, action(TacticalAction::EndTurn))).await;
    for _ in 0..2 {
        Box::pin(step(
            f,
            path,
            TableTransportChannel::Host,
            action(TacticalAction::EndTurn),
        ))
        .await;
    }
    let current = view(f, &player(f))
        .await
        .tactical
        .unwrap()
        .attack_options
        .unwrap();
    assert!(
        current.targets.iter().any(|target| target.actor == near),
        "the independent target is genuinely visible and legal"
    );
    assert_eq!(current.actor, f.actors[0]);
    assert!(
        !current
            .equipment
            .as_ref()
            .unwrap()
            .pickups
            .iter()
            .any(|offer| offer.item == javelins[0])
    );
    // The unchanged caller proves all same-prefix before/after admission denials
    // and a genuine different nearby Item's offered pickup at the selected after.
    (near, javelins[0], foreign)
}
