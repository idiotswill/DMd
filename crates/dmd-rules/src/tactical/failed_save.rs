//! Legendary Resistance changes a source-validated outcome before its consequences.
//! Raw dice and voluntary-failure evidence remain immutable, including natural 1s.
use super::turns::*;
use super::*;
use crate::tactical_creatures::{
    CreatureFailedSaveProof, creature_legendary_resistance_available,
    use_creature_legendary_resistance,
};

pub(super) fn is_failure(
    state: &CampaignState,
    pending: &TacticalPendingWork,
    result: Option<&RollResult>,
) -> Result<bool, RulesError> {
    if !matches!(
        pending.key.role,
        TacticalRollRole::DeathSave
            | TacticalRollRole::EffectSave
            | TacticalRollRole::Concentration
    ) {
        return Ok(false);
    }
    let Some(result) = result else {
        return Ok(true);
    };
    let request = super::continuations::request(state, &pending.work, pending.key)?
        .ok_or_else(|| invalid("automatic failure has no raw roll"))?;
    let roll = request.resolve(result)?;
    match &pending.work.kind {
        TacticalWorkKind::DeathSave { .. } => {
            Ok(roll.kept_dice[0].value != 20 && (roll.kept_dice[0].value == 1 || roll.total < 10))
        }
        TacticalWorkKind::ConcentrationSave { damage_taken, .. } => {
            Ok(i64::from(roll.total) < i64::from((damage_taken / 2).clamp(10, 30)))
        }
        TacticalWorkKind::Effect { ticket } => {
            match super::continuations::ticket(state, *ticket)?.payload {
                EffectTriggerPayload::SavingThrow { dc, .. } => Ok(roll.total < i32::from(dc)),
                _ => Err(invalid("failed save source has no saving throw")),
            }
        }
        _ => Err(invalid("failed save role differs from source work")),
    }
}

pub(super) fn available(state: &CampaignState, actor: EntityId) -> Result<bool, RulesError> {
    let Some(current) = state
        .rules
        .as_ref()
        .and_then(|r| r.tactical_creatures.as_ref())
    else {
        return Ok(false);
    };
    if current.profile(actor).is_none() {
        return Ok(false);
    }
    creature_legendary_resistance_available(state, current, actor)
        .map_err(|e| invalid(&e.to_string()))
}

/// Returns a sealed token only for the exact live pause and accepted failed outcome.
/// An arbitrary old failed roll, caller-supplied DC, or forged success flag cannot be
/// converted into a source budget expenditure.
pub fn validate_failed_save(
    state: &CampaignState,
    failed: &TacticalFailedSave,
) -> Result<CreatureFailedSaveProof, RulesError> {
    let r = resolution(state)?;
    let encounter_id = encounter(state)?.id;
    let rules = state.rules.as_ref().ok_or(RulesError::Uninitialized)?;
    let p = &failed.pending;
    super::turn_validation::selected_applicable(state, &p.work)?;
    if r.failed_save.as_ref() != Some(failed)
        || r.pending.is_some()
        || r.legendary_window.is_some()
        || rules.pending.is_some()
        || p.key != super::continuations::key(state, &p.work)?
        || failed.resolved_by.expected_event_sequence > state.applied_event_sequence
        || failed.issued_by.expected_event_sequence > failed.resolved_by.expected_event_sequence
        || failed.issued_by.expected_event_sequence < r.origin.expected_event_sequence
        || !is_failure(state, p, failed.result.as_ref())?
        || !available(state, p.key.subject)?
    {
        return Err(invalid(
            "failed save does not identify a live source resistance decision",
        ));
    }
    validate_equipment_change_origin(state, &failed.issued_by, p.key.subject)
        .map_err(|e| invalid(&e))?;
    validate_equipment_change_origin(state, &failed.resolved_by, p.key.subject)
        .map_err(|e| invalid(&e))?;
    let request = super::continuations::request(state, &p.work, p.key)?;
    if let Some(raw) = &failed.result {
        let request = request.ok_or_else(|| invalid("automatic failure supplied raw dice"))?;
        authorize(state, &failed.resolved_by, p.key.subject)?;
        if (request.visibility == RollVisibility::Secret || raw.source == RollSource::Digital)
            && !matches!(
                failed.resolved_by.issuer,
                CommandIssuer::Admin | CommandIssuer::System
            )
        {
            return Err(RulesError::Unauthorized);
        }
        let resolved = request.resolve(raw)?;
        if !rules.rolls.iter().any(|roll| {
            roll.request == request
                && roll.result == *raw
                && roll.issued_by == failed.issued_by
                && roll.accepted_by == failed.resolved_by
                && roll.resolved == resolved
                && roll.purpose
                    == (PendingPurpose::TacticalResolution {
                        encounter: encounter_id,
                        key: p.key,
                    })
        }) {
            return Err(invalid("failed save lacks matching accepted raw dice"));
        }
    } else {
        let expected = if request.is_some() {
            TacticalSaveFailure::Voluntary
        } else {
            TacticalSaveFailure::Automatic
        };
        if !flow(state)?.save_decisions.iter().any(|d| {
            d.key == p.key
                && d.issued_by == failed.issued_by
                && d.resolved_by == failed.resolved_by
                && d.failure == expected
        }) {
            return Err(invalid("failed save lacks matching non-rolled decision"));
        }
    }
    Ok(CreatureFailedSaveProof::from_validated_resolution(
        state.campaign_id(),
        state.applied_event_sequence,
        p.key.subject,
        p.key.request_id(),
    ))
}

