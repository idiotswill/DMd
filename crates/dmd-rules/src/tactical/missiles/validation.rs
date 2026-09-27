//! Authenticate committed source occurrences, decisions and their single live
//! frame partition. Semantic history replay additionally proves each transition.
use super::*;

fn cast_occurrences(r: &TacticalResolution) -> Result<(), RulesError> {
    // Casting reserves an ordinal without creating a work node. A completed
    // child therefore continues to reserve that identity after leaving casts.
    let mut reserved = HashSet::new();
    for cast in r
        .casts
        .iter()
        .chain(
            r.missiles
                .iter()
                .filter_map(|missile| missile.completed_cast.as_deref()),
        )
        .chain(r.missiles.iter().flat_map(|missile| {
            missile
                .respondents
                .iter()
                .filter_map(|respondent| respondent.completed_shield.as_deref())
        }))
        .chain(
            r.hit_review
                .iter()
                .filter_map(|hit| hit.completed_shield.as_deref()),
        )
    {
        let occurrence = cast.cast.plan.occurrence;
        if occurrence >= r.next_occurrence || !reserved.insert(occurrence) {
            return Err(invalid(
                "reused or future casting occurrence across live and retired sources",
            ));
        }
    }
    if r.work_trace.as_ref().is_some_and(|trace| {
        trace
            .nodes
            .iter()
            .any(|node| reserved.contains(&node.work.occurrence))
    }) {
        return Err(invalid(
            "casting occurrence collides with retained work identity",
        ));
    }
    Ok(())
}

fn decision(
    state: &CampaignState,
    meta: &CommandMeta,
    actor: EntityId,
    after: &CommandMeta,
) -> Result<(), RulesError> {
    validate_equipment_change_origin(state, meta, actor).map_err(|error| invalid(&error))?;
    if meta.expected_event_sequence <= after.expected_event_sequence
        || meta.id == after.id
        || meta.session_id != after.session_id
    {
        return Err(invalid("missile decision precedes its source trigger"));
    }
    Ok(())
}

fn source<'a>(
    r: &'a TacticalResolution,
    missile: &'a TacticalMissile,
) -> Result<&'a TacticalCasting, RulesError> {
    let live = r
        .casts
        .iter()
        .find(|cast| cast.cast.plan.occurrence == missile.cast);
    match (live, missile.completed_cast.as_deref()) {
        (Some(cast), None) | (None, Some(cast)) => Ok(cast),
        _ => Err(invalid("missile has absent or duplicate casting authority")),
    }
}

fn work_parent(r: &TacticalResolution, work: &TacticalWorkItem) -> Result<Option<u16>, RulesError> {
    r.work_trace
        .as_ref()
        .and_then(|trace| trace.nodes.iter().find(|node| node.work == *work))
        .map(|node| node.parent)
        .ok_or_else(|| invalid("missile work lacks its exact ancestry"))
}

