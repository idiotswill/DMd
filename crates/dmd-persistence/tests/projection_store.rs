use std::{path::PathBuf, str::FromStr};

use dmd_domain::{
    Campaign, CampaignId, CampaignState, CampaignStatus, CommandId, CommandIssuer, CommandMeta,
    EntityExistence, EntityId, EntityKind, EventId, EventSource, Location, LocationId,
    PendingEvent, SerializedRecord, VersionedRef, WorldClock, WorldDuration, WorldEntity,
    WorldInstant,
};
use dmd_persistence::{
    ProjectionStoreError, ReplayApplyError, ReplayEventApplier, StoredJournalEvent,
    commit_campaign_transition, initialize_campaign_state, list_projected_entities_at_location,
    load_campaign_projection_summary, load_campaign_state, load_journal_events, migrate_sqlite,
    open_sqlite, rebuild_campaign_projections,
};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
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

fn state(name: &str) -> (CampaignState, LocationId, EntityId) {
    let campaign_id = CampaignId::new();
    let location_id = LocationId::new();
    let entity_id = EntityId::new();
    let mut state = CampaignState::empty(
        Campaign {
            id: campaign_id,
            display_name: name.into(),
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
    );
    state.locations.insert(
        location_id,
        Location {
            id: location_id,
            campaign_id,
            display_name: "Test Location".into(),
            parent_location_id: None,
        },
    );
    state.entities.insert(
        entity_id,
        WorldEntity {
            id: entity_id,
            campaign_id,
            display_name: "Test Entity".into(),
            kind: EntityKind::Npc,
            existence: EntityExistence::Present,
            location_id: Some(location_id),
        },
    );
    (state, location_id, entity_id)
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

fn event(at: WorldInstant) -> dmd_domain::EncodedPendingEvent {
    PendingEvent {
        id: EventId::new(),
        occurred_at: at,
        source: EventSource::WorldSimulation,
        actor: None,
        caused_by_event_ids: vec![],
        payload: WorldDuration(1),
    }
    .encode("test.time_advanced", 1)
    .expect("typed event payload should encode")
}

struct NoopApplier;

impl ReplayEventApplier for NoopApplier {
    fn apply(
        &self,
        _state: &mut CampaignState,
        event: &StoredJournalEvent,
    ) -> Result<(), ReplayApplyError> {
        Err(ReplayApplyError::UnsupportedEvent {
            kind: event.payload.kind.clone(),
            schema_version: event.payload.schema_version,
        })
    }
}

#[tokio::test]
async fn projections_are_queryable_and_campaign_isolated() {
    let pool = test_pool().await;
    let (a, a_location, a_entity) = state("A");
    let (b, b_location, b_entity) = state("B");
    initialize_campaign_state(&pool, &a).await.unwrap();
    initialize_campaign_state(&pool, &b).await.unwrap();

    let a_summary = load_campaign_projection_summary(&pool, a.campaign_id())
        .await
        .unwrap();
    let b_summary = load_campaign_projection_summary(&pool, b.campaign_id())
        .await
        .unwrap();
    assert_eq!(a_summary.entities, 1);
    assert_eq!(b_summary.entities, 1);

    let a_rows = list_projected_entities_at_location(
        &pool,
        a.campaign_id(),
        Some(&a_location.0.to_string()),
    )
    .await
    .unwrap();
    assert_eq!(a_rows.len(), 1);
    assert_eq!(a_rows[0].entity_id, a_entity.0.to_string());
    assert_ne!(a_rows[0].entity_id, b_entity.0.to_string());

    let wrong_campaign_rows = list_projected_entities_at_location(
        &pool,
        a.campaign_id(),
        Some(&b_location.0.to_string()),
    )
    .await
    .unwrap();
    assert!(wrong_campaign_rows.is_empty());
}

#[tokio::test]
async fn projection_corruption_is_detected_and_rebuild_repairs_from_replay() {
    let pool = test_pool().await;
    let (state, _, _) = state("Repair");
    initialize_campaign_state(&pool, &state).await.unwrap();

    sqlx::query("DELETE FROM projection_entities WHERE campaign_id = ?")
        .bind(state.campaign_id().0.to_string())
        .execute(&pool)
        .await
        .unwrap();
    assert!(matches!(
        load_campaign_projection_summary(&pool, state.campaign_id()).await,
        Err(ProjectionStoreError::CountMismatch)
    ));

    let rebuilt = rebuild_campaign_projections(&pool, state.campaign_id(), &NoopApplier)
        .await
        .unwrap();
    assert_eq!(rebuilt.applied_event_sequence, 0);
    assert_eq!(rebuilt.entities, 1);
}

#[tokio::test]
async fn projection_failure_rolls_back_authoritative_transition_and_journal() {
    let pool = test_pool().await;
    let (current, _, _) = state("Atomic");
    initialize_campaign_state(&pool, &current).await.unwrap();
    sqlx::query(
        r#"
        CREATE TRIGGER test_projection_failure
        BEFORE INSERT ON projection_entities
        BEGIN
            SELECT RAISE(ABORT, 'forced projection failure');
        END;
        "#,
    )
    .execute(&pool)
    .await
    .unwrap();

    let mut next = current.clone();
    next.clock.now = next.clock.now.advance(WorldDuration(1));
    next.applied_event_sequence = 1;
    let result = commit_campaign_transition(
        &pool,
        &command_meta(current.campaign_id(), 0),
        &command_payload(),
        &next,
        &[event(next.clock.now)],
        "advance time",
    )
    .await;
    assert!(result.is_err());

    let loaded = load_campaign_state(&pool, current.campaign_id())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(loaded, current);
    assert!(
        load_journal_events(&pool, current.campaign_id(), 0)
            .await
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        load_campaign_projection_summary(&pool, current.campaign_id())
            .await
            .unwrap()
            .applied_event_sequence,
        0
    );
}

#[tokio::test]
async fn projections_survive_close_and_reopen() {
    let path: PathBuf =
        std::env::temp_dir().join(format!("dmd-projections-{}.sqlite", Uuid::new_v4()));
    let url = format!("sqlite://{}", path.display());
    let pool = open_sqlite(&url).await.unwrap();
    let (state, location_id, entity_id) = state("Restart");
    initialize_campaign_state(&pool, &state).await.unwrap();
    pool.close().await;

    let reopened = open_sqlite(&url).await.unwrap();
    let rows = list_projected_entities_at_location(
        &reopened,
        state.campaign_id(),
        Some(&location_id.0.to_string()),
    )
    .await
    .unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].entity_id, entity_id.0.to_string());
    reopened.close().await;
    let _ = std::fs::remove_file(path);
}
