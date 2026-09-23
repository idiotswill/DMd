use serde::{Deserialize, Serialize};

use crate::{CampaignId, FactionId};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FactionStatus {
    Active,
    Dormant,
    Dissolved,
    Destroyed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Faction {
    pub id: FactionId,
    pub campaign_id: CampaignId,
    pub display_name: String,
    pub status: FactionStatus,
}
