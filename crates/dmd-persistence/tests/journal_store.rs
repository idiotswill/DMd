use std::{fs, path::Path};

use dmd_domain::{
    Campaign, CampaignId, CampaignState, CampaignStatus, Claim, ClaimId, ClaimSource, CommandId,
    CommandIssuer, CommandMeta, EventId, EventSource, FactValue, PendingEvent, PlaySessionId,
    Proposition, SerializedRecord, SubjectRef, VersionedRef, WorldClock, WorldDuration,
    WorldInstant,
};
use dmd_persistence::{
    JournalStoreError, commit_campaign_transition, initialize_campaign_state, load_campaign_state,
    load_command_audit, load_journal_events, migrate_sqlite, open_sqlite,
};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::str::FromStr;
use uuid::Uuid;

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
            display_name: "Journal Test".into(),
            status: CampaignStatus::Active,
            world_seed: 77,
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
        .expect("typed command payload should encode")
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
    .encode("test.time_advanced", 1)
    .expect("typed event payload should encode")
}

fn next_state(current: &CampaignState, event_count: u64) -> CampaignState {
    let mut next = current.clone();
    next.clock.now = next.clock.now.advance(WorldDuration(event_count as i64));
    next.applied_event_sequence = current.applied_event_sequence + event_count;
    next
}

#[tokio::test]
async fn initializes_and_loads_clean_campaign_state() {
    let pool = test_pool().await;
    let state = state();

    initialize_campaign_state(&pool, &state)
        .await
        .expect("clean state should initialize");
    let loaded = load_campaign_state(&pool, state.campaign_id())
        .await
        .expect("state should load")
        .expect("state should exist");

    assert_eq!(loaded, state);
    assert!(
        load_journal_events(&pool, state.campaign_id(), 0)
            .await
            .expect("journal should load")
            .is_empty()
    );
}

#[tokio::test]
async fn initial_state_cannot_reference_nonexistent_journal_event() {
    let pool = test_pool().await;
    let mut initial = state();
    let missing_event = EventId::new();
    let claim = Claim {
        id: ClaimId::new(),
        campaign_id: initial.campaign_id(),
        source: ClaimSource::Unknown,
        proposition: Proposition {
            subject: SubjectRef::Campaign(initial.campaign_id()),
            predicate: "preexisting_claim".into(),
            value: FactValue::Boolean(true),
        },
        made_at: initial.clock.now,
        source_event_id: missing_event,
    };
    initial.claims.insert(claim.id, claim);

    let result = initialize_campaign_state(&pool, &initial).await;

    assert!(matches!(
        result,
        Err(JournalStoreError::MissingStateEventReference(event_id)) if event_id == missing_event
    ));
    assert!(
        load_campaign_state(&pool, initial.campaign_id())
            .await
            .expect("lookup should succeed")
            .is_none()
    );
}

#[tokio::test]
async fn accepted_transition_commits_state_audit_and_event_atomically() {
    let pool = test_pool().await;
    let current = state();
    initialize_campaign_state(&pool, &current)
        .await
        .expect("state should initialize");

    let meta = command_meta(current.campaign_id(), 0);
    let payload = command_payload();
    let event_id = EventId::new();
    let event = event(event_id, vec![], WorldInstant(101));
    let next = next_state(&current, 1);

    let receipt = commit_campaign_transition(
        &pool,
        &meta,
        &payload,
        &next,
        &[event],
        "World time advanced by one test tick.",
    )
    .await
    .expect("transition should commit");

    assert_eq!(receipt.command_id, meta.id);
    assert_eq!(receipt.emitted_event_ids, vec![event_id]);
    assert_eq!(receipt.resulting_event_sequence, 1);

    let loaded = load_campaign_state(&pool, current.campaign_id())
        .await
        .expect("state should load")
        .expect("state should exist");
    assert_eq!(loaded, next);

    let journal = load_journal_events(&pool, current.campaign_id(), 0)
        .await
        .expect("journal should load");
    assert_eq!(journal.len(), 1);
    assert_eq!(journal[0].meta.sequence, 1);
    assert_eq!(journal[0].meta.command_id, Some(meta.id));
    assert_eq!(journal[0].payload.kind, "test.time_advanced");

    let audit = load_command_audit(&pool, meta.id)
        .await
        .expect("audit should load")
        .expect("audit should exist");
    assert_eq!(audit.meta.issuer, CommandIssuer::System);
    assert_eq!(audit.meta.actor, None);
    assert_eq!(audit.payload.kind, "test.advance_time");
    assert_eq!(audit.emitted_event_ids, vec![event_id]);
    assert_eq!(audit.resulting_event_sequence, 1);
}

