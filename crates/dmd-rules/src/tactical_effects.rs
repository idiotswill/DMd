//! Internal, sequence-neutral reducer for source-authorized tactical effect facts.
//! Do not expose EffectLifecycleAction as a player command. The encounter resolver
//! validates source definitions, ownership, geometry and actual damage before calling.
use dmd_domain::*;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("invalid effect lifecycle: {0}")]
pub struct EffectLifecycleError(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EffectEndReason {
    Expired,
    SourceTrigger,
    Saved,
    Dispelled,
    Dismissed,
    ConcentrationBroken,
    ConcentrationReplaced,
    NoRemainingTargets,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum EffectLifecycleOperation {
    /// Replaces the owner's former group immediately, even if the new cast later fails.
    BeginConcentration {
        group: ConcentrationGroup,
    },
    /// Internal source cast commit/release only. This cannot create a group or
    /// refresh an already-active spell. Install activates it after target saves.
    SetCastingDuration {
        group: EffectId,
        source: EffectSource,
        expires: TacticalEffectExpiry,
    },
    Install {
        effects: Vec<TacticalEffect>,
    },
    EndEffect {
        effect: EffectId,
        reason: EffectEndReason,
    },
    EndConcentration {
        owner: EntityId,
        reason: EffectEndReason,
    },
    Observe(EffectObservation),
    /// After authoritative initiative ends, preserve lasting effects and release only
    /// the combat cursor/limits. The next encounter can begin at turn number one.
    LeaveCombat,
    /// Root records the controlling player's chosen simultaneous order. Saves/damage
    /// are resolved by the ordinary rules continuation before this acknowledgement.
    ResolveTrigger {
        ticket: EffectTicketId,
        outcome: EffectTriggerResolution,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EffectTriggerResolution {
    Apply,
    SavingThrow { success: bool },
    SkipInactive,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EffectLifecycleAction {
    pub step: u16,
    pub operation: EffectLifecycleOperation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EndedTacticalEffect {
    pub id: EffectId,
    pub group: bool,
    pub reason: EffectEndReason,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EffectLifecycleTransition {
    pub next_effects: TacticalEffects,
    pub ended: Vec<EndedTacticalEffect>,
    /// Newly queued tickets, also retained in next_effects.pending. This reducer never
    /// rolls dice, applies HP damage or chooses a simultaneous-effects order.
    pub triggers: Vec<ScheduledEffectTrigger>,
    /// False means a selected ticket was suppressed or its shared per-turn limit used;
    /// the caller must not apply its damage/save payload. None means no ticket selected.
    pub selected_trigger_applies: Option<bool>,
}

fn invalid(message: impl Into<String>) -> EffectLifecycleError {
    EffectLifecycleError(message.into())
}

fn precedence(effect: &TacticalEffect) -> (i32, u64, u16, u16) {
    (
        effect.overlap.as_ref().map_or(0, |o| o.potency),
        effect
            .established_at
            .as_ref()
            .map_or(0, |s| s.command.expected_event_sequence),
        effect.established_at.as_ref().map_or(0, |s| s.step),
        effect.source.ordinal,
    )
}

/// Suppression is derived for each affected target. Older castings retain identity,
/// duration, concentration and end-on-damage clauses while a stronger casting applies.
pub fn effect_applies_to(
    effects: &TacticalEffects,
    effect: &TacticalEffect,
    target: EntityId,
) -> bool {
    if !effect_has_target(effect, target) {
        return false;
    }
    not_suppressed(effects, effect, target)
}

fn not_suppressed(effects: &TacticalEffects, effect: &TacticalEffect, target: EntityId) -> bool {
    let Some(overlap) = &effect.overlap else {
        return true;
    };
    !effects.effects.iter().any(|other| {
        other.id != effect.id
            && effect_has_target(other, target)
            && other.overlap.as_ref().is_some_and(|o| o.key == overlap.key)
            && precedence(other) > precedence(effect)
    })
}

/// Ephemeral query adapter only. Never persist these records in legacy RulesState.effects:
/// the tactical group owns concentration and expiry, not the legacy one-ID model.
pub fn active_effect_views(effects: &TacticalEffects) -> Vec<ActiveEffect> {
    effects
        .effects
        .iter()
        .flat_map(|effect| {
            let TacticalEffectTarget::Creature(target) = effect.target else {
                return Vec::new();
            };
            if !effect_applies_to(effects, effect, target) {
                return Vec::new();
            }
            effect
                .conditions
                .iter()
                .map(|view| ActiveEffect {
                    id: view.id,
                    source: effect.source.actor,
                    target,
                    condition: Some(view.condition),
                    label: effect.source.definition_id.clone(),
                    // The lifecycle has already decided expiry; legacy mutation must not own it.
                    expires: Expiry::Never,
                    concentration_owner: None,
                })
                .collect()
        })
        .collect()
}

fn remove_group(result: &mut EffectLifecycleTransition, id: EffectId, reason: EffectEndReason) {
    let members: Vec<_> = result
        .next_effects
        .effects
        .iter()
        .filter(|e| e.concentration_group == Some(id))
        .map(|e| e.id)
        .collect();
    for member in members {
        remove_effect(result, member, reason, false);
    }
    if let Some(index) = result.next_effects.groups.iter().position(|g| g.id == id) {
        result.next_effects.groups.remove(index);
        result
            .next_effects
            .pending
            .retain(|ticket| ticket.effect != id);
        result.ended.push(EndedTacticalEffect {
            id,
            group: true,
            reason,
        });
    }
}

fn remove_effect(
    result: &mut EffectLifecycleTransition,
    id: EffectId,
    reason: EffectEndReason,
    prune: bool,
) {
    if let Some(index) = result.next_effects.effects.iter().position(|e| e.id == id) {
        let effect = result.next_effects.effects.remove(index);
        result
            .next_effects
            .trigger_uses
            .retain(|usage| usage.effect != id);
        result
            .next_effects
            .pending
            .retain(|ticket| ticket.effect != id);
        result.ended.push(EndedTacticalEffect {
            id,
            group: false,
            reason,
        });
        if prune
            && let Some(group) = effect.concentration_group
            && !result
                .next_effects
                .effects
                .iter()
                .any(|e| e.concentration_group == Some(group))
        {
            remove_group(result, group, EffectEndReason::NoRemainingTargets);
        }
    }
}

fn expiry_due(
    expiry: &mut TacticalEffectExpiry,
    observation: &EffectObservation,
    now: WorldInstant,
) -> bool {
    if expiry_matches(expiry, observation, now) {
        return true;
    }
    if let TacticalEffectExpiry::AfterOwnerBoundaries {
        owner,
        boundary,
        remaining,
    } = expiry
        && matches!(observation, EffectObservation::Turn(turn) if turn.actor == *owner && turn.boundary == *boundary)
    {
        *remaining -= 1;
    }
    false
}

fn queue(
    result: &mut EffectLifecycleTransition,
    mut ticket: ScheduledEffectTrigger,
) -> Result<(), EffectLifecycleError> {
    // Expiry is due once. Equal-looking damage observations remain distinct occurrences.
    if ticket.rule_index.is_none()
        && result
            .next_effects
            .pending
            .iter()
            .any(|p| p.effect == ticket.effect && p.payload == ticket.payload)
    {
        return Ok(());
    }
    if result.next_effects.pending.len() >= MAX_PENDING_EFFECT_TRIGGERS {
        return Err(invalid("pending effect trigger capacity exceeded"));
    }
    ticket.id.ordinal = result.triggers.len() as u16;
    result.next_effects.pending.push(ticket.clone());
    result.triggers.push(ticket);
    Ok(())
}

fn ticket(
    effect: EffectId,
    source: &EffectSource,
    target: EntityId,
    rule_index: Option<u16>,
    payload: EffectTriggerPayload,
    cause: &EffectObservation,
    stamp: &EffectOperationStamp,
) -> ScheduledEffectTrigger {
    ScheduledEffectTrigger {
        id: EffectTicketId {
            command: stamp.command.id,
            step: stamp.step,
            ordinal: 0,
        },
        effect,
        rule_index,
        source: source.clone(),
        target,
        payload,
        cause: cause.clone(),
        origin: stamp.clone(),
    }
}

fn observe(
    campaign: &CampaignState,
    result: &mut EffectLifecycleTransition,
    observation: &EffectObservation,
    stamp: &EffectOperationStamp,
) -> Result<(), EffectLifecycleError> {
    let entity_exists = |id| {
        campaign.entities.contains_key(&id)
            && campaign
                .rules
                .as_ref()
                .is_some_and(|rules| rules.entities.contains_key(&id))
    };
    match observation {
        EffectObservation::Turn(turn) => {
            if !result.next_effects.pending.is_empty() {
                return Err(invalid(
                    "resolve pending effects before advancing the turn boundary",
                ));
            }
            let timing = campaign
                .rules
                .as_ref()
                .and_then(|r| r.timing.as_ref())
                .ok_or_else(|| invalid("turn observation requires authoritative initiative"))?;
            if turn.number != timing.turn_number
                || timing
                    .order
                    .get(timing.index)
                    .is_none_or(|e| e.actor != turn.actor)
            {
                return Err(invalid(
                    "turn observation differs from authoritative initiative",
                ));
            }
            if let Some(previous) = result.next_effects.turn {
                let allowed = match (previous.boundary, turn.boundary) {
                    (TurnBoundary::Start, TurnBoundary::End) => {
                        previous.actor == turn.actor && previous.number == turn.number
                    }
                    (TurnBoundary::End, TurnBoundary::Start) => {
                        previous.number.checked_add(1) == Some(turn.number)
                    }
                    _ => false,
                };
                if !allowed {
                    return Err(invalid("duplicate, skipped or out-of-order turn boundary"));
                }
            } else if turn.boundary != TurnBoundary::Start {
                return Err(invalid("first effect turn observation must be a start"));
            }
            if turn.boundary == TurnBoundary::Start {
                result.next_effects.trigger_uses.clear();
            }
            result.next_effects.turn = Some(*turn);
        }
        EffectObservation::Damage { source, target, .. } => {
            if !entity_exists(*target) || source.is_some_and(|id| !entity_exists(id)) {
                return Err(invalid("damage observation references an unknown entity"));
            }
        }
        EffectObservation::ArmorWorn { target } => {
            if !entity_exists(*target)
                || campaign
                    .rules
                    .as_ref()
                    .and_then(|rules| rules.tactical_inventory.as_ref())
                    .and_then(|inventory| inventory.loadout(*target))
                    .is_none_or(|loadout| loadout.worn_armor.is_none())
            {
                return Err(invalid("armor observation requires actual worn armor"));
            }
        }
        EffectObservation::Zone {
            effect,
            target,
            contact,
        } => {
            if !result.next_effects.pending.is_empty() {
                return Err(invalid(
                    "resolve pending effects before changing zone membership",
                ));
            }
            if !entity_exists(*target) {
                return Err(invalid("zone observation references an unknown target"));
            }
            let record = result
                .next_effects
                .effects
                .iter_mut()
                .find(|e| e.id == *effect)
                .ok_or_else(|| invalid("zone observation references an unknown effect"))?;
            let TacticalEffectTarget::Zone { occupants } = &mut record.target else {
                return Err(invalid("zone observation references a creature effect"));
            };
            if *contact != ZoneContact::CreatureLeaves {
                if occupants.contains(target) {
                    return Err(invalid("target already occupies the zone"));
                }
                occupants.push(*target);
            } else if !occupants.contains(target) {
                return Err(invalid("target does not occupy the zone"));
            }
            // A leave trigger observes the old membership; remove it after delivery below.
        }
        EffectObservation::Time => {
            if !result.next_effects.pending.is_empty() {
                return Err(invalid("resolve pending effects before advancing time"));
            }
        }
    }
    // SRD187: observation records due consequences; the current turn's controller chooses
    // their order. Neither vector order nor an implicit expiry-first policy decides it.
    let groups: Vec<_> = result
        .next_effects
        .groups
        .iter_mut()
        .filter_map(|g| {
            expiry_due(&mut g.expires, observation, campaign.clock.now).then_some(g.clone())
        })
        .collect();
    for group in groups {
        queue(
            result,
            ticket(
                group.id,
                &group.source,
                group.source.actor,
                None,
                EffectTriggerPayload::ExpireConcentrationGroup,
                observation,
                stamp,
            ),
        )?;
    }
    let due: Vec<_> = result
        .next_effects
        .effects
        .iter_mut()
        .filter_map(|e| {
            expiry_due(&mut e.expires, observation, campaign.clock.now).then_some(e.clone())
        })
        .collect();
    for effect in due {
        let target = match effect.target {
            TacticalEffectTarget::Creature(id) => id,
            TacticalEffectTarget::Zone { .. } => effect.source.actor,
        };
        queue(
            result,
            ticket(
                effect.id,
                &effect.source,
                target,
                None,
                EffectTriggerPayload::ExpireTargetEffect,
                observation,
                stamp,
            ),
        )?;
    }
    let records = result.next_effects.effects.clone();
    for effect in records {
        for (index, rule) in effect.triggers.iter().enumerate() {
            for target in effect_trigger_targets(&effect, rule, observation) {
                if matches!(
                    rule.frequency,
                    EffectTriggerFrequency::OncePerTargetPerTurn { .. }
                ) && result.next_effects.turn.is_none()
                {
                    return Err(invalid(
                        "per-turn trigger lacks an authoritative turn cursor",
                    ));
                }
                queue(
                    result,
                    ticket(
                        effect.id,
                        &effect.source,
                        target,
                        Some(index as u16),
                        rule.payload.clone(),
                        observation,
                        stamp,
                    ),
                )?;
            }
        }
    }
    if let EffectObservation::Zone {
        effect,
        target,
        contact: ZoneContact::CreatureLeaves,
    } = observation
        && let Some(record) = result
            .next_effects
            .effects
            .iter_mut()
            .find(|e| e.id == *effect)
        && let TacticalEffectTarget::Zone { occupants } = &mut record.target
    {
        occupants.retain(|id| id != target);
    }
    Ok(())
}

/// Query immediately before the root constructs a roll/damage continuation. Re-query
/// after another selected simultaneous consequence, since suppression can change.
pub fn trigger_is_applicable(
    effects: &TacticalEffects,
    ticket: &ScheduledEffectTrigger,
) -> Result<bool, EffectLifecycleError> {
    if !effects.pending.contains(ticket) {
        return Err(invalid("unknown or altered pending effect ticket"));
    }
    if matches!(
        ticket.payload,
        EffectTriggerPayload::ExpireConcentrationGroup | EffectTriggerPayload::ExpireTargetEffect
    ) {
        return Ok(true);
    }
    let effect = effects
        .effects
        .iter()
        .find(|e| e.id == ticket.effect)
        .ok_or_else(|| invalid("pending trigger has no effect"))?;
    let rule = ticket
        .rule_index
        .and_then(|i| effect.triggers.get(usize::from(i)))
        .ok_or_else(|| invalid("pending trigger has no rule"))?;
    if !matches!(
        ticket.payload,
        EffectTriggerPayload::EndTargetEffect | EffectTriggerPayload::EndConcentrationGroup
    ) && !not_suppressed(effects, effect, ticket.target)
    {
        return Ok(false);
    }
    if let EffectTriggerFrequency::OncePerTargetPerTurn { key } = &rule.frequency {
        if effects.turn.is_none() {
            return Err(invalid("per-turn trigger lacks a turn cursor"));
        }
        if effects
            .trigger_uses
            .iter()
            .any(|u| u.effect == effect.id && u.target == ticket.target && u.key == *key)
        {
            return Ok(false);
        }
    }
    Ok(true)
}

fn resolve_trigger(
    result: &mut EffectLifecycleTransition,
    id: EffectTicketId,
    outcome: EffectTriggerResolution,
    stamp: &EffectOperationStamp,
) -> Result<(), EffectLifecycleError> {
    let index = result
        .next_effects
        .pending
        .iter()
        .position(|t| t.id == id)
        .ok_or_else(|| invalid("unknown or already resolved effect ticket"))?;
    let pending = result.next_effects.pending[index].clone();
    let applies = trigger_is_applicable(&result.next_effects, &pending)?;
    if applies == (outcome == EffectTriggerResolution::SkipInactive) {
        return Err(invalid(
            "ticket resolution does not match current applicability",
        ));
    }
    if applies
        && (matches!(pending.payload, EffectTriggerPayload::SavingThrow { .. })
            != matches!(outcome, EffectTriggerResolution::SavingThrow { .. }))
    {
        return Err(invalid("ticket resolution has the wrong result kind"));
    }
    result.next_effects.pending.remove(index);
    result.selected_trigger_applies = Some(applies);
    if !applies {
        return Ok(());
    }
    if let Some(effect) = result
        .next_effects
        .effects
        .iter()
        .find(|e| e.id == pending.effect)
        && let Some(rule) = pending
            .rule_index
            .and_then(|i| effect.triggers.get(usize::from(i)))
        && let EffectTriggerFrequency::OncePerTargetPerTurn { key } = &rule.frequency
    {
        let turn = result.next_effects.turn.expect("validated turn frequency");
        result.next_effects.trigger_uses.push(EffectTriggerUse {
            effect: effect.id,
            key: key.clone(),
            target: pending.target,
            turn_actor: turn.actor,
            turn_number: turn.number,
            origin: stamp.clone(),
        });
    }
    match pending.payload {
        EffectTriggerPayload::ExpireTargetEffect => {
            remove_effect(result, pending.effect, EffectEndReason::Expired, true)
        }
        EffectTriggerPayload::ExpireConcentrationGroup => {
            remove_group(result, pending.effect, EffectEndReason::Expired)
        }
        EffectTriggerPayload::EndTargetEffect => {
            remove_effect(result, pending.effect, EffectEndReason::SourceTrigger, true)
        }
        EffectTriggerPayload::EndConcentrationGroup => {
            let group = result
                .next_effects
                .effects
                .iter()
                .find(|e| e.id == pending.effect)
                .and_then(|e| e.concentration_group)
                .ok_or_else(|| invalid("group-ending trigger lacks a group"))?;
            remove_group(result, group, EffectEndReason::SourceTrigger);
        }
        EffectTriggerPayload::SavingThrow {
            on_success,
            on_failure,
            ..
        } => {
            let EffectTriggerResolution::SavingThrow { success } = outcome else {
                unreachable!("checked result kind")
            };
            match if success { on_success } else { on_failure } {
                EffectSaveEnd::None => {}
                EffectSaveEnd::TargetEffect => {
                    remove_effect(result, pending.effect, EffectEndReason::Saved, true)
                }
                EffectSaveEnd::ConcentrationGroup => {
                    let group = result
                        .next_effects
                        .effects
                        .iter()
                        .find(|e| e.id == pending.effect)
                        .and_then(|e| e.concentration_group)
                        .ok_or_else(|| invalid("save result lacks its concentration group"))?;
                    remove_group(result, group, EffectEndReason::Saved);
                }
            }
        }
        EffectTriggerPayload::Damage { .. } => {}
    }
    Ok(())
}

pub fn apply_effect_lifecycle(
    campaign: &CampaignState,
    current: &TacticalEffects,
    meta: &CommandMeta,
    action: &EffectLifecycleAction,
) -> Result<EffectLifecycleTransition, EffectLifecycleError> {
    current.validate(campaign).map_err(invalid)?;
    if meta.campaign_id != campaign.campaign_id()
        || meta.expected_event_sequence != campaign.applied_event_sequence
    {
        return Err(invalid("foreign campaign or stale effect operation"));
    }
    if let Some(last) = &current.last_operation
        && (meta.expected_event_sequence < last.command.expected_event_sequence
            || (meta.expected_event_sequence == last.command.expected_event_sequence
                && (meta != &last.command || action.step <= last.step)))
    {
        return Err(invalid("duplicate or out-of-order effect operation"));
    }
    let stamp = EffectOperationStamp {
        command: meta.clone(),
        step: action.step,
    };
    let mut result = EffectLifecycleTransition {
        next_effects: current.clone(),
        ended: Vec::new(),
        triggers: Vec::new(),
        selected_trigger_applies: None,
    };
    result.next_effects.last_operation = Some(stamp.clone());
    match &action.operation {
        EffectLifecycleOperation::BeginConcentration { group } => {
            if group.stage != ConcentrationStage::Casting || group.source.command != *meta {
                return Err(invalid(
                    "new concentration must begin casting with this command",
                ));
            }
            if result.next_effects.groups.iter().any(|g| g.id == group.id)
                || result.next_effects.effects.iter().any(|e| e.id == group.id)
            {
                return Err(invalid("concentration identity is already in use"));
            }
            if let Some(previous) = result.next_effects.group_for_owner(group.source.actor) {
                let id = previous.id;
                remove_group(&mut result, id, EffectEndReason::ConcentrationReplaced);
            }
            result.next_effects.groups.push(group.clone());
        }
        EffectLifecycleOperation::SetCastingDuration {
            group,
            source,
            expires,
        } => {
            if result
                .next_effects
                .pending
                .iter()
                .any(|ticket| ticket.effect == *group)
            {
                return Err(invalid("casting group has an unresolved expiry"));
            }
            let casting = result
                .next_effects
                .groups
                .iter_mut()
                .find(|g| g.id == *group)
                .ok_or_else(|| invalid("casting duration references an unknown group"))?;
            if casting.stage != ConcentrationStage::Casting || casting.source != *source {
                return Err(invalid(
                    "casting duration differs from its unactivated source group",
                ));
            }
            if matches!(casting.expires, TacticalEffectExpiry::AtTime(at) if at <= campaign.clock.now)
            {
                return Err(invalid("expired casting cannot receive a new duration"));
            }
            if matches!(expires, TacticalEffectExpiry::Never)
                || matches!(expires, TacticalEffectExpiry::AtTime(at) if *at <= campaign.clock.now)
            {
                return Err(invalid("casting duration must have a future source expiry"));
            }
            casting.expires = expires.clone();
        }
        EffectLifecycleOperation::Install { effects } => {
            if effects.is_empty() {
                return Err(invalid("empty effect installation"));
            }
            for effect in effects {
                if effect.established_at.is_some() {
                    return Err(invalid(
                        "installation proposal must not supply an accepted installation stamp",
                    ));
                }
                if let Some(id) = effect.concentration_group {
                    let group = result
                        .next_effects
                        .groups
                        .iter_mut()
                        .find(|g| g.id == id)
                        .ok_or_else(|| {
                            invalid("effect installation references an unknown concentration group")
                        })?;
                    group.stage = ConcentrationStage::Active;
                }
                let mut installed = effect.clone();
                installed.established_at = Some(stamp.clone());
                result.next_effects.effects.push(installed);
            }
        }
        EffectLifecycleOperation::EndEffect { effect, reason } => {
            if !result.next_effects.effects.iter().any(|e| e.id == *effect) {
                return Err(invalid("cannot end an unknown effect"));
            }
            remove_effect(&mut result, *effect, *reason, true);
        }
        EffectLifecycleOperation::EndConcentration { owner, reason } => {
            let id = result
                .next_effects
                .group_for_owner(*owner)
                .ok_or_else(|| invalid("owner has no concentration group"))?
                .id;
            remove_group(&mut result, id, *reason);
        }
        EffectLifecycleOperation::Observe(observation) => {
            observe(campaign, &mut result, observation, &stamp)?
        }
        EffectLifecycleOperation::LeaveCombat => {
            if campaign
                .rules
                .as_ref()
                .is_some_and(|rules| rules.timing.is_some())
                || !result.next_effects.pending.is_empty()
            {
                return Err(invalid(
                    "initiative and pending effects must finish before leaving combat",
                ));
            }
            result.next_effects.turn = None;
            result.next_effects.trigger_uses.clear();
        }
        EffectLifecycleOperation::ResolveTrigger { ticket, outcome } => {
            resolve_trigger(&mut result, *ticket, *outcome, &stamp)?
        }
    }
    result.next_effects.validate(campaign).map_err(invalid)?;
    Ok(result)
}
