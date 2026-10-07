//! Sealed attack facts borrowed from the actual execution candidate. A physical
//! reconstruction copy never receives execution authority.
use super::*;
use execution::ReadContext;
mod modern;

pub(in crate::tactical) fn retain_completed_cast(
    state: &mut CampaignState,
    meta: &CommandMeta,
    record: &TacticalCasting,
) -> Result<(), RulesError> {
    if crate::table::grapple_enabled(state) && resolution(state)?.grapple.is_some() {
        modern::retain(state, meta, record)?;
    }
    Ok(())
}

pub(crate) struct AttackRead<'a> {
    state: &'a CampaignState,
    actor: EntityId,
    target: EntityId,
    physical: bool,
    grips: Vec<TacticalGrip>,
}

/// The complete selected opportunity menu has a hand read of its own. Unarmed
/// attack conditions intentionally do not inherit outgoing hand reservations.
pub(crate) struct OpportunityWindowRead<'a> {
    state: &'a CampaignState,
    actor: EntityId,
    grips: Vec<TacticalGrip>,
}

impl<'a> OpportunityWindowRead<'a> {
    pub(in crate::tactical) fn retained(
        read: &ReadContext<'a>,
        attack: &TacticalAttack,
    ) -> Result<Self, RulesError> {
        let state = read.state();
        read.require_guarded("window hands require original owned execution")?;
        if !crate::table::grapple_enabled(state)
            || resolution(state)?.attack.as_ref() != Some(attack)
        {
            return Err(invalid(
                "window read differs from its attached enabled attack",
            ));
        }
        modern::validate_cuts(state)?;
        let cut = modern::window_cut(state, attack)?;
        let proofs = &context(state)?.proofs;
        let grips = cut
            .grips
            .iter()
            .map(|id| {
                proofs
                    .iter()
                    .find(|proof| proof.declaration.id == *id)
                    .cloned()
                    .ok_or_else(|| invalid("window hand proof absent"))
            })
            .collect::<Result<_, _>>()?;
        Ok(Self {
            state,
            actor: attack.actor,
            grips,
        })
    }
    pub(crate) fn state(&self) -> &'a CampaignState {
        self.state
    }
    pub(crate) fn actor(&self) -> EntityId {
        self.actor
    }
    pub(crate) fn grips(&self) -> &[TacticalGrip] {
        &self.grips
    }
}

fn physical(attack: &TacticalAttack) -> bool {
    attack.weapon().is_some()
        || matches!(
            attack.source,
            TacticalAttackSource::CreatureFeature {
                weapon: Some(_),
                ..
            }
        )
}

fn relevant(grip: &TacticalGrip, actor: EntityId, target: EntityId, physical: bool) -> bool {
    grip.declaration.target == actor
        || grip.declaration.target == target
        || (physical && grip.declaration.grappler == actor)
}

fn live(state: &CampaignState) -> impl Iterator<Item = &TacticalGrip> {
    state
        .rules
        .iter()
        .flat_map(|r| r.tactical_grapples.iter())
        .flat_map(|g| &g.active)
}

impl<'a> AttackRead<'a> {
    pub(in crate::tactical) fn current(
        read: &ReadContext<'a>,
        actor: EntityId,
        target: EntityId,
        physical: bool,
    ) -> Result<Self, RulesError> {
        let state = read.state();
        if has_unimplemented_grapple_records(state) {
            read.require_guarded("attack reads require private execution continuity")?;
        }
        let grips = live(state)
            .filter(|g| relevant(g, actor, target, physical))
            .cloned()
            .collect();
        Ok(Self {
            state,
            actor,
            target,
            physical,
            grips,
        })
    }

    pub(in crate::tactical) fn retained(
        read: &ReadContext<'a>,
        attack: &TacticalAttack,
    ) -> Result<Self, RulesError> {
        let state = read.state();
        if !has_unimplemented_grapple_records(state) {
            return Self::current(read, attack.actor, attack.target, physical(attack));
        }
        read.require_guarded("retained attack lacks private execution continuity")?;
        if resolution(state)?.attack.as_ref() != Some(attack) {
            return Err(invalid("attack read differs from the attached declaration"));
        }
        validate_cuts(state)?;
        let mut result = Self::current(read, attack.actor, attack.target, physical(attack))?;
        if !result.grips.is_empty() && resolution(state)?.grapple.is_none() {
            return Err(invalid(
                "relevant live relation lacks its admitted attack cut",
            ));
        }
        if let Some(c) = &resolution(state)?.grapple
            && let Some(cut) = c.cuts.iter().rev().find(|cut| {
                matches!(cut.key.reader,
                GrappleReader::AttackAdmission { attack: id } if id == attack.origin.id)
            })
        {
            result.grips = cut
                .grips
                .iter()
                .map(|id| {
                    c.proofs
                        .iter()
                        .find(|p| p.declaration.id == *id)
                        .cloned()
                        .ok_or_else(|| invalid("admission proof absent"))
                })
                .collect::<Result<_, _>>()?;
        }
        Ok(result)
    }

