//! Immutable source/policy controls only. No Ogre is created, admitted or played.
use dmd_domain::*;
use dmd_rules::{tactical_creature_equipment::*, tactical_creatures::*, tactical_definitions::*};

#[test]
fn immutable_ogre_retains_every_printed_stat_and_the_reviewed_annotation() {
    let s = bundled_ogre().unwrap();
    assert_eq!(s.id, "ogre");
    assert_eq!(s.name, "Ogre");
    assert_eq!(s.coverage, DefinitionCoverage::CompleteStatBlock);
    assert_eq!(s.source_pages, [312]);
    assert_eq!(s.ordinary_hands, Some(OrdinaryHandAnatomy::TwoHandsV1));
    assert!(s.traits.is_empty() && s.legendary_budget.is_none());
    let a = &s.statistics;
    assert_eq!(a.size, SourceSize::Large);
    assert_eq!(a.allowed_sizes, [SourceSize::Large]);
    assert_eq!(a.creature_type, CreatureType::Giant);
    assert!(a.creature_tags.is_empty());
    assert_eq!(a.alignment, "Chaotic Evil");
    assert_eq!((a.armor_class, a.hit_points), (11, 68));
    assert_eq!(
        a.hit_point_formula.dice,
        [DieSpec {
            count: 8,
            sides: 10
        }]
    );
    assert_eq!(a.hit_point_formula.fixed, 24);
    assert_eq!(a.ability_scores, [19, 8, 16, 5, 7, 7]);
    assert_eq!(a.saving_throw_modifiers, [4, -1, 3, -3, -2, -2]);
    assert_eq!(a.initiative_modifier, -1);
    assert_eq!(a.proficiency_bonus, 2);
    assert_eq!(
        (a.challenge_rating.as_str(), a.experience_points),
        ("2", 450)
    );
    assert_eq!(
        a.speeds,
        Speeds {
            walk: 40,
            burrow: 0,
            climb: 0,
            fly: 0,
            swim: 0,
            hover: false
        }
    );
    assert_eq!(
        a.senses,
        SenseRanges {
            blindsight: 0,
            darkvision: 60,
            tremorsense: 0,
            truesight: 0,
            passive_perception: 8
        }
    );
    assert!(
        a.skills.is_empty()
            && a.damage_resistances.is_empty()
            && a.damage_vulnerabilities.is_empty()
    );
    assert!(
        a.damage_immunities.is_empty() && a.condition_immunities.is_empty() && !a.exhaustion_immune
    );
    assert_eq!(a.languages, ["Common", "Giant"]);
    assert!(a.can_speak && a.additional_languages == 0 && a.prepared_defense.is_none());
    assert_eq!(a.gear, ["greatclub", "javelin"]);
    assert_eq!(
        a.gear_quantities,
        std::collections::BTreeMap::from([("javelin".into(), 3)])
    );
    assert_eq!(
        s.features.iter().map(|f| f.id.as_str()).collect::<Vec<_>>(),
        ["greatclub", "javelin-melee", "javelin-thrown"]
    );
    assert!(s.features.iter().all(|f| f.source_page == 312
        && f.activation == FeatureActivation::Action
        && f.usage.is_none()
        && f.spell_component_waivers.is_none()));
}

#[test]
fn full_pin_lookup_does_not_enable_current_or_id_only_admission() {
    let s = bundled_ogre().unwrap();
    let pin = creature_source_pin(s).unwrap();
    assert!(std::ptr::eq(creature_source(&pin).unwrap(), s));
    assert_eq!(immutable_creature_sources().unwrap().len(), 13);
    assert!(creature_definition("ogre").is_err());
    let current = current_creature_sources().unwrap();
    assert_eq!(current.len(), 11);
    assert!(current.iter().all(|s| s.id != "ogre"));
    for field in 0..4 {
        let mut wrong = pin.clone();
        match field {
            0 => wrong.ruleset_id.push('x'),
            1 => wrong.ruleset_version.push('x'),
            2 => wrong.definition_id = "goblin-warrior".into(),
            _ => wrong.definition_fingerprint.push('0'),
        }
        assert!(creature_source(&wrong).is_err());
        assert!(ogre_weapon_program(&wrong, "greatclub").is_err());
    }
    let mut altered = s.clone();
    altered
        .statistics
        .gear_quantities
        .insert("javelin".into(), 4);
    assert!(creature_source(&creature_source_pin(&altered).unwrap()).is_err());
}

