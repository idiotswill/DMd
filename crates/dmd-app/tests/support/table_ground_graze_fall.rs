//! The actual 114HP source flyer is depleted only by accepted weapon attacks.
use super::*;

async fn ledge(f: &mut Fixture, path: &Path) -> (EntityId, CreatureSourcePin) {
    let target = Box::pin(current_source(f, path, "chimera", CreatureSize::Large)).await;
    let setup = TableBattlefieldSetup {
        encounter_id: EncounterId::new(), scene_id: SceneId::new(), location_id: LocationId::new(),
        name: "Supported attacker beside a source flyer".into(), area_grid_policy: None,
        battlefield: Battlefield {
            bounds: SpatialBox { min: point(0, 0, 0), max: point(100, 100, 60) },
            floor_z: 0, floor_surface: "stone".into(), ambient_light: LightLevel::Bright,
            terrain: vec![], lights: vec![],
            obstacles: vec![SpatialObstacle {
                id: "supporting-ledge".into(),
                volume: SpatialBox { min: point(0, 0, 0), max: point(20, 30, 40) },
                blocks_movement: true, blocks_sight: true, cover: CoverDegree::Total, observable: true,
            }],
        },
        characters: vec![TableCharacterPlacement {
            character_id: f.characters[0], position: point(10, 10, 40), height: 12,
            allies: vec![], enemies: vec![target],
        }],
        creatures: vec![TableCreaturePlacement {
            actor: target, public_label: "Large flying creature".into(), position: point(20, 10, 40),
            height: 20, allies: vec![], enemies: vec![f.actors[0]],
        }],
        geometry_ruling: Ruling { basis: RulingBasis::GmAdjudication,
            reason: "Adjacent occupied cells at the solid ledge; edge contact does not support the flyer.".into() },
    };
    Box::pin(battlefield(f, path, setup)).await;
    let combatants = vec![
        TacticalCombatant {
            actor: f.actors[0],
            source: TacticalSource::Character,
            surprised: false,
        },
        TacticalCombatant {
            actor: target,
            source: TacticalSource::Creature {
                definition_id: "chimera".into(),
            },
            surprised: false,
        },
    ];
    Box::pin(begin(f, path, combatants)).await;
    Box::pin(player_raw(f, path, 18)).await;
    Box::pin(raw(f, path, TableTransportChannel::Host, 2)).await;
    Box::pin(activate(f, path)).await;
    let ready = state(f).await;
    let rules = ready.rules.as_ref().unwrap();
    assert_eq!(rules.entities[&target].hp, 114);
    assert_eq!(rules.entities[&target].max_hp, 114);
    assert!(!rules.entities[&target].uses_death_saves);
    let pin = rules
        .tactical_creatures
        .as_ref()
        .unwrap()
        .profile(target)
        .unwrap()
        .source
        .clone();
    let encounter = ready.encounter.as_ref().unwrap();
    dmd_rules::spatial::validate_encounter(encounter, &ready).unwrap();
    let attacker = encounter.participant(f.actors[0]).unwrap();
    let flyer = encounter.participant(target).unwrap();
    assert_eq!(
        dmd_rules::spatial::participant_distance(attacker, flyer).unwrap(),
        10
    );
    assert_eq!(attacker.reach, 10);
    assert!(
        dmd_rules::spatial::perceive(encounter, &ready, f.actors[0], target)
            .unwrap()
            .precisely_located
    );
    assert!(
        dmd_rules::spatial::fall_destination(encounter, f.actors[0])
            .unwrap()
            .is_none()
    );
    let fall = dmd_rules::spatial::fall_destination(encounter, target)
        .unwrap()
        .unwrap();
    assert_eq!(fall.from, point(20, 10, 40));
    assert_eq!(fall.to, point(20, 10, 0));
    assert_eq!(fall.surface, FallSurface::Floor);
    assert_eq!(flyer.movement.fly, Some(120));
    assert!(!flyer.movement.hover);
    assert!(
        dmd_rules::spatial::flight_loss_fall(encounter, &ready, target)
            .unwrap()
            .is_none()
    );
    (target, pin)
}

