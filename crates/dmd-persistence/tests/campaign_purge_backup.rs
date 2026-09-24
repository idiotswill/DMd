use std::str::FromStr;

use dmd_domain::{
    Campaign, CampaignId, CampaignState, CampaignStatus, VersionedRef, WorldClock, WorldInstant,
};
use dmd_persistence::{
    CampaignPurgeAuthorization, CampaignStorageStatus, LifecycleError, archive_campaign,
    create_campaign, export_campaign, migrate_sqlite, open_campaign, purge_campaign,
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

fn state() -> CampaignState {
    let campaign_id = CampaignId::new();
    CampaignState::empty(
        Campaign {
            id: campaign_id,
            display_name: "Backup race".into(),
            status: CampaignStatus::Active,
            world_seed: 101,
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

#[tokio::test]
async fn purge_rejects_backup_when_lifecycle_metadata_changed_after_export() {
    let pool = test_pool().await;
    let campaign = state();
    create_campaign(&pool, &campaign)
        .await
        .expect("campaign should create");

    let stale_backup = export_campaign(&pool, campaign.campaign_id())
        .await
        .expect("campaign should export");
    archive_campaign(&pool, campaign.campaign_id())
        .await
        .expect("archive should persist");

    assert!(matches!(
        purge_campaign(
            &pool,
            campaign.campaign_id(),
            &CampaignPurgeAuthorization::admin("stale backup must not authorize deletion"),
            &stale_backup,
        )
        .await,
        Err(LifecycleError::StalePurgeBackup)
    ));

    let reopened = open_campaign(&pool, campaign.campaign_id())
        .await
        .expect("stale purge must leave campaign intact");
    assert_eq!(
        reopened.lifecycle.storage_status,
        CampaignStorageStatus::Archived
    );

    let fresh_backup = export_campaign(&pool, campaign.campaign_id())
        .await
        .expect("archived campaign should export");
    purge_campaign(
        &pool,
        campaign.campaign_id(),
        &CampaignPurgeAuthorization::admin("fresh backup authorizes deletion"),
        &fresh_backup,
    )
    .await
    .expect("fresh backup should authorize purge");
}
