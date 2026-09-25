//! Turn expenditure helpers. Trigger/feature legality is established by the encounter
//! resolver before these pure mutations; callers commit only a fully validated clone.
use crate::{RulesError, tactical_conditions::can_act};
use dmd_domain::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TacticalCost {
    Action,
    BonusAction,
    Reaction,
    Free,
}

fn prerequisite(message: &str) -> RulesError {
    RulesError::Prerequisite(message.into())
}

pub fn active_actor(timing: &CombatTiming) -> Result<EntityId, RulesError> {
    timing
        .order
        .get(timing.index)
        .map(|entry| entry.actor)
        .ok_or_else(|| prerequisite("encounter has no current turn"))
}

pub fn spend_cost(
    rules: &mut RulesState,
    actor: EntityId,
    cost: TacticalCost,
) -> Result<(), RulesError> {
    if !can_act(rules, actor)? {
        return Err(prerequisite("actor cannot act"));
    }
    let timing = rules
        .timing
        .as_mut()
        .ok_or_else(|| prerequisite("initiative is not active"))?;
    if !timing.order.iter().any(|entry| entry.actor == actor) {
        return Err(prerequisite("actor is not in this initiative order"));
    }
    if cost != TacticalCost::Reaction && active_actor(timing)? != actor {
        return Err(prerequisite("wait for the actor's turn"));
    }
    match cost {
        TacticalCost::Action if !timing.action_spent => timing.action_spent = true,
        TacticalCost::BonusAction if !timing.bonus_action_spent => timing.bonus_action_spent = true,
        TacticalCost::Reaction if !timing.reactions_spent.contains(&actor) => {
            timing.reactions_spent.push(actor)
        }
        TacticalCost::Free => (),
        _ => return Err(prerequisite("that action budget has already been spent")),
    }
    Ok(())
}

pub fn movement_remaining(
    budget: &TacticalTurnBudget,
    selected: DashSpeed,
    current_speeds: &MovementProfile,
) -> Result<u32, RulesError> {
    let speed = match selected {
        DashSpeed::Speed => Some(current_speeds.walk),
        DashSpeed::Climb => current_speeds.climb,
        DashSpeed::Swim => current_speeds.swim,
        DashSpeed::Fly => current_speeds.fly,
        DashSpeed::Burrow => current_speeds.burrow,
    }
    .ok_or_else(|| prerequisite("actor does not have that special speed"))?;
    if budget.dash_grants.len() > 20 {
        return Err(prerequisite("too many Dash grants in one turn"));
    }
    let grants = budget
        .dash_grants
        .iter()
        .filter(|grant| grant.speed == selected)
        .count() as u32;
    let allowance = speed
        .checked_mul(grants + 1)
        .ok_or_else(|| prerequisite("movement allowance exceeds supported bounds"))?;
    Ok(allowance.saturating_sub(budget.movement_spent))
}

pub fn spend_movement(
    budget: &mut TacticalTurnBudget,
    cost: u32,
    selected: DashSpeed,
    current_speeds: &MovementProfile,
) -> Result<(), RulesError> {
    if cost > movement_remaining(budget, selected, current_speeds)? {
        return Err(prerequisite("not enough movement remains for this path"));
    }
    budget.movement_spent = budget
        .movement_spent
        .checked_add(cost)
        .ok_or_else(|| prerequisite("movement expenditure exceeds supported bounds"))?;
    Ok(())
}

/// Dash retains the chosen speed (SRD p.180); it is not converted into whichever speed
/// is queried later. The cross-mode boundary convention is recorded in ADR 026.
pub fn grant_dash(
    budget: &mut TacticalTurnBudget,
    speed: DashSpeed,
    origin: CommandId,
    current_speeds: &MovementProfile,
) -> Result<(), RulesError> {
    movement_remaining(budget, speed, current_speeds)?;
    if budget.dash_grants.len() >= 20
        || budget
            .dash_grants
            .iter()
            .any(|grant| grant.origin == origin)
    {
        return Err(prerequisite("duplicate or excessive Dash grant"));
    }
    budget.dash_grants.push(DashGrant { speed, origin });
    Ok(())
}

/// Standing and mounting each cost half Speed, rounded down in feet (SRD pp.15,186).
pub fn half_speed_cost(current_speed_units: u32) -> Result<u32, RulesError> {
    if current_speed_units == 0 {
        return Err(prerequisite("Speed is zero"));
    }
    Ok((current_speed_units / 4) * 2)
}

pub fn start_attack_action(
    rules: &mut RulesState,
    budget: &mut TacticalTurnBudget,
    actor: EntityId,
    attacks_granted: u8,
) -> Result<(), RulesError> {
    if attacks_granted == 0 || attacks_granted > 20 || budget.attacks_remaining > 0 {
        return Err(prerequisite("invalid or already active Attack action"));
    }
    spend_cost(rules, actor, TacticalCost::Action)?;
    budget.attacks_remaining = attacks_granted;
    Ok(())
}

pub fn spend_attack(budget: &mut TacticalTurnBudget) -> Result<(), RulesError> {
    budget.attacks_remaining = budget
        .attacks_remaining
        .checked_sub(1)
        .ok_or_else(|| prerequisite("no attacks remain in this Attack action"))?;
    Ok(())
}

/// Slotted reaction spells belong to their own caster's one-slot-per-turn limit.
pub fn spend_slot_turn(
    timing: &mut CombatTiming,
    budget: &mut TacticalTurnBudget,
    caster: EntityId,
) -> Result<(), RulesError> {
    if !timing.order.iter().any(|entry| entry.actor == caster) {
        return Err(prerequisite("caster is not in this encounter"));
    }
    if active_actor(timing)? == caster {
        if timing.slot_spent_this_turn {
            return Err(prerequisite(
                "caster already expended a spell slot this turn",
            ));
        }
        timing.slot_spent_this_turn = true;
    } else {
        if budget.other_slot_casters.contains(&caster) {
            return Err(prerequisite(
                "caster already expended a spell slot this turn",
            ));
        }
        budget.other_slot_casters.push(caster);
    }
    Ok(())
}

/// Called only after ordered end-turn work completes. Start-turn effects can then pause
/// on this new exact turn; resetting again on resume would incorrectly refund resources.
pub fn advance_turn(
    timing: &mut CombatTiming,
    budget: &mut TacticalTurnBudget,
) -> Result<EntityId, RulesError> {
    active_actor(timing)?;
    let turn = timing
        .turn_number
        .checked_add(1)
        .ok_or_else(|| prerequisite("turn counter overflow"))?;
    let index = timing
        .index
        .checked_add(1)
        .ok_or_else(|| prerequisite("turn index overflow"))?;
    let wrapped = index == timing.order.len();
    let round = if wrapped {
        timing
            .round
            .checked_add(1)
            .ok_or_else(|| prerequisite("round counter overflow"))?
    } else {
        timing.round
    };
    let next = if wrapped { 0 } else { index };
    let actor = timing.order[next].actor;
    timing.index = next;
    timing.round = round;
    timing.turn_number = turn;
    timing.action_spent = false;
    timing.bonus_action_spent = false;
    timing.slot_spent_this_turn = false;
    timing.reactions_spent.retain(|spent| *spent != actor);
    *budget = TacticalTurnBudget::default();
    Ok(actor)
}
