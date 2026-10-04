//! Target responses, amount collection and ordered impacts use one shared cursor.
mod validation;
pub(super) use validation::{validate, validate_cast_partition, validate_work};

use super::turns::*;
use super::*;
use crate::tactical_spells::*;
use std::collections::HashSet;

fn current(state: &CampaignState, missile: u16) -> Result<&TacticalMissile, RulesError> {
    resolution(state)?
        .missiles
        .iter()
        .find(|record| record.work.occurrence == missile)
        .ok_or_else(|| prerequisite("that missile response is no longer available"))
}

fn current_mut(
    state: &mut CampaignState,
    missile: u16,
) -> Result<&mut TacticalMissile, RulesError> {
    resolution_mut(state)?
        .missiles
        .iter_mut()
        .find(|record| record.work.occurrence == missile)
        .ok_or_else(|| prerequisite("that missile response is no longer available"))
}

fn require_window(state: &CampaignState, window: TacticalWorkKey) -> Result<u16, RulesError> {
    if window.resolution != resolution(state)?.origin.id
        || !TacticalExecutionVersion::from_flow_version(flow(state)?.version)
            .is_some_and(TacticalExecutionVersion::supports_missile_shield)
    {
        return Err(prerequisite("that missile response is no longer available"));
    }
    current(state, window.occurrence)?;
    Ok(window.occurrence)
}

fn owner(state: &CampaignState, meta: &CommandMeta, actor: EntityId) -> Result<(), RulesError> {
    if controller(state, actor).is_some_and(|player| meta.issuer != CommandIssuer::Player(player)) {
        return Err(RulesError::Unauthorized);
    }
    authorize(state, meta, actor)
}

pub(super) fn is_program(record: &TacticalCasting) -> bool {
    record.cast.plan.program.source.spell_id == "magic-missile"
        && matches!(
            record.cast.plan.program.nodes.as_slice(),
            [SpellProgramNode::AutomaticDamage {
                share: SpellDamageShare::PerDart,
                ..
            }]
        )
}

pub(super) fn owns_cast(state: &CampaignState, cast: u16) -> Option<u16> {
    resolution(state)
        .ok()?
        .missiles
        .iter()
        .find(|record| record.cast == cast && record.completed_cast.is_none())
        .map(|record| record.work.occurrence)
}

pub(super) fn waiting(state: &CampaignState) -> bool {
    state
        .encounter
        .as_ref()
        .and_then(|e| e.flow.as_ref())
        .and_then(|f| f.resolution.as_ref())
        .is_some_and(|r| {
            r.missiles.iter().any(|record| {
                matches!(
                    record.stage,
                    TacticalMissileStage::Collecting | TacticalMissileStage::Selected { .. }
                )
            })
        })
}

