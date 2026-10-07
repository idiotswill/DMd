use super::*;
use std::collections::HashSet;

fn require(condition: bool, message: &str) -> Result<(), String> {
    if condition {
        Ok(())
    } else {
        Err(message.into())
    }
}

fn meta(value: &CommandMeta) -> Result<(), String> {
    require(
        !value.id.0.is_nil() && !value.campaign_id.0.is_nil(),
        "nil grapple command identity",
    )
}

fn pin(value: &CreatureSourcePin) -> Result<(), String> {
    require(
        !value.ruleset_id.is_empty()
            && !value.ruleset_version.is_empty()
            && !value.definition_id.is_empty()
            && value.definition_fingerprint.len() == 16
            && value
                .definition_fingerprint
                .bytes()
                .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase()),
        "invalid grapple source pin shape",
    )
}

fn not_before(later: &CommandMeta, earlier: &CommandMeta) -> Result<(), String> {
    meta(later)?;
    meta(earlier)?;
    require(
        later.campaign_id == earlier.campaign_id
            && (later.expected_event_sequence > earlier.expected_event_sequence
                || later == earlier),
        "grapple command chronology differs",
    )
}

fn ids(values: &[GrappleId]) -> Result<(), String> {
    require(
        !values.is_empty()
            && values.iter().all(|id| !id.0.is_nil())
            && values.windows(2).all(|pair| pair[0].0 < pair[1].0),
        "grapple references must be nonempty, sorted and unique",
    )
}

impl TacticalGrappleDeclaration {
    pub fn validate_shape(&self) -> Result<(), String> {
        meta(&self.origin)?;
        require(
            !self.grappler.0.is_nil()
                && !self.target.0.is_nil()
                && self.grappler != self.target
                && self.id
                    == GrappleId::from_declaration(
                        self.origin.id,
                        self.grappler,
                        self.target,
                        self.hand,
                    )
                && !self.window.id.0.is_nil()
                && self.window.kind == WeaponActionKind::AttackAction
                && self.range == 10,
            "invalid ordinary grapple declaration identity/window/range",
        )?;
        match &self.anatomy {
            GrappleAnatomyProof::HumanCreationV1 { character } => {
                require(!character.0.is_nil(), "nil grapple character source")?;
            }
            GrappleAnatomyProof::Creature { source, .. } => pin(source)?,
        }
        if let Some(source) = &self.target_source {
            pin(source)?;
        }
        // Ability/PB/DC, anatomy, admission geometry and authority are source/rules
        // checks. Structural validity is never permission to execute this record.
        Ok(())
    }
}

impl TacticalGrappleSave {
    pub fn validate_shape(&self, declaration: &TacticalGrappleDeclaration) -> Result<(), String> {
        self.validate_with_interim(declaration, false)
    }
    fn validate_with_interim(
        &self,
        declaration: &TacticalGrappleDeclaration,
        interim: bool,
    ) -> Result<(), String> {
        not_before(&self.chosen_by, &declaration.origin)?;
        require(
            self.key.origin == declaration.origin.id
                && self.key.subject == declaration.target
                && self.key.role == TacticalRollRole::GrappleSave
                && self
                    .chosen_by
                    .actor
                    .is_none_or(|actor| actor == AgentRef::Entity(declaration.target)),
            "grapple save key differs from its paid source/target",
        )?;
        if let Some(request) = &self.request {
            require(
                request.id == self.key.request_id()
                    && request.roller == Some(declaration.target)
                    && request.dice
                        == [DieSpec {
                            count: 1,
                            sides: 20,
                        }],
                "grapple save request identity or die shape differs",
            )?;
        }
        let Some(proof) = &self.proof else {
            return require(
                self.request.is_some() || interim,
                "unresolved save has no physical request",
            );
        };
        require(
            proof.evidence.key() == self.key,
            "grapple final save key differs",
        )?;
        not_before(proof.evidence.resolved_by(), &self.chosen_by)?;
        match &proof.evidence {
            GrappleSaveEvidence::Physical { .. } => {
                require(
                    self.request.is_some(),
                    "physical save proof lacks its request",
                )?;
            }
            GrappleSaveEvidence::Decision(decision) => {
                require(
                    decision.issued_by == self.chosen_by,
                    "save decision issue differs",
                )?;
                require(
                    match decision.failure {
                        TacticalSaveFailure::Automatic => self.request.is_none(),
                        TacticalSaveFailure::Voluntary => self.request.is_some(),
                    },
                    "save decision does not preserve automatic/voluntary request shape",
                )?;
                require(
                    !proof.final_success
                        || proof.legendary.as_ref().is_some_and(|r| r.use_resistance),
                    "a no-die failure needs a genuine final override to succeed",
                )?;
            }
        }
        if let Some(legendary) = &proof.legendary {
            not_before(&legendary.chosen_by, proof.evidence.resolved_by())?;
            require(
                proof.finalized_by == legendary.chosen_by
                    && proof.final_success == legendary.use_resistance,
                "legendary save decision/finality differs",
            )?;
        } else {
            require(
                proof.finalized_by == *proof.evidence.resolved_by(),
                "save finalizer differs",
            )?;
        }
        Ok(())
    }
}

