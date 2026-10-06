use super::*;
use crate::spatial::{MovementAllowance, MovementStep, SpatialError, SpatialPath};

pub(super) fn validate_direction(
    state: &CampaignState,
    destination: SpatialPoint,
) -> Result<(), RulesError> {
    destination.validate().map_err(|e| invalid(&e))?;
    let s = current(state)?;
    let e = encounter(state)?;
    let actor = e
        .participant(s.actor)
        .ok_or_else(|| invalid("Shove actor absent"))?;
    let target = e
        .participant(s.target)
        .ok_or_else(|| invalid("Shove target absent"))?;
    let delta = [
        destination.x - target.position.x,
        destination.y - target.position.y,
        destination.z - target.position.z,
    ];
    let mut after = target.clone();
    after.position = destination;
    if delta.iter().any(|d| ![-10, 0, 10].contains(d))
        || crate::spatial::grid_distance(target.position, destination)
            .map_err(|e| invalid(&e.to_string()))?
            != 10
        || crate::spatial::participant_distance(actor, &after)
            .map_err(|e| invalid(&e.to_string()))?
            <= crate::spatial::participant_distance(actor, target)
                .map_err(|e| invalid(&e.to_string()))?
    {
        return Err(prerequisite(
            "Choose one grid step of five feet away from the shover.",
        ));
    }
    Ok(())
}

/// This internal classification is Host-only. Every proposed Push has identical
/// owner-facing staging; no hidden path/trait determines a displayed option.
pub(super) fn path(
    state: &CampaignState,
    intent: &TacticalShovePush,
) -> Result<Result<crate::spatial::MovementPlan, SpatialError>, RulesError> {
    let s = current(state)?;
    let rules = state.rules.as_ref().ok_or(RulesError::Uninitialized)?;
    if crate::active_conditions(rules, s.target).contains(&Condition::Grappled)
        || crate::tactical_effect_adapter::condition_effects(rules)
            .any(|e| e.source == s.target && e.condition == Some(Condition::Grappled))
    {
        return Err(prerequisite(
            "This coupled motion requires its supported source continuation.",
        ));
    }
    validate_direction(state, intent.destination)?;
    let e = encounter(state)?;
    let target = e
        .participant(s.target)
        .ok_or_else(|| invalid("Shove target absent"))?;
    if target.position != intent.from {
        return Err(invalid("Push origin moved before its ruling"));
    }
    // Forced motion has no locomotion permission. Walk checks solid volume/sweep;
    // its falls_after flag is deliberately not used to suppress genuine flight.
    Ok(crate::spatial::evaluate_path_progress(
        e,
        state,
        s.target,
        &SpatialPath {
            steps: vec![MovementStep {
                destination: intent.destination,
                mode: MovementMode::Walk,
            }],
        },
        &MovementAllowance {
            forced: true,
            ..MovementAllowance::default()
        },
        &TacticalMovementProgress::default(),
        true,
    ))
}

pub(in crate::tactical) fn choose_outcome(
    state: &mut CampaignState,
    meta: &CommandMeta,
    choice: &ShoveChoice,
) -> Result<(), RulesError> {
    require_execution(state)?;
    let s = current(state)?;
    if s.stage != TacticalShoveStage::OutcomeChoice {
        return Err(RulesError::Pending);
    }
    authorize_owner(state, meta, s.actor)?;
    let selected = s
        .selected
        .clone()
        .ok_or_else(|| invalid("Shove outcome lacks selected work"))?;
    match choice {
        ShoveChoice::Push { destination } => {
            validate_direction(state, *destination)?;
            let from = encounter(state)?
                .participant(s.target)
                .ok_or_else(|| invalid("Shove target absent"))?
                .position;
            let s = current_mut(state)?;
            s.push = Some(TacticalShovePush {
                chosen_by: meta.clone(),
                from,
                destination: *destination,
                convention: ShoveGeometryConvention::GridFiveFootAwayV1,
            });
            s.stage = TacticalShoveStage::PushReview;
            // Physical route/trait evaluation waits for the same Host stage for
            // every intent, including a completely clear ordinary map.
            Ok(())
        }
        ShoveChoice::Prone => {
            let target = s.target;
            let immune = state
                .rules
                .as_ref()
                .ok_or(RulesError::Uninitialized)?
                .entities
                .get(&target)
                .ok_or_else(|| invalid("Shove target mechanics absent"))?
                .condition_immunities
                .contains(&Condition::Prone);
            let previous = super::super::work_trace::enter(state, &selected)?;
            if !immune {
                state
                    .rules
                    .as_mut()
                    .ok_or(RulesError::Uninitialized)?
                    .entities
                    .get_mut(&target)
                    .ok_or_else(|| invalid("Shove target mechanics absent"))?
                    .prone = true;
            }
            let s = current_mut(state)?;
            s.selected = None;
            s.stage = TacticalShoveStage::Resolving;
            s.effect = Some(TacticalShoveEffect::Prone {
                chosen_by: meta.clone(),
                applied: !immune,
            });
            push_frame(state, vec![TacticalWorkKind::FinishShove])?;
            if !immune {
                queue_fall(state, meta, &selected)?;
            }
            super::super::work_trace::leave(state, previous)?;
            refresh_dodges(state)?;
            pump(state, meta)
        }
    }
}

