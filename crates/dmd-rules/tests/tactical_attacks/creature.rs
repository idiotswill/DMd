use super::*;
use dmd_rules::tactical_creature_equipment::*;
use dmd_rules::tactical_creatures::*;

fn horse(target_x: i32) -> Fixture {
    let mut f = Fixture::new();
    let actor = f.actors[0];
    f.state.characters.retain(|_, c| c.entity_id != actor);
    f.state.entities.get_mut(&actor).unwrap().kind = EntityKind::Creature;
    f.state.rules.as_mut().unwrap().entities.remove(&actor);
    let origin = f.meta(None);
    let built = build_creature(
        &f.state,
        &origin,
        actor,
        &CreatureBuildChoice {
            definition_id: "warhorse".into(),
            size: CreatureSize::Large,
            additional_languages: vec![],
            hit_points: CreatureHitPointChoice::Average,
            controller: CreatureController::Player(f.players[0]),
            in_lair: false,
        },
    )
    .unwrap();
    let rules = f.state.rules.as_mut().unwrap();
    rules.entities.insert(actor, built.mechanics);
    rules.tactical_creatures = Some(TacticalCreatures {
        schema_version: TACTICAL_CREATURES_SCHEMA_VERSION,
        profiles: vec![built.profile],
        runtime: vec![built.runtime],
    });
    assert!(creature_equipment_plan("warhorse", 0).unwrap().is_empty());
    f.state = materialize_creature_equipment(&f.state, &origin, actor, 0, &[], &f.pack).unwrap();
    let e = f.state.encounter.as_mut().unwrap();
    e.participants[0].size = CreatureSize::Large;
    e.participants[0].height = 20;
    e.participants[0].movement = built.movement;
    e.participants[0].senses = built.senses;
    e.participants[1].position.x = target_x;
    f.entity_mut(1).max_hp = 100;
    f.entity_mut(1).hp = 100;
    f.state.applied_event_sequence += 1;
    f
}

fn walk(f: &mut Fixture, points: &[(i32, i32)]) -> TacticalEvent {
    f.run(
        Some(0),
        TacticalAction::Move {
            path: points
                .iter()
                .map(|&(x, y)| TacticalMoveStep {
                    destination: SpatialPoint { x, y, z: 0 },
                    mode: MovementMode::Walk,
                })
                .collect(),
        },
    )
}

fn hooves(f: &Fixture) -> TacticalAction {
    TacticalAction::CreatureAttack {
        target: f.actors[1],
        feature_id: "hooves".into(),
        weapon: None,
    }
}

fn attack(f: &Fixture) -> &TacticalAttack {
    f.flow()
        .resolution
        .as_ref()
        .unwrap()
        .attack
        .as_ref()
        .unwrap()
}

fn dice_count(f: &Fixture) -> u16 {
    assert!(f.request().dice.iter().all(|die| die.sides == 4));
    f.request().dice.iter().map(|die| die.count).sum()
}

#[test]
fn source_charge_retains_real_twenty_foot_approach_and_replays_raw_hit_damage_and_prone() {
    let mut f = horse(70);
    f.begin();
    let movement = walk(&mut f, &[(20, 10), (30, 10), (40, 10), (50, 10)]);
    let items = f.state.items.clone();
    let event = f.run(Some(0), hooves(&f));
    assert_eq!(f.request().modifier, 6); // Printed stat block, never level-zero PB.
    assert_eq!(f.request().mode, RollMode::Normal);
    assert_eq!(f.request().roller, Some(f.actors[0]));
    assert_eq!(attack(&f).origin, event.meta);
    assert_eq!(
        attack(&f).admission,
        TacticalAttackAdmission::CreatureAction {
            approach: Some(TacticalChargeApproach {
                origin: movement.meta,
                movement: TacticalStraightMovement {
                    start: SpatialPoint { x: 10, y: 10, z: 0 },
                    end: SpatialPoint { x: 50, y: 10, z: 0 },
                },
            }),
        }
    );
    assert!(f.flow().budget.movement_progress.is_none());
    assert!(f.flow().budget.movement_origin.is_none());
    assert!(f.rules().timing.as_ref().unwrap().action_spent);
    assert!(!f.rules().timing.as_ref().unwrap().bonus_action_spent);
    assert_eq!(f.flow().budget.attacks_remaining, 0);
    f.rejected(
        Some(1),
        TacticalAction::SubmitRoll {
            result: f.raw(&[15]),
        },
    );
    f.rejected(
        Some(0),
        TacticalAction::SubmitRoll {
            result: f.raw(&[21]),
        },
    );
    f.roll(0, &[15]);
    assert_eq!(dice_count(&f), 4);
    assert!(!f.rules().entities[&f.actors[1]].prone);
    f.rejected(
        Some(0),
        TacticalAction::SubmitRoll {
            result: f.raw(&[1, 2]),
        },
    );
    f.roll(0, &[1, 2, 3, 4]);
    assert_eq!(f.rules().entities[&f.actors[1]].hp, 86);
    assert!(f.rules().entities[&f.actors[1]].prone);
    assert!(f.rules().pending.is_none()); // The source has no Charge saving throw.
    assert!(f.flow().resolution.is_none());
    assert_eq!(f.state.items, items);
    assert!(f.flow().budget.weapon_history.is_empty());
    f.rejected(Some(0), hooves(&f));
}

