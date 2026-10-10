//! Table attachment of source equipment. CharacterProfile remains creation evidence;
//! the materialized ItemIds and current loadout determine live armor and availability.
use dmd_domain::*;
use dmd_rules::{RulesPack, tactical_inventory::*};

pub(crate) fn view(
    state: &CampaignState,
    character: CharacterId,
    pack: &RulesPack,
    host: bool,
) -> Result<crate::TableEquipmentView, String> {
    let plan =
        starting_equipment_plan(state, character, pack).map_err(|error| error.to_string())?;
    let inventory = state
        .rules
        .as_ref()
        .and_then(|rules| rules.tactical_inventory.as_ref());
    let prepared = inventory.is_some_and(|inventory| inventory.receipt(character).is_some());
    let loadout = inventory.and_then(|inventory| inventory.loadout(plan.actor));
    let mut items = state
        .items
        .values()
        .filter(|item| item.custody == Custody::Entity(plan.actor) && item.quantity > 0)
        .map(|item| crate::TableItemView {
            id: item.id,
            quantity: item.quantity,
            name: if host {
                item.display_name.clone()
            } else {
                equipment_definition(&item.definition_id)
                    .map(|definition| definition.display_name.clone())
                    .unwrap_or_else(|_| "Carried object".into())
            },
        })
        .collect::<Vec<_>>();
    items.sort_by_key(|item| item.id.0);
    Ok(crate::TableEquipmentView {
        prepared,
        initial_item_count: plan.identity_count,
        items,
        worn_armor: loadout.and_then(|loadout| loadout.worn_armor),
        shield: loadout.and_then(|loadout| loadout.shield),
        hands: loadout
            .map(|loadout| loadout.hands.clone())
            .unwrap_or_default(),
    })
}
