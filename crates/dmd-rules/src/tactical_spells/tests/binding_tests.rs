use super::*;
use crate::tactical_creatures::{CreatureBuildChoice, CreatureHitPointChoice, build_creature};

fn point(x: i32, y: i32, z: i32) -> SpatialPoint {
    SpatialPoint { x, y, z }
}

fn scene(spell: &str, level: u8, target_source: &str) -> (CampaignState, SpellCastPlan, EntityId) {
    let resource = if level == 0 {
        SpellResourceChoice::Cantrip
    } else {
        SpellResourceChoice::Slot { level }
    };
    let (mut state, meta, mut choice) = fixture(spell, resource);
    let actor = choice.actor;
    let target = EntityId::new();
    let location = LocationId::new();
    let scene_id = SceneId::new();
    state.locations.insert(
        location,
        Location {
            id: location,
            campaign_id: state.campaign_id(),
            display_name: "A place".into(),
            parent_location_id: None,
        },
    );
    state.entities.get_mut(&actor).unwrap().location_id = Some(location);
    state.entities.insert(
        target,
        WorldEntity {
            id: target,
            campaign_id: state.campaign_id(),
            display_name: "Private identity".into(),
            kind: EntityKind::Creature,
            existence: EntityExistence::Present,
            location_id: Some(location),
        },
    );
    let source = crate::tactical_creatures::creature_definition(target_source).unwrap();
    let built = build_creature(
        &state,
        &meta,
        target,
        &CreatureBuildChoice {
            definition_id: target_source.into(),
            size: crate::tactical_creatures::source_size(source.statistics.size),
            additional_languages: vec![],
            hit_points: CreatureHitPointChoice::Average,
            controller: CreatureController::Autonomous,
            in_lair: false,
        },
    )
    .unwrap();
    let rules = state.rules.as_mut().unwrap();
    rules.entities.insert(target, built.mechanics);
    rules.tactical_creatures = Some(TacticalCreatures {
        schema_version: 1,
        profiles: vec![built.profile],
        runtime: vec![built.runtime],
    });
    rules.tactical_inventory = Some(TacticalInventory {
        loadouts: vec![ActorEquipmentLoadout {
            actor,
            hands: WeaponLoadout::default(),
            worn_armor: None,
            shield: None,
            command: meta.clone(),
        }],
        ..TacticalInventory::default()
    });
    if spell == "hold-person" {
        let item = ItemId::new();
        state.items.insert(
            item,
            ItemInstance {
                id: item,
                campaign_id: state.campaign_id(),
                definition_id: "spell-material:hold-person".into(),
                display_name: "Any name".into(),
                quantity: 1,
                owner: Ownership::Unowned,
                custody: Custody::Entity(actor),
                state: ItemState::Intact,
            },
        );
        choice.material = SpellMaterialChoice::Material { item };
    }
    state.scenes.insert(
        scene_id,
        Scene {
            id: scene_id,
            campaign_id: state.campaign_id(),
            location_id: location,
            mode: SceneMode::Combat,
            status: SceneStatus::Active,
            started_at: state.clock.now,
            presences: [actor, target]
                .into_iter()
                .map(|entity_id| ScenePresence {
                    entity_id,
                    role: PresenceRole::Participant,
                })
                .collect(),
        },
    );
    let participant = |entity_id, position| TacticalParticipant {
        entity_id,
        position,
        size: CreatureSize::Medium,
        public_label: "A traveler".into(),
        height: 12,
        reach: 10,
        movement: MovementProfile {
            walk: 60,
            climb: None,
            swim: None,
            fly: None,
            burrow: None,
            hover: false,
        },
        senses: Senses::default(),
        allies: vec![],
        enemies: vec![],
    };
    let mut other = participant(target, point(20, 10, 0));
    other.size = crate::tactical_creatures::source_size(source.statistics.size);
    other.movement = built.movement;
    other.senses = built.senses;
    state.encounter = Some(TacticalEncounter {
        id: EncounterId::new(),
        scene_id,
        battlefield: Battlefield {
            bounds: SpatialBox {
                min: point(0, 0, 0),
                max: point(400, 100, 40),
            },
            floor_z: 0,
            floor_surface: "ground".into(),
            ambient_light: LightLevel::Bright,
            terrain: vec![],
            obstacles: vec![],
            lights: vec![],
        },
        participants: vec![participant(actor, point(10, 10, 0)), other],
        knowledge: vec![],
        origin: meta.clone(),
        geometry_ruling: Ruling {
            basis: RulingBasis::GmAdjudication,
            reason: "Authored source test geometry".into(),
        },
        flow: None,
    });
    let plan = plan_spell_cast(&state, &meta, &choice).unwrap();
    (state, plan, target)
}

