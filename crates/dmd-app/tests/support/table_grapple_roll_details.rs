//! New live guidance reads actual accepted, owned work; historical DTOs stay exact.
use super::*;

async fn submit(
    f: &mut Fixture,
    channel: TableTransportChannel,
    input: TableTransportInput,
    version: u32,
) -> TableTransportRequest {
    let mut request = f.request(channel, input).await;
    request.version = version;
    Box::pin(f.cold(request.clone())).await;
    request
}

async fn choose(f: &mut Fixture, channel: TableTransportChannel, label: &str, version: u32) {
    let mut request = f.choose(channel, label).await;
    request.version = version;
    Box::pin(f.cold(request)).await;
}

async fn fixture(version: u32, source_owner: bool) -> Fixture {
    let mut f = Box::pin(Fixture::new()).await;
    if source_owner {
        let actor = f.goblin;
        let controller = CreatureController::Player(f.players[1]);
        Box::pin(f.host(TableAction::SetSourceCreatureController { actor, controller })).await;
    }
    Box::pin(f.activate()).await;
    if version == 4 {
        Box::pin(submit(
            &mut f,
            TableTransportChannel::Host,
            TableTransportInput::Action(Box::new(TableAction::EnableGrappleTransport)),
            version,
        ))
        .await;
    } else {
        assert_eq!(version, 3);
    }
    f
}

fn source(f: &Fixture) -> TableTransportChannel {
    TableTransportChannel::SourceCreature {
        player_id: f.players[1],
        actor: f.goblin,
    }
}

async fn details_request(f: &Fixture, channel: TableTransportChannel) -> TableRollDetailsRequest {
    let view = f.view(channel.clone()).await;
    TableRollDetailsRequest {
        version: 1,
        campaign_id: f.campaign,
        channel,
        revision: view.revision,
        roll_id: view.roll.unwrap().id,
    }
}

async fn unchanged(f: &Fixture, before: &CampaignExport, rows: &[(String, Vec<Vec<String>>)]) {
    let mut after = export_campaign(&f.pool, f.campaign).await.unwrap();
    after.exported_at_utc = before.exported_at_utc.clone();
    assert_eq!(&after, before);
    assert_eq!(all_rows(&f.pool).await, rows);
}

async fn refused(f: &Fixture, request: TableRollDetailsRequest) {
    let before = export_campaign(&f.pool, f.campaign).await.unwrap();
    let rows = all_rows(&f.pool).await;
    assert!(matches!(
        Box::pin(f.runtime.table_roll_details(request)).await,
        Err(RunnableCampaignError::TableRejected(_))
    ));
    Box::pin(unchanged(f, &before, &rows)).await;
}

