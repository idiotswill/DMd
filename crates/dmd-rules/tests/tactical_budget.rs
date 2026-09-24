use dmd_domain::*;
use dmd_rules::tactical_budget::*;

fn timing(a: EntityId, b: EntityId) -> CombatTiming {
    CombatTiming {
        order: vec![
            InitiativeEntry {
                actor: a,
                total: 15,
                tie_break: 0,
            },
            InitiativeEntry {
                actor: b,
                total: 10,
                tie_break: 0,
            },
        ],
        index: 0,
        round: 1,
        turn_number: 1,
        action_spent: false,
        bonus_action_spent: false,
        slot_spent_this_turn: false,
        reactions_spent: vec![],
    }
}

#[test]
fn switching_speeds_subtracts_all_previous_expenditure_and_dash_tracks_current_speed() {
    let mut budget = TacticalTurnBudget::default();
    let speeds = MovementProfile {
        walk: 60,
        fly: Some(120),
        climb: None,
        swim: None,
        burrow: None,
        hover: false,
    };
    spend_movement(&mut budget, 40, DashSpeed::Speed, &speeds).unwrap();
    assert_eq!(
        movement_remaining(&budget, DashSpeed::Fly, &speeds).unwrap(),
        80
    );
    let origin = CommandId::new();
    grant_dash(&mut budget, DashSpeed::Speed, origin, &speeds).unwrap();
    assert_eq!(
        movement_remaining(&budget, DashSpeed::Speed, &speeds).unwrap(),
        80
    );
    assert_eq!(
        movement_remaining(&budget, DashSpeed::Fly, &speeds).unwrap(),
        80
    );
    assert!(grant_dash(&mut budget, DashSpeed::Speed, origin, &speeds).is_err());
    let slower = MovementProfile {
        walk: 20,
        fly: Some(40),
        ..speeds.clone()
    };
    assert_eq!(
        movement_remaining(&budget, DashSpeed::Speed, &slower).unwrap(),
        0
    );
    let before = budget.clone();
    assert!(spend_movement(&mut budget, 1, DashSpeed::Speed, &slower).is_err());
    assert_eq!(budget, before);
    let mut flying_dash = TacticalTurnBudget::default();
    grant_dash(&mut flying_dash, DashSpeed::Fly, CommandId::new(), &speeds).unwrap();
    assert_eq!(
        movement_remaining(&flying_dash, DashSpeed::Speed, &speeds).unwrap(),
        60
    );
    assert_eq!(
        movement_remaining(&flying_dash, DashSpeed::Fly, &speeds).unwrap(),
        240
    );
    assert_eq!(
        movement_remaining(&flying_dash, DashSpeed::Fly, &slower).unwrap(),
        80
    );
    assert!(grant_dash(&mut flying_dash, DashSpeed::Swim, CommandId::new(), &speeds).is_err());
}

fn rules(a: EntityId, b: EntityId) -> RulesState {
    RulesState {
        pack_id: "srd-5.2".into(),
        pack_version: "5.2.1".into(),
        entities: std::collections::HashMap::from([
            (a, MechanicalEntity::basic(a)),
            (b, MechanicalEntity::basic(b)),
        ]),
        house_rules: HouseRules::default(),
        effects: vec![],
        pending: None,
        rolls: vec![],
        cancelled_roll_ids: vec![],
        rulings: vec![],
        timing: Some(timing(a, b)),
        rests: vec![],
        completed_short_rests: vec![],
        permission: None,
    }
}

