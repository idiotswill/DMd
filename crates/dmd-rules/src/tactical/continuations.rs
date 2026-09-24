use super::turns::*;
use super::*;
use crate::tactical_conditions::{self, TestDisposition};
use crate::tactical_damage::VitalityFollowup;
use crate::tactical_effects::*;

pub(super) fn ticket(
    state: &CampaignState,
    id: EffectTicketId,
) -> Result<&ScheduledEffectTrigger, RulesError> {
    effects(state)?
        .pending
        .iter()
        .find(|t| t.id == id)
        .ok_or_else(|| invalid("effect ticket is no longer pending"))
}
fn save_modifier(
    state: &CampaignState,
    actor: EntityId,
    ability: Ability,
) -> Result<i32, RulesError> {
    let rules = state.rules.as_ref().ok_or(RulesError::Uninitialized)?;
    let entity = rules
        .entities
        .get(&actor)
        .ok_or_else(|| invalid("saving actor missing"))?;
    match &flow(state)?
        .combatants
        .iter()
        .find(|c| c.actor == actor)
        .ok_or_else(|| invalid("saving actor outside encounter"))?
        .source
    {
        TacticalSource::Character => Ok(crate::test_modifier(entity, &TestKind::Save { ability })),
        TacticalSource::Creature { definition_id } => Ok(i32::from(
            definitions()?
                .creature(definition_id)
                .ok_or_else(|| invalid("unknown source creature"))?
                .statistics
                .saving_throw_modifiers[ability.index()],
        ) - i32::from(entity.exhaustion) * 2),
    }
}
fn visibility(state: &CampaignState, actor: EntityId) -> RollVisibility {
    if controller(state, actor).is_some() {
        RollVisibility::Public
    } else {
        RollVisibility::Secret
    }
}
fn save_request(
    state: &CampaignState,
    actor: EntityId,
    key: TacticalRollKey,
    ability: Ability,
    reason: &str,
) -> Result<Option<RollRequest>, RulesError> {
    let rules = state.rules.as_ref().ok_or(RulesError::Uninitialized)?;
    let disposition = tactical_conditions::save_conditions(
        rules,
        actor,
        ability,
        Circumstances::default(),
        dodge_context(state, actor)?,
    )?;
    let TestDisposition::Roll(mode) = disposition else {
        return Ok(None);
    };
    Ok(Some(RollRequest {
        id: key.request_id(),
        roller: Some(actor),
        dice: vec![DieSpec {
            count: 1,
            sides: 20,
        }],
        modifier: save_modifier(state, actor, ability)?,
        mode,
        visibility: visibility(state, actor),
        reason: reason.into(),
    }))
}

