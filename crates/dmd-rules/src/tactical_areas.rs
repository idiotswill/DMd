//! Source-bound area primitives. Public inputs supply aim, never membership or DC.
//! The tactical driver alone attaches these proofs and applies source costs.
mod damage;
mod geometry;
mod source;
#[cfg(test)]
mod tests;
pub use damage::{area_amount_request, area_damage_operation};
pub use geometry::{BoundAreaGeometry, bind_area_geometry};
pub use source::{AreaProgram, source_area_program};

use crate::RulesError;
use dmd_domain::*;

fn invalid(message: impl Into<String>) -> RulesError {
    RulesError::Invalid(message.into())
}
fn unavailable(message: impl Into<String>) -> RulesError {
    RulesError::Prerequisite(message.into())
}