#[test]
fn source_charge_critical_doubles_both_dice_pools_and_rounds_resistance_once() {
    let mut f = horse(70);
    f.entity_mut(1).resistances.insert(DamageType::Bludgeoning);
    f.begin();
    walk(&mut f, &[(20, 10), (30, 10), (40, 10), (50, 10)]);
    f.run(Some(0), hooves(&f));
    f.roll(0, &[20]);
    assert_eq!(dice_count(&f), 8);
    // Base 5+4, Charge 5: floor((9+5)/2)=7, not floor(9/2)+floor(5/2)=6.
    f.roll(0, &[1, 1, 1, 2, 1, 1, 1, 2]);
    assert_eq!(f.rules().entities[&f.actors[1]].hp, 93);
    assert!(f.rules().entities[&f.actors[1]].prone);
}

#[test]
fn below_twenty_feet_or_direction_change_does_not_use_total_movement_as_charge() {
    for (target, path) in [
        (60, vec![(20, 10), (30, 10), (40, 10)]),
        (70, vec![(20, 10), (30, 10), (30, 20), (40, 10), (50, 10)]),
    ] {
        let mut f = horse(target);
        f.begin();
        walk(&mut f, &path);
        f.run(Some(0), hooves(&f));
        f.roll(0, &[15]);
        assert_eq!(dice_count(&f), 2);
        f.roll(0, &[2, 3]);
        assert_eq!(f.rules().entities[&f.actors[1]].hp, 91);
        assert!(!f.rules().entities[&f.actors[1]].prone);
    }
}

#[test]
fn source_charge_uses_target_size_and_prone_immunity_independently() {
    for (size, immune, expected_dice, expected_hp) in [
        (CreatureSize::Huge, false, 2, 94),
        (CreatureSize::Large, true, 4, 92),
    ] {
        let mut f = horse(70);
        let e = f.state.encounter.as_mut().unwrap();
        e.battlefield.bounds.max.x = 150;
        e.participants[1].size = size;
        e.participants[1].height = size.footprint_units() as u32;
        if immune {
            f.entity_mut(1)
                .condition_immunities
                .insert(Condition::Prone);
        }
        f.begin();
        walk(&mut f, &[(20, 10), (30, 10), (40, 10), (50, 10)]);
        f.run(Some(0), hooves(&f));
        f.roll(0, &[15]);
        assert_eq!(dice_count(&f), expected_dice);
        f.roll(0, &vec![1; usize::from(expected_dice)]);
        assert_eq!(f.rules().entities[&f.actors[1]].hp, expected_hp);
        assert!(!f.rules().entities[&f.actors[1]].prone);
    }
}

#[test]
fn continuous_split_commands_charge_but_a_new_turn_cannot_reuse_the_old_approach() {
    let mut f = horse(70);
    f.begin();
    walk(&mut f, &[(20, 10), (30, 10)]);
    let last = walk(&mut f, &[(40, 10), (50, 10)]);
    f.run(Some(0), hooves(&f));
    let TacticalAttackAdmission::CreatureAction {
        approach: Some(proof),
    } = &attack(&f).admission
    else {
        panic!()
    };
    assert_eq!(proof.origin, last.meta);
    assert_eq!(proof.movement.start.x, 10);
    f.roll(0, &[1]);
    assert!(f.rules().pending.is_none());
    assert_eq!(f.rules().entities[&f.actors[1]].hp, 100);
    assert!(!f.rules().entities[&f.actors[1]].prone);
    f.run(Some(0), TacticalAction::EndTurn);
    f.run(Some(1), TacticalAction::EndTurn);
    f.run(Some(0), hooves(&f));
    assert_eq!(
        attack(&f).admission,
        TacticalAttackAdmission::CreatureAction { approach: None }
    );
    f.roll(0, &[15]);
    assert_eq!(dice_count(&f), 2);
    f.roll(0, &[2, 3]);
    assert!(!f.rules().entities[&f.actors[1]].prone);
}

