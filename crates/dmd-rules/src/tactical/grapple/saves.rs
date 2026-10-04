use super::*;

pub(super) fn expected_save(
    state: &CampaignState,
    d: &TacticalGrappleDeclaration,
    ability: GrappleSaveAbility,
    key: TacticalRollKey,
) -> Result<Option<RollRequest>, RulesError> {
    let rules = state.rules.as_ref().ok_or(RulesError::Uninitialized)?;
    let circumstance = Circumstances {
        disadvantage: super::super::attacks::body_untrained_armor(state, d.target)?,
        ..Circumstances::default()
    };
    let disposition = crate::tactical_conditions::save_conditions(
        rules,
        d.target,
        ability.ability(),
        circumstance,
        dodge_context(state, d.target)?,
    )?;
    let crate::tactical_conditions::TestDisposition::Roll(mode) = disposition else {
        return Ok(None);
    };
    let modifier = super::super::continuations::save_modifier(state, d.target, ability.ability())?
        + if ability == GrappleSaveAbility::Dexterity {
            super::super::shove::cover_bonus(state, d.grappler, d.target)?
        } else {
            0
        };
    Ok(Some(RollRequest {
        id: key.request_id(),
        roller: Some(d.target),
        dice: vec![DieSpec {
            count: 1,
            sides: 20,
        }],
        modifier,
        mode,
        visibility: super::super::continuations::visibility(state, d.target),
        reason: "Grapple saving throw".into(),
    }))
}

#[cfg(test)]
pub(in crate::tactical) fn choose_save(
    state: &mut CampaignState,
    meta: &CommandMeta,
    grip: GrappleId,
    ability: GrappleSaveAbility,
) -> Result<(), RulesError> {
    choose_save_with_context(
        state,
        meta,
        grip,
        ability,
        &mut ExecutionContext::ordinary(),
    )
}

pub(in crate::tactical) fn choose_save_with_context(
    state: &mut CampaignState,
    meta: &CommandMeta,
    grip: GrappleId,
    ability: GrappleSaveAbility,
    execution: &mut ExecutionContext<'_>,
) -> Result<(), RulesError> {
    transaction(state, meta, execution, |next, execution| {
        validate(next)?;
        let a = attempt(next)?;
        if a.declaration.id != grip || a.stage != TacticalGrappleAttemptStage::SaveChoice {
            return Err(RulesError::Pending);
        }
        super::super::shove::authorize_owner(next, meta, a.declaration.target)?;
        let selected = a
            .selected
            .clone()
            .ok_or_else(|| invalid("Grapple save choice lacks work"))?;
        let key = TacticalRollKey {
            origin: a.declaration.origin.id,
            role: TacticalRollRole::GrappleSave,
            subject: a.declaration.target,
            occurrence: resolution(next)?.next_occurrence,
        };
        let request = expected_save(next, &a.declaration, ability, key)?;
        let previous = super::super::work_trace::enter(next, &selected)?;
        let result = (|| {
            let a = attempt_mut(next)?;
            a.selected = None;
            a.stage = TacticalGrappleAttemptStage::Saving;
            a.save = Some(TacticalGrappleSave {
                ability,
                chosen_by: meta.clone(),
                key,
                request,
                proof: None,
            });
            push_frame(next, vec![TacticalWorkKind::GrappleSave { grip }])
        })();
        let reset = super::super::work_trace::leave(next, previous);
        result?;
        reset?;
        super::super::turns::pump_with_context(next, meta, execution)
    })
}

pub(super) fn escape_request(
    state: &CampaignState,
    e: &TacticalGrappleEscape,
) -> Result<RollRequest, RulesError> {
    let rules = state.rules.as_ref().ok_or(RulesError::Uninitialized)?;
    let entity = rules
        .entities
        .get(&e.actor)
        .ok_or_else(|| invalid("Escape actor absent"))?;
    if entity.death.dead || !crate::tactical_conditions::can_act(rules, e.actor)? {
        return Err(prerequisite("The target cannot attempt Escape."));
    }
    let (ability, skill) = e.choice.test();
    let kind = TestKind::Check {
        ability,
        skill: Some(skill),
    };
    let modifier = match rules
        .tactical_creatures
        .as_ref()
        .and_then(|c| c.profile(e.actor))
    {
        Some(profile) => crate::tactical_creatures::creature_test_modifier(profile, entity, &kind)
            .map_err(|e| invalid(&e.to_string()))?,
        None => crate::test_modifier(entity, &kind),
    };
    let encounter = encounter(state)?;
    let mut fear_visible = false;
    for effect in crate::tactical_effect_adapter::condition_effects(rules) {
        if effect.target == e.actor
            && effect.condition == Some(Condition::Frightened)
            && encounter.participant(effect.source).is_some()
        {
            fear_visible |= crate::spatial::perceive(encounter, state, e.actor, effect.source)
                .map_err(|e| invalid(&e.to_string()))?
                .sees;
        }
    }
    let circumstance = Circumstances {
        disadvantage: super::super::attacks::body_untrained_armor(state, e.actor)?,
        ..Circumstances::default()
    };
    let crate::tactical_conditions::TestDisposition::Roll(mode) =
        crate::tactical_conditions::check_conditions(
            rules,
            e.actor,
            false,
            false,
            fear_visible,
            circumstance,
        )?
    else {
        return Err(prerequisite("The target cannot perform this Escape check."));
    };
    Ok(RollRequest {
        id: e.key.request_id(),
        roller: Some(e.actor),
        dice: vec![DieSpec {
            count: 1,
            sides: 20,
        }],
        modifier,
        mode,
        visibility: super::super::continuations::visibility(state, e.actor),
        reason: match e.choice {
            GrappleEscapeChoice::Athletics => "Strength (Athletics) Escape",
            GrappleEscapeChoice::Acrobatics => "Dexterity (Acrobatics) Escape",
        }
        .into(),
    })
}

