use super::*;

fn node<'a>(
    state: &'a CampaignState,
    work: &TacticalWorkItem,
) -> Result<&'a TacticalWorkNode, RulesError> {
    resolution(state)?
        .work_trace
        .as_ref()
        .and_then(|t| t.nodes.iter().find(|n| n.work == *work))
        .ok_or_else(|| invalid("Grapple work lacks its exact trace node"))
}
fn causal(
    state: &CampaignState,
    meta: &CommandMeta,
    origin: &CommandMeta,
    actor: EntityId,
) -> Result<(), RulesError> {
    validate_equipment_change_origin(state, meta, actor).map_err(|e| invalid(&e))?;
    if meta.campaign_id != origin.campaign_id
        || meta.expected_event_sequence < origin.expected_event_sequence
        || (meta.expected_event_sequence == origin.expected_event_sequence && meta != origin)
        || (meta.id == origin.id && meta != origin)
    {
        return Err(invalid("Grapple command chronology differs"));
    }
    Ok(())
}

pub(in crate::tactical) fn validate_work(
    state: &CampaignState,
    work: &TacticalWorkItem,
) -> Result<EntityId, RulesError> {
    require_execution(state)?;
    node(state, work)?;
    if work.occurrence >= resolution(state)?.next_occurrence {
        return Err(invalid("Future Grapple work occurrence"));
    }
    let actor = match work.kind {
        TacticalWorkKind::BeginGrapple { grip } => {
            let a = attempt(state)?;
            if a.declaration.id != grip
                || !matches!(
                    a.stage,
                    TacticalGrappleAttemptStage::Queued | TacticalGrappleAttemptStage::SaveChoice
                )
            {
                return Err(invalid("Grapple Begin work stage differs"));
            }
            a.declaration.target
        }
        TacticalWorkKind::GrappleSave { grip } => {
            let a = attempt(state)?;
            if a.declaration.id != grip || a.stage != TacticalGrappleAttemptStage::Saving {
                return Err(invalid("Grapple save work stage differs"));
            }
            a.declaration.target
        }
        TacticalWorkKind::GrappleAfterEquipment { grip } => {
            let a = attempt(state)?;
            if a.declaration.id != grip
                || a.stage != TacticalGrappleAttemptStage::AfterEquipment
                || a.equipment.before_change.is_some()
                || a.equipment.after.is_some()
            {
                return Err(invalid("Grapple after work stage differs"));
            }
            a.declaration.grappler
        }
        TacticalWorkKind::GrappleEscapeCheck { grip } => {
            let e = escape(state)?;
            if e.grip != grip
                || !matches!(
                    e.stage,
                    TacticalGrappleEscapeStage::Queued | TacticalGrappleEscapeStage::Rolling
                )
            {
                return Err(invalid("Escape work stage differs"));
            }
            e.actor
        }
        _ => return Err(invalid("Not Grapple work")),
    };
    Ok(actor)
}

/// Request creation precedes Automatic decision/pending installation. Only the
/// exact entered producer or retained selected tuple may use this narrower read.
pub(super) fn validate_request_source(
    state: &CampaignState,
    work: &TacticalWorkItem,
) -> Result<(), RulesError> {
    validate_work(state, work)?;
    let r = resolution(state)?;
    if !r
        .work_trace
        .as_ref()
        .is_some_and(|t| t.active == Some(work.occurrence))
        && !r.pending.as_ref().is_some_and(|p| p.work == *work)
        && !r
            .failed_save
            .as_ref()
            .is_some_and(|p| p.pending.work == *work)
    {
        return Err(invalid(
            "Grapple request has no exact producer or selected work",
        ));
    }
    admission::supported_context(state)?;
    match work.kind {
        TacticalWorkKind::GrappleSave { .. } => admission::validate_attempt_admission(state),
        TacticalWorkKind::GrappleEscapeCheck { .. } => validate_escape_source(state),
        _ => Err(invalid("Grapple choice is not a request producer")),
    }
}

