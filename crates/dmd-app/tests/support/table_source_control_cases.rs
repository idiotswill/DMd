use super::*;
use dmd_persistence::{CampaignExport, ProjectionAudience};
use dmd_rules::tactical::TacticalAction;

fn runtime(pool: sqlx::SqlitePool) -> CampaignRuntime {
    CampaignRuntime::from_content_root(
        pool,
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../content"),
    )
}
async fn presented(f: &Fixture, channel: &TableTransportChannel) -> TablePresentedView {
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
    action: TableAction,
) -> TableTransportRequest {
    let view = presented(f, &channel).await;
    TableTransportRequest {
        version: if view.source_control.is_some()
            || matches!(action, TableAction::EnableSourceActorAccess { .. })
        {
            2
        } else {
            1
        },
        command_id: CommandId::new(),
        campaign_id: f.campaign,
        session_id: view
            .active_session
            .as_ref()
            .map(|session| session.session_id),
        channel,
        revision: view.revision,
        input: TableTransportInput::Action(Box::new(action)),
    }
}
async fn accept(
    f: &Fixture,
    channel: TableTransportChannel,
    action: TableAction,
) -> TableTransportRequest {
    let request = request(f, channel, action).await;
    Box::pin(f.runtime.submit_presented_table(request.clone()))
        .await
        .unwrap();
    request
}
async fn unchanged_rejection(f: &Fixture, request: TableTransportRequest) {
    unchanged_rejection_message(f, request).await;
}
async fn owner_rejection(f: &Fixture, request: TableTransportRequest) {
    assert_eq!(
        unchanged_rejection_message(f, request).await,
        "This source creature's player must make the decision or report its public dice."
    );
}
async fn unchanged_rejection_message(f: &Fixture, request: TableTransportRequest) -> String {
    let before = export_campaign(&f.pool, f.campaign).await.unwrap();
    let Err(RunnableCampaignError::TableRejected(message)) =
        Box::pin(f.runtime.submit_presented_table(request)).await
    else {
        panic!("expected a definite rejection before any write");
    };
    let mut after = export_campaign(&f.pool, f.campaign).await.unwrap();
    after.exported_at_utc = before.exported_at_utc.clone();
    assert_eq!(after, before);
    message
}
async fn reopen(f: &mut Fixture, path: &Path) {
    let expected = f
        .runtime
        .open_campaign(f.campaign)
        .await
        .unwrap()
        .state()
        .clone();
    f.pool.close().await;
    f.pool = dmd_persistence::open_sqlite_path(path).await.unwrap();
    f.runtime = runtime(f.pool.clone());
    assert_eq!(
        f.runtime.resume_campaign(f.campaign).await.unwrap().state(),
        &expected
    );
}
async fn create_mage(f: &Fixture) -> (EntityId, TableTransportRequest, TableTransportResult) {
    let host = presented(f, &TableTransportChannel::Host).await;
    let catalog = f
        .runtime
        .table_creature_options(TableCreatureOptionsRequest {
            campaign_id: f.campaign,
            channel: TableTransportChannel::Host,
            revision: host.revision,
        })
        .await
        .unwrap();
    let source = catalog
        .iter()
        .find(|entry| entry.definition_id == "mage")
        .unwrap();
    let actor = EntityId::new();
    let request = request(
        f,
        TableTransportChannel::Host,
        TableAction::CreateCreature {
            creation: Box::new(TableCreatureCreation {
                entity_id: actor,
                name: "Private source Mage".into(),
                definition_id: source.definition_id.clone(),
                size: CreatureSize::Medium,
                additional_languages: vec!["dwarvish".into(), "elvish".into(), "draconic".into()],
                ammunition_units: 0,
                item_ids: (0..source.item_count).map(|_| ItemId::new()).collect(),
            }),
        },
    )
    .await;
    let response = Box::pin(f.runtime.submit_presented_table(request.clone()))
        .await
        .unwrap();
    (actor, request, response)
}

