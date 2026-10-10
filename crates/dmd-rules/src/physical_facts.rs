//! Source amounts, typed Host facts and custody-derived load share the table owner.
use dmd_domain::*;
use std::collections::HashSet;
pub mod catalog;
pub(crate) mod execution;
mod load;
use execution::PhysicalSources;
pub(crate) use load::load;

pub fn enabled(state: &CampaignState) -> bool {
    state.physical_facts.is_some()
}

pub fn settled(state: &CampaignState) -> Result<(), String> {
    crate::table::source_control::settled(state)?;
    if state
        .table
        .as_ref()
        .is_some_and(|t| t.inspiration_transfer.is_some())
        || state.encounter.as_ref().is_some_and(|e| {
            e.flow
                .as_ref()
                .is_none_or(|f| f.phase != TacticalPhase::Finished)
        })
    {
        return Err(
            "Finish the encounter and all pending work before editing physical facts.".into(),
        );
    }
    Ok(())
}

pub fn actor_identity(state: &CampaignState, actor: EntityId) -> Result<String, String> {
    let entity = state
        .entities
        .get(&actor)
        .ok_or("Physical subject is absent.")?;
    let pc = state
        .table
        .as_ref()
        .and_then(|t| t.character_profiles.values().find(|p| p.entity_id == actor));
    let creature = state
        .rules
        .as_ref()
        .and_then(|r| r.tactical_creatures.as_ref())
        .and_then(|c| c.profile(actor));
    catalog::fingerprint(&(
        entity.id,
        &entity.kind,
        pc.map(|p| (&p.creation_source, &p.species_id, p.size)),
        creature.map(|p| (&p.source, p.size, &p.origin)),
    ))
}

fn children(state: &CampaignState, item: ItemId) -> Vec<ItemId> {
    let mut ids = state
        .items
        .values()
        .filter(|i| i.custody == Custody::Container(item))
        .map(|i| i.id)
        .collect::<Vec<_>>();
    ids.sort_by_key(|id| id.0);
    ids
}

pub(crate) fn offers(
    state: &CampaignState,
    sources: &PhysicalSources,
) -> Result<Vec<PhysicalFactOffer>, String> {
    if settled(state).is_err() {
        return Ok(Vec::new());
    }
    let Some(facts) = &state.physical_facts else {
        return Ok(vec![PhysicalFactOffer::Enable {
            catalog: catalog::pin()?,
        }]);
    };
    let previous = |subject| facts.get(subject).map(|f| f.origin.id);
    let mut actors = state.entities.keys().copied().collect::<Vec<_>>();
    actors.sort_by_key(|id| id.0);
    let mut result = Vec::new();
    for actor in actors {
        let identity = actor_identity(state, actor)?;
        result.push(PhysicalFactOffer::Body {
            actor,
            identity: identity.clone(),
            previous: previous(PhysicalSubject::Body(actor)),
        });
        if let Ok(allocations) = sources.coverage(state, actor) {
            result.push(PhysicalFactOffer::Coverage {
                actor,
                identity: identity.clone(),
                allocations,
                previous: previous(PhysicalSubject::Coverage(actor)),
            });
            result.push(PhysicalFactOffer::PersonalItem { actor, identity });
        }
    }
    let mut items = state.items.values().collect::<Vec<_>>();
    items.sort_by_key(|i| i.id.0);
    for item in items {
        if item.definition_id != CURRENCY_LOT_DEFINITION {
            result.push(PhysicalFactOffer::Item {
                item: item.clone(),
                contents: children(state, item.id),
                previous: previous(PhysicalSubject::Item(item.id)),
            });
        }
    }
    let mut characters = state.characters.values().collect::<Vec<_>>();
    characters.sort_by_key(|c| c.id.0);
    for character in characters {
        let Some(profile) = state.table.as_ref().and_then(|t| {
            t.character_profiles
                .values()
                .find(|p| p.entity_id == character.entity_id)
        }) else {
            continue;
        };
        if profile.money_cp == 0 {
            continue;
        }
        let lot = match facts
            .get(PhysicalSubject::Currency(character.id))
            .map(|f| &f.value)
        {
            Some(PhysicalFactValue::Currency { item, .. }) => Some(
                state
                    .items
                    .get(item)
                    .ok_or("Accepted currency identity is absent.")?
                    .clone(),
            ),
            _ => None,
        };
        result.push(PhysicalFactOffer::Currency {
            character: character.id,
            identity: actor_identity(state, character.entity_id)?,
            value_cp: profile.money_cp,
            lot,
            previous: previous(PhysicalSubject::Currency(character.id)),
        });
    }
    Ok(result)
}

