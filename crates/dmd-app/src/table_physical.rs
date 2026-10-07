use crate::*;
use dmd_domain::*;
use dmd_rules::physical_facts::catalog::{self, CatalogMass};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TablePhysicalView<K = PhysicalFactOffer> {
    pub version: u32,
    pub actors: Vec<TablePhysicalActor>,
    pub controls: Vec<TablePhysicalControl<K>>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TablePhysicalActor {
    pub actor: EntityId,
    pub name: String,
    pub body_pounds: Option<String>,
    pub load: Option<PhysicalLoad>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TablePhysicalControl<K = PhysicalFactOffer> {
    pub key: K,
    pub label: String,
    pub kind: String,
    pub source: Option<String>,
    pub conditions: Vec<String>,
    pub unit: bool,
    pub gross: bool,
    pub wallet_cp: Option<u32>,
}

pub(crate) fn view(
    read: &dmd_rules::table::TableRead<'_>,
    viewer: &TableViewer,
) -> Result<TablePhysicalView, String> {
    let state = read.state();
    let host = matches!(viewer, TableViewer::Host);
    let facts = state
        .physical_facts
        .as_ref()
        .ok_or("Physical facts are not active.")?;
    let mut ids = state.entities.keys().copied().collect::<Vec<_>>();
    ids.sort_by_key(|id| id.0);
    let mut actors = Vec::new();
    for actor in ids {
        let owned = matches!(viewer, TableViewer::Player(player) if state.characters.values().any(|c| c.entity_id == actor && c.controlling_player_id == Some(*player)));
        if !host && !owned {
            continue;
        }
        let identity = dmd_rules::physical_facts::actor_identity(state, actor)?;
        let body_pounds = match facts.get(PhysicalSubject::Body(actor)).map(|f| &f.value) {
            Some(PhysicalFactValue::Body {
                identity: bound,
                mass: Some(mass),
            }) if *bound == identity => Some(mass.pounds()),
            _ => None,
        };
        // An opaque aggregate may disclose another owner's private physical facts.
        // Withhold the whole load rather than silently omit any physical object.
        let private_load = state.items.values().any(|item| {
            let mut custody = item.custody;
            for _ in 0..=state.items.len() {
                match custody {
                    Custody::Entity(holder) => {
                        return holder == actor && item.owner != Ownership::Entity(actor);
                    }
                    Custody::Container(id) => match state.items.get(&id) {
                        Some(parent) => custody = parent.custody,
                        None => return true,
                    },
                    _ => return false,
                }
            }
            true
        });
        actors.push(TablePhysicalActor {
            actor,
            name: state.entities[&actor].display_name.clone(),
            body_pounds,
            load: if host || !private_load {
                Some(read.physical_load(actor)?)
            } else {
                None
            },
        });
    }
    let mut controls = Vec::new();
    if host {
        for offer in read.physical_fact_offers()? {
            let mut control = TablePhysicalControl {
                key: offer.clone(),
                label: String::new(),
                kind: String::new(),
                source: None,
                conditions: Vec::new(),
                unit: false,
                gross: false,
                wallet_cp: None,
            };
            match offer {
                PhysicalFactOffer::Enable { .. } => continue,
                PhysicalFactOffer::Body { actor, .. } => {
                    control.kind = "body".into();
                    control.label =
                        format!("{}: unladen body", state.entities[&actor].display_name);
                }
                PhysicalFactOffer::Coverage { actor, .. } => {
                    control.kind = "coverage".into();
                    control.label = format!(
                        "{}: confirm all apparel and payload is listed",
                        state.entities[&actor].display_name
                    );
                }
                PhysicalFactOffer::PersonalItem { actor, .. } => {
                    control.kind = "personal".into();
                    control.label = format!(
                        "{}: record additional apparel or payload",
                        state.entities[&actor].display_name
                    );
                }
                PhysicalFactOffer::Currency {
                    character,
                    value_cp,
                    ..
                } => {
                    control.kind = "currency".into();
                    control.label = format!(
                        "{}: actual wallet coins",
                        state.characters[&character].display_name
                    );
                    control.wallet_cp = Some(value_cp);
                }
                PhysicalFactOffer::Item { item, contents, .. } => {
                    control.kind = "item".into();
                    control.label = format!("{} ({} units)", item.display_name, item.quantity);
                    match catalog::entry(&item.definition_id).map(|entry| &entry.mass) {
                        Some(CatalogMass::Intrinsic { mass }) => {
                            control.source = Some(format!("{} lb per physical unit", mass.pounds()))
                        }
                        Some(CatalogMass::ConditionalGross { condition, mass }) => {
                            control.source = Some(format!(
                                "{} lb when {condition}, including contents",
                                mass.pounds()
                            ));
                            control.conditions.push(condition.clone());
                            control.gross = contents.is_empty();
                        }
                        Some(CatalogMass::Subtype { variants }) => {
                            control.unit = variants.iter().any(|v| v.mass.is_none());
                            control.conditions = variants.iter().map(|v| v.id.clone()).collect();
                            control.source = Some("Select the actual symbol form; an unquantified form remains unknown.".into());
                        }
                        Some(CatalogMass::Unquantified) | None => {
                            control.unit = true;
                            control.gross = contents.is_empty();
                        }
                    }
                }
            }
            controls.push(control);
        }
    }
    Ok(TablePhysicalView {
        version: 5,
        actors,
        controls,
    })
}
