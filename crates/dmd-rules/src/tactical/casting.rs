//! Source spell work in the one tactical continuation. Public choices contain no
//! prepaid permission, derived DC, damage amount, or effect payload.
use super::turns::*;
use super::*;
use crate::tactical_creatures::{
    CreatureActionCost, CreatureScheduleOperation, apply_creature_schedule,
};
use crate::tactical_effects::*;
use crate::tactical_spells::*;

pub(super) fn current_cast(
    state: &CampaignState,
    cast: u16,
) -> Result<&TacticalCasting, RulesError> {
    resolution(state)?
        .casts
        .iter()
        .find(|record| record.cast.plan.occurrence == cast)
        .ok_or_else(|| invalid("spell work has no retained cast"))
}

pub(super) fn complete_occurrence(
    state: &mut CampaignState,
    cast: u16,
    at: SpellProgramOccurrence,
) -> Result<(), RulesError> {
    let record = resolution_mut(state)?
        .casts
        .iter_mut()
        .find(|record| record.cast.plan.occurrence == cast)
        .ok_or_else(|| invalid("completed spell has no retained cast"))?;
    if at.node != 0
        || usize::from(at.target) >= record.targets.len()
        || record.completed.contains(&at)
    {
        return Err(invalid("spell occurrence is absent or already completed"));
    }
    record.completed.push(at);
    Ok(())
}

fn context(state: &CampaignState, actor: EntityId) -> Result<SpellCastContext, RulesError> {
    let rules = state.rules.as_ref().ok_or(RulesError::Uninitialized)?;
    let timing = rules
        .timing
        .as_ref()
        .ok_or_else(|| invalid("casting has no turn"))?;
    Ok(SpellCastContext {
        now: state.clock.now,
        turn_number: timing.turn_number,
        current_actor: active(state)?,
        can_act: crate::tactical_conditions::can_act(rules, actor)?,
        concentration: rules
            .entities
            .get(&actor)
            .ok_or_else(|| invalid("caster absent"))?
            .concentration,
    })
}

