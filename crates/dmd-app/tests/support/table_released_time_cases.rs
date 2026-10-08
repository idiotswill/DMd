//! Current public table producers, real source casting/Medicine and physical dice.
//! Every durable step uses independently restored file SQLite and exact cold retry.
use super::*;

fn advance(seconds: u32) -> TableAction {
    action(TacticalAction::AdvanceReleasedTime {
        seconds,
        ordering: ReleasedTimeOrdering::HostSelect,
        ruling: "Private timing decision: wait for the retained absolute deadlines.".into(),
    })
}

async fn step(
    f: &mut Fixture,
    url: &str,
    directory: &Path,
    action: TableAction,
) -> TableTransportResult {
    let request = request(f, None, action).await;
    Box::pin(durable_step(f, url, directory, "elapsed", request)).await
}

async fn refuse(f: &Fixture, request: TableTransportRequest) {
    let before = Box::new(export_campaign(&f.pool, f.campaign).await.unwrap());
    let Err(RunnableCampaignError::TableRejected(_)) =
        Box::pin(f.runtime.submit_presented_table(request)).await
    else {
        panic!("expected an authority/state rejection before durable mutation");
    };
    let mut after = Box::new(export_campaign(&f.pool, f.campaign).await.unwrap());
    after.exported_at_utc = before.exported_at_utc.clone();
    assert_eq!(after, before, "refusal changes no persisted row");
}

