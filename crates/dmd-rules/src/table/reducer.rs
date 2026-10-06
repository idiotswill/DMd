//! The single pure table-state mutation reducer.
use super::TableSessionChange as SessionChange;
use super::*;
use crate::{RulesAction, RulesEvent, RulesOutcome, RulesPack};

fn tactical_transition(
    state: &CampaignState,
    next: &mut CampaignState,
    meta: &CommandMeta,
    action: &crate::tactical::TacticalAction,
    pack: &RulesPack,
    historical: Option<ExpectedNested<'_>>,
    execution: &mut crate::tactical::grapple::execution::ExecutionContext<'_>,
) -> Result<crate::tactical::TacticalEvent, String> {
    if let Some(expected) = historical {
        let child = expected
            .tactical_event
            .ok_or("Historical encounter action lacks its exact source event.")?;
        if child.meta != *meta || child.action != *action {
            return Err("Historical encounter event differs from its table envelope.".into());
        }
    }
    let event = crate::tactical::apply_table_with_context(
        state,
        next,
        meta,
        action,
        pack,
        historical.is_some(),
        execution,
    )
    .map_err(|error| error.to_string())?;
    if historical
        .and_then(|expected| expected.tactical_event)
        .is_some_and(|expected| expected != &event)
    {
        return Err("Table source event disagrees with deterministic replay.".into());
    }
    Ok(event)
}

pub fn resolve_ordinary(
    state: &CampaignState,
    meta: &CommandMeta,
    action: &TableOperation,
    pack: &RulesPack,
    historical: Option<ExpectedNested<'_>>,
) -> Result<TableOperationTransition, String> {
    validate_table(state, pack)?;
    let mut next = state.clone();
    let produced = apply_operation(
        state,
        &mut next,
        meta,
        action,
        pack,
        historical,
        &mut crate::tactical::grapple::execution::ExecutionContext::ordinary(),
    )?;
    Ok(TableOperationTransition {
        state: next,
        produced,
    })
}

