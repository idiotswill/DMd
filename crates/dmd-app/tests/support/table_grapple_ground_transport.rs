//! Real accepted creation and physical grip setup, reused without editing its
//! original helpers. Every new command uses the explicit v4 transport.
use super::*;

// Separate accepted setup gives the independent reactor its actual hostile
// allegiance to the holder. The original fixture's asymmetric enemies stay exact.
async fn opportunity_fixture() -> Fixture {
    ground_fixture(vec![], vec![]).await
}
async fn ground_fixture(terrain: Vec<TerrainVolume>, obstacles: Vec<SpatialObstacle>) -> Fixture {
    let directory = std::env::temp_dir().join(format!("dmd-ground-drag-{}", CampaignId::new().0));
    std::fs::create_dir(&directory).unwrap();
    let path = directory.join("campaign.sqlite");
    let pool = open_sqlite_path(&path).await.unwrap();
    let f = Fixture {
        runtime: runtime(pool.clone()),
        pool,
        directory,
        path,
        campaign: CampaignId::new(),
        players: [PlayerId::new(), PlayerId::new()],
        characters: [CharacterId::new(), CharacterId::new()],
        actors: [EntityId::new(), EntityId::new()],
        session: PlaySessionId::new(),
        goblin: EntityId::new(),
        opponent: Some(EntityId::new()),
    };
    f.runtime
        .create_table_campaign(
            f.campaign,
            "Ground drag reactions",
            TableContract::default(),
        )
        .await
        .unwrap();
    for i in 0..2 {
        Box::pin(f.host(TableAction::AddPlayer {
            id: f.players[i],
            name: format!("Player {i}"),
        }))
        .await;
        Box::pin(f.host(TableAction::CreateCharacter {
            character_id: f.characters[i],
            entity_id: f.actors[i],
            player_id: f.players[i],
            input: creation(&format!("Character {i}")),
        }))
        .await;
    }
    let view = f.view(TableTransportChannel::Host).await;
    let catalog = f
        .runtime
        .table_creature_options(TableCreatureOptionsRequest {
            campaign_id: f.campaign,
            channel: TableTransportChannel::Host,
            revision: view.revision,
        })
        .await
        .unwrap();
    let source = catalog
        .iter()
        .find(|source| source.definition_id == "goblin-warrior")
        .unwrap();
    for actor in [f.goblin, f.opponent.unwrap()] {
        Box::pin(f.host(TableAction::CreateCreature {
            creation: Box::new(TableCreatureCreation {
                entity_id: actor,
                name: "Independent goblin".into(),
                definition_id: source.definition_id.clone(),
                source: source.source.clone(),
                size: CreatureSize::Small,
                additional_languages: vec![],
                ammunition_units: 20,
                item_ids: (0..source.item_count).map(|_| ItemId::new()).collect(),
            }),
        }))
        .await;
    }
    Box::pin(f.host(TableAction::EnableSourceActorAccess { adopted: vec![] })).await;
    for actor in [f.goblin, f.opponent.unwrap()] {
        Box::pin(f.host(TableAction::SetSourceCreatureController {
            actor,
            controller: CreatureController::Host,
        }))
        .await;
    }
    for i in 0..2 {
        let view = f.view(TableTransportChannel::Host).await;
        let count = view
            .characters
            .iter()
            .find(|c| c.character_id == f.characters[i])
            .unwrap()
            .equipment
            .as_ref()
            .unwrap()
            .initial_item_count;
        Box::pin(f.host(TableAction::PrepareEquipment {
            character_id: f.characters[i],
            item_ids: (0..count).map(|_| ItemId::new()).collect(),
        }))
        .await;
    }
    Box::pin(
        f.host(TableAction::StartSession {
            id: f.session,
            name: "Drag session".into(),
            participants: (0..2)
                .map(|i| SessionParticipant {
                    player_id: f.players[i],
                    character_id: Some(f.characters[i]),
                    attendance: AttendanceStatus::Present,
                })
                .collect(),
        }),
    )
    .await;
    Box::pin(
        f.host(TableAction::PrepareBattlefield {
            setup: Box::new(TableBattlefieldSetup {
                encounter_id: EncounterId::new(),
                scene_id: SceneId::new(),
                location_id: LocationId::new(),
                name: "Courtyard".into(),
                battlefield: Battlefield {
                    bounds: SpatialBox {
                        min: SpatialPoint { x: 0, y: 0, z: 0 },
                        max: SpatialPoint {
                            x: 100,
                            y: 100,
                            z: 60,
                        },
                    },
                    floor_z: 0,
                    floor_surface: "stone".into(),
                    ambient_light: LightLevel::Bright,
                    terrain,
                    obstacles,
                    lights: vec![],
                },
                characters: vec![TableCharacterPlacement {
                    character_id: f.characters[0],
                    position: SpatialPoint { x: 10, y: 10, z: 0 },
                    height: 12,
                    allies: vec![],
                    enemies: vec![f.goblin, f.opponent.unwrap()],
                }],
                creatures: vec![(f.goblin, 20, 10), (f.opponent.unwrap(), 10, 20)]
                    .into_iter()
                    .map(|(actor, x, y)| TableCreaturePlacement {
                        actor,
                        public_label: if actor == f.goblin {
                            "Small armored figure".into()
                        } else {
                            "Other guard".into()
                        },
                        position: SpatialPoint { x, y, z: 0 },
                        height: 8,
                        allies: vec![],
                        enemies: vec![f.actors[0]],
                    })
                    .collect(),
                geometry_ruling: Ruling {
                    basis: RulingBasis::GmAdjudication,
                    reason: "Independent enemy guards stand beside the holder.".into(),
                },
                area_grid_policy: None,
            }),
        }),
    )
    .await;
    Box::pin(f.host(TableAction::Tactical {
        action: TacticalAction::Begin {
            execution: TacticalExecutionVersion::EncounterReleaseV1,
            combatants: vec![
                TacticalCombatant {
                    actor: f.actors[0],
                    source: TacticalSource::Character,
                    surprised: false,
                },
                TacticalCombatant {
                    actor: f.goblin,
                    source: TacticalSource::Creature {
                        definition_id: "goblin-warrior".into(),
                    },
                    surprised: false,
                },
                TacticalCombatant {
                    actor: f.opponent.unwrap(),
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
                    actors: vec![f.goblin, f.opponent.unwrap()],
                    request_id: RollRequestId::new(),
                },
            ],
        },
    }))
    .await;
    Box::pin(f.roll(f.pc(0), 18)).await;
    Box::pin(f.roll(TableTransportChannel::Host, 2)).await;
    Box::pin(f.host(TableAction::Tactical {
        action: TacticalAction::ProposeInitiativeTie {
            order: vec![f.goblin, f.opponent.unwrap()],
        },
    }))
    .await;
    let state = f.state().await;
    let encounter = state.encounter.as_ref().unwrap();
    assert_eq!(
        dmd_rules::spatial::participant_distance(
            encounter.participant(f.actors[0]).unwrap(),
            encounter.participant(f.opponent.unwrap()).unwrap()
        )
        .unwrap(),
        10
    );
    assert!(
        encounter
            .participant(f.opponent.unwrap())
            .unwrap()
            .enemies
            .contains(&f.actors[0])
    );
    f
}

