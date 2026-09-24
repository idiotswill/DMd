use std::str::FromStr;

use dmd_domain::{
    AgentRef, AttendanceStatus, Campaign, CampaignId, CampaignState, CampaignStatus, Character,
    CharacterId, CharacterStatus, CommandId, CommandIssuer, CommandMeta, EntityExistence, EntityId,
    EntityKind, EventId, EventSource, Faction, FactionId, FactionStatus, PendingEvent, PlaySession,
    PlaySessionId, PlaySessionStatus, Player, PlayerId, SerializedRecord, SessionParticipant,
    VersionedRef, WorldClock, WorldDuration, WorldEntity, WorldInstant,
};
use dmd_persistence::{
    CampaignExport, CampaignPurgeAuthorization, LifecycleError, commit_campaign_transition,
    create_campaign, export_campaign, load_campaign_projection_summary, migrate_sqlite,
    open_campaign, purge_campaign, restore_campaign, save_play_session,
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

struct Fixture {
    state: CampaignState,
    player_id: PlayerId,
    character_id: CharacterId,
    entity_id: EntityId,
    faction_id: FactionId,
}

fn fixture() -> Fixture {
    let campaign_id = CampaignId::new();
    let mut state = CampaignState::empty(
        Campaign {
            id: campaign_id,
            display_name: "Lifecycle integrity".into(),
            status: CampaignStatus::Active,
            world_seed: 303,
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
    let player_id = PlayerId::new();
    state.players.insert(
        player_id,
        Player {
            id: player_id,
            campaign_id,
            display_name: "Player".into(),
        },
    );
    let entity_id = EntityId::new();
    state.entities.insert(
        entity_id,
        WorldEntity {
            id: entity_id,
            campaign_id,
            display_name: "Character entity".into(),
            kind: EntityKind::Character,
            existence: EntityExistence::Present,
            location_id: None,
        },
    );
    let character_id = CharacterId::new();
    state.characters.insert(
        character_id,
        Character {
            id: character_id,
            entity_id,
            campaign_id,
            controlling_player_id: Some(player_id),
            display_name: "Character".into(),
            status: CharacterStatus::Active,
        },
    );
    let faction_id = FactionId::new();
    state.factions.insert(
        faction_id,
        Faction {
            id: faction_id,
            campaign_id,
            display_name: "Faction".into(),
            status: FactionStatus::Active,
        },
    );
    assert!(state.validate().is_empty());
    Fixture {
        state,
        player_id,
        character_id,
        entity_id,
        faction_id,
    }
}

async fn persisted_export(pool: &sqlx::SqlitePool) -> CampaignExport {
    let fixture = fixture();
    create_campaign(pool, &fixture.state)
        .await
        .expect("campaign should create");
    let session = PlaySession {
        id: PlaySessionId::new(),
        campaign_id: fixture.state.campaign_id(),
        display_name: "Integrity session".into(),
        status: PlaySessionStatus::Active,
        started_at_world: fixture.state.clock.now,
        ended_at_world: None,
        participants: vec![SessionParticipant {
            player_id: fixture.player_id,
            character_id: Some(fixture.character_id),
            attendance: AttendanceStatus::Present,
        }],
    };
    save_play_session(pool, &fixture.state, &session)
        .await
        .expect("session should persist");
    let first_id = EventId::new();
    let second_id = EventId::new();
    let events = vec![
        PendingEvent {
            id: first_id,
            occurred_at: WorldInstant(101),
            source: EventSource::RuleResolution,
            actor: Some(AgentRef::Faction(fixture.faction_id)),
            caused_by_event_ids: vec![],
            payload: WorldDuration(1),
        }
        .encode("test.integrity_event", 1)
        .expect("event should encode"),
        PendingEvent {
            id: second_id,
            occurred_at: WorldInstant(102),
            source: EventSource::RuleResolution,
            actor: Some(AgentRef::Faction(fixture.faction_id)),
            caused_by_event_ids: vec![first_id],
            payload: WorldDuration(1),
        }
        .encode("test.integrity_event", 1)
        .expect("event should encode"),
    ];
    let mut next = fixture.state.clone();
    next.clock.now = WorldInstant(102);
    next.applied_event_sequence = 2;
    let command = CommandMeta {
        id: CommandId::new(),
        campaign_id: fixture.state.campaign_id(),
        session_id: Some(session.id),
        issuer: CommandIssuer::Player(fixture.player_id),
        actor: Some(AgentRef::Entity(fixture.entity_id)),
        expected_event_sequence: 0,
    };
    let payload = SerializedRecord::encode("test.integrity_command", 1, &WorldDuration(2))
        .expect("command should encode");
    commit_campaign_transition(
        pool,
        &command,
        &payload,
        &next,
        &events,
        "exercise lifecycle integrity",
    )
    .await
    .expect("transition should commit");
    export_campaign(pool, fixture.state.campaign_id())
        .await
        .expect("campaign should export")
}

#[tokio::test]
async fn purge_authorization_cannot_unlock_selective_history_deletion() {
    let pool = test_pool().await;
    let export = persisted_export(&pool).await;
    sqlx::query(
        "INSERT INTO campaign_purge_authorizations (campaign_id, authorized_by, reason) VALUES (?, 'admin', ?)",
    )
    .bind(&export.campaign_id)
    .bind("attempt selective surgery")
    .execute(&pool)
    .await
    .expect("authorization row should be insertable by the lifecycle transaction shape");
    for table in [
        "event_causes",
        "event_journal",
        "command_audit",
        "campaign_snapshots",
    ] {
        let sql = format!("DELETE FROM {table} WHERE campaign_id = ?");
        let error = sqlx::query(&sql)
            .bind(&export.campaign_id)
            .execute(&pool)
            .await
            .expect_err("selective history deletion must remain blocked");
        assert!(
            error.to_string().contains("immutable"),
            "unexpected {table} error: {error}"
        );
    }
    sqlx::query("DELETE FROM campaign_purge_authorizations WHERE campaign_id = ?")
        .bind(&export.campaign_id)
        .execute(&pool)
        .await
        .expect("test authorization cleanup should work");
    let campaign_id = CampaignId(uuid::Uuid::parse_str(&export.campaign_id).unwrap());
    open_campaign(&pool, campaign_id)
        .await
        .expect("failed selective surgery must leave campaign intact");
}

#[tokio::test]
async fn authorized_root_delete_mechanically_removes_complete_campaign_aggregate() {
    let pool = test_pool().await;
    let export = persisted_export(&pool).await;
    let session_id = export.play_sessions[0].id.clone();

    sqlx::query(
        "INSERT INTO campaign_purge_authorizations (campaign_id, authorized_by, reason) VALUES (?, 'admin', ?)",
    )
    .bind(&export.campaign_id)
    .bind("exercise direct aggregate root purge")
    .execute(&pool)
    .await
    .expect("manual purge authorization should insert");

    sqlx::query("DELETE FROM campaign_state_current WHERE campaign_id = ?")
        .bind(&export.campaign_id)
        .execute(&pool)
        .await
        .expect("authorized root deletion should complete the aggregate purge");

    for table in [
        "campaign_state_current",
        "campaign_lifecycle",
        "play_sessions",
        "command_audit",
        "event_journal",
        "event_causes",
        "campaign_snapshots",
        "campaign_purge_authorizations",
        "campaign_restore_authorizations",
    ] {
        let sql = format!("SELECT COUNT(*) FROM {table} WHERE campaign_id = ?");
        let count: i64 = sqlx::query_scalar(&sql)
            .bind(&export.campaign_id)
            .fetch_one(&pool)
            .await
            .expect("campaign-owned row count should query");
        assert_eq!(count, 0, "{table} must be removed by root purge");
    }

    let participants: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM play_session_participants WHERE session_id = ?")
            .bind(&session_id)
            .fetch_one(&pool)
            .await
            .expect("participant count should query");
    assert_eq!(participants, 0, "session participants must cascade away");

    let projections: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM projection_heads WHERE campaign_id = ?")
            .bind(&export.campaign_id)
            .fetch_one(&pool)
            .await
            .expect("projection head count should query");
    assert_eq!(projections, 0, "derivative projections must cascade away");
}

async fn assert_restore_rejected_without_write(
    pool: &sqlx::SqlitePool,
    export: &CampaignExport,
    label: &str,
) {
    assert!(
        matches!(
            restore_campaign(pool, export).await,
            Err(LifecycleError::CorruptExport(_))
        ),
        "{label} must be rejected as corrupt"
    );
    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM campaign_state_current WHERE campaign_id = ?")
            .bind(&export.campaign_id)
            .fetch_one(pool)
            .await
            .expect("existence query should work");
    assert_eq!(count, 0, "{label} must fail before restore writes commit");
}

#[tokio::test]
async fn restore_rejects_invalid_session_and_authority_references_before_writing() {
    let pool = test_pool().await;
    let export = persisted_export(&pool).await;
    let campaign_id = CampaignId(uuid::Uuid::parse_str(&export.campaign_id).unwrap());
    purge_campaign(
        &pool,
        campaign_id,
        &CampaignPurgeAuthorization::admin("prepare restore integrity mutations"),
        &export,
    )
    .await
    .expect("valid exact backup should permit purge");

    let mut missing_player = export.clone();
    missing_player.play_session_participants[0].player_id = uuid::Uuid::new_v4().to_string();
    assert_restore_rejected_without_write(&pool, &missing_player, "missing participant player")
        .await;

    let mut missing_character = export.clone();
    missing_character.play_session_participants[0].character_id =
        Some(uuid::Uuid::new_v4().to_string());
    assert_restore_rejected_without_write(
        &pool,
        &missing_character,
        "missing participant character",
    )
    .await;

    let mut missing_issuer = export.clone();
    missing_issuer.command_audit[0].issuer_player_id = Some(uuid::Uuid::new_v4().to_string());
    assert_restore_rejected_without_write(&pool, &missing_issuer, "missing player issuer").await;

    let mut missing_command_actor = export.clone();
    missing_command_actor.command_audit[0].actor_id = Some(uuid::Uuid::new_v4().to_string());
    assert_restore_rejected_without_write(&pool, &missing_command_actor, "missing command actor")
        .await;

    let mut missing_event_actor = export.clone();
    missing_event_actor.event_journal[0].actor_id = Some(uuid::Uuid::new_v4().to_string());
    assert_restore_rejected_without_write(&pool, &missing_event_actor, "missing event actor").await;

    let mut missing_session = export.clone();
    missing_session.command_audit[0].session_id = Some(uuid::Uuid::new_v4().to_string());
    assert_restore_rejected_without_write(&pool, &missing_session, "missing command session").await;

    let mut cross_campaign_session = export.clone();
    cross_campaign_session.play_sessions[0].campaign_id = uuid::Uuid::new_v4().to_string();
    assert_restore_rejected_without_write(
        &pool,
        &cross_campaign_session,
        "cross-campaign play session",
    )
    .await;

    restore_campaign(&pool, &export)
        .await
        .expect("original validated export should still restore");
}

#[tokio::test]
async fn restore_rejects_malformed_serialized_and_audit_metadata_before_writing() {
    let pool = test_pool().await;
    let export = persisted_export(&pool).await;
    let campaign_id = CampaignId(uuid::Uuid::parse_str(&export.campaign_id).unwrap());
    purge_campaign(
        &pool,
        campaign_id,
        &CampaignPurgeAuthorization::admin("prepare serialized restore integrity mutations"),
        &export,
    )
    .await
    .expect("valid exact backup should permit purge");

    let mut invalid_command_json = export.clone();
    invalid_command_json.command_audit[0].payload_json = "{".into();
    assert_restore_rejected_without_write(&pool, &invalid_command_json, "invalid command JSON")
        .await;

    let mut blank_command_kind = export.clone();
    blank_command_kind.command_audit[0].command_kind = "   ".into();
    assert_restore_rejected_without_write(&pool, &blank_command_kind, "blank command kind").await;

    let mut zero_command_version = export.clone();
    zero_command_version.command_audit[0].command_schema_version = 0;
    assert_restore_rejected_without_write(&pool, &zero_command_version, "zero command schema")
        .await;

    let mut overflow_command_version = export.clone();
    overflow_command_version.command_audit[0].command_schema_version = i64::from(u32::MAX) + 1;
    assert_restore_rejected_without_write(
        &pool,
        &overflow_command_version,
        "overflow command schema",
    )
    .await;

    let mut empty_resolution = export.clone();
    empty_resolution.command_audit[0].resolution_explanation = "   ".into();
    assert_restore_rejected_without_write(&pool, &empty_resolution, "empty resolution").await;

    let mut invalid_event_json = export.clone();
    invalid_event_json.event_journal[0].payload_json = "{".into();
    assert_restore_rejected_without_write(&pool, &invalid_event_json, "invalid event JSON").await;

    let mut blank_event_kind = export.clone();
    blank_event_kind.event_journal[0].event_kind = "  ".into();
    assert_restore_rejected_without_write(&pool, &blank_event_kind, "blank event kind").await;

    let mut zero_event_version = export.clone();
    zero_event_version.event_journal[0].event_schema_version = 0;
    assert_restore_rejected_without_write(&pool, &zero_event_version, "zero event schema").await;

    let mut overflow_event_version = export.clone();
    overflow_event_version.event_journal[0].event_schema_version = i64::from(u32::MAX) + 1;
    assert_restore_rejected_without_write(&pool, &overflow_event_version, "overflow event schema")
        .await;

    let mut rejected_origin = export.clone();
    rejected_origin.command_audit[0].accepted = 0;
    assert_restore_rejected_without_write(&pool, &rejected_origin, "event from rejected command")
        .await;

    let mut wrong_resulting_sequence = export.clone();
    wrong_resulting_sequence.command_audit[0].resulting_event_sequence += 1;
    assert_restore_rejected_without_write(
        &pool,
        &wrong_resulting_sequence,
        "command resulting sequence mismatch",
    )
    .await;

    let mut wrong_expected_sequence = export.clone();
    wrong_expected_sequence.command_audit[0].expected_event_sequence = 1;
    assert_restore_rejected_without_write(
        &pool,
        &wrong_expected_sequence,
        "command expected sequence mismatch",
    )
    .await;

    restore_campaign(&pool, &export)
        .await
        .expect("original replay-safe export should still restore");
    let projection = load_campaign_projection_summary(&pool, campaign_id)
        .await
        .expect("restore must rebuild derivative projections from authoritative state");
    assert_eq!(
        projection.applied_event_sequence,
        u64::try_from(export.current_state.applied_event_sequence).unwrap()
    );
}