pub(super) fn key(
    state: &CampaignState,
    work: &TacticalWorkItem,
) -> Result<TacticalRollKey, RulesError> {
    let (role, subject) = match &work.kind {
        TacticalWorkKind::DeathSave { actor } => (TacticalRollRole::DeathSave, *actor),
        TacticalWorkKind::StableRecovery { actor, .. } => {
            (TacticalRollRole::StableRecovery, *actor)
        }
        TacticalWorkKind::ConcentrationSave { actor, .. } => {
            (TacticalRollRole::Concentration, *actor)
        }
        TacticalWorkKind::Effect { ticket: id } => {
            let ticket = ticket(state, *id)?;
            (
                match ticket.payload {
                    EffectTriggerPayload::SavingThrow { .. } => TacticalRollRole::EffectSave,
                    EffectTriggerPayload::Damage { .. } => TacticalRollRole::EffectDamage,
                    _ => return Err(invalid("effect work has no roll")),
                },
                ticket.target,
            )
        }
        TacticalWorkKind::RecoverStable { .. } => return Err(invalid("wake-up has no roll")),
    };
    Ok(TacticalRollKey {
        origin: resolution(state)?.origin.id,
        role,
        subject,
        occurrence: work.occurrence,
    })
}
pub(super) fn ruling(role: TacticalRollRole) -> Ruling {
    let (page, reason) = match role {
        TacticalRollRole::DeathSave => (
            17,
            "Death saving throw at the start of the creature's turn.",
        ),
        TacticalRollRole::StableRecovery => (18, "A stable creature regains 1 HP after 1d4 hours."),
        TacticalRollRole::Concentration => (
            180,
            "A separate Constitution save follows this damage occurrence.",
        ),
        TacticalRollRole::EffectSave => (
            187,
            "Resolve the saving throw derived from the retained source effect.",
        ),
        TacticalRollRole::EffectDamage => (17, "Roll the retained source effect's damage dice."),
    };
    Ruling {
        basis: RulingBasis::Srd { page },
        reason: reason.into(),
    }
}
pub(super) fn request(
    state: &CampaignState,
    work: &TacticalWorkItem,
    key: TacticalRollKey,
) -> Result<Option<RollRequest>, RulesError> {
    let rules = state.rules.as_ref().ok_or(RulesError::Uninitialized)?;
    match &work.kind {
        TacticalWorkKind::DeathSave { actor } => {
            let context = crate::tactical_vitality_adapter::context(
                state,
                *actor,
                VitalityOrigin {
                    command: resolution(state)?.origin.clone(),
                    occurrence: work.occurrence,
                },
            )?;
            let mut request = crate::tactical_damage::death_save_request(
                &rules.entities[actor],
                &context,
                key.request_id(),
            )
            .map_err(|e| invalid(&e.to_string()))?;
            request.visibility = visibility(state, *actor);
            Ok(Some(request))
        }
        TacticalWorkKind::StableRecovery { actor, .. } => {
            let mut request =
                crate::tactical_damage::stable_recovery_request(*actor, key.request_id())
                    .map_err(|e| invalid(&e.to_string()))?;
            request.visibility = visibility(state, *actor);
            Ok(Some(request))
        }
        TacticalWorkKind::ConcentrationSave { actor, .. } => save_request(
            state,
            *actor,
            key,
            Ability::Constitution,
            "Concentration saving throw",
        ),
        TacticalWorkKind::Effect { ticket: id } => {
            let ticket = ticket(state, *id)?;
            match &ticket.payload {
                EffectTriggerPayload::SavingThrow { ability, .. } => {
                    save_request(state, ticket.target, key, *ability, "Effect saving throw")
                }
                EffectTriggerPayload::Damage { dice, modifier, .. } => Ok(Some(RollRequest {
                    id: key.request_id(),
                    roller: Some(ticket.source.actor),
                    dice: dice.clone(),
                    modifier: i32::from(*modifier),
                    mode: RollMode::Normal,
                    visibility: visibility(state, ticket.source.actor),
                    reason: "Effect damage".into(),
                })),
                _ => Err(invalid("selected effect has no raw roll")),
            }
        }
        TacticalWorkKind::RecoverStable { .. } => Err(invalid("stable wake-up has no raw roll")),
    }
}