async fn request(
    f: &Fixture,
    channel: TableTransportChannel,
    input: TableTransportInput,
) -> TableTransportRequest {
    let mut request = f.request(channel, input).await;
    request.version = 4;
    request
}
async fn enable(f: &mut Fixture) {
    let request = request(
        f,
        TableTransportChannel::Host,
        TableTransportInput::Action(Box::new(TableAction::EnableGrappleTransport)),
    )
    .await;
    Box::pin(f.cold(request)).await;
}
async fn drag(f: &Fixture, path: Vec<TacticalMoveStep>) -> TableTransportRequest {
    let offered = f.view(f.pc(0)).await.grapple.unwrap();
    assert_eq!(offered.version, 4);
    assert_eq!(offered.ground_drag.len(), 1);
    request(
        f,
        f.pc(0),
        TableTransportInput::MoveGrappled {
            option: offered.ground_drag[0].key,
            path,
        },
    )
    .await
}
fn step(x: i32, y: i32) -> TacticalMoveStep {
    TacticalMoveStep {
        destination: SpatialPoint { x, y, z: 0 },
        mode: MovementMode::Walk,
    }
}
fn flow(state: &CampaignState) -> &TacticalFlow {
    state.encounter.as_ref().unwrap().flow.as_ref().unwrap()
}
async fn release(f: &Fixture) -> TableTransportRequest {
    let view = f.view(f.pc(0)).await;
    let option = view
        .grapple
        .unwrap()
        .choices
        .into_iter()
        .find(|choice| choice.label.starts_with("Release "))
        .unwrap();
    request(
        f,
        f.pc(0),
        TableTransportInput::GrappleChoice { handle: option.key },
    )
    .await
}
async fn raw(f: &Fixture, channel: TableTransportChannel, value: u16) -> TableTransportRequest {
    let roll = f.view(channel.clone()).await.roll.unwrap();
    let dice = roll
        .dice
        .iter()
        .flat_map(|die| {
            std::iter::repeat_n(
                DieResult {
                    sides: die.sides,
                    value,
                },
                if roll.mode == RollMode::Normal {
                    usize::from(die.count)
                } else {
                    2
                },
            )
        })
        .collect();
    request(
        f,
        channel,
        action(TacticalAction::SubmitRoll {
            result: RollResult {
                request_id: roll.id,
                source: RollSource::Physical,
                dice,
            },
        }),
    )
    .await
}
#[tokio::test]
async fn ground_drag_upgrade_retains_v3_retry_but_refuses_new_v3_and_foreign_raw_or_stale_options()
{
    let mut f = Box::pin(Fixture::new()).await;
    let old = Box::pin(f.activate()).await;
    let grip = Box::pin(f.establish_pc_grip()).await;
    enable(&mut f).await;
    Box::pin(f.runtime.submit_presented_table(old))
        .await
        .unwrap();
    let mut old_version = request(&f, f.pc(0), action(TacticalAction::Dodge)).await;
    old_version.version = 3;
    Box::pin(f.reject(old_version)).await;
    let direct = request(
        &f,
        f.pc(0),
        action(TacticalAction::MoveGrappled {
            grip,
            path: vec![step(20, 10)],
        }),
    )
    .await;
    Box::pin(f.reject(direct)).await;
    let offered = drag(&f, vec![step(20, 10)]).await;
    let mut foreign = offered.clone();
    foreign.command_id = CommandId::new();
    foreign.channel = f.pc(1);
    foreign.revision = f.view(f.pc(1)).await.revision;
    Box::pin(f.reject(foreign)).await;
    let other = f.view(f.pc(1)).await;
    assert!(other.grapple.unwrap().ground_drag.is_empty());
    let release = release(&f).await;
    Box::pin(f.cold(release)).await;
    Box::pin(f.reject(offered)).await;
    f.close().await;
}
#[tokio::test]
async fn ground_drag_moves_both_bodies_once_with_one_budget_and_no_selected_target_opportunity() {
    let mut f = Box::pin(Fixture::new()).await;
    Box::pin(f.activate()).await;
    let grip = Box::pin(f.establish_pc_grip()).await;
    enable(&mut f).await;
    let before = f.state().await;
    let movement = drag(&f, vec![step(20, 10), step(30, 10)]).await;
    Box::pin(f.cold(movement.clone())).await;
    let after = f.state().await;
    let encounter = after.encounter.as_ref().unwrap();
    assert_eq!(
        encounter.participant(f.actors[0]).unwrap().position,
        SpatialPoint { x: 30, y: 10, z: 0 }
    );
    assert_eq!(
        encounter.participant(f.goblin).unwrap().position,
        SpatialPoint { x: 40, y: 10, z: 0 }
    );
    assert!(flow(&after).resolution.is_none());
    assert_eq!(flow(&after).budget.movement_spent, 40);
    let result = flow(&after).last_movement.as_ref().unwrap();
    assert_eq!(result.original.id, movement.command_id);
    assert_eq!(result.reason, TacticalMovementEnd::Completed);
    assert_eq!(result.completed_steps, 2);
    assert_eq!(
        result.transport,
        Some(GrappleTransportResult {
            kind: GrappleTransportKind::GroundDragV1,
            grip,
            target: f.goblin,
            target_start: SpatialPoint { x: 20, y: 10, z: 0 },
            target_endpoint: SpatialPoint { x: 40, y: 10, z: 0 },
            ordinary_cost: 20,
            haul_cost: 20
        })
    );
    assert_eq!(
        after.rules.as_ref().unwrap().rolls,
        before.rules.as_ref().unwrap().rolls
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
        before
            .rules
            .as_ref()
            .unwrap()
            .timing
            .as_ref()
            .unwrap()
            .reactions_spent
    );
    assert_eq!(
        after
            .rules
            .as_ref()
            .unwrap()
            .tactical_grapples
            .as_ref()
            .unwrap()
            .grip(grip),
        before
            .rules
            .as_ref()
            .unwrap()
            .tactical_grapples
            .as_ref()
            .unwrap()
            .grip(grip)
    );
    let mut forged = after.clone();
    forged
        .encounter
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .last_movement
        .as_mut()
        .unwrap()
        .transport
        .as_mut()
        .unwrap()
        .haul_cost = 19;
    Box::pin(reject_state_image(&f, forged)).await;
    f.close().await;
}
#[tokio::test]
async fn ground_drag_budget_stop_keeps_the_paid_prefix_and_both_endpoints() {
    let mut f = Box::pin(Fixture::new()).await;
    Box::pin(f.activate()).await;
    Box::pin(f.establish_pc_grip()).await;
    enable(&mut f).await;
    let request = drag(
        &f,
        vec![step(20, 10), step(30, 10), step(40, 10), step(50, 10)],
    )
    .await;
    Box::pin(f.cold(request)).await;
    let state = f.state().await;
    let result = flow(&state).last_movement.as_ref().unwrap();
    assert_eq!(
        (
            result.completed_steps,
            result.requested_steps,
            result.spent_after
        ),
        (3, 4, 60)
    );
    assert_eq!(result.reason, TacticalMovementEnd::Stopped);
    assert_eq!(result.endpoint, SpatialPoint { x: 40, y: 10, z: 0 });
    assert_eq!(
        result.transport.as_ref().unwrap().target_endpoint,
        SpatialPoint { x: 50, y: 10, z: 0 }
    );
    assert_eq!(
        (
            result.transport.as_ref().unwrap().ordinary_cost,
            result.transport.as_ref().unwrap().haul_cost
        ),
        (30, 30)
    );
    f.close().await;
}
#[tokio::test]
async fn ground_drag_release_retires_only_the_unanswered_departure_and_cold_replays_the_stop() {
    let mut f = Box::pin(opportunity_fixture()).await;
    Box::pin(f.activate()).await;
    Box::pin(f.establish_pc_grip()).await;
    enable(&mut f).await;
    let movement = drag(&f, vec![step(0, 0)]).await;
    Box::pin(f.cold(movement)).await;
    let paused = f.state().await;
    assert_eq!(resolution(&paused).movement.as_ref().unwrap().next_step, 0);
    assert_eq!(
        resolution(&paused)
            .movement
            .as_ref()
            .unwrap()
            .opportunity
            .as_ref()
            .unwrap()
            .reactor,
        f.opponent.unwrap()
    );
    let release = release(&f).await;
    Box::pin(f.cold(release)).await;
    let after = f.state().await;
    let result = flow(&after).last_movement.as_ref().unwrap();
    assert_eq!((result.completed_steps, result.spent_after), (0, 0));
    assert_eq!(result.reason, TacticalMovementEnd::Stopped);
    assert_eq!(result.endpoint, SpatialPoint { x: 10, y: 10, z: 0 });
    assert_eq!(
        result.transport.as_ref().unwrap().target_endpoint,
        SpatialPoint { x: 20, y: 10, z: 0 }
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
        paused
            .rules
            .as_ref()
            .unwrap()
            .timing
            .as_ref()
            .unwrap()
            .reactions_spent
    );
    f.close().await;
}
#[tokio::test]
async fn ground_drag_release_keeps_issued_opportunity_dice_and_reaction_then_stops_after_the_child()
{
    let mut f = Box::pin(opportunity_fixture()).await;
    Box::pin(f.activate()).await;
    Box::pin(f.establish_pc_grip()).await;
    enable(&mut f).await;
    let movement = drag(&f, vec![step(0, 0)]).await;
    Box::pin(f.cold(movement)).await;
    let request = request(
        &f,
        TableTransportChannel::Host,
        action(TacticalAction::OpportunityAttack {
            choice: TacticalMeleeChoice::UnarmedDamage {
                ability: Ability::Strength,
            },
        }),
    )
    .await;
    Box::pin(f.cold(request)).await;
    let issued = f.state().await;
    let pending = issued.rules.as_ref().unwrap().pending.clone().unwrap();
    let release = release(&f).await;
    Box::pin(f.cold(release)).await;
    let waiting = f.state().await;
    assert_eq!(
        waiting.rules.as_ref().unwrap().pending.as_ref(),
        Some(&pending)
    );
    assert_eq!(resolution(&waiting).attack, resolution(&issued).attack);
    assert_eq!(resolution(&waiting).frames, resolution(&issued).frames);
    let roll = raw(&f, TableTransportChannel::Host, 1).await;
    Box::pin(f.cold(roll)).await;
    let done = f.state().await;
    assert_eq!(
        flow(&done).last_movement.as_ref().unwrap().reason,
        TacticalMovementEnd::Stopped
    );
    assert_eq!(flow(&done).budget.movement_spent, 0);
    assert_eq!(
        done.rules.as_ref().unwrap().rolls.last().unwrap().request,
        pending.request
    );
    assert!(
        done.rules
            .as_ref()
            .unwrap()
            .timing
            .as_ref()
            .unwrap()
            .reactions_spent
            .contains(&f.opponent.unwrap())
    );
    f.close().await;
}

