use super::*;
use std::collections::{BTreeMap, BTreeSet};

fn fixture() -> (MechanicalEntity, TacticalRecovery, VitalityContext) {
    let entity = MechanicalEntity {
        entity_id: EntityId::new(),
        character_features: None,
        level: 1,
        ability_scores: [10; 6],
        armor: ArmorClass::Fixed(10),
        max_hp: 12,
        hp: 12,
        temporary_hp: 0,
        hit_dice: HitDice {
            sides: 10,
            maximum: 1,
            remaining: 1,
        },
        death: DeathState::default(),
        uses_death_saves: true,
        exhaustion: 0,
        heroic_inspiration: false,
        prone: false,
        saving_proficiencies: BTreeSet::new(),
        skill_proficiencies: BTreeMap::new(),
        attacks: BTreeSet::new(),
        attack_proficiencies: BTreeSet::new(),
        prepared_spells: BTreeSet::new(),
        spellcasting: None,
        resources: BTreeMap::new(),
        resistances: BTreeSet::new(),
        vulnerabilities: BTreeSet::new(),
        damage_immunities: BTreeSet::new(),
        condition_immunities: BTreeSet::new(),
        concentration: None,
        last_long_rest_finished: None,
    };
    let context = VitalityContext {
        origin: VitalityOrigin {
            command: CommandMeta {
                id: CommandId::new(),
                campaign_id: CampaignId::new(),
                session_id: None,
                issuer: CommandIssuer::System,
                actor: None,
                expected_event_sequence: 7,
            },
            occurrence: 0,
        },
        now: WorldInstant(100),
        conditions: HashSet::new(),
        underwater: false,
        defenses: VitalityDefenses::default(),
        death_save_mode: RollMode::Normal,
        death_save_bonus: 0,
    };
    (entity, TacticalRecovery::default(), context)
}
fn packet(amount: u32) -> DamagePacket {
    DamagePacket {
        cause: DamageCause::Other,
        components: vec![DamageComponent {
            damage_type: DamageType::Fire,
            amounts: vec![amount],
            adjustments: vec![],
        }],
    }
}
fn damage(amount: u32) -> VitalityOperation {
    VitalityOperation::Damage {
        packet: packet(amount),
        knockout: None,
    }
}
fn melee(amount: u32, knockout: Option<KnockoutChoice>) -> VitalityOperation {
    let mut packet = packet(amount);
    packet.cause = DamageCause::Attack {
        attacker: EntityId::new(),
        melee: true,
        critical: false,
    };
    VitalityOperation::Damage { packet, knockout }
}
fn raw(id: RollRequestId, sides: u16, value: u16) -> RollResult {
    RollResult {
        request_id: id,
        source: RollSource::Physical,
        dice: vec![DieResult { sides, value }],
    }
}
fn zero(entity: &mut MechanicalEntity) {
    entity.hp = 0;
    entity.prone = true;
}
fn death_roll(
    entity: &MechanicalEntity,
    recovery: &TacticalRecovery,
    context: &VitalityContext,
    value: u16,
) -> VitalityTransition {
    let id = RollRequestId::new();
    reduce_vitality(
        entity,
        recovery,
        context,
        &VitalityOperation::DeathSave {
            request_id: id,
            result: raw(id, 20, value),
        },
    )
    .unwrap()
}
fn knock_out() -> (VitalityTransition, VitalityContext) {
    let (entity, recovery, context) = fixture();
    let next = reduce_vitality(
        &entity,
        &recovery,
        &context,
        &melee(100, Some(KnockoutChoice::KnockOut)),
    )
    .unwrap();
    (next, context)
}

#[test]
fn source_damage_adjustments_resistance_then_vulnerability_round_once_per_type() {
    let (mut entity, _, context) = fixture();
    entity.resistances.insert(DamageType::Fire);
    entity.vulnerabilities.insert(DamageType::Fire);
    let mut input = packet(28);
    input.components[0]
        .adjustments
        .push(DamageAdjustment::Add(-5));
    assert_eq!(damage_taken(&entity, &context, &input).unwrap(), 22); // SRD17 worked example.
    input.components[0].amounts = vec![1, 2];
    input.components[0].adjustments.clear();
    assert_eq!(damage_taken(&entity, &context, &input).unwrap(), 2); // (1+2)/2*2, not per die.
    input.components[0]
        .adjustments
        .push(DamageAdjustment::Multiply {
            numerator: 1,
            denominator: 2,
        });
    assert_eq!(damage_taken(&entity, &context, &input).unwrap(), 0); // Save half, then resistance.
}

