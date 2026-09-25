//! Only authorized current actors receive their real carried shield choices.
use dmd_domain::*;
use dmd_rules::tactical_inventory::{EquipmentKind, equipment_definition};

pub(super) fn options(
    state: &CampaignState,
    actor: EntityId,
) -> Result<Option<crate::TableShieldOptions>, String> {
    let Some(rules) = &state.rules else {
        return Ok(None);
    };
    if rules
        .timing
        .as_ref()
        .is_none_or(|timing| timing.action_spent)
        || !dmd_rules::tactical_conditions::can_act(rules, actor).map_err(|e| e.to_string())?
    {
        return Ok(None);
    }
    let Some(loadout) = rules
        .tactical_inventory
        .as_ref()
        .and_then(|inventory| inventory.loadout(actor))
    else {
        return Ok(None);
    };
    let mut shields = Vec::new();
    if loadout.shield.is_none() {
        for item in state.items.values().filter(|item| {
            item.custody == Custody::Entity(actor)
                && item.state == ItemState::Intact
                && item.quantity == 1
        }) {
            if !equipment_definition(&item.definition_id)
                .is_ok_and(|d| d.kind == EquipmentKind::Shield)
            {
                continue;
            }
            let hands = [Hand::Left, Hand::Right]
                .into_iter()
                .filter(|hand| {
                    (loadout.hands.hands[hand.index()] == HandAssignment::Free
                        || loadout.hands.hands[hand.index()] == HandAssignment::Item(item.id))
                        && loadout.hands.hands[1 - hand.index()] != HandAssignment::Item(item.id)
                })
                .collect::<Vec<_>>();
            if !hands.is_empty() {
                shields.push(crate::TableShieldChoice {
                    item: item.id,
                    hands,
                });
            }
        }
    }
    shields.sort_by_key(|shield| shield.item.0);
    Ok(
        (loadout.shield.is_some() || !shields.is_empty()).then_some(crate::TableShieldOptions {
            actor,
            donned: loadout.shield,
            shields,
        }),
    )
}
