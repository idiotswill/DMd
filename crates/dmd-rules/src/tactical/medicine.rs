//! Owned first aid spends an Action before requesting the helper's actual Medicine dice.
use super::turns::*;
use super::*;

fn eligible(
    state: &CampaignState,
    actor: EntityId,
    target: EntityId,
    purpose: MedicinePurpose,
) -> Result<(), RulesError> {
    let e = encounter(state)?;
    // Knowledge is checked before inspecting the target's private vitality state.
    if !crate::spatial::perceive(e, state, actor, target).is_ok_and(|p| p.precisely_located) {
        return Err(prerequisite(
            "First aid requires a currently located target.",
        ));
    }
    let from = e
        .participant(actor)
        .ok_or_else(|| invalid("Helper absent"))?;
    let to = e
        .participant(target)
        .ok_or_else(|| invalid("First-aid target absent"))?;
    let a = from.center().map_err(|e| invalid(&e))?;
    let b = to.center().map_err(|e| invalid(&e))?;
    // Source first aid requires physical administration but specifies no range.
    // This tactical contact adjudication is exposed in the desktop explanation.
    if crate::spatial::participant_distance(from, to).map_err(|e| invalid(&e.to_string()))? > 10 {
        return Err(prerequisite("Move within 5 feet to administer first aid."));
    }
    for obstacle in &e.battlefield.obstacles {
        if obstacle.blocks_movement
            && crate::spatial::segment_intersects(a, b, obstacle.volume)
                .map_err(|e| invalid(&e.to_string()))?
        {
            return Err(prerequisite(
                "First aid needs an unobstructed physical contact path.",
            ));
        }
    }
    let rules = state.rules.as_ref().ok_or(RulesError::Uninitialized)?;
    if !crate::tactical_conditions::can_act(rules, actor)? {
        return Err(prerequisite("The helper cannot take an Action."));
    }
    let entity = rules
        .entities
        .get(&target)
        .ok_or_else(|| invalid("Target absent"))?;
    let allowed = !entity.death.dead
        && match purpose {
            MedicinePurpose::Stabilize => entity.hp == 0 && !entity.death.stable,
            MedicinePurpose::EndKnockout => rules
                .tactical_recovery
                .as_ref()
                .and_then(|records| records.get(&target))
                .is_some_and(|record| record.knockout.is_some()),
        };
    if !allowed {
        return Err(prerequisite(
            "The target does not need this kind of first aid.",
        ));
    }
    Ok(())
}

pub(super) fn begin(
    state: &mut CampaignState,
    meta: &CommandMeta,
    target: EntityId,
    purpose: MedicinePurpose,
) -> Result<(), RulesError> {
    if flow(state)?.phase != TacticalPhase::Active || flow(state)?.resolution.is_some() {
        return Err(RulesError::Pending);
    }
    let actor = active(state)?;
    authorize(state, meta, actor)?;
    eligible(state, actor, target, purpose)?;
    super::falling::require_settled_before_action(state)?;
    let rules = state.rules.as_mut().ok_or(RulesError::Uninitialized)?;
    crate::tactical_budget::spend_cost(rules, actor, crate::tactical_budget::TacticalCost::Action)?;
    crate::kernel::interrupt_rest(rules, actor, state.clock.now);
    let turn_number = rules.timing.as_ref().unwrap().turn_number;
    let work_trace = super::work_trace::initial(state)?;
    let flow = flow_mut(state)?;
    flow.budget.movement_progress = None;
    flow.budget.movement_origin = None;
    flow.resolution = Some(Box::new(TacticalResolution {
        origin: meta.clone(),
        turn_actor: actor,
        turn_number,
        boundary: TurnBoundary::Start,
        frames: vec![],
        pending: None,
        failed_save: None,
        legendary_window: None,
        attack: None,
        movement: None,
        casts: vec![],
        falls: vec![],
        areas: vec![],
        work_trace,
        next_occurrence: 0,
    }));
    push_frame(
        state,
        vec![TacticalWorkKind::Medicine {
            actor,
            target,
            purpose,
        }],
    )?;
    pump(state, meta)
}

pub(super) fn validate_work(
    state: &CampaignState,
    work: &TacticalWorkItem,
) -> Result<EntityId, RulesError> {
    let TacticalWorkKind::Medicine {
        actor,
        target,
        purpose,
    } = work.kind
    else {
        return Err(invalid("Not Medicine work"));
    };
    let r = resolution(state)?;
    let rules = state.rules.as_ref().ok_or(RulesError::Uninitialized)?;
    if actor != r.turn_actor
        || actor != active(state)?
        || work.occurrence != 0
        || r.next_occurrence != 1
        || r.boundary != TurnBoundary::Start
        || !rules.timing.as_ref().is_some_and(|t| t.action_spent)
        || r.attack.is_some()
        || r.movement.is_some()
        || !r.casts.is_empty()
        || !r.falls.is_empty()
        || !r.areas.is_empty()
        || r.frames.iter().flatten().any(|w| w != work)
        || r.failed_save.is_some()
        || r.legendary_window.is_some()
    {
        return Err(invalid("Medicine work differs from its paid Action."));
    }
    authorize(state, &r.origin, actor)?;
    eligible(state, actor, target, purpose)?;
    Ok(actor)
}

pub(super) fn request(
    state: &CampaignState,
    work: &TacticalWorkItem,
    key: TacticalRollKey,
) -> Result<Option<RollRequest>, RulesError> {
    let actor = validate_work(state, work)?;
    let rules = state.rules.as_ref().ok_or(RulesError::Uninitialized)?;
    let entity = &rules.entities[&actor];
    let kind = TestKind::Check {
        ability: Ability::Wisdom,
        skill: Some(Skill::Medicine),
    };
    let modifier = match rules
        .tactical_creatures
        .as_ref()
        .and_then(|c| c.profile(actor))
    {
        Some(profile) => crate::tactical_creatures::creature_test_modifier(profile, entity, &kind)
            .map_err(|e| invalid(&e.to_string()))?,
        None => crate::test_modifier(entity, &kind),
    };
    let e = encounter(state)?;
    let mut fear_visible = false;
    for effect in crate::tactical_effect_adapter::condition_effects(rules) {
        if effect.target == actor
            && effect.condition == Some(Condition::Frightened)
            && e.participant(effect.source).is_some()
        {
            fear_visible |= crate::spatial::perceive(e, state, actor, effect.source)
                .map_err(|e| invalid(&e.to_string()))?
                .sees;
        }
    }
    let crate::tactical_conditions::TestDisposition::Roll(mode) =
        crate::tactical_conditions::check_conditions(
            rules,
            actor,
            false,
            false,
            fear_visible,
            Circumstances::default(),
        )?
    else {
        return Err(prerequisite("The helper cannot perform first aid."));
    };
    Ok(Some(RollRequest {
        id: key.request_id(),
        roller: Some(actor),
        dice: vec![DieSpec {
            count: 1,
            sides: 20,
        }],
        modifier,
        mode,
        visibility: if controller(state, actor).is_some() {
            RollVisibility::Public
        } else {
            RollVisibility::Secret
        },
        reason: "Wisdom (Medicine) first aid".into(),
    }))
}
