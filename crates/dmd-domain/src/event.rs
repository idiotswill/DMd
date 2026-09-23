use serde::{Deserialize, Serialize};

use crate::{AgentRef, CampaignId, CommandId, EventId, PlaySessionId, WorldInstant};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventSource {
    PlayerAction,
    RuleResolution,
    WorldSimulation,
    ProceduralGeneration,
    AdminCorrection,
    Import,
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
}
