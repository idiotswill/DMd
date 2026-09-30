use dmd_domain::*;
use dmd_rules::{tactical_creatures::*, tactical_definitions::SourceSize};

fn fixture() -> (CampaignState, CommandMeta, EntityId) {
    let actor = EntityId::new();
    let campaign = Campaign {
        id: CampaignId::new(),
        display_name: "World".into(),
        status: CampaignStatus::Active,
        world_seed: 1,
        ruleset: VersionedRef {
            id: "srd-5.2".into(),
            version: "5.2.1".into(),
        },
        content_packs: vec![],
    };
    let mut state = CampaignState::empty(
        campaign,
        WorldClock {
            now: WorldInstant(0),
            calendar_id: "seconds".into(),
        },
    );
    state.applied_event_sequence = 10;
    state.entities.insert(
        actor,
        WorldEntity {
            id: actor,
            campaign_id: state.campaign_id(),
            display_name: "Independent name".into(),
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
        expected_event_sequence: 10,
    };
    (state, meta, actor)
}

#[test]
fn mage_import_requires_real_preparation_and_retains_exact_physical_components() {
    let (state, meta, actor) = fixture();
    let mut selection = choice("mage", CreatureSize::Small);
    selection.additional_languages = vec!["Elvish".into(), "Dwarvish".into(), "Draconic".into()];
    let built = build_creature(&state, &meta, actor, &selection).unwrap();
    assert_eq!(built.mechanics.armor, ArmorClass::Fixed(12));
    assert_eq!(built.mechanics.hp, 81);
    assert_eq!(built.mechanics.hit_dice.maximum, 18);
    assert!(built.mechanics.prepared_spells.is_empty());
    assert!(built.mechanics.spellcasting.is_none());
    assert_eq!(
        built.runtime.limited_uses,
        [CreatureLimitedUse {
            feature_id: "protective-magic".into(),
            spell_id: None,
            spent: 0
        }]
    );
    validate_creature_profile(&state, &built.profile, &built.mechanics).unwrap();
    let mut forged = built.mechanics.clone();
    forged.armor = ArmorClass::Fixed(15);
    assert!(validate_creature_profile(&state, &built.profile, &forged).is_err());
    let equipment =
        dmd_rules::tactical_creature_equipment::creature_equipment_plan("mage", 0).unwrap();
    assert_eq!(
        equipment
            .iter()
            .map(|e| (e.definition_id.as_str(), e.quantity))
            .collect::<Vec<_>>(),
        vec![("spell-material:mage-armor", 1), ("wand", 1)]
    );
    let wand = dmd_rules::tactical_inventory::equipment_definition("wand").unwrap();
    assert_eq!(wand.source_page, 96);
    assert_eq!(
        wand.stacking,
        dmd_rules::tactical_inventory::ItemStacking::Individual
    );
}
fn choice(id: &str, size: CreatureSize) -> CreatureBuildChoice {
    CreatureBuildChoice {
        definition_id: id.into(),
        size,
        additional_languages: vec![],
        hit_points: CreatureHitPointChoice::Average,
        controller: CreatureController::Autonomous,
        in_lair: false,
    }
}

#[test]
fn night_hag_profile_has_source_saves_and_at_will_grant_without_pc_slots_or_gear() {
    let (state, meta, actor) = fixture();
    let selection = choice("night-hag", CreatureSize::Medium);
    let built = build_creature(&state, &meta, actor, &selection).unwrap();
    assert_eq!(built.mechanics.armor, ArmorClass::Fixed(17));
    assert_eq!((built.mechanics.hp, built.mechanics.max_hp), (112, 112));
    assert_eq!(
        built.mechanics.hit_dice,
        HitDice {
            sides: 8,
            maximum: 15,
            remaining: 15
        }
    );
    assert_eq!(built.mechanics.level, 0);
    assert!(built.mechanics.prepared_spells.is_empty());
    assert!(built.mechanics.spellcasting.is_none());
    assert!(built.mechanics.resources.is_empty());
    assert!(built.runtime.limited_uses.is_empty() && built.runtime.recharge.is_empty());
    assert_eq!(
        built.mechanics.resistances,
        [DamageType::Cold, DamageType::Fire].into()
    );
    assert_eq!(
        built.mechanics.condition_immunities,
        [Condition::Charmed].into()
    );
    assert_eq!(built.movement.walk, 60);
    assert_eq!(built.senses.darkvision, 240);
    for (ability, modifier) in [
        (Ability::Strength, 4),
        (Ability::Dexterity, 2),
        (Ability::Constitution, 3),
        (Ability::Intelligence, 3),
        (Ability::Wisdom, 2),
        (Ability::Charisma, 3),
    ] {
        assert_eq!(
            creature_test_modifier(
                &built.profile,
                &built.mechanics,
                &TestKind::Save { ability }
            )
            .unwrap(),
            modifier
        );
    }
    assert_eq!(
        creature_test_modifier(&built.profile, &built.mechanics, &TestKind::Initiative).unwrap(),
        5
    );
    validate_creature_profile(&state, &built.profile, &built.mechanics).unwrap();
    let mut forged = built.profile.clone();
    forged.source.definition_fingerprint = "0000000000000000".into();
    assert!(validate_creature_profile(&state, &forged, &built.mechanics).is_err());
    assert!(
        build_creature(
            &state,
            &meta,
            actor,
            &choice("night-hag", CreatureSize::Small)
        )
        .is_err()
    );
    let mut invented_language = selection;
    invented_language.additional_languages.push("Elvish".into());
    assert!(build_creature(&state, &meta, actor, &invented_language).is_err());
    assert!(
        dmd_rules::tactical_creature_equipment::creature_equipment_plan("night-hag", 0)
            .unwrap()
            .is_empty()
    );
    assert!(
        dmd_rules::tactical_creature_equipment::creature_equipment_plan("night-hag", 1).is_err()
    );
}

#[test]
fn source_statistics_use_real_hit_dice_and_explicit_modifiers_without_fake_pc_levels() {
    let (state, meta, actor) = fixture();
    let built = build_creature(
        &state,
        &meta,
        actor,
        &choice("adult-red-dragon", CreatureSize::Huge),
    )
    .unwrap();
    assert_eq!(built.mechanics.level, 0);
    assert_eq!(
        built.mechanics.hit_dice,
        HitDice {
            sides: 12,
            maximum: 19,
            remaining: 19
        }
    );
    assert_eq!(built.mechanics.max_hp, 256);
    assert_eq!(built.mechanics.armor, ArmorClass::Fixed(19));
    assert_eq!(
        creature_test_modifier(&built.profile, &built.mechanics, &TestKind::Initiative).unwrap(),
        12
    );
    assert_eq!(
        creature_test_modifier(
            &built.profile,
            &built.mechanics,
            &TestKind::Save {
                ability: Ability::Dexterity
            }
        )
        .unwrap(),
        6
    );
    assert_eq!(
        creature_test_modifier(
            &built.profile,
            &built.mechanics,
            &TestKind::Check {
                ability: Ability::Wisdom,
                skill: Some(Skill::Perception)
            }
        )
        .unwrap(),
        13
    );
    assert_eq!(creature_proficiency_bonus(&built.profile).unwrap(), 6);
    assert_eq!(built.movement.fly, Some(160));
    assert_eq!(built.senses.blindsight, 120);
    assert_eq!(built.senses.darkvision, 240);
    assert!(built.mechanics.attacks.is_empty());
    assert!(built.mechanics.saving_proficiencies.is_empty());
    validate_creature_profile(&state, &built.profile, &built.mechanics).unwrap();
    assert_eq!(built.profile.origin, meta);
}

#[test]
fn every_represented_definition_constructs_and_roundtrips_without_losing_source_identity() {
    for source in &creature_definitions().unwrap().creatures {
        let (state, meta, actor) = fixture();
        let mut selection = choice(&source.id, source_size(source.statistics.size));
        selection.additional_languages = (0..source.statistics.additional_languages)
            .map(|index| format!("Chosen language {index}"))
            .collect();
        let built = build_creature(&state, &meta, actor, &selection).unwrap();
        let restored: CreatureProfile =
            serde_json::from_slice(&serde_json::to_vec(&built.profile).unwrap()).unwrap();
        assert_eq!(restored, built.profile);
        assert_eq!(source_for_profile(&restored).unwrap(), source);
        validate_creature_profile(&state, &restored, &built.mechanics).unwrap();
        assert_eq!(
            built.mechanics.hit_dice.maximum,
            u8::try_from(source.statistics.hit_point_formula.dice[0].count).unwrap()
        );
        assert_eq!(
            built
                .mechanics
                .damage_immunities
                .iter()
                .copied()
                .collect::<Vec<_>>(),
            source.statistics.damage_immunities
        );
        assert!(!built.mechanics.uses_death_saves);
    }
    assert_eq!(source_size(SourceSize::Tiny), CreatureSize::Tiny);
    assert_eq!(
        source_size(SourceSize::Gargantuan),
        CreatureSize::Gargantuan
    );
}

#[test]
fn raw_hp_faces_are_retained_and_resolved_instead_of_combined_with_the_average() {
    let (state, meta, actor) = fixture();
    let id = RollRequestId::new();
    let mut selection = choice("goblin-warrior", CreatureSize::Small);
    let result = RollResult {
        request_id: id,
        source: RollSource::Physical,
        dice: vec![DieResult { sides: 6, value: 2 }; 3],
    };
    selection.hit_points = CreatureHitPointChoice::Rolled {
        request_id: id,
        result: result.clone(),
    };
    let built = build_creature(&state, &meta, actor, &selection).unwrap();
    assert_eq!(built.mechanics.max_hp, 6);
    let CreatureHitPointOrigin::Rolled {
        request,
        result: recorded,
    } = built.profile.hit_points.clone()
    else {
        panic!("raw HP must persist")
    };
    assert_eq!(recorded, result);
    assert_eq!(request.modifier, 0);
    assert_eq!(request.visibility, RollVisibility::Secret);
    let mut corrupt = built.profile.clone();
    if let CreatureHitPointOrigin::Rolled { request, .. } = &mut corrupt.hit_points {
        request.modifier = 100;
    }
    assert!(validate_creature_profile(&state, &corrupt, &built.mechanics).is_err());
    let CreatureHitPointChoice::Rolled { result, .. } = &mut selection.hit_points else {
        unreachable!()
    };
    result.dice[0].value = 7;
    assert!(build_creature(&state, &meta, actor, &selection).is_err());
}

#[test]
fn live_vitality_changes_do_not_rewrite_creation_evidence_or_restore_spent_hd() {
    let (state, meta, actor) = fixture();
    let built = build_creature(
        &state,
        &meta,
        actor,
        &choice("young-red-dragon", CreatureSize::Large),
    )
    .unwrap();
    let mut live = built.mechanics.clone();
    live.max_hp = 150;
    live.hp = 3;
    live.temporary_hp = 10;
    live.hit_dice.remaining = 2;
    live.exhaustion = 1;
    validate_creature_profile(&state, &built.profile, &live).unwrap();
    assert_eq!(
        creature_test_modifier(&built.profile, &live, &TestKind::Initiative).unwrap(),
        2
    );
    assert_eq!(built.profile.hit_points, CreatureHitPointOrigin::Average);
    for case in 0..5 {
        let mut corrupt = live.clone();
        match case {
            0 => corrupt.level = 10,
            1 => corrupt.hit_dice.maximum = 10,
            2 => corrupt.ability_scores[0] += 1,
            3 => corrupt.armor = ArmorClass::Fixed(30),
            4 => {
                corrupt.attacks.insert("dagger".into());
            }
            _ => unreachable!(),
        }
        assert!(validate_creature_profile(&state, &built.profile, &corrupt).is_err());
    }
}

#[test]
fn invalid_source_choices_authority_and_provenance_are_rejected_without_mutation() {
    let (state, meta, actor) = fixture();
    let before = state.clone();
    let selection = choice("wolf", CreatureSize::Medium);
    for case in 0..7 {
        let mut command = meta.clone();
        let mut candidate = selection.clone();
        match case {
            0 => command.expected_event_sequence -= 1,
            1 => command.campaign_id = CampaignId::new(),
            2 => command.issuer = CommandIssuer::Import,
            3 => command.actor = Some(AgentRef::Entity(EntityId::new())),
            4 => candidate.size = CreatureSize::Gargantuan,
            5 => candidate.additional_languages.push("Common".into()),
            6 => candidate.definition_id = "invented-monster".into(),
            _ => unreachable!(),
        }
        assert!(build_creature(&state, &command, actor, &candidate).is_err());
        assert_eq!(state, before);
    }
    let built = build_creature(&state, &meta, actor, &selection).unwrap();
    let mut profile = built.profile.clone();
    profile.source.definition_fingerprint = "0000000000000000".into();
    assert!(validate_creature_profile(&state, &profile, &built.mechanics).is_err());
    let mut value = serde_json::to_value(built.profile).unwrap();
    value["caller_attack_bonus"] = serde_json::json!(100);
    assert!(serde_json::from_value::<CreatureProfile>(value).is_err());
}

#[test]
fn air_source_is_additive_faithful_and_requires_an_exact_pin() {
    use dmd_rules::tactical_definitions::*;
    let bytes = TACTICAL_DEFINITIONS_JSON.as_bytes();
    let hash = bytes.iter().fold(0xcbf29ce484222325u64, |h, b| {
        (h ^ u64::from(*b)).wrapping_mul(0x100000001b3)
    });
    assert_eq!(
        hash, 0x83e72a9963e5b579,
        "historical V1 bytes must not change"
    );
    assert!(creature_definition("air-elemental").is_err());
    let air = bundled_air_elemental().unwrap();
    let pin = creature_source_pin(air).unwrap();
    assert_eq!(creature_source(&pin).unwrap(), air);
    assert_eq!(air.coverage, DefinitionCoverage::CompleteStatBlock);
    assert_eq!(air.source_pages, [258, 259]);
    assert_eq!(air.statistics.ability_scores, [14, 20, 14, 6, 10, 6]);
    assert_eq!(air.statistics.saving_throw_modifiers, [2, 5, 2, -2, 0, -2]);
    assert_eq!(air.statistics.proficiency_bonus, 3);
    assert_eq!(air.statistics.challenge_rating, "5");
    assert_eq!(air.statistics.experience_points, 1800);
    assert_eq!(air.statistics.senses.passive_perception, 10);
    assert_eq!(air.statistics.languages, ["Primordial (Auran)"]);
    assert!(air.statistics.can_speak && air.statistics.exhaustion_immune);
    assert_eq!(
        air.traits,
        [MonsterTrait::AirForm {
            minimum_passage_inches: 1
        }]
    );
    let multiattack = air.features.iter().find(|f| f.id == "multiattack").unwrap();
    assert_eq!(multiattack.source_page, 258);
    assert!(
        matches!(&multiattack.feature, MonsterFeature::Multiattack { count: 2, attack_options }
        if attack_options == &["thunderous-slam"])
    );
    let slam = air
        .features
        .iter()
        .find(|f| f.id == "thunderous-slam")
        .unwrap();
    assert_eq!(slam.source_page, 258);
    assert!(matches!(&slam.feature, MonsterFeature::Attack {
        bonus: 8, delivery: AttackDelivery::Melee { reach_feet: 10 }, damage,
        extra_damage_if_attack_had_advantage, conditional_hits
    } if damage == &[DamageComponent {
        amount: DamageFormula { dice: vec![DieSpec { count: 2, sides: 8 }], fixed: 5 },
        damage_type: DamageType::Thunder,
    }] && extra_damage_if_attack_had_advantage.is_empty() && conditional_hits.is_empty()));
    let whirlwind = air.features.iter().find(|f| f.id == "whirlwind").unwrap();
    assert_eq!(whirlwind.source_page, 259);
    assert_eq!(
        whirlwind.usage,
        Some(FeatureUsage::Recharge(Recharge {
            die_sides: 6,
            minimum: 4,
            maximum: 6
        }))
    );
    assert!(
        matches!(&whirlwind.feature, MonsterFeature::SharedSpaceSave {
        target_size_at_most: SourceSize::Medium, ability: Ability::Strength, dc:13,
        push_up_to_feet:20, condition_on_failure:Condition::Prone, half_damage_on_success:true,
        damage: DamageComponent { amount: DamageFormula { dice, fixed:2 }, damage_type:DamageType::Thunder }
    } if dice == &[DieSpec { count:4, sides:10 }])
    );
    let (state, meta, actor) = fixture();
    let selection = choice("air-elemental", CreatureSize::Large);
    assert!(build_creature(&state, &meta, actor, &selection).is_err());
    let built = build_creature_from_source(&state, &meta, actor, &selection, Some(&pin)).unwrap();
    assert_eq!(built.mechanics.armor, ArmorClass::Fixed(15));
    assert_eq!((built.mechanics.hp, built.mechanics.max_hp), (90, 90));
    assert_eq!(
        built.mechanics.resistances,
        [
            DamageType::Bludgeoning,
            DamageType::Lightning,
            DamageType::Piercing,
            DamageType::Slashing
        ]
        .into()
    );
    assert_eq!(
        built.mechanics.damage_immunities,
        [DamageType::Poison, DamageType::Thunder].into()
    );
    assert_eq!(
        built.mechanics.condition_immunities,
        [
            Condition::Grappled,
            Condition::Paralyzed,
            Condition::Petrified,
            Condition::Poisoned,
            Condition::Prone,
            Condition::Restrained,
            Condition::Unconscious
        ]
        .into()
    );
    assert_eq!(
        (
            built.movement.walk,
            built.movement.fly,
            built.movement.hover
        ),
        (20, Some(180), true)
    );
    assert_eq!(built.senses.darkvision, 120);
    assert_eq!(
        creature_test_modifier(&built.profile, &built.mechanics, &TestKind::Initiative).unwrap(),
        5
    );
    for (ability, expected) in [
        (Ability::Strength, 2),
        (Ability::Dexterity, 5),
        (Ability::Constitution, 2),
        (Ability::Intelligence, -2),
        (Ability::Wisdom, 0),
        (Ability::Charisma, -2),
    ] {
        assert_eq!(
            creature_test_modifier(
                &built.profile,
                &built.mechanics,
                &TestKind::Save { ability }
            )
            .unwrap(),
            expected
        );
    }
    validate_creature_profile(&state, &built.profile, &built.mechanics).unwrap();
    let restored: CreatureProfile =
        serde_json::from_slice(&serde_json::to_vec(&built.profile).unwrap()).unwrap();
    assert_eq!(source_for_profile(&restored).unwrap(), air);
    assert!(
        dmd_rules::tactical_creature_equipment::creature_equipment_plan_from_source(&pin, 0)
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        dmd_rules::tactical_creature_equipment::creature_attack_gear(
            &built.profile,
            "thunderous-slam"
        )
        .unwrap(),
        None
    );
    for field in 0..4 {
        let mut forged = pin.clone();
        match field {
            0 => forged.ruleset_id.push('x'),
            1 => forged.ruleset_version.push('x'),
            2 => forged.definition_id = "wolf".into(),
            _ => forged.definition_fingerprint = "0000000000000000".into(),
        }
        assert!(creature_source(&forged).is_err());
        assert!(
            build_creature_from_source(&state, &meta, actor, &selection, Some(&forged)).is_err()
        );
    }
    assert!(
        source_for_actor(&state, actor, "air-elemental").is_err(),
        "profileless lookup is V1 only"
    );
}
