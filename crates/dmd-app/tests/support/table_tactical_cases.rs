use super::*;
use dmd_rules::tactical::TacticalAction;

async fn prepare(f: &Fixture) {
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

fn begin(f: &Fixture) -> TableAction {
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

async fn roll(f: &Fixture, index: usize, face: u16) -> TableAction {
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
