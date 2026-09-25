//! Source gear and required components are physical campaign items (SRD255/257).
//! This internal creation operation is committed with the source creature's profile.
use std::collections::{BTreeMap, HashSet};

use dmd_domain::*;

use crate::{
    RulesError, RulesPack, ability_modifier, tactical_creatures::*, tactical_definitions::*,
    tactical_inventory::*,
};

fn invalid(message: impl Into<String>) -> RulesError {
    RulesError::Invalid(message.into())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatureEquipmentAllocation {
    pub definition_id: String,
    pub quantity: u32,
}

/// Source-backed attack implements. Feature IDs are canonical, never item labels.
/// SRD255/257 requires actual Gear for these stat-block weapon attacks; intrinsic
/// attacks (including the fanatic's magical Pact Blade, SRD278) are not loot weapons.
pub fn creature_attack_gear(
    profile: &CreatureProfile,
    feature_id: &str,
) -> Result<Option<&'static str>, RulesError> {
    let source = source_for_profile(profile).map_err(|e| invalid(e.to_string()))?;
    if !source.features.iter().any(|feature| {
        feature.id == feature_id && matches!(feature.feature, MonsterFeature::Attack { .. })
    }) {
        return Err(invalid("The selected source feature is not an attack."));
    }
    let gear = match (source.id.as_str(), feature_id) {
        ("goblin-warrior", "scimitar") => Some("scimitar"),
        ("goblin-warrior" | "skeleton", "shortbow") => Some("shortbow"),
        ("skeleton", "shortsword") => Some("shortsword"),
        ("wolf", "bite")
        | ("warhorse", "hooves")
        | ("young-red-dragon" | "adult-red-dragon", "rend")
        | ("cultist-fanatic", "pact-blade")
        | ("chimera", "bite" | "claw" | "ram") => None,
        _ => return Err(invalid("Source attack implement interpretation is absent.")),
    };
    if gear.is_some_and(|id| !source.statistics.gear.iter().any(|entry| entry == id)) {
        return Err(invalid("Source attack implement is absent from its Gear."));
    }
    Ok(gear)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceSpellMaterial {
    pub spell_id: String,
    pub value_cp: u32,
}

/// The exact registered definition establishes the material and its source value.
/// A caller still validates current custody, quantity, hands and consumption.
pub fn source_spell_material(
    item: &ItemInstance,
) -> Result<Option<SourceSpellMaterial>, RulesError> {
    let Some(id) = item.definition_id.strip_prefix("spell-material:") else {
        return Ok(None);
    };
    let definitions = creature_definitions().map_err(|e| invalid(e.to_string()))?;
    let spell = definitions
        .spell(id)
        .ok_or_else(|| invalid("Unknown specified spell material."))?;
    let material = spell
        .components
        .material
        .as_ref()
        .ok_or_else(|| invalid("Spell has no specified material."))?;
    Ok(Some(SourceSpellMaterial {
        spell_id: spell.id.clone(),
        value_cp: material.minimum_cost_cp,
    }))
}

/// The source provides necessary ammunition but no numeric stock. A finite quantity
/// is an explicit host setup choice; it never refreshes when an encounter is reloaded.
pub fn creature_equipment_plan(
    definition_id: &str,
    ammunition_units: u16,
) -> Result<Vec<CreatureEquipmentAllocation>, RulesError> {
    let definitions = creature_definitions().map_err(|e| invalid(e.to_string()))?;
    let source = creature_definition(definition_id).map_err(|e| invalid(e.to_string()))?;
    let mut result = BTreeMap::new();
    let mut ammunition = HashSet::new();
    for id in &source.statistics.gear {
        equipment_definition(id).map_err(|e| invalid(e.to_string()))?;
        result.insert(id.clone(), 1);
        if let Some(weapon) = definitions.weapon(id)
            && let Some(id) = crate::tactical_weapons::required_ammunition_definition(weapon)
        {
            ammunition.insert(id);
        }
    }
    if ammunition_units > 1000 || (ammunition.is_empty() != (ammunition_units == 0)) {
        return Err(invalid(
            "Choose 1 to 1000 ammunition units for a source ranged weapon, otherwise zero.",
        ));
    }
    for id in ammunition {
        result.insert(id.into(), u32::from(ammunition_units));
    }
    for feature in &source.features {
        if feature
            .spell_component_waivers
            .is_some_and(|waivers| waivers.material)
        {
            continue;
        }
        if let MonsterFeature::Spellcasting { spells, .. } = &feature.feature {
            for grant in spells {
                let spell = definitions
                    .spell(&grant.spell_id)
                    .ok_or_else(|| invalid("Unknown source spell."))?;
                if spell.components.material.is_some() {
                    result.insert(format!("spell-material:{}", spell.id), 1);
                }
            }
        }
    }
    Ok(result
        .into_iter()
        .map(|(definition_id, quantity)| CreatureEquipmentAllocation {
            definition_id,
            quantity,
        })
        .collect())
}

/// Establishes custody once during source creation. No RNG, new IDs or journal writes.
pub fn materialize_creature_equipment(
    state: &CampaignState,
    meta: &CommandMeta,
    actor: EntityId,
    ammunition_units: u16,
    item_ids: &[ItemId],
    pack: &RulesPack,
) -> Result<CampaignState, RulesError> {
    if meta.expected_event_sequence != state.applied_event_sequence {
        return Err(RulesError::Stale);
    }
    if !matches!(meta.issuer, CommandIssuer::System | CommandIssuer::Admin) {
        return Err(RulesError::Unauthorized);
    }
    let rules = state.rules.as_ref().ok_or(RulesError::Uninitialized)?;
    let profile = rules
        .tactical_creatures
        .as_ref()
        .and_then(|creatures| creatures.profile(actor))
        .ok_or_else(|| invalid("Source creature profile is absent."))?;
    if profile.origin != *meta
        || rules
            .tactical_inventory
            .as_ref()
            .is_some_and(|inventory| inventory.loadout(actor).is_some())
        || state.items.values().any(|item| {
            item.owner == Ownership::Entity(actor) || item.custody == Custody::Entity(actor)
        })
    {
        return Err(invalid(
            "Creature equipment must be materialized once with its creation.",
        ));
    }
    let allocations = creature_equipment_plan(&profile.source.definition_id, ammunition_units)?;
    if item_ids.len() != allocations.len()
        || item_ids
            .iter()
            .any(|id| id.0.is_nil() || state.items.contains_key(id))
        || item_ids.iter().collect::<HashSet<_>>().len() != item_ids.len()
    {
        return Err(invalid(
            "Creature equipment requires distinct new identities for its source allocations.",
        ));
    }
    let mut next = state.clone();
    let mut loadout = ActorEquipmentLoadout {
        actor,
        hands: WeaponLoadout::default(),
        worn_armor: None,
        shield: None,
        command: meta.clone(),
    };
    for (allocation, id) in allocations.iter().zip(item_ids) {
        let definition =
            equipment_definition(&allocation.definition_id).map_err(|e| invalid(e.to_string()))?;
        next.items.insert(
            *id,
            ItemInstance {
                id: *id,
                campaign_id: meta.campaign_id,
                definition_id: allocation.definition_id.clone(),
                display_name: definition.display_name.clone(),
                quantity: allocation.quantity,
                owner: Ownership::Entity(actor),
                custody: Custody::Entity(actor),
                state: ItemState::Intact,
            },
        );
        match definition.kind {
            EquipmentKind::Armor => loadout.worn_armor = Some(*id),
            EquipmentKind::Shield => {
                loadout.shield = Some(*id);
                loadout.hands.hands[Hand::Left.index()] = HandAssignment::Item(*id);
            }
            _ => (),
        }
    }
    next.rules
        .as_mut()
        .ok_or(RulesError::Uninitialized)?
        .tactical_inventory
        .get_or_insert_default()
        .loadouts
        .push(loadout);
    let armor = creature_current_armor(&next, profile)?;
    next.rules
        .as_mut()
        .ok_or(RulesError::Uninitialized)?
        .entities
        .get_mut(&actor)
        .ok_or_else(|| invalid("Creature mechanics are absent."))?
        .armor = armor;
    validate_tactical_inventory(
        &next,
        next.rules
            .as_ref()
            .and_then(|r| r.tactical_inventory.as_ref())
            .expect("inventory inserted"),
        pack,
    )
    .map_err(|e| invalid(e.to_string()))?;
    Ok(next)
}

/// Separates source natural defenses from physical armor/shield. The source's initial
/// equipment formula must equal its printed AC before this armor interpretation applies.
pub fn creature_current_armor(
    state: &CampaignState,
    profile: &CreatureProfile,
) -> Result<ArmorClass, RulesError> {
    let source = source_for_profile(profile).map_err(|e| invalid(e.to_string()))?;
    let initial = u16::from(source.statistics.armor_class);
    let Some(loadout) = state
        .rules
        .as_ref()
        .and_then(|r| r.tactical_inventory.as_ref())
        .and_then(|i| i.loadout(profile.actor))
    else {
        return Ok(ArmorClass::Fixed(initial));
    };
    validate_loadout(state, loadout).map_err(|e| invalid(e.to_string()))?;
    let had_armor = source
        .statistics
        .gear
        .iter()
        .any(|id| id == "leather-armor");
    let had_shield = source.statistics.gear.iter().any(|id| id == "shield");
    let dex = ability_modifier(source.statistics.ability_scores[Ability::Dexterity.index()]);
    if had_armor && 11 + dex + if had_shield { 2 } else { 0 } != i32::from(initial) {
        return Err(invalid(
            "Source armor requires an explicit defense interpretation.",
        ));
    }
    let worn_base = match loadout.worn_armor {
        None => 10,
        Some(id) => match state.items.get(&id).map(|item| item.definition_id.as_str()) {
            Some("leather-armor") => 11,
            _ => return Err(invalid("Unsupported source creature armor.")),
        },
    };
    let armor = worn_base + dex;
    let unarmored = if had_armor {
        10 + dex
    } else {
        i32::from(initial) - if had_shield { 2 } else { 0 }
    };
    // Anyone can don a shield, but only source training grants its AC benefit
    // (SRD92). A source monster has training with the armor in its stat block.
    let total = armor.max(unarmored)
        + if had_shield && loadout.shield.is_some() {
            2
        } else {
            0
        };
    u16::try_from(total)
        .map(ArmorClass::Fixed)
        .map_err(|_| invalid("Creature armor is outside bounds."))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_gear_is_recoverable_but_intrinsic_flourishes_are_not_loot() {
        let goblin = creature_equipment_plan("goblin-warrior", 20).unwrap();
        assert_eq!(
            goblin
                .iter()
                .map(|item| (item.definition_id.as_str(), item.quantity))
                .collect::<Vec<_>>(),
            vec![
                ("arrows", 20),
                ("leather-armor", 1),
                ("scimitar", 1),
                ("shield", 1),
                ("shortbow", 1)
            ]
        );
        let cultist = creature_equipment_plan("cultist-fanatic", 0).unwrap();
        assert!(
            cultist
                .iter()
                .any(|item| item.definition_id == "holy-symbol")
        );
        assert!(
            cultist
                .iter()
                .any(|item| item.definition_id == "spell-material:hold-person")
        );
        assert!(
            !cultist
                .iter()
                .any(|item| item.definition_id == "pact-blade")
        );
        assert!(
            creature_equipment_plan("adult-red-dragon", 0)
                .unwrap()
                .is_empty(),
            "actual source material waiver, no invented teeth/claw loot"
        );
    }

    #[test]
    fn initial_ammunition_is_explicit_and_bounded() {
        assert!(creature_equipment_plan("goblin-warrior", 0).is_err());
        assert!(creature_equipment_plan("goblin-warrior", 1001).is_err());
        assert!(creature_equipment_plan("wolf", 1).is_err());
        assert!(creature_equipment_plan("wolf", 0).unwrap().is_empty());
    }
}
