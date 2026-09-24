//! Durable starting-grant evidence and current equipment references.
//!
//! Creation evidence never changes when an item is spent, lost, borrowed or destroyed.
//! Rules reconstruct the source grant; the application authenticates retained commands
//! against its journal. This structural validator does not authenticate history.
use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::{
    AgentRef, CampaignState, CharacterId, CharacterProfile, CommandIssuer, CommandMeta, EntityId,
    HandAssignment, ItemId, WeaponLoadout,
};

pub const TACTICAL_INVENTORY_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EquipmentSourcePin {
    pub ruleset_id: String,
    pub ruleset_version: String,
    pub profile_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StartingEquipmentAllocation {
    pub definition_id: String,
    /// Total units originally granted, independent of current live quantities.
    pub initial_quantity: u32,
    /// Source-derived order: one identity for a stack, one per individual item.
    pub item_ids: Vec<ItemId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StartingEquipmentReceipt {
    pub character_id: CharacterId,
    pub actor: EntityId,
    /// The exact command that materialized these identities, not an invented
    /// replacement for the earlier character-creation command.
    pub command: CommandMeta,
    pub source: EquipmentSourcePin,
    pub creation_profile: CharacterProfile,
    pub allocations: Vec<StartingEquipmentAllocation>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActorEquipmentLoadout {
    pub actor: EntityId,
    pub hands: WeaponLoadout,
    pub worn_armor: Option<ItemId>,
    /// A donned shield must also occupy one of the hands.
    pub shield: Option<ItemId>,
    /// Last accepted equipment change; timing/permission belongs to its resolver.
    pub command: CommandMeta,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TacticalInventory {
    pub schema_version: u32,
    pub receipts: Vec<StartingEquipmentReceipt>,
    /// NPC loadouts may exist without a PC starting-grant receipt.
    pub loadouts: Vec<ActorEquipmentLoadout>,
}

impl Default for TacticalInventory {
    fn default() -> Self {
        Self {
            schema_version: TACTICAL_INVENTORY_SCHEMA_VERSION,
            receipts: Vec::new(),
            loadouts: Vec::new(),
        }
    }
}

impl TacticalInventory {
    pub fn receipt(&self, character: CharacterId) -> Option<&StartingEquipmentReceipt> {
        self.receipts
            .iter()
            .find(|entry| entry.character_id == character)
    }

    pub fn loadout(&self, actor: EntityId) -> Option<&ActorEquipmentLoadout> {
        self.loadouts.iter().find(|entry| entry.actor == actor)
    }

    /// Structural checks only. Also run rules::validate_tactical_inventory before
    /// making restored state runnable; only it reconstructs source allocations.
    pub fn validate(&self, state: &CampaignState) -> Result<(), String> {
        if self.schema_version != TACTICAL_INVENTORY_SCHEMA_VERSION
            || self.receipts.len() > state.characters.len()
            || self.loadouts.len() > state.entities.len()
        {
            return Err("invalid tactical inventory version or record count".into());
        }
        let mut characters = HashSet::new();
        let mut actors = HashSet::new();
        let mut commands = HashSet::new();
        let mut granted_ids = HashSet::new();
        for receipt in &self.receipts {
            let character = state
                .characters
                .get(&receipt.character_id)
                .ok_or("starting equipment character is absent")?;
            if receipt.character_id.0.is_nil()
                || character.id != receipt.character_id
                || character.entity_id != receipt.actor
                || character.campaign_id != state.campaign_id()
                || !characters.insert(receipt.character_id)
                || !actors.insert(receipt.actor)
                || !commands.insert(receipt.command.id)
            {
                return Err("invalid or duplicate starting equipment identity".into());
            }
            validate_equipment_origin(state, &receipt.command, receipt.actor)?;
            let profile = state
                .table
                .as_ref()
                .and_then(|table| table.character_profiles.get(&receipt.character_id))
                .ok_or("starting equipment profile is absent")?;
            if profile != &receipt.creation_profile
                || profile.entity_id != receipt.actor
                || receipt.source.ruleset_id != state.campaign.ruleset.id
                || receipt.source.ruleset_version != state.campaign.ruleset.version
                || receipt.source.profile_id.is_empty()
                || receipt.source.profile_id.len() > 128
                || receipt.allocations.len() > 64
            {
                return Err("starting equipment source or creation evidence differs".into());
            }
            let mut definitions = HashSet::new();
            for allocation in &receipt.allocations {
                if allocation.definition_id.is_empty()
                    || allocation.definition_id.len() > 128
                    || !definitions.insert(&allocation.definition_id)
                    || allocation.initial_quantity == 0
                    || allocation.initial_quantity > 1000
                    || allocation.item_ids.is_empty()
                    || allocation.item_ids.len() > allocation.initial_quantity as usize
                {
                    return Err("invalid initial equipment allocation".into());
                }
                for id in &allocation.item_ids {
                    let item = state
                        .items
                        .get(id)
                        .ok_or("granted item or tombstone is absent")?;
                    if id.0.is_nil()
                        || !granted_ids.insert(*id)
                        || item.id != *id
                        || item.campaign_id != state.campaign_id()
                        || item.definition_id != allocation.definition_id
                    {
                        return Err("invalid, shared or replaced granted item identity".into());
                    }
                }
            }
            let loadout = self
                .loadout(receipt.actor)
                .ok_or("materialized character has no current equipment record")?;
            if loadout.command.expected_event_sequence < receipt.command.expected_event_sequence {
                return Err("current equipment record predates starting grant".into());
            }
        }
        let mut equipped_actors = HashSet::new();
        for loadout in &self.loadouts {
            if !equipped_actors.insert(loadout.actor) {
                return Err("duplicate current equipment record".into());
            }
            validate_equipment_change_origin(state, &loadout.command, loadout.actor)?;
            for id in loadout
                .hands
                .hands
                .iter()
                .filter_map(|hand| match hand {
                    HandAssignment::Free => None,
                    HandAssignment::Item(id) => Some(*id),
                })
                .chain(loadout.worn_armor)
                .chain(loadout.shield)
            {
                let item = state.items.get(&id).ok_or("equipped item is absent")?;
                if id.0.is_nil() || item.id != id || item.campaign_id != state.campaign_id() {
                    return Err("equipped item identity differs".into());
                }
            }
        }
        Ok(())
    }
}

/// Retained authority references remain valid after a controller/owner transfer.
/// The issuing resolver checks ownership at the original head; journal replay proves it.
pub fn validate_equipment_origin(
    state: &CampaignState,
    command: &CommandMeta,
    actor: EntityId,
) -> Result<(), String> {
    validate_equipment_change_origin(state, command, actor)?;
    if command
        .actor
        .is_some_and(|id| id != AgentRef::Entity(actor))
        || matches!(command.issuer, CommandIssuer::Player(_))
            && command.actor != Some(AgentRef::Entity(actor))
    {
        return Err("starting grant must originate with its recipient or the host".into());
    }
    Ok(())
}

/// Consequences such as dropping held items retain the causing actor's command,
/// even when that actor differs from the equipment holder. This authenticates
/// references only; the resolver and journal prove the specific consequence.
pub fn validate_equipment_change_origin(
    state: &CampaignState,
    command: &CommandMeta,
    actor: EntityId,
) -> Result<(), String> {
    let entity = state
        .entities
        .get(&actor)
        .ok_or("equipment actor is absent")?;
    if actor.0.is_nil()
        || entity.id != actor
        || entity.campaign_id != state.campaign_id()
        || command.id.0.is_nil()
        || command.campaign_id != state.campaign_id()
        || command.expected_event_sequence > state.applied_event_sequence
        || command.session_id.is_some_and(|id| id.0.is_nil())
        || command.actor.is_some_and(|origin| match origin {
            AgentRef::Entity(id) => state.entities.get(&id).is_none_or(|entity| {
                id.0.is_nil() || entity.id != id || entity.campaign_id != state.campaign_id()
            }),
            AgentRef::Faction(_) => true,
        })
    {
        return Err("invalid equipment command provenance".into());
    }
    match command.issuer {
        CommandIssuer::System | CommandIssuer::Admin => Ok(()),
        CommandIssuer::Player(id)
            if matches!(command.actor, Some(AgentRef::Entity(_)))
                && state.players.get(&id).is_some_and(|player| {
                    !id.0.is_nil() && player.id == id && player.campaign_id == state.campaign_id()
                }) =>
        {
            Ok(())
        }
        _ => Err("invalid equipment command issuer".into()),
    }
}
