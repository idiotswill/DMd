use super::*;

async fn view(f: &Fixture, player: usize) -> TablePresentedView {
    f.runtime
        .presented_table_view(f.campaign, TableViewer::Player(f.players[player]))
        .await
        .unwrap()
}
fn request(
    f: &Fixture,
    view: &TablePresentedView,
    input: TableTransportInput,
) -> TableTransportRequest {
    TableTransportRequest {
        version: TABLE_TRANSPORT_VERSION,
        command_id: CommandId::new(),
        campaign_id: f.campaign,
        session_id: Some(f.session),
        channel: TableTransportChannel::Player {
            player_id: f.players[0],
            character_id: f.characters[0],
        },
        revision: view.revision,
        input,
    }
}
fn text(value: &str) -> TableTransportInput {
    TableTransportInput::Text { text: value.into() }
}
async fn withdraw(f: &Fixture) {
    let current = view(f, 0).await;
    let pending = current.pending.as_ref().unwrap();
    f.runtime
        .submit_presented_table(request(
            f,
            &current,
            TableTransportInput::Action(Box::new(TableAction::CancelDecision {
                pending_id: pending.id,
                revision: pending.revision,
            })),
        ))
        .await
        .unwrap();
}
fn runtime(pool: sqlx::SqlitePool) -> CampaignRuntime {
    CampaignRuntime::from_content_root(
        pool,
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../content"),
    )
}
fn assert_no_head(value: &impl serde::Serialize) {
    let json = serde_json::to_string(value).unwrap();
    for forbidden in [
        "event_sequence",
        "expected_event_sequence",
        "\"occurrence\"",
        "\"origin\"",
        "\"meta\"",
        "already_accepted",
    ] {
        assert!(
            !json.contains(forbidden),
            "player wire contains {forbidden}: {json}"
        );
    }
}

#[tokio::test]
async fn protocol_legacy_transport_only_recovers_a_genuine_original_acceptance() {
    let mut f = Box::pin(Fixture::new()).await;
    let meta = f.player_meta(0).await;
    let action = TableAction::Declare {
        text: "I climb the ledge".into(),
    };
    f.runtime
        .execute_table(meta.clone(), action.clone())
        .await
        .unwrap();
    let original = LegacyTableRequest {
        command_id: meta.id,
        campaign_id: f.campaign,
        session_id: meta.session_id,
        expected_event_sequence: meta.expected_event_sequence,
        channel: TableTransportChannel::Player {
            player_id: f.players[0],
            character_id: f.characters[0],
        },
        input: LegacyTableInput::Action(Box::new(action)),
    };
    view(&f, 0).await;
    withdraw(&f).await;
    f.host(TableAction::EndSession, Some(f.session)).await;
    let recovered = f
        .runtime
        .recover_legacy_table_request(original.clone())
        .await
        .unwrap();
    assert_no_head(&recovered);
    let before = export_campaign(&f.pool, f.campaign).await.unwrap();
    let mut changed = original.clone();
    changed.command_id = CommandId::new();
    assert!(matches!(
        f.runtime.recover_legacy_table_request(changed).await,
        Err(RunnableCampaignError::TableRejected(_))
    ));
    let mut changed = original.clone();
    changed.expected_event_sequence += 1;
    assert!(
        f.runtime
            .recover_legacy_table_request(changed)
            .await
            .is_err()
    );
    let mut changed = original;
    changed.channel = TableTransportChannel::Player {
        player_id: f.players[1],
        character_id: f.characters[1],
    };
    assert!(
        f.runtime
            .recover_legacy_table_request(changed)
            .await
            .is_err()
    );
    assert_eq!(
        export_campaign(&f.pool, f.campaign)
            .await
            .unwrap()
            .current_state,
        before.current_state
    );
}

