use dmd_domain::{
    AttendanceStatus, CampaignId, CampaignState, CharacterId, PlaySession, PlaySessionId,
    PlaySessionStatus, PlayerId, SessionParticipant, WorldInstant,
};
use sqlx::{Row, SqlitePool};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum SessionStoreError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error("cannot persist a play session against invalid world state")]
    InvalidWorldState,
    #[error("play session violates shape or world-reference invariants")]
    InvalidSession,
    #[error("play session has too many participants to persist an ordinal")]
    ParticipantOrdinalOverflow,
    #[error("invalid UUID stored in {field}: {source}")]
    InvalidUuid {
        field: &'static str,
        #[source]
        source: uuid::Error,
    },
    #[error("invalid stored play session status: {0}")]
    InvalidSessionStatus(String),
    #[error("invalid stored attendance status: {0}")]
    InvalidAttendanceStatus(String),
}

pub async fn save_play_session(
    pool: &SqlitePool,
    state: &CampaignState,
    session: &PlaySession,
) -> Result<(), SessionStoreError> {
    if !state.validate().is_empty() {
        return Err(SessionStoreError::InvalidWorldState);
    }
    if !session.validate_against_state(state).is_empty() {
        return Err(SessionStoreError::InvalidSession);
    }

    let mut transaction = pool.begin().await?;
    sqlx::query(
        r#"
        INSERT INTO play_sessions (
            id, campaign_id, display_name, status, started_at_world, ended_at_world
        ) VALUES (?, ?, ?, ?, ?, ?)
        ON CONFLICT(id) DO UPDATE SET
            campaign_id = excluded.campaign_id,
            display_name = excluded.display_name,
            status = excluded.status,
            started_at_world = excluded.started_at_world,
            ended_at_world = excluded.ended_at_world
        "#,
    )
    .bind(session.id.0.to_string())
    .bind(session.campaign_id.0.to_string())
    .bind(&session.display_name)
    .bind(encode_session_status(session.status))
    .bind(session.started_at_world.0)
    .bind(session.ended_at_world.map(|instant| instant.0))
    .execute(&mut *transaction)
    .await?;

    sqlx::query("DELETE FROM play_session_participants WHERE session_id = ?")
        .bind(session.id.0.to_string())
        .execute(&mut *transaction)
        .await?;

    for (ordinal, participant) in session.participants.iter().enumerate() {
        let ordinal = i64::try_from(ordinal)
            .map_err(|_| SessionStoreError::ParticipantOrdinalOverflow)?;
        sqlx::query(
            r#"
            INSERT INTO play_session_participants (
                session_id, ordinal, player_id, character_id, attendance
            ) VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(session.id.0.to_string())
        .bind(ordinal)
        .bind(participant.player_id.0.to_string())
        .bind(participant.character_id.map(|id| id.0.to_string()))
        .bind(encode_attendance(participant.attendance))
        .execute(&mut *transaction)
        .await?;
    }

    transaction.commit().await?;
    Ok(())
}

pub async fn load_play_session(
    pool: &SqlitePool,
    session_id: PlaySessionId,
) -> Result<Option<PlaySession>, SessionStoreError> {
    let Some(row) = sqlx::query(
        r#"
        SELECT id, campaign_id, display_name, status, started_at_world, ended_at_world
        FROM play_sessions
        WHERE id = ?
        "#,
    )
    .bind(session_id.0.to_string())
    .fetch_optional(pool)
    .await?
    else {
        return Ok(None);
    };

    let participant_rows = sqlx::query(
        r#"
        SELECT player_id, character_id, attendance
        FROM play_session_participants
        WHERE session_id = ?
        ORDER BY ordinal ASC
        "#,
    )
    .bind(session_id.0.to_string())
    .fetch_all(pool)
    .await?;

    let mut participants = Vec::with_capacity(participant_rows.len());
    for participant_row in participant_rows {
        let player_id: String = participant_row.try_get("player_id")?;
        let character_id: Option<String> = participant_row.try_get("character_id")?;
        let attendance: String = participant_row.try_get("attendance")?;
        participants.push(SessionParticipant {
            player_id: PlayerId(parse_uuid(&player_id, "player_id")?),
            character_id: character_id
                .as_deref()
                .map(|value| parse_uuid(value, "character_id").map(CharacterId))
                .transpose()?,
            attendance: decode_attendance(&attendance)?,
        });
    }

    let id: String = row.try_get("id")?;
    let campaign_id: String = row.try_get("campaign_id")?;
    let status: String = row.try_get("status")?;
    let started_at_world: i64 = row.try_get("started_at_world")?;
    let ended_at_world: Option<i64> = row.try_get("ended_at_world")?;

    Ok(Some(PlaySession {
        id: PlaySessionId(parse_uuid(&id, "id")?),
        campaign_id: CampaignId(parse_uuid(&campaign_id, "campaign_id")?),
        display_name: row.try_get("display_name")?,
        status: decode_session_status(&status)?,
        started_at_world: WorldInstant(started_at_world),
        ended_at_world: ended_at_world.map(WorldInstant),
        participants,
    }))
}

pub async fn load_active_play_session(
    pool: &SqlitePool,
    campaign_id: CampaignId,
) -> Result<Option<PlaySession>, SessionStoreError> {
    let id = sqlx::query_scalar::<_, String>(
        "SELECT id FROM play_sessions WHERE campaign_id = ? AND status = 'active'",
    )
    .bind(campaign_id.0.to_string())
    .fetch_optional(pool)
    .await?;

    match id {
        Some(id) => load_play_session(pool, PlaySessionId(parse_uuid(&id, "id")?)).await,
        None => Ok(None),
    }
}

fn encode_session_status(status: PlaySessionStatus) -> &'static str {
    match status {
        PlaySessionStatus::Active => "active",
        PlaySessionStatus::Closed => "closed",
    }
}

