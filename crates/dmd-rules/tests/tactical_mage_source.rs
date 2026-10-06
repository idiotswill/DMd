//! Immutable source controls. Pure builder installation below is not gameplay history.
use dmd_domain::*;
use dmd_rules::{RulesPack, tactical_creatures::*, tactical_definitions::*};
use std::collections::BTreeSet;

#[test]
fn mage_v2_changes_only_reviewed_anatomy_and_keeps_exact_original_registry() {
    let old = creature_definition("mage").unwrap();
    let new = bundled_mage_v2().unwrap();
    assert!(old.ordinary_hands.is_none());
    let old_pin = creature_source_pin(old).unwrap();
    assert_eq!(
        old_pin,
        CreatureSourcePin {
            ruleset_id: "srd-5.2".into(),
            ruleset_version: "5.2.1".into(),
            definition_id: "mage".into(),
            definition_fingerprint: "af0f81ba7833b9c4".into(),
        }
    );
    let mut expected = old.clone();
    expected.ordinary_hands = Some(OrdinaryHandAnatomy::TwoHandsV1);
    assert_eq!(new, &expected);
    assert!(
        serde_json::to_value(old)
            .unwrap()
            .get("ordinary_hands")
            .is_none()
    );
    let new_pin = creature_source_pin(new).unwrap();
    assert_ne!(old_pin, new_pin);
    assert_eq!(creature_source(&old_pin).unwrap(), old);
    assert_eq!(creature_source(&new_pin).unwrap(), new);
    assert_eq!(creature_definition("mage").unwrap(), old);
    let original = creature_definitions()
        .unwrap()
        .creatures
        .iter()
        .chain([
            bundled_air_elemental().unwrap(),
            bundled_goblin_warrior_v2().unwrap(),
            bundled_ogre().unwrap(),
        ])
        .map(|source| serde_json::to_string(&creature_source_pin(source).unwrap()).unwrap())
        .collect::<BTreeSet<_>>();
    assert_eq!(original.len(), 13);
    let all = immutable_creature_sources().unwrap();
    assert_eq!(all.len(), 14);
    let all_pins = all
        .iter()
        .map(|source| serde_json::to_string(&creature_source_pin(source).unwrap()).unwrap())
        .collect::<BTreeSet<_>>();
    assert_eq!(all_pins.len(), 14);
    let mut expected_pins = original;
    assert!(expected_pins.insert(serde_json::to_string(&new_pin).unwrap()));
    assert_eq!(all_pins, expected_pins);
    let current = current_creature_sources().unwrap();
    assert_eq!(
        current
            .iter()
            .filter(|source| source.id == "mage")
            .copied()
            .collect::<Vec<_>>(),
        vec![new]
    );
    for field in 0..4 {
        let mut forged = new_pin.clone();
        match field {
            0 => forged.ruleset_id = "other".into(),
            1 => forged.ruleset_version = "other".into(),
            2 => forged.definition_id = "other".into(),
            _ => forged.definition_fingerprint = "0000000000000000".into(),
        }
        assert!(creature_source(&forged).is_err());
    }
}

#[test]
fn both_mage_revisions_keep_original_exact_physical_allocation_and_omissions() {
    let original = creature_definition("mage").unwrap();
    let candidate = bundled_mage_v2().unwrap();
    let old_plan =
        dmd_rules::tactical_creature_equipment::creature_equipment_plan("mage", 0).unwrap();
    assert_eq!(
        old_plan
            .iter()
            .map(|entry| (entry.definition_id.as_str(), entry.quantity))
            .collect::<Vec<_>>(),
        [("spell-material:mage-armor", 1), ("wand", 1)]
    );
    for source in [original, candidate] {
        let pin = creature_source_pin(source).unwrap();
        assert_eq!(
            dmd_rules::tactical_creature_equipment::creature_equipment_plan_from_source(&pin, 0)
                .unwrap(),
            old_plan
        );
        assert!(
            dmd_rules::tactical_creature_equipment::creature_equipment_plan_from_source(&pin, 1)
                .is_err()
        );
        assert!(source.statistics.gear_quantities.is_empty());
        assert!(
            !serde_json::to_string(source)
                .unwrap()
                .contains("gear_quantities")
        );
        let mut with_empty = serde_json::to_value(source).unwrap();
        with_empty["statistics"]["gear_quantities"] = serde_json::json!({});
        let roundtrip: CreatureDefinition = serde_json::from_value(with_empty).unwrap();
        assert_eq!(roundtrip, *source);
        assert_eq!(creature_source_pin(&roundtrip).unwrap(), pin);
        assert_eq!(source.coverage, original.coverage);
        assert!(matches!(
            source.coverage,
            DefinitionCoverage::SelectedFeatures { .. }
        ));
        assert_eq!(
            source
                .statistics
                .prepared_defense
                .as_ref()
                .unwrap()
                .spell_id,
            "mage-armor"
        );
        assert!(source.traits.is_empty());
        assert_eq!(
            source
                .features
                .iter()
                .map(|feature| feature.id.as_str())
                .collect::<Vec<_>>(),
            ["spellcasting", "protective-magic"]
        );
    }
}

