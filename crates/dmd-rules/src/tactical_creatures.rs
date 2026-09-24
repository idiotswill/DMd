//! Source-faithful creature capabilities and deterministic limited-use execution.
mod policy;
mod profile;
mod schedule;
pub use policy::*;
pub use profile::*;
pub use schedule::*;

use crate::tactical_definitions::{
    CreatureDefinition, TACTICAL_DEFINITIONS_JSON, TacticalDefinitions,
};
use dmd_domain::*;
use std::sync::OnceLock;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum CreatureError {
    #[error("invalid creature state or source: {0}")]
    Invalid(String),
    #[error("creature operation is not authorized")]
    Unauthorized,
    #[error("creature command observes a stale head")]
    Stale,
    #[error("creature capability is unavailable: {0}")]
    Unavailable(String),
}
fn invalid(message: impl Into<String>) -> CreatureError {
    CreatureError::Invalid(message.into())
}

pub fn creature_definitions() -> Result<&'static TacticalDefinitions, CreatureError> {
    static DEFINITIONS: OnceLock<Result<TacticalDefinitions, CreatureError>> = OnceLock::new();
    DEFINITIONS
        .get_or_init(|| {
            TacticalDefinitions::from_json(TACTICAL_DEFINITIONS_JSON)
                .map_err(|error| invalid(error.to_string()))
        })
        .as_ref()
        .map_err(Clone::clone)
}
pub fn creature_definition(id: &str) -> Result<&'static CreatureDefinition, CreatureError> {
    creature_definitions()?
        .creature(id)
        .ok_or_else(|| invalid("unknown source creature"))
}
pub fn creature_definition_fingerprint(
    definition: &CreatureDefinition,
) -> Result<String, CreatureError> {
    let bytes = serde_json::to_vec(definition).map_err(|error| invalid(error.to_string()))?;
    let hash = bytes.iter().fold(0xcbf29ce484222325u64, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x100000001b3)
    });
    Ok(format!("{hash:016x}"))
}
pub fn source_for_profile(
    profile: &CreatureProfile,
) -> Result<&'static CreatureDefinition, CreatureError> {
    let definitions = creature_definitions()?;
    let source = creature_definition(&profile.source.definition_id)?;
    if profile.source.ruleset_id != definitions.ruleset_id
        || profile.source.ruleset_version != definitions.ruleset_version
        || profile.source.definition_fingerprint != creature_definition_fingerprint(source)?
    {
        return Err(invalid(
            "creature source pin or definition fingerprint differs",
        ));
    }
    Ok(source)
}
