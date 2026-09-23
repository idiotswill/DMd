use serde::{Deserialize, Serialize};

use crate::{
    BeliefId, CampaignId, ClaimId, EntityId, EventId, FactId, FactionId, ItemId, LocationId,
    WorldInstant,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SubjectRef {
    Campaign(CampaignId),
    Entity(EntityId),
    Location(LocationId),
    Faction(FactionId),
    Item(ItemId),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FactValue {
    Boolean(bool),
    Integer(i64),
    Text(String),
    Entity(EntityId),
    Location(LocationId),
    Faction(FactionId),
    Item(ItemId),
    Time(WorldInstant),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Proposition {
    pub subject: SubjectRef,
    pub predicate: String,
    pub value: FactValue,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FactProvenance {
    InitialWorldState,
    Generated,
    Observed,
    RuleResolution,
    Imported,
    AdminCorrection,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fact {
    pub id: FactId,
    pub campaign_id: CampaignId,
    pub proposition: Proposition,
    pub valid_from: WorldInstant,
    pub valid_until: Option<WorldInstant>,
    pub provenance: FactProvenance,
    pub source_event_id: Option<EventId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Claim {
    pub id: ClaimId,
    pub campaign_id: CampaignId,
    pub speaker: EntityId,
    /// What the speaker asserted. It is not promoted to world truth by existing.
    pub proposition: Proposition,
    pub made_at: WorldInstant,
    pub source_event_id: EventId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BeliefConfidence {
    Doubtful,
    Possible,
    Likely,
    Confident,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BeliefBasis {
    Fact(FactId),
    Claim(ClaimId),
    DirectObservation(EventId),
    Inference(Vec<BeliefBasis>),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Belief {
    pub id: BeliefId,
    pub campaign_id: CampaignId,
    pub holder: EntityId,
    pub proposition: Proposition,
    pub confidence: BeliefConfidence,
    pub basis: Vec<BeliefBasis>,
    pub updated_at: WorldInstant,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum KnowledgeHolder {
    Entity(EntityId),
    /// Explicitly shared table knowledge; never implied by out-of-character chatter alone.
    Table,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum KnowledgeTarget {
    Fact(FactId),
    Claim(ClaimId),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KnowledgeRecord {
    pub campaign_id: CampaignId,
    pub holder: KnowledgeHolder,
    pub target: KnowledgeTarget,
    pub acquired_at: WorldInstant,
    pub source_event_id: EventId,
}
