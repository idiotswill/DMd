use super::*;
use dmd_rules::tactical::TacticalAction;

fn runtime(pool: sqlx::SqlitePool) -> CampaignRuntime {
    CampaignRuntime::from_content_root(
        pool,
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../content"),
    )
}
async fn state(f: &Fixture) -> CampaignState {
    f.runtime
        .open_campaign(f.campaign)
        .await
        .unwrap()
        .state()
        .clone()
}
async fn view(f: &Fixture, player: usize) -> TablePresentedView {
    f.runtime
        .presented_table_view(f.campaign, TableViewer::Player(f.players[player]))
        .await
        .unwrap()
}
async fn request(f: &Fixture, action: TacticalAction) -> TableTransportRequest {
    TableTransportRequest {
        version: TABLE_TRANSPORT_VERSION,
        command_id: CommandId::new(),
        campaign_id: f.campaign,
        session_id: Some(f.session),
        channel: TableTransportChannel::Player {
            player_id: f.players[0],
            character_id: f.characters[0],
        },
        revision: view(f, 0).await.revision,
        input: TableTransportInput::Action(Box::new(TableAction::Tactical { action })),
    }
}
async fn cold_step(f: &mut Fixture, url: &str, request: TableTransportRequest) {
    let exported = export_campaign(&f.pool, f.campaign).await.unwrap();
    let pool = open_sqlite("sqlite::memory:").await.unwrap();
    let mirror = runtime(pool.clone());
    Box::pin(mirror.restore_campaign(&exported)).await.unwrap();
    f.pool.close().await;
    f.pool = open_sqlite(url).await.unwrap();
    f.runtime = runtime(f.pool.clone());
    let accepted = Box::pin(f.runtime.submit_presented_table(request.clone()))
        .await
        .unwrap();
    Box::pin(mirror.submit_presented_table(request.clone()))
        .await
        .unwrap();
    assert_eq!(
        mirror.open_campaign(f.campaign).await.unwrap().state(),
        &state(f).await
    );
    pool.close().await;
    f.pool.close().await;
    f.pool = open_sqlite(url).await.unwrap();
    f.runtime = runtime(f.pool.clone());
    let saved = export_campaign(&f.pool, f.campaign).await.unwrap();
    assert_eq!(
        Box::pin(f.runtime.submit_presented_table(request.clone()))
            .await
            .unwrap(),
        accepted
    );
    let mut changed = request;
    changed.input = TableTransportInput::Action(Box::new(TableAction::Tactical {
        action: TacticalAction::Dodge,
    }));
    assert!(
        Box::pin(f.runtime.submit_presented_table(changed))
            .await
            .is_err()
    );
    let mut after = export_campaign(&f.pool, f.campaign).await.unwrap();
    after.exported_at_utc = saved.exported_at_utc.clone();
    assert_eq!(after, saved);
}
async fn reject_forged_budget(f: &Fixture) {
    let mut forged = state(f).await;
    forged
        .rules
        .as_mut()
        .unwrap()
        .timing
        .as_mut()
        .unwrap()
        .bonus_action_spent = true;
    dmd_rules::tactical::validate_tactical_state(&forged).unwrap();
    let mut exported = export_campaign(&f.pool, f.campaign).await.unwrap();
    exported.current_state.state_json = forged.encode_json().unwrap();
    let pool = open_sqlite("sqlite::memory:").await.unwrap();
    assert!(
        Box::pin(runtime(pool.clone()).restore_campaign(&exported))
            .await
            .is_err()
    );
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM campaign_state_current")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 0);
    pool.close().await;
}
async fn exercise(f: &mut Fixture, url: &str) {
    view(f, 0).await; // New opaque presentation begins before accepting this feature.
    let target = Box::pin(table_attack_cases::prepare_at(
        f,
        SpatialPoint { x: 20, y: 10, z: 0 },
    ))
    .await;
    let original = state(f).await;
    let initial_hp = original.rules.as_ref().unwrap().entities[&target].hp;
    assert_eq!(initial_hp, 10); // Actual Goblin Warrior source, not an HP patch.
    let initial_equipment = original.rules.as_ref().unwrap().tactical_inventory.clone();
    for round in 0..3 {
        let attack = request(f, TacticalAction::UnarmedStrike { target }).await;
        Box::pin(cold_step(f, url, attack)).await;
        if round == 0 {
            Box::pin(reject_forged_budget(f)).await;
        }
        let owned = view(f, 0).await;
        let roll = owned.roll.unwrap();
        assert_eq!(roll.modifier, 5);
        assert_eq!(
            roll.dice,
            vec![DieSpec {
                count: 1,
                sides: 20
            }]
        );
        assert_ne!(
            roll.id,
            state(f).await.rules.unwrap().pending.unwrap().request.id
        );
        let raw = request(
            f,
            TacticalAction::SubmitRoll {
                result: RollResult {
                    request_id: roll.id,
                    source: RollSource::Physical,
                    dice: vec![DieResult {
                        sides: 20,
                        value: 20,
                    }],
                },
            },
        )
        .await;
        let mut foreign = raw.clone();
        foreign.command_id = CommandId::new();
        foreign.channel = TableTransportChannel::Player {
            player_id: f.players[1],
            character_id: f.characters[1],
        };
        foreign.revision = view(f, 1).await.revision;
        let before = state(f).await;
        assert!(
            Box::pin(f.runtime.submit_presented_table(foreign))
                .await
                .is_err()
        );
        assert_eq!(state(f).await, before);
        Box::pin(cold_step(f, url, raw)).await;
        let current = state(f).await;
        let rules = current.rules.as_ref().unwrap();
        assert!(rules.pending.is_none()); // A critical never fabricates a fixed-damage die.
        assert_eq!(
            rules.rolls.last().unwrap().result.dice,
            vec![DieResult {
                sides: 20,
                value: 20
            }]
        );
        assert!(rules.timing.as_ref().unwrap().action_spent);
        assert!(!rules.timing.as_ref().unwrap().bonus_action_spent);
        assert!(rules.timing.as_ref().unwrap().reactions_spent.is_empty());
        assert_eq!(rules.tactical_inventory, initial_equipment);
        assert!(
            current
                .encounter
                .as_ref()
                .unwrap()
                .flow
                .as_ref()
                .unwrap()
                .budget
                .weapon_history
                .is_empty()
        );
        if round < 2 {
            assert_eq!(rules.entities[&target].hp, initial_hp - 4 * (round + 1));
            assert!(
                current
                    .encounter
                    .as_ref()
                    .unwrap()
                    .flow
                    .as_ref()
                    .unwrap()
                    .resolution
                    .is_none()
            );
            let duplicate = request(f, TacticalAction::UnarmedStrike { target }).await;
            assert!(
                Box::pin(f.runtime.submit_presented_table(duplicate))
                    .await
                    .is_err()
            );
            let end = request(f, TacticalAction::EndTurn).await;
            Box::pin(f.runtime.submit_presented_table(end))
                .await
                .unwrap();
            f.host(
                TableAction::Tactical {
                    action: TacticalAction::EndTurn,
                },
                Some(f.session),
            )
            .await;
        } else {
            assert_eq!(rules.entities[&target].hp, 2); // Damage waits for the actual knockout choice.
            let knockout = request(
                f,
                TacticalAction::ChooseAttackKnockout {
                    choice: KnockoutChoice::KnockOut,
                },
            )
            .await;
            Box::pin(cold_step(f, url, knockout)).await;
        }
    }
    let current = state(f).await;
    assert_eq!(current.rules.as_ref().unwrap().entities[&target].hp, 1);
    assert!(
        current
            .rules
            .as_ref()
            .unwrap()
            .tactical_recovery
            .as_ref()
            .unwrap()[&target]
            .knockout
            .is_some()
    );
    assert!(
        current
            .encounter
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .resolution
            .is_none()
    );
}

#[tokio::test]
async fn actual_unarmed_attacks_and_knockout_survive_owned_cold_dice_retry_and_semantic_restore() {
    let directory = std::env::temp_dir().join(format!("dmd-unarmed-{}", CampaignId::new().0));
    std::fs::create_dir_all(&directory).unwrap();
    let url = format!(
        "sqlite://{}",
        directory
            .join("campaign.sqlite")
            .to_string_lossy()
            .replace('\\', "/")
    );
    let pool = open_sqlite(&url).await.unwrap();
    let mut f = Box::pin(Fixture::with_pool(TableContract::default(), pool)).await;
    Box::pin(exercise(&mut f, &url)).await;
    f.pool.close().await;
    drop(f);
    sqlite_test_cleanup::remove_closed_directory(&directory)
        .await
        .unwrap();
}
