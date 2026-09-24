use super::*;
use std::collections::BTreeMap;

/// Deterministic upper bound on geometry/feature visits in one perception query.
/// Storage bounds do not imply that their Cartesian product is safe to traverse.
pub const MAX_PERCEPTION_QUERY_WORK: usize = 8_000_000;
struct QueryWork(usize);
impl QueryWork {
    fn new() -> Self {
        Self(MAX_PERCEPTION_QUERY_WORK)
    }
    fn charge(&mut self, visits: usize) -> Result<(), SpatialError> {
        self.0 = self.0.checked_sub(visits).ok_or(SpatialError::Capacity)?;
        Ok(())
    }
    fn sight(&mut self, field: &Battlefield) -> Result<(), SpatialError> {
        self.charge(1 + field.obstacles.len() + field.terrain.len())
    }
}

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
    illumination_unchecked(encounter, point, &mut QueryWork::new())
}
fn illumination_unchecked(
    encounter: &TacticalEncounter,
    point: SpatialPoint,
    work: &mut QueryWork,
) -> Result<LightLevel, SpatialError> {
    let mut level = encounter.battlefield.ambient_light;
    work.charge(encounter.battlefield.terrain.len())?;
    if encounter
        .battlefield
        .terrain
        .iter()
        .any(|t| t.magical_darkness && t.volume.contains(point))
    {
        return Ok(LightLevel::Darkness);
    }
    if level == LightLevel::Bright {
        return Ok(level);
    }
    for light in &encounter.battlefield.lights {
        work.charge(1)?;
        if light.dim_radius == 0 {
            continue;
        }
        let origin = match light.attached_to {
            Some(actor) => {
                work.charge(encounter.participants.len())?;
                participant(encounter, actor)?.center().map_err(invalid)?
            }
            None => light.position,
        };
        let distance = grid_distance(origin, point)?;
        if distance > light.dim_radius {
            continue;
        }
        work.sight(&encounter.battlefield)?;
        if !geometry::clear_sight(&encounter.battlefield, origin, point)? {
            continue;
        }
        if light.bright_radius > 0 && distance <= light.bright_radius {
            return Ok(LightLevel::Bright);
        }
        if level == LightLevel::Darkness {
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
    perceive_with_work(encounter, state, observer, target, &mut QueryWork::new())
}
fn perceive_with_work(
    encounter: &TacticalEncounter,
    state: &CampaignState,
    observer: EntityId,
    target: EntityId,
    work: &mut QueryWork,
) -> Result<PerceptionResult, SpatialError> {
    let actor = participant(encounter, observer)?;
    let subject = participant(encounter, target)?;
    if !aware(state, observer) {
        return Ok(PerceptionResult {
            sees: false,
            precisely_located: false,
            modality: None,
            sight_disadvantage: false,
        });
    }
    let from = actor.center().map_err(invalid)?;
    let target_volume = subject.volume().map_err(invalid)?;
    let actor_conditions = conditions(state, observer);
    let target_conditions = conditions(state, target);
    let mut dim_visible = false;
    for point in geometry::samples(target_volume) {
        if within(from, point, actor.senses.blindsight)? {
            work.charge(encounter.battlefield.obstacles.len())?;
            // Composite cover may require adjudication without proving Total.
            // Absence of that proof is not proof of an opening: Blindsight needs
            // an actual unobstructed candidate ray (SRD177).
            if geometry::clear_effect(&encounter.battlefield, from, point)? {
                return Ok(PerceptionResult {
                    sees: true,
                    precisely_located: true,
                    modality: Some(PerceptionModality::Blindsight),
                    sight_disadvantage: false,
                });
            }
        }
        work.sight(&encounter.battlefield)?;
        if actor_conditions.contains(&Condition::Blinded)
            || !geometry::clear_sight_with_truesight(
                &encounter.battlefield,
                from,
                point,
                actor.senses.truesight,
            )?
        {
            continue;
        }
        let true_sight = within(from, point, actor.senses.truesight)?;
        if target_conditions.contains(&Condition::Invisible) && !true_sight {
            continue;
        }
        work.charge(encounter.battlefield.terrain.len())?;
        let magical = encounter
            .battlefield
            .terrain
            .iter()
            .any(|t| t.magical_darkness && t.volume.contains(point));
        let light = illumination_unchecked(encounter, point, work)?;
        let darkvision = within(from, point, actor.senses.darkvision)? && !magical;
        let sight = match light {
            LightLevel::Bright => Some(false),
            LightLevel::Dim => Some(!darkvision && !true_sight),
            LightLevel::Darkness if true_sight => Some(false),
            LightLevel::Darkness if darkvision => Some(true),
            _ => None,
        };
        if let Some(mut disadvantage) = sight {
            work.charge(encounter.battlefield.terrain.len())?;
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
    if observer == target {
        // Awareness of one's own body is enough for a self/touch target, but it
        // cannot satisfy an effect which explicitly requires a target one can see.
        return Ok(PerceptionResult {
            sees: false,
            precisely_located: true,
            modality: None,
            sight_disadvantage: false,
        });
    }
    work.charge(encounter.battlefield.terrain.len() * 2)?;
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
    /// An unaware actor cannot discover a new location after being moved externally.
    pub position: Option<SpatialPoint>,
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
    let mut work = QueryWork::new();
    let mut contacts = Vec::new();
    let memory = encounter.knowledge.iter().find(|k| k.observer == observer);
    for target in &encounter.participants {
        if target.entity_id == observer {
            continue;
        }
        let perception =
            perceive_with_work(encounter, state, observer, target.entity_id, &mut work)?;
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
    if !aware(state, observer) {
        return Ok(ActorTacticalView {
            observer,
            position: None,
            contacts,
            cells: cells.into_values().collect(),
        });
    }
    let from = actor.center().map_err(invalid)?;
    let field = &encounter.battlefield;
    let blind = conditions(state, observer).contains(&Condition::Blinded);
    for x in (field.bounds.min.x..field.bounds.max.x).step_by(10) {
        for y in (field.bounds.min.y..field.bounds.max.y).step_by(10) {
            work.charge(1)?;
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
            work.charge(field.terrain.len())?;
            let magical = field
                .terrain
                .iter()
                .any(|t| t.magical_darkness && t.volume.contains(point));
            let illuminated = illumination_unchecked(encounter, point, &mut work)?
                != LightLevel::Darkness
                || within(from, point, actor.senses.truesight)?
                || (!magical && within(from, point, actor.senses.darkvision)?);
            work.sight(field)?;
            let mut visible = if special {
                geometry::clear_effect(field, from, point)?
            } else {
                !blind
                    && illuminated
                    && geometry::clear_sight_with_truesight(
                        field,
                        from,
                        point,
                        actor.senses.truesight,
                    )?
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
                work.charge(field.obstacles.len())?;
                for obstacle in field
                    .obstacles
                    .iter()
                    .filter(|o| o.observable && o.volume.intersects(cell))
                {
                    work.charge(1)?;
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
                    work.sight(field)?;
                    work.charge(field.terrain.len())?;
                    if surface.validate().is_ok()
                        && geometry::clear_sight_with_truesight(
                            field,
                            from,
                            surface,
                            actor.senses.truesight,
                        )?
                        && (illumination_unchecked(encounter, surface, &mut work)?
                            != LightLevel::Darkness
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
                work.sight(field)?;
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
        position: Some(actor.position),
        contacts,
        cells: cells.into_values().collect(),
    })
}
