//! Internal movement admission/progress around the sole spatial evaluator.
//! Reaction eligibility and queue mutation live in the tactical resolver. No function
//! here accepts player-authored movement allowances, hidden truth or forced-motion flags.
use dmd_domain::*;

use crate::{RulesError, spatial::*};

fn invalid(message: impl Into<String>) -> RulesError {
    RulesError::Invalid(message.into())
}
fn prerequisite(message: impl Into<String>) -> RulesError {
    RulesError::Prerequisite(message.into())
}
fn encounter(state: &CampaignState) -> Result<&TacticalEncounter, RulesError> {
    state
        .encounter
        .as_ref()
        .ok_or_else(|| prerequisite("No active battlefield."))
}
fn budget(state: &CampaignState) -> Result<&TacticalTurnBudget, RulesError> {
    encounter(state)?
        .flow
        .as_ref()
        .map(|flow| &flow.budget)
        .ok_or_else(|| prerequisite("Initiative has not started."))
}

pub(crate) fn progress(state: &CampaignState) -> Result<TacticalMovementProgress, RulesError> {
    Ok(budget(state)?.movement_progress.clone().unwrap_or_default())
}

fn validate_progress(progress: &TacticalMovementProgress, spent: u32) -> Result<(), RulesError> {
    if progress.walked_runup > spent || progress.walked_runup > 10_000 {
        return Err(invalid("Run-up exceeds accepted movement."));
    }
    if let Some(jump) = progress.jump {
        jump.start.validate().map_err(invalid)?;
        if jump.had_runup != (progress.walked_runup >= 20) {
            return Err(invalid("Jump run-up differs from its retained movement."));
        }
    }
    if let Some(straight) = progress.straight {
        straight.start.validate().map_err(invalid)?;
        straight.end.validate().map_err(invalid)?;
        let distance =
            grid_distance(straight.start, straight.end).map_err(|e| invalid(e.to_string()))?;
        if distance == 0 || distance > spent {
            return Err(invalid("Straight travel exceeds actual movement."));
        }
    }
    Ok(())
}

pub(crate) fn validate_budget_progress(state: &CampaignState) -> Result<(), RulesError> {
    let budget = budget(state)?;
    validate_progress(&progress(state)?, budget.movement_spent)?;
    let actor = state
        .rules
        .as_ref()
        .and_then(|r| r.timing.as_ref())
        .ok_or_else(|| invalid("Movement timing absent."))
        .and_then(crate::tactical_budget::active_actor)?;
    if budget.movement_progress.is_some() != budget.movement_origin.is_some() {
        return Err(invalid(
            "Movement progress lacks its accepted mover command.",
        ));
    }
    if let Some(origin) = &budget.movement_origin {
        validate_equipment_origin(state, origin, actor).map_err(invalid)?;
    }
    if progress(state)?.straight.is_some_and(|straight| {
        encounter(state).is_ok_and(|e| {
            e.participant(actor)
                .is_none_or(|p| p.position != straight.end)
        })
    }) {
        return Err(invalid(
            "Straight movement endpoint changed without ending continuity.",
        ));
    }
    Ok(())
}

/// Proposal shape uses only submitted points and the mover's own grid. It must not
/// inspect the map's remote occupants, solids, support, terrain or path cost.
fn validate_path_shape(
    start: SpatialPoint,
    size: CreatureSize,
    path: &[TacticalMoveStep],
) -> Result<(), RulesError> {
    if path.is_empty() || path.len() > 1024 {
        return Err(invalid(
            "Movement needs between one and 1024 adjacent steps.",
        ));
    }
    let alignment = if size == CreatureSize::Tiny { 5 } else { 10 };
    let mut position = start;
    for step in path {
        step.destination.validate().map_err(invalid)?;
        if step.destination.x.rem_euclid(alignment) != 0
            || step.destination.y.rem_euclid(alignment) != 0
        {
            return Err(prerequisite(
                "Destination is not aligned to this actor's grid.",
            ));
        }
        let distance = grid_distance(position, step.destination)
            .map_err(|error| invalid(error.to_string()))?;
        if distance == 0 || distance > 10 {
            return Err(prerequisite(
                "Movement steps must be distinct adjacent positions.",
            ));
        }
        if step.mode == MovementMode::Teleport {
            return Err(prerequisite("Teleportation requires its source feature."));
        }
        position = step.destination;
    }
    Ok(())
}

