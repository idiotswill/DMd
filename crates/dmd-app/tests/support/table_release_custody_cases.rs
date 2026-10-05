//! A source bow is fired, then dropped by a genuine knockout. Its retired
//! placement and spent ammunition survive release, new initiative and first aid.
use super::*;

struct SourceGear {
    goblin: EntityId,
    bow: ItemId,
    arrows: ItemId,
    shield: ItemId,
    shot: TableTransportRequest,
    shot_response: TableTransportResult,
}

async fn accept(
    f: &mut Fixture,
    url: &str,
    directory: &Path,
    label: &str,
    player: Option<usize>,
    declaration: TableAction,
) -> (TableTransportRequest, TableTransportResult) {
    let request = request(f, player, declaration).await;
    let response = Box::pin(durable_step(f, url, directory, label, request.clone())).await;
    (request, response)
}

fn assert_retired_gear(state: &CampaignState, gear: &SourceGear, ground: &TacticalGroundItem) {
    let space = &state.encounter_history.as_ref().unwrap().spaces[0];
    assert_eq!(space.ground_items, vec![ground.clone()]);
    assert_eq!(
        state.items[&gear.bow].custody,
        Custody::Location(space.location_id)
    );
    assert_eq!(
        state.items[&gear.arrows].custody,
        Custody::Entity(gear.goblin)
    );
    assert_eq!(state.items[&gear.arrows].quantity, 19);
    assert_eq!(
        state.items[&gear.shield].custody,
        Custody::Entity(gear.goblin)
    );
    let inventory = state
        .rules
        .as_ref()
        .unwrap()
        .tactical_inventory
        .as_ref()
        .unwrap();
    assert!(
        inventory
            .loadout(gear.goblin)
            .unwrap()
            .hands
            .hands
            .iter()
            .all(|hand| *hand != HandAssignment::Item(gear.bow))
    );
    if let Some(flow) = &state.encounter.as_ref().unwrap().flow {
        assert!(flow.ground_items.iter().all(|entry| entry.item != gear.bow));
    }
}

async fn fire_source_bow(f: &mut Fixture, url: &str, directory: &Path) -> SourceGear {
    let goblin = Box::pin(table_attack_cases::prepare_at(
        f,
        SpatialPoint { x: 40, y: 10, z: 0 },
    ))
    .await;
    Box::pin(direct(f, Some(0), TacticalAction::EndTurn)).await;
    let host = view(f, None).await.tactical.unwrap();
    let shield = host.shield_options.unwrap().donned.unwrap();
    let options = host.attack_options.unwrap();
    let bow = options
        .weapons
        .iter()
        .find(|weapon| weapon.name == "Shortbow")
        .unwrap();
    let arrows = bow.ammunition[0].id;
    let bow = bow.item;
    assert_eq!(state(f).await.items[&arrows].quantity, 20);
    Box::pin(rejected(f, Some(0), action(TacticalAction::DoffShield))).await;
    Box::pin(accept(
        f,
        url,
        directory,
        "real-shield-doff",
        None,
        action(TacticalAction::DoffShield),
    ))
    .await;
    // Doffing paid the source's Action. The bow shot waits for its next real turn.
    Box::pin(direct(f, None, TacticalAction::EndTurn)).await;
    Box::pin(direct(f, Some(0), TacticalAction::EndTurn)).await;
    let attack = action(table_shield_cases::bow_action(
        bow,
        arrows,
        f.actors[0],
        false,
    ));
    Box::pin(rejected(f, Some(0), attack.clone())).await;
    let (shot, shot_response) = Box::pin(accept(
        f,
        url,
        directory,
        "spent-source-arrow",
        None,
        attack,
    ))
    .await;
    let pending = Box::new(state(f).await);
    assert_eq!(pending.items[&arrows].quantity, 19);
    assert_eq!(
        pending
            .rules
            .as_ref()
            .unwrap()
            .tactical_inventory
            .as_ref()
            .unwrap()
            .loadout(goblin)
            .unwrap()
            .hands
            .hands,
        [HandAssignment::Item(bow); 2]
    );
    assert_eq!(view(f, None).await.roll.unwrap().mode, RollMode::Normal);
    let miss = raw_action(f, None, &[1]).await;
    Box::pin(accept(f, url, directory, "source-bow-miss", None, miss)).await;
    assert_eq!(state(f).await.items[&arrows].quantity, 19);
    Box::pin(direct(f, None, TacticalAction::EndTurn)).await;
    SourceGear {
        goblin,
        bow,
        arrows,
        shield,
        shot,
        shot_response,
    }
}

