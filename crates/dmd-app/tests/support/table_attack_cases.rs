use super::*;
use dmd_rules::tactical::TacticalAction;

#[tokio::test]
async fn physical_attack_knockout_and_npc_shield_drop_restore_through_the_table() {
    let mut creation = input("Character 0");
    creation.purchases.push(EquipmentChoice {
        item_id: "dagger".into(),
        quantity: 1,
    });
    let mut f = Fixture::with_creation(TableContract::default(), Some(creation)).await;
    let target = prepare(&mut f).await;
    let attack = begin_attack(&f, target).await;
    check_knockout(&f, target, attack).await;
    f.pool.close().await;
}

async fn prepare(f: &mut Fixture) -> EntityId {
    let target = EntityId::new();
    let allocation =
        dmd_rules::tactical_creature_equipment::creature_equipment_plan("goblin-warrior", 20)
            .unwrap();
    f.host(
        TableAction::CreateCreature {
            creation: Box::new(TableCreatureCreation {
                entity_id: target,
                name: "Private sentry".into(),
                definition_id: "goblin-warrior".into(),
                size: CreatureSize::Small,
                additional_languages: vec![],
                ammunition_units: 20,
                item_ids: allocation.iter().map(|_| ItemId::new()).collect(),
            }),
        },
        Some(f.session),
    )
    .await;
    table_tactical_cases::prepare_source_scene_at(f, target, SpatialPoint { x: 20, y: 10, z: 0 })
        .await;
    f.host(
        TableAction::Tactical {
            action: TacticalAction::Begin {
                combatants: vec![
                    TacticalCombatant {
                        actor: f.actors[0],
                        source: TacticalSource::Character,
                        surprised: false,
                    },
                    TacticalCombatant {
                        actor: target,
                        source: TacticalSource::Creature {
                            definition_id: "goblin-warrior".into(),
                        },
                        surprised: false,
                    },
                ],
                groups: vec![
                    InitiativeGroup {
                        actors: vec![f.actors[0]],
                        request_id: RollRequestId::new(),
                    },
                    InitiativeGroup {
                        actors: vec![target],
                        request_id: RollRequestId::new(),
                    },
                ],
            },
        },
        Some(f.session),
    )
    .await;
    submit(f, false, &[18]).await;
    submit(f, true, &[2]).await;
    target
}

async fn submit(f: &Fixture, host: bool, values: &[u16]) {
    let viewer = if host {
        TableViewer::Host
    } else {
        TableViewer::Player(f.players[0])
    };
    let view = f.runtime.table_view(f.campaign, viewer).await.unwrap();
    let request = view.roll.unwrap();
    let sides = if request.mode == RollMode::Normal {
        request
            .dice
            .iter()
            .flat_map(|die| std::iter::repeat_n(die.sides, usize::from(die.count)))
            .collect::<Vec<_>>()
    } else {
        assert_eq!(
            request.dice,
            vec![DieSpec {
                count: 1,
                sides: 20
            }]
        );
        vec![20, 20]
    };
    assert_eq!(sides.len(), values.len());
    let meta = if host {
        f.meta(CommandIssuer::Admin, None, Some(f.session)).await
    } else {
        f.player_meta(0).await
    };
    f.runtime
        .execute_table(
            meta,
            TableAction::Tactical {
                action: TacticalAction::SubmitRoll {
                    result: RollResult {
                        request_id: request.id,
                        source: RollSource::Physical,
                        dice: sides
                            .into_iter()
                            .zip(values)
                            .map(|(sides, value)| DieResult {
                                sides,
                                value: *value,
                            })
                            .collect(),
                    },
                },
            },
        )
        .await
        .unwrap();
}

