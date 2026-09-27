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
async fn view(f: &Fixture, player: usize) -> TablePresentedView {
    f.runtime
        .presented_table_view(f.campaign, TableViewer::Player(f.players[player]))
        .await
        .unwrap()
}
fn action(action: TacticalAction) -> TableAction {
    TableAction::Tactical { action }
}
async fn direct(f: &Fixture, player: Option<usize>, request: TacticalAction) {
    let meta = match player {
        Some(index) => f.player_meta(index).await,
        None => f.meta(CommandIssuer::Admin, None, Some(f.session)).await,
    };
    Box::pin(f.runtime.execute_table(meta, action(request)))
        .await
        .unwrap();
}
async fn raw(f: &Fixture, player: Option<usize>, faces: &[u16]) {
    let viewer = player.map_or(TableViewer::Host, |i| TableViewer::Player(f.players[i]));
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
            .flat_map(|d| std::iter::repeat_n(d.sides, usize::from(d.count)))
            .collect::<Vec<_>>()
    } else {
        vec![20, 20]
    };
    assert_eq!(sides.len(), faces.len());
    Box::pin(direct(
        f,
        player,
        TacticalAction::SubmitRoll {
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
        },
    ))
    .await;
}
pub(super) async fn prepare(f: &mut Fixture, knockout: bool) {
    f.host(TableAction::EndSession, Some(f.session)).await;
    f.session = PlaySessionId::new();
    f.host(
        TableAction::StartSession {
            id: f.session,
            name: "First aid after a real attack".into(),
            participants: (0..2)
                .map(|i| SessionParticipant {
                    player_id: f.players[i],
                    character_id: Some(f.characters[i]),
                    attendance: AttendanceStatus::Present,
                })
                .collect(),
        },
        Some(f.session),
    )
    .await;
    for character in f.characters {
        let host = f
            .runtime
            .table_view(f.campaign, TableViewer::Host)
            .await
            .unwrap();
        let count = host
            .characters
            .iter()
            .find(|c| c.character_id == character)
            .unwrap()
            .equipment
            .as_ref()
            .unwrap()
            .initial_item_count;
        f.host(
            TableAction::PrepareEquipment {
                character_id: character,
                item_ids: (0..count).map(|_| ItemId::new()).collect(),
            },
            Some(f.session),
        )
        .await;
    }
    let npc = EntityId::new();
    let gear =
        dmd_rules::tactical_creature_equipment::creature_equipment_plan("goblin-warrior", 20)
            .unwrap();
    f.host(
        TableAction::CreateCreature {
            creation: Box::new(TableCreatureCreation {
                entity_id: npc,
                name: "Private sentry".into(),
                definition_id: "goblin-warrior".into(),
                size: CreatureSize::Small,
                additional_languages: vec![],
                ammunition_units: 20,
                item_ids: gear.iter().map(|_| ItemId::new()).collect(),
            }),
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
                name: "A bright clearing".into(),
                battlefield: Battlefield {
                    bounds: SpatialBox {
                        min: point(0, 0, 0),
                        max: point(100, 100, 40),
                    },
                    floor_z: 0,
                    floor_surface: "stone".into(),
                    ambient_light: LightLevel::Bright,
                    terrain: vec![],
                    obstacles: vec![],
                    lights: vec![],
                },
                characters: (0..2)
                    .map(|i| TableCharacterPlacement {
                        character_id: f.characters[i],
                        position: point(10 + i as i32 * 10, 10, 0),
                        height: 12,
                        allies: vec![f.actors[1 - i]],
                        enemies: vec![npc],
                    })
                    .collect(),
                creatures: vec![TableCreaturePlacement {
                    actor: npc,
                    public_label: "Armored sentry".into(),
                    position: point(30, 10, 0),
                    height: 8,
                    allies: vec![],
                    enemies: f.actors.to_vec(),
                }],
                area_grid_policy: None,
                geometry_ruling: Ruling {
                    basis: RulingBasis::GmAdjudication,
                    reason: "Clear adjacent occupied spaces.".into(),
                },
            }),
        },
        Some(f.session),
    )
    .await;
    view(f, 0).await; // Initialize immutable modern presentation before accepting new work.
    Box::pin(direct(
        f,
        None,
        TacticalAction::Begin {
            execution: TacticalExecutionVersion::ShieldHitV1,
            combatants: vec![
                TacticalCombatant {
                    actor: f.actors[0],
                    source: TacticalSource::Character,
                    surprised: false,
                },
                TacticalCombatant {
                    actor: f.actors[1],
                    source: TacticalSource::Character,
                    surprised: false,
                },
                TacticalCombatant {
                    actor: npc,
                    source: TacticalSource::Creature {
                        definition_id: "goblin-warrior".into(),
                    },
                    surprised: false,
                },
            ],
            groups: [f.actors[0], f.actors[1], npc]
                .into_iter()
                .map(|actor| InitiativeGroup {
                    actors: vec![actor],
                    request_id: RollRequestId::new(),
                })
                .collect(),
        },
    ))
    .await;
    Box::pin(raw(f, Some(0), &[18])).await;
    Box::pin(raw(f, Some(1), &[10])).await;
    Box::pin(raw(f, None, &[2])).await;
    Box::pin(direct(f, Some(0), TacticalAction::EndTurn)).await;
    Box::pin(direct(f, Some(1), TacticalAction::EndTurn)).await;
    let current = state(f).await;
    assert_eq!(
        current.rules.as_ref().unwrap().entities[&f.actors[1]].hp,
        12
    );
    let weapon = current
        .items
        .values()
        .find(|item| item.definition_id == "scimitar" && item.custody == Custody::Entity(npc))
        .unwrap()
        .id;
    Box::pin(direct(
        f,
        None,
        TacticalAction::CreatureWeaponAttack {
            feature_id: "scimitar".into(),
            choice: CreatureWeaponUseChoice {
                weapon,
                target: f.actors[1],
                grip: WeaponGrip::OneHand(Hand::Right),
                ammunition: None,
                equipment_change: Some(AttackEquipmentChange {
                    timing: EquipmentChangeTiming::BeforeAttack,
                    operation: AttackEquipmentOperation::Equip {
                        item: weapon,
                        hand: Hand::Right,
                    },
                }),
            },
        },
    ))
    .await;
    Box::pin(raw(f, None, &[20])).await;
    Box::pin(table_hit_driver::decline_hit_responses(f)).await;
    Box::pin(raw(f, None, &[5, 5])).await; // Genuine source critical2d6+2 =12; no HP patch.
    Box::pin(direct(
        f,
        None,
        TacticalAction::ChooseAttackKnockout {
            choice: if knockout {
                KnockoutChoice::KnockOut
            } else {
                KnockoutChoice::NormalDamage
            },
        },
    ))
    .await;
    Box::pin(direct(f, None, TacticalAction::EndTurn)).await;
    let current = state(f).await;
    let target = &current.rules.as_ref().unwrap().entities[&f.actors[1]];
    assert_eq!(target.hp, if knockout { 1 } else { 0 });
    assert!(!target.death.dead);
}
async fn request(f: &Fixture, player: usize, request: TacticalAction) -> TableTransportRequest {
    TableTransportRequest {
        version: TABLE_TRANSPORT_VERSION,
        command_id: CommandId::new(),
        campaign_id: f.campaign,
        session_id: Some(f.session),
        channel: TableTransportChannel::Player {
            player_id: f.players[player],
            character_id: f.characters[player],
        },
        revision: view(f, player).await.revision,
        input: TableTransportInput::Action(Box::new(action(request))),
    }
}
async fn cold_step(f: &mut Fixture, url: &str, request: TableTransportRequest) {
    let before = export_campaign(&f.pool, f.campaign).await.unwrap();
    let mirror_pool = open_sqlite("sqlite::memory:").await.unwrap();
    let mirror = runtime(mirror_pool.clone());
    Box::pin(mirror.restore_campaign(&before)).await.unwrap();
    let result = Box::pin(f.runtime.submit_presented_table(request.clone()))
        .await
        .unwrap();
    Box::pin(mirror.submit_presented_table(request.clone()))
        .await
        .unwrap();
    let current = state(f).await;
    assert_eq!(
        mirror.open_campaign(f.campaign).await.unwrap().state(),
        &current
    );
    mirror_pool.close().await;
    let saved = export_campaign(&f.pool, f.campaign).await.unwrap();
    f.pool.close().await;
    f.pool = open_sqlite(url).await.unwrap();
    f.runtime = runtime(f.pool.clone());
    assert_eq!(
        Box::pin(f.runtime.submit_presented_table(request.clone()))
            .await
            .unwrap(),
        result
    );
    let mut changed = request.clone();
    changed.input = TableTransportInput::Action(Box::new(action(TacticalAction::Dodge)));
    assert!(
        Box::pin(f.runtime.submit_presented_table(changed))
            .await
            .is_err()
    );
    let mut after = export_campaign(&f.pool, f.campaign).await.unwrap();
    after.exported_at_utc = saved.exported_at_utc.clone();
    assert_eq!(after, saved);
    assert_eq!(state(f).await, current);
}
async fn reject_forged_history(f: &Fixture) {
    let mut forged = state(f).await;
    forged
        .rules
        .as_mut()
        .unwrap()
        .timing
        .as_mut()
        .unwrap()
        .bonus_action_spent = true;
    dmd_rules::tactical::validate_tactical_state(&forged).unwrap();
    let mut export = export_campaign(&f.pool, f.campaign).await.unwrap();
    export.current_state.state_json = forged.encode_json().unwrap();
    let pool = open_sqlite("sqlite::memory:").await.unwrap();
    let restored = runtime(pool.clone());
    assert!(Box::pin(restored.restore_campaign(&export)).await.is_err());
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM campaign_state_current")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 0);
    pool.close().await;
}
async fn first_aid(f: &mut Fixture, url: &str, knockout: bool) {
    let target = f.actors[1];
    let aid = TacticalAction::FirstAid {
        target,
        purpose: if knockout {
            MedicinePurpose::EndKnockout
        } else {
            MedicinePurpose::Stabilize
        },
    };
    let foreign = request(f, 1, aid.clone()).await;
    let before = state(f).await;
    assert!(
        Box::pin(f.runtime.submit_presented_table(foreign))
            .await
            .is_err()
    );
    assert_eq!(state(f).await, before);
    let accepted = request(f, 0, aid).await;
    Box::pin(cold_step(f, url, accepted)).await;
    Box::pin(reject_forged_history(f)).await;
    let pending = view(f, 0).await.roll.unwrap();
    assert_eq!(pending.reason, "Wisdom (Medicine) first aid");
    assert_eq!(
        pending.dice,
        vec![DieSpec {
            count: 1,
            sides: 20
        }]
    );
    let canonical = state(f).await.rules.unwrap().pending.unwrap().request.id;
    assert_ne!(pending.id, canonical);
    let dice = request(
        f,
        0,
        TacticalAction::SubmitRoll {
            result: RollResult {
                request_id: pending.id,
                source: RollSource::Physical,
                dice: vec![DieResult {
                    sides: 20,
                    value: 10,
                }],
            },
        },
    )
    .await;
    Box::pin(cold_step(f, url, dice)).await;
    let current = state(f).await;
    let rules = current.rules.as_ref().unwrap();
    assert_eq!(rules.entities[&target].hp, if knockout { 1 } else { 0 });
    assert_eq!(rules.entities[&target].death.stable, !knockout);
    assert_eq!(
        rules.rolls.last().unwrap().result.dice,
        vec![DieResult {
            sides: 20,
            value: 10
        }]
    );
    assert!(rules.timing.as_ref().unwrap().action_spent);
    assert!(!rules.timing.as_ref().unwrap().bonus_action_spent);
    if !knockout {
        let pending = view(f, 1).await.roll.unwrap();
        assert_eq!(pending.dice, vec![DieSpec { count: 1, sides: 4 }]);
        let dice = request(
            f,
            1,
            TacticalAction::SubmitRoll {
                result: RollResult {
                    request_id: pending.id,
                    source: RollSource::Physical,
                    dice: vec![DieResult { sides: 4, value: 2 }],
                },
            },
        )
        .await;
        Box::pin(cold_step(f, url, dice)).await;
        let current = state(f).await;
        let recovery = current
            .rules
            .as_ref()
            .unwrap()
            .tactical_recovery
            .as_ref()
            .unwrap()[&target]
            .stable
            .as_ref()
            .unwrap();
        assert_eq!(
            dmd_rules::tactical_damage::stable_wake_at(recovery).unwrap(),
            Some(WorldInstant(7206))
        );
    } else {
        assert!(!dmd_rules::active_conditions(rules, target).contains(&Condition::Unconscious));
    }
}
#[tokio::test]
async fn first_aid_after_source_attack_survives_cold_owned_dice_retry_and_semantic_restore() {
    for knockout in [false, true] {
        let directory = std::env::temp_dir().join(format!("dmd-first-aid-{}", CampaignId::new().0));
        std::fs::create_dir_all(&directory).unwrap();
        let url = format!(
            "sqlite://{}",
            directory
                .join("campaign.sqlite")
                .to_string_lossy()
                .replace('\\', "/")
        );
        let pool = open_sqlite(&url).await.unwrap();
        let mut f = Box::pin(Fixture::with_pool(TableContract::default(), pool)).await;
        Box::pin(prepare(&mut f, knockout)).await;
        Box::pin(first_aid(&mut f, &url, knockout)).await;
        f.pool.close().await;
        drop(f);
        sqlite_test_cleanup::remove_closed_directory(&directory)
            .await
            .unwrap();
    }
}
