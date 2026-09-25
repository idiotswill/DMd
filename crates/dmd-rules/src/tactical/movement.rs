//! Segment execution in the existing tactical queue. This module is attached only
//! after the attack author's shared-resolution checkpoint and sealed OA adapter.
use super::turns::*;
use super::*;
use crate::spatial::{MovementSegment, participant_distance, perceive};

fn current(state: &CampaignState) -> Result<&TacticalMovement, RulesError> {
    resolution(state)?
        .movement
        .as_deref()
        .ok_or_else(|| invalid("Movement work is absent."))
}
fn current_mut(state: &mut CampaignState) -> Result<&mut TacticalMovement, RulesError> {
    resolution_mut(state)?
        .movement
        .as_deref_mut()
        .ok_or_else(|| invalid("Movement work is absent."))
}

pub(super) fn begin(
    state: &mut CampaignState,
    meta: &CommandMeta,
    path: &[TacticalMoveStep],
) -> Result<(), RulesError> {
    let actor = active(state)?;
    authorize(state, meta, actor)?;
    let movement = crate::tactical_movement::admit(state, meta, actor, path)?;
    let turn_number = state
        .rules
        .as_ref()
        .and_then(|r| r.timing.as_ref())
        .ok_or_else(|| invalid("Movement timing is absent."))?
        .turn_number;
    flow_mut(state)?.resolution = Some(Box::new(TacticalResolution {
        origin: meta.clone(),
        turn_actor: actor,
        turn_number,
        boundary: TurnBoundary::Start,
        frames: vec![],
        pending: None,
        failed_save: None,
        legendary_window: None,
        attack: None,
        movement: Some(Box::new(movement)),
        casts: vec![],
        next_occurrence: 0,
    }));
    push_frame(state, vec![TacticalWorkKind::MoveSegment])?;
    pump(state, meta)
}

pub(super) fn selected_opportunity(
    state: &CampaignState,
) -> Result<&TacticalOpportunityWindow, RulesError> {
    current(state)?
        .opportunity
        .as_ref()
        .ok_or_else(|| prerequisite("No opportunity attack is due."))
}

fn options(
    state: &CampaignState,
    movement: &TacticalMovement,
    reactor: EntityId,
    segment: &MovementSegment,
) -> Result<Vec<TacticalMeleeOption>, RulesError> {
    let encounter = encounter(state)?;
    let enemy = encounter
        .participant(reactor)
        .ok_or_else(|| invalid("Reactor is absent."))?;
    let mover = encounter
        .participant(movement.actor)
        .ok_or_else(|| invalid("Mover is absent."))?;
    let rules = state.rules.as_ref().ok_or(RulesError::Uninitialized)?;
    if !enemy.enemies.contains(&movement.actor)
        || flow(state)?.budget.disengaged.is_some()
        || !crate::tactical_conditions::can_act(rules, reactor)?
        || rules
            .timing
            .as_ref()
            .is_none_or(|t| t.reactions_spent.contains(&reactor))
        || !perceive(encounter, state, reactor, movement.actor)
            .map_err(|e| invalid(&e.to_string()))?
            .sees
    {
        return Ok(vec![]);
    }
    let before = participant_distance(enemy, mover).map_err(|e| invalid(&e.to_string()))?;
    let mut after = mover.clone();
    after.position = segment.to;
    let after = participant_distance(enemy, &after).map_err(|e| invalid(&e.to_string()))?;
    Ok(
        super::attacks::opportunity_options_for_crossing(state, reactor, movement.actor, after)?
            .into_iter()
            .filter(|option| before <= option.reach && after > option.reach)
            .collect(),
    )
}