fn text(value: &str, max: usize) -> Result<(), String> {
    if value.trim().is_empty() || value.len() > max || value.chars().any(char::is_control) {
        return Err("Enter a short physical description or correction reason.".into());
    }
    Ok(())
}

fn insert(
    state: &mut CampaignState,
    meta: &CommandMeta,
    subject: PhysicalSubject,
    previous: Option<CommandId>,
    value: PhysicalFactValue,
    reason: &str,
) -> Result<(), String> {
    text(reason, 1000)?;
    let facts = state
        .physical_facts
        .as_mut()
        .ok_or("Enable physical facts first.")?;
    if facts.get(subject).map(|f| f.origin.id) != previous {
        return Err("The accepted physical fact changed.".into());
    }
    let fact = PhysicalFact {
        subject,
        origin: meta.clone(),
        previous,
        value,
        reason: reason.to_owned(),
    };
    if let Some(index) = facts.records.iter().position(|f| f.subject == subject) {
        facts.records[index] = fact;
    } else {
        if facts.records.len() >= 4096 {
            return Err("Physical fact limit reached.".into());
        }
        facts.records.push(fact);
    }
    Ok(())
}

fn parse_item(
    item: &ItemInstance,
    contents: &[ItemId],
    input: &PhysicalItemInput,
) -> Result<PhysicalItemMass, String> {
    use catalog::CatalogMass;
    let printed = catalog::entry(&item.definition_id).map(|e| &e.mass);
    Ok(match input {
        PhysicalItemInput::Unknown => PhysicalItemMass::Unknown,
        PhysicalItemInput::NonstandardUnit {
            description,
            pounds,
        } => {
            text(description, 200)?;
            PhysicalItemMass::NonstandardUnit {
                description: description.clone(),
                mass: PhysicalMass::parse_pounds(pounds)?,
            }
        }
        PhysicalItemInput::Catalog { condition } => {
            catalog::amount(&item.definition_id, condition.as_deref())?;
            if matches!(printed, Some(CatalogMass::ConditionalGross { .. })) && !contents.is_empty()
            {
                return Err(
                    "Inclusive container mass cannot also contain tracked contents.".into(),
                );
            }
            PhysicalItemMass::Catalog {
                condition: condition.clone(),
            }
        }
        PhysicalItemInput::Unit { pounds, condition } => {
            if !(matches!(printed, None | Some(CatalogMass::Unquantified)) && condition.is_none()
                || matches!(printed, Some(CatalogMass::Subtype { .. }))
                    && catalog::amount(&item.definition_id, condition.as_deref())?.is_none())
            {
                return Err(
                    "Printed source mass cannot be overridden by an authored unit mass.".into(),
                );
            }
            PhysicalItemMass::Unit {
                mass: PhysicalMass::parse_pounds(pounds)?,
                condition: condition.clone(),
            }
        }
        PhysicalItemInput::Gross { pounds } => {
            if !matches!(
                printed,
                None | Some(CatalogMass::Unquantified | CatalogMass::ConditionalGross { .. })
            ) || !contents.is_empty()
            {
                return Err("Gross mass requires an unquantified object without separately tracked contents.".into());
            }
            PhysicalItemMass::Gross {
                mass: PhysicalMass::parse_pounds(pounds)?,
            }
        }
    })
}

