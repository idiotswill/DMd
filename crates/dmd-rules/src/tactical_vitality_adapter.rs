//! Internal attachment of source-derived vitality. Public tactical commands cannot
//! supply a VitalityOperation, defenses, damage amount, or lifecycle acknowledgement.
use crate::{RulesError, tactical_damage::*};
use dmd_domain::*;

fn invalid(message: impl Into<String>) -> RulesError {
    RulesError::Invalid(message.into())
}

pub(crate) fn context(
    state: &CampaignState,
    actor: EntityId,
    origin: VitalityOrigin,
) -> Result<VitalityContext, RulesError> {
    let rules = state.rules.as_ref().ok_or(RulesError::Uninitialized)?;
    if !rules.entities.contains_key(&actor) {
        return Err(invalid("missing vitality actor"));
    }
    let underwater = if let Some(encounter) = &state.encounter {
        if let Some(participant) = encounter.participant(actor) {
            let volume = participant.volume().map_err(invalid)?;
            encounter
                .battlefield
                .terrain
                .iter()
                .any(|terrain| terrain.water && terrain.volume.encloses(volume))
        } else {
            false
        }
    } else {
        false
    };
    Ok(VitalityContext {
        origin,
        now: state.clock.now,
        conditions: crate::active_conditions(rules, actor),
        underwater,
        defenses: VitalityDefenses::default(),
        death_save_mode: RollMode::Normal,
        death_save_bonus: 0,
    })
}

pub(crate) fn validate_attachment(state: &CampaignState) -> Result<(), RulesError> {
    let Some(rules) = &state.rules else {
        return Ok(());
    };
    let Some(recoveries) = &rules.tactical_recovery else {
        return Ok(());
    };
    if recoveries.len() > rules.entities.len() {
        return Err(invalid("too many recovery records"));
    }
    for (actor, recovery) in recoveries {
        let entity = rules
            .entities
            .get(actor)
            .ok_or_else(|| invalid("unknown recovery actor"))?;
        let origins = recovery
            .knockout
            .iter()
            .map(|r| &r.origin)
            .chain(recovery.stable.iter().map(|r| &r.origin))
            .chain(
                recovery
                    .knockout_rest
                    .iter()
                    .flat_map(|r| [&r.knockout_origin, &r.started_by]),
            )
            .collect::<Vec<_>>();
        for origin in &origins {
            validate_equipment_change_origin(state, &origin.command, *actor).map_err(invalid)?;
        }
        // Empty attachment is a versioned opt-in marker; it has no invented provenance.
        if let Some(origin) = origins
            .into_iter()
            .max_by_key(|o| (o.command.expected_event_sequence, o.occurrence))
        {
            let context = context(state, *actor, origin.clone())?;
            validate_recovery(entity, recovery, &context).map_err(|e| invalid(e.to_string()))?;
        } else if recovery != &TacticalRecovery::default() {
            return Err(invalid("unrecognized recovery state"));
        }
        if let Some(rest) = &recovery.knockout_rest
            && !rules.rests.iter().any(|r| {
                r.actor == *actor && r.kind == RestKind::Short && r.started_at == rest.started_at
            })
        {
            return Err(invalid(
                "knockout rest evidence differs from authoritative rest progress",
            ));
        }
    }
    Ok(())
}

pub(crate) fn apply(
    state: &mut CampaignState,
    actor: EntityId,
    origin: VitalityOrigin,
    operation: &VitalityOperation,
) -> Result<VitalityTransition, RulesError> {
    let context = context(state, actor, origin)?;
    let rules = state.rules.as_ref().ok_or(RulesError::Uninitialized)?;
    let entity = rules
        .entities
        .get(&actor)
        .ok_or_else(|| invalid("missing vitality actor"))?;
    let recovery = rules
        .tactical_recovery
        .as_ref()
        .and_then(|r| r.get(&actor))
        .cloned()
        .unwrap_or_default();
    let transition = reduce_vitality(entity, &recovery, &context, operation)
        .map_err(|e| invalid(e.to_string()))?;
    if transition.awaiting_choice {
        return Err(invalid(
            "vitality choice requires its durable source continuation",
        ));
    }
    let rules = state.rules.as_mut().ok_or(RulesError::Uninitialized)?;
    rules.entities.insert(actor, transition.entity.clone());
    let recoveries = rules.tactical_recovery.get_or_insert_default();
    if transition.recovery == TacticalRecovery::default() {
        recoveries.remove(&actor);
    } else {
        recoveries.insert(actor, transition.recovery.clone());
    }
    if transition.entity.death.dead {
        state
            .entities
            .get_mut(&actor)
            .ok_or_else(|| invalid("missing vitality world actor"))?
            .existence = EntityExistence::Dead;
        for character in state
            .characters
            .values_mut()
            .filter(|c| c.entity_id == actor)
        {
            character.status = CharacterStatus::Dead;
        }
    }
    Ok(transition)
}

