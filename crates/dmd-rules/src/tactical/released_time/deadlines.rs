use super::*;

pub(super) fn preflight(state: &CampaignState) -> Result<(), RulesError> {
    let rules = state.rules.as_ref().ok_or(RulesError::Uninitialized)?;
    if !rules.rests.is_empty()
        || rules.tactical_recovery.as_ref().is_some_and(|rows| {
            rows.values()
                .any(|r| r.knockout.is_some() || r.knockout_rest.is_some())
        })
    {
        return Err(prerequisite(
            "released rest and knockout completion are not admitted",
        ));
    }
    if rules.effects.iter().any(|e| e.condition.is_some())
        || rules
            .tactical_effects
            .as_ref()
            .is_some_and(|effects| effects.effects.iter().any(|e| !e.conditions.is_empty()))
    {
        return Err(prerequisite(
            "released condition transitions require their supported custody and physical lifecycle",
        ));
    }
    // Source support is read from actual geometry; no synthetic ground actor or
    // flight cadence is installed. Stable Unconscious is deliberately permitted.
    let current = encounter(state)?;
    for participant in &current.participants {
        let body = participant.volume().map_err(|e| invalid(&e))?;
        let contacts_top = |surface: SpatialBox| {
            surface.max.z == participant.position.z
                && body.min.x < surface.max.x
                && body.max.x > surface.min.x
                && body.min.y < surface.max.y
                && body.max.y > surface.min.y
        };
        let supported = participant.position.z == current.battlefield.floor_z
            || current
                .battlefield
                .obstacles
                .iter()
                .any(|obstacle| obstacle.blocks_movement && contacts_top(obstacle.volume))
            || current.battlefield.terrain.iter().any(|terrain| {
                (terrain.supports_top || terrain.burrowable) && contacts_top(terrain.volume)
            });
        let intersects_liquid = current
            .battlefield
            .terrain
            .iter()
            .any(|terrain| terrain.water && body.intersects(terrain.volume));
        // None also means swimming/submersion to the shared falling reader.
        // This bounded producer admits actual floor/solid-top support only;
        // it does not adjudicate elapsed liquid/underwater physical obligations.
        if !supported
            || intersects_liquid
            || crate::spatial::fall_destination(current, participant.entity_id)
                .map_err(|e| invalid(&e.to_string()))?
                .is_some()
            || crate::spatial::flight_loss_fall(encounter(state)?, state, participant.entity_id)
                .map_err(|e| invalid(&e.to_string()))?
                .is_some()
        {
            return Err(prerequisite(
                "released time requires settled ground-supported participants",
            ));
        }
    }
    Ok(())
}

/// Shape survives source removal. Replay must still authenticate which source
/// actually occupied this ticket; no retained manifest authenticates itself.
pub(super) fn binding_shape(
    batch: &ReleasedDeadlineBatch,
    binding: &ReleasedDeadlineBinding,
    effect_ordinal: u16,
) -> bool {
    match (&binding.source, &binding.work.kind) {
        (
            ReleasedDeadlineSource::Effect { .. } | ReleasedDeadlineSource::Group { .. },
            TacticalWorkKind::Effect { ticket },
        ) => {
            *ticket
                == EffectTicketId {
                    command: batch.observed.command.id,
                    step: batch.observed.step,
                    ordinal: effect_ordinal,
                }
        }
        (
            ReleasedDeadlineSource::Legacy { effect, .. },
            TacticalWorkKind::ExpireLegacyEffect { effect: id },
        ) => effect == id,
        (
            ReleasedDeadlineSource::Stable { actor, .. },
            TacticalWorkKind::RecoverStable { actor: id },
        ) => actor == id,
        _ => false,
    }
}

