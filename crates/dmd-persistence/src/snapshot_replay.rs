use std::collections::{HashMap, HashSet};

use dmd_domain::{
    AgentRef, BeliefBasis, CURRENT_STATE_SCHEMA_VERSION, CampaignId, CampaignState, CommandId,
    EntityId, EventId, EventMeta, EventSource, FactionId, PlaySessionId, WorldInstant,
};
use sqlx::{
    Row, SqlitePool,
    sqlite::{SqliteConnection, SqliteRow},
};
use thiserror::Error;
use uuid::Uuid;

use crate::journal_store::{StoredJournalEvent, StoredPayload};

const EVENT_LOOKUP_BATCH_SIZE: usize = 400;
pub const SNAPSHOT_EVENT_INTERVAL: u64 = 100;

pub trait SnapshotMigration: Send + Sync {
    fn source_version(&self) -> u32;
    fn migrate_json(&self, json: &str) -> Result<String, String>;
}

pub struct CampaignStateSnapshotCodec {
    migrations: HashMap<u32, Box<dyn SnapshotMigration>>,
}

impl Default for CampaignStateSnapshotCodec {
    fn default() -> Self {
        let mut migrations: HashMap<u32, Box<dyn SnapshotMigration>> = HashMap::new();
        migrations.insert(1, Box::new(StateSchemaOneToTwo));
        migrations.insert(2, Box::new(StateSchemaTwoToThree));
        migrations.insert(3, Box::new(StateSchemaThreeToFour));
        Self { migrations }
    }
}

struct StateSchemaOneToTwo;

struct StateSchemaTwoToThree;

struct StateSchemaThreeToFour;

impl SnapshotMigration for StateSchemaThreeToFour {
    fn source_version(&self) -> u32 {
        3
    }

    fn migrate_json(&self, json: &str) -> Result<String, String> {
        let legacy = CampaignState::decode_json(json).map_err(|error| error.to_string())?;
        reject_legacy_encounter(&legacy)?;
        if legacy.schema_version != 3 || !legacy.validate().is_empty() {
            return Err("schema-3 state is structurally invalid".into());
        }
        let mut value: serde_json::Value =
            serde_json::from_str(json).map_err(|error| error.to_string())?;
        let object = value
            .as_object_mut()
            .ok_or("schema-3 state must be an object")?;
        object.insert("schema_version".into(), serde_json::json!(4));
        object.insert("encounter".into(), serde_json::Value::Null);
        serde_json::to_string(&value).map_err(|error| error.to_string())
    }
}

/// Legacy versions must not acquire future authority merely because their JSON has extra fields.
/// Decode the typed shape first so duplicate authoritative fields cannot be hidden by `Value`.
fn reject_legacy_encounter(legacy: &CampaignState) -> Result<(), String> {
    if legacy.encounter.is_some()
        || legacy.rules.as_ref().is_some_and(|rules| {
            rules.tactical_effects.is_some()
                || rules.tactical_inventory.is_some()
                || rules.tactical_creatures.is_some()
        })
    {
        return Err("legacy state unexpectedly contains tactical encounter data".into());
    }
    Ok(())
}

pub(crate) fn preflight_legacy_authority(json: &str) -> Result<(), String> {
    // Typed fields reject duplicates, including a second null hiding a first value.
    // Established legacy preflights retain their original migration checksums.
    #[derive(serde::Deserialize)]
    struct Probe {
        rules: Option<RulesProbe>,
    }
    #[derive(serde::Deserialize)]
    struct RulesProbe {
        tactical_effects: Option<serde_json::Value>,
        tactical_inventory: Option<serde_json::Value>,
        tactical_creatures: Option<serde_json::Value>,
    }
    let probe: Probe = serde_json::from_str(json).map_err(|error| error.to_string())?;
    if probe.rules.is_some_and(|rules| {
        rules.tactical_effects.is_some()
            || rules.tactical_inventory.is_some()
            || rules.tactical_creatures.is_some()
    }) {
        return Err("legacy state unexpectedly contains tactical setup authority".into());
    }
    Ok(())
}