#[tokio::test]
async fn stale_transition_is_rejected_without_mutation() {
    let pool = test_pool().await;
    let current = state();
    initialize_campaign_state(&pool, &current)
        .await
        .expect("state should initialize");

    let first_meta = command_meta(current.campaign_id(), 0);
    let first_next = next_state(&current, 1);
    commit_campaign_transition(
        &pool,
        &first_meta,
        &command_payload(),
        &first_next,
        &[event(EventId::new(), vec![], WorldInstant(101))],
        "First change.",
    )
    .await
    .expect("first transition should commit");

    let stale_meta = command_meta(current.campaign_id(), 0);
    let stale_next = next_state(&first_next, 1);
    let result = commit_campaign_transition(
        &pool,
        &stale_meta,
        &command_payload(),
        &stale_next,
        &[event(EventId::new(), vec![], WorldInstant(102))],
        "Stale change.",
    )
    .await;

    assert!(matches!(
        result,
        Err(JournalStoreError::StaleState {
            expected: 0,
            actual: 1
        })
    ));
    let loaded = load_campaign_state(&pool, current.campaign_id())
        .await
        .expect("state should load")
        .expect("state should exist");
    assert_eq!(loaded, first_next);
    assert_eq!(
        load_journal_events(&pool, current.campaign_id(), 0)
            .await
            .expect("journal should load")
            .len(),
        1
    );
}

#[tokio::test]
async fn same_batch_causal_parent_must_precede_child() {
    let pool = test_pool().await;
    let current = state();
    initialize_campaign_state(&pool, &current)
        .await
        .expect("state should initialize");
    let first = EventId::new();
    let second = EventId::new();
    let next = next_state(&current, 2);

    commit_campaign_transition(
        &pool,
        &command_meta(current.campaign_id(), 0),
        &command_payload(),
        &next,
        &[
            event(first, vec![], WorldInstant(101)),
            event(second, vec![first], WorldInstant(102)),
        ],
        "Two causally ordered changes.",
    )
    .await
    .expect("ordered causal batch should commit");

    let journal = load_journal_events(&pool, current.campaign_id(), 0)
        .await
        .expect("journal should load");
    assert_eq!(journal[0].meta.sequence, 1);
    assert_eq!(journal[1].meta.sequence, 2);
    assert_eq!(journal[1].meta.caused_by_event_ids, vec![first]);
}

#[tokio::test]
async fn future_causal_parent_rejects_whole_batch() {
    let pool = test_pool().await;
    let current = state();
    initialize_campaign_state(&pool, &current)
        .await
        .expect("state should initialize");
    let first = EventId::new();
    let second = EventId::new();
    let next = next_state(&current, 2);

    let result = commit_campaign_transition(
        &pool,
        &command_meta(current.campaign_id(), 0),
        &command_payload(),
        &next,
        &[
            event(first, vec![second], WorldInstant(101)),
            event(second, vec![], WorldInstant(102)),
        ],
        "Invalid future cause.",
    )
    .await;

    assert!(matches!(
        result,
        Err(JournalStoreError::FutureCausalParent {
            event_id,
            parent_id
        }) if event_id == first && parent_id == second
    ));
    let loaded = load_campaign_state(&pool, current.campaign_id())
        .await
        .expect("state should load")
        .expect("state should exist");
    assert_eq!(loaded, current);
    assert!(
        load_journal_events(&pool, current.campaign_id(), 0)
            .await
            .expect("journal should load")
            .is_empty()
    );
}

#[tokio::test]
async fn cross_campaign_causal_parent_is_rejected() {
    let pool = test_pool().await;
    let first_campaign = state();
    let second_campaign = state();
    initialize_campaign_state(&pool, &first_campaign)
        .await
        .expect("first state should initialize");
    initialize_campaign_state(&pool, &second_campaign)
        .await
        .expect("second state should initialize");

    let parent_id = EventId::new();
    commit_campaign_transition(
        &pool,
        &command_meta(first_campaign.campaign_id(), 0),
        &command_payload(),
        &next_state(&first_campaign, 1),
        &[event(parent_id, vec![], WorldInstant(101))],
        "First campaign event.",
    )
    .await
    .expect("first campaign event should commit");

    let child_id = EventId::new();
    let result = commit_campaign_transition(
        &pool,
        &command_meta(second_campaign.campaign_id(), 0),
        &command_payload(),
        &next_state(&second_campaign, 1),
        &[event(child_id, vec![parent_id], WorldInstant(101))],
        "Cross-campaign cause should fail.",
    )
    .await;

    assert!(matches!(
        result,
        Err(JournalStoreError::CausalParentCampaignMismatch { event_id, .. })
            if event_id == parent_id
    ));
}

