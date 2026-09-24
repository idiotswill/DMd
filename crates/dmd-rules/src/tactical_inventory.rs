//! Source-derived physical starting grants. No dice, generated identities or persistence.
//! The application commits this state and extension together in a versioned event.
mod registry;
pub use registry::*;

use std::collections::HashSet;

use dmd_domain::*;
use thiserror::Error;

use crate::{RulesPack, starter_catalog, validate_character_profile};

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum InventoryError {
    #[error("invalid equipment state: {0}")]
    Invalid(String),
    #[error("equipment command observes a stale campaign head")]
    Stale,
    #[error("equipment command is not authorized for this character")]
    Unauthorized,
    #[error("starting equipment has already been provisioned for {0:?}")]
    AlreadyProvisioned(CharacterId),
    #[error("existing equipment requires explicit reconciliation: {0:?}")]
    ReconciliationRequired(Vec<ItemId>),
    #[error("expected {expected} new item identities, received {actual}")]
    IdentityCount { expected: usize, actual: usize },
    #[error("new item identity is nil, duplicate or already present: {0:?}")]
    IdentityCollision(ItemId),
}
fn invalid(message: impl Into<String>) -> InventoryError {
    InventoryError::Invalid(message.into())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StartingEquipmentPlanAllocation {
    pub definition: EquipmentDefinition,
    pub initial_quantity: u32,
    pub identity_count: usize,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StartingEquipmentPlan {
    pub character_id: CharacterId,
    pub actor: EntityId,
    pub source: EquipmentSourcePin,
    pub creation_profile: CharacterProfile,
    /// Sorted by exact source definition ID, then each item's ordinal.
    pub allocations: Vec<StartingEquipmentPlanAllocation>,
    pub identity_count: usize,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InventoryTransition {
    /// Journal sequence is intentionally unchanged; persistence advances it.
    pub next_state: CampaignState,
    pub next_inventory: TacticalInventory,
    pub receipt: StartingEquipmentReceipt,
}

/// Immutable source query, independent of whether this character was already granted.
/// Application callers retain the supplied IDs with the original accepted request.
pub fn starting_equipment_plan(
    state: &CampaignState,
    character_id: CharacterId,
    pack: &RulesPack,
) -> Result<StartingEquipmentPlan, InventoryError> {
    validate_pack(state, pack)?;
    let character = state
        .characters
        .get(&character_id)
        .ok_or_else(|| invalid("unknown character"))?;
    let profile = state
        .table
        .as_ref()
        .and_then(|table| table.character_profiles.get(&character_id))
        .ok_or_else(|| invalid("character has no source creation evidence"))?;
    if character.id != character_id
        || character_id.0.is_nil()
        || character.campaign_id != state.campaign_id()
        || character.entity_id != profile.entity_id
        || !state.entities.contains_key(&character.entity_id)
    {
        return Err(invalid("character/source identity mismatch"));
    }
    validate_character_profile(profile, pack).map_err(|error| invalid(error.to_string()))?;
    let catalog = starter_catalog();
    let mut allocations = Vec::with_capacity(profile.equipment.len());
    for equipment in &profile.equipment {
        let definition = equipment_definition(&equipment.item_id)?.clone();
        let quantity = u32::from(equipment.quantity);
        allocations.push(StartingEquipmentPlanAllocation {
            identity_count: match definition.stacking {
                ItemStacking::Individual => quantity as usize,
                ItemStacking::Stack => 1,
            },
            initial_quantity: quantity,
            definition,
        });
    }
    let identity_count = allocations.iter().map(|entry| entry.identity_count).sum();
    Ok(StartingEquipmentPlan {
        character_id,
        actor: profile.entity_id,
        source: EquipmentSourcePin {
            ruleset_id: catalog.ruleset_id,
            ruleset_version: catalog.version,
            profile_id: catalog.profile_id,
        },
        creation_profile: profile.clone(),
        allocations,
        identity_count,
    })
}

/// Only this resolver's source-derived plan may create items. A repeat grant is an
/// error, including after all granted gear is lost/spent. Accepted-command recovery
/// belongs to the application, before calling this resolver at a new head.
pub fn materialize_starting_equipment(
    state: &CampaignState,
    current: &TacticalInventory,
    meta: &CommandMeta,
    character_id: CharacterId,
    new_item_ids: &[ItemId],
    pack: &RulesPack,
) -> Result<InventoryTransition, InventoryError> {
    if meta.expected_event_sequence != state.applied_event_sequence {
        return Err(InventoryError::Stale);
    }
    validate_tactical_inventory(state, current, pack)?;
    let plan = starting_equipment_plan(state, character_id, pack)?;
    validate_equipment_origin(state, meta, plan.actor).map_err(invalid)?;
    let character = &state.characters[&character_id];
    if character.status != CharacterStatus::Active
        || state.entities[&plan.actor].existence != EntityExistence::Present
        || match meta.issuer {
            CommandIssuer::System | CommandIssuer::Admin => false,
            CommandIssuer::Player(id) => character.controlling_player_id != Some(id),
            CommandIssuer::Import => true,
        }
    {
        return Err(InventoryError::Unauthorized);
    }
    if current.receipt(character_id).is_some() {
        return Err(InventoryError::AlreadyProvisioned(character_id));
    }
    if current.loadout(plan.actor).is_some() {
        return Err(invalid(
            "unprovisioned character already has an equipment record",
        ));
    }
    let mut ambiguous = state
        .items
        .values()
        .filter(|item| {
            plan.allocations
                .iter()
                .any(|allocation| allocation.definition.id == item.definition_id)
                && (item.owner == Ownership::Entity(plan.actor)
                    || ultimately_carried_by(state, item, plan.actor))
        })
        .map(|item| item.id)
        .collect::<Vec<_>>();
    ambiguous.sort_by_key(|id| id.0);
    if !ambiguous.is_empty() {
        return Err(InventoryError::ReconciliationRequired(ambiguous));
    }
    if new_item_ids.len() != plan.identity_count {
        return Err(InventoryError::IdentityCount {
            expected: plan.identity_count,
            actual: new_item_ids.len(),
        });
    }
    let mut seen = HashSet::new();
    for id in new_item_ids {
        if id.0.is_nil() || state.items.contains_key(id) || !seen.insert(*id) {
            return Err(InventoryError::IdentityCollision(*id));
        }
    }
    let mut next_state = state.clone();
    let mut allocations = Vec::with_capacity(plan.allocations.len());
    let mut offset = 0;
    for allocation in &plan.allocations {
        let ids = new_item_ids[offset..offset + allocation.identity_count].to_vec();
        offset += allocation.identity_count;
        for id in &ids {
            next_state.items.insert(
                *id,
                ItemInstance {
                    id: *id,
                    campaign_id: state.campaign_id(),
                    definition_id: allocation.definition.id.clone(),
                    display_name: allocation.definition.display_name.clone(),
                    quantity: match allocation.definition.stacking {
                        ItemStacking::Individual => 1,
                        ItemStacking::Stack => allocation.initial_quantity,
                    },
                    owner: Ownership::Entity(plan.actor),
                    custody: Custody::Entity(plan.actor),
                    state: ItemState::Intact,
                },
            );
        }
        allocations.push(StartingEquipmentAllocation {
            definition_id: allocation.definition.id.clone(),
            initial_quantity: allocation.initial_quantity,
            item_ids: ids,
        });
    }
    let first_id = |definition: &str| {
        allocations
            .iter()
            .find(|entry| entry.definition_id == definition)
            .and_then(|entry| entry.item_ids.first())
            .copied()
    };
    let worn_armor = plan
        .creation_profile
        .worn_armor
        .as_deref()
        .and_then(first_id);
    let shield = plan
        .creation_profile
        .shield
        .then(|| first_id("shield"))
        .flatten();
    let mut hands = WeaponLoadout::default();
    if let Some(id) = shield {
        hands.hands[Hand::Left.index()] = HandAssignment::Item(id);
    }
    let loadout = ActorEquipmentLoadout {
        actor: plan.actor,
        hands,
        worn_armor,
        shield,
        command: meta.clone(),
    };
    let receipt = StartingEquipmentReceipt {
        character_id,
        actor: plan.actor,
        command: meta.clone(),
        source: plan.source,
        creation_profile: plan.creation_profile,
        allocations,
    };
    let mut next_inventory = current.clone();
    next_inventory.receipts.push(receipt.clone());
    next_inventory.loadouts.push(loadout);
    validate_tactical_inventory(&next_state, &next_inventory, pack)?;
    Ok(InventoryTransition {
        next_state,
        next_inventory,
        receipt,
    })
}

/// Validate even at the earliest snapshot: receipts retain source-derived evidence,
/// while live custody/loadout may legitimately differ from the initial allocation.
pub fn validate_tactical_inventory(
    state: &CampaignState,
    current: &TacticalInventory,
    pack: &RulesPack,
) -> Result<(), InventoryError> {
    validate_pack(state, pack)?;
    if !state.validate().is_empty() {
        return Err(invalid("campaign has invalid identity/reference state"));
    }
    current.validate(state).map_err(invalid)?;
    // NPC grants and loose/borrowed objects need the same physical invariants as
    // PC starting allocations. Unknown campaign objects remain available for
    // improvised use; recognizing a definition does not grant custody or training.
    let registry = equipment_registry()?;
    for item in state.items.values() {
        if let Some(definition) = registry
            .iter()
            .find(|definition| definition.id == item.definition_id)
        {
            validate_physical_quantity(item, definition)?;
        }
    }
    for receipt in &current.receipts {
        let plan = starting_equipment_plan(state, receipt.character_id, pack)?;
        if receipt.source != plan.source || receipt.allocations.len() != plan.allocations.len() {
            return Err(invalid("starting equipment does not match source plan"));
        }
        for (recorded, expected) in receipt.allocations.iter().zip(&plan.allocations) {
            if recorded.definition_id != expected.definition.id
                || recorded.initial_quantity != expected.initial_quantity
                || recorded.item_ids.len() != expected.identity_count
            {
                return Err(invalid(
                    "starting equipment allocation differs from source grant",
                ));
            }
            for id in &recorded.item_ids {
                validate_physical_quantity(&state.items[id], &expected.definition)?;
            }
        }
    }
    for loadout in &current.loadouts {
        validate_loadout(state, loadout)?;
    }
    Ok(())
}

/// Equipment timing and authorization are checked by the enclosing accepted action.
/// This verifies current physical references, including borrowed equipment.
pub fn validate_loadout(
    state: &CampaignState,
    loadout: &ActorEquipmentLoadout,
) -> Result<(), InventoryError> {
    validate_equipment_change_origin(state, &loadout.command, loadout.actor).map_err(invalid)?;
    let mut held = Vec::new();
    for hand in &loadout.hands.hands {
        if let HandAssignment::Item(id) = hand {
            usable_item(state, *id, loadout.actor)?;
            if held.contains(id) && state.items[id].quantity != 1 {
                return Err(invalid(
                    "holding one item in both hands requires one physical object",
                ));
            }
            held.push(*id);
        }
    }
    if let Some(id) = loadout.worn_armor
        && (!usable_item(state, id, loadout.actor)?
            .is_some_and(|definition| definition.kind == EquipmentKind::Armor)
            || held.contains(&id)
            || loadout.shield == Some(id))
    {
        return Err(invalid(
            "worn armor must be distinct source armor in custody",
        ));
    }
    if let Some(id) = loadout.shield
        && (!usable_item(state, id, loadout.actor)?
            .is_some_and(|definition| definition.kind == EquipmentKind::Shield)
            || held.iter().filter(|held_id| **held_id == id).count() != 1)
    {
        return Err(invalid("donned shield must occupy exactly one hand"));
    }
    Ok(())
}

fn usable_item(
    state: &CampaignState,
    id: ItemId,
    actor: EntityId,
) -> Result<Option<&'static EquipmentDefinition>, InventoryError> {
    let item = state
        .items
        .get(&id)
        .ok_or_else(|| invalid("equipment item is absent"))?;
    if id.0.is_nil()
        || item.id != id
        || item.campaign_id != state.campaign_id()
        || item.custody != Custody::Entity(actor)
        || item.state != ItemState::Intact
        || item.quantity == 0
        || item.definition_id.trim().is_empty()
    {
        return Err(invalid(
            "equipped item must be intact and physically carried by actor",
        ));
    }
    // Holding a physical campaign object is not a source weapon grant. Unknown
    // definitions may be improvised objects; the source adjudicator, not this
    // loadout validator, decides their eventual attack mechanics.
    let definition = equipment_registry()?
        .iter()
        .find(|definition| definition.id == item.definition_id);
    if let Some(definition) = definition {
        validate_physical_quantity(item, definition)?;
    }
    Ok(definition)
}

fn validate_physical_quantity(
    item: &ItemInstance,
    definition: &EquipmentDefinition,
) -> Result<(), InventoryError> {
    if matches!(item.state, ItemState::Custom(_))
        || (matches!(item.state, ItemState::Intact | ItemState::Damaged) && item.quantity == 0)
        || (definition.stacking == ItemStacking::Individual && item.quantity > 1)
        || (item.state == ItemState::Spent && item.quantity != 0)
    {
        return Err(invalid(
            "physical equipment has an invalid quantity or unsupported state",
        ));
    }
    Ok(())
}

fn validate_pack(state: &CampaignState, pack: &RulesPack) -> Result<(), InventoryError> {
    pack.validate()
        .map_err(|error| invalid(error.to_string()))?;
    if state.campaign.ruleset.id != pack.id || state.campaign.ruleset.version != pack.version {
        return Err(invalid(
            "equipment source does not match campaign ruleset pin",
        ));
    }
    Ok(())
}

fn ultimately_carried_by(state: &CampaignState, item: &ItemInstance, actor: EntityId) -> bool {
    let mut custody = item.custody;
    let mut seen = HashSet::new();
    loop {
        match custody {
            Custody::Entity(id) => return id == actor,
            Custody::Container(id) if seen.insert(id) => {
                let Some(container) = state.items.get(&id) else {
                    return false;
                };
                custody = container.custody;
            }
            _ => return false,
        }
    }
}