#[test]
fn heal_binds_current_touch_and_rejects_range_without_spending() {
    let (mut state, plan, target) = scene("cure-wounds", 1, "wolf");
    let before = state.clone();
    let bound = bind_spell(&state, &plan, &SpellTargetChoice::Entities(vec![target])).unwrap();
    assert_eq!(bound.kind(), ExecutableSpellKind::Healing);
    assert_eq!(bound.targets()[0].actor(), target);
    assert_eq!(state, before);
    state.encounter.as_mut().unwrap().participants[1].position.x = 30;
    // A long weapon/stat-block attack reach does not grant long-range touching.
    state.encounter.as_mut().unwrap().participants[0].reach = 40;
    let outside = state.clone();
    assert!(bind_spell(&state, &plan, &SpellTargetChoice::Entities(vec![target])).is_err());
    assert_eq!(state, outside);
}

#[test]
fn self_healing_requires_no_external_sight_or_creature_type_disclosure() {
    let (mut state, plan, _) = scene("cure-wounds", 1, "wolf");
    state.encounter.as_mut().unwrap().battlefield.ambient_light = LightLevel::Darkness;
    let bound = bind_spell(
        &state,
        &plan,
        &SpellTargetChoice::Entities(vec![plan.choice.actor]),
    )
    .unwrap();
    assert_eq!(bound.targets()[0].actor(), plan.choice.actor);
    assert!(bound.targets()[0].valid_type());
}

#[test]
fn invalid_source_type_is_private_bound_outcome_not_free_rejection() {
    let (state, plan, target) = scene("hold-person", 2, "wolf");
    let before = state.clone();
    let bound = bind_spell(&state, &plan, &SpellTargetChoice::Entities(vec![target])).unwrap();
    assert!(!bound.targets()[0].valid_type());
    assert_eq!(state, before);
    let (state, plan, target) = scene("hold-person", 2, "cultist-fanatic");
    assert!(
        bind_spell(&state, &plan, &SpellTargetChoice::Entities(vec![target]))
            .unwrap()
            .targets()[0]
            .valid_type()
    );
}

#[test]
fn rays_and_darts_retain_every_occurrence_but_creature_selector_rejects_duplicates() {
    for spell in ["magic-missile", "scorching-ray"] {
        let (state, plan, target) =
            scene(spell, if spell == "magic-missile" { 1 } else { 2 }, "wolf");
        let selected = SpellTargetChoice::Entities(vec![target; 3]);
        let bound = bind_spell(&state, &plan, &selected).unwrap();
        assert_eq!(bound.targets().len(), 3);
        assert!(bound.targets().iter().all(|t| t.actor() == target));
        assert!(bind_spell(&state, &plan, &SpellTargetChoice::Entities(vec![target; 2])).is_err());
        let restored: CampaignState =
            CampaignState::decode_json(&state.encode_json().unwrap()).unwrap();
        assert_eq!(bind_spell(&restored, &plan, &selected).unwrap(), bound);
    }
    let (state, plan, target) = scene("hold-person", 3, "cultist-fanatic");
    assert!(bind_spell(&state, &plan, &SpellTargetChoice::Entities(vec![target; 2])).is_err());
}

#[test]
fn hidden_unlocated_ids_and_sight_required_targets_do_not_bind_through_darkness() {
    let (mut state, plan, target) = scene("hold-person", 2, "wolf");
    state.encounter.as_mut().unwrap().battlefield.ambient_light = LightLevel::Darkness;
    let hidden = bind_spell(&state, &plan, &SpellTargetChoice::Entities(vec![target])).unwrap_err();
    let absent = bind_spell(
        &state,
        &plan,
        &SpellTargetChoice::Entities(vec![EntityId::new()]),
    )
    .unwrap_err();
    assert_eq!(hidden.to_string(), absent.to_string());
    state.encounter.as_mut().unwrap().participants[1].position.x = 380;
    let hidden_far =
        bind_spell(&state, &plan, &SpellTargetChoice::Entities(vec![target])).unwrap_err();
    assert_eq!(hidden.to_string(), hidden_far.to_string());
    state.encounter.as_mut().unwrap().participants[1].position.x = 20;
    state.encounter.as_mut().unwrap().participants[0]
        .senses
        .tremorsense = 100;
    assert!(bind_spell(&state, &plan, &SpellTargetChoice::Entities(vec![target])).is_err());
    state.encounter.as_mut().unwrap().participants[0]
        .senses
        .blindsight = 100;
    assert!(bind_spell(&state, &plan, &SpellTargetChoice::Entities(vec![target])).is_ok());
}

