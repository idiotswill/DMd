use super::*;
use std::collections::HashSet;

fn causal(
    state: &CampaignState,
    meta: &CommandMeta,
    fall: &TacticalFall,
) -> Result<(), RulesError> {
    validate_equipment_change_origin(state, meta, fall.actor).map_err(|e| invalid(&e))?;
    if meta.expected_event_sequence < fall.origin.expected_event_sequence
        || (meta.expected_event_sequence == fall.origin.expected_event_sequence
            && *meta != fall.origin)
        || (meta.id == fall.origin.id && *meta != fall.origin)
    {
        return Err(invalid("Fall command chronology differs."));
    }
    Ok(())
}
fn recorded<'a>(
    state: &'a CampaignState,
    fall: &TacticalFall,
    key: TacticalRollKey,
) -> Result<&'a RecordedRoll, RulesError> {
    if key.origin != resolution(state)?.origin.id
        || key.subject != fall.actor
        || key.occurrence >= resolution(state)?.next_occurrence
    {
        return Err(invalid("Fall roll has a foreign occurrence."));
    }
    let roll = state
        .rules
        .as_ref()
        .ok_or(RulesError::Uninitialized)?
        .rolls
        .iter()
        .find(|r| r.request.id == key.request_id())
        .ok_or_else(|| invalid("Fall raw history is absent."))?;
    if roll.purpose
        != (PendingPurpose::TacticalResolution {
            encounter: encounter(state)?.id,
            key,
        })
    {
        return Err(invalid("Fall history has a different source purpose."));
    }
    causal(state, &roll.issued_by, fall)?;
    causal(state, &roll.accepted_by, fall)?;
    Ok(roll)
}
fn landing(
    state: &CampaignState,
    fall: &TacticalFall,
    landing: &TacticalLiquidLanding,
) -> Result<(), RulesError> {
    causal(state, &landing.accepted_by, fall)?;
    authorize(state, &landing.accepted_by, fall.actor)?;
    let rules = state.rules.as_ref().ok_or(RulesError::Uninitialized)?;
    if !matches!(fall.path.surface, FallSurface::Liquid { .. })
        || rules
            .timing
            .as_ref()
            .is_none_or(|t| !t.reactions_spent.contains(&fall.actor))
    {
        return Err(invalid("Liquid landing lacks its actual Reaction."));
    }
    let roll = rules
        .rolls
        .iter()
        .find(|r| r.request.id == landing.result.request_id)
        .ok_or_else(|| invalid("Liquid landing raw history is absent."))?;
    let PendingPurpose::TacticalResolution { key, .. } = roll.purpose else {
        return Err(invalid("Liquid history has a different purpose."));
    };
    if key.role != TacticalRollRole::LiquidLandingCheck {
        return Err(invalid("Liquid history has a different role."));
    }
    let roll = recorded(state, fall, key)?;
    if roll.result != landing.result
        || roll.issued_by != landing.accepted_by
        || roll.request.roller != Some(fall.actor)
    {
        return Err(invalid(
            "Liquid landing differs from its accepted raw evidence.",
        ));
    }
    // A completed fall can itself change conditions after the earlier check; the
    // immutable recorded request is checked before completion and by semantic replay.
    if !matches!(fall.stage, TacticalFallStage::Complete { .. })
        && roll.request
            != source::liquid_landing_request(
                state,
                fall.actor,
                landing.choice,
                key.request_id(),
                visibility(state, fall.actor),
            )?
    {
        return Err(invalid(
            "Liquid landing request differs from its current source.",
        ));
    }
    Ok(())
}

