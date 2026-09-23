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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntityExistence {
    Present,
    Missing,
    Destroyed,
    Dead,
    Retired,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorldEntity {
    pub id: EntityId,
    pub campaign_id: CampaignId,
    pub display_name: String,
    pub kind: EntityKind,
    pub existence: EntityExistence,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Location {
    pub id: LocationId,
    pub campaign_id: CampaignId,
    pub display_name: String,
    pub parent_location_id: Option<LocationId>,
}
