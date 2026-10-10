//! Constructed private-helper inputs, NOT accepted pickup histories or Ogre play.
//! The frozen source export is only a convenient real character/scene container;
//! the explicit local alterations below cannot be accepted as rules replay.
use super::*;

struct Fixture {
    state: CampaignState,
    attack: TacticalAttack,
    pack: RulesPack,
    choice: WeaponUseChoice,
}

impl Fixture {
    fn new() -> Self {
        let export: serde_json::Value = serde_json::from_str(include_str!(
            "../../../dmd-app/tests/fixtures/shield-hit-v1-selected.json"
        ))
        .unwrap();
        let mut state: CampaignState =
            serde_json::from_str(export["current_state"]["state_json"].as_str().unwrap()).unwrap();
        let attack = state
            .encounter
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .resolution
            .as_ref()
            .unwrap()
            .attack
            .clone()
            .unwrap();
        let mut choice = attack.weapon().unwrap().choice.clone();
        choice.equipment_change = Some(AttackEquipmentChange {
            timing: EquipmentChangeTiming::BeforeAttack,
            operation: AttackEquipmentOperation::Pickup {
                item: choice.weapon,
                hand: Hand::Right,
            },
        });
        let encounter = state.encounter.as_mut().unwrap();
        let location = state.scenes[&encounter.scene_id].location_id;
        let participant = encounter.participant(attack.actor).unwrap();
        let point = SpatialPoint {
            x: participant.position.x + 5,
            y: participant.position.y + 5,
            z: participant.position.z,
        };
        let flow = encounter.flow.as_mut().unwrap();
        flow.version = TacticalExecutionVersion::EncounterReleaseV1.flow_version();
        flow.phase = TacticalPhase::Active;
        flow.resolution = None;
        flow.budget = TacticalTurnBudget::default();
        flow.ground_items = vec![TacticalGroundItem {
            item: choice.weapon,
            position: point,
            origin: attack.weapon().unwrap().equipment_before.command.clone(),
        }];
        let rules = state.rules.as_mut().unwrap();
        rules.timing.as_mut().unwrap().action_spent = false;
        rules.pending = None;
        *rules
            .tactical_inventory
            .as_mut()
            .unwrap()
            .loadouts
            .iter_mut()
            .find(|l| l.actor == attack.actor)
            .unwrap() = attack.weapon().unwrap().equipment_before.clone();
        let item = state.items.get_mut(&choice.weapon).unwrap();
        item.custody = Custody::Location(location);
        item.owner = Ownership::Entity(attack.target);
        item.definition_id = "javelin".into();
        state.applied_event_sequence = attack.origin.expected_event_sequence;
        let pack = RulesPack::from_json(include_str!("../../../../content/srd-5.2.1/kernel.json"))
            .unwrap();
        Self {
            state,
            attack,
            pack,
            choice,
        }
    }

    fn input(&self) -> WeaponAttackInput<'_> {
        let rules = self.state.rules.as_ref().unwrap();
        let profile = self
            .state
            .table
            .as_ref()
            .unwrap()
            .character_profiles
            .values()
            .find(|p| p.entity_id == self.attack.actor)
            .unwrap();
        WeaponAttackInput {
            state: &self.state,
            choice: &self.choice,
            source: WeaponActorSource::Character(profile),
            pack: &self.pack,
            definitions: bundled_tactical_definitions().unwrap(),
            context: WeaponAttackContext {
                origin: &self.attack.origin,
                actor: self.attack.actor,
                turn_number: rules.timing.as_ref().unwrap().turn_number,
                on_actor_turn: true,
                window: self.attack.weapon().unwrap().window,
                distance: 10,
                base_reach: 10,
                mounted: false,
                underwater: false,
                has_swim_speed: false,
                target_is_creature: true,
                target_size: CreatureSize::Medium,
                distance_from_trigger_target: None,
            },
            loadout: &rules
                .tactical_inventory
                .as_ref()
                .unwrap()
                .loadout(self.attack.actor)
                .unwrap()
                .hands,
            history: &[],
        }
    }
}

