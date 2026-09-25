use super::*;
use crate::tactical_budget::{self as budget, TacticalCost};
use crate::tactical_effects::*;

pub(super) fn active(state: &CampaignState) -> Result<EntityId, RulesError> {
    budget::active_actor(
        state
            .rules
            .as_ref()
            .and_then(|r| r.timing.as_ref())
            .ok_or_else(|| prerequisite("no current turn"))?,
    )
}
pub(super) fn resolution(state: &CampaignState) -> Result<&TacticalResolution, RulesError> {
    flow(state)?
        .resolution
        .as_deref()
        .ok_or_else(|| invalid("missing tactical continuation"))
}
pub(super) fn resolution_mut(
    state: &mut CampaignState,
) -> Result<&mut TacticalResolution, RulesError> {
    flow_mut(state)?
        .resolution
        .as_deref_mut()
        .ok_or_else(|| invalid("missing tactical continuation"))
}
pub(super) fn effects(state: &CampaignState) -> Result<&TacticalEffects, RulesError> {
    state
        .rules
        .as_ref()
        .and_then(|r| r.tactical_effects.as_ref())
        .ok_or_else(|| invalid("missing effect attachment"))
}
pub(super) fn end_space_prone(state: &CampaignState, actor: EntityId) -> Result<bool, RulesError> {
    let encounter = encounter(state)?;
    let participant = encounter
        .participant(actor)
        .ok_or_else(|| invalid("ending actor is absent"))?;
    let entity = state
        .rules
        .as_ref()
        .ok_or(RulesError::Uninitialized)?
        .entities
        .get(&actor)
        .ok_or_else(|| invalid("ending actor mechanics are absent"))?;
    if participant.size == CreatureSize::Tiny
        || entity.condition_immunities.contains(&Condition::Prone)
    {
        return Ok(false);
    }
    let volume = participant.volume().map_err(|error| invalid(&error))?;
    for other in &encounter.participants {
        if other.entity_id != actor
            && other.size.rank() >= participant.size.rank()
            && volume.intersects(other.volume().map_err(|error| invalid(&error))?)
        {
            return Ok(true);
        }
    }
    Ok(false)
}
pub(super) fn effect_operation(
    state: &mut CampaignState,
    meta: &CommandMeta,
    operation: EffectLifecycleOperation,
) -> Result<(), RulesError> {
    let step = state
        .rules
        .as_ref()
        .and_then(|r| r.tactical_effects.as_ref())
        .and_then(|e| e.last_operation.as_ref())
        .filter(|last| last.command.id == meta.id)
        .map_or(Ok(0), |last| {
            last.step
                .checked_add(1)
                .ok_or_else(|| invalid("effect step capacity exceeded"))
        })?;
    let previously_unconscious = state
        .rules
        .as_ref()
        .ok_or(RulesError::Uninitialized)?
        .entities
        .keys()
        .copied()
        .filter(|id| {
            crate::active_conditions(state.rules.as_ref().expect("rules checked"), *id)
                .contains(&Condition::Unconscious)
        })
        .collect::<Vec<_>>();
    let (next, _) = crate::tactical_effect_adapter::apply_effect_operation(
        state,
        meta,
        &EffectLifecycleAction { step, operation },
    )?;
    *state = next;
    let rules = state.rules.as_ref().ok_or(RulesError::Uninitialized)?;
    let mut newly_unconscious = rules
        .entities
        .keys()
        .copied()
        .filter(|id| {
            !previously_unconscious.contains(id)
                && crate::active_conditions(rules, *id).contains(&Condition::Unconscious)
        })
        .collect::<Vec<_>>();
    newly_unconscious.sort_by_key(|id| id.0);
    for actor in newly_unconscious {
        crate::tactical_vitality_adapter::drop_held(state, actor, meta)?;
    }
    refresh_dodges(state)?;
    Ok(())
}
pub(super) fn acknowledge(
    state: &mut CampaignState,
    meta: &CommandMeta,
    ticket: EffectTicketId,
    outcome: EffectTriggerResolution,
) -> Result<(), RulesError> {
    effect_operation(
        state,
        meta,
        EffectLifecycleOperation::ResolveTrigger { ticket, outcome },
    )
}
pub(super) fn push_frame(
    state: &mut CampaignState,
    kinds: Vec<TacticalWorkKind>,
) -> Result<(), RulesError> {
    if kinds.is_empty() {
        return Ok(());
    }
    let resolution = resolution_mut(state)?;
    if resolution.frames.len() >= 128 {
        return Err(invalid("consequence nesting capacity exceeded"));
    }
    let mut frame = Vec::new();
    for kind in kinds {
        if resolution.next_occurrence >= 32_768 {
            return Err(invalid("consequence work capacity exceeded"));
        }
        let occurrence = resolution.next_occurrence;
        resolution.next_occurrence += 1;
        frame.push(TacticalWorkItem { occurrence, kind });
    }
    resolution.frames.push(frame);
    Ok(())
}
pub(super) fn new_effect_work(state: &CampaignState) -> Result<Vec<TacticalWorkKind>, RulesError> {
    let resolution = resolution(state)?;
    Ok(effects(state)?.pending.iter().filter(|ticket| {
        !resolution.frames.iter().flatten().chain(resolution.pending.iter().map(|p| &p.work)).chain(resolution.failed_save.iter().map(|f| &f.pending.work))
            .any(|work| matches!(work.kind, TacticalWorkKind::Effect { ticket: id } if id == ticket.id))
    }).map(|t| TacticalWorkKind::Effect { ticket: t.id }).collect())
}

