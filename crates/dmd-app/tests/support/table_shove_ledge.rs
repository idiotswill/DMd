use super::*;

pub(super) async fn prepare(f: &mut Fixture) -> EntityId {
    let mage = EntityId::new();
    let gear = dmd_rules::tactical_creature_equipment::creature_equipment_plan("mage", 0).unwrap();
    f.host(
        TableAction::CreateCreature {
            creation: Box::new(TableCreatureCreation {
                entity_id: mage,
                name: "Private ledge Mage".into(),
                definition_id: "mage".into(),
                source: Some(
                    dmd_rules::tactical_creatures::creature_source_pin(
                        dmd_rules::tactical_creatures::creature_definition("mage").unwrap(),
                    )
                    .unwrap(),
                ),
                size: CreatureSize::Medium,
                additional_languages: vec!["dwarvish".into(), "elvish".into(), "draconic".into()],
                ammunition_units: 0,
                item_ids: gear.iter().map(|_| ItemId::new()).collect(),
            }),
        },
        Some(f.session),
    )
    .await;
    let setup = f
        .runtime
        .table_view(f.campaign, TableViewer::Host)
        .await
        .unwrap();
    let count = setup
        .characters
        .iter()
        .find(|c| c.character_id == f.characters[0])
        .unwrap()
        .equipment
        .as_ref()
        .unwrap()
        .initial_item_count;
    f.host(
        TableAction::PrepareEquipment {
            character_id: f.characters[0],
            item_ids: (0..count).map(|_| ItemId::new()).collect(),
        },
        Some(f.session),
    )
    .await;
    let p = |x, y, z| SpatialPoint { x, y, z };
    f.host(
        TableAction::PrepareBattlefield {
            setup: Box::new(TableBattlefieldSetup {
                encounter_id: EncounterId::new(),
                scene_id: SceneId::new(),
                location_id: LocationId::new(),
                name: "A supported ledge over water".into(),
                battlefield: Battlefield {
                    bounds: SpatialBox {
                        min: p(0, 0, 0),
                        max: p(100, 100, 100),
                    },
                    floor_z: 0,
                    floor_surface: "stone".into(),
                    ambient_light: LightLevel::Bright,
                    obstacles: vec![SpatialObstacle {
                        id: "supporting-platform".into(),
                        volume: SpatialBox {
                            min: p(0, 0, 0),
                            max: p(30, 30, 50),
                        },
                        blocks_movement: true,
                        blocks_sight: true,
                        observable: true,
                        cover: CoverDegree::Total,
                    }],
                    terrain: vec![TerrainVolume {
                        id: "private-landing-water".into(),
                        volume: SpatialBox {
                            min: p(30, 0, 0),
                            max: p(90, 90, 10),
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
                    }],
                    lights: vec![],
                },
                characters: vec![TableCharacterPlacement {
                    character_id: f.characters[0],
                    position: p(10, 10, 50),
                    height: 12,
                    allies: vec![],
                    enemies: vec![mage],
                }],
                creatures: vec![TableCreaturePlacement {
                    actor: mage,
                    public_label: "Visible traveler".into(),
                    position: p(20, 10, 50),
                    height: 12,
                    allies: vec![],
                    enemies: vec![f.actors[0]],
                }],
                area_grid_policy: None,
                geometry_ruling: Ruling {
                    basis: RulingBasis::GmAdjudication,
                    reason: "Real platform support and a separate liquid first-contact surface."
                        .into(),
                },
            }),
        },
        Some(f.session),
    )
    .await;
    f.host(
        TableAction::Tactical {
            action: TacticalAction::Begin {
                execution: TacticalExecutionVersion::ShieldMissileV1,
                combatants: vec![
                    TacticalCombatant {
                        actor: f.actors[0],
                        source: TacticalSource::Character,
                        surprised: false,
                    },
                    TacticalCombatant {
                        actor: mage,
                        source: TacticalSource::Creature {
                            definition_id: "mage".into(),
                        },
                        surprised: false,
                    },
                ],
                groups: vec![
                    InitiativeGroup {
                        actors: vec![f.actors[0]],
                        request_id: RollRequestId::new(),
                    },
                    InitiativeGroup {
                        actors: vec![mage],
                        request_id: RollRequestId::new(),
                    },
                ],
            },
        },
        Some(f.session),
    )
    .await;
    table_attack_cases::submit(f, false, &[18]).await;
    table_attack_cases::submit(f, true, &[2]).await;
    mage
}
