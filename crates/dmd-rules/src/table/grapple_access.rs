//! The explicit table activation derives all provenance from the accepted command.
use super::*;

pub(super) fn activate(
    state: &CampaignState,
    meta: &CommandMeta,
) -> Result<TableGrappleAccess, String> {
    let table = state.table.as_ref().ok_or("This campaign has no table.")?;
    let session = table
        .active_session
        .as_ref()
        .ok_or("Start the current table session first.")?;
    if meta.issuer != CommandIssuer::Admin
        || meta.actor.is_some()
        || meta.campaign_id != state.campaign_id()
        || meta.expected_event_sequence != state.applied_event_sequence
        || meta.session_id != Some(session.session_id)
    {
        return Err("Grapple activation requires the current host and session.".into());
    }
    if table.grapple_access.is_some() || has_unimplemented_grapple_records(state) {
        return Err(
            "Grapple activation requires a table without earlier Grapple authority.".into(),
        );
    }
    source_control::settled(state)?;
    let flow = state
        .encounter
        .as_ref()
        .and_then(|encounter| encounter.flow.as_ref())
        .ok_or("Start an encounter before enabling Grapple.")?;
    if flow.version != TacticalExecutionVersion::EncounterReleaseV1.flow_version()
        || flow.phase != TacticalPhase::Active
    {
        return Err("Enable Grapple at a settled current encounter turn.".into());
    }
    Ok(TableGrappleAccess {
        version: TableGrappleAccessVersion::OrdinaryGrappleV1,
        origin: meta.clone(),
    })
}
