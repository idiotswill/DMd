use super::*;
use dmd_rules::tactical::TacticalAction;

#[tokio::test]
async fn source_casting_keeps_player_saves_ray_causes_and_cold_retry_authority() {
    let mut f = Fixture::new().await;
    let (cultist, dragon, hidden) = create_casters(&mut f).await;
    prepare(&mut f, cultist, dragon, hidden).await;
    let hold = cast_hold(&f, cultist, hidden).await;
    finish_hold(&f, cultist, hold).await;
    finish_rays(&f, cultist, dragon).await;
    f.pool.close().await;
}

async fn create_casters(f: &mut Fixture) -> (EntityId, EntityId, EntityId) {
    let cultist = EntityId::new();
    let dragon = EntityId::new();
    let hidden = EntityId::new();
    for (actor, definition, size, ammunition) in [
        (cultist, "cultist-fanatic", CreatureSize::Medium, 0),
        (dragon, "adult-red-dragon", CreatureSize::Huge, 0),
        (hidden, "goblin-warrior", CreatureSize::Small, 20),
    ] {
        let allocation =
            dmd_rules::tactical_creature_equipment::creature_equipment_plan(definition, ammunition)
                .unwrap();
        f.host(
            TableAction::CreateCreature {
                creation: Box::new(TableCreatureCreation {
                    entity_id: actor,
                    name: format!("Private {definition}"),
                    definition_id: definition.into(),
                    size,
                    additional_languages: vec![],
                    ammunition_units: ammunition,
                    item_ids: allocation.iter().map(|_| ItemId::new()).collect(),
                }),
            },
            Some(f.session),
        )
        .await;
    }
    (cultist, dragon, hidden)
}

async fn prepare(f: &mut Fixture, cultist: EntityId, dragon: EntityId, hidden: EntityId) {
    let view = f
        .runtime
        .table_view(f.campaign, TableViewer::Host)
        .await
        .unwrap();
    let count = view
        .characters
        .iter()
        .find(|character| character.character_id == f.characters[0])
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
    let point = |x, y, z| SpatialPoint { x, y, z };
    f.host(
        TableAction::PrepareBattlefield {
            setup: Box::new(TableBattlefieldSetup {
                encounter_id: EncounterId::new(),
                scene_id: SceneId::new(),
                location_id: LocationId::new(),
                name: "Open courtyard".into(),
                battlefield: Battlefield {
                    bounds: SpatialBox {
                        min: point(0, 0, 0),
                        max: point(200, 120, 80),
                    },
                    floor_z: 0,
                    floor_surface: "stone".into(),
                    ambient_light: LightLevel::Bright,
                    terrain: vec![],
                    obstacles: vec![SpatialObstacle {
                        id: "courtyard-wall".into(),
                        volume: SpatialBox {
                            min: point(0, 80, 0),
                            max: point(200, 82, 80),
                        },
                        blocks_movement: true,
                        blocks_sight: true,
                        observable: true,
                        cover: CoverDegree::Total,
                    }],
                    lights: vec![],
                },
                characters: vec![TableCharacterPlacement {
                    character_id: f.characters[0],
                    position: point(10, 10, 0),
                    height: 12,
                    allies: vec![],
                    enemies: vec![cultist, dragon],
                }],
                creatures: vec![
                    TableCreaturePlacement {
                        actor: cultist,
                        public_label: "Robed figure".into(),
                        position: point(40, 10, 0),
                        height: 12,
                        allies: vec![],
                        enemies: vec![f.actors[0], dragon],
                    },
                    TableCreaturePlacement {
                        actor: dragon,
                        public_label: "Large winged creature".into(),
                        position: point(100, 40, 0),
                        height: 40,
                        allies: vec![],
                        enemies: vec![f.actors[0], cultist],
                    },
                    TableCreaturePlacement {
                        actor: hidden,
                        public_label: "Hidden guard".into(),
                        position: point(70, 90, 0),
                        height: 8,
                        allies: vec![],
                        enemies: vec![],
                    },
                ],
                area_grid_policy: None,
                geometry_ruling: Ruling {
                    basis: RulingBasis::GmAdjudication,
                    reason: "Open visible courtyard and explicit creature positions.".into(),
                },
            }),
        },
        Some(f.session),
    )
    .await;
    let combatants = vec![
        TacticalCombatant {
            actor: cultist,
            source: TacticalSource::Creature {
                definition_id: "cultist-fanatic".into(),
            },
            surprised: false,
        },
        TacticalCombatant {
            actor: dragon,
            source: TacticalSource::Creature {
                definition_id: "adult-red-dragon".into(),
            },
            surprised: false,
        },
        TacticalCombatant {
            actor: f.actors[0],
            source: TacticalSource::Character,
            surprised: false,
        },
        TacticalCombatant {
            actor: hidden,
            source: TacticalSource::Creature {
                definition_id: "goblin-warrior".into(),
            },
            surprised: false,
        },
    ];
    let groups = combatants
        .iter()
        .map(|combatant| InitiativeGroup {
            actors: vec![combatant.actor],
            request_id: RollRequestId::new(),
        })
        .collect();
    f.host(
        TableAction::Tactical {
            action: TacticalAction::Begin {
                execution: dmd_domain::TacticalExecutionVersion::ReactionsV1,
                combatants,
                groups,
            },
        },
        Some(f.session),
    )
    .await;
    submit(f, true, &[20]).await;
    submit(f, true, &[1]).await;
    submit(f, false, &[2]).await;
    submit(f, true, &[1]).await;
    assert_eq!(
        f.runtime
            .table_view(f.campaign, TableViewer::Host)
            .await
            .unwrap()
            .tactical
            .unwrap()
            .active_actor,
        Some(cultist)
    );
}

