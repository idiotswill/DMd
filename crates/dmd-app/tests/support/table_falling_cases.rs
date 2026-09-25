use super::*;
use dmd_rules::tactical::TacticalAction;

fn point(x: i32, y: i32, z: i32) -> SpatialPoint {
    SpatialPoint { x, y, z }
}
fn action(action: TacticalAction) -> TableAction {
    TableAction::Tactical { action }
}
fn content() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../content")
}

async fn prepare_ledge(f: &mut Fixture) {
    let view = f
        .runtime
        .table_view(f.campaign, TableViewer::Host)
        .await
        .unwrap();
    let count = view
        .characters
        .iter()
        .find(|c| c.character_id == f.characters[0])
        .unwrap()
        .equipment
        .as_ref()
        .unwrap()
        .initial_item_count;
    f.host(
        TableAction::PrepareEquipment {
            character_id: f.characters[0],
            item_ids: (0..count).map(|_| ItemId::new()).collect(),
        },
        Some(f.session),
    )
    .await;
    f.host(
        TableAction::PrepareBattlefield {
            setup: Box::new(TableBattlefieldSetup {
                encounter_id: EncounterId::new(),
                scene_id: SceneId::new(),
                location_id: LocationId::new(),
                name: "A ledge above water".into(),
                battlefield: Battlefield {
                    bounds: SpatialBox {
                        min: point(0, 0, 0),
                        max: point(100, 100, 100),
                    },
                    floor_z: 0,
                    floor_surface: "stone".into(),
                    ambient_light: LightLevel::Bright,
                    obstacles: vec![SpatialObstacle {
                        id: "host-platform-identity".into(),
                        volume: SpatialBox {
                            min: point(0, 0, 0),
                            max: point(20, 30, 30),
                        },
                        blocks_movement: true,
                        blocks_sight: true,
                        observable: true,
                        cover: CoverDegree::Total,
                    }],
                    terrain: vec![TerrainVolume {
                        id: "private-water-identity".into(),
                        volume: SpatialBox {
                            min: point(20, 0, 0),
                            max: point(90, 90, 10),
                        },
                        difficult: false,
                        water: true,
                        climbable: false,
                        burrowable: false,
                        supports_top: false,
                        surface: None,
                        obscuration: Obscuration::None,
                        magical_darkness: false,
                        observable: false,
                    }],
                    lights: vec![],
                },
                characters: vec![TableCharacterPlacement {
                    character_id: f.characters[0],
                    position: point(10, 10, 30),
                    height: 12,
                    allies: vec![],
                    enemies: vec![],
                }],
                creatures: vec![],
                area_grid_policy: None,
                geometry_ruling: Ruling {
                    basis: RulingBasis::GmAdjudication,
                    reason: "An authored platform and liquid surface with first-contact landing."
                        .into(),
                },
            }),
        },
        Some(f.session),
    )
    .await;
    f.host(
        action(TacticalAction::Begin {
            execution: dmd_domain::TacticalExecutionVersion::ShieldHitV1,
            combatants: vec![TacticalCombatant {
                actor: f.actors[0],
                source: TacticalSource::Character,
                surprised: false,
            }],
            groups: vec![InitiativeGroup {
                actors: vec![f.actors[0]],
                request_id: RollRequestId::new(),
            }],
        }),
        Some(f.session),
    )
    .await;
    let request = request(f).await;
    f.runtime
        .execute_table(f.player_meta(0).await, raw(&request, &[15]))
        .await
        .unwrap();
}