    pub(crate) fn state(&self) -> &'a CampaignState {
        self.state
    }
    pub(crate) fn actor(&self) -> EntityId {
        self.actor
    }
    pub(crate) fn target(&self) -> EntityId {
        self.target
    }
    pub(crate) fn grips(&self) -> &[TacticalGrip] {
        &self.grips
    }
    pub(crate) fn uses_hands(&self) -> bool {
        self.physical
    }
    pub(in crate::tactical) fn captured(&self) -> Vec<TacticalGrip> {
        self.grips.clone()
    }

    /// Only the new live-grip contribution is overlaid. Every old effect, source,
    /// entity, Dodge record and physical fact remains current.
    pub(crate) fn condition_rules(&self) -> RulesState {
        let mut rules = self
            .state
            .rules
            .as_ref()
            .expect("authenticated rules")
            .clone();
        rules.tactical_grapples = (!self.grips.is_empty()).then(|| TacticalGrapples {
            schema_version: TACTICAL_GRAPPLES_SCHEMA_VERSION,
            active: self.grips.clone(),
        });
        rules
    }
}

pub(in crate::tactical) fn capture_admission(
    state: &mut CampaignState,
    meta: &CommandMeta,
    proofs: Vec<TacticalGrip>,
) -> Result<(), RulesError> {
    if crate::table::grapple_enabled(state) {
        return modern::capture_admission(state, meta, proofs);
    }
    if proofs.is_empty() {
        return Ok(());
    }
    let r = resolution(state)?;
    let attack = r
        .attack
        .as_ref()
        .ok_or_else(|| invalid("admission has no attack"))?;
    if attack.origin != *meta || r.origin != *meta || r.frames.len() != 1 || r.frames[0].len() != 1
    {
        return Err(invalid("admission is not the actual fresh attack producer"));
    }
    let work = &r.frames[0][0];
    if work.kind != TacticalWorkKind::AttackRoll {
        return Err(invalid("admission has no actual AttackRoll"));
    }
    let key = GrappleCutKey {
        work: work_key(state, work)?,
        reader: GrappleReader::AttackAdmission { attack: meta.id },
    };
    let expected: Vec<_> = live(state)
        .filter(|g| relevant(g, attack.actor, attack.target, physical(attack)))
        .cloned()
        .collect();
    if proofs != expected || r.grapple.is_some() {
        return Err(invalid("fresh admission membership differs"));
    }
    let grips = proofs.iter().map(|g| g.declaration.id).collect();
    resolution_mut(state)?.grapple = Some(Box::new(TacticalGrappleResolution {
        transport: None,
        activity: None,
        proofs,
        cuts: vec![GrappleReadCut {
            key,
            issued_by: meta.clone(),
            grips,
            source_attack: None,
        }],
        ends: vec![],
        opportunity_refreshes: vec![],
        completed_casts: vec![],
    }));
    validate_cuts(state)
}

pub(in crate::tactical) fn capture_issue(
    state: &mut CampaignState,
    meta: &CommandMeta,
    work: &TacticalWorkItem,
    roll: TacticalRollKey,
) -> Result<(), RulesError> {
    if crate::table::grapple_enabled(state) {
        return modern::capture_issue(state, meta, work, roll);
    }
    if !matches!(
        roll.role,
        TacticalRollRole::Attack | TacticalRollRole::AttackDamage
    ) {
        return Ok(());
    }
    let Some(c) = resolution(state)?.grapple.as_ref() else {
        return Ok(());
    };
    let Some(source) = c.cuts.iter().find(|cut| matches!(cut.key.reader, GrappleReader::AttackAdmission { attack } if attack == roll.origin)).cloned() else {
        return Err(invalid("attack request has no admission"));
    };
    let key = GrappleCutKey {
        work: work_key(state, work)?,
        reader: GrappleReader::RequestIssue { roll },
    };
    if resolution(state)?
        .work_trace
        .as_ref()
        .and_then(|t| t.active)
        != Some(work.occurrence)
    {
        return Err(invalid("request cut lacks its actual entered producer"));
    }
    context_mut(state)?.cuts.push(GrappleReadCut {
        key,
        issued_by: meta.clone(),
        grips: source.grips,
        source_attack: Some(source.key),
    });
    validate_cuts(state)
}

