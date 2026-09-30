//! Genuine installed Air source through normal table creation and control.
//! No manufactured mechanics, immunity, flight profile or saved-state setup.
use super::*;

async fn host_cold(f: &mut Fixture, url: &str, action: TableAction) {
    let command = request(f, true, TableTransportInput::Action(Box::new(action))).await;
    Box::pin(cold(f, url, command)).await;
}

async fn prepare(f: &mut Fixture, url: &str, wall: bool) -> (EntityId, CreatureSourcePin) {
    Box::pin(host_cold(
        f,
        url,
        TableAction::EnableSourceActorAccess { adopted: vec![] },
    ))
    .await;
    let catalog = f
        .runtime
        .table_creature_options(TableCreatureOptionsRequest {
            campaign_id: f.campaign,
            channel: TableTransportChannel::Host,
            revision: view(f, true).await.revision,
        })
        .await
        .unwrap();
    let source = catalog
        .iter()
        .find(|s| s.definition_id == "air-elemental")
        .unwrap();
    let pin = source.source.clone().expect("current installed source pin");
    assert_eq!(source.sizes, [CreatureSize::Large]);
    assert_eq!(source.item_count, 0);
    let air = EntityId::new();
    Box::pin(host_cold(
        f,
        url,
        TableAction::CreateCreature {
            creation: Box::new(TableCreatureCreation {
                entity_id: air,
                name: "Private Air source".into(),
                definition_id: source.definition_id.clone(),
                source: Some(pin.clone()),
                size: CreatureSize::Large,
                additional_languages: vec![],
                ammunition_units: 0,
                item_ids: vec![],
            }),
        },
    ))
    .await;
    Box::pin(host_cold(
        f,
        url,
        TableAction::SetSourceCreatureController {
            actor: air,
            controller: CreatureController::Host,
        },
    ))
    .await;
    let character = f.characters[0];
    let character_actor = f.actors[0];
    let setup = f
        .runtime
        .table_view(f.campaign, TableViewer::Host)
        .await
        .unwrap();
    let count = setup
        .characters
        .iter()
        .find(|c| c.character_id == character)
        .unwrap()
        .equipment
        .as_ref()
        .unwrap()
        .initial_item_count;
    Box::pin(host_cold(
        f,
        url,
        TableAction::PrepareEquipment {
            character_id: character,
            item_ids: (0..count).map(|_| ItemId::new()).collect(),
        },
    ))
    .await;
    let p = |x, y, z| SpatialPoint { x, y, z };
    let mut obstacles = vec![SpatialObstacle {
        id: "character-support".into(),
        volume: SpatialBox {
            min: p(0, 0, 0),
            max: p(20, 30, 40),
        },
        blocks_movement: true,
        blocks_sight: true,
        observable: true,
        cover: CoverDegree::Total,
    }];
    if wall {
        obstacles.push(SpatialObstacle {
            id: "private-air-form-wall".into(),
            volume: SpatialBox {
                min: p(45, 0, 0),
                max: p(46, 90, 90),
            },
            blocks_movement: true,
            blocks_sight: false,
            observable: false,
            cover: CoverDegree::None,
        });
    }
    Box::pin(host_cold(f, url, TableAction::PrepareBattlefield { setup: Box::new(TableBattlefieldSetup {
        encounter_id:EncounterId::new(), scene_id:SceneId::new(), location_id:LocationId::new(),
        name:"Supported character beside a hovering creature".into(),
        battlefield:Battlefield { bounds:SpatialBox { min:p(0,0,0), max:p(100,100,100) }, floor_z:0,
            floor_surface:"stone".into(), ambient_light:LightLevel::Bright, obstacles, terrain:vec![], lights:vec![] },
        characters:vec![TableCharacterPlacement { character_id:character, position:p(10,10,40), height:12, allies:vec![], enemies:vec![air] }],
        creatures:vec![TableCreaturePlacement { actor:air, public_label:"Visible whirling shape".into(), position:p(20,10,40), height:20, allies:vec![], enemies:vec![character_actor] }],
        area_grid_policy:None, geometry_ruling:Ruling { basis:RulingBasis::GmAdjudication, reason:"Character stands on a real platform beside actual source-supported hovering flight.".into() },
    }) })).await;
    Box::pin(host_cold(
        f,
        url,
        TableAction::Tactical {
            action: TacticalAction::Begin {
                execution: TacticalExecutionVersion::ShieldMissileV1,
                combatants: vec![
                    TacticalCombatant {
                        actor: character_actor,
                        source: TacticalSource::Character,
                        surprised: false,
                    },
                    TacticalCombatant {
                        actor: air,
                        source: TacticalSource::Creature {
                            definition_id: "air-elemental".into(),
                        },
                        surprised: false,
                    },
                ],
                groups: vec![
                    InitiativeGroup {
                        actors: vec![character_actor],
                        request_id: RollRequestId::new(),
                    },
                    InitiativeGroup {
                        actors: vec![air],
                        request_id: RollRequestId::new(),
                    },
                ],
            },
        },
    ))
    .await;
    for (host, value) in [(false, 18), (true, 2)] {
        let roll = view(f, host).await.roll.unwrap();
        assert_eq!(roll.mode, RollMode::Normal);
        assert_eq!(
            roll.dice,
            [DieSpec {
                count: 1,
                sides: 20
            }]
        );
        let command = request(
            f,
            host,
            action(TacticalAction::SubmitRoll {
                result: RollResult {
                    request_id: roll.id,
                    source: RollSource::Physical,
                    dice: vec![DieResult { sides: 20, value }],
                },
            }),
        )
        .await;
        Box::pin(cold(f, url, command)).await;
    }
    (air, pin)
}

