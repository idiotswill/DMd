//! Actual purchased equipment, source Goblin, hit responses and physical dice.
//! No saved-state mutation supplies a turn, feature use or opportunity window.
use super::*;

struct FirstEncounter {
    goblin: EntityId,
    dagger: ItemId,
    completion: TacticalCompletion,
    rolls: Vec<RecordedRoll>,
    savage_request: TableTransportRequest,
    savage_response: TableTransportResult,
}

fn marker(state: &CampaignState, actor: EntityId) -> Option<u64> {
    state.rules.as_ref().unwrap().entities[&actor]
        .character_features
        .as_ref()
        .unwrap()
        .savage_attacker_turn
}

fn dagger_choice(dagger: ItemId, target: EntityId) -> WeaponUseChoice {
    WeaponUseChoice {
        weapon: dagger,
        target,
        delivery: WeaponDelivery::Melee,
        ability: Ability::Strength,
        grip: WeaponGrip::OneHand(Hand::Right),
        purpose: WeaponAttackPurpose::Normal,
        ammunition: None,
        equipment_change: None,
    }
}

async fn accept(
    f: &mut Fixture,
    url: &str,
    directory: &Path,
    label: &str,
    player: Option<usize>,
    action: TableAction,
) -> (TableTransportRequest, TableTransportResult) {
    let request = request(f, player, action).await;
    let accepted = Box::pin(durable_step(f, url, directory, label, request.clone())).await;
    (request, accepted)
}

async fn savage_roll(
    f: &mut Fixture,
    url: &str,
    directory: &Path,
    label: &str,
    dice: u16,
    turn: u64,
) -> (TableTransportRequest, TableTransportResult) {
    let presented = view(f, Some(0)).await;
    let pending = presented.roll.unwrap();
    assert_eq!(pending.reason, "Attack damage");
    assert_eq!(pending.mode, RollMode::Normal);
    assert_eq!(
        pending.dice,
        vec![DieSpec {
            count: dice,
            sides: 4
        }]
    );
    let before = Box::new(state(f).await);
    assert_eq!(
        before
            .rules
            .as_ref()
            .unwrap()
            .timing
            .as_ref()
            .unwrap()
            .turn_number,
        turn
    );
    assert_ne!(marker(&before, f.actors[0]), Some(turn));
    let query = TableRollOptionsRequest {
        campaign_id: f.campaign,
        channel: TableTransportChannel::Player {
            player_id: f.players[0],
            character_id: f.characters[0],
        },
        revision: presented.revision,
        roll_id: pending.id,
    };
    let options = f.runtime.table_roll_options(query).await.unwrap();
    assert_eq!(
        options.savage_attacker.unwrap().weapon_dice,
        usize::from(dice)
    );
    let raw = |value| RollResult {
        request_id: pending.id,
        source: RollSource::Physical,
        dice: (0..dice).map(|_| DieResult { sides: 4, value }).collect(),
    };
    let declaration = action(TacticalAction::SubmitSavageAttacker {
        roll: SavageAttackerRoll {
            weapon_dice: Some(usize::from(dice)),
            first: raw(1),
            second: raw(4),
            chosen: DamageRollChoice::First,
            inspiration: None,
        },
    });
    Box::pin(rejected(f, Some(1), declaration.clone())).await;
    let accepted = Box::pin(accept(f, url, directory, label, Some(0), declaration)).await;
    let after = Box::new(state(f).await);
    assert_eq!(marker(&after, f.actors[0]), Some(turn));
    let record = after.rules.as_ref().unwrap().rolls.last().unwrap();
    assert_eq!(
        record.savage_attacker.as_ref().unwrap().chosen,
        DamageRollChoice::First
    );
    assert_ne!(
        record.request.id, pending.id,
        "the presented handle is not the canonical raw ID"
    );
    accepted
}

