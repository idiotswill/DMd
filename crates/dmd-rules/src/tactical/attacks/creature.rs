//! A source stat-block melee Action enters the existing attack/damage queue.
use super::*;
use crate::tactical_creatures::{
    CreatureActionCost, CreatureScheduleOperation, apply_creature_schedule,
};
use crate::tactical_definitions::{AttackDelivery, FeatureActivation, MonsterFeature};

pub(in crate::tactical) fn begin_creature_attack(
    state: &mut CampaignState,
    meta: &CommandMeta,
    target: EntityId,
    feature_id: &str,
    weapon: Option<ItemId>,
) -> Result<(), RulesError> {
    if flow(state)?.phase != TacticalPhase::Active || flow(state)?.resolution.is_some() {
        return Err(RulesError::Pending);
    }
    let actor = active(state)?;
    authorize(state, meta, actor)?;
    planning::require_located_target(state, actor, target)?;
    if actor == target {
        return Err(prerequisite("this source attack requires another target"));
    }
    let current = state
        .rules
        .as_ref()
        .and_then(|rules| rules.tactical_creatures.as_ref())
        .ok_or_else(|| prerequisite("a pinned creature profile is required"))?;
    let source_step = apply_creature_schedule(
        state,
        current,
        meta,
        &CreatureScheduleOperation::BeginFeature {
            actor,
            selection: CreatureFeatureSelection {
                feature_id: feature_id.into(),
                spell_id: None,
                simple_action: None,
            },
            steps: vec![],
        },
    )
    .map_err(|error| prerequisite(&error.to_string()))?;
    let feature = source_step
        .feature
        .as_ref()
        .ok_or_else(|| prerequisite("this source feature requires its enclosing continuation"))?;
    if source_step.cost != CreatureActionCost::Action
        || feature.invocation != *meta
        || feature.enclosing_activation.origin != *meta
        || feature.enclosing_activation.activation != FeatureActivation::Action
        || !feature.enclosing_activation.attack_action
        || feature.feature.usage.is_some()
        || !matches!(
            feature.feature.feature,
            MonsterFeature::Attack {
                delivery: AttackDelivery::Melee { .. },
                ..
            }
        )
    {
        return Err(prerequisite(
            "this source attack requires its supported activation path",
        ));
    }
    // Real stat-block weapon attacks also need their physical weapon-property
    // continuation. This first own-turn source path is intrinsic, never a fake ItemId.
    let profile = intrinsic::profile(state, actor)?;
    if weapon.is_some()
        || crate::tactical_creature_equipment::creature_attack_gear(profile, feature_id)?.is_some()
    {
        return Err(prerequisite(
            "this source weapon needs its physical attack continuation",
        ));
    }
    let approach = crate::tactical_movement::straight_approach(state, actor, target)?
        .map(|(origin, movement, _)| TacticalChargeApproach { origin, movement });
    let mut attack = TacticalAttack {
        origin: meta.clone(),
        actor,
        target,
        delivery: TacticalAttackDelivery::Melee,
        source: TacticalAttackSource::CreatureFeature {
            source: feature.source.clone(),
            feature_id: feature_id.into(),
            weapon,
        },
        admission: TacticalAttackAdmission::CreatureAction { approach },
        attack_modifier: 0,
        mode: RollMode::Normal,
        armor_class: 0,
        critical_on_hit: false,
        automatic_miss: false,
        damage: vec![],
        stage: TacticalAttackStage::AttackRoll,
        attack_roll: None,
        damage_roll: None,
        outcome: None,
    };
    // Reconstruct every applicable source effect before any source/action cost.
    let plan = intrinsic::plan(state, &attack)?;
    attack.attack_modifier = plan.modifier;
    attack.mode = plan.mode;
    attack.armor_class = plan.armor;
    attack.critical_on_hit = plan.critical;
    attack.damage = plan.damage;
    let mut budget = flow(state)?.budget.clone();
    let now = state.clock.now;
    let rules = state.rules.as_mut().ok_or(RulesError::Uninitialized)?;
    crate::tactical_budget::start_attack_action(rules, &mut budget, actor, 1)?;
    crate::tactical_budget::spend_attack(&mut budget)?;
    budget.attack_window = Some(WeaponActionWindow {
        id: meta.id,
        kind: WeaponActionKind::AttackAction,
    });
    budget.movement_progress = None;
    budget.movement_origin = None;
    crate::kernel::interrupt_rest(rules, actor, now);
    rules.tactical_creatures = Some(source_step.next);
    let turn_number = rules
        .timing
        .as_ref()
        .ok_or_else(|| invalid("turn absent"))?
        .turn_number;
    flow_mut(state)?.budget = budget;
    let work_trace = super::super::work_trace::initial(state)?;
    flow_mut(state)?.resolution = Some(Box::new(TacticalResolution {
        origin: meta.clone(),
        turn_actor: actor,
        turn_number,
        boundary: TurnBoundary::Start,
        frames: vec![],
        pending: None,
        failed_save: None,
        legendary_window: None,
        attack: Some(attack),
        movement: None,
        casts: vec![],
        falls: vec![],
        areas: vec![],
        work_trace,
        next_occurrence: 0,
    }));
    push_frame(state, vec![TacticalWorkKind::AttackRoll])?;
    pump(state, meta)
}

