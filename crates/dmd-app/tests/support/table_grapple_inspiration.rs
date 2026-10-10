//! Genuine Host awards and physical PC rerolls through original owned history.
//! Mutated states are hostile recovery inputs only, never accepted setup.
use super::*;

const REASON: &str = "For protecting the retreating group.";

async fn request(
    f: &Fixture,
    channel: TableTransportChannel,
    input: TableTransportInput,
) -> TableTransportRequest {
    let mut request = f.request(channel, input).await;
    request.version = 4;
    request
}

#[path = "table_inspiration_transfer.rs"]
mod transfer;
fn award(character_id: CharacterId, reason: &str) -> TableTransportInput {
    TableTransportInput::Action(Box::new(TableAction::AwardHeroicInspiration {
        character_id,
        reason: reason.into(),
    }))
}
async fn enable(f: &mut Fixture) {
    Box::pin(f.activate()).await;
    let command = request(
        f,
        TableTransportChannel::Host,
        TableTransportInput::Action(Box::new(TableAction::EnableGrappleTransport)),
    )
    .await;
    Box::pin(f.cold(command)).await;
}
async fn grant(f: &mut Fixture) -> TableTransportRequest {
    let command = request(
        f,
        TableTransportChannel::Host,
        award(f.characters[0], REASON),
    )
    .await;
    Box::pin(f.cold(command.clone())).await;
    command
}
async fn act(
    f: &mut Fixture,
    channel: TableTransportChannel,
    input: TacticalAction,
) -> TableTransportRequest {
    let command = request(f, channel, action(input)).await;
    Box::pin(f.cold(command.clone())).await;
    command
}
async fn choose(f: &mut Fixture, channel: TableTransportChannel, label: &str) {
    let mut command = f.choose(channel, label).await;
    command.version = 4;
    Box::pin(f.cold(command)).await;
}
async fn target_pc(f: &mut Fixture, save: &str) {
    let pc = f.pc(0);
    Box::pin(act(f, pc.clone(), TacticalAction::EndTurn)).await;
    let offer = f
        .view(TableTransportChannel::Host)
        .await
        .grapple
        .unwrap()
        .choices
        .into_iter()
        .find(|option| {
            option.actor == f.goblin && option.label == "Grapple Character 0 with right hand"
        })
        .unwrap();
    let command = request(
        f,
        TableTransportChannel::Host,
        TableTransportInput::GrappleChoice { handle: offer.key },
    )
    .await;
    Box::pin(f.cold(command)).await;
    Box::pin(choose(f, pc, &format!("Resist Grapple with {save}"))).await;
}
async fn inspired(f: &Fixture, original: u16, replacement: u16) -> TableTransportRequest {
    let roll = f.view(f.pc(0)).await.roll.unwrap();
    assert_eq!(roll.mode, RollMode::Normal);
    assert_eq!(
        roll.dice,
        vec![DieSpec {
            count: 1,
            sides: 20
        }]
    );
    request(
        f,
        f.pc(0),
        action(TacticalAction::SubmitRollWithInspiration {
            result: RollResult {
                request_id: roll.id,
                source: RollSource::Physical,
                dice: vec![DieResult {
                    sides: 20,
                    value: original,
                }],
            },
            die_index: 0,
            replacement: DieResult {
                sides: 20,
                value: replacement,
            },
        }),
    )
    .await
}
async fn finish_goblin(f: &mut Fixture) {
    Box::pin(choose(
        f,
        TableTransportChannel::Host,
        "Finish without changing equipment",
    ))
    .await;
}
async fn escape(f: &mut Fixture, skill: &str) {
    Box::pin(act(f, TableTransportChannel::Host, TacticalAction::EndTurn)).await;
    let offer = f
        .view(f.pc(0))
        .await
        .grapple
        .unwrap()
        .choices
        .into_iter()
        .find(|option| {
            option.actor == f.actors[0] && option.label.ends_with(&format!("using {skill}"))
        })
        .unwrap();
    let command = request(
        f,
        f.pc(0),
        TableTransportInput::GrappleChoice { handle: offer.key },
    )
    .await;
    Box::pin(f.cold(command)).await;
}
async fn options(f: &Fixture, channel: TableTransportChannel) -> TableRollOptions {
    let view = f.view(channel.clone()).await;
    f.runtime
        .table_roll_options(TableRollOptionsRequest {
            campaign_id: f.campaign,
            channel,
            revision: view.revision,
            roll_id: view.roll.unwrap().id,
        })
        .await
        .unwrap()
}

