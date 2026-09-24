use dmd_domain::*;
use dmd_rules::tactical_conditions::*;
use std::collections::HashMap;

fn fixture() -> (RulesState, EntityId, EntityId, EntityId) {
    let a = EntityId::new();
    let b = EntityId::new();
    let c = EntityId::new();
    (
        RulesState {
            pack_id: "srd-5.2".into(),
            pack_version: "5.2.1".into(),
            entities: HashMap::from([
                (a, MechanicalEntity::basic(a)),
                (b, MechanicalEntity::basic(b)),
                (c, MechanicalEntity::basic(c)),
            ]),
            house_rules: HouseRules::default(),
            effects: vec![],
            tactical_effects: None,
            tactical_inventory: None,
            pending: None,
            rolls: vec![],
            cancelled_roll_ids: vec![],
            rulings: vec![],
            timing: None,
            rests: vec![],
            completed_short_rests: vec![],
            permission: None,
        },
        a,
        b,
        c,
    )
}
fn effect(rules: &mut RulesState, source: EntityId, target: EntityId, condition: Condition) {
    rules.effects.push(ActiveEffect {
        id: EffectId::new(),
        source,
        target,
        condition: Some(condition),
        label: "Source condition".into(),
        expires: Expiry::Never,
        concentration_owner: None,
    });
}
fn seen() -> AttackPerception {
    AttackPerception {
        attacker_sees_target: true,
        target_sees_attacker: true,
        fear_source_in_sight: false,
        within_five_feet: true,
        hostile_ranged_threat: false,
    }
}
fn attack(
    rules: &RulesState,
    a: EntityId,
    b: EntityId,
    senses: AttackPerception,
) -> AttackConditions {
    attack_conditions(
        rules,
        a,
        b,
        false,
        senses,
        Circumstances::default(),
        DodgeContext::default(),
    )
    .unwrap()
}

#[test]
fn special_perception_defeats_invisibility_without_creating_global_visibility() {
    let (mut rules, a, b, _) = fixture();
    effect(&mut rules, a, a, Condition::Invisible);
    effect(&mut rules, b, b, Condition::Blinded);
    // The blinded defender has Blindsight within this particular pair's range.
    assert_eq!(attack(&rules, a, b, seen()).mode, RollMode::Normal);
    let mut outside = seen();
    outside.target_sees_attacker = false;
    assert_eq!(attack(&rules, a, b, outside).mode, RollMode::Advantage);
}

#[test]
fn mutual_unseen_and_other_disadvantages_cancel_once() {
    let (mut rules, a, b, _) = fixture();
    effect(&mut rules, b, a, Condition::Poisoned);
    let mut senses = seen();
    senses.attacker_sees_target = false;
    senses.target_sees_attacker = false;
    assert_eq!(attack(&rules, a, b, senses).mode, RollMode::Normal);
}

#[test]
fn grapple_penalty_tracks_actual_grapplers_and_charm_blocks_harm_only_to_charmer() {
    let (mut rules, a, b, c) = fixture();
    effect(&mut rules, b, a, Condition::Grappled);
    assert_eq!(attack(&rules, a, b, seen()).mode, RollMode::Normal);
    assert_eq!(attack(&rules, a, c, seen()).mode, RollMode::Disadvantage);
    effect(&mut rules, c, a, Condition::Charmed);
    assert!(!may_harm(&rules, a, c));
    assert!(may_harm(&rules, a, b));
    assert!(
        attack_conditions(
            &rules,
            a,
            c,
            false,
            seen(),
            Circumstances::default(),
            DodgeContext::default()
        )
        .is_err()
    );
}

