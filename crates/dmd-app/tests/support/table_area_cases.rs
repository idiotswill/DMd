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
async fn state(f: &Fixture) -> CampaignState {
    f.runtime
        .open_campaign(f.campaign)
        .await
        .unwrap()
        .state()
        .clone()
}
async fn host(f: &Fixture, action: TacticalAction) -> CommandMeta {
    let meta = f.meta(CommandIssuer::Admin, None, Some(f.session)).await;
    f.runtime
        .execute_table(meta.clone(), self::action(action))
        .await
        .unwrap();
    meta
}
async fn submit(f: &Fixture, player: bool, faces: &[u16]) -> CommandMeta {
    submit_both(f, player, faces, None).await.0
}
async fn execute_both(
    f: &Fixture,
    mirror: Option<&CampaignRuntime>,
    meta: &CommandMeta,
    action: &TableAction,
) {
    f.runtime
        .execute_table(meta.clone(), action.clone())
        .await
        .unwrap();
    if let Some(mirror) = mirror {
        mirror
            .execute_table(meta.clone(), action.clone())
            .await
            .unwrap();
        assert_eq!(
            mirror.open_campaign(f.campaign).await.unwrap().state(),
            &state(f).await
        );
    }
}
async fn submit_both(
    f: &Fixture,
    player: bool,
    faces: &[u16],
    mirror: Option<&CampaignRuntime>,
) -> (CommandMeta, TableAction) {
    let viewer = if player {
        TableViewer::Player(f.players[0])
    } else {
        TableViewer::Host
    };
    let request = f
        .runtime
        .table_view(f.campaign, viewer)
        .await
        .unwrap()
        .roll
        .unwrap();
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
    let meta = if player {
        f.player_meta(0).await
    } else {
        f.meta(CommandIssuer::Admin, None, Some(f.session)).await
    };
    let action = action(TacticalAction::SubmitRoll {
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
    });
    execute_both(f, mirror, &meta, &action).await;
    (meta, action)
}
async fn reopen(f: &mut Fixture, path: &Path) {
    f.pool.close().await;
    f.pool = dmd_persistence::open_sqlite_path(path).await.unwrap();
    f.runtime = CampaignRuntime::from_content_root(f.pool.clone(), content());
    f.runtime.resume_campaign(f.campaign).await.unwrap();
}

