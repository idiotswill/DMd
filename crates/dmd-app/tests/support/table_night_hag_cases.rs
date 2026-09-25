use super::*;
use dmd_rules::tactical::TacticalAction;

fn runtime(pool: sqlx::SqlitePool) -> CampaignRuntime {
    CampaignRuntime::from_content_root(
        pool,
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../content"),
    )
}
async fn state(f: &Fixture) -> CampaignState {
    f.runtime
        .open_campaign(f.campaign)
        .await
        .unwrap()
        .state()
        .clone()
}
async fn view(f: &Fixture) -> TablePresentedView {
    f.runtime
        .presented_table_view(f.campaign, TableViewer::Host)
        .await
        .unwrap()
}
async fn request(f: &Fixture, input: TableTransportInput) -> TableTransportRequest {
    TableTransportRequest {
        version: TABLE_TRANSPORT_VERSION,
        command_id: CommandId::new(),
        campaign_id: f.campaign,
        session_id: Some(f.session),
        channel: TableTransportChannel::Host,
        revision: view(f).await.revision,
        input,
    }
}
fn tactical(action: TacticalAction) -> TableTransportInput {
    TableTransportInput::Action(Box::new(TableAction::Tactical { action }))
}
async fn submit(f: &Fixture, input: TableTransportInput) -> TableTransportRequest {
    let request = request(f, input).await;
    Box::pin(f.runtime.submit_presented_table(request.clone()))
        .await
        .unwrap();
    request
}
async fn reject(f: &Fixture, input: TableTransportInput) {
    let request = request(f, input).await;
    let before = export_campaign(&f.pool, f.campaign).await.unwrap();
    assert!(
        Box::pin(f.runtime.submit_presented_table(request))
            .await
            .is_err()
    );
    let mut after = export_campaign(&f.pool, f.campaign).await.unwrap();
    after.exported_at_utc = before.exported_at_utc.clone();
    assert_eq!(
        after, before,
        "rejection cannot pay, journal or change projection history"
    );
}

async fn create_sources(f: &Fixture) -> (EntityId, EntityId) {
    let hag = EntityId::new();
    let target = EntityId::new();
    for (actor, id) in [(hag, "night-hag"), (target, "warhorse")] {
        let catalog = f
            .runtime
            .table_creature_options(TableCreatureOptionsRequest {
                campaign_id: f.campaign,
                channel: TableTransportChannel::Host,
                revision: view(f).await.revision,
            })
            .await
            .unwrap();
        let source = catalog
            .iter()
            .find(|source| source.definition_id == id)
            .unwrap();
        if id == "night-hag" {
            assert_eq!(
                source.item_count, 0,
                "no invented Soul Bag, focus or components"
            );
            assert_eq!(source.additional_languages, 0);
            assert_eq!(source.sizes, [CreatureSize::Medium]);
            assert!(
                source
                    .omitted_features
                    .iter()
                    .any(|s| s == "magic-resistance")
            );
            assert!(source.omitted_features.iter().any(|s| s == "soul-bag"));
        }
        Box::pin(submit(
            f,
            TableTransportInput::Action(Box::new(TableAction::CreateCreature {
                creation: Box::new(TableCreatureCreation {
                    entity_id: actor,
                    name: format!("Private {id}"),
                    definition_id: source.definition_id.clone(),
                    size: source.sizes[0],
                    additional_languages: vec![],
                    ammunition_units: 0,
                    item_ids: (0..source.item_count).map(|_| ItemId::new()).collect(),
                }),
            })),
        ))
        .await;
    }
    let created = state(f).await;
    let rules = created.rules.as_ref().unwrap();
    let entity = &rules.entities[&hag];
    assert_eq!((entity.hp, entity.max_hp), (112, 112));
    assert_eq!(entity.armor, ArmorClass::Fixed(17));
    assert!(entity.prepared_spells.is_empty() && entity.spellcasting.is_none());
    assert!(entity.resources.is_empty());
    assert!(
        rules
            .tactical_creatures
            .as_ref()
            .unwrap()
            .runtime(hag)
            .unwrap()
            .limited_uses
            .is_empty()
    );
    assert!(
        !created
            .items
            .values()
            .any(|item| item.custody == Custody::Entity(hag))
    );
    (hag, target)
}

