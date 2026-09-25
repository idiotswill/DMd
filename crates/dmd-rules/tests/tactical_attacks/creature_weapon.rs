use super::*;
use dmd_rules::tactical_creature_equipment::*;
use dmd_rules::tactical_creatures::*;

fn goblin() -> Fixture {
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
            definition_id: "goblin-warrior".into(),
            size: CreatureSize::Small,
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
    let ids = creature_equipment_plan("goblin-warrior", 3)
        .unwrap()
        .iter()
        .map(|_| ItemId::new())
        .collect::<Vec<_>>();
    f.state = materialize_creature_equipment(&f.state, &origin, actor, 3, &ids, &f.pack).unwrap();
    let encounter = f.state.encounter.as_mut().unwrap();
    encounter.participants[0].size = CreatureSize::Small;
    encounter.participants[0].height = 10;
    encounter.participants[0].movement = built.movement;
    encounter.participants[0].senses = built.senses;
    encounter.participants[1].position.x = 20;
    encounter.battlefield.bounds.max.x = 1000;
    f.entity_mut(1).max_hp = 100;
    f.entity_mut(1).hp = 100;
    f.state.applied_event_sequence += 1;
    f
}

fn item(f: &Fixture, id: &str) -> ItemId {
    f.state
        .items
        .values()
        .find(|item| item.definition_id == id)
        .unwrap()
        .id
}

fn choice(f: &Fixture, id: &str) -> CreatureWeaponUseChoice {
    CreatureWeaponUseChoice {
        weapon: item(f, id),
        target: f.actors[1],
        grip: if id == "shortbow" {
            WeaponGrip::TwoHands
        } else {
            WeaponGrip::OneHand(Hand::Right)
        },
        ammunition: (id == "shortbow").then(|| item(f, "arrows")),
        equipment_change: Some(AttackEquipmentChange {
            timing: EquipmentChangeTiming::BeforeAttack,
            operation: AttackEquipmentOperation::Equip {
                item: item(f, id),
                hand: Hand::Right,
            },
        }),
    }
}

fn action(id: &str, choice: CreatureWeaponUseChoice) -> TacticalAction {
    TacticalAction::CreatureWeaponAttack {
        feature_id: id.into(),
        choice,
    }
}

fn pending(f: &Fixture) -> &TacticalAttack {
    f.flow()
        .resolution
        .as_ref()
        .unwrap()
        .attack
        .as_ref()
        .unwrap()
}

fn raw(f: &Fixture, values: &[u16]) -> RollResult {
    let request = f.request();
    let mut sides = request
        .dice
        .iter()
        .flat_map(|spec| std::iter::repeat_n(spec.sides, usize::from(spec.count)));
    RollResult {
        request_id: request.id,
        source: RollSource::Physical,
        dice: values
            .iter()
            .map(|value| DieResult {
                sides: sides.next().unwrap_or(request.dice[0].sides),
                value: *value,
            })
            .collect(),
    }
}

fn roll(f: &mut Fixture, actor: usize, values: &[u16]) {
    f.run(
        Some(actor),
        TacticalAction::SubmitRoll {
            result: raw(f, values),
        },
    );
}

/// Legal prepared starting state for this reducer slice, not a playable Doff
/// Shield action. That paid equipment transition remains an explicit Gate4 task.
fn prepare_bow_without_shield(f: &mut Fixture) {
    let actor = f.actors[0];
    let origin = f.meta(None);
    let profile = f
        .rules()
        .tactical_creatures
        .as_ref()
        .unwrap()
        .profile(actor)
        .unwrap()
        .clone();
    let loadout = f
        .state
        .rules
        .as_mut()
        .unwrap()
        .tactical_inventory
        .as_mut()
        .unwrap()
        .loadouts
        .iter_mut()
        .find(|l| l.actor == actor)
        .unwrap();
    loadout.shield = None;
    loadout.hands = WeaponLoadout::default();
    loadout.command = origin;
    f.entity_mut(0).armor = creature_current_armor(&f.state, &profile).unwrap();
    f.state.encounter.as_mut().unwrap().participants[1]
        .position
        .x = 60;
    f.state.applied_event_sequence += 1;
}