pub(super) fn stage_or_finish(
    state: &mut CampaignState,
    meta: &CommandMeta,
    pending: TacticalPendingWork,
    result: Option<&RollResult>,
) -> Result<(), RulesError> {
    if is_failure(state, &pending, result)? && available(state, pending.key.subject)? {
        let issued_by = if result.is_some() {
            state
                .rules
                .as_ref()
                .and_then(|r| {
                    r.rolls
                        .iter()
                        .find(|roll| roll.request.id == pending.key.request_id())
                })
                .map(|r| r.issued_by.clone())
        } else {
            flow(state)?
                .save_decisions
                .iter()
                .find(|d| d.key == pending.key)
                .map(|d| d.issued_by.clone())
        }
        .ok_or_else(|| invalid("failed save receipt is absent"))?;
        resolution_mut(state)?.pending = None;
        resolution_mut(state)?.failed_save = Some(TacticalFailedSave {
            pending,
            issued_by,
            resolved_by: meta.clone(),
            result: result.cloned(),
        });
        return Ok(());
    }
    super::continuations::finish(state, meta, pending, result, false)
}

pub(super) fn choose(
    state: &mut CampaignState,
    meta: &CommandMeta,
    use_resistance: bool,
) -> Result<(), RulesError> {
    let failed = resolution(state)?
        .failed_save
        .clone()
        .ok_or_else(|| prerequisite("no Legendary Resistance decision is due"))?;
    let proof = validate_failed_save(state, &failed)?;
    // Creature authority is distinct from the active turn's controller. The source
    // adapter checks autonomous/summoned controller binding; declining needs it too.
    let current = state
        .rules
        .as_ref()
        .and_then(|r| r.tactical_creatures.as_ref())
        .ok_or_else(|| invalid("source creature attachment absent"))?;
    let consumed = use_creature_legendary_resistance(state, current, meta, &proof)
        .map_err(|e| invalid(&e.to_string()))?;
    if use_resistance {
        state
            .rules
            .as_mut()
            .ok_or(RulesError::Uninitialized)?
            .tactical_creatures = Some(consumed);
    }
    resolution_mut(state)?.failed_save = None;
    super::continuations::finish(
        state,
        meta,
        failed.pending,
        failed.result.as_ref(),
        use_resistance,
    )?;
    pump(state, meta)
}
