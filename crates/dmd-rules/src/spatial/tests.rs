use super::*;
#[path = "falling_tests.rs"]
mod falling_tests;

fn point(x: i32, y: i32, z: i32) -> SpatialPoint {
    SpatialPoint { x, y, z }
}
fn volume(min: SpatialPoint, max: SpatialPoint) -> SpatialBox {
    SpatialBox { min, max }
}
fn direction(x: i32, y: i32, z: i32) -> SpatialDirection {
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
            flow: None,
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
            area_grid_policy: None,
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
fn resumed_segments_preserve_jump_distance_and_transit_occupancy() {
    let mut f = Fixture::new();
    let path = |destination, mode| SpatialPath {
        steps: vec![MovementStep { destination, mode }],
    };
    let first = evaluate_path_progress(
        &f.encounter,
        &f.state,
        f.a,
        &path(point(20, 10, 0), MovementMode::Jump),
        &MovementAllowance::default(),
        &TacticalMovementProgress::default(),
        false,
    )
    .unwrap();
    assert_eq!(first.progress.jump.unwrap().start, point(10, 10, 0));
    f.actor(f.a).position = first.destination;
    // Strength 16 allows a standing long jump of 8ft, not 8ft after every pause.
    assert!(
        evaluate_path_progress(
            &f.encounter,
            &f.state,
            f.a,
            &path(point(30, 10, 0), MovementMode::Jump),
            &MovementAllowance {
                spent: first.total_cost,
                ..Default::default()
            },
            &first.progress,
            true,
        )
        .is_err()
    );
    let running = TacticalMovementProgress {
        walked_runup: 20,
        straight: first.progress.straight,
        jump: Some(TacticalJumpProgress {
            start: point(10, 10, 0),
            had_runup: true,
        }),
    };
    let landed = evaluate_path_progress(
        &f.encounter,
        &f.state,
        f.a,
        &path(point(30, 10, 0), MovementMode::Jump),
        &MovementAllowance {
            spent: 20 + first.total_cost,
            ..Default::default()
        },
        &running,
        true,
    )
    .unwrap();
    assert_eq!(landed.progress.walked_runup, 0);
    assert_eq!(landed.progress.jump, None);
    assert_eq!(
        landed.progress.straight,
        Some(TacticalStraightMovement {
            start: point(10, 10, 0),
            end: point(30, 10, 0),
        })
    );

    f.actor(f.a).position = point(40, 10, 0);
    let ally = f.b;
    f.actor(f.a).allies.push(ally);
    let destination = path(point(50, 10, 0), MovementMode::Walk);
    let transit = evaluate_path_progress(
        &f.encounter,
        &f.state,
        f.a,
        &destination,
        &MovementAllowance::default(),
        &TacticalMovementProgress::default(),
        false,
    )
    .unwrap();
    assert_eq!(transit.total_cost, 10);
    assert!(
        evaluate_path_progress(
            &f.encounter,
            &f.state,
            f.a,
            &destination,
            &MovementAllowance::default(),
            &TacticalMovementProgress::default(),
            true,
        )
        .is_err()
    );
}

#[test]
fn displacement_ends_runup_and_progress_queries_remain_immutable_and_bounded() {
    let f = Fixture::new();
    let before = f.encounter.clone();
    let progress = TacticalMovementProgress {
        walked_runup: 20,
        straight: None,
        jump: Some(TacticalJumpProgress {
            start: point(0, 10, 0),
            had_runup: true,
        }),
    };
    for (mode, allowance) in [
        (
            MovementMode::Walk,
            MovementAllowance {
                forced: true,
                ..Default::default()
            },
        ),
        (
            MovementMode::Teleport,
            MovementAllowance {
                teleport_range: Some(40),
                ..Default::default()
            },
        ),
    ] {
        let plan = evaluate_path_progress(
            &f.encounter,
            &f.state,
            f.a,
            &SpatialPath {
                steps: vec![MovementStep {
                    destination: point(20, 10, 0),
                    mode,
                }],
            },
            &allowance,
            &progress,
            true,
        )
        .unwrap();
        assert_eq!(plan.total_cost, 0);
        assert_eq!(plan.progress, TacticalMovementProgress::default());
        assert!(plan.segments[0].opportunities.is_empty());
        assert_eq!(f.encounter, before);
    }
    let invalid = TacticalMovementProgress {
        walked_runup: 10_001,
        jump: None,
        straight: None,
    };
    assert!(
        evaluate_path_progress(
            &f.encounter,
            &f.state,
            f.a,
            &SpatialPath {
                steps: vec![MovementStep {
                    destination: point(20, 10, 0),
                    mode: MovementMode::Walk
                }]
            },
            &MovementAllowance::default(),
            &invalid,
            true,
        )
        .is_err()
    );
}

#[test]
fn cumulative_movement_cap_is_checked_even_with_remaining_source_speed() {
    let mut f = Fixture::new();
    f.actor(f.a).movement.walk = 2000;
    let allowance = MovementAllowance {
        spent: 9990,
        dash: DashGrants {
            speed: 5,
            ..Default::default()
        },
        ..Default::default()
    };
    let steps = [(point(20, 10, 0), MovementMode::Walk)];
    assert_eq!(f.path(&steps, allowance.clone()).unwrap().total_cost, 10);
    // Six times the selected 2000-unit Speed permits 12000; this rejection is
    // specifically the runtime's cumulative capacity, not exhausted source Speed.
    assert_eq!(
        f.path(
            &steps,
            MovementAllowance {
                spent: 9991,
                ..allowance
            }
        ),
        Err(SpatialError::Capacity)
    );
}

#[test]
fn source_charge_geometry_requires_continuous_forward_travel_toward_the_target() {
    let mut f = Fixture::new();
    let mut straight = None;
    for x in [20, 30, 40, 50] {
        straight = Some(
            append_straight_movement(straight, point(x - 10, 10, 0), point(x, 10, 0)).unwrap(),
        );
    }
    let straight = straight.unwrap();
    assert_eq!(grid_distance(straight.start, straight.end).unwrap(), 40); // 20ft, regardless of terrain cost.
    f.actor(f.a).position = straight.end;
    f.actor(f.b).position = point(80, 10, 0);
    assert!(
        straight_movement_toward(
            f.encounter.participant(f.a).unwrap(),
            f.encounter.participant(f.b).unwrap(),
            &straight
        )
        .unwrap()
    );
    // A turn or reversal begins a new streak: accumulated distance cannot grant Charge.
    for destination in [point(50, 20, 0), point(40, 10, 0)] {
        let changed = append_straight_movement(Some(straight), straight.end, destination).unwrap();
        assert_eq!(changed.start, straight.end);
        assert_eq!(grid_distance(changed.start, changed.end).unwrap(), 10);
    }
    f.actor(f.b).position = point(80, 30, 0);
    assert!(
        !straight_movement_toward(
            f.encounter.participant(f.a).unwrap(),
            f.encounter.participant(f.b).unwrap(),
            &straight
        )
        .unwrap()
    );
    f.actor(f.b).position = point(0, 10, 0);
    assert!(
        !straight_movement_toward(
            f.encounter.participant(f.a).unwrap(),
            f.encounter.participant(f.b).unwrap(),
            &straight
        )
        .unwrap()
    );
    // An externally changed starting point cannot reuse the earlier segment chain.
    let displaced =
        append_straight_movement(Some(straight), point(60, 20, 0), point(70, 20, 0)).unwrap();
    assert_eq!(displaced.start, point(60, 20, 0));
}

#[test]
fn self_location_does_not_bypass_source_sight_requirements() {
    let mut f = Fixture::new();
    f.condition(f.a, Condition::Blinded);
    let self_view = perceive(&f.encounter, &f.state, f.a, f.a).unwrap();
    assert!(!self_view.sees);
    assert!(self_view.precisely_located); // A self/touch effect does not require sight.
    assert_eq!(self_view.modality, None);
    f.actor(f.a).senses.truesight = 60;
    assert!(!perceive(&f.encounter, &f.state, f.a, f.a).unwrap().sees);
    f.actor(f.a).senses.blindsight = 10;
    assert_eq!(
        perceive(&f.encounter, &f.state, f.a, f.a).unwrap().modality,
        Some(PerceptionModality::Blindsight)
    );

    f.state.rules.as_mut().unwrap().effects.clear();
    f.actor(f.a).senses = Senses::default();
    f.encounter.battlefield.ambient_light = LightLevel::Darkness;
    let dark = perceive(&f.encounter, &f.state, f.a, f.a).unwrap();
    assert!(!dark.sees && dark.precisely_located);
    f.actor(f.a).senses.darkvision = 60;
    assert!(perceive(&f.encounter, &f.state, f.a, f.a).unwrap().sees);
    f.condition(f.a, Condition::Invisible);
    assert!(!perceive(&f.encounter, &f.state, f.a, f.a).unwrap().sees);
    f.actor(f.a).senses.truesight = 60;
    assert!(perceive(&f.encounter, &f.state, f.a, f.a).unwrap().sees);
    f.condition(f.a, Condition::Unconscious);
    let unaware = perceive(&f.encounter, &f.state, f.a, f.a).unwrap();
    assert!(!unaware.sees && !unaware.precisely_located);
}

#[test]
fn lifecycle_frightened_blocks_approach_without_a_duplicate_legacy_effect() {
    use crate::tactical_effects::{EffectLifecycleAction, EffectLifecycleOperation};
    let mut f = Fixture::new();
    let meta = CommandMeta {
        id: CommandId::new(),
        expected_event_sequence: f.state.applied_event_sequence,
        ..f.encounter.origin.clone()
    };
    let fear = TacticalEffect {
        id: EffectId::new(),
        source: EffectSource {
            definition_id: "source-fear-clause".into(),
            actor: f.b,
            command: meta.clone(),
            ordinal: 0,
        },
        established_at: None,
        target: TacticalEffectTarget::Creature(f.a),
        concentration_group: None,
        expires: TacticalEffectExpiry::Never,
        overlap: None,
        conditions: vec![EffectCondition {
            id: EffectId::new(),
            condition: Condition::Frightened,
        }],
        defenses: vec![],
        triggers: vec![],
    };
    f.state = crate::tactical_effect_adapter::apply_effect_operation(
        &f.state,
        &meta,
        &EffectLifecycleAction {
            step: 0,
            operation: EffectLifecycleOperation::Install {
                effects: vec![fear],
            },
        },
    )
    .unwrap()
    .0;
    assert!(f.state.rules.as_ref().unwrap().effects.is_empty());
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
fn dash_grants_apply_only_to_the_selected_speed_and_share_spent_movement() {
    let mut f = Fixture::new();
    f.actor(f.a).movement.fly = Some(120);
    let walk = DashGrants {
        speed: 1,
        ..Default::default()
    };
    let fly = DashGrants {
        fly: 1,
        ..Default::default()
    };
    // Units are half-feet: Speed30/Fly60 becomes60/60 with Dash(Speed),
    // or30/120 with Dash(Fly), as explicitly adjudicated in ADR026.
    for (dash, mode, maximum) in [
        (walk.clone(), MovementMode::Walk, 120),
        (walk.clone(), MovementMode::Fly, 120),
        (fly.clone(), MovementMode::Walk, 60),
        (fly.clone(), MovementMode::Fly, 240),
    ] {
        assert!(
            f.path(
                &[(point(20, 10, 0), mode)],
                MovementAllowance {
                    dash: dash.clone(),
                    spent: maximum - 10,
                    ..Default::default()
                }
            )
            .is_ok()
        );
        assert!(
            f.path(
                &[(point(20, 10, 0), mode)],
                MovementAllowance {
                    dash,
                    spent: maximum,
                    ..Default::default()
                }
            )
            .is_err()
        );
    }
    assert!(
        f.path(
            &[
                (point(20, 10, 0), MovementMode::Walk),
                (point(20, 10, 10), MovementMode::Fly),
            ],
            MovementAllowance {
                dash: walk.clone(),
                spent: 110,
                ..Default::default()
            }
        )
        .is_err()
    );
    f.state
        .rules
        .as_mut()
        .unwrap()
        .entities
        .get_mut(&f.a)
        .unwrap()
        .exhaustion = 1;
    assert!(
        f.path(
            &[(point(20, 10, 0), MovementMode::Walk)],
            MovementAllowance {
                dash: walk.clone(),
                spent: 90,
                ..Default::default()
            }
        )
        .is_ok()
    );
    assert!(
        f.path(
            &[(point(20, 10, 0), MovementMode::Walk)],
            MovementAllowance {
                dash: walk,
                spent: 100,
                ..Default::default()
            }
        )
        .is_err()
    );
    assert!(
        f.path(
            &[(point(20, 10, 0), MovementMode::Walk)],
            MovementAllowance {
                dash: DashGrants {
                    speed: u8::MAX,
                    fly: u8::MAX,
                    ..Default::default()
                },
                ..Default::default()
            }
        )
        .is_err()
    );
}

#[test]
fn climbing_and_swimming_dash_fallbacks_follow_the_speed_actually_used() {
    let mut f = Fixture::new();
    let terrain = f.terrain("traversal", volume(point(20, 0, -10), point(40, 100, 20)));
    terrain.water = true;
    terrain.climbable = true;
    for mode in [MovementMode::Climb, MovementMode::Swim] {
        // No special speed: each foot costs two, and Dash(Speed) applies.
        assert_eq!(
            f.path(
                &[(point(20, 10, 0), mode)],
                MovementAllowance {
                    spent: 60,
                    dash: DashGrants {
                        speed: 1,
                        ..Default::default()
                    },
                    ..Default::default()
                }
            )
            .unwrap()
            .total_cost,
            20
        );
        assert!(
            f.path(
                &[(point(20, 10, 0), mode)],
                MovementAllowance {
                    spent: 60,
                    dash: DashGrants {
                        climb: 1,
                        swim: 1,
                        ..Default::default()
                    },
                    ..Default::default()
                }
            )
            .is_err()
        );
    }
    f.actor(f.a).movement.climb = Some(60);
    f.actor(f.a).movement.swim = Some(60);
    for mode in [MovementMode::Climb, MovementMode::Swim] {
        assert!(
            f.path(
                &[(point(20, 10, 0), mode)],
                MovementAllowance {
                    spent: 60,
                    dash: DashGrants {
                        speed: 1,
                        ..Default::default()
                    },
                    ..Default::default()
                }
            )
            .is_err()
        );
        assert_eq!(
            f.path(
                &[(point(20, 10, 0), mode)],
                MovementAllowance {
                    spent: 60,
                    dash: DashGrants {
                        climb: 1,
                        swim: 1,
                        ..Default::default()
                    },
                    ..Default::default()
                }
            )
            .unwrap()
            .total_cost,
            10
        );
    }
}

#[test]
fn dead_creatures_cannot_move_or_teleport_voluntarily_but_can_be_moved() {
    let mut f = Fixture::new();
    let entity = f
        .state
        .rules
        .as_mut()
        .unwrap()
        .entities
        .get_mut(&f.a)
        .unwrap();
    entity.hp = 0;
    entity.death.dead = true;
    f.state.entities.get_mut(&f.a).unwrap().existence = EntityExistence::Dead;
    for mode in [MovementMode::Walk, MovementMode::Teleport] {
        assert!(
            f.path(
                &[(point(20, 10, 0), mode)],
                MovementAllowance {
                    teleport_range: Some(20),
                    ..Default::default()
                }
            )
            .is_err()
        );
        assert_eq!(
            f.path(
                &[(point(20, 10, 0), mode)],
                MovementAllowance {
                    forced: true,
                    teleport_range: Some(20),
                    ..Default::default()
                }
            )
            .unwrap()
            .total_cost,
            0
        );
    }
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
fn dim_only_emitters_never_create_bright_light_at_their_origin() {
    let mut f = Fixture::new();
    f.encounter.battlefield.ambient_light = LightLevel::Darkness;
    f.encounter.battlefield.lights.push(SpatialLight {
        id: "dancing-light".into(),
        position: point(55, 15, 6),
        bright_radius: 0,
        dim_radius: 20,
        attached_to: None,
    });
    for position in [point(55, 15, 6), point(75, 15, 6)] {
        assert_eq!(
            illumination(&f.encounter, position).unwrap(),
            LightLevel::Dim
        );
    }
    f.encounter.battlefield.lights[0].dim_radius = 0;
    assert_eq!(
        illumination(&f.encounter, point(55, 15, 6)).unwrap(),
        LightLevel::Darkness
    );
}

#[test]
fn composite_sampled_cover_cannot_claim_total_across_an_unsampled_gap() {
    let mut f = Fixture::new();
    f.encounter.battlefield.bounds = volume(point(-200, -200, -200), point(200, 200, 200));
    let origin = point(0, 0, 10);
    let target = volume(point(100, 0, 0), point(120, 20, 20));
    f.wall(
        "lower",
        volume(point(50, -100, -100), point(51, 2, 100)),
        CoverDegree::Total,
        true,
    );
    f.wall(
        "upper",
        volume(point(50, 3, -100), point(51, 100, 100)),
        CoverDegree::Total,
        true,
    );
    // Every old sample is blocked, yet an actual ray passes through the opening.
    for sample in geometry::samples(target) {
        assert!(!geometry::clear_effect(&f.encounter.battlefield, origin, sample).unwrap());
    }
    assert!(geometry::clear_effect(&f.encounter.battlefield, origin, point(110, 5, 10)).unwrap());
    let result = cover_from(&f.encounter, origin, target, &[f.a, f.b, f.c]).unwrap();
    assert_ne!(result.degree, CoverDegree::Total);
    assert!(result.requires_adjudication);
    assert!(result.armor_and_dexterity_bonus().is_err());
    f.encounter.battlefield.obstacles.clear();
    f.wall(
        "one-solid",
        volume(point(50, -100, -100), point(51, 100, 100)),
        CoverDegree::Total,
        true,
    );
    assert_eq!(
        cover_from(&f.encounter, origin, target, &[f.a, f.b, f.c]).unwrap(),
        CoverAssessment {
            degree: CoverDegree::Total,
            requires_adjudication: false,
        }
    );
}

#[test]
fn blindsight_requires_a_clear_ray_through_composite_cover() {
    let mut f = Fixture::new();
    f.encounter.battlefield.bounds = volume(point(-200, -200, -200), point(200, 200, 200));
    f.encounter.battlefield.ambient_light = LightLevel::Darkness;
    f.actor(f.a).position = point(-10, -10, 0);
    f.actor(f.a).senses.blindsight = 300;
    f.condition(f.a, Condition::Blinded);
    f.actor(f.b).position = point(100, 0, 0);
    f.actor(f.b).size = CreatureSize::Large;
    f.actor(f.b).height = 20;
    f.wall(
        "lower",
        volume(point(50, -100, -100), point(51, 2, 100)),
        CoverDegree::Total,
        true,
    );
    f.wall(
        "upper",
        volume(point(50, 2, -100), point(51, 100, 100)),
        CoverDegree::Total,
        true,
    );
    let from = f.encounter.participant(f.a).unwrap().center().unwrap();
    let target = f.encounter.participant(f.b).unwrap().volume().unwrap();
    // Neither panel proves complete cover on its own; together they are a solid wall.
    let cover = cover_from(&f.encounter, from, target, &[f.a, f.b, f.c]).unwrap();
    assert_ne!(cover.degree, CoverDegree::Total);
    assert!(cover.requires_adjudication);
    let blocked = perceive(&f.encounter, &f.state, f.a, f.b).unwrap();
    assert!(!blocked.sees && !blocked.precisely_located);
    assert_eq!(blocked.modality, None);
    assert!(
        !project_actor_view(&f.encounter, &f.state, f.a)
            .unwrap()
            .contacts
            .iter()
            .any(|contact| contact.entity_id == f.b)
    );

    // Opening a gap crossed by the target-center ray establishes actual visibility.
    // An unsampled opening remains uncertain; no contact is fabricated from that.
    f.encounter.battlefield.obstacles[0].volume.max.y = 1;
    f.encounter.battlefield.obstacles[1].volume.min.y = 4;
    assert!(
        geometry::clear_effect(
            &f.encounter.battlefield,
            from,
            f.encounter.participant(f.b).unwrap().center().unwrap()
        )
        .unwrap()
    );
    let visible = perceive(&f.encounter, &f.state, f.a, f.b).unwrap();
    assert!(visible.sees && visible.precisely_located);
    assert_eq!(visible.modality, Some(PerceptionModality::Blindsight));
    assert!(
        project_actor_view(&f.encounter, &f.state, f.a)
            .unwrap()
            .contacts
            .iter()
            .any(|contact| contact.entity_id == f.b)
    );
}

#[test]
fn magical_darkness_blocks_crossing_rays_but_truesight_does_not_bypass_fog() {
    let mut f = Fixture::new();
    f.terrain("darkness", volume(point(30, 0, -20), point(40, 100, 80)))
        .magical_darkness = true;
    // Target remains in bright light beyond the obscured slab.
    assert_eq!(
        illumination(&f.encounter, point(55, 15, 6)).unwrap(),
        LightLevel::Bright
    );
    assert!(!perceive(&f.encounter, &f.state, f.a, f.b).unwrap().sees);
    f.actor(f.a).senses.darkvision = 120;
    assert!(!perceive(&f.encounter, &f.state, f.a, f.b).unwrap().sees);
    let hidden = project_actor_view(&f.encounter, &f.state, f.a).unwrap();
    assert!(hidden.contacts.is_empty());
    assert!(hidden.cells.iter().all(|cell| cell.position.x < 30));
    f.actor(f.a).senses.truesight = 24;
    assert!(!perceive(&f.encounter, &f.state, f.a, f.b).unwrap().sees);
    // Entire obscured part lies within 25 units; bright target may be farther away.
    f.actor(f.a).senses.truesight = 25;
    assert!(perceive(&f.encounter, &f.state, f.a, f.b).unwrap().sees);
    f.actor(f.a).senses.truesight = 120;
    let visible = project_actor_view(&f.encounter, &f.state, f.a).unwrap();
    assert!(
        visible
            .contacts
            .iter()
            .any(|contact| contact.entity_id == f.b)
    );
    assert!(visible.cells.iter().any(|cell| cell.position.x == 50));
    // A separate heavy obscuration cause remains opaque, even in the same volume.
    f.encounter.battlefield.terrain[0].obscuration = Obscuration::Heavy;
    assert!(!perceive(&f.encounter, &f.state, f.a, f.b).unwrap().sees);
    f.actor(f.a).senses.blindsight = 120;
    assert!(perceive(&f.encounter, &f.state, f.a, f.b).unwrap().sees);
}

#[test]
fn projection_prunes_zero_output_lights_and_rejects_excessive_combined_work() {
    let mut f = Fixture::new();
    f.encounter.battlefield.bounds = volume(point(0, 0, -20), point(1280, 2560, 100));
    f.encounter.battlefield.ambient_light = LightLevel::Darkness;
    for n in 0..128 {
        f.encounter.battlefield.lights.push(SpatialLight {
            id: format!("lamp-{n}"),
            position: point(15, 15, 6),
            bright_radius: 0,
            dim_radius: 0,
            attached_to: None,
        });
    }
    // The maximum sparse floor and 128 extinguished emitters remain usable.
    assert!(
        project_actor_view(&f.encounter, &f.state, f.a)
            .unwrap()
            .cells
            .is_empty()
    );
    for light in &mut f.encounter.battlefield.lights {
        light.dim_radius = 4000;
    }
    for n in 0..128 {
        f.wall(
            &format!("obstacle-{n}"),
            volume(point(1200, 2400, 0), point(1210, 2410, 10)),
            CoverDegree::Half,
            false,
        );
    }
    let before = f.encounter.clone();
    assert_eq!(
        project_actor_view(&f.encounter, &f.state, f.a),
        Err(SpatialError::Capacity)
    );
    assert_eq!(f.encounter, before);
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
fn unaware_observers_keep_only_memory_despite_special_senses_or_hidden_movement() {
    for cause in ["condition", "zero-hp", "dead"] {
        let mut f = Fixture::new();
        f.actor(f.a).senses = Senses {
            darkvision: 200,
            blindsight: 200,
            tremorsense: 200,
            truesight: 200,
        };
        assert!(perceive(&f.encounter, &f.state, f.a, f.b).unwrap().sees);
        f.encounter.knowledge.push(ActorKnowledge {
            observer: f.a,
            contacts: vec![RememberedContact {
                target: f.b,
                position: point(50, 10, 0),
                modality: PerceptionModality::Sight,
                origin: f.encounter.origin.clone(),
            }],
            terrain: vec![RememberedCell {
                position: point(10, 10, 0),
                difficult: false,
                blocked: false,
                origin: f.encounter.origin.clone(),
            }],
        });
        if cause == "condition" {
            f.condition(f.a, Condition::Unconscious);
        } else {
            let entity = f
                .state
                .rules
                .as_mut()
                .unwrap()
                .entities
                .get_mut(&f.a)
                .unwrap();
            entity.hp = 0;
            entity.prone = true;
            if cause == "dead" {
                entity.death.dead = true;
                f.state.entities.get_mut(&f.a).unwrap().existence = EntityExistence::Dead;
            }
        }
        let before_state = f.state.clone();
        let before_encounter = f.encounter.clone();
        let view = project_actor_view(&f.encounter, &f.state, f.a).unwrap();
        assert_eq!(view.position, None);
        assert_eq!(view.contacts.len(), 1);
        assert_eq!(view.contacts[0].status, ContactStatus::Remembered);
        assert_eq!(view.contacts[0].position, point(50, 10, 0));
        assert_eq!(view.cells.len(), 1);
        assert!(!view.cells[0].currently_seen);
        assert_eq!(f.state, before_state);
        assert_eq!(f.encounter, before_encounter);
        for target in [f.a, f.b, f.c] {
            let perception = perceive(&f.encounter, &f.state, f.a, target).unwrap();
            assert!(!perception.sees && !perception.precisely_located);
            assert_eq!(perception.modality, None);
        }
        f.actor(f.a).position = point(0, 0, 0);
        f.actor(f.b).position = point(80, 20, 0);
        f.terrain("new-terrain", volume(point(20, 0, 0), point(40, 100, 20)))
            .difficult = true;
        assert_eq!(
            project_actor_view(&f.encounter, &f.state, f.a).unwrap(),
            view
        );
        let restored: TacticalEncounter =
            serde_json::from_str(&serde_json::to_string(&f.encounter).unwrap()).unwrap();
        assert_eq!(project_actor_view(&restored, &f.state, f.a).unwrap(), view);
    }
}

#[test]
fn incapacitated_and_stunned_observers_retain_source_awareness() {
    for condition in [Condition::Incapacitated, Condition::Stunned] {
        let mut f = Fixture::new();
        f.condition(f.a, condition);
        assert!(perceive(&f.encounter, &f.state, f.a, f.b).unwrap().sees);
        let view = project_actor_view(&f.encounter, &f.state, f.a).unwrap();
        assert_eq!(view.position, Some(point(10, 10, 0)));
        assert!(
            view.contacts
                .iter()
                .all(|contact| contact.status == ContactStatus::Seen)
        );
        assert!(view.cells.iter().any(|cell| cell.currently_seen));
        assert!(
            f.path(
                &[(point(20, 10, 0), MovementMode::Walk)],
                MovementAllowance::default()
            )
            .is_ok()
        );
    }
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
fn arbitrary_integer_directions_preserve_orientation_scale_and_overflow_bounds() {
    let f = Fixture::new();
    let origin = point(30, 30, 20);
    let contains = |direction, target| {
        area_contains_point(
            &f.encounter,
            &SpatialArea::Line {
                origin,
                direction,
                length: 30,
                width: 2,
                include_origin: false,
            },
            target,
        )
        .unwrap()
    };
    assert!(contains(direction(2, 1, 0), point(50, 40, 20)));
    assert!(!contains(direction(2, 1, 0), point(40, 40, 20)));
    assert!(contains(direction(2, 1, 1), point(50, 40, 30)));
    for target in [point(50, 40, 20), point(40, 40, 20), point(20, 20, 20)] {
        assert_eq!(
            contains(direction(2, 1, 0), target),
            contains(direction(200_000, 100_000, 0), target)
        );
    }
    let cone = SpatialArea::Cone {
        origin,
        direction: direction(2, 1, 1),
        length: 30,
        include_origin: false,
    };
    assert!(area_contains_point(&f.encounter, &cone, point(50, 40, 30)).unwrap());
    assert!(!area_contains_point(&f.encounter, &cone, point(10, 20, 10)).unwrap());
    let cube = SpatialArea::Cube {
        origin,
        face_center: origin,
        direction: direction(2, 1, 0),
        size: 30,
        include_origin: false,
    };
    assert!(area_contains_point(&f.encounter, &cube, point(50, 40, 20)).unwrap());
    assert!(!area_contains_point(&f.encounter, &cube, point(10, 20, 20)).unwrap());
    for bad in [i32::MIN, i32::MAX, MAX_SPATIAL_DIRECTION_COMPONENT + 1] {
        assert!(
            area_contains_point(
                &f.encounter,
                &SpatialArea::Line {
                    origin,
                    direction: direction(bad, 1, 0),
                    length: 1,
                    width: 1,
                    include_origin: false,
                },
                origin
            )
            .is_err()
        );
    }
    // Exercise cross-basis squares at maximum coordinate and direction deltas.
    let minimum = point(
        -MAX_SPATIAL_COORDINATE,
        -MAX_SPATIAL_COORDINATE,
        -MAX_SPATIAL_COORDINATE,
    );
    let maximum = point(
        MAX_SPATIAL_COORDINATE,
        MAX_SPATIAL_COORDINATE,
        MAX_SPATIAL_COORDINATE,
    );
    let diagonal = direction(
        MAX_SPATIAL_DIRECTION_COMPONENT,
        MAX_SPATIAL_DIRECTION_COMPONENT,
        MAX_SPATIAL_DIRECTION_COMPONENT,
    );
    assert!(
        !area_contains_point(
            &f.encounter,
            &SpatialArea::Line {
                origin: minimum,
                direction: diagonal,
                length: 4000,
                width: 4000,
                include_origin: false,
            },
            maximum
        )
        .unwrap()
    );
    // A cube validates the far-offset face using cross-basis arithmetic before rejecting it.
    assert!(
        SpatialArea::Cube {
            origin: maximum,
            face_center: minimum,
            direction: direction(
                MAX_SPATIAL_DIRECTION_COMPONENT,
                -MAX_SPATIAL_DIRECTION_COMPONENT,
                0
            ),
            size: 4000,
            include_origin: false,
        }
        .validate(&f.encounter)
        .is_err()
    );
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
