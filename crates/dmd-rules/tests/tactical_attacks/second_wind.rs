use super::*;

fn fighter() -> Fixture {
    let mut f = Fixture::new();
    f.arm("club", false, false);
    f.entity_mut(0).hp = 1;
    f.entity_mut(0).heroic_inspiration = true;
    f.begin();
    f
}

#[test]
fn second_wind_pays_once_retains_raw_dice_and_uses_only_the_bonus_action() {
    let mut f = fighter();
    let actor = f.actors[0];
    f.rejected(Some(1), TacticalAction::SecondWind);
    let declaration = f.run(Some(0), TacticalAction::SecondWind);
    let pending = f.rules().pending.as_ref().unwrap();
    assert_eq!(
        pending.request.dice,
        vec![DieSpec {
            count: 1,
            sides: 10
        }]
    );
    assert_eq!(pending.request.modifier, 1);
    assert_eq!(pending.request.roller, Some(actor));
    assert!(
        matches!(pending.purpose, PendingPurpose::TacticalResolution { key, .. }
        if key.role == TacticalRollRole::SecondWind && key.origin == declaration.meta.id)
    );
    assert!(f.rules().timing.as_ref().unwrap().bonus_action_spent);
    assert!(!f.rules().timing.as_ref().unwrap().action_spent);
    assert_eq!(
        f.rules().entities[&actor]
            .character_features
            .as_ref()
            .unwrap()
            .second_wind_remaining,
        1
    );
    f.rejected(Some(0), TacticalAction::SecondWind);
    f.rejected(Some(0), TacticalAction::EndTurn);
    f.rejected(
        Some(1),
        TacticalAction::SubmitRoll {
            result: f.raw(&[4]),
        },
    );
    f.rejected(
        Some(0),
        TacticalAction::SubmitRoll {
            result: f.raw(&[11]),
        },
    );
    let raw = f.raw(&[4]);
    f.run(
        Some(0),
        TacticalAction::SubmitRoll {
            result: raw.clone(),
        },
    );
    assert_eq!(f.rules().entities[&actor].hp, 6);
    assert_eq!(f.rules().rolls.last().unwrap().result, raw);
    assert!(f.flow().resolution.is_none());
    f.rejected(Some(0), TacticalAction::SecondWind);
    f.rejected(Some(0), TacticalAction::SubmitRoll { result: raw });
    f.run(Some(0), TacticalAction::Dodge);
    assert!(f.rules().timing.as_ref().unwrap().action_spent);
    f.run(Some(0), TacticalAction::EndTurn);
    f.run(Some(1), TacticalAction::EndTurn);
    f.run(Some(0), TacticalAction::SecondWind);
    assert_eq!(
        f.rules().entities[&actor]
            .character_features
            .as_ref()
            .unwrap()
            .second_wind_remaining,
        0
    );
    let original = f.raw(&[1]);
    f.run(
        Some(0),
        TacticalAction::SubmitRollWithInspiration {
            result: original.clone(),
            die_index: 0,
            replacement: DieResult {
                sides: 10,
                value: 10,
            },
        },
    );
    assert_eq!(
        f.rules().entities[&actor].hp,
        f.rules().entities[&actor].max_hp
    );
    assert!(!f.rules().entities[&actor].heroic_inspiration);
    assert_eq!(
        f.rules().rolls.last().unwrap().original_result,
        Some(original)
    );
    f.run(Some(0), TacticalAction::EndTurn);
    f.run(Some(1), TacticalAction::EndTurn);
    f.rejected(Some(0), TacticalAction::SecondWind);
}

#[test]
fn source_grant_budget_and_incapacity_fail_before_any_cost() {
    for variation in 0..6 {
        let mut f = fighter();
        match variation {
            0 => f.state.table.as_mut().unwrap().character_profiles.clear(),
            1 => {
                f.entity_mut(0)
                    .character_features
                    .as_mut()
                    .unwrap()
                    .fighter_level = 2
            }
            2 => {
                f.entity_mut(0)
                    .character_features
                    .as_mut()
                    .unwrap()
                    .second_wind_remaining = 0
            }
            3 => {
                f.state
                    .rules
                    .as_mut()
                    .unwrap()
                    .timing
                    .as_mut()
                    .unwrap()
                    .bonus_action_spent = true
            }
            4 => f.entity_mut(0).hp = 0,
            _ => f.entity_mut(0).exhaustion = 6,
        }
        f.rejected(Some(0), TacticalAction::SecondWind);
    }
}

#[test]
fn retained_second_wind_cannot_forge_the_source_payment_or_request() {
    let mut f = fighter();
    f.run(Some(0), TacticalAction::SecondWind);
    for variation in 0..5 {
        let mut forged = f.state.clone();
        match variation {
            0 => {
                forged
                    .rules
                    .as_mut()
                    .unwrap()
                    .timing
                    .as_mut()
                    .unwrap()
                    .bonus_action_spent = false
            }
            1 => {
                forged
                    .rules
                    .as_mut()
                    .unwrap()
                    .entities
                    .get_mut(&f.actors[0])
                    .unwrap()
                    .character_features
                    .as_mut()
                    .unwrap()
                    .second_wind_remaining = 2
            }
            2 => {
                forged
                    .rules
                    .as_mut()
                    .unwrap()
                    .pending
                    .as_mut()
                    .unwrap()
                    .request
                    .modifier = 3
            }
            3 => {
                forged
                    .rules
                    .as_mut()
                    .unwrap()
                    .pending
                    .as_mut()
                    .unwrap()
                    .request
                    .dice[0]
                    .sides = 20
            }
            _ => {
                let r = forged
                    .encounter
                    .as_mut()
                    .unwrap()
                    .flow
                    .as_mut()
                    .unwrap()
                    .resolution
                    .as_mut()
                    .unwrap();
                let TacticalWorkKind::SecondWind { uses_before, .. } =
                    &mut r.pending.as_mut().unwrap().work.kind
                else {
                    panic!()
                };
                *uses_before = 1;
            }
        }
        assert!(
            validate_tactical_state(&forged).is_err(),
            "forgery {variation}"
        );
    }
}