#[tokio::test]
async fn source_control_real_mage_ownership_raw_dice_self_cast_and_cold_retry() {
    let path =
        std::env::temp_dir().join(format!("dmd-source-control-{}.sqlite", CampaignId::new().0));
    let pool = dmd_persistence::open_sqlite_path(&path).await.unwrap();
    let mut f = Box::pin(Fixture::with_pool(TableContract::default(), pool)).await;
    let (mage, old_request, old_response) = Box::pin(create_mage(&f)).await;
    let prefix = export_campaign(&f.pool, f.campaign).await.unwrap();
    let (activation, assignment) = Box::pin(enable_and_assign(&f, mage)).await;
    Box::pin(reopen(&mut f, &path)).await;
    assert_eq!(
        Box::pin(f.runtime.submit_presented_table(old_request.clone()))
            .await
            .unwrap(),
        old_response
    );
    let mut fresh_old = old_request;
    fresh_old.command_id = CommandId::new();
    unchanged_rejection(&f, fresh_old).await;
    let after = export_campaign(&f.pool, f.campaign).await.unwrap();
    assert_eq!(
        &after.table_projection_history[..prefix.table_projection_history.len()],
        &prefix.table_projection_history
    );
    // Bindings are ordered by random CommandId, never by acceptance time.
    for original in &prefix.table_transport_bindings {
        assert_eq!(
            after
                .table_transport_bindings
                .iter()
                .find(|binding| binding.meta.id == original.meta.id),
            Some(original)
        );
    }
    assert!(
        after.table_projection_history[prefix.table_projection_history.len()..]
            .iter()
            .all(|record| record.version == 2)
    );
    let raw = Box::pin(prepare_owned_turn(&mut f, mage)).await;
    Box::pin(reopen(&mut f, &path)).await;
    let rolled = Box::pin(f.runtime.submit_presented_table(raw.clone()))
        .await
        .unwrap();
    assert_eq!(
        Box::pin(f.runtime.submit_presented_table(raw.clone()))
            .await
            .unwrap(),
        rolled
    );
    let cast = Box::pin(cast_mage_armor(&f, mage)).await;
    Box::pin(reopen(&mut f, &path)).await;
    let result = Box::pin(f.runtime.submit_presented_table(cast.clone()))
        .await
        .unwrap();
    Box::pin(reject_transfer_of_held_work(&f, mage)).await;
    accept(
        &f,
        TableTransportChannel::Host,
        TableAction::SetSourceCreatureController {
            actor: mage,
            controller: CreatureController::Host,
        },
    )
    .await;
    // Accepted bodies recover before ownership/current-revision checks, including the activation itself.
    assert_eq!(
        Box::pin(f.runtime.submit_presented_table(cast.clone()))
            .await
            .unwrap(),
        result
    );
    assert_eq!(
        Box::pin(f.runtime.submit_presented_table(raw))
            .await
            .unwrap(),
        rolled
    );
    for accepted in [activation, assignment] {
        Box::pin(f.runtime.submit_presented_table(accepted))
            .await
            .unwrap();
    }
    let mut new_cast = cast;
    new_cast.command_id = CommandId::new();
    unchanged_rejection(&f, new_cast).await;
    let owner = presented(
        &f,
        &TableTransportChannel::SourceCreature {
            player_id: f.players[0],
            actor: mage,
        },
    )
    .await;
    assert!(owner.source_control.unwrap().actors.is_empty());
    let export = export_campaign(&f.pool, f.campaign).await.unwrap();
    Box::pin(restore_and_reject_forgery(&f, &export, mage)).await;
    f.pool.close().await;
    drop(f);
    sqlite_test_cleanup::remove_closed_file(&path)
        .await
        .unwrap();
}

