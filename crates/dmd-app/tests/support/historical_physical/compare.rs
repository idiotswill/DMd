//! Exact old images and narrowly related *new* presentation slots are separate checks.
use super::*;
use std::collections::{HashMap, HashSet};

pub fn exact_export(expected: &CampaignExport, actual: &CampaignExport) {
    let mut actual = actual.clone();
    actual.exported_at_utc.clone_from(&expected.exported_at_utc);
    assert_eq!(
        &actual, expected,
        "only export wall-clock metadata may differ"
    );
}
pub fn prefix(before: &CampaignExport, after: &CampaignExport) {
    assert_eq!(before.campaign_id, after.campaign_id);
    assert!(after.command_audit.starts_with(&before.command_audit));
    assert!(after.event_journal.starts_with(&before.event_journal));
    assert!(after.snapshots.starts_with(&before.snapshots));
    assert!(
        after
            .table_projection_history
            .starts_with(&before.table_projection_history)
    );
    let old: HashSet<_> = before
        .table_transport_bindings
        .iter()
        .map(|b| b.meta.id)
        .collect();
    assert_eq!(
        after
            .table_transport_bindings
            .iter()
            .filter(|b| old.contains(&b.meta.id))
            .collect::<Vec<_>>(),
        before.table_transport_bindings.iter().collect::<Vec<_>>()
    );
    let events: HashSet<_> = before.event_journal.iter().map(|e| &e.id).collect();
    assert_eq!(
        after
            .event_causes
            .iter()
            .filter(|c| events.contains(&c.event_id))
            .collect::<Vec<_>>(),
        before.event_causes.iter().collect::<Vec<_>>()
    );
    assert!(after.observations.starts_with(&before.observations));
}
pub fn event(export: &CampaignExport, command: CommandId) -> TableEvent {
    let rows = export
        .event_journal
        .iter()
        .filter(|e| e.command_id == command.0.to_string())
        .collect::<Vec<_>>();
    assert_eq!(rows.len(), 1);
    serde_json::from_str(&rows[0].payload_json).unwrap()
}
pub fn receipt(
    expected: &TableTransportResult,
    actual: &TableTransportResult,
    revision: ProjectionRevision,
) {
    let (TableTransportResult::Accepted(expected), TableTransportResult::Accepted(actual)) =
        (expected, actual)
    else {
        panic!("genuine accepted command required")
    };
    assert_eq!(expected.command_id, actual.command_id);
    assert_eq!(expected.outcome, actual.outcome);
    assert_eq!(actual.revision, revision);
}
pub fn fresh(
    before: &CampaignExport,
    expected: &CampaignExport,
    actual: &CampaignExport,
    request: &TableTransportRequest,
) {
    prefix(before, expected);
    prefix(before, actual);
    assert_eq!(
        decoded(expected),
        decoded(actual),
        "fresh canonical outcome"
    );
    assert_eq!(expected.format_version, actual.format_version);
    assert_eq!(expected.state_schema_version, actual.state_schema_version);
    assert_eq!(expected.campaign_id, actual.campaign_id);
    assert_eq!(expected.lifecycle, actual.lifecycle);
    assert_eq!(expected.play_sessions, actual.play_sessions);
    assert_eq!(
        expected.play_session_participants,
        actual.play_session_participants
    );
    assert_eq!(expected.snapshots, actual.snapshots);
    assert_eq!(expected.observations, actual.observations);
    assert_eq!(
        expected.current_state.campaign_id,
        actual.current_state.campaign_id
    );
    assert_eq!(
        expected.current_state.schema_version,
        actual.current_state.schema_version
    );
    assert_eq!(
        expected.current_state.applied_event_sequence,
        actual.current_state.applied_event_sequence
    );
    assert_eq!(
        actual.current_state.applied_event_sequence,
        before.current_state.applied_event_sequence + 1
    );
    assert_eq!(actual.command_audit.len(), before.command_audit.len() + 1);
    assert_eq!(actual.event_journal.len(), before.event_journal.len() + 1);
    assert_eq!(
        actual.table_projection_history.len(),
        before.table_projection_history.len() + 1
    );
    assert_eq!(
        actual.table_transport_bindings.len(),
        before.table_transport_bindings.len() + 1
    );
    let expected_event = event(expected, request.command_id);
    assert_eq!(expected_event, event(actual, request.command_id));
    let e = expected.event_journal.last().unwrap();
    let a = actual.event_journal.last().unwrap();
    assert_eq!(a.command_id, request.command_id.0.to_string());
    // Only this new outer EventId and the proven equal typed event's serialization.
    let mut row = a.clone();
    row.id.clone_from(&e.id);
    row.payload_json.clone_from(&e.payload_json);
    assert_eq!(&row, e);
    assert!(!before.event_journal.iter().any(|old| old.id == a.id));
    assert_eq!(actual.command_audit.last(), expected.command_audit.last());
    let mut causes = actual.event_causes.clone();
    for cause in &mut causes {
        if cause.event_id == a.id {
            cause.event_id.clone_from(&e.id);
        }
        // Causal parents must be old, literal identities; never map arbitrary UUIDs.
        assert_ne!(cause.cause_event_id, a.id);
    }
    assert_eq!(causes, expected.event_causes);
    let ep = expected.table_projection_history.last().unwrap();
    let ap = actual.table_projection_history.last().unwrap();
    assert_eq!(
        (ep.version, ep.campaign_id, ep.ordinal),
        (ap.version, ap.campaign_id, ap.ordinal)
    );
    assert_eq!(
        ep.cause,
        dmd_persistence::ProjectionCause::Event {
            id: EventId(uuid::Uuid::parse_str(&e.id).unwrap()),
            command_id: request.command_id
        }
    );
    assert_eq!(
        ap.cause,
        dmd_persistence::ProjectionCause::Event {
            id: EventId(uuid::Uuid::parse_str(&a.id).unwrap()),
            command_id: request.command_id
        }
    );
    let mut visibility = ap.transcript.clone();
    for visible in &mut visibility {
        if visible.event_id.0.to_string() == a.id {
            visible.event_id = EventId(uuid::Uuid::parse_str(&e.id).unwrap());
        }
    }
    assert_eq!(visibility, ep.transcript);
    assert_eq!(ep.changes.len(), ap.changes.len());
    let old_revisions = before
        .table_projection_history
        .iter()
        .flat_map(|p| &p.changes)
        .map(|c| c.revision)
        .collect::<HashSet<_>>();
    let old_handles = before
        .table_projection_history
        .iter()
        .flat_map(|p| &p.changes)
        .flat_map(|c| {
            c.handles
                .iter()
                .map(move |h| (h.opaque, c.audience, &h.capability))
        })
        .collect::<Vec<_>>();
    let mut new_revisions = HashSet::new();
    let mut new_handles = HashMap::new();
    for (ec, ac) in ep.changes.iter().zip(&ap.changes) {
        assert_eq!(ec.audience, ac.audience, "ordered changed audiences");
        assert_eq!(ec.previous, ac.previous);
        assert_eq!(ec.previous, Some(latest(before, ec.audience).revision));
        assert!(!old_revisions.contains(&ac.revision));
        assert!(new_revisions.insert(ac.revision));
        assert_eq!(ec.handles.len(), ac.handles.len());
        for (eh, ah) in ec.handles.iter().zip(&ac.handles) {
            assert_eq!(
                eh.capability, ah.capability,
                "canonical capability order and ownership"
            );
            if let Some(old) = latest(before, ec.audience)
                .handles
                .iter()
                .find(|h| h.capability == eh.capability)
            {
                assert_eq!(eh.opaque, old.opaque);
                assert_eq!(ah.opaque, old.opaque);
            } else {
                assert!(
                    !old_handles.iter().any(|(id, _, _)| *id == ah.opaque),
                    "new prompt must not borrow any old handle"
                );
            }
            assert!(
                new_handles
                    .insert(ah.opaque, (ac.audience, &ah.capability))
                    .is_none(),
                "new audience projections cannot share opaque handles"
            );
        }
        // Digests remain untouched. Both images pass actual restore/replay below.
        assert_eq!(ec.visible_digest.len(), 64);
        assert_eq!(ac.visible_digest.len(), 64);
    }
    let eb = expected
        .table_transport_bindings
        .iter()
        .find(|b| b.meta.id == request.command_id)
        .unwrap();
    let ab = actual
        .table_transport_bindings
        .iter()
        .find(|b| b.meta.id == request.command_id)
        .unwrap();
    assert_eq!(ab.request_json, eb.request_json);
    assert_eq!(ab.request_json, serde_json::to_string(request).unwrap());
    let er: TableTransportResult = serde_json::from_str(&eb.response_json).unwrap();
    let ar: TableTransportResult = serde_json::from_str(&ab.response_json).unwrap();
    receipt(&er, &ar, latest(actual, ab.audience).revision);
    let mut binding = ab.clone();
    binding.response_json.clone_from(&eb.response_json);
    assert_eq!(&binding, eb);
}