#[test]
fn no_stacking_petrification_underwater_and_immunity() {
    let (mut entity, _, mut context) = fixture();
    entity.resistances.insert(DamageType::Fire);
    context.underwater = true;
    context.conditions.insert(Condition::Petrified);
    assert_eq!(damage_taken(&entity, &context, &packet(9)).unwrap(), 4);
    entity.vulnerabilities.insert(DamageType::Fire);
    entity.damage_immunities.insert(DamageType::Fire);
    assert_eq!(damage_taken(&entity, &context, &packet(9)).unwrap(), 0);
}

#[test]
fn typed_components_round_separately_but_form_one_damage_occurrence() {
    let (mut entity, recovery, context) = fixture();
    entity
        .resistances
        .extend([DamageType::Fire, DamageType::Cold]);
    entity.concentration = Some(EffectId::new());
    let mut input = packet(3);
    input.components.push(DamageComponent {
        damage_type: DamageType::Cold,
        amounts: vec![3],
        adjustments: vec![],
    });
    let next = reduce_vitality(
        &entity,
        &recovery,
        &context,
        &VitalityOperation::Damage {
            packet: input,
            knockout: None,
        },
    )
    .unwrap();
    assert_eq!(next.outcome.damage_taken, 2);
    assert_eq!(next.entity.hp, 10);
    assert_eq!(
        next.followups
            .iter()
            .filter(|f| matches!(f, VitalityFollowup::ConcentrationSave { .. }))
            .count(),
        1
    );
}

#[test]
fn temp_hp_absorption_still_requires_concentration_and_interrupts_rest() {
    let (mut entity, recovery, context) = fixture();
    let group = EffectId::new();
    entity.concentration = Some(group);
    entity.temporary_hp = 50;
    let next = reduce_vitality(&entity, &recovery, &context, &damage(31)).unwrap();
    assert_eq!((next.entity.hp, next.entity.temporary_hp), (12, 19));
    assert_eq!(
        (
            next.outcome.damage_taken,
            next.outcome.hp_lost,
            next.outcome.temporary_hp_lost
        ),
        (31, 0, 31)
    );
    assert!(
        next.followups
            .contains(&VitalityFollowup::ConcentrationSave {
                group,
                damage_taken: 31,
                dc: 15
            })
    );
    assert!(next.followups.contains(&VitalityFollowup::InterruptRest));
    assert_eq!(entity.temporary_hp, 50);
}

#[test]
fn concentration_has_per_occurrence_identity_floor_and_cap() {
    let (mut entity, recovery, mut context) = fixture();
    entity.max_hp = 1000;
    entity.hp = 1000;
    let group = EffectId::new();
    entity.concentration = Some(group);
    for (occurrence, amount, dc) in [
        (0, 1, 10),
        (1, 21, 10),
        (2, 59, 29),
        (3, 61, 30),
        (4, 900, 30),
    ] {
        context.origin.occurrence = occurrence;
        let next = reduce_vitality(&entity, &recovery, &context, &damage(amount)).unwrap();
        assert_eq!(next.origin.occurrence, occurrence);
        assert!(
            next.followups
                .contains(&VitalityFollowup::ConcentrationSave {
                    group,
                    damage_taken: amount,
                    dc
                })
        );
    }
}

#[test]
fn zero_or_immune_damage_does_not_interrupt_or_request_concentration() {
    let (mut entity, recovery, context) = fixture();
    entity.concentration = Some(EffectId::new());
    for immune in [false, true] {
        if immune {
            entity.damage_immunities.insert(DamageType::Fire);
        }
        let next = reduce_vitality(
            &entity,
            &recovery,
            &context,
            &damage(if immune { 100 } else { 0 }),
        )
        .unwrap();
        assert_eq!(next.entity, entity);
        assert!(next.followups.is_empty());
    }
}