#[tokio::test]
async fn hidden_target_only_wall_stops_the_pair_without_cost_or_relocating_either_body() {
    let wall = SpatialObstacle {
        id: "hidden seam".into(),
        volume: SpatialBox {
            min: SpatialPoint { x: 30, y: 10, z: 0 },
            max: SpatialPoint {
                x: 31,
                y: 20,
                z: 12,
            },
        },
        blocks_movement: true,
        blocks_sight: false,
        observable: false,
        cover: CoverDegree::None,
    };
    let mut f = Box::pin(ground_fixture(vec![], vec![wall])).await;
    Box::pin(f.activate()).await;
    Box::pin(f.establish_pc_grip()).await;
    enable(&mut f).await;
    let before = f.view(f.pc(0)).await;
    let visible = serde_json::to_string(&before).unwrap();
    assert!(!visible.contains("hidden seam"));
    let movement = drag(&f, vec![step(20, 10)]).await;
    Box::pin(f.cold(movement)).await;
    let after = f.state().await;
    let result = flow(&after).last_movement.as_ref().unwrap();
    assert_eq!(result.reason, TacticalMovementEnd::Stopped);
    assert_eq!((result.completed_steps, result.spent_after), (0, 0));
    assert_eq!(result.endpoint, SpatialPoint { x: 10, y: 10, z: 0 });
    assert_eq!(
        result.transport.as_ref().unwrap().target_endpoint,
        SpatialPoint { x: 20, y: 10, z: 0 }
    );
    assert!(
        !serde_json::to_string(&f.view(f.pc(0)).await)
            .unwrap()
            .contains("hidden seam")
    );
    f.close().await;
}
#[tokio::test]
async fn ground_drag_declined_opportunity_commits_one_paired_step_and_refuses_forged_retained_sources()
 {
    let mut f = Box::pin(opportunity_fixture()).await;
    Box::pin(f.activate()).await;
    Box::pin(f.establish_pc_grip()).await;
    enable(&mut f).await;
    let movement = drag(&f, vec![step(0, 0)]).await;
    Box::pin(f.cold(movement)).await;
    let paused = f.state().await;
    for mutation in ["target", "grip", "origin", "path", "stop"] {
        let mut forged = paused.clone();
        let history = forged
            .encounter
            .as_mut()
            .unwrap()
            .flow
            .as_mut()
            .unwrap()
            .resolution
            .as_mut()
            .unwrap()
            .grapple
            .as_mut()
            .unwrap()
            .transport
            .as_mut()
            .unwrap();
        match mutation {
            "target" => history.admission.target = f.opponent.unwrap(),
            "grip" => history.admission.grip = GrappleId(uuid::Uuid::new_v4()),
            "origin" => history.admission.origin.id = CommandId::new(),
            "path" => history.admission.path[0].destination = SpatialPoint { x: 0, y: 10, z: 0 },
            "stop" => {
                history.stop = Some(GrappleTransportStop {
                    cause: history.admission.origin.clone(),
                    ended_grip: history.admission.grip,
                })
            }
            _ => unreachable!(),
        }
        Box::pin(reject_state_image(&f, forged)).await;
    }
    let decline = request(
        &f,
        TableTransportChannel::Host,
        action(TacticalAction::DeclineOpportunity),
    )
    .await;
    Box::pin(f.cold(decline)).await;
    let done = f.state().await;
    let result = flow(&done).last_movement.as_ref().unwrap();
    assert_eq!(result.reason, TacticalMovementEnd::Completed);
    assert_eq!((result.completed_steps, result.spent_after), (1, 20));
    assert_eq!(result.endpoint, SpatialPoint { x: 0, y: 0, z: 0 });
    assert_eq!(
        result.transport.as_ref().unwrap().target_endpoint,
        SpatialPoint { x: 10, y: 0, z: 0 }
    );
    assert!(
        !done
            .rules
            .as_ref()
            .unwrap()
            .timing
            .as_ref()
            .unwrap()
            .reactions_spent
            .contains(&f.opponent.unwrap())
    );
    f.close().await;
}

