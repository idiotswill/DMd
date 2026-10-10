//! Complete audience and persistence comparisons at real equipment continuations.
use super::final_helpers::*;
use super::*;

#[path = "table_ground_privacy_maps.rs"]
mod maps;

async fn pc_selected(
    f: &mut Fixture,
    path: &Path,
    hit: bool,
) -> (EntityId, ItemId, TableTransportChannel) {
    let target = Box::pin(setup(f, path)).await;
    let item = daggers(&state(f).await, f.actors[0])[0];
    Box::pin(equip_melee(f, path, target, item)).await;
    let mut choice = throw_choice(item, target);
    choice.delivery = WeaponDelivery::Melee;
    choice.after_equipment = Some(AfterAttackEquipmentIntent::Choose);
    Box::pin(player_step(
        f,
        path,
        action(TacticalAction::Attack { choice }),
    ))
    .await;
    Box::pin(player_raw(f, path, if hit { 20 } else { 1 })).await;
    (target, item, player(f))
}

#[tokio::test]
async fn owned_and_source_after_choices_preserve_complete_unrelated_presentations() {
    for source in [false, true] {
        for apply in [false, true] {
            let (mut f, directory, path) = Box::pin(file_fixture("private-choices")).await;
            let (actor, item, channel) = if source {
                Box::pin(source_selected(&mut f, &path, false)).await
            } else {
                let (_, item, channel) = Box::pin(pc_selected(&mut f, &path, false)).await;
                (f.actors[0], item, channel)
            };
            let selected = state(&f).await;
            let owner_before = view(&f, &channel).await;
            let card = owner_before
                .tactical
                .as_ref()
                .unwrap()
                .attack_equipment
                .as_ref()
                .unwrap();
            assert_eq!(card.actor, actor);
            assert!(card.may_decline);
            let operation = AttackEquipmentOperation::Unequip { item };
            assert!(
                card.operations
                    .iter()
                    .any(|offer| offer.operation == operation)
            );
            let decision = request(
                &f,
                channel.clone(),
                TableTransportInput::AttackEquipment {
                    handle: card.key,
                    choice: if apply {
                        AttackEquipmentChoice::Apply(operation)
                    } else {
                        AttackEquipmentChoice::Decline
                    },
                },
            )
            .await;
            // Capture whole DTO after request/view bootstrap, including its private
            // transcript, revision and every issued capability, without normalization.
            let unrelated = view(&f, &other_player(&f)).await;
            assert!(
                unrelated
                    .tactical
                    .as_ref()
                    .unwrap()
                    .attack_equipment
                    .is_none()
            );
            let host = view(&f, &TableTransportChannel::Host).await;
            let accepted = Box::pin(cold_step(&mut f, &path, decision.clone())).await;
            assert_eq!(view(&f, &other_player(&f)).await, unrelated);
            let owner_after = view(&f, &channel).await;
            assert_ne!(owner_after.revision, owner_before.revision);
            assert!(
                owner_after
                    .tactical
                    .as_ref()
                    .unwrap()
                    .attack_equipment
                    .is_none()
            );
            assert_ne!(
                view(&f, &TableTransportChannel::Host).await.diagnostics,
                host.diagnostics
            );
            let settled = state(&f).await;
            assert_eq!(settled.items, selected.items);
            assert_eq!(
                settled.rules.as_ref().unwrap().rolls,
                selected.rules.as_ref().unwrap().rolls
            );
            assert_eq!(
                settled.rules.as_ref().unwrap().timing,
                selected.rules.as_ref().unwrap().timing
            );
            assert_eq!(
                hands(&settled, actor)
                    .hands
                    .contains(&HandAssignment::Item(item)),
                !apply
            );
            assert_eq!(
                settled.rules.as_ref().unwrap().tactical_creatures,
                selected.rules.as_ref().unwrap().tactical_creatures
            );
            Box::pin(atomic_retry(&f, decision, &accepted)).await;
            assert_eq!(view(&f, &other_player(&f)).await, unrelated);
            Box::pin(close_fixture(f, &directory)).await;
        }
    }
}