#[test]
fn private_selected_pickup_preserves_item_owner_and_ordinary_damage_but_public_planning_refuses() {
    let f = Fixture::new();
    let original = f.state.clone();
    let plan = prepare(&f.input()).unwrap();
    assert_eq!(plan.damage.dice, vec![DieSpec { count: 1, sides: 6 }]);
    assert_eq!(
        plan.loadout_for_attack.hands[1],
        HandAssignment::Item(f.choice.weapon)
    );
    let image = plan.receipt.ground_pickup_before.as_ref().unwrap();
    assert_eq!(image.item, f.state.items[&f.choice.weapon]);
    assert_eq!(image.item.owner, Ownership::Entity(f.attack.target));
    assert_eq!(
        image.ground,
        f.state
            .encounter
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .ground_items[0]
    );
    assert!(
        prepare_weapon_attack(&f.input())
            .unwrap_err()
            .to_string()
            .contains("not enabled")
    );
    assert_eq!(f.state, original);
}

#[test]
fn private_different_weapon_pickup_consumes_one_hand_without_changing_attack_damage() {
    let mut f = Fixture::new();
    let picked = f.choice.weapon;
    let other = ItemId::new();
    let mut weapon = f.state.items[&picked].clone();
    weapon.id = other;
    weapon.custody = Custody::Entity(f.attack.actor);
    weapon.definition_id = "dagger".into();
    f.state.items.insert(other, weapon);
    f.state
        .rules
        .as_mut()
        .unwrap()
        .tactical_inventory
        .as_mut()
        .unwrap()
        .loadouts
        .iter_mut()
        .find(|l| l.actor == f.attack.actor)
        .unwrap()
        .hands
        .hands[0] = HandAssignment::Item(other);
    f.choice.weapon = other;
    f.choice.grip = WeaponGrip::OneHand(Hand::Left);
    let plan = prepare(&f.input()).unwrap();
    assert_eq!(plan.damage.dice, vec![DieSpec { count: 1, sides: 4 }]);
    assert_eq!(
        plan.loadout_for_attack.hands,
        [HandAssignment::Item(other), HandAssignment::Item(picked)]
    );
    assert_eq!(plan.receipt.ground_pickup_before.unwrap().item.id, picked);
    f.state.items.get_mut(&other).unwrap().definition_id = "greatclub".into();
    f.choice.grip = WeaponGrip::TwoHands;
    let original = f.state.clone();
    assert!(prepare(&f.input()).is_err());
    assert_eq!(f.state, original);
}

#[test]
fn private_pickup_refuses_wrong_custody_duplicates_hand_capability_and_changed_allowance() {
    for case in 0..10 {
        let mut f = Fixture::new();
        match case {
            0 => f.state.items.get_mut(&f.choice.weapon).unwrap().custody = Custody::Missing,
            1 => f.state.items.get_mut(&f.choice.weapon).unwrap().quantity = 2,
            2 => f.state.items.get_mut(&f.choice.weapon).unwrap().state = ItemState::Destroyed,
            3 => {
                let flow = f.state.encounter.as_mut().unwrap().flow.as_mut().unwrap();
                flow.ground_items.push(flow.ground_items[0].clone());
            }
            4 => {
                f.state
                    .rules
                    .as_mut()
                    .unwrap()
                    .tactical_inventory
                    .as_mut()
                    .unwrap()
                    .loadouts
                    .iter_mut()
                    .find(|l| l.actor == f.attack.actor)
                    .unwrap()
                    .hands
                    .hands[1] = HandAssignment::Item(f.choice.weapon)
            }
            5 => {
                f.choice.equipment_change.as_mut().unwrap().timing =
                    EquipmentChangeTiming::AfterAttack
            }
            6 => {
                f.state
                    .rules
                    .as_mut()
                    .unwrap()
                    .timing
                    .as_mut()
                    .unwrap()
                    .action_spent = true
            }
            7 => {
                f.state
                    .encounter
                    .as_mut()
                    .unwrap()
                    .flow
                    .as_mut()
                    .unwrap()
                    .ground_items[0]
                    .position
                    .z += 50
            }
            8 => {
                f.state
                    .rules
                    .as_mut()
                    .unwrap()
                    .entities
                    .get_mut(&f.attack.actor)
                    .unwrap()
                    .hp = 0
            }
            _ => f.attack.origin.issuer = CommandIssuer::Import,
        }
        let before = f.state.clone();
        assert!(prepare(&f.input()).is_err(), "case {case}");
        assert_eq!(f.state, before, "case {case}");
    }
}