async fn first_encounter(f: &mut Fixture, url: &str, directory: &Path) -> FirstEncounter {
    // This existing helper creates the source creature, equips the purchased
    // dagger, rolls a real critical hit and obtains the target's actual decline.
    let goblin = Box::pin(table_savage_cases::prepare(f)).await;
    let before = Box::new(state(f).await);
    assert_eq!(before.rules.as_ref().unwrap().entities[&goblin].hp, 10);
    assert_eq!(marker(&before, f.actors[0]), None);
    let HandAssignment::Item(dagger) = before
        .rules
        .as_ref()
        .unwrap()
        .tactical_inventory
        .as_ref()
        .unwrap()
        .loadout(f.actors[0])
        .unwrap()
        .hands
        .hands[Hand::Right.index()]
    else {
        panic!("the first genuine attack must leave the purchased dagger held");
    };
    let (savage_request, savage_response) =
        Box::pin(savage_roll(f, url, directory, "first-savage", 2, 1)).await;
    assert_eq!(
        state(f).await.rules.as_ref().unwrap().entities[&goblin].hp,
        5
    );
    // The paid attack ends by the ordinary PC EndTurn, not by release clearing it.
    Box::pin(direct(f, Some(0), TacticalAction::EndTurn)).await;
    Box::pin(accept(
        f,
        url,
        directory,
        "first-conclusion",
        None,
        conclusion(),
    ))
    .await;
    let settled = Box::new(state(f).await);
    let finish = request(f, None, action(TacticalAction::FinishEncounter)).await;
    Box::pin(durable_step(f, url, directory, "first-finished", finish)).await;
    let finished = Box::new(state(f).await);
    assert_eq!(finished.clock, settled.clock);
    assert_eq!(
        finished.rules.as_ref().unwrap().entities,
        settled.rules.as_ref().unwrap().entities
    );
    assert_eq!(
        finished.rules.as_ref().unwrap().rolls,
        settled.rules.as_ref().unwrap().rolls
    );
    assert_eq!(finished.items, settled.items);
    assert_eq!(
        finished.rules.as_ref().unwrap().tactical_inventory,
        settled.rules.as_ref().unwrap().tactical_inventory
    );
    assert_eq!(marker(&finished, f.actors[0]), Some(1));
    assert!(finished.rules.as_ref().unwrap().timing.is_none());
    let completion = finished
        .encounter_history
        .as_ref()
        .unwrap()
        .last()
        .unwrap()
        .clone();
    assert_eq!(completion.final_turn.number, 2);
    assert_eq!(completion.final_turn.actor, goblin);
    FirstEncounter {
        goblin,
        dagger,
        completion,
        rolls: finished.rules.as_ref().unwrap().rolls.clone(),
        savage_request,
        savage_response,
    }
}

