//! Compose source counters with the single central turn cursor. This module never
//! executes a monster weapon, spell, or Multiattack payload on behalf of its controller.
use super::turns::*;
use super::*;
use crate::tactical_creatures::*;

fn current(state: &CampaignState) -> Result<&TacticalCreatures, RulesError> {
    state
        .rules
        .as_ref()
        .and_then(|r| r.tactical_creatures.as_ref())
        .ok_or_else(|| invalid("source creature attachment absent"))
}
fn error(error: CreatureError) -> RulesError {
    invalid(&error.to_string())
}

pub(super) fn boundary(
    state: &mut CampaignState,
    meta: &CommandMeta,
    work: &mut Vec<TacticalWorkKind>,
) -> Result<(), RulesError> {
    let Some(creatures) = state
        .rules
        .as_ref()
        .and_then(|r| r.tactical_creatures.as_ref())
    else {
        return Ok(());
    };
    let r = resolution(state)?;
    let turn = CreatureTurn {
        encounter_id: encounter(state)?.id,
        actor: r.turn_actor,
        number: r.turn_number,
        boundary: r.boundary,
    };
    let origin = r.origin.id;
    let mut actors = creatures
        .profiles
        .iter()
        .filter(|p| encounter(state).is_ok_and(|e| e.participant(p.actor).is_some()))
        .map(|p| p.actor)
        .collect::<Vec<_>>();
    actors.sort_by_key(|id| id.0);
    // The opportunity is AFTER End effects, not among them. Do not pre-filter an
    // Incapacitated creature: an End effect might remove that condition before here.
    if turn.boundary == TurnBoundary::End {
        let mut windows = Vec::new();
        for actor in &actors {
            if *actor != turn.actor
                && source_for_profile(
                    current(state)?
                        .profile(*actor)
                        .ok_or_else(|| invalid("creature profile absent"))?,
                )
                .map_err(error)?
                .legendary_budget
                .is_some()
            {
                windows.push(TacticalWorkKind::LegendaryWindow { actor: *actor });
            }
        }
        push_frame(state, windows)?;
    }
    for actor in actors {
        let hooks = creature_turn_hooks(state, current(state)?, actor, turn).map_err(error)?;
        let mut ids = Vec::new();
        for feature_id in hooks.recharge {
            let offset =
                u16::try_from(work.len()).map_err(|_| invalid("recharge work capacity"))?;
            let occurrence = resolution(state)?
                .next_occurrence
                .checked_add(offset)
                .ok_or_else(|| invalid("recharge occurrence overflow"))?;
            let key = TacticalRollKey {
                origin,
                role: TacticalRollRole::CreatureRecharge,
                subject: actor,
                occurrence,
            };
            ids.push(CreatureRechargeId {
                feature_id: feature_id.clone(),
                request_id: key.request_id(),
            });
            work.push(TacticalWorkKind::CreatureRecharge { actor, feature_id });
        }
        let transition =
            observe_creature_turn(state, current(state)?, meta, actor, turn, ids).map_err(error)?;
        state
            .rules
            .as_mut()
            .ok_or(RulesError::Uninitialized)?
            .tactical_creatures = Some(transition.next);
    }
    Ok(())
}

