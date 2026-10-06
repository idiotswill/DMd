//! Genuine producers for the five independently reviewed public-path defects.
//! State mutations below occur only in explicitly hostile import negatives.
use super::*;

pub(super) fn resolution_mut(state: &mut CampaignState) -> &mut TacticalResolution {
    state.encounter.as_mut().unwrap().flow.as_mut().unwrap().resolution.as_mut().unwrap()
}

fn path(x: i32, y: i32) -> Vec<TacticalMoveStep> {
    vec![TacticalMoveStep { destination: SpatialPoint { x, y, z: 0 }, mode: MovementMode::Walk }]
}

async fn exact_retry(f: &mut Fixture, request: TableTransportRequest, accepted: TableTransportResult) {
    Box::pin(f.cold(request.clone())).await;
    let before = all_rows(&f.pool).await;
    assert_eq!(Box::pin(f.runtime.submit_presented_table(request)).await.unwrap(), accepted);
    assert_eq!(all_rows(&f.pool).await, before);
}

#[tokio::test]
async fn live_grip_blocks_only_finish_and_preserves_pause_owner_resume_release_and_replacement() {
    let mut f = Box::pin(Fixture::new()).await;
    Box::pin(f.activate()).await;
    let grip = Box::pin(f.establish_pc_grip()).await;
    let pc = f.pc(0);
    // Actual Start of the Goblin's turn retires the PC's attack/equipment window.
    Box::pin(f.cold_action(pc.clone(), TacticalAction::EndTurn)).await;
    let conclusion = Box::pin(f.cold_action(TableTransportChannel::Host,
        TacticalAction::ConcludeHostilities {
            cadence: AftermathCadence::ContinueExistingOrder,
            ruling: "Both combatants stop fighting while the hold is maintained.".into(),
        })).await;
    let held = f.state().await;
    let flow = held.encounter.as_ref().unwrap().flow.as_ref().unwrap();
    assert!(flow.resolution.is_none() && flow.ready.is_empty() && flow.dodges.is_empty());
    assert!(flow.budget.attack_window.is_none());
    assert_eq!(flow.budget.attacks_remaining, 0);
    assert!(held.rules.as_ref().unwrap().timing.as_ref().unwrap().reactions_spent.is_empty());
    let error = dmd_rules::tactical::encounter_release_preflight(&held).unwrap_err().to_string();
    assert!(error.contains("live grips"), "Finish must reach the live-grip boundary: {error}");
    let host = f.view(TableTransportChannel::Host).await.tactical.unwrap();
    assert!(!host.release.unwrap().may_finish);
    assert!(host.aftermath.unwrap().may_pause_session);
    let finish = f.request(TableTransportChannel::Host, action(TacticalAction::FinishEncounter)).await;
    Box::pin(f.reject(finish)).await;
    let pause = f.request(TableTransportChannel::Host,
        TableTransportInput::Action(Box::new(TableAction::EndSession))).await;
    Box::pin(f.cold(pause)).await;
    let paused = f.state().await;
    assert!(paused.table.as_ref().unwrap().active_session.is_none());
    assert_eq!(paused.rules.as_ref().unwrap().timing, held.rules.as_ref().unwrap().timing);
    assert_eq!(paused.rules.as_ref().unwrap().tactical_grapples, held.rules.as_ref().unwrap().tactical_grapples);
    assert!(paused.rules.as_ref().unwrap().tactical_grapples.as_ref().unwrap().grip(grip).is_some());
    f.session = PlaySessionId::new();
    let attendees = (0..2).map(|i| SessionParticipant {
        player_id: f.players[i], character_id: Some(f.characters[i]), attendance: AttendanceStatus::Present,
    }).collect::<Vec<_>>();
    let missing = f.request(TableTransportChannel::Host,
        TableTransportInput::Action(Box::new(TableAction::StartSession {
            id: f.session, name: "Held cadence continues".into(), participants: attendees[1..].to_vec(),
        }))).await;
    Box::pin(f.reject(missing)).await;
    let resume = f.request(TableTransportChannel::Host,
        TableTransportInput::Action(Box::new(TableAction::StartSession {
            id: f.session, name: "Held cadence continues".into(), participants: attendees,
        }))).await;
    Box::pin(f.cold(resume)).await;
    let release = f.choose(pc, "Release Small armored figure from left hand").await;
    Box::pin(f.cold(release)).await;
    let released = f.state().await;
    assert!(released.rules.as_ref().unwrap().tactical_grapples.is_none());
    dmd_rules::tactical::encounter_release_preflight(&released).unwrap();
    assert!(f.view(TableTransportChannel::Host).await.tactical.unwrap().release.unwrap().may_finish);
    let finished_request = Box::pin(f.cold_action(TableTransportChannel::Host, TacticalAction::FinishEncounter)).await;
    let finished = f.state().await;
    assert_eq!(finished.encounter.as_ref().unwrap().flow.as_ref().unwrap().phase, TacticalPhase::Finished);
    assert!(finished.rules.as_ref().unwrap().timing.is_none());
    let completion = finished.encounter_history.as_ref().unwrap().last().unwrap().clone();
    assert_eq!(completion.conclusion_origin.id, conclusion.command_id);
    assert_eq!(completion.released_by.id, finished_request.command_id);
    for state in [&paused, &released, &finished] {
        assert_eq!(state.rules.as_ref().unwrap().rolls, held.rules.as_ref().unwrap().rolls);
        assert_eq!(state.rules.as_ref().unwrap().cancelled_roll_ids, held.rules.as_ref().unwrap().cancelled_roll_ids);
        assert_eq!(state.encounter.as_ref().unwrap().flow.as_ref().unwrap().save_decisions, flow.save_decisions);
    }
    let original = export_campaign(&f.pool, f.campaign).await.unwrap();
    let mut setup = original.event_journal.iter().find_map(|row| {
        let event: TableEvent = serde_json::from_str(&row.payload_json).ok()?;
        match event.action { TableAction::PrepareBattlefield { setup } => Some(setup), _ => None }
    }).unwrap();
    setup.encounter_id = EncounterId::new();
    setup.scene_id = SceneId::new();
    setup.location_id = LocationId::new();
    setup.name = "Next genuine courtyard encounter".into();
    let replacement = f.request(TableTransportChannel::Host,
        TableTransportInput::Action(Box::new(TableAction::PrepareBattlefield { setup }))).await;
    Box::pin(f.cold(replacement)).await;
    let established = f.state().await;
    assert!(established.encounter.as_ref().unwrap().flow.is_none());
    assert_eq!(established.encounter_history.as_ref().unwrap().last(), Some(&completion));
    assert_eq!(established.rules.as_ref().unwrap().rolls, held.rules.as_ref().unwrap().rolls);
    assert_eq!(established.rules.as_ref().unwrap().cancelled_roll_ids, held.rules.as_ref().unwrap().cancelled_roll_ids);
    Box::pin(f.cold_action(TableTransportChannel::Host, TacticalAction::Begin {
        execution: TacticalExecutionVersion::EncounterReleaseV1,
        combatants: vec![
            TacticalCombatant { actor: f.actors[0], source: TacticalSource::Character, surprised: false },
            TacticalCombatant { actor: f.goblin, source: TacticalSource::Creature { definition_id: "goblin-warrior".into() }, surprised: false },
        ],
        groups: vec![
            InitiativeGroup { actors: vec![f.actors[0]], request_id: RollRequestId::new() },
            InitiativeGroup { actors: vec![f.goblin], request_id: RollRequestId::new() },
        ],
    })).await;
    let pc = f.pc(0);
    Box::pin(f.cold_roll(pc, 18)).await;
    Box::pin(f.cold_roll(TableTransportChannel::Host, 2)).await;
    let next = f.state().await;
    assert!(next.rules.as_ref().unwrap().timing.as_ref().unwrap().turn_number > completion.final_turn.number);
    assert_eq!(next.encounter_history.as_ref().unwrap().last(), Some(&completion));
    assert!(next.rules.as_ref().unwrap().rolls.starts_with(&held.rules.as_ref().unwrap().rolls));
    f.close().await;
}