#[test]
fn massive_damage_uses_remainder_after_temp_hp_and_current_hp() {
    let (mut entity, recovery, context) = fixture();
    entity.hp = 6;
    entity.temporary_hp = 5;
    let living = reduce_vitality(&entity, &recovery, &context, &damage(22)).unwrap();
    assert_eq!(living.entity.hp, 0);
    assert!(!living.entity.death.dead);
    let dead = reduce_vitality(&entity, &recovery, &context, &damage(23)).unwrap();
    assert!(dead.entity.death.dead);
    assert!(dead.outcome.died);
    assert_eq!(dead.outcome.hp_lost, 6);
}

#[test]
fn monsters_die_at_zero_unless_their_individual_death_save_policy_allows_it() {
    let (mut entity, recovery, context) = fixture();
    entity.uses_death_saves = false;
    assert!(
        reduce_vitality(&entity, &recovery, &context, &damage(12))
            .unwrap()
            .entity
            .death
            .dead
    );
    entity.uses_death_saves = true;
    assert!(
        !reduce_vitality(&entity, &recovery, &context, &damage(12))
            .unwrap()
            .entity
            .death
            .dead
    );
}

#[test]
fn damage_at_zero_counts_critical_failures_even_when_temp_hp_absorbs_it() {
    let (mut entity, recovery, context) = fixture();
    zero(&mut entity);
    entity.temporary_hp = 100;
    let mut input = packet(1);
    input.cause = DamageCause::Attack {
        attacker: EntityId::new(),
        melee: false,
        critical: true,
    };
    let next = reduce_vitality(
        &entity,
        &recovery,
        &context,
        &VitalityOperation::Damage {
            packet: input,
            knockout: None,
        },
    )
    .unwrap();
    assert_eq!(next.entity.death.failures, 2);
    assert_eq!(next.entity.temporary_hp, 99);
    let next = reduce_vitality(&next.entity, &next.recovery, &context, &damage(1)).unwrap();
    assert!(next.entity.death.dead);
    assert!(
        reduce_vitality(&entity, &recovery, &context, &damage(12))
            .unwrap()
            .entity
            .death
            .dead
    );
}

#[test]
fn critical_hit_is_not_a_generic_damage_multiplier() {
    let (entity, recovery, context) = fixture();
    let mut input = packet(1); // A blowgun's source fixed1 remains1 on a critical hit.
    input.cause = DamageCause::Attack {
        attacker: EntityId::new(),
        melee: false,
        critical: true,
    };
    let next = reduce_vitality(
        &entity,
        &recovery,
        &context,
        &VitalityOperation::Damage {
            packet: input,
            knockout: None,
        },
    )
    .unwrap();
    assert_eq!(next.outcome.damage_taken, 1);
    assert_eq!(next.entity.hp, 11);
}

#[test]
fn graze_miss_damage_cannot_gain_ordinary_damage_bonuses_or_vulnerability() {
    let (mut entity, recovery, mut context) = fixture();
    entity.vulnerabilities.insert(DamageType::Fire);
    let mut input = packet(3);
    input.cause = DamageCause::Graze {
        attacker: EntityId::new(),
        ability_modifier: 3,
    };
    assert_eq!(damage_taken(&entity, &context, &input).unwrap(), 3);
    context.defenses.resistances.insert(DamageType::Fire);
    assert_eq!(damage_taken(&entity, &context, &input).unwrap(), 1);
    input.components[0]
        .adjustments
        .push(DamageAdjustment::Add(1));
    assert!(damage_taken(&entity, &context, &input).is_err());
    input.components[0].adjustments = vec![DamageAdjustment::Multiply {
        numerator: 2,
        denominator: 1,
    }];
    assert!(damage_taken(&entity, &context, &input).is_err());
    input.components[0].adjustments.clear();
    let mut dying = entity.clone();
    zero(&mut dying);
    let next = reduce_vitality(
        &dying,
        &recovery,
        &context,
        &VitalityOperation::Damage {
            packet: input,
            knockout: None,
        },
    )
    .unwrap();
    assert_eq!(next.entity.death.failures, 1); // Graze misses, hence cannot deliver a critical hit.
}

