use super::*;
use dmd_rules::tactical::TacticalAction;

async fn state(f: &Fixture) -> CampaignState {
    f.runtime
        .open_campaign(f.campaign)
        .await
        .unwrap()
        .state()
        .clone()
}

async fn host_view(f: &Fixture) -> TableTacticalView {
    f.runtime
        .table_view(f.campaign, TableViewer::Host)
        .await
        .unwrap()
        .tactical
        .unwrap()
}

async fn host_action(f: &Fixture, action: TacticalAction) -> CommandMeta {
    let meta = f.meta(CommandIssuer::Admin, None, Some(f.session)).await;
    f.runtime
        .execute_table(meta.clone(), TableAction::Tactical { action })
        .await
        .unwrap();
    meta
}

async fn end_player(f: &Fixture) {
    f.runtime
        .execute_table(
            f.player_meta(0).await,
            TableAction::Tactical {
                action: TacticalAction::EndTurn,
            },
        )
        .await
        .unwrap();
}

async fn next_npc_turn(f: &Fixture) {
    host_action(f, TacticalAction::EndTurn).await;
    end_player(f).await;
}

fn armor(state: &CampaignState, actor: EntityId) -> i32 {
    dmd_rules::armor_class(&state.rules.as_ref().unwrap().entities[&actor])
}

#[tokio::test]
async fn paid_source_shield_bow_shield_controls_survive_cold_retry_and_hostile_restore() {
    let directory = std::env::temp_dir().join(format!("dmd-paid-shield-{}", CampaignId::new().0));
    std::fs::create_dir_all(&directory).unwrap();
    let database = directory.join("campaign.sqlite");
    let pool = dmd_persistence::open_sqlite_path(&database).await.unwrap();
    let mut f = Box::pin(Fixture::with_pool(TableContract::default(), pool)).await;
    let npc = Box::pin(table_attack_cases::prepare_at(
        &mut f,
        SpatialPoint { x: 60, y: 10, z: 0 },
    ))
    .await;
    Box::pin(end_player(&f)).await;
    let (shield, bow, arrows, doff) = Box::pin(doff_and_reopen(&mut f, npc, &database)).await;
    Box::pin(shoot_and_stow(&f, npc, bow, arrows, shield, doff)).await;
    Box::pin(don_and_validate_restore(&f, npc, shield)).await;
    f.pool.close().await;
    drop(f);
    sqlite_test_cleanup::remove_closed_file(&database)
        .await
        .unwrap();
    let _ = std::fs::remove_dir(directory);
}

async fn doff_and_reopen(
    f: &mut Fixture,
    npc: EntityId,
    database: &Path,
) -> (ItemId, ItemId, ItemId, CommandMeta) {
    let view = host_view(f).await;
    let shield = view.shield_options.unwrap().donned.unwrap();
    let weapons = view.attack_options.unwrap();
    let bow = weapons
        .weapons
        .iter()
        .find(|weapon| weapon.name == "Shortbow")
        .unwrap();
    assert_eq!(
        bow.source_features,
        vec![TableCreatureAttackChoice {
            feature_id: "shortbow".into(),
            label: "Shortbow".into(),
            weapon: Some(bow.item),
        }]
    );
    assert!(
        bow.purposes.contains(&WeaponAttackPurpose::Normal),
        "ordinary physical use remains legal"
    );
    let arrows = bow.ammunition[0].id;
    let bow = bow.item;
    for player in f.players {
        let private = f
            .runtime
            .table_view(f.campaign, TableViewer::Player(player))
            .await
            .unwrap()
            .tactical
            .unwrap();
        assert!(private.shield_options.is_none());
        assert!(private.attack_options.is_none());
    }
    let before = state(f).await;
    assert_eq!(armor(&before, npc), 15);
    let wrong = f.player_meta(0).await;
    assert!(
        f.runtime
            .execute_table(
                wrong,
                TableAction::Tactical {
                    action: TacticalAction::DoffShield
                }
            )
            .await
            .is_err()
    );
    assert_eq!(state(f).await, before);
    let meta = host_action(f, TacticalAction::DoffShield).await;
    let accepted = state(f).await;
    assert_eq!(armor(&accepted, npc), 13);
    assert_eq!(accepted.items, before.items);
    let loadout = accepted
        .rules
        .as_ref()
        .unwrap()
        .tactical_inventory
        .as_ref()
        .unwrap()
        .loadout(npc)
        .unwrap();
    assert_eq!(loadout.command, meta);
    assert_eq!(loadout.shield, None);
    assert_eq!(loadout.hands, WeaponLoadout::default());
    assert!(
        host_view(f).await.shield_options.is_none(),
        "the Action was spent"
    );
    f.pool.close().await;
    f.pool = dmd_persistence::open_sqlite_path(database).await.unwrap();
    f.runtime = CampaignRuntime::from_content_root(
        f.pool.clone(),
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../content"),
    );
    assert_eq!(
        f.runtime.resume_campaign(f.campaign).await.unwrap().state(),
        &accepted
    );
    assert!(
        f.runtime
            .execute_table(
                meta.clone(),
                TableAction::Tactical {
                    action: TacticalAction::DoffShield
                }
            )
            .await
            .unwrap()
            .already_accepted
    );
    assert_eq!(state(f).await, accepted);
    table_attack_cases::assert_restore(f).await;
    (shield, bow, arrows, meta)
}

