use super::*;

/// Square-grid distance (SRD p.13). Vertical grid steps use the same declared metric;
/// the separately authored line/volume geometry remains Euclidean.
pub fn grid_distance(a: SpatialPoint, b: SpatialPoint) -> Result<u32, SpatialError> {
    a.validate().map_err(invalid)?;
    b.validate().map_err(invalid)?;
    Ok(a.x
        .abs_diff(b.x)
        .max(a.y.abs_diff(b.y))
        .max(a.z.abs_diff(b.z)))
}

#[derive(Clone, Copy)]
struct Fraction {
    numerator: i64,
    denominator: i64,
}
impl Fraction {
    fn new(numerator: i64, denominator: i64) -> Self {
        if denominator < 0 {
            Self {
                numerator: -numerator,
                denominator: -denominator,
            }
        } else {
            Self {
                numerator,
                denominator,
            }
        }
    }
    fn compare(self, other: Self) -> std::cmp::Ordering {
        (i128::from(self.numerator) * i128::from(other.denominator))
            .cmp(&(i128::from(other.numerator) * i128::from(self.denominator)))
    }
}

/// True only when the segment passes through the volume's interior. Touching a face,
/// edge or corner is not passage through solid material. All products use i128.
pub fn segment_intersects(
    a: SpatialPoint,
    b: SpatialPoint,
    volume: SpatialBox,
) -> Result<bool, SpatialError> {
    Ok(segment_interval(a, b, volume)?.is_some())
}
fn segment_interval(
    a: SpatialPoint,
    b: SpatialPoint,
    volume: SpatialBox,
) -> Result<Option<(Fraction, Fraction)>, SpatialError> {
    a.validate().map_err(invalid)?;
    b.validate().map_err(invalid)?;
    volume.validate().map_err(invalid)?;
    let mut low = Fraction::new(0, 1);
    let mut high = Fraction::new(1, 1);
    for (start, end, min, max) in [
        (a.x, b.x, volume.min.x, volume.max.x),
        (a.y, b.y, volume.min.y, volume.max.y),
        (a.z, b.z, volume.min.z, volume.max.z),
    ] {
        let direction = i64::from(end) - i64::from(start);
        if direction == 0 {
            if start <= min || start >= max {
                return Ok(None);
            }
            continue;
        }
        let mut enter = Fraction::new(i64::from(min) - i64::from(start), direction);
        let mut leave = Fraction::new(i64::from(max) - i64::from(start), direction);
        if enter.compare(leave).is_gt() {
            std::mem::swap(&mut enter, &mut leave);
        }
        if enter.compare(low).is_gt() {
            low = enter;
        }
        if leave.compare(high).is_lt() {
            high = leave;
        }
        if !low.compare(high).is_lt() {
            return Ok(None);
        }
    }
    Ok(low.compare(high).is_lt().then_some((low, high)))
}