async fn enable_and_assign(
    f: &Fixture,
    mage: EntityId,
) -> (TableTransportRequest, TableTransportRequest) {
    let host = presented(f, &TableTransportChannel::Host).await;
    assert!(host.source_control.is_none());
    let options = f
        .runtime
        .table_source_control_options(TableCreatureOptionsRequest {
            campaign_id: f.campaign,
            channel: TableTransportChannel::Host,
            revision: host.revision,
        })
        .await
        .unwrap();
    assert!(!options.enabled && options.settled && options.adopted.is_empty());
    let activation = request(
        f,
        TableTransportChannel::Host,
        TableAction::EnableSourceActorAccess {
            adopted: options.adopted,
        },
    )
    .await;
    let mut wrong = activation.clone();
    wrong.version = 1;
    unchanged_rejection(f, wrong).await;
    let before = f
        .runtime
        .open_campaign(f.campaign)
        .await
        .unwrap()
        .state()
        .clone();
    Box::pin(f.runtime.submit_presented_table(activation.clone()))
        .await
        .unwrap();
    let enabled_export = export_campaign(&f.pool, f.campaign).await.unwrap();
    let legacy_meta = f.meta(CommandIssuer::Admin, None, Some(f.session)).await;
    assert!(matches!(
        Box::pin(f.runtime.execute_table(
            legacy_meta,
            TableAction::SetSituation {
                situation: TableSituation::default()
            }
        ))
        .await,
        Err(RunnableCampaignError::TableRejected(_))
    ));
    let legacy_observation = f.player_meta(0).await;
    assert!(matches!(
        Box::pin(f.runtime.observe_table_text(
            legacy_observation.clone(),
            ObservationId(legacy_observation.id.0),
            "What can I do?"
        ))
        .await,
        Err(RunnableCampaignError::TableRejected(_))
    ));
    let mut unchanged = export_campaign(&f.pool, f.campaign).await.unwrap();
    unchanged.exported_at_utc = enabled_export.exported_at_utc.clone();
    assert_eq!(
        unchanged, enabled_export,
        "fresh legacy writers cannot bypass v2 admission"
    );
    assert_eq!(
        f.runtime
            .open_campaign(f.campaign)
            .await
            .unwrap()
            .state()
            .rules,
        before.rules
    );
    let assignment = accept(
        f,
        TableTransportChannel::Host,
        TableAction::SetSourceCreatureController {
            actor: mage,
            controller: CreatureController::Player(f.players[0]),
        },
    )
    .await;
    let after = f
        .runtime
        .open_campaign(f.campaign)
        .await
        .unwrap()
        .state()
        .clone();
    assert_eq!(after.characters.len(), before.characters.len());
    assert_eq!(after.items, before.items);
    let creatures = after
        .rules
        .as_ref()
        .unwrap()
        .tactical_creatures
        .as_ref()
        .unwrap();
    assert_eq!(
        creatures.runtime(mage).unwrap().controller,
        CreatureController::Player(f.players[0])
    );
    assert_eq!(
        creatures.runtime(mage).unwrap().control_origin.id,
        assignment.command_id
    );
    let before_runtime = before
        .rules
        .as_ref()
        .unwrap()
        .tactical_creatures
        .as_ref()
        .unwrap()
        .runtime(mage)
        .unwrap();
    assert_eq!(
        creatures.runtime(mage).unwrap().in_lair,
        before_runtime.in_lair
    );
    assert_eq!(
        creatures.runtime(mage).unwrap().lair_origin,
        before_runtime.lair_origin
    );
    let other = presented(
        f,
        &TableTransportChannel::Player {
            player_id: f.players[1],
            character_id: f.characters[1],
        },
    )
    .await;
    assert!(other.source_control.as_ref().unwrap().actors.is_empty());
    assert!(
        !serde_json::to_string(&other)
            .unwrap()
            .contains("Private source Mage")
    );
    (activation, assignment)
}

