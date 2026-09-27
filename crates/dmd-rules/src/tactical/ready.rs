//! SRD186–187: the paid declaration survives other creatures' turns. Trigger
//! witnessing and response admission are separate from the declaration's text.
use super::turns::*;
use super::*;

fn subject(trigger: &ReadyTrigger) -> Option<ReadySubject> {
    match trigger {
        ReadyTrigger::MovementFinished { subject }
        | ReadyTrigger::AttackFinished { subject }
        | ReadyTrigger::SpellFinished { subject } => Some(*subject),
        ReadyTrigger::Adjudicated { .. } => None,
    }
}

fn trigger_shape(trigger: &ReadyTrigger) -> Result<(), RulesError> {
    if let ReadyTrigger::Adjudicated { description } = trigger
        && (description.trim().is_empty()
            || description.len() > 512
            || description.chars().any(char::is_control))
    {
        return Err(invalid("Ready circumstance must be bounded, readable text"));
    }
    if let Some(ReadySubject::Creature(actor)) = subject(trigger)
        && actor.0.is_nil()
    {
        return Err(invalid("Ready circumstance has no creature identity"));
    }
    Ok(())
}

pub(super) fn declare(
    state: &mut CampaignState,
    meta: &CommandMeta,
    trigger: &ReadyTrigger,
    action: &ReadyAction,
) -> Result<(), RulesError> {
    if flow(state)?.phase != TacticalPhase::Active || flow(state)?.resolution.is_some() {
        return Err(RulesError::Pending);
    }
    let actor = active(state)?;
    authorize(state, meta, actor)?;
    if !TacticalExecutionVersion::from_flow_version(flow(state)?.version)
        .is_some_and(TacticalExecutionVersion::retains_work_ancestry)
    {
        return Err(prerequisite("Ready requires the current tactical executor"));
    }
    trigger_shape(trigger)?;
    if matches!(action, ReadyAction::Spell { .. }) {
        return Err(prerequisite(
            "readied spells require the held-casting admission",
        ));
    }
    // A future circumstance need not be happening or visible now. Do not query
    // the selected subject's existence/position at declaration: that would turn
    // admission into an oracle and forbid waiting for a creature to emerge.
    // Only a later actually perceived milestone can authorize the response.
    if flow(state)?.ready.len() >= MAX_TACTICAL_READY
        || flow(state)?.ready.iter().any(|ready| ready.actor == actor)
    {
        return Err(prerequisite("a Ready declaration is already held"));
    }
    super::falling::require_settled_before_action(state)?;
    let rules = state.rules.as_mut().ok_or(RulesError::Uninitialized)?;
    crate::tactical_budget::spend_cost(rules, actor, crate::tactical_budget::TacticalCost::Action)?;
    crate::kernel::interrupt_rest(rules, actor, state.clock.now);
    let turn = rules
        .timing
        .as_ref()
        .ok_or_else(|| invalid("Ready has no turn"))?
        .turn_number;
    let f = flow_mut(state)?;
    f.budget.movement_progress = None;
    f.budget.movement_origin = None;
    f.ready.push(TacticalReady {
        origin: meta.clone(),
        actor,
        declared_on_turn: turn,
        trigger: trigger.clone(),
        action: action.clone(),
        held_spell: None,
    });
    Ok(())
}

pub(super) fn abandon(
    state: &mut CampaignState,
    meta: &CommandMeta,
    actor: EntityId,
) -> Result<(), RulesError> {
    // A general host override is not the player's voluntary choice. Check the
    // principal before looking up private declarations, including off-turn ones.
    if controller(state, actor).is_some_and(|player| meta.issuer != CommandIssuer::Player(player)) {
        return Err(RulesError::Unauthorized);
    }
    authorize(state, meta, actor)?;
    let f = flow(state)?;
    if f.phase != TacticalPhase::Active || f.resolution.is_some() {
        return Err(RulesError::Pending);
    }
    let ready = f
        .ready
        .iter()
        .find(|ready| ready.actor == actor)
        .ok_or_else(|| prerequisite("No Ready declaration is held."))?;
    if ready.held_spell.is_some() {
        return Err(prerequisite(
            "A held spell requires its source concentration cleanup.",
        ));
    }
    // Removing the declaration never rewinds its paid Action, spends a Reaction,
    // advances time, or changes an unrelated effect/concentration group.
    flow_mut(state)?.ready.retain(|ready| ready.actor != actor);
    Ok(())
}

pub(super) fn expire_owner(
    state: &mut CampaignState,
    _meta: &CommandMeta,
    actor: EntityId,
) -> Result<(), RulesError> {
    if flow(state)?
        .ready
        .iter()
        .any(|ready| ready.actor == actor && ready.held_spell.is_some())
    {
        return Err(invalid(
            "held spell expiry requires its source concentration transition",
        ));
    }
    flow_mut(state)?.ready.retain(|ready| ready.actor != actor);
    Ok(())
}

pub(super) fn validate(state: &CampaignState) -> Result<(), RulesError> {
    let f = flow(state)?;
    if f.ready.len() > MAX_TACTICAL_READY
        || (!f.ready.is_empty()
            && !TacticalExecutionVersion::from_flow_version(f.version)
                .is_some_and(TacticalExecutionVersion::retains_work_ancestry))
        || (f.phase != TacticalPhase::Active && !f.ready.is_empty())
    {
        return Err(invalid("inactive or excessive Ready declarations"));
    }
    let mut actors = std::collections::HashSet::new();
    let mut origins = std::collections::HashSet::new();
    for ready in &f.ready {
        trigger_shape(&ready.trigger)?;
        authorize(state, &ready.origin, ready.actor)?;
        validate_equipment_change_origin(state, &ready.origin, ready.actor)
            .map_err(|error| invalid(&error))?;
        let timing = state
            .rules
            .as_ref()
            .and_then(|rules| rules.timing.as_ref())
            .ok_or_else(|| invalid("Ready lacks initiative timing"))?;
        let current = active(state)?;
        if !actors.insert(ready.actor)
            || !origins.insert(ready.origin.id)
            || !f
                .combatants
                .iter()
                .any(|combatant| combatant.actor == ready.actor)
            || ready.declared_on_turn == 0
            || ready.declared_on_turn > timing.turn_number
            || (ready.actor == current && ready.declared_on_turn != timing.turn_number)
            || (ready.declared_on_turn == timing.turn_number
                && (ready.actor != current || !timing.action_spent))
            || matches!(ready.action, ReadyAction::Spell { .. })
            || ready.held_spell.is_some()
        {
            return Err(invalid(
                "Ready identity, Action payment, phase or source differs",
            ));
        }
        // A declaration expires at the next own Start, including a complete
        // round whose intervening resolutions used no new player command.
        let distance = timing.turn_number - ready.declared_on_turn;
        if distance >= timing.order.len() as u64 {
            return Err(invalid("Ready declaration passed its next own turn"));
        }
    }
    Ok(())
}