fn validate_escape_source(state: &CampaignState) -> Result<(), RulesError> {
    let e = escape(state)?;
    let r = resolution(state)?;
    let proof = context(state)?
        .proofs
        .iter()
        .find(|p| p.declaration.id == e.grip)
        .ok_or_else(|| invalid("Escape grip proof absent"))?;
    if e.origin != r.origin
        || e.actor != r.turn_actor
        || e.actor != proof.declaration.target
        || e.difficulty != proof.declaration.escape_dc
        || e.key.origin != e.origin.id
        || e.key.subject != e.actor
        || e.key.role != TacticalRollRole::GrappleEscape
        || !state
            .rules
            .as_ref()
            .and_then(|r| r.timing.as_ref())
            .is_some_and(|t| t.action_spent)
    {
        return Err(invalid("Escape paid source differs"));
    }
    super::super::shove::authorize_owner(state, &e.origin, e.actor)?;
    if e.outcome.is_none() && lifecycle::live_grip(state, e.grip)? != proof {
        return Err(invalid("Escape live grip differs"));
    }
    Ok(())
}

fn validate_evidence(
    state: &CampaignState,
    save: &TacticalGrappleSave,
    declaration: &TacticalGrappleDeclaration,
) -> Result<(), RulesError> {
    admission::validate_declaration_source(state, declaration)?;
    // Pending work still derives its request from the actual current source.
    // Final evidence must retain the issued request: establishing this grip can
    // end Dodge, and release must not rewrite that completed Advantage save.
    // Shape, raw/decision, arithmetic, ownership and chronology below remain
    // necessary; matching retained copies alone do not prove original history.
    // Public retained/raw/restore admission therefore remains closed.
    if save.proof.is_none()
        && saves::expected_save(state, declaration, save.ability, save.key)? != save.request
    {
        return Err(invalid(
            "Retained Grapple request differs from its actual source",
        ));
    }
    causal(
        state,
        &save.chosen_by,
        &declaration.origin,
        declaration.target,
    )?;
    super::super::shove::authorize_owner(state, &save.chosen_by, declaration.target)?;
    if let Some(proof) = &save.proof {
        save.validate_shape(declaration).map_err(|e| invalid(&e))?;
        if saves::evidence(state, save)? != proof.evidence {
            return Err(invalid("Grapple final evidence differs"));
        }
        causal(
            state,
            proof.evidence.resolved_by(),
            &save.chosen_by,
            declaration.target,
        )?;
        causal(
            state,
            &proof.finalized_by,
            proof.evidence.resolved_by(),
            declaration.target,
        )?;
        super::super::shove::authorize_owner(state, &proof.finalized_by, declaration.target)?;
        let raw = state
            .rules
            .as_ref()
            .ok_or(RulesError::Uninitialized)?
            .rolls
            .iter()
            .find(|r| r.request.id == save.key.request_id());
        let base_success = if let Some(raw) = raw {
            if (raw.request.visibility == RollVisibility::Secret
                || raw.result.source == RollSource::Digital)
                && !matches!(
                    raw.accepted_by.issuer,
                    CommandIssuer::Admin | CommandIssuer::System
                )
            {
                return Err(RulesError::Unauthorized);
            }
            if raw.resolved != raw.request.resolve(&raw.result)? {
                return Err(invalid("Grapple raw arithmetic differs"));
            }
            crate::test_outcome::ability_test_success(
                &raw.resolved,
                declaration.escape_dc,
                &state.rules.as_ref().unwrap().house_rules,
            )?
        } else {
            false
        };
        let consumed = state
            .rules
            .as_ref()
            .and_then(|r| r.tactical_creatures.as_ref())
            .and_then(|c| c.runtime(declaration.target))
            .is_some_and(|r| {
                r.legendary_resistance_rolls
                    .contains(&save.key.request_id())
            });
        if consumed != proof.legendary.as_ref().is_some_and(|l| l.use_resistance) {
            return Err(invalid("Grapple LR proof differs from source expenditure"));
        }
        // This guarded core permits no intervening source/resource change. A
        // failure with available LR must have reached the producer's decision
        // pause; omitting a decline cannot turn it into an ordinary failure.
        // Exact-key expenditure above also covers the last available use.
        if !base_success
            && proof.legendary.is_none()
            && super::super::failed_save::available(state, declaration.target)?
        {
            return Err(invalid("Grapple failure omitted its required LR decision"));
        }
        if let Some(legendary) = &proof.legendary {
            causal(
                state,
                &legendary.chosen_by,
                proof.evidence.resolved_by(),
                declaration.target,
            )?;
            if !legendary.use_resistance
                && !super::super::failed_save::available(state, declaration.target)?
            {
                return Err(invalid("Grapple decline had no source LR allowance"));
            }
        }
        if proof.final_success
            != (base_success || proof.legendary.as_ref().is_some_and(|l| l.use_resistance))
            || (base_success && proof.legendary.is_some())
        {
            return Err(invalid("Grapple final success differs"));
        }
    }
    Ok(())
}

