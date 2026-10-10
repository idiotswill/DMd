use crate::{RulesPack, tactical_inventory::*};
use dmd_domain::*;

pub(crate) fn prepare(
    state: &CampaignState,
    meta: &CommandMeta,
    character_id: CharacterId,
    ids: &[ItemId],
    pack: &RulesPack,
) -> Result<CampaignState, String> {
    let current = state
        .rules
        .as_ref()
        .ok_or("No mechanical state.")?
        .tactical_inventory
        .clone()
        .unwrap_or_default();
    let transition = materialize_starting_equipment(state, &current, meta, character_id, ids, pack)
        .map_err(|error| error.to_string())?;
    let mut next = transition.next_state;
    next.rules
        .as_mut()
        .ok_or("No mechanical state.")?
        .tactical_inventory = Some(transition.next_inventory);
    Ok(next)
}

pub(crate) fn validate_character_equipment(
    state: &CampaignState,
    profile: &CharacterProfile,
    entity: &MechanicalEntity,
    pack: &RulesPack,
) -> Result<(), String> {
    let Some(inventory) = state
        .rules
        .as_ref()
        .and_then(|rules| rules.tactical_inventory.as_ref())
    else {
        return crate::validate_character_mechanics(profile, entity, pack)
            .map_err(|error| error.to_string());
    };
    let Some(loadout) = inventory.loadout(profile.entity_id) else {
        return crate::validate_character_mechanics(profile, entity, pack)
            .map_err(|error| error.to_string());
    };
    if !inventory
        .receipts
        .iter()
        .any(|receipt| receipt.actor == profile.entity_id)
    {
        return Err("A character loadout requires its source grant receipt.".into());
    }
    crate::validate_character_intrinsics(profile, entity, pack)
        .map_err(|error| error.to_string())?;
    validate_loadout(state, loadout).map_err(|error| error.to_string())?;
    let expected = current_armor(state, loadout)?;
    if entity.armor != expected
        || entity
            .character_features
            .as_ref()
            .is_none_or(|features| features.wearing_armor != loadout.worn_armor.is_some())
    {
        return Err("Current armor differs from physical equipment.".into());
    }
    Ok(())
}

fn current_armor(
    state: &CampaignState,
    loadout: &ActorEquipmentLoadout,
) -> Result<ArmorClass, String> {
    // The current source armor catalog contains Leather Armor. Adding source armor
    // later extends this adapter; unknown definitions never grant invented AC.
    let base = match loadout.worn_armor {
        None => 10,
        Some(id) => match state.items.get(&id).map(|item| item.definition_id.as_str()) {
            Some("leather-armor") => 11,
            _ => return Err("Unsupported source armor definition.".into()),
        },
    };
    Ok(ArmorClass::Armor {
        base,
        dexterity_cap: None,
        shield: loadout.shield.is_some(),
    })
}
