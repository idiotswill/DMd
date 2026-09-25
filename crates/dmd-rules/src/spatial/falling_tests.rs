use super::*;
use crate::tactical_falling::*;

fn falling_fixture() -> Fixture {
    let mut f = Fixture::new();
    f.actor(f.a).position.z = 80;
    f
}

#[test]
fn falling_uses_first_physical_contact_including_small_ledges() {
    let mut f = falling_fixture();
    // Neither abstract difficult terrain nor obscuration is a physical platform.
    let abstract_region = f.terrain("fog", volume(point(0, 0, 0), point(100, 100, 70)));
    abstract_region.obscuration = Obscuration::Heavy;
    abstract_region.difficult = true;
    assert_eq!(
        fall_destination(&f.encounter, f.a).unwrap().unwrap().to.z,
        0
    );
    f.terrain("platform", volume(point(10, 10, 0), point(20, 20, 30)))
        .supports_top = true;
    assert_eq!(
        fall_destination(&f.encounter, f.a).unwrap().unwrap().to.z,
        30
    );
    // A positive half-foot overlap catches the body under the explicit map convention.
    f.wall(
        "ledge",
        volume(point(19, 10, 45), point(21, 20, 50)),
        CoverDegree::Total,
        true,
    );
    let path = fall_destination(&f.encounter, f.a).unwrap().unwrap();
    assert_eq!(path.to, point(10, 10, 50));
    assert_eq!(
        path.surface,
        FallSurface::SolidObstacle { id: "ledge".into() }
    );
    f.encounter.battlefield.obstacles[0].volume.min.x = 20;
    assert_eq!(
        fall_destination(&f.encounter, f.a).unwrap().unwrap().to.z,
        30
    );
    // A restored origin actually inside a collider is invalid, not teleported through it.
    f.encounter.battlefield.obstacles[0].volume.min.x = 19;
    f.encounter.battlefield.obstacles[0].volume.max.z = 85;
    assert!(fall_destination(&f.encounter, f.a).is_err());
    f.encounter.battlefield.obstacles.clear();
    // Burrowable ground is already a physical solid in the movement authority.
    // The fall cannot tunnel through it merely because supports_top was omitted.
    f.terrain("earth", volume(point(10, 10, 0), point(20, 20, 60)))
        .burrowable = true;
    assert_eq!(
        fall_destination(&f.encounter, f.a).unwrap().unwrap().to.z,
        60
    );
}

#[test]
fn falling_honors_water_height_solid_ties_and_involuntary_occupancy() {
    let mut f = falling_fixture();
    f.terrain("water", volume(point(0, 0, 0), point(100, 100, 20)))
        .water = true;
    let water = fall_destination(&f.encounter, f.a).unwrap().unwrap();
    assert_eq!(water.to.z, 20);
    assert!(matches!(water.surface, FallSurface::Liquid { .. }));
    // Creature occupancy cannot quietly create damage, a new platform, or side relocation.
    f.actor(f.b).position = point(10, 10, 20);
    assert_eq!(fall_destination(&f.encounter, f.a).unwrap().unwrap(), water);
    f.wall(
        "raft",
        volume(point(10, 10, 19), point(20, 20, 20)),
        CoverDegree::Half,
        false,
    );
    assert!(matches!(
        fall_destination(&f.encounter, f.a)
            .unwrap()
            .unwrap()
            .surface,
        FallSurface::SolidObstacle { .. }
    ));
    f.encounter.battlefield.obstacles.clear();
    f.actor(f.a).position.z = 10;
    assert!(fall_destination(&f.encounter, f.a).unwrap().is_none());
    f.actor(f.a).position.z = 20;
    assert!(fall_destination(&f.encounter, f.a).unwrap().is_none());
}

#[test]
fn flying_loss_uses_real_conditions_and_hover_never_suspends_a_corpse() {
    let mut f = falling_fixture();
    f.actor(f.a).movement.fly = Some(60);
    assert!(
        flight_loss_fall(&f.encounter, &f.state, f.a)
            .unwrap()
            .is_none()
    );
    f.condition(f.a, Condition::Incapacitated);
    assert!(
        flight_loss_fall(&f.encounter, &f.state, f.a)
            .unwrap()
            .is_some()
    );
    f.actor(f.a).movement.hover = true;
    assert!(
        flight_loss_fall(&f.encounter, &f.state, f.a)
            .unwrap()
            .is_none()
    );
    f.state.rules.as_mut().unwrap().effects.clear();
    f.state
        .rules
        .as_mut()
        .unwrap()
        .entities
        .get_mut(&f.a)
        .unwrap()
        .prone = true;
    f.actor(f.a).movement.hover = false;
    assert!(
        flight_loss_fall(&f.encounter, &f.state, f.a)
            .unwrap()
            .is_some()
    );
    f.state
        .rules
        .as_mut()
        .unwrap()
        .entities
        .get_mut(&f.a)
        .unwrap()
        .prone = false;
    f.actor(f.a).movement.fly = Some(10);
    f.state
        .rules
        .as_mut()
        .unwrap()
        .entities
        .get_mut(&f.a)
        .unwrap()
        .exhaustion = 1;
    assert!(
        flight_loss_fall(&f.encounter, &f.state, f.a)
            .unwrap()
            .is_some()
    );
    f.actor(f.a).movement.hover = true;
    assert!(
        flight_loss_fall(&f.encounter, &f.state, f.a)
            .unwrap()
            .is_none()
    );
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
    let path = flight_loss_fall(&f.encounter, &f.state, f.a)
        .unwrap()
        .unwrap();
    assert_eq!(path.to.z, 0);
    f.actor(f.a).position = path.to;
    assert!(
        flight_loss_fall(&f.encounter, &f.state, f.a)
            .unwrap()
            .is_none()
    );
}