async fn begin_attack(f: &Fixture, target: EntityId) -> CommandMeta {
    let unrelated = f
        .runtime
        .table_view(f.campaign, TableViewer::Player(f.players[1]))
        .await
        .unwrap();
    assert!(unrelated.tactical.unwrap().attack_options.is_none());
    let view = f
        .runtime
        .table_view(f.campaign, TableViewer::Player(f.players[0]))
        .await
        .unwrap();
    let options = view.tactical.unwrap().attack_options.unwrap();
    assert_eq!(options.actor, f.actors[0]);
    assert_eq!(
        options.targets,
        vec![TableAttackTarget {
            actor: target,
            label: "Small armored figure".into()
        }]
    );
    let json = serde_json::to_string(&options).unwrap();
    for hidden in ["Private sentry", "goblin-warrior", "armor_class", "max_hp"] {
        assert!(!json.contains(hidden));
    }
    let weapon = options
        .weapons
        .iter()
        .find(|weapon| weapon.name == "Dagger")
        .unwrap()
        .item;
    let action = TableAction::Tactical {
        action: TacticalAction::Attack {
            choice: WeaponUseChoice {
                weapon,
                target,
                delivery: WeaponDelivery::Melee,
                ability: Ability::Strength,
                grip: WeaponGrip::OneHand(Hand::Right),
                purpose: WeaponAttackPurpose::Normal,
                ammunition: None,
                equipment_change: Some(AttackEquipmentChange {
                    timing: EquipmentChangeTiming::BeforeAttack,
                    operation: AttackEquipmentOperation::Equip {
                        item: weapon,
                        hand: Hand::Right,
                    },
                }),
            },
        },
    };
    let before = f
        .runtime
        .open_campaign(f.campaign)
        .await
        .unwrap()
        .state()
        .clone();
    assert!(
        f.runtime
            .execute_table(f.player_meta(1).await, action.clone())
            .await
            .is_err()
    );
    assert_eq!(
        f.runtime.open_campaign(f.campaign).await.unwrap().state(),
        &before
    );
    let meta = f.player_meta(0).await;
    f.runtime
        .execute_table(meta.clone(), action.clone())
        .await
        .unwrap();
    assert!(
        f.runtime
            .execute_table(meta.clone(), action)
            .await
            .unwrap()
            .already_accepted
    );
    assert_restore(f).await;
    submit(f, false, &[20]).await;
    let request = f
        .runtime
        .table_view(f.campaign, TableViewer::Player(f.players[0]))
        .await
        .unwrap()
        .roll
        .unwrap();
    assert_eq!(request.reason, "Weapon damage");
    assert_eq!(request.dice, vec![DieSpec { count: 2, sides: 4 }]);
    assert_restore(f).await;
    submit(f, false, &[4, 4]).await;
    meta
}

async fn assert_restore(f: &Fixture) {
    let export = export_campaign(&f.pool, f.campaign).await.unwrap();
    let pool = open_sqlite("sqlite::memory:").await.unwrap();
    let restored = CampaignRuntime::from_content_root(
        pool.clone(),
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../content"),
    );
    restored.restore_campaign(&export).await.unwrap();
    assert_eq!(
        restored.resume_campaign(f.campaign).await.unwrap().state(),
        f.runtime.open_campaign(f.campaign).await.unwrap().state()
    );
    pool.close().await;
}

async fn execute_after_restore(f: &Fixture, meta: CommandMeta, action: TableAction) {
    let export = export_campaign(&f.pool, f.campaign).await.unwrap();
    let pool = open_sqlite("sqlite::memory:").await.unwrap();
    let restored = CampaignRuntime::from_content_root(
        pool.clone(),
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../content"),
    );
    restored.restore_campaign(&export).await.unwrap();
    restored.resume_campaign(f.campaign).await.unwrap();
    restored
        .execute_table(meta.clone(), action.clone())
        .await
        .unwrap();
    f.runtime.execute_table(meta, action).await.unwrap();
    assert_eq!(
        restored.open_campaign(f.campaign).await.unwrap().state(),
        f.runtime.open_campaign(f.campaign).await.unwrap().state()
    );
    pool.close().await;
}

