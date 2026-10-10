//! Pure printed/ordinary composition and rejection controls. The plan images below
//! are synthetic inputs to a private policy, not materialized Ogre profiles or
//! accepted attacks. Genuine Ogre gameplay is covered by application scenarios.
use super::*;
use crate::tactical_creatures::{creature_source_pin, ogre_weapon_program};
use crate::tactical_definitions::bundled_ogre;

fn program(id: &str) -> OgreWeaponProgram {
    ogre_weapon_program(&creature_source_pin(bundled_ogre().unwrap()).unwrap(), id).unwrap()
}

fn ordinary_image(program: &OgreWeaponProgram) -> WeaponAttackPlan {
    let actor = EntityId::new();
    let item = ItemId::new();
    let origin = CommandMeta {
        id: CommandId::new(),
        campaign_id: CampaignId::new(),
        session_id: None,
        issuer: CommandIssuer::Admin,
        actor: Some(AgentRef::Entity(actor)),
        expected_event_sequence: 0,
    };
    let hands = WeaponLoadout {
        hands: [HandAssignment::Item(item), HandAssignment::Free],
    };
    WeaponAttackPlan {
        receipt: WeaponAttackReceipt {
            after_equipment: None,
            ground_pickup_before: None,
            origin: origin.clone(),
            actor,
            turn_number: 1,
            on_actor_turn: true,
            window: WeaponActionWindow {
                id: origin.id,
                kind: WeaponActionKind::AttackAction,
            },
            weapon: item,
            definition_id: program.weapon().id.clone(),
            target: EntityId::new(),
            delivery: program.delivery(),
            ability: Ability::Strength,
            grip: if program.weapon().hands == crate::tactical_definitions::WeaponHands::Two {
                WeaponGrip::TwoHands
            } else {
                WeaponGrip::OneHand(Hand::Left)
            },
            ammunition: None,
            purpose: WeaponAttackPurpose::Normal,
            outcome: WeaponAttackOutcome::Pending,
        },
        attack_modifier: 6,
        ability_modifier: 4,
        proficiency_bonus: 2,
        proficient: true,
        damage: WeaponDamageProfile {
            dice: program.weapon().damage.dice.clone(),
            modifier: 4,
            damage_type: program.weapon().damage_type,
        },
        disadvantage: vec![],
        automatic_miss: false,
        mastery: None,
        reach: 10,
        loadout_for_attack: hands.clone(),
        loadout_after_attack: hands,
        ammunition: None,
        thrown_weapon: (program.delivery() == WeaponDelivery::Thrown).then_some(item),
    }
}

#[test]
fn only_exact_printed_dice_replace_validated_ordinary_dice() {
    for (id, sides, damage_type, delivery) in [
        (
            "greatclub",
            8,
            DamageType::Bludgeoning,
            WeaponDelivery::Melee,
        ),
        (
            "javelin-melee",
            6,
            DamageType::Piercing,
            WeaponDelivery::Melee,
        ),
        (
            "javelin-thrown",
            6,
            DamageType::Piercing,
            WeaponDelivery::Thrown,
        ),
    ] {
        let p = program(id);
        let mut plan = ordinary_image(&p);
        let facts = ogre_facts(p);
        let original = plan.clone();
        require_matching_facts(&facts, &plan, 0).unwrap();
        assert_eq!(facts.delivery, delivery);
        assert_eq!(plan.damage.dice, [DieSpec { count: 1, sides }]);
        for mode in [
            RollMode::Normal,
            RollMode::Advantage,
            RollMode::Disadvantage,
        ] {
            assert_eq!(
                source_damage(&facts, mode),
                [AttackDamageComponent {
                    dice: vec![DieSpec { count: 2, sides }],
                    modifier: 4,
                    damage_type,
                }]
            );
        }
        assert_eq!(
            plan, original,
            "ordinary damage and physical plan remain unchanged"
        );
        // Synthetic ordinary disadvantage/miss flags survive source composition;
        // actual geometry and automatic completion remain later gameplay coverage.
        plan.disadvantage.push(WeaponDisadvantage::LongRange);
        plan.automatic_miss = true;
        plan.attack_modifier = 2;
        require_matching_facts(&facts, &plan, 2).unwrap();
        assert_eq!(source_damage(&facts, RollMode::Disadvantage)[0].modifier, 4);
        assert!(require_matching_facts(&facts, &plan, 0).is_err());
    }
}