async fn prepare(f: &mut Fixture) -> [EntityId; 3] {
    let actors = [EntityId::new(), EntityId::new(), EntityId::new()];
    for (actor, id, size) in [
        (actors[0], "cultist-fanatic", CreatureSize::Medium),
        (actors[1], "chimera", CreatureSize::Large),
        (actors[2], "wolf", CreatureSize::Medium),
    ] {
        let items = dmd_rules::tactical_creature_equipment::creature_equipment_plan(id, 0).unwrap();
        f.host(
            TableAction::CreateCreature {
                creation: Box::new(TableCreatureCreation {
                    entity_id: actor,
                    name: format!("Private {id}"),
                    definition_id: id.into(),
                    size,
                    additional_languages: vec![],
                    ammunition_units: 0,
                    item_ids: items.iter().map(|_| ItemId::new()).collect(),
                }),
            },
            Some(f.session),
        )
        .await;
    }
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
                name: "A misty crossing".into(),
                battlefield: Battlefield {
                    bounds: SpatialBox {
                        min: point(0, 0, 0),
                        max: point(120, 100, 80),
                    },
                    floor_z: 0,
                    floor_surface: "stone".into(),
                    ambient_light: LightLevel::Bright,
                    obstacles: vec![],
                    lights: vec![],
                    terrain: vec![TerrainVolume {
                        id: "private-mist".into(),
                        volume: SpatialBox {
                            min: point(50, 40, 0),
                            max: point(70, 65, 20),
                        },
                        difficult: false,
                        water: false,
                        climbable: false,
                        burrowable: false,
                        supports_top: false,
                        surface: None,
                        obscuration: Obscuration::Heavy,
                        magical_darkness: false,
                        observable: false,
                    }],
                },
                characters: vec![TableCharacterPlacement {
                    character_id: f.characters[0],
                    position: point(40, 40, 0),
                    height: 12,
                    allies: vec![],
                    enemies: vec![],
                }],
                creatures: vec![
                    TableCreaturePlacement {
                        actor: actors[0],
                        public_label: "Robed traveler".into(),
                        position: point(40, 50, 0),
                        height: 12,
                        allies: vec![],
                        enemies: vec![],
                    },
                    TableCreaturePlacement {
                        actor: actors[1],
                        public_label: "Three-headed beast".into(),
                        position: point(10, 40, 0),
                        height: 12,
                        allies: vec![],
                        enemies: vec![],
                    },
                    TableCreaturePlacement {
                        actor: actors[2],
                        public_label: "Unseen wolf".into(),
                        position: point(50, 45, 0),
                        height: 10,
                        allies: vec![],
                        enemies: vec![],
                    },
                ],
                area_grid_policy: Some(TacticalAreaGridPolicy::OccupiedCellCentersV1),
                geometry_ruling: Ruling {
                    basis: RulingBasis::GmAdjudication,
                    reason: "Explicit occupied-space area policy and mist with clear effect paths."
                        .into(),
                },
            }),
        },
        Some(f.session),
    )
    .await;
    let combatants = vec![
        TacticalCombatant {
            actor: actors[0],
            source: TacticalSource::Creature {
                definition_id: "cultist-fanatic".into(),
            },
            surprised: false,
        },
        TacticalCombatant {
            actor: actors[1],
            source: TacticalSource::Creature {
                definition_id: "chimera".into(),
            },
            surprised: false,
        },
        TacticalCombatant {
            actor: f.actors[0],
            source: TacticalSource::Character,
            surprised: false,
        },
        TacticalCombatant {
            actor: actors[2],
            source: TacticalSource::Creature {
                definition_id: "wolf".into(),
            },
            surprised: false,
        },
    ];
    let groups = combatants
        .iter()
        .map(|c| InitiativeGroup {
            actors: vec![c.actor],
            request_id: RollRequestId::new(),
        })
        .collect();
    host(f, TacticalAction::Begin { combatants, groups }).await;
    submit(f, false, &[20]).await;
    submit(f, false, &[18]).await;
    submit(f, true, &[2]).await;
    submit(f, false, &[1]).await;
    actors
}

async fn concentrate(f: &Fixture, cultist: EntityId) {
    let options = f
        .runtime
        .table_view(f.campaign, TableViewer::Host)
        .await
        .unwrap()
        .tactical
        .unwrap()
        .casting_options
        .unwrap();
    let choice = options
        .variants
        .iter()
        .find(|v| v.choice.spell_id == "hold-person")
        .unwrap()
        .choice
        .clone();
    host(
        f,
        TacticalAction::CastSpell {
            choice,
            targets: SpellTargetChoice::Entities(vec![f.actors[0]]),
        },
    )
    .await;
    submit(f, true, &[20]).await;
    assert!(
        state(f).await.rules.as_ref().unwrap().entities[&cultist]
            .concentration
            .is_some()
    );
    host(f, TacticalAction::EndTurn).await;
}

async fn begin_area(
    f: &Fixture,
    chimera: EntityId,
    hidden: EntityId,
) -> (CommandMeta, TableAction) {
    let before = state(f).await;
    let options = f
        .runtime
        .table_view(f.campaign, TableViewer::Host)
        .await
        .unwrap()
        .tactical
        .unwrap()
        .area_options
        .unwrap();
    assert_eq!(options.actor, chimera);
    assert_eq!(options.controller, None);
    assert_eq!(options.variants.len(), 1);
    assert_eq!(options.variants[0].feature_id, "fire-breath");
    assert_eq!(
        state(f).await,
        before,
        "preview must not spend or recharge source resources"
    );
    let player = f
        .runtime
        .table_view(f.campaign, TableViewer::Player(f.players[0]))
        .await
        .unwrap();
    assert!(player.tactical.as_ref().unwrap().area_options.is_none());
    let private = serde_json::to_string(&player).unwrap();
    for secret in [
        hidden.0.to_string(),
        "Private wolf".into(),
        "Unseen wolf".into(),
        "fire-breath".into(),
        "private-mist".into(),
    ] {
        assert!(!private.contains(&secret), "{secret}");
    }
    let action = action(TacticalAction::CreatureArea {
        feature_id: options.variants[0].feature_id.clone(),
        aim: TacticalAreaAim {
            origin: point(30, 50, 6),
            toward: point(60, 50, 6),
            include_origin: false,
        },
        ordering: TacticalAreaOrdering::Host,
    });
    assert!(
        f.runtime
            .execute_table(f.player_meta(0).await, action.clone())
            .await
            .is_err()
    );
    assert_eq!(state(f).await, before);
    let meta = f.meta(CommandIssuer::Admin, None, Some(f.session)).await;
    f.runtime
        .execute_table(meta.clone(), action.clone())
        .await
        .unwrap();
    (meta, action)
}

