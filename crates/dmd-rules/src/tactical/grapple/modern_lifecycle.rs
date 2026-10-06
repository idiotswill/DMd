//! Actual work-owned relation ends and flight reads for the activated table path.
use super::*;

fn empty_context() -> TacticalGrappleResolution {
    TacticalGrappleResolution {
        activity: None,
        proofs: vec![],
        cuts: vec![],
        ends: vec![],
        opportunity_refreshes: vec![],
    }
}

pub(super) fn retain(state: &mut CampaignState, grip: &TacticalGrip) -> Result<(), RulesError> {
    let c = resolution_mut(state)?
        .grapple
        .get_or_insert_with(|| Box::new(empty_context()));
    if let Some(old) = c
        .proofs
        .iter()
        .find(|old| old.declaration.id == grip.declaration.id)
    {
        if old != grip {
            return Err(invalid("lifecycle rewrites an inherited grip"));
        }
    } else {
        c.proofs.push(grip.clone());
        c.proofs.sort_by_key(|grip| grip.declaration.id.0);
    }
    Ok(())
}

fn active_work(state: &CampaignState) -> Result<TacticalWorkItem, RulesError> {
    let trace = resolution(state)?
        .work_trace
        .as_ref()
        .ok_or_else(|| invalid("lifecycle ancestry absent"))?;
    let occurrence = trace
        .active
        .ok_or_else(|| invalid("lifecycle has no actual entered work"))?;
    trace
        .nodes
        .iter()
        .find(|n| n.work.occurrence == occurrence)
        .map(|n| n.work.clone())
        .ok_or_else(|| invalid("lifecycle active work absent"))
}

fn moved_actor(state: &CampaignState, work: &TacticalWorkItem) -> Result<EntityId, RulesError> {
    let r = resolution(state)?;
    match work.kind {
        TacticalWorkKind::MoveSegment => r.movement.as_ref().map(|m| m.actor).or_else(|| {
            flow(state)
                .ok()?
                .last_movement
                .as_ref()
                .filter(|m| m.original == r.origin)
                .map(|m| m.actor)
        }),
        TacticalWorkKind::ChooseShoveOutcome | TacticalWorkKind::FinishShove => {
            r.shove.as_ref().map(|s| s.target)
        }
        TacticalWorkKind::BeginFall { fall }
        | TacticalWorkKind::FallDamage { fall }
        | TacticalWorkKind::LiquidLandingCheck { fall } => {
            r.falls.get(usize::from(fall)).map(|fall| fall.actor)
        }
        _ => None,
    }
    .ok_or_else(|| invalid("out-of-range relation lacks its actual displacement work"))
}

pub(super) fn settle(state: &mut CampaignState, meta: &CommandMeta) -> Result<(), RulesError> {
    let grips = state
        .rules
        .as_ref()
        .and_then(|rules| rules.tactical_grapples.as_ref())
        .map(|live| live.active.clone())
        .unwrap_or_default();
    if grips.is_empty() {
        return Ok(());
    }
    let work = active_work(state)?;
    let key = work_key(state, &work)?;
    for grip in grips {
        let rules = state.rules.as_ref().ok_or(RulesError::Uninitialized)?;
        let holder = grip.declaration.grappler;
        let cause = if rules.entities.get(&holder).is_none_or(|e| e.death.dead) {
            Some(GrappleEndCause::Dead { work: key })
        } else if !crate::tactical_conditions::can_act(rules, holder)? {
            Some(GrappleEndCause::Incapacitated { work: key })
        } else {
            let e = encounter(state)?;
            let a = e
                .participant(holder)
                .ok_or_else(|| invalid("holder absent"))?;
            let b = e
                .participant(grip.declaration.target)
                .ok_or_else(|| invalid("held target absent"))?;
            if crate::spatial::participant_distance(a, b).map_err(|e| invalid(&e.to_string()))?
                > grip.declaration.range
            {
                let moved_actor = moved_actor(state, &work)?;
                if moved_actor != holder && moved_actor != grip.declaration.target {
                    return Err(invalid("range end belongs to an unrelated mover"));
                }
                Some(GrappleEndCause::OutOfRange {
                    work: key,
                    moved_actor,
                })
            } else {
                None
            }
        };
        if let Some(cause) = cause {
            lifecycle::end_grip(state, meta, grip.declaration.id, cause)?;
        }
    }
    Ok(())
}

