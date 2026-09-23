use serde::{Deserialize, Serialize};

use crate::{
    AgentRef, CampaignId, CommandId, EventId, PlaySessionId, RecordCodecError, SerializedRecord,
    WorldInstant,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventSource {
    PlayerAction,
    RuleResolution,
    WorldSimulation,
    ProceduralGeneration,
    AdminCorrection,
    Import,
}

/// Event produced by a resolver before persistence allocates its campaign sequence.
///
/// Campaign/session/command correlation is inherited from the trusted command at commit time, so
/// a resolver cannot accidentally persist an event under a different campaign or authority path.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PendingEvent<T> {
    pub id: EventId,
    pub occurred_at: WorldInstant,
    pub source: EventSource,
    pub actor: Option<AgentRef>,
    pub caused_by_event_ids: Vec<EventId>,
    pub payload: T,
}

impl<T: Serialize> PendingEvent<T> {
    pub fn encode(
        &self,
        kind: impl Into<String>,
        schema_version: u32,
    ) -> Result<EncodedPendingEvent, RecordCodecError> {
        Ok(EncodedPendingEvent {
            id: self.id,
            occurred_at: self.occurred_at,
            source: self.source.clone(),
            actor: self.actor,
            caused_by_event_ids: self.caused_by_event_ids.clone(),
            payload: SerializedRecord::encode(kind, schema_version, &self.payload)?,
        })
    }
}

/// Persistence-ready event whose payload was serialized from a typed value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncodedPendingEvent {
    pub id: EventId,
    pub occurred_at: WorldInstant,
    pub source: EventSource,
    pub actor: Option<AgentRef>,
    pub caused_by_event_ids: Vec<EventId>,
    pub payload: SerializedRecord,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventMeta {
    pub id: EventId,
    pub campaign_id: CampaignId,
    /// None is valid for between-session simulation, imports, or maintenance events.
    pub session_id: Option<PlaySessionId>,
    /// Monotonic within a campaign. Persistence owns allocation and uniqueness.
    pub sequence: u64,
    pub occurred_at: WorldInstant,
    pub source: EventSource,
    pub actor: Option<AgentRef>,
    /// Direct causal parents. Empty means no earlier material event is recorded as a cause.
    /// Multiple parents allow consequences produced jointly by several prior developments.
    pub caused_by_event_ids: Vec<EventId>,
    pub command_id: Option<CommandId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventEnvelope<T> {
    pub meta: EventMeta,
    pub payload: T,
}

impl<T> EventEnvelope<T> {
    #[must_use]
    pub fn map_payload<U>(self, f: impl FnOnce(T) -> U) -> EventEnvelope<U> {
        EventEnvelope {
            meta: self.meta,
            payload: f(self.payload),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FactionId;

    #[test]
    fn event_can_record_multiple_direct_causes() {
        let first = EventId::new();
        let second = EventId::new();
        let meta = EventMeta {
            id: EventId::new(),
            campaign_id: CampaignId::new(),
            session_id: None,
            sequence: 3,
            occurred_at: WorldInstant(10),
            source: EventSource::WorldSimulation,
            actor: Some(AgentRef::Faction(FactionId::new())),
            caused_by_event_ids: vec![first, second],
            command_id: None,
        };

        assert_eq!(meta.caused_by_event_ids, vec![first, second]);
        assert!(matches!(meta.actor, Some(AgentRef::Faction(_))));
    }

    #[test]
    fn pending_event_encodes_typed_payload_without_assigning_sequence() {
        let event = PendingEvent {
            id: EventId::new(),
            occurred_at: WorldInstant(20),
            source: EventSource::RuleResolution,
            actor: None,
            caused_by_event_ids: vec![],
            payload: 7_i64,
        };

        let encoded = event
            .encode("test.value_changed", 1)
            .expect("typed payload should encode");

        assert_eq!(encoded.id, event.id);
        assert_eq!(encoded.payload.kind(), "test.value_changed");
        assert_eq!(encoded.payload.schema_version(), 1);
    }
}
