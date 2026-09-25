use dmd_domain::*;
use dmd_persistence::*;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::str::FromStr;
use uuid::Uuid;

async fn pool() -> sqlx::SqlitePool {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            SqliteConnectOptions::from_str("sqlite::memory:")
                .unwrap()
                .foreign_keys(true),
        )
        .await
        .unwrap();
    migrate_sqlite(&pool).await.unwrap();
    pool
}

fn state() -> CampaignState {
    CampaignState::empty(
        Campaign {
            id: CampaignId::new(),
            display_name: "Presentation ledger".into(),
            status: CampaignStatus::Active,
            world_seed: 93,
            ruleset: VersionedRef {
                id: "test.rules".into(),
                version: "1".into(),
            },
            content_packs: vec![],
        },
        WorldClock {
            now: WorldInstant(0),
            calendar_id: "test".into(),
        },
    )
}

fn baseline(state: &CampaignState) -> TableProjectionRecord {
    TableProjectionRecord {
        version: 1,
        campaign_id: state.campaign_id(),
        ordinal: 1,
        cause: ProjectionCause::Bootstrap {
            event_sequence: 0,
            observation_ordinal: 0,
        },
        transcript: vec![],
        changes: vec![ProjectionChange {
            audience: ProjectionAudience::Host,
            previous: None,
            revision: ProjectionRevision(Uuid::new_v4()),
            visible_digest: "a".repeat(64),
            handles: vec![],
        }],
    }
}

#[tokio::test]
async fn protocol_history_roundtrips_and_is_immutable_until_whole_campaign_purge() {
    let pool = pool().await;
    let state = state();
    create_campaign(&pool, &state).await.unwrap();
    let record = baseline(&state);
    let mut tx = pool.begin_with("BEGIN IMMEDIATE").await.unwrap();
    insert_table_projection_record(&mut tx, &record)
        .await
        .unwrap();
    tx.commit().await.unwrap();
    let exported = export_campaign(&pool, state.campaign_id()).await.unwrap();
    assert_eq!(exported.format_version, 3);
    assert_eq!(exported.table_projection_history, vec![record]);
    for sql in [
        "DELETE FROM table_projection_history",
        "UPDATE table_projection_history SET ordinal=2",
    ] {
        assert!(sqlx::query(sql).execute(&pool).await.is_err());
    }
    let restored = self::pool().await;
    restore_campaign(&restored, &exported).await.unwrap();
    assert_eq!(
        export_campaign(&restored, state.campaign_id())
            .await
            .unwrap()
            .table_projection_history,
        exported.table_projection_history
    );
    let backup = export_campaign(&restored, state.campaign_id())
        .await
        .unwrap();
    purge_campaign(
        &restored,
        state.campaign_id(),
        &CampaignPurgeAuthorization::admin("test complete aggregate deletion"),
        &backup,
    )
    .await
    .unwrap();
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM table_projection_history")
            .fetch_one(&restored)
            .await
            .unwrap(),
        0
    );
}

#[tokio::test]
async fn legacy_exports_upgrade_without_protocol_but_cannot_smuggle_future_rows() {
    let pool = pool().await;
    let state = state();
    create_campaign(&pool, &state).await.unwrap();
    let exported = export_campaign(&pool, state.campaign_id()).await.unwrap();
    let mut legacy = exported.clone();
    legacy.format_version = 2;
    let before = legacy.clone();
    let upgraded = legacy.upgraded().unwrap();
    assert_eq!(legacy, before);
    assert_eq!(upgraded.format_version, 3);
    assert_eq!(upgraded.snapshots, legacy.snapshots);
    legacy.table_projection_history.push(baseline(&state));
    assert!(legacy.upgraded().is_err());
    let destination = self::pool().await;
    assert!(restore_campaign(&destination, &legacy).await.is_err());
    assert!(
        load_campaign_state(&destination, state.campaign_id())
            .await
            .unwrap()
            .is_none()
    );
}