#[test]
fn source_scimitar_equips_stowed_real_weapon_beside_shield_and_pays_one_action() {
    let mut f = goblin();
    let selected = choice(&f, "scimitar");
    let original = f.loadout().clone();
    assert_eq!(original.hands.hands[1], HandAssignment::Free);
    assert_eq!(
        original.hands.hands[0],
        HandAssignment::Item(item(&f, "shield"))
    );
    f.begin();
    let before = f.state.clone();
    let accepted = f.run(Some(0), action("scimitar", selected.clone()));
    assert_eq!(f.request().modifier, 4);
    assert_eq!(f.request().roller, Some(f.actors[0]));
    assert_eq!(
        pending(&f).weapon().unwrap().choice.ability,
        Ability::Dexterity
    );
    assert_eq!(pending(&f).weapon().unwrap().equipment_before, original);
    assert_eq!(pending(&f).origin, accepted.meta);
    assert!(matches!(
        pending(&f).source,
        TacticalAttackSource::CreatureWeapon { .. }
    ));
    assert_eq!(
        f.loadout().hands.hands[1],
        HandAssignment::Item(selected.weapon)
    );
    assert_eq!(f.loadout().shield, original.shield);
    assert_eq!(f.state.items, before.items);
    assert!(f.rules().timing.as_ref().unwrap().action_spent);
    assert!(!f.rules().timing.as_ref().unwrap().bonus_action_spent);
    f.rejected(
        Some(1),
        TacticalAction::SubmitRoll {
            result: raw(&f, &[15]),
        },
    );
    roll(&mut f, 0, &[15]);
    assert_eq!(f.request().dice, vec![DieSpec { count: 1, sides: 6 }]);
    roll(&mut f, 0, &[4]);
    assert_eq!(f.rules().entities[&f.actors[1]].hp, 94);
    assert_eq!(f.flow().budget.weapon_history.len(), 1);
    assert!(f.flow().resolution.is_none());
    f.rejected(Some(0), action("scimitar", selected));
}

#[test]
fn source_bow_never_waives_shield_two_hands_or_its_explicit_equip_allowance() {
    let mut f = goblin();
    f.begin();
    let selected = choice(&f, "shortbow");
    f.rejected(Some(0), action("shortbow", selected.clone()));
    let mut one_hand = selected.clone();
    one_hand.grip = WeaponGrip::OneHand(Hand::Right);
    f.rejected(Some(0), action("shortbow", one_hand));
    let mut doff = selected;
    doff.equipment_change = Some(AttackEquipmentChange {
        timing: EquipmentChangeTiming::BeforeAttack,
        operation: AttackEquipmentOperation::Unequip {
            item: item(&f, "shield"),
        },
    });
    f.rejected(Some(0), action("shortbow", doff));
    assert_eq!(f.state.items[&item(&f, "arrows")].quantity, 3);
    assert!(!f.rules().timing.as_ref().unwrap().action_spent);

    let mut f = goblin();
    prepare_bow_without_shield(&mut f);
    f.begin();
    let mut selected = choice(&f, "shortbow");
    selected.equipment_change = None;
    f.rejected(Some(0), action("shortbow", selected));
    let mut selected = choice(&f, "shortbow");
    selected.ammunition = None;
    f.rejected(Some(0), action("shortbow", selected));
    let mut selected = choice(&f, "shortbow");
    selected.ammunition = Some(item(&f, "scimitar"));
    f.rejected(Some(0), action("shortbow", selected));
    assert_eq!(f.state.items[&item(&f, "arrows")].quantity, 3);
    assert!(!f.rules().timing.as_ref().unwrap().action_spent);
}