async fn prepare_owned_turn(f: &mut Fixture, mage: EntityId) -> TableTransportRequest {
    accept(f, TableTransportChannel::Host, TableAction::EndSession).await;
    f.session = PlaySessionId::new();
    let mut start = request(
        f,
        TableTransportChannel::Host,
        TableAction::StartSession {
            id: f.session,
            name: "Source owner session".into(),
            participants: vec![
                SessionParticipant {
                    player_id: f.players[0],
                    character_id: None,
                    attendance: AttendanceStatus::Present,
                },
                SessionParticipant {
                    player_id: f.players[1],
                    character_id: None,
                    attendance: AttendanceStatus::Absent,
                },
            ],
        },
    )
    .await;
    start.session_id = Some(f.session);
    Box::pin(f.runtime.submit_presented_table(start))
        .await
        .unwrap();
    let p = |x, y, z| SpatialPoint { x, y, z };
    accept(
        f,
        TableTransportChannel::Host,
        TableAction::PrepareBattlefield {
            setup: Box::new(TableBattlefieldSetup {
                encounter_id: EncounterId::new(),
                scene_id: SceneId::new(),
                location_id: LocationId::new(),
                name: "Source courtyard".into(),
                battlefield: Battlefield {
                    bounds: SpatialBox {
                        min: p(0, 0, 0),
                        max: p(100, 100, 40),
                    },
                    floor_z: 0,
                    floor_surface: "stone".into(),
                    ambient_light: LightLevel::Bright,
                    terrain: vec![],
                    obstacles: vec![],
                    lights: vec![],
                },
                characters: vec![],
                creatures: vec![TableCreaturePlacement {
                    actor: mage,
                    public_label: "Spellcaster".into(),
                    position: p(10, 10, 0),
                    height: 12,
                    allies: vec![],
                    enemies: vec![],
                }],
                area_grid_policy: None,
                geometry_ruling: Ruling {
                    basis: RulingBasis::GmAdjudication,
                    reason: "An open stone courtyard.".into(),
                },
            }),
        },
    )
    .await;
    accept(
        f,
        TableTransportChannel::Host,
        TableAction::Tactical {
            action: TacticalAction::Begin {
                execution: TacticalExecutionVersion::ReactionsV1,
                combatants: vec![TacticalCombatant {
                    actor: mage,
                    source: TacticalSource::Creature {
                        definition_id: "mage".into(),
                    },
                    surprised: false,
                }],
                groups: vec![InitiativeGroup {
                    actors: vec![mage],
                    request_id: RollRequestId::new(),
                }],
            },
        },
    )
    .await;
    let owner = TableTransportChannel::SourceCreature {
        player_id: f.players[0],
        actor: mage,
    };
    let view = presented(f, &owner).await;
    let roll = view
        .roll
        .as_ref()
        .expect("owned source initiative is visible");
    assert_eq!(roll.roller, Some(mage));
    assert_eq!(view.roll_channel, Some(TableRollChannel::Tactical));
    assert!(
        presented(
            f,
            &TableTransportChannel::Player {
                player_id: f.players[1],
                character_id: f.characters[1]
            }
        )
        .await
        .roll
        .is_none()
    );
    let transfer = request(
        f,
        TableTransportChannel::Host,
        TableAction::SetSourceCreatureController {
            actor: mage,
            controller: CreatureController::Host,
        },
    )
    .await;
    unchanged_rejection(f, transfer).await;
    let raw = request(
        f,
        owner,
        TableAction::Tactical {
            action: TacticalAction::SubmitRoll {
                result: RollResult {
                    request_id: roll.id,
                    source: RollSource::Physical,
                    dice: vec![DieResult {
                        sides: 20,
                        value: 12,
                    }],
                },
            },
        },
    )
    .await;
    let mut host_raw = raw.clone();
    host_raw.channel = TableTransportChannel::Host;
    let host_view = presented(f, &TableTransportChannel::Host).await;
    host_raw.revision = host_view.revision;
    let TableTransportInput::Action(host_action) = &mut host_raw.input else {
        unreachable!()
    };
    let TableAction::Tactical {
        action: TacticalAction::SubmitRoll { result },
    } = host_action.as_mut()
    else {
        unreachable!()
    };
    result.request_id = host_view.roll.unwrap().id;
    owner_rejection(f, host_raw).await;
    let mut stolen = raw.clone();
    stolen.channel = TableTransportChannel::SourceCreature {
        player_id: f.players[1],
        actor: mage,
    };
    unchanged_rejection(f, stolen).await;
    let mut wrong_actor = raw.clone();
    wrong_actor.channel = TableTransportChannel::SourceCreature {
        player_id: f.players[0],
        actor: f.actors[0],
    };
    unchanged_rejection(f, wrong_actor).await;
    raw
}