fn expire_legacy(
    state: &mut CampaignState,
    actor: EntityId,
    number: u64,
    boundary: TurnBoundary,
) -> Result<(), RulesError> {
    let rules = state.rules.as_mut().ok_or(RulesError::Uninitialized)?;
    let expired: Vec<_> = rules
        .effects
        .iter()
        .filter(|e| match e.expires {
            Expiry::AtTime(at) => at <= state.clock.now,
            Expiry::AtTurn {
                actor: owner,
                turn_number,
                boundary: at,
            } => owner == actor && turn_number <= number && at == boundary,
            Expiry::Never => false,
        })
        .map(|e| e.id)
        .collect();
    rules.effects.retain(|e| !expired.contains(&e.id));
    for entity in rules.entities.values_mut() {
        if entity.concentration.is_some_and(|id| expired.contains(&id)) {
            entity.concentration = None;
        }
    }
    Ok(())
}

/// Called once after initiative is committed and once after each completed End frame.
/// The cursor is installed before any request is emitted, never refunded on resume.
pub(super) fn begin_boundary(
    state: &mut CampaignState,
    meta: &CommandMeta,
    boundary: TurnBoundary,
) -> Result<(), RulesError> {
    begin_boundary_from(state, meta, boundary, 0)
}

fn begin_boundary_from(
    state: &mut CampaignState,
    meta: &CommandMeta,
    boundary: TurnBoundary,
    first_occurrence: u16,
) -> Result<(), RulesError> {
    let actor = active(state)?;
    let number = state
        .rules
        .as_ref()
        .and_then(|r| r.timing.as_ref())
        .ok_or_else(|| invalid("turn absent"))?
        .turn_number;
    if flow(state)?.resolution.is_some() {
        return Err(RulesError::Pending);
    }
    flow_mut(state)?.resolution = Some(Box::new(TacticalResolution {
        origin: meta.clone(),
        turn_actor: actor,
        turn_number: number,
        boundary,
        frames: vec![],
        pending: None,
        failed_save: None,
        legendary_window: None,
        attack: None,
        movement: None,
        casts: vec![],
        falls: vec![],
        areas: vec![],
        next_occurrence: first_occurrence,
    }));
    state
        .rules
        .as_mut()
        .ok_or(RulesError::Uninitialized)?
        .tactical_recovery
        .get_or_insert_default();
    if boundary == TurnBoundary::Start {
        flow_mut(state)?.dodges.retain(|d| d.actor != actor);
    }
    expire_legacy(state, actor, number, boundary)?;
    effect_operation(
        state,
        meta,
        EffectLifecycleOperation::Observe(EffectObservation::Time),
    )?;
    effect_operation(
        state,
        meta,
        EffectLifecycleOperation::Observe(EffectObservation::Turn(EffectTurn {
            actor,
            number,
            boundary,
        })),
    )?;
    let mut work = new_effect_work(state)?;
    if boundary == TurnBoundary::End && end_space_prone(state, actor)? {
        work.push(TacticalWorkKind::EndOccupiedSpace { actor });
    }
    let rules = state.rules.as_ref().ok_or(RulesError::Uninitialized)?;
    if boundary == TurnBoundary::Start {
        let entity = &rules.entities[&actor];
        if entity.hp == 0 && !entity.death.dead && !entity.death.stable && entity.uses_death_saves {
            work.push(TacticalWorkKind::DeathSave { actor });
        }
    }
    let mut recoverable = rules
        .tactical_recovery
        .as_ref()
        .into_iter()
        .flat_map(|r| r.iter())
        .filter_map(|(actor, recovery)| recovery.stable.as_ref().map(|s| (*actor, s)))
        .collect::<Vec<_>>();
    recoverable.sort_by_key(|(actor, _)| actor.0);
    for (actor, stable) in recoverable {
        if crate::tactical_damage::stable_wake_at(stable)
            .map_err(|e| invalid(&e.to_string()))?
            .is_some_and(|at| at <= state.clock.now)
        {
            work.push(TacticalWorkKind::RecoverStable { actor });
        }
    }
    super::creature_bridge::boundary(state, meta, &mut work)?;
    push_frame(state, work)?;
    pump(state, meta)
}

