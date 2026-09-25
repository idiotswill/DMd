use dmd_domain::{
    AttendanceStatus, CURRENT_STATE_SCHEMA_VERSION, CampaignId, CampaignState, CommandIssuer,
    NewSessionObservation, ObservationAudience, ObservationId, PlaySessionId, PlaySessionStatus,
    SessionObservation,
};
use sqlx::{Row, SqliteConnection, SqlitePool, sqlite::SqliteRow};
use thiserror::Error;

use crate::session_store::load_play_session_on;

#[derive(Debug, Error)]
pub enum ObservationStoreError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Session(#[from] crate::SessionStoreError),
    #[error("observation metadata, payload or references are invalid")]
    InvalidObservation,
    #[error("observation identity already exists with different content")]
    ConflictingIdentity,
    #[error("observation requires an authorized attending player or host")]
    Unauthorized,
    #[error("observation ordinal exceeds durable storage limits")]
    OrdinalOverflow,
}

/// Append conversation without changing the authoritative state, command audit or event sequence.
/// Identical stable IDs return their original rows, including after a session closes.
pub async fn append_session_observations(
    pool: &SqlitePool,
    campaign_id: CampaignId,
    observations: &[NewSessionObservation],
) -> Result<Vec<SessionObservation>, ObservationStoreError> {
    // Reserve the writer before reading attendance/head and allocating ordinals.
    let mut transaction = pool.begin_with("BEGIN IMMEDIATE").await?;
    let result =
        append_session_observations_in_transaction(&mut transaction, campaign_id, observations)
            .await?;
    transaction.commit().await?;
    Ok(result)
}

/// The caller reserves the SQLite writer and rolls back on any error. Conversation
/// and its exact transport response can then commit without a second connection.
pub async fn append_session_observations_in_transaction(
    transaction: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    campaign_id: CampaignId,
    observations: &[NewSessionObservation],
) -> Result<Vec<SessionObservation>, ObservationStoreError> {
    if observations.is_empty() || observations.len() > 128 {
        return Err(ObservationStoreError::InvalidObservation);
    }
    let state_row = sqlx::query(
        "SELECT state_json, schema_version, applied_event_sequence FROM campaign_state_current WHERE campaign_id = ?",
    )
    .bind(campaign_id.0.to_string())
    .fetch_optional(&mut **transaction)
    .await?
    .ok_or(ObservationStoreError::InvalidObservation)?;
    let state = CampaignState::decode_json(&state_row.try_get::<String, _>("state_json")?)
        .map_err(|_| ObservationStoreError::InvalidObservation)?;
    if state.campaign_id() != campaign_id
        || !state.validate().is_empty()
        || state.schema_version != CURRENT_STATE_SCHEMA_VERSION
        || state_row.try_get::<i64, _>("schema_version")? != i64::from(CURRENT_STATE_SCHEMA_VERSION)
        || i64::try_from(state.applied_event_sequence).ok()
            != Some(state_row.try_get("applied_event_sequence")?)
    {
        return Err(ObservationStoreError::InvalidObservation);
    }
    crate::session_store::validate_table_session_projection(transaction, &state).await?;
    let mut ordinal: i64 = sqlx::query_scalar(
        "SELECT COALESCE(MAX(ordinal), 0) FROM session_observations WHERE campaign_id = ?",
    )
    .bind(campaign_id.0.to_string())
    .fetch_one(&mut **transaction)
    .await?;
    let mut result = Vec::with_capacity(observations.len());
    for record in observations {
        if record.campaign_id != campaign_id || !record.valid_shape() {
            return Err(ObservationStoreError::InvalidObservation);
        }
        if let Some(row) = sqlx::query("SELECT * FROM session_observations WHERE id = ?")
            .bind(record.id.0.to_string())
            .fetch_optional(&mut **transaction)
            .await?
        {
            let existing = decode_observation(row)?;
            if existing.record != *record {
                return Err(ObservationStoreError::ConflictingIdentity);
            }
            result.push(existing);
            continue;
        }
        validate_new_observation(transaction, &state, record).await?;
        ordinal = ordinal
            .checked_add(1)
            .ok_or(ObservationStoreError::OrdinalOverflow)?;
        let observation = SessionObservation {
            ordinal: u64::try_from(ordinal).map_err(|_| ObservationStoreError::OrdinalOverflow)?,
            record: record.clone(),
        };
        insert_observation(transaction, &observation).await?;
        result.push(observation);
    }
    Ok(result)
}

async fn validate_new_observation(
    connection: &mut SqliteConnection,
    state: &CampaignState,
    record: &NewSessionObservation,
) -> Result<(), ObservationStoreError> {
    if record.observed_event_sequence > state.applied_event_sequence {
        return Err(ObservationStoreError::InvalidObservation);
    }
    let session = match record.session_id {
        Some(id) => {
            let session = load_play_session_on(connection, id)
                .await?
                .ok_or(ObservationStoreError::InvalidObservation)?;
            if session.campaign_id != record.campaign_id
                || session.status != PlaySessionStatus::Active
                || !session.validate_against_state(state).is_empty()
            {
                return Err(ObservationStoreError::InvalidObservation);
            }
            Some(session)
        }
        None => None,
    };
    if let ObservationAudience::Player(id) = record.audience
        && (!state.players.contains_key(&id)
            || session.as_ref().is_some_and(|s| {
                !s.participants
                    .iter()
                    .any(|p| p.player_id == id && p.attendance == AttendanceStatus::Present)
            }))
    {
        return Err(ObservationStoreError::InvalidObservation);
    }
    match record.issuer {
        CommandIssuer::System | CommandIssuer::Admin => Ok(()),
        CommandIssuer::Player(id) if state.players.contains_key(&id) => {
            let attending = session.as_ref().is_some_and(|s| {
                s.participants
                    .iter()
                    .any(|p| p.player_id == id && p.attendance == AttendanceStatus::Present)
            });
            if !attending
                || matches!(record.audience, ObservationAudience::Player(other) if id != other)
            {
                return Err(ObservationStoreError::Unauthorized);
            }
            Ok(())
        }
        _ => Err(ObservationStoreError::Unauthorized),
    }
}

