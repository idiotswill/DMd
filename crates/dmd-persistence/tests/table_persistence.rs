use std::{borrow::Cow, str::FromStr};

use dmd_domain::*;
use dmd_persistence::*;
use serde_json::{Value, json};
use sqlx::{
    SqlitePool,
    migrate::Migrator,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};

async fn pool() -> SqlitePool {
    let pool = bare_pool().await;
    migrate_sqlite(&pool).await.expect("migrate");
    pool
}

async fn bare_pool() -> SqlitePool {
    SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            SqliteConnectOptions::from_str("sqlite::memory:")
                .expect("URL")
                .foreign_keys(true),
        )
        .await
        .expect("open")
}

fn state() -> CampaignState {
    let mut state = CampaignState::empty(
        Campaign {
            id: CampaignId::new(),
            display_name: "Durable table".into(),
            status: CampaignStatus::Active,
            world_seed: 517,
            ruleset: TableContract::default().ruleset,
            content_packs: vec![],
        },
        WorldClock {
            now: WorldInstant(100),
            calendar_id: "test".into(),
        },
    );
    for name in ["One", "Two"] {
        let player = Player {
            id: PlayerId::new(),
            campaign_id: state.campaign_id(),
            display_name: name.into(),
        };
        state.players.insert(player.id, player);
    }
    state
}

fn session(state: &CampaignState) -> PlaySession {
    let mut ids: Vec<_> = state.players.keys().copied().collect();
    ids.sort_by_key(|id| id.0);
    PlaySession {
        id: PlaySessionId::new(),
        campaign_id: state.campaign_id(),
        display_name: "First session".into(),
        status: PlaySessionStatus::Active,
        started_at_world: state.clock.now,
        ended_at_world: None,
        participants: ids
            .into_iter()
            .map(|player_id| SessionParticipant {
                player_id,
                character_id: None,
                attendance: AttendanceStatus::Present,
            })
            .collect(),
    }
}

fn observation(state: &CampaignState, session: &PlaySession) -> NewSessionObservation {
    NewSessionObservation {
        id: ObservationId::new(),
        campaign_id: state.campaign_id(),
        session_id: Some(session.id),
        issuer: CommandIssuer::Player(session.participants[0].player_id),
        audience: ObservationAudience::Party,
        observed_event_sequence: state.applied_event_sequence,
        kind: "table.query".into(),
        payload_schema_version: 1,
        payload_json: json!({"text":"What can I see?"}).to_string(),
    }
}

async fn commit(
    pool: &SqlitePool,
    before: &CampaignState,
    next: &CampaignState,
    change: &SessionChange,
    issuer: CommandIssuer,
    session_id: PlaySessionId,
) -> Result<CommitReceipt, JournalStoreError> {
    commit_campaign_transition_with_session(
        pool,
        &CommandMeta {
            id: CommandId::new(),
            campaign_id: before.campaign_id(),
            session_id: Some(session_id),
            issuer,
            actor: None,
            expected_event_sequence: before.applied_event_sequence,
        },
        &SerializedRecord::encode("test.session", 1, &()).expect("payload"),
        next,
        &[PendingEvent {
            id: EventId::new(),
            occurred_at: next.clock.now,
            source: EventSource::AdminCorrection,
            actor: None,
            caused_by_event_ids: vec![],
            payload: (),
        }
        .encode("test.session_changed", 1)
        .expect("event")],
        "Session lifecycle transition",
        Some(change),
    )
    .await
}

