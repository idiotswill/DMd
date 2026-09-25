use super::*;
use dmd_rules::tactical::TacticalAction;

fn point(x: i32, y: i32) -> SpatialPoint {
    SpatialPoint { x, y, z: 0 }
}
fn action(action: TacticalAction) -> TableAction {
    TableAction::Tactical { action }
}
fn content() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../content")
}
async fn state(f: &Fixture) -> CampaignState {
    f.runtime
        .open_campaign(f.campaign)
        .await
        .unwrap()
        .state()
        .clone()
}
async fn issue(f: &Fixture, player: Option<usize>, action: TacticalAction) -> CommandMeta {
    let meta = match player {
        Some(index) => f.player_meta(index).await,
        None => f.meta(CommandIssuer::Admin, None, Some(f.session)).await,
    };
    f.runtime
        .execute_table(meta.clone(), self::action(action))
        .await
        .unwrap();
    meta
}
async fn raw(f: &Fixture, player: Option<usize>, faces: &[u16]) -> (CommandMeta, TableAction) {
    let viewer = player.map_or(TableViewer::Host, |index| {
        TableViewer::Player(f.players[index])
    });
    let request = f
        .runtime
        .table_view(f.campaign, viewer)
        .await
        .unwrap()
        .roll
        .unwrap();
    let sides = if request.mode == RollMode::Normal {
        request
            .dice
            .iter()
            .flat_map(|d| std::iter::repeat_n(d.sides, usize::from(d.count)))
            .collect::<Vec<_>>()
    } else {
        vec![20, 20]
    };
    assert_eq!(sides.len(), faces.len());
    let meta = match player {
        Some(index) => f.player_meta(index).await,
        None => f.meta(CommandIssuer::Admin, None, Some(f.session)).await,
    };
    (
        meta,
        action(TacticalAction::SubmitRoll {
            result: RollResult {
                request_id: request.id,
                source: RollSource::Physical,
                dice: sides
                    .into_iter()
                    .zip(faces)
                    .map(|(sides, value)| DieResult {
                        sides,
                        value: *value,
                    })
                    .collect(),
            },
        }),
    )
}
async fn roll_setup(f: &Fixture, player: Option<usize>, faces: &[u16]) -> CommandMeta {
    let (meta, action) = raw(f, player, faces).await;
    f.runtime.execute_table(meta.clone(), action).await.unwrap();
    meta
}
async fn reopen(f: &mut Fixture, path: &Path) {
    f.pool.close().await;
    f.pool = dmd_persistence::open_sqlite_path(path).await.unwrap();
    f.runtime = CampaignRuntime::from_content_root(f.pool.clone(), content());
    f.runtime.resume_campaign(f.campaign).await.unwrap();
}
async fn normalized_export(f: &Fixture) -> dmd_persistence::CampaignExport {
    let mut export = export_campaign(&f.pool, f.campaign).await.unwrap();
    // The capture timestamp is not persisted campaign data. Every other field stays exact.
    export.exported_at_utc.clear();
    export
}
async fn rejected(f: &Fixture, meta: CommandMeta, action: TableAction) {
    let before = normalized_export(f).await;
    assert!(f.runtime.execute_table(meta, action).await.is_err());
    assert_eq!(normalized_export(f).await, before);
}
async fn both(f: &Fixture, mirror: &CampaignRuntime, meta: &CommandMeta, action: &TableAction) {
    assert!(
        !f.runtime
            .execute_table(meta.clone(), action.clone())
            .await
            .unwrap()
            .already_accepted
    );
    assert!(
        !mirror
            .execute_table(meta.clone(), action.clone())
            .await
            .unwrap()
            .already_accepted
    );
    assert_eq!(
        mirror.open_campaign(f.campaign).await.unwrap().state(),
        &state(f).await
    );
}
async fn cold_retry(f: &mut Fixture, path: &Path, meta: &CommandMeta, action: &TableAction) {
    let before = normalized_export(f).await;
    reopen(f, path).await;
    assert!(
        f.runtime
            .execute_table(meta.clone(), action.clone())
            .await
            .unwrap()
            .already_accepted
    );
    assert_eq!(normalized_export(f).await, before);
}