#[tokio::test]
async fn protocol_shared_utterances_remain_visible_without_private_decision_provenance() {
    let f = Box::pin(Fixture::new()).await;
    let other = view(&f, 1).await;
    f.runtime
        .submit_presented_table(request(&f, &view(&f, 0).await, text("I climb the ledge")))
        .await
        .unwrap();
    let after = view(&f, 1).await;
    assert_ne!(after.revision, other.revision);
    assert!(after.pending.is_none());
    assert!(
        after
            .transcript
            .iter()
            .any(|e| e.kind == "declaration" && e.text.contains("I climb the ledge"))
    );
    f.runtime
        .submit_presented_table(request(
            &f,
            &view(&f, 0).await,
            text("Actually I wait here"),
        ))
        .await
        .unwrap();
    let corrected = view(&f, 1).await;
    assert!(
        corrected
            .transcript
            .iter()
            .any(|e| e.kind == "correction" && e.text.contains("wait here"))
    );
    assert!(corrected.pending.is_none());
    assert_no_head(&corrected);
    let own = view(&f, 0).await;
    let pending = own.pending.as_ref().unwrap();
    f.runtime
        .submit_presented_table(request(
            &f,
            &own,
            TableTransportInput::Action(Box::new(TableAction::CancelDecision {
                pending_id: pending.id,
                revision: pending.revision,
            })),
        ))
        .await
        .unwrap();
    let withdrawn = view(&f, 1).await;
    assert_ne!(withdrawn.revision, corrected.revision);
    assert_eq!(withdrawn.transcript.len(), corrected.transcript.len() + 1);
}

#[tokio::test]
async fn protocol_hidden_history_changes_neither_player_projection_nor_revision() {
    let mut f = Box::pin(Fixture::new()).await;
    let before = view(&f, 0).await;
    let other_before = view(&f, 1).await;
    assert_no_head(&before);
    let baseline = export_campaign(&f.pool, f.campaign).await.unwrap();
    let fork_pool = open_sqlite("sqlite::memory:").await.unwrap();
    let fork = runtime(fork_pool);
    fork.restore_campaign(&baseline).await.unwrap();
    // DC is host-only. Its accepted canonical event must not create a generic
    // player transcript entry, rotate a revision, or stale this pending input.
    let mut situation = f.runtime.read_host_situation(f.campaign).await.unwrap();
    situation.challenges[0].dc += 2;
    f.host(TableAction::SetSituation { situation }, Some(f.session))
        .await;
    assert_eq!(view(&f, 0).await, before);
    assert_eq!(view(&f, 1).await, other_before);
    assert_eq!(
        fork.presented_table_view(f.campaign, TableViewer::Player(f.players[0]))
            .await
            .unwrap(),
        before
    );
    let submitted = request(&f, &before, text("How do I roll?"));
    let result = f
        .runtime
        .submit_presented_table(submitted.clone())
        .await
        .unwrap();
    assert_no_head(&result);
    fork.submit_presented_table(submitted).await.unwrap();
    assert_eq!(
        view(&f, 1).await,
        other_before,
        "another player's private answer must not rotate this audience"
    );
    let exported = export_campaign(&f.pool, f.campaign).await.unwrap();
    let restored = runtime(open_sqlite("sqlite::memory:").await.unwrap());
    restored.restore_campaign(&exported).await.unwrap();
    assert_eq!(
        restored
            .presented_table_view(f.campaign, TableViewer::Player(f.players[0]))
            .await
            .unwrap(),
        view(&f, 0).await
    );
}

#[tokio::test]
async fn protocol_exact_action_and_answer_retries_precede_stale_session_checks() {
    let mut f = Box::pin(Fixture::new()).await;
    let original = request(&f, &view(&f, 0).await, text("How do I roll?"));
    let answer = f
        .runtime
        .submit_presented_table(original.clone())
        .await
        .unwrap();
    let declare = request(&f, &view(&f, 0).await, text("I climb the ledge"));
    let accepted = f
        .runtime
        .submit_presented_table(declare.clone())
        .await
        .unwrap();
    assert_no_head(&accepted);
    withdraw(&f).await;
    f.host(TableAction::EndSession, Some(f.session)).await;
    let exported = export_campaign(&f.pool, f.campaign).await.unwrap();
    let restored = runtime(open_sqlite("sqlite::memory:").await.unwrap());
    restored.restore_campaign(&exported).await.unwrap();
    assert_eq!(
        restored
            .submit_presented_table(original.clone())
            .await
            .unwrap(),
        answer
    );
    assert_eq!(
        restored
            .submit_presented_table(declare.clone())
            .await
            .unwrap(),
        accepted
    );
    {
        // All original envelope fields belong to the identity, including its old
        // revision and channel; a retry never silently reconstructs new metadata.
        let mut altered = original.clone();
        altered.input = text("What can I see?");
        assert!(matches!(
            restored.submit_presented_table(altered).await,
            Err(RunnableCampaignError::TableRejected(_))
        ));
    }
    let mut altered = original.clone();
    altered.channel = TableTransportChannel::Host;
    assert!(restored.submit_presented_table(altered).await.is_err());
    let mut altered = declare;
    altered.revision = original.revision;
    assert!(restored.submit_presented_table(altered).await.is_err());
    assert_eq!(
        export_campaign(&f.pool, f.campaign)
            .await
            .unwrap()
            .current_state,
        exported.current_state
    );
}