#[test]
fn renamed_false_material_and_lost_real_material_cannot_cast() {
    let (mut state, plan, target) = scene("hold-person", 2, "wolf");
    let SpellMaterialChoice::Material { item } = plan.choice.material else {
        panic!()
    };
    state.items.get_mut(&item).unwrap().definition_id = "rope".into();
    state.items.get_mut(&item).unwrap().display_name = "spell-material:hold-person".into();
    assert!(bind_spell(&state, &plan, &SpellTargetChoice::Entities(vec![target])).is_err());
    state.items.get_mut(&item).unwrap().definition_id = "spell-material:hold-person".into();
    state.items.get_mut(&item).unwrap().custody = Custody::Missing;
    assert!(bind_spell(&state, &plan, &SpellTargetChoice::Entities(vec![target])).is_err());
}

#[test]
fn incomplete_source_programs_are_rejected_before_concentration_or_slots() {
    for (spell, level) in [
        ("fireball", 3),
        ("command", 1),
        ("fog-cloud", 1),
        ("shield", 1),
        ("dancing-lights", 0),
    ] {
        let (state, meta, choice) = fixture(
            spell,
            if level == 0 {
                SpellResourceChoice::Cantrip
            } else {
                SpellResourceChoice::Slot { level }
            },
        );
        let before = state.clone();
        let plan = plan_spell_cast(&state, &meta, &choice).unwrap();
        assert!(executable_spell_kind(&plan).is_err(), "{spell}");
        assert_eq!(state, before);
    }
}

#[test]
fn total_cover_and_charmed_source_block_harm_without_changing_state() {
    let (mut state, plan, target) = scene("fire-bolt", 0, "wolf");
    state.encounter.as_mut().unwrap().participants[1].position.x = 40;
    state
        .encounter
        .as_mut()
        .unwrap()
        .battlefield
        .obstacles
        .push(SpatialObstacle {
            id: "wall".into(),
            volume: SpatialBox {
                min: point(25, 0, 0),
                max: point(30, 100, 40),
            },
            blocks_movement: true,
            blocks_sight: false,
            observable: true,
            cover: CoverDegree::Total,
        });
    let before = state.clone();
    assert!(bind_spell(&state, &plan, &SpellTargetChoice::Entities(vec![target])).is_err());
    assert_eq!(state, before);
    state
        .encounter
        .as_mut()
        .unwrap()
        .battlefield
        .obstacles
        .clear();
    state.rules.as_mut().unwrap().effects.push(ActiveEffect {
        id: EffectId::new(),
        source: target,
        target: plan.choice.actor,
        condition: Some(Condition::Charmed),
        label: "Source charm".into(),
        expires: Expiry::Never,
        concentration_owner: None,
    });
    let before = state.clone();
    assert!(bind_spell(&state, &plan, &SpellTargetChoice::Entities(vec![target])).is_err());
    assert_eq!(state, before);
}

#[test]
fn physical_hands_and_armor_training_override_old_sheet_booleans() {
    let (mut state, plan, target) = scene("cure-wounds", 1, "wolf");
    let actor = plan.choice.actor;
    let item = ItemId::new();
    state.items.insert(
        item,
        ItemInstance {
            id: item,
            campaign_id: state.campaign_id(),
            definition_id: "leather-armor".into(),
            display_name: "A coat".into(),
            quantity: 1,
            owner: Ownership::Unowned,
            custody: Custody::Entity(actor),
            state: ItemState::Intact,
        },
    );
    state
        .rules
        .as_mut()
        .unwrap()
        .tactical_inventory
        .as_mut()
        .unwrap()
        .loadouts[0]
        .worn_armor = Some(item);
    assert!(bind_spell(&state, &plan, &SpellTargetChoice::Entities(vec![target])).is_err());
    let loadout = &mut state
        .rules
        .as_mut()
        .unwrap()
        .tactical_inventory
        .as_mut()
        .unwrap()
        .loadouts[0];
    loadout.worn_armor = None;
    loadout.hands.hands = [HandAssignment::Item(item), HandAssignment::Item(item)];
    assert!(bind_spell(&state, &plan, &SpellTargetChoice::Entities(vec![target])).is_err());
    state.rules.as_mut().unwrap().tactical_inventory = None;
    assert!(bind_spell(&state, &plan, &SpellTargetChoice::Entities(vec![target])).is_err());
}