#[test]
fn eligible_knockout_waits_for_attacker_choice_without_changing_hp() {
    let (entity, recovery, context) = fixture();
    let next = reduce_vitality(&entity, &recovery, &context, &melee(100, None)).unwrap();
    assert!(next.awaiting_choice);
    assert_eq!(next.entity, entity);
    assert_eq!(next.recovery, recovery);
    assert!(matches!(
        next.followups.as_slice(),
        [VitalityFollowup::ChooseKnockout { .. }]
    ));
    let normal = reduce_vitality(
        &entity,
        &recovery,
        &context,
        &melee(100, Some(KnockoutChoice::NormalDamage)),
    )
    .unwrap();
    assert!(normal.entity.death.dead);
}

#[test]
fn knockout_is_one_hp_and_ends_only_its_owned_condition_after_full_rest() {
    let (next, mut context) = knock_out();
    assert_eq!(next.entity.hp, 1);
    assert!(next.entity.prone);
    assert!(!next.entity.death.dead);
    assert!(recovery_conditions(&next.entity, &next.recovery).contains(&Condition::Unconscious));
    context.now = WorldInstant(3699);
    assert!(
        reduce_vitality(
            &next.entity,
            &next.recovery,
            &context,
            &VitalityOperation::CompleteShortRest
        )
        .is_err()
    );
    context.now = WorldInstant(3700);
    context.conditions.insert(Condition::Unconscious); // A separate spell's condition is untouched.
    let awake = reduce_vitality(
        &next.entity,
        &next.recovery,
        &context,
        &VitalityOperation::CompleteShortRest,
    )
    .unwrap();
    assert!(awake.recovery.knockout.is_none());
    assert_eq!(awake.entity.hp, 1);
    assert!(awake.entity.prone);
    assert!(context.conditions.contains(&Condition::Unconscious));
}

#[test]
fn damage_interrupts_knockout_rest_but_does_not_clear_unconsciousness() {
    let (mut next, mut context) = knock_out();
    next.entity.temporary_hp = 10;
    context.now = WorldInstant(200);
    let interrupted = reduce_vitality(&next.entity, &next.recovery, &context, &damage(1)).unwrap();
    assert_eq!(interrupted.entity.hp, 1);
    assert_eq!(
        interrupted
            .recovery
            .knockout
            .as_ref()
            .unwrap()
            .short_rest_started_at,
        None
    );
    context.now = WorldInstant(4000);
    assert!(
        reduce_vitality(
            &interrupted.entity,
            &interrupted.recovery,
            &context,
            &VitalityOperation::CompleteShortRest
        )
        .is_err()
    );
    let restarted = reduce_vitality(
        &interrupted.entity,
        &interrupted.recovery,
        &context,
        &VitalityOperation::StartKnockoutRest,
    )
    .unwrap();
    assert_eq!(
        restarted.recovery.knockout.unwrap().short_rest_started_at,
        Some(WorldInstant(4000))
    );
}

#[test]
fn healing_requires_actual_hp_regained_and_preserves_prone_and_unrelated_causes() {
    let (next, mut context) = knock_out();
    context.conditions.insert(Condition::Unconscious);
    let unchanged = reduce_vitality(
        &next.entity,
        &next.recovery,
        &context,
        &VitalityOperation::Heal { amount: 0 },
    )
    .unwrap();
    assert!(unchanged.recovery.knockout.is_some());
    let healed = reduce_vitality(
        &next.entity,
        &next.recovery,
        &context,
        &VitalityOperation::Heal { amount: 100 },
    )
    .unwrap();
    assert_eq!(healed.outcome.hp_regained, 11);
    assert_eq!(healed.entity.hp, 12);
    assert!(healed.recovery.knockout.is_none());
    assert!(healed.entity.prone);
    assert!(context.conditions.contains(&Condition::Unconscious));
    let mut maximum_one = next.entity.clone();
    maximum_one.max_hp = 1;
    let overheal = reduce_vitality(
        &maximum_one,
        &next.recovery,
        &context,
        &VitalityOperation::Heal { amount: 20 },
    )
    .unwrap();
    assert_eq!(overheal.outcome.hp_regained, 0);
    assert!(overheal.recovery.knockout.is_some());
}

