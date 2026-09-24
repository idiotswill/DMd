use std::{fs, path::PathBuf, str::FromStr};

use dmd_domain::{
    Campaign, CampaignId, CampaignState, CampaignStatus, CommandId, CommandIssuer, CommandMeta,
    EventId, EventSource, PendingEvent, SerializedRecord, VersionedRef, WorldClock, WorldDuration,
    WorldInstant,
};
use dmd_persistence::{
    CampaignExport, CampaignPurgeAuthorization, CampaignStorageStatus, LifecycleError,
    archive_campaign, commit_campaign_transition, create_campaign, export_campaign, list_campaigns,
    migrate_sqlite, open_campaign, open_sqlite, purge_campaign, restore_campaign,
};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

async fn test_pool() -> sqlx::SqlitePool {
    let options = SqliteConnectOptions::from_str("sqlite::memory:")
        .expect("valid SQLite URL")
        .foreign_keys(true);
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await
        .expect("in-memory SQLite should open");
    migrate_sqlite(&pool)
        .await
        .expect("migrations should initialize");
    pool
}

fn state(name: &str, seed: u64) -> CampaignState {
    let campaign_id = CampaignId::new();
    CampaignState::empty(
        Campaign {
            id: campaign_id,
            display_name: name.into(),
            status: CampaignStatus::Active,
            world_seed: seed,
            ruleset: VersionedRef {
                id: "test.rules".into(),
                version: "1".into(),
            },
            content_packs: vec![],
        },
        WorldClock {
            now: WorldInstant(100),
            calendar_id: "test.calendar".into(),
        },
    )
}

fn command_meta(campaign_id: CampaignId, expected_event_sequence: u64) -> CommandMeta {
    CommandMeta {
        id: CommandId::new(),
        campaign_id,
        session_id: None,
        issuer: CommandIssuer::System,
        actor: None,
        expected_event_sequence,
    }
}

fn command_payload() -> SerializedRecord {
    SerializedRecord::encode("test.advance_time", 1, &WorldDuration(1))
        .expect("command payload should encode")
}

fn event(at: WorldInstant) -> dmd_domain::EncodedPendingEvent {
    PendingEvent {
        id: EventId::new(),
        occurred_at: at,
        source: EventSource::WorldSimulation,
        actor: None,
        caused_by_event_ids: vec![],
        payload: WorldDuration(1),
    }
    .encode("test.time_advanced", 1)
    .expect("event should encode")
}

async fn advance_once(pool: &sqlx::SqlitePool, current: &CampaignState) -> CampaignState {
    let mut next = current.clone();
    next.clock.now = next.clock.now.advance(WorldDuration(1));
    next.applied_event_sequence += 1;
    let pending = event(next.clock.now);
    commit_campaign_transition(
        pool,
        &command_meta(current.campaign_id(), current.applied_event_sequence),
        &command_payload(),
        &next,
        &[pending],
        "advance lifecycle test clock",
    )
    .await
    .expect("transition should commit");
    next
}

