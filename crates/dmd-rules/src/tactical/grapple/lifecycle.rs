use super::*;

pub(super) fn live_grip(state: &CampaignState, id: GrappleId) -> Result<&TacticalGrip, RulesError> {
    state
        .rules
        .as_ref()
        .and_then(|r| r.tactical_grapples.as_ref())
        .and_then(|g| g.grip(id))
        .ok_or_else(|| prerequisite("This live grip is absent."))
}

pub(super) fn end_grip(
    state: &mut CampaignState,
    meta: &CommandMeta,
    id: GrappleId,
    cause: GrappleEndCause,
) -> Result<(), RulesError> {
    let grip = live_grip(state, id)?.clone();
    if crate::table::grapple_enabled(state) && flow(state)?.resolution.is_some() {
        modern_lifecycle::retain(state, &grip)?;
    }
    if flow(state)?
        .resolution
        .as_ref()
        .and_then(|r| r.grapple.as_ref())
        .is_some_and(|c| {
            crate::table::grapple_enabled(state)
                || c.activity.is_some()
                || c.cuts.iter().any(|cut| cut.grips.contains(&id))
        })
    {
        let c = context_mut(state)?;
        if !c.proofs.iter().any(|g| g.declaration.id == id) {
            c.proofs.push(grip);
            c.proofs.sort_by_key(|g| g.declaration.id.0);
        }
        if c.ends.iter().any(|e| e.grip == id) {
            return Err(invalid("Grip already has an end receipt"));
        }
        c.ends.push(GrappleEndReceipt {
            grip: id,
            caused_by: meta.clone(),
            cause,
        });
    }
    let rules = state.rules.as_mut().ok_or(RulesError::Uninitialized)?;
    let live = rules
        .tactical_grapples
        .as_mut()
        .ok_or_else(|| invalid("Live grip context absent"))?;
    live.active.retain(|g| g.declaration.id != id);
    if live.active.is_empty() {
        rules.tactical_grapples = None;
    }
    if crate::table::grapple_enabled(state) && flow(state)?.resolution.is_some() {
        super::super::movement::refresh_after_grip_end(state, meta, id)?;
    }
    Ok(())
}

/// Retire only a matching *unfinished* request. A prior voluntary failure may
/// already have canceled this ID; accepted dice must never acquire cancellation.
#[cfg(test)]
fn cancel_pending(
    state: &mut CampaignState,
    key: TacticalRollKey,
) -> Result<Option<RollRequestId>, RulesError> {
    // Legacy private controls have no continuity authority to mint. The actual
    // typed dispatcher always supplies its real lifecycle command below.
    let meta = resolution(state)?.origin.clone();
    cancel_pending_with_context(state, &meta, key, &mut ExecutionContext::ordinary())
}

fn cancel_pending_with_context(
    state: &mut CampaignState,
    meta: &CommandMeta,
    key: TacticalRollKey,
    execution: &mut ExecutionContext<'_>,
) -> Result<Option<RollRequestId>, RulesError> {
    let Some(p) = resolution(state)?.pending.clone() else {
        return Ok(None);
    };
    if p.key != key {
        return Err(invalid("Withdrawal names another pending work"));
    }
    let rules = state.rules.as_ref().ok_or(RulesError::Uninitialized)?;
    let raw = rules
        .pending
        .as_ref()
        .ok_or_else(|| invalid("Selected Grapple request absent"))?;
    if raw.request.id != key.request_id()
        || !matches!(raw.purpose,PendingPurpose::TacticalResolution { key: actual,.. } if actual==key)
        || rules.rolls.iter().any(|r| r.request.id == raw.request.id)
        || rules.cancelled_roll_ids.contains(&raw.request.id)
    {
        return Err(invalid("Grapple cancellation identity differs"));
    }
    let id = raw.request.id;
    execution.observe_cancellation(state, meta, &p)?;
    let rules = state.rules.as_mut().ok_or(RulesError::Uninitialized)?;
    rules.pending = None;
    rules.cancelled_roll_ids.push(id);
    resolution_mut(state)?.pending = None;
    Ok(Some(id))
}

