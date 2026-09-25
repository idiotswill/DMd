//! Pure table transitions compose validated source rules into one atomic event.
use crate::table_protocol::*;
use dmd_conversation::{LocalText, interpret_local_text};
use dmd_domain::*;
use dmd_persistence::SessionChange;
use dmd_rules::{RulesAction, RulesEvent, RulesOutcome, RulesPack};

pub(crate) struct TableTransition {
    pub state: CampaignState,
    pub event: TableEvent,
    pub session_change: Option<SessionChange>,
}

pub(crate) fn resolve_table(
    state: &CampaignState,
    meta: &CommandMeta,
    action: &TableAction,
    pack: &RulesPack,
) -> Result<TableTransition, String> {
    if meta.campaign_id != state.campaign_id()
        || meta.expected_event_sequence != state.applied_event_sequence
    {
        return Err("The table changed. Refresh and review your action before retrying.".into());
    }
    validate_table(state, pack)?;
    let mut next = state.clone();
    let mut session_change = None;
    let mut rules_event = None;
    let mut mechanics = None;
    let message = match action {
        TableAction::UpdateContract { contract } => {
            host(meta)?;
            contract.validate()?;
            let current = table(&next)?;
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
            table_mut(&mut next)?.contract = contract.clone();
            "The table contract was updated.".into()
        }
        TableAction::AddPlayer { id, name } => {
            host(meta)?;
            setup_only(&next)?;
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
        TableAction::CreateCharacter {
            character_id,
            entity_id,
            player_id,
            input,
        } => {
            host(meta)?;
            setup_only(&next)?;
            if !next.players.contains_key(player_id)
                || next.characters.contains_key(character_id)
                || next.entities.contains_key(entity_id)
            {
                return Err("Select an existing player and a new character identity.".into());
            }
            let built = dmd_rules::build_character(input, *entity_id, pack)
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
                &mut next,
                meta,
                RulesAction::CreateCharacter {
                    entity_id: *entity_id,
                    input: input.clone(),
                },
                pack,
                &mut rules_event,
                &mut mechanics,
            )?;
            table_mut(&mut next)?
                .character_profiles
                .insert(*character_id, built.profile.clone());
            format!(
                "{} was created as a level 1 Human Fighter with the Soldier background.",
                built.profile.name
            )
        }
        TableAction::PrepareEquipment {
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
                || state.encounter.is_some()
            {
                return Err(
                    "Finish pending decisions and prepare equipment before battlefield setup."
                        .into(),
                );
            }
            next = crate::table_equipment::prepare(state, meta, *character_id, item_ids, pack)?;
            "Starting equipment is ready for play.".into()
        }
        TableAction::CreateCreature { creation } => {
            host(meta)?;
            if table(state)?.active_session.is_some() {
                active(state, meta)?;
            } else if meta.session_id.is_some() {
                return Err("Creature setup does not belong to an active session.".into());
            }
            idle(state)?;
            if state.encounter.is_some() {
                return Err("Prepare source creatures before setting up the battlefield.".into());
            }
            next = crate::table_creatures::create(state, meta, creation, pack)?;
            // Private preparation must not teach players an actor's name or existence.
            "Host preparation recorded.".into()
        }
        TableAction::StartSession {
            id,
            name,
            participants,
        } => {
            host(meta)?;
            setup_only(&next)?;
            bounded_text(name, 200)?;
            if meta.session_id != Some(*id)
                || participants.is_empty()
                || !participants
                    .iter()
                    .any(|p| p.attendance == AttendanceStatus::Present && p.character_id.is_some())
            {
                return Err("Choose at least one attending player and their character.".into());
            }
            for participant in participants {
                if let Some(id) = participant.character_id {
                    let pc = next
                        .characters
                        .get(&id)
                        .ok_or("Unknown session character.")?;
                    if pc.status != CharacterStatus::Active
                        || pc.controlling_player_id != Some(participant.player_id)
                    {
                        return Err(
                            "A session character must be active and belong to the selected player."
                                .into(),
                        );
                    }
                    if !table(&next)?.character_profiles.contains_key(&id) {
                        return Err("This character has no supported creation profile.".into());
                    }
                }
            }
            let binding = ActiveTableSession {
                session_id: *id,
                display_name: name.trim().into(),
                started_at_world: next.clock.now,
                participants: participants.clone(),
            };
            let session = binding.as_session(meta.campaign_id);
            if !session.validate_against_state(&next).is_empty() {
                return Err("The session bindings are inconsistent.".into());
            }
            table_mut(&mut next)?.active_session = Some(binding);
            session_change = Some(SessionChange::Start { session });
            format!("Session started: {}.", name.trim())
        }
        TableAction::EndSession => {
            host(meta)?;
            idle(&next)?;
            let binding = active(&next, meta)?;
            let expected = binding.as_session(meta.campaign_id);
            let mut ended = expected.clone();
            ended.status = PlaySessionStatus::Closed;
            ended.ended_at_world = Some(next.clock.now);
            table_mut(&mut next)?.active_session = None;
            session_change = Some(SessionChange::Replace {
                expected,
                next: ended,
            });
            "Session ended. Accepted events and the recap are saved.".into()
        }
        TableAction::SetSituation { situation } => {
            host(meta)?;
            if table(&next)?.roll_context.is_some() {
                return Err("Resolve the pending roll before changing its situation.".into());
            }
            if situation
                .challenges
                .iter()
                .any(|challenge| challenge.resolution.is_some())
            {
                return Err("A new situation cannot contain invented resolutions.".into());
            }
            if table(&next)?.active_session.is_some() {
                active(&next, meta)?;
            }
            table_mut(&mut next)?.situation = situation.clone();
            format!("Situation: {}. {}", situation.title, situation.description)
        }
        TableAction::Declare { text } => {
            idle(&next)?;
            let (player_id, character_id, actor, session_id) = player_channel(&next, meta)?;
            let intent = declaration(text, &table(&next)?.situation)?;
            let response = intent_message(&intent);
            table_mut(&mut next)?.pending = Some(PendingTableDecision {
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
        TableAction::Correct {
            pending_id,
            revision,
            text,
        } => {
            let pending = owned_pending(&next, meta, *pending_id, *revision)?.clone();
            let intent = declaration(text, &table(&next)?.situation)?;
            let response = intent_message(&intent);
            table_mut(&mut next)?.pending = Some(PendingTableDecision {
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
        TableAction::CancelDecision {
            pending_id,
            revision,
        } => {
            owned_pending(&next, meta, *pending_id, *revision)?;
            table_mut(&mut next)?.pending = None;
            "The uncommitted declaration was withdrawn. No world outcome occurred.".into()
        }
        TableAction::Adjudicate {
            pending_id,
            revision,
            request_id,
        } => {
            host(meta)?;
            active(&next, meta)?;
            let pending = pending_match(&next, *pending_id, *revision)?.clone();
            let (action, challenge_id) = match &pending.intent {
                TableIntent::Check {
                    kind,
                    challenge_id: Some(id),
                    ..
                } => {
                    let challenge = table(&next)?
                        .situation
                        .challenges
                        .iter()
                        .find(|c| c.id == *id)
                        .ok_or("The challenge is no longer available.")?;
                    if challenge.resolution.is_some() || &challenge.kind != kind {
                        return Err("The challenge context changed. Review the declaration.".into());
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
            table_mut(&mut next)?.pending = None;
            apply_rules(
                &mut next,
                meta,
                action,
                pack,
                &mut rules_event,
                &mut mechanics,
            )?;
            match &mechanics {
                Some(RulesOutcome::RollRequested(_)) => {
                    table_mut(&mut next)?.roll_context = Some(TableRollContext {
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
        TableAction::SubmitPhysical { request_id, faces } => {
            let (player, character, actor, _) = player_channel(&next, meta)?;
            let context = table(&next)?
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
                &mut next,
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
            )?;
            table_mut(&mut next)?.roll_context = None;
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
                        let challenge = table_mut(&mut next)?
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
    validate_table(&next, pack)?;
    Ok(TableTransition {
        state: next,
        event: TableEvent {
            meta: meta.clone(),
            action: action.clone(),
            outcome: TableOutcome { message, mechanics },
            rules_event,
        },
        session_change,
    })
}

pub(crate) fn replay_table(
    state: &CampaignState,
    event: &TableEvent,
    pack: &RulesPack,
) -> Result<TableTransition, String> {
    let transition = resolve_table(state, &event.meta, &event.action, pack)?;
    if transition.event != *event {
        return Err("Table event disagrees with deterministic replay.".into());
    }
    Ok(transition)
}

pub(crate) fn validate_table(state: &CampaignState, pack: &RulesPack) -> Result<(), String> {
    dmd_rules::validate_state(state, pack).map_err(|error| error.to_string())?;
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
            crate::table_equipment::validate_character_equipment(state, profile, entity, pack)?;
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
) -> Result<(), String> {
    let transition =
        dmd_rules::resolve(state, meta, &action, pack).map_err(|error| error.to_string())?;
    *state = transition.next_state;
    *outcome = Some(transition.outcome);
    *event = Some(transition.event);
    Ok(())
}
pub(crate) fn table(state: &CampaignState) -> Result<&TableState, String> {
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
pub(crate) fn player_channel(
    state: &CampaignState,
    meta: &CommandMeta,
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
                && pc.status == CharacterStatus::Active
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
fn declaration(text: &str, situation: &TableSituation) -> Result<TableIntent, String> {
    bounded_text(text, 8000)?;
    match interpret_local_text(text, situation) {
        LocalText::Declaration(intent) => Ok(intent),
        LocalText::Correction(text) => declaration(&text, situation),
        _ => Err(
            "That text is a question or table chat; send it through the conversation path.".into(),
        ),
    }
}
fn intent_message(intent: &TableIntent) -> String {
    match intent { TableIntent::Unresolved{question}=>question.clone(), _=>"Declaration understood. It remains uncommitted until the table requests its roll; you can correct it now.".into() }
}
