//! Source falling interrupts share the existing tactical work stack and raw rolls.
mod validation;
pub(super) use validation::{validate, validate_work};

use super::turns::*;
use super::*;
use crate::tactical_falling as source;

fn current(state: &CampaignState, index: u16) -> Result<&TacticalFall, RulesError> {
    resolution(state)?
        .falls
        .get(usize::from(index))
        .ok_or_else(|| invalid("Fall occurrence is absent."))
}
fn current_mut(state: &mut CampaignState, index: u16) -> Result<&mut TacticalFall, RulesError> {
    resolution_mut(state)?
        .falls
        .get_mut(usize::from(index))
        .ok_or_else(|| invalid("Fall occurrence is absent."))
}
fn visibility(state: &CampaignState, actor: EntityId) -> RollVisibility {
    if controller(state, actor).is_some() {
        RollVisibility::Public
    } else {
        RollVisibility::Secret
    }
}
fn dead(state: &CampaignState, actor: EntityId) -> Result<bool, RulesError> {
    Ok(state
        .rules
        .as_ref()
        .ok_or(RulesError::Uninitialized)?
        .entities
        .get(&actor)
        .ok_or_else(|| invalid("Falling actor mechanics are absent."))?
        .death
        .dead)
}
pub(super) fn selected(state: &CampaignState) -> Result<Option<u16>, RulesError> {
    let mut selected = None;
    for (index, fall) in resolution(state)?.falls.iter().enumerate() {
        if matches!(fall.stage, TacticalFallStage::LandingChoice) {
            if selected.is_some() {
                return Err(invalid("Several liquid landing choices are selected."));
            }
            selected = Some(u16::try_from(index).map_err(|_| invalid("Fall capacity."))?);
        }
    }
    Ok(selected)
}
fn register(state: &mut CampaignState, fall: TacticalFall) -> Result<u16, RulesError> {
    let r = resolution_mut(state)?;
    if r.falls.len() >= 256
        || r.falls.iter().any(|other| {
            other.actor == fall.actor && !matches!(other.stage, TacticalFallStage::Complete { .. })
        })
    {
        return Err(invalid("Fall occurrence capacity or duplicate live actor."));
    }
    let index = r.falls.len() as u16;
    r.falls.push(fall);
    Ok(index)
}

/// Derive unsupported air from current source state. A declared jump keeps its
/// traversal while that movement remains usable; lost movement cannot suspend it.
fn loss(
    state: &CampaignState,
    actor: EntityId,
) -> Result<Option<(TacticalFallCause, SpatialFall)>, RulesError> {
    let e = encounter(state)?;
    if let Some(path) = crate::spatial::flight_loss_fall(e, state, actor)
        .map_err(|error| invalid(&error.to_string()))?
    {
        return Ok(Some((TacticalFallCause::FlightLost, path)));
    }
    let participant = e
        .participant(actor)
        .ok_or_else(|| invalid("Fall actor absent."))?;
    if participant.movement.fly.is_some() {
        return Ok(None);
    }
    if flow(state)?
        .resolution
        .as_ref()
        .and_then(|r| r.movement.as_ref())
        .is_some_and(|movement| movement.actor == actor)
        && flow(state)?
            .budget
            .movement_progress
            .as_ref()
            .is_some_and(|p| p.jump.is_some())
    {
        // Actual accepted jumping takes precedence over a region that merely
        // permits climbing. A knocked-out jumper has not grabbed that surface.
        return if crate::tactical_movement::capability(state, actor, MovementMode::Jump).is_ok() {
            Ok(None)
        } else {
            Ok(crate::spatial::fall_destination(e, actor)
                .map_err(|error| invalid(&error.to_string()))?
                .map(|path| (TacticalFallCause::Unsupported, path)))
        };
    }
    let body = participant.volume().map_err(|error| invalid(&error))?;
    // Authored climbable surfaces support the existing climbing mode. Merely
    // difficult/obscured space does not. A source fall out of such a grip needs its
    // own explicit release/displacement rather than an invented climbing-failure rule.
    if e.battlefield
        .terrain
        .iter()
        .any(|t| t.climbable && body.intersects(t.volume))
    {
        return Ok(None);
    }
    if crate::spatial::buried_support(e, participant)
        .map_err(|error| invalid(&error.to_string()))?
    {
        return Ok(None);
    }
    Ok(crate::spatial::fall_destination(e, actor)
        .map_err(|error| invalid(&error.to_string()))?
        .map(|path| (TacticalFallCause::Unsupported, path)))
}

