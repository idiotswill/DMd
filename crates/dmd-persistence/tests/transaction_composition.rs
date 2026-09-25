use dmd_domain::*;
use dmd_persistence::*;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::{str::FromStr, time::Duration};

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
            display_name: "Atomic presentation composition".into(),
            status: CampaignStatus::Active,
            world_seed: 912,
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

#[tokio::test]
async fn accepted_game_and_observation_remain_uncommitted_until_outer_transaction_commits() {
    let pool = pool().await;
    let before = state();
    create_campaign(&pool, &before).await.unwrap();
    let meta = CommandMeta {
        id: CommandId::new(),
        campaign_id: before.campaign_id(),
        session_id: None,
        issuer: CommandIssuer::Admin,
        actor: None,
        expected_event_sequence: 0,
    };
    let mut after = before.clone();
    after.applied_event_sequence = 1;
    after.clock.now = WorldInstant(1);
    let payload = SerializedRecord::encode("test.time", 1, &1_u8).unwrap();
    let event = PendingEvent {
        id: EventId::new(),
        occurred_at: after.clock.now,
        source: EventSource::RuleResolution,
        actor: None,
        caused_by_event_ids: vec![],
        payload: 1_u8,
    }
    .encode("test.time", 1)
    .unwrap();
    let observation = NewSessionObservation {
        id: ObservationId::new(),
        campaign_id: before.campaign_id(),
        session_id: None,
        issuer: CommandIssuer::Admin,
        audience: ObservationAudience::Party,
        observed_event_sequence: 1,
        kind: "test.presentation".into(),
        payload_schema_version: 1,
        payload_json: "{\"text\":\"Visible outcome\"}".into(),
    };

    // With only one connection, any accidental pool acquisition inside these helpers
    // would deadlock rather than successfully composing the original transaction.
    for commit in [false, true] {
        let mut tx = pool.begin_with("BEGIN IMMEDIATE").await.unwrap();
        tokio::time::timeout(Duration::from_secs(10), async {
            commit_campaign_transition_in_transaction(
                &mut tx,
                &meta,
                &payload,
                &after,
                std::slice::from_ref(&event),
                "Time advanced",
                None,
            )
            .await
            .unwrap();
            append_session_observations_in_transaction(
                &mut tx,
                before.campaign_id(),
                std::slice::from_ref(&observation),
            )
            .await
            .unwrap();
            let staged = export_campaign_in_transaction(&mut tx, before.campaign_id())
                .await
                .unwrap();
            assert_eq!(staged.current_state.applied_event_sequence, 1);
            assert_eq!(staged.command_audit.len(), 1);
            assert_eq!(staged.event_journal.len(), 1);
            assert_eq!(staged.observations.len(), 1);
        })
        .await
        .expect("same-connection composition must not await another pool connection");
        if commit {
            tx.commit().await.unwrap();
        } else {
            tx.rollback().await.unwrap();
        }
        let persisted = export_campaign(&pool, before.campaign_id()).await.unwrap();
        assert_eq!(
            persisted.current_state.applied_event_sequence,
            i64::from(commit)
        );
        assert_eq!(persisted.command_audit.len(), usize::from(commit));
        assert_eq!(persisted.event_journal.len(), usize::from(commit));
        assert_eq!(persisted.observations.len(), usize::from(commit));
    }
}
