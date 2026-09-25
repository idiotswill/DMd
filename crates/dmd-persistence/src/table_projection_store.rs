//! Durable presentation evidence. The application authenticates contents against
//! historical rules/table replay; this store never treats presentation as game state.
use dmd_domain::{CampaignId, CommandId, CommandMeta, EventId, ObservationId, PlayerId};
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqliteConnection};
use std::collections::{HashMap, HashSet};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProjectionAudience {
    Host,
    Player(PlayerId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ProjectionRevision(pub Uuid);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum ProjectionCause {
    /// First protocol use preserves old canonical bytes and records its replayed
    /// historical visibility. No opaque tokens existed before this checkpoint.
    Bootstrap {
        event_sequence: u64,
        observation_ordinal: u64,
    },
    Event {
        id: EventId,
        command_id: CommandId,
    },
    Observation {
        id: ObservationId,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TranscriptVisibility {
    pub event_id: EventId,
    pub audiences: Vec<ProjectionAudience>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum ProjectionCapability {
    /// No canonical request UUID is sent to the player. Its deterministic internal
    /// occurrence may otherwise expose secret intervening work by enumeration.
    Roll {
        canonical: dmd_domain::RollRequestId,
    },
    Work {
        origin: CommandId,
        occurrence: u16,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectionHandle {
    pub opaque: Uuid,
    pub capability: ProjectionCapability,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectionChange {
    pub audience: ProjectionAudience,
    pub previous: Option<ProjectionRevision>,
    pub revision: ProjectionRevision,
    /// SHA-256 of the canonical audience projection, excluding opaque presentation
    /// IDs and host diagnostics. Never returned to a player or used as a revision.
    pub visible_digest: String,
    pub handles: Vec<ProjectionHandle>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TableProjectionRecord {
    pub version: u32,
    pub campaign_id: CampaignId,
    /// Internal ledger ordering only; never a player-facing cursor.
    pub ordinal: u64,
    pub cause: ProjectionCause,
    pub transcript: Vec<TranscriptVisibility>,
    /// Exactly the audiences whose visible presentation changed. Hidden-only
    /// acceptance still records its cause, with no player revision change.
    pub changes: Vec<ProjectionChange>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum TransportAcceptance {
    Command { resulting_event_sequence: u64 },
    Observation { id: ObservationId },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TableTransportBinding {
    pub version: u32,
    pub meta: CommandMeta,
    pub audience: ProjectionAudience,
    pub projection_ordinal: u64,
    pub acceptance: TransportAcceptance,
    /// Application-defined typed envelopes, decoded and semantically checked by
    /// the application before restore. Keep the complete original input and output.
    pub request_json: String,
    pub response_json: String,
}

#[derive(Debug, Error)]
pub enum TableProjectionStoreError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error("invalid table presentation or transport record")]
    InvalidRecord,
}

pub async fn load_table_projection_history(
    connection: &mut SqliteConnection,
    campaign_id: CampaignId,
) -> Result<Vec<TableProjectionRecord>, TableProjectionStoreError> {
    let rows = sqlx::query("SELECT ordinal,record_json FROM table_projection_history WHERE campaign_id=? ORDER BY ordinal")
        .bind(campaign_id.0.to_string()).fetch_all(connection).await?;
    rows.into_iter()
        .map(|row| {
            let record: TableProjectionRecord =
                serde_json::from_str(&row.try_get::<String, _>("record_json")?)
                    .map_err(|_| TableProjectionStoreError::InvalidRecord)?;
            if record.campaign_id != campaign_id
                || record.version != 1
                || i64::try_from(record.ordinal).ok() != Some(row.try_get("ordinal")?)
            {
                return Err(TableProjectionStoreError::InvalidRecord);
            }
            Ok(record)
        })
        .collect()
}

pub async fn insert_table_projection_record(
    connection: &mut SqliteConnection,
    record: &TableProjectionRecord,
) -> Result<(), TableProjectionStoreError> {
    if record.version != 1 || record.ordinal == 0 {
        return Err(TableProjectionStoreError::InvalidRecord);
    }
    let ordinal =
        i64::try_from(record.ordinal).map_err(|_| TableProjectionStoreError::InvalidRecord)?;
    let json =
        serde_json::to_string(record).map_err(|_| TableProjectionStoreError::InvalidRecord)?;
    sqlx::query(
        "INSERT INTO table_projection_history(campaign_id,ordinal,record_json) VALUES(?,?,?)",
    )
    .bind(record.campaign_id.0.to_string())
    .bind(ordinal)
    .bind(json)
    .execute(connection)
    .await?;
    Ok(())
}

pub async fn load_table_transport_bindings(
    connection: &mut SqliteConnection,
    campaign_id: CampaignId,
) -> Result<Vec<TableTransportBinding>, TableProjectionStoreError> {
    let rows = sqlx::query("SELECT command_id,projection_ordinal,record_json FROM table_transport_bindings WHERE campaign_id=? ORDER BY command_id")
        .bind(campaign_id.0.to_string()).fetch_all(connection).await?;
    rows.into_iter()
        .map(|row| {
            let record: TableTransportBinding =
                serde_json::from_str(&row.try_get::<String, _>("record_json")?)
                    .map_err(|_| TableProjectionStoreError::InvalidRecord)?;
            if record.meta.campaign_id != campaign_id
                || record.version != 1
                || record.meta.id.0.to_string() != row.try_get::<String, _>("command_id")?
                || i64::try_from(record.projection_ordinal).ok()
                    != Some(row.try_get("projection_ordinal")?)
            {
                return Err(TableProjectionStoreError::InvalidRecord);
            }
            Ok(record)
        })
        .collect()
}

pub async fn insert_table_transport_binding(
    connection: &mut SqliteConnection,
    record: &TableTransportBinding,
) -> Result<(), TableProjectionStoreError> {
    if record.version != 1 || record.projection_ordinal == 0 {
        return Err(TableProjectionStoreError::InvalidRecord);
    }
    let ordinal = i64::try_from(record.projection_ordinal)
        .map_err(|_| TableProjectionStoreError::InvalidRecord)?;
    let json =
        serde_json::to_string(record).map_err(|_| TableProjectionStoreError::InvalidRecord)?;
    sqlx::query("INSERT INTO table_transport_bindings(command_id,campaign_id,projection_ordinal,record_json) VALUES(?,?,?,?)")
        .bind(record.meta.id.0.to_string()).bind(record.meta.campaign_id.0.to_string()).bind(ordinal).bind(json).execute(connection).await?;
    Ok(())
}

/// Structural/provenance validation independent of application DTOs. The application
/// additionally replays visible presentation and decodes exact request/response bodies.
pub(crate) fn validate_portable_protocol(
    export: &crate::CampaignExport,
    state: &dmd_domain::CampaignState,
) -> Result<(), crate::LifecycleError> {
    use crate::LifecycleError;
    let invalid =
        || LifecycleError::CorruptExport("invalid table projection/transport history".into());
    let audience_valid = |audience: ProjectionAudience| match audience {
        ProjectionAudience::Host => true,
        ProjectionAudience::Player(id) => state.players.contains_key(&id),
    };
    let head = state.applied_event_sequence;
    let mut event_head = 0;
    let mut observation_head = 0;
    let mut revisions = HashMap::new();
    let mut revision_ids = HashSet::new();
    let mut handles = HashMap::new();
    let mut visible_events = HashSet::new();
    let events = export
        .event_journal
        .iter()
        .map(|row| (row.id.as_str(), row))
        .collect::<HashMap<_, _>>();
    let observations = export
        .observations
        .iter()
        .map(|row| (row.record.id, row))
        .collect::<HashMap<_, _>>();
    for (index, record) in export.table_projection_history.iter().enumerate() {
        if record.version != 1
            || record.campaign_id != state.campaign_id()
            || record.ordinal != index as u64 + 1
        {
            return Err(invalid());
        }
        match record.cause {
            ProjectionCause::Bootstrap {
                event_sequence,
                observation_ordinal,
            } => {
                if index != 0
                    || event_sequence > head
                    || observation_ordinal > export.observations.len() as u64
                {
                    return Err(invalid());
                }
                event_head = event_sequence;
                observation_head = observation_ordinal;
            }
            ProjectionCause::Event { id, command_id } => {
                let row = events.get(id.0.to_string().as_str()).ok_or_else(invalid)?;
                if index == 0
                    || row.command_id != command_id.0.to_string()
                    || u64::try_from(row.sequence).ok() != event_head.checked_add(1)
                {
                    return Err(invalid());
                }
                event_head += 1;
            }
            ProjectionCause::Observation { id } => {
                let row = observations.get(&id).ok_or_else(invalid)?;
                if index == 0
                    || Some(row.ordinal) != observation_head.checked_add(1)
                    || row.record.observed_event_sequence != event_head
                {
                    return Err(invalid());
                }
                observation_head += 1;
            }
        }
        for visible in &record.transcript {
            let row = events
                .get(visible.event_id.0.to_string().as_str())
                .ok_or_else(invalid)?;
            if row.sequence < 0
                || row.sequence as u64 > event_head
                || !visible_events.insert(visible.event_id)
            {
                return Err(invalid());
            }
            let mut unique = HashSet::new();
            if visible.audiences.is_empty()
                || visible
                    .audiences
                    .iter()
                    .any(|audience| !audience_valid(*audience) || !unique.insert(*audience))
            {
                return Err(invalid());
            }
        }
        let mut changed = HashSet::new();
        for change in &record.changes {
            if !audience_valid(change.audience)
                || !changed.insert(change.audience)
                || change.previous != revisions.get(&change.audience).copied()
                || change.revision.0.get_version() != Some(uuid::Version::Random)
                || !revision_ids.insert(change.revision)
                || change.visible_digest.len() != 64
                || !change
                    .visible_digest
                    .bytes()
                    .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
            {
                return Err(invalid());
            }
            revisions.insert(change.audience, change.revision);
            let mut unique = HashSet::new();
            for handle in &change.handles {
                if handle.opaque.get_version() != Some(uuid::Version::Random)
                    || !unique.insert(handle.opaque)
                {
                    return Err(invalid());
                }
                if let Some(existing) =
                    handles.insert(handle.opaque, (change.audience, &handle.capability))
                    && existing != (change.audience, &handle.capability)
                {
                    return Err(invalid());
                }
            }
        }
    }
    if !export.table_projection_history.is_empty()
        && (event_head != head || observation_head != export.observations.len() as u64)
    {
        return Err(invalid());
    }
    let mut commands = HashSet::new();
    for binding in &export.table_transport_bindings {
        let (issuer, player) = crate::journal_store::encode_issuer(binding.meta.issuer);
        let (actor_kind, actor_id) = crate::journal_store::encode_agent(binding.meta.actor);
        if binding.version != 1
            || binding.meta.campaign_id != state.campaign_id()
            || !commands.insert(binding.meta.id)
            || binding.projection_ordinal == 0
            || binding.projection_ordinal > export.table_projection_history.len() as u64
            || !audience_valid(binding.audience)
            || !matches!(
                (binding.audience, binding.meta.issuer),
                (ProjectionAudience::Host, dmd_domain::CommandIssuer::Admin)
                    | (
                        ProjectionAudience::Player(_),
                        dmd_domain::CommandIssuer::Player(_)
                    )
            )
            || matches!((binding.audience, binding.meta.issuer),
                (ProjectionAudience::Player(a), dmd_domain::CommandIssuer::Player(b)) if a != b)
            || binding.request_json.len() > 2_000_000
            || binding.response_json.len() > 2_000_000
            || [&binding.request_json, &binding.response_json]
                .into_iter()
                .any(|json| {
                    !serde_json::from_str::<serde_json::Value>(json)
                        .is_ok_and(|value| value.is_object())
                })
        {
            return Err(invalid());
        }
        let presentation =
            &export.table_projection_history[binding.projection_ordinal as usize - 1];
        match binding.acceptance {
            TransportAcceptance::Command {
                resulting_event_sequence,
            } => {
                let ProjectionCause::Event { id, command_id } = presentation.cause else {
                    return Err(invalid());
                };
                if command_id != binding.meta.id
                    || events
                        .get(id.0.to_string().as_str())
                        .and_then(|row| u64::try_from(row.sequence).ok())
                        != Some(resulting_event_sequence)
                {
                    return Err(invalid());
                }
                let audit = export
                    .command_audit
                    .iter()
                    .find(|row| row.id == binding.meta.id.0.to_string())
                    .ok_or_else(invalid)?;
                if audit.accepted != 1
                    || audit.issuer_kind != issuer
                    || audit.issuer_player_id != player
                    || audit.actor_kind.as_deref() != actor_kind
                    || audit.actor_id != actor_id
                    || audit.session_id != binding.meta.session_id.map(|id| id.0.to_string())
                    || u64::try_from(audit.expected_event_sequence).ok()
                        != Some(binding.meta.expected_event_sequence)
                    || u64::try_from(audit.resulting_event_sequence).ok()
                        != Some(resulting_event_sequence)
                {
                    return Err(invalid());
                }
            }
            TransportAcceptance::Observation { id } => {
                if presentation.cause != (ProjectionCause::Observation { id }) {
                    return Err(invalid());
                }
                let observation = observations.get(&id).ok_or_else(invalid)?;
                if id.0 != binding.meta.id.0
                    || observation.record.issuer != binding.meta.issuer
                    || observation.record.session_id != binding.meta.session_id
                    || observation.record.observed_event_sequence
                        != binding.meta.expected_event_sequence
                {
                    return Err(invalid());
                }
            }
        }
    }
    Ok(())
}
