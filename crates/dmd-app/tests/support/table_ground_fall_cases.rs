//! Genuine source HP depletion and knockout cause a fall before retained equipment.
use super::*;

async fn prepare(f: &mut Fixture, path: &Path) -> (EntityId, ItemId, CreatureSourcePin) {
    let host = view(f, &TableTransportChannel::Host).await;
    let catalog = f
        .runtime
        .table_creature_options(TableCreatureOptionsRequest {
            campaign_id: f.campaign,
            channel: TableTransportChannel::Host,
            revision: host.revision,
        })
        .await
        .unwrap();
    let source = catalog
        .iter()
        .find(|source| source.definition_id == "chimera")
        .unwrap();
    let actor = EntityId::new();
    let pin = source.source.clone().unwrap();
    f.host(
        TableAction::CreateCreature {
            creation: Box::new(TableCreatureCreation {
                entity_id: actor,
                name: "Source flyer above the ledge".into(),
                definition_id: source.definition_id.clone(),
                source: Some(pin.clone()),
                size: CreatureSize::Large,
                additional_languages: vec![],
                ammunition_units: 0,
                item_ids: (0..source.item_count).map(|_| ItemId::new()).collect(),
            }),
        },
        Some(f.session),
    )
    .await;
    let count = host
        .characters
        .iter()
        .find(|c| c.character_id == f.characters[0])
        .unwrap()
        .equipment
        .as_ref()
        .unwrap()
        .initial_item_count;
    f.host(
        TableAction::PrepareEquipment {
            character_id: f.characters[0],
            item_ids: (0..count).map(|_| ItemId::new()).collect(),
        },
        Some(f.session),
    )
    .await;
    f.host(TableAction::PrepareBattlefield { setup: Box::new(TableBattlefieldSetup {
        encounter_id: EncounterId::new(),
        scene_id: SceneId::new(),
        location_id: LocationId::new(),
        name: "Supported ledge and open air".into(),
        battlefield: Battlefield {
            bounds: SpatialBox {
                min: SpatialPoint { x: 0, y: 0, z: 0 },
                max: SpatialPoint { x: 100, y: 100, z: 60 },
            },
            floor_z: 0,
            floor_surface: "stone".into(),
            ambient_light: LightLevel::Bright,
            terrain: vec![],
            obstacles: vec![SpatialObstacle {
                id: "supporting-ledge".into(),
                volume: SpatialBox {
                    min: SpatialPoint { x: 0, y: 0, z: 0 },
                    max: SpatialPoint { x: 20, y: 30, z: 40 },
                },
                blocks_movement: true,
                blocks_sight: true,
                cover: CoverDegree::Total,
                observable: true,
            }],
            lights: vec![],
        },
        characters: vec![TableCharacterPlacement {
            character_id: f.characters[0],
            position: SpatialPoint { x: 10, y: 10, z: 40 },
            height: 12,
            allies: vec![],
            enemies: vec![actor],
        }],
        creatures: vec![TableCreaturePlacement {
            actor,
            public_label: "Large flying creature".into(),
            position: SpatialPoint { x: 20, y: 10, z: 40 },
            height: 20,
            allies: vec![],
            enemies: vec![f.actors[0]],
        }],
        area_grid_policy: None,
        geometry_ruling: Ruling {
            basis: RulingBasis::GmAdjudication,
            reason: "Adjacent occupied cells at a solid ledge; edge contact alone does not support the flyer.".into(),
        },
    }) }, Some(f.session)).await;
    let actors = [f.actors[0], actor];
    Box::pin(step(
        f,
        path,
        TableTransportChannel::Host,
        action(TacticalAction::Begin {
            execution: TacticalExecutionVersion::EncounterReleaseV1,
            combatants: vec![
                TacticalCombatant {
                    actor: actors[0],
                    source: TacticalSource::Character,
                    surprised: false,
                },
                TacticalCombatant {
                    actor,
                    source: TacticalSource::Creature {
                        definition_id: "chimera".into(),
                    },
                    surprised: false,
                },
            ],
            groups: actors
                .into_iter()
                .map(|actor| InitiativeGroup {
                    actors: vec![actor],
                    request_id: RollRequestId::new(),
                })
                .collect(),
        }),
    ))
    .await;
    Box::pin(player_raw(f, path, 18)).await;
    Box::pin(raw(f, path, TableTransportChannel::Host, 2)).await;
    Box::pin(step(
        f,
        path,
        TableTransportChannel::Host,
        action(TacticalAction::ActivateAttackEquipment),
    ))
    .await;
    let ready = state(f).await;
    let rules = ready.rules.as_ref().unwrap();
    assert_eq!(rules.entities[&actor].hp, 114);
    assert_eq!(rules.entities[&actor].max_hp, 114);
    assert!(!rules.entities[&actor].uses_death_saves);
    assert_eq!(
        rules
            .tactical_creatures
            .as_ref()
            .unwrap()
            .profile(actor)
            .unwrap()
            .source,
        pin
    );
    let encounter = ready.encounter.as_ref().unwrap();
    dmd_rules::spatial::validate_encounter(encounter, &ready).unwrap();
    let attacker = encounter.participant(f.actors[0]).unwrap();
    let target = encounter.participant(actor).unwrap();
    assert_eq!(
        dmd_rules::spatial::participant_distance(attacker, target).unwrap(),
        10
    );
    assert_eq!(attacker.reach, 10);
    assert!(
        dmd_rules::spatial::perceive(encounter, &ready, f.actors[0], actor)
            .unwrap()
            .precisely_located
    );
    assert!(
        dmd_rules::spatial::fall_destination(encounter, f.actors[0])
            .unwrap()
            .is_none()
    );
    let fall = dmd_rules::spatial::fall_destination(encounter, actor)
        .unwrap()
        .unwrap();
    assert_eq!(
        fall.from,
        SpatialPoint {
            x: 20,
            y: 10,
            z: 40
        }
    );
    assert_eq!(fall.to, SpatialPoint { x: 20, y: 10, z: 0 });
    assert_eq!(fall.surface, FallSurface::Floor);
    assert_eq!(target.movement.fly, Some(120));
    assert!(!target.movement.hover);
    assert!(
        dmd_rules::spatial::flight_loss_fall(encounter, &ready, actor)
            .unwrap()
            .is_none()
    );
    let item = ready
        .items
        .values()
        .find(|i| i.custody == Custody::Entity(f.actors[0]) && i.definition_id == "dagger")
        .unwrap()
        .id;
    (actor, item, pin)
}