async fn actual_knockout(
    f: &mut Fixture,
    url: &str,
    directory: &Path,
    gear: &SourceGear,
) -> TacticalGroundItem {
    assert_eq!(
        state(f).await.rules.as_ref().unwrap().entities[&gear.goblin].hp,
        10
    );
    Box::pin(direct(
        f,
        Some(0),
        TacticalAction::Move {
            path: [20, 30]
                .into_iter()
                .map(|x| TacticalMoveStep {
                    destination: SpatialPoint { x, y: 10, z: 0 },
                    mode: MovementMode::Walk,
                })
                .collect(),
        },
    ))
    .await;
    let options = view(f, Some(0))
        .await
        .tactical
        .unwrap()
        .attack_options
        .unwrap();
    let dagger = options
        .weapons
        .iter()
        .find(|weapon| weapon.name == "Dagger")
        .unwrap()
        .item;
    Box::pin(accept(
        f,
        url,
        directory,
        "melee-before-knockout",
        Some(0),
        action(TacticalAction::Attack {
            choice: WeaponUseChoice {
                weapon: dagger,
                target: gear.goblin,
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
        }),
    ))
    .await;
    Box::pin(table_hit_driver::roll_then_decline(f, false, &[20])).await;
    let damage = raw_action(f, Some(0), &[4, 4]).await;
    Box::pin(accept(
        f,
        url,
        directory,
        "lethal-melee-damage",
        Some(0),
        damage,
    ))
    .await;
    assert_eq!(
        view(f, Some(0)).await.tactical.unwrap().attack_decision,
        Some(TableAttackDecision {
            actor: f.actors[0],
            kind: TableAttackDecisionKind::Knockout
        })
    );
    let knockout = action(TacticalAction::ChooseAttackKnockout {
        choice: KnockoutChoice::KnockOut,
    });
    Box::pin(rejected(f, Some(1), knockout.clone())).await;
    let (choice, _) = Box::pin(accept(
        f,
        url,
        directory,
        "source-knocked-out",
        Some(0),
        knockout,
    ))
    .await;
    let dropped = Box::new(state(f).await);
    let rules = dropped.rules.as_ref().unwrap();
    assert_eq!(rules.entities[&gear.goblin].hp, 1);
    assert!(dmd_rules::active_conditions(rules, gear.goblin).contains(&Condition::Unconscious));
    assert_eq!(
        rules
            .tactical_inventory
            .as_ref()
            .unwrap()
            .loadout(gear.goblin)
            .unwrap()
            .hands,
        WeaponLoadout::default()
    );
    let recovery = &rules.tactical_recovery.as_ref().unwrap()[&gear.goblin];
    assert_eq!(
        recovery.knockout.as_ref().unwrap().origin.command.id,
        choice.command_id
    );
    assert_eq!(
        recovery
            .knockout_rest
            .as_ref()
            .unwrap()
            .started_by
            .command
            .id,
        choice.command_id
    );
    assert!(
        rules
            .rests
            .iter()
            .any(|rest| rest.actor == gear.goblin && rest.kind == RestKind::Short)
    );
    let ground = dropped
        .encounter
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .ground_items
        .iter()
        .find(|entry| entry.item == gear.bow)
        .unwrap()
        .clone();
    assert_eq!(ground.origin.id, choice.command_id);
    assert_eq!(ground.position, SpatialPoint { x: 40, y: 10, z: 0 });
    assert_eq!(dropped.items[&gear.arrows].quantity, 19);
    assert_eq!(
        dropped.items[&gear.shield].custody,
        Custody::Entity(gear.goblin)
    );
    // The completed paid PC attack retires at its genuine EndTurn. The source's
    // own Start neither wakes it nor grants replacement weapons or ammunition.
    Box::pin(direct(f, Some(0), TacticalAction::EndTurn)).await;
    ground
}

async fn release_with_recovery(
    f: &mut Fixture,
    url: &str,
    directory: &Path,
    gear: &SourceGear,
    ground: &TacticalGroundItem,
) -> TacticalRecovery {
    Box::pin(accept(
        f,
        url,
        directory,
        "custody-conclusion",
        None,
        conclusion(),
    ))
    .await;
    let before = Box::new(state(f).await);
    let recovery = before
        .rules
        .as_ref()
        .unwrap()
        .tactical_recovery
        .as_ref()
        .unwrap()[&gear.goblin]
        .clone();
    let finish = request(f, None, action(TacticalAction::FinishEncounter)).await;
    let finish_id = finish.command_id;
    Box::pin(durable_step(f, url, directory, "custody-finished", finish)).await;
    let finished = Box::new(state(f).await);
    assert_eq!(finished.clock, before.clock);
    assert_eq!(finished.items, before.items);
    assert_eq!(
        finished.rules.as_ref().unwrap().entities,
        before.rules.as_ref().unwrap().entities
    );
    assert_eq!(
        finished.rules.as_ref().unwrap().tactical_recovery,
        before.rules.as_ref().unwrap().tactical_recovery
    );
    assert_eq!(
        finished.rules.as_ref().unwrap().rests,
        before.rules.as_ref().unwrap().rests
    );
    assert_eq!(
        finished.rules.as_ref().unwrap().tactical_inventory,
        before.rules.as_ref().unwrap().tactical_inventory
    );
    assert_ne!(
        ground.origin.id, finish_id,
        "release does not replace the actual knockout drop cause"
    );
    assert_retired_gear(&finished, gear, ground);
    let host = view(f, None).await.tactical.unwrap().release.unwrap();
    assert_eq!(host.required_actors, vec![gear.goblin]);
    recovery
}

async fn replace_and_roll(
    f: &mut Fixture,
    url: &str,
    directory: &Path,
    gear: &SourceGear,
    ground: &TacticalGroundItem,
    recovery: &TacticalRecovery,
) {
    let finished = Box::new(state(f).await);
    let receipt = finished.encounter_history.as_ref().unwrap().last().unwrap();
    assert_eq!(receipt.final_turn.number, 6);
    let mut setup = replacement(f, &finished, gear.goblin);
    setup.name = "Another battlefield with the surviving source creature".into();
    setup.creatures[0].public_label = "Unconscious small figure".into();
    setup.creatures[0].position = SpatialPoint { x: 20, y: 10, z: 0 };
    setup.creatures[0].height = 8;
    let mut omitted = setup.clone();
    omitted.creatures.clear();
    omitted.characters[0].enemies.clear();
    Box::pin(rejected(
        f,
        None,
        TableAction::PrepareBattlefield {
            setup: Box::new(omitted),
        },
    ))
    .await;
    Box::pin(accept(
        f,
        url,
        directory,
        "custody-replacement",
        None,
        TableAction::PrepareBattlefield {
            setup: Box::new(setup),
        },
    ))
    .await;
    let prepared = Box::new(state(f).await);
    assert_eq!(
        &prepared
            .rules
            .as_ref()
            .unwrap()
            .tactical_recovery
            .as_ref()
            .unwrap()[&gear.goblin],
        recovery
    );
    assert_eq!(
        prepared.rules.as_ref().unwrap().rests,
        finished.rules.as_ref().unwrap().rests
    );
    assert_retired_gear(&prepared, gear, ground);
    let actor = f.actors[0];
    Box::pin(accept(
        f,
        url,
        directory,
        "unconscious-source-initiative",
        None,
        action(TacticalAction::Begin {
            execution: TacticalExecutionVersion::ReleasedTimeV1,
            combatants: vec![
                TacticalCombatant {
                    actor,
                    source: TacticalSource::Character,
                    surprised: false,
                },
                TacticalCombatant {
                    actor: gear.goblin,
                    source: TacticalSource::Creature {
                        definition_id: "goblin-warrior".into(),
                    },
                    surprised: false,
                },
            ],
            groups: [actor, gear.goblin]
                .into_iter()
                .map(|actor| InitiativeGroup {
                    actors: vec![actor],
                    request_id: RollRequestId::new(),
                })
                .collect(),
        }),
    ))
    .await;
    let begun = Box::new(state(f).await);
    let after = &begun
        .rules
        .as_ref()
        .unwrap()
        .tactical_recovery
        .as_ref()
        .unwrap()[&gear.goblin];
    let original = recovery.knockout.as_ref().unwrap();
    let retained = after.knockout.as_ref().unwrap();
    assert_eq!(retained.origin, original.origin);
    assert_eq!(retained.inflicted_at, original.inflicted_at);
    assert_eq!(retained.short_rest_started_at, None);
    assert_eq!(after.knockout_rest, None);
    assert!(
        !begun
            .rules
            .as_ref()
            .unwrap()
            .rests
            .iter()
            .any(|rest| rest.actor == gear.goblin)
    );
    assert!(
        dmd_rules::active_conditions(begun.rules.as_ref().unwrap(), gear.goblin)
            .contains(&Condition::Unconscious)
    );
    assert_retired_gear(&begun, gear, ground);
    let player_roll = raw_action(f, Some(0), &[20]).await;
    Box::pin(accept(
        f,
        url,
        directory,
        "custody-player-initiative",
        Some(0),
        player_roll,
    ))
    .await;
    assert_eq!(
        view(f, None).await.roll.unwrap().mode,
        RollMode::Disadvantage
    );
    let source_roll = raw_action(f, None, &[2, 2]).await;
    Box::pin(accept(
        f,
        url,
        directory,
        "incapacitated-initiative-raw",
        None,
        source_roll,
    ))
    .await;
    let active = Box::new(state(f).await);
    let timing = active.rules.as_ref().unwrap().timing.as_ref().unwrap();
    assert_eq!(timing.turn_number, receipt.final_turn.number + 1);
    assert_eq!(timing.order[timing.index].actor, actor);
    assert_eq!(active.items, finished.items);
    assert_retired_gear(&active, gear, ground);
}

async fn wake_without_regrant(
    f: &mut Fixture,
    url: &str,
    directory: &Path,
    gear: &SourceGear,
    ground: &TacticalGroundItem,
) {
    let aid = action(TacticalAction::FirstAid {
        target: gear.goblin,
        purpose: MedicinePurpose::EndKnockout,
    });
    Box::pin(rejected(f, Some(1), aid.clone())).await;
    Box::pin(accept(f, url, directory, "source-first-aid", Some(0), aid)).await;
    let pending = view(f, Some(0)).await.roll.unwrap();
    assert_eq!(pending.reason, "Wisdom (Medicine) first aid");
    let roll = raw_action(f, Some(0), &[20]).await;
    Box::pin(accept(f, url, directory, "source-wakes", Some(0), roll)).await;
    let awake = Box::new(state(f).await);
    let rules = awake.rules.as_ref().unwrap();
    assert_eq!(rules.entities[&gear.goblin].hp, 1);
    assert!(!dmd_rules::active_conditions(rules, gear.goblin).contains(&Condition::Unconscious));
    assert!(
        !rules
            .tactical_recovery
            .as_ref()
            .unwrap()
            .contains_key(&gear.goblin)
    );
    assert!(
        rules.entities[&gear.goblin].prone,
        "first aid does not stand the patient up"
    );
    assert_retired_gear(&awake, gear, ground);
    Box::pin(direct(f, Some(0), TacticalAction::EndTurn)).await;
    Box::pin(accept(
        f,
        url,
        directory,
        "source-stands-after-first-aid",
        None,
        action(TacticalAction::StandProne),
    ))
    .await;
    assert!(!state(f).await.rules.as_ref().unwrap().entities[&gear.goblin].prone);
    // The source can act again, but its dropped bow is physically elsewhere.
    let impossible = action(table_shield_cases::bow_action(
        gear.bow,
        gear.arrows,
        f.actors[0],
        false,
    ));
    Box::pin(rejected(f, None, impossible)).await;
    let options = view(f, None)
        .await
        .tactical
        .unwrap()
        .attack_options
        .unwrap();
    assert!(!options.weapons.iter().any(|weapon| weapon.item == gear.bow));
    let scimitar = options
        .weapons
        .iter()
        .find(|weapon| weapon.name == "Scimitar")
        .unwrap()
        .item;
    let target = f.actors[0];
    Box::pin(accept(
        f,
        url,
        directory,
        "awake-source-uses-owned-weapon",
        None,
        action(TacticalAction::CreatureWeaponAttack {
            feature_id: "scimitar".into(),
            choice: CreatureWeaponUseChoice {
                weapon: scimitar,
                target,
                grip: WeaponGrip::OneHand(Hand::Right),
                ammunition: None,
                equipment_change: Some(AttackEquipmentChange {
                    timing: EquipmentChangeTiming::BeforeAttack,
                    operation: AttackEquipmentOperation::Equip {
                        item: scimitar,
                        hand: Hand::Right,
                    },
                }),
            },
        }),
    ))
    .await;
    let miss = raw_action(f, None, &[1]).await;
    Box::pin(accept(f, url, directory, "awake-source-miss", None, miss)).await;
    assert_retired_gear(&state(f).await, gear, ground);
    let before = Box::new(export_campaign(&f.pool, f.campaign).await.unwrap());
    assert_eq!(
        serde_json::to_vec(
            &Box::pin(f.runtime.submit_presented_table(gear.shot.clone()))
                .await
                .unwrap()
        )
        .unwrap(),
        serde_json::to_vec(&gear.shot_response).unwrap()
    );
    let mut changed = gear.shot.clone();
    changed.input = TableTransportInput::Action(Box::new(action(TacticalAction::DoffShield)));
    assert!(
        Box::pin(f.runtime.submit_presented_table(changed))
            .await
            .is_err()
    );
    let mut after = Box::new(export_campaign(&f.pool, f.campaign).await.unwrap());
    after.exported_at_utc = before.exported_at_utc.clone();
    assert_eq!(
        after, before,
        "old bow receipt retries never consume a second arrow or reacquire the bow"
    );
}

#[tokio::test]
async fn spent_source_arrows_and_unconscious_bow_drop_survive_release_replacement_and_actual_first_aid()
 {
    let directory =
        std::env::temp_dir().join(format!("dmd-release-custody-{}", CampaignId::new().0));
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
    let gear = Box::pin(fire_source_bow(&mut f, &url, &directory)).await;
    let ground = Box::pin(actual_knockout(&mut f, &url, &directory, &gear)).await;
    let recovery = Box::pin(release_with_recovery(
        &mut f, &url, &directory, &gear, &ground,
    ))
    .await;
    Box::pin(replace_and_roll(
        &mut f, &url, &directory, &gear, &ground, &recovery,
    ))
    .await;
    Box::pin(wake_without_regrant(
        &mut f, &url, &directory, &gear, &ground,
    ))
    .await;
    f.pool.close().await;
    drop(f);
    sqlite_test_cleanup::remove_closed_directory(&directory)
        .await
        .unwrap();
}