#[test]
fn accepted_action_interrupts_charge_and_rejected_source_choice_spends_nothing() {
    let mut f = horse(70);
    f.begin();
    walk(&mut f, &[(20, 10), (30, 10), (40, 10), (50, 10)]);
    for action in [
        TacticalAction::CreatureAttack {
            target: f.actors[1],
            feature_id: "invented-breath".into(),
            weapon: None,
        },
        TacticalAction::CreatureAttack {
            target: f.actors[1],
            feature_id: "hooves".into(),
            weapon: Some(ItemId::new()),
        },
    ] {
        f.rejected(Some(0), action);
    }
    f.rejected(Some(1), hooves(&f));
    let before = f.state.clone();
    let mut stale = f.meta(Some(0));
    stale.expected_event_sequence -= 1;
    assert!(resolve_tactical(&f.state, &stale, &hooves(&f), &f.pack).is_err());
    assert_eq!(f.state, before);
    assert!(f.flow().budget.movement_progress.is_some());
    assert!(!f.rules().timing.as_ref().unwrap().action_spent);
    f.run(Some(0), TacticalAction::Dodge);
    assert!(f.flow().budget.movement_progress.is_none());
    f.rejected(Some(0), hooves(&f));
}

#[test]
fn source_charge_knockout_choice_and_concentration_resume_keep_one_damage_application() {
    let mut f = horse(70);
    let group = super::spell::focus(&mut f);
    f.begin();
    walk(&mut f, &[(20, 10), (30, 10), (40, 10), (50, 10)]);
    f.run(Some(0), hooves(&f));
    f.roll(0, &[15]);
    f.roll(0, &[1, 2, 3, 4]);
    assert_eq!(f.rules().entities[&f.actors[1]].hp, 86);
    assert!(f.rules().entities[&f.actors[1]].prone);
    assert!(f.flow().resolution.as_ref().unwrap().attack.is_none());
    assert_eq!(f.request().roller, Some(f.actors[1]));
    assert!(
        matches!(f.flow().resolution.as_ref().unwrap().pending.as_ref().unwrap().work.kind,
        TacticalWorkKind::ConcentrationSave { group: actual, .. } if actual == group)
    );
    f.run(Some(1), TacticalAction::VoluntarilyFailSave);
    assert!(f.flow().resolution.is_none());
    assert_eq!(f.rules().entities[&f.actors[1]].hp, 86);
    assert!(f.rules().entities[&f.actors[1]].concentration.is_none());

    let mut f = horse(70);
    f.entity_mut(1).hp = 5;
    f.begin();
    walk(&mut f, &[(20, 10), (30, 10), (40, 10), (50, 10)]);
    f.run(Some(0), hooves(&f));
    f.roll(0, &[15]);
    f.roll(0, &[1, 2, 3, 4]);
    assert_eq!(attack(&f).stage, TacticalAttackStage::KnockoutChoice);
    assert_eq!(f.rules().entities[&f.actors[1]].hp, 5);
    assert!(!f.rules().entities[&f.actors[1]].prone);
    f.rejected(
        Some(1),
        TacticalAction::ChooseAttackKnockout {
            choice: KnockoutChoice::KnockOut,
        },
    );
    f.run(
        Some(0),
        TacticalAction::ChooseAttackKnockout {
            choice: KnockoutChoice::KnockOut,
        },
    );
    assert_eq!(f.rules().entities[&f.actors[1]].hp, 1);
    assert!(f.rules().entities[&f.actors[1]].prone);
    assert!(f.flow().resolution.is_none());
}

