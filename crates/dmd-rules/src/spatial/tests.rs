use super::*;

fn point(x: i32, y: i32, z: i32) -> SpatialPoint {
    SpatialPoint { x, y, z }
}
fn volume(min: SpatialPoint, max: SpatialPoint) -> SpatialBox {
    SpatialBox { min, max }
}
fn direction(x: i8, y: i8, z: i8) -> SpatialDirection {
    SpatialDirection { x, y, z }
}
struct Fixture {
    state: CampaignState,
    encounter: TacticalEncounter,
    a: EntityId,
    b: EntityId,
    c: EntityId,
}
impl Fixture {
    fn new() -> Self {
        let mut state = CampaignState::empty(
            Campaign {
                id: CampaignId::new(),
                display_name: "Independent spatial scenario".into(),
                status: CampaignStatus::Active,
                world_seed: 37,
                ruleset: VersionedRef {
                    id: "srd-5.2".into(),
                    version: "5.2.1".into(),
                },
                content_packs: vec![],
            },
            WorldClock {
                now: WorldInstant(0),
                calendar_id: "test".into(),
            },
        );
        let location = LocationId::new();
        let scene = SceneId::new();
        let a = EntityId::new();
        let b = EntityId::new();
        let c = EntityId::new();
        state.locations.insert(
            location,
            Location {
                id: location,
                campaign_id: state.campaign_id(),
                display_name: "Any setting".into(),
                parent_location_id: None,
            },
        );
        for id in [a, b, c] {
            state.entities.insert(
                id,
                WorldEntity {
                    id,
                    campaign_id: state.campaign_id(),
                    display_name: "SECRET TRUE IDENTITY".into(),
                    kind: EntityKind::Creature,
                    existence: EntityExistence::Present,
                    location_id: Some(location),
                },
            );
        }
        state.scenes.insert(
            scene,
            Scene {
                id: scene,
                campaign_id: state.campaign_id(),
                location_id: location,
                mode: SceneMode::Combat,
                status: SceneStatus::Active,
                started_at: WorldInstant(0),
                presences: vec![a, b, c]
                    .into_iter()
                    .map(|entity_id| ScenePresence {
                        entity_id,
                        role: PresenceRole::Participant,
                    })
                    .collect(),
            },
        );
        let meta = CommandMeta {
            id: CommandId::new(),
            campaign_id: state.campaign_id(),
            session_id: None,
            issuer: CommandIssuer::Admin,
            actor: None,
            expected_event_sequence: 0,
        };
        let ruling = Ruling {
            basis: RulingBasis::GmAdjudication,
            reason: "Authored test geometry; SRD5.2.1 grid convention".into(),
        };
        let pack =
            crate::RulesPack::from_json(include_str!("../../../../content/srd-5.2.1/kernel.json"))
                .unwrap();
        let entities = [a, b, c]
            .into_iter()
            .map(|id| {
                let mut e = MechanicalEntity::basic(id);
                e.ability_scores[0] = 16;
                e
            })
            .collect();
        state = crate::resolve(
            &state,
            &meta,
            &crate::RulesAction::Initialize {
                entities,
                house_rules: HouseRules::default(),
                ruling: ruling.clone(),
            },
            &pack,
        )
        .unwrap()
        .next_state;
        let participants = [
            (a, point(10, 10, 0)),
            (b, point(50, 10, 0)),
            (c, point(70, 70, 0)),
        ]
        .into_iter()
        .map(|(entity_id, position)| TacticalParticipant {
            entity_id,
            position,
            size: CreatureSize::Medium,
            public_label: "Visible traveler".into(),
            height: 12,
            reach: 10,
            movement: MovementProfile {
                walk: 60,
                climb: None,
                swim: None,
                fly: None,
                burrow: None,
                hover: false,
            },
            senses: Senses::default(),
            allies: vec![],
            enemies: vec![],
        })
        .collect();
        let encounter = TacticalEncounter {
            id: EncounterId::new(),
            scene_id: scene,
            battlefield: Battlefield {
                bounds: volume(point(0, 0, -20), point(100, 100, 100)),
                floor_z: 0,
                floor_surface: "ground".into(),
                ambient_light: LightLevel::Bright,
                terrain: vec![],
                obstacles: vec![],
                lights: vec![],
            },
            participants,
            knowledge: vec![],
            origin: meta,
            geometry_ruling: ruling,
        };
        encounter.validate(&state).unwrap();
        Self {
            state,
            encounter,
            a,
            b,
            c,
        }
    }
    fn actor(&mut self, id: EntityId) -> &mut TacticalParticipant {
        self.encounter
            .participants
            .iter_mut()
            .find(|p| p.entity_id == id)
            .unwrap()
    }
    fn condition(&mut self, id: EntityId, condition: Condition) {
        self.state
            .rules
            .as_mut()
            .unwrap()
            .effects
            .push(ActiveEffect {
                id: EffectId::new(),
                source: self.b,
                target: id,
                condition: Some(condition),
                label: "Source condition".into(),
                expires: Expiry::Never,
                concentration_owner: None,
            });
    }
    fn terrain(&mut self, id: &str, volume: SpatialBox) -> &mut TerrainVolume {
        self.encounter.battlefield.terrain.push(TerrainVolume {
            id: id.into(),
            volume,
            difficult: false,
            observable: true,
            water: false,
            climbable: false,
            burrowable: false,
            supports_top: false,
            surface: None,
            obscuration: Obscuration::None,
            magical_darkness: false,
        });
        self.encounter.battlefield.terrain.last_mut().unwrap()
    }
    fn wall(&mut self, id: &str, volume: SpatialBox, cover: CoverDegree, sight: bool) {
        self.encounter.battlefield.obstacles.push(SpatialObstacle {
            id: id.into(),
            volume,
            blocks_movement: true,
            blocks_sight: sight,
            observable: true,
            cover,
        });
    }
    fn path(
        &self,
        steps: &[(SpatialPoint, MovementMode)],
        allowance: MovementAllowance,
    ) -> Result<MovementPlan, SpatialError> {
        evaluate_path(
            &self.encounter,
            &self.state,
            self.a,
            &SpatialPath {
                steps: steps
                    .iter()
                    .map(|(destination, mode)| MovementStep {
                        destination: *destination,
                        mode: *mode,
                    })
                    .collect(),
            },
            &allowance,
        )
    }
}

