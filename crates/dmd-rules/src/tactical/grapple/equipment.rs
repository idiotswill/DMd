use super::*;
use crate::tactical_hands::EffectiveHands;

pub(super) fn write_loadout(
    rules: &mut RulesState,
    loadout: ActorEquipmentLoadout,
) -> Result<(), RulesError> {
    let current = rules
        .tactical_inventory
        .as_mut()
        .and_then(|i| i.loadouts.iter_mut().find(|l| l.actor == loadout.actor))
        .ok_or_else(|| invalid("Grapple physical loadout absent"))?;
    *current = loadout;
    Ok(())
}

pub(in crate::tactical) fn apply_after_equipment(
    state: &mut CampaignState,
    meta: &CommandMeta,
    grip: GrappleId,
    work: TacticalWorkKey,
    operation: AttackEquipmentOperation,
) -> Result<(), RulesError> {
    choose(state, meta, grip, work, Some(operation))
}
pub(in crate::tactical) fn decline_after_equipment(
    state: &mut CampaignState,
    meta: &CommandMeta,
    grip: GrappleId,
    work: TacticalWorkKey,
) -> Result<(), RulesError> {
    choose(state, meta, grip, work, None)
}
fn choose(
    state: &mut CampaignState,
    meta: &CommandMeta,
    grip: GrappleId,
    expected: TacticalWorkKey,
    operation: Option<AttackEquipmentOperation>,
) -> Result<(), RulesError> {
    transaction(state, meta, |next| {
        validate(next)?;
        let a = attempt(next)?;
        if a.declaration.id != grip
            || a.stage != TacticalGrappleAttemptStage::AfterEquipment
            || a.equipment.before_change.is_some()
            || a.equipment.after.is_some()
        {
            return Err(RulesError::Pending);
        }
        let selected = a
            .selected
            .clone()
            .ok_or_else(|| invalid("Grapple equipment selection absent"))?;
        if work_key(next, &selected)? != expected {
            return Err(invalid("Grapple equipment work differs"));
        }
        let actor = a.declaration.grappler;
        let window = a.declaration.window;
        super::super::shove::authorize_owner(next, meta, actor)?;
        let decision = if let Some(operation) = operation {
            let before = admission::loadout(next, actor)?.clone();
            let mut after = before.clone();
            let rules = next.rules.as_ref().ok_or(RulesError::Uninitialized)?;
            let hands = EffectiveHands::current(next, rules, actor)?;
            hands.validate_loadout(&before.hands)?;
            crate::tactical_inventory::validate_loadout(next, &before)
                .map_err(|e| invalid(&e.to_string()))?;
            crate::tactical_weapons::apply_attack_equipment_operation(
                next,
                actor,
                window,
                definitions()?,
                &mut after.hands,
                operation,
                &hands,
            )
            .map_err(|e| prerequisite(&e.to_string()))?;
            after.command = meta.clone();
            write_loadout(next.rules.as_mut().ok_or(RulesError::Uninitialized)?, after)?;
            GrappleEquipmentDecision::Applied {
                chosen_by: meta.clone(),
                work: expected,
                operation,
                equipment_before: Box::new(before),
            }
        } else {
            GrappleEquipmentDecision::Declined {
                chosen_by: meta.clone(),
                work: expected,
            }
        };
        let previous = super::super::work_trace::enter(next, &selected)?;
        let a = attempt_mut(next)?;
        a.equipment.after = Some(decision);
        a.selected = None;
        a.stage = TacticalGrappleAttemptStage::Complete;
        super::super::work_trace::leave(next, previous)?;
        // Core-owned proof/end records remain until this final owned choice.
        // No unrelated consumer can reach this temporary internal boundary.
        pump(next, meta)
    })
}
