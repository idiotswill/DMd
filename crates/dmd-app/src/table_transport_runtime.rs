//! One SQLite writer owns revision comparison, canonical-head derivation, gameplay
//! acceptance, and the durable exact response. No second pool acquisition occurs.
use crate::table_presentation_history::{self as presentation, PresentationHistory};
use crate::table_transport::{TransportedTableAction, TransportedTableObservation};
use crate::*;
use dmd_conversation::{LocalText, interpret_local_text};
use dmd_domain::*;
use dmd_persistence::*;
use dmd_rules::{RulesPack, tactical::TacticalAction};
use std::collections::HashMap;

fn rejected(message: impl ToString) -> RunnableCampaignError {
    RunnableCampaignError::TableRejected(message.to_string())
}
fn recovery(error: impl ToString) -> RunnableCampaignError {
    RunnableCampaignError::Table(error.to_string())
}
fn json<T: serde::Serialize>(value: &T) -> Result<String, String> {
    serde_json::to_string(value).map_err(|e| e.to_string())
}

fn request_meta(
    state: &CampaignState,
    request: &TableTransportRequest,
    latest: &HashMap<ProjectionAudience, ProjectionChange>,
) -> Result<CommandMeta, String> {
    if request.version != TABLE_TRANSPORT_VERSION || request.campaign_id != state.campaign_id() {
        return Err("Unsupported table request.".into());
    }
    if latest.get(&request.channel.audience()).map(|v| v.revision) != Some(request.revision) {
        return Err("The visible table changed. Refresh before continuing.".into());
    }
    let (issuer, actor) = match request.channel {
        TableTransportChannel::Host => (CommandIssuer::Admin, None),
        TableTransportChannel::Player {
            player_id,
            character_id,
        } => {
            let character = state
                .characters
                .get(&character_id)
                .filter(|c| c.controlling_player_id == Some(player_id))
                .ok_or("Select a character controlled by this player.")?;
            (
                CommandIssuer::Player(player_id),
                Some(AgentRef::Entity(character.entity_id)),
            )
        }
    };
    Ok(CommandMeta {
        id: request.command_id,
        campaign_id: request.campaign_id,
        session_id: request.session_id,
        issuer,
        actor,
        expected_event_sequence: state.applied_event_sequence,
    })
}
enum Intent {
    Action(Box<TableAction>),
    Observation(String),
}
fn derive_intent(
    state: &CampaignState,
    request: &TableTransportRequest,
    latest: &HashMap<ProjectionAudience, ProjectionChange>,
) -> Result<Intent, String> {
    let current = latest
        .get(&request.channel.audience())
        .ok_or("Unknown audience.")?;
    let roll = |opaque: RollRequestId| {
        current
            .handles
            .iter()
            .find_map(|h| match h.capability {
                ProjectionCapability::Roll { canonical } if h.opaque == opaque.0 => Some(canonical),
                _ => None,
            })
            .ok_or_else(|| "That roll is not available in this view.".to_owned())
    };
    Ok(match &request.input {
        TableTransportInput::SelectWork { handle } => {
            let (origin, occurrence) = current
                .handles
                .iter()
                .find_map(|h| match h.capability {
                    ProjectionCapability::Work { origin, occurrence } if h.opaque == handle.0 => {
                        Some((origin, occurrence))
                    }
                    _ => None,
                })
                .ok_or("That decision is not available in this view.")?;
            if presentation::work_origin(state) != Some(origin) {
                return Err("That decision is no longer available.".into());
            }
            Intent::Action(Box::new(TableAction::Tactical {
                action: TacticalAction::ChooseTurnWork { occurrence },
            }))
        }
        TableTransportInput::Action(action) => {
            let mut action = (**action).clone();
            match &mut action {
                TableAction::SubmitPhysical { request_id, .. } => *request_id = roll(*request_id)?,
                TableAction::Tactical { action } => match action {
                    TacticalAction::ChooseTurnWork { .. } => {
                        return Err("Select the visible decision handle.".into());
                    }
                    TacticalAction::SubmitRoll { result }
                    | TacticalAction::SubmitRollWithInspiration { result, .. } => {
                        if result.source != RollSource::Physical {
                            return Err("Physical input cannot choose the roll source.".into());
                        }
                        result.request_id = roll(result.request_id)?;
                    }
                    TacticalAction::SubmitSavageAttacker { roll: sets } => {
                        if sets.first.source != RollSource::Physical
                            || sets.second.source != RollSource::Physical
                            || sets.first.request_id != sets.second.request_id
                        {
                            return Err(
                                "Both physical sets must answer the same visible roll.".into()
                            );
                        }
                        let canonical = roll(sets.first.request_id)?;
                        sets.first.request_id = canonical;
                        sets.second.request_id = canonical;
                    }
                    _ => {}
                },
                _ => {}
            }
            Intent::Action(Box::new(action))
        }
        TableTransportInput::Text { text } => {
            bounded_text(text, 8000)?;
            let table = crate::table_engine::table(state)?;
            match interpret_local_text(text, &table.situation) {
                LocalText::RulesQuestion
                | LocalText::CharacterQuestion
                | LocalText::WorldQuestion
                | LocalText::TableChat => Intent::Observation(text.clone()),
                LocalText::Correction(text) => {
                    let pending = table
                        .pending
                        .as_ref()
                        .ok_or("There is no uncommitted declaration to correct.")?;
                    Intent::Action(Box::new(TableAction::Correct {
                        pending_id: pending.id,
                        revision: pending.revision,
                        text,
                    }))
                }
                LocalText::Declaration(_) => {
                    Intent::Action(Box::new(TableAction::Declare { text: text.clone() }))
                }
            }
        }
    })
}
fn receipt(
    request: &TableTransportRequest,
    history: &PresentationHistory,
    outcome: &TableOutcome,
) -> Result<TableTransportResult, String> {
    Ok(TableTransportResult::Accepted(TablePresentedReceipt {
        command_id: request.command_id,
        revision: history
            .latest
            .get(&request.channel.audience())
            .ok_or("missing accepted audience")?
            .revision,
        outcome: TablePresentedOutcome {
            message: outcome.message.clone(),
        },
    }))
}
fn observed(
    request: &TableTransportRequest,
    history: &PresentationHistory,
    body: &TableObservationBody,
) -> Result<TableTransportResult, String> {
    Ok(TableTransportResult::Observed(TablePresentedObservation {
        command_id: request.command_id,
        revision: history
            .latest
            .get(&request.channel.audience())
            .ok_or("missing observation audience")?
            .revision,
        text: body.text.clone(),
        answer: body.answer.clone(),
    }))
}
fn binding(
    export: &CampaignExport,
    id: CommandId,
) -> Result<Option<&TableTransportBinding>, String> {
    let mut found = export
        .table_transport_bindings
        .iter()
        .filter(|b| b.meta.id == id);
    let result = found.next();
    if found.next().is_some() {
        return Err("duplicate transport acceptance".into());
    }
    Ok(result)
}
fn check_binding(
    saved: &TableTransportBinding,
    request: &TableTransportRequest,
    meta: &CommandMeta,
    history: &PresentationHistory,
    acceptance: TransportAcceptance,
    response: TableTransportResult,
) -> Result<(), String> {
    if &saved.meta != meta
        || saved.audience != request.channel.audience()
        || saved.projection_ordinal != history.ordinal
        || saved.acceptance != acceptance
        || saved.request_json != json(request)?
        || saved.response_json != json(&response)?
    {
        return Err("transport binding disagrees with historical acceptance".into());
    }
    Ok(())
}
pub(crate) fn validate_event_binding(
    export: &CampaignExport,
    before: &CampaignState,
    after: &CampaignState,
    row: &EventJournalRow,
    prior: &HashMap<ProjectionAudience, ProjectionChange>,
    history: &PresentationHistory,
    _pack: &RulesPack,
) -> Result<(), String> {
    let event: TableEvent = serde_json::from_str(&row.payload_json).map_err(|e| e.to_string())?;
    let audit = export
        .command_audit
        .iter()
        .find(|a| a.id == row.command_id)
        .ok_or("event audit absent")?;
    let saved = binding(export, event.meta.id)?;
    if audit.command_schema_version == 1 {
        if saved.is_some() {
            return Err("legacy acceptance has an unsolicited transport binding".into());
        }
        return Ok(());
    }
    if audit.command_schema_version != 2 {
        return Err("unsupported transport acceptance".into());
    }
    let envelope: TransportedTableAction =
        serde_json::from_str(&audit.payload_json).map_err(|e| e.to_string())?;
    let meta = request_meta(before, &envelope.request, prior)?;
    let Intent::Action(action) = derive_intent(before, &envelope.request, prior)? else {
        return Err("observation request became a game command".into());
    };
    if meta != event.meta || *action != event.action || *action != envelope.action {
        return Err("transport intent does not reproduce canonical action".into());
    }
    check_binding(
        saved.ok_or("canonical transport acceptance lost its binding")?,
        &envelope.request,
        &meta,
        history,
        TransportAcceptance::Command {
            resulting_event_sequence: after.applied_event_sequence,
        },
        receipt(&envelope.request, history, &event.outcome)?,
    )
}
pub(crate) fn validate_observation_binding(
    export: &CampaignExport,
    state: &CampaignState,
    observation: &SessionObservation,
    prior: &HashMap<ProjectionAudience, ProjectionChange>,
    history: &PresentationHistory,
    pack: &RulesPack,
) -> Result<(), String> {
    let record = &observation.record;
    let saved = binding(export, CommandId(record.id.0))?;
    if record.payload_schema_version == 1 {
        if saved.is_some() {
            return Err("legacy observation has an unsolicited transport binding".into());
        }
        return Ok(());
    }
    if record.kind != "table.conversation" || record.payload_schema_version != 2 {
        return Err("unsupported protocol observation".into());
    }
    let envelope: TransportedTableObservation =
        serde_json::from_str(&record.payload_json).map_err(|e| e.to_string())?;
    let meta = request_meta(state, &envelope.request, prior)?;
    let Intent::Observation(text) = derive_intent(state, &envelope.request, prior)? else {
        return Err("game action became a protocol observation".into());
    };
    let (expected, mut expected_record) =
        crate::table_runtime::propose_observation(state, pack, &meta, record.id, &text)
            .map_err(|e| e.to_string())?;
    expected_record.payload_schema_version = 2;
    expected_record.payload_json = json(&envelope)?;
    if expected != envelope.body || &expected_record != record {
        return Err("observation differs from its authenticated historical answer".into());
    }
    check_binding(
        saved.ok_or("canonical observation lost its transport binding")?,
        &envelope.request,
        &meta,
        history,
        TransportAcceptance::Observation { id: record.id },
        observed(&envelope.request, history, &expected)?,
    )
}

