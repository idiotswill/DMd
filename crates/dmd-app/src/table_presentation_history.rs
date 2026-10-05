//! Historical audience evidence derives from the same authenticated semantic replay
//! used by restore. Opaque identifiers never become gameplay authority.
use crate::{
    TABLE_EVENT_KIND, TablePresentedView, TableView, TableViewer,
    table_projection::{ProjectionEvent, ProjectionRead, project_table_read},
    table_transport::presented_view,
};
use dmd_domain::*;
use dmd_persistence::*;
use dmd_rules::RulesPack;
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

#[derive(Default)]
pub(crate) struct PresentationHistory {
    pub latest: HashMap<ProjectionAudience, ProjectionChange>,
    pub visibility: Vec<TranscriptVisibility>,
    pub ordinal: u64,
    pub observation_ordinal: u64,
    observation_audiences: HashMap<ObservationId, Vec<ProjectionAudience>>,
}

fn audiences(state: &CampaignState) -> Vec<ProjectionAudience> {
    let mut players = state.players.keys().copied().collect::<Vec<_>>();
    players.sort_by_key(|id| id.0);
    std::iter::once(ProjectionAudience::Host)
        .chain(players.into_iter().map(ProjectionAudience::Player))
        .collect()
}
pub(crate) fn viewer(audience: ProjectionAudience) -> TableViewer {
    match audience {
        ProjectionAudience::Host => TableViewer::Host,
        ProjectionAudience::Player(id) => TableViewer::Player(id),
    }
}
pub(crate) fn work_origin(state: &CampaignState) -> Option<CommandId> {
    state
        .encounter
        .as_ref()?
        .flow
        .as_ref()?
        .resolution
        .as_ref()
        .map(|r| r.origin.id)
}
fn capabilities(
    raw: &TableView,
    state: &CampaignState,
) -> Result<Vec<ProjectionCapability>, String> {
    let mut result = Vec::new();
    if let Some(grapple) = &raw.grapple {
        result.extend(
            grapple
                .choices
                .iter()
                .map(|choice| ProjectionCapability::GrappleChoice {
                    offer: choice.key.clone(),
                }),
        );
    }
    if let Some(shove) = raw.tactical.as_ref().and_then(|t| t.shove.as_ref()) {
        result.push(ProjectionCapability::ShoveDecision {
            origin: shove.key.resolution,
            occurrence: shove.key.occurrence,
            stage: shove.stage,
        });
    }
    if let Some(roll) = &raw.roll {
        result.push(ProjectionCapability::Roll { canonical: roll.id });
    }
    if let Some(continuation) = raw.tactical.as_ref().and_then(|t| t.continuation.as_ref()) {
        for choice in &continuation.choices {
            result.push(ProjectionCapability::Work {
                origin: work_origin(state).ok_or("work without resolution")?,
                occurrence: choice.occurrence,
            });
        }
    }
    if let Some(hit) = raw.tactical.as_ref().and_then(|t| t.hit.as_deref()) {
        let mut add = |key: TacticalWorkKey, role| {
            result.push(ProjectionCapability::HitResponse {
                origin: key.resolution,
                occurrence: key.occurrence,
                role,
            })
        };
        if let Some(order) = &hit.order {
            add(order.key, ProjectionHitRole::Order);
        }
        if let Some(key) = hit.delegate {
            add(key, ProjectionHitRole::Delegate);
        }
        if let Some(response) = &hit.response {
            add(
                response.key,
                if response.selected {
                    ProjectionHitRole::Selected
                } else {
                    ProjectionHitRole::Intent
                },
            );
        }
    }
    if let Some(missile) = raw.tactical.as_ref().and_then(|t| t.missile.as_deref()) {
        let mut add = |key: TacticalWorkKey, role| {
            result.push(ProjectionCapability::MissileResponse {
                origin: key.resolution,
                occurrence: key.occurrence,
                role,
            })
        };
        if let Some(order) = &missile.order {
            add(order.key, ProjectionMissileRole::Order);
        }
        if let Some(key) = missile.delegate {
            add(key, ProjectionMissileRole::Delegate);
        }
        for response in &missile.responses {
            add(
                response.key,
                if response.selected {
                    ProjectionMissileRole::Selected {
                        actor: response.actor,
                    }
                } else {
                    ProjectionMissileRole::Intent {
                        actor: response.actor,
                    }
                },
            );
        }
    }
    Ok(result)
}
fn digest(raw: TableView, state: &CampaignState) -> Result<String, String> {
    // Fixed presentation-local placeholders erase all hidden canonical occurrence
    // encodings. Capability replacement is checked separately as a new visible prompt.
    let handles = capabilities(&raw, state)?
        .into_iter()
        .enumerate()
        .map(|(i, capability)| ProjectionHandle {
            opaque: Uuid::from_u128(i as u128 + 1),
            capability,
        })
        .collect::<Vec<_>>();
    let mut value = presented_view(
        raw,
        ProjectionAudience::Host,
        ProjectionRevision(Uuid::nil()),
        &handles,
        work_origin(state),
    )?;
    value.diagnostics = None;
    let bytes = serde_json::to_vec(&value).map_err(|e| e.to_string())?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}
