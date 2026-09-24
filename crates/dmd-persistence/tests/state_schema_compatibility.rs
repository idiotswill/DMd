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
    object.remove("table");
    object.remove("encounter");
    serde_json::to_string(&value).expect("legacy JSON")
}

fn schema_three_json(current: &str) -> String {
    let mut value: Value = serde_json::from_str(current).expect("state JSON");
    let object = value.as_object_mut().expect("state object");
    object.insert("schema_version".into(), json!(3));
    object.remove("encounter");
    serde_json::to_string_pretty(&value).expect("schema-3 JSON")
}

fn table_state() -> CampaignState {
    let mut initial = state();
    let contract = dmd_domain::TableContract::default();
    initial.campaign.ruleset = contract.ruleset.clone();
    let mut table = dmd_domain::TableState::new(contract);
    table.situation.title = "An unfinished journey".into();
    table.situation.description = "The table's established situation survives the upgrade.".into();
    initial.table = Some(table);
    initial
}

fn encounter_state() -> CampaignState {
    use dmd_domain::*;
    let mut initial = state();
    let campaign_id = initial.campaign_id();
    let location_id = LocationId::new();
    let scene_id = SceneId::new();
    let actor = EntityId::new();
    initial.locations.insert(
        location_id,
        Location {
            id: location_id,
            campaign_id,
            display_name: "Encounter location".into(),
            parent_location_id: None,
        },
    );
    initial.entities.insert(
        actor,
        WorldEntity {
            id: actor,
            campaign_id,
            display_name: "Encounter participant".into(),
            kind: EntityKind::Creature,
            existence: EntityExistence::Present,
            location_id: Some(location_id),
        },
    );
    initial.scenes.insert(
        scene_id,
        Scene {
            id: scene_id,
            campaign_id,
            location_id,
            mode: SceneMode::Combat,
            status: SceneStatus::Active,
            started_at: initial.clock.now,
            presences: vec![ScenePresence {
                entity_id: actor,
                role: PresenceRole::Participant,
            }],
        },
    );
    initial.rules = Some(RulesState {
        pack_id: initial.campaign.ruleset.id.clone(),
        pack_version: initial.campaign.ruleset.version.clone(),
        entities: [(actor, MechanicalEntity::basic(actor))]
            .into_iter()
            .collect(),
        house_rules: HouseRules::default(),
        effects: vec![],
        tactical_effects: None,
        pending: None,
        rolls: vec![],
        cancelled_roll_ids: vec![],
        rulings: vec![],
        timing: Some(CombatTiming {
            order: vec![InitiativeEntry {
                actor,
                total: 15,
                tie_break: 1,
            }],
            index: 0,
            round: 2,
            turn_number: 2,
            action_spent: true,
            bonus_action_spent: false,
            slot_spent_this_turn: false,
            reactions_spent: vec![actor],
        }),
        rests: vec![],
        completed_short_rests: vec![],
        permission: None,
    });
    let origin = CommandMeta {
        id: CommandId::new(),
        campaign_id,
        session_id: None,
        issuer: CommandIssuer::Admin,
        actor: None,
        expected_event_sequence: 0,
    };
    let position = SpatialPoint { x: 10, y: 10, z: 0 };
    initial.encounter = Some(TacticalEncounter {
        flow: None,
        id: EncounterId::new(),
        scene_id,
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
            ambient_light: LightLevel::Dim,
            terrain: vec![],
            obstacles: vec![],
            lights: vec![],
        },
        participants: vec![TacticalParticipant {
            entity_id: actor,
            public_label: "A traveler".into(),
            position,
            size: CreatureSize::Medium,
            height: 12,
            reach: 10,
            movement: MovementProfile {
                walk: 60,
                climb: None,
                swim: None,
                fly: None,
                burrow: None,
                hover: false,
            },
            senses: Senses {
                darkvision: 120,
                ..Senses::default()
            },
            allies: vec![],
            enemies: vec![],
        }],
        knowledge: vec![ActorKnowledge {
            observer: actor,
            contacts: vec![],
            terrain: vec![RememberedCell {
                position,
                difficult: false,
                blocked: false,
                origin: origin.clone(),
            }],
        }],
        origin,
        geometry_ruling: Ruling {
            basis: RulingBasis::GmAdjudication,
            reason: "The host established this room.".into(),
        },
    });
    assert!(initial.validate().is_empty());
    initial
}

