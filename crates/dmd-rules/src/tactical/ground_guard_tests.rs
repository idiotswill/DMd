//! Hostile choice/retained-image controls. Altered images are never positive play.
use super::*;

fn original() -> CampaignState {
    let export: serde_json::Value = serde_json::from_str(include_str!(
        "../../../dmd-app/tests/fixtures/shield-hit-v1-selected.json"
    ))
    .unwrap();
    serde_json::from_str(export["current_state"]["state_json"].as_str().unwrap()).unwrap()
}

#[test]
fn every_pickup_producer_including_nested_cleave_refuses_both_policies_at_every_version() {
    let state = original();
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
        .as_ref()
        .unwrap();
    let mut choice = attack.weapon().unwrap().choice.clone();
    choice.equipment_change = Some(AttackEquipmentChange {
        timing: EquipmentChangeTiming::BeforeAttack,
        operation: AttackEquipmentOperation::Pickup {
            item: choice.weapon,
            hand: Hand::Right,
        },
    });
    let actions = [
        TacticalAction::Attack {
            choice: choice.clone(),
        },
        TacticalAction::OpportunityAttack {
            choice: TacticalMeleeChoice::Weapon(choice.clone()),
        },
        TacticalAction::CreatureWeaponAttack {
            feature_id: "javelin-melee".into(),
            choice: CreatureWeaponUseChoice {
                weapon: choice.weapon,
                target: choice.target,
                grip: choice.grip,
                ammunition: None,
                equipment_change: choice.equipment_change,
            },
        },
        TacticalAction::ChooseAttackMastery {
            choice: WeaponMasteryChoice::Cleave {
                attack: Box::new(choice),
            },
        },
    ];
    let mut meta = attack.origin.clone();
    meta.id = CommandId::new();
    meta.expected_event_sequence = state.applied_event_sequence;
    let pack =
        RulesPack::from_json(include_str!("../../../../content/srd-5.2.1/kernel.json")).unwrap();
    for version in [1, 2, 3, 4, 5, 6, u32::MAX] {
        let mut hostile = state.clone();
        hostile
            .encounter
            .as_mut()
            .unwrap()
            .flow
            .as_mut()
            .unwrap()
            .version = version;
        let before = hostile.clone();
        for policy in [ExecutionPolicy::Live, ExecutionPolicy::Historical] {
            for action in &actions {
                assert!(action_uses_ground_pickup(action));
                let error =
                    resolve_with_policy(&hostile, &meta, action, &pack, policy).unwrap_err();
                assert!(
                    error
                        .to_string()
                        .contains("ground pickup execution is not enabled")
                );
                assert_eq!(hostile, before);
            }
        }
    }
}

#[test]
fn retained_pickup_choice_is_denied_even_without_its_required_before_image() {
    let mut state = original();
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
    let weapon = attack.weapon_mut().unwrap();
    assert!(weapon.ground_pickup_before.is_none());
    weapon.choice.equipment_change = Some(AttackEquipmentChange {
        timing: EquipmentChangeTiming::AfterAttack,
        operation: AttackEquipmentOperation::Pickup {
            item: weapon.choice.weapon,
            hand: Hand::Left,
        },
    });
    let pack =
        RulesPack::from_json(include_str!("../../../../content/srd-5.2.1/kernel.json")).unwrap();
    let before = state.clone();
    assert!(has_unimplemented_ground_records(&state));
    assert!(
        crate::validate_state(&state, &pack)
            .unwrap_err()
            .to_string()
            .contains("ground pickup execution is not enabled")
    );
    assert!(validate_tactical_state(&state).is_err());
    assert_eq!(state, before);
}
