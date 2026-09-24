use crate::{EntityId, RollRequestId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RollMode {
    Normal,
    Advantage,
    Disadvantage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RollVisibility {
    Public,
    Private,
    Secret,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RollSource {
    /// A human rolled physical dice and reported the faces to the runtime.
    Physical,
    /// The runtime generated the dice result.
    Digital,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DieSpec {
    pub count: u16,
    pub sides: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DieResult {
    pub sides: u16,
    pub value: u16,
}

/// An authoritative request for raw dice input.
///
/// The modifier and roll mode belong to the request so a physical-dice result only supplies raw
/// faces. Players/providers do not submit a precomputed authoritative total.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RollRequest {
    pub id: RollRequestId,
    pub roller: Option<EntityId>,
    pub dice: Vec<DieSpec>,
    pub modifier: i32,
    pub mode: RollMode,
    pub visibility: RollVisibility,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RollResult {
    pub request_id: RollRequestId,
    pub source: RollSource,
    pub dice: Vec<DieResult>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResolvedRoll {
    pub request_id: RollRequestId,
    pub source: RollSource,
    pub raw_dice: Vec<DieResult>,
    pub kept_dice: Vec<DieResult>,
    pub modifier: i32,
    pub total: i32,
}