#[test]
fn first_aid_does_not_silently_double_as_stabilization() {
    let (next, context) = knock_out();
    let zero = reduce_vitality(&next.entity, &next.recovery, &context, &damage(1)).unwrap();
    assert_eq!(zero.entity.hp, 0);
    assert!(zero.recovery.knockout.is_some());
    let aided = reduce_vitality(
        &zero.entity,
        &zero.recovery,
        &context,
        &VitalityOperation::Medicine {
            purpose: MedicinePurpose::EndKnockout,
            total: 10,
        },
    )
    .unwrap();
    assert!(aided.recovery.knockout.is_none());
    assert!(!aided.entity.death.stable);
    let stable = reduce_vitality(
        &zero.entity,
        &zero.recovery,
        &context,
        &VitalityOperation::Medicine {
            purpose: MedicinePurpose::Stabilize,
            total: 10,
        },
    )
    .unwrap();
    assert!(stable.entity.death.stable);
    assert!(stable.recovery.knockout.is_some());
}

#[test]
fn invalid_knockout_cases_never_mutate_the_input() {
    let (entity, recovery, context) = fixture();
    let mut input = packet(12);
    for cause in [
        DamageCause::Other,
        DamageCause::Attack {
            attacker: EntityId::new(),
            melee: false,
            critical: true,
        },
    ] {
        input.cause = cause;
        assert!(
            reduce_vitality(
                &entity,
                &recovery,
                &context,
                &VitalityOperation::Damage {
                    packet: input.clone(),
                    knockout: Some(KnockoutChoice::KnockOut)
                }
            )
            .is_err()
        );
    }
    assert!(
        reduce_vitality(
            &entity,
            &recovery,
            &context,
            &melee(11, Some(KnockoutChoice::KnockOut))
        )
        .is_err()
    );
    let mut at_zero = entity.clone();
    zero(&mut at_zero);
    assert!(
        reduce_vitality(
            &at_zero,
            &recovery,
            &context,
            &melee(12, Some(KnockoutChoice::KnockOut))
        )
        .is_err()
    );
    assert_eq!(entity.hp, 12);
    assert_eq!(recovery, TacticalRecovery::default());
}

#[test]
fn unconscious_immunity_prevents_knockout_condition_but_not_the_one_hp_choice() {
    let (mut entity, recovery, context) = fixture();
    entity.condition_immunities.insert(Condition::Unconscious);
    let next = reduce_vitality(
        &entity,
        &recovery,
        &context,
        &melee(20, Some(KnockoutChoice::KnockOut)),
    )
    .unwrap();
    assert_eq!(next.entity.hp, 1);
    assert!(!next.entity.prone);
    assert!(next.recovery.knockout.is_none());
    assert!(!next.followups.contains(&VitalityFollowup::DropHeldItems));
    assert!(
        !next
            .followups
            .iter()
            .any(|f| matches!(f, VitalityFollowup::StartKnockoutShortRest { .. }))
    );
}

#[test]
fn later_immunity_suppresses_owned_conditions_without_erasing_their_recovery_cause() {
    let (mut next, context) = knock_out();
    let original = next.recovery.clone();
    next.entity
        .condition_immunities
        .insert(Condition::Unconscious);
    let group = EffectId::new();
    next.entity.concentration = Some(group);
    assert!(recovery_conditions(&next.entity, &next.recovery).is_empty());
    let suppressed = reduce_vitality(
        &next.entity,
        &next.recovery,
        &context,
        &VitalityOperation::ReconcileConditions,
    )
    .unwrap();
    assert!(suppressed.followups.is_empty());
    assert_eq!(suppressed.recovery, original);
    next.entity.condition_immunities.clear();
    let resumed = reduce_vitality(
        &next.entity,
        &next.recovery,
        &context,
        &VitalityOperation::ReconcileConditions,
    )
    .unwrap();
    assert!(
        resumed
            .followups
            .contains(&VitalityFollowup::EndConcentration { group })
    );
    assert_eq!(resumed.recovery, original);
    next.entity
        .condition_immunities
        .insert(Condition::Incapacitated);
    assert!(!recovery_conditions(&next.entity, &next.recovery).contains(&Condition::Incapacitated));
    let immune = reduce_vitality(
        &next.entity,
        &next.recovery,
        &context,
        &VitalityOperation::ReconcileConditions,
    )
    .unwrap();
    assert!(immune.followups.is_empty());
}