#[test]
fn fall_dice_count_full_tens_of_feet_and_cap_without_overflow() {
    let f = falling_fixture();
    let mut path = fall_destination(&f.encounter, f.a).unwrap().unwrap();
    let id = RollRequestId::new();
    for (height, count) in [
        (1, 0),
        (19, 0),
        (20, 1),
        (39, 1),
        (40, 2),
        (400, 20),
        (400_000, 20),
    ] {
        path.from.z = if height == 400_000 { 200_000 } else { height };
        path.to.z = if height == 400_000 { -200_000 } else { 0 };
        let request = fall_damage_request(&path, f.a, id, RollVisibility::Public).unwrap();
        assert_eq!(request.map(|r| r.dice[0].count).unwrap_or(0), count);
    }
    path.from.z = i32::MAX;
    assert!(fall_distance(&path).is_err());
    path.from.z = 20;
    path.to = point(20, 10, 0);
    assert!(fall_distance(&path).is_err());
    path.to = path.from;
    assert!(fall_distance(&path).is_err());
}

#[test]
fn liquid_reaction_requires_actual_capacity_and_check_uses_source_skill() {
    let mut f = falling_fixture();
    let rules = f.state.rules.as_mut().unwrap();
    rules.timing = Some(CombatTiming {
        order: vec![InitiativeEntry {
            actor: f.a,
            total: 10,
            tie_break: 0,
        }],
        index: 0,
        round: 1,
        turn_number: 1,
        action_spent: false,
        bonus_action_spent: false,
        slot_spent_this_turn: false,
        reactions_spent: vec![],
    });
    let entity = rules.entities.get_mut(&f.a).unwrap();
    entity
        .skill_proficiencies
        .insert(Skill::Athletics, Proficiency::Proficient);
    entity.exhaustion = 1;
    let id = RollRequestId::new();
    assert!(can_attempt_liquid_landing(&f.state, f.a).unwrap());
    let request = liquid_landing_request(
        &f.state,
        f.a,
        LiquidLandingChoice::Athletics,
        id,
        RollVisibility::Public,
    )
    .unwrap();
    assert_eq!(request.modifier, 3); // Strength16 + proficiency2 - exhaustion2.
    f.state
        .rules
        .as_mut()
        .unwrap()
        .timing
        .as_mut()
        .unwrap()
        .reactions_spent
        .push(f.a);
    assert!(!can_attempt_liquid_landing(&f.state, f.a).unwrap());
    // The accepted choice has spent the Reaction, but its persisted request rederives.
    assert_eq!(
        liquid_landing_request(
            &f.state,
            f.a,
            LiquidLandingChoice::Athletics,
            id,
            RollVisibility::Public
        )
        .unwrap(),
        request
    );
    f.condition(f.a, Condition::Poisoned);
    assert_eq!(
        liquid_landing_request(
            &f.state,
            f.a,
            LiquidLandingChoice::Acrobatics,
            id,
            RollVisibility::Public
        )
        .unwrap()
        .mode,
        RollMode::Disadvantage
    );
    f.condition(f.a, Condition::Incapacitated);
    assert!(!can_attempt_liquid_landing(&f.state, f.a).unwrap());
    assert!(
        liquid_landing_request(
            &f.state,
            f.a,
            LiquidLandingChoice::Athletics,
            id,
            RollVisibility::Public
        )
        .is_err()
    );
}

