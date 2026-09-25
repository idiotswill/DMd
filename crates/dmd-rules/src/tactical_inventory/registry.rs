//! Exact source identity registry, separate from shop availability or item prices.
use std::sync::OnceLock;

use super::{InventoryError, invalid};
use crate::{
    starter_catalog,
    tactical_definitions::{TACTICAL_DEFINITIONS_JSON, TacticalDefinitions},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AmmunitionStackKind {
    Arrows,
    Bolts,
    Needles,
    FirearmBullets,
    SlingBullets,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EquipmentKind {
    Weapon,
    Ammunition(AmmunitionStackKind),
    Armor,
    Shield,
    Supply,
    Gear,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemStacking {
    Individual,
    Stack,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EquipmentDefinition {
    pub id: String,
    pub display_name: String,
    pub kind: EquipmentKind,
    pub stacking: ItemStacking,
    pub source_page: u16,
}

pub fn equipment_registry() -> Result<&'static [EquipmentDefinition], InventoryError> {
    static REGISTRY: OnceLock<Result<Vec<EquipmentDefinition>, InventoryError>> = OnceLock::new();
    REGISTRY
        .get_or_init(build_registry)
        .as_ref()
        .map(Vec::as_slice)
        .map_err(Clone::clone)
}
pub fn equipment_definition(id: &str) -> Result<&'static EquipmentDefinition, InventoryError> {
    equipment_registry()?
        .iter()
        .find(|entry| entry.id == id)
        .ok_or_else(|| invalid(format!("unknown source equipment identity: {id}")))
}

fn build_registry() -> Result<Vec<EquipmentDefinition>, InventoryError> {
    let definitions = TacticalDefinitions::from_json(TACTICAL_DEFINITIONS_JSON)
        .map_err(|error| invalid(error.to_string()))?;
    let mut registry = definitions
        .weapons
        .into_iter()
        .map(|weapon| EquipmentDefinition {
            id: weapon.id,
            display_name: weapon.name,
            kind: EquipmentKind::Weapon,
            stacking: ItemStacking::Individual,
            source_page: weapon.source_page,
        })
        .collect::<Vec<_>>();
    // SRD96 ammunition table distinguishes firearm and sling bullets explicitly.
    for (id, name, kind) in [
        ("arrows", "Arrows", AmmunitionStackKind::Arrows),
        ("bolts", "Bolts", AmmunitionStackKind::Bolts),
        ("needles", "Needles", AmmunitionStackKind::Needles),
        (
            "firearm-bullets",
            "Firearm Bullets",
            AmmunitionStackKind::FirearmBullets,
        ),
        (
            "sling-bullets",
            "Sling Bullets",
            AmmunitionStackKind::SlingBullets,
        ),
    ] {
        registry.push(EquipmentDefinition {
            id: id.into(),
            display_name: name.into(),
            kind: EquipmentKind::Ammunition(kind),
            stacking: ItemStacking::Stack,
            source_page: 96,
        });
    }
    // These exact IDs come from the current source-reviewed creation catalog.
    // The choice of identity-per-container/physical-object preserves custody; a
    // ration units use the catalog's food quantity (price SRD95, description SRD99),
    // not distinct named objects. Stack grouping is an inventory identity policy.
    for item in starter_catalog().items {
        if registry.iter().any(|entry| entry.id == item.id) {
            continue;
        }
        let (kind, stacking) = match item.id.as_str() {
            "leather-armor" => (EquipmentKind::Armor, ItemStacking::Individual),
            "shield" => (EquipmentKind::Shield, ItemStacking::Individual),
            "rations" => (EquipmentKind::Supply, ItemStacking::Stack),
            "quiver" | "backpack" | "travelers-clothes" | "rope" | "waterskin" | "gaming-dice"
            | "dragonchess" | "playing-cards" | "three-dragon-ante" => {
                (EquipmentKind::Gear, ItemStacking::Individual)
            }
            _ => return Err(invalid("creation source has no physical identity policy")),
        };
        registry.push(EquipmentDefinition {
            id: item.id,
            display_name: item.name,
            kind,
            stacking,
            source_page: item.source_page,
        });
    }
    // SRD98; carrying one does not itself grant a class's spellcasting-focus feature.
    registry.push(EquipmentDefinition {
        id: "holy-symbol".into(),
        display_name: "Holy Symbol".into(),
        kind: EquipmentKind::Gear,
        stacking: ItemStacking::Individual,
        source_page: 98,
    });
    // SRD96. A physical wand does not invent a class's focus permission; the
    // source spell still needs its actual component unless a real grant says so.
    registry.push(EquipmentDefinition {
        id: "wand".into(),
        display_name: "Wand".into(),
        kind: EquipmentKind::Gear,
        stacking: ItemStacking::Individual,
        source_page: 96,
    });
    // Exact per-spell material identity. The source descriptor supplies its nature,
    // minimum value and consumption rule; display names never establish those facts.
    for spell in definitions.spells {
        if let Some(material) = spell.components.material {
            registry.push(EquipmentDefinition {
                id: format!("spell-material:{}", spell.id),
                display_name: material.description,
                kind: EquipmentKind::Gear,
                stacking: ItemStacking::Stack,
                source_page: *spell
                    .source_pages
                    .first()
                    .ok_or_else(|| invalid("Spell material source page is absent."))?,
            });
        }
    }
    registry.sort_by(|a, b| a.id.cmp(&b.id));
    if registry
        .windows(2)
        .any(|entries| entries[0].id == entries[1].id)
    {
        return Err(invalid("duplicate source equipment identity"));
    }
    Ok(registry)
}