pub(super) fn flight_cause(
    state: &mut CampaignState,
    meta: &CommandMeta,
    actor: EntityId,
) -> Result<Option<TacticalFallCause>, RulesError> {
    let proofs = state
        .rules
        .as_ref()
        .and_then(|rules| rules.tactical_grapples.as_ref())
        .map(|live| {
            live.active
                .iter()
                .filter(|g| g.declaration.target == actor)
                .cloned()
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let Some(source) = proofs.iter().find(|g| g.established_by == *meta).cloned() else {
        return Ok(None);
    };
    let work = active_work(state)?;
    let key = GrappleCutKey {
        work: work_key(state, &work)?,
        reader: GrappleReader::FlightLoss { actor },
    };
    if source.work != key.work {
        return Err(invalid(
            "flight loss is not owned by the actual grip producer",
        ));
    }
    for proof in &proofs {
        retain(state, proof)?;
    }
    if context(state)?.cuts.iter().any(|cut| cut.key == key) {
        return Err(invalid("flight loss was captured twice"));
    }
    context_mut(state)?.cuts.push(GrappleReadCut {
        key,
        issued_by: meta.clone(),
        grips: proofs.iter().map(|g| g.declaration.id).collect(),
        source_attack: None,
    });
    Ok(Some(TacticalFallCause::GrappleFlightLost {
        grip: source.declaration.id,
        consequence: source.established_by,
        work: source.work,
        cut: key,
    }))
}

pub(super) fn validate_flight(
    state: &CampaignState,
    fall: &TacticalFall,
) -> Result<(), RulesError> {
    let TacticalFallCause::GrappleFlightLost {
        grip,
        consequence,
        work,
        cut,
    } = &fall.cause
    else {
        return Err(invalid("not a Grapple flight loss"));
    };
    let c = context(state)?;
    let proof = c
        .proofs
        .iter()
        .find(|p| p.declaration.id == *grip)
        .ok_or_else(|| invalid("flight source proof absent"))?;
    let read = c
        .cuts
        .iter()
        .find(|read| read.key == *cut)
        .ok_or_else(|| invalid("flight source cut absent"))?;
    if proof.declaration.target != fall.actor
        || proof.work != *work
        || proof.established_by != *consequence
        || fall.origin != *consequence
        || read.issued_by != *consequence
        || read.source_attack.is_some()
        || read.key.work != *work
        || read.key.reader != (GrappleReader::FlightLoss { actor: fall.actor })
        || !read.grips.contains(grip)
    {
        return Err(invalid("fall differs from its actual establishing work"));
    }
    let mut image = state.clone();
    let participants = &mut image
        .encounter
        .as_mut()
        .ok_or_else(|| invalid("fall encounter absent"))?
        .participants;
    let actor = participants
        .iter_mut()
        .find(|p| p.entity_id == fall.actor)
        .ok_or_else(|| invalid("fall actor absent"))?;
    actor.position = fall.path.from;
    let proofs = read
        .grips
        .iter()
        .map(|id| {
            c.proofs
                .iter()
                .find(|p| p.declaration.id == *id)
                .filter(|p| p.declaration.target == fall.actor)
                .cloned()
                .ok_or_else(|| invalid("flight cut contains unrelated proof"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    image
        .rules
        .as_mut()
        .ok_or(RulesError::Uninitialized)?
        .tactical_grapples = Some(TacticalGrapples {
        schema_version: TACTICAL_GRAPPLES_SCHEMA_VERSION,
        active: proofs,
    });
    if crate::spatial::flight_loss_fall(encounter(&image)?, &image, fall.actor)
        .map_err(|e| invalid(&e.to_string()))?
        .as_ref()
        != Some(&fall.path)
    {
        return Err(invalid(
            "flight read does not reproduce its source geometry",
        ));
    }
    Ok(())
}

pub(super) fn validate_end(
    state: &CampaignState,
    proof: &TacticalGrip,
    end: &GrappleEndReceipt,
) -> Result<(), RulesError> {
    let work = end
        .cause
        .work()
        .ok_or_else(|| invalid("work-owned ending has no source"))?;
    let r = resolution(state)?;
    let node = r
        .work_trace
        .as_ref()
        .and_then(|t| {
            t.nodes
                .iter()
                .find(|n| n.work.occurrence == work.occurrence)
        })
        .ok_or_else(|| invalid("relation ending work absent"))?;
    if work.resolution != r.origin.id
        || end.caused_by.campaign_id != state.campaign_id()
        || end.caused_by.expected_event_sequence < proof.established_by.expected_event_sequence
        || end.caused_by.expected_event_sequence > state.applied_event_sequence
    {
        return Err(invalid("relation ending chronology differs"));
    }
    if let GrappleEndCause::OutOfRange {
        moved_actor: actor, ..
    } = end.cause
    {
        if actor != proof.declaration.grappler && actor != proof.declaration.target
            || moved_actor(state, &node.work)? != actor
        {
            return Err(invalid(
                "relation ending differs from its displacement source",
            ));
        }
    }
    Ok(())
}