pub(super) fn start(
    state: &mut CampaignState,
    meta: &CommandMeta,
    work: &TacticalWorkItem,
) -> Result<bool, RulesError> {
    match work.kind {
        TacticalWorkKind::MoveSegment => {
            advance_segment(state, meta)?;
            Ok(true)
        }
        TacticalWorkKind::MovementOpportunity { reactor } => {
            let movement = current(state)?.clone();
            let segment = match crate::tactical_movement::next_segment(state, &movement) {
                Ok(segment) => segment,
                Err(RulesError::Prerequisite(_)) => {
                    // Accepted consequences can make the next step illegal. They do
                    // not turn corrupt identity/bounds into a legitimate cancellation.
                    unavailable(state, meta, reactor)?;
                    return Ok(true);
                }
                Err(error) => return Err(error),
            };
            let options = match options(state, &movement, reactor, &segment) {
                Ok(options) => options,
                Err(RulesError::Prerequisite(_)) => {
                    finish_movement(state, meta, TacticalMovementEnd::Stopped)?;
                    return Ok(true);
                }
                Err(error) => return Err(error),
            };
            if !options.is_empty() {
                current_mut(state)?.opportunity = Some(TacticalOpportunityWindow {
                    origin: meta.clone(),
                    reactor,
                    mover: movement.actor,
                    step_index: movement.next_step,
                    from: segment.from,
                    to: segment.to,
                    options,
                });
            } else {
                unavailable(state, meta, reactor)?;
            }
            Ok(true)
        }
        _ => Ok(false),
    }
}

fn unavailable(
    state: &mut CampaignState,
    meta: &CommandMeta,
    reactor: EntityId,
) -> Result<(), RulesError> {
    current_mut(state)?
        .decisions
        .push(TacticalOpportunityDecision {
            reactor,
            origin: meta.clone(),
            kind: TacticalOpportunityDecisionKind::Unavailable,
        });
    Ok(())
}

/// Resolve canceled source windows before asking anyone to order phantom choices.
/// Called by the common pump after nested consequences and before its frame choice.
pub(super) fn prune(state: &mut CampaignState, meta: &CommandMeta) -> Result<(), RulesError> {
    let Some(movement) = resolution(state)?.movement.clone() else {
        return Ok(());
    };
    let queued = resolution(state)?
        .frames
        .iter()
        .flatten()
        .filter_map(|work| match work.kind {
            TacticalWorkKind::MovementOpportunity { reactor } => Some(reactor),
            _ => None,
        })
        .collect::<Vec<_>>();
    if queued.is_empty() {
        return Ok(());
    }
    let segment = match crate::tactical_movement::next_segment(state, &movement) {
        Ok(segment) => Some(segment),
        Err(RulesError::Prerequisite(_)) => None,
        Err(error) => return Err(error),
    };
    for reactor in queued {
        let still_due = match &segment {
            Some(segment) => match options(state, &movement, reactor, segment) {
                Ok(options) => !options.is_empty(),
                // Keep the MoveSegment underneath existing children. It rechecks
                // this unresolved source crossing and stops before departing;
                // do not remove a parent needed by an already accepted attack.
                Err(RulesError::Prerequisite(_)) => false,
                Err(error) => return Err(error),
            },
            None => false,
        };
        if !still_due {
            for frame in &mut resolution_mut(state)?.frames {
                frame.retain(|work| !matches!(work.kind, TacticalWorkKind::MovementOpportunity { reactor: actor } if actor == reactor));
            }
            unavailable(state, meta, reactor)?;
        }
    }
    Ok(())
}

