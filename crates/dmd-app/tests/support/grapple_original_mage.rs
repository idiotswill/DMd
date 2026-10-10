//! Continue an unchanged original journal before using its Mage in current play.
use super::*;

fn flow(state: &CampaignState) -> &TacticalFlow {
    state.encounter.as_ref().unwrap().flow.as_ref().unwrap()
}

impl Fixture {
    pub(super) async fn with_original_mage() -> Self {
        let bytes = std::fs::read(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures/shield-hit-v1-selected.json"),
        )
        .unwrap();
        let original = CampaignExport::from_json(std::str::from_utf8(&bytes).unwrap()).unwrap();
        assert_eq!(original.to_json().unwrap().as_bytes(), bytes);
        let state = CampaignState::decode_json(&original.current_state.state_json).unwrap();
        assert_eq!(flow(&state).version, 3);
        let hit = flow(&state)
            .resolution
            .as_ref()
            .unwrap()
            .hit_review
            .as_ref()
            .unwrap();
        assert_eq!(hit.stage, TacticalHitReviewStage::Selected);
        let mage = hit.respondent.as_ref().unwrap().actor;
        let creatures = state
            .rules
            .as_ref()
            .unwrap()
            .tactical_creatures
            .as_ref()
            .unwrap();
        let old = creatures.profile(mage).unwrap().clone();
        assert_eq!(old.source.definition_fingerprint, "af0f81ba7833b9c4");
        assert_eq!(old.size, CreatureSize::Medium);
        let CreatureController::Player(owner) = creatures.runtime(mage).unwrap().controller else {
            panic!("the original Mage retains its original player owner");
        };
        let directory =
            std::env::temp_dir().join(format!("dmd-grapple-old-mage-{}", CampaignId::new().0));
        std::fs::create_dir(&directory).unwrap();
        let path = directory.join("campaign.sqlite");
        let pool = open_sqlite_path(&path).await.unwrap();
        let mut f = Self {
            runtime: runtime(pool.clone()),
            pool,
            directory,
            path,
            campaign: state.campaign_id(),
            players: [PlayerId::new(), PlayerId::new()],
            characters: [CharacterId::new(), CharacterId::new()],
            actors: [EntityId::new(), EntityId::new()],
            session: state
                .table
                .as_ref()
                .unwrap()
                .active_session
                .as_ref()
                .unwrap()
                .session_id,
            goblin: mage,
            opponent: None,
        };
        Box::pin(f.runtime.restore_campaign(&original))
            .await
            .unwrap();
        let mut restored = export_campaign(&f.pool, f.campaign).await.unwrap();
        restored
            .exported_at_utc
            .clone_from(&original.exported_at_utc);
        assert_eq!(restored, original);
        // Continue the actual original selected response, including its paid use.
        let channel = TableTransportChannel::SourceCreature {
            player_id: owner,
            actor: mage,
        };
        let shown = f.view(channel.clone()).await;
        let response = shown
            .tactical
            .as_ref()
            .unwrap()
            .hit
            .as_ref()
            .unwrap()
            .response
            .as_ref()
            .unwrap();
        let choice = response.shield[0].clone();
        assert_eq!(choice.spell_id, "shield");
        Box::pin(f.send(
            channel,
            TableTransportInput::HitResponse {
                handle: response.key,
                decision: Box::new(TableHitInput::Cast { choice }),
            },
        ))
        .await;
        let settled = f.state().await;
        assert!(flow(&settled).resolution.is_none());
        assert_eq!(
            settled.rules.as_ref().unwrap().rolls,
            state.rules.as_ref().unwrap().rolls
        );
        assert_eq!(
            settled
                .rules
                .as_ref()
                .unwrap()
                .tactical_creatures
                .as_ref()
                .unwrap()
                .profile(mage),
            Some(&old)
        );
        assert_eq!(
            dmd_rules::tactical_defenses::effective_armor_class(&settled, mage).unwrap(),
            17
        );
        Box::pin(f.host(TableAction::Tactical {
            action: TacticalAction::UpgradeExecutionTo {
                execution: TacticalExecutionVersion::EncounterReleaseV1,
            },
        }))
        .await;
        let upgraded = f.state().await;
        let timing = upgraded.rules.as_ref().unwrap().timing.as_ref().unwrap();
        let current = timing.order[timing.index].actor;
        let pc = upgraded
            .characters
            .values()
            .find(|pc| pc.entity_id == current)
            .unwrap();
        Box::pin(f.send(
            TableTransportChannel::Player {
                player_id: pc.controlling_player_id.unwrap(),
                character_id: pc.id,
            },
            action(TacticalAction::EndTurn),
        ))
        .await;
        Box::pin(f.host(TableAction::Tactical {
            action: TacticalAction::ConcludeHostilities {
                cadence: AftermathCadence::ContinueExistingOrder,
                ruling: "The original paid Shield and turn have settled; end this conflict.".into(),
            },
        }))
        .await;
        dmd_rules::tactical::encounter_release_preflight(&f.state().await).unwrap();
        Box::pin(f.host(TableAction::Tactical {
            action: TacticalAction::FinishEncounter,
        }))
        .await;
        assert_eq!(flow(&f.state().await).phase, TacticalPhase::Finished);
        Box::pin(f.host(TableAction::EndSession)).await;

        // Fresh ordinary PCs make the current Grapple setup independent of the
        // old attacker's ability scores or equipment, without modifying history.
        for index in 0..2 {
            Box::pin(f.host(TableAction::AddPlayer {
                id: f.players[index],
                name: format!("Current player {index}"),
            }))
            .await;
            Box::pin(f.host(TableAction::CreateCharacter {
                character_id: f.characters[index],
                entity_id: f.actors[index],
                player_id: f.players[index],
                input: creation(&format!("Current character {index}")),
            }))
            .await;
            let shown = f.view(TableTransportChannel::Host).await;
            let count = shown
                .characters
                .iter()
                .find(|pc| pc.character_id == f.characters[index])
                .unwrap()
                .equipment
                .as_ref()
                .unwrap()
                .initial_item_count;
            Box::pin(f.host(TableAction::PrepareEquipment {
                character_id: f.characters[index],
                item_ids: (0..count).map(|_| ItemId::new()).collect(),
            }))
            .await;
        }
        let shown = f.view(TableTransportChannel::Host).await;
        let adoption = f
            .runtime
            .table_source_control_options(TableCreatureOptionsRequest {
                campaign_id: f.campaign,
                channel: TableTransportChannel::Host,
                revision: shown.revision,
            })
            .await
            .unwrap();
        assert!(adoption.enabled && adoption.settled);
        assert_eq!(
            f.state().await.table.as_ref().unwrap().source_actor_access,
            state.table.as_ref().unwrap().source_actor_access
        );
        Box::pin(f.host(TableAction::SetSourceCreatureController {
            actor: mage,
            controller: CreatureController::Host,
        }))
        .await;
        f.session = PlaySessionId::new();
        Box::pin(
            f.host(TableAction::StartSession {
                id: f.session,
                name: "Current play with the original Mage".into(),
                participants: (0..2)
                    .map(|i| SessionParticipant {
                        player_id: f.players[i],
                        character_id: Some(f.characters[i]),
                        attendance: AttendanceStatus::Present,
                    })
                    .collect(),
            }),
        )
        .await;
        Box::pin(f.host(TableAction::PrepareBattlefield {
            setup: Box::new(TableBattlefieldSetup {
                encounter_id: EncounterId::new(),
                scene_id: SceneId::new(),
                location_id: LocationId::new(),
                name: "Courtyard".into(),
                battlefield: Battlefield {
                    bounds: SpatialBox {
                        min: SpatialPoint { x: 0, y: 0, z: 0 },
                        max: SpatialPoint {
                            x: 100,
                            y: 100,
                            z: 60,
                        },
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
                    position: SpatialPoint { x: 10, y: 10, z: 0 },
                    height: 12,
                    allies: vec![],
                    enemies: vec![mage],
                }],
                creatures: vec![TableCreaturePlacement {
                    actor: mage,
                    public_label: "Small armored figure".into(),
                    position: SpatialPoint { x: 20, y: 10, z: 0 },
                    height: 8,
                    allies: vec![],
                    enemies: vec![f.actors[0]],
                }],
                geometry_ruling: Ruling {
                    basis: RulingBasis::GmAdjudication,
                    reason: "Host establishes the visible courtyard.".into(),
                },
                area_grid_policy: None,
            }),
        }))
        .await;
        Box::pin(
            f.host(TableAction::Tactical {
                action: TacticalAction::Begin {
                    execution: TacticalExecutionVersion::EncounterReleaseV1,
                    combatants: vec![
                        TacticalCombatant {
                            actor: f.actors[0],
                            source: TacticalSource::Character,
                            surprised: false,
                        },
                        TacticalCombatant {
                            actor: mage,
                            source: TacticalSource::Creature {
                                definition_id: "mage".into(),
                            },
                            surprised: false,
                        },
                    ],
                    groups: [f.actors[0], mage]
                        .into_iter()
                        .map(|actor| InitiativeGroup {
                            actors: vec![actor],
                            request_id: RollRequestId::new(),
                        })
                        .collect(),
                },
            }),
        )
        .await;
        Box::pin(f.roll(f.pc(0), 18)).await;
        Box::pin(f.roll(TableTransportChannel::Host, 2)).await;
        let ready = f.state().await;
        assert_eq!(flow(&ready).phase, TacticalPhase::Active);
        assert_eq!(
            ready
                .rules
                .as_ref()
                .unwrap()
                .tactical_creatures
                .as_ref()
                .unwrap()
                .profile(mage),
            Some(&old)
        );
        let accepted = export_campaign(&f.pool, f.campaign).await.unwrap();
        assert!(accepted.event_journal.starts_with(&original.event_journal));
        assert!(
            accepted
                .table_projection_history
                .starts_with(&original.table_projection_history)
        );
        for binding in &original.table_transport_bindings {
            assert!(accepted.table_transport_bindings.contains(binding));
        }
        f
    }
}