pub(super) fn open(
    state: &mut CampaignState,
    meta: &CommandMeta,
    work: &TacticalWorkItem,
    cast: u16,
) -> Result<(), RulesError> {
    let record = super::casting::current_cast(state, cast)?.clone();
    if !is_program(&record)
        || !TacticalExecutionVersion::from_flow_version(flow(state)?.version)
            .is_some_and(TacticalExecutionVersion::supports_missile_shield)
        || record.cast.plan.choice.actor
            != resolution(state)?.turn_context().map_err(invalid)?.actor
        || owns_cast(state, cast).is_some()
        || record.targets.is_empty()
    {
        return Err(invalid(
            "missile trigger differs from its admitted current-turn source",
        ));
    }
    let mut seen = HashSet::new();
    let respondents = record
        .targets
        .iter()
        .filter(|target| seen.insert(target.actor))
        .map(|target| TacticalMissileRespondent {
            response: TacticalShieldRespondent {
                actor: target.actor,
                intent: None,
                declined_after_selection: None,
            },
            completed_shield: None,
        })
        .collect();
    let observer = record.cast.plan.choice.actor;
    let known = crate::spatial::project_actor_view(encounter(state)?, state, observer)
        .map_err(|error| invalid(&error.to_string()))?;
    let darts = record
        .targets
        .iter()
        .enumerate()
        .map(|(target, bound)| {
            let known_target_label = if bound.actor == observer {
                "Self".into()
            } else {
                known
                    .contacts
                    .iter()
                    .find(|contact| {
                        contact.entity_id == bound.actor
                            && contact.status == crate::spatial::ContactStatus::Seen
                    })
                    .and_then(|contact| contact.label.clone())
                    .ok_or_else(|| invalid("committed missile target lacks its seen label"))?
            };
            Ok(TacticalMissileDart {
                at: SpellProgramOccurrence {
                    node: 0,
                    target: target as u16,
                },
                target: bound.actor,
                known_target_label,
                amount: None,
                impact_occurrence: None,
                selected_by: None,
                completed_by: None,
            })
        })
        .collect::<Result<Vec<_>, RulesError>>()?;
    resolution_mut(state)?.missiles.push(TacticalMissile {
        cast,
        work: work.clone(),
        cause: meta.clone(),
        stage: TacticalMissileStage::Collecting,
        delegated_by: None,
        order: None,
        respondents,
        darts,
        completed_cast: None,
    });
    push_frame(
        state,
        vec![TacticalWorkKind::ResumeMissile {
            missile: work.occurrence,
        }],
    )
}

fn ordered(state: &CampaignState, missile: &TacticalMissile) -> Result<Vec<EntityId>, RulesError> {
    let instruction = &missile
        .order
        .as_ref()
        .ok_or_else(|| invalid("missile order absent"))?
        .instruction;
    let initiative = state
        .rules
        .as_ref()
        .and_then(|r| r.timing.as_ref())
        .ok_or_else(|| invalid("missile order has no initiative"))?
        .order
        .iter()
        .map(|entry| entry.actor)
        .collect::<Vec<_>>();
    let accepted = missile
        .respondents
        .iter()
        .filter(|r| {
            r.response
                .intent
                .as_ref()
                .is_some_and(|intent| intent.accepted)
        })
        .map(|r| r.response.actor)
        .collect::<Vec<_>>();
    // Admission separately checked the original controller's knowledge. Sorting
    // the already admitted instruction must not re-evaluate later visibility.
    order_reaction_respondents(
        instruction,
        &initiative,
        &initiative.iter().copied().collect(),
        &accepted,
    )
}

fn select_next(state: &mut CampaignState, missile: u16) -> Result<(), RulesError> {
    let record = current(state, missile)?;
    if record.order.is_none()
        || record
            .respondents
            .iter()
            .any(|r| r.response.intent.is_none())
    {
        return Ok(());
    }
    let next = ordered(state, record)?.into_iter().find_map(|actor| {
        record.respondents.iter().position(|respondent| {
            respondent.response.actor == actor
                && respondent.response.declined_after_selection.is_none()
                && respondent.completed_shield.is_none()
        })
    });
    current_mut(state, missile)?.stage = next.map_or(TacticalMissileStage::Amounts, |respondent| {
        TacticalMissileStage::Selected {
            respondent: respondent as u16,
        }
    });
    Ok(())
}

fn advance(state: &mut CampaignState, meta: &CommandMeta, missile: u16) -> Result<(), RulesError> {
    select_next(state, missile)?;
    pump(state, meta)
}

