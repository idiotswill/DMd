use super::*;

fn causal(state: &CampaignState, meta: &CommandMeta, actor: EntityId) -> Result<(), RulesError> {
    let s = current(state)?;
    validate_equipment_change_origin(state, meta, actor).map_err(|e| invalid(&e))?;
    if meta.session_id != s.origin.session_id
        || meta.expected_event_sequence < s.origin.expected_event_sequence
        || (meta.expected_event_sequence == s.origin.expected_event_sequence && *meta != s.origin)
        || (meta.id == s.origin.id && *meta != s.origin)
    {
        return Err(invalid("Shove decision has foreign chronology"));
    }
    Ok(())
}

pub(in crate::tactical) fn validate_work(
    state: &CampaignState,
    work: &TacticalWorkItem,
) -> Result<EntityId, RulesError> {
    let s = current(state)?;
    if !matches!(
        (&work.kind, s.stage),
        (TacticalWorkKind::BeginShove, TacticalShoveStage::SaveChoice)
            | (TacticalWorkKind::ShoveSave, TacticalShoveStage::Saving)
            | (
                TacticalWorkKind::ChooseShoveOutcome,
                TacticalShoveStage::OutcomeChoice | TacticalShoveStage::PushReview
            )
            | (TacticalWorkKind::FinishShove, TacticalShoveStage::Resolving)
    ) {
        return Err(invalid("Shove work differs from its retained stage"));
    }
    Ok(if work.kind == TacticalWorkKind::ShoveSave {
        s.target
    } else {
        s.actor
    })
}