impl TacticalGrip {
    pub fn validate_shape(&self) -> Result<(), String> {
        self.declaration.validate_shape()?;
        self.save.validate_shape(&self.declaration)?;
        let proof = self
            .save
            .proof
            .as_ref()
            .ok_or("established grip lacks a final save")?;
        require(
            !proof.final_success,
            "successful save cannot establish a grip",
        )?;
        not_before(&self.established_by, &proof.finalized_by)?;
        require(
            !self.work.resolution.0.is_nil(),
            "grip has no establishing work identity",
        )
        // Its old work container may have retired. Source and original replay must
        // authenticate this reference; do not require today's resolution to own it.
    }
}

impl TacticalGrapples {
    pub fn validate_shape(&self) -> Result<(), String> {
        require(
            self.schema_version == TACTICAL_GRAPPLES_SCHEMA_VERSION,
            "unknown grapple schema",
        )?;
        ids(&self
            .active
            .iter()
            .map(|g| g.declaration.id)
            .collect::<Vec<_>>())?;
        let mut hands = HashSet::new();
        for grip in &self.active {
            grip.validate_shape()?;
            require(
                hands.insert((grip.declaration.grappler, grip.declaration.hand.index())),
                "one anatomical hand cannot hold two live grips",
            )?;
        }
        Ok(())
    }
    pub fn grip(&self, id: GrappleId) -> Option<&TacticalGrip> {
        self.active.iter().find(|g| g.declaration.id == id)
    }
}

fn node(
    resolution: &TacticalResolution,
    key: TacticalWorkKey,
) -> Result<&TacticalWorkNode, String> {
    require(
        key.resolution == resolution.origin.id,
        "work key belongs to another resolution",
    )?;
    let trace = resolution
        .work_trace
        .as_ref()
        .ok_or("grapple consumer has no work trace")?;
    let mut found = trace
        .nodes
        .iter()
        .filter(|n| n.work.occurrence == key.occurrence);
    let value = found.next().ok_or("grapple work occurrence is absent")?;
    require(found.next().is_none(), "duplicate grapple work occurrence")?;
    Ok(value)
}

/// Strict ancestry is necessary for a same-command material consequence. Merely
/// comparing command sequences or allocated occurrence numbers is insufficient.
fn descendant(
    resolution: &TacticalResolution,
    child: TacticalWorkKey,
    parent: TacticalWorkKey,
) -> Result<bool, String> {
    node(resolution, parent)?;
    let mut at = node(resolution, child)?.parent;
    let mut seen = HashSet::new();
    while let Some(occurrence) = at {
        require(seen.insert(occurrence), "cyclic grapple work ancestry")?;
        if occurrence == parent.occurrence {
            return Ok(true);
        }
        at = node(
            resolution,
            TacticalWorkKey {
                resolution: child.resolution,
                occurrence,
            },
        )?
        .parent;
    }
    Ok(false)
}

#[derive(Clone, Copy)]
enum ReadShape {
    Legacy,
    Activated,
}