#[test]
fn prone_distance_and_paralysis_critical_are_independent_of_attack_type() {
    let (mut rules, a, b, _) = fixture();
    effect(&mut rules, a, b, Condition::Paralyzed);
    effect(&mut rules, a, b, Condition::Prone);
    let near = attack_conditions(
        &rules,
        a,
        b,
        true,
        seen(),
        Circumstances::default(),
        DodgeContext::default(),
    )
    .unwrap();
    assert!(near.critical_on_hit);
    assert_eq!(near.mode, RollMode::Advantage);
    let mut far = seen();
    far.within_five_feet = false;
    let far = attack(&rules, a, b, far);
    assert!(!far.critical_on_hit);
    assert_eq!(far.mode, RollMode::Normal);
}

#[test]
fn dodge_loses_benefits_at_zero_speed_and_requires_sight_only_for_attacks() {
    let (rules, a, b, _) = fixture();
    let mut senses = seen();
    senses.target_sees_attacker = false;
    let dodge = DodgeContext {
        declared: true,
        effective_speed_units: 60,
    };
    assert_eq!(
        attack_conditions(&rules, a, b, false, senses, Circumstances::default(), dodge)
            .unwrap()
            .mode,
        RollMode::Advantage
    );
    assert_eq!(
        save_conditions(
            &rules,
            b,
            Ability::Dexterity,
            Circumstances::default(),
            dodge
        )
        .unwrap(),
        TestDisposition::Roll(RollMode::Advantage)
    );
    let zero = DodgeContext {
        effective_speed_units: 0,
        ..dodge
    };
    assert_eq!(
        save_conditions(
            &rules,
            b,
            Ability::Dexterity,
            Circumstances::default(),
            zero
        )
        .unwrap(),
        TestDisposition::Roll(RollMode::Normal)
    );
}

#[test]
fn revised_stunned_does_not_zero_speed_but_paralysis_does() {
    let (mut rules, a, b, _) = fixture();
    effect(&mut rules, b, a, Condition::Stunned);
    assert!(!can_act(&rules, a).unwrap());
    assert!(!can_speak(&rules, a).unwrap());
    assert_eq!(effective_speed(&rules, a, 60).unwrap(), 60);
    assert_eq!(
        save_conditions(
            &rules,
            a,
            Ability::Dexterity,
            Circumstances::default(),
            DodgeContext::default()
        )
        .unwrap(),
        TestDisposition::AutomaticFailure
    );
    effect(&mut rules, b, a, Condition::Paralyzed);
    assert_eq!(effective_speed(&rules, a, 60).unwrap(), 0);
}

#[test]
fn sensory_failure_is_not_an_attack_or_generic_disadvantage_and_deafness_is_not_muteness() {
    let (mut rules, a, b, _) = fixture();
    effect(&mut rules, b, a, Condition::Deafened);
    assert!(can_speak(&rules, a).unwrap());
    assert_eq!(
        check_conditions(&rules, a, false, true, false, Circumstances::default()).unwrap(),
        TestDisposition::AutomaticFailure
    );
    assert_eq!(
        check_conditions(&rules, a, false, false, false, Circumstances::default()).unwrap(),
        TestDisposition::Roll(RollMode::Normal)
    );
    assert_eq!(
        check_conditions(
            &rules,
            a,
            true,
            false,
            false,
            Circumstances {
                advantage: true,
                ..Default::default()
            }
        )
        .unwrap(),
        TestDisposition::AutomaticFailure
    );
}

#[test]
fn fear_applies_only_when_its_source_is_in_sight_and_exhaustion_affects_every_speed() {
    let (mut rules, a, b, _) = fixture();
    effect(&mut rules, b, a, Condition::Frightened);
    assert_eq!(attack(&rules, a, b, seen()).mode, RollMode::Normal);
    let mut scared = seen();
    scared.fear_source_in_sight = true;
    assert_eq!(attack(&rules, a, b, scared).mode, RollMode::Disadvantage);
    rules.entities.get_mut(&a).unwrap().exhaustion = 3;
    assert_eq!(effective_speed(&rules, a, 60).unwrap(), 30);
    assert_eq!(effective_speed(&rules, a, 20).unwrap(), 0);
}
