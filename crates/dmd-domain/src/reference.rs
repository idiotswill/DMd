use serde::{Deserialize, Serialize};

use crate::{EntityId, FactionId};

/// A world actor capable of holding knowledge, causing events, or issuing actions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentRef {
    Entity(EntityId),
    Faction(FactionId),
}