fn admitted_raw_origin(
    resolution: &TacticalResolution,
    source: GrappleCutKey,
    roll: TacticalRollKey,
    reads: ReadShape,
) -> Result<CommandId, String> {
    let GrappleReader::AttackAdmission { attack } = source.reader else {
        return Err("raw source is not an attack admission".into());
    };
    if matches!(reads, ReadShape::Legacy) {
        return Ok(attack);
    }
    let root = node(resolution, source.work)?;
    require(
        root.work.kind == TacticalWorkKind::AttackRoll,
        "admission has no AttackRoll",
    )?;
    let mut current = root;
    loop {
        if let TacticalWorkKind::SpellProgram { cast, at } = current.work.kind {
            let mut records = resolution
                .casts
                .iter()
                .chain(resolution.grapple.iter().flat_map(|context| {
                    context
                        .completed_casts
                        .iter()
                        .map(|receipt| receipt.record.as_ref())
                }))
                .filter(|record| record.cast.plan.occurrence == cast);
            let record = records.next().ok_or("raw source cast absent")?;
            require(records.next().is_none(), "raw source cast ambiguous")?;
            require(
                record
                    .targets
                    .get(usize::from(at.target))
                    .is_some_and(|target| target.actor == roll.subject),
                "raw source target differs from its spell occurrence",
            )?;
            return Ok(record.cast.plan.origin.id);
        }
        let Some(parent) = current.parent else {
            return Ok(attack);
        };
        require(
            parent < current.work.occurrence,
            "raw source ancestry is not causal",
        )?;
        current = node(
            resolution,
            TacticalWorkKey {
                resolution: source.work.resolution,
                occurrence: parent,
            },
        )?;
    }
}

impl TacticalGrappleResolution {
    /// Local shape only. Live source readers and original accepted replay must
    /// also prove membership completeness, source program, authority and causes.
    pub fn validate_shape(
        &self,
        resolution: &TacticalResolution,
        live: Option<&TacticalGrapples>,
    ) -> Result<(), String> {
        self.validate_read_shape(resolution, live, ReadShape::Legacy)
    }

    /// The full-state caller has validated the explicit table marker. This
    /// permits a recorded empty mechanical read, never execution authority.
    pub(crate) fn validate_activated_shape(
        &self,
        resolution: &TacticalResolution,
        live: Option<&TacticalGrapples>,
    ) -> Result<(), String> {
        self.validate_read_shape(resolution, live, ReadShape::Activated)
    }