pub(in crate::tactical) fn attack_root(
    state: &CampaignState,
) -> Result<&TacticalWorkNode, RulesError> {
    if crate::table::grapple_enabled(state) {
        return modern::attack_root(state);
    }
    let trace = resolution(state)?
        .work_trace
        .as_ref()
        .ok_or_else(|| invalid("attack trace absent"))?;
    let mut roots = trace
        .nodes
        .iter()
        .filter(|n| n.work.kind == TacticalWorkKind::AttackRoll && n.parent.is_none());
    let root = roots
        .next()
        .ok_or_else(|| invalid("own-turn attack root absent"))?;
    if roots.next().is_some() {
        return Err(invalid("multiple own-turn attack roots"));
    }
    Ok(root)
}

pub(in crate::tactical) fn in_lineage(
    state: &CampaignState,
    occurrence: u16,
    root: u16,
) -> Result<bool, RulesError> {
    let trace = resolution(state)?
        .work_trace
        .as_ref()
        .ok_or_else(|| invalid("attack trace absent"))?;
    let mut at = Some(occurrence);
    for _ in 0..trace.nodes.len() {
        let Some(current) = at else {
            return Ok(false);
        };
        if current == root {
            return Ok(true);
        }
        let n = trace
            .nodes
            .binary_search_by_key(&current, |n| n.work.occurrence)
            .ok()
            .and_then(|index| trace.nodes.get(index))
            .ok_or_else(|| invalid("ancestor absent"))?;
        if n.parent.is_some_and(|p| p >= current) {
            return Err(invalid("attack ancestry is not causal"));
        }
        at = n.parent;
    }
    Ok(false)
}

pub(in crate::tactical) fn validate_cuts(state: &CampaignState) -> Result<(), RulesError> {
    if crate::table::grapple_enabled(state) {
        return modern::validate_cuts(state);
    }
    let r = resolution(state)?;
    let Some(c) = r.grapple.as_ref().filter(|c| c.activity.is_none()) else {
        return Ok(());
    };
    if c.cuts.is_empty() || !c.opportunity_refreshes.is_empty() || !c.completed_casts.is_empty() {
        return Err(invalid("unsupported cut-only context"));
    }
    let root = attack_root(state)?;
    let mut admissions = c
        .cuts
        .iter()
        .filter(|cut| matches!(cut.key.reader, GrappleReader::AttackAdmission { .. }));
    let admission = admissions
        .next()
        .ok_or_else(|| invalid("attack admission absent"))?;
    if admissions.next().is_some()
        || admission.source_attack.is_some()
        || admission.key.work != work_key(state, &root.work)?
        || admission.key.reader
            != (GrappleReader::AttackAdmission {
                attack: r.origin.id,
            })
        || admission.issued_by != r.origin
    {
        return Err(invalid("attack admission identity differs"));
    }
    let proof_ids: Vec<_> = c.proofs.iter().map(|g| g.declaration.id).collect();
    if proof_ids != admission.grips {
        return Err(invalid("attack proof closure differs from admission"));
    }
    let rules = state.rules.as_ref().ok_or(RulesError::Uninitialized)?;
    let issued: Vec<_> = rules
        .rolls
        .iter()
        .map(|raw| &raw.purpose)
        .chain(rules.pending.iter().map(|pending| &pending.purpose))
        .filter_map(|purpose| match purpose {
            PendingPurpose::TacticalResolution { key, .. }
                if key.origin == r.origin.id
                    && matches!(
                        key.role,
                        TacticalRollRole::Attack | TacticalRollRole::AttackDamage
                    ) =>
            {
                Some(*key)
            }
            _ => None,
        })
        .collect();
    for key in &issued {
        if c.cuts
            .iter()
            .filter(|cut| cut.key.reader == (GrappleReader::RequestIssue { roll: *key }))
            .count()
            != 1
        {
            return Err(invalid("actual attack request lacks its unique issue cut"));
        }
    }
    if let Some(attack) = &r.attack {
        if attack.origin != r.origin {
            return Err(invalid("attack origin differs"));
        }
        let mut all: Vec<_> = live(state).cloned().collect();
        for proof in &c.proofs {
            if !all.iter().any(|g| g.declaration.id == proof.declaration.id) {
                all.push(proof.clone());
            }
        }
        all.retain(|g| relevant(g, attack.actor, attack.target, physical(attack)));
        all.sort_by_key(|g| g.declaration.id.0);
        let ids: Vec<_> = all.iter().map(|g| g.declaration.id).collect();
        if admission.grips != ids || c.proofs != all {
            return Err(invalid(
                "attack admission membership is incomplete or unrelated",
            ));
        }
    }
    for cut in &c.cuts {
        if cut == admission {
            continue;
        }
        let GrappleReader::RequestIssue { roll } = cut.key.reader else {
            return Err(invalid("unsupported attack reader"));
        };
        if issued.iter().filter(|key| **key == roll).count() != 1 {
            return Err(invalid(
                "request cut lacks its actual pending or recorded request",
            ));
        }
        let trace = r
            .work_trace
            .as_ref()
            .ok_or_else(|| invalid("attack trace absent"))?;
        let node = trace
            .nodes
            .iter()
            .find(|n| n.work.occurrence == cut.key.work.occurrence)
            .ok_or_else(|| invalid("request work absent"))?;
        let kind = match roll.role {
            TacticalRollRole::Attack => TacticalWorkKind::AttackRoll,
            TacticalRollRole::AttackDamage => TacticalWorkKind::AttackDamage,
            _ => return Err(invalid("unsupported attack raw role")),
        };
        let subject_matches = if let Some(attack) = &r.attack {
            roll.subject == attack.target
        } else {
            state.rules.as_ref().is_some_and(|rules| rules.rolls.iter().any(|raw|
                raw.request.roller == Some(r.turn_actor)
                    && matches!(raw.purpose, PendingPurpose::TacticalResolution { key, .. } if key == roll)))
        };
        if roll.origin != r.origin.id
            || !subject_matches
            || roll.occurrence != node.work.occurrence
            || node.work.kind != kind
            || cut.source_attack != Some(admission.key)
            || cut.grips != admission.grips
            || !in_lineage(state, node.work.occurrence, root.work.occurrence)?
            || cut.issued_by.expected_event_sequence < admission.issued_by.expected_event_sequence
            || cut.issued_by.expected_event_sequence > state.applied_event_sequence
        {
            return Err(invalid(
                "attack request cut differs from its source lineage",
            ));
        }
        validate_equipment_change_origin(state, &cut.issued_by, r.turn_actor)
            .map_err(|e| invalid(&e))?;
    }
    Ok(())
}