fn bow_action(bow: ItemId, arrows: ItemId, target: EntityId, stow: bool) -> TacticalAction {
    TacticalAction::CreatureWeaponAttack {
        feature_id: "shortbow".into(),
        choice: CreatureWeaponUseChoice {
            weapon: bow,
            target,
            grip: WeaponGrip::TwoHands,
            ammunition: Some(arrows),
            equipment_change: Some(AttackEquipmentChange {
                timing: if stow {
                    EquipmentChangeTiming::AfterAttack
                } else {
                    EquipmentChangeTiming::BeforeAttack
                },
                operation: if stow {
                    AttackEquipmentOperation::Unequip { item: bow }
                } else {
                    AttackEquipmentOperation::Equip {
                        item: bow,
                        hand: Hand::Left,
                    }
                },
            }),
        },
    }
}

async fn shoot_and_stow(
    f: &Fixture,
    npc: EntityId,
    bow: ItemId,
    arrows: ItemId,
    shield: ItemId,
    stale: CommandMeta,
) {
    next_npc_turn(f).await;
    let before = state(f).await;
    let mut changed = stale;
    changed.id = CommandId::new();
    assert!(
        f.runtime
            .execute_table(
                changed,
                TableAction::Tactical {
                    action: bow_action(bow, arrows, f.actors[0], false)
                }
            )
            .await
            .is_err()
    );
    assert_eq!(state(f).await, before);
    let meta = f.meta(CommandIssuer::Admin, None, Some(f.session)).await;
    table_attack_cases::execute_after_restore(
        f,
        meta,
        TableAction::Tactical {
            action: bow_action(bow, arrows, f.actors[0], false),
        },
    )
    .await;
    let pending = state(f).await;
    assert_eq!(pending.items[&arrows].quantity, 19);
    assert!(host_view(f).await.shield_options.is_none());
    assert!(
        f.runtime
            .execute_table(
                f.meta(CommandIssuer::Admin, None, Some(f.session)).await,
                TableAction::Tactical {
                    action: TacticalAction::DonShield {
                        shield,
                        hand: Hand::Right
                    }
                }
            )
            .await
            .is_err()
    );
    assert_eq!(state(f).await, pending);
    Box::pin(table_hit_driver::roll_then_decline(f, true, &[15])).await;
    table_attack_cases::assert_restore(f).await;
    table_attack_cases::submit(f, true, &[1]).await;
    let hit = state(f).await;
    assert_eq!(
        hit.rules.as_ref().unwrap().entities[&f.actors[0]].hp,
        before.rules.as_ref().unwrap().entities[&f.actors[0]].hp - 3
    );
    next_npc_turn(f).await;
    assert!(
        host_view(f).await.shield_options.is_none(),
        "bow occupies both hands"
    );
    let before = state(f).await;
    assert!(
        f.runtime
            .execute_table(
                f.meta(CommandIssuer::Admin, None, Some(f.session)).await,
                TableAction::Tactical {
                    action: TacticalAction::DonShield {
                        shield,
                        hand: Hand::Right
                    }
                }
            )
            .await
            .is_err()
    );
    assert_eq!(state(f).await, before);
    host_action(f, bow_action(bow, arrows, f.actors[0], true)).await;
    table_attack_cases::submit(f, true, &[1]).await;
    let stowed = state(f).await;
    assert_eq!(stowed.items[&arrows].quantity, 18);
    assert_eq!(
        stowed
            .rules
            .as_ref()
            .unwrap()
            .tactical_inventory
            .as_ref()
            .unwrap()
            .loadout(npc)
            .unwrap()
            .hands,
        WeaponLoadout::default()
    );
    next_npc_turn(f).await;
}