#[test]
fn source_attack_proof_requires_committed_matching_cast_and_exact_ray() {
    let (state, plan, target) = scene("scorching-ray", 2, "wolf");
    let bound = bind_spell(&state, &plan, &SpellTargetChoice::Entities(vec![target; 3])).unwrap();
    let started = begin_cast(&plan, &context(&plan)).unwrap();
    assert!(spell_attack_occurrence(&started.cast, &bound, 0, 0).is_err());
    let committed = step(&started.cast, &context(&plan), SpellCastAdvance::Commit).cast;
    let ray = spell_attack_occurrence(&committed, &bound, 0, 2).unwrap();
    assert_eq!(ray.target(), target);
    assert_eq!(ray.actor(), plan.choice.actor);
    assert_eq!(ray.target_ordinal(), 2);
    assert_eq!(ray.intrinsic_attack_bonus(), 6);
    assert_eq!(ray.damage().dice, [DieSpec { count: 2, sides: 6 }]);
    assert!(spell_attack_occurrence(&committed, &bound, 0, 3).is_err());
    let (_, other, _) = scene("scorching-ray", 2, "wolf");
    let other = step(
        &begin_cast(&other, &context(&other)).unwrap().cast,
        &context(&other),
        SpellCastAdvance::Commit,
    )
    .cast;
    assert!(spell_attack_occurrence(&other, &bound, 0, 0).is_err());
}

#[test]
fn unresolved_slot_reservation_blocks_nested_slot_but_counterspell_exception_releases_it() {
    let (mut state, meta, choice) = fixture("cure-wounds", SpellResourceChoice::Slot { level: 1 });
    add_flow(&mut state, &meta, choice.actor, TacticalSource::Character);
    let plan = plan_spell_cast(&state, &meta, &choice).unwrap();
    let casting = begin_cast(&plan, &context(&plan)).unwrap().cast;
    let before = state.clone();
    validate_spell_slot_reservation(&state, &plan, &[]).unwrap();
    assert!(validate_spell_slot_reservation(&state, &plan, &[&casting]).is_err());
    let countered = step(&casting, &context(&plan), SpellCastAdvance::Countered).cast;
    validate_spell_slot_reservation(&state, &plan, &[&countered]).unwrap();
    assert_eq!(state, before);
    let paid = apply_spell_expenditure(&state, choice.actor, &plan.expenditure).unwrap();
    assert!(validate_spell_slot_reservation(&paid, &plan, &[]).is_err());
}

#[test]
fn retained_binding_roundtrips_source_occurrences_and_rejects_forged_shape() {
    let (state, plan, target) = scene("scorching-ray", 2, "wolf");
    let selection = SpellTargetChoice::Entities(vec![target; 3]);
    let bound = bind_spell(&state, &plan, &selection).unwrap();
    let started = begin_cast(&plan, &context(&plan)).unwrap();
    let committed = step(&started.cast, &context(&plan), SpellCastAdvance::Commit).cast;
    let record = retain_spell_cast(committed, &bound, selection, None).unwrap();
    let restored: TacticalCasting =
        serde_json::from_slice(&serde_json::to_vec(&record).unwrap()).unwrap();
    assert_eq!(record, restored);
    assert_eq!(bound, retained_spell_binding(&restored).unwrap());
    let proof = spell_attack_occurrence(
        &restored.cast,
        &retained_spell_binding(&restored).unwrap(),
        0,
        2,
    )
    .unwrap();
    assert_eq!(proof.node_ordinal(), 0);
    assert_eq!(proof.target_ordinal(), 2);
    let mut completed_record = record.clone();
    completed_record
        .completed
        .push(SpellProgramOccurrence { node: 0, target: 2 });
    assert!(
        spell_attack_occurrence(
            &completed_record.cast,
            &retained_spell_binding(&completed_record).unwrap(),
            0,
            2
        )
        .is_err()
    );
    assert!(spell_attack_occurrence(&record.cast, &bound, 1, 2).is_err());
    for bad in [
        {
            let mut r = record.clone();
            r.targets.pop();
            r
        },
        {
            let mut r = record.clone();
            r.targets[0].source_type_matches = false;
            r
        },
        {
            let mut r = record.clone();
            r.targets[0].actor = EntityId::new();
            r
        },
        {
            let mut r = record.clone();
            r.consumed_material = Some(ItemId::new());
            r
        },
        {
            let mut r = record.clone();
            r.completed = vec![SpellProgramOccurrence { node: 1, target: 0 }];
            r
        },
        {
            let mut r = record.clone();
            r.completed = vec![SpellProgramOccurrence { node: 0, target: 3 }];
            r
        },
        {
            let mut r = record.clone();
            r.completed = vec![SpellProgramOccurrence { node: 0, target: 0 }; 2];
            r
        },
        {
            let mut r = record.clone();
            r.cast.phase = SpellCastPhase::Casting;
            r.completed = vec![SpellProgramOccurrence { node: 0, target: 0 }];
            r
        },
    ] {
        assert!(validate_retained_spell(&bad).is_err());
    }
    let mut json = serde_json::to_value(&record).unwrap();
    json["prepaid"] = serde_json::json!(true);
    assert!(serde_json::from_value::<TacticalCasting>(json).is_err());
}