async fn second_initiative(f: &mut Fixture, url: &str, directory: &Path, first: &FirstEncounter) {
    let actor = f.actors[0];
    let finished = Box::new(state(f).await);
    let mut setup = replacement(f, &finished, first.goblin);
    setup.name = "The same sentry at the next encounter".into();
    setup.creatures[0].public_label = "Small armored figure".into();
    setup.creatures[0].position = SpatialPoint { x: 20, y: 10, z: 0 };
    Box::pin(accept(
        f,
        url,
        directory,
        "savage-replacement",
        None,
        TableAction::PrepareBattlefield {
            setup: Box::new(setup),
        },
    ))
    .await;
    let prepared = Box::new(state(f).await);
    assert_eq!(marker(&prepared, f.actors[0]), Some(1));
    assert_eq!(
        prepared.rules.as_ref().unwrap().entities,
        finished.rules.as_ref().unwrap().entities
    );
    assert!(prepared.rules.as_ref().unwrap().timing.is_none());
    Box::pin(accept(
        f,
        url,
        directory,
        "savage-initiative",
        None,
        action(TacticalAction::Begin {
            execution: TacticalExecutionVersion::ReleasedTimeV1,
            combatants: vec![
                TacticalCombatant {
                    actor: first.goblin,
                    source: TacticalSource::Creature {
                        definition_id: "goblin-warrior".into(),
                    },
                    surprised: false,
                },
                TacticalCombatant {
                    actor,
                    source: TacticalSource::Character,
                    surprised: false,
                },
            ],
            groups: [first.goblin, actor]
                .into_iter()
                .map(|actor| InitiativeGroup {
                    actors: vec![actor],
                    request_id: RollRequestId::new(),
                })
                .collect(),
        }),
    ))
    .await;
    assert_eq!(marker(&state(f).await, f.actors[0]), Some(1));
    for (player, face, label) in [
        (None, 20, "goblin-initiative"),
        (Some(0), 2, "pc-initiative"),
    ] {
        let raw = raw_action(f, player, &[face]).await;
        Box::pin(accept(f, url, directory, label, player, raw)).await;
    }
    let active = Box::new(state(f).await);
    let timing = active.rules.as_ref().unwrap().timing.as_ref().unwrap();
    assert_eq!(timing.turn_number, first.completion.final_turn.number + 1);
    assert_eq!(timing.turn_number, 3);
    assert_eq!(timing.round, 1);
    assert_eq!(timing.order[timing.index].actor, first.goblin);
    assert!(timing.reactions_spent.is_empty());
    assert_eq!(marker(&active, f.actors[0]), Some(1));
    assert_eq!(
        active.rules.as_ref().unwrap().entities[&f.actors[0]],
        finished.rules.as_ref().unwrap().entities[&f.actors[0]]
    );
    assert_eq!(
        active.rules.as_ref().unwrap().tactical_inventory,
        finished.rules.as_ref().unwrap().tactical_inventory
    );
    assert_eq!(active.encounter_history, finished.encounter_history);
}

