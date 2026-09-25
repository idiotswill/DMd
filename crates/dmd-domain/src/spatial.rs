//! Authoritative tactical truth. Coordinates and lengths use half-foot units, not pixels.
//! Timing, HP, conditions and action budgets remain in `RulesState`.
use crate::{
    AgentRef, CampaignState, CommandIssuer, CommandMeta, EncounterId, EntityId, PresenceRole,
    Ruling, SceneId,
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

pub const SPATIAL_UNITS_PER_FOOT: u32 = 2;
pub const GRID_SQUARE_UNITS: i32 = 10;
pub const MAX_SPATIAL_COORDINATE: i32 = 200_000;
pub const MAX_BATTLEFIELD_GRID_CELLS: i64 = 32_768;
pub const MAX_TACTICAL_PARTICIPANTS: usize = 128;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpatialPoint {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpatialBox {
    pub min: SpatialPoint,
    pub max: SpatialPoint,
}
impl SpatialBox {
    /// Half-open volume; a creature standing on a solid top is not inside the solid.
    pub fn contains(self, point: SpatialPoint) -> bool {
        point.x >= self.min.x
            && point.x < self.max.x
            && point.y >= self.min.y
            && point.y < self.max.y
            && point.z >= self.min.z
            && point.z < self.max.z
    }
    pub fn intersects(self, other: Self) -> bool {
        self.min.x < other.max.x
            && self.max.x > other.min.x
            && self.min.y < other.max.y
            && self.max.y > other.min.y
            && self.min.z < other.max.z
            && self.max.z > other.min.z
    }
    pub fn encloses(self, other: Self) -> bool {
        other.min.x >= self.min.x
            && other.max.x <= self.max.x
            && other.min.y >= self.min.y
            && other.max.y <= self.max.y
            && other.min.z >= self.min.z
            && other.max.z <= self.max.z
    }
    pub fn validate(self) -> Result<(), String> {
        for point in [self.min, self.max] {
            point.validate()?;
        }
        if self.min.x >= self.max.x || self.min.y >= self.max.y || self.min.z >= self.max.z {
            return Err("spatial box must have positive dimensions".into());
        }
        Ok(())
    }
}
impl SpatialPoint {
    pub fn validate(self) -> Result<(), String> {
        if [self.x, self.y, self.z]
            .into_iter()
            .any(|n| !(-MAX_SPATIAL_COORDINATE..=MAX_SPATIAL_COORDINATE).contains(&n))
        {
            return Err("spatial coordinate exceeds supported bounds".into());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CreatureSize {
    Tiny,
    Small,
    Medium,
    Large,
    Huge,
    Gargantuan,
}
impl CreatureSize {
    pub const fn footprint_units(self) -> i32 {
        match self {
            Self::Tiny => 5,
            Self::Small | Self::Medium => 10,
            Self::Large => 20,
            Self::Huge => 30,
            Self::Gargantuan => 40,
        }
    }
    pub const fn rank(self) -> i32 {
        match self {
            Self::Tiny => 0,
            Self::Small => 1,
            Self::Medium => 2,
            Self::Large => 3,
            Self::Huge => 4,
            Self::Gargantuan => 5,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MovementMode {
    Walk,
    Climb,
    Swim,
    Fly,
    Burrow,
    Crawl,
    Jump,
    Teleport,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MovementProfile {
    pub walk: u32,
    pub climb: Option<u32>,
    pub swim: Option<u32>,
    pub fly: Option<u32>,
    pub burrow: Option<u32>,
    pub hover: bool,
}
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Senses {
    pub darkvision: u32,
    pub blindsight: u32,
    pub tremorsense: u32,
    pub truesight: u32,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CoverDegree {
    None,
    Half,
    ThreeQuarters,
    Total,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LightLevel {
    Bright,
    Dim,
    Darkness,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Obscuration {
    None,
    Light,
    Heavy,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TerrainVolume {
    pub id: String,
    pub volume: SpatialBox,
    pub difficult: bool,
    /// Whether its terrain properties are apparent by sight; hidden hazards need discovery.
    pub observable: bool,
    pub water: bool,
    pub climbable: bool,
    pub burrowable: bool,
    pub supports_top: bool,
    /// Connected material/liquid identity for Tremorsense; not a faction/knowledge channel.
    pub surface: Option<String>,
    /// Non-darkness obscuration such as fog or foliage; Truesight does not bypass it.
    pub obscuration: Obscuration,
    /// Independently blocks sight through this volume unless Truesight reaches
    /// its far edge. A darkness-only volume uses obscuration: None.
    pub magical_darkness: bool,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpatialObstacle {
    pub id: String,
    pub volume: SpatialBox,
    pub blocks_movement: bool,
    pub blocks_sight: bool,
    /// Physical truth does not by itself make an invisible/concealed obstacle known.
    pub observable: bool,
    /// Authored cover geometry is a GM adjudication, not an invented SRD ray-count rule.
    pub cover: CoverDegree,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpatialLight {
    pub id: String,
    pub position: SpatialPoint,
    pub bright_radius: u32,
    pub dim_radius: u32,
    /// A moving emitter follows this participant's center; position is the stationary fallback only.
    pub attached_to: Option<EntityId>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Battlefield {
    pub bounds: SpatialBox,
    pub floor_z: i32,
    pub floor_surface: String,
    pub ambient_light: LightLevel,
    pub terrain: Vec<TerrainVolume>,
    pub obstacles: Vec<SpatialObstacle>,
    pub lights: Vec<SpatialLight>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TacticalParticipant {
    pub entity_id: EntityId,
    pub position: SpatialPoint,
    pub size: CreatureSize,
    /// What a perceiving creature sees, not the world record's possibly secret true name.
    pub public_label: String,
    /// Authored body height in half-feet. Footprint comes from size, not a UI token image.
    pub height: u32,
    pub reach: u32,
    pub movement: MovementProfile,
    pub senses: Senses,
    pub allies: Vec<EntityId>,
    pub enemies: Vec<EntityId>,
}
impl TacticalParticipant {
    pub fn volume(&self) -> Result<SpatialBox, String> {
        let size = self.size.footprint_units();
        Ok(SpatialBox {
            min: self.position,
            max: SpatialPoint {
                x: self
                    .position
                    .x
                    .checked_add(size)
                    .ok_or("participant x overflow")?,
                y: self
                    .position
                    .y
                    .checked_add(size)
                    .ok_or("participant y overflow")?,
                z: self
                    .position
                    .z
                    .checked_add(
                        i32::try_from(self.height).map_err(|_| "participant height overflow")?,
                    )
                    .ok_or("participant z overflow")?,
            },
        })
    }
    pub fn center(&self) -> Result<SpatialPoint, String> {
        let volume = self.volume()?;
        Ok(SpatialPoint {
            x: volume.min.x + (volume.max.x - volume.min.x) / 2,
            y: volume.min.y + (volume.max.y - volume.min.y) / 2,
            z: volume.min.z + (volume.max.z - volume.min.z) / 2,
        })
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PerceptionModality {
    Sight,
    Blindsight,
    Tremorsense,
    Hearing,
    Communicated,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RememberedContact {
    pub target: EntityId,
    pub position: SpatialPoint,
    pub modality: PerceptionModality,
    pub origin: CommandMeta,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RememberedCell {
    pub position: SpatialPoint,
    pub difficult: bool,
    pub blocked: bool,
    pub origin: CommandMeta,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActorKnowledge {
    pub observer: EntityId,
    pub contacts: Vec<RememberedContact>,
    pub terrain: Vec<RememberedCell>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TacticalEncounter {
    pub id: EncounterId,
    pub scene_id: SceneId,
    pub battlefield: Battlefield,
    pub participants: Vec<TacticalParticipant>,
    pub knowledge: Vec<ActorKnowledge>,
    pub origin: CommandMeta,
    pub geometry_ruling: Ruling,
    /// Optional host map adjudication, authenticated by this setup's origin.
    /// Historical maps without it must choose a policy before area activation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub area_grid_policy: Option<crate::TacticalAreaGridPolicy>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flow: Option<crate::TacticalFlow>,
}

fn identifier(value: &str) -> Result<(), String> {
    if value.is_empty()
        || value.len() > 100
        || !value
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"-._".contains(&c))
    {
        return Err("invalid spatial identifier".into());
    }
    Ok(())
}
fn origin(meta: &CommandMeta, state: &CampaignState) -> Result<(), String> {
    if meta.campaign_id != state.campaign_id()
        || meta.expected_event_sequence > state.applied_event_sequence
    {
        return Err("spatial origin has foreign campaign or future head".into());
    }
    if matches!(meta.issuer, CommandIssuer::Player(player) if !state.players.contains_key(&player))
        || matches!(meta.actor, Some(AgentRef::Entity(actor)) if !state.entities.contains_key(&actor))
        || matches!(meta.actor, Some(AgentRef::Faction(actor)) if !state.factions.contains_key(&actor))
    {
        return Err("spatial origin references an unknown issuer or actor".into());
    }
    Ok(())
}
impl TacticalEncounter {
    pub fn participant(&self, actor: EntityId) -> Option<&TacticalParticipant> {
        self.participants.iter().find(|p| p.entity_id == actor)
    }
    pub fn validate(&self, state: &CampaignState) -> Result<(), String> {
        self.validate_geometry()?;
        origin(&self.origin, state)?;
        if !matches!(
            self.origin.issuer,
            CommandIssuer::Admin | CommandIssuer::System
        ) || self.origin.actor.is_some()
        {
            return Err("encounter authoring requires the trusted host channel".into());
        }
        let scene = state
            .scenes
            .get(&self.scene_id)
            .ok_or("encounter scene is missing")?;
        if scene.campaign_id != state.campaign_id() {
            return Err("encounter scene belongs to another campaign".into());
        }
        for participant in &self.participants {
            let entity = state
                .entities
                .get(&participant.entity_id)
                .ok_or("encounter entity is missing")?;
            if entity.campaign_id != state.campaign_id()
                || entity.location_id != Some(scene.location_id)
                || !scene.presences.iter().any(|p| {
                    p.entity_id == participant.entity_id && p.role == PresenceRole::Participant
                })
            {
                return Err("encounter entity placement disagrees with scene presence".into());
            }
            if !state
                .rules
                .as_ref()
                .is_some_and(|r| r.entities.contains_key(&participant.entity_id))
            {
                return Err("encounter participant has no authoritative mechanics".into());
            }
        }
        for knowledge in &self.knowledge {
            for contact in &knowledge.contacts {
                origin(&contact.origin, state)?;
            }
            for cell in &knowledge.terrain {
                origin(&cell.origin, state)?;
            }
        }
        Ok(())
    }
    /// Geometry-only checks are also used by immutable queries on a standalone battlefield.
    pub fn validate_geometry(&self) -> Result<(), String> {
        let field = &self.battlefield;
        field.bounds.validate()?;
        identifier(&field.floor_surface)?;
        if field.floor_z < field.bounds.min.z
            || field.floor_z >= field.bounds.max.z
            || [
                field.bounds.min.x,
                field.bounds.min.y,
                field.bounds.max.x,
                field.bounds.max.y,
            ]
            .into_iter()
            .any(|n| n % GRID_SQUARE_UNITS != 0)
        {
            return Err("invalid battlefield floor/grid bounds".into());
        }
        let cells = i64::from((field.bounds.max.x - field.bounds.min.x) / GRID_SQUARE_UNITS)
            * i64::from((field.bounds.max.y - field.bounds.min.y) / GRID_SQUARE_UNITS);
        if cells > MAX_BATTLEFIELD_GRID_CELLS
            || field.terrain.len() > 512
            || field.obstacles.len() > 512
            || field.lights.len() > 128
        {
            return Err("battlefield exceeds bounded geometry capacity".into());
        }
        if self.participants.is_empty()
            || self.participants.len() > MAX_TACTICAL_PARTICIPANTS
            || self.knowledge.len() > self.participants.len()
        {
            return Err("invalid encounter participant/knowledge count".into());
        }
        if self.geometry_ruling.reason.trim().is_empty() || self.geometry_ruling.reason.len() > 4000
        {
            return Err("authored geometry requires a bounded ruling".into());
        }
        let mut features = HashSet::new();
        for terrain in &field.terrain {
            identifier(&terrain.id)?;
            terrain.volume.validate()?;
            if !features.insert(&terrain.id) || !field.bounds.encloses(terrain.volume) {
                return Err("invalid/duplicate terrain volume".into());
            }
            if let Some(surface) = &terrain.surface {
                identifier(surface)?;
            }
        }
        for obstacle in &field.obstacles {
            identifier(&obstacle.id)?;
            obstacle.volume.validate()?;
            if !features.insert(&obstacle.id) || !field.bounds.encloses(obstacle.volume) {
                return Err("invalid/duplicate obstacle volume".into());
            }
        }
        let actors = self
            .participants
            .iter()
            .map(|p| p.entity_id)
            .collect::<HashSet<_>>();
        if actors.len() != self.participants.len() {
            return Err("duplicate encounter participant".into());
        }
        for participant in &self.participants {
            participant.position.validate()?;
            if participant.height == 0
                || participant.height > 200
                || participant.reach == 0
                || participant.reach > 4000
                || !field.bounds.encloses(participant.volume()?)
            {
                return Err("participant extends beyond the battlefield".into());
            }
            let alignment = if participant.size == CreatureSize::Tiny {
                5
            } else {
                GRID_SQUARE_UNITS
            };
            if participant.position.x.rem_euclid(alignment) != 0
                || participant.position.y.rem_euclid(alignment) != 0
                || participant.public_label.trim().is_empty()
                || participant.public_label.len() > 200
            {
                return Err("invalid participant grid alignment or public label".into());
            }
            let speeds = &participant.movement;
            if [
                Some(speeds.walk),
                speeds.climb,
                speeds.swim,
                speeds.fly,
                speeds.burrow,
            ]
            .into_iter()
            .flatten()
            .any(|speed| speed > 2000)
                || (speeds.hover && speeds.fly.is_none())
                || [
                    participant.senses.darkvision,
                    participant.senses.blindsight,
                    participant.senses.tremorsense,
                    participant.senses.truesight,
                ]
                .into_iter()
                .any(|range| range > 4000)
            {
                return Err("unsupported movement/sense profile".into());
            }
            let mut relations = HashSet::new();
            for other in participant.allies.iter().chain(&participant.enemies) {
                if *other == participant.entity_id
                    || !actors.contains(other)
                    || !relations.insert(*other)
                {
                    return Err("invalid/duplicate tactical relation".into());
                }
            }
        }
        for light in &field.lights {
            identifier(&light.id)?;
            light.position.validate()?;
            if !features.insert(&light.id)
                || light.bright_radius > light.dim_radius
                || light.dim_radius > 4000
                || !field.bounds.contains(light.position)
                || light.attached_to.is_some_and(|id| !actors.contains(&id))
            {
                return Err("invalid/duplicate spatial light".into());
            }
        }
        let mut observers = HashSet::new();
        for memory in &self.knowledge {
            if !actors.contains(&memory.observer)
                || !observers.insert(memory.observer)
                || memory.contacts.len() > self.participants.len()
                || memory.terrain.len() > MAX_BATTLEFIELD_GRID_CELLS as usize
            {
                return Err("invalid/duplicate observer knowledge".into());
            }
            let mut targets = HashSet::new();
            for contact in &memory.contacts {
                contact.position.validate()?;
                if contact.target == memory.observer
                    || !actors.contains(&contact.target)
                    || !targets.insert(contact.target)
                    || !field.bounds.contains(contact.position)
                {
                    return Err("invalid/duplicate remembered contact".into());
                }
            }
            let mut cells = HashSet::new();
            for cell in &memory.terrain {
                cell.position.validate()?;
                if !field.bounds.contains(cell.position) || !cells.insert(cell.position) {
                    return Err("invalid/duplicate remembered terrain".into());
                }
            }
        }
        Ok(())
    }
}
