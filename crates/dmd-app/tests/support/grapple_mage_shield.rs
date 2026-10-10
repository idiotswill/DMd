//! A real accepted hit tests S-only Shield against current grip reservations.
use super::*;

#[tokio::test]
async fn one_real_mage_grip_leaves_a_free_hand_for_paid_shield_without_extra_dice() {
    Box::pin(shield_case(false, false)).await;
}

#[tokio::test]
async fn two_real_mage_grips_refuse_shield_acceptance_without_spending_a_use_or_reaction() {
    Box::pin(shield_case(true, false)).await;
}

#[tokio::test]
async fn releasing_one_mage_grip_during_unanswered_hit_restores_shield_and_keeps_original_attack() {
    Box::pin(shield_case(true, true)).await;
}

fn uses(state: &CampaignState, actor: EntityId) -> u8 {
    state
        .rules
        .as_ref()
        .unwrap()
        .tactical_creatures
        .as_ref()
        .unwrap()
        .runtime(actor)
        .unwrap()
        .limited_uses
        .iter()
        .find(|usage| usage.feature_id == "protective-magic" && usage.spell_id.is_none())
        .unwrap()
        .spent
}

async fn grip(f: &mut Fixture, other: bool) {
    let (label, channel) = if other {
        (
            "Grapple Other guard with right hand",
            TableTransportChannel::Host,
        )
    } else {
        ("Grapple Character 0 with left hand", f.pc(0))
    };
    let start = f.choose(TableTransportChannel::Host, label).await;
    Box::pin(f.cold(start)).await;
    let save = f
        .choose(channel.clone(), "Resist Grapple with Dexterity")
        .await;
    Box::pin(f.cold(save)).await;
    Box::pin(f.cold_roll(channel, 1)).await;
    let finish = f
        .choose(
            TableTransportChannel::Host,
            "Finish without changing equipment",
        )
        .await;
    Box::pin(f.cold(finish)).await;
}

