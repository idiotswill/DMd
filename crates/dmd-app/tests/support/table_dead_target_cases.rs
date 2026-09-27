use super::*;
use dmd_rules::tactical::TacticalAction;

fn action(action: TacticalAction) -> TableAction {
    TableAction::Tactical { action }
}
fn content() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../content")
}
async fn state(f: &Fixture) -> CampaignState {
    f.runtime
        .open_campaign(f.campaign)
        .await
        .unwrap()
        .state()
        .clone()
}
async fn export(f: &Fixture) -> dmd_persistence::CampaignExport {
    let mut export = export_campaign(&f.pool, f.campaign).await.unwrap();
    export.exported_at_utc.clear();
    export
}
async fn reopen(f: &mut Fixture, path: &Path) {
    f.pool.close().await;
    f.pool = dmd_persistence::open_sqlite_path(path).await.unwrap();
    f.runtime = CampaignRuntime::from_content_root(f.pool.clone(), content());
    f.runtime.resume_campaign(f.campaign).await.unwrap();
}
async fn player_action(f: &Fixture, request: TacticalAction) -> CommandMeta {
    let meta = f.player_meta(0).await;
    f.runtime
        .execute_table(meta.clone(), action(request))
        .await
        .unwrap();
    meta
}
fn dagger(weapon: ItemId, target: EntityId, equip: bool) -> WeaponUseChoice {
    WeaponUseChoice {
        weapon,
        target,
        delivery: WeaponDelivery::Melee,
        ability: Ability::Strength,
        grip: WeaponGrip::OneHand(Hand::Right),
        purpose: WeaponAttackPurpose::Normal,
        ammunition: None,
        equipment_change: equip.then_some(AttackEquipmentChange {
            timing: EquipmentChangeTiming::BeforeAttack,
            operation: AttackEquipmentOperation::Equip {
                item: weapon,
                hand: Hand::Right,
            },
        }),
    }
}

async fn lethal_attack(f: &Fixture, target: EntityId) -> (ItemId, CommandMeta, TableAction) {
    let before = state(f).await;
    let entity = &before.rules.as_ref().unwrap().entities[&target];
    assert_eq!(entity.hp, 10, "genuine Goblin Warrior source HP");
    assert!(!entity.death.dead);
    assert!(!entity.uses_death_saves);
    let view = f
        .runtime
        .table_view(f.campaign, TableViewer::Player(f.players[0]))
        .await
        .unwrap();
    let options = view.tactical.unwrap().attack_options.unwrap();
    let weapon = options
        .weapons
        .iter()
        .find(|weapon| weapon.name == "Dagger")
        .unwrap()
        .item;
    let request = action(TacticalAction::Attack {
        choice: dagger(weapon, target, true),
    });
    let origin = f.player_meta(0).await;
    f.runtime
        .execute_table(origin.clone(), request.clone())
        .await
        .unwrap();
    // Actual critical hit and its two damage dice; no supplied HP or fabricated effect.
    Box::pin(table_hit_driver::roll_then_decline(f, false, &[20])).await;
    table_attack_cases::submit(f, false, &[4, 4]).await;
    let view = f
        .runtime
        .table_view(f.campaign, TableViewer::Player(f.players[0]))
        .await
        .unwrap();
    assert_eq!(
        view.tactical.unwrap().attack_decision.unwrap().kind,
        TableAttackDecisionKind::Knockout
    );
    Box::pin(table_attack_cases::assert_restore(f)).await;
    player_action(
        f,
        TacticalAction::ChooseAttackKnockout {
            choice: KnockoutChoice::NormalDamage,
        },
    )
    .await;
    let current = state(f).await;
    assert!(current.rules.as_ref().unwrap().entities[&target].death.dead);
    assert_eq!(current.entities[&target].existence, EntityExistence::Dead);
    assert!(
        current
            .encounter
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .resolution
            .is_none()
    );
    assert_eq!(f.runtime.replay_rules(f.campaign).await.unwrap(), current);
    Box::pin(table_attack_cases::assert_restore(f)).await;
    (weapon, origin, request)
}

