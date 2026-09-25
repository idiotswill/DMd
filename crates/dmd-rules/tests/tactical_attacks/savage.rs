use super::*;

fn sets(
    f: &Fixture,
    first: &[u16],
    second: &[u16],
    chosen: DamageRollChoice,
) -> SavageAttackerRoll {
    SavageAttackerRoll {
        weapon_dice: Some(savage_attacker_dice(&f.state, &f.pack).unwrap()),
        first: f.raw(first),
        second: f.raw(second),
        chosen,
        inspiration: None,
    }
}

#[test]
fn source_critical_sets_keep_the_chosen_raw_and_spend_once_per_global_turn() {
    let mut f = Fixture::new();
    let choice = f.arm("club", false, false);
    let other = f.item("club", 1);
    f.state
        .rules
        .as_mut()
        .unwrap()
        .tactical_inventory
        .as_mut()
        .unwrap()
        .loadouts[0]
        .hands
        .hands[1] = HandAssignment::Item(other);
    f.begin();
    assert!(savage_attacker_dice(&f.state, &f.pack).is_err());
    let attack = f.run(
        Some(0),
        TacticalAction::Attack {
            choice: choice.clone(),
        },
    );
    assert!(savage_attacker_dice(&f.state, &f.pack).is_err());
    f.roll(0, &[20]);
    let raw = sets(&f, &[1, 2], &[4, 4], DamageRollChoice::First);
    assert_eq!(raw.weapon_dice, Some(2));
    f.rejected(
        Some(1),
        TacticalAction::SubmitSavageAttacker { roll: raw.clone() },
    );
    let mut forged = raw.clone();
    forged.weapon_dice = Some(1);
    f.rejected(
        Some(0),
        TacticalAction::SubmitSavageAttacker { roll: forged },
    );
    f.run(
        Some(0),
        TacticalAction::SubmitSavageAttacker { roll: raw.clone() },
    );
    assert_eq!(f.rules().entities[&f.actors[1]].hp, 44);
    assert_eq!(f.rules().rolls.last().unwrap().result, raw.first);
    assert_eq!(f.rules().rolls.last().unwrap().savage_attacker, Some(raw));
    let mut bonus = choice.clone();
    bonus.weapon = other;
    bonus.grip = WeaponGrip::OneHand(Hand::Right);
    bonus.purpose = WeaponAttackPurpose::LightBonus {
        trigger: attack.meta.id,
    };
    f.run(Some(0), TacticalAction::Attack { choice: bonus });
    f.roll(0, &[15]);
    assert!(savage_attacker_dice(&f.state, &f.pack).is_err());
    let raw = SavageAttackerRoll {
        weapon_dice: Some(1),
        first: f.raw(&[1]),
        second: f.raw(&[4]),
        chosen: DamageRollChoice::Second,
        inspiration: None,
    };
    f.rejected(Some(0), TacticalAction::SubmitSavageAttacker { roll: raw });
    f.roll(0, &[1]);

    // Once per turn includes another creature's turn, without refreshing the
    // player's Action, Bonus Action or Reaction more than their source permits.
    f.run(Some(0), TacticalAction::EndTurn);
    f.run(
        Some(1),
        TacticalAction::Move {
            path: vec![TacticalMoveStep {
                destination: SpatialPoint { x: 30, y: 10, z: 0 },
                mode: MovementMode::Walk,
            }],
        },
    );
    f.run(
        Some(0),
        TacticalAction::OpportunityAttack {
            choice: TacticalMeleeChoice::Weapon(choice),
        },
    );
    f.roll(0, &[15]);
    let raw = sets(&f, &[1], &[4], DamageRollChoice::Second);
    f.run(Some(0), TacticalAction::SubmitSavageAttacker { roll: raw });
    let rules = f.rules();
    let timing = rules.timing.as_ref().unwrap();
    assert_eq!(
        rules.entities[&f.actors[0]]
            .character_features
            .as_ref()
            .unwrap()
            .savage_attacker_turn,
        Some(timing.turn_number)
    );
    assert_eq!(timing.order[timing.index].actor, f.actors[1]);
    assert!(timing.reactions_spent.contains(&f.actors[0]));
    assert!(!timing.action_spent);
    assert_eq!(f.flow().budget.movement_spent, 10);
    assert!(f.flow().resolution.is_none());
}

#[test]
fn savage_inspiration_retains_both_sets_and_one_expenditure_or_can_be_declined() {
    let mut f = Fixture::new();
    let choice = f.arm("club", false, false);
    f.entity_mut(0).heroic_inspiration = true;
    f.begin();
    f.run(
        Some(0),
        TacticalAction::Attack {
            choice: choice.clone(),
        },
    );
    f.roll(0, &[15]);
    let mut raw = sets(&f, &[1], &[2], DamageRollChoice::Second);
    raw.inspiration = Some(SavageInspiration {
        roll: DamageRollChoice::Second,
        die_index: 0,
        replacement: DieResult { sides: 4, value: 4 },
    });
    let mut forged = raw.clone();
    forged.inspiration.as_mut().unwrap().replacement.sides = 6;
    f.rejected(
        Some(0),
        TacticalAction::SubmitSavageAttacker { roll: forged },
    );
    f.run(
        Some(0),
        TacticalAction::SubmitSavageAttacker { roll: raw.clone() },
    );
    assert_eq!(f.rules().rolls.last().unwrap().savage_attacker, Some(raw));
    assert_eq!(f.rules().rolls.last().unwrap().result.dice[0].value, 4);
    assert!(!f.rules().entities[&f.actors[0]].heroic_inspiration);
    f.run(Some(0), TacticalAction::EndTurn);
    f.run(Some(1), TacticalAction::EndTurn);
    f.run(Some(0), TacticalAction::Attack { choice });
    f.roll(0, &[15]);
    assert_eq!(savage_attacker_dice(&f.state, &f.pack).unwrap(), 1);
    f.roll(0, &[2]);
    assert!(f.rules().rolls.last().unwrap().savage_attacker.is_none());
    assert_eq!(
        f.rules().entities[&f.actors[0]]
            .character_features
            .as_ref()
            .unwrap()
            .savage_attacker_turn,
        Some(1)
    );
}

#[test]
fn missing_source_grant_and_invalid_raw_sets_are_rejected_unchanged() {
    let mut f = Fixture::new();
    let choice = f.arm("club", false, false);
    f.begin();
    f.run(Some(0), TacticalAction::Attack { choice });
    f.roll(0, &[15]);
    let raw = sets(&f, &[1], &[4], DamageRollChoice::Second);
    for variation in 0..5 {
        let mut bad = raw.clone();
        match variation {
            0 => bad.weapon_dice = None,
            1 => bad.second.request_id = RollRequestId::new(),
            2 => bad.second.source = RollSource::Digital,
            3 => bad.second.dice[0].value = 5,
            _ => {
                bad.inspiration = Some(SavageInspiration {
                    roll: DamageRollChoice::First,
                    die_index: 0,
                    replacement: DieResult { sides: 4, value: 4 },
                })
            }
        }
        f.rejected(Some(0), TacticalAction::SubmitSavageAttacker { roll: bad });
    }
    f.state.table.as_mut().unwrap().character_profiles.clear();
    f.rejected(Some(0), TacticalAction::SubmitSavageAttacker { roll: raw });
}
