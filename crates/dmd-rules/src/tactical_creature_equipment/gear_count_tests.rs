//! Source-planning controls only: no proposed count-bearing source is admitted.
use super::*;
use serde_json::{Value, json};

fn proposed(gear: Value, quantities: Value) -> TacticalDefinitions {
    let mut value: Value = serde_json::from_str(TACTICAL_DEFINITIONS_JSON).unwrap();
    value["creatures"][0]["id"] = json!("unadmitted-gear-proposal");
    value["creatures"][0]["statistics"]["gear"] = gear;
    value["creatures"][0]["statistics"]["gear_quantities"] = quantities;
    TacticalDefinitions::from_json(&value.to_string()).unwrap()
}

fn pairs(plan: &[CreatureEquipmentAllocation]) -> Vec<(&str, u32)> {
    plan.iter()
        .map(|a| (a.definition_id.as_str(), a.quantity))
        .collect()
}

#[test]
fn fixed_individual_counts_expand_but_stacks_keep_their_quantity() {
    // These are actual Greatclub/Javelin definitions and the proposed equipment
    // counts, not an invented Ogre stat block, admitted pin or gameplay state.
    let definitions = proposed(json!(["javelin", "greatclub"]), json!({"javelin": 3}));
    let source = &definitions.creatures[0];
    let plan = equipment_plan(source, &definitions, 0).unwrap();
    assert_eq!(
        pairs(&plan),
        vec![
            ("greatclub", 1),
            ("javelin", 1),
            ("javelin", 1),
            ("javelin", 1)
        ]
    );
    assert!(equipment_plan(source, &definitions, 1).is_err());
    let pin = creature_source_pin(source).unwrap();
    assert!(
        creature_equipment_plan_from_source(&pin, 0).is_err(),
        "planning never registers a proposed source"
    );

    let definitions = proposed(json!(["arrows"]), json!({"arrows": 3}));
    assert_eq!(
        pairs(&equipment_plan(&definitions.creatures[0], &definitions, 0).unwrap()),
        vec![("arrows", 3)]
    );
}

#[test]
fn post_load_classification_rejects_unknown_or_ambiguous_equipped_gear() {
    for gear in ["missing-definition", "leather-armor", "shield"] {
        let definitions = proposed(json!([gear]), json!({(gear): 2}));
        assert!(
            equipment_plan(&definitions.creatures[0], &definitions, 0).is_err(),
            "{gear}"
        );
    }
}

#[test]
fn an_unregistered_quantity_cannot_reach_expansion_through_a_known_source_id() {
    let mut source = creature_definition("goblin-warrior").unwrap().clone();
    source
        .statistics
        .gear_quantities
        .insert("scimitar".into(), u32::MAX);
    let pin = creature_source_pin(&source).unwrap();
    assert!(
        creature_equipment_plan_from_source(&pin, 20)
            .unwrap_err()
            .to_string()
            .contains("fingerprint differs")
    );
}

#[test]
fn fixed_and_generated_namespaces_cannot_overwrite_quantities() {
    for count in [1, 3] {
        let overrides = if count == 1 {
            json!({})
        } else {
            json!({"arrows": count})
        };
        let definitions = proposed(json!(["shortbow", "arrows"]), overrides);
        assert!(equipment_plan(&definitions.creatures[0], &definitions, 20).is_err());
    }
    let definitions = creature_definitions().unwrap();
    let mut source = definitions.creature("cultist-fanatic").unwrap().clone();
    source
        .statistics
        .gear
        .push("spell-material:hold-person".into());
    source
        .statistics
        .gear_quantities
        .insert("spell-material:hold-person".into(), 3);
    assert!(equipment_plan(&source, definitions, 0).is_err());
}

#[test]
fn repeated_generated_requirements_keep_one_original_stack() {
    let definitions = proposed(json!(["shortbow", "longbow"]), json!({}));
    assert_eq!(
        pairs(&equipment_plan(&definitions.creatures[0], &definitions, 20).unwrap()),
        vec![("arrows", 20), ("longbow", 1), ("shortbow", 1)]
    );
    let definitions = creature_definitions().unwrap();
    let mut source = definitions.creature("cultist-fanatic").unwrap().clone();
    let mut repeated = source
        .features
        .iter()
        .find(|f| matches!(f.feature, MonsterFeature::Spellcasting { .. }))
        .unwrap()
        .clone();
    repeated.id = "additional-material-grant".into();
    source.features.push(repeated);
    let plan = equipment_plan(&source, definitions, 0).unwrap();
    assert_eq!(
        pairs(&plan),
        vec![
            ("holy-symbol", 1),
            ("leather-armor", 1),
            ("spell-material:hold-person", 1)
        ]
    );
}

#[test]
fn every_immutable_source_keeps_its_original_ordered_allocations() {
    // Ogre's new allocation is tested separately; retain every original vector.
    let sources: Vec<_> = immutable_creature_sources()
        .unwrap()
        .into_iter()
        .filter(|source| source.id != "ogre")
        .collect();
    assert_eq!(sources.len(), 12);
    for source in sources {
        assert!(source.statistics.gear_quantities.is_empty());
        let (ammunition, expected) = match source.id.as_str() {
            "goblin-warrior" => (
                20,
                vec![
                    ("arrows", 20),
                    ("leather-armor", 1),
                    ("scimitar", 1),
                    ("shield", 1),
                    ("shortbow", 1),
                ],
            ),
            "skeleton" => (20, vec![("arrows", 20), ("shortbow", 1), ("shortsword", 1)]),
            "cultist-fanatic" => (
                0,
                vec![
                    ("holy-symbol", 1),
                    ("leather-armor", 1),
                    ("spell-material:hold-person", 1),
                ],
            ),
            "mage" => (0, vec![("spell-material:mage-armor", 1), ("wand", 1)]),
            "wolf" | "warhorse" | "young-red-dragon" | "adult-red-dragon" | "chimera"
            | "night-hag" | "air-elemental" => (0, vec![]),
            other => panic!("unreviewed source {other}"),
        };
        let pin = creature_source_pin(source).unwrap();
        assert_eq!(
            pairs(&creature_equipment_plan_from_source(&pin, ammunition).unwrap()),
            expected,
            "{pin:?}"
        );
    }
}
