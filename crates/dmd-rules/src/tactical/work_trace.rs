//! Causal work ownership for the one continuation stack. Only executor-created
//! ancestry can carry an area's invocation-specific ordering delegation.
use super::turns::*;
use super::*;
use std::collections::HashMap;

pub(super) fn initial(state: &CampaignState) -> Result<Option<TacticalWorkTrace>, RulesError> {
    Ok(
        TacticalExecutionVersion::from_flow_version(flow(state)?.version)
            .is_some_and(TacticalExecutionVersion::retains_work_ancestry)
            .then(TacticalWorkTrace::default),
    )
}

pub(super) fn register(
    resolution: &mut TacticalResolution,
    work: &TacticalWorkItem,
) -> Result<(), RulesError> {
    if let Some(trace) = &mut resolution.work_trace {
        if trace.nodes.len() >= 32_768
            || trace
                .nodes
                .last()
                .is_some_and(|node| node.work.occurrence >= work.occurrence)
            || trace.active.is_some_and(|parent| {
                parent >= work.occurrence
                    || trace
                        .nodes
                        .binary_search_by_key(&parent, |node| node.work.occurrence)
                        .is_err()
            })
        {
            return Err(invalid("work ancestry is duplicated, future or unbounded"));
        }
        trace.nodes.push(TacticalWorkNode {
            work: work.clone(),
            parent: trace.active,
        });
    }
    Ok(())
}

/// The old executor has no ancestry image; it retains its dedicated area-only
/// behavior. A current executor must bind every live occurrence to its exact node.
pub(super) fn enter(
    state: &mut CampaignState,
    work: &TacticalWorkItem,
) -> Result<Option<u16>, RulesError> {
    let Some(trace) = &mut resolution_mut(state)?.work_trace else {
        return Ok(None);
    };
    if trace
        .nodes
        .binary_search_by_key(&work.occurrence, |node| node.work.occurrence)
        .ok()
        .and_then(|index| trace.nodes.get(index))
        .is_none_or(|node| node.work != *work)
    {
        return Err(invalid("executed work has no exact ancestry record"));
    }
    Ok(trace.active.replace(work.occurrence))
}

pub(super) fn leave(state: &mut CampaignState, previous: Option<u16>) -> Result<(), RulesError> {
    if let Some(trace) = &mut resolution_mut(state)?.work_trace {
        trace.active = previous;
    }
    Ok(())
}

pub(super) fn prior_work(
    state: &CampaignState,
    kind: &TacticalWorkKind,
) -> Result<Option<TacticalWorkItem>, RulesError> {
    let Some(trace) = &resolution(state)?.work_trace else {
        return Ok(None);
    };
    trace
        .nodes
        .iter()
        .rev()
        .find(|node| node.work.kind == *kind)
        .map(|node| Some(node.work.clone()))
        .ok_or_else(|| invalid("selected consequence lost its originating work"))
}

fn area_kind(kind: &TacticalWorkKind) -> Option<u16> {
    match *kind {
        TacticalWorkKind::AreaDamageRoll { area }
        | TacticalWorkKind::AreaSave { area, .. }
        | TacticalWorkKind::BeginAreaDamage { area }
        | TacticalWorkKind::ApplyAreaDamage { area, .. }
        | TacticalWorkKind::FinishArea { area } => Some(area),
        _ => None,
    }
}

type WorkScopes<'a> = HashMap<u16, (&'a TacticalWorkItem, Option<u16>)>;

