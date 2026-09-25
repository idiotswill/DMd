//! Retained source area work, resolved by the single tactical continuation.
//! A public declaration chooses an aim, never targets or authoritative rule values.
use crate::{
    CommandMeta, CoverDegree, CreatureSourcePin, EntityId, SpatialBox, SpatialPoint,
    TacticalRollKey,
};
use serde::{Deserialize, Serialize};

/// An explicit host map adjudication. SRD177/179 define shape and blocking, not
/// how a three-dimensional creature volume is rasterized onto a particular map.
/// The chosen policy belongs to encounter setup, not to an actor's declaration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TacticalAreaGridPolicy {
    /// Test centers of every occupied part of each 5ft grid cell/vertical band;
    /// round an unrepresentable midpoint toward the occupied minimum. At least
    /// half/three quarters of these rays blocked by Total-cover terrain grants
    /// Half/ThreeQuarters cover, combined with authored cover and creature cover
    /// by taking the greatest grade. At least one in-shape, unblocked sample is
    /// required. This convention does not grant spreading around corners.
    OccupiedCellCentersV1,
}

/// `toward` specifies a direction from `origin`, not a victim or a target location
/// with a sight requirement. The source fixes all dimensions and the allowed shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TacticalAreaAim {
    pub origin: SpatialPoint,
    pub toward: SpatialPoint,
    pub include_origin: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TacticalAreaSource {
    pub actor: EntityId,
    pub pin: CreatureSourcePin,
    pub feature_id: String,
    /// This primitive's accepted source invocation, separate from a future paid
    /// enclosing Multiattack. No client can assert a prepaid resource permission.
    pub invocation: CommandMeta,
    pub enclosing_origin: CommandMeta,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TacticalAreaSave {
    /// The existing roll/explicit-save-decision journal owns the actual outcome.
    pub key: TacticalRollKey,
    pub succeeded: bool,
    pub resolved_by: CommandMeta,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TacticalAreaTarget {
    pub actor: EntityId,
    /// Geometry at the original simultaneous effect, not a later fallen position.
    pub volume: SpatialBox,
    pub cover: CoverDegree,
    pub save: Option<TacticalAreaSave>,
    pub applied_by: Option<CommandMeta>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TacticalAreaStage {
    DamageRoll,
    SavingThrows,
    ApplyingDamage,
    Complete,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TacticalArea {
    /// Allocated by the one shared resolution, never a vector index or random ID.
    pub occurrence: u16,
    pub source: TacticalAreaSource,
    pub aim: TacticalAreaAim,
    pub policy: TacticalAreaGridPolicy,
    pub geometry_origin: CommandMeta,
    /// Truth-only membership. Player projections must not expose unseen actors,
    /// their count, their cover or their private save/concentration work.
    pub targets: Vec<TacticalAreaTarget>,
    /// Exactly one shared source amount; each victim uses this same accepted roll.
    pub damage: Option<TacticalRollKey>,
    pub stage: TacticalAreaStage,
}