#[test]
fn restored_source_charge_rejects_changed_origin_geometry_dice_and_action_cost() {
    let mut f = horse(70);
    f.begin();
    walk(&mut f, &[(20, 10), (30, 10), (40, 10), (50, 10)]);
    f.run(Some(0), hooves(&f));
    for mutation in 0..16 {
        let mut corrupt = f.state.clone();
        let flow = corrupt.encounter.as_mut().unwrap().flow.as_mut().unwrap();
        let attack = flow.resolution.as_mut().unwrap().attack.as_mut().unwrap();
        let TacticalAttackAdmission::CreatureAction {
            approach: Some(proof),
        } = &mut attack.admission
        else {
            panic!()
        };
        match mutation {
            0 => proof.origin.campaign_id = CampaignId::new(),
            1 => proof.origin.actor = Some(AgentRef::Entity(f.actors[1])),
            2 => proof.origin.expected_event_sequence = attack.origin.expected_event_sequence,
            3 => proof.origin.id = attack.origin.id,
            4 => proof.movement.end.x -= 10,
            5 => proof.movement.start.x = proof.movement.end.x,
            6 => proof.movement.start.x -= 20,
            7 => attack.damage[1].dice[0].count += 1,
            8 => attack.attack_modifier += 1,
            9 => attack.armor_class += 1,
            10 => flow.budget.attacks_remaining = 1,
            11 => {
                corrupt
                    .rules
                    .as_mut()
                    .unwrap()
                    .timing
                    .as_mut()
                    .unwrap()
                    .action_spent = false
            }
            12 => attack.admission = TacticalAttackAdmission::OwnTurn,
            13 => flow.last_movement = None,
            14 => flow.last_movement.as_mut().unwrap().original.id = CommandId::new(),
            15 => proof.origin.id = CommandId::new(),
            _ => unreachable!(),
        }
        assert!(
            validate_tactical_state(&corrupt).is_err(),
            "source charge forgery {mutation}"
        );
    }
    f.roll(0, &[15]);
    let mut corrupt = f.state.clone();
    corrupt
        .rules
        .as_mut()
        .unwrap()
        .pending
        .as_mut()
        .unwrap()
        .request
        .dice
        .pop();
    assert!(validate_state(&corrupt, &f.pack).is_err());
}

#[test]
fn source_opportunity_hooves_cannot_inherit_an_earlier_own_turn_charge() {
    let mut f = horse(70);
    let encounter = f.state.encounter.as_mut().unwrap();
    encounter.participants[0].enemies = vec![f.actors[1]];
    encounter.participants[1].enemies = vec![f.actors[0]];
    f.begin();
    walk(&mut f, &[(20, 10), (30, 10), (40, 10), (50, 10)]);
    f.run(Some(0), TacticalAction::EndTurn);
    f.run(
        Some(1),
        TacticalAction::Move {
            path: vec![TacticalMoveStep {
                destination: SpatialPoint { x: 80, y: 10, z: 0 },
                mode: MovementMode::Walk,
            }],
        },
    );
    f.run(
        Some(0),
        TacticalAction::OpportunityAttack {
            choice: TacticalMeleeChoice::CreatureFeature {
                feature_id: "hooves".into(),
                weapon: None,
            },
        },
    );
    assert!(matches!(
        attack(&f).admission,
        TacticalAttackAdmission::Opportunity(_)
    ));
    f.roll(0, &[15]);
    assert_eq!(dice_count(&f), 2);
    f.roll(0, &[2, 3]);
    assert_eq!(f.rules().entities[&f.actors[1]].hp, 91);
    assert!(!f.rules().entities[&f.actors[1]].prone);
    assert!(!f.rules().timing.as_ref().unwrap().action_spent);
    assert!(
        f.rules()
            .timing
            .as_ref()
            .unwrap()
            .reactions_spent
            .contains(&f.actors[0])
    );
    assert!(f.flow().resolution.is_none());
}

#[test]
fn source_action_checks_location_before_private_range_or_implementation_details() {
    let mut f = horse(40);
    let encounter = f.state.encounter.as_mut().unwrap();
    encounter.battlefield.ambient_light = LightLevel::Darkness;
    encounter.battlefield.bounds.max.x = 200;
    f.begin();
    let mut errors = vec![];
    for position in [40, 150] {
        f.state.encounter.as_mut().unwrap().participants[1]
            .position
            .x = position;
        let before = f.state.clone();
        errors.push(
            resolve_tactical(&f.state, &f.meta(Some(0)), &hooves(&f), &f.pack)
                .unwrap_err()
                .to_string(),
        );
        assert_eq!(f.state, before);
    }
    errors.push(
        resolve_tactical(
            &f.state,
            &f.meta(Some(0)),
            &TacticalAction::CreatureAttack {
                target: EntityId::new(),
                feature_id: "hooves".into(),
                weapon: None,
            },
            &f.pack,
        )
        .unwrap_err()
        .to_string(),
    );
    assert_eq!(errors[0], errors[1]);
    assert_eq!(errors[0], errors[2]);
    assert!(errors[0].contains("currently located target"));
    assert!(!f.rules().timing.as_ref().unwrap().action_spent);
}