pub(in crate::tactical) fn rule_push(
    state: &mut CampaignState,
    meta: &CommandMeta,
    ruling: ShoveGeometryRuling,
) -> Result<(), RulesError> {
    require_execution(state)?;
    privileged(meta)?;
    let s = current(state)?;
    if s.stage != TacticalShoveStage::PushReview {
        return Err(RulesError::Pending);
    }
    let selected = s
        .selected
        .clone()
        .ok_or_else(|| invalid("Push review lacks selected work"))?;
    let intent = s
        .push
        .clone()
        .ok_or_else(|| invalid("Push review lacks player intent"))?;
    if ruling == ShoveGeometryRuling::ReturnToShover {
        let s = current_mut(state)?;
        s.push = None;
        s.stage = TacticalShoveStage::OutcomeChoice;
        return Ok(());
    }
    let route = path(state, &intent)?;
    match ruling {
        ShoveGeometryRuling::CommitExactPush => {
            let plan = route.map_err(|e| prerequisite(&e.to_string()))?;
            if plan.destination != intent.destination
                || plan.segments.len() != 1
                || plan.total_cost != 0
                || !plan.segments[0].opportunities.is_empty()
                || plan.segments[0].from != intent.from
                || plan.segments[0].to != intent.destination
                || plan.progress != TacticalMovementProgress::default()
            {
                return Err(invalid("Forced Push differs from its exact source route"));
            }
        }
        ShoveGeometryRuling::ConfirmBlockedNoMovement => {
            // Unsupported, invalid state, unknown actor and capacity errors never
            // authenticate an obstruction. Source-aware Air guards stay distinct.
            if !matches!(route, Err(SpatialError::Illegal(_)))
                || !crate::spatial::forced_step_physically_blocked(
                    encounter(state)?,
                    target_actor(state)?,
                    intent.destination,
                )
                .map_err(|e| invalid(&e.to_string()))?
            {
                return Err(prerequisite(
                    "Only a proven physical obstruction or boundary can confirm no movement.",
                ));
            }
        }
        ShoveGeometryRuling::ReturnToShover => unreachable!("handled above"),
    }
    let target = s.target;
    let previous = super::super::work_trace::enter(state, &selected)?;
    if ruling == ShoveGeometryRuling::CommitExactPush {
        encounter_mut(state)?
            .participants
            .iter_mut()
            .find(|p| p.entity_id == target)
            .ok_or_else(|| invalid("Pushed target absent"))?
            .position = intent.destination;
        // Only that body's voluntary continuity is invalidated. The shover's
        // already-paid Attack cleared its own run-up at admission.
        if active(state)? == target {
            flow_mut(state)?.budget.movement_progress = None;
            flow_mut(state)?.budget.movement_origin = None;
        }
    }
    let s = current_mut(state)?;
    s.push = None;
    s.selected = None;
    s.stage = TacticalShoveStage::Resolving;
    s.effect = Some(if ruling == ShoveGeometryRuling::CommitExactPush {
        TacticalShoveEffect::Push {
            intent,
            ruled_by: meta.clone(),
        }
    } else {
        TacticalShoveEffect::BlockedPush {
            intent,
            ruled_by: meta.clone(),
        }
    });
    push_frame(state, vec![TacticalWorkKind::FinishShove])?;
    if ruling == ShoveGeometryRuling::CommitExactPush {
        queue_fall(state, meta, &selected)?;
    }
    super::super::work_trace::leave(state, previous)?;
    refresh_dodges(state)?;
    pump(state, meta)
}

fn encounter_mut(state: &mut CampaignState) -> Result<&mut TacticalEncounter, RulesError> {
    state
        .encounter
        .as_mut()
        .ok_or_else(|| invalid("Shove encounter absent"))
}

pub(super) fn fall_path(state: &CampaignState) -> Result<Option<SpatialFall>, RulesError> {
    let s = current(state)?;
    let e = encounter(state)?;
    let target = e
        .participant(s.target)
        .ok_or_else(|| invalid("Shove target absent"))?;
    if target.movement.fly.is_some() {
        return crate::spatial::flight_loss_fall(e, state, s.target)
            .map_err(|e| invalid(&e.to_string()));
    }
    if matches!(s.effect, Some(TacticalShoveEffect::Push { .. })) {
        // Forced displacement terminates a climb/jump's prior support. Actual
        // ground, ledge, buried support and liquid are rederived at the endpoint.
        return crate::spatial::fall_destination(e, s.target).map_err(|e| invalid(&e.to_string()));
    }
    Ok(None)
}

fn queue_fall(
    state: &mut CampaignState,
    meta: &CommandMeta,
    selected: &TacticalWorkItem,
) -> Result<(), RulesError> {
    let Some(path) = fall_path(state)? else {
        return Ok(());
    };
    let s = current(state)?;
    let fall = TacticalFall {
        origin: meta.clone(),
        actor: s.target,
        cause: TacticalFallCause::Shove {
            shove: s.origin.clone(),
            consequence: meta.clone(),
            work: TacticalWorkKey {
                resolution: s.origin.id,
                occurrence: selected.occurrence,
            },
        },
        path,
        stage: TacticalFallStage::Queued,
    };
    let falls = &mut resolution_mut(state)?.falls;
    let index = u16::try_from(falls.len()).map_err(|_| invalid("Too many falling occurrences"))?;
    falls.push(fall);
    push_frame(state, vec![TacticalWorkKind::BeginFall { fall: index }])
}

fn target_actor(state: &CampaignState) -> Result<EntityId, RulesError> {
    Ok(current(state)?.target)
}
