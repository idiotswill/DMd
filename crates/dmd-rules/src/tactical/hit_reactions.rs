//! Source Shield decisions attached to the accepted hit in the existing cursor.
use super::turns::*;
use super::*;
use crate::tactical_spells::*;

fn review(state: &CampaignState) -> Result<&TacticalHitReview, RulesError> {
    resolution(state)?
        .hit_review
        .as_deref()
        .ok_or_else(|| prerequisite("no hit response is due"))
}
fn review_mut(state: &mut CampaignState) -> Result<&mut TacticalHitReview, RulesError> {
    resolution_mut(state)?
        .hit_review
        .as_deref_mut()
        .ok_or_else(|| prerequisite("no hit response is due"))
}
fn require_window(state: &CampaignState, window: TacticalWorkKey) -> Result<(), RulesError> {
    if window.resolution != resolution(state)?.origin.id
        || window.occurrence != review(state)?.work.occurrence
        || flow(state)?.version != TacticalExecutionVersion::ShieldHitV1.flow_version()
    {
        return Err(prerequisite("that hit response is no longer available"));
    }
    Ok(())
}
fn owner(state: &CampaignState, meta: &CommandMeta, actor: EntityId) -> Result<(), RulesError> {
    if controller(state, actor).is_some_and(|player| meta.issuer != CommandIssuer::Player(player)) {
        return Err(RulesError::Unauthorized);
    }
    authorize(state, meta, actor)
}

/// Pure private affordances. The application must restrict this query to an
/// audience's owned actor. Returned choices never bypass selected admission.
pub fn shield_choices(
    state: &CampaignState,
    actor: EntityId,
) -> Result<Vec<SpellCastChoice>, RulesError> {
    let rules = state.rules.as_ref().ok_or(RulesError::Uninitialized)?;
    let entity = rules
        .entities
        .get(&actor)
        .ok_or_else(|| invalid("response actor is absent"))?;
    let r = resolution(state)?;
    // Read-only planning identity, never stored, submitted or used as a receipt.
    // Actual selected admission rederives everything using the real command.
    let meta = CommandMeta {
        id: CommandId(uuid::Uuid::new_v5(&r.origin.id.0, actor.0.as_bytes())),
        campaign_id: state.campaign_id(),
        session_id: r.origin.session_id,
        issuer: controller(state, actor).map_or(CommandIssuer::Admin, CommandIssuer::Player),
        actor: Some(AgentRef::Entity(actor)),
        expected_event_sequence: state.applied_event_sequence,
    };
    let mut grants = Vec::new();
    if entity.prepared_spells.iter().any(|spell| spell == "shield") {
        if let Some(casting) = &entity.spellcasting {
            for level in 1..=9u8 {
                if casting.slots[usize::from(level - 1)] > 0 {
                    grants.push((
                        SpellGrantChoice::Prepared,
                        SpellResourceChoice::Slot { level },
                    ));
                }
            }
        }
    }
    if let Some(profile) = rules
        .tactical_creatures
        .as_ref()
        .and_then(|creatures| creatures.profile(actor))
    {
        let source = crate::tactical_creatures::source_for_profile(profile)
            .map_err(|error| invalid(&error.to_string()))?;
        for feature in &source.features {
            if let crate::tactical_definitions::MonsterFeature::Spellcasting { spells, .. } =
                &feature.feature
                && spells.iter().any(|spell| spell.spell_id == "shield")
            {
                grants.push((
                    SpellGrantChoice::CreatureFeature {
                        feature_id: feature.id.clone(),
                    },
                    SpellResourceChoice::SourceFeature,
                ));
            }
        }
    }
    let mut choices = Vec::new();
    for (grant, resource) in grants {
        let choice = SpellCastChoice {
            actor,
            spell_id: "shield".into(),
            grant,
            resource,
            material: SpellMaterialChoice::None,
            mode: SpellCastMode::Immediate,
        };
        if super::casting::shield_admission(state, &meta, &choice, r.next_occurrence).is_ok() {
            choices.push(choice);
        }
    }
    Ok(choices)
}

