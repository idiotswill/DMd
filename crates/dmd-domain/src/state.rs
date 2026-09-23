use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    Belief, BeliefId, Campaign, CampaignId, Character, CharacterId, Claim, ClaimId, DirectiveId,
    EntityId, Fact, FactId, Faction, FactionId, ItemId, ItemInstance, KnowledgeRecord, Location,
    LocationId, Player, PlayerId, Scene, SceneId, StandingDirective, WorldClock, WorldEntity,
};

pub const CURRENT_STATE_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CampaignState {
    pub schema_version: u32,
    pub campaign: Campaign,
    pub clock: WorldClock,
    pub players: HashMap<PlayerId, Player>,
    pub characters: HashMap<CharacterId, Character>,
    pub entities: HashMap<EntityId, WorldEntity>,
    pub factions: HashMap<FactionId, Faction>,
    pub locations: HashMap<LocationId, Location>,
    pub scenes: HashMap<SceneId, Scene>,
    pub items: HashMap<ItemId, ItemInstance>,
    pub facts: HashMap<FactId, Fact>,
    pub claims: HashMap<ClaimId, Claim>,
    pub beliefs: HashMap<BeliefId, Belief>,
    pub knowledge: Vec<KnowledgeRecord>,
    pub directives: HashMap<DirectiveId, StandingDirective>,
    /// Last event sequence incorporated into this materialized snapshot.
    pub applied_event_sequence: u64,
}

impl CampaignState {
    #[must_use]
    pub fn empty(campaign: Campaign, clock: WorldClock) -> Self {
        Self {
            schema_version: CURRENT_STATE_SCHEMA_VERSION,
            campaign,
            clock,
            players: HashMap::new(),
            characters: HashMap::new(),
            entities: HashMap::new(),
            factions: HashMap::new(),
            locations: HashMap::new(),
            scenes: HashMap::new(),
            items: HashMap::new(),
            facts: HashMap::new(),
            claims: HashMap::new(),
            beliefs: HashMap::new(),
            knowledge: Vec::new(),
            directives: HashMap::new(),
            applied_event_sequence: 0,
        }
    }

    #[must_use]
    pub fn campaign_id(&self) -> CampaignId {
        self.campaign.id
    }

    pub fn encode_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub fn decode_json(value: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(value)
    }
}