#[test]
fn source_shortbow_reserves_last_real_arrow_once_and_replays_each_pending_stage() {
    let mut f = goblin();
    prepare_bow_without_shield(&mut f);
    let selected = choice(&f, "shortbow");
    let arrow = selected.ammunition.unwrap();
    f.state.items.get_mut(&arrow).unwrap().quantity = 1;
    f.begin();
    let before = f.state.clone();
    let event = f.run(Some(0), action("shortbow", selected.clone()));
    assert_eq!(f.state.items[&arrow].quantity, 0);
    assert_eq!(f.state.items[&arrow].state, ItemState::Spent);
    assert_eq!(
        pending(&f)
            .weapon()
            .unwrap()
            .ammunition
            .as_ref()
            .unwrap()
            .quantity_before,
        1
    );
    assert_eq!(
        f.loadout().hands.hands,
        [HandAssignment::Item(selected.weapon); 2]
    );
    assert_eq!(
        replay_tactical(&before, &event, &f.pack)
            .unwrap()
            .next_state
            .items,
        f.state.items
    );
    roll(&mut f, 0, &[15]);
    f.rejected(
        Some(0),
        TacticalAction::SubmitRoll {
            result: raw(&f, &[7]),
        },
    );
    roll(&mut f, 0, &[3]);
    assert_eq!(f.rules().entities[&f.actors[1]].hp, 95);
    assert_eq!(f.state.items[&arrow].quantity, 0);
    f.run(Some(0), TacticalAction::EndTurn);
    f.run(Some(1), TacticalAction::EndTurn);
    let mut held = selected;
    held.equipment_change = None;
    f.rejected(Some(0), action("shortbow", held));
    assert_eq!(f.state.items[&arrow].state, ItemState::Spent);
}

#[test]
fn source_advantage_extra_die_is_mandatory_cancels_normally_and_doubles_on_critical() {
    for (id, cancel, critical) in [
        ("scimitar", false, true),
        ("shortbow", false, false),
        ("shortbow", true, false),
    ] {
        let mut f = goblin();
        if id == "shortbow" {
            prepare_bow_without_shield(&mut f);
        }
        f.state.rules.as_mut().unwrap().effects.push(ActiveEffect {
            id: EffectId::new(),
            source: f.actors[0],
            target: f.actors[1],
            condition: Some(Condition::Blinded),
            label: "Source blindness".into(),
            expires: Expiry::Never,
            concentration_owner: None,
        });
        if cancel {
            f.entity_mut(0).prone = true;
        }
        f.begin();
        let selected = choice(&f, id);
        f.run(Some(0), action(id, selected));
        assert_eq!(
            f.request().mode,
            if cancel {
                RollMode::Normal
            } else {
                RollMode::Advantage
            }
        );
        let face = if critical { 20 } else { 15 };
        roll(
            &mut f,
            0,
            &if cancel { vec![face] } else { vec![face, face] },
        );
        let count = if critical { 2 } else { 1 };
        let mut expected = vec![DieSpec { count, sides: 6 }];
        if !cancel {
            expected.push(DieSpec { count, sides: 4 });
        }
        assert_eq!(f.request().dice, expected);
        let values = if critical {
            vec![2, 3, 1, 2]
        } else if cancel {
            vec![3]
        } else {
            vec![3, 2]
        };
        roll(&mut f, 0, &values);
        assert_eq!(
            f.rules().entities[&f.actors[1]].hp,
            if critical {
                90
            } else if cancel {
                95
            } else {
                93
            }
        );
    }
}

#[test]
fn source_shortbow_uses_printed_ranges_and_cannot_probe_a_hidden_retained_id() {
    for (x, mode) in [
        (170, RollMode::Normal),
        (180, RollMode::Disadvantage),
        (650, RollMode::Disadvantage),
    ] {
        let mut f = goblin();
        prepare_bow_without_shield(&mut f);
        f.state.encounter.as_mut().unwrap().participants[1]
            .position
            .x = x;
        f.begin();
        let selected = choice(&f, "shortbow");
        f.run(Some(0), action("shortbow", selected));
        assert_eq!(f.request().mode, mode);
        roll(
            &mut f,
            0,
            &if mode == RollMode::Normal {
                vec![1]
            } else {
                vec![1, 1]
            },
        );
    }
    let mut f = goblin();
    prepare_bow_without_shield(&mut f);
    f.state.encounter.as_mut().unwrap().participants[1]
        .position
        .x = 660;
    f.begin();
    let selected = choice(&f, "shortbow");
    f.rejected(Some(0), action("shortbow", selected.clone()));
    let encounter = f.state.encounter.as_mut().unwrap();
    encounter.battlefield.ambient_light = LightLevel::Darkness;
    // Both positions are beyond the actual source Darkvision; keep senses intact.
    let mut errors = vec![];
    for x in [180, 660] {
        f.state.encounter.as_mut().unwrap().participants[1]
            .position
            .x = x;
        errors.push(
            resolve_tactical(
                &f.state,
                &f.meta(Some(0)),
                &action("shortbow", selected.clone()),
                &f.pack,
            )
            .unwrap_err()
            .to_string(),
        );
    }
    assert_eq!(errors[0], errors[1]);
    assert!(errors[0].contains("currently located target"));
}