fn validate_equipment(state: &CampaignState, a: &TacticalGrappleAttempt) -> Result<(), RulesError> {
    let before = &a.equipment.equipment_before;
    crate::tactical_inventory::validate_loadout(state, before)
        .map_err(|e| invalid(&e.to_string()))?;
    if a.equipment.before_change.is_some() && a.equipment.after.is_some() {
        return Err(invalid("Grapple allowance was used twice"));
    }
    if a.outcome.is_none() {
        return admission::validate_attempt_admission(state);
    }
    let mut expected = before.clone();
    // Completed validation reconstructs only physical changes. It never asks to
    // ignore the now-live grip or to treat its occupied hand as currently free.
    if let Some(op) = a.equipment.before_change {
        let hands = crate::tactical_hands::EffectiveHands::current(
            state,
            state.rules.as_ref().unwrap(),
            a.declaration.grappler,
        )?;
        crate::tactical_weapons::apply_attack_equipment_operation(
            state,
            a.declaration.grappler,
            a.declaration.window,
            definitions()?,
            &mut expected.hands,
            op,
            &hands,
        )
        .map_err(|e| invalid(&e.to_string()))?;
        expected.command = a.declaration.origin.clone();
    }
    if let Some(after) = &a.equipment.after {
        let (chosen, work) = match after {
            GrappleEquipmentDecision::Declined { chosen_by, work } => (chosen_by, *work),
            GrappleEquipmentDecision::Applied {
                chosen_by,
                work,
                operation,
                equipment_before,
            } => {
                if equipment_before.as_ref() != &expected {
                    return Err(invalid("After equipment before-image differs"));
                }
                let hands = crate::tactical_hands::EffectiveHands::current(
                    state,
                    state.rules.as_ref().unwrap(),
                    a.declaration.grappler,
                )?;
                crate::tactical_weapons::apply_attack_equipment_operation(
                    state,
                    a.declaration.grappler,
                    a.declaration.window,
                    definitions()?,
                    &mut expected.hands,
                    *operation,
                    &hands,
                )
                .map_err(|e| invalid(&e.to_string()))?;
                expected.command = chosen_by.clone();
                (chosen_by, *work)
            }
        };
        causal(state, chosen, &a.declaration.origin, a.declaration.grappler)?;
        let outcome = match a
            .outcome
            .as_ref()
            .ok_or_else(|| invalid("Equipment lacks resolved Attempt"))?
        {
            GrappleAttemptOutcome::Established { .. } => {
                &a.save
                    .as_ref()
                    .and_then(|s| s.proof.as_ref())
                    .ok_or_else(|| invalid("Equipment lacks final save"))?
                    .finalized_by
            }
            GrappleAttemptOutcome::Resisted { resolved_by }
            | GrappleAttemptOutcome::Immune { resolved_by } => resolved_by,
            GrappleAttemptOutcome::Withdrawn { withdrawn_by, .. } => withdrawn_by,
        };
        causal(state, chosen, outcome, a.declaration.grappler)?;
        super::super::shove::authorize_owner(state, chosen, a.declaration.grappler)?;
        if let Some(end) = context(state)?
            .ends
            .iter()
            .find(|e| e.grip == a.declaration.id)
        {
            causal(state, chosen, &end.caused_by, a.declaration.grappler)?;
        }
        let node = resolution(state)?
            .work_trace
            .as_ref()
            .and_then(|t| {
                t.nodes
                    .iter()
                    .find(|n| n.work.occurrence == work.occurrence)
            })
            .ok_or_else(|| invalid("After work absent"))?;
        if work.resolution != resolution(state)?.origin.id
            || node.work.kind
                != (TacticalWorkKind::GrappleAfterEquipment {
                    grip: a.declaration.id,
                })
        {
            return Err(invalid("After work differs"));
        }
    }
    if admission::loadout(state, a.declaration.grappler)? != &expected {
        return Err(invalid("Grapple physical equipment result differs"));
    }
    Ok(())
}

