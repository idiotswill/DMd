//! Optional excess awards use real Host production and the attending owner.
//! Existing first-award/physical-roll fixtures and bodies remain unchanged.
use super::*;

const EXTRA: &str = "For returning to help a companion escape.";

fn excess(character_id: CharacterId) -> TableTransportInput {
    TableTransportInput::Action(Box::new(TableAction::AwardExcessInspiration {
        character_id,
        reason: EXTRA.into(),
    }))
}
async fn grant_to(f: &mut Fixture, index: usize) -> TableTransportRequest {
    let command = request(
        f,
        TableTransportChannel::Host,
        award(f.characters[index], REASON),
    )
    .await;
    Box::pin(f.cold(command.clone())).await;
    command
}
async fn extra(f: &mut Fixture, index: usize) -> TableTransportRequest {
    let command = request(f, TableTransportChannel::Host, excess(f.characters[index])).await;
    Box::pin(f.cold(command.clone())).await;
    command
}
async fn decision(f: &Fixture, index: usize, label: &str) -> TableTransportRequest {
    let transfer = f.view(f.pc(index)).await.inspiration_transfer.unwrap();
    let handle = transfer
        .choices
        .iter()
        .find(|choice| choice.label == label)
        .unwrap()
        .key;
    request(
        f,
        f.pc(index),
        TableTransportInput::InspirationTransfer { handle },
    )
    .await
}
fn has_inspiration(state: &CampaignState, actor: EntityId) -> bool {
    state.rules.as_ref().unwrap().entities[&actor].heroic_inspiration
}
fn pending(state: &CampaignState, actor: EntityId) -> bool {
    state.rules.as_ref().unwrap().entities[&actor]
        .character_features
        .as_ref()
        .unwrap()
        .inspiration_transfer_pending
}
async fn private_unchanged(f: &Fixture, retained: &TableView, presented: &TablePresentedView) {
    Box::pin(assert_private_history(f, retained, presented)).await;
    let mut current = f
        .runtime
        .table_view(f.campaign, TableViewer::Player(f.players[1]))
        .await
        .unwrap();
    // The internal canonical head advances globally; every audience-visible field stays exact.
    current.event_sequence = retained.event_sequence;
    assert_eq!(&current, retained);
}
async fn physical(f: &mut Fixture, channel: TableTransportChannel, face: u16) {
    let roll = f.view(channel.clone()).await.roll.unwrap();
    let dice = roll
        .dice
        .iter()
        .flat_map(|die| {
            std::iter::repeat_n(
                DieResult {
                    sides: die.sides,
                    value: face,
                },
                if roll.mode == RollMode::Normal {
                    usize::from(die.count)
                } else {
                    2
                },
            )
        })
        .collect();
    Box::pin(act(
        f,
        channel,
        TacticalAction::SubmitRoll {
            result: RollResult {
                request_id: roll.id,
                source: RollSource::Physical,
                dice,
            },
        },
    ))
    .await;
}
async fn next_session(f: &mut Fixture, absent: Option<usize>) {
    Box::pin(act(
        f,
        TableTransportChannel::Host,
        TacticalAction::ConcludeHostilities {
            cadence: AftermathCadence::ContinueExistingOrder,
            ruling: "Both sides stop while preserving the existing turn order.".into(),
        },
    ))
    .await;
    let end = request(
        f,
        TableTransportChannel::Host,
        TableTransportInput::Action(Box::new(TableAction::EndSession)),
    )
    .await;
    Box::pin(f.cold(end)).await;
    let id = PlaySessionId::new();
    let mut command = request(
        f,
        TableTransportChannel::Host,
        TableTransportInput::Action(Box::new(TableAction::StartSession {
            id,
            name: "The next evening".into(),
            participants: (0..2)
                .map(|index| SessionParticipant {
                    player_id: f.players[index],
                    character_id: Some(f.characters[index]),
                    attendance: if absent == Some(index) {
                        AttendanceStatus::Absent
                    } else {
                        AttendanceStatus::Present
                    },
                })
                .collect(),
        })),
    )
    .await;
    command.session_id = Some(id);
    Box::pin(f.cold(command)).await;
    f.session = id;
}

