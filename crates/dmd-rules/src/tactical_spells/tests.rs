use super::*;

fn fixture(
    spell_id: &str,
    resource: SpellResourceChoice,
) -> (CampaignState, CommandMeta, SpellCastChoice) {
    let mut state = CampaignState::empty(
        Campaign {
            id: CampaignId::new(),
            display_name: "Independent source fixture".into(),
            status: CampaignStatus::Active,
            world_seed: 1,
            ruleset: VersionedRef {
                id: "srd-5.2".into(),
                version: "5.2.1".into(),
            },
            content_packs: vec![],
        },
        WorldClock {
            now: WorldInstant(100),
            calendar_id: "seconds".into(),
        },
    );
    let actor = EntityId::new();
    state.entities.insert(
        actor,
        WorldEntity {
            id: actor,
            campaign_id: state.campaign_id(),
            display_name: "Any name".into(),
            kind: EntityKind::Character,
            existence: EntityExistence::Present,
            location_id: None,
        },
    );
    let mut entity = MechanicalEntity::basic(actor);
    entity.level = 5;
    entity.ability_scores[Ability::Wisdom.index()] = 16;
    entity.prepared_spells.insert(spell_id.into());
    entity.spellcasting = Some(Spellcasting {
        ability: Ability::Wisdom,
        slot_maxima: [4; 9],
        slots: [4; 9],
        can_speak: true,
        free_hand: true,
        material_focus: true,
    });
    state.rules = Some(RulesState {
        pack_id: "srd-5.2".into(),
        pack_version: "5.2.1".into(),
        entities: [(actor, entity)].into(),
        house_rules: HouseRules::default(),
        effects: vec![],
        tactical_effects: None,
        tactical_inventory: None,
        tactical_recovery: None,
        pending: None,
        rolls: vec![],
        cancelled_roll_ids: vec![],
        rulings: vec![],
        timing: None,
        rests: vec![],
        completed_short_rests: vec![],
        permission: None,
    });
    state.applied_event_sequence = 7;
    let meta = CommandMeta {
        id: CommandId::new(),
        campaign_id: state.campaign_id(),
        session_id: None,
        issuer: CommandIssuer::System,
        actor: None,
        expected_event_sequence: 7,
    };
    let choice = SpellCastChoice {
        actor,
        spell_id: spell_id.into(),
        grant: SpellGrantChoice::Prepared,
        resource,
        material: SpellMaterialChoice::None,
        mode: SpellCastMode::Immediate,
    };
    (state, meta, choice)
}
fn context(plan: &SpellCastPlan) -> SpellCastContext {
    SpellCastContext {
        now: WorldInstant(100),
        turn_number: 3,
        current_actor: plan.choice.actor,
        concentration: plan.concentration_group,
    }
}
fn next_meta(cast: &SpellCast) -> CommandMeta {
    CommandMeta {
        id: CommandId::new(),
        expected_event_sequence: cast.last_operation.expected_event_sequence + 1,
        ..cast.last_operation.clone()
    }
}
fn step(
    cast: &SpellCast,
    context: &SpellCastContext,
    operation: SpellCastAdvance,
) -> SpellCastTransition {
    let meta = next_meta(cast);
    let before = cast.clone();
    let restored: SpellCast = serde_json::from_slice(&serde_json::to_vec(cast).unwrap()).unwrap();
    let result = advance_cast(cast, &meta, context, operation).unwrap();
    assert_eq!(
        advance_cast(&restored, &meta, context, operation).unwrap(),
        result
    );
    assert_eq!(*cast, before);
    validate_spell_cast(&result.cast).unwrap();
    result
}