pub(super) fn begin(
    state: &mut CampaignState,
    meta: &CommandMeta,
    choice: &SpellCastChoice,
    targets: &SpellTargetChoice,
) -> Result<(), RulesError> {
    if flow(state)?.phase != TacticalPhase::Active || flow(state)?.resolution.is_some() {
        return Err(RulesError::Pending);
    }
    authorize(state, meta, choice.actor)?;
    if active(state)? != choice.actor {
        return Err(prerequisite("this casting requires the caster's turn"));
    }
    if !matches!(choice.mode, SpellCastMode::Immediate) {
        return Err(prerequisite(
            "readied casting requires its supported perceived-trigger path",
        ));
    }
    let occurrence = 0;
    let source_step = match &choice.grant {
        SpellGrantChoice::Prepared => None,
        SpellGrantChoice::CreatureFeature { feature_id } => {
            if choice.resource != SpellResourceChoice::SourceFeature {
                return Err(invalid("source creature spell cannot select a slot"));
            }
            let current = state
                .rules
                .as_ref()
                .and_then(|r| r.tactical_creatures.as_ref())
                .ok_or_else(|| prerequisite("source creature profile absent"))?;
            Some(
                apply_creature_schedule(
                    state,
                    current,
                    meta,
                    &CreatureScheduleOperation::BeginFeature {
                        actor: choice.actor,
                        selection: CreatureFeatureSelection {
                            feature_id: feature_id.clone(),
                            spell_id: Some(choice.spell_id.clone()),
                            simple_action: None,
                        },
                        steps: vec![],
                    },
                )
                .map_err(|error| invalid(&error.to_string()))?,
            )
        }
    };
    let feature = source_step
        .as_ref()
        .map(|step| {
            step.feature
                .as_ref()
                .ok_or_else(|| invalid("selected source feature does not produce a spell"))
        })
        .transpose()?;
    let plan = if let Some(feature) = feature {
        plan_spell_from_feature(
            state,
            feature,
            choice.material,
            choice.mode.clone(),
            occurrence,
        )?
    } else {
        plan_spell_cast_at(state, meta, choice, occurrence)?
    };
    if plan.choice != *choice {
        return Err(invalid(
            "source casting choice differs from its authenticated feature",
        ));
    }
    let bound = bind_spell(state, &plan, targets)?;
    if bound.consumed_material().is_some() {
        return Err(prerequisite(
            "consumed spell materials require their physical expenditure path",
        ));
    }
    validate_spell_slot_reservation(state, &plan, &[])?;
    let begun = begin_cast(&plan, &context(state, choice.actor)?)?;
    let record = retain_spell_cast(begun.cast, &bound, targets.clone(), feature)?;
    // Every prerequisite above used the pre-action state. From here the outer
    // resolve_tactical clone makes source counter, action, concentration and queue atomic.
    if let Some(source) = source_step {
        let cost = match source.cost {
            CreatureActionCost::Action => SpellCastingCost::Action,
            CreatureActionCost::BonusAction => SpellCastingCost::BonusAction,
            _ => {
                return Err(prerequisite(
                    "source casting activation needs its enclosing continuation",
                ));
            }
        };
        *state = apply_spell_casting_cost(state, choice.actor, cost)?;
        state
            .rules
            .as_mut()
            .ok_or(RulesError::Uninitialized)?
            .tactical_creatures = Some(source.next);
    }
    for obligation in begun.obligations {
        match obligation {
            SpellCastObligation::SpendCastingCost { actor, cost } => {
                *state = apply_spell_casting_cost(state, actor, cost)?;
            }
            SpellCastObligation::RequireCreatureFeatureReceipt { .. }
                if record.creature_activation.is_some() =>
            {
                ()
            }
            SpellCastObligation::BeginConcentration { group } => {
                effect_operation(
                    state,
                    meta,
                    EffectLifecycleOperation::BeginConcentration { group },
                )?;
            }
            _ => return Err(invalid("unhandled source casting admission obligation")),
        }
    }
    if plan.program.spell_level > 0 {
        crate::kernel::interrupt_rest(
            state.rules.as_mut().ok_or(RulesError::Uninitialized)?,
            choice.actor,
            state.clock.now,
        );
    }
    let turn = context(state, choice.actor)?.turn_number;
    let budget = &mut flow_mut(state)?.budget;
    budget.movement_progress = None;
    budget.movement_origin = None;
    flow_mut(state)?.resolution = Some(Box::new(TacticalResolution {
        origin: meta.clone(),
        turn_actor: choice.actor,
        turn_number: turn,
        boundary: TurnBoundary::Start,
        frames: vec![],
        pending: None,
        failed_save: None,
        legendary_window: None,
        attack: None,
        movement: None,
        casts: vec![record],
        next_occurrence: 1,
    }));
    commit(state, meta, occurrence)?;
    pump(state, meta)
}

fn commit(state: &mut CampaignState, meta: &CommandMeta, cast: u16) -> Result<(), RulesError> {
    let record = current_cast(state, cast)?.clone();
    let context = context(state, record.cast.plan.choice.actor)?;
    let operation = if !context.can_act {
        SpellCastAdvance::Interrupt(SpellCastingInterruption::Incapacitated)
    } else if record.cast.plan.concentration_group.is_some()
        && record.cast.plan.concentration_group != context.concentration
    {
        SpellCastAdvance::Interrupt(SpellCastingInterruption::ConcentrationLost)
    } else {
        SpellCastAdvance::Commit
    };
    let transition = advance_cast(&record.cast, meta, &context, operation)?;
    resolution_mut(state)?
        .casts
        .iter_mut()
        .find(|r| r.cast.plan.occurrence == cast)
        .ok_or_else(|| invalid("casting vanished before commit"))?
        .cast = Box::new(transition.cast);
    for obligation in transition.obligations {
        match obligation {
            SpellCastObligation::CommitExpenditure { actor, expenditure } => {
                *state = apply_spell_expenditure(state, actor, &expenditure)?;
            }
            SpellCastObligation::ActivateConcentration { .. } => {
                let operation =
                    spell_casting_duration_operation(current_cast(state, cast)?, state.clock.now)?;
                effect_operation(state, meta, operation)?;
            }
            SpellCastObligation::EndConcentration { actor, group } => {
                end_owned_group(state, meta, actor, group)?;
            }
            SpellCastObligation::QueueProgram { .. } => {
                let record = current_cast(state, cast)?;
                // None of the current executable programs consumes a material.
                // A future consumed-material descriptor must get its exact physical
                // expenditure at admission, before its program can enter this path.
                if record.consumed_material.is_some() {
                    return Err(prerequisite(
                        "consumed spell materials require their physical expenditure path",
                    ));
                }
                let count = record.targets.len();
                push_frame(state, vec![TacticalWorkKind::FinishSpell { cast }])?;
                for target in (0..count).rev() {
                    push_frame(
                        state,
                        vec![TacticalWorkKind::SpellProgram {
                            cast,
                            at: SpellProgramOccurrence {
                                node: 0,
                                target: target as u16,
                            },
                        }],
                    )?;
                }
            }
            _ => return Err(invalid("unhandled source casting commit obligation")),
        }
    }
    Ok(())
}