#[test]
fn model_rejects_foreign_missing_duplicated_and_unbounded_truth() {
    let f = Fixture::new();
    let mut copy = f.encounter.clone();
    copy.origin.campaign_id = CampaignId::new();
    assert!(copy.validate(&f.state).is_err());
    let mut copy = f.encounter.clone();
    copy.origin.issuer = CommandIssuer::Player(PlayerId::new());
    assert!(copy.validate(&f.state).is_err());
    let mut copy = f.encounter.clone();
    copy.origin.actor = Some(AgentRef::Entity(EntityId::new()));
    assert!(copy.validate(&f.state).is_err());
    let mut copy = f.encounter.clone();
    copy.scene_id = SceneId::new();
    assert!(copy.validate(&f.state).is_err());
    let mut copy = f.encounter.clone();
    copy.participants.push(copy.participants[0].clone());
    assert!(copy.validate(&f.state).is_err());
    let mut copy = f.encounter.clone();
    copy.participants[0].position.x = 15;
    assert!(copy.validate(&f.state).is_err());
    copy.participants[0].size = CreatureSize::Tiny;
    assert!(copy.validate(&f.state).is_ok());
    let mut copy = f.encounter.clone();
    copy.participants[0].height = u32::MAX;
    assert!(copy.validate(&f.state).is_err());
    let mut copy = f.encounter.clone();
    copy.battlefield.bounds.max.x = i32::MAX;
    assert!(copy.validate(&f.state).is_err());
    let mut state = f.state.clone();
    state.entities.get_mut(&f.a).unwrap().location_id = None;
    assert!(f.encounter.validate(&state).is_err());
    let mut json = serde_json::to_value(&f.encounter).unwrap();
    json.as_object_mut()
        .unwrap()
        .insert("guaranteed_hit".into(), true.into());
    assert!(serde_json::from_value::<TacticalEncounter>(json).is_err());
}

