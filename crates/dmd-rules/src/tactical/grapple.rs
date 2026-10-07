//! Guarded ordinary Grapple core. Public Live and Historical dispatch deny this
//! entire family until temporal consumers and original replay are implemented.
pub(crate) mod admission;
mod equipment;
pub(crate) mod execution;
mod lifecycle;
mod modern_lifecycle;
mod offers;
pub(crate) use offers::{action as table_action, choices as table_choices};
mod profile;
pub(crate) mod reads;
mod saves;
pub(crate) mod transport;
mod validation;
use super::turns::*;
use super::*;
use execution::ExecutionContext;

pub(super) use equipment::choose_with_context as choose_equipment_with_context;
#[cfg(test)]
pub(super) use equipment::{apply_after_equipment, decline_after_equipment};
#[cfg(test)]
pub(super) use lifecycle::{begin_escape, release, withdraw};
pub(super) use lifecycle::{
    begin_escape_with_context, release_with_context, withdraw_with_context,
};
#[cfg(test)]
pub(super) use saves::choose_save;
pub(super) use saves::choose_save_with_context;
pub(super) use saves::{finish, request, save_failed};
pub(super) use validation::{validate, validate_work};
pub(super) fn flight_cause(
    state: &mut CampaignState,
    meta: &CommandMeta,
    actor: EntityId,
) -> Result<Option<TacticalFallCause>, RulesError> {
    if crate::table::grapple_enabled(state) {
        modern_lifecycle::flight_cause(state, meta, actor)
    } else {
        Ok(None)
    }
}
pub(super) fn validate_flight(
    state: &CampaignState,
    fall: &TacticalFall,
) -> Result<(), RulesError> {
    if !crate::table::grapple_enabled(state) {
        return Err(invalid("Grapple flight-loss execution is not enabled."));
    }
    modern_lifecycle::validate_flight(state, fall)
}
pub(super) fn capture_self_only_movement(
    state: &mut CampaignState,
    meta: &CommandMeta,
) -> Result<(), RulesError> {
    if !crate::table::grapple_enabled(state) {
        return Ok(());
    }
    let movement = resolution(state)?
        .movement
        .as_ref()
        .ok_or_else(|| invalid("self-only movement absent"))?;
    if movement.origin != *meta || !movement.traversed.is_empty() {
        return Err(invalid(
            "self-only admission is not its actual move producer",
        ));
    }
    let proofs = state
        .rules
        .as_ref()
        .and_then(|r| r.tactical_grapples.as_ref())
        .map(|live| {
            live.active
                .iter()
                .filter(|g| g.declaration.grappler == movement.actor)
                .cloned()
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    if proofs.is_empty() {
        return Ok(());
    }
    for proof in &proofs {
        modern_lifecycle::retain(state, proof)?;
    }
    resolution_mut(state)?
        .movement
        .as_mut()
        .ok_or_else(|| invalid("movement absent"))?
        .grapple_self_only = Some(GrappleSelfOnlyAdmission {
        origin: meta.clone(),
        grips: proofs.iter().map(|g| g.declaration.id).collect(),
    });
    Ok(())
}

fn require_execution(state: &CampaignState) -> Result<(), RulesError> {
    if flow(state)?.version != TacticalExecutionVersion::EncounterReleaseV1.flow_version() {
        return Err(prerequisite(
            "Grapple requires encounter release execution.",
        ));
    }
    Ok(())
}

pub(super) fn guard_action(
    state: &CampaignState,
    action: &TacticalAction,
) -> Result<(), RulesError> {
    if matches!(
        action,
        TacticalAction::Grapple { .. }
            | TacticalAction::MoveSelfOnly { .. }
            | TacticalAction::MoveGrappled { .. }
            | TacticalAction::ChooseGrappleSave { .. }
            | TacticalAction::ApplyGrappleAfterEquipment { .. }
            | TacticalAction::DeclineGrappleAfterEquipment { .. }
            | TacticalAction::WithdrawGrapple { .. }
            | TacticalAction::EscapeGrapple { .. }
            | TacticalAction::ReleaseGrapple { .. }
    ) {
        require_execution(state)?;
        return Err(prerequisite(
            "Grapple execution is not enabled by this guarded core checkpoint.",
        ));
    }
    Ok(())
}

fn context(state: &CampaignState) -> Result<&TacticalGrappleResolution, RulesError> {
    resolution(state)?
        .grapple
        .as_deref()
        .ok_or_else(|| invalid("Grapple context absent"))
}
fn context_mut(state: &mut CampaignState) -> Result<&mut TacticalGrappleResolution, RulesError> {
    resolution_mut(state)?
        .grapple
        .as_deref_mut()
        .ok_or_else(|| invalid("Grapple context absent"))
}
fn attempt(state: &CampaignState) -> Result<&TacticalGrappleAttempt, RulesError> {
    match context(state)?.activity.as_ref() {
        Some(GrappleActivity::Attempt(a)) => Ok(a),
        _ => Err(invalid("Grapple Attempt absent")),
    }
}
fn attempt_mut(state: &mut CampaignState) -> Result<&mut TacticalGrappleAttempt, RulesError> {
    match context_mut(state)?.activity.as_mut() {
        Some(GrappleActivity::Attempt(a)) => Ok(a),
        _ => Err(invalid("Grapple Attempt absent")),
    }
}
fn escape(state: &CampaignState) -> Result<&TacticalGrappleEscape, RulesError> {
    match context(state)?.activity.as_ref() {
        Some(GrappleActivity::Escape(a)) => Ok(a),
        _ => Err(invalid("Grapple Escape absent")),
    }
}
fn escape_mut(state: &mut CampaignState) -> Result<&mut TacticalGrappleEscape, RulesError> {
    match context_mut(state)?.activity.as_mut() {
        Some(GrappleActivity::Escape(a)) => Ok(a),
        _ => Err(invalid("Grapple Escape absent")),
    }
}
fn work_key(state: &CampaignState, work: &TacticalWorkItem) -> Result<TacticalWorkKey, RulesError> {
    Ok(TacticalWorkKey {
        resolution: resolution(state)?.origin.id,
        occurrence: work.occurrence,
    })
}
fn new_context(activity: GrappleActivity) -> TacticalGrappleResolution {
    TacticalGrappleResolution {
        transport: None,
        activity: Some(activity),
        proofs: vec![],
        cuts: vec![],
        ends: vec![],
        opportunity_refreshes: vec![],
        completed_casts: vec![],
    }
}
fn install_resolution(
    state: &mut CampaignState,
    meta: &CommandMeta,
    actor: EntityId,
    grapple: TacticalGrappleResolution,
) -> Result<(), RulesError> {
    let turn_number = state
        .rules
        .as_ref()
        .and_then(|r| r.timing.as_ref())
        .ok_or_else(|| invalid("Grapple turn absent"))?
        .turn_number;
    let work_trace = super::work_trace::initial(state)?;
    flow_mut(state)?.resolution = Some(Box::new(TacticalResolution {
        attack_after_equipment: None,
        origin: meta.clone(),
        turn_actor: actor,
        turn_number,
        boundary: TurnBoundary::Start,
        frames: vec![],
        pending: None,
        failed_save: None,
        legendary_window: None,
        attack: None,
        shove: None,
        hit_review: None,
        movement: None,
        casts: vec![],
        missiles: vec![],
        falls: vec![],
        areas: vec![],
        work_trace,
        next_occurrence: 0,
        grapple: Some(Box::new(grapple)),
    }));
    Ok(())
}

/// Private entry points also validate on a clone: their unsupported-context
/// refusal must not leave a partly paid private result in guarded tests.
fn transaction<'a>(
    state: &mut CampaignState,
    meta: &CommandMeta,
    execution: &mut ExecutionContext<'a>,
    apply: impl FnOnce(&mut CampaignState, &mut ExecutionContext<'a>) -> Result<(), RulesError>,
) -> Result<(), RulesError> {
    require_execution(state)?;
    if meta.campaign_id != state.campaign_id() {
        return Err(RulesError::Unauthorized);
    }
    if meta.expected_event_sequence != state.applied_event_sequence {
        return Err(RulesError::Stale);
    }
    execution.transaction(state, apply)
}

#[cfg(test)]
pub(super) fn begin(
    state: &mut CampaignState,
    meta: &CommandMeta,
    target: EntityId,
    hand: Hand,
    before_change: Option<AttackEquipmentOperation>,
    pack: &RulesPack,
) -> Result<(), RulesError> {
    begin_with_context(
        state,
        meta,
        target,
        hand,
        before_change,
        pack,
        &mut ExecutionContext::ordinary(),
    )
}

pub(super) fn begin_with_context(
    state: &mut CampaignState,
    meta: &CommandMeta,
    target: EntityId,
    hand: Hand,
    before_change: Option<AttackEquipmentOperation>,
    pack: &RulesPack,
    execution: &mut ExecutionContext<'_>,
) -> Result<(), RulesError> {
    transaction(state, meta, execution, |next, execution| {
        let plan = admission::plan_attempt_with_read(
            &execution.read(next)?,
            meta,
            target,
            hand,
            before_change,
            pack,
        )?;
        let actor = plan.attempt.declaration.grappler;
        let id = plan.attempt.declaration.id;
        let now = next.clock.now;
        let rules = next.rules.as_mut().ok_or(RulesError::Uninitialized)?;
        let mut budget = plan.budget;
        if plan.new_action {
            crate::tactical_budget::start_attack_action(rules, &mut budget, actor, 1)?;
        }
        crate::tactical_budget::spend_attack(&mut budget)?;
        crate::kernel::interrupt_rest(rules, actor, now);
        if before_change.is_some() {
            equipment::write_loadout(rules, plan.loadout)?;
        }
        budget.movement_progress = None;
        budget.movement_origin = None;
        flow_mut(next)?.budget = budget;
        install_resolution(
            next,
            meta,
            actor,
            new_context(GrappleActivity::Attempt(Box::new(plan.attempt))),
        )?;
        push_frame(next, vec![TacticalWorkKind::BeginGrapple { grip: id }])?;
        super::turns::pump_with_context(next, meta, execution)
    })
}

pub(super) fn waiting(state: &CampaignState) -> bool {
    attempt(state).is_ok_and(|a| a.selected.is_some())
}
pub(super) fn is_work(kind: &TacticalWorkKind) -> bool {
    matches!(
        kind,
        TacticalWorkKind::BeginGrapple { .. }
            | TacticalWorkKind::GrappleSave { .. }
            | TacticalWorkKind::GrappleAfterEquipment { .. }
            | TacticalWorkKind::GrappleEscapeCheck { .. }
    )
}
pub(super) fn start(
    state: &mut CampaignState,
    work: &TacticalWorkItem,
) -> Result<bool, RulesError> {
    match work.kind {
        TacticalWorkKind::BeginGrapple { grip } => {
            let a = attempt_mut(state)?;
            if a.declaration.id != grip || a.stage != TacticalGrappleAttemptStage::Queued {
                return Err(invalid("Grapple start stage differs"));
            }
            a.stage = TacticalGrappleAttemptStage::SaveChoice;
            a.selected = Some(work.clone());
        }
        TacticalWorkKind::GrappleAfterEquipment { grip } => {
            let a = attempt_mut(state)?;
            if a.declaration.id != grip
                || a.stage != TacticalGrappleAttemptStage::AfterEquipment
                || a.selected.is_some()
            {
                return Err(invalid("Grapple equipment stage differs"));
            }
            a.selected = Some(work.clone());
        }
        TacticalWorkKind::GrappleEscapeCheck { grip } => {
            let e = escape_mut(state)?;
            if e.grip != grip || e.stage != TacticalGrappleEscapeStage::Queued {
                return Err(invalid("Escape start stage differs"));
            }
            e.stage = TacticalGrappleEscapeStage::Rolling;
            return Ok(false);
        }
        _ => return Ok(false),
    }
    Ok(true)
}
pub(super) fn key(
    state: &CampaignState,
    work: &TacticalWorkItem,
) -> Result<TacticalRollKey, RulesError> {
    match work.kind {
        TacticalWorkKind::GrappleSave { grip } if attempt(state)?.declaration.id == grip => {
            let a = attempt(state)?;
            Ok(TacticalRollKey {
                origin: a.declaration.origin.id,
                role: TacticalRollRole::GrappleSave,
                subject: a.declaration.target,
                occurrence: work.occurrence,
            })
        }
        TacticalWorkKind::GrappleEscapeCheck { grip } if escape(state)?.grip == grip => {
            let e = escape(state)?;
            Ok(TacticalRollKey {
                origin: e.origin.id,
                role: TacticalRollRole::GrappleEscape,
                subject: e.actor,
                occurrence: work.occurrence,
            })
        }
        _ => Err(invalid("Grapple choice has no raw key")),
    }
}
pub(super) fn authorize_pending(
    state: &CampaignState,
    meta: &CommandMeta,
    key: TacticalRollKey,
) -> Result<(), RulesError> {
    if matches!(
        key.role,
        TacticalRollRole::GrappleSave | TacticalRollRole::GrappleEscape
    ) {
        super::shove::authorize_owner(state, meta, key.subject)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests;