/// Parents are allocated before children. One forward pass checks the DAG and
/// derives every scope; it must not walk the entire ancestry once per node.
fn scopes(resolution: &TacticalResolution) -> Result<WorkScopes<'_>, RulesError> {
    let mut scopes = HashMap::new();
    let Some(trace) = &resolution.work_trace else {
        return Ok(scopes);
    };
    if trace.nodes.len() > 32_768 {
        return Err(invalid("excessive causal work ancestry"));
    }
    if trace
        .nodes
        .windows(2)
        .any(|pair| pair[0].work.occurrence >= pair[1].work.occurrence)
    {
        return Err(invalid("work ancestry is not in allocation order"));
    }
    for node in &trace.nodes {
        if node.work.occurrence >= resolution.next_occurrence
            || scopes.contains_key(&node.work.occurrence)
        {
            return Err(invalid("future or duplicate causal work identity"));
        }
        let inherited = if let Some(parent) = node.parent {
            if parent >= node.work.occurrence {
                return Err(invalid("work ancestor does not precede its child"));
            }
            scopes
                .get(&parent)
                .map(|(_, scope)| *scope)
                .ok_or_else(|| invalid("work ancestor is absent"))?
        } else {
            None
        };
        let area = if matches!(node.work.kind, TacticalWorkKind::CommitShield { .. }) {
            // Preserve the causal parent, but a respondent's independent cast
            // cannot inherit an area's invocation-specific ordering delegation.
            None
        } else {
            area_kind(&node.work.kind).or(inherited)
        };
        if let Some(area) = area {
            if !resolution
                .areas
                .iter()
                .any(|record| record.occurrence == area)
            {
                return Err(invalid("ordering ancestor lacks its admitted area"));
            }
            if !matches!(
                node.work.kind,
                TacticalWorkKind::AreaDamageRoll { .. }
                    | TacticalWorkKind::AreaSave { .. }
                    | TacticalWorkKind::BeginAreaDamage { .. }
                    | TacticalWorkKind::ApplyAreaDamage { .. }
                    | TacticalWorkKind::FinishArea { .. }
                    | TacticalWorkKind::ConcentrationSave { .. }
                    | TacticalWorkKind::StableRecovery { .. }
                    | TacticalWorkKind::RecoverStable { .. }
                    | TacticalWorkKind::Effect { .. }
                    | TacticalWorkKind::BeginFall { .. }
                    | TacticalWorkKind::LiquidLandingCheck { .. }
                    | TacticalWorkKind::FallDamage { .. }
            ) {
                return Err(invalid(
                    "independent action inherits unrelated area ordering",
                ));
            }
        }
        scopes.insert(node.work.occurrence, (&node.work, area));
    }
    Ok(scopes)
}

fn area_for(
    resolution: &TacticalResolution,
    scopes: &WorkScopes<'_>,
    work: &TacticalWorkItem,
) -> Result<Option<u16>, RulesError> {
    if resolution.work_trace.is_none() {
        return Ok(resolution
            .areas
            .iter()
            .find(|area| area.source.invocation == resolution.origin)
            .map(|area| area.occurrence));
    }
    scopes
        .get(&work.occurrence)
        .filter(|(original, _)| **original == *work)
        .map(|(_, area)| *area)
        .ok_or_else(|| invalid("ordering work lacks its exact causal node"))
}

/// True only when every currently offered sibling belongs to the same admitted
/// area, or to the source-specific after-turn legendary adjudication. The mere
/// presence of an area elsewhere does not authorize a host-only choice here.
pub fn tactical_frame_host_ordering(resolution: &TacticalResolution) -> Result<bool, RulesError> {
    if resolution.work_trace.is_none() && !resolution.areas.is_empty() {
        // The legacy validator permits only one dedicated original invocation.
        return Ok(true);
    }
    let scopes = scopes(resolution)?;
    if let Some(work) = resolution
        .pending
        .as_ref()
        .map(|pending| &pending.work)
        .or_else(|| {
            resolution
                .failed_save
                .as_ref()
                .map(|failed| &failed.pending.work)
        })
    {
        return Ok(area_for(resolution, &scopes, work)?.is_some());
    }
    if resolution.legendary_window.is_some() {
        return Ok(true);
    }
    if let Some((index, _)) = resolution
        .falls
        .iter()
        .enumerate()
        .find(|(_, fall)| fall.stage == TacticalFallStage::LandingChoice)
    {
        if resolution.work_trace.is_none() {
            // Old non-area liquid choices belong to their actor; their saved
            // executor never retained a causal ancestry image.
            return Ok(false);
        }
        let work = resolution
            .work_trace
            .as_ref()
            .and_then(|trace| {
                trace.nodes.iter().rev().find(|node| {
                    node.work.kind == TacticalWorkKind::BeginFall { fall: index as u16 }
                })
            })
            .ok_or_else(|| invalid("selected fall lacks its ordering ancestry"))?;
        return Ok(area_for(resolution, &scopes, &work.work)?.is_some());
    }
    let Some(frame) = resolution.frames.last().filter(|frame| !frame.is_empty()) else {
        return Ok(false);
    };
    if frame
        .iter()
        .all(|work| matches!(work.kind, TacticalWorkKind::LegendaryWindow { .. }))
    {
        return Ok(true);
    }
    let first = area_for(resolution, &scopes, &frame[0])?;
    for work in &frame[1..] {
        if area_for(resolution, &scopes, work)? != first {
            return Err(invalid(
                "simultaneous work mixes unrelated ordering authorities",
            ));
        }
    }
    Ok(first.is_some())
}