#[tokio::test]
async fn voluntary_player_target_history_survives_transfer_but_new_input_requires_the_current_owner() {
    let mut f = Box::pin(Fixture::with_opponent("goblin-warrior", CreatureSize::Small, true)).await;
    Box::pin(f.activate()).await;
    let target = f.opponent.unwrap();
    let player = f.players[1];
    let source = TableTransportChannel::SourceCreature { player_id: player, actor: target };
    let assign = f.request(TableTransportChannel::Host,
        TableTransportInput::Action(Box::new(TableAction::SetSourceCreatureController {
            actor: target, controller: CreatureController::Player(player),
        }))).await;
    Box::pin(f.cold(assign)).await;
    let pc = f.pc(0);
    Box::pin(f.cold_action(pc, TacticalAction::EndTurn)).await;
    let offer = f.view(TableTransportChannel::Host).await.grapple.unwrap().choices.into_iter()
        .find(|offer| offer.actor == f.goblin && offer.label == "Grapple Other guard with right hand").unwrap();
    let start = f.request(TableTransportChannel::Host, TableTransportInput::GrappleChoice { handle: offer.key }).await;
    Box::pin(f.cold(start)).await;
    let save = f.choose(source.clone(), "Resist Grapple with Strength").await;
    Box::pin(f.cold(save)).await;
    let voluntary = Box::pin(f.cold_action(source.clone(), TacticalAction::VoluntarilyFailSave)).await;
    let accepted = Box::pin(f.runtime.submit_presented_table(voluntary.clone())).await.unwrap();
    let finish = f.choose(TableTransportChannel::Host, "Finish without changing equipment").await;
    Box::pin(f.cold(finish)).await;
    let before = f.state().await;
    let decision = before.encounter.as_ref().unwrap().flow.as_ref().unwrap().save_decisions.last().unwrap().clone();
    assert_eq!(decision.resolved_by.issuer, CommandIssuer::Player(player));
    assert_eq!(decision.key.subject, target);
    assert_eq!(decision.failure, TacticalSaveFailure::Voluntary);
    assert_eq!(decision.resolved_by.id, voluntary.command_id);
    assert!(before.rules.as_ref().unwrap().cancelled_roll_ids.contains(&decision.key.request_id()));
    let transfer = f.request(TableTransportChannel::Host,
        TableTransportInput::Action(Box::new(TableAction::SetSourceCreatureController {
            actor: target, controller: CreatureController::Host,
        }))).await;
    Box::pin(f.cold(transfer)).await;
    Box::pin(exact_retry(&mut f, voluntary, accepted)).await;
    let transferred = f.state().await;
    assert_eq!(transferred.encounter.as_ref().unwrap().flow.as_ref().unwrap().save_decisions.last(), Some(&decision));
    for mutation in ["owner", "key", "origin", "cancellation"] {
        let mut forged = transferred.clone();
        let saved = forged.encounter.as_mut().unwrap().flow.as_mut().unwrap().save_decisions.last_mut().unwrap();
        match mutation {
            "owner" => saved.resolved_by.issuer = CommandIssuer::Admin,
            "key" => saved.key.subject = f.goblin,
            "origin" => saved.resolved_by.id = CommandId::new(),
            "cancellation" => forged.rules.as_mut().unwrap().cancelled_roll_ids.retain(|id| *id != decision.key.request_id()),
            _ => unreachable!(),
        }
        Box::pin(reject_state_image(&f, forged)).await;
    }
    Box::pin(f.cold_action(TableTransportChannel::Host, TacticalAction::EndTurn)).await;
    let target_turn = f.state().await;
    let timing = target_turn.rules.as_ref().unwrap().timing.as_ref().unwrap();
    assert_eq!(timing.order[timing.index].actor, target);
    let old_owner = f.request(source, action(TacticalAction::EndTurn)).await;
    Box::pin(f.reject(old_owner)).await;
    Box::pin(f.cold_action(TableTransportChannel::Host, TacticalAction::EndTurn)).await;
    let next_turn = f.state().await;
    let timing = next_turn.rules.as_ref().unwrap().timing.as_ref().unwrap();
    assert_eq!(timing.order[timing.index].actor, f.actors[0]);
    f.close().await;
}

