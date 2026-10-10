//! Actual table commands, source creation and physical inputs. No fabricated
//! positive state or event is used to reach either encounter or its completion.
use super::*;

#[path = "table_release_savage_cases.rs"]
mod savage;

#[path = "table_release_custody_cases.rs"]
mod custody;

#[path = "table_release_death_cases.rs"]
mod death;

#[path = "table_release_recharge_cases.rs"]
mod recharge;

async fn closed_request(f: &Fixture, action: TableAction) -> TableTransportRequest {
    let mut request = request(f, None, action).await;
    request.session_id = None;
    request
}

async fn durable_step(
    f: &mut Fixture,
    url: &str,
    directory: &Path,
    label: &str,
    request: TableTransportRequest,
) -> TableTransportResult {
    let before = Box::new(export_campaign(&f.pool, f.campaign).await.unwrap());
    let path = directory.join(format!("{label}-{}.sqlite", CommandId::new().0));
    let mirror_pool = dmd_persistence::open_sqlite_path(&path).await.unwrap();
    let mirror = runtime(mirror_pool.clone());
    Box::pin(mirror.restore_campaign(&before)).await.unwrap();
    mirror_pool.close().await;
    let mirror_pool = dmd_persistence::open_sqlite_path(&path).await.unwrap();
    let mirror = runtime(mirror_pool.clone());
    Box::pin(mirror.resume_campaign(f.campaign)).await.unwrap();
    let accepted = Box::pin(f.runtime.submit_presented_table(request.clone()))
        .await
        .unwrap();
    let independent = Box::pin(mirror.submit_presented_table(request.clone()))
        .await
        .unwrap();
    match (&accepted, &independent) {
        (TableTransportResult::Accepted(left), TableTransportResult::Accepted(right)) => {
            assert_eq!(left.outcome, right.outcome);
            assert_eq!(left.command_id, right.command_id);
        }
        _ => panic!("a release scenario step must be accepted in both independent files"),
    }
    let expected = Box::new(export_campaign(&f.pool, f.campaign).await.unwrap());
    let copied = Box::new(export_campaign(&mirror_pool, f.campaign).await.unwrap());
    assert_eq!(
        CampaignState::decode_json(&expected.current_state.state_json).unwrap(),
        CampaignState::decode_json(&copied.current_state.state_json).unwrap()
    );
    assert_eq!(
        serde_json::to_vec(
            &Box::pin(mirror.submit_presented_table(request.clone()))
                .await
                .unwrap()
        )
        .unwrap(),
        serde_json::to_vec(&independent).unwrap()
    );
    mirror_pool.close().await;
    Box::pin(reopen(f, url)).await;
    assert_eq!(
        serde_json::to_vec(
            &Box::pin(f.runtime.submit_presented_table(request.clone()))
                .await
                .unwrap()
        )
        .unwrap(),
        serde_json::to_vec(&accepted).unwrap()
    );
    let mut changed = request;
    changed.input = TableTransportInput::Action(Box::new(action(TacticalAction::Dodge)));
    assert!(
        Box::pin(f.runtime.submit_presented_table(changed))
            .await
            .is_err()
    );
    let mut after = Box::new(export_campaign(&f.pool, f.campaign).await.unwrap());
    after.exported_at_utc = expected.exported_at_utc.clone();
    assert_eq!(
        after, expected,
        "changed-body refusal and retry preserve every durable row"
    );
    accepted
}