pub(in crate::tactical) fn validate_work(
    state: &CampaignState,
    work: &TacticalWorkItem,
) -> Result<EntityId, RulesError> {
    let r = resolution(state)?;
    if !TacticalExecutionVersion::from_flow_version(flow(state)?.version)
        .is_some_and(TacticalExecutionVersion::supports_missile_shield)
    {
        return Err(invalid("missile work attached to an earlier executor"));
    }
    let missile_id = match work.kind {
        TacticalWorkKind::BeginMissile { cast } => {
            let casting = super::super::casting::current_cast(state, cast)?;
            if !is_program(casting) {
                return Err(invalid("missile trigger has a different program"));
            }
            return Ok(casting.cast.plan.choice.actor);
        }
        TacticalWorkKind::ResumeMissile { missile }
        | TacticalWorkKind::BeginMissileImpacts { missile }
        | TacticalWorkKind::ApplyMissileImpact { missile, .. }
        | TacticalWorkKind::CommitMissileShield { missile, .. } => missile,
        _ => return Err(invalid("not missile work")),
    };
    let missile = current(state, missile_id)?;
    let parent = work_parent(r, work)?;
    match work.kind {
        TacticalWorkKind::ResumeMissile { .. } => {
            if parent != Some(missile_id)
                || !matches!(
                    missile.stage,
                    TacticalMissileStage::Collecting
                        | TacticalMissileStage::Selected { .. }
                        | TacticalMissileStage::Casting { .. }
                )
            {
                return Err(invalid(
                    "missile response cursor has another stage or parent",
                ));
            }
        }
        TacticalWorkKind::CommitMissileShield {
            respondent, cast, ..
        } => {
            if parent != Some(missile_id)
                || missile.stage != (TacticalMissileStage::Casting { respondent, cast })
            {
                return Err(invalid("missile Shield work has another selected owner"));
            }
        }
        TacticalWorkKind::BeginMissileImpacts { .. } => {
            if missile.stage != TacticalMissileStage::Amounts
                || parent
                    .and_then(|id| {
                        r.work_trace
                            .as_ref()?
                            .nodes
                            .iter()
                            .find(|node| node.work.occurrence == id)
                    })
                    .is_none_or(|node| {
                        node.work.kind
                            != (TacticalWorkKind::ResumeMissile {
                                missile: missile_id,
                            })
                    })
            {
                return Err(invalid(
                    "missile impact barrier has another stage or parent",
                ));
            }
        }
        TacticalWorkKind::ApplyMissileImpact { at, .. } => {
            let dart = missile
                .darts
                .get(usize::from(at.target))
                .filter(|dart| dart.at == at)
                .ok_or_else(|| invalid("missile impact names an unbound dart"))?;
            if missile.stage != TacticalMissileStage::Impacts
                || dart.impact_occurrence != Some(work.occurrence)
                || dart.completed_by.is_some()
                || parent
                    .and_then(|id| {
                        r.work_trace
                            .as_ref()?
                            .nodes
                            .iter()
                            .find(|node| node.work.occurrence == id)
                    })
                    .is_none_or(|node| {
                        node.work.kind
                            != (TacticalWorkKind::BeginMissileImpacts {
                                missile: missile_id,
                            })
                    })
            {
                return Err(invalid(
                    "missile impact differs from its committed occurrence",
                ));
            }
            return Ok(dart.target);
        }
        _ => unreachable!(),
    }
    Ok(source(r, missile)?.cast.plan.choice.actor)
}

pub(in crate::tactical) fn validate_cast_partition(
    state: &CampaignState,
    cast: &TacticalCasting,
) -> Result<(), RulesError> {
    let r = resolution(state)?;
    let missile = r
        .missiles
        .iter()
        .find(|m| m.cast == cast.cast.plan.occurrence)
        .ok_or_else(|| invalid("missile partition has no committed record"))?;
    let live = r
        .frames
        .iter()
        .flatten()
        .chain(r.pending.iter().map(|pending| &pending.work))
        .chain(r.failed_save.iter().map(|failed| &failed.pending.work))
        .collect::<Vec<_>>();
    if live
        .iter()
        .filter(|work| work.kind == TacticalWorkKind::FinishSpell { cast: missile.cast })
        .count()
        != 1
    {
        return Err(invalid("missile lacks its single cast completion"));
    }
    let resumes = live
        .iter()
        .filter(|work| {
            work.kind
                == TacticalWorkKind::ResumeMissile {
                    missile: missile.work.occurrence,
                }
        })
        .count();
    let barriers = live
        .iter()
        .filter(|work| {
            work.kind
                == TacticalWorkKind::BeginMissileImpacts {
                    missile: missile.work.occurrence,
                }
        })
        .count();
    let response_stage = matches!(
        missile.stage,
        TacticalMissileStage::Collecting
            | TacticalMissileStage::Selected { .. }
            | TacticalMissileStage::Casting { .. }
    );
    if resumes != usize::from(response_stage)
        || barriers != usize::from(missile.stage == TacticalMissileStage::Amounts)
    {
        return Err(invalid(
            "missile response or amount barrier is omitted or duplicated",
        ));
    }
    for dart in &missile.darts {
        let amounts = live
            .iter()
            .filter(|work| {
                work.kind
                    == TacticalWorkKind::SpellProgram {
                        cast: missile.cast,
                        at: dart.at,
                    }
            })
            .count();
        let impacts = live
            .iter()
            .filter(|work| {
                work.kind
                    == TacticalWorkKind::ApplyMissileImpact {
                        missile: missile.work.occurrence,
                        at: dart.at,
                    }
            })
            .count();
        if amounts
            != usize::from(missile.stage == TacticalMissileStage::Amounts && dart.amount.is_none())
            || impacts
                != usize::from(
                    missile.stage == TacticalMissileStage::Impacts && dart.completed_by.is_none(),
                )
            || cast.completed.contains(&dart.at) != dart.completed_by.is_some()
        {
            return Err(invalid(
                "missile dart has a missing or duplicate amount/impact partition",
            ));
        }
    }
    if live.iter().any(|work| {
        matches!(work.kind, TacticalWorkKind::SpellProgram { cast: id, at }
        if id == missile.cast && !missile.darts.iter().any(|dart| dart.at == at))
    }) {
        return Err(invalid("missile queues an unbound amount"));
    }
    Ok(())
}