impl SnapshotMigration for StateSchemaTwoToThree {
    fn source_version(&self) -> u32 {
        2
    }

    fn migrate_json(&self, json: &str) -> Result<String, String> {
        let legacy = CampaignState::decode_json(json).map_err(|error| error.to_string())?;
        reject_legacy_encounter(&legacy)?;
        if legacy.schema_version != 2 || !legacy.validate().is_empty() {
            return Err("schema-2 state is structurally invalid".into());
        }
        if legacy.table.is_some() {
            return Err("schema-2 state unexpectedly contains table data".into());
        }
        let mut value: serde_json::Value =
            serde_json::from_str(json).map_err(|error| error.to_string())?;
        let object = value
            .as_object_mut()
            .ok_or("schema-2 state must be an object")?;
        object.insert("schema_version".into(), serde_json::json!(3));
        object.insert("table".into(), serde_json::Value::Null);
        serde_json::to_string(&value).map_err(|error| error.to_string())
    }
}

impl SnapshotMigration for StateSchemaOneToTwo {
    fn source_version(&self) -> u32 {
        1
    }

    fn migrate_json(&self, json: &str) -> Result<String, String> {
        upgrade_state_schema_one(json)
    }
}

/// Explicit schema-1 compatibility, shared by snapshot and portable-export decoding.
/// Historical input is never edited, and unexpected mechanical data is never discarded.
pub(crate) fn upgrade_state_schema_one(json: &str) -> Result<String, String> {
    // Decode the original typed shape before going through Value: a map conversion alone would
    // silently collapse duplicate fields and could conceal malformed legacy authoritative input.
    let legacy = CampaignState::decode_json(json).map_err(|error| error.to_string())?;
    reject_legacy_encounter(&legacy)?;
    if !legacy.validate().is_empty() {
        return Err("schema-1 state is structurally invalid".into());
    }
    let mut value: serde_json::Value =
        serde_json::from_str(json).map_err(|error| error.to_string())?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| "schema-1 state must be a JSON object".to_owned())?;
    if object
        .get("schema_version")
        .and_then(serde_json::Value::as_u64)
        != Some(1)
    {
        return Err("schema-1 state metadata does not match its embedded schema".into());
    }
    if object.get("rules").is_some_and(|rules| !rules.is_null()) {
        return Err("schema-1 state unexpectedly contains mechanical rules data".into());
    }
    object.insert("schema_version".into(), serde_json::json!(2));
    object.insert("rules".into(), serde_json::Value::Null);
    serde_json::to_string(&value).map_err(|error| error.to_string())
}

impl CampaignStateSnapshotCodec {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Keep unknown JSON data intact while applying the same validated migration chain as anchors.
    pub(crate) fn upgraded_json(
        &self,
        stored_version: u32,
        json: &str,
    ) -> Result<String, SnapshotCodecError> {
        self.decode_state(stored_version, json)?;
        let mut upgraded = json.to_owned();
        for version in stored_version..CURRENT_STATE_SCHEMA_VERSION {
            let migration =
                self.migrations
                    .get(&version)
                    .ok_or(SnapshotCodecError::MissingMigration {
                        from_version: version,
                    })?;
            upgraded = migration.migrate_json(&upgraded).map_err(|message| {
                SnapshotCodecError::MigrationFailed {
                    from_version: version,
                    message,
                }
            })?;
        }
        Ok(upgraded)
    }

    pub fn register<M>(&mut self, migration: M) -> Result<(), SnapshotCodecError>
    where
        M: SnapshotMigration + 'static,
    {
        let from_version = migration.source_version();
        if from_version >= CURRENT_STATE_SCHEMA_VERSION {
            return Err(SnapshotCodecError::InvalidMigrationRegistration {
                from_version,
                current_version: CURRENT_STATE_SCHEMA_VERSION,
            });
        }
        if self.migrations.contains_key(&from_version) {
            return Err(SnapshotCodecError::DuplicateMigration { from_version });
        }
        self.migrations.insert(from_version, Box::new(migration));
        Ok(())
    }

