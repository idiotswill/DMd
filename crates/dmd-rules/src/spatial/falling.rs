use super::*;

fn horizontal_contact(body: SpatialBox, surface: SpatialBox) -> bool {
    body.min.x < surface.max.x
        && body.max.x > surface.min.x
        && body.min.y < surface.max.y
        && body.max.y > surface.min.y
}

/// Authored Burrow support means the entire foot space is within real penetrable
/// ground at its current depth. Adjacent ground volumes may jointly cover it;
/// merely clipping a wall's side cannot hold an otherwise airborne creature.
pub(crate) fn buried_support(
    encounter: &TacticalEncounter,
    actor: &TacticalParticipant,
) -> Result<bool, SpatialError> {
    if actor.movement.burrow.is_none() {
        return Ok(false);
    }
    let body = actor.volume().map_err(invalid)?;
    let ground = encounter
        .battlefield
        .terrain
        .iter()
        .filter(|t| {
            t.burrowable
                && t.volume.min.z <= body.min.z
                && body.min.z < t.volume.max.z
                && horizontal_contact(body, t.volume)
        })
        .map(|t| t.volume)
        .collect::<Vec<_>>();
    if ground.is_empty() {
        return Ok(false);
    }
    let mut xs = vec![body.min.x, body.max.x];
    for part in &ground {
        xs.push(part.min.x.clamp(body.min.x, body.max.x));
        xs.push(part.max.x.clamp(body.min.x, body.max.x));
    }
    xs.sort_unstable();
    xs.dedup();
    for slab in xs.windows(2) {
        let mut ys = ground
            .iter()
            .filter(|p| p.min.x <= slab[0] && p.max.x >= slab[1])
            .map(|p| (p.min.y.max(body.min.y), p.max.y.min(body.max.y)))
            .collect::<Vec<_>>();
        ys.sort_unstable();
        let mut covered = body.min.y;
        for (from, to) in ys {
            if from > covered {
                break;
            }
            covered = covered.max(to);
        }
        if covered < body.max.y {
            return Ok(false);
        }
    }
    Ok(true)
}

pub(super) fn validate_physical_position(
    encounter: &TacticalEncounter,
    actor: &TacticalParticipant,
) -> Result<(), SpatialError> {
    let body = actor.volume().map_err(invalid)?;
    if encounter
        .battlefield
        .obstacles
        .iter()
        .any(|o| o.blocks_movement && body.intersects(o.volume))
    {
        return Err(invalid(
            "creature is embedded in a movement-blocking obstacle",
        ));
    }
    let in_ground = encounter
        .battlefield
        .terrain
        .iter()
        .any(|t| t.burrowable && body.intersects(t.volume));
    if (in_ground || actor.position.z < encounter.battlefield.floor_z)
        && !buried_support(encounter, actor)?
    {
        return Err(invalid("underground creature lacks source Burrow support"));
    }
    Ok(())
}

/// Reject impossible initial positions before an accepted action can produce a drop
/// or fall. Every admitted map has an authored floor; source Burrow positions remain
/// legal. Nonblocking terrain and creature overlap are intentionally unaffected.
pub fn validate_physical_positions(encounter: &TacticalEncounter) -> Result<(), SpatialError> {
    encounter.validate_geometry().map_err(invalid)?;
    for actor in &encounter.participants {
        validate_physical_position(encounter, actor)?;
    }
    Ok(())
}

fn surface_order(surface: &FallSurface) -> (u8, &str) {
    match surface {
        FallSurface::SolidObstacle { id } => (0, id),
        FallSurface::SupportingTerrain { id } => (1, id),
        FallSurface::Floor => (2, ""),
        FallSurface::Liquid { id } => (3, id),
    }
}