fn original_state() -> CampaignState {
    let export: serde_json::Value = serde_json::from_str(include_str!(
        "../../dmd-app/tests/fixtures/shield-hit-v1-selected.json"
    ))
    .unwrap();
    CampaignState::decode_json(export["current_state"]["state_json"].as_str().unwrap()).unwrap()
}

#[test]
fn mage_builder_preserves_both_sizes_and_requires_the_exact_annotated_source() {
    let pack =
        RulesPack::from_json(include_str!("../../../content/srd-5.2.1/kernel.json")).unwrap();
    let original = original_state();
    let original_profile = original
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
    assert_eq!(
        dmd_rules::tactical_grapple_sources::ordinary_grapple_anatomy(
            &original,
            original_profile.actor,
            &pack
        )
        .unwrap(),
        None
    );
    for size in [CreatureSize::Small, CreatureSize::Medium] {
        let mut state = original.clone();
        let actor = EntityId::new();
        state.entities.insert(
            actor,
            WorldEntity {
                id: actor,
                campaign_id: state.campaign_id(),
                display_name: "No anatomical authority in this name".into(),
                kind: EntityKind::Creature,
                existence: EntityExistence::Present,
                location_id: None,
            },
        );
        let meta = CommandMeta {
            id: CommandId::new(),
            campaign_id: state.campaign_id(),
            session_id: None,
            issuer: CommandIssuer::Admin,
            actor: None,
            expected_event_sequence: state.applied_event_sequence,
        };
        let pin = creature_source_pin(bundled_mage_v2().unwrap()).unwrap();
        let choice = CreatureBuildChoice {
            definition_id: "mage".into(),
            size,
            additional_languages: vec!["dwarvish".into(), "elvish".into(), "draconic".into()],
            hit_points: CreatureHitPointChoice::Average,
            controller: CreatureController::Host,
            in_lair: false,
        };
        let built = build_creature_from_source(&state, &meta, actor, &choice, Some(&pin)).unwrap();
        assert_eq!(built.profile.size, size);
        assert_eq!(built.profile.source, pin);
        assert_eq!(built.mechanics.hp, 81);
        // Isolated pure builder installation, not fabricated accepted journal history.
        let rules = state.rules.as_mut().unwrap();
        rules.entities.insert(actor, built.mechanics);
        let creatures = rules.tactical_creatures.as_mut().unwrap();
        creatures.profiles.push(built.profile);
        creatures.runtime.push(built.runtime);
        assert_eq!(
            dmd_rules::tactical_grapple_sources::ordinary_grapple_anatomy(&state, actor, &pack)
                .unwrap(),
            Some(GrappleAnatomyProof::Creature {
                source: pin,
                ordinary_hands: OrdinaryHandAnatomy::TwoHandsV1
            })
        );
        assert!(bundled_air_elemental().unwrap().ordinary_hands.is_none());
    }
}

#[test]
fn mage_v2_manifest_declares_the_exact_compiled_payload() {
    let manifest: serde_json::Value =
        serde_json::from_str(include_str!("../../../content/srd-5.2.1/manifest.json")).unwrap();
    let files = manifest["files"].as_array().unwrap();
    let rows = files
        .iter()
        .filter(|entry| entry["path"] == "mage-v2.json")
        .collect::<Vec<_>>();
    assert_eq!(rows.len(), 1);
    let hash = MAGE_V2_SOURCE_JSON
        .as_bytes()
        .iter()
        .fold(0xcbf29ce484222325u64, |hash, byte| {
            (hash ^ u64::from(*byte)).wrapping_mul(0x100000001b3)
        });
    assert_eq!(rows[0]["byte_len"], MAGE_V2_SOURCE_JSON.len());
    assert_eq!(rows[0]["checksum"]["algorithm"], "fnv1a64");
    assert_eq!(rows[0]["checksum"]["value"], format!("{hash:016x}"));
    assert_eq!(
        serde_json::from_str::<CreatureDefinition>(MAGE_V2_SOURCE_JSON).unwrap(),
        *bundled_mage_v2().unwrap()
    );
}
