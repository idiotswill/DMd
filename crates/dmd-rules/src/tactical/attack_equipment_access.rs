//! Encounter-local production opt-in. Public rollout remains guarded until the
//! application can authenticate its original command and presentation history.
use super::*;

#[cfg(test)]
#[path = "attack_equipment_access_tests.rs"]
mod tests;

/// Complete consistency inventory, independent of rollout permission. The public
/// kernel/tactical wrappers still reject the marker and every new record first.
pub(super) fn validate_records(state: &CampaignState) -> Result<(), RulesError> {
    validate(state)?;
    let Some(flow) = state.encounter.as_ref().and_then(|e| e.flow.as_ref()) else {
        return Ok(());
    };
    if has_attack_equipment_records(state) && flow.attack_equipment_access.is_none() {
        return Err(invalid("equipment evidence lacks its encounter activation"));
    }
    let mut ids = std::collections::HashSet::new();
    for receipt in &flow.budget.weapon_history {
        if !ids.insert(receipt.origin.id) {
            return Err(invalid("duplicate physical attack receipt"));
        }
        if let Some(image) = &receipt.ground_pickup_before {
            require_origin(state, &receipt.origin)?;
            if !receipt.on_actor_turn
                || receipt.window.kind != WeaponActionKind::AttackAction
                || receipt.purpose != WeaponAttackPurpose::Normal
                || receipt.after_equipment.is_some()
            {
                return Err(invalid(
                    "ground receipt has no ordinary equipment allowance",
                ));
            }
            validate_image(state, image, receipt.actor, &receipt.origin)?;
        }
        if let Some(after) = &receipt.after_equipment {
            require_origin(state, &after.cause.origin)?;
            if let Some(ground) = after
                .applied
                .as_ref()
                .and_then(|a| a.ground_before.as_ref())
            {
                validate_image(state, ground, receipt.actor, &after.chosen_by)?;
            }
        }
    }
    if let Some(resolution) = &flow.resolution {
        if let Some(after) = &resolution.attack_after_equipment {
            require_origin(state, &after.cause.origin)?;
        }
        if let Some(attack) = &resolution.attack
            && let Some(weapon) = attack.weapon()
        {
            let new = is_ground_pickup(weapon.choice.equipment_change)
                || weapon.choice.after_equipment.is_some();
            if new {
                require_origin(state, &attack.origin)?;
            }
            if is_ground_pickup(weapon.choice.equipment_change)
                != weapon.ground_pickup_before.is_some()
                || (weapon.ground_pickup_before.is_some()
                    && weapon.choice.after_equipment.is_some())
            {
                return Err(invalid(
                    "physical before image differs from its declared allowance",
                ));
            }
            let receipts = flow
                .budget
                .weapon_history
                .iter()
                .filter(|r| r.origin.id == attack.origin.id)
                .collect::<Vec<_>>();
            if new
                && (receipts.len() != 1
                    || receipts[0].ground_pickup_before != weapon.ground_pickup_before)
            {
                return Err(invalid(
                    "pending equipment image lacks its unique matching receipt",
                ));
            }
        }
    }
    super::attack_equipment::validate(state)
}

fn validate_image(
    state: &CampaignState,
    image: &AttackGroundPickupBefore,
    actor: EntityId,
    command: &CommandMeta,
) -> Result<(), RulesError> {
    let encounter = encounter(state)?;
    validate_equipment_change_origin(state, &image.equipment.command, actor)
        .map_err(|e| invalid(&e))?;
    validate_equipment_change_origin(state, &image.ground.origin, actor)
        .map_err(|e| invalid(&e))?;
    image.ground.position.validate().map_err(|e| invalid(&e))?;
    if image.item.id.0.is_nil()
        || image.item.id != image.ground.item
        || image.item.campaign_id != state.campaign_id()
        || image.item.quantity != 1
        || image.item.state != ItemState::Intact
        || image.item.custody != Custody::Location(image.location)
        || image.equipment.actor != actor
        || image.encounter != encounter.id
        || image.scene != encounter.scene_id
        || !state
            .scenes
            .get(&image.scene)
            .is_some_and(|scene| scene.location_id == image.location)
        || state
            .items
            .get(&image.item.id)
            .is_none_or(|item| item.definition_id != image.item.definition_id)
        || definitions()?.weapon(&image.item.definition_id).is_none()
        || !encounter.battlefield.bounds.contains(image.ground.position)
        || image.ground_index > 16_384
        || image.equipment.command.expected_event_sequence >= command.expected_event_sequence
        || image.ground.origin.expected_event_sequence >= command.expected_event_sequence
    {
        return Err(invalid("ground before image provenance differs"));
    }
    // A completed receipt is immutable evidence, not a request to restore old
    // custody or hands after another action. Replay authenticates the old cut.
    Ok(())
}