#[test]
fn nested_cast_identity_is_stable_distinct_and_bounded() {
    let (state, meta, choice) = fixture("hold-person", SpellResourceChoice::Slot { level: 2 });
    let first = plan_spell_cast_at(&state, &meta, &choice, 4).unwrap();
    let second = plan_spell_cast_at(&state, &meta, &choice, 5).unwrap();
    assert_ne!(first.concentration_group, second.concentration_group);
    assert_ne!(
        first.concentration_group.unwrap(),
        spell_concentration_id(meta.id, EntityId::new(), 4)
    );
    assert_eq!(
        first,
        plan_spell_cast_at(&state, &meta, &choice, 4).unwrap()
    );
    let restored: SpellCastPlan =
        serde_json::from_slice(&serde_json::to_vec(&first).unwrap()).unwrap();
    validate_spell_plan(&restored).unwrap();
    let begun = begin_cast(&restored, &context(&restored)).unwrap();
    assert!(begun.obligations.iter().any(|o| matches!(o,
        SpellCastObligation::BeginConcentration { group } if group.source.ordinal == 4
    )));
    let committed = step(&begun.cast, &context(&restored), SpellCastAdvance::Commit);
    assert!(committed.obligations.iter().any(|o| matches!(o,
        SpellCastObligation::QueueProgram { origin, occurrence: 4 } if *origin == meta
    )));
    assert!(plan_spell_cast_at(&state, &meta, &choice, 32_768).is_err());
    let mut forged = first.clone();
    forged.occurrence = 5;
    assert!(validate_spell_plan(&forged).is_err());
    forged.occurrence = u16::MAX;
    assert!(validate_spell_plan(&forged).is_err());
}

#[test]
fn prepared_source_derives_modifier_upcast_healing_and_does_not_mutate_slots() {
    let (state, meta, choice) = fixture("cure-wounds", SpellResourceChoice::Slot { level: 3 });
    let before = state.clone();
    let plan = plan_spell_cast(&state, &meta, &choice).unwrap();
    assert_eq!(plan.program.save_dc, Some(14));
    assert_eq!(plan.program.attack_bonus, Some(6));
    assert_eq!(
        plan.program.nodes,
        [SpellProgramNode::Heal {
            dice: vec![DieSpec { count: 6, sides: 8 }],
            modifier: 3
        }]
    );
    assert_eq!(state, before);
}

#[test]
fn cantrip_upgrade_uses_character_level_and_never_a_spell_slot() {
    let (state, meta, choice) = fixture("fire-bolt", SpellResourceChoice::Cantrip);
    let plan = plan_spell_cast(&state, &meta, &choice).unwrap();
    assert_eq!(plan.expenditure, SpellExpenditure::None);
    assert!(
        matches!(&plan.program.nodes[0], SpellProgramNode::AttackDamage { damage, share: SpellDamageShare::PerAttack, .. }
        if damage.dice == [DieSpec { count: 2, sides: 10 }])
    );
    let mut illegal = choice;
    illegal.resource = SpellResourceChoice::Slot { level: 1 };
    assert!(plan_spell_cast(&state, &meta, &illegal).is_err());
}

#[test]
fn save_damage_upcast_and_target_rules_remain_source_bound() {
    let (state, meta, choice) = fixture("burning-hands", SpellResourceChoice::Slot { level: 4 });
    let plan = plan_spell_cast(&state, &meta, &choice).unwrap();
    assert_eq!(
        plan.program.targets,
        SpellTargetRule::Area(SpellAreaShape::Cone { length_feet: 15 })
    );
    assert!(
        matches!(&plan.program.nodes[0], SpellProgramNode::SaveDamage { ability: Ability::Dexterity, damage, half_on_success: true, .. }
        if damage.dice == [DieSpec { count: 6, sides: 6 }])
    );
    let (state, meta, choice) = fixture("magic-missile", SpellResourceChoice::Slot { level: 4 });
    let plan = plan_spell_cast(&state, &meta, &choice).unwrap();
    assert_eq!(
        plan.program.targets,
        SpellTargetRule::Darts {
            count: 6,
            requires_sight: true
        }
    );
    assert!(matches!(
        &plan.program.nodes[0],
        SpellProgramNode::AutomaticDamage {
            share: SpellDamageShare::PerDart,
            ..
        }
    ));
}

#[test]
fn source_target_upcast_is_not_caller_supplied_count_or_radius() {
    let (state, meta, choice) = fixture("hold-person", SpellResourceChoice::Slot { level: 4 });
    let plan = plan_spell_cast(&state, &meta, &choice).unwrap();
    assert_eq!(
        plan.program.targets,
        SpellTargetRule::Creatures {
            maximum: 3,
            creature_type: Some("Humanoid".into()),
            requires_sight: true
        }
    );
    let (state, meta, choice) = fixture("fog-cloud", SpellResourceChoice::Slot { level: 3 });
    let plan = plan_spell_cast(&state, &meta, &choice).unwrap();
    assert_eq!(
        plan.program.targets,
        SpellTargetRule::Area(SpellAreaShape::Sphere { radius_feet: 60 })
    );
}

