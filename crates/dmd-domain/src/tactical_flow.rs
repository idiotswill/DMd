//! Tactical accounting which is not represented by the legacy action/reaction budgets.
use crate::{CommandId, CommandMeta, EntityId, PlayerId, RollRequestId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TacticalTurnBudget {
    /// Shared expenditure across movement modes, measured in half-foot units.
    pub movement_spent: u32,
    pub dash_grants: Vec<DashGrant>,
    /// Unresolved attacks granted by the current Attack action, not additional actions.
    pub attacks_remaining: u8,
    pub object_interaction_spent: bool,
    pub light_attack_earned_with: Option<String>,
    pub light_attack_spent: bool,
    pub nick_spent: bool,
    pub cleave_spent: bool,
    /// Every source of a slotted cast on this turn except the active actor, whose existing
    /// CombatTiming.slot_spent_this_turn is authoritative. Clear at every turn boundary.
    pub other_slot_casters: Vec<EntityId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DashSpeed {
    Speed,
    Climb,
    Swim,
    Fly,
    Burrow,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DashGrant {
    pub speed: DashSpeed,
    pub origin: CommandId,
}

/// The source used for combat statistics. Monster values come from the pinned
/// stat block rather than the PC level/proficiency formulas.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum TacticalSource {
    Character,
    Creature { definition_id: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TacticalCombatant {
    pub actor: EntityId,
    pub source: TacticalSource,
    pub surprised: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InitiativeGroup {
    pub actors: Vec<EntityId>,
    pub request_id: RollRequestId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InitiativeTie {
    pub total: i32,
    pub actors: Vec<EntityId>,
    pub proposed_order: Option<Vec<EntityId>>,
    /// Player-only ties require agreement from every affected controller.
    pub accepted_by: Vec<PlayerId>,
    pub host_decided: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum TacticalPhase {
    Initiative { next_group: usize },
    InitiativeTies { ties: Vec<InitiativeTie> },
    Active,
    Finished,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TacticalFlow {
    pub version: u32,
    pub origin: CommandMeta,
    pub combatants: Vec<TacticalCombatant>,
    pub initiative_groups: Vec<InitiativeGroup>,
    pub initiative_decisions: Vec<InitiativeTie>,
    pub phase: TacticalPhase,
    pub budget: TacticalTurnBudget,
}
