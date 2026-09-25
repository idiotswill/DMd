use super::*;

fn fixture() -> Fixture {
    let mut f = Fixture::new();
    f.arm("greatsword", false, false);
    f.begin();
    f
}
fn strike(f: &Fixture) -> TacticalAction {
    TacticalAction::UnarmedStrike {
        target: f.actors[1],
    }
}

#[test]
fn owned_unarmed_damage_spends_an_attack_with_full_hands_and_no_damage_dice_or_weapon_receipt() {
    for (face, damage) in [(1, 0), (15, 4), (20, 4)] {
        let mut f = fixture();
        let loadout = f.loadout().clone();
        assert!(!loadout.hands.hands.contains(&HandAssignment::Free));
        let action = strike(&f);
        f.rejected(Some(1), action.clone());
        f.run(Some(0), action.clone());
        let request = &f.rules().pending.as_ref().unwrap().request;
        assert_eq!(request.roller, Some(f.actors[0]));
        assert_eq!(request.modifier, 5);
        assert_eq!(
            request.dice,
            vec![DieSpec {
                count: 1,
                sides: 20
            }]
        );
        assert!(f.rules().timing.as_ref().unwrap().action_spent);
        assert!(!f.rules().timing.as_ref().unwrap().bonus_action_spent);
        assert!(
            f.rules()
                .timing
                .as_ref()
                .unwrap()
                .reactions_spent
                .is_empty()
        );
        f.rejected(
            Some(1),
            TacticalAction::SubmitRoll {
                result: f.raw(&[face]),
            },
        );
        f.rejected(Some(0), TacticalAction::EndTurn);
        let raw = f.raw(&[face]);
        f.run(
            Some(0),
            TacticalAction::SubmitRoll {
                result: raw.clone(),
            },
        );
        assert_eq!(f.rules().entities[&f.actors[1]].hp, 50 - damage);
        assert_eq!(f.rules().rolls.last().unwrap().result, raw);
        assert!(f.rules().pending.is_none());
        assert!(f.flow().resolution.is_none());
        assert!(f.flow().budget.weapon_history.is_empty());
        assert_eq!(f.loadout(), &loadout);
        assert!(
            f.rules().entities[&f.actors[0]]
                .character_features
                .as_ref()
                .unwrap()
                .savage_attacker_turn
                .is_none()
        );
        f.rejected(Some(0), action);
        f.rejected(Some(0), TacticalAction::Dodge);
    }
}

#[test]
fn unarmed_attack_uses_an_already_open_attack_action_and_rejects_forged_cost_or_source() {
    let mut f = fixture();
    let start = f.run(Some(0), TacticalAction::StartAttackAction);
    f.run(Some(0), strike(&f));
    let attack = f
        .flow()
        .resolution
        .as_ref()
        .unwrap()
        .attack
        .as_ref()
        .unwrap();
    assert!(
        matches!(attack.admission, TacticalAttackAdmission::UnarmedAction { window } if window.id == start.meta.id)
    );
    assert_eq!(f.flow().budget.attacks_remaining, 0);
    for case in 0..5 {
        let mut forged = f.state.clone();
        let rules = forged.rules.as_mut().unwrap();
        let attack = forged
            .encounter
            .as_mut()
            .unwrap()
            .flow
            .as_mut()
            .unwrap()
            .resolution
            .as_mut()
            .unwrap()
            .attack
            .as_mut()
            .unwrap();
        match case {
            0 => rules.timing.as_mut().unwrap().action_spent = false,
            1 => attack.admission = TacticalAttackAdmission::OwnTurn,
            2 => attack.attack_modifier += 1,
            3 => {
                attack.source = TacticalAttackSource::Unarmed {
                    ability: Ability::Dexterity,
                }
            }
            _ => {
                attack.admission = TacticalAttackAdmission::UnarmedAction {
                    window: WeaponActionWindow {
                        id: CommandId::new(),
                        kind: WeaponActionKind::AttackAction,
                    },
                }
            }
        }
        assert!(validate_tactical_state(&forged).is_err(), "case {case}");
    }
    f.roll(0, &[15]);
    assert_eq!(f.rules().entities[&f.actors[1]].hp, 46);
}

#[test]
fn unarmed_admission_checks_knowledge_reach_ability_to_act_and_full_cover_before_cost() {
    for case in 0..5 {
        let mut f = fixture();
        match case {
            0 => {
                f.state.encounter.as_mut().unwrap().participants[1]
                    .position
                    .x = 40
            }
            1 => {
                f.state
                    .encounter
                    .as_mut()
                    .unwrap()
                    .battlefield
                    .ambient_light = LightLevel::Darkness
            }
            2 => f.state.rules.as_mut().unwrap().effects.push(ActiveEffect {
                id: EffectId::new(),
                source: f.actors[1],
                target: f.actors[0],
                condition: Some(Condition::Incapacitated),
                label: "Source incapacity".into(),
                expires: Expiry::Never,
                concentration_owner: None,
            }),
            3 => f.zero_hp_target(true),
            _ => f
                .state
                .encounter
                .as_mut()
                .unwrap()
                .battlefield
                .obstacles
                .push(SpatialObstacle {
                    id: "transparent-total-cover".into(),
                    volume: SpatialBox {
                        min: SpatialPoint { x: 19, y: 10, z: 0 },
                        max: SpatialPoint {
                            x: 21,
                            y: 20,
                            z: 20,
                        },
                    },
                    blocks_movement: true,
                    blocks_sight: false,
                    observable: true,
                    cover: CoverDegree::Total,
                }),
        }
        validate_state(&f.state, &f.pack).unwrap();
        validate_tactical_state(&f.state).unwrap();
        f.rejected(Some(0), strike(&f));
        assert!(!f.rules().timing.as_ref().unwrap().action_spent);
    }
    let f = fixture();
    f.rejected(
        Some(0),
        TacticalAction::UnarmedStrike {
            target: f.actors[0],
        },
    );
}

