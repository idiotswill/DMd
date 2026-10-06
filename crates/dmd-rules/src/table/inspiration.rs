//! An explicit Host ruling is the source of an award; transport activation is not.
use super::*;

/// Pure provenance shared with the original-history outer-action join.
pub fn award_ruling(reason: &str) -> Ruling {
    Ruling { basis: RulingBasis::Srd { page: 8 }, reason: reason.into() }
}

pub(super) fn award(
    state: &CampaignState,
    next: &mut CampaignState,
    meta: &CommandMeta,
    character_id: CharacterId,
    reason: &str,
) -> Result<(), String> {
    let table = state.table.as_ref().ok_or("This campaign has no table.")?;
    if meta.issuer != CommandIssuer::Admin || meta.actor.is_some()
        || meta.campaign_id != state.campaign_id()
        || meta.expected_event_sequence != state.applied_event_sequence
        || table.active_session.as_ref().is_none_or(|session| meta.session_id != Some(session.session_id)) {
        return Err("An Inspiration award requires the current Host and session.".into());
    }
    if !super::grapple_transport_enabled(state) {
        return Err("Enable dragging and Inspiration controls before awarding Inspiration.".into());
    }
    super::source_control::settled(state)?;
    let character = state.characters.get(&character_id).filter(|character| {
        character.status == CharacterStatus::Active
            && character.campaign_id == state.campaign_id()
            && character.controlling_player_id.is_some_and(|player| state.players.contains_key(&player))
            && table.character_profiles.get(&character_id).is_some_and(|profile| profile.entity_id == character.entity_id)
    }).ok_or("Select an active player character with an assigned player and source-created sheet.")?;
    let rules = next.rules.as_mut().ok_or("Character mechanics are absent.")?;
    crate::kernel::grant_inspiration(rules, meta, character.entity_id, &award_ruling(reason))
        .map_err(|error| error.to_string())?;
    Ok(())
}