async fn request(f: &Fixture) -> RollRequest {
    f.runtime
        .table_view(f.campaign, TableViewer::Player(f.players[0]))
        .await
        .unwrap()
        .roll
        .unwrap()
}
fn raw(request: &RollRequest, faces: &[u16]) -> TableAction {
    let sides = if request.mode == RollMode::Normal {
        request
            .dice
            .iter()
            .flat_map(|die| std::iter::repeat_n(die.sides, usize::from(die.count)))
            .collect::<Vec<_>>()
    } else {
        vec![20, 20]
    };
    assert_eq!(sides.len(), faces.len());
    action(TacticalAction::SubmitRoll {
        result: RollResult {
            request_id: request.id,
            source: RollSource::Physical,
            dice: sides
                .into_iter()
                .zip(faces)
                .map(|(sides, value)| DieResult {
                    sides,
                    value: *value,
                })
                .collect(),
        },
    })
}
async fn state(f: &Fixture) -> CampaignState {
    f.runtime
        .open_campaign(f.campaign)
        .await
        .unwrap()
        .state()
        .clone()
}
async fn reopen(f: &mut Fixture, path: &Path) {
    f.pool.close().await;
    let pool = dmd_persistence::open_sqlite_path(path).await.unwrap();
    f.runtime = CampaignRuntime::from_content_root(pool.clone(), content());
    f.pool = pool;
    f.runtime.resume_campaign(f.campaign).await.unwrap();
}
async fn execute_both(
    f: &Fixture,
    mirror: &CampaignRuntime,
    meta: CommandMeta,
    action: TableAction,
) {
    f.runtime
        .execute_table(meta.clone(), action.clone())
        .await
        .unwrap();
    mirror.execute_table(meta, action).await.unwrap();
    assert_eq!(
        mirror.open_campaign(f.campaign).await.unwrap().state(),
        &state(f).await
    );
}

async fn reject_tampered_fall_export(f: &Fixture) {
    let export = export_campaign(&f.pool, f.campaign).await.unwrap();
    for mutation in 0..4 {
        let mut bad = export.clone();
        let mut image = CampaignState::decode_json(&bad.current_state.state_json).unwrap();
        let resolution = image
            .encounter
            .as_mut()
            .unwrap()
            .flow
            .as_mut()
            .unwrap()
            .resolution
            .as_mut()
            .unwrap();
        match mutation {
            0 => resolution.falls[0].path.to.z += 10,
            1 => resolution.falls[0].origin.id = CommandId::new(),
            2 => match &mut resolution.falls[0].stage {
                TacticalFallStage::LandingCheck { accepted_by, .. } => {
                    accepted_by.id = CommandId::new()
                }
                TacticalFallStage::Damage {
                    landing: Some(landing),
                } => landing.accepted_by.id = CommandId::new(),
                _ => resolution.falls[0].actor = f.actors[1],
            },
            _ => {
                // A believable old image cannot be injected as a new historical anchor.
                resolution.falls[0].path.surface = FallSurface::Liquid {
                    id: "invented-water".into(),
                };
            }
        }
        let encoded = serde_json::to_string(&image).unwrap();
        if mutation == 3 {
            bad.snapshots
                .retain(|row| row.event_sequence != image.applied_event_sequence as i64);
            bad.snapshots.push(dmd_persistence::SnapshotRow {
                campaign_id: f.campaign.0.to_string(),
                event_sequence: image.applied_event_sequence as i64,
                state_schema_version: i64::from(image.schema_version),
                state_json: encoded,
                created_at_utc: bad.exported_at_utc.clone(),
            });
        } else {
            bad.current_state.state_json = encoded;
        }
        bad.upgraded().expect(
            "portable structure remains valid; tactical preflight must reject the forged meaning",
        );
        let pool = open_sqlite("sqlite::memory:").await.unwrap();
        let runtime = CampaignRuntime::from_content_root(pool.clone(), content());
        assert!(
            runtime.restore_campaign(&bad).await.is_err(),
            "mutation {mutation}"
        );
        assert!(runtime.open_campaign(f.campaign).await.is_err());
        for table in [
            "campaign_state_current",
            "campaign_lifecycle",
            "event_journal",
            "command_audit",
            "campaign_snapshots",
        ] {
            let rows: i64 = sqlx::query_scalar(&format!("SELECT COUNT(*) FROM {table}"))
                .fetch_one(&pool)
                .await
                .unwrap();
            assert_eq!(rows, 0, "partial restore wrote {table}");
        }
        pool.close().await;
    }
}

#[tokio::test]
async fn liquid_landing_choices_and_raw_dice_reopen_retry_restore_and_replay_through_sqlite() {
    for choice in [
        Some(LiquidLandingChoice::Athletics),
        Some(LiquidLandingChoice::Acrobatics),
        None,
    ] {
        // Keep this long, multi-reopen scenario off the default Windows test stack.
        // Each phase below has its own bounded future; the stack limit is unchanged.
        Box::pin(run_landing_case(choice)).await;
    }
}