/// First vertical contact with the authored map. This is an explicit geometry
/// convention: any positive footprint overlap catches a falling body on a solid
/// top, including a narrow ledge. Mere edge contact does not. It is not an SRD
/// collision algorithm. Source damage is resolved separately.
///
/// Other creatures do not become invented platforms or receive optional collision
/// damage. An involuntary landing may share their space. The caller must not expose
/// this truth-only query as a player's preview of hidden surfaces.
pub fn fall_destination(
    encounter: &TacticalEncounter,
    actor: EntityId,
) -> Result<Option<SpatialFall>, SpatialError> {
    encounter.validate_geometry().map_err(invalid)?;
    let actor = participant(encounter, actor)?;
    let body = actor.volume().map_err(invalid)?;
    let field = &encounter.battlefield;
    if actor.position.z < field.floor_z {
        return Err(invalid("fall origin is below the authored floor"));
    }
    if field
        .obstacles
        .iter()
        .any(|o| o.blocks_movement && body.intersects(o.volume))
        || field
            .terrain
            .iter()
            .any(|t| t.burrowable && body.intersects(t.volume))
    {
        return Err(invalid("fall origin intersects solid authored geometry"));
    }
    // Already in liquid is swimming/submersion, not another air-to-liquid impact.
    if field.terrain.iter().any(|t| {
        t.water
            && horizontal_contact(body, t.volume)
            && actor.position.z >= t.volume.min.z
            && actor.position.z < t.volume.max.z
    }) {
        return Ok(None);
    }
    let mut surfaces = vec![(field.floor_z, FallSurface::Floor)];
    surfaces.extend(
        field
            .obstacles
            .iter()
            .filter(|o| {
                o.blocks_movement
                    && o.volume.max.z <= actor.position.z
                    && horizontal_contact(body, o.volume)
            })
            .map(|o| {
                (
                    o.volume.max.z,
                    FallSurface::SolidObstacle { id: o.id.clone() },
                )
            }),
    );
    surfaces.extend(field.terrain.iter().filter_map(|t| {
        if t.volume.max.z > actor.position.z || !horizontal_contact(body, t.volume) {
            return None;
        }
        // Obscuration, difficult terrain and a climbable region do not themselves
        // create a load-bearing top. Burrowable authored ground is physically solid.
        let surface = if t.supports_top || t.burrowable {
            FallSurface::SupportingTerrain { id: t.id.clone() }
        } else if t.water {
            FallSurface::Liquid { id: t.id.clone() }
        } else {
            return None;
        };
        Some((t.volume.max.z, surface))
    }));
    surfaces.sort_by(|a, b| {
        b.0.cmp(&a.0)
            .then_with(|| surface_order(&a.1).cmp(&surface_order(&b.1)))
    });
    let (z, surface) = surfaces
        .into_iter()
        .next()
        .ok_or_else(|| invalid("authored fall has no landing"))?;
    if z == actor.position.z {
        return Ok(None);
    }
    let to = SpatialPoint {
        z,
        ..actor.position
    };
    Ok(Some(SpatialFall {
        from: actor.position,
        to,
        surface,
    }))
}

/// SRD Flying/Fly Speed: Hover preserves flight through incapacitation, Prone and
/// zero Fly Speed, but not death. Grounded and submerged creatures do not fall again.
/// This checks actual mechanics, never a caller-supplied permission or condition list.
pub fn flight_loss_fall(
    encounter: &TacticalEncounter,
    state: &CampaignState,
    actor: EntityId,
) -> Result<Option<SpatialFall>, SpatialError> {
    validate_encounter(encounter, state)?;
    let participant = participant(encounter, actor)?;
    // A creature with a Fly Speed may currently be burrowing through actual ground.
    // It is not airborne merely because its source profile also permits flight.
    validate_physical_position(encounter, participant)?;
    if participant.position.z == encounter.battlefield.floor_z
        || buried_support(encounter, participant)?
    {
        return Ok(None);
    }
    let Some(base) = participant.movement.fly else {
        return Ok(None);
    };
    let rules = state
        .rules
        .as_ref()
        .ok_or_else(|| invalid("mechanics absent"))?;
    let speed = crate::tactical_conditions::effective_speed(rules, actor, base)
        .map_err(|e| invalid(e.to_string()))?;
    let conditions = conditions(state, actor);
    let lost = dead(state, actor)
        || (!participant.movement.hover
            && (speed == 0
                || conditions.contains(&Condition::Incapacitated)
                || conditions.contains(&Condition::Prone)));
    if lost {
        fall_destination(encounter, actor)
    } else {
        Ok(None)
    }
}

/// Dropped held objects settle independently of a creature's personal Fly/Hover.
/// The holder footprint uses the same explicit first-contact map adjudication as
/// creature landing; object size, breakage and collision damage are not invented.
pub fn drop_destination(
    encounter: &TacticalEncounter,
    actor: EntityId,
) -> Result<SpatialPoint, SpatialError> {
    encounter.validate_geometry().map_err(invalid)?;
    let holder = participant(encounter, actor)?;
    validate_physical_position(encounter, holder)?;
    if buried_support(encounter, holder)? {
        // Real surrounding ground supports a drop at its actual buried location.
        return Ok(holder.position);
    }
    Ok(fall_destination(encounter, actor)?.map_or(holder.position, |fall| fall.to))
}
