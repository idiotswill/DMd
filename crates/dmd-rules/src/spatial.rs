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
fn dead(state: &CampaignState, actor: EntityId) -> bool {
    state
        .rules
        .as_ref()
        .and_then(|rules| rules.entities.get(&actor))
        .is_some_and(|entity| entity.death.dead)
        || state
            .entities
            .get(&actor)
            .is_some_and(|entity| entity.existence == EntityExistence::Dead)
}
fn aware(state: &CampaignState, actor: EntityId) -> bool {
    // Unaware (SRD191) suppresses awareness regardless of sensing modality. Neither
    // Incapacitated nor Stunned alone carries that clause (SRD184/189).
    !dead(state, actor) && !conditions(state, actor).contains(&Condition::Unconscious)
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum SpatialTarget {
    Entity(EntityId),
    Location(SpatialPoint),
}