#[test]
fn temp_hp_replacement_is_explicit_and_never_heals_or_stacks() {
    let (mut entity, recovery, context) = fixture();
    zero(&mut entity);
    entity.temporary_hp = 12;
    let next = reduce_vitality(
        &entity,
        &recovery,
        &context,
        &VitalityOperation::TemporaryHitPoints {
            amount: 10,
            choice: None,
        },
    )
    .unwrap();
    assert!(next.awaiting_choice);
    assert_eq!(next.entity, entity);
    for (choice, expected) in [
        (TemporaryHpChoice::KeepExisting, 12),
        (TemporaryHpChoice::Replace, 10),
    ] {
        let next = reduce_vitality(
            &entity,
            &recovery,
            &context,
            &VitalityOperation::TemporaryHitPoints {
                amount: 10,
                choice: Some(choice),
            },
        )
        .unwrap();
        assert_eq!(next.entity.temporary_hp, expected);
        assert_eq!(next.entity.hp, 0);
        assert!(next.entity.prone);
    }
    let rest = reduce_vitality(
        &entity,
        &recovery,
        &context,
        &VitalityOperation::CompleteLongRest,
    )
    .unwrap();
    assert_eq!(rest.entity.temporary_hp, 0); // Parent alone authorizes whether a long rest actually completed.
}

#[test]
fn death_save_raw_faces_and_thresholds_obey_source_not_natural_extremes_house_rules() {
    let (mut entity, recovery, mut context) = fixture();
    zero(&mut entity);
    context.death_save_bonus = 100;
    let one = death_roll(&entity, &recovery, &context, 1);
    assert_eq!(one.entity.death.failures, 2);
    assert_eq!(one.outcome.death_save_succeeded, Some(false));
    context.death_save_bonus = -100;
    let twenty = death_roll(&entity, &recovery, &context, 20);
    assert_eq!(twenty.entity.hp, 1);
    assert_eq!(twenty.entity.death, DeathState::default());
    assert!(twenty.entity.prone);
    context.death_save_bonus = 0;
    entity.exhaustion = 1;
    assert_eq!(
        death_roll(&entity, &recovery, &context, 11)
            .entity
            .death
            .failures,
        1
    );
    assert_eq!(
        death_roll(&entity, &recovery, &context, 12)
            .entity
            .death
            .successes,
        1
    );
}

#[test]
fn third_success_stabilizes_resets_both_counts_and_requests_raw_d4() {
    let (mut entity, recovery, context) = fixture();
    zero(&mut entity);
    entity.death.successes = 2;
    entity.death.failures = 2;
    let next = death_roll(&entity, &recovery, &context, 10);
    assert_eq!(
        next.entity.death,
        DeathState {
            stable: true,
            ..DeathState::default()
        }
    );
    assert!(next.recovery.stable.is_some());
    assert!(matches!(
        next.followups.as_slice(),
        [VitalityFollowup::StableRecoveryRoll { .. }]
    ));
    assert!(death_save_request(&next.entity, &context, RollRequestId::new()).is_err());
    assert!(
        death_roll(&entity, &recovery, &context, 9)
            .entity
            .death
            .dead
    );
}

#[test]
fn stable_recovery_waits_for_raw_d4_and_elapsed_time_and_does_not_reuse_invalidated_timer() {
    let (mut entity, recovery, mut context) = fixture();
    zero(&mut entity);
    let stable =
        reduce_vitality(&entity, &recovery, &context, &VitalityOperation::Stabilize).unwrap();
    assert!(
        reduce_vitality(
            &stable.entity,
            &stable.recovery,
            &context,
            &VitalityOperation::RecoverStable
        )
        .is_err()
    );
    let id = RollRequestId::new();
    let timer = reduce_vitality(
        &stable.entity,
        &stable.recovery,
        &context,
        &VitalityOperation::StableRecoveryRoll {
            origin: context.origin.clone(),
            request_id: id,
            result: raw(id, 4, 2),
        },
    )
    .unwrap();
    context.now = WorldInstant(7299);
    assert!(
        reduce_vitality(
            &timer.entity,
            &timer.recovery,
            &context,
            &VitalityOperation::RecoverStable
        )
        .is_err()
    );
    context.now = WorldInstant(7300);
    let awake = reduce_vitality(
        &timer.entity,
        &timer.recovery,
        &context,
        &VitalityOperation::RecoverStable,
    )
    .unwrap();
    assert_eq!(awake.entity.hp, 1);
    assert!(awake.recovery.stable.is_none());
    assert!(awake.entity.prone);
    let hurt = reduce_vitality(&timer.entity, &timer.recovery, &context, &damage(1)).unwrap();
    assert!(!hurt.entity.death.stable);
    assert!(hurt.recovery.stable.is_none());
    assert_eq!(hurt.entity.death.failures, 1);
    assert!(
        reduce_vitality(
            &hurt.entity,
            &hurt.recovery,
            &context,
            &VitalityOperation::RecoverStable
        )
        .is_err()
    );
}