fn accept(
    state: &mut CampaignState,
    meta: &CommandMeta,
    acceptance: &PhysicalFactAcceptance,
    sources: &PhysicalSources,
) -> Result<Vec<ItemInstance>, String> {
    if meta.issuer != CommandIssuer::Admin
        || meta.actor.is_some()
        || meta.campaign_id != state.campaign_id()
        || meta.expected_event_sequence != state.applied_event_sequence
        || meta.session_id
            != state
                .table
                .as_ref()
                .and_then(|t| t.active_session.as_ref())
                .map(|s| s.session_id)
    {
        return Err("Physical facts require the current trusted Host and session.".into());
    }
    if !offers(state, sources)?.contains(&acceptance.offer) {
        return Err("This physical-fact control is no longer current.".into());
    }
    let mut created = Vec::new();
    match (&acceptance.offer, &acceptance.input) {
        (PhysicalFactOffer::Enable { catalog }, PhysicalFactInput::Enable) => {
            if state.physical_facts.is_some()
                || *catalog != catalog::pin()?
                || state.campaign.ruleset.id != catalog.ruleset_id
                || state.campaign.ruleset.version != catalog.ruleset_version
            {
                return Err("Unsupported physical catalog activation.".into());
            }
            state.physical_facts = Some(Box::new(PhysicalFacts {
                schema_version: PHYSICAL_FACTS_VERSION,
                catalog: catalog.clone(),
                origin: meta.clone(),
                records: Vec::new(),
            }));
        }
        (
            PhysicalFactOffer::Body {
                actor,
                identity,
                previous,
            },
            PhysicalFactInput::Body { pounds, reason },
        ) => {
            let mass = pounds
                .as_deref()
                .map(PhysicalMass::parse_pounds)
                .transpose()?;
            insert(
                state,
                meta,
                PhysicalSubject::Body(*actor),
                *previous,
                PhysicalFactValue::Body {
                    identity: identity.clone(),
                    mass,
                },
                reason,
            )?;
        }
        (
            PhysicalFactOffer::Item {
                item,
                contents,
                previous,
            },
            PhysicalFactInput::Item { choice, reason },
        ) => {
            let mass = parse_item(item, contents, choice)?;
            insert(
                state,
                meta,
                PhysicalSubject::Item(item.id),
                *previous,
                PhysicalFactValue::Item {
                    definition_id: item.definition_id.clone(),
                    quantity: item.quantity,
                    state: item.state.clone(),
                    contents: contents.clone(),
                    mass,
                },
                reason,
            )?;
        }
        (
            PhysicalFactOffer::Coverage {
                actor,
                identity,
                allocations,
                previous,
            },
            PhysicalFactInput::Coverage { complete, reason },
        ) => {
            insert(
                state,
                meta,
                PhysicalSubject::Coverage(*actor),
                *previous,
                PhysicalFactValue::Coverage {
                    identity: identity.clone(),
                    allocations: allocations.clone(),
                    complete: *complete,
                },
                reason,
            )?;
        }
        (
            PhysicalFactOffer::PersonalItem { actor, identity: _ },
            PhysicalFactInput::PersonalItem {
                name,
                pounds,
                separate_from_listed,
                reason,
            },
        ) => {
            text(name, 200)?;
            if !separate_from_listed {
                return Err("Confirm this is separate additional apparel or payload, excluding the body and listed equipment.".into());
            }
            let item = ItemInstance {
                id: ItemId(meta.id.0),
                campaign_id: meta.campaign_id,
                definition_id: CUSTOM_LOAD_DEFINITION.into(),
                display_name: name.clone(),
                quantity: 1,
                owner: Ownership::Entity(*actor),
                custody: Custody::Entity(*actor),
                state: ItemState::Intact,
            };
            if state.items.contains_key(&item.id) || state.items.len() >= 4096 {
                return Err("New physical item identity is unavailable.".into());
            }
            let mass = PhysicalMass::parse_pounds(pounds)?;
            insert(
                state,
                meta,
                PhysicalSubject::Item(item.id),
                None,
                PhysicalFactValue::Item {
                    definition_id: item.definition_id.clone(),
                    quantity: 1,
                    state: ItemState::Intact,
                    contents: Vec::new(),
                    mass: PhysicalItemMass::Unit {
                        mass,
                        condition: None,
                    },
                },
                reason,
            )?;
            state.items.insert(item.id, item.clone());
            created.push(item);
        }
        (
            PhysicalFactOffer::Currency {
                character,
                identity,
                value_cp,
                lot,
                previous,
            },
            PhysicalFactInput::Currency { coins, reason },
        ) => {
            let (count, value) = coins.totals()?;
            if count == 0 || value != *value_cp {
                return Err(
                    "Actual coin denominations must represent the whole unchanged wallet value."
                        .into(),
                );
            }
            let actor = state
                .characters
                .get(character)
                .ok_or("Character is absent.")?
                .entity_id;
            let item = if let Some(lot) = lot {
                let old = state
                    .physical_facts
                    .as_ref()
                    .and_then(|f| f.get(PhysicalSubject::Currency(*character)))
                    .ok_or("Original currency realization is absent.")?;
                let PhysicalFactValue::Currency {
                    coins: original,
                    value_cp: original_value,
                    ..
                } = &old.value
                else {
                    return Err("Invalid original currency realization.".into());
                };
                if *original_value != *value_cp
                    || lot.quantity != original.totals()?.0
                    || lot.state != ItemState::Intact
                    || lot.definition_id != CURRENCY_LOT_DEFINITION
                {
                    return Err("Changed or destroyed coins require a real disposition; a fact correction cannot recreate them.".into());
                }
                let mut updated = lot.clone();
                updated.quantity = count;
                updated
            } else {
                ItemInstance {
                    id: ItemId(meta.id.0),
                    campaign_id: meta.campaign_id,
                    definition_id: CURRENCY_LOT_DEFINITION.into(),
                    display_name: "Recorded wallet coins".into(),
                    quantity: count,
                    owner: Ownership::Entity(actor),
                    custody: Custody::Entity(actor),
                    state: ItemState::Intact,
                }
            };
            if lot.is_none() && (state.items.contains_key(&item.id) || state.items.len() >= 4096) {
                return Err("Currency identity is unavailable.".into());
            }
            insert(
                state,
                meta,
                PhysicalSubject::Currency(*character),
                *previous,
                PhysicalFactValue::Currency {
                    identity: identity.clone(),
                    value_cp: *value_cp,
                    item: item.id,
                    coins: *coins,
                },
                reason,
            )?;
            state.items.insert(item.id, item.clone());
            // Existing lot replacement is observed separately by the mass delta.
            created.push(item);
        }
        _ => return Err("The entered fields do not match this physical-fact control.".into()),
    }
    Ok(created)
}