pub(super) fn respond(
    state: &mut CampaignState,
    meta: &CommandMeta,
    window: TacticalWorkKey,
    actor: EntityId,
    accept: bool,
) -> Result<(), RulesError> {
    let missile = require_window(state, window)?;
    owner(state, meta, actor)?;
    let record = current(state, missile)?;
    let index = record
        .respondents
        .iter()
        .position(|r| r.response.actor == actor)
        .ok_or(RulesError::Unauthorized)?;
    if record.stage != TacticalMissileStage::Collecting
        || record.respondents[index].response.intent.is_some()
    {
        return Err(prerequisite("that missile response was already decided"));
    }
    if accept && super::hit_reactions::shield_choices(state, actor)?.is_empty() {
        return Err(prerequisite(
            "no source Shield response is currently available",
        ));
    }
    current_mut(state, missile)?.respondents[index]
        .response
        .intent = Some(TacticalReactionIntent {
        origin: meta.clone(),
        accepted: accept,
    });
    advance(state, meta, missile)
}

pub(super) fn delegate(
    state: &mut CampaignState,
    meta: &CommandMeta,
    window: TacticalWorkKey,
) -> Result<(), RulesError> {
    let missile = require_window(state, window)?;
    owner(
        state,
        meta,
        resolution(state)?.turn_context().map_err(invalid)?.actor,
    )?;
    let record = current_mut(state, missile)?;
    if record.stage != TacticalMissileStage::Collecting
        || record.order.is_some()
        || record.delegated_by.is_some()
    {
        return Err(prerequisite(
            "missile ordering authority was already decided",
        ));
    }
    record.delegated_by = Some(meta.clone());
    Ok(())
}

pub(super) fn order(
    state: &mut CampaignState,
    meta: &CommandMeta,
    window: TacticalWorkKey,
    instruction: &TacticalReactionOrdering,
) -> Result<(), RulesError> {
    let missile = require_window(state, window)?;
    let record = current(state, missile)?;
    if record.stage != TacticalMissileStage::Collecting || record.order.is_some() {
        return Err(prerequisite(
            "the missile ordering instruction is already fixed",
        ));
    }
    if record.delegated_by.is_some() {
        privileged(meta)?;
    } else {
        owner(
            state,
            meta,
            resolution(state)?.turn_context().map_err(invalid)?.actor,
        )?;
    }
    super::hit_reactions::validate_order(state, meta, instruction)?;
    current_mut(state, missile)?.order = Some(TacticalReactionOrderDecision {
        origin: meta.clone(),
        instruction: instruction.clone(),
    });
    advance(state, meta, missile)
}

pub(super) fn decline(
    state: &mut CampaignState,
    meta: &CommandMeta,
    window: TacticalWorkKey,
) -> Result<(), RulesError> {
    let missile = require_window(state, window)?;
    let TacticalMissileStage::Selected { respondent } = current(state, missile)?.stage else {
        return Err(RulesError::Pending);
    };
    let actor = current(state, missile)?.respondents[usize::from(respondent)]
        .response
        .actor;
    owner(state, meta, actor)?;
    current_mut(state, missile)?.respondents[usize::from(respondent)]
        .response
        .declined_after_selection = Some(meta.clone());
    advance(state, meta, missile)
}

pub(super) fn cast(
    state: &mut CampaignState,
    meta: &CommandMeta,
    window: TacticalWorkKey,
    choice: &SpellCastChoice,
) -> Result<(), RulesError> {
    let missile = require_window(state, window)?;
    let TacticalMissileStage::Selected { respondent } = current(state, missile)?.stage else {
        return Err(RulesError::Pending);
    };
    let actor = current(state, missile)?.respondents[usize::from(respondent)]
        .response
        .actor;
    owner(state, meta, actor)?;
    if choice.actor != actor {
        return Err(RulesError::Unauthorized);
    }
    let cast = resolution(state)?.next_occurrence;
    if cast >= 32_768 || resolution(state)?.casts.len() >= MAX_TACTICAL_CASTS {
        return Err(invalid("response casting capacity exceeded"));
    }
    let (record, source) = super::casting::shield_admission(state, meta, choice, cast)?;
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
    current_mut(state, missile)?.stage = TacticalMissileStage::Casting { respondent, cast };
    let original = current(state, missile)?.work.clone();
    let previous = super::work_trace::enter(state, &original)?;
    push_frame(
        state,
        vec![TacticalWorkKind::CommitMissileShield {
            missile,
            respondent,
            cast,
        }],
    )?;
    super::work_trace::leave(state, previous)?;
    pump(state, meta)
}

