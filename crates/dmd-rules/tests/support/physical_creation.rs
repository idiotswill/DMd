use super::*;

fn physical_input() -> CharacterCreationInput {
    let mut selected = input();
    selected
        .purchases
        .extend(["greatsword", "glaive"].map(|id| EquipmentChoice {
            item_id: id.into(),
            quantity: 1,
        }));
    selected.masteries = ["greatsword".into(), "glaive".into(), "dagger".into()];
    selected
}

#[test]
fn current_catalog_appends_verified_prices_without_rewriting_the_original_sixteen() {
    let old = starter_catalog();
    let current = physical_starter_catalog();
    assert_eq!(old.items.len(), 16);
    assert_eq!(current.items.len(), 18);
    assert_eq!(&current.items[..16], old.items.as_slice());
    assert_eq!(current.starting_money_cp, old.starting_money_cp);
    for (id, cp) in [("glaive", 2000), ("greatsword", 5000)] {
        let row = current.items.iter().find(|row| row.id == id).unwrap();
        assert_eq!(
            (
                row.unit_cost_cp,
                row.purchase_multiple,
                row.source_page,
                row.weapon
            ),
            (cp, 1, 91, true)
        );
    }
    let source = current_character_creation_source().unwrap();
    assert_eq!(source.ruleset_id, "srd-5.2");
    assert_eq!(source.ruleset_version, "5.2.1");
    assert_eq!(source.catalog_schema_version, 1);
    assert_eq!(
        source.profile_id,
        "human-fighter-soldier-level-1-physical-v1"
    );
    assert_eq!(source.definition_fingerprint, "e2d57011783b3fae");
    source.validate_shape().unwrap();
    assert_eq!(
        serde_json::from_slice::<StarterCatalog>(include_bytes!(
            "../../../../content/srd-5.2.1/character-creation-physical-v1.json"
        ))
        .unwrap(),
        current
    );
}

#[test]
fn current_creation_derives_prices_and_masteries_with_only_original_kernel_registrations() {
    let actor = EntityId::new();
    let source = current_character_creation_source().unwrap();
    let selected = physical_input();
    let built = build_character_from_source(&selected, actor, &source, &pack()).unwrap();
    assert_eq!(built.profile.money_cp, 9580);
    assert_eq!(built.profile.creation_source, Some(source));
    assert_eq!(built.profile.masteries, selected.masteries);
    let legacy = build_character(&input(), actor, &pack()).unwrap();
    assert_eq!(
        built.mechanics, legacy.mechanics,
        "physical purchases do not create kernel attacks"
    );
    validate_character_profile(&built.profile, &pack()).unwrap();
    validate_character_intrinsics(&built.profile, &built.mechanics, &pack()).unwrap();
    assert!(build_character(&selected, actor, &pack()).is_err());
    let mut only_physical = selected;
    only_physical
        .purchases
        .retain(|purchase| !["club", "dagger", "shortbow"].contains(&purchase.item_id.as_str()));
    let physical = build_character_from_source(
        &only_physical,
        actor,
        built.profile.creation_source.as_ref().unwrap(),
        &pack(),
    )
    .unwrap();
    assert!(physical.mechanics.attacks.is_empty());
    assert!(physical.mechanics.attack_proficiencies.is_empty());
}

#[test]
fn absent_or_null_profile_source_remains_legacy_and_does_not_authorize_new_purchases() {
    let actor = EntityId::new();
    let old = build_character(&input(), actor, &pack()).unwrap();
    let old_json = serde_json::to_string(&old.profile).unwrap();
    assert!(!old_json.contains("creation_source"));
    let mut value = serde_json::to_value(&old.profile).unwrap();
    value["creation_source"] = serde_json::Value::Null;
    let explicit_null: CharacterProfile = serde_json::from_value(value).unwrap();
    assert_eq!(explicit_null, old.profile);
    assert_eq!(serde_json::to_string(&explicit_null).unwrap(), old_json);
    let mut physical = build_character_from_source(
        &physical_input(),
        actor,
        &current_character_creation_source().unwrap(),
        &pack(),
    )
    .unwrap()
    .profile;
    physical.creation_source = None;
    assert!(validate_character_profile(&physical, &pack()).is_err());
}

#[test]
fn every_pin_field_and_derived_price_is_checked_and_unknown_input_is_refused() {
    let original = current_character_creation_source().unwrap();
    for field in [
        "ruleset_id",
        "ruleset_version",
        "catalog_schema_version",
        "profile_id",
        "definition_fingerprint",
    ] {
        let mut value = serde_json::to_value(&original).unwrap();
        value[field] = if field == "catalog_schema_version" {
            serde_json::json!(2)
        } else {
            serde_json::json!("foreign")
        };
        let wrong: CharacterCreationSourcePin = serde_json::from_value(value).unwrap();
        assert!(
            build_character_from_source(&physical_input(), EntityId::new(), &wrong, &pack())
                .is_err(),
            "{field}"
        );
    }
    let mut unknown = serde_json::to_value(&original).unwrap();
    unknown["paid"] = serde_json::json!(true);
    assert!(serde_json::from_value::<CharacterCreationSourcePin>(unknown).is_err());
    for field in ["source", "creation_source", "equipment", "money_cp"] {
        let mut forged = serde_json::to_value(physical_input()).unwrap();
        forged[field] = serde_json::json!({});
        assert!(serde_json::from_value::<CharacterCreationInput>(forged).is_err());
    }
    let original =
        build_character_from_source(&physical_input(), EntityId::new(), &original, &pack())
            .unwrap()
            .profile;
    for mutation in 0..4 {
        let mut forged = original.clone();
        match mutation {
            0 => forged.money_cp += 1,
            1 => {
                forged
                    .equipment
                    .iter_mut()
                    .find(|item| item.item_id == "glaive")
                    .unwrap()
                    .unit_cost_cp = 0
            }
            2 => {
                forged
                    .equipment
                    .iter_mut()
                    .find(|item| item.item_id == "greatsword")
                    .unwrap()
                    .source_page = 1
            }
            _ => {
                forged
                    .creation_source
                    .as_mut()
                    .unwrap()
                    .definition_fingerprint = "0000000000000000".into()
            }
        }
        assert!(validate_character_profile(&forged, &pack()).is_err());
    }
}

#[test]
fn explicit_current_kernel_input_requires_its_exact_source_without_extending_legacy_input() {
    let action = RulesAction::CreateCharacterFromSource {
        entity_id: EntityId::new(),
        source: current_character_creation_source().unwrap(),
        input: physical_input(),
    };
    let encoded = serde_json::to_value(&action).unwrap();
    assert_eq!(
        serde_json::from_value::<RulesAction>(encoded.clone()).unwrap(),
        action
    );
    for mode in 0..3 {
        let mut malformed = encoded.clone();
        let body = malformed["CreateCharacterFromSource"]
            .as_object_mut()
            .unwrap();
        match mode {
            0 => {
                body.remove("source");
            }
            1 => {
                body.insert("source".into(), serde_json::Value::Null);
            }
            _ => {
                body.get_mut("source").unwrap()["unknown"] = serde_json::json!(true);
            }
        }
        assert!(serde_json::from_value::<RulesAction>(malformed).is_err());
    }
    let old = RulesAction::CreateCharacter {
        entity_id: EntityId::new(),
        input: input(),
    };
    let bytes = serde_json::to_string(&old).unwrap();
    assert!(!bytes.contains("source"));
    assert_eq!(
        serde_json::to_string(&serde_json::from_str::<RulesAction>(&bytes).unwrap()).unwrap(),
        bytes
    );
}