#[tokio::test]
async fn atomic_migration_and_open_futures_remain_send_for_native_commands() {
    let db = tokio::spawn(dmd_persistence::open_sqlite("sqlite::memory:"))
        .await
        .unwrap()
        .unwrap();
    tokio::spawn(async move {
        dmd_persistence::migrate_sqlite(&db).await.unwrap();
        db.close().await;
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn typed_encounter_round_trip_preserves_geometry_knowledge_and_existing_timing() {
    let source = pool(false).await;
    let initial = encounter_state();
    create_campaign(&source, &initial)
        .await
        .expect("valid encounter creates");
    let export = export_campaign(&source, initial.campaign_id())
        .await
        .expect("export");
    let original_json = export.to_json().expect("portable JSON");
    let destination = pool(false).await;
    let decoded = CampaignExport::from_json(&original_json).expect("decode");
    restore_campaign(&destination, &decoded)
        .await
        .expect("restore encounter");
    assert_eq!(
        open_campaign(&destination, initial.campaign_id())
            .await
            .expect("open")
            .state,
        initial
    );
    assert_eq!(
        replay_campaign_to_head(&destination, initial.campaign_id(), &ClockApplier)
            .await
            .expect("replay anchor"),
        initial
    );
    let after = export_campaign(&destination, initial.campaign_id())
        .await
        .expect("export again");
    assert_eq!(after.current_state, export.current_state);
    assert_eq!(after.snapshots, export.snapshots);
    for corruption in [
        "unknown_pending",
        "missing_scene",
        "foreign_origin",
        "future_origin",
        "duplicate_participant",
    ] {
        for owner in ["current", "anchor"] {
            let mut corrupt = export.clone();
            let json = if owner == "current" {
                &mut corrupt.current_state.state_json
            } else {
                &mut corrupt.snapshots[0].state_json
            };
            let mut value: Value = serde_json::from_str(json).expect("state JSON");
            match corruption {
                "unknown_pending" => value["encounter"]["pending"] = json!({"raw_after_state": {}}),
                "missing_scene" => {
                    value["encounter"]["scene_id"] = json!(dmd_domain::SceneId::new())
                }
                "foreign_origin" => {
                    value["encounter"]["origin"]["campaign_id"] = json!(CampaignId::new())
                }
                "future_origin" => {
                    value["encounter"]["origin"]["expected_event_sequence"] = json!(1)
                }
                "duplicate_participant" => {
                    let duplicate = value["encounter"]["participants"][0].clone();
                    value["encounter"]["participants"]
                        .as_array_mut()
                        .expect("participants")
                        .push(duplicate);
                }
                _ => unreachable!(),
            }
            *json = value.to_string();
            let empty = pool(false).await;
            assert!(
                restore_campaign(&empty, &corrupt).await.is_err(),
                "{owner}/{corruption} must fail closed"
            );
            assert!(
                dmd_persistence::list_campaigns(&empty)
                    .await
                    .expect("list")
                    .is_empty()
            );
        }
    }
}

async fn gate_three_pool() -> SqlitePool {
    let options = SqliteConnectOptions::from_str("sqlite::memory:")
        .expect("valid SQLite URL")
        .foreign_keys(true);
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await
        .expect("database opens");
    Migrator {
        migrations: Cow::Owned(
            MIGRATOR
                .iter()
                .filter(|migration| migration.version < 10)
                .cloned()
                .collect(),
        ),
        ..Migrator::DEFAULT
    }
    .run(&pool)
    .await
    .expect("Gate 3 migrations apply");
    pool
}

async fn insert_gate_three(pool: &SqlitePool, initial: &CampaignState) -> String {
    let json = schema_three_json(&initial.encode_json().expect("state encodes"));
    sqlx::query(
        "INSERT INTO campaign_state_current (campaign_id, schema_version, applied_event_sequence, state_json) VALUES (?, 3, 0, ?)",
    )
    .bind(initial.campaign_id().0.to_string())
    .bind(&json)
    .execute(pool)
    .await
    .expect("Gate 3 row and anchor insert");
    json
}

#[tokio::test]
async fn schema_three_migration_preserves_table_and_exact_anchor_metadata() {
    let pool = gate_three_pool().await;
    let initial = table_state();
    let original_json = insert_gate_three(&pool, &initial).await;
    let anchor_before: (i64, String, String) = sqlx::query_as(
        "SELECT state_schema_version, state_json, created_at_utc FROM campaign_snapshots WHERE campaign_id = ?",
    )
    .bind(initial.campaign_id().0.to_string())
    .fetch_one(&pool).await.expect("original anchor");

    migrate_sqlite(&pool).await.expect("schema 4 migration");

    let opened = open_campaign(&pool, initial.campaign_id())
        .await
        .expect("open");
    assert_eq!(
        opened.state, initial,
        "only the optional encounter field and schema change"
    );
    assert_eq!(
        opened.lifecycle.state_schema_version,
        CURRENT_STATE_SCHEMA_VERSION
    );
    let summary = load_campaign_projection_summary(&pool, initial.campaign_id())
        .await
        .expect("projection");
    assert_eq!(summary.state_schema_version, CURRENT_STATE_SCHEMA_VERSION);
    let anchor_after: (i64, String, String) = sqlx::query_as(
        "SELECT state_schema_version, state_json, created_at_utc FROM campaign_snapshots WHERE campaign_id = ?",
    )
    .bind(initial.campaign_id().0.to_string())
    .fetch_one(&pool).await.expect("unchanged anchor");
    assert_eq!(anchor_before, anchor_after);
    assert_eq!(anchor_after.0, 3);
    assert_eq!(anchor_after.1, original_json);
    assert_eq!(
        replay_campaign_to_head(&pool, initial.campaign_id(), &ClockApplier)
            .await
            .expect("replay"),
        initial
    );
    let exported = export_campaign(&pool, initial.campaign_id())
        .await
        .expect("export");
    assert_eq!(exported.format_version, 2);
    assert_eq!(exported.snapshots[0].state_schema_version, 3);
    assert_eq!(exported.snapshots[0].state_json, original_json);
    let restored = self::pool(false).await;
    restore_campaign(&restored, &exported)
        .await
        .expect("mixed-version restore");
    assert_eq!(
        export_campaign(&restored, initial.campaign_id())
            .await
            .expect("restored export")
            .snapshots,
        exported.snapshots
    );
}

#[tokio::test]
async fn schema_three_export_upgrades_only_current_image_and_keeps_accepted_history() {
    let mut legacy = legacy_export().await;
    // The immutable generic time event is valid under either state schema. Keep its exact bytes.
    legacy.format_version = 2;
    legacy.state_schema_version = 3;
    legacy.lifecycle.state_schema_version = 3;
    legacy.current_state.schema_version = 3;
    let upgraded_current = CampaignStateSnapshotCodec::new()
        .decode_state(1, &legacy.current_state.state_json)
        .expect("legacy current decodes");
    legacy.current_state.state_json =
        schema_three_json(&upgraded_current.encode_json().expect("JSON"));
    for snapshot in &mut legacy.snapshots {
        let state = CampaignStateSnapshotCodec::new()
            .decode_state(1, &snapshot.state_json)
            .expect("anchor decodes");
        snapshot.state_schema_version = 3;
        snapshot.state_json = schema_three_json(&state.encode_json().expect("JSON"));
    }
    let original = legacy.clone();
    let upgraded = legacy.upgraded().expect("format 2 schema 3 upgrades");
    assert_eq!(legacy, original);
    assert_eq!(upgraded.state_schema_version, CURRENT_STATE_SCHEMA_VERSION);
    assert_eq!(upgraded.snapshots, original.snapshots);
    assert_eq!(upgraded.command_audit, original.command_audit);
    assert_eq!(upgraded.event_journal, original.event_journal);
    assert_eq!(upgraded.event_causes, original.event_causes);
    assert_eq!(upgraded.play_sessions, original.play_sessions);
    assert_eq!(
        upgraded.play_session_participants,
        original.play_session_participants
    );
    assert_eq!(upgraded.observations, original.observations);
    let destination = pool(false).await;
    restore_campaign(&destination, &legacy)
        .await
        .expect("restore original schema 3 export");
    let campaign_id = upgraded_current.campaign_id();
    assert_eq!(
        replay_campaign_to_head(&destination, campaign_id, &ClockApplier)
            .await
            .expect("replay"),
        upgraded_current
    );
    assert_eq!(
        export_campaign(&destination, campaign_id)
            .await
            .expect("export")
            .event_journal,
        original.event_journal
    );
}

#[tokio::test]
async fn corrupt_schema_three_upgrade_rolls_back_all_current_images() {
    for corruption in [
        "malformed",
        "schema",
        "numeric_schema",
        "identity",
        "sequence",
        "encounter",
        "duplicate",
        "duplicate_encounter",
        "tactical_effects",
        "lifecycle",
    ] {
        let pool = gate_three_pool().await;
        let valid = table_state();
        let valid_json = insert_gate_three(&pool, &valid).await;
        let invalid = table_state();
        let invalid_json = insert_gate_three(&pool, &invalid).await;
        let mut value: Value = serde_json::from_str(&invalid_json).expect("JSON");
        match corruption {
            "schema" => value["schema_version"] = json!(2),
            "numeric_schema" => value["schema_version"] = json!(3.0),
            "identity" => value["campaign"]["id"] = json!(CampaignId::new()),
            "sequence" => value["applied_event_sequence"] = json!(1),
            "encounter" => {
                value["encounter"] = json!({"unknown_pending_action": "must not be erased"})
            }
            "tactical_effects" => {
                value["rules"] = serde_json::to_value(encounter_state().rules.unwrap()).unwrap();
                value["rules"]["tactical_effects"] = json!(dmd_domain::TacticalEffects::default());
            }
            _ => (),
        }
        let corrupted = match corruption {
            "malformed" => "{".to_owned(),
            "duplicate" => value.to_string().replacen('{', "{\"schema_version\":3,", 1),
            "duplicate_encounter" => {
                value
                    .to_string()
                    .replacen('{', "{\"encounter\":null,\"encounter\":null,", 1)
            }
            _ => value.to_string(),
        };
        sqlx::query("UPDATE campaign_state_current SET state_json = ? WHERE campaign_id = ?")
            .bind(&corrupted)
            .bind(invalid.campaign_id().0.to_string())
            .execute(&pool)
            .await
            .expect("corrupt fixture");
        if corruption == "lifecycle" {
            sqlx::query(
                "UPDATE campaign_lifecycle SET state_schema_version = 2 WHERE campaign_id = ?",
            )
            .bind(invalid.campaign_id().0.to_string())
            .execute(&pool)
            .await
            .expect("lifecycle fixture");
        }
        assert!(
            migrate_sqlite(&pool).await.is_err(),
            "{corruption} must block migration"
        );
        for (id, expected_json) in [
            (valid.campaign_id(), valid_json),
            (invalid.campaign_id(), corrupted),
        ] {
            let actual: (i64, String) = sqlx::query_as("SELECT schema_version, state_json FROM campaign_state_current WHERE campaign_id = ?")
                .bind(id.0.to_string()).fetch_one(&pool).await.expect("surviving current image");
            assert_eq!(
                actual,
                (3, expected_json),
                "{corruption}: no partial upgrade or repair"
            );
        }
        let unchanged_schema: i64 = sqlx::query_scalar(
            "SELECT state_schema_version FROM campaign_lifecycle WHERE campaign_id = ?",
        )
        .bind(valid.campaign_id().0.to_string())
        .fetch_one(&pool)
        .await
        .expect("lifecycle");
        assert_eq!(unchanged_schema, 3);
        let migrated: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM _sqlx_migrations WHERE version = 10")
                .fetch_one(&pool)
                .await
                .expect("migration history");
        assert_eq!(migrated, 0);
    }
}

#[tokio::test]
async fn late_encounter_preflight_failure_rolls_back_the_entire_older_migration_chain() {
    for stored_schema in [1, 2] {
        let pool = pool(true).await;
        if stored_schema == 2 {
            Migrator {
                migrations: Cow::Owned(
                    MIGRATOR
                        .iter()
                        .filter(|migration| migration.version < 9)
                        .cloned()
                        .collect(),
                ),
                ..Migrator::DEFAULT
            }
            .run(&pool)
            .await
            .expect("Gate 2 migrations apply");
        }
        let valid = state();
        let invalid = state();
        let mut originals = Vec::new();
        for (initial, corrupted) in [(&valid, false), (&invalid, true)] {
            let mut value: Value =
                serde_json::from_str(&legacy_json(&initial.encode_json().expect("JSON")))
                    .expect("value");
            value["schema_version"] = json!(stored_schema);
            if stored_schema == 2 {
                value["rules"] = Value::Null;
            }
            if corrupted {
                value["encounter"] = json!({"pending": "future authority must survive rejection"});
            }
            let json = value.to_string();
            sqlx::query("INSERT INTO campaign_state_current (campaign_id, schema_version, applied_event_sequence, state_json) VALUES (?, ?, 0, ?)")
                .bind(initial.campaign_id().0.to_string()).bind(stored_schema).bind(&json).execute(&pool).await.expect("older fixture");
            originals.push((initial.campaign_id(), json));
        }
        let anchors_before: Vec<(String, i64, String, String)> = sqlx::query_as(
            "SELECT campaign_id, state_schema_version, state_json, created_at_utc FROM campaign_snapshots ORDER BY campaign_id",
        ).fetch_all(&pool).await.expect("anchors");
        let migrations_before: Vec<i64> =
            sqlx::query_scalar("SELECT version FROM _sqlx_migrations ORDER BY version")
                .fetch_all(&pool)
                .await
                .expect("migration versions");
        assert!(matches!(
            migrate_sqlite(&pool).await,
            Err(sqlx::migrate::MigrateError::ExecuteMigration(_, 10))
        ));
        for (id, original) in originals {
            let actual: (i64, String) = sqlx::query_as("SELECT schema_version, state_json FROM campaign_state_current WHERE campaign_id = ?")
                .bind(id.0.to_string()).fetch_one(&pool).await.expect("unchanged current");
            assert_eq!(actual, (i64::from(stored_schema), original));
            let lifecycle: i64 = sqlx::query_scalar(
                "SELECT state_schema_version FROM campaign_lifecycle WHERE campaign_id = ?",
            )
            .bind(id.0.to_string())
            .fetch_one(&pool)
            .await
            .expect("lifecycle");
            assert_eq!(lifecycle, i64::from(stored_schema));
        }
        let anchors_after: Vec<(String, i64, String, String)> = sqlx::query_as(
            "SELECT campaign_id, state_schema_version, state_json, created_at_utc FROM campaign_snapshots ORDER BY campaign_id",
        ).fetch_all(&pool).await.expect("anchors");
        assert_eq!(anchors_before, anchors_after);
        let migrations_after: Vec<i64> =
            sqlx::query_scalar("SELECT version FROM _sqlx_migrations ORDER BY version")
                .fetch_all(&pool)
                .await
                .expect("migration versions");
        assert_eq!(
            migrations_before, migrations_after,
            "no earlier pending migration can commit alone"
        );
        let observation_tables: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'session_observations'")
            .fetch_one(&pool).await.expect("schema inventory");
        assert_eq!(
            observation_tables, 0,
            "migration 9's new table also rolls back"
        );
    }
}

#[test]
fn every_legacy_snapshot_path_rejects_future_encounter_authority() {
    let codec = CampaignStateSnapshotCodec::new();
    for version in 1..=3 {
        let mut value: Value =
            serde_json::from_str(&state().encode_json().expect("JSON")).expect("value");
        value["schema_version"] = json!(version);
        assert!(codec.decode_state(version, &value.to_string()).is_ok());
        for encounter in [
            json!({"pending": {"raw": "invented authority"}}),
            json!([]),
            json!(true),
            json!(7),
            json!("unsupported"),
        ] {
            value["encounter"] = encounter;
            assert!(
                codec.decode_state(version, &value.to_string()).is_err(),
                "schema {version} must reject future encounter data"
            );
        }
        value["encounter"] = Value::Null;
        let duplicate = value.to_string().replacen('{', "{\"encounter\":null,", 1);
        assert!(codec.decode_state(version, &duplicate).is_err());
    }
    let mut malformed_current: Value =
        serde_json::from_str(&state().encode_json().expect("JSON")).expect("value");
    malformed_current["encounter"] = json!({"pending": {"raw": "invented authority"}});
    assert!(
        codec
            .decode_state(CURRENT_STATE_SCHEMA_VERSION, &malformed_current.to_string())
            .is_err()
    );
    // Even a well-formed, otherwise valid encounter cannot be smuggled through a legacy version.
    let current = encounter_state();
    for version in 1..=3 {
        let mut legacy = current.clone();
        legacy.schema_version = version;
        assert!(matches!(
            codec.decode_state(version, &legacy.encode_json().expect("JSON")),
            Err(SnapshotCodecError::MigrationFailed { message, .. }) if message.contains("tactical encounter")
        ));
    }
}

#[test]
fn old_saves_reject_even_empty_well_formed_tactical_effect_authority() {
    let codec = CampaignStateSnapshotCodec::new();
    let mut current = encounter_state();
    current.encounter = None;
    current.rules.as_mut().unwrap().timing = None;
    current.rules.as_mut().unwrap().tactical_effects = Some(dmd_domain::TacticalEffects::default());
    assert!(
        codec
            .decode_state(4, &current.encode_json().unwrap())
            .is_ok()
    );
    for version in 1..=3 {
        current.schema_version = version;
        assert!(matches!(
            codec.decode_state(version, &current.encode_json().unwrap()),
            Err(SnapshotCodecError::MigrationFailed { .. })
        ));
    }
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
    export.format_version = 1;
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
    assert_eq!(
        row.get::<i64, _>("schema_version"),
        i64::from(CURRENT_STATE_SCHEMA_VERSION)
    );
    let current: Value =
        serde_json::from_str(&row.get::<String, _>("state_json")).expect("current JSON");
    let mut expected: Value = serde_json::from_str(&legacy).expect("legacy JSON");
    expected["schema_version"] = json!(CURRENT_STATE_SCHEMA_VERSION);
    expected["rules"] = Value::Null;
    expected["table"] = Value::Null;
    expected["encounter"] = Value::Null;
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