#[tokio::test]
async fn session_start_end_are_atomic_and_closed_history_and_cas_are_enforced() {
    let pool = pool().await;
    let initial = state();
    create_campaign(&pool, &initial).await.expect("create");
    let active = session(&initial);
    let change = SessionChange::Start {
        session: active.clone(),
    };
    let mut next = initial.clone();
    next.applied_event_sequence = 1;
    assert!(matches!(
        commit(
            &pool,
            &initial,
            &next,
            &change,
            CommandIssuer::Admin,
            PlaySessionId::new()
        )
        .await,
        Err(JournalStoreError::Session(
            SessionStoreError::InvalidSession
        ))
    ));
    next.applied_event_sequence = 2; // Fails after the provisional session write.
    assert!(matches!(
        commit(
            &pool,
            &initial,
            &next,
            &change,
            CommandIssuer::Admin,
            active.id
        )
        .await,
        Err(JournalStoreError::NextStateSequenceMismatch { .. })
    ));
    assert!(
        load_play_session(&pool, active.id)
            .await
            .expect("read")
            .is_none()
    );
    assert_eq!(
        open_campaign(&pool, initial.campaign_id())
            .await
            .expect("open")
            .state,
        initial
    );
    assert!(
        export_campaign(&pool, initial.campaign_id())
            .await
            .expect("export")
            .command_audit
            .is_empty()
    );
    next.applied_event_sequence = 1;
    assert!(matches!(
        commit(
            &pool,
            &initial,
            &next,
            &change,
            CommandIssuer::Player(active.participants[0].player_id),
            active.id
        )
        .await,
        Err(JournalStoreError::Session(SessionStoreError::Unauthorized))
    ));
    commit(
        &pool,
        &initial,
        &next,
        &change,
        CommandIssuer::Admin,
        active.id,
    )
    .await
    .expect("start");
    assert_eq!(
        load_active_play_session(&pool, initial.campaign_id())
            .await
            .expect("read"),
        Some(active.clone())
    );
    let mut final_state = next.clone();
    final_state.applied_event_sequence = 2;
    final_state.clock.now = WorldInstant(101);
    let mut closed = active.clone();
    closed.status = PlaySessionStatus::Closed;
    closed.ended_at_world = Some(final_state.clock.now);
    let mut stale = active.clone();
    stale.display_name = "stale".into();
    assert!(matches!(
        commit(
            &pool,
            &next,
            &final_state,
            &SessionChange::Replace {
                expected: stale,
                next: closed.clone(),
            },
            CommandIssuer::Admin,
            active.id
        )
        .await,
        Err(JournalStoreError::Session(SessionStoreError::Conflict))
    ));
    commit(
        &pool,
        &next,
        &final_state,
        &SessionChange::Replace {
            expected: active.clone(),
            next: closed.clone(),
        },
        CommandIssuer::Admin,
        active.id,
    )
    .await
    .expect("end");
    let mut later = final_state.clone();
    later.applied_event_sequence = 3;
    assert!(matches!(
        commit(
            &pool,
            &final_state,
            &later,
            &SessionChange::Replace {
                expected: closed.clone(),
                next: active.clone(),
            },
            CommandIssuer::Admin,
            active.id
        )
        .await,
        Err(JournalStoreError::Session(
            SessionStoreError::ClosedSessionImmutable
        ))
    ));
    assert!(matches!(
        commit(
            &pool,
            &final_state,
            &later,
            &change,
            CommandIssuer::Admin,
            active.id
        )
        .await,
        Err(JournalStoreError::Session(SessionStoreError::Conflict))
    ));
    assert_eq!(
        load_play_session(&pool, active.id).await.expect("read"),
        Some(closed)
    );
    assert_eq!(
        export_campaign(&pool, initial.campaign_id())
            .await
            .expect("export")
            .event_journal
            .len(),
        2
    );
}

