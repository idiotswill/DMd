//! Source-faithful creature capabilities and deterministic limited-use execution.
mod policy;
mod profile;
mod schedule;
pub use policy::*;
pub use profile::*;
pub use schedule::*;

use crate::tactical_definitions::{
    CreatureDefinition, TacticalDefinitions, bundled_tactical_definitions,
};
use dmd_domain::*;
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
    bundled_tactical_definitions().map_err(|error| invalid(error.to_string()))
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
    creature_source(&profile.source)
}

/// Full immutable identity lookup. ID-only lookup above deliberately remains V1.
pub fn creature_source(
    pin: &CreatureSourcePin,
) -> Result<&'static CreatureDefinition, CreatureError> {
    for source in immutable_creature_sources()? {
        if source.id == pin.definition_id && creature_source_pin(source)? == *pin {
            return Ok(source);
        }
    }
    Err(invalid(
        "creature source pin or definition fingerprint differs",
    ))
}

pub fn creature_source_pin(
    source: &CreatureDefinition,
) -> Result<CreatureSourcePin, CreatureError> {
    let definitions = creature_definitions()?;
    Ok(CreatureSourcePin {
        ruleset_id: definitions.ruleset_id.clone(),
        ruleset_version: definitions.ruleset_version.clone(),
        definition_id: source.id.clone(),
        definition_fingerprint: creature_definition_fingerprint(source)?,
    })
}

/// All immutable revisions, including any that may cease to be current admissions.
pub fn immutable_creature_sources() -> Result<Vec<&'static CreatureDefinition>, CreatureError> {
    let mut sources: Vec<_> = creature_definitions()?.creatures.iter().collect();
    sources.push(
        crate::tactical_definitions::bundled_air_elemental().map_err(|e| invalid(e.to_string()))?,
    );
    Ok(sources)
}

/// Current admission does not supersede any V1 source in this additive slice.
pub fn current_creature_sources() -> Result<Vec<&'static CreatureDefinition>, CreatureError> {
    immutable_creature_sources()
}

/// A retained profile always wins over the flow's historical ID-only reference.
/// A profileless actor can only resolve a frozen V1 definition.
pub fn source_for_actor(
    state: &CampaignState,
    actor: EntityId,
    definition_id: &str,
) -> Result<&'static CreatureDefinition, CreatureError> {
    if let Some(profile) = state
        .rules
        .as_ref()
        .and_then(|r| r.tactical_creatures.as_ref())
        .and_then(|c| c.profile(actor))
    {
        if profile.source.definition_id != definition_id {
            return Err(invalid("actor and flow source identities differ"));
        }
        source_for_profile(profile)
    } else {
        creature_definition(definition_id)
    }
}

pub fn has_air_form(source: &CreatureDefinition) -> bool {
    source
        .traits
        .iter()
        .any(|t| matches!(t, crate::tactical_definitions::MonsterTrait::AirForm { .. }))
}

/// Limits are current affordance data, never part of historical presentations.
pub fn creature_execution_limits(source: &CreatureDefinition) -> Vec<String> {
    if has_air_form(source) {
        vec!["Air Form special shared-space movement and obstacle-contact/narrow-passage geometry are not yet executable. Unobstructed movement and ordinary permitted creature transit are available.".into(),
            "Multiattack and Whirlwind are not yet executable.".into()]
    } else {
        vec![]
    }
}