#[test]
fn grid_distance_and_solid_ray_boundaries_are_checked_integer_geometry() {
    assert_eq!(grid_distance(point(0, 0, 0), point(10, 10, 0)).unwrap(), 10);
    assert_eq!(
        grid_distance(point(-200000, 0, 0), point(200000, 0, 0)).unwrap(),
        400000
    );
    assert!(grid_distance(point(i32::MIN, 0, 0), point(0, 0, 0)).is_err());
    let wall = volume(point(10, 10, 0), point(20, 20, 20));
    assert!(segment_intersects(point(0, 15, 10), point(30, 15, 10), wall).unwrap());
    assert!(!segment_intersects(point(0, 10, 10), point(30, 10, 10), wall).unwrap());
    assert!(!segment_intersects(point(0, 0, 10), point(10, 10, 10), wall).unwrap());
    assert!(segment_intersects(point(15, 15, 10), point(15, 15, 10), wall).unwrap());
    assert!(segment_intersects(point(0, 0, 0), point(i32::MAX, 0, 0), wall).is_err());
}

#[test]
fn movement_uses_grid_cost_additive_difficult_terrain_and_solid_corners() {
    let mut f = Fixture::new();
    assert_eq!(
        f.path(
            &[(point(20, 20, 0), MovementMode::Walk)],
            MovementAllowance::default()
        )
        .unwrap()
        .total_cost,
        10
    );
    f.terrain("rubble", volume(point(20, 20, 0), point(30, 30, 20)))
        .difficult = true;
    f.terrain("foliage", volume(point(20, 20, 0), point(30, 30, 20)))
        .difficult = true;
    assert_eq!(
        f.path(
            &[(point(20, 20, 0), MovementMode::Walk)],
            MovementAllowance::default()
        )
        .unwrap()
        .total_cost,
        20
    );
    f.condition(f.a, Condition::Prone);
    assert_eq!(
        f.path(
            &[(point(20, 20, 0), MovementMode::Crawl)],
            MovementAllowance::default()
        )
        .unwrap()
        .total_cost,
        30
    );
    assert!(
        f.path(
            &[(point(20, 20, 0), MovementMode::Walk)],
            MovementAllowance::default()
        )
        .is_err()
    );
    f.state.rules.as_mut().unwrap().effects.clear();
    f.wall(
        "corner",
        volume(point(20, 10, 0), point(30, 20, 20)),
        CoverDegree::Total,
        true,
    );
    assert!(
        f.path(
            &[(point(20, 20, 0), MovementMode::Walk)],
            MovementAllowance::default()
        )
        .is_err()
    );
    f.encounter.battlefield.obstacles.clear();
    f.wall(
        "thin-wall",
        volume(point(20, 0, 0), point(21, 100, 50)),
        CoverDegree::Total,
        true,
    );
    assert!(
        f.path(
            &[(point(20, 10, 0), MovementMode::Walk)],
            MovementAllowance::default()
        )
        .is_err()
    );
}

#[test]
fn occupancy_distinguishes_allies_tiny_incapacitated_and_endpoint_legality() {
    let mut f = Fixture::new();
    f.actor(f.b).position = point(20, 10, 0);
    let path = [
        (point(20, 10, 0), MovementMode::Walk),
        (point(30, 10, 0), MovementMode::Walk),
    ];
    assert!(f.path(&path, MovementAllowance::default()).is_err());
    let ally = f.b;
    f.actor(f.a).allies.push(ally);
    assert_eq!(
        f.path(&path, MovementAllowance::default())
            .unwrap()
            .total_cost,
        20
    );
    assert!(f.path(&path[..1], MovementAllowance::default()).is_err());
    f.actor(f.a).allies.clear();
    f.condition(f.b, Condition::Incapacitated);
    assert_eq!(
        f.path(&path, MovementAllowance::default())
            .unwrap()
            .total_cost,
        30
    );
    f.state.rules.as_mut().unwrap().effects.clear();
    f.actor(f.b).size = CreatureSize::Tiny;
    assert_eq!(
        f.path(&path, MovementAllowance::default())
            .unwrap()
            .total_cost,
        20
    );
}

