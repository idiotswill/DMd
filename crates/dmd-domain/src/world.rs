use serde::{Deserialize, Serialize};

use crate::{CampaignId, EntityId, LocationId};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntityKind {
    Character,
    Npc,
    Creature,
    Object,
    Hazard,
    Other(String),
}

/// Objective coarse existence of an entity in world state.
/// Party/campaign lifecycle such as character retirement is modeled separately.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntityExistence {
    Present,
    Missing,
    Destroyed,
    Dead,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorldEntity {
    pub id: EntityId,
    pub campaign_id: CampaignId,
    pub display_name: String,
    pub kind: EntityKind,
    pub existence: EntityExistence,
    /// Coarse persistent world position. Tactical coordinates belong to the spatial layer.
    /// `None` means unknown, unplaced, extraplanar, or otherwise intentionally unresolved.
    pub location_id: Option<LocationId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Location {
    pub id: LocationId,
    pub campaign_id: CampaignId,
    pub display_name: String,
    pub parent_location_id: Option<LocationId>,
}