#[test]
fn unarmed_conditions_exhaustion_and_inspiration_preserve_actual_raw_faces() {
    let mut f = fixture();
    f.entity_mut(0).exhaustion = 1;
    f.entity_mut(0).heroic_inspiration = true;
    f.state.rules.as_mut().unwrap().effects.push(ActiveEffect {
        id: EffectId::new(),
        source: f.actors[1],
        target: f.actors[0],
        condition: Some(Condition::Poisoned),
        label: "Source poison".into(),
        expires: Expiry::Never,
        concentration_owner: None,
    });
    f.run(Some(0), strike(&f));
    assert_eq!(f.request().modifier, 3);
    assert_eq!(f.request().mode, RollMode::Disadvantage);
    let original = f.raw(&[1, 15]);
    f.rejected(
        Some(1),
        TacticalAction::SubmitRollWithInspiration {
            result: original.clone(),
            die_index: 0,
            replacement: DieResult {
                sides: 20,
                value: 20,
            },
        },
    );
    f.run(
        Some(0),
        TacticalAction::SubmitRollWithInspiration {
            result: original.clone(),
            die_index: 0,
            replacement: DieResult {
                sides: 20,
                value: 20,
            },
        },
    );
    let record = f.rules().rolls.last().unwrap();
    assert_eq!(record.original_result, Some(original));
    assert_eq!(
        record.resolved.kept_dice,
        vec![DieResult {
            sides: 20,
            value: 15
        }]
    );
    assert!(!f.rules().entities[&f.actors[0]].heroic_inspiration);
    assert_eq!(f.rules().entities[&f.actors[1]].hp, 46);
    assert!(f.flow().resolution.is_none());
}

#[test]
fn unarmed_worn_armor_training_and_creature_proficiency_use_actual_sources() {
    for trained in [false, true] {
        let mut f = fixture();
        let armor = f.item("leather-armor", 1);
        f.state
            .rules
            .as_mut()
            .unwrap()
            .tactical_inventory
            .as_mut()
            .unwrap()
            .loadouts[0]
            .worn_armor = Some(armor);
        if !trained {
            f.state
                .table
                .as_mut()
                .unwrap()
                .character_profiles
                .values_mut()
                .next()
                .unwrap()
                .armor_training
                .clear();
        }
        f.run(Some(0), strike(&f));
        assert_eq!(
            f.request().mode,
            if trained {
                RollMode::Normal
            } else {
                RollMode::Disadvantage
            }
        );
        f.roll(0, if trained { &[15] } else { &[15, 1] });
        assert_eq!(
            f.rules().entities[&f.actors[1]].hp,
            if trained { 46 } else { 50 }
        );
    }
    let mut f = creature_weapon::goblin();
    f.begin();
    let strength = ability_modifier(
        f.rules().entities[&f.actors[0]].ability_scores[Ability::Strength.index()],
    );
    f.run(Some(0), strike(&f));
    assert_eq!(f.request().modifier, strength + 2); // CR source proficiency, not a PC level.
    assert_eq!(f.request().mode, RollMode::Normal); // Actual source leather training.
    f.roll(0, &[20]);
    assert_eq!(
        f.rules().entities[&f.actors[1]].hp,
        100 - (1 + strength).max(0) as u32
    );
    assert!(f.flow().budget.weapon_history.is_empty());
}

#[test]
fn unarmed_fixed_damage_has_zero_floor_and_a_real_melee_knockout_choice() {
    let mut weak = fixture();
    weak.entity_mut(0).ability_scores[Ability::Strength.index()] = 1;
    weak.run(Some(0), strike(&weak));
    weak.roll(0, &[20]);
    assert_eq!(weak.rules().entities[&weak.actors[1]].hp, 50);
    let mut f = fixture();
    f.entity_mut(1).hp = 3;
    f.run(Some(0), strike(&f));
    f.roll(0, &[20]);
    assert!(f.rules().pending.is_none());
    assert_eq!(
        f.flow()
            .resolution
            .as_ref()
            .unwrap()
            .attack
            .as_ref()
            .unwrap()
            .stage,
        TacticalAttackStage::KnockoutChoice
    );
    f.run(
        Some(0),
        TacticalAction::ChooseAttackKnockout {
            choice: KnockoutChoice::KnockOut,
        },
    );
    assert_eq!(f.rules().entities[&f.actors[1]].hp, 1);
    assert!(
        f.rules().tactical_recovery.as_ref().unwrap()[&f.actors[1]]
            .knockout
            .is_some()
    );
    assert!(f.flow().resolution.is_none());
}