#[cfg(test)]
pub(in crate::tactical) fn withdraw(
    state: &mut CampaignState,
    meta: &CommandMeta,
    id: GrappleId,
) -> Result<(), RulesError> {
    withdraw_with_context(state, meta, id, &mut ExecutionContext::ordinary())
}

pub(in crate::tactical) fn withdraw_with_context(
    state: &mut CampaignState,
    meta: &CommandMeta,
    id: GrappleId,
    execution: &mut ExecutionContext<'_>,
) -> Result<(), RulesError> {
    transaction(state, meta, execution, |next, execution| {
        validate(next)?;
        let a = attempt(next)?.clone();
        if a.declaration.id != id || a.reservation().is_none() {
            return Err(prerequisite(
                "Only the actual pending Attempt may be withdrawn.",
            ));
        }
        super::super::shove::authorize_owner(next, meta, a.declaration.grappler)?;
        let selected = a
            .selected
            .clone()
            .or_else(|| {
                resolution(next)
                    .ok()?
                    .pending
                    .as_ref()
                    .map(|p| p.work.clone())
            })
            .or_else(|| {
                resolution(next)
                    .ok()?
                    .failed_save
                    .as_ref()
                    .map(|f| f.pending.work.clone())
            })
            .ok_or_else(|| invalid("Withdrawal lacks its actual work"))?;
        let previous = super::super::work_trace::enter(next, &selected)?;
        let result = (|| {
            let cancelled = if let Some(save) = &a.save {
                cancel_pending_with_context(next, meta, save.key, execution)?
            } else {
                None
            };
            if let Some(failed) = &resolution(next)?.failed_save {
                if a.save.as_ref().is_none_or(|s| s.key != failed.pending.key) {
                    return Err(invalid("Withdrawal names another failed save"));
                }
                resolution_mut(next)?.failed_save = None;
            }
            attempt_mut(next)?.outcome = Some(GrappleAttemptOutcome::Withdrawn {
                withdrawn_by: meta.clone(),
                cancelled,
            });
            saves::complete_attempt(next)
        })();
        let reset = super::super::work_trace::leave(next, previous);
        result?;
        reset?;
        super::super::turns::pump_with_context(next, meta, execution)
    })
}

#[cfg(test)]
pub(in crate::tactical) fn release(
    state: &mut CampaignState,
    meta: &CommandMeta,
    id: GrappleId,
) -> Result<(), RulesError> {
    release_with_context(state, meta, id, &mut ExecutionContext::ordinary())
}

pub(in crate::tactical) fn release_with_context(
    state: &mut CampaignState,
    meta: &CommandMeta,
    id: GrappleId,
    execution: &mut ExecutionContext<'_>,
) -> Result<(), RulesError> {
    transaction(state, meta, execution, |next, execution| {
        validate(next)?;
        let holder = live_grip(next, id)?.declaration.grappler;
        super::super::shove::authorize_owner(next, meta, holder)?;
        // An unrelated Grapple activity is also outside this core's consumer set.
        if crate::table::grapple_enabled(next) {
            admission::supported_context(next)?;
        } else if let Ok(c) = context(next) {
            match c.activity.as_ref() {
                None => admission::supported_context(next)?,
                Some(GrappleActivity::Attempt(a))
                    if a.declaration.id == id
                        && a.stage == TacticalGrappleAttemptStage::AfterEquipment => {}
                Some(GrappleActivity::Escape(e)) if e.grip == id => (),
                _ => {
                    return Err(prerequisite(
                        "Release needs its implemented core consumer context.",
                    ));
                }
            }
        }
        let obsolete = escape(next)
            .ok()
            .filter(|escape| escape.grip == id)
            .cloned();
        let cancelled = if let Some(e) = &obsolete {
            cancel_pending_with_context(next, meta, e.key, execution)?
        } else {
            None
        };
        end_grip(next, meta, id, GrappleEndCause::Released)?;
        if obsolete.is_some() {
            let e = escape_mut(next)?;
            e.outcome = Some(GrappleEscapeOutcome::Obsolete {
                ended_by: meta.clone(),
                cancelled,
            });
            e.stage = TacticalGrappleEscapeStage::Complete;
            e.selected = None;
            super::super::turns::pump_with_context(next, meta, execution)?;
        }
        // Preserve a selected after-equipment choice and its proof/end. An idle
        // release creates no invented resolution or work node.
        Ok(())
    })
}