async fn prepare_mages(f: &mut Fixture) -> ([EntityId; 3], SessionParticipant) {
    // Keep both measured players outside the encounter. A separate, normally
    // created attending character supplies the existing setup prerequisite.
    let mut attendees = state(f)
        .await
        .table
        .unwrap()
        .active_session
        .unwrap()
        .participants;
    f.host(TableAction::EndSession, Some(f.session)).await;
    let player = PlayerId::new();
    let character = CharacterId::new();
    let actor = EntityId::new();
    f.host(
        TableAction::AddPlayer {
            id: player,
            name: "Courtyard participant".into(),
        },
        None,
    )
    .await;
    f.host(
        TableAction::CreateCharacter {
            character_id: character,
            entity_id: actor,
            player_id: player,
            input: input("Courtyard fighter"),
        },
        None,
    )
    .await;
    let count = view(f, None)
        .await
        .characters
        .iter()
        .find(|entry| entry.character_id == character)
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
        None,
    )
    .await;
    let attendee = SessionParticipant {
        player_id: player,
        character_id: Some(character),
        attendance: AttendanceStatus::Present,
    };
    attendees.push(attendee.clone());
    f.session = PlaySessionId::new();
    f.host(
        TableAction::StartSession {
            id: f.session,
            name: "Courtyard session".into(),
            participants: attendees,
        },
        Some(f.session),
    )
    .await;
    let mut actors = Vec::new();
    for _ in 0..3 {
        let (actor, _, _) =
            Box::pin(super::super::super::table_source_control_cases::create_mage(f)).await;
        actors.push(actor);
    }
    let point = |x, y, z| SpatialPoint { x, y, z };
    f.host(
        TableAction::PrepareBattlefield {
            setup: Box::new(TableBattlefieldSetup {
                encounter_id: EncounterId::new(),
                scene_id: SceneId::new(),
                location_id: LocationId::new(),
                name: "A closed courtyard with one attending character".into(),
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
                characters: vec![TableCharacterPlacement {
                    character_id: character,
                    position: point(80, 80, 0),
                    height: 12,
                    allies: vec![],
                    enemies: vec![],
                }],
                creatures: actors
                    .iter()
                    .enumerate()
                    .map(|(i, actor)| TableCreaturePlacement {
                        actor: *actor,
                        public_label: "Spellcaster".into(),
                        position: point(10 + i as i32 * 20, 20, 0),
                        height: 12,
                        allies: vec![],
                        enemies: vec![],
                    })
                    .collect(),
                area_grid_policy: None,
                geometry_ruling: Ruling {
                    basis: RulingBasis::GmAdjudication,
                    reason: "Three separate source spaces and one attending character; both measured observers are absent from the scene.".into(),
                },
            }),
        },
        Some(f.session),
    )
    .await;
    let begin = request(
        f,
        None,
        action(TacticalAction::Begin {
            execution: TacticalExecutionVersion::ReleasedTimeV1,
            combatants: actors
                .iter()
                .map(|actor| TacticalCombatant {
                    actor: *actor,
                    source: TacticalSource::Creature {
                        definition_id: "mage".into(),
                    },
                    surprised: false,
                })
                .chain(std::iter::once(TacticalCombatant {
                    actor,
                    source: TacticalSource::Character,
                    surprised: false,
                }))
                .collect(),
            groups: vec![
                InitiativeGroup {
                    actors: actors.clone(),
                    request_id: RollRequestId::new(),
                },
                InitiativeGroup {
                    actors: vec![actor],
                    request_id: RollRequestId::new(),
                },
            ],
        }),
    )
    .await;
    let mut old_begin = begin.clone();
    old_begin.command_id = CommandId::new();
    let TableTransportInput::Action(ref mut table_action) = old_begin.input else {
        unreachable!()
    };
    let TableAction::Tactical {
        action: TacticalAction::Begin { execution, .. },
    } = table_action.as_mut()
    else {
        unreachable!()
    };
    *execution = TacticalExecutionVersion::EncounterReleaseV1;
    Box::pin(refuse(f, old_begin)).await;
    assert!(matches!(
        Box::pin(f.runtime.submit_presented_table(begin))
            .await
            .unwrap(),
        TableTransportResult::Accepted(_)
    ));
    Box::pin(setup_raw(f, None, &[12])).await;
    let owner = f
        .runtime
        .presented_table_view(f.campaign, TableViewer::Player(player))
        .await
        .unwrap();
    let roll = owner.roll.unwrap();
    assert_eq!(roll.mode, RollMode::Normal);
    assert_eq!(
        roll.dice,
        vec![DieSpec {
            count: 1,
            sides: 20
        }]
    );
    let owner_roll = channel_request(
        f,
        TableTransportChannel::Player {
            player_id: player,
            character_id: character,
        },
        action(TacticalAction::SubmitRoll {
            result: RollResult {
                request_id: roll.id,
                source: RollSource::Physical,
                dice: vec![DieResult {
                    sides: 20,
                    value: 1,
                }],
            },
        }),
    )
    .await;
    assert!(matches!(
        Box::pin(f.runtime.submit_presented_table(owner_roll))
            .await
            .unwrap(),
        TableTransportResult::Accepted(_)
    ));
    let tie = request(
        f,
        None,
        action(TacticalAction::ProposeInitiativeTie {
            order: actors.clone(),
        }),
    )
    .await;
    assert!(matches!(
        Box::pin(f.runtime.submit_presented_table(tie))
            .await
            .unwrap(),
        TableTransportResult::Accepted(_)
    ));
    let started = state(f).await;
    assert_eq!(started.clock.now, WorldInstant(0));
    let timing = started.rules.as_ref().unwrap().timing.as_ref().unwrap();
    assert_eq!(
        timing
            .order
            .iter()
            .map(|entry| entry.actor)
            .collect::<Vec<_>>(),
        actors
            .iter()
            .copied()
            .chain(std::iter::once(actor))
            .collect::<Vec<_>>()
    );
    assert_eq!(
        timing
            .order
            .iter()
            .map(|entry| entry.total)
            .collect::<Vec<_>>(),
        vec![14, 14, 14, 3]
    );
    assert!(f.actors.iter().all(|observer| {
        started
            .encounter
            .as_ref()
            .unwrap()
            .participant(*observer)
            .is_none()
    }));
    (actors.try_into().unwrap(), attendee)
}

async fn armor(f: &mut Fixture, url: &str, directory: &Path, actor: EntityId) {
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
        .find(|v| v.choice.spell_id == "mage-armor")
        .unwrap()
        .choice;
    assert!(matches!(
        choice.material,
        SpellMaterialChoice::Material { .. }
    ));
    Box::pin(step(
        f,
        url,
        directory,
        action(TacticalAction::CastSpell {
            choice,
            targets: SpellTargetChoice::Entities(vec![actor]),
        }),
    ))
    .await;
    assert_eq!(
        dmd_rules::tactical_defenses::effective_armor_class(&state(f).await, actor).unwrap(),
        15
    );
}

async fn choose(f: &Fixture, index: usize) -> TableTransportRequest {
    let host = view(f, None).await;
    let choices = &host
        .tactical
        .as_ref()
        .unwrap()
        .released_time
        .as_ref()
        .unwrap()
        .interval
        .as_ref()
        .unwrap()
        .choices;
    let mut request = request(f, None, advance(1)).await;
    request.input = TableTransportInput::SelectWork {
        handle: choices[index].handle,
    };
    request
}