pub(super) fn open(
    state: &mut CampaignState,
    cause: &CommandMeta,
    work: &TacticalWorkItem,
) -> Result<(), RulesError> {
    let attack = resolution(state)?
        .attack
        .as_ref()
        .ok_or_else(|| invalid("hit lacks an attack"))?;
    if review(state).is_ok() || work.kind != TacticalWorkKind::AttackRoll {
        return Err(invalid("hit review is duplicated or has the wrong cause"));
    }
    let actor = attack.target;
    let attack_origin = attack.origin.id;
    let attack_roll = attack
        .attack_roll
        .ok_or_else(|| invalid("hit review lacks accepted dice"))?;
    let cover_bonus =
        attack.armor_class - crate::tactical_defenses::effective_armor_class(state, actor)?;
    // Every hit target acknowledges this coordination stage, even without an
    // eligible spell. Otherwise completion timing discloses private eligibility.
    let respondent = Some(TacticalShieldRespondent {
        actor,
        intent: None,
        declined_after_selection: None,
    });
    resolution_mut(state)?
        .attack
        .as_mut()
        .ok_or_else(|| invalid("hit disappeared"))?
        .stage = TacticalAttackStage::HitReview;
    resolution_mut(state)?.hit_review = Some(Box::new(TacticalHitReview {
        work: work.clone(),
        cause: cause.clone(),
        attack_origin,
        attack_roll,
        cover_bonus,
        stage: TacticalHitReviewStage::Collecting,
        delegated_by: None,
        order: None,
        respondent,
        selected_cast: None,
        completed_shield: None,
    }));
    push_frame(state, vec![TacticalWorkKind::ResumeHit { attack_origin }])
}

pub(super) fn waiting(state: &CampaignState) -> bool {
    state
        .encounter
        .as_ref()
        .and_then(|e| e.flow.as_ref())
        .and_then(|f| f.resolution.as_ref())
        .and_then(|r| r.hit_review.as_ref())
        .is_some_and(|review| {
            matches!(
                review.stage,
                TacticalHitReviewStage::Collecting | TacticalHitReviewStage::Selected
            )
        })
}

fn advance(state: &mut CampaignState, meta: &CommandMeta) -> Result<(), RulesError> {
    let current = review(state)?;
    if current.order.is_none()
        || current
            .respondent
            .as_ref()
            .is_some_and(|r| r.intent.is_none())
    {
        return Ok(());
    }
    let accepted = current
        .respondent
        .as_ref()
        .is_some_and(|r| r.intent.as_ref().is_some_and(|i| i.accepted));
    review_mut(state)?.stage = if accepted {
        TacticalHitReviewStage::Selected
    } else {
        TacticalHitReviewStage::Resolved
    };
    pump(state, meta)
}

pub(super) fn respond(
    state: &mut CampaignState,
    meta: &CommandMeta,
    window: TacticalWorkKey,
    accept: bool,
) -> Result<(), RulesError> {
    require_window(state, window)?;
    let current = review(state)?;
    let respondent = current
        .respondent
        .as_ref()
        .ok_or(RulesError::Unauthorized)?;
    owner(state, meta, respondent.actor)?;
    if current.stage != TacticalHitReviewStage::Collecting || respondent.intent.is_some() {
        return Err(prerequisite("that response was already decided"));
    }
    if accept && shield_choices(state, respondent.actor)?.is_empty() {
        return Err(prerequisite(
            "no source Shield response is currently available",
        ));
    }
    review_mut(state)?
        .respondent
        .as_mut()
        .ok_or(RulesError::Unauthorized)?
        .intent = Some(TacticalReactionIntent {
        origin: meta.clone(),
        accepted: accept,
    });
    advance(state, meta)
}

pub(super) fn delegate(
    state: &mut CampaignState,
    meta: &CommandMeta,
    window: TacticalWorkKey,
) -> Result<(), RulesError> {
    require_window(state, window)?;
    owner(state, meta, resolution(state)?.turn_actor)?;
    let current = review_mut(state)?;
    if current.stage != TacticalHitReviewStage::Collecting
        || current.order.is_some()
        || current.delegated_by.is_some()
    {
        return Err(prerequisite("ordering authority was already decided"));
    }
    current.delegated_by = Some(meta.clone());
    Ok(())
}