/// Trusted raw ledger read. The application must apply audience filtering before display.
pub async fn load_session_observations(
    pool: &SqlitePool,
    campaign_id: CampaignId,
    session_id: Option<PlaySessionId>,
    after_ordinal: u64,
    limit: u32,
) -> Result<Vec<SessionObservation>, ObservationStoreError> {
    if !(1..=1000).contains(&limit) {
        return Err(ObservationStoreError::InvalidObservation);
    }
    let after = i64::try_from(after_ordinal).map_err(|_| ObservationStoreError::OrdinalOverflow)?;
    let rows = sqlx::query(
        "SELECT * FROM session_observations WHERE campaign_id = ? AND session_id IS ? \
         AND ordinal > ? ORDER BY ordinal LIMIT ?",
    )
    .bind(campaign_id.0.to_string())
    .bind(session_id.map(|id| id.0.to_string()))
    .bind(after)
    .bind(i64::from(limit))
    .fetch_all(pool)
    .await?;
    rows.into_iter().map(decode_observation).collect()
}

/// Trusted campaign history across session boundaries; callers must filter audiences.
pub async fn load_campaign_observations(
    pool: &SqlitePool,
    campaign_id: CampaignId,
    after_ordinal: u64,
    limit: u32,
) -> Result<Vec<SessionObservation>, ObservationStoreError> {
    if !(1..=1000).contains(&limit) {
        return Err(ObservationStoreError::InvalidObservation);
    }
    let after = i64::try_from(after_ordinal).map_err(|_| ObservationStoreError::OrdinalOverflow)?;
    sqlx::query("SELECT * FROM session_observations WHERE campaign_id = ? AND ordinal > ? ORDER BY ordinal LIMIT ?")
        .bind(campaign_id.0.to_string()).bind(after).bind(i64::from(limit))
        .fetch_all(pool).await?.into_iter().map(decode_observation).collect()
}

/// Trusted retry lookup scoped by campaign; the application must compare the original request.
pub async fn load_session_observation(
    pool: &SqlitePool,
    campaign_id: CampaignId,
    id: ObservationId,
) -> Result<Option<SessionObservation>, ObservationStoreError> {
    sqlx::query("SELECT * FROM session_observations WHERE campaign_id = ? AND id = ?")
        .bind(campaign_id.0.to_string())
        .bind(id.0.to_string())
        .fetch_optional(pool)
        .await?
        .map(decode_observation)
        .transpose()
}

pub(crate) async fn insert_observation(
    connection: &mut SqliteConnection,
    observation: &SessionObservation,
) -> Result<(), ObservationStoreError> {
    let record = &observation.record;
    let ordinal =
        i64::try_from(observation.ordinal).map_err(|_| ObservationStoreError::OrdinalOverflow)?;
    let head = i64::try_from(record.observed_event_sequence)
        .map_err(|_| ObservationStoreError::InvalidObservation)?;
    let json =
        serde_json::to_string(record).map_err(|_| ObservationStoreError::InvalidObservation)?;
    sqlx::query(
        "INSERT INTO session_observations (id, campaign_id, ordinal, session_id, observed_event_sequence, record_json) \
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(record.id.0.to_string())
    .bind(record.campaign_id.0.to_string())
    .bind(ordinal)
    .bind(record.session_id.map(|id| id.0.to_string()))
    .bind(head)
    .bind(json)
    .execute(&mut *connection)
    .await?;
    Ok(())
}

pub(crate) fn decode_observation(
    row: SqliteRow,
) -> Result<SessionObservation, ObservationStoreError> {
    let json: String = row.try_get("record_json")?;
    let record: NewSessionObservation =
        serde_json::from_str(&json).map_err(|_| ObservationStoreError::InvalidObservation)?;
    let ordinal = u64::try_from(row.try_get::<i64, _>("ordinal")?)
        .map_err(|_| ObservationStoreError::InvalidObservation)?;
    if ordinal == 0
        || !record.valid_shape()
        || record.id.0.to_string() != row.try_get::<String, _>("id")?
        || record.campaign_id.0.to_string() != row.try_get::<String, _>("campaign_id")?
        || record.session_id.map(|id| id.0.to_string())
            != row.try_get::<Option<String>, _>("session_id")?
        || i64::try_from(record.observed_event_sequence).ok()
            != Some(row.try_get("observed_event_sequence")?)
    {
        return Err(ObservationStoreError::InvalidObservation);
    }
    Ok(SessionObservation { ordinal, record })
}