#[tokio::test]
async fn unavailable_ground_probes_are_equivalent_before_attack_and_at_selected_after() {
    for case in [
        maps::MapCase::Distant,
        maps::MapCase::Hidden,
        maps::MapCase::Transparent,
    ] {
        let (mut f, directory, path) = Box::pin(file_fixture("unavailable-probes")).await;
        let (target, ground, foreign_custody) =
            Box::pin(maps::inaccessible_ground(&mut f, &path, case)).await;
        let (mut independent, foreign_directory, foreign_path) =
            Box::pin(file_fixture("independent-item")).await;
        Box::pin(setup(&mut independent, &foreign_path)).await;
        let foreign_campaign_item = daggers(&state(&independent).await, independent.actors[0])[0];
        assert_ne!(independent.campaign, f.campaign);
        let before = state(&f).await;
        assert!(!before.items.contains_key(&foreign_campaign_item));
        assert_ne!(
            before.items[&foreign_custody].custody,
            Custody::Entity(f.actors[0])
        );
        let own_item = daggers(&before, f.actors[0])[0];
        let probes = [
            ItemId::new(),
            ground,
            foreign_custody,
            foreign_campaign_item,
        ];
        assert!(probes.iter().all(|item| *item != own_item));
        let mut before_errors = Vec::new();
        for item in probes {
            // The same legal declared attack differs only in its pickup ItemId.
            let mut choice = throw_choice(own_item, target);
            choice.equipment_change = Some(AttackEquipmentChange {
                timing: EquipmentChangeTiming::BeforeAttack,
                operation: AttackEquipmentOperation::Pickup {
                    item,
                    hand: Hand::Left,
                },
            });
            let invalid = request(&f, player(&f), action(TacticalAction::Attack { choice })).await;
            before_errors.push(Box::pin(atomic_rejection(&f, invalid)).await);
        }
        assert!(
            before_errors.windows(2).all(|pair| pair[0] == pair[1]),
            "same-prefix unavailable before-Pickup probes must disclose the same public denial: {before_errors:?}"
        );
        assert!(
            before_errors
                .iter()
                .all(|message| message.contains("ground equipment is unavailable"))
        );
        let mut choice = throw_choice(own_item, target);
        choice.after_equipment = Some(AfterAttackEquipmentIntent::Choose);
        Box::pin(player_step(
            &mut f,
            &path,
            action(TacticalAction::Attack { choice }),
        ))
        .await;
        Box::pin(player_raw(&mut f, &path, 1)).await;
        let card = view(&f, &player(&f))
            .await
            .tactical
            .unwrap()
            .attack_equipment
            .unwrap();
        assert!(card.may_decline);
        assert!(
            card.operations.iter().any(|offer| offer.operation
                == AttackEquipmentOperation::Pickup {
                    item: own_item,
                    hand: Hand::Right
                }),
            "a different real nearby ground Item is offered at this same cut"
        );
        let mut after_errors = Vec::new();
        for item in probes {
            assert!(!card.operations.iter().any(|offer| matches!(offer.operation, AttackEquipmentOperation::Pickup { item: offered, .. } if offered == item)));
            let invalid = request(
                &f,
                player(&f),
                TableTransportInput::AttackEquipment {
                    handle: card.key,
                    choice: AttackEquipmentChoice::Apply(AttackEquipmentOperation::Pickup {
                        item,
                        hand: Hand::Left,
                    }),
                },
            )
            .await;
            after_errors.push(Box::pin(atomic_rejection(&f, invalid)).await);
        }
        assert!(
            after_errors.windows(2).all(|pair| pair[0] == pair[1]),
            "same-prefix unavailable after-Pickup probes must disclose the same public denial: {after_errors:?}"
        );
        assert!(
            after_errors
                .iter()
                .all(|message| message.contains("ground equipment is unavailable"))
        );
        // Before and after may have different outer admission classes; do not
        // conflate them or compare them with owner/envelope denials.
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
        Box::pin(close_fixture(independent, &foreign_directory)).await;
        Box::pin(close_fixture(f, &directory)).await;
    }
}

