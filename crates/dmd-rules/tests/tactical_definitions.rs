// This slice does not edit the shared lib.rs; the root integration exports this module.
#[path = "../src/tactical_definitions.rs"]
mod tactical_definitions;

use dmd_domain::{Ability, Condition, DamageType, DieSpec};
use serde_json::{Value, json};
use tactical_definitions::*;

fn pack() -> TacticalDefinitions {
    TacticalDefinitions::from_json(TACTICAL_DEFINITIONS_JSON).expect("reviewed source definitions")
}

fn rejects(edit: impl FnOnce(&mut Value)) {
    let mut value: Value = serde_json::from_str(TACTICAL_DEFINITIONS_JSON).unwrap();
    edit(&mut value);
    assert!(TacticalDefinitions::from_json(&value.to_string()).is_err());
}

#[test]
fn every_srd_weapon_and_property_is_present_with_exceptional_delivery_preserved() {
    let p = pack();
    let mut actual: Vec<_> = p.weapons.iter().map(|w| w.id.as_str()).collect();
    actual.sort_unstable();
    let mut expected: Vec<_> = "club dagger greatclub handaxe javelin light-hammer mace quarterstaff sickle spear dart light-crossbow shortbow sling battleaxe flail glaive greataxe greatsword halberd lance longsword maul morningstar pike rapier scimitar shortsword trident warhammer war-pick whip blowgun hand-crossbow heavy-crossbow longbow musket pistol".split_whitespace().collect();
    expected.sort_unstable();
    assert_eq!(actual, expected);
    assert_eq!(p.weapon_properties.len(), 10);
    assert_eq!(p.masteries.len(), 8);
    assert!(p.weapons.iter().all(|w| w.source_page == 91));
    let blowgun = p.weapon("blowgun").unwrap();
    assert!(blowgun.damage.dice.is_empty());
    assert_eq!(blowgun.damage.fixed, 1);
    assert_eq!(blowgun.ammunition, Some(AmmunitionKind::Needle));
    assert_eq!(
        p.weapon("lance").unwrap().hands,
        WeaponHands::TwoUnlessMounted
    );
    assert_eq!(p.weapon("whip").unwrap().reach_bonus_feet, 5);
    let dagger = p.weapon("dagger").unwrap();
    assert_eq!(dagger.kind, WeaponKind::Melee);
    assert_eq!(
        dagger.range.unwrap(),
        WeaponRange {
            normal_feet: 20,
            long_feet: 60
        }
    );
    assert_eq!(dagger.mastery, WeaponMastery::Nick);
    assert_eq!(p.weapon("longbow").unwrap().range.unwrap().long_feet, 600);
    assert_eq!(
        p.weapon("trident")
            .unwrap()
            .versatile_damage
            .as_ref()
            .unwrap()
            .dice,
        vec![DieSpec {
            count: 1,
            sides: 10
        }]
    );
    assert!(p.weapon("invented-weapon").is_none());
}

