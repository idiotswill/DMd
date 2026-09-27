use super::*;
use dmd_rules::tactical::TacticalAction;

fn runtime(pool: sqlx::SqlitePool) -> CampaignRuntime {
    CampaignRuntime::from_content_root(
        pool,
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../content"),
    )
}
async fn view(f: &Fixture, player: usize) -> TablePresentedView {
    f.runtime
        .presented_table_view(f.campaign, TableViewer::Player(f.players[player]))
        .await
        .unwrap()
}
async fn state(f: &Fixture) -> CampaignState {
    f.runtime
        .open_campaign(f.campaign)
        .await
        .unwrap()
        .state()
        .clone()
}
fn channel(f: &Fixture, player: usize) -> TableTransportChannel {
    TableTransportChannel::Player {
        player_id: f.players[player],
        character_id: f.characters[player],
    }
}

async fn prepare(f: &mut Fixture) -> EntityId {
    // Start immutable presentation history before this new feature is used.
    view(f, 0).await;
    let target = Box::pin(table_attack_cases::prepare_at(
        f,
        SpatialPoint { x: 20, y: 10, z: 0 },
    ))
    .await;
    let options = view(f, 0).await.tactical.unwrap().attack_options.unwrap();
    let weapon = options
        .weapons
        .iter()
        .find(|weapon| weapon.name == "Dagger")
        .unwrap()
        .item;
    let action = TacticalAction::Attack {
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
    };
    f.runtime
        .execute_table(f.player_meta(0).await, TableAction::Tactical { action })
        .await
        .unwrap();
    Box::pin(table_hit_driver::roll_then_decline(f, false, &[20])).await;
    target
}

async fn play(f: &mut Fixture, url: &str, target: EntityId) {
    let before = export_campaign(&f.pool, f.campaign).await.unwrap();
    let original_state = state(f).await;
    f.pool.close().await;
    f.pool = open_sqlite(url).await.unwrap();
    f.runtime = runtime(f.pool.clone());
    let current = view(f, 0).await;
    let pending = current.roll.as_ref().unwrap();
    assert_eq!(pending.dice, vec![DieSpec { count: 2, sides: 4 }]);
    let query = TableRollOptionsRequest {
        campaign_id: f.campaign,
        channel: channel(f, 0),
        revision: current.revision,
        roll_id: pending.id,
    };
    let options = f
        .runtime
        .table_roll_options(query.clone())
        .await
        .unwrap()
        .savage_attacker
        .unwrap();
    assert_eq!(options.weapon_dice, 2);
    let foreign = view(f, 1).await;
    let mut forged = query.clone();
    forged.channel = channel(f, 1);
    forged.revision = foreign.revision;
    assert!(f.runtime.table_roll_options(forged).await.is_err());
    let mut after_query = export_campaign(&f.pool, f.campaign).await.unwrap();
    after_query.exported_at_utc = before.exported_at_utc.clone();
    assert_eq!(
        before, after_query,
        "roll-option reads must not write presentation or game state"
    );

    let raw = |values: [u16; 2]| RollResult {
        request_id: pending.id,
        source: RollSource::Physical,
        dice: values
            .into_iter()
            .map(|value| DieResult { sides: 4, value })
            .collect(),
    };
    let request = TableTransportRequest {
        version: TABLE_TRANSPORT_VERSION,
        command_id: CommandId::new(),
        campaign_id: f.campaign,
        session_id: Some(f.session),
        channel: channel(f, 0),
        revision: current.revision,
        input: TableTransportInput::Action(Box::new(TableAction::Tactical {
            action: TacticalAction::SubmitSavageAttacker {
                roll: SavageAttackerRoll {
                    weapon_dice: Some(2),
                    first: raw([1, 2]),
                    second: raw([4, 4]),
                    chosen: DamageRollChoice::First,
                    inspiration: None,
                },
            },
        })),
    };
    let mirror_pool = open_sqlite("sqlite::memory:").await.unwrap();
    let mirror = runtime(mirror_pool.clone());
    mirror.restore_campaign(&before).await.unwrap();
    assert_eq!(
        mirror
            .table_roll_options(query.clone())
            .await
            .unwrap()
            .savage_attacker,
        Some(options)
    );
    let accepted = f
        .runtime
        .submit_presented_table(request.clone())
        .await
        .unwrap();
    let mirrored = mirror
        .submit_presented_table(request.clone())
        .await
        .unwrap();
    assert_eq!(
        mirror
            .submit_presented_table(request.clone())
            .await
            .unwrap(),
        mirrored
    );
    let final_state = state(f).await;
    assert_eq!(
        mirror.open_campaign(f.campaign).await.unwrap().state(),
        &final_state
    );
    let rules = final_state.rules.as_ref().unwrap();
    assert_eq!(
        rules.entities[&target].hp,
        original_state.rules.as_ref().unwrap().entities[&target].hp - 6
    );
    let record = rules.rolls.last().unwrap();
    assert_eq!(
        record.savage_attacker.as_ref().unwrap().chosen,
        DamageRollChoice::First
    );
    assert_ne!(
        record.request.id, pending.id,
        "opaque handles must normalize to the original canonical roll"
    );
    assert_eq!(
        rules.entities[&f.actors[0]]
            .character_features
            .as_ref()
            .unwrap()
            .savage_attacker_turn,
        Some(1)
    );
    let saved = export_campaign(&f.pool, f.campaign).await.unwrap();
    f.pool.close().await;
    f.pool = open_sqlite(url).await.unwrap();
    f.runtime = runtime(f.pool.clone());
    assert_eq!(
        f.runtime
            .submit_presented_table(request.clone())
            .await
            .unwrap(),
        accepted
    );
    assert!(f.runtime.table_roll_options(query).await.is_err());
    let mut changed = request;
    let TableTransportInput::Action(action) = &mut changed.input else {
        panic!()
    };
    let TableAction::Tactical {
        action: TacticalAction::SubmitSavageAttacker { roll },
    } = action.as_mut()
    else {
        panic!()
    };
    roll.chosen = DamageRollChoice::Second;
    assert!(f.runtime.submit_presented_table(changed).await.is_err());
    let mut after = export_campaign(&f.pool, f.campaign).await.unwrap();
    after.exported_at_utc = saved.exported_at_utc.clone();
    assert_eq!(after, saved);
    mirror_pool.close().await;
}

