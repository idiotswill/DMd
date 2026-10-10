//! Private proof carried by the existing owner; never a serialized source cache.
use super::*;
use crate::table::{TableOperation, execution::ClosedImage};

#[derive(Clone, Default, serde::Serialize)]
pub(crate) struct PhysicalSources {
    allocations: Vec<SourceAllocation>,
}
#[derive(Clone, serde::Serialize)]
struct SourceAllocation {
    actor: EntityId,
    origin: CommandMeta,
    source: String,
    items: Vec<SourceItem>,
}
#[derive(Clone, serde::Serialize)]
struct SourceItem {
    id: ItemId,
    definition: String,
    initial_quantity: u32,
    quantity_unit: String,
}
impl PhysicalSources {
    pub(crate) fn after(
        &self,
        before: &CampaignState,
        after: &CampaignState,
        meta: &CommandMeta,
        operation: &TableOperation,
    ) -> Result<Self, String> {
        let (actor, source, ids) = match operation {
            TableOperation::PrepareEquipment {
                character_id,
                item_ids,
            } => {
                let receipt = after
                    .rules
                    .as_ref()
                    .and_then(|r| r.tactical_inventory.as_ref())
                    .and_then(|i| {
                        i.receipts
                            .iter()
                            .find(|r| r.character_id == *character_id && r.command == *meta)
                    })
                    .ok_or("Successful equipment materializer lost its original receipt.")?;
                (
                    receipt.actor,
                    catalog::fingerprint(&(&receipt.source, &receipt.creation_profile))?,
                    item_ids,
                )
            }
            TableOperation::CreateCreature { creation } => {
                let profile = after
                    .rules
                    .as_ref()
                    .and_then(|r| r.tactical_creatures.as_ref())
                    .and_then(|c| c.profile(creation.entity_id))
                    .filter(|p| p.origin == *meta)
                    .ok_or("Successful creature materializer lost its original profile.")?;
                (
                    creation.entity_id,
                    catalog::fingerprint(&profile.source)?,
                    &creation.item_ids,
                )
            }
            _ => return Ok(self.clone()),
        };
        if self.allocations.iter().any(|a| a.actor == actor) {
            return Err("Source allocations were materialized twice.".into());
        }
        let mut items = Vec::new();
        for id in ids {
            if before.items.contains_key(id) {
                return Err("Source materializer reused a physical identity.".into());
            }
            let item = after
                .items
                .get(id)
                .ok_or("Source materializer lost an item.")?;
            let unit = catalog::entry(&item.definition_id)
                .ok_or("Source item is absent from the physical catalog.")?;
            items.push(SourceItem {
                id: *id,
                definition: item.definition_id.clone(),
                initial_quantity: item.quantity,
                quantity_unit: unit.quantity_unit.clone(),
            });
        }
        let mut next = self.clone();
        next.allocations.push(SourceAllocation {
            actor,
            origin: meta.clone(),
            source,
            items,
        });
        Ok(next)
    }
    pub(crate) fn contains(&self, item: &ItemInstance) -> bool {
        self.allocations
            .iter()
            .flat_map(|a| &a.items)
            .any(|grant| grant.id == item.id && grant.definition == item.definition_id)
    }
    pub(crate) fn coverage(
        &self,
        state: &CampaignState,
        actor: EntityId,
    ) -> Result<String, String> {
        let allocations = self
            .allocations
            .iter()
            .filter(|a| a.actor == actor)
            .collect::<Vec<_>>();
        let has_profile = state
            .table
            .as_ref()
            .is_some_and(|t| t.character_profiles.values().any(|p| p.entity_id == actor))
            || state
                .rules
                .as_ref()
                .and_then(|r| r.tactical_creatures.as_ref())
                .is_some_and(|c| c.profile(actor).is_some());
        if has_profile && allocations.is_empty() {
            return Err(
                "Materialize the actual starting equipment before confirming complete coverage."
                    .into(),
            );
        }
        catalog::fingerprint(&allocations)
    }
}

pub(crate) struct MassCommand<'a> {
    before: &'a ClosedImage,
    candidate: *const CampaignState,
    command: &'a CommandMeta,
    produced: Option<Box<PhysicalFacts>>,
    created: Vec<ItemInstance>,
}
impl<'a> MassCommand<'a> {
    pub(crate) fn new(
        before: &'a ClosedImage,
        candidate: &CampaignState,
        command: &'a CommandMeta,
    ) -> Self {
        Self {
            before,
            candidate: std::ptr::from_ref(candidate),
            command,
            produced: None,
            created: Vec::new(),
        }
    }
    pub(crate) fn check(&self, state: &CampaignState) -> Result<(), String> {
        if !std::ptr::eq(self.candidate, state) {
            return Err("Physical-fact proof belongs to another candidate.".into());
        }
        Ok(())
    }
    pub(crate) fn sources(&self) -> &PhysicalSources {
        self.before.physical_sources()
    }
    pub(crate) fn validate_read(&self, state: &CampaignState) -> Result<(), String> {
        self.check(state)?;
        let expected = self
            .produced
            .as_ref()
            .or(self.before.state().physical_facts.as_ref());
        if state.physical_facts.as_ref() != expected {
            return Err("Physical facts lack the exact predecessor or current producer.".into());
        }
        Ok(())
    }
    pub(crate) fn apply(
        &mut self,
        state: &mut CampaignState,
        meta: &CommandMeta,
        acceptance: &PhysicalFactAcceptance,
    ) -> Result<(), String> {
        self.check(state)?;
        if self.produced.is_some() || self.command != meta || state != self.before.state() {
            return Err(
                "Physical-fact producer requires its exact untouched predecessor and command."
                    .into(),
            );
        }
        let created = super::accept(state, meta, acceptance, self.sources())?;
        self.created = created;
        self.produced = state.physical_facts.clone();
        self.validate_read(state)
    }
    pub(crate) fn validate_delta(&self, state: &CampaignState) -> Result<(), String> {
        self.validate_read(state)?;
        if self.produced.is_some() {
            let mut expected = self.before.state().items.clone();
            for item in &self.created {
                expected.insert(item.id, item.clone());
            }
            if state.items != expected {
                return Err("Physical materialization differs from the actual producer.".into());
            }
        }
        for item in state.items.values() {
            if matches!(
                item.definition_id.as_str(),
                CUSTOM_LOAD_DEFINITION | CURRENCY_LOT_DEFINITION
            ) && !self.before.state().items.contains_key(&item.id)
                && !self.created.contains(item)
            {
                return Err("Unobserved physical-fact materialization.".into());
            }
        }
        Ok(())
    }
}