pub(super) fn approach_distance(
    state: &CampaignState,
    attack: &TacticalAttack,
) -> Result<Option<u32>, RulesError> {
    let TacticalAttackAdmission::CreatureAction {
        approach: Some(proof),
    } = &attack.admission
    else {
        return Ok(None);
    };
    validate_equipment_origin(state, &proof.origin, attack.actor)
        .map_err(|error| invalid(&error))?;
    if proof.origin.expected_event_sequence >= attack.origin.expected_event_sequence
        || proof.origin.id == attack.origin.id
    {
        return Err(invalid("Charge movement must precede this source action"));
    }
    proof
        .movement
        .start
        .validate()
        .map_err(|error| invalid(&error))?;
    proof
        .movement
        .end
        .validate()
        .map_err(|error| invalid(&error))?;
    let distance = crate::spatial::grid_distance(proof.movement.start, proof.movement.end)
        .map_err(|error| invalid(&error.to_string()))?;
    let flow = flow(state)?;
    let reached = flow
        .last_movement
        .as_ref()
        .ok_or_else(|| invalid("Charge lacks its completed movement receipt"))?;
    let turn_number = state
        .rules
        .as_ref()
        .and_then(|rules| rules.timing.as_ref())
        .ok_or_else(|| invalid("Charge turn is absent"))?
        .turn_number;
    if reached.original != proof.origin
        || reached.actor != attack.actor
        || reached.turn_number != turn_number
        || reached.reason != TacticalMovementEnd::Completed
        || reached.endpoint != proof.movement.end
        || reached.spent_after != flow.budget.movement_spent
    {
        return Err(invalid("Charge differs from its latest completed movement"));
    }
    let e = encounter(state)?;
    let actor = e
        .participant(attack.actor)
        .ok_or_else(|| invalid("source attacker absent"))?;
    let target = e
        .participant(attack.target)
        .ok_or_else(|| invalid("source target absent"))?;
    if distance == 0
        || distance > flow.budget.movement_spent
        || actor.position != proof.movement.end
        || !crate::spatial::straight_movement_toward(actor, target, &proof.movement)
            .map_err(|error| invalid(&error.to_string()))?
    {
        return Err(invalid(
            "Charge proof differs from actual straight approach",
        ));
    }
    Ok(Some(distance))
}

pub(super) fn validate_admission(
    state: &CampaignState,
    attack: &TacticalAttack,
) -> Result<(), RulesError> {
    let r = resolution(state)?;
    let rules = state.rules.as_ref().ok_or(RulesError::Uninitialized)?;
    let runtime = rules
        .tactical_creatures
        .as_ref()
        .and_then(|c| c.runtime(attack.actor))
        .ok_or_else(|| invalid("source runtime absent"))?;
    let timing = rules
        .timing
        .as_ref()
        .ok_or_else(|| invalid("source timing absent"))?;
    let feature_id = match &attack.source {
        TacticalAttackSource::CreatureFeature {
            feature_id,
            weapon: None,
            ..
        } => feature_id,
        TacticalAttackSource::CreatureWeapon { feature_id, .. }
            if matches!(
                attack.admission,
                TacticalAttackAdmission::CreatureAction { approach: None }
            ) =>
        {
            feature_id
        }
        _ => {
            return Err(invalid(
                "source action lacks its canonical creature feature",
            ));
        }
    };
    let source =
        crate::tactical_creatures::source_for_profile(intrinsic::profile(state, attack.actor)?)
            .map_err(|error| invalid(&error.to_string()))?;
    let feature = source
        .features
        .iter()
        .find(|f| f.id == *feature_id)
        .ok_or_else(|| invalid("source feature absent"))?;
    if attack.origin != r.origin
        || attack.actor != r.turn_actor
        || r.boundary != TurnBoundary::Start
        || r.movement.is_some()
        || !r.casts.is_empty()
        || runtime.last_operation != attack.origin
        || runtime.routine.is_some()
        || feature.activation != FeatureActivation::Action
        || feature.usage.is_some()
        || !timing.action_spent
        || flow(state)?.budget.attacks_remaining != 0
        || flow(state)?.budget.attack_window
            != Some(WeaponActionWindow {
                id: attack.origin.id,
                kind: WeaponActionKind::AttackAction,
            })
        || flow(state)?.budget.movement_progress.is_some()
        || flow(state)?.budget.movement_origin.is_some()
    {
        return Err(invalid(
            "source attack lacks its exact accepted action and cost",
        ));
    }
    approach_distance(state, attack)?;
    Ok(())
}