#[tokio::test]
async fn protocol_visible_aba_never_revives_an_old_revision() {
    let mut f = Box::pin(Fixture::new()).await;
    let old = view(&f, 0).await;
    let a = f.runtime.read_host_situation(f.campaign).await.unwrap();
    let mut b = a.clone();
    b.description = "A new visible scene.".into();
    f.host(TableAction::SetSituation { situation: b }, Some(f.session))
        .await;
    let middle = view(&f, 0).await;
    f.host(TableAction::SetSituation { situation: a }, Some(f.session))
        .await;
    let final_view = view(&f, 0).await;
    assert_ne!(old.revision, middle.revision);
    assert_ne!(old.revision, final_view.revision);
    assert_eq!(old.situation_description, final_view.situation_description);
    let before = export_campaign(&f.pool, f.campaign).await.unwrap();
    assert!(matches!(
        f.runtime
            .submit_presented_table(request(&f, &old, text("I climb")))
            .await,
        Err(RunnableCampaignError::TableRejected(_))
    ));
    assert_eq!(
        export_campaign(&f.pool, f.campaign)
            .await
            .unwrap()
            .current_state,
        before.current_state
    );
}

#[tokio::test]
async fn protocol_lost_binding_write_rolls_back_action_and_reuses_exact_request() {
    let f = Box::pin(Fixture::new()).await;
    let before = view(&f, 0).await;
    let original = request(&f, &before, text("I climb the ledge"));
    let snapshot = export_campaign(&f.pool, f.campaign).await.unwrap();
    sqlx::query("CREATE TRIGGER fail_transport BEFORE INSERT ON table_transport_bindings BEGIN SELECT RAISE(FAIL,'simulated acknowledgement failure'); END").execute(&f.pool).await.unwrap();
    assert!(matches!(
        f.runtime.submit_presented_table(original.clone()).await,
        Err(RunnableCampaignError::Table(_))
    ));
    let failed = export_campaign(&f.pool, f.campaign).await.unwrap();
    assert_eq!(failed.current_state, snapshot.current_state);
    assert_eq!(failed.event_journal, snapshot.event_journal);
    assert_eq!(failed.command_audit, snapshot.command_audit);
    assert_eq!(
        failed.table_projection_history,
        snapshot.table_projection_history
    );
    assert_eq!(
        failed.table_transport_bindings,
        snapshot.table_transport_bindings
    );
    assert_eq!(view(&f, 0).await, before);
    sqlx::query("DROP TRIGGER fail_transport")
        .execute(&f.pool)
        .await
        .unwrap();
    let accepted = f
        .runtime
        .submit_presented_table(original.clone())
        .await
        .unwrap();
    assert_eq!(
        f.runtime.submit_presented_table(original).await.unwrap(),
        accepted
    );
}

#[tokio::test]
async fn protocol_restore_rejects_removed_and_changed_authority_before_any_write() {
    let f = Box::pin(Fixture::new()).await;
    let original = request(&f, &view(&f, 0).await, text("How do I roll?"));
    f.runtime
        .submit_presented_table(original.clone())
        .await
        .unwrap();
    let exported = export_campaign(&f.pool, f.campaign).await.unwrap();
    let mut corruptions = Vec::new();
    let mut changed = exported.clone();
    changed.table_projection_history.clear();
    changed.table_transport_bindings.clear();
    corruptions.push(changed);
    let mut changed = exported.clone();
    changed.table_transport_bindings.clear();
    corruptions.push(changed);
    let mut changed = exported.clone();
    changed.table_transport_bindings[0].response_json =
        "{\"Observed\":{\"answer\":\"invented\"}}".into();
    corruptions.push(changed);
    let mut changed = exported.clone();
    let mut request = original.clone();
    request.channel = TableTransportChannel::Host;
    changed.table_transport_bindings[0].request_json = serde_json::to_string(&request).unwrap();
    corruptions.push(changed);
    let mut changed = exported.clone();
    changed.table_projection_history[0].changes[0].visible_digest = "0".repeat(64);
    corruptions.push(changed);
    let mut changed = exported.clone();
    changed
        .table_projection_history
        .last_mut()
        .unwrap()
        .changes
        .iter_mut()
        .find(|change| change.audience == dmd_persistence::ProjectionAudience::Player(f.players[0]))
        .unwrap()
        .revision = dmd_persistence::ProjectionRevision(CommandId::new().0);
    corruptions.push(changed);
    let mut changed = exported.clone();
    changed.table_projection_history[0].transcript[0]
        .audiences
        .push(dmd_persistence::ProjectionAudience::Player(f.players[1]));
    corruptions.push(changed);
    for altered in corruptions {
        let pool = open_sqlite("sqlite::memory:").await.unwrap();
        let target = runtime(pool.clone());
        assert!(
            target.restore_campaign(&altered).await.is_err(),
            "accepted changed presentation authority"
        );
        for table in [
            "campaign_state_current",
            "event_journal",
            "command_audit",
            "session_observations",
            "table_projection_history",
            "table_transport_bindings",
        ] {
            let count: i64 = sqlx::query_scalar(&format!("SELECT COUNT(*) FROM {table}"))
                .fetch_one(&pool)
                .await
                .unwrap();
            assert_eq!(count, 0, "partial restore in {table}");
        }
    }
}