async fn exercise(f: &mut Fixture, url: &str, scenario: &str) {
    let (air, pin) = Box::pin(prepare(f, url, scenario == "unsupported")).await;
    let before = state(f).await;
    let rules = before.rules.as_ref().unwrap();
    let source = rules.tactical_creatures.as_ref().unwrap();
    assert_eq!(source.profile(air).unwrap().source, pin);
    assert_eq!(
        source.runtime(air).unwrap().controller,
        CreatureController::Host
    );
    assert!(
        rules.entities[&air]
            .condition_immunities
            .contains(&Condition::Prone)
    );
    let body = before.encounter.as_ref().unwrap().participant(air).unwrap();
    assert_eq!(body.movement.fly, Some(180));
    assert!(body.movement.hover);
    let original = body.position;
    let begin = request(f, false, action(TacticalAction::Shove { target: air })).await;
    Box::pin(cold(f, url, begin)).await;
    let ability = if scenario == "push" {
        ShoveSaveAbility::Dexterity
    } else {
        ShoveSaveAbility::Strength
    };
    let choose = decision(f, true, TableShoveInput::Save { ability }).await;
    Box::pin(cold(f, url, choose)).await;
    let roll = view(f, true).await.roll.unwrap();
    assert_eq!(
        roll.modifier,
        if ability == ShoveSaveAbility::Strength {
            2
        } else {
            5
        }
    );
    assert_eq!(
        roll.dice,
        [DieSpec {
            count: 1,
            sides: 20
        }]
    );
    let report = request(
        f,
        true,
        action(TacticalAction::SubmitRoll {
            result: RollResult {
                request_id: roll.id,
                source: RollSource::Physical,
                dice: vec![DieResult {
                    sides: 20,
                    value: 1,
                }],
            },
        }),
    )
    .await;
    Box::pin(cold(f, url, report)).await;
    let owned = view(f, false).await;
    assert_eq!(
        owned
            .tactical
            .as_ref()
            .unwrap()
            .shove
            .as_ref()
            .unwrap()
            .stage,
        TacticalShoveStage::OutcomeChoice
    );
    let visible = serde_json::to_string(&owned).unwrap();
    for secret in [
        "air-elemental",
        "Private Air source",
        "condition_immunities",
        "private-air-form-wall",
        "difficulty",
    ] {
        assert!(
            !visible.contains(secret),
            "private fact in owner view: {secret}"
        );
    }
    if scenario != "prone" {
        let destination = SpatialPoint { x: 30, ..original };
        let push = decision(
            f,
            false,
            TableShoveInput::Outcome {
                choice: ShoveChoice::Push { destination },
            },
        )
        .await;
        Box::pin(cold(f, url, push)).await;
        assert!(view(f, false).await.tactical.unwrap().shove.is_none());
        assert_eq!(
            view(f, true).await.tactical.unwrap().shove.unwrap().stage,
            TacticalShoveStage::PushReview
        );
        if scenario == "unsupported" {
            let paused = state(f).await;
            assert!(matches!(
                dmd_rules::spatial::evaluate_path(
                    paused.encounter.as_ref().unwrap(),
                    &paused,
                    air,
                    &dmd_rules::spatial::SpatialPath {
                        steps: vec![dmd_rules::spatial::MovementStep {
                            destination,
                            mode: MovementMode::Walk
                        }]
                    },
                    &dmd_rules::spatial::MovementAllowance {
                        forced: true,
                        ..Default::default()
                    },
                ),
                Err(dmd_rules::spatial::SpatialError::Unsupported)
            ));
            for ruling in [
                ShoveGeometryRuling::CommitExactPush,
                ShoveGeometryRuling::ConfirmBlockedNoMovement,
            ] {
                let attempt = decision(f, true, TableShoveInput::RulePush { ruling }).await;
                Box::pin(reject(f, attempt)).await;
            }
            let return_choice = decision(
                f,
                true,
                TableShoveInput::RulePush {
                    ruling: ShoveGeometryRuling::ReturnToShover,
                },
            )
            .await;
            Box::pin(cold(f, url, return_choice)).await;
            assert_eq!(
                state(f)
                    .await
                    .encounter
                    .as_ref()
                    .unwrap()
                    .participant(air)
                    .unwrap()
                    .position,
                original
            );
            assert_eq!(
                view(f, false).await.tactical.unwrap().shove.unwrap().stage,
                TacticalShoveStage::OutcomeChoice
            );
        } else {
            let commit = decision(
                f,
                true,
                TableShoveInput::RulePush {
                    ruling: ShoveGeometryRuling::CommitExactPush,
                },
            )
            .await;
            Box::pin(cold(f, url, commit)).await;
        }
    }
    if scenario != "push" {
        let prone = decision(
            f,
            false,
            TableShoveInput::Outcome {
                choice: ShoveChoice::Prone,
            },
        )
        .await;
        // cold accepts the exact outcome after reopening, restores independently,
        // reopens again and retries the same original paid selection envelope.
        Box::pin(cold(f, url, prone)).await;
    }
    let after = state(f).await;
    let rules = after.rules.as_ref().unwrap();
    let flow = after.encounter.as_ref().unwrap().flow.as_ref().unwrap();
    assert!(flow.resolution.is_none());
    assert!(rules.pending.is_none());
    assert!(rules.timing.as_ref().unwrap().action_spent);
    assert!(rules.timing.as_ref().unwrap().reactions_spent.is_empty());
    assert_eq!(flow.budget.attacks_remaining, 0);
    assert_eq!(flow.budget.movement_spent, 0);
    assert!(!rules.entities[&air].prone);
    assert_eq!(
        rules.entities[&air].hp,
        before.rules.as_ref().unwrap().entities[&air].hp
    );
    assert_eq!(
        rules.tactical_inventory,
        before.rules.as_ref().unwrap().tactical_inventory
    );
    assert_eq!(
        after
            .encounter
            .as_ref()
            .unwrap()
            .participant(air)
            .unwrap()
            .position,
        SpatialPoint {
            x: if scenario == "push" { 30 } else { 20 },
            ..original
        }
    );
    assert_eq!(
        rules.rolls.len(),
        before.rules.as_ref().unwrap().rolls.len() + 1,
        "no fall, damage or replacement dice from an immune choice or healthy Hover"
    );
    assert!(
        matches!(rules.rolls.last().unwrap().purpose,PendingPurpose::TacticalResolution { key,.. } if key.role == TacticalRollRole::ShoveSave)
    );
    let spent = request(f, false, action(TacticalAction::Shove { target: air })).await;
    Box::pin(reject(f, spent)).await;
}

#[tokio::test]
async fn installed_air_immunity_hover_and_unsupported_form_use_real_cold_table_commands() {
    for scenario in ["prone", "push", "unsupported"] {
        let directory = std::env::temp_dir().join(format!("dmd-shove-air-{}", CampaignId::new().0));
        std::fs::create_dir_all(&directory).unwrap();
        let url = format!(
            "sqlite://{}",
            directory
                .join("campaign.sqlite")
                .to_string_lossy()
                .replace('\\', "/")
        );
        let pool = open_sqlite(&url).await.unwrap();
        let mut f = Box::pin(Fixture::with_pool(TableContract::default(), pool)).await;
        Box::pin(exercise(&mut f, &url, scenario)).await;
        f.pool.close().await;
        drop(f);
        sqlite_test_cleanup::remove_closed_directory(&directory)
            .await
            .unwrap();
    }
}
