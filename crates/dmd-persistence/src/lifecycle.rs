use std::collections::{HashMap, HashSet};

use dmd_domain::{
    AttendanceStatus, BeliefBasis, CURRENT_STATE_SCHEMA_VERSION, CampaignId, CampaignState,
    CharacterId, CommandIssuer, EntityId, EventId, FactionId, PlaySession, PlaySessionId,
    PlaySessionStatus, PlayerId, SessionParticipant, WorldInstant,
};
use serde::{Deserialize, Serialize};
use sqlx::{Row, Sqlite, SqlitePool, Transaction, sqlite::SqliteRow};
use thiserror::Error;

use crate::{
    CampaignStateSnapshotCodec, JournalStoreError, initialize_campaign_state, load_campaign_state,
    snapshot_replay::upgrade_state_schema_one,
};

pub const CAMPAIGN_EXPORT_FORMAT_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CampaignStorageStatus {
    Active,
    Archived,
}

impl CampaignStorageStatus {
    fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Archived => "archived",
        }
    }

    fn parse(value: &str) -> Result<Self, LifecycleError> {
        match value {
            "active" => Ok(Self::Active),
            "archived" => Ok(Self::Archived),
            other => Err(LifecycleError::CorruptLifecycle(format!(
                "invalid storage status {other}"
            ))),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CampaignLifecycleSummary {
    pub campaign_id: String,
    pub display_name: String,
    pub storage_status: CampaignStorageStatus,
    pub state_schema_version: u32,
    pub created_at_utc: String,
    pub archived_at_utc: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenCampaign {
    pub lifecycle: CampaignLifecycleSummary,
    pub state: CampaignState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CampaignPurgeAuthorization {
    pub issuer: CommandIssuer,
    pub reason: String,
}

impl CampaignPurgeAuthorization {
    pub fn admin(reason: impl Into<String>) -> Self {
        Self {
            issuer: CommandIssuer::Admin,
            reason: reason.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CampaignExport {
    pub format_version: u32,
    pub state_schema_version: u32,
    pub campaign_id: String,
    pub exported_at_utc: String,
    pub lifecycle: CampaignLifecycleSummary,
    pub current_state: CurrentStateRow,
    pub play_sessions: Vec<PlaySessionRow>,
    pub play_session_participants: Vec<PlaySessionParticipantRow>,
    pub command_audit: Vec<CommandAuditRow>,
    pub event_journal: Vec<EventJournalRow>,
    pub event_causes: Vec<EventCauseRow>,
    pub snapshots: Vec<SnapshotRow>,
}

impl CampaignExport {
    pub fn to_json(&self) -> Result<String, LifecycleError> {
        serde_json::to_string_pretty(self)
            .map_err(|error| LifecycleError::ExportEncoding(error.to_string()))
    }

    pub fn from_json(value: &str) -> Result<Self, LifecycleError> {
        serde_json::from_str(value)
            .map_err(|error| LifecycleError::ExportEncoding(error.to_string()))
    }

    /// Validate and upgrade a portable export without mutating the source or its immutable history.
    ///
    /// Application restore preflight must use this before resolving the exported campaign's
    /// content. Schema-1 current state gains only an explicit null rules field. Original snapshot
    /// bytes, journal records, audit records, and their metadata remain unchanged.
    pub fn upgraded(&self) -> Result<Self, LifecycleError> {
        if self.state_schema_version == CURRENT_STATE_SCHEMA_VERSION {
            validate_export(self)?;
            return Ok(self.clone());
        }
        if self.state_schema_version != 1 {
            return Err(LifecycleError::IncompatibleStateSchema {
                actual: self.state_schema_version,
                supported: CURRENT_STATE_SCHEMA_VERSION,
            });
        }
        validate_export_metadata_version(self, 1)?;
        if self
            .snapshots
            .iter()
            .any(|snapshot| snapshot.state_schema_version > 1)
        {
            return Err(LifecycleError::CorruptExport(
                "schema-1 export contains a newer snapshot".into(),
            ));
        }
        let mut upgraded = self.clone();
        upgraded.current_state.state_json =
            upgrade_state_schema_one(&self.current_state.state_json).map_err(|message| {
                LifecycleError::CorruptExport(format!("schema-1 state: {message}"))
            })?;
        upgraded.state_schema_version = CURRENT_STATE_SCHEMA_VERSION;
        upgraded.current_state.schema_version = i64::from(CURRENT_STATE_SCHEMA_VERSION);
        upgraded.lifecycle.state_schema_version = CURRENT_STATE_SCHEMA_VERSION;
        validate_export(&upgraded)?;
        Ok(upgraded)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CurrentStateRow {
    pub campaign_id: String,
    pub schema_version: i64,
    pub applied_event_sequence: i64,
    pub state_json: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlaySessionRow {
    pub id: String,
    pub campaign_id: String,
    pub display_name: String,
    pub status: String,
    pub started_at_world: i64,
    pub ended_at_world: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlaySessionParticipantRow {
    pub session_id: String,
    pub ordinal: i64,
    pub player_id: String,
    pub character_id: Option<String>,
    pub attendance: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandAuditRow {
    pub id: String,
    pub campaign_id: String,
    pub session_id: Option<String>,
    pub issuer_kind: String,
    pub issuer_player_id: Option<String>,
    pub actor_kind: Option<String>,
    pub actor_id: Option<String>,
    pub expected_event_sequence: i64,
    pub command_kind: String,
    pub command_schema_version: i64,
    pub payload_json: String,
    pub accepted: i64,
    pub resolution_explanation: String,
    pub resulting_event_sequence: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventJournalRow {
    pub id: String,
    pub campaign_id: String,
    pub sequence: i64,
    pub session_id: Option<String>,
    pub occurred_at_world: i64,
    pub source: String,
    pub actor_kind: Option<String>,
    pub actor_id: Option<String>,
    pub command_id: String,
    pub event_kind: String,
    pub event_schema_version: i64,
    pub payload_json: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventCauseRow {
    pub campaign_id: String,
    pub event_id: String,
    pub cause_event_id: String,
    pub ordinal: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotRow {
    pub campaign_id: String,
    pub event_sequence: i64,
    pub state_schema_version: i64,
    pub state_json: String,
    pub created_at_utc: String,
}

#[derive(Debug, Error)]
pub enum LifecycleError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Journal(#[from] JournalStoreError),
    #[error("campaign does not exist")]
    CampaignNotFound,
    #[error("campaign already exists")]
    CampaignAlreadyExists,
    #[error("campaign lifecycle metadata is corrupt: {0}")]
    CorruptLifecycle(String),
    #[error("campaign export is corrupt: {0}")]
    CorruptExport(String),
    #[error("campaign export format {actual} is incompatible; supported format is {supported}")]
    IncompatibleExportFormat { actual: u32, supported: u32 },
    #[error("campaign state schema {actual} is incompatible; supported schema is {supported}")]
    IncompatibleStateSchema { actual: u32, supported: u32 },
    #[error("campaign export JSON could not be encoded/decoded: {0}")]
    ExportEncoding(String),
    #[error("whole-campaign purge requires an admin issuer")]
    PurgeRequiresAdmin,
    #[error("whole-campaign purge reason must not be empty")]
    EmptyPurgeReason,
    #[error("campaign changed after backup export; purge aborted")]
    StalePurgeBackup,
}

pub async fn create_campaign(
    pool: &SqlitePool,
    state: &CampaignState,
) -> Result<OpenCampaign, LifecycleError> {
    initialize_campaign_state(pool, state)
        .await
        .map_err(|error| match error {
            JournalStoreError::StateAlreadyInitialized => LifecycleError::CampaignAlreadyExists,
            other => LifecycleError::Journal(other),
        })?;
    open_campaign(pool, state.campaign_id()).await
}

pub async fn open_campaign(
    pool: &SqlitePool,
    campaign_id: CampaignId,
) -> Result<OpenCampaign, LifecycleError> {
    let state = load_campaign_state(pool, campaign_id)
        .await?
        .ok_or(LifecycleError::CampaignNotFound)?;
    let lifecycle = load_lifecycle(pool, campaign_id).await?;
    Ok(OpenCampaign { lifecycle, state })
}

pub async fn list_campaigns(
    pool: &SqlitePool,
) -> Result<Vec<CampaignLifecycleSummary>, LifecycleError> {
    let rows = sqlx::query(
        "SELECT campaign_id, display_name, storage_status, state_schema_version, created_at_utc, archived_at_utc \
         FROM campaign_lifecycle ORDER BY created_at_utc, campaign_id",
    )
    .fetch_all(pool)
    .await?;
    rows.iter().map(decode_lifecycle_row).collect()
}

pub async fn archive_campaign(
    pool: &SqlitePool,
    campaign_id: CampaignId,
) -> Result<CampaignLifecycleSummary, LifecycleError> {
    let result = sqlx::query(
        "UPDATE campaign_lifecycle SET storage_status = 'archived', \
         archived_at_utc = COALESCE(archived_at_utc, CURRENT_TIMESTAMP) WHERE campaign_id = ?",
    )
    .bind(campaign_id.0.to_string())
    .execute(pool)
    .await?;
    if result.rows_affected() == 0 {
        return Err(LifecycleError::CampaignNotFound);
    }
    load_lifecycle(pool, campaign_id).await
}

pub async fn export_campaign(
    pool: &SqlitePool,
    campaign_id: CampaignId,
) -> Result<CampaignExport, LifecycleError> {
    let mut tx = pool.begin().await?;
    let export = export_campaign_in_transaction(&mut tx, campaign_id).await?;
    tx.commit().await?;
    Ok(export)
}

async fn export_campaign_in_transaction(
    tx: &mut Transaction<'_, Sqlite>,
    campaign_id: CampaignId,
) -> Result<CampaignExport, LifecycleError> {
    let campaign_id_text = campaign_id.0.to_string();

    let current = sqlx::query(
        "SELECT campaign_id, schema_version, applied_event_sequence, state_json \
         FROM campaign_state_current WHERE campaign_id = ?",
    )
    .bind(&campaign_id_text)
    .fetch_optional(&mut **tx)
    .await?
    .ok_or(LifecycleError::CampaignNotFound)?;
    let current_state = CurrentStateRow {
        campaign_id: current.try_get("campaign_id")?,
        schema_version: current.try_get("schema_version")?,
        applied_event_sequence: current.try_get("applied_event_sequence")?,
        state_json: current.try_get("state_json")?,
    };

    let lifecycle_row = sqlx::query(
        "SELECT campaign_id, display_name, storage_status, state_schema_version, created_at_utc, archived_at_utc \
         FROM campaign_lifecycle WHERE campaign_id = ?",
    )
    .bind(&campaign_id_text)
    .fetch_optional(&mut **tx)
    .await?
    .ok_or_else(|| LifecycleError::CorruptLifecycle("missing lifecycle row".into()))?;
    let lifecycle = decode_lifecycle_row(&lifecycle_row)?;

    let play_sessions = sqlx::query(
        "SELECT id, campaign_id, display_name, status, started_at_world, ended_at_world \
         FROM play_sessions WHERE campaign_id = ? ORDER BY started_at_world, id",
    )
    .bind(&campaign_id_text)
    .fetch_all(&mut **tx)
    .await?
    .into_iter()
    .map(decode_play_session_row)
    .collect::<Result<Vec<_>, _>>()?;

    let play_session_participants = sqlx::query(
        "SELECT p.session_id, p.ordinal, p.player_id, p.character_id, p.attendance \
         FROM play_session_participants p JOIN play_sessions s ON s.id = p.session_id \
         WHERE s.campaign_id = ? ORDER BY p.session_id, p.ordinal",
    )
    .bind(&campaign_id_text)
    .fetch_all(&mut **tx)
    .await?
    .into_iter()
    .map(decode_participant_row)
    .collect::<Result<Vec<_>, _>>()?;

    let command_audit = sqlx::query(
        "SELECT id, campaign_id, session_id, issuer_kind, issuer_player_id, actor_kind, actor_id, \
         expected_event_sequence, command_kind, command_schema_version, payload_json, accepted, \
         resolution_explanation, resulting_event_sequence FROM command_audit \
         WHERE campaign_id = ? ORDER BY resulting_event_sequence, id",
    )
    .bind(&campaign_id_text)
    .fetch_all(&mut **tx)
    .await?
    .into_iter()
    .map(decode_command_audit_row)
    .collect::<Result<Vec<_>, _>>()?;

    let event_journal = sqlx::query(
        "SELECT id, campaign_id, sequence, session_id, occurred_at_world, source, actor_kind, actor_id, \
         command_id, event_kind, event_schema_version, payload_json FROM event_journal \
         WHERE campaign_id = ? ORDER BY sequence",
    )
    .bind(&campaign_id_text)
    .fetch_all(&mut **tx)
    .await?
    .into_iter()
    .map(decode_event_journal_row)
    .collect::<Result<Vec<_>, _>>()?;

    let event_causes = sqlx::query(
        "SELECT campaign_id, event_id, cause_event_id, ordinal FROM event_causes \
         WHERE campaign_id = ? ORDER BY event_id, ordinal",
    )
    .bind(&campaign_id_text)
    .fetch_all(&mut **tx)
    .await?
    .into_iter()
    .map(decode_event_cause_row)
    .collect::<Result<Vec<_>, _>>()?;

    let snapshots = sqlx::query(
        "SELECT campaign_id, event_sequence, state_schema_version, state_json, created_at_utc \
         FROM campaign_snapshots WHERE campaign_id = ? ORDER BY event_sequence",
    )
    .bind(&campaign_id_text)
    .fetch_all(&mut **tx)
    .await?
    .into_iter()
    .map(decode_snapshot_row)
    .collect::<Result<Vec<_>, _>>()?;

    let exported_at_utc: String = sqlx::query_scalar("SELECT CURRENT_TIMESTAMP")
        .fetch_one(&mut **tx)
        .await?;
    let export = CampaignExport {
        format_version: CAMPAIGN_EXPORT_FORMAT_VERSION,
        state_schema_version: u32::try_from(current_state.schema_version)
            .map_err(|_| LifecycleError::CorruptExport("invalid state schema version".into()))?,
        campaign_id: campaign_id_text,
        exported_at_utc,
        lifecycle,
        current_state,
        play_sessions,
        play_session_participants,
        command_audit,
        event_journal,
        event_causes,
        snapshots,
    };
    validate_export(&export)?;
    Ok(export)
}

pub async fn purge_campaign(
    pool: &SqlitePool,
    campaign_id: CampaignId,
    authorization: &CampaignPurgeAuthorization,
    backup: &CampaignExport,
) -> Result<(), LifecycleError> {
    if authorization.issuer != CommandIssuer::Admin {
        return Err(LifecycleError::PurgeRequiresAdmin);
    }
    if authorization.reason.trim().is_empty() {
        return Err(LifecycleError::EmptyPurgeReason);
    }
    validate_export(backup)?;

    let campaign_id_text = campaign_id.0.to_string();
    if backup.campaign_id != campaign_id_text {
        return Err(LifecycleError::StalePurgeBackup);
    }

    let mut tx = pool.begin().await?;
    // This write acquires SQLite's write reservation before the comparison export is read, so no
    // campaign-owned writer can change sessions, lifecycle metadata, or accepted history between
    // backup verification and whole-aggregate deletion.
    sqlx::query(
        "INSERT INTO campaign_purge_authorizations (campaign_id, authorized_by, reason) \
         VALUES (?, 'admin', ?)",
    )
    .bind(&campaign_id_text)
    .bind(authorization.reason.trim())
    .execute(&mut *tx)
    .await?;

    let current = export_campaign_in_transaction(&mut tx, campaign_id).await?;
    if !same_export_payload(backup, &current) {
        tx.rollback().await?;
        return Err(LifecycleError::StalePurgeBackup);
    }

    let deleted = sqlx::query("DELETE FROM campaign_state_current WHERE campaign_id = ?")
        .bind(&campaign_id_text)
        .execute(&mut *tx)
        .await?;
    if deleted.rows_affected() != 1 {
        tx.rollback().await?;
        return Err(LifecycleError::CampaignNotFound);
    }
    tx.commit().await?;
    Ok(())
}

fn same_export_payload(left: &CampaignExport, right: &CampaignExport) -> bool {
    left.format_version == right.format_version
        && left.state_schema_version == right.state_schema_version
        && left.campaign_id == right.campaign_id
        && left.lifecycle == right.lifecycle
        && left.current_state == right.current_state
        && left.play_sessions == right.play_sessions
        && left.play_session_participants == right.play_session_participants
        && left.command_audit == right.command_audit
        && left.event_journal == right.event_journal
        && left.event_causes == right.event_causes
        && left.snapshots == right.snapshots
}

pub async fn restore_campaign(
    pool: &SqlitePool,
    export: &CampaignExport,
) -> Result<OpenCampaign, LifecycleError> {
    let upgraded = export.upgraded()?;
    let export = &upgraded;
    let mut tx = pool.begin().await?;
    let exists =
        sqlx::query_scalar::<_, i64>("SELECT 1 FROM campaign_state_current WHERE campaign_id = ?")
            .bind(&export.campaign_id)
            .fetch_optional(&mut *tx)
            .await?;
    if exists.is_some() {
        tx.rollback().await?;
        return Err(LifecycleError::CampaignAlreadyExists);
    }

    sqlx::query(
        "INSERT INTO campaign_restore_authorizations (campaign_id, authorized_by) VALUES (?, 'import')",
    )
    .bind(&export.campaign_id)
    .execute(&mut *tx)
    .await?;
    sqlx::query(
        "INSERT INTO campaign_state_current \
         (campaign_id, schema_version, applied_event_sequence, state_json) VALUES (?, ?, ?, ?)",
    )
    .bind(&export.current_state.campaign_id)
    .bind(export.current_state.schema_version)
    .bind(export.current_state.applied_event_sequence)
    .bind(&export.current_state.state_json)
    .execute(&mut *tx)
    .await?;
    sqlx::query(
        "UPDATE campaign_lifecycle SET display_name = ?, storage_status = ?, state_schema_version = ?, \
         created_at_utc = ?, archived_at_utc = ? WHERE campaign_id = ?",
    )
    .bind(&export.lifecycle.display_name)
    .bind(export.lifecycle.storage_status.as_str())
    .bind(i64::from(export.lifecycle.state_schema_version))
    .bind(&export.lifecycle.created_at_utc)
    .bind(&export.lifecycle.archived_at_utc)
    .bind(&export.campaign_id)
    .execute(&mut *tx)
    .await?;

    for row in &export.play_sessions {
        insert_play_session(&mut tx, row).await?;
    }
    for row in &export.play_session_participants {
        insert_participant(&mut tx, row).await?;
    }
    for row in &export.command_audit {
        insert_command_audit(&mut tx, row).await?;
    }
    for row in &export.event_journal {
        insert_event_journal(&mut tx, row).await?;
    }
    for row in &export.event_causes {
        insert_event_cause(&mut tx, row).await?;
    }
    for row in &export.snapshots {
        insert_snapshot(&mut tx, row).await?;
    }
    sqlx::query("DELETE FROM campaign_restore_authorizations WHERE campaign_id = ?")
        .bind(&export.campaign_id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;

    let campaign_uuid = uuid::Uuid::parse_str(&export.campaign_id)
        .map_err(|error| LifecycleError::CorruptExport(format!("invalid campaign id: {error}")))?;
    open_campaign(pool, CampaignId(campaign_uuid)).await
}

async fn load_lifecycle(
    pool: &SqlitePool,
    campaign_id: CampaignId,
) -> Result<CampaignLifecycleSummary, LifecycleError> {
    let row = sqlx::query(
        "SELECT campaign_id, display_name, storage_status, state_schema_version, created_at_utc, archived_at_utc \
         FROM campaign_lifecycle WHERE campaign_id = ?",
    )
    .bind(campaign_id.0.to_string())
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| LifecycleError::CorruptLifecycle("missing lifecycle row".into()))?;
    decode_lifecycle_row(&row)
}

fn decode_lifecycle_row(row: &SqliteRow) -> Result<CampaignLifecycleSummary, LifecycleError> {
    let raw_version: i64 = row.try_get("state_schema_version")?;
    Ok(CampaignLifecycleSummary {
        campaign_id: row.try_get("campaign_id")?,
        display_name: row.try_get("display_name")?,
        storage_status: CampaignStorageStatus::parse(
            row.try_get::<String, _>("storage_status")?.as_str(),
        )?,
        state_schema_version: u32::try_from(raw_version)
            .map_err(|_| LifecycleError::CorruptLifecycle("invalid schema version".into()))?,
        created_at_utc: row.try_get("created_at_utc")?,
        archived_at_utc: row.try_get("archived_at_utc")?,
    })
}

fn decode_play_session_row(row: SqliteRow) -> Result<PlaySessionRow, sqlx::Error> {
    Ok(PlaySessionRow {
        id: row.try_get("id")?,
        campaign_id: row.try_get("campaign_id")?,
        display_name: row.try_get("display_name")?,
        status: row.try_get("status")?,
        started_at_world: row.try_get("started_at_world")?,
        ended_at_world: row.try_get("ended_at_world")?,
    })
}

fn decode_participant_row(row: SqliteRow) -> Result<PlaySessionParticipantRow, sqlx::Error> {
    Ok(PlaySessionParticipantRow {
        session_id: row.try_get("session_id")?,
        ordinal: row.try_get("ordinal")?,
        player_id: row.try_get("player_id")?,
        character_id: row.try_get("character_id")?,
        attendance: row.try_get("attendance")?,
    })
}

fn decode_command_audit_row(row: SqliteRow) -> Result<CommandAuditRow, sqlx::Error> {
    Ok(CommandAuditRow {
        id: row.try_get("id")?,
        campaign_id: row.try_get("campaign_id")?,
        session_id: row.try_get("session_id")?,
        issuer_kind: row.try_get("issuer_kind")?,
        issuer_player_id: row.try_get("issuer_player_id")?,
        actor_kind: row.try_get("actor_kind")?,
        actor_id: row.try_get("actor_id")?,
        expected_event_sequence: row.try_get("expected_event_sequence")?,
        command_kind: row.try_get("command_kind")?,
        command_schema_version: row.try_get("command_schema_version")?,
        payload_json: row.try_get("payload_json")?,
        accepted: row.try_get("accepted")?,
        resolution_explanation: row.try_get("resolution_explanation")?,
        resulting_event_sequence: row.try_get("resulting_event_sequence")?,
    })
}

fn decode_event_journal_row(row: SqliteRow) -> Result<EventJournalRow, sqlx::Error> {
    Ok(EventJournalRow {
        id: row.try_get("id")?,
        campaign_id: row.try_get("campaign_id")?,
        sequence: row.try_get("sequence")?,
        session_id: row.try_get("session_id")?,
        occurred_at_world: row.try_get("occurred_at_world")?,
        source: row.try_get("source")?,
        actor_kind: row.try_get("actor_kind")?,
        actor_id: row.try_get("actor_id")?,
        command_id: row.try_get("command_id")?,
        event_kind: row.try_get("event_kind")?,
        event_schema_version: row.try_get("event_schema_version")?,
        payload_json: row.try_get("payload_json")?,
    })
}

fn decode_event_cause_row(row: SqliteRow) -> Result<EventCauseRow, sqlx::Error> {
    Ok(EventCauseRow {
        campaign_id: row.try_get("campaign_id")?,
        event_id: row.try_get("event_id")?,
        cause_event_id: row.try_get("cause_event_id")?,
        ordinal: row.try_get("ordinal")?,
    })
}

fn decode_snapshot_row(row: SqliteRow) -> Result<SnapshotRow, sqlx::Error> {
    Ok(SnapshotRow {
        campaign_id: row.try_get("campaign_id")?,
        event_sequence: row.try_get("event_sequence")?,
        state_schema_version: row.try_get("state_schema_version")?,
        state_json: row.try_get("state_json")?,
        created_at_utc: row.try_get("created_at_utc")?,
    })
}

async fn insert_play_session(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    row: &PlaySessionRow,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO play_sessions \
         (id, campaign_id, display_name, status, started_at_world, ended_at_world) \
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(&row.id)
    .bind(&row.campaign_id)
    .bind(&row.display_name)
    .bind(&row.status)
    .bind(row.started_at_world)
    .bind(row.ended_at_world)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

async fn insert_participant(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    row: &PlaySessionParticipantRow,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO play_session_participants \
         (session_id, ordinal, player_id, character_id, attendance) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&row.session_id)
    .bind(row.ordinal)
    .bind(&row.player_id)
    .bind(&row.character_id)
    .bind(&row.attendance)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

async fn insert_command_audit(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    row: &CommandAuditRow,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO command_audit \
         (id, campaign_id, session_id, issuer_kind, issuer_player_id, actor_kind, actor_id, \
         expected_event_sequence, command_kind, command_schema_version, payload_json, accepted, \
         resolution_explanation, resulting_event_sequence) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&row.id)
    .bind(&row.campaign_id)
    .bind(&row.session_id)
    .bind(&row.issuer_kind)
    .bind(&row.issuer_player_id)
    .bind(&row.actor_kind)
    .bind(&row.actor_id)
    .bind(row.expected_event_sequence)
    .bind(&row.command_kind)
    .bind(row.command_schema_version)
    .bind(&row.payload_json)
    .bind(row.accepted)
    .bind(&row.resolution_explanation)
    .bind(row.resulting_event_sequence)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

async fn insert_event_journal(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    row: &EventJournalRow,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO event_journal \
         (id, campaign_id, sequence, session_id, occurred_at_world, source, actor_kind, actor_id, \
         command_id, event_kind, event_schema_version, payload_json) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&row.id)
    .bind(&row.campaign_id)
    .bind(row.sequence)
    .bind(&row.session_id)
    .bind(row.occurred_at_world)
    .bind(&row.source)
    .bind(&row.actor_kind)
    .bind(&row.actor_id)
    .bind(&row.command_id)
    .bind(&row.event_kind)
    .bind(row.event_schema_version)
    .bind(&row.payload_json)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

async fn insert_event_cause(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    row: &EventCauseRow,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO event_causes (campaign_id, event_id, cause_event_id, ordinal) \
         VALUES (?, ?, ?, ?)",
    )
    .bind(&row.campaign_id)
    .bind(&row.event_id)
    .bind(&row.cause_event_id)
    .bind(row.ordinal)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

async fn insert_snapshot(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    row: &SnapshotRow,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO campaign_snapshots \
         (campaign_id, event_sequence, state_schema_version, state_json, created_at_utc) \
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&row.campaign_id)
    .bind(row.event_sequence)
    .bind(row.state_schema_version)
    .bind(&row.state_json)
    .bind(&row.created_at_utc)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

fn validate_export(export: &CampaignExport) -> Result<(), LifecycleError> {
    validate_export_metadata(export)?;
    let state = decode_exported_state(export)?;
    let head = u64::try_from(export.current_state.applied_event_sequence)
        .map_err(|_| LifecycleError::CorruptExport("negative event head".into()))?;
    if state.applied_event_sequence != head {
        return Err(LifecycleError::CorruptExport(
            "embedded event head mismatch".into(),
        ));
    }

    let session_ids = validate_sessions(export, &state)?;
    let event_sequences = validate_journal(export, head, &state, &session_ids)?;
    validate_snapshots(export, &event_sequences)?;
    validate_state_provenance(
        &state,
        &event_sequences,
        i64::try_from(head).unwrap_or(i64::MAX),
    )?;
    Ok(())
}

fn validate_export_metadata(export: &CampaignExport) -> Result<(), LifecycleError> {
    validate_export_metadata_version(export, CURRENT_STATE_SCHEMA_VERSION)
}

fn validate_export_metadata_version(
    export: &CampaignExport,
    expected_state_version: u32,
) -> Result<(), LifecycleError> {
    if export.format_version != CAMPAIGN_EXPORT_FORMAT_VERSION {
        return Err(LifecycleError::IncompatibleExportFormat {
            actual: export.format_version,
            supported: CAMPAIGN_EXPORT_FORMAT_VERSION,
        });
    }
    if export.state_schema_version != expected_state_version {
        return Err(LifecycleError::IncompatibleStateSchema {
            actual: export.state_schema_version,
            supported: CURRENT_STATE_SCHEMA_VERSION,
        });
    }
    if export.current_state.campaign_id != export.campaign_id
        || export.lifecycle.campaign_id != export.campaign_id
    {
        return Err(LifecycleError::CorruptExport(
            "campaign identity mismatch".into(),
        ));
    }
    if export.current_state.schema_version != i64::from(export.state_schema_version)
        || export.lifecycle.state_schema_version != export.state_schema_version
    {
        return Err(LifecycleError::CorruptExport(
            "state schema metadata mismatch".into(),
        ));
    }
    uuid::Uuid::parse_str(&export.campaign_id)
        .map_err(|error| LifecycleError::CorruptExport(format!("invalid campaign id: {error}")))?;
    Ok(())
}

fn decode_exported_state(export: &CampaignExport) -> Result<CampaignState, LifecycleError> {
    let state = CampaignState::decode_json(&export.current_state.state_json)
        .map_err(|error| LifecycleError::CorruptExport(format!("current state JSON: {error}")))?;
    if state.campaign_id().0.to_string() != export.campaign_id {
        return Err(LifecycleError::CorruptExport(
            "embedded campaign identity mismatch".into(),
        ));
    }
    if state.schema_version != export.state_schema_version || !state.validate().is_empty() {
        return Err(LifecycleError::CorruptExport(
            "current state is structurally invalid".into(),
        ));
    }
    Ok(state)
}

#[derive(Debug, Clone)]
struct ExportCommandMetadata {
    session_id: Option<String>,
    accepted: bool,
    expected_event_sequence: i64,
    resulting_event_sequence: i64,
}

fn validate_serialized_record(
    kind: &str,
    schema_version: i64,
    payload_json: &str,
    owner: &str,
) -> Result<(), LifecycleError> {
    if kind.trim().is_empty() {
        return Err(LifecycleError::CorruptExport(format!(
            "{owner} record kind must not be empty"
        )));
    }
    let schema_version = u32::try_from(schema_version).map_err(|_| {
        LifecycleError::CorruptExport(format!(
            "{owner} record schema version is outside the supported u32 range"
        ))
    })?;
    if schema_version == 0 {
        return Err(LifecycleError::CorruptExport(format!(
            "{owner} record schema version must be positive"
        )));
    }
    serde_json::from_str::<serde_json::Value>(payload_json).map_err(|error| {
        LifecycleError::CorruptExport(format!("{owner} payload JSON is invalid: {error}"))
    })?;
    Ok(())
}

fn validate_journal(
    export: &CampaignExport,
    head: u64,
    state: &CampaignState,
    session_ids: &HashSet<String>,
) -> Result<HashMap<String, i64>, LifecycleError> {
    if export.event_journal.len() as u64 != head {
        return Err(LifecycleError::CorruptExport(
            "journal count does not match current-state head".into(),
        ));
    }

    let mut commands = HashMap::with_capacity(export.command_audit.len());
    for row in &export.command_audit {
        if row.campaign_id != export.campaign_id || commands.contains_key(&row.id) {
            return Err(LifecycleError::CorruptExport(
                "invalid or duplicate command audit row".into(),
            ));
        }
        parse_export_uuid(&row.id, "command audit id")?;
        validate_session_reference(row.session_id.as_deref(), session_ids, "command")?;
        validate_command_issuer(row, state)?;
        validate_actor_reference(
            row.actor_kind.as_deref(),
            row.actor_id.as_deref(),
            state,
            "command actor",
        )?;
        validate_serialized_record(
            &row.command_kind,
            row.command_schema_version,
            &row.payload_json,
            "command",
        )?;
        if row.expected_event_sequence < 0
            || row.resulting_event_sequence < row.expected_event_sequence
        {
            return Err(LifecycleError::CorruptExport(
                "invalid command audit sequence metadata".into(),
            ));
        }
        let accepted = match row.accepted {
            0 => false,
            1 => true,
            _ => {
                return Err(LifecycleError::CorruptExport(
                    "invalid command acceptance metadata".into(),
                ));
            }
        };
        if accepted {
            if row.resolution_explanation.trim().is_empty() {
                return Err(LifecycleError::CorruptExport(
                    "accepted command resolution explanation must not be empty".into(),
                ));
            }
            if row.resulting_event_sequence <= row.expected_event_sequence {
                return Err(LifecycleError::CorruptExport(
                    "accepted command must advance the event sequence".into(),
                ));
            }
        }
        commands.insert(
            row.id.clone(),
            ExportCommandMetadata {
                session_id: row.session_id.clone(),
                accepted,
                expected_event_sequence: row.expected_event_sequence,
                resulting_event_sequence: row.resulting_event_sequence,
            },
        );
    }

    let mut event_sequences = HashMap::with_capacity(export.event_journal.len());
    let mut command_event_sequences = HashMap::<String, Vec<i64>>::with_capacity(commands.len());
    for (index, event) in export.event_journal.iter().enumerate() {
        let expected = i64::try_from(index + 1)
            .map_err(|_| LifecycleError::CorruptExport("journal sequence overflow".into()))?;
        if event.campaign_id != export.campaign_id || event.sequence != expected {
            return Err(LifecycleError::CorruptExport(
                "journal is not contiguous for one campaign".into(),
            ));
        }
        parse_export_uuid(&event.id, "event id")?;
        validate_session_reference(event.session_id.as_deref(), session_ids, "event")?;
        validate_event_source(&event.source)?;
        validate_actor_reference(
            event.actor_kind.as_deref(),
            event.actor_id.as_deref(),
            state,
            "event actor",
        )?;
        validate_serialized_record(
            &event.event_kind,
            event.event_schema_version,
            &event.payload_json,
            "event",
        )?;
        let Some(command) = commands.get(&event.command_id) else {
            return Err(LifecycleError::CorruptExport(
                "event references missing command".into(),
            ));
        };
        if !command.accepted {
            return Err(LifecycleError::CorruptExport(
                "event references rejected command".into(),
            ));
        }
        if event.session_id.as_deref() != command.session_id.as_deref() {
            return Err(LifecycleError::CorruptExport(
                "event session does not match originating command session".into(),
            ));
        }
        if event.sequence <= command.expected_event_sequence
            || event.sequence > command.resulting_event_sequence
        {
            return Err(LifecycleError::CorruptExport(
                "event sequence falls outside originating command batch".into(),
            ));
        }
        command_event_sequences
            .entry(event.command_id.clone())
            .or_default()
            .push(event.sequence);
        if event_sequences
            .insert(event.id.clone(), event.sequence)
            .is_some()
        {
            return Err(LifecycleError::CorruptExport(
                "duplicate event id in journal".into(),
            ));
        }
    }

    for (command_id, command) in &commands {
        let sequences = command_event_sequences
            .get(command_id)
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        if !command.accepted {
            if !sequences.is_empty() {
                return Err(LifecycleError::CorruptExport(
                    "rejected command unexpectedly emitted events".into(),
                ));
            }
            continue;
        }

        let span = command
            .resulting_event_sequence
            .checked_sub(command.expected_event_sequence)
            .ok_or_else(|| {
                LifecycleError::CorruptExport("invalid accepted command sequence span".into())
            })?;
        let expected_count = usize::try_from(span).map_err(|_| {
            LifecycleError::CorruptExport("accepted command event span overflow".into())
        })?;
        if sequences.len() != expected_count {
            return Err(LifecycleError::CorruptExport(
                "accepted command event count does not match sequence metadata".into(),
            ));
        }
        for (offset, sequence) in sequences.iter().enumerate() {
            let offset = i64::try_from(offset).map_err(|_| {
                LifecycleError::CorruptExport("accepted command event offset overflow".into())
            })?;
            let expected_sequence = command
                .expected_event_sequence
                .checked_add(offset + 1)
                .ok_or_else(|| {
                    LifecycleError::CorruptExport("accepted command event sequence overflow".into())
                })?;
            if *sequence != expected_sequence {
                return Err(LifecycleError::CorruptExport(
                    "accepted command event batch is not contiguous".into(),
                ));
            }
        }
    }

    for cause in &export.event_causes {
        if cause.campaign_id != export.campaign_id {
            return Err(LifecycleError::CorruptExport(
                "cross-campaign causal row".into(),
            ));
        }
        let child = event_sequences.get(&cause.event_id).copied();
        let parent = event_sequences.get(&cause.cause_event_id).copied();
        match (child, parent) {
            (Some(child), Some(parent)) if parent < child => {}
            _ => return Err(LifecycleError::CorruptExport("invalid causal edge".into())),
        }
    }
    Ok(event_sequences)
}

fn validate_sessions(
    export: &CampaignExport,
    state: &CampaignState,
) -> Result<HashSet<String>, LifecycleError> {
    let mut session_ids = HashSet::with_capacity(export.play_sessions.len());
    let mut participants_by_session =
        HashMap::<&str, Vec<&PlaySessionParticipantRow>>::with_capacity(export.play_sessions.len());
    for participant in &export.play_session_participants {
        participants_by_session
            .entry(participant.session_id.as_str())
            .or_default()
            .push(participant);
    }

    for session_row in &export.play_sessions {
        if session_row.campaign_id != export.campaign_id
            || !session_ids.insert(session_row.id.clone())
        {
            return Err(LifecycleError::CorruptExport(
                "invalid or duplicate play session".into(),
            ));
        }
        let id = PlaySessionId(parse_export_uuid(&session_row.id, "play session id")?);
        let campaign_id = CampaignId(parse_export_uuid(
            &session_row.campaign_id,
            "play session campaign id",
        )?);
        let status = match session_row.status.as_str() {
            "active" => PlaySessionStatus::Active,
            "closed" => PlaySessionStatus::Closed,
            _ => {
                return Err(LifecycleError::CorruptExport(
                    "invalid play session status".into(),
                ));
            }
        };
        let mut rows = participants_by_session
            .remove(session_row.id.as_str())
            .unwrap_or_default();
        rows.sort_by_key(|row| row.ordinal);
        let mut participants = Vec::with_capacity(rows.len());
        for (expected_ordinal, row) in rows.into_iter().enumerate() {
            let expected_ordinal = i64::try_from(expected_ordinal).map_err(|_| {
                LifecycleError::CorruptExport("participant ordinal overflow".into())
            })?;
            if row.ordinal != expected_ordinal {
                return Err(LifecycleError::CorruptExport(
                    "play session participant ordinals are not contiguous".into(),
                ));
            }
            let player_id = PlayerId(parse_export_uuid(
                &row.player_id,
                "play session participant player id",
            )?);
            let character_id = row
                .character_id
                .as_deref()
                .map(|value| {
                    parse_export_uuid(value, "play session participant character id")
                        .map(CharacterId)
                })
                .transpose()?;
            let attendance = match row.attendance.as_str() {
                "present" => AttendanceStatus::Present,
                "absent" => AttendanceStatus::Absent,
                _ => {
                    return Err(LifecycleError::CorruptExport(
                        "invalid play session attendance".into(),
                    ));
                }
            };
            participants.push(SessionParticipant {
                player_id,
                character_id,
                attendance,
            });
        }
        let session = PlaySession {
            id,
            campaign_id,
            display_name: session_row.display_name.clone(),
            status,
            started_at_world: WorldInstant(session_row.started_at_world),
            ended_at_world: session_row.ended_at_world.map(WorldInstant),
            participants,
        };
        if !session.validate_against_state(state).is_empty() {
            return Err(LifecycleError::CorruptExport(
                "play session violates world-reference invariants".into(),
            ));
        }
    }
    if !participants_by_session.is_empty() {
        return Err(LifecycleError::CorruptExport(
            "participant references missing session".into(),
        ));
    }
    Ok(session_ids)
}

fn validate_session_reference(
    session_id: Option<&str>,
    session_ids: &HashSet<String>,
    owner: &str,
) -> Result<(), LifecycleError> {
    let Some(session_id) = session_id else {
        return Ok(());
    };
    parse_export_uuid(session_id, &format!("{owner} session id"))?;
    if !session_ids.contains(session_id) {
        return Err(LifecycleError::CorruptExport(format!(
            "{owner} references missing play session"
        )));
    }
    Ok(())
}

fn validate_command_issuer(
    row: &CommandAuditRow,
    state: &CampaignState,
) -> Result<(), LifecycleError> {
    match (row.issuer_kind.as_str(), row.issuer_player_id.as_deref()) {
        ("player", Some(value)) => {
            let player_id = PlayerId(parse_export_uuid(value, "command issuer player id")?);
            if !state.players.contains_key(&player_id) {
                return Err(LifecycleError::CorruptExport(
                    "command issuer references missing player".into(),
                ));
            }
        }
        ("system" | "admin" | "import", None) => {}
        _ => {
            return Err(LifecycleError::CorruptExport(
                "invalid command issuer metadata".into(),
            ));
        }
    }
    Ok(())
}

fn validate_actor_reference(
    actor_kind: Option<&str>,
    actor_id: Option<&str>,
    state: &CampaignState,
    field: &str,
) -> Result<(), LifecycleError> {
    match (actor_kind, actor_id) {
        (None, None) => Ok(()),
        (Some("entity"), Some(value)) => {
            let entity_id = EntityId(parse_export_uuid(value, &format!("{field} entity id"))?);
            if state.entities.contains_key(&entity_id) {
                Ok(())
            } else {
                Err(LifecycleError::CorruptExport(format!(
                    "{field} references missing entity"
                )))
            }
        }
        (Some("faction"), Some(value)) => {
            let faction_id = FactionId(parse_export_uuid(value, &format!("{field} faction id"))?);
            if state.factions.contains_key(&faction_id) {
                Ok(())
            } else {
                Err(LifecycleError::CorruptExport(format!(
                    "{field} references missing faction"
                )))
            }
        }
        _ => Err(LifecycleError::CorruptExport(format!(
            "invalid {field} metadata"
        ))),
    }
}

fn validate_event_source(source: &str) -> Result<(), LifecycleError> {
    match source {
        "player_action"
        | "rule_resolution"
        | "world_simulation"
        | "procedural_generation"
        | "admin_correction"
        | "import" => Ok(()),
        _ => Err(LifecycleError::CorruptExport(
            "invalid event source metadata".into(),
        )),
    }
}

fn parse_export_uuid(value: &str, field: &str) -> Result<uuid::Uuid, LifecycleError> {
    uuid::Uuid::parse_str(value)
        .map_err(|error| LifecycleError::CorruptExport(format!("invalid {field}: {error}")))
}

fn validate_snapshots(
    export: &CampaignExport,
    event_sequences: &HashMap<String, i64>,
) -> Result<(), LifecycleError> {
    if export.snapshots.is_empty() {
        return Err(LifecycleError::CorruptExport(
            "campaign has no recovery snapshot".into(),
        ));
    }
    let mut seen_sequences = HashSet::with_capacity(export.snapshots.len());
    for snapshot in &export.snapshots {
        if snapshot.campaign_id != export.campaign_id
            || snapshot.event_sequence < 0
            || snapshot.event_sequence > export.current_state.applied_event_sequence
            || !seen_sequences.insert(snapshot.event_sequence)
        {
            return Err(LifecycleError::CorruptExport(
                "invalid snapshot metadata".into(),
            ));
        }
        let stored_version = u32::try_from(snapshot.state_schema_version)
            .map_err(|_| LifecycleError::CorruptExport("invalid snapshot schema version".into()))?;
        let snapshot_state = CampaignStateSnapshotCodec::new()
            .decode_state(stored_version, &snapshot.state_json)
            .map_err(|error| LifecycleError::CorruptExport(format!("snapshot state: {error}")))?;
        if snapshot_state.campaign_id().0.to_string() != export.campaign_id
            || snapshot_state.applied_event_sequence != snapshot.event_sequence as u64
        {
            return Err(LifecycleError::CorruptExport(
                "snapshot state metadata mismatch".into(),
            ));
        }
        validate_state_provenance(&snapshot_state, event_sequences, snapshot.event_sequence)?;
    }
    Ok(())
}

fn validate_state_provenance(
    state: &CampaignState,
    event_sequences: &HashMap<String, i64>,
    max_sequence: i64,
) -> Result<(), LifecycleError> {
    for event_id in collect_state_event_references(state) {
        let id = event_id.0.to_string();
        match event_sequences.get(&id) {
            Some(sequence) if *sequence <= max_sequence => {}
            _ => {
                return Err(LifecycleError::CorruptExport(
                    "state references missing or future journal provenance".into(),
                ));
            }
        }
    }
    Ok(())
}

fn collect_state_event_references(state: &CampaignState) -> HashSet<EventId> {
    let mut references = HashSet::new();
    for fact in state.facts.values() {
        if let Some(event_id) = fact.source_event_id {
            references.insert(event_id);
        }
    }
    for claim in state.claims.values() {
        references.insert(claim.source_event_id);
    }
    for belief in state.beliefs.values() {
        for basis in &belief.basis {
            collect_belief_event_references(basis, &mut references);
        }
    }
    for knowledge in &state.knowledge {
        references.insert(knowledge.source_event_id);
    }
    references
}

fn collect_belief_event_references(basis: &BeliefBasis, references: &mut HashSet<EventId>) {
    match basis {
        BeliefBasis::DirectObservation(event_id) => {
            references.insert(*event_id);
        }
        BeliefBasis::Inference(nested) => {
            for basis in nested {
                collect_belief_event_references(basis, references);
            }
        }
        BeliefBasis::Fact(_) | BeliefBasis::Claim(_) => {}
    }
}
