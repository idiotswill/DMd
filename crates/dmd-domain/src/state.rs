use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    Belief, BeliefId, Campaign, CampaignId, Character, CharacterId, Claim, ClaimId, DirectiveId,
    EntityId, Fact, FactId, ItemId, ItemInstance, KnowledgeRecord, Location, LocationId, Player,
    PlayerId, Scene, SceneId, StandingDirective, WorldClock, WorldEntity,
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

    #[must_use]
    pub fn validate(&self) -> Vec<StateInvariantViolation> {
        let mut violations = Vec::new();
        let expected = self.campaign.id;

        for (key, player) in &self.players {
            if *key != player.id {
                violations.push(StateInvariantViolation::KeyDoesNotMatchRecord("player".into()));
            }
            check_campaign(expected, player.campaign_id, "player", &mut violations);
        }

        for (key, character) in &self.characters {
            if *key != character.id {
                violations.push(StateInvariantViolation::KeyDoesNotMatchRecord("character".into()));
            }
            check_campaign(expected, character.campaign_id, "character", &mut violations);
            if !self.entities.contains_key(&character.entity_id) {
                violations.push(StateInvariantViolation::MissingReference {
                    owner: "character".into(),
                    target: "entity".into(),
                });
            }
            if let Some(player_id) = character.controlling_player_id
                && !self.players.contains_key(&player_id)
            {
                violations.push(StateInvariantViolation::MissingReference {
                    owner: "character".into(),
                    target: "controlling player".into(),
                });
            }
        }

        for (key, entity) in &self.entities {
            if *key != entity.id {
                violations.push(StateInvariantViolation::KeyDoesNotMatchRecord("entity".into()));
            }
            check_campaign(expected, entity.campaign_id, "entity", &mut violations);
        }

        for (key, location) in &self.locations {
            if *key != location.id {
                violations.push(StateInvariantViolation::KeyDoesNotMatchRecord("location".into()));
            }
            check_campaign(expected, location.campaign_id, "location", &mut violations);
            if let Some(parent_id) = location.parent_location_id
                && !self.locations.contains_key(&parent_id)
            {
                violations.push(StateInvariantViolation::MissingReference {
                    owner: "location".into(),
                    target: "parent location".into(),
                });
            }
        }

        for (key, scene) in &self.scenes {
            if *key != scene.id {
                violations.push(StateInvariantViolation::KeyDoesNotMatchRecord("scene".into()));
            }
            check_campaign(expected, scene.campaign_id, "scene", &mut violations);
            if !self.locations.contains_key(&scene.location_id) {
                violations.push(StateInvariantViolation::MissingReference {
                    owner: "scene".into(),
                    target: "location".into(),
                });
            }
            for presence in &scene.presences {
                if !self.entities.contains_key(&presence.entity_id) {
                    violations.push(StateInvariantViolation::MissingReference {
                        owner: "scene presence".into(),
                        target: "entity".into(),
                    });
                }
            }
        }

        for item in self.items.values() {
            check_campaign(expected, item.campaign_id, "item", &mut violations);
        }
        for fact in self.facts.values() {
            check_campaign(expected, fact.campaign_id, "fact", &mut violations);
        }
        for claim in self.claims.values() {
            check_campaign(expected, claim.campaign_id, "claim", &mut violations);
        }
        for belief in self.beliefs.values() {
            check_campaign(expected, belief.campaign_id, "belief", &mut violations);
        }
        for knowledge in &self.knowledge {
            check_campaign(expected, knowledge.campaign_id, "knowledge", &mut violations);
        }
        for directive in self.directives.values() {
            check_campaign(expected, directive.campaign_id, "directive", &mut violations);
            if !self.scenes.contains_key(&directive.scene_id) {
                violations.push(StateInvariantViolation::MissingReference {
                    owner: "directive".into(),
                    target: "scene".into(),
                });
            }
        }

        violations
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StateInvariantViolation {
    CampaignMismatch {
        record_kind: String,
        expected: CampaignId,
        actual: CampaignId,
    },
    KeyDoesNotMatchRecord(String),
    MissingReference {
        owner: String,
        target: String,
    },
}

fn check_campaign(
    expected: CampaignId,
    actual: CampaignId,
    record_kind: &str,
    violations: &mut Vec<StateInvariantViolation>,
) {
    if expected != actual {
        violations.push(StateInvariantViolation::CampaignMismatch {
            record_kind: record_kind.into(),
            expected,
            actual,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CampaignStatus, VersionedRef, WorldInstant};

    fn campaign(id: CampaignId) -> Campaign {
        Campaign {
            id,
            display_name: "Generic Test Campaign".into(),
            status: CampaignStatus::Active,
            world_seed: 42,
            ruleset: VersionedRef {
                id: "test.rules".into(),
                version: "1".into(),
            },
            content_packs: vec![],
        }
    }

    #[test]
    fn empty_state_is_valid() {
        let id = CampaignId::new();
        let state = CampaignState::empty(
            campaign(id),
            WorldClock {
                now: WorldInstant(0),
                calendar_id: "test.calendar".into(),
            },
        );
        assert!(state.validate().is_empty());
    }

    #[test]
    fn cross_campaign_record_is_rejected() {
        let id = CampaignId::new();
        let mut state = CampaignState::empty(
            campaign(id),
            WorldClock {
                now: WorldInstant(0),
                calendar_id: "test.calendar".into(),
            },
        );
        let player = Player {
            id: PlayerId::new(),
            campaign_id: CampaignId::new(),
            display_name: "Player".into(),
        };
        state.players.insert(player.id, player);

        assert!(state
            .validate()
            .iter()
            .any(|v| matches!(v, StateInvariantViolation::CampaignMismatch { .. })));
    }
}