#[test]
fn unknown_foreign_and_unseen_item_probes_have_the_same_private_refusal() {
    let mut unknown = Fixture::new();
    unknown.choice.equipment_change.as_mut().unwrap().operation =
        AttackEquipmentOperation::Pickup {
            item: ItemId::new(),
            hand: Hand::Right,
        };
    let mut foreign = Fixture::new();
    foreign
        .state
        .items
        .get_mut(&foreign.choice.weapon)
        .unwrap()
        .campaign_id = CampaignId::new();
    let mut unseen = Fixture::new();
    unseen
        .state
        .encounter
        .as_mut()
        .unwrap()
        .battlefield
        .ambient_light = LightLevel::Darkness;
    for f in [&unknown, &foreign, &unseen] {
        let before = f.state.clone();
        assert_eq!(prepare(&f.input()).unwrap_err(), illegal(UNAVAILABLE));
        assert_eq!(f.state, before);
    }
}

#[test]
fn a_preparation_cannot_be_consumed_with_another_input_state_origin_or_selected_hand() {
    let f = Fixture::new();
    for case in 0..3 {
        let input = f.input();
        let prepared = PreparedPickup::new(
            &f.state,
            &f.attack.origin,
            f.attack.actor,
            input.context.window,
            f.choice.weapon,
            Hand::Right,
            &f.pack,
            input.definitions,
        )
        .unwrap();
        let alternate_state = f.state.clone();
        let mut alternate_meta = f.attack.origin.clone();
        alternate_meta.id = CommandId::new();
        let mut alternate_choice = f.choice.clone();
        alternate_choice
            .equipment_change
            .as_mut()
            .unwrap()
            .operation = AttackEquipmentOperation::Pickup {
            item: f.choice.weapon,
            hand: Hand::Left,
        };
        let mut changed = input;
        match case {
            0 => changed.state = &alternate_state,
            1 => changed.context.origin = &alternate_meta,
            _ => changed.choice = &alternate_choice,
        }
        assert!(prepared.plan(&changed).is_err(), "case {case}");
    }
}

#[test]
fn private_pickup_does_not_treat_a_provisional_reserved_hand_as_free() {
    let mut f = Fixture::new();
    crate::tactical_hands::tests::install_attempt(&mut f.state, f.attack.actor, Hand::Right);
    // Keep the helper's supported flow5 shape. This is a private physical
    // composition control, not accepted Grapple/pickup history.
    let before = f.state.clone();
    let input = f.input();
    let hands =
        EffectiveHands::current(&f.state, f.state.rules.as_ref().unwrap(), f.attack.actor).unwrap();
    assert_eq!(input.loadout.hands, [HandAssignment::Free; 2]);
    assert!(hands.is_reserved(Hand::Right));
    assert!(hands.is_free(input.loadout, Hand::Left));
    let derive = |hand| {
        PreparedPickup::derive(
            input.state,
            input.context.origin,
            input.context.actor,
            input.context.window,
            f.choice.weapon,
            hand,
            input.pack,
            input.definitions,
        )
    };
    assert!(derive(Hand::Right).is_err());
    let picked = derive(Hand::Left).unwrap();
    assert_eq!(picked.before.item, before.items[&f.choice.weapon]);
    assert_eq!(picked.before.equipment.hands, *input.loadout);
    assert_eq!(
        picked.before.ground,
        before
            .encounter
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .ground_items[0]
    );
    let mut expected = before.clone();
    expected.items.get_mut(&f.choice.weapon).unwrap().custody = Custody::Entity(f.attack.actor);
    expected
        .encounter
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .ground_items
        .clear();
    expected
        .rules
        .as_mut()
        .unwrap()
        .tactical_inventory
        .as_mut()
        .unwrap()
        .loadouts
        .iter_mut()
        .find(|loadout| loadout.actor == f.attack.actor)
        .unwrap()
        .hands
        .hands[Hand::Left.index()] = HandAssignment::Item(f.choice.weapon);
    assert_eq!(picked.candidate, expected);
    // Fresh/public admission remains closed separately from the paired hand test.
    assert!(prepare(&input).is_err());
    assert!(prepare_weapon_attack(&input).is_err());
    assert_eq!(f.state, before);
    f.state
        .encounter
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .version = 6;
    let before = f.state.clone();
    assert!(
        EffectiveHands::current(&f.state, f.state.rules.as_ref().unwrap(), f.attack.actor).is_err()
    );
    assert!(prepare(&f.input()).is_err());
    assert!(prepare_weapon_attack(&f.input()).is_err());
    assert_eq!(f.state, before);
}

