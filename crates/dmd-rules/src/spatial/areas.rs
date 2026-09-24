use super::*;

/// Any nonzero bounded integer vector, including the delta between two valid points.
/// Squared integer predicates normalize directions without rounding floating-point
/// vectors. At these bounds, even squared cross-basis products remain within i128.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpatialDirection {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}
pub const MAX_SPATIAL_DIRECTION_COMPONENT: i32 = 2 * MAX_SPATIAL_COORDINATE;
impl SpatialDirection {
    fn validate(self) -> Result<(), SpatialError> {
        if [self.x, self.y, self.z].iter().all(|&n| n == 0)
            || [self.x, self.y, self.z].iter().any(|&n| {
                !(-MAX_SPATIAL_DIRECTION_COMPONENT..=MAX_SPATIAL_DIRECTION_COMPONENT).contains(&n)
            })
        {
            return Err(invalid("invalid spatial direction"));
        }
        Ok(())
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum SpatialArea {
    Cone {
        origin: SpatialPoint,
        direction: SpatialDirection,
        length: u32,
        include_origin: bool,
    },
    /// The origin may be anywhere on the selected face, including a corner.
    Cube {
        origin: SpatialPoint,
        face_center: SpatialPoint,
        direction: SpatialDirection,
        size: u32,
        include_origin: bool,
    },
    Cylinder {
        origin: SpatialPoint,
        radius: u32,
        height: u32,
        upward: bool,
    },
    Emanation {
        source: EntityId,
        radius: u32,
        include_origin: bool,
    },
    /// Full width in both perpendicular dimensions (authored grid prism convention).
    Line {
        origin: SpatialPoint,
        direction: SpatialDirection,
        length: u32,
        width: u32,
        include_origin: bool,
    },
    Sphere {
        origin: SpatialPoint,
        radius: u32,
    },
}

fn bounded_dimension(value: u32) -> Result<(), SpatialError> {
    if value == 0 || value > 4000 {
        return Err(invalid(
            "area dimension must be between half a foot and 2000 feet",
        ));
    }
    Ok(())
}
impl SpatialArea {
    pub fn validate(&self, encounter: &TacticalEncounter) -> Result<(), SpatialError> {
        match *self {
            Self::Cone {
                origin,
                direction,
                length,
                ..
            } => {
                origin.validate().map_err(invalid)?;
                direction.validate()?;
                bounded_dimension(length)?;
            }
            Self::Cube {
                origin,
                face_center,
                direction,
                size,
                ..
            } => {
                origin.validate().map_err(invalid)?;
                face_center.validate().map_err(invalid)?;
                direction.validate()?;
                bounded_dimension(size)?;
                if directional(origin, face_center, direction).0 != 0
                    || !in_cross_section(origin, face_center, direction, size)
                {
                    return Err(invalid("cube origin must lie on its selected face"));
                }
            }
            Self::Cylinder {
                origin,
                radius,
                height,
                ..
            } => {
                origin.validate().map_err(invalid)?;
                bounded_dimension(radius)?;
                bounded_dimension(height)?;
            }
            Self::Emanation { source, radius, .. } => {
                participant(encounter, source)?;
                bounded_dimension(radius)?;
            }
            Self::Line {
                origin,
                direction,
                length,
                width,
                ..
            } => {
                origin.validate().map_err(invalid)?;
                direction.validate()?;
                bounded_dimension(length)?;
                bounded_dimension(width)?;
            }
            Self::Sphere { origin, radius } => {
                origin.validate().map_err(invalid)?;
                bounded_dimension(radius)?;
            }
        }
        Ok(())
    }
}
fn differences(a: SpatialPoint, b: SpatialPoint) -> [i128; 3] {
    [
        i128::from(a.x) - i128::from(b.x),
        i128::from(a.y) - i128::from(b.y),
        i128::from(a.z) - i128::from(b.z),
    ]
}
fn squared(d: [i128; 3]) -> i128 {
    d.into_iter().map(|n| n * n).sum()
}
fn dot(a: [i128; 3], b: [i128; 3]) -> i128 {
    a.into_iter().zip(b).map(|(a, b)| a * b).sum()
}
fn in_cross_section(
    point: SpatialPoint,
    origin: SpatialPoint,
    direction: SpatialDirection,
    width: u32,
) -> bool {
    let axis = [
        i128::from(direction.x),
        i128::from(direction.y),
        i128::from(direction.z),
    ];
    let side = if direction.x == 0 && direction.y == 0 {
        [1, 0, 0]
    } else {
        [-axis[1], axis[0], 0]
    };
    let up = [
        axis[1] * side[2] - axis[2] * side[1],
        axis[2] * side[0] - axis[0] * side[2],
        axis[0] * side[1] - axis[1] * side[0],
    ];
    let delta = differences(point, origin);
    [side, up]
        .into_iter()
        .all(|basis| 4 * dot(delta, basis).pow(2) <= i128::from(width).pow(2) * squared(basis))
}
fn directional(
    point: SpatialPoint,
    origin: SpatialPoint,
    direction: SpatialDirection,
) -> (i128, i128, i128) {
    let delta = differences(point, origin);
    let axis = [
        i128::from(direction.x),
        i128::from(direction.y),
        i128::from(direction.z),
    ];
    (
        delta.into_iter().zip(axis).map(|(a, b)| a * b).sum(),
        squared(axis),
        squared(delta),
    )
}
fn clamp_to_volume(point: SpatialPoint, volume: SpatialBox) -> SpatialPoint {
    SpatialPoint {
        x: point.x.clamp(volume.min.x, volume.max.x),
        y: point.y.clamp(volume.min.y, volume.max.y),
        z: point.z.clamp(volume.min.z, volume.max.z),
    }
}

/// Exact continuous point membership for the declared geometry. Area rasterization is
/// a separate, explicit cell-center convention, never hidden inside this predicate.
pub fn area_contains_point(
    encounter: &TacticalEncounter,
    area: &SpatialArea,
    point: SpatialPoint,
) -> Result<bool, SpatialError> {
    encounter.validate_geometry().map_err(invalid)?;
    area.validate(encounter)?;
    point.validate().map_err(invalid)?;
    contains_unchecked(encounter, area, point)
}
fn contains_unchecked(
    encounter: &TacticalEncounter,
    area: &SpatialArea,
    point: SpatialPoint,
) -> Result<bool, SpatialError> {
    Ok(match *area {
        SpatialArea::Sphere { origin, radius } => {
            squared(differences(point, origin)) <= i128::from(radius).pow(2)
        }
        SpatialArea::Cylinder {
            origin,
            radius,
            height,
            upward,
        } => {
            let d = differences(point, origin);
            let vertical = if upward { d[2] } else { -d[2] };
            d[0] * d[0] + d[1] * d[1] <= i128::from(radius).pow(2)
                && (0..=i128::from(height)).contains(&vertical)
        }
        SpatialArea::Cone {
            origin,
            direction,
            length,
            include_origin,
        } => {
            if point == origin {
                include_origin
            } else {
                let (dot, norm, radius) = directional(point, origin, direction);
                dot > 0
                    && dot * dot <= i128::from(length).pow(2) * norm
                    && 4 * radius * norm <= 5 * dot * dot
            }
        }
        SpatialArea::Line {
            origin,
            direction,
            length,
            width,
            include_origin,
        } => {
            let (dot, norm, _) = directional(point, origin, direction);
            (point != origin || include_origin)
                && dot >= 0
                && dot * dot <= i128::from(length).pow(2) * norm
                && in_cross_section(point, origin, direction, width)
        }
        SpatialArea::Cube {
            origin,
            face_center,
            direction,
            size,
            include_origin,
        } => {
            if point == origin {
                include_origin
            } else {
                let (dot, norm, _) = directional(point, face_center, direction);
                dot >= 0
                    && dot * dot <= i128::from(size).pow(2) * norm
                    && in_cross_section(point, face_center, direction, size)
            }
        }
        SpatialArea::Emanation {
            source,
            radius,
            include_origin,
        } => {
            let volume = participant(encounter, source)?.volume().map_err(invalid)?;
            if volume.contains(point) {
                include_origin
            } else {
                squared(differences(point, clamp_to_volume(point, volume)))
                    <= i128::from(radius).pow(2)
            }
        }
    })
}
fn origin_for(
    encounter: &TacticalEncounter,
    area: &SpatialArea,
    point: SpatialPoint,
) -> Result<SpatialPoint, SpatialError> {
    Ok(match *area {
        SpatialArea::Cone { origin, .. }
        | SpatialArea::Cube { origin, .. }
        | SpatialArea::Cylinder { origin, .. }
        | SpatialArea::Line { origin, .. }
        | SpatialArea::Sphere { origin, .. } => origin,
        SpatialArea::Emanation { source, .. } => clamp_to_volume(
            point,
            participant(encounter, source)?.volume().map_err(invalid)?,
        ),
    })
}

/// SRD p.177: blocked straight lines exclude a point, even when it lies in the shape.
pub fn area_reaches_point(
    encounter: &TacticalEncounter,
    area: &SpatialArea,
    point: SpatialPoint,
) -> Result<bool, SpatialError> {
    if !area_contains_point(encounter, area, point)? {
        return Ok(false);
    }
    geometry::clear_effect(
        &encounter.battlefield,
        origin_for(encounter, area, point)?,
        point,
    )
}

/// Deterministic 5ft voxel-center rasterization; callers needing boundary-square
/// discretion must record a ruling rather than treating a preview as authority.
pub fn area_cells(
    encounter: &TacticalEncounter,
    area: &SpatialArea,
) -> Result<Vec<SpatialPoint>, SpatialError> {
    encounter.validate_geometry().map_err(invalid)?;
    area.validate(encounter)?;
    let bounds = encounter.battlefield.bounds;
    let count = i64::from((bounds.max.x - bounds.min.x) / GRID_SQUARE_UNITS)
        * i64::from((bounds.max.y - bounds.min.y) / GRID_SQUARE_UNITS)
        * ((i64::from(bounds.max.z) - i64::from(bounds.min.z) + 9) / 10);
    if count > MAX_BATTLEFIELD_GRID_CELLS {
        return Err(SpatialError::Capacity);
    }
    let mut result = Vec::new();
    for x in ((bounds.min.x + 5)..bounds.max.x).step_by(10) {
        for y in ((bounds.min.y + 5)..bounds.max.y).step_by(10) {
            for z in ((bounds.min.z + 5)..bounds.max.z).step_by(10) {
                let point = SpatialPoint { x, y, z };
                if contains_unchecked(encounter, area, point)?
                    && geometry::clear_effect(
                        &encounter.battlefield,
                        origin_for(encounter, area, point)?,
                        point,
                    )?
                {
                    result.push(point);
                }
            }
        }
    }
    Ok(result)
}
