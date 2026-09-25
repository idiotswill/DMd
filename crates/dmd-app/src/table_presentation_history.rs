//! Historical audience evidence derives from the same authenticated semantic replay
//! used by restore. Opaque identifiers never become gameplay authority.
use crate::{
    TABLE_EVENT_KIND, TablePresentedView, TableView, TableViewer,
    table_projection::{ProjectionEvent, project_table},
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
fn raw_view(
    state: &CampaignState,
    pack: &RulesPack,
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
    project_table(
        state,
        viewer(audience),
        pack,
        events,
        &observations,
        Some(&visible),
    )
    .map_err(|e| e.to_string())
}

pub(crate) fn current_view(
    state: &CampaignState,
    pack: &RulesPack,
    export: &CampaignExport,
    history: &PresentationHistory,
    audience: ProjectionAudience,
) -> Result<TablePresentedView, String> {
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
        raw_view(
            state,
            pack,
            &events,
            &export.observations,
            history,
            audience,
        )?,
        audience,
        latest.revision,
        &latest.handles,
        work_origin(state),
    )
}

/// A gameplay event is published to an audience only when its acceptance changes
/// that audience's presented game state. This cannot be recomputed using later sight.
fn classify(
    before: &CampaignState,
    after: &CampaignState,
    event: &EventJournalRow,
    pack: &RulesPack,
) -> Result<Option<TranscriptVisibility>, String> {
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
        let new = project_table(
            after,
            viewer(audience),
            pack,
            &[],
            &[],
            Some(&HashSet::new()),
        )
        .map_err(|e| e.to_string())?;
        let changed = if before.players.contains_key(&id) {
            let old = project_table(
                before,
                viewer(audience),
                pack,
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
    state: &CampaignState,
    pack: &RulesPack,
    events: &[ProjectionEvent],
    observations: &[SessionObservation],
    history: &PresentationHistory,
) -> Result<Vec<(ProjectionAudience, String, Vec<ProjectionCapability>)>, String> {
    let mut result = Vec::new();
    for audience in audiences(state) {
        let raw = raw_view(state, pack, events, observations, history, audience)?;
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
    state: &CampaignState,
    pack: &RulesPack,
    events: &[ProjectionEvent],
    observations: &[SessionObservation],
    history: &mut PresentationHistory,
    record: &TableProjectionRecord,
    transcript: &[TranscriptVisibility],
) -> Result<(), String> {
    if record.version != crate::table_source_control::presentation_version(state)
        || record.ordinal != history.ordinal + 1
        || record.transcript != transcript
    {
        return Err("presentation history/cause visibility mismatch".into());
    }
    let expected = expected_changes(state, pack, events, observations, history)?;
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
    let events = export
        .event_journal
        .iter()
        .map(ProjectionEvent::try_from)
        .collect::<Result<Vec<_>, _>>()?;
    let mut changes = Vec::new();
    for (audience, visible_digest, capabilities) in
        expected_changes(state, pack, &events, &export.observations, history)?
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
    let result = classify(before, after, row, pack)?
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
    let mut history = PresentationHistory::default();
    let current =
        CampaignState::decode_json(&export.current_state.state_json).map_err(|e| e.to_string())?;
    let marked = export
        .command_audit
        .iter()
        .any(|a| a.command_kind == "table.action" && matches!(a.command_schema_version, 2 | 3))
        || export.observations.iter().any(|o| {
            o.record.kind == "table.conversation"
                && matches!(o.record.payload_schema_version, 2 | 3)
        })
        || crate::table_source_control::enabled(&current);
    if current.table.is_none() {
        if !export.table_projection_history.is_empty()
            || !export.table_transport_bindings.is_empty()
            || marked
        {
            return Err("presentation requires a table history".into());
        }
        crate::rules_restore::visit_rules_history(export, pack, |_, _, _| Ok(()))?;
        return Ok(history);
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
    let mut next_record = 1usize;
    let mut bootstrapped = false;
    crate::rules_restore::visit_rules_history(export, pack, |before, state, row| {
        if state.table.is_none() {
            return Err("table presentation requires its original table anchor".into());
        }
        for observation in &export.observations {
            if observation.record.observed_event_sequence == state.applied_event_sequence {
                accepted_observation_visibility(state, &observation.record, &mut history);
            } else if before.is_none()
                && observation.record.observed_event_sequence < state.applied_event_sequence
            {
                history
                    .observation_audiences
                    .insert(observation.record.id, vec![ProjectionAudience::Host]);
            }
        }
        let event_visibility = if let (Some(before), Some(row)) = (before, row) {
            accepted_event_visibility(before, state, row, pack, &mut history)?
        } else {
            // No knowledge is invented for an opaque pre-table anchor. Such old
            // entries remain host diagnostics and cannot become player transcript.
            history.visibility.extend(
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
        if state.applied_event_sequence < bootstrap_head {
            return Ok(());
        }
        if !bootstrapped {
            if state.applied_event_sequence != bootstrap_head {
                return Err("bootstrap is before the authenticated anchor".into());
            }
            // A new acceptance can never precede initialization of its protocol.
            if export.command_audit.iter().any(|a| {
                matches!(a.command_schema_version, 2 | 3)
                    && a.command_kind == "table.action"
                    && a.resulting_event_sequence <= bootstrap_head as i64
            }) || export.observations.iter().any(|o| {
                o.ordinal <= bootstrap_observation
                    && o.record.kind == "table.conversation"
                    && matches!(o.record.payload_schema_version, 2 | 3)
            }) {
                return Err("protocol marker precedes its bootstrap".into());
            }
            history.observation_ordinal = bootstrap_observation;
            if let Some(record) = bootstrap {
                let transcript = history.visibility.clone();
                validate_record(
                    state,
                    pack,
                    &events,
                    &export.observations,
                    &mut history,
                    record,
                    &transcript,
                )?;
            }
            bootstrapped = true;
        } else if let Some(row) = row {
            let record = export
                .table_projection_history
                .get(next_record)
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
            let prior = history.latest.clone();
            validate_record(
                state,
                pack,
                &events,
                &export.observations,
                &mut history,
                record,
                &event_visibility,
            )?;
            crate::table_transport_runtime::validate_event_binding(
                export,
                before.ok_or("missing prior image")?,
                state,
                row,
                &prior,
                &history,
                pack,
            )?;
            next_record += 1;
        }
        while let Some(record) = export.table_projection_history.get(next_record) {
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
            if observation.ordinal != history.observation_ordinal + 1 {
                return Err("presentation observation order mismatch".into());
            }
            let prior = history.latest.clone();
            history.observation_ordinal = observation.ordinal;
            validate_record(
                state,
                pack,
                &events,
                &export.observations,
                &mut history,
                record,
                &[],
            )?;
            crate::table_transport_runtime::validate_observation_binding(
                export,
                state,
                observation,
                &prior,
                &history,
                pack,
            )?;
            next_record += 1;
        }
        Ok(())
    })?;
    if !bootstrapped {
        return Err("presentation bootstrap image unavailable".into());
    }
    if bootstrap.is_some() && next_record != export.table_projection_history.len() {
        return Err("unconsumed presentation evidence".into());
    }
    if bootstrap.is_none() {
        history.observation_ordinal = export.observations.len() as u64;
    }
    Ok(history)
}