#[test]
fn liquid_success_halves_before_target_defenses_and_keeps_raw_faces() {
    use crate::tactical_damage::*;
    let mut f = falling_fixture();
    f.terrain("water", volume(point(0, 0, 0), point(100, 100, 20)))
        .water = true;
    f.actor(f.a).position.z = 40;
    let path = fall_destination(&f.encounter, f.a).unwrap().unwrap();
    let check_id = RollRequestId::new();
    let raw = RollResult {
        request_id: check_id,
        source: RollSource::Physical,
        dice: vec![DieResult {
            sides: 20,
            value: 12,
        }],
    };
    let success = resolve_liquid_landing(
        &f.state,
        f.a,
        &path,
        LiquidLandingChoice::Athletics,
        check_id,
        RollVisibility::Public,
        &raw,
    )
    .unwrap();
    assert!(success.successful()); // Strength16 modifier+3 reaches15 exactly.
    let id = RollRequestId::new();
    let damage = RollResult {
        request_id: id,
        source: RollSource::Physical,
        dice: vec![DieResult { sides: 6, value: 5 }],
    };
    let packet = fall_damage_packet(&path, f.a, id, Some(&damage), Some(&success)).unwrap();
    let entity = f
        .state
        .rules
        .as_mut()
        .unwrap()
        .entities
        .get_mut(&f.a)
        .unwrap();
    entity.resistances.insert(DamageType::Bludgeoning);
    entity.temporary_hp = 10;
    let context = VitalityContext {
        origin: VitalityOrigin {
            command: f.encounter.origin.clone(),
            occurrence: 0,
        },
        now: f.state.clock.now,
        conditions: Default::default(),
        underwater: false,
        defenses: VitalityDefenses::default(),
        death_save_mode: RollMode::Normal,
        death_save_bonus: 0,
    };
    let reduced = reduce_vitality(
        entity,
        &TacticalRecovery::default(),
        &context,
        &VitalityOperation::Damage {
            packet,
            knockout: None,
        },
    )
    .unwrap();
    assert_eq!(reduced.outcome.damage_taken, 1); // floor(5/2), then resistance.
    assert_eq!(reduced.outcome.hp_lost, 0);
    assert_eq!(reduced.outcome.temporary_hp_lost, 1);
    assert_eq!(damage.dice[0].value, 5);
    // The later landing adapter must use damage_taken for Prone, not HP loss.
    let mut wrong_path = path.clone();
    wrong_path.from.z += 1;
    assert!(fall_damage_packet(&wrong_path, f.a, id, Some(&damage), Some(&success)).is_err());
    assert!(fall_damage_packet(&path, f.b, id, Some(&damage), Some(&success)).is_err());
    assert!(fall_damage_packet(&path, f.a, RollRequestId::new(), Some(&damage), None).is_err());
    let mut failed = raw.clone();
    failed.dice[0].value = 11;
    assert!(
        !resolve_liquid_landing(
            &f.state,
            f.a,
            &path,
            LiquidLandingChoice::Athletics,
            check_id,
            RollVisibility::Public,
            &failed
        )
        .unwrap()
        .successful()
    );
}

#[test]
fn short_falls_do_not_fabricate_raw_dice_or_damage() {
    let f = falling_fixture();
    let mut path = fall_destination(&f.encounter, f.a).unwrap().unwrap();
    path.from.z = 19;
    let id = RollRequestId::new();
    let packet = fall_damage_packet(&path, f.a, id, None, None).unwrap();
    assert_eq!(packet.components[0].amounts, vec![0]);
    let forged = RollResult {
        request_id: id,
        source: RollSource::Physical,
        dice: vec![DieResult { sides: 6, value: 1 }],
    };
    assert!(fall_damage_packet(&path, f.a, id, Some(&forged), None).is_err());
}

#[test]
fn off_map_frightened_source_does_not_block_or_penalize_liquid_landing() {
    let mut f = falling_fixture();
    f.condition(f.a, Condition::Frightened);
    f.state.encounter = Some(f.encounter.clone());
    f.state
        .encounter
        .as_mut()
        .unwrap()
        .participants
        .retain(|p| p.entity_id != f.b);
    let request = liquid_landing_request(
        &f.state,
        f.a,
        LiquidLandingChoice::Athletics,
        RollRequestId::new(),
        RollVisibility::Public,
    )
    .unwrap();
    assert_eq!(request.mode, RollMode::Normal);
    // The same real source participates and is visible again: source disadvantage applies.
    f.state.encounter = Some(f.encounter.clone());
    let request = liquid_landing_request(
        &f.state,
        f.a,
        LiquidLandingChoice::Athletics,
        RollRequestId::new(),
        RollVisibility::Public,
    )
    .unwrap();
    assert_eq!(request.mode, RollMode::Disadvantage);
}

