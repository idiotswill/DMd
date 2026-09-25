use super::*;
use crate::spatial::{SpatialArea, SpatialDirection, area_reaches_point, segment_intersects};
use std::collections::BTreeMap;

/// Query work counts actual sample/blocker visits, not only stored object count.
pub const MAX_AREA_QUERY_WORK: usize = 8_000_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundAreaGeometry {
    aim: TacticalAreaAim,
    policy: TacticalAreaGridPolicy,
    targets: Vec<TacticalAreaTarget>,
}
impl BoundAreaGeometry {
    pub fn aim(&self) -> TacticalAreaAim {
        self.aim
    }
    pub fn policy(&self) -> TacticalAreaGridPolicy {
        self.policy
    }
    /// Truth-only result. A player preview must project geometry without these
    /// identities/counts/cover grades; unseen creatures are still affected.
    pub fn targets(&self) -> &[TacticalAreaTarget] {
        &self.targets
    }
}

fn charge(work: &mut usize, amount: usize) -> Result<(), RulesError> {
    *work = work
        .checked_sub(amount)
        .ok_or_else(|| unavailable("area geometry exceeds its bounded work capacity"))?;
    Ok(())
}
fn spatial(error: crate::spatial::SpatialError) -> RulesError {
    invalid(error.to_string())
}

/// Centers of each actually occupied part of a 5ft cell/band. A Tiny creature or
/// short top band does not inherit the empty portion's center. Where a midpoint
/// is between half-foot coordinates, round toward the occupied minimum on that
/// axis. This is explicit map adjudication, not a source-prescribed raster rule.
fn centers(minimum: i32, maximum: i32, grid_origin: i32) -> Vec<i32> {
    let first =
        grid_origin + (minimum - grid_origin).div_euclid(GRID_SQUARE_UNITS) * GRID_SQUARE_UNITS;
    (first..maximum)
        .step_by(GRID_SQUARE_UNITS as usize)
        .map(|cell| {
            let low = cell.max(minimum);
            let high = (cell + GRID_SQUARE_UNITS).min(maximum);
            low + (high - low) / 2
        })
        .collect()
}

fn shape(
    encounter: &TacticalEncounter,
    program: &AreaProgram,
    aim: TacticalAreaAim,
) -> Result<SpatialArea, RulesError> {
    aim.origin.validate().map_err(invalid)?;
    aim.toward.validate().map_err(invalid)?;
    let source = encounter
        .participant(program.actor())
        .ok_or_else(|| unavailable("area source is outside the encounter"))?;
    let volume = source.volume().map_err(invalid)?;
    // Source origin may lie on the occupied surface, including an outward face.
    if aim.origin.x < volume.min.x
        || aim.origin.x > volume.max.x
        || aim.origin.y < volume.min.y
        || aim.origin.y > volume.max.y
        || aim.origin.z < volume.min.z
        || aim.origin.z > volume.max.z
    {
        return Err(unavailable(
            "the area must originate from the source creature's space",
        ));
    }
    let area = SpatialArea::Cone {
        origin: aim.origin,
        direction: SpatialDirection {
            x: aim.toward.x - aim.origin.x,
            y: aim.toward.y - aim.origin.y,
            z: aim.toward.z - aim.origin.z,
        },
        length: program.length(),
        include_origin: aim.include_origin,
    };
    area.validate(encounter).map_err(spatial)?;
    Ok(area)
}

