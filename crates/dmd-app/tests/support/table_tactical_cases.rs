use super::*;
use dmd_rules::tactical::TacticalAction;

#[tokio::test]
async fn source_creature_gear_is_atomic_private_and_restores_without_regranting() {
    // Separate independently awaited stages keep debug poll stack frames bounded.
    // All authority, privacy, replay and physical-inventory assertions remain active.
    let mut f = Fixture::new().await;
    let (created, actor) = verify_creature_creation(&f).await;
    verify_creature_armor(&created, actor);
    verify_creature_scene(&mut f, actor).await;
    f.pool.close().await;
}

async fn verify_creature_creation(f: &Fixture) -> (CampaignState, EntityId) {
    let actor = EntityId::new();
    let allocations =
        dmd_rules::tactical_creature_equipment::creature_equipment_plan("goblin-warrior", 20)
            .unwrap();
    let creation = TableCreatureCreation {
        entity_id: actor,
        name: "Private sentry".into(),
        definition_id: "goblin-warrior".into(),
        size: CreatureSize::Small,
        additional_languages: vec![],
        ammunition_units: 20,
        item_ids: allocations.iter().map(|_| ItemId::new()).collect(),
    };
    let action = TableAction::CreateCreature {
        creation: Box::new(creation.clone()),
    };
    let before = f
        .runtime
        .open_campaign(f.campaign)
        .await
        .unwrap()
        .state()
        .clone();
    assert!(
        f.runtime
            .execute_table(f.player_meta(0).await, action.clone())
            .await
            .is_err()
    );
    assert_eq!(
        f.runtime.open_campaign(f.campaign).await.unwrap().state(),
        &before
    );
    let mut duplicate = creation.clone();
    duplicate.item_ids[1] = duplicate.item_ids[0];
    assert!(
        f.runtime
            .execute_table(
                f.meta(CommandIssuer::Admin, None, Some(f.session)).await,
                TableAction::CreateCreature {
                    creation: Box::new(duplicate)
                }
            )
            .await
            .is_err()
    );
    assert_eq!(
        f.runtime.open_campaign(f.campaign).await.unwrap().state(),
        &before
    );
    let meta = f.meta(CommandIssuer::Admin, None, Some(f.session)).await;
    f.runtime
        .execute_table(meta.clone(), action.clone())
        .await
        .unwrap();
    assert!(
        f.runtime
            .execute_table(meta.clone(), action.clone())
            .await
            .unwrap()
            .already_accepted
    );
    let created = f
        .runtime
        .open_campaign(f.campaign)
        .await
        .unwrap()
        .state()
        .clone();
    assert_eq!(created.items.len(), allocations.len());
    let rules = created.rules.as_ref().unwrap();
    assert_eq!(dmd_rules::armor_class(&rules.entities[&actor]), 15);
    let loadout = rules
        .tactical_inventory
        .as_ref()
        .unwrap()
        .loadout(actor)
        .unwrap();
    let shield = loadout.shield.unwrap();
    assert_eq!(created.items[&shield].custody, Custody::Entity(actor));
    let pack: dmd_rules::RulesPack =
        serde_json::from_str(include_str!("../../../../content/srd-5.2.1/kernel.json")).unwrap();
    for (definition, spent) in [("scimitar", false), ("arrows", true)] {
        let mut malformed = created.clone();
        let item = malformed
            .items
            .values_mut()
            .find(|item| item.definition_id == definition)
            .unwrap();
        if spent {
            item.state = ItemState::Spent;
        } else {
            item.quantity = 2;
        }
        assert!(
            dmd_rules::validate_state(&malformed, &pack).is_err(),
            "unheld source equipment must be checked"
        );
    }
    let view = f
        .runtime
        .table_view(f.campaign, TableViewer::Player(f.players[0]))
        .await
        .unwrap();
    assert!(
        !serde_json::to_string(&view)
            .unwrap()
            .contains("Private sentry")
    );
    assert!(
        !serde_json::to_string(&view)
            .unwrap()
            .contains(&actor.0.to_string())
    );
    let export = export_campaign(&f.pool, f.campaign).await.unwrap();
    let pool = open_sqlite("sqlite::memory:").await.unwrap();
    let restored = CampaignRuntime::from_content_root(
        pool.clone(),
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../content"),
    );
    restored.restore_campaign(&export).await.unwrap();
    assert_eq!(
        restored.resume_campaign(f.campaign).await.unwrap().state(),
        &created
    );
    assert!(
        restored
            .execute_table(meta, action)
            .await
            .unwrap()
            .already_accepted
    );
    assert_eq!(
        restored.resume_campaign(f.campaign).await.unwrap().state(),
        &created
    );

    pool.close().await;
    (created, actor)
}

