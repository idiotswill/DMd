use std::collections::{HashMap, HashSet};

use dmd_domain::{
    AgentRef, BeliefBasis, CURRENT_STATE_SCHEMA_VERSION, CampaignId, CampaignState, CommandId,
    CommandIssuer, CommandMeta, EncodedPendingEvent, EntityId, EventId, EventMeta, EventSource,
    FactionId, PlaySessionId, PlayerId, SerializedRecord, WorldInstant,
};
use sqlx::{Row, SqlitePool, sqlite::SqliteRow};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitReceipt {
    pub command_id: CommandId,
    pub emitted_event_ids: Vec<EventId>,
    pub resulting_event_sequence: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredPayload {
    pub kind: String,
    pub schema_version: u32,
    pub json: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredJournalEvent {
    pub meta: EventMeta,
    pub payload: StoredPayload,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredCommandAudit {
    pub meta: CommandMeta,
    pub payload: StoredPayload,
    pub accepted: bool,
    pub resolution_explanation: String,
    pub resulting_event_sequence: u64,
    pub emitted_event_ids: Vec<EventId>,
}

#[derive(Debug, Error)]
pub enum JournalStoreError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error("campaign state is structurally invalid")]
    InvalidState,
    #[error("new campaign state must start at event sequence 0, got {0}")]
    InitialSequenceNotZero(u64),
    #[error("campaign state is already initialized")]
    StateAlreadyInitialized,
    #[error("campaign state is not initialized")]
    StateNotInitialized,
    #[error("campaign state JSON could not be encoded: {0}")]
    StateSerialization(String),
    #[error("stored campaign state is corrupt: {0}")]
    CorruptState(String),
    #[error("unsupported campaign-state schema version {actual}; supported version is {supported}")]
    UnsupportedStateSchema { actual: u32, supported: u32 },
    #[error("command campaign does not match the resulting campaign state")]
    CampaignMismatch,
    #[error("command was based on event sequence {expected}, but current sequence is {actual}")]
    StaleState { expected: u64, actual: u64 },
    #[error("a material state transition must emit at least one event")]
    EmptyEventBatch,
    #[error("event sequence exceeds SQLite's signed integer range")]
    SequenceOverflow,
    #[error("next state records applied sequence {actual}, expected {expected}")]
    NextStateSequenceMismatch { expected: u64, actual: u64 },
    #[error("resolution explanation must not be empty")]
    EmptyResolutionExplanation,
    #[error("player issuer does not exist in the authoritative campaign state: {0:?}")]
    MissingIssuerPlayer(PlayerId),
    #[error("command actor does not exist in the authoritative campaign state: {0:?}")]
    MissingCommandActor(AgentRef),
    #[error("event actor does not exist in the resulting campaign state: {0:?}")]
    MissingEventActor(AgentRef),
    #[error("play session does not exist: {0:?}")]
    MissingSession(PlaySessionId),
    #[error("play session {session_id:?} belongs to campaign {actual:?}, not {expected:?}")]
    SessionCampaignMismatch {
        session_id: PlaySessionId,
        expected: CampaignId,
        actual: CampaignId,
    },
    #[error("command id is already present in the audit log: {0:?}")]
    DuplicateCommandId(CommandId),
    #[error("event id appears more than once in the pending batch: {0:?}")]
    DuplicateEventId(EventId),
    #[error("event id already exists in the journal: {0:?}")]
    EventAlreadyExists(EventId),
    #[error("causal parent appears more than once for one event: {0:?}")]
    DuplicateCausalParent(EventId),
    #[error("causal parent event does not exist: {0:?}")]
    MissingCausalParent(EventId),
    #[error("causal parent event {event_id:?} belongs to campaign {actual:?}, not {expected:?}")]
    CausalParentCampaignMismatch {
        event_id: EventId,
        expected: CampaignId,
        actual: CampaignId,
    },
    #[error("causal parent {parent_id:?} does not precede child event {event_id:?}")]
    FutureCausalParent {
        event_id: EventId,
        parent_id: EventId,
    },
    #[error("resulting state references missing journal event: {0:?}")]
    MissingStateEventReference(EventId),
    #[error(
        "resulting state event reference {event_id:?} belongs to campaign {actual:?}, not {expected:?}"
    )]
    StateEventReferenceCampaignMismatch {
        event_id: EventId,
        expected: CampaignId,
        actual: CampaignId,
    },
    #[error(
        "journal head is inconsistent with materialized state: state={state_sequence}, count={event_count}, max={max_sequence}"
    )]
    CorruptJournalHead {
        state_sequence: u64,
        event_count: u64,
        max_sequence: u64,
    },
    #[error("another writer changed the campaign head before this transition could commit")]
    ConcurrentWrite,
    #[error("invalid UUID stored in {field}: {source}")]
    InvalidUuid {
        field: &'static str,
        #[source]
        source: uuid::Error,
    },
    #[error("invalid stored integer in {field}: {value}")]
    InvalidInteger { field: &'static str, value: i64 },
    #[error("invalid stored event source: {0}")]
    InvalidEventSource(String),
    #[error("invalid stored command issuer: {0}")]
    InvalidIssuer(String),
    #[error("invalid stored actor metadata")]
    InvalidActor,
}

