use serde::{Deserialize, Serialize};

use crate::{
    AgentRef, BeliefId, CampaignId, ClaimId, EntityId, EventId, FactId, FactionId, ItemId,
    KnowledgeId, LocationId, WorldInstant,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClaimSource {
    Agent(AgentRef),
    /// A document, ledger, inscription object, recording, or similar source whose author may be unknown.
    Item(ItemId),
    /// A fixed inscription, notice, environmental message, or other location-bound source.
    Location(LocationId),
    /// Rumor/hearsay whose origin is intentionally unresolved.
    Unknown,
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
    /// Where the assertion came from. A source is evidence/provenance, never proof of truth.
    pub source: ClaimSource,
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
    /// Individual or institution whose internal model contains this belief.
    pub holder: AgentRef,
    pub proposition: Proposition,
    pub confidence: BeliefConfidence,
    pub basis: Vec<BeliefBasis>,
    pub updated_at: WorldInstant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KnowledgeHolder {
    Agent(AgentRef),
    /// Explicitly shared table knowledge; never implied by out-of-character chatter alone.
    Table,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KnowledgeTarget {
    Fact(FactId),
    Claim(ClaimId),
}

/// Current materialized knowledge relation.
/// Repeated observations/acquisitions belong in the event journal rather than duplicate rows here.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KnowledgeRecord {
    pub id: KnowledgeId,
    pub campaign_id: CampaignId,
    pub holder: KnowledgeHolder,
    pub target: KnowledgeTarget,
    pub acquired_at: WorldInstant,
    pub source_event_id: EventId,
}
