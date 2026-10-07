//! Real current purchases, an incoming grip, Graze and owned equipment completion.
use super::*;

#[tokio::test]
async fn current_greatsword_graze_under_a_real_grip_keeps_paid_equipment_parent_and_cold_history() {
    for apply_and_stow in [false, true] {
        Box::pin(run("greatsword", apply_and_stow)).await;
    }
}

#[tokio::test]
async fn current_glaive_graze_under_a_real_grip_keeps_paid_equipment_parent_and_cold_history() {
    for apply_and_stow in [false, true] {
        Box::pin(run("glaive", apply_and_stow)).await;
    }
}

fn choice(weapon: ItemId, target: EntityId, equip: bool) -> WeaponUseChoice {
    WeaponUseChoice {
        weapon,
        target,
        delivery: WeaponDelivery::Melee,
        ability: Ability::Strength,
        grip: WeaponGrip::TwoHands,
        purpose: WeaponAttackPurpose::Normal,
        ammunition: None,
        equipment_change: equip.then_some(AttackEquipmentChange {
            timing: EquipmentChangeTiming::BeforeAttack,
            operation: AttackEquipmentOperation::Equip {
                item: weapon,
                hand: Hand::Right,
            },
        }),
        after_equipment: (!equip).then_some(AfterAttackEquipmentIntent::Choose),
    }
}

fn hands(state: &CampaignState, actor: EntityId) -> [HandAssignment; 2] {
    state
        .rules
        .as_ref()
        .unwrap()
        .tactical_inventory
        .as_ref()
        .unwrap()
        .loadout(actor)
        .unwrap()
        .hands
        .hands
}