async fn host_action(f: &Fixture, action: TacticalAction) -> CommandMeta {
    let meta = f.meta(CommandIssuer::Admin, None, Some(f.session)).await;
    f.runtime
        .execute_table(meta.clone(), TableAction::Tactical { action })
        .await
        .unwrap();
    meta
}

async fn cast_hold(f: &Fixture, cultist: EntityId, hidden: EntityId) -> (CommandMeta, TableAction) {
    let source_before = f
        .runtime
        .open_campaign(f.campaign)
        .await
        .unwrap()
        .state()
        .clone();
    let view = f
        .runtime
        .table_view(f.campaign, TableViewer::Host)
        .await
        .unwrap();
    let options = view.tactical.unwrap().casting_options.unwrap();
    assert_eq!(options.actor, cultist);
    assert_eq!(
        f.runtime.open_campaign(f.campaign).await.unwrap().state(),
        &source_before,
        "preview must not consume source counters or change history"
    );
    let variant = options
        .variants
        .iter()
        .find(|variant| variant.choice.spell_id == "hold-person")
        .expect("real source material and ability are available");
    assert!(matches!(
        variant.choice.material,
        SpellMaterialChoice::Material { .. }
    ));
    assert!(
        variant
            .targets
            .iter()
            .any(|target| target.actor == f.actors[0])
    );
    assert!(
        !variant.targets.iter().any(|target| target.actor == hidden),
        "even the host's caster choices use the actor's perception"
    );
    // Private source type eligibility never filters a legally chosen non-Humanoid.
    assert!(
        variant
            .targets
            .iter()
            .any(|target| target.label == "Large winged creature")
    );
    let player = f
        .runtime
        .table_view(f.campaign, TableViewer::Player(f.players[0]))
        .await
        .unwrap();
    assert!(player.tactical.as_ref().unwrap().casting_options.is_none());
    let private = serde_json::to_string(&player).unwrap();
    for secret in [
        "Private cultist-fanatic",
        "cultist-fanatic",
        "hold-person",
        "spell-material:",
        "source_type_matches",
    ] {
        assert!(!private.contains(secret), "{secret}");
    }
    let action = TableAction::Tactical {
        action: TacticalAction::CastSpell {
            choice: variant.choice.clone(),
            targets: SpellTargetChoice::Entities(vec![f.actors[0]]),
        },
    };
    let before = f
        .runtime
        .open_campaign(f.campaign)
        .await
        .unwrap()
        .state()
        .clone();
    assert!(
        f.runtime
            .execute_table(f.player_meta(0).await, action.clone())
            .await
            .is_err()
    );
    assert_eq!(
        f.runtime.open_campaign(f.campaign).await.unwrap().state(),
        &before
    );
    let meta = f.meta(CommandIssuer::Admin, None, Some(f.session)).await;
    f.runtime
        .execute_table(meta.clone(), action.clone())
        .await
        .unwrap();
    cold_retry(f, &meta, &action).await;
    (meta, action)
}