    pub fn decode_state(
        &self,
        stored_version: u32,
        json: &str,
    ) -> Result<CampaignState, SnapshotCodecError> {
        if stored_version > CURRENT_STATE_SCHEMA_VERSION {
            return Err(SnapshotCodecError::UnsupportedFutureVersion {
                actual: stored_version,
                supported: CURRENT_STATE_SCHEMA_VERSION,
            });
        }

        let mut version = stored_version;
        let mut migrated_json = json.to_owned();
        while version < CURRENT_STATE_SCHEMA_VERSION {
            let Some(migration) = self.migrations.get(&version) else {
                return Err(SnapshotCodecError::MissingMigration {
                    from_version: version,
                });
            };
            migrated_json = migration.migrate_json(&migrated_json).map_err(|message| {
                SnapshotCodecError::MigrationFailed {
                    from_version: version,
                    message,
                }
            })?;
            version = version
                .checked_add(1)
                .ok_or(SnapshotCodecError::VersionOverflow)?;
        }

        let state = CampaignState::decode_json(&migrated_json)
            .map_err(|error| SnapshotCodecError::Decode(error.to_string()))?;
        if state.schema_version != CURRENT_STATE_SCHEMA_VERSION {
            return Err(SnapshotCodecError::DecodedSchemaMismatch {
                actual: state.schema_version,
                expected: CURRENT_STATE_SCHEMA_VERSION,
            });
        }
        if !state.validate().is_empty() {
            return Err(SnapshotCodecError::InvalidState);
        }
        Ok(state)
    }
}

#[derive(Debug, Error)]
pub enum SnapshotCodecError {
    #[error(
        "snapshot schema version {actual} is newer than supported campaign-state version {supported}"
    )]
    UnsupportedFutureVersion { actual: u32, supported: u32 },
    #[error("no snapshot migration is registered from schema version {from_version}")]
    MissingMigration { from_version: u32 },
    #[error("snapshot migration from schema version {from_version} is registered more than once")]
    DuplicateMigration { from_version: u32 },
    #[error(
        "snapshot migration from version {from_version} is invalid for current version {current_version}"
    )]
    InvalidMigrationRegistration {
        from_version: u32,
        current_version: u32,
    },
    #[error("snapshot migration from version {from_version} failed: {message}")]
    MigrationFailed { from_version: u32, message: String },
    #[error("snapshot schema version overflow")]
    VersionOverflow,
    #[error("snapshot JSON could not be decoded: {0}")]
    Decode(String),
    #[error("decoded snapshot schema version is {actual}, expected {expected}")]
    DecodedSchemaMismatch { actual: u32, expected: u32 },
    #[error("decoded snapshot violates campaign-state domain invariants")]
    InvalidState,
}

#[derive(Debug, Error)]
pub enum ReplayApplyError {
    #[error("unsupported event kind/version: {kind}@{schema_version}")]
    UnsupportedEvent { kind: String, schema_version: u32 },
    #[error("invalid event payload for {kind}@{schema_version}: {message}")]
    InvalidPayload {
        kind: String,
        schema_version: u32,
        message: String,
    },
    #[error("event could not be applied: {0}")]
    InvalidTransition(String),
}

pub trait ReplayEventApplier: Send + Sync {
    fn apply(
        &self,
        state: &mut CampaignState,
        event: &StoredJournalEvent,
    ) -> Result<(), ReplayApplyError>;
}

