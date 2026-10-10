use super::*;
use crate::tactical::grapple::execution::ReadContext;

pub(crate) fn carried_by(
    state: &CampaignState,
    item: &ItemInstance,
) -> Result<Option<EntityId>, String> {
    let mut custody = item.custody;
    let mut seen = HashSet::new();
    loop {
        match custody {
            Custody::Entity(actor) => return Ok(Some(actor)),
            Custody::Container(container) => {
                if !seen.insert(container) || seen.len() > 4096 {
                    return Err("Physical containment is cyclic or exceeds its bound.".into());
                }
                custody = state
                    .items
                    .get(&container)
                    .ok_or("Physical container is absent.")?
                    .custody;
            }
            Custody::Location(_) | Custody::Missing | Custody::Destroyed => return Ok(None),
        }
    }
}

fn item_mass(
    state: &CampaignState,
    sources: &PhysicalSources,
    item: &ItemInstance,
) -> Result<Option<PhysicalMass>, String> {
    let facts = state
        .physical_facts
        .as_ref()
        .ok_or("Physical facts are not enabled.")?;
    let currency = facts.records.iter().find_map(|f| match &f.value {
        PhysicalFactValue::Currency {
            item: id,
            coins,
            identity,
            value_cp,
        } if *id == item.id => Some((f.subject, *coins, identity, *value_cp)),
        _ => None,
    });
    if item.definition_id == CURRENCY_LOT_DEFINITION || currency.is_some() {
        if item.definition_id != CURRENCY_LOT_DEFINITION {
            return Ok(None);
        }
        let value = currency;
        let Some((PhysicalSubject::Currency(character), coins, identity, value_cp)) = value else {
            return Ok(None);
        };
        let Some(pc) = state.characters.get(&character) else {
            return Ok(None);
        };
        let current_value = state
            .table
            .as_ref()
            .and_then(|t| t.character_profiles.get(&character))
            .map(|p| p.money_cp);
        if current_value != Some(value_cp)
            || actor_identity(state, pc.entity_id)? != *identity
            || coins.totals()?.0 != item.quantity
            || item.state != ItemState::Intact
        {
            return Ok(None);
        }
        return PhysicalMass(20_000).times(item.quantity).map(Some);
    }
    // A physical coin lot must still match its accepted whole-wallet count and
    // condition. The ordinary spent-ammunition shortcut cannot realize a wallet.
    if item.state == ItemState::Spent && item.quantity == 0 {
        return Ok(Some(PhysicalMass(0)));
    }
    if item.quantity == 0 || !matches!(item.state, ItemState::Intact | ItemState::Damaged) {
        return Ok(None);
    }
    let fact = facts.get(PhysicalSubject::Item(item.id));
    if let Some(PhysicalFact {
        value:
            PhysicalFactValue::Item {
                definition_id,
                quantity,
                state: recorded_state,
                contents,
                mass,
            },
        ..
    }) = fact
    {
        if definition_id != &item.definition_id {
            return Ok(None);
        }
        return match mass {
            PhysicalItemMass::Unknown => Ok(None),
            PhysicalItemMass::Unit { mass, .. }
            | PhysicalItemMass::NonstandardUnit { mass, .. } => mass.times(item.quantity).map(Some),
            PhysicalItemMass::Gross { mass } => Ok((*quantity == item.quantity
                && recorded_state == &item.state
                && contents.is_empty()
                && children(state, item.id).is_empty())
            .then_some(*mass)),
            PhysicalItemMass::Catalog { condition } => {
                let inclusive = matches!(
                    catalog::entry(definition_id).map(|e| &e.mass),
                    Some(catalog::CatalogMass::ConditionalGross { .. })
                );
                if inclusive
                    && (*quantity != item.quantity
                        || recorded_state != &item.state
                        || !children(state, item.id).is_empty())
                {
                    return Ok(None);
                }
                catalog::amount(definition_id, condition.as_deref())?
                    .map(|mass| mass.times(item.quantity))
                    .transpose()
            }
        };
    }
    if sources.contains(item)
        && let Some(catalog::MassEntry {
            mass: catalog::CatalogMass::Intrinsic { mass },
            ..
        }) = catalog::entry(&item.definition_id)
    {
        return mass.times(item.quantity).map(Some);
    }
    Ok(None)
}

