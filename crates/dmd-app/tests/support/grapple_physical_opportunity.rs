//! A real held Glaive becomes usable only after its supporting hand is released.
use super::*;

#[tokio::test]
async fn actual_grip_blocks_held_glaive_reach_without_spending_a_reaction() {
    Box::pin(run(false)).await;
}

#[tokio::test]
async fn actual_release_during_another_reaction_restores_held_glaive_before_the_same_crossing() {
    Box::pin(run(true)).await;
}

fn glaive(weapon: ItemId, target: EntityId) -> WeaponUseChoice {
    WeaponUseChoice {
        weapon,
        target,
        delivery: WeaponDelivery::Melee,
        ability: Ability::Strength,
        grip: WeaponGrip::TwoHands,
        purpose: WeaponAttackPurpose::Normal,
        ammunition: None,
        equipment_change: None,
        after_equipment: None,
    }
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

fn held(state: &CampaignState, actor: EntityId) -> [HandAssignment; 2] {
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

async fn run(release_hand: bool) {
    let mut f = Box::pin(Fixture::with_physical_opportunity()).await;
    Box::pin(f.activate()).await;
    Box::pin(f.cold_action(
        TableTransportChannel::Host,
        TacticalAction::ActivateAttackEquipment,
    ))
    .await;
    let actor = f.actors[0];
    let target = f.goblin;
    let mover = f.opponent.unwrap();
    let pc = f.pc(0);
    let initial = f.state().await;
    let weapon = initial
        .items
        .values()
        .find(|item| item.custody == Custody::Entity(actor) && item.definition_id == "glaive")
        .unwrap()
        .id;
    assert_eq!(initial.items[&weapon].quantity, 1);
    let receipt = initial
        .rules
        .as_ref()
        .unwrap()
        .tactical_inventory
        .as_ref()
        .unwrap()
        .receipt(f.characters[0])
        .unwrap();
    assert_eq!(
        receipt.source.profile_id,
        "human-fighter-soldier-level-1-physical-v1"
    );
    assert!(receipt.creation_profile.creation_source.is_some());
    assert_eq!(held(&initial, actor), [HandAssignment::Free; 2]);
    for (who, expected) in [
        (actor, SpatialPoint { x: 10, y: 10, z: 0 }),
        (target, SpatialPoint { x: 20, y: 20, z: 0 }),
        (mover, SpatialPoint { x: 10, y: 30, z: 0 }),
    ] {
        assert_eq!(position(&initial, who), expected);
    }
    let encounter = initial.encounter.as_ref().unwrap();
    let distance = |a, b| {
        dmd_rules::spatial::participant_distance(
            encounter.participant(a).unwrap(),
            encounter.participant(b).unwrap(),
        )
        .unwrap()
    };
    assert_eq!(distance(actor, target), 10);
    assert_eq!(distance(actor, mover), 20);
    assert_eq!(distance(target, mover), 10);

    let attempt = f
        .choose(pc.clone(), "Grapple Small armored figure with left hand")
        .await;
    Box::pin(f.cold(attempt)).await;
    let save = f
        .choose(TableTransportChannel::Host, "Resist Grapple with Strength")
        .await;
    Box::pin(f.cold(save)).await;
    Box::pin(f.cold_roll(TableTransportChannel::Host, 1)).await;
    let equip = f
        .choose(pc.clone(), "After Grapple: equip in right hand Glaive")
        .await;
    Box::pin(f.cold(equip)).await;
    let gripped = f.state().await;
    let grip = gripped
        .rules
        .as_ref()
        .unwrap()
        .tactical_grapples
        .as_ref()
        .unwrap()
        .active[0]
        .clone();
    assert_eq!(grip.declaration.grappler, actor);
    assert_eq!(grip.declaration.target, target);
    assert_eq!(grip.declaration.hand, Hand::Left);
    assert_eq!(
        held(&gripped, actor),
        [HandAssignment::Free, HandAssignment::Item(weapon)]
    );
    Box::pin(f.cold_action(pc.clone(), TacticalAction::EndTurn)).await;
    Box::pin(f.cold_action(TableTransportChannel::Host, TacticalAction::EndTurn)).await;
    let movement = Box::pin(f.cold_action(
        TableTransportChannel::Host,
        TacticalAction::Move {
            path: vec![TacticalMoveStep {
                destination: SpatialPoint { x: 10, y: 40, z: 0 },
                mode: MovementMode::Walk,
            }],
        },
    ))
    .await;
    let paused = f.state().await;
    let original_window = resolution(&paused)
        .movement
        .as_ref()
        .unwrap()
        .opportunity
        .as_ref()
        .unwrap()
        .clone();
    assert_eq!(
        (original_window.reactor, original_window.mover),
        (target, mover)
    );
    assert_eq!(original_window.from, position(&initial, mover));
    assert_eq!(original_window.to, SpatialPoint { x: 10, y: 40, z: 0 });
    assert!(
        original_window
            .options
            .iter()
            .any(|option| option.source == TacticalMeleeSource::Unarmed)
    );
    assert_eq!(resolution(&paused).origin.id, movement.command_id);
    assert_eq!(position(&paused, mover), position(&initial, mover));
    assert_eq!(
        paused
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
    assert!(
        f.view(pc.clone())
            .await
            .tactical
            .unwrap()
            .opportunity
            .is_none()
    );
    let blocked = f
        .request(
            pc.clone(),
            action(TacticalAction::OpportunityAttack {
                choice: TacticalMeleeChoice::Weapon(glaive(weapon, mover)),
            }),
        )
        .await;
    Box::pin(f.reject(blocked)).await;
    let outsider = f.view(f.pc(1)).await;
    let private = f
        .runtime
        .table_view(f.campaign, TableViewer::Player(f.players[1]))
        .await
        .unwrap();

    if release_hand {
        let release = f
            .choose(pc.clone(), "Release Small armored figure from left hand")
            .await;
        let mut foreign = release.clone();
        foreign.command_id = CommandId::new();
        foreign.channel = f.pc(1);
        foreign.revision = outsider.revision;
        Box::pin(f.reject(foreign)).await;
        Box::pin(f.cold(release.clone())).await;
        let released = f.state().await;
        assert!(released.rules.as_ref().unwrap().tactical_grapples.is_none());
        assert_eq!(
            resolution(&released)
                .movement
                .as_ref()
                .unwrap()
                .opportunity
                .as_ref(),
            Some(&original_window)
        );
        assert!(
            resolution(&released)
                .grapple
                .as_ref()
                .unwrap()
                .ends
                .iter()
                .any(|end| {
                    end.grip == grip.declaration.id && end.caused_by.id == release.command_id
                })
        );
        assert_eq!(held(&released, actor), held(&gripped, actor));
        assert_eq!(released.items, initial.items);
        assert_eq!(
            released.rules.as_ref().unwrap().rolls,
            paused.rules.as_ref().unwrap().rolls
        );
    }
    Box::pin(f.cold_action(
        TableTransportChannel::Host,
        TacticalAction::DeclineOpportunity,
    ))
    .await;
    if !release_hand {
        let finished = f.state().await;
        assert!(
            finished
                .encounter
                .as_ref()
                .unwrap()
                .flow
                .as_ref()
                .unwrap()
                .resolution
                .is_none()
        );
        assert_eq!(
            position(&finished, mover),
            SpatialPoint { x: 10, y: 40, z: 0 }
        );
        assert_eq!(
            finished
                .rules
                .as_ref()
                .unwrap()
                .tactical_grapples
                .as_ref()
                .unwrap()
                .active,
            vec![grip]
        );
        assert!(
            finished
                .rules
                .as_ref()
                .unwrap()
                .timing
                .as_ref()
                .unwrap()
                .reactions_spent
                .is_empty()
        );
        assert_eq!(held(&finished, actor), held(&gripped, actor));
        assert_eq!(finished.items, initial.items);
        assert_eq!(
            finished.rules.as_ref().unwrap().rolls,
            paused.rules.as_ref().unwrap().rolls
        );
        Box::pin(assert_private_history(&f, &private, &outsider)).await;
        f.close().await;
        return;
    }

    let offered = f.state().await;
    assert_eq!(position(&offered, mover), position(&initial, mover));
    assert_eq!(
        offered
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
    let window = resolution(&offered)
        .movement
        .as_ref()
        .unwrap()
        .opportunity
        .as_ref()
        .unwrap()
        .clone();
    assert_eq!((window.reactor, window.mover), (actor, mover));
    assert_eq!(
        window.options,
        vec![TacticalMeleeOption {
            source: TacticalMeleeSource::Weapon { item: weapon },
            reach: 20
        }]
    );
    let card = f
        .view(pc.clone())
        .await
        .tactical
        .unwrap()
        .opportunity
        .unwrap();
    assert!(!card.unarmed);
    assert_eq!((card.actor, card.target.actor), (actor, mover));
    assert_eq!(
        card.weapons
            .unwrap()
            .weapons
            .iter()
            .find(|entry| entry.item == weapon)
            .unwrap()
            .grips,
        vec![WeaponGrip::TwoHands]
    );
    assert!(
        f.view(f.pc(1))
            .await
            .tactical
            .unwrap()
            .opportunity
            .is_none()
    );
    for invalid in [
        WeaponUseChoice {
            grip: WeaponGrip::OneHand(Hand::Right),
            ..glaive(weapon, mover)
        },
        WeaponUseChoice {
            weapon: ItemId::new(),
            ..glaive(weapon, mover)
        },
        WeaponUseChoice {
            after_equipment: Some(AfterAttackEquipmentIntent::Choose),
            ..glaive(weapon, mover)
        },
        WeaponUseChoice {
            equipment_change: Some(AttackEquipmentChange {
                timing: EquipmentChangeTiming::BeforeAttack,
                operation: AttackEquipmentOperation::Equip {
                    item: weapon,
                    hand: Hand::Right,
                },
            }),
            ..glaive(weapon, mover)
        },
    ] {
        let request = f
            .request(
                pc.clone(),
                action(TacticalAction::OpportunityAttack {
                    choice: TacticalMeleeChoice::Weapon(invalid),
                }),
            )
            .await;
        Box::pin(f.reject(request)).await;
    }
    let selected = Box::pin(f.cold_action(
        pc.clone(),
        TacticalAction::OpportunityAttack {
            choice: TacticalMeleeChoice::Weapon(glaive(weapon, mover)),
        },
    ))
    .await;
    let issued = f.state().await;
    let attack = resolution(&issued).attack.as_ref().unwrap();
    assert_eq!(attack.origin.id, selected.command_id);
    assert_eq!(
        attack.admission,
        TacticalAttackAdmission::Opportunity(Box::new(window.clone()))
    );
    assert_eq!(attack.weapon().unwrap().choice, glaive(weapon, mover));
    assert!(attack.weapon().unwrap().after_equipment_parent.is_none());
    let reads = resolution(&issued).grapple.as_ref().unwrap();
    let cut = reads
        .cuts
        .iter()
        .find(|cut| matches!(cut.key.reader, GrappleReader::OpportunityWindow { .. }))
        .unwrap();
    assert_eq!(cut.key.work.resolution, movement.command_id);
    assert_eq!(
        cut.key.reader,
        GrappleReader::OpportunityWindow {
            attack: selected.command_id,
            window: window.origin.id,
            reactor: actor,
            mover,
            step: window.step_index,
        }
    );
    assert!(
        cut.grips.is_empty(),
        "the accepted release precedes this new hand cut"
    );
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
    assert_eq!(held(&issued, actor), [HandAssignment::Item(weapon); 2]);
    let pending = issued
        .rules
        .as_ref()
        .unwrap()
        .pending
        .as_ref()
        .unwrap()
        .clone();
    assert_eq!(pending.request.mode, RollMode::Normal);
    assert_eq!(
        pending.request.dice,
        vec![DieSpec {
            count: 1,
            sides: 20
        }]
    );
    let mut forged = issued.clone();
    let cut = forged
        .encounter
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .resolution
        .as_mut()
        .unwrap()
        .grapple
        .as_mut()
        .unwrap()
        .cuts
        .iter_mut()
        .find(|cut| matches!(cut.key.reader, GrappleReader::OpportunityWindow { .. }))
        .unwrap();
    let GrappleReader::OpportunityWindow { window, .. } = &mut cut.key.reader else {
        unreachable!()
    };
    *window = CommandId::new();
    Box::pin(reject_state_image(&f, forged)).await;
    Box::pin(f.cold_roll(pc.clone(), 1)).await;
    assert_eq!(
        resolution(&f.state().await).attack.as_ref().unwrap().stage,
        TacticalAttackStage::MasteryChoice
    );
    Box::pin(f.cold_action(
        pc,
        TacticalAction::ChooseAttackMastery {
            choice: WeaponMasteryChoice::Decline,
        },
    ))
    .await;
    Box::pin(assert_private_history(&f, &private, &outsider)).await;
    let finished = f.state().await;
    let rules = finished.rules.as_ref().unwrap();
    let flow = finished.encounter.as_ref().unwrap().flow.as_ref().unwrap();
    assert!(flow.resolution.is_none());
    assert_eq!(
        position(&finished, mover),
        SpatialPoint { x: 10, y: 40, z: 0 }
    );
    assert_eq!(flow.budget.movement_spent, 10);
    assert_eq!(rules.timing.as_ref().unwrap().reactions_spent, vec![actor]);
    assert!(!rules.timing.as_ref().unwrap().action_spent);
    assert!(rules.tactical_grapples.is_none());
    assert_eq!(finished.items, initial.items);
    assert_eq!(
        rules.tactical_creatures.as_ref().unwrap().profiles,
        initial
            .rules
            .as_ref()
            .unwrap()
            .tactical_creatures
            .as_ref()
            .unwrap()
            .profiles
    );
    for source in [target, mover] {
        let old = initial
            .rules
            .as_ref()
            .unwrap()
            .tactical_creatures
            .as_ref()
            .unwrap()
            .runtime(source)
            .unwrap();
        let current = rules
            .tactical_creatures
            .as_ref()
            .unwrap()
            .runtime(source)
            .unwrap();
        assert_eq!(current.limited_uses, old.limited_uses);
        assert_eq!(current.recharge, old.recharge);
        assert_eq!(current.controller, old.controller);
        assert_eq!(current.control_origin, old.control_origin);
    }
    assert!(
        rules
            .rolls
            .starts_with(&paused.rules.as_ref().unwrap().rolls)
    );
    let raw = rules
        .rolls
        .iter()
        .filter(|raw| raw.request.id == pending.request.id)
        .collect::<Vec<_>>();
    assert_eq!(raw.len(), 1);
    assert_eq!(raw[0].result.source, RollSource::Physical);
    assert_eq!(
        raw[0].result.dice,
        vec![DieResult {
            sides: 20,
            value: 1
        }]
    );
    let receipt = flow
        .budget
        .weapon_history
        .iter()
        .find(|receipt| receipt.origin.id == selected.command_id)
        .unwrap();
    assert_eq!(receipt.weapon, weapon);
    assert!(receipt.after_equipment.is_none());
    assert_eq!(held(&finished, actor), [HandAssignment::Item(weapon); 2]);
    f.close().await;
}
