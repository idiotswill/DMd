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