async fn reject_forged_area(f: &Fixture) {
    let exported = export_campaign(&f.pool, f.campaign).await.unwrap();
    for mutation in 0..3 {
        let mut bad = exported.clone();
        let mut image = CampaignState::decode_json(&bad.current_state.state_json).unwrap();
        let record = &mut image
            .encounter
            .as_mut()
            .unwrap()
            .flow
            .as_mut()
            .unwrap()
            .resolution
            .as_mut()
            .unwrap()
            .areas[0];
        match mutation {
            0 => record.ordering = TacticalAreaOrdering::DelegateToHost,
            1 => record.source.invocation.id = CommandId::new(),
            _ => record.aim.toward.x += 1, // Same direction and membership, different accepted declaration.
        }
        assert!(image.validate().is_empty());
        bad.current_state.state_json = serde_json::to_string(&image).unwrap();
        bad.upgraded()
            .expect("portable structure remains valid before semantic area preflight");
        let pool = open_sqlite("sqlite::memory:").await.unwrap();
        let runtime = CampaignRuntime::from_content_root(pool.clone(), content());
        assert!(
            runtime.restore_campaign(&bad).await.is_err(),
            "mutation {mutation}"
        );
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

async fn finish_area(
    f: &mut Fixture,
    actors: [EntityId; 3],
    path: &Path,
    mirror: &CampaignRuntime,
) {
    let initial = state(f).await;
    submit_both(f, false, &[1; 7], Some(mirror)).await;
    let mut seen_save = false;
    let mut seen_concentration = false;
    for _ in 0..16 {
        let current = state(f).await;
        let Some(resolution) = current
            .encounter
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .resolution
            .as_ref()
        else {
            break;
        };
        let player = f
            .runtime
            .table_view(f.campaign, TableViewer::Player(f.players[0]))
            .await
            .unwrap();
        assert!(
            player.tactical.as_ref().unwrap().continuation.is_none(),
            "another actor's area never reveals its ordering work"
        );
        let private = serde_json::to_string(&player).unwrap();
        assert!(!private.contains(&actors[2].0.to_string()));
        assert!(!private.contains("Private wolf"));
        if let Some(pending) = &resolution.pending {
            let player_roll = pending.key.subject == f.actors[0];
            match pending.key.role {
                TacticalRollRole::AreaSave => {
                    assert_eq!(
                        current.rules.as_ref().unwrap().entities[&f.actors[0]].hp,
                        initial.rules.as_ref().unwrap().entities[&f.actors[0]].hp
                    );
                    assert_eq!(
                        current.rules.as_ref().unwrap().entities[&actors[0]].hp,
                        initial.rules.as_ref().unwrap().entities[&actors[0]].hp
                    );
                    if player_roll {
                        seen_save = true;
                        assert_eq!(player.roll.as_ref().unwrap().reason, "Saving throw");
                        assert_eq!(
                            player.tactical.as_ref().unwrap().may_fail_save,
                            Some(f.actors[0])
                        );
                        let other = f
                            .runtime
                            .table_view(f.campaign, TableViewer::Player(f.players[1]))
                            .await
                            .unwrap();
                        assert!(other.roll.is_none());
                        assert!(other.tactical.unwrap().may_fail_save.is_none());
                        reopen(f, path).await;
                        assert_eq!(
                            state(f).await,
                            current,
                            "pending player save survives disk reopen"
                        );
                        assert_eq!(
                            f.runtime
                                .table_view(f.campaign, TableViewer::Player(f.players[0]))
                                .await
                                .unwrap(),
                            player
                        );
                    } else {
                        assert!(player.roll.is_none());
                    }
                    let (meta, action) = submit_both(f, player_roll, &[1], Some(mirror)).await;
                    if player_roll {
                        let after = state(f).await;
                        reopen(f, path).await;
                        assert!(
                            f.runtime
                                .execute_table(meta, action)
                                .await
                                .unwrap()
                                .already_accepted
                        );
                        assert_eq!(
                            state(f).await,
                            after,
                            "accepted save retry must not apply another victim's work twice"
                        );
                    }
                }
                TacticalRollRole::Concentration => {
                    seen_concentration = true;
                    assert_eq!(pending.key.subject, actors[0]);
                    assert!(player.roll.is_none());
                    reopen(f, path).await;
                    assert_eq!(
                        state(f).await,
                        current,
                        "area concentration child survives disk reopen"
                    );
                    submit_both(f, false, &[20], Some(mirror)).await;
                }
                _ => panic!("unexpected area child: {:?}", pending.key.role),
            }
        } else {
            let view = f
                .runtime
                .table_view(f.campaign, TableViewer::Host)
                .await
                .unwrap();
            let continuation = view.tactical.unwrap().continuation.unwrap();
            assert!(continuation.host_adjudication);
            let choice = continuation.choices.first().unwrap();
            assert!(choice.label.contains(':'));
            let action = TacticalAction::ChooseTurnWork {
                occurrence: choice.occurrence,
            };
            assert!(
                f.runtime
                    .execute_table(f.player_meta(0).await, self::action(action.clone()))
                    .await
                    .is_err()
            );
            let meta = f.meta(CommandIssuer::Admin, None, Some(f.session)).await;
            execute_both(f, Some(mirror), &meta, &self::action(action)).await;
        }
    }
    assert!(seen_save && seen_concentration);
    let final_state = state(f).await;
    assert!(
        final_state
            .encounter
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .resolution
            .is_none()
    );
    for actor in [f.actors[0], actors[0], actors[2]] {
        assert_eq!(
            initial.rules.as_ref().unwrap().entities[&actor].hp
                - final_state.rules.as_ref().unwrap().entities[&actor].hp,
            7
        );
    }
    assert!(
        final_state.rules.as_ref().unwrap().entities[&actors[0]]
            .concentration
            .is_some()
    );
    assert_eq!(
        f.runtime.replay_rules(f.campaign).await.unwrap(),
        final_state
    );
    assert_eq!(mirror.replay_rules(f.campaign).await.unwrap(), final_state);
}

#[tokio::test]
async fn source_area_private_ordering_saves_concentration_and_cold_retry_use_sqlite() {
    let path = std::env::temp_dir().join(format!("dmd-table-area-{}.sqlite", CommandId::new().0));
    let pool = dmd_persistence::open_sqlite_path(&path).await.unwrap();
    let mut f = Fixture::with_pool(TableContract::default(), pool).await;
    let actors = prepare(&mut f).await;
    concentrate(&f, actors[0]).await;
    let (meta, action) = begin_area(&f, actors[1], actors[2]).await;
    let pending = state(&f).await;
    reject_forged_area(&f).await;
    let export = export_campaign(&f.pool, f.campaign).await.unwrap();
    let mirror_pool = open_sqlite("sqlite::memory:").await.unwrap();
    let mirror = CampaignRuntime::from_content_root(mirror_pool.clone(), content());
    mirror.restore_campaign(&export).await.unwrap();
    assert_eq!(
        mirror.resume_campaign(f.campaign).await.unwrap().state(),
        &pending
    );
    reopen(&mut f, &path).await;
    assert_eq!(state(&f).await, pending);
    assert!(
        f.runtime
            .execute_table(meta.clone(), action.clone())
            .await
            .unwrap()
            .already_accepted
    );
    assert_eq!(
        state(&f).await,
        pending,
        "uncertain accepted retry must not charge twice"
    );
    finish_area(&mut f, actors, &path, &mirror).await;
    let final_state = state(&f).await;
    reopen(&mut f, &path).await;
    assert!(
        f.runtime
            .execute_table(meta, action)
            .await
            .unwrap()
            .already_accepted
    );
    assert_eq!(state(&f).await, final_state);
    assert_eq!(
        f.runtime.replay_rules(f.campaign).await.unwrap(),
        final_state
    );
    mirror_pool.close().await;
    f.pool.close().await;
    std::fs::remove_file(path).unwrap();
}