#[tokio::test]
async fn excess_award_has_exact_host_origin_one_flag_one_ruling_and_no_unrelated_projection_change()
{
    let mut f = Box::pin(Fixture::new()).await;
    enable(&mut f).await;
    grant(&mut f).await;
    let before = f.state().await;
    let private = f.view(f.pc(1)).await;
    let retained = f
        .runtime
        .table_view(f.campaign, TableViewer::Player(f.players[1]))
        .await
        .unwrap();
    let command = extra(&mut f, 0).await;
    let after = f.state().await;
    let origin = after
        .table
        .as_ref()
        .unwrap()
        .inspiration_transfer
        .as_ref()
        .unwrap();
    let ruling = after.rules.as_ref().unwrap().rulings.last().unwrap();
    assert_eq!(origin.origin.id, command.command_id);
    assert_eq!(origin.character_id, f.characters[0]);
    assert_eq!(ruling.command, origin.origin);
    assert_eq!(
        ruling.ruling,
        Ruling {
            basis: RulingBasis::Srd { page: 8 },
            reason: EXTRA.into()
        }
    );
    assert!(has_inspiration(&after, f.actors[0]) && pending(&after, f.actors[0]));
    assert!(!has_inspiration(&after, f.actors[1]) && !pending(&after, f.actors[1]));
    let mut normalized = after.clone();
    normalized.applied_event_sequence = before.applied_event_sequence;
    normalized.table.as_mut().unwrap().inspiration_transfer = None;
    let rules = normalized.rules.as_mut().unwrap();
    rules.rulings.pop();
    rules
        .entities
        .get_mut(&f.actors[0])
        .unwrap()
        .character_features
        .as_mut()
        .unwrap()
        .inspiration_transfer_pending = false;
    assert_eq!(
        normalized, before,
        "no turn, clock, grip, dice or extra resource changed"
    );
    private_unchanged(&f, &retained, &private).await;
    let owner = f.view(f.pc(0)).await;
    let shown = owner.inspiration_transfer.unwrap();
    assert_eq!(
        shown
            .choices
            .iter()
            .map(|c| c.label.as_str())
            .collect::<Vec<_>>(),
        vec!["Give to Character 1", "Decline the extra Inspiration"]
    );
    assert!(
        owner
            .characters
            .iter()
            .find(|c| c.character_id == f.characters[1])
            .unwrap()
            .details
            .is_none()
    );
    assert!(
        f.view(TableTransportChannel::Host)
            .await
            .inspiration_transfer
            .unwrap()
            .choices
            .is_empty()
    );
    let export = export_campaign(&f.pool, f.campaign).await.unwrap();
    let row = export
        .event_journal
        .iter()
        .find(|row| row.command_id == command.command_id.0.to_string())
        .unwrap();
    let event: TableEvent = serde_json::from_str(&row.payload_json).unwrap();
    assert_eq!(event.meta, origin.origin);
    assert_eq!(
        event.action,
        TableAction::AwardExcessInspiration {
            character_id: f.characters[0],
            reason: EXTRA.into()
        }
    );
    assert!(
        event.rules_event.is_none()
            && event.tactical_event.is_none()
            && event.outcome.mechanics.is_none()
    );
    assert_eq!(
        export
            .command_audit
            .iter()
            .find(|row| row.id == command.command_id.0.to_string())
            .unwrap()
            .command_schema_version,
        5
    );
    f.close().await;
}