/// Ordinary movement's temporary airborne jump steps remain under their accepted
/// origin while usable. Source interruptions insert falling above that same path.
pub(super) fn require_settled_before_action(state: &CampaignState) -> Result<(), RulesError> {
    // Supported transitions pump these occurrences before returning to idle. A
    // malformed/imported idle map must not interleave an old fall after a newly
    // bound instantaneous effect. The error does not identify any hidden actor.
    for participant in &encounter(state)?.participants {
        if loss(state, participant.entity_id)?.is_some() {
            return Err(prerequisite(
                "the encounter must finish its pending physical consequences",
            ));
        }
    }
    Ok(())
}

/// Ordinary movement's temporary airborne jump steps remain under their accepted
/// origin while usable. Source interruptions insert falling above that same path.
pub(super) fn queue_losses(
    state: &mut CampaignState,
    meta: &CommandMeta,
) -> Result<(), RulesError> {
    let mut actors = encounter(state)?
        .participants
        .iter()
        .map(|p| p.entity_id)
        .collect::<Vec<_>>();
    actors.sort_by_key(|id| id.0);
    let mut kinds = Vec::new();
    for actor in actors {
        if resolution(state)?
            .falls
            .iter()
            .any(|f| f.actor == actor && !matches!(f.stage, TacticalFallStage::Complete { .. }))
        {
            continue;
        }
        if let Some((cause, path)) = loss(state, actor)? {
            let fall = register(
                state,
                TacticalFall {
                    origin: meta.clone(),
                    actor,
                    cause,
                    path,
                    stage: TacticalFallStage::Queued,
                },
            )?;
            kinds.push(TacticalWorkKind::BeginFall { fall });
        }
    }
    push_frame(state, kinds)
}

pub(super) fn queue_movement_end(
    state: &mut CampaignState,
    meta: &CommandMeta,
    movement: &TacticalMovement,
) -> Result<bool, RulesError> {
    let Some(last) = movement.traversed.last() else {
        return Ok(false);
    };
    if matches!(
        last.mode,
        MovementMode::Fly | MovementMode::Climb | MovementMode::Swim | MovementMode::Burrow
    ) {
        return Err(invalid(
            "Unsupported fall flag for a source-supported movement mode.",
        ));
    }
    let path = crate::spatial::fall_destination(encounter(state)?, movement.actor)
        .map_err(|e| invalid(&e.to_string()))?
        .ok_or_else(|| invalid("Required fall lost its canonical support geometry."))?;
    let fall = register(
        state,
        TacticalFall {
            origin: meta.clone(),
            actor: movement.actor,
            cause: TacticalFallCause::MovementEnd {
                movement: movement.origin.clone(),
                step_index: movement.next_step - 1,
            },
            path,
            stage: TacticalFallStage::Queued,
        },
    )?;
    push_frame(state, vec![TacticalWorkKind::BeginFall { fall }])?;
    Ok(true)
}