#[test]
fn source_physical_attack_uses_custody_not_ownership_and_rejects_false_implements() {
    let mut f = goblin();
    let selected = choice(&f, "scimitar");
    f.state.items.get_mut(&selected.weapon).unwrap().owner = Ownership::Entity(f.actors[1]);
    f.begin();
    let before = f.state.clone();
    for mutation in 0..5 {
        let mut candidate = f.state.clone();
        let weapon = candidate.items.get_mut(&selected.weapon).unwrap();
        match mutation {
            0 => weapon.custody = Custody::Entity(f.actors[1]),
            1 => weapon.custody = Custody::Missing,
            2 => weapon.state = ItemState::Destroyed,
            3 => weapon.quantity = 2,
            4 => weapon.definition_id = "dagger".into(),
            _ => unreachable!(),
        }
        assert!(
            resolve_tactical(
                &candidate,
                &f.meta(Some(0)),
                &action("scimitar", selected.clone()),
                &f.pack
            )
            .is_err()
        );
    }
    assert_eq!(f.state, before);
    f.run(Some(0), action("scimitar", selected));
    roll(&mut f, 0, &[15]);
    roll(&mut f, 0, &[2]);
    assert_eq!(f.rules().entities[&f.actors[1]].hp, 96);
}

#[test]
fn source_physical_damage_finishes_equipment_before_target_concentration_and_knockout() {
    let mut f = goblin();
    let group = super::spell::focus(&mut f);
    let weapon = item(&f, "scimitar");
    // Prepared held scimitar permits the one AFTER-attack unequip choice.
    let meta = f.meta(None);
    f.state
        .rules
        .as_mut()
        .unwrap()
        .tactical_inventory
        .as_mut()
        .unwrap()
        .loadouts[0]
        .hands
        .hands[1] = HandAssignment::Item(weapon);
    f.state
        .rules
        .as_mut()
        .unwrap()
        .tactical_inventory
        .as_mut()
        .unwrap()
        .loadouts[0]
        .command = meta;
    f.state.applied_event_sequence += 1;
    let mut selected = choice(&f, "scimitar");
    selected.equipment_change = Some(AttackEquipmentChange {
        timing: EquipmentChangeTiming::AfterAttack,
        operation: AttackEquipmentOperation::Unequip { item: weapon },
    });
    f.begin();
    f.run(Some(0), action("scimitar", selected));
    roll(&mut f, 0, &[15]);
    assert_eq!(f.loadout().hands.hands[1], HandAssignment::Item(weapon));
    roll(&mut f, 0, &[3]);
    assert_eq!(f.loadout().hands.hands[1], HandAssignment::Free);
    assert!(f.flow().resolution.as_ref().unwrap().attack.is_none());
    assert!(
        matches!(f.flow().resolution.as_ref().unwrap().pending.as_ref().unwrap().work.kind, TacticalWorkKind::ConcentrationSave { group: actual, .. } if actual == group)
    );
    f.run(Some(1), TacticalAction::VoluntarilyFailSave);
    assert_eq!(f.rules().entities[&f.actors[1]].hp, 95);
    assert!(f.flow().resolution.is_none());

    let mut f = goblin();
    f.entity_mut(1).hp = 3;
    f.begin();
    let selected = choice(&f, "scimitar");
    f.run(Some(0), action("scimitar", selected));
    roll(&mut f, 0, &[15]);
    roll(&mut f, 0, &[3]);
    assert_eq!(pending(&f).stage, TacticalAttackStage::KnockoutChoice);
    assert_eq!(f.rules().entities[&f.actors[1]].hp, 3);
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
    assert!(f.flow().resolution.is_none());
}

