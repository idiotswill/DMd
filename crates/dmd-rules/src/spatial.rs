//! Deterministic SRD 5.2.1 geometry/perception queries. Nothing here commits a command.
//! Half-foot integer units preserve Tiny spaces and exact heights without floating point.
mod areas;
mod geometry;
mod movement;
mod perception;
#[cfg(test)]
mod tests;
pub use areas::*;
use dmd_domain::*;
pub use geometry::{CoverAssessment, cover_from, grid_distance, segment_intersects};
pub use movement::*;
pub use perception::*;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum SpatialError {
    #[error("invalid spatial state: {0}")]
    Invalid(String),
    #[error("unknown tactical participant")]
    UnknownActor,
    #[error("spatial query exceeds bounded capacity")]
    Capacity,
    #[error("spatial action is not legal: {0}")]
    Illegal(String),
}
fn invalid(message: impl Into<String>) -> SpatialError {
    SpatialError::Invalid(message.into())
}
fn illegal(message: impl Into<String>) -> SpatialError {
    SpatialError::Illegal(message.into())
}
pub fn validate_encounter(
    encounter: &TacticalEncounter,
    state: &CampaignState,
) -> Result<(), SpatialError> {
    encounter.validate(state).map_err(invalid)
}
fn participant(
    encounter: &TacticalEncounter,
    actor: EntityId,
) -> Result<&TacticalParticipant, SpatialError> {
    encounter
        .participant(actor)
        .ok_or(SpatialError::UnknownActor)
}
fn conditions(state: &CampaignState, actor: EntityId) -> Vec<Condition> {
    state.rules.as_ref().map_or_else(Vec::new, |r| {
        crate::active_conditions(r, actor).into_iter().collect()
    })
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum SpatialTarget {
    Entity(EntityId),
    Location(SpatialPoint),
}