pub(super) fn start(
    state: &mut CampaignState,
    meta: &CommandMeta,
    work: TacticalWorkItem,
) -> Result<(), RulesError> {
    match &work.kind {
        TacticalWorkKind::Effect { ticket: id } => {
            let trigger = ticket(state, *id)?;
            if !trigger_is_applicable(effects(state)?, trigger)
                .map_err(|e| invalid(&e.to_string()))?
            {
                return acknowledge(state, meta, *id, EffectTriggerResolution::SkipInactive);
            }
            if !matches!(
                trigger.payload,
                EffectTriggerPayload::SavingThrow { .. } | EffectTriggerPayload::Damage { .. }
            ) {
                return acknowledge(state, meta, *id, EffectTriggerResolution::Apply);
            }
        }
        TacticalWorkKind::DeathSave { actor } => {
            let e = &state
                .rules
                .as_ref()
                .ok_or(RulesError::Uninitialized)?
                .entities[actor];
            if e.hp != 0 || e.death.dead || e.death.stable {
                return Ok(());
            }
        }
        TacticalWorkKind::ConcentrationSave { actor, group, .. } => {
            let e = &state
                .rules
                .as_ref()
                .ok_or(RulesError::Uninitialized)?
                .entities[actor];
            if e.concentration != Some(*group) {
                return Ok(());
            }
        }
        TacticalWorkKind::RecoverStable { actor } => {
            apply_vitality(
                state,
                meta,
                *actor,
                work.occurrence,
                VitalityOperation::RecoverStable,
                None,
            )?;
            return Ok(());
        }
        TacticalWorkKind::StableRecovery { actor, origin } => {
            let current = state
                .rules
                .as_ref()
                .and_then(|r| r.tactical_recovery.as_ref())
                .and_then(|r| r.get(actor))
                .and_then(|r| r.stable.as_ref());
            if current.is_none_or(|r| r.origin != *origin || r.delay_roll.is_some()) {
                return Ok(());
            }
        }
    }
    let key = key(state, &work)?;
    let derived = request(state, &work, key)?;
    let pending = TacticalPendingWork { work, key };
    resolution_mut(state)?.pending = Some(pending.clone());
    let Some(request) = derived else {
        flow_mut(state)?.save_decisions.push(TacticalSaveDecision {
            key,
            issued_by: meta.clone(),
            resolved_by: meta.clone(),
            failure: TacticalSaveFailure::Automatic,
        });
        finish(state, meta, pending, None)?;
        return Ok(());
    };
    let encounter = encounter(state)?.id;
    let rules = state.rules.as_mut().ok_or(RulesError::Uninitialized)?;
    if rules.rolls.iter().any(|r| r.request.id == request.id)
        || rules.cancelled_roll_ids.contains(&request.id)
        || rules.pending.is_some()
    {
        return Err(invalid("tactical request identity already used"));
    }
    rules.pending = Some(PendingRoll {
        issued_by: meta.clone(),
        request,
        purpose: PendingPurpose::TacticalResolution { encounter, key },
        ruling: ruling(key.role),
    });
    Ok(())
}

pub(super) fn submit(
    state: &mut CampaignState,
    meta: &CommandMeta,
    result: &RollResult,
    inspiration: Option<(usize, DieResult)>,
) -> Result<(), RulesError> {
    let pending = state
        .rules
        .as_ref()
        .and_then(|r| r.pending.as_ref())
        .ok_or(RulesError::NoPending)?
        .clone();
    super::validation::validate_tactical_pending(state, &pending)?;
    let actor = pending
        .request
        .roller
        .ok_or_else(|| invalid("missing tactical roller"))?;
    authorize(state, meta, actor)?;
    if (result.source == RollSource::Digital
        || pending.request.visibility == RollVisibility::Secret)
        && !matches!(meta.issuer, CommandIssuer::Admin | CommandIssuer::System)
    {
        return Err(RulesError::Unauthorized);
    }
    pending.request.resolve(result)?;
    let mut accepted = result.clone();
    if let Some((index, replacement)) = inspiration {
        let die = accepted
            .dice
            .get_mut(index)
            .ok_or_else(|| invalid("Inspiration die index absent"))?;
        if die.sides != replacement.sides {
            return Err(invalid("Inspiration die sides differ"));
        }
        *die = replacement;
        let entity = state
            .rules
            .as_mut()
            .ok_or(RulesError::Uninitialized)?
            .entities
            .get_mut(&actor)
            .ok_or_else(|| invalid("roller absent"))?;
        if !entity.heroic_inspiration {
            return Err(prerequisite("no Heroic Inspiration"));
        }
        entity.heroic_inspiration = false;
    }
    let resolved = pending.request.resolve(&accepted)?;
    let continuation = resolution(state)?
        .pending
        .clone()
        .ok_or_else(|| invalid("missing pending work"))?;
    let rules = state.rules.as_mut().ok_or(RulesError::Uninitialized)?;
    rules.rolls.push(RecordedRoll {
        issued_by: pending.issued_by,
        accepted_by: meta.clone(),
        request: pending.request,
        result: accepted.clone(),
        resolved,
        purpose: pending.purpose,
        original_result: inspiration.map(|_| result.clone()),
        savage_attacker: None,
    });
    rules.pending = None;
    finish(state, meta, continuation, Some(&accepted))?;
    pump(state, meta)
}