#[test]
fn movement_modes_and_speed_switching_do_not_refresh_a_turn_budget() {
    let mut f = Fixture::new();
    f.terrain("water", volume(point(20, 0, -10), point(40, 100, 20)))
        .water = true;
    assert_eq!(
        f.path(
            &[(point(20, 10, 0), MovementMode::Swim)],
            MovementAllowance::default()
        )
        .unwrap()
        .total_cost,
        20
    );
    f.actor(f.a).movement.swim = Some(80);
    assert_eq!(
        f.path(
            &[(point(20, 10, 0), MovementMode::Swim)],
            MovementAllowance::default()
        )
        .unwrap()
        .total_cost,
        10
    );
    assert!(
        f.path(
            &[(point(20, 10, 0), MovementMode::Walk)],
            MovementAllowance {
                spent: 55,
                ..Default::default()
            }
        )
        .is_err()
    );
    assert!(
        f.path(
            &[(point(20, 10, 0), MovementMode::Swim)],
            MovementAllowance {
                spent: 55,
                ..Default::default()
            }
        )
        .is_ok()
    );
    f.condition(f.a, Condition::Stunned);
    assert!(
        f.path(
            &[(point(20, 10, 0), MovementMode::Walk)],
            MovementAllowance::default()
        )
        .is_ok()
    );
    f.condition(f.a, Condition::Restrained);
    assert!(
        f.path(
            &[(point(20, 10, 0), MovementMode::Walk)],
            MovementAllowance::default()
        )
        .is_err()
    );
}

#[test]
fn flying_climbing_burrowing_and_jump_limits_use_real_elevation() {
    let mut f = Fixture::new();
    assert!(
        f.path(
            &[(point(10, 10, 10), MovementMode::Fly)],
            MovementAllowance::default()
        )
        .is_err()
    );
    f.actor(f.a).movement.fly = Some(60);
    assert!(
        !f.path(
            &[(point(10, 10, 10), MovementMode::Fly)],
            MovementAllowance::default()
        )
        .unwrap()
        .falls_at_end
    );
    f.condition(f.a, Condition::Incapacitated);
    assert!(
        f.path(
            &[(point(10, 10, 10), MovementMode::Fly)],
            MovementAllowance::default()
        )
        .is_err()
    );
    f.actor(f.a).movement.hover = true;
    assert!(
        f.path(
            &[(point(10, 10, 10), MovementMode::Fly)],
            MovementAllowance::default()
        )
        .is_ok()
    );
    f.state.rules.as_mut().unwrap().effects.clear();
    f.terrain("ladder", volume(point(10, 10, 0), point(20, 20, 50)))
        .climbable = true;
    assert_eq!(
        f.path(
            &[(point(10, 10, 10), MovementMode::Climb)],
            MovementAllowance::default()
        )
        .unwrap()
        .total_cost,
        20
    );
    f.actor(f.a).movement.climb = Some(60);
    assert_eq!(
        f.path(
            &[(point(10, 10, 10), MovementMode::Climb)],
            MovementAllowance::default()
        )
        .unwrap()
        .total_cost,
        10
    );
    f.encounter.battlefield.terrain.clear();
    f.actor(f.a).movement.burrow = Some(60);
    f.terrain("earth", volume(point(0, 0, -20), point(100, 100, 0)))
        .burrowable = true;
    assert!(
        f.path(
            &[(point(10, 10, -10), MovementMode::Burrow)],
            MovementAllowance::default()
        )
        .is_ok()
    );
    assert!(
        f.path(
            &[(point(10, 10, -10), MovementMode::Walk)],
            MovementAllowance::default()
        )
        .is_err()
    );
    let jump = [
        (point(20, 10, 0), MovementMode::Jump),
        (point(30, 10, 0), MovementMode::Jump),
    ];
    assert!(f.path(&jump, MovementAllowance::default()).is_err());
    assert!(
        f.path(
            &jump,
            MovementAllowance {
                runup: 20,
                ..Default::default()
            }
        )
        .is_ok()
    );
    assert!(
        f.path(
            &[(point(10, 10, 10), MovementMode::Jump)],
            MovementAllowance::default()
        )
        .is_err()
    );
    assert!(
        f.path(
            &[(point(10, 10, 10), MovementMode::Jump)],
            MovementAllowance {
                runup: 20,
                ..Default::default()
            }
        )
        .unwrap()
        .falls_at_end
    );
}

