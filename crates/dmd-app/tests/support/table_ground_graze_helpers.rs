//! Bounded accepted-command helpers for the three genuine Graze families.
use super::*;

pub(super) fn resolution(state: &CampaignState) -> &TacticalResolution {
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
}
pub(super) fn weapon_choice(
    item: ItemId,
    target: EntityId,
    equip: bool,
    after: bool,
) -> WeaponUseChoice {
    assert!(
        !(equip && after),
        "before Equip and later after Choose belong to different attacks"
    );
    WeaponUseChoice {
        weapon: item,
        target,
        delivery: WeaponDelivery::Melee,
        ability: Ability::Strength,
        grip: WeaponGrip::TwoHands,
        purpose: WeaponAttackPurpose::Normal,
        ammunition: None,
        equipment_change: equip.then_some(AttackEquipmentChange {
            timing: EquipmentChangeTiming::BeforeAttack,
            operation: AttackEquipmentOperation::Equip {
                item,
                hand: Hand::Right,
            },
        }),
        after_equipment: after.then_some(AfterAttackEquipmentIntent::Choose),
    }
}
pub(super) async fn choose_graze(
    f: &mut Fixture,
    path: &Path,
    apply: bool,
) -> TableTransportRequest {
    Box::pin(player_step(
        f,
        path,
        action(TacticalAction::ChooseAttackMastery {
            choice: if apply {
                WeaponMasteryChoice::Graze
            } else {
                WeaponMasteryChoice::Decline
            },
        }),
    ))
    .await
}
pub(super) async fn equip_prior_miss(f: &mut Fixture, path: &Path, item: ItemId, target: EntityId) {
    let before = state(f).await;
    let equip = Box::pin(player_step(
        f,
        path,
        action(TacticalAction::Attack {
            choice: weapon_choice(item, target, true, false),
        }),
    ))
    .await;
    Box::pin(player_raw(f, path, 1)).await;
    assert_eq!(
        view(f, &player(f))
            .await
            .tactical
            .unwrap()
            .attack_decision
            .unwrap()
            .kind,
        TableAttackDecisionKind::Graze
    );
    Box::pin(choose_graze(f, path, false)).await;
    let after = state(f).await;
    assert_eq!(
        after.rules.as_ref().unwrap().entities[&target].hp,
        before.rules.as_ref().unwrap().entities[&target].hp
    );
    assert_eq!(
        hands(&after, f.actors[0]).hands,
        [HandAssignment::Item(item); 2]
    );
    assert_eq!(after.items, before.items);
    let flow = after.encounter.as_ref().unwrap().flow.as_ref().unwrap();
    assert!(flow.resolution.is_none());
    let receipt = flow
        .budget
        .weapon_history
        .iter()
        .find(|r| r.origin.id == equip.command_id)
        .unwrap();
    assert_eq!(receipt.outcome, WeaponAttackOutcome::Miss);
    assert!(receipt.after_equipment.is_none());
}

pub(super) struct GrazeCut {
    pub attack: TableTransportRequest,
    attack_response: TableTransportResult,
    raw: TableTransportRequest,
    raw_response: TableTransportResult,
    pub parent: AttackEquipmentCompletionParent,
    pub paused: CampaignState,
}

