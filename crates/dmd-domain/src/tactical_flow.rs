//! Tactical accounting which is not represented by the legacy action/reaction budgets.
use crate::{
    CommandId, CommandMeta, EntityId, PlayerId, RollRequestId, TacticalDodge, TacticalResolution,
    TacticalSaveDecision, WeaponActionWindow, WeaponAttackReceipt,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TacticalTurnBudget {
    /// Shared expenditure across movement modes, measured in half-foot units.
    pub movement_spent: u32,
    /// Continuous run-up/jump context; source movement derives it and intervening
    /// non-movement actions/displacements end it. Absent in earlier schema-4 saves.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub movement_progress: Option<crate::TacticalMovementProgress>,
    /// Original accepted mover command for the most recently committed progress.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub movement_origin: Option<CommandMeta>,
    pub dash_grants: Vec<DashGrant>,
    /// Unresolved attacks granted by the current Attack action, not additional actions.
    pub attacks_remaining: u8,
    pub object_interaction_spent: bool,
    pub attack_window: Option<WeaponActionWindow>,
    /// Current global turn's receipts, including off-turn attacks; no duplicate Light,
    /// Nick, Cleave or Loading booleans can drift from these physical-identity records.
    pub weapon_history: Vec<WeaponAttackReceipt>,
    pub disengaged: Option<CommandMeta>,
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

/// An explicit GM timing interpretation after fighting stops. This is not a
/// source rule which dismisses effects or manufactures turns for their owners.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AftermathCadence {
    ContinueExistingOrder,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TacticalAftermath {
    pub origin: CommandMeta,
    pub cadence: AftermathCadence,
    /// Private host explanation; not an asserted victory, surrender or reward.
    pub ruling: String,
    pub concluded_at: crate::WorldInstant,
    pub concluded_on_turn: crate::EffectTurn,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    // The growing interrupt cursor lives on the heap. Keeping it inline inflated
    // every CampaignState async/serde stack frame even outside an encounter.
    // Box is transparent on the existing durable JSON wire.
    pub resolution: Option<Box<TacticalResolution>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dodges: Vec<TacticalDodge>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub save_decisions: Vec<TacticalSaveDecision>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ground_items: Vec<crate::TacticalGroundItem>,
    /// Historical reached-place evidence survives the next turn's budget reset.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_movement: Option<crate::TacticalMovementResult>,
    /// Ready survives other actors' resolutions and ends before its owner's next
    /// turn. The declaration owns its paid Action; release owns a later Reaction.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ready: Vec<crate::TacticalReady>,
    /// Hostilities can end while source timing continues on the original order.
    /// Box avoids increasing every historical CampaignState stack frame; absent
    /// preserves the exact old durable JSON and presentation contract.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub aftermath: Option<Box<TacticalAftermath>>,
}