#[tokio::test]
async fn preactivation_v3_roll_options_keep_the_old_bytes_and_read_only_audience_history() {
    let mut f = Box::pin(Fixture::new()).await;
    Box::pin(f.activate()).await;
    let attempt = f
        .choose(f.pc(0), "Grapple Small armored figure with left hand")
        .await;
    Box::pin(f.cold(attempt)).await;
    let save = f
        .choose(TableTransportChannel::Host, "Resist Grapple with Strength")
        .await;
    Box::pin(f.cold(save)).await;
    let rows = all_rows(&f.pool).await;
    let before = f.view(TableTransportChannel::Host).await;
    let answer = options(&f, TableTransportChannel::Host).await;
    assert_eq!(answer.heroic_inspiration, None);
    assert_eq!(
        serde_json::to_value(answer).unwrap(),
        serde_json::json!({"savage_attacker":null})
    );
    assert_eq!(f.view(TableTransportChannel::Host).await, before);
    assert_eq!(all_rows(&f.pool).await, rows);
    let bad = f
        .request(TableTransportChannel::Host, award(f.characters[0], REASON))
        .await;
    Box::pin(f.reject(bad)).await;
    f.close().await;
}

#[tokio::test]
async fn host_award_changes_only_the_real_pc_resource_and_exact_outer_ruling_and_private_projection()
 {
    let mut f = Box::pin(Fixture::new()).await;
    enable(&mut f).await;
    let before = f.state().await;
    assert!(!before.rules.as_ref().unwrap().entities[&f.actors[0]].heroic_inspiration);
    let private = f.view(f.pc(1)).await;
    let retained = f
        .runtime
        .table_view(f.campaign, TableViewer::Player(f.players[1]))
        .await
        .unwrap();
    let owner = f.view(f.pc(0)).await;
    let command = grant(&mut f).await;
    let after = f.state().await;
    let rules = after.rules.as_ref().unwrap();
    assert!(rules.entities[&f.actors[0]].heroic_inspiration);
    let ruling = rules.rulings.last().unwrap();
    assert_eq!(ruling.command.id, command.command_id);
    assert_eq!(ruling.command.issuer, CommandIssuer::Admin);
    assert_eq!(ruling.command.actor, None);
    assert_eq!(ruling.command.session_id, Some(f.session));
    assert_eq!(
        ruling.ruling,
        Ruling {
            basis: RulingBasis::Srd { page: 8 },
            reason: REASON.into()
        }
    );
    let mut restored = after.clone();
    restored.applied_event_sequence = before.applied_event_sequence;
    restored
        .rules
        .as_mut()
        .unwrap()
        .entities
        .get_mut(&f.actors[0])
        .unwrap()
        .heroic_inspiration = false;
    restored.rules.as_mut().unwrap().rulings.pop();
    assert_eq!(
        restored, before,
        "award changed state beyond resource, ruling and accepted sequence"
    );
    Box::pin(assert_private_history(&f, &retained, &private)).await;
    assert_ne!(f.view(f.pc(0)).await.revision, owner.revision);
    let export = export_campaign(&f.pool, f.campaign).await.unwrap();
    let event = export
        .event_journal
        .iter()
        .find(|row| row.command_id == command.command_id.0.to_string())
        .unwrap();
    let event: TableEvent = serde_json::from_str(&event.payload_json).unwrap();
    assert_eq!(
        event.action,
        TableAction::AwardHeroicInspiration {
            character_id: f.characters[0],
            reason: REASON.into()
        }
    );
    assert_eq!(event.meta, ruling.command);
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
async fn award_refuses_old_transport_foreign_authority_bad_identity_and_invalid_reason_without_writes()
 {
    let mut f = Box::pin(Fixture::new()).await;
    Box::pin(f.activate()).await;
    let old = f
        .request(TableTransportChannel::Host, award(f.characters[0], REASON))
        .await;
    Box::pin(f.reject(old)).await;
    let premature = request(
        &f,
        TableTransportChannel::Host,
        award(f.characters[0], REASON),
    )
    .await;
    Box::pin(f.reject(premature)).await;
    let activation = request(
        &f,
        TableTransportChannel::Host,
        TableTransportInput::Action(Box::new(TableAction::EnableGrappleTransport)),
    )
    .await;
    Box::pin(f.cold(activation)).await;
    for channel in [
        f.pc(0),
        f.pc(1),
        TableTransportChannel::SourceCreature {
            player_id: f.players[0],
            actor: f.goblin,
        },
    ] {
        let bad = request(&f, channel, award(f.characters[0], REASON)).await;
        Box::pin(f.reject(bad)).await;
    }
    for (recipient, reason) in [
        (CharacterId::new(), REASON.to_owned()),
        (CharacterId(f.goblin.0), REASON.into()),
        (f.characters[0], " ".into()),
        (f.characters[0], "é".repeat(1001)),
    ] {
        let bad = request(&f, TableTransportChannel::Host, award(recipient, &reason)).await;
        Box::pin(f.reject(bad)).await;
    }
    let valid = request(
        &f,
        TableTransportChannel::Host,
        award(f.characters[0], REASON),
    )
    .await;
    for mutation in ["version", "campaign", "session", "revision"] {
        let mut bad = valid.clone();
        bad.command_id = CommandId::new();
        match mutation {
            "version" => bad.version = 3,
            "campaign" => bad.campaign_id = CampaignId::new(),
            "session" => bad.session_id = Some(PlaySessionId::new()),
            "revision" => bad.revision = f.view(f.pc(0)).await.revision,
            _ => unreachable!(),
        }
        Box::pin(f.reject(bad)).await;
    }
    grant(&mut f).await;
    Box::pin(f.reject(valid)).await;
    let duplicate = request(
        &f,
        TableTransportChannel::Host,
        award(f.characters[0], "Another award must not stack or transfer."),
    )
    .await;
    Box::pin(f.reject(duplicate)).await;
    assert!(!f.state().await.rules.unwrap().entities[&f.actors[1]].heroic_inspiration);
    f.close().await;
}

#[tokio::test]
async fn direct_table_and_raw_kernel_grant_remain_refused_after_shared_activation() {
    let mut f = Box::pin(Fixture::new()).await;
    enable(&mut f).await;
    let before = all_rows(&f.pool).await;
    let state = f.state().await;
    let context = RulesContext {
        campaign_id: f.campaign,
        issuer: CommandIssuer::Admin,
        actor: None,
        session_id: Some(f.session),
        expected_event_sequence: state.applied_event_sequence,
    };
    let meta = CommandMeta {
        id: CommandId::new(),
        campaign_id: f.campaign,
        issuer: CommandIssuer::Admin,
        actor: None,
        session_id: Some(f.session),
        expected_event_sequence: state.applied_event_sequence,
    };
    assert!(
        Box::pin(f.runtime.execute_table(
            meta,
            TableAction::AwardHeroicInspiration {
                character_id: f.characters[0],
                reason: REASON.into()
            }
        ))
        .await
        .is_err()
    );
    assert!(
        Box::pin(f.runtime.execute_rules(
            context,
            dmd_rules::RulesAction::GrantInspiration {
                actor: f.actors[0],
                ruling: Ruling {
                    basis: RulingBasis::Srd { page: 8 },
                    reason: REASON.into()
                },
            }
        ))
        .await
        .is_err()
    );
    assert_eq!(all_rows(&f.pool).await, before);
    assert_eq!(f.state().await, state);
    f.close().await;
}

#[tokio::test]
async fn genuine_pc_strength_and_dexterity_saves_use_original_and_mandatory_higher_or_lower_replacement()
 {
    for (ability, modifier) in [("Strength", 5), ("Dexterity", 2)] {
        for (original, replacement, success) in [(1, 20, true), (20, 1, false)] {
            let mut f = Box::pin(Fixture::new()).await;
            enable(&mut f).await;
            grant(&mut f).await;
            target_pc(&mut f, ability).await;
            let pending_state = f.state().await;
            let pending = pending_state
                .rules
                .as_ref()
                .unwrap()
                .pending
                .as_ref()
                .unwrap();
            assert_eq!(pending.request.roller, Some(f.actors[0]));
            assert_eq!(pending.request.modifier, modifier);
            assert!(
                matches!(pending.purpose, PendingPurpose::TacticalResolution { key, .. } if key.role == TacticalRollRole::GrappleSave)
            );
            let Some(GrappleActivity::Attempt(attempt)) = &resolution(&pending_state)
                .grapple
                .as_ref()
                .unwrap()
                .activity
            else {
                panic!("expected genuine save activity")
            };
            assert_eq!(attempt.declaration.grappler, f.goblin);
            assert_eq!(attempt.declaration.target, f.actors[0]);
            assert_eq!(
                attempt.declaration.escape_dc, 9,
                "current Goblin Strength -1 and proficiency +2"
            );
            assert_eq!(options(&f, f.pc(0)).await.heroic_inspiration, Some(true));
            let command = inspired(&f, original, replacement).await;
            Box::pin(f.cold(command.clone())).await;
            let after = f.state().await;
            let rules = after.rules.as_ref().unwrap();
            let recorded = rules.rolls.last().unwrap();
            assert_eq!(recorded.request, pending.request);
            assert_eq!(recorded.issued_by, pending.issued_by);
            assert_eq!(recorded.purpose, pending.purpose);
            assert_eq!(recorded.accepted_by.id, command.command_id);
            assert_eq!(
                recorded.accepted_by.issuer,
                CommandIssuer::Player(f.players[0])
            );
            assert_eq!(
                recorded.original_result.as_ref().unwrap().dice,
                vec![DieResult {
                    sides: 20,
                    value: original
                }]
            );
            assert_eq!(
                recorded.original_result.as_ref().unwrap().request_id,
                pending.request.id
            );
            assert_eq!(
                recorded.original_result.as_ref().unwrap().source,
                RollSource::Physical
            );
            assert_eq!(
                recorded.result.dice,
                vec![DieResult {
                    sides: 20,
                    value: replacement
                }]
            );
            assert_eq!(recorded.resolved.total, i32::from(replacement) + modifier);
            assert!(!rules.entities[&f.actors[0]].heroic_inspiration);
            let grips = rules
                .tactical_grapples
                .as_ref()
                .map_or(0, |graph| graph.active.len());
            assert_eq!(grips, usize::from(!success));
            assert_eq!(
                rules.rolls.len(),
                pending_state.rules.as_ref().unwrap().rolls.len() + 1
            );
            finish_goblin(&mut f).await;
            f.close().await;
        }
    }
}

#[tokio::test]
async fn genuine_pc_athletics_and_acrobatics_escape_spend_one_action_and_inspiration_on_success_or_failure()
 {
    for (skill, modifier) in [("Athletics", 5), ("Acrobatics", 4)] {
        for (original, replacement, success) in [(1, 20, true), (20, 1, false)] {
            let mut f = Box::pin(Fixture::new()).await;
            enable(&mut f).await;
            grant(&mut f).await;
            target_pc(&mut f, "Strength").await;
            let pc = f.pc(0);
            Box::pin(act(&mut f, pc, TacticalAction::VoluntarilyFailSave)).await;
            let declaration = f
                .state()
                .await
                .rules
                .unwrap()
                .tactical_grapples
                .unwrap()
                .active[0]
                .declaration
                .clone();
            let grip = declaration.id;
            assert_eq!(declaration.escape_dc, 9);
            finish_goblin(&mut f).await;
            escape(&mut f, skill).await;
            let before = f.state().await;
            let pending = before.rules.as_ref().unwrap().pending.as_ref().unwrap();
            assert_eq!(pending.request.modifier, modifier);
            assert!(
                matches!(pending.purpose, PendingPurpose::TacticalResolution { key, .. } if key.role == TacticalRollRole::GrappleEscape)
            );
            assert!(
                before
                    .rules
                    .as_ref()
                    .unwrap()
                    .timing
                    .as_ref()
                    .unwrap()
                    .action_spent
            );
            for forbidden in [
                TacticalAction::VoluntarilyFailSave,
                TacticalAction::UseLegendaryResistance,
            ] {
                let bad = request(&f, f.pc(0), action(forbidden)).await;
                Box::pin(f.reject(bad)).await;
            }
            let command = inspired(&f, original, replacement).await;
            Box::pin(f.cold(command.clone())).await;
            let after = f.state().await;
            let rules = after.rules.as_ref().unwrap();
            let roll = rules.rolls.last().unwrap();
            assert_eq!(roll.request, pending.request);
            assert_eq!(roll.issued_by, pending.issued_by);
            assert_eq!(roll.purpose, pending.purpose);
            assert_eq!(roll.accepted_by.id, command.command_id);
            assert_eq!(
                roll.original_result.as_ref().unwrap().dice,
                vec![DieResult {
                    sides: 20,
                    value: original
                }]
            );
            assert_eq!(
                roll.result.dice,
                vec![DieResult {
                    sides: 20,
                    value: replacement
                }]
            );
            assert_eq!(roll.resolved.total, i32::from(replacement) + modifier);
            assert_eq!(
                rules
                    .tactical_grapples
                    .as_ref()
                    .and_then(|graph| graph.grip(grip))
                    .is_none(),
                success
            );
            assert!(!rules.entities[&f.actors[0]].heroic_inspiration);
            assert!(rules.timing.as_ref().unwrap().action_spent);
            assert!(!rules.cancelled_roll_ids.contains(&pending.request.id));
            f.close().await;
        }
    }
}

#[tokio::test]
async fn pending_award_and_wrong_raw_foreign_or_invalid_inspiration_submissions_preserve_every_row()
{
    let mut f = Box::pin(Fixture::new()).await;
    enable(&mut f).await;
    grant(&mut f).await;
    target_pc(&mut f, "Strength").await;
    let state = f.state().await;
    let raw = state
        .rules
        .as_ref()
        .unwrap()
        .pending
        .as_ref()
        .unwrap()
        .request
        .id;
    let pending_award = request(
        &f,
        TableTransportChannel::Host,
        award(f.characters[1], REASON),
    )
    .await;
    Box::pin(f.reject(pending_award)).await;
    let valid = inspired(&f, 1, 20).await;
    for mutation in [
        "host", "other", "raw", "unknown", "digital", "index", "sides", "value", "original",
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
            _ => {
                let TableTransportInput::Action(action) = &mut bad.input else {
                    unreachable!()
                };
                let TableAction::Tactical {
                    action:
                        TacticalAction::SubmitRollWithInspiration {
                            result,
                            die_index,
                            replacement,
                        },
                } = action.as_mut()
                else {
                    unreachable!()
                };
                match mutation {
                    "raw" => result.request_id = raw,
                    "unknown" => result.request_id = RollRequestId::new(),
                    "digital" => result.source = RollSource::Digital,
                    "index" => *die_index = 1,
                    "sides" => replacement.sides = 12,
                    "value" => replacement.value = 21,
                    "original" => result.dice[0].value = 0,
                    _ => unreachable!(),
                }
            }
        }
        Box::pin(f.reject(bad)).await;
    }
    let owner_view = f.view(f.pc(0)).await;
    let observer_view = f.view(f.pc(1)).await;
    let host_view = f.view(TableTransportChannel::Host).await;
    assert!(observer_view.roll.is_none());
    let rows = all_rows(&f.pool).await;
    for (channel, revision, roll_id) in [
        (
            f.pc(1),
            observer_view.revision,
            owner_view.roll.as_ref().unwrap().id,
        ),
        (
            TableTransportChannel::Host,
            host_view.revision,
            owner_view.roll.as_ref().unwrap().id,
        ),
        (
            f.pc(0),
            owner_view.revision,
            host_view.roll.as_ref().unwrap().id,
        ),
        (f.pc(0), owner_view.revision, raw),
        (
            TableTransportChannel::SourceCreature {
                player_id: f.players[0],
                actor: f.goblin,
            },
            owner_view.revision,
            owner_view.roll.as_ref().unwrap().id,
        ),
    ] {
        assert!(
            f.runtime
                .table_roll_options(TableRollOptionsRequest {
                    campaign_id: f.campaign,
                    channel,
                    revision,
                    roll_id
                })
                .await
                .is_err()
        );
    }
    // The Host's own capability may read the PC resource, but never submit for that PC.
    assert_eq!(
        options(&f, TableTransportChannel::Host)
            .await
            .heroic_inspiration,
        Some(true)
    );
    assert_eq!(all_rows(&f.pool).await, rows);
    assert_eq!(f.state().await, state);
    Box::pin(f.cold(valid)).await;
    f.close().await;
}