#[tokio::test]
async fn protocol_roll_handles_hide_canonical_ids_and_reject_direct_raw_addressing() {
    let mut f = Box::pin(Fixture::new()).await;
    let input = request(&f, &view(&f, 0).await, text("I climb the ledge"));
    f.runtime.submit_presented_table(input).await.unwrap();
    let pending = f
        .runtime
        .open_campaign(f.campaign)
        .await
        .unwrap()
        .state()
        .table
        .as_ref()
        .unwrap()
        .pending
        .clone()
        .unwrap();
    let canonical = RollRequestId::new();
    f.host(
        TableAction::Adjudicate {
            pending_id: pending.id,
            revision: pending.revision,
            request_id: canonical,
        },
        Some(f.session),
    )
    .await;
    let presented = view(&f, 0).await;
    assert_no_head(&presented);
    let opaque = presented.roll.as_ref().unwrap().id;
    assert_ne!(opaque, canonical);
    let mut wrong_capability = export_campaign(&f.pool, f.campaign).await.unwrap();
    let handle = wrong_capability
        .table_projection_history
        .iter_mut()
        .flat_map(|record| &mut record.changes)
        .filter(|change| {
            change.audience == dmd_persistence::ProjectionAudience::Player(f.players[0])
        })
        .flat_map(|change| &mut change.handles)
        .find(|handle| handle.opaque == opaque.0)
        .unwrap();
    handle.capability = dmd_persistence::ProjectionCapability::Roll {
        canonical: RollRequestId::new(),
    };
    let rejected_pool = open_sqlite("sqlite::memory:").await.unwrap();
    assert!(
        runtime(rejected_pool.clone())
            .restore_campaign(&wrong_capability)
            .await
            .is_err()
    );
    let rows: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM campaign_state_current")
        .fetch_one(&rejected_pool)
        .await
        .unwrap();
    assert_eq!(rows, 0);
    let mut raw = request(
        &f,
        &presented,
        TableTransportInput::Action(Box::new(TableAction::SubmitPhysical {
            request_id: canonical,
            faces: vec![15],
        })),
    );
    assert!(matches!(
        f.runtime.submit_presented_table(raw.clone()).await,
        Err(RunnableCampaignError::TableRejected(_))
    ));
    raw.input = TableTransportInput::Action(Box::new(TableAction::SubmitPhysical {
        request_id: opaque,
        faces: vec![15],
    }));
    let accepted = f.runtime.submit_presented_table(raw.clone()).await.unwrap();
    assert_no_head(&accepted);
    let exported = export_campaign(&f.pool, f.campaign).await.unwrap();
    let restored = runtime(open_sqlite("sqlite::memory:").await.unwrap());
    restored.restore_campaign(&exported).await.unwrap();
    assert_eq!(
        restored.submit_presented_table(raw).await.unwrap(),
        accepted
    );
}