#[tokio::test]
async fn owner_transfer_and_decline_replay_cold_without_stacking_action_cost_or_privileged_player_ruling()
 {
    for give in [true, false] {
        let mut f = Box::pin(Fixture::new()).await;
        enable(&mut f).await;
        grant(&mut f).await;
        extra(&mut f, 0).await;
        let before = f.state().await;
        let private = f.view(f.pc(1)).await;
        let retained = f
            .runtime
            .table_view(f.campaign, TableViewer::Player(f.players[1]))
            .await
            .unwrap();
        let label = if give {
            "Give to Character 1"
        } else {
            "Decline the extra Inspiration"
        };
        let command = decision(&f, 0, label).await;
        Box::pin(f.cold(command.clone())).await;
        let after = f.state().await;
        assert!(after.table.as_ref().unwrap().inspiration_transfer.is_none());
        assert!(has_inspiration(&after, f.actors[0]) && !pending(&after, f.actors[0]));
        assert_eq!(has_inspiration(&after, f.actors[1]), give);
        assert!(!pending(&after, f.actors[1]));
        let mut normalized = after.clone();
        normalized.applied_event_sequence = before.applied_event_sequence;
        normalized.table.as_mut().unwrap().inspiration_transfer =
            before.table.as_ref().unwrap().inspiration_transfer.clone();
        let rules = normalized.rules.as_mut().unwrap();
        rules
            .entities
            .get_mut(&f.actors[0])
            .unwrap()
            .character_features
            .as_mut()
            .unwrap()
            .inspiration_transfer_pending = true;
        rules
            .entities
            .get_mut(&f.actors[1])
            .unwrap()
            .heroic_inspiration = false;
        assert_eq!(
            normalized, before,
            "choice only clears its flag/provenance and optionally gives one resource"
        );
        if !give {
            private_unchanged(&f, &retained, &private).await;
        }
        let export = export_campaign(&f.pool, f.campaign).await.unwrap();
        let event: TableEvent = serde_json::from_str(
            &export
                .event_journal
                .iter()
                .find(|row| row.command_id == command.command_id.0.to_string())
                .unwrap()
                .payload_json,
        )
        .unwrap();
        assert_eq!(event.meta.issuer, CommandIssuer::Player(f.players[0]));
        assert_eq!(event.meta.actor, Some(AgentRef::Entity(f.actors[0])));
        assert_eq!(
            event.action,
            TableAction::ResolveHostInspirationTransfer {
                choice: TableInspirationTransferChoice {
                    award: before
                        .table
                        .as_ref()
                        .unwrap()
                        .inspiration_transfer
                        .as_ref()
                        .unwrap()
                        .origin
                        .id,
                    character_id: f.characters[0],
                    recipient: give.then_some(f.characters[1]),
                }
            }
        );
        assert!(
            event.rules_event.is_none()
                && event.tactical_event.is_none()
                && event.outcome.mechanics.is_none()
        );
        assert!(f.view(f.pc(0)).await.inspiration_transfer.is_none());
        let mut spent = command;
        spent.command_id = CommandId::new();
        spent.revision = f.view(f.pc(0)).await.revision;
        Box::pin(f.reject(spent.clone())).await;
        extra(&mut f, 0).await;
        spent.revision = f.view(f.pc(0)).await.revision;
        Box::pin(f.reject(spent)).await;
        let fresh = decision(&f, 0, "Decline the extra Inspiration").await;
        Box::pin(f.cold(fresh)).await;
        f.close().await;
    }
}

