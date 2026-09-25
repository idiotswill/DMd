use dmd_domain::{Ability, Condition, DamageType, DieSpec};
use dmd_rules::tactical_definitions::*;
use serde_json::{Value, json};

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
        matches!(&adult.coverage, DefinitionCoverage::SelectedFeatures { omitted } if omitted.contains(&"spellcasting-detect-magic".into()))
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

#[test]
fn chimera_and_dragon_mixed_routines_retain_real_substitution_limits() {
    let p = pack();
    let chimera = p.creature("chimera").unwrap();
    assert_eq!(chimera.statistics.hit_points, 114);
    assert_eq!(
        chimera.statistics.hit_point_formula.dice,
        vec![DieSpec {
            count: 12,
            sides: 10
        }]
    );
    let MonsterFeature::MultiattackRoutine { slots, limits } = &chimera.features[0].feature else {
        panic!("mixed source routine")
    };
    assert_eq!(
        slots.iter().map(|s| s.options.len()).collect::<Vec<_>>(),
        vec![1, 1, 2]
    );
    assert_eq!(slots[2].options[1].feature_id, "fire-breath");
    assert!(limits.is_empty());
    let MonsterFeature::Attack {
        damage,
        extra_damage_if_attack_had_advantage,
        ..
    } = &chimera
        .features
        .iter()
        .find(|f| f.id == "bite")
        .unwrap()
        .feature
    else {
        panic!("bite")
    };
    assert_eq!(damage[0].amount.dice, vec![DieSpec { count: 2, sides: 6 }]);
    assert_eq!(
        extra_damage_if_attack_had_advantage[0].amount.dice,
        vec![DieSpec { count: 2, sides: 6 }]
    );
    let dragon = p.creature("adult-red-dragon").unwrap();
    let MonsterFeature::MultiattackRoutine { slots, limits } = &dragon.features[0].feature else {
        panic!("replacement routine")
    };
    assert_eq!(slots.len(), 3);
    assert_eq!(limits[0].maximum, 1);
    assert_eq!(
        limits[0].selection.spell_id.as_deref(),
        Some("scorching-ray")
    );
    for id in ["commanding-presence", "fiery-rays"] {
        let f = dragon.features.iter().find(|f| f.id == id).unwrap();
        assert_eq!(f.usage, Some(FeatureUsage::OnceUntilOwnTurnStart));
        assert_eq!(
            f.spell_component_waivers,
            Some(SpellComponentWaivers {
                verbal: false,
                somatic: false,
                material: true
            })
        );
    }
    let command = p.spell("command").unwrap();
    assert_eq!(command.source_pages, vec![116]);
    assert!(
        matches!(&command.effects[0],EffectDescriptor::SaveCommand{choices,..} if choices.len()==5)
    );
    assert_eq!(
        p.spell("scorching-ray").unwrap().targets,
        TargetSelection::Rays { count: 3 }
    );
    let fireball = p.spell("fireball").unwrap();
    assert_eq!(fireball.source_pages, vec![131]);
    assert_eq!(fireball.range, SpellRange::Distance { feet: 150 });
    assert_eq!(
        fireball.targets,
        TargetSelection::Area {
            shape: AreaShape::Sphere { radius_feet: 20 }
        }
    );
    assert!(
        matches!(&fireball.effects[0],EffectDescriptor::SaveDamage{ability:Ability::Dexterity,damage,half_on_success:true,..} if damage.amount.dice==vec![DieSpec{count:8,sides:6}])
    );
    rejects(|v| {
        v["creatures"][5]["features"][0]["feature"]["MultiattackRoutine"]["limits"][0]["maximum"] =
            json!(0)
    });
    rejects(|v| {
        v["creatures"][7]["features"][0]["feature"]["MultiattackRoutine"]["slots"][2]["options"]
            [1]["feature_id"] = json!("multiattack")
    });
}

#[test]
fn mixed_multiattack_limits_require_a_complete_feasible_assignment() {
    let mut value: Value = serde_json::from_str(TACTICAL_DEFINITIONS_JSON).unwrap();
    let dragon = value["creatures"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|creature| creature["id"] == "adult-red-dragon")
        .unwrap();
    let routine = &mut dragon["features"][0]["feature"]["MultiattackRoutine"];
    let attack = routine["slots"][0]["options"][0].clone();
    let ray = routine["slots"][0]["options"][1].clone();
    // An early greedy ray choice must be reassigned to leave it for the only-ray slot.
    routine["slots"] = json!([
        {"options": [ray.clone(), attack.clone()]},
        {"options": [ray.clone()]},
        {"options": [attack]},
    ]);
    let text = serde_json::to_string(&value).unwrap();
    TacticalDefinitions::from_json(&text).expect("a complete non-greedy source assignment exists");
    let dragon = value["creatures"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|creature| creature["id"] == "adult-red-dragon")
        .unwrap();
    dragon["features"][0]["feature"]["MultiattackRoutine"]["slots"] = json!([
        {"options": [ray.clone()]},
        {"options": [ray.clone()]},
        {"options": [ray]},
    ]);
    let error =
        TacticalDefinitions::from_json(&serde_json::to_string(&value).unwrap()).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("shared limits prevent completing")
    );
}