/// Own source capability only, distinct from whether a particular place can be
/// reached. A later source interruption rechecks this before the next real step.
pub(crate) fn capability(
    state: &CampaignState,
    actor: EntityId,
    mode: MovementMode,
) -> Result<(), RulesError> {
    let mover = encounter(state)?
        .participant(actor)
        .ok_or_else(|| invalid("Mover is absent."))?;
    let rules = state.rules.as_ref().ok_or(RulesError::Uninitialized)?;
    let conditions = crate::active_conditions(rules, actor);
    if conditions.contains(&Condition::Prone) && mode != MovementMode::Crawl {
        return Err(prerequisite("Prone movement must crawl."));
    }
    let base = match mode {
        MovementMode::Walk | MovementMode::Crawl | MovementMode::Jump => mover.movement.walk,
        MovementMode::Climb => mover.movement.climb.unwrap_or(mover.movement.walk),
        MovementMode::Swim => mover.movement.swim.unwrap_or(mover.movement.walk),
        MovementMode::Fly => mover
            .movement
            .fly
            .ok_or_else(|| prerequisite("Actor has no Fly Speed."))?,
        MovementMode::Burrow => mover
            .movement
            .burrow
            .ok_or_else(|| prerequisite("Actor has no Burrow Speed."))?,
        MovementMode::Teleport => {
            return Err(prerequisite("Teleportation requires its source feature."));
        }
    };
    if crate::tactical_conditions::effective_speed(rules, actor, base)? == 0 {
        return Err(prerequisite(
            "The actor's current source state prevents this movement.",
        ));
    }
    if mode == MovementMode::Fly
        && !mover.movement.hover
        && conditions.contains(&Condition::Incapacitated)
    {
        return Err(prerequisite(
            "Unsupported flight cannot continue while incapacitated.",
        ));
    }
    Ok(())
}

pub(crate) fn validate_result(state: &CampaignState) -> Result<(), RulesError> {
    let Some(flow) = encounter(state)?.flow.as_ref() else {
        return Ok(());
    };
    let Some(result) = &flow.last_movement else {
        return Ok(());
    };
    if !matches!(flow.phase, TacticalPhase::Active | TacticalPhase::Finished) {
        return Err(invalid("Movement result precedes the first combat turn."));
    }
    validate_equipment_origin(state, &result.original, result.actor).map_err(invalid)?;
    validate_equipment_change_origin(state, &result.cause, result.actor).map_err(invalid)?;
    result.start.validate().map_err(invalid)?;
    result.endpoint.validate().map_err(invalid)?;
    let cost = result
        .spent_after
        .checked_sub(result.spent_before)
        .ok_or_else(|| invalid("Movement result refunds accepted expenditure."))?;
    let sequence = result.original.expected_event_sequence;
    if result.requested_steps == 0
        || result.requested_steps > 1024
        || result.completed_steps > result.requested_steps
        || result.spent_after > 10_000
        || result.turn_number == 0
        || state
            .rules
            .as_ref()
            .and_then(|rules| rules.timing.as_ref())
            .is_some_and(|timing| result.turn_number > timing.turn_number)
        || result.cause.expected_event_sequence < sequence
        || (result.cause.expected_event_sequence == sequence && result.cause != result.original)
        || (result.cause.id == result.original.id && result.cause != result.original)
        || cost < u32::from(result.completed_steps)
        || cost > u32::from(result.completed_steps) * 30
        || (result.reason == TacticalMovementEnd::Completed)
            != (result.completed_steps == result.requested_steps)
        || (result.completed_steps == 0
            && result.reason != TacticalMovementEnd::Interrupted
            && result.endpoint != result.start)
        || (result.reason != TacticalMovementEnd::Interrupted
            && grid_distance(result.start, result.endpoint)
                .map_err(|error| invalid(error.to_string()))?
                > u32::from(result.completed_steps) * 10)
    {
        return Err(invalid(
            "Movement result has incompatible source, counts or expenditure.",
        ));
    }
    Ok(())
}