impl CampaignRuntime {
    pub async fn table_roll_options(
        &self,
        request: TableRollOptionsRequest,
    ) -> Result<TableRollOptions, RunnableCampaignError> {
        // A read transaction provides one snapshot; this neither bootstraps nor
        // rewrites accepted audience digests, bindings, responses or game state.
        let mut tx = self.pool.begin().await.map_err(recovery)?;
        let export = export_campaign_in_transaction(&mut tx, request.campaign_id)
            .await
            .map_err(recovery)?;
        let (state, pack) = self.protocol_pack(&export)?;
        let history = presentation::validate_history(&export, &pack).map_err(recovery)?;
        let visible = history
            .latest
            .get(&request.channel.audience())
            .filter(|entry| entry.revision == request.revision)
            .ok_or_else(|| rejected("Refresh the current roll before viewing its options."))?;
        let canonical = visible
            .handles
            .iter()
            .find_map(|handle| match handle.capability {
                ProjectionCapability::Roll { canonical } if handle.opaque == request.roll_id.0 => {
                    Some(canonical)
                }
                _ => None,
            })
            .ok_or_else(|| rejected("That roll is not available in this view."))?;
        let pending = state
            .rules
            .as_ref()
            .and_then(|rules| rules.pending.as_ref())
            .filter(|pending| pending.request.id == canonical)
            .ok_or_else(|| rejected("That roll has already changed."))?;
        let actor = pending
            .request
            .roller
            .ok_or_else(|| recovery("Pending roller absent."))?;
        if let TableTransportChannel::Player {
            player_id,
            character_id,
        } = request.channel
            && !state
                .characters
                .get(&character_id)
                .is_some_and(|character| {
                    character.controlling_player_id == Some(player_id)
                        && character.entity_id == actor
                })
        {
            return Err(rejected("Select the character who owns this roll."));
        }
        let savage_attacker = dmd_rules::tactical::savage_attacker_dice(&state, &pack)
            .ok()
            .map(|weapon_dice| TableSavageAttackerOption {
                weapon_dice,
                heroic_inspiration: state.rules.as_ref().unwrap().entities[&actor]
                    .heroic_inspiration,
            });
        tx.commit().await.map_err(recovery)?;
        Ok(TableRollOptions { savage_attacker })
    }
    pub async fn recover_legacy_table_request(
        &self,
        request: LegacyTableRequest,
    ) -> Result<LegacyTableAcknowledgement, RunnableCampaignError> {
        let mut tx = self
            .pool
            .begin_with("BEGIN IMMEDIATE")
            .await
            .map_err(recovery)?;
        let export = export_campaign_in_transaction(&mut tx, request.campaign_id)
            .await
            .map_err(recovery)?;
        let (_, pack) = self.protocol_pack(&export)?;
        presentation::validate_history(&export, &pack).map_err(recovery)?;
        let (meta, response) = if let Some(audit) = export
            .command_audit
            .iter()
            .find(|a| a.id == request.command_id.0.to_string())
        {
            if audit.command_schema_version != 1 {
                return Err(rejected(
                    "Use the original versioned request to recover this acceptance.",
                ));
            }
            let action = crate::rules_restore::table_audit_action(audit).map_err(recovery)?;
            let matches = match (&request.input, &action) {
                (LegacyTableInput::Action(input), action) => input.as_ref() == action,
                (LegacyTableInput::Text { text }, TableAction::Declare { text: accepted }) => {
                    text == accepted
                }
                (LegacyTableInput::Text { text }, TableAction::Correct { text: accepted, .. }) => {
                    matches!(interpret_local_text(text,&TableSituation::default()),LocalText::Correction(value) if &value==accepted)
                }
                _ => false,
            };
            if !matches {
                return Err(rejected(
                    "This identity already belongs to different input.",
                ));
            }
            let outcome: TableOutcome =
                serde_json::from_str(&audit.resolution_explanation).map_err(recovery)?;
            (
                crate::rules_restore::audit_meta(audit).map_err(recovery)?,
                LegacyTableAcknowledgement::Accepted {
                    command_id: request.command_id,
                    outcome: TablePresentedOutcome {
                        message: outcome.message,
                    },
                },
            )
        } else if let Some(observation) = export
            .observations
            .iter()
            .find(|o| o.record.id.0 == request.command_id.0)
        {
            if observation.record.payload_schema_version != 1 {
                return Err(rejected(
                    "Use the original versioned request to recover this answer.",
                ));
            }
            let body = crate::table_runtime::validate_table_observation(&observation.record)
                .map_err(recovery)?;
            if !matches!(&request.input,LegacyTableInput::Text{text} if text==&body.text) {
                return Err(rejected(
                    "This identity already belongs to different input.",
                ));
            }
            (
                body.meta,
                LegacyTableAcknowledgement::Observed {
                    command_id: request.command_id,
                    text: body.text,
                    answer: body.answer,
                },
            )
        } else {
            return Err(rejected(
                "This legacy request was not accepted. Refresh before submitting new input.",
            ));
        };
        if meta.campaign_id != request.campaign_id
            || meta.expected_event_sequence != request.expected_event_sequence
            || meta.session_id != request.session_id
        {
            return Err(rejected(
                "Legacy request context differs from its original acceptance.",
            ));
        }
        // Authenticate selection at the original semantic image, not today's
        // attendance or ownership. The caller cannot substitute an arbitrary actor.
        let mut matched = false;
        crate::rules_restore::visit_rules_history(&export, &pack, |_, state, _| {
            if state.applied_event_sequence == meta.expected_event_sequence {
                matched = match request.channel {
                    TableTransportChannel::Host => {
                        meta.issuer == CommandIssuer::Admin && meta.actor.is_none()
                    }
                    TableTransportChannel::Player {
                        player_id,
                        character_id,
                    } => {
                        meta.issuer == CommandIssuer::Player(player_id)
                            && state.characters.get(&character_id).is_some_and(|c| {
                                c.controlling_player_id == Some(player_id)
                                    && meta.actor == Some(AgentRef::Entity(c.entity_id))
                            })
                    }
                };
            }
            Ok(())
        })
        .map_err(recovery)?;
        if !matched {
            return Err(rejected(
                "Legacy channel does not match its original acceptance.",
            ));
        }
        tx.commit().await.map_err(recovery)?;
        Ok(response)
    }
    /// Trusted internal v1 API remains available, but every accepted write after
    /// presentation initialization participates in the same durable transaction.
    pub(crate) async fn execute_legacy_table(
        &self,
        meta: CommandMeta,
        action: TableAction,
        accept_new: bool,
    ) -> Result<TableReceipt, RunnableCampaignError> {
        let mut tx = self
            .pool
            .begin_with("BEGIN IMMEDIATE")
            .await
            .map_err(recovery)?;
        let mut export = export_campaign_in_transaction(&mut tx, meta.campaign_id)
            .await
            .map_err(recovery)?;
        let (state, pack) = self.protocol_pack(&export)?;
        let mut history = presentation::validate_history(&export, &pack).map_err(recovery)?;
        if let Some(audit) = export
            .command_audit
            .iter()
            .find(|a| a.id == meta.id.0.to_string())
        {
            if audit.command_schema_version != 1
                || crate::rules_restore::audit_meta(audit).map_err(recovery)? != meta
                || crate::rules_restore::table_audit_action(audit).map_err(recovery)? != action
            {
                return Err(rejected(
                    "This request identity already belongs to different input.",
                ));
            }
            let receipt = TableReceipt {
                command_id: meta.id,
                event_sequence: u64::try_from(audit.resulting_event_sequence).map_err(recovery)?,
                outcome: serde_json::from_str(&audit.resolution_explanation).map_err(recovery)?,
                already_accepted: true,
            };
            tx.commit().await.map_err(recovery)?;
            return Ok(receipt);
        }
        if !accept_new {
            return Err(rejected(
                "Refresh this legacy request before submitting new input.",
            ));
        }
        if export
            .observations
            .iter()
            .any(|o| o.record.id.0 == meta.id.0)
        {
            return Err(rejected("This identity belongs to a conversation request."));
        }
        // A legacy-only campaign need not acquire a presentation protocol until a
        // desktop actually requests one. Once issued, revisions cannot be bypassed.
        let mut transition =
            crate::table_engine::resolve_table(&state, &meta, &action, &pack).map_err(rejected)?;
        transition.state.applied_event_sequence = meta
            .expected_event_sequence
            .checked_add(1)
            .ok_or_else(|| rejected("Event sequence exhausted."))?;
        let id = EventId::new();
        let event = PendingEvent {
            id,
            occurred_at: transition.state.clock.now,
            source: EventSource::RuleResolution,
            actor: meta.actor,
            caused_by_event_ids: vec![],
            payload: transition.event.clone(),
        }
        .encode(TABLE_EVENT_KIND, TABLE_EVENT_VERSION)
        .map_err(rejected)?;
        commit_campaign_transition_in_transaction(
            &mut tx,
            &meta,
            &SerializedRecord::encode("table.action", 1, &action).map_err(rejected)?,
            &transition.state,
            &[event],
            &json(&transition.event.outcome).map_err(rejected)?,
            transition.session_change.as_ref(),
        )
        .await
        .map_err(|e| RunnableCampaignError::Journal(Box::new(e)))?;
        if history.ordinal > 0 {
            let row =
                event_row(&meta, id, &transition.state, &transition.event).map_err(recovery)?;
            export.event_journal.push(row.clone());
            let visible = presentation::accepted_event_visibility(
                &state,
                &transition.state,
                &row,
                &pack,
                &mut history,
            )
            .map_err(recovery)?;
            let record = presentation::build_record(
                &transition.state,
                &pack,
                &export,
                &mut history,
                ProjectionCause::Event {
                    id,
                    command_id: meta.id,
                },
                visible,
            )
            .map_err(recovery)?;
            insert_table_projection_record(&mut tx, &record)
                .await
                .map_err(recovery)?;
        }
        let complete = export_campaign_in_transaction(&mut tx, meta.campaign_id)
            .await
            .map_err(recovery)?;
        presentation::validate_history(&complete, &pack).map_err(recovery)?;
        let receipt = TableReceipt {
            command_id: meta.id,
            event_sequence: transition.state.applied_event_sequence,
            outcome: transition.event.outcome,
            already_accepted: false,
        };
        tx.commit().await.map_err(recovery)?;
        Ok(receipt)
    }
    pub(crate) async fn observe_legacy_table(
        &self,
        meta: CommandMeta,
        id: ObservationId,
        text: &str,
        accept_new: bool,
    ) -> Result<TableObservationBody, RunnableCampaignError> {
        bounded_text(text, 8000).map_err(rejected)?;
        if id.0 != meta.id.0 {
            return Err(rejected("Conversation and command identities disagree."));
        }
        let mut tx = self
            .pool
            .begin_with("BEGIN IMMEDIATE")
            .await
            .map_err(recovery)?;
        let mut export = export_campaign_in_transaction(&mut tx, meta.campaign_id)
            .await
            .map_err(recovery)?;
        let (state, pack) = self.protocol_pack(&export)?;
        let mut history = presentation::validate_history(&export, &pack).map_err(recovery)?;
        if let Some(saved) = export.observations.iter().find(|o| o.record.id == id) {
            let body = crate::table_runtime::validate_table_observation(&saved.record)
                .map_err(recovery)?;
            if saved.record.payload_schema_version != 1 || body.meta != meta || body.text != text {
                return Err(rejected(
                    "This conversation identity already belongs to different input.",
                ));
            }
            tx.commit().await.map_err(recovery)?;
            return Ok(body);
        }
        if !accept_new {
            return Err(rejected(
                "Refresh this legacy request before submitting new input.",
            ));
        }
        if export
            .command_audit
            .iter()
            .any(|a| a.id == meta.id.0.to_string())
        {
            return Err(rejected("This identity belongs to a game command."));
        }
        let (body, record) =
            crate::table_runtime::propose_observation(&state, &pack, &meta, id, text)?;
        let accepted =
            append_session_observations_in_transaction(&mut tx, meta.campaign_id, &[record])
                .await
                .map_err(recovery)?;
        if history.ordinal > 0 {
            history.observation_ordinal = accepted[0].ordinal;
            presentation::accepted_observation_visibility(
                &state,
                &accepted[0].record,
                &mut history,
            );
            export.observations.extend(accepted);
            let record = presentation::build_record(
                &state,
                &pack,
                &export,
                &mut history,
                ProjectionCause::Observation { id },
                vec![],
            )
            .map_err(recovery)?;
            insert_table_projection_record(&mut tx, &record)
                .await
                .map_err(recovery)?;
        }
        let complete = export_campaign_in_transaction(&mut tx, meta.campaign_id)
            .await
            .map_err(recovery)?;
        presentation::validate_history(&complete, &pack).map_err(recovery)?;
        tx.commit().await.map_err(recovery)?;
        Ok(body)
    }
    fn protocol_pack(
        &self,
        export: &CampaignExport,
    ) -> Result<(CampaignState, RulesPack), RunnableCampaignError> {
        let state =
            CampaignState::decode_json(&export.current_state.state_json).map_err(recovery)?;
        let catalog = self.load_catalog()?;
        let content = catalog.resolve_campaign(&state.campaign)?;
        let pack = crate::rules_runtime::load_rules_pack(&content)?;
        crate::table_engine::validate_table(&state, &pack).map_err(recovery)?;
        Ok((state, pack))
    }
    async fn bootstrap_presentation(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        export: &mut CampaignExport,
        state: &CampaignState,
        pack: &RulesPack,
        history: &mut PresentationHistory,
    ) -> Result<(), RunnableCampaignError> {
        if history.ordinal != 0 {
            return Ok(());
        }
        let record = presentation::build_record(
            state,
            pack,
            export,
            history,
            ProjectionCause::Bootstrap {
                event_sequence: state.applied_event_sequence,
                observation_ordinal: history.observation_ordinal,
            },
            history.visibility.clone(),
        )
        .map_err(recovery)?;
        insert_table_projection_record(tx, &record)
            .await
            .map_err(recovery)?;
        export.table_projection_history.push(record);
        Ok(())
    }
    pub async fn presented_table_view(
        &self,
        campaign_id: CampaignId,
        viewer: TableViewer,
    ) -> Result<TablePresentedView, RunnableCampaignError> {
        let mut tx = self
            .pool
            .begin_with("BEGIN IMMEDIATE")
            .await
            .map_err(recovery)?;
        let mut export = export_campaign_in_transaction(&mut tx, campaign_id)
            .await
            .map_err(recovery)?;
        let (state, pack) = self.protocol_pack(&export)?;
        let mut history = presentation::validate_history(&export, &pack).map_err(recovery)?;
        self.bootstrap_presentation(&mut tx, &mut export, &state, &pack, &mut history)
            .await?;
        let audience = match viewer {
            TableViewer::Host => ProjectionAudience::Host,
            TableViewer::Player(id) => ProjectionAudience::Player(id),
        };
        let view = presentation::current_view(&state, &pack, &export, &history, audience)
            .map_err(rejected)?;
        tx.commit().await.map_err(recovery)?;
        Ok(view)
    }
    pub async fn submit_presented_table(
        &self,
        request: TableTransportRequest,
    ) -> Result<TableTransportResult, RunnableCampaignError> {
        let mut tx = self
            .pool
            .begin_with("BEGIN IMMEDIATE")
            .await
            .map_err(recovery)?;
        let mut export = export_campaign_in_transaction(&mut tx, request.campaign_id)
            .await
            .map_err(recovery)?;
        let (state, pack) = self.protocol_pack(&export)?;
        let mut history = presentation::validate_history(&export, &pack).map_err(recovery)?;
        // Exact accepted identity wins before current audience ownership, attendance,
        // pending choice, or revision. A changed original envelope is never a retry.
        if let Some(saved) = binding(&export, request.command_id).map_err(recovery)? {
            if saved.request_json != json(&request).map_err(rejected)? {
                return Err(rejected(
                    "This request identity already belongs to different input.",
                ));
            }
            let response = serde_json::from_str(&saved.response_json).map_err(recovery)?;
            tx.commit().await.map_err(recovery)?;
            return Ok(response);
        }
        if export
            .command_audit
            .iter()
            .any(|a| a.id == request.command_id.0.to_string())
            || export
                .observations
                .iter()
                .any(|o| o.record.id.0 == request.command_id.0)
        {
            return Err(rejected(
                "This identity belongs to a legacy request; recover its original input.",
            ));
        }
        self.bootstrap_presentation(&mut tx, &mut export, &state, &pack, &mut history)
            .await?;
        let meta = request_meta(&state, &request, &history.latest).map_err(rejected)?;
        let intent = derive_intent(&state, &request, &history.latest).map_err(rejected)?;
        let (response, acceptance) = match intent {
            Intent::Action(action) => {
                let mut transition =
                    crate::table_engine::resolve_table(&state, &meta, &action, &pack)
                        .map_err(rejected)?;
                transition.state.applied_event_sequence = meta
                    .expected_event_sequence
                    .checked_add(1)
                    .ok_or_else(|| rejected("Event sequence exhausted."))?;
                let envelope = TransportedTableAction {
                    request: request.clone(),
                    action: *action,
                };
                let payload =
                    SerializedRecord::encode("table.action", 2, &envelope).map_err(rejected)?;
                let id = EventId::new();
                let event = PendingEvent {
                    id,
                    occurred_at: transition.state.clock.now,
                    source: EventSource::RuleResolution,
                    actor: meta.actor,
                    caused_by_event_ids: vec![],
                    payload: transition.event.clone(),
                }
                .encode(TABLE_EVENT_KIND, TABLE_EVENT_VERSION)
                .map_err(rejected)?;
                let explanation = json(&transition.event.outcome).map_err(rejected)?;
                commit_campaign_transition_in_transaction(
                    &mut tx,
                    &meta,
                    &payload,
                    &transition.state,
                    &[event],
                    &explanation,
                    transition.session_change.as_ref(),
                )
                .await
                .map_err(|e| RunnableCampaignError::Journal(Box::new(e)))?;
                // Only the rendered event fields are needed before the complete
                // transaction can be exported/validated again.
                let row =
                    event_row(&meta, id, &transition.state, &transition.event).map_err(recovery)?;
                export.event_journal.push(row.clone());
                let visible = presentation::accepted_event_visibility(
                    &state,
                    &transition.state,
                    &row,
                    &pack,
                    &mut history,
                )
                .map_err(recovery)?;
                let record = presentation::build_record(
                    &transition.state,
                    &pack,
                    &export,
                    &mut history,
                    ProjectionCause::Event {
                        id,
                        command_id: meta.id,
                    },
                    visible,
                )
                .map_err(recovery)?;
                insert_table_projection_record(&mut tx, &record)
                    .await
                    .map_err(recovery)?;
                (
                    receipt(&request, &history, &transition.event.outcome).map_err(recovery)?,
                    TransportAcceptance::Command {
                        resulting_event_sequence: transition.state.applied_event_sequence,
                    },
                )
            }
            Intent::Observation(text) => {
                let id = ObservationId(meta.id.0);
                let (body, mut observation) =
                    crate::table_runtime::propose_observation(&state, &pack, &meta, id, &text)?;
                observation.payload_schema_version = 2;
                observation.payload_json = json(&TransportedTableObservation {
                    request: request.clone(),
                    body: body.clone(),
                })
                .map_err(rejected)?;
                append_session_observations_in_transaction(
                    &mut tx,
                    meta.campaign_id,
                    &[observation.clone()],
                )
                .await
                .map_err(recovery)?;
                history.observation_ordinal += 1;
                presentation::accepted_observation_visibility(&state, &observation, &mut history);
                export.observations.push(SessionObservation {
                    ordinal: history.observation_ordinal,
                    record: observation,
                });
                let record = presentation::build_record(
                    &state,
                    &pack,
                    &export,
                    &mut history,
                    ProjectionCause::Observation { id },
                    vec![],
                )
                .map_err(recovery)?;
                insert_table_projection_record(&mut tx, &record)
                    .await
                    .map_err(recovery)?;
                (
                    observed(&request, &history, &body).map_err(recovery)?,
                    TransportAcceptance::Observation { id },
                )
            }
        };
        let saved = TableTransportBinding {
            version: 1,
            meta,
            audience: request.channel.audience(),
            projection_ordinal: history.ordinal,
            acceptance,
            request_json: json(&request).map_err(recovery)?,
            response_json: json(&response).map_err(recovery)?,
        };
        insert_table_transport_binding(&mut tx, &saved)
            .await
            .map_err(recovery)?;
        // Verify the whole staged composition before publishing any game or answer.
        let complete = export_campaign_in_transaction(&mut tx, request.campaign_id)
            .await
            .map_err(recovery)?;
        presentation::validate_history(&complete, &pack).map_err(recovery)?;
        tx.commit().await.map_err(recovery)?;
        Ok(response)
    }
}

fn event_row(
    meta: &CommandMeta,
    id: EventId,
    state: &CampaignState,
    event: &TableEvent,
) -> Result<EventJournalRow, String> {
    let (actor_kind, actor_id) = match meta.actor {
        None => (None, None),
        Some(AgentRef::Entity(id)) => (Some("entity".into()), Some(id.0.to_string())),
        Some(AgentRef::Faction(id)) => (Some("faction".into()), Some(id.0.to_string())),
    };
    Ok(EventJournalRow {
        id: id.0.to_string(),
        campaign_id: meta.campaign_id.0.to_string(),
        sequence: state.applied_event_sequence as i64,
        session_id: meta.session_id.map(|id| id.0.to_string()),
        occurred_at_world: state.clock.now.0,
        source: "rule_resolution".into(),
        actor_kind,
        actor_id,
        command_id: meta.id.0.to_string(),
        event_kind: TABLE_EVENT_KIND.into(),
        event_schema_version: i64::from(TABLE_EVENT_VERSION),
        payload_json: json(event)?,
    })
}
