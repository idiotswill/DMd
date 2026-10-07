//! Normal current creation is the only producer of these physical weapons.
use super::physical_creation_driver as driver;
use super::*;
use dmd_rules::tactical::TacticalAction;
use driver::*;
#[path = "physical_creation_history.rs"]
mod history;

#[tokio::test]
async fn current_physical_purchases_materialize_once_and_real_graze_hit_crit_survive_cold_portable_cuts()
 {
    for definition in ["greatsword", "glaive"] {
        for (face, graze) in [(1, true), (2, true), (2, false), (12, false), (20, false)] {
            Box::pin(weapon_case(definition, face, graze)).await;
        }
    }
}
async fn weapon_case(definition: &str, face: u16, graze: bool) {
    let base_dice = if definition == "greatsword" { 2 } else { 1 };
    let damage_dice = base_dice * if face == 20 { 2 } else { 1 };
    let expected_loss = if face >= 10 {
        damage_dice + 3
    } else if graze {
        3
    } else {
        0
    };
    let path = std::env::temp_dir().join(format!(
        "dmd-current-physical-{}.sqlite",
        CommandId::new().0
    ));
    let mut f = Box::pin(blank(&path)).await;
    let creation = Box::pin(create(&mut f, &path, definition)).await;
    let weapon = Box::pin(prepare(&mut f, &path, 10)).await;
    let owner = player(&f, 0);
    let outsider = player(&f, 1);
    let current = view(&f, &owner).await;
    let offered = current.tactical.unwrap().attack_options.unwrap();
    assert!(
        offered
            .weapons
            .iter()
            .any(|entry| entry.item == weapon && entry.grips == vec![WeaponGrip::TwoHands])
    );
    assert!(
        view(&f, &outsider)
            .await
            .tactical
            .unwrap()
            .attack_options
            .is_none()
    );
    let declared = tactical(TacticalAction::Attack {
        choice: choice(&f, weapon, true),
    });
    let wrong_owner = request(&f, outsider.clone(), declared.clone()).await;
    Box::pin(reject(&f, wrong_owner)).await;
    Box::pin(step(&mut f, &path, owner.clone(), declared)).await;
    Box::pin(roll(&mut f, &path, 0, &[face])).await;
    if face < 10 {
        let pending = state(&f).await;
        assert_eq!(
            pending.rules.as_ref().unwrap().entities[&f.actors[1]].hp,
            12,
            "a miss waits for its actual owner's Graze choice"
        );
        assert_eq!(
            view(&f, &owner)
                .await
                .tactical
                .unwrap()
                .attack_decision
                .unwrap()
                .kind,
            TableAttackDecisionKind::Graze
        );
        assert!(
            view(&f, &outsider)
                .await
                .tactical
                .unwrap()
                .attack_decision
                .is_none()
        );
        let decision = tactical(TacticalAction::ChooseAttackMastery {
            choice: if graze {
                WeaponMasteryChoice::Graze
            } else {
                WeaponMasteryChoice::Decline
            },
        });
        let wrong_owner = request(&f, outsider, decision.clone()).await;
        Box::pin(reject(&f, wrong_owner)).await;
        Box::pin(step(&mut f, &path, owner, decision)).await;
    } else {
        Box::pin(decline_hit(&mut f, &path, 0)).await;
        let roll_view = view(&f, &owner).await.roll.unwrap();
        assert_eq!(
            roll_view.dice,
            vec![DieSpec {
                count: damage_dice as u16,
                sides: if definition == "greatsword" { 6 } else { 10 }
            }]
        );
        let faces = [1; 4];
        Box::pin(roll(&mut f, &path, 0, &faces[..damage_dice as usize])).await;
    }
    let final_state = state(&f).await;
    let rules = final_state.rules.as_ref().unwrap();
    assert_eq!(rules.entities[&f.actors[1]].hp, 12 - expected_loss);
    assert_eq!(
        rules.rolls.len(),
        if face >= 10 { 4 } else { 3 },
        "Graze must not fabricate a damage roll"
    );
    assert!(rules.pending.is_none());
    assert!(rules.timing.as_ref().unwrap().action_spent);
    assert!(rules.timing.as_ref().unwrap().reactions_spent.is_empty());
    assert!(
        final_state
            .encounter
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .resolution
            .is_none()
    );
    assert_eq!(final_state.items[&weapon].quantity, 1);
    assert_eq!(
        final_state.items[&weapon].custody,
        Custody::Entity(f.actors[0])
    );
    assert_eq!(
        final_state
            .items
            .values()
            .filter(|item| item.definition_id == definition)
            .count(),
        1
    );
    // An accepted pre-session creation remains exactly retryable after combat.
    let before = export_campaign(&f.pool, f.campaign).await.unwrap();
    assert!(matches!(
        Box::pin(f.runtime.submit_presented_table(creation))
            .await
            .unwrap(),
        TableTransportResult::Accepted(_)
    ));
    let mut after = export_campaign(&f.pool, f.campaign).await.unwrap();
    after.exported_at_utc = before.exported_at_utc.clone();
    assert_eq!(before, after);
    Box::pin(close(f, &path)).await;
}

