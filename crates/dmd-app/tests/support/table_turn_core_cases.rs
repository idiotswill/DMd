use super::table_tactical_cases::{begin, prepare, roll};
use super::*;
use dmd_rules::tactical::TacticalAction;

fn runtime(pool: sqlx::SqlitePool) -> CampaignRuntime {
    CampaignRuntime::from_content_root(
        pool,
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../content"),
    )
}

async fn current(f: &Fixture) -> CampaignState {
    f.runtime
        .open_campaign(f.campaign)
        .await
        .unwrap()
        .state()
        .clone()
}

async fn prepare_round(f: &mut Fixture) {
    f.host(TableAction::EndSession, Some(f.session)).await;
    f.session = PlaySessionId::new();
    f.host(
        TableAction::StartSession {
            id: f.session,
            name: "A durable encounter".into(),
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
    prepare(f).await;
}

// Each phase is independently boxed so default Windows async poll stacks remain
// bounded. Both the original disk runtime and the restored mirror execute real input.
async fn cold_step(f: &mut Fixture, url: &str, meta: CommandMeta, action: TableAction) {
    let export = export_campaign(&f.pool, f.campaign).await.unwrap();
    let mirror_pool = open_sqlite("sqlite::memory:").await.unwrap();
    let mirror = runtime(mirror_pool.clone());
    mirror.restore_campaign(&export).await.unwrap();
    let accepted = f
        .runtime
        .execute_table(meta.clone(), action.clone())
        .await
        .unwrap();
    let replayed = mirror
        .execute_table(meta.clone(), action.clone())
        .await
        .unwrap();
    assert_eq!(accepted, replayed);
    assert!(!accepted.already_accepted);
    let state = current(f).await;
    assert_eq!(
        mirror.resume_campaign(f.campaign).await.unwrap().state(),
        &state
    );
    mirror_pool.close().await;

    let before_retry = export_campaign(&f.pool, f.campaign).await.unwrap();
    f.pool.close().await;
    f.pool = open_sqlite(url).await.unwrap();
    f.runtime = runtime(f.pool.clone());
    assert_eq!(current(f).await, state);
    let retried = f
        .runtime
        .execute_table(meta.clone(), action.clone())
        .await
        .unwrap();
    assert!(retried.already_accepted);
    assert_eq!(retried.command_id, accepted.command_id);
    assert_eq!(retried.event_sequence, accepted.event_sequence);
    assert_eq!(retried.outcome, accepted.outcome);
    assert_eq!(current(f).await, state);
    let mut after_retry = export_campaign(&f.pool, f.campaign).await.unwrap();
    // This timestamp describes the export request, not a persisted campaign row.
    // Reopening and retrying may cross a wall-clock second; all durable fields
    // must still match exactly, including their own creation timestamps.
    after_retry.exported_at_utc = before_retry.exported_at_utc.clone();
    assert_eq!(after_retry, before_retry);

    let mut stale = meta;
    stale.id = CommandId::new();
    assert!(f.runtime.execute_table(stale, action).await.is_err());
    assert_eq!(current(f).await, state);
}

async fn initiative(f: &mut Fixture, url: &str) {
    let meta = f.meta(CommandIssuer::Admin, None, Some(f.session)).await;
    let action = begin(f);
    Box::pin(cold_step(f, url, meta, action)).await;
    for (index, face) in [(0, 16), (1, 7)] {
        let view = f
            .runtime
            .table_view(f.campaign, TableViewer::Player(f.players[index]))
            .await
            .unwrap();
        assert_eq!(view.roll_channel, Some(TableRollChannel::Tactical));
        let action = roll(f, index, face).await;
        let before = current(f).await;
        assert!(
            f.runtime
                .execute_table(f.player_meta(1 - index).await, action.clone())
                .await
                .is_err()
        );
        assert_eq!(current(f).await, before);
        let meta = f.player_meta(index).await;
        Box::pin(cold_step(f, url, meta, action)).await;
    }
}

async fn ordinary_checks_on_prepared_map(f: &mut Fixture, url: &str) {
    for (text, face) in [("I climb the ledge", 12), ("I use Second Wind", 7)] {
        f.runtime
            .submit_table_text(f.player_meta(0).await, text)
            .await
            .unwrap();
        let pending = f
            .runtime
            .table_view(f.campaign, TableViewer::Player(f.players[0]))
            .await
            .unwrap()
            .pending
            .unwrap();
        let request_id = RollRequestId::new();
        f.host(
            TableAction::Adjudicate {
                pending_id: pending.id,
                revision: pending.revision,
                request_id,
            },
            Some(f.session),
        )
        .await;
        let view = f
            .runtime
            .table_view(f.campaign, TableViewer::Player(f.players[0]))
            .await
            .unwrap();
        assert_eq!(view.tactical.unwrap().phase, "setup");
        assert_eq!(view.roll_channel, Some(TableRollChannel::Table));
        let request = view.roll.unwrap();
        assert_eq!(request.id, request_id);
        let other = f
            .runtime
            .table_view(f.campaign, TableViewer::Player(f.players[1]))
            .await
            .unwrap();
        if other.roll.is_none() {
            assert_eq!(other.roll_channel, None);
        }
        let before = current(f).await;
        let wrong = TableAction::Tactical {
            action: TacticalAction::SubmitRoll {
                result: RollResult {
                    request_id,
                    source: RollSource::Physical,
                    dice: vec![DieResult {
                        sides: request.dice[0].sides,
                        value: face,
                    }],
                },
            },
        };
        assert!(
            f.runtime
                .execute_table(f.player_meta(0).await, wrong)
                .await
                .is_err()
        );
        assert_eq!(current(f).await, before);
        let meta = f.player_meta(0).await;
        Box::pin(cold_step(
            f,
            url,
            meta,
            TableAction::SubmitPhysical {
                request_id,
                faces: vec![face],
            },
        ))
        .await;
        let complete = f
            .runtime
            .table_view(f.campaign, TableViewer::Player(f.players[0]))
            .await
            .unwrap();
        assert!(complete.roll.is_none());
        assert_eq!(complete.roll_channel, None);
    }
    let state = current(f).await;
    assert!(
        state.table.as_ref().unwrap().situation.challenges[0]
            .resolution
            .as_ref()
            .unwrap()
            .success
    );
    assert_eq!(state.rules.as_ref().unwrap().rolls.len(), 2);
}

async fn complete_round(f: &mut Fixture, url: &str) {
    for (actor, action) in [
        (0, TacticalAction::Dodge),
        (0, TacticalAction::EndTurn),
        (
            1,
            TacticalAction::Dash {
                speed: DashSpeed::Speed,
            },
        ),
        (1, TacticalAction::EndTurn),
    ] {
        let meta = f.player_meta(actor).await;
        Box::pin(cold_step(f, url, meta, TableAction::Tactical { action })).await;
    }
    let state = current(f).await;
    let timing = state.rules.as_ref().unwrap().timing.as_ref().unwrap();
    assert_eq!(timing.round, 2);
    assert_eq!(timing.order[timing.index].actor, f.actors[0]);
    assert_eq!(state.clock.now, WorldInstant(6));
    assert!(!timing.action_spent);
    assert_eq!(
        state
            .encounter
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .budget,
        TacticalTurnBudget::default()
    );
    assert!(
        state
            .encounter
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .dodges
            .is_empty()
    );
    for (index, player) in f.players.iter().copied().enumerate() {
        let view = f
            .runtime
            .table_view(f.campaign, TableViewer::Player(player))
            .await
            .unwrap();
        let tactical = view.tactical.unwrap();
        assert!(tactical.battlefield.is_none());
        assert!(tactical.participants.is_empty());
        assert_eq!(
            tactical
                .attack_options
                .as_ref()
                .map(|options| options.actor),
            (index == 0).then_some(f.actors[0])
        );
        assert!(tactical.casting_options.is_none());
        assert_eq!(
            tactical
                .movement_options
                .as_ref()
                .map(|options| options.actor),
            (index == 0).then_some(f.actors[0])
        );
    }
}

async fn reject_forged_authority(f: &Fixture) {
    let export = export_campaign(&f.pool, f.campaign).await.unwrap();
    for (case, backfill) in (0..7).flat_map(|case| [false, true].map(|backfill| (case, backfill))) {
        let mut forged = export.clone();
        let mut state = current(f).await;
        let budget = &mut state
            .encounter
            .as_mut()
            .unwrap()
            .flow
            .as_mut()
            .unwrap()
            .budget;
        match case {
            0 => budget.movement_progress = Some(TacticalMovementProgress::default()),
            1 => budget.movement_origin = Some(f.player_meta(0).await),
            2 => budget.attacks_remaining = 1,
            3 => {
                budget.attack_window = Some(WeaponActionWindow {
                    id: CommandId::new(),
                    kind: WeaponActionKind::AttackAction,
                })
            }
            4 => budget.other_slot_casters.push(f.actors[1]),
            5 => budget.object_interaction_spent = true,
            _ => {
                state
                    .rules
                    .as_mut()
                    .unwrap()
                    .timing
                    .as_mut()
                    .unwrap()
                    .slot_spent_this_turn = true
            }
        }
        forged.current_state.state_json = state.encode_json().unwrap();
        if backfill {
            forged.snapshots = vec![dmd_persistence::SnapshotRow {
                campaign_id: forged.current_state.campaign_id.clone(),
                event_sequence: forged.current_state.applied_event_sequence,
                state_schema_version: forged.current_state.schema_version,
                state_json: forged.current_state.state_json.clone(),
                created_at_utc: "2026-09-25 00:00:00".into(),
            }];
        }
        // The envelope parses, so only real application/source authentication can
        // reject invented action authority before any writes, even though the
        // corresponding real feature is now implemented.
        forged.upgraded().unwrap();
        let pool = open_sqlite("sqlite::memory:").await.unwrap();
        let restored = runtime(pool.clone());
        assert!(
            restored.restore_campaign(&forged).await.is_err(),
            "accepted forged budget case {case}, backfill {backfill}"
        );
        for table in [
            "campaign_state_current",
            "campaign_lifecycle",
            "event_journal",
            "command_audit",
            "campaign_snapshots",
        ] {
            let count: i64 = sqlx::query_scalar(&format!("SELECT COUNT(*) FROM {table}"))
                .fetch_one(&pool)
                .await
                .unwrap();
            assert_eq!(count, 0, "partial restore wrote {table}");
        }
        pool.close().await;
    }
}

#[tokio::test]
async fn complete_table_round_survives_every_disk_reopen_retry_and_semantic_restore() {
    let directory = std::env::temp_dir().join(format!("dmd-turn-core-{}", CampaignId::new().0));
    std::fs::create_dir_all(&directory).unwrap();
    let path = directory.join("campaign.sqlite");
    let url = format!("sqlite://{}", path.to_string_lossy().replace('\\', "/"));
    let pool = open_sqlite(&url).await.unwrap();
    let mut f = Box::pin(Fixture::with_pool(TableContract::default(), pool)).await;
    Box::pin(prepare_round(&mut f)).await;
    Box::pin(ordinary_checks_on_prepared_map(&mut f, &url)).await;
    Box::pin(initiative(&mut f, &url)).await;
    Box::pin(complete_round(&mut f, &url)).await;
    Box::pin(reject_forged_authority(&f)).await;
    f.pool.close().await;
    drop(f);
    sqlite_test_cleanup::remove_closed_directory(&directory)
        .await
        .unwrap();
}

async fn pending_second_wind(f: &mut Fixture, url: &str, spoken: bool) {
    let (meta, action) = if spoken {
        f.runtime
            .submit_table_text(f.player_meta(0).await, "I use Second Wind")
            .await
            .unwrap();
        let pending = f
            .runtime
            .table_view(f.campaign, TableViewer::Player(f.players[0]))
            .await
            .unwrap()
            .pending
            .unwrap();
        (
            f.meta(CommandIssuer::Admin, None, Some(f.session)).await,
            TableAction::Adjudicate {
                pending_id: pending.id,
                revision: pending.revision,
                request_id: RollRequestId::new(),
            },
        )
    } else {
        (
            f.player_meta(0).await,
            TableAction::Tactical {
                action: TacticalAction::SecondWind,
            },
        )
    };
    Box::pin(cold_step(f, url, meta, action)).await;
    let view = f
        .runtime
        .table_view(f.campaign, TableViewer::Player(f.players[0]))
        .await
        .unwrap();
    assert!(view.pending.is_none());
    assert_eq!(view.roll_channel, Some(TableRollChannel::Tactical));
    let request = view.roll.unwrap();
    assert_eq!(
        request.dice,
        vec![DieSpec {
            count: 1,
            sides: 10
        }]
    );
    assert_eq!(request.modifier, 1);
    let before = current(f).await;
    assert!(
        before
            .rules
            .as_ref()
            .unwrap()
            .timing
            .as_ref()
            .unwrap()
            .bonus_action_spent
    );
    assert!(
        !before
            .rules
            .as_ref()
            .unwrap()
            .timing
            .as_ref()
            .unwrap()
            .action_spent
    );
    let raw = TableAction::Tactical {
        action: TacticalAction::SubmitRoll {
            result: RollResult {
                request_id: request.id,
                source: RollSource::Physical,
                dice: vec![DieResult {
                    sides: 10,
                    value: 7,
                }],
            },
        },
    };
    assert!(
        f.runtime
            .execute_table(f.player_meta(1).await, raw.clone())
            .await
            .is_err()
    );
    assert!(
        f.runtime
            .execute_table(
                f.player_meta(0).await,
                TableAction::SubmitPhysical {
                    request_id: request.id,
                    faces: vec![7]
                }
            )
            .await
            .is_err()
    );
    assert_eq!(current(f).await, before);
    if !spoken {
        Box::pin(reject_forged_second_wind(f)).await;
    }
    let meta = f.player_meta(0).await;
    Box::pin(cold_step(f, url, meta, raw)).await;
    let state = current(f).await;
    let rules = state.rules.as_ref().unwrap();
    let actor = &rules.entities[&f.actors[0]];
    assert_eq!(actor.hp, actor.max_hp); // Genuine full-health use still pays; no over-healing.
    assert_eq!(
        actor
            .character_features
            .as_ref()
            .unwrap()
            .second_wind_remaining,
        if spoken { 0 } else { 1 }
    );
    assert_eq!(
        rules.rolls.last().unwrap().result.dice,
        vec![DieResult {
            sides: 10,
            value: 7
        }]
    );
    assert!(
        state
            .encounter
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .resolution
            .is_none()
    );
}

async fn reject_forged_second_wind(f: &Fixture) {
    let mut export = export_campaign(&f.pool, f.campaign).await.unwrap();
    let mut forged = current(f).await;
    forged
        .rules
        .as_mut()
        .unwrap()
        .entities
        .get_mut(&f.actors[0])
        .unwrap()
        .character_features
        .as_mut()
        .unwrap()
        .second_wind_remaining = 0;
    let resolution = forged
        .encounter
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .resolution
        .as_mut()
        .unwrap();
    let work = &mut resolution.pending.as_mut().unwrap().work;
    let TacticalWorkKind::SecondWind { uses_before, .. } = &mut work.kind else {
        panic!()
    };
    *uses_before = 1;
    // Current execution also retains the identical source work in its causal
    // trace. Forge both images coherently so this still tests semantic history,
    // rather than stopping at a mismatched redundant work record.
    let changed = work.clone();
    if let Some(trace) = &mut resolution.work_trace {
        trace
            .nodes
            .iter_mut()
            .find(|node| node.work.occurrence == changed.occurrence)
            .unwrap()
            .work = changed;
    }
    // This invented extra expenditure is structurally coherent; only the real
    // accepted history proves that exactly one of the two uses was spent.
    dmd_rules::tactical::validate_tactical_state(&forged).unwrap();
    export.current_state.state_json = forged.encode_json().unwrap();
    let pool = open_sqlite("sqlite::memory:").await.unwrap();
    let restored = runtime(pool.clone());
    assert!(restored.restore_campaign(&export).await.is_err());
    let rows: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM campaign_state_current")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(rows, 0);
    pool.close().await;
}

#[tokio::test]
async fn second_wind_direct_and_declared_actions_survive_cold_dice_retry_and_restore() {
    let directory = std::env::temp_dir().join(format!("dmd-second-wind-{}", CampaignId::new().0));
    std::fs::create_dir_all(&directory).unwrap();
    let path = directory.join("campaign.sqlite");
    let url = format!("sqlite://{}", path.to_string_lossy().replace('\\', "/"));
    let pool = open_sqlite(&url).await.unwrap();
    let mut f = Box::pin(Fixture::with_pool(TableContract::default(), pool)).await;
    Box::pin(prepare_round(&mut f)).await;
    Box::pin(initiative(&mut f, &url)).await;
    Box::pin(pending_second_wind(&mut f, &url, false)).await;
    Box::pin(complete_round(&mut f, &url)).await;
    Box::pin(pending_second_wind(&mut f, &url, true)).await;
    f.pool.close().await;
    drop(f);
    sqlite_test_cleanup::remove_closed_directory(&directory)
        .await
        .unwrap();
}