pub(in crate::tactical) fn validate(state: &CampaignState) -> Result<(), RulesError> {
    let Some(r) = flow(state)?.resolution.as_deref() else {
        return Ok(());
    };
    let Some(s) = r.shove.as_deref() else {
        if r.frames
            .iter()
            .flatten()
            .chain(r.pending.iter().map(|p| &p.work))
            .chain(r.failed_save.iter().map(|p| &p.pending.work))
            .chain(
                r.work_trace
                    .iter()
                    .flat_map(|trace| trace.nodes.iter().map(|node| &node.work)),
            )
            .any(|w| is_work(&w.kind))
        {
            return Err(invalid("Shove work lacks its admitted body action"));
        }
        return Ok(());
    };
    require_execution(state)?;
    validate_ancestry(r, s)?;
    let rules = state.rules.as_ref().ok_or(RulesError::Uninitialized)?;
    if s.origin != r.origin
        || s.actor != r.turn_context().map_err(invalid)?.actor
        || s.actor == s.target
        || s.window.kind != WeaponActionKind::AttackAction
        || flow(state)?.budget.attack_window != Some(s.window)
        || !rules.timing.as_ref().is_some_and(|t| t.action_spent)
        || r.attack.is_some()
        || r.movement.is_some()
        || !r.casts.is_empty()
        || !r.missiles.is_empty()
        || !r.areas.is_empty()
        || s.actor_source != source_pin(state, s.actor)?
        || s.target_source != source_pin(state, s.target)?
        || s.difficulty != difficulty(state, s.actor)?
    {
        return Err(invalid("Shove admission or paid source facts differ"));
    }
    causal(state, &s.origin, s.actor)?;
    authorize_owner(state, &s.origin, s.actor)?;
    opposition(state, s.actor, s.target)?;
    s.actor_from.validate().map_err(|e| invalid(&e))?;
    s.target_from.validate().map_err(|e| invalid(&e))?;
    let e = encounter(state)?;
    let actor = e
        .participant(s.actor)
        .ok_or_else(|| invalid("Shove actor absent"))?;
    let target = e
        .participant(s.target)
        .ok_or_else(|| invalid("Shove target absent"))?;
    let mut original_target = target.clone();
    original_target.position = s.target_from;
    if actor.position != s.actor_from
        || (s.effect.is_none() && target.position != s.target_from)
        || target.size.rank() > actor.size.rank() + 1
        || crate::spatial::participant_distance(actor, &original_target)
            .map_err(|e| invalid(&e.to_string()))?
            > 10
    {
        return Err(invalid("Shove retained reach or size differs"));
    }
    let selected_kind = match s.stage {
        TacticalShoveStage::SaveChoice => Some(TacticalWorkKind::BeginShove),
        TacticalShoveStage::OutcomeChoice | TacticalShoveStage::PushReview => {
            Some(TacticalWorkKind::ChooseShoveOutcome)
        }
        TacticalShoveStage::Saving | TacticalShoveStage::Resolving => None,
        TacticalShoveStage::Queued => return Err(invalid("Unpumped Shove cannot be persisted")),
    };
    if s.selected.as_ref().map(|w| &w.kind) != selected_kind.as_ref()
        || s.push.is_some() != (s.stage == TacticalShoveStage::PushReview)
        || s.save.is_none() != (s.stage == TacticalShoveStage::SaveChoice)
        || (s.stage != TacticalShoveStage::Resolving && s.effect.is_some())
    {
        return Err(invalid("Shove selected work and stage disagree"));
    }
    if let Some(save) = &s.save {
        causal(state, &save.chosen_by, s.target)?;
        authorize_owner(state, &save.chosen_by, s.target)?;
        if save.key.origin != s.origin.id
            || save.key.subject != s.target
            || save.key.role != TacticalRollRole::ShoveSave
            || save.key.occurrence >= r.next_occurrence
            || save.final_success.is_some() != save.resolved_by.is_some()
            || (s.stage == TacticalShoveStage::Saving) != save.final_success.is_none()
            || (matches!(
                s.stage,
                TacticalShoveStage::OutcomeChoice | TacticalShoveStage::PushReview
            ) && save.final_success != Some(false))
        {
            return Err(invalid("Shove save identity or result differs"));
        }
        if let Some(meta) = &save.resolved_by {
            causal(state, meta, s.target)?;
            authorize_owner(state, meta, s.target)?;
            if meta.expected_event_sequence < save.chosen_by.expected_event_sequence
                || (meta.expected_event_sequence == save.chosen_by.expected_event_sequence
                    && *meta != save.chosen_by)
            {
                return Err(invalid(
                    "Shove final result predates its saving ability choice",
                ));
            }
        }
        if save.final_success.is_none()
            && derive_request(state, save.ability, save.key)? != save.request
        {
            return Err(invalid("Pending Shove save differs from current sources"));
        }
        if let Some(request) = &save.request {
            if request.id != save.key.request_id() || request.roller != Some(s.target) {
                return Err(invalid("Shove request has another raw identity"));
            }
            if let Some(roll) = rules
                .rolls
                .iter()
                .find(|roll| roll.request.id == request.id)
            {
                if roll.request != *request
                    || roll.issued_by != save.chosen_by
                    || roll.purpose
                        != (PendingPurpose::TacticalResolution {
                            encounter: e.id,
                            key: save.key,
                        })
                {
                    return Err(invalid("Shove save differs from accepted raw evidence"));
                }
            } else if save.final_success.is_some()
                && !flow(state)?
                    .save_decisions
                    .iter()
                    .any(|d| d.key == save.key)
            {
                return Err(invalid(
                    "Resolved Shove save lacks physical or voluntary evidence",
                ));
            }
        } else if !flow(state)?
            .save_decisions
            .iter()
            .any(|d| d.key == save.key && d.failure == TacticalSaveFailure::Automatic)
        {
            return Err(invalid("Automatic Shove save lacks its source decision"));
        }
        if let Some(final_success) = save.final_success {
            let raw_success = if let Some(roll) = rules
                .rolls
                .iter()
                .find(|roll| roll.request.id == save.key.request_id())
            {
                crate::test_outcome::ability_test_success(
                    &roll.request.resolve(&roll.result)?,
                    s.difficulty,
                    &rules.house_rules,
                )?
            } else {
                false
            };
            let resisted = rules
                .tactical_creatures
                .as_ref()
                .and_then(|c| c.runtime(s.target))
                .is_some_and(|runtime| {
                    runtime
                        .legendary_resistance_rolls
                        .contains(&save.key.request_id())
                });
            if final_success != (raw_success || resisted) {
                return Err(invalid(
                    "Shove final save outcome differs from raw dice and source resistance",
                ));
            }
        }
    }
    if let Some(intent) = &s.push {
        consequence(state, &intent.chosen_by)?;
        if intent.from != s.target_from || target.position != intent.from {
            return Err(invalid("Pending Push changed its source position"));
        }
        geometry::validate_direction(state, intent.destination)?;
    }
    if let Some(effect) = &s.effect {
        if s.save.as_ref().and_then(|save| save.final_success) != Some(false) {
            return Err(invalid("Shove consequence lacks a final failure"));
        }
        match effect {
            TacticalShoveEffect::Prone { chosen_by, applied } => {
                consequence(state, chosen_by)?;
                if *applied
                    == rules.entities[&s.target]
                        .condition_immunities
                        .contains(&Condition::Prone)
                {
                    return Err(invalid("Shove Prone result contradicts source immunity"));
                }
                if *applied && !rules.entities[&s.target].prone {
                    return Err(invalid("Applied Shove Prone is absent"));
                }
                validate_endpoint(state, s.target_from)?;
            }
            TacticalShoveEffect::Push { intent, ruled_by }
            | TacticalShoveEffect::BlockedPush { intent, ruled_by } => {
                consequence(state, &intent.chosen_by)?;
                causal(state, ruled_by, s.target)?;
                privileged(ruled_by)?;
                if intent.from != s.target_from
                    || ruled_by.expected_event_sequence <= intent.chosen_by.expected_event_sequence
                {
                    return Err(invalid("Push ruling lacks its earlier source intent"));
                }
                if intent.convention != ShoveGeometryConvention::GridFiveFootAwayV1 {
                    return Err(invalid("Push has another geometry convention"));
                }
                // Reconstruct only this body's earlier position. This never
                // mutates the campaign or grants permission to a live command.
                let mut before = state.clone();
                before
                    .encounter
                    .as_mut()
                    .ok_or_else(|| invalid("Shove map absent"))?
                    .participants
                    .iter_mut()
                    .find(|p| p.entity_id == s.target)
                    .ok_or_else(|| invalid("Shove body absent"))?
                    .position = intent.from;
                let route = geometry::path(&before, intent)?;
                if matches!(effect, TacticalShoveEffect::Push { .. }) {
                    let route = route.map_err(|e| invalid(&e.to_string()))?;
                    if route.destination != intent.destination
                        || route.total_cost != 0
                        || route.segments.len() != 1
                        || !route.segments[0].opportunities.is_empty()
                        || route.segments[0].from != intent.from
                        || route.segments[0].to != intent.destination
                        || route.progress != TacticalMovementProgress::default()
                    {
                        return Err(invalid(
                            "Retained Push differs from exact accepted geometry",
                        ));
                    }
                    validate_endpoint(state, intent.destination)?;
                } else {
                    if !matches!(route, Err(crate::spatial::SpatialError::Illegal(_)))
                        || !crate::spatial::forced_step_physically_blocked(
                            encounter(&before)?,
                            s.target,
                            intent.destination,
                        )
                        .map_err(|e| invalid(&e.to_string()))?
                    {
                        return Err(invalid("Blocked Push lacks positive physical obstruction"));
                    }
                    validate_endpoint(state, intent.from)?;
                }
            }
        }
    } else if s.stage == TacticalShoveStage::Resolving
        && s.save.as_ref().and_then(|save| save.final_success) != Some(true)
    {
        return Err(invalid(
            "Failed Shove completed without a chosen consequence",
        ));
    }
    Ok(())
}