fn validate_attempt(state: &CampaignState, a: &TacticalGrappleAttempt) -> Result<(), RulesError> {
    admission::validate_attempt_source(state)?;
    validate_equipment(state, a)?;
    let r = resolution(state)?;
    let expected_selected = match a.stage {
        TacticalGrappleAttemptStage::SaveChoice => Some(TacticalWorkKind::BeginGrapple {
            grip: a.declaration.id,
        }),
        TacticalGrappleAttemptStage::AfterEquipment => {
            Some(TacticalWorkKind::GrappleAfterEquipment {
                grip: a.declaration.id,
            })
        }
        TacticalGrappleAttemptStage::Saving | TacticalGrappleAttemptStage::Complete => None,
        _ => return Err(invalid("Unpumped Grapple stage is not a durable pause")),
    };
    if a.selected.as_ref().map(|w| &w.kind) != expected_selected.as_ref() {
        return Err(invalid("Grapple selected stage differs"));
    }
    if let Some(selected) = &a.selected {
        validate_work(state, selected)?;
    }
    if (a.stage == TacticalGrappleAttemptStage::SaveChoice) != a.save.is_none()
        && !matches!(a.outcome, Some(GrappleAttemptOutcome::Withdrawn { .. }))
    {
        return Err(invalid("Grapple save/stage differs"));
    }
    if a.stage == TacticalGrappleAttemptStage::Saving {
        let save = a
            .save
            .as_ref()
            .ok_or_else(|| invalid("Saving ability absent"))?;
        if save.proof.is_some() {
            return Err(invalid("Pending save has final proof"));
        }
        if let Some(failed) = &r.failed_save {
            if r.pending.is_some() || failed.pending.key != save.key {
                return Err(invalid("Grapple failed save tuple differs"));
            }
            saves::evidence(state, save)?;
            super::super::failed_save::validate_failed_save(state, failed)?;
        } else {
            let p = r
                .pending
                .as_ref()
                .ok_or_else(|| invalid("Saving stage lacks selected raw work"))?;
            let raw = state
                .rules
                .as_ref()
                .and_then(|r| r.pending.as_ref())
                .ok_or_else(|| invalid("Saving stage lacks raw request"))?;
            if p.key != save.key || raw.issued_by != save.chosen_by {
                return Err(invalid("Grapple pending identity differs"));
            }
            super::super::turn_validation::pending(state, raw)?;
        }
    }
    if let Some(save) = &a.save {
        let issued = TacticalWorkItem {
            occurrence: save.key.occurrence,
            kind: TacticalWorkKind::GrappleSave {
                grip: a.declaration.id,
            },
        };
        node(state, &issued)?;
        validate_evidence(state, save, &a.declaration)?;
    }
    if let Some(outcome) = &a.outcome {
        match outcome {
            GrappleAttemptOutcome::Established { grip } => {
                let proof = context(state)?
                    .proofs
                    .iter()
                    .find(|p| p.declaration.id == *grip)
                    .ok_or_else(|| invalid("Established outcome proof absent"))?;
                if *grip != a.declaration.id
                    || proof.declaration != a.declaration
                    || Some(&proof.save) != a.save.as_ref()
                {
                    return Err(invalid("Established outcome proof differs"));
                }
            }
            GrappleAttemptOutcome::Resisted { resolved_by }
            | GrappleAttemptOutcome::Immune { resolved_by } => {
                let proof = a
                    .save
                    .as_ref()
                    .and_then(|s| s.proof.as_ref())
                    .ok_or_else(|| invalid("Outcome final save absent"))?;
                let immune = state.rules.as_ref().unwrap().entities[&a.declaration.target]
                    .condition_immunities
                    .contains(&Condition::Grappled);
                if *resolved_by != proof.finalized_by
                    || match outcome {
                        GrappleAttemptOutcome::Resisted { .. } => !proof.final_success,
                        _ => proof.final_success || !immune,
                    }
                {
                    return Err(invalid("Grapple outcome differs from final save/source"));
                }
            }
            GrappleAttemptOutcome::Withdrawn {
                withdrawn_by,
                cancelled,
            } => {
                causal(
                    state,
                    withdrawn_by,
                    &a.declaration.origin,
                    a.declaration.grappler,
                )?;
                super::super::shove::authorize_owner(state, withdrawn_by, a.declaration.grappler)?;
                if a.save.as_ref().is_some_and(|s| s.proof.is_some()) {
                    return Err(invalid("Finalized attempt cannot be withdrawn"));
                }
                if let Some(save) = &a.save {
                    let rules = state.rules.as_ref().unwrap();
                    causal(state, withdrawn_by, &save.chosen_by, a.declaration.grappler)?;
                    if let Some(id) = cancelled {
                        if save.request.is_none()
                            || flow(state)?
                                .save_decisions
                                .iter()
                                .any(|d| d.key == save.key)
                            || *id != save.key.request_id()
                            || rules.cancelled_roll_ids.iter().filter(|p| *p == id).count() != 1
                            || rules.rolls.iter().any(|r| r.request.id == *id)
                        {
                            return Err(invalid("Withdrawal canceled identity differs"));
                        }
                    } else {
                        let evidence = saves::evidence(state, save)?;
                        causal(
                            state,
                            withdrawn_by,
                            evidence.resolved_by(),
                            a.declaration.grappler,
                        )?;
                    }
                } else if cancelled.is_some() {
                    return Err(invalid("Withdrawal invented a request"));
                }
            }
        }
        if a.stage == TacticalGrappleAttemptStage::Complete
            && (a.equipment.before_change.is_some() == a.equipment.after.is_some())
        {
            return Err(invalid(
                "Completed Grapple allowance lacks exactly one use/decline",
            ));
        }
    }
    Ok(())
}

