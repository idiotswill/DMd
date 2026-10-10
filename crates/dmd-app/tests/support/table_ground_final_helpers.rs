//! Additive application controls; positive state only comes from accepted commands.
use super::*;
use std::collections::BTreeMap;
use std::path::PathBuf;

pub(super) async fn file_fixture(label: &str) -> (Fixture, PathBuf, PathBuf) {
    let directory =
        std::env::temp_dir().join(format!("dmd-ground-{label}-{}", CampaignId::new().0));
    std::fs::create_dir_all(&directory).unwrap();
    let path = directory.join("campaign.sqlite");
    let pool = dmd_persistence::open_sqlite_path(&path).await.unwrap();
    let mut creation = input("Actual ground controller");
    creation.purchases.push(EquipmentChoice {
        item_id: "dagger".into(),
        quantity: 2,
    });
    let fixture = Box::pin(Fixture::with_creation_pool(
        TableContract::default(),
        Some(creation),
        pool,
    ))
    .await;
    (fixture, directory, path)
}

pub(super) async fn close_fixture(f: Fixture, directory: &Path) {
    f.pool.close().await;
    drop(f);
    sqlite_test_cleanup::remove_closed_directory(directory)
        .await
        .unwrap();
}

pub(super) fn other_player(f: &Fixture) -> TableTransportChannel {
    TableTransportChannel::Player {
        player_id: f.players[1],
        character_id: f.characters[1],
    }
}

pub(super) fn daggers(state: &CampaignState, actor: EntityId) -> Vec<ItemId> {
    let mut items = state
        .items
        .values()
        .filter(|item| item.custody == Custody::Entity(actor) && item.definition_id == "dagger")
        .map(|item| item.id)
        .collect::<Vec<_>>();
    items.sort_by_key(|item| item.0);
    assert_eq!(items.len(), 2);
    items
}

pub(super) fn hands(state: &CampaignState, actor: EntityId) -> &WeaponLoadout {
    &state
        .rules
        .as_ref()
        .unwrap()
        .tactical_inventory
        .as_ref()
        .unwrap()
        .loadouts
        .iter()
        .find(|loadout| loadout.actor == actor)
        .unwrap()
        .hands
}

pub(super) fn face_input(shown: TablePresentedView, face: u16) -> TableTransportInput {
    let roll = shown.roll.as_ref().unwrap();
    let count = if roll.mode == RollMode::Normal {
        roll.dice.iter().map(|die| usize::from(die.count)).sum()
    } else {
        2
    };
    physical_input(shown, &vec![face; count])
}