fn advance_segment(state: &mut CampaignState, meta: &CommandMeta) -> Result<(), RulesError> {
    let movement = current(state)?.clone();
    if usize::from(movement.next_step) == movement.path.len() {
        return finish_movement(state, meta, TacticalMovementEnd::Completed);
    }
    let segment = match crate::tactical_movement::next_segment(state, &movement) {
        Ok(segment) => segment,
        Err(RulesError::Prerequisite(_)) => {
            // A reaction's accepted consequences must remain committed even if they
            // make the remaining path impossible. Never roll back damage to finish it.
            let expected = movement
                .traversed
                .last()
                .map_or(movement.initial_position, |receipt| receipt.to);
            let displaced = encounter(state)?
                .participant(movement.actor)
                .is_none_or(|actor| actor.position != expected);
            let capability_lost = crate::tactical_movement::capability(
                state,
                movement.actor,
                movement.path[usize::from(movement.next_step)].mode,
            )
            .is_err();
            return finish_movement(
                state,
                meta,
                if displaced || capability_lost {
                    TacticalMovementEnd::Interrupted
                } else {
                    TacticalMovementEnd::Stopped
                },
            );
        }
        Err(error) => return Err(error),
    };
    let mut actors = encounter(state)?
        .participants
        .iter()
        .map(|p| p.entity_id)
        .collect::<Vec<_>>();
    actors.sort_by_key(|actor| actor.0);
    let mut opportunities = Vec::new();
    for actor in actors {
        let answered = movement.decisions.iter().any(|decision| {
            decision.reactor == actor
                && decision.kind != TacticalOpportunityDecisionKind::Unavailable
        });
        if actor == movement.actor || answered {
            continue;
        }
        let options = match options(state, &movement, actor, &segment) {
            Ok(options) => options,
            Err(RulesError::Prerequisite(_)) => {
                return finish_movement(state, meta, TacticalMovementEnd::Stopped);
            }
            Err(error) => return Err(error),
        };
        if !options.is_empty() {
            opportunities.push(actor);
        }
    }
    if !opportunities.is_empty() {
        let movement = current_mut(state)?;
        for actor in &opportunities {
            if !movement.offered.contains(actor) {
                movement.offered.push(*actor);
            }
            // An automatic cancellation never spends or declines a future eligible
            // opportunity if another reaction restores the ability before departure.
            movement
                .decisions
                .retain(|decision| decision.reactor != *actor);
        }
        // The original step remains below every reaction and its nested consequences.
        push_frame(state, vec![TacticalWorkKind::MoveSegment])?;
        push_frame(
            state,
            opportunities
                .into_iter()
                .map(|reactor| TacticalWorkKind::MovementOpportunity { reactor })
                .collect(),
        )?;
        return Ok(());
    }
    state
        .encounter
        .as_mut()
        .ok_or_else(|| invalid("Battlefield disappeared."))?
        .participants
        .iter_mut()
        .find(|p| p.entity_id == movement.actor)
        .ok_or_else(|| invalid("Mover disappeared."))?
        .position = segment.to;
    // A healed knockout victim may still be resting; actual movement ends that
    // downtime. Offering or declining a reaction without moving does not do so.
    crate::kernel::interrupt_rest(
        state.rules.as_mut().ok_or(RulesError::Uninitialized)?,
        movement.actor,
        state.clock.now,
    );
    let budget = &mut flow_mut(state)?.budget;
    budget.movement_spent = budget
        .movement_spent
        .checked_add(segment.cost)
        .ok_or_else(|| invalid("Movement budget overflow."))?;
    budget.movement_progress = (segment.progress_after != TacticalMovementProgress::default())
        .then_some(segment.progress_after.clone());
    budget.movement_origin = budget
        .movement_progress
        .as_ref()
        .map(|_| movement.origin.clone());
    let movement = current_mut(state)?;
    movement.traversed.push(TacticalMovementReceipt {
        cause: meta.clone(),
        from: segment.from,
        to: segment.to,
        mode: segment.mode,
        cost: segment.cost,
        progress_after: segment.progress_after,
    });
    movement.next_step = movement
        .next_step
        .checked_add(1)
        .ok_or_else(|| invalid("Movement cursor overflow."))?;
    movement.offered.clear();
    movement.decisions.clear();
    push_frame(state, vec![TacticalWorkKind::MoveSegment])?;
    refresh_dodges(state)
}

