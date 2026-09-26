//! Explicit host conclusion without erasing any source timing or consequence.
use super::*;

fn ruling_shape(ruling: &str) -> Result<(), RulesError> {
    if ruling.trim().is_empty() || ruling.len() > 2_000 || ruling != ruling.trim() {
        return Err(invalid(
            "aftermath ruling must be trimmed and contain 1–2000 bytes",
        ));
    }
    Ok(())
}

/// Quiescence is not absence of lasting/relative effects or paid Ready. Those
/// continue under the existing cadence. Only work already awaiting resolution
/// prevents a session boundary; no branch below mutates or resolves that work.
fn require_quiescent(state: &CampaignState) -> Result<(), RulesError> {
    let f = flow(state)?;
    let rules = state.rules.as_ref().ok_or(RulesError::Uninitialized)?;
    if f.phase != TacticalPhase::Active
        || f.resolution.is_some()
        || rules.pending.is_some()
        || rules
            .tactical_effects
            .as_ref()
            .is_some_and(|effects| !effects.pending.is_empty())
        || rules.tactical_creatures.as_ref().is_some_and(|creatures| {
            creatures.runtime.iter().any(|runtime| {
                runtime.routine.is_some()
                    || runtime.recharge.iter().any(|row| row.pending.is_some())
            })
        })
    {
        return Err(prerequisite(
            "finish pending encounter decisions before concluding or pausing aftermath",
        ));
    }
    super::falling::require_settled_before_action(state)
}

pub(super) fn conclude(
    state: &mut CampaignState,
    meta: &CommandMeta,
    cadence: AftermathCadence,
    ruling: &str,
) -> Result<(), RulesError> {
    privileged(meta)?;
    ruling_shape(ruling)?;
    require_quiescent(state)?;
    if flow(state)?.aftermath.is_some() {
        return Err(prerequisite(
            "hostilities were already concluded on this retained cadence",
        ));
    }
    let timing = state
        .rules
        .as_ref()
        .and_then(|rules| rules.timing.as_ref())
        .ok_or_else(|| invalid("aftermath requires existing initiative timing"))?;
    let concluded_on_turn = EffectTurn {
        actor: turns::active(state)?,
        number: timing.turn_number,
        boundary: TurnBoundary::Start,
    };
    let record = TacticalAftermath {
        origin: meta.clone(),
        cadence,
        ruling: ruling.into(),
        concluded_at: state.clock.now,
        concluded_on_turn,
    };
    flow_mut(state)?.aftermath = Some(Box::new(record));
    Ok(())
}

pub(super) fn validate(state: &CampaignState) -> Result<(), RulesError> {
    let f = flow(state)?;
    let Some(record) = &f.aftermath else {
        return Ok(());
    };
    privileged(&record.origin)?;
    ruling_shape(&record.ruling)?;
    let timing = state
        .rules
        .as_ref()
        .and_then(|rules| rules.timing.as_ref())
        .ok_or_else(|| invalid("aftermath lost its retained cadence"))?;
    validate_equipment_change_origin(state, &record.origin, record.concluded_on_turn.actor)
        .map_err(|error| invalid(&error))?;
    if !matches!(
        TacticalExecutionVersion::from_flow_version(f.version),
        Some(TacticalExecutionVersion::ReactionsV1 | TacticalExecutionVersion::ShieldHitV1)
    ) || f.phase != TacticalPhase::Active
        || record.origin.expected_event_sequence <= f.origin.expected_event_sequence
        || record.concluded_at > state.clock.now
        || record.concluded_on_turn.number == 0
        || record.concluded_on_turn.number > timing.turn_number
        || record.concluded_on_turn.boundary != TurnBoundary::Start
        || !timing
            .order
            .iter()
            .any(|entry| entry.actor == record.concluded_on_turn.actor)
    {
        return Err(invalid(
            "aftermath conclusion differs from its retained timing or origin",
        ));
    }
    if record.origin.expected_event_sequence == state.applied_event_sequence
        && (record.concluded_at != state.clock.now
            || record.concluded_on_turn.number != timing.turn_number
            || record.concluded_on_turn.actor != turns::active(state)?)
    {
        return Err(invalid(
            "new aftermath conclusion has a different current boundary",
        ));
    }
    Ok(())
}

/// The application uses the same rule-level boundary before closing a session.
/// This neither ends timing nor authorizes a different encounter or battlefield.
pub fn require_aftermath_session_boundary(state: &CampaignState) -> Result<(), RulesError> {
    validate(state)?;
    if flow(state)?.aftermath.is_none() {
        return Err(prerequisite(
            "conclude hostilities before pausing the encounter session",
        ));
    }
    require_quiescent(state)
}
