//! One source area occurrence in the shared resolution. Membership is truth-only;
//! all saves complete before any simultaneous damage consequence is applied.
mod validation;
use super::turns::*;
use super::*;
use crate::tactical_areas::*;
use crate::tactical_creatures::{
    CreatureActionCost, CreatureScheduleOperation, apply_creature_schedule,
};
use crate::tactical_definitions::{FeatureActivation, MonsterFeature};
pub(super) use validation::{validate, validate_work};

fn record(state: &CampaignState, area: u16) -> Result<&TacticalArea, RulesError> {
    resolution(state)?
        .areas
        .iter()
        .find(|a| a.occurrence == area)
        .ok_or_else(|| invalid("area work lacks its accepted source"))
}
fn record_mut(state: &mut CampaignState, area: u16) -> Result<&mut TacticalArea, RulesError> {
    resolution_mut(state)?
        .areas
        .iter_mut()
        .find(|a| a.occurrence == area)
        .ok_or_else(|| invalid("area work lacks its accepted source"))
}
fn program(state: &CampaignState, area: u16) -> Result<AreaProgram, RulesError> {
    let r = record(state, area)?;
    let source = source_area_program(state, r.source.actor, &r.source.feature_id)?;
    if source.source() != &r.source.pin {
        return Err(invalid(
            "area source differs from its actual creature profile",
        ));
    }
    Ok(source)
}
fn target(state: &CampaignState, area: u16, index: u16) -> Result<&TacticalAreaTarget, RulesError> {
    record(state, area)?
        .targets
        .get(usize::from(index))
        .ok_or_else(|| invalid("area target is outside accepted membership"))
}
fn target_mut(
    state: &mut CampaignState,
    area: u16,
    index: u16,
) -> Result<&mut TacticalAreaTarget, RulesError> {
    record_mut(state, area)?
        .targets
        .get_mut(usize::from(index))
        .ok_or_else(|| invalid("area target is outside accepted membership"))
}
pub(super) fn is_work(kind: &TacticalWorkKind) -> bool {
    matches!(
        kind,
        TacticalWorkKind::AreaDamageRoll { .. }
            | TacticalWorkKind::AreaSave { .. }
            | TacticalWorkKind::BeginAreaDamage { .. }
            | TacticalWorkKind::ApplyAreaDamage { .. }
            | TacticalWorkKind::FinishArea { .. }
    )
}
fn identity(kind: &TacticalWorkKind) -> Result<(u16, Option<u16>), RulesError> {
    match *kind {
        TacticalWorkKind::AreaSave { area, target }
        | TacticalWorkKind::ApplyAreaDamage { area, target } => Ok((area, Some(target))),
        TacticalWorkKind::AreaDamageRoll { area }
        | TacticalWorkKind::BeginAreaDamage { area }
        | TacticalWorkKind::FinishArea { area } => Ok((area, None)),
        _ => Err(invalid("work is not an area occurrence")),
    }
}