pub(in crate::tactical) fn request(
    state: &CampaignState,
    work: &TacticalWorkItem,
    expected: TacticalRollKey,
) -> Result<Option<RollRequest>, RulesError> {
    if key(state, work)? != expected {
        return Err(invalid("Grapple raw key differs"));
    }
    validation::validate_request_source(state, work)?;
    match work.kind {
        TacticalWorkKind::GrappleSave { .. } => {
            let save = attempt(state)?
                .save
                .as_ref()
                .ok_or_else(|| invalid("Grapple save choice absent"))?;
            if save.key != expected
                || expected_save(state, &attempt(state)?.declaration, save.ability, expected)?
                    != save.request
            {
                return Err(invalid("Grapple request differs from actual source"));
            }
            Ok(save.request.clone())
        }
        TacticalWorkKind::GrappleEscapeCheck { .. } => {
            let e = escape(state)?;
            let request = escape_request(state, e)?;
            if e.key != expected || e.request.as_ref() != Some(&request) {
                return Err(invalid("Escape request differs from actual source"));
            }
            Ok(Some(request))
        }
        _ => Err(invalid("Grapple material choice has no raw request")),
    }
}

pub(in crate::tactical) fn save_failed(
    state: &CampaignState,
    pending: &TacticalPendingWork,
    result: Option<&RollResult>,
) -> Result<bool, RulesError> {
    let Some(result) = result else {
        return Ok(true);
    };
    let request = request(state, &pending.work, pending.key)?
        .ok_or_else(|| invalid("Automatic Grapple save has no raw dice"))?;
    Ok(!crate::test_outcome::ability_test_success(
        &request.resolve(result)?,
        attempt(state)?.declaration.escape_dc,
        &state
            .rules
            .as_ref()
            .ok_or(RulesError::Uninitialized)?
            .house_rules,
    )?)
}

pub(super) fn evidence(
    state: &CampaignState,
    save: &TacticalGrappleSave,
) -> Result<GrappleSaveEvidence, RulesError> {
    let rules = state.rules.as_ref().ok_or(RulesError::Uninitialized)?;
    let id = save.key.request_id();
    let rolls = rules
        .rolls
        .iter()
        .filter(|r| r.request.id == id)
        .collect::<Vec<_>>();
    let decisions = flow(state)?
        .save_decisions
        .iter()
        .filter(|d| d.key == save.key)
        .collect::<Vec<_>>();
    if rolls.len() > 1 || decisions.len() > 1 || (!rolls.is_empty() && !decisions.is_empty()) {
        return Err(invalid("Grapple evidence identity is not unique"));
    }
    let cancelled = rules
        .cancelled_roll_ids
        .iter()
        .filter(|value| **value == id)
        .count();
    if let Some(raw) = rolls.first() {
        if cancelled != 0 {
            return Err(invalid("Accepted Grapple dice were canceled"));
        }
        if save.request.as_ref() != Some(&raw.request)
            || raw.issued_by != save.chosen_by
            || raw.purpose
                != (PendingPurpose::TacticalResolution {
                    encounter: encounter(state)?.id,
                    key: save.key,
                })
        {
            return Err(invalid("Grapple physical evidence differs"));
        }
        return Ok(GrappleSaveEvidence::Physical {
            key: save.key,
            accepted_by: raw.accepted_by.clone(),
        });
    }
    let decision = decisions
        .first()
        .copied()
        .ok_or_else(|| invalid("Grapple result evidence absent"))?;
    if decision.issued_by != save.chosen_by
        || (decision.failure == TacticalSaveFailure::Automatic) != save.request.is_none()
        || cancelled != usize::from(decision.failure == TacticalSaveFailure::Voluntary)
    {
        return Err(invalid("Grapple no-die evidence differs"));
    }
    Ok(GrappleSaveEvidence::Decision(decision.clone()))
}

