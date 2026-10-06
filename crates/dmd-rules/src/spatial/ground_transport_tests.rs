//! Pure geometry/cost controls. These do not claim campaign admission authority.
use super::*;

fn pair() -> Fixture {
    let mut f = Fixture::new();
    f.actor(f.b).position = point(20, 10, 0);
    f
}
fn drag(f: &Fixture, destination: SpatialPoint, mode: MovementMode, allowance: MovementAllowance) -> Result<CoupledGroundSegment, SpatialError> {
    evaluate_coupled_ground(CoupledGroundQuery { encounter: &f.encounter, state: &f.state,
        holder: f.a, target: f.b, step: &TacticalMoveStep { destination, mode },
        allowance: &allowance, progress: &TacticalMovementProgress::default(), ends_move: true })
}
#[test]
fn support_union_rejects_endpoint_ledges_and_a_single_open_gap_in_both_directions() {
    let a = point(10, 10, 20);
    let b = point(20, 10, 20);
    let ledges = [volume(point(0, 10, 0), point(11, 20, 20)), volume(point(29, 10, 0), point(40, 20, 20))];
    assert!(!geometry::continuous_horizontal_support(a,b,10,&ledges));
    assert!(!geometry::continuous_horizontal_support(b,a,10,&ledges));
    let singleton = [volume(point(0, 10, 0), point(15, 20, 20)), volume(point(25, 10, 0), point(40, 20, 20))];
    assert!(!geometry::continuous_horizontal_support(a,b,10,&singleton));
    let connected = [singleton[0], volume(point(24, 10, 0), point(40, 20, 20))];
    assert!(geometry::continuous_horizontal_support(a,b,10,&connected));
    assert!(geometry::continuous_horizontal_support(b,a,10,&connected));
    assert!(!geometry::continuous_horizontal_support(a,b,10,&[volume(point(0, 20, 0),point(50,30,20))]));
}
#[test]
fn ground_support_checks_swept_water_before_floor_and_allows_a_dry_bridge() {
    let mut f = pair();
    f.terrain("water",volume(point(25,10,-10),point(26,20,3))).water=true;
    assert!(drag(&f,point(20,10,0),MovementMode::Walk,MovementAllowance::default()).is_err());
    f.encounter.battlefield.terrain[0].volume=volume(point(0,0,0),point(100,100,10));
    f.actor(f.a).position.z=20;
    f.actor(f.b).position.z=20;
    f.terrain("dry bridge",volume(point(0,0,10),point(80,30,20))).supports_top=true;
    let result=drag(&f,point(20,10,20),MovementMode::Walk,MovementAllowance::default()).unwrap();
    assert_eq!(result.holder.cost,20);
    assert_eq!(result.target.to,point(30,10,20));
}
#[test]
fn ground_drag_moves_jointly_through_each_others_vacated_cells() {
    let f=pair();
    let result=drag(&f,point(20,10,0),MovementMode::Walk,MovementAllowance::default()).unwrap();
    assert_eq!(result.holder.from,point(10,10,0));
    assert_eq!(result.holder.to,point(20,10,0));
    assert_eq!(result.target.from,point(20,10,0));
    assert_eq!(result.target.to,point(30,10,0));
    assert_eq!((result.ordinary_cost,result.haul_cost,result.holder.cost),(10,10,20));
}
#[test]
fn ground_drag_keeps_third_party_occupancy_and_target_solid_sweep() {
    let mut f=pair();
    f.actor(f.c).position=point(30,10,0);
    assert!(drag(&f,point(20,10,0),MovementMode::Walk,MovementAllowance::default()).is_err());
    f.actor(f.c).position=point(70,70,0);
    f.wall("target-only wall",volume(point(30,10,0),point(31,20,12)),CoverDegree::Total,false);
    assert!(drag(&f,point(20,10,0),MovementMode::Walk,MovementAllowance::default()).is_err());
}
#[test]
fn crawl_difficult_ground_and_haul_are_additive_without_target_prone_surcharge() {
    let mut f=pair();
    f.condition(f.a,Condition::Prone);
    f.condition(f.b,Condition::Prone);
    f.terrain("rubble",volume(point(0,0,0),point(70,30,3))).difficult=true;
    let result=drag(&f,point(20,10,0),MovementMode::Crawl,MovementAllowance::default()).unwrap();
    assert_eq!((result.ordinary_cost,result.haul_cost,result.holder.cost),(30,10,40));
    let mut allowance=MovementAllowance {spent:30,..MovementAllowance::default()};
    assert!(drag(&f,point(20,10,0),MovementMode::Crawl,allowance.clone()).is_err());
    allowance.dash.speed=1;
    assert_eq!(drag(&f,point(20,10,0),MovementMode::Crawl,allowance).unwrap().holder.cost,40);
}
#[test]
fn haul_size_exemption_uses_tiny_or_two_categories_not_mass() {
    let mut f=pair();
    f.actor(f.b).size=CreatureSize::Tiny;
    assert_eq!(drag(&f,point(20,10,0),MovementMode::Walk,MovementAllowance::default()).unwrap().haul_cost,0);
    f.actor(f.b).size=CreatureSize::Small;
    f.actor(f.a).size=CreatureSize::Large;
    f.actor(f.a).position=point(20,10,0);
    f.actor(f.b).position=point(10,10,0);
    assert_eq!(drag(&f,point(30,10,0),MovementMode::Walk,MovementAllowance::default()).unwrap().haul_cost,0);
    f.actor(f.b).size=CreatureSize::Medium;
    assert_eq!(drag(&f,point(30,10,0),MovementMode::Walk,MovementAllowance::default()).unwrap().haul_cost,10);
}
#[test]
fn selected_enemy_moves_in_the_after_pose_and_does_not_invent_an_opportunity() {
    let mut f=pair();
    f.actor(f.b).enemies=vec![f.a];
    let result=drag(&f,point(0,10,0),MovementMode::Walk,MovementAllowance::default()).unwrap();
    assert!(result.holder.opportunities.iter().all(|crossing| crossing.actor!=f.b));
    assert_eq!(result.target.to,point(10,10,0));
}
#[test]
fn unsupported_airborne_vertical_and_dead_target_drag_refuse_before_motion() {
    let mut f=pair();
    assert!(drag(&f,point(20,10,10),MovementMode::Walk,MovementAllowance::default()).is_err());
    assert!(drag(&f,point(20,10,0),MovementMode::Fly,MovementAllowance::default()).is_err());
    f.actor(f.b).position.z=10;
    assert!(drag(&f,point(20,10,0),MovementMode::Walk,MovementAllowance::default()).is_err());
    f.actor(f.b).position.z=0;
    f.state.rules.as_mut().unwrap().entities.get_mut(&f.b).unwrap().death.dead=true;
    assert!(drag(&f,point(20,10,0),MovementMode::Walk,MovementAllowance::default()).is_err());
}
