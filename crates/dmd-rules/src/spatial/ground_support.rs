//! Positive dry support for a fixed-height translated ordinary body.
use super::*;

pub(super) fn ground_translation_supported(
    encounter: &TacticalEncounter,
    body: &TacticalParticipant,
    destination: SpatialPoint,
) -> Result<bool, SpatialError> {
    body.position.validate().map_err(invalid)?;
    destination.validate().map_err(invalid)?;
    if destination.z != body.position.z {
        return Ok(false);
    }
    let volume = body.volume().map_err(invalid)?;
    let footprint = body.size.footprint_units();
    // Test water before floor: submerged bodies also have fall_destination(None).
    if encounter.battlefield.terrain.iter().any(|terrain| {
        terrain.water
            && volume.min.z < terrain.volume.max.z
            && volume.max.z > terrain.volume.min.z
            && geometry::horizontal_sweep_contacts(
                body.position,
                destination,
                footprint,
                terrain.volume,
            )
    }) {
        return Ok(false);
    }
    if body.position.z == encounter.battlefield.floor_z {
        return Ok(true);
    }
    let surfaces = encounter
        .battlefield
        .obstacles
        .iter()
        .filter(|obstacle| obstacle.blocks_movement && obstacle.volume.max.z == body.position.z)
        .map(|obstacle| obstacle.volume)
        .chain(
            encounter
                .battlefield
                .terrain
                .iter()
                .filter(|terrain| {
                    !terrain.water
                        && (terrain.supports_top || terrain.burrowable)
                        && terrain.volume.max.z == body.position.z
                })
                .map(|terrain| terrain.volume),
        )
        .collect::<Vec<_>>();
    Ok(geometry::continuous_horizontal_support(
        body.position,
        destination,
        footprint,
        &surfaces,
    ))
}