async fn read_pending(
    f: &mut Fixture,
    owner: TableTransportChannel,
    label: &str,
    version: u32,
    inspired: bool,
) -> TableRollDetailsRequest {
    let request = details_request(f, owner.clone()).await;
    let original = f.view(owner.clone()).await;
    assert_eq!(original.roll.as_ref().unwrap().reason, "Unsupported roll");
    assert_eq!(original.grapple.as_ref().unwrap().version, version);
    let viewer = match owner {
        TableTransportChannel::Player { player_id, .. }
        | TableTransportChannel::SourceCreature { player_id, .. } => TableViewer::Player(player_id),
        TableTransportChannel::Host => unreachable!(),
    };
    let raw = Box::pin(f.runtime.table_view(f.campaign, viewer.clone()))
        .await
        .unwrap();
    assert_eq!(raw.roll.as_ref().unwrap().reason, "Unsupported roll");
    let before = export_campaign(&f.pool, f.campaign).await.unwrap();
    let rows = all_rows(&f.pool).await;
    let options = Box::pin(f.runtime.table_roll_options(TableRollOptionsRequest {
        campaign_id: request.campaign_id,
        channel: request.channel.clone(),
        revision: request.revision,
        roll_id: request.roll_id,
    }))
    .await
    .unwrap();
    assert_eq!(
        options.heroic_inspiration,
        (version == 4).then_some(inspired)
    );
    let details = Box::pin(f.runtime.table_roll_details(request.clone()))
        .await
        .unwrap();
    assert_eq!(details.version, 1);
    assert_eq!(details.options, options);
    assert_eq!(details.display_reason, label);
    assert_eq!(
        serde_json::to_value(&details.options).unwrap(),
        if version == 4 {
            serde_json::json!({"savage_attacker":null,"heroic_inspiration":inspired})
        } else {
            serde_json::json!({"savage_attacker":null})
        }
    );
    assert_eq!(f.view(owner.clone()).await, original);
    assert_eq!(
        Box::pin(f.runtime.table_view(f.campaign, viewer.clone()))
            .await
            .unwrap(),
        raw
    );
    Box::pin(unchanged(f, &before, &rows)).await;

    // Host keeps its existing visibility without receiving the owner's authority.
    let host = details_request(f, TableTransportChannel::Host).await;
    assert_eq!(
        Box::pin(f.runtime.table_roll_details(host.clone()))
            .await
            .unwrap(),
        details
    );
    Box::pin(unchanged(f, &before, &rows)).await;
    f.pool.close().await;
    f.pool = open_sqlite_path(&f.path).await.unwrap();
    f.runtime = runtime(f.pool.clone());
    assert_eq!(f.view(owner.clone()).await, original);
    assert_eq!(
        Box::pin(f.runtime.table_view(f.campaign, viewer))
            .await
            .unwrap(),
        raw
    );
    assert_eq!(
        Box::pin(f.runtime.table_roll_details(request.clone()))
            .await
            .unwrap(),
        details
    );
    Box::pin(unchanged(f, &before, &rows)).await;

    let mirror_pool = open_sqlite("sqlite::memory:").await.unwrap();
    let mirror = runtime(mirror_pool.clone());
    Box::pin(mirror.restore_campaign(&before)).await.unwrap();
    let mirror_rows = all_rows(&mirror_pool).await;
    assert_eq!(
        Box::pin(mirror.table_roll_details(request.clone()))
            .await
            .unwrap(),
        details
    );
    assert_eq!(all_rows(&mirror_pool).await, mirror_rows);
    mirror_pool.close().await;

    for mutation in [
        "version",
        "revision",
        "handle",
        "foreign-handle",
        "canonical-handle",
        "other-pc",
        "wrong-character",
        "other-source",
        "wrong-controller",
    ] {
        let mut bad = request.clone();
        match mutation {
            "version" => bad.version = 2,
            "revision" => {
                bad.revision = dmd_persistence::ProjectionRevision(RollRequestId::new().0)
            }
            "handle" => bad.roll_id = RollRequestId::new(),
            "foreign-handle" => bad.roll_id = host.roll_id,
            "canonical-handle" => {
                bad.roll_id = f.state().await.rules.unwrap().pending.unwrap().request.id
            }
            "other-pc" => {
                bad.channel = f.pc(1);
                bad.revision = f.view(bad.channel.clone()).await.revision;
            }
            "wrong-character" => {
                bad.channel = TableTransportChannel::Player {
                    player_id: if owner == source(f) {
                        f.players[1]
                    } else {
                        f.players[0]
                    },
                    character_id: if owner == source(f) {
                        f.characters[0]
                    } else {
                        f.characters[1]
                    },
                };
            }
            "other-source" => {
                // Same audience/revision/handle, wrong selected actor.
                bad.channel = TableTransportChannel::SourceCreature {
                    player_id: if owner == source(f) {
                        f.players[1]
                    } else {
                        f.players[0]
                    },
                    actor: if owner == source(f) {
                        f.actors[1]
                    } else {
                        f.goblin
                    },
                };
            }
            "wrong-controller" => {
                bad.channel = TableTransportChannel::SourceCreature {
                    player_id: f.players[0],
                    actor: if owner == source(f) {
                        f.goblin
                    } else {
                        f.actors[0]
                    },
                };
                bad.revision = f.view(bad.channel.clone()).await.revision;
            }
            _ => unreachable!(),
        }
        assert_ne!(bad, request);
        Box::pin(refused(f, bad)).await;
    }
    let mut unknown = serde_json::to_value(&request).unwrap();
    unknown
        .as_object_mut()
        .unwrap()
        .insert("purpose".into(), "GrappleSave".into());
    assert!(serde_json::from_value::<TableRollDetailsRequest>(unknown).is_err());
    assert_eq!(
        Box::pin(f.runtime.table_roll_details(request.clone()))
            .await
            .unwrap(),
        details
    );
    Box::pin(unchanged(f, &before, &rows)).await;
    request
}