fn visible_ids(history: &PresentationHistory, audience: ProjectionAudience) -> HashSet<EventId> {
    history
        .visibility
        .iter()
        .filter(|v| v.audiences.contains(&audience))
        .map(|v| v.event_id)
        .collect()
}
pub(crate) fn raw_view(
    read: ProjectionRead<'_>,
    events: &[ProjectionEvent],
    observations: &[SessionObservation],
    history: &PresentationHistory,
    audience: ProjectionAudience,
) -> Result<TableView, String> {
    let visible = visible_ids(history, audience);
    let observations = observations
        .iter()
        .filter(|o| {
            o.ordinal <= history.observation_ordinal
                && history
                    .observation_audiences
                    .get(&o.record.id)
                    .is_some_and(|a| a.contains(&audience))
        })
        .cloned()
        .collect::<Vec<_>>();
    project_table_read(
        read,
        viewer(audience),
        events,
        &observations,
        Some(&visible),
    )
    .map_err(|e| e.to_string())
}

pub(crate) fn current_view_read(
    read: ProjectionRead<'_>,
    export: &CampaignExport,
    history: &PresentationHistory,
    audience: ProjectionAudience,
) -> Result<TablePresentedView, String> {
    let state = read.state();
    let events = export
        .event_journal
        .iter()
        .map(ProjectionEvent::try_from)
        .collect::<Result<Vec<_>, _>>()?;
    let latest = history
        .latest
        .get(&audience)
        .ok_or("unknown presentation audience")?;
    presented_view(
        raw_view(read, &events, &export.observations, history, audience)?,
        audience,
        latest.revision,
        &latest.handles,
        work_origin(state),
    )
}

/// A gameplay event is published to an audience only when its acceptance changes
/// that audience's presented game state. This cannot be recomputed using later sight.
fn classify(
    before_read: ProjectionRead<'_>,
    after_read: ProjectionRead<'_>,
    event: &EventJournalRow,
) -> Result<Option<TranscriptVisibility>, String> {
    let before = before_read.state();
    let after = after_read.state();
    if event.event_kind != TABLE_EVENT_KIND {
        return Ok(None);
    }
    let body: crate::TableEvent =
        serde_json::from_str(&event.payload_json).map_err(|e| e.to_string())?;
    if matches!(
        body.action,
        crate::TableAction::Declare { .. }
            | crate::TableAction::Correct { .. }
            | crate::TableAction::CancelDecision { .. }
    ) {
        // Spoken proposals/corrections are shared table utterances independently
        // of which player owns the private adjudication controls.
        return Ok(Some(TranscriptVisibility {
            event_id: EventId(Uuid::parse_str(&event.id).map_err(|e| e.to_string())?),
            audiences: audiences(after),
        }));
    }
    let mut visible = vec![ProjectionAudience::Host];
    for audience in audiences(after).into_iter().skip(1) {
        let ProjectionAudience::Player(id) = audience else {
            unreachable!()
        };
        let new = project_table_read(
            after_read,
            viewer(audience),
            &[],
            &[],
            Some(&HashSet::new()),
        )
        .map_err(|e| e.to_string())?;
        let changed = if before.players.contains_key(&id) {
            let old = project_table_read(
                before_read,
                viewer(audience),
                &[],
                &[],
                Some(&HashSet::new()),
            )
            .map_err(|e| e.to_string())?;
            capabilities(&old, before)? != capabilities(&new, after)?
                || digest(old, before)? != digest(new, after)?
        } else {
            true
        };
        if changed {
            visible.push(audience);
        }
    }
    Ok(Some(TranscriptVisibility {
        event_id: EventId(Uuid::parse_str(&event.id).map_err(|e| e.to_string())?),
        audiences: visible,
    }))
}