async fn cold_retry(f: &Fixture, meta: &CommandMeta, action: &TableAction) {
    let export = export_campaign(&f.pool, f.campaign).await.unwrap();
    let path = std::env::temp_dir().join(format!("dmd-casting-{}.sqlite", CommandId::new().0));
    let content = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../content");
    let pool = dmd_persistence::open_sqlite_path(&path).await.unwrap();
    let restored = CampaignRuntime::from_content_root(pool.clone(), &content);
    restored.restore_campaign(&export).await.unwrap();
    pool.close().await;
    drop(restored);
    drop(pool);
    let reopened_pool = dmd_persistence::open_sqlite_path(&path).await.unwrap();
    let reopened = CampaignRuntime::from_content_root(reopened_pool.clone(), &content);
    let expected = f
        .runtime
        .open_campaign(f.campaign)
        .await
        .unwrap()
        .state()
        .clone();
    assert_eq!(
        reopened.resume_campaign(f.campaign).await.unwrap().state(),
        &expected
    );
    assert!(
        reopened
            .execute_table(meta.clone(), action.clone())
            .await
            .unwrap()
            .already_accepted
    );
    assert_eq!(
        reopened.open_campaign(f.campaign).await.unwrap().state(),
        &expected
    );
    // A missing accepted origin cannot be replaced by a plausible current snapshot.
    let mut hostile = export.clone();
    hostile
        .command_audit
        .retain(|audit| audit.id != meta.id.0.to_string());
    assert!(reopened.restore_campaign(&hostile).await.is_err());
    assert_eq!(
        reopened.open_campaign(f.campaign).await.unwrap().state(),
        &expected
    );
    reopened_pool.close().await;
    drop(reopened);
    drop(reopened_pool);
    sqlite_test_cleanup::remove_closed_file(&path)
        .await
        .unwrap();
}

async fn finish_hold(f: &Fixture, cultist: EntityId, accepted: (CommandMeta, TableAction)) {
    let player = f
        .runtime
        .table_view(f.campaign, TableViewer::Player(f.players[0]))
        .await
        .unwrap();
    assert_eq!(player.roll.as_ref().unwrap().roller, Some(f.actors[0]));
    assert_eq!(player.roll.as_ref().unwrap().reason, "Saving throw");
    assert_eq!(player.tactical.unwrap().may_fail_save, Some(f.actors[0]));
    let other = f
        .runtime
        .table_view(f.campaign, TableViewer::Player(f.players[1]))
        .await
        .unwrap();
    assert!(other.roll.is_none());
    assert!(other.tactical.unwrap().may_fail_save.is_none());
    submit(f, false, &[2]).await;
    let state = f
        .runtime
        .open_campaign(f.campaign)
        .await
        .unwrap()
        .state()
        .clone();
    assert!(
        state.rules.as_ref().unwrap().entities[&cultist]
            .concentration
            .is_some()
    );
    assert!(
        dmd_rules::active_conditions(state.rules.as_ref().unwrap(), f.actors[0])
            .contains(&Condition::Paralyzed)
    );
    assert!(
        f.runtime
            .execute_table(accepted.0, accepted.1)
            .await
            .unwrap()
            .already_accepted
    );
    host_action(f, TacticalAction::EndTurn).await;
    // The other creature's end creates the dragon's genuine optional window.
    let view = f
        .runtime
        .table_view(f.campaign, TableViewer::Host)
        .await
        .unwrap();
    if view.tactical.unwrap().legendary_action.is_some() {
        host_action(f, TacticalAction::DeclineLegendaryAction).await;
    }
}

