//! Accepted application commands, physical dice and actual file-SQLite cuts.
//! No positive scenario imports a fabricated paid or activated campaign image.
use super::*;
use dmd_persistence::CampaignExport;
use dmd_rules::tactical::TacticalAction;

#[path = "table_ogre_equipment_cases.rs"]
mod ogre;

fn runtime(pool: sqlx::SqlitePool) -> CampaignRuntime {
    CampaignRuntime::from_content_root(
        pool,
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../content"),
    )
}
fn player(f: &Fixture) -> TableTransportChannel {
    TableTransportChannel::Player {
        player_id: f.players[0],
        character_id: f.characters[0],
    }
}
fn action(action: TacticalAction) -> TableTransportInput {
    TableTransportInput::Action(Box::new(TableAction::Tactical { action }))
}
async fn state(f: &Fixture) -> CampaignState {
    f.runtime
        .open_campaign(f.campaign)
        .await
        .unwrap()
        .state()
        .clone()
}
async fn view(f: &Fixture, channel: &TableTransportChannel) -> TablePresentedView {
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
async fn request(
    f: &Fixture,
    channel: TableTransportChannel,
    input: TableTransportInput,
) -> TableTransportRequest {
    let projected = view(f, &channel).await;
    TableTransportRequest {
        version: if projected.source_control.is_some()
            || matches!(&input, TableTransportInput::Action(action) if matches!(action.as_ref(), TableAction::EnableSourceActorAccess { .. }))
        {
            2
        } else {
            1
        },
        command_id: CommandId::new(),
        campaign_id: f.campaign,
        session_id: Some(f.session),
        channel,
        revision: projected.revision,
        input,
    }
}
async fn reopen(f: &mut Fixture, path: &Path) {
    f.pool.close().await;
    f.pool = dmd_persistence::open_sqlite_path(path).await.unwrap();
    f.runtime = runtime(f.pool.clone());
    Box::pin(f.runtime.resume_campaign(f.campaign))
        .await
        .unwrap();
}
async fn rejected(f: &Fixture, request: TableTransportRequest) {
    let before = export_campaign(&f.pool, f.campaign).await.unwrap();
    assert!(
        Box::pin(f.runtime.submit_presented_table(request))
            .await
            .is_err()
    );
    let mut after = export_campaign(&f.pool, f.campaign).await.unwrap();
    after.exported_at_utc = before.exported_at_utc.clone();
    assert_eq!(after, before, "refusal changed a persistent row");
}
async fn cold_step(
    f: &mut Fixture,
    path: &Path,
    request: TableTransportRequest,
) -> TableTransportResult {
    let before = export_campaign(&f.pool, f.campaign).await.unwrap();
    let pool = open_sqlite("sqlite::memory:").await.unwrap();
    let mirror = runtime(pool.clone());
    Box::pin(mirror.restore_campaign(&before)).await.unwrap();
    Box::pin(reopen(f, path)).await;
    let accepted = Box::pin(f.runtime.submit_presented_table(request.clone()))
        .await
        .unwrap();
    let independent = Box::pin(mirror.submit_presented_table(request.clone()))
        .await
        .unwrap();
    let (TableTransportResult::Accepted(a), TableTransportResult::Accepted(b)) =
        (&accepted, &independent)
    else {
        panic!("accepted gameplay required")
    };
    assert_eq!(a.outcome, b.outcome);
    assert_eq!(
        mirror.open_campaign(f.campaign).await.unwrap().state(),
        &state(f).await
    );
    assert_eq!(
        Box::pin(mirror.submit_presented_table(request.clone()))
            .await
            .unwrap(),
        independent
    );
    pool.close().await;
    Box::pin(reopen(f, path)).await;
    let saved = export_campaign(&f.pool, f.campaign).await.unwrap();
    assert_eq!(
        Box::pin(f.runtime.submit_presented_table(request.clone()))
            .await
            .unwrap(),
        accepted
    );
    let mut after = export_campaign(&f.pool, f.campaign).await.unwrap();
    after.exported_at_utc = saved.exported_at_utc.clone();
    assert_eq!(after, saved);
    let mut changed = request;
    changed.input = action(TacticalAction::Dodge);
    Box::pin(rejected(f, changed)).await;
    accepted
}
async fn step(
    f: &mut Fixture,
    path: &Path,
    channel: TableTransportChannel,
    input: TableTransportInput,
) -> TableTransportRequest {
    let request = request(f, channel, input).await;
    Box::pin(cold_step(f, path, request.clone())).await;
    request
}
async fn raw(f: &mut Fixture, path: &Path, channel: TableTransportChannel, face: u16) {
    let roll = view(f, &channel).await.roll.unwrap();
    let sides = if roll.mode == RollMode::Normal {
        roll.dice
            .iter()
            .flat_map(|die| std::iter::repeat_n(die.sides, usize::from(die.count)))
            .collect::<Vec<_>>()
    } else {
        vec![20, 20]
    };
    Box::pin(step(
        f,
        path,
        channel,
        action(TacticalAction::SubmitRoll {
            result: RollResult {
                request_id: roll.id,
                source: RollSource::Physical,
                dice: sides
                    .into_iter()
                    .map(|sides| DieResult {
                        sides,
                        value: face.min(sides),
                    })
                    .collect(),
            },
        }),
    ))
    .await;
}
async fn player_step(
    f: &mut Fixture,
    path: &Path,
    input: TableTransportInput,
) -> TableTransportRequest {
    let channel = player(f);
    Box::pin(step(f, path, channel, input)).await
}
async fn player_raw(f: &mut Fixture, path: &Path, face: u16) {
    let channel = player(f);
    Box::pin(raw(f, path, channel, face)).await;
}
async fn next_player_turn(f: &mut Fixture, path: &Path) {
    Box::pin(player_step(f, path, action(TacticalAction::EndTurn))).await;
    Box::pin(step(
        f,
        path,
        TableTransportChannel::Host,
        action(TacticalAction::EndTurn),
    ))
    .await;
    assert_eq!(
        state(f).await.rules.unwrap().timing.unwrap().order[0].actor,
        f.actors[0]
    );
}
fn throw_choice(item: ItemId, target: EntityId) -> WeaponUseChoice {
    WeaponUseChoice {
        weapon: item,
        target,
        delivery: WeaponDelivery::Thrown,
        ability: Ability::Strength,
        grip: WeaponGrip::OneHand(Hand::Right),
        purpose: WeaponAttackPurpose::Normal,
        ammunition: None,
        equipment_change: None,
        after_equipment: None,
    }
}
async fn setup(f: &mut Fixture, path: &Path) -> EntityId {
    Box::pin(setup_at(f, path, SpatialPoint { x: 20, y: 10, z: 0 })).await
}
async fn setup_at(f: &mut Fixture, path: &Path, point: SpatialPoint) -> EntityId {
    let target = Box::pin(table_attack_cases::prepare_at(f, point)).await;
    let old = view(f, &player(f)).await;
    assert!(
        old.tactical
            .as_ref()
            .unwrap()
            .attack_options
            .as_ref()
            .unwrap()
            .equipment
            .is_none()
    );
    assert!(old.tactical.as_ref().unwrap().attack_equipment.is_none());
    assert!(
        !serde_json::to_string(&old)
            .unwrap()
            .contains("equipment_enabled")
    );
    let forbidden = request(
        f,
        player(f),
        action(TacticalAction::ActivateAttackEquipment),
    )
    .await;
    Box::pin(rejected(f, forbidden)).await;
    let activation = Box::pin(step(
        f,
        path,
        TableTransportChannel::Host,
        action(TacticalAction::ActivateAttackEquipment),
    ))
    .await;
    let activated = state(f).await;
    assert_eq!(
        activated
            .encounter
            .unwrap()
            .flow
            .unwrap()
            .attack_equipment_access
            .unwrap()
            .origin
            .id,
        activation.command_id
    );
    target
}
async fn forged_refused(export: &CampaignExport) {
    let pool = open_sqlite("sqlite::memory:").await.unwrap();
    let before = all_destination_rows(&pool).await;
    assert!(
        Box::pin(runtime(pool.clone()).restore_campaign(export))
            .await
            .is_err()
    );
    assert_eq!(
        all_destination_rows(&pool).await,
        before,
        "preflight changed a destination row"
    );
    pool.close().await;
}

async fn all_destination_rows(
    pool: &sqlx::SqlitePool,
) -> std::collections::BTreeMap<String, Vec<Vec<String>>> {
    use sqlx::Row;
    let names: Vec<String> =
        sqlx::query_scalar("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
            .fetch_all(pool)
            .await
            .unwrap();
    let mut result = std::collections::BTreeMap::new();
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
                .map(|c| format!("quote({})", quote(c)))
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
async fn hostile_images(f: &Fixture) {
    let original = export_campaign(&f.pool, f.campaign).await.unwrap();
    for mutation in 0..3 {
        let mut changed = state(f).await;
        let flow = changed.encounter.as_mut().unwrap().flow.as_mut().unwrap();
        match mutation {
            0 => {
                flow.attack_equipment_access
                    .as_mut()
                    .unwrap()
                    .origin
                    .campaign_id = CampaignId::new()
            }
            1 => flow.attack_equipment_access.as_mut().unwrap().origin.id = CommandId::new(),
            2 => flow.attack_equipment_access = None,
            _ => unreachable!(),
        }
        let mut forged = original.clone();
        forged.current_state.state_json = changed.encode_json().unwrap();
        forged.snapshots.retain(|row| {
            row.event_sequence != i64::try_from(changed.applied_event_sequence).unwrap()
        });
        forged.snapshots.push(dmd_persistence::SnapshotRow {
            campaign_id: forged.campaign_id.clone(),
            event_sequence: i64::try_from(changed.applied_event_sequence).unwrap(),
            state_schema_version: i64::from(changed.schema_version),
            state_json: changed.encode_json().unwrap(),
            created_at_utc: forged.exported_at_utc.clone(),
        });
        forged.snapshots.sort_by_key(|row| row.event_sequence);
        Box::pin(forged_refused(&forged)).await;
    }
    let mut forged = original.clone();
    let activation = state(f)
        .await
        .encounter
        .unwrap()
        .flow
        .unwrap()
        .attack_equipment_access
        .unwrap()
        .origin
        .id;
    let audit = forged
        .command_audit
        .iter_mut()
        .find(|row| row.id == activation.0.to_string())
        .unwrap();
    audit.accepted = 0;
    Box::pin(forged_refused(&forged)).await;
}

async fn hostile_capabilities(f: &Fixture, handle: CommandId) {
    let original = export_campaign(&f.pool, f.campaign).await.unwrap();
    for mutation in 0..4 {
        let mut forged = original.clone();
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
            panic!("equipment capability required")
        };
        match mutation {
            0 => *occurrence = occurrence.wrapping_add(1),
            1 => *origin = CommandId::new(),
            2 => change.audience = dmd_persistence::ProjectionAudience::Player(f.players[1]),
            3 => change.visible_digest = "0".repeat(64),
            _ => unreachable!(),
        }
        Box::pin(forged_refused(&forged)).await;
    }
}

async fn hostile_envelope(f: &Fixture, accepted: CommandId) {
    let original = export_campaign(&f.pool, f.campaign).await.unwrap();
    for response in [false, true] {
        let mut forged = original.clone();
        let binding = forged
            .table_transport_bindings
            .iter_mut()
            .find(|binding| binding.meta.id == accepted)
            .unwrap();
        if response {
            binding.response_json = "null".into();
        } else {
            let mut request: TableTransportRequest =
                serde_json::from_str(&binding.request_json).unwrap();
            let TableTransportInput::AttackEquipment { handle, .. } = request.input else {
                panic!("equipment envelope required")
            };
            request.input = TableTransportInput::AttackEquipment {
                handle,
                choice: AttackEquipmentChoice::Apply(AttackEquipmentOperation::Pickup {
                    item: ItemId::new(),
                    hand: Hand::Left,
                }),
            };
            binding.request_json = serde_json::to_string(&request).unwrap();
        }
        Box::pin(forged_refused(&forged)).await;
    }
}

async fn selected_throw(f: &mut Fixture, path: &Path, apply: bool) {
    let target = Box::pin(setup(f, path)).await;
    let initial = state(f).await;
    let item = initial
        .items
        .values()
        .find(|i| i.custody == Custody::Entity(f.actors[0]) && i.definition_id == "dagger")
        .unwrap()
        .clone();
    let mut choice = throw_choice(item.id, target);
    choice.after_equipment = Some(AfterAttackEquipmentIntent::Choose);
    let attack = Box::pin(player_step(
        f,
        path,
        action(TacticalAction::Attack { choice }),
    ))
    .await;
    Box::pin(hostile_images(f)).await;
    Box::pin(player_raw(f, path, 1)).await;
    let selected = state(f).await;
    assert!(
        selected
            .rules
            .as_ref()
            .unwrap()
            .timing
            .as_ref()
            .unwrap()
            .action_spent
    );
    let flow = selected.encounter.as_ref().unwrap().flow.as_ref().unwrap();
    let resolution = flow.resolution.as_ref().unwrap();
    assert!(resolution.attack.is_none());
    assert_eq!(
        resolution
            .attack_after_equipment
            .as_ref()
            .unwrap()
            .cause
            .origin
            .id,
        attack.command_id
    );
    let shown = view(f, &player(f))
        .await
        .tactical
        .unwrap()
        .attack_equipment
        .unwrap();
    Box::pin(hostile_capabilities(f, shown.key)).await;
    assert_eq!(shown.actor, f.actors[0]);
    let operation = AttackEquipmentOperation::Pickup {
        item: item.id,
        hand: Hand::Right,
    };
    assert!(
        shown
            .operations
            .iter()
            .any(|offer| offer.operation == operation)
    );
    let unrelated = view(
        f,
        &TableTransportChannel::Player {
            player_id: f.players[1],
            character_id: f.characters[1],
        },
    )
    .await;
    assert!(
        unrelated
            .tactical
            .as_ref()
            .unwrap()
            .attack_equipment
            .is_none()
    );
    let stale = request(
        f,
        player(f),
        TableTransportInput::AttackEquipment {
            handle: shown.key,
            choice: AttackEquipmentChoice::Decline,
        },
    )
    .await;
    let raw_work = request(
        f,
        player(f),
        action(TacticalAction::ChooseAttackEquipment {
            work: TacticalWorkKey {
                resolution: resolution.origin.id,
                occurrence: resolution
                    .attack_after_equipment
                    .as_ref()
                    .unwrap()
                    .work
                    .occurrence,
            },
            choice: AttackEquipmentChoice::Decline,
        }),
    )
    .await;
    Box::pin(rejected(f, raw_work)).await;
    let wrong_kind = request(
        f,
        player(f),
        TableTransportInput::SelectWork { handle: shown.key },
    )
    .await;
    Box::pin(rejected(f, wrong_kind)).await;
    for bad in [
        ItemId::new(),
        initial
            .items
            .values()
            .find(|i| i.custody == Custody::Entity(target))
            .unwrap()
            .id,
    ] {
        let request = request(
            f,
            player(f),
            TableTransportInput::AttackEquipment {
                handle: shown.key,
                choice: AttackEquipmentChoice::Apply(AttackEquipmentOperation::Pickup {
                    item: bad,
                    hand: Hand::Right,
                }),
            },
        )
        .await;
        Box::pin(rejected(f, request)).await;
    }
    assert_eq!(state(f).await, selected);
    let final_request = Box::pin(player_step(
        f,
        path,
        TableTransportInput::AttackEquipment {
            handle: shown.key,
            choice: if apply {
                AttackEquipmentChoice::Apply(operation)
            } else {
                AttackEquipmentChoice::Decline
            },
        },
    ))
    .await;
    let finished = state(f).await;
    Box::pin(hostile_envelope(f, final_request.command_id)).await;
    let restored_item = &finished.items[&item.id];
    assert_eq!(restored_item.owner, item.owner);
    assert_eq!(restored_item.quantity, 1);
    assert_eq!(restored_item.custody == Custody::Entity(f.actors[0]), apply);
    assert_eq!(
        finished.rules.as_ref().unwrap().rolls,
        selected.rules.as_ref().unwrap().rolls
    );
    assert_eq!(
        finished.rules.as_ref().unwrap().timing,
        selected.rules.as_ref().unwrap().timing
    );
    assert!(
        finished
            .encounter
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .resolution
            .is_none()
    );
    Box::pin(rejected(f, stale)).await;
    Box::pin(next_player_turn(f, path)).await;
    let accepted = Box::pin(f.runtime.submit_presented_table(final_request.clone()))
        .await
        .unwrap();
    Box::pin(reopen(f, path)).await;
    assert_eq!(
        Box::pin(f.runtime.submit_presented_table(final_request))
            .await
            .unwrap(),
        accepted
    );
}

async fn changed_session_retry(f: &mut Fixture, path: &Path, original: TableTransportRequest) {
    let accepted = Box::pin(f.runtime.submit_presented_table(original.clone()))
        .await
        .unwrap();
    Box::pin(step(
        f,
        path,
        TableTransportChannel::Host,
        action(TacticalAction::ConcludeHostilities {
            cadence: AftermathCadence::ContinueExistingOrder,
            ruling: "Hostilities have ceased after the equipment decision settled.".into(),
        }),
    ))
    .await;
    Box::pin(step(
        f,
        path,
        TableTransportChannel::Host,
        TableTransportInput::Action(Box::new(TableAction::EndSession)),
    ))
    .await;
    f.session = PlaySessionId::new();
    let start = TableAction::StartSession {
        id: f.session,
        name: "Next accepted session".into(),
        participants: (0..2)
            .map(|i| SessionParticipant {
                player_id: f.players[i],
                character_id: Some(f.characters[i]),
                attendance: if i == 0 {
                    AttendanceStatus::Present
                } else {
                    AttendanceStatus::Absent
                },
            })
            .collect(),
    };
    Box::pin(step(
        f,
        path,
        TableTransportChannel::Host,
        TableTransportInput::Action(Box::new(start)),
    ))
    .await;
    let before = export_campaign(&f.pool, f.campaign).await.unwrap();
    assert_ne!(original.session_id, Some(f.session));
    Box::pin(reopen(f, path)).await;
    assert_eq!(
        Box::pin(f.runtime.submit_presented_table(original))
            .await
            .unwrap(),
        accepted
    );
    let mut after = export_campaign(&f.pool, f.campaign).await.unwrap();
    after.exported_at_utc = before.exported_at_utc.clone();
    assert_eq!(after, before);
}

async fn no_apply(f: &mut Fixture, path: &Path) {
    let target = Box::pin(setup_at(f, path, SpatialPoint { x: 80, y: 10, z: 0 })).await;
    let item = state(f)
        .await
        .items
        .values()
        .find(|i| i.custody == Custody::Entity(f.actors[0]) && i.definition_id == "dagger")
        .unwrap()
        .id;
    let mut choice = throw_choice(item, target);
    choice.after_equipment = Some(AfterAttackEquipmentIntent::Choose);
    Box::pin(player_step(
        f,
        path,
        action(TacticalAction::Attack { choice }),
    ))
    .await;
    Box::pin(player_raw(f, path, 1)).await;
    let shown = view(f, &player(f))
        .await
        .tactical
        .unwrap()
        .attack_equipment
        .unwrap();
    assert!(
        shown.operations.is_empty(),
        "the thrown dagger is outside physical pickup reach"
    );
    assert!(shown.may_decline);
    for item in [item, ItemId::new()] {
        let forbidden = request(
            f,
            player(f),
            TableTransportInput::AttackEquipment {
                handle: shown.key,
                choice: AttackEquipmentChoice::Apply(AttackEquipmentOperation::Pickup {
                    item,
                    hand: Hand::Right,
                }),
            },
        )
        .await;
        Box::pin(rejected(f, forbidden)).await;
    }
    let declined = Box::pin(player_step(
        f,
        path,
        TableTransportInput::AttackEquipment {
            handle: shown.key,
            choice: AttackEquipmentChoice::Decline,
        },
    ))
    .await;
    Box::pin(changed_session_retry(f, path, declined)).await;
}

async fn before_pickup(f: &mut Fixture, path: &Path, different: bool) {
    let target = Box::pin(setup(f, path)).await;
    let initial = state(f).await;
    let mut daggers = initial
        .items
        .values()
        .filter(|i| i.custody == Custody::Entity(f.actors[0]) && i.definition_id == "dagger")
        .map(|i| i.id)
        .collect::<Vec<_>>();
    daggers.sort_by_key(|id| id.0);
    assert_eq!(daggers.len(), 2);
    Box::pin(player_step(
        f,
        path,
        action(TacticalAction::Attack {
            choice: throw_choice(daggers[0], target),
        }),
    ))
    .await;
    Box::pin(player_raw(f, path, 1)).await;
    Box::pin(next_player_turn(f, path)).await;
    let before = state(f).await;
    let options = view(f, &player(f))
        .await
        .tactical
        .unwrap()
        .attack_options
        .unwrap();
    assert!(
        options
            .equipment
            .unwrap()
            .pickups
            .iter()
            .any(|p| p.item == daggers[0])
    );
    let weapon = daggers[usize::from(different)];
    let hand = if different { Hand::Left } else { Hand::Right };
    let mut choice = throw_choice(weapon, target);
    choice.equipment_change = Some(AttackEquipmentChange {
        timing: EquipmentChangeTiming::BeforeAttack,
        operation: AttackEquipmentOperation::Pickup {
            item: daggers[0],
            hand,
        },
    });
    let mut invalid = choice.clone();
    invalid.target = EntityId::new();
    let forbidden = request(
        f,
        player(f),
        action(TacticalAction::Attack { choice: invalid }),
    )
    .await;
    Box::pin(rejected(f, forbidden)).await;
    assert_eq!(state(f).await, before);
    Box::pin(player_step(
        f,
        path,
        action(TacticalAction::Attack { choice }),
    ))
    .await;
    let paid = state(f).await;
    assert_eq!(
        paid.items[&daggers[0]].custody,
        Custody::Entity(f.actors[0])
    );
    assert_eq!(
        paid.items[&daggers[0]].owner,
        before.items[&daggers[0]].owner
    );
    assert_eq!(paid.items.len(), before.items.len());
    assert!(
        paid.rules
            .as_ref()
            .unwrap()
            .timing
            .as_ref()
            .unwrap()
            .action_spent
    );
    Box::pin(player_raw(f, path, 1)).await;
    let complete = state(f).await;
    assert_eq!(
        complete.items[&daggers[0]].custody == Custody::Entity(f.actors[0]),
        different
    );
    assert!(
        complete
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

async fn next_source_turn(f: &mut Fixture, path: &Path) {
    Box::pin(step(
        f,
        path,
        TableTransportChannel::Host,
        action(TacticalAction::EndTurn),
    ))
    .await;
    Box::pin(player_step(f, path, action(TacticalAction::EndTurn))).await;
}

async fn printed_after(
    f: &mut Fixture,
    path: &Path,
    bow: bool,
    hit: bool,
    player_controlled: bool,
) {
    let source = Box::pin(setup(f, path)).await;
    if player_controlled {
        Box::pin(step(
            f,
            path,
            TableTransportChannel::Host,
            TableTransportInput::Action(Box::new(TableAction::EnableSourceActorAccess {
                adopted: vec![],
            })),
        ))
        .await;
        let controller = CreatureController::Player(f.players[0]);
        Box::pin(step(
            f,
            path,
            TableTransportChannel::Host,
            TableTransportInput::Action(Box::new(TableAction::SetSourceCreatureController {
                actor: source,
                controller,
            })),
        ))
        .await;
    }
    let source_channel = if player_controlled {
        TableTransportChannel::SourceCreature {
            player_id: f.players[0],
            actor: source,
        }
    } else {
        TableTransportChannel::Host
    };
    Box::pin(player_step(f, path, action(TacticalAction::EndTurn))).await;
    if bow {
        Box::pin(step(
            f,
            path,
            source_channel.clone(),
            action(TacticalAction::DoffShield),
        ))
        .await;
        Box::pin(step(
            f,
            path,
            source_channel.clone(),
            action(TacticalAction::EndTurn),
        ))
        .await;
        Box::pin(player_step(f, path, action(TacticalAction::EndTurn))).await;
    }
    let initial = state(f).await;
    let definition = if bow { "shortbow" } else { "scimitar" };
    let item = initial
        .items
        .values()
        .find(|i| i.custody == Custody::Entity(source) && i.definition_id == definition)
        .unwrap()
        .id;
    let arrows = initial
        .items
        .values()
        .find(|i| i.custody == Custody::Entity(source) && i.definition_id == "arrows")
        .unwrap()
        .clone();
    assert_eq!(arrows.quantity, 20);
    let pin = initial
        .rules
        .as_ref()
        .unwrap()
        .tactical_creatures
        .as_ref()
        .unwrap()
        .profile(source)
        .unwrap()
        .clone();
    let mut choice = CreatureWeaponUseChoice {
        weapon: item,
        target: f.actors[0],
        grip: if bow {
            WeaponGrip::TwoHands
        } else {
            WeaponGrip::OneHand(Hand::Right)
        },
        ammunition: bow.then_some(arrows.id),
        equipment_change: Some(AttackEquipmentChange {
            timing: EquipmentChangeTiming::BeforeAttack,
            operation: AttackEquipmentOperation::Equip {
                item,
                hand: Hand::Right,
            },
        }),
        after_equipment: None,
    };
    // The genuine first source attack readies its own printed gear. No initial
    // loadout edit supplies the second attack's held weapon or allowance.
    Box::pin(step(
        f,
        path,
        source_channel.clone(),
        action(TacticalAction::CreatureWeaponAttack {
            feature_id: definition.into(),
            choice: choice.clone(),
        }),
    ))
    .await;
    Box::pin(raw(f, path, source_channel.clone(), 1)).await;
    Box::pin(step(
        f,
        path,
        source_channel.clone(),
        action(TacticalAction::EndTurn),
    ))
    .await;
    Box::pin(player_step(f, path, action(TacticalAction::EndTurn))).await;
    choice.equipment_change = None;
    choice.after_equipment = Some(AfterAttackEquipmentIntent::Choose);
    let before = state(f).await;
    let attack = Box::pin(step(
        f,
        path,
        source_channel.clone(),
        action(TacticalAction::CreatureWeaponAttack {
            feature_id: definition.into(),
            choice,
        }),
    ))
    .await;
    let pending = state(f).await;
    let physical = pending
        .encounter
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .resolution
        .as_ref()
        .unwrap()
        .attack
        .as_ref()
        .unwrap();
    assert!(
        matches!(&physical.source, TacticalAttackSource::CreatureWeapon { feature_id, .. } if feature_id == definition)
    );
    assert_eq!(
        pending.items[&arrows.id].quantity,
        before.items[&arrows.id].quantity - u32::from(bow)
    );
    Box::pin(raw(
        f,
        path,
        source_channel.clone(),
        if hit { 20 } else { 1 },
    ))
    .await;
    if hit {
        Box::pin(table_hit_driver::decline_hit_responses(f)).await;
        // cold_step below imports and reopens the actual DamageRoll cut.
        Box::pin(raw(f, path, source_channel.clone(), 1)).await;
    }
    let completed = state(f).await;
    let offered = view(f, &source_channel)
        .await
        .tactical
        .unwrap()
        .attack_equipment
        .unwrap();
    assert_eq!(offered.actor, source);
    assert!(
        completed
            .encounter
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .resolution
            .as_ref()
            .unwrap()
            .attack
            .is_none()
    );
    let unrelated = view(
        f,
        &TableTransportChannel::Player {
            player_id: f.players[1],
            character_id: f.characters[1],
        },
    )
    .await;
    assert!(
        unrelated
            .tactical
            .as_ref()
            .unwrap()
            .attack_equipment
            .is_none()
    );
    let operation = AttackEquipmentOperation::Unequip { item };
    assert!(
        offered
            .operations
            .iter()
            .any(|entry| entry.operation == operation)
    );
    let final_request = Box::pin(step(
        f,
        path,
        source_channel.clone(),
        TableTransportInput::AttackEquipment {
            handle: offered.key,
            choice: if hit {
                AttackEquipmentChoice::Apply(operation)
            } else {
                AttackEquipmentChoice::Decline
            },
        },
    ))
    .await;
    let final_state = state(f).await;
    assert_eq!(
        final_state
            .rules
            .as_ref()
            .unwrap()
            .tactical_creatures
            .as_ref()
            .unwrap()
            .profile(source)
            .unwrap(),
        &pin
    );
    assert_eq!(final_state.items[&arrows.id], completed.items[&arrows.id]);
    assert_eq!(final_state.items[&item], initial.items[&item]);
    assert_eq!(
        final_state.rules.as_ref().unwrap().rolls,
        completed.rules.as_ref().unwrap().rolls
    );
    let receipt = final_state
        .encounter
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .budget
        .weapon_history
        .iter()
        .find(|r| r.origin.id == attack.command_id)
        .unwrap();
    assert!(
        matches!(&receipt.after_equipment.as_ref().unwrap().cause.source, AttackEquipmentSource::Creature { feature_id, .. } if feature_id == definition)
    );
    let original_response = Box::pin(f.runtime.submit_presented_table(final_request.clone()))
        .await
        .unwrap();
    if !player_controlled {
        Box::pin(step(
            f,
            path,
            TableTransportChannel::Host,
            TableTransportInput::Action(Box::new(TableAction::EnableSourceActorAccess {
                adopted: vec![],
            })),
        ))
        .await;
    }
    let controller = if player_controlled {
        CreatureController::Host
    } else {
        CreatureController::Player(f.players[0])
    };
    Box::pin(step(
        f,
        path,
        TableTransportChannel::Host,
        TableTransportInput::Action(Box::new(TableAction::SetSourceCreatureController {
            actor: source,
            controller,
        })),
    ))
    .await;
    let before_retry = export_campaign(&f.pool, f.campaign).await.unwrap();
    Box::pin(reopen(f, path)).await;
    assert_eq!(
        Box::pin(f.runtime.submit_presented_table(final_request))
            .await
            .unwrap(),
        original_response
    );
    let mut after_retry = export_campaign(&f.pool, f.campaign).await.unwrap();
    after_retry.exported_at_utc = before_retry.exported_at_utc.clone();
    assert_eq!(after_retry, before_retry);
    if player_controlled {
        let forbidden = request(f, source_channel, action(TacticalAction::EndTurn)).await;
        Box::pin(rejected(f, forbidden)).await;
    }
}

async fn printed_before_after_real_knockout(f: &mut Fixture, path: &Path, bow: bool) {
    let source = Box::pin(setup(f, path)).await;
    let dagger = state(f)
        .await
        .items
        .values()
        .find(|i| i.custody == Custody::Entity(f.actors[0]) && i.definition_id == "dagger")
        .unwrap()
        .id;
    let mut ready_dagger = throw_choice(dagger, source);
    ready_dagger.delivery = WeaponDelivery::Melee;
    ready_dagger.equipment_change = Some(AttackEquipmentChange {
        timing: EquipmentChangeTiming::BeforeAttack,
        operation: AttackEquipmentOperation::Equip {
            item: dagger,
            hand: Hand::Right,
        },
    });
    Box::pin(player_step(
        f,
        path,
        action(TacticalAction::Attack {
            choice: ready_dagger,
        }),
    ))
    .await;
    Box::pin(player_raw(f, path, 1)).await;
    Box::pin(player_step(f, path, action(TacticalAction::EndTurn))).await;
    if bow {
        Box::pin(step(
            f,
            path,
            TableTransportChannel::Host,
            action(TacticalAction::DoffShield),
        ))
        .await;
        Box::pin(next_source_turn(f, path)).await;
    }
    let initial = state(f).await;
    let definition = if bow { "shortbow" } else { "scimitar" };
    let item = initial
        .items
        .values()
        .find(|i| i.custody == Custody::Entity(source) && i.definition_id == definition)
        .unwrap()
        .clone();
    let arrows = initial
        .items
        .values()
        .find(|i| i.custody == Custody::Entity(source) && i.definition_id == "arrows")
        .unwrap()
        .id;
    let source_pin = initial
        .rules
        .as_ref()
        .unwrap()
        .tactical_creatures
        .as_ref()
        .unwrap()
        .profile(source)
        .unwrap()
        .clone();
    let mut choice = CreatureWeaponUseChoice {
        weapon: item.id,
        target: f.actors[0],
        grip: if bow {
            WeaponGrip::TwoHands
        } else {
            WeaponGrip::OneHand(Hand::Right)
        },
        ammunition: bow.then_some(arrows),
        equipment_change: Some(AttackEquipmentChange {
            timing: EquipmentChangeTiming::BeforeAttack,
            operation: AttackEquipmentOperation::Equip {
                item: item.id,
                hand: Hand::Right,
            },
        }),
        after_equipment: None,
    };
    Box::pin(step(
        f,
        path,
        TableTransportChannel::Host,
        action(TacticalAction::CreatureWeaponAttack {
            feature_id: definition.into(),
            choice: choice.clone(),
        }),
    ))
    .await;
    Box::pin(raw(f, path, TableTransportChannel::Host, 1)).await;
    Box::pin(step(
        f,
        path,
        TableTransportChannel::Host,
        action(TacticalAction::EndTurn),
    ))
    .await;
    let mut melee = throw_choice(dagger, source);
    melee.delivery = WeaponDelivery::Melee;
    melee.after_equipment = Some(AfterAttackEquipmentIntent::Choose);
    Box::pin(player_step(
        f,
        path,
        action(TacticalAction::Attack { choice: melee }),
    ))
    .await;
    Box::pin(player_raw(f, path, 20)).await;
    Box::pin(table_hit_driver::decline_hit_responses(f)).await;
    Box::pin(player_raw(f, path, 4)).await;
    let at_knockout = view(f, &player(f)).await.tactical.unwrap();
    assert!(
        at_knockout.attack_equipment.is_none(),
        "after-equipment must wait for the real knockout choice"
    );
    assert_eq!(
        at_knockout.attack_decision.unwrap().kind,
        TableAttackDecisionKind::Knockout
    );
    Box::pin(player_step(
        f,
        path,
        action(TacticalAction::ChooseAttackKnockout {
            choice: KnockoutChoice::KnockOut,
        }),
    ))
    .await;
    let after_knockout = state(f).await;
    assert_eq!(
        after_knockout.rules.as_ref().unwrap().entities[&source].hp,
        1
    );
    assert!(matches!(
        after_knockout.items[&item.id].custody,
        Custody::Location(_)
    ));
    assert!(
        after_knockout
            .encounter
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .ground_items
            .iter()
            .any(|ground| ground.item == item.id)
    );
    let shown = view(f, &player(f))
        .await
        .tactical
        .unwrap()
        .attack_equipment
        .unwrap();
    Box::pin(player_step(
        f,
        path,
        TableTransportInput::AttackEquipment {
            handle: shown.key,
            choice: AttackEquipmentChoice::Decline,
        },
    ))
    .await;
    Box::pin(next_player_turn(f, path)).await;
    Box::pin(player_step(
        f,
        path,
        action(TacticalAction::FirstAid {
            target: source,
            purpose: MedicinePurpose::EndKnockout,
        }),
    ))
    .await;
    Box::pin(player_raw(f, path, 20)).await;
    Box::pin(player_step(f, path, action(TacticalAction::EndTurn))).await;
    let before = state(f).await;
    assert!(
        !dmd_rules::active_conditions(before.rules.as_ref().unwrap(), source)
            .contains(&Condition::Unconscious)
    );
    assert_eq!(before.items[&item.id].owner, item.owner);
    let options = view(f, &TableTransportChannel::Host)
        .await
        .tactical
        .unwrap()
        .attack_options
        .unwrap();
    assert!(options.weapons.iter().any(|w| {
        w.item == item.id
            && w.source_features
                .iter()
                .any(|feature| feature.feature_id == definition)
    }));
    choice.equipment_change = Some(AttackEquipmentChange {
        timing: EquipmentChangeTiming::BeforeAttack,
        operation: AttackEquipmentOperation::Pickup {
            item: item.id,
            hand: Hand::Right,
        },
    });
    Box::pin(step(
        f,
        path,
        TableTransportChannel::Host,
        action(TacticalAction::CreatureWeaponAttack {
            feature_id: definition.into(),
            choice,
        }),
    ))
    .await;
    let paid = state(f).await;
    assert_eq!(paid.items[&item.id].custody, Custody::Entity(source));
    assert_eq!(paid.items[&item.id].owner, item.owner);
    assert_eq!(paid.items[&item.id].quantity, 1);
    assert_eq!(
        paid.items[&arrows].quantity,
        before.items[&arrows].quantity - u32::from(bow)
    );
    assert!(
        paid.rules
            .as_ref()
            .unwrap()
            .timing
            .as_ref()
            .unwrap()
            .action_spent
    );
    Box::pin(raw(f, path, TableTransportChannel::Host, 1)).await;
    let finished = state(f).await;
    assert_eq!(
        finished
            .rules
            .as_ref()
            .unwrap()
            .tactical_creatures
            .as_ref()
            .unwrap()
            .profile(source)
            .unwrap(),
        &source_pin
    );
    assert_eq!(finished.items.len(), initial.items.len());
    assert!(
        finished
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

#[tokio::test]
async fn owned_after_throw_apply_or_decline_survives_real_cold_sqlite_and_exact_transport_retry() {
    for mode in 0..3 {
        let directory =
            std::env::temp_dir().join(format!("dmd-ground-after-{}", CampaignId::new().0));
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join("campaign.sqlite");
        let pool = dmd_persistence::open_sqlite_path(&path).await.unwrap();
        let mut creation = input("Recovery actor");
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
        if mode < 2 {
            Box::pin(selected_throw(&mut f, &path, mode == 1)).await;
        } else {
            Box::pin(no_apply(&mut f, &path)).await;
        }
        f.pool.close().await;
        drop(f);
        sqlite_test_cleanup::remove_closed_directory(&directory)
            .await
            .unwrap();
    }
}

#[tokio::test]
async fn selected_and_different_before_pickup_preserve_real_item_identity_and_atomic_paid_cuts() {
    for different in [false, true] {
        let directory =
            std::env::temp_dir().join(format!("dmd-ground-before-{}", CampaignId::new().0));
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join("campaign.sqlite");
        let pool = dmd_persistence::open_sqlite_path(&path).await.unwrap();
        let mut creation = input("Recovery actor");
        creation.purchases.push(EquipmentChoice {
            item_id: "dagger".into(),
            quantity: 2,
        });
        let mut f = Box::pin(Fixture::with_creation_pool(
            TableContract::default(),
            Some(creation),
            pool,
        ))
        .await;
        Box::pin(before_pickup(&mut f, &path, different)).await;
        f.pool.close().await;
        drop(f);
        sqlite_test_cleanup::remove_closed_directory(&directory)
            .await
            .unwrap();
    }
}

#[tokio::test]
async fn pinned_parent_goblin_scimitar_shortbow_after_miss_and_critical_keep_paid_source_through_cold_damage()
 {
    for (bow, hit, player_controlled) in [
        (false, false, false),
        (false, true, false),
        (true, false, false),
        (true, true, false),
        (false, false, true),
        (true, true, true),
    ] {
        let directory =
            std::env::temp_dir().join(format!("dmd-ground-source-{}", CampaignId::new().0));
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join("campaign.sqlite");
        let pool = dmd_persistence::open_sqlite_path(&path).await.unwrap();
        let mut f = Box::pin(Fixture::with_pool(TableContract::default(), pool)).await;
        Box::pin(printed_after(&mut f, &path, bow, hit, player_controlled)).await;
        f.pool.close().await;
        drop(f);
        sqlite_test_cleanup::remove_closed_directory(&directory)
            .await
            .unwrap();
    }
}

#[tokio::test]
async fn genuine_goblin_scimitar_and_shortbow_before_pickup_follow_knockout_drop_and_physical_first_aid()
 {
    for bow in [false, true] {
        let directory =
            std::env::temp_dir().join(format!("dmd-ground-source-before-{}", CampaignId::new().0));
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join("campaign.sqlite");
        let pool = dmd_persistence::open_sqlite_path(&path).await.unwrap();
        let mut creation = input("Recovery actor");
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
        Box::pin(printed_before_after_real_knockout(&mut f, &path, bow)).await;
        f.pool.close().await;
        drop(f);
        sqlite_test_cleanup::remove_closed_directory(&directory)
            .await
            .unwrap();
    }
}
