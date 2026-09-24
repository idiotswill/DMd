//! Retained falling causes. The shared tactical cursor, not this record, owns work.
use crate::{CommandMeta, EntityId, RollResult, SpatialPoint};
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
    pub path: SpatialFall,
    pub stage: TacticalFallStage,
}