/// The host policy is read from the authenticated encounter attachment.
/// Ordinary declarations cannot choose or override it.
/// No sight/known-victim test is appropriate for an unguided source cone.
pub fn bind_area_geometry(
    encounter: &TacticalEncounter,
    state: &CampaignState,
    program: &AreaProgram,
    aim: TacticalAreaAim,
) -> Result<BoundAreaGeometry, RulesError> {
    let policy = encounter
        .area_grid_policy
        .ok_or_else(|| unavailable("the host must choose how areas use this map"))?;
    crate::spatial::validate_encounter(encounter, state).map_err(spatial)?;
    let area = shape(encounter, program, aim)?;
    let grid = encounter.battlefield.bounds.min;
    let mut work = MAX_AREA_QUERY_WORK;
    let mut reached = BTreeMap::new();
    let mut targets = vec![];
    let rules = state.rules.as_ref().ok_or(RulesError::Uninitialized)?;
    let mut participants = encounter.participants.iter().collect::<Vec<_>>();
    participants.sort_by_key(|participant| participant.entity_id);
    for participant in participants {
        let actor = participant.entity_id;
        let Some(entity) = rules.entities.get(&actor) else {
            continue;
        };
        if entity.death.dead
            || state.entities.get(&actor).is_none_or(|e| {
                !matches!(
                    e.kind,
                    EntityKind::Character | EntityKind::Npc | EntityKind::Creature
                )
            })
        {
            continue;
        }
        let volume = participant.volume().map_err(invalid)?;
        let xs = centers(volume.min.x, volume.max.x, grid.x);
        let ys = centers(volume.min.y, volume.max.y, grid.y);
        let zs = centers(volume.min.z, volume.max.z, grid.z);
        let sample_count = xs
            .len()
            .checked_mul(ys.len())
            .and_then(|n| n.checked_mul(zs.len()))
            .ok_or_else(|| invalid("area occupied sample capacity overflow"))?;
        charge(&mut work, sample_count)?;
        let mut blocked = 0usize;
        let mut included = false;
        let mut cover = CoverDegree::None;
        for &x in &xs {
            for &y in &ys {
                for &z in &zs {
                    let point = SpatialPoint { x, y, z };
                    let mut total_blocked = false;
                    charge(
                        &mut work,
                        encounter.battlefield.obstacles.len() + encounter.participants.len(),
                    )?;
                    for obstacle in &encounter.battlefield.obstacles {
                        if segment_intersects(aim.origin, point, obstacle.volume)
                            .map_err(spatial)?
                        {
                            if obstacle.cover == CoverDegree::Total {
                                total_blocked = true;
                            } else {
                                cover = cover.max(obstacle.cover);
                            }
                        }
                    }
                    for other in &encounter.participants {
                        if other.entity_id != actor
                            && other.entity_id != program.actor()
                            && segment_intersects(
                                aim.origin,
                                point,
                                other.volume().map_err(invalid)?,
                            )
                            .map_err(spatial)?
                        {
                            cover = cover.max(CoverDegree::Half);
                        }
                    }
                    blocked += usize::from(total_blocked);
                    let reaches = if let Some(value) = reached.get(&point) {
                        *value
                    } else {
                        // The public point query also validates the complete geometry.
                        // Count those visits as well as the clear-effect blocker scan.
                        charge(
                            &mut work,
                            64 + encounter.participants.len()
                                + 2 * encounter.battlefield.obstacles.len()
                                + encounter.battlefield.terrain.len()
                                + encounter.battlefield.lights.len(),
                        )?;
                        let value = area_reaches_point(encounter, &area, point).map_err(spatial)?;
                        reached.insert(point, value);
                        value
                    };
                    included |= reaches;
                }
            }
        }
        if !included {
            continue;
        }
        // The policy explicitly estimates a partial Total-cover shadow from the
        // entire occupied volume. Never reveal an unseen victim by requiring a
        // new victim-specific ruling. Authored grades and creature cover combine
        // by taking the greatest grade, rather than adding their bonuses.
        if blocked * 4 >= sample_count * 3 {
            cover = cover.max(CoverDegree::ThreeQuarters);
        } else if blocked * 2 >= sample_count {
            cover = cover.max(CoverDegree::Half);
        }
        targets.push(TacticalAreaTarget {
            actor,
            volume,
            cover,
            save: None,
            applied_by: None,
        });
    }
    Ok(BoundAreaGeometry {
        aim,
        policy,
        targets,
    })
}