/// The source says a creature drops what it is holding when it becomes Unconscious
/// (SRD191), including a held shield. No voluntary Utilize action is charged for that
/// involuntary consequence. The exact ItemId/owner survives and custody moves to scene.
pub(crate) fn drop_held(
    state: &mut CampaignState,
    actor: EntityId,
    origin: &CommandMeta,
) -> Result<(), RulesError> {
    if state
        .rules
        .as_ref()
        .and_then(|r| r.tactical_inventory.as_ref())
        .and_then(|inventory| {
            inventory
                .loadouts
                .iter()
                .find(|loadout| loadout.actor == actor)
        })
        .is_none_or(|loadout| {
            !loadout
                .hands
                .hands
                .iter()
                .any(|hand| matches!(hand, HandAssignment::Item(_)))
        })
    {
        return Ok(());
    }
    let encounter = state
        .encounter
        .as_ref()
        .ok_or_else(|| invalid("drop requires tactical location"))?;
    let position = crate::spatial::drop_destination(encounter, actor)
        .map_err(|error| invalid(error.to_string()))?;
    let location = state
        .scenes
        .get(&encounter.scene_id)
        .ok_or_else(|| invalid("drop scene absent"))?
        .location_id;
    let rules = state.rules.as_mut().ok_or(RulesError::Uninitialized)?;
    let Some(loadout) = rules
        .tactical_inventory
        .as_mut()
        .and_then(|i| i.loadouts.iter_mut().find(|l| l.actor == actor))
    else {
        return Ok(());
    };
    let mut dropped = loadout
        .hands
        .hands
        .iter()
        .filter_map(|h| {
            if let HandAssignment::Item(id) = h {
                Some(*id)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    dropped.sort_by_key(|id| id.0);
    dropped.dedup();
    if dropped.is_empty() {
        return Ok(());
    }
    loadout.hands = WeaponLoadout::default();
    loadout.shield = None;
    loadout.command = origin.clone();
    let mechanics = rules
        .entities
        .get_mut(&actor)
        .ok_or_else(|| invalid("drop mechanics absent"))?;
    match &mut mechanics.armor {
        ArmorClass::Armor { shield, .. } | ArmorClass::HeavyArmor { shield, .. } => *shield = false,
        ArmorClass::Fixed(_) => (),
    }
    let flow = state
        .encounter
        .as_mut()
        .and_then(|e| e.flow.as_mut())
        .ok_or_else(|| invalid("drop lacks encounter execution"))?;
    for item in dropped {
        state
            .items
            .get_mut(&item)
            .ok_or_else(|| invalid("held item absent"))?
            .custody = Custody::Location(location);
        flow.ground_items.retain(|entry| entry.item != item);
        flow.ground_items.push(TacticalGroundItem {
            item,
            position,
            origin: origin.clone(),
        });
    }
    if let Some(profile) = state
        .rules
        .as_ref()
        .and_then(|r| r.tactical_creatures.as_ref())
        .and_then(|creatures| creatures.profile(actor))
    {
        let armor = crate::tactical_creature_equipment::creature_current_armor(state, profile)?;
        state
            .rules
            .as_mut()
            .ok_or(RulesError::Uninitialized)?
            .entities
            .get_mut(&actor)
            .ok_or_else(|| invalid("drop mechanics absent"))?
            .armor = armor;
    }
    Ok(())
}
