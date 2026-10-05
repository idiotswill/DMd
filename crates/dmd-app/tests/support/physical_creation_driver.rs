//! Real setup and one-command cold/portable execution for physical creation tests.
use super::*;
use sqlx::Row;

pub(super) fn runtime(pool: sqlx::SqlitePool) -> CampaignRuntime {
    CampaignRuntime::from_content_root(
        pool,
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../content"),
    )
}
pub(super) fn action(value: TableAction) -> TableTransportInput {
    TableTransportInput::Action(Box::new(value))
}
pub(super) fn tactical(value: TacticalAction) -> TableTransportInput {
    action(TableAction::Tactical { action: value })
}
pub(super) fn player(f: &Fixture, index: usize) -> TableTransportChannel {
    TableTransportChannel::Player {
        player_id: f.players[index],
        character_id: f.characters[index],
    }
}
pub(super) async fn state(f: &Fixture) -> CampaignState {
    f.runtime
        .open_campaign(f.campaign)
        .await
        .unwrap()
        .state()
        .clone()
}
pub(super) async fn view(f: &Fixture, channel: &TableTransportChannel) -> TablePresentedView {
    let viewer = match channel {
        TableTransportChannel::Host => TableViewer::Host,
        TableTransportChannel::Player { player_id, .. }
        | TableTransportChannel::SourceCreature { player_id, .. } => {
            TableViewer::Player(*player_id)
        }
    };
    f.runtime
        .presented_table_view(f.campaign, viewer)
        .await
        .unwrap()
}
pub(super) async fn request(
    f: &Fixture,
    channel: TableTransportChannel,
    input: TableTransportInput,
) -> TableTransportRequest {
    let view = view(f, &channel).await;
    let session_id = match &input {
        TableTransportInput::Action(action) => match action.as_ref() {
            TableAction::StartSession { id, .. } => Some(*id),
            _ => view
                .active_session
                .as_ref()
                .map(|session| session.session_id),
        },
        _ => view
            .active_session
            .as_ref()
            .map(|session| session.session_id),
    };
    TableTransportRequest {
        version: TABLE_TRANSPORT_VERSION,
        command_id: CommandId::new(),
        campaign_id: f.campaign,
        session_id,
        channel,
        revision: view.revision,
        input,
    }
}
pub(super) async fn reopen(f: &mut Fixture, path: &Path) {
    f.pool.close().await;
    f.pool = dmd_persistence::open_sqlite_path(path).await.unwrap();
    f.runtime = runtime(f.pool.clone());
    Box::pin(f.runtime.resume_campaign(f.campaign))
        .await
        .unwrap();
}
pub(super) async fn cold_step(f: &mut Fixture, path: &Path, request: TableTransportRequest) {
    let prefix = export_campaign(&f.pool, f.campaign).await.unwrap();
    let mirror_pool = open_sqlite("sqlite::memory:").await.unwrap();
    let mirror = runtime(mirror_pool.clone());
    Box::pin(mirror.restore_campaign(&prefix)).await.unwrap();
    Box::pin(reopen(f, path)).await;
    let first = Box::pin(f.runtime.submit_presented_table(request.clone()))
        .await
        .unwrap();
    let second = Box::pin(mirror.submit_presented_table(request.clone()))
        .await
        .unwrap();
    let (
        TableTransportResult::Accepted(first_receipt),
        TableTransportResult::Accepted(second_receipt),
    ) = (&first, &second)
    else {
        panic!("accepted command required")
    };
    assert_eq!(first_receipt.outcome, second_receipt.outcome);
    assert_eq!(
        mirror.open_campaign(f.campaign).await.unwrap().state(),
        &state(f).await
    );
    assert_eq!(
        Box::pin(mirror.submit_presented_table(request.clone()))
            .await
            .unwrap(),
        second
    );
    let after = export_campaign(&mirror_pool, f.campaign).await.unwrap();
    let second_pool = open_sqlite("sqlite::memory:").await.unwrap();
    Box::pin(runtime(second_pool.clone()).restore_campaign(&after))
        .await
        .unwrap();
    second_pool.close().await;
    mirror_pool.close().await;
    Box::pin(reopen(f, path)).await;
    let saved = export_campaign(&f.pool, f.campaign).await.unwrap();
    assert_eq!(
        Box::pin(f.runtime.submit_presented_table(request.clone()))
            .await
            .unwrap(),
        first
    );
    let mut altered = request;
    altered.input = action(TableAction::AddPlayer {
        id: PlayerId::new(),
        name: "Altered retry".into(),
    });
    assert!(
        Box::pin(f.runtime.submit_presented_table(altered))
            .await
            .is_err()
    );
    let mut after = export_campaign(&f.pool, f.campaign).await.unwrap();
    after.exported_at_utc = saved.exported_at_utc.clone();
    assert_eq!(after, saved);
}
pub(super) async fn step(
    f: &mut Fixture,
    path: &Path,
    channel: TableTransportChannel,
    input: TableTransportInput,
) -> TableTransportRequest {
    let request = request(f, channel, input).await;
    Box::pin(cold_step(f, path, request.clone())).await;
    request
}
pub(super) async fn reject(f: &Fixture, request: TableTransportRequest) {
    let before = all_rows(&f.pool).await;
    assert!(
        Box::pin(f.runtime.submit_presented_table(request))
            .await
            .is_err()
    );
    assert_eq!(all_rows(&f.pool).await, before);
}
// Include every table and every value, not merely campaign/current row counts.
pub(super) async fn all_rows(pool: &sqlx::SqlitePool) -> Vec<(String, Vec<Vec<String>>)> {
    let tables: Vec<String> =
        sqlx::query_scalar("SELECT name FROM sqlite_master WHERE type = 'table' ORDER BY name")
            .fetch_all(pool)
            .await
            .unwrap();
    let mut result = Vec::new();
    for table in tables {
        let quoted = format!("\"{}\"", table.replace('"', "\"\""));
        let columns = sqlx::query(&format!("PRAGMA table_info({quoted})"))
            .fetch_all(pool)
            .await
            .unwrap();
        let expressions: Vec<_> = columns
            .iter()
            .map(|column| {
                let name: String = column.get("name");
                format!("quote(\"{}\")", name.replace('"', "\"\""))
            })
            .collect();
        let rows = sqlx::query(&format!("SELECT {} FROM {quoted}", expressions.join(",")))
            .fetch_all(pool)
            .await
            .unwrap();
        let mut values: Vec<Vec<String>> = rows
            .iter()
            .map(|row| (0..expressions.len()).map(|index| row.get(index)).collect())
            .collect();
        values.sort();
        result.push((table, values));
    }
    result
}
pub(super) async fn blank(path: &Path) -> Fixture {
    let pool = dmd_persistence::open_sqlite_path(path).await.unwrap();
    let mut f = Fixture {
        runtime: runtime(pool.clone()),
        pool,
        campaign: CampaignId::new(),
        players: [PlayerId::new(), PlayerId::new()],
        characters: [CharacterId::new(), CharacterId::new()],
        actors: [EntityId::new(), EntityId::new()],
        session: PlaySessionId::new(),
    };
    f.runtime
        .create_table_campaign(
            f.campaign,
            "Physical purchase campaign",
            TableContract::default(),
        )
        .await
        .unwrap();
    for index in 0..2 {
        let input = action(TableAction::AddPlayer {
            id: f.players[index],
            name: format!("Player {index}"),
        });
        Box::pin(step(&mut f, path, TableTransportChannel::Host, input)).await;
    }
    f
}
pub(super) async fn create(f: &mut Fixture, path: &Path, weapon: &str) -> TableTransportRequest {
    let options = f
        .runtime
        .character_creation_options(f.campaign)
        .await
        .unwrap();
    assert_eq!(options.catalog.items.len(), 18);
    let mut first_request = None;
    for index in 0..2 {
        let mut selected = input(&format!("Character {index}"));
        if index == 0 {
            selected.purchases.push(EquipmentChoice {
                item_id: weapon.into(),
                quantity: 1,
            });
            selected.masteries = ["greatsword".into(), "glaive".into(), "dagger".into()];
        }
        let input = action(TableAction::CreateCharacterFromSource {
            character_id: f.characters[index],
            entity_id: f.actors[index],
            player_id: f.players[index],
            source: options.source.clone(),
            input: selected,
        });
        let accepted = Box::pin(step(f, path, TableTransportChannel::Host, input)).await;
        if index == 0 {
            first_request = Some(accepted);
        }
    }
    let created = state(f).await;
    assert!(created.items.is_empty());
    let profile = &created.table.as_ref().unwrap().character_profiles[&f.characters[0]];
    assert_eq!(profile.creation_source.as_ref(), Some(&options.source));
    assert_eq!(
        profile.money_cp,
        20500 - 1000 - if weapon == "glaive" { 2000 } else { 5000 }
    );
    assert!(
        created.rules.as_ref().unwrap().entities[&f.actors[0]]
            .attacks
            .is_empty()
    );
    let first_request = first_request.unwrap();
    let reused_identity =
        request(f, TableTransportChannel::Host, first_request.input.clone()).await;
    Box::pin(reject(f, reused_identity)).await;
    first_request
}
pub(super) async fn prepare(f: &mut Fixture, path: &Path, distance: i32) -> ItemId {
    for index in 0..2 {
        let current = view(f, &TableTransportChannel::Host).await;
        let count = current
            .characters
            .iter()
            .find(|character| character.character_id == f.characters[index])
            .unwrap()
            .equipment
            .as_ref()
            .unwrap()
            .initial_item_count;
        let input = action(TableAction::PrepareEquipment {
            character_id: f.characters[index],
            item_ids: (0..count).map(|_| ItemId::new()).collect(),
        });
        Box::pin(step(f, path, TableTransportChannel::Host, input.clone())).await;
        let second_grant = request(f, TableTransportChannel::Host, input).await;
        Box::pin(reject(f, second_grant)).await;
    }
    let created = state(f).await;
    let weapon = created
        .items
        .values()
        .find(|item| ["glaive", "greatsword"].contains(&item.definition_id.as_str()))
        .unwrap();
    assert_eq!(weapon.quantity, 1);
    assert_eq!(weapon.custody, Custody::Entity(f.actors[0]));
    let weapon_id = weapon.id;
    let receipt = created
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
    assert_eq!(
        receipt.creation_profile,
        created.table.as_ref().unwrap().character_profiles[&f.characters[0]]
    );
    let start = action(TableAction::StartSession {
        id: f.session,
        name: "Practice".into(),
        participants: (0..2)
            .map(|index| SessionParticipant {
                player_id: f.players[index],
                character_id: Some(f.characters[index]),
                attendance: AttendanceStatus::Present,
            })
            .collect(),
    });
    Box::pin(step(f, path, TableTransportChannel::Host, start)).await;
    let setup = action(TableAction::PrepareBattlefield {
        setup: Box::new(TableBattlefieldSetup {
            encounter_id: EncounterId::new(),
            scene_id: SceneId::new(),
            location_id: LocationId::new(),
            name: "Practice floor".into(),
            area_grid_policy: None,
            battlefield: Battlefield {
                bounds: SpatialBox {
                    min: SpatialPoint { x: 0, y: 0, z: 0 },
                    max: SpatialPoint {
                        x: 100,
                        y: 100,
                        z: 40,
                    },
                },
                floor_z: 0,
                floor_surface: "stone".into(),
                ambient_light: LightLevel::Bright,
                terrain: vec![],
                obstacles: vec![],
                lights: vec![],
            },
            characters: (0..2)
                .map(|index| TableCharacterPlacement {
                    character_id: f.characters[index],
                    position: point(if index == 0 { 10 } else { 10 + distance }),
                    height: 12,
                    allies: vec![],
                    enemies: vec![f.actors[1 - index]],
                })
                .collect(),
            creatures: vec![],
            geometry_ruling: Ruling {
                basis: RulingBasis::GmAdjudication,
                reason: "A level, illuminated practice floor with known opposition.".into(),
            },
        }),
    });
    Box::pin(step(f, path, TableTransportChannel::Host, setup)).await;
    let begin = tactical(TacticalAction::Begin {
        execution: TacticalExecutionVersion::EncounterReleaseV1,
        combatants: f
            .actors
            .iter()
            .map(|actor| TacticalCombatant {
                actor: *actor,
                source: TacticalSource::Character,
                surprised: false,
            })
            .collect(),
        groups: f
            .actors
            .iter()
            .map(|actor| InitiativeGroup {
                actors: vec![*actor],
                request_id: RollRequestId::new(),
            })
            .collect(),
    });
    Box::pin(step(f, path, TableTransportChannel::Host, begin)).await;
    Box::pin(roll(f, path, 0, &[18])).await;
    Box::pin(roll(f, path, 1, &[2])).await;
    weapon_id
}
pub(super) fn point(x: i32) -> SpatialPoint {
    SpatialPoint { x, y: 10, z: 0 }
}
pub(super) fn choice(f: &Fixture, weapon: ItemId, equip: bool) -> WeaponUseChoice {
    WeaponUseChoice {
        weapon,
        target: f.actors[1],
        delivery: WeaponDelivery::Melee,
        ability: Ability::Strength,
        grip: WeaponGrip::TwoHands,
        purpose: WeaponAttackPurpose::Normal,
        ammunition: None,
        after_equipment: None,
        equipment_change: equip.then_some(AttackEquipmentChange {
            timing: EquipmentChangeTiming::BeforeAttack,
            operation: AttackEquipmentOperation::Equip {
                item: weapon,
                hand: Hand::Right,
            },
        }),
    }
}
pub(super) async fn roll(f: &mut Fixture, path: &Path, index: usize, values: &[u16]) {
    let channel = player(f, index);
    let request = view(f, &channel).await.roll.unwrap();
    assert_eq!(request.mode, RollMode::Normal);
    let sides: Vec<_> = request
        .dice
        .iter()
        .flat_map(|die| std::iter::repeat_n(die.sides, usize::from(die.count)))
        .collect();
    assert_eq!(sides.len(), values.len());
    let input = tactical(TacticalAction::SubmitRoll {
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
    });
    Box::pin(step(f, path, channel, input)).await;
}
pub(super) async fn decline_hit(f: &mut Fixture, path: &Path, turn_owner: usize) {
    let order_owner = player(f, turn_owner);
    let order = view(f, &order_owner)
        .await
        .tactical
        .unwrap()
        .hit
        .unwrap()
        .order
        .unwrap();
    Box::pin(step(
        f,
        path,
        order_owner,
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
    let target_owner = player(f, 1);
    let response = view(f, &target_owner)
        .await
        .tactical
        .unwrap()
        .hit
        .unwrap()
        .response
        .unwrap();
    assert_eq!(response.actor, f.actors[1]);
    Box::pin(step(
        f,
        path,
        target_owner,
        TableTransportInput::HitResponse {
            handle: response.key,
            decision: Box::new(TableHitInput::Respond { accept: false }),
        },
    ))
    .await;
}
pub(super) async fn close(f: Fixture, path: &Path) {
    f.pool.close().await;
    drop(f);
    sqlite_test_cleanup::remove_closed_file(path).await.unwrap();
}
