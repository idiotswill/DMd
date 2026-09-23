use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::{
    Belief, BeliefId, Campaign, CampaignId, Character, CharacterId, Claim, ClaimId, Custody,
    DirectiveId, EntityId, EntityKind, Fact, FactId, FactValue, Faction, FactionId, ItemId,
    ItemInstance, KnowledgeHolder, KnowledgeRecord, KnowledgeTarget, Location, LocationId,
    Ownership, Player, PlayerId, PresenceRole, Scene, SceneId, SceneStatus, StandingDirective,
    SubjectRef, WorldClock, WorldEntity,
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

    #[must_use]
    pub fn validate(&self) -> Vec<StateInvariantViolation> {
        let mut violations = Vec::new();
        let expected = self.campaign.id;
        let mut character_entities = HashSet::new();

        for (key, player) in &self.players {
            check_key(*key == player.id, "player", &mut violations);
            check_campaign(expected, player.campaign_id, "player", &mut violations);
        }

        for (key, character) in &self.characters {
            check_key(*key == character.id, "character", &mut violations);
            check_campaign(
                expected,
                character.campaign_id,
                "character",
                &mut violations,
            );
            match self.entities.get(&character.entity_id) {
                Some(entity) if entity.kind != EntityKind::Character => {
                    violations.push(StateInvariantViolation::WrongEntityKind {
                        owner: "character".into(),
                        expected: "Character".into(),
                    });
                }
                Some(_) => {}
                None => violations.push(StateInvariantViolation::MissingReference {
                    owner: "character".into(),
                    target: "entity".into(),
                }),
            }
            if !character_entities.insert(character.entity_id) {
                violations.push(StateInvariantViolation::DuplicateReference {
                    owner: "character".into(),
                    target: "entity".into(),
                });
            }
            if let Some(player_id) = character.controlling_player_id {
                if !self.players.contains_key(&player_id) {
                    violations.push(StateInvariantViolation::MissingReference {
                        owner: "character".into(),
                        target: "controlling player".into(),
                    });
                }
            }
        }

        for (key, entity) in &self.entities {
            check_key(*key == entity.id, "entity", &mut violations);
            check_campaign(expected, entity.campaign_id, "entity", &mut violations);
            if let Some(location_id) = entity.location_id {
                check_exists(
                    self.locations.contains_key(&location_id),
                    "entity",
                    "location",
                    &mut violations,
                );
            }
        }

        for (key, faction) in &self.factions {
            check_key(*key == faction.id, "faction", &mut violations);
            check_campaign(expected, faction.campaign_id, "faction", &mut violations);
        }

        for (key, location) in &self.locations {
            check_key(*key == location.id, "location", &mut violations);
            check_campaign(expected, location.campaign_id, "location", &mut violations);
            if let Some(parent_id) = location.parent_location_id {
                if parent_id == location.id {
                    violations.push(StateInvariantViolation::SelfReference(
                        "location parent".into(),
                    ));
                } else if !self.locations.contains_key(&parent_id) {
                    violations.push(StateInvariantViolation::MissingReference {
                        owner: "location".into(),
                        target: "parent location".into(),
                    });
                }
            }
        }

        let mut active_scene_participants = HashSet::new();
        for (key, scene) in &self.scenes {
            check_key(*key == scene.id, "scene", &mut violations);
            check_campaign(expected, scene.campaign_id, "scene", &mut violations);
            if !self.locations.contains_key(&scene.location_id) {
                violations.push(StateInvariantViolation::MissingReference {
                    owner: "scene".into(),
                    target: "location".into(),
                });
            }
            let mut scene_entities = HashSet::new();
            for presence in &scene.presences {
                let entity = self.entities.get(&presence.entity_id);
                if entity.is_none() {
                    violations.push(StateInvariantViolation::MissingReference {
                        owner: "scene presence".into(),
                        target: "entity".into(),
                    });
                }
                if !scene_entities.insert(presence.entity_id) {
                    violations.push(StateInvariantViolation::DuplicateReference {
                        owner: "scene".into(),
                        target: "presence entity".into(),
                    });
                }
                if scene.status == SceneStatus::Active && presence.role == PresenceRole::Participant
                {
                    if !active_scene_participants.insert(presence.entity_id) {
                        violations.push(StateInvariantViolation::MultipleActiveSceneParticipation(
                            presence.entity_id,
                        ));
                    }
                    if let Some(entity) = entity {
                        if entity.location_id != Some(scene.location_id) {
                            violations.push(StateInvariantViolation::SceneLocationMismatch {
                                entity_id: presence.entity_id,
                                scene_id: scene.id,
                                scene_location_id: scene.location_id,
                                entity_location_id: entity.location_id,
                            });
                        }
                    }
                }
            }
        }

        for (key, item) in &self.items {
            check_key(*key == item.id, "item", &mut violations);
            check_campaign(expected, item.campaign_id, "item", &mut violations);
            match item.owner {
                Ownership::Entity(entity_id) => {
                    check_exists(
                        self.entities.contains_key(&entity_id),
                        "item",
                        "owner entity",
                        &mut violations,
                    );
                }
                Ownership::Faction(faction_id) => {
                    check_exists(
                        self.factions.contains_key(&faction_id),
                        "item",
                        "owner faction",
                        &mut violations,
                    );
                }
                Ownership::Unowned => {}
            }
            match item.custody {
                Custody::Entity(entity_id) => {
                    check_exists(
                        self.entities.contains_key(&entity_id),
                        "item",
                        "custody entity",
                        &mut violations,
                    );
                }
                Custody::Location(location_id) => {
                    check_exists(
                        self.locations.contains_key(&location_id),
                        "item",
                        "custody location",
                        &mut violations,
                    );
                }
                Custody::Container(container_id) => {
                    check_exists(
                        self.items.contains_key(&container_id),
                        "item",
                        "container item",
                        &mut violations,
                    );
                    if container_id == item.id {
                        violations.push(StateInvariantViolation::SelfReference(
                            "item container".into(),
                        ));
                    }
                }
                Custody::Missing | Custody::Destroyed => {}
            }
        }

        for (key, fact) in &self.facts {
            check_key(*key == fact.id, "fact", &mut violations);
            check_campaign(expected, fact.campaign_id, "fact", &mut violations);
            self.validate_proposition_refs(
                &fact.proposition.subject,
                &fact.proposition.value,
                &mut violations,
            );
        }
        for (key, claim) in &self.claims {
            check_key(*key == claim.id, "claim", &mut violations);
            check_campaign(expected, claim.campaign_id, "claim", &mut violations);
            check_exists(
                self.entities.contains_key(&claim.speaker),
                "claim",
                "speaker entity",
                &mut violations,
            );
            self.validate_proposition_refs(
                &claim.proposition.subject,
                &claim.proposition.value,
                &mut violations,
            );
        }
        for (key, belief) in &self.beliefs {
            check_key(*key == belief.id, "belief", &mut violations);
            check_campaign(expected, belief.campaign_id, "belief", &mut violations);
            check_exists(
                self.entities.contains_key(&belief.holder),
                "belief",
                "holder entity",
                &mut violations,
            );
            self.validate_proposition_refs(
                &belief.proposition.subject,
                &belief.proposition.value,
                &mut violations,
            );
        }
        for knowledge in &self.knowledge {
            check_campaign(
                expected,
                knowledge.campaign_id,
                "knowledge",
                &mut violations,
            );
            if let KnowledgeHolder::Entity(entity_id) = knowledge.holder {
                check_exists(
                    self.entities.contains_key(&entity_id),
                    "knowledge",
                    "holder entity",
                    &mut violations,
                );
            }
            match knowledge.target {
                KnowledgeTarget::Fact(fact_id) => {
                    check_exists(
                        self.facts.contains_key(&fact_id),
                        "knowledge",
                        "fact",
                        &mut violations,
                    );
                }
                KnowledgeTarget::Claim(claim_id) => {
                    check_exists(
                        self.claims.contains_key(&claim_id),
                        "knowledge",
                        "claim",
                        &mut violations,
                    );
                }
            }
        }
        for (key, directive) in &self.directives {
            check_key(*key == directive.id, "directive", &mut violations);
            check_campaign(
                expected,
                directive.campaign_id,
                "directive",
                &mut violations,
            );
            check_exists(
                self.scenes.contains_key(&directive.scene_id),
                "directive",
                "scene",
                &mut violations,
            );
            for actor in &directive.actors {
                check_exists(
                    self.entities.contains_key(actor),
                    "directive",
                    "actor entity",
                    &mut violations,
                );
            }
        }

        violations
    }

    fn validate_proposition_refs(
        &self,
        subject: &SubjectRef,
        value: &FactValue,
        violations: &mut Vec<StateInvariantViolation>,
    ) {
        match subject {
            SubjectRef::Campaign(id) => {
                if *id != self.campaign.id {
                    violations.push(StateInvariantViolation::CampaignMismatch {
                        record_kind: "proposition subject".into(),
                        expected: self.campaign.id,
                        actual: *id,
                    });
                }
            }
            SubjectRef::Entity(id) => check_exists(
                self.entities.contains_key(id),
                "proposition",
                "subject entity",
                violations,
            ),
            SubjectRef::Location(id) => check_exists(
                self.locations.contains_key(id),
                "proposition",
                "subject location",
                violations,
            ),
            SubjectRef::Faction(id) => check_exists(
                self.factions.contains_key(id),
                "proposition",
                "subject faction",
                violations,
            ),
            SubjectRef::Item(id) => check_exists(
                self.items.contains_key(id),
                "proposition",
                "subject item",
                violations,
            ),
        }

        match value {
            FactValue::Entity(id) => check_exists(
                self.entities.contains_key(id),
                "proposition",
                "value entity",
                violations,
            ),
            FactValue::Location(id) => check_exists(
                self.locations.contains_key(id),
                "proposition",
                "value location",
                violations,
            ),
            FactValue::Faction(id) => check_exists(
                self.factions.contains_key(id),
                "proposition",
                "value faction",
                violations,
            ),
            FactValue::Item(id) => check_exists(
                self.items.contains_key(id),
                "proposition",
                "value item",
                violations,
            ),
            FactValue::Boolean(_)
            | FactValue::Integer(_)
            | FactValue::Text(_)
            | FactValue::Time(_) => {}
        }
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
    DuplicateReference {
        owner: String,
        target: String,
    },
    SelfReference(String),
    WrongEntityKind {
        owner: String,
        expected: String,
    },
    MultipleActiveSceneParticipation(EntityId),
    SceneLocationMismatch {
        entity_id: EntityId,
        scene_id: SceneId,
        scene_location_id: LocationId,
        entity_location_id: Option<LocationId>,
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

fn check_key(matches: bool, record_kind: &str, violations: &mut Vec<StateInvariantViolation>) {
    if !matches {
        violations.push(StateInvariantViolation::KeyDoesNotMatchRecord(
            record_kind.into(),
        ));
    }
}

fn check_exists(
    exists: bool,
    owner: &str,
    target: &str,
    violations: &mut Vec<StateInvariantViolation>,
) {
    if !exists {
        violations.push(StateInvariantViolation::MissingReference {
            owner: owner.into(),
            target: target.into(),
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

    fn empty_state() -> CampaignState {
        let id = CampaignId::new();
        CampaignState::empty(
            campaign(id),
            WorldClock {
                now: crate::WorldInstant(0),
                calendar_id: "test.calendar".into(),
            },
        )
    }

    #[test]
    fn empty_state_is_valid() {
        assert!(empty_state().validate().is_empty());
    }

    #[test]
    fn cross_campaign_record_is_rejected() {
        let mut state = empty_state();
        let player = Player {
            id: PlayerId::new(),
            campaign_id: CampaignId::new(),
            display_name: "Player".into(),
        };
        state.players.insert(player.id, player);

        assert!(
            state
                .validate()
                .iter()
                .any(|v| matches!(v, StateInvariantViolation::CampaignMismatch { .. }))
        );
    }

    #[test]
    fn dangling_scene_presence_is_rejected() {
        let mut state = empty_state();
        let location = Location {
            id: LocationId::new(),
            campaign_id: state.campaign_id(),
            display_name: "Test Location".into(),
            parent_location_id: None,
        };
        state.locations.insert(location.id, location.clone());
        let scene = Scene {
            id: SceneId::new(),
            campaign_id: state.campaign_id(),
            location_id: location.id,
            mode: crate::SceneMode::Exploration,
            status: crate::SceneStatus::Active,
            started_at: WorldInstant(0),
            presences: vec![crate::ScenePresence {
                entity_id: EntityId::new(),
                role: crate::PresenceRole::Participant,
            }],
        };
        state.scenes.insert(scene.id, scene);

        assert!(state.validate().iter().any(|v| matches!(
            v,
            StateInvariantViolation::MissingReference { owner, target }
                if owner == "scene presence" && target == "entity"
        )));
    }
}