async fn mage_interval(f: &mut Fixture, url: &str, directory: &Path, index: usize) {
    let (mages, attendee) = Box::pin(prepare_mages(f)).await;
    for (i, actor) in mages.into_iter().enumerate() {
        Box::pin(armor(f, url, directory, actor)).await;
        if i != 2 {
            Box::pin(step(f, url, directory, action(TacticalAction::EndTurn))).await;
        }
    }
    Box::pin(step(f, url, directory, conclusion())).await;
    Box::pin(step(
        f,
        url,
        directory,
        action(TacticalAction::FinishEncounter),
    ))
    .await;
    let released = Box::new(state(f).await);
    assert_eq!(released.clock.now, WorldInstant(0));
    assert!(released.rules.as_ref().unwrap().timing.is_none());
    let effects = released
        .rules
        .as_ref()
        .unwrap()
        .tactical_effects
        .as_ref()
        .unwrap()
        .effects
        .clone();
    assert_eq!(effects.len(), 3);
    assert!(
        effects
            .iter()
            .all(|e| e.expires == TacticalEffectExpiry::AtTime(WorldInstant(28_800)))
    );
    for player in 0..2 {
        assert!(
            view(f, Some(player))
                .await
                .tactical
                .as_ref()
                .is_none_or(|t| t.released_time.is_none())
        );
        Box::pin(refuse(f, request(f, Some(player), advance(1)).await)).await;
    }
    Box::pin(refuse(f, request(f, None, advance(0)).await)).await;
    Box::pin(step(f, url, directory, advance(28_799))).await;
    assert_eq!(
        state(f)
            .await
            .rules
            .unwrap()
            .tactical_effects
            .unwrap()
            .effects,
        effects
    );
    let first = request(f, None, advance(61)).await;
    let first_result = Box::pin(durable_step(f, url, directory, "interval", first.clone())).await;
    let paused = Box::new(state(f).await);
    let r = paused
        .encounter
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .resolution
        .as_ref()
        .unwrap();
    let context = r.released_interval().unwrap();
    assert_eq!(context.started_at, WorldInstant(28_799));
    assert_eq!(context.progress_at, WorldInstant(28_800));
    assert_eq!(context.target_at, WorldInstant(28_860));
    assert_eq!(r.frames.last().unwrap().len(), 3);
    assert_eq!(
        paused
            .rules
            .as_ref()
            .unwrap()
            .tactical_effects
            .as_ref()
            .unwrap()
            .effects,
        effects
    );
    let host = view(f, None).await;
    let interval = host
        .tactical
        .as_ref()
        .unwrap()
        .released_time
        .as_ref()
        .unwrap()
        .interval
        .as_ref()
        .unwrap();
    assert_eq!(interval.choices.len(), 3);
    assert!(interval.choices.iter().all(|c| c.handle != r.origin.id));
    let selected = choose(f, index).await;
    let stale = choose(f, (index + 1) % 3).await;
    let mut foreign = selected.clone();
    foreign.command_id = CommandId::new();
    foreign.channel = TableTransportChannel::Player {
        player_id: f.players[0],
        character_id: f.characters[0],
    };
    foreign.revision = view(f, Some(0)).await.revision;
    Box::pin(refuse(f, foreign)).await;
    let mut forged = selected.clone();
    forged.command_id = CommandId::new();
    forged.input = TableTransportInput::SelectWork {
        handle: r.origin.id,
    };
    Box::pin(refuse(f, forged)).await;
    Box::pin(refuse(
        f,
        request(
            f,
            None,
            action(TacticalAction::ChooseTurnWork {
                occurrence: r.frames.last().unwrap()[index].occurrence,
            }),
        )
        .await,
    ))
    .await;
    Box::pin(refuse(f, request(f, None, advance(1)).await)).await;
    Box::pin(refuse(
        f,
        request(f, None, action(TacticalAction::EndTurn)).await,
    ))
    .await;
    Box::pin(refuse(
        f,
        request(
            f,
            None,
            TableAction::PrepareBattlefield {
                setup: Box::new(replacement(f, &paused, mages[0])),
            },
        )
        .await,
    ))
    .await;
    let count = view(f, None)
        .await
        .characters
        .iter()
        .find(|c| c.character_id == f.characters[0])
        .unwrap()
        .equipment
        .as_ref()
        .unwrap()
        .initial_item_count;
    Box::pin(refuse(
        f,
        request(
            f,
            None,
            TableAction::PrepareEquipment {
                character_id: f.characters[0],
                item_ids: (0..count).map(|_| ItemId::new()).collect(),
            },
        )
        .await,
    ))
    .await;
    let unrelated = [view(f, Some(0)).await, view(f, Some(1)).await];
    Box::pin(durable_step(
        f,
        url,
        directory,
        "first-deadline",
        selected.clone(),
    ))
    .await;
    assert_eq!(
        view(f, Some(0)).await,
        unrelated[0],
        "private ordering preserves every unrelated DTO field and revision"
    );
    assert_eq!(view(f, Some(1)).await, unrelated[1]);
    Box::pin(refuse(f, stale)).await;
    let partial = Box::new(export_campaign(&f.pool, f.campaign).await.unwrap());
    let current = CampaignState::decode_json(&partial.current_state.state_json).unwrap();
    assert_eq!(
        current
            .rules
            .as_ref()
            .unwrap()
            .tactical_effects
            .as_ref()
            .unwrap()
            .effects
            .len(),
        2
    );
    let fixed = current
        .encounter
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .resolution
        .clone();
    Box::pin(step(f, url, directory, TableAction::EndSession)).await;
    let closed = request(f, None, advance(1)).await;
    Box::pin(refuse(f, closed)).await;
    f.session = PlaySessionId::new();
    let mut attendees = participants(f);
    attendees.push(attendee);
    let start = TableAction::StartSession {
        id: f.session,
        name: "Continue the same elapsed interval".into(),
        participants: attendees,
    };
    Box::pin(step(f, url, directory, start)).await;
    assert_eq!(
        state(f).await.encounter.unwrap().flow.unwrap().resolution,
        fixed
    );
    assert_eq!(
        Box::pin(f.runtime.submit_presented_table(first.clone()))
            .await
            .unwrap(),
        first_result,
        "original accepted envelope remains retryable across the new session"
    );
    let last = choose(f, 0).await;
    Box::pin(durable_step(f, url, directory, "last-deadlines", last)).await;
    let completed = Box::new(state(f).await);
    assert_eq!(completed.clock.now, WorldInstant(28_860));
    assert!(completed.rules.as_ref().unwrap().timing.is_none());
    assert!(
        completed
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
        completed
            .rules
            .as_ref()
            .unwrap()
            .tactical_effects
            .as_ref()
            .unwrap()
            .effects
            .is_empty()
    );
    for actor in mages {
        assert_eq!(
            dmd_rules::tactical_defenses::effective_armor_class(&completed, actor).unwrap(),
            12
        );
    }
    assert_eq!(completed.items, released.items);
    assert_eq!(
        completed.rules.as_ref().unwrap().rolls,
        released.rules.as_ref().unwrap().rolls
    );
    assert_eq!(
        completed.rules.as_ref().unwrap().tactical_creatures,
        released.rules.as_ref().unwrap().tactical_creatures
    );
    assert_eq!(
        completed.encounter_history.as_ref().unwrap().last(),
        released.encounter_history.as_ref().unwrap().last()
    );
    assert_eq!(
        completed
            .encounter_history
            .as_ref()
            .unwrap()
            .elapsed_intervals
            .last()
            .unwrap()
            .origin
            .id,
        first.command_id
    );
    Box::pin(reject_forged_intervals(
        f,
        directory,
        &partial.current_state,
    ))
    .await;
    Box::pin(step(f, url, directory, advance(1))).await;
    assert_eq!(state(f).await.clock.now, WorldInstant(28_861));
    Box::pin(refuse(f, {
        let mut request = selected;
        request.command_id = CommandId::new();
        request.session_id = Some(f.session);
        request.revision = view(f, None).await.revision;
        request
    }))
    .await;
}