pub(super) fn order(
    state: &mut CampaignState,
    meta: &CommandMeta,
    window: TacticalWorkKey,
    instruction: &TacticalReactionOrdering,
) -> Result<(), RulesError> {
    require_window(state, window)?;
    let current = review(state)?;
    if current.stage != TacticalHitReviewStage::Collecting || current.order.is_some() {
        return Err(prerequisite(
            "the hit's ordering instruction is already fixed",
        ));
    }
    let actor = resolution(state)?.turn_actor;
    if current.delegated_by.is_some() {
        privileged(meta)?;
    } else {
        owner(state, meta, actor)?;
    }
    // Admission must not depend on the private accepted set, even if already known.
    validate_order(state, meta, instruction)?;
    review_mut(state)?.order = Some(TacticalReactionOrderDecision {
        origin: meta.clone(),
        instruction: instruction.clone(),
    });
    advance(state, meta)
}

pub(super) fn decline(
    state: &mut CampaignState,
    meta: &CommandMeta,
    window: TacticalWorkKey,
) -> Result<(), RulesError> {
    require_window(state, window)?;
    let current = review(state)?;
    let actor = current
        .respondent
        .as_ref()
        .ok_or(RulesError::Unauthorized)?
        .actor;
    owner(state, meta, actor)?;
    if current.stage != TacticalHitReviewStage::Selected {
        return Err(RulesError::Pending);
    }
    let current = review_mut(state)?;
    current
        .respondent
        .as_mut()
        .ok_or(RulesError::Unauthorized)?
        .declined_after_selection = Some(meta.clone());
    current.stage = TacticalHitReviewStage::Resolved;
    pump(state, meta)
}

pub(super) fn cast(
    state: &mut CampaignState,
    meta: &CommandMeta,
    window: TacticalWorkKey,
    choice: &SpellCastChoice,
) -> Result<(), RulesError> {
    require_window(state, window)?;
    let current = review(state)?;
    let actor = current
        .respondent
        .as_ref()
        .ok_or(RulesError::Unauthorized)?
        .actor;
    owner(state, meta, actor)?;
    if current.stage != TacticalHitReviewStage::Selected || choice.actor != actor {
        return Err(RulesError::Unauthorized);
    }
    let occurrence = resolution(state)?.next_occurrence;
    if occurrence >= 32_768 || resolution(state)?.casts.len() >= MAX_TACTICAL_CASTS {
        return Err(invalid("response casting capacity exceeded"));
    }
    let (record, source) = super::casting::shield_admission(state, meta, choice, occurrence)?;
    *state = apply_spell_casting_cost(state, actor, SpellCastingCost::Reaction)?;
    if let Some(source) = source {
        state
            .rules
            .as_mut()
            .ok_or(RulesError::Uninitialized)?
            .tactical_creatures = Some(source);
    }
    crate::kernel::interrupt_rest(
        state.rules.as_mut().ok_or(RulesError::Uninitialized)?,
        actor,
        state.clock.now,
    );
    resolution_mut(state)?.next_occurrence += 1;
    resolution_mut(state)?.casts.push(record);
    review_mut(state)?.selected_cast = Some(occurrence);
    review_mut(state)?.stage = TacticalHitReviewStage::Casting;
    let original = review(state)?.work.clone();
    let previous = super::work_trace::enter(state, &original)?;
    push_frame(
        state,
        vec![TacticalWorkKind::CommitShield { cast: occurrence }],
    )?;
    super::work_trace::leave(state, previous)?;
    pump(state, meta)
}

pub(super) fn finish_shield(
    state: &mut CampaignState,
    record: &TacticalCasting,
) -> Result<bool, RulesError> {
    let Some(current) = resolution_mut(state)?.hit_review.as_deref_mut() else {
        return Ok(false);
    };
    if current.selected_cast != Some(record.cast.plan.occurrence) {
        return Ok(false);
    }
    if current.stage != TacticalHitReviewStage::Casting || current.completed_shield.is_some() {
        return Err(invalid(
            "Shield completion differs from the selected hit response",
        ));
    }
    current.completed_shield = Some(Box::new(record.clone()));
    current.stage = TacticalHitReviewStage::Resolved;
    Ok(true)
}

