use std::{borrow::Cow, str::FromStr};

use dmd_domain::{
    CURRENT_STATE_SCHEMA_VERSION, Campaign, CampaignId, CampaignState, CampaignStatus, CommandId,
    CommandIssuer, CommandMeta, EventId, EventSource, PendingEvent, SerializedRecord, VersionedRef,
    WorldClock, WorldDuration, WorldInstant,
};
use dmd_persistence::{
    CampaignExport, CampaignStateSnapshotCodec, MIGRATOR, ReplayApplyError, ReplayEventApplier,
    SnapshotCodecError, SnapshotMigration, StoredJournalEvent, commit_campaign_transition,
    create_campaign, export_campaign, load_campaign_projection_summary, migrate_sqlite,
    open_campaign, replay_campaign_to_head, restore_campaign,
};
use serde_json::{Value, json};
use sqlx::{
    Row, SqlitePool,
    migrate::Migrator,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};

async fn pool(legacy: bool) -> SqlitePool {
    let options = SqliteConnectOptions::from_str("sqlite::memory:")
        .expect("valid SQLite URL")
        .foreign_keys(true);
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await
        .expect("database opens");
    if legacy {
        let gate_one = Migrator {
            migrations: Cow::Owned(
                MIGRATOR
                    .iter()
                    .filter(|migration| migration.version < 8)
                    .cloned()
                    .collect(),
            ),
            ..Migrator::DEFAULT
        };
        gate_one.run(&pool).await.expect("Gate 1 migrations apply");
    } else {
        migrate_sqlite(&pool)
            .await
            .expect("current migrations apply");
    }
    pool
}

