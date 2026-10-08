//! New session-wide awards after actual release; hostile clones are refusal inputs only.
use super::*;
use dmd_rules::tactical::{
    encounter_release_preflight, finished_encounter_dependencies, require_finished_encounter,
    retained_encounter_dependencies,
};

async fn finish(f: &mut Fixture) {
    Box::pin(act(
        f,
        TableTransportChannel::Host,
        TacticalAction::ConcludeHostilities {
            cadence: AftermathCadence::ContinueExistingOrder,
            ruling: "The combatants stop; the attending table continues playing.".into(),
        },
    ))
    .await;
    Box::pin(act(
        f,
        TableTransportChannel::Host,
        TacticalAction::FinishEncounter,
    ))
    .await;
    let state = f.state().await;
    require_finished_encounter(&state).unwrap();
    assert_eq!(
        state
            .table
            .as_ref()
            .unwrap()
            .active_session
            .as_ref()
            .unwrap()
            .session_id,
        f.session
    );
}

async fn portable_reads_and_retry(f: &Fixture, accepted: TableTransportRequest) {
    let viewers = [
        TableViewer::Host,
        TableViewer::Player(f.players[0]),
        TableViewer::Player(f.players[1]),
    ];
    let mut views = vec![];
    for viewer in &viewers {
        views.push(
            f.runtime
                .presented_table_view(f.campaign, viewer.clone())
                .await
                .unwrap(),
        );
    }
    let response = Box::pin(f.runtime.submit_presented_table(accepted.clone()))
        .await
        .unwrap();
    let export = export_campaign(&f.pool, f.campaign).await.unwrap();
    let pool = open_sqlite("sqlite::memory:").await.unwrap();
    let restored = runtime(pool.clone());
    Box::pin(restored.restore_campaign(&export)).await.unwrap();
    for (viewer, expected) in viewers.into_iter().zip(views) {
        assert_eq!(
            restored
                .presented_table_view(f.campaign, viewer)
                .await
                .unwrap(),
            expected
        );
    }
    let rows = all_rows(&pool).await;
    assert_eq!(
        Box::pin(restored.submit_presented_table(accepted))
            .await
            .unwrap(),
        response
    );
    assert_eq!(
        all_rows(&pool).await,
        rows,
        "portable exact retry wrote rows"
    );
    assert_eq!(
        Box::pin(restored.replay_rules(f.campaign)).await.unwrap(),
        f.state().await
    );
    pool.close().await;
}

#[tokio::test]
async fn post_release_decline_and_gift_preserve_recovery_privacy_and_resources() {
    for give in [false, true] {
        let mut f = Box::pin(Fixture::new()).await;
        enable(&mut f).await;
        Box::pin(finish(&mut f)).await;
        let released = f.state().await;
        let dependencies = retained_encounter_dependencies(&released).unwrap();
        grant(&mut f).await;
        let private = f.view(f.pc(1)).await;
        let retained = f
            .runtime
            .table_view(f.campaign, TableViewer::Player(f.players[1]))
            .await
            .unwrap();
        let awarded = extra(&mut f, 0).await;
        let pending_state = f.state().await;
        assert_eq!(
            finished_encounter_dependencies(&pending_state).unwrap(),
            dependencies
        );
        assert!(retained_encounter_dependencies(&pending_state).is_err());
        assert!(require_finished_encounter(&pending_state).is_err());
        assert_eq!(pending_state.encounter_history, released.encounter_history);
        assert_eq!(pending_state.clock, released.clock);
        let companion = pending_state
            .table
            .as_ref()
            .unwrap()
            .inspiration_transfer
            .as_ref()
            .unwrap();
        assert!(
            companion.origin.expected_event_sequence
                > released
                    .encounter_history
                    .as_ref()
                    .unwrap()
                    .last()
                    .unwrap()
                    .released_by
                    .expected_event_sequence
        );
        let host = f.view(TableTransportChannel::Host).await;
        assert!(host.inspiration_transfer.unwrap().choices.is_empty());
        assert_eq!(
            host.tactical.unwrap().release.unwrap().required_actors,
            dependencies
        );
        assert_eq!(
            f.view(f.pc(0))
                .await
                .inspiration_transfer
                .unwrap()
                .choices
                .len(),
            2
        );
        private_unchanged(&f, &retained, &private).await;
        Box::pin(portable_reads_and_retry(&f, awarded)).await;
        let choice = decision(
            &f,
            0,
            if give {
                "Give to Character 1"
            } else {
                "Decline the extra Inspiration"
            },
        )
        .await;
        Box::pin(f.cold(choice.clone())).await;
        Box::pin(portable_reads_and_retry(&f, choice)).await;
        let after = f.state().await;
        require_finished_encounter(&after).unwrap();
        assert!(has_inspiration(&after, f.actors[0]));
        assert_eq!(has_inspiration(&after, f.actors[1]), give);
        assert!(!pending(&after, f.actors[0]) && !pending(&after, f.actors[1]));
        assert!(after.table.as_ref().unwrap().inspiration_transfer.is_none());
        if !give {
            private_unchanged(&f, &retained, &private).await;
        }
        // Only two explicit award rulings and the intended resources may differ.
        let mut normalized = after;
        normalized.applied_event_sequence = released.applied_event_sequence;
        let rules = normalized.rules.as_mut().unwrap();
        assert_eq!(
            rules.rulings.len(),
            released.rules.as_ref().unwrap().rulings.len() + 2
        );
        rules.rulings = released.rules.as_ref().unwrap().rulings.clone();
        for actor in f.actors {
            rules.entities.get_mut(&actor).unwrap().heroic_inspiration =
                released.rules.as_ref().unwrap().entities[&actor].heroic_inspiration;
        }
        assert_eq!(
            normalized, released,
            "award/choice changed unrelated retained state"
        );
        f.close().await;
    }
}