#[test]
fn cannot_invent_grant_payment_or_control() {
    let (mut state, meta, mut choice) =
        fixture("cure-wounds", SpellResourceChoice::Slot { level: 1 });
    let before = state.clone();
    choice.spell_id = "hold-person".into();
    assert!(plan_spell_cast(&state, &meta, &choice).is_err());
    choice.spell_id = "cure-wounds".into();
    choice.resource = SpellResourceChoice::Cantrip;
    assert!(plan_spell_cast(&state, &meta, &choice).is_err());
    choice.resource = SpellResourceChoice::Slot { level: 1 };
    let foreign = CommandMeta {
        issuer: CommandIssuer::Player(PlayerId::new()),
        actor: Some(AgentRef::Entity(choice.actor)),
        ..meta.clone()
    };
    assert!(plan_spell_cast(&state, &foreign, &choice).is_err());
    assert_eq!(state, before);
    state
        .rules
        .as_mut()
        .unwrap()
        .entities
        .get_mut(&choice.actor)
        .unwrap()
        .spellcasting
        .as_mut()
        .unwrap()
        .slots[0] = 0;
    assert!(plan_spell_cast(&state, &meta, &choice).is_err());
}

#[test]
fn counterspell_preserves_slot_obligation_but_not_original_action() {
    let (state, meta, choice) = fixture("cure-wounds", SpellResourceChoice::Slot { level: 1 });
    let plan = plan_spell_cast(&state, &meta, &choice).unwrap();
    let started = begin_cast(&plan, &context(&plan)).unwrap();
    assert_eq!(
        started.obligations,
        [SpellCastObligation::SpendCastingCost {
            actor: choice.actor,
            cost: SpellCastingCost::Action
        }]
    );
    let countered = step(&started.cast, &context(&plan), SpellCastAdvance::Countered);
    assert_eq!(countered.cast.phase, SpellCastPhase::Countered);
    assert!(countered.obligations.is_empty());
    assert!(
        advance_cast(
            &countered.cast,
            &next_meta(&countered.cast),
            &context(&plan),
            SpellCastAdvance::Commit
        )
        .is_err()
    );
}

#[test]
fn concentration_begins_before_interruption_and_countering_ends_only_its_group() {
    let (state, meta, choice) = fixture("hold-person", SpellResourceChoice::Slot { level: 2 });
    let plan = plan_spell_cast(&state, &meta, &choice).unwrap();
    let mut ctx = context(&plan);
    ctx.concentration = Some(EffectId::new());
    let started = begin_cast(&plan, &ctx).unwrap();
    assert!(
        matches!(&started.obligations[1], SpellCastObligation::BeginConcentration { group }
        if Some(group.id) == plan.concentration_group && group.stage == ConcentrationStage::Casting)
    );
    ctx.concentration = plan.concentration_group;
    let countered = step(&started.cast, &ctx, SpellCastAdvance::Countered);
    assert_eq!(
        countered.obligations,
        [SpellCastObligation::EndConcentration {
            actor: choice.actor,
            group: plan.concentration_group.unwrap()
        }]
    );
    ctx.concentration = Some(EffectId::new());
    let countered_after_loss = step(&started.cast, &ctx, SpellCastAdvance::Countered);
    assert!(countered_after_loss.obligations.is_empty());
    assert!(
        advance_cast(
            &started.cast,
            &next_meta(&started.cast),
            &ctx,
            SpellCastAdvance::Commit
        )
        .is_err()
    );
}

#[test]
fn ready_pays_once_holds_even_instant_magic_and_releases_only_reaction() {
    let (state, meta, mut choice) = fixture("cure-wounds", SpellResourceChoice::Slot { level: 1 });
    choice.mode = SpellCastMode::Ready {
        trigger: "When the ally reaches me".into(),
    };
    let plan = plan_spell_cast(&state, &meta, &choice).unwrap();
    let mut ctx = context(&plan);
    let started = begin_cast(&plan, &ctx).unwrap();
    let held = step(&started.cast, &ctx, SpellCastAdvance::Commit);
    assert_eq!(held.cast.phase, SpellCastPhase::Held);
    assert_eq!(
        held.obligations,
        [SpellCastObligation::CommitExpenditure {
            actor: choice.actor,
            expenditure: SpellExpenditure::Slot { level: 1 }
        }]
    );
    ctx.turn_number += 1;
    ctx.current_actor = EntityId::new();
    let released = step(&held.cast, &ctx, SpellCastAdvance::ReleaseReady);
    assert_eq!(released.cast.phase, SpellCastPhase::Released);
    assert_eq!(
        released
            .obligations
            .iter()
            .filter(|o| matches!(o, SpellCastObligation::CommitExpenditure { .. }))
            .count(),
        0
    );
    assert!(matches!(
        released.obligations[0],
        SpellCastObligation::SpendCastingCost {
            cost: SpellCastingCost::Reaction,
            ..
        }
    ));
    assert!(
        released
            .obligations
            .iter()
            .any(|o| matches!(o, SpellCastObligation::EndConcentration { .. }))
    );
    assert!(
        advance_cast(
            &released.cast,
            &next_meta(&released.cast),
            &ctx,
            SpellCastAdvance::ReleaseReady
        )
        .is_err()
    );
}