#[derive(Debug, Error)]
pub enum SnapshotReplayError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Codec(#[from] SnapshotCodecError),
    #[error("campaign state is not initialized")]
    CampaignNotInitialized,
    #[error("replay target {target} is beyond journal head {head}")]
    TargetBeyondHead { target: u64, head: u64 },
    #[error("no campaign snapshot exists at or before replay target sequence {target}")]
    MissingSnapshot { target: u64 },
    #[error("snapshot campaign does not match requested campaign")]
    SnapshotCampaignMismatch,
    #[error("snapshot state records sequence {state_sequence}, row records {row_sequence}")]
    SnapshotSequenceMismatch {
        state_sequence: u64,
        row_sequence: u64,
    },
    #[error("journal head is inconsistent: count={event_count}, max={max_sequence}")]
    CorruptJournalHead { event_count: u64, max_sequence: u64 },
    #[error(
        "journal prefix through {target} is inconsistent: count={event_count}, max={max_sequence}"
    )]
    CorruptJournalPrefix {
        target: u64,
        event_count: u64,
        max_sequence: u64,
    },
    #[error("replay journal gap: expected sequence {expected}, found {actual:?}")]
    JournalGap { expected: u64, actual: Option<u64> },
    #[error("event sequence {0} exceeds SQLite signed integer range")]
    SequenceOverflow(u64),
    #[error("stored UUID in {field} is invalid: {source}")]
    InvalidUuid {
        field: &'static str,
        #[source]
        source: uuid::Error,
    },
    #[error("stored integer in {field} is invalid: {value}")]
    InvalidInteger { field: &'static str, value: i64 },
    #[error("stored event source is invalid: {0}")]
    InvalidEventSource(String),
    #[error("stored actor metadata is invalid")]
    InvalidActor,
    #[error("causal parent {parent_id:?} does not precede child {event_id:?}")]
    InvalidCausalEdge {
        event_id: EventId,
        parent_id: EventId,
    },
    #[error("replay applier failed at sequence {sequence} for {kind}@{schema_version}: {source}")]
    EventApply {
        sequence: u64,
        kind: String,
        schema_version: u32,
        #[source]
        source: ReplayApplyError,
    },
    #[error("replay event changed campaign identity")]
    CampaignIdentityChanged,
    #[error("replay event changed state schema version to {actual}, expected {expected}")]
    StateSchemaChanged { actual: u32, expected: u32 },
    #[error("replayed state violates campaign-state domain invariants")]
    InvalidReplayedState,
    #[error("replayed state references missing journal event: {0:?}")]
    MissingStateEventReference(EventId),
    #[error("replayed state event reference belongs to another campaign: {0:?}")]
    StateEventReferenceCampaignMismatch(EventId),
    #[error(
        "replayed state event reference {event_id:?} is after target sequence {target}: {sequence}"
    )]
    StateEventReferenceAfterTarget {
        event_id: EventId,
        sequence: u64,
        target: u64,
    },
}

pub async fn load_campaign_snapshot_at_or_before(
    pool: &SqlitePool,
    campaign_id: CampaignId,
    target_sequence: u64,
    codec: &CampaignStateSnapshotCodec,
) -> Result<Option<CampaignState>, SnapshotReplayError> {
    let mut transaction = pool.begin().await?;
    let snapshot = load_snapshot(&mut transaction, campaign_id, target_sequence, codec).await?;
    if let Some((snapshot_sequence, state)) = &snapshot {
        verify_journal_prefix(&mut transaction, campaign_id, *snapshot_sequence).await?;
        validate_state_event_references(&mut transaction, campaign_id, *snapshot_sequence, state)
            .await?;
    }
    transaction.commit().await?;
    Ok(snapshot.map(|(_, state)| state))
}

pub async fn replay_campaign_to_head(
    pool: &SqlitePool,
    campaign_id: CampaignId,
    applier: &dyn ReplayEventApplier,
) -> Result<CampaignState, SnapshotReplayError> {
    let codec = CampaignStateSnapshotCodec::default();
    replay_campaign_to_head_with_codec(pool, campaign_id, &codec, applier).await
}

pub async fn replay_campaign_to_head_with_codec(
    pool: &SqlitePool,
    campaign_id: CampaignId,
    codec: &CampaignStateSnapshotCodec,
    applier: &dyn ReplayEventApplier,
) -> Result<CampaignState, SnapshotReplayError> {
    let mut transaction = pool.begin().await?;
    let head = load_campaign_head(&mut transaction, campaign_id).await?;
    let state = replay_to_target(&mut transaction, campaign_id, head, codec, applier).await?;
    transaction.commit().await?;
    Ok(state)
}