async fn shield_case(two_grips: bool, release_during_hit: bool) {
    let mut f = Box::pin(Fixture::with_opponent(
        "mage",
        CreatureSize::Medium,
        two_grips,
    ))
    .await;
    Box::pin(f.activate()).await;
    let mage = f.goblin;
    let pc = f.pc(0);
    Box::pin(f.cold_action(pc.clone(), TacticalAction::EndTurn)).await;
    Box::pin(grip(&mut f, false)).await;
    if two_grips {
        Box::pin(f.cold_action(TableTransportChannel::Host, TacticalAction::EndTurn)).await;
        Box::pin(f.cold_action(TableTransportChannel::Host, TacticalAction::EndTurn)).await;
        Box::pin(f.cold_action(pc.clone(), TacticalAction::EndTurn)).await;
        Box::pin(grip(&mut f, true)).await;
    }
    Box::pin(f.cold_action(TableTransportChannel::Host, TacticalAction::EndTurn)).await;
    if two_grips {
        Box::pin(f.cold_action(TableTransportChannel::Host, TacticalAction::EndTurn)).await;
    }
    let before = f.state().await;
    let rules = before.rules.as_ref().unwrap();
    let original_grips = rules.tactical_grapples.as_ref().unwrap().active.clone();
    assert_eq!(original_grips.len(), if two_grips { 2 } else { 1 });
    let current_pin = dmd_rules::tactical_creatures::creature_source_pin(
        dmd_rules::tactical_definitions::bundled_mage_v2().unwrap(),
    )
    .unwrap();
    assert_eq!(
        rules
            .tactical_creatures
            .as_ref()
            .unwrap()
            .profile(mage)
            .unwrap()
            .source,
        current_pin
    );
    for held in &original_grips {
        assert_eq!(held.declaration.grappler, mage);
        assert_eq!(
            held.declaration.anatomy,
            GrappleAnatomyProof::Creature {
                source: current_pin.clone(),
                ordinary_hands: OrdinaryHandAnatomy::TwoHandsV1,
            }
        );
    }
    assert_eq!(rules.entities[&mage].hp, 81);
    assert_eq!(uses(&before, mage), 0);
    assert!(
        !rules
            .timing
            .as_ref()
            .unwrap()
            .reactions_spent
            .contains(&mage)
    );
    let dagger = before
        .items
        .values()
        .find(|item| item.custody == Custody::Entity(f.actors[0]) && item.definition_id == "dagger")
        .unwrap()
        .id;
    Box::pin(f.cold_action(
        pc.clone(),
        TacticalAction::Attack {
            choice: WeaponUseChoice {
                after_equipment: None,
                weapon: dagger,
                target: mage,
                delivery: WeaponDelivery::Melee,
                ability: Ability::Strength,
                grip: WeaponGrip::OneHand(Hand::Right),
                purpose: WeaponAttackPurpose::Normal,
                ammunition: None,
                equipment_change: Some(AttackEquipmentChange {
                    timing: EquipmentChangeTiming::BeforeAttack,
                    operation: AttackEquipmentOperation::Equip {
                        item: dagger,
                        hand: Hand::Right,
                    },
                }),
            },
        },
    ))
    .await;
    assert_eq!(
        f.view(pc.clone()).await.roll.unwrap().mode,
        RollMode::Normal
    );
    let reported = Box::pin(f.cold_roll(pc.clone(), 10)).await;
    let hit = f.state().await;
    let attack = resolution(&hit).attack.as_ref().unwrap().clone();
    let roll_id = attack.attack_roll.unwrap();
    let actual_roll = hit
        .rules
        .as_ref()
        .unwrap()
        .rolls
        .iter()
        .find(|roll| roll.request.id == roll_id)
        .unwrap();
    assert_eq!(actual_roll.accepted_by.id, reported.command_id);
    assert_eq!(
        actual_roll.result.dice,
        vec![DieResult {
            sides: 20,
            value: 10
        }]
    );
    assert_eq!(actual_roll.resolved.total, 15);
    let order = f
        .view(pc.clone())
        .await
        .tactical
        .unwrap()
        .hit
        .unwrap()
        .order
        .unwrap();
    let ordered = f
        .request(
            pc.clone(),
            TableTransportInput::HitResponse {
                handle: order.key,
                decision: Box::new(TableHitInput::Order {
                    instruction: TacticalReactionOrdering {
                        ranked: vec![],
                        unlisted: ReactionUnlistedOrder::AfterForward,
                    },
                }),
            },
        )
        .await;
    Box::pin(f.cold(ordered)).await;
    let response = f
        .view(TableTransportChannel::Host)
        .await
        .tactical
        .unwrap()
        .hit
        .unwrap()
        .response
        .unwrap();
    assert!(!response.selected);
    assert_eq!(response.actor, mage);
    assert_eq!(response.shield.len(), if two_grips { 0 } else { 1 });
    if two_grips {
        let rejected = f
            .request(
                TableTransportChannel::Host,
                TableTransportInput::HitResponse {
                    handle: response.key,
                    decision: Box::new(TableHitInput::Respond { accept: true }),
                },
            )
            .await;
        Box::pin(f.reject(rejected)).await;
    }
    let mut expected_grips = original_grips;
    if release_during_hit {
        let release = f
            .choose(
                TableTransportChannel::Host,
                "Release Character 0 from left hand",
            )
            .await;
        Box::pin(f.cold(release)).await;
        expected_grips.retain(|held| held.declaration.hand == Hand::Right);
        let released = f.state().await;
        assert_eq!(resolution(&released).attack.as_ref(), Some(&attack));
        assert_eq!(
            released.rules.as_ref().unwrap().rolls,
            hit.rules.as_ref().unwrap().rolls
        );
        assert_eq!(
            released
                .rules
                .as_ref()
                .unwrap()
                .tactical_grapples
                .as_ref()
                .unwrap()
                .active,
            expected_grips
        );
    }
    let cast_shield = !two_grips || release_during_hit;
    let response = f
        .view(TableTransportChannel::Host)
        .await
        .tactical
        .unwrap()
        .hit
        .unwrap()
        .response
        .unwrap();
    assert_eq!(response.shield.len(), usize::from(cast_shield));
    let respond = f
        .request(
            TableTransportChannel::Host,
            TableTransportInput::HitResponse {
                handle: response.key,
                decision: Box::new(TableHitInput::Respond {
                    accept: cast_shield,
                }),
            },
        )
        .await;
    Box::pin(f.cold(respond)).await;
    if cast_shield {
        let response = f
            .view(TableTransportChannel::Host)
            .await
            .tactical
            .unwrap()
            .hit
            .unwrap()
            .response
            .unwrap();
        assert!(response.selected);
        assert_eq!(response.shield.len(), 1);
        let choice = response.shield[0].clone();
        assert_eq!(choice.actor, mage);
        assert_eq!(choice.spell_id, "shield");
        assert_eq!(choice.material, SpellMaterialChoice::None);
        let request = f
            .request(
                TableTransportChannel::Host,
                TableTransportInput::HitResponse {
                    handle: response.key,
                    decision: Box::new(TableHitInput::Cast { choice }),
                },
            )
            .await;
        let outsider = f.view(f.pc(1)).await;
        let private = f
            .runtime
            .table_view(f.campaign, TableViewer::Player(f.players[1]))
            .await
            .unwrap();
        Box::pin(f.cold(request)).await;
        Box::pin(assert_private_history(&f, &private, &outsider)).await;
    } else {
        assert_eq!(
            f.view(pc.clone()).await.roll.unwrap().dice,
            vec![DieSpec { count: 1, sides: 4 }]
        );
        Box::pin(f.cold_roll(pc, 1)).await;
    }
    let after = f.state().await;
    let rules = after.rules.as_ref().unwrap();
    assert!(
        after
            .encounter
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .resolution
            .is_none()
    );
    assert_eq!(rules.entities[&mage].hp, if cast_shield { 81 } else { 77 });
    assert_eq!(uses(&after, mage), u8::from(cast_shield));
    assert_eq!(
        rules
            .timing
            .as_ref()
            .unwrap()
            .reactions_spent
            .contains(&mage),
        cast_shield
    );
    assert_eq!(
        rules.tactical_grapples.as_ref().unwrap().active,
        expected_grips
    );
    assert!(rules.rolls.starts_with(&hit.rules.as_ref().unwrap().rolls));
    assert_eq!(
        rules.rolls.len(),
        hit.rules.as_ref().unwrap().rolls.len() + usize::from(!cast_shield)
    );
    assert_eq!(
        rules
            .tactical_creatures
            .as_ref()
            .unwrap()
            .profile(mage)
            .unwrap()
            .source,
        current_pin
    );
    assert_eq!(after.items, before.items);
    f.close().await;
}