#[test]
fn mage_reaction_and_preparation_are_source_grants_not_fabricated_pc_spells() {
    let p = pack();
    let mage = p.creature("mage").unwrap();
    assert_eq!(mage.source_pages, vec![305]);
    assert_eq!(mage.statistics.ability_scores, [9, 14, 11, 17, 12, 11]);
    assert_eq!(mage.statistics.saving_throw_modifiers, [-1, 2, 0, 6, 4, 0]);
    assert_eq!(mage.statistics.armor_class, 15);
    assert_eq!(
        mage.statistics.prepared_defense.as_ref().unwrap().spell_id,
        "mage-armor"
    );
    assert_eq!(mage.statistics.gear, ["wand"]);
    assert_eq!(mage.statistics.additional_languages, 3);
    let reaction = mage
        .features
        .iter()
        .find(|f| f.id == "protective-magic")
        .unwrap();
    assert_eq!(reaction.activation, FeatureActivation::Reaction);
    assert_eq!(reaction.usage, Some(FeatureUsage::PerLongRest { uses: 3 }));
    let MonsterFeature::Spellcasting {
        ability,
        save_dc,
        attack_bonus,
        spells,
    } = &reaction.feature
    else {
        panic!("source spell feature")
    };
    assert_eq!(
        (*ability, *save_dc, *attack_bonus),
        (Ability::Intelligence, Some(14), None)
    );
    assert_eq!(
        spells
            .iter()
            .map(|s| (s.spell_id.as_str(), s.cast_level, s.uses_per_long_rest))
            .collect::<Vec<_>>(),
        vec![("counterspell", 3, None), ("shield", 1, None)]
    );
    let counter = p.spell("counterspell").unwrap();
    assert_eq!(counter.source_pages, [120]);
    assert_eq!(counter.range, SpellRange::Distance { feet: 60 });
    assert_eq!(
        counter.components,
        SpellComponents {
            verbal: false,
            somatic: true,
            material: None
        }
    );
    assert_eq!(
        counter.effects,
        [EffectDescriptor::InterruptSpellCasting {
            ability: Ability::Constitution,
            preserve_spell_slot: true
        }]
    );
    let armor = p.spell("mage-armor").unwrap();
    assert_eq!(armor.source_pages, [145]);
    assert_eq!(
        armor.duration,
        EffectDuration::Seconds {
            seconds: 28_800,
            concentration: false
        }
    );
    assert_eq!(
        armor.effects,
        [EffectDescriptor::BaseArmorClass {
            base: 13,
            ability: Ability::Dexterity,
            ends_when_wearing_armor: true
        }]
    );
    // New optional source metadata must not change any older typed source pin.
    for old in p.creatures.iter().filter(|c| c.id != "mage") {
        assert!(
            !serde_json::to_value(old).unwrap()["statistics"]
                .as_object()
                .unwrap()
                .contains_key("prepared_defense")
        );
    }
}

#[test]
fn malformed_reaction_or_prepared_defense_source_is_rejected() {
    fn spell<'a>(v: &'a mut Value, id: &str) -> &'a mut Value {
        v["spells"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|s| s["id"] == id)
            .unwrap()
    }
    fn mage(v: &mut Value) -> &mut Value {
        v["creatures"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|c| c["id"] == "mage")
            .unwrap()
    }
    rejects(|v| spell(v, "counterspell")["range"] = json!({"Distance":{"feet":120}}));
    rejects(|v| {
        spell(v, "counterspell")["effects"][0]["InterruptSpellCasting"]["preserve_spell_slot"] =
            json!(false)
    });
    rejects(|v| {
        spell(v, "counterspell")["effects"][0]["InterruptSpellCasting"]["ability"] =
            json!("Intelligence")
    });
    rejects(|v| spell(v, "counterspell")["targets"]["Creatures"]["requires_sight"] = json!(false));
    rejects(|v| spell(v, "mage-armor")["duration"]["Seconds"]["seconds"] = json!(86400));
    rejects(|v| {
        spell(v, "mage-armor")["effects"][0]["BaseArmorClass"]["ends_when_wearing_armor"] =
            json!(false)
    });
    rejects(|v| mage(v)["statistics"]["prepared_defense"]["spell_id"] = json!("shield"));
    rejects(|v| mage(v)["statistics"]["armor_class"] = json!(18));
    rejects(|v| {
        mage(v)["features"][1]["feature"]["Spellcasting"]["spells"][0]["spell_id"] =
            json!("mage-armor")
    });
    rejects(|v| {
        mage(v)["features"][0]["feature"]["Spellcasting"]["spells"][0]["spell_id"] = json!("shield")
    });
}