fn expected_changes(
    read: ProjectionRead<'_>,
    events: &[ProjectionEvent],
    observations: &[SessionObservation],
    history: &PresentationHistory,
) -> Result<Vec<(ProjectionAudience, String, Vec<ProjectionCapability>)>, String> {
    let state = read.state();
    let mut result = Vec::new();
    for audience in audiences(state) {
        let raw = raw_view(read, events, observations, history, audience)?;
        let capabilities = capabilities(&raw, state)?;
        let digest = digest(raw, state)?;
        let unchanged = history.latest.get(&audience).is_some_and(|old| {
            old.visible_digest == digest
                && old
                    .handles
                    .iter()
                    .map(|h| &h.capability)
                    .eq(capabilities.iter())
        });
        if !unchanged {
            result.push((audience, digest, capabilities));
        }
    }
    Ok(result)
}
fn validate_record(
    read: ProjectionRead<'_>,
    events: &[ProjectionEvent],
    observations: &[SessionObservation],
    history: &mut PresentationHistory,
    record: &TableProjectionRecord,
    transcript: &[TranscriptVisibility],
) -> Result<(), String> {
    let state = read.state();
    if record.version != crate::table_source_control::presentation_version(state)
        || record.ordinal != history.ordinal + 1
        || record.transcript != transcript
    {
        return Err("presentation history/cause visibility mismatch".into());
    }
    let expected = expected_changes(read, events, observations, history)?;
    if record.changes.len() != expected.len() {
        return Err("presentation changed-audience set mismatch".into());
    }
    for (change, (audience, digest, capabilities)) in record.changes.iter().zip(expected) {
        let prior = history.latest.get(&audience);
        if change.audience != audience
            || change.previous != prior.map(|p| p.revision)
            || change.visible_digest != digest
            || !change
                .handles
                .iter()
                .map(|h| &h.capability)
                .eq(capabilities.iter())
        {
            return Err("presentation revision does not match its historical audience".into());
        }
        // Retain handles for prompts that survived a different visible change. A
        // fresh prompt gets a new opaque handle, never another audience's handle.
        for handle in &change.handles {
            if let Some(old) =
                prior.and_then(|p| p.handles.iter().find(|h| h.capability == handle.capability))
                && old.opaque != handle.opaque
            {
                return Err("surviving presentation capability changed identity".into());
            }
        }
        history.latest.insert(audience, change.clone());
    }
    history.ordinal = record.ordinal;
    Ok(())
}

pub(crate) fn build_record(
    state: &CampaignState,
    pack: &RulesPack,
    export: &CampaignExport,
    history: &mut PresentationHistory,
    cause: ProjectionCause,
    transcript: Vec<TranscriptVisibility>,
) -> Result<TableProjectionRecord, String> {
    build_record_read(
        ProjectionRead::Ordinary { state, pack },
        export,
        history,
        cause,
        transcript,
    )
}