async fn real_hit_and_damage_refusals(
    f: &mut Fixture,
    path: &Path,
    source: bool,
    target: EntityId,
    channel: &TableTransportChannel,
) {
    Box::pin(no_after_card(f)).await;
    let order = view(f, &player(f))
        .await
        .tactical
        .unwrap()
        .hit
        .unwrap()
        .order
        .unwrap();
    // This is an actual issued hit capability, not an invented future after card.
    let wrong_kind = request(
        f,
        channel.clone(),
        TableTransportInput::AttackEquipment {
            handle: order.key,
            choice: AttackEquipmentChoice::Decline,
        },
    )
    .await;
    Box::pin(atomic_rejection(f, wrong_kind)).await;
    let target_actor = if source { f.actors[0] } else { target };
    let target_channel = if source {
        player(f)
    } else {
        TableTransportChannel::Host
    };
    let before = state(f).await;
    let cause = before
        .encounter
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .resolution
        .as_ref()
        .unwrap()
        .hit_review
        .as_ref()
        .unwrap()
        .cause
        .clone();
    // Ordering belongs to the actual turn actor. In the source case that is
    // Goblin, even though the same player also controls the target character.
    Box::pin(step(
        f,
        path,
        channel.clone(),
        TableTransportInput::HitResponse {
            handle: order.key,
            decision: Box::new(TableHitInput::Order {
                instruction: TacticalReactionOrdering {
                    ranked: vec![],
                    unlisted: ReactionUnlistedOrder::AfterForward,
                },
            }),
        },
    ))
    .await;
    let response = view(f, &target_channel)
        .await
        .tactical
        .unwrap()
        .hit
        .unwrap()
        .response
        .unwrap();
    assert_eq!(response.actor, target_actor);
    assert!(!response.selected);
    Box::pin(step(
        f,
        path,
        target_channel,
        TableTransportInput::HitResponse {
            handle: response.key,
            decision: Box::new(TableHitInput::Respond { accept: false }),
        },
    ))
    .await;
    let pending = state(f).await;
    assert_eq!(
        pending.applied_event_sequence,
        before.applied_event_sequence + 2
    );
    assert_eq!(
        pending.rules.as_ref().unwrap().rolls,
        before.rules.as_ref().unwrap().rolls
    );
    assert_eq!(
        pending
            .rules
            .as_ref()
            .unwrap()
            .timing
            .as_ref()
            .unwrap()
            .reactions_spent,
        before
            .rules
            .as_ref()
            .unwrap()
            .timing
            .as_ref()
            .unwrap()
            .reactions_spent
    );
    assert_eq!(
        pending
            .rules
            .as_ref()
            .unwrap()
            .pending
            .as_ref()
            .unwrap()
            .issued_by,
        cause
    );
    let resolution = pending
        .encounter
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .resolution
        .as_ref()
        .unwrap();
    let work = resolution.pending.as_ref().unwrap();
    assert_eq!(work.key.role, TacticalRollRole::AttackDamage);
    assert_eq!(work.work.kind, TacticalWorkKind::AttackDamage);
    Box::pin(no_after_card(f)).await;
    for input in [
        TableTransportInput::AttackEquipment {
            handle: order.key,
            choice: AttackEquipmentChoice::Decline,
        },
        action(TacticalAction::ChooseAttackEquipment {
            work: TacticalWorkKey {
                resolution: resolution.origin.id,
                occurrence: work.work.occurrence,
            },
            choice: AttackEquipmentChoice::Decline,
        }),
    ] {
        let invalid = request(f, channel.clone(), input).await;
        Box::pin(atomic_rejection(f, invalid)).await;
    }
    Box::pin(hostile_activation_rows(f)).await;
    Box::pin(raw(f, path, channel.clone(), 1)).await;
}

