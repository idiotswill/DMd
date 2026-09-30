//! Actual source creation, source-only attendance, opaque response ownership and
//! file-SQLite continuation. No initial rules image is manufactured or rewritten.
use super::*;
use dmd_rules::tactical::TacticalAction;

#[path = "table_timed_expiry_cases.rs"]
mod timed_expiry;

struct Sources {
    actors: [EntityId; 4],
    owners: [PlayerId; 4],
    spectator: PlayerId,
}
impl Sources {
    fn channel(&self, index: usize) -> TableTransportChannel {
        TableTransportChannel::SourceCreature {
            player_id: self.owners[index],
            actor: self.actors[index],
        }
    }
}
fn runtime(pool: sqlx::SqlitePool) -> CampaignRuntime {
    CampaignRuntime::from_content_root(
        pool,
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../content"),
    )
}
fn action(action: TableAction) -> TableTransportInput {
    TableTransportInput::Action(Box::new(action))
}
fn tactical(action_: TacticalAction) -> TableTransportInput {
    action(TableAction::Tactical { action: action_ })
}
fn conclusion() -> TableTransportInput {
    tactical(TacticalAction::ConcludeHostilities {
        cadence: AftermathCadence::ContinueExistingOrder,
        ruling: "The source-creature practice ends; retain current order and ongoing effects."
            .into(),
    })
}
fn missile(handle: CommandId, decision: TableMissileInput) -> TableTransportInput {
    TableTransportInput::MissileResponse {
        handle,
        decision: Box::new(decision),
    }
}
async fn state(f: &Fixture) -> Box<CampaignState> {
    let exported = Box::new(export_campaign(&f.pool, f.campaign).await.unwrap());
    Box::new(CampaignState::decode_json(&exported.current_state.state_json).unwrap())
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
async fn foreign(f: &Fixture, sources: &Sources) -> TablePresentedView {
    f.runtime
        .presented_table_view(f.campaign, TableViewer::Player(sources.spectator))
        .await
        .unwrap()
}
async fn request(
    f: &Fixture,
    channel: TableTransportChannel,
    input: TableTransportInput,
) -> TableTransportRequest {
    let projected = view(f, &channel).await;
    let session_id = if let TableTransportInput::Action(table) = &input
        && let TableAction::StartSession { id, .. } = **table
    {
        Some(id)
    } else {
        projected
            .active_session
            .as_ref()
            .map(|session| session.session_id)
    };
    TableTransportRequest {
        version: 2,
        command_id: CommandId::new(),
        campaign_id: f.campaign,
        session_id,
        revision: projected.revision,
        channel,
        input,
    }
}
async fn accept(
    f: &Fixture,
    channel: TableTransportChannel,
    input: TableTransportInput,
) -> TableTransportRequest {
    let request = request(f, channel, input).await;
    assert!(matches!(
        Box::pin(f.runtime.submit_presented_table(request.clone()))
            .await
            .unwrap(),
        TableTransportResult::Accepted(_)
    ));
    request
}
async fn unchanged(f: &Fixture, request: TableTransportRequest) {
    let before = Box::new(export_campaign(&f.pool, f.campaign).await.unwrap());
    assert!(
        Box::pin(f.runtime.submit_presented_table(request))
            .await
            .is_err()
    );
    let mut after = Box::new(export_campaign(&f.pool, f.campaign).await.unwrap());
    after.exported_at_utc = before.exported_at_utc.clone();
    assert_eq!(
        after, before,
        "denial must preserve the entire store, including receipts and projection history"
    );
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
async fn reopen(f: &mut Fixture, path: &Path) {
    f.pool.close().await;
    f.pool = dmd_persistence::open_sqlite_path(path).await.unwrap();
    f.runtime = runtime(f.pool.clone());
    Box::pin(f.runtime.resume_campaign(f.campaign))
        .await
        .unwrap();
}
// Both independent continuations are actual closed/reopened files. These named
// stages remain externally observable through normal export_campaign while the
// scenario is running; no production or test-only capture switch is introduced.
async fn cold_step(
    f: &mut Fixture,
    path: &Path,
    stage: &str,
    request: TableTransportRequest,
) -> TableTransportResult {
    let before = Box::new(export_campaign(&f.pool, f.campaign).await.unwrap());
    let mirror_path = path
        .parent()
        .unwrap()
        .join(format!("{stage}-{}.sqlite", request.command_id.0));
    let pool = dmd_persistence::open_sqlite_path(&mirror_path)
        .await
        .unwrap();
    Box::pin(runtime(pool.clone()).restore_campaign(&before))
        .await
        .unwrap();
    pool.close().await;
    let pool = dmd_persistence::open_sqlite_path(&mirror_path)
        .await
        .unwrap();
    let mirror = runtime(pool.clone());
    Box::pin(mirror.resume_campaign(f.campaign)).await.unwrap();
    Box::pin(reopen(f, path)).await;
    let primary = Box::pin(f.runtime.submit_presented_table(request.clone()))
        .await
        .unwrap();
    let independent = Box::pin(mirror.submit_presented_table(request.clone()))
        .await
        .unwrap();
    let (TableTransportResult::Accepted(a), TableTransportResult::Accepted(b)) =
        (&primary, &independent)
    else {
        panic!("accepted command required")
    };
    assert_eq!(a.outcome, b.outcome);
    let mirrored = Box::new(export_campaign(&pool, f.campaign).await.unwrap());
    let mut primary_state = Box::new(
        export_campaign(&f.pool, f.campaign)
            .await
            .unwrap()
            .current_state,
    );
    // Independent restores may serialize HashMap object keys in a different
    // order. Compare every JSON value (including unknown fields and array order)
    // before normalizing only that encoding difference for the row comparison.
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&mirrored.current_state.state_json).unwrap(),
        serde_json::from_str::<serde_json::Value>(&primary_state.state_json).unwrap()
    );
    primary_state.state_json = mirrored.current_state.state_json.clone();
    assert_eq!(*primary_state, mirrored.current_state);
    drop(primary_state);
    assert_eq!(
        Box::pin(mirror.submit_presented_table(request.clone()))
            .await
            .unwrap(),
        independent
    );
    let recovered_path = path
        .parent()
        .unwrap()
        .join(format!("{stage}-recovered-{}.sqlite", request.command_id.0));
    let recovered = dmd_persistence::open_sqlite_path(&recovered_path)
        .await
        .unwrap();
    Box::pin(runtime(recovered.clone()).restore_campaign(&mirrored))
        .await
        .unwrap();
    recovered.close().await;
    pool.close().await;
    Box::pin(reopen(f, path)).await;
    let before_retry = Box::new(export_campaign(&f.pool, f.campaign).await.unwrap());
    assert_eq!(
        Box::pin(f.runtime.submit_presented_table(request.clone()))
            .await
            .unwrap(),
        primary
    );
    let mut changed = request;
    changed.input = tactical(TacticalAction::Dodge);
    Box::pin(unchanged(f, changed)).await;
    let mut after_retry = Box::new(export_campaign(&f.pool, f.campaign).await.unwrap());
    after_retry.exported_at_utc = before_retry.exported_at_utc.clone();
    assert_eq!(after_retry, before_retry);
    primary
}
fn raw(projected: TablePresentedView, face: u16) -> TableTransportInput {
    let roll = projected.roll.unwrap();
    let dice = if roll.mode == RollMode::Normal {
        roll.dice
            .iter()
            .flat_map(|die| std::iter::repeat_n(die.sides, usize::from(die.count)))
            .collect::<Vec<_>>()
    } else {
        vec![20, 20]
    };
    tactical(TacticalAction::SubmitRoll {
        result: RollResult {
            request_id: roll.id,
            source: RollSource::Physical,
            dice: dice
                .into_iter()
                .map(|sides| DieResult { sides, value: face })
                .collect(),
        },
    })
}
fn record(state: &CampaignState) -> &TacticalMissile {
    &state
        .encounter
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .resolution
        .as_ref()
        .unwrap()
        .missiles[0]
}
async fn prepare(f: &mut Fixture, same_target_owner: bool) -> Sources {
    Box::pin(prepare_sources(f, same_target_owner, false)).await
}
async fn prepare_sources(
    f: &mut Fixture,
    same_target_owner: bool,
    concentrating_source: bool,
) -> Sources {
    Box::pin(prepare_with_character(
        f,
        same_target_owner,
        concentrating_source,
        false,
    ))
    .await
}
async fn prepare_with_character(
    f: &mut Fixture,
    same_target_owner: bool,
    concentrating_source: bool,
    include_character: bool,
) -> Sources {
    // Add an actual unrelated viewer while the campaign is between sessions.
    f.host(TableAction::EndSession, Some(f.session)).await;
    let spectator = PlayerId::new();
    f.host(
        TableAction::AddPlayer {
            id: spectator,
            name: "Observer".into(),
        },
        None,
    )
    .await;
    f.session = PlaySessionId::new();
    f.host(
        TableAction::StartSession {
            id: f.session,
            name: "Source setup".into(),
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
    let sources = Sources {
        actors: std::array::from_fn(|_| EntityId::new()),
        owners: [
            f.players[0],
            f.players[0],
            f.players[1],
            if same_target_owner {
                f.players[1]
            } else {
                f.players[0]
            },
        ],
        spectator,
    };
    let definitions = [
        "night-hag",
        "night-hag",
        "mage",
        if concentrating_source {
            "cultist-fanatic"
        } else {
            "mage"
        },
    ];
    for (index, definition) in definitions.into_iter().enumerate() {
        let host = view(f, &TableTransportChannel::Host).await;
        let options = f
            .runtime
            .table_creature_options(TableCreatureOptionsRequest {
                campaign_id: f.campaign,
                channel: TableTransportChannel::Host,
                revision: host.revision,
            })
            .await
            .unwrap();
        let source = options
            .iter()
            .find(|source| source.definition_id == definition)
            .unwrap();
        f.host(
            TableAction::CreateCreature {
                creation: Box::new(TableCreatureCreation {
                    entity_id: sources.actors[index],
                    name: format!("Private source {index}"),
                    definition_id: definition.into(),
                    size: CreatureSize::Medium,
                    additional_languages: if definition == "mage" {
                        vec!["dwarvish".into(), "elvish".into(), "draconic".into()]
                    } else {
                        vec![]
                    },
                    ammunition_units: 0,
                    item_ids: (0..source.item_count).map(|_| ItemId::new()).collect(),
                }),
            },
            Some(f.session),
        )
        .await;
    }
    if include_character {
        let count = view(f, &TableTransportChannel::Host)
            .await
            .characters
            .iter()
            .find(|character| character.character_id == f.characters[1])
            .unwrap()
            .equipment
            .as_ref()
            .unwrap()
            .initial_item_count;
        f.host(
            TableAction::PrepareEquipment {
                character_id: f.characters[1],
                item_ids: (0..count).map(|_| ItemId::new()).collect(),
            },
            Some(f.session),
        )
        .await;
    }
    let host = view(f, &TableTransportChannel::Host).await;
    let adoption = f
        .runtime
        .table_source_control_options(TableCreatureOptionsRequest {
            campaign_id: f.campaign,
            channel: TableTransportChannel::Host,
            revision: host.revision,
        })
        .await
        .unwrap();
    Box::pin(accept(
        f,
        TableTransportChannel::Host,
        action(TableAction::EnableSourceActorAccess {
            adopted: adoption.adopted,
        }),
    ))
    .await;
    for index in 0..4 {
        Box::pin(accept(
            f,
            TableTransportChannel::Host,
            action(TableAction::SetSourceCreatureController {
                actor: sources.actors[index],
                controller: CreatureController::Player(sources.owners[index]),
            }),
        ))
        .await;
    }
    Box::pin(accept(
        f,
        TableTransportChannel::Host,
        action(TableAction::EndSession),
    ))
    .await;
    f.session = PlaySessionId::new();
    Box::pin(accept(
        f,
        TableTransportChannel::Host,
        action(TableAction::StartSession {
            id: f.session,
            name: "Only source creatures".into(),
            participants: f
                .players
                .iter()
                .enumerate()
                .map(|(index, player)| SessionParticipant {
                    player_id: *player,
                    character_id: (include_character && index == 1).then_some(f.characters[1]),
                    attendance: AttendanceStatus::Present,
                })
                .collect(),
        }),
    ))
    .await;
    assert!(
        include_character
            || view(f, &TableTransportChannel::Host)
                .await
                .active_session
                .unwrap()
                .participants
                .iter()
                .all(|participant| participant.character_id.is_none())
    );
    let point = |x, y, z| SpatialPoint { x, y, z };
    Box::pin(accept(
        f,
        TableTransportChannel::Host,
        action(TableAction::PrepareBattlefield {
            setup: Box::new(TableBattlefieldSetup {
                encounter_id: EncounterId::new(),
                scene_id: SceneId::new(),
                location_id: LocationId::new(),
                name: "Bright stone chamber".into(),
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
                characters: if include_character {
                    vec![TableCharacterPlacement {
                        character_id: f.characters[1],
                        position: point(130, 10, 0),
                        height: 12,
                        allies: vec![],
                        enemies: vec![],
                    }]
                } else {
                    vec![]
                },
                creatures: sources
                    .actors
                    .iter()
                    .enumerate()
                    .map(|(i, actor)| TableCreaturePlacement {
                        actor: *actor,
                        public_label: [
                            "First horned figure",
                            "Second horned figure",
                            "Red cloak",
                            "Blue cloak",
                        ][i]
                            .into(),
                        position: point(10 + i as i32 * 30, 10, 0),
                        height: 12,
                        allies: vec![],
                        enemies: vec![],
                    })
                    .collect(),
                area_grid_policy: None,
                geometry_ruling: Ruling {
                    basis: RulingBasis::GmAdjudication,
                    reason: "All four source creatures are visible on a level floor.".into(),
                },
            }),
        }),
    ))
    .await;
    let mut combatants = sources
        .actors
        .iter()
        .enumerate()
        .map(|(i, actor)| TacticalCombatant {
            actor: *actor,
            source: TacticalSource::Creature {
                definition_id: definitions[i].into(),
            },
            surprised: i % 2 == 1,
        })
        .collect::<Vec<_>>();
    if include_character {
        combatants.push(TacticalCombatant {
            actor: f.actors[1],
            source: TacticalSource::Character,
            surprised: false,
        });
    }
    let groups = combatants
        .iter()
        .map(|combatant| InitiativeGroup {
            actors: vec![combatant.actor],
            request_id: RollRequestId::new(),
        })
        .collect();
    Box::pin(accept(
        f,
        TableTransportChannel::Host,
        tactical(TacticalAction::Begin {
            execution: TacticalExecutionVersion::EncounterReleaseV1,
            combatants,
            groups,
        }),
    ))
    .await;
    for (index, face) in [20, 18, 5, 3].into_iter().enumerate() {
        let channel = sources.channel(index);
        let input = raw(view(f, &channel).await, face);
        Box::pin(accept(f, channel, input)).await;
    }
    if include_character {
        let channel = TableTransportChannel::Player {
            player_id: f.players[1],
            character_id: f.characters[1],
        };
        let input = raw(view(f, &channel).await, 1);
        Box::pin(accept(f, channel, input)).await;
    }
    assert_eq!(
        view(f, &sources.channel(0))
            .await
            .tactical
            .unwrap()
            .active_actor,
        Some(sources.actors[0])
    );
    sources
}
async fn begin(f: &Fixture, sources: &Sources, caster: usize) -> TableTransportRequest {
    let channel = sources.channel(caster);
    let casting = view(f, &channel)
        .await
        .tactical
        .unwrap()
        .casting_options
        .unwrap();
    let choice = casting
        .variants
        .iter()
        .find(|variant| variant.choice.spell_id == "magic-missile")
        .unwrap()
        .choice
        .clone();
    assert_eq!(choice.resource, SpellResourceChoice::SourceFeature);
    Box::pin(accept(
        f,
        channel,
        tactical(TacticalAction::CastSpell {
            choice,
            targets: SpellTargetChoice::Entities(
                (0..6).map(|index| sources.actors[2 + index % 2]).collect(),
            ),
        }),
    ))
    .await
}
async fn response(f: &Fixture, sources: &Sources, index: usize) -> TableHitResponse<CommandId> {
    view(f, &sources.channel(index))
        .await
        .tactical
        .unwrap()
        .missile
        .unwrap()
        .responses
        .into_iter()
        .find(|response| response.actor == sources.actors[index])
        .unwrap()
}
async fn order(f: &Fixture, sources: &Sources, caster: usize, reverse: bool) {
    let channel = sources.channel(caster);
    let key = view(f, &channel)
        .await
        .tactical
        .unwrap()
        .missile
        .unwrap()
        .order
        .unwrap()
        .key;
    Box::pin(accept(
        f,
        channel,
        missile(
            key,
            TableMissileInput::Order {
                instruction: TacticalReactionOrdering {
                    ranked: if reverse {
                        vec![sources.actors[3], sources.actors[2]]
                    } else {
                        vec![sources.actors[2], sources.actors[3]]
                    },
                    unlisted: ReactionUnlistedOrder::AfterForward,
                },
            },
        ),
    ))
    .await;
}

async fn amounts(
    f: &mut Fixture,
    path: &Path,
    sources: &Sources,
    caster: usize,
    cold: bool,
) -> Box<CampaignState> {
    let before = Box::pin(state(f)).await;
    let hp: Vec<_> = sources.actors[2..]
        .iter()
        .map(|actor| before.rules.as_ref().unwrap().entities[actor].hp)
        .collect();
    drop(before);
    let mut ids = std::collections::HashSet::new();
    for ordinal in 0..6 {
        let projected = view(f, &sources.channel(caster)).await;
        let roll = projected.roll.as_ref().unwrap();
        assert!(
            ids.insert(roll.id),
            "each dart requires a distinct physical face"
        );
        assert_eq!(roll.dice, vec![DieSpec { count: 1, sides: 4 }]);
        assert!(
            projected
                .tactical
                .as_ref()
                .unwrap()
                .continuation
                .as_ref()
                .unwrap()
                .choices
                .is_empty()
        );
        let snapshot = Box::pin(state(f)).await;
        assert_eq!(
            record(&snapshot)
                .darts
                .iter()
                .filter(|dart| dart.amount.is_some())
                .count(),
            ordinal
        );
        for (index, actor) in sources.actors[2..].iter().enumerate() {
            assert_eq!(
                snapshot.rules.as_ref().unwrap().entities[actor].hp,
                hp[index],
                "no dart applies before the complete face barrier"
            );
        }
        drop(snapshot);
        let request = request(f, sources.channel(caster), raw(projected, 1)).await;
        if cold && (ordinal == 2 || ordinal == 5) {
            Box::pin(cold_step(
                f,
                path,
                if ordinal == 2 {
                    "partial-amount-collection"
                } else {
                    "final-amount-barrier"
                },
                request,
            ))
            .await;
        } else {
            Box::pin(f.runtime.submit_presented_table(request))
                .await
                .unwrap();
        }
    }
    let complete = Box::pin(state(f)).await;
    assert_eq!(record(&complete).stage, TacticalMissileStage::Impacts);
    assert!(
        record(&complete)
            .darts
            .iter()
            .all(|dart| dart.amount.is_some() && dart.completed_by.is_none())
    );
    for (index, actor) in sources.actors[2..].iter().enumerate() {
        assert_eq!(
            complete.rules.as_ref().unwrap().entities[actor].hp,
            hp[index]
        );
    }
    complete
}
async fn impacts(
    f: &mut Fixture,
    path: &Path,
    sources: &Sources,
    caster: usize,
    reverse: bool,
    cold: bool,
) {
    let mut count = 0;
    let mut stale = None;
    while Box::pin(state(f))
        .await
        .encounter
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .resolution
        .is_some()
    {
        let channel = sources.channel(caster);
        let projected = view(f, &channel).await;
        assert!(projected.roll.is_none());
        let choices = projected.tactical.unwrap().continuation.unwrap().choices;
        assert!(choices.len() >= 2);
        let mut handles = std::collections::HashSet::new();
        let mut labels = std::collections::HashSet::new();
        for choice in &choices {
            assert!(handles.insert(choice.handle));
            assert!(labels.insert(&choice.label));
            assert!(choice.label.starts_with("Dart "));
            assert!(
                choice.label.ends_with(": Red cloak") || choice.label.ends_with(": Blue cloak")
            );
        }
        if let Some(old) = stale.take() {
            let rejected = request(
                f,
                channel.clone(),
                TableTransportInput::SelectWork { handle: old },
            )
            .await;
            Box::pin(unchanged(f, rejected)).await;
        }
        let chosen = if reverse {
            choices.last().unwrap()
        } else {
            &choices[0]
        };
        stale = Some(chosen.handle);
        let selected = request(
            f,
            channel,
            TableTransportInput::SelectWork {
                handle: chosen.handle,
            },
        )
        .await;
        // Neither the Host nor a respondent receives the current turn's ordering.
        let host_choice = view(f, &TableTransportChannel::Host)
            .await
            .tactical
            .unwrap()
            .continuation
            .unwrap()
            .choices
            .into_iter()
            .find(|choice| choice.label == chosen.label)
            .unwrap();
        let forged = request(
            f,
            TableTransportChannel::Host,
            TableTransportInput::SelectWork {
                handle: host_choice.handle,
            },
        )
        .await;
        Box::pin(unchanged(f, forged)).await;
        if cold && count == 0 {
            Box::pin(cold_step(f, path, "first-impact", selected)).await;
        } else {
            Box::pin(f.runtime.submit_presented_table(selected))
                .await
                .unwrap();
        }
        count += 1;
        assert!(
            count <= 5,
            "only the final singleton may execute automatically"
        );
    }
    assert_eq!(count, 5);
}
async fn hostile_restore(f: &Fixture, accepted_impact: &CampaignState) {
    let current = Box::new(export_campaign(&f.pool, f.campaign).await.unwrap());
    let sequence = i64::try_from(accepted_impact.applied_event_sequence).unwrap();
    assert!(sequence < current.current_state.applied_event_sequence);
    let mut anchored = current.clone();
    anchored
        .snapshots
        .retain(|row| row.event_sequence != sequence);
    anchored.snapshots.push(dmd_persistence::SnapshotRow {
        campaign_id: anchored.campaign_id.clone(),
        event_sequence: sequence,
        state_schema_version: i64::from(accepted_impact.schema_version),
        state_json: accepted_impact.encode_json().unwrap(),
        created_at_utc: anchored.exported_at_utc.clone(),
    });
    anchored.snapshots.sort_by_key(|row| row.event_sequence);
    let pool = open_sqlite("sqlite::memory:").await.unwrap();
    Box::pin(runtime(pool.clone()).restore_campaign(&anchored))
        .await
        .unwrap();
    pool.close().await;
    for mutation in 0..4 {
        let mut bad = if mutation == 0 {
            anchored.clone()
        } else {
            current.clone()
        };
        match mutation {
            0 => {
                let mut changed = Box::new(accepted_impact.clone());
                changed
                    .encounter
                    .as_mut()
                    .unwrap()
                    .flow
                    .as_mut()
                    .unwrap()
                    .resolution
                    .as_mut()
                    .unwrap()
                    .missiles[0]
                    .darts[0]
                    .known_target_label = "Forged hidden identity".into();
                bad.snapshots
                    .iter_mut()
                    .find(|row| row.event_sequence == sequence)
                    .unwrap()
                    .state_json = changed.encode_json().unwrap();
            }
            1 => {
                let row = bad
                    .table_projection_history
                    .iter_mut()
                    .find_map(|row| {
                        row.changes.iter_mut().find_map(|change| {
                            change.handles.iter_mut().find(|handle| {
                                matches!(
                                    handle.capability,
                                    dmd_persistence::ProjectionCapability::MissileResponse { .. }
                                )
                            })
                        })
                    })
                    .unwrap();
                if let dmd_persistence::ProjectionCapability::MissileResponse {
                    occurrence, ..
                } = &mut row.capability
                {
                    *occurrence += 1;
                }
            }
            2 => {
                let row = bad
                    .table_transport_bindings
                    .iter_mut()
                    .find(|row| row.request_json.contains("MissileResponse"))
                    .unwrap();
                let mut input: TableTransportRequest =
                    serde_json::from_str(&row.request_json).unwrap();
                if let TableTransportInput::MissileResponse { handle, .. } = &mut input.input {
                    *handle = CommandId::new();
                }
                row.request_json = serde_json::to_string(&input).unwrap();
            }
            3 => {
                let row = bad
                    .event_journal
                    .iter_mut()
                    .find(|row| row.payload_json.contains("RespondToMissile"))
                    .unwrap();
                let mut event: TableEvent = serde_json::from_str(&row.payload_json).unwrap();
                if let TableAction::Tactical {
                    action: TacticalAction::RespondToMissile { actor, .. },
                } = &mut event.action
                {
                    *actor = EntityId::new();
                }
                row.payload_json = serde_json::to_string(&event).unwrap();
            }
            _ => unreachable!(),
        }
        assert_ne!(bad, anchored);
        let pool = open_sqlite("sqlite::memory:").await.unwrap();
        // This fresh destination contains only migration bookkeeping, so all
        // campaign tables start empty. Compare every initialized table's count,
        // including tables populated by SQL triggers and seeded schema metadata.
        let before = destination_rows(&pool).await;
        assert!(
            Box::pin(runtime(pool.clone()).restore_campaign(&bad))
                .await
                .is_err(),
            "forged missile history {mutation}"
        );
        assert_eq!(
            destination_rows(&pool).await,
            before,
            "rejected forged history leaves every destination table unchanged"
        );
        pool.close().await;
    }
}

async fn reject_changed_current_missile(f: &Fixture) {
    let original = Box::new(export_campaign(&f.pool, f.campaign).await.unwrap());
    let genuine = Box::pin(state(f)).await;
    for mutation in 0..3 {
        let mut changed = genuine.clone();
        let missile = &mut changed
            .encounter
            .as_mut()
            .unwrap()
            .flow
            .as_mut()
            .unwrap()
            .resolution
            .as_mut()
            .unwrap()
            .missiles[0];
        match mutation {
            0 => missile.darts[0].amount.as_mut().unwrap().subject = EntityId::new(),
            1 => missile.respondents[0].response.actor = missile.respondents[1].response.actor,
            2 => {
                missile.respondents[0]
                    .completed_shield
                    .as_mut()
                    .unwrap()
                    .cast
                    .plan
                    .choice
                    .spell_id = "magic-missile".into()
            }
            _ => unreachable!(),
        }
        let mut bad = original.clone();
        bad.current_state.state_json = changed.encode_json().unwrap();
        let pool = open_sqlite("sqlite::memory:").await.unwrap();
        let before = destination_rows(&pool).await;
        assert!(
            Box::pin(runtime(pool.clone()).restore_campaign(&bad))
                .await
                .is_err(),
            "changed current missile {mutation}"
        );
        assert_eq!(
            destination_rows(&pool).await,
            before,
            "rejected current image leaves every destination table unchanged"
        );
        pool.close().await;
    }
}

#[tokio::test]
async fn missile_owned_sources_keep_private_responses_and_cold_actor_bound_controls() {
    for same_owner in [false, true] {
        Box::pin(owned_source_scenario(same_owner)).await;
    }
}
async fn owned_source_scenario(same_owner: bool) {
    let directory = std::env::temp_dir().join(format!("dmd-missile-owned-{}", CampaignId::new().0));
    std::fs::create_dir_all(&directory).unwrap();
    let path = directory.join("campaign.sqlite");
    let pool = dmd_persistence::open_sqlite_path(&path).await.unwrap();
    let contract = TableContract {
        pvp_policy: "Both owners consent to this source-creature practice encounter.".into(),
        ..TableContract::default()
    };
    let mut f = Box::pin(Fixture::with_pool(contract, pool)).await;
    let sources = Box::pin(prepare(&mut f, same_owner)).await;
    Box::pin(begin(&f, &sources, 0)).await;
    let opening = Box::pin(state(&f)).await;
    assert_eq!(
        record(&opening).respondents.len(),
        2,
        "repeated targets receive one response each"
    );
    let first = response(&f, &sources, 2).await;
    let second = response(&f, &sources, 3).await;
    assert_ne!(
        first.key, second.key,
        "each actor has a distinct opaque intent capability"
    );
    assert!(!first.shield.is_empty() && !second.shield.is_empty());
    let baseline = foreign(&f, &sources).await;
    assert!(baseline.tactical.as_ref().unwrap().missile.is_none());
    let window = TacticalWorkKey {
        resolution: record(&opening).cause.id,
        occurrence: record(&opening).work.occurrence,
    };
    for input in [
        missile(first.key, TableMissileInput::Decline),
        missile(
            first.key,
            TableMissileInput::Cast {
                choice: first.shield[0].clone(),
            },
        ),
        tactical(TacticalAction::RespondToMissile {
            window,
            actor: sources.actors[2],
            accept: true,
        }),
        TableTransportInput::HitResponse {
            handle: first.key,
            decision: Box::new(TableHitInput::Respond { accept: true }),
        },
        tactical(TacticalAction::EndTurn),
    ] {
        let forged = request(&f, sources.channel(2), input).await;
        Box::pin(unchanged(&f, forged)).await;
    }
    // A second source actor owned by the same player still cannot answer for it.
    let forged = request(
        &f,
        sources.channel(3),
        missile(first.key, TableMissileInput::Respond { accept: true }),
    )
    .await;
    Box::pin(unchanged(&f, forged)).await;
    let forged = request(
        &f,
        TableTransportChannel::Host,
        missile(first.key, TableMissileInput::Respond { accept: true }),
    )
    .await;
    Box::pin(unchanged(&f, forged)).await;
    // Exercise both arrival directions independently of the selected response order.
    for index in if same_owner { [3, 2] } else { [2, 3] } {
        let response = response(&f, &sources, index).await;
        let before = foreign(&f, &sources).await;
        let intent = request(
            &f,
            sources.channel(index),
            missile(response.key, TableMissileInput::Respond { accept: true }),
        )
        .await;
        Box::pin(cold_step(&mut f, &path, "private-missile-intent", intent)).await;
        assert_eq!(
            foreign(&f, &sources).await,
            before,
            "complete unrelated DTO, revision and transcript remain unchanged by private intent"
        );
    }
    assert_eq!(foreign(&f, &sources).await, baseline);
    let offered = Box::pin(state(&f)).await;
    assert_eq!(
        offered.rules, opening.rules,
        "offering does not spend or grant Shield"
    );
    drop(offered);
    let key = view(&f, &sources.channel(0))
        .await
        .tactical
        .unwrap()
        .missile
        .unwrap()
        .order
        .unwrap()
        .key;
    let ordering = request(
        &f,
        sources.channel(0),
        missile(
            key,
            TableMissileInput::Order {
                instruction: TacticalReactionOrdering {
                    ranked: if same_owner {
                        vec![sources.actors[2], sources.actors[3]]
                    } else {
                        vec![sources.actors[3], sources.actors[2]]
                    },
                    unlisted: ReactionUnlistedOrder::AfterForward,
                },
            },
        ),
    )
    .await;
    Box::pin(cold_step(&mut f, &path, "private-response-order", ordering)).await;
    let mut accepted_shields = Vec::new();
    for index in if same_owner { [2, 3] } else { [3, 2] } {
        let selected = response(&f, &sources, index).await;
        assert!(selected.selected);
        let premature = request(&f, TableTransportChannel::Host, conclusion()).await;
        Box::pin(unchanged(&f, premature)).await;
        let other = if index == 2 { 3 } else { 2 };
        let forged = request(
            &f,
            sources.channel(other),
            missile(
                selected.key,
                TableMissileInput::Cast {
                    choice: selected.shield[0].clone(),
                },
            ),
        )
        .await;
        Box::pin(unchanged(&f, forged)).await;
        let selected_request = request(
            &f,
            sources.channel(index),
            missile(
                selected.key,
                TableMissileInput::Cast {
                    choice: selected.shield[0].clone(),
                },
            ),
        )
        .await;
        let accepted = Box::pin(cold_step(
            &mut f,
            &path,
            "selected-source-shield",
            selected_request.clone(),
        ))
        .await;
        accepted_shields.push((selected_request, accepted));
    }
    let paid = Box::pin(state(&f)).await;
    assert_eq!(record(&paid).stage, TacticalMissileStage::Amounts);
    for index in [2, 3] {
        let runtime = paid
            .rules
            .as_ref()
            .unwrap()
            .tactical_creatures
            .as_ref()
            .unwrap()
            .runtime(sources.actors[index])
            .unwrap();
        assert_eq!(
            runtime
                .limited_uses
                .iter()
                .filter(|usage| usage.feature_id == "protective-magic")
                .map(|usage| usage.spent)
                .sum::<u8>(),
            1
        );
        assert!(
            record(&paid).respondents[index - 2]
                .completed_shield
                .is_some()
        );
    }
    let impact = Box::pin(amounts(&mut f, &path, &sources, 0, true)).await;
    let premature = request(&f, TableTransportChannel::Host, conclusion()).await;
    Box::pin(unchanged(&f, premature)).await;
    Box::pin(reject_changed_current_missile(&f)).await;
    Box::pin(impacts(&mut f, &path, &sources, 0, !same_owner, true)).await;
    let finished = Box::pin(state(&f)).await;
    for actor in &sources.actors[2..] {
        assert_eq!(
            finished.rules.as_ref().unwrap().entities[actor].hp,
            81,
            "Shield prevents every dart without suppressing its raw face"
        );
    }
    let stale = request(
        &f,
        sources.channel(2),
        missile(first.key, TableMissileInput::Respond { accept: false }),
    )
    .await;
    Box::pin(unchanged(&f, stale)).await;
    Box::pin(hostile_restore(&f, &impact)).await;
    Box::pin(accept(&f, TableTransportChannel::Host, conclusion())).await;
    Box::pin(accept(
        &f,
        TableTransportChannel::Host,
        action(TableAction::SetSourceCreatureController {
            actor: sources.actors[2],
            controller: CreatureController::Host,
        }),
    ))
    .await;
    Box::pin(accept(
        &f,
        TableTransportChannel::Host,
        action(TableAction::EndSession),
    ))
    .await;
    f.session = PlaySessionId::new();
    Box::pin(accept(
        &f,
        TableTransportChannel::Host,
        action(TableAction::StartSession {
            id: f.session,
            name: "Later attendance".into(),
            participants: (0..2)
                .map(|index| SessionParticipant {
                    player_id: f.players[index],
                    character_id: Some(f.characters[index]),
                    attendance: AttendanceStatus::Present,
                })
                .collect(),
        }),
    ))
    .await;
    Box::pin(reopen(&mut f, &path)).await;
    for (original, expected) in accepted_shields {
        let before = Box::new(export_campaign(&f.pool, f.campaign).await.unwrap());
        assert_eq!(
            Box::pin(f.runtime.submit_presented_table(original.clone()))
                .await
                .unwrap(),
            expected,
            "accepted opaque casts recover before changed ownership and session admission"
        );
        let mut changed = original;
        changed.command_id = CommandId::new();
        Box::pin(unchanged(&f, changed)).await;
        let mut after = Box::new(export_campaign(&f.pool, f.campaign).await.unwrap());
        after.exported_at_utc = before.exported_at_utc.clone();
        assert_eq!(after, before);
    }
    f.pool.close().await;
    drop(f);
    sqlite_test_cleanup::remove_closed_directory(&directory)
        .await
        .unwrap();
}

#[tokio::test]
async fn missile_zero_one_two_eligible_sources_keep_uniform_ordering_and_private_acknowledgments() {
    let directory =
        std::env::temp_dir().join(format!("dmd-missile-eligibility-{}", CampaignId::new().0));
    std::fs::create_dir_all(&directory).unwrap();
    let path = directory.join("common-prefix.sqlite");
    let pool = dmd_persistence::open_sqlite_path(&path).await.unwrap();
    let mut f = Box::pin(Fixture::with_pool(TableContract::default(), pool)).await;
    let sources = Box::pin(prepare(&mut f, false)).await;
    let prefix = Box::new(export_campaign(&f.pool, f.campaign).await.unwrap());
    let mut ordering = None;
    // Different real precursor histories may legitimately produce different
    // public consequences. We compare whole foreign views around each hidden
    // intent within a history, and only potential ordering across histories.
    for spent in 0..=2 {
        let branch_path = directory.join(format!("reaction-precursor-{spent}.sqlite"));
        let pool = dmd_persistence::open_sqlite_path(&branch_path)
            .await
            .unwrap();
        let app = runtime(pool.clone());
        Box::pin(app.restore_campaign(&prefix)).await.unwrap();
        let mut branch = Fixture {
            runtime: app,
            pool,
            campaign: f.campaign,
            players: f.players,
            characters: f.characters,
            actors: f.actors,
            session: f.session,
        };
        Box::pin(begin(&branch, &sources, 0)).await;
        Box::pin(order(&branch, &sources, 0, false)).await;
        for index in [2, 3] {
            let offered = response(&branch, &sources, index).await;
            Box::pin(accept(
                &branch,
                sources.channel(index),
                missile(
                    offered.key,
                    TableMissileInput::Respond {
                        accept: index - 2 < spent || (spent == 0 && index == 2),
                    },
                ),
            ))
            .await;
        }
        if spent == 0 {
            let selected = response(&branch, &sources, 2).await;
            assert!(selected.selected);
            let decline = request(
                &branch,
                sources.channel(2),
                missile(selected.key, TableMissileInput::Decline),
            )
            .await;
            Box::pin(cold_step(
                &mut branch,
                &branch_path,
                "selected-source-decline",
                decline,
            ))
            .await;
        }
        for index in 2..2 + spent {
            let selected = response(&branch, &sources, index).await;
            assert!(selected.selected);
            Box::pin(accept(
                &branch,
                sources.channel(index),
                missile(
                    selected.key,
                    TableMissileInput::Cast {
                        choice: selected.shield[0].clone(),
                    },
                ),
            ))
            .await;
        }
        Box::pin(amounts(&mut branch, &branch_path, &sources, 0, false)).await;
        Box::pin(impacts(
            &mut branch,
            &branch_path,
            &sources,
            0,
            false,
            false,
        ))
        .await;
        Box::pin(accept(
            &branch,
            sources.channel(0),
            tactical(TacticalAction::EndTurn),
        ))
        .await;
        assert_eq!(
            view(&branch, &sources.channel(1))
                .await
                .tactical
                .unwrap()
                .active_actor,
            Some(sources.actors[1])
        );
        // Neither Mage has started a turn: actual paid Reactions remain spent.
        Box::pin(begin(&branch, &sources, 1)).await;
        let order_view = view(&branch, &sources.channel(1))
            .await
            .tactical
            .unwrap()
            .missile
            .unwrap();
        let potential = order_view.order.as_ref().unwrap();
        let visible = (potential.actor, potential.participants.clone());
        if let Some(expected) = &ordering {
            assert_eq!(
                &visible, expected,
                "ordering never exposes the eligible/responding set"
            );
        } else {
            ordering = Some(visible);
        }
        let mut eligible = 0;
        for index in [2, 3] {
            let offered = response(&branch, &sources, index).await;
            eligible += usize::from(!offered.shield.is_empty());
            assert!(!offered.selected);
            let before = foreign(&branch, &sources).await;
            Box::pin(accept(
                &branch,
                sources.channel(index),
                missile(offered.key, TableMissileInput::Respond { accept: false }),
            ))
            .await;
            assert_eq!(
                foreign(&branch, &sources).await,
                before,
                "zero/one/two private eligibility cannot change unrelated DTO/revision/transcript on acknowledgment"
            );
        }
        assert_eq!(eligible, 2 - spent);
        assert!(
            view(&branch, &sources.channel(1)).await.roll.is_none(),
            "even zero possible responses does not infer an order or bypass controller consent"
        );
        let host = view(&branch, &TableTransportChannel::Host)
            .await
            .tactical
            .unwrap()
            .missile
            .unwrap();
        assert!(host.order.is_none() && host.responses.is_empty());
        let delegate = view(&branch, &sources.channel(1))
            .await
            .tactical
            .unwrap()
            .missile
            .unwrap()
            .delegate
            .unwrap();
        Box::pin(accept(
            &branch,
            sources.channel(1),
            missile(delegate, TableMissileInput::Delegate),
        ))
        .await;
        let host = view(&branch, &TableTransportChannel::Host)
            .await
            .tactical
            .unwrap()
            .missile
            .unwrap();
        assert!(host.responses.is_empty());
        let ordering = request(
            &branch,
            TableTransportChannel::Host,
            missile(
                host.order.unwrap().key,
                TableMissileInput::Order {
                    instruction: TacticalReactionOrdering {
                        ranked: vec![sources.actors[3], sources.actors[2]],
                        unlisted: ReactionUnlistedOrder::AfterReverse,
                    },
                },
            ),
        )
        .await;
        Box::pin(cold_step(
            &mut branch,
            &branch_path,
            "delegated-empty-response-order",
            ordering,
        ))
        .await;
        Box::pin(amounts(&mut branch, &branch_path, &sources, 1, false)).await;
        Box::pin(impacts(&mut branch, &branch_path, &sources, 1, true, false)).await;
        // Response delegation was consumed at this exact window; Host could not
        // use it for any impact choices in impacts(). Already active Shields from
        // the precursor still protect their actor for their full duration.
        let finished = Box::pin(state(&branch)).await;
        for index in [2, 3] {
            let expected = if index - 2 < spent { 81 } else { 69 };
            assert_eq!(
                finished.rules.as_ref().unwrap().entities[&sources.actors[index]].hp,
                expected
            );
        }
        branch.pool.close().await;
        drop(branch);
    }
    f.pool.close().await;
    drop(f);
    sqlite_test_cleanup::remove_closed_directory(&directory)
        .await
        .unwrap();
}

#[tokio::test]
async fn missile_each_dart_finishes_its_owned_concentration_child_before_next_impact() {
    let directory =
        std::env::temp_dir().join(format!("dmd-missile-concentration-{}", CampaignId::new().0));
    std::fs::create_dir_all(&directory).unwrap();
    let path = directory.join("campaign.sqlite");
    let pool = dmd_persistence::open_sqlite_path(&path).await.unwrap();
    let mut f = Box::pin(Fixture::with_pool(TableContract::default(), pool)).await;
    let sources = Box::pin(prepare_sources(&mut f, true, true)).await;
    for index in [0, 1, 2] {
        Box::pin(accept(
            &f,
            sources.channel(index),
            tactical(TacticalAction::EndTurn),
        ))
        .await;
    }
    let casting = view(&f, &sources.channel(3))
        .await
        .tactical
        .unwrap()
        .casting_options
        .unwrap();
    let hold = casting
        .variants
        .iter()
        .find(|variant| variant.choice.spell_id == "hold-person")
        .unwrap()
        .choice
        .clone();
    Box::pin(accept(
        &f,
        sources.channel(3),
        tactical(TacticalAction::CastSpell {
            choice: hold,
            targets: SpellTargetChoice::Entities(vec![sources.actors[2]]),
        }),
    ))
    .await;
    let failed = raw(view(&f, &sources.channel(2)).await, 1);
    Box::pin(accept(&f, sources.channel(2), failed)).await;
    assert!(
        Box::pin(state(&f)).await.rules.as_ref().unwrap().entities[&sources.actors[3]]
            .concentration
            .is_some()
    );
    Box::pin(accept(
        &f,
        sources.channel(3),
        tactical(TacticalAction::EndTurn),
    ))
    .await;
    let casting = view(&f, &sources.channel(0))
        .await
        .tactical
        .unwrap()
        .casting_options
        .unwrap();
    let choice = casting
        .variants
        .iter()
        .find(|variant| variant.choice.spell_id == "magic-missile")
        .unwrap()
        .choice
        .clone();
    Box::pin(accept(
        &f,
        sources.channel(0),
        tactical(TacticalAction::CastSpell {
            choice,
            targets: SpellTargetChoice::Entities(vec![sources.actors[3]; 6]),
        }),
    ))
    .await;
    Box::pin(order(&f, &sources, 0, true)).await;
    let response = response(&f, &sources, 3).await;
    assert!(response.shield.is_empty());
    Box::pin(accept(
        &f,
        sources.channel(3),
        missile(response.key, TableMissileInput::Respond { accept: false }),
    ))
    .await;
    let before = Box::pin(amounts(&mut f, &path, &sources, 0, true)).await;
    let starting_hp = before.rules.as_ref().unwrap().entities[&sources.actors[3]].hp;
    let mut saves = 0;
    let mut choices = 0;
    let mut raw_ids = std::collections::HashSet::new();
    while Box::pin(state(&f))
        .await
        .encounter
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .resolution
        .is_some()
    {
        let saver = view(&f, &sources.channel(3)).await;
        if let Some(roll) = &saver.roll {
            assert!(roll.reason.contains("concentration"));
            assert!(raw_ids.insert(roll.id));
            assert!(view(&f, &sources.channel(0)).await.roll.is_none());
            let during = Box::pin(state(&f)).await;
            assert_eq!(
                during.rules.as_ref().unwrap().entities[&sources.actors[3]].hp,
                starting_hp - 2 * (saves + 1)
            );
            assert_eq!(
                record(&during)
                    .darts
                    .iter()
                    .filter(|dart| dart.completed_by.is_some())
                    .count(),
                (saves + 1) as usize
            );
            let input = raw(saver, 20);
            let wrong = request(&f, sources.channel(0), input.clone()).await;
            Box::pin(unchanged(&f, wrong)).await;
            let next = view(&f, &sources.channel(0))
                .await
                .tactical
                .unwrap()
                .continuation
                .unwrap();
            assert!(
                next.choices.is_empty(),
                "the outer owner cannot skip the pending concentration child"
            );
            let request = request(&f, sources.channel(3), input).await;
            if saves == 0 {
                Box::pin(failed_concentration_branch(
                    &f,
                    &directory,
                    &sources,
                    starting_hp,
                ))
                .await;
            }
            if saves == 0 || saves == 4 {
                Box::pin(cold_step(
                    &mut f,
                    &path,
                    "impact-concentration-child",
                    request,
                ))
                .await;
            } else {
                Box::pin(f.runtime.submit_presented_table(request))
                    .await
                    .unwrap();
            }
            saves += 1;
        } else {
            let active = view(&f, &sources.channel(0))
                .await
                .tactical
                .unwrap()
                .continuation
                .unwrap();
            assert!(active.choices.len() >= 2);
            let last = active.choices.last().unwrap();
            assert!(last.label.ends_with(": Blue cloak"));
            Box::pin(accept(
                &f,
                sources.channel(0),
                TableTransportInput::SelectWork {
                    handle: last.handle,
                },
            ))
            .await;
            choices += 1;
        }
        assert!(saves <= 6 && choices <= 5);
    }
    assert_eq!(
        (choices, saves),
        (5, 6),
        "the last singleton follows the foreign-owned save without borrowing its ordering authority"
    );
    let finished = Box::pin(state(&f)).await;
    assert_eq!(
        finished.rules.as_ref().unwrap().entities[&sources.actors[3]].hp,
        starting_hp - 12
    );
    assert!(
        finished.rules.as_ref().unwrap().entities[&sources.actors[3]]
            .concentration
            .is_some()
    );
    let exported = Box::new(export_campaign(&f.pool, f.campaign).await.unwrap());
    let pool = open_sqlite("sqlite::memory:").await.unwrap();
    Box::pin(runtime(pool.clone()).restore_campaign(&exported))
        .await
        .unwrap();
    pool.close().await;
    f.pool.close().await;
    drop(f);
    sqlite_test_cleanup::remove_closed_directory(&directory)
        .await
        .unwrap();
}

async fn failed_concentration_branch(
    f: &Fixture,
    directory: &Path,
    sources: &Sources,
    starting_hp: u32,
) {
    let exported = Box::new(export_campaign(&f.pool, f.campaign).await.unwrap());
    let path = directory.join("failed-first-concentration.sqlite");
    let pool = dmd_persistence::open_sqlite_path(&path).await.unwrap();
    let app = runtime(pool.clone());
    Box::pin(app.restore_campaign(&exported)).await.unwrap();
    let mut branch = Fixture {
        runtime: app,
        pool,
        campaign: f.campaign,
        players: f.players,
        characters: f.characters,
        actors: f.actors,
        session: f.session,
    };
    let failed = raw(view(&branch, &sources.channel(3)).await, 1);
    let failed = request(&branch, sources.channel(3), failed).await;
    Box::pin(cold_step(
        &mut branch,
        &path,
        "failed-concentration-child",
        failed,
    ))
    .await;
    let current = Box::pin(state(&branch)).await;
    assert!(
        current.rules.as_ref().unwrap().entities[&sources.actors[3]]
            .concentration
            .is_none()
    );
    assert!(
        !dmd_rules::active_conditions(current.rules.as_ref().unwrap(), sources.actors[2])
            .contains(&Condition::Paralyzed)
    );
    assert_eq!(
        record(&current)
            .darts
            .iter()
            .filter(|dart| dart.completed_by.is_none())
            .count(),
        5
    );
    for _ in 0..4 {
        assert!(
            view(&branch, &sources.channel(3)).await.roll.is_none(),
            "ending concentration removes its future saves, not the remaining strikes"
        );
        Box::pin(reverse_next(&branch, sources.channel(0))).await;
    }
    let final_state = Box::pin(state(&branch)).await;
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
    assert_eq!(
        final_state.rules.as_ref().unwrap().entities[&sources.actors[3]].hp,
        starting_hp - 12
    );
    branch.pool.close().await;
}

async fn collect_fixed_faces(f: &Fixture, channel: TableTransportChannel, faces: &[u16]) {
    let before = Box::pin(state(f)).await;
    let mut ids = std::collections::HashSet::new();
    for face in faces {
        let projected = view(f, &channel).await;
        assert!(ids.insert(projected.roll.as_ref().unwrap().id));
        let input = raw(projected, *face);
        Box::pin(accept(f, channel.clone(), input)).await;
        let after = Box::pin(state(f)).await;
        for (actor, mechanics) in &before.rules.as_ref().unwrap().entities {
            assert_eq!(
                after.rules.as_ref().unwrap().entities[actor].hp,
                mechanics.hp,
                "the all-face barrier includes unconscious characters"
            );
        }
    }
}
async fn reverse_next(f: &Fixture, channel: TableTransportChannel) {
    let choices = view(f, &channel)
        .await
        .tactical
        .unwrap()
        .continuation
        .unwrap()
        .choices;
    assert!(choices.len() >= 2);
    Box::pin(accept(
        f,
        channel,
        TableTransportInput::SelectWork {
            handle: choices.last().unwrap().handle,
        },
    ))
    .await;
}
#[tokio::test]
async fn missile_real_first_aid_then_separate_failures_retains_committed_darts_after_death() {
    let directory = std::env::temp_dir().join(format!("dmd-missile-death-{}", CampaignId::new().0));
    std::fs::create_dir_all(&directory).unwrap();
    let path = directory.join("campaign.sqlite");
    let pool = dmd_persistence::open_sqlite_path(&path).await.unwrap();
    let mut f = Box::pin(Fixture::with_pool(TableContract::default(), pool)).await;
    let sources = Box::pin(prepare_with_character(&mut f, true, false, true)).await;
    let target = f.actors[1];
    let owner = TableTransportChannel::Player {
        player_id: f.players[1],
        character_id: f.characters[1],
    };
    assert_eq!(
        Box::pin(state(&f)).await.rules.as_ref().unwrap().entities[&target].hp,
        12
    );
    let options = view(&f, &sources.channel(0))
        .await
        .tactical
        .unwrap()
        .casting_options
        .unwrap();
    let choice = options
        .variants
        .iter()
        .find(|variant| variant.choice.spell_id == "magic-missile")
        .unwrap()
        .choice
        .clone();
    Box::pin(accept(
        &f,
        sources.channel(0),
        tactical(TacticalAction::CastSpell {
            choice,
            targets: SpellTargetChoice::Entities(vec![
                target,
                target,
                target,
                sources.actors[3],
                sources.actors[3],
                sources.actors[3],
            ]),
        }),
    ))
    .await;
    Box::pin(order(&f, &sources, 0, true)).await;
    for (channel, actor) in [
        (owner.clone(), target),
        (sources.channel(3), sources.actors[3]),
    ] {
        let response = view(&f, &channel)
            .await
            .tactical
            .unwrap()
            .missile
            .unwrap()
            .responses
            .into_iter()
            .find(|response| response.actor == actor)
            .unwrap();
        Box::pin(accept(
            &f,
            channel,
            missile(response.key, TableMissileInput::Respond { accept: false }),
        ))
        .await;
    }
    Box::pin(collect_fixed_faces(&f, sources.channel(0), &[4; 6])).await;
    for _ in 0..5 {
        Box::pin(reverse_next(&f, sources.channel(0))).await;
    }
    let down = Box::pin(state(&f)).await;
    let mechanics = &down.rules.as_ref().unwrap().entities[&target];
    assert_eq!(mechanics.hp, 0);
    assert!(!mechanics.death.dead && !mechanics.death.stable);
    assert_eq!(mechanics.death.failures, 0);
    drop(down);
    for index in [0, 1, 2] {
        Box::pin(accept(
            &f,
            sources.channel(index),
            tactical(TacticalAction::EndTurn),
        ))
        .await;
    }
    Box::pin(accept(
        &f,
        sources.channel(3),
        tactical(TacticalAction::Move {
            path: vec![
                TacticalMoveStep {
                    destination: SpatialPoint {
                        x: 110,
                        y: 10,
                        z: 0,
                    },
                    mode: MovementMode::Walk,
                },
                TacticalMoveStep {
                    destination: SpatialPoint {
                        x: 120,
                        y: 10,
                        z: 0,
                    },
                    mode: MovementMode::Walk,
                },
            ],
        }),
    ))
    .await;
    Box::pin(accept(
        &f,
        sources.channel(3),
        tactical(TacticalAction::FirstAid {
            target,
            purpose: MedicinePurpose::Stabilize,
        }),
    ))
    .await;
    let medicine = raw(view(&f, &sources.channel(3)).await, 20);
    Box::pin(accept(&f, sources.channel(3), medicine)).await;
    let recovery = raw(view(&f, &owner).await, 2);
    let recovery = request(&f, owner.clone(), recovery).await;
    Box::pin(cold_step(
        &mut f,
        &path,
        "genuine-stable-recovery",
        recovery,
    ))
    .await;
    assert!(
        Box::pin(state(&f)).await.rules.as_ref().unwrap().entities[&target]
            .death
            .stable
    );
    Box::pin(accept(
        &f,
        sources.channel(3),
        tactical(TacticalAction::EndTurn),
    ))
    .await;
    Box::pin(accept(&f, owner.clone(), tactical(TacticalAction::EndTurn))).await;
    let options = view(&f, &sources.channel(0))
        .await
        .tactical
        .unwrap()
        .casting_options
        .unwrap();
    let choice = options
        .variants
        .iter()
        .find(|variant| variant.choice.spell_id == "magic-missile")
        .unwrap()
        .choice
        .clone();
    Box::pin(accept(
        &f,
        sources.channel(0),
        tactical(TacticalAction::CastSpell {
            choice,
            targets: SpellTargetChoice::Entities(vec![target; 6]),
        }),
    ))
    .await;
    Box::pin(order(&f, &sources, 0, false)).await;
    let response = view(&f, &owner)
        .await
        .tactical
        .unwrap()
        .missile
        .unwrap()
        .responses
        .into_iter()
        .find(|response| response.actor == target)
        .unwrap();
    assert!(response.shield.is_empty());
    Box::pin(accept(
        &f,
        owner.clone(),
        missile(response.key, TableMissileInput::Respond { accept: false }),
    ))
    .await;
    Box::pin(collect_fixed_faces(
        &f,
        sources.channel(0),
        &[1, 2, 3, 4, 1, 2],
    ))
    .await;
    let all_faces = Box::pin(state(&f)).await;
    assert!(
        all_faces.rules.as_ref().unwrap().entities[&target]
            .death
            .stable
    );
    let keys = record(&all_faces)
        .darts
        .iter()
        .map(|dart| dart.amount.unwrap())
        .collect::<Vec<_>>();
    for failures in 1..=3 {
        Box::pin(reverse_next(&f, sources.channel(0))).await;
        let current = Box::pin(state(&f)).await;
        let mechanics = &current.rules.as_ref().unwrap().entities[&target];
        assert_eq!(mechanics.hp, 0);
        assert!(!mechanics.death.stable);
        assert_eq!(
            mechanics.death.failures,
            if failures == 3 { 0 } else { failures },
            "each dart is one ordinary damage instance, never a critical attack"
        );
        assert_eq!(mechanics.death.dead, failures == 3);
        assert_eq!(
            record(&current)
                .darts
                .iter()
                .filter(|dart| dart.completed_by.is_some())
                .count(),
            usize::from(failures)
        );
        assert_eq!(
            record(&current)
                .darts
                .iter()
                .map(|dart| dart.amount.unwrap())
                .collect::<Vec<_>>(),
            keys
        );
    }
    let post_death = view(&f, &sources.channel(0))
        .await
        .tactical
        .unwrap()
        .continuation
        .unwrap()
        .choices;
    assert_eq!(
        post_death.len(),
        3,
        "death cannot erase the remaining committed strikes"
    );
    let remaining = request(
        &f,
        sources.channel(0),
        TableTransportInput::SelectWork {
            handle: post_death[0].handle,
        },
    )
    .await;
    Box::pin(cold_step(
        &mut f,
        &path,
        "committed-dart-after-death",
        remaining,
    ))
    .await;
    Box::pin(reverse_next(&f, sources.channel(0))).await;
    let finished = Box::pin(state(&f)).await;
    let mechanics = &finished.rules.as_ref().unwrap().entities[&target];
    assert_eq!(mechanics.hp, 0);
    assert_eq!(
        mechanics.death.failures, 0,
        "death clears counters; later darts do not create corpse failures"
    );
    assert!(mechanics.death.dead);
    assert_eq!(finished.entities[&target].existence, EntityExistence::Dead);
    assert_eq!(
        finished.characters[&f.characters[1]].status,
        CharacterStatus::Dead
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
    f.pool.close().await;
    drop(f);
    sqlite_test_cleanup::remove_closed_directory(&directory)
        .await
        .unwrap();
}