async fn run_case(f: &mut Fixture, path: &Path) {
    let (target, item, pin) = Box::pin(prepare(f, path)).await;
    let mut final_attack = None;
    let mut final_damage = None;
    for index in 0u32..11 {
        let before = state(f).await;
        let rules = before.rules.as_ref().unwrap();
        let timing = rules.timing.as_ref().unwrap();
        assert_eq!(timing.order[timing.index].actor, f.actors[0]);
        assert_eq!(rules.entities[&target].hp, 114 - index * 11);
        assert!(!rules.entities[&target].death.dead);
        let mut choice = throw_choice(item, target);
        choice.delivery = WeaponDelivery::Melee;
        choice.equipment_change = (index == 0).then_some(AttackEquipmentChange {
            timing: EquipmentChangeTiming::BeforeAttack,
            operation: AttackEquipmentOperation::Equip {
                item,
                hand: Hand::Right,
            },
        });
        choice.after_equipment = (index == 10).then_some(AfterAttackEquipmentIntent::Choose);
        let attack = Box::pin(player_step(
            f,
            path,
            action(TacticalAction::Attack { choice }),
        ))
        .await;
        let input = physical_input(view(f, &player(f)).await, &[20]);
        let hit = Box::pin(player_step(f, path, input)).await;
        Box::pin(no_after_card(f)).await;
        Box::pin(decline_hit_cold(
            f,
            path,
            target,
            TableTransportChannel::Host,
        ))
        .await;
        let pending = view(f, &player(f)).await.roll.unwrap();
        assert_eq!(pending.mode, RollMode::Normal);
        assert_eq!(pending.dice, vec![DieSpec { count: 2, sides: 4 }]);
        assert_eq!(pending.modifier, 0);
        {
            let damage_state = state(f).await;
            let resolution = damage_state
                .encounter
                .as_ref()
                .unwrap()
                .flow
                .as_ref()
                .unwrap()
                .resolution
                .as_ref()
                .unwrap();
            assert_eq!(
                resolution.attack.as_ref().unwrap().damage,
                vec![AttackDamageComponent {
                    damage_type: DamageType::Piercing,
                    dice: vec![DieSpec { count: 1, sides: 4 }],
                    modifier: 3,
                }]
            );
        }
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
        let input = physical_input(view(f, &player(f)).await, &[4, 4]);
        let damage = Box::pin(player_step(f, path, input)).await;
        if index < 10 {
            let completed = state(f).await;
            assert_eq!(
                completed.rules.as_ref().unwrap().entities[&target].hp,
                114 - (index + 1) * 11
            );
            let flow = completed.encounter.as_ref().unwrap().flow.as_ref().unwrap();
            assert!(flow.resolution.is_none());
            let receipt = flow
                .budget
                .weapon_history
                .iter()
                .find(|r| r.origin.id == attack.command_id)
                .unwrap();
            assert_eq!(
                receipt.outcome,
                WeaponAttackOutcome::Hit {
                    critical: true,
                    damage_dealt: 11
                }
            );
            assert!(receipt.after_equipment.is_none());
            assert_eq!(
                completed
                    .encounter
                    .as_ref()
                    .unwrap()
                    .participant(target)
                    .unwrap()
                    .position
                    .z,
                40
            );
            Box::pin(next_player_turn(f, path)).await;
        } else {
            final_attack = Some(attack);
            final_damage = Some(damage);
        }
    }
    let attack = final_attack.unwrap();
    let damage = final_damage.unwrap();
    let knockout_view = view(f, &player(f)).await.tactical.unwrap();
    assert_eq!(
        knockout_view.attack_decision.unwrap().kind,
        TableAttackDecisionKind::Knockout
    );
    assert!(knockout_view.attack_equipment.is_none());
    let at_knockout = state(f).await;
    assert_eq!(at_knockout.rules.as_ref().unwrap().entities[&target].hp, 4);
    let resolution = at_knockout
        .encounter
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .resolution
        .as_ref()
        .unwrap();
    let parent = resolution
        .attack
        .as_ref()
        .unwrap()
        .weapon()
        .unwrap()
        .after_equipment_parent
        .clone()
        .unwrap();
    assert_eq!(parent.pause, AttackEquipmentPause::Knockout);
    assert_eq!(parent.paused_by.id, damage.command_id);
    assert_eq!(
        parent.accepted_raw,
        resolution.attack.as_ref().unwrap().damage_roll
    );
    assert!(parent.accepted_raw.is_some());
    let knockout = Box::pin(player_step(
        f,
        path,
        action(TacticalAction::ChooseAttackKnockout {
            choice: KnockoutChoice::KnockOut,
        }),
    ))
    .await;
    let queued = state(f).await;
    let rules = queued.rules.as_ref().unwrap();
    assert_eq!(rules.entities[&target].hp, 1);
    assert!(!rules.entities[&target].death.dead);
    let conditions = dmd_rules::active_conditions(rules, target);
    assert!(conditions.contains(&Condition::Unconscious));
    assert!(conditions.contains(&Condition::Prone));
    let resolution = queued
        .encounter
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .resolution
        .as_ref()
        .unwrap();
    assert!(resolution.attack.is_none());
    let after = resolution.attack_after_equipment.as_ref().unwrap();
    assert!(after.selected_by.is_none());
    assert_eq!(after.cause.origin.id, attack.command_id);
    assert_eq!(after.cause.completed_by.id, knockout.command_id);
    assert_eq!(after.cause.completed_work, parent.work);
    assert_eq!(after.cause.actor, f.actors[0]);
    assert_eq!(after.cause.choice.target, target);
    let retained = after.cause.clone();
    let pending = resolution.pending.as_ref().unwrap();
    assert_eq!(pending.key.role, TacticalRollRole::FallDamage);
    assert_eq!(pending.key.origin, attack.command_id);
    assert_eq!(pending.key.subject, target);
    let TacticalWorkKind::FallDamage { fall: index } = pending.work.kind else {
        panic!("actual fall damage must precede equipment")
    };
    let fall = &resolution.falls[usize::from(index)];
    assert_eq!(fall.actor, target);
    assert_eq!(fall.origin.id, knockout.command_id);
    assert_eq!(fall.cause, TacticalFallCause::FlightLost);
    assert_eq!(
        fall.path.from,
        SpatialPoint {
            x: 20,
            y: 10,
            z: 40
        }
    );
    assert_eq!(fall.path.to, SpatialPoint { x: 20, y: 10, z: 0 });
    assert_eq!(fall.stage, TacticalFallStage::Damage { landing: None });
    let trace = resolution.work_trace.as_ref().unwrap();
    let begin = trace
        .nodes
        .iter()
        .find(|node| node.work.kind == TacticalWorkKind::BeginFall { fall: index })
        .unwrap();
    assert_eq!(begin.parent, Some(parent.work.occurrence));
    assert!(begin.work.occurrence > after.work.occurrence);
    let fall_node = trace
        .nodes
        .iter()
        .find(|node| node.work == pending.work)
        .unwrap();
    assert_eq!(fall_node.parent, Some(begin.work.occurrence));
    assert!(trace.active.is_none());
    assert!(
        resolution
            .frames
            .iter()
            .any(|frame| frame.contains(&after.work))
    );
    assert_eq!(
        rules.pending.as_ref().unwrap().issued_by.id,
        knockout.command_id
    );
    assert!(view(f, &player(f)).await.roll.is_none());
    let host_roll = view(f, &TableTransportChannel::Host).await.roll.unwrap();
    assert_eq!(host_roll.roller, Some(target));
    assert_eq!(host_roll.mode, RollMode::Normal);
    assert_eq!(host_roll.dice, vec![DieSpec { count: 2, sides: 6 }]);
    Box::pin(premature_after(f)).await;
    let input = physical_input(view(f, &TableTransportChannel::Host).await, &[1, 1]);
    let landing = Box::pin(step(f, path, TableTransportChannel::Host, input)).await;
    let landed = state(f).await;
    let landed_rules = landed.rules.as_ref().unwrap();
    assert_eq!(landed_rules.entities[&target].hp, 0);
    assert!(landed_rules.entities[&target].death.dead);
    assert_eq!(landed.entities[&target].existence, EntityExistence::Dead);
    let encounter = landed.encounter.as_ref().unwrap();
    assert_eq!(
        encounter.participant(target).unwrap().position,
        SpatialPoint { x: 20, y: 10, z: 0 }
    );
    assert_eq!(
        encounter.participant(f.actors[0]).unwrap().position,
        SpatialPoint {
            x: 10,
            y: 10,
            z: 40
        }
    );
    let resolution = encounter
        .flow
        .as_ref()
        .unwrap()
        .resolution
        .as_ref()
        .unwrap();
    assert!(resolution.pending.is_none());
    assert!(resolution.failed_save.is_none());
    assert!(matches!(&resolution.falls[usize::from(index)].stage,
        TacticalFallStage::Complete { damage: Some(key), resolved_by, .. }
        if *key == pending.key && resolved_by.id == landing.command_id));
    let selected = resolution.attack_after_equipment.as_ref().unwrap();
    assert_eq!(selected.cause, retained);
    assert_eq!(
        selected.selected_by.as_ref().unwrap().id,
        landing.command_id
    );
    let shown = view(f, &player(f))
        .await
        .tactical
        .unwrap()
        .attack_equipment
        .unwrap();
    assert_eq!(shown.actor, f.actors[0]);
    assert!(
        shown.may_decline,
        "target death must not erase the attacker's retained Decline"
    );
    assert!(
        shown
            .operations
            .iter()
            .any(|candidate| candidate.operation == AttackEquipmentOperation::Unequip { item })
    );
    Box::pin(hostile_populated_destination(f)).await;
    let decline = Box::pin(player_step(
        f,
        path,
        TableTransportInput::AttackEquipment {
            handle: shown.key,
            choice: AttackEquipmentChoice::Decline,
        },
    ))
    .await;
    let finished = state(f).await;
    let flow = finished.encounter.as_ref().unwrap().flow.as_ref().unwrap();
    assert!(flow.resolution.is_none());
    assert_eq!(finished.items, landed.items);
    let final_rules = finished.rules.as_ref().unwrap();
    assert_eq!(final_rules.rolls, landed_rules.rolls);
    assert_eq!(final_rules.timing, landed_rules.timing);
    assert_eq!(
        final_rules.entities[&target],
        landed_rules.entities[&target]
    );
    assert_eq!(
        final_rules
            .tactical_creatures
            .as_ref()
            .unwrap()
            .profile(target)
            .unwrap()
            .source,
        pin
    );
    let receipt = flow
        .budget
        .weapon_history
        .iter()
        .find(|r| r.origin.id == attack.command_id)
        .unwrap();
    let equipment = receipt.after_equipment.as_ref().unwrap();
    assert_eq!(equipment.cause, retained);
    assert_eq!(equipment.chosen_by.id, decline.command_id);
    assert_eq!(equipment.selected_by.id, landing.command_id);
    assert!(equipment.applied.is_none());
    let hands = view(f, &player(f))
        .await
        .tactical
        .unwrap()
        .attack_options
        .unwrap()
        .hands;
    assert!(hands.hands.contains(&HandAssignment::Item(item)));
}

#[tokio::test]
async fn genuine_chimera_knockout_fall_and_target_death_preserve_cold_after_equipment_decline() {
    let directory =
        std::env::temp_dir().join(format!("dmd-ground-source-fall-{}", CampaignId::new().0));
    std::fs::create_dir_all(&directory).unwrap();
    let path = directory.join("campaign.sqlite");
    let pool = dmd_persistence::open_sqlite_path(&path).await.unwrap();
    let mut creation = input("Equipment actor");
    creation.purchases.push(EquipmentChoice {
        item_id: "dagger".into(),
        quantity: 1,
    });
    let mut f = Box::pin(Fixture::with_creation_pool(
        TableContract::default(),
        Some(creation),
        pool,
    ))
    .await;
    Box::pin(run_case(&mut f, &path)).await;
    f.pool.close().await;
    drop(f);
    sqlite_test_cleanup::remove_closed_directory(&directory)
        .await
        .unwrap();
}