#[tokio::test]
async fn consumed_inspiration_cannot_be_spent_again_but_old_award_and_reroll_retry_survive_session_change()
 {
    let mut f = Box::pin(Fixture::new()).await;
    enable(&mut f).await;
    let award_request = grant(&mut f).await;
    target_pc(&mut f, "Strength").await;
    let reroll = inspired(&f, 20, 1).await;
    Box::pin(f.cold(reroll.clone())).await;
    let award_response = Box::pin(f.runtime.submit_presented_table(award_request.clone()))
        .await
        .unwrap();
    let reroll_response = Box::pin(f.runtime.submit_presented_table(reroll.clone()))
        .await
        .unwrap();
    finish_goblin(&mut f).await;
    escape(&mut f, "Athletics").await;
    assert_eq!(options(&f, f.pc(0)).await.heroic_inspiration, Some(false));
    let second = inspired(&f, 1, 20).await;
    Box::pin(f.reject(second)).await;
    for (index, value) in [(1, 20), (0, 21), (0, 0)] {
        let mut bad = inspired(&f, 1, 20).await;
        let TableTransportInput::Action(action) = &mut bad.input else {
            unreachable!()
        };
        let TableAction::Tactical {
            action:
                TacticalAction::SubmitRollWithInspiration {
                    die_index,
                    replacement,
                    ..
                },
        } = action.as_mut()
        else {
            unreachable!()
        };
        *die_index = index;
        replacement.value = value;
        Box::pin(f.reject(bad)).await;
    }
    let shown = f.view(f.pc(0)).await.roll.unwrap();
    let pc = f.pc(0);
    Box::pin(act(
        &mut f,
        pc,
        TacticalAction::SubmitRoll {
            result: RollResult {
                request_id: shown.id,
                source: RollSource::Physical,
                dice: vec![DieResult {
                    sides: 20,
                    value: 20,
                }],
            },
        },
    ))
    .await;
    Box::pin(act(
        &mut f,
        TableTransportChannel::Host,
        TacticalAction::ConcludeHostilities {
            cadence: AftermathCadence::ContinueExistingOrder,
            ruling: "Both sides stop fighting while retaining their current order.".into(),
        },
    ))
    .await;
    let end = request(
        &f,
        TableTransportChannel::Host,
        TableTransportInput::Action(Box::new(TableAction::EndSession)),
    )
    .await;
    Box::pin(f.cold(end)).await;
    let closed_award = request(
        &f,
        TableTransportChannel::Host,
        award(f.characters[1], REASON),
    )
    .await;
    Box::pin(f.reject(closed_award)).await;
    let new_session = PlaySessionId::new();
    let mut start = request(
        &f,
        TableTransportChannel::Host,
        TableTransportInput::Action(Box::new(TableAction::StartSession {
            id: new_session,
            name: "Next session".into(),
            participants: (0..2)
                .map(|i| SessionParticipant {
                    player_id: f.players[i],
                    character_id: Some(f.characters[i]),
                    attendance: AttendanceStatus::Present,
                })
                .collect(),
        })),
    )
    .await;
    start.session_id = Some(new_session);
    Box::pin(f.cold(start)).await;
    let rows = all_rows(&f.pool).await;
    assert_eq!(
        Box::pin(f.runtime.submit_presented_table(award_request))
            .await
            .unwrap(),
        award_response
    );
    assert_eq!(
        Box::pin(f.runtime.submit_presented_table(reroll))
            .await
            .unwrap(),
        reroll_response
    );
    assert_eq!(all_rows(&f.pool).await, rows);
    assert!(!f.state().await.rules.unwrap().entities[&f.actors[0]].heroic_inspiration);
    f.close().await;
}