pub(crate) fn apply_operation(
    state: &CampaignState,
    next: &mut CampaignState,
    meta: &CommandMeta,
    action: &TableOperation,
    pack: &RulesPack,
    historical: Option<ExpectedNested<'_>>,
    execution: &mut crate::tactical::grapple::execution::ExecutionContext<'_>,
) -> Result<TableProduced, String> {
    if meta.campaign_id != state.campaign_id()
        || meta.expected_event_sequence != state.applied_event_sequence
    {
        return Err("The table changed. Refresh and review your action before retrying.".into());
    }
    validate_table_with_read(&execution.read(next).map_err(|e| e.to_string())?, pack)?;
    let mut session_change = None;
    let mut rules_event = None;
    let mut tactical_event = None;
    let mut mechanics = None;
    let message = match action {
        TableOperation::AwardHeroicInspiration {
            character_id,
            reason,
        } => {
            if !execution.is_owned() {
                return Err("An Inspiration award requires original table history.".into());
            }
            super::inspiration::award(state, next, meta, *character_id, reason)?;
            "Heroic Inspiration awarded.".into()
        }
        TableOperation::EnableGrappleTransport => {
            if !execution.is_owned() {
                return Err("Ground drag activation requires original table history.".into());
            }
            table_mut(next)?.grapple_access = Some(Box::new(
                super::grapple_access::activate_transport(state, meta)?,
            ));
            "Dragging and Inspiration controls are enabled for this table.".into()
        }
        TableOperation::EnableGrappleAccess => {
            if !execution.is_owned() {
                return Err("Grapple activation requires original table history.".into());
            }
            let access = super::grapple_access::activate(state, meta)?;
            table_mut(next)?.grapple_access = Some(Box::new(access));
            "Grapple is enabled for this table.".into()
        }
        TableOperation::UpdateContract { contract } => {
            host(meta)?;
            contract.validate()?;
            let current = table(next)?;
            if current.active_session.is_some()
                || current.pending.is_some()
                || current.roll_context.is_some()
            {
                return Err("End the session before changing the table contract.".into());
            }
            if next
                .rules
                .as_ref()
                .is_some_and(|rules| rules.house_rules != contract.house_rules)
            {
                return Err(
                    "Mechanical house rules cannot change after character creation in this gate."
                        .into(),
                );
            }
            table_mut(next)?.contract = contract.clone();
            "The table contract was updated.".into()
        }
        TableOperation::AddPlayer { id, name } => {
            host(meta)?;
            setup_only(next)?;
            bounded_text(name, 200)?;
            if next.players.contains_key(id) {
                return Err("That player identity already exists.".into());
            }
            next.players.insert(
                *id,
                Player {
                    id: *id,
                    campaign_id: meta.campaign_id,
                    display_name: name.trim().into(),
                },
            );
            format!("{} joined the campaign.", name.trim())
        }
        TableOperation::CreateCharacter {
            character_id,
            entity_id,
            player_id,
            input,
        } => {
            host(meta)?;
            setup_only(next)?;
            if !next.players.contains_key(player_id)
                || next.characters.contains_key(character_id)
                || next.entities.contains_key(entity_id)
            {
                return Err("Select an existing player and a new character identity.".into());
            }
            let built = crate::build_character(input, *entity_id, pack)
                .map_err(|error| error.to_string())?;
            next.entities.insert(
                *entity_id,
                WorldEntity {
                    id: *entity_id,
                    campaign_id: meta.campaign_id,
                    display_name: built.profile.name.clone(),
                    kind: EntityKind::Character,
                    existence: EntityExistence::Present,
                    location_id: None,
                },
            );
            next.characters.insert(
                *character_id,
                Character {
                    id: *character_id,
                    entity_id: *entity_id,
                    campaign_id: meta.campaign_id,
                    controlling_player_id: Some(*player_id),
                    display_name: built.profile.name.clone(),
                    status: CharacterStatus::Active,
                },
            );
            apply_rules(
                next,
                meta,
                RulesAction::CreateCharacter {
                    entity_id: *entity_id,
                    input: input.clone(),
                },
                pack,
                &mut rules_event,
                &mut mechanics,
                execution,
            )?;
            table_mut(next)?
                .character_profiles
                .insert(*character_id, built.profile.clone());
            format!(
                "{} was created as a level 1 Human Fighter with the Soldier background.",
                built.profile.name
            )
        }
        TableOperation::PrepareEquipment {
            character_id,
            item_ids,
        } => {
            host(meta)?;
            if table(state)?.active_session.is_some() {
                active(state, meta)?;
            } else if meta.session_id.is_some() {
                return Err("Equipment setup does not belong to an active session.".into());
            }
            if table(state)?.pending.is_some()
                || table(state)?.roll_context.is_some()
                || state
                    .rules
                    .as_ref()
                    .is_some_and(|rules| rules.pending.is_some())
                || state.encounter.as_ref().is_some_and(|encounter| {
                    encounter.flow.is_some()
                        && crate::tactical::require_finished_encounter(state).is_err()
                })
            {
                return Err(
                    "Finish pending decisions and prepare equipment before initiative.".into(),
                );
            }
            *next = crate::table::equipment::prepare(state, meta, *character_id, item_ids, pack)?;
            "Starting equipment is ready for play.".into()
        }
        TableOperation::CreateCreature { creation } => {
            host(meta)?;
            if historical.is_none() {
                let pin = creation
                    .source
                    .as_ref()
                    .ok_or("Choose an exact current creature source before creating a creature.")?;
                let admitted = crate::tactical_creatures::current_creature_sources()
                    .map_err(|e| e.to_string())?
                    .into_iter()
                    .any(|source| {
                        source.id == creation.definition_id
                            && crate::tactical_creatures::creature_source_pin(source).as_ref()
                                == Ok(pin)
                    });
                if !admitted {
                    return Err("Creature source is not an exact current admission.".into());
                }
            }
            if table(state)?.active_session.is_some() {
                active(state, meta)?;
            } else if meta.session_id.is_some() {
                return Err("Creature setup does not belong to an active session.".into());
            }
            idle(state)?;
            if state.encounter.is_some()
                && crate::tactical::require_finished_encounter(state).is_err()
            {
                return Err("Prepare source creatures before setting up the battlefield.".into());
            }
            *next = crate::table::creatures::create(state, meta, creation, pack)?;
            // Private preparation must not teach players an actor's name or existence.
            "Host preparation recorded.".into()
        }
        TableOperation::EnableSourceActorAccess { adopted } => {
            *next = crate::table::source_control::activate(state, meta, adopted)?;
            "Source creature control is enabled for this table.".into()
        }
        TableOperation::SetSourceCreatureController { actor, controller } => {
            *next = crate::table::source_control::assign(state, meta, *actor, *controller)?;
            "Source control updated.".into()
        }
        TableOperation::Tactical { action } => {
            require_tactical_session(state, meta, action)?;
            crate::table::source_control::authorize_tactical(state, meta, action)?;
            match meta.issuer {
                CommandIssuer::Player(_) => {
                    if crate::table::source_control::attending_source(state, meta).is_err() {
                        player_channel_for_encounter(state, meta, true)?;
                    }
                }
                _ => host(meta)?,
            }
            if table(state)?.pending.is_some() || table(state)?.roll_context.is_some() {
                return Err("Finish the pending table decision before an encounter action.".into());
            }
            if matches!(action, crate::tactical::TacticalAction::Establish { .. }) {
                return Err(
                    "Use battlefield setup to place source-derived characters and creatures."
                        .into(),
                );
            }
            tactical_event = Some(tactical_transition(
                state, next, meta, action, pack, historical, execution,
            )?);
            // Detailed outcomes belong to the viewer-specific tactical projection.
            // This transcript is shared by the whole table, including unaware PCs.
            if matches!(
                action,
                crate::tactical::TacticalAction::ConcludeHostilities { .. }
            ) {
                "Hostilities concluded. Ongoing saves and durations continue in the existing turn order.".into()
            } else if matches!(action, crate::tactical::TacticalAction::FinishEncounter) {
                "Encounter finished. Lasting consequences and equipment remain saved.".into()
            } else {
                "Encounter action recorded.".into()
            }
        }
        TableOperation::PrepareBattlefield { setup } => {
            host(meta)?;
            active(state, meta)?;
            idle(state)?;
            let action = crate::table::battlefield::prepare(state, next, meta, setup)?;
            let staged = next.clone();
            tactical_event = Some(tactical_transition(
                &staged, next, meta, &action, pack, historical, execution,
            )?);
            "The encounter map is ready.".into()
        }
        TableOperation::StartSession {
            id,
            name,
            participants,
        } => {
            host(meta)?;
            setup_only(next)?;
            bounded_text(name, 200)?;
            if meta.session_id != Some(*id)
                || participants.is_empty()
                || !participants.iter().any(|p| {
                    p.attendance == AttendanceStatus::Present
                        && (p.character_id.is_some()
                            || crate::table::source_control::has_owned_source(state, p.player_id))
                })
            {
                return Err("Choose at least one attending player and their character.".into());
            }
            for participant in participants {
                if let Some(id) = participant.character_id {
                    let pc = next
                        .characters
                        .get(&id)
                        .ok_or("Unknown session character.")?;
                    let retained_dead = pc.status == CharacterStatus::Dead
                        && next
                            .encounter
                            .as_ref()
                            .and_then(|encounter| encounter.flow.as_ref())
                            .is_some_and(|flow| {
                                flow.aftermath.is_some()
                                    && flow.phase == TacticalPhase::Active
                                    && flow
                                        .combatants
                                        .iter()
                                        .any(|combatant| combatant.actor == pc.entity_id)
                            });
                    if (pc.status != CharacterStatus::Active && !retained_dead)
                        || pc.controlling_player_id != Some(participant.player_id)
                    {
                        return Err(
                            "A session character must be active and belong to the selected player."
                                .into(),
                        );
                    }
                    if !table(next)?.character_profiles.contains_key(&id) {
                        return Err("This character has no supported creation profile.".into());
                    }
                }
            }
            require_aftermath_attendance(next, participants)?;
            let binding = ActiveTableSession {
                session_id: *id,
                display_name: name.trim().into(),
                started_at_world: next.clock.now,
                participants: participants.clone(),
            };
            let session = binding.as_session(meta.campaign_id);
            if !session.validate_against_state(next).is_empty() {
                return Err("The session bindings are inconsistent.".into());
            }
            table_mut(next)?.active_session = Some(binding);
            session_change = Some(SessionChange::Start { session });
            format!("Session started: {}.", name.trim())
        }
        TableOperation::EndSession => {
            host(meta)?;
            idle(next)?;
            if next
                .encounter
                .as_ref()
                .is_some_and(|encounter| encounter.flow.is_some())
                && crate::tactical::require_finished_encounter(next).is_err()
            {
                crate::tactical::require_aftermath_session_boundary(next)
                    .map_err(|error| error.to_string())?;
            }
            let binding = active(next, meta)?;
            let expected = binding.as_session(meta.campaign_id);
            let mut ended = expected.clone();
            ended.status = PlaySessionStatus::Closed;
            ended.ended_at_world = Some(next.clock.now);
            table_mut(next)?.active_session = None;
            session_change = Some(SessionChange::Replace {
                expected,
                next: ended,
            });
            "Session ended. Accepted events and the recap are saved.".into()
        }
        TableOperation::SetSituation { situation } => {
            host(meta)?;
            if table(next)?.roll_context.is_some() {
                return Err("Resolve the pending roll before changing its situation.".into());
            }
            if situation
                .challenges
                .iter()
                .any(|challenge| challenge.resolution.is_some())
            {
                return Err("A new situation cannot contain invented resolutions.".into());
            }
            if table(next)?.active_session.is_some() {
                active(next, meta)?;
            }
            table_mut(next)?.situation = situation.clone();
            format!("Situation: {}. {}", situation.title, situation.description)
        }
        TableOperation::Declare { text, intent } => {
            idle(next)?;
            let (player_id, character_id, actor, session_id) = player_channel(next, meta)?;
            bounded_text(text, 8000)?;
            let intent = intent.clone();
            let response = intent_message(&intent);
            table_mut(next)?.pending = Some(PendingTableDecision {
                id: meta.id,
                session_id,
                player_id,
                character_id,
                actor,
                origin: meta.clone(),
                revision: 0,
                text: text.trim().into(),
                intent,
            });
            response
        }
        TableOperation::Correct {
            pending_id,
            revision,
            text,
            intent,
        } => {
            let pending = owned_pending(next, meta, *pending_id, *revision)?.clone();
            bounded_text(text, 8000)?;
            let intent = intent.clone();
            let response = intent_message(&intent);
            table_mut(next)?.pending = Some(PendingTableDecision {
                text: text.trim().into(),
                intent,
                origin: meta.clone(),
                revision: pending
                    .revision
                    .checked_add(1)
                    .ok_or("Correction revision exhausted.")?,
                ..pending
            });
            format!("Uncommitted declaration corrected. {response}")
        }
        TableOperation::CancelDecision {
            pending_id,
            revision,
        } => {
            owned_pending(next, meta, *pending_id, *revision)?;
            table_mut(next)?.pending = None;
            "The uncommitted declaration was withdrawn. No world outcome occurred.".into()
        }
        TableOperation::Adjudicate {
            pending_id,
            revision,
            request_id,
        } => {
            host(meta)?;
            active(next, meta)?;
            let pending = pending_match(next, *pending_id, *revision)?.clone();
            if pending.intent == TableIntent::SecondWind
                && next
                    .encounter
                    .as_ref()
                    .is_some_and(|encounter| encounter.flow.is_some())
            {
                let acting = next
                    .rules
                    .as_ref()
                    .and_then(|r| r.timing.as_ref())
                    .and_then(|t| t.order.get(t.index))
                    .map(|entry| entry.actor);
                if acting != Some(pending.actor) {
                    return Err("Second Wind needs the declaring character's turn.".into());
                }
                table_mut(next)?.pending = None;
                let before = next.clone();
                tactical_event = Some(tactical_transition(
                    &before,
                    next,
                    meta,
                    &crate::tactical::TacticalAction::SecondWind,
                    pack,
                    historical,
                    execution,
                )?);
                "Second Wind is paid. Report the requested physical d10.".into()
            } else {
                let (action, challenge_id) = match &pending.intent {
                    TableIntent::Check {
                        kind,
                        challenge_id: Some(id),
                        ..
                    } => {
                        let challenge = table(next)?
                            .situation
                            .challenges
                            .iter()
                            .find(|c| c.id == *id)
                            .ok_or("The challenge is no longer available.")?;
                        if challenge.resolution.is_some() || &challenge.kind != kind {
                            return Err(
                                "The challenge context changed. Review the declaration.".into()
                            );
                        }
                        (
                            RulesAction::RequestTest {
                                actor: pending.actor,
                                kind: kind.clone(),
                                dc: i32::from(challenge.dc),
                                visibility: RollVisibility::Public,
                                circumstances: Circumstances::default(),
                                ruling: Ruling {
                                    basis: RulingBasis::GmAdjudication,
                                    reason: format!(
                                        "Established challenge {}: {}",
                                        challenge.id, challenge.title
                                    ),
                                },
                                request_id: *request_id,
                            },
                            Some(id.clone()),
                        )
                    }
                    TableIntent::SecondWind => (
                        RulesAction::SecondWind {
                            actor: pending.actor,
                            request_id: *request_id,
                        },
                        None,
                    ),
                    _ => {
                        return Err(
                            "This decision still needs clarification and supported table context."
                                .into(),
                        );
                    }
                };
                table_mut(next)?.pending = None;
                apply_rules(
                    next,
                    meta,
                    action,
                    pack,
                    &mut rules_event,
                    &mut mechanics,
                    execution,
                )?;
                match &mechanics {
                    Some(RulesOutcome::RollRequested(_)) => {
                        table_mut(next)?.roll_context = Some(TableRollContext {
                            request_id: *request_id,
                            session_id: pending.session_id,
                            player_id: pending.player_id,
                            character_id: pending.character_id,
                            actor: pending.actor,
                            challenge_id,
                            declaration: pending.text,
                        });
                        "A roll was requested. Report the physical die faces shown in the request."
                            .into()
                    }
                    Some(RulesOutcome::AutomaticTest { .. }) => {
                        return Err(
                            "Automatic challenge outcomes need an explicit supported table ruling."
                                .into(),
                        );
                    }
                    _ => "The supported action was resolved.".into(),
                }
            }
        }
        TableOperation::SubmitPhysical { request_id, faces } => {
            let (player, character, actor, _) = player_channel(next, meta)?;
            let context = table(next)?
                .roll_context
                .clone()
                .ok_or("No table roll is pending.")?;
            if context.request_id != *request_id
                || context.player_id != player
                || context.character_id != character
                || context.actor != actor
            {
                return Err(
                    "Only the requested character's attending player can report this roll.".into(),
                );
            }
            let pending = next
                .rules
                .as_ref()
                .and_then(|rules| rules.pending.as_ref())
                .ok_or("No rules roll is pending.")?;
            let sides = pending
                .request
                .dice
                .iter()
                .flat_map(|die| {
                    std::iter::repeat_n(
                        die.sides,
                        if pending.request.mode == RollMode::Normal {
                            usize::from(die.count)
                        } else {
                            2
                        },
                    )
                })
                .collect::<Vec<_>>();
            if sides.len() != faces.len() {
                return Err("Report exactly the requested number of raw die faces.".into());
            }
            let dice = sides
                .iter()
                .zip(faces)
                .map(|(sides, value)| DieResult {
                    sides: *sides,
                    value: *value,
                })
                .collect();
            apply_rules(
                next,
                meta,
                RulesAction::SubmitRoll {
                    result: RollResult {
                        request_id: *request_id,
                        source: RollSource::Physical,
                        dice,
                    },
                },
                pack,
                &mut rules_event,
                &mut mechanics,
                execution,
            )?;
            table_mut(next)?.roll_context = None;
            match &mechanics {
                Some(RulesOutcome::RollResolved {
                    roll,
                    success,
                    amount,
                    followup,
                    ..
                }) => {
                    if followup.is_some() {
                        return Err(
                            "This table action requires a follow-up outside its supported path."
                                .into(),
                        );
                    }
                    if let Some(id) = context.challenge_id {
                        let challenge = table_mut(next)?
                            .situation
                            .challenges
                            .iter_mut()
                            .find(|c| c.id == id)
                            .ok_or("Unknown challenge.")?;
                        let success = success.ok_or("A challenge roll has no success result.")?;
                        challenge.resolution = Some(ChallengeResolution {
                            actor,
                            request_id: *request_id,
                            success,
                            total: roll.total,
                        });
                        format!(
                            "{} Total {}: {}",
                            if success { "Success." } else { "Failure." },
                            roll.total,
                            if success {
                                &challenge.success
                            } else {
                                &challenge.failure
                            }
                        )
                    } else {
                        format!(
                            "Roll total {}.{}",
                            roll.total,
                            amount.map_or(String::new(), |amount| format!(
                                " Hit points regained: {amount}."
                            ))
                        )
                    }
                }
                _ => return Err("The table expected a completed physical roll.".into()),
            }
        }
    };
    validate_table_with_read(&execution.read(next).map_err(|e| e.to_string())?, pack)?;
    Ok(TableProduced {
        message,
        mechanics,
        rules_event,
        tactical_event,
        session_change,
    })
}