async fn next_player_turn(f: &mut Fixture, target: EntityId) {
    player_action(f, TacticalAction::EndTurn).await;
    let current = state(f).await;
    let timing = current.rules.as_ref().unwrap().timing.as_ref().unwrap();
    assert_eq!(timing.order[timing.index].actor, target);
    // The host advances the dead source actor's boundary; no new action is invented.
    f.host(action(TacticalAction::EndTurn), Some(f.session))
        .await;
    let current = state(f).await;
    let timing = current.rules.as_ref().unwrap().timing.as_ref().unwrap();
    assert_eq!(timing.order[timing.index].actor, f.actors[0]);
    assert_eq!(timing.round, 2);
    assert!(!timing.action_spent);
    assert!(!timing.bonus_action_spent);
    assert!(
        current
            .encounter
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .resolution
            .is_none()
    );
}

async fn reject_new_attack(f: &mut Fixture, path: &Path, target: EntityId, weapon: ItemId) {
    let view = f
        .runtime
        .table_view(f.campaign, TableViewer::Player(f.players[0]))
        .await
        .unwrap();
    let options = view.tactical.unwrap().attack_options.unwrap();
    assert_eq!(options.actor, f.actors[0]);
    assert!(
        options
            .targets
            .iter()
            .any(|candidate| candidate.actor == target),
        "located contact remains visible without filtering on secret vitality"
    );
    let before = state(f).await;
    let durable = export(f).await;
    let meta = f.player_meta(0).await;
    let request = action(TacticalAction::Attack {
        choice: dagger(weapon, target, false),
    });
    let error = f
        .runtime
        .execute_table(meta.clone(), request.clone())
        .await
        .unwrap_err();
    assert!(
        matches!(error, RunnableCampaignError::TableRejected(_)),
        "{error}"
    );
    assert_eq!(state(f).await, before);
    assert_eq!(
        export(f).await,
        durable,
        "no campaign, journal, audit, session or snapshot write"
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
    assert_eq!(
        before
            .rules
            .as_ref()
            .unwrap()
            .tactical_inventory
            .as_ref()
            .unwrap()
            .loadout(f.actors[0])
            .unwrap()
            .hands
            .hands[Hand::Right.index()],
        HandAssignment::Item(weapon)
    );
    Box::pin(reopen(f, path)).await;
    assert!(f.runtime.execute_table(meta, request).await.is_err());
    assert_eq!(export(f).await, durable);
    assert_eq!(f.runtime.replay_rules(f.campaign).await.unwrap(), before);
    Box::pin(table_attack_cases::assert_restore(f)).await;
}

#[tokio::test]
async fn lethal_ordinary_attack_remains_restorable_but_fresh_dead_target_attack_writes_nothing() {
    Box::pin(run_case()).await;
}
async fn run_case() {
    let path = std::env::temp_dir().join(format!("dmd-dead-target-{}.sqlite", CommandId::new().0));
    let pool = dmd_persistence::open_sqlite_path(&path).await.unwrap();
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
    let target = Box::pin(table_attack_cases::prepare_at(
        &mut f,
        SpatialPoint { x: 20, y: 10, z: 0 },
    ))
    .await;
    let (weapon, original, accepted) = Box::pin(lethal_attack(&f, target)).await;
    let durable = export(&f).await;
    Box::pin(reopen(&mut f, &path)).await;
    assert!(
        f.runtime
            .execute_table(original, accepted)
            .await
            .unwrap()
            .already_accepted
    );
    assert_eq!(
        export(&f).await,
        durable,
        "retry cannot repeat lethal damage or equipment costs"
    );
    Box::pin(next_player_turn(&mut f, target)).await;
    Box::pin(reject_new_attack(&mut f, &path, target, weapon)).await;
    f.pool.close().await;
    drop(f);
    sqlite_test_cleanup::remove_closed_file(&path)
        .await
        .unwrap();
}