    fn validate_read_shape(
        &self,
        resolution: &TacticalResolution,
        live: Option<&TacticalGrapples>,
        reads: ReadShape,
    ) -> Result<(), String> {
        require(!self.is_empty(), "empty grapple resolution attachment")?;
        require(
            (matches!(reads, ReadShape::Activated) || self.completed_casts.is_empty())
                && self.completed_casts.len() <= MAX_TACTICAL_CASTS,
            "completed cast evidence exceeds its activated scope or capacity",
        )?;
        for (index, receipt) in self.completed_casts.iter().enumerate() {
            let record = &receipt.record;
            let cast = record.cast.plan.occurrence;
            require(
                !resolution
                    .casts
                    .iter()
                    .any(|live| live.cast.plan.occurrence == cast)
                    && !self.completed_casts[..index]
                        .iter()
                        .any(|old| old.record.cast.plan.occurrence == cast)
                    && node(resolution, receipt.work)?.work.kind
                        == (TacticalWorkKind::FinishSpell { cast })
                    && !record.targets.is_empty()
                    && record.completed.len() == record.targets.len(),
                "completed cast evidence lacks its unique completed source/work",
            )?;
            not_before(&record.cast.plan.origin, &resolution.origin)?;
            not_before(&receipt.finished_by, &record.cast.last_operation)?;
            not_before(&receipt.finished_by, &record.cast.plan.origin)?;
        }
        let mut proof_ids = HashSet::new();
        for proof in &self.proofs {
            proof.validate_shape()?;
            require(
                proof.declaration.origin.campaign_id == resolution.origin.campaign_id,
                "retained grip belongs to another campaign",
            )?;
            require(
                proof_ids.insert(proof.declaration.id),
                "duplicate retained grip proof",
            )?;
            if let Some(active) = live.and_then(|g| g.grip(proof.declaration.id)) {
                require(active == proof, "live grip and immutable proof differ")?;
            }
        }
        let proof = |id| {
            self.proofs
                .iter()
                .find(|p| p.declaration.id == id)
                .ok_or_else(|| "unknown retained grip reference".to_string())
        };
        for (index, cut) in self.cuts.iter().enumerate() {
            node(resolution, cut.key.work)?;
            not_before(&cut.issued_by, &resolution.origin)?;
            require(
                !self.cuts[..index].iter().any(|c| c.key == cut.key),
                "duplicate grapple read cut",
            )?;
            if !matches!(reads, ReadShape::Activated)
                || !cut.grips.is_empty()
                || matches!(cut.key.reader, GrappleReader::FlightLoss { .. })
            {
                ids(&cut.grips)?;
            }
            for id in &cut.grips {
                not_before(&cut.issued_by, &proof(*id)?.established_by)?;
            }
            match cut.key.reader {
                GrappleReader::AttackAdmission { attack } => {
                    require(!attack.0.is_nil(), "nil admitted attack cut")?
                }
                GrappleReader::OpportunityWindow {
                    attack,
                    window,
                    reactor,
                    mover,
                    ..
                } => {
                    require(
                        matches!(reads, ReadShape::Activated)
                            && !attack.0.is_nil()
                            && !window.0.is_nil()
                            && !reactor.0.is_nil()
                            && !mover.0.is_nil()
                            && reactor != mover
                            && cut.issued_by.id == attack
                            && cut.source_attack.is_none()
                            && node(resolution, cut.key.work)?.work.kind
                                == TacticalWorkKind::AttackRoll
                            && cut.grips.iter().all(|id| {
                                proof(*id).is_ok_and(|grip| grip.declaration.grappler == reactor)
                            }),
                        "opportunity window cut lacks its activated source",
                    )?;
                }
                GrappleReader::RequestIssue { roll } => require(
                    !roll.origin.0.is_nil() && !roll.subject.0.is_nil(),
                    "nil request cut source",
                )?,
                GrappleReader::FlightLoss { actor } => require(
                    cut.grips.iter().any(|id| {
                        self.proofs
                            .iter()
                            .any(|p| p.declaration.id == *id && p.declaration.target == actor)
                    }),
                    "flight-loss cut names another actor",
                )?,
            }
            if let Some(source) = cut.source_attack {
                let GrappleReader::RequestIssue { roll } = cut.key.reader else {
                    return Err("only a request can inherit an admitted attack cut".into());
                };
                let source_cut = self
                    .cuts
                    .iter()
                    .find(|c| c.key == source)
                    .ok_or("missing attack admission cut")?;
                require(
                    source != cut.key
                        && source_cut.source_attack.is_none()
                        && source_cut.grips == cut.grips
                        && admitted_raw_origin(resolution, source, roll, reads)? == roll.origin
                        && matches!(
                            roll.role,
                            TacticalRollRole::Attack | TacticalRollRole::AttackDamage
                        ),
                    "grapple cut inheritance is not a direct same-attack admission",
                )?;
                not_before(&cut.issued_by, &source_cut.issued_by)?;
                require(
                    source.work == cut.key.work
                        || descendant(resolution, cut.key.work, source.work)?,
                    "inherited attack request has unrelated ancestry",
                )?;
            }
        }
        let mut ended = HashSet::new();
        for end in &self.ends {
            let grip = proof(end.grip)?;
            require(ended.insert(end.grip), "grip ended twice")?;
            require(
                live.is_none_or(|g| g.grip(end.grip).is_none()),
                "ended grip is still live",
            )?;
            not_before(&end.caused_by, &grip.established_by)?;
            match &end.cause {
                GrappleEndCause::Released => require(
                    end.caused_by
                        .actor
                        .is_none_or(|actor| actor == AgentRef::Entity(grip.declaration.grappler)),
                    "release names another actor",
                )?,
                GrappleEndCause::Escaped { roll, .. } => require(
                    roll.role == TacticalRollRole::GrappleEscape
                        && roll.subject == grip.declaration.target
                        && !roll.origin.0.is_nil(),
                    "ending names another Escape check",
                )?,
                GrappleEndCause::OutOfRange { moved_actor, .. } => require(
                    *moved_actor == grip.declaration.grappler
                        || *moved_actor == grip.declaration.target,
                    "range ending names an unrelated body",
                )?,
                GrappleEndCause::Incapacitated { .. } | GrappleEndCause::Dead { .. } => (),
            }
            if let Some(work) = end.cause.work() {
                node(resolution, work)?;
                if end.caused_by == grip.established_by {
                    require(
                        descendant(resolution, work, grip.work)?,
                        "same-command ending lacks later causal child",
                    )?;
                }
            } else {
                require(
                    end.caused_by != grip.established_by,
                    "owner release cannot precede establishment",
                )?;
            }
            for cut in self
                .cuts
                .iter()
                .filter(|c| c.grips.contains(&end.grip) && c.source_attack.is_none())
            {
                not_before(&end.caused_by, &cut.issued_by)?;
                if end.caused_by == cut.issued_by {
                    let work = end
                        .cause
                        .work()
                        .ok_or("same-command cut ending has no producer")?;
                    require(
                        descendant(resolution, work, cut.key.work)?,
                        "ending is not after its live read cut",
                    )?;
                }
            }
        }
        for retained in &self.proofs {
            require(
                live.is_some_and(|g| g.grip(retained.declaration.id).is_some())
                    || ended.contains(&retained.declaration.id),
                "retained grip has neither live authority nor an ending",
            )?;
        }
        if let Some(activity) = &self.activity {
            match activity {
                GrappleActivity::Attempt(attempt) => {
                    attempt.declaration.validate_shape()?;
                    require(
                        attempt.declaration.origin.campaign_id == resolution.origin.campaign_id,
                        "attempt belongs to another campaign",
                    )?;
                    attempt
                        .equipment
                        .validate_shape(&attempt.declaration, resolution)?;
                    require(
                        (attempt.stage == TacticalGrappleAttemptStage::Complete
                            && (attempt.equipment.before_change.is_some()
                                != attempt.equipment.after.is_some()))
                            || (attempt.stage != TacticalGrappleAttemptStage::Complete
                                && attempt.equipment.after.is_none()),
                        "Grapple equipment decision/stage differs",
                    )?;
                    if let Some(save) = &attempt.save {
                        let interim = resolution.failed_save.as_ref().is_some_and(|failed| {
                            failed.pending.key == save.key
                                && failed.result.is_none()
                                && failed.pending.work.kind
                                    == TacticalWorkKind::GrappleSave {
                                        grip: attempt.declaration.id,
                                    }
                        }) || matches!(
                            attempt.outcome,
                            Some(GrappleAttemptOutcome::Withdrawn { .. })
                        );
                        save.validate_with_interim(&attempt.declaration, interim)?;
                    }
                    if let Some((_, actor, hand)) = attempt.reservation() {
                        require(
                            live.is_none_or(|g| {
                                !g.active.iter().any(|p| {
                                    p.declaration.grappler == actor && p.declaration.hand == hand
                                })
                            }),
                            "attempt collides with live hand reservation",
                        )?;
                    }
                    require(
                        matches!(
                            attempt.stage,
                            TacticalGrappleAttemptStage::AfterEquipment
                                | TacticalGrappleAttemptStage::Complete
                        ) == attempt.outcome.is_some(),
                        "grapple attempt outcome/stage differs",
                    )?;
                }
                GrappleActivity::Escape(escape) => {
                    let grip = proof(escape.grip)?;
                    not_before(&escape.origin, &resolution.origin)?;
                    require(
                        escape.actor == grip.declaration.target
                            && escape.difficulty == grip.declaration.escape_dc
                            && escape.key.role == TacticalRollRole::GrappleEscape
                            && escape.key.origin == escape.origin.id
                            && escape.key.subject == escape.actor,
                        "Escape source/check identity differs",
                    )?;
                    if let Some(request) = &escape.request {
                        require(
                            request.id == escape.key.request_id()
                                && request.roller == Some(escape.actor)
                                && request.dice
                                    == [DieSpec {
                                        count: 1,
                                        sides: 20,
                                    }],
                            "Escape request differs",
                        )?;
                    }
                    require(
                        (escape.stage == TacticalGrappleEscapeStage::Complete)
                            == escape.outcome.is_some(),
                        "Escape outcome/stage differs",
                    )?;
                }
            }
        }
        for (index, refresh) in self.opportunity_refreshes.iter().enumerate() {
            require(
                matches!(node(resolution, refresh.work)?.work.kind, TacticalWorkKind::MovementOpportunity { reactor } if reactor == refresh.reactor),
                "refresh names another opportunity work",
            )?;
            require(
                ended.contains(&refresh.ended_grip)
                    && !refresh.resulting.is_empty()
                    && refresh.previous != refresh.resulting,
                "invalid opportunity refresh cause/options",
            )?;
            require(
                proof(refresh.ended_grip)?.declaration.grappler == refresh.reactor,
                "hand refresh names another holder",
            )?;
            for options in [&refresh.previous, &refresh.resulting] {
                require(
                    !options
                        .iter()
                        .enumerate()
                        .any(|(i, option)| options[..i].contains(option)),
                    "duplicate refreshed opportunity option",
                )?;
            }
            if let Some(previous) = self.opportunity_refreshes[..index]
                .iter()
                .rev()
                .find(|r| r.work == refresh.work)
            {
                require(
                    previous.resulting == refresh.previous
                        && previous.window_origin == refresh.window_origin
                        && previous.movement_origin == refresh.movement_origin
                        && previous.step == refresh.step,
                    "opportunity refresh chain changes its original window",
                )?;
            }
        }
        if let Some(movement) = &resolution.movement {
            if let Some(admission) = &movement.grapple_self_only {
                require(
                    admission.origin == movement.origin,
                    "self-only movement origin differs",
                )?;
                ids(&admission.grips)?;
                for id in &admission.grips {
                    require(
                        proof(*id)?.declaration.grappler == movement.actor,
                        "self-only move names another holder",
                    )?;
                }
            }
            if let Some(window) = &movement.opportunity
                && let Some(refresh) = self.opportunity_refreshes.iter().rev().find(|r| {
                    r.window_origin == window.origin
                        && r.reactor == window.reactor
                        && r.step == window.step_index
                })
            {
                require(
                    refresh.movement_origin == movement.origin
                        && refresh.resulting == window.options,
                    "retained window differs from final refresh",
                )?;
            }
        }
        for fall in &resolution.falls {
            if let TacticalFallCause::GrappleFlightLost {
                grip,
                consequence,
                work,
                cut,
            } = &fall.cause
            {
                let source = proof(*grip)?;
                let image = self
                    .cuts
                    .iter()
                    .find(|c| c.key == *cut)
                    .ok_or("grapple fall lost its read cut")?;
                require(
                    source.declaration.target == fall.actor
                        && source.work == *work
                        && source.established_by == *consequence
                        && image.grips.contains(grip)
                        && image.key.reader == (GrappleReader::FlightLoss { actor: fall.actor }),
                    "grapple fall source identity differs",
                )?;
            }
        }
        Ok(())
    }
}