pub fn validate_shapes(state: &CampaignState) -> Result<(), String> {
    let Some(facts) = &state.physical_facts else {
        return Ok(());
    };
    if state.schema_version < 4
        || facts.schema_version != PHYSICAL_FACTS_VERSION
        || facts.catalog != catalog::pin()?
        || facts.records.len() > 4096
    {
        return Err("Unsupported physical facts or catalog.".into());
    }
    let origin = |m: &CommandMeta| {
        if m.id.0.is_nil()
            || m.campaign_id != state.campaign_id()
            || m.issuer != CommandIssuer::Admin
            || m.actor.is_some()
            || m.expected_event_sequence > state.applied_event_sequence
        {
            Err("Invalid physical-fact origin.".to_owned())
        } else {
            Ok(())
        }
    };
    origin(&facts.origin)?;
    let mut subjects = HashSet::new();
    let mut lots = HashSet::new();
    for fact in &facts.records {
        origin(&fact.origin)?;
        text(&fact.reason, 1000)?;
        if !subjects.insert(fact.subject) || fact.previous == Some(fact.origin.id) {
            return Err("Duplicate physical fact or cyclic amendment.".into());
        }
        let valid = match (&fact.subject, &fact.value) {
            (PhysicalSubject::Body(actor), PhysicalFactValue::Body { identity, mass }) => {
                state.entities.contains_key(actor)
                    && identity.len() == 16
                    && mass.is_none_or(|m| m.0 > 0)
            }
            (
                PhysicalSubject::Coverage(actor),
                PhysicalFactValue::Coverage {
                    identity,
                    allocations,
                    ..
                },
            ) => {
                state.entities.contains_key(actor)
                    && identity.len() == 16
                    && allocations.len() == 16
            }
            (
                PhysicalSubject::Item(item),
                PhysicalFactValue::Item {
                    definition_id,
                    contents,
                    mass,
                    ..
                },
            ) => {
                state.items.contains_key(item)
                    && !definition_id.is_empty()
                    && contents.len() <= 4096
                    && match mass {
                        PhysicalItemMass::Unit { mass, .. } | PhysicalItemMass::Gross { mass } => {
                            mass.0 > 0
                        }
                        PhysicalItemMass::NonstandardUnit { description, mass } => {
                            mass.0 > 0 && text(description, 200).is_ok()
                        }
                        _ => true,
                    }
            }
            (
                PhysicalSubject::Currency(character),
                PhysicalFactValue::Currency {
                    identity,
                    value_cp,
                    item,
                    coins,
                },
            ) => {
                state.characters.contains_key(character)
                    && state.items.contains_key(item)
                    && lots.insert(*item)
                    && identity.len() == 16
                    && coins.totals().is_ok_and(|(n, v)| n > 0 && v == *value_cp)
            }
            _ => false,
        };
        if !valid {
            return Err("Malformed physical fact subject, amount or scope.".into());
        }
    }
    Ok(())
}