/// Internal source adapter query. The actor must be the current turn's mover; the
/// returned command and geometry are captured before the attack consumes progress.
pub(crate) fn straight_approach(
    state: &CampaignState,
    actor: EntityId,
    target: EntityId,
) -> Result<Option<(CommandMeta, TacticalStraightMovement, u32)>, RulesError> {
    validate_budget_progress(state)?;
    let timing = state
        .rules
        .as_ref()
        .and_then(|r| r.timing.as_ref())
        .ok_or_else(|| invalid("Movement timing absent."))?;
    if crate::tactical_budget::active_actor(timing)? != actor {
        return Ok(None);
    }
    let Some(straight) = progress(state)?.straight else {
        return Ok(None);
    };
    let encounter = encounter(state)?;
    let mover = encounter
        .participant(actor)
        .ok_or_else(|| invalid("Mover absent."))?;
    let target = encounter
        .participant(target)
        .ok_or_else(|| invalid("Movement target absent."))?;
    if !straight_movement_toward(mover, target, &straight).map_err(|e| invalid(e.to_string()))? {
        return Ok(None);
    }
    let origin = budget(state)?
        .movement_origin
        .clone()
        .ok_or_else(|| invalid("Movement source absent."))?;
    Ok(Some((
        origin,
        straight,
        grid_distance(straight.start, straight.end).map_err(|e| invalid(e.to_string()))?,
    )))
}

fn allowance(state: &CampaignState) -> Result<MovementAllowance, RulesError> {
    let budget = budget(state)?;
    let mut dash = DashGrants::default();
    for grant in &budget.dash_grants {
        let count = match grant.speed {
            DashSpeed::Speed => &mut dash.speed,
            DashSpeed::Climb => &mut dash.climb,
            DashSpeed::Swim => &mut dash.swim,
            DashSpeed::Fly => &mut dash.fly,
            DashSpeed::Burrow => &mut dash.burrow,
        };
        *count = count
            .checked_add(1)
            .ok_or_else(|| invalid("Dash capacity exceeded."))?;
    }
    Ok(MovementAllowance {
        spent: budget.movement_spent,
        dash,
        disengaged: budget.disengaged.is_some(),
        // Ordinary public movement can never manufacture source displacement/teleport.
        forced: false,
        teleport_range: None,
        runup: 0,
    })
}

fn evaluate(
    state: &CampaignState,
    actor: EntityId,
    steps: &[TacticalMoveStep],
    ends_move: bool,
) -> Result<MovementPlan, RulesError> {
    if steps.iter().any(|step| step.mode == MovementMode::Teleport) {
        return Err(prerequisite("Teleportation requires its source feature."));
    }
    let path = SpatialPath {
        steps: steps
            .iter()
            .map(|step| MovementStep {
                destination: step.destination,
                mode: step.mode,
            })
            .collect(),
    };
    evaluate_path_progress(
        encounter(state)?,
        state,
        actor,
        &path,
        &allowance(state)?,
        &progress(state)?,
        ends_move,
    )
    .map_err(|error| match error {
        SpatialError::Illegal(message) => prerequisite(message),
        // The cumulative runtime work/movement bound is reached only at this real
        // segment; it must not undo a prefix already accepted in the same attempt.
        SpatialError::Capacity => prerequisite("Movement reached its bounded execution capacity."),
        other => invalid(other.to_string()),
    })
}