fn validate_grip(state: &CampaignState, grip: &TacticalGrip) -> Result<(), RulesError> {
    validate_evidence(state, &grip.save, &grip.declaration)?;
    let proof = grip
        .save
        .proof
        .as_ref()
        .ok_or_else(|| invalid("Live grip has no final save"))?;
    if grip.established_by != proof.finalized_by
        || grip.work
            != (TacticalWorkKey {
                resolution: grip.declaration.origin.id,
                occurrence: grip.save.key.occurrence,
            })
        || state
            .rules
            .as_ref()
            .ok_or(RulesError::Uninitialized)?
            .entities[&grip.declaration.target]
            .condition_immunities
            .contains(&Condition::Grappled)
    {
        return Err(invalid("Live grip differs from exact failed-save producer"));
    }
    Ok(())
}

fn validate_escape(state: &CampaignState, e: &TacticalGrappleEscape) -> Result<(), RulesError> {
    validate_escape_source(state)?;
    let work = TacticalWorkItem {
        occurrence: e.key.occurrence,
        kind: TacticalWorkKind::GrappleEscapeCheck { grip: e.grip },
    };
    node(state, &work)?;
    let request = saves::escape_request(state, e)?;
    if e.request.as_ref() != Some(&request) || e.selected.is_some() {
        return Err(invalid("Escape request/selection differs"));
    }
    let rules = state.rules.as_ref().ok_or(RulesError::Uninitialized)?;
    if e.stage == TacticalGrappleEscapeStage::Rolling {
        let raw = rules
            .pending
            .as_ref()
            .ok_or_else(|| invalid("Escape raw request absent"))?;
        return super::super::turn_validation::pending(state, raw);
    }
    if e.stage != TacticalGrappleEscapeStage::Complete
        || resolution(state)?.pending.is_some()
        || rules.pending.is_some()
    {
        return Err(invalid("Escape durable stage differs"));
    }
    match e
        .outcome
        .as_ref()
        .ok_or_else(|| invalid("Escape outcome absent"))?
    {
        GrappleEscapeOutcome::Checked {
            resolved_by,
            succeeded,
        } => {
            causal(state, resolved_by, &e.origin, e.actor)?;
            super::super::shove::authorize_owner(state, resolved_by, e.actor)?;
            let mut matches = rules
                .rolls
                .iter()
                .filter(|r| r.request.id == e.key.request_id());
            let raw = matches
                .next()
                .ok_or_else(|| invalid("Escape accepted dice absent"))?;
            if (raw.request.visibility == RollVisibility::Secret
                || raw.result.source == RollSource::Digital)
                && !matches!(
                    raw.accepted_by.issuer,
                    CommandIssuer::Admin | CommandIssuer::System
                )
            {
                return Err(RulesError::Unauthorized);
            }
            if matches.next().is_some()
                || raw.request != request
                || raw.issued_by != e.origin
                || raw.accepted_by != *resolved_by
                || raw.resolved != request.resolve(&raw.result)?
                || raw.purpose
                    != (PendingPurpose::TacticalResolution {
                        encounter: encounter(state)?.id,
                        key: e.key,
                    })
                || rules.cancelled_roll_ids.contains(&request.id)
                || *succeeded
                    != crate::test_outcome::ability_test_success(
                        &raw.resolved,
                        e.difficulty,
                        &rules.house_rules,
                    )?
            {
                return Err(invalid("Escape final result differs from actual check"));
            }
            let end = context(state)?.ends.iter().find(|end| end.grip == e.grip);
            if *succeeded {
                if end.is_none_or(|end| {
                    end.caused_by != *resolved_by
                        || end.cause
                            != (GrappleEndCause::Escaped {
                                roll: e.key,
                                work: TacticalWorkKey {
                                    resolution: e.origin.id,
                                    occurrence: e.key.occurrence,
                                },
                            })
                }) {
                    return Err(invalid("Escape success lacks its exact end"));
                }
            } else if end.is_some() || lifecycle::live_grip(state, e.grip).is_err() {
                return Err(invalid("Failed Escape lost its live grip"));
            }
        }
        GrappleEscapeOutcome::Obsolete {
            ended_by,
            cancelled,
        } => {
            let holder = context(state)?
                .proofs
                .iter()
                .find(|proof| proof.declaration.id == e.grip)
                .ok_or_else(|| invalid("Released Escape grip proof absent"))?
                .declaration
                .grappler;
            causal(state, ended_by, &e.origin, holder)?;
            if *cancelled != Some(e.key.request_id())
                || rules
                    .cancelled_roll_ids
                    .iter()
                    .filter(|id| **id == e.key.request_id())
                    .count()
                    != 1
                || rules
                    .rolls
                    .iter()
                    .any(|r| r.request.id == e.key.request_id())
                || !context(state)?.ends.iter().any(|end| {
                    end.grip == e.grip
                        && end.caused_by == *ended_by
                        && end.cause == GrappleEndCause::Released
                })
            {
                return Err(invalid("Released Escape cancellation differs"));
            }
        }
    }
    Ok(())
}