#[tokio::test]
async fn pending_finished_choice_keeps_transition_and_owner_admission_strict() {
    let mut f = Box::pin(Fixture::new()).await;
    enable(&mut f).await;
    // Preserve a genuine setup payload, allocating only fresh replacement identities.
    let export = export_campaign(&f.pool, f.campaign).await.unwrap();
    let mut setup = export
        .event_journal
        .iter()
        .filter_map(|row| serde_json::from_str::<TableEvent>(&row.payload_json).ok())
        .find_map(|event| match event.action {
            TableAction::PrepareBattlefield { setup } => Some(setup),
            _ => None,
        })
        .unwrap();
    setup.encounter_id = EncounterId::new();
    setup.scene_id = SceneId::new();
    setup.location_id = LocationId::new();
    Box::pin(act(
        &mut f,
        TableTransportChannel::Host,
        TacticalAction::ConcludeHostilities {
            cadence: AftermathCadence::ContinueExistingOrder,
            ruling: "Stop fighting before testing strict release admission.".into(),
        },
    ))
    .await;
    encounter_release_preflight(&f.state().await).unwrap();
    grant(&mut f).await;
    extra(&mut f, 0).await;
    let active_pending = f.state().await;
    assert!(encounter_release_preflight(&active_pending).is_err());
    assert!(finished_encounter_dependencies(&active_pending).is_err());
    let premature = request(
        &f,
        TableTransportChannel::Host,
        action(TacticalAction::FinishEncounter),
    )
    .await;
    Box::pin(f.reject(premature)).await;
    let old_choice = decision(&f, 0, "Decline the extra Inspiration").await;
    Box::pin(f.cold(old_choice)).await;
    Box::pin(act(
        &mut f,
        TableTransportChannel::Host,
        TacticalAction::FinishEncounter,
    ))
    .await;
    require_finished_encounter(&f.state().await).unwrap();
    extra(&mut f, 0).await;
    let state = f.state().await;
    let create = TableAction::CreateCharacter {
        character_id: CharacterId::new(),
        entity_id: EntityId::new(),
        player_id: f.players[0],
        input: creation("Later traveler"),
    };
    let replacement = TableAction::PrepareBattlefield { setup };
    for input in [
        TableTransportInput::Action(Box::new(TableAction::EndSession)),
        TableTransportInput::Action(Box::new(replacement.clone())),
        TableTransportInput::Action(Box::new(create.clone())),
        action(TacticalAction::FinishEncounter),
        action(TacticalAction::EndTurn),
    ] {
        let bad = request(&f, TableTransportChannel::Host, input).await;
        Box::pin(f.reject(bad)).await;
    }
    assert!(encounter_release_preflight(&state).is_err());
    assert!(require_finished_encounter(&state).is_err());
    assert!(retained_encounter_dependencies(&state).is_err());
    finished_encounter_dependencies(&state).unwrap();
    let valid = decision(&f, 0, "Decline the extra Inspiration").await;
    for channel in [
        TableTransportChannel::Host,
        f.pc(1),
        TableTransportChannel::SourceCreature {
            player_id: f.players[0],
            actor: f.goblin,
        },
    ] {
        let mut wrong_owner = valid.clone();
        wrong_owner.command_id = CommandId::new();
        wrong_owner.revision = f.view(channel.clone()).await.revision;
        wrong_owner.channel = channel;
        Box::pin(f.reject(wrong_owner)).await;
    }
    let mut foreign = valid.clone();
    foreign.command_id = CommandId::new();
    foreign.campaign_id = CampaignId::new();
    Box::pin(f.reject(foreign)).await;
    for issuer in [CommandIssuer::Admin, CommandIssuer::Player(f.players[0])] {
        let rows = all_rows(&f.pool).await;
        let context = RulesContext {
            campaign_id: f.campaign,
            issuer,
            actor: Some(f.actors[0]),
            session_id: Some(f.session),
            expected_event_sequence: state.applied_event_sequence,
        };
        assert!(
            Box::pin(
                f.runtime
                    .execute_tactical(context.clone(), TacticalAction::FinishEncounter)
            )
            .await
            .is_err()
        );
        assert!(
            Box::pin(f.runtime.execute_rules(
                context,
                dmd_rules::RulesAction::ResolveInspirationTransfer {
                    actor: f.actors[0],
                    recipient: None
                }
            ))
            .await
            .is_err()
        );
        assert_eq!(all_rows(&f.pool).await, rows);
    }
    Box::pin(f.cold(valid)).await;
    require_finished_encounter(&f.state().await).unwrap();
    let end = request(
        &f,
        TableTransportChannel::Host,
        TableTransportInput::Action(Box::new(TableAction::EndSession)),
    )
    .await;
    Box::pin(f.cold(end)).await;
    let creation = request(
        &f,
        TableTransportChannel::Host,
        TableTransportInput::Action(Box::new(create)),
    )
    .await;
    Box::pin(f.cold(creation)).await;
    let session = PlaySessionId::new();
    let mut start = request(
        &f,
        TableTransportChannel::Host,
        TableTransportInput::Action(Box::new(TableAction::StartSession {
            id: session,
            name: "Continued play".into(),
            participants: (0..2)
                .map(|index| SessionParticipant {
                    player_id: f.players[index],
                    character_id: Some(f.characters[index]),
                    attendance: AttendanceStatus::Present,
                })
                .collect(),
        })),
    )
    .await;
    start.session_id = Some(session);
    Box::pin(f.cold(start)).await;
    f.session = session;
    let next = request(
        &f,
        TableTransportChannel::Host,
        TableTransportInput::Action(Box::new(replacement)),
    )
    .await;
    Box::pin(f.cold(next)).await;
    assert_eq!(f.state().await.encounter_history, state.encounter_history);
    f.close().await;
}