pub(crate) fn build_record_read(
    read: ProjectionRead<'_>,
    export: &CampaignExport,
    history: &mut PresentationHistory,
    cause: ProjectionCause,
    transcript: Vec<TranscriptVisibility>,
) -> Result<TableProjectionRecord, String> {
    let state = read.state();
    let events = export
        .event_journal
        .iter()
        .map(ProjectionEvent::try_from)
        .collect::<Result<Vec<_>, _>>()?;
    let mut changes = Vec::new();
    for (audience, visible_digest, capabilities) in
        expected_changes(read, &events, &export.observations, history)?
    {
        let prior = history.latest.get(&audience);
        let handles = capabilities
            .into_iter()
            .map(|capability| {
                let opaque = prior
                    .and_then(|p| p.handles.iter().find(|h| h.capability == capability))
                    .map_or_else(Uuid::new_v4, |h| h.opaque);
                ProjectionHandle { opaque, capability }
            })
            .collect();
        changes.push(ProjectionChange {
            audience,
            previous: prior.map(|p| p.revision),
            revision: ProjectionRevision(Uuid::new_v4()),
            visible_digest,
            handles,
        });
    }
    let record = TableProjectionRecord {
        version: crate::table_source_control::presentation_version(state),
        campaign_id: state.campaign_id(),
        ordinal: history.ordinal + 1,
        cause,
        transcript,
        changes,
    };
    for change in &record.changes {
        history.latest.insert(change.audience, change.clone());
    }
    history.ordinal = record.ordinal;
    Ok(record)
}

pub(crate) fn accepted_event_visibility(
    before: &CampaignState,
    after: &CampaignState,
    row: &EventJournalRow,
    pack: &RulesPack,
    history: &mut PresentationHistory,
) -> Result<Vec<TranscriptVisibility>, String> {
    accepted_event_visibility_read(
        ProjectionRead::Ordinary {
            state: before,
            pack,
        },
        ProjectionRead::Ordinary { state: after, pack },
        row,
        history,
    )
}

pub(crate) fn accepted_event_visibility_read(
    before: ProjectionRead<'_>,
    after: ProjectionRead<'_>,
    row: &EventJournalRow,
    history: &mut PresentationHistory,
) -> Result<Vec<TranscriptVisibility>, String> {
    let result = classify(before, after, row)?
        .into_iter()
        .collect::<Vec<_>>();
    history.visibility.extend(result.clone());
    Ok(result)
}

pub(crate) fn accepted_observation_visibility(
    state: &CampaignState,
    record: &NewSessionObservation,
    history: &mut PresentationHistory,
) {
    let visible = audiences(state)
        .into_iter()
        .filter(|audience| match (audience, &record.audience) {
            (ProjectionAudience::Host, _) => true,
            (_, ObservationAudience::Party) => true,
            (ProjectionAudience::Player(a), ObservationAudience::Player(b)) => a == b,
            _ => false,
        })
        .collect();
    history.observation_audiences.insert(record.id, visible);
}

/// Empty legacy history is permitted only without any canonical protocol marker.
/// The first durable bootstrap records all authenticated historical visibility.
pub(crate) fn validate_history(
    export: &CampaignExport,
    pack: &RulesPack,
) -> Result<PresentationHistory, String> {
    crate::rules_restore::authenticate_history(export, pack.clone())
        .map(|verified| verified.into_parts().1)
}

pub(crate) struct LegacySelectionCheck {
    pub meta: CommandMeta,
    pub channel: crate::TableTransportChannel,
    matched: bool,
}
impl LegacySelectionCheck {
    pub(crate) fn new(meta: CommandMeta, channel: crate::TableTransportChannel) -> Self {
        Self {
            meta,
            channel,
            matched: false,
        }
    }
    fn observe(&mut self, state: &CampaignState) {
        if state.applied_event_sequence != self.meta.expected_event_sequence {
            return;
        }
        self.matched = match self.channel {
            crate::TableTransportChannel::SourceCreature { .. } => false,
            crate::TableTransportChannel::Host => {
                self.meta.issuer == CommandIssuer::Admin && self.meta.actor.is_none()
            }
            crate::TableTransportChannel::Player {
                player_id,
                character_id,
            } => {
                self.meta.issuer == CommandIssuer::Player(player_id)
                    && state.characters.get(&character_id).is_some_and(|c| {
                        c.controlling_player_id == Some(player_id)
                            && self.meta.actor == Some(AgentRef::Entity(c.entity_id))
                    })
            }
        };
    }
}