#[test]
fn death_and_incapacitation_end_concentration_without_an_irrelevant_save() {
    let (mut entity, recovery, mut context) = fixture();
    let group = EffectId::new();
    entity.concentration = Some(group);
    for condition in [
        Condition::Incapacitated,
        Condition::Stunned,
        Condition::Paralyzed,
        Condition::Petrified,
        Condition::Unconscious,
    ] {
        context.conditions = HashSet::from([condition]);
        let next = reduce_vitality(
            &entity,
            &recovery,
            &context,
            &VitalityOperation::ReconcileConditions,
        )
        .unwrap();
        assert_eq!(
            next.followups,
            vec![VitalityFollowup::EndConcentration { group }]
        );
    }
    context.conditions.clear();
    let next = reduce_vitality(&entity, &recovery, &context, &damage(12)).unwrap();
    assert!(
        next.followups
            .contains(&VitalityFollowup::EndConcentration { group })
    );
    assert!(
        !next
            .followups
            .iter()
            .any(|f| matches!(f, VitalityFollowup::ConcentrationSave { .. }))
    );
    assert_eq!(next.entity.concentration, Some(group)); // Parent must atomically remove the actual effect group.
}

#[test]
fn maximum_hp_zero_kills_and_ordinary_healing_cannot_revive() {
    let (entity, recovery, context) = fixture();
    let next = reduce_vitality(
        &entity,
        &recovery,
        &context,
        &VitalityOperation::SetMaximumHitPoints { maximum: 0 },
    )
    .unwrap();
    assert_eq!((next.entity.hp, next.entity.max_hp), (0, 0));
    assert!(next.entity.death.dead);
    assert!(
        reduce_vitality(
            &next.entity,
            &next.recovery,
            &context,
            &VitalityOperation::Heal { amount: 10 }
        )
        .is_err()
    );
    let increase = reduce_vitality(
        &entity,
        &recovery,
        &context,
        &VitalityOperation::SetMaximumHitPoints { maximum: 24 },
    )
    .unwrap();
    assert_eq!((increase.entity.hp, increase.entity.max_hp), (12, 24));
}

#[test]
fn invalid_damage_bounds_dice_and_forged_recovery_preserve_original_values() {
    let (entity, recovery, mut context) = fixture();
    let before = entity.clone();
    for adjustment in [
        DamageAdjustment::Multiply {
            numerator: 1,
            denominator: 0,
        },
        DamageAdjustment::Add(i32::MIN),
        DamageAdjustment::Multiply {
            numerator: 1001,
            denominator: 1,
        },
    ] {
        let mut input = packet(1);
        input.components[0].adjustments.push(adjustment);
        assert!(damage_taken(&entity, &context, &input).is_err());
    }
    let mut input = packet(1);
    input.components.push(input.components[0].clone());
    assert!(damage_taken(&entity, &context, &input).is_err());
    assert!(
        reduce_vitality(
            &entity,
            &recovery,
            &context,
            &damage(MAX_VITALITY_AMOUNT + 1)
        )
        .is_err()
    );
    let mut dying = entity.clone();
    zero(&mut dying);
    let id = RollRequestId::new();
    for result in [
        raw(id, 20, 21),
        raw(id, 6, 4),
        raw(RollRequestId::new(), 20, 12),
    ] {
        assert!(
            reduce_vitality(
                &dying,
                &recovery,
                &context,
                &VitalityOperation::DeathSave {
                    request_id: id,
                    result
                }
            )
            .is_err()
        );
    }
    let forged = TacticalRecovery {
        stable: Some(StableRecovery {
            origin: context.origin.clone(),
            stabilized_at: context.now,
            delay_roll: None,
        }),
        ..TacticalRecovery::default()
    };
    assert!(validate_recovery(&entity, &forged, &context).is_err());
    context.death_save_bonus = i32::MIN;
    assert!(
        reduce_vitality(
            &entity,
            &recovery,
            &context,
            &VitalityOperation::Heal { amount: 1 }
        )
        .is_err()
    );
    assert_eq!(entity, before);
    assert_eq!(recovery, TacticalRecovery::default());
}

