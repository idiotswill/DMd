//! FinishSpell keeps source evidence for its still-retained occurrence reads.
use super::*;

fn not_before(later: &CommandMeta, earlier: &CommandMeta) -> bool {
    later.campaign_id == earlier.campaign_id
        && (later.expected_event_sequence > earlier.expected_event_sequence || later == earlier)
        && (later.id != earlier.id || later == earlier)
}

/// Called by the actual FinishSpell immediately before removing the live cast.
/// The temporary duplicate cannot escape that producer or become executable work.
pub(in crate::tactical::grapple::reads) fn retain(
    state: &mut CampaignState,
    meta: &CommandMeta,
    record: &TacticalCasting,
) -> Result<(), RulesError> {
    let cast = record.cast.plan.occurrence;
    let mut referenced = false;
    for cut in &context(state)?.cuts {
        if matches!(cut.key.reader, GrappleReader::AttackAdmission { .. })
            && raw_source_for_admission(state, cut)?
                .spell
                .is_some_and(|spell| spell.cast_occurrence() == cast)
        {
            referenced = true;
        }
    }
    if !referenced {
        return Ok(());
    }
    let r = resolution(state)?;
    let trace = trace(state)?;
    let work = trace
        .nodes
        .iter()
        .find(|node| Some(node.work.occurrence) == trace.active)
        .filter(|node| node.work.kind == (TacticalWorkKind::FinishSpell { cast }))
        .ok_or_else(|| invalid("completed cast lacks its entered FinishSpell producer"))?;
    if r.casts.iter().filter(|live| *live == record).count() != 1
        || context(state)?.completed_casts.len() >= MAX_TACTICAL_CASTS
        || context(state)?
            .completed_casts
            .iter()
            .any(|old| old.record.cast.plan.occurrence == cast)
    {
        return Err(invalid(
            "completed cast source is absent, duplicated or over capacity",
        ));
    }
    let receipt = GrappleCompletedCast {
        record: Box::new(record.clone()),
        work: work_key(state, &work.work)?,
        finished_by: meta.clone(),
    };
    context_mut(state)?.completed_casts.push(receipt);
    Ok(())
}

pub(super) fn validate(state: &CampaignState) -> Result<(), RulesError> {
    let r = resolution(state)?;
    let c = context(state)?;
    let trace = trace(state)?;
    if c.completed_casts.len() > MAX_TACTICAL_CASTS {
        return Err(invalid("completed cast evidence capacity exceeded"));
    }
    for (index, receipt) in c.completed_casts.iter().enumerate() {
        let record = receipt.record.as_ref();
        let plan = &record.cast.plan;
        let cast = plan.occurrence;
        crate::tactical_spells::validate_retained_spell(record)?;
        crate::tactical::casting::validate_actor_source(state, record)?;
        if r.casts.iter().any(|live| live.cast.plan.occurrence == cast)
            || c.completed_casts[..index]
                .iter()
                .any(|old| old.record.cast.plan.occurrence == cast)
            || cast >= r.next_occurrence
            || record.completed.len() != record.targets.len()
            || !matches!(
                record.cast.phase,
                SpellCastPhase::Committed | SpellCastPhase::Released
            )
            || record.cast.started_at > state.clock.now
            || record.cast.started_on_turn > r.turn_number
            || !not_before(&plan.origin, &r.origin)
            || !not_before(&receipt.finished_by, &record.cast.last_operation)
            || !not_before(&receipt.finished_by, &plan.origin)
            || receipt.finished_by.id.0.is_nil()
            || receipt.finished_by.expected_event_sequence > state.applied_event_sequence
            || !flow(state)?
                .combatants
                .iter()
                .any(|actor| actor.actor == plan.choice.actor)
        {
            return Err(invalid(
                "completed cast identity, completion or chronology differs",
            ));
        }
        validate_equipment_change_origin(state, &plan.origin, plan.choice.actor)
            .map_err(|error| invalid(&error))?;
        validate_equipment_change_origin(state, &record.cast.last_operation, plan.choice.actor)
            .map_err(|error| invalid(&error))?;
        validate_equipment_change_origin(state, &receipt.finished_by, plan.choice.actor)
            .map_err(|error| invalid(&error))?;
        authorize(state, &plan.origin, plan.choice.actor)?;
        let mut finishes = trace
            .nodes
            .iter()
            .filter(|node| node.work.kind == (TacticalWorkKind::FinishSpell { cast }));
        let finish = finishes
            .next()
            .ok_or_else(|| invalid("completed cast FinishSpell absent"))?;
        if finishes.next().is_some()
            || receipt.work.resolution != r.origin.id
            || receipt.work.occurrence != finish.work.occurrence
        {
            return Err(invalid("completed cast FinishSpell identity is ambiguous"));
        }
        // Commitment allocates one FinishSpell, then every target program as
        // siblings. Allocation order is not execution order: the frames are LIFO.
        let programs: Vec<_> = trace.nodes.iter().filter(|node| {
            matches!(node.work.kind, TacticalWorkKind::SpellProgram { cast: source, .. } if source == cast)
        }).collect();
        if programs.len() != record.completed.len()
            || record.completed.iter().any(|at| {
                programs
                    .iter()
                    .filter(|node| {
                        node.work.kind == (TacticalWorkKind::SpellProgram { cast, at: *at })
                            && node.parent == finish.parent
                            && node.work.occurrence > finish.work.occurrence
                    })
                    .count()
                    != 1
            })
            || r.attack
                .as_ref()
                .and_then(crate::tactical::attacks::spell_occurrence)
                .is_some_and(|(source, _)| source == cast)
        {
            return Err(invalid(
                "completed cast retains executable work or loses its causal partition",
            ));
        }
        for work in r
            .frames
            .iter()
            .flatten()
            .chain(r.pending.iter().map(|pending| &pending.work))
            .chain(r.failed_save.iter().map(|failed| &failed.pending.work))
        {
            if matches!(work.kind,
                TacticalWorkKind::SpellProgram { cast: source, .. }
                | TacticalWorkKind::FinishSpell { cast: source } if source == cast)
            {
                return Err(invalid("completed cast still owns queued spell work"));
            }
            if let Ok(admission) = admission_for_work(state, work.occurrence)
                && raw_source_for_admission(state, admission)?
                    .spell
                    .is_some_and(|spell| spell.cast_occurrence() == cast)
            {
                return Err(invalid(
                    "completed cast still owns queued attack consequences",
                ));
            }
        }
        let mut referenced = false;
        for admission in c
            .cuts
            .iter()
            .filter(|cut| matches!(cut.key.reader, GrappleReader::AttackAdmission { .. }))
        {
            if !raw_source_for_admission(state, admission)?
                .spell
                .is_some_and(|spell| spell.cast_occurrence() == cast)
            {
                continue;
            }
            referenced = true;
            if !not_before(&receipt.finished_by, &admission.issued_by)
                || c.cuts
                    .iter()
                    .filter(|cut| cut.source_attack == Some(admission.key))
                    .any(|cut| !not_before(&receipt.finished_by, &cut.issued_by))
            {
                return Err(invalid(
                    "completed cast precedes its consuming attack reads",
                ));
            }
        }
        if !referenced {
            return Err(invalid(
                "completed cast has no consuming spell attack admission",
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
