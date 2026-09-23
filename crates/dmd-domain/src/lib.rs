use serde::{Deserialize, Serialize};
use uuid::Uuid;

macro_rules! id_type {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(pub Uuid);

        impl $name {
            pub fn new() -> Self {
                Self(Uuid::new_v4())
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }
    };
}

id_type!(CampaignId);
id_type!(PlayerId);
id_type!(CharacterId);
id_type!(SceneId);
id_type!(LocationId);
id_type!(EntityId);
id_type!(EventId);
id_type!(FactId);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CharacterStatus {
    Active,
    Absent,
    Retired,
    Dead,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CharacterRef {
    pub id: CharacterId,
    pub display_name: String,
    pub status: CharacterStatus,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identifiers_are_distinct_values() {
        assert_ne!(CharacterId::new().0, CharacterId::new().0);
    }
}