fn finish_movement(
    state: &mut CampaignState,
    meta: &CommandMeta,
    reason: TacticalMovementEnd,
) -> Result<(), RulesError> {
    let movement = current(state)?.clone();
    let result = TacticalMovementResult {
        original: movement.origin.clone(),
        cause: meta.clone(),
        actor: movement.actor,
        turn_number: resolution(state)?.turn_number,
        start: movement.initial_position,
        endpoint: encounter(state)?
            .participant(movement.actor)
            .ok_or_else(|| invalid("Mover disappeared before movement completed."))?
            .position,
        requested_steps: u16::try_from(movement.path.len())
            .map_err(|_| invalid("Movement path capacity."))?,
        completed_steps: movement.next_step,
        spent_before: movement.initial_spent,
        spent_after: flow(state)?.budget.movement_spent,
        reason,
    };
    let resolution = resolution_mut(state)?;
    resolution.movement = None;
    // Only this movement's future crossings are canceled. Independent attack,
    // casting, concentration and effect children retain their original queue/order.
    for frame in &mut resolution.frames {
        frame.retain(|work| {
            !matches!(
                work.kind,
                TacticalWorkKind::MoveSegment | TacticalWorkKind::MovementOpportunity { .. }
            )
        });
    }
    let flow = flow_mut(state)?;
    flow.last_movement = Some(result);
    if reason != TacticalMovementEnd::Completed {
        flow.budget.movement_progress = None;
        flow.budget.movement_origin = None;
    }
    crate::tactical_movement::validate_result(state)
}

/// The attack adapter calls this before spending any reaction. It cannot accept a
/// caller-authored window: the selected source and crossing belong to this state.
pub(super) fn validate_opportunity(
    state: &CampaignState,
    reactor: EntityId,
    mover: EntityId,
) -> Result<&TacticalOpportunityWindow, RulesError> {
    let movement = current(state)?;
    let window = movement
        .opportunity
        .as_ref()
        .ok_or_else(|| prerequisite("No opportunity attack is due."))?;
    validate_equipment_change_origin(state, &window.origin, reactor).map_err(|e| invalid(&e))?;
    if window.reactor != reactor
        || window.mover != mover
        || mover != movement.actor
        || window.step_index != movement.next_step
        || !movement.offered.contains(&reactor)
        || window.origin.expected_event_sequence < movement.origin.expected_event_sequence
        || resolution(state)?.attack.is_some()
        || resolution(state)?.pending.is_some()
        || resolution(state)?.failed_save.is_some()
        || resolution(state)?.legendary_window.is_some()
    {
        return Err(invalid(
            "Opportunity differs from its live movement crossing.",
        ));
    }
    let segment = crate::tactical_movement::next_segment(state, movement)?;
    if window.from != segment.from
        || window.to != segment.to
        || window.options.is_empty()
        || options(state, movement, reactor, &segment)? != window.options
    {
        return Err(invalid(
            "Opportunity source, geometry or eligibility changed.",
        ));
    }
    Ok(window)
}

pub(super) fn decline(state: &mut CampaignState, meta: &CommandMeta) -> Result<(), RulesError> {
    let window = current(state)?
        .opportunity
        .clone()
        .ok_or_else(|| prerequisite("No opportunity attack is due."))?;
    validate_opportunity(state, window.reactor, window.mover)?;
    authorize(state, meta, window.reactor)?;
    current_mut(state)?
        .decisions
        .push(TacticalOpportunityDecision {
            reactor: window.reactor,
            origin: meta.clone(),
            kind: TacticalOpportunityDecisionKind::Declined,
        });
    current_mut(state)?.opportunity = None;
    pump(state, meta)
}

/// Called only after the source attack adapter has validated and reserved its reaction.
pub(super) fn record_attack(
    state: &mut CampaignState,
    meta: &CommandMeta,
    reactor: EntityId,
) -> Result<(), RulesError> {
    let movement = current_mut(state)?;
    let window = movement
        .opportunity
        .as_ref()
        .ok_or_else(|| invalid("Missing accepted opportunity."))?;
    if window.reactor != reactor
        || movement
            .decisions
            .iter()
            .any(|decision| decision.reactor == reactor)
    {
        return Err(invalid(
            "Opportunity response differs or was already accepted.",
        ));
    }
    movement.decisions.push(TacticalOpportunityDecision {
        reactor,
        origin: meta.clone(),
        kind: TacticalOpportunityDecisionKind::Attack,
    });
    movement.opportunity = None;
    Ok(())
}