async fn check_knockout(f: &Fixture, target: EntityId, attack: CommandMeta) {
    let view = f
        .runtime
        .table_view(f.campaign, TableViewer::Player(f.players[0]))
        .await
        .unwrap();
    assert!(view.roll.is_none());
    assert_eq!(
        view.tactical.unwrap().attack_decision,
        Some(TableAttackDecision {
            actor: f.actors[0],
            kind: TableAttackDecisionKind::Knockout
        })
    );
    let unrelated = f
        .runtime
        .table_view(f.campaign, TableViewer::Player(f.players[1]))
        .await
        .unwrap();
    assert!(unrelated.tactical.unwrap().attack_decision.is_none());
    let action = TableAction::Tactical {
        action: TacticalAction::ChooseAttackKnockout {
            choice: KnockoutChoice::KnockOut,
        },
    };
    let meta = f.player_meta(0).await;
    execute_after_restore(f, meta.clone(), action.clone()).await;
    assert!(
        f.runtime
            .execute_table(meta.clone(), action)
            .await
            .unwrap()
            .already_accepted
    );
    let state = f
        .runtime
        .open_campaign(f.campaign)
        .await
        .unwrap()
        .state()
        .clone();
    let rules = state.rules.as_ref().unwrap();
    let npc = &rules.entities[&target];
    assert_eq!(npc.hp, 1);
    assert_eq!(dmd_rules::armor_class(npc), 13);
    let recovery = &rules.tactical_recovery.as_ref().unwrap()[&target];
    assert!(recovery.knockout.is_some());
    assert_eq!(
        recovery.knockout_rest.as_ref().unwrap().started_by.command,
        meta
    );
    let inventory = rules.tactical_inventory.as_ref().unwrap();
    assert_eq!(inventory.loadout(target).unwrap().shield, None);
    assert_eq!(
        inventory.loadout(target).unwrap().hands,
        WeaponLoadout::default()
    );
    let shield = state
        .items
        .values()
        .find(|item| item.definition_id == "shield")
        .unwrap();
    assert_ne!(shield.custody, Custody::Entity(target));
    assert!(
        state
            .encounter
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .ground_items
            .iter()
            .any(|item| item.item == shield.id)
    );
    let history = &state
        .encounter
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .budget
        .weapon_history;
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].origin, attack);
    assert_restore(f).await;
    f.runtime
        .execute_table(
            f.player_meta(0).await,
            TableAction::Tactical {
                action: TacticalAction::EndTurn,
            },
        )
        .await
        .unwrap();
    let host = f
        .runtime
        .table_view(f.campaign, TableViewer::Host)
        .await
        .unwrap()
        .tactical
        .unwrap();
    assert_eq!(host.active_actor, Some(target));
    assert_eq!(host.participants.len(), 2);
    let npc_options = host.attack_options.unwrap();
    assert_eq!(npc_options.actor, target);
    assert!(
        npc_options.targets.is_empty(),
        "host map truth is not the unconscious NPC's perception"
    );
}

#[tokio::test]
async fn thrown_weapon_custody_removes_it_from_later_table_choices() {
    let mut creation = input("Character 0");
    creation.purchases.push(EquipmentChoice {
        item_id: "dagger".into(),
        quantity: 1,
    });
    let mut f = Fixture::with_creation(TableContract::default(), Some(creation)).await;
    let target = prepare(&mut f).await;
    let options = f
        .runtime
        .table_view(f.campaign, TableViewer::Player(f.players[0]))
        .await
        .unwrap()
        .tactical
        .unwrap()
        .attack_options
        .unwrap();
    let weapon = options
        .weapons
        .iter()
        .find(|weapon| weapon.name == "Dagger")
        .unwrap()
        .item;
    f.runtime
        .execute_table(
            f.player_meta(0).await,
            TableAction::Tactical {
                action: TacticalAction::Attack {
                    choice: WeaponUseChoice {
                        weapon,
                        target,
                        delivery: WeaponDelivery::Thrown,
                        ability: Ability::Strength,
                        grip: WeaponGrip::OneHand(Hand::Right),
                        purpose: WeaponAttackPurpose::Normal,
                        ammunition: None,
                        equipment_change: None,
                    },
                },
            },
        )
        .await
        .unwrap();
    let view = f
        .runtime
        .table_view(f.campaign, TableViewer::Player(f.players[0]))
        .await
        .unwrap();
    assert_eq!(view.roll.unwrap().mode, RollMode::Disadvantage);
    submit(&f, false, &[20, 20]).await;
    submit(&f, false, &[4, 4]).await;
    let view = f
        .runtime
        .table_view(f.campaign, TableViewer::Player(f.players[0]))
        .await
        .unwrap();
    assert!(
        view.tactical
            .unwrap()
            .attack_options
            .unwrap()
            .weapons
            .iter()
            .all(|entry| entry.item != weapon)
    );
    let state = f
        .runtime
        .open_campaign(f.campaign)
        .await
        .unwrap()
        .state()
        .clone();
    assert_ne!(state.items[&weapon].custody, Custody::Entity(f.actors[0]));
    assert_restore(&f).await;
    f.pool.close().await;
}

