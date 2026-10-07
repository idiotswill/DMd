//! Current physical creation, Ground, paired movement and owned Inspiration.
//! Setup and every material continuation use accepted commands and original replay.
use super::*;

#[path = "grapple_combined_ogre.rs"]
mod ogre;

fn flow(state: &CampaignState) -> &TacticalFlow {
    state.encounter.as_ref().unwrap().flow.as_ref().unwrap()
}
fn position(state: &CampaignState, actor: EntityId) -> SpatialPoint {
    state
        .encounter
        .as_ref()
        .unwrap()
        .participant(actor)
        .unwrap()
        .position
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
fn physical_weapon(f: &Fixture, state: &CampaignState, definition: &str) -> ItemId {
    let inventory = state
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
        &state.table.as_ref().unwrap().character_profiles[&f.characters[0]]
    );
    assert!(receipt.creation_profile.creation_source.is_some());
    let items = state
        .items
        .values()
        .filter(|item| {
            item.custody == Custody::Entity(f.actors[0]) && item.definition_id == definition
        })
        .collect::<Vec<_>>();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].quantity, 1);
    items[0].id
}
async fn request(
    f: &Fixture,
    channel: TableTransportChannel,
    input: TableTransportInput,
) -> TableTransportRequest {
    let mut request = f.request(channel, input).await;
    request.version = 4;
    request
}
async fn submit(
    f: &mut Fixture,
    channel: TableTransportChannel,
    input: TableTransportInput,
) -> TableTransportRequest {
    let request = request(f, channel, input).await;
    Box::pin(f.cold(request.clone())).await;
    request
}
async fn act(
    f: &mut Fixture,
    channel: TableTransportChannel,
    input: TacticalAction,
) -> TableTransportRequest {
    Box::pin(submit(f, channel, action(input))).await
}
async fn choose(
    f: &mut Fixture,
    channel: TableTransportChannel,
    label: &str,
) -> TableTransportRequest {
    let mut request = f.choose(channel, label).await;
    request.version = 4;
    Box::pin(f.cold(request.clone())).await;
    request
}
async fn raw(f: &mut Fixture, channel: TableTransportChannel, value: u16) -> TableTransportRequest {
    let roll = f.view(channel.clone()).await.roll.unwrap();
    let dice = roll
        .dice
        .iter()
        .flat_map(|die| {
            std::iter::repeat_n(
                DieResult {
                    sides: die.sides,
                    value,
                },
                if roll.mode == RollMode::Normal {
                    usize::from(die.count)
                } else {
                    2
                },
            )
        })
        .collect();
    Box::pin(act(
        f,
        channel,
        TacticalAction::SubmitRoll {
            result: RollResult {
                request_id: roll.id,
                source: RollSource::Physical,
                dice,
            },
        },
    ))
    .await
}
async fn enable(f: &mut Fixture) -> (TableTransportRequest, TableTransportRequest) {
    Box::pin(f.activate()).await;
    let transport = Box::pin(submit(
        f,
        TableTransportChannel::Host,
        TableTransportInput::Action(Box::new(TableAction::EnableGrappleTransport)),
    ))
    .await;
    let ground = Box::pin(act(
        f,
        TableTransportChannel::Host,
        TacticalAction::ActivateAttackEquipment,
    ))
    .await;
    let state = f.state().await;
    assert_eq!(
        state
            .table
            .as_ref()
            .unwrap()
            .grapple_access
            .as_ref()
            .unwrap()
            .ground_transport
            .as_ref()
            .unwrap()
            .origin
            .id,
        transport.command_id
    );
    assert_eq!(
        flow(&state)
            .attack_equipment_access
            .as_ref()
            .unwrap()
            .origin
            .id,
        ground.command_id
    );
    assert_eq!(f.view(f.pc(0)).await.grapple.unwrap().version, 4);
    (transport, ground)
}
fn step(x: i32, y: i32) -> TacticalMoveStep {
    TacticalMoveStep {
        destination: SpatialPoint { x, y, z: 0 },
        mode: MovementMode::Walk,
    }
}
async fn drag(
    f: &mut Fixture,
    channel: TableTransportChannel,
    holder: EntityId,
    path: Vec<TacticalMoveStep>,
) -> TableTransportRequest {
    let view = f.view(channel.clone()).await.grapple.unwrap();
    assert_eq!(view.version, 4);
    let options = view
        .ground_drag
        .iter()
        .filter(|offer| offer.actor == holder)
        .collect::<Vec<_>>();
    assert_eq!(options.len(), 1);
    Box::pin(submit(
        f,
        channel,
        TableTransportInput::MoveGrappled {
            option: options[0].key,
            path,
        },
    ))
    .await
}
async fn reject_other_channels(f: &Fixture, input: TableTransportInput) {
    for channel in [TableTransportChannel::Host, f.pc(1)] {
        let wrong = request(f, channel, input.clone()).await;
        let rows = all_rows(&f.pool).await;
        let state = f.state().await;
        Box::pin(f.reject(wrong)).await;
        assert_eq!(all_rows(&f.pool).await, rows);
        assert_eq!(f.state().await, state);
    }
}
fn dagger(item: ItemId, target: EntityId, delivery: WeaponDelivery) -> WeaponUseChoice {
    WeaponUseChoice {
        weapon: item,
        target,
        delivery,
        ability: Ability::Strength,
        grip: WeaponGrip::OneHand(Hand::Right),
        purpose: WeaponAttackPurpose::Normal,
        ammunition: None,
        equipment_change: None,
        after_equipment: Some(AfterAttackEquipmentIntent::Choose),
    }
}
async fn holding_dagger() -> (Fixture, GrappleId, ItemId) {
    let mut f = Box::pin(Fixture::with_physical_weapon("glaive")).await;
    enable(&mut f).await;
    let initial = f.state().await;
    let item = physical_weapon(&f, &initial, "dagger");
    let pc = f.pc(0);
    Box::pin(choose(
        &mut f,
        pc.clone(),
        "Grapple Small armored figure with left hand",
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
        "After Grapple: equip in right hand Dagger",
    ))
    .await;
    let state = f.state().await;
    let grip = &state
        .rules
        .as_ref()
        .unwrap()
        .tactical_grapples
        .as_ref()
        .unwrap()
        .active[0]
        .declaration;
    assert_eq!(
        (grip.grappler, grip.target, grip.hand),
        (f.actors[0], f.goblin, Hand::Left)
    );
    let id = grip.id;
    assert_eq!(
        hands(&state, f.actors[0]),
        [HandAssignment::Free, HandAssignment::Item(item)]
    );
    assert_eq!(state.items, initial.items);
    Box::pin(act(&mut f, pc, TacticalAction::EndTurn)).await;
    Box::pin(act(
        &mut f,
        TableTransportChannel::Host,
        TacticalAction::EndTurn,
    ))
    .await;
    (f, id, item)
}