async fn prepare(f: &mut Fixture) -> EntityId {
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
    issue(
        f,
        None,
        TacticalAction::Begin {
            execution: TacticalExecutionVersion::ReactionsV1,
            combatants,
            groups,
        },
    )
    .await;
    roll_setup(f, Some(0), &[18]).await;
    roll_setup(f, None, &[10]).await;
    roll_setup(f, Some(1), &[1]).await;
    cultist
}

fn dagger_choice(weapon: ItemId, target: EntityId, equip: bool) -> WeaponUseChoice {
    WeaponUseChoice {
        weapon,
        target,
        delivery: WeaponDelivery::Melee,
        ability: Ability::Strength,
        grip: WeaponGrip::OneHand(Hand::Right),
        purpose: WeaponAttackPurpose::Normal,
        ammunition: None,
        equipment_change: equip.then_some(AttackEquipmentChange {
            timing: EquipmentChangeTiming::BeforeAttack,
            operation: AttackEquipmentOperation::Equip {
                item: weapon,
                hand: Hand::Right,
            },
        }),
    }
}
async fn arm_then_concentrate(f: &Fixture, cultist: EntityId) -> ItemId {
    let view = f
        .runtime
        .table_view(f.campaign, TableViewer::Player(f.players[0]))
        .await
        .unwrap();
    let weapon = view
        .tactical
        .unwrap()
        .attack_options
        .unwrap()
        .weapons
        .iter()
        .find(|w| w.name == "Dagger")
        .unwrap()
        .item;
    issue(
        f,
        Some(0),
        TacticalAction::Attack {
            choice: dagger_choice(weapon, cultist, true),
        },
    )
    .await;
    roll_setup(f, Some(0), &[1]).await;
    issue(f, Some(0), TacticalAction::EndTurn).await;
    let view = f
        .runtime
        .table_view(f.campaign, TableViewer::Host)
        .await
        .unwrap();
    let variant = view
        .tactical
        .unwrap()
        .casting_options
        .unwrap()
        .variants
        .into_iter()
        .find(|v| v.choice.spell_id == "hold-person")
        .unwrap();
    let SpellMaterialChoice::Material { item } = variant.choice.material else {
        panic!("the genuine source spell requires its material item");
    };
    assert_eq!(
        state(f).await.items[&item].custody,
        Custody::Entity(cultist)
    );
    let cast = issue(
        f,
        None,
        TacticalAction::CastSpell {
            choice: variant.choice,
            targets: SpellTargetChoice::Entities(vec![f.actors[1]]),
        },
    )
    .await;
    roll_setup(f, Some(1), &[1]).await;
    let current = state(f).await;
    assert_eq!(
        current
            .rules
            .as_ref()
            .unwrap()
            .tactical_effects
            .as_ref()
            .unwrap()
            .group_for_owner(cultist)
            .unwrap()
            .source
            .command,
        cast
    );
    assert!(
        current.rules.as_ref().unwrap().entities[&cultist]
            .concentration
            .is_some()
    );
    assert!(
        dmd_rules::active_conditions(current.rules.as_ref().unwrap(), f.actors[1])
            .contains(&Condition::Paralyzed)
    );
    assert!(
        current
            .rules
            .as_ref()
            .unwrap()
            .timing
            .as_ref()
            .unwrap()
            .action_spent
    );
    weapon
}

