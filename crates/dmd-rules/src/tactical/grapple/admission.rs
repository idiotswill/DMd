use super::*;
use crate::tactical_hands::EffectiveHands;

pub(super) struct AttemptPlan {
    pub attempt: TacticalGrappleAttempt,
    pub loadout: ActorEquipmentLoadout,
    pub budget: TacticalTurnBudget,
    pub new_action: bool,
}

/// Constructed only inside immediate admission reconstruction. Neither this
/// borrowed proof nor its excluded hand view escapes that call.
pub(crate) struct AuthenticatedAttemptAdmission<'a> {
    state: &'a CampaignState,
    attempt: &'a TacticalGrappleAttempt,
}
impl<'a> AuthenticatedAttemptAdmission<'a> {
    pub(crate) fn state(&self) -> &'a CampaignState {
        self.state
    }
    pub(crate) fn attempt(&self) -> &'a TacticalGrappleAttempt {
        self.attempt
    }
}

pub(super) fn loadout(
    state: &CampaignState,
    actor: EntityId,
) -> Result<&ActorEquipmentLoadout, RulesError> {
    state
        .rules
        .as_ref()
        .and_then(|r| r.tactical_inventory.as_ref())
        .and_then(|i| i.loadout(actor))
        .ok_or_else(|| invalid("Grapple requires actual physical equipment context"))
}

pub(super) fn supported_context(state: &CampaignState) -> Result<(), RulesError> {
    require_execution(state)?;
    if let Some(r) = &flow(state)?.resolution
        && (r.grapple.is_none()
            || r.attack.is_some()
            || r.shove.is_some()
            || r.hit_review.is_some()
            || r.movement.is_some()
            || !r.casts.is_empty()
            || !r.missiles.is_empty()
            || !r.falls.is_empty()
            || !r.areas.is_empty()
            || r.legendary_window.is_some()
            || r.frames.iter().flatten().any(|w| !is_work(&w.kind))
            || r.frames.iter().any(|f| f.len() > 1)
            || r.pending.as_ref().is_some_and(|p| !is_work(&p.work.kind))
            || r.failed_save
                .as_ref()
                .is_some_and(|p| !is_work(&p.pending.work.kind))
            || r.grapple
                .as_ref()
                .is_some_and(|g| !g.cuts.is_empty() || !g.opportunity_refreshes.is_empty()))
    {
        return Err(prerequisite(
            "This guarded Grapple core does not yet support the retained temporal consumer.",
        ));
    }
    if let Some(live) = state
        .rules
        .as_ref()
        .and_then(|r| r.tactical_grapples.as_ref())
    {
        for grip in &live.active {
            let rules = state.rules.as_ref().ok_or(RulesError::Uninitialized)?;
            if !crate::tactical_conditions::can_act(rules, grip.declaration.grappler)?
                || rules
                    .entities
                    .get(&grip.declaration.grappler)
                    .is_none_or(|e| e.death.dead)
            {
                return Err(prerequisite(
                    "Holder breaks require the guarded lifecycle follow-up.",
                ));
            }
            deferred_projection(state, grip.declaration.target)?;
            let e = encounter(state)?;
            let a = e
                .participant(grip.declaration.grappler)
                .ok_or_else(|| invalid("Grip holder absent"))?;
            let t = e
                .participant(grip.declaration.target)
                .ok_or_else(|| invalid("Grip target absent"))?;
            if crate::spatial::participant_distance(a, t).map_err(|e| invalid(&e.to_string()))?
                > grip.declaration.range
            {
                return Err(prerequisite(
                    "Range breaks require the guarded lifecycle follow-up.",
                ));
            }
        }
    }
    Ok(())
}

pub(super) fn deferred_projection(
    state: &CampaignState,
    target: EntityId,
) -> Result<(), RulesError> {
    if flow(state)?.dodges.iter().any(|d| d.actor == target) {
        return Err(prerequisite(
            "Grapple ending Dodge requires the guarded condition lifecycle.",
        ));
    }
    let e = encounter(state)?;
    let target = e
        .participant(target)
        .ok_or_else(|| invalid("Grapple target absent"))?;
    if target.movement.fly.is_some()
        && !target.movement.hover
        && crate::spatial::fall_destination(e, target.entity_id)
            .map_err(|e| invalid(&e.to_string()))?
            .is_some()
    {
        return Err(prerequisite(
            "Grapple flight loss requires the guarded fall lifecycle.",
        ));
    }
    Ok(())
}

