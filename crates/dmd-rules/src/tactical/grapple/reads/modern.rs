//! Occurrence-bound attack reads for the original-history-owned table executor.
use super::*;

fn trace(state: &CampaignState) -> Result<&TacticalWorkTrace, RulesError> {
    resolution(state)?
        .work_trace
        .as_ref()
        .ok_or_else(|| invalid("attack ancestry absent"))
}

fn admission_for_work(
    state: &CampaignState,
    occurrence: u16,
) -> Result<&GrappleReadCut, RulesError> {
    let c = context(state)?;
    let mut at = Some(occurrence);
    let trace = trace(state)?;
    for _ in 0..=trace.nodes.len() {
        let current = at.ok_or_else(|| invalid("attack work has no admission ancestor"))?;
        let node = trace
            .nodes
            .iter()
            .find(|n| n.work.occurrence == current)
            .ok_or_else(|| invalid("attack ancestor absent"))?;
        if node.work.kind == TacticalWorkKind::AttackRoll {
            let mut cuts = c.cuts.iter().filter(|cut| {
                cut.key.work.occurrence == current
                    && matches!(cut.key.reader, GrappleReader::AttackAdmission { .. })
            });
            let cut = cuts
                .next()
                .ok_or_else(|| invalid("attack root has no admission"))?;
            if cuts.next().is_some() {
                return Err(invalid("attack root has multiple admissions"));
            }
            return Ok(cut);
        }
        if node.parent.is_some_and(|parent| parent >= current) {
            return Err(invalid("attack ancestry is not causal"));
        }
        at = node.parent;
    }
    Err(invalid("attack ancestry is cyclic"))
}

pub(super) fn current_admission<'a>(
    state: &'a CampaignState,
    attack: &TacticalAttack,
) -> Result<&'a GrappleReadCut, RulesError> {
    // There is one attached attack. A later ray may share the accepting command
    // with an earlier ray, so command identity alone is never an occurrence key.
    context(state)?
        .cuts
        .iter()
        .rev()
        .find(|cut| {
            cut.key.reader
                == (GrappleReader::AttackAdmission {
                    attack: attack.origin.id,
                })
        })
        .ok_or_else(|| invalid("attached attack has no occurrence admission"))
}

pub(super) fn capture_admission(
    state: &mut CampaignState,
    meta: &CommandMeta,
    proofs: Vec<TacticalGrip>,
) -> Result<(), RulesError> {
    let r = resolution(state)?;
    let attack = r
        .attack
        .as_ref()
        .ok_or_else(|| invalid("admission has no actual attack"))?;
    let work = r
        .frames
        .last()
        .and_then(|f| f.last())
        .filter(|w| w.kind == TacticalWorkKind::AttackRoll)
        .ok_or_else(|| invalid("admission lacks the just-created AttackRoll"))?;
    if attack.origin != *meta
        || trace(state)?.nodes.last().is_none_or(|n| n.work != *work)
        || proofs
            != live(state)
                .filter(|g| relevant(g, attack.actor, attack.target, physical(attack)))
                .cloned()
                .collect::<Vec<_>>()
    {
        return Err(invalid("attack admission differs from its actual producer"));
    }
    let key = GrappleCutKey {
        work: work_key(state, work)?,
        reader: GrappleReader::AttackAdmission { attack: meta.id },
    };
    let grips = proofs.iter().map(|proof| proof.declaration.id).collect();
    let c = resolution_mut(state)?.grapple.get_or_insert_with(|| {
        Box::new(TacticalGrappleResolution {
            activity: None,
            proofs: vec![],
            cuts: vec![],
            ends: vec![],
            opportunity_refreshes: vec![],
        })
    });
    if c.cuts.iter().any(|cut| cut.key == key) {
        return Err(invalid("attack admission already captured"));
    }
    for proof in proofs {
        if let Some(existing) = c
            .proofs
            .iter()
            .find(|old| old.declaration.id == proof.declaration.id)
        {
            if existing != &proof {
                return Err(invalid("attack rewrote a retained grip"));
            }
        } else {
            c.proofs.push(proof);
        }
    }
    c.proofs.sort_by_key(|proof| proof.declaration.id.0);
    c.cuts.push(GrappleReadCut {
        key,
        issued_by: meta.clone(),
        grips,
        source_attack: None,
    });
    validate_cuts(state)
}

pub(super) fn capture_issue(
    state: &mut CampaignState,
    meta: &CommandMeta,
    work: &TacticalWorkItem,
    roll: TacticalRollKey,
) -> Result<(), RulesError> {
    if !matches!(
        roll.role,
        TacticalRollRole::Attack | TacticalRollRole::AttackDamage
    ) {
        return Ok(());
    }
    let source = admission_for_work(state, work.occurrence)?.clone();
    if trace(state)?.active != Some(work.occurrence) {
        return Err(invalid("request has no entered producer"));
    }
    let key = GrappleCutKey {
        work: work_key(state, work)?,
        reader: GrappleReader::RequestIssue { roll },
    };
    context_mut(state)?.cuts.push(GrappleReadCut {
        key,
        issued_by: meta.clone(),
        grips: source.grips,
        source_attack: Some(source.key),
    });
    validate_cuts(state)
}

