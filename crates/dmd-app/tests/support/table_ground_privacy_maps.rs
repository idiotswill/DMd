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
    let far_point = point(
        if matches!(case, MapCase::Distant) {
            80
        } else {
            25
        },
        15,
        0,
    );
    let obstacles = if matches!(case, MapCase::Distant) {
        vec![]
    } else {
        vec![SpatialObstacle {
            id: "real-partition".into(),
            volume: SpatialBox {
                min: point(21, 0, 0),
                max: point(22, 100, 50),
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
        encounter_id: EncounterId::new(), scene_id: SceneId::new(), location_id: LocationId::new(), name: "Accepted partition courtyard".into(),
        battlefield: Battlefield { bounds: SpatialBox { min: point(0, 0, 0), max: point(160, 100, 50) }, floor_z: 0, floor_surface: "stone".into(), ambient_light: LightLevel::Bright, terrain: vec![], obstacles, lights: vec![] },
        characters: vec![TableCharacterPlacement { character_id: f.characters[0], position: point(10, 10, 0), height: 12, allies: vec![], enemies: vec![near, ogre, far] }],
        creatures: vec![
            TableCreaturePlacement { actor: ogre, public_label: "Large armed figure".into(), position: point(far_point.x + 25, 15, 0), height: 20, allies: vec![], enemies: vec![far, f.actors[0]] },
            TableCreaturePlacement { actor: near, public_label: "Nearby small figure".into(), position: point(10, 20, 0), height: 8, allies: vec![], enemies: vec![f.actors[0]] },
            TableCreaturePlacement { actor: far, public_label: "Far-side small figure".into(), position: far_point, height: 8, allies: vec![], enemies: vec![ogre, f.actors[0]] },
        ], area_grid_policy: None, geometry_ruling: Ruling { basis: RulingBasis::GmAdjudication, reason: "Two disjoint spaces beside a full-height partition; each actual throw and its target stay on the same side. The separate PC target stays visible on the PC side.".into() },
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
            groups: actors
                .iter()
                .map(|actor| InitiativeGroup {
                    actors: vec![*actor],
                    request_id: RollRequestId::new(),
                })
                .collect(),
        }),
    ))
    .await;
    Box::pin(player_raw(f, path, 18)).await;
    Box::pin(raw(f, path, source.clone(), 12)).await;
    Box::pin(raw(f, path, TableTransportChannel::Host, 6)).await;
    Box::pin(raw(f, path, TableTransportChannel::Host, 2)).await;
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
    assert_eq!(
        dmd_rules::spatial::perceive(encounter, &thrown, f.actors[0], far)
            .unwrap()
            .sees,
        !matches!(case, MapCase::Hidden)
    );
    // Exact within-reach endpoint isolates the transparent physical obstruction.
    if !matches!(case, MapCase::Distant) {
        assert_eq!(
            dmd_rules::spatial::grid_distance(point(15, 15, 6), far_point).unwrap(),
            10
        );
    }
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
    (near, javelins[0], foreign)
}
