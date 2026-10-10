//! Object-point geometry controls, not pickup actions or histories.
use super::*;

#[test]
fn point_reach_uses_real_footprint_vertical_cells_and_exact_endpoint() {
    let mut f = Fixture::new();
    // Medium footprint [10,20): horizontal occupied center15. Point25 is
    // exactly five feet away; no fabricated target cell expands the allowance.
    assert!(object_point_access(&f.encounter, &f.state, f.a, point(25, 15, 0)).unwrap());
    assert!(!object_point_access(&f.encounter, &f.state, f.a, point(26, 15, 0)).unwrap());
    assert!(object_point_access(&f.encounter, &f.state, f.a, point(15, 15, 17)).unwrap());
    assert!(!object_point_access(&f.encounter, &f.state, f.a, point(15, 15, 18)).unwrap());
    f.actor(f.a).size = CreatureSize::Large;
    assert!(object_point_access(&f.encounter, &f.state, f.a, point(35, 15, 0)).unwrap());
    assert!(!object_point_access(&f.encounter, &f.state, f.a, point(36, 15, 0)).unwrap());
    f.actor(f.a).size = CreatureSize::Tiny;
    // Tiny five-half-foot space follows integer occupied-center endpoints12..13.
    assert!(object_point_access(&f.encounter, &f.state, f.a, point(23, 12, 0)).unwrap());
    assert!(!object_point_access(&f.encounter, &f.state, f.a, point(24, 12, 0)).unwrap());
}

#[test]
fn object_knowledge_uses_current_senses_not_creature_tremorsense_or_host_knowledge() {
    let mut f = Fixture::new();
    let target = point(25, 15, 0);
    f.encounter.battlefield.ambient_light = LightLevel::Darkness;
    f.actor(f.a).senses.tremorsense = 120;
    assert!(!object_point_access(&f.encounter, &f.state, f.a, target).unwrap());
    f.actor(f.a).senses.darkvision = 120;
    assert!(object_point_access(&f.encounter, &f.state, f.a, target).unwrap());
    f.condition(f.a, Condition::Blinded);
    assert!(!object_point_access(&f.encounter, &f.state, f.a, target).unwrap());
    f.actor(f.a).senses.blindsight = 120;
    assert!(object_point_access(&f.encounter, &f.state, f.a, target).unwrap());
    f.condition(f.a, Condition::Unconscious);
    assert!(!object_point_access(&f.encounter, &f.state, f.a, target).unwrap());
}

#[test]
fn visible_point_behind_transparent_physical_barrier_is_not_reachable() {
    let mut f = Fixture::new();
    let target = point(25, 15, 0);
    f.wall(
        "transparent",
        volume(point(21, 0, -10), point(22, 100, 100)),
        CoverDegree::None,
        false,
    );
    assert!(geometry::clear_sight(&f.encounter.battlefield, point(15, 15, 6), target).unwrap());
    assert!(!object_point_access(&f.encounter, &f.state, f.a, target).unwrap());
}

#[test]
fn darkness_and_obscuration_are_queried_at_the_real_object_point() {
    let mut f = Fixture::new();
    let target = point(25, 15, 0);
    f.actor(f.a).senses.darkvision = 120;
    f.terrain("dark", volume(point(22, 10, -5), point(28, 20, 10)))
        .magical_darkness = true;
    assert!(!object_point_access(&f.encounter, &f.state, f.a, target).unwrap());
    f.actor(f.a).senses.truesight = 120;
    assert!(object_point_access(&f.encounter, &f.state, f.a, target).unwrap());
    f.actor(f.a).senses.truesight = 0;
    f.encounter.battlefield.terrain.clear();
    f.terrain("mist", volume(point(22, 10, -5), point(28, 20, 10)))
        .obscuration = Obscuration::Heavy;
    assert!(!object_point_access(&f.encounter, &f.state, f.a, target).unwrap());
}
