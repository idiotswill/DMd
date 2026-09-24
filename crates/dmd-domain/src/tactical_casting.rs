//! Internal casting evidence retained by the one tactical resolution. These records
//! are not player permissions: source validation and journal replay authenticate them.
use crate::{CommandMeta, EntityId, ItemId, SpellCast, SpellTargetChoice};
use serde::{Deserialize, Serialize};

pub const MAX_TACTICAL_CASTS: usize = 128;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpellBoundTarget {
    pub actor: EntityId,
    /// Private source truth used for SRD106's apparent successful save/no effect.
    /// Player projections must never expose this as an explanation.
    pub source_type_matches: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpellEnclosingActivation {
    Action,
    BonusAction,
    Legendary,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpellCreatureActivation {
    /// A Multiattack's enclosing command can precede this spell's invocation.
    pub origin: CommandMeta,
    pub activation: SpellEnclosingActivation,
    pub attack_action: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TacticalCasting {
    /// Heap storage also keeps CampaignState async and serde stack frames bounded.
    pub cast: Box<SpellCast>,
    /// None while Ready holds energy; targets are chosen at the genuine release.
    pub selection: Option<SpellTargetChoice>,
    /// Ordered occurrences, including repeated rays/darts. These are never sorted.
    pub targets: Vec<SpellBoundTarget>,
    pub consumed_material: Option<ItemId>,
    /// Created only from a genuine source feature result, never a client prepaid flag.
    pub creature_activation: Option<SpellCreatureActivation>,
    /// Node/target occurrences already applied. The shared work frames own ordering;
    /// this evidence rejects duplicate application after a resumed child consequence.
    pub completed: Vec<SpellProgramOccurrence>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpellProgramOccurrence {
    pub node: u8,
    pub target: u16,
}

/// Stable identity shared by raw replay and lifecycle installation. A condition's
/// view ID is deliberately distinct from its owning effect and concentration IDs.
pub fn spell_program_effect_id(
    command: crate::CommandId,
    actor: EntityId,
    cast_occurrence: u16,
    at: SpellProgramOccurrence,
    condition_view: bool,
) -> crate::EffectId {
    let mut name = b"dmd.tactical.spell.effect.v1\0".to_vec();
    name.extend_from_slice(actor.0.as_bytes());
    name.extend_from_slice(&cast_occurrence.to_be_bytes());
    name.push(at.node);
    name.extend_from_slice(&at.target.to_be_bytes());
    name.push(u8::from(condition_view));
    crate::EffectId(uuid::Uuid::new_v5(&command.0, &name))
}