#[tokio::test]
async fn realistic_multi_campaign_archive_export_purge_restore_sequence_is_isolated() {
    let pool = test_pool().await;
    let alpha = state("Alpha", 11);
    let beta = state("Beta", 22);
    create_campaign(&pool, &alpha)
        .await
        .expect("alpha should create");
    create_campaign(&pool, &beta)
        .await
        .expect("beta should create");
    let alpha_advanced = advance_once(&pool, &alpha).await;

    let archived = archive_campaign(&pool, alpha.campaign_id())
        .await
        .expect("archive should persist");
    assert_eq!(archived.storage_status, CampaignStorageStatus::Archived);

    let export = export_campaign(&pool, alpha.campaign_id())
        .await
        .expect("export should validate");
    assert_eq!(export.event_journal.len(), 1);
    assert!(!export.snapshots.is_empty());
    let encoded = export.to_json().expect("export should serialize");
    assert_eq!(
        CampaignExport::from_json(&encoded).expect("export should deserialize"),
        export
    );

    let backup = export.clone();
    purge_campaign(
        &pool,
        alpha.campaign_id(),
        &CampaignPurgeAuthorization::admin("replace campaign from tested backup"),
        &backup,
    )
    .await
    .expect("authorized purge should succeed");
    assert!(matches!(
        open_campaign(&pool, alpha.campaign_id()).await,
        Err(LifecycleError::CampaignNotFound)
    ));

    let beta_open = open_campaign(&pool, beta.campaign_id())
        .await
        .expect("unrelated campaign must survive purge");
    assert_eq!(beta_open.state, beta);

    let restored = restore_campaign(&pool, &backup)
        .await
        .expect("validated backup should restore");
    assert_eq!(restored.state, alpha_advanced);
    assert_eq!(
        restored.lifecycle.storage_status,
        CampaignStorageStatus::Archived
    );
    let reexported = export_campaign(&pool, alpha.campaign_id())
        .await
        .expect("restored campaign should re-export");
    assert_eq!(reexported.snapshots, backup.snapshots);
    assert_eq!(reexported.command_audit, backup.command_audit);
    assert_eq!(reexported.event_journal, backup.event_journal);
    assert_eq!(reexported.event_causes, backup.event_causes);

    let listed = list_campaigns(&pool).await.expect("list should work");
    assert_eq!(listed.len(), 2);
    assert!(
        listed
            .iter()
            .any(|row| row.campaign_id == alpha.campaign_id().0.to_string())
    );
    assert!(
        listed
            .iter()
            .any(|row| row.campaign_id == beta.campaign_id().0.to_string())
    );
}

#[tokio::test]
async fn ordinary_history_delete_guards_remain_active() {
    let pool = test_pool().await;
    let campaign = state("Guarded", 33);
    create_campaign(&pool, &campaign)
        .await
        .expect("campaign should create");
    let advanced = advance_once(&pool, &campaign).await;

    let event_id: String =
        sqlx::query_scalar("SELECT id FROM event_journal WHERE campaign_id = ? LIMIT 1")
            .bind(campaign.campaign_id().0.to_string())
            .fetch_one(&pool)
            .await
            .expect("event should exist");
    let error = sqlx::query("DELETE FROM event_journal WHERE id = ?")
        .bind(event_id)
        .execute(&pool)
        .await
        .expect_err("ordinary event deletion must remain blocked");
    assert!(
        error
            .to_string()
            .contains("event journal records are immutable")
    );

    let reopened = open_campaign(&pool, campaign.campaign_id())
        .await
        .expect("failed history surgery must not damage campaign");
    assert_eq!(reopened.state, advanced);
}

#[tokio::test]
async fn purge_requires_admin_and_valid_backup() {
    let pool = test_pool().await;
    let campaign = state("Protected", 44);
    create_campaign(&pool, &campaign)
        .await
        .expect("campaign should create");

    let backup = export_campaign(&pool, campaign.campaign_id())
        .await
        .expect("campaign should export before purge");
    let denied = CampaignPurgeAuthorization {
        issuer: CommandIssuer::System,
        reason: "not admin".into(),
    };
    assert!(matches!(
        purge_campaign(&pool, campaign.campaign_id(), &denied, &backup).await,
        Err(LifecycleError::PurgeRequiresAdmin)
    ));
    open_campaign(&pool, campaign.campaign_id())
        .await
        .expect("denied purge must leave campaign intact");

    sqlx::query("UPDATE campaign_state_current SET state_json = '{}' WHERE campaign_id = ?")
        .bind(campaign.campaign_id().0.to_string())
        .execute(&pool)
        .await
        .expect("structurally corrupt but valid JSON should be writable for the failure test");
    assert!(matches!(
        purge_campaign(
            &pool,
            campaign.campaign_id(),
            &CampaignPurgeAuthorization::admin("corrupt campaign"),
            &backup,
        )
        .await,
        Err(LifecycleError::CorruptExport(_))
    ));
    let still_exists: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM campaign_state_current WHERE campaign_id = ?")
            .bind(campaign.campaign_id().0.to_string())
            .fetch_one(&pool)
            .await
            .expect("existence query should work");
    assert_eq!(still_exists, 1);
}