#[tokio::test]
async fn observations_are_immutable_idempotent_and_export_restore_without_world_mutation() {
    let pool = pool().await;
    let initial = state();
    create_campaign(&pool, &initial).await.expect("create");
    let active = session(&initial);
    save_play_session(&pool, &initial, &active)
        .await
        .expect("fixture session");
    let before = export_campaign(&pool, initial.campaign_id())
        .await
        .expect("before");
    let record = observation(&initial, &active);
    let saved = append_session_observations(
        &pool,
        initial.campaign_id(),
        &[record.clone(), record.clone()],
    )
    .await
    .expect("append");
    assert_eq!(saved[0], saved[1]);
    assert_eq!(saved[0].ordinal, 1);
    let mut conflicting = record.clone();
    conflicting.payload_json = "{}".into();
    assert!(matches!(
        append_session_observations(&pool, initial.campaign_id(), &[conflicting]).await,
        Err(ObservationStoreError::ConflictingIdentity)
    ));
    let mut closed = active.clone();
    closed.status = PlaySessionStatus::Closed;
    closed.ended_at_world = Some(initial.clock.now);
    save_play_session(&pool, &initial, &closed)
        .await
        .expect("close fixture");
    assert_eq!(
        append_session_observations(&pool, initial.campaign_id(), &[record])
            .await
            .expect("retry after close"),
        vec![saved[0].clone()]
    );
    assert!(
        append_session_observations(
            &pool,
            initial.campaign_id(),
            &[observation(&initial, &active)]
        )
        .await
        .is_err()
    );
    let after = export_campaign(&pool, initial.campaign_id())
        .await
        .expect("after");
    assert_eq!(after.current_state, before.current_state);
    assert_eq!(after.command_audit, before.command_audit);
    assert_eq!(after.event_journal, before.event_journal);
    assert_eq!(after.snapshots, before.snapshots);
    assert_eq!(after.observations, vec![saved[0].clone()]);
    assert_eq!(
        load_campaign_observations(&pool, initial.campaign_id(), 0, 20)
            .await
            .expect("campaign history"),
        after.observations
    );
    assert_eq!(
        load_session_observation(&pool, initial.campaign_id(), saved[0].record.id)
            .await
            .expect("lookup"),
        Some(saved[0].clone())
    );
    assert!(
        load_session_observation(&pool, CampaignId::new(), saved[0].record.id)
            .await
            .expect("isolated lookup")
            .is_none()
    );
    assert!(
        load_campaign_observations(&pool, initial.campaign_id(), 0, 0)
            .await
            .is_err()
    );
    assert!(
        load_session_observations(&pool, initial.campaign_id(), Some(active.id), u64::MAX, 1)
            .await
            .is_err()
    );
    for sql in [
        "UPDATE session_observations SET record_json = '{}'",
        "DELETE FROM session_observations",
    ] {
        assert!(sqlx::query(sql).execute(&pool).await.is_err());
    }
    let restored_pool = self::pool().await;
    restore_campaign(&restored_pool, &after)
        .await
        .expect("restore");
    for mutation in 0..7 {
        let mut corrupt = after.clone();
        match mutation {
            0 => corrupt.observations[0].ordinal = 2,
            1 => corrupt.observations[0].record.campaign_id = CampaignId::new(),
            2 => corrupt.observations[0].record.session_id = Some(PlaySessionId::new()),
            3 => corrupt.observations[0].record.observed_event_sequence = 1,
            4 => corrupt.observations[0].record.issuer = CommandIssuer::Import,
            5 => {
                corrupt.observations[0].record.audience =
                    ObservationAudience::Player(PlayerId::new())
            }
            6 => {
                let duplicate = corrupt.observations[0].clone();
                corrupt.observations.push(duplicate);
            }
            _ => unreachable!(),
        }
        let empty = self::pool().await;
        assert!(
            restore_campaign(&empty, &corrupt).await.is_err(),
            "observation mutation {mutation}"
        );
        assert!(list_campaigns(&empty).await.expect("empty list").is_empty());
    }
    assert_eq!(
        load_session_observations(
            &restored_pool,
            initial.campaign_id(),
            Some(active.id),
            0,
            20
        )
        .await
        .expect("history"),
        after.observations
    );
    assert!(matches!(
        purge_campaign(
            &pool,
            initial.campaign_id(),
            &CampaignPurgeAuthorization::admin("backup stale"),
            &before
        )
        .await,
        Err(LifecycleError::StalePurgeBackup)
    ));
    purge_campaign(
        &pool,
        initial.campaign_id(),
        &CampaignPurgeAuthorization::admin("verified backup"),
        &after,
    )
    .await
    .expect("aggregate purge");
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM session_observations")
            .fetch_one(&pool)
            .await
            .expect("count"),
        0
    );
}

