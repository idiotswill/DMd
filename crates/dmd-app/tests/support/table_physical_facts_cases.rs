//! Genuine current creation, opaque physical controls and original-history recovery.
use super::physical_creation_driver::*;
use super::*;
use dmd_rules::tactical::TacticalAction;

async fn v5_request(
    f: &Fixture,
    channel: TableTransportChannel,
    input: TableTransportInput,
) -> TableTransportRequest {
    let mut value = request(f, channel, input).await;
    value.version = 5;
    value
}
async fn v5_step(
    f: &mut Fixture,
    path: &Path,
    channel: TableTransportChannel,
    input: TableTransportInput,
) -> TableTransportRequest {
    let value = v5_request(f, channel, input).await;
    Box::pin(cold_step(f, path, value.clone())).await;
    value
}
async fn fact_request(
    f: &Fixture,
    kind: &str,
    label: &str,
    input: PhysicalFactInput,
) -> TableTransportRequest {
    let projected = view(f, &TableTransportChannel::Host).await;
    let control = projected
        .physical
        .as_ref()
        .unwrap()
        .controls
        .iter()
        .find(|control| control.kind == kind && control.label.contains(label))
        .unwrap();
    v5_request(
        f,
        TableTransportChannel::Host,
        TableTransportInput::PhysicalFact {
            handle: control.key,
            input,
        },
    )
    .await
}
async fn fact(
    f: &mut Fixture,
    path: &Path,
    kind: &str,
    label: &str,
    input: PhysicalFactInput,
) -> TableTransportRequest {
    let value = fact_request(f, kind, label, input).await;
    Box::pin(cold_step(f, path, value.clone())).await;
    value
}
fn reason() -> String {
    "Host records the actual physical object, excluding separately listed contents.".into()
}
async fn load_for(f: &Fixture, actor: EntityId) -> PhysicalLoad {
    view(f, &TableTransportChannel::Host)
        .await
        .physical
        .unwrap()
        .actors
        .into_iter()
        .find(|entry| entry.actor == actor)
        .unwrap()
        .load
        .unwrap()
}
async fn assert_mass_capabilities(
    f: &Fixture,
    accepted: &TableTransportRequest,
    grapple_version: Option<u32>,
) {
    let current = state(f).await;
    assert_eq!(
        dmd_rules::table::source_control::presentation_version(&current),
        5
    );
    assert_eq!(
        dmd_rules::table::grapple_enabled(&current),
        grapple_version.is_some()
    );
    assert_eq!(
        dmd_rules::table::grapple_transport_enabled(&current),
        grapple_version == Some(4)
    );
    for channel in [TableTransportChannel::Host, player(f, 0), player(f, 1)] {
        let projected = view(f, &channel).await;
        assert_eq!(projected.physical.as_ref().unwrap().version, 5);
        assert_eq!(
            projected.grapple.as_ref().map(|grapple| grapple.version),
            grapple_version
        );
        if let Some(source) = &projected.source_control {
            assert_eq!(source.version, 2);
        }
        if grapple_version != Some(4) {
            assert!(projected.inspiration_transfer.is_none());
        }
    }
    let saved = export_campaign(&f.pool, f.campaign).await.unwrap();
    assert_eq!(saved.table_projection_history.last().unwrap().version, 5);
    let binding = saved
        .table_transport_bindings
        .iter()
        .find(|binding| binding.meta.id == accepted.command_id)
        .unwrap();
    assert_eq!(binding.version, 5);
    let request: TableTransportRequest = serde_json::from_str(&binding.request_json).unwrap();
    assert_eq!(request.version, 5);
    assert_eq!(&request, accepted);
    assert_eq!(
        saved
            .table_projection_history
            .iter()
            .find(|record| record.ordinal == binding.projection_ordinal)
            .unwrap()
            .version,
        5
    );
    if grapple_version != Some(4) {
        for input in [
            TableAction::AwardHeroicInspiration {
                character_id: f.characters[0],
                reason: "Physical facts alone do not grant Inspiration.".into(),
            },
            TableAction::AwardExcessInspiration {
                character_id: f.characters[0],
                reason: "Grapple alone does not grant excess Inspiration.".into(),
            },
        ] {
            let forbidden = v5_request(f, TableTransportChannel::Host, action(input)).await;
            Box::pin(reject(f, forbidden)).await;
        }
    }
}
async fn create_pc(
    f: &mut Fixture,
    path: &Path,
    index: usize,
    purchases: &[(&str, u16)],
) -> TableTransportRequest {
    let options = f
        .runtime
        .character_creation_options(f.campaign)
        .await
        .unwrap();
    let mut selected = input(&format!("Mass character {index}"));
    selected.purchases = purchases
        .iter()
        .map(|(id, quantity)| EquipmentChoice {
            item_id: (*id).into(),
            quantity: *quantity,
        })
        .collect();
    selected.masteries = ["greatsword".into(), "glaive".into(), "dagger".into()];
    let input = action(TableAction::CreateCharacterFromSource {
        character_id: f.characters[index],
        entity_id: f.actors[index],
        player_id: f.players[index],
        source: options.source,
        input: selected,
    });
    Box::pin(step(f, path, TableTransportChannel::Host, input)).await
}
async fn materialize(f: &mut Fixture, path: &Path, index: usize) {
    let projected = view(f, &TableTransportChannel::Host).await;
    let count = projected
        .characters
        .iter()
        .find(|c| c.character_id == f.characters[index])
        .unwrap()
        .equipment
        .as_ref()
        .unwrap()
        .initial_item_count;
    let input = action(TableAction::PrepareEquipment {
        character_id: f.characters[index],
        item_ids: (0..count).map(|_| ItemId::new()).collect(),
    });
    Box::pin(step(f, path, TableTransportChannel::Host, input)).await;
}