async fn run_landing_case(choice: Option<LiquidLandingChoice>) {
    let path = std::env::temp_dir().join(format!("dmd-table-fall-{}.sqlite", CommandId::new().0));
    let pool = dmd_persistence::open_sqlite_path(&path).await.unwrap();
    let mut f = Box::pin(Fixture::with_pool(TableContract::default(), pool)).await;
    Box::pin(prepare_ledge(&mut f)).await;
    let (before_hp, move_meta, pending, view) = Box::pin(step_off_and_check_privacy(&f)).await;
    let choice_action = action(TacticalAction::ChooseLiquidLanding { choice });
    let export = export_campaign(&f.pool, f.campaign).await.unwrap();
    let mirror_pool = open_sqlite("sqlite::memory:").await.unwrap();
    let mirror = CampaignRuntime::from_content_root(mirror_pool.clone(), content());
    mirror.restore_campaign(&export).await.unwrap();
    reopen(&mut f, &path).await;
    assert_eq!(
        f.runtime
            .table_view(f.campaign, TableViewer::Player(f.players[0]))
            .await
            .unwrap(),
        view
    );
    assert_eq!(state(&f).await, pending);
    let choice_meta = f.player_meta(0).await;
    execute_both(&f, &mirror, choice_meta.clone(), choice_action.clone()).await;
    let after_choice = state(&f).await;
    assert_eq!(
        after_choice
            .rules
            .as_ref()
            .unwrap()
            .timing
            .as_ref()
            .unwrap()
            .reactions_spent
            .contains(&f.actors[0]),
        choice.is_some()
    );
    reopen(&mut f, &path).await;
    assert!(
        f.runtime
            .execute_table(choice_meta.clone(), choice_action.clone())
            .await
            .unwrap()
            .already_accepted
    );
    assert_eq!(state(&f).await, after_choice);
    let stale = CommandMeta {
        id: CommandId::new(),
        ..choice_meta.clone()
    };
    assert!(f.runtime.execute_table(stale, choice_action).await.is_err());
    Box::pin(reject_tampered_fall_export(&f)).await;

    if let Some(choice) = choice {
        let request = request(&f).await;
        assert_eq!(
            request.reason,
            match choice {
                LiquidLandingChoice::Athletics => "Strength (Athletics) liquid landing check",
                LiquidLandingChoice::Acrobatics => "Dexterity (Acrobatics) liquid landing check",
            }
        );
        assert_eq!(request.roller, Some(f.actors[0]));
        let view = f
            .runtime
            .table_view(f.campaign, TableViewer::Player(f.players[0]))
            .await
            .unwrap();
        assert!(view.tactical.as_ref().unwrap().liquid_landing.is_none());
        assert!(view.tactical.as_ref().unwrap().may_fail_save.is_none());
        assert!(
            f.runtime
                .execute_table(
                    f.player_meta(0).await,
                    action(TacticalAction::VoluntarilyFailSave)
                )
                .await
                .is_err()
        );
        let roll_meta = f.player_meta(0).await;
        let rolled = raw(&request, &[15]);
        execute_both(&f, &mirror, roll_meta.clone(), rolled.clone()).await;
        reopen(&mut f, &path).await;
        assert!(
            f.runtime
                .execute_table(roll_meta.clone(), rolled)
                .await
                .unwrap()
                .already_accepted
        );
        assert!(
            f.runtime
                .execute_table(roll_meta, raw(&request, &[1]))
                .await
                .is_err()
        );
        Box::pin(reject_tampered_fall_export(&f)).await;
    }
    Box::pin(finish_landing(
        &mut f,
        &mirror,
        &mirror_pool,
        &path,
        before_hp,
        move_meta,
        choice.is_some(),
    ))
    .await;
    mirror_pool.close().await;
    f.pool.close().await;
    drop(mirror);
    drop(mirror_pool);
    drop(f);
    sqlite_test_cleanup::remove_closed_file(&path)
        .await
        .unwrap();
}