pub(super) fn voluntarily_fail(
    state: &mut CampaignState,
    meta: &CommandMeta,
) -> Result<(), RulesError> {
    let pending = state
        .rules
        .as_ref()
        .and_then(|r| r.pending.as_ref())
        .ok_or(RulesError::NoPending)?
        .clone();
    super::validation::validate_tactical_pending(state, &pending)?;
    let continuation = resolution(state)?
        .pending
        .clone()
        .ok_or_else(|| invalid("missing pending work"))?;
    if !matches!(
        continuation.key.role,
        TacticalRollRole::DeathSave
            | TacticalRollRole::EffectSave
            | TacticalRollRole::Concentration
    ) {
        return Err(prerequisite("pending work is not a saving throw"));
    }
    authorize(
        state,
        meta,
        pending
            .request
            .roller
            .ok_or_else(|| invalid("missing save actor"))?,
    )?;
    let rules = state.rules.as_mut().ok_or(RulesError::Uninitialized)?;
    rules.pending = None;
    rules.cancelled_roll_ids.push(pending.request.id);
    flow_mut(state)?.save_decisions.push(TacticalSaveDecision {
        key: continuation.key,
        issued_by: pending.issued_by,
        resolved_by: meta.clone(),
        failure: TacticalSaveFailure::Voluntary,
    });
    finish(state, meta, continuation, None)?;
    pump(state, meta)
}

fn finish(
    state: &mut CampaignState,
    meta: &CommandMeta,
    pending: TacticalPendingWork,
    result: Option<&RollResult>,
) -> Result<(), RulesError> {
    // Derive the current request before removing the selected ticket/cause.
    let raw = result
        .map(|result| {
            request(state, &pending.work, pending.key)?
                .ok_or_else(|| invalid("automatic save cannot accept dice"))?
                .resolve(result)
                .map_err(RulesError::from)
        })
        .transpose()?;
    resolution_mut(state)?.pending = None;
    match pending.work.kind {
        TacticalWorkKind::DeathSave { actor } => {
            let operation = result.map_or(VitalityOperation::FailDeathSave, |result| {
                VitalityOperation::DeathSave {
                    request_id: pending.key.request_id(),
                    result: result.clone(),
                }
            });
            apply_vitality(state, meta, actor, pending.work.occurrence, operation, None)?;
        }
        TacticalWorkKind::StableRecovery { actor, origin } => {
            let result = result.ok_or_else(|| invalid("recovery requires a raw d4"))?;
            apply_vitality(
                state,
                meta,
                actor,
                pending.work.occurrence,
                VitalityOperation::StableRecoveryRoll {
                    origin,
                    request_id: pending.key.request_id(),
                    result: result.clone(),
                },
                None,
            )?;
        }
        TacticalWorkKind::ConcentrationSave {
            actor,
            group,
            damage_taken,
        } => {
            let dc = (damage_taken / 2).clamp(10, 30);
            if raw.is_none_or(|r| i64::from(r.total) < i64::from(dc)) {
                end_concentration(state, meta, actor, group)?;
            }
        }
        TacticalWorkKind::Effect { ticket: id } => {
            let trigger = ticket(state, id)?.clone();
            match trigger.payload {
                EffectTriggerPayload::SavingThrow { dc, .. } => acknowledge(
                    state,
                    meta,
                    id,
                    EffectTriggerResolution::SavingThrow {
                        success: raw.is_some_and(|r| r.total >= i32::from(dc)),
                    },
                )?,
                EffectTriggerPayload::Damage { damage_type, .. } => {
                    let total = raw
                        .ok_or_else(|| invalid("damage requires raw dice"))?
                        .total
                        .max(0) as u32;
                    acknowledge(state, meta, id, EffectTriggerResolution::Apply)?;
                    apply_vitality(
                        state,
                        meta,
                        trigger.target,
                        pending.work.occurrence,
                        VitalityOperation::Damage {
                            packet: DamagePacket {
                                cause: DamageCause::Other,
                                components: vec![DamageComponent {
                                    damage_type,
                                    amounts: vec![total],
                                    adjustments: vec![],
                                }],
                            },
                            knockout: None,
                        },
                        Some(trigger.source.actor),
                    )?;
                }
                _ => return Err(invalid("pending effect is not roll-bearing")),
            }
        }
        TacticalWorkKind::RecoverStable { .. } => {
            return Err(invalid("wake-up cannot be pending dice"));
        }
    }
    refresh_dodges(state)?;
    Ok(())
}