#[tokio::test]
async fn actual_pc_body_container_coins_and_apparel_are_exact_private_owned_and_cold_portable() {
    Box::pin(pc_case()).await;
}
async fn pc_case() {
    let path = std::env::temp_dir().join(format!("dmd-mass-pc-{}.sqlite", CommandId::new().0));
    let mut f = Box::pin(blank(&path)).await;
    let creation = Box::pin(create_pc(
        &mut f,
        &path,
        0,
        &[
            ("leather-armor", 1),
            ("glaive", 1),
            ("waterskin", 1),
            ("gaming-dice", 1),
            ("backpack", 1),
            ("arrows", 20),
        ],
    ))
    .await;
    Box::pin(materialize(&mut f, &path, 0)).await;
    assert_eq!(
        state(&f).await.table.unwrap().character_profiles[&f.characters[0]].money_cp,
        17170
    );
    let previous = Box::pin(f.runtime.submit_presented_table(creation.clone()))
        .await
        .unwrap();
    Box::pin(v5_step(
        &mut f,
        &path,
        TableTransportChannel::Host,
        TableTransportInput::EnablePhysicalFacts,
    ))
    .await;
    assert_eq!(
        Box::pin(f.runtime.submit_presented_table(creation))
            .await
            .unwrap(),
        previous
    );
    let unrelated = view(&f, &player(&f, 1)).await;
    let initial = load_for(&f, f.actors[0]).await;
    assert_eq!(initial.known_pounds, "22");
    assert!(!initial.complete);
    assert!(initial.body_and_load_pounds.is_none());
    let body = Box::pin(fact(
        &mut f,
        &path,
        "body",
        "Mass character 0",
        PhysicalFactInput::Body {
            pounds: Some("180.25".into()),
            reason: reason(),
        },
    ))
    .await;
    Box::pin(fact(
        &mut f,
        &path,
        "item",
        "Waterskin",
        PhysicalFactInput::Item {
            choice: PhysicalItemInput::Catalog {
                condition: Some("full".into()),
            },
            reason: reason(),
        },
    ))
    .await;
    Box::pin(fact(
        &mut f,
        &path,
        "item",
        "Gaming Set: Dice",
        PhysicalFactInput::Item {
            choice: PhysicalItemInput::Unit {
                pounds: "0.125".into(),
                condition: None,
            },
            reason: reason(),
        },
    ))
    .await;
    let apparel = Box::pin(fact(
        &mut f,
        &path,
        "personal",
        "Mass character 0",
        PhysicalFactInput::PersonalItem {
            name: "Separate travel cloak".into(),
            pounds: "2.5".into(),
            separate_from_listed: true,
            reason: reason(),
        },
    ))
    .await;
    let coins = PhysicalCoins {
        cp: 0,
        sp: 7,
        ep: 0,
        gp: 171,
        pp: 0,
    };
    let currency = Box::pin(fact(
        &mut f,
        &path,
        "currency",
        "Mass character 0",
        PhysicalFactInput::Currency {
            coins,
            reason:
                "The existing wallet contains these actual coins, with no other monetary asset."
                    .into(),
        },
    ))
    .await;
    Box::pin(fact(
        &mut f,
        &path,
        "coverage",
        "Mass character 0",
        PhysicalFactInput::Coverage {
            complete: true,
            reason: "The listed cloak is the only additional apparel or payload.".into(),
        },
    ))
    .await;
    let complete = load_for(&f, f.actors[0]).await;
    assert!(complete.complete);
    assert_eq!(complete.known_pounds, "33.185");
    assert_eq!(complete.body_and_load_pounds.as_deref(), Some("213.435"));
    let current = state(&f).await;
    let identity = dmd_rules::physical_facts::actor_identity(&current, f.actors[0]).unwrap();
    let mut unrelated_change = current.clone();
    let profile = unrelated_change
        .table
        .as_mut()
        .unwrap()
        .character_profiles
        .get_mut(&f.characters[0])
        .unwrap();
    profile.money_cp += 1;
    profile.experience_points += 10;
    profile.description = "A changed narrative description".into();
    unrelated_change
        .entities
        .get_mut(&f.actors[0])
        .unwrap()
        .existence = EntityExistence::Dead;
    assert_eq!(
        dmd_rules::physical_facts::actor_identity(&unrelated_change, f.actors[0]).unwrap(),
        identity,
        "physical identity excludes unrelated wallet, experience, prose and death status"
    );
    unrelated_change
        .table
        .as_mut()
        .unwrap()
        .character_profiles
        .get_mut(&f.characters[0])
        .unwrap()
        .size = CharacterSize::Small;
    assert_ne!(
        dmd_rules::physical_facts::actor_identity(&unrelated_change, f.actors[0]).unwrap(),
        identity,
        "an actual changed body form needs a current fact"
    );
    assert!(current.table.as_ref().unwrap().grapple_access.is_none());
    assert!(
        current
            .table
            .as_ref()
            .unwrap()
            .source_actor_access
            .is_none()
    );
    assert_eq!(
        current.items[&ItemId(apparel.command_id.0)].definition_id,
        CUSTOM_LOAD_DEFINITION
    );
    assert_eq!(current.items[&ItemId(currency.command_id.0)].quantity, 178);
    assert_eq!(
        current.table.as_ref().unwrap().character_profiles[&f.characters[0]].money_cp,
        17170
    );
    assert_eq!(
        view(&f, &player(&f, 1)).await,
        unrelated,
        "private facts must not refresh unrelated views or handles"
    );
    let own = view(&f, &player(&f, 0)).await.physical.unwrap();
    assert!(own.controls.is_empty());
    assert_eq!(own.actors.len(), 1);
    assert_eq!(own.actors[0].load.as_ref(), Some(&complete));
    assert!(
        dmd_rules::table::CampaignExecution::from_original_anchor(
            current.clone(),
            dmd_rules::RulesPack::from_json(include_str!(
                "../../../../content/srd-5.2.1/kernel.json"
            ))
            .unwrap()
        )
        .is_err()
    );
    // An old accepted retry is stable, but a new old-envelope submission is not an activation bypass.
    for version in 1..=4 {
        let mut legacy = request(
            &f,
            TableTransportChannel::Host,
            action(TableAction::AddPlayer {
                id: PlayerId::new(),
                name: "Rejected old version".into(),
            }),
        )
        .await;
        legacy.version = version;
        Box::pin(reject(&f, legacy)).await;
    }
    let fields = PhysicalFactInput::Body {
        pounds: Some("180.5".into()),
        reason: "Correct measured unladen body weight.".into(),
    };
    let next = fact_request(&f, "body", "Mass character 0", fields.clone()).await;
    let mut wrong_session = next.clone();
    wrong_session.session_id = Some(PlaySessionId::new());
    Box::pin(reject(&f, wrong_session)).await;
    let mut wrong_channel = next.clone();
    wrong_channel.channel = player(&f, 0);
    Box::pin(reject(&f, wrong_channel)).await;
    let mut stale = body;
    stale.command_id = CommandId::new();
    Box::pin(reject(&f, stale)).await;
    let override_source = fact_request(
        &f,
        "item",
        "Glaive",
        PhysicalFactInput::Item {
            choice: PhysicalItemInput::Unit {
                pounds: "99".into(),
                condition: None,
            },
            reason: reason(),
        },
    )
    .await;
    Box::pin(reject(&f, override_source)).await;
    Box::pin(cold_step(&mut f, &path, next)).await;
    assert_eq!(
        load_for(&f, f.actors[0])
            .await
            .body_and_load_pounds
            .as_deref(),
        Some("213.685")
    );
    assert_eq!(view(&f, &player(&f, 1)).await, unrelated);
    let item_count = state(&f).await.items.len();
    Box::pin(fact(
        &mut f,
        &path,
        "currency",
        "Mass character 0",
        PhysicalFactInput::Currency {
            coins,
            reason: "Recount confirms exactly the same denominations in the same physical lot."
                .into(),
        },
    ))
    .await;
    assert_eq!(state(&f).await.items.len(), item_count);
    assert_eq!(
        state(&f).await.items[&ItemId(currency.command_id.0)],
        current.items[&ItemId(currency.command_id.0)]
    );
    Box::pin(hostile_history(&f)).await;
    Box::pin(close(f, &path)).await;
}

