use super::*;
use crate::tactical_creatures::{
    CreatureBuildChoice, CreatureHitPointChoice, build_creature, creature_definition, source_size,
};

fn point(x: i32, y: i32, z: i32) -> SpatialPoint {
    SpatialPoint { x, y, z }
}
fn volume(min: SpatialPoint, max: SpatialPoint) -> SpatialBox {
    SpatialBox { min, max }
}

struct Fixture {
    state: CampaignState,
    encounter: TacticalEncounter,
    actor: EntityId,
    target: EntityId,
}
impl Fixture {
    fn new(source: &str) -> Self {
        let pack =
            crate::RulesPack::from_json(include_str!("../../../../content/srd-5.2.1/kernel.json"))
                .unwrap();
        let mut state = CampaignState::empty(
            Campaign {
                id: CampaignId::new(),
                display_name: "Source area fixture".into(),
                status: CampaignStatus::Active,
                world_seed: 9,
                ruleset: VersionedRef {
                    id: pack.id.clone(),
                    version: pack.version.clone(),
                },
                content_packs: vec![],
            },
            WorldClock {
                now: WorldInstant(0),
                calendar_id: "seconds".into(),
            },
        );
        let actor = EntityId::new();
        let target = EntityId::new();
        let place = LocationId::new();
        let scene = SceneId::new();
        state.locations.insert(
            place,
            Location {
                id: place,
                campaign_id: state.campaign_id(),
                display_name: "Field".into(),
                parent_location_id: None,
            },
        );
        for id in [actor, target] {
            state.entities.insert(
                id,
                WorldEntity {
                    id,
                    campaign_id: state.campaign_id(),
                    display_name: "Private true identity".into(),
                    kind: EntityKind::Creature,
                    existence: EntityExistence::Present,
                    location_id: Some(place),
                },
            );
        }
        state.scenes.insert(
            scene,
            Scene {
                id: scene,
                campaign_id: state.campaign_id(),
                location_id: place,
                mode: SceneMode::Combat,
                status: SceneStatus::Active,
                started_at: state.clock.now,
                presences: [actor, target]
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
            expected_event_sequence: state.applied_event_sequence,
        };
        let definition = creature_definition(source).unwrap();
        let size = source_size(definition.statistics.size);
        let built = build_creature(
            &state,
            &meta,
            actor,
            &CreatureBuildChoice {
                definition_id: source.into(),
                size,
                additional_languages: vec![],
                hit_points: CreatureHitPointChoice::Average,
                controller: CreatureController::Autonomous,
                in_lair: false,
            },
        )
        .unwrap();
        let ruling = Ruling {
            basis: RulingBasis::GmAdjudication,
            reason: "Explicit occupied-space center area policy for this map".into(),
        };
        state = crate::resolve(
            &state,
            &meta,
            &crate::RulesAction::Initialize {
                entities: vec![built.mechanics, MechanicalEntity::basic(target)],
                house_rules: HouseRules::default(),
                ruling: ruling.clone(),
            },
            &pack,
        )
        .unwrap()
        .next_state;
        state.rules.as_mut().unwrap().tactical_creatures = Some(TacticalCreatures {
            schema_version: TACTICAL_CREATURES_SCHEMA_VERSION,
            profiles: vec![built.profile],
            runtime: vec![built.runtime],
        });
        let participants = vec![
            TacticalParticipant {
                entity_id: actor,
                position: point(20, 40, 0),
                size,
                public_label: "Source creature".into(),
                height: 10,
                reach: 10,
                movement: built.movement,
                senses: built.senses,
                allies: vec![],
                enemies: vec![],
            },
            TacticalParticipant {
                entity_id: target,
                position: point(60, 40, 0),
                size: CreatureSize::Large,
                public_label: "Creature".into(),
                height: 10,
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
            },
        ];
        let encounter = TacticalEncounter {
            id: EncounterId::new(),
            scene_id: scene,
            battlefield: Battlefield {
                bounds: volume(point(0, 0, -20), point(200, 200, 100)),
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
            flow: None,
        };
        encounter.validate(&state).unwrap();
        Self {
            state,
            encounter,
            actor,
            target,
        }
    }
    fn aim(&self) -> TacticalAreaAim {
        let body = self
            .encounter
            .participant(self.actor)
            .unwrap()
            .volume()
            .unwrap();
        TacticalAreaAim {
            origin: point(body.max.x, 50, 5),
            toward: point(180, 50, 5),
            include_origin: false,
        }
    }
    fn bind(&self) -> BoundAreaGeometry {
        bind_area_geometry(
            &self.encounter,
            &self.state,
            &source_area_program(&self.state, self.actor, "fire-breath").unwrap(),
            self.aim(),
            Some(TacticalAreaGridPolicy::OccupiedCellCentersV1),
        )
        .unwrap()
    }
    fn target_mut(&mut self) -> &mut TacticalParticipant {
        self.encounter
            .participants
            .iter_mut()
            .find(|p| p.entity_id == self.target)
            .unwrap()
    }
    fn wall(&mut self, id: &str, min: SpatialPoint, max: SpatialPoint, cover: CoverDegree) {
        self.encounter.battlefield.obstacles.push(SpatialObstacle {
            id: id.into(),
            volume: volume(min, max),
            blocks_movement: false,
            blocks_sight: true,
            observable: false,
            cover,
        });
    }
}

#[test]
fn actual_source_programs_keep_printed_dimensions_dc_and_damage_without_a_grant_override() {
    for (id, length, dc, count, sides, page) in [
        ("young-red-dragon", 60, 17, 16, 6, 318),
        ("adult-red-dragon", 120, 21, 17, 6, 319),
        ("chimera", 30, 15, 7, 8, 273),
    ] {
        let f = Fixture::new(id);
        let before = f.state.clone();
        let source = source_area_program(&f.state, f.actor, "fire-breath").unwrap();
        assert_eq!(
            (source.length(), source.dc(), source.source_page()),
            (length, dc, page)
        );
        assert_eq!(source.dice(), [DieSpec { count, sides }]);
        assert_eq!(source.ability(), Ability::Dexterity);
        assert_eq!(source.damage_type(), DamageType::Fire);
        assert_eq!(source.modifier(), 0);
        assert!(source.half_on_success());
        assert_eq!(source.source().definition_id, id);
        assert!(source_area_program(&f.state, f.actor, "rend").is_err());
        assert!(source_area_program(&f.state, f.target, "fire-breath").is_err());
        assert_eq!(f.state, before);
    }
}

#[test]
fn absent_host_policy_and_forged_origins_reject_without_source_or_state_changes() {
    let f = Fixture::new("young-red-dragon");
    let program = source_area_program(&f.state, f.actor, "fire-breath").unwrap();
    let before = f.state.clone();
    assert!(bind_area_geometry(&f.encounter, &f.state, &program, f.aim(), None).is_err());
    for aim in [
        TacticalAreaAim {
            origin: point(180, 50, 5),
            ..f.aim()
        },
        TacticalAreaAim {
            toward: f.aim().origin,
            ..f.aim()
        },
        TacticalAreaAim {
            toward: point(MAX_SPATIAL_COORDINATE + 1, 0, 0),
            ..f.aim()
        },
    ] {
        assert!(
            bind_area_geometry(
                &f.encounter,
                &f.state,
                &program,
                aim,
                Some(TacticalAreaGridPolicy::OccupiedCellCentersV1)
            )
            .is_err()
        );
    }
    assert_eq!(f.state, before);
}

#[test]
fn each_occupied_cell_matters_even_when_large_creature_center_is_outside_cone() {
    let mut f = Fixture::new("young-red-dragon");
    f.target_mut().position.y = 60;
    let target = f.encounter.participant(f.target).unwrap();
    let aim = f.aim();
    let cone = crate::spatial::SpatialArea::Cone {
        origin: aim.origin,
        direction: crate::spatial::SpatialDirection { x: 1, y: 0, z: 0 },
        length: 60,
        include_origin: false,
    };
    assert!(
        !crate::spatial::area_contains_point(&f.encounter, &cone, target.center().unwrap())
            .unwrap()
    );
    assert!(f.bind().targets().iter().any(|t| t.actor == f.target));
}

#[test]
fn tiny_and_partial_vertical_occupancies_use_their_actual_occupied_midpoints() {
    let mut f = Fixture::new("young-red-dragon");
    let target = f.target_mut();
    target.size = CreatureSize::Tiny;
    target.position = point(60, 50, 0);
    target.height = 3;
    assert!(f.bind().targets().iter().any(|t| t.actor == f.target));
    f.target_mut().position.z = 30;
    assert!(!f.bind().targets().iter().any(|t| t.actor == f.target));
    // A tall creature's lowest occupied band remains affected even though the
    // full body's center is outside the cone's vertical extent.
    f.target_mut().position.z = 0;
    f.target_mut().height = 90;
    assert!(f.bind().targets().iter().any(|t| t.actor == f.target));
}

#[test]
fn host_grid_policy_resolves_half_and_three_quarters_total_cover_shadows() {
    let mut f = Fixture::new("young-red-dragon");
    assert_eq!(
        f.bind()
            .targets()
            .iter()
            .find(|t| t.actor == f.target)
            .unwrap()
            .cover,
        CoverDegree::None
    );
    f.wall(
        "lower",
        point(50, 0, 0),
        point(51, 50, 10),
        CoverDegree::Total,
    );
    assert_eq!(
        f.bind()
            .targets()
            .iter()
            .find(|t| t.actor == f.target)
            .unwrap()
            .cover,
        CoverDegree::Half
    );
    f.wall(
        "far-upper",
        point(70, 50, 0),
        point(71, 100, 10),
        CoverDegree::Total,
    );
    assert_eq!(
        f.bind()
            .targets()
            .iter()
            .find(|t| t.actor == f.target)
            .unwrap()
            .cover,
        CoverDegree::ThreeQuarters
    );
    f.wall(
        "closed",
        point(50, 0, 0),
        point(51, 100, 10),
        CoverDegree::Total,
    );
    assert!(!f.bind().targets().iter().any(|t| t.actor == f.target));
}

#[test]
fn authored_cover_uses_the_greatest_grade_and_never_adds_bonuses() {
    let mut f = Fixture::new("young-red-dragon");
    f.wall("low", point(50, 0, 0), point(51, 50, 10), CoverDegree::Half);
    f.wall(
        "high",
        point(55, 0, 0),
        point(56, 50, 10),
        CoverDegree::ThreeQuarters,
    );
    assert_eq!(
        f.bind()
            .targets()
            .iter()
            .find(|t| t.actor == f.target)
            .unwrap()
            .cover,
        CoverDegree::ThreeQuarters
    );
}

#[test]
fn membership_is_actual_geometry_independent_of_sight_labels_and_perception_history() {
    let mut f = Fixture::new("young-red-dragon");
    let before = f.bind();
    f.encounter.battlefield.ambient_light = LightLevel::Darkness;
    f.encounter.participants[0].senses = Senses::default();
    f.target_mut().public_label = "A different visible description".into();
    f.state.entities.get_mut(&f.target).unwrap().display_name =
        "Never disclose this identity".into();
    assert_eq!(f.bind(), before);
}

#[test]
fn only_source_creatures_are_affected_and_one_body_has_one_target_record() {
    let mut f = Fixture::new("young-red-dragon");
    assert_eq!(
        f.bind()
            .targets()
            .iter()
            .filter(|t| t.actor == f.target)
            .count(),
        1
    );
    f.state.entities.get_mut(&f.target).unwrap().kind = EntityKind::Object;
    assert!(!f.bind().targets().iter().any(|t| t.actor == f.target));
    f.state.entities.get_mut(&f.target).unwrap().kind = EntityKind::Creature;
    f.state
        .rules
        .as_mut()
        .unwrap()
        .entities
        .get_mut(&f.target)
        .unwrap()
        .death
        .dead = true;
    assert!(!f.bind().targets().iter().any(|t| t.actor == f.target));
}

#[test]
fn one_canonical_damage_result_produces_distinct_save_adjustments_without_attack_semantics() {
    let f = Fixture::new("chimera");
    let program = source_area_program(&f.state, f.actor, "fire-breath").unwrap();
    let id = RollRequestId::new();
    let request = area_amount_request(&program, id, RollVisibility::Public).unwrap();
    assert_eq!(request.roller, Some(f.actor));
    assert_eq!(request.dice, vec![DieSpec { count: 7, sides: 8 }]);
    assert_eq!(request.mode, RollMode::Normal);
    let raw = RollResult {
        request_id: id,
        source: RollSource::Physical,
        dice: [8, 7, 6, 5, 3, 1, 1]
            .into_iter()
            .map(|value| DieResult { sides: 8, value })
            .collect(),
    };
    for success in [false, true] {
        let VitalityOperation::Damage { packet, knockout } =
            area_damage_operation(&program, id, &raw, success).unwrap()
        else {
            panic!()
        };
        assert_eq!(packet.cause, DamageCause::Other);
        assert_eq!(knockout, None);
        assert_eq!(packet.components[0].amounts, vec![31]);
        assert_eq!(
            packet.components[0].adjustments,
            if success {
                vec![DamageAdjustment::Multiply {
                    numerator: 1,
                    denominator: 2,
                }]
            } else {
                vec![]
            }
        );
    }
    let mut bad = raw.clone();
    bad.dice[0].value = 9;
    assert!(area_damage_operation(&program, id, &bad, false).is_err());
    assert!(area_damage_operation(&program, RollRequestId::new(), &raw, false).is_err());
    bad = raw.clone();
    bad.dice.push(DieResult { sides: 8, value: 8 });
    assert!(area_damage_operation(&program, id, &bad, false).is_err());
    assert_eq!(raw.dice.len(), 7);
}

#[test]
fn source_creator_explicitly_controls_only_the_cone_origin_point_inclusion() {
    let f = Fixture::new("young-red-dragon");
    let program = source_area_program(&f.state, f.actor, "fire-breath").unwrap();
    for include_origin in [false, true] {
        let aim = TacticalAreaAim {
            origin: point(35, 55, 5),
            toward: point(180, 55, 5),
            include_origin,
        };
        let bound = bind_area_geometry(
            &f.encounter,
            &f.state,
            &program,
            aim,
            Some(TacticalAreaGridPolicy::OccupiedCellCentersV1),
        )
        .unwrap();
        assert_eq!(
            bound.targets().iter().any(|target| target.actor == f.actor),
            include_origin
        );
    }
}
