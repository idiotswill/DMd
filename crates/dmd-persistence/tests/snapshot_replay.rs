use std::str::FromStr;

use dmd_domain::{
    Campaign, CampaignId, CampaignState, CampaignStatus, CommandId, CommandIssuer, CommandMeta,
    EventId, EventSource, PendingEvent, SerializedRecord, VersionedRef, WorldClock, WorldDuration,
    WorldInstant,
};
use dmd_persistence::{
    CampaignStateSnapshotCodec, ReplayApplyError, ReplayEventApplier, SNAPSHOT_EVENT_INTERVAL,
    SnapshotCodecError, SnapshotMigration, SnapshotReplayError, StoredJournalEvent,
    commit_campaign_transition, initialize_campaign_state, load_campaign_snapshot_at_or_before,
    migrate_sqlite, replay_campaign_to_head, replay_campaign_to_sequence,
};
use sqlx::{
    Row,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};

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
            display_name: "Snapshot Replay Test".into(),
            status: CampaignStatus::Active,
            world_seed: 17,
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

fn command_payload(duration: WorldDuration) -> SerializedRecord {
    SerializedRecord::encode("test.advance_time", 1, &duration)
        .expect("typed command should encode")
}

fn event(
    id: EventId,
    duration: WorldDuration,
    at: WorldInstant,
    schema_version: u32,
) -> dmd_domain::EncodedPendingEvent {
    PendingEvent {
        id,
        occurred_at: at,
        source: EventSource::WorldSimulation,
        actor: None,
        caused_by_event_ids: vec![],
        payload: duration,
    }
    .encode("test.time_advanced", schema_version)
    .expect("typed event should encode")
}

fn advance(current: &CampaignState, duration: WorldDuration, event_count: u64) -> CampaignState {
    let mut next = current.clone();
    next.clock.now = next.clock.now.advance(duration);
    next.applied_event_sequence += event_count;
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
        let duration: WorldDuration = serde_json::from_str(&event.payload.json).map_err(|error| {
            ReplayApplyError::InvalidPayload {
                kind: event.payload.kind.clone(),
                schema_version: event.payload.schema_version,
                message: error.to_string(),
            }
        })?;
        state.clock.now = state.clock.now.advance(duration);
        Ok(())
    }
}

struct ZeroToOneMigration;

impl SnapshotMigration for ZeroToOneMigration {
    fn from_version(&self) -> u32 {
        0
    }

    fn migrate_json(&self, json: &str) -> Result<String, String> {
        Ok(json.replacen("\"schema_version\":0", "\"schema_version\":1", 1))
    }
}

#[tokio::test]
async fn new_campaign_gets_sequence_zero_snapshot_and_snapshot_is_immutable() {
    let pool = test_pool().await;
    let initial = state();
    initialize_campaign_state(&pool, &initial)
        .await
        .expect("campaign should initialize");

    let row = sqlx::query(
        "SELECT event_sequence, state_schema_version, state_json FROM campaign_snapshots WHERE campaign_id = ?",
    )
    .bind(initial.campaign_id().0.to_string())
    .fetch_one(&pool)
    .await
    .expect("snapshot should exist");
    let sequence: i64 = row.try_get("event_sequence").expect("event sequence");
    let schema_version: i64 = row
        .try_get("state_schema_version")
        .expect("schema version");
    let json: String = row.try_get("state_json").expect("state JSON");
    assert_eq!(sequence, 0);
    assert_eq!(schema_version, 1);
    assert_eq!(json, initial.encode_json().expect("state should encode"));

    let update = sqlx::query(
        "UPDATE campaign_snapshots SET state_json = '{}' WHERE campaign_id = ? AND event_sequence = 0",
    )
    .bind(initial.campaign_id().0.to_string())
    .execute(&pool)
    .await;
    assert!(update.is_err(), "snapshot update must be rejected");

    let delete = sqlx::query(
        "DELETE FROM campaign_snapshots WHERE campaign_id = ? AND event_sequence = 0",
    )
    .bind(initial.campaign_id().0.to_string())
    .execute(&pool)
    .await;
    assert!(delete.is_err(), "snapshot delete must be rejected");
}