async fn reject_forgery(f: &Fixture) {
    for variation in 0..2 {
        let mut exported = export_campaign(&f.pool, f.campaign).await.unwrap();
        let mut forged = state(f).await;
        let rules = forged.rules.as_mut().unwrap();
        if variation == 0 {
            rules
                .entities
                .get_mut(&f.actors[0])
                .unwrap()
                .character_features
                .as_mut()
                .unwrap()
                .savage_attacker_turn = None;
        } else {
            rules
                .rolls
                .last_mut()
                .unwrap()
                .savage_attacker
                .as_mut()
                .unwrap()
                .second
                .dice[0]
                .value = 3;
        }
        let pack = dmd_rules::RulesPack::from_json(include_str!(
            "../../../../content/srd-5.2.1/kernel.json"
        ))
        .unwrap();
        dmd_rules::validate_state(&forged, &pack).unwrap();
        dmd_rules::tactical::validate_tactical_state(&forged).unwrap();
        exported.current_state.state_json = forged.encode_json().unwrap();
        let pool = open_sqlite("sqlite::memory:").await.unwrap();
        assert!(
            runtime(pool.clone())
                .restore_campaign(&exported)
                .await
                .is_err()
        );
        let rows: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM campaign_state_current")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(rows, 0);
        pool.close().await;
    }
}

#[tokio::test]
async fn tactical_savage_survives_owned_opaque_dice_cold_retry_and_hostile_restore() {
    let directory = std::env::temp_dir().join(format!("dmd-savage-{}", CampaignId::new().0));
    std::fs::create_dir_all(&directory).unwrap();
    let path = directory.join("campaign.sqlite");
    let url = format!("sqlite://{}", path.to_string_lossy().replace('\\', "/"));
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
    let target = Box::pin(prepare(&mut f)).await;
    Box::pin(play(&mut f, &url, target)).await;
    Box::pin(reject_forgery(&f)).await;
    f.pool.close().await;
    drop(f);
    sqlite_test_cleanup::remove_closed_directory(&directory)
        .await
        .unwrap();
}