async fn off_turn_savage(f: &mut Fixture, url: &str, directory: &Path, first: &FirstEncounter) {
    let (movement, _) = Box::pin(accept(
        f,
        url,
        directory,
        "goblin-leaves-reach",
        None,
        action(TacticalAction::Move {
            path: vec![TacticalMoveStep {
                destination: SpatialPoint { x: 30, y: 10, z: 0 },
                mode: MovementMode::Walk,
            }],
        }),
    ))
    .await;
    let owner = view(f, Some(0)).await;
    let opportunity = owner.tactical.unwrap().opportunity.unwrap();
    assert_eq!(opportunity.actor, f.actors[0]);
    assert_eq!(opportunity.target.actor, first.goblin);
    assert!(
        opportunity
            .weapons
            .unwrap()
            .weapons
            .iter()
            .any(|weapon| weapon.item == first.dagger)
    );
    assert!(
        view(f, Some(1))
            .await
            .tactical
            .unwrap()
            .opportunity
            .is_none()
    );
    let pending = Box::new(state(f).await);
    assert_eq!(
        pending
            .encounter
            .as_ref()
            .unwrap()
            .participant(first.goblin)
            .unwrap()
            .position
            .x,
        20
    );
    assert_eq!(
        pending
            .rules
            .as_ref()
            .unwrap()
            .timing
            .as_ref()
            .unwrap()
            .turn_number,
        3
    );
    let attack = action(TacticalAction::OpportunityAttack {
        choice: TacticalMeleeChoice::Weapon(dagger_choice(first.dagger, first.goblin)),
    });
    Box::pin(rejected(f, Some(1), attack.clone())).await;
    let (opportunity, _) =
        Box::pin(accept(f, url, directory, "pc-opportunity", Some(0), attack)).await;
    let attacking = Box::new(state(f).await);
    let resolution = attacking
        .encounter
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .resolution
        .as_ref()
        .unwrap();
    assert_eq!(resolution.origin.id, movement.command_id);
    assert_eq!(
        resolution.attack.as_ref().unwrap().origin.id,
        opportunity.command_id
    );
    assert!(matches!(&resolution.attack.as_ref().unwrap().admission,
        TacticalAttackAdmission::Opportunity(window) if window.origin.id == movement.command_id));
    Box::pin(table_hit_driver::roll_then_decline(f, false, &[15])).await;
    Box::pin(savage_roll(f, url, directory, "off-turn-savage", 1, 3)).await;
    let after = Box::new(state(f).await);
    assert_eq!(after.rules.as_ref().unwrap().entities[&first.goblin].hp, 1);
    assert_eq!(
        after
            .encounter
            .as_ref()
            .unwrap()
            .participant(first.goblin)
            .unwrap()
            .position
            .x,
        30
    );
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
    assert_eq!(
        after
            .rules
            .as_ref()
            .unwrap()
            .timing
            .as_ref()
            .unwrap()
            .reactions_spent,
        vec![f.actors[0]]
    );
    assert_eq!(marker(&after, f.actors[0]), Some(3));

    // The Goblin's completed Walk retains continuity. Its real Dash resets that
    // bookkeeping without touching the PC's already-spent opportunity Reaction.
    Box::pin(accept(
        f,
        url,
        directory,
        "goblin-dash-settles-walk",
        None,
        action(TacticalAction::Dash {
            speed: DashSpeed::Speed,
        }),
    ))
    .await;
    let dashed = Box::new(state(f).await);
    assert!(
        dashed
            .encounter
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .budget
            .movement_progress
            .is_none()
    );
    assert_eq!(
        dashed
            .rules
            .as_ref()
            .unwrap()
            .timing
            .as_ref()
            .unwrap()
            .reactions_spent,
        vec![f.actors[0]]
    );
    Box::pin(accept(
        f,
        url,
        directory,
        "second-conclusion",
        None,
        conclusion(),
    ))
    .await;
    let blocked = request(f, None, action(TacticalAction::FinishEncounter)).await;
    let reason = Box::pin(rejected_request(f, blocked)).await;
    assert!(
        reason.contains("spent Reactions must reach their owners' real Start"),
        "{reason}"
    );
    let capability = view(f, None).await.tactical.unwrap().release.unwrap();
    assert!(!capability.may_finish);
    assert!(capability.blocker.unwrap().contains("spent Reactions"));
    assert!(view(f, Some(1)).await.tactical.unwrap().release.is_none());
}

