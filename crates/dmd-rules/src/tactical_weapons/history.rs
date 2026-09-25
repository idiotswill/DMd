use super::*;
use std::collections::HashSet;

fn is_light(purpose: WeaponAttackPurpose) -> bool {
    matches!(
        purpose,
        WeaponAttackPurpose::LightBonus { .. } | WeaponAttackPurpose::Nick { .. }
    )
}
pub(super) fn validate(
    input: &WeaponAttackInput<'_>,
    weapon: &WeaponDefinition,
    mastery: Option<WeaponMastery>,
) -> Result<(), WeaponError> {
    let context = &input.context;
    require(
        input.history.len() <= 128 && context.turn_number > 0,
        "invalid bounded weapon turn history",
    )?;
    let mut commands = HashSet::new();
    let mut light_users = HashSet::new();
    let mut cleave_users = HashSet::new();
    let mut loaded = HashSet::new();
    for receipt in input.history {
        require(
            receipt.origin.campaign_id == input.state.campaign_id()
                && receipt.origin.expected_event_sequence <= input.state.applied_event_sequence
                && receipt.turn_number == context.turn_number
                && receipt.origin.id != context.origin.id
                && commands.insert(receipt.origin.id),
            "stale, foreign or duplicate weapon receipt",
        )?;
        require(
            input.state.entities.contains_key(&receipt.actor)
                && input.state.entities.contains_key(&receipt.target),
            "weapon receipt refers to an unknown actor or target",
        )?;
        require(
            receipt
                .origin
                .actor
                .as_ref()
                .is_none_or(|actor| *actor == AgentRef::Entity(receipt.actor)),
            "weapon receipt origin names a different actor",
        )?;
        let item = input
            .state
            .items
            .get(&receipt.weapon)
            .ok_or_else(|| invalid("receipt weapon identity is missing"))?;
        require(
            item.id == receipt.weapon
                && item.campaign_id == input.state.campaign_id()
                && item.definition_id == receipt.definition_id,
            "receipt weapon definition changed",
        )?;
        let prior = input
            .definitions
            .weapon(&receipt.definition_id)
            .ok_or_else(|| invalid("receipt has an unknown weapon definition"))?;
        for same_window in input
            .history
            .iter()
            .filter(|entry| entry.window.id == receipt.window.id)
        {
            require(
                same_window.window.kind == receipt.window.kind
                    && same_window.actor == receipt.actor,
                "action window identity is shared by different actors or action kinds",
            )?;
        }
        if is_light(receipt.purpose) {
            require(
                light_users.insert(receipt.actor),
                "Light and Nick share one extra attack per turn",
            )?;
        }
        if matches!(receipt.purpose, WeaponAttackPurpose::Cleave { .. }) {
            require(
                cleave_users.insert(receipt.actor),
                "duplicate Cleave use in one turn",
            )?;
        }
        if prior.properties.contains(&WeaponProperty::Loading) {
            require(
                loaded.insert((receipt.window.id, receipt.weapon)),
                "Loading fired twice in one action opportunity",
            )?;
        }
    }
    for prior in input
        .history
        .iter()
        .filter(|entry| entry.window.id == context.window.id)
    {
        require(
            prior.actor == context.actor && prior.window.kind == context.window.kind,
            "current action window conflicts with an accepted receipt",
        )?;
    }
    if weapon.properties.contains(&WeaponProperty::Loading) {
        require(
            !loaded.contains(&(context.window.id, input.choice.weapon)),
            "Loading permits one shot per action opportunity",
        )?;
    }
    let trigger_id = match input.choice.purpose {
        WeaponAttackPurpose::Normal => return Ok(()),
        WeaponAttackPurpose::LightBonus { trigger }
        | WeaponAttackPurpose::Nick { trigger }
        | WeaponAttackPurpose::Cleave { trigger } => trigger,
    };
    let prior = input
        .history
        .iter()
        .find(|receipt| receipt.origin.id == trigger_id)
        .ok_or_else(|| illegal("extra attack has no accepted trigger"))?;
    require(
        prior.actor == context.actor && prior.outcome != WeaponAttackOutcome::Pending,
        "extra attack trigger belongs to another actor or has not resolved",
    )?;
    let prior_weapon = input
        .definitions
        .weapon(&prior.definition_id)
        .ok_or_else(|| invalid("unknown trigger weapon"))?;
    match input.choice.purpose {
        WeaponAttackPurpose::LightBonus { .. } | WeaponAttackPurpose::Nick { .. } => {
            require(
                context.on_actor_turn
                    && prior.on_actor_turn
                    && prior.window.kind == WeaponActionKind::AttackAction
                    && prior.purpose == WeaponAttackPurpose::Normal
                    && prior.weapon != input.choice.weapon
                    && prior_weapon.properties.contains(&WeaponProperty::Light)
                    && weapon.properties.contains(&WeaponProperty::Light)
                    && !light_users.contains(&context.actor),
                "Light needs an own-turn Attack-action trigger and a different Light weapon",
            )?;
            if matches!(input.choice.purpose, WeaponAttackPurpose::Nick { .. }) {
                require(
                    mastery == Some(WeaponMastery::Nick) && context.window == prior.window,
                    "Nick must use its unlocked weapon in the triggering Attack action",
                )?;
            } else {
                require(
                    context.window.kind == WeaponActionKind::BonusAction,
                    "Light extra attack requires the later Bonus Action",
                )?;
            }
        }
        WeaponAttackPurpose::Cleave { .. } => {
            require(
                mastery == Some(WeaponMastery::Cleave)
                    && prior.weapon == input.choice.weapon
                    && prior.delivery == WeaponDelivery::Melee
                    && input.choice.delivery == WeaponDelivery::Melee
                    && matches!(prior.outcome, WeaponAttackOutcome::Hit { .. })
                    && context.target_is_creature
                    && prior.target != input.choice.target
                    && context.window == prior.window
                    && context
                        .distance_from_trigger_target
                        .is_some_and(|distance| distance <= 10)
                    && !cleave_users.contains(&context.actor),
                "Cleave needs the same weapon, a melee hit, and a distinct nearby creature once per turn",
            )?;
            let target = input
                .state
                .entities
                .get(&prior.target)
                .ok_or_else(|| invalid("missing trigger target"))?;
            require(
                matches!(
                    target.kind,
                    EntityKind::Character | EntityKind::Npc | EntityKind::Creature
                ),
                "Cleave trigger must hit a creature",
            )?;
        }
        WeaponAttackPurpose::Normal => unreachable!("normal handled above"),
    }
    Ok(())
}