#[tokio::test]
async fn wrong_audience_kind_revision_and_changed_envelopes_leave_every_saved_row_unchanged() {
    for source in [false, true] {
        for hit in [false, true] {
            let (mut f, directory, path) = Box::pin(file_fixture("exact-envelopes")).await;
            let old_revision = view(&f, &player(&f)).await.revision;
            let (target, item, channel) = if source {
                Box::pin(source_selected(&mut f, &path, hit)).await
            } else {
                Box::pin(pc_selected(&mut f, &path, hit)).await
            };
            if hit {
                Box::pin(real_hit_and_damage_refusals(
                    &mut f, &path, source, target, &channel,
                ))
                .await;
            }
            let owner = view(&f, &channel).await;
            let card = owner
                .tactical
                .as_ref()
                .unwrap()
                .attack_equipment
                .as_ref()
                .unwrap();
            let operation = AttackEquipmentOperation::Unequip { item };
            assert!(
                card.operations
                    .iter()
                    .any(|offer| offer.operation == operation)
            );
            let selected = state(&f).await;
            let resolution = selected
                .encounter
                .as_ref()
                .unwrap()
                .flow
                .as_ref()
                .unwrap()
                .resolution
                .as_ref()
                .unwrap();
            let after = resolution.attack_after_equipment.as_ref().unwrap();
            assert!(after.selected_by.is_some());
            let valid = request(
                &f,
                channel.clone(),
                TableTransportInput::AttackEquipment {
                    handle: card.key,
                    choice: AttackEquipmentChoice::Apply(operation),
                },
            )
            .await;
            for unauthorized in [other_player(&f), TableTransportChannel::Host] {
                let mut stolen = valid.clone();
                stolen.command_id = CommandId::new();
                stolen.revision = view(&f, &unauthorized).await.revision;
                stolen.channel = unauthorized;
                Box::pin(atomic_rejection(&f, stolen)).await;
            }
            if source {
                let mut wrong_actor = valid.clone();
                wrong_actor.command_id = CommandId::new();
                wrong_actor.channel = player(&f);
                Box::pin(atomic_rejection(&f, wrong_actor)).await;
            }
            for input in [
                TableTransportInput::SelectWork { handle: card.key },
                action(TacticalAction::ChooseAttackEquipment {
                    work: TacticalWorkKey {
                        resolution: resolution.origin.id,
                        occurrence: after.work.occurrence,
                    },
                    choice: AttackEquipmentChoice::Apply(operation),
                }),
                action(TacticalAction::EndTurn),
                action(TacticalAction::Dodge),
            ] {
                let invalid = request(&f, channel.clone(), input).await;
                Box::pin(atomic_rejection(&f, invalid)).await;
            }
            let mut stale_revision = valid.clone();
            stale_revision.command_id = CommandId::new();
            stale_revision.revision = old_revision;
            Box::pin(atomic_rejection(&f, stale_revision)).await;
            Box::pin(hostile_private_records(&f, card.key, None)).await;
            let stale_handle = request(
                &f,
                channel.clone(),
                TableTransportInput::AttackEquipment {
                    handle: card.key,
                    choice: AttackEquipmentChoice::Decline,
                },
            )
            .await;
            let accepted = Box::pin(cold_step(&mut f, &path, valid.clone())).await;
            Box::pin(atomic_retry(&f, valid.clone(), &accepted)).await;
            let mut current_with_old_handle = stale_handle;
            current_with_old_handle.revision = view(&f, &channel).await.revision;
            Box::pin(atomic_rejection(&f, current_with_old_handle)).await;
            for mutation in 0..6 {
                let mut changed = valid.clone();
                match mutation {
                    0 => {
                        changed.input = TableTransportInput::AttackEquipment {
                            handle: card.key,
                            choice: AttackEquipmentChoice::Decline,
                        }
                    }
                    1 => changed.session_id = Some(PlaySessionId::new()),
                    2 => changed.channel = other_player(&f),
                    3 => changed.revision = old_revision,
                    4 => changed.version = if valid.version == 1 { 2 } else { 1 },
                    5 => changed.command_id = CommandId::new(),
                    _ => unreachable!(),
                }
                Box::pin(atomic_rejection(&f, changed)).await;
            }
            Box::pin(hostile_private_records(
                &f,
                card.key,
                Some(valid.command_id),
            ))
            .await;
            Box::pin(close_fixture(f, &directory)).await;
        }
    }
}