async fn physical(
    f: &mut Fixture,
    owner: TableTransportChannel,
    version: u32,
    inspired: bool,
) -> TableTransportRequest {
    let roll = f.view(owner.clone()).await.roll.unwrap();
    assert_eq!(roll.mode, RollMode::Normal);
    assert_eq!(
        roll.dice,
        vec![DieSpec {
            count: 1,
            sides: 20
        }]
    );
    let result = RollResult {
        request_id: roll.id,
        source: RollSource::Physical,
        dice: vec![DieResult {
            sides: 20,
            value: if inspired { 1 } else { 20 },
        }],
    };
    let action = if inspired {
        TacticalAction::SubmitRollWithInspiration {
            result,
            die_index: 0,
            replacement: DieResult {
                sides: 20,
                value: 20,
            },
        }
    } else {
        TacticalAction::SubmitRoll { result }
    };
    let mut host_result = match &action {
        TacticalAction::SubmitRoll { result }
        | TacticalAction::SubmitRollWithInspiration { result, .. } => result.clone(),
        _ => unreachable!(),
    };
    host_result.request_id = f.view(TableTransportChannel::Host).await.roll.unwrap().id;
    // Host has a valid, current own-audience handle from the read above. Even
    // that handle cannot authorize reporting a PC/source owner's public dice.
    let mut foreign = f
        .request(
            TableTransportChannel::Host,
            super::action(TacticalAction::SubmitRoll {
                result: host_result,
            }),
        )
        .await;
    foreign.version = version;
    Box::pin(f.reject(foreign)).await;
    Box::pin(submit(f, owner, super::action(action), version)).await
}

async fn pending_save(
    f: &mut Fixture,
    source_owner: bool,
    ability: &str,
    version: u32,
) -> TableTransportChannel {
    let owner = if source_owner { source(f) } else { f.pc(0) };
    if source_owner {
        let pc = f.pc(0);
        Box::pin(choose(
            f,
            pc,
            "Grapple Small armored figure with left hand",
            version,
        ))
        .await;
    } else {
        let pc = f.pc(0);
        Box::pin(submit(f, pc, action(TacticalAction::EndTurn), version)).await;
        Box::pin(choose(
            f,
            TableTransportChannel::Host,
            "Grapple Character 0 with right hand",
            version,
        ))
        .await;
    }
    Box::pin(choose(
        f,
        owner.clone(),
        &format!("Resist Grapple with {ability}"),
        version,
    ))
    .await;
    owner
}

