//! Pure audience projection shared by live transport and historical privacy replay.
use crate::table_engine::table;
use crate::table_runtime::{roll_label, sheet_details, validate_table_observation};
use crate::{
    RunnableCampaignError, TABLE_EVENT_KIND, TABLE_EVENT_VERSION, TableAction, TableCharacterView,
    TableEvent, TableRollChannel, TableTranscriptEntry, TableView, TableViewer,
};
use dmd_domain::*;
use dmd_persistence::{EventJournalRow, StoredJournalEvent};
use dmd_rules::{RulesAnswer, RulesPack, RulesQuery};
use std::collections::HashSet;
fn invalid(error: impl ToString) -> RunnableCampaignError {
    RunnableCampaignError::TableRejected(error.to_string())
}
fn recovery(error: impl ToString) -> RunnableCampaignError {
    RunnableCampaignError::Table(error.to_string())
}

/// Borrows either an ordinary legacy image or one exact owned replay image.
/// The owned variant has no state argument that could substitute a forged clone.
#[derive(Clone, Copy)]
pub(crate) enum ProjectionRead<'a> {
    Ordinary {
        state: &'a CampaignState,
        pack: &'a RulesPack,
    },
    Owned(&'a dmd_rules::table::TableRead<'a>),
}
impl<'a> ProjectionRead<'a> {
    pub(crate) fn tactical(self) -> crate::table_tactical::TacticalRead<'a> {
        match self {
            Self::Ordinary { state, .. } => crate::table_tactical::TacticalRead::Ordinary(state),
            Self::Owned(read) => crate::table_tactical::TacticalRead::Owned(read),
        }
    }
    pub(crate) fn state(self) -> &'a CampaignState {
        match self {
            Self::Ordinary { state, .. } => state,
            Self::Owned(read) => read.state(),
        }
    }
    pub(crate) fn pack(self) -> &'a RulesPack {
        match self {
            Self::Ordinary { pack, .. } => pack,
            Self::Owned(read) => read.pack(),
        }
    }
    pub(crate) fn query(
        self,
        issuer: CommandIssuer,
        query: &RulesQuery,
    ) -> Result<RulesAnswer, dmd_rules::RulesError> {
        match self {
            Self::Ordinary { state, pack } => dmd_rules::query(state, issuer, query, pack),
            Self::Owned(read) => read.query(issuer, query),
        }
    }
}

/// Only presentation inputs; no synthetic game envelope or source authority.
pub(crate) struct ProjectionEvent {
    pub id: EventId,
    pub sequence: u64,
    pub kind: String,
    pub version: u32,
    pub json: String,
}
impl From<&StoredJournalEvent> for ProjectionEvent {
    fn from(event: &StoredJournalEvent) -> Self {
        Self {
            id: event.meta.id,
            sequence: event.meta.sequence,
            kind: event.payload.kind.clone(),
            version: event.payload.schema_version,
            json: event.payload.json.clone(),
        }
    }
}
impl TryFrom<&EventJournalRow> for ProjectionEvent {
    type Error = String;
    fn try_from(row: &EventJournalRow) -> Result<Self, String> {
        Ok(Self {
            id: EventId(uuid::Uuid::parse_str(&row.id).map_err(|e| e.to_string())?),
            sequence: u64::try_from(row.sequence).map_err(|e| e.to_string())?,
            kind: row.event_kind.clone(),
            version: u32::try_from(row.event_schema_version).map_err(|e| e.to_string())?,
            json: row.payload_json.clone(),
        })
    }
}

#[cfg(test)]
pub(crate) fn project_table(
    state: &CampaignState,
    viewer: TableViewer,
    pack: &RulesPack,
    events: &[ProjectionEvent],
    observations: &[SessionObservation],
    visible_events: Option<&HashSet<EventId>>,
) -> Result<TableView, RunnableCampaignError> {
    project_table_read(
        ProjectionRead::Ordinary { state, pack },
        viewer,
        events,
        observations,
        visible_events,
    )
}