#[tokio::test]
async fn transport_binding_matches_the_atomic_canonical_acceptance_and_rejects_foreign_provenance()
{
    let pool = pool().await;
    let state = state();
    create_campaign(&pool, &state).await.unwrap();
    let initial = baseline(&state);
    let meta = CommandMeta {
        id: CommandId::new(),
        campaign_id: state.campaign_id(),
        session_id: None,
        issuer: CommandIssuer::Admin,
        actor: None,
        expected_event_sequence: 0,
    };
    let mut next = state.clone();
    next.applied_event_sequence = 1;
    let event_id = EventId::new();
    let event = PendingEvent {
        id: event_id,
        occurred_at: state.clock.now,
        source: EventSource::RuleResolution,
        actor: None,
        caused_by_event_ids: vec![],
        payload: "accepted",
    }
    .encode("test.accepted", 1)
    .unwrap();
    let record = TableProjectionRecord {
        ordinal: 2,
        cause: ProjectionCause::Event {
            id: event_id,
            command_id: meta.id,
        },
        transcript: vec![TranscriptVisibility {
            event_id,
            audiences: vec![ProjectionAudience::Host],
        }],
        changes: vec![ProjectionChange {
            audience: ProjectionAudience::Host,
            previous: Some(initial.changes[0].revision),
            revision: ProjectionRevision(Uuid::new_v4()),
            visible_digest: "b".repeat(64),
            handles: vec![],
        }],
        ..initial.clone()
    };
    let binding = TableTransportBinding {
        version: 1,
        meta: meta.clone(),
        audience: ProjectionAudience::Host,
        projection_ordinal: 2,
        acceptance: TransportAcceptance::Command {
            resulting_event_sequence: 1,
        },
        request_json: "{\"action\":\"original\"}".into(),
        response_json: "{\"message\":\"accepted\"}".into(),
    };
    let mut tx = pool.begin_with("BEGIN IMMEDIATE").await.unwrap();
    insert_table_projection_record(&mut tx, &initial)
        .await
        .unwrap();
    commit_campaign_transition_in_transaction(
        &mut tx,
        &meta,
        &SerializedRecord::encode("test.action", 1, &"original").unwrap(),
        &next,
        &[event],
        "accepted",
        None,
    )
    .await
    .unwrap();
    insert_table_projection_record(&mut tx, &record)
        .await
        .unwrap();
    insert_table_transport_binding(&mut tx, &binding)
        .await
        .unwrap();
    tx.commit().await.unwrap();
    let exported = export_campaign(&pool, state.campaign_id()).await.unwrap();
    assert_eq!(exported.table_transport_bindings, vec![binding]);
    for sql in [
        "DELETE FROM table_transport_bindings",
        "UPDATE table_transport_bindings SET projection_ordinal=1",
    ] {
        assert!(sqlx::query(sql).execute(&pool).await.is_err());
    }
    let restored = self::pool().await;
    restore_campaign(&restored, &exported).await.unwrap();
    assert_eq!(
        export_campaign(&restored, state.campaign_id())
            .await
            .unwrap()
            .table_transport_bindings,
        exported.table_transport_bindings
    );
    let mut corruptions = Vec::new();
    let mut altered = exported.clone();
    altered.table_transport_bindings[0].projection_ordinal = 1;
    corruptions.push(altered);
    let mut altered = exported.clone();
    altered.table_transport_bindings[0].meta.id = CommandId::new();
    corruptions.push(altered);
    let mut altered = exported.clone();
    altered.table_projection_history[1].changes[0].previous =
        Some(ProjectionRevision(Uuid::new_v4()));
    corruptions.push(altered);
    let mut altered = exported.clone();
    altered.table_projection_history.pop();
    corruptions.push(altered);
    for altered in corruptions {
        let destination = self::pool().await;
        assert!(restore_campaign(&destination, &altered).await.is_err());
        assert_eq!(
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM campaign_state_current")
                .fetch_one(&destination)
                .await
                .unwrap(),
            0
        );
    }
}