async fn prepare(f: &mut Fixture, hag: EntityId, target: EntityId) {
    let equipment_count = view(f)
        .await
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
            item_ids: (0..equipment_count).map(|_| ItemId::new()).collect(),
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
                name: "Stone clearing".into(),
                battlefield: Battlefield {
                    bounds: SpatialBox {
                        min: point(0, 0, 0),
                        max: point(160, 100, 60),
                    },
                    floor_z: 0,
                    floor_surface: "stone".into(),
                    ambient_light: LightLevel::Bright,
                    terrain: vec![],
                    obstacles: vec![],
                    lights: vec![],
                },
                characters: vec![TableCharacterPlacement {
                    character_id: f.characters[0],
                    position: point(10, 10, 0),
                    height: 12,
                    allies: vec![],
                    enemies: vec![hag],
                }],
                creatures: vec![
                    TableCreaturePlacement {
                        actor: hag,
                        public_label: "Horned figure".into(),
                        position: point(40, 10, 0),
                        height: 12,
                        allies: vec![],
                        enemies: vec![target],
                    },
                    TableCreaturePlacement {
                        actor: target,
                        public_label: "Horse".into(),
                        position: point(80, 10, 0),
                        height: 16,
                        allies: vec![],
                        enemies: vec![hag],
                    },
                ],
                area_grid_policy: None,
                geometry_ruling: Ruling {
                    basis: RulingBasis::GmAdjudication,
                    reason: "Visible source creatures on an open level floor.".into(),
                },
            }),
        },
        Some(f.session),
    )
    .await;
    let combatants = vec![
        TacticalCombatant {
            actor: hag,
            source: TacticalSource::Creature {
                definition_id: "night-hag".into(),
            },
            surprised: false,
        },
        TacticalCombatant {
            actor: target,
            source: TacticalSource::Creature {
                definition_id: "warhorse".into(),
            },
            surprised: false,
        },
        TacticalCombatant {
            actor: f.actors[0],
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
    Box::pin(submit(
        f,
        tactical(TacticalAction::Begin {
            execution: TacticalExecutionVersion::ReactionsV1,
            combatants,
            groups,
        }),
    ))
    .await;
    for (host, face) in [(true, 20), (true, 8), (false, 1)] {
        let projected = f
            .runtime
            .table_view(
                f.campaign,
                if host {
                    TableViewer::Host
                } else {
                    TableViewer::Player(f.players[0])
                },
            )
            .await
            .unwrap();
        let roll = projected.roll.unwrap();
        let meta = if host {
            f.meta(CommandIssuer::Admin, None, Some(f.session)).await
        } else {
            f.player_meta(0).await
        };
        Box::pin(f.runtime.execute_table(
            meta,
            TableAction::Tactical {
                action: TacticalAction::SubmitRoll {
                    result: RollResult {
                        request_id: roll.id,
                        source: RollSource::Physical,
                        dice: vec![DieResult {
                            sides: 20,
                            value: face,
                        }],
                    },
                },
            },
        ))
        .await
        .unwrap();
    }
    assert_eq!(view(f).await.tactical.unwrap().active_actor, Some(hag));
}

async fn begin_cast(f: &Fixture, hag: EntityId, target: EntityId) -> CommandId {
    let before = state(f).await;
    let options = view(f).await.tactical.unwrap().casting_options.unwrap();
    assert_eq!(options.actor, hag);
    assert_eq!(options.variants.len(), 1);
    let variant = &options.variants[0];
    assert_eq!(variant.choice.spell_id, "magic-missile");
    assert_eq!(
        variant.choice.grant,
        SpellGrantChoice::CreatureFeature {
            feature_id: "spellcasting".into()
        }
    );
    assert_eq!(variant.choice.resource, SpellResourceChoice::SourceFeature);
    assert_eq!(variant.choice.material, SpellMaterialChoice::None);
    assert!(variant.targets.iter().any(|entry| entry.actor == target));
    assert_eq!(
        state(f).await,
        before,
        "projection cannot pay a source activation"
    );
    for (spell, resource, grant, count) in [
        (
            "magic-missile",
            SpellResourceChoice::SourceFeature,
            variant.choice.grant.clone(),
            3,
        ),
        (
            "magic-missile",
            SpellResourceChoice::Slot { level: 4 },
            variant.choice.grant.clone(),
            6,
        ),
        (
            "magic-missile",
            SpellResourceChoice::SourceFeature,
            SpellGrantChoice::Prepared,
            6,
        ),
        (
            "shield",
            SpellResourceChoice::SourceFeature,
            variant.choice.grant.clone(),
            1,
        ),
    ] {
        let mut choice = variant.choice.clone();
        choice.spell_id = spell.into();
        choice.resource = resource;
        choice.grant = grant;
        Box::pin(reject(
            f,
            tactical(TacticalAction::CastSpell {
                choice,
                targets: SpellTargetChoice::Entities(vec![target; count]),
            }),
        ))
        .await;
    }
    let accepted = Box::pin(submit(
        f,
        tactical(TacticalAction::CastSpell {
            choice: variant.choice.clone(),
            targets: SpellTargetChoice::Entities(vec![target; 6]),
        }),
    ))
    .await;
    let paid = state(f).await;
    let timing = paid.rules.as_ref().unwrap().timing.as_ref().unwrap();
    assert!(timing.action_spent);
    assert!(!timing.bonus_action_spent && !timing.slot_spent_this_turn);
    let flow = paid.encounter.as_ref().unwrap().flow.as_ref().unwrap();
    assert_eq!(flow.budget.attacks_remaining, 0);
    assert!(flow.budget.weapon_history.is_empty());
    let cast = &flow.resolution.as_ref().unwrap().casts[0];
    assert_eq!(cast.cast.plan.origin.id, accepted.command_id);
    assert_eq!(cast.cast.plan.program.spell_level, 4);
    assert_eq!(
        cast.cast
            .plan
            .program
            .source
            .creature_definition_id
            .as_deref(),
        Some("night-hag")
    );
    assert_eq!(cast.targets.len(), 6);
    assert!(cast.targets.iter().all(|t| t.actor == target));
    accepted.command_id
}

async fn cold_amount(f: &mut Fixture, url: &str, request: TableTransportRequest) {
    let exported = export_campaign(&f.pool, f.campaign).await.unwrap();
    let pool = open_sqlite("sqlite::memory:").await.unwrap();
    let mirror = runtime(pool.clone());
    Box::pin(mirror.restore_campaign(&exported)).await.unwrap();
    f.pool.close().await;
    f.pool = open_sqlite(url).await.unwrap();
    f.runtime = runtime(f.pool.clone());
    let accepted = Box::pin(f.runtime.submit_presented_table(request.clone()))
        .await
        .unwrap();
    Box::pin(mirror.submit_presented_table(request.clone()))
        .await
        .unwrap();
    assert_eq!(
        mirror.open_campaign(f.campaign).await.unwrap().state(),
        &state(f).await
    );
    pool.close().await;
    drop(mirror);
    drop(pool);
    f.pool.close().await;
    f.pool = open_sqlite(url).await.unwrap();
    f.runtime = runtime(f.pool.clone());
    let before = export_campaign(&f.pool, f.campaign).await.unwrap();
    assert_eq!(
        Box::pin(f.runtime.submit_presented_table(request.clone()))
            .await
            .unwrap(),
        accepted
    );
    let mut changed = request;
    changed.input = tactical(TacticalAction::Dodge);
    assert!(
        Box::pin(f.runtime.submit_presented_table(changed))
            .await
            .is_err()
    );
    let mut after = export_campaign(&f.pool, f.campaign).await.unwrap();
    after.exported_at_utc = before.exported_at_utc.clone();
    assert_eq!(after, before);
}

async fn finish_darts(
    f: &mut Fixture,
    url: &str,
    hag: EntityId,
    target: EntityId,
    source: CommandId,
) {
    let mut ids = std::collections::HashSet::new();
    for dart in 0..6 {
        let projected = view(f).await;
        let roll = projected
            .roll
            .expect("one source d4 amount per dart, not an attack/save");
        assert!(ids.insert(roll.id));
        assert_eq!(roll.dice, [DieSpec { count: 1, sides: 4 }]);
        assert_eq!(roll.modifier, 1);
        let pending = state(f).await;
        let flow = pending.encounter.as_ref().unwrap().flow.as_ref().unwrap();
        assert_eq!(
            flow.resolution.as_ref().unwrap().casts[0]
                .cast
                .plan
                .origin
                .id,
            source
        );
        assert_eq!(
            pending.rules.as_ref().unwrap().entities[&target].hp,
            19 - 2 * dart
        );
        let request = request(
            f,
            tactical(TacticalAction::SubmitRoll {
                result: RollResult {
                    request_id: roll.id,
                    source: RollSource::Physical,
                    dice: vec![DieResult { sides: 4, value: 1 }],
                },
            }),
        )
        .await;
        Box::pin(cold_amount(f, url, request)).await;
    }
    let finished = state(f).await;
    let rules = finished.rules.as_ref().unwrap();
    assert_eq!(rules.entities[&target].hp, 7);
    assert_eq!(rules.entities[&hag].hp, 112);
    assert!(rules.entities[&hag].resources.is_empty());
    assert!(rules.entities[&hag].prepared_spells.is_empty());
    assert!(rules.entities[&hag].spellcasting.is_none());
    assert!(
        rules
            .tactical_creatures
            .as_ref()
            .unwrap()
            .runtime(hag)
            .unwrap()
            .limited_uses
            .is_empty()
    );
    let flow = finished.encounter.as_ref().unwrap().flow.as_ref().unwrap();
    assert!(flow.resolution.is_none());
    assert!(flow.budget.weapon_history.is_empty());
    assert_eq!(flow.budget.attacks_remaining, 0);
    assert!(rules.timing.as_ref().unwrap().action_spent);
    assert!(!rules.timing.as_ref().unwrap().slot_spent_this_turn);
    assert!(view(f).await.roll.is_none());
}

#[tokio::test]
async fn night_hag_current_catalog_casts_six_real_darts_through_cold_owned_transport() {
    let directory = std::env::temp_dir().join(format!("dmd-night-hag-{}", CampaignId::new().0));
    std::fs::create_dir_all(&directory).unwrap();
    let database = directory.join("campaign.sqlite");
    let url = format!("sqlite://{}", database.display());
    let pool = open_sqlite(&url).await.unwrap();
    let mut f = Box::pin(Fixture::with_pool(TableContract::default(), pool)).await;
    let (hag, target) = Box::pin(create_sources(&f)).await;
    Box::pin(prepare(&mut f, hag, target)).await;
    let source = Box::pin(begin_cast(&f, hag, target)).await;
    Box::pin(finish_darts(&mut f, &url, hag, target, source)).await;
    f.pool.close().await;
    drop(f);
    sqlite_test_cleanup::remove_closed_file(&database)
        .await
        .unwrap();
    sqlite_test_cleanup::remove_closed_directory(&directory)
        .await
        .unwrap();
}
