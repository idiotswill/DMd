//! Actor-owned movement methods; geometry/costs are validated on accepted commands.
use dmd_domain::*;

#[cfg(test)]
#[path = "table_opportunity_tests.rs"]
mod tests;

pub(super) fn options(
    read: super::TacticalRead<'_>,
    actor: EntityId,
) -> Result<Option<crate::TableMovementOptions>, String> {
    let state = read.state();
    let Some(encounter) = &state.encounter else {
        return Ok(None);
    };
    let Some(participant) = encounter.participant(actor) else {
        return Ok(None);
    };
    let Some(flow) = &encounter.flow else {
        return Ok(None);
    };
    let Some(rules) = &state.rules else {
        return Ok(None);
    };
    let view = dmd_rules::spatial::project_actor_view(encounter, state, actor)
        .map_err(|error| error.to_string())?;
    let Some(position) = view.position else {
        return Ok(None);
    };
    let speed = |base| {
        dmd_rules::tactical_conditions::effective_speed(rules, actor, base)
            .map_err(|error| error.to_string())
    };
    let mut current = participant.movement.clone();
    current.walk = speed(current.walk)?;
    current.climb = current.climb.map(speed).transpose()?;
    current.swim = current.swim.map(speed).transpose()?;
    current.fly = current.fly.map(speed).transpose()?;
    current.burrow = current.burrow.map(speed).transpose()?;
    let remaining = |kind| {
        dmd_rules::tactical_budget::movement_remaining(&flow.budget, kind, &current)
            .is_ok_and(|remaining| remaining > 0)
    };
    let walk = remaining(DashSpeed::Speed);
    let conditions = dmd_rules::active_conditions(rules, actor);
    let prone = conditions.contains(&Condition::Prone);
    let mut modes = vec![];
    if walk {
        modes.push(MovementMode::Crawl);
        if !prone {
            modes.extend([MovementMode::Walk, MovementMode::Jump]);
        }
    }
    if !prone
        && if current.climb.is_some() {
            remaining(DashSpeed::Climb)
        } else {
            walk
        }
    {
        modes.push(MovementMode::Climb);
    }
    if !prone
        && if current.swim.is_some() {
            remaining(DashSpeed::Swim)
        } else {
            walk
        }
    {
        modes.push(MovementMode::Swim);
    }
    if !prone
        && remaining(DashSpeed::Fly)
        && (current.hover || !conditions.contains(&Condition::Incapacitated))
    {
        modes.push(MovementMode::Fly);
    }
    if !prone && remaining(DashSpeed::Burrow) {
        modes.push(MovementMode::Burrow);
    }
    Ok((!modes.is_empty()).then_some(crate::TableMovementOptions {
        actor,
        position,
        grid_units: if participant.size == CreatureSize::Tiny {
            5
        } else {
            10
        },
        modes,
        self_only_required: matches!(read, super::TacticalRead::Owned(_))
            && dmd_rules::table::grapple_enabled(state)
            && rules.tactical_grapples.as_ref().is_some_and(|grapples|
                grapples.active.iter().any(|grip| grip.declaration.grappler == actor)),
    }))
}

#[cfg(test)]
pub(super) fn opportunity(
    state: &CampaignState,
    window: &TacticalOpportunityWindow,
) -> Result<crate::TableOpportunityView, String> {
    opportunity_read(super::TacticalRead::Ordinary(state), window)
}

pub(super) fn opportunity_read(
    read: super::TacticalRead<'_>,
    window: &TacticalOpportunityWindow,
) -> Result<crate::TableOpportunityView, String> {
    let state = read.state();
    let encounter = state
        .encounter
        .as_ref()
        .ok_or("Opportunity encounter is absent.")?;
    let view = dmd_rules::spatial::project_actor_view(encounter, state, window.reactor)
        .map_err(|error| error.to_string())?;
    let target = crate::TableAttackTarget {
        actor: window.mover,
        label: view
            .contacts
            .iter()
            .find(|contact| {
                contact.entity_id == window.mover
                    && contact.status != dmd_rules::spatial::ContactStatus::Remembered
            })
            .and_then(|contact| contact.label.clone())
            .unwrap_or_else(|| "Located creature".into()),
    };
    let mut weapons = super::attacks::options(read, window.reactor)?;
    if let Some(options) = &mut weapons {
        let hands = read
            .hands(window.reactor)
            .map_err(|error| error.to_string())?;
        hands
            .validate_loadout(&options.hands)
            .map_err(|error| error.to_string())?;
        options.targets = vec![target.clone()];
        options.weapons.retain(|weapon| {
            window.options.iter().any(|option|
            matches!(option.source, TacticalMeleeSource::Weapon { item } if item == weapon.item))
        });
        for weapon in &mut options.weapons {
            weapon.source_features.clear();
            weapon.deliveries = vec![WeaponDelivery::Melee];
            weapon.purposes = vec![WeaponAttackPurpose::Normal];
            weapon.grips.retain(|grip| match grip {
                WeaponGrip::OneHand(hand) => hands.holds(&options.hands, *hand, weapon.item),
                WeaponGrip::TwoHands => hands.can_use_two_hands(&options.hands, weapon.item),
            });
        }
        options.weapons.retain(|weapon| !weapon.grips.is_empty());
    }
    let creature = encounter
        .flow
        .as_ref()
        .and_then(|flow| {
            flow.combatants
                .iter()
                .find(|combatant| combatant.actor == window.reactor)
        })
        .map(|combatant| match &combatant.source {
            TacticalSource::Creature { definition_id } => {
                dmd_rules::tactical_creatures::source_for_actor(
                    state,
                    window.reactor,
                    definition_id,
                )
                .map(Some)
                .map_err(|e| e.to_string())
            }
            TacticalSource::Character => Ok(None),
        })
        .transpose()?
        .flatten();
    let features = window
        .options
        .iter()
        .filter_map(|option| {
            let TacticalMeleeSource::CreatureFeature { feature_id, weapon } = &option.source else {
                return None;
            };
            Some(crate::TableCreatureAttackChoice {
                feature_id: feature_id.clone(),
                weapon: *weapon,
                label: creature
                    .and_then(|source| {
                        source
                            .features
                            .iter()
                            .find(|feature| &feature.id == feature_id)
                    })
                    .map(|feature| feature.name.clone())
                    .unwrap_or_else(|| "Creature attack".into()),
            })
        })
        .collect();
    Ok(crate::TableOpportunityView {
        actor: window.reactor,
        target,
        weapons,
        features,
        unarmed: window
            .options
            .iter()
            .any(|option| option.source == TacticalMeleeSource::Unarmed),
    })
}