#[tokio::test]
async fn physical_glaive_opportunity_during_drag_retains_paid_pair_and_both_cold_endpoints() {
    let mut f = Box::pin(Fixture::with_physical_opportunity()).await;
    enable(&mut f).await;
    let initial = f.state().await;
    let weapon = physical_weapon(&f, &initial, "glaive");
    let pc = f.pc(0);
    let actor = f.actors[0];
    // The outer guard begins outside unarmed reach. Using the nearer guard as
    // holder would provoke an earlier, different unarmed crossing on step zero.
    let holder = f.opponent.unwrap();
    let target = f.goblin;
    let mut use_weapon = WeaponUseChoice {
        weapon,
        target,
        delivery: WeaponDelivery::Melee,
        ability: Ability::Strength,
        grip: WeaponGrip::TwoHands,
        purpose: WeaponAttackPurpose::Normal,
        ammunition: None,
        equipment_change: Some(AttackEquipmentChange {
            timing: EquipmentChangeTiming::BeforeAttack,
            operation: AttackEquipmentOperation::Equip {
                item: weapon,
                hand: Hand::Right,
            },
        }),
        after_equipment: None,
    };
    Box::pin(act(
        &mut f,
        pc.clone(),
        TacticalAction::Attack {
            choice: use_weapon.clone(),
        },
    ))
    .await;
    Box::pin(raw(&mut f, pc.clone(), 1)).await;
    Box::pin(act(
        &mut f,
        pc.clone(),
        TacticalAction::ChooseAttackMastery {
            choice: WeaponMasteryChoice::Decline,
        },
    ))
    .await;
    assert_eq!(
        hands(&f.state().await, actor),
        [HandAssignment::Item(weapon); 2]
    );
    Box::pin(act(&mut f, pc.clone(), TacticalAction::EndTurn)).await;
    Box::pin(act(
        &mut f,
        TableTransportChannel::Host,
        TacticalAction::EndTurn,
    ))
    .await;
    Box::pin(choose(
        &mut f,
        TableTransportChannel::Host,
        "Grapple Small armored figure with right hand",
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
        TableTransportChannel::Host,
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
        .declaration
        .clone();
    assert_eq!((grip.grappler, grip.target), (holder, target));
    assert_eq!(
        position(&before, holder),
        SpatialPoint { x: 10, y: 30, z: 0 }
    );
    assert_eq!(
        position(&before, target),
        SpatialPoint { x: 20, y: 20, z: 0 }
    );
    let movement = Box::pin(drag(
        &mut f,
        TableTransportChannel::Host,
        holder,
        vec![step(20, 30), step(20, 40)],
    ))
    .await;
    let paused = f.state().await;
    let r = resolution(&paused);
    let moving = r.movement.as_ref().unwrap();
    assert_eq!(moving.next_step, 1);
    assert_eq!(flow(&paused).budget.movement_spent, 20);
    assert_eq!(
        position(&paused, holder),
        SpatialPoint { x: 20, y: 30, z: 0 }
    );
    assert_eq!(
        position(&paused, target),
        SpatialPoint { x: 30, y: 20, z: 0 }
    );
    let window = moving.opportunity.as_ref().unwrap().clone();
    assert_eq!(
        (window.reactor, window.mover, window.step_index),
        (actor, holder, 1)
    );
    assert_eq!(
        window.options,
        vec![TacticalMeleeOption {
            source: TacticalMeleeSource::Weapon { item: weapon },
            reach: 20,
        }]
    );
    let history = r
        .grapple
        .as_ref()
        .unwrap()
        .transport
        .as_ref()
        .unwrap()
        .clone();
    assert_eq!(history.admission.origin.id, movement.command_id);
    assert_eq!(
        (
            history.admission.grip,
            history.admission.holder,
            history.admission.target
        ),
        (grip.id, holder, target)
    );
    assert_eq!(history.steps.len(), 1);
    let paid = &history.steps[0];
    assert_eq!(
        paid.holder,
        GrappleBodyDisplacement {
            actor: holder,
            from: position(&before, holder),
            to: position(&paused, holder),
        }
    );
    assert_eq!(
        paid.target,
        GrappleBodyDisplacement {
            actor: target,
            from: position(&before, target),
            to: position(&paused, target),
        }
    );
    assert_eq!(
        (paid.ordinary_cost, paid.haul_cost, paid.total_cost()),
        (10, 10, Some(20))
    );
    assert_eq!(paid.work.resolution, movement.command_id);
    assert_eq!(paid.cause.id, movement.command_id);
    assert_eq!(moving.traversed.len(), 1);
    assert_eq!(moving.traversed[0].cause, paid.cause);
    assert_eq!(moving.traversed[0].from, paid.holder.from);
    assert_eq!(moving.traversed[0].to, paid.holder.to);
    assert_eq!(moving.traversed[0].cost, paid.total_cost().unwrap());
    assert!(
        paused
            .rules
            .as_ref()
            .unwrap()
            .timing
            .as_ref()
            .unwrap()
            .reactions_spent
            .is_empty()
    );
    use_weapon.equipment_change = None;
    use_weapon.target = holder;
    let input = action(TacticalAction::OpportunityAttack {
        choice: TacticalMeleeChoice::Weapon(use_weapon.clone()),
    });
    reject_other_channels(&f, input.clone()).await;
    let attack = Box::pin(submit(&mut f, pc.clone(), input)).await;
    let issued = f.state().await;
    let current = resolution(&issued).attack.as_ref().unwrap();
    assert_eq!(current.origin.id, attack.command_id);
    assert_eq!(current.weapon().unwrap().choice, use_weapon);
    assert_eq!(
        current.admission,
        TacticalAttackAdmission::Opportunity(Box::new(window))
    );
    assert_eq!(
        resolution(&issued)
            .grapple
            .as_ref()
            .unwrap()
            .transport
            .as_ref()
            .unwrap(),
        &history
    );
    assert_eq!(position(&issued, holder), position(&paused, holder));
    assert_eq!(position(&issued, target), position(&paused, target));
    assert_eq!(
        issued
            .rules
            .as_ref()
            .unwrap()
            .timing
            .as_ref()
            .unwrap()
            .reactions_spent,
        vec![actor]
    );
    let pending = issued
        .rules
        .as_ref()
        .unwrap()
        .pending
        .as_ref()
        .unwrap()
        .clone();
    assert_eq!(pending.request.roller, Some(actor));
    assert_eq!(pending.request.mode, RollMode::Normal);
    let raw = Box::pin(raw(&mut f, pc.clone(), 1)).await;
    let mastery = f.state().await;
    assert_eq!(
        resolution(&mastery).attack.as_ref().unwrap().stage,
        TacticalAttackStage::MasteryChoice
    );
    assert_eq!(
        resolution(&mastery)
            .grapple
            .as_ref()
            .unwrap()
            .transport
            .as_ref()
            .unwrap(),
        &history
    );
    Box::pin(act(
        &mut f,
        pc,
        TacticalAction::ChooseAttackMastery {
            choice: WeaponMasteryChoice::Decline,
        },
    ))
    .await;
    let after = f.state().await;
    assert!(flow(&after).resolution.is_none());
    let result = flow(&after).last_movement.as_ref().unwrap();
    assert_eq!(result.original.id, movement.command_id);
    assert_eq!(
        (
            result.completed_steps,
            result.requested_steps,
            result.spent_after
        ),
        (2, 2, 40)
    );
    assert_eq!(result.reason, TacticalMovementEnd::Completed);
    assert_eq!(flow(&after).budget.movement_spent, 40);
    assert_eq!(result.endpoint, SpatialPoint { x: 20, y: 40, z: 0 });
    assert_eq!(
        result.transport,
        Some(GrappleTransportResult {
            kind: GrappleTransportKind::GroundDragV1,
            grip: grip.id,
            target,
            target_start: position(&before, target),
            target_endpoint: SpatialPoint { x: 30, y: 30, z: 0 },
            ordinary_cost: 20,
            haul_cost: 20,
        })
    );
    assert_eq!(position(&after, holder), result.endpoint);
    assert_eq!(
        position(&after, target),
        result.transport.as_ref().unwrap().target_endpoint
    );
    assert_eq!(after.items, before.items);
    assert_eq!(hands(&after, actor), [HandAssignment::Item(weapon); 2]);
    let rules = after.rules.as_ref().unwrap();
    assert_eq!(
        rules.tactical_grapples.as_ref().unwrap().active,
        before
            .rules
            .as_ref()
            .unwrap()
            .tactical_grapples
            .as_ref()
            .unwrap()
            .active
    );
    assert_eq!(rules.timing.as_ref().unwrap().reactions_spent, vec![actor]);
    assert_eq!(
        rules.rolls.len(),
        before.rules.as_ref().unwrap().rolls.len() + 1
    );
    let recorded = rules.rolls.last().unwrap();
    assert_eq!(recorded.request, pending.request);
    assert_eq!(recorded.accepted_by.id, raw.command_id);
    assert_eq!(
        recorded.result.dice,
        vec![DieResult {
            sides: 20,
            value: 1
        }]
    );
    let receipt = flow(&after)
        .budget
        .weapon_history
        .iter()
        .find(|receipt| receipt.origin.id == attack.command_id)
        .unwrap();
    assert_eq!(
        (receipt.weapon, receipt.target, receipt.outcome),
        (weapon, holder, WeaponAttackOutcome::Miss)
    );
    assert_eq!(receipt.window.kind, WeaponActionKind::Reaction);
    let rows = all_rows(&f.pool).await;
    Box::pin(f.runtime.submit_presented_table(movement))
        .await
        .unwrap();
    assert_eq!(all_rows(&f.pool).await, rows);
    f.close().await;
}

#[tokio::test]
async fn v4_ground_after_equipment_retains_real_drag_and_owned_cold_decision() {
    let (mut f, grip, item) = Box::pin(holding_dagger()).await;
    let pc = f.pc(0);
    let actor = f.actors[0];
    let target = f.goblin;
    let movement = Box::pin(drag(
        &mut f,
        pc.clone(),
        actor,
        vec![step(20, 10), step(30, 10)],
    ))
    .await;
    let moved = f.state().await;
    assert_eq!(
        flow(&moved).last_movement.as_ref().unwrap().original.id,
        movement.command_id
    );
    assert_eq!(
        flow(&moved)
            .last_movement
            .as_ref()
            .unwrap()
            .transport
            .as_ref()
            .unwrap()
            .grip,
        grip
    );
    let attack = Box::pin(act(
        &mut f,
        pc.clone(),
        TacticalAction::Attack {
            choice: dagger(item, target, WeaponDelivery::Melee),
        },
    ))
    .await;
    let paid = f.state().await;
    assert!(
        paid.rules
            .as_ref()
            .unwrap()
            .timing
            .as_ref()
            .unwrap()
            .action_spent
    );
    let pending = paid
        .rules
        .as_ref()
        .unwrap()
        .pending
        .as_ref()
        .unwrap()
        .clone();
    let raw = Box::pin(raw(&mut f, pc.clone(), 1)).await;
    let selected = f.state().await;
    let after_equipment = resolution(&selected)
        .attack_after_equipment
        .as_ref()
        .unwrap()
        .clone();
    assert_eq!(after_equipment.cause.origin.id, attack.command_id);
    assert_eq!(after_equipment.cause.completed_by.id, raw.command_id);
    assert_eq!(after_equipment.cause.outcome, WeaponAttackOutcome::Miss);
    assert_eq!(after_equipment.cause.choice.weapon, item);
    assert_eq!(after_equipment.cause.actor, actor);
    assert_eq!(flow(&selected).last_movement, flow(&moved).last_movement);
    let shown = f.view(pc.clone()).await;
    assert_eq!(shown.grapple.unwrap().version, 4);
    let card = shown.tactical.unwrap().attack_equipment.unwrap();
    let operation = AttackEquipmentOperation::Unequip { item };
    assert!(
        card.operations
            .iter()
            .any(|offer| offer.operation == operation)
    );
    let input = TableTransportInput::AttackEquipment {
        handle: card.key,
        choice: AttackEquipmentChoice::Apply(operation),
    };
    reject_other_channels(&f, input.clone()).await;
    let command = Box::pin(submit(&mut f, pc, input)).await;
    let final_state = f.state().await;
    assert!(flow(&final_state).resolution.is_none());
    assert_eq!(flow(&final_state).last_movement, flow(&moved).last_movement);
    assert_eq!(hands(&final_state, actor), [HandAssignment::Free; 2]);
    assert!(
        final_state
            .rules
            .as_ref()
            .unwrap()
            .tactical_grapples
            .as_ref()
            .unwrap()
            .grip(grip)
            .is_some()
    );
    assert_eq!(final_state.items, moved.items);
    let receipt = flow(&final_state)
        .budget
        .weapon_history
        .iter()
        .find(|receipt| receipt.origin.id == attack.command_id)
        .unwrap()
        .after_equipment
        .as_ref()
        .unwrap();
    assert_eq!(receipt.cause, after_equipment.cause);
    assert_eq!(receipt.selected_by, after_equipment.selected_by.unwrap());
    assert_eq!(
        receipt.work,
        TacticalWorkKey {
            resolution: attack.command_id,
            occurrence: after_equipment.work.occurrence,
        }
    );
    assert_eq!(receipt.chosen_by.id, command.command_id);
    assert_eq!(receipt.applied.as_ref().unwrap().operation, operation);
    let recorded = final_state.rules.as_ref().unwrap().rolls.last().unwrap();
    assert_eq!(recorded.request, pending.request);
    assert_eq!(recorded.accepted_by.id, raw.command_id);
    assert_eq!(
        recorded.result.dice,
        vec![DieResult {
            sides: 20,
            value: 1
        }]
    );
    f.close().await;
}

#[tokio::test]
async fn displaced_target_drop_and_owned_pickup_use_final_position_and_only_unreserved_hand() {
    let (mut f, grip, item) = Box::pin(holding_dagger()).await;
    let pc = f.pc(0);
    let actor = f.actors[0];
    let target = f.goblin;
    let initial = f.state().await;
    let initial_item = initial.items[&item].clone();
    let movement = Box::pin(drag(
        &mut f,
        pc.clone(),
        actor,
        vec![step(20, 10), step(30, 10), step(40, 10)],
    ))
    .await;
    let moved = f.state().await;
    assert_eq!(position(&moved, actor), SpatialPoint { x: 40, y: 10, z: 0 });
    assert_eq!(
        position(&moved, target),
        SpatialPoint { x: 50, y: 10, z: 0 }
    );
    assert_eq!(
        flow(&moved).last_movement.as_ref().unwrap().original.id,
        movement.command_id
    );
    assert_eq!(flow(&moved).budget.movement_spent, 60);
    let attack = Box::pin(act(
        &mut f,
        pc.clone(),
        TacticalAction::Attack {
            choice: dagger(item, target, WeaponDelivery::Thrown),
        },
    ))
    .await;
    let paid = f.state().await;
    let pending = paid
        .rules
        .as_ref()
        .unwrap()
        .pending
        .as_ref()
        .unwrap()
        .clone();
    assert_eq!(pending.request.mode, RollMode::Disadvantage);
    let raw = Box::pin(raw(&mut f, pc.clone(), 1)).await;
    let dropped = f.state().await;
    assert_eq!(flow(&dropped).ground_items.len(), 1);
    let ground = flow(&dropped).ground_items[0].clone();
    assert_eq!(ground.item, item);
    assert_eq!(ground.origin.id, raw.command_id);
    assert_eq!(ground.position, position(&moved, target));
    assert_ne!(ground.position, position(&initial, target));
    assert_ne!(ground.position, position(&initial, actor));
    let mut stale_position = dropped.clone();
    stale_position
        .encounter
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .ground_items[0]
        .position = position(&initial, target);
    Box::pin(reject_state_image(&f, stale_position)).await;
    let scene = &dropped.scenes[&dropped.encounter.as_ref().unwrap().scene_id];
    assert_eq!(
        dropped.items[&item].custody,
        Custody::Location(scene.location_id)
    );
    assert_eq!(dropped.items[&item].owner, initial_item.owner);
    assert_eq!(dropped.items[&item].quantity, initial_item.quantity);
    assert_eq!(hands(&dropped, actor), [HandAssignment::Free; 2]);
    assert!(
        dropped
            .rules
            .as_ref()
            .unwrap()
            .tactical_grapples
            .as_ref()
            .unwrap()
            .grip(grip)
            .is_some()
    );
    let card = f
        .view(pc.clone())
        .await
        .tactical
        .unwrap()
        .attack_equipment
        .unwrap();
    let right = AttackEquipmentOperation::Pickup {
        item,
        hand: Hand::Right,
    };
    let left = AttackEquipmentOperation::Pickup {
        item,
        hand: Hand::Left,
    };
    assert!(card.operations.iter().any(|offer| offer.operation == right));
    assert!(!card.operations.iter().any(|offer| offer.operation == left));
    let wrong_hand = request(
        &f,
        pc.clone(),
        TableTransportInput::AttackEquipment {
            handle: card.key,
            choice: AttackEquipmentChoice::Apply(left),
        },
    )
    .await;
    let rows = all_rows(&f.pool).await;
    Box::pin(f.reject(wrong_hand)).await;
    assert_eq!(all_rows(&f.pool).await, rows);
    let input = TableTransportInput::AttackEquipment {
        handle: card.key,
        choice: AttackEquipmentChoice::Apply(right),
    };
    reject_other_channels(&f, input.clone()).await;
    let pickup = Box::pin(submit(&mut f, pc, input)).await;
    let after = f.state().await;
    assert!(flow(&after).ground_items.is_empty());
    assert_eq!(after.items[&item], initial_item);
    assert_eq!(after.items, initial.items);
    assert_eq!(
        hands(&after, actor),
        [HandAssignment::Free, HandAssignment::Item(item)]
    );
    assert_eq!(flow(&after).last_movement, flow(&moved).last_movement);
    assert_eq!(
        after
            .rules
            .as_ref()
            .unwrap()
            .tactical_grapples
            .as_ref()
            .unwrap()
            .active,
        initial
            .rules
            .as_ref()
            .unwrap()
            .tactical_grapples
            .as_ref()
            .unwrap()
            .active
    );
    let receipt = flow(&after)
        .budget
        .weapon_history
        .iter()
        .find(|receipt| receipt.origin.id == attack.command_id)
        .unwrap()
        .after_equipment
        .as_ref()
        .unwrap();
    assert_eq!(receipt.chosen_by.id, pickup.command_id);
    assert_eq!(receipt.cause.origin.id, attack.command_id);
    assert_eq!(receipt.cause.completed_by.id, raw.command_id);
    assert_eq!(receipt.cause.outcome, WeaponAttackOutcome::Miss);
    assert_eq!(
        receipt.cause.choice,
        dagger(item, target, WeaponDelivery::Thrown)
    );
    let applied = receipt.applied.as_ref().unwrap();
    assert_eq!(applied.operation, right);
    assert_eq!(applied.ground_before.as_ref().unwrap().ground, ground);
    assert_eq!(
        applied.ground_before.as_ref().unwrap().item,
        dropped.items[&item]
    );
    let recorded = after.rules.as_ref().unwrap().rolls.last().unwrap();
    assert_eq!(recorded.request, pending.request);
    assert_eq!(
        recorded.result.dice,
        vec![
            DieResult {
                sides: 20,
                value: 1
            };
            2
        ]
    );
    f.close().await;
}

async fn award(f: &mut Fixture) -> TableTransportRequest {
    let character_id = f.characters[0];
    Box::pin(submit(
        f,
        TableTransportChannel::Host,
        TableTransportInput::Action(Box::new(TableAction::AwardHeroicInspiration {
            character_id,
            reason: "For protecting the retreating group.".into(),
        })),
    ))
    .await
}
async fn reroll(f: &Fixture, original: u16, replacement: u16) -> TableTransportRequest {
    let shown = f.view(f.pc(0)).await.roll.unwrap();
    assert_eq!(shown.mode, RollMode::Normal);
    assert_eq!(
        shown.dice,
        vec![DieSpec {
            count: 1,
            sides: 20
        }]
    );
    request(
        f,
        f.pc(0),
        action(TacticalAction::SubmitRollWithInspiration {
            result: RollResult {
                request_id: shown.id,
                source: RollSource::Physical,
                dice: vec![DieResult {
                    sides: 20,
                    value: original,
                }],
            },
            die_index: 0,
            replacement: DieResult {
                sides: 20,
                value: replacement,
            },
        }),
    )
    .await
}
fn assert_inspiration_roll(
    f: &Fixture,
    before: &CampaignState,
    after: &CampaignState,
    command: &TableTransportRequest,
    original: u16,
    replacement: u16,
    modifier: i32,
) {
    let pending = before.rules.as_ref().unwrap().pending.as_ref().unwrap();
    assert!(before.rules.as_ref().unwrap().entities[&f.actors[0]].heroic_inspiration);
    let rules = after.rules.as_ref().unwrap();
    let recorded = rules.rolls.last().unwrap();
    assert_eq!(recorded.request, pending.request);
    assert_eq!(recorded.issued_by, pending.issued_by);
    assert_eq!(recorded.purpose, pending.purpose);
    assert_eq!(recorded.request.modifier, modifier);
    assert_eq!(recorded.accepted_by.id, command.command_id);
    assert_eq!(
        recorded.accepted_by.issuer,
        CommandIssuer::Player(f.players[0])
    );
    assert_eq!(
        recorded.original_result,
        Some(RollResult {
            request_id: pending.request.id,
            source: RollSource::Physical,
            dice: vec![DieResult {
                sides: 20,
                value: original
            }],
        })
    );
    assert_eq!(recorded.result.request_id, pending.request.id);
    assert_eq!(recorded.result.source, RollSource::Physical);
    assert_eq!(
        recorded.result.dice,
        vec![DieResult {
            sides: 20,
            value: replacement
        }]
    );
    assert_eq!(recorded.resolved.total, i32::from(replacement) + modifier);
    assert_eq!(
        rules.rolls.len(),
        before.rules.as_ref().unwrap().rolls.len() + 1
    );
    assert!(!rules.entities[&f.actors[0]].heroic_inspiration);
    assert!(!rules.cancelled_roll_ids.contains(&pending.request.id));
}

#[tokio::test]
async fn physical_source_pc_host_awards_pay_owned_save_and_escape_rerolls_without_foreign_writes() {
    let mut f = Box::pin(Fixture::with_physical_weapon("glaive")).await;
    enable(&mut f).await;
    let initial = f.state().await;
    physical_weapon(&f, &initial, "glaive");
    let award_save = Box::pin(award(&mut f)).await;
    let awarded = f.state().await;
    assert!(awarded.rules.as_ref().unwrap().entities[&f.actors[0]].heroic_inspiration);
    assert_eq!(
        awarded
            .rules
            .as_ref()
            .unwrap()
            .rulings
            .last()
            .unwrap()
            .command
            .id,
        award_save.command_id
    );
    assert!(!awarded.rules.as_ref().unwrap().entities[&f.actors[1]].heroic_inspiration);
    let pc = f.pc(0);
    Box::pin(act(&mut f, pc.clone(), TacticalAction::EndTurn)).await;
    Box::pin(choose(
        &mut f,
        TableTransportChannel::Host,
        "Grapple Character 0 with right hand",
    ))
    .await;
    Box::pin(choose(&mut f, pc.clone(), "Resist Grapple with Strength")).await;
    let saving = f.state().await;
    let pending = saving.rules.as_ref().unwrap().pending.as_ref().unwrap();
    assert_eq!(pending.request.roller, Some(f.actors[0]));
    assert!(
        matches!(pending.purpose, PendingPurpose::TacticalResolution { key, .. } if key.role == TacticalRollRole::GrappleSave)
    );
    let failed = reroll(&f, 20, 1).await;
    reject_other_channels(&f, failed.input.clone()).await;
    Box::pin(f.cold(failed.clone())).await;
    let gripped = f.state().await;
    assert_inspiration_roll(&f, &saving, &gripped, &failed, 20, 1, 5);
    let grip = gripped
        .rules
        .as_ref()
        .unwrap()
        .tactical_grapples
        .as_ref()
        .unwrap()
        .active[0]
        .declaration
        .clone();
    assert_eq!(
        (grip.grappler, grip.target, grip.escape_dc),
        (f.goblin, f.actors[0], 9)
    );
    Box::pin(choose(
        &mut f,
        TableTransportChannel::Host,
        "Finish without changing equipment",
    ))
    .await;
    let award_escape = Box::pin(award(&mut f)).await;
    let reawarded = f.state().await;
    assert!(reawarded.rules.as_ref().unwrap().entities[&f.actors[0]].heroic_inspiration);
    assert_eq!(
        reawarded
            .rules
            .as_ref()
            .unwrap()
            .rulings
            .last()
            .unwrap()
            .command
            .id,
        award_escape.command_id
    );
    Box::pin(act(
        &mut f,
        TableTransportChannel::Host,
        TacticalAction::EndTurn,
    ))
    .await;
    let escape = f
        .view(pc.clone())
        .await
        .grapple
        .unwrap()
        .choices
        .into_iter()
        .find(|offer| offer.actor == f.actors[0] && offer.label.ends_with("using Acrobatics"))
        .unwrap();
    Box::pin(submit(
        &mut f,
        pc,
        TableTransportInput::GrappleChoice { handle: escape.key },
    ))
    .await;
    let escaping = f.state().await;
    assert!(
        escaping
            .rules
            .as_ref()
            .unwrap()
            .timing
            .as_ref()
            .unwrap()
            .action_spent
    );
    let pending = escaping.rules.as_ref().unwrap().pending.as_ref().unwrap();
    assert!(
        matches!(pending.purpose, PendingPurpose::TacticalResolution { key, .. } if key.role == TacticalRollRole::GrappleEscape)
    );
    let success = reroll(&f, 1, 20).await;
    reject_other_channels(&f, success.input.clone()).await;
    Box::pin(f.cold(success.clone())).await;
    let after = f.state().await;
    assert_inspiration_roll(&f, &escaping, &after, &success, 1, 20, 4);
    assert!(after.rules.as_ref().unwrap().tactical_grapples.is_none());
    assert!(
        after
            .rules
            .as_ref()
            .unwrap()
            .timing
            .as_ref()
            .unwrap()
            .action_spent
    );
    assert_eq!(after.items, initial.items);
    assert_eq!(
        after.table.as_ref().unwrap().character_profiles,
        initial.table.as_ref().unwrap().character_profiles
    );
    assert_eq!(hands(&after, f.actors[0]), hands(&initial, f.actors[0]));
    let rows = all_rows(&f.pool).await;
    for accepted in [award_save, failed, award_escape, success] {
        Box::pin(f.runtime.submit_presented_table(accepted))
            .await
            .unwrap();
    }
    assert_eq!(all_rows(&f.pool).await, rows);
    assert!(!f.state().await.rules.unwrap().entities[&f.actors[0]].heroic_inspiration);
    f.close().await;
}