#[test]
fn private_inverse_checks_current_image_and_rederives_original_physical_cut_atomically() {
    let mut f = Fixture::new();
    // Keep unrelated ground order. These extra records are private synthetic
    // inputs only, and must not disappear or move during the bounded inverse.
    let ground = &mut f
        .state
        .encounter
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .ground_items;
    for _ in 0..2 {
        let mut other = ground[0].clone();
        other.item = ItemId::new();
        ground.push(other);
    }
    let plan = prepare(&f.input()).unwrap();
    let image = plan.receipt.ground_pickup_before.clone().unwrap();
    let prepared = PreparedPickup::new(
        &f.state,
        &f.attack.origin,
        f.attack.actor,
        f.attack.weapon().unwrap().window,
        image.item.id,
        Hand::Right,
        &f.pack,
        bundled_tactical_definitions().unwrap(),
    )
    .unwrap();
    let mut current = prepared.candidate;
    let equipment = current
        .rules
        .as_mut()
        .unwrap()
        .tactical_inventory
        .as_mut()
        .unwrap()
        .loadouts
        .iter_mut()
        .find(|l| l.actor == f.attack.actor)
        .unwrap();
    equipment.hands = plan.loadout_for_attack;
    equipment.command = f.attack.origin.clone();
    let mut attack = f.attack.clone();
    let w = attack.weapon_mut().unwrap();
    w.choice = f.choice.clone();
    w.ground_pickup_before = Some(image);
    for case in 0..6 {
        let mut hostile = current.clone();
        let mut selected = attack.clone();
        match case {
            0 => {}
            1 => hostile.items.get_mut(&f.choice.weapon).unwrap().quantity = 2,
            2 => {
                hostile
                    .rules
                    .as_mut()
                    .unwrap()
                    .tactical_inventory
                    .as_mut()
                    .unwrap()
                    .loadouts
                    .iter_mut()
                    .find(|l| l.actor == f.attack.actor)
                    .unwrap()
                    .hands
                    .hands[1] = HandAssignment::Free
            }
            3 => {
                selected
                    .weapon_mut()
                    .unwrap()
                    .ground_pickup_before
                    .as_mut()
                    .unwrap()
                    .ground
                    .origin
                    .campaign_id = CampaignId::new()
            }
            4 => {
                selected
                    .weapon_mut()
                    .unwrap()
                    .ground_pickup_before
                    .as_mut()
                    .unwrap()
                    .ground
                    .position
                    .z += 50
            }
            _ => {
                selected
                    .weapon_mut()
                    .unwrap()
                    .ground_pickup_before
                    .as_mut()
                    .unwrap()
                    .ground_index = u32::MAX
            }
        }
        let mut before = hostile.clone();
        *before
            .rules
            .as_mut()
            .unwrap()
            .tactical_inventory
            .as_mut()
            .unwrap()
            .loadouts
            .iter_mut()
            .find(|l| l.actor == f.attack.actor)
            .unwrap() = attack.weapon().unwrap().equipment_before.clone();
        let staged = before.clone();
        let original = hostile.clone();
        let result = restore_before_image(&hostile, &mut before, &selected, &f.pack);
        if case == 0 {
            result.unwrap();
            assert_eq!(before, f.state);
        } else {
            assert!(result.is_err(), "case {case}");
            assert_eq!(before, staged);
        }
        assert_eq!(hostile, original);
    }
}

#[test]
fn legacy_absent_receipts_keep_exact_json_and_new_images_never_grant_retained_authority() {
    let export: serde_json::Value = serde_json::from_str(include_str!(
        "../../../dmd-app/tests/fixtures/shield-hit-v1-selected.json"
    ))
    .unwrap();
    let old: serde_json::Value =
        serde_json::from_str(export["current_state"]["state_json"].as_str().unwrap()).unwrap();
    let receipt = &old["encounter"]["flow"]["budget"]["weapon_history"][0];
    let parsed: WeaponAttackReceipt = serde_json::from_value(receipt.clone()).unwrap();
    assert_eq!(serde_json::to_value(parsed).unwrap(), *receipt);
    let physical = &old["encounter"]["flow"]["resolution"]["attack"]["source"]["Weapon"];
    let parsed: TacticalWeaponAttack = serde_json::from_value(physical.clone()).unwrap();
    assert_eq!(serde_json::to_value(parsed).unwrap(), *physical);
    let f = Fixture::new();
    let plan = prepare(&f.input()).unwrap();
    let mut poisoned = f.state.clone();
    poisoned
        .encounter
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .budget
        .weapon_history
        .push(plan.receipt);
    assert!(has_unimplemented_ground_records(&poisoned));
    assert!(
        crate::validate_state(&poisoned, &f.pack)
            .unwrap_err()
            .to_string()
            .contains("ground pickup execution is not enabled")
    );
    assert!(crate::tactical::validate_tactical_state(&poisoned).is_err());
}
