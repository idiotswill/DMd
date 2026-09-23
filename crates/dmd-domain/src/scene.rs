use serde::{Deserialize, Serialize};

use crate::{
    CampaignId, DirectiveId, EntityId, LocationId, SceneId, WorldDuration, WorldInstant,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SceneMode {
    Exploration,
    Social,
    Combat,
    Travel,
    Downtime,
    Other(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SceneStatus {
    Active,
    Paused,
    Closed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PresenceRole {
    Participant,
    Observer,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScenePresence {
    pub entity_id: EntityId,
    pub role: PresenceRole,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Scene {
    pub id: SceneId,
    pub campaign_id: CampaignId,
    pub location_id: LocationId,
    pub mode: SceneMode,
    pub status: SceneStatus,
    pub started_at: WorldInstant,
    pub presences: Vec<ScenePresence>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DirectiveStopCondition {
    DangerDetected,
    DecisionRequired,
    TrailLost,
    SignificantDiscovery,
    DestinationReached,
    TimeElapsed(WorldDuration),
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StandingDirective {
    pub id: DirectiveId,
    pub campaign_id: CampaignId,
    pub scene_id: SceneId,
    pub actors: Vec<EntityId>,
    /// Human-readable semantic intent. Later layers may attach a structured plan beside it.
    pub summary: String,
    pub stop_conditions: Vec<DirectiveStopCondition>,
    pub created_at: WorldInstant,
    pub active: bool,
}