fn end_owned_group(
    state: &mut CampaignState,
    meta: &CommandMeta,
    actor: EntityId,
    group: EffectId,
) -> Result<(), RulesError> {
    if effects(state)?
        .group_for_owner(actor)
        .is_some_and(|owned| owned.id == group)
    {
        effect_operation(
            state,
            meta,
            EffectLifecycleOperation::EndConcentration {
                owner: actor,
                reason: EffectEndReason::ConcentrationBroken,
            },
        )?;
    }
    Ok(())
}

pub(super) fn key(
    state: &CampaignState,
    work: &TacticalWorkItem,
) -> Result<TacticalRollKey, RulesError> {
    let TacticalWorkKind::SpellProgram { cast, at } = work.kind else {
        return Err(invalid("spell completion has no raw dice"));
    };
    let record = current_cast(state, cast)?;
    let target = record
        .targets
        .get(usize::from(at.target))
        .ok_or_else(|| invalid("spell target missing"))?;
    let role = match executable_spell_kind(&record.cast.plan)? {
        ExecutableSpellKind::SavingThrowCondition => TacticalRollRole::SpellSave,
        ExecutableSpellKind::Healing | ExecutableSpellKind::AutomaticDamage => {
            TacticalRollRole::SpellAmount
        }
        ExecutableSpellKind::AttackDamage => {
            return Err(invalid("spell attack uses its attack request"));
        }
    };
    Ok(TacticalRollKey {
        origin: record.cast.plan.origin.id,
        role,
        subject: target.actor,
        occurrence: work.occurrence,
    })
}

pub(super) fn request(
    state: &CampaignState,
    work: &TacticalWorkItem,
    key: TacticalRollKey,
) -> Result<Option<RollRequest>, RulesError> {
    let TacticalWorkKind::SpellProgram { cast, at } = work.kind else {
        return Err(invalid("not spell dice work"));
    };
    let record = current_cast(state, cast)?;
    match record.cast.plan.program.nodes.as_slice() {
        [SpellProgramNode::SaveCondition { ability, .. }] => super::continuations::save_request(
            state,
            record.targets[usize::from(at.target)].actor,
            key,
            *ability,
            "Spell saving throw",
        ),
        [SpellProgramNode::Heal { .. } | SpellProgramNode::AutomaticDamage { .. }] => {
            let visibility = if controller(state, record.cast.plan.choice.actor).is_some() {
                RollVisibility::Public
            } else {
                RollVisibility::Secret
            };
            Ok(Some(spell_amount_request(
                record,
                at,
                key.request_id(),
                visibility,
            )?))
        }
        _ => Err(invalid("spell program has no ordinary source request")),
    }
}

/// True means the occurrence completed without a raw request. Invalid source
/// types/dead creatures are a paid no effect, never a free type-identification error.
pub(super) fn start(
    state: &mut CampaignState,
    meta: &CommandMeta,
    cast: u16,
    at: SpellProgramOccurrence,
) -> Result<bool, RulesError> {
    let record = current_cast(state, cast)?;
    let target = record
        .targets
        .get(usize::from(at.target))
        .ok_or_else(|| invalid("spell target absent"))?;
    let unavailable_target = state
        .rules
        .as_ref()
        .and_then(|r| r.entities.get(&target.actor))
        .is_none_or(|entity| entity.death.dead);
    let group_lost = record.cast.plan.program.concentration
        && state
            .rules
            .as_ref()
            .and_then(|r| r.entities.get(&record.cast.plan.choice.actor))
            .is_none_or(|caster| caster.concentration != record.cast.plan.concentration_group);
    if !target.source_type_matches || unavailable_target || group_lost {
        complete_occurrence(state, cast, at)?;
        return Ok(true);
    }
    if executable_spell_kind(&record.cast.plan)? == ExecutableSpellKind::AttackDamage {
        let bound = retained_spell_binding(record)?;
        let proof = spell_attack_occurrence(&record.cast, &bound, at.node, at.target)?;
        super::attacks::begin_spell_attack(state, meta, &proof)?;
        return Ok(true);
    }
    Ok(false)
}

