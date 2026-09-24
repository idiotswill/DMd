use super::*;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PerceptionResult {
    pub sees: bool,
    pub precisely_located: bool,
    pub modality: Option<PerceptionModality>,
    pub sight_disadvantage: bool,
}
fn within(from: SpatialPoint, to: SpatialPoint, radius: u32) -> Result<bool, SpatialError> {
    Ok(radius > 0 && grid_distance(from, to)? <= radius)
}

pub fn illumination(
    encounter: &TacticalEncounter,
    point: SpatialPoint,
) -> Result<LightLevel, SpatialError> {
    encounter.validate_geometry().map_err(invalid)?;
    point.validate().map_err(invalid)?;
    illumination_unchecked(encounter, point)
}
fn illumination_unchecked(
    encounter: &TacticalEncounter,
    point: SpatialPoint,
) -> Result<LightLevel, SpatialError> {
    let mut level = encounter.battlefield.ambient_light;
    if encounter
        .battlefield
        .terrain
        .iter()
        .any(|t| t.magical_darkness && t.volume.contains(point))
    {
        return Ok(LightLevel::Darkness);
    }
    for light in &encounter.battlefield.lights {
        let origin = match light.attached_to {
            Some(actor) => participant(encounter, actor)?.center().map_err(invalid)?,
            None => light.position,
        };
        if !geometry::clear_sight(&encounter.battlefield, origin, point)? {
            continue;
        }
        if grid_distance(origin, point)? <= light.bright_radius {
            return Ok(LightLevel::Bright);
        }
        if grid_distance(origin, point)? <= light.dim_radius && level == LightLevel::Darkness {
            level = LightLevel::Dim;
        }
    }
    Ok(level)
}
fn shared_surface(
    encounter: &TacticalEncounter,
    a: &TacticalParticipant,
    b: &TacticalParticipant,
) -> bool {
    let surfaces = |p: &TacticalParticipant| {
        let mut result = Vec::new();
        if p.position.z == encounter.battlefield.floor_z {
            result.push(encounter.battlefield.floor_surface.as_str());
        }
        for terrain in &encounter.battlefield.terrain {
            let horizontal = p.position.x < terrain.volume.max.x
                && p.position.x + p.size.footprint_units() > terrain.volume.min.x
                && p.position.y < terrain.volume.max.y
                && p.position.y + p.size.footprint_units() > terrain.volume.min.y;
            if let Some(surface) = terrain.surface.as_deref()
                && horizontal
                && ((terrain.supports_top && p.position.z == terrain.volume.max.z)
                    || (terrain.water && terrain.volume.contains(p.position)))
            {
                result.push(surface);
            }
        }
        result
    };
    let left = surfaces(a);
    let right = surfaces(b);
    left.iter().any(|surface| right.contains(surface))
}

