//! Durable table observations are conversation history, never authoritative world events.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{CampaignId, CommandIssuer, PlaySessionId, PlayerId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ObservationId(pub Uuid);

impl ObservationId {
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for ObservationId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ObservationAudience {
    Party,
    Player(PlayerId),
    Host,
}

/// The application supplies trusted issuer/audience metadata and an already typed body.
/// Stable IDs make retrying an identical batch safe without replacing prior conversation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NewSessionObservation {
    pub id: ObservationId,
    pub campaign_id: CampaignId,
    pub session_id: Option<PlaySessionId>,
    pub issuer: CommandIssuer,
    pub audience: ObservationAudience,
    /// Authoritative head seen while producing this observation, not a new world event.
    pub observed_event_sequence: u64,
    pub kind: String,
    pub payload_schema_version: u32,
    pub payload_json: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SessionObservation {
    /// Monotonic per-campaign conversation order, independent of the world event sequence.
    pub ordinal: u64,
    pub record: NewSessionObservation,
}

impl NewSessionObservation {
    /// Structural bounds apply to raw recovery as well as application writes.
    #[must_use]
    pub fn valid_shape(&self) -> bool {
        !self.kind.trim().is_empty()
            && self.kind.len() <= 100
            && self.payload_schema_version > 0
            && self.payload_json.len() <= 65_536
            && serde_json::from_str::<serde_json::Value>(&self.payload_json).is_ok()
    }
}