pub(super) fn validate(state: &CampaignState) -> Result<(), RulesError> {
    crate::tactical_movement::validate_budget_progress(state)?;
    if let Some(origin) = &flow(state)?.budget.movement_origin {
        authorize(state, origin, active(state)?)?;
    }
    let Some(resolution) = flow(state)?.resolution.as_ref() else {
        return Ok(());
    };
    let Some(movement) = &resolution.movement else {
        if resolution.frames.iter().flatten().any(|work| {
            matches!(
                work.kind,
                TacticalWorkKind::MoveSegment | TacticalWorkKind::MovementOpportunity { .. }
            )
        }) {
            return Err(invalid("Movement frame lacks accepted intent."));
        }
        return Ok(());
    };
    crate::tactical_movement::validate_history(state, movement)?;
    if movement.origin != resolution.origin
        || movement.actor != resolution.turn_actor
        || resolution.boundary != TurnBoundary::Start
        || movement.opportunity.is_some() && resolution.attack.is_some()
        || movement.offered.len() > encounter(state)?.participants.len()
        || movement.decisions.len() > movement.offered.len()
    {
        return Err(invalid("Movement identity or selected decision differs."));
    }
    authorize(state, &movement.origin, movement.actor)?;
    for work in resolution.frames.iter().flatten() {
        if let TacticalWorkKind::MovementOpportunity { reactor } = work.kind {
            if !movement.offered.contains(&reactor) {
                return Err(invalid(
                    "Queued opportunity was never derived for this crossing.",
                ));
            }
            let segment = crate::tactical_movement::next_segment(state, movement)?;
            if options(state, movement, reactor, &segment)?.is_empty() {
                return Err(invalid("Queued opportunity is no longer source-eligible."));
            }
        }
    }
    let mut seen = std::collections::HashSet::new();
    for actor in &movement.offered {
        if !seen.insert(*actor)
            || encounter(state)?.participant(*actor).is_none()
            || *actor == movement.actor
        {
            return Err(invalid("Invalid movement opportunity identity."));
        }
        let queued = resolution
            .frames
            .iter()
            .flatten()
            .filter(|work| {
                matches!(work.kind,
            TacticalWorkKind::MovementOpportunity { reactor } if reactor == *actor)
            })
            .count();
        let selected = usize::from(
            movement
                .opportunity
                .as_ref()
                .is_some_and(|window| window.reactor == *actor),
        );
        let answered = movement
            .decisions
            .iter()
            .filter(|decision| decision.reactor == *actor)
            .count();
        if queued + selected + answered != 1 {
            return Err(invalid(
                "Offered opportunity lacks exactly one live or accepted response.",
            ));
        }
    }
    for decision in &movement.decisions {
        validate_equipment_change_origin(state, &decision.origin, decision.reactor)
            .map_err(|e| invalid(&e))?;
        if decision.kind != TacticalOpportunityDecisionKind::Unavailable {
            authorize(state, &decision.origin, decision.reactor)?;
        }
        if !movement.offered.contains(&decision.reactor)
            || decision.origin.expected_event_sequence < movement.origin.expected_event_sequence
            || (decision.kind == TacticalOpportunityDecisionKind::Attack
                && state
                    .rules
                    .as_ref()
                    .and_then(|r| r.timing.as_ref())
                    .is_none_or(|timing| !timing.reactions_spent.contains(&decision.reactor)))
        {
            return Err(invalid(
                "Opportunity response has incompatible source or reaction cost.",
            ));
        }
    }
    if let Some(window) = &movement.opportunity {
        validate_opportunity(state, window.reactor, window.mover)?;
    }
    let next_steps = resolution
        .frames
        .iter()
        .flatten()
        .filter(|work| matches!(work.kind, TacticalWorkKind::MoveSegment))
        .count();
    if next_steps != 1 {
        return Err(invalid(
            "Movement must retain exactly its next uncommitted segment.",
        ));
    }
    Ok(())
}
