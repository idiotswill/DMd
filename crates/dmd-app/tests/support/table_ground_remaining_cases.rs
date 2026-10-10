//! A retained operation can expire; a current issued card is not mutable caller authority.
use super::final_helpers::*;
use super::*;

#[tokio::test]
async fn accepted_unequip_becomes_unavailable_after_real_throw_completion_but_decline_and_old_retry_remain()
 {
    let (mut f, directory, path) = Box::pin(file_fixture("retained-operation")).await;
    let target = Box::pin(setup(&mut f, &path)).await;
    let initial = state(&f).await;
    let item = daggers(&initial, f.actors[0])[0];
    Box::pin(equip_melee(&mut f, &path, target, item)).await;
    let operation = AttackEquipmentOperation::Unequip { item };
    let mut choice = throw_choice(item, target);
    choice.delivery = WeaponDelivery::Melee;
    choice.after_equipment = Some(AfterAttackEquipmentIntent::Choose);
    Box::pin(player_step(
        &mut f,
        &path,
        action(TacticalAction::Attack { choice }),
    ))
    .await;
    Box::pin(player_raw(&mut f, &path, 1)).await;
    let old_card = view(&f, &player(&f))
        .await
        .tactical
        .unwrap()
        .attack_equipment
        .unwrap();
    assert!(
        old_card
            .operations
            .iter()
            .any(|offer| offer.operation == operation)
    );
    let old_request = request(
        &f,
        player(&f),
        TableTransportInput::AttackEquipment {
            handle: old_card.key,
            choice: AttackEquipmentChoice::Apply(operation),
        },
    )
    .await;
    let old_response = Box::pin(cold_step(&mut f, &path, old_request.clone())).await;
    let carried = state(&f).await;
    assert_eq!(carried.items[&item].custody, Custody::Entity(f.actors[0]));
    assert!(
        !hands(&carried, f.actors[0])
            .hands
            .contains(&HandAssignment::Item(item))
    );
    Box::pin(next_player_turn(&mut f, &path)).await;

    let mut choice = throw_choice(item, target);
    choice.after_equipment = Some(AfterAttackEquipmentIntent::Choose);
    let attack = Box::pin(player_step(
        &mut f,
        &path,
        action(TacticalAction::Attack {
            choice: choice.clone(),
        }),
    ))
    .await;
    let pending = state(&f).await;
    let flow = pending.encounter.as_ref().unwrap().flow.as_ref().unwrap();
    let resolution = flow.resolution.as_ref().unwrap();
    assert_eq!(
        resolution.attack.as_ref().unwrap().origin.id,
        attack.command_id
    );
    assert_eq!(pending.items[&item], carried.items[&item]);
    assert_eq!(
        hands(&pending, f.actors[0]).hands[Hand::Right.index()],
        HandAssignment::Item(item)
    );
    assert!(
        pending
            .rules
            .as_ref()
            .unwrap()
            .timing
            .as_ref()
            .unwrap()
            .action_spent
    );
    let paid = flow
        .budget
        .weapon_history
        .iter()
        .find(|receipt| receipt.origin.id == attack.command_id)
        .unwrap();
    assert_eq!(paid.weapon, item);
    assert_eq!(paid.target, target);
    assert_eq!(paid.delivery, WeaponDelivery::Thrown);
    assert_eq!(
        resolution.attack.as_ref().unwrap().weapon().unwrap().choice,
        choice
    );
    assert_eq!(paid.outcome, WeaponAttackOutcome::Pending);
    Box::pin(no_after_card(&f)).await;
    let raw_request = request(&f, player(&f), face_input(view(&f, &player(&f)).await, 1)).await;
    Box::pin(cold_step(&mut f, &path, raw_request.clone())).await;

    let completed = state(&f).await;
    let encounter = completed.encounter.as_ref().unwrap();
    let flow = encounter.flow.as_ref().unwrap();
    let resolution = flow.resolution.as_ref().unwrap();
    let after = resolution.attack_after_equipment.as_ref().unwrap();
    assert!(resolution.attack.is_none());
    assert_eq!(after.cause.origin.id, attack.command_id);
    assert_eq!(after.cause.choice, choice);
    assert_eq!(after.cause.completed_by.id, raw_request.command_id);
    assert_eq!(after.cause.outcome, WeaponAttackOutcome::Miss);
    let trace = resolution.work_trace.as_ref().unwrap();
    assert_eq!(after.cause.completed_work.resolution, resolution.origin.id);
    assert!(trace.nodes.iter().any(|node| node.work.occurrence
        == after.cause.completed_work.occurrence
        && node.work.kind == TacticalWorkKind::FinishAttack));
    let ground = flow
        .ground_items
        .iter()
        .find(|ground| ground.item == item)
        .unwrap();
    assert_eq!(
        ground.position,
        encounter.participant(target).unwrap().position
    );
    assert_eq!(ground.origin.id, raw_request.command_id);
    assert_eq!(
        completed.items[&item].custody,
        Custody::Location(completed.scenes[&encounter.scene_id].location_id)
    );
    assert_eq!(completed.items[&item].owner, initial.items[&item].owner);
    assert!(
        !hands(&completed, f.actors[0])
            .hands
            .contains(&HandAssignment::Item(item))
    );
    let card = view(&f, &player(&f))
        .await
        .tactical
        .unwrap()
        .attack_equipment
        .unwrap();
    assert_ne!(card.key, old_card.key);
    assert!(card.may_decline);
    assert!(
        !card
            .operations
            .iter()
            .any(|offer| offer.operation == operation)
    );
    let ineligible = request(
        &f,
        player(&f),
        TableTransportInput::AttackEquipment {
            handle: card.key,
            choice: AttackEquipmentChoice::Apply(operation),
        },
    )
    .await;
    Box::pin(atomic_rejection(&f, ineligible)).await;
    Box::pin(atomic_retry(&f, old_request.clone(), &old_response)).await;
    let mut changed = old_request;
    changed.input = TableTransportInput::AttackEquipment {
        handle: card.key,
        choice: AttackEquipmentChoice::Decline,
    };
    Box::pin(atomic_rejection(&f, changed)).await;
    let unrelated = view(&f, &other_player(&f)).await;
    let decline = request(
        &f,
        player(&f),
        TableTransportInput::AttackEquipment {
            handle: card.key,
            choice: AttackEquipmentChoice::Decline,
        },
    )
    .await;
    Box::pin(cold_step(&mut f, &path, decline)).await;
    assert_eq!(view(&f, &other_player(&f)).await, unrelated);
    let settled = state(&f).await;
    assert_eq!(settled.items, completed.items);
    assert_eq!(hands(&settled, f.actors[0]), hands(&completed, f.actors[0]));
    assert_eq!(
        settled.rules.as_ref().unwrap().rolls,
        completed.rules.as_ref().unwrap().rolls
    );
    assert_eq!(
        settled.rules.as_ref().unwrap().timing,
        completed.rules.as_ref().unwrap().timing
    );
    assert!(
        settled
            .encounter
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .resolution
            .is_none()
    );
    Box::pin(close_fixture(f, &directory)).await;
}
