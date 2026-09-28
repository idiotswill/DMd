//! Flow4 evidence for committed Magic Missile targets, responses and impacts.
//! The ordinary resolution frames remain the only executable work queue.
use crate::{
    CommandMeta, EntityId, SpellProgramOccurrence, TacticalCasting, TacticalReactionOrderDecision,
    TacticalRollKey, TacticalShieldRespondent, TacticalWorkItem,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum TacticalMissileStage {
    Collecting,
    Selected { respondent: u16 },
    Casting { respondent: u16, cast: u16 },
    Amounts,
    Impacts,
    Completed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TacticalMissileRespondent {
    /// One acknowledgment per distinct committed target, regardless of source
    /// eligibility. Repeated darts never spend multiple Reactions on one Shield.
    pub response: TacticalShieldRespondent,
    /// Source evidence retained after this independently owned child has left
    /// the live casting registry. Never a second resource or effect authority.
    pub completed_shield: Option<Box<TacticalCasting>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TacticalMissileDart {
    pub at: SpellProgramOccurrence,
    pub target: EntityId,
    /// Seen by the current-turn caster at commitment, never learned later from
    /// another owned observer or changed visibility while consequences drain.
    pub known_target_label: String,
    /// Exact accepted raw request identity. Faces/amount/cause remain in the
    /// authoritative RecordedRoll; collecting it does not complete this dart.
    pub amount: Option<TacticalRollKey>,
    /// Allocated only when every target response and amount is complete. The
    /// whole impact set is then installed as one ordinary sibling frame.
    pub impact_occurrence: Option<u16>,
    /// The turn controller's explicit material choice. The final singleton is
    /// automatic and has none, even if a different owner's child save resumes it.
    pub selected_by: Option<CommandMeta>,
    /// Also retained for prevented/dead-target no-effects. A nested child may
    /// alter the target, but cannot remove or retarget a committed occurrence.
    pub completed_by: Option<CommandMeta>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TacticalMissile {
    /// Occurrence of the single live source casting record.
    pub cast: u16,
    /// Exact target-commitment trigger. Its work occurrence identifies this
    /// record and its private response window within the enclosing resolution.
    pub work: TacticalWorkItem,
    pub cause: CommandMeta,
    pub stage: TacticalMissileStage,
    pub delegated_by: Option<CommandMeta>,
    pub order: Option<TacticalReactionOrderDecision>,
    pub respondents: Vec<TacticalMissileRespondent>,
    pub darts: Vec<TacticalMissileDart>,
    /// Retired source proof, populated only after FinishSpell removes the live
    /// cast. Needed while an enclosing resolution retains this work's ancestry.
    pub completed_cast: Option<Box<TacticalCasting>>,
}