async fn run(definition: &str, apply_and_stow: bool) {
    let mut f = Box::pin(Fixture::with_physical_weapon(definition)).await;
    Box::pin(f.activate()).await;
    let activation = Box::pin(f.cold_action(
        TableTransportChannel::Host,
        TacticalAction::ActivateAttackEquipment,
    ))
    .await;
    let original = f.state().await;
    assert_eq!(
        original
            .encounter
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .attack_equipment_access
            .as_ref()
            .unwrap()
            .origin
            .id,
        activation.command_id
    );
    let actor = f.actors[0];
    let target = f.goblin;
    let physical = original
        .items
        .values()
        .filter(|item| item.custody == Custody::Entity(actor) && item.definition_id == definition)
        .collect::<Vec<_>>();
    assert_eq!(physical.len(), 1);
    assert_eq!(physical[0].quantity, 1);
    let weapon = physical[0].id;
    let inventory = original
        .rules
        .as_ref()
        .unwrap()
        .tactical_inventory
        .as_ref()
        .unwrap();
    let receipt = inventory.receipt(f.characters[0]).unwrap();
    assert_eq!(
        receipt.source.profile_id,
        "human-fighter-soldier-level-1-physical-v1"
    );
    assert_eq!(
        &receipt.creation_profile,
        &original.table.as_ref().unwrap().character_profiles[&f.characters[0]]
    );
    assert!(receipt.creation_profile.creation_source.is_some());
    assert_eq!(hands(&original, actor), [HandAssignment::Free; 2]);
    let pc = f.pc(0);
    Box::pin(f.cold_action(
        pc.clone(),
        TacticalAction::Attack {
            choice: choice(weapon, target, true),
        },
    ))
    .await;
    Box::pin(f.cold_roll(pc.clone(), 1)).await;
    Box::pin(f.cold_action(
        pc.clone(),
        TacticalAction::ChooseAttackMastery {
            choice: WeaponMasteryChoice::Decline,
        },
    ))
    .await;
    assert_eq!(f.state().await.rules.unwrap().entities[&target].hp, 10);
    assert_eq!(
        hands(&f.state().await, actor),
        [HandAssignment::Item(weapon); 2]
    );
    Box::pin(f.cold_action(pc.clone(), TacticalAction::EndTurn)).await;
    let start = f
        .choose(
            TableTransportChannel::Host,
            "Grapple Character 0 with right hand",
        )
        .await;
    Box::pin(f.cold(start)).await;
    let save = f.choose(pc.clone(), "Resist Grapple with Dexterity").await;
    Box::pin(f.cold(save)).await;
    Box::pin(f.cold_roll(pc.clone(), 1)).await;
    let finish = f
        .choose(
            TableTransportChannel::Host,
            "Finish without changing equipment",
        )
        .await;
    Box::pin(f.cold(finish)).await;
    Box::pin(f.cold_action(TableTransportChannel::Host, TacticalAction::EndTurn)).await;
    let before = f.state().await;
    let live = before
        .rules
        .as_ref()
        .unwrap()
        .tactical_grapples
        .as_ref()
        .unwrap()
        .active
        .clone();
    assert_eq!(live.len(), 1);
    assert_eq!(live[0].declaration.grappler, target);
    assert_eq!(live[0].declaration.target, actor);
    let attack = Box::pin(f.cold_action(
        pc.clone(),
        TacticalAction::Attack {
            choice: choice(weapon, target, false),
        },
    ))
    .await;
    assert_eq!(
        f.view(pc.clone()).await.roll.unwrap().mode,
        RollMode::Normal
    );
    let reported = Box::pin(f.cold_roll(pc.clone(), 1)).await;
    let paused = f.state().await;
    let r = resolution(&paused);
    let current = r.attack.as_ref().unwrap();
    assert_eq!(current.stage, TacticalAttackStage::MasteryChoice);
    assert!(current.damage_roll.is_none());
    assert!(r.attack_after_equipment.is_none());
    let parent = current
        .weapon()
        .unwrap()
        .after_equipment_parent
        .clone()
        .unwrap();
    assert_eq!(parent.pause, AttackEquipmentPause::Graze);
    assert_eq!(parent.paused_by.id, reported.command_id);
    assert_eq!(parent.accepted_raw, current.attack_roll);
    assert_eq!(parent.work.resolution, attack.command_id);
    assert!(
        r.grapple
            .as_ref()
            .unwrap()
            .cuts
            .iter()
            .any(|cut| cut.key.reader
                == GrappleReader::AttackAdmission {
                    attack: attack.command_id
                }
                && cut.grips.contains(&live[0].declaration.id))
    );
    assert_eq!(paused.rules.as_ref().unwrap().entities[&target].hp, 10);
    assert_eq!(
        paused.rules.as_ref().unwrap().rolls.len(),
        before.rules.as_ref().unwrap().rolls.len() + 1
    );
    assert_eq!(
        f.view(pc.clone())
            .await
            .tactical
            .unwrap()
            .attack_decision
            .unwrap()
            .kind,
        TableAttackDecisionKind::Graze
    );
    let graze = Box::pin(f.cold_action(
        pc.clone(),
        TacticalAction::ChooseAttackMastery {
            choice: if apply_and_stow {
                WeaponMasteryChoice::Graze
            } else {
                WeaponMasteryChoice::Decline
            },
        },
    ))
    .await;
    let selected = f.state().await;
    let r = resolution(&selected);
    assert!(r.attack.is_none());
    let after = r.attack_after_equipment.as_ref().unwrap();
    assert_eq!(after.cause.origin.id, attack.command_id);
    assert_eq!(after.cause.completed_work, parent.work);
    assert_eq!(after.cause.completed_by.id, graze.command_id);
    assert_eq!(after.selected_by.as_ref().unwrap().id, graze.command_id);
    assert_eq!(after.cause.outcome, WeaponAttackOutcome::Miss);
    assert_eq!(
        selected.rules.as_ref().unwrap().entities[&target].hp,
        if apply_and_stow { 7 } else { 10 }
    );
    assert_eq!(
        selected.rules.as_ref().unwrap().rolls,
        paused.rules.as_ref().unwrap().rolls
    );
    assert_eq!(
        selected
            .rules
            .as_ref()
            .unwrap()
            .tactical_grapples
            .as_ref()
            .unwrap()
            .active,
        live
    );
    assert_eq!(hands(&selected, actor), [HandAssignment::Item(weapon); 2]);
    let card = f
        .view(pc.clone())
        .await
        .tactical
        .unwrap()
        .attack_equipment
        .unwrap();
    let operation = AttackEquipmentOperation::Unequip { item: weapon };
    assert!(
        card.operations
            .iter()
            .any(|offer| offer.operation == operation)
    );
    for channel in [TableTransportChannel::Host, f.pc(1)] {
        let wrong = f
            .request(
                channel,
                TableTransportInput::AttackEquipment {
                    handle: card.key,
                    choice: AttackEquipmentChoice::Decline,
                },
            )
            .await;
        Box::pin(f.reject(wrong)).await;
    }
    let mut forged = selected.clone();
    forged
        .encounter
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .resolution
        .as_mut()
        .unwrap()
        .attack_after_equipment
        .as_mut()
        .unwrap()
        .cause
        .completed_by
        .id = CommandId::new();
    Box::pin(reject_state_image(&f, forged)).await;
    let decision = f
        .request(
            pc,
            TableTransportInput::AttackEquipment {
                handle: card.key,
                choice: if apply_and_stow {
                    AttackEquipmentChoice::Apply(operation)
                } else {
                    AttackEquipmentChoice::Decline
                },
            },
        )
        .await;
    let outsider = f.view(f.pc(1)).await;
    let private = f
        .runtime
        .table_view(f.campaign, TableViewer::Player(f.players[1]))
        .await
        .unwrap();
    Box::pin(f.cold(decision.clone())).await;
    Box::pin(assert_private_history(&f, &private, &outsider)).await;
    let mut changed = decision.clone();
    changed.input = TableTransportInput::AttackEquipment {
        handle: card.key,
        choice: if apply_and_stow {
            AttackEquipmentChoice::Decline
        } else {
            AttackEquipmentChoice::Apply(operation)
        },
    };
    Box::pin(f.reject(changed)).await;
    let final_state = f.state().await;
    let rules = final_state.rules.as_ref().unwrap();
    let flow = final_state
        .encounter
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap();
    assert!(flow.resolution.is_none());
    let record = flow
        .budget
        .weapon_history
        .iter()
        .find(|receipt| receipt.origin.id == attack.command_id)
        .unwrap();
    assert_eq!(record.weapon, weapon);
    let finished = record.after_equipment.as_ref().unwrap();
    assert_eq!(finished.cause, after.cause);
    assert_eq!(finished.chosen_by.id, decision.command_id);
    assert_eq!(
        finished.applied.as_ref().map(|change| change.operation),
        apply_and_stow.then_some(operation)
    );
    assert_eq!(final_state.items, original.items);
    assert_eq!(rules.rolls, selected.rules.as_ref().unwrap().rolls);
    assert_eq!(rules.tactical_grapples.as_ref().unwrap().active, live);
    assert_eq!(
        hands(&final_state, actor),
        if apply_and_stow {
            [HandAssignment::Free; 2]
        } else {
            [HandAssignment::Item(weapon); 2]
        }
    );
    f.close().await;
}