#[test]
fn shared_damage_request_doubles_only_the_printed_dice() {
    // Reuse a genuine old attack container only to exercise the shared request
    // formatter with synthetic damage/outcome inputs. This is not a valid source
    // after-image, accepted Ogre attack, critical roll or retained replay proof.
    let export: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../../dmd-app/tests/fixtures/shield-hit-v1-selected.json"
    ))
    .unwrap();
    let original: CampaignState =
        serde_json::from_str(export["current_state"]["state_json"].as_str().unwrap()).unwrap();
    for id in ["greatclub", "javelin-melee", "javelin-thrown"] {
        let facts = ogre_facts(program(id));
        for critical in [false, true] {
            let mut state = original.clone();
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
            attack.damage = source_damage(&facts, RollMode::Normal);
            attack.outcome = Some(WeaponAttackOutcome::Hit {
                critical,
                damage_dealt: 0,
            });
            let work = TacticalWorkItem {
                occurrence: 0,
                kind: TacticalWorkKind::AttackDamage,
            };
            let key = super::super::key(&state, &work).unwrap();
            let request = super::super::request(&state, &work, key).unwrap().unwrap();
            assert_eq!(
                request.dice,
                [DieSpec {
                    count: if critical { 4 } else { 2 },
                    sides: if id == "greatclub" { 8 } else { 6 }
                }]
            );
            assert_eq!(
                request.modifier, 0,
                "flat damage is applied once after dice"
            );
            assert_eq!(
                state
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
                    .as_ref()
                    .unwrap()
                    .damage[0]
                    .modifier,
                4
            );
        }
    }
}

#[test]
fn altered_physical_numerics_delivery_and_grip_cannot_receive_printed_dice() {
    for id in ["greatclub", "javelin-melee", "javelin-thrown"] {
        let p = program(id);
        let plan = ordinary_image(&p);
        let facts = ogre_facts(p);
        for case in 0..13 {
            let mut bad = plan.clone();
            match case {
                0 => bad.damage.dice[0].count = 2, // Already printed is not ordinary proof.
                1 => bad.damage.dice[0].sides = 20,
                2 => bad.damage.modifier += 1,
                3 => bad.damage.damage_type = DamageType::Fire,
                4 => bad.attack_modifier += 1,
                5 => bad.receipt.definition_id = "dagger".into(),
                6 => bad.receipt.delivery = WeaponDelivery::Shot,
                7 => bad.receipt.ability = Ability::Dexterity,
                8 => {
                    bad.receipt.purpose = WeaponAttackPurpose::LightBonus {
                        trigger: CommandId::new(),
                    }
                }
                9 => bad.receipt.ammunition = Some(ItemId::new()),
                10 => {
                    bad.ammunition = Some(AmmunitionExpenditure {
                        stack: ItemId::new(),
                        quantity: 1,
                    })
                }
                11 => {
                    bad.receipt.grip = if id == "greatclub" {
                        WeaponGrip::OneHand(Hand::Left)
                    } else {
                        WeaponGrip::TwoHands
                    }
                }
                _ => {
                    bad.thrown_weapon = if id == "javelin-thrown" {
                        None
                    } else {
                        Some(bad.receipt.weapon)
                    }
                }
            }
            assert!(
                require_matching_facts(&facts, &bad, 0).is_err(),
                "{id} case {case}"
            );
        }
    }
}

#[test]
fn default_policy_still_requires_the_complete_printed_formula() {
    // Counterfactual strict-policy control on the same immutable numeric facts;
    // not a claim that a new source was admitted or old Goblin gameplay executed.
    let p = program("greatclub");
    let plan = ordinary_image(&p);
    let mut facts = ogre_facts(p);
    facts.ogre = None;
    assert!(require_matching_facts(&facts, &plan, 0).is_err());
    let mut matched = plan;
    matched.damage.dice = facts.base.dice.clone();
    require_matching_facts(&facts, &matched, 0).unwrap();
}

