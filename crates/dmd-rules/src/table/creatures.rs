use crate::{RulesPack, tactical_creatures::*};
use dmd_domain::*;

pub(crate) fn create(
    state: &CampaignState,
    meta: &CommandMeta,
    creation: &super::TableCreatureCreation,
    pack: &RulesPack,
) -> Result<CampaignState, String> {
    bounded_text(&creation.name, 200)?;
    if state.rules.is_none() {
        return Err("Create a player character before preparing creatures.".into());
    }
    if creation.entity_id.0.is_nil() || state.entities.contains_key(&creation.entity_id) {
        return Err("Choose a new creature identity.".into());
    }
    let mut next = state.clone();
    next.entities.insert(
        creation.entity_id,
        WorldEntity {
            id: creation.entity_id,
            campaign_id: meta.campaign_id,
            display_name: creation.name.clone(),
            kind: EntityKind::Creature,
            existence: EntityExistence::Present,
            location_id: None,
        },
    );
    let built = build_creature_from_source(
        &next,
        meta,
        creation.entity_id,
        &CreatureBuildChoice {
            definition_id: creation.definition_id.clone(),
            size: creation.size,
            additional_languages: creation.additional_languages.clone(),
            hit_points: CreatureHitPointChoice::Average,
            controller: CreatureController::Autonomous,
            in_lair: false,
        },
        creation.source.as_ref(),
    )
    .map_err(|e| e.to_string())?;
    let rules = next.rules.as_mut().ok_or("Missing mechanical state.")?;
    rules.entities.insert(creation.entity_id, built.mechanics);
    let creatures = rules.tactical_creatures.get_or_insert_default();
    creatures.profiles.push(built.profile);
    creatures.runtime.push(built.runtime);
    next = crate::tactical_creature_equipment::materialize_creature_equipment(
        &next,
        meta,
        creation.entity_id,
        creation.ammunition_units,
        &creation.item_ids,
        pack,
    )
    .map_err(|e| e.to_string())?;
    Ok(next)
}
