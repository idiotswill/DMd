//! One owned ground drag on the shared movement executor. Retained history is
//! evidence only: each uncommitted segment still requires the actual live grip.
use super::*;

pub(crate) fn history(state: &CampaignState) -> Option<&GrappleTransportHistory> {
    state.encounter.as_ref()?.flow.as_ref()?.resolution.as_ref()?
        .grapple.as_ref()?.transport.as_ref()
}

pub(crate) fn admit(
    read: &execution::ReadContext<'_>, meta: &CommandMeta, grip: GrappleId,
    path: &[TacticalMoveStep],
) -> Result<(), RulesError> {
    read.require_guarded("ground drag requires owned original history")?;
    let state = read.state();
    require_execution(state)?;
    if !crate::table::grapple_transport_enabled(state) {
        return Err(prerequisite("ground drag requires its table activation"));
    }
    let proof = lifecycle::live_grip(state, grip)?;
    let holder = proof.declaration.grappler;
    super::super::shove::authorize_owner(state, meta, holder)?;
    if active(state)? != holder {
        return Err(prerequisite("ground drag belongs to the holder's current turn"));
    }
    let start = encounter(state)?.participant(holder).ok_or_else(|| invalid("holder absent"))?.position;
    if path.iter().any(|step| !matches!(step.mode, MovementMode::Walk | MovementMode::Crawl)
        || step.destination.z != start.z) {
        return Err(prerequisite("ground drag requires horizontal walking or crawling"));
    }
    Ok(())
}

pub(crate) fn capture(state: &mut CampaignState, grip: GrappleId) -> Result<(), RulesError> {
    let proof = lifecycle::live_grip(state, grip)?.clone();
    let movement = resolution(state)?.movement.as_ref().ok_or_else(|| invalid("drag movement absent"))?;
    let target = encounter(state)?.participant(proof.declaration.target)
        .ok_or_else(|| invalid("drag target absent"))?;
    let admission = GrappleTransportAdmission {
        origin: movement.origin.clone(), kind: GrappleTransportKind::GroundDragV1, grip,
        holder: movement.actor, target: target.entity_id,
        holder_from: movement.initial_position, target_from: target.position,
        path: movement.path.clone(), spent_before: movement.initial_spent,
    };
    modern_lifecycle::retain(state, &proof)?;
    let context = context_mut(state)?;
    if context.transport.is_some() { return Err(invalid("ground drag captured twice")); }
    context.transport = Some(GrappleTransportHistory { admission, steps: vec![], stop: None });
    Ok(())
}

pub(crate) fn current_target(state: &CampaignState, movement: &TacticalMovement) -> Result<Option<EntityId>, RulesError> {
    let Some(history) = history(state) else { return Ok(None); };
    if history.admission.origin != movement.origin || history.admission.holder != movement.actor {
        return Err(invalid("ground drag belongs to another movement"));
    }
    if history.stop.is_some() { return Err(prerequisite("the selected grip ended")); }
    let proof = lifecycle::live_grip(state, history.admission.grip)?;
    if proof.declaration.grappler != movement.actor || proof.declaration.target != history.admission.target {
        return Err(invalid("ground drag live relation differs"));
    }
    let expected = history.steps.last().map_or(history.admission.target_from, |s| s.target.to);
    if encounter(state)?.participant(history.admission.target).is_none_or(|body| body.position != expected) {
        return Err(prerequisite("accepted target displacement interrupted ground drag"));
    }
    Ok(Some(history.admission.target))
}

pub(crate) fn interrupted(state: &CampaignState) -> Result<bool, RulesError> {
    let Some(history) = history(state) else { return Ok(false); };
    if let Some(stop) = &history.stop {
        return Ok(context(state)?.ends.iter().find(|end| end.grip == stop.ended_grip)
            .is_none_or(|end| end.cause != GrappleEndCause::Released));
    }
    let expected = history.steps.last().map_or(history.admission.target_from, |s| s.target.to);
    Ok(encounter(state)?.participant(history.admission.target).is_none_or(|body| body.position != expected)
        || state.rules.as_ref().and_then(|r| r.entities.get(&history.admission.target)).is_none_or(|body| body.death.dead))
}

pub(crate) fn record(
    state: &mut CampaignState, meta: &CommandMeta, segment: &crate::spatial::CoupledGroundSegment,
) -> Result<(), RulesError> {
    let work = modern_lifecycle::active_work(state)?;
    if work.kind != TacticalWorkKind::MoveSegment { return Err(invalid("drag step is not entered movement work")); }
    let key = work_key(state, &work)?;
    let history = context_mut(state)?.transport.as_mut().ok_or_else(|| invalid("drag history absent"))?;
    let step = u16::try_from(history.steps.len()).map_err(|_| invalid("drag receipt capacity"))?;
    history.steps.push(GrappleTransportStepReceipt {
        work: key, step, cause: meta.clone(),
        holder: GrappleBodyDisplacement { actor: history.admission.holder, from: segment.holder.from, to: segment.holder.to },
        target: segment.target.clone(), mode: segment.holder.mode,
        ordinary_cost: segment.ordinary_cost, haul_cost: segment.haul_cost,
        holder_size: segment.holder_size, target_size: segment.target_size,
    });
    Ok(())
}