#[tokio::test]
async fn incompatible_restore_leaves_other_campaign_untouched() {
    let pool = test_pool().await;
    let source = state("Source", 55);
    let other = state("Other", 66);
    create_campaign(&pool, &source)
        .await
        .expect("source should create");
    create_campaign(&pool, &other)
        .await
        .expect("other should create");
    let mut export = export_campaign(&pool, source.campaign_id())
        .await
        .expect("source should export");
    purge_campaign(
        &pool,
        source.campaign_id(),
        &CampaignPurgeAuthorization::admin("prepare incompatible restore test"),
        &export,
    )
    .await
    .expect("source should purge");

    export.format_version += 1;
    assert!(matches!(
        restore_campaign(&pool, &export).await,
        Err(LifecycleError::IncompatibleExportFormat { .. })
    ));
    let other_open = open_campaign(&pool, other.campaign_id())
        .await
        .expect("other campaign must remain intact");
    assert_eq!(other_open.state, other);
}

#[tokio::test]
async fn partial_restore_collision_rolls_back_entire_campaign() {
    let pool = test_pool().await;
    let source = state("Collision Source", 77);
    let other = state("Collision Other", 88);
    create_campaign(&pool, &source)
        .await
        .expect("source should create");
    create_campaign(&pool, &other)
        .await
        .expect("other should create");
    let source_advanced = advance_once(&pool, &source).await;
    let other_advanced = advance_once(&pool, &other).await;
    let mut export = export_campaign(&pool, source.campaign_id())
        .await
        .expect("source should export");
    let other_export = export_campaign(&pool, other.campaign_id())
        .await
        .expect("other should export");
    purge_campaign(
        &pool,
        source.campaign_id(),
        &CampaignPurgeAuthorization::admin("prepare collision restore test"),
        &export,
    )
    .await
    .expect("source should purge");

    let colliding_command = other_export.command_audit[0].id.clone();
    export.command_audit[0].id.clone_from(&colliding_command);
    export.event_journal[0]
        .command_id
        .clone_from(&colliding_command);
    assert!(matches!(
        restore_campaign(&pool, &export).await,
        Err(LifecycleError::Sqlx(_))
    ));
    assert!(matches!(
        open_campaign(&pool, source.campaign_id()).await,
        Err(LifecycleError::CampaignNotFound)
    ));
    let other_open = open_campaign(&pool, other.campaign_id())
        .await
        .expect("other campaign must survive failed restore");
    assert_eq!(other_open.state, other_advanced);
    assert_ne!(source_advanced, other_open.state);
}

#[tokio::test]
async fn restored_campaign_reopens_after_database_restart() {
    let mut path: PathBuf = std::env::temp_dir();
    path.push(format!("dmd-lifecycle-{}.sqlite", uuid::Uuid::new_v4()));
    let url = format!("sqlite://{}", path.display());
    let campaign = state("Restart", 99);
    let expected;

    {
        let pool = open_sqlite(&url).await.expect("database should open");
        create_campaign(&pool, &campaign)
            .await
            .expect("campaign should create");
        expected = advance_once(&pool, &campaign).await;
        archive_campaign(&pool, campaign.campaign_id())
            .await
            .expect("archive should persist");
        let backup = export_campaign(&pool, campaign.campaign_id())
            .await
            .expect("campaign should export before purge");
        purge_campaign(
            &pool,
            campaign.campaign_id(),
            &CampaignPurgeAuthorization::admin("restart restore test"),
            &backup,
        )
        .await
        .expect("campaign should purge with backup");
        restore_campaign(&pool, &backup)
            .await
            .expect("backup should restore");
        pool.close().await;
    }
    {
        let pool = open_sqlite(&url).await.expect("database should reopen");
        let reopened = open_campaign(&pool, campaign.campaign_id())
            .await
            .expect("restored campaign should reopen after restart");
        assert_eq!(reopened.state, expected);
        assert_eq!(
            reopened.lifecycle.storage_status,
            CampaignStorageStatus::Archived
        );
        pool.close().await;
    }

    if PathBuf::from(&path).exists() {
        fs::remove_file(&path).expect("temporary database should delete");
    }
}