pub(super) async fn start_graze(
    f: &mut Fixture,
    path: &Path,
    item: ItemId,
    target: EntityId,
    face: u16,
) -> GrazeCut {
    let before = state(f).await;
    assert_eq!(
        hands(&before, f.actors[0]).hands,
        [HandAssignment::Item(item); 2]
    );
    assert!(
        !before
            .rules
            .as_ref()
            .unwrap()
            .timing
            .as_ref()
            .unwrap()
            .action_spent
    );
    let choice = weapon_choice(item, target, false, true);
    let attack = request(
        f,
        player(f),
        action(TacticalAction::Attack {
            choice: choice.clone(),
        }),
    )
    .await;
    let attack_response = Box::pin(cold_step(f, path, attack.clone())).await;
    let paid = state(f).await;
    let shown = view(f, &player(f)).await;
    let presented_roll = shown.roll.as_ref().unwrap();
    assert_eq!(presented_roll.mode, RollMode::Normal);
    assert_eq!(
        presented_roll.dice,
        vec![DieSpec {
            count: 1,
            sides: 20
        }]
    );
    let raw = request(f, player(f), physical_input(shown, &[face])).await;
    let raw_response = Box::pin(cold_step(f, path, raw.clone())).await;
    let paused = state(f).await;
    let rules = paused.rules.as_ref().unwrap();
    assert_eq!(
        rules.entities[&target].hp,
        before.rules.as_ref().unwrap().entities[&target].hp
    );
    assert_eq!(
        rules.rolls.len(),
        before.rules.as_ref().unwrap().rolls.len() + 1
    );
    assert_eq!(
        &rules.rolls[..before.rules.as_ref().unwrap().rolls.len()],
        before.rules.as_ref().unwrap().rolls.as_slice()
    );
    assert_eq!(rules.timing, paid.rules.as_ref().unwrap().timing);
    assert!(rules.pending.is_none());
    assert!(rules.timing.as_ref().unwrap().action_spent);
    assert_eq!(paused.items, before.items);
    assert_eq!(hands(&paused, f.actors[0]), hands(&before, f.actors[0]));
    let r = resolution(&paused);
    assert!(r.pending.is_none());
    assert!(
        r.attack_after_equipment.is_none(),
        "Graze has not completed this attack yet"
    );
    let attack_state = r.attack.as_ref().unwrap();
    assert_eq!(attack_state.stage, TacticalAttackStage::MasteryChoice);
    assert_eq!(attack_state.origin.id, attack.command_id);
    assert!(attack_state.damage_roll.is_none());
    let weapon = attack_state.weapon().unwrap();
    assert_eq!(weapon.choice, choice);
    let parent = weapon.after_equipment_parent.clone().unwrap();
    assert_eq!(parent.pause, AttackEquipmentPause::Graze);
    assert_eq!(parent.paused_by.id, raw.command_id);
    assert_eq!(parent.accepted_raw, attack_state.attack_roll);
    assert!(parent.accepted_raw.is_some());
    assert_eq!(parent.suspended_outcome, WeaponAttackOutcome::Miss);
    assert_eq!(parent.work.resolution, attack.command_id);
    let trace = r.work_trace.as_ref().unwrap();
    assert!(trace.active.is_none());
    assert_eq!(
        trace
            .nodes
            .iter()
            .find(|node| node.work.occurrence == parent.work.occurrence)
            .unwrap()
            .work
            .kind,
        TacticalWorkKind::FinishAttack
    );
    let accepted = rules
        .rolls
        .iter()
        .find(|row| Some(row.request.id) == parent.accepted_raw)
        .unwrap();
    assert_eq!(accepted.accepted_by.id, raw.command_id);
    assert_eq!(accepted.result.source, RollSource::Physical);
    assert_eq!(
        accepted.result.dice,
        vec![DieResult {
            sides: 20,
            value: face
        }]
    );
    let flow = paused.encounter.as_ref().unwrap().flow.as_ref().unwrap();
    let receipt = flow
        .budget
        .weapon_history
        .iter()
        .find(|row| row.origin.id == attack.command_id)
        .unwrap();
    assert_eq!(receipt.weapon, item);
    assert_eq!(receipt.target, target);
    assert_eq!(receipt.grip, WeaponGrip::TwoHands);
    assert_eq!(receipt.window, weapon.window);
    assert_eq!(receipt.outcome, WeaponAttackOutcome::Pending);
    assert!(receipt.after_equipment.is_none());
    assert_eq!(
        flow.budget.weapon_history.len(),
        before
            .encounter
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .budget
            .weapon_history
            .len()
            + 1
    );
    Box::pin(no_after_card(f)).await;
    let owner = view(f, &player(f)).await;
    assert!(owner.roll.is_none());
    assert_eq!(
        owner.tactical.unwrap().attack_decision.unwrap().kind,
        TableAttackDecisionKind::Graze
    );
    let other = view(f, &other_player(f)).await;
    assert!(other.roll.is_none());
    assert!(other.tactical.as_ref().unwrap().attack_decision.is_none());
    assert!(other.tactical.as_ref().unwrap().attack_equipment.is_none());
    // Ordinary mastery retains the existing privileged Host authority; only the
    // unrelated player is denied here. The later owned equipment card is stricter.
    let mut refusals = Vec::new();
    for choice in [WeaponMasteryChoice::Graze, WeaponMasteryChoice::Decline] {
        let wrong_owner = request(
            f,
            other_player(f),
            action(TacticalAction::ChooseAttackMastery { choice }),
        )
        .await;
        refusals.push(Box::pin(atomic_rejection(f, wrong_owner)).await);
    }
    assert_eq!(refusals[0], refusals[1]);
    // This is an actual retained FinishAttack key, never a guessed future after key.
    let raw_work = request(
        f,
        player(f),
        action(TacticalAction::ChooseAttackEquipment {
            work: parent.work,
            choice: AttackEquipmentChoice::Decline,
        }),
    )
    .await;
    Box::pin(atomic_rejection(f, raw_work)).await;
    let no_new_roll = request(f, player(f), raw.input.clone()).await;
    Box::pin(atomic_rejection(f, no_new_roll)).await;
    Box::pin(atomic_retry(f, raw.clone(), &raw_response)).await;
    GrazeCut {
        attack,
        attack_response,
        raw,
        raw_response,
        parent,
        paused,
    }
}