#[tokio::test]
async fn unarmed_opportunity_seals_full_holder_hands_separately_and_keeps_them_after_release() {
    let mut f = Box::pin(Fixture::with_opponent_geometry("goblin-warrior", CreatureSize::Small, true, true)).await;
    Box::pin(f.activate()).await;
    let grip = Box::pin(f.establish_pc_grip()).await;
    let pc = f.pc(0);
    let actor = f.actors[0];
    let third = f.opponent.unwrap();
    Box::pin(f.cold_action(pc.clone(), TacticalAction::EndTurn)).await;
    Box::pin(f.cold_action(TableTransportChannel::Host, TacticalAction::EndTurn)).await;
    let movement = Box::pin(f.cold_action(TableTransportChannel::Host,
        TacticalAction::Move { path: path(10, 30) })).await;
    // Authored allegiances make the PC the only enemy with this crossing trigger.
    assert_eq!(f.view(TableTransportChannel::Host).await.tactical.unwrap().opportunity.unwrap().actor, actor);
    let offer = f.view(pc.clone()).await.tactical.unwrap().opportunity.unwrap();
    assert!(offer.unarmed);
    assert_eq!((offer.actor, offer.target.actor), (actor, third));
    let selected = Box::pin(f.cold_action(pc.clone(), TacticalAction::OpportunityAttack {
        choice: TacticalMeleeChoice::UnarmedDamage { ability: Ability::Strength },
    })).await;
    let issued = f.state().await;
    let r = resolution(&issued);
    let reads = r.grapple.as_ref().unwrap();
    let attack = r.attack.as_ref().unwrap();
    let TacticalAttackAdmission::Opportunity(window) = &attack.admission else { panic!("actual OA required") };
    let window_cut = reads.cuts.iter().find(|cut| matches!(cut.key.reader, GrappleReader::OpportunityWindow { .. })).unwrap();
    assert_eq!(window_cut.grips, vec![grip]);
    assert_eq!(window_cut.key.reader, GrappleReader::OpportunityWindow {
        attack: selected.command_id, window: window.origin.id, reactor: actor, mover: third, step: window.step_index,
    });
    assert_eq!(window_cut.key.work.resolution, movement.command_id);
    assert!(window_cut.source_attack.is_none());
    for cut in reads.cuts.iter().filter(|cut| matches!(cut.key.reader,
        GrappleReader::AttackAdmission { .. } | GrappleReader::RequestIssue { .. })) {
        assert!(cut.grips.is_empty(), "unarmed attack itself has no outgoing hand component");
    }
    let pending = issued.rules.as_ref().unwrap().pending.as_ref().unwrap().clone();
    let cuts = reads.cuts.clone();
    let proofs = reads.proofs.clone();
    let equipment = issued.rules.as_ref().unwrap().tactical_inventory.clone();
    for mutation in ["window", "work", "omitted-reservation", "hand", "source"] {
        let mut forged = issued.clone();
        let reads = resolution_mut(&mut forged).grapple.as_mut().unwrap();
        let cut = reads.cuts.iter_mut().find(|cut| matches!(cut.key.reader, GrappleReader::OpportunityWindow { .. })).unwrap();
        match mutation {
            "window" => { let GrappleReader::OpportunityWindow { window, .. } = &mut cut.key.reader else { unreachable!() }; *window = CommandId::new(); }
            "work" => cut.key.work.occurrence = 0,
            "omitted-reservation" => cut.grips.clear(),
            "hand" => reads.proofs[0].declaration.hand = Hand::Right,
            "source" => reads.proofs[0].declaration.grappler = third,
            _ => unreachable!(),
        }
        Box::pin(reject_state_image(&f, forged)).await;
    }
    let release = f.choose(pc.clone(), "Release Small armored figure from left hand").await;
    Box::pin(f.cold(release)).await;
    let released = f.state().await;
    assert!(released.rules.as_ref().unwrap().tactical_grapples.is_none());
    assert_eq!(released.rules.as_ref().unwrap().pending.as_ref(), Some(&pending));
    assert_eq!(resolution(&released).attack.as_ref(), Some(attack));
    assert_eq!(resolution(&released).grapple.as_ref().unwrap().cuts, cuts);
    assert_eq!(resolution(&released).grapple.as_ref().unwrap().proofs, proofs);
    assert_eq!(released.rules.as_ref().unwrap().tactical_inventory, equipment);
    Box::pin(f.cold_roll(pc, 1)).await;
    let after = f.state().await;
    assert_eq!(after.encounter.as_ref().unwrap().participant(third).unwrap().position, SpatialPoint { x: 10, y: 30, z: 0 });
    assert_eq!(after.rules.as_ref().unwrap().rolls.iter().filter(|raw| raw.request.id == pending.request.id).count(), 1);
    assert_eq!(after.rules.as_ref().unwrap().timing.as_ref().unwrap().reactions_spent.iter().filter(|spent| **spent == actor).count(), 1);
    assert_eq!(after.rules.as_ref().unwrap().tactical_inventory, equipment);
    f.close().await;
}