#[tokio::test]
async fn light_and_nick_table_choices_use_real_current_turn_triggers_only_once() {
    for nick in [false, true] {
        let mut creation = input("Character 0");
        creation.purchases.push(EquipmentChoice {
            item_id: "dagger".into(),
            quantity: 2,
        });
        let mut f = Fixture::with_creation(TableContract::default(), Some(creation)).await;
        let target = prepare(&mut f).await;
        let options = attack_options(&f).await;
        let daggers = options
            .weapons
            .iter()
            .filter(|weapon| weapon.name == "Dagger")
            .map(|weapon| weapon.item)
            .collect::<Vec<_>>();
        assert_eq!(daggers.len(), 2);
        let trigger = f.player_meta(0).await;
        f.runtime
            .execute_table(
                trigger.clone(),
                TableAction::Tactical {
                    action: TacticalAction::Attack {
                        choice: WeaponUseChoice {
                            weapon: daggers[0],
                            target,
                            delivery: WeaponDelivery::Melee,
                            ability: Ability::Strength,
                            grip: WeaponGrip::OneHand(Hand::Left),
                            purpose: WeaponAttackPurpose::Normal,
                            ammunition: None,
                            equipment_change: Some(AttackEquipmentChange {
                                timing: EquipmentChangeTiming::BeforeAttack,
                                operation: AttackEquipmentOperation::Equip {
                                    item: daggers[0],
                                    hand: Hand::Left,
                                },
                            }),
                        },
                    },
                },
            )
            .await
            .unwrap();
        submit(&f, false, &[1]).await;
        check_light_followup(&f, target, &daggers, trigger, nick).await;
        f.pool.close().await;
    }
}

async fn attack_options(f: &Fixture) -> TableAttackOptions {
    f.runtime
        .table_view(f.campaign, TableViewer::Player(f.players[0]))
        .await
        .unwrap()
        .tactical
        .unwrap()
        .attack_options
        .unwrap()
}

async fn check_light_followup(
    f: &Fixture,
    target: EntityId,
    daggers: &[ItemId],
    trigger: CommandMeta,
    nick: bool,
) {
    let options = attack_options(f).await;
    assert!(
        options
            .weapons
            .iter()
            .find(|weapon| weapon.item == daggers[0])
            .unwrap()
            .purposes
            .is_empty()
    );
    assert_eq!(
        options
            .weapons
            .iter()
            .find(|weapon| weapon.item == daggers[1])
            .unwrap()
            .purposes,
        vec![
            WeaponAttackPurpose::LightBonus {
                trigger: trigger.id
            },
            WeaponAttackPurpose::Nick {
                trigger: trigger.id
            }
        ]
    );
    let choice = WeaponUseChoice {
        weapon: daggers[1],
        target,
        delivery: if nick {
            WeaponDelivery::Melee
        } else {
            WeaponDelivery::Thrown
        },
        ability: Ability::Strength,
        grip: WeaponGrip::OneHand(Hand::Right),
        purpose: if nick {
            WeaponAttackPurpose::Nick {
                trigger: trigger.id,
            }
        } else {
            WeaponAttackPurpose::LightBonus {
                trigger: trigger.id,
            }
        },
        ammunition: None,
        equipment_change: nick.then_some(AttackEquipmentChange {
            timing: EquipmentChangeTiming::BeforeAttack,
            operation: AttackEquipmentOperation::Equip {
                item: daggers[1],
                hand: Hand::Right,
            },
        }),
    };
    let meta = f.player_meta(0).await;
    execute_after_restore(
        f,
        meta,
        TableAction::Tactical {
            action: TacticalAction::Attack { choice },
        },
    )
    .await;
    submit(f, false, if nick { &[15] } else { &[15, 15] }).await;
    submit(f, false, &[4]).await;
    assert!(
        attack_options(f)
            .await
            .weapons
            .iter()
            .all(|weapon| weapon.purposes.is_empty())
    );
    let state = f
        .runtime
        .open_campaign(f.campaign)
        .await
        .unwrap()
        .state()
        .clone();
    let rules = state.rules.as_ref().unwrap();
    assert!(rules.timing.as_ref().unwrap().action_spent);
    assert_eq!(rules.timing.as_ref().unwrap().bonus_action_spent, !nick);
    assert_eq!(
        rules.entities[&target].hp, 6,
        "Light extra damage omits the positive ability modifier"
    );
    assert_restore(f).await;
}
