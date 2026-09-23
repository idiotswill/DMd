use serde::{Deserialize, Serialize};

use crate::{CampaignId, CommandId, EntityId, EventId, WorldInstant};

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
    /// Monotonic within a campaign. Persistence owns allocation and uniqueness.
    pub sequence: u64,
    pub occurred_at: WorldInstant,
    pub source: EventSource,
    pub actor: Option<EntityId>,
    pub caused_by_event_id: Option<EventId>,
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