async fn hostile_history(f: &Fixture) {
    let original = export_campaign(&f.pool, f.campaign).await.unwrap();
    let pool = open_sqlite("sqlite::memory:").await.unwrap();
    let target = runtime(pool.clone());
    target
        .create_table_campaign(
            CampaignId::new(),
            "Unrelated preserved destination",
            TableContract::default(),
        )
        .await
        .unwrap();
    let before = all_rows(&pool).await;
    for mutation in 0..6 {
        let mut forged = original.clone();
        let mut current: CampaignState =
            serde_json::from_str(&forged.current_state.state_json).unwrap();
        match mutation {
            0 | 1 => {
                let facts = current.physical_facts.as_mut().unwrap();
                let record = facts
                    .records
                    .iter_mut()
                    .find(|fact| fact.subject == PhysicalSubject::Body(f.actors[0]))
                    .unwrap();
                if mutation == 0 {
                    let PhysicalFactValue::Body { mass, .. } = &mut record.value else {
                        unreachable!()
                    };
                    *mass = Some(PhysicalMass(1));
                } else {
                    record.origin.issuer = CommandIssuer::System;
                }
                forged.current_state.state_json = serde_json::to_string(&current).unwrap();
                let mut latest = forged.snapshots.last().unwrap().clone();
                latest.event_sequence = forged.current_state.applied_event_sequence;
                latest.state_json = forged.current_state.state_json.clone();
                forged.snapshots.push(latest);
            }
            2 => {
                let row = forged
                    .command_audit
                    .iter_mut()
                    .find(|row| {
                        row.payload_json.contains("PhysicalFact")
                            && row.payload_json.contains("180.5")
                    })
                    .unwrap();
                row.payload_json = row.payload_json.replace("180.5", "999.5");
            }
            3 => {
                let row = forged
                    .event_journal
                    .iter_mut()
                    .find(|row| {
                        row.payload_json.contains("PhysicalFact")
                            && row.payload_json.contains("180.5")
                    })
                    .unwrap();
                row.payload_json = row.payload_json.replace("180.5", "999.5");
            }
            4 => {
                forged.format_version = 2;
                forged.table_projection_history.clear();
                forged.table_transport_bindings.clear();
            }
            _ => {
                let needle = "\"physical_facts\":";
                assert!(forged.current_state.state_json.contains(needle));
                forged.current_state.state_json = forged.current_state.state_json.replacen(
                    needle,
                    "\"physical_facts\":null,\"physical_facts\":",
                    1,
                );
            }
        }
        assert!(
            Box::pin(target.restore_campaign(&forged)).await.is_err(),
            "forged mass history {mutation}"
        );
        assert_eq!(
            all_rows(&pool).await,
            before,
            "refusal {mutation} changed destination rows"
        );
    }
    Box::pin(target.restore_campaign(&original)).await.unwrap();
    assert_eq!(
        target.open_campaign(f.campaign).await.unwrap().state(),
        &state(f).await
    );
    pool.close().await;
}