fn verify_creature_armor(created: &CampaignState, actor: EntityId) {
    let shield = created
        .rules
        .as_ref()
        .unwrap()
        .tactical_inventory
        .as_ref()
        .unwrap()
        .loadout(actor)
        .unwrap()
        .shield
        .unwrap();
    // The source profile remains immutable while live custody governs shield AC.
    // This isolated query test is not a journaled loss or a replacement for the
    // unconscious-drop integration scenario in the turn scheduler.
    let mut lost = created.clone();
    let inventory = lost
        .rules
        .as_mut()
        .unwrap()
        .tactical_inventory
        .as_mut()
        .unwrap();
    let loadout = inventory
        .loadouts
        .iter_mut()
        .find(|l| l.actor == actor)
        .unwrap();
    loadout.shield = None;
    loadout.hands = WeaponLoadout::default();
    lost.items.get_mut(&shield).unwrap().custody = Custody::Missing;
    let profile = lost
        .rules
        .as_ref()
        .unwrap()
        .tactical_creatures
        .as_ref()
        .unwrap()
        .profile(actor)
        .unwrap()
        .clone();
    assert!(
        dmd_rules::tactical_creatures::validate_creature_profile(
            &lost,
            &profile,
            &lost.rules.as_ref().unwrap().entities[&actor]
        )
        .is_err()
    );
    let armor =
        dmd_rules::tactical_creature_equipment::creature_current_armor(&lost, &profile).unwrap();
    lost.rules
        .as_mut()
        .unwrap()
        .entities
        .get_mut(&actor)
        .unwrap()
        .armor = armor;
    assert_eq!(
        dmd_rules::armor_class(&lost.rules.as_ref().unwrap().entities[&actor]),
        13
    );
    dmd_rules::tactical_creatures::validate_creature_profile(
        &lost,
        &profile,
        &lost.rules.as_ref().unwrap().entities[&actor],
    )
    .unwrap();
    assert_eq!(
        dmd_rules::tactical_creature_equipment::creature_attack_gear(&profile, "scimitar").unwrap(),
        Some("scimitar")
    );
}