async fn cast_mage_armor(f: &Fixture, mage: EntityId) -> TableTransportRequest {
    let channel = TableTransportChannel::SourceCreature {
        player_id: f.players[0],
        actor: mage,
    };
    let view = presented(f, &channel).await;
    let tactical = view.tactical.unwrap();
    assert_eq!(tactical.active_actor, Some(mage));
    let options = tactical.casting_options.unwrap();
    let variant = options
        .variants
        .iter()
        .find(|variant| variant.choice.spell_id == "mage-armor")
        .expect("real source self-cast option");
    assert!(matches!(
        variant.choice.material,
        SpellMaterialChoice::Material { .. }
    ));
    assert_eq!(
        variant
            .targets
            .iter()
            .map(|target| target.actor)
            .collect::<Vec<_>>(),
        vec![mage]
    );
    let before = f
        .runtime
        .open_campaign(f.campaign)
        .await
        .unwrap()
        .state()
        .clone();
    let cast = request(
        f,
        channel,
        TableAction::Tactical {
            action: TacticalAction::CastSpell {
                choice: variant.choice.clone(),
                targets: SpellTargetChoice::Entities(vec![mage]),
            },
        },
    )
    .await;
    let mut host_cast = cast.clone();
    host_cast.channel = TableTransportChannel::Host;
    host_cast.revision = presented(f, &TableTransportChannel::Host).await.revision;
    owner_rejection(f, host_cast).await;
    Box::pin(f.runtime.submit_presented_table(cast.clone()))
        .await
        .unwrap();
    let after = f
        .runtime
        .open_campaign(f.campaign)
        .await
        .unwrap()
        .state()
        .clone();
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
    assert!(
        after
            .rules
            .as_ref()
            .unwrap()
            .timing
            .as_ref()
            .unwrap()
            .action_spent
    );
    assert_eq!(
        after.rules.as_ref().unwrap().entities[&mage].armor,
        ArmorClass::Fixed(12)
    );
    assert_eq!(
        dmd_rules::tactical_defenses::effective_armor_class(&after, mage).unwrap(),
        15
    );
    assert_eq!(
        after.items, before.items,
        "nonconsumed source material remains physical"
    );
    cast
}

async fn reject_transfer_of_held_work(f: &Fixture, mage: EntityId) {
    let channel = TableTransportChannel::SourceCreature {
        player_id: f.players[0],
        actor: mage,
    };
    accept(
        f,
        channel.clone(),
        TableAction::Tactical {
            action: TacticalAction::EndTurn,
        },
    )
    .await;
    accept(
        f,
        channel.clone(),
        TableAction::Tactical {
            action: TacticalAction::Ready {
                trigger: ReadyTrigger::MovementFinished {
                    subject: ReadySubject::AnyOther,
                },
                action: ReadyAction::Move,
            },
        },
    )
    .await;
    let transfer = request(
        f,
        TableTransportChannel::Host,
        TableAction::SetSourceCreatureController {
            actor: mage,
            controller: CreatureController::Host,
        },
    )
    .await;
    unchanged_rejection(f, transfer).await;
    let host_abandon = request(
        f,
        TableTransportChannel::Host,
        TableAction::Tactical {
            action: TacticalAction::AbandonReady { actor: mage },
        },
    )
    .await;
    owner_rejection(f, host_abandon).await;
    accept(
        f,
        channel,
        TableAction::Tactical {
            action: TacticalAction::AbandonReady { actor: mage },
        },
    )
    .await;
}