#[tokio::test]
async fn observation_batch_rejects_invalid_authority_references_and_rolls_back() {
    let pool = pool().await;
    let initial = state();
    create_campaign(&pool, &initial).await.expect("create");
    let mut active = session(&initial);
    active.participants[1].attendance = AttendanceStatus::Absent;
    save_play_session(&pool, &initial, &active)
        .await
        .expect("session");
    for corruption in 0..8 {
        let valid = observation(&initial, &active);
        let mut invalid = observation(&initial, &active);
        match corruption {
            0 => invalid.issuer = CommandIssuer::Player(PlayerId::new()),
            1 => invalid.audience = ObservationAudience::Player(active.participants[1].player_id),
            2 => invalid.session_id = Some(PlaySessionId::new()),
            3 => invalid.observed_event_sequence = 1,
            4 => invalid.campaign_id = CampaignId::new(),
            5 => invalid.issuer = CommandIssuer::Import,
            6 => invalid.payload_json = "{".into(),
            7 => invalid.session_id = None,
            _ => unreachable!(),
        }
        assert!(
            append_session_observations(&pool, initial.campaign_id(), &[valid, invalid])
                .await
                .is_err(),
            "mutation {corruption}"
        );
        assert!(
            export_campaign(&pool, initial.campaign_id())
                .await
                .expect("export")
                .observations
                .is_empty()
        );
    }
}

fn schema_two_json(state: &CampaignState) -> String {
    let mut value: Value =
        serde_json::from_str(&state.encode_json().expect("JSON")).expect("value");
    value["schema_version"] = json!(2);
    value.as_object_mut().expect("object").remove("table");
    value.as_object_mut().expect("object").remove("encounter");
    value.to_string()
}

async fn gate_two_pool() -> SqlitePool {
    let pool = bare_pool().await;
    Migrator {
        migrations: Cow::Owned(MIGRATOR.iter().filter(|m| m.version < 9).cloned().collect()),
        ..Migrator::DEFAULT
    }
    .run(&pool)
    .await
    .expect("Gate 2 migrations");
    pool
}

async fn insert_gate_two(pool: &SqlitePool, initial: &CampaignState, json: &str) {
    sqlx::query("INSERT INTO campaign_state_current (campaign_id, schema_version, applied_event_sequence, state_json) VALUES (?,2,0,?)")
        .bind(initial.campaign_id().0.to_string()).bind(json).execute(pool).await.expect("legacy row");
}

#[tokio::test]
async fn schema_two_database_and_format_one_export_upgrade_preserve_exact_old_anchors() {
    let pool = gate_two_pool().await;
    let mut initial = state();
    initial.rules = Some(RulesState {
        pack_id: "srd-5.2".into(),
        pack_version: "5.2.1".into(),
        entities: Default::default(),
        house_rules: HouseRules::default(),
        effects: vec![],
        tactical_effects: None,
        tactical_inventory: None,
        pending: None,
        rolls: vec![],
        cancelled_roll_ids: vec![],
        rulings: vec![],
        timing: None,
        rests: vec![],
        completed_short_rests: vec![],
        permission: None,
    });
    let old_json = schema_two_json(&initial);
    insert_gate_two(&pool, &initial, &old_json).await;
    migrate_sqlite(&pool).await.expect("upgrade");
    let upgraded = export_campaign(&pool, initial.campaign_id())
        .await
        .expect("export");
    assert_eq!(
        open_campaign(&pool, initial.campaign_id())
            .await
            .expect("open")
            .state,
        initial
    );
    assert_eq!(upgraded.snapshots[0].state_json, old_json);
    assert_eq!(upgraded.snapshots[0].state_schema_version, 2);
    let mut legacy = upgraded.clone();
    legacy.format_version = 1;
    legacy.state_schema_version = 2;
    legacy.lifecycle.state_schema_version = 2;
    legacy.current_state.schema_version = 2;
    legacy.current_state.state_json = old_json;
    let mut json = serde_json::to_value(&legacy).expect("export JSON");
    json.as_object_mut().expect("object").remove("observations");
    let legacy = CampaignExport::from_json(&json.to_string()).expect("format 1 without field");
    let before = legacy.clone();
    let upgraded = legacy.upgraded().expect("explicit upgrade");
    assert_eq!(legacy, before);
    assert_eq!(upgraded.format_version, 2);
    assert_eq!(upgraded.snapshots, legacy.snapshots);
    let destination = self::pool().await;
    restore_campaign(&destination, &legacy)
        .await
        .expect("restore Gate 2");
    assert_eq!(
        open_campaign(&destination, initial.campaign_id())
            .await
            .expect("open")
            .state,
        initial
    );
    let mut smuggled = legacy.clone();
    smuggled.observations.push(SessionObservation {
        ordinal: 1,
        record: observation(&initial, &session(&initial)),
    });
    assert!(smuggled.upgraded().is_err());
}