pub(super) fn validate(state: &CampaignState) -> Result<(), RulesError> {
    let Some(resolution) = flow(state)?.resolution.as_deref() else {
        return Ok(());
    };
    let expected = TacticalExecutionVersion::from_flow_version(flow(state)?.version)
        .is_some_and(TacticalExecutionVersion::retains_work_ancestry);
    if resolution.work_trace.is_some() != expected {
        return Err(invalid("work ancestry differs from its execution version"));
    }
    let Some(trace) = &resolution.work_trace else {
        return Ok(());
    };
    if trace.active.is_some() || trace.nodes.len() > 32_768 {
        return Err(invalid("unfinished or excessive in-process work ancestry"));
    }
    let scopes = scopes(resolution)?;
    let mut response_casts = std::collections::HashSet::new();
    for node in &trace.nodes {
        if !matches!(
            node.work.kind,
            TacticalWorkKind::CommitShield { .. } | TacticalWorkKind::ResumeHit { .. }
        ) {
            continue;
        }
        let parent = node
            .parent
            .and_then(|occurrence| scopes.get(&occurrence))
            .map(|(work, _)| *work)
            .ok_or_else(|| invalid("hit response lacks its original parent"))?;
        if flow(state)?.version != TacticalExecutionVersion::ShieldHitV1.flow_version()
            || parent.kind != TacticalWorkKind::AttackRoll
        {
            return Err(invalid(
                "hit response ancestry uses another executor or trigger",
            ));
        }
        match node.work.kind {
            TacticalWorkKind::CommitShield { cast } => {
                if cast <= parent.occurrence
                    || cast >= node.work.occurrence
                    || !response_casts.insert(cast)
                {
                    return Err(invalid(
                        "independent Shield child reuses or invents a cast occurrence",
                    ));
                }
            }
            TacticalWorkKind::ResumeHit { attack_origin, attack_roll } => {
                // Nested opportunity attacks and later spell rays have their own
                // issuance origins. Bind the exact accepted request rather than
                // guessing its key from the root movement/casting command.
                let roll = state
                    .rules
                    .as_ref()
                    .ok_or(RulesError::Uninitialized)?
                    .rolls
                    .iter()
                    .find(|roll| roll.request.id == attack_roll)
                    .ok_or_else(|| invalid("retired hit lacks its accepted attack"))?;
                let PendingPurpose::TacticalResolution { encounter: encounter_id, key } = roll.purpose else {
                    return Err(invalid("retired hit is not an accepted tactical attack"));
                };
                if encounter_id != encounter(state)?.id
                    || key.role != TacticalRollRole::Attack
                    || key.occurrence != parent.occurrence
                    || key.request_id() != attack_roll
                    || roll.issued_by.id != attack_origin
                    || roll.issued_by.session_id != resolution.origin.session_id
                    || roll.issued_by.expected_event_sequence < resolution.origin.expected_event_sequence
                {
                    return Err(invalid(
                        "retired hit resume differs from its original attack",
                    ));
                }
            }
            _ => unreachable!(),
        }
    }
    for work in resolution
        .frames
        .iter()
        .flatten()
        .chain(resolution.pending.iter().map(|pending| &pending.work))
        .chain(
            resolution
                .failed_save
                .iter()
                .map(|failed| &failed.pending.work),
        )
        .chain(
            resolution
                .legendary_window
                .iter()
                .map(|window| &window.work),
        )
    {
        if scopes
            .get(&work.occurrence)
            .is_none_or(|(original, _)| **original != *work)
        {
            return Err(invalid("live work differs from its causal source record"));
        }
    }
    tactical_frame_host_ordering(resolution)?;
    Ok(())
}