#[cfg(test)]
pub(in crate::tactical) fn begin_escape(
    state: &mut CampaignState,
    meta: &CommandMeta,
    id: GrappleId,
    choice: GrappleEscapeChoice,
) -> Result<(), RulesError> {
    begin_escape_with_context(state, meta, id, choice, &mut ExecutionContext::ordinary())
}

pub(in crate::tactical) fn begin_escape_with_context(
    state: &mut CampaignState,
    meta: &CommandMeta,
    id: GrappleId,
    choice: GrappleEscapeChoice,
    execution: &mut ExecutionContext<'_>,
) -> Result<(), RulesError> {
    transaction(state, meta, execution, |next, execution| {
        admission::supported_context(next)?;
        if flow(next)?.phase != TacticalPhase::Active || flow(next)?.resolution.is_some() {
            return Err(RulesError::Pending);
        }
        let actor = active(next)?;
        let grip = live_grip(next, id)?.clone();
        if grip.declaration.target != actor {
            return Err(RulesError::Unauthorized);
        }
        super::super::shove::authorize_owner(next, meta, actor)?;
        super::super::falling::require_settled_before_action(next)?;
        let mut escape = TacticalGrappleEscape {
            origin: meta.clone(),
            actor,
            grip: id,
            choice,
            difficulty: grip.declaration.escape_dc,
            key: TacticalRollKey {
                origin: meta.id,
                role: TacticalRollRole::GrappleEscape,
                subject: actor,
                occurrence: 0,
            },
            request: None,
            selected: None,
            stage: TacticalGrappleEscapeStage::Queued,
            outcome: None,
        };
        escape.request = Some(saves::escape_request(next, &escape)?);
        let now = next.clock.now;
        let rules = next.rules.as_mut().ok_or(RulesError::Uninitialized)?;
        crate::tactical_budget::spend_cost(
            rules,
            actor,
            crate::tactical_budget::TacticalCost::Action,
        )?;
        crate::kernel::interrupt_rest(rules, actor, now);
        let mut c = new_context(GrappleActivity::Escape(Box::new(escape)));
        c.proofs.push(grip);
        install_resolution(next, meta, actor, c)?;
        flow_mut(next)?.budget.movement_progress = None;
        flow_mut(next)?.budget.movement_origin = None;
        push_frame(
            next,
            vec![TacticalWorkKind::GrappleEscapeCheck { grip: id }],
        )?;
        super::super::turns::pump_with_context(next, meta, execution)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn already_voluntary_cancelled_request_is_not_cancelled_a_second_time() {
        let mut f = super::super::tests::Fixture::new();
        let id = f.begin(Hand::Left, None);
        let meta = f.meta(Some(f.target));
        choose_save(&mut f.state, &meta, id, GrappleSaveAbility::Strength).unwrap();
        f.state.applied_event_sequence += 1;
        let key = attempt(&f.state).unwrap().save.as_ref().unwrap().key;
        let meta = f.meta(Some(f.target));
        super::super::super::continuations::voluntarily_fail(&mut f.state, &meta).unwrap();
        let before = f.state.rules.as_ref().unwrap().cancelled_roll_ids.clone();
        assert_eq!(
            before.iter().filter(|id| **id == key.request_id()).count(),
            1
        );
        assert_eq!(cancel_pending(&mut f.state, key).unwrap(), None);
        assert_eq!(f.state.rules.as_ref().unwrap().cancelled_roll_ids, before);
        // This exact no-pending helper control does not claim a real LR pause.
    }
}
