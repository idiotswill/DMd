use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MovementStep {
    pub destination: SpatialPoint,
    pub mode: MovementMode,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpatialPath {
    pub steps: Vec<MovementStep>,
}
/// Counts of accepted Dash grants for each selected speed (SRD180; ADR026).
/// Every mode still subtracts the same movement already spent (SRD188).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DashGrants {
    pub speed: u8,
    pub climb: u8,
    pub swim: u8,
    pub fly: u8,
    pub burrow: u8,
}
impl DashGrants {
    fn total(&self) -> u16 {
        [self.speed, self.climb, self.swim, self.fly, self.burrow]
            .into_iter()
            .map(u16::from)
            .sum()
    }
    fn for_mode(&self, profile: &MovementProfile, mode: MovementMode) -> u8 {
        match mode {
            MovementMode::Walk | MovementMode::Crawl | MovementMode::Jump => self.speed,
            MovementMode::Climb if profile.climb.is_some() => self.climb,
            MovementMode::Swim if profile.swim.is_some() => self.swim,
            MovementMode::Climb | MovementMode::Swim => self.speed,
            MovementMode::Fly => self.fly,
            MovementMode::Burrow => self.burrow,
            MovementMode::Teleport => 0,
        }
    }
}
/// Query input derived by the encounter resolver from its single turn/feature state.
/// This DTO does not grant authority and must never be accepted from a player as truth.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MovementAllowance {
    pub spent: u32,
    pub dash: DashGrants,
    pub forced: bool,
    pub disengaged: bool,
    pub teleport_range: Option<u32>,
    pub runup: u32,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpportunityCrossing {
    pub actor: EntityId,
    pub before_leaving: SpatialPoint,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MovementSegment {
    pub from: SpatialPoint,
    pub to: SpatialPoint,
    pub mode: MovementMode,
    pub cost: u32,
    pub opportunities: Vec<OpportunityCrossing>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MovementPlan {
    pub segments: Vec<MovementSegment>,
    pub total_cost: u32,
    pub destination: SpatialPoint,
    pub falls_at_end: bool,
}

fn interval_distance(min_a: i32, max_a: i32, min_b: i32, max_b: i32) -> u32 {
    if max_a < min_b {
        min_b.abs_diff(max_a)
    } else if max_b < min_a {
        min_a.abs_diff(max_b)
    } else {
        0
    }
}
/// Minimum distance between occupied grid-cell centers; large footprints do not use
/// their token center for reach. Height contributes through occupied vertical cells.
pub fn participant_distance(
    a: &TacticalParticipant,
    b: &TacticalParticipant,
) -> Result<u32, SpatialError> {
    let a = a.volume().map_err(invalid)?;
    let b = b.volume().map_err(invalid)?;
    a.validate().map_err(invalid)?;
    b.validate().map_err(invalid)?;
    let axes = [
        (a.min.x, a.max.x, b.min.x, b.max.x),
        (a.min.y, a.max.y, b.min.y, b.max.y),
        (a.min.z, a.max.z, b.min.z, b.max.z),
    ];
    Ok(axes
        .into_iter()
        .map(|(amin, amax, bmin, bmax)| {
            let ainset = ((amax - amin).min(GRID_SQUARE_UNITS)) / 2;
            let binset = ((bmax - bmin).min(GRID_SQUARE_UNITS)) / 2;
            interval_distance(amin + ainset, amax - ainset, bmin + binset, bmax - binset)
        })
        .max()
        .unwrap_or(0))
}
fn speed(
    actor: &TacticalParticipant,
    state: &CampaignState,
    mode: MovementMode,
) -> Result<u32, SpatialError> {
    let conditions = conditions(state, actor.entity_id);
    if [
        Condition::Grappled,
        Condition::Restrained,
        Condition::Paralyzed,
        Condition::Petrified,
        Condition::Unconscious,
    ]
    .iter()
    .any(|c| conditions.contains(c))
    {
        return Ok(0);
    }
    let p = &actor.movement;
    let base = match mode {
        MovementMode::Walk | MovementMode::Crawl | MovementMode::Jump => p.walk,
        MovementMode::Climb => p.climb.unwrap_or(p.walk),
        MovementMode::Swim => p.swim.unwrap_or(p.walk),
        MovementMode::Fly => p.fly.ok_or_else(|| illegal("creature has no Fly Speed"))?,
        MovementMode::Burrow => p
            .burrow
            .ok_or_else(|| illegal("creature has no Burrow Speed"))?,
        MovementMode::Teleport => 0,
    };
    if mode == MovementMode::Fly
        && !p.hover
        && (conditions.contains(&Condition::Incapacitated)
            || conditions.contains(&Condition::Prone))
    {
        return Err(illegal(
            "unsupported flight cannot continue while incapacitated or prone",
        ));
    }
    let exhaustion = state
        .rules
        .as_ref()
        .and_then(|r| r.entities.get(&actor.entity_id))
        .map_or(0, |e| e.exhaustion);
    Ok(base.saturating_sub(u32::from(exhaustion) * 10))
}
fn region_at(
    encounter: &TacticalEncounter,
    volume: SpatialBox,
    test: impl Fn(&TerrainVolume) -> bool,
) -> bool {
    encounter
        .battlefield
        .terrain
        .iter()
        .any(|t| test(t) && t.volume.intersects(volume))
}
fn supported(encounter: &TacticalEncounter, actor: &TacticalParticipant) -> bool {
    actor.position.z == encounter.battlefield.floor_z
        || encounter.battlefield.terrain.iter().any(|t| {
            t.supports_top
                && actor.position.z == t.volume.max.z
                && actor.position.x >= t.volume.min.x
                && actor.position.x + actor.size.footprint_units() <= t.volume.max.x
                && actor.position.y >= t.volume.min.y
                && actor.position.y + actor.size.footprint_units() <= t.volume.max.y
        })
}
fn occupancy(
    encounter: &TacticalEncounter,
    state: &CampaignState,
    actor: &TacticalParticipant,
    last: bool,
) -> Result<bool, SpatialError> {
    let volume = actor.volume().map_err(invalid)?;
    let mut difficult = false;
    for other in &encounter.participants {
        if other.entity_id == actor.entity_id
            || !volume.intersects(other.volume().map_err(invalid)?)
        {
            continue;
        }
        if last {
            return Err(illegal(
                "cannot willingly end movement in another creature's space",
            ));
        }
        let ally = actor.allies.contains(&other.entity_id);
        if !ally
            && other.size != CreatureSize::Tiny
            && (actor.size.rank() - other.size.rank()).abs() < 2
            && !conditions(state, other.entity_id).contains(&Condition::Incapacitated)
        {
            return Err(illegal("creature blocks passage"));
        }
        difficult |= !ally && other.size != CreatureSize::Tiny;
    }
    Ok(difficult)
}
fn physical_blocked(encounter: &TacticalEncounter, volume: SpatialBox, mode: MovementMode) -> bool {
    encounter
        .battlefield
        .obstacles
        .iter()
        .any(|o| o.blocks_movement && o.volume.intersects(volume))
        || (mode != MovementMode::Burrow
            && encounter
                .battlefield
                .terrain
                .iter()
                .any(|t| t.burrowable && t.volume.intersects(volume)))
}
fn swept_blocked(
    encounter: &TacticalEncounter,
    from: &TacticalParticipant,
    to: &TacticalParticipant,
    mode: MovementMode,
) -> Result<bool, SpatialError> {
    // Expand each solid by the moving footprint. A line through that Minkowski box
    // detects thin barriers between endpoints, independent of waypoint spacing.
    let footprint = from.size.footprint_units();
    let height = i32::try_from(from.height).map_err(|_| invalid("height overflow"))?;
    for volume in encounter
        .battlefield
        .obstacles
        .iter()
        .filter(|o| o.blocks_movement)
        .map(|o| o.volume)
        .chain(
            encounter
                .battlefield
                .terrain
                .iter()
                .filter(|t| t.burrowable && mode != MovementMode::Burrow)
                .map(|t| t.volume),
        )
    {
        let expanded = SpatialBox {
            min: SpatialPoint {
                x: volume.min.x - footprint,
                y: volume.min.y - footprint,
                z: volume.min.z - height,
            },
            max: volume.max,
        };
        // Expanded query bounds may extend beyond authored coordinate limits; clamp to
        // the representable world, never wrap integer subtraction.
        let expanded = SpatialBox {
            min: SpatialPoint {
                x: expanded.min.x.max(-MAX_SPATIAL_COORDINATE),
                y: expanded.min.y.max(-MAX_SPATIAL_COORDINATE),
                z: expanded.min.z.max(-MAX_SPATIAL_COORDINATE),
            },
            ..expanded
        };
        if geometry::segment_intersects(from.position, to.position, expanded)? {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Query a proposed path without spending movement. The runtime must resolve each
/// segment's reaction window before committing subsequent segments and re-evaluate
/// the remainder if the reaction changes truth. Never commit this whole plan blindly.
pub fn evaluate_path(
    encounter: &TacticalEncounter,
    state: &CampaignState,
    actor_id: EntityId,
    path: &SpatialPath,
    allowance: &MovementAllowance,
) -> Result<MovementPlan, SpatialError> {
    validate_encounter(encounter, state)?;
    if path.steps.is_empty()
        || path.steps.len() > 1024
        || allowance.dash.total() > 4
        || allowance.spent > 10_000
        || allowance.runup > 10_000
        || allowance.teleport_range.is_some_and(|r| r == 0 || r > 4000)
    {
        return Err(invalid("invalid bounded movement query"));
    }
    let mut working = encounter.clone();
    let index = working
        .participants
        .iter()
        .position(|p| p.entity_id == actor_id)
        .ok_or(SpatialError::UnknownActor)?;
    let mut moving = working.participants[index].clone();
    if !allowance.forced && dead(state, actor_id) {
        return Err(illegal("dead creatures cannot move voluntarily"));
    }
    let mut segments = Vec::new();
    let mut cost = 0u32;
    let mut runup = allowance.runup;
    let mut jump_start = None;
    let strength = state
        .rules
        .as_ref()
        .and_then(|r| r.entities.get(&actor_id))
        .map(|e| e.ability_scores[0])
        .ok_or(SpatialError::UnknownActor)?;
    for (step_index, step) in path.steps.iter().enumerate() {
        step.destination.validate().map_err(invalid)?;
        let alignment = if moving.size == CreatureSize::Tiny {
            5
        } else {
            10
        };
        if step.destination.x.rem_euclid(alignment) != 0
            || step.destination.y.rem_euclid(alignment) != 0
        {
            return Err(illegal("destination is not aligned to the creature's grid"));
        }
        let distance = grid_distance(moving.position, step.destination)?;
        if distance == 0 {
            return Err(illegal("zero-distance movement step"));
        }
        let teleport = step.mode == MovementMode::Teleport;
        if teleport {
            if !allowance
                .teleport_range
                .is_some_and(|range| distance <= range)
            {
                return Err(illegal("teleport needs an applicable source feature/range"));
            }
        } else if distance > 10 {
            return Err(illegal(
                "ordinary path steps must be adjacent grid positions",
            ));
        }
        let mut next = moving.clone();
        next.position = step.destination;
        let volume = next.volume().map_err(invalid)?;
        if !encounter.battlefield.bounds.encloses(volume)
            || physical_blocked(encounter, volume, step.mode)
        {
            return Err(illegal(
                "destination is outside the battlefield or inside solid terrain",
            ));
        }
        if !teleport && swept_blocked(encounter, &moving, &next, step.mode)? {
            return Err(illegal("movement crosses solid terrain"));
        }
        if !teleport && moving.position.x != next.position.x && moving.position.y != next.position.y
        {
            for corner in [
                SpatialPoint {
                    x: moving.position.x,
                    y: next.position.y,
                    z: next.position.z,
                },
                SpatialPoint {
                    x: next.position.x,
                    y: moving.position.y,
                    z: next.position.z,
                },
            ] {
                let mut corner_actor = moving.clone();
                corner_actor.position = corner;
                if physical_blocked(
                    encounter,
                    corner_actor.volume().map_err(invalid)?,
                    step.mode,
                ) {
                    return Err(illegal("diagonal movement cannot cross a solid corner"));
                }
            }
        }
        let last = step_index + 1 == path.steps.len();
        let occupied_difficult = if allowance.forced {
            false
        } else {
            occupancy(&working, state, &next, last)?
        };
        if !allowance.forced && !teleport {
            if conditions(state, actor_id).contains(&Condition::Prone)
                && step.mode != MovementMode::Crawl
            {
                return Err(illegal("prone movement must crawl"));
            }
            match step.mode {
                MovementMode::Walk | MovementMode::Crawl if !supported(encounter, &next) => {
                    return Err(illegal(
                        "walking/crawling destination has no supporting surface",
                    ));
                }
                MovementMode::Swim if !region_at(encounter, volume, |t| t.water) => {
                    return Err(illegal("swimming requires water"));
                }
                MovementMode::Climb if !region_at(encounter, volume, |t| t.climbable) => {
                    return Err(illegal("climbing requires authored climbable terrain"));
                }
                MovementMode::Burrow if !region_at(encounter, volume, |t| t.burrowable) => {
                    return Err(illegal("burrowing requires penetrable terrain"));
                }
                _ => {}
            }
            if step.mode == MovementMode::Jump {
                let (start, had_runup) = *jump_start.get_or_insert((moving.position, runup >= 20));
                let horizontal = start
                    .x
                    .abs_diff(next.position.x)
                    .max(start.y.abs_diff(next.position.y));
                let horizontal_limit = u32::from(strength) * if had_runup { 2 } else { 1 };
                let vertical_limit = u32::try_from((3 + crate::ability_modifier(strength)).max(0))
                    .map_err(|_| invalid("jump arithmetic"))?
                    * if had_runup { 2 } else { 1 };
                if horizontal > horizontal_limit
                    || next.position.z.saturating_sub(start.z)
                        > i32::try_from(vertical_limit).map_err(|_| invalid("jump arithmetic"))?
                {
                    return Err(illegal("jump exceeds Strength-derived distance or height"));
                }
            } else {
                jump_start = None;
            }
            for effect in state
                .rules
                .as_ref()
                .into_iter()
                .flat_map(|r| &r.effects)
                .filter(|e| e.target == actor_id && e.condition == Some(Condition::Frightened))
            {
                let fear = participant(encounter, effect.source)?;
                if participant_distance(&next, fear)? < participant_distance(&moving, fear)? {
                    return Err(illegal("frightened movement cannot approach its source"));
                }
            }
        }
        let difficult = occupied_difficult || region_at(encounter, volume, |t| t.difficult);
        let extra = match step.mode {
            MovementMode::Crawl => 1,
            MovementMode::Climb if moving.movement.climb.is_none() => 1,
            MovementMode::Swim if moving.movement.swim.is_none() => 1,
            _ => 0,
        };
        let step_cost = if teleport || allowance.forced {
            0
        } else {
            distance
                .checked_mul(1 + extra + u32::from(difficult))
                .ok_or_else(|| invalid("movement cost overflow"))?
        };
        cost = cost
            .checked_add(step_cost)
            .ok_or_else(|| invalid("movement cost overflow"))?;
        if !teleport && !allowance.forced {
            let maximum = speed(&moving, state, step.mode)?
                .checked_mul(1 + u32::from(allowance.dash.for_mode(&moving.movement, step.mode)))
                .ok_or_else(|| invalid("movement budget overflow"))?;
            if allowance
                .spent
                .checked_add(cost)
                .ok_or_else(|| invalid("movement budget overflow"))?
                > maximum
            {
                return Err(illegal(
                    "path exceeds remaining movement for its selected speed",
                ));
            }
        }
        let mut opportunities = Vec::new();
        if !teleport && !allowance.forced && !allowance.disengaged {
            for enemy in &working.participants {
                if enemy.enemies.contains(&actor_id)
                    && participant_distance(enemy, &moving)? <= enemy.reach
                    && participant_distance(enemy, &next)? > enemy.reach
                    && !conditions(state, enemy.entity_id).contains(&Condition::Incapacitated)
                    && !state
                        .rules
                        .as_ref()
                        .and_then(|r| r.timing.as_ref())
                        .is_some_and(|t| t.reactions_spent.contains(&enemy.entity_id))
                    && perception::perceive_unchecked(&working, state, enemy.entity_id, actor_id)?
                        .sees
                {
                    opportunities.push(OpportunityCrossing {
                        actor: enemy.entity_id,
                        before_leaving: moving.position,
                    });
                }
            }
        }
        opportunities.sort_by_key(|o| o.actor.0);
        segments.push(MovementSegment {
            from: moving.position,
            to: next.position,
            mode: step.mode,
            cost: step_cost,
            opportunities,
        });
        if step.mode == MovementMode::Walk && moving.position.z == next.position.z {
            runup = runup.saturating_add(distance);
        } else if step.mode != MovementMode::Jump {
            runup = 0;
        }
        moving = next;
        working.participants[index] = moving.clone();
    }
    let final_mode = path.steps.last().ok_or_else(|| invalid("empty path"))?.mode;
    let falls_at_end = !supported(encounter, &moving)
        && final_mode != MovementMode::Fly
        && !region_at(encounter, moving.volume().map_err(invalid)?, |t| {
            t.water || t.climbable || t.burrowable
        });
    Ok(MovementPlan {
        segments,
        total_cost: cost,
        destination: moving.position,
        falls_at_end,
    })
}