#[test]
fn recovery_json_roundtrip_preserves_source_and_raw_faces_and_rejects_unknown_fields() {
    let (next, context) = knock_out();
    let json = serde_json::to_string(&next.recovery).unwrap();
    let restored: TacticalRecovery = serde_json::from_str(&json).unwrap();
    assert_eq!(restored, next.recovery);
    validate_recovery(&next.entity, &restored, &context).unwrap();
    let mut value = serde_json::to_value(restored).unwrap();
    value["invented_authority"] = true.into();
    assert!(serde_json::from_value::<TacticalRecovery>(value).is_err());
    let mut dying = next.entity;
    zero(&mut dying);
    let stable = reduce_vitality(
        &dying,
        &TacticalRecovery::default(),
        &context,
        &VitalityOperation::Stabilize,
    )
    .unwrap();
    let id = RollRequestId::new();
    let pending = VitalityOperation::StableRecoveryRoll {
        origin: context.origin.clone(),
        request_id: id,
        result: raw(id, 4, 3),
    };
    let recorded = reduce_vitality(&stable.entity, &stable.recovery, &context, &pending).unwrap();
    let bytes = serde_json::to_vec(&recorded.recovery).unwrap();
    let restored: TacticalRecovery = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(
        restored.stable.as_ref().unwrap().delay_roll,
        Some(raw(id, 4, 3))
    );
    assert_eq!(
        stable_wake_at(restored.stable.as_ref().unwrap()).unwrap(),
        Some(WorldInstant(10_900))
    );
    assert!(reduce_vitality(&recorded.entity, &restored, &context, &pending).is_err());
}

#[test]
fn forged_or_future_recovery_origin_is_rejected_without_replacing_the_record() {
    let (next, context) = knock_out();
    let original = next.recovery.clone();
    for forged_command in [
        CommandMeta {
            id: CommandId::new(),
            ..context.origin.command.clone()
        },
        CommandMeta {
            expected_event_sequence: 8,
            ..context.origin.command.clone()
        },
        CommandMeta {
            campaign_id: CampaignId::new(),
            ..context.origin.command.clone()
        },
    ] {
        let mut forged = original.clone();
        forged.knockout.as_mut().unwrap().origin.command = forged_command;
        assert!(validate_recovery(&next.entity, &forged, &context).is_err());
    }
    let mut future = original.clone();
    future.knockout.as_mut().unwrap().short_rest_started_at = Some(WorldInstant(101));
    assert!(validate_recovery(&next.entity, &future, &context).is_err());
    assert_eq!(next.recovery, original);
}

#[test]
fn recovery_time_overflow_or_wrong_occurrence_does_not_accept_a_raw_die() {
    let (mut entity, recovery, mut context) = fixture();
    zero(&mut entity);
    context.now = WorldInstant(i64::MAX - 100);
    let stable =
        reduce_vitality(&entity, &recovery, &context, &VitalityOperation::Stabilize).unwrap();
    let id = RollRequestId::new();
    let original = stable.recovery.clone();
    let operation = VitalityOperation::StableRecoveryRoll {
        origin: context.origin.clone(),
        request_id: id,
        result: raw(id, 4, 1),
    };
    assert!(reduce_vitality(&stable.entity, &stable.recovery, &context, &operation).is_err());
    let mut wrong_origin = context.origin.clone();
    wrong_origin.occurrence = 1;
    assert!(
        reduce_vitality(
            &stable.entity,
            &stable.recovery,
            &context,
            &VitalityOperation::StableRecoveryRoll {
                origin: wrong_origin,
                request_id: id,
                result: raw(id, 4, 1),
            }
        )
        .is_err()
    );
    assert_eq!(stable.recovery, original);
    assert!(stable.recovery.stable.unwrap().delay_roll.is_none());
}
