//! Current physical Ogre sources through an actual paired GroundDrag crossing.
//! Every material producer uses the existing cold/portable/original-replay driver.
use super::*;

#[tokio::test]
async fn current_ogre_greatclub_opportunity_keeps_its_two_hands_and_completes_the_drag() {
    Box::pin(run("greatclub", WeaponGrip::TwoHands)).await;
}

#[tokio::test]
async fn current_ogre_javelin_opportunity_keeps_its_selected_hand_and_completes_the_drag() {
    Box::pin(run("javelin-melee", WeaponGrip::OneHand(Hand::Right))).await;
}

fn active(state: &CampaignState) -> EntityId {
    let timing = state.rules.as_ref().unwrap().timing.as_ref().unwrap();
    timing.order[timing.index].actor
}

async fn run(feature: &str, selected_grip: WeaponGrip) {
    let mut f = Box::pin(Fixture::with_creation_layout_and_opposition(
        "ogre",
        CreatureSize::Large,
        true,
        true,
        Some("glaive"),
        false,
        false,
        true,
    ))
    .await;
    enable(&mut f).await;
    let holder = f.actors[0];
    let reactor = f.goblin;
    let target = f.opponent.unwrap();
    let pc = f.pc(0);
    let initial = f.state().await;
    assert_eq!(
        position(&initial, holder),
        SpatialPoint { x: 10, y: 10, z: 0 }
    );
    assert_eq!(
        position(&initial, reactor),
        SpatialPoint { x: 20, y: 10, z: 0 }
    );
    assert_eq!(
        position(&initial, target),
        SpatialPoint { x: 10, y: 20, z: 0 }
    );
    let encounter = initial.encounter.as_ref().unwrap();
    assert_eq!(
        encounter.participant(holder).unwrap().enemies,
        vec![reactor, target]
    );
    assert_eq!(
        encounter.participant(target).unwrap().enemies,
        vec![reactor, holder]
    );
    assert_eq!(
        dmd_rules::spatial::participant_distance(
            encounter.participant(reactor).unwrap(),
            encounter.participant(holder).unwrap(),
        )
        .unwrap(),
        10
    );
    physical_weapon(&f, &initial, "glaive");
    let profile = initial
        .rules
        .as_ref()
        .unwrap()
        .tactical_creatures
        .as_ref()
        .unwrap()
        .profile(reactor)
        .unwrap()
        .clone();
    assert_eq!(
        profile.source,
        dmd_rules::tactical_creatures::creature_source_pin(
            dmd_rules::tactical_definitions::bundled_ogre().unwrap()
        )
        .unwrap()
    );
    assert_eq!(
        initial
            .encounter
            .as_ref()
            .unwrap()
            .participant(reactor)
            .unwrap()
            .size,
        CreatureSize::Large
    );
    let definition = if feature == "greatclub" {
        "greatclub"
    } else {
        "javelin"
    };
    let mut weapons = initial
        .items
        .values()
        .filter(|item| item.custody == Custody::Entity(reactor) && item.definition_id == definition)
        .map(|item| item.id)
        .collect::<Vec<_>>();
    weapons.sort_by_key(|id| id.0);
    let item = weapons[0];
    assert_eq!(initial.items[&item].quantity, 1);
    assert_eq!(hands(&initial, reactor), [HandAssignment::Free; 2]);

    // Reach the real Ogre turn without changing the existing fixture's order.
    for _ in 0..3 {
        let actor = active(&f.state().await);
        if actor == reactor {
            break;
        }
        assert!(actor == holder || actor == target);
        let channel = if actor == holder {
            pc.clone()
        } else {
            TableTransportChannel::Host
        };
        Box::pin(act(&mut f, channel, TacticalAction::EndTurn)).await;
    }
    assert_eq!(active(&f.state().await), reactor);
    Box::pin(act(
        &mut f,
        TableTransportChannel::Host,
        TacticalAction::CreatureWeaponAttack {
            feature_id: feature.into(),
            choice: CreatureWeaponUseChoice {
                weapon: item,
                target: holder,
                grip: selected_grip,
                ammunition: None,
                equipment_change: Some(AttackEquipmentChange {
                    timing: EquipmentChangeTiming::BeforeAttack,
                    operation: AttackEquipmentOperation::Equip {
                        item,
                        hand: Hand::Right,
                    },
                }),
                after_equipment: None,
            },
        },
    ))
    .await;
    let own_attack = f.state().await;
    assert!(
        own_attack
            .rules
            .as_ref()
            .unwrap()
            .timing
            .as_ref()
            .unwrap()
            .action_spent
    );
    assert_eq!(
        resolution(&own_attack)
            .attack
            .as_ref()
            .unwrap()
            .weapon()
            .unwrap()
            .choice
            .weapon,
        item
    );
    Box::pin(raw(&mut f, TableTransportChannel::Host, 1)).await;
    assert!(flow(&f.state().await).resolution.is_none());
    Box::pin(act(
        &mut f,
        TableTransportChannel::Host,
        TacticalAction::EndTurn,
    ))
    .await;
    assert_eq!(active(&f.state().await), holder);

    Box::pin(choose(
        &mut f,
        pc.clone(),
        "Grapple Other guard with left hand",
    ))
    .await;
    Box::pin(choose(
        &mut f,
        TableTransportChannel::Host,
        "Resist Grapple with Strength",
    ))
    .await;
    Box::pin(raw(&mut f, TableTransportChannel::Host, 1)).await;
    Box::pin(choose(
        &mut f,
        pc.clone(),
        "Finish without changing equipment",
    ))
    .await;
    let before = f.state().await;
    let grip = before
        .rules
        .as_ref()
        .unwrap()
        .tactical_grapples
        .as_ref()
        .unwrap()
        .active[0]
        .clone();
    assert_eq!(
        (
            grip.declaration.grappler,
            grip.declaration.target,
            grip.declaration.hand
        ),
        (holder, target, Hand::Left)
    );
    let movement = Box::pin(drag(&mut f, pc.clone(), holder, vec![step(0, 10)])).await;
    let paused = f.state().await;
    let r = resolution(&paused);
    let window = r
        .movement
        .as_ref()
        .unwrap()
        .opportunity
        .as_ref()
        .unwrap()
        .clone();
    assert_eq!(
        (window.reactor, window.mover, window.step_index),
        (reactor, holder, 0)
    );
    assert_eq!(window.from, position(&before, holder));
    assert_eq!(window.to, SpatialPoint { x: 0, y: 10, z: 0 });
    assert!(window.options.iter().any(|option| option.reach == 10
        && option.source
            == TacticalMeleeSource::CreatureWeapon {
                feature_id: feature.into(),
                item,
            }));
    let offered = f
        .view(TableTransportChannel::Host)
        .await
        .tactical
        .unwrap()
        .opportunity
        .unwrap();
    assert_eq!(offered.actor, reactor);
    assert!(
        offered
            .physical_source_weapons
            .iter()
            .any(|option| option.item == item
                && option.feature_id == feature
                && option.grips.contains(&selected_grip))
    );
    let history = r
        .grapple
        .as_ref()
        .unwrap()
        .transport
        .as_ref()
        .unwrap()
        .clone();
    assert!(history.steps.is_empty());
    assert_eq!(history.admission.origin.id, movement.command_id);
    assert_eq!(
        (history.admission.holder, history.admission.target),
        (holder, target)
    );
    assert_eq!(flow(&paused).budget.movement_spent, 0);
    assert_eq!(position(&paused, holder), position(&before, holder));
    assert_eq!(position(&paused, target), position(&before, target));
    let choice = TacticalMeleeChoice::CreatureWeapon {
        feature_id: feature.into(),
        weapon: item,
        grip: selected_grip,
    };
    let wrong_grip = match selected_grip {
        WeaponGrip::TwoHands => WeaponGrip::OneHand(Hand::Right),
        _ => WeaponGrip::OneHand(Hand::Left),
    };
    let foreign_item = physical_weapon(&f, &paused, "glaive");
    for wrong in [
        TacticalMeleeChoice::CreatureWeapon {
            feature_id: "javelin-thrown".into(),
            weapon: item,
            grip: selected_grip,
        },
        TacticalMeleeChoice::CreatureWeapon {
            feature_id: feature.into(),
            weapon: foreign_item,
            grip: selected_grip,
        },
        TacticalMeleeChoice::CreatureWeapon {
            feature_id: feature.into(),
            weapon: item,
            grip: wrong_grip,
        },
        TacticalMeleeChoice::CreatureFeature {
            feature_id: feature.into(),
            weapon: Some(item),
        },
    ] {
        let request = request(
            &f,
            TableTransportChannel::Host,
            action(TacticalAction::OpportunityAttack { choice: wrong }),
        )
        .await;
        Box::pin(f.reject(request)).await;
    }
    for channel in [pc.clone(), f.pc(1)] {
        let request = request(
            &f,
            channel,
            action(TacticalAction::OpportunityAttack {
                choice: choice.clone(),
            }),
        )
        .await;
        Box::pin(f.reject(request)).await;
    }
    assert_eq!(f.state().await, paused);
    let accepted = Box::pin(act(
        &mut f,
        TableTransportChannel::Host,
        TacticalAction::OpportunityAttack { choice },
    ))
    .await;
    let issued = f.state().await;
    let attack = resolution(&issued).attack.as_ref().unwrap();
    assert_eq!(attack.origin.id, accepted.command_id);
    assert_eq!((attack.actor, attack.target), (reactor, holder));
    assert_eq!(
        attack.admission,
        TacticalAttackAdmission::Opportunity(Box::new(window))
    );
    let TacticalAttackSource::CreatureWeapon {
        source,
        feature_id,
        weapon,
    } = &attack.source
    else {
        panic!("physical source opportunity was rewritten to another attack kind")
    };
    assert_eq!(source, &profile.source);
    assert_eq!(feature_id, feature);
    assert_eq!(
        (
            weapon.choice.weapon,
            weapon.choice.target,
            weapon.choice.grip
        ),
        (item, holder, selected_grip)
    );
    assert_eq!(weapon.window.kind, WeaponActionKind::Reaction);
    assert!(
        weapon.choice.equipment_change.is_none()
            && weapon.choice.after_equipment.is_none()
            && weapon.choice.ammunition.is_none()
            && weapon.after_equipment_parent.is_none()
            && weapon.ground_pickup_before.is_none()
    );
    assert_eq!(issued.items, before.items);
    assert_eq!(
        resolution(&issued)
            .grapple
            .as_ref()
            .unwrap()
            .transport
            .as_ref(),
        Some(&history)
    );
    let timing = issued.rules.as_ref().unwrap().timing.as_ref().unwrap();
    assert_eq!(timing.reactions_spent, vec![reactor]);
    assert_eq!(
        timing.action_spent,
        paused
            .rules
            .as_ref()
            .unwrap()
            .timing
            .as_ref()
            .unwrap()
            .action_spent
    );
    assert_eq!(
        flow(&issued).budget.attacks_remaining,
        flow(&paused).budget.attacks_remaining
    );
    let pending = issued
        .rules
        .as_ref()
        .unwrap()
        .pending
        .as_ref()
        .unwrap()
        .clone();
    assert_eq!(pending.request.roller, Some(reactor));
    assert_eq!(
        pending.request.dice,
        vec![DieSpec {
            count: 1,
            sides: 20
        }]
    );
    assert_eq!(pending.request.mode, RollMode::Normal);
    assert_eq!(
        issued.rules.as_ref().unwrap().rolls,
        paused.rules.as_ref().unwrap().rolls
    );

    // Hostile images are rejected through the actual restore/replay boundary.
    for mutation in 0..4 {
        let mut forged = issued.clone();
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
        let TacticalAttackSource::CreatureWeapon {
            source,
            feature_id,
            weapon,
        } = &mut attack.source
        else {
            unreachable!()
        };
        match mutation {
            0 => source.definition_fingerprint.push('x'),
            1 => *feature_id = "javelin-thrown".into(),
            2 => weapon.choice.weapon = foreign_item,
            3 => weapon.choice.grip = wrong_grip,
            _ => unreachable!(),
        }
        Box::pin(reject_state_image(&f, forged)).await;
    }
    let reported = Box::pin(raw(&mut f, TableTransportChannel::Host, 1)).await;
    let after = f.state().await;
    assert!(flow(&after).resolution.is_none());
    let result = flow(&after).last_movement.as_ref().unwrap();
    assert_eq!(result.original.id, movement.command_id);
    assert_eq!(result.reason, TacticalMovementEnd::Completed);
    assert_eq!(
        (
            result.completed_steps,
            result.requested_steps,
            result.spent_after
        ),
        (1, 1, 20)
    );
    assert_eq!(result.endpoint, SpatialPoint { x: 0, y: 10, z: 0 });
    assert_eq!(
        result.transport,
        Some(GrappleTransportResult {
            kind: GrappleTransportKind::GroundDragV1,
            grip: grip.declaration.id,
            target,
            target_start: position(&before, target),
            target_endpoint: SpatialPoint { x: 0, y: 20, z: 0 },
            ordinary_cost: 10,
            haul_cost: 10,
        })
    );
    assert_eq!(position(&after, holder), result.endpoint);
    assert_eq!(
        position(&after, target),
        result.transport.as_ref().unwrap().target_endpoint
    );
    assert_eq!(position(&after, reactor), position(&initial, reactor));
    let encounter = after.encounter.as_ref().unwrap();
    assert_eq!(
        dmd_rules::spatial::participant_distance(
            encounter.participant(reactor).unwrap(),
            encounter.participant(holder).unwrap(),
        )
        .unwrap(),
        20
    );
    assert_eq!(after.items, initial.items);
    assert_eq!(hands(&after, reactor), hands(&issued, reactor));
    let rules = after.rules.as_ref().unwrap();
    assert_eq!(
        rules.timing.as_ref().unwrap().reactions_spent,
        vec![reactor]
    );
    assert_eq!(rules.tactical_grapples.as_ref().unwrap().active, vec![grip]);
    assert_eq!(
        rules.rolls.len(),
        paused.rules.as_ref().unwrap().rolls.len() + 1
    );
    let recorded = rules.rolls.last().unwrap();
    assert_eq!(recorded.request, pending.request);
    assert_eq!(recorded.accepted_by.id, reported.command_id);
    assert_eq!(
        recorded.result.dice,
        vec![DieResult {
            sides: 20,
            value: 1
        }]
    );
    assert_eq!(recorded.result.source, RollSource::Physical);
    let receipt = flow(&after)
        .budget
        .weapon_history
        .iter()
        .find(|receipt| receipt.origin.id == accepted.command_id)
        .unwrap();
    assert_eq!(
        (
            receipt.weapon,
            receipt.target,
            receipt.grip,
            receipt.outcome
        ),
        (item, holder, selected_grip, WeaponAttackOutcome::Miss)
    );
    assert_eq!(receipt.window.kind, WeaponActionKind::Reaction);
    f.close().await;
}