async fn restore_and_reject_forgery(f: &Fixture, export: &CampaignExport, mage: EntityId) {
    let pool = open_sqlite("sqlite::memory:").await.unwrap();
    let app = runtime(pool.clone());
    Box::pin(app.restore_campaign(export)).await.unwrap();
    assert_eq!(
        app.open_campaign(f.campaign).await.unwrap().state(),
        f.runtime.open_campaign(f.campaign).await.unwrap().state()
    );
    pool.close().await;
    let binding_id = export
        .table_transport_bindings
        .iter()
        .find(|binding| binding.version == 2 && binding.audience == ProjectionAudience::Host)
        .expect("genuine activated Host binding")
        .meta
        .id;
    let source_binding_id = export.table_transport_bindings.iter().find(|binding| {
        let original:TableTransportRequest=serde_json::from_str(&binding.request_json).unwrap();
        binding.version==2 && matches!(original.channel,TableTransportChannel::SourceCreature{player_id,actor} if player_id==f.players[0] && actor==mage)
            && matches!(original.input,TableTransportInput::Action(action) if matches!(*action,TableAction::Tactical{action:TacticalAction::SubmitRoll{..}}))
    }).expect("genuine owned source raw-roll binding").meta.id;
    let mut cases = Vec::new();
    let mut bad = export.clone();
    bad.table_projection_history.clear();
    bad.table_transport_bindings.clear();
    cases.push(("dropped presentation history", bad));
    let mut bad = export.clone();
    bad.table_projection_history.last_mut().unwrap().version = 1;
    cases.push(("downgraded projection version", bad));
    let mut bad = export.clone();
    bad.table_transport_bindings
        .iter_mut()
        .find(|binding| binding.meta.id == binding_id)
        .unwrap()
        .version = 1;
    cases.push(("downgraded v2 binding", bad));
    let mut bad = export.clone();
    bad.table_transport_bindings
        .iter_mut()
        .find(|binding| binding.meta.id == source_binding_id)
        .unwrap()
        .audience = ProjectionAudience::Player(f.players[1]);
    cases.push(("foreign binding audience", bad));
    let mut bad = export.clone();
    let binding = bad
        .table_transport_bindings
        .iter_mut()
        .find(|binding| binding.meta.id == source_binding_id)
        .unwrap();
    let mut input: TableTransportRequest = serde_json::from_str(&binding.request_json).unwrap();
    input.channel = TableTransportChannel::SourceCreature {
        player_id: f.players[1],
        actor: mage,
    };
    binding.request_json = serde_json::to_string(&input).unwrap();
    cases.push(("foreign source request channel", bad));
    let mut bad = export.clone();
    bad.current_state.state_json = bad
        .current_state
        .state_json
        .replace("SourceActorsV1", "InventedControl");
    cases.push(("unknown source-access version", bad));
    for kind in 0..4 {
        let mut bad = export.clone();
        let mut changed = CampaignState::decode_json(&bad.current_state.state_json).unwrap();
        match kind {
            0 => changed.table.as_mut().unwrap().source_actor_access = None,
            1 => {
                changed
                    .table
                    .as_mut()
                    .unwrap()
                    .source_actor_access
                    .as_mut()
                    .unwrap()
                    .origin
                    .id = CommandId::new()
            }
            2 => {
                changed
                    .rules
                    .as_mut()
                    .unwrap()
                    .tactical_creatures
                    .as_mut()
                    .unwrap()
                    .runtime
                    .iter_mut()
                    .find(|entry| entry.actor == mage)
                    .unwrap()
                    .control_origin
                    .id = CommandId::new()
            }
            _ => changed
                .table
                .as_mut()
                .unwrap()
                .source_actor_access
                .as_mut()
                .unwrap()
                .adopted
                .push(TableSourceAdoption {
                    actor: mage,
                    player_id: f.players[1],
                    source: changed
                        .rules
                        .as_ref()
                        .unwrap()
                        .tactical_creatures
                        .as_ref()
                        .unwrap()
                        .profile(mage)
                        .unwrap()
                        .source
                        .clone(),
                    profile_origin: changed
                        .rules
                        .as_ref()
                        .unwrap()
                        .tactical_creatures
                        .as_ref()
                        .unwrap()
                        .profile(mage)
                        .unwrap()
                        .origin
                        .clone(),
                    control_origin: changed
                        .rules
                        .as_ref()
                        .unwrap()
                        .tactical_creatures
                        .as_ref()
                        .unwrap()
                        .runtime(mage)
                        .unwrap()
                        .control_origin
                        .clone(),
                }),
        }
        bad.current_state.state_json = changed.encode_json().unwrap();
        bad.snapshots
            .retain(|row| row.event_sequence != bad.current_state.applied_event_sequence);
        bad.snapshots.push(dmd_persistence::SnapshotRow {
            campaign_id: bad.current_state.campaign_id.clone(),
            event_sequence: bad.current_state.applied_event_sequence,
            state_schema_version: bad.current_state.schema_version,
            state_json: bad.current_state.state_json.clone(),
            created_at_utc: "2026-09-25 00:00:00".into(),
        });
        cases.push((
            [
                "removed activation marker",
                "invented activation command",
                "invented control command",
                "invented adoption",
            ][kind],
            bad,
        ));
    }
    let mut bad = export.clone();
    bad.table_transport_bindings
        .iter_mut()
        .find(|binding| binding.meta.id == source_binding_id)
        .unwrap()
        .response_json = "{}".into();
    cases.push(("changed accepted response", bad));
    let mut bad = export.clone();
    let capability = bad
        .table_projection_history
        .iter_mut()
        .flat_map(|record| &mut record.changes)
        .filter(|change| change.audience == ProjectionAudience::Player(f.players[0]))
        .flat_map(|change| &mut change.handles)
        .find(|handle| {
            matches!(
                handle.capability,
                dmd_persistence::ProjectionCapability::Roll { .. }
            )
        })
        .unwrap();
    capability.opaque = uuid::Uuid::new_v4();
    cases.push(("changed opaque capability", bad));
    for (label, bad) in cases {
        assert_ne!(
            &bad, export,
            "{label} must actually alter the source export"
        );
        let pool = open_sqlite("sqlite::memory:").await.unwrap();
        let app = runtime(pool.clone());
        assert!(
            Box::pin(app.restore_campaign(&bad)).await.is_err(),
            "{label}"
        );
        for table in [
            "campaign_state_current",
            "campaign_lifecycle",
            "event_journal",
            "command_audit",
            "campaign_snapshots",
            "table_projection_history",
            "table_transport_bindings",
        ] {
            let count: i64 = sqlx::query_scalar(&format!("SELECT COUNT(*) FROM {table}"))
                .fetch_one(&pool)
                .await
                .unwrap();
            assert_eq!(count, 0, "{label}: {table}");
        }
        pool.close().await;
    }
}