pub async fn initialize_campaign_state(
    pool: &SqlitePool,
    state: &CampaignState,
) -> Result<(), JournalStoreError> {
    ensure_supported_state(state)?;
    if state.applied_event_sequence != 0 {
        return Err(JournalStoreError::InitialSequenceNotZero(
            state.applied_event_sequence,
        ));
    }
    if let Some(event_id) = collect_state_event_references(state).into_iter().next() {
        return Err(JournalStoreError::MissingStateEventReference(event_id));
    }

    let campaign_id = state.campaign_id().0.to_string();
    let already_exists =
        sqlx::query_scalar::<_, i64>("SELECT 1 FROM campaign_state_current WHERE campaign_id = ?")
            .bind(&campaign_id)
            .fetch_optional(pool)
            .await?;
    if already_exists.is_some() {
        return Err(JournalStoreError::StateAlreadyInitialized);
    }

    let state_json = state
        .encode_json()
        .map_err(|error| JournalStoreError::StateSerialization(error.to_string()))?;
    sqlx::query(
        "INSERT INTO campaign_state_current (campaign_id, schema_version, applied_event_sequence, state_json) VALUES (?, ?, 0, ?)",
    )
    .bind(campaign_id)
    .bind(i64::from(state.schema_version))
    .bind(state_json)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn load_campaign_state(
    pool: &SqlitePool,
    campaign_id: CampaignId,
) -> Result<Option<CampaignState>, JournalStoreError> {
    let row = sqlx::query(
        "SELECT campaign_id, schema_version, applied_event_sequence, state_json FROM campaign_state_current WHERE campaign_id = ?",
    )
    .bind(campaign_id.0.to_string())
    .fetch_optional(pool)
    .await?;

    let Some(row) = row else {
        return Ok(None);
    };

    let state = decode_state_row(&row)?;
    let head = sqlx::query(
        "SELECT COUNT(*) AS event_count, COALESCE(MAX(sequence), 0) AS max_sequence FROM event_journal WHERE campaign_id = ?",
    )
    .bind(campaign_id.0.to_string())
    .fetch_one(pool)
    .await?;
    verify_journal_head(&state, &head)?;
    verify_loaded_state_event_references(pool, campaign_id, &state).await?;

    Ok(Some(state))
}

pub async fn commit_campaign_transition(
    pool: &SqlitePool,
    command_meta: &CommandMeta,
    command_payload: &SerializedRecord,
    next_state: &CampaignState,
    events: &[EncodedPendingEvent],
    resolution_explanation: &str,
) -> Result<CommitReceipt, JournalStoreError> {
    ensure_supported_state(next_state)?;
    if command_meta.campaign_id != next_state.campaign_id() {
        return Err(JournalStoreError::CampaignMismatch);
    }
    if events.is_empty() {
        return Err(JournalStoreError::EmptyEventBatch);
    }
    if resolution_explanation.trim().is_empty() {
        return Err(JournalStoreError::EmptyResolutionExplanation);
    }

    let next_state_json = next_state
        .encode_json()
        .map_err(|error| JournalStoreError::StateSerialization(error.to_string()))?;
    let mut transaction = pool.begin().await?;
    let campaign_id = command_meta.campaign_id.0.to_string();

    let current_row = sqlx::query(
        "SELECT campaign_id, schema_version, applied_event_sequence, state_json FROM campaign_state_current WHERE campaign_id = ?",
    )
    .bind(&campaign_id)
    .fetch_optional(&mut *transaction)
    .await?;
    let Some(current_row) = current_row else {
        return Err(JournalStoreError::StateNotInitialized);
    };
    let current_state = decode_state_row(&current_row)?;

    let head = sqlx::query(
        "SELECT COUNT(*) AS event_count, COALESCE(MAX(sequence), 0) AS max_sequence FROM event_journal WHERE campaign_id = ?",
    )
    .bind(&campaign_id)
    .fetch_one(&mut *transaction)
    .await?;
    verify_journal_head(&current_state, &head)?;

    if current_state.applied_event_sequence != command_meta.expected_event_sequence {
        return Err(JournalStoreError::StaleState {
            expected: command_meta.expected_event_sequence,
            actual: current_state.applied_event_sequence,
        });
    }

    validate_command_authority(&current_state, command_meta)?;
    validate_session(&mut transaction, command_meta).await?;

    let duplicate_command =
        sqlx::query_scalar::<_, String>("SELECT campaign_id FROM command_audit WHERE id = ?")
            .bind(command_meta.id.0.to_string())
            .fetch_optional(&mut *transaction)
            .await?;
    if duplicate_command.is_some() {
        return Err(JournalStoreError::DuplicateCommandId(command_meta.id));
    }

    let event_count =
        u64::try_from(events.len()).map_err(|_| JournalStoreError::SequenceOverflow)?;
    let target_sequence = current_state
        .applied_event_sequence
        .checked_add(event_count)
        .ok_or(JournalStoreError::SequenceOverflow)?;
    let _ = sequence_to_i64(target_sequence)?;
    if next_state.applied_event_sequence != target_sequence {
        return Err(JournalStoreError::NextStateSequenceMismatch {
            expected: target_sequence,
            actual: next_state.applied_event_sequence,
        });
    }

    let mut event_positions = HashMap::new();
    for (index, event) in events.iter().enumerate() {
        if event_positions.insert(event.id, index).is_some() {
            return Err(JournalStoreError::DuplicateEventId(event.id));
        }
        if let Some(actor) = event.actor
            && !agent_exists(next_state, actor)
        {
            return Err(JournalStoreError::MissingEventActor(actor));
        }

        let existing =
            sqlx::query_scalar::<_, String>("SELECT campaign_id FROM event_journal WHERE id = ?")
                .bind(event.id.0.to_string())
                .fetch_optional(&mut *transaction)
                .await?;
        if existing.is_some() {
            return Err(JournalStoreError::EventAlreadyExists(event.id));
        }
    }

    validate_state_event_references(
        &mut transaction,
        command_meta.campaign_id,
        next_state,
        &event_positions,
    )
    .await?;

    for (index, event) in events.iter().enumerate() {
        let child_sequence = sequence_at(current_state.applied_event_sequence, index)?;
        let mut unique_causes = HashSet::new();
        for cause in &event.caused_by_event_ids {
            if !unique_causes.insert(*cause) {
                return Err(JournalStoreError::DuplicateCausalParent(*cause));
            }
            if let Some(parent_index) = event_positions.get(cause) {
                if *parent_index >= index {
                    return Err(JournalStoreError::FutureCausalParent {
                        event_id: event.id,
                        parent_id: *cause,
                    });
                }
                continue;
            }

            let parent =
                sqlx::query("SELECT campaign_id, sequence FROM event_journal WHERE id = ?")
                    .bind(cause.0.to_string())
                    .fetch_optional(&mut *transaction)
                    .await?;
            let Some(parent) = parent else {
                return Err(JournalStoreError::MissingCausalParent(*cause));
            };
            let parent_campaign: String = parent.try_get("campaign_id")?;
            let parent_campaign =
                CampaignId(parse_uuid(&parent_campaign, "event_journal.campaign_id")?);
            if parent_campaign != command_meta.campaign_id {
                return Err(JournalStoreError::CausalParentCampaignMismatch {
                    event_id: *cause,
                    expected: command_meta.campaign_id,
                    actual: parent_campaign,
                });
            }
            let parent_sequence =
                stored_u64(parent.try_get("sequence")?, "event_journal.sequence")?;
            if parent_sequence >= child_sequence {
                return Err(JournalStoreError::FutureCausalParent {
                    event_id: event.id,
                    parent_id: *cause,
                });
            }
        }
    }

    let updated = sqlx::query(
        "UPDATE campaign_state_current SET schema_version = ?, applied_event_sequence = ?, state_json = ? WHERE campaign_id = ? AND applied_event_sequence = ?",
    )
    .bind(i64::from(next_state.schema_version))
    .bind(sequence_to_i64(target_sequence)?)
    .bind(&next_state_json)
    .bind(&campaign_id)
    .bind(sequence_to_i64(current_state.applied_event_sequence)?)
    .execute(&mut *transaction)
    .await?;
    if updated.rows_affected() != 1 {
        transaction.rollback().await?;
        return Err(JournalStoreError::ConcurrentWrite);
    }

    let (issuer_kind, issuer_player_id) = encode_issuer(command_meta.issuer);
    let (command_actor_kind, command_actor_id) = encode_agent(command_meta.actor);
    sqlx::query(
        r#"
        INSERT INTO command_audit (
            id, campaign_id, session_id, issuer_kind, issuer_player_id,
            actor_kind, actor_id, expected_event_sequence, command_kind,
            command_schema_version, payload_json, accepted, resolution_explanation,
            resulting_event_sequence
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 1, ?, ?)
        "#,
    )
    .bind(command_meta.id.0.to_string())
    .bind(&campaign_id)
    .bind(command_meta.session_id.map(|id| id.0.to_string()))
    .bind(issuer_kind)
    .bind(issuer_player_id)
    .bind(command_actor_kind)
    .bind(command_actor_id)
    .bind(sequence_to_i64(command_meta.expected_event_sequence)?)
    .bind(command_payload.kind())
    .bind(i64::from(command_payload.schema_version()))
    .bind(command_payload.json())
    .bind(resolution_explanation)
    .bind(sequence_to_i64(target_sequence)?)
    .execute(&mut *transaction)
    .await?;

    for (index, event) in events.iter().enumerate() {
        let sequence = sequence_at(current_state.applied_event_sequence, index)?;
        let (actor_kind, actor_id) = encode_agent(event.actor);
        sqlx::query(
            r#"
            INSERT INTO event_journal (
                id, campaign_id, sequence, session_id, occurred_at_world, source,
                actor_kind, actor_id, command_id, event_kind, event_schema_version, payload_json
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(event.id.0.to_string())
        .bind(&campaign_id)
        .bind(sequence_to_i64(sequence)?)
        .bind(command_meta.session_id.map(|id| id.0.to_string()))
        .bind(event.occurred_at.0)
        .bind(encode_event_source(&event.source))
        .bind(actor_kind)
        .bind(actor_id)
        .bind(command_meta.id.0.to_string())
        .bind(event.payload.kind())
        .bind(i64::from(event.payload.schema_version()))
        .bind(event.payload.json())
        .execute(&mut *transaction)
        .await?;

        for (ordinal, cause) in event.caused_by_event_ids.iter().enumerate() {
            let ordinal =
                i64::try_from(ordinal).map_err(|_| JournalStoreError::SequenceOverflow)?;
            sqlx::query(
                "INSERT INTO event_causes (campaign_id, event_id, cause_event_id, ordinal) VALUES (?, ?, ?, ?)",
            )
            .bind(&campaign_id)
            .bind(event.id.0.to_string())
            .bind(cause.0.to_string())
            .bind(ordinal)
            .execute(&mut *transaction)
            .await?;
        }
    }

    transaction.commit().await?;

    Ok(CommitReceipt {
        command_id: command_meta.id,
        emitted_event_ids: events.iter().map(|event| event.id).collect(),
        resulting_event_sequence: target_sequence,
    })
}

pub async fn load_journal_events(
    pool: &SqlitePool,
    campaign_id: CampaignId,
    after_sequence: u64,
) -> Result<Vec<StoredJournalEvent>, JournalStoreError> {
    let rows = sqlx::query(
        r#"
        SELECT id, campaign_id, sequence, session_id, occurred_at_world, source,
               actor_kind, actor_id, command_id, event_kind, event_schema_version, payload_json
        FROM event_journal
        WHERE campaign_id = ? AND sequence > ?
        ORDER BY sequence ASC
        "#,
    )
    .bind(campaign_id.0.to_string())
    .bind(sequence_to_i64(after_sequence)?)
    .fetch_all(pool)
    .await?;

    let mut events = Vec::with_capacity(rows.len());
    for row in rows {
        let event_id_string: String = row.try_get("id")?;
        let event_id = EventId(parse_uuid(&event_id_string, "event_journal.id")?);
        let cause_rows = sqlx::query(
            "SELECT cause_event_id FROM event_causes WHERE campaign_id = ? AND event_id = ? ORDER BY ordinal ASC",
        )
        .bind(campaign_id.0.to_string())
        .bind(&event_id_string)
        .fetch_all(pool)
        .await?;
        let mut causes = Vec::with_capacity(cause_rows.len());
        for cause_row in cause_rows {
            let value: String = cause_row.try_get("cause_event_id")?;
            causes.push(EventId(parse_uuid(&value, "event_causes.cause_event_id")?));
        }

        let stored_campaign: String = row.try_get("campaign_id")?;
        let stored_campaign =
            CampaignId(parse_uuid(&stored_campaign, "event_journal.campaign_id")?);
        let session_id = parse_optional_play_session(row.try_get("session_id")?)?;
        let actor = decode_agent(row.try_get("actor_kind")?, row.try_get("actor_id")?)?;
        let command_id: String = row.try_get("command_id")?;
        let source: String = row.try_get("source")?;

        events.push(StoredJournalEvent {
            meta: EventMeta {
                id: event_id,
                campaign_id: stored_campaign,
                session_id,
                sequence: stored_u64(row.try_get("sequence")?, "event_journal.sequence")?,
                occurred_at: WorldInstant(row.try_get("occurred_at_world")?),
                source: decode_event_source(&source)?,
                actor,
                caused_by_event_ids: causes,
                command_id: Some(CommandId(parse_uuid(
                    &command_id,
                    "event_journal.command_id",
                )?)),
            },
            payload: StoredPayload {
                kind: row.try_get("event_kind")?,
                schema_version: stored_u32(
                    row.try_get("event_schema_version")?,
                    "event_journal.event_schema_version",
                )?,
                json: row.try_get("payload_json")?,
            },
        });
    }

    Ok(events)
}

pub async fn load_command_audit(
    pool: &SqlitePool,
    command_id: CommandId,
) -> Result<Option<StoredCommandAudit>, JournalStoreError> {
    let row = sqlx::query(
        r#"
        SELECT id, campaign_id, session_id, issuer_kind, issuer_player_id,
               actor_kind, actor_id, expected_event_sequence, command_kind,
               command_schema_version, payload_json, accepted, resolution_explanation,
               resulting_event_sequence
        FROM command_audit
        WHERE id = ?
        "#,
    )
    .bind(command_id.0.to_string())
    .fetch_optional(pool)
    .await?;
    let Some(row) = row else {
        return Ok(None);
    };

    let campaign_string: String = row.try_get("campaign_id")?;
    let campaign_id = CampaignId(parse_uuid(&campaign_string, "command_audit.campaign_id")?);
    let issuer_kind: String = row.try_get("issuer_kind")?;
    let issuer_player_id: Option<String> = row.try_get("issuer_player_id")?;
    let actor = decode_agent(row.try_get("actor_kind")?, row.try_get("actor_id")?)?;
    let session_id = parse_optional_play_session(row.try_get("session_id")?)?;
    let emitted_rows = sqlx::query(
        "SELECT id FROM event_journal WHERE campaign_id = ? AND command_id = ? ORDER BY sequence ASC",
    )
    .bind(&campaign_string)
    .bind(command_id.0.to_string())
    .fetch_all(pool)
    .await?;
    let mut emitted_event_ids = Vec::with_capacity(emitted_rows.len());
    for emitted_row in emitted_rows {
        let value: String = emitted_row.try_get("id")?;
        emitted_event_ids.push(EventId(parse_uuid(&value, "event_journal.id")?));
    }

    let accepted: i64 = row.try_get("accepted")?;
    Ok(Some(StoredCommandAudit {
        meta: CommandMeta {
            id: command_id,
            campaign_id,
            session_id,
            issuer: decode_issuer(&issuer_kind, issuer_player_id.as_deref())?,
            actor,
            expected_event_sequence: stored_u64(
                row.try_get("expected_event_sequence")?,
                "command_audit.expected_event_sequence",
            )?,
        },
        payload: StoredPayload {
            kind: row.try_get("command_kind")?,
            schema_version: stored_u32(
                row.try_get("command_schema_version")?,
                "command_audit.command_schema_version",
            )?,
            json: row.try_get("payload_json")?,
        },
        accepted: accepted == 1,
        resolution_explanation: row.try_get("resolution_explanation")?,
        resulting_event_sequence: stored_u64(
            row.try_get("resulting_event_sequence")?,
            "command_audit.resulting_event_sequence",
        )?,
        emitted_event_ids,
    }))
}

fn ensure_supported_state(state: &CampaignState) -> Result<(), JournalStoreError> {
    if state.schema_version != CURRENT_STATE_SCHEMA_VERSION {
        return Err(JournalStoreError::UnsupportedStateSchema {
            actual: state.schema_version,
            supported: CURRENT_STATE_SCHEMA_VERSION,
        });
    }
    if !state.validate().is_empty() {
        return Err(JournalStoreError::InvalidState);
    }
    Ok(())
}

fn decode_state_row(row: &SqliteRow) -> Result<CampaignState, JournalStoreError> {
    let campaign_id: String = row.try_get("campaign_id")?;
    let campaign_id = CampaignId(parse_uuid(
        &campaign_id,
        "campaign_state_current.campaign_id",
    )?);
    let schema_version = stored_u32(
        row.try_get("schema_version")?,
        "campaign_state_current.schema_version",
    )?;
    let sequence = stored_u64(
        row.try_get("applied_event_sequence")?,
        "campaign_state_current.applied_event_sequence",
    )?;
    let state_json: String = row.try_get("state_json")?;
    let state = CampaignState::decode_json(&state_json)
        .map_err(|error| JournalStoreError::CorruptState(format!("invalid JSON: {error}")))?;

    if state.campaign_id() != campaign_id {
        return Err(JournalStoreError::CorruptState(
            "row campaign_id does not match serialized state".into(),
        ));
    }
    if state.schema_version != schema_version {
        return Err(JournalStoreError::CorruptState(
            "row schema_version does not match serialized state".into(),
        ));
    }
    if state.applied_event_sequence != sequence {
        return Err(JournalStoreError::CorruptState(
            "row event sequence does not match serialized state".into(),
        ));
    }
    if state.schema_version != CURRENT_STATE_SCHEMA_VERSION {
        return Err(JournalStoreError::UnsupportedStateSchema {
            actual: state.schema_version,
            supported: CURRENT_STATE_SCHEMA_VERSION,
        });
    }
    if !state.validate().is_empty() {
        return Err(JournalStoreError::CorruptState(
            "serialized state violates domain invariants".into(),
        ));
    }

    Ok(state)
}

fn verify_journal_head(state: &CampaignState, row: &SqliteRow) -> Result<(), JournalStoreError> {
    let event_count = stored_u64(row.try_get("event_count")?, "event_journal.count")?;
    let max_sequence = stored_u64(row.try_get("max_sequence")?, "event_journal.max_sequence")?;
    if event_count != state.applied_event_sequence || max_sequence != state.applied_event_sequence {
        return Err(JournalStoreError::CorruptJournalHead {
            state_sequence: state.applied_event_sequence,
            event_count,
            max_sequence,
        });
    }
    Ok(())
}

async fn verify_loaded_state_event_references(
    pool: &SqlitePool,
    campaign_id: CampaignId,
    state: &CampaignState,
) -> Result<(), JournalStoreError> {
    for event_id in collect_state_event_references(state) {
        let row = sqlx::query("SELECT campaign_id FROM event_journal WHERE id = ?")
            .bind(event_id.0.to_string())
            .fetch_optional(pool)
            .await?;
        let Some(row) = row else {
            return Err(JournalStoreError::MissingStateEventReference(event_id));
        };
        let stored_campaign: String = row.try_get("campaign_id")?;
        let actual = CampaignId(parse_uuid(&stored_campaign, "event_journal.campaign_id")?);
        if actual != campaign_id {
            return Err(JournalStoreError::StateEventReferenceCampaignMismatch {
                event_id,
                expected: campaign_id,
                actual,
            });
        }
    }
    Ok(())
}

fn validate_command_authority(
    state: &CampaignState,
    meta: &CommandMeta,
) -> Result<(), JournalStoreError> {
    if let CommandIssuer::Player(player_id) = meta.issuer
        && !state.players.contains_key(&player_id)
    {
        return Err(JournalStoreError::MissingIssuerPlayer(player_id));
    }
    if let Some(actor) = meta.actor
        && !agent_exists(state, actor)
    {
        return Err(JournalStoreError::MissingCommandActor(actor));
    }
    Ok(())
}

fn agent_exists(state: &CampaignState, agent: AgentRef) -> bool {
    match agent {
        AgentRef::Entity(entity_id) => state.entities.contains_key(&entity_id),
        AgentRef::Faction(faction_id) => state.factions.contains_key(&faction_id),
    }
}

async fn validate_session(
    transaction: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    meta: &CommandMeta,
) -> Result<(), JournalStoreError> {
    let Some(session_id) = meta.session_id else {
        return Ok(());
    };

    let row = sqlx::query("SELECT campaign_id FROM play_sessions WHERE id = ?")
        .bind(session_id.0.to_string())
        .fetch_optional(&mut **transaction)
        .await?;
    let Some(row) = row else {
        return Err(JournalStoreError::MissingSession(session_id));
    };
    let campaign_id: String = row.try_get("campaign_id")?;
    let actual = CampaignId(parse_uuid(&campaign_id, "play_sessions.campaign_id")?);
    if actual != meta.campaign_id {
        return Err(JournalStoreError::SessionCampaignMismatch {
            session_id,
            expected: meta.campaign_id,
            actual,
        });
    }
    Ok(())
}

async fn validate_state_event_references(
    transaction: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    campaign_id: CampaignId,
    state: &CampaignState,
    pending_events: &HashMap<EventId, usize>,
) -> Result<(), JournalStoreError> {
    for event_id in collect_state_event_references(state) {
        if pending_events.contains_key(&event_id) {
            continue;
        }
        let row = sqlx::query("SELECT campaign_id FROM event_journal WHERE id = ?")
            .bind(event_id.0.to_string())
            .fetch_optional(&mut **transaction)
            .await?;
        let Some(row) = row else {
            return Err(JournalStoreError::MissingStateEventReference(event_id));
        };
        let stored_campaign: String = row.try_get("campaign_id")?;
        let actual = CampaignId(parse_uuid(&stored_campaign, "event_journal.campaign_id")?);
        if actual != campaign_id {
            return Err(JournalStoreError::StateEventReferenceCampaignMismatch {
                event_id,
                expected: campaign_id,
                actual,
            });
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

fn sequence_at(base: u64, index: usize) -> Result<u64, JournalStoreError> {
    let offset = u64::try_from(index)
        .map_err(|_| JournalStoreError::SequenceOverflow)?
        .checked_add(1)
        .ok_or(JournalStoreError::SequenceOverflow)?;
    base.checked_add(offset)
        .ok_or(JournalStoreError::SequenceOverflow)
}

fn sequence_to_i64(value: u64) -> Result<i64, JournalStoreError> {
    i64::try_from(value).map_err(|_| JournalStoreError::SequenceOverflow)
}

fn stored_u64(value: i64, field: &'static str) -> Result<u64, JournalStoreError> {
    u64::try_from(value).map_err(|_| JournalStoreError::InvalidInteger { field, value })
}

fn stored_u32(value: i64, field: &'static str) -> Result<u32, JournalStoreError> {
    u32::try_from(value).map_err(|_| JournalStoreError::InvalidInteger { field, value })
}

fn parse_uuid(value: &str, field: &'static str) -> Result<Uuid, JournalStoreError> {
    Uuid::parse_str(value).map_err(|source| JournalStoreError::InvalidUuid { field, source })
}

fn parse_optional_play_session(
    value: Option<String>,
) -> Result<Option<PlaySessionId>, JournalStoreError> {
    value
        .as_deref()
        .map(|value| parse_uuid(value, "session_id").map(PlaySessionId))
        .transpose()
}

fn encode_issuer(issuer: CommandIssuer) -> (&'static str, Option<String>) {
    match issuer {
        CommandIssuer::Player(player_id) => ("player", Some(player_id.0.to_string())),
        CommandIssuer::System => ("system", None),
        CommandIssuer::Admin => ("admin", None),
        CommandIssuer::Import => ("import", None),
    }
}

fn decode_issuer(kind: &str, player_id: Option<&str>) -> Result<CommandIssuer, JournalStoreError> {
    match (kind, player_id) {
        ("player", Some(player_id)) => Ok(CommandIssuer::Player(PlayerId(parse_uuid(
            player_id,
            "command_audit.issuer_player_id",
        )?))),
        ("system", None) => Ok(CommandIssuer::System),
        ("admin", None) => Ok(CommandIssuer::Admin),
        ("import", None) => Ok(CommandIssuer::Import),
        _ => Err(JournalStoreError::InvalidIssuer(kind.into())),
    }
}

fn encode_agent(agent: Option<AgentRef>) -> (Option<&'static str>, Option<String>) {
    match agent {
        Some(AgentRef::Entity(entity_id)) => (Some("entity"), Some(entity_id.0.to_string())),
        Some(AgentRef::Faction(faction_id)) => (Some("faction"), Some(faction_id.0.to_string())),
        None => (None, None),
    }
}

fn decode_agent(
    kind: Option<String>,
    id: Option<String>,
) -> Result<Option<AgentRef>, JournalStoreError> {
    match (kind.as_deref(), id.as_deref()) {
        (None, None) => Ok(None),
        (Some("entity"), Some(id)) => Ok(Some(AgentRef::Entity(EntityId(parse_uuid(
            id, "actor_id",
        )?)))),
        (Some("faction"), Some(id)) => Ok(Some(AgentRef::Faction(FactionId(parse_uuid(
            id, "actor_id",
        )?)))),
        _ => Err(JournalStoreError::InvalidActor),
    }
}

fn encode_event_source(source: &EventSource) -> &'static str {
    match source {
        EventSource::PlayerAction => "player_action",
        EventSource::RuleResolution => "rule_resolution",
        EventSource::WorldSimulation => "world_simulation",
        EventSource::ProceduralGeneration => "procedural_generation",
        EventSource::AdminCorrection => "admin_correction",
        EventSource::Import => "import",
    }
}

fn decode_event_source(value: &str) -> Result<EventSource, JournalStoreError> {
    match value {
        "player_action" => Ok(EventSource::PlayerAction),
        "rule_resolution" => Ok(EventSource::RuleResolution),
        "world_simulation" => Ok(EventSource::WorldSimulation),
        "procedural_generation" => Ok(EventSource::ProceduralGeneration),
        "admin_correction" => Ok(EventSource::AdminCorrection),
        "import" => Ok(EventSource::Import),
        other => Err(JournalStoreError::InvalidEventSource(other.into())),
    }
}