pub(super) fn plan_attempt(
    state: &CampaignState,
    meta: &CommandMeta,
    target: EntityId,
    hand: Hand,
    before_change: Option<AttackEquipmentOperation>,
    pack: &RulesPack,
) -> Result<AttemptPlan, RulesError> {
    supported_context(state)?;
    if flow(state)?.phase != TacticalPhase::Active || flow(state)?.resolution.is_some() {
        return Err(RulesError::Pending);
    }
    let actor = active(state)?;
    super::super::shove::authorize_owner(state, meta, actor)?;
    if actor == target {
        return Err(prerequisite("Grapple needs a supported opposing target."));
    }
    super::super::attacks::admit_body_target(state, actor, target)?;
    super::super::shove::opposition(state, actor, target)?;
    super::super::falling::require_settled_before_action(state)?;
    let rules = state.rules.as_ref().ok_or(RulesError::Uninitialized)?;
    if !crate::tactical_conditions::can_act(rules, actor)?
        || !crate::tactical_conditions::may_harm(rules, actor, target)
    {
        return Err(prerequisite(
            "The actor cannot take this harmful body action.",
        ));
    }
    if rules
        .tactical_creatures
        .as_ref()
        .and_then(|c| c.runtime(actor))
        .is_some_and(|r| r.routine.is_some())
    {
        return Err(prerequisite(
            "Ordinary Grapple cannot replace a source routine step.",
        ));
    }
    let e = encounter(state)?;
    let a = e
        .participant(actor)
        .ok_or_else(|| invalid("Grappler absent"))?;
    let t = e
        .participant(target)
        .ok_or_else(|| invalid("Grapple target absent"))?;
    if crate::spatial::participant_distance(a, t).map_err(|e| invalid(&e.to_string()))? > 10
        || t.size.rank() > a.size.rank() + 1
    {
        return Err(prerequisite(
            "The target is outside ordinary Grapple reach or size.",
        ));
    }
    super::super::shove::cover_bonus(state, actor, target)?;
    deferred_projection(state, target)?;
    let anatomy = crate::tactical_grapple_sources::ordinary_grapple_anatomy(state, actor, pack)?
        .ok_or_else(|| prerequisite("This source has no supported ordinary hand anatomy."))?;
    let target_source = super::super::shove::source_pin(state, target)?;
    let mut budget = flow(state)?.budget.clone();
    let new_action = budget.attacks_remaining == 0;
    let window = if new_action {
        WeaponActionWindow {
            id: meta.id,
            kind: WeaponActionKind::AttackAction,
        }
    } else {
        budget
            .attack_window
            .ok_or_else(|| invalid("Grapple Attack window absent"))?
    };
    if window.kind != WeaponActionKind::AttackAction {
        return Err(prerequisite(
            "Grapple needs an ordinary Attack opportunity.",
        ));
    }
    budget.attack_window = Some(window);
    let before = loadout(state, actor)?.clone();
    crate::tactical_inventory::validate_loadout(state, &before)
        .map_err(|e| invalid(&e.to_string()))?;
    let hands = EffectiveHands::current(state, rules, actor)?;
    hands.validate_loadout(&before.hands)?;
    let mut after = before.clone();
    if let Some(operation) = before_change {
        crate::tactical_weapons::apply_attack_equipment_operation(
            state,
            actor,
            window,
            definitions()?,
            &mut after.hands,
            operation,
            &hands,
        )
        .map_err(|e| prerequisite(&e.to_string()))?;
        after.command = meta.clone();
    }
    if !hands.is_free(&after.hands, hand) {
        return Err(prerequisite("Choose an effective free ordinary hand."));
    }
    let declaration = TacticalGrappleDeclaration {
        id: GrappleId::from_declaration(meta.id, actor, target, hand),
        origin: meta.clone(),
        grappler: actor,
        target,
        hand,
        window,
        anatomy,
        target_source,
        grappler_from: a.position,
        target_from: t.position,
        range: 10,
        escape_dc: super::super::shove::difficulty(state, actor)?,
    };
    Ok(AttemptPlan {
        attempt: TacticalGrappleAttempt {
            declaration,
            equipment: GrappleEquipmentAdmission {
                equipment_before: before,
                before_change,
                after: None,
            },
            stage: TacticalGrappleAttemptStage::Queued,
            selected: None,
            save: None,
            outcome: None,
        },
        loadout: after,
        budget,
        new_action,
    })
}

/// The current guarded core permits no source/geometry changes while a grip is
/// retained. This is not authentication of historical temporal cuts.
pub(super) fn validate_declaration_source(
    state: &CampaignState,
    d: &TacticalGrappleDeclaration,
) -> Result<(), RulesError> {
    d.validate_shape().map_err(|e| invalid(&e))?;
    let pack = RulesPack::from_json(include_str!("../../../../../content/srd-5.2.1/kernel.json"))?;
    if d.origin.campaign_id != state.campaign_id()
        || crate::tactical_grapple_sources::ordinary_grapple_anatomy(state, d.grappler, &pack)?
            .as_ref()
            != Some(&d.anatomy)
        || super::super::shove::source_pin(state, d.target)? != d.target_source
        || super::super::shove::difficulty(state, d.grappler)? != d.escape_dc
    {
        return Err(invalid(
            "Grapple declaration differs from its actual source",
        ));
    }
    Ok(())
}