#[test]
fn wrong_turn_absent_dead_and_incapacitated_actions_leave_budgets_unchanged() {
    let a = EntityId::new();
    let b = EntityId::new();
    let mut rules = rules(a, b);
    for actor in [b, EntityId::new()] {
        let before = rules.clone();
        assert!(spend_cost(&mut rules, actor, TacticalCost::Action).is_err());
        assert_eq!(rules, before);
    }
    rules.entities.get_mut(&a).unwrap().death.dead = true;
    let before = rules.clone();
    assert!(spend_cost(&mut rules, a, TacticalCost::Action).is_err());
    assert_eq!(rules, before);
    rules.entities.get_mut(&a).unwrap().death.dead = false;
    rules.effects.push(ActiveEffect {
        id: EffectId::new(),
        source: b,
        target: a,
        condition: Some(Condition::Incapacitated),
        label: "Source effect".into(),
        expires: Expiry::Never,
        concentration_owner: None,
    });
    for cost in [
        TacticalCost::Action,
        TacticalCost::BonusAction,
        TacticalCost::Reaction,
    ] {
        let before = rules.clone();
        assert!(spend_cost(&mut rules, a, cost).is_err());
        assert_eq!(rules, before);
    }
}

#[test]
fn off_turn_reaction_spends_once_and_attack_action_preserves_separate_attack_count() {
    let a = EntityId::new();
    let b = EntityId::new();
    let mut rules = rules(a, b);
    spend_cost(&mut rules, b, TacticalCost::Reaction).unwrap();
    let before = rules.clone();
    assert!(spend_cost(&mut rules, b, TacticalCost::Reaction).is_err());
    assert_eq!(rules, before);
    let mut budget = TacticalTurnBudget::default();
    assert!(start_attack_action(&mut rules, &mut budget, a, 0).is_err());
    assert_eq!(rules, before);
    start_attack_action(&mut rules, &mut budget, a, 2).unwrap();
    assert!(rules.timing.as_ref().unwrap().action_spent);
    spend_attack(&mut budget).unwrap();
    let before = (rules.clone(), budget.clone());
    assert!(start_attack_action(&mut rules, &mut budget, a, 2).is_err());
    assert_eq!((rules.clone(), budget.clone()), before);
    spend_attack(&mut budget).unwrap();
    let before = budget.clone();
    assert!(spend_attack(&mut budget).is_err());
    assert_eq!(budget, before);
}

#[test]
fn round_down_uses_feet_and_zero_speed_does_not_allow_standing() {
    assert_eq!(half_speed_cost(70).unwrap(), 34); // 35 feet / 2 => 17 feet.
    assert_eq!(half_speed_cost(60).unwrap(), 30);
    assert!(half_speed_cost(0).is_err());
}

#[test]
fn off_turn_slot_accounting_is_per_caster_and_resets_on_each_turn() {
    let a = EntityId::new();
    let b = EntityId::new();
    let mut timing = timing(a, b);
    let mut budget = TacticalTurnBudget::default();
    spend_slot_turn(&mut timing, &mut budget, a).unwrap();
    spend_slot_turn(&mut timing, &mut budget, b).unwrap();
    assert!(spend_slot_turn(&mut timing, &mut budget, a).is_err());
    assert!(spend_slot_turn(&mut timing, &mut budget, b).is_err());
    assert_eq!(advance_turn(&mut timing, &mut budget).unwrap(), b);
    spend_slot_turn(&mut timing, &mut budget, b).unwrap();
    spend_slot_turn(&mut timing, &mut budget, a).unwrap();
}

#[test]
fn only_the_actor_starting_a_turn_recovers_its_reaction_and_overflow_changes_nothing() {
    let a = EntityId::new();
    let b = EntityId::new();
    let mut timing = timing(a, b);
    let mut budget = TacticalTurnBudget::default();
    timing.reactions_spent = vec![a, b];
    budget.movement_spent = 50;
    assert_eq!(advance_turn(&mut timing, &mut budget).unwrap(), b);
    assert_eq!(timing.reactions_spent, vec![a]);
    assert_eq!(budget.movement_spent, 0);
    timing.round = u32::MAX;
    let before = (timing.clone(), budget.clone());
    assert!(advance_turn(&mut timing, &mut budget).is_err());
    assert_eq!((timing, budget), before);
}