/// Reconstruct the original defense without changing source, equipment, geometry
/// or condition inputs. Only this exact response's canonical Shield is excluded.
pub(super) fn original_defense(
    state: &CampaignState,
    attacker: EntityId,
    target: EntityId,
) -> Result<i32, RulesError> {
    let Some(r) = state
        .encounter
        .as_ref()
        .and_then(|e| e.flow.as_ref())
        .and_then(|f| f.resolution.as_ref())
    else {
        return crate::tactical_defenses::effective_armor_class(state, target);
    };
    let Some(hit) = r.hit_review.as_ref().filter(|hit| {
        r.attack.as_ref().is_some_and(|a| {
            a.actor == attacker && a.target == target && a.origin.id == hit.attack_origin
        })
    }) else {
        return crate::tactical_defenses::effective_armor_class(state, target);
    };
    let Some(record) = hit.completed_shield.as_deref().or_else(|| {
        hit.selected_cast.and_then(|cast| {
            r.casts
                .iter()
                .find(|record| record.cast.plan.occurrence == cast)
        })
    }) else {
        return crate::tactical_defenses::effective_armor_class(state, target);
    };
    validate_retained_spell(record)?;
    if record.cast.plan.choice.spell_id != "shield"
        || record.cast.plan.choice.actor != target
        || record.cast.plan.cost != SpellCastingCost::Reaction
        || record.cast.plan.origin.expected_event_sequence <= hit.cause.expected_event_sequence
        || Some(record.cast.plan.occurrence) != hit.selected_cast
        || record.targets.len() != 1
        || record.targets[0].actor != target
    {
        return Err(invalid(
            "hit defense exclusion lacks its selected source response",
        ));
    }
    // Execution rejects running a completed occurrence twice. Reconstruct its
    // immutable source on a private copy; never reopen the authoritative cast.
    let mut source = record.clone();
    if hit.completed_shield.is_some() {
        if source.completed.as_slice() != [SpellProgramOccurrence { node: 0, target: 0 }] {
            return Err(invalid(
                "completed Shield has a different occurrence partition",
            ));
        }
        source.completed.clear();
    }
    let mut effect = spell_defense_effect(
        state,
        &source,
        SpellProgramOccurrence { node: 0, target: 0 },
    )?
    .ok_or_else(|| invalid("source Shield has no defense"))?;
    let mut prior = effects(state)?.clone();
    if let Some(actual) = prior.effects.iter().find(|current| current.id == effect.id) {
        let stamp = actual
            .established_at
            .as_ref()
            .ok_or_else(|| invalid("Shield lacks installation provenance"))?;
        let last = prior
            .last_operation
            .as_ref()
            .ok_or_else(|| invalid("Shield lacks a lifecycle operation"))?;
        if stamp.command != record.cast.last_operation
            || stamp.command.expected_event_sequence > last.command.expected_event_sequence
            || (stamp.command.expected_event_sequence == last.command.expected_event_sequence
                && (stamp.command != last.command || stamp.step > last.step))
        {
            return Err(invalid(
                "Shield installation differs from its selected casting",
            ));
        }
        effect.established_at = Some(stamp.clone());
        if *actual != effect {
            return Err(invalid("Shield defense differs from its source program"));
        }
    } else if hit.completed_shield.is_some() {
        return Err(invalid("completed Shield lost its source defense"));
    }
    prior.effects.retain(|current| current.id != effect.id);
    crate::tactical_defenses::effective_armor_class_with_effects(state, target, Some(&prior))
}

pub(super) fn damage_cause(state: &CampaignState) -> Result<&CommandMeta, RulesError> {
    let attack = resolution(state)?
        .attack
        .as_ref()
        .ok_or_else(|| invalid("damage has no attack"))?;
    state
        .rules
        .as_ref()
        .ok_or(RulesError::Uninitialized)?
        .rolls
        .iter()
        .find(|record| Some(record.request.id) == attack.attack_roll)
        .map(|record| &record.accepted_by)
        .ok_or_else(|| invalid("damage has no accepted hit"))
}