pub(crate) struct HistoryVerifier<'a> {
    export: &'a CampaignExport,
    history: PresentationHistory,
    events: Vec<ProjectionEvent>,
    bootstrap_head: u64,
    bootstrap_observation: u64,
    next_record: usize,
    bootstrapped: bool,
    table: bool,
    selection: Option<LegacySelectionCheck>,
}
impl<'a> HistoryVerifier<'a> {
    pub(crate) fn new(
        export: &'a CampaignExport,
        selection: Option<LegacySelectionCheck>,
    ) -> Result<Self, String> {
        let current = CampaignState::decode_json(&export.current_state.state_json)
            .map_err(|e| e.to_string())?;
        let marked = export.command_audit.iter().any(|a| {
            a.command_kind == "table.action" && matches!(a.command_schema_version, 2 | 3 | 4)
        }) || export.observations.iter().any(|o| {
            o.record.kind == "table.conversation"
                && matches!(o.record.payload_schema_version, 2 | 3 | 4)
        }) || crate::table_source_control::enabled(&current)
            || dmd_rules::table::grapple_enabled(&current);
        if current.table.is_none() {
            if !export.table_projection_history.is_empty()
                || !export.table_transport_bindings.is_empty()
                || marked
            {
                return Err("presentation requires a table history".into());
            }
        }
        let bootstrap = export.table_projection_history.first();
        if bootstrap.is_none() && (marked || !export.table_transport_bindings.is_empty()) {
            return Err("canonical protocol acceptance is missing presentation history".into());
        }
        let (bootstrap_head, bootstrap_observation) = match bootstrap.map(|b| &b.cause) {
            Some(ProjectionCause::Bootstrap {
                event_sequence,
                observation_ordinal,
            }) => (*event_sequence, *observation_ordinal),
            None => (
                current.applied_event_sequence,
                export.observations.len() as u64,
            ),
            _ => return Err("presentation lacks its initial bootstrap".into()),
        };
        let events = export
            .event_journal
            .iter()
            .map(ProjectionEvent::try_from)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            export,
            history: PresentationHistory::default(),
            events,
            bootstrap_head,
            bootstrap_observation,
            next_record: 1,
            bootstrapped: false,
            table: current.table.is_some(),
            selection,
        })
    }
    pub(crate) fn anchor(&mut self, read: dmd_rules::table::TableRead<'_>) -> Result<(), String> {
        self.frame(None, read, None)
    }
    pub(crate) fn step(
        &mut self,
        before: dmd_rules::table::TableRead<'_>,
        after: dmd_rules::table::TableRead<'_>,
        row: &EventJournalRow,
    ) -> Result<(), String> {
        self.frame(Some(before), after, Some(row))
    }
    fn frame(
        &mut self,
        prior_read: Option<dmd_rules::table::TableRead<'_>>,
        read: dmd_rules::table::TableRead<'_>,
        row: Option<&EventJournalRow>,
    ) -> Result<(), String> {
        let state = read.state();
        if let Some(selection) = &mut self.selection {
            selection.observe(state);
        }
        if !self.table {
            return Ok(());
        }
        let before = prior_read.as_ref().map(|read| read.state());
        let projection = ProjectionRead::Owned(&read);
        let export = self.export;
        let bootstrap = export.table_projection_history.first();
        let events = &self.events;

        if state.table.is_none() {
            return Err("table presentation requires its original table anchor".into());
        }
        for observation in &export.observations {
            if observation.record.observed_event_sequence == state.applied_event_sequence {
                accepted_observation_visibility(state, &observation.record, &mut self.history);
            } else if before.is_none()
                && observation.record.observed_event_sequence < state.applied_event_sequence
            {
                self.history
                    .observation_audiences
                    .insert(observation.record.id, vec![ProjectionAudience::Host]);
            }
        }
        let event_visibility = if let (Some(_), Some(row)) = (before, row) {
            accepted_event_visibility_read(
                ProjectionRead::Owned(prior_read.as_ref().expect("prior image")),
                projection,
                row,
                &mut self.history,
            )?
        } else {
            // No knowledge is invented for an opaque pre-table anchor. Such old
            // entries remain host diagnostics and cannot become player transcript.
            self.history.visibility.extend(
                export
                    .event_journal
                    .iter()
                    .filter(|e| {
                        e.sequence <= state.applied_event_sequence as i64
                            && e.event_kind == TABLE_EVENT_KIND
                    })
                    .map(|e| {
                        Ok(TranscriptVisibility {
                            event_id: EventId(Uuid::parse_str(&e.id).map_err(|e| e.to_string())?),
                            audiences: vec![ProjectionAudience::Host],
                        })
                    })
                    .collect::<Result<Vec<_>, String>>()?,
            );
            Vec::new()
        };
        if state.applied_event_sequence < self.bootstrap_head {
            return Ok(());
        }
        if !self.bootstrapped {
            if state.applied_event_sequence != self.bootstrap_head {
                return Err("bootstrap is before the authenticated anchor".into());
            }
            // A new acceptance can never precede initialization of its protocol.
            if export.command_audit.iter().any(|a| {
                matches!(a.command_schema_version, 2 | 3 | 4)
                    && a.command_kind == "table.action"
                    && a.resulting_event_sequence <= self.bootstrap_head as i64
            }) || export.observations.iter().any(|o| {
                o.ordinal <= self.bootstrap_observation
                    && o.record.kind == "table.conversation"
                    && matches!(o.record.payload_schema_version, 2 | 3 | 4)
            }) {
                return Err("protocol marker precedes its bootstrap".into());
            }
            self.history.observation_ordinal = self.bootstrap_observation;
            if let Some(record) = bootstrap {
                let transcript = self.history.visibility.clone();
                validate_record(
                    projection,
                    events,
                    &export.observations,
                    &mut self.history,
                    record,
                    &transcript,
                )?;
            }
            self.bootstrapped = true;
        } else if let Some(row) = row {
            let record = export
                .table_projection_history
                .get(self.next_record)
                .ok_or("missing accepted event presentation")?;
            let cause = ProjectionCause::Event {
                id: EventId(Uuid::parse_str(&row.id).map_err(|e| e.to_string())?),
                command_id: CommandId(Uuid::parse_str(&row.command_id).map_err(|e| e.to_string())?),
            };
            if record.cause != cause {
                return Err("presentation event order mismatch".into());
            }
            // Transport request/response semantics are authenticated separately at
            // this exact before/after image, before advancing the stored revision.
            let prior = self.history.latest.clone();
            validate_record(
                projection,
                events,
                &export.observations,
                &mut self.history,
                record,
                &event_visibility,
            )?;
            crate::table_transport_runtime::validate_event_binding(
                export,
                before.ok_or("missing prior image")?,
                state,
                row,
                &prior,
                &self.history,
                read.pack(),
            )?;
            self.next_record += 1;
        }
        while let Some(record) = export.table_projection_history.get(self.next_record) {
            let ProjectionCause::Observation { id } = record.cause else {
                break;
            };
            let observation = export
                .observations
                .iter()
                .find(|o| o.record.id == id)
                .ok_or("presentation observation absent")?;
            if observation.record.observed_event_sequence != state.applied_event_sequence {
                break;
            }
            if observation.ordinal != self.history.observation_ordinal + 1 {
                return Err("presentation observation order mismatch".into());
            }
            let prior = self.history.latest.clone();
            self.history.observation_ordinal = observation.ordinal;
            validate_record(
                projection,
                events,
                &export.observations,
                &mut self.history,
                record,
                &[],
            )?;
            crate::table_transport_runtime::validate_observation_binding(
                export,
                projection,
                observation,
                &prior,
                &self.history,
            )?;
            self.next_record += 1;
        }
        Ok(())
    }
    pub(crate) fn finish(mut self) -> Result<(PresentationHistory, Option<bool>), String> {
        let export = self.export;
        let bootstrap = export.table_projection_history.first();
        if self.table {
            if !self.bootstrapped {
                return Err("presentation bootstrap image unavailable".into());
            }
            if bootstrap.is_some() && self.next_record != export.table_projection_history.len() {
                return Err("unconsumed presentation evidence".into());
            }
            if bootstrap.is_none() {
                self.history.observation_ordinal = export.observations.len() as u64;
            }
        }
        Ok((
            self.history,
            self.selection.map(|selection| selection.matched),
        ))
    }
}