async fn reject_forged_intervals(
    f: &Fixture,
    directory: &Path,
    earlier: &dmd_persistence::CurrentStateRow,
) {
    let mut original = Box::new(export_campaign(&f.pool, f.campaign).await.unwrap());
    original
        .snapshots
        .retain(|s| s.event_sequence != earlier.applied_event_sequence);
    original.snapshots.push(dmd_persistence::SnapshotRow {
        campaign_id: earlier.campaign_id.clone(),
        event_sequence: earlier.applied_event_sequence,
        state_schema_version: earlier.schema_version,
        state_json: earlier.state_json.clone(),
        created_at_utc: original.exported_at_utc.clone(),
    });
    original.snapshots.sort_by_key(|s| s.event_sequence);
    let path = directory.join("hostile-destination.sqlite");
    let pool = dmd_persistence::open_sqlite_path(&path).await.unwrap();
    let destination = runtime(pool.clone());
    Box::pin(destination.restore_campaign(&original))
        .await
        .unwrap();
    pool.close().await;
    drop(destination);
    drop(pool);
    let path = directory.join("unrelated-hostile-destination.sqlite");
    let pool = dmd_persistence::open_sqlite_path(&path).await.unwrap();
    let unrelated = Box::pin(Fixture::with_pool(TableContract::default(), pool.clone())).await;
    let destination = runtime(pool.clone());
    let before = destination_rows(&pool).await;
    for mutation in 0..10 {
        let mut forged = original.clone();
        let snapshot = forged
            .snapshots
            .iter()
            .find(|s| s.event_sequence == earlier.applied_event_sequence)
            .unwrap();
        let mut image = CampaignState::decode_json(&snapshot.state_json).unwrap();
        let flow = image.encounter.as_mut().unwrap().flow.as_mut().unwrap();
        let r = flow.resolution.as_mut().unwrap();
        match mutation {
            0 => r.released_interval_mut().unwrap().target_at.0 += 1,
            1 => {
                r.released_interval_mut()
                    .unwrap()
                    .batches
                    .last_mut()
                    .unwrap()
                    .completions[0]
                    .completed_by
                    .id = CommandId::new()
            }
            2 => r
                .released_interval_mut()
                .unwrap()
                .batches
                .last_mut()
                .unwrap()
                .bindings
                .pop()
                .map(|_| ())
                .unwrap(),
            3 => r.released_interval_mut().unwrap().ruling = "Invented accepted ruling".into(),
            4 => image.clock.now.0 += 1,
            5 => {
                image
                    .rules
                    .as_mut()
                    .unwrap()
                    .tactical_effects
                    .as_mut()
                    .unwrap()
                    .effects[0]
                    .source
                    .command
                    .id = CommandId::new()
            }
            6 => {
                image
                    .rules
                    .as_mut()
                    .unwrap()
                    .tactical_effects
                    .as_mut()
                    .unwrap()
                    .effects[0]
                    .expires = TacticalEffectExpiry::AtTime(WorldInstant(28_801))
            }
            7 => {
                let mut current =
                    CampaignState::decode_json(&forged.current_state.state_json).unwrap();
                current
                    .encounter_history
                    .as_mut()
                    .unwrap()
                    .elapsed_intervals
                    .last_mut()
                    .unwrap()
                    .completed_at
                    .0 += 1;
                forged.current_state.state_json = serde_json::to_string(&current).unwrap();
            }
            8 => {
                let origin = r.origin.id.0.to_string();
                forged
                    .command_audit
                    .iter_mut()
                    .find(|a| a.id == origin)
                    .unwrap()
                    .session_id = Some(PlaySessionId::new().0.to_string());
            }
            9 => {
                let origin = r.origin.id.0.to_string();
                forged
                    .event_journal
                    .iter_mut()
                    .find(|e| e.command_id == origin)
                    .unwrap()
                    .session_id = Some(PlaySessionId::new().0.to_string());
            }
            _ => unreachable!(),
        }
        forged
            .snapshots
            .iter_mut()
            .find(|s| s.event_sequence == earlier.applied_event_sequence)
            .unwrap()
            .state_json = serde_json::to_string(&image).unwrap();
        assert_ne!(forged, original, "mutation {mutation} is real");
        assert!(
            Box::pin(destination.restore_campaign(&forged))
                .await
                .is_err(),
            "mutation {mutation}"
        );
        assert_eq!(
            destination_rows(&pool).await,
            before,
            "all destination tables unchanged for mutation {mutation}"
        );
    }
    pool.close().await;
    drop(unrelated);
    drop(destination);
    drop(pool);
}