pub fn validate_table(state: &CampaignState, pack: &RulesPack) -> Result<(), String> {
    validate_table_with_read(
        &crate::tactical::grapple::execution::ReadContext::ordinary(state),
        pack,
    )
}

pub(crate) fn validate_table_with_read(
    read: &crate::tactical::grapple::execution::ReadContext<'_>,
    pack: &RulesPack,
) -> Result<(), String> {
    crate::kernel::validate_state_with_read(read, pack).map_err(|error| error.to_string())?;
    let state = read.state();
    if !state.validate().is_empty() {
        return Err("Campaign state is inconsistent.".into());
    }
    if let Some(table) = &state.table {
        table.validate(state)?;
        for profile in table.character_profiles.values() {
            let entity = state
                .rules
                .as_ref()
                .and_then(|rules| rules.entities.get(&profile.entity_id))
                .ok_or("A table character has no authoritative mechanical sheet.")?;
            crate::table::equipment::validate_character_equipment(state, profile, entity, pack)?;
        }
    }
    Ok(())
}

fn apply_rules(
    state: &mut CampaignState,
    meta: &CommandMeta,
    action: RulesAction,
    pack: &RulesPack,
    event: &mut Option<RulesEvent>,
    outcome: &mut Option<RulesOutcome>,
    execution: &mut crate::tactical::grapple::execution::ExecutionContext<'_>,
) -> Result<(), String> {
    let before = state.clone();
    let produced =
        crate::kernel::apply_table_with_context(&before, state, meta, &action, pack, execution)
            .map_err(|error| error.to_string())?;
    *outcome = Some(produced.outcome.clone());
    *event = Some(produced);
    Ok(())
}
pub fn table(state: &CampaignState) -> Result<&TableState, String> {
    state
        .table
        .as_ref()
        .ok_or("This campaign has no desktop table configuration.".into())
}
fn table_mut(state: &mut CampaignState) -> Result<&mut TableState, String> {
    state
        .table
        .as_mut()
        .ok_or("This campaign has no desktop table configuration.".into())
}
fn host(meta: &CommandMeta) -> Result<(), String> {
    if !matches!(meta.issuer, CommandIssuer::Admin | CommandIssuer::System) || meta.actor.is_some()
    {
        return Err("This action requires the trusted local host channel.".into());
    }
    Ok(())
}
fn setup_only(state: &CampaignState) -> Result<(), String> {
    idle(state)?;
    if table(state)?.active_session.is_some() {
        return Err("End the current session before changing campaign membership.".into());
    }
    Ok(())
}
fn idle(state: &CampaignState) -> Result<(), String> {
    let table = table(state)?;
    if table.pending.is_some()
        || table.roll_context.is_some()
        || state.rules.as_ref().is_some_and(|r| r.pending.is_some())
    {
        return Err(
            "Resolve or correct the pending decision first. You can quit and resume it safely."
                .into(),
        );
    }
    Ok(())
}
/// An aftermath session cannot omit an existing owner and later strand their
/// mandatory turn/save. No host replacement or implicit attendance is introduced.
fn require_aftermath_attendance(
    state: &CampaignState,
    participants: &[SessionParticipant],
) -> Result<(), String> {
    let Some(flow) = state
        .encounter
        .as_ref()
        .and_then(|encounter| encounter.flow.as_ref())
        .filter(|flow| flow.aftermath.is_some() && flow.phase == TacticalPhase::Active)
    else {
        return Ok(());
    };
    for combatant in &flow.combatants {
        if let Some(character) = state
            .characters
            .values()
            .find(|character| character.entity_id == combatant.actor)
            && let Some(player) = character.controlling_player_id
            && !participants.iter().any(|participant| {
                participant.player_id == player
                    && participant.character_id == Some(character.id)
                    && participant.attendance == AttendanceStatus::Present
            })
        {
            return Err("Resume aftermath with every retained character's controller present and bound to that character, including dead characters.".into());
        }
        if let Some(CreatureController::Player(player)) = state
            .rules
            .as_ref()
            .and_then(|rules| rules.tactical_creatures.as_ref())
            .and_then(|creatures| creatures.runtime(combatant.actor))
            .map(|runtime| runtime.controller)
            && !participants.iter().any(|participant| {
                participant.player_id == player
                    && participant.attendance == AttendanceStatus::Present
            })
        {
            return Err("Resume aftermath with every retained source creature's controller explicitly present.".into());
        }
    }
    Ok(())
}

