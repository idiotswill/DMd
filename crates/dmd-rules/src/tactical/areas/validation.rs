use super::*;
use std::collections::HashSet;

pub(in crate::tactical) fn validate_work(
    state: &CampaignState,
    work: &TacticalWorkItem,
) -> Result<EntityId, RulesError> {
    let (area, index) = identity(&work.kind)?;
    let r = record(state, area)?;
    let valid = match work.kind {
        TacticalWorkKind::AreaDamageRoll { .. } => {
            r.stage == TacticalAreaStage::DamageRoll && r.damage.is_none()
        }
        TacticalWorkKind::AreaSave { .. } => {
            r.stage == TacticalAreaStage::SavingThrows
                && target(state, area, index.expect("save index"))?
                    .save
                    .is_none()
        }
        TacticalWorkKind::BeginAreaDamage { .. } => r.stage == TacticalAreaStage::SavingThrows,
        TacticalWorkKind::ApplyAreaDamage { .. } => {
            r.stage == TacticalAreaStage::ApplyingDamage
                && target(state, area, index.expect("damage index"))?
                    .applied_by
                    .is_none()
        }
        TacticalWorkKind::FinishArea { .. } => r.stage != TacticalAreaStage::Complete,
        _ => false,
    };
    if !valid {
        return Err(invalid(
            "area work differs from its retained phase or target",
        ));
    }
    Ok(match index {
        Some(index) => target(state, area, index)?.actor,
        None => r.source.actor,
    })
}

fn provenance(
    state: &CampaignState,
    r: &TacticalArea,
    meta: &CommandMeta,
    subject: EntityId,
) -> Result<(), RulesError> {
    validate_equipment_change_origin(state, meta, subject).map_err(|e| invalid(&e))?;
    if meta.expected_event_sequence < r.source.invocation.expected_event_sequence
        || (meta.expected_event_sequence == r.source.invocation.expected_event_sequence
            && *meta != r.source.invocation)
    {
        return Err(invalid(
            "area consequence predates or changes its accepted invocation",
        ));
    }
    Ok(())
}
fn accepted<'a>(
    state: &'a CampaignState,
    r: &TacticalArea,
    key: TacticalRollKey,
    role: TacticalRollRole,
    subject: EntityId,
) -> Result<Option<&'a RecordedRoll>, RulesError> {
    if key.origin != r.source.invocation.id
        || key.role != role
        || key.subject != subject
        || key.occurrence <= r.occurrence
        || key.occurrence >= resolution(state)?.next_occurrence
    {
        return Err(invalid("area receipt has a foreign or future raw identity"));
    }
    let found = state
        .rules
        .as_ref()
        .ok_or(RulesError::Uninitialized)?
        .rolls
        .iter()
        .find(|roll| roll.request.id == key.request_id());
    if let Some(roll) = found {
        if roll.purpose
            != (PendingPurpose::TacticalResolution {
                encounter: encounter(state)?.id,
                key,
            })
            || roll.request.roller != Some(subject)
            || roll.resolved != roll.request.resolve(&roll.result)?
        {
            return Err(invalid(
                "area raw evidence differs from its accepted source role",
            ));
        }
        provenance(state, r, &roll.issued_by, subject)?;
        provenance(state, r, &roll.accepted_by, subject)?;
    }
    Ok(found)
}

