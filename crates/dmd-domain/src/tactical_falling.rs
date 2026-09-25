//! Retained falling causes. The shared tactical cursor, not this record, owns work.
use crate::{CommandMeta, EntityId, RollResult, SpatialPoint, TacticalRollKey};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum FallSurface {
    Floor,
    SolidObstacle { id: String },
    SupportingTerrain { id: String },
    Liquid { id: String },
}

/// Source-independent authored-map geometry, measured in half-feet.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpatialFall {
    pub from: SpatialPoint,
    pub to: SpatialPoint,
    pub surface: FallSurface,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LiquidLandingChoice {
    Athletics,
    Acrobatics,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum TacticalFallStage {
    /// Retained source occurrence awaiting its place in the existing work frames.
    Queued,
    /// A possible liquid Reaction remains an explicit controller decision.
    LandingChoice,
    LandingCheck {
        choice: LiquidLandingChoice,
        accepted_by: CommandMeta,
    },
    Damage {
        /// None means no Reaction was used. The raw result remains in RulesState.
        landing: Option<TacticalLiquidLanding>,
    },
    Complete {
        landing: Option<TacticalLiquidLanding>,
        /// None for a short fall or an already dead body; no fabricated raw roll.
        damage: Option<TacticalRollKey>,
        resolved_by: CommandMeta,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum TacticalFallCause {
    /// A committed final ordinary-movement segment ended without physical support.
    MovementEnd {
        movement: CommandMeta,
        /// Zero-based submitted step which was actually committed before falling.
        step_index: u16,
    },
    /// Current source mechanics no longer sustain this creature's flight.
    FlightLost,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TacticalLiquidLanding {
    pub choice: LiquidLandingChoice,
    pub accepted_by: CommandMeta,
    pub result: RollResult,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TacticalFall {
    pub origin: CommandMeta,
    pub actor: EntityId,
    pub cause: TacticalFallCause,
    pub path: SpatialFall,
    pub stage: TacticalFallStage,
}