#[test]
fn opportunity_boundaries_use_sight_reach_and_actual_movement_cause() {
    let mut f = Fixture::new();
    f.actor(f.b).position = point(20, 10, 0);
    let enemy = f.a;
    f.actor(f.b).enemies.push(enemy);
    let step = [(point(0, 10, 0), MovementMode::Walk)];
    let plan = f.path(&step, MovementAllowance::default()).unwrap();
    assert_eq!(
        plan.segments[0].opportunities,
        vec![OpportunityCrossing {
            actor: f.b,
            before_leaving: point(10, 10, 0)
        }]
    );
    assert!(
        f.path(
            &step,
            MovementAllowance {
                disengaged: true,
                ..Default::default()
            }
        )
        .unwrap()
        .segments[0]
            .opportunities
            .is_empty()
    );
    assert!(
        f.path(
            &step,
            MovementAllowance {
                forced: true,
                ..Default::default()
            }
        )
        .unwrap()
        .segments[0]
            .opportunities
            .is_empty()
    );
    assert!(
        f.path(
            &[(point(0, 10, 0), MovementMode::Teleport)],
            MovementAllowance {
                teleport_range: Some(20),
                ..Default::default()
            }
        )
        .unwrap()
        .segments[0]
            .opportunities
            .is_empty()
    );
    f.condition(f.a, Condition::Invisible);
    assert!(
        f.path(&step, MovementAllowance::default())
            .unwrap()
            .segments[0]
            .opportunities
            .is_empty()
    );
    f.actor(f.b).senses.blindsight = 60;
    assert_eq!(
        f.path(&step, MovementAllowance::default())
            .unwrap()
            .segments[0]
            .opportunities
            .len(),
        1
    );
    f.condition(f.b, Condition::Incapacitated);
    assert!(
        f.path(&step, MovementAllowance::default())
            .unwrap()
            .segments[0]
            .opportunities
            .is_empty()
    );
}

#[test]
fn cover_is_directional_maximum_not_stacked_and_opaque_geometry_is_separate() {
    let mut f = Fixture::new();
    let origin = f.encounter.participant(f.a).unwrap().center().unwrap();
    let target = f.encounter.participant(f.b).unwrap().volume().unwrap();
    f.actor(f.c).position = point(30, 10, 0);
    assert_eq!(
        cover_from(&f.encounter, origin, target, &[f.a, f.b])
            .unwrap()
            .degree,
        CoverDegree::Half
    );
    f.actor(f.c).position = point(70, 70, 0);
    f.wall(
        "low",
        volume(point(30, 0, 0), point(32, 100, 20)),
        CoverDegree::Half,
        false,
    );
    f.wall(
        "high",
        volume(point(35, 0, 0), point(37, 100, 20)),
        CoverDegree::ThreeQuarters,
        false,
    );
    let cover = cover_from(&f.encounter, origin, target, &[f.a, f.b]).unwrap();
    assert_eq!(cover.armor_and_dexterity_bonus().unwrap(), 5);
    assert!(perceive(&f.encounter, &f.state, f.a, f.b).unwrap().sees);
    assert_eq!(
        cover_from(&f.encounter, point(65, 15, 6), target, &[f.a, f.b])
            .unwrap()
            .degree,
        CoverDegree::None
    );
    f.wall(
        "wall",
        volume(point(40, 0, -10), point(42, 100, 40)),
        CoverDegree::Total,
        true,
    );
    let cover = cover_from(&f.encounter, origin, target, &[f.a, f.b]).unwrap();
    assert_eq!(cover.degree, CoverDegree::Total);
    assert!(cover.armor_and_dexterity_bonus().is_err());
    assert!(!perceive(&f.encounter, &f.state, f.a, f.b).unwrap().sees);
    f.encounter
        .battlefield
        .obstacles
        .last_mut()
        .unwrap()
        .volume
        .max
        .y = 16;
    assert!(
        cover_from(&f.encounter, origin, target, &[f.a, f.b])
            .unwrap()
            .requires_adjudication
    );
}