/// Source/payment authentication shared by provisional and completed records.
/// Completed history never requests a current-free-hand exemption.
pub(super) fn validate_attempt_source(state: &CampaignState) -> Result<(), RulesError> {
    let a = attempt(state)?;
    let d = &a.declaration;
    validate_declaration_source(state, d)?;
    let r = resolution(state)?;
    let rules = state.rules.as_ref().ok_or(RulesError::Uninitialized)?;
    if d.origin != r.origin
        || d.grappler != r.turn_actor
        || d.origin.campaign_id != state.campaign_id()
        || d.window
            != flow(state)?
                .budget
                .attack_window
                .ok_or_else(|| invalid("Grapple budget window absent"))?
        || !rules.timing.as_ref().is_some_and(|t| t.action_spent)
        || a.equipment.equipment_before.actor != d.grappler
    {
        return Err(invalid("Grapple paid admission/source differs"));
    }
    let trace = r
        .work_trace
        .as_ref()
        .ok_or_else(|| invalid("Grapple admission lacks work ancestry"))?;
    if trace.nodes.first().is_none_or(|n| {
        n.parent.is_some()
            || n.work
                != (TacticalWorkItem {
                    occurrence: 0,
                    kind: TacticalWorkKind::BeginGrapple { grip: d.id },
                })
    }) || trace.nodes.iter().filter(|n| n.parent.is_none()).count() != 1
    {
        return Err(invalid(
            "Grapple admission has no exact original Begin work",
        ));
    }
    if let Some(save) = &a.save
        && !trace.nodes.iter().any(|n| {
            n.parent == Some(0)
                && n.work
                    == (TacticalWorkItem {
                        occurrence: save.key.occurrence,
                        kind: TacticalWorkKind::GrappleSave { grip: d.id },
                    })
        })
    {
        return Err(invalid("Grapple admission has no exact save child"));
    }
    validate_equipment_change_origin(state, &d.origin, d.grappler).map_err(|e| invalid(&e))?;
    super::super::shove::authorize_owner(state, &d.origin, d.grappler)?;
    super::super::shove::opposition(state, d.grappler, d.target)?;
    let e = encounter(state)?;
    let from = e
        .participant(d.grappler)
        .ok_or_else(|| invalid("Grappler absent"))?;
    let to = e
        .participant(d.target)
        .ok_or_else(|| invalid("Grapple target absent"))?;
    if from.position != d.grappler_from
        || to.position != d.target_from
        || to.size.rank() > from.size.rank() + 1
        || crate::spatial::participant_distance(from, to).map_err(|e| invalid(&e.to_string()))?
            > d.range
    {
        return Err(invalid("Core Grapple retained geometry differs"));
    }
    // Later history readers will authenticate moved/retired admission separately.
    Ok(())
}

pub(super) fn validate_attempt_admission(state: &CampaignState) -> Result<(), RulesError> {
    validate_attempt_source(state)?;
    let a = attempt(state)?;
    if a.reservation().is_none() {
        return Err(invalid(
            "Only a provisional Attempt can exclude its reservation",
        ));
    }
    let proof = AuthenticatedAttemptAdmission { state, attempt: a };
    EffectiveHands::validate_attempt_equipment(&proof)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn borrowed_exemption_rejects_foreign_attachment_and_revalidates_mutated_source() {
        let mut f = super::super::tests::Fixture::new();
        f.begin(Hand::Left, None);
        let actual = attempt(&f.state).unwrap();
        let foreign = actual.clone();
        let token = AuthenticatedAttemptAdmission {
            state: &f.state,
            attempt: &foreign,
        };
        assert!(EffectiveHands::validate_attempt_equipment(&token).is_err());
        let clone = f.state.clone();
        let token = AuthenticatedAttemptAdmission {
            state: &clone,
            attempt: actual,
        };
        assert!(EffectiveHands::validate_attempt_equipment(&token).is_err());
        validate_attempt_admission(&f.state).unwrap();
        // No excluded view is returned; each later call must reconstruct against
        // the new attached image even at the same command sequence.
        let dagger = f
            .state
            .items
            .values()
            .find(|i| i.custody == Custody::Entity(f.human) && i.definition_id == "dagger")
            .unwrap()
            .id;
        let mut changed = f.state.clone();
        changed.items.get_mut(&dagger).unwrap().state = ItemState::Damaged;
        assert!(validate_attempt_admission(&changed).is_err());
        let mut changed = f.state.clone();
        attempt_mut(&mut changed)
            .unwrap()
            .declaration
            .target_source
            .as_mut()
            .unwrap()
            .definition_fingerprint = "0000000000000000".into();
        assert!(validate_attempt_admission(&changed).is_err());
        let mut changed = f.state.clone();
        attempt_mut(&mut changed).unwrap().declaration.window.id = CommandId::new();
        assert!(validate_attempt_admission(&changed).is_err());
        validate_attempt_admission(&f.state).unwrap();
    }
}
