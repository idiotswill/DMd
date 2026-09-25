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

async fn reject_invented_declaration(f: &Fixture, ready: TacticalReady) {
    let mut forged = state(f).await;
    forged
        .encounter
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .ready
        .push(ready);
    // The paid turn still makes this structurally plausible, but the accepted
    // abandonment event prevents restoring it as current campaign history.
    dmd_rules::tactical::validate_tactical_state(&forged).unwrap();
    let mut exported = export_campaign(&f.pool, f.campaign).await.unwrap();
    exported.current_state.state_json = forged.encode_json().unwrap();
    let pool = open_sqlite("sqlite::memory:").await.unwrap();
    assert!(
        Box::pin(runtime(pool.clone()).restore_campaign(&exported))
            .await
            .is_err()
    );
    let rows: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM campaign_state_current")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(rows, 0);
    pool.close().await;
}

async fn exercise(f: &mut Fixture, url: &str) {
    view(f, 0).await;
    Box::pin(table_attack_cases::prepare_at(
        f,
        SpatialPoint { x: 20, y: 10, z: 0 },
    ))
    .await;
    let declaration = request(
        f,
        TacticalAction::Ready {
            trigger: ReadyTrigger::MovementFinished {
                subject: ReadySubject::AnyOther,
            },
            action: ReadyAction::Move,
        },
    )
    .await;
    Box::pin(f.runtime.submit_presented_table(declaration.clone()))
        .await
        .unwrap();
    let held = state(f).await;
    let record = held
        .encounter
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .ready[0]
        .clone();
    assert!(
        held.rules
            .as_ref()
            .unwrap()
            .timing
            .as_ref()
            .unwrap()
            .action_spent
    );
    assert_eq!(
        view(f, 0).await.tactical.as_ref().unwrap().ready,
        vec![TableReadyView {
            actor: f.actors[0],
            action: "Movement".into(),
            may_abandon: true
        }]
    );
    let foreign = view(f, 1).await;
    assert!(foreign.tactical.as_ref().unwrap().ready.is_empty());
    let host = f
        .runtime
        .presented_table_view(f.campaign, TableViewer::Host)
        .await
        .unwrap();
    assert!(!host.tactical.as_ref().unwrap().ready[0].may_abandon);
    let action = TacticalAction::AbandonReady { actor: f.actors[0] };
    let cancellation = request(f, action.clone()).await;
    let mut unauthorized = cancellation.clone();
    unauthorized.command_id = CommandId::new();
    unauthorized.channel = TableTransportChannel::Host;
    unauthorized.revision = host.revision;
    assert!(
        Box::pin(f.runtime.submit_presented_table(unauthorized))
            .await
            .is_err()
    );
    let mut unauthorized = cancellation.clone();
    unauthorized.command_id = CommandId::new();
    unauthorized.channel = TableTransportChannel::Player {
        player_id: f.players[1],
        character_id: f.characters[1],
    };
    unauthorized.revision = foreign.revision;
    assert!(
        Box::pin(f.runtime.submit_presented_table(unauthorized))
            .await
            .is_err()
    );
    assert_eq!(state(f).await, held);
    let exported = export_campaign(&f.pool, f.campaign).await.unwrap();
    let mirror_pool = open_sqlite("sqlite::memory:").await.unwrap();
    let mirror = runtime(mirror_pool.clone());
    Box::pin(mirror.restore_campaign(&exported)).await.unwrap();
    f.pool.close().await;
    f.pool = open_sqlite(url).await.unwrap();
    f.runtime = runtime(f.pool.clone());
    assert_eq!(state(f).await, held);
    let accepted = Box::pin(f.runtime.submit_presented_table(cancellation.clone()))
        .await
        .unwrap();
    Box::pin(mirror.submit_presented_table(cancellation.clone()))
        .await
        .unwrap();
    let after = state(f).await;
    assert_eq!(
        mirror.open_campaign(f.campaign).await.unwrap().state(),
        &after
    );
    assert!(
        after
            .encounter
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .ready
            .is_empty()
    );
    assert_eq!(after.rules, held.rules);
    assert_eq!(after.clock, held.clock);
    assert_eq!(view(f, 1).await, foreign); // Includes revision, transcript and opaque capabilities.
    mirror_pool.close().await;
    f.pool.close().await;
    f.pool = open_sqlite(url).await.unwrap();
    f.runtime = runtime(f.pool.clone());
    let saved = export_campaign(&f.pool, f.campaign).await.unwrap();
    assert_eq!(
        Box::pin(f.runtime.submit_presented_table(cancellation.clone()))
            .await
            .unwrap(),
        accepted
    );
    let mut changed = cancellation;
    changed.input = TableTransportInput::Action(Box::new(TableAction::Tactical {
        action: TacticalAction::Dodge,
    }));
    assert!(
        Box::pin(f.runtime.submit_presented_table(changed))
            .await
            .is_err()
    );
    let mut retried = export_campaign(&f.pool, f.campaign).await.unwrap();
    retried.exported_at_utc = saved.exported_at_utc.clone();
    assert_eq!(retried, saved);
    // A different command cannot repeat abandonment or reclaim its spent Action.
    let duplicate = request(f, action).await;
    assert!(
        Box::pin(f.runtime.submit_presented_table(duplicate))
            .await
            .is_err()
    );
    let dodge = request(f, TacticalAction::Dodge).await;
    assert!(
        Box::pin(f.runtime.submit_presented_table(dodge))
            .await
            .is_err()
    );
    Box::pin(reject_invented_declaration(f, record)).await;
}

#[tokio::test]
async fn owned_ready_abandonment_survives_cold_retry_without_changing_foreign_presentation() {
    let directory = std::env::temp_dir().join(format!("dmd-ready-abandon-{}", CampaignId::new().0));
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