pub(in crate::tactical) fn validate(state: &CampaignState) -> Result<(), RulesError> {
    admission::supported_context(state)?;
    validate_tactical_grapple_shapes(state).map_err(|e| invalid(&e))?;
    if let Some(live) = state
        .rules
        .as_ref()
        .and_then(|r| r.tactical_grapples.as_ref())
    {
        for grip in &live.active {
            validate_grip(state, grip)?;
            // Only live holders reserve current physical hands. Ended proofs
            // below authenticate history and must not block lawful re-equipping.
            // The physical primitive does not recurse through source anatomy.
            let holder = grip.declaration.grappler;
            let loadout = admission::loadout(state, holder)?;
            crate::tactical_inventory::validate_loadout(state, loadout)
                .map_err(|e| invalid(&e.to_string()))?;
            crate::tactical_hands::EffectiveHands::current(
                state,
                state.rules.as_ref().ok_or(RulesError::Uninitialized)?,
                holder,
            )?
            .validate_loadout(&loadout.hands)?;
        }
    }
    let Some(r) = &flow(state)?.resolution else {
        return Ok(());
    };
    let c = context(state)?;
    for proof in &c.proofs {
        validate_grip(state, proof)?;
    }
    for end in &c.ends {
        let proof = c
            .proofs
            .iter()
            .find(|g| g.declaration.id == end.grip)
            .ok_or_else(|| invalid("End lost its grip proof"))?;
        match end.cause {
            GrappleEndCause::Released => {
                causal(
                    state,
                    &end.caused_by,
                    &proof.established_by,
                    proof.declaration.grappler,
                )?;
                super::super::shove::authorize_owner(
                    state,
                    &end.caused_by,
                    proof.declaration.grappler,
                )?;
            }
            GrappleEndCause::Escaped { .. } if matches!(c.activity.as_ref(), Some(GrappleActivity::Escape(e)) if e.grip == end.grip) =>
                {}
            _ => return Err(invalid("Unsupported core end cause")),
        }
    }
    super::super::work_trace::validate(state)?;
    if r.pending.is_some() && waiting(state) {
        return Err(invalid("Grapple choice has competing dice"));
    }
    let trace = r
        .work_trace
        .as_ref()
        .ok_or_else(|| invalid("Grapple work trace absent"))?;
    for node in &trace.nodes {
        let parent = node
            .parent
            .and_then(|p| trace.nodes.iter().find(|n| n.work.occurrence == p));
        match node.work.kind {
            TacticalWorkKind::BeginGrapple { .. } | TacticalWorkKind::GrappleEscapeCheck { .. }
                if node.parent.is_none() => {}
            TacticalWorkKind::GrappleSave { grip }
                if parent
                    .is_some_and(|p| p.work.kind == TacticalWorkKind::BeginGrapple { grip }) => {}
            TacticalWorkKind::GrappleAfterEquipment { grip }
                if parent.is_some_and(|p| {
                    p.work.kind == TacticalWorkKind::BeginGrapple { grip }
                        || p.work.kind == TacticalWorkKind::GrappleSave { grip }
                }) => {}
            _ => return Err(invalid("Core Grapple work ancestry differs")),
        }
    }
    match c.activity.as_ref() {
        Some(GrappleActivity::Attempt(a)) => validate_attempt(state, a),
        Some(GrappleActivity::Escape(e)) => validate_escape(state, e),
        None => Err(invalid("Core Grapple context lacks activity")),
    }
}