#[tokio::test]
async fn actual_current_ogre_gets_nominal_source_load_and_authored_body_without_grapple_activation()
{
    let path = std::env::temp_dir().join(format!("dmd-mass-ogre-{}.sqlite", CommandId::new().0));
    let mut f = Box::pin(blank(&path)).await;
    Box::pin(create_pc(&mut f, &path, 0, &[("leather-armor", 1)])).await;
    Box::pin(v5_step(
        &mut f,
        &path,
        TableTransportChannel::Host,
        TableTransportInput::EnablePhysicalFacts,
    ))
    .await;
    let projected = view(&f, &TableTransportChannel::Host).await;
    let options = f
        .runtime
        .table_creature_options(TableCreatureOptionsRequest {
            campaign_id: f.campaign,
            channel: TableTransportChannel::Host,
            revision: projected.revision,
        })
        .await
        .unwrap();
    let ogre = options
        .iter()
        .find(|option| option.definition_id == "ogre")
        .unwrap();
    let actor = EntityId::new();
    let create = action(TableAction::CreateCreature {
        creation: Box::new(TableCreatureCreation {
            entity_id: actor,
            name: "Measured Ogre".into(),
            definition_id: ogre.definition_id.clone(),
            source: ogre.source.clone(),
            size: ogre.sizes[0],
            additional_languages: vec![],
            ammunition_units: 0,
            item_ids: (0..ogre.item_count).map(|_| ItemId::new()).collect(),
        }),
    });
    Box::pin(v5_step(&mut f, &path, TableTransportChannel::Host, create)).await;
    assert_eq!(load_for(&f, actor).await.known_pounds, "16");
    assert!(!load_for(&f, actor).await.complete);
    Box::pin(fact(&mut f, &path, "body", "Measured Ogre", PhysicalFactInput::Body { pounds: Some("600".into()), reason: "Authored actual unladen body measurement; this is not a source-inferred Ogre weight.".into() })).await;
    Box::pin(fact(
        &mut f,
        &path,
        "personal",
        "Measured Ogre",
        PhysicalFactInput::PersonalItem {
            name: "Separate Ogre cloak".into(),
            pounds: "3".into(),
            separate_from_listed: true,
            reason: reason(),
        },
    ))
    .await;
    Box::pin(fact(
        &mut f,
        &path,
        "coverage",
        "Measured Ogre",
        PhysicalFactInput::Coverage {
            complete: true,
            reason: "All equipment and the separate cloak are now listed.".into(),
        },
    ))
    .await;
    let load = load_for(&f, actor).await;
    assert!(load.complete);
    assert_eq!(load.known_pounds, "19");
    assert_eq!(load.body_and_load_pounds.as_deref(), Some("619"));
    Box::pin(v5_step(
        &mut f,
        &path,
        TableTransportChannel::Host,
        action(TableAction::EnableSourceActorAccess { adopted: vec![] }),
    ))
    .await;
    let controller = CreatureController::Player(f.players[0]);
    Box::pin(v5_step(
        &mut f,
        &path,
        TableTransportChannel::Host,
        action(TableAction::SetSourceCreatureController { actor, controller }),
    ))
    .await;
    let controlled = view(&f, &player(&f, 0)).await.physical.unwrap();
    assert!(
        controlled.actors.iter().all(|entry| entry.actor != actor),
        "source control does not reveal a creature's private body or gear facts"
    );
    let mut forbidden = fact_request(
        &f,
        "body",
        "Measured Ogre",
        PhysicalFactInput::Body {
            pounds: Some("601".into()),
            reason: reason(),
        },
    )
    .await;
    forbidden.channel = TableTransportChannel::SourceCreature {
        player_id: f.players[0],
        actor,
    };
    forbidden.revision = view(&f, &forbidden.channel).await.revision;
    Box::pin(reject(&f, forbidden)).await;

    assert!(
        view(&f, &player(&f, 1))
            .await
            .physical
            .unwrap()
            .actors
            .is_empty()
    );
    Box::pin(close(f, &path)).await;
}
async fn begin_v5(f: &mut Fixture, path: &Path) {
    Box::pin(begin_with_version(f, path, 5)).await;
}
async fn version_step(
    f: &mut Fixture,
    path: &Path,
    channel: TableTransportChannel,
    input: TableTransportInput,
    version: u32,
) -> TableTransportRequest {
    let mut value = request(f, channel, input).await;
    value.version = version;
    Box::pin(cold_step(f, path, value.clone())).await;
    value
}
async fn begin_with_version(f: &mut Fixture, path: &Path, version: u32) {
    let start = action(TableAction::StartSession {
        id: f.session,
        name: "Physical custody practice".into(),
        participants: (0..2)
            .map(|index| SessionParticipant {
                player_id: f.players[index],
                character_id: Some(f.characters[index]),
                attendance: AttendanceStatus::Present,
            })
            .collect(),
    });
    Box::pin(version_step(
        f,
        path,
        TableTransportChannel::Host,
        start,
        version,
    ))
    .await;
    let setup = action(TableAction::PrepareBattlefield {
        setup: Box::new(TableBattlefieldSetup {
            encounter_id: EncounterId::new(),
            scene_id: SceneId::new(),
            location_id: LocationId::new(),
            name: "Known level floor".into(),
            area_grid_policy: None,
            battlefield: Battlefield {
                bounds: SpatialBox {
                    min: SpatialPoint { x: 0, y: 0, z: 0 },
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
            characters: (0..2)
                .map(|index| TableCharacterPlacement {
                    character_id: f.characters[index],
                    position: SpatialPoint {
                        x: 10 + 10 * index as i32,
                        y: 10,
                        z: 0,
                    },
                    height: 12,
                    allies: vec![],
                    enemies: vec![f.actors[1 - index]],
                })
                .collect(),
            creatures: vec![],
            geometry_ruling: Ruling {
                basis: RulingBasis::GmAdjudication,
                reason:
                    "Two opposing participants stand five feet apart on the level visible floor."
                        .into(),
            },
        }),
    });
    Box::pin(version_step(
        f,
        path,
        TableTransportChannel::Host,
        setup,
        version,
    ))
    .await;
    let begin = tactical(TacticalAction::Begin {
        execution: TacticalExecutionVersion::EncounterReleaseV1,
        combatants: f
            .actors
            .iter()
            .map(|actor| TacticalCombatant {
                actor: *actor,
                source: TacticalSource::Character,
                surprised: false,
            })
            .collect(),
        groups: f
            .actors
            .iter()
            .map(|actor| InitiativeGroup {
                actors: vec![*actor],
                request_id: RollRequestId::new(),
            })
            .collect(),
    });
    Box::pin(version_step(
        f,
        path,
        TableTransportChannel::Host,
        begin,
        version,
    ))
    .await;
    Box::pin(roll_with_version(f, path, 0, 18, version)).await;
    Box::pin(roll_with_version(f, path, 1, 2, version)).await;
}
async fn roll_v5(f: &mut Fixture, path: &Path, index: usize, face: u16) {
    Box::pin(roll_with_version(f, path, index, face, 5)).await;
}
async fn roll_with_version(f: &mut Fixture, path: &Path, index: usize, face: u16, version: u32) {
    let channel = player(f, index);
    let offered = view(f, &channel).await.roll.unwrap();
    let sides = if offered.mode == RollMode::Normal {
        offered
            .dice
            .iter()
            .flat_map(|die| std::iter::repeat_n(die.sides, usize::from(die.count)))
            .collect::<Vec<_>>()
    } else {
        vec![20, 20]
    };
    let input = tactical(TacticalAction::SubmitRoll {
        result: RollResult {
            request_id: offered.id,
            source: RollSource::Physical,
            dice: sides
                .into_iter()
                .map(|sides| DieResult {
                    sides,
                    value: face.min(sides),
                })
                .collect(),
        },
    });
    Box::pin(version_step(f, path, channel, input, version)).await;
}
#[tokio::test]
async fn nonstandard_same_item_follows_real_throw_and_pickup_without_invalidating_coverage_or_ownership()
 {
    Box::pin(custody_case()).await;
}
async fn custody_case() {
    let path = std::env::temp_dir().join(format!("dmd-mass-custody-{}.sqlite", CommandId::new().0));
    let mut f = Box::pin(blank(&path)).await;
    Box::pin(create_pc(
        &mut f,
        &path,
        0,
        &[("leather-armor", 1), ("dagger", 1)],
    ))
    .await;
    Box::pin(create_pc(&mut f, &path, 1, &[("leather-armor", 1)])).await;
    Box::pin(materialize(&mut f, &path, 0)).await;
    Box::pin(materialize(&mut f, &path, 1)).await;
    let mass_activation = Box::pin(v5_step(
        &mut f,
        &path,
        TableTransportChannel::Host,
        TableTransportInput::EnablePhysicalFacts,
    ))
    .await;
    Box::pin(assert_mass_capabilities(&f, &mass_activation, None)).await;
    let dagger = state(&f)
        .await
        .items
        .values()
        .find(|item| item.definition_id == "dagger")
        .unwrap()
        .clone();
    let classification=Box::pin(fact(&mut f,&path,"item","Dagger",PhysicalFactInput::Item { choice:PhysicalItemInput::NonstandardUnit { description:"This particular dagger has a heavy custom pommel; its combat properties are unchanged.".into(),pounds:"3.25".into() },reason:reason() })).await;
    for index in 0..2 {
        let label = format!("Mass character {index}");
        Box::pin(fact(
            &mut f,
            &path,
            "body",
            &label,
            PhysicalFactInput::Body {
                pounds: Some("150".into()),
                reason: reason(),
            },
        ))
        .await;
        Box::pin(fact(
            &mut f,
            &path,
            "currency",
            &label,
            PhysicalFactInput::Currency {
                coins: PhysicalCoins {
                    cp: 0,
                    sp: 0,
                    ep: 0,
                    gp: if index == 0 { 193 } else { 195 },
                    pp: 0,
                },
                reason: "Explicit actual gold coins realize the unchanged wallet balance.".into(),
            },
        ))
        .await;
        Box::pin(fact(
            &mut f,
            &path,
            "coverage",
            &label,
            PhysicalFactInput::Coverage {
                complete: true,
                reason: "All actual additional apparel and payload is already listed.".into(),
            },
        ))
        .await;
    }
    let initial = state(&f).await;
    let facts = initial.physical_facts.clone();
    assert_eq!(
        initial.items[&dagger.id], dagger,
        "classification must not edit item identity, custody or combat definition"
    );
    assert_eq!(load_for(&f, f.actors[0]).await.known_pounds, "17.11");
    let stale = fact_request(
        &f,
        "body",
        "Mass character 0",
        PhysicalFactInput::Body {
            pounds: Some("151".into()),
            reason: reason(),
        },
    )
    .await;
    Box::pin(begin_v5(&mut f, &path)).await;
    assert!(
        view(&f, &TableTransportChannel::Host)
            .await
            .physical
            .unwrap()
            .controls
            .is_empty()
    );
    Box::pin(reject(&f, stale)).await;
    // Mass-only v5 does not grant Grapple/Ground permissions.
    let false_transport = v5_request(
        &f,
        TableTransportChannel::Host,
        action(TableAction::EnableGrappleTransport),
    )
    .await;
    Box::pin(reject(&f, false_transport)).await;
    Box::pin(v5_step(
        &mut f,
        &path,
        TableTransportChannel::Host,
        tactical(TacticalAction::ActivateAttackEquipment),
    ))
    .await;
    let thrown = WeaponUseChoice {
        weapon: dagger.id,
        target: f.actors[1],
        delivery: WeaponDelivery::Thrown,
        ability: Ability::Strength,
        grip: WeaponGrip::OneHand(Hand::Right),
        purpose: WeaponAttackPurpose::Normal,
        ammunition: None,
        after_equipment: None,
        equipment_change: Some(AttackEquipmentChange {
            timing: EquipmentChangeTiming::BeforeAttack,
            operation: AttackEquipmentOperation::Equip {
                item: dagger.id,
                hand: Hand::Right,
            },
        }),
    };
    let channel = player(&f, 0);
    Box::pin(v5_step(
        &mut f,
        &path,
        channel,
        tactical(TacticalAction::Attack { choice: thrown }),
    ))
    .await;
    Box::pin(roll_v5(&mut f, &path, 0, 1)).await;
    let released = state(&f).await;
    assert!(matches!(
        released.items[&dagger.id].custody,
        Custody::Location(_)
    ));
    assert_eq!(released.items[&dagger.id].owner, dagger.owner);
    assert_eq!(released.physical_facts, facts);
    assert!(load_for(&f, f.actors[0]).await.complete);
    assert_eq!(load_for(&f, f.actors[0]).await.known_pounds, "13.86");
    // Genuine later feature activations retain their own predicates under v5.
    let grapple_activation = Box::pin(v5_step(
        &mut f,
        &path,
        TableTransportChannel::Host,
        action(TableAction::EnableGrappleAccess),
    ))
    .await;
    Box::pin(assert_mass_capabilities(&f, &grapple_activation, Some(3))).await;
    let ground_activation = Box::pin(v5_step(
        &mut f,
        &path,
        TableTransportChannel::Host,
        action(TableAction::EnableGrappleTransport),
    ))
    .await;
    Box::pin(assert_mass_capabilities(&f, &ground_activation, Some(4))).await;
    let channel = player(&f, 0);
    Box::pin(v5_step(
        &mut f,
        &path,
        channel,
        tactical(TacticalAction::EndTurn),
    ))
    .await;
    let pickup = WeaponUseChoice {
        weapon: dagger.id,
        target: f.actors[0],
        delivery: WeaponDelivery::Melee,
        ability: Ability::Strength,
        grip: WeaponGrip::OneHand(Hand::Right),
        purpose: WeaponAttackPurpose::Normal,
        ammunition: None,
        after_equipment: None,
        equipment_change: Some(AttackEquipmentChange {
            timing: EquipmentChangeTiming::BeforeAttack,
            operation: AttackEquipmentOperation::Pickup {
                item: dagger.id,
                hand: Hand::Right,
            },
        }),
    };
    let channel = player(&f, 1);
    Box::pin(v5_step(
        &mut f,
        &path,
        channel,
        tactical(TacticalAction::Attack { choice: pickup }),
    ))
    .await;
    let picked = state(&f).await;
    assert_eq!(
        picked.items[&dagger.id].custody,
        Custody::Entity(f.actors[1])
    );
    assert_eq!(picked.items[&dagger.id].owner, dagger.owner);
    assert_eq!(picked.items[&dagger.id].quantity, dagger.quantity);
    assert_eq!(picked.physical_facts, facts);
    assert_eq!(
        picked
            .physical_facts
            .as_ref()
            .unwrap()
            .get(PhysicalSubject::Item(dagger.id))
            .unwrap()
            .origin
            .id,
        classification.command_id
    );
    assert!(load_for(&f, f.actors[0]).await.complete);
    assert!(load_for(&f, f.actors[1]).await.complete);
    assert_eq!(load_for(&f, f.actors[1]).await.known_pounds, "17.15");
    let private = view(&f, &player(&f, 1)).await.physical.unwrap();
    assert!(
        private.actors[0].load.is_none(),
        "an aggregate must not disclose another owner's private physical item fact"
    );
    Box::pin(roll_v5(&mut f, &path, 1, 1)).await;
    assert_eq!(state(&f).await.physical_facts, facts);
    Box::pin(close(f, &path)).await;
}

#[tokio::test]
async fn mass_only_paid_attack_after_equipment_apply_and_decline_keep_the_exact_owned_candidate() {
    for apply in [false, true] {
        Box::pin(after_equipment_case(apply)).await;
    }
}
async fn after_equipment_case(apply: bool) {
    let path = std::env::temp_dir().join(format!(
        "dmd-mass-after-equipment-{}.sqlite",
        CommandId::new().0
    ));
    let mut f = Box::pin(blank(&path)).await;
    Box::pin(create_pc(
        &mut f,
        &path,
        0,
        &[("leather-armor", 1), ("dagger", 1)],
    ))
    .await;
    Box::pin(create_pc(&mut f, &path, 1, &[("leather-armor", 1)])).await;
    Box::pin(materialize(&mut f, &path, 0)).await;
    Box::pin(materialize(&mut f, &path, 1)).await;
    Box::pin(v5_step(
        &mut f,
        &path,
        TableTransportChannel::Host,
        TableTransportInput::EnablePhysicalFacts,
    ))
    .await;
    let dagger = state(&f)
        .await
        .items
        .values()
        .find(|item| item.definition_id == "dagger")
        .unwrap()
        .clone();
    Box::pin(begin_v5(&mut f, &path)).await;
    Box::pin(v5_step(
        &mut f,
        &path,
        TableTransportChannel::Host,
        tactical(TacticalAction::ActivateAttackEquipment),
    ))
    .await;
    let equipped_attack = WeaponUseChoice {
        weapon: dagger.id,
        target: f.actors[1],
        delivery: WeaponDelivery::Melee,
        ability: Ability::Strength,
        grip: WeaponGrip::OneHand(Hand::Right),
        purpose: WeaponAttackPurpose::Normal,
        ammunition: None,
        after_equipment: None,
        equipment_change: Some(AttackEquipmentChange {
            timing: EquipmentChangeTiming::BeforeAttack,
            operation: AttackEquipmentOperation::Equip {
                item: dagger.id,
                hand: Hand::Right,
            },
        }),
    };
    let channel = player(&f, 0);
    Box::pin(v5_step(
        &mut f,
        &path,
        channel.clone(),
        tactical(TacticalAction::Attack {
            choice: equipped_attack.clone(),
        }),
    ))
    .await;
    Box::pin(roll_v5(&mut f, &path, 0, 1)).await;
    assert!(
        state(&f)
            .await
            .encounter
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .resolution
            .is_none()
    );
    for index in 0..2 {
        let actor_channel = player(&f, index);
        Box::pin(v5_step(
            &mut f,
            &path,
            actor_channel,
            tactical(TacticalAction::EndTurn),
        ))
        .await;
    }
    // The first genuine paid attack equipped the dagger. This later attack has
    // only the after-attack intent, exactly as the existing producer requires.
    let choice = WeaponUseChoice {
        after_equipment: Some(AfterAttackEquipmentIntent::Choose),
        equipment_change: None,
        ..equipped_attack
    };
    Box::pin(v5_step(
        &mut f,
        &path,
        channel.clone(),
        tactical(TacticalAction::Attack { choice }),
    ))
    .await;
    Box::pin(roll_v5(&mut f, &path, 0, 1)).await;
    let before = state(&f).await;
    assert!(before.table.as_ref().unwrap().grapple_access.is_none());
    let offered = view(&f, &channel)
        .await
        .tactical
        .unwrap()
        .attack_equipment
        .unwrap();
    let operation = AttackEquipmentOperation::Unequip { item: dagger.id };
    assert!(
        offered
            .operations
            .iter()
            .any(|entry| entry.operation == operation)
    );
    Box::pin(v5_step(
        &mut f,
        &path,
        channel,
        TableTransportInput::AttackEquipment {
            handle: offered.key,
            choice: if apply {
                AttackEquipmentChoice::Apply(operation)
            } else {
                AttackEquipmentChoice::Decline
            },
        },
    ))
    .await;
    let after = state(&f).await;
    assert_eq!(after.physical_facts, before.physical_facts);
    assert_eq!(after.items[&dagger.id], dagger);
    assert!(
        after
            .rules
            .as_ref()
            .unwrap()
            .timing
            .as_ref()
            .unwrap()
            .action_spent
    );
    assert!(
        after
            .encounter
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .resolution
            .is_none()
    );
    let loadout = after
        .rules
        .as_ref()
        .unwrap()
        .tactical_inventory
        .as_ref()
        .unwrap()
        .loadout(f.actors[0])
        .unwrap();
    assert_eq!(
        loadout
            .hands
            .hands
            .contains(&HandAssignment::Item(dagger.id)),
        !apply
    );
    assert_eq!(load_for(&f, f.actors[0]).await.known_pounds, "11");
    Box::pin(close(f, &path)).await;
}

#[tokio::test]
async fn current_v4_finished_encounter_activates_mass_then_v5_inspiration_keeps_original_authority()
{
    Box::pin(v4_activation_case()).await;
}
async fn v4_activation_case() {
    // Current-build v4 production, explicitly not the independent historical corpus.
    let path = std::env::temp_dir().join(format!("dmd-mass-v4-{}.sqlite", CommandId::new().0));
    let mut f = Box::pin(blank(&path)).await;
    for index in 0..2 {
        Box::pin(create_pc(&mut f, &path, index, &[("leather-armor", 1)])).await;
        Box::pin(materialize(&mut f, &path, index)).await;
    }
    Box::pin(begin_with_version(&mut f, &path, 1)).await;
    Box::pin(version_step(
        &mut f,
        &path,
        TableTransportChannel::Host,
        action(TableAction::EnableGrappleAccess),
        3,
    ))
    .await;
    Box::pin(version_step(
        &mut f,
        &path,
        TableTransportChannel::Host,
        action(TableAction::EnableGrappleTransport),
        4,
    ))
    .await;
    let forbidden = v5_request(
        &f,
        TableTransportChannel::Host,
        TableTransportInput::EnablePhysicalFacts,
    )
    .await;
    Box::pin(reject(&f, forbidden)).await;
    let original=Box::pin(version_step(&mut f,&path,TableTransportChannel::Host,tactical(TacticalAction::ConcludeHostilities { cadence:AftermathCadence::ContinueExistingOrder,ruling:"The actual current encounter is idle and the participants have concluded hostilities.".into() }),4)).await;
    let response = Box::pin(f.runtime.submit_presented_table(original.clone()))
        .await
        .unwrap();
    Box::pin(version_step(
        &mut f,
        &path,
        TableTransportChannel::Host,
        tactical(TacticalAction::FinishEncounter),
        4,
    ))
    .await;
    assert!(state(&f).await.physical_facts.is_none());
    let mass_activation = Box::pin(v5_step(
        &mut f,
        &path,
        TableTransportChannel::Host,
        TableTransportInput::EnablePhysicalFacts,
    ))
    .await;
    Box::pin(assert_mass_capabilities(&f, &mass_activation, Some(4))).await;
    assert_eq!(
        Box::pin(f.runtime.submit_presented_table(original))
            .await
            .unwrap(),
        response,
        "saved pre-activation retries retain their exact original response"
    );
    let attachment = state(&f).await.physical_facts;
    let character_id = f.characters[0];
    Box::pin(v5_step(
        &mut f,
        &path,
        TableTransportChannel::Host,
        action(TableAction::AwardHeroicInspiration {
            character_id,
            reason: "For the participant's careful cooperation.".into(),
        }),
    ))
    .await;
    assert!(state(&f).await.rules.as_ref().unwrap().entities[&f.actors[0]].heroic_inspiration);
    Box::pin(v5_step(
        &mut f,
        &path,
        TableTransportChannel::Host,
        action(TableAction::AwardExcessInspiration {
            character_id,
            reason: "For returning to aid a companion.".into(),
        }),
    ))
    .await;
    assert!(
        view(&f, &TableTransportChannel::Host)
            .await
            .physical
            .unwrap()
            .controls
            .is_empty()
    );
    let channel = player(&f, 0);
    let transfer = view(&f, &channel).await.inspiration_transfer.unwrap();
    let handle = transfer
        .choices
        .iter()
        .find(|choice| choice.label == "Decline the extra Inspiration")
        .unwrap()
        .key;
    Box::pin(v5_step(
        &mut f,
        &path,
        channel,
        TableTransportInput::InspirationTransfer { handle },
    ))
    .await;
    let accepted = state(&f).await;
    assert_eq!(accepted.physical_facts, attachment);
    assert!(
        accepted
            .table
            .as_ref()
            .unwrap()
            .inspiration_transfer
            .is_none()
    );
    assert!(
        accepted
            .table
            .as_ref()
            .unwrap()
            .grapple_access
            .as_ref()
            .unwrap()
            .ground_transport
            .is_some()
    );
    assert!(
        !view(&f, &TableTransportChannel::Host)
            .await
            .physical
            .unwrap()
            .controls
            .is_empty()
    );
    let mut old = v5_request(
        &f,
        TableTransportChannel::Host,
        action(TableAction::AwardHeroicInspiration {
            character_id: f.characters[1],
            reason: "A fresh old envelope cannot bypass mass versioning.".into(),
        }),
    )
    .await;
    old.version = 4;
    Box::pin(reject(&f, old)).await;
    Box::pin(close(f, &path)).await;
}