pub(super) async fn finish_equipment(
    f: &mut Fixture,
    path: &Path,
    item: ItemId,
    cut: &GrazeCut,
    unequip: bool,
    hostile: bool,
) {
    let selected = state(f).await;
    let rules = selected.rules.as_ref().unwrap();
    let r = resolution(&selected);
    assert!(r.attack.is_none());
    assert!(r.pending.is_none());
    assert!(rules.pending.is_none());
    let after = r.attack_after_equipment.as_ref().unwrap();
    assert_eq!(after.cause.origin.id, cut.attack.command_id);
    assert_eq!(
        after.cause.origin,
        resolution(&cut.paused).attack.as_ref().unwrap().origin
    );
    assert_eq!(after.cause.actor, f.actors[0]);
    assert_eq!(after.cause.completed_work, cut.parent.work);
    assert_eq!(after.cause.source, AttackEquipmentSource::Ordinary);
    assert_eq!(after.cause.outcome, WeaponAttackOutcome::Miss);
    let original_weapon = resolution(&cut.paused)
        .attack
        .as_ref()
        .unwrap()
        .weapon()
        .unwrap();
    assert_eq!(after.cause.choice, original_weapon.choice);
    assert_eq!(after.cause.window, original_weapon.window);
    let selected_by = after.selected_by.as_ref().unwrap();
    let cause = after.cause.clone();
    let key = TacticalWorkKey {
        resolution: cut.attack.command_id,
        occurrence: after.work.occurrence,
    };
    let card = view(f, &player(f))
        .await
        .tactical
        .unwrap()
        .attack_equipment
        .unwrap();
    assert_eq!(card.actor, f.actors[0]);
    assert!(card.may_decline);
    let operation = AttackEquipmentOperation::Unequip { item };
    assert!(
        card.operations
            .iter()
            .any(|offer| offer.operation == operation)
    );
    if hostile {
        Box::pin(hostile_private_records(f, card.key, None)).await;
    }
    assert!(
        view(f, &other_player(f))
            .await
            .tactical
            .unwrap()
            .attack_equipment
            .is_none()
    );
    let host_card = view(f, &TableTransportChannel::Host)
        .await
        .tactical
        .unwrap()
        .attack_equipment
        .unwrap();
    assert_eq!(host_card.actor, card.actor);
    assert_eq!(host_card.operations, card.operations);
    assert_eq!(host_card.may_decline, card.may_decline);
    assert_ne!(host_card.key, card.key);
    for channel in [TableTransportChannel::Host, other_player(f)] {
        let wrong_owner = request(
            f,
            channel,
            TableTransportInput::AttackEquipment {
                handle: card.key,
                choice: AttackEquipmentChoice::Decline,
            },
        )
        .await;
        Box::pin(atomic_rejection(f, wrong_owner)).await;
    }
    // Host has its own projected capability; the player-owner rule still denies it.
    let host_own_key = request(
        f,
        TableTransportChannel::Host,
        TableTransportInput::AttackEquipment {
            handle: host_card.key,
            choice: AttackEquipmentChoice::Decline,
        },
    )
    .await;
    Box::pin(atomic_rejection(f, host_own_key)).await;
    let raw_work = request(
        f,
        player(f),
        action(TacticalAction::ChooseAttackEquipment {
            work: key,
            choice: AttackEquipmentChoice::Decline,
        }),
    )
    .await;
    Box::pin(atomic_rejection(f, raw_work)).await;
    let wrong_kind = request(
        f,
        player(f),
        TableTransportInput::SelectWork { handle: card.key },
    )
    .await;
    Box::pin(atomic_rejection(f, wrong_kind)).await;
    let mut stale = request(
        f,
        player(f),
        TableTransportInput::AttackEquipment {
            handle: card.key,
            choice: AttackEquipmentChoice::Decline,
        },
    )
    .await;
    stale.revision = cut.attack.revision;
    Box::pin(atomic_rejection(f, stale)).await;
    let decision = request(
        f,
        player(f),
        TableTransportInput::AttackEquipment {
            handle: card.key,
            choice: if unequip {
                AttackEquipmentChoice::Apply(operation)
            } else {
                AttackEquipmentChoice::Decline
            },
        },
    )
    .await;
    // Public damage/conditions already settled. This comparison covers only the
    // private equipment operation, including revision, transcript and handles.
    let unrelated = view(f, &other_player(f)).await;
    let accepted = Box::pin(cold_step(f, path, decision.clone())).await;
    assert_eq!(view(f, &other_player(f)).await, unrelated);
    Box::pin(atomic_retry(f, decision.clone(), &accepted)).await;
    Box::pin(atomic_retry(f, cut.attack.clone(), &cut.attack_response)).await;
    Box::pin(atomic_retry(f, cut.raw.clone(), &cut.raw_response)).await;
    let mut changed = decision.clone();
    changed.input = TableTransportInput::AttackEquipment {
        handle: card.key,
        choice: if unequip {
            AttackEquipmentChoice::Decline
        } else {
            AttackEquipmentChoice::Apply(operation)
        },
    };
    Box::pin(atomic_rejection(f, changed)).await;
    assert_eq!(view(f, &other_player(f)).await, unrelated);
    let finished = state(f).await;
    let final_rules = finished.rules.as_ref().unwrap();
    assert_eq!(finished.items, selected.items);
    assert_eq!(final_rules.rolls, rules.rolls);
    assert_eq!(final_rules.timing, rules.timing);
    assert_eq!(final_rules.entities, rules.entities);
    assert_eq!(final_rules.tactical_effects, rules.tactical_effects);
    assert_eq!(final_rules.tactical_creatures, rules.tactical_creatures);
    assert_eq!(
        hands(&finished, f.actors[0]).hands,
        if unequip {
            [HandAssignment::Free; 2]
        } else {
            [HandAssignment::Item(item); 2]
        }
    );
    let flow = finished.encounter.as_ref().unwrap().flow.as_ref().unwrap();
    assert!(flow.resolution.is_none());
    let receipt = flow
        .budget
        .weapon_history
        .iter()
        .find(|row| row.origin.id == cut.attack.command_id)
        .unwrap();
    assert_eq!(receipt.outcome, WeaponAttackOutcome::Miss);
    let equipment = receipt.after_equipment.as_ref().unwrap();
    assert_eq!(equipment.cause, cause);
    assert_eq!(equipment.work, key);
    assert_eq!(&equipment.selected_by, selected_by);
    assert_eq!(equipment.chosen_by.id, decision.command_id);
    assert_eq!(
        equipment.applied.as_ref().map(|applied| applied.operation),
        unequip.then_some(operation)
    );
    if hostile {
        Box::pin(hostile_private_records(
            f,
            card.key,
            Some(decision.command_id),
        ))
        .await;
    }
}