pub async fn replay_campaign_to_sequence(
    pool: &SqlitePool,
    campaign_id: CampaignId,
    target_sequence: u64,
    applier: &dyn ReplayEventApplier,
) -> Result<CampaignState, SnapshotReplayError> {
    let codec = CampaignStateSnapshotCodec::default();
    replay_campaign_to_sequence_with_codec(pool, campaign_id, target_sequence, &codec, applier)
        .await
}

pub async fn replay_campaign_to_sequence_with_codec(
    pool: &SqlitePool,
    campaign_id: CampaignId,
    target_sequence: u64,
    codec: &CampaignStateSnapshotCodec,
    applier: &dyn ReplayEventApplier,
) -> Result<CampaignState, SnapshotReplayError> {
    let mut transaction = pool.begin().await?;
    let head = load_campaign_head(&mut transaction, campaign_id).await?;
    if target_sequence > head {
        return Err(SnapshotReplayError::TargetBeyondHead {
            target: target_sequence,
            head,
        });
    }
    let state = replay_to_target(
        &mut transaction,
        campaign_id,
        target_sequence,
        codec,
        applier,
    )
    .await?;
    transaction.commit().await?;
    Ok(state)
}

async fn replay_to_target(
    connection: &mut SqliteConnection,
    campaign_id: CampaignId,
    target_sequence: u64,
    codec: &CampaignStateSnapshotCodec,
    applier: &dyn ReplayEventApplier,
) -> Result<CampaignState, SnapshotReplayError> {
    verify_journal_prefix(connection, campaign_id, target_sequence).await?;

    let Some((snapshot_sequence, mut state)) =
        load_snapshot(connection, campaign_id, target_sequence, codec).await?
    else {
        return Err(SnapshotReplayError::MissingSnapshot {
            target: target_sequence,
        });
    };

    validate_state_event_references(connection, campaign_id, snapshot_sequence, &state).await?;

    let events =
        load_replay_events(connection, campaign_id, snapshot_sequence, target_sequence).await?;
    let expected_count = target_sequence
        .checked_sub(snapshot_sequence)
        .expect("snapshot is selected at or before target");
    if u64::try_from(events.len()).ok() != Some(expected_count) {
        let actual = events.first().map(|event| event.meta.sequence);
        return Err(SnapshotReplayError::JournalGap {
            expected: snapshot_sequence.saturating_add(1),
            actual,
        });
    }

    for (index, event) in events.iter().enumerate() {
        let expected = snapshot_sequence
            .checked_add(
                u64::try_from(index).map_err(|_| SnapshotReplayError::JournalGap {
                    expected: snapshot_sequence,
                    actual: None,
                })?,
            )
            .and_then(|value| value.checked_add(1))
            .ok_or(SnapshotReplayError::JournalGap {
                expected: snapshot_sequence,
                actual: None,
            })?;
        if event.meta.sequence != expected {
            return Err(SnapshotReplayError::JournalGap {
                expected,
                actual: Some(event.meta.sequence),
            });
        }

        applier
            .apply(&mut state, event)
            .map_err(|source| SnapshotReplayError::EventApply {
                sequence: event.meta.sequence,
                kind: event.payload.kind.clone(),
                schema_version: event.payload.schema_version,
                source,
            })?;

        if state.campaign_id() != campaign_id {
            return Err(SnapshotReplayError::CampaignIdentityChanged);
        }
        if state.schema_version != CURRENT_STATE_SCHEMA_VERSION {
            return Err(SnapshotReplayError::StateSchemaChanged {
                actual: state.schema_version,
                expected: CURRENT_STATE_SCHEMA_VERSION,
            });
        }
        state.applied_event_sequence = event.meta.sequence;
    }

    if state.applied_event_sequence != target_sequence || !state.validate().is_empty() {
        return Err(SnapshotReplayError::InvalidReplayedState);
    }
    validate_state_event_references(connection, campaign_id, target_sequence, &state).await?;
    Ok(state)
}