pub(super) fn recharge<'a>(
    state: &'a CampaignState,
    actor: EntityId,
    feature: &str,
) -> Result<&'a CreatureRechargeTicket, RulesError> {
    current(state)?
        .runtime(actor)
        .and_then(|r| r.recharge.iter().find(|r| r.feature_id == feature))
        .and_then(|r| r.pending.as_ref())
        .ok_or_else(|| invalid("source recharge ticket absent"))
}
pub(super) fn submit_recharge(
    state: &mut CampaignState,
    meta: &CommandMeta,
    actor: EntityId,
    feature_id: &str,
    result: &RollResult,
) -> Result<(), RulesError> {
    let transition = apply_creature_schedule(
        state,
        current(state)?,
        meta,
        &CreatureScheduleOperation::SubmitRecharge {
            actor,
            feature_id: feature_id.into(),
            result: result.clone(),
        },
    )
    .map_err(error)?;
    state
        .rules
        .as_mut()
        .ok_or(RulesError::Uninitialized)?
        .tactical_creatures = Some(transition.next);
    Ok(())
}
pub(super) fn after_turn(state: &mut CampaignState) -> Result<(), RulesError> {
    let Some(frame) = resolution(state)?.frames.last() else {
        return Ok(());
    };
    if frame.is_empty()
        || !frame
            .iter()
            .all(|w| matches!(w.kind, TacticalWorkKind::LegendaryWindow { .. }))
    {
        return Ok(());
    }
    let mut unavailable = Vec::new();
    for work in frame {
        let TacticalWorkKind::LegendaryWindow { actor } = work.kind else {
            unreachable!()
        };
        if !creature_legendary_action_available(state, current(state)?, actor).map_err(error)? {
            unavailable.push(actor);
        }
    }
    // Disengage lasts for the rest of the turn, not through after-turn actions.
    flow_mut(state)?.budget.disengaged = None;
    resolution_mut(state)?.frames.last_mut().ok_or_else(||invalid("legendary frame absent"))?.retain(|w|!matches!(w.kind,TacticalWorkKind::LegendaryWindow{actor} if unavailable.contains(&actor)));
    Ok(())
}
pub(super) fn offer(
    state: &mut CampaignState,
    meta: &CommandMeta,
    work: TacticalWorkItem,
    actor: EntityId,
) -> Result<(), RulesError> {
    if !creature_legendary_action_available(state, current(state)?, actor).map_err(error)? {
        return Ok(());
    }
    flow_mut(state)?.budget.disengaged = None;
    resolution_mut(state)?.legendary_window = Some(TacticalLegendaryWindow {
        work,
        origin: meta.clone(),
    });
    Ok(())
}
pub(super) fn decline(state: &mut CampaignState, meta: &CommandMeta) -> Result<(), RulesError> {
    let window = resolution(state)?
        .legendary_window
        .clone()
        .ok_or_else(|| prerequisite("no Legendary Action decision is due"))?;
    let TacticalWorkKind::LegendaryWindow { actor } = window.work.kind else {
        return Err(invalid("invalid legendary decision"));
    };
    let transition = apply_creature_schedule(
        state,
        current(state)?,
        meta,
        &CreatureScheduleOperation::DeclineLegendaryAction { actor },
    )
    .map_err(error)?;
    state
        .rules
        .as_mut()
        .ok_or(RulesError::Uninitialized)?
        .tactical_creatures = Some(transition.next);
    resolution_mut(state)?.legendary_window = None;
    pump(state, meta)
}
pub(super) fn validate_window(
    state: &CampaignState,
    window: &TacticalLegendaryWindow,
) -> Result<(), RulesError> {
    let r = resolution(state)?;
    let TacticalWorkKind::LegendaryWindow { actor } = window.work.kind else {
        return Err(invalid("invalid legendary work"));
    };
    if r.legendary_window.as_ref() != Some(window)
        || r.pending.is_some()
        || r.failed_save.is_some()
        || state.rules.as_ref().is_some_and(|r| r.pending.is_some())
        || r.boundary != TurnBoundary::End
        || r.turn_actor == actor
        || flow(state)?.budget.disengaged.is_some()
        || !creature_legendary_action_available(state, current(state)?, actor).map_err(error)?
    {
        return Err(invalid("invalid source Legendary Action window"));
    }
    validate_equipment_change_origin(state, &window.origin, actor).map_err(|e| invalid(&e))
}