fn end_concentration(
    state: &mut CampaignState,
    meta: &CommandMeta,
    actor: EntityId,
    group: EffectId,
) -> Result<(), RulesError> {
    if effects(state)?
        .group_for_owner(actor)
        .is_some_and(|g| g.id == group)
    {
        effect_operation(
            state,
            meta,
            EffectLifecycleOperation::EndConcentration {
                owner: actor,
                reason: EffectEndReason::ConcentrationBroken,
            },
        )
    } else {
        let rules = state.rules.as_mut().ok_or(RulesError::Uninitialized)?;
        rules
            .effects
            .retain(|e| e.concentration_owner != Some(actor));
        if let Some(e) = rules.entities.get_mut(&actor) {
            e.concentration = None;
        }
        Ok(())
    }
}

pub(super) fn apply_vitality(
    state: &mut CampaignState,
    meta: &CommandMeta,
    actor: EntityId,
    occurrence: u16,
    operation: VitalityOperation,
    damage_source: Option<EntityId>,
) -> Result<(), RulesError> {
    let transition = crate::tactical_vitality_adapter::apply(
        state,
        actor,
        VitalityOrigin {
            command: meta.clone(),
            occurrence,
        },
        &operation,
    )?;
    let mut work = Vec::new();
    for followup in transition.followups {
        match followup {
            VitalityFollowup::ConcentrationSave {
                group,
                damage_taken,
                ..
            } => work.push(TacticalWorkKind::ConcentrationSave {
                actor,
                group,
                damage_taken,
            }),
            VitalityFollowup::EndConcentration { group } => {
                end_concentration(state, meta, actor, group)?
            }
            VitalityFollowup::DropHeldItems => {
                crate::tactical_vitality_adapter::drop_held(state, actor, meta)?
            }
            VitalityFollowup::InterruptRest => {
                let rules = state.rules.as_mut().ok_or(RulesError::Uninitialized)?;
                crate::kernel::interrupt_rest(rules, actor, state.clock.now);
            }
            VitalityFollowup::StartKnockoutShortRest { started_at } => {
                let rules = state.rules.as_mut().ok_or(RulesError::Uninitialized)?;
                rules.rests.retain(|r| r.actor != actor);
                rules.rests.push(RestProgress {
                    actor,
                    kind: RestKind::Short,
                    started_at,
                });
            }
            VitalityFollowup::StableRecoveryRoll { origin } => {
                work.push(TacticalWorkKind::StableRecovery { actor, origin })
            }
            VitalityFollowup::ChooseKnockout { .. }
            | VitalityFollowup::ChooseTemporaryHitPoints { .. } => {
                return Err(invalid("unpersisted vitality decision"));
            }
        }
    }
    if transition.outcome.damage_taken > 0 {
        effect_operation(
            state,
            meta,
            EffectLifecycleOperation::Observe(EffectObservation::Damage {
                source: damage_source,
                target: actor,
                amount: transition.outcome.damage_taken,
            }),
        )?;
    }
    work.extend(new_effect_work(state)?);
    push_frame(state, work)?;
    refresh_dodges(state)
}
