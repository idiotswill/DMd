use std::str::FromStr;

use dmd_domain::{
    Campaign, CampaignId, CampaignState, CampaignStatus, CommandId, CommandIssuer, CommandMeta,
    EventId, EventSource, PendingEvent, SerializedRecord, VersionedRef, WorldClock, WorldDuration,
    WorldInstant,
};
use dmd_persistence::{
    ReplayApplyError, ReplayEventApplier, SnapshotCodecError, SnapshotReplayError,
    StoredJournalEvent, commit_campaign_transition, initialize_campaign_state, migrate_sqlite,
    replay_campaign_to_head,
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
    CampaignState::empty(
        Campaign {
            id: CampaignId::new(),
            display_name: "Snapshot Recovery Test".into(),
            status: CampaignStatus::Active,
            world_seed: 23,
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

fn payload(duration: WorldDuration) -> SerializedRecord {
    SerializedRecord::encode("test.advance_time", 1, &duration)
        .expect("typed command should encode")
}

fn event(duration: WorldDuration, at: WorldInstant) -> dmd_domain::EncodedPendingEvent {
    PendingEvent {
        id: EventId::new(),
        occurred_at: at,
        source: EventSource::WorldSimulation,
        actor: None,
        caused_by_event_ids: vec![],
        payload: duration,
    }
    .encode("test.time_advanced", 1)
    .expect("typed event should encode")
}

fn advance(current: &CampaignState, duration: WorldDuration) -> CampaignState {
    let mut next = current.clone();
    next.clock.now = next.clock.now.advance(duration);
    next.applied_event_sequence += 1;
    next
}

struct ClockApplier;

impl ReplayEventApplier for ClockApplier {
    fn apply(
        &self,
        state: &mut CampaignState,
        event: &StoredJournalEvent,
    ) -> Result<(), ReplayApplyError> {
        if event.payload.kind != "test.time_advanced" || event.payload.schema_version != 1 {
            return Err(ReplayApplyError::UnsupportedEvent {
                kind: event.payload.kind.clone(),
                schema_version: event.payload.schema_version,
            });
        }
        let duration = event.payload.json.parse::<i64>().map_err(|error| {
            ReplayApplyError::InvalidPayload {
                kind: event.payload.kind.clone(),
                schema_version: event.payload.schema_version,
                message: error.to_string(),
            }
        })?;
        state.clock.now = state.clock.now.advance(WorldDuration(duration));
        Ok(())
    }
}

struct CampaignChangingApplier;

impl ReplayEventApplier for CampaignChangingApplier {
    fn apply(
        &self,
        state: &mut CampaignState,
        _event: &StoredJournalEvent,
    ) -> Result<(), ReplayApplyError> {
        state.campaign.id = CampaignId::new();
        Ok(())
    }
}

struct SchemaChangingApplier;

impl ReplayEventApplier for SchemaChangingApplier {
    fn apply(
        &self,
        state: &mut CampaignState,
        _event: &StoredJournalEvent,
    ) -> Result<(), ReplayApplyError> {
        state.schema_version += 1;
        Ok(())
    }
}

async fn commit_two_events(pool: &sqlx::SqlitePool) -> (CampaignState, CampaignState) {
    let initial = state();
    initialize_campaign_state(pool, &initial)
        .await
        .expect("campaign should initialize");

    let first_duration = WorldDuration(3);
    let after_first = advance(&initial, first_duration);
    commit_campaign_transition(
        pool,
        &command_meta(initial.campaign_id(), 0),
        &payload(first_duration),
        &after_first,
        &[event(first_duration, after_first.clock.now)],
        "First recovery event.",
    )
    .await
    .expect("first transition should commit");

    let second_duration = WorldDuration(4);
    let after_second = advance(&after_first, second_duration);
    commit_campaign_transition(
        pool,
        &command_meta(initial.campaign_id(), 1),
        &payload(second_duration),
        &after_second,
        &[event(second_duration, after_second.clock.now)],
        "Second recovery event.",
    )
    .await
    .expect("second transition should commit");

    (initial, after_second)
}

#[tokio::test]
async fn replay_recovers_when_materialized_current_state_is_corrupt() {
    let pool = test_pool().await;
    let (initial, expected) = commit_two_events(&pool).await;

    sqlx::query(
        "UPDATE campaign_state_current SET applied_event_sequence = 99, state_json = 'not-json' WHERE campaign_id = ?",
    )
    .bind(initial.campaign_id().0.to_string())
    .execute(&pool)
    .await
    .expect("fixture should corrupt only the materialized current-state artifact");

    let replayed = replay_campaign_to_head(&pool, initial.campaign_id(), &ClockApplier)
        .await
        .expect("journal plus snapshot should recover without trusting corrupt current state");
    assert_eq!(replayed, expected);
}

#[tokio::test]
async fn replay_rejects_a_gap_in_the_authoritative_journal() {
    let pool = test_pool().await;
    let (initial, _) = commit_two_events(&pool).await;

    sqlx::query("DROP TRIGGER tr_event_journal_no_delete")
        .execute(&pool)
        .await
        .expect("corruption fixture should disable the normal delete guard");
    sqlx::query("DELETE FROM event_journal WHERE campaign_id = ? AND sequence = 1")
        .bind(initial.campaign_id().0.to_string())
        .execute(&pool)
        .await
        .expect("fixture should create a journal gap");

    let result = replay_campaign_to_head(&pool, initial.campaign_id(), &ClockApplier).await;
    assert!(matches!(
        result,
        Err(SnapshotReplayError::CorruptJournalHead {
            event_count: 1,
            max_sequence: 2
        })
    ));
}

#[tokio::test]
async fn replay_fails_explicitly_when_no_snapshot_is_available() {
    let pool = test_pool().await;
    let initial = state();
    initialize_campaign_state(&pool, &initial)
        .await
        .expect("campaign should initialize");

    sqlx::query("DROP TRIGGER tr_campaign_snapshots_no_delete")
        .execute(&pool)
        .await
        .expect("corruption fixture should disable the snapshot delete guard");
    sqlx::query("DELETE FROM campaign_snapshots WHERE campaign_id = ?")
        .bind(initial.campaign_id().0.to_string())
        .execute(&pool)
        .await
        .expect("fixture should remove the only snapshot");

    let result = replay_campaign_to_head(&pool, initial.campaign_id(), &ClockApplier).await;
    assert!(matches!(
        result,
        Err(SnapshotReplayError::MissingSnapshot { target: 0 })
    ));
}

#[tokio::test]
async fn replay_rejects_snapshot_newer_than_supported_state_schema() {
    let pool = test_pool().await;
    let initial = state();
    initialize_campaign_state(&pool, &initial)
        .await
        .expect("campaign should initialize");

    sqlx::query("DROP TRIGGER tr_campaign_snapshots_no_update")
        .execute(&pool)
        .await
        .expect("corruption fixture should disable the snapshot update guard");
    sqlx::query(
        "UPDATE campaign_snapshots SET state_schema_version = 2 WHERE campaign_id = ? AND event_sequence = 0",
    )
    .bind(initial.campaign_id().0.to_string())
    .execute(&pool)
    .await
    .expect("fixture should mark the snapshot as a future version");

    let result = replay_campaign_to_head(&pool, initial.campaign_id(), &ClockApplier).await;
    assert!(matches!(
        result,
        Err(SnapshotReplayError::Codec(
            SnapshotCodecError::UnsupportedFutureVersion {
                actual: 2,
                supported: 1
            }
        ))
    ));
}

#[tokio::test]
async fn replay_rejects_applier_that_changes_campaign_identity() {
    let pool = test_pool().await;
    let initial = state();
    initialize_campaign_state(&pool, &initial)
        .await
        .expect("campaign should initialize");
    let next = advance(&initial, WorldDuration(1));
    commit_campaign_transition(
        &pool,
        &command_meta(initial.campaign_id(), 0),
        &payload(WorldDuration(1)),
        &next,
        &[event(WorldDuration(1), next.clock.now)],
        "Identity guard event.",
    )
    .await
    .expect("transition should commit");

    let result = replay_campaign_to_head(&pool, initial.campaign_id(), &CampaignChangingApplier).await;
    assert!(matches!(
        result,
        Err(SnapshotReplayError::CampaignIdentityChanged)
    ));
}

#[tokio::test]
async fn replay_rejects_applier_that_changes_state_schema() {
    let pool = test_pool().await;
    let initial = state();
    initialize_campaign_state(&pool, &initial)
        .await
        .expect("campaign should initialize");
    let next = advance(&initial, WorldDuration(1));
    commit_campaign_transition(
        &pool,
        &command_meta(initial.campaign_id(), 0),
        &payload(WorldDuration(1)),
        &next,
        &[event(WorldDuration(1), next.clock.now)],
        "Schema guard event.",
    )
    .await
    .expect("transition should commit");

    let result = replay_campaign_to_head(&pool, initial.campaign_id(), &SchemaChangingApplier).await;
    assert!(matches!(
        result,
        Err(SnapshotReplayError::StateSchemaChanged {
            actual: 2,
            expected: 1
        })
    ));
}
