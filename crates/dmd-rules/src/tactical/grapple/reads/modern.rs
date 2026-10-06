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

struct AttackRawSource {
    origin: CommandId,
    spell: Option<crate::tactical_spells::SpellAttackOccurrence>,
}

fn raw_source_for_admission(
    state: &CampaignState,
    admission: &GrappleReadCut,
) -> Result<AttackRawSource, RulesError> {
    let GrappleReader::AttackAdmission { attack } = admission.key.reader else {
        return Err(invalid("raw source is not an attack admission"));
    };
    let trace = trace(state)?;
    let mut at = Some(admission.key.work.occurrence);
    while let Some(current) = at {
        let mut nodes = trace.nodes.iter().filter(|node| node.work.occurrence == current);
        let node = nodes.next().ok_or_else(|| invalid("raw source ancestry absent"))?;
        if nodes.next().is_some() || node.parent.is_some_and(|parent| parent >= current) {
            return Err(invalid("raw source ancestry is ambiguous or noncausal"));
        }
        if let TacticalWorkKind::SpellProgram { cast, at } = node.work.kind {
            let mut records = resolution(state)?.casts.iter()
                .filter(|record| record.cast.plan.occurrence == cast);
            let record = records.next().ok_or_else(|| invalid("raw source cast absent"))?;
            if records.next().is_some() {
                return Err(invalid("raw source cast is ambiguous"));
            }
            // Authenticate the complete original first. The temporary source-only
            // image permits reconstructing an earlier completed ray, not executing it.
            crate::tactical_spells::retained_spell_binding(record)?;
            let mut source = record.clone();
            source.completed.retain(|completed| *completed != at);
            let bound = crate::tactical_spells::retained_spell_binding(&source)?;
            let spell = crate::tactical_spells::spell_attack_occurrence(
                &source.cast, &bound, at.node, at.target,
            )?;
            return Ok(AttackRawSource { origin: spell.origin().id, spell: Some(spell) });
        }
        at = node.parent;
    }
    Ok(AttackRawSource { origin: attack, spell: None })
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
    let window = if let TacticalAttackAdmission::Opportunity(window) = &attack.admission {
        Some((GrappleCutKey {
            work: key.work,
            reader: GrappleReader::OpportunityWindow {
                attack: attack.origin.id, window: window.origin.id,
                reactor: window.reactor, mover: window.mover, step: window.step_index,
            },
        }, live(state).filter(|grip| grip.declaration.grappler == attack.actor)
            .cloned().collect::<Vec<_>>()))
    } else { None };
    let all_proofs = proofs.into_iter().chain(window.iter().flat_map(|(_, proofs)| proofs.iter().cloned()));
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
    for proof in all_proofs {
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
    if let Some((key, proofs)) = window {
        c.cuts.push(GrappleReadCut {
            key, issued_by: meta.clone(),
            grips: proofs.iter().map(|proof| proof.declaration.id).collect(),
            source_attack: None,
        });
    }
    validate_cuts(state)
}

pub(super) fn window_cut<'a>(
    state: &'a CampaignState,
    attack: &TacticalAttack,
) -> Result<&'a GrappleReadCut, RulesError> {
    let TacticalAttackAdmission::Opportunity(window) = &attack.admission else {
        return Err(invalid("window read is not an opportunity attack"));
    };
    let admission = current_admission(state, attack)?;
    let key = GrappleCutKey { work: admission.key.work,
        reader: GrappleReader::OpportunityWindow {
            attack: attack.origin.id, window: window.origin.id,
            reactor: window.reactor, mover: window.mover, step: window.step_index,
        },
    };
    let mut cuts = context(state)?.cuts.iter().filter(|cut| cut.key == key);
    let cut = cuts.next().ok_or_else(|| invalid("opportunity attack lacks original window hands"))?;
    if cuts.next().is_some() { return Err(invalid("opportunity window cut is ambiguous")); }
    Ok(cut)
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
    let mut issued = Vec::new();
    for (purpose, request) in rules.rolls.iter().map(|raw| (&raw.purpose, &raw.request))
        .chain(rules.pending.iter().map(|raw| (&raw.purpose, &raw.request)))
    {
        let PendingPurpose::TacticalResolution { encounter: id, key } = purpose else { continue };
        if *id != encounter(state)?.id
            || !matches!(key.role, TacticalRollRole::Attack | TacticalRollRole::AttackDamage)
            || !trace.nodes.iter().any(|node| node.work.occurrence == key.occurrence)
        { continue; }
        // Older retired resolutions can reuse occurrence ordinals. Their paid
        // origins do not belong to this trace and must not become current cuts.
        let Ok(admission) = admission_for_work(state, key.occurrence) else { continue };
        let source = raw_source_for_admission(state, admission)?;
        if source.origin != key.origin { continue; }
        if source.spell.as_ref().is_some_and(|spell|
            spell.target() != key.subject || request.roller != Some(spell.actor()))
        {
            return Err(invalid("spell raw differs from its exact paid occurrence"));
        }
        issued.push(*key);
    }
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
            GrappleReader::OpportunityWindow { attack, window, reactor, mover, .. } => {
                let admission = admission_for_work(state, node.work.occurrence)?;
                if node.work.kind != TacticalWorkKind::AttackRoll
                    || cut.issued_by.id != attack || window.0.is_nil() || reactor == mover
                    || cut.source_attack.is_some()
                    || admission.key.reader != (GrappleReader::AttackAdmission { attack })
                    || admission.issued_by != cut.issued_by
                    || cut.grips.iter().any(|id| c.proofs.iter().find(|proof| proof.declaration.id == *id)
                        .is_none_or(|proof| proof.declaration.grappler != reactor))
                { return Err(invalid("opportunity window source or reservations differ")); }
                let mut at = node.parent;
                let mut found = false;
                while let Some(occurrence) = at {
                    let mut nodes = trace.nodes.iter().filter(|node| node.work.occurrence == occurrence);
                    let parent = nodes.next().ok_or_else(|| invalid("opportunity ancestry absent"))?;
                    if nodes.next().is_some() || parent.parent.is_some_and(|next| next >= occurrence) {
                        return Err(invalid("opportunity ancestry is ambiguous or noncausal"));
                    }
                    if parent.work.kind == (TacticalWorkKind::MovementOpportunity { reactor }) {
                        found = true;
                        break;
                    }
                    at = parent.parent;
                }
                if !found { return Err(invalid("window cut lacks actual opportunity ancestry")); }
            }
            GrappleReader::RequestIssue { roll } => {
                let source = admission_for_work(state, node.work.occurrence)?;
                let raw_source = raw_source_for_admission(state, source)?;
                let kind = match roll.role {
                    TacticalRollRole::Attack => TacticalWorkKind::AttackRoll,
                    TacticalRollRole::AttackDamage => TacticalWorkKind::AttackDamage,
                    _ => return Err(invalid("unsupported attack raw role")),
                };
                if node.work.kind != kind
                    || roll.occurrence != node.work.occurrence
                    || raw_source.origin != roll.origin
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
        if matches!(attack.admission, TacticalAttackAdmission::Opportunity(_)) {
            window_cut(state, attack)?;
        }
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