async fn deplete(f: &mut Fixture, path: &Path, target: EntityId, item: ItemId) {
    // Four criticals (4d6 at6 +Strength3), then one ordinary hit (2d6 at1 +3).
    // The very first accepted hit owns before Equip. Every later hit is held.
    let mut expected_hp = 114;
    for index in 0..5 {
        let critical = index < 4;
        let before = state(f).await;
        let rules = before.rules.as_ref().unwrap();
        assert_eq!(rules.entities[&target].hp, expected_hp);
        assert!(!rules.entities[&target].death.dead);
        let timing = rules.timing.as_ref().unwrap();
        assert_eq!(timing.order[timing.index].actor, f.actors[0]);
        let attack = Box::pin(player_step(
            f,
            path,
            action(TacticalAction::Attack {
                choice: weapon_choice(item, target, index == 0, false),
            }),
        ))
        .await;
        let input = physical_input(view(f, &player(f)).await, &[if critical { 20 } else { 19 }]);
        let hit = Box::pin(player_step(f, path, input)).await;
        Box::pin(no_after_card(f)).await;
        Box::pin(decline_hit_cold(
            f,
            path,
            target,
            TableTransportChannel::Host,
        ))
        .await;
        let shown = view(f, &player(f)).await;
        let pending = shown.roll.as_ref().unwrap();
        assert_eq!(pending.mode, RollMode::Normal);
        assert_eq!(
            pending.dice,
            vec![DieSpec {
                count: if critical { 4 } else { 2 },
                sides: 6
            }]
        );
        assert_eq!(pending.modifier, 0);
        assert_eq!(pending.roller, Some(f.actors[0]));
        let damage_wait = state(f).await;
        let damage_rules = damage_wait.rules.as_ref().unwrap();
        let raw_pending = damage_rules.pending.as_ref().unwrap();
        let damage_resolution = resolution(&damage_wait);
        let damage_work = damage_resolution.pending.as_ref().unwrap();
        let active_attack = damage_resolution.attack.as_ref().unwrap();
        assert_eq!(damage_resolution.origin.id, attack.command_id);
        assert_eq!(active_attack.origin.id, attack.command_id);
        assert_eq!(active_attack.actor, f.actors[0]);
        assert_eq!(active_attack.target, target);
        assert_eq!(active_attack.stage, TacticalAttackStage::DamageRoll);
        assert_eq!(
            active_attack.outcome,
            Some(WeaponAttackOutcome::Hit {
                critical,
                damage_dealt: 0,
            })
        );
        assert_eq!(
            active_attack.damage,
            vec![AttackDamageComponent {
                damage_type: DamageType::Slashing,
                dice: vec![DieSpec { count: 2, sides: 6 }],
                modifier: 3,
            }]
        );
        assert_eq!(active_attack.weapon().unwrap().choice.weapon, item);
        assert_eq!(damage_work.work.kind, TacticalWorkKind::AttackDamage);
        assert_eq!(damage_work.key.origin, attack.command_id);
        assert_eq!(damage_work.key.subject, target);
        assert_eq!(damage_work.key.role, TacticalRollRole::AttackDamage);
        assert_eq!(damage_work.key.occurrence, damage_work.work.occurrence);
        assert_eq!(raw_pending.request.id, damage_work.key.request_id());
        assert_eq!(raw_pending.request.roller, Some(f.actors[0]));
        assert_eq!(raw_pending.request.dice, pending.dice);
        assert_eq!(raw_pending.request.modifier, 0);
        assert_eq!(raw_pending.request.mode, RollMode::Normal);
        assert_eq!(
            raw_pending.purpose,
            PendingPurpose::TacticalResolution {
                encounter: damage_wait.encounter.as_ref().unwrap().id,
                key: damage_work.key,
            }
        );
        assert_eq!(
            damage_resolution
                .work_trace
                .as_ref()
                .unwrap()
                .nodes
                .iter()
                .filter(|node| node.work == damage_work.work)
                .count(),
            1
        );
        assert_eq!(
            state(f)
                .await
                .rules
                .as_ref()
                .unwrap()
                .pending
                .as_ref()
                .unwrap()
                .issued_by
                .id,
            hit.command_id
        );
        let input = physical_input(shown, if critical { &[6, 6, 6, 6] } else { &[1, 1] });
        let damage_input = Box::pin(player_step(f, path, input)).await;
        let damage = if critical { 27 } else { 5 };
        expected_hp -= damage;
        let after = state(f).await;
        let rules = after.rules.as_ref().unwrap();
        assert_eq!(rules.entities[&target].hp, expected_hp);
        assert!(!rules.entities[&target].death.dead);
        assert_eq!(
            rules.rolls.len(),
            before.rules.as_ref().unwrap().rolls.len() + 2
        );
        assert_eq!(
            &rules.rolls[..damage_rules.rolls.len()],
            damage_rules.rolls.as_slice()
        );
        let damage_raw = rules.rolls.last().unwrap();
        assert_eq!(damage_raw.request, raw_pending.request);
        assert_eq!(damage_raw.accepted_by.id, damage_input.command_id);
        assert_eq!(damage_raw.result.source, RollSource::Physical);
        assert_eq!(
            damage_raw.result.dice,
            vec![
                DieResult {
                    sides: 6,
                    value: if critical { 6 } else { 1 },
                };
                if critical { 4 } else { 2 }
            ]
        );
        assert_eq!(damage_raw.resolved.total, if critical { 24 } else { 2 });
        assert_eq!(after.items, before.items);
        assert_eq!(
            hands(&after, f.actors[0]).hands,
            [HandAssignment::Item(item); 2]
        );
        let encounter = after.encounter.as_ref().unwrap();
        assert_eq!(
            encounter.participant(target).unwrap().position,
            point(20, 10, 40)
        );
        let flow = encounter.flow.as_ref().unwrap();
        assert!(flow.resolution.is_none());
        let receipt = flow
            .budget
            .weapon_history
            .iter()
            .find(|row| row.origin.id == attack.command_id)
            .unwrap();
        assert_eq!(
            receipt.outcome,
            WeaponAttackOutcome::Hit {
                critical,
                damage_dealt: damage
            }
        );
        assert!(receipt.after_equipment.is_none());
        Box::pin(next_player_turn(f, path)).await;
    }
    assert_eq!(expected_hp, 1);
    assert_eq!(state(f).await.rules.unwrap().entities[&target].hp, 1);
}