/// Historical geometry/grants are additionally authenticated by app replay from
/// the pre-tactical anchor. This validator closes structural/source/work forgery
/// without re-evaluating a prior save or membership after damage/falling changes.
pub(in crate::tactical) fn validate(state: &CampaignState) -> Result<(), RulesError> {
    let Some(cursor) = flow(state)?.resolution.as_deref() else {
        return Ok(());
    };
    // This admitted slice creates one direct own-turn source Action. Nested
    // Multiattack/reaction activations need their own authenticated receipt path.
    if cursor.areas.len() > 1 {
        return Err(invalid("area occurrence capacity exceeded"));
    }
    let rules = state.rules.as_ref().ok_or(RulesError::Uninitialized)?;
    let mut area_ids = HashSet::new();
    for r in &cursor.areas {
        let source = program(state, r.occurrence)?;
        if r.occurrence != 0
            || !area_ids.insert(r.occurrence)
            || r.occurrence >= cursor.next_occurrence
            || r.source.invocation != cursor.origin
            || r.source.enclosing_origin != r.source.invocation
            || r.source.actor != cursor.turn_actor
            || r.geometry_origin != encounter(state)?.origin
            || encounter(state)?.area_grid_policy != Some(r.policy)
            || !rules
                .timing
                .as_ref()
                .is_some_and(|timing| timing.action_spent)
            || r.targets.len() > encounter(state)?.participants.len()
        {
            return Err(invalid(
                "area source, geometry, activation or capacity differs",
            ));
        }
        authorize(state, &r.source.invocation, r.source.actor)?;
        provenance(state, r, &r.source.invocation, r.source.actor)?;
        r.aim.origin.validate().map_err(|e| invalid(&e))?;
        r.aim.toward.validate().map_err(|e| invalid(&e))?;
        if r.aim.origin == r.aim.toward {
            return Err(invalid("area direction is empty"));
        }
        let runtime = rules
            .tactical_creatures
            .as_ref()
            .and_then(|c| c.runtime(r.source.actor))
            .ok_or_else(|| invalid("area source runtime is absent"))?;
        if r.stage == TacticalAreaStage::DamageRoll
            && (runtime.last_operation != r.source.invocation
                || runtime.routine.is_some()
                || runtime.observed_turn
                    != Some(CreatureTurn {
                        encounter_id: encounter(state)?.id,
                        actor: r.source.actor,
                        number: cursor.turn_number,
                        boundary: TurnBoundary::Start,
                    }))
        {
            return Err(invalid("area lacks its actual source activation receipt"));
        }
        // Admission requires settled physical consequences; amount and all saves
        // then precede damage. Reconstruct exact membership/cover in those phases.
        // Once damage begins, subsequent falls cannot rewrite the original cone.
        if matches!(
            r.stage,
            TacticalAreaStage::DamageRoll | TacticalAreaStage::SavingThrows
        ) {
            if !cursor.falls.is_empty() {
                return Err(invalid(
                    "area pre-damage phase contains interleaved falling",
                ));
            }
            super::super::falling::require_settled_before_action(state)?;
            let bound = bind_area_geometry(encounter(state)?, state, &source, r.aim)?;
            let mut retained = r.targets.clone();
            for target in &mut retained {
                target.save = None;
                target.applied_by = None;
            }
            if bound.targets() != retained {
                return Err(invalid(
                    "area membership or cover differs from accepted geometry",
                ));
            }
        }
        if runtime
            .recharge
            .iter()
            .any(|recharge| recharge.feature_id == r.source.feature_id && recharge.available)
        {
            return Err(invalid("area source recharge was not spent"));
        }
        let mut actors = HashSet::new();
        let mut saves = HashSet::new();
        let mut damage = HashSet::new();
        for (index, target) in r.targets.iter().enumerate() {
            if !actors.insert(target.actor)
                || !rules.entities.contains_key(&target.actor)
                || encounter(state)?.participant(target.actor).is_none()
                || target.cover == CoverDegree::Total
            {
                return Err(invalid("invalid or duplicate area membership"));
            }
            target.volume.validate().map_err(|e| invalid(&e))?;
            if let Some(save) = &target.save {
                saves.insert(index as u16);
                provenance(state, r, &save.resolved_by, target.actor)?;
                let raw = accepted(state, r, save.key, TacticalRollRole::AreaSave, target.actor)?;
                if r.stage == TacticalAreaStage::SavingThrows {
                    let expected = request(
                        state,
                        &TacticalWorkItem {
                            occurrence: save.key.occurrence,
                            kind: TacticalWorkKind::AreaSave {
                                area: r.occurrence,
                                target: index as u16,
                            },
                        },
                        save.key,
                    )?;
                    if let Some(raw) = raw {
                        if expected.as_ref() != Some(&raw.request) {
                            return Err(invalid(
                                "completed area save differs from its source request",
                            ));
                        }
                    } else {
                        let expected_failure = if expected.is_some() {
                            TacticalSaveFailure::Voluntary
                        } else {
                            TacticalSaveFailure::Automatic
                        };
                        if !flow(state)?.save_decisions.iter().any(|decision| {
                            decision.key == save.key && decision.failure == expected_failure
                        }) {
                            return Err(invalid(
                                "completed area failure differs from its source disposition",
                            ));
                        }
                    }
                }
                let ordinary = if let Some(raw) = raw {
                    if save.resolved_by.expected_event_sequence
                        < raw.accepted_by.expected_event_sequence
                    {
                        return Err(invalid(
                            "area save resolution predates its accepted raw evidence",
                        ));
                    }
                    crate::test_outcome::ability_test_success(
                        &raw.resolved,
                        i32::from(source.dc()),
                        &rules.house_rules,
                    )?
                } else {
                    let decision = flow(state)?
                        .save_decisions
                        .iter()
                        .find(|decision| decision.key == save.key)
                        .ok_or_else(|| {
                            invalid("area save lacks actual dice or a chosen/automatic failure")
                        })?;
                    if save.resolved_by.expected_event_sequence
                        < decision.resolved_by.expected_event_sequence
                    {
                        return Err(invalid(
                            "area save resolution predates its failure decision",
                        ));
                    }
                    false
                };
                let resistance = rules
                    .tactical_creatures
                    .as_ref()
                    .and_then(|c| c.runtime(target.actor))
                    .is_some_and(|runtime| {
                        runtime
                            .legendary_resistance_rolls
                            .contains(&save.key.request_id())
                    });
                if save.succeeded != (ordinary || resistance) {
                    return Err(invalid("area save outcome differs from accepted evidence"));
                }
            }
            if let Some(meta) = &target.applied_by {
                if target.save.as_ref().is_none_or(|save| {
                    meta.expected_event_sequence < save.resolved_by.expected_event_sequence
                }) {
                    return Err(invalid("area damage preceded its save"));
                }
                provenance(state, r, meta, target.actor)?;
                damage.insert(index as u16);
            }
        }
        match (r.stage, r.damage) {
            (TacticalAreaStage::DamageRoll, None) if saves.is_empty() && damage.is_empty() => (),
            (TacticalAreaStage::DamageRoll, _) | (_, None) => {
                return Err(invalid("area phase differs from shared damage evidence"));
            }
            (_, Some(key)) => {
                let raw = accepted(state, r, key, TacticalRollRole::AreaDamage, r.source.actor)?
                    .ok_or_else(|| invalid("shared area amount lacks actual raw dice"))?;
                let expected =
                    area_amount_request(&source, key.request_id(), raw.request.visibility)?;
                if raw.request != expected {
                    return Err(invalid("shared amount differs from canonical source dice"));
                }
                expected.resolve(&raw.result)?;
            }
        }
        let mut amounts = 0;
        let mut phases = 0;
        let mut finishes = 0;
        let mut phase_frame = None;
        for (frame_index, work) in cursor
            .frames
            .iter()
            .enumerate()
            .flat_map(|(i, f)| f.iter().map(move |w| (Some(i), w)))
            .chain(cursor.pending.iter().map(|p| (None, &p.work)))
            .chain(cursor.failed_save.iter().map(|p| (None, &p.pending.work)))
        {
            if !is_work(&work.kind) || identity(&work.kind)?.0 != r.occurrence {
                continue;
            }
            validate_work(state, work)?;
            match work.kind {
                TacticalWorkKind::AreaDamageRoll { .. } => amounts += 1,
                TacticalWorkKind::AreaSave { target, .. } => {
                    if !saves.insert(target) {
                        return Err(invalid("duplicate area save work"));
                    }
                }
                TacticalWorkKind::ApplyAreaDamage { target, .. } => {
                    if !damage.insert(target) {
                        return Err(invalid("duplicate area damage work"));
                    }
                }
                TacticalWorkKind::BeginAreaDamage { .. } => {
                    phases += 1;
                    phase_frame = frame_index;
                }
                TacticalWorkKind::FinishArea { .. } => {
                    finishes += 1;
                    if frame_index != Some(0) {
                        return Err(invalid("area completion is not below its source work"));
                    }
                }
                _ => unreachable!("area work"),
            }
        }
        // Every save must remain above the phase transition. A forged frame order
        // must fail before any subset of targets can start taking damage.
        if let Some(phase_frame) = phase_frame {
            for (index, frame) in cursor.frames.iter().enumerate() {
                if index <= phase_frame && frame.iter().any(|work| matches!(work.kind, TacticalWorkKind::AreaSave { area, .. } if area == r.occurrence)) {
                    return Err(invalid("area save is queued beneath its damage phase"));
                }
            }
        }
        let count = r.targets.len();
        let valid = match r.stage {
            TacticalAreaStage::DamageRoll => {
                amounts == 1
                    && phases == 0
                    && finishes == 1
                    && saves.is_empty()
                    && damage.is_empty()
            }
            TacticalAreaStage::SavingThrows => {
                amounts == 0
                    && phases == 1
                    && finishes == 1
                    && saves.len() == count
                    && damage.is_empty()
            }
            TacticalAreaStage::ApplyingDamage => {
                amounts == 0
                    && phases == 0
                    && finishes == 1
                    && r.targets.iter().all(|t| t.save.is_some())
                    && damage.len() == count
            }
            TacticalAreaStage::Complete => {
                amounts == 0
                    && phases == 0
                    && finishes == 0
                    && r.targets.iter().all(|t| t.applied_by.is_some())
            }
        };
        if !valid {
            return Err(invalid("area work omitted or duplicated a target or phase"));
        }
    }
    Ok(())
}