#[tokio::test]
async fn current_glaive_held_at_ten_feet_uses_one_reaction_without_equipping_and_resumes_cold_movement()
 {
    Box::pin(glaive_case()).await;
}
async fn glaive_case() {
    let path =
        std::env::temp_dir().join(format!("dmd-current-glaive-{}.sqlite", CommandId::new().0));
    let mut f = Box::pin(blank(&path)).await;
    Box::pin(create(&mut f, &path, "glaive")).await;
    let weapon = Box::pin(prepare(&mut f, &path, 20)).await;
    let owner = player(&f, 0);
    let mover = player(&f, 1);
    let attack = tactical(TacticalAction::Attack {
        choice: choice(&f, weapon, true),
    });
    Box::pin(step(&mut f, &path, owner.clone(), attack)).await;
    Box::pin(roll(&mut f, &path, 0, &[1])).await;
    Box::pin(step(
        &mut f,
        &path,
        owner.clone(),
        tactical(TacticalAction::ChooseAttackMastery {
            choice: WeaponMasteryChoice::Decline,
        }),
    ))
    .await;
    Box::pin(step(
        &mut f,
        &path,
        owner.clone(),
        tactical(TacticalAction::EndTurn),
    ))
    .await;
    let before = state(&f).await;
    let held = before
        .rules
        .as_ref()
        .unwrap()
        .tactical_inventory
        .as_ref()
        .unwrap()
        .loadout(f.actors[0])
        .unwrap()
        .clone();
    assert_eq!(
        before
            .encounter
            .as_ref()
            .unwrap()
            .participant(f.actors[1])
            .unwrap()
            .position,
        point(30)
    );
    let movement = Box::pin(step(
        &mut f,
        &path,
        mover.clone(),
        tactical(TacticalAction::Move {
            path: vec![TacticalMoveStep {
                destination: point(40),
                mode: MovementMode::Walk,
            }],
        }),
    ))
    .await;
    let pending = state(&f).await;
    assert_eq!(
        pending
            .encounter
            .as_ref()
            .unwrap()
            .participant(f.actors[1])
            .unwrap()
            .position,
        point(30)
    );
    assert_eq!(
        pending
            .encounter
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .budget
            .movement_spent,
        0
    );
    let opportunity = view(&f, &owner)
        .await
        .tactical
        .unwrap()
        .opportunity
        .unwrap();
    assert_eq!(opportunity.actor, f.actors[0]);
    assert_eq!(opportunity.target.actor, f.actors[1]);
    assert!(
        opportunity
            .weapons
            .unwrap()
            .weapons
            .iter()
            .any(|entry| entry.item == weapon)
    );
    assert!(
        view(&f, &mover)
            .await
            .tactical
            .unwrap()
            .opportunity
            .is_none()
    );
    // Opportunity attacks cannot draw a weapon, even the same owned source item.
    let equip = request(
        &f,
        owner.clone(),
        tactical(TacticalAction::OpportunityAttack {
            choice: TacticalMeleeChoice::Weapon(choice(&f, weapon, true)),
        }),
    )
    .await;
    Box::pin(reject(&f, equip)).await;
    for wrong_choice in [
        WeaponUseChoice {
            grip: WeaponGrip::OneHand(Hand::Right),
            ..choice(&f, weapon, false)
        },
        WeaponUseChoice {
            weapon: ItemId::new(),
            ..choice(&f, weapon, false)
        },
    ] {
        let invalid = request(
            &f,
            owner.clone(),
            tactical(TacticalAction::OpportunityAttack {
                choice: TacticalMeleeChoice::Weapon(wrong_choice),
            }),
        )
        .await;
        Box::pin(reject(&f, invalid)).await;
    }
    let reaction = tactical(TacticalAction::OpportunityAttack {
        choice: TacticalMeleeChoice::Weapon(choice(&f, weapon, false)),
    });
    let wrong = request(&f, mover.clone(), reaction.clone()).await;
    Box::pin(reject(&f, wrong)).await;
    let mut stale = request(&f, owner.clone(), reaction.clone()).await;
    stale.revision = movement.revision;
    Box::pin(reject(&f, stale)).await;
    let reaction = Box::pin(step(&mut f, &path, owner.clone(), reaction)).await;
    let admitted = state(&f).await;
    let flow = admitted.encounter.as_ref().unwrap().flow.as_ref().unwrap();
    let resolution = flow.resolution.as_ref().unwrap();
    assert_eq!(resolution.origin.id, movement.command_id);
    assert_eq!(
        resolution.attack.as_ref().unwrap().origin.id,
        reaction.command_id
    );
    assert_eq!(
        admitted
            .rules
            .as_ref()
            .unwrap()
            .timing
            .as_ref()
            .unwrap()
            .reactions_spent,
        vec![f.actors[0]]
    );
    assert!(
        !admitted
            .rules
            .as_ref()
            .unwrap()
            .timing
            .as_ref()
            .unwrap()
            .action_spent,
        "the mover retains its Action"
    );
    // Using a held weapon keeps its equipment but records this accepted command.
    let mut expected_loadout = held;
    expected_loadout.command = CommandMeta {
        id: reaction.command_id,
        campaign_id: f.campaign,
        session_id: Some(f.session),
        issuer: CommandIssuer::Player(f.players[0]),
        actor: Some(AgentRef::Entity(f.actors[0])),
        expected_event_sequence: pending.applied_event_sequence,
    };
    assert_eq!(
        resolution.attack.as_ref().unwrap().origin,
        expected_loadout.command
    );
    assert_eq!(
        admitted
            .rules
            .as_ref()
            .unwrap()
            .tactical_inventory
            .as_ref()
            .unwrap()
            .loadout(f.actors[0])
            .unwrap(),
        &expected_loadout
    );
    Box::pin(roll(&mut f, &path, 0, &[12])).await;
    Box::pin(decline_hit(&mut f, &path, 1)).await;
    let damage_request = view(&f, &owner).await.roll.unwrap();
    assert_eq!(damage_request.mode, RollMode::Normal);
    assert_eq!(
        damage_request.dice,
        vec![DieSpec {
            count: 1,
            sides: 10
        }]
    );
    let before_damage = state(&f).await;
    let damage = Box::pin(step(
        &mut f,
        &path,
        owner,
        tactical(TacticalAction::SubmitRoll {
            result: RollResult {
                request_id: damage_request.id,
                source: RollSource::Physical,
                dice: vec![DieResult {
                    sides: 10,
                    value: 1,
                }],
            },
        }),
    ))
    .await;
    expected_loadout.command = CommandMeta {
        id: damage.command_id,
        expected_event_sequence: before_damage.applied_event_sequence,
        ..expected_loadout.command.clone()
    };
    let finished = state(&f).await;
    let flow = finished.encounter.as_ref().unwrap().flow.as_ref().unwrap();
    assert!(flow.resolution.is_none());
    assert_eq!(flow.budget.movement_spent, 10);
    assert_eq!(
        finished
            .encounter
            .as_ref()
            .unwrap()
            .participant(f.actors[1])
            .unwrap()
            .position,
        point(40)
    );
    assert_eq!(
        finished.rules.as_ref().unwrap().entities[&f.actors[1]].hp,
        8
    );
    assert_eq!(
        finished
            .rules
            .as_ref()
            .unwrap()
            .timing
            .as_ref()
            .unwrap()
            .reactions_spent,
        vec![f.actors[0]]
    );
    assert!(
        !finished
            .rules
            .as_ref()
            .unwrap()
            .timing
            .as_ref()
            .unwrap()
            .action_spent
    );
    assert_eq!(
        finished
            .rules
            .as_ref()
            .unwrap()
            .tactical_inventory
            .as_ref()
            .unwrap()
            .loadout(f.actors[0])
            .unwrap(),
        &expected_loadout
    );
    assert_eq!(finished.items, before.items);
    assert_eq!(f.runtime.replay_rules(f.campaign).await.unwrap(), finished);
    Box::pin(close(f, &path)).await;
}

#[tokio::test]
async fn current_creation_source_and_original_history_are_required_before_any_restore_writes() {
    Box::pin(history::run()).await;
}