/// The source attachment has no independent clock. Every retained recharge must
/// appear exactly once in central work, and every participating runtime must have
/// observed the current central boundary before the transition is committed.
pub(super) fn validate(state: &CampaignState) -> Result<(), RulesError> {
    let rules = state.rules.as_ref().ok_or(RulesError::Uninitialized)?;
    let Some(creatures) = &rules.tactical_creatures else {
        return Ok(());
    };
    validate_tactical_creatures(state, creatures).map_err(error)?;
    let encounter = encounter(state)?;
    let flow = flow(state)?;
    let effect_turn = effects(state)?
        .turn
        .ok_or_else(|| invalid("source hook lacks central cursor"))?;
    let turn = CreatureTurn {
        encounter_id: encounter.id,
        actor: effect_turn.actor,
        number: effect_turn.number,
        boundary: effect_turn.boundary,
    };
    let work = flow
        .resolution
        .as_ref()
        .into_iter()
        .flat_map(|r| {
            r.frames
                .iter()
                .flatten()
                .chain(r.pending.iter().map(|p| &p.work))
                .chain(r.failed_save.iter().map(|f| &f.pending.work))
                .chain(r.legendary_window.iter().map(|w| &w.work))
        })
        .collect::<Vec<_>>();
    let mut recharge_keys = std::collections::HashSet::new();
    let mut legendary_actors = std::collections::HashSet::new();
    for item in &work {
        match &item.kind {
            TacticalWorkKind::CreatureRecharge { actor, feature_id } => {
                if !recharge_keys.insert((*actor, feature_id.as_str())) {
                    return Err(invalid("source recharge occurrence duplicated"));
                }
            }
            TacticalWorkKind::LegendaryWindow { actor } if !legendary_actors.insert(*actor) => {
                return Err(invalid("legendary opportunity duplicated"));
            }
            _ => (),
        }
    }
    for runtime in &creatures.runtime {
        if encounter.participant(runtime.actor).is_some() && runtime.observed_turn != Some(turn) {
            return Err(invalid("source creature cursor differs from central turn"));
        }
        for row in &runtime.recharge {
            if row.pending.is_some()
                != recharge_keys.contains(&(runtime.actor, row.feature_id.as_str()))
            {
                return Err(invalid("source recharge omitted from central work"));
            }
            if let Some(record) = &row.last_roll {
                let roll = rules
                    .rolls
                    .iter()
                    .find(|r| r.request.id == record.ticket.request.id)
                    .ok_or_else(|| invalid("source recharge lacks authoritative raw history"))?;
                let PendingPurpose::TacticalResolution { encounter: id, key } = roll.purpose else {
                    return Err(invalid("source recharge has a different roll purpose"));
                };
                if id != record.ticket.turn.encounter_id
                    || key.role != TacticalRollRole::CreatureRecharge
                    || key.subject != runtime.actor
                    || key.origin != record.ticket.origin.id
                    || key.request_id() != record.ticket.request.id
                    || roll.request != record.ticket.request
                    || roll.result != record.result
                    || roll.accepted_by != record.accepted_by
                    || roll.issued_by.expected_event_sequence
                        < record.ticket.origin.expected_event_sequence
                {
                    return Err(invalid(
                        "source recharge differs from authoritative raw history",
                    ));
                }
            }
        }
        if turn.boundary == TurnBoundary::End
            && encounter.participant(runtime.actor).is_some()
            && creature_legendary_action_available(state, creatures, runtime.actor)
                .map_err(error)?
            && !legendary_actors.contains(&runtime.actor)
        {
            return Err(invalid("available after-turn source opportunity omitted"));
        }
    }
    let after_turn = flow.resolution.as_ref().is_some_and(|r| {
        r.legendary_window.is_some()
            || r.frames.last().is_some_and(|f| {
                !f.is_empty()
                    && f.iter()
                        .all(|w| matches!(w.kind, TacticalWorkKind::LegendaryWindow { .. }))
            })
    });
    if after_turn && flow.budget.disengaged.is_some() {
        return Err(invalid("Disengage retained after turn ended"));
    }
    Ok(())
}