async fn reject_then_restore_valid(f: &Fixture, valid: &CampaignExport, state: &CampaignState) {
    let mut hostile = valid.clone();
    let json = serde_json::to_string(state).unwrap();
    hostile.current_state.state_json = json.clone();
    if let Some(latest) = hostile
        .snapshots
        .iter_mut()
        .max_by_key(|snapshot| snapshot.event_sequence)
        && u64::try_from(latest.event_sequence).ok() == Some(state.applied_event_sequence)
    {
        latest.state_json = json;
    }
    let pool = open_sqlite("sqlite::memory:").await.unwrap();
    let destination = runtime(pool.clone());
    destination
        .create_table_campaign(
            CampaignId::new(),
            "Unrelated destination",
            TableContract::default(),
        )
        .await
        .unwrap();
    let rows = all_rows(&pool).await;
    assert!(
        Box::pin(destination.restore_campaign(&hostile))
            .await
            .is_err()
    );
    assert_eq!(
        all_rows(&pool).await,
        rows,
        "hostile Finished restore wrote a row"
    );
    Box::pin(destination.restore_campaign(valid)).await.unwrap();
    assert_eq!(
        destination.open_campaign(f.campaign).await.unwrap().state(),
        &f.state().await
    );
    assert_eq!(
        Box::pin(destination.replay_rules(f.campaign))
            .await
            .unwrap(),
        f.state().await
    );
    pool.close().await;
}