pub(super) fn begin(
    state: &mut CampaignState,
    meta: &CommandMeta,
    feature_id: &str,
    aim: TacticalAreaAim,
) -> Result<(), RulesError> {
    if flow(state)?.phase != TacticalPhase::Active || flow(state)?.resolution.is_some() {
        return Err(RulesError::Pending);
    }
    let actor = active(state)?;
    authorize(state, meta, actor)?;
    let rules = state.rules.as_ref().ok_or(RulesError::Uninitialized)?;
    // A cone-dependent rejection would disclose a hidden charmer; omitting them
    // would invent a source exception. This bounded path is unavailable while
    // any sourced restriction is active, even for a cone aimed safely away.
    // Playable Charmed-area adjudication remains active Gate4 work.
    if crate::tactical_effect_adapter::condition_effects(rules)
        .any(|effect| effect.target == actor && effect.condition == Some(Condition::Charmed))
    {
        return Err(prerequisite(
            "damaging area use while Charmed needs its supported adjudication",
        ));
    }
    super::falling::require_settled_before_action(state)?;
    let source = source_area_program(state, actor, feature_id)?;
    // The policy is immutable encounter setup. No victim-specific error or client
    // target list decides whether this source cone reaches unseen creatures.
    let bound = bind_area_geometry(encounter(state)?, state, &source, aim)?;
    let current = state
        .rules
        .as_ref()
        .and_then(|r| r.tactical_creatures.as_ref())
        .ok_or_else(|| prerequisite("source creature attachment is required"))?;
    let step = apply_creature_schedule(
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
    .map_err(|e| prerequisite(&e.to_string()))?;
    let feature = step
        .feature
        .as_ref()
        .ok_or_else(|| prerequisite("area requires its enclosing source continuation"))?;
    if step.cost != CreatureActionCost::Action
        || feature.invocation != *meta
        || feature.enclosing_activation.origin != *meta
        || feature.source != *source.source()
        || feature.enclosing_activation.activation != FeatureActivation::Action
        || !matches!(feature.feature.feature, MonsterFeature::SaveArea { .. })
    {
        return Err(prerequisite(
            "area requires its genuine source Action activation",
        ));
    }
    let record = TacticalArea {
        occurrence: 0,
        source: TacticalAreaSource {
            actor,
            pin: source.source().clone(),
            feature_id: feature_id.into(),
            invocation: feature.invocation.clone(),
            enclosing_origin: feature.enclosing_activation.origin.clone(),
        },
        aim,
        policy: bound.policy(),
        geometry_origin: encounter(state)?.origin.clone(),
        targets: bound.targets().to_vec(),
        damage: None,
        stage: TacticalAreaStage::DamageRoll,
    };
    // Resolve_tactical commits only this fully validated clone. Source recharge,
    // Action, rest interruption and continuation therefore change together.
    let now = state.clock.now;
    let rules = state.rules.as_mut().ok_or(RulesError::Uninitialized)?;
    crate::tactical_budget::spend_cost(rules, actor, crate::tactical_budget::TacticalCost::Action)?;
    crate::kernel::interrupt_rest(rules, actor, now);
    rules.tactical_creatures = Some(step.next);
    let turn_number = rules
        .timing
        .as_ref()
        .ok_or_else(|| invalid("area has no current turn"))?
        .turn_number;
    let budget = &mut flow_mut(state)?.budget;
    budget.movement_progress = None;
    budget.movement_origin = None;
    flow_mut(state)?.resolution = Some(Box::new(TacticalResolution {
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
        areas: vec![record],
        next_occurrence: 1,
    }));
    push_frame(state, vec![TacticalWorkKind::FinishArea { area: 0 }])?;
    push_frame(state, vec![TacticalWorkKind::AreaDamageRoll { area: 0 }])?;
    pump(state, meta)
}

pub(super) fn key(
    state: &CampaignState,
    work: &TacticalWorkItem,
) -> Result<TacticalRollKey, RulesError> {
    let (area, index) = identity(&work.kind)?;
    let r = record(state, area)?;
    let (role, subject) = match work.kind {
        TacticalWorkKind::AreaDamageRoll { .. } => (TacticalRollRole::AreaDamage, r.source.actor),
        TacticalWorkKind::AreaSave { .. } => (
            TacticalRollRole::AreaSave,
            target(state, area, index.expect("save index"))?.actor,
        ),
        _ => return Err(invalid("area phase transition has no raw roll")),
    };
    Ok(TacticalRollKey {
        origin: r.source.invocation.id,
        role,
        subject,
        occurrence: work.occurrence,
    })
}
pub(super) fn request(
    state: &CampaignState,
    work: &TacticalWorkItem,
    key: TacticalRollKey,
) -> Result<Option<RollRequest>, RulesError> {
    if self::key(state, work)? != key {
        return Err(invalid("area raw identity differs"));
    }
    let (area, index) = identity(&work.kind)?;
    let program = program(state, area)?;
    if let Some(index) = index {
        let target = target(state, area, index)?;
        let mut request = super::continuations::save_request(
            state,
            target.actor,
            key,
            program.ability(),
            "Saving throw against an area effect",
        )?;
        if let Some(request) = &mut request
            && program.ability() == Ability::Dexterity
        {
            request.modifier += match target.cover {
                CoverDegree::Half => 2,
                CoverDegree::ThreeQuarters => 5,
                _ => 0,
            };
        }
        Ok(request)
    } else {
        Ok(Some(area_amount_request(
            &program,
            key.request_id(),
            if controller(state, program.actor()).is_some() {
                RollVisibility::Public
            } else {
                RollVisibility::Secret
            },
        )?))
    }
}
pub(super) fn save_failed(
    state: &CampaignState,
    pending: &TacticalPendingWork,
    result: Option<&RollResult>,
) -> Result<bool, RulesError> {
    let (area, _) = identity(&pending.work.kind)?;
    let Some(raw) = result else {
        return Ok(true);
    };
    let request = request(state, &pending.work, pending.key)?
        .ok_or_else(|| invalid("automatic area save has no dice"))?;
    let resolved = request.resolve(raw)?;
    Ok(!crate::test_outcome::ability_test_success(
        &resolved,
        i32::from(program(state, area)?.dc()),
        &state
            .rules
            .as_ref()
            .ok_or(RulesError::Uninitialized)?
            .house_rules,
    )?)
}

/// Returns true for phase transitions; raw work continues through the common
/// pending/automatic/voluntary/Legendary Resistance machinery.
pub(super) fn start(
    state: &mut CampaignState,
    meta: &CommandMeta,
    work: &TacticalWorkItem,
) -> Result<bool, RulesError> {
    if !is_work(&work.kind) {
        return Ok(false);
    }
    validate_work(state, work)?;
    match work.kind {
        TacticalWorkKind::AreaDamageRoll { .. } | TacticalWorkKind::AreaSave { .. } => Ok(false),
        TacticalWorkKind::BeginAreaDamage { area } => {
            if record(state, area)?
                .targets
                .iter()
                .any(|target| target.save.is_none())
            {
                return Err(invalid(
                    "area damage preceded an outstanding simultaneous save",
                ));
            }
            record_mut(state, area)?.stage = TacticalAreaStage::ApplyingDamage;
            let kinds = (0..record(state, area)?.targets.len())
                .map(|target| TacticalWorkKind::ApplyAreaDamage {
                    area,
                    target: target as u16,
                })
                .collect();
            push_frame(state, kinds)?;
            Ok(true)
        }
        TacticalWorkKind::ApplyAreaDamage {
            area,
            target: index,
        } => {
            let target = target(state, area, index)?.clone();
            let r = record(state, area)?;
            let amount = r
                .damage
                .ok_or_else(|| invalid("area damage lacks shared raw amount"))?;
            let raw = state
                .rules
                .as_ref()
                .ok_or(RulesError::Uninitialized)?
                .rolls
                .iter()
                .find(|roll| roll.request.id == amount.request_id())
                .ok_or_else(|| invalid("shared area amount is absent"))?
                .result
                .clone();
            let source = program(state, area)?;
            let operation = area_damage_operation(
                &source,
                amount.request_id(),
                &raw,
                target
                    .save
                    .as_ref()
                    .ok_or_else(|| invalid("area target lacks its save"))?
                    .succeeded,
            )?;
            // Mark the source occurrence before opening its vitality children.
            // Another simultaneous consequence can kill a victim first; a body
            // is not revived or assigned invented object HP by this program.
            target_mut(state, area, index)?.applied_by = Some(meta.clone());
            if state
                .rules
                .as_ref()
                .and_then(|r| r.entities.get(&target.actor))
                .is_some_and(|e| !e.death.dead)
            {
                super::continuations::apply_vitality(
                    state,
                    meta,
                    target.actor,
                    work.occurrence,
                    operation,
                    Some(source.actor()),
                )?;
            }
            Ok(true)
        }
        TacticalWorkKind::FinishArea { area } => {
            if record(state, area)?
                .targets
                .iter()
                .any(|target| target.applied_by.is_none())
            {
                return Err(invalid("area completion omitted a target"));
            }
            record_mut(state, area)?.stage = TacticalAreaStage::Complete;
            Ok(true)
        }
        _ => unreachable!("checked area work"),
    }
}
pub(super) fn finish(
    state: &mut CampaignState,
    meta: &CommandMeta,
    pending: &TacticalPendingWork,
    result: Option<&RollResult>,
    forced_success: bool,
) -> Result<(), RulesError> {
    match pending.work.kind {
        TacticalWorkKind::AreaDamageRoll { area } => {
            if forced_success {
                return Err(invalid("save override cannot alter area damage"));
            }
            let result = result.ok_or_else(|| invalid("area damage requires raw dice"))?;
            request(state, &pending.work, pending.key)?
                .ok_or_else(|| invalid("area amount has no request"))?
                .resolve(result)?;
            let r = record_mut(state, area)?;
            if r.stage != TacticalAreaStage::DamageRoll || r.damage.is_some() {
                return Err(invalid("shared area amount already accepted"));
            }
            r.damage = Some(pending.key);
            r.stage = TacticalAreaStage::SavingThrows;
            let count = r.targets.len();
            push_frame(state, vec![TacticalWorkKind::BeginAreaDamage { area }])?;
            push_frame(
                state,
                (0..count)
                    .map(|target| TacticalWorkKind::AreaSave {
                        area,
                        target: target as u16,
                    })
                    .collect(),
            )?;
        }
        TacticalWorkKind::AreaSave {
            area,
            target: index,
        } => {
            let succeeded = forced_success || !save_failed(state, pending, result)?;
            let target = target_mut(state, area, index)?;
            if target.save.is_some() {
                return Err(invalid("area save already accepted"));
            }
            target.save = Some(TacticalAreaSave {
                key: pending.key,
                succeeded,
                resolved_by: meta.clone(),
            });
        }
        _ => return Err(invalid("area phase transition cannot await dice")),
    }
    Ok(())
}