fn index(work: &TacticalWorkItem) -> Option<u16> {
    match work.kind {
        TacticalWorkKind::BeginFall { fall }
        | TacticalWorkKind::LiquidLandingCheck { fall }
        | TacticalWorkKind::FallDamage { fall } => Some(fall),
        _ => None,
    }
}
pub(super) fn key(
    state: &CampaignState,
    work: &TacticalWorkItem,
) -> Result<TacticalRollKey, RulesError> {
    let fall = current(
        state,
        index(work).ok_or_else(|| invalid("Work is not falling."))?,
    )?;
    let role = match work.kind {
        TacticalWorkKind::LiquidLandingCheck { .. } => TacticalRollRole::LiquidLandingCheck,
        TacticalWorkKind::FallDamage { .. } => TacticalRollRole::FallDamage,
        _ => return Err(invalid("Fall choice has no raw roll.")),
    };
    Ok(TacticalRollKey {
        origin: resolution(state)?.origin.id,
        role,
        subject: fall.actor,
        occurrence: work.occurrence,
    })
}
pub(super) fn request(
    state: &CampaignState,
    work: &TacticalWorkItem,
    key: TacticalRollKey,
) -> Result<Option<RollRequest>, RulesError> {
    let fall = current(
        state,
        index(work).ok_or_else(|| invalid("Work is not falling."))?,
    )?;
    if key != self::key(state, work)? {
        return Err(invalid("Fall raw identity differs."));
    }
    match (&work.kind, &fall.stage) {
        (
            TacticalWorkKind::LiquidLandingCheck { .. },
            TacticalFallStage::LandingCheck { choice, .. },
        ) => Ok(Some(source::liquid_landing_request(
            state,
            fall.actor,
            *choice,
            key.request_id(),
            visibility(state, fall.actor),
        )?)),
        (TacticalWorkKind::FallDamage { .. }, TacticalFallStage::Damage { .. }) => {
            if dead(state, fall.actor)? {
                return Ok(None);
            }
            source::fall_damage_request(
                &fall.path,
                fall.actor,
                key.request_id(),
                visibility(state, fall.actor),
            )
        }
        _ => Err(invalid("Fall work differs from its retained stage.")),
    }
}

pub(super) fn start(
    state: &mut CampaignState,
    meta: &CommandMeta,
    work: &TacticalWorkItem,
) -> Result<bool, RulesError> {
    let Some(index) = index(work) else {
        return Ok(false);
    };
    match work.kind {
        TacticalWorkKind::BeginFall { .. } => {
            let fall = current(state, index)?.clone();
            if fall.stage != TacticalFallStage::Queued {
                return Err(invalid("Fall already started."));
            }
            if matches!(fall.path.surface, FallSurface::Liquid { .. })
                && source::can_attempt_liquid_landing(state, fall.actor)?
            {
                current_mut(state, index)?.stage = TacticalFallStage::LandingChoice;
            } else {
                current_mut(state, index)?.stage = TacticalFallStage::Damage { landing: None };
                push_frame(state, vec![TacticalWorkKind::FallDamage { fall: index }])?;
            }
            Ok(true)
        }
        TacticalWorkKind::FallDamage { .. } => {
            let key = key(state, work)?;
            if request(state, work, key)?.is_none() {
                land(state, meta, index, key, None)?;
                Ok(true)
            } else {
                Ok(false)
            }
        }
        TacticalWorkKind::LiquidLandingCheck { .. } => Ok(false),
        _ => unreachable!("index accepts only fall work"),
    }
}

pub(super) fn choose(
    state: &mut CampaignState,
    meta: &CommandMeta,
    choice: Option<LiquidLandingChoice>,
) -> Result<(), RulesError> {
    let index = selected(state)?.ok_or_else(|| prerequisite("No liquid landing choice is due."))?;
    let actor = current(state, index)?.actor;
    authorize(state, meta, actor)?;
    let kind = if let Some(choice) = choice {
        if !source::can_attempt_liquid_landing(state, actor)? {
            return Err(prerequisite(
                "The actor cannot spend a liquid landing Reaction.",
            ));
        }
        state
            .rules
            .as_mut()
            .ok_or(RulesError::Uninitialized)?
            .timing
            .as_mut()
            .ok_or_else(|| invalid("Landing Reaction lacks combat timing."))?
            .reactions_spent
            .push(actor);
        current_mut(state, index)?.stage = TacticalFallStage::LandingCheck {
            choice,
            accepted_by: meta.clone(),
        };
        TacticalWorkKind::LiquidLandingCheck { fall: index }
    } else {
        current_mut(state, index)?.stage = TacticalFallStage::Damage { landing: None };
        TacticalWorkKind::FallDamage { fall: index }
    };
    // The liquid choice changes its actor's Reaction, not the parent occurrence's
    // ordering ownership. Preserve the exact BeginFall consequence ancestry.
    let parent =
        super::work_trace::prior_work(state, &TacticalWorkKind::BeginFall { fall: index })?;
    let previous = parent
        .as_ref()
        .map(|work| super::work_trace::enter(state, work))
        .transpose()?
        .flatten();
    let queued = push_frame(state, vec![kind]);
    let reset = super::work_trace::leave(state, previous);
    queued?;
    reset?;
    pump(state, meta)
}