pub(super) fn pump(state: &mut CampaignState, meta: &CommandMeta) -> Result<(), RulesError> {
    for _ in 0..32_768 {
        if super::falling::selected(state)?.is_some()
            || resolution(state)?
                .movement
                .as_ref()
                .is_some_and(|m| m.opportunity.is_some())
            || resolution(state)?.attack.as_ref().is_some_and(|a| {
                matches!(
                    a.stage,
                    TacticalAttackStage::KnockoutChoice | TacticalAttackStage::MasteryChoice
                )
            })
        {
            while resolution(state)?.frames.last().is_some_and(Vec::is_empty) {
                resolution_mut(state)?.frames.pop();
            }
        }
        super::movement::prune(state, meta)?;
        if resolution(state)?.pending.is_some()
            || super::falling::selected(state)?.is_some()
            || resolution(state)?.failed_save.is_some()
            || resolution(state)?.legendary_window.is_some()
            || resolution(state)?
                .movement
                .as_ref()
                .is_some_and(|m| m.opportunity.is_some())
            || resolution(state)?.attack.as_ref().is_some_and(|a| {
                matches!(
                    a.stage,
                    TacticalAttackStage::KnockoutChoice | TacticalAttackStage::MasteryChoice
                )
            })
        {
            return Ok(());
        }
        // Ending a group can cancel sibling tickets. They cannot remain a phantom choice.
        super::falling::queue_losses(state, meta)?;
        let live: Vec<_> = effects(state)?.pending.iter().map(|t| t.id).collect();
        let r = resolution_mut(state)?;
        for frame in &mut r.frames {
            frame.retain(|w| !matches!(w.kind, TacticalWorkKind::Effect { ticket } if !live.contains(&ticket)));
        }
        while r.frames.last().is_some_and(Vec::is_empty) {
            r.frames.pop();
        }
        super::creature_bridge::after_turn(state)?;
        let r = resolution_mut(state)?;
        while r.frames.last().is_some_and(Vec::is_empty) {
            r.frames.pop();
        }
        let Some(frame) = r.frames.last() else {
            let boundary = r.boundary;
            let next_occurrence = r.next_occurrence;
            flow_mut(state)?.resolution = None;
            if boundary == TurnBoundary::End {
                let mut next_budget = TacticalTurnBudget::default();
                let rules = state.rules.as_mut().ok_or(RulesError::Uninitialized)?;
                let timing = rules
                    .timing
                    .as_mut()
                    .ok_or_else(|| invalid("turn absent"))?;
                let old_round = timing.round;
                budget::advance_turn(timing, &mut next_budget)?;
                if timing.round != old_round {
                    state.clock.now.0 = state
                        .clock
                        .now
                        .0
                        .checked_add(6)
                        .ok_or_else(|| invalid("world clock overflow"))?;
                }
                flow_mut(state)?.budget = next_budget;
                return begin_boundary_from(state, meta, TurnBoundary::Start, next_occurrence);
            }
            return Ok(());
        };
        if frame.len() > 1 {
            return Ok(());
        }
        let work = resolution_mut(state)?
            .frames
            .last_mut()
            .and_then(Vec::pop)
            .ok_or_else(|| invalid("work disappeared"))?;
        super::continuations::start(state, meta, work)?;
    }
    Err(invalid("consequence execution capacity exceeded"))
}