#[tokio::test]
async fn explicit_self_only_in_range_then_suspended_last_release_keeps_the_original_route_and_retry() {
    let mut f = Box::pin(Fixture::new()).await;
    let pc = f.pc(0);
    let no_grip = f.request(pc.clone(), action(TacticalAction::MoveSelfOnly { path: path(10, 20) })).await;
    Box::pin(f.reject(no_grip)).await;
    let old_options = f.view(pc.clone()).await.tactical.unwrap().movement_options.unwrap();
    assert!(!old_options.self_only_required);
    assert!(serde_json::to_value(&old_options).unwrap().get("self_only_required").is_none());
    Box::pin(f.activate()).await;
    let no_grip = f.request(pc.clone(), action(TacticalAction::MoveSelfOnly { path: path(10, 20) })).await;
    Box::pin(f.reject(no_grip)).await;
    let grip = Box::pin(f.establish_pc_grip()).await;
    let before = f.state().await;
    let target_position = before.encounter.as_ref().unwrap().participant(f.goblin).unwrap().position;
    assert!(f.view(pc.clone()).await.tactical.unwrap().movement_options.unwrap().self_only_required);
    let in_range = path(10, 20);
    let ordinary = f.request(pc.clone(), action(TacticalAction::Move { path: in_range.clone() })).await;
    Box::pin(f.reject(ordinary.clone())).await;
    let explicit = f.request(pc.clone(), action(TacticalAction::MoveSelfOnly { path: in_range })).await;
    for version in [1, 2] {
        let mut old = explicit.clone(); old.command_id = CommandId::new(); old.version = version;
        Box::pin(f.reject(old)).await;
    }
    let mut foreign = explicit.clone(); foreign.command_id = CommandId::new();
    foreign.channel = f.pc(1); foreign.revision = f.view(f.pc(1)).await.revision;
    Box::pin(f.reject(foreign)).await;
    let pack = dmd_rules::RulesPack::from_json(include_str!("../../../../content/srd-5.2.1/kernel.json")).unwrap();
    let raw_meta = CommandMeta { id: CommandId::new(), campaign_id: f.campaign,
        session_id: Some(f.session), issuer: CommandIssuer::Player(f.players[0]),
        actor: Some(AgentRef::Entity(f.actors[0])), expected_event_sequence: before.applied_event_sequence };
    assert!(dmd_rules::tactical::resolve_tactical(&before, &raw_meta,
        &TacticalAction::MoveSelfOnly { path: path(10, 20) }, &pack).is_err());
    Box::pin(f.cold(explicit.clone())).await;
    let accepted = Box::pin(f.runtime.submit_presented_table(explicit.clone())).await.unwrap();
    let stayed = f.state().await;
    assert!(stayed.rules.as_ref().unwrap().tactical_grapples.as_ref().unwrap().grip(grip).is_some());
    assert_eq!(stayed.encounter.as_ref().unwrap().participant(f.goblin).unwrap().position, target_position);
    assert_eq!(stayed.encounter.as_ref().unwrap().participant(f.actors[0]).unwrap().position, SpatialPoint { x: 10, y: 20, z: 0 });
    assert_eq!(stayed.encounter.as_ref().unwrap().flow.as_ref().unwrap().budget.movement_spent, 10);
    // An earlier ordinary Move is never silently translated into self-only input.
    Box::pin(f.reject(ordinary)).await;
    let mut stale = explicit.clone(); stale.command_id = CommandId::new();
    Box::pin(f.reject(stale)).await;
    Box::pin(f.cold_action(pc.clone(), TacticalAction::MoveSelfOnly { path: path(0, 20) })).await;
    let suspended = f.state().await;
    let movement = resolution(&suspended).movement.as_ref().unwrap().clone();
    assert_eq!(movement.grapple_self_only.as_ref().unwrap().grips, vec![grip]);
    assert_eq!(movement.initial_spent, 10);
    assert_eq!(movement.next_step, 0);
    assert!(movement.traversed.is_empty());
    assert_eq!(f.view(TableTransportChannel::Host).await.tactical.unwrap().opportunity.unwrap().actor, f.goblin);
    let release = f.choose(pc, "Release Small armored figure from left hand").await;
    Box::pin(f.cold(release)).await;
    let released = f.state().await;
    assert!(released.rules.as_ref().unwrap().tactical_grapples.is_none());
    let retained = resolution(&released).movement.as_ref().unwrap();
    assert_eq!(retained.origin, movement.origin);
    assert_eq!(retained.path, movement.path);
    assert_eq!(retained.initial_spent, movement.initial_spent);
    assert_eq!(retained.next_step, movement.next_step);
    assert_eq!(retained.traversed, movement.traversed);
    assert_eq!(retained.grapple_self_only, movement.grapple_self_only);
    Box::pin(f.cold_action(TableTransportChannel::Host, TacticalAction::DeclineOpportunity)).await;
    let after = f.state().await;
    assert_eq!(after.encounter.as_ref().unwrap().participant(f.actors[0]).unwrap().position, SpatialPoint { x: 0, y: 20, z: 0 });
    assert_eq!(after.encounter.as_ref().unwrap().participant(f.goblin).unwrap().position, target_position);
    assert_eq!(after.encounter.as_ref().unwrap().flow.as_ref().unwrap().budget.movement_spent, 20);
    Box::pin(exact_retry(&mut f, explicit, accepted)).await;
    f.close().await;
}
