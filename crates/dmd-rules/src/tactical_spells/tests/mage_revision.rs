//! Source tuple identity only; no Counterspell trigger/runtime admission is claimed.
use super::*;
use crate::tactical_creatures::{creature_definition, creature_source_pin};
use crate::tactical_definitions::bundled_mage_v2;

#[test]
fn mage_original_and_candidate_spell_tuples_bind_each_of_the_three_exact_revisions() {
    let defs = definitions().unwrap();
    let old = creature_definition("mage").unwrap();
    let new = bundled_mage_v2().unwrap();
    let export: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../dmd-app/tests/fixtures/shield-hit-v1-selected.json"
    ))
    .unwrap();
    let state = CampaignState::decode_json(export["current_state"]["state_json"].as_str().unwrap())
        .unwrap();
    let profile = state
        .rules
        .as_ref()
        .unwrap()
        .tactical_creatures
        .as_ref()
        .unwrap()
        .profiles
        .iter()
        .find(|p| p.source.definition_id == "mage")
        .unwrap();
    assert_eq!(profile.source, creature_source_pin(old).unwrap());
    // This profile clone tests a pure pin relation, never a restored positive history.
    let mut candidate_profile = profile.clone();
    candidate_profile.source = creature_source_pin(new).unwrap();
    for (spell_id, feature_id) in [
        ("mage-armor", "spellcasting"),
        ("shield", "protective-magic"),
        ("counterspell", "protective-magic"),
    ] {
        let spell = defs.spell(spell_id).unwrap();
        let old_feature = old.features.iter().find(|f| f.id == feature_id).unwrap();
        let new_feature = new.features.iter().find(|f| f.id == feature_id).unwrap();
        assert_eq!(old_feature, new_feature);
        let old_pin = source_pin(defs, spell, Some(old), Some(old_feature)).unwrap();
        let new_pin = source_pin(defs, spell, Some(new), Some(new_feature)).unwrap();
        assert_ne!(old_pin, new_pin);
        assert_eq!(
            creature_for_spell_source(defs, spell, &old_pin).unwrap(),
            old
        );
        assert_eq!(
            creature_for_spell_source(defs, spell, &new_pin).unwrap(),
            new
        );
        validate_creature_spell_source(profile, &old_pin).unwrap();
        validate_creature_spell_source(&candidate_profile, &new_pin).unwrap();
        assert!(validate_creature_spell_source(profile, &new_pin).is_err());
        assert!(validate_creature_spell_source(&candidate_profile, &old_pin).is_err());
        let mut forged = new_pin.clone();
        forged.fingerprint = "fnv1a64:0000000000000000".into();
        assert!(creature_for_spell_source(defs, spell, &forged).is_err());
        assert_eq!(
            source_components(defs, spell, &old_pin).unwrap(),
            source_components(defs, spell, &new_pin).unwrap()
        );
    }
}