#[tokio::test]
async fn periodic_snapshot_is_created_after_interval_crossing() {
    let pool = test_pool().await;
    let initial = state();
    initialize_campaign_state(&pool, &initial)
        .await
        .expect("campaign should initialize");

    let events = (0..SNAPSHOT_EVENT_INTERVAL)
        .map(|offset| {
            event(
                EventId::new(),
                WorldDuration(1),
                WorldInstant(101 + i64::try_from(offset).expect("small test offset")),
                1,
            )
        })
        .collect::<Vec<_>>();
    let next = advance(
        &initial,
        WorldDuration(i64::try_from(SNAPSHOT_EVENT_INTERVAL).expect("small interval")),
        SNAPSHOT_EVENT_INTERVAL,
    );
    commit_campaign_transition(
        &pool,
        &command_meta(initial.campaign_id(), 0),
        &command_payload(WorldDuration(1)),
        &next,
        &events,
        "Advance enough events to cross the snapshot interval.",
    )
    .await
    .expect("transition should commit");

    let sequences = sqlx::query_scalar::<_, i64>(
        "SELECT event_sequence FROM campaign_snapshots WHERE campaign_id = ? ORDER BY event_sequence",
    )
    .bind(initial.campaign_id().0.to_string())
    .fetch_all(&pool)
    .await
    .expect("snapshot sequences should load");
    assert_eq!(
        sequences,
        vec![
            0,
            i64::try_from(SNAPSHOT_EVENT_INTERVAL).expect("small interval")
        ]
    );
}

#[tokio::test]
async fn replay_from_sequence_zero_reconstructs_current_head() {
    let pool = test_pool().await;
    let initial = state();
    initialize_campaign_state(&pool, &initial)
        .await
        .expect("campaign should initialize");

    let first_duration = WorldDuration(3);
    let after_first = advance(&initial, first_duration, 1);
    commit_campaign_transition(
        &pool,
        &command_meta(initial.campaign_id(), 0),
        &command_payload(first_duration),
        &after_first,
        &[event(
            EventId::new(),
            first_duration,
            after_first.clock.now,
            1,
        )],
        "First replayable change.",
    )
    .await
    .expect("first transition should commit");

    let second_duration = WorldDuration(4);
    let after_second = advance(&after_first, second_duration, 1);
    commit_campaign_transition(
        &pool,
        &command_meta(initial.campaign_id(), 1),
        &command_payload(second_duration),
        &after_second,
        &[event(
            EventId::new(),
            second_duration,
            after_second.clock.now,
            1,
        )],
        "Second replayable change.",
    )
    .await
    .expect("second transition should commit");

    let replayed = replay_campaign_to_head(&pool, initial.campaign_id(), &ClockApplier)
        .await
        .expect("replay should reconstruct the current state");
    assert_eq!(replayed, after_second);

    let historical = replay_campaign_to_sequence(&pool, initial.campaign_id(), 1, &ClockApplier)
        .await
        .expect("historical replay should stop at sequence one");
    assert_eq!(historical, after_first);
}

#[tokio::test]
async fn replay_rejects_unsupported_event_version() {
    let pool = test_pool().await;
    let initial = state();
    initialize_campaign_state(&pool, &initial)
        .await
        .expect("campaign should initialize");

    let duration = WorldDuration(1);
    let next = advance(&initial, duration, 1);
    commit_campaign_transition(
        &pool,
        &command_meta(initial.campaign_id(), 0),
        &command_payload(duration),
        &next,
        &[event(EventId::new(), duration, next.clock.now, 2)],
        "Persist an event version the test applier does not support.",
    )
    .await
    .expect("persistence should preserve the typed version without guessing semantics");

    let result = replay_campaign_to_head(&pool, initial.campaign_id(), &ClockApplier).await;
    assert!(matches!(
        result,
        Err(SnapshotReplayError::EventApply {
            source: ReplayApplyError::UnsupportedEvent { schema_version: 2, .. },
            ..
        })
    ));
}

#[tokio::test]
async fn replay_rejects_target_beyond_head() {
    let pool = test_pool().await;
    let initial = state();
    initialize_campaign_state(&pool, &initial)
        .await
        .expect("campaign should initialize");

    let result = replay_campaign_to_sequence(&pool, initial.campaign_id(), 1, &ClockApplier).await;
    assert!(matches!(
        result,
        Err(SnapshotReplayError::TargetBeyondHead { target: 1, head: 0 })
    ));
}