pub(in crate::tactical) fn validate_work(
    state: &CampaignState,
    work: &TacticalWorkItem,
) -> Result<EntityId, RulesError> {
    let fall = current(
        state,
        index(work).ok_or_else(|| invalid("Work is not falling."))?,
    )?;
    if !matches!(
        (&work.kind, &fall.stage),
        (
            TacticalWorkKind::BeginFall { .. },
            TacticalFallStage::Queued
        ) | (
            TacticalWorkKind::LiquidLandingCheck { .. },
            TacticalFallStage::LandingCheck { .. }
        ) | (
            TacticalWorkKind::FallDamage { .. },
            TacticalFallStage::Damage { .. }
        )
    ) {
        return Err(invalid("Fall work differs from retained progress."));
    }
    Ok(fall.actor)
}

pub(in crate::tactical) fn validate(state: &CampaignState) -> Result<(), RulesError> {
    let resolution = flow(state)?.resolution.as_deref();
    for actor in &encounter(state)?.participants {
        if loss(state, actor.entity_id)?.is_some()
            && resolution.is_none_or(|r| {
                !r.falls.iter().any(|fall| {
                    fall.actor == actor.entity_id
                        && !matches!(fall.stage, TacticalFallStage::Complete { .. })
                })
            })
        {
            return Err(invalid("Unsupported lost flight lacks its queued landing."));
        }
    }
    let Some(r) = resolution else {
        return Ok(());
    };
    if r.falls.len() > 256 {
        return Err(invalid("Too many retained fall occurrences."));
    }
    let work = r
        .frames
        .iter()
        .flatten()
        .chain(r.pending.iter().map(|p| &p.work))
        .chain(r.failed_save.iter().map(|f| &f.pending.work))
        .collect::<Vec<_>>();
    let mut live_actors = HashSet::new();
    let mut damage_ids = HashSet::new();
    let mut landing_ids = HashSet::new();
    selected(state)?;
    for (index, fall) in r.falls.iter().enumerate() {
        validate_equipment_change_origin(state, &fall.origin, fall.actor)
            .map_err(|e| invalid(&e))?;
        if fall.origin.expected_event_sequence < r.origin.expected_event_sequence
            || (fall.origin.expected_event_sequence == r.origin.expected_event_sequence
                && fall.origin != r.origin)
            || (fall.origin.id == r.origin.id && fall.origin != r.origin)
        {
            return Err(invalid("Fall predates its parent resolution."));
        }
        source::fall_distance(&fall.path)?;
        let matching = work
            .iter()
            .filter(|work| super::index(work).is_some_and(|i| usize::from(i) == index))
            .collect::<Vec<_>>();
        let complete = matches!(fall.stage, TacticalFallStage::Complete { .. });
        let choice = matches!(fall.stage, TacticalFallStage::LandingChoice);
        if matching.len() != usize::from(!complete && !choice) {
            return Err(invalid("Fall source work is missing or duplicated."));
        }
        if !complete {
            if !live_actors.insert(fall.actor) {
                return Err(invalid("Actor has multiple live falls."));
            }
            let actor = encounter(state)?
                .participant(fall.actor)
                .ok_or_else(|| invalid("Falling participant absent."))?;
            if actor.position != fall.path.from
                || crate::spatial::fall_destination(encounter(state)?, fall.actor)
                    .map_err(|e| invalid(&e.to_string()))?
                    .as_ref()
                    != Some(&fall.path)
            {
                return Err(invalid(
                    "Retained fall differs from actual authored geometry.",
                ));
            }
        }
        match &fall.cause {
            TacticalFallCause::FlightLost if !complete => {
                if crate::spatial::flight_loss_fall(encounter(state)?, state, fall.actor)
                    .map_err(|e| invalid(&e.to_string()))?
                    .as_ref()
                    != Some(&fall.path)
                {
                    return Err(invalid("Retained fall lacks source flight loss."));
                }
            }
            TacticalFallCause::FlightLost => (),
            TacticalFallCause::Unsupported if !complete => {
                if loss(state, fall.actor)?
                    != Some((TacticalFallCause::Unsupported, fall.path.clone()))
                {
                    return Err(invalid(
                        "Retained fall still has sustaining movement or support.",
                    ));
                }
            }
            TacticalFallCause::Unsupported => (),
            TacticalFallCause::MovementEnd {
                movement,
                step_index,
            } => {
                validate_equipment_origin(state, movement, fall.actor).map_err(|e| invalid(&e))?;
                if movement.expected_event_sequence > fall.origin.expected_event_sequence {
                    return Err(invalid("Fall precedes its original movement."));
                }
                if !complete {
                    let current = r
                        .movement
                        .as_ref()
                        .ok_or_else(|| invalid("Movement fall lacks its accepted parent."))?;
                    let last = current
                        .traversed
                        .last()
                        .ok_or_else(|| invalid("Movement fall precedes real displacement."))?;
                    if current.origin != *movement
                        || current.actor != fall.actor
                        || usize::from(*step_index) + 1 != current.traversed.len()
                        || last.to != fall.path.from
                        || last.cause != fall.origin
                        || matches!(
                            last.mode,
                            MovementMode::Fly
                                | MovementMode::Climb
                                | MovementMode::Swim
                                | MovementMode::Burrow
                        )
                        || (last.mode == MovementMode::Jump
                            && current.traversed.len() != current.path.len())
                    {
                        return Err(invalid(
                            "Movement fall differs from its actual committed step.",
                        ));
                    }
                }
            }
        }
        match &fall.stage {
            TacticalFallStage::Queued => (),
            TacticalFallStage::LandingChoice => {
                if !matches!(fall.path.surface, FallSurface::Liquid { .. })
                    || !source::can_attempt_liquid_landing(state, fall.actor)?
                {
                    return Err(invalid("Retained landing choice is unavailable."));
                }
            }
            TacticalFallStage::LandingCheck { accepted_by, .. } => {
                causal(state, accepted_by, fall)?;
                authorize(state, accepted_by, fall.actor)?;
                if !matches!(fall.path.surface, FallSurface::Liquid { .. })
                    || state
                        .rules
                        .as_ref()
                        .and_then(|r| r.timing.as_ref())
                        .is_none_or(|t| !t.reactions_spent.contains(&fall.actor))
                {
                    return Err(invalid("Landing check lacks its spent Reaction."));
                }
                if let Some(pending) = &r.pending
                    && super::index(&pending.work).is_some_and(|i| usize::from(i) == index)
                    && state
                        .rules
                        .as_ref()
                        .and_then(|r| r.pending.as_ref())
                        .is_none_or(|p| p.issued_by != *accepted_by)
                {
                    return Err(invalid(
                        "Landing check issuer differs from accepted choice.",
                    ));
                }
            }
            TacticalFallStage::Damage { landing: choice }
            | TacticalFallStage::Complete {
                landing: choice, ..
            } => {
                if let Some(choice) = choice {
                    if !landing_ids.insert(choice.result.request_id) {
                        return Err(invalid("Liquid result reused across falls."));
                    }
                    landing(state, fall, choice)?;
                }
            }
        }
        if let TacticalFallStage::Complete {
            damage,
            resolved_by,
            ..
        } = &fall.stage
        {
            causal(state, resolved_by, fall)?;
            if let Some(key) = damage {
                if key.role != TacticalRollRole::FallDamage || !damage_ids.insert(key.request_id())
                {
                    return Err(invalid("Fall damage identity is foreign or reused."));
                }
                let roll = recorded(state, fall, *key)?;
                if roll.accepted_by != *resolved_by
                    || Some(roll.request.clone())
                        != source::fall_damage_request(
                            &fall.path,
                            fall.actor,
                            key.request_id(),
                            visibility(state, fall.actor),
                        )?
                {
                    return Err(invalid(
                        "Fall damage differs from its source distance or acceptance.",
                    ));
                }
            } else if !dead(state, fall.actor)? && source::fall_distance(&fall.path)? >= 20 {
                return Err(invalid(
                    "A living full-height fall lacks accepted raw damage.",
                ));
            }
        }
    }
    Ok(())
}