/// Only these two typed host commands may even request the closed-session route.
/// Recovery checks this shape, then semantic replay repeats the complete preflight.
pub fn closed_session_release_action(action: &crate::tactical::TacticalAction) -> bool {
    matches!(
        action,
        crate::tactical::TacticalAction::FinishEncounter
            | crate::tactical::TacticalAction::UpgradeExecutionTo {
                execution: TacticalExecutionVersion::EncounterReleaseV1,
            }
    )
}

fn require_tactical_session(
    state: &CampaignState,
    meta: &CommandMeta,
    action: &crate::tactical::TacticalAction,
) -> Result<(), String> {
    if table(state)?.active_session.is_some() {
        active(state, meta)?;
        return Ok(());
    }
    host(meta)?;
    if meta.session_id.is_some() || !closed_session_release_action(action) {
        return Err("The command does not belong to the active session.".into());
    }
    crate::tactical::encounter_release_preflight(state).map_err(|error| error.to_string())?;
    Ok(())
}

fn active<'a>(
    state: &'a CampaignState,
    meta: &CommandMeta,
) -> Result<&'a ActiveTableSession, String> {
    table(state)?
        .active_session
        .as_ref()
        .filter(|session| Some(session.session_id) == meta.session_id)
        .ok_or("The command does not belong to the active session.".into())
}
pub fn player_channel(
    state: &CampaignState,
    meta: &CommandMeta,
) -> Result<(PlayerId, CharacterId, EntityId, PlaySessionId), String> {
    player_channel_for_encounter(state, meta, false)
}