fn consequence(state: &CampaignState, meta: &CommandMeta) -> Result<(), RulesError> {
    let s = current(state)?;
    causal(state, meta, s.actor)?;
    authorize_owner(state, meta, s.actor)?;
    let resolved = s
        .save
        .as_ref()
        .and_then(|save| save.resolved_by.as_ref())
        .ok_or_else(|| invalid("Shove consequence lacks its final save command"))?;
    if meta.expected_event_sequence <= resolved.expected_event_sequence || meta.id == resolved.id {
        return Err(invalid("Shove consequence predates its final failed save"));
    }
    Ok(())
}

fn validate_ancestry(r: &TacticalResolution, s: &TacticalShove) -> Result<(), RulesError> {
    let trace = r
        .work_trace
        .as_ref()
        .ok_or_else(|| invalid("Shove has no causal work trace"))?;
    let nodes = |kind: TacticalWorkKind| {
        trace
            .nodes
            .iter()
            .filter(move |node| node.work.kind == kind)
    };
    let begin = nodes(TacticalWorkKind::BeginShove).collect::<Vec<_>>();
    let saves = nodes(TacticalWorkKind::ShoveSave).collect::<Vec<_>>();
    let outcomes = nodes(TacticalWorkKind::ChooseShoveOutcome).collect::<Vec<_>>();
    let finishes = nodes(TacticalWorkKind::FinishShove).collect::<Vec<_>>();
    let failed = s.save.as_ref().and_then(|save| save.final_success) == Some(false);
    if (s.stage == TacticalShoveStage::Resolving && s.save.is_none())
        || begin.len() != 1
        || begin[0].parent.is_some()
        || begin[0].work.occurrence != 0
        || saves.len() != usize::from(s.save.is_some())
        || outcomes.len() != usize::from(failed)
        || finishes.len() != usize::from(s.stage == TacticalShoveStage::Resolving)
    {
        return Err(invalid(
            "Shove causal nodes are absent, duplicated or premature",
        ));
    }
    if let Some(save) = &s.save
        && (saves[0].parent != Some(begin[0].work.occurrence)
            || saves[0].work.occurrence != save.key.occurrence)
    {
        return Err(invalid("Shove save has another selected parent"));
    }
    if failed && outcomes[0].parent != Some(saves[0].work.occurrence) {
        return Err(invalid("Shove outcome is not a child of its failed save"));
    }
    if let Some(finish) = finishes.first() {
        let parent = if failed {
            outcomes[0].work.occurrence
        } else {
            saves[0].work.occurrence
        };
        if finish.parent != Some(parent) {
            return Err(invalid("Shove completion has another cause"));
        }
    }
    Ok(())
}

