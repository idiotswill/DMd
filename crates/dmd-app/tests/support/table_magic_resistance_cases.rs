//! Actual source gameplay negative controls. No positive MR claim is made here:
//! Hold Person cannot target a Fiend successfully, and Chimera breath is nonmagical.
use super::*;
use dmd_rules::tactical::TacticalAction;

fn runtime(pool: sqlx::SqlitePool) -> CampaignRuntime {
    CampaignRuntime::from_content_root(
        pool,
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../content"),
    )
}
async fn state(f: &Fixture) -> CampaignState {
    f.runtime
        .open_campaign(f.campaign)
        .await
        .unwrap()
        .state()
        .clone()
}
async fn view(f: &Fixture) -> TablePresentedView {
    f.runtime
        .presented_table_view(f.campaign, TableViewer::Host)
        .await
        .unwrap()
}
async fn destination_rows(pool: &sqlx::SqlitePool) -> std::collections::BTreeMap<String, i64> {
    let tables: Vec<String> =
        sqlx::query_scalar("SELECT name FROM sqlite_master WHERE type = 'table' ORDER BY name")
            .fetch_all(pool)
            .await
            .unwrap();
    let mut rows = std::collections::BTreeMap::new();
    for table in tables {
        let quoted = table.replace('"', "\"\"");
        let count = sqlx::query_scalar(&format!("SELECT COUNT(*) FROM \"{quoted}\""))
            .fetch_one(pool)
            .await
            .unwrap();
        rows.insert(table, count);
    }
    rows
}
fn tactical(action: TacticalAction) -> TableTransportInput {
    TableTransportInput::Action(Box::new(TableAction::Tactical { action }))
}
async fn submit(f: &Fixture, input: TableTransportInput) -> TableTransportRequest {
    let request = TableTransportRequest {
        version: TABLE_TRANSPORT_VERSION,
        command_id: CommandId::new(),
        campaign_id: f.campaign,
        session_id: Some(f.session),
        channel: TableTransportChannel::Host,
        revision: view(f).await.revision,
        input,
    };
    Box::pin(f.runtime.submit_presented_table(request.clone()))
        .await
        .unwrap();
    request
}
async fn raw(f: &Fixture, faces: &[u16]) -> TableTransportRequest {
    let request = view(f).await.roll.unwrap();
    assert_eq!(request.mode, RollMode::Normal);
    let sides: Vec<_> = request
        .dice
        .iter()
        .flat_map(|die| std::iter::repeat_n(die.sides, usize::from(die.count)))
        .collect();
    assert_eq!(sides.len(), faces.len());
    Box::pin(submit(
        f,
        tactical(TacticalAction::SubmitRoll {
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
        }),
    ))
    .await
}
async fn cold_retry(f: &mut Fixture, path: &Path, request: &TableTransportRequest) {
    let before = export_campaign(&f.pool, f.campaign).await.unwrap();
    let expected = state(f).await;
    let mirror_pool = open_sqlite("sqlite::memory:").await.unwrap();
    let mirror = runtime(mirror_pool.clone());
    Box::pin(mirror.restore_campaign(&before)).await.unwrap();
    assert_eq!(
        mirror.resume_campaign(f.campaign).await.unwrap().state(),
        &expected
    );
    let mirror_receipt = Box::pin(mirror.submit_presented_table(request.clone()))
        .await
        .unwrap();
    f.pool.close().await;
    f.pool = dmd_persistence::open_sqlite_path(path).await.unwrap();
    f.runtime = runtime(f.pool.clone());
    assert_eq!(
        f.runtime.resume_campaign(f.campaign).await.unwrap().state(),
        &expected
    );
    let receipt = Box::pin(f.runtime.submit_presented_table(request.clone()))
        .await
        .unwrap();
    assert_eq!(receipt, mirror_receipt);
    let mut after = export_campaign(&f.pool, f.campaign).await.unwrap();
    after.exported_at_utc = before.exported_at_utc.clone();
    assert_eq!(
        after, before,
        "cold accepted retry cannot issue another save or pay twice"
    );
    mirror_pool.close().await;
}
async fn prepare(f: &mut Fixture) -> [EntityId; 3] {
    let actors = [EntityId::new(), EntityId::new(), EntityId::new()];
    for (actor, id) in actors
        .into_iter()
        .zip(["cultist-fanatic", "chimera", "night-hag"])
    {
        let catalog = f
            .runtime
            .table_creature_options(TableCreatureOptionsRequest {
                campaign_id: f.campaign,
                channel: TableTransportChannel::Host,
                revision: view(f).await.revision,
            })
            .await
            .unwrap();
        let option = catalog
            .iter()
            .find(|entry| entry.definition_id == id)
            .unwrap();
        Box::pin(submit(
            f,
            TableTransportInput::Action(Box::new(TableAction::CreateCreature {
                creation: Box::new(TableCreatureCreation {
                    entity_id: actor,
                    name: format!("Private {id}"),
                    definition_id: id.into(),
                    source: option.source.clone(),
                    size: option.sizes[0],
                    additional_languages: vec![],
                    ammunition_units: 0,
                    item_ids: (0..option.item_count).map(|_| ItemId::new()).collect(),
                }),
            })),
        ))
        .await;
    }
    let count = view(f)
        .await
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
    let point = |x, y, z| SpatialPoint { x, y, z };
    f.host(
        TableAction::PrepareBattlefield {
            setup: Box::new(TableBattlefieldSetup {
                encounter_id: EncounterId::new(),
                scene_id: SceneId::new(),
                location_id: LocationId::new(),
                name: "Source save controls".into(),
                battlefield: Battlefield {
                    bounds: SpatialBox {
                        min: point(0, 0, 0),
                        max: point(160, 100, 60),
                    },
                    floor_z: 0,
                    floor_surface: "stone".into(),
                    ambient_light: LightLevel::Bright,
                    terrain: vec![],
                    obstacles: vec![],
                    lights: vec![],
                },
                characters: vec![TableCharacterPlacement {
                    character_id: f.characters[0],
                    position: point(10, 10, 0),
                    height: 12,
                    allies: vec![],
                    enemies: vec![],
                }],
                creatures: actors
                    .into_iter()
                    .zip([point(10, 40, 0), point(40, 40, 0), point(80, 40, 0)])
                    .enumerate()
                    .map(|(index, (actor, position))| TableCreaturePlacement {
                        actor,
                        public_label: format!("Figure {index}"),
                        position,
                        height: 12,
                        allies: vec![],
                        enemies: vec![],
                    })
                    .collect(),
                area_grid_policy: Some(TacticalAreaGridPolicy::OccupiedCellCentersV1),
                geometry_ruling: Ruling {
                    basis: RulingBasis::GmAdjudication,
                    reason: "Visible bodies on an open floor; explicit occupied-cell area policy."
                        .into(),
                },
            }),
        },
        Some(f.session),
    )
    .await;
    let mut combatants: Vec<_> = actors
        .into_iter()
        .zip(["cultist-fanatic", "chimera", "night-hag"])
        .map(|(actor, definition_id)| TacticalCombatant {
            actor,
            source: TacticalSource::Creature {
                definition_id: definition_id.into(),
            },
            surprised: false,
        })
        .collect();
    combatants.push(TacticalCombatant {
        actor: f.actors[0],
        source: TacticalSource::Character,
        surprised: false,
    });
    let groups = combatants
        .iter()
        .map(|c| InitiativeGroup {
            actors: vec![c.actor],
            request_id: RollRequestId::new(),
        })
        .collect();
    Box::pin(submit(
        f,
        tactical(TacticalAction::Begin {
            execution: TacticalExecutionVersion::ShieldMissileV1,
            combatants,
            groups,
        }),
    ))
    .await;
    for face in [20, 18, 6] {
        Box::pin(raw(f, &[face])).await;
    }
    let player_roll = f
        .runtime
        .table_view(f.campaign, TableViewer::Player(f.players[0]))
        .await
        .unwrap()
        .roll
        .unwrap();
    Box::pin(f.runtime.execute_table(
        f.player_meta(0).await,
        TableAction::Tactical {
            action: TacticalAction::SubmitRoll {
                result: RollResult {
                    request_id: player_roll.id,
                    source: RollSource::Physical,
                    dice: vec![DieResult {
                        sides: 20,
                        value: 1,
                    }],
                },
            },
        },
    ))
    .await
    .unwrap();
    assert_eq!(
        view(f).await.tactical.unwrap().active_actor,
        Some(actors[0])
    );
    actors
}

