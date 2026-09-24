//! Read-only physical choices. The accepted action revalidates every source and fact.
use dmd_domain::*;
use dmd_rules::tactical_definitions::{
    TACTICAL_DEFINITIONS_JSON, TacticalDefinitions, WeaponHands, WeaponKind, WeaponProperty,
};

pub(super) fn options(
    state: &CampaignState,
    actor: EntityId,
) -> Result<Option<crate::TableAttackOptions>, String> {
    let Some(encounter) = &state.encounter else {
        return Ok(None);
    };
    let Some(loadout) = state
        .rules
        .as_ref()
        .and_then(|rules| rules.tactical_inventory.as_ref())
        .and_then(|inventory| inventory.loadout(actor))
    else {
        return Ok(None);
    };
    let definitions = TacticalDefinitions::from_json(TACTICAL_DEFINITIONS_JSON)
        .map_err(|error| error.to_string())?;
    let usable = |item: &&ItemInstance| {
        item.custody == Custody::Entity(actor)
            && item.state == ItemState::Intact
            && item.quantity > 0
    };
    let mut weapons = Vec::new();
    for item in state.items.values().filter(usable) {
        let Some(weapon) = definitions.weapon(&item.definition_id) else {
            continue;
        };
        let deliveries = match weapon.kind {
            WeaponKind::Melee => {
                let mut choices = vec![WeaponDelivery::Melee];
                if weapon.properties.contains(&WeaponProperty::Thrown) {
                    choices.push(WeaponDelivery::Thrown);
                }
                choices
            }
            WeaponKind::Ranged if weapon.ammunition.is_some() => vec![WeaponDelivery::Shot],
            WeaponKind::Ranged => vec![WeaponDelivery::Thrown],
        };
        let abilities = if weapon.properties.contains(&WeaponProperty::Finesse) {
            vec![Ability::Strength, Ability::Dexterity]
        } else {
            vec![match weapon.kind {
                WeaponKind::Melee => Ability::Strength,
                WeaponKind::Ranged => Ability::Dexterity,
            }]
        };
        let mut grips = match weapon.hands {
            WeaponHands::One => vec![
                WeaponGrip::OneHand(Hand::Left),
                WeaponGrip::OneHand(Hand::Right),
            ],
            // The current source resolver has no mounted action authority yet.
            WeaponHands::Two | WeaponHands::TwoUnlessMounted => vec![WeaponGrip::TwoHands],
        };
        if weapon.versatile_damage.is_some() {
            grips.push(WeaponGrip::TwoHands);
        }
        let ammunition_id = dmd_rules::tactical_weapons::required_ammunition_definition(weapon);
        let mut ammunition = state
            .items
            .values()
            .filter(usable)
            .filter(|candidate| Some(candidate.definition_id.as_str()) == ammunition_id)
            .map(|candidate| crate::TableItemView {
                id: candidate.id,
                name: dmd_rules::tactical_inventory::equipment_definition(&candidate.definition_id)
                    .map(|source| source.display_name.clone())
                    .unwrap_or_else(|_| "Ammunition".into()),
                quantity: candidate.quantity,
            })
            .collect::<Vec<_>>();
        ammunition.sort_by_key(|stack| stack.id.0);
        weapons.push(crate::TableWeaponChoice {
            item: item.id,
            name: weapon.name.clone(),
            deliveries,
            abilities,
            grips,
            ammunition_required: ammunition_id.is_some(),
            ammunition,
        });
    }
    weapons.sort_by(|a, b| a.name.cmp(&b.name).then(a.item.0.cmp(&b.item.0)));
    // Target knowledge belongs to the acting creature, even if one human controls
    // several observers. The host's full map is not this actor's perception.
    let observed = dmd_rules::spatial::project_actor_view(encounter, state, actor)
        .map_err(|error| error.to_string())?;
    let targets = observed
        .contacts
        .into_iter()
        .filter(|contact| {
            contact.entity_id != actor
                && contact.status != dmd_rules::spatial::ContactStatus::Remembered
        })
        .map(|contact| crate::TableAttackTarget {
            actor: contact.entity_id,
            label: contact.label.unwrap_or_else(|| "Located creature".into()),
        })
        .collect();
    Ok(Some(crate::TableAttackOptions {
        actor,
        hands: loadout.hands.clone(),
        weapons,
        targets,
    }))
}