pub(super) fn resume(
    state: &mut CampaignState,
    attack_origin: CommandId,
) -> Result<(), RulesError> {
    let hit = review(state)?;
    let attack = resolution(state)?
        .attack
        .as_ref()
        .ok_or_else(|| invalid("resumed hit has no attack"))?;
    if hit.stage != TacticalHitReviewStage::Resolved
        || hit.attack_origin != attack_origin
        || attack.origin.id != attack_origin
        || attack.stage != TacticalAttackStage::HitReview
    {
        return Err(invalid(
            "resumed hit differs from its completed response stage",
        ));
    }
    let record = state
        .rules
        .as_ref()
        .ok_or(RulesError::Uninitialized)?
        .rolls
        .iter()
        .find(|record| record.request.id == hit.attack_roll)
        .ok_or_else(|| invalid("resumed hit lost its dice"))?;
    let face = record.resolved.kept_dice[0].value;
    let armor =
        crate::tactical_defenses::effective_armor_class(state, attack.target)? + hit.cover_bonus;
    let hits = face != 1 && (face == 20 || record.resolved.total >= armor);
    let critical = face == 20 || attack.critical_on_hit;
    let attack = resolution_mut(state)?
        .attack
        .as_mut()
        .ok_or_else(|| invalid("resumed attack disappeared"))?;
    attack.stage = if hits {
        TacticalAttackStage::DamageRoll
    } else {
        TacticalAttackStage::Finishing
    };
    attack.outcome = Some(if hits {
        WeaponAttackOutcome::Hit {
            critical,
            damage_dealt: 0,
        }
    } else {
        WeaponAttackOutcome::Miss
    });
    push_frame(
        state,
        vec![if hits {
            TacticalWorkKind::AttackDamage
        } else {
            TacticalWorkKind::FinishAttack
        }],
    )
}

fn validate_order(
    state: &CampaignState,
    meta: &CommandMeta,
    instruction: &TacticalReactionOrdering,
) -> Result<(), RulesError> {
    let actor = resolution(state)?.turn_actor;
    let initiative = state
        .rules
        .as_ref()
        .and_then(|rules| rules.timing.as_ref())
        .ok_or_else(|| invalid("reaction order has no initiative"))?
        .order
        .iter()
        .map(|entry| entry.actor)
        .collect::<Vec<_>>();
    let known = if matches!(meta.issuer, CommandIssuer::Admin | CommandIssuer::System) {
        initiative.iter().copied().collect()
    } else {
        crate::spatial::project_actor_view(encounter(state)?, state, actor)
            .map_err(|error| invalid(&error.to_string()))?
            .contacts
            .into_iter()
            .filter(|contact| contact.status != crate::spatial::ContactStatus::Remembered)
            .map(|contact| contact.entity_id)
            .chain(std::iter::once(actor))
            .collect()
    };
    order_reaction_respondents(instruction, &initiative, &known, &[]).map(|_| ())
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
        return Err(invalid("hit decision does not follow its genuine cause"));
    }
    Ok(())
}

pub(super) fn validate_work(
    state: &CampaignState,
    work: &TacticalWorkItem,
) -> Result<EntityId, RulesError> {
    let hit = review(state)?;
    let r = resolution(state)?;
    let trace = r
        .work_trace
        .as_ref()
        .ok_or_else(|| invalid("hit response lacks causal ancestry"))?;
    let node = trace
        .nodes
        .iter()
        .find(|node| node.work == *work)
        .ok_or_else(|| invalid("hit response work lacks its exact causal node"))?;
    if node.parent != Some(hit.work.occurrence) {
        return Err(invalid("hit response work belongs to another trigger"));
    }
    match work.kind {
        TacticalWorkKind::ResumeHit { attack_origin } if attack_origin == hit.attack_origin => (),
        TacticalWorkKind::CommitShield { cast }
            if hit.selected_cast == Some(cast) && hit.stage == TacticalHitReviewStage::Casting =>
        {
            ()
        }
        _ => {
            return Err(invalid(
                "hit response work differs from its accepted window",
            ));
        }
    }
    Ok(r.attack
        .as_ref()
        .ok_or_else(|| invalid("hit response lost its attack"))?
        .target)
}

