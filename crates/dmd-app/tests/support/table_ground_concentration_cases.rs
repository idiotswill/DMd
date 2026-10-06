//! Real Hold Person concentration children precede the attack equipment decision.
use super::*;

fn point(x: i32, y: i32) -> SpatialPoint {
    SpatialPoint { x, y, z: 0 }
}
fn second_player(f: &Fixture) -> TableTransportChannel {
    TableTransportChannel::Player {
        player_id: f.players[1],
        character_id: f.characters[1],
    }
}
async fn prepare(f: &mut Fixture, path: &Path) -> EntityId {
    f.host(TableAction::EndSession, Some(f.session)).await;
    f.session = PlaySessionId::new();
    f.host(
        TableAction::StartSession {
            id: f.session,
            name: "Both travelers present".into(),
            participants: (0..2)
                .map(|index| SessionParticipant {
                    player_id: f.players[index],
                    character_id: Some(f.characters[index]),
                    attendance: AttendanceStatus::Present,
                })
                .collect(),
        },
        Some(f.session),
    )
    .await;
    let cultist = EntityId::new();
    let allocation =
        dmd_rules::tactical_creature_equipment::creature_equipment_plan("cultist-fanatic", 0)
            .unwrap();
    f.host(
        TableAction::CreateCreature {
            creation: Box::new(TableCreatureCreation {
                entity_id: cultist,
                name: "Private concentration source".into(),
                definition_id: "cultist-fanatic".into(),
                source: Some(
                    dmd_rules::tactical_creatures::creature_source_pin(
                        dmd_rules::tactical_creatures::creature_definition("cultist-fanatic")
                            .unwrap(),
                    )
                    .unwrap(),
                ),
                size: CreatureSize::Medium,
                additional_languages: vec![],
                ammunition_units: 0,
                item_ids: allocation.iter().map(|_| ItemId::new()).collect(),
            }),
        },
        Some(f.session),
    )
    .await;
    for character_id in f.characters {
        let view = f
            .runtime
            .table_view(f.campaign, TableViewer::Host)
            .await
            .unwrap();
        let count = view
            .characters
            .iter()
            .find(|c| c.character_id == character_id)
            .unwrap()
            .equipment
            .as_ref()
            .unwrap()
            .initial_item_count;
        f.host(
            TableAction::PrepareEquipment {
                character_id,
                item_ids: (0..count).map(|_| ItemId::new()).collect(),
            },
            Some(f.session),
        )
        .await;
    }
    f.host(
        TableAction::PrepareBattlefield {
            setup: Box::new(TableBattlefieldSetup {
                encounter_id: EncounterId::new(),
                scene_id: SceneId::new(),
                location_id: LocationId::new(),
                name: "Crossing the courtyard".into(),
                area_grid_policy: None,
                battlefield: Battlefield {
                    bounds: SpatialBox {
                        min: point(0, 0),
                        max: SpatialPoint {
                            x: 100,
                            y: 100,
                            z: 40,
                        },
                    },
                    floor_z: 0,
                    floor_surface: "stone".into(),
                    ambient_light: LightLevel::Bright,
                    terrain: vec![],
                    obstacles: vec![],
                    lights: vec![],
                },
                characters: vec![
                    TableCharacterPlacement {
                        character_id: f.characters[0],
                        position: point(10, 10),
                        height: 12,
                        allies: vec![f.actors[1]],
                        enemies: vec![cultist],
                    },
                    TableCharacterPlacement {
                        character_id: f.characters[1],
                        position: point(10, 40),
                        height: 12,
                        allies: vec![f.actors[0]],
                        enemies: vec![cultist],
                    },
                ],
                creatures: vec![TableCreaturePlacement {
                    actor: cultist,
                    public_label: "Robed traveler".into(),
                    position: point(20, 10),
                    height: 12,
                    allies: vec![],
                    enemies: f.actors.to_vec(),
                }],
                geometry_ruling: Ruling {
                    basis: RulingBasis::GmAdjudication,
                    reason: "Host established a level, illuminated courtyard and known opposition."
                        .into(),
                },
            }),
        },
        Some(f.session),
    )
    .await;
    let combatants = vec![
        TacticalCombatant {
            actor: f.actors[0],
            source: TacticalSource::Character,
            surprised: false,
        },
        TacticalCombatant {
            actor: cultist,
            source: TacticalSource::Creature {
                definition_id: "cultist-fanatic".into(),
            },
            surprised: false,
        },
        TacticalCombatant {
            actor: f.actors[1],
            source: TacticalSource::Character,
            surprised: false,
        },
    ];
    let groups = combatants
        .iter()
        .map(|c| InitiativeGroup {
            actors: vec![c.actor],
            request_id: RollRequestId::new(),
        })
        .collect();
    Box::pin(step(
        f,
        path,
        TableTransportChannel::Host,
        action(TacticalAction::Begin {
            execution: TacticalExecutionVersion::EncounterReleaseV1,
            combatants,
            groups,
        }),
    ))
    .await;
    Box::pin(player_raw(f, path, 18)).await;
    Box::pin(raw(f, path, TableTransportChannel::Host, 10)).await;
    let second = second_player(f);
    Box::pin(raw(f, path, second, 1)).await;
    Box::pin(step(
        f,
        path,
        TableTransportChannel::Host,
        action(TacticalAction::ActivateAttackEquipment),
    ))
    .await;
    cultist
}

