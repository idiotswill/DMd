//! Actual table actors, normal source creation, opaque controls and a real SQLite
//! file. Every accepted Shove stage is cold reopened, mirrored and retried.
use super::*;
use dmd_rules::tactical::TacticalAction;
#[path = "table_shove_air.rs"]
mod air;
#[path = "table_shove_ledge.rs"]
mod ledge;

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
async fn view(f: &Fixture, host: bool) -> TablePresentedView {
    f.runtime
        .presented_table_view(
            f.campaign,
            if host {
                TableViewer::Host
            } else {
                TableViewer::Player(f.players[0])
            },
        )
        .await
        .unwrap()
}
fn action(action: TacticalAction) -> TableTransportInput {
    TableTransportInput::Action(Box::new(TableAction::Tactical { action }))
}
async fn request(f: &Fixture, host: bool, input: TableTransportInput) -> TableTransportRequest {
    TableTransportRequest {
        version: TABLE_SOURCE_TRANSPORT_VERSION,
        command_id: CommandId::new(),
        campaign_id: f.campaign,
        session_id: Some(f.session),
        revision: view(f, host).await.revision,
        channel: if host {
            TableTransportChannel::Host
        } else {
            TableTransportChannel::Player {
                player_id: f.players[0],
                character_id: f.characters[0],
            }
        },
        input,
    }
}
async fn decision(f: &Fixture, host: bool, decision: TableShoveInput) -> TableTransportRequest {
    let key = view(f, host).await.tactical.unwrap().shove.unwrap().key;
    request(
        f,
        host,
        TableTransportInput::ShoveDecision {
            handle: key,
            decision: Box::new(decision),
        },
    )
    .await
}
async fn cold(f: &mut Fixture, url: &str, request: TableTransportRequest) {
    let before = export_campaign(&f.pool, f.campaign).await.unwrap();
    let mirror_pool = open_sqlite("sqlite::memory:").await.unwrap();
    let mirror = runtime(mirror_pool.clone());
    Box::pin(mirror.restore_campaign(&before)).await.unwrap();
    f.pool.close().await;
    f.pool = open_sqlite(url).await.unwrap();
    f.runtime = runtime(f.pool.clone());
    let accepted = Box::pin(f.runtime.submit_presented_table(request.clone()))
        .await
        .unwrap();
    Box::pin(mirror.submit_presented_table(request.clone()))
        .await
        .unwrap();
    assert_eq!(
        mirror.open_campaign(f.campaign).await.unwrap().state(),
        &state(f).await
    );
    mirror_pool.close().await;
    f.pool.close().await;
    f.pool = open_sqlite(url).await.unwrap();
    f.runtime = runtime(f.pool.clone());
    let saved = export_campaign(&f.pool, f.campaign).await.unwrap();
    assert_eq!(
        Box::pin(f.runtime.submit_presented_table(request.clone()))
            .await
            .unwrap(),
        accepted
    );
    let mut changed = request;
    changed.input = action(TacticalAction::Dodge);
    assert!(
        Box::pin(f.runtime.submit_presented_table(changed))
            .await
            .is_err()
    );
    let mut after = export_campaign(&f.pool, f.campaign).await.unwrap();
    after.exported_at_utc = saved.exported_at_utc.clone();
    assert_eq!(after, saved, "retry changed durable state/history");
}
async fn reject(f: &Fixture, request: TableTransportRequest) {
    let before = export_campaign(&f.pool, f.campaign).await.unwrap();
    assert!(
        Box::pin(f.runtime.submit_presented_table(request))
            .await
            .is_err()
    );
    let mut after = export_campaign(&f.pool, f.campaign).await.unwrap();
    after.exported_at_utc = before.exported_at_utc.clone();
    assert_eq!(after, before);
}
async fn reject_forged_source(f: &Fixture) {
    let mut export = export_campaign(&f.pool, f.campaign).await.unwrap();
    let mut current = state(f).await;
    current
        .encounter
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .resolution
        .as_mut()
        .unwrap()
        .shove
        .as_mut()
        .unwrap()
        .difficulty += 1;
    export.current_state.state_json = current.encode_json().unwrap();
    let pool = open_sqlite("sqlite::memory:").await.unwrap();
    assert!(
        Box::pin(runtime(pool.clone()).restore_campaign(&export))
            .await
            .is_err()
    );
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM campaign_state_current")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 0);
    pool.close().await;
}
async fn exercise(f: &mut Fixture, url: &str, push: bool, falling: bool) {
    view(f, false).await;
    let target = if falling {
        Box::pin(ledge::prepare(f)).await
    } else {
        Box::pin(table_attack_cases::prepare_at(
            f,
            SpatialPoint { x: 20, y: 10, z: 0 },
        ))
        .await
    };
    // The original scene helper intentionally creates an Autonomous source.
    // This slice needs explicit Host ownership, established by real table actions.
    let enable = request(
        f,
        true,
        TableTransportInput::Action(Box::new(TableAction::EnableSourceActorAccess {
            adopted: vec![],
        })),
    )
    .await;
    Box::pin(cold(f, url, enable)).await;
    let assign = request(
        f,
        true,
        TableTransportInput::Action(Box::new(TableAction::SetSourceCreatureController {
            actor: target,
            controller: CreatureController::Host,
        })),
    )
    .await;
    Box::pin(cold(f, url, assign)).await;
    let original = state(f).await;
    let original_hp = original.rules.as_ref().unwrap().entities[&target].hp;
    let begin = request(f, false, action(TacticalAction::Shove { target })).await;
    Box::pin(cold(f, url, begin)).await;
    Box::pin(reject_forged_source(f)).await;
    let host = view(f, true).await;
    assert_eq!(
        host.tactical
            .as_ref()
            .unwrap()
            .shove
            .as_ref()
            .unwrap()
            .stage,
        TacticalShoveStage::SaveChoice
    );
    let owner = view(f, false).await;
    assert!(owner.tactical.as_ref().unwrap().shove.is_none());
    assert!(owner.roll.is_none());
    let unrelated = f
        .runtime
        .presented_table_view(f.campaign, TableViewer::Player(f.players[1]))
        .await
        .unwrap();
    assert!(unrelated.tactical.as_ref().unwrap().shove.is_none());
    let choose = decision(
        f,
        true,
        TableShoveInput::Save {
            ability: ShoveSaveAbility::Strength,
        },
    )
    .await;
    let mut leaked = request(f, false, choose.input.clone()).await;
    Box::pin(reject(f, leaked.clone())).await;
    leaked.input = action(TacticalAction::ChooseShoveSave {
        ability: ShoveSaveAbility::Strength,
    });
    Box::pin(reject(f, leaked)).await;
    Box::pin(cold(f, url, choose.clone())).await;
    let mut stale = choose;
    stale.command_id = CommandId::new();
    stale.revision = view(f, true).await.revision;
    Box::pin(reject(f, stale)).await;
    let raw = view(f, true).await.roll.unwrap();
    assert_eq!(raw.modifier, -1);
    assert_ne!(
        raw.id,
        state(f).await.rules.unwrap().pending.unwrap().request.id
    );
    let report = request(
        f,
        true,
        action(TacticalAction::SubmitRoll {
            result: RollResult {
                request_id: raw.id,
                source: RollSource::Physical,
                dice: vec![DieResult {
                    sides: 20,
                    value: 1,
                }],
            },
        }),
    )
    .await;
    Box::pin(cold(f, url, report)).await;
    assert!(
        view(f, true).await.tactical.unwrap().shove.is_none(),
        "host cannot choose an owned consequence"
    );
    let owned = view(f, false).await.tactical.unwrap().shove.unwrap();
    assert_eq!(owned.stage, TacticalShoveStage::OutcomeChoice);
    let json = serde_json::to_string(&owned).unwrap();
    for hidden in [
        "difficulty",
        "condition_immunities",
        "goblin-warrior",
        "Private sentry",
        "controller",
        "armor_class",
    ] {
        assert!(!json.contains(hidden));
    }
    let choice = if push {
        ShoveChoice::Push {
            destination: SpatialPoint {
                x: 30,
                y: 10,
                z: if falling { 50 } else { 0 },
            },
        }
    } else {
        ShoveChoice::Prone
    };
    let choose = decision(f, false, TableShoveInput::Outcome { choice }).await;
    let mut wrong_stage = choose.clone();
    wrong_stage.command_id = CommandId::new();
    if let TableTransportInput::ShoveDecision { decision, .. } = &mut wrong_stage.input {
        **decision = TableShoveInput::Save {
            ability: ShoveSaveAbility::Strength,
        };
    }
    Box::pin(reject(f, wrong_stage)).await;
    Box::pin(cold(f, url, choose)).await;
    if push {
        assert_eq!(
            state(f)
                .await
                .encounter
                .unwrap()
                .participant(target)
                .unwrap()
                .position
                .x,
            20
        );
        assert!(view(f, false).await.tactical.unwrap().shove.is_none());
        let ruling = decision(
            f,
            true,
            TableShoveInput::RulePush {
                ruling: ShoveGeometryRuling::CommitExactPush,
            },
        )
        .await;
        Box::pin(cold(f, url, ruling)).await;
    }
    if falling {
        let current = state(f).await;
        let r = current
            .encounter
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .resolution
            .as_ref()
            .unwrap();
        assert_eq!(r.falls[0].stage, TacticalFallStage::LandingChoice);
        assert!(matches!(r.falls[0].cause, TacticalFallCause::Shove { .. }));
        assert_eq!((r.falls[0].path.from.z, r.falls[0].path.to.z), (50, 10));
        let json = serde_json::to_string(&view(f, false).await).unwrap();
        assert!(!json.contains("private-landing-water"));
        let landing = request(
            f,
            true,
            action(TacticalAction::ChooseLiquidLanding {
                choice: Some(LiquidLandingChoice::Athletics),
            }),
        )
        .await;
        Box::pin(cold(f, url, landing)).await;
        for (sides, values) in [(20, vec![1]), (6, vec![2, 3])] {
            let roll = view(f, true).await.roll.unwrap();
            assert_eq!(
                roll.dice,
                vec![DieSpec {
                    count: values.len() as u16,
                    sides
                }]
            );
            let report = request(
                f,
                true,
                action(TacticalAction::SubmitRoll {
                    result: RollResult {
                        request_id: roll.id,
                        source: RollSource::Physical,
                        dice: values
                            .into_iter()
                            .map(|value| DieResult { sides, value })
                            .collect(),
                    },
                }),
            )
            .await;
            Box::pin(cold(f, url, report)).await;
        }
    }
    let finished = state(f).await;
    let rules = finished.rules.as_ref().unwrap();
    assert_eq!(
        rules.entities[&target].hp,
        original_hp - if falling { 5 } else { 0 }
    );
    assert_eq!(rules.entities[&target].prone, !push || falling);
    assert_eq!(
        finished
            .encounter
            .as_ref()
            .unwrap()
            .participant(target)
            .unwrap()
            .position
            .x,
        if push { 30 } else { 20 }
    );
    assert_eq!(
        rules.tactical_inventory,
        original.rules.as_ref().unwrap().tactical_inventory
    );
    assert!(rules.timing.as_ref().unwrap().action_spent);
    assert_eq!(
        rules.timing.as_ref().unwrap().reactions_spent,
        if falling { vec![target] } else { vec![] }
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
    let saves = rules.rolls.iter().filter(|r| matches!(r.purpose, PendingPurpose::TacticalResolution { key, .. } if key.role == TacticalRollRole::ShoveSave)).collect::<Vec<_>>();
    assert_eq!(saves.len(), 1);
    assert_eq!(saves[0].result.dice[0].value, 1);
    let exported = export_campaign(&f.pool, f.campaign).await.unwrap();
    let mirror_pool = open_sqlite("sqlite::memory:").await.unwrap();
    let mirror = runtime(mirror_pool.clone());
    Box::pin(mirror.restore_campaign(&exported)).await.unwrap();
    assert_eq!(
        mirror.open_campaign(f.campaign).await.unwrap().state(),
        &finished
    );
    mirror_pool.close().await;
}

#[tokio::test]
async fn owned_shove_prone_and_push_preserve_real_cold_sqlite_capabilities_raw_faces_and_retries() {
    for (push, falling) in [(false, false), (true, false), (true, true)] {
        let directory = std::env::temp_dir().join(format!("dmd-shove-{}", CampaignId::new().0));
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
        Box::pin(exercise(&mut f, &url, push, falling)).await;
        f.pool.close().await;
        drop(f);
        sqlite_test_cleanup::remove_closed_directory(&directory)
            .await
            .unwrap();
    }
}

#[tokio::test]
async fn current_shove_releases_to_actorless_time_and_retains_exact_old_decision_retries() {
    let directory = std::env::temp_dir().join(format!("dmd-shove-release-{}", CampaignId::new().0));
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
    // The existing driver admits the real source, pays the Shove, reports its
    // physical save and chooses Prone through actual opaque ownership handles.
    Box::pin(exercise(&mut f, &url, false, false)).await;
    let after_shove = state(&f).await;
    let flow = after_shove
        .encounter
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap();
    assert_eq!(
        flow.version,
        TacticalExecutionVersion::ReleasedTimeV1.flow_version()
    );
    assert_eq!(flow.phase, TacticalPhase::Active);
    assert!(flow.resolution.is_none());
    let original = export_campaign(&f.pool, f.campaign).await.unwrap();
    let decisions = original
        .table_transport_bindings
        .iter()
        .filter_map(|binding| {
            let request: TableTransportRequest =
                serde_json::from_str(&binding.request_json).unwrap();
            matches!(request.input, TableTransportInput::ShoveDecision { .. })
                .then_some((request, binding.response_json.clone()))
        })
        .collect::<Vec<_>>();
    assert_eq!(
        decisions.len(),
        2,
        "one save selection and one Prone decision"
    );
    for action_value in [
        TacticalAction::ConcludeHostilities {
            cadence: AftermathCadence::ContinueExistingOrder,
            ruling: "End the resolved body-action encounter without another turn.".into(),
        },
        TacticalAction::FinishEncounter,
    ] {
        let command = request(&f, true, action(action_value)).await;
        Box::pin(cold(&mut f, &url, command)).await;
    }
    let released = state(&f).await;
    let release = released
        .encounter_history
        .as_ref()
        .unwrap()
        .last()
        .unwrap()
        .clone();
    assert_eq!(release.execution, TacticalExecutionVersion::ReleasedTimeV1);
    assert!(released.rules.as_ref().unwrap().timing.is_none());
    let command = request(
        &f,
        true,
        action(TacticalAction::AdvanceReleasedTime {
            seconds: 30,
            ordering: ReleasedTimeOrdering::HostSelect,
            ruling: "Wait after the actual Shove and completed encounter.".into(),
        }),
    )
    .await;
    Box::pin(cold(&mut f, &url, command)).await;
    let elapsed = state(&f).await;
    assert_eq!(elapsed.clock.now, WorldInstant(released.clock.now.0 + 30));
    assert_eq!(
        elapsed.encounter_history.as_ref().unwrap().last(),
        Some(&release)
    );
    assert!(elapsed.rules.as_ref().unwrap().timing.is_none());
    let flow = elapsed.encounter.as_ref().unwrap().flow.as_ref().unwrap();
    assert_eq!(flow.phase, TacticalPhase::Finished);
    assert!(flow.resolution.is_none());
    assert_eq!(flow.budget, TacticalTurnBudget::default());
    assert_eq!(elapsed.items, released.items);
    assert_eq!(
        elapsed.rules.as_ref().unwrap().rolls,
        released.rules.as_ref().unwrap().rolls
    );
    let host = view(&f, true).await.tactical.unwrap();
    assert!(host.active_actor.is_none());
    assert!(host.shove.is_none());

    let target = elapsed
        .encounter
        .as_ref()
        .unwrap()
        .participants
        .iter()
        .find(|p| p.entity_id != f.actors[0])
        .unwrap()
        .entity_id;
    for host in [false, true] {
        let fresh = request(&f, host, action(TacticalAction::Shove { target })).await;
        Box::pin(reject(&f, fresh)).await;
    }
    let before_retry = export_campaign(&f.pool, f.campaign).await.unwrap();
    for (accepted, response) in decisions {
        assert_eq!(
            serde_json::to_string(
                &Box::pin(f.runtime.submit_presented_table(accepted.clone()))
                    .await
                    .unwrap(),
            )
            .unwrap(),
            response
        );
        let mut fresh = accepted;
        fresh.command_id = CommandId::new();
        fresh.revision = view(&f, matches!(fresh.channel, TableTransportChannel::Host))
            .await
            .revision;
        Box::pin(reject(&f, fresh)).await;
    }
    let mut after_retry = export_campaign(&f.pool, f.campaign).await.unwrap();
    after_retry.exported_at_utc = before_retry.exported_at_utc.clone();
    assert_eq!(after_retry, before_retry);
    assert_eq!(
        &after_retry.event_journal[..original.event_journal.len()],
        &original.event_journal
    );
    assert_eq!(
        &after_retry.table_projection_history[..original.table_projection_history.len()],
        &original.table_projection_history
    );
    f.pool.close().await;
    drop(f);
    sqlite_test_cleanup::remove_closed_directory(&directory)
        .await
        .unwrap();
}