async fn step_off_and_check_privacy(f: &Fixture) -> (u32, CommandMeta, CampaignState, TableView) {
    let before_hp = state(f).await.rules.as_ref().unwrap().entities[&f.actors[0]].hp;
    let move_meta = f.player_meta(0).await;
    f.runtime
        .execute_table(
            move_meta.clone(),
            action(TacticalAction::Move {
                path: vec![
                    TacticalMoveStep {
                        destination: point(20, 10, 30),
                        mode: MovementMode::Walk,
                    },
                    TacticalMoveStep {
                        destination: point(30, 10, 30),
                        mode: MovementMode::Walk,
                    },
                ],
            }),
        )
        .await
        .unwrap();
    let pending = state(f).await;
    let view = f
        .runtime
        .table_view(f.campaign, TableViewer::Player(f.players[0]))
        .await
        .unwrap();
    assert!(view.roll.is_none());
    assert_eq!(
        view.tactical.as_ref().unwrap().liquid_landing,
        Some(TableLiquidLandingView { actor: f.actors[0] })
    );
    let json = serde_json::to_string(&view).unwrap();
    assert!(!json.contains("private-water-identity"));
    assert!(!json.contains("host-platform-identity"));
    let other = f
        .runtime
        .table_view(f.campaign, TableViewer::Player(f.players[1]))
        .await
        .unwrap();
    assert!(other.tactical.unwrap().liquid_landing.is_none());
    assert!(other.roll.is_none());
    assert!(
        f.runtime
            .table_view(f.campaign, TableViewer::Host)
            .await
            .unwrap()
            .tactical
            .unwrap()
            .liquid_landing
            .is_some()
    );

    let choice_action = action(TacticalAction::ChooseLiquidLanding { choice: None });
    let wrong = f
        .meta(
            CommandIssuer::Player(f.players[1]),
            Some(f.actors[0]),
            Some(f.session),
        )
        .await;
    assert!(
        f.runtime
            .execute_table(wrong, choice_action.clone())
            .await
            .is_err()
    );
    assert_eq!(state(f).await, pending);
    Box::pin(reject_tampered_fall_export(f)).await;

    (before_hp, move_meta, pending, view)
}

async fn finish_landing(
    f: &mut Fixture,
    mirror: &CampaignRuntime,
    mirror_pool: &sqlx::SqlitePool,
    path: &Path,
    before_hp: u32,
    move_meta: CommandMeta,
    halved: bool,
) {
    let request = request(f).await;
    assert_eq!(request.reason, "Falling damage");
    assert_eq!(request.dice, vec![DieSpec { count: 1, sides: 6 }]);
    let damage_meta = f.player_meta(0).await;
    let rolled = raw(&request, &[6]);
    execute_both(f, mirror, damage_meta.clone(), rolled.clone()).await;
    let landed = state(f).await;
    let flow = landed.encounter.as_ref().unwrap().flow.as_ref().unwrap();
    assert!(flow.resolution.is_none());
    assert_eq!(
        landed
            .encounter
            .as_ref()
            .unwrap()
            .participant(f.actors[0])
            .unwrap()
            .position,
        point(20, 10, 10)
    );
    let receipt = flow.last_movement.as_ref().unwrap();
    assert_eq!(receipt.original, move_meta);
    assert_eq!(receipt.cause, damage_meta);
    assert_eq!(receipt.reason, TacticalMovementEnd::Fell);
    assert_eq!(
        (
            receipt.completed_steps,
            receipt.requested_steps,
            receipt.spent_after
        ),
        (1, 2, 10)
    );
    let entity = &landed.rules.as_ref().unwrap().entities[&f.actors[0]];
    assert!(entity.prone);
    assert_eq!(before_hp - entity.hp, if halved { 3 } else { 6 });
    reopen(f, path).await;
    assert!(
        f.runtime
            .execute_table(damage_meta, rolled)
            .await
            .unwrap()
            .already_accepted
    );
    assert_eq!(state(f).await, landed);
    assert_eq!(f.runtime.replay_rules(f.campaign).await.unwrap(), landed);
    let final_export = export_campaign(&f.pool, f.campaign).await.unwrap();
    let mirror_export = export_campaign(mirror_pool, f.campaign).await.unwrap();
    assert_eq!(
        final_export.event_journal.len(),
        mirror_export.event_journal.len()
    );
    for (actual, replayed) in final_export
        .event_journal
        .iter()
        .zip(mirror_export.event_journal)
    {
        assert_eq!(actual.sequence, replayed.sequence);
        assert_eq!(actual.command_id, replayed.command_id);
        assert_eq!(actual.payload_json, replayed.payload_json);
    }
    assert_eq!(final_export.command_audit, mirror_export.command_audit);
    let final_view = f
        .runtime
        .table_view(f.campaign, TableViewer::Player(f.players[0]))
        .await
        .unwrap();
    assert!(final_view.roll.is_none());
    assert!(final_view.tactical.unwrap().liquid_landing.is_none());
}