#[tokio::test]
async fn corrected_hag_has_real_fiend_no_effect_and_nonmagical_breath_controls_after_cold_restore()
{
    let path =
        std::env::temp_dir().join(format!("dmd-hag-categories-{}.sqlite", CampaignId::new().0));
    let pool = dmd_persistence::open_sqlite_path(&path).await.unwrap();
    let mut f = Fixture::with_pool(TableContract::default(), pool).await;
    let [cultist, chimera, hag] = Box::pin(prepare(&mut f)).await;
    let before = state(&f).await;
    let profile = before
        .rules
        .as_ref()
        .unwrap()
        .tactical_creatures
        .as_ref()
        .unwrap()
        .profile(hag)
        .unwrap();
    assert_eq!(
        dmd_rules::tactical_creatures::source_for_profile(profile).unwrap(),
        dmd_rules::tactical_definitions::bundled_night_hag_v2().unwrap()
    );
    // Same-ID statistics still agree, so only authentic creation history can
    // reject a silent revision swap. Keep every accepted origin/receipt intact.
    let original = export_campaign(&f.pool, f.campaign).await.unwrap();
    let mut forged = original.clone();
    let mut altered = before.clone();
    altered
        .rules
        .as_mut()
        .unwrap()
        .tactical_creatures
        .as_mut()
        .unwrap()
        .profiles
        .iter_mut()
        .find(|profile| profile.actor == hag)
        .unwrap()
        .source = dmd_rules::tactical_creatures::creature_source_pin(
        dmd_rules::tactical_creatures::creature_definition("night-hag").unwrap(),
    )
    .unwrap();
    forged.current_state.state_json = altered.encode_json().unwrap();
    // A fresh destination cannot mask failed semantic validation behind an
    // already-existing campaign. Check every table, including retained receipts.
    let destination_pool = open_sqlite("sqlite::memory:").await.unwrap();
    let destination = runtime(destination_pool.clone());
    let empty_rows = destination_rows(&destination_pool).await;
    assert!(
        Box::pin(destination.restore_campaign(&forged))
            .await
            .is_err()
    );
    assert_eq!(destination_rows(&destination_pool).await, empty_rows);
    assert_eq!(
        Box::pin(destination.restore_campaign(&original))
            .await
            .unwrap()
            .state(),
        &before,
        "the genuine export must still restore into the same rejected destination"
    );
    destination_pool.close().await;
    let mut unchanged = export_campaign(&f.pool, f.campaign).await.unwrap();
    unchanged.exported_at_utc = original.exported_at_utc.clone();
    assert_eq!(unchanged, original);
    let options = view(&f).await.tactical.unwrap().casting_options.unwrap();
    let hold = options
        .variants
        .iter()
        .find(|entry| entry.choice.spell_id == "hold-person")
        .unwrap();
    assert!(
        hold.targets.iter().any(|target| target.actor == hag),
        "private type does not filter a legal selection"
    );
    let paid = Box::pin(submit(
        &f,
        tactical(TacticalAction::CastSpell {
            choice: hold.choice.clone(),
            targets: SpellTargetChoice::Entities(vec![hag]),
        }),
    ))
    .await;
    let after = state(&f).await;
    let rules = after.rules.as_ref().unwrap();
    assert!(rules.timing.as_ref().unwrap().action_spent);
    let spent = |state: &CampaignState| {
        state
            .rules
            .as_ref()
            .unwrap()
            .tactical_creatures
            .as_ref()
            .unwrap()
            .runtime(cultist)
            .unwrap()
            .limited_uses
            .iter()
            .map(|entry| u32::from(entry.spent))
            .sum::<u32>()
    };
    assert_eq!(spent(&after), spent(&before) + 1);
    assert!(
        view(&f).await.roll.is_none(),
        "a Fiend creates no Hold Person save"
    );
    assert!(!dmd_rules::active_conditions(rules, hag).contains(&Condition::Paralyzed));
    assert!(rules.tactical_effects.as_ref().is_none_or(|effects| {
        !effects
            .effects
            .iter()
            .any(|effect| effect.target == TacticalEffectTarget::Creature(hag))
    }));
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
    Box::pin(cold_retry(&mut f, &path, &paid)).await;
    Box::pin(submit(&f, tactical(TacticalAction::EndTurn))).await;
    let projected = view(&f).await;
    let area = projected.tactical.unwrap().area_options.unwrap();
    assert_eq!(area.actor, chimera);
    assert_eq!(area.variants[0].feature_id, "fire-breath");
    let breath = Box::pin(submit(
        &f,
        tactical(TacticalAction::CreatureArea {
            feature_id: area.variants[0].feature_id.clone(),
            aim: TacticalAreaAim {
                origin: SpatialPoint { x: 60, y: 50, z: 6 },
                toward: SpatialPoint { x: 90, y: 50, z: 6 },
                include_origin: false,
            },
            ordering: TacticalAreaOrdering::Host,
        }),
    ))
    .await;
    Box::pin(cold_retry(&mut f, &path, &breath)).await;
    let amount = Box::pin(raw(&f, &[1; 7])).await;
    Box::pin(cold_retry(&mut f, &path, &amount)).await;
    let saving = view(&f).await.roll.unwrap();
    assert_eq!(saving.roller, Some(hag));
    assert_eq!(saving.reason, "Saving throw");
    assert_eq!(saving.modifier, 2);
    assert_eq!(
        saving.mode,
        RollMode::Normal,
        "nonmagical breath cannot acquire MR advantage"
    );
    assert_eq!(
        saving.dice,
        [DieSpec {
            count: 1,
            sides: 20
        }]
    );
    let player = f
        .runtime
        .table_view(f.campaign, TableViewer::Player(f.players[0]))
        .await
        .unwrap();
    assert!(player.roll.is_none());
    let private = serde_json::to_string(&player).unwrap();
    for secret in [
        "MagicResistance",
        "magic-resistance",
        "night-hag",
        "fire-breath",
        "source_type_matches",
    ] {
        assert!(!private.contains(secret), "{secret}");
    }
    let save = Box::pin(raw(&f, &[1])).await;
    Box::pin(cold_retry(&mut f, &path, &save)).await;
    assert!(
        state(&f)
            .await
            .encounter
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .resolution
            .is_none()
    );
    assert_eq!(
        state(&f).await.rules.as_ref().unwrap().entities[&hag].hp,
        109,
        "7 actual fire damage is resisted to 3"
    );
    f.pool.close().await;
    drop(f);
    sqlite_test_cleanup::remove_closed_file(&path)
        .await
        .unwrap();
}