pub(super) fn samples(volume: SpatialBox) -> Vec<SpatialPoint> {
    let mut samples = Vec::with_capacity(9);
    // Inset from the face by half a foot to avoid treating a zero-width boundary as
    // exposure. Geometry is authored at this resolution; no percentage claim is made.
    for x in [volume.min.x, volume.max.x - 1] {
        for y in [volume.min.y, volume.max.y - 1] {
            for z in [volume.min.z, volume.max.z - 1] {
                samples.push(SpatialPoint { x, y, z });
            }
        }
    }
    samples.push(SpatialPoint {
        x: volume.min.x + (volume.max.x - volume.min.x) / 2,
        y: volume.min.y + (volume.max.y - volume.min.y) / 2,
        z: volume.min.z + (volume.max.z - volume.min.z) / 2,
    });
    samples
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoverAssessment {
    pub degree: CoverDegree,
    /// Partial/composite Total-Cover geometry has no SRD-specified raster percentage.
    /// The resolver must obtain a recorded geometry ruling before relying on its grade.
    pub requires_adjudication: bool,
}
impl CoverAssessment {
    pub fn armor_and_dexterity_bonus(self) -> Result<i32, SpatialError> {
        if self.requires_adjudication {
            return Err(illegal("partial cover needs an authored geometry ruling"));
        }
        match self.degree {
            CoverDegree::None => Ok(0),
            CoverDegree::Half => Ok(2),
            CoverDegree::ThreeQuarters => Ok(5),
            CoverDegree::Total => Err(illegal("total cover prevents direct targeting")),
        }
    }
}
pub fn cover_from(
    encounter: &TacticalEncounter,
    origin: SpatialPoint,
    target: SpatialBox,
    exclude: &[EntityId],
) -> Result<CoverAssessment, SpatialError> {
    encounter.validate_geometry().map_err(invalid)?;
    origin.validate().map_err(invalid)?;
    target.validate().map_err(invalid)?;
    cover_unchecked(encounter, origin, target, exclude)
}
pub(super) fn cover_unchecked(
    encounter: &TacticalEncounter,
    origin: SpatialPoint,
    target: SpatialBox,
    exclude: &[EntityId],
) -> Result<CoverAssessment, SpatialError> {
    let points = samples(target);
    let mut degree = CoverDegree::None;
    let mut partial_total = false;
    for obstacle in &encounter.battlefield.obstacles {
        let hits = points
            .iter()
            .map(|&p| segment_intersects(origin, p, obstacle.volume))
            .collect::<Result<Vec<_>, _>>()?;
        if obstacle.cover == CoverDegree::Total {
            // A single convex solid's shadow is convex. Blocking every extreme
            // target vertex proves the entire target is covered. The union of
            // several sampled shadows does not: a narrow opening may be missed.
            let mut entire_target = true;
            for x in [target.min.x, target.max.x] {
                for y in [target.min.y, target.max.y] {
                    for z in [target.min.z, target.max.z] {
                        entire_target &=
                            segment_intersects(origin, SpatialPoint { x, y, z }, obstacle.volume)?;
                    }
                }
            }
            if entire_target {
                return Ok(CoverAssessment {
                    degree: CoverDegree::Total,
                    requires_adjudication: false,
                });
            }
            partial_total |= hits.iter().any(|&x| x);
        } else if hits.iter().any(|&x| x) {
            degree = degree.max(obstacle.cover);
        }
    }
    for other in &encounter.participants {
        if !exclude.contains(&other.entity_id) {
            let volume = other.volume().map_err(invalid)?;
            for &point in &points {
                if segment_intersects(origin, point, volume)? {
                    degree = degree.max(CoverDegree::Half);
                    break;
                }
            }
        }
    }
    Ok(CoverAssessment {
        degree,
        requires_adjudication: partial_total,
    })
}

pub(super) fn clear_sight(
    field: &Battlefield,
    from: SpatialPoint,
    to: SpatialPoint,
) -> Result<bool, SpatialError> {
    clear_sight_with_truesight(field, from, to, 0)
}
pub(super) fn clear_sight_with_truesight(
    field: &Battlefield,
    from: SpatialPoint,
    to: SpatialPoint,
    truesight_range: u32,
) -> Result<bool, SpatialError> {
    for obstacle in &field.obstacles {
        if obstacle.blocks_sight
            && (obstacle.volume.contains(to) || segment_intersects(from, to, obstacle.volume)?)
        {
            return Ok(false);
        }
    }
    for terrain in &field.terrain {
        // Heavy represents a separate cause such as fog/foliage. Darkness-only
        // volumes use magical_darkness and need no duplicate Heavy marker.
        if terrain.obscuration == Obscuration::Heavy
            && (terrain.volume.contains(to) || segment_intersects(from, to, terrain.volume)?)
        {
            return Ok(false);
        }
        if terrain.magical_darkness
            && let Some((_, exit)) = segment_interval(from, to, terrain.volume)?
            && (truesight_range == 0
                || i128::from(grid_distance(from, to)?) * i128::from(exit.numerator)
                    > i128::from(truesight_range) * i128::from(exit.denominator))
        {
            // Truesight must cover the entire obscured portion, not necessarily
            // an otherwise illuminated target beyond its range (SRD122/190).
            return Ok(false);
        }
    }
    Ok(true)
}
pub(super) fn clear_effect(
    field: &Battlefield,
    from: SpatialPoint,
    to: SpatialPoint,
) -> Result<bool, SpatialError> {
    for obstacle in &field.obstacles {
        if obstacle.cover == CoverDegree::Total && segment_intersects(from, to, obstacle.volume)? {
            return Ok(false);
        }
    }
    Ok(true)
}
