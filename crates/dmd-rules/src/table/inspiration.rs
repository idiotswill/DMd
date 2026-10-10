//! An explicit Host ruling is the source of an award; transport activation is not.
use super::*;

/// Pure provenance shared with the original-history outer-action join.
pub fn award_ruling(reason: &str) -> Ruling {
    Ruling {
        basis: RulingBasis::Srd { page: 8 },
        reason: reason.into(),
    }
}

pub(super) fn award(
    state: &CampaignState,
    next: &mut CampaignState,
    meta: &CommandMeta,
    character_id: CharacterId,
    reason: &str,
) -> Result<(), String> {
    let table = state.table.as_ref().ok_or("This campaign has no table.")?;
    if meta.issuer != CommandIssuer::Admin
        || meta.actor.is_some()
        || meta.campaign_id != state.campaign_id()
        || meta.expected_event_sequence != state.applied_event_sequence
        || table
            .active_session
            .as_ref()
            .is_none_or(|session| meta.session_id != Some(session.session_id))
    {
        return Err("An Inspiration award requires the current Host and session.".into());
    }
    if !super::grapple_transport_enabled(state) {
        return Err("Enable dragging and Inspiration controls before awarding Inspiration.".into());
    }
    super::source_control::settled(state)?;
    let character = state
        .characters
        .get(&character_id)
        .filter(|character| {
            character.status == CharacterStatus::Active
                && character.campaign_id == state.campaign_id()
                && character
                    .controlling_player_id
                    .is_some_and(|player| state.players.contains_key(&player))
                && table
                    .character_profiles
                    .get(&character_id)
                    .is_some_and(|profile| profile.entity_id == character.entity_id)
        })
        .ok_or(
            "Select an active player character with an assigned player and source-created sheet.",
        )?;
    let rules = next
        .rules
        .as_mut()
        .ok_or("Character mechanics are absent.")?;
    crate::kernel::grant_inspiration(rules, meta, character.entity_id, &award_ruling(reason))
        .map_err(|error| error.to_string())?;
    Ok(())
}

pub(super) fn award_excess(
    state: &CampaignState,
    next: &mut CampaignState,
    meta: &CommandMeta,
    character_id: CharacterId,
    reason: &str,
) -> Result<(), String> {
    let table = state.table.as_ref().ok_or("This campaign has no table.")?;
    let session = table
        .active_session
        .as_ref()
        .ok_or("An excess award requires an active session.")?;
    if meta.issuer != CommandIssuer::Admin
        || meta.actor.is_some()
        || meta.campaign_id != state.campaign_id()
        || meta.expected_event_sequence != state.applied_event_sequence
        || meta.session_id != Some(session.session_id)
        || !super::grapple_transport_enabled(state)
    {
        return Err("An excess award requires the current Host and table controls.".into());
    }
    super::source_control::settled(state)?;
    let character = eligible_character(state, character_id)
        .ok_or("Select a living source-created player character.")?;
    let player = character
        .controlling_player_id
        .ok_or("The character has no controller.")?;
    table.validate_attendance(session.session_id, player, character_id)?;
    let rules = next
        .rules
        .as_mut()
        .ok_or("Character mechanics are absent.")?;
    if rules.entities.values().any(|entity| {
        entity
            .character_features
            .as_ref()
            .is_some_and(|features| features.inspiration_transfer_pending)
    }) {
        return Err("Finish the existing Inspiration choice before another award.".into());
    }
    crate::kernel::begin_host_inspiration_transfer(
        rules,
        meta,
        character.entity_id,
        &award_ruling(reason),
    )
    .map_err(|error| error.to_string())?;
    next.table.as_mut().unwrap().inspiration_transfer = Some(TableInspirationTransfer {
        origin: meta.clone(),
        character_id,
    });
    Ok(())
}

fn eligible_character(state: &CampaignState, id: CharacterId) -> Option<&Character> {
    let character = state.characters.get(&id)?;
    (character.status == CharacterStatus::Active
        && character.campaign_id == state.campaign_id()
        && character
            .controlling_player_id
            .is_some_and(|player| state.players.contains_key(&player))
        && state
            .table
            .as_ref()?
            .character_profiles
            .get(&id)
            .is_some_and(|profile| profile.entity_id == character.entity_id)
        && state
            .rules
            .as_ref()?
            .entities
            .get(&character.entity_id)
            .is_some_and(|entity| !entity.death.dead))
    .then_some(character)
}

/// Only the attending owner learns the minimum eligibility needed for this gift.
pub fn transfer_choices(
    state: &CampaignState,
    issuer: CommandIssuer,
) -> Result<Vec<TableInspirationTransferChoice>, String> {
    let Some(table) = &state.table else {
        return Ok(vec![]);
    };
    let Some(transfer) = &table.inspiration_transfer else {
        return Ok(vec![]);
    };
    transfer.validate(state)?;
    let character = &state.characters[&transfer.character_id];
    if character.controlling_player_id.map(CommandIssuer::Player) != Some(issuer) {
        return Ok(vec![]);
    }
    let mut recipients = state
        .characters
        .keys()
        .copied()
        .filter(|id| *id != transfer.character_id)
        .filter(|id| {
            eligible_character(state, *id).is_some_and(|c| {
                !state.rules.as_ref().unwrap().entities[&c.entity_id].heroic_inspiration
            })
        })
        .collect::<Vec<_>>();
    recipients.sort_by_key(|id| id.0);
    let mut choices = recipients
        .into_iter()
        .map(|recipient| TableInspirationTransferChoice {
            award: transfer.origin.id,
            character_id: transfer.character_id,
            recipient: Some(recipient),
        })
        .collect::<Vec<_>>();
    choices.push(TableInspirationTransferChoice {
        award: transfer.origin.id,
        character_id: transfer.character_id,
        recipient: None,
    });
    Ok(choices)
}

pub(super) fn resolve_transfer(
    state: &CampaignState,
    next: &mut CampaignState,
    meta: &CommandMeta,
    choice: &TableInspirationTransferChoice,
) -> Result<(), String> {
    let (_, character, actor, _) = super::reducer::player_channel(state, meta)?;
    if character != choice.character_id
        || !super::grapple_transport_enabled(state)
        || !transfer_choices(state, meta.issuer)?.contains(choice)
    {
        return Err("Select your current Inspiration choice.".into());
    }
    let recipient = choice.recipient.map(|id| state.characters[&id].entity_id);
    crate::kernel::resolve_inspiration_transfer(
        state,
        next.rules
            .as_mut()
            .ok_or("Character mechanics are absent.")?,
        meta,
        actor,
        recipient,
    )
    .map_err(|error| error.to_string())?;
    next.table.as_mut().unwrap().inspiration_transfer = None;
    Ok(())
}