#[tokio::test]
async fn ground_drag_four_cost_crawl_survives_final_result_and_original_replay() {
    let rubble = TerrainVolume {
        id: "rubble".into(),
        volume: SpatialBox {
            min: SpatialPoint { x: 0, y: 0, z: 0 },
            max: SpatialPoint { x: 80, y: 30, z: 3 },
        },
        difficult: true,
        observable: true,
        water: false,
        climbable: false,
        burrowable: false,
        supports_top: false,
        surface: None,
        obscuration: Obscuration::None,
        magical_darkness: false,
    };
    let mut f = Box::pin(ground_fixture(vec![rubble], vec![])).await;
    Box::pin(f.activate()).await;
    Box::pin(f.establish_pc_grip()).await;
    enable(&mut f).await;
    let movement = drag(
        &f,
        vec![TacticalMoveStep {
            destination: SpatialPoint { x: 20, y: 10, z: 0 },
            mode: MovementMode::Crawl,
        }],
    )
    .await;
    Box::pin(f.cold(movement)).await;
    let state = f.state().await;
    let result = flow(&state).last_movement.as_ref().unwrap();
    assert_eq!((result.completed_steps, result.spent_after), (1, 40));
    assert_eq!(result.reason, TacticalMovementEnd::Completed);
    assert_eq!(
        (
            result.transport.as_ref().unwrap().ordinary_cost,
            result.transport.as_ref().unwrap().haul_cost
        ),
        (30, 10)
    );
    f.close().await;
}
#[tokio::test]
async fn ground_drag_activation_is_host_only_settled_and_replayed_from_its_original_command() {
    let mut f = Box::pin(Fixture::new()).await;
    Box::pin(f.activate()).await;
    let unauthorized = request(
        &f,
        f.pc(0),
        TableTransportInput::Action(Box::new(TableAction::EnableGrappleTransport)),
    )
    .await;
    Box::pin(f.reject(unauthorized)).await;
    let attempt = f
        .choose(f.pc(0), "Grapple Small armored figure with left hand")
        .await;
    Box::pin(f.cold(attempt)).await;
    let unsettled = request(
        &f,
        TableTransportChannel::Host,
        TableTransportInput::Action(Box::new(TableAction::EnableGrappleTransport)),
    )
    .await;
    Box::pin(f.reject(unsettled)).await;
    let withdraw = f.choose(f.pc(0), "Withdraw this Grapple attempt").await;
    Box::pin(f.cold(withdraw)).await;
    enable(&mut f).await;
    let original = f.state().await;
    let mut forged = original.clone();
    forged
        .table
        .as_mut()
        .unwrap()
        .grapple_access
        .as_mut()
        .unwrap()
        .ground_transport
        .as_mut()
        .unwrap()
        .origin
        .id = CommandId::new();
    Box::pin(reject_state_image(&f, forged)).await;
    let mut forged = original;
    forged
        .table
        .as_mut()
        .unwrap()
        .grapple_access
        .as_mut()
        .unwrap()
        .ground_transport = None;
    Box::pin(reject_state_image(&f, forged)).await;
    f.close().await;
}
