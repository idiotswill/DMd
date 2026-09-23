use serde::{Deserialize, Serialize};

use crate::{CampaignId, CharacterId, EntityId, PlayerId};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VersionedRef {
    pub id: String,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CampaignStatus {
    Active,
    Paused,
    Archived,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Campaign {
    pub id: CampaignId,
    pub display_name: String,
    pub status: CampaignStatus,
    pub world_seed: u64,
    pub ruleset: VersionedRef,
    pub content_packs: Vec<VersionedRef>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Player {
    pub id: PlayerId,
    pub campaign_id: CampaignId,
    pub display_name: String,
}

/// Persistent lifecycle of a player character across the campaign.
/// Session attendance and temporary scene absence are modeled elsewhere.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CharacterStatus {
    Active,
    Retired,
    Dead,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Character {
    pub id: CharacterId,
    /// World-facing identity used by spatial, scene, knowledge, and event systems.
    pub entity_id: EntityId,
    pub campaign_id: CampaignId,
    /// None permits NPC conversion, unassigned imports, and temporarily ownerless PCs.
    pub controlling_player_id: Option<PlayerId>,
    pub display_name: String,
    pub status: CharacterStatus,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn character_name_is_not_identity() {
        let campaign_id = CampaignId::new();
        let a = Character {
            id: CharacterId::new(),
            entity_id: EntityId::new(),
            campaign_id,
            controlling_player_id: None,
            display_name: "Same Name".into(),
            status: CharacterStatus::Active,
        };
        let mut b = a.clone();
        b.id = CharacterId::new();
        b.entity_id = EntityId::new();
        assert_ne!(a.id, b.id);
        assert_eq!(a.display_name, b.display_name);
    }
}