#[test]
fn held_concentration_spell_activates_same_group_and_source_duration() {
    let (state, meta, mut choice) = fixture("hold-person", SpellResourceChoice::Slot { level: 2 });
    choice.mode = SpellCastMode::Ready {
        trigger: "When the visible guard steps forward".into(),
    };
    let plan = plan_spell_cast(&state, &meta, &choice).unwrap();
    let ctx = context(&plan);
    let held = step(
        &begin_cast(&plan, &ctx).unwrap().cast,
        &ctx,
        SpellCastAdvance::Commit,
    );
    let released = step(&held.cast, &ctx, SpellCastAdvance::ReleaseReady);
    assert!(
        released
            .obligations
            .contains(&SpellCastObligation::ActivateConcentration {
                group: plan.concentration_group.unwrap(),
                duration: SpellDuration::Seconds { seconds: 60 }
            })
    );
    assert!(
        !released
            .obligations
            .iter()
            .any(|o| matches!(o, SpellCastObligation::BeginConcentration { .. }))
    );
}

#[test]
fn ready_expiry_and_concentration_loss_never_refund_payment_or_erase_replacement() {
    let (state, meta, mut choice) = fixture("cure-wounds", SpellResourceChoice::Slot { level: 1 });
    choice.mode = SpellCastMode::Ready {
        trigger: "When the ally arrives".into(),
    };
    let plan = plan_spell_cast(&state, &meta, &choice).unwrap();
    let mut ctx = context(&plan);
    let held = step(
        &begin_cast(&plan, &ctx).unwrap().cast,
        &ctx,
        SpellCastAdvance::Commit,
    );
    assert!(
        advance_cast(
            &held.cast,
            &next_meta(&held.cast),
            &ctx,
            SpellCastAdvance::ExpireHeld
        )
        .is_err()
    );
    ctx.turn_number += 3;
    assert!(
        advance_cast(
            &held.cast,
            &next_meta(&held.cast),
            &ctx,
            SpellCastAdvance::ReleaseReady
        )
        .is_err()
    );
    let expired = step(&held.cast, &ctx, SpellCastAdvance::ExpireHeld);
    assert_eq!(expired.cast.phase, SpellCastPhase::Expired);
    ctx.turn_number = 4;
    ctx.current_actor = EntityId::new();
    ctx.concentration = Some(EffectId::new());
    assert!(
        step(&held.cast, &ctx, SpellCastAdvance::ExpireHeld)
            .obligations
            .is_empty()
    );
}

#[test]
fn reaction_and_bonus_spells_cannot_be_readied() {
    for id in ["shield", "shield-of-faith"] {
        let (state, meta, mut choice) = fixture(id, SpellResourceChoice::Slot { level: 1 });
        choice.mode = SpellCastMode::Ready {
            trigger: "When someone arrives".into(),
        };
        assert!(plan_spell_cast(&state, &meta, &choice).is_err());
    }
}

#[test]
fn forged_program_payment_group_and_provenance_are_rejected_without_mutation() {
    let (state, meta, choice) = fixture("hold-person", SpellResourceChoice::Slot { level: 2 });
    let plan = plan_spell_cast(&state, &meta, &choice).unwrap();
    let original = begin_cast(&plan, &context(&plan)).unwrap().cast;
    for index in 0..5 {
        let mut forged = original.clone();
        match index {
            0 => forged.plan.program.save_dc = Some(99),
            1 => forged.plan.expenditure = SpellExpenditure::None,
            2 => forged.plan.concentration_group = Some(EffectId::new()),
            3 => forged.plan.program.ability_modifier = i16::MAX,
            _ => forged.plan.program.nodes.clear(),
        }
        assert!(validate_spell_cast(&forged).is_err());
    }
    let before = original.clone();
    let mut changed = next_meta(&original);
    changed.expected_event_sequence = original.last_operation.expected_event_sequence;
    assert!(
        advance_cast(
            &original,
            &changed,
            &context(&plan),
            SpellCastAdvance::Commit
        )
        .is_err()
    );
    assert_eq!(original, before);
}