pub(super) fn validate(state: &CampaignState) -> Result<(), RulesError> {
    let Some(encounter) = &state.encounter else {
        return Ok(());
    };
    let Some(flow) = &encounter.flow else {
        return Ok(());
    };
    let Some(access) = &flow.attack_equipment_access else {
        return Ok(());
    };
    let origin = &access.origin;
    if access.version != AttackEquipmentAccessVersion::GroundEquipmentV1
        || flow.version != TacticalExecutionVersion::EncounterReleaseV1.flow_version()
        || !matches!(flow.phase, TacticalPhase::Active | TacticalPhase::Finished)
        || origin.id.0.is_nil()
        || origin.id == flow.origin.id
        || origin.campaign_id != state.campaign_id()
        || !matches!(origin.issuer, CommandIssuer::Admin | CommandIssuer::System)
        || origin.actor.is_some()
        || origin.session_id.is_none_or(|id| id.0.is_nil())
        || origin.expected_event_sequence <= flow.origin.expected_event_sequence
        || origin.expected_event_sequence > state.applied_event_sequence
    {
        return Err(invalid("attack equipment activation lineage differs"));
    }
    Ok(())
}

pub(super) fn require_origin(
    state: &CampaignState,
    origin: &CommandMeta,
) -> Result<(), RulesError> {
    validate(state)?;
    let access = flow(state)?
        .attack_equipment_access
        .as_ref()
        .ok_or_else(|| prerequisite("ground pickup execution is not enabled"))?;
    if origin.campaign_id != access.origin.campaign_id
        || origin.id == access.origin.id
        || origin.expected_event_sequence <= access.origin.expected_event_sequence
    {
        return Err(invalid("equipment command precedes its actual activation"));
    }
    Ok(())
}

pub(super) fn activate(
    state: &mut CampaignState,
    meta: &CommandMeta,
    pack: &RulesPack,
) -> Result<(), RulesError> {
    // Only an ordinary valid pre-activation state can enter this producer. A
    // current forged marker or completed receipt is never a construction API.
    crate::validate_state(state, pack)?;
    validate_tactical_state(state)?;
    if meta.campaign_id != state.campaign_id()
        || meta.expected_event_sequence != state.applied_event_sequence
        || meta.id.0.is_nil()
        || !matches!(meta.issuer, CommandIssuer::Admin | CommandIssuer::System)
        || meta.actor.is_some()
    {
        return Err(RulesError::Unauthorized);
    }
    let table = state
        .table
        .as_ref()
        .ok_or_else(|| prerequisite("equipment activation requires an active table session"))?;
    let session = table
        .active_session
        .as_ref()
        .ok_or_else(|| prerequisite("equipment activation requires an active table session"))?;
    let flow = flow(state)?;
    let rules = state.rules.as_ref().ok_or(RulesError::Uninitialized)?;
    let timing = rules
        .timing
        .as_ref()
        .ok_or_else(|| prerequisite("equipment activation requires a settled active turn"))?;
    if meta.session_id != Some(session.session_id)
        || flow.version != TacticalExecutionVersion::EncounterReleaseV1.flow_version()
        || flow.phase != TacticalPhase::Active
        || flow.attack_equipment_access.is_some()
        || flow.resolution.is_some()
        || !flow.ready.is_empty()
        || flow.budget.attacks_remaining != 0
        || flow.budget.attack_window.is_some()
        || !flow.budget.weapon_history.is_empty()
        || timing.action_spent
        || timing.bonus_action_spent
        || timing.slot_spent_this_turn
        || rules.pending.is_some()
        || rules.permission.is_some()
        || table.pending.is_some()
        || table.roll_context.is_some()
        || rules
            .tactical_effects
            .as_ref()
            .is_some_and(|effects| !effects.pending.is_empty())
        || rules.tactical_creatures.as_ref().is_some_and(|creatures| {
            creatures.runtime.iter().any(|runtime| {
                runtime.routine.is_some()
                    || runtime.recharge.iter().any(|entry| entry.pending.is_some())
            })
        })
        || meta.expected_event_sequence <= flow.origin.expected_event_sequence
        || meta.id == flow.origin.id
    {
        return Err(prerequisite(
            "equipment activation requires a settled unpaid active turn",
        ));
    }
    // No source-control capability, time, resource or physical state changes.
    flow_mut(state)?.attack_equipment_access = Some(Box::new(TacticalAttackEquipmentAccess {
        version: AttackEquipmentAccessVersion::GroundEquipmentV1,
        origin: meta.clone(),
    }));
    validate(state)
}
