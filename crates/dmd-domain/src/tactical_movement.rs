//! Retained movement intent and source-derived opportunity choices.
//! These records describe accepted work; deserializing one never grants movement.
use crate::{CommandMeta, EntityId, ItemId, MovementMode, SpatialPoint};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TacticalMoveStep {
    pub destination: SpatialPoint,
    pub mode: MovementMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TacticalJumpProgress {
    pub start: SpatialPoint,
    pub had_runup: bool,
}

/// Consecutive collinear forward travel, independent of terrain expenditure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TacticalStraightMovement {
    pub start: SpatialPoint,
    pub end: SpatialPoint,
}

/// Continuous movement context, measured in half-feet. The spatial evaluator derives
/// this; the turn budget retains it across accepted path segments and reactions.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TacticalMovementProgress {
    pub walked_runup: u32,
    pub jump: Option<TacticalJumpProgress>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub straight: Option<TacticalStraightMovement>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum TacticalMeleeSource {
    Weapon {
        item: ItemId,
    },
    Unarmed,
    CreatureFeature {
        feature_id: String,
        /// Source Gear requires a real currently held physical weapon. Intrinsic
        /// attacks have no retrievable implement (SRD255).
        weapon: Option<ItemId>,
    },
}

/// Internal source query result, not a client-supplied reach permission.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TacticalMeleeOption {
    pub source: TacticalMeleeSource,
    pub reach: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TacticalMovementReceipt {
    pub cause: CommandMeta,
    pub from: SpatialPoint,
    pub to: SpatialPoint,
    pub mode: MovementMode,
    pub cost: u32,
    pub progress_after: TacticalMovementProgress,
}

/// One selected opportunity before a segment commits. Other same-time opportunities
/// remain in the shared resolution frames, not in a competing movement queue.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TacticalOpportunityWindow {
    pub origin: CommandMeta,
    pub reactor: EntityId,
    pub mover: EntityId,
    pub step_index: u16,
    pub from: SpatialPoint,
    pub to: SpatialPoint,
    pub options: Vec<TacticalMeleeOption>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TacticalOpportunityDecisionKind {
    Declined,
    Attack,
    /// Derived consequences made the crossing/attack unavailable before its offer;
    /// preserves the actual causing command without inventing a player's decline.
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TacticalOpportunityDecision {
    pub reactor: EntityId,
    pub origin: CommandMeta,
    pub kind: TacticalOpportunityDecisionKind,
}

/// Retained ordinary movement. Actual current position and expenditure stay in the
/// encounter participant and turn budget. The initial image/receipts explain how the
/// accepted path reached its current cursor, which semantic replay verifies.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TacticalMovement {
    pub origin: CommandMeta,
    pub actor: EntityId,
    pub path: Vec<TacticalMoveStep>,
    pub initial_position: SpatialPoint,
    pub initial_spent: u32,
    pub initial_progress: TacticalMovementProgress,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub initial_progress_origin: Option<CommandMeta>,
    pub next_step: u16,
    pub traversed: Vec<TacticalMovementReceipt>,
    /// Actors whose opportunity at this crossing has already been offered. Clearing
    /// a selected attack must not reopen its trigger, even if it declined or missed.
    pub offered: Vec<EntityId>,
    pub decisions: Vec<TacticalOpportunityDecision>,
    pub opportunity: Option<TacticalOpportunityWindow>,
}