async fn forged_restore(f: &Fixture, phase: u8) {
    let mut export = export_campaign(&f.pool, f.campaign).await.unwrap();
    let mut image = CampaignState::decode_json(&export.current_state.state_json).unwrap();
    let flow = image.encounter.as_mut().unwrap().flow.as_mut().unwrap();
    match phase {
        0 => {
            flow.resolution
                .as_mut()
                .unwrap()
                .movement
                .as_mut()
                .unwrap()
                .origin
                .id = CommandId::new()
        }
        1 => {
            let TacticalAttackAdmission::Opportunity(window) = &mut flow
                .resolution
                .as_mut()
                .unwrap()
                .attack
                .as_mut()
                .unwrap()
                .admission
            else {
                panic!("expected actual opportunity source");
            };
            window.origin.id = CommandId::new();
        }
        2 => {
            image
                .rules
                .as_mut()
                .unwrap()
                .pending
                .as_mut()
                .unwrap()
                .issued_by
                .id = CommandId::new()
        }
        _ => flow.last_movement.as_mut().unwrap().cause.id = CommandId::new(),
    }
    assert!(image.validate().is_empty());
    export.current_state.state_json = serde_json::to_string(&image).unwrap();
    export
        .upgraded()
        .expect("structurally valid before application provenance preflight");
    let destination = open_sqlite("sqlite::memory:").await.unwrap();
    let runtime = CampaignRuntime::from_content_root(destination.clone(), content());
    assert!(runtime.restore_campaign(&export).await.is_err());
    for table in [
        "campaign_state_current",
        "campaign_lifecycle",
        "event_journal",
        "command_audit",
        "campaign_snapshots",
    ] {
        let count: i64 = sqlx::query_scalar(&format!("SELECT COUNT(*) FROM {table}"))
            .fetch_one(&destination)
            .await
            .unwrap();
        assert_eq!(count, 0, "partial forged restore wrote {table}");
    }
    destination.close().await;
}

async fn react(
    f: &mut Fixture,
    path: &Path,
    mirror: &CampaignRuntime,
    cultist: EntityId,
    weapon: ItemId,
    movement: &CommandMeta,
) -> (CommandMeta, CommandMeta) {
    let pending = state(f).await;
    let view = f
        .runtime
        .table_view(f.campaign, TableViewer::Player(f.players[0]))
        .await
        .unwrap();
    let opportunity = view.tactical.unwrap().opportunity.unwrap();
    assert_eq!(opportunity.actor, f.actors[0]);
    assert_eq!(opportunity.target.actor, cultist);
    assert!(
        opportunity
            .weapons
            .unwrap()
            .weapons
            .iter()
            .any(|w| w.item == weapon)
    );
    assert!(
        f.runtime
            .table_view(f.campaign, TableViewer::Player(f.players[1]))
            .await
            .unwrap()
            .tactical
            .unwrap()
            .opportunity
            .is_none()
    );
    assert_eq!(
        pending
            .encounter
            .as_ref()
            .unwrap()
            .participant(cultist)
            .unwrap()
            .position,
        point(20, 10)
    );
    assert_eq!(
        pending
            .encounter
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .budget
            .movement_spent,
        0
    );
    Box::pin(forged_restore(f, 0)).await;
    let declaration = action(TacticalAction::OpportunityAttack {
        choice: TacticalMeleeChoice::Weapon(dagger_choice(weapon, cultist, false)),
    });
    Box::pin(rejected(f, f.player_meta(1).await, declaration.clone())).await;
    let meta = f.player_meta(0).await;
    Box::pin(rejected(
        f,
        CommandMeta {
            id: CommandId::new(),
            expected_event_sequence: meta.expected_event_sequence - 1,
            ..meta.clone()
        },
        declaration.clone(),
    ))
    .await;
    both(f, mirror, &meta, &declaration).await;
    Box::pin(cold_retry(f, path, &meta, &declaration)).await;
    let current = state(f).await;
    let resolution = current
        .encounter
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .resolution
        .as_ref()
        .unwrap();
    assert_eq!(&resolution.origin, movement);
    assert_eq!(resolution.attack.as_ref().unwrap().origin, meta);
    assert!(
        matches!(&resolution.attack.as_ref().unwrap().admission, TacticalAttackAdmission::Opportunity(window) if &window.origin == movement)
    );
    assert_eq!(resolution.pending.as_ref().unwrap().key.origin, meta.id);
    assert_eq!(
        current
            .rules
            .as_ref()
            .unwrap()
            .pending
            .as_ref()
            .unwrap()
            .issued_by,
        meta
    );
    Box::pin(forged_restore(f, 1)).await;
    let hit = raw(f, Some(0), &[15]).await;
    Box::pin(rejected(f, f.player_meta(1).await, hit.1.clone())).await;
    both(f, mirror, &hit.0, &hit.1).await;
    Box::pin(cold_retry(f, path, &hit.0, &hit.1)).await;
    let mut changed = hit.1.clone();
    let TableAction::Tactical {
        action: TacticalAction::SubmitRoll { result },
    } = &mut changed
    else {
        panic!("expected the accepted physical attack roll");
    };
    result.dice[0].value = 14;
    Box::pin(rejected(f, hit.0.clone(), changed)).await;
    let current = state(f).await;
    assert_eq!(
        current
            .rules
            .as_ref()
            .unwrap()
            .pending
            .as_ref()
            .unwrap()
            .issued_by,
        hit.0
    );
    assert_eq!(
        current
            .encounter
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .resolution
            .as_ref()
            .unwrap()
            .pending
            .as_ref()
            .unwrap()
            .key
            .role,
        TacticalRollRole::AttackDamage
    );
    let damage = raw(f, Some(0), &[2]).await;
    both(f, mirror, &damage.0, &damage.1).await;
    Box::pin(cold_retry(f, path, &damage.0, &damage.1)).await;
    (meta, damage.0)
}

