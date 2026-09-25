//! A reservation is derived from an unresolved source cast in the shared cursor.
//! It is not an extra resource pool, refund counter, or client-supplied permission.
use super::*;

/// Check the one-slot-per-turn rule before admitting an interrupting cast. The
/// scheduler supplies its actual retained casts (including suspended parents).
/// Committed/Held casts have already paid through the ordinary timing authority;
/// only Casting reserves a not-yet-expended slot for Counterspell's source exception.
pub fn validate_spell_slot_reservation(
    state: &CampaignState,
    plan: &SpellCastPlan,
    retained: &[&SpellCast],
) -> Result<(), RulesError> {
    validate_spell_plan(plan)?;
    let SpellExpenditure::Slot { level } = plan.expenditure else {
        return Ok(());
    };
    let rules = state.rules.as_ref().ok_or(RulesError::Uninitialized)?;
    let timing = rules
        .timing
        .as_ref()
        .ok_or_else(|| unavailable("slotted cast requires an active turn"))?;
    let current = crate::tactical_budget::active_actor(timing)?;
    let flow = state
        .encounter
        .as_ref()
        .and_then(|e| e.flow.as_ref())
        .ok_or_else(|| unavailable("slotted cast requires a tactical turn budget"))?;
    if (current == plan.choice.actor && timing.slot_spent_this_turn)
        || flow.budget.other_slot_casters.contains(&plan.choice.actor)
    {
        return Err(unavailable(
            "caster already expended a spell slot this turn",
        ));
    }
    let caster = rules
        .entities
        .get(&plan.choice.actor)
        .and_then(|e| e.spellcasting.as_ref())
        .ok_or_else(|| unavailable("caster has no spell slots"))?;
    if caster.slots[usize::from(level - 1)] == 0 {
        return Err(unavailable("selected spell slot is exhausted"));
    }
    if retained.len() > 128 {
        return Err(invalid("retained casting depth exceeds capacity"));
    }
    for cast in retained {
        validate_spell_cast(cast)?;
        if cast.phase == SpellCastPhase::Casting
            && cast.plan.choice.actor == plan.choice.actor
            && matches!(cast.plan.expenditure, SpellExpenditure::Slot { .. })
        {
            return Err(unavailable(
                "an unresolved cast reserves this caster's spell slot for the turn",
            ));
        }
    }
    Ok(())
}