pub(super) fn choose(
    state: &mut CampaignState,
    meta: &CommandMeta,
    occurrence: u16,
) -> Result<(), RulesError> {
    let after_turn = resolution(state)?.frames.last().is_some_and(|frame| {
        !frame.is_empty()
            && frame
                .iter()
                .all(|w| matches!(w.kind, TacticalWorkKind::LegendaryWindow { .. }))
    });
    if after_turn {
        privileged(meta)?;
    } else {
        authorize(state, meta, resolution(state)?.turn_actor)?;
    }
    if resolution(state)?.pending.is_some()
        || super::falling::selected(state)?.is_some()
        || resolution(state)?.failed_save.is_some()
        || resolution(state)?.legendary_window.is_some()
        || resolution(state)?
            .movement
            .as_ref()
            .is_some_and(|m| m.opportunity.is_some())
        || resolution(state)?.attack.as_ref().is_some_and(|a| {
            matches!(
                a.stage,
                TacticalAttackStage::KnockoutChoice | TacticalAttackStage::MasteryChoice
            )
        })
    {
        return Err(RulesError::Pending);
    }
    let frame = resolution_mut(state)?
        .frames
        .last_mut()
        .ok_or_else(|| invalid("no simultaneous work"))?;
    if frame.len() < 2 {
        return Err(prerequisite("no simultaneous choice is due"));
    }
    let index = frame
        .iter()
        .position(|work| work.occurrence == occurrence)
        .ok_or_else(|| invalid("unknown simultaneous work"))?;
    let work = frame.remove(index);
    super::continuations::start(state, meta, work)?;
    pump(state, meta)
}