/// Reconstruct the retained record from the exact accepted canonical command.
/// This is a comparison helper for original history, never an admission API.
pub fn accepted_fact(
    meta: &CommandMeta,
    acceptance: &PhysicalFactAcceptance,
) -> Result<Option<PhysicalFact>, String> {
    let (subject, previous, value, reason) = match (&acceptance.offer, &acceptance.input) {
        (PhysicalFactOffer::Enable { .. }, PhysicalFactInput::Enable) => return Ok(None),
        (
            PhysicalFactOffer::Body {
                actor,
                identity,
                previous,
            },
            PhysicalFactInput::Body { pounds, reason },
        ) => (
            PhysicalSubject::Body(*actor),
            *previous,
            PhysicalFactValue::Body {
                identity: identity.clone(),
                mass: pounds
                    .as_deref()
                    .map(PhysicalMass::parse_pounds)
                    .transpose()?,
            },
            reason,
        ),
        (
            PhysicalFactOffer::Item {
                item,
                contents,
                previous,
            },
            PhysicalFactInput::Item { choice, reason },
        ) => (
            PhysicalSubject::Item(item.id),
            *previous,
            PhysicalFactValue::Item {
                definition_id: item.definition_id.clone(),
                quantity: item.quantity,
                state: item.state.clone(),
                contents: contents.clone(),
                mass: parse_item(item, contents, choice)?,
            },
            reason,
        ),
        (
            PhysicalFactOffer::Coverage {
                actor,
                identity,
                allocations,
                previous,
            },
            PhysicalFactInput::Coverage { complete, reason },
        ) => (
            PhysicalSubject::Coverage(*actor),
            *previous,
            PhysicalFactValue::Coverage {
                identity: identity.clone(),
                allocations: allocations.clone(),
                complete: *complete,
            },
            reason,
        ),
        (
            PhysicalFactOffer::PersonalItem { .. },
            PhysicalFactInput::PersonalItem { pounds, reason, .. },
        ) => (
            PhysicalSubject::Item(ItemId(meta.id.0)),
            None,
            PhysicalFactValue::Item {
                definition_id: CUSTOM_LOAD_DEFINITION.into(),
                quantity: 1,
                state: ItemState::Intact,
                contents: Vec::new(),
                mass: PhysicalItemMass::Unit {
                    mass: PhysicalMass::parse_pounds(pounds)?,
                    condition: None,
                },
            },
            reason,
        ),
        (
            PhysicalFactOffer::Currency {
                character,
                identity,
                value_cp,
                lot,
                previous,
            },
            PhysicalFactInput::Currency { coins, reason },
        ) => (
            PhysicalSubject::Currency(*character),
            *previous,
            PhysicalFactValue::Currency {
                identity: identity.clone(),
                value_cp: *value_cp,
                item: lot.as_ref().map_or(ItemId(meta.id.0), |item| item.id),
                coins: *coins,
            },
            reason,
        ),
        _ => return Err("Physical fact fields disagree with their original control.".into()),
    };
    Ok(Some(PhysicalFact {
        subject,
        origin: meta.clone(),
        previous,
        value,
        reason: reason.clone(),
    }))
}