#[test]
fn an_invented_retained_ogre_profile_cannot_bypass_closed_creation() {
    // A hostile typed record, never an accepted profile or historical producer.
    let profile = CreatureProfile {
        actor: EntityId::new(),
        origin: CommandMeta {
            id: CommandId::new(),
            campaign_id: CampaignId::new(),
            session_id: None,
            issuer: CommandIssuer::Admin,
            actor: None,
            expected_event_sequence: 0,
        },
        source: creature_source_pin(bundled_ogre().unwrap()).unwrap(),
        size: CreatureSize::Large,
        additional_languages: vec![],
        hit_points: CreatureHitPointOrigin::Average,
    };
    assert!(
        initial_creature_mechanics(&profile)
            .unwrap_err()
            .to_string()
            .contains("Ogre creation is not admitted")
    );
}

#[test]
fn closed_program_preserves_printed_damage_without_changing_ordinary_weapons() {
    let pin = creature_source_pin(bundled_ogre().unwrap()).unwrap();
    for (id, weapon_id, sides, damage_type, delivery, hands) in [
        (
            "greatclub",
            "greatclub",
            8,
            DamageType::Bludgeoning,
            WeaponDelivery::Melee,
            WeaponHands::Two,
        ),
        (
            "javelin-melee",
            "javelin",
            6,
            DamageType::Piercing,
            WeaponDelivery::Melee,
            WeaponHands::One,
        ),
        (
            "javelin-thrown",
            "javelin",
            6,
            DamageType::Piercing,
            WeaponDelivery::Thrown,
            WeaponHands::One,
        ),
    ] {
        let p = ogre_weapon_program(&pin, id).unwrap();
        assert_eq!(p.source(), &pin);
        assert_eq!((p.feature_id(), p.weapon().id.as_str()), (id, weapon_id));
        assert_eq!(
            (p.delivery(), p.ability(), p.attack_bonus()),
            (delivery, Ability::Strength, 6)
        );
        assert_eq!(
            p.printed_base_damage().amount.dice,
            [DieSpec { count: 2, sides }]
        );
        assert_eq!(p.printed_base_damage().amount.fixed, 4);
        assert_eq!(p.printed_base_damage().damage_type, damage_type);
        assert_eq!(p.weapon().damage.dice, [DieSpec { count: 1, sides }]);
        assert_eq!(p.weapon().damage.fixed, 0);
        assert_eq!(p.weapon().hands, hands);
        assert!(p.weapon().ammunition.is_none());
        match p.source_delivery() {
            AttackDelivery::Melee { reach_feet } => assert_eq!(*reach_feet, 5),
            AttackDelivery::Ranged { range } => {
                assert_eq!((range.normal_feet, range.long_feet), (30, 120))
            }
        }
    }
    for id in ["javelin", "multiattack", "shortbow", "Greatclub", ""] {
        assert!(ogre_weapon_program(&pin, id).is_err());
    }
    let goblin = creature_source_pin(bundled_goblin_warrior_v2().unwrap()).unwrap();
    assert!(ogre_weapon_program(&goblin, "greatclub").is_err());
}

#[test]
fn registered_ogre_allocation_is_four_individuals_without_ammunition_or_admission() {
    let pin = creature_source_pin(bundled_ogre().unwrap()).unwrap();
    let plan = creature_equipment_plan_from_source(&pin, 0).unwrap();
    assert_eq!(
        plan.iter()
            .map(|a| (a.definition_id.as_str(), a.quantity))
            .collect::<Vec<_>>(),
        [
            ("greatclub", 1),
            ("javelin", 1),
            ("javelin", 1),
            ("javelin", 1)
        ]
    );
    assert!(creature_equipment_plan_from_source(&pin, 1).is_err());
    assert!(creature_equipment_plan("ogre", 0).is_err());
    // Allocation positions are not materialized ItemIds or an accepted grant.
    assert!(
        current_creature_sources()
            .unwrap()
            .iter()
            .all(|s| s.id != "ogre")
    );
}

#[test]
fn ogre_manifest_pins_exact_bytes_and_old_tactical_catalog_stays_separate() {
    let manifest: serde_json::Value =
        serde_json::from_str(include_str!("../../../content/srd-5.2.1/manifest.json")).unwrap();
    let row = manifest["files"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["path"] == "ogre-v1.json")
        .unwrap();
    let hash = OGRE_SOURCE_JSON
        .bytes()
        .fold(0xcbf29ce484222325_u64, |h, b| {
            (h ^ u64::from(b)).wrapping_mul(0x100000001b3)
        });
    assert_eq!(row["byte_len"], OGRE_SOURCE_JSON.len());
    assert_eq!(row["checksum"]["algorithm"], "fnv1a64");
    assert_eq!(row["checksum"]["value"], format!("{hash:016x}"));
    assert!(
        bundled_tactical_definitions()
            .unwrap()
            .creature("ogre")
            .is_none()
    );
    let parsed: CreatureDefinition = serde_json::from_str(OGRE_SOURCE_JSON).unwrap();
    assert_eq!(&parsed, bundled_ogre().unwrap());
}