fn state() -> CampaignState {
    CampaignState::empty(
        Campaign {
            id: CampaignId::new(),
            display_name: "Compatibility campaign".into(),
            status: CampaignStatus::Active,
            world_seed: 391,
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

fn legacy_json(current: &str) -> String {
    let mut value: Value = serde_json::from_str(current).expect("state JSON");
    let object = value.as_object_mut().expect("state object");
    object.insert("schema_version".into(), json!(1));
    object.remove("rules");
    serde_json::to_string(&value).expect("legacy JSON")
}

async fn insert_legacy(pool: &SqlitePool, state: &CampaignState) -> String {
    let json = legacy_json(&state.encode_json().expect("state encodes"));
    sqlx::query(
        "INSERT INTO campaign_state_current (campaign_id, schema_version, applied_event_sequence, state_json) VALUES (?, 1, 0, ?)",
    )
    .bind(state.campaign_id().0.to_string())
    .bind(&json)
    .execute(pool)
    .await
    .expect("Gate 1 row inserts with original snapshot");
    json
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
        let duration: WorldDuration =
            serde_json::from_str(&event.payload.json).map_err(|error| {
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

async fn legacy_export() -> CampaignExport {
    let pool = pool(false).await;
    let initial = state();
    create_campaign(&pool, &initial)
        .await
        .expect("campaign creates");
    let mut next = initial.clone();
    next.clock.now = WorldInstant(101);
    next.applied_event_sequence = 1;
    let command = CommandMeta {
        id: CommandId::new(),
        campaign_id: initial.campaign_id(),
        session_id: None,
        issuer: CommandIssuer::System,
        actor: None,
        expected_event_sequence: 0,
    };
    let pending = PendingEvent {
        id: EventId::new(),
        occurred_at: next.clock.now,
        source: EventSource::WorldSimulation,
        actor: None,
        caused_by_event_ids: vec![],
        payload: WorldDuration(1),
    }
    .encode("test.time_advanced", 1)
    .expect("event encodes");
    commit_campaign_transition(
        &pool,
        &command,
        &SerializedRecord::encode("test.advance_time", 1, &WorldDuration(1))
            .expect("command encodes"),
        &next,
        &[pending],
        "Compatibility fixture time advance.",
    )
    .await
    .expect("transition commits");
    let mut export = export_campaign(&pool, initial.campaign_id())
        .await
        .expect("campaign exports");
    export.state_schema_version = 1;
    export.lifecycle.state_schema_version = 1;
    export.current_state.schema_version = 1;
    export.current_state.state_json = legacy_json(&export.current_state.state_json);
    for snapshot in &mut export.snapshots {
        snapshot.state_schema_version = 1;
        snapshot.state_json = legacy_json(&snapshot.state_json);
    }
    export
}

#[tokio::test]
async fn gate_one_database_upgrades_current_rows_and_replays_unchanged_anchor() {
    let pool = pool(true).await;
    let initial = state();
    let legacy = insert_legacy(&pool, &initial).await;
    let snapshot_before: (String, String) = sqlx::query_as(
        "SELECT state_json, created_at_utc FROM campaign_snapshots WHERE campaign_id = ?",
    )
    .bind(initial.campaign_id().0.to_string())
    .fetch_one(&pool)
    .await
    .expect("legacy snapshot");

    migrate_sqlite(&pool)
        .await
        .expect("schema upgrade succeeds");

    let opened = open_campaign(&pool, initial.campaign_id())
        .await
        .expect("upgraded campaign opens");
    assert_eq!(opened.state, initial);
    assert_eq!(
        opened.lifecycle.state_schema_version,
        CURRENT_STATE_SCHEMA_VERSION
    );
    let summary = load_campaign_projection_summary(&pool, initial.campaign_id())
        .await
        .expect("projection remains queryable");
    assert_eq!(summary.state_schema_version, CURRENT_STATE_SCHEMA_VERSION);
    let row = sqlx::query(
        "SELECT schema_version, state_json FROM campaign_state_current WHERE campaign_id = ?",
    )
    .bind(initial.campaign_id().0.to_string())
    .fetch_one(&pool)
    .await
    .expect("current row");
    assert_eq!(row.get::<i64, _>("schema_version"), 2);
    let current: Value =
        serde_json::from_str(&row.get::<String, _>("state_json")).expect("current JSON");
    let mut expected: Value = serde_json::from_str(&legacy).expect("legacy JSON");
    expected["schema_version"] = json!(2);
    expected["rules"] = Value::Null;
    assert_eq!(current, expected);
    let snapshot_after: (String, String) = sqlx::query_as(
        "SELECT state_json, created_at_utc FROM campaign_snapshots WHERE campaign_id = ?",
    )
    .bind(initial.campaign_id().0.to_string())
    .fetch_one(&pool)
    .await
    .expect("unchanged snapshot");
    assert_eq!(snapshot_before, snapshot_after);
    assert_eq!(
        replay_campaign_to_head(&pool, initial.campaign_id(), &ClockApplier)
            .await
            .expect("legacy anchor replays"),
        initial
    );
    let exported = export_campaign(&pool, initial.campaign_id())
        .await
        .expect("mixed-version export validates");
    assert_eq!(exported.snapshots[0].state_schema_version, 1);
    assert_eq!(exported.snapshots[0].state_json, legacy);
    assert!(
        sqlx::query("UPDATE campaign_snapshots SET state_json = '{}' WHERE campaign_id = ?")
            .bind(initial.campaign_id().0.to_string())
            .execute(&pool)
            .await
            .is_err()
    );
}

#[tokio::test]
async fn legacy_export_upgrade_preserves_history_and_restores_replayable_campaign() {
    let legacy = legacy_export().await;
    let original = legacy.clone();
    let upgraded = legacy.upgraded().expect("legacy export upgrades");
    assert_eq!(legacy, original, "source export is immutable");
    assert_eq!(upgraded.state_schema_version, CURRENT_STATE_SCHEMA_VERSION);
    assert_eq!(
        upgraded.current_state.schema_version,
        i64::from(CURRENT_STATE_SCHEMA_VERSION)
    );
    assert_eq!(
        upgraded.lifecycle.state_schema_version,
        CURRENT_STATE_SCHEMA_VERSION
    );
    let json: Value =
        serde_json::from_str(&upgraded.current_state.state_json).expect("upgraded JSON");
    assert!(
        json.as_object()
            .expect("state object")
            .contains_key("rules")
    );
    assert!(json["rules"].is_null());
    assert_eq!(upgraded.snapshots, original.snapshots);
    assert_eq!(upgraded.command_audit, original.command_audit);
    assert_eq!(upgraded.event_journal, original.event_journal);
    assert_eq!(upgraded.event_causes, original.event_causes);

    let pool = pool(false).await;
    let restored = restore_campaign(&pool, &legacy)
        .await
        .expect("legacy export restores");
    let reopened = open_campaign(&pool, restored.state.campaign_id())
        .await
        .expect("restored campaign reopens");
    assert_eq!(restored, reopened);
    let replayed = replay_campaign_to_head(&pool, restored.state.campaign_id(), &ClockApplier)
        .await
        .expect("snapshot and journal replay");
    assert_eq!(replayed, reopened.state);
    let new_export = export_campaign(&pool, restored.state.campaign_id())
        .await
        .expect("restored campaign exports");
    assert_eq!(new_export.snapshots, legacy.snapshots);
    assert_eq!(new_export.command_audit, legacy.command_audit);
    assert_eq!(new_export.event_journal, legacy.event_journal);
}

#[tokio::test]
async fn corrupt_legacy_database_migration_rolls_back_every_campaign() {
    for corruption in [
        "malformed",
        "schema",
        "identity",
        "sequence",
        "rules",
        "lifecycle",
        "duplicate",
        "numeric_schema",
    ] {
        let pool = pool(true).await;
        let valid = state();
        let valid_json = insert_legacy(&pool, &valid).await;
        let invalid = state();
        let invalid_json = insert_legacy(&pool, &invalid).await;
        let mut value: Value = serde_json::from_str(&invalid_json).expect("fixture JSON");
        match corruption {
            "schema" => value["schema_version"] = json!(2),
            "identity" => value["campaign"]["id"] = json!(CampaignId::new()),
            "sequence" => value["applied_event_sequence"] = json!(1),
            "rules" => value["rules"] = json!({ "unrecognized": "must not be erased" }),
            "numeric_schema" => value["schema_version"] = json!(1.0),
            _ => {}
        }
        let corrupted_json = if corruption == "malformed" {
            "{".into()
        } else if corruption == "duplicate" {
            value.to_string().replacen('{', "{\"schema_version\":1,", 1)
        } else {
            value.to_string()
        };
        sqlx::query("UPDATE campaign_state_current SET state_json = ? WHERE campaign_id = ?")
            .bind(&corrupted_json)
            .bind(invalid.campaign_id().0.to_string())
            .execute(&pool)
            .await
            .expect("corruption fixture installs");
        if corruption == "lifecycle" {
            sqlx::query(
                "UPDATE campaign_lifecycle SET state_schema_version = 2 WHERE campaign_id = ?",
            )
            .bind(invalid.campaign_id().0.to_string())
            .execute(&pool)
            .await
            .expect("lifecycle mismatch installs");
        }
        assert!(
            migrate_sqlite(&pool).await.is_err(),
            "{corruption} must block upgrade"
        );
        let actual: (i64, String) = sqlx::query_as(
            "SELECT schema_version, state_json FROM campaign_state_current WHERE campaign_id = ?",
        )
        .bind(valid.campaign_id().0.to_string())
        .fetch_one(&pool)
        .await
        .expect("valid row survives");
        assert_eq!(
            actual,
            (1, valid_json),
            "{corruption} must not partly upgrade another campaign"
        );
        let actual_bad: (i64, String) = sqlx::query_as(
            "SELECT schema_version, state_json FROM campaign_state_current WHERE campaign_id = ?",
        )
        .bind(invalid.campaign_id().0.to_string())
        .fetch_one(&pool)
        .await
        .expect("invalid row survives");
        assert_eq!(
            actual_bad,
            (1, corrupted_json),
            "{corruption} must not be repaired implicitly"
        );
    }
}

#[tokio::test]
async fn invalid_legacy_exports_fail_before_restore_writes() {
    let export = legacy_export().await;
    for corruption in [
        "schema",
        "embedded",
        "rules",
        "future_snapshot",
        "journal",
        "snapshot_identity",
    ] {
        let mut invalid = export.clone();
        let mut current: Value =
            serde_json::from_str(&invalid.current_state.state_json).expect("current JSON");
        match corruption {
            "schema" => invalid.lifecycle.state_schema_version = 2,
            "embedded" => current["schema_version"] = json!(2),
            "rules" => current["rules"] = json!({"unrecognized": true}),
            "future_snapshot" => invalid.snapshots[0].state_schema_version = 2,
            "journal" => invalid.event_journal.clear(),
            "snapshot_identity" => {
                let mut snapshot: Value =
                    serde_json::from_str(&invalid.snapshots[0].state_json).expect("snapshot JSON");
                snapshot["campaign"]["id"] = json!(CampaignId::new());
                invalid.snapshots[0].state_json = snapshot.to_string();
            }
            _ => unreachable!(),
        }
        invalid.current_state.state_json = current.to_string();
        let pool = pool(false).await;
        assert!(
            restore_campaign(&pool, &invalid).await.is_err(),
            "{corruption} must fail"
        );
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM campaign_state_current")
            .fetch_one(&pool)
            .await
            .expect("row count");
        assert_eq!(count, 0, "{corruption} must not commit any campaign");
    }
}

struct ReplacementMigration;

impl SnapshotMigration for ReplacementMigration {
    fn source_version(&self) -> u32 {
        1
    }
    fn migrate_json(&self, _: &str) -> Result<String, String> {
        Err("must never replace built-in migration".into())
    }
}

#[test]
fn built_in_snapshot_upgrade_is_explicit_and_cannot_be_replaced() {
    let initial = state();
    let legacy = legacy_json(&initial.encode_json().expect("state encodes"));
    let mut codec = CampaignStateSnapshotCodec::new();
    assert!(matches!(
        codec.register(ReplacementMigration),
        Err(SnapshotCodecError::DuplicateMigration { from_version: 1 })
    ));
    assert_eq!(
        codec
            .decode_state(1, &legacy)
            .expect("built-in upgrade retained"),
        initial
    );
    let mut invalid: Value = serde_json::from_str(&legacy).expect("legacy JSON");
    invalid["rules"] = json!({"must_not_erase": true});
    assert!(matches!(
        codec.decode_state(1, &invalid.to_string()),
        Err(SnapshotCodecError::MigrationFailed {
            from_version: 1,
            ..
        })
    ));
    assert!(matches!(
        codec.decode_state(1, &initial.encode_json().expect("current JSON")),
        Err(SnapshotCodecError::MigrationFailed {
            from_version: 1,
            ..
        })
    ));
    let duplicate = legacy.replacen('{', "{\"schema_version\":1,", 1);
    assert!(matches!(
        codec.decode_state(1, &duplicate),
        Err(SnapshotCodecError::MigrationFailed {
            from_version: 1,
            ..
        })
    ));
}