async fn own_turn_savage(f: &mut Fixture, url: &str, directory: &Path, first: &FirstEncounter) {
    let before = Box::new(state(f).await);
    Box::pin(accept(
        f,
        url,
        directory,
        "real-pc-start",
        None,
        action(TacticalAction::EndTurn),
    ))
    .await;
    let started = Box::new(state(f).await);
    let timing = started.rules.as_ref().unwrap().timing.as_ref().unwrap();
    assert_eq!(timing.turn_number, 4);
    assert_eq!(timing.order[timing.index].actor, f.actors[0]);
    assert!(timing.reactions_spent.is_empty());
    assert_eq!(
        marker(&started, f.actors[0]),
        Some(3),
        "Start does not erase a previous Savage epoch"
    );
    assert_eq!(
        started.rules.as_ref().unwrap().rolls,
        before.rules.as_ref().unwrap().rolls
    );
    assert!(
        view(f, None)
            .await
            .tactical
            .unwrap()
            .release
            .unwrap()
            .may_finish
    );
    Box::pin(direct(
        f,
        Some(0),
        TacticalAction::Move {
            path: vec![TacticalMoveStep {
                destination: SpatialPoint { x: 20, y: 10, z: 0 },
                mode: MovementMode::Walk,
            }],
        },
    ))
    .await;
    Box::pin(accept(
        f,
        url,
        directory,
        "pc-own-turn-attack",
        Some(0),
        action(TacticalAction::Attack {
            choice: dagger_choice(first.dagger, first.goblin),
        }),
    ))
    .await;
    Box::pin(table_hit_driver::roll_then_decline(f, false, &[15])).await;
    Box::pin(savage_roll(f, url, directory, "own-turn-savage", 1, 4)).await;
    assert_eq!(
        view(f, Some(0)).await.tactical.unwrap().attack_decision,
        Some(TableAttackDecision {
            actor: f.actors[0],
            kind: TableAttackDecisionKind::Knockout
        })
    );
    Box::pin(accept(
        f,
        url,
        directory,
        "actual-knockout",
        Some(0),
        action(TacticalAction::ChooseAttackKnockout {
            choice: KnockoutChoice::KnockOut,
        }),
    ))
    .await;
    let final_state = Box::new(state(f).await);
    assert_eq!(marker(&final_state, f.actors[0]), Some(4));
    assert_eq!(
        final_state.rules.as_ref().unwrap().entities[&first.goblin].hp,
        1
    );
    assert!(
        final_state
            .rules
            .as_ref()
            .unwrap()
            .tactical_recovery
            .as_ref()
            .unwrap()[&first.goblin]
            .knockout
            .is_some()
    );
    assert!(
        final_state
            .rules
            .as_ref()
            .unwrap()
            .timing
            .as_ref()
            .unwrap()
            .reactions_spent
            .is_empty()
    );
    assert_eq!(
        final_state.encounter_history.as_ref().unwrap().completions,
        vec![first.completion.clone()]
    );
    assert!(
        final_state
            .rules
            .as_ref()
            .unwrap()
            .rolls
            .starts_with(&first.rolls)
    );
    let before_retry = Box::new(export_campaign(&f.pool, f.campaign).await.unwrap());
    assert_eq!(
        serde_json::to_vec(
            &Box::pin(
                f.runtime
                    .submit_presented_table(first.savage_request.clone())
            )
            .await
            .unwrap()
        )
        .unwrap(),
        serde_json::to_vec(&first.savage_response).unwrap()
    );
    let mut changed = first.savage_request.clone();
    let TableTransportInput::Action(declaration) = &mut changed.input else {
        panic!("the original Savage request is an action");
    };
    let TableAction::Tactical {
        action: TacticalAction::SubmitSavageAttacker { roll },
    } = declaration.as_mut()
    else {
        panic!("the original request retains its explicit Savage action");
    };
    roll.first.dice[0].value = 2;
    assert!(
        Box::pin(f.runtime.submit_presented_table(changed))
            .await
            .is_err()
    );
    let mut after_retry = Box::new(export_campaign(&f.pool, f.campaign).await.unwrap());
    after_retry.exported_at_utc = before_retry.exported_at_utc.clone();
    assert_eq!(
        after_retry, before_retry,
        "old exact retry and changed-body refusal cannot spend or rewrite the new epoch"
    );
}

#[tokio::test]
async fn savage_uses_distinct_encounter_turns_and_spent_reaction_waits_for_real_own_start() {
    let directory =
        std::env::temp_dir().join(format!("dmd-release-savage-{}", CampaignId::new().0));
    std::fs::create_dir_all(&directory).unwrap();
    let url = format!(
        "sqlite://{}",
        directory
            .join("campaign.sqlite")
            .to_string_lossy()
            .replace('\\', "/")
    );
    let pool = open_sqlite(&url).await.unwrap();
    let mut creation = input("Character 0");
    creation.purchases.push(EquipmentChoice {
        item_id: "dagger".into(),
        quantity: 1,
    });
    let mut f = Box::pin(Fixture::with_creation_pool(
        TableContract::default(),
        Some(creation),
        pool,
    ))
    .await;
    let first = Box::pin(first_encounter(&mut f, &url, &directory)).await;
    Box::pin(second_initiative(&mut f, &url, &directory, &first)).await;
    Box::pin(off_turn_savage(&mut f, &url, &directory, &first)).await;
    Box::pin(own_turn_savage(&mut f, &url, &directory, &first)).await;
    f.pool.close().await;
    drop(f);
    sqlite_test_cleanup::remove_closed_directory(&directory)
        .await
        .unwrap();
}