#[test]
fn incomplete_casting_window_cannot_cross_turn_or_time_backwards() {
    let (state, meta, choice) = fixture("cure-wounds", SpellResourceChoice::Slot { level: 1 });
    let plan = plan_spell_cast(&state, &meta, &choice).unwrap();
    let mut ctx = context(&plan);
    let started = begin_cast(&plan, &ctx).unwrap();
    ctx.turn_number += 1;
    assert!(
        advance_cast(
            &started.cast,
            &next_meta(&started.cast),
            &ctx,
            SpellCastAdvance::Commit
        )
        .is_err()
    );
    ctx = context(&plan);
    ctx.now = WorldInstant(99);
    assert!(
        advance_cast(
            &started.cast,
            &next_meta(&started.cast),
            &ctx,
            SpellCastAdvance::Countered
        )
        .is_err()
    );
}

#[test]
fn material_and_somatic_access_use_identity_custody_and_hands() {
    let (mut state, meta, mut choice) =
        fixture("hold-person", SpellResourceChoice::Slot { level: 2 });
    let item = ItemId::new();
    choice.material = SpellMaterialChoice::Material { item };
    state.items.insert(
        item,
        ItemInstance {
            id: item,
            campaign_id: state.campaign_id(),
            definition_id: "source-material".into(),
            display_name: "Any display name".into(),
            quantity: 1,
            owner: Ownership::Unowned,
            custody: Custody::Entity(choice.actor),
            state: ItemState::Intact,
        },
    );
    let fact = SpellMaterialFact::Specified {
        item,
        spell_id: "hold-person".into(),
        value_cp: 0,
    };
    let plan = plan_spell_cast(&state, &meta, &choice).unwrap();
    assert_eq!(
        validate_spell_components(&state, &plan, true, Some(&fact)).unwrap(),
        None
    );
    assert!(validate_spell_components(&state, &plan, false, Some(&fact)).is_err());
    state.items.get_mut(&item).unwrap().custody = Custody::Missing;
    assert!(validate_spell_components(&state, &plan, true, Some(&fact)).is_err());
    state.items.get_mut(&item).unwrap().custody = Custody::Entity(choice.actor);
    state
        .rules
        .as_mut()
        .unwrap()
        .entities
        .get_mut(&choice.actor)
        .unwrap()
        .spellcasting
        .as_mut()
        .unwrap()
        .free_hand = false;
    assert!(validate_spell_components(&state, &plan, true, Some(&fact)).is_err());
}

#[test]
fn live_occupied_hands_override_legacy_free_hand_flag() {
    let (mut state, meta, choice) = fixture("cure-wounds", SpellResourceChoice::Slot { level: 1 });
    let plan = plan_spell_cast(&state, &meta, &choice).unwrap();
    state.rules.as_mut().unwrap().tactical_inventory = Some(TacticalInventory {
        loadouts: vec![ActorEquipmentLoadout {
            actor: choice.actor,
            hands: WeaponLoadout {
                hands: [
                    HandAssignment::Item(ItemId::new()),
                    HandAssignment::Item(ItemId::new()),
                ],
            },
            worn_armor: None,
            shield: None,
            command: meta,
        }],
        ..TacticalInventory::default()
    });
    assert!(validate_spell_components(&state, &plan, true, None).is_err());
}