#[test]
fn source_spells_preserve_reaction_ongoing_save_spatial_and_existing_kernel_clauses() {
    let p = pack();
    let shield = p.spell("shield").unwrap();
    assert_eq!(shield.duration, EffectDuration::UntilStartOfCastersNextTurn);
    assert_eq!(
        shield.casting_time,
        CastingTime::Reaction {
            trigger: ReactionTrigger::HitByAttackOrTargetedByMagicMissile
        }
    );
    assert!(shield.effects.contains(&EffectDescriptor::ArmorClassBonus {
        bonus: 5,
        includes_triggering_attack: true
    }));
    let hold = p.spell("hold-person").unwrap();
    assert_eq!(
        hold.targets,
        TargetSelection::Creatures {
            maximum: 1,
            creature_type: Some(CreatureType::Humanoid),
            requires_sight: true
        }
    );
    assert_eq!(
        hold.effects,
        vec![EffectDescriptor::SaveCondition {
            ability: Ability::Wisdom,
            condition: Condition::Paralyzed,
            repeat: Some(RepeatSave::EndOfTargetsTurnEndsOnSuccess)
        }]
    );
    assert_eq!(hold.upcast, Some(Upcast::AdditionalTargets { count: 1 }));
    let thunder = p.spell("thunderwave").unwrap();
    assert_eq!(
        thunder.targets,
        TargetSelection::Area {
            shape: AreaShape::Cube { side_feet: 15 }
        }
    );
    assert!(
        thunder
            .effects
            .contains(&EffectDescriptor::PushUnsecuredObjectsEntirelyInArea { feet: 10 })
    );
    assert!(
        thunder
            .effects
            .contains(&EffectDescriptor::Audible { feet: 300 })
    );
    assert_eq!(p.spell("cure-wounds").unwrap().range, SpellRange::Touch);
    assert!(
        matches!(&p.spell("fire-bolt").unwrap().effects[0], EffectDescriptor::RangedSpellAttack { extra_die_at_levels, ignite_unworn_uncarried_target: true, .. } if extra_die_at_levels == &[5, 11, 17])
    );
    let lights = p.spell("dancing-lights").unwrap();
    assert_eq!(
        lights.targets,
        TargetSelection::LightPoints {
            maximum: 4,
            may_combine_as_medium_form: true
        }
    );
    assert_eq!(
        lights.effects,
        vec![EffectDescriptor::MovableDimLights {
            radius_feet: 10,
            bonus_action_move_feet: 60,
            maximum_separation_feet: 20,
            vanish_outside_casting_range: true
        }]
    );
}

#[test]
fn monster_source_bonuses_recharge_and_partial_support_are_explicit() {
    let p = pack();
    let goblin = p.creature("goblin-warrior").unwrap();
    assert_eq!(goblin.statistics.creature_type, CreatureType::Fey);
    assert_eq!(goblin.statistics.skills[0].modifier, 6);
    assert!(
        matches!(&goblin.features[0].feature, MonsterFeature::Attack { extra_damage_if_attack_had_advantage, .. } if extra_damage_if_attack_had_advantage[0].amount.dice == vec![DieSpec { count: 1, sides: 4 }])
    );
    let skeleton = p.creature("skeleton").unwrap();
    assert_eq!(
        skeleton.statistics.damage_vulnerabilities,
        vec![DamageType::Bludgeoning]
    );
    assert!(skeleton.statistics.exhaustion_immune);
    assert!(!skeleton.statistics.can_speak);
    let horse = p.creature("warhorse").unwrap();
    // The source Wisdom save is +3, not its +1 ability modifier.
    assert_eq!(horse.statistics.saving_throw_modifiers[4], 3);
    let young = p.creature("young-red-dragon").unwrap();
    assert_eq!(young.statistics.initiative_modifier, 4);
    assert_eq!(young.statistics.ability_scores[1], 10);
    assert!(matches!(
        &young.features[2].feature,
        MonsterFeature::SaveArea {
            dc: 17,
            recharge: Recharge {
                die_sides: 6,
                minimum: 5,
                maximum: 6
            },
            ..
        }
    ));
    let adult = p.creature("adult-red-dragon").unwrap();
    assert!(
        matches!(&adult.coverage, DefinitionCoverage::SelectedFeatures { omitted } if omitted.contains(&"spellcasting".into()))
    );
    assert_eq!(
        adult.legendary_budget,
        Some(LegendaryBudget {
            uses: 3,
            uses_in_lair: 4
        })
    );
    let cultist = p.creature("cultist-fanatic").unwrap();
    assert!(
        matches!(&cultist.coverage, DefinitionCoverage::SelectedFeatures { omitted } if omitted.contains(&"spiritual-weapon".into()))
    );
    assert!(
        matches!(&cultist.features[1].feature, MonsterFeature::Spellcasting { ability: Ability::Wisdom, save_dc: Some(12), spells, .. } if spells == &vec![InnateSpell { spell_id: "hold-person".into(), cast_level: 2, uses_per_long_rest: Some(1) }])
    );
}

#[test]
fn unknown_fields_source_pins_duplicates_and_invalid_pages_fail_closed() {
    rejects(|v| v["schema_version"] = json!(2));
    rejects(|v| v["ruleset_version"] = json!("5.1"));
    rejects(|v| v["weapons"][0]["automatic_hit"] = json!(true));
    rejects(|v| v["weapons"][1]["id"] = json!("club"));
    rejects(|v| v["weapons"][0]["source_page"] = json!(365));
    rejects(|v| v["spells"][0]["source_pages"] = json!([114, 114]));
    rejects(|v| {
        v["masteries"].as_array_mut().unwrap().pop();
    });
    rejects(|v| v["weapon_properties"][1]["property"] = json!("Ammunition"));
}