#[tokio::test]
async fn hostile_finished_choices_cannot_borrow_release_or_other_pending_work() {
    let mut f = Box::pin(Fixture::new()).await;
    enable(&mut f).await;
    Box::pin(finish(&mut f)).await;
    grant(&mut f).await;
    extra(&mut f, 0).await;
    let state = f.state().await;
    let valid = export_campaign(&f.pool, f.campaign).await.unwrap();
    let release_sequence = state
        .encounter_history
        .as_ref()
        .unwrap()
        .last()
        .unwrap()
        .released_by
        .expected_event_sequence;
    for mutation in [
        "missing-companion",
        "at-release",
        "before-release",
        "second-flag",
        "raw-pending",
        "permission",
        "foreign-owner",
        "unowned-origin",
    ] {
        let mut hostile = state.clone();
        match mutation {
            "missing-companion" => hostile.table.as_mut().unwrap().inspiration_transfer = None,
            "at-release" | "before-release" => {
                hostile
                    .table
                    .as_mut()
                    .unwrap()
                    .inspiration_transfer
                    .as_mut()
                    .unwrap()
                    .origin
                    .expected_event_sequence = if mutation == "at-release" {
                    release_sequence
                } else {
                    release_sequence.checked_sub(1).unwrap()
                }
            }
            "second-flag" => {
                let other = hostile
                    .rules
                    .as_mut()
                    .unwrap()
                    .entities
                    .get_mut(&f.actors[1])
                    .unwrap();
                other.heroic_inspiration = true;
                other
                    .character_features
                    .as_mut()
                    .unwrap()
                    .inspiration_transfer_pending = true;
            }
            "raw-pending" => {
                // Reopening an actual retired initiative request is hostile input,
                // never a way to produce the accepted Finished setup above.
                let roll = hostile.rules.as_ref().unwrap().rolls[0].clone();
                hostile.rules.as_mut().unwrap().pending = Some(PendingRoll {
                    issued_by: roll.issued_by,
                    request: roll.request,
                    purpose: roll.purpose,
                    ruling: Ruling {
                        basis: RulingBasis::GmAdjudication,
                        reason: "Hostile retained raw request".into(),
                    },
                });
            }
            "permission" => {
                hostile.rules.as_mut().unwrap().permission = Some(ActionPermission {
                    issued_by: hostile
                        .table
                        .as_ref()
                        .unwrap()
                        .inspiration_transfer
                        .as_ref()
                        .unwrap()
                        .origin
                        .clone(),
                    actor: f.actors[0],
                    target: f.goblin,
                    content_id: "dagger".into(),
                    spell: false,
                    circumstances: Circumstances::default(),
                    within_five_feet: true,
                    ruling: Ruling {
                        basis: RulingBasis::GmAdjudication,
                        reason: "Hostile permission".into(),
                    },
                })
            }
            "foreign-owner" => {
                hostile
                    .table
                    .as_mut()
                    .unwrap()
                    .inspiration_transfer
                    .as_mut()
                    .unwrap()
                    .character_id = CharacterId::new()
            }
            "unowned-origin" => {
                hostile
                    .table
                    .as_mut()
                    .unwrap()
                    .inspiration_transfer
                    .as_mut()
                    .unwrap()
                    .origin
                    .id = CommandId::new()
            }
            _ => unreachable!(),
        }
        let query = finished_encounter_dependencies(&hostile);
        if mutation == "unowned-origin" {
            // Structural readability cannot authenticate an invented command ID.
            // The original journal/reducer must still reject this portable image.
            query.unwrap();
        } else {
            assert!(query.is_err(), "Finished read admitted {mutation}");
        }
        Box::pin(reject_then_restore_valid(&f, &valid, &hostile)).await;
    }
    let choice = decision(&f, 0, "Decline the extra Inspiration").await;
    Box::pin(f.cold(choice)).await;
    f.close().await;
}
