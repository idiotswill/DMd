//! SRD190 Shove choices and evidence within the existing tactical continuation.
//! Retained facts are executor output, never client-supplied source permissions.
use crate::{
    Ability, CommandMeta, CreatureSourcePin, EntityId, RollRequest, SpatialPoint, TacticalRollKey,
    TacticalWorkItem, WeaponActionWindow,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShoveSaveAbility {
    Strength,
    Dexterity,
}

impl ShoveSaveAbility {
    pub fn ability(self) -> Ability {
        match self {
            Self::Strength => Ability::Strength,
            Self::Dexterity => Ability::Dexterity,
        }
    }
}

/// The shover supplies a consequence and, for Push, its intended endpoint. The
/// resolver authenticates distance/direction; only a later Host ruling moves it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum ShoveChoice {
    Prone,
    Push { destination: SpatialPoint },
}

/// Closed interpretation of the existing grid metric. This cannot grant source
/// support, change distance, override unsupported operations or choose Prone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShoveGeometryRuling {
    CommitExactPush,
    ConfirmBlockedNoMovement,
    ReturnToShover,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShoveGeometryConvention {
    GridFiveFootAwayV1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TacticalShoveStage {
    Queued,
    SaveChoice,
    Saving,
    OutcomeChoice,
    PushReview,
    Resolving,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TacticalShoveSave {
    pub ability: ShoveSaveAbility,
    pub chosen_by: CommandMeta,
    pub key: TacticalRollKey,
    /// Sealed before the selected save. None means a source automatic failure;
    /// no fake request or raw face is created. Completed rolls live in RulesState.
    pub request: Option<RollRequest>,
    pub final_success: Option<bool>,
    pub resolved_by: Option<CommandMeta>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TacticalShovePush {
    pub chosen_by: CommandMeta,
    pub from: SpatialPoint,
    pub destination: SpatialPoint,
    pub convention: ShoveGeometryConvention,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum TacticalShoveEffect {
    Prone {
        chosen_by: CommandMeta,
        /// False is the source-correct accepted no-effect result for immunity.
        applied: bool,
    },
    Push {
        intent: TacticalShovePush,
        ruled_by: CommandMeta,
    },
    BlockedPush {
        intent: TacticalShovePush,
        ruled_by: CommandMeta,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TacticalShove {
    pub origin: CommandMeta,
    pub actor: EntityId,
    pub target: EntityId,
    pub window: WeaponActionWindow,
    pub actor_source: Option<CreatureSourcePin>,
    pub target_source: Option<CreatureSourcePin>,
    pub actor_from: SpatialPoint,
    pub target_from: SpatialPoint,
    pub difficulty: i32,
    pub stage: TacticalShoveStage,
    /// A material decision selected from the single work stack, with exact trace
    /// ancestry. Raw save ownership remains TacticalResolution.pending instead.
    pub selected: Option<TacticalWorkItem>,
    pub save: Option<TacticalShoveSave>,
    pub push: Option<TacticalShovePush>,
    pub effect: Option<TacticalShoveEffect>,
}