pub(super) fn speeds(
    state: &CampaignState,
    actor: EntityId,
) -> Result<MovementProfile, RulesError> {
    let mut speeds = encounter(state)?
        .participant(actor)
        .ok_or_else(|| invalid("movement actor absent"))?
        .movement
        .clone();
    let rules = state.rules.as_ref().ok_or(RulesError::Uninitialized)?;
    speeds.walk = crate::tactical_conditions::effective_speed(rules, actor, speeds.walk)?;
    for value in [
        &mut speeds.climb,
        &mut speeds.swim,
        &mut speeds.fly,
        &mut speeds.burrow,
    ]
    .into_iter()
    .flatten()
    {
        *value = crate::tactical_conditions::effective_speed(rules, actor, *value)?;
    }
    Ok(speeds)
}
pub(super) fn dodge_context(
    state: &CampaignState,
    actor: EntityId,
) -> Result<crate::tactical_conditions::DodgeContext, RulesError> {
    Ok(crate::tactical_conditions::DodgeContext {
        declared: flow(state)?.dodges.iter().any(|d| d.actor == actor),
        effective_speed_units: speeds(state, actor)?.walk,
    })
}
pub(super) fn refresh_dodges(state: &mut CampaignState) -> Result<(), RulesError> {
    if state
        .encounter
        .as_ref()
        .and_then(|e| e.flow.as_ref())
        .is_none()
    {
        return Ok(());
    }
    let mut expired = Vec::new();
    for d in &flow(state)?.dodges {
        if speeds(state, d.actor)?.walk == 0
            || !crate::tactical_conditions::can_act(
                state.rules.as_ref().ok_or(RulesError::Uninitialized)?,
                d.actor,
            )?
        {
            expired.push(d.actor);
        }
    }
    flow_mut(state)?
        .dodges
        .retain(|d| !expired.contains(&d.actor));
    Ok(())
}
pub(super) fn core_action(
    state: &mut CampaignState,
    meta: &CommandMeta,
    action: &TacticalAction,
) -> Result<(), RulesError> {
    if flow(state)?.phase != TacticalPhase::Active || flow(state)?.resolution.is_some() {
        return Err(RulesError::Pending);
    }
    let actor = active(state)?;
    authorize(state, meta, actor)?;
    if matches!(action, TacticalAction::EndTurn) {
        return begin_boundary(state, meta, TurnBoundary::End);
    }
    let current_speeds = speeds(state, actor)?;
    if matches!(action, TacticalAction::StandProne) {
        let rules = state.rules.as_ref().ok_or(RulesError::Uninitialized)?;
        if !rules.entities[&actor].prone
            || rules.entities[&actor].death.dead
            || crate::active_conditions(rules, actor).contains(&Condition::Unconscious)
        {
            return Err(prerequisite("actor cannot stand from Prone"));
        }
        let cost = budget::half_speed_cost(current_speeds.walk)?;
        budget::spend_movement(
            &mut flow_mut(state)?.budget,
            cost,
            DashSpeed::Speed,
            &current_speeds,
        )?;
        state
            .rules
            .as_mut()
            .ok_or(RulesError::Uninitialized)?
            .entities
            .get_mut(&actor)
            .ok_or_else(|| invalid("actor absent"))?
            .prone = false;
        flow_mut(state)?.budget.movement_progress = None;
        flow_mut(state)?.budget.movement_origin = None;
        return Ok(());
    }
    let mut turn_budget = flow(state)?.budget.clone();
    let rules = state.rules.as_mut().ok_or(RulesError::Uninitialized)?;
    match action {
        TacticalAction::Dash { speed } => {
            budget::spend_cost(rules, actor, TacticalCost::Action)?;
            budget::grant_dash(&mut turn_budget, *speed, meta.id, &current_speeds)?;
        }
        TacticalAction::Disengage => {
            budget::spend_cost(rules, actor, TacticalCost::Action)?;
            turn_budget.disengaged = Some(meta.clone());
        }
        TacticalAction::Dodge => {
            budget::spend_cost(rules, actor, TacticalCost::Action)?;
            let number = rules
                .timing
                .as_ref()
                .ok_or_else(|| invalid("turn absent"))?
                .turn_number;
            flow_mut(state)?.dodges.push(TacticalDodge {
                actor,
                origin: meta.clone(),
                declared_on_turn: number,
            });
        }
        TacticalAction::StartAttackAction => {
            // Supported PC class is Fighter 1. Source Multiattack also uses the Attack
            // action (SRD257), but its pinned routine is selected through the creature
            // feature path; this ordinary Attack choice grants one attack.
            budget::start_attack_action(rules, &mut turn_budget, actor, 1)?;
            turn_budget.attack_window = Some(WeaponActionWindow {
                id: meta.id,
                kind: WeaponActionKind::AttackAction,
            });
        }
        _ => return Err(invalid("not a core turn action")),
    }
    turn_budget.movement_progress = None;
    turn_budget.movement_origin = None;
    flow_mut(state)?.budget = turn_budget;
    // These accepted combat actions exceed the Short Rest's permitted downtime
    // (SRD187). EndTurn and standing alone take their earlier paths unchanged.
    crate::kernel::interrupt_rest(
        state.rules.as_mut().ok_or(RulesError::Uninitialized)?,
        actor,
        state.clock.now,
    );
    refresh_dodges(state)
}