#[tokio::test]
async fn only_current_attending_owner_may_use_opaque_gift_or_decline_and_raw_authority_cannot_substitute()
 {
    let mut f = Box::pin(Fixture::new()).await;
    let foreign = Box::pin(Fixture::new()).await;
    enable(&mut f).await;
    grant(&mut f).await;
    extra(&mut f, 0).await;
    let valid = decision(&f, 0, "Give to Character 1").await;
    for mutation in [
        "host",
        "other",
        "source",
        "wrong-character",
        "unknown",
        "raw-award",
        "old-version",
        "session",
        "revision",
        "campaign",
    ] {
        let mut bad = valid.clone();
        bad.command_id = CommandId::new();
        match mutation {
            "host" => {
                bad.channel = TableTransportChannel::Host;
                bad.revision = f.view(bad.channel.clone()).await.revision;
            }
            "other" => {
                bad.channel = f.pc(1);
                bad.revision = f.view(bad.channel.clone()).await.revision;
            }
            "source" => {
                bad.channel = TableTransportChannel::SourceCreature {
                    player_id: f.players[0],
                    actor: f.goblin,
                }
            }
            "wrong-character" => {
                bad.channel = TableTransportChannel::Player {
                    player_id: f.players[0],
                    character_id: f.characters[1],
                }
            }
            "unknown" => {
                bad.input = TableTransportInput::InspirationTransfer {
                    handle: CommandId::new(),
                }
            }
            "raw-award" => {
                bad.input = TableTransportInput::InspirationTransfer {
                    handle: f
                        .state()
                        .await
                        .table
                        .unwrap()
                        .inspiration_transfer
                        .unwrap()
                        .origin
                        .id,
                }
            }
            "old-version" => bad.version = 3,
            "session" => bad.session_id = Some(PlaySessionId::new()),
            "revision" => bad.revision = f.view(TableTransportChannel::Host).await.revision,
            "campaign" => bad.campaign_id = CampaignId::new(),
            _ => unreachable!(),
        }
        Box::pin(f.reject(bad)).await;
    }
    let state = f.state().await;
    let award = state
        .table
        .as_ref()
        .unwrap()
        .inspiration_transfer
        .as_ref()
        .unwrap()
        .origin
        .id;
    for recipient in [
        Some(f.characters[0]),
        Some(CharacterId::new()),
        Some(foreign.characters[0]),
        Some(CharacterId(f.goblin.0)),
        None,
    ] {
        let canonical = TableAction::ResolveHostInspirationTransfer {
            choice: TableInspirationTransferChoice {
                award,
                character_id: f.characters[0],
                recipient,
            },
        };
        let bad = request(
            &f,
            f.pc(0),
            TableTransportInput::Action(Box::new(canonical.clone())),
        )
        .await;
        Box::pin(f.reject(bad)).await;
        for issuer in [
            CommandIssuer::Admin,
            CommandIssuer::System,
            CommandIssuer::Import,
            CommandIssuer::Player(f.players[0]),
            CommandIssuer::Player(f.players[1]),
        ] {
            let rows = all_rows(&f.pool).await;
            let meta = CommandMeta {
                id: CommandId::new(),
                campaign_id: f.campaign,
                session_id: Some(f.session),
                issuer,
                actor: Some(AgentRef::Entity(f.actors[0])),
                expected_event_sequence: state.applied_event_sequence,
            };
            assert!(
                Box::pin(f.runtime.execute_table(meta, canonical.clone()))
                    .await
                    .is_err()
            );
            assert!(
                Box::pin(f.runtime.execute_rules(
                    RulesContext {
                        campaign_id: f.campaign,
                        issuer,
                        actor: Some(f.actors[0]),
                        session_id: Some(f.session),
                        expected_event_sequence: state.applied_event_sequence
                    },
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
    }
    Box::pin(f.cold(valid)).await;
    foreign.close().await;
    f.close().await;
}

#[tokio::test]
async fn pending_choice_blocks_other_actor_gameplay_session_and_host_mutations_but_keeps_reads_questions_and_retries()
 {
    let mut f = Box::pin(Fixture::new()).await;
    enable(&mut f).await;
    let pc = f.pc(0);
    let dodge = Box::pin(act(&mut f, pc, TacticalAction::Dodge)).await;
    let original = Box::pin(f.runtime.submit_presented_table(dodge.clone()))
        .await
        .unwrap();
    grant_to(&mut f, 1).await;
    extra(&mut f, 1).await;
    let before = f.state().await;
    for (channel, input) in [
        (f.pc(0), action(TacticalAction::EndTurn)),
        (
            TableTransportChannel::Host,
            action(TacticalAction::ConcludeHostilities {
                cadence: AftermathCadence::ContinueExistingOrder,
                ruling: "Stop fighting.".into(),
            }),
        ),
        (
            TableTransportChannel::Host,
            TableTransportInput::Action(Box::new(TableAction::EndSession)),
        ),
        (
            TableTransportChannel::Host,
            TableTransportInput::Action(Box::new(TableAction::SetSourceCreatureController {
                actor: f.goblin,
                controller: CreatureController::Player(f.players[0]),
            })),
        ),
        (TableTransportChannel::Host, excess(f.characters[1])),
        (TableTransportChannel::Host, award(f.characters[0], REASON)),
        (
            f.pc(0),
            TableTransportInput::Text {
                text: "I climb the wall.".into(),
            },
        ),
    ] {
        let bad = request(&f, channel, input).await;
        Box::pin(f.reject(bad)).await;
    }
    let rows = all_rows(&f.pool).await;
    assert_eq!(
        Box::pin(f.runtime.submit_presented_table(dodge))
            .await
            .unwrap(),
        original
    );
    assert_eq!(all_rows(&f.pool).await, rows);
    let question = request(
        &f,
        f.pc(0),
        TableTransportInput::Text {
            text: "What is my health?".into(),
        },
    )
    .await;
    assert!(matches!(
        Box::pin(f.runtime.submit_presented_table(question))
            .await
            .unwrap(),
        TableTransportResult::Observed(_)
    ));
    assert_eq!(f.state().await, before);
    let decline = decision(&f, 1, "Decline the extra Inspiration").await;
    Box::pin(f.cold(decline)).await;
    f.close().await;
}

#[tokio::test]
async fn excess_requires_real_inspiration_settled_host_authority_and_existing_supported_source() {
    let mut f = Box::pin(Fixture::new()).await;
    Box::pin(f.activate()).await;
    let before = f.state().await;
    assert!(
        !serde_json::to_value(before.table.as_ref().unwrap())
            .unwrap()
            .as_object()
            .unwrap()
            .contains_key("inspiration_transfer")
    );
    assert!(
        !serde_json::to_value(f.view(f.pc(0)).await)
            .unwrap()
            .as_object()
            .unwrap()
            .contains_key("inspiration_transfer")
    );
    let rejected = f
        .request(TableTransportChannel::Host, excess(f.characters[0]))
        .await;
    Box::pin(f.reject(rejected)).await;
    let activation = request(
        &f,
        TableTransportChannel::Host,
        TableTransportInput::Action(Box::new(TableAction::EnableGrappleTransport)),
    )
    .await;
    Box::pin(f.cold(activation)).await;
    let absent_resource = request(&f, TableTransportChannel::Host, excess(f.characters[0])).await;
    Box::pin(f.reject(absent_resource)).await;
    grant(&mut f).await;
    for channel in [
        f.pc(0),
        f.pc(1),
        TableTransportChannel::SourceCreature {
            player_id: f.players[0],
            actor: f.goblin,
        },
    ] {
        let bad = request(&f, channel, excess(f.characters[0])).await;
        Box::pin(f.reject(bad)).await;
    }
    for character_id in [CharacterId::new(), CharacterId(f.goblin.0)] {
        let bad = request(&f, TableTransportChannel::Host, excess(character_id)).await;
        Box::pin(f.reject(bad)).await;
    }
    for reason in [" ".into(), "é".repeat(1001)] {
        let bad = request(
            &f,
            TableTransportChannel::Host,
            TableTransportInput::Action(Box::new(TableAction::AwardExcessInspiration {
                character_id: f.characters[0],
                reason,
            })),
        )
        .await;
        Box::pin(f.reject(bad)).await;
    }
    target_pc(&mut f, "Strength").await;
    let pending_roll = request(&f, TableTransportChannel::Host, excess(f.characters[0])).await;
    Box::pin(f.reject(pending_roll)).await;
    f.close().await;
}

#[tokio::test]
async fn inspired_recipient_is_not_an_option_and_no_recipient_is_chosen_automatically() {
    let mut f = Box::pin(Fixture::new()).await;
    enable(&mut f).await;
    grant(&mut f).await;
    grant_to(&mut f, 1).await;
    extra(&mut f, 0).await;
    let view = f.view(f.pc(0)).await;
    assert_eq!(
        view.inspiration_transfer
            .unwrap()
            .choices
            .iter()
            .map(|choice| choice.label.as_str())
            .collect::<Vec<_>>(),
        vec!["Decline the extra Inspiration"]
    );
    let state = f.state().await;
    assert!(has_inspiration(&state, f.actors[0]) && has_inspiration(&state, f.actors[1]));
    assert!(pending(&state, f.actors[0]));
    let decline = decision(&f, 0, "Decline the extra Inspiration").await;
    Box::pin(f.cold(decline)).await;
    assert!(has_inspiration(&f.state().await, f.actors[1]));
    f.close().await;
}

#[tokio::test]
async fn absent_recipient_may_receive_passively_but_absent_owner_cannot_be_given_a_pending_choice()
{
    for giver in [0, 1] {
        let mut f = Box::pin(Fixture::new()).await;
        enable(&mut f).await;
        grant_to(&mut f, giver).await;
        next_session(&mut f, Some(1)).await;
        if giver == 1 {
            let bad = request(&f, TableTransportChannel::Host, excess(f.characters[1])).await;
            Box::pin(f.reject(bad)).await;
        } else {
            extra(&mut f, 0).await;
            let command = decision(&f, 0, "Give to Character 1").await;
            Box::pin(f.cold(command)).await;
            assert!(has_inspiration(&f.state().await, f.actors[1]));
        }
        f.close().await;
    }
}

#[tokio::test]
async fn genuinely_transferred_resource_pays_for_physical_grapple_save_and_escape_and_old_receipts_survive()
 {
    for escape_roll in [false, true] {
        let mut f = Box::pin(Fixture::new()).await;
        enable(&mut f).await;
        grant_to(&mut f, 1).await;
        let awarded = extra(&mut f, 1).await;
        let transfer = decision(&f, 1, "Give to Character 0").await;
        Box::pin(f.cold(transfer.clone())).await;
        let award_receipt = Box::pin(f.runtime.submit_presented_table(awarded.clone()))
            .await
            .unwrap();
        let transfer_receipt = Box::pin(f.runtime.submit_presented_table(transfer.clone()))
            .await
            .unwrap();
        target_pc(&mut f, "Strength").await;
        if escape_roll {
            let pc = f.pc(0);
            Box::pin(act(&mut f, pc, TacticalAction::VoluntarilyFailSave)).await;
            finish_goblin(&mut f).await;
            escape(&mut f, "Athletics").await;
        }
        assert_eq!(options(&f, f.pc(0)).await.heroic_inspiration, Some(true));
        let before = f.state().await;
        let old_roll = before.rules.as_ref().unwrap().pending.as_ref().unwrap();
        let command = inspired(&f, 1, 20).await;
        Box::pin(f.cold(command.clone())).await;
        let after = f.state().await;
        let rules = after.rules.as_ref().unwrap();
        let result = rules.rolls.last().unwrap();
        assert_eq!(result.request, old_roll.request);
        assert_eq!(
            result.original_result.as_ref().unwrap().dice,
            vec![DieResult {
                sides: 20,
                value: 1
            }]
        );
        assert_eq!(
            result.result.dice,
            vec![DieResult {
                sides: 20,
                value: 20
            }]
        );
        assert_eq!(result.resolved.total, 25);
        assert_eq!(result.accepted_by.id, command.command_id);
        assert_eq!(
            result.accepted_by.issuer,
            CommandIssuer::Player(f.players[0])
        );
        assert!(!has_inspiration(&after, f.actors[0]));
        assert!(has_inspiration(&after, f.actors[1]));
        assert_eq!(
            rules
                .tactical_grapples
                .as_ref()
                .map_or(0, |graph| graph.active.len()),
            0
        );
        if !escape_roll {
            finish_goblin(&mut f).await;
        }
        next_session(&mut f, None).await;
        let rows = all_rows(&f.pool).await;
        assert_eq!(
            Box::pin(f.runtime.submit_presented_table(awarded))
                .await
                .unwrap(),
            award_receipt
        );
        assert_eq!(
            Box::pin(f.runtime.submit_presented_table(transfer))
                .await
                .unwrap(),
            transfer_receipt
        );
        assert_eq!(all_rows(&f.pool).await, rows);
        assert!(!has_inspiration(&f.state().await, f.actors[0]));
        f.close().await;
    }
}

#[tokio::test]
async fn host_transfer_flag_provenance_ruling_and_pending_snapshot_cannot_be_forged_or_relabelled_resourceful()
 {
    let mut f = Box::pin(Fixture::new()).await;
    enable(&mut f).await;
    let first = grant(&mut f).await;
    let award = extra(&mut f, 0).await;
    let state = f.state().await;
    for mutation in [
        "flag",
        "companion",
        "resource",
        "origin",
        "original-award",
        "owner",
        "ruling",
        "other-flag",
        "dead-owner",
        "absent-owner",
    ] {
        let mut forged = state.clone();
        match mutation {
            "flag" => {
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
                    .inspiration_transfer_pending = false
            }
            "companion" => forged.table.as_mut().unwrap().inspiration_transfer = None,
            "resource" => {
                forged
                    .rules
                    .as_mut()
                    .unwrap()
                    .entities
                    .get_mut(&f.actors[0])
                    .unwrap()
                    .heroic_inspiration = false
            }
            "origin" => {
                forged
                    .table
                    .as_mut()
                    .unwrap()
                    .inspiration_transfer
                    .as_mut()
                    .unwrap()
                    .origin
                    .id = CommandId::new()
            }
            "original-award" => {
                forged
                    .table
                    .as_mut()
                    .unwrap()
                    .inspiration_transfer
                    .as_mut()
                    .unwrap()
                    .origin
                    .id = first.command_id
            }
            "owner" => {
                forged
                    .table
                    .as_mut()
                    .unwrap()
                    .inspiration_transfer
                    .as_mut()
                    .unwrap()
                    .character_id = f.characters[1]
            }
            "ruling" => {
                forged
                    .rules
                    .as_mut()
                    .unwrap()
                    .rulings
                    .last_mut()
                    .unwrap()
                    .ruling
                    .reason = "Invented excess".into()
            }
            "other-flag" => {
                let entity = forged
                    .rules
                    .as_mut()
                    .unwrap()
                    .entities
                    .get_mut(&f.actors[1])
                    .unwrap();
                entity.heroic_inspiration = true;
                entity
                    .character_features
                    .as_mut()
                    .unwrap()
                    .inspiration_transfer_pending = true;
            }
            "dead-owner" => {
                forged.characters.get_mut(&f.characters[0]).unwrap().status = CharacterStatus::Dead
            }
            "absent-owner" => {
                forged
                    .table
                    .as_mut()
                    .unwrap()
                    .active_session
                    .as_mut()
                    .unwrap()
                    .participants[0]
                    .attendance = AttendanceStatus::Absent
            }
            _ => unreachable!(),
        }
        Box::pin(reject_state_image(&f, forged)).await;
    }
    let export = export_campaign(&f.pool, f.campaign).await.unwrap();
    for mutation in [
        "reason",
        "issuer",
        "recipient",
        "first-award",
        "nested",
        "audit",
        "anchor",
    ] {
        let mut forged = export.clone();
        if mutation == "audit" {
            forged
                .command_audit
                .iter_mut()
                .find(|row| row.id == award.command_id.0.to_string())
                .unwrap()
                .command_schema_version = 4;
        } else if mutation == "anchor" {
            let anchor = forged
                .snapshots
                .iter_mut()
                .min_by_key(|row| row.event_sequence)
                .unwrap();
            let mut image = CampaignState::decode_json(&anchor.state_json).unwrap();
            image.table.as_mut().unwrap().inspiration_transfer =
                state.table.as_ref().unwrap().inspiration_transfer.clone();
            anchor.state_json = image.encode_json().unwrap();
        } else {
            let row = forged
                .event_journal
                .iter_mut()
                .find(|row| row.command_id == award.command_id.0.to_string())
                .unwrap();
            let mut event: TableEvent = serde_json::from_str(&row.payload_json).unwrap();
            match mutation {
                "reason" => {
                    event.action = TableAction::AwardExcessInspiration {
                        character_id: f.characters[0],
                        reason: "Invented".into(),
                    }
                }
                "issuer" => event.meta.issuer = CommandIssuer::System,
                "recipient" => {
                    event.action = TableAction::AwardExcessInspiration {
                        character_id: f.characters[1],
                        reason: EXTRA.into(),
                    }
                }
                "first-award" => {
                    event.action = TableAction::AwardHeroicInspiration {
                        character_id: f.characters[0],
                        reason: EXTRA.into(),
                    }
                }
                "nested" => event.outcome.mechanics = Some(dmd_rules::RulesOutcome::Changed),
                _ => unreachable!(),
            }
            row.payload_json = serde_json::to_string(&event).unwrap();
        }
        Box::pin(hostile_destination(&forged)).await;
    }
    f.close().await;
}

#[tokio::test]
async fn accepted_transfer_recipient_authority_ruling_and_restored_resource_are_bound_to_actual_history()
 {
    let mut f = Box::pin(Fixture::new()).await;
    enable(&mut f).await;
    grant(&mut f).await;
    extra(&mut f, 0).await;
    let command = decision(&f, 0, "Give to Character 1").await;
    Box::pin(f.cold(command.clone())).await;
    let state = f.state().await;
    for actor in f.actors {
        let mut forged = state.clone();
        forged
            .rules
            .as_mut()
            .unwrap()
            .entities
            .get_mut(&actor)
            .unwrap()
            .heroic_inspiration = false;
        Box::pin(reject_state_image(&f, forged)).await;
    }
    let export = export_campaign(&f.pool, f.campaign).await.unwrap();
    for mutation in [
        "declined", "self", "unknown", "award", "host", "system", "other", "nested",
    ] {
        let mut forged = export.clone();
        let row = forged
            .event_journal
            .iter_mut()
            .find(|row| row.command_id == command.command_id.0.to_string())
            .unwrap();
        let mut event: TableEvent = serde_json::from_str(&row.payload_json).unwrap();
        let TableAction::ResolveHostInspirationTransfer { choice } = &mut event.action else {
            unreachable!()
        };
        match mutation {
            "declined" => choice.recipient = None,
            "self" => choice.recipient = Some(f.characters[0]),
            "unknown" => choice.recipient = Some(CharacterId::new()),
            "award" => choice.award = CommandId::new(),
            "host" => event.meta.issuer = CommandIssuer::Admin,
            "system" => event.meta.issuer = CommandIssuer::System,
            "other" => event.meta.issuer = CommandIssuer::Player(f.players[1]),
            "nested" => event.outcome.mechanics = Some(dmd_rules::RulesOutcome::Changed),
            _ => unreachable!(),
        }
        row.payload_json = serde_json::to_string(&event).unwrap();
        Box::pin(hostile_destination(&forged)).await;
    }
    f.close().await;
}

#[tokio::test]
async fn genuinely_dead_pc_is_excluded_from_transfer_after_source_creature_critical_damage() {
    let mut f = Box::pin(Fixture::with_source("chimera", CreatureSize::Large)).await;
    enable(&mut f).await;
    grant_to(&mut f, 1).await;
    let pc = f.pc(0);
    let target = f.actors[0];
    Box::pin(act(&mut f, pc.clone(), TacticalAction::EndTurn)).await;
    Box::pin(act(
        &mut f,
        TableTransportChannel::Host,
        TacticalAction::CreatureAttack {
            target,
            feature_id: "bite".into(),
            weapon: None,
        },
    ))
    .await;
    physical(&mut f, TableTransportChannel::Host, 20).await;
    let order = f
        .view(TableTransportChannel::Host)
        .await
        .tactical
        .unwrap()
        .hit
        .unwrap()
        .order
        .unwrap();
    let command = request(
        &f,
        TableTransportChannel::Host,
        TableTransportInput::HitResponse {
            handle: order.key,
            decision: Box::new(TableHitInput::Order {
                instruction: TacticalReactionOrdering {
                    ranked: vec![],
                    unlisted: ReactionUnlistedOrder::AfterForward,
                },
            }),
        },
    )
    .await;
    Box::pin(f.cold(command)).await;
    let response = f
        .view(pc.clone())
        .await
        .tactical
        .unwrap()
        .hit
        .unwrap()
        .response
        .unwrap();
    let command = request(
        &f,
        pc,
        TableTransportInput::HitResponse {
            handle: response.key,
            decision: Box::new(TableHitInput::Respond { accept: false }),
        },
    )
    .await;
    Box::pin(f.cold(command)).await;
    assert_eq!(
        f.view(TableTransportChannel::Host).await.roll.unwrap().dice,
        vec![DieSpec { count: 4, sides: 6 }]
    );
    physical(&mut f, TableTransportChannel::Host, 6).await;
    Box::pin(act(
        &mut f,
        TableTransportChannel::Host,
        TacticalAction::ChooseAttackKnockout {
            choice: KnockoutChoice::NormalDamage,
        },
    ))
    .await;
    let dead = f.state().await;
    assert_eq!(
        dead.characters[&f.characters[0]].status,
        CharacterStatus::Dead
    );
    assert!(dead.rules.as_ref().unwrap().entities[&target].death.dead);
    assert_eq!(dead.rules.as_ref().unwrap().entities[&target].hp, 0);
    assert!(
        dead.encounter
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .resolution
            .is_none()
    );
    extra(&mut f, 1).await;
    let shown = f.view(f.pc(1)).await.inspiration_transfer.unwrap();
    assert_eq!(
        shown
            .choices
            .iter()
            .map(|choice| choice.label.as_str())
            .collect::<Vec<_>>(),
        vec!["Decline the extra Inspiration"]
    );
    let decline = decision(&f, 1, "Decline the extra Inspiration").await;
    Box::pin(f.cold(decline)).await;
    assert!(has_inspiration(&f.state().await, f.actors[1]));
    assert!(!has_inspiration(&f.state().await, target));
    f.close().await;
}

#[tokio::test]
async fn opaque_capability_and_saved_binding_cannot_change_the_giver_recipient_or_decline_on_restore()
 {
    let mut f = Box::pin(Fixture::new()).await;
    enable(&mut f).await;
    grant(&mut f).await;
    extra(&mut f, 0).await;
    let pending_export = export_campaign(&f.pool, f.campaign).await.unwrap();
    for mutation in ["self", "unknown", "decline", "award", "owner"] {
        let mut forged = pending_export.clone();
        let option = forged
            .table_projection_history
            .iter_mut()
            .rev()
            .flat_map(|record| record.changes.iter_mut())
            .flat_map(|change| change.handles.iter_mut())
            .find_map(|handle| match &mut handle.capability {
                dmd_persistence::ProjectionCapability::InspirationTransfer { choice }
                    if choice.recipient.is_some() =>
                {
                    Some(choice)
                }
                _ => None,
            })
            .unwrap();
        match mutation {
            "self" => option.recipient = Some(f.characters[0]),
            "unknown" => option.recipient = Some(CharacterId::new()),
            "decline" => option.recipient = None,
            "award" => option.award = CommandId::new(),
            "owner" => option.character_id = f.characters[1],
            _ => unreachable!(),
        }
        Box::pin(hostile_destination(&forged)).await;
    }
    let command = decision(&f, 0, "Give to Character 1").await;
    Box::pin(f.cold(command.clone())).await;
    let accepted = export_campaign(&f.pool, f.campaign).await.unwrap();
    for mutation in ["handle", "channel", "version", "session"] {
        let mut forged = accepted.clone();
        let binding = forged
            .table_transport_bindings
            .iter_mut()
            .find(|binding| binding.meta.id == command.command_id)
            .unwrap();
        let mut input: TableTransportRequest = serde_json::from_str(&binding.request_json).unwrap();
        match mutation {
            "handle" => {
                input.input = TableTransportInput::InspirationTransfer {
                    handle: CommandId::new(),
                }
            }
            "channel" => input.channel = TableTransportChannel::Host,
            "version" => input.version = 3,
            "session" => input.session_id = Some(PlaySessionId::new()),
            _ => unreachable!(),
        }
        binding.request_json = serde_json::to_string(&input).unwrap();
        Box::pin(hostile_destination(&forged)).await;
    }
    f.close().await;
}