async fn load_campaign_head(
    connection: &mut SqliteConnection,
    campaign_id: CampaignId,
) -> Result<u64, SnapshotReplayError> {
    let row = sqlx::query(
        r#"
        SELECT COUNT(journal.sequence) AS event_count,
               COALESCE(MAX(journal.sequence), 0) AS max_sequence
        FROM campaign_state_current AS current
        LEFT JOIN event_journal AS journal
          ON journal.campaign_id = current.campaign_id
        WHERE current.campaign_id = ?
        GROUP BY current.campaign_id
        "#,
    )
    .bind(campaign_id.0.to_string())
    .fetch_optional(&mut *connection)
    .await?;
    let Some(row) = row else {
        return Err(SnapshotReplayError::CampaignNotInitialized);
    };
    let event_count = stored_u64(row.try_get("event_count")?, "event_journal.count")?;
    let max_sequence = stored_u64(row.try_get("max_sequence")?, "event_journal.max_sequence")?;
    if event_count != max_sequence {
        return Err(SnapshotReplayError::CorruptJournalHead {
            event_count,
            max_sequence,
        });
    }
    Ok(max_sequence)
}

async fn load_snapshot(
    connection: &mut SqliteConnection,
    campaign_id: CampaignId,
    target_sequence: u64,
    codec: &CampaignStateSnapshotCodec,
) -> Result<Option<(u64, CampaignState)>, SnapshotReplayError> {
    let row = sqlx::query(
        r#"
        SELECT campaign_id, event_sequence, state_schema_version, state_json
        FROM campaign_snapshots
        WHERE campaign_id = ? AND event_sequence <= ?
        ORDER BY event_sequence DESC
        LIMIT 1
        "#,
    )
    .bind(campaign_id.0.to_string())
    .bind(sequence_to_i64(target_sequence)?)
    .fetch_optional(&mut *connection)
    .await?;
    let Some(row) = row else {
        return Ok(None);
    };

    let stored_campaign: String = row.try_get("campaign_id")?;
    let stored_campaign = CampaignId(parse_uuid(
        &stored_campaign,
        "campaign_snapshots.campaign_id",
    )?);
    if stored_campaign != campaign_id {
        return Err(SnapshotReplayError::SnapshotCampaignMismatch);
    }
    let row_sequence = stored_u64(
        row.try_get("event_sequence")?,
        "campaign_snapshots.event_sequence",
    )?;
    let schema_version = stored_u32(
        row.try_get("state_schema_version")?,
        "campaign_snapshots.state_schema_version",
    )?;
    let state_json: String = row.try_get("state_json")?;
    let state = codec.decode_state(schema_version, &state_json)?;
    if state.campaign_id() != campaign_id {
        return Err(SnapshotReplayError::SnapshotCampaignMismatch);
    }
    if state.applied_event_sequence != row_sequence {
        return Err(SnapshotReplayError::SnapshotSequenceMismatch {
            state_sequence: state.applied_event_sequence,
            row_sequence,
        });
    }
    Ok(Some((row_sequence, state)))
}

async fn verify_journal_prefix(
    connection: &mut SqliteConnection,
    campaign_id: CampaignId,
    target_sequence: u64,
) -> Result<(), SnapshotReplayError> {
    let row = sqlx::query(
        "SELECT COUNT(*) AS event_count, COALESCE(MAX(sequence), 0) AS max_sequence FROM event_journal WHERE campaign_id = ? AND sequence <= ?",
    )
    .bind(campaign_id.0.to_string())
    .bind(sequence_to_i64(target_sequence)?)
    .fetch_one(&mut *connection)
    .await?;
    let event_count = stored_u64(row.try_get("event_count")?, "event_journal.count")?;
    let max_sequence = stored_u64(row.try_get("max_sequence")?, "event_journal.max_sequence")?;
    if event_count != target_sequence || max_sequence != target_sequence {
        return Err(SnapshotReplayError::CorruptJournalPrefix {
            target: target_sequence,
            event_count,
            max_sequence,
        });
    }
    Ok(())
}

