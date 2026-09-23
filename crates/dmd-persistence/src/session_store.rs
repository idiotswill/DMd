use dmd_domain::{
    AttendanceStatus, CampaignId, CharacterId, PlaySession, PlaySessionId, PlaySessionStatus,
    PlayerId, SessionParticipant, WorldInstant,
};
use sqlx::{Row, SqlitePool};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum SessionStoreError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error("play session violates domain shape invariants")]
    InvalidShape,
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

pub async fn ensure_session_schema(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS play_sessions (
            id TEXT PRIMARY KEY NOT NULL,
            campaign_id TEXT NOT NULL,
            display_name TEXT NOT NULL,
            status TEXT NOT NULL CHECK (status IN ('active', 'closed')),
            started_at_world INTEGER NOT NULL,
            ended_at_world INTEGER NULL
        )
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE UNIQUE INDEX IF NOT EXISTS ux_play_sessions_one_active_per_campaign
        ON play_sessions(campaign_id)
        WHERE status = 'active'
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS play_session_participants (
            session_id TEXT NOT NULL,
            ordinal INTEGER NOT NULL,
            player_id TEXT NOT NULL,
            character_id TEXT NULL,
            attendance TEXT NOT NULL CHECK (attendance IN ('present', 'absent')),
            PRIMARY KEY (session_id, player_id),
            FOREIGN KEY (session_id) REFERENCES play_sessions(id) ON DELETE CASCADE
        )
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE UNIQUE INDEX IF NOT EXISTS ux_play_session_character_assignment
        ON play_session_participants(session_id, character_id)
        WHERE character_id IS NOT NULL
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn save_play_session(
    pool: &SqlitePool,
    session: &PlaySession,
) -> Result<(), SessionStoreError> {
    if !session.validate_shape().is_empty() {
        return Err(SessionStoreError::InvalidShape);
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
        sqlx::query(
            r#"
            INSERT INTO play_session_participants (
                session_id, ordinal, player_id, character_id, attendance
            ) VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(session.id.0.to_string())
        .bind(i64::try_from(ordinal).expect("participant ordinal must fit in i64"))
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
    use sqlx::sqlite::SqlitePoolOptions;

    async fn test_pool() -> SqlitePool {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("in-memory SQLite should open");
        ensure_session_schema(&pool)
            .await
            .expect("session schema should initialize");
        pool
    }

    fn active_session(campaign_id: CampaignId, label: &str) -> PlaySession {
        PlaySession {
            id: PlaySessionId::new(),
            campaign_id,
            display_name: label.into(),
            status: PlaySessionStatus::Active,
            started_at_world: WorldInstant(100),
            ended_at_world: None,
            participants: vec![
                SessionParticipant {
                    player_id: PlayerId::new(),
                    character_id: Some(CharacterId::new()),
                    attendance: AttendanceStatus::Present,
                },
                SessionParticipant {
                    player_id: PlayerId::new(),
                    character_id: None,
                    attendance: AttendanceStatus::Absent,
                },
            ],
        }
    }

    #[tokio::test]
    async fn session_round_trips_through_sqlite() {
        let pool = test_pool().await;
        let session = active_session(CampaignId::new(), "Session 1");

        save_play_session(&pool, &session)
            .await
            .expect("session should save");
        let loaded = load_play_session(&pool, session.id)
            .await
            .expect("session should load")
            .expect("session should exist");

        assert_eq!(loaded, session);
    }

    #[tokio::test]
    async fn only_one_active_session_is_allowed_per_campaign() {
        let pool = test_pool().await;
        let campaign_id = CampaignId::new();
        let first = active_session(campaign_id, "First");
        let second = active_session(campaign_id, "Second");

        save_play_session(&pool, &first)
            .await
            .expect("first active session should save");
        assert!(save_play_session(&pool, &second).await.is_err());
    }

    #[tokio::test]
    async fn closing_a_session_allows_the_next_one_to_start() {
        let pool = test_pool().await;
        let campaign_id = CampaignId::new();
        let mut first = active_session(campaign_id, "First");
        save_play_session(&pool, &first)
            .await
            .expect("first active session should save");

        first.status = PlaySessionStatus::Closed;
        first.ended_at_world = Some(WorldInstant(150));
        save_play_session(&pool, &first)
            .await
            .expect("first session should close");

        let second = active_session(campaign_id, "Second");
        save_play_session(&pool, &second)
            .await
            .expect("next active session should save");

        let loaded = load_active_play_session(&pool, campaign_id)
            .await
            .expect("active session lookup should succeed")
            .expect("second session should be active");
        assert_eq!(loaded.id, second.id);
    }
}