#[test]
fn liquid_landing_preserves_the_campaigns_opted_in_natural_extremes_policy() {
    let mut f = falling_fixture();
    f.terrain("water", volume(point(0, 0, 0), point(100, 100, 20)))
        .water = true;
    let path = fall_destination(&f.encounter, f.a).unwrap().unwrap();
    let id = RollRequestId::new();
    let mut raw = RollResult {
        request_id: id,
        source: RollSource::Physical,
        dice: vec![DieResult {
            sides: 20,
            value: 1,
        }],
    };
    let entity = f
        .state
        .rules
        .as_mut()
        .unwrap()
        .entities
        .get_mut(&f.a)
        .unwrap();
    entity.ability_scores[Ability::Strength.index()] = 30;
    entity
        .skill_proficiencies
        .insert(Skill::Athletics, Proficiency::Expertise);
    let outcome = |state: &CampaignState, raw: &RollResult| {
        resolve_liquid_landing(
            state,
            f.a,
            &path,
            LiquidLandingChoice::Athletics,
            id,
            RollVisibility::Public,
            raw,
        )
        .unwrap()
        .successful()
    };
    assert!(outcome(&f.state, &raw)); // RAW natural1 +14 still meets DC15.
    f.state
        .rules
        .as_mut()
        .unwrap()
        .house_rules
        .ability_test_natural_extremes = true;
    assert!(!outcome(&f.state, &raw));
    let entity = f
        .state
        .rules
        .as_mut()
        .unwrap()
        .entities
        .get_mut(&f.a)
        .unwrap();
    entity.ability_scores[Ability::Strength.index()] = 1;
    entity.skill_proficiencies.clear();
    entity.exhaustion = 5;
    raw.dice[0].value = 20;
    assert!(outcome(&f.state, &raw));
    f.state
        .rules
        .as_mut()
        .unwrap()
        .house_rules
        .ability_test_natural_extremes = false;
    assert!(!outcome(&f.state, &raw)); // RAW 20-5-10 fails; no rewritten natural face.
    assert_eq!(raw.dice[0].value, 20);
}

#[test]
fn burrow_support_requires_actual_ground_coverage_and_keeps_nonblocking_regions_legal() {
    let mut f = falling_fixture();
    f.actor(f.a).movement.fly = Some(60);
    f.actor(f.a).movement.burrow = Some(60);
    f.condition(f.a, Condition::Incapacitated);
    f.terrain("side-contact", volume(point(19, 10, 0), point(30, 20, 90)))
        .burrowable = true;
    assert!(validate_physical_positions(&f.encounter).is_err());
    assert!(flight_loss_fall(&f.encounter, &f.state, f.a).is_err());
    assert!(drop_destination(&f.encounter, f.a).is_err());
    f.encounter.battlefield.terrain.clear();
    // Adjacent authored ground pieces together support the full footprint.
    f.terrain("earth-left", volume(point(10, 10, 0), point(15, 20, 90)))
        .burrowable = true;
    f.terrain("earth-right", volume(point(15, 10, 0), point(20, 20, 90)))
        .burrowable = true;
    validate_physical_positions(&f.encounter).unwrap();
    assert!(
        flight_loss_fall(&f.encounter, &f.state, f.a)
            .unwrap()
            .is_none()
    );
    assert_eq!(
        drop_destination(&f.encounter, f.a).unwrap(),
        point(10, 10, 80)
    );
    f.actor(f.a).movement.burrow = None;
    assert!(validate_physical_positions(&f.encounter).is_err());
    f.encounter.battlefield.terrain.clear();
    let fog = f.terrain("fog", volume(point(10, 10, 0), point(20, 20, 90)));
    fog.difficult = true;
    fog.obscuration = Obscuration::Heavy;
    validate_physical_positions(&f.encounter).unwrap();
    assert_eq!(
        flight_loss_fall(&f.encounter, &f.state, f.a)
            .unwrap()
            .unwrap()
            .to
            .z,
        0
    );
}

#[test]
fn walking_off_support_returns_only_the_prefix_before_falling_while_a_jump_continues() {
    let mut f = falling_fixture();
    f.wall(
        "platform",
        volume(point(0, 0, 0), point(20, 30, 80)),
        CoverDegree::Total,
        true,
    );
    let walk = f
        .path(
            &[
                (point(20, 10, 80), MovementMode::Walk),
                (point(30, 10, 80), MovementMode::Walk),
            ],
            MovementAllowance::default(),
        )
        .unwrap();
    assert_eq!(walk.segments.len(), 1);
    assert_eq!(walk.destination, point(20, 10, 80));
    assert!(walk.falls_at_end && walk.segments[0].falls_after);
    assert_eq!(walk.total_cost, 10);
    let jump = f
        .path(
            &[
                (point(20, 10, 80), MovementMode::Jump),
                (point(30, 10, 80), MovementMode::Jump),
            ],
            MovementAllowance {
                runup: 20,
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(jump.segments.len(), 2);
    assert!(!jump.segments[0].falls_after);
    assert!(jump.segments[1].falls_after);
    assert_eq!(jump.destination, point(30, 10, 80));
}