#[tokio::test]
async fn resulting_state_cannot_reference_unpersisted_event() {
    let pool = test_pool().await;
    let current = state();
    initialize_campaign_state(&pool, &current)
        .await
        .expect("state should initialize");
    let missing_event = EventId::new();
    let mut next = next_state(&current, 1);
    let claim = Claim {
        id: ClaimId::new(),
        campaign_id: current.campaign_id(),
        source: ClaimSource::Unknown,
        proposition: Proposition {
            subject: SubjectRef::Campaign(current.campaign_id()),
            predicate: "test_claim".into(),
            value: FactValue::Boolean(true),
        },
        made_at: WorldInstant(101),
        source_event_id: missing_event,
    };
    next.claims.insert(claim.id, claim);

    let result = commit_campaign_transition(
        &pool,
        &command_meta(current.campaign_id(), 0),
        &command_payload(),
        &next,
        &[event(EventId::new(), vec![], WorldInstant(101))],
        "Claim with missing provenance.",
    )
    .await;

    assert!(matches!(
        result,
        Err(JournalStoreError::MissingStateEventReference(event_id)) if event_id == missing_event
    ));
}

#[tokio::test]
async fn session_from_another_campaign_is_rejected() {
    let pool = test_pool().await;
    let current = state();
    initialize_campaign_state(&pool, &current)
        .await
        .expect("state should initialize");
    let session_id = PlaySessionId::new();
    sqlx::query(
        "INSERT INTO play_sessions (id, campaign_id, display_name, status, started_at_world, ended_at_world) VALUES (?, ?, 'Other Campaign Session', 'closed', 0, 1)",
    )
    .bind(session_id.0.to_string())
    .bind(CampaignId::new().0.to_string())
    .execute(&pool)
    .await
    .expect("foreign-campaign session fixture should insert");

    let mut meta = command_meta(current.campaign_id(), 0);
    meta.session_id = Some(session_id);
    let result = commit_campaign_transition(
        &pool,
        &meta,
        &command_payload(),
        &next_state(&current, 1),
        &[event(EventId::new(), vec![], WorldInstant(101))],
        "Wrong session campaign.",
    )
    .await;

    assert!(matches!(
        result,
        Err(JournalStoreError::SessionCampaignMismatch {
            session_id: actual,
            ..
        }) if actual == session_id
    ));
}

#[tokio::test]
async fn rollback_and_reopen_exposes_only_last_committed_state() {
    let path = std::env::temp_dir().join(format!("dmd-journal-{}.sqlite", Uuid::new_v4()));
    let url = format!("sqlite://{}", path.display());
    let pool = open_sqlite(&url).await.expect("file database should open");
    let current = state();
    initialize_campaign_state(&pool, &current)
        .await
        .expect("state should initialize");

    let mut transaction = pool.begin().await.expect("transaction should begin");
    sqlx::query(
        "UPDATE campaign_state_current SET applied_event_sequence = 99 WHERE campaign_id = ?",
    )
    .bind(current.campaign_id().0.to_string())
    .execute(&mut *transaction)
    .await
    .expect("test mutation should execute inside transaction");
    transaction
        .rollback()
        .await
        .expect("test transaction should roll back");
    pool.close().await;

    let reopened = open_sqlite(&url).await.expect("database should reopen");
    let loaded = load_campaign_state(&reopened, current.campaign_id())
        .await
        .expect("state should recover")
        .expect("state should exist");
    assert_eq!(loaded, current);
    assert!(
        load_journal_events(&reopened, current.campaign_id(), 0)
            .await
            .expect("journal should load")
            .is_empty()
    );
    reopened.close().await;
    cleanup_sqlite_files(&path);
}

fn cleanup_sqlite_files(path: &Path) {
    let _ = fs::remove_file(path);
    let _ = fs::remove_file(format!("{}-wal", path.display()));
    let _ = fs::remove_file(format!("{}-shm", path.display()));
}