pub(super) fn finish_shield(
    state: &mut CampaignState,
    record: &TacticalCasting,
) -> Result<bool, RulesError> {
    let cast = record.cast.plan.occurrence;
    let parent = resolution(state)?
        .missiles
        .iter()
        .find_map(|missile| match missile.stage {
            TacticalMissileStage::Casting {
                respondent,
                cast: selected,
            } if selected == cast => Some((missile.work.occurrence, respondent)),
            _ => None,
        });
    let Some((missile, respondent)) = parent else {
        return Ok(false);
    };
    let target = &mut current_mut(state, missile)?.respondents[usize::from(respondent)];
    if target.completed_shield.is_some() || target.response.actor != record.cast.plan.choice.actor {
        return Err(invalid(
            "missile Shield completion differs from its selected owner",
        ));
    }
    target.completed_shield = Some(Box::new(record.clone()));
    // The caller removes the completed live cast before the outer pump resumes.
    // Select the next response without consuming/replacing the original cursor.
    select_next(state, missile)?;
    Ok(true)
}

pub(super) fn resume(state: &mut CampaignState, missile: u16) -> Result<(), RulesError> {
    let record = current(state, missile)?.clone();
    if record.stage != TacticalMissileStage::Amounts {
        return Err(invalid("missile amounts preceded completed responses"));
    }
    if record.respondents.iter().any(|r| {
        r.response.intent.as_ref().is_some_and(|i| i.accepted)
            && r.response.declined_after_selection.is_none()
            && r.completed_shield.is_none()
    }) {
        return Err(invalid("missile amounts preceded a selected response"));
    }
    push_frame(
        state,
        vec![TacticalWorkKind::BeginMissileImpacts { missile }],
    )?;
    for dart in record.darts.iter().rev() {
        push_frame(
            state,
            vec![TacticalWorkKind::SpellProgram {
                cast: record.cast,
                at: dart.at,
            }],
        )?;
    }
    Ok(())
}

pub(super) fn retain_amount(
    state: &mut CampaignState,
    pending: &TacticalPendingWork,
) -> Result<(), RulesError> {
    let TacticalWorkKind::SpellProgram { cast, at } = pending.work.kind else {
        return Err(invalid("missile amount is not a spell occurrence"));
    };
    let missile = owns_cast(state, cast).ok_or_else(|| invalid("missile amount lost its cast"))?;
    let record = current_mut(state, missile)?;
    if record.stage != TacticalMissileStage::Amounts {
        return Err(invalid("missile amount is outside collection"));
    }
    let dart = record
        .darts
        .get_mut(usize::from(at.target))
        .filter(|dart| dart.at == at)
        .ok_or_else(|| invalid("missile amount target is absent"))?;
    if dart.amount.is_some() {
        return Err(invalid("duplicate missile amount"));
    }
    dart.amount = Some(pending.key);
    Ok(())
}

pub(super) fn begin_impacts(state: &mut CampaignState, missile: u16) -> Result<(), RulesError> {
    let record = current(state, missile)?;
    if record.stage != TacticalMissileStage::Amounts
        || record.darts.iter().any(|dart| dart.amount.is_none())
    {
        return Err(invalid(
            "missile impact preceded complete amount collection",
        ));
    }
    let kinds = record
        .darts
        .iter()
        .map(|dart| TacticalWorkKind::ApplyMissileImpact {
            missile,
            at: dart.at,
        })
        .collect();
    let first = resolution(state)?.next_occurrence;
    push_frame(state, kinds)?;
    let record = current_mut(state, missile)?;
    for (index, dart) in record.darts.iter_mut().enumerate() {
        dart.impact_occurrence = Some(first + index as u16);
    }
    record.stage = TacticalMissileStage::Impacts;
    Ok(())
}