fn validate_shield(
    state: &CampaignState,
    missile: &TacticalMissile,
    index: usize,
    casting: &TacticalCasting,
    after: &CommandMeta,
) -> Result<(), RulesError> {
    let r = resolution(state)?;
    let actor = missile.respondents[index].response.actor;
    validate_retained_spell(casting)?;
    super::super::casting::validate_actor_source(state, casting)?;
    decision(state, &casting.cast.plan.origin, actor, after)?;
    owner(state, &casting.cast.plan.origin, actor)?;
    if casting.cast.plan.choice.spell_id != "shield"
        || casting.cast.plan.choice.actor != actor
        || casting.cast.plan.cost != SpellCastingCost::Reaction
        || casting.cast.plan.occurrence >= r.next_occurrence
        || casting.cast.phase != SpellCastPhase::Committed
        || casting.cast.last_operation != casting.cast.plan.origin
        || casting.cast.started_on_turn != r.turn_number
        || casting.cast.started_at != state.clock.now
        || casting.targets.len() != 1
        || casting.targets[0].actor != actor
        || casting.completed.as_slice() != [SpellProgramOccurrence { node: 0, target: 0 }]
        || r.casts
            .iter()
            .any(|cast| cast.cast.plan.occurrence == casting.cast.plan.occurrence)
        || !state
            .rules
            .as_ref()
            .and_then(|rules| rules.timing.as_ref())
            .is_some_and(|timing| timing.reactions_spent.contains(&actor))
        || casting
            .creature_activation
            .as_ref()
            .is_some_and(|activation| {
                activation.activation != SpellEnclosingActivation::Reaction
                    || activation.origin != casting.cast.plan.origin
                    || activation.attack_action
            })
    {
        return Err(invalid(
            "missile Shield differs from its selected source casting",
        ));
    }
    let commits = r
        .work_trace
        .as_ref()
        .ok_or_else(|| invalid("missile Shield lacks ancestry"))?
        .nodes
        .iter()
        .filter(|node| {
            node.work.kind
                == TacticalWorkKind::CommitMissileShield {
                    missile: missile.work.occurrence,
                    respondent: index as u16,
                    cast: casting.cast.plan.occurrence,
                }
        })
        .collect::<Vec<_>>();
    if commits.len() != 1
        || commits[0].parent != Some(missile.work.occurrence)
        || casting.cast.plan.occurrence <= missile.work.occurrence
        || casting.cast.plan.occurrence >= commits[0].work.occurrence
    {
        return Err(invalid(
            "missile Shield lacks its independent selected child",
        ));
    }
    let mut incomplete = casting.clone();
    incomplete.completed.clear();
    let mut expected = spell_defense_effect(
        state,
        &incomplete,
        SpellProgramOccurrence { node: 0, target: 0 },
    )?
    .ok_or_else(|| invalid("missile Shield has no canonical defense"))?;
    let actual = effects(state)?
        .effects
        .iter()
        .find(|effect| effect.id == expected.id)
        .ok_or_else(|| invalid("missile Shield lost its installed defense"))?;
    let stamp = actual
        .established_at
        .as_ref()
        .ok_or_else(|| invalid("missile Shield has no installation cause"))?;
    if stamp.command != casting.cast.last_operation {
        return Err(invalid("missile Shield has a different installation cause"));
    }
    expected.established_at = Some(stamp.clone());
    if *actual != expected {
        return Err(invalid("missile Shield defense differs from its source"));
    }
    Ok(())
}