// Retain each storage type and value. SQL quote(TEXT) truncates at embedded NUL;
// hex(CAST(... AS BLOB)) does not. SQLite's 26-digit alternate-form REAL text
// preserves a round-trippable numeric value. Include every table and column.
pub(super) async fn typed_rows(pool: &sqlx::SqlitePool) -> BTreeMap<String, Vec<Vec<String>>> {
    use sqlx::Row;
    let names: Vec<String> =
        sqlx::query_scalar("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
            .fetch_all(pool)
            .await
            .unwrap();
    let mut result = BTreeMap::new();
    for name in names {
        let columns: Vec<String> =
            sqlx::query_scalar("SELECT name FROM pragma_table_info(?) ORDER BY cid")
                .bind(&name)
                .fetch_all(pool)
                .await
                .unwrap();
        let quote = |value: &str| format!("\"{}\"", value.replace('"', "\"\""));
        let statement = format!(
            "SELECT {} FROM {}",
            columns
                .iter()
                .map(|column| {
                    let quoted = quote(column);
                    format!("json_array(typeof({quoted}),CASE typeof({quoted}) WHEN 'real' THEN printf('%!.26g',{quoted}) ELSE hex(CAST({quoted} AS BLOB)) END)")
                })
                .collect::<Vec<_>>()
                .join(","),
            quote(&name)
        );
        let mut rows = sqlx::query(&statement)
            .fetch_all(pool)
            .await
            .unwrap()
            .into_iter()
            .map(|row| {
                (0..columns.len())
                    .map(|i| row.get::<String, _>(i))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        rows.sort();
        result.insert(name, rows);
    }
    result
}

struct Snapshot {
    state: CampaignState,
    export: CampaignExport,
    views: Vec<TablePresentedView>,
    rows: BTreeMap<String, Vec<Vec<String>>>,
}

async fn snapshot(f: &Fixture) -> Snapshot {
    // Presentation bootstrap is complete before any persistent baseline.
    let mut views = Vec::new();
    for channel in [TableTransportChannel::Host, player(f), other_player(f)] {
        views.push(view(f, &channel).await);
    }
    Snapshot {
        state: state(f).await,
        export: export_campaign(&f.pool, f.campaign).await.unwrap(),
        views,
        rows: typed_rows(&f.pool).await,
    }
}

async fn unchanged(f: &Fixture, before: Snapshot) {
    let mut after = snapshot(f).await;
    after.export.exported_at_utc = before.export.exported_at_utc.clone();
    assert_eq!(
        after.state, before.state,
        "refusal or retry changed campaign state"
    );
    assert_eq!(
        after.export, before.export,
        "refusal or retry changed the complete export"
    );
    assert_eq!(
        after.views, before.views,
        "refusal or retry changed a complete audience DTO"
    );
    assert_eq!(
        after.rows, before.rows,
        "refusal or retry changed any typed persistent row"
    );
}

pub(super) async fn atomic_rejection(f: &Fixture, request: TableTransportRequest) -> String {
    let before = snapshot(f).await;
    let error = Box::pin(f.runtime.submit_presented_table(request))
        .await
        .unwrap_err();
    let RunnableCampaignError::TableRejected(message) = error else {
        panic!("expected input rejection, got {error:?}")
    };
    Box::pin(unchanged(f, before)).await;
    message
}

pub(super) async fn atomic_retry(
    f: &Fixture,
    request: TableTransportRequest,
    expected: &TableTransportResult,
) {
    let before = snapshot(f).await;
    assert_eq!(
        &Box::pin(f.runtime.submit_presented_table(request))
            .await
            .unwrap(),
        expected
    );
    Box::pin(unchanged(f, before)).await;
}

pub(super) async fn equip_melee(f: &mut Fixture, path: &Path, target: EntityId, item: ItemId) {
    let mut choice = throw_choice(item, target);
    choice.delivery = WeaponDelivery::Melee;
    choice.equipment_change = Some(AttackEquipmentChange {
        timing: EquipmentChangeTiming::BeforeAttack,
        operation: AttackEquipmentOperation::Equip {
            item,
            hand: Hand::Right,
        },
    });
    Box::pin(player_step(
        f,
        path,
        action(TacticalAction::Attack { choice }),
    ))
    .await;
    Box::pin(player_raw(f, path, 1)).await;
    assert_eq!(
        hands(&state(f).await, f.actors[0]).hands[Hand::Right.index()],
        HandAssignment::Item(item)
    );
    Box::pin(next_player_turn(f, path)).await;
}

pub(super) async fn current_source(
    f: &mut Fixture,
    path: &Path,
    definition: &str,
    size: CreatureSize,
) -> EntityId {
    let shown = view(f, &TableTransportChannel::Host).await;
    let catalog = f
        .runtime
        .table_creature_options(TableCreatureOptionsRequest {
            campaign_id: f.campaign,
            channel: TableTransportChannel::Host,
            revision: shown.revision,
        })
        .await
        .unwrap();
    let source = catalog
        .iter()
        .find(|source| source.definition_id == definition)
        .unwrap();
    let actor = EntityId::new();
    Box::pin(step(
        f,
        path,
        TableTransportChannel::Host,
        TableTransportInput::Action(Box::new(TableAction::CreateCreature {
            creation: Box::new(TableCreatureCreation {
                entity_id: actor,
                name: format!("Private {definition}"),
                definition_id: definition.into(),
                source: source.source.clone(),
                size,
                additional_languages: vec![],
                ammunition_units: if source.ammunition_required { 20 } else { 0 },
                item_ids: (0..source.item_count).map(|_| ItemId::new()).collect(),
            }),
        })),
    ))
    .await;
    actor
}

pub(super) async fn control_source(
    f: &mut Fixture,
    path: &Path,
    actor: EntityId,
) -> TableTransportChannel {
    Box::pin(step(
        f,
        path,
        TableTransportChannel::Host,
        TableTransportInput::Action(Box::new(TableAction::EnableSourceActorAccess {
            adopted: vec![],
        })),
    ))
    .await;
    Box::pin(step(
        f,
        path,
        TableTransportChannel::Host,
        TableTransportInput::Action(Box::new(TableAction::SetSourceCreatureController {
            actor,
            controller: CreatureController::Player(f.players[0]),
        })),
    ))
    .await;
    TableTransportChannel::SourceCreature {
        player_id: f.players[0],
        actor,
    }
}

// Genuine selected Scimitar allowance. Keep every earlier printed_after body.
pub(super) async fn source_selected(
    f: &mut Fixture,
    path: &Path,
    hit: bool,
) -> (EntityId, ItemId, TableTransportChannel) {
    let source = Box::pin(setup(f, path)).await;
    let channel = Box::pin(control_source(f, path, source)).await;
    Box::pin(player_step(f, path, action(TacticalAction::EndTurn))).await;
    let item = state(f)
        .await
        .items
        .values()
        .find(|item| item.custody == Custody::Entity(source) && item.definition_id == "scimitar")
        .unwrap()
        .id;
    let mut choice = CreatureWeaponUseChoice {
        weapon: item,
        target: f.actors[0],
        grip: WeaponGrip::OneHand(Hand::Right),
        ammunition: None,
        equipment_change: Some(AttackEquipmentChange {
            timing: EquipmentChangeTiming::BeforeAttack,
            operation: AttackEquipmentOperation::Equip {
                item,
                hand: Hand::Right,
            },
        }),
        after_equipment: None,
    };
    Box::pin(step(
        f,
        path,
        channel.clone(),
        action(TacticalAction::CreatureWeaponAttack {
            feature_id: "scimitar".into(),
            choice: choice.clone(),
        }),
    ))
    .await;
    Box::pin(raw(f, path, channel.clone(), 1)).await;
    Box::pin(step(
        f,
        path,
        channel.clone(),
        action(TacticalAction::EndTurn),
    ))
    .await;
    Box::pin(player_step(f, path, action(TacticalAction::EndTurn))).await;
    choice.equipment_change = None;
    choice.after_equipment = Some(AfterAttackEquipmentIntent::Choose);
    Box::pin(step(
        f,
        path,
        channel.clone(),
        action(TacticalAction::CreatureWeaponAttack {
            feature_id: "scimitar".into(),
            choice,
        }),
    ))
    .await;
    Box::pin(raw(f, path, channel.clone(), if hit { 20 } else { 1 })).await;
    (source, item, channel)
}

// Negative copies only: the destination itself is an unrelated genuine campaign.
pub(super) async fn hostile_private_records(
    f: &Fixture,
    handle: CommandId,
    accepted: Option<CommandId>,
) {
    let export = export_campaign(&f.pool, f.campaign).await.unwrap();
    let pool = open_sqlite("sqlite::memory:").await.unwrap();
    let destination = Box::pin(Fixture::with_pool(TableContract::default(), pool.clone())).await;
    assert_ne!(destination.campaign, f.campaign);
    let baseline = snapshot(&destination).await;
    for mutation in 0..(if accepted.is_some() { 6 } else { 4 }) {
        let mut forged = export.clone();
        if mutation < 4 {
            let change = forged
                .table_projection_history
                .iter_mut()
                .rev()
                .flat_map(|row| row.changes.iter_mut())
                .find(|change| {
                    change
                        .handles
                        .iter()
                        .any(|candidate| candidate.opaque == handle.0)
                })
                .unwrap();
            let capability = &mut change
                .handles
                .iter_mut()
                .find(|candidate| candidate.opaque == handle.0)
                .unwrap()
                .capability;
            let dmd_persistence::ProjectionCapability::AttackEquipment { origin, occurrence } =
                capability
            else {
                panic!("real equipment capability required")
            };
            match mutation {
                0 => *occurrence = occurrence.wrapping_add(1),
                1 => *origin = CommandId::new(),
                2 => change.audience = dmd_persistence::ProjectionAudience::Player(f.players[1]),
                3 => change.visible_digest = "0".repeat(64),
                _ => unreachable!(),
            }
        } else {
            let binding = forged
                .table_transport_bindings
                .iter_mut()
                .find(|binding| binding.meta.id == accepted.unwrap())
                .unwrap();
            if mutation == 4 {
                binding.response_json = "null".into();
            } else {
                let mut changed: TableTransportRequest =
                    serde_json::from_str(&binding.request_json).unwrap();
                changed.input = TableTransportInput::AttackEquipment {
                    handle,
                    choice: AttackEquipmentChoice::Apply(AttackEquipmentOperation::Pickup {
                        item: ItemId::new(),
                        hand: Hand::Left,
                    }),
                };
                binding.request_json = serde_json::to_string(&changed).unwrap();
            }
        }
        assert!(
            Box::pin(destination.runtime.restore_campaign(&forged))
                .await
                .is_err()
        );
        assert_eq!(typed_rows(&pool).await, baseline.rows);
        assert_eq!(state(&destination).await, baseline.state);
    }
    Box::pin(unchanged(&destination, baseline)).await;
    pool.close().await;
}

pub(super) async fn hostile_activation_rows(f: &Fixture) {
    let original = export_campaign(&f.pool, f.campaign).await.unwrap();
    let pool = open_sqlite("sqlite::memory:").await.unwrap();
    let destination = Box::pin(Fixture::with_pool(TableContract::default(), pool.clone())).await;
    assert_ne!(destination.campaign, f.campaign);
    let baseline = snapshot(&destination).await;
    let mut forged = original;
    let mut changed = state(f).await;
    changed
        .encounter
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .attack_equipment_access
        .as_mut()
        .unwrap()
        .origin
        .id = CommandId::new();
    forged.current_state.state_json = changed.encode_json().unwrap();
    let sequence = i64::try_from(changed.applied_event_sequence).unwrap();
    forged
        .snapshots
        .retain(|row| row.event_sequence != sequence);
    forged.snapshots.push(dmd_persistence::SnapshotRow {
        campaign_id: forged.campaign_id.clone(),
        event_sequence: sequence,
        state_schema_version: i64::from(changed.schema_version),
        state_json: changed.encode_json().unwrap(),
        created_at_utc: forged.exported_at_utc.clone(),
    });
    forged.snapshots.sort_by_key(|row| row.event_sequence);
    assert!(
        Box::pin(destination.runtime.restore_campaign(&forged))
            .await
            .is_err()
    );
    Box::pin(unchanged(&destination, baseline)).await;
    pool.close().await;
}