async fn rows(pool: &sqlx::SqlitePool) -> std::collections::BTreeMap<String, i64> {
    let names: Vec<String> =
        sqlx::query_scalar("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
            .fetch_all(pool)
            .await
            .unwrap();
    let mut counts = std::collections::BTreeMap::new();
    for name in names {
        let quoted = name.replace('"', "\"\"");
        let count = sqlx::query_scalar(&format!("SELECT COUNT(*) FROM \"{quoted}\""))
            .fetch_one(pool)
            .await
            .unwrap();
        counts.insert(name, count);
    }
    counts
}

async fn reject_history_forgery(f: &Fixture, retired: &dmd_persistence::CurrentStateRow) {
    let original = Box::new(export_campaign(&f.pool, f.campaign).await.unwrap());
    // Retain a genuine earlier image even if normal snapshot cadence has not
    // retained that particular command. Its positive restore must pass first.
    let mut with_retired = original.clone();
    with_retired
        .snapshots
        .retain(|row| row.event_sequence != retired.applied_event_sequence);
    with_retired.snapshots.push(dmd_persistence::SnapshotRow {
        campaign_id: retired.campaign_id.clone(),
        event_sequence: retired.applied_event_sequence,
        state_schema_version: retired.schema_version,
        state_json: retired.state_json.clone(),
        created_at_utc: with_retired.exported_at_utc.clone(),
    });
    with_retired.snapshots.sort_by_key(|row| row.event_sequence);
    let pool = open_sqlite("sqlite::memory:").await.unwrap();
    Box::pin(runtime(pool.clone()).restore_campaign(&with_retired))
        .await
        .unwrap();
    pool.close().await;
    for retired_image in [false, true] {
        let mut forged = with_retired.clone();
        let json = if retired_image {
            &mut forged
                .snapshots
                .iter_mut()
                .find(|row| row.event_sequence == retired.applied_event_sequence)
                .unwrap()
                .state_json
        } else {
            &mut forged.current_state.state_json
        };
        let mut state = CampaignState::decode_json(json).unwrap();
        state.encounter_history.as_mut().unwrap().spaces[0]
            .battlefield
            .ambient_light = LightLevel::Dim;
        if retired_image {
            state.encounter.as_mut().unwrap().battlefield.ambient_light = LightLevel::Dim;
        }
        assert!(
            state.validate().is_empty(),
            "negative image still has valid structural geometry"
        );
        *json = serde_json::to_string(&state).unwrap();
        forged.upgraded().unwrap();
        let pool = open_sqlite("sqlite::memory:").await.unwrap();
        let before = rows(&pool).await;
        assert!(
            Box::pin(runtime(pool.clone()).restore_campaign(&forged))
                .await
                .is_err()
        );
        assert_eq!(
            rows(&pool).await,
            before,
            "semantic refusal writes no table, including seeded metadata"
        );
        pool.close().await;
    }
}

fn replacement(f: &Fixture, old: &CampaignState, mage: EntityId) -> TableBattlefieldSetup {
    let encounter = old.encounter.as_ref().unwrap();
    TableBattlefieldSetup {
        encounter_id: EncounterId::new(),
        scene_id: SceneId::new(),
        location_id: old.scenes[&encounter.scene_id].location_id,
        name: "Another encounter in the same courtyard".into(),
        battlefield: encounter.battlefield.clone(),
        characters: vec![TableCharacterPlacement {
            character_id: f.characters[0],
            position: SpatialPoint { x: 10, y: 10, z: 0 },
            height: 12,
            allies: vec![],
            enemies: vec![mage],
        }],
        creatures: vec![TableCreaturePlacement {
            actor: mage,
            public_label: "Spellcaster".into(),
            position: SpatialPoint { x: 10, y: 40, z: 0 },
            height: 12,
            allies: vec![],
            enemies: vec![f.actors[0]],
        }],
        area_grid_policy: None,
        geometry_ruling: Ruling {
            basis: RulingBasis::GmAdjudication,
            reason: "New explicit positions; old dropped items stay on the old battlefield.".into(),
        },
    }
}

async fn scenario(f: &mut Fixture, url: &str, directory: &Path) {
    let (mage, cultist) = Box::pin(prepare_magic(f)).await;
    Box::pin(rejected(f, None, action(TacticalAction::FinishEncounter))).await;
    Box::pin(cast(f, url, mage, "mage-armor", mage)).await;
    Box::pin(cold_action(f, url, None, action(TacticalAction::EndTurn))).await;
    Box::pin(cold_action(
        f,
        url,
        Some(0),
        action(TacticalAction::SecondWind),
    ))
    .await;
    Box::pin(cold_raw(f, url, Some(0), &[4])).await;
    let attack = view(f, Some(0))
        .await
        .tactical
        .unwrap()
        .attack_options
        .unwrap();
    let dagger = attack
        .weapons
        .iter()
        .find(|weapon| weapon.name == "Dagger")
        .unwrap()
        .item;
    Box::pin(cold_action(
        f,
        url,
        Some(0),
        action(TacticalAction::Attack {
            choice: WeaponUseChoice {
                after_equipment: None,
                weapon: dagger,
                target: cultist,
                delivery: WeaponDelivery::Thrown,
                ability: Ability::Strength,
                grip: WeaponGrip::OneHand(Hand::Right),
                purpose: WeaponAttackPurpose::Normal,
                ammunition: None,
                equipment_change: Some(AttackEquipmentChange {
                    timing: EquipmentChangeTiming::BeforeAttack,
                    operation: AttackEquipmentOperation::Equip {
                        item: dagger,
                        hand: Hand::Right,
                    },
                }),
            },
        }),
    ))
    .await;
    let roll = view(f, Some(0)).await.roll.unwrap();
    let faces = if roll.mode == RollMode::Normal {
        vec![1]
    } else {
        vec![1, 1]
    };
    Box::pin(cold_raw(f, url, Some(0), &faces)).await;
    Box::pin(cold_action(
        f,
        url,
        Some(0),
        action(TacticalAction::EndTurn),
    ))
    .await;
    Box::pin(conclude(f, url)).await;
    let before = Box::new(state(f).await);
    let flow = before.encounter.as_ref().unwrap().flow.as_ref().unwrap();
    let ground = flow
        .ground_items
        .iter()
        .find(|ground| ground.item == dagger)
        .unwrap()
        .clone();
    let deadline = before
        .rules
        .as_ref()
        .unwrap()
        .tactical_effects
        .as_ref()
        .unwrap()
        .effects
        .iter()
        .find(|effect| effect.source.actor == mage)
        .unwrap()
        .expires
        .clone();
    for player in [0, 1] {
        assert!(
            view(f, Some(player))
                .await
                .tactical
                .unwrap()
                .release
                .is_none()
        );
    }
    assert!(
        view(f, None)
            .await
            .tactical
            .unwrap()
            .release
            .unwrap()
            .may_finish
    );
    Box::pin(rejected(
        f,
        Some(0),
        action(TacticalAction::FinishEncounter),
    ))
    .await;
    let end = request(f, None, TableAction::EndSession).await;
    Box::pin(durable_step(f, url, directory, "closed-aftermath", end)).await;
    // Closed administrative commands require an explicit absent binding.
    Box::pin(rejected(f, None, action(TacticalAction::FinishEncounter))).await;
    let mut player = closed_request(f, action(TacticalAction::FinishEncounter)).await;
    player.channel = TableTransportChannel::Player {
        player_id: f.players[0],
        character_id: f.characters[0],
    };
    player.revision = view(f, Some(0)).await.revision;
    Box::pin(rejected_request(f, player)).await;
    let other = closed_request(f, action(TacticalAction::EndTurn)).await;
    Box::pin(rejected_request(f, other)).await;
    let finish = closed_request(f, action(TacticalAction::FinishEncounter)).await;
    Box::pin(durable_step(f, url, directory, "finished", finish.clone())).await;
    let retired = export_campaign(&f.pool, f.campaign)
        .await
        .unwrap()
        .current_state;
    let finished = Box::new(state(f).await);
    assert_eq!(finished.clock, before.clock);
    assert_eq!(finished.items, before.items);
    assert_eq!(
        finished.rules.as_ref().unwrap().entities,
        before.rules.as_ref().unwrap().entities
    );
    assert_eq!(
        finished.rules.as_ref().unwrap().tactical_inventory,
        before.rules.as_ref().unwrap().tactical_inventory
    );
    assert_eq!(
        finished.encounter_history.as_ref().unwrap().spaces[0].ground_items,
        vec![ground.clone()]
    );
    let receipt = finished
        .encounter_history
        .as_ref()
        .unwrap()
        .last()
        .unwrap()
        .clone();
    assert_eq!(receipt.released_by.id, finish.command_id);
    assert_eq!(
        finished.scenes[&receipt.scene_id].status,
        SceneStatus::Closed
    );
    let repeated = closed_request(f, action(TacticalAction::FinishEncounter)).await;
    Box::pin(rejected_request(f, repeated)).await;
    // The second player has no surviving obligation and need not attend again.
    f.session = PlaySessionId::new();
    let start = request(
        f,
        None,
        TableAction::StartSession {
            id: f.session,
            name: "The next encounter".into(),
            participants: vec![SessionParticipant {
                player_id: f.players[0],
                character_id: Some(f.characters[0]),
                attendance: AttendanceStatus::Present,
            }],
        },
    )
    .await;
    Box::pin(durable_step(f, url, directory, "new-session", start)).await;
    let setup = replacement(f, &finished, mage);
    let mut omitted = setup.clone();
    omitted.creatures.clear();
    omitted.characters[0].enemies.clear();
    Box::pin(rejected(
        f,
        None,
        TableAction::PrepareBattlefield {
            setup: Box::new(omitted),
        },
    ))
    .await;
    assert_eq!(state(f).await.encounter, finished.encounter);
    let mut reused = setup.clone();
    reused.scene_id = receipt.scene_id;
    Box::pin(rejected(
        f,
        None,
        TableAction::PrepareBattlefield {
            setup: Box::new(reused),
        },
    ))
    .await;
    let prepare = request(
        f,
        None,
        TableAction::PrepareBattlefield {
            setup: Box::new(setup.clone()),
        },
    )
    .await;
    Box::pin(durable_step(f, url, directory, "replacement", prepare)).await;
    let begin = request(
        f,
        None,
        action(TacticalAction::Begin {
            execution: TacticalExecutionVersion::EncounterReleaseV1,
            combatants: vec![
                TacticalCombatant {
                    actor: mage,
                    source: TacticalSource::Creature {
                        definition_id: "mage".into(),
                    },
                    surprised: false,
                },
                TacticalCombatant {
                    actor: f.actors[0],
                    source: TacticalSource::Character,
                    surprised: false,
                },
            ],
            groups: [mage, f.actors[0]]
                .into_iter()
                .map(|actor| InitiativeGroup {
                    actors: vec![actor],
                    request_id: RollRequestId::new(),
                })
                .collect(),
        }),
    )
    .await;
    Box::pin(durable_step(f, url, directory, "second-initiative", begin)).await;
    Box::pin(cold_raw(f, url, None, &[20])).await;
    Box::pin(cold_raw(f, url, Some(0), &[3])).await;
    let next = Box::new(state(f).await);
    let rules = next.rules.as_ref().unwrap();
    assert_eq!(
        rules.timing.as_ref().unwrap().turn_number,
        receipt.final_turn.number + 1
    );
    assert_eq!(rules.timing.as_ref().unwrap().round, 1);
    assert_eq!(
        rules
            .tactical_effects
            .as_ref()
            .unwrap()
            .effects
            .iter()
            .find(|effect| effect.source.actor == mage)
            .unwrap()
            .expires,
        deadline
    );
    assert_eq!(
        rules.entities[&f.actors[0]],
        finished.rules.as_ref().unwrap().entities[&f.actors[0]]
    );
    assert_eq!(
        rules
            .tactical_creatures
            .as_ref()
            .unwrap()
            .runtime(mage)
            .unwrap()
            .limited_uses,
        finished
            .rules
            .as_ref()
            .unwrap()
            .tactical_creatures
            .as_ref()
            .unwrap()
            .runtime(mage)
            .unwrap()
            .limited_uses
    );
    assert_eq!(next.encounter_history, finished.encounter_history);
    assert_eq!(
        next.items[&dagger].custody,
        Custody::Location(setup.location_id)
    );
    assert!(
        next.encounter
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .ground_items
            .is_empty()
    );
    let movement = request(
        f,
        None,
        action(TacticalAction::Move {
            path: vec![TacticalMoveStep {
                destination: SpatialPoint { x: 20, y: 40, z: 0 },
                mode: MovementMode::Walk,
            }],
        }),
    )
    .await;
    Box::pin(durable_step(
        f,
        url,
        directory,
        "new-encounter-movement",
        movement,
    ))
    .await;
    assert_eq!(
        state(f)
            .await
            .encounter
            .as_ref()
            .unwrap()
            .participant(mage)
            .unwrap()
            .position
            .x,
        20
    );
    let before_retry = export_campaign(&f.pool, f.campaign).await.unwrap();
    Box::pin(f.runtime.submit_presented_table(finish))
        .await
        .unwrap();
    let mut after_retry = export_campaign(&f.pool, f.campaign).await.unwrap();
    after_retry.exported_at_utc = before_retry.exported_at_utc.clone();
    assert_eq!(
        after_retry, before_retry,
        "old finish retry does not finish the replacement encounter"
    );
    Box::pin(reject_history_forgery(f, &retired)).await;
}

#[tokio::test]
async fn real_release_reuses_source_actors_and_preserves_old_scene_custody_across_two_encounters() {
    let directory =
        std::env::temp_dir().join(format!("dmd-encounter-release-{}", CampaignId::new().0));
    std::fs::create_dir_all(&directory).unwrap();
    let url = format!(
        "sqlite://{}",
        directory
            .join("campaign.sqlite")
            .to_string_lossy()
            .replace('\\', "/")
    );
    let pool = open_sqlite(&url).await.unwrap();
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
    Box::pin(scenario(&mut f, &url, &directory)).await;
    f.pool.close().await;
    drop(f);
    sqlite_test_cleanup::remove_closed_directory(&directory)
        .await
        .unwrap();
}