fn decode_session_status(value: &str) -> Result<PlaySessionStatus, SessionStoreError> {
    match value {
        "active" => Ok(PlaySessionStatus::Active),
        "closed" => Ok(PlaySessionStatus::Closed),
        other => Err(SessionStoreError::InvalidSessionStatus(other.into())),
    }
}

fn encode_attendance(status: AttendanceStatus) -> &'static str {
    match status {
        AttendanceStatus::Present => "present",
        AttendanceStatus::Absent => "absent",
    }
}

fn decode_attendance(value: &str) -> Result<AttendanceStatus, SessionStoreError> {
    match value {
        "present" => Ok(AttendanceStatus::Present),
        "absent" => Ok(AttendanceStatus::Absent),
        other => Err(SessionStoreError::InvalidAttendanceStatus(other.into())),
    }
}

fn parse_uuid(value: &str, field: &'static str) -> Result<Uuid, SessionStoreError> {
    Uuid::parse_str(value).map_err(|source| SessionStoreError::InvalidUuid { field, source })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::migrate_sqlite;
    use dmd_domain::{
        Campaign, CampaignStatus, Character, CharacterStatus, EntityExistence, EntityId, EntityKind,
        Player, VersionedRef, WorldClock, WorldEntity,
    };
    use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
    use std::str::FromStr;

    async fn test_pool() -> SqlitePool {
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
            .expect("session migration should initialize");
        pool
    }

    fn empty_state() -> CampaignState {
        let campaign_id = CampaignId::new();
        CampaignState::empty(
            Campaign {
                id: campaign_id,
                display_name: "Persistence Test Campaign".into(),
                status: CampaignStatus::Active,
                world_seed: 99,
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

    fn add_participant(state: &mut CampaignState, with_character: bool) -> SessionParticipant {
        let player = Player {
            id: PlayerId::new(),
            campaign_id: state.campaign_id(),
            display_name: "Test Player".into(),
        };
        state.players.insert(player.id, player.clone());

        let character_id = if with_character {
            let entity = WorldEntity {
                id: EntityId::new(),
                campaign_id: state.campaign_id(),
                display_name: "Test Character".into(),
                kind: EntityKind::Character,
                existence: EntityExistence::Present,
                location_id: None,
            };
            state.entities.insert(entity.id, entity.clone());
            let character = Character {
                id: CharacterId::new(),
                entity_id: entity.id,
                campaign_id: state.campaign_id(),
                controlling_player_id: Some(player.id),
                display_name: "Test Character".into(),
                status: CharacterStatus::Active,
            };
            state.characters.insert(character.id, character.clone());
            Some(character.id)
        } else {
            None
        };

        SessionParticipant {
            player_id: player.id,
            character_id,
            attendance: if with_character {
                AttendanceStatus::Present
            } else {
                AttendanceStatus::Absent
            },
        }
    }

    fn active_session(state: &mut CampaignState, label: &str) -> PlaySession {
        let present = add_participant(state, true);
        let absent = add_participant(state, false);
        PlaySession {
            id: PlaySessionId::new(),
            campaign_id: state.campaign_id(),
            display_name: label.into(),
            status: PlaySessionStatus::Active,
            started_at_world: state.clock.now,
            ended_at_world: None,
            participants: vec![present, absent],
        }
    }

    #[tokio::test]
    async fn session_round_trips_through_sqlite() {
        let pool = test_pool().await;
        let mut state = empty_state();
        let session = active_session(&mut state, "Session 1");

        save_play_session(&pool, &state, &session)
            .await
            .expect("session should save");
        let loaded = load_play_session(&pool, session.id)
            .await
            .expect("session should load")
            .expect("session should exist");

        assert_eq!(loaded, session);
    }

    #[tokio::test]
    async fn invalid_session_references_are_rejected_before_sql() {
        let pool = test_pool().await;
        let state = empty_state();
        let session = PlaySession {
            id: PlaySessionId::new(),
            campaign_id: state.campaign_id(),
            display_name: "Invalid".into(),
            status: PlaySessionStatus::Active,
            started_at_world: state.clock.now,
            ended_at_world: None,
            participants: vec![SessionParticipant {
                player_id: PlayerId::new(),
                character_id: Some(CharacterId::new()),
                attendance: AttendanceStatus::Present,
            }],
        };

        assert!(matches!(
            save_play_session(&pool, &state, &session).await,
            Err(SessionStoreError::InvalidSession)
        ));
    }

    #[tokio::test]
    async fn only_one_active_session_is_allowed_per_campaign() {
        let pool = test_pool().await;
        let mut state = empty_state();
        let first = active_session(&mut state, "First");
        let second = active_session(&mut state, "Second");

        save_play_session(&pool, &state, &first)
            .await
            .expect("first active session should save");
        assert!(save_play_session(&pool, &state, &second).await.is_err());
    }

    #[tokio::test]
    async fn closing_a_session_allows_the_next_one_to_start() {
        let pool = test_pool().await;
        let mut state = empty_state();
        let mut first = active_session(&mut state, "First");
        save_play_session(&pool, &state, &first)
            .await
            .expect("first active session should save");

        first.status = PlaySessionStatus::Closed;
        first.ended_at_world = Some(WorldInstant(150));
        save_play_session(&pool, &state, &first)
            .await
            .expect("first session should close");

        let second = active_session(&mut state, "Second");
        save_play_session(&pool, &state, &second)
            .await
            .expect("next active session should save");

        let loaded = load_active_play_session(&pool, state.campaign_id())
            .await
            .expect("active session lookup should succeed")
            .expect("second session should be active");
        assert_eq!(loaded.id, second.id);
    }
}