#[test]
fn invalid_type_evidence_is_retained_without_redeciding_current_position() {
    let (mut state, plan, target) = scene("hold-person", 2, "wolf");
    let selection = SpellTargetChoice::Entities(vec![target]);
    let bound = bind_spell(&state, &plan, &selection).unwrap();
    let record = retain_spell_cast(
        begin_cast(&plan, &context(&plan)).unwrap().cast,
        &bound,
        selection,
        None,
    )
    .unwrap();
    assert!(!record.targets[0].source_type_matches);
    // Later current geometry cannot rewrite the historical accepted admission.
    state.encounter.as_mut().unwrap().participants[1].position.x = 380;
    assert!(bind_spell(&state, &plan, record.selection.as_ref().unwrap()).is_err());
    assert!(!retained_spell_binding(&record).unwrap().targets()[0].valid_type());
}

fn executable_record(
    spell: &str,
    level: u8,
    source: &str,
) -> (CampaignState, TacticalCasting, EntityId) {
    let (mut state, plan, target) = scene(spell, level, source);
    let count = match plan.program.targets {
        SpellTargetRule::Darts { count, .. } | SpellTargetRule::Rays { count } => count,
        _ => 1,
    };
    let selection = SpellTargetChoice::Entities(vec![target; usize::from(count)]);
    let bound = bind_spell(&state, &plan, &selection).unwrap();
    let started = begin_cast(&plan, &context(&plan)).unwrap();
    for obligation in started.obligations {
        if let SpellCastObligation::BeginConcentration { group } = obligation {
            state = crate::tactical_effect_adapter::apply_effect_operation(
                &state,
                &plan.origin,
                &crate::tactical_effects::EffectLifecycleAction {
                    step: 0,
                    operation:
                        crate::tactical_effects::EffectLifecycleOperation::BeginConcentration {
                            group,
                        },
                },
            )
            .unwrap()
            .0;
        }
    }
    let committed = step(&started.cast, &context(&plan), SpellCastAdvance::Commit).cast;
    (
        state,
        retain_spell_cast(committed, &bound, selection, None).unwrap(),
        target,
    )
}