async fn destination_rows(
    pool: &sqlx::SqlitePool,
) -> std::collections::BTreeMap<String, Vec<String>> {
    use sqlx::Row;
    let names: Vec<String> =
        sqlx::query_scalar("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
            .fetch_all(pool)
            .await
            .unwrap();
    let mut result = std::collections::BTreeMap::new();
    for name in names {
        let quoted = name.replace('"', "\"\"");
        let columns = sqlx::query(&format!("PRAGMA table_info(\"{quoted}\")"))
            .fetch_all(pool)
            .await
            .unwrap();
        let cells = columns
            .iter()
            .map(|column| {
                let name: String = column.get("name");
                let quoted = name.replace('"', "\"\"");
                // Preserve null/type and the complete stored text/blob bytes; no row
                // identity, schema metadata or existing campaign may change on refusal.
                format!("json_array(typeof(\"{quoted}\"),hex(\"{quoted}\"))")
            })
            .collect::<Vec<_>>()
            .join(",");
        let values: Vec<String> = sqlx::query_scalar(&format!(
            "SELECT json_array({cells}) AS image FROM \"{quoted}\" ORDER BY image"
        ))
        .fetch_all(pool)
        .await
        .unwrap();
        result.insert(name, values);
    }
    result
}

#[tokio::test]
async fn real_mage_elapsed_deadlines_are_host_opaque_private_and_durable_in_both_orders() {
    for index in [0, 2] {
        let directory =
            std::env::temp_dir().join(format!("dmd-elapsed-mages-{}", CampaignId::new().0));
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
        Box::pin(mage_interval(&mut f, &url, &directory, index)).await;
        f.pool.close().await;
        drop(f);
        sqlite_test_cleanup::remove_closed_directory(&directory)
            .await
            .unwrap();
    }
}

fn independent_view_identity(mut view: TablePresentedView) -> serde_json::Value {
    // Independent accepted commands allocate random presentation/entry identities.
    // Normalize only those identities; retain every label, field, order and count.
    for entry in &mut view.transcript {
        entry.id.clear();
    }
    let mut value = serde_json::to_value(view).unwrap();
    value.as_object_mut().unwrap().remove("revision");
    value
}

#[tokio::test]
async fn zero_one_two_hidden_deadlines_have_equal_unrelated_views_at_equal_world_time() {
    let directory =
        std::env::temp_dir().join(format!("dmd-elapsed-privacy-{}", CampaignId::new().0));
    std::fs::create_dir_all(&directory).unwrap();
    let prefix_pool = dmd_persistence::open_sqlite_path(&directory.join("prefix.sqlite"))
        .await
        .unwrap();
    let mut prefix = Box::pin(Fixture::with_pool(TableContract::default(), prefix_pool)).await;
    let (mages, _attendee) = Box::pin(prepare_mages(&mut prefix)).await;
    let saved = Box::new(
        export_campaign(&prefix.pool, prefix.campaign)
            .await
            .unwrap(),
    );
    let mut expected = None;
    for count in 0..=2 {
        let path = directory.join(format!("case-{count}.sqlite"));
        let url = format!("sqlite://{}", path.to_string_lossy().replace('\\', "/"));
        let pool = open_sqlite(&url).await.unwrap();
        let branch_runtime = runtime(pool.clone());
        Box::pin(branch_runtime.restore_campaign(&saved))
            .await
            .unwrap();
        let mut f = Fixture {
            runtime: branch_runtime,
            pool,
            campaign: prefix.campaign,
            players: prefix.players,
            characters: prefix.characters,
            actors: prefix.actors,
            session: prefix.session,
        };
        for (index, actor) in mages.into_iter().enumerate() {
            if index < count {
                Box::pin(armor(&mut f, &url, &directory, actor)).await;
            }
            if index != 2 {
                Box::pin(step(
                    &mut f,
                    &url,
                    &directory,
                    action(TacticalAction::EndTurn),
                ))
                .await;
            }
        }
        Box::pin(step(&mut f, &url, &directory, conclusion())).await;
        Box::pin(step(
            &mut f,
            &url,
            &directory,
            action(TacticalAction::FinishEncounter),
        ))
        .await;
        Box::pin(step(&mut f, &url, &directory, advance(28_800))).await;
        let current = state(&f).await;
        assert_eq!(current.clock.now, WorldInstant(28_800));
        let remaining = current
            .rules
            .as_ref()
            .unwrap()
            .tactical_effects
            .as_ref()
            .unwrap()
            .effects
            .len();
        assert_eq!(remaining, if count == 2 { 2 } else { 0 });
        let unrelated = [view(&f, Some(0)).await, view(&f, Some(1)).await];
        for visible in &unrelated {
            let encoded = serde_json::to_string(visible).unwrap();
            assert!(!encoded.contains("Private timing decision"));
            assert!(!encoded.contains("Resolve simultaneous deadline"));
            for actor in mages {
                assert!(!encoded.contains(&actor.0.to_string()));
            }
        }
        let normalized = unrelated.clone().map(independent_view_identity);
        if let Some(expected) = &expected {
            assert_eq!(&normalized, expected, "hidden count {count}");
        } else {
            expected = Some(normalized);
        }
        if count == 2 {
            let selection = choose(&f, 0).await;
            Box::pin(durable_step(
                &mut f,
                &url,
                &directory,
                "private-final-deadline",
                selection,
            ))
            .await;
            assert_eq!(view(&f, Some(0)).await, unrelated[0]);
            assert_eq!(view(&f, Some(1)).await, unrelated[1]);
        }
        f.pool.close().await;
        drop(f);
    }
    prefix.pool.close().await;
    drop(prefix);
    sqlite_test_cleanup::remove_closed_directory(&directory)
        .await
        .unwrap();
}

async fn stable_interval(f: &mut Fixture, url: &str, directory: &Path) {
    Box::pin(super::super::super::table_medicine_cases::prepare(f, false)).await;
    let target = f.actors[1];
    let aid = request(
        f,
        Some(0),
        action(TacticalAction::FirstAid {
            target,
            purpose: MedicinePurpose::Stabilize,
        }),
    )
    .await;
    Box::pin(durable_step(f, url, directory, "medicine", aid)).await;
    for (player, faces) in [(0, vec![20]), (1, vec![2])] {
        let dice = request(f, Some(player), raw_action(f, Some(player), &faces).await).await;
        Box::pin(durable_step(f, url, directory, "physical-dice", dice)).await;
    }
    let stabilized = Box::new(state(f).await);
    let recovery = stabilized
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
    assert_eq!(
        recovery.delay_roll.as_ref().unwrap().source,
        RollSource::Physical
    );
    assert_eq!(
        recovery.delay_roll.as_ref().unwrap().dice,
        vec![DieResult { sides: 4, value: 2 }]
    );
    Box::pin(step(f, url, directory, conclusion())).await;
    Box::pin(step(
        f,
        url,
        directory,
        action(TacticalAction::FinishEncounter),
    ))
    .await;
    Box::pin(step(f, url, directory, advance(7199))).await;
    let before = Box::new(state(f).await);
    assert_eq!(before.clock.now, WorldInstant(7205));
    assert_eq!(before.rules.as_ref().unwrap().entities[&target].hp, 0);
    assert!(
        before.rules.as_ref().unwrap().entities[&target]
            .death
            .stable
    );
    Box::pin(step(f, url, directory, advance(1))).await;
    let awake = Box::new(state(f).await);
    assert_eq!(awake.clock.now, WorldInstant(7206));
    assert_eq!(awake.rules.as_ref().unwrap().entities[&target].hp, 1);
    assert!(!awake.rules.as_ref().unwrap().entities[&target].death.stable);
    assert_eq!(
        awake.rules.as_ref().unwrap().rolls,
        before.rules.as_ref().unwrap().rolls
    );
    assert!(awake.rules.as_ref().unwrap().timing.is_none());
    Box::pin(step(f, url, directory, advance(1))).await;
    assert_eq!(
        state(f).await.rules.unwrap().entities[&target],
        awake.rules.as_ref().unwrap().entities[&target]
    );
}

#[tokio::test]
async fn real_attack_medicine_and_physical_d4_wake_once_at_the_absolute_deadline() {
    let directory =
        std::env::temp_dir().join(format!("dmd-elapsed-recovery-{}", CampaignId::new().0));
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
    Box::pin(stable_interval(&mut f, &url, &directory)).await;
    f.pool.close().await;
    drop(f);
    sqlite_test_cleanup::remove_closed_directory(&directory)
        .await
        .unwrap();
}

#[tokio::test]
async fn genuine_old_source_armor_releases_upgrades_and_expires_without_rewriting_history() {
    let original = Box::new(
        dmd_persistence::CampaignExport::from_json(include_str!(
            "../fixtures/reactions-v1-aftermath-be544.json"
        ))
        .unwrap(),
    );
    let old = Box::new(CampaignState::decode_json(&original.current_state.state_json).unwrap());
    let mage = old
        .encounter
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .combatants[0]
        .actor;
    let directory =
        std::env::temp_dir().join(format!("dmd-elapsed-original-{}", CampaignId::new().0));
    std::fs::create_dir_all(&directory).unwrap();
    let url = format!(
        "sqlite://{}",
        directory
            .join("campaign.sqlite")
            .to_string_lossy()
            .replace('\\', "/")
    );
    let pool = open_sqlite(&url).await.unwrap();
    let app = runtime(pool.clone());
    Box::pin(app.restore_campaign(&original)).await.unwrap();
    let pcs = old.characters.values().collect::<Vec<_>>();
    assert_eq!(pcs.len(), 2);
    let mut f = Fixture {
        runtime: app,
        pool,
        campaign: old.campaign_id(),
        players: [
            pcs[0].controlling_player_id.unwrap(),
            pcs[1].controlling_player_id.unwrap(),
        ],
        characters: [pcs[0].id, pcs[1].id],
        actors: [pcs[0].entity_id, pcs[1].entity_id],
        session: PlaySessionId::new(),
    };
    let upgrade = closed_request(
        &f,
        action(TacticalAction::UpgradeExecutionTo {
            execution: TacticalExecutionVersion::EncounterReleaseV1,
        }),
    )
    .await;
    Box::pin(durable_step(
        &mut f,
        &url,
        &directory,
        "original-five",
        upgrade,
    ))
    .await;
    let finish = closed_request(&f, action(TacticalAction::FinishEncounter)).await;
    Box::pin(durable_step(
        &mut f,
        &url,
        &directory,
        "original-finish",
        finish,
    ))
    .await;
    let five = Box::new(state(&f).await);
    let receipt = five
        .encounter_history
        .as_ref()
        .unwrap()
        .last()
        .unwrap()
        .clone();
    assert_eq!(
        receipt.execution,
        TacticalExecutionVersion::EncounterReleaseV1
    );
    assert_eq!(
        five.rules
            .as_ref()
            .unwrap()
            .tactical_effects
            .as_ref()
            .unwrap()
            .effects,
        old.rules
            .as_ref()
            .unwrap()
            .tactical_effects
            .as_ref()
            .unwrap()
            .effects
    );
    let upgrade_seven = action(TacticalAction::UpgradeExecutionTo {
        execution: TacticalExecutionVersion::ReleasedTimeV1,
    });
    let closed = closed_request(&f, upgrade_seven.clone()).await;
    Box::pin(refuse(&f, closed)).await;
    let start = TableAction::StartSession {
        id: f.session,
        name: "Continue the old source deadlines".into(),
        participants: participants(&f),
    };
    Box::pin(step(&mut f, &url, &directory, start)).await;
    let old_elapsed = request(&f, None, advance(1)).await;
    Box::pin(refuse(&f, old_elapsed)).await;
    Box::pin(step(&mut f, &url, &directory, upgrade_seven)).await;
    let seven = Box::new(state(&f).await);
    let provenance = seven
        .encounter
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .released_time_upgrade
        .as_ref()
        .unwrap();
    assert_eq!(provenance.release, receipt.released_by.id);
    assert_eq!(
        provenance.from,
        TacticalExecutionVersion::EncounterReleaseV1
    );
    assert_eq!(provenance.to, TacticalExecutionVersion::ReleasedTimeV1);
    assert_eq!(seven.clock, five.clock);
    assert_eq!(seven.items, five.items);
    assert_eq!(seven.rules, five.rules);
    Box::pin(step(&mut f, &url, &directory, advance(28_800))).await;
    let expired = Box::new(state(&f).await);
    assert_eq!(expired.clock.now, WorldInstant(28_800));
    assert_eq!(
        dmd_rules::tactical_defenses::effective_armor_class(&expired, mage).unwrap(),
        12
    );
    assert_eq!(
        expired.encounter_history.as_ref().unwrap().last(),
        Some(&receipt)
    );
    let after = Box::new(export_campaign(&f.pool, f.campaign).await.unwrap());
    assert_eq!(
        &after.event_journal[..original.event_journal.len()],
        &original.event_journal
    );
    assert_eq!(
        &after.table_projection_history[..original.table_projection_history.len()],
        &original.table_projection_history
    );
    for audit in &original.command_audit {
        assert!(after.command_audit.contains(audit));
    }
    for snapshot in &original.snapshots {
        assert!(after.snapshots.contains(snapshot));
    }
    for binding in &original.table_transport_bindings {
        assert!(after.table_transport_bindings.contains(binding));
        let request: TableTransportRequest = serde_json::from_str(&binding.request_json).unwrap();
        let response = Box::pin(f.runtime.submit_presented_table(request))
            .await
            .unwrap();
        assert_eq!(
            serde_json::to_string(&response).unwrap(),
            binding.response_json
        );
    }
    let mut retried = Box::new(export_campaign(&f.pool, f.campaign).await.unwrap());
    retried.exported_at_utc = after.exported_at_utc.clone();
    assert_eq!(retried, after);
    f.pool.close().await;
    drop(f);
    sqlite_test_cleanup::remove_closed_directory(&directory)
        .await
        .unwrap();
}