fn add_flow(
    state: &mut CampaignState,
    meta: &CommandMeta,
    actor: EntityId,
    source: TacticalSource,
) {
    state.encounter = Some(TacticalEncounter {
        id: EncounterId::new(),
        scene_id: SceneId::new(),
        battlefield: Battlefield {
            bounds: SpatialBox {
                min: SpatialPoint { x: 0, y: 0, z: 0 },
                max: SpatialPoint {
                    x: 100,
                    y: 100,
                    z: 100,
                },
            },
            floor_z: 0,
            floor_surface: "ground".into(),
            ambient_light: LightLevel::Bright,
            terrain: vec![],
            obstacles: vec![],
            lights: vec![],
        },
        participants: vec![],
        knowledge: vec![],
        origin: meta.clone(),
        geometry_ruling: Ruling {
            basis: RulingBasis::Srd { page: 13 },
            reason: "Source timing fixture".into(),
        },
        flow: Some(TacticalFlow {
            version: 1,
            origin: meta.clone(),
            combatants: vec![TacticalCombatant {
                actor,
                source,
                surprised: false,
            }],
            initiative_groups: vec![],
            initiative_decisions: vec![],
            phase: TacticalPhase::Active,
            budget: TacticalTurnBudget::default(),
            resolution: None,
            dodges: vec![],
            save_decisions: vec![],
            ground_items: vec![],
        }),
    });
    state.rules.as_mut().unwrap().timing = Some(CombatTiming {
        order: vec![InitiativeEntry {
            actor,
            total: 10,
            tie_break: 0,
        }],
        index: 0,
        round: 1,
        turn_number: 3,
        action_spent: false,
        bonus_action_spent: false,
        slot_spent_this_turn: false,
        reactions_spent: vec![],
    });
}

#[test]
fn actual_cost_and_slot_reducers_keep_one_authority_and_reject_partial_payment() {
    let (mut state, meta, choice) = fixture("cure-wounds", SpellResourceChoice::Slot { level: 1 });
    add_flow(&mut state, &meta, choice.actor, TacticalSource::Character);
    let before = state.clone();
    let action = apply_spell_casting_cost(&state, choice.actor, SpellCastingCost::Action).unwrap();
    assert!(
        action
            .rules
            .as_ref()
            .unwrap()
            .timing
            .as_ref()
            .unwrap()
            .action_spent
    );
    assert_eq!(
        action.rules.as_ref().unwrap().entities[&choice.actor]
            .spellcasting
            .as_ref()
            .unwrap()
            .slots[0],
        4
    );
    let paid = apply_spell_expenditure(&action, choice.actor, &SpellExpenditure::Slot { level: 1 })
        .unwrap();
    assert_eq!(
        paid.rules.as_ref().unwrap().entities[&choice.actor]
            .spellcasting
            .as_ref()
            .unwrap()
            .slots[0],
        3
    );
    assert!(
        paid.rules
            .as_ref()
            .unwrap()
            .timing
            .as_ref()
            .unwrap()
            .slot_spent_this_turn
    );
    let paid_before = paid.clone();
    assert!(
        apply_spell_expenditure(&paid, choice.actor, &SpellExpenditure::Slot { level: 2 }).is_err()
    );
    assert_eq!(paid, paid_before);
    assert_eq!(state, before);
}

#[test]
fn source_creature_numbers_and_prepaid_activation_are_not_fabricated_pc_levels() {
    let (mut state, meta, mut choice) = fixture("hold-person", SpellResourceChoice::SourceFeature);
    add_flow(
        &mut state,
        &meta,
        choice.actor,
        TacticalSource::Creature {
            definition_id: "cultist-fanatic".into(),
        },
    );
    choice.grant = SpellGrantChoice::CreatureFeature {
        feature_id: "spellcasting".into(),
    };
    // The mechanical placeholder's level and ability must not override the source stat block.
    state
        .rules
        .as_mut()
        .unwrap()
        .entities
        .get_mut(&choice.actor)
        .unwrap()
        .level = 20;
    let plan = plan_spell_cast(&state, &meta, &choice).unwrap();
    assert!(plan.activation_prepaid);
    assert_eq!(plan.program.spell_level, 2);
    assert_eq!(plan.program.save_dc, Some(12));
    assert_eq!(plan.program.attack_bonus, Some(4));
    assert_eq!(plan.program.cantrip_character_level, None);
    assert_eq!(
        plan.expenditure,
        SpellExpenditure::CreatureUse {
            feature_id: "spellcasting".into(),
            spell_id: "hold-person".into(),
            maximum: 1
        }
    );
    let begun = begin_cast(&plan, &context(&plan)).unwrap();
    assert!(matches!(
        begun.obligations[0],
        SpellCastObligation::RequireCreatureFeatureReceipt { .. }
    ));
    assert!(
        !begun
            .obligations
            .iter()
            .any(|o| matches!(o, SpellCastObligation::SpendCastingCost { .. }))
    );
    let committed = step(&begun.cast, &context(&plan), SpellCastAdvance::Commit);
    assert!(
        !committed
            .obligations
            .iter()
            .any(|o| matches!(o, SpellCastObligation::CommitExpenditure { .. }))
    );
    assert!(apply_spell_expenditure(&state, choice.actor, &plan.expenditure).is_err());
}