async fn load_replay_events(
    connection: &mut SqliteConnection,
    campaign_id: CampaignId,
    after_sequence: u64,
    through_sequence: u64,
) -> Result<Vec<StoredJournalEvent>, SnapshotReplayError> {
    let rows = sqlx::query(
        r#"
        SELECT id, campaign_id, sequence, session_id, occurred_at_world, source,
               actor_kind, actor_id, command_id, event_kind, event_schema_version, payload_json
        FROM event_journal
        WHERE campaign_id = ? AND sequence > ? AND sequence <= ?
        ORDER BY sequence ASC
        "#,
    )
    .bind(campaign_id.0.to_string())
    .bind(sequence_to_i64(after_sequence)?)
    .bind(sequence_to_i64(through_sequence)?)
    .fetch_all(&mut *connection)
    .await?;

    let cause_rows = sqlx::query(
        r#"
        SELECT causes.event_id, causes.cause_event_id,
               child.sequence AS child_sequence, parent.sequence AS parent_sequence
        FROM event_causes AS causes
        JOIN event_journal AS child
          ON child.campaign_id = causes.campaign_id
         AND child.id = causes.event_id
        JOIN event_journal AS parent
          ON parent.campaign_id = causes.campaign_id
         AND parent.id = causes.cause_event_id
        WHERE causes.campaign_id = ? AND child.sequence > ? AND child.sequence <= ?
        ORDER BY child.sequence ASC, causes.ordinal ASC
        "#,
    )
    .bind(campaign_id.0.to_string())
    .bind(sequence_to_i64(after_sequence)?)
    .bind(sequence_to_i64(through_sequence)?)
    .fetch_all(&mut *connection)
    .await?;

    let mut causes_by_event = HashMap::<EventId, Vec<EventId>>::new();
    for row in cause_rows {
        let event_id_string: String = row.try_get("event_id")?;
        let cause_id_string: String = row.try_get("cause_event_id")?;
        let event_id = EventId(parse_uuid(&event_id_string, "event_causes.event_id")?);
        let parent_id = EventId(parse_uuid(&cause_id_string, "event_causes.cause_event_id")?);
        let child_sequence = stored_u64(
            row.try_get("child_sequence")?,
            "event_causes.child_sequence",
        )?;
        let parent_sequence = stored_u64(
            row.try_get("parent_sequence")?,
            "event_causes.parent_sequence",
        )?;
        if parent_sequence >= child_sequence {
            return Err(SnapshotReplayError::InvalidCausalEdge {
                event_id,
                parent_id,
            });
        }
        causes_by_event.entry(event_id).or_default().push(parent_id);
    }

    let mut events = Vec::with_capacity(rows.len());
    for row in rows {
        events.push(decode_journal_row(row, &mut causes_by_event)?);
    }
    Ok(events)
}