pub(crate) fn project_table_read(
    read: ProjectionRead<'_>,
    viewer: TableViewer,
    events: &[ProjectionEvent],
    observations: &[SessionObservation],
    visible_events: Option<&HashSet<EventId>>,
) -> Result<TableView, RunnableCampaignError> {
    let state = read.state();
    let mut view =
        project_table_v1_read(read, viewer.clone(), events, observations, visible_events)?;
    if dmd_rules::physical_facts::enabled(state) {
        let ProjectionRead::Owned(owned) = read else {
            return Err(invalid("Physical facts require original table history."));
        };
        view.physical = Some(crate::table_physical::view(owned, &viewer).map_err(invalid)?);
    }
    if dmd_rules::table::grapple_enabled(state) {
        let ProjectionRead::Owned(owned) = read else {
            return Err(invalid(
                "Grapple presentation requires original table history.",
            ));
        };
        view.grapple = Some(crate::table_tactical::grapple_view(owned, &viewer).map_err(invalid)?);
    }
    if crate::table_source_control::enabled(state) {
        view.source_control = Some(crate::TableSourceControlView {
            version: 2,
            actors: crate::table_source_control::visible_actors(state, &viewer).map_err(invalid)?,
        });
        view.tactical =
            crate::table_tactical::view_read(read.tactical(), &viewer, true, read.pack())
                .map_err(invalid)?;
        // Only source-owned public tactical requests gain the new table visibility.
        // Legacy PC kernel queries (including source-inappropriate sheet formulas) stay frozen.
        if let TableViewer::Player(player) = viewer
            && let Some(pending) = state
                .rules
                .as_ref()
                .and_then(|rules| rules.pending.as_ref())
            && pending.request.visibility == RollVisibility::Public
            && matches!(
                pending.purpose,
                PendingPurpose::TacticalInitiative { .. }
                    | PendingPurpose::TacticalResolution { .. }
            )
            && pending
                .request
                .roller
                .is_some_and(|actor| crate::table_source_control::owns_source(state, player, actor))
        {
            let mut request = pending.request.clone();
            request.reason = roll_label(&pending.purpose, state);
            view.roll = Some(request);
            view.roll_channel = Some(TableRollChannel::Tactical);
        }
    }
    if let Some(transfer) = state
        .table
        .as_ref()
        .and_then(|table| table.inspiration_transfer.as_ref())
    {
        let issuer = match viewer {
            TableViewer::Host => CommandIssuer::Admin,
            TableViewer::Player(player) => CommandIssuer::Player(player),
        };
        let choices =
            dmd_rules::table::inspiration_transfer_choices(state, issuer).map_err(invalid)?;
        if matches!(viewer, TableViewer::Host) || !choices.is_empty() {
            view.inspiration_transfer = Some(crate::TableInspirationTransferView {
                character_id: transfer.character_id,
                choices: choices
                    .into_iter()
                    .map(|choice| crate::TableInspirationOption {
                        label: choice.recipient.map_or_else(
                            || "Decline the extra Inspiration".into(),
                            |id| format!("Give to {}", state.characters[&id].display_name),
                        ),
                        key: choice,
                    })
                    .collect(),
            });
        }
    }
    Ok(view)
}

/// Historical version-one projection, including PC-only ownership, is immutable.
#[cfg(test)]
pub(crate) fn project_table_v1(
    state: &CampaignState,
    viewer: TableViewer,
    pack: &RulesPack,
    events: &[ProjectionEvent],
    observations: &[SessionObservation],
    visible_events: Option<&HashSet<EventId>>,
) -> Result<TableView, RunnableCampaignError> {
    project_table_v1_read(
        ProjectionRead::Ordinary { state, pack },
        viewer,
        events,
        observations,
        visible_events,
    )
}