pub(crate) fn admit(
    state: &CampaignState,
    meta: &CommandMeta,
    actor: EntityId,
    path: &[TacticalMoveStep],
) -> Result<TacticalMovement, RulesError> {
    if meta.campaign_id != state.campaign_id()
        || meta.expected_event_sequence != state.applied_event_sequence
    {
        return Err(RulesError::Stale);
    }
    let encounter = encounter(state)?;
    let flow = encounter
        .flow
        .as_ref()
        .ok_or_else(|| prerequisite("Initiative has not started."))?;
    if flow.phase != TacticalPhase::Active || flow.resolution.is_some() {
        return Err(RulesError::Pending);
    }
    let timing = state
        .rules
        .as_ref()
        .and_then(|rules| rules.timing.as_ref())
        .ok_or_else(|| prerequisite("Initiative has not started."))?;
    if crate::tactical_budget::active_actor(timing)? != actor {
        return Err(prerequisite(
            "Ordinary movement belongs to the current turn.",
        ));
    }
    let mover = encounter
        .participant(actor)
        .ok_or_else(|| invalid("Mover is absent."))?;
    validate_path_shape(mover.position, mover.size, path)?;
    for step in path {
        capability(state, actor, step.mode)?;
    }
    Ok(TacticalMovement {
        origin: meta.clone(),
        actor,
        path: path.to_vec(),
        initial_position: mover.position,
        initial_spent: flow.budget.movement_spent,
        initial_progress: progress(state)?,
        initial_progress_origin: flow.budget.movement_origin.clone(),
        next_step: 0,
        traversed: vec![],
        offered: vec![],
        decisions: vec![],
        opportunity: None,
    })
}

/// Re-evaluates only the next uncommitted segment using current truth and current
/// source progress. Past movement is neither undone nor recomputed with new conditions.
pub(crate) fn next_segment(
    state: &CampaignState,
    movement: &TacticalMovement,
) -> Result<MovementSegment, RulesError> {
    let index = usize::from(movement.next_step);
    let step = movement
        .path
        .get(index)
        .ok_or_else(|| invalid("Movement cursor is complete."))?;
    let expected_position = movement
        .traversed
        .last()
        .map_or(movement.initial_position, |receipt| receipt.to);
    if encounter(state)?
        .participant(movement.actor)
        .is_none_or(|mover| mover.position != expected_position)
    {
        return Err(prerequisite(
            "Accepted displacement interrupted the remaining movement.",
        ));
    }
    capability(state, movement.actor, step.mode)?;
    let plan = evaluate(
        state,
        movement.actor,
        std::slice::from_ref(step),
        index + 1 == movement.path.len(),
    )?;
    if index + 1 == movement.path.len() && plan.falls_at_end {
        return Err(prerequisite(
            "The remaining destination requires falling resolution.",
        ));
    }
    plan.segments
        .into_iter()
        .next()
        .ok_or_else(|| invalid("Movement has no next segment."))
}