async fn break_concentration(
    f: &mut Fixture,
    path: &Path,
    mirror: &CampaignRuntime,
    cultist: EntityId,
    movement: &CommandMeta,
    damage: &CommandMeta,
) -> CommandMeta {
    let current = state(f).await;
    let resolution = current
        .encounter
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .resolution
        .as_ref()
        .unwrap();
    assert_eq!(&resolution.origin, movement);
    assert!(
        resolution.attack.is_none(),
        "physical receipt finishes before its damage children"
    );
    let pending = resolution.pending.as_ref().unwrap();
    assert_eq!(pending.key.role, TacticalRollRole::Concentration);
    assert_eq!(pending.key.origin, movement.id);
    assert_eq!(pending.key.subject, cultist);
    assert_eq!(
        &current
            .rules
            .as_ref()
            .unwrap()
            .pending
            .as_ref()
            .unwrap()
            .issued_by,
        damage
    );
    assert_eq!(
        current
            .encounter
            .as_ref()
            .unwrap()
            .participant(cultist)
            .unwrap()
            .position,
        point(20, 10)
    );
    for player in f.players {
        assert!(
            f.runtime
                .table_view(f.campaign, TableViewer::Player(player))
                .await
                .unwrap()
                .roll
                .is_none()
        );
    }
    let view = f
        .runtime
        .table_view(f.campaign, TableViewer::Host)
        .await
        .unwrap();
    assert_eq!(view.roll.as_ref().unwrap().roller, Some(cultist));
    assert!(
        view.roll
            .unwrap()
            .reason
            .to_lowercase()
            .contains("concentration")
    );
    Box::pin(forged_restore(f, 2)).await;
    let save = raw(f, None, &[1]).await;
    Box::pin(rejected(f, f.player_meta(0).await, save.1.clone())).await;
    both(f, mirror, &save.0, &save.1).await;
    Box::pin(cold_retry(f, path, &save.0, &save.1)).await;
    save.0
}