async fn concentrate(f: &mut Fixture, path: &Path, cultist: EntityId) -> (ItemId, EffectId) {
    let ready = state(f).await;
    let item = ready
        .items
        .values()
        .find(|i| i.custody == Custody::Entity(f.actors[0]) && i.definition_id == "dagger")
        .unwrap()
        .id;
    let mut choice = throw_choice(item, cultist);
    choice.delivery = WeaponDelivery::Melee;
    choice.equipment_change = Some(AttackEquipmentChange {
        timing: EquipmentChangeTiming::BeforeAttack,
        operation: AttackEquipmentOperation::Equip {
            item,
            hand: Hand::Right,
        },
    });
    Box::pin(player_step(
        f,
        path,
        action(TacticalAction::Attack { choice }),
    ))
    .await;
    Box::pin(player_raw(f, path, 1)).await;
    Box::pin(player_step(f, path, action(TacticalAction::EndTurn))).await;
    let variant = view(f, &TableTransportChannel::Host)
        .await
        .tactical
        .unwrap()
        .casting_options
        .unwrap()
        .variants
        .into_iter()
        .find(|v| v.choice.spell_id == "hold-person")
        .unwrap();
    let SpellMaterialChoice::Material { item: material } = variant.choice.material else {
        panic!("actual source material required")
    };
    assert_eq!(
        state(f).await.items[&material].custody,
        Custody::Entity(cultist)
    );
    let target = f.actors[1];
    let cast = Box::pin(step(
        f,
        path,
        TableTransportChannel::Host,
        action(TacticalAction::CastSpell {
            choice: variant.choice,
            targets: SpellTargetChoice::Entities(vec![target]),
        }),
    ))
    .await;
    let second = second_player(f);
    Box::pin(raw(f, path, second.clone(), 1)).await;
    let focused = state(f).await;
    let group = focused
        .rules
        .as_ref()
        .unwrap()
        .tactical_effects
        .as_ref()
        .unwrap()
        .group_for_owner(cultist)
        .unwrap();
    assert_eq!(group.source.command.id, cast.command_id);
    let group_id = group.id;
    assert!(
        dmd_rules::active_conditions(focused.rules.as_ref().unwrap(), f.actors[1])
            .contains(&Condition::Paralyzed)
    );
    assert!(
        focused
            .rules
            .as_ref()
            .unwrap()
            .timing
            .as_ref()
            .unwrap()
            .action_spent
    );
    Box::pin(step(
        f,
        path,
        TableTransportChannel::Host,
        action(TacticalAction::EndTurn),
    ))
    .await;
    Box::pin(step(
        f,
        path,
        second.clone(),
        action(TacticalAction::EndTurn),
    ))
    .await;
    // The held creature's real end-turn save also fails before the attacker acts.
    Box::pin(raw(f, path, second, 1)).await;
    let next = state(f).await;
    let timing = next.rules.as_ref().unwrap().timing.as_ref().unwrap();
    assert_eq!(timing.order[timing.index].actor, f.actors[0]);
    assert_eq!(
        next.rules
            .as_ref()
            .unwrap()
            .tactical_effects
            .as_ref()
            .unwrap()
            .group_for_owner(cultist)
            .unwrap()
            .id,
        group_id
    );
    (item, group_id)
}