#[tokio::test]
async fn protocol_concurrent_requests_are_serialized_and_exact_response_survives_file_reopen() {
    Box::pin(concurrent_file_case()).await;
}
async fn concurrent_file_case() {
    let path = std::env::temp_dir().join(format!("dmd-protocol-{}.sqlite", CommandId::new().0));
    let pool = dmd_persistence::open_sqlite_path(&path).await.unwrap();
    let f = Box::pin(Fixture::with_pool(TableContract::default(), pool)).await;
    let first_view = view(&f, 0).await;
    let baseline = export_campaign(&f.pool, f.campaign).await.unwrap();
    let original = request(&f, &first_view, text("I climb the ledge"));
    // Independent connections contend on the same on-disk writer lock. They
    // cannot both pass a stale check outside the acceptance transaction.
    let peer_pool = dmd_persistence::open_sqlite_path(&path).await.unwrap();
    let peer = runtime(peer_pool.clone());
    let (one, two) = tokio::join!(
        Box::pin(f.runtime.submit_presented_table(original.clone())),
        Box::pin(peer.submit_presented_table(original.clone())),
    );
    let accepted = one.unwrap();
    assert_eq!(two.unwrap(), accepted);
    let once = export_campaign(&f.pool, f.campaign).await.unwrap();
    assert_eq!(once.event_journal.len(), baseline.event_journal.len() + 1);
    assert_eq!(once.table_transport_bindings.len(), 1);
    withdraw(&f).await;
    let current = view(&f, 0).await;
    let a = request(&f, &current, text("I climb the ledge"));
    let b = request(&f, &current, text("I wait by the ledge"));
    let before = export_campaign(&f.pool, f.campaign).await.unwrap();
    let (one, two) = tokio::join!(
        Box::pin(f.runtime.submit_presented_table(a)),
        Box::pin(peer.submit_presented_table(b)),
    );
    assert_ne!(
        one.is_ok(),
        two.is_ok(),
        "one visible revision has one winner"
    );
    let rejected = if let Err(error) = one {
        error
    } else {
        two.unwrap_err()
    };
    assert!(matches!(rejected, RunnableCampaignError::TableRejected(_)));
    let after = export_campaign(&f.pool, f.campaign).await.unwrap();
    assert_eq!(after.event_journal.len(), before.event_journal.len() + 1);
    assert_eq!(
        after.table_transport_bindings.len(),
        before.table_transport_bindings.len() + 1
    );
    let presented = view(&f, 0).await;
    peer_pool.close().await;
    f.pool.close().await;
    let reopened_pool = dmd_persistence::open_sqlite_path(&path).await.unwrap();
    let reopened = runtime(reopened_pool.clone());
    assert_eq!(
        reopened.submit_presented_table(original).await.unwrap(),
        accepted
    );
    assert_eq!(
        reopened
            .presented_table_view(f.campaign, TableViewer::Player(f.players[0]))
            .await
            .unwrap(),
        presented
    );
    let mut reopened_export = export_campaign(&reopened_pool, f.campaign).await.unwrap();
    reopened_export.exported_at_utc = after.exported_at_utc.clone();
    assert_eq!(reopened_export, after);
    reopened_pool.close().await;
    std::fs::remove_file(path).unwrap();
}

#[tokio::test]
async fn protocol_party_observation_uses_original_membership_after_later_join_and_restore() {
    let mut f = Box::pin(Fixture::new()).await;
    let original = request(&f, &view(&f, 0).await, text("thanks"));
    f.runtime.submit_presented_table(original).await.unwrap();
    let old_member = view(&f, 1).await;
    assert!(
        old_member
            .transcript
            .iter()
            .any(|entry| entry.text.starts_with("thanks\n"))
    );
    f.host(TableAction::EndSession, Some(f.session)).await;
    let new_player = PlayerId::new();
    f.host(
        TableAction::AddPlayer {
            id: new_player,
            name: "Later member".into(),
        },
        None,
    )
    .await;
    let later = f
        .runtime
        .presented_table_view(f.campaign, TableViewer::Player(new_player))
        .await
        .unwrap();
    assert!(
        !later
            .transcript
            .iter()
            .any(|entry| entry.text.starts_with("thanks\n"))
    );
    let exported = export_campaign(&f.pool, f.campaign).await.unwrap();
    let restored = runtime(open_sqlite("sqlite::memory:").await.unwrap());
    restored.restore_campaign(&exported).await.unwrap();
    assert_eq!(
        restored
            .presented_table_view(f.campaign, TableViewer::Player(new_player))
            .await
            .unwrap(),
        later
    );
    assert!(
        restored
            .presented_table_view(f.campaign, TableViewer::Player(f.players[1]))
            .await
            .unwrap()
            .transcript
            .iter()
            .any(|entry| entry.text.starts_with("thanks\n"))
    );
}