#[tokio::test]
async fn hostile_award_resource_ruling_outer_authority_audit_and_original_anchor_fail_full_destination_restore()
 {
    let mut f = Box::pin(Fixture::new()).await;
    enable(&mut f).await;
    let award = grant(&mut f).await;
    let state = f.state().await;
    for mutation in ["resource", "ruling", "ruling-origin", "other-recipient"] {
        let mut forged = state.clone();
        let rules = forged.rules.as_mut().unwrap();
        match mutation {
            "resource" => {
                rules
                    .entities
                    .get_mut(&f.actors[0])
                    .unwrap()
                    .heroic_inspiration = false
            }
            "ruling" => rules.rulings.last_mut().unwrap().ruling.reason = "Invented reason".into(),
            "ruling-origin" => rules.rulings.last_mut().unwrap().command.id = CommandId::new(),
            "other-recipient" => {
                rules
                    .entities
                    .get_mut(&f.actors[0])
                    .unwrap()
                    .heroic_inspiration = false;
                rules
                    .entities
                    .get_mut(&f.actors[1])
                    .unwrap()
                    .heroic_inspiration = true;
            }
            _ => unreachable!(),
        }
        Box::pin(reject_state_image(&f, forged)).await;
    }
    let export = export_campaign(&f.pool, f.campaign).await.unwrap();
    for mutation in ["reason", "recipient", "issuer", "nested", "audit", "anchor"] {
        let mut forged = export.clone();
        match mutation {
            "audit" => {
                forged
                    .command_audit
                    .iter_mut()
                    .find(|row| row.id == award.command_id.0.to_string())
                    .unwrap()
                    .command_schema_version = 4
            }
            "anchor" => {
                let anchor = forged
                    .snapshots
                    .iter_mut()
                    .min_by_key(|row| row.event_sequence)
                    .unwrap();
                let mut anchor_state = CampaignState::decode_json(&anchor.state_json).unwrap();
                anchor_state.rules = state.rules.clone();
                anchor.state_json = anchor_state.encode_json().unwrap();
            }
            _ => {
                let row = forged
                    .event_journal
                    .iter_mut()
                    .find(|row| row.command_id == award.command_id.0.to_string())
                    .unwrap();
                let mut event: TableEvent = serde_json::from_str(&row.payload_json).unwrap();
                match mutation {
                    "reason" => {
                        event.action = TableAction::AwardHeroicInspiration {
                            character_id: f.characters[0],
                            reason: "Invented reason".into(),
                        }
                    }
                    "recipient" => {
                        event.action = TableAction::AwardHeroicInspiration {
                            character_id: f.characters[1],
                            reason: REASON.into(),
                        }
                    }
                    "issuer" => event.meta.issuer = CommandIssuer::System,
                    "nested" => event.outcome.mechanics = Some(dmd_rules::RulesOutcome::Changed),
                    _ => unreachable!(),
                }
                row.payload_json = serde_json::to_string(&event).unwrap();
            }
        }
        Box::pin(hostile_destination(&forged)).await;
    }
    f.close().await;
}

#[tokio::test]
async fn hostile_original_or_accepted_inspiration_faces_and_owner_cannot_replace_replayed_evidence()
{
    let mut f = Box::pin(Fixture::new()).await;
    enable(&mut f).await;
    grant(&mut f).await;
    target_pc(&mut f, "Dexterity").await;
    let command = inspired(&f, 1, 20).await;
    Box::pin(f.cold(command)).await;
    let state = f.state().await;
    for mutation in ["original", "accepted", "owner", "resource"] {
        let mut forged = state.clone();
        let rules = forged.rules.as_mut().unwrap();
        let roll = rules.rolls.last_mut().unwrap();
        match mutation {
            "original" => roll.original_result.as_mut().unwrap().dice[0].value = 2,
            "accepted" => roll.result.dice[0].value = 19,
            "owner" => roll.accepted_by.issuer = CommandIssuer::Admin,
            "resource" => {
                rules
                    .entities
                    .get_mut(&f.actors[0])
                    .unwrap()
                    .heroic_inspiration = true
            }
            _ => unreachable!(),
        }
        Box::pin(reject_state_image(&f, forged)).await;
    }
    f.close().await;
}
