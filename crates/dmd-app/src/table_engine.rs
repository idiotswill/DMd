//! Pure table envelopes: language is a proposal; the rules owner performs mutations.
use crate::table_protocol::*;
use dmd_conversation::{LocalText, interpret_local_text};
use dmd_domain::*;
use dmd_persistence::SessionChange;
use dmd_rules::RulesPack;
pub(crate) use dmd_rules::table::reducer::{
    closed_session_release_action, player_channel, table, validate_table,
};
use dmd_rules::table::{ExpectedNested, TableOperation, TableProduced, TableSessionChange};

pub(crate) struct TableTransition {
    pub state: CampaignState,
    pub event: TableEvent,
    pub session_change: Option<SessionChange>,
}

fn operation(state: &CampaignState, action: &TableAction) -> Result<TableOperation, String> {
    Ok(match action {
        TableAction::AwardExcessInspiration {
            character_id,
            reason,
        } => TableOperation::AwardExcessInspiration {
            character_id: *character_id,
            reason: reason.clone(),
        },
        TableAction::ResolveHostInspirationTransfer { choice } => {
            TableOperation::ResolveHostInspirationTransfer {
                choice: choice.clone(),
            }
        }
        TableAction::AwardHeroicInspiration {
            character_id,
            reason,
        } => TableOperation::AwardHeroicInspiration {
            character_id: *character_id,
            reason: reason.clone(),
        },
        TableAction::EnableGrappleAccess => TableOperation::EnableGrappleAccess,
        TableAction::CreateCharacterFromSource {
            character_id,
            entity_id,
            player_id,
            source,
            input,
        } => TableOperation::CreateCharacterFromSource {
            character_id: *character_id,
            entity_id: *entity_id,
            player_id: *player_id,
            source: source.clone(),
            input: input.clone(),
        },
        TableAction::EnableGrappleTransport => TableOperation::EnableGrappleTransport,
        TableAction::UpdateContract { contract } => TableOperation::UpdateContract {
            contract: contract.clone(),
        },
        TableAction::AddPlayer { id, name } => TableOperation::AddPlayer {
            id: *id,
            name: name.clone(),
        },
        TableAction::CreateCharacter {
            character_id,
            entity_id,
            player_id,
            input,
        } => TableOperation::CreateCharacter {
            character_id: *character_id,
            entity_id: *entity_id,
            player_id: *player_id,
            input: input.clone(),
        },
        TableAction::PrepareEquipment {
            character_id,
            item_ids,
        } => TableOperation::PrepareEquipment {
            character_id: *character_id,
            item_ids: item_ids.clone(),
        },
        TableAction::CreateCreature { creation } => TableOperation::CreateCreature {
            creation: creation.clone(),
        },
        TableAction::EnableSourceActorAccess { adopted } => {
            TableOperation::EnableSourceActorAccess {
                adopted: adopted.clone(),
            }
        }
        TableAction::SetSourceCreatureController { actor, controller } => {
            TableOperation::SetSourceCreatureController {
                actor: *actor,
                controller: *controller,
            }
        }
        TableAction::Tactical { action } => TableOperation::Tactical {
            action: action.clone(),
        },
        TableAction::PrepareBattlefield { setup } => TableOperation::PrepareBattlefield {
            setup: setup.clone(),
        },
        TableAction::StartSession {
            id,
            name,
            participants,
        } => TableOperation::StartSession {
            id: *id,
            name: name.clone(),
            participants: participants.clone(),
        },
        TableAction::EndSession => TableOperation::EndSession,
        TableAction::SetSituation { situation } => TableOperation::SetSituation {
            situation: situation.clone(),
        },
        TableAction::Declare { text } => TableOperation::Declare {
            text: text.clone(),
            intent: declaration(text, &table(state)?.situation)?,
        },
        TableAction::Correct {
            pending_id,
            revision,
            text,
        } => TableOperation::Correct {
            pending_id: *pending_id,
            revision: *revision,
            text: text.clone(),
            intent: declaration(text, &table(state)?.situation)?,
        },
        TableAction::CancelDecision {
            pending_id,
            revision,
        } => TableOperation::CancelDecision {
            pending_id: *pending_id,
            revision: *revision,
        },
        TableAction::Adjudicate {
            pending_id,
            revision,
            request_id,
        } => TableOperation::Adjudicate {
            pending_id: *pending_id,
            revision: *revision,
            request_id: *request_id,
        },
        TableAction::SubmitPhysical { request_id, faces } => TableOperation::SubmitPhysical {
            request_id: *request_id,
            faces: faces.clone(),
        },
    })
}
fn composed(
    meta: &CommandMeta,
    action: &TableAction,
    produced: TableProduced,
) -> (TableEvent, Option<SessionChange>) {
    let session_change = produced.session_change.map(|change| match change {
        TableSessionChange::Start { session } => SessionChange::Start { session },
        TableSessionChange::Replace { expected, next } => SessionChange::Replace { expected, next },
    });
    (
        TableEvent {
            meta: meta.clone(),
            action: action.clone(),
            outcome: TableOutcome {
                message: produced.message,
                mechanics: produced.mechanics,
            },
            rules_event: produced.rules_event,
            tactical_event: produced.tactical_event,
        },
        session_change,
    )
}