pub(super) fn complete_attempt(state: &mut CampaignState) -> Result<(), RulesError> {
    let a = attempt_mut(state)?;
    a.selected = None;
    if a.equipment.before_change.is_some() {
        a.stage = TacticalGrappleAttemptStage::Complete;
    } else {
        a.stage = TacticalGrappleAttemptStage::AfterEquipment;
        let grip = a.declaration.id;
        push_frame(
            state,
            vec![TacticalWorkKind::GrappleAfterEquipment { grip }],
        )?;
    }
    Ok(())
}

pub(in crate::tactical) fn finish(
    state: &mut CampaignState,
    meta: &CommandMeta,
    pending: &TacticalPendingWork,
    result: Option<&RollResult>,
    forced_success: bool,
) -> Result<(), RulesError> {
    match pending.work.kind {
        TacticalWorkKind::GrappleSave { .. } => {
            admission::validate_attempt_admission(state)?;
            let failed = save_failed(state, pending, result)?;
            let old = attempt(state)?.clone();
            let save = old
                .save
                .as_ref()
                .ok_or_else(|| invalid("Grapple save absent"))?;
            let evidence = evidence(state, save)?;
            let legendary = resolution(state)?
                .failed_save
                .as_ref()
                .map(|f| {
                    if f.pending != *pending {
                        return Err(invalid("Grapple LR work differs"));
                    }
                    super::super::shove::authorize_owner(state, meta, pending.key.subject)?;
                    Ok(GrappleLegendaryDecision {
                        chosen_by: meta.clone(),
                        use_resistance: forced_success,
                    })
                })
                .transpose()?;
            let succeeded = forced_success || !failed;
            let immune = state
                .rules
                .as_ref()
                .and_then(|r| r.entities.get(&old.declaration.target))
                .ok_or_else(|| invalid("Grapple target mechanics absent"))?
                .condition_immunities
                .contains(&Condition::Grappled);
            if !succeeded && !immune {
                admission::deferred_flight_loss(state, old.declaration.target)?;
            }
            let proof = GrappleSaveProof {
                evidence,
                legendary,
                final_success: succeeded,
                finalized_by: meta.clone(),
            };
            attempt_mut(state)?.save.as_mut().unwrap().proof = Some(proof);
            resolution_mut(state)?.failed_save = None;
            let outcome = if succeeded {
                GrappleAttemptOutcome::Resisted {
                    resolved_by: meta.clone(),
                }
            } else if immune {
                GrappleAttemptOutcome::Immune {
                    resolved_by: meta.clone(),
                }
            } else {
                let grip = TacticalGrip {
                    declaration: old.declaration,
                    established_by: meta.clone(),
                    work: work_key(state, &pending.work)?,
                    save: attempt(state)?.save.clone().unwrap(),
                };
                let id = grip.declaration.id;
                let rules = state.rules.as_mut().ok_or(RulesError::Uninitialized)?;
                let live = rules
                    .tactical_grapples
                    .get_or_insert_with(|| TacticalGrapples {
                        schema_version: TACTICAL_GRAPPLES_SCHEMA_VERSION,
                        active: vec![],
                    });
                if live.active.iter().any(|g| {
                    g.declaration.id == id
                        || (g.declaration.grappler == grip.declaration.grappler
                            && g.declaration.hand == grip.declaration.hand)
                }) {
                    return Err(invalid("Grapple live hand collides"));
                }
                live.active.push(grip.clone());
                live.active.sort_by_key(|g| g.declaration.id.0);
                context_mut(state)?.proofs.push(grip);
                GrappleAttemptOutcome::Established { grip: id }
            };
            attempt_mut(state)?.outcome = Some(outcome);
            if !succeeded && !immune {
                // The live relation and its final outcome already exist. Their
                // derived Speed zero ends the actual Dodge permanently before
                // the next owned equipment pause or creating-work retirement.
                super::super::turns::refresh_dodges(state)?;
            }
            complete_attempt(state)
        }
        TacticalWorkKind::GrappleEscapeCheck { .. } => {
            if forced_success {
                return Err(invalid("A saving-throw override cannot change Escape"));
            }
            let e = escape(state)?.clone();
            let request = e
                .request
                .as_ref()
                .ok_or_else(|| invalid("Escape request absent"))?;
            let result = result.ok_or_else(|| invalid("Escape requires actual dice"))?;
            let succeeded = crate::test_outcome::ability_test_success(
                &request.resolve(result)?,
                e.difficulty,
                &state
                    .rules
                    .as_ref()
                    .ok_or(RulesError::Uninitialized)?
                    .house_rules,
            )?;
            if succeeded {
                lifecycle::end_grip(
                    state,
                    meta,
                    e.grip,
                    GrappleEndCause::Escaped {
                        roll: e.key,
                        work: work_key(state, &pending.work)?,
                    },
                )?;
            }
            let e = escape_mut(state)?;
            e.outcome = Some(GrappleEscapeOutcome::Checked {
                resolved_by: meta.clone(),
                succeeded,
            });
            e.stage = TacticalGrappleEscapeStage::Complete;
            Ok(())
        }
        _ => Err(invalid("Grapple choice cannot finish raw work")),
    }
}