async fn don_and_validate_restore(f: &Fixture, npc: EntityId, shield: ItemId) {
    let options = host_view(f).await.shield_options.unwrap();
    assert_eq!(options.actor, npc);
    assert_eq!(options.donned, None);
    assert_eq!(
        options.shields,
        vec![TableShieldChoice {
            item: shield,
            hands: vec![Hand::Left, Hand::Right]
        }]
    );
    let meta = host_action(
        f,
        TacticalAction::DonShield {
            shield,
            hand: Hand::Right,
        },
    )
    .await;
    let accepted = state(f).await;
    assert_eq!(armor(&accepted, npc), 15);
    let loadout = accepted
        .rules
        .as_ref()
        .unwrap()
        .tactical_inventory
        .as_ref()
        .unwrap()
        .loadout(npc)
        .unwrap();
    assert_eq!(loadout.command, meta);
    assert_eq!(loadout.shield, Some(shield));
    assert_eq!(accepted.items[&shield].custody, Custody::Entity(npc));
    table_attack_cases::assert_restore(f).await;
    let export = export_campaign(&f.pool, f.campaign).await.unwrap();
    let unrelated = export
        .event_journal
        .iter()
        .find_map(|row| {
            let event: TableEvent = serde_json::from_str(&row.payload_json).ok()?;
            (matches!(
                event.action,
                TableAction::Tactical {
                    action: TacticalAction::EndTurn
                }
            ) && event.meta.issuer == CommandIssuer::Admin)
                .then_some(event.meta)
        })
        .unwrap();
    for kind in 0..3 {
        let mut corrupted = export.clone();
        let mut changed = accepted.clone();
        if kind == 0 {
            changed
                .rules
                .as_mut()
                .unwrap()
                .tactical_inventory
                .as_mut()
                .unwrap()
                .loadouts
                .iter_mut()
                .find(|entry| entry.actor == npc)
                .unwrap()
                .command = unrelated.clone();
        } else {
            changed
                .rules
                .as_mut()
                .unwrap()
                .entities
                .get_mut(&npc)
                .unwrap()
                .armor = ArmorClass::Fixed(17);
        }
        if kind == 2 {
            assert!(
                corrupted
                    .snapshots
                    .iter()
                    .all(|snapshot| snapshot.event_sequence
                        != corrupted.current_state.applied_event_sequence)
            );
            corrupted.snapshots.push(dmd_persistence::SnapshotRow {
                campaign_id: corrupted.current_state.campaign_id.clone(),
                event_sequence: corrupted.current_state.applied_event_sequence,
                state_schema_version: corrupted.current_state.schema_version,
                state_json: changed.encode_json().unwrap(),
                created_at_utc: "2026-09-25 00:00:00".into(),
            });
        } else {
            corrupted.current_state.state_json = changed.encode_json().unwrap();
        }
        assert!(
            corrupted.upgraded().is_ok(),
            "corruption {kind} must reach application source/history validation"
        );
        let pool = open_sqlite("sqlite::memory:").await.unwrap();
        let runtime = CampaignRuntime::from_content_root(
            pool.clone(),
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../content"),
        );
        assert!(
            runtime.restore_campaign(&corrupted).await.is_err(),
            "corruption {kind}"
        );
        for table in [
            "campaign_state_current",
            "campaign_lifecycle",
            "event_journal",
            "command_audit",
            "campaign_snapshots",
        ] {
            let count: i64 = sqlx::query_scalar(&format!("SELECT COUNT(*) FROM {table}"))
                .fetch_one(&pool)
                .await
                .unwrap();
            assert_eq!(count, 0, "failed restore leaked rows into {table}");
        }
        pool.close().await;
    }
}

#[tokio::test]
async fn trained_player_shield_actions_preserve_defense_style_and_ownership_projection() {
    let mut creation = input("Shield bearer");
    creation.purchases.push(EquipmentChoice {
        item_id: "shield".into(),
        quantity: 1,
    });
    creation.shield = true;
    let mut f = Box::pin(Fixture::with_creation(
        TableContract::default(),
        Some(creation),
    ))
    .await;
    Box::pin(table_attack_cases::prepare_at(
        &mut f,
        SpatialPoint { x: 60, y: 10, z: 0 },
    ))
    .await;
    let view = f
        .runtime
        .table_view(f.campaign, TableViewer::Player(f.players[0]))
        .await
        .unwrap()
        .tactical
        .unwrap();
    let shield = view.shield_options.unwrap().donned.unwrap();
    let before = state(&f).await;
    assert_eq!(armor(&before, f.actors[0]), 16);
    f.runtime
        .execute_table(
            f.player_meta(0).await,
            TableAction::Tactical {
                action: TacticalAction::DoffShield,
            },
        )
        .await
        .unwrap();
    assert_eq!(
        armor(&state(&f).await, f.actors[0]),
        14,
        "leather and Defense remain"
    );
    end_player(&f).await;
    host_action(&f, TacticalAction::EndTurn).await;
    f.runtime
        .execute_table(
            f.player_meta(0).await,
            TableAction::Tactical {
                action: TacticalAction::DonShield {
                    shield,
                    hand: Hand::Left,
                },
            },
        )
        .await
        .unwrap();
    assert_eq!(armor(&state(&f).await, f.actors[0]), 16);
    table_attack_cases::assert_restore(&f).await;
    f.pool.close().await;
}