async fn choose_next(f: &Fixture) {
    let view = f
        .runtime
        .table_view(f.campaign, TableViewer::Host)
        .await
        .unwrap();
    if let Some(choice) = view
        .tactical
        .and_then(|t| t.continuation)
        .and_then(|c| c.choices.first().cloned())
    {
        host_action(
            f,
            TacticalAction::ChooseTurnWork {
                occurrence: choice.occurrence,
            },
        )
        .await;
    }
}

async fn finish_rays(f: &Fixture, cultist: EntityId, dragon: EntityId) {
    let view = f
        .runtime
        .table_view(f.campaign, TableViewer::Host)
        .await
        .unwrap();
    let options = view.tactical.unwrap().casting_options.unwrap();
    assert_eq!(options.actor, dragon);
    let variant = options
        .variants
        .iter()
        .find(|variant| variant.choice.spell_id == "scorching-ray")
        .unwrap();
    assert_eq!(variant.minimum_targets, 3);
    assert!(variant.repeated_targets);
    assert!(
        !options
            .variants
            .iter()
            .any(|variant| variant.choice.spell_id == "fireball")
    );
    assert!(
        options
            .unavailable
            .iter()
            .any(|reason| reason.starts_with("Fireball:"))
    );
    let action = TableAction::Tactical {
        action: TacticalAction::CastSpell {
            choice: variant.choice.clone(),
            targets: SpellTargetChoice::Entities(vec![cultist; 3]),
        },
    };
    let cast = f.meta(CommandIssuer::Admin, None, Some(f.session)).await;
    f.runtime
        .execute_table(cast.clone(), action.clone())
        .await
        .unwrap();
    cold_retry(f, &cast, &action).await;
    let mut prior_save = None;
    let mut keys = std::collections::HashSet::new();
    for _ in 0..3 {
        choose_next(f).await;
        let state = f
            .runtime
            .open_campaign(f.campaign)
            .await
            .unwrap()
            .state()
            .clone();
        let r = state
            .encounter
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .resolution
            .as_ref()
            .unwrap();
        let attack = r.attack.as_ref().unwrap();
        assert_eq!(attack.actor, dragon);
        assert!(
            matches!(&attack.admission, TacticalAttackAdmission::Spell { casting_origin } if casting_origin == &cast)
        );
        if let Some(previous) = &prior_save {
            assert_eq!(&attack.origin, previous);
        }
        let request = state.rules.as_ref().unwrap().pending.as_ref().unwrap();
        assert_eq!(request.request.roller, Some(dragon));
        assert!(keys.insert(request.request.id));
        submit(f, true, &[10]).await;
        submit(f, true, &[1, 1]).await;
        let view = f
            .runtime
            .table_view(f.campaign, TableViewer::Host)
            .await
            .unwrap();
        assert_eq!(view.roll.as_ref().unwrap().roller, Some(cultist));
        assert!(view.roll.as_ref().unwrap().reason.contains("concentration"));
        prior_save = Some(submit(f, true, &[20]).await);
    }
    let state = f
        .runtime
        .open_campaign(f.campaign)
        .await
        .unwrap()
        .state()
        .clone();
    assert!(
        state
            .encounter
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .resolution
            .is_none()
    );
    assert_eq!(state.rules.as_ref().unwrap().entities[&cultist].hp, 38);
    assert!(
        state.rules.as_ref().unwrap().entities[&cultist]
            .concentration
            .is_some()
    );
    cold_retry(f, &cast, &action).await;
}

async fn submit(f: &Fixture, host: bool, faces: &[u16]) -> CommandMeta {
    let viewer = if host {
        TableViewer::Host
    } else {
        TableViewer::Player(f.players[0])
    };
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
            .flat_map(|die| std::iter::repeat_n(die.sides, usize::from(die.count)))
            .collect::<Vec<_>>()
    } else {
        vec![20, 20]
    };
    assert_eq!(sides.len(), faces.len());
    let meta = if host {
        f.meta(CommandIssuer::Admin, None, Some(f.session)).await
    } else {
        f.player_meta(0).await
    };
    let action = TableAction::Tactical {
        action: TacticalAction::SubmitRoll {
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
        },
    };
    f.runtime.execute_table(meta.clone(), action).await.unwrap();
    meta
}