fn validate_endpoint(state: &CampaignState, endpoint: SpatialPoint) -> Result<(), RulesError> {
    let s = current(state)?;
    let r = resolution(state)?;
    let child = r.falls.iter().find(
        |fall| matches!(&fall.cause, TacticalFallCause::Shove { shove, .. } if *shove == s.origin),
    );
    let expected = match child {
        Some(fall) if fall.path.from != endpoint => {
            return Err(invalid("Shove landing starts at a different endpoint"));
        }
        Some(fall) if matches!(fall.stage, TacticalFallStage::Complete { .. }) => fall.path.to,
        _ => endpoint,
    };
    if encounter(state)?
        .participant(s.target)
        .is_none_or(|target| target.position != expected)
    {
        return Err(invalid(
            "Shove body is not at its committed endpoint or child landing",
        ));
    }
    Ok(())
}

pub(in crate::tactical) fn validate_fall(
    state: &CampaignState,
    fall: &TacticalFall,
    origin: &CommandMeta,
    consequence: &CommandMeta,
    work: TacticalWorkKey,
) -> Result<(), RulesError> {
    let s = current(state)?;
    let r = resolution(state)?;
    let actual = match s.effect.as_ref() {
        Some(TacticalShoveEffect::Prone {
            chosen_by,
            applied: true,
        }) => chosen_by,
        Some(TacticalShoveEffect::Push { ruled_by, .. }) => ruled_by,
        _ => return Err(invalid("Shove fall lacks a displacement or applied Prone")),
    };
    if *origin != s.origin
        || *consequence != *actual
        || fall.origin != *actual
        || fall.actor != s.target
        || work.resolution != s.origin.id
        || r.work_trace.as_ref().is_none_or(|trace| {
            !trace.nodes.iter().any(|node| {
                node.work.occurrence == work.occurrence
                    && node.work.kind == TacticalWorkKind::ChooseShoveOutcome
            })
        })
    {
        return Err(invalid("Shove fall has foreign causal work"));
    }
    let index = r
        .falls
        .iter()
        .position(|record| std::ptr::eq(record, fall))
        .ok_or_else(|| invalid("Unretained Shove fall"))?;
    if r.work_trace.as_ref().is_none_or(|trace| {
        !trace.nodes.iter().any(|node| {
            node.work.kind == TacticalWorkKind::BeginFall { fall: index as u16 }
                && node.parent == Some(work.occurrence)
        })
    }) {
        return Err(invalid(
            "Shove fall is not a child of its accepted consequence",
        ));
    }
    let mut before_landing = state.clone();
    before_landing
        .encounter
        .as_mut()
        .ok_or_else(|| invalid("Shove map absent"))?
        .participants
        .iter_mut()
        .find(|p| p.entity_id == s.target)
        .ok_or_else(|| invalid("Shove body absent"))?
        .position = fall.path.from;
    if geometry::fall_path(&before_landing)?.as_ref() != Some(&fall.path) {
        return Err(invalid(
            "Shove fall differs from its actual support or flight loss",
        ));
    }
    Ok(())
}