#[test]
fn illumination_and_special_senses_do_not_conflate_location_with_sight() {
    let mut f = Fixture::new();
    f.encounter.battlefield.ambient_light = LightLevel::Darkness;
    assert!(!perceive(&f.encounter, &f.state, f.a, f.b).unwrap().sees);
    f.actor(f.a).senses.darkvision = 120;
    let result = perceive(&f.encounter, &f.state, f.a, f.b).unwrap();
    assert!(result.sees && result.sight_disadvantage);
    f.condition(f.b, Condition::Invisible);
    assert!(!perceive(&f.encounter, &f.state, f.a, f.b).unwrap().sees);
    f.actor(f.a).senses.tremorsense = 120;
    let result = perceive(&f.encounter, &f.state, f.a, f.b).unwrap();
    assert!(!result.sees && result.precisely_located);
    f.actor(f.b).position.z = 20;
    assert!(
        !perceive(&f.encounter, &f.state, f.a, f.b)
            .unwrap()
            .precisely_located
    );
    f.actor(f.a).senses.blindsight = 120;
    f.condition(f.a, Condition::Blinded);
    assert!(perceive(&f.encounter, &f.state, f.a, f.b).unwrap().sees);
    f.wall(
        "solid",
        volume(point(30, 0, -20), point(40, 100, 100)),
        CoverDegree::Total,
        false,
    );
    assert!(!perceive(&f.encounter, &f.state, f.a, f.b).unwrap().sees);
}

#[test]
fn fog_magic_darkness_and_light_sources_have_distinct_effects() {
    let mut f = Fixture::new();
    f.terrain("fog", volume(point(30, 0, -20), point(80, 100, 80)))
        .obscuration = Obscuration::Heavy;
    assert!(!perceive(&f.encounter, &f.state, f.a, f.b).unwrap().sees);
    assert_eq!(
        cover_from(
            &f.encounter,
            f.encounter.participant(f.a).unwrap().center().unwrap(),
            f.encounter.participant(f.b).unwrap().volume().unwrap(),
            &[f.a, f.b]
        )
        .unwrap()
        .degree,
        CoverDegree::None
    );
    f.actor(f.a).senses.blindsight = 120;
    assert!(perceive(&f.encounter, &f.state, f.a, f.b).unwrap().sees);
    f.actor(f.a).senses.blindsight = 0;
    f.encounter.battlefield.terrain.clear();
    f.terrain("magic", volume(point(30, 0, -20), point(80, 100, 80)))
        .magical_darkness = true;
    f.actor(f.a).senses.darkvision = 120;
    assert!(!perceive(&f.encounter, &f.state, f.a, f.b).unwrap().sees);
    f.actor(f.a).senses.truesight = 120;
    assert!(perceive(&f.encounter, &f.state, f.a, f.b).unwrap().sees);
    f.encounter.battlefield.terrain.clear();
    f.encounter.battlefield.ambient_light = LightLevel::Darkness;
    f.encounter.battlefield.lights.push(SpatialLight {
        id: "torch".into(),
        position: point(55, 15, 6),
        bright_radius: 10,
        dim_radius: 20,
        attached_to: None,
    });
    assert_eq!(
        illumination(&f.encounter, point(60, 15, 6)).unwrap(),
        LightLevel::Bright
    );
    assert_eq!(
        illumination(&f.encounter, point(75, 15, 6)).unwrap(),
        LightLevel::Dim
    );
    assert_eq!(
        illumination(&f.encounter, point(76, 15, 6)).unwrap(),
        LightLevel::Darkness
    );
}