/// Shape/source validation complements replay of every accepted command. No
/// retained number, response, or effect is accepted as an authority of its own.
pub(super) fn validate(state: &CampaignState) -> Result<(), RulesError> {
    let Some(r) = flow(state)?.resolution.as_deref() else {
        return Ok(());
    };
    let Some(hit) = r.hit_review.as_deref() else {
        let required = r.attack.as_ref().is_some_and(|attack| {
            if attack.stage == TacticalAttackStage::HitReview {
                return true;
            }
            if flow(state).is_ok_and(|flow| {
                flow.version == TacticalExecutionVersion::ShieldHitV1.flow_version()
            }) {
                return attack
                    .attack_roll
                    .and_then(|id| {
                        state
                            .rules
                            .as_ref()?
                            .rolls
                            .iter()
                            .find(|roll| roll.request.id == id)
                    })
                    .is_some_and(|roll| {
                        roll.resolved.kept_dice.first().is_some_and(|face| {
                            face.value != 1
                                && (face.value == 20 || roll.resolved.total >= attack.armor_class)
                        })
                    });
            }
            false
        });
        if required {
            return Err(invalid("hit pause lacks its source response window"));
        }
        return Ok(());
    };
    let attack = r
        .attack
        .as_ref()
        .ok_or_else(|| invalid("hit response lost its attack"))?;
    let rules = state.rules.as_ref().ok_or(RulesError::Uninitialized)?;
    let roll = rules
        .rolls
        .iter()
        .find(|roll| roll.request.id == hit.attack_roll)
        .ok_or_else(|| invalid("hit response lost its accepted dice"))?;
    let trace = r
        .work_trace
        .as_ref()
        .ok_or_else(|| invalid("hit response lacks causal ancestry"))?;
    if flow(state)?.version != TacticalExecutionVersion::ShieldHitV1.flow_version()
        || hit.attack_origin != attack.origin.id
        || attack.attack_roll != Some(hit.attack_roll)
        || hit.cause != roll.accepted_by
        || hit.work.kind != TacticalWorkKind::AttackRoll
        || hit.work.occurrence >= r.next_occurrence
        || trace.nodes.iter().all(|node| node.work != hit.work)
        || hit.respondent.is_none()
        || roll.purpose
            != (PendingPurpose::TacticalResolution {
                encounter: encounter(state)?.id,
                key: super::continuations::key(state, &hit.work)?,
            })
        || roll.request.resolve(&roll.result)? != roll.resolved
        || hit.cover_bonus
            != attack.armor_class - original_defense(state, attack.actor, attack.target)?
        || !matches!(hit.cover_bonus, 0 | 2 | 5)
    {
        return Err(invalid(
            "hit response differs from its source attack and dice",
        ));
    }
    let face = roll
        .resolved
        .kept_dice
        .first()
        .ok_or_else(|| invalid("hit has no kept attack die"))?
        .value;
    if face == 1 || (face != 20 && roll.resolved.total < attack.armor_class) {
        return Err(invalid("reaction window is not attached to a hit"));
    }
    let mut decisions = std::collections::HashSet::new();
    let mut retain_decision =
        |meta: &CommandMeta, actor, after: &CommandMeta| -> Result<(), RulesError> {
            decision(state, meta, actor, after)?;
            if !decisions.insert(meta.id) {
                return Err(invalid("one command impersonates multiple hit decisions"));
            }
            Ok(())
        };
    if let Some(delegated) = &hit.delegated_by {
        retain_decision(delegated, r.turn_actor, &hit.cause)?;
        owner(state, delegated, r.turn_actor)?;
    }
    if let Some(order) = &hit.order {
        retain_decision(
            &order.origin,
            r.turn_actor,
            hit.delegated_by.as_ref().unwrap_or(&hit.cause),
        )?;
        if hit.delegated_by.is_some() {
            privileged(&order.origin)?;
        } else {
            owner(state, &order.origin, r.turn_actor)?;
        }
        validate_order(state, &order.origin, &order.instruction)?;
    }
    let intent = hit
        .respondent
        .as_ref()
        .and_then(|respondent| respondent.intent.as_ref());
    let declined = hit
        .respondent
        .as_ref()
        .and_then(|respondent| respondent.declined_after_selection.as_ref());
    if let Some(respondent) = &hit.respondent {
        if respondent.actor != attack.target {
            return Err(invalid("Shield respondent is not the hit target"));
        }
        if let Some(intent) = intent {
            retain_decision(&intent.origin, respondent.actor, &hit.cause)?;
            owner(state, &intent.origin, respondent.actor)?;
        }
    }
    let all_decided = hit.order.is_some() && (hit.respondent.is_none() || intent.is_some());
    let accepted = intent.is_some_and(|intent| intent.accepted);
    let selected_after = match (&hit.order, intent) {
        (Some(order), Some(intent)) if accepted => Some(
            if order.origin.expected_event_sequence > intent.origin.expected_event_sequence {
                &order.origin
            } else {
                &intent.origin
            },
        ),
        _ => None,
    };
    if let Some(declined) = declined {
        retain_decision(
            declined,
            attack.target,
            selected_after.ok_or_else(|| invalid("decline lacks accepted selection"))?,
        )?;
        owner(state, declined, attack.target)?;
    }
    if let Some(record) = hit.completed_shield.as_deref() {
        validate_retained_spell(record)?;
        super::casting::validate_actor_source(state, record)?;
        retain_decision(
            &record.cast.plan.origin,
            attack.target,
            selected_after.ok_or_else(|| invalid("Shield lacks accepted selection"))?,
        )?;
        owner(state, &record.cast.plan.origin, attack.target)?;
        if hit.selected_cast != Some(record.cast.plan.occurrence)
            || record.cast.plan.occurrence >= r.next_occurrence
            || record.cast.phase != SpellCastPhase::Committed
            || record.cast.last_operation != record.cast.plan.origin
            || record.cast.started_on_turn != r.turn_number
            || record.cast.started_at != state.clock.now
            || record.completed.as_slice() != [SpellProgramOccurrence { node: 0, target: 0 }]
            || declined.is_some()
            || r.casts
                .iter()
                .any(|cast| cast.cast.plan.occurrence == record.cast.plan.occurrence)
            || !rules
                .timing
                .as_ref()
                .is_some_and(|timing| timing.reactions_spent.contains(&attack.target))
            || record
                .creature_activation
                .as_ref()
                .is_some_and(|activation| {
                    activation.activation != SpellEnclosingActivation::Reaction
                        || activation.origin != record.cast.plan.origin
                        || activation.attack_action
                })
        {
            return Err(invalid(
                "Shield completion differs from its selected source cast",
            ));
        }
        let commits = trace
            .nodes
            .iter()
            .filter(|node| {
                node.work.kind
                    == (TacticalWorkKind::CommitShield {
                        cast: record.cast.plan.occurrence,
                    })
            })
            .collect::<Vec<_>>();
        if commits.len() != 1 || commits[0].parent != Some(hit.work.occurrence) {
            return Err(invalid(
                "Shield completion lacks its exact independent child",
            ));
        }
    } else if hit.selected_cast.is_some() {
        return Err(invalid("selected Shield has no completed source receipt"));
    }
    let resume = r
        .frames
        .iter()
        .flatten()
        .filter(|work| matches!(work.kind, TacticalWorkKind::ResumeHit { .. }))
        .collect::<Vec<_>>();
    if r.frames
        .iter()
        .flatten()
        .any(|work| matches!(work.kind, TacticalWorkKind::CommitShield { .. }))
    {
        return Err(invalid("unresolved automatic Shield commit was persisted"));
    }
    match hit.stage {
        TacticalHitReviewStage::Collecting | TacticalHitReviewStage::Selected => {
            if attack.stage != TacticalAttackStage::HitReview
                || resume.len() != 1
                || r.frames.last().map(Vec::as_slice) != Some(std::slice::from_ref(resume[0]))
                || hit.completed_shield.is_some()
                || declined.is_some()
                || (hit.stage == TacticalHitReviewStage::Collecting && all_decided)
                || (hit.stage == TacticalHitReviewStage::Selected && (!all_decided || !accepted))
                || (accepted && shield_choices(state, attack.target)?.is_empty())
            {
                return Err(invalid(
                    "hit collection/selection differs from its waiting work",
                ));
            }
            validate_work(state, resume[0])?;
        }
        TacticalHitReviewStage::Resolved => {
            if !all_decided
                || !resume.is_empty()
                || attack.stage == TacticalAttackStage::HitReview
                || (accepted && declined.is_none() && hit.completed_shield.is_none())
                || (!accepted && (declined.is_some() || hit.completed_shield.is_some()))
            {
                return Err(invalid("resolved hit lacks its actual response decisions"));
            }
        }
        TacticalHitReviewStage::Casting => {
            return Err(invalid(
                "automatic Shield child was not completed before persistence",
            ));
        }
    }
    Ok(())
}