pub(crate) fn resolve_table(
    state: &CampaignState,
    meta: &CommandMeta,
    action: &TableAction,
    pack: &RulesPack,
) -> Result<TableTransition, String> {
    let operation = operation(state, action)?;
    let transition =
        dmd_rules::table::reducer::resolve_ordinary(state, meta, &operation, pack, None)?;
    let (event, session_change) = composed(meta, action, transition.produced);
    Ok(TableTransition {
        state: transition.state,
        event,
        session_change,
    })
}

pub(crate) fn replay_table(
    state: &CampaignState,
    event: &TableEvent,
    pack: &RulesPack,
) -> Result<TableTransition, String> {
    let operation = operation(state, &event.action)?;
    if matches!(event.action, TableAction::Tactical { .. }) && event.tactical_event.is_none() {
        return Err("Historical encounter action lacks its exact source event.".into());
    }
    let transition = dmd_rules::table::reducer::resolve_ordinary(
        state,
        &event.meta,
        &operation,
        pack,
        Some(ExpectedNested {
            rules_event: event.rules_event.as_ref(),
            tactical_event: event.tactical_event.as_ref(),
        }),
    )?;
    let (actual, session_change) = composed(&event.meta, &event.action, transition.produced);
    if actual != *event {
        return Err("Table event disagrees with deterministic replay.".into());
    }
    Ok(TableTransition {
        state: transition.state,
        event: actual,
        session_change,
    })
}

pub(crate) fn prepare_owned<'a>(
    owner: &'a mut dmd_rules::table::CampaignExecution,
    meta: &CommandMeta,
    action: &TableAction,
) -> Result<
    (
        dmd_rules::table::PreparedTableStep<'a>,
        TableEvent,
        Option<SessionChange>,
    ),
    String,
> {
    let operation = operation(owner.read().state(), action)?;
    let step = owner.prepare_table(meta, &operation)?;
    let (event, session_change) = composed(meta, action, step.produced().clone());
    Ok((step, event, session_change))
}

pub(crate) fn prepare_replay_owned<'a>(
    owner: &'a mut dmd_rules::table::CampaignExecution,
    event: &TableEvent,
) -> Result<dmd_rules::table::PreparedTableStep<'a>, String> {
    let operation = operation(owner.read().state(), &event.action)?;
    let step = owner.prepare_table_replay(
        &event.meta,
        &operation,
        ExpectedNested {
            rules_event: event.rules_event.as_ref(),
            tactical_event: event.tactical_event.as_ref(),
        },
    )?;
    let (actual, _) = composed(&event.meta, &event.action, step.produced().clone());
    if actual != *event {
        return Err("Table event disagrees with deterministic replay.".into());
    }
    Ok(step)
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
