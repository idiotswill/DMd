//! Current advisory choices, separate from immutable table presentation and commands.
use super::*;
use crate::tactical::grapple::execution::ReadContext;

pub(super) fn features(
    read: &ReadContext<'_>,
    issuer: CommandIssuer,
    actor: EntityId,
) -> Result<Vec<(String, String)>, String> {
    read.require_guarded("Intrinsic choices require original table history.")
        .map_err(|error| error.to_string())?;
    let state = read.state();
    let unavailable = "Select the current source actor and finish pending decisions first.";
    let table = state.table.as_ref().ok_or(unavailable)?;
    let session = table.active_session.as_ref().ok_or(unavailable)?;
    let rules = state.rules.as_ref().ok_or(unavailable)?;
    let encounter = state.encounter.as_ref().ok_or(unavailable)?;
    let flow = encounter.flow.as_ref().ok_or(unavailable)?;
    let runtime = rules
        .tactical_creatures
        .as_ref()
        .and_then(|creatures| creatures.runtime(actor))
        .ok_or(unavailable)?;
    let authorized = match issuer {
        CommandIssuer::Admin => {
            !source_control::enabled(state)
                || !matches!(runtime.controller, CreatureController::Player(_))
        }
        CommandIssuer::Player(player) => {
            source_control::owns_source(state, player, actor)
                && session.participants.iter().any(|participant| {
                    participant.player_id == player
                        && participant.attendance == AttendanceStatus::Present
                })
        }
        _ => false,
    };
    if !authorized
        || flow.phase != TacticalPhase::Active
        || flow.version != TacticalExecutionVersion::EncounterReleaseV1.flow_version()
        || flow.resolution.is_some()
        || table.pending.is_some()
        || table.roll_context.is_some()
        || table.inspiration_transfer.is_some()
        || rules.pending.is_some()
        || rules.timing.as_ref().is_none_or(|timing| {
            timing
                .order
                .get(timing.index)
                .is_none_or(|turn| turn.actor != actor)
        })
        || encounter.participant(actor).is_none()
    {
        return Err(unavailable.into());
    }
    crate::tactical_creatures::intrinsic_action_features(state, actor)
        .map(|features| {
            features
                .into_iter()
                .map(|feature| (feature.id.clone(), feature.name.clone()))
                .collect()
        })
        // This read never exposes another actor's private source-work failure.
        .map_err(|_| unavailable.into())
}