#[tokio::test]
async fn source_control_keeps_genuine_v1_corpus_bytes_and_original_legacy_acceptance() {
    for captured in [
        include_str!("../fixtures/legacy-savage-f960.json"),
        include_str!("../fixtures/legacy-declared-second-wind-f669.json"),
    ] {
        let export = CampaignExport::from_json(captured).unwrap();
        let campaign = CampaignState::decode_json(&export.current_state.state_json)
            .unwrap()
            .campaign_id();
        let pool = open_sqlite("sqlite::memory:").await.unwrap();
        let app = runtime(pool.clone());
        Box::pin(app.restore_campaign(&export)).await.unwrap();
        let raw = app.table_view(campaign, TableViewer::Host).await.unwrap();
        assert!(raw.source_control.is_none());
        assert!(
            !serde_json::to_string(&raw)
                .unwrap()
                .contains("source_control")
        );
        assert!(
            export.table_transport_bindings.is_empty(),
            "these old captures predate accepted modern transport; no binding evidence is claimed"
        );
        let event: TableEvent =
            serde_json::from_str(&export.event_journal.last().unwrap().payload_json).unwrap();
        let receipt = Box::pin(app.execute_table(event.meta.clone(), event.action))
            .await
            .unwrap();
        assert!(receipt.already_accepted);
        assert_eq!(receipt.outcome, event.outcome);
        let mut after = export_campaign(&pool, campaign).await.unwrap();
        after.exported_at_utc = export.exported_at_utc.clone();
        assert_eq!(
            after, export,
            "every captured historical byte remains unchanged"
        );
        pool.close().await;
    }
}