fn map_key(key: &mut CommandId, map: &HashMap<uuid::Uuid, uuid::Uuid>) {
    key.0 = *map
        .get(&key.0)
        .expect("typed presented key has exact audience/capability join");
}
/// This operates only on a comparison copy; no mapped image is restored or accepted.
fn relate_view(view: &mut TablePresentedView, map: &HashMap<uuid::Uuid, uuid::Uuid>) {
    if let Some(roll) = &mut view.roll {
        roll.id.0 = map[&roll.id.0];
    }
    if let Some(physical) = &mut view.physical {
        for control in &mut physical.controls {
            map_key(&mut control.key, map);
        }
    }
    if let Some(transfer) = &mut view.inspiration_transfer {
        for choice in &mut transfer.choices {
            map_key(&mut choice.key, map);
        }
    }
    if let Some(grapple) = &mut view.grapple {
        for choice in &mut grapple.choices {
            map_key(&mut choice.key, map);
        }
        for choice in &mut grapple.ground_drag {
            map_key(&mut choice.key, map);
        }
    }
    if let Some(tactical) = &mut view.tactical {
        if let Some(equipment) = &mut tactical.attack_equipment {
            map_key(&mut equipment.key, map);
        }
        if let Some(shove) = &mut tactical.shove {
            map_key(&mut shove.key, map);
        }
        if let Some(work) = &mut tactical.continuation {
            for choice in &mut work.choices {
                map_key(&mut choice.handle, map);
            }
        }
        if let Some(hit) = &mut tactical.hit {
            if let Some(order) = &mut hit.order {
                map_key(&mut order.key, map);
            }
            if let Some(delegate) = &mut hit.delegate {
                map_key(delegate, map);
            }
            if let Some(response) = &mut hit.response {
                map_key(&mut response.key, map);
            }
        }
        if let Some(missile) = &mut tactical.missile {
            if let Some(order) = &mut missile.order {
                map_key(&mut order.key, map);
            }
            if let Some(delegate) = &mut missile.delegate {
                map_key(delegate, map);
            }
            for response in &mut missile.responses {
                map_key(&mut response.key, map);
            }
        }
    }
}
pub fn fresh_views(
    before: &CampaignExport,
    expected: &CampaignExport,
    actual: &CampaignExport,
    old: &[archive::Audience],
    new: &[archive::Audience],
) {
    assert_eq!(old.len(), new.len());
    let old_event = &expected.event_journal.last().unwrap().id;
    let new_event = &actual.event_journal.last().unwrap().id;
    for (e, a) in old.iter().zip(new) {
        assert_eq!(e.channel, a.channel);
        assert_eq!(e.roll_options, a.roll_options);
        let who = audience(&e.channel);
        let ec = latest(expected, who);
        let ac = latest(actual, who);
        assert_eq!(e.presented.revision, ec.revision);
        assert_eq!(a.presented.revision, ac.revision);
        let changed = expected
            .table_projection_history
            .last()
            .unwrap()
            .changes
            .iter()
            .any(|c| c.audience == who);
        if !changed {
            assert_eq!(ec, latest(before, who));
            assert_eq!(ac, ec);
            assert_eq!(
                serde_json::to_vec(&e.presented).unwrap(),
                serde_json::to_vec(&a.presented).unwrap()
            );
            assert_eq!(e.raw, a.raw);
            continue;
        }
        let mut map = HashMap::new();
        assert_eq!(ec.handles.len(), ac.handles.len());
        for (eh, ah) in ec.handles.iter().zip(&ac.handles) {
            assert_eq!(eh.capability, ah.capability);
            assert!(map.insert(ah.opaque, eh.opaque).is_none());
        }
        let mut presented = a.presented.clone();
        relate_view(&mut presented, &map);
        presented.revision = ec.revision;
        for line in &mut presented.transcript {
            if &line.id == new_event {
                line.id.clone_from(old_event);
            }
        }
        assert_eq!(
            &presented, &e.presented,
            "all other typed presented slots remain literal"
        );
        let mut raw = a.raw.clone();
        for line in &mut raw.transcript {
            if &line.id == new_event {
                line.id.clone_from(old_event);
            }
        }
        assert_eq!(
            raw, e.raw,
            "raw canonical identities do not undergo opaque mapping"
        );
    }
}
pub fn private_award(
    before: &[archive::Audience],
    after: &[archive::Audience],
    label: &str,
    route: &str,
) {
    if matches!(
        label,
        "before-first-host-award" | "first-award-before-duplicate"
    ) || (label == "pending-owner-transfer" && route == "inspiration-decline-v4")
    {
        let e = &before[2];
        let a = &after[2];
        assert_eq!(e.channel, a.channel);
        assert_eq!(
            serde_json::to_vec(&e.presented).unwrap(),
            serde_json::to_vec(&a.presented).unwrap()
        );
        let mut raw = a.raw.clone();
        raw.event_sequence = e.raw.event_sequence;
        assert_eq!(raw, e.raw);
    }
}