pub(crate) fn load(read: &ReadContext<'_>, actor: EntityId) -> Result<PhysicalLoad, String> {
    let state = read.state();
    let sources = read.physical_sources().map_err(|e| e.to_string())?;
    if !state.validate_references().is_empty() || state.items.len() > 4096 {
        return Err("Physical inventory has invalid references or exceeds its bound.".into());
    }
    let facts = state
        .physical_facts
        .as_ref()
        .ok_or("Physical facts are not enabled.")?;
    let identity = actor_identity(state, actor)?;
    let mut known = PhysicalMass(0);
    let mut unresolved = Vec::new();
    let mut item_details = Vec::new();
    let mut items = state.items.values().collect::<Vec<_>>();
    items.sort_by_key(|item| item.id.0);
    for item in items {
        if carried_by(state, item)? == Some(actor) {
            let mass = item_mass(state, sources, item)?;
            let basis = if mass.is_none() {
                "Mass or physical condition is unresolved".into()
            } else if item.definition_id == CURRENCY_LOT_DEFINITION {
                "Recorded actual coins; 50 coins per pound".into()
            } else if let Some(PhysicalFact {
                value: PhysicalFactValue::Item { mass, .. },
                ..
            }) = facts.get(PhysicalSubject::Item(item.id))
            {
                match mass {
                    PhysicalItemMass::Catalog { .. } => {
                        "Printed mass with accepted ordinary applicability or condition".into()
                    }
                    PhysicalItemMass::NonstandardUnit { description, .. } => {
                        format!("Accepted nonstandard physical object: {description}")
                    }
                    PhysicalItemMass::Unit { .. } => "Authored physical weight per unit".into(),
                    PhysicalItemMass::Gross { .. } => {
                        "Authored current weight for this whole object".into()
                    }
                    PhysicalItemMass::Unknown => "Unresolved".into(),
                }
            } else {
                "Printed source mass for the actual materialized item".into()
            };
            item_details.push(PhysicalItemLoad {
                item: item.id,
                name: item.display_name.clone(),
                quantity: item.quantity,
                pounds: mass.map(PhysicalMass::pounds),
                basis,
            });
            match mass {
                Some(mass) => known = known.plus(mass)?,
                None => unresolved.push(format!(
                    "{}: mass or physical condition is unresolved",
                    item.display_name
                )),
            }
        }
    }
    let coverage = sources.coverage(state, actor);
    if !matches!(facts.get(PhysicalSubject::Coverage(actor)).map(|f| &f.value), Some(PhysicalFactValue::Coverage { identity: bound, allocations, complete: true }) if bound == &identity && coverage.as_ref() == Ok(allocations))
    {
        unresolved.push("Additional apparel and payload coverage is not confirmed for the current body and starting allocations.".into());
    }
    for character in state.characters.values().filter(|c| c.entity_id == actor) {
        if state
            .table
            .as_ref()
            .and_then(|t| t.character_profiles.get(&character.id))
            .is_some_and(|p| p.money_cp > 0)
        {
            let represented = facts
                .get(PhysicalSubject::Currency(character.id))
                .and_then(|f| match &f.value {
                    PhysicalFactValue::Currency { item, .. } => state.items.get(item),
                    _ => None,
                });
            let represented_mass = represented
                .map(|item| item_mass(state, sources, item))
                .transpose()?
                .flatten();
            if represented_mass.is_none() {
                unresolved
                    .push("The nonzero wallet's actual physical coins are unresolved.".into());
            }
        }
    }
    let complete = unresolved.is_empty();
    let body_and_load_pounds = if complete {
        match facts.get(PhysicalSubject::Body(actor)).map(|f| &f.value) {
            Some(PhysicalFactValue::Body {
                identity: bound,
                mass: Some(mass),
            }) if *bound == identity => Some(known.plus(*mass)?.pounds()),
            _ => None,
        }
    } else {
        None
    };
    Ok(PhysicalLoad {
        items: item_details,
        known_pounds: known.pounds(),
        complete,
        unresolved,
        body_and_load_pounds,
    })
}