async fn verify_creature_scene(f: &mut Fixture, actor: EntityId) {
    prepare_source_scene(f, actor).await;
    let player = f
        .runtime
        .table_view(f.campaign, TableViewer::Player(f.players[0]))
        .await
        .unwrap();
    let serialized = serde_json::to_string(&player).unwrap();
    assert!(serialized.contains("Small armored figure"));
    assert!(!serialized.contains("Private sentry"));
    assert!(!serialized.contains("goblin-warrior"));
    assert!(player.creature_setup.is_none());
    let tactical = player.tactical.unwrap();
    assert!(tactical.combatant_sources.is_empty());
    assert!(
        tactical.observers[0]
            .contacts
            .iter()
            .any(|contact| contact.entity_id == actor
                && contact.status == dmd_rules::spatial::ContactStatus::Seen)
    );
    f.host(
        TableAction::Tactical {
            action: TacticalAction::Begin {
                combatants: vec![
                    TacticalCombatant {
                        actor: f.actors[0],
                        source: TacticalSource::Character,
                        surprised: false,
                    },
                    TacticalCombatant {
                        actor,
                        source: TacticalSource::Creature {
                            definition_id: "goblin-warrior".into(),
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
                        actors: vec![actor],
                        request_id: RollRequestId::new(),
                    },
                ],
            },
        },
        Some(f.session),
    )
    .await;
    f.runtime
        .execute_table(f.player_meta(0).await, roll(f, 0, 15).await)
        .await
        .unwrap();
    let pending = f
        .runtime
        .open_campaign(f.campaign)
        .await
        .unwrap()
        .state()
        .clone();
    let pending_request = pending
        .rules
        .as_ref()
        .unwrap()
        .pending
        .as_ref()
        .unwrap()
        .request
        .clone();
    assert_eq!(pending_request.roller, Some(actor));
    assert_eq!(pending_request.visibility, RollVisibility::Secret);
    let scene_export = export_campaign(&f.pool, f.campaign).await.unwrap();
    let scene_pool = open_sqlite("sqlite::memory:").await.unwrap();
    let scene_restore = CampaignRuntime::from_content_root(
        scene_pool.clone(),
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../content"),
    );
    scene_restore.restore_campaign(&scene_export).await.unwrap();
    assert_eq!(
        scene_restore
            .resume_campaign(f.campaign)
            .await
            .unwrap()
            .state(),
        &pending
    );
    let rolled = TableAction::Tactical {
        action: TacticalAction::SubmitRoll {
            result: RollResult {
                request_id: pending_request.id,
                source: RollSource::Physical,
                dice: vec![DieResult {
                    sides: 20,
                    value: 4,
                }],
            },
        },
    };
    let roll_meta = f.meta(CommandIssuer::Admin, None, Some(f.session)).await;
    f.runtime
        .execute_table(roll_meta.clone(), rolled.clone())
        .await
        .unwrap();
    scene_restore
        .execute_table(roll_meta, rolled)
        .await
        .unwrap();
    assert_eq!(
        scene_restore
            .resume_campaign(f.campaign)
            .await
            .unwrap()
            .state(),
        f.runtime.open_campaign(f.campaign).await.unwrap().state()
    );
    scene_pool.close().await;
}

async fn prepare_source_scene(f: &mut Fixture, actor: EntityId) {
    prepare_source_scene_at(f, actor, SpatialPoint { x: 30, y: 10, z: 0 }).await;
}

pub(super) async fn prepare_source_scene_at(
    f: &mut Fixture,
    actor: EntityId,
    position: SpatialPoint,
) {
    let view = f
        .runtime
        .table_view(f.campaign, TableViewer::Host)
        .await
        .unwrap();
    let equipment = view
        .characters
        .iter()
        .find(|character| character.character_id == f.characters[0])
        .unwrap()
        .equipment
        .as_ref()
        .unwrap();
    f.host(
        TableAction::PrepareEquipment {
            character_id: f.characters[0],
            item_ids: (0..equipment.initial_item_count)
                .map(|_| ItemId::new())
                .collect(),
        },
        Some(f.session),
    )
    .await;
    f.host(
        TableAction::PrepareBattlefield {
            setup: Box::new(TableBattlefieldSetup {
                encounter_id: EncounterId::new(),
                scene_id: SceneId::new(),
                location_id: LocationId::new(),
                name: "Private source scene".into(),
                battlefield: Battlefield {
                    bounds: SpatialBox {
                        min: SpatialPoint { x: 0, y: 0, z: 0 },
                        max: SpatialPoint {
                            x: 100,
                            y: 100,
                            z: 40,
                        },
                    },
                    floor_z: 0,
                    floor_surface: "stone".into(),
                    ambient_light: LightLevel::Bright,
                    terrain: vec![],
                    obstacles: vec![],
                    lights: vec![],
                },
                characters: vec![TableCharacterPlacement {
                    character_id: f.characters[0],
                    position: SpatialPoint { x: 10, y: 10, z: 0 },
                    height: 12,
                    allies: vec![],
                    enemies: vec![actor],
                }],
                creatures: vec![TableCreaturePlacement {
                    actor,
                    public_label: "Small armored figure".into(),
                    position,
                    height: 8,
                    allies: vec![],
                    enemies: vec![f.actors[0]],
                }],
                area_grid_policy: None,
                geometry_ruling: Ruling {
                    basis: RulingBasis::GmAdjudication,
                    reason: "Host established visible terrain and public descriptors.".into(),
                },
            }),
        },
        Some(f.session),
    )
    .await;
}

pub(super) async fn prepare(f: &Fixture) {
    for character in f.characters {
        let view = f
            .runtime
            .table_view(f.campaign, TableViewer::Host)
            .await
            .unwrap();
        let equipment = view
            .characters
            .iter()
            .find(|c| c.character_id == character)
            .unwrap()
            .equipment
            .as_ref()
            .unwrap();
        let action = TableAction::PrepareEquipment {
            character_id: character,
            item_ids: (0..equipment.initial_item_count)
                .map(|_| ItemId::new())
                .collect(),
        };
        f.runtime
            .execute_table(
                f.meta(CommandIssuer::Admin, None, Some(f.session)).await,
                action,
            )
            .await
            .unwrap();
    }
    let point = |x, y, z| SpatialPoint { x, y, z };
    f.runtime
        .execute_table(
            f.meta(CommandIssuer::Admin, None, Some(f.session)).await,
            TableAction::PrepareBattlefield {
                setup: Box::new(TableBattlefieldSetup {
                    encounter_id: EncounterId::new(),
                    scene_id: SceneId::new(),
                    location_id: LocationId::new(),
                    name: "Unannounced host location".into(),
                    battlefield: Battlefield {
                        bounds: SpatialBox {
                            min: point(0, 0, 0),
                            max: point(100, 100, 40),
                        },
                        floor_z: 0,
                        floor_surface: "stone".into(),
                        ambient_light: LightLevel::Darkness,
                        terrain: vec![],
                        obstacles: vec![],
                        lights: vec![],
                    },
                    characters: f
                        .characters
                        .iter()
                        .enumerate()
                        .map(|(index, id)| TableCharacterPlacement {
                            character_id: *id,
                            position: point(10 + index as i32 * 40, 10, 0),
                            height: 12,
                            allies: vec![],
                            enemies: vec![],
                        })
                        .collect(),
                    creatures: vec![],
                    area_grid_policy: None,
                    geometry_ruling: Ruling {
                        basis: RulingBasis::GmAdjudication,
                        reason: "Host established bounded physical terrain.".into(),
                    },
                }),
            },
        )
        .await
        .unwrap();
}

pub(super) fn begin(f: &Fixture) -> TableAction {
    TableAction::Tactical {
        action: TacticalAction::Begin {
            combatants: f
                .actors
                .iter()
                .map(|actor| TacticalCombatant {
                    actor: *actor,
                    source: TacticalSource::Character,
                    surprised: false,
                })
                .collect(),
            groups: f
                .actors
                .iter()
                .map(|actor| InitiativeGroup {
                    actors: vec![*actor],
                    request_id: RollRequestId::new(),
                })
                .collect(),
        },
    }
}

pub(super) async fn roll(f: &Fixture, index: usize, face: u16) -> TableAction {
    let view = f
        .runtime
        .table_view(f.campaign, TableViewer::Player(f.players[index]))
        .await
        .unwrap();
    let request = view.roll.unwrap();
    TableAction::Tactical {
        action: TacticalAction::SubmitRoll {
            result: RollResult {
                request_id: request.id,
                source: RollSource::Physical,
                dice: vec![DieResult {
                    sides: 20,
                    value: face,
                }],
            },
        },
    }
}

#[tokio::test]
async fn session_bound_tactical_rolls_restart_and_restore_without_hidden_map_truth() {
    let mut f = Fixture::new().await;
    f.host(TableAction::EndSession, Some(f.session)).await;
    f.session = PlaySessionId::new();
    f.host(
        TableAction::StartSession {
            id: f.session,
            name: "Both players present".into(),
            participants: (0..2)
                .map(|index| SessionParticipant {
                    player_id: f.players[index],
                    character_id: Some(f.characters[index]),
                    attendance: AttendanceStatus::Present,
                })
                .collect(),
        },
        Some(f.session),
    )
    .await;
    prepare(&f).await;
    let host = f
        .runtime
        .table_view(f.campaign, TableViewer::Host)
        .await
        .unwrap()
        .tactical
        .unwrap();
    assert!(host.battlefield.is_some());
    assert_eq!(host.participants.len(), 2);
    let player = f
        .runtime
        .table_view(f.campaign, TableViewer::Player(f.players[0]))
        .await
        .unwrap();
    let tactical = player.tactical.as_ref().unwrap();
    assert!(tactical.battlefield.is_none());
    assert!(tactical.participants.is_empty());
    assert_eq!(tactical.observers.len(), 1);
    assert!(tactical.observers[0].contacts.is_empty());
    assert!(
        !serde_json::to_string(tactical)
            .unwrap()
            .contains(&f.actors[1].0.to_string())
    );
    assert!(
        !player
            .transcript
            .iter()
            .any(|entry| entry.text.contains("Unannounced host location"))
    );
    let action = begin(&f);
    assert!(
        f.runtime
            .execute_table(f.player_meta(0).await, action.clone())
            .await
            .is_err()
    );
    let before = f
        .runtime
        .open_campaign(f.campaign)
        .await
        .unwrap()
        .state()
        .clone();
    let mut no_session = f.meta(CommandIssuer::Admin, None, Some(f.session)).await;
    no_session.session_id = None;
    assert!(
        f.runtime
            .execute_table(no_session, action.clone())
            .await
            .is_err()
    );
    assert_eq!(
        f.runtime.open_campaign(f.campaign).await.unwrap().state(),
        &before
    );
    f.runtime
        .execute_table(
            f.meta(CommandIssuer::Admin, None, Some(f.session)).await,
            action,
        )
        .await
        .unwrap();
    let first = roll(&f, 0, 15).await;
    assert!(
        f.runtime
            .execute_table(f.player_meta(1).await, first.clone())
            .await
            .is_err()
    );
    f.runtime
        .execute_table(f.player_meta(0).await, first)
        .await
        .unwrap();
    let pending = f
        .runtime
        .open_campaign(f.campaign)
        .await
        .unwrap()
        .state()
        .clone();
    let export = export_campaign(&f.pool, f.campaign).await.unwrap();
    let target = open_sqlite("sqlite::memory:").await.unwrap();
    let restored = CampaignRuntime::from_content_root(
        target.clone(),
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../content"),
    );
    restored.restore_campaign(&export).await.unwrap();
    assert_eq!(
        restored.resume_campaign(f.campaign).await.unwrap().state(),
        &pending
    );
    let second = roll(&f, 1, 8).await;
    let meta = f.player_meta(1).await;
    f.runtime
        .execute_table(meta.clone(), second.clone())
        .await
        .unwrap();
    restored.execute_table(meta, second).await.unwrap();
    let active = f
        .runtime
        .open_campaign(f.campaign)
        .await
        .unwrap()
        .state()
        .clone();
    assert_eq!(
        restored.resume_campaign(f.campaign).await.unwrap().state(),
        &active
    );
    assert_eq!(
        active
            .rules
            .as_ref()
            .unwrap()
            .timing
            .as_ref()
            .unwrap()
            .order[0]
            .actor,
        f.actors[0]
    );
    assert!(
        f.runtime
            .execute_table(
                f.meta(CommandIssuer::Admin, None, Some(f.session)).await,
                TableAction::EndSession
            )
            .await
            .is_err()
    );

    let mut corrupt = export.clone();
    let event = corrupt.event_journal.last_mut().unwrap();
    let mut payload: serde_json::Value = serde_json::from_str(&event.payload_json).unwrap();
    payload["tactical_event"]["meta"]["id"] = serde_json::json!(CommandId::new());
    event.payload_json = payload.to_string();
    let bad_pool = open_sqlite("sqlite::memory:").await.unwrap();
    let bad = CampaignRuntime::from_content_root(
        bad_pool.clone(),
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../content"),
    );
    assert!(bad.restore_campaign(&corrupt).await.is_err());
    assert!(
        dmd_persistence::open_campaign(&bad_pool, f.campaign)
            .await
            .is_err()
    );
    target.close().await;
    bad_pool.close().await;
    f.pool.close().await;
}
