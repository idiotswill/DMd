//! Explicit immutable current creation source. Old ID-only creation stays unchanged.
use super::*;

pub fn physical_starter_catalog() -> StarterCatalog {
    serde_json::from_str(include_str!(
        "../../../../content/srd-5.2.1/character-creation-physical-v1.json"
    ))
    .expect("reviewed physical creation source is valid")
}

pub fn current_character_creation_source() -> Result<CharacterCreationSourcePin, RulesError> {
    let catalog = physical_starter_catalog();
    let bytes = serde_json::to_vec(&catalog).map_err(|error| invalid(&error.to_string()))?;
    let hash = bytes.iter().fold(0xcbf29ce484222325u64, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x100000001b3)
    });
    Ok(CharacterCreationSourcePin {
        ruleset_id: catalog.ruleset_id,
        ruleset_version: catalog.version,
        catalog_schema_version: catalog.schema_version,
        profile_id: catalog.profile_id,
        definition_fingerprint: format!("{hash:016x}"),
    })
}

pub fn build_character_from_source(
    input: &CharacterCreationInput,
    entity_id: EntityId,
    source: &CharacterCreationSourcePin,
    pack: &RulesPack,
) -> Result<BuiltCharacter, RulesError> {
    source.validate_shape().map_err(|error| invalid(&error))?;
    if *source != current_character_creation_source()? {
        return Err(invalid(
            "character creation source pin or fingerprint differs",
        ));
    }
    build_with_catalog(
        input,
        entity_id,
        pack,
        &physical_starter_catalog(),
        Some(source.clone()),
    )
}