pub(super) fn inventory(
    state: &CampaignState,
) -> Result<Vec<(WorldInstant, ReleasedDeadlineSource)>, RulesError> {
    let rules = state.rules.as_ref().ok_or(RulesError::Uninitialized)?;
    let mut sources = Vec::new();
    if let Some(effects) = &rules.tactical_effects {
        for group in &effects.groups {
            if let TacticalEffectExpiry::AtTime(at) = group.expires {
                sources.push((
                    at,
                    ReleasedDeadlineSource::Group {
                        group: group.id,
                        source: group.source.clone(),
                    },
                ));
            }
        }
        for effect in &effects.effects {
            if let TacticalEffectExpiry::AtTime(at) = effect.expires {
                sources.push((
                    at,
                    ReleasedDeadlineSource::Effect {
                        effect: effect.id,
                        source: effect.source.clone(),
                        established_at: effect.established_at.clone().ok_or_else(|| {
                            invalid("timed effect lacks its real establishment stamp")
                        })?,
                        group: effect.concentration_group,
                    },
                ));
            }
        }
    }
    for effect in &rules.effects {
        if let Expiry::AtTime(at) = effect.expires {
            sources.push((
                at,
                ReleasedDeadlineSource::Legacy {
                    effect: effect.id,
                    source: effect.source,
                    target: effect.target,
                    concentration_owner: effect.concentration_owner,
                },
            ));
        }
    }
    if let Some(recoveries) = &rules.tactical_recovery {
        let mut ordered = recoveries.iter().collect::<Vec<_>>();
        ordered.sort_by_key(|(actor, _)| actor.0);
        for (actor, recovery) in ordered {
            if let Some(stable) = &recovery.stable {
                let at = crate::tactical_damage::stable_wake_at(stable)
                    .map_err(|e| invalid(&e.to_string()))?
                    .ok_or_else(|| invalid("stable wake lacks accepted delay"))?;
                let roll = stable
                    .delay_roll
                    .as_ref()
                    .ok_or_else(|| invalid("stable delay absent"))?;
                if rules.cancelled_roll_ids.contains(&roll.request_id)
                    || !rules.rolls.iter().any(|accepted| accepted.result == *roll
                        && accepted.issued_by.expected_event_sequence >= stable.origin.command.expected_event_sequence
                        && matches!(accepted.purpose, PendingPurpose::TacticalResolution { key, .. } if key.subject == *actor && key.role == TacticalRollRole::StableRecovery && key.request_id() == roll.request_id))
                {
                    return Err(invalid(
                        "stable delay is not an accepted non-cancelled raw roll",
                    ));
                }
                sources.push((
                    at,
                    ReleasedDeadlineSource::Stable {
                        actor: *actor,
                        origin: stable.origin.clone(),
                        stabilized_at: stable.stabilized_at,
                        roll: roll.request_id,
                    },
                ));
            }
        }
    }
    Ok(sources)
}

pub(super) fn source_present(
    state: &CampaignState,
    binding: &ReleasedDeadlineBinding,
) -> Result<bool, RulesError> {
    Ok(inventory(state)?
        .iter()
        .any(|(at, source)| *at == binding.due_at && *source == binding.source)
        && work_matches(state, &binding.source, &binding.work)?)
}

pub(super) fn work_matches(
    state: &CampaignState,
    source: &ReleasedDeadlineSource,
    work: &TacticalWorkItem,
) -> Result<bool, RulesError> {
    Ok(match (source, &work.kind) {
        (ReleasedDeadlineSource::Group { group, source }, TacticalWorkKind::Effect { ticket }) => {
            effects(state)?.pending.iter().any(|t| {
                t.id == *ticket
                    && t.effect == *group
                    && t.source == *source
                    && t.payload == EffectTriggerPayload::ExpireConcentrationGroup
                    && t.cause == EffectObservation::Time
                    && t.rule_index.is_none()
            })
        }
        (
            ReleasedDeadlineSource::Effect { effect, source, .. },
            TacticalWorkKind::Effect { ticket },
        ) => effects(state)?.pending.iter().any(|t| {
            t.id == *ticket
                && t.effect == *effect
                && t.source == *source
                && t.payload == EffectTriggerPayload::ExpireTargetEffect
                && t.cause == EffectObservation::Time
                && t.rule_index.is_none()
        }),
        (
            ReleasedDeadlineSource::Legacy { effect, .. },
            TacticalWorkKind::ExpireLegacyEffect { effect: id },
        ) => effect == id,
        (
            ReleasedDeadlineSource::Stable { actor, .. },
            TacticalWorkKind::RecoverStable { actor: id },
        ) => actor == id,
        _ => false,
    })
}

pub(super) fn cleanup_cancels(
    applied: &ReleasedDeadlineSource,
    cancelled: &ReleasedDeadlineSource,
) -> bool {
    match (applied, cancelled) {
        (
            ReleasedDeadlineSource::Group { group, .. },
            ReleasedDeadlineSource::Effect {
                group: Some(owner), ..
            },
        ) => group == owner,
        // Expiring the final member retires its otherwise empty concentration
        // group through the same effect reducer and removes that pending ticket.
        (
            ReleasedDeadlineSource::Effect {
                group: Some(owner), ..
            },
            ReleasedDeadlineSource::Group { group, .. },
        ) => owner == group,
        _ => false,
    }
}