async fn run_case(f: &mut Fixture, path: &Path, succeeds: bool) {
    let cultist = Box::pin(prepare(f, path)).await;
    let (item, group) = Box::pin(concentrate(f, path, cultist)).await;
    let initial = state(f).await;
    let hp = initial.rules.as_ref().unwrap().entities[&cultist].hp;
    let mut choice = throw_choice(item, cultist);
    choice.delivery = WeaponDelivery::Melee;
    choice.after_equipment = Some(AfterAttackEquipmentIntent::Choose);
    let attack = Box::pin(player_step(
        f,
        path,
        action(TacticalAction::Attack { choice }),
    ))
    .await;
    Box::pin(player_raw(f, path, 19)).await;
    Box::pin(decline_hit_cold(
        f,
        path,
        cultist,
        TableTransportChannel::Host,
    ))
    .await;
    let pending_damage = view(f, &player(f)).await.roll.unwrap();
    assert_eq!(pending_damage.dice, vec![DieSpec { count: 1, sides: 4 }]);
    Box::pin(no_after_card(f)).await;
    let input = physical_input(view(f, &player(f)).await, &[2]);
    let damage = Box::pin(player_step(f, path, input)).await;
    let queued = state(f).await;
    assert_eq!(queued.rules.as_ref().unwrap().entities[&cultist].hp, hp - 5);
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
    assert_eq!(after.cause.origin.id, attack.command_id);
    assert_eq!(after.cause.completed_by.id, damage.command_id);
    let pending = resolution.pending.as_ref().unwrap();
    assert_eq!(pending.key.role, TacticalRollRole::Concentration);
    assert!(
        matches!(pending.work.kind, TacticalWorkKind::ConcentrationSave { actor, group: actual, .. } if actor == cultist && actual == group)
    );
    let trace = resolution.work_trace.as_ref().unwrap();
    let node = trace
        .nodes
        .iter()
        .find(|node| node.work == pending.work)
        .unwrap();
    assert_eq!(node.parent, Some(after.cause.completed_work.occurrence));
    assert_eq!(
        queued
            .rules
            .as_ref()
            .unwrap()
            .pending
            .as_ref()
            .unwrap()
            .issued_by
            .id,
        damage.command_id
    );
    assert!(view(f, &player(f)).await.roll.is_none());
    assert!(view(f, &second_player(f)).await.roll.is_none());
    Box::pin(premature_after(f)).await;
    let save = view(f, &TableTransportChannel::Host).await.roll.unwrap();
    assert_eq!(save.roller, Some(cultist));
    assert_eq!(save.mode, RollMode::Normal);
    assert_eq!(
        save.dice,
        vec![DieSpec {
            count: 1,
            sides: 20
        }]
    );
    Box::pin(raw(
        f,
        path,
        TableTransportChannel::Host,
        if succeeds { 20 } else { 1 },
    ))
    .await;
    let finished_child = state(f).await;
    let rules = finished_child.rules.as_ref().unwrap();
    assert_eq!(rules.entities[&cultist].concentration.is_some(), succeeds);
    assert_eq!(
        rules
            .tactical_effects
            .as_ref()
            .unwrap()
            .group_for_owner(cultist)
            .is_some(),
        succeeds
    );
    assert_eq!(
        dmd_rules::active_conditions(rules, f.actors[1]).contains(&Condition::Paralyzed),
        succeeds
    );
    let selected_resolution = finished_child
        .encounter
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .resolution
        .as_ref()
        .unwrap();
    assert!(selected_resolution.pending.is_none());
    // The genuine Cultist has no Legendary Resistance; failure finishes in this
    // same accepted save command, without an invented failed-save decision.
    assert!(selected_resolution.failed_save.is_none());
    assert!(
        selected_resolution
            .attack_after_equipment
            .as_ref()
            .unwrap()
            .selected_by
            .is_some()
    );
    let shown = view(f, &player(f))
        .await
        .tactical
        .unwrap()
        .attack_equipment
        .unwrap();
    Box::pin(hostile_populated_destination(f)).await;
    let operation = AttackEquipmentOperation::Unequip { item };
    assert!(
        shown
            .operations
            .iter()
            .any(|candidate| candidate.operation == operation)
    );
    Box::pin(player_step(
        f,
        path,
        TableTransportInput::AttackEquipment {
            handle: shown.key,
            choice: if succeeds {
                AttackEquipmentChoice::Apply(operation)
            } else {
                AttackEquipmentChoice::Decline
            },
        },
    ))
    .await;
    let final_state = state(f).await;
    assert!(
        final_state
            .encounter
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .resolution
            .is_none()
    );
    assert_eq!(final_state.items, finished_child.items);
    assert_eq!(final_state.rules.as_ref().unwrap().rolls, rules.rolls);
    assert_eq!(final_state.rules.as_ref().unwrap().timing, rules.timing);
}

#[tokio::test]
async fn real_hold_person_concentration_success_and_failure_finish_before_cold_after_equipment() {
    for succeeds in [false, true] {
        let directory =
            std::env::temp_dir().join(format!("dmd-ground-concentration-{}", CampaignId::new().0));
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
        Box::pin(run_case(&mut f, &path, succeeds)).await;
        f.pool.close().await;
        drop(f);
        sqlite_test_cleanup::remove_closed_directory(&directory)
            .await
            .unwrap();
    }
}