#[test]
fn actor_projection_is_immutable_non_omniscient_and_preserves_old_positions() {
    let mut f = Fixture::new();
    f.wall(
        "screen",
        volume(point(30, 0, -20), point(40, 100, 100)),
        CoverDegree::Total,
        true,
    );
    let before = f.state.clone();
    let geometry_before = f.encounter.clone();
    let view = project_actor_view(&f.encounter, &f.state, f.a).unwrap();
    assert!(view.contacts.is_empty());
    assert!(
        view.cells
            .iter()
            .any(|cell| cell.position.x == 30 && cell.blocked)
    );
    assert!(!serde_json::to_string(&view).unwrap().contains("SECRET"));
    assert_eq!(f.state, before);
    assert_eq!(f.encounter, geometry_before);
    f.actor(f.b).position = point(60, 20, 0);
    f.state
        .rules
        .as_mut()
        .unwrap()
        .entities
        .get_mut(&f.b)
        .unwrap()
        .hp = 1;
    assert_eq!(
        project_actor_view(&f.encounter, &f.state, f.a).unwrap(),
        view
    );
    f.encounter.knowledge.push(ActorKnowledge {
        observer: f.a,
        contacts: vec![RememberedContact {
            target: f.b,
            position: point(50, 10, 0),
            modality: PerceptionModality::Sight,
            origin: f.encounter.origin.clone(),
        }],
        terrain: vec![],
    });
    let remembered = project_actor_view(&f.encounter, &f.state, f.a).unwrap();
    assert_eq!(remembered.contacts.len(), 1);
    assert_eq!(remembered.contacts[0].position, point(50, 10, 0));
    assert_eq!(remembered.contacts[0].status, ContactStatus::Remembered);
    f.actor(f.b).position = point(80, 30, 0);
    assert_eq!(
        project_actor_view(&f.encounter, &f.state, f.a).unwrap(),
        remembered
    );
    let saved = serde_json::to_string(&f.encounter).unwrap();
    let restored: TacticalEncounter = serde_json::from_str(&saved).unwrap();
    assert_eq!(
        project_actor_view(&restored, &f.state, f.a).unwrap(),
        remembered
    );
}

#[test]
fn hidden_terrain_and_invisible_obstacles_do_not_enter_actor_views() {
    let mut f = Fixture::new();
    let before = project_actor_view(&f.encounter, &f.state, f.a).unwrap();
    let hidden = f.terrain(
        "concealed-hazard",
        volume(point(20, 20, 0), point(30, 30, 10)),
    );
    hidden.difficult = true;
    hidden.observable = false;
    f.wall(
        "invisible-barrier",
        volume(point(20, 20, 0), point(30, 30, 10)),
        CoverDegree::Total,
        false,
    );
    f.encounter
        .battlefield
        .obstacles
        .last_mut()
        .unwrap()
        .observable = false;
    assert_eq!(
        project_actor_view(&f.encounter, &f.state, f.a).unwrap(),
        before
    );
    assert!(
        f.path(
            &[(point(20, 20, 0), MovementMode::Walk)],
            MovementAllowance::default()
        )
        .is_err()
    );
}