fn project_table_v1_read(
    read: ProjectionRead<'_>,
    viewer: TableViewer,
    events: &[ProjectionEvent],
    observations: &[SessionObservation],
    visible_events: Option<&HashSet<EventId>>,
) -> Result<TableView, RunnableCampaignError> {
    let state = read.state();
    let pack = read.pack();
    let campaign_id = state.campaign_id();
    let table = table(state).map_err(invalid)?;
    if let TableViewer::Player(player) = viewer
        && !state.players.contains_key(&player)
    {
        return Err(invalid("Unknown player view."));
    }

    let issuer = match viewer {
        TableViewer::Host => CommandIssuer::Admin,
        TableViewer::Player(player) => CommandIssuer::Player(player),
    };
    let mut players = state.players.values().cloned().collect::<Vec<_>>();
    players.sort_by_key(|player| player.id.0);
    let mut characters = Vec::new();
    for character in state.characters.values() {
        let may_see = matches!(viewer, TableViewer::Host)
            || matches!(viewer,TableViewer::Player(player) if character.controlling_player_id==Some(player));
        let sheet = if may_see
            && state
                .rules
                .as_ref()
                .is_some_and(|rules| rules.entities.contains_key(&character.entity_id))
        {
            Some(read.query(
                issuer,
                &RulesQuery::Character {
                    actor: character.entity_id,
                },
            )?)
        } else {
            None
        };
        characters.push(TableCharacterView {
            character_id: character.id,
            player_id: character.controlling_player_id,
            entity_id: character.entity_id,
            name: character.display_name.clone(),
            profile: may_see
                .then(|| table.character_profiles.get(&character.id).cloned())
                .flatten(),
            sheet,
            equipment: if may_see && table.character_profiles.contains_key(&character.id) {
                Some(
                    crate::table_equipment::view(
                        state,
                        character.id,
                        pack,
                        matches!(viewer, TableViewer::Host),
                    )
                    .map_err(invalid)?,
                )
            } else {
                None
            },
            second_wind_remaining: if may_see {
                state
                    .rules
                    .as_ref()
                    .and_then(|r| r.entities.get(&character.entity_id))
                    .and_then(|e| e.character_features.as_ref())
                    .map(|f| f.second_wind_remaining)
            } else {
                None
            },
            details: if may_see {
                state.rules.as_ref().and_then(|rules| {
                    rules
                        .entities
                        .get(&character.entity_id)
                        .map(|entity| sheet_details(rules, entity))
                })
            } else {
                None
            },
        });
    }
    characters.sort_by_key(|character| character.character_id.0);
    let mut roll_channel = None;
    let roll = if state.rules.is_some() {
        match read.query(issuer, &RulesQuery::PendingRoll)? {
            RulesAnswer::PendingRoll(Some(mut request)) => {
                let pending = state
                    .rules
                    .as_ref()
                    .and_then(|rules| rules.pending.as_ref())
                    .filter(|pending| pending.request.id == request.id)
                    .ok_or_else(|| recovery("Visible roll has no matching pending purpose."))?;
                // Presentation only: leave the persisted request and its replay inputs intact.
                request.reason = roll_label(&pending.purpose, state);
                roll_channel = Some(match pending.purpose {
                    PendingPurpose::TacticalInitiative { .. }
                    | PendingPurpose::TacticalResolution { .. } => TableRollChannel::Tactical,
                    _ => TableRollChannel::Table,
                });
                Some(request)
            }
            _ => None,
        }
    } else {
        None
    };
    let pending = table
        .pending
        .as_ref()
        .filter(|pending| {
            matches!(viewer, TableViewer::Host)
                || matches!(viewer,TableViewer::Player(player) if pending.player_id==player)
        })
        .cloned();
    let mut transcript = Vec::new();
    let mut recap = Vec::new();
    for event in events {
        if event.sequence > state.applied_event_sequence {
            break;
        }
        if event.kind != TABLE_EVENT_KIND {
            continue;
        }
        if event.version != TABLE_EVENT_VERSION {
            return Err(recovery("Unsupported table transcript event version."));
        }
        if !matches!(viewer, TableViewer::Host)
            && visible_events.is_some_and(|ids| !ids.contains(&event.id))
        {
            continue;
        }
        let event_body: TableEvent = serde_json::from_str(&event.json).map_err(recovery)?;
        let (kind, text) = match &event_body.action {
            TableAction::Declare { text } => (
                "declaration",
                format!("Proposed: {text}\n{}", event_body.outcome.message),
            ),
            TableAction::Correct { text, .. } => (
                "correction",
                format!("Correction: {text}\n{}", event_body.outcome.message),
            ),
            TableAction::SubmitPhysical { .. } => ("outcome", event_body.outcome.message.clone()),
            _ => ("table", event_body.outcome.message.clone()),
        };
        if kind == "outcome" {
            recap.push(text.clone());
        }
        let speaker = match event_body.meta.issuer {
            CommandIssuer::Player(player) => state
                .players
                .get(&player)
                .map_or("Player", |p| p.display_name.as_str())
                .to_owned(),
            _ => "Table".into(),
        };
        transcript.push((
            event.sequence,
            0,
            TableTranscriptEntry {
                id: event.id.0.to_string(),
                event_sequence: event.sequence,
                kind: kind.into(),
                speaker,
                text,
            },
        ));
    }
    for observation in observations {
        let record = &observation.record;
        let visible = matches!(record.audience, ObservationAudience::Party)
            || matches!(viewer, TableViewer::Host)
            || matches!((&viewer,&record.audience),(TableViewer::Player(a),ObservationAudience::Player(b)) if a==b);
        if !visible || record.observed_event_sequence > state.applied_event_sequence {
            continue;
        }
        if record.kind != "table.conversation" {
            continue;
        }
        let body = validate_table_observation(record).map_err(recovery)?;
        let speaker = match record.issuer {
            CommandIssuer::Player(player) => state
                .players
                .get(&player)
                .map_or("Player", |p| p.display_name.as_str())
                .to_owned(),
            _ => "Table".into(),
        };
        transcript.push((
            record.observed_event_sequence,
            observation.ordinal,
            TableTranscriptEntry {
                id: record.id.0.to_string(),
                event_sequence: record.observed_event_sequence,
                kind: "conversation".into(),
                speaker,
                text: format!("{}\n{}", body.text, body.answer),
            },
        ));
    }
    transcript.sort_by_key(|(sequence, ordinal, _)| (*sequence, *ordinal));
    let start = transcript.len().saturating_sub(500);
    let transcript = transcript
        .into_iter()
        .skip(start)
        .map(|(_, _, entry)| entry)
        .collect();
    let recap = recap
        .into_iter()
        .rev()
        .take(8)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    Ok(TableView {
        inspiration_transfer: None,
        physical: None,
        grapple: None,
        source_control: None,
        campaign_id,
        name: state.campaign.display_name.clone(),
        event_sequence: state.applied_event_sequence,
        contract: table.contract.clone(),
        players,
        characters,
        active_session: table.active_session.clone(),
        pending,
        roll,
        roll_channel,
        tactical: crate::table_tactical::view_read(read.tactical(), &viewer, false, pack)
            .map_err(invalid)?,
        creature_setup: crate::table_creatures::view(state, matches!(viewer, TableViewer::Host))
            .map_err(invalid)?,
        situation_title: table.situation.title.clone(),
        situation_description: table.situation.description.clone(),
        transcript,
        recap,
    })
}