#[test]
fn contradictory_weapon_requirements_and_impossible_dice_are_rejected() {
    rejects(|v| v["weapons"][1]["range"]["long_feet"] = json!(10));
    rejects(|v| v["weapons"][1]["ammunition"] = json!("Arrow"));
    rejects(|v| v["weapons"][0]["hands"] = json!("TwoUnlessMounted"));
    rejects(|v| v["weapons"][0]["reach_bonus_feet"] = json!(5));
    rejects(|v| v["weapons"][0]["damage"]["dice"][0]["count"] = json!(0));
    rejects(|v| v["weapons"][0]["damage"]["dice"][0]["sides"] = json!(7));
    rejects(|v| v["weapons"][11]["ammunition"] = Value::Null);
    rejects(|v| v["weapons"][7]["versatile_damage"] = Value::Null);
}

#[test]
fn invalid_spell_targets_dependencies_and_continuations_are_rejected() {
    rejects(|v| {
        v["spells"][3]["effects"][1]["PreventSpellDamage"]["spell_id"] = json!("missing-spell")
    });
    rejects(|v| v["spells"][2]["duration"] = json!("Instantaneous"));
    rejects(|v| v["spells"][1]["targets"] = json!("Caster"));
    rejects(|v| v["spells"][1]["upcast"] = json!({"AdditionalDarts":{"count":1}}));
    rejects(|v| v["spells"][3]["casting_time"] = json!("Action"));
    rejects(|v| {
        v["spells"][8]["effects"][0]["RangedSpellAttack"]["extra_die_at_levels"] =
            json!([4, 10, 16])
    });
    rejects(|v| {
        v["spells"][9]["targets"] =
            json!({"Creatures":{"maximum":4,"creature_type":null,"requires_sight":false}})
    });
}

#[test]
fn monster_references_recharge_budgets_and_partial_labels_are_validated() {
    rejects(|v| {
        v["creatures"][4]["features"][0]["feature"]["Multiattack"]["attack_options"] =
            json!(["fire-breath"])
    });
    rejects(|v| {
        v["creatures"][4]["features"][2]["feature"]["SaveArea"]["recharge"]["minimum"] = json!(7)
    });
    rejects(|v| v["creatures"][5]["legendary_budget"] = Value::Null);
    rejects(|v| v["creatures"][5]["coverage"] = json!({"SelectedFeatures":{"omitted":[]}}));
    rejects(|v| {
        v["creatures"][6]["features"][1]["feature"]["Spellcasting"]["spells"][0]["cast_level"] =
            json!(1)
    });
    rejects(|v| {
        v["creatures"][6]["features"][1]["feature"]["Spellcasting"]["spells"][0]["spell_id"] =
            json!("invented-spell")
    });
    rejects(|v| v["creatures"][0]["statistics"]["speeds"]["hover"] = json!(true));
    rejects(|v| v["creatures"][0]["statistics"]["allowed_sizes"] = json!(["Huge"]));
}

#[test]
fn manifest_declares_exact_tactical_bytes_and_definitions_round_trip() {
    let manifest: Value =
        serde_json::from_str(include_str!("../../../content/srd-5.2.1/manifest.json")).unwrap();
    let row = manifest["files"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["path"] == "tactical.json")
        .unwrap();
    assert_eq!(row["byte_len"], TACTICAL_DEFINITIONS_JSON.len());
    let hash = TACTICAL_DEFINITIONS_JSON
        .bytes()
        .fold(0xcbf29ce484222325_u64, |hash, byte| {
            (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3)
        });
    assert_eq!(row["checksum"]["algorithm"], "fnv1a64");
    assert_eq!(row["checksum"]["value"], format!("{hash:016x}"));
    let p = pack();
    assert_eq!(
        p,
        TacticalDefinitions::from_json(&serde_json::to_string(&p).unwrap()).unwrap()
    );
}