#[tokio::test]
async fn table_binding_and_session_projection_must_change_together() {
    let pool = pool().await;
    let mut initial = state();
    initial.table = Some(TableState::new(TableContract::default()));
    create_campaign(&pool, &initial)
        .await
        .expect("create table");
    let active = session(&initial);
    let change = SessionChange::Start {
        session: active.clone(),
    };
    let mut next = initial.clone();
    next.applied_event_sequence = 1;
    assert!(matches!(
        commit(
            &pool,
            &initial,
            &next,
            &change,
            CommandIssuer::Admin,
            active.id
        )
        .await,
        Err(JournalStoreError::Session(SessionStoreError::Conflict))
    ));
    assert!(
        load_play_session(&pool, active.id)
            .await
            .expect("rolled back")
            .is_none()
    );
    next.table.as_mut().expect("table").active_session = Some(ActiveTableSession {
        session_id: active.id,
        display_name: active.display_name.clone(),
        started_at_world: active.started_at_world,
        participants: active.participants.clone(),
    });
    commit(
        &pool,
        &initial,
        &next,
        &change,
        CommandIssuer::Admin,
        active.id,
    )
    .await
    .expect("atomic binding");
    assert_eq!(
        open_campaign(&pool, initial.campaign_id())
            .await
            .expect("open")
            .state,
        next
    );
    let export = export_campaign(&pool, initial.campaign_id())
        .await
        .expect("export");
    let mut invalid = export.clone();
    invalid.play_sessions[0].display_name = "unrecorded rewrite".into();
    assert!(invalid.upgraded().is_err());
    let mut drift = active.clone();
    drift.display_name = "raw projection drift".into();
    save_play_session(&pool, &next, &drift)
        .await
        .expect("legacy raw fixture");
    assert!(open_campaign(&pool, initial.campaign_id()).await.is_err());
    assert!(export_campaign(&pool, initial.campaign_id()).await.is_err());
}

#[tokio::test]
async fn schema_three_migration_preflight_rolls_back_all_current_images() {
    for corruption in ["table", "duplicate", "schema", "lifecycle", "malformed"] {
        let pool = gate_two_pool().await;
        let valid = state();
        let valid_json = schema_two_json(&valid);
        insert_gate_two(&pool, &valid, &valid_json).await;
        let invalid = state();
        let old = schema_two_json(&invalid);
        insert_gate_two(&pool, &invalid, &old).await;
        let mut value: Value = serde_json::from_str(&old).expect("value");
        match corruption {
            "table" => value["table"] = json!({"unknown":"must not erase"}),
            "schema" => value["schema_version"] = json!(3),
            _ => (),
        }
        let damaged = match corruption {
            "malformed" => "{".into(),
            "duplicate" => value.to_string().replacen('{', "{\"schema_version\":2,", 1),
            _ => value.to_string(),
        };
        sqlx::query("UPDATE campaign_state_current SET state_json = ? WHERE campaign_id = ?")
            .bind(&damaged)
            .bind(invalid.campaign_id().0.to_string())
            .execute(&pool)
            .await
            .expect("fixture");
        if corruption == "lifecycle" {
            sqlx::query("UPDATE campaign_lifecycle SET state_schema_version=1 WHERE campaign_id=?")
                .bind(invalid.campaign_id().0.to_string())
                .execute(&pool)
                .await
                .expect("fixture");
        }
        assert!(migrate_sqlite(&pool).await.is_err(), "{corruption}");
        let actual: (i64, String) = sqlx::query_as(
            "SELECT schema_version,state_json FROM campaign_state_current WHERE campaign_id=?",
        )
        .bind(valid.campaign_id().0.to_string())
        .fetch_one(&pool)
        .await
        .expect("valid row");
        assert_eq!(actual, (2, valid_json));
    }
}
