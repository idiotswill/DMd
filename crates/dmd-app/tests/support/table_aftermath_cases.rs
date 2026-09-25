use super::*;
use dmd_rules::tactical::TacticalAction;

fn runtime(pool: sqlx::SqlitePool) -> CampaignRuntime {
    CampaignRuntime::from_content_root(
        pool,
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../content"),
    )
}
fn action(action: TacticalAction) -> TableAction {
    TableAction::Tactical { action }
}
fn conclusion() -> TableAction {
    action(TacticalAction::ConcludeHostilities {
        cadence: AftermathCadence::ContinueExistingOrder,
        ruling: "Private GM choice: preserve ongoing saves and duration boundaries.".into(),
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
async fn view(f: &Fixture, player: Option<usize>) -> TablePresentedView {
    f.runtime
        .presented_table_view(
            f.campaign,
            player.map_or(TableViewer::Host, |i| TableViewer::Player(f.players[i])),
        )
        .await
        .unwrap()
}
async fn request(f: &Fixture, player: Option<usize>, action: TableAction) -> TableTransportRequest {
    TableTransportRequest {
        version: TABLE_TRANSPORT_VERSION,
        command_id: CommandId::new(),
        campaign_id: f.campaign,
        session_id: Some(f.session),
        revision: view(f, player).await.revision,
        channel: player.map_or(TableTransportChannel::Host, |i| {
            TableTransportChannel::Player {
                player_id: f.players[i],
                character_id: f.characters[i],
            }
        }),
        input: TableTransportInput::Action(Box::new(action)),
    }
}
async fn direct(f: &Fixture, player: Option<usize>, action: TacticalAction) {
    let meta = match player {
        Some(i) => f.player_meta(i).await,
        None => f.meta(CommandIssuer::Admin, None, Some(f.session)).await,
    };
    Box::pin(f.runtime.execute_table(meta, self::action(action)))
        .await
        .unwrap();
}
async fn raw_action(f: &Fixture, player: Option<usize>, faces: &[u16]) -> TableAction {
    let roll = view(f, player).await.roll.unwrap();
    let sides = if roll.mode == RollMode::Normal {
        roll.dice
            .iter()
            .flat_map(|d| std::iter::repeat_n(d.sides, usize::from(d.count)))
            .collect::<Vec<_>>()
    } else {
        vec![20, 20]
    };
    assert_eq!(sides.len(), faces.len());
    action(TacticalAction::SubmitRoll {
        result: RollResult {
            request_id: roll.id,
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
async fn setup_raw(f: &Fixture, player: Option<usize>, faces: &[u16]) {
    let action = raw_action(f, player, faces).await;
    let request = request(f, player, action).await;
    Box::pin(f.runtime.submit_presented_table(request))
        .await
        .unwrap();
}
async fn reopen(f: &mut Fixture, url: &str) {
    f.pool.close().await;
    f.pool = open_sqlite(url).await.unwrap();
    f.runtime = runtime(f.pool.clone());
    Box::pin(f.runtime.resume_campaign(f.campaign))
        .await
        .unwrap();
}

/// Each step starts from an independently semantically restored portable save.
/// The accepted original envelope is retried after closing the real file pool.
async fn cold_step(f: &mut Fixture, url: &str, request: TableTransportRequest) {
    let before = export_campaign(&f.pool, f.campaign).await.unwrap();
    let mirror_pool = open_sqlite("sqlite::memory:").await.unwrap();
    let mirror = runtime(mirror_pool.clone());
    Box::pin(mirror.restore_campaign(&before)).await.unwrap();
    let result = Box::pin(f.runtime.submit_presented_table(request.clone()))
        .await
        .unwrap();
    let mirrored = Box::pin(mirror.submit_presented_table(request.clone()))
        .await
        .unwrap();
    match (&result, &mirrored) {
        (TableTransportResult::Accepted(left), TableTransportResult::Accepted(right)) => {
            assert_eq!(left.command_id, right.command_id);
            assert_eq!(left.outcome, right.outcome);
        }
        _ => panic!("an action must be accepted as an action in both databases"),
    }
    // New audience revisions are intentionally random in independent databases;
    // accepted retries within each database must retain that database's response.
    assert_eq!(
        Box::pin(mirror.submit_presented_table(request.clone()))
            .await
            .unwrap(),
        mirrored
    );
    let expected = state(f).await;
    assert_eq!(
        mirror.open_campaign(f.campaign).await.unwrap().state(),
        &expected
    );
    mirror_pool.close().await;
    let saved = export_campaign(&f.pool, f.campaign).await.unwrap();
    Box::pin(reopen(f, url)).await;
    assert_eq!(
        Box::pin(f.runtime.submit_presented_table(request.clone()))
            .await
            .unwrap(),
        result
    );
    let mut changed = request;
    changed.input = TableTransportInput::Action(Box::new(action(TacticalAction::Dodge)));
    assert!(
        Box::pin(f.runtime.submit_presented_table(changed))
            .await
            .is_err()
    );
    let mut retried = export_campaign(&f.pool, f.campaign).await.unwrap();
    retried.exported_at_utc = saved.exported_at_utc.clone();
    assert_eq!(retried, saved);
    assert_eq!(state(f).await, expected);
}
async fn cold_action(f: &mut Fixture, url: &str, player: Option<usize>, action: TableAction) {
    let request = request(f, player, action).await;
    Box::pin(cold_step(f, url, request)).await;
}
async fn cold_raw(f: &mut Fixture, url: &str, player: Option<usize>, faces: &[u16]) {
    let action = raw_action(f, player, faces).await;
    Box::pin(cold_action(f, url, player, action)).await;
}
async fn rejected(f: &Fixture, player: Option<usize>, action: TableAction) {
    let request = request(f, player, action).await;
    let before = export_campaign(&f.pool, f.campaign).await.unwrap();
    assert!(
        Box::pin(f.runtime.submit_presented_table(request))
            .await
            .is_err()
    );
    let mut after = export_campaign(&f.pool, f.campaign).await.unwrap();
    after.exported_at_utc = before.exported_at_utc.clone();
    assert_eq!(after, before, "rejected command changes no durable row");
}
fn participants(f: &Fixture) -> Vec<SessionParticipant> {
    (0..2)
        .map(|i| SessionParticipant {
            player_id: f.players[i],
            character_id: Some(f.characters[i]),
            attendance: AttendanceStatus::Present,
        })
        .collect()
}
async fn session_rollover(f: &mut Fixture, url: &str) {
    let prior = state(f).await;
    Box::pin(cold_action(f, url, None, TableAction::EndSession)).await;
    assert!(view(f, None).await.active_session.is_none());
    // Neither quitting nor opening a different real session advances game time.
    f.session = PlaySessionId::new();
    let mut absent = participants(f);
    absent[1].attendance = AttendanceStatus::Absent;
    Box::pin(rejected(
        f,
        None,
        TableAction::StartSession {
            id: f.session,
            name: "Missing retained controller".into(),
            participants: absent,
        },
    ))
    .await;
    let mut unbound = participants(f);
    unbound[1].character_id = None;
    Box::pin(rejected(
        f,
        None,
        TableAction::StartSession {
            id: f.session,
            name: "Missing retained character binding".into(),
            participants: unbound,
        },
    ))
    .await;
    let start = TableAction::StartSession {
        id: f.session,
        name: "Aftermath resumed".into(),
        participants: participants(f),
    };
    Box::pin(cold_action(f, url, None, start)).await;
    let after = state(f).await;
    assert_eq!(after.rules, prior.rules);
    assert_eq!(after.encounter, prior.encounter);
    assert_eq!(after.clock, prior.clock);
    assert_eq!(after.items, prior.items);
    assert_eq!(after.characters, prior.characters);
}
async fn conclude(f: &mut Fixture, url: &str) {
    Box::pin(rejected(f, None, TableAction::EndSession)).await;
    Box::pin(rejected(f, Some(0), conclusion())).await;
    let before = state(f).await;
    Box::pin(cold_action(f, url, None, conclusion())).await;
    let mut after = state(f).await;
    let marker = after
        .encounter
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .aftermath
        .take()
        .unwrap();
    assert_eq!(marker.concluded_at, before.clock.now);
    assert_eq!(after.encounter, before.encounter);
    assert_eq!(after.rules, before.rules);
    assert_eq!(after.clock, before.clock);
    assert_eq!(after.items, before.items);
    assert_eq!(after.characters, before.characters);
    assert!(
        view(f, None)
            .await
            .tactical
            .unwrap()
            .aftermath
            .unwrap()
            .may_pause_session
    );
    for i in 0..2 {
        let presented = view(f, Some(i)).await;
        let aftermath = presented
            .tactical
            .as_ref()
            .unwrap()
            .aftermath
            .as_ref()
            .unwrap();
        assert!(aftermath.host_ruling.is_none());
        assert!(!aftermath.may_pause_session);
        assert!(
            !serde_json::to_string(&presented)
                .unwrap()
                .contains("Private GM choice")
        );
    }
    Box::pin(rejected(f, None, conclusion())).await;
    Box::pin(reject_forged_conclusion(f)).await;
}
async fn reject_forged_conclusion(f: &Fixture) {
    let original = export_campaign(&f.pool, f.campaign).await.unwrap();
    for kind in 0..5 {
        let mut export = original.clone();
        let mut image = CampaignState::decode_json(&export.current_state.state_json).unwrap();
        let flow = image.encounter.as_mut().unwrap().flow.as_mut().unwrap();
        match kind {
            0 => flow.aftermath.as_mut().unwrap().origin.id = CommandId::new(),
            1 => flow.aftermath.as_mut().unwrap().ruling = "Invented accepted ruling".into(),
            2 => flow.aftermath = None,
            3 => flow.aftermath.as_mut().unwrap().concluded_on_turn.actor = f.actors[0],
            4 => {}
            _ => unreachable!(),
        }
        export.current_state.state_json = serde_json::to_string(&image).unwrap();
        if kind == 4 {
            // A believable post-conclusion image cannot replace the original
            // pre-tactical anchor and authorize its own retained cadence.
            export.snapshots = vec![dmd_persistence::SnapshotRow {
                campaign_id: f.campaign.0.to_string(),
                event_sequence: image.applied_event_sequence as i64,
                state_schema_version: i64::from(image.schema_version),
                state_json: export.current_state.state_json.clone(),
                created_at_utc: export.exported_at_utc.clone(),
            }];
        }
        assert_ne!(export, original, "case {kind} must alter the export");
        export
            .upgraded()
            .expect("generic export structure remains valid");
        let pool = open_sqlite("sqlite::memory:").await.unwrap();
        assert!(
            Box::pin(runtime(pool.clone()).restore_campaign(&export))
                .await
                .is_err(),
            "case {kind}"
        );
        for table in [
            "campaign_state_current",
            "campaign_lifecycle",
            "command_audit",
            "event_journal",
            "campaign_snapshots",
        ] {
            let count: i64 = sqlx::query_scalar(&format!("SELECT COUNT(*) FROM {table}"))
                .fetch_one(&pool)
                .await
                .unwrap();
            assert_eq!(count, 0, "case {kind}: {table}");
        }
        pool.close().await;
    }
}

async fn prepare_magic(f: &mut Fixture) -> (EntityId, EntityId) {
    f.host(TableAction::EndSession, Some(f.session)).await;
    f.session = PlaySessionId::new();
    f.host(
        TableAction::StartSession {
            id: f.session,
            name: "Both controllers present".into(),
            participants: participants(f),
        },
        Some(f.session),
    )
    .await;
    for character_id in f.characters {
        let count = f
            .runtime
            .table_view(f.campaign, TableViewer::Host)
            .await
            .unwrap()
            .characters
            .iter()
            .find(|c| c.character_id == character_id)
            .unwrap()
            .equipment
            .as_ref()
            .unwrap()
            .initial_item_count;
        f.host(
            TableAction::PrepareEquipment {
                character_id,
                item_ids: (0..count).map(|_| ItemId::new()).collect(),
            },
            Some(f.session),
        )
        .await;
    }
    let mage = EntityId::new();
    let cultist = EntityId::new();
    for (actor, id, languages) in [
        (
            mage,
            "mage",
            vec!["dwarvish".into(), "elvish".into(), "draconic".into()],
        ),
        (cultist, "cultist-fanatic", vec![]),
    ] {
        let host = view(f, None).await;
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
            .find(|source| source.definition_id == id)
            .unwrap();
        let creation = TableAction::CreateCreature {
            creation: Box::new(TableCreatureCreation {
                entity_id: actor,
                name: format!("Private {id}"),
                definition_id: source.definition_id.clone(),
                size: source.sizes[0],
                additional_languages: languages,
                ammunition_units: 0,
                item_ids: (0..source.item_count).map(|_| ItemId::new()).collect(),
            }),
        };
        let request = request(f, None, creation).await;
        Box::pin(f.runtime.submit_presented_table(request))
            .await
            .unwrap();
    }
    let point = |x, y, z| SpatialPoint { x, y, z };
    f.host(
        TableAction::PrepareBattlefield {
            setup: Box::new(TableBattlefieldSetup {
                encounter_id: EncounterId::new(),
                scene_id: SceneId::new(),
                location_id: LocationId::new(),
                name: "Open court".into(),
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
                        position: point(10 + i as i32 * 20, 10, 0),
                        height: 12,
                        allies: vec![],
                        enemies: vec![mage, cultist],
                    })
                    .collect(),
                creatures: [(mage, "Spellcaster", 10), (cultist, "Robed figure", 30)]
                    .into_iter()
                    .map(|(actor, label, x)| TableCreaturePlacement {
                        actor,
                        public_label: label.into(),
                        position: point(x, 40, 0),
                        height: 12,
                        allies: vec![],
                        enemies: f.actors.to_vec(),
                    })
                    .collect(),
                area_grid_policy: None,
                geometry_ruling: Ruling {
                    basis: RulingBasis::GmAdjudication,
                    reason: "Explicit open ground and source creature placement.".into(),
                },
            }),
        },
        Some(f.session),
    )
    .await;
    let ordered = [mage, f.actors[0], cultist, f.actors[1]];
    Box::pin(direct(
        f,
        None,
        TacticalAction::Begin {
            execution: TacticalExecutionVersion::ReactionsV1,
            combatants: ordered
                .into_iter()
                .map(|actor| TacticalCombatant {
                    actor,
                    surprised: false,
                    source: if actor == mage {
                        TacticalSource::Creature {
                            definition_id: "mage".into(),
                        }
                    } else if actor == cultist {
                        TacticalSource::Creature {
                            definition_id: "cultist-fanatic".into(),
                        }
                    } else {
                        TacticalSource::Character
                    },
                })
                .collect(),
            groups: ordered
                .into_iter()
                .map(|actor| InitiativeGroup {
                    actors: vec![actor],
                    request_id: RollRequestId::new(),
                })
                .collect(),
        },
    ))
    .await;
    for (player, face) in [(None, 20), (Some(0), 18), (None, 10), (Some(1), 2)] {
        Box::pin(setup_raw(f, player, &[face])).await;
    }
    (mage, cultist)
}
async fn cast(f: &mut Fixture, url: &str, actor: EntityId, spell: &str, target: EntityId) {
    let options = view(f, None)
        .await
        .tactical
        .unwrap()
        .casting_options
        .unwrap();
    assert_eq!(options.actor, actor);
    let choice = options
        .variants
        .into_iter()
        .find(|v| v.choice.spell_id == spell)
        .unwrap()
        .choice;
    assert!(matches!(
        choice.grant,
        SpellGrantChoice::CreatureFeature { .. }
    ));
    assert_eq!(choice.resource, SpellResourceChoice::SourceFeature);
    Box::pin(cold_action(
        f,
        url,
        None,
        action(TacticalAction::CastSpell {
            choice,
            targets: SpellTargetChoice::Entities(vec![target]),
        }),
    ))
    .await;
}
async fn magic_scenario(f: &mut Fixture, url: &str) {
    let (mage, cultist) = Box::pin(prepare_magic(f)).await;
    assert_eq!(
        dmd_rules::tactical_defenses::effective_armor_class(&state(f).await, mage).unwrap(),
        12
    );
    Box::pin(cast(f, url, mage, "mage-armor", mage)).await;
    let armored = state(f).await;
    assert_eq!(
        dmd_rules::tactical_defenses::effective_armor_class(&armored, mage).unwrap(),
        15
    );
    let defense = armored
        .rules
        .as_ref()
        .unwrap()
        .tactical_effects
        .as_ref()
        .unwrap()
        .effects
        .iter()
        .find(|effect| !effect.defenses.is_empty())
        .unwrap()
        .clone();
    Box::pin(cold_action(f, url, None, action(TacticalAction::EndTurn))).await;
    Box::pin(cold_action(
        f,
        url,
        Some(0),
        action(TacticalAction::Ready {
            trigger: ReadyTrigger::MovementFinished {
                subject: ReadySubject::AnyOther,
            },
            action: ReadyAction::Move,
        }),
    ))
    .await;
    Box::pin(cold_action(
        f,
        url,
        Some(0),
        action(TacticalAction::EndTurn),
    ))
    .await;
    let target = f.actors[1];
    Box::pin(cast(f, url, cultist, "hold-person", target)).await;
    Box::pin(rejected(f, None, conclusion())).await; // Real selected player save.
    Box::pin(cold_raw(f, url, Some(1), &[1])).await;
    Box::pin(cold_action(f, url, None, action(TacticalAction::EndTurn))).await;
    assert!(
        dmd_rules::active_conditions(state(f).await.rules.as_ref().unwrap(), f.actors[1])
            .contains(&Condition::Paralyzed)
    );
    Box::pin(conclude(f, url)).await;
    Box::pin(session_rollover(f, url)).await;
    Box::pin(cold_action(
        f,
        url,
        Some(1),
        action(TacticalAction::EndTurn),
    ))
    .await;
    assert!(view(f, Some(1)).await.roll.is_some());
    Box::pin(rejected(f, None, TableAction::EndSession)).await;
    Box::pin(cold_raw(f, url, Some(1), &[20])).await;
    let current = state(f).await;
    assert!(
        !dmd_rules::active_conditions(current.rules.as_ref().unwrap(), f.actors[1])
            .contains(&Condition::Paralyzed)
    );
    assert_eq!(
        current.rules.as_ref().unwrap().entities[&cultist].concentration,
        None
    );
    assert_eq!(
        current
            .encounter
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .ready
            .len(),
        1
    );
    assert_eq!(
        current
            .rules
            .as_ref()
            .unwrap()
            .tactical_effects
            .as_ref()
            .unwrap()
            .effects
            .iter()
            .find(|effect| effect.id == defense.id),
        Some(&defense)
    );
    Box::pin(cold_action(f, url, None, action(TacticalAction::EndTurn))).await;
    let current = state(f).await;
    assert!(
        current
            .encounter
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .ready
            .is_empty()
    );
    assert_eq!(
        dmd_rules::tactical_defenses::effective_armor_class(&current, mage).unwrap(),
        15
    );
}

async fn dying_scenario(f: &mut Fixture, url: &str) {
    Box::pin(table_medicine_cases::prepare(f, false)).await;
    // The source Goblin's real critical hit reduced this genuine PC to zero.
    assert_eq!(
        state(f).await.rules.as_ref().unwrap().entities[&f.actors[1]].hp,
        0
    );
    Box::pin(cold_action(
        f,
        url,
        Some(0),
        action(TacticalAction::Ready {
            trigger: ReadyTrigger::MovementFinished {
                subject: ReadySubject::AnyOther,
            },
            action: ReadyAction::Move,
        }),
    ))
    .await;
    // Use the same conclusion assertions except the alternate actor corruption:
    // here the current actor is PC0, so corrupt to PC1 instead.
    Box::pin(cold_action(f, url, None, conclusion())).await;
    Box::pin(session_rollover(f, url)).await;
    Box::pin(cold_action(
        f,
        url,
        Some(0),
        action(TacticalAction::EndTurn),
    ))
    .await;
    assert_eq!(
        view(f, Some(1)).await.roll.unwrap().reason,
        "Death saving throw"
    );
    Box::pin(rejected(f, None, TableAction::EndSession)).await;
    Box::pin(cold_raw(f, url, Some(1), &[1])).await;
    assert_eq!(
        state(f).await.rules.as_ref().unwrap().entities[&f.actors[1]]
            .death
            .failures,
        2
    );
    Box::pin(session_rollover(f, url)).await;
    Box::pin(cold_action(
        f,
        url,
        Some(1),
        action(TacticalAction::EndTurn),
    ))
    .await;
    Box::pin(cold_action(f, url, None, action(TacticalAction::EndTurn))).await;
    Box::pin(cold_action(
        f,
        url,
        Some(0),
        action(TacticalAction::EndTurn),
    ))
    .await;
    Box::pin(cold_raw(f, url, Some(1), &[2])).await;
    assert_eq!(
        state(f).await.characters[&f.characters[1]].status,
        CharacterStatus::Dead
    );
    Box::pin(session_rollover(f, url)).await; // Bind retained dead PC; never revive them.
    Box::pin(cold_action(
        f,
        url,
        Some(1),
        action(TacticalAction::EndTurn),
    ))
    .await;
    assert!(
        state(f).await.rules.as_ref().unwrap().entities[&f.actors[1]]
            .death
            .dead
    );
}

#[tokio::test]
async fn aftermath_preserves_real_mage_hold_ready_and_controller_saves_across_sessions() {
    let directory =
        std::env::temp_dir().join(format!("dmd-aftermath-magic-{}", CampaignId::new().0));
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
    Box::pin(magic_scenario(&mut f, &url)).await;
    f.pool.close().await;
    drop(f);
    sqlite_test_cleanup::remove_closed_directory(&directory)
        .await
        .unwrap();
}
#[tokio::test]
async fn aftermath_retains_real_dying_and_dead_character_authority_without_healing() {
    let directory =
        std::env::temp_dir().join(format!("dmd-aftermath-death-{}", CampaignId::new().0));
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
    Box::pin(dying_scenario(&mut f, &url)).await;
    f.pool.close().await;
    drop(f);
    sqlite_test_cleanup::remove_closed_directory(&directory)
        .await
        .unwrap();
}