/// Historical cost claims require journal replay; these checks additionally reject
/// impossible identity/cursor/budget images at the earliest retained anchor.
pub(crate) fn validate_history(
    state: &CampaignState,
    movement: &TacticalMovement,
) -> Result<(), RulesError> {
    validate_equipment_origin(state, &movement.origin, movement.actor).map_err(invalid)?;
    if movement.path.is_empty()
        || movement.path.len() > 1024
        || usize::from(movement.next_step) != movement.traversed.len()
        || movement.traversed.len() > movement.path.len()
        || movement.initial_spent > 10_000
        || movement.initial_progress.walked_runup > 10_000
    {
        return Err(invalid("Invalid retained movement bounds/cursor."));
    }
    movement.initial_position.validate().map_err(invalid)?;
    let mover = encounter(state)?
        .participant(movement.actor)
        .ok_or_else(|| invalid("Mover is absent."))?;
    validate_path_shape(movement.initial_position, mover.size, &movement.path)?;
    validate_progress(&movement.initial_progress, movement.initial_spent)?;
    if (movement.initial_progress != TacticalMovementProgress::default())
        != movement.initial_progress_origin.is_some()
    {
        return Err(invalid("Initial movement progress lacks its command."));
    }
    if let Some(origin) = &movement.initial_progress_origin {
        validate_equipment_origin(state, origin, movement.actor).map_err(invalid)?;
    }
    let mut position = movement.initial_position;
    let mut spent = movement.initial_spent;
    let mut progress = movement.initial_progress.clone();
    let mut previous_cause = &movement.origin;
    for (index, step) in movement.path.iter().enumerate() {
        step.destination.validate().map_err(invalid)?;
        if step.mode == MovementMode::Teleport {
            return Err(invalid("Ordinary movement contains a teleport."));
        }
        if let Some(receipt) = movement.traversed.get(index) {
            validate_equipment_change_origin(state, &receipt.cause, movement.actor)
                .map_err(invalid)?;
            let distance =
                grid_distance(receipt.from, receipt.to).map_err(|e| invalid(e.to_string()))?;
            if receipt.cause.expected_event_sequence < movement.origin.expected_event_sequence
                || receipt.cause.expected_event_sequence < previous_cause.expected_event_sequence
                || (receipt.cause.expected_event_sequence == previous_cause.expected_event_sequence
                    && &receipt.cause != previous_cause)
                || (receipt.cause.id == previous_cause.id && &receipt.cause != previous_cause)
                || receipt.from != position
                || receipt.to != step.destination
                || receipt.mode != step.mode
                || receipt.cost == 0
                || receipt.cost > 30
                || distance == 0
                || distance > 10
                || receipt.cost < distance
                || receipt.cost % distance != 0
                || receipt.cost / distance > 3
                || receipt.progress_after.walked_runup > 10_000
            {
                return Err(invalid("Movement receipt differs from its accepted path."));
            }
            spent = spent
                .checked_add(receipt.cost)
                .ok_or_else(|| invalid("Movement budget overflow."))?;
            validate_progress(&receipt.progress_after, spent)?;
            let mut expected_progress = match receipt.mode {
                MovementMode::Walk if receipt.from.z == receipt.to.z => TacticalMovementProgress {
                    walked_runup: if progress.jump.is_some() {
                        distance
                    } else {
                        progress.walked_runup.saturating_add(distance).min(10_000)
                    },
                    jump: None,
                    straight: None,
                },
                MovementMode::Jump if index + 1 != movement.path.len() => {
                    TacticalMovementProgress {
                        walked_runup: progress.walked_runup,
                        jump: Some(progress.jump.unwrap_or(TacticalJumpProgress {
                            start: receipt.from,
                            had_runup: progress.walked_runup >= 20,
                        })),
                        straight: None,
                    }
                }
                _ => TacticalMovementProgress::default(),
            };
            expected_progress.straight = Some(
                append_straight_movement(progress.straight, receipt.from, receipt.to)
                    .map_err(|e| invalid(e.to_string()))?,
            );
            if receipt.progress_after != expected_progress {
                return Err(invalid(
                    "Movement receipt invents continuous jump or run-up progress.",
                ));
            }
            progress = receipt.progress_after.clone();
            position = receipt.to;
            previous_cause = &receipt.cause;
        }
    }
    let encounter = encounter(state)?;
    if encounter
        .participant(movement.actor)
        .is_none_or(|actor| actor.position != position)
        || spent > 10_000
        || budget(state)?.movement_spent != spent
        || self::progress(state)? != progress
        || budget(state)?.movement_origin.as_ref()
            != if movement.traversed.is_empty() {
                movement.initial_progress_origin.as_ref()
            } else {
                Some(&movement.origin)
            }
    {
        return Err(invalid(
            "Movement cursor differs from current position or budget.",
        ));
    }
    Ok(())
}