#[test]
fn night_hag_selected_spellcasting_preserves_printed_statistics_and_omissions() {
    let p = pack();
    let hag = p.creature("night-hag").unwrap();
    let stats = &hag.statistics;
    assert_eq!(hag.source_pages, [311]);
    assert_eq!(stats.size, SourceSize::Medium);
    assert_eq!(stats.allowed_sizes, [SourceSize::Medium]);
    assert_eq!(stats.creature_type, CreatureType::Fiend);
    assert!(stats.creature_tags.is_empty());
    assert_eq!(stats.alignment, "Neutral Evil");
    assert_eq!((stats.armor_class, stats.hit_points), (17, 112));
    assert_eq!(
        stats.hit_point_formula.dice,
        [DieSpec {
            count: 15,
            sides: 8
        }]
    );
    assert_eq!(stats.hit_point_formula.fixed, 45);
    assert_eq!(stats.ability_scores, [18, 15, 16, 16, 14, 16]);
    assert_eq!(stats.saving_throw_modifiers, [4, 2, 3, 3, 2, 3]);
    assert_eq!((stats.initiative_modifier, stats.proficiency_bonus), (5, 3));
    assert_eq!(
        (stats.challenge_rating.as_str(), stats.experience_points),
        ("5", 1800)
    );
    assert_eq!(stats.speeds.walk, 30);
    assert_eq!(
        (
            stats.speeds.fly,
            stats.speeds.climb,
            stats.speeds.swim,
            stats.speeds.burrow
        ),
        (0, 0, 0, 0)
    );
    assert!(!stats.speeds.hover);
    assert_eq!(
        stats
            .skills
            .iter()
            .map(|s| (s.skill, s.modifier))
            .collect::<Vec<_>>(),
        vec![
            (dmd_domain::Skill::Deception, 6),
            (dmd_domain::Skill::Insight, 5),
            (dmd_domain::Skill::Perception, 5),
            (dmd_domain::Skill::Stealth, 5),
        ]
    );
    assert_eq!(
        (stats.senses.darkvision, stats.senses.passive_perception),
        (120, 15)
    );
    assert_eq!(
        (
            stats.senses.blindsight,
            stats.senses.tremorsense,
            stats.senses.truesight
        ),
        (0, 0, 0)
    );
    assert_eq!(
        stats.damage_resistances,
        [DamageType::Cold, DamageType::Fire]
    );
    assert!(stats.damage_immunities.is_empty() && stats.damage_vulnerabilities.is_empty());
    assert_eq!(stats.condition_immunities, [Condition::Charmed]);
    assert!(!stats.exhaustion_immune);
    assert_eq!(
        stats.languages,
        ["Abyssal", "Common", "Infernal", "Primordial"]
    );
    assert_eq!(stats.additional_languages, 0);
    assert!(stats.can_speak && stats.gear.is_empty());
    assert!(stats.prepared_defense.is_none());
    assert!(hag.traits.is_empty() && hag.legendary_budget.is_none());
    assert_eq!(hag.features.len(), 1);
    let feature = &hag.features[0];
    assert_eq!(feature.id, "spellcasting");
    assert_eq!(feature.activation, FeatureActivation::Action);
    assert_eq!(feature.source_page, 311);
    assert!(feature.usage.is_none());
    assert_eq!(
        serde_json::to_value(feature.spell_component_waivers).unwrap(),
        json!({"verbal":false,"somatic":false,"material":true})
    );
    let MonsterFeature::Spellcasting {
        ability,
        save_dc,
        attack_bonus,
        spells,
    } = &feature.feature
    else {
        panic!("source spellcasting")
    };
    assert_eq!(
        (*ability, *save_dc, *attack_bonus),
        (Ability::Intelligence, Some(14), None)
    );
    assert_eq!(spells.len(), 1);
    assert_eq!(
        (
            spells[0].spell_id.as_str(),
            spells[0].cast_level,
            spells[0].uses_per_long_rest
        ),
        ("magic-missile", 4, None)
    );
    assert_eq!(
        serde_json::to_value(&hag.coverage).unwrap(),
        json!({"SelectedFeatures":{"omitted":[
            "coven-magic", "magic-resistance", "soul-bag", "multiattack", "claw", "nightmare-haunting",
            "spellcasting-detect-magic", "spellcasting-etherealness", "spellcasting-phantasmal-killer",
            "spellcasting-plane-shift", "shape-shift"
        ]}})
    );
}