pub(super) fn select_impact(
    state: &mut CampaignState,
    meta: &CommandMeta,
    work: &TacticalWorkItem,
) -> Result<(), RulesError> {
    let TacticalWorkKind::ApplyMissileImpact { missile, at } = work.kind else {
        return Ok(());
    };
    owner(
        state,
        meta,
        resolution(state)?.turn_context().map_err(invalid)?.actor,
    )?;
    let record = current_mut(state, missile)?;
    let dart = record
        .darts
        .get_mut(usize::from(at.target))
        .filter(|dart| dart.at == at)
        .ok_or_else(|| invalid("selected missile impact has no committed dart"))?;
    if record.stage != TacticalMissileStage::Impacts
        || dart.selected_by.is_some()
        || dart.completed_by.is_some()
        || dart.impact_occurrence != Some(work.occurrence)
    {
        return Err(invalid(
            "selected missile impact differs from its pending occurrence",
        ));
    }
    dart.selected_by = Some(meta.clone());
    Ok(())
}

pub(super) fn impact(
    state: &mut CampaignState,
    meta: &CommandMeta,
    work: &TacticalWorkItem,
    missile: u16,
    at: SpellProgramOccurrence,
) -> Result<(), RulesError> {
    let record = current(state, missile)?;
    let dart = record
        .darts
        .get(usize::from(at.target))
        .filter(|dart| dart.at == at)
        .ok_or_else(|| invalid("missile impact target is absent"))?;
    if record.stage != TacticalMissileStage::Impacts
        || dart.impact_occurrence != Some(work.occurrence)
        || dart.completed_by.is_some()
    {
        return Err(invalid("missile impact is not pending"));
    }
    if dart.selected_by.as_ref().is_some_and(|selected| selected != meta)
        || dart.selected_by.is_none() && resolution(state)?.frames.iter().flatten()
            .any(|work| matches!(work.kind, TacticalWorkKind::ApplyMissileImpact { missile: id, .. } if id == missile))
    { return Err(invalid("unselected missile impact is not the final singleton")); }
    let key = dart
        .amount
        .ok_or_else(|| invalid("missile impact lacks accepted faces"))?;
    let roll = state
        .rules
        .as_ref()
        .ok_or(RulesError::Uninitialized)?
        .rolls
        .iter()
        .find(|roll| roll.request.id == key.request_id())
        .cloned()
        .ok_or_else(|| invalid("missile amount lost its exact accepted record"))?;
    let cast = record.cast;
    let target = dart.target;
    let casting = super::casting::current_cast(state, cast)?.clone();
    let operation = spell_amount_operation(state, &casting, at, key.request_id(), &roll.result)?;
    current_mut(state, missile)?.darts[usize::from(at.target)].completed_by = Some(meta.clone());
    super::casting::complete_occurrence(state, cast, at)?;
    if let Some(operation) = operation {
        super::continuations::apply_vitality_from_cause(
            state,
            meta,
            target,
            VitalityOrigin {
                command: roll.accepted_by,
                occurrence: work.occurrence,
            },
            operation,
            Some(casting.cast.plan.choice.actor),
        )?;
    }
    Ok(())
}

pub(super) fn finish_cast(
    state: &mut CampaignState,
    casting: &TacticalCasting,
) -> Result<(), RulesError> {
    let Some(missile) = owns_cast(state, casting.cast.plan.occurrence) else {
        return Ok(());
    };
    let record = current_mut(state, missile)?;
    if record.stage != TacticalMissileStage::Impacts
        || record.darts.iter().any(|dart| dart.completed_by.is_none())
    {
        return Err(invalid(
            "missile cast completed before its committed impacts",
        ));
    }
    record.completed_cast = Some(Box::new(casting.clone()));
    record.stage = TacticalMissileStage::Completed;
    Ok(())
}