pub(super) fn finish(
    state: &mut CampaignState,
    meta: &CommandMeta,
    pending: &TacticalPendingWork,
    result: Option<&RollResult>,
    forced_success: bool,
) -> Result<(), RulesError> {
    let TacticalWorkKind::SpellProgram { cast, at } = pending.work.kind else {
        return Err(invalid("not spell completion"));
    };
    let record = current_cast(state, cast)?.clone();
    let target = record.targets[usize::from(at.target)].actor;
    match record.cast.plan.program.nodes.as_slice() {
        [SpellProgramNode::SaveCondition { .. }] => {
            let success = forced_success || !save_failed(state, pending, result)?;
            let effect = if success {
                None
            } else {
                spell_condition_effect(state, &record, at)?
            };
            complete_occurrence(state, cast, at)?;
            if let Some(effect) = effect {
                effect_operation(
                    state,
                    meta,
                    EffectLifecycleOperation::Install {
                        effects: vec![effect],
                    },
                )?;
                let work = new_effect_work(state)?;
                push_frame(state, work)?;
            }
        }
        [SpellProgramNode::Heal { .. } | SpellProgramNode::AutomaticDamage { .. }] => {
            let operation = spell_amount_operation(
                state,
                &record,
                at,
                pending.key.request_id(),
                result.ok_or_else(|| invalid("spell amount requires raw dice"))?,
            )?;
            complete_occurrence(state, cast, at)?;
            if let Some(operation) = operation {
                super::continuations::apply_vitality(
                    state,
                    meta,
                    target,
                    pending.work.occurrence,
                    operation,
                    Some(record.cast.plan.choice.actor),
                )?;
            }
        }
        _ => return Err(invalid("spell result differs from supported source work")),
    }
    Ok(())
}

pub(super) fn save_failed(
    state: &CampaignState,
    pending: &TacticalPendingWork,
    result: Option<&RollResult>,
) -> Result<bool, RulesError> {
    let Some(result) = result else {
        return Ok(true);
    };
    let TacticalWorkKind::SpellProgram { cast, .. } = pending.work.kind else {
        return Err(invalid("not spell saving work"));
    };
    let record = current_cast(state, cast)?;
    let dc = record
        .cast
        .plan
        .program
        .save_dc
        .ok_or_else(|| invalid("spell has no source saving DC"))?;
    let roll = request(state, &pending.work, pending.key)?
        .ok_or_else(|| invalid("automatic save cannot supply dice"))?
        .resolve(result)?;
    Ok(!crate::test_outcome::ability_test_success(
        &roll,
        i32::from(dc),
        &state
            .rules
            .as_ref()
            .ok_or(RulesError::Uninitialized)?
            .house_rules,
    )?)
}

pub(super) fn finish_cast(
    state: &mut CampaignState,
    meta: &CommandMeta,
    cast: u16,
) -> Result<(), RulesError> {
    let record = current_cast(state, cast)?.clone();
    if record.completed.len() != record.targets.len() {
        return Err(invalid("spell cleanup preceded unfinished target work"));
    }
    if let Some(group) = record.cast.plan.concentration_group
        && effects(state)?
            .groups
            .iter()
            .any(|owned| owned.id == group && owned.stage == ConcentrationStage::Casting)
    {
        end_owned_group(state, meta, record.cast.plan.choice.actor, group)?;
    }
    resolution_mut(state)?
        .casts
        .retain(|r| r.cast.plan.occurrence != cast);
    Ok(())
}

pub(super) fn validate_work(
    state: &CampaignState,
    work: &TacticalWorkItem,
) -> Result<EntityId, RulesError> {
    let (cast, at) = match work.kind {
        TacticalWorkKind::SpellProgram { cast, at } => (cast, Some(at)),
        TacticalWorkKind::FinishSpell { cast } => (cast, None),
        _ => return Err(invalid("not spell work")),
    };
    let record = current_cast(state, cast)?;
    if let Some(at) = at {
        if at.node != 0 || record.completed.contains(&at) {
            return Err(invalid(
                "queued spell occurrence is completed or outside its source program",
            ));
        }
        return record
            .targets
            .get(usize::from(at.target))
            .map(|target| target.actor)
            .ok_or_else(|| invalid("queued spell target is outside source binding"));
    }
    Ok(record.cast.plan.choice.actor)
}