pub fn perceive(
    encounter: &TacticalEncounter,
    state: &CampaignState,
    observer: EntityId,
    target: EntityId,
) -> Result<PerceptionResult, SpatialError> {
    validate_encounter(encounter, state)?;
    perceive_unchecked(encounter, state, observer, target)
}
pub(super) fn perceive_unchecked(
    encounter: &TacticalEncounter,
    state: &CampaignState,
    observer: EntityId,
    target: EntityId,
) -> Result<PerceptionResult, SpatialError> {
    let actor = participant(encounter, observer)?;
    let subject = participant(encounter, target)?;
    if observer == target {
        return Ok(PerceptionResult {
            sees: true,
            precisely_located: true,
            modality: Some(PerceptionModality::Sight),
            sight_disadvantage: false,
        });
    }
    let from = actor.center().map_err(invalid)?;
    let target_volume = subject.volume().map_err(invalid)?;
    let actor_conditions = conditions(state, observer);
    let target_conditions = conditions(state, target);
    let cover = geometry::cover_unchecked(encounter, from, target_volume, &[observer, target])?;
    let mut dim_visible = false;
    for point in geometry::samples(target_volume) {
        if cover.degree != CoverDegree::Total && within(from, point, actor.senses.blindsight)? {
            return Ok(PerceptionResult {
                sees: true,
                precisely_located: true,
                modality: Some(PerceptionModality::Blindsight),
                sight_disadvantage: false,
            });
        }
        if actor_conditions.contains(&Condition::Blinded)
            || !geometry::clear_sight(&encounter.battlefield, from, point)?
        {
            continue;
        }
        let true_sight = within(from, point, actor.senses.truesight)?;
        if target_conditions.contains(&Condition::Invisible) && !true_sight {
            continue;
        }
        let magical = encounter
            .battlefield
            .terrain
            .iter()
            .any(|t| t.magical_darkness && t.volume.contains(point));
        let light = illumination_unchecked(encounter, point)?;
        let darkvision = within(from, point, actor.senses.darkvision)? && !magical;
        let sight = match light {
            LightLevel::Bright => Some(false),
            LightLevel::Dim => Some(!darkvision && !true_sight),
            LightLevel::Darkness if true_sight => Some(false),
            LightLevel::Darkness if darkvision => Some(true),
            _ => None,
        };
        if let Some(mut disadvantage) = sight {
            disadvantage |= encounter
                .battlefield
                .terrain
                .iter()
                .any(|t| t.obscuration == Obscuration::Light && t.volume.contains(point));
            if !disadvantage {
                return Ok(PerceptionResult {
                    sees: true,
                    precisely_located: true,
                    modality: Some(PerceptionModality::Sight),
                    sight_disadvantage: false,
                });
            }
            dim_visible = true;
        }
    }
    if dim_visible {
        return Ok(PerceptionResult {
            sees: true,
            precisely_located: true,
            modality: Some(PerceptionModality::Sight),
            sight_disadvantage: true,
        });
    }
    let located = within(
        from,
        subject.center().map_err(invalid)?,
        actor.senses.tremorsense,
    )? && shared_surface(encounter, actor, subject);
    Ok(PerceptionResult {
        sees: false,
        precisely_located: located,
        modality: located.then_some(PerceptionModality::Tremorsense),
        sight_disadvantage: false,
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContactStatus {
    Seen,
    Located,
    Remembered,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TacticalContactView {
    pub entity_id: EntityId,
    pub label: Option<String>,
    pub position: SpatialPoint,
    pub status: ContactStatus,
    pub modality: PerceptionModality,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TacticalCellView {
    pub position: SpatialPoint,
    pub difficult: bool,
    pub blocked: bool,
    pub currently_seen: bool,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActorTacticalView {
    pub observer: EntityId,
    pub position: SpatialPoint,
    pub contacts: Vec<TacticalContactView>,
    pub cells: Vec<TacticalCellView>,
}

/// Enemy policy and player rendering consume this capability-restricted DTO, never
/// the encounter aggregate. Remembered contacts use the saved position, not truth.
pub fn project_actor_view(
    encounter: &TacticalEncounter,
    state: &CampaignState,
    observer: EntityId,
) -> Result<ActorTacticalView, SpatialError> {
    validate_encounter(encounter, state)?;
    let actor = participant(encounter, observer)?;
    let mut contacts = Vec::new();
    let memory = encounter.knowledge.iter().find(|k| k.observer == observer);
    for target in &encounter.participants {
        if target.entity_id == observer {
            continue;
        }
        let perception = perceive_unchecked(encounter, state, observer, target.entity_id)?;
        if perception.precisely_located {
            contacts.push(TacticalContactView {
                entity_id: target.entity_id,
                label: perception.sees.then(|| target.public_label.clone()),
                position: target.position,
                status: if perception.sees {
                    ContactStatus::Seen
                } else {
                    ContactStatus::Located
                },
                modality: perception
                    .modality
                    .ok_or_else(|| invalid("located contact lacks a modality"))?,
            });
        } else if let Some(known) =
            memory.and_then(|m| m.contacts.iter().find(|k| k.target == target.entity_id))
        {
            contacts.push(TacticalContactView {
                entity_id: known.target,
                label: None,
                position: known.position,
                status: ContactStatus::Remembered,
                modality: known.modality,
            });
        }
    }
    contacts.sort_by_key(|contact| contact.entity_id.0);
    let mut cells = BTreeMap::new();
    if let Some(memory) = memory {
        for cell in &memory.terrain {
            cells.insert(
                cell.position,
                TacticalCellView {
                    position: cell.position,
                    difficult: cell.difficult,
                    blocked: cell.blocked,
                    currently_seen: false,
                },
            );
        }
    }
    let from = actor.center().map_err(invalid)?;
    let field = &encounter.battlefield;
    let blind = conditions(state, observer).contains(&Condition::Blinded);
    for x in (field.bounds.min.x..field.bounds.max.x).step_by(10) {
        for y in (field.bounds.min.y..field.bounds.max.y).step_by(10) {
            let position = SpatialPoint {
                x,
                y,
                z: field.floor_z,
            };
            let point = SpatialPoint {
                x: x + 5,
                y: y + 5,
                z: field.floor_z + 1,
            };
            let special = within(from, point, actor.senses.blindsight)?;
            let magical = field
                .terrain
                .iter()
                .any(|t| t.magical_darkness && t.volume.contains(point));
            let illuminated = illumination_unchecked(encounter, point)? != LightLevel::Darkness
                || within(from, point, actor.senses.truesight)?
                || (!magical && within(from, point, actor.senses.darkvision)?);
            let mut visible = if special {
                geometry::clear_effect(field, from, point)?
            } else {
                !blind && illuminated && geometry::clear_sight(field, from, point)?
            };
            // A wall's near face can be visible although the center of its occupied
            // square is behind that face. Never reveal the interior/backside cells.
            if !visible && !blind {
                let cell = SpatialBox {
                    min: position,
                    max: SpatialPoint {
                        x: x + 10,
                        y: y + 10,
                        z: field.floor_z + 1,
                    },
                };
                for obstacle in field
                    .obstacles
                    .iter()
                    .filter(|o| o.observable && o.volume.intersects(cell))
                {
                    let clipped = SpatialBox {
                        min: SpatialPoint {
                            x: cell.min.x.max(obstacle.volume.min.x),
                            y: cell.min.y.max(obstacle.volume.min.y),
                            z: cell.min.z.max(obstacle.volume.min.z),
                        },
                        max: SpatialPoint {
                            x: cell.max.x.min(obstacle.volume.max.x),
                            y: cell.max.y.min(obstacle.volume.max.y),
                            z: cell.max.z.min(obstacle.volume.max.z),
                        },
                    };
                    let outside = |value: i32, min: i32, max: i32| {
                        if value < min {
                            min - 1
                        } else if value >= max {
                            max
                        } else {
                            value
                        }
                    };
                    let surface = SpatialPoint {
                        x: outside(from.x, clipped.min.x, clipped.max.x),
                        y: outside(from.y, clipped.min.y, clipped.max.y),
                        z: outside(from.z, clipped.min.z, clipped.max.z),
                    };
                    if surface.validate().is_ok()
                        && geometry::clear_sight(field, from, surface)?
                        && (illumination_unchecked(encounter, surface)? != LightLevel::Darkness
                            || within(from, surface, actor.senses.truesight)?
                            || (within(from, surface, actor.senses.darkvision)?
                                && !field
                                    .terrain
                                    .iter()
                                    .any(|t| t.magical_darkness && t.volume.contains(surface))))
                    {
                        visible = true;
                        break;
                    }
                }
            }
            if visible {
                cells.insert(
                    position,
                    TacticalCellView {
                        position,
                        difficult: field
                            .terrain
                            .iter()
                            .any(|t| t.observable && t.difficult && t.volume.contains(point)),
                        blocked: field.obstacles.iter().any(|o| {
                            o.observable
                                && o.blocks_movement
                                && o.volume.intersects(SpatialBox {
                                    min: position,
                                    max: SpatialPoint {
                                        x: x + 10,
                                        y: y + 10,
                                        z: field.floor_z + i32::try_from(actor.height).unwrap_or(1),
                                    },
                                })
                        }),
                        currently_seen: true,
                    },
                );
            }
        }
    }
    Ok(ActorTacticalView {
        observer,
        position: actor.position,
        contacts,
        cells: cells.into_values().collect(),
    })
}
