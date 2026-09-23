use std::str::FromStr;

use dmd_domain::{
    Campaign, CampaignId, CampaignState, CampaignStatus, Claim, ClaimId, ClaimSource, CommandId,
    CommandIssuer, CommandMeta, EventId, EventSource, FactValue, PendingEvent, Proposition,
    SerializedRecord, SubjectRef, VersionedRef, WorldClock, WorldDuration, WorldInstant,
};
use dmd_persistence::{
    JournalStoreError, commit_campaign_transition, initialize_campaign_state, load_command_audit,
    migrate_sqlite,
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
            display_name: "Journal Integrity Test".into(),
            status: CampaignStatus::Active,
            world_seed: 91,
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
    SerializedRecord::encode("test.advance", 1, &WorldDuration(1))
        .expect("typed command should encode")
}

fn event(id: EventId, causes: Vec<EventId>, at: WorldInstant) -> dmd_domain::EncodedPendingEvent {
    PendingEvent {
        id,
        occurred_at: at,
        source: EventSource::WorldSimulation,
        actor: None,
        caused_by_event_ids: causes,
        payload: WorldDuration(1),
    }
    .encode("test.advanced", 1)
    .expect("typed event should encode")
}

fn advance(current: &CampaignState) -> CampaignState {
    let mut next = current.clone();
    next.clock.now = next.clock.now.advance(WorldDuration(1));
    next.applied_event_sequence += 1;
    next
}

#[tokio::test]
async fn transition_rejects_corrupt_current_state_provenance_before_writing() {
    let pool = test_pool().await;
    let initial = state();
    initialize_campaign_state(&pool, &initial)
        .await
        .expect("state should initialize");

    let first_meta = command_meta(initial.campaign_id(), 0);
    let first_event = EventId::new();
    let after_first = advance(&initial);
    commit_campaign_transition(
        &pool,
        &first_meta,
        &command_payload(),
        &after_first,
        &[event(first_event, vec![], WorldInstant(101))],
        "First valid change.",
    )
    .await
    .expect("first transition should commit");

    let missing_event = EventId::new();
    let mut corrupt = after_first.clone();
    let claim = Claim {
        id: ClaimId::new(),
        campaign_id: corrupt.campaign_id(),
        source: ClaimSource::Unknown,
        proposition: Proposition {
            subject: SubjectRef::Campaign(corrupt.campaign_id()),
            predicate: "corrupt_provenance".into(),
            value: FactValue::Boolean(true),
        },
        made_at: corrupt.clock.now,
        source_event_id: missing_event,
    };
    corrupt.claims.insert(claim.id, claim);
    let corrupt_json = corrupt
        .encode_json()
        .expect("corrupt fixture should encode");
    sqlx::query("UPDATE campaign_state_current SET state_json = ? WHERE campaign_id = ?")
        .bind(corrupt_json)
        .bind(corrupt.campaign_id().0.to_string())
        .execute(&pool)
        .await
        .expect("fixture should corrupt only the serialized current state");

    let second_meta = command_meta(after_first.campaign_id(), 1);
    let result = commit_campaign_transition(
        &pool,
        &second_meta,
        &command_payload(),
        &advance(&after_first),
        &[event(EventId::new(), vec![first_event], WorldInstant(102))],
        "Must not heal corrupt provenance implicitly.",
    )
    .await;

    assert!(matches!(
        result,
        Err(JournalStoreError::MissingStateEventReference(event_id)) if event_id == missing_event
    ));

    let row = sqlx::query(
        "SELECT applied_event_sequence FROM campaign_state_current WHERE campaign_id = ?",
    )
    .bind(after_first.campaign_id().0.to_string())
    .fetch_one(&pool)
    .await
    .expect("state head should remain readable");
    let sequence: i64 = row
        .try_get("applied_event_sequence")
        .expect("sequence column");
    assert_eq!(sequence, 1);

    let event_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM event_journal WHERE campaign_id = ?")
            .bind(after_first.campaign_id().0.to_string())
            .fetch_one(&pool)
            .await
            .expect("event count should load");
    assert_eq!(event_count, 1);
    assert!(
        load_command_audit(&pool, second_meta.id)
            .await
            .expect("audit lookup should succeed")
            .is_none()
    );
}

#[tokio::test]
async fn journal_history_rejects_direct_deletes() {
    let pool = test_pool().await;
    let initial = state();
    initialize_campaign_state(&pool, &initial)
        .await
        .expect("state should initialize");

    let first_meta = command_meta(initial.campaign_id(), 0);
    let first_event = EventId::new();
    let after_first = advance(&initial);
    commit_campaign_transition(
        &pool,
        &first_meta,
        &command_payload(),
        &after_first,
        &[event(first_event, vec![], WorldInstant(101))],
        "First event.",
    )
    .await
    .expect("first event should commit");

    let second_meta = command_meta(initial.campaign_id(), 1);
    let second_event = EventId::new();
    let after_second = advance(&after_first);
    commit_campaign_transition(
        &pool,
        &second_meta,
        &command_payload(),
        &after_second,
        &[event(second_event, vec![first_event], WorldInstant(102))],
        "Second event caused by first.",
    )
    .await
    .expect("second event should commit");

    let cause_delete =
        sqlx::query("DELETE FROM event_causes WHERE campaign_id = ? AND event_id = ?")
            .bind(initial.campaign_id().0.to_string())
            .bind(second_event.0.to_string())
            .execute(&pool)
            .await;
    assert!(cause_delete.is_err(), "causal history must be append-only");

    let event_delete = sqlx::query("DELETE FROM event_journal WHERE id = ?")
        .bind(second_event.0.to_string())
        .execute(&pool)
        .await;
    assert!(event_delete.is_err(), "event history must be append-only");

    let command_delete = sqlx::query("DELETE FROM command_audit WHERE id = ?")
        .bind(first_meta.id.0.to_string())
        .execute(&pool)
        .await;
    assert!(command_delete.is_err(), "command audit must be append-only");

    let campaign_delete = sqlx::query("DELETE FROM campaign_state_current WHERE campaign_id = ?")
        .bind(initial.campaign_id().0.to_string())
        .execute(&pool)
        .await;
    assert!(
        campaign_delete.is_err(),
        "campaign deletion must wait for an explicit aggregate-purge lifecycle path"
    );

    let state_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM campaign_state_current WHERE campaign_id = ?")
            .bind(initial.campaign_id().0.to_string())
            .fetch_one(&pool)
            .await
            .expect("state count should load");
    let event_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM event_journal WHERE campaign_id = ?")
            .bind(initial.campaign_id().0.to_string())
            .fetch_one(&pool)
            .await
            .expect("event count should load");
    let cause_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM event_causes WHERE campaign_id = ?")
            .bind(initial.campaign_id().0.to_string())
            .fetch_one(&pool)
            .await
            .expect("cause count should load");

    assert_eq!(state_count, 1);
    assert_eq!(event_count, 2);
    assert_eq!(cause_count, 1);
}