/// The complete work partition is necessary in addition to source reconstruction:
/// a forged queued ordinal cannot silently omit or duplicate an admitted target.
pub(super) fn validate(state: &CampaignState) -> Result<(), RulesError> {
    let Some(r) = flow(state)?.resolution.as_deref() else {
        return Ok(());
    };
    if r.casts.len() > MAX_TACTICAL_CASTS {
        return Err(invalid("casting capacity exceeded"));
    }
    let mut ids = std::collections::HashSet::new();
    for record in &r.casts {
        validate_retained_spell(record)?;
        let plan = &record.cast.plan;
        let actor = plan.choice.actor;
        if !ids.insert(plan.occurrence)
            || plan.occurrence >= r.next_occurrence
            || !matches!(
                record.cast.phase,
                SpellCastPhase::Committed | SpellCastPhase::Released
            )
            || record.cast.started_at > state.clock.now
            || record.cast.started_on_turn > r.turn_number
            || plan.origin.expected_event_sequence < r.origin.expected_event_sequence
            || record.cast.last_operation.expected_event_sequence > state.applied_event_sequence
            || !flow(state)?
                .combatants
                .iter()
                .any(|combatant| combatant.actor == actor)
        {
            return Err(invalid(
                "invalid retained tactical cast timing, identity or phase",
            ));
        }
        validate_equipment_change_origin(state, &plan.origin, actor).map_err(|e| invalid(&e))?;
        validate_equipment_change_origin(state, &record.cast.last_operation, actor)
            .map_err(|e| invalid(&e))?;
        authorize(state, &plan.origin, actor)?;
        if let SpellGrantChoice::CreatureFeature { feature_id } = &plan.choice.grant {
            // The canonical program proves what a source can cast; this binding
            // also proves that the retained actor actually has that source.
            // Keep the immutable profile here, not mutable transformed statistics.
            let profile = state
                .rules
                .as_ref()
                .and_then(|rules| rules.tactical_creatures.as_ref())
                .and_then(|creatures| creatures.profile(actor))
                .ok_or_else(|| invalid("retained source caster profile is absent"))?;
            let source = crate::tactical_creatures::source_for_profile(profile)
                .map_err(|error| invalid(&error.to_string()))?;
            if plan.program.source.creature_definition_id.as_deref() != Some(source.id.as_str())
                || plan.program.source.feature_id.as_ref() != Some(feature_id)
                || !source
                    .features
                    .iter()
                    .any(|feature| feature.id == *feature_id)
            {
                return Err(invalid(
                    "retained spell grant differs from its actor's source profile",
                ));
            }
        }
        let mut partition: std::collections::HashSet<_> =
            record.completed.iter().copied().collect();
        let mut finish_count = 0;
        for work in r
            .frames
            .iter()
            .flatten()
            .chain(r.pending.iter().map(|p| &p.work))
            .chain(r.failed_save.iter().map(|p| &p.pending.work))
        {
            match work.kind {
                TacticalWorkKind::SpellProgram { cast, at } if cast == plan.occurrence => {
                    if at.node != 0
                        || usize::from(at.target) >= record.targets.len()
                        || !partition.insert(at)
                    {
                        return Err(invalid("duplicate or invalid cast work partition"));
                    }
                }
                TacticalWorkKind::FinishSpell { cast } if cast == plan.occurrence => {
                    finish_count += 1
                }
                _ => (),
            }
        }
        if let Some((cast, at)) = r.attack.as_ref().and_then(super::attacks::spell_occurrence)
            && cast == plan.occurrence
            && (at.node != 0
                || usize::from(at.target) >= record.targets.len()
                || !partition.insert(at))
        {
            return Err(invalid(
                "live spell attack duplicates or invents a target occurrence",
            ));
        }
        if finish_count != 1 || partition.len() != record.targets.len() {
            return Err(invalid(
                "cast work omitted a target or its single completion",
            ));
        }
    }
    Ok(())
}