pub(in crate::tactical) fn paused_parent(
    read: &ReadContext<'_>,
    kind: TacticalWorkKind,
) -> Result<Option<TacticalWorkItem>, RulesError> {
    let state = read.state();
    let r = resolution(state)?;
    let attack = r
        .attack
        .as_ref()
        .ok_or_else(|| invalid("paused attack absent"))?;
    if read.attack_retained(attack)?.is_none() {
        return Ok(None);
    }
    let expected = match attack.stage {
        TacticalAttackStage::KnockoutChoice => TacticalWorkKind::AttackDamage,
        TacticalAttackStage::MasteryChoice => TacticalWorkKind::FinishAttack,
        _ => return Err(invalid("no supported paused completion")),
    };
    if kind != expected || r.pending.is_some() || r.failed_save.is_some() {
        return Err(invalid("paused completion source differs"));
    }
    let root = attack_root(state)?;
    let trace = r
        .work_trace
        .as_ref()
        .ok_or_else(|| invalid("paused trace absent"))?;
    let mut nodes = Vec::new();
    for node in trace.nodes.iter().filter(|n| n.work.kind == kind) {
        if !crate::table::grapple_enabled(state)
            || in_lineage(state, node.work.occurrence, root.work.occurrence)?
        {
            nodes.push(node);
        }
    }
    if nodes.len() != 1
        || !in_lineage(state, nodes[0].work.occurrence, root.work.occurrence)?
        || r.frames.iter().flatten().any(|w| w == &nodes[0].work)
    {
        return Err(invalid(
            "paused completion has no unique retired source node",
        ));
    }
    if kind == TacticalWorkKind::AttackDamage {
        match attack.damage_roll {
            Some(id)
                if state.rules.as_ref().is_some_and(|rules| {
                    rules.rolls.iter().any(|raw| {
                        raw.request.id == id
                            && matches!(raw.purpose, PendingPurpose::TacticalResolution { key, .. }
                if key.origin == attack.origin.id && key.role == TacticalRollRole::AttackDamage
                    && key.occurrence == nodes[0].work.occurrence && key.subject == attack.target)
                    })
                }) => {}
            None if attack.damage.iter().all(|c| c.dice.is_empty()) => {}
            _ => {
                return Err(invalid(
                    "paused damage parent lacks its actual raw or fixed source",
                ));
            }
        }
    } else if attack.outcome != Some(WeaponAttackOutcome::Miss)
        || attack.attack_roll.is_none()
        || attack.automatic_miss
    {
        return Err(invalid("mastery pause lacks its actual rolled miss"));
    }
    Ok(Some(nodes[0].work.clone()))
}