fn player_channel_for_encounter(
    state: &CampaignState,
    meta: &CommandMeta,
    allow_dead_participant: bool,
) -> Result<(PlayerId, CharacterId, EntityId, PlaySessionId), String> {
    let CommandIssuer::Player(player) = meta.issuer else {
        return Err("Select an attending player for this declaration.".into());
    };
    let Some(AgentRef::Entity(actor)) = meta.actor else {
        return Err("Select that player's character.".into());
    };
    let session = active(state, meta)?;
    let character = state
        .characters
        .values()
        .find(|pc| {
            pc.entity_id == actor
                && pc.controlling_player_id == Some(player)
                && (pc.status == CharacterStatus::Active
                    || allow_dead_participant
                        && pc.status == CharacterStatus::Dead
                        && state
                            .encounter
                            .as_ref()
                            .and_then(|encounter| encounter.flow.as_ref())
                            .is_some_and(|flow| {
                                flow.combatants
                                    .iter()
                                    .any(|combatant| combatant.actor == actor)
                            }))
        })
        .ok_or("The selected player does not control that active character.")?;
    table(state)?.validate_attendance(session.session_id, player, character.id)?;
    Ok((player, character.id, actor, session.session_id))
}
fn pending_match(
    state: &CampaignState,
    id: CommandId,
    revision: u32,
) -> Result<&PendingTableDecision, String> {
    table(state)?
        .pending
        .as_ref()
        .filter(|pending| pending.id == id && pending.revision == revision)
        .ok_or("That declaration changed or was already resolved.".into())
}
fn owned_pending<'a>(
    state: &'a CampaignState,
    meta: &CommandMeta,
    id: CommandId,
    revision: u32,
) -> Result<&'a PendingTableDecision, String> {
    let (player, character, actor, _) = player_channel(state, meta)?;
    let pending = pending_match(state, id, revision)?;
    if pending.player_id != player || pending.character_id != character || pending.actor != actor {
        return Err("Only the declaring player can correct this pending action.".into());
    }
    Ok(pending)
}
fn intent_message(intent: &TableIntent) -> String {
    match intent { TableIntent::Unresolved{question}=>question.clone(), _=>"Declaration understood. It remains uncommitted until the table requests its roll; you can correct it now.".into() }
}