pub(super) fn validate_cuts(state: &CampaignState) -> Result<(), RulesError> {
    let r = resolution(state)?;
    let Some(c) = &r.grapple else {
        return Ok(());
    };
    let trace = trace(state)?;
    let rules = state.rules.as_ref().ok_or(RulesError::Uninitialized)?;
    let issued = rules.rolls.iter().map(|raw| &raw.purpose)
        .chain(rules.pending.iter().map(|p| &p.purpose))
        .filter_map(|p| match p {
            PendingPurpose::TacticalResolution { encounter: id, key } if *id == encounter(state).ok()?.id
                && matches!(key.role, TacticalRollRole::Attack | TacticalRollRole::AttackDamage)
                && trace.nodes.iter().any(|node| node.work.occurrence == key.occurrence)
                && c.cuts.iter().any(|cut| matches!(cut.key.reader, GrappleReader::AttackAdmission { attack } if attack == key.origin)) => Some(*key),
            _ => None,
        }).collect::<Vec<_>>();
    for cut in &c.cuts {
        let node = trace
            .nodes
            .iter()
            .find(|n| n.work.occurrence == cut.key.work.occurrence)
            .ok_or_else(|| invalid("cut work absent"))?;
        if cut.key.work.resolution != r.origin.id
            || cut.issued_by.campaign_id != state.campaign_id()
            || cut.issued_by.expected_event_sequence < r.origin.expected_event_sequence
            || cut.issued_by.expected_event_sequence > state.applied_event_sequence
            || c.cuts.iter().filter(|other| other.key == cut.key).count() != 1
            || cut
                .grips
                .iter()
                .any(|id| c.proofs.iter().filter(|p| p.declaration.id == *id).count() != 1)
        {
            return Err(invalid(
                "cut identity, chronology, or proof closure differs",
            ));
        }
        match cut.key.reader {
            GrappleReader::AttackAdmission { attack } => {
                if node.work.kind != TacticalWorkKind::AttackRoll
                    || attack != cut.issued_by.id
                    || cut.source_attack.is_some()
                {
                    return Err(invalid("attack admission source differs"));
                }
            }
            GrappleReader::RequestIssue { roll } => {
                let source = admission_for_work(state, node.work.occurrence)?;
                let kind = match roll.role {
                    TacticalRollRole::Attack => TacticalWorkKind::AttackRoll,
                    TacticalRollRole::AttackDamage => TacticalWorkKind::AttackDamage,
                    _ => return Err(invalid("unsupported attack raw role")),
                };
                if node.work.kind != kind
                    || roll.occurrence != node.work.occurrence
                    || source.key.reader
                        != (GrappleReader::AttackAdmission {
                            attack: roll.origin,
                        })
                    || cut.source_attack != Some(source.key)
                    || cut.grips != source.grips
                    || cut.issued_by.expected_event_sequence
                        < source.issued_by.expected_event_sequence
                    || issued.iter().filter(|key| **key == roll).count() != 1
                {
                    return Err(invalid("request differs from its exact attack occurrence"));
                }
            }
            GrappleReader::FlightLoss { .. } => {
                // Its separate lifecycle validator authenticates the actual fall producer.
                if cut.source_attack.is_some() {
                    return Err(invalid("flight loss inherits an attack admission"));
                }
            }
        }
    }
    for key in issued {
        if c.cuts
            .iter()
            .filter(|cut| cut.key.reader == (GrappleReader::RequestIssue { roll: key }))
            .count()
            != 1
        {
            return Err(invalid("actual attack raw lacks its unique issue cut"));
        }
    }
    if let Some(attack) = &r.attack {
        let admission = current_admission(state, attack)?;
        for id in &admission.grips {
            let proof = c
                .proofs
                .iter()
                .find(|proof| proof.declaration.id == *id)
                .ok_or_else(|| invalid("attack proof absent"))?;
            if !relevant(proof, attack.actor, attack.target, physical(attack)) {
                return Err(invalid("unrelated attack proof"));
            }
        }
        for id in [attack.attack_roll, attack.damage_roll]
            .into_iter()
            .flatten()
        {
            let raw = rules
                .rolls
                .iter()
                .find(|raw| raw.request.id == id)
                .ok_or_else(|| invalid("attached attack raw absent"))?;
            let PendingPurpose::TacticalResolution { key, .. } = raw.purpose else {
                return Err(invalid("attack raw purpose differs"));
            };
            if key.subject != attack.target
                || raw.request.roller != Some(attack.actor)
                || admission_for_work(state, key.occurrence)?.key != admission.key
            {
                return Err(invalid("attached attack borrows another occurrence's raw"));
            }
        }
    }
    Ok(())
}

pub(super) fn attack_root(state: &CampaignState) -> Result<&TacticalWorkNode, RulesError> {
    let attack = resolution(state)?
        .attack
        .as_ref()
        .ok_or_else(|| invalid("attached attack absent"))?;
    let source = current_admission(state, attack)?;
    trace(state)?
        .nodes
        .iter()
        .find(|node| node.work.occurrence == source.key.work.occurrence)
        .ok_or_else(|| invalid("attached attack root absent"))
}