#[test]
fn restored_source_weapon_rejects_source_grip_ammo_bonus_and_pending_fact_forgeries() {
    let mut f = goblin();
    prepare_bow_without_shield(&mut f);
    f.begin();
    let selected = choice(&f, "shortbow");
    f.run(Some(0), action("shortbow", selected));
    for mutation in 0..12 {
        let mut state = f.state.clone();
        let attack = state
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
        let TacticalAttackSource::CreatureWeapon {
            source,
            feature_id,
            weapon,
        } = &mut attack.source
        else {
            panic!()
        };
        match mutation {
            0 => source.definition_id = "skeleton".into(),
            1 => *feature_id = "scimitar".into(),
            2 => weapon.choice.grip = WeaponGrip::OneHand(Hand::Right),
            3 => weapon.choice.ability = Ability::Strength,
            4 => weapon.ammunition.as_mut().unwrap().quantity_before += 1,
            5 => weapon.equipment_before.command.campaign_id = CampaignId::new(),
            6 => attack.attack_modifier += 1,
            7 => attack.damage[0].modifier += 1,
            8 => attack.mode = RollMode::Advantage,
            9 => attack.armor_class += 1,
            10 => attack.admission = TacticalAttackAdmission::OwnTurn,
            11 => {
                state
                    .rules
                    .as_mut()
                    .unwrap()
                    .timing
                    .as_mut()
                    .unwrap()
                    .action_spent = false
            }
            _ => unreachable!(),
        }
        assert!(
            validate_tactical_state(&state).is_err(),
            "source weapon forgery {mutation}"
        );
    }
}

#[test]
fn dead_body_rejects_before_source_cost_but_zero_hp_living_target_remains_legal() {
    for dead in [true, false] {
        let mut f = goblin();
        prepare_bow_without_shield(&mut f);
        f.begin();
        f.entity_mut(1).hp = 0;
        f.entity_mut(1).prone = true;
        f.entity_mut(1).death.dead = dead;
        if dead {
            f.state.entities.get_mut(&f.actors[1]).unwrap().existence = EntityExistence::Dead;
            f.state
                .characters
                .values_mut()
                .find(|c| c.entity_id == f.actors[1])
                .unwrap()
                .status = CharacterStatus::Dead;
        }
        validate_state(&f.state, &f.pack).unwrap();
        validate_tactical_state(&f.state).unwrap();
        assert!(f.state.validate().is_empty());
        let selected = choice(&f, "shortbow");
        if dead {
            let before = f.state.clone();
            let error = resolve_tactical(
                &f.state,
                &f.meta(Some(0)),
                &action("shortbow", selected),
                &f.pack,
            )
            .unwrap_err();
            assert!(
                matches!(error, RulesError::Prerequisite(message) if message.contains("body/object"))
            );
            assert_eq!(f.state, before);
            assert_eq!(f.state.items[&item(&f, "arrows")].quantity, 3);
            assert!(!f.rules().timing.as_ref().unwrap().action_spent);
        } else {
            f.run(Some(0), action("shortbow", selected));
            // Prone at range cancels the Unconscious target's Advantage.
            assert_eq!(f.request().mode, RollMode::Normal);
            roll(&mut f, 0, &[15]);
            roll(&mut f, 0, &[1]);
            assert_eq!(f.rules().entities[&f.actors[1]].hp, 0);
            assert!(!f.rules().entities[&f.actors[1]].death.dead);
            assert_eq!(f.rules().entities[&f.actors[1]].death.failures, 1);
            assert!(f.flow().resolution.is_none());
        }
    }
}

#[test]
fn lethal_source_weapon_completes_and_retained_dead_target_state_replays() {
    let mut f = goblin();
    prepare_bow_without_shield(&mut f);
    f.entity_mut(1).hp = 3;
    f.entity_mut(1).max_hp = 3;
    f.begin();
    let selected = choice(&f, "shortbow");
    f.run(Some(0), action("shortbow", selected));
    roll(&mut f, 0, &[15]);
    roll(&mut f, 0, &[6]); // Eight damage leaves enough excess for instant death.
    assert!(f.rules().entities[&f.actors[1]].death.dead);
    assert_eq!(
        f.state.entities[&f.actors[1]].existence,
        EntityExistence::Dead
    );
    assert!(f.flow().resolution.is_none());
    assert_eq!(f.state.items[&item(&f, "arrows")].quantity, 2);
    let restored = CampaignState::decode_json(&f.state.encode_json().unwrap()).unwrap();
    assert_eq!(restored, f.state);
    validate_state(&restored, &f.pack).unwrap();
    validate_tactical_state(&restored).unwrap();
    assert!(restored.validate().is_empty());
}
