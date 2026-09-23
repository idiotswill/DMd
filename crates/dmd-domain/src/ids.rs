use serde::{Deserialize, Serialize};
use uuid::Uuid;

macro_rules! id_type {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(pub Uuid);

        impl $name {
            #[must_use]
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
id_type!(PlaySessionId);
id_type!(PlayerId);
id_type!(CharacterId);
id_type!(EntityId);
id_type!(LocationId);
id_type!(SceneId);
id_type!(ItemId);
id_type!(FactionId);
id_type!(QuestId);
id_type!(EffectId);
id_type!(FactId);
id_type!(ClaimId);
id_type!(BeliefId);
id_type!(KnowledgeId);
id_type!(EvidenceId);
id_type!(EventId);
id_type!(CommandId);
id_type!(RollRequestId);
id_type!(DirectiveId);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identifiers_are_distinct_values() {
        assert_ne!(CharacterId::new().0, CharacterId::new().0);
    }
}
