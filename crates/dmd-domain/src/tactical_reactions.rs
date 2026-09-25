//! Source reaction and Ready identities. These records describe accepted work;
//! they do not grant a client permission to interrupt an arbitrary occurrence.
use crate::{
    CommandId, CommandMeta, EntityId, SpatialPoint, SpellCastChoice, TacticalCasting,
    TacticalWorkItem,
};
use serde::{Deserialize, Serialize};

pub const MAX_TACTICAL_READY: usize = 128;
pub const MAX_TACTICAL_REACTION_DEPTH: usize = 32;

/// Explicit current-turn instruction over potential participants. None of these
/// choices has a default: private eligibility or arrival order never chooses it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReactionUnlistedOrder {
    BeforeForward,
    BeforeReverse,
    AfterForward,
    AfterReverse,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TacticalReactionOrdering {
    /// Controller-ranked known participants, whether eligible or not. This list
    /// must never be generated from the private accepted-response set.
    pub ranked: Vec<EntityId>,
    pub unlisted: ReactionUnlistedOrder,
}

/// Semantic executor version, independent of the campaign JSON schema. Missing
/// fields in old accepted Begin events mean Legacy; live admission cannot choose
/// that version to suppress a mandatory source interruption.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum TacticalExecutionVersion {
    #[default]
    Legacy,
    ReactionsV1,
}

impl TacticalExecutionVersion {
    pub fn is_legacy(&self) -> bool {
        *self == Self::Legacy
    }

    pub fn flow_version(self) -> u32 {
        match self {
            Self::Legacy => 1,
            Self::ReactionsV1 => 2,
        }
    }
}

/// A local work occurrence is meaningful only inside its original resolution.
/// The transport may substitute an audience-bound opaque handle for this pair.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TacticalWorkKey {
    pub resolution: CommandId,
    pub occurrence: u16,
}

/// Causal ancestry, not another work queue. Completed parents are retained until
/// their resolution ends so ordering consent cannot leak across an interruption.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TacticalWorkNode {
    pub work: TacticalWorkItem,
    pub parent: Option<u16>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TacticalWorkTrace {
    pub nodes: Vec<TacticalWorkNode>,
    /// Only an in-process stack marker. Every completed public transition must
    /// reset it; no wire payload can supply an execution context.
    #[serde(skip)]
    pub active: Option<u16>,
}

/// A perception requirement, not a list of hostile or known creatures. `AnyOther`
/// never permits a hidden actor to become an offered trigger without a witness.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReadySubject {
    Creature(EntityId),
    AnyOther,
}

/// Supported mechanical milestones finish before a Ready response begins. An
/// abstract circumstance uses a separately journaled host adjudication tied to
/// a genuine completed occurrence; its description alone cannot trigger work.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum ReadyTrigger {
    MovementFinished { subject: ReadySubject },
    AttackFinished { subject: ReadySubject },
    SpellFinished { subject: ReadySubject },
    Adjudicated { description: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum ReadyAction {
    Attack,
    Move,
    /// The spell is cast when declared, with its genuine source grant, material,
    /// slot/source expenditure and concentration. Targets are selected on release.
    Spell {
        choice: Box<SpellCastChoice>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TacticalReady {
    pub origin: CommandMeta,
    pub actor: EntityId,
    pub declared_on_turn: u64,
    pub trigger: ReadyTrigger,
    pub action: ReadyAction,
    /// Present only for a successfully cast spell in its Held phase. This lives
    /// outside the current resolution so other actors can actually take turns.
    pub held_spell: Option<Box<TacticalCasting>>,
}

/// Observable source milestones contain only the completed action's facts.
/// Authoritative perception is rechecked for each potential respondent; these
/// private records are never sent wholesale to a player projection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum TacticalReactionTrigger {
    AttackHit {
        attack_origin: CommandId,
        target: EntityId,
    },
    MagicMissileTarget {
        casting_origin: CommandId,
        cast: u16,
        target: EntityId,
    },
    Casting {
        casting_origin: CommandId,
        cast: u16,
        caster: EntityId,
    },
    MovementFinished {
        actor: EntityId,
        from: SpatialPoint,
        to: SpatialPoint,
    },
    AttackFinished {
        actor: EntityId,
    },
    SpellFinished {
        actor: EntityId,
    },
    Adjudicated {
        declaration: CommandId,
        completed: TacticalWorkKey,
        adjudicated_by: CommandMeta,
        reason: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum TacticalReactionOffer {
    Shield,
    Counterspell,
    Ready { declaration: CommandId },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TacticalReactionWindow {
    pub work: TacticalWorkItem,
    /// The actual command which caused this milestone, not the reacting actor's
    /// declaration and not a fabricated command attributed to the victim.
    pub cause: CommandMeta,
    pub respondent: EntityId,
    pub trigger: TacticalReactionTrigger,
    pub offers: Vec<TacticalReactionOffer>,
}