#[test]
fn snapshot_codec_rejects_future_and_missing_legacy_migration() {
    let current = state();
    let current_json = current.encode_json().expect("state should encode");
    let codec = CampaignStateSnapshotCodec::new();

    assert!(matches!(
        codec.decode_state(2, &current_json),
        Err(SnapshotCodecError::UnsupportedFutureVersion {
            actual: 2,
            supported: 1
        })
    ));

    let mut legacy = current;
    legacy.schema_version = 0;
    let legacy_json = legacy.encode_json().expect("legacy fixture should encode");
    assert!(matches!(
        codec.decode_state(0, &legacy_json),
        Err(SnapshotCodecError::MissingMigration { from_version: 0 })
    ));
}

#[test]
fn snapshot_codec_applies_registered_sequential_migration() {
    let mut legacy = state();
    legacy.schema_version = 0;
    let legacy_json = legacy.encode_json().expect("legacy fixture should encode");

    let mut codec = CampaignStateSnapshotCodec::new();
    codec
        .register(ZeroToOneMigration)
        .expect("migration should register");
    let migrated = codec
        .decode_state(0, &legacy_json)
        .expect("registered migration should reach current schema");

    assert_eq!(migrated.schema_version, 1);
    assert_eq!(migrated.campaign_id(), legacy.campaign_id());
}

#[tokio::test]
async fn migration_sql_backfills_existing_materialized_head() {
    let options = SqliteConnectOptions::from_str("sqlite::memory:")
        .expect("valid SQLite URL")
        .foreign_keys(true);
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await
        .expect("in-memory SQLite should open");

    sqlx::raw_sql(
        r#"
        CREATE TABLE campaign_state_current (
            campaign_id TEXT PRIMARY KEY NOT NULL,
            schema_version INTEGER NOT NULL CHECK (schema_version > 0),
            applied_event_sequence INTEGER NOT NULL CHECK (applied_event_sequence >= 0),
            state_json TEXT NOT NULL
        );
        "#,
    )
    .execute(&pool)
    .await
    .expect("legacy current-state table should exist");

    let mut current = state();
    current.applied_event_sequence = 7;
    let current_json = current.encode_json().expect("state should encode");
    sqlx::query(
        "INSERT INTO campaign_state_current (campaign_id, schema_version, applied_event_sequence, state_json) VALUES (?, ?, ?, ?)",
    )
    .bind(current.campaign_id().0.to_string())
    .bind(i64::from(current.schema_version))
    .bind(7_i64)
    .bind(&current_json)
    .execute(&pool)
    .await
    .expect("legacy current state should insert");

    sqlx::raw_sql(include_str!("../migrations/0005_campaign_snapshots.sql"))
        .execute(&pool)
        .await
        .expect("snapshot migration SQL should run");

    let row = sqlx::query(
        "SELECT event_sequence, state_schema_version, state_json FROM campaign_snapshots WHERE campaign_id = ?",
    )
    .bind(current.campaign_id().0.to_string())
    .fetch_one(&pool)
    .await
    .expect("backfilled snapshot should exist");
    let sequence: i64 = row.try_get("event_sequence").expect("event sequence");
    let schema_version: i64 = row
        .try_get("state_schema_version")
        .expect("schema version");
    let json: String = row.try_get("state_json").expect("state JSON");
    assert_eq!(sequence, 7);
    assert_eq!(schema_version, 1);
    assert_eq!(json, current_json);
}

#[tokio::test]
async fn snapshot_lookup_returns_none_before_oldest_available_snapshot() {
    let pool = test_pool().await;
    let mut current = state();
    current.applied_event_sequence = 5;
    let current_json = current.encode_json().expect("state should encode");
    sqlx::query(
        "INSERT INTO campaign_state_current (campaign_id, schema_version, applied_event_sequence, state_json) VALUES (?, ?, ?, ?)",
    )
    .bind(current.campaign_id().0.to_string())
    .bind(i64::from(current.schema_version))
    .bind(5_i64)
    .bind(current_json)
    .execute(&pool)
    .await
    .expect("state insert should trigger a snapshot at sequence five");

    let codec = CampaignStateSnapshotCodec::new();
    let snapshot = load_campaign_snapshot_at_or_before(
        &pool,
        current.campaign_id(),
        4,
        &codec,
    )
    .await
    .expect("snapshot lookup should succeed");
    assert!(snapshot.is_none());
}