pub(super) fn finish(
    state: &mut CampaignState,
    meta: &CommandMeta,
    pending: &TacticalPendingWork,
    result: Option<&RollResult>,
) -> Result<(), RulesError> {
    let index = index(&pending.work).ok_or_else(|| invalid("Work is not falling."))?;
    match pending.work.kind {
        TacticalWorkKind::LiquidLandingCheck { .. } => {
            let TacticalFallStage::LandingCheck {
                choice,
                accepted_by,
            } = current(state, index)?.stage.clone()
            else {
                return Err(invalid("Liquid check has no accepted Reaction."));
            };
            let result =
                result.ok_or_else(|| invalid("A liquid landing check needs real dice."))?;
            request(state, &pending.work, pending.key)?
                .ok_or_else(|| invalid("Liquid request absent."))?
                .resolve(result)?;
            current_mut(state, index)?.stage = TacticalFallStage::Damage {
                landing: Some(TacticalLiquidLanding {
                    choice,
                    accepted_by,
                    result: result.clone(),
                }),
            };
            push_frame(state, vec![TacticalWorkKind::FallDamage { fall: index }])
        }
        TacticalWorkKind::FallDamage { .. } => land(state, meta, index, pending.key, result),
        _ => Err(invalid("Fall choice cannot finish a raw roll.")),
    }
}

fn land(
    state: &mut CampaignState,
    meta: &CommandMeta,
    index: u16,
    key: TacticalRollKey,
    result: Option<&RollResult>,
) -> Result<(), RulesError> {
    let fall = current(state, index)?.clone();
    let TacticalFallStage::Damage { landing } = &fall.stage else {
        return Err(invalid("Fall is not ready to land."));
    };
    let actor = fall.actor;
    let landing_outcome = landing
        .as_ref()
        .map(|landing| {
            source::resolve_liquid_landing(
                state,
                actor,
                &fall.path,
                landing.choice,
                landing.result.request_id,
                visibility(state, actor),
                &landing.result,
            )
        })
        .transpose()?;
    let was_dead = dead(state, actor)?;
    if was_dead && result.is_some() {
        return Err(invalid("A dead body has no living fall damage roll."));
    }
    let packet = if was_dead {
        None
    } else {
        Some(source::fall_damage_packet(
            &fall.path,
            actor,
            key.request_id(),
            result,
            landing_outcome.as_ref(),
        )?)
    };
    let mover = state
        .encounter
        .as_mut()
        .ok_or_else(|| invalid("Fall encounter absent."))?
        .participants
        .iter_mut()
        .find(|p| p.entity_id == actor)
        .ok_or_else(|| invalid("Falling actor absent."))?;
    if mover.position != fall.path.from {
        return Err(invalid("Falling origin was displaced without its source."));
    }
    mover.position = fall.path.to;
    current_mut(state, index)?.stage = TacticalFallStage::Complete {
        landing: landing.clone(),
        damage: result.map(|_| key),
        resolved_by: meta.clone(),
    };
    // Physical landing precedes vitality and any newly dropped held items. Only this
    // actor's voluntary path is canceled; unrelated accepted children are retained.
    super::movement::landed(state, meta, actor)?;
    if let Some(packet) = packet {
        let outcome = super::continuations::apply_vitality_with_outcome(
            state,
            meta,
            actor,
            key.occurrence,
            VitalityOperation::Damage {
                packet,
                knockout: None,
            },
            None,
        )?;
        let entity = state
            .rules
            .as_mut()
            .ok_or(RulesError::Uninitialized)?
            .entities
            .get_mut(&actor)
            .ok_or_else(|| invalid("Landed actor mechanics absent."))?;
        if outcome.damage_taken > 0 && !entity.condition_immunities.contains(&Condition::Prone) {
            entity.prone = true;
        }
    }
    refresh_dodges(state)
}
