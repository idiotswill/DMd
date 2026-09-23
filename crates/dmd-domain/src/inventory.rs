use serde::{Deserialize, Serialize};

use crate::{CampaignId, EntityId, FactionId, ItemId, LocationId};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Ownership {
    Entity(EntityId),
    Faction(FactionId),
    Unowned,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Custody {
    Entity(EntityId),
    Location(LocationId),
    Container(ItemId),
    Missing,
    Destroyed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ItemState {
    Intact,
    Damaged,
    Spent,
    Destroyed,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemInstance {
    pub id: ItemId,
    pub campaign_id: CampaignId,
    pub definition_id: String,
    pub display_name: String,
    pub quantity: u32,
    /// Legal/social ownership and physical custody are intentionally independent.
    pub owner: Ownership,
    pub custody: Custody,
    pub state: ItemState,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn borrowed_item_can_have_different_owner_and_carrier() {
        let owner = EntityId::new();
        let carrier = EntityId::new();
        let item = ItemInstance {
            id: ItemId::new(),
            campaign_id: CampaignId::new(),
            definition_id: "test.item".into(),
            display_name: "Borrowed Item".into(),
            quantity: 1,
            owner: Ownership::Entity(owner),
            custody: Custody::Entity(carrier),
            state: ItemState::Intact,
        };

        assert_ne!(owner, carrier);
        assert_eq!(item.owner, Ownership::Entity(owner));
        assert_eq!(item.custody, Custody::Entity(carrier));
    }
}