#[test]
fn typed_physical_choice_is_distinct_and_carries_no_mechanical_overrides() {
    let item = ItemId::new();
    let choice = TacticalMeleeChoice::CreatureWeapon {
        feature_id: "greatclub".into(),
        weapon: item,
        grip: WeaponGrip::TwoHands,
    };
    let value = serde_json::to_value(&choice).unwrap();
    assert_eq!(
        value,
        serde_json::json!({"CreatureWeapon": {
            "feature_id":"greatclub", "weapon":item, "grip":"TwoHands"
        }})
    );
    assert_eq!(
        serde_json::from_value::<TacticalMeleeChoice>(value.clone()).unwrap(),
        choice
    );
    for key in [
        "damage",
        "source",
        "target",
        "equipment_change",
        "ammunition",
    ] {
        let mut forged = value.clone();
        forged["CreatureWeapon"][key] = serde_json::json!(1);
        assert!(serde_json::from_value::<TacticalMeleeChoice>(forged).is_err());
    }
    let old = TacticalMeleeChoice::CreatureFeature {
        feature_id: "greatclub".into(),
        weapon: Some(item),
    };
    assert_eq!(
        serde_json::to_value(&old).unwrap(),
        serde_json::json!({"CreatureFeature": {
            "feature_id":"greatclub", "weapon":item
        }})
    );
    assert_ne!(
        TacticalMeleeSource::CreatureWeapon {
            feature_id: "greatclub".into(),
            item
        },
        TacticalMeleeSource::CreatureFeature {
            feature_id: "greatclub".into(),
            weapon: Some(item)
        }
    );
}

#[test]
fn physical_source_executor_gate_does_not_depend_on_live_policy() {
    // Negative/version-unit images derived from a genuine old snapshot; changing
    // its flow version is not an accepted upgrade or any new source play.
    let mut state = crate::tactical_hands::tests::source_state();
    for version in [1, 2, 3, 4, 6] {
        state
            .encounter
            .as_mut()
            .unwrap()
            .flow
            .as_mut()
            .unwrap()
            .version = version;
        assert!(
            require_ogre_execution(&state)
                .unwrap_err()
                .to_string()
                .contains("current executor")
        );
    }
    state
        .encounter
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .version = 5;
    require_ogre_execution(&state).unwrap();
}

#[test]
fn new_physical_oa_wire_refuses_old_executors_under_both_policies_without_mutation() {
    let original = crate::tactical_hands::tests::source_state();
    let pack = crate::tactical_hands::tests::pack();
    // Flow2 is the untouched original cut. Other versions are explicitly synthetic
    // negative version controls; no invented Ogre or accepted upgrade is involved.
    for version in [1, 2, 3, 4] {
        let mut state = original.clone();
        state
            .encounter
            .as_mut()
            .unwrap()
            .flow
            .as_mut()
            .unwrap()
            .version = version;
        let meta = CommandMeta {
            id: CommandId::new(),
            campaign_id: state.campaign_id(),
            session_id: None,
            issuer: CommandIssuer::Admin,
            actor: None,
            expected_event_sequence: state.applied_event_sequence,
        };
        let before = serde_json::to_value(&state).unwrap();
        let action = TacticalAction::OpportunityAttack {
            choice: TacticalMeleeChoice::CreatureWeapon {
                feature_id: "greatclub".into(),
                weapon: ItemId::new(),
                grip: WeaponGrip::TwoHands,
            },
        };
        for policy in [
            crate::tactical::ExecutionPolicy::Live,
            crate::tactical::ExecutionPolicy::Historical,
        ] {
            let error = crate::tactical::resolve_with_policy(&state, &meta, &action, &pack, policy)
                .unwrap_err();
            assert!(
                error
                    .to_string()
                    .contains("physical Ogre attacks require the current executor"),
                "flow {version}: {error}"
            );
            assert_eq!(serde_json::to_value(&state).unwrap(), before);
        }
    }
}