#[test]
fn source_healing_roll_has_no_fake_bonus_and_dead_target_is_a_paid_no_effect() {
    let (mut state, record, target) = executable_record("cure-wounds", 1, "wolf");
    let occurrence = SpellProgramOccurrence { node: 0, target: 0 };
    let id = RollRequestId::new();
    let request = spell_amount_request(&record, occurrence, id, RollVisibility::Public).unwrap();
    assert_eq!(request.dice, [DieSpec { count: 2, sides: 8 }]);
    assert_eq!(request.modifier, 3);
    let raw = RollResult {
        request_id: id,
        source: RollSource::Physical,
        dice: vec![
            DieResult { sides: 8, value: 4 },
            DieResult { sides: 8, value: 5 },
        ],
    };
    let operation = spell_amount_operation(&state, &record, occurrence, id, &raw).unwrap();
    assert_eq!(operation, Some(VitalityOperation::Heal { amount: 12 }));
    let actor = record.cast.plan.choice.actor;
    let mut slot_state = state.clone();
    add_flow(
        &mut slot_state,
        &record.cast.plan.origin,
        actor,
        TacticalSource::Character,
    );
    state = apply_spell_expenditure(&slot_state, actor, &record.cast.plan.expenditure).unwrap();
    let paid_slots = state.rules.as_ref().unwrap().entities[&actor]
        .spellcasting
        .as_ref()
        .unwrap()
        .slots;
    let entity = state
        .rules
        .as_mut()
        .unwrap()
        .entities
        .get_mut(&target)
        .unwrap();
    entity.hp = 0;
    entity.death.dead = true;
    entity.prone = true;
    assert!(
        spell_amount_operation(&state, &record, occurrence, id, &raw)
            .unwrap()
            .is_none()
    );
    assert_eq!(state.rules.as_ref().unwrap().entities[&target].hp, 0);
    assert!(state.rules.as_ref().unwrap().entities[&target].death.dead);
    assert_eq!(
        state.rules.as_ref().unwrap().entities[&actor]
            .spellcasting
            .as_ref()
            .unwrap()
            .slots,
        paid_slots
    );
    let mut wrong = raw.clone();
    wrong.dice.pop();
    assert!(spell_amount_operation(&state, &record, occurrence, id, &wrong).is_err());
}

#[test]
fn automatic_darts_use_individual_raw_occurrences_and_no_attack_critical_cause() {
    let (state, mut record, _) = executable_record("magic-missile", 1, "wolf");
    for ordinal in 0..3 {
        let at = SpellProgramOccurrence {
            node: 0,
            target: ordinal,
        };
        let id = RollRequestId::new();
        let raw = RollResult {
            request_id: id,
            source: RollSource::Physical,
            dice: vec![DieResult {
                sides: 4,
                value: ordinal + 1,
            }],
        };
        let operation = spell_amount_operation(&state, &record, at, id, &raw)
            .unwrap()
            .unwrap();
        assert_eq!(
            operation,
            VitalityOperation::Damage {
                packet: DamagePacket {
                    cause: DamageCause::Other,
                    components: vec![dmd_domain::DamageComponent {
                        damage_type: DamageType::Force,
                        amounts: vec![u32::from(ordinal) + 2],
                        adjustments: vec![],
                    }]
                },
                knockout: None
            }
        );
        record.completed.push(at);
        assert!(spell_amount_operation(&state, &record, at, id, &raw).is_err());
    }
}

#[test]
fn condition_program_retains_source_duration_repeat_save_and_stable_effect_identities() {
    let (mut state, record, target) = executable_record("hold-person", 2, "cultist-fanatic");
    let at = SpellProgramOccurrence { node: 0, target: 0 };
    let effect = spell_condition_effect(&state, &record, at)
        .unwrap()
        .unwrap();
    assert_eq!(
        effect.expires,
        TacticalEffectExpiry::AtTime(WorldInstant(160))
    );
    assert_eq!(effect.conditions[0].condition, Condition::Paralyzed);
    assert_ne!(effect.id, effect.conditions[0].id);
    assert_ne!(Some(effect.id), record.cast.plan.concentration_group);
    assert_eq!(
        effect.triggers[0].payload,
        EffectTriggerPayload::SavingThrow {
            ability: Ability::Wisdom,
            dc: 14,
            on_success: EffectSaveEnd::TargetEffect,
            on_failure: EffectSaveEnd::None
        }
    );
    let restored: TacticalCasting =
        serde_json::from_slice(&serde_json::to_vec(&record).unwrap()).unwrap();
    assert_eq!(
        effect,
        spell_condition_effect(&state, &restored, at)
            .unwrap()
            .unwrap()
    );
    state
        .rules
        .as_mut()
        .unwrap()
        .entities
        .get_mut(&target)
        .unwrap()
        .condition_immunities
        .insert(Condition::Paralyzed);
    assert!(
        spell_condition_effect(&state, &record, at)
            .unwrap()
            .is_none()
    );
    state
        .rules
        .as_mut()
        .unwrap()
        .entities
        .get_mut(&target)
        .unwrap()
        .condition_immunities
        .clear();
    state
        .rules
        .as_mut()
        .unwrap()
        .tactical_effects
        .as_mut()
        .unwrap()
        .groups
        .clear();
    assert!(
        spell_condition_effect(&state, &record, at)
            .unwrap()
            .is_none()
    );
    let (state, record, _) = executable_record("hold-person", 2, "wolf");
    assert!(
        spell_condition_effect(&state, &record, at)
            .unwrap()
            .is_none()
    );
}