fn decode_journal_row(
    row: SqliteRow,
    causes_by_event: &mut HashMap<EventId, Vec<EventId>>,
) -> Result<StoredJournalEvent, SnapshotReplayError> {
    let event_id_string: String = row.try_get("id")?;
    let event_id = EventId(parse_uuid(&event_id_string, "event_journal.id")?);
    let campaign_string: String = row.try_get("campaign_id")?;
    let campaign_id = CampaignId(parse_uuid(&campaign_string, "event_journal.campaign_id")?);
    let session_id = parse_optional_play_session(row.try_get("session_id")?)?;
    let actor = decode_agent(row.try_get("actor_kind")?, row.try_get("actor_id")?)?;
    let command_id: String = row.try_get("command_id")?;
    let source: String = row.try_get("source")?;

    Ok(StoredJournalEvent {
        meta: EventMeta {
            id: event_id,
            campaign_id,
            session_id,
            sequence: stored_u64(row.try_get("sequence")?, "event_journal.sequence")?,
            occurred_at: WorldInstant(row.try_get("occurred_at_world")?),
            source: decode_event_source(&source)?,
            actor,
            caused_by_event_ids: causes_by_event.remove(&event_id).unwrap_or_default(),
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
    })
}

async fn validate_state_event_references(
    connection: &mut SqliteConnection,
    campaign_id: CampaignId,
    target_sequence: u64,
    state: &CampaignState,
) -> Result<(), SnapshotReplayError> {
    let references = collect_state_event_references(state);
    if references.is_empty() {
        return Ok(());
    }

    let ids = references.iter().copied().collect::<Vec<_>>();
    let mut metadata = HashMap::<EventId, (CampaignId, u64)>::with_capacity(ids.len());
    for chunk in ids.chunks(EVENT_LOOKUP_BATCH_SIZE) {
        let placeholders = vec!["?"; chunk.len()].join(", ");
        let sql = format!(
            "SELECT id, campaign_id, sequence FROM event_journal WHERE id IN ({placeholders})"
        );
        let mut query = sqlx::query(&sql);
        for event_id in chunk {
            query = query.bind(event_id.0.to_string());
        }
        for row in query.fetch_all(&mut *connection).await? {
            let id: String = row.try_get("id")?;
            let campaign: String = row.try_get("campaign_id")?;
            metadata.insert(
                EventId(parse_uuid(&id, "event_journal.id")?),
                (
                    CampaignId(parse_uuid(&campaign, "event_journal.campaign_id")?),
                    stored_u64(row.try_get("sequence")?, "event_journal.sequence")?,
                ),
            );
        }
    }

    for event_id in references {
        let Some((found_campaign, sequence)) = metadata.get(&event_id).copied() else {
            return Err(SnapshotReplayError::MissingStateEventReference(event_id));
        };
        if found_campaign != campaign_id {
            return Err(SnapshotReplayError::StateEventReferenceCampaignMismatch(
                event_id,
            ));
        }
        if sequence > target_sequence {
            return Err(SnapshotReplayError::StateEventReferenceAfterTarget {
                event_id,
                sequence,
                target: target_sequence,
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

fn decode_event_source(value: &str) -> Result<EventSource, SnapshotReplayError> {
    match value {
        "player_action" => Ok(EventSource::PlayerAction),
        "rule_resolution" => Ok(EventSource::RuleResolution),
        "world_simulation" => Ok(EventSource::WorldSimulation),
        "procedural_generation" => Ok(EventSource::ProceduralGeneration),
        "admin_correction" => Ok(EventSource::AdminCorrection),
        "import" => Ok(EventSource::Import),
        other => Err(SnapshotReplayError::InvalidEventSource(other.into())),
    }
}

fn decode_agent(
    kind: Option<String>,
    id: Option<String>,
) -> Result<Option<AgentRef>, SnapshotReplayError> {
    match (kind.as_deref(), id.as_deref()) {
        (None, None) => Ok(None),
        (Some("entity"), Some(id)) => Ok(Some(AgentRef::Entity(EntityId(parse_uuid(
            id, "actor_id",
        )?)))),
        (Some("faction"), Some(id)) => Ok(Some(AgentRef::Faction(FactionId(parse_uuid(
            id, "actor_id",
        )?)))),
        _ => Err(SnapshotReplayError::InvalidActor),
    }
}

fn parse_optional_play_session(
    value: Option<String>,
) -> Result<Option<PlaySessionId>, SnapshotReplayError> {
    value
        .as_deref()
        .map(|value| parse_uuid(value, "session_id").map(PlaySessionId))
        .transpose()
}

fn parse_uuid(value: &str, field: &'static str) -> Result<Uuid, SnapshotReplayError> {
    Uuid::parse_str(value).map_err(|source| SnapshotReplayError::InvalidUuid { field, source })
}

fn sequence_to_i64(value: u64) -> Result<i64, SnapshotReplayError> {
    i64::try_from(value).map_err(|_| SnapshotReplayError::SequenceOverflow(value))
}

fn stored_u64(value: i64, field: &'static str) -> Result<u64, SnapshotReplayError> {
    u64::try_from(value).map_err(|_| SnapshotReplayError::InvalidInteger { field, value })
}

fn stored_u32(value: i64, field: &'static str) -> Result<u32, SnapshotReplayError> {
    u32::try_from(value).map_err(|_| SnapshotReplayError::InvalidInteger { field, value })
}