pub(in crate::tactical) fn validate(state: &CampaignState) -> Result<(), RulesError> {
    let Some(r) = flow(state)?.resolution.as_deref() else {
        return Ok(());
    };
    let enabled = TacticalExecutionVersion::from_flow_version(flow(state)?.version)
        .is_some_and(TacticalExecutionVersion::supports_missile_shield);
    if !enabled && !r.missiles.is_empty() {
        return Err(invalid("earlier executor retains missile authority"));
    }
    if enabled {
        cast_occurrences(r)?;
    }
    if r.missiles.len() > MAX_TACTICAL_CASTS {
        return Err(invalid("too many retained missiles"));
    }
    let rules = state.rules.as_ref().ok_or(RulesError::Uninitialized)?;
    let mut casts = HashSet::new();
    let mut windows = HashSet::new();
    let mut decisions = HashSet::new();
    for missile in &r.missiles {
        let cast = source(r, missile)?;
        validate_retained_spell(cast)?;
        super::super::casting::validate_actor_source(state, cast)?;
        if !is_program(cast)
            || !casts.insert(missile.cast)
            || !windows.insert(missile.work.occurrence)
            || cast.cast.plan.occurrence != missile.cast
            || cast.cast.plan.choice.actor != r.turn_actor
            || missile.work.kind != (TacticalWorkKind::BeginMissile { cast: missile.cast })
            || missile.cause != cast.cast.last_operation
            || missile.cause != cast.cast.plan.origin
            || missile.work.occurrence >= r.next_occurrence
            || missile.darts.len() != cast.targets.len()
            || missile.darts.is_empty()
            || missile.completed_cast.is_some()
                != (missile.stage == TacticalMissileStage::Completed)
        {
            return Err(invalid(
                "missile trigger differs from its original committed cast",
            ));
        }
        work_parent(r, &missile.work)?;
        let mut distinct = HashSet::new();
        let targets = cast
            .targets
            .iter()
            .filter(|target| distinct.insert(target.actor))
            .map(|target| target.actor)
            .collect::<Vec<_>>();
        if missile
            .respondents
            .iter()
            .map(|respondent| respondent.response.actor)
            .collect::<Vec<_>>()
            != targets
        {
            return Err(invalid(
                "missile respondents differ from distinct committed targets",
            ));
        }
        let mut retain =
            |meta: &CommandMeta, actor, after: &CommandMeta| -> Result<(), RulesError> {
                decision(state, meta, actor, after)?;
                if !decisions.insert(meta.id) {
                    return Err(invalid(
                        "one command impersonates multiple missile decisions",
                    ));
                }
                Ok(())
            };
        if let Some(delegated) = &missile.delegated_by {
            retain(delegated, r.turn_actor, &missile.cause)?;
            owner(state, delegated, r.turn_actor)?;
        }
        if let Some(order) = &missile.order {
            retain(
                &order.origin,
                r.turn_actor,
                missile.delegated_by.as_ref().unwrap_or(&missile.cause),
            )?;
            if missile.delegated_by.is_some() {
                privileged(&order.origin)?;
            } else {
                owner(state, &order.origin, r.turn_actor)?;
            }
            ordered(state, missile)?;
        }
        let all_decided = missile.order.is_some()
            && missile
                .respondents
                .iter()
                .all(|r| r.response.intent.is_some());
        let selected_after = missile
            .respondents
            .iter()
            .filter_map(|r| r.response.intent.as_ref().map(|i| &i.origin))
            .chain(missile.order.iter().map(|order| &order.origin))
            .max_by_key(|meta| meta.expected_event_sequence)
            .unwrap_or(&missile.cause);
        for (index, respondent) in missile.respondents.iter().enumerate() {
            let response = &respondent.response;
            if let Some(intent) = &response.intent {
                retain(&intent.origin, response.actor, &missile.cause)?;
                owner(state, &intent.origin, response.actor)?;
            }
            if response.declined_after_selection.is_some() && respondent.completed_shield.is_some()
                || (response.declined_after_selection.is_some()
                    || respondent.completed_shield.is_some())
                    && (!all_decided
                        || !response
                            .intent
                            .as_ref()
                            .is_some_and(|intent| intent.accepted))
            {
                return Err(invalid(
                    "missile response resolved without a unique accepted selection",
                ));
            }
            if let Some(declined) = &response.declined_after_selection {
                retain(declined, response.actor, selected_after)?;
                owner(state, declined, response.actor)?;
            }
            if let Some(shield) = respondent.completed_shield.as_deref() {
                retain(&shield.cast.plan.origin, response.actor, selected_after)?;
                validate_shield(state, missile, index, shield, selected_after)?;
            }
        }
        let pending_respondents = if all_decided {
            ordered(state, missile)?
                .into_iter()
                .filter_map(|actor| {
                    missile.respondents.iter().position(|r| {
                        r.response.actor == actor
                            && r.response.declined_after_selection.is_none()
                            && r.completed_shield.is_none()
                    })
                })
                .collect::<Vec<_>>()
        } else {
            vec![]
        };
        if all_decided {
            let mut previous = selected_after;
            let mut unfinished = false;
            for actor in ordered(state, missile)? {
                let respondent = missile
                    .respondents
                    .iter()
                    .find(|r| r.response.actor == actor)
                    .ok_or_else(|| invalid("ordered missile respondent is absent"))?;
                let completed = respondent
                    .response
                    .declined_after_selection
                    .as_ref()
                    .or_else(|| {
                        respondent
                            .completed_shield
                            .as_ref()
                            .map(|shield| &shield.cast.plan.origin)
                    });
                if let Some(completed) = completed {
                    if unfinished
                        || completed.expected_event_sequence <= previous.expected_event_sequence
                    {
                        return Err(invalid(
                            "missile responses did not resolve in their accepted order",
                        ));
                    }
                    previous = completed;
                } else {
                    unfinished = true;
                }
            }
        }
        match missile.stage {
            TacticalMissileStage::Collecting if !all_decided => (),
            TacticalMissileStage::Selected { respondent }
                if all_decided && pending_respondents.first() == Some(&usize::from(respondent)) => {
            }
            TacticalMissileStage::Amounts
            | TacticalMissileStage::Impacts
            | TacticalMissileStage::Completed
                if all_decided && pending_respondents.is_empty() => {}
            _ => {
                return Err(invalid(
                    "missile stage differs from its collected decisions",
                ));
            }
        }
        for (index, dart) in missile.darts.iter().enumerate() {
            if dart.at
                != (SpellProgramOccurrence {
                    node: 0,
                    target: index as u16,
                })
                || dart.target != cast.targets[index].actor
                || dart.known_target_label.trim().is_empty()
                || dart.known_target_label.len() > 4096
                || dart.amount.is_some()
                    && !matches!(
                        missile.stage,
                        TacticalMissileStage::Amounts
                            | TacticalMissileStage::Impacts
                            | TacticalMissileStage::Completed
                    )
                || matches!(
                    missile.stage,
                    TacticalMissileStage::Impacts | TacticalMissileStage::Completed
                ) && dart.amount.is_none()
                || dart.impact_occurrence.is_some()
                    != matches!(
                        missile.stage,
                        TacticalMissileStage::Impacts | TacticalMissileStage::Completed
                    )
                || dart.completed_by.is_some() && dart.impact_occurrence.is_none()
                || dart.selected_by.is_some() && dart.selected_by != dart.completed_by
                || missile.stage == TacticalMissileStage::Completed && dart.completed_by.is_none()
            {
                return Err(invalid(
                    "missile dart differs from its committed target or stage",
                ));
            }
            if let Some(key) = dart.amount {
                let work = r
                    .work_trace
                    .as_ref()
                    .and_then(|trace| {
                        trace
                            .nodes
                            .iter()
                            .find(|node| node.work.occurrence == key.occurrence)
                    })
                    .ok_or_else(|| invalid("missile amount lacks its source occurrence"))?;
                let roll = rules
                    .rolls
                    .iter()
                    .find(|roll| roll.request.id == key.request_id())
                    .ok_or_else(|| invalid("missile amount lacks its exact accepted raw record"))?;
                let mut incomplete = cast.clone();
                incomplete
                    .completed
                    .retain(|completed| *completed != dart.at);
                if key.origin != cast.cast.plan.origin.id
                    || key.subject != dart.target
                    || key.role != TacticalRollRole::SpellAmount
                    || work.work.kind
                        != (TacticalWorkKind::SpellProgram {
                            cast: missile.cast,
                            at: dart.at,
                        })
                    || work
                        .parent
                        .and_then(|parent| {
                            r.work_trace
                                .as_ref()?
                                .nodes
                                .iter()
                                .find(|node| node.work.occurrence == parent)
                        })
                        .is_none_or(|parent| {
                            parent.work.kind
                                != TacticalWorkKind::ResumeMissile {
                                    missile: missile.work.occurrence,
                                }
                        })
                    || roll.purpose
                        != (PendingPurpose::TacticalResolution {
                            encounter: encounter(state)?.id,
                            key,
                        })
                    || roll.request
                        != spell_amount_request(
                            &incomplete,
                            dart.at,
                            key.request_id(),
                            roll.request.visibility,
                        )?
                    || roll.request.resolve(&roll.result)? != roll.resolved
                    || roll.issued_by.expected_event_sequence
                        <= missile.cause.expected_event_sequence
                    || roll.issued_by.session_id != missile.cause.session_id
                {
                    return Err(invalid(
                        "missile amount differs from its original dice and source",
                    ));
                }
                if let Some(completed) = &dart.completed_by {
                    decision(state, completed, r.turn_actor, &roll.accepted_by)?;
                    if let Some(selected) = &dart.selected_by {
                        owner(state, selected, r.turn_actor)?;
                    } else if missile
                        .darts
                        .iter()
                        .any(|other| other.completed_by.is_none())
                        || cast.completed.last() != Some(&dart.at)
                    {
                        return Err(invalid(
                            "automatic missile completion preceded another impact",
                        ));
                    }
                }
            }
            if let Some(occurrence) = dart.impact_occurrence {
                let node = r
                    .work_trace
                    .as_ref()
                    .and_then(|trace| {
                        trace
                            .nodes
                            .iter()
                            .find(|node| node.work.occurrence == occurrence)
                    })
                    .ok_or_else(|| invalid("missile impact lacks its retained occurrence"))?;
                if node.work.kind
                    != (TacticalWorkKind::ApplyMissileImpact {
                        missile: missile.work.occurrence,
                        at: dart.at,
                    })
                    || node
                        .parent
                        .and_then(|parent| {
                            r.work_trace
                                .as_ref()?
                                .nodes
                                .iter()
                                .find(|node| node.work.occurrence == parent)
                        })
                        .is_none_or(|parent| {
                            parent.work.kind
                                != TacticalWorkKind::BeginMissileImpacts {
                                    missile: missile.work.occurrence,
                                }
                        })
                {
                    return Err(invalid("missile impact has a different barrier parent"));
                }
            }
        }
        if missile.completed_cast.is_none() {
            validate_cast_partition(state, cast)?;
        }
        if missile
            .darts
            .iter()
            .filter(|dart| dart.completed_by.is_some() && dart.selected_by.is_none())
            .count()
            > 1
        {
            return Err(invalid(
                "more than one missile impact bypassed explicit ordering",
            ));
        }
    }
    // Even retired new nodes must name an authenticated retained source record.
    if let Some(trace) = &r.work_trace {
        for node in &trace.nodes {
            let missile = match node.work.kind {
                TacticalWorkKind::BeginMissile { .. } => Some(node.work.occurrence),
                TacticalWorkKind::ResumeMissile { missile }
                | TacticalWorkKind::CommitMissileShield { missile, .. }
                | TacticalWorkKind::BeginMissileImpacts { missile }
                | TacticalWorkKind::ApplyMissileImpact { missile, .. } => Some(missile),
                _ => None,
            };
            if missile.is_some_and(|missile| !windows.contains(&missile)) {
                return Err(invalid("retired missile work lacks its committed source"));
            }
            if let TacticalWorkKind::ApplyMissileImpact { missile, at } = node.work.kind
                && current(state, missile)?
                    .darts
                    .get(usize::from(at.target))
                    .filter(|dart| dart.at == at)
                    .is_none_or(|dart| dart.impact_occurrence != Some(node.work.occurrence))
            {
                return Err(invalid(
                    "retired impact is not an exact committed dart occurrence",
                ));
            }
            if let TacticalWorkKind::CommitMissileShield {
                missile,
                respondent,
                cast,
            } = node.work.kind
            {
                let record = current(state, missile)?;
                if record
                    .respondents
                    .get(usize::from(respondent))
                    .and_then(|r| r.completed_shield.as_ref())
                    .is_none_or(|shield| shield.cast.plan.occurrence != cast)
                {
                    return Err(invalid(
                        "retired missile Shield has no exact selected source receipt",
                    ));
                }
            }
        }
        for missile in &r.missiles {
            let amounts = trace
                .nodes
                .iter()
                .filter_map(|node| match node.work.kind {
                    TacticalWorkKind::SpellProgram { cast, at } if cast == missile.cast => Some(at),
                    _ => None,
                })
                .collect::<Vec<_>>();
            let collecting_or_later = matches!(
                missile.stage,
                TacticalMissileStage::Amounts
                    | TacticalMissileStage::Impacts
                    | TacticalMissileStage::Completed
            );
            if amounts.len()
                != if collecting_or_later {
                    missile.darts.len()
                } else {
                    0
                }
                || collecting_or_later
                    && missile
                        .darts
                        .iter()
                        .any(|dart| amounts.iter().filter(|at| **at == dart.at).count() != 1)
            {
                return Err(invalid(
                    "retired amount work omits or duplicates a committed dart",
                ));
            }
            let resumes = trace
                .nodes
                .iter()
                .filter(|node| {
                    node.work.kind
                        == TacticalWorkKind::ResumeMissile {
                            missile: missile.work.occurrence,
                        }
                })
                .count();
            let barriers = trace
                .nodes
                .iter()
                .filter(|node| {
                    node.work.kind
                        == TacticalWorkKind::BeginMissileImpacts {
                            missile: missile.work.occurrence,
                        }
                })
                .count();
            if resumes != 1
                || barriers
                    != usize::from(matches!(
                        missile.stage,
                        TacticalMissileStage::Amounts
                            | TacticalMissileStage::Impacts
                            | TacticalMissileStage::Completed
                    ))
            {
                return Err(invalid(
                    "retired missile response or impact barrier is duplicated or absent",
                ));
            }
        }
    }
    Ok(())
}