impl GrappleEquipmentAdmission {
    fn validate_shape(
        &self,
        declaration: &TacticalGrappleDeclaration,
        resolution: &TacticalResolution,
    ) -> Result<(), String> {
        require(
            self.equipment_before.actor == declaration.grappler,
            "Grapple equipment belongs to another actor",
        )?;
        not_before(&declaration.origin, &self.equipment_before.command)?;
        require(
            self.before_change.is_none() || self.after.is_none(),
            "Grapple equipment allowance used twice",
        )?;
        if let Some(after) = &self.after {
            let (chosen_by, work) = match after {
                GrappleEquipmentDecision::Declined { chosen_by, work } => (chosen_by, *work),
                GrappleEquipmentDecision::Applied {
                    chosen_by,
                    work,
                    equipment_before,
                    ..
                } => {
                    require(
                        equipment_before.actor == declaration.grappler,
                        "after equipment belongs to another actor",
                    )?;
                    meta(&equipment_before.command)?;
                    (chosen_by, *work)
                }
            };
            not_before(chosen_by, &declaration.origin)?;
            require(
                node(resolution, work)?.work.kind
                    == TacticalWorkKind::GrappleAfterEquipment {
                        grip: declaration.id,
                    },
                "equipment decision names another work",
            )?;
        }
        Ok(())
    }
}
