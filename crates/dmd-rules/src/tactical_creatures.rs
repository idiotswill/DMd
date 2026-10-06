//! Source-faithful creature capabilities and deterministic limited-use execution.
mod ogre;
mod policy;
mod profile;
mod schedule;
pub use ogre::*;
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

/// All immutable revisions, including registry-only sources not yet admitted.
/// Presence here never grants creation or historical profile authority.
pub fn immutable_creature_sources() -> Result<Vec<&'static CreatureDefinition>, CreatureError> {
    let mut sources: Vec<_> = creature_definitions()?.creatures.iter().collect();
    sources.push(
        crate::tactical_definitions::bundled_air_elemental().map_err(|e| invalid(e.to_string()))?,
    );
    sources.push(
        crate::tactical_definitions::bundled_goblin_warrior_v2()
            .map_err(|e| invalid(e.to_string()))?,
    );
    sources.push(crate::tactical_definitions::bundled_ogre().map_err(|e| invalid(e.to_string()))?);
    sources
        .push(crate::tactical_definitions::bundled_mage_v2().map_err(|e| invalid(e.to_string()))?);
    Ok(sources)
}

/// Current admission selects the reviewed revision; immutable lookup and ID-only
/// V1 helpers retain every historical source identity.
pub fn current_creature_sources() -> Result<Vec<&'static CreatureDefinition>, CreatureError> {
    let legacy_goblin = creature_source_pin(creature_definition("goblin-warrior")?)?;
    let legacy_mage = creature_source_pin(creature_definition("mage")?)?;
    immutable_creature_sources()?
        .into_iter()
        .filter_map(|source| match creature_source_pin(source) {
            Ok(pin) if pin == legacy_goblin || pin == legacy_mage => None,
            Ok(_) => Some(Ok(source)),
            Err(error) => Some(Err(error)),
        })
        .collect()
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