async fn case(source_owner: bool, escape: bool, version: u32) {
    for choice in if escape {
        ["Athletics", "Acrobatics"]
    } else {
        ["Strength", "Dexterity"]
    } {
        let mut f = Box::pin(fixture(version, source_owner)).await;
        let inspired = version == 4 && !source_owner;
        if inspired {
            let character_id = f.characters[0];
            Box::pin(submit(
                &mut f,
                TableTransportChannel::Host,
                TableTransportInput::Action(Box::new(TableAction::AwardHeroicInspiration {
                    character_id,
                    reason: "For protecting a companion during the actual encounter.".into(),
                })),
                version,
            ))
            .await;
        }
        let owner = Box::pin(pending_save(
            &mut f,
            source_owner,
            if escape { "Strength" } else { choice },
            version,
        ))
        .await;
        let label = if escape {
            Box::pin(submit(
                &mut f,
                owner.clone(),
                action(TacticalAction::VoluntarilyFailSave),
                version,
            ))
            .await;
            let attacker = if source_owner {
                f.pc(0)
            } else {
                TableTransportChannel::Host
            };
            Box::pin(choose(
                &mut f,
                attacker.clone(),
                "Finish without changing equipment",
                version,
            ))
            .await;
            Box::pin(submit(
                &mut f,
                attacker,
                action(TacticalAction::EndTurn),
                version,
            ))
            .await;
            let offer = f
                .view(owner.clone())
                .await
                .grapple
                .unwrap()
                .choices
                .into_iter()
                .find(|offer| offer.label.ends_with(&format!("using {choice}")))
                .unwrap();
            Box::pin(submit(
                &mut f,
                owner.clone(),
                TableTransportInput::GrappleChoice { handle: offer.key },
                version,
            ))
            .await;
            if choice == "Athletics" {
                "Strength (Athletics) Escape"
            } else {
                "Dexterity (Acrobatics) Escape"
            }
        } else {
            "Grapple saving throw"
        };
        let pending_state = f.state().await;
        let pending = pending_state
            .rules
            .as_ref()
            .unwrap()
            .pending
            .clone()
            .unwrap();
        assert!(
            matches!(&pending.purpose, PendingPurpose::TacticalResolution { key, .. }
            if key.role == if escape { TacticalRollRole::GrappleEscape } else { TacticalRollRole::GrappleSave })
        );
        let saved_details = Box::pin(read_pending(
            &mut f,
            owner.clone(),
            label,
            version,
            inspired,
        ))
        .await;
        let accepted = Box::pin(physical(&mut f, owner, version, inspired)).await;
        let after = f.state().await;
        let rules = after.rules.as_ref().unwrap();
        let paid = rules.rolls.last().unwrap();
        assert_eq!(
            rules.rolls.len(),
            pending_state.rules.as_ref().unwrap().rolls.len() + 1
        );
        assert_eq!(paid.request, pending.request);
        assert_eq!(paid.purpose, pending.purpose);
        assert_eq!(paid.issued_by, pending.issued_by);
        assert_eq!(paid.accepted_by.id, accepted.command_id);
        assert_eq!(
            paid.result.dice,
            vec![DieResult {
                sides: 20,
                value: 20
            }]
        );
        assert_eq!(paid.resolved.total, 20 + pending.request.modifier);
        assert_eq!(paid.original_result.is_some(), inspired);
        if inspired {
            assert_eq!(
                paid.original_result.as_ref().unwrap().dice,
                vec![DieResult {
                    sides: 20,
                    value: 1
                }]
            );
            assert!(!rules.entities[&f.actors[0]].heroic_inspiration);
        }
        Box::pin(refused(&f, saved_details)).await;
        if !escape {
            let attacker = if source_owner {
                f.pc(0)
            } else {
                TableTransportChannel::Host
            };
            Box::pin(choose(
                &mut f,
                attacker,
                "Finish without changing equipment",
                version,
            ))
            .await;
        }
        f.close().await;
    }
}

#[tokio::test]
async fn actual_g3_pc_saves_and_escapes_keep_old_projection_and_improve_cold_live_details() {
    Box::pin(case(false, false, 3)).await;
    Box::pin(case(false, true, 3)).await;
}

#[tokio::test]
async fn actual_g4_pc_saves_and_escapes_keep_owned_inspiration_submission_and_exact_retry() {
    Box::pin(case(false, false, 4)).await;
    Box::pin(case(false, true, 4)).await;
}

#[tokio::test]
async fn actual_g3_source_saves_and_escapes_keep_source_controller_and_read_only_details() {
    Box::pin(case(true, false, 3)).await;
    Box::pin(case(true, true, 3)).await;
}

#[tokio::test]
async fn actual_g4_source_saves_and_escapes_keep_source_controller_and_read_only_details() {
    Box::pin(case(true, false, 4)).await;
    Box::pin(case(true, true, 4)).await;
}