async fn verify_finish(
    f: &Fixture,
    mirror: &CampaignRuntime,
    cultist: EntityId,
    movement: &CommandMeta,
    reaction: &CommandMeta,
    callback: &CommandMeta,
    hp: u32,
) {
    let current = state(f).await;
    let encounter = current.encounter.as_ref().unwrap();
    let flow = encounter.flow.as_ref().unwrap();
    let timing = current.rules.as_ref().unwrap().timing.as_ref().unwrap();
    assert_eq!(timing.order[timing.index].actor, cultist);
    assert!(
        timing.action_spent,
        "source casting already spent this actor's Action"
    );
    assert!(!timing.bonus_action_spent);
    assert!(timing.reactions_spent.contains(&f.actors[0]));
    assert!(!timing.reactions_spent.contains(&cultist));
    assert!(flow.resolution.is_none());
    assert_eq!(flow.budget.attacks_remaining, 0);
    assert_eq!(flow.budget.movement_spent, 20);
    assert_eq!(
        encounter.participant(cultist).unwrap().position,
        point(40, 10)
    );
    assert_eq!(
        encounter.participant(f.actors[0]).unwrap().position,
        point(10, 10)
    );
    let receipt = flow.last_movement.as_ref().unwrap();
    assert_eq!(&receipt.original, movement);
    assert_eq!(&receipt.cause, callback);
    assert_eq!(
        (
            receipt.requested_steps,
            receipt.completed_steps,
            receipt.spent_before,
            receipt.spent_after
        ),
        (2, 2, 0, 20)
    );
    assert_eq!(receipt.reason, TacticalMovementEnd::Completed);
    assert!(
        flow.budget
            .weapon_history
            .iter()
            .any(|r| &r.origin == reaction && r.actor == f.actors[0] && !r.on_actor_turn)
    );
    assert_eq!(
        hp - current.rules.as_ref().unwrap().entities[&cultist].hp,
        5
    );
    assert!(
        current.rules.as_ref().unwrap().entities[&cultist]
            .concentration
            .is_none()
    );
    assert!(
        !dmd_rules::active_conditions(current.rules.as_ref().unwrap(), f.actors[1])
            .contains(&Condition::Paralyzed)
    );
    assert_eq!(f.runtime.replay_rules(f.campaign).await.unwrap(), current);
    assert_eq!(mirror.replay_rules(f.campaign).await.unwrap(), current);
    Box::pin(forged_restore(f, 3)).await;
}

#[tokio::test]
async fn damaging_owned_opportunity_breaks_real_concentration_and_resumes_cold_movement() {
    // Keep each scenario phase's future on the heap, with the default test-thread stack.
    Box::pin(run_case()).await;
}
async fn run_case() {
    let path = std::env::temp_dir().join(format!(
        "dmd-oa-concentration-{}.sqlite",
        CommandId::new().0
    ));
    let pool = dmd_persistence::open_sqlite_path(&path).await.unwrap();
    let mut creation = input("Character 0");
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
    let cultist = Box::pin(prepare(&mut f)).await;
    let weapon = Box::pin(arm_then_concentrate(&f, cultist)).await;
    let hp = state(&f).await.rules.as_ref().unwrap().entities[&cultist].hp;
    let movement = f.meta(CommandIssuer::Admin, None, Some(f.session)).await;
    let move_action = action(TacticalAction::Move {
        path: vec![
            TacticalMoveStep {
                destination: point(30, 10),
                mode: MovementMode::Walk,
            },
            TacticalMoveStep {
                destination: point(40, 10),
                mode: MovementMode::Walk,
            },
        ],
    });
    f.runtime
        .execute_table(movement.clone(), move_action.clone())
        .await
        .unwrap();
    Box::pin(cold_retry(&mut f, &path, &movement, &move_action)).await;
    let export = export_campaign(&f.pool, f.campaign).await.unwrap();
    let mirror_pool = open_sqlite("sqlite::memory:").await.unwrap();
    let mirror = CampaignRuntime::from_content_root(mirror_pool.clone(), content());
    mirror.restore_campaign(&export).await.unwrap();
    let (reaction, damage) =
        Box::pin(react(&mut f, &path, &mirror, cultist, weapon, &movement)).await;
    let callback = Box::pin(break_concentration(
        &mut f, &path, &mirror, cultist, &movement, &damage,
    ))
    .await;
    Box::pin(verify_finish(
        &f, &mirror, cultist, &movement, &reaction, &callback, hp,
    ))
    .await;
    Box::pin(cold_retry(&mut f, &path, &movement, &move_action)).await;
    mirror_pool.close().await;
    f.pool.close().await;
    drop(mirror);
    drop(mirror_pool);
    drop(f);
    sqlite_test_cleanup::remove_closed_file(&path)
        .await
        .unwrap();
}