pub(crate) fn result(state: &CampaignState) -> Result<Option<GrappleTransportResult>, RulesError> {
    let Some(history) = history(state) else { return Ok(None); };
    let target_endpoint = encounter(state)?.participant(history.admission.target)
        .ok_or_else(|| invalid("drag target disappeared"))?.position;
    let (ordinary_cost, haul_cost) = history.steps.iter().try_fold((0u32, 0u32), |(ordinary, haul), step| {
        Ok::<_, RulesError>((ordinary.checked_add(step.ordinary_cost).ok_or_else(|| invalid("drag cost overflow"))?,
            haul.checked_add(step.haul_cost).ok_or_else(|| invalid("drag cost overflow"))?))
    })?;
    Ok(Some(GrappleTransportResult { kind: history.admission.kind, grip: history.admission.grip,
        target: history.admission.target, target_start: history.admission.target_from,
        target_endpoint, ordinary_cost, haul_cost }))
}

/// Latch only this selected relation. The movement adapter retires unanswered
/// prompts while leaving every already-issued child and its parent in place.
pub(crate) fn ended(state: &mut CampaignState, meta: &CommandMeta, grip: GrappleId) -> Result<bool, RulesError> {
    if history(state).is_none_or(|history| history.admission.grip != grip) { return Ok(false); }
    let context = context_mut(state)?;
    if !context.ends.iter().any(|end| end.grip == grip && end.caused_by == *meta) {
        return Err(invalid("ground drag stop has no actual ending"));
    }
    context.transport.as_mut().ok_or_else(|| invalid("drag history absent"))?.stop =
        Some(GrappleTransportStop { cause: meta.clone(), ended_grip: grip });
    super::super::movement::stop_ground_transport(state, meta)?;
    Ok(true)
}

pub(crate) fn moved_by(state: &CampaignState, work: &TacticalWorkItem, actor: EntityId) -> bool {
    work.kind == TacticalWorkKind::MoveSegment && history(state).is_some_and(|history|
        history.steps.iter().any(|step| step.work.occurrence == work.occurrence
            && ((step.holder.actor == actor && step.holder.from != step.holder.to)
                || (step.target.actor == actor && step.target.from != step.target.to))))
}

pub(crate) fn validate(state: &CampaignState) -> Result<(), RulesError> {
    let Some(history) = history(state) else { return Ok(()); };
    if !crate::table::grapple_transport_enabled(state) { return Err(invalid("ground drag activation absent")); }
    let resolution = resolution(state)?;
    let context = context(state)?;
    let proof = context.proofs.iter().find(|p| p.declaration.id == history.admission.grip)
        .ok_or_else(|| invalid("drag proof absent"))?;
    validation::causal(state, &history.admission.origin, &proof.established_by, history.admission.holder)?;
    super::super::shove::authorize_owner(state, &history.admission.origin, history.admission.holder)?;
    for step in &history.steps {
        if step.cause.expected_event_sequence > state.applied_event_sequence {
            return Err(invalid("drag step comes from the future"));
        }
    }
    if resolution.movement.is_none() {
        let result = flow(state)?.last_movement.as_ref().ok_or_else(|| invalid("retired drag lacks its result"))?;
        if result.original != history.admission.origin || result.transport != self::result(state)?
            || usize::from(result.completed_steps) != history.steps.len()
            || usize::from(result.requested_steps) != history.admission.path.len()
            || result.start != history.admission.holder_from || result.spent_before != history.admission.spent_before {
            return Err(invalid("retired drag result differs from its retained execution"));
        }
    }
    Ok(())
}

/// Offers disclose only an owned selected relation; a path is checked at actual
/// movement time so no remote geometry or hidden future blockers are previewed.
pub(crate) fn choices(read: &execution::ReadContext<'_>, issuer: CommandIssuer) -> Result<Vec<TableGrappleTransportOffer>, RulesError> {
    let state = read.state();
    if !crate::table::grapple_transport_enabled(state) { return Ok(vec![]); }
    read.require_guarded("ground drag choices require original table history")?;
    let Some(session) = state.table.as_ref().and_then(|t| t.active_session.as_ref()) else { return Ok(vec![]); };
    if let CommandIssuer::Player(player) = issuer
        && !session.participants.iter().any(|p| p.player_id == player && p.attendance == AttendanceStatus::Present) {
        return Ok(vec![]);
    }
    let current_flow = flow(state)?;
    if current_flow.phase != TacticalPhase::Active || current_flow.resolution.is_some()
        || current_flow.version != TacticalExecutionVersion::EncounterReleaseV1.flow_version() {
        return Ok(vec![]);
    }
    let holder = active(state)?;
    let meta = CommandMeta {
        id: CommandId(uuid::Uuid::new_v5(&state.campaign_id().0, holder.0.as_bytes())),
        campaign_id: state.campaign_id(), session_id: Some(session.session_id), issuer,
        actor: matches!(issuer, CommandIssuer::Player(_)).then_some(AgentRef::Entity(holder)),
        expected_event_sequence: state.applied_event_sequence,
    };
    if super::super::shove::authorize_owner(state, &meta, holder).is_err() { return Ok(vec![]); }
    let rules = state.rules.as_ref().ok_or(RulesError::Uninitialized)?;
    if let CommandIssuer::Player(player) = issuer
        && rules.tactical_creatures.as_ref().and_then(|c| c.runtime(holder)).is_some()
        && !crate::table::source_control::owns_source(state, player, holder) { return Ok(vec![]); }
    let mut result = rules.tactical_grapples.as_ref().into_iter().flat_map(|g| &g.active)
        .filter(|g| g.declaration.grappler == holder)
        .map(|g| TableGrappleTransportOffer { actor: holder, grip: g.declaration.id, kind: GrappleTransportKind::GroundDragV1 })
        .collect::<Vec<_>>();
    result.sort_by_key(|offer| offer.grip.0);
    Ok(result)
}