pub(super) async fn run() {
    let (mut f, directory, path, item) = Box::pin(current_fixture("greatsword")).await;
    let (target, source) = Box::pin(ledge(&mut f, &path)).await;
    Box::pin(deplete(&mut f, &path, target, item)).await;
    let cut = Box::pin(start_graze(&mut f, &path, item, target, 1)).await;
    assert_eq!(cut.paused.rules.as_ref().unwrap().entities[&target].hp, 1);
    let mastery = Box::pin(choose_graze(&mut f, &path, true)).await;
    let landed = state(&f).await;
    let rules = landed.rules.as_ref().unwrap();
    assert_eq!(rules.entities[&target].hp, 0);
    assert!(rules.entities[&target].death.dead);
    assert_eq!(landed.entities[&target].existence, EntityExistence::Dead);
    assert_eq!(
        rules.rolls,
        cut.paused.rules.as_ref().unwrap().rolls,
        "Graze and an already-dead falling body must not invent damage dice"
    );
    assert!(rules.pending.is_none());
    assert_eq!(landed.items, cut.paused.items);
    assert_eq!(
        rules
            .tactical_creatures
            .as_ref()
            .unwrap()
            .profile(target)
            .unwrap()
            .source,
        source
    );
    let encounter = landed.encounter.as_ref().unwrap();
    assert_eq!(
        encounter.participant(target).unwrap().position,
        point(20, 10, 0)
    );
    assert_eq!(
        encounter.participant(f.actors[0]).unwrap().position,
        point(10, 10, 40)
    );
    let r = resolution(&landed);
    assert!(r.attack.is_none());
    assert!(r.pending.is_none());
    assert!(r.failed_save.is_none());
    assert_eq!(r.falls.len(), 1);
    let after = r.attack_after_equipment.as_ref().unwrap();
    assert_eq!(after.cause.origin.id, cut.attack.command_id);
    assert_eq!(after.cause.completed_by.id, mastery.command_id);
    assert_eq!(after.cause.completed_work, cut.parent.work);
    assert_eq!(after.cause.choice.target, target);
    assert_eq!(after.selected_by.as_ref().unwrap().id, mastery.command_id);
    let fall = &r.falls[0];
    assert_eq!(fall.actor, target);
    assert_eq!(fall.origin.id, mastery.command_id);
    assert_eq!(fall.cause, TacticalFallCause::FlightLost);
    assert_eq!(fall.path.from, point(20, 10, 40));
    assert_eq!(fall.path.to, point(20, 10, 0));
    assert!(
        matches!(&fall.stage, TacticalFallStage::Complete { landing: None, damage: None, resolved_by }
        if resolved_by.id == mastery.command_id)
    );
    let trace = r.work_trace.as_ref().unwrap();
    assert!(trace.active.is_none());
    let begin = trace
        .nodes
        .iter()
        .find(|node| node.work.kind == TacticalWorkKind::BeginFall { fall: 0 })
        .unwrap();
    assert_eq!(begin.parent, Some(cut.parent.work.occurrence));
    assert!(begin.work.occurrence > after.work.occurrence);
    let damage = trace
        .nodes
        .iter()
        .find(|node| node.work.kind == TacticalWorkKind::FallDamage { fall: 0 })
        .unwrap();
    assert_eq!(damage.parent, Some(begin.work.occurrence));
    assert!(damage.work.occurrence > begin.work.occurrence);
    for channel in [player(&f), other_player(&f), TableTransportChannel::Host] {
        assert!(view(&f, &channel).await.roll.is_none());
    }
    Box::pin(finish_equipment(&mut f, &path, item, &cut, false, true)).await;
    Box::pin(close_fixture(f, &directory)).await;
}