#[test]
fn all_six_area_shapes_honor_exact_boundaries_and_origin_rules() {
    let f = Fixture::new();
    let origin = point(30, 30, 20);
    let east = direction(1, 0, 0);
    let cone = SpatialArea::Cone {
        origin,
        direction: east,
        length: 20,
        include_origin: false,
    };
    assert!(!area_contains_point(&f.encounter, &cone, origin).unwrap());
    assert!(area_contains_point(&f.encounter, &cone, point(50, 40, 20)).unwrap());
    assert!(!area_contains_point(&f.encounter, &cone, point(50, 41, 20)).unwrap());
    assert!(!area_contains_point(&f.encounter, &cone, point(51, 30, 20)).unwrap());
    let cube = SpatialArea::Cube {
        origin: point(30, 20, 10),
        face_center: origin,
        direction: east,
        size: 20,
        include_origin: false,
    };
    assert!(area_contains_point(&f.encounter, &cube, point(50, 40, 30)).unwrap());
    assert!(!area_contains_point(&f.encounter, &cube, point(30, 20, 10)).unwrap());
    assert!(!area_contains_point(&f.encounter, &cube, point(51, 30, 20)).unwrap());
    let cylinder = SpatialArea::Cylinder {
        origin,
        radius: 10,
        height: 20,
        upward: false,
    };
    assert!(area_contains_point(&f.encounter, &cylinder, point(36, 38, 0)).unwrap());
    assert!(!area_contains_point(&f.encounter, &cylinder, point(36, 39, 0)).unwrap());
    assert!(!area_contains_point(&f.encounter, &cylinder, point(30, 30, 21)).unwrap());
    let line = SpatialArea::Line {
        origin,
        direction: east,
        length: 20,
        width: 10,
        include_origin: false,
    };
    assert!(!area_contains_point(&f.encounter, &line, origin).unwrap());
    assert!(area_contains_point(&f.encounter, &line, point(50, 35, 25)).unwrap());
    assert!(!area_contains_point(&f.encounter, &line, point(50, 36, 20)).unwrap());
    let sphere = SpatialArea::Sphere { origin, radius: 10 };
    assert!(area_contains_point(&f.encounter, &sphere, point(36, 38, 20)).unwrap());
    assert!(!area_contains_point(&f.encounter, &sphere, point(36, 38, 21)).unwrap());
    let emanation = SpatialArea::Emanation {
        source: f.a,
        radius: 10,
        include_origin: false,
    };
    assert!(!area_contains_point(&f.encounter, &emanation, point(15, 15, 6)).unwrap());
    assert!(area_contains_point(&f.encounter, &emanation, point(30, 15, 6)).unwrap());
    assert!(!area_contains_point(&f.encounter, &emanation, point(31, 15, 6)).unwrap());
    let diagonal = SpatialArea::Cone {
        origin,
        direction: direction(1, 1, 0),
        length: 20,
        include_origin: false,
    };
    assert!(area_contains_point(&f.encounter, &diagonal, point(40, 40, 20)).unwrap());
    assert!(!area_contains_point(&f.encounter, &diagonal, point(45, 45, 20)).unwrap());
}

#[test]
fn areas_respect_total_cover_without_confusing_fog_and_are_bounded() {
    let mut f = Fixture::new();
    let area = SpatialArea::Sphere {
        origin: point(15, 15, 5),
        radius: 80,
    };
    let target = point(55, 15, 5);
    assert!(area_reaches_point(&f.encounter, &area, target).unwrap());
    f.terrain("fog", volume(point(30, 0, -20), point(40, 100, 100)))
        .obscuration = Obscuration::Heavy;
    assert!(area_reaches_point(&f.encounter, &area, target).unwrap());
    f.wall(
        "solid",
        volume(point(30, 0, -20), point(40, 100, 100)),
        CoverDegree::Total,
        false,
    );
    assert!(!area_reaches_point(&f.encounter, &area, target).unwrap());
    let cells = area_cells(&f.encounter, &area).unwrap();
    assert!(!cells.is_empty());
    assert!(cells.iter().all(|p| p.x < 30));
    assert!(
        area_contains_point(
            &f.encounter,
            &SpatialArea::Sphere {
                origin: point(i32::MIN, 0, 0),
                radius: u32::MAX
            },
            target
        )
        .is_err()
    );
    assert!(
        area_contains_point(
            &f.encounter,
            &SpatialArea::Cone {
                origin: target,
                direction: direction(0, 0, 0),
                length: 1,
                include_origin: false
            },
            target
        )
        .is_err()
    );
}

#[test]
fn source_conditions_restrict_approach_without_requiring_sight() {
    let mut f = Fixture::new();
    f.condition(f.a, Condition::Frightened);
    assert!(
        f.path(
            &[(point(20, 10, 0), MovementMode::Walk)],
            MovementAllowance::default()
        )
        .is_err()
    );
    assert!(
        f.path(
            &[(point(0, 10, 0), MovementMode::Walk)],
            MovementAllowance::default()
        )
        .is_ok()
    );
    let mut large = f.encounter.participant(f.b).unwrap().clone();
    large.size = CreatureSize::Large;
    large.position = point(20, 10, 0);
    assert_eq!(
        participant_distance(f.encounter.participant(f.a).unwrap(), &large).unwrap(),
        10
    );
}
