//! Original accepted creation, physical equipment and opaque choices. No saved
//! state, source grant, roll authority or tactical projection is injected.
use dmd_app::*;
use dmd_domain::*;
use dmd_persistence::{CampaignExport, export_campaign, open_sqlite, open_sqlite_path};
use dmd_rules::{CharacterCreationInput, RulesQuery, tactical::TacticalAction};
use sqlx::Row;
use std::path::{Path, PathBuf};

async fn all_rows(pool: &sqlx::SqlitePool) -> Vec<(String, Vec<Vec<String>>)> {
    let tables = sqlx::query_scalar::<_, String>(
        "SELECT name FROM sqlite_master WHERE type='table' ORDER BY name",
    )
    .fetch_all(pool)
    .await
    .unwrap();
    let mut result = vec![];
    for table in tables {
        let quoted = format!("\"{}\"", table.replace('"', "\"\""));
        let columns = sqlx::query(&format!("PRAGMA table_info({quoted})"))
            .fetch_all(pool)
            .await
            .unwrap();
        let expressions = columns
            .iter()
            .map(|column| {
                let name: String = column.get("name");
                let name = format!("\"{}\"", name.replace('"', "\"\""));
                format!("typeof({name}) || ':' || hex(CAST({name} AS BLOB))")
            })
            .collect::<Vec<_>>();
        let mut rows = sqlx::query(&format!("SELECT {} FROM {quoted}", expressions.join(",")))
            .fetch_all(pool)
            .await
            .unwrap()
            .iter()
            .map(|row| {
                (0..expressions.len())
                    .map(|index| row.get::<String, _>(index))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        rows.sort();
        result.push((table, rows));
    }
    result
}

async fn hostile_destination(export: &CampaignExport) {
    let pool = open_sqlite("sqlite::memory:").await.unwrap();
    let destination = runtime(pool.clone());
    destination
        .create_table_campaign(
            CampaignId::new(),
            "Unrelated real destination",
            TableContract::default(),
        )
        .await
        .unwrap();
    let before = all_rows(&pool).await;
    assert!(
        Box::pin(destination.restore_campaign(export))
            .await
            .is_err()
    );
    assert_eq!(
        all_rows(&pool).await,
        before,
        "hostile restore changed a destination table"
    );
    pool.close().await;
}

#[allow(dead_code)] // This shared test cleanup also supplies a file-only variant.
#[path = "support/sqlite_test_cleanup.rs"]
mod cleanup;

fn runtime(pool: sqlx::SqlitePool) -> CampaignRuntime {
    CampaignRuntime::from_content_root(
        pool,
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../content"),
    )
}
fn creation(name: &str) -> CharacterCreationInput {
    CharacterCreationInput {
        name: name.into(),
        pronouns: "they/them".into(),
        description: "A traveler".into(),
        alignment: "Neutral Good".into(),
        backstory: "A private goal".into(),
        ability_scores: [15, 14, 13, 8, 10, 12],
        background_boosts: [2, 0, 1, 0, 0, 0],
        fighter_skills: [Skill::Perception, Skill::Survival],
        human_skill: Skill::Insight,
        skilled_skills: [Skill::Acrobatics, Skill::Stealth, Skill::Investigation],
        size: CharacterSize::Medium,
        languages: ["dwarvish".into(), "elvish".into()],
        fighting_style: FightingStyle::Defense,
        gaming_set: GamingSet::Dice,
        purchases: ["leather-armor", "dagger", "club", "shortbow"]
            .into_iter()
            .map(|item_id| EquipmentChoice {
                item_id: item_id.into(),
                quantity: 1,
            })
            .collect(),
        worn_armor: Some("leather-armor".into()),
        shield: false,
        masteries: ["club".into(), "dagger".into(), "shortbow".into()],
    }
}
fn action(action: TacticalAction) -> TableTransportInput {
    TableTransportInput::Action(Box::new(TableAction::Tactical { action }))
}

struct Fixture {
    pool: sqlx::SqlitePool,
    runtime: CampaignRuntime,
    directory: PathBuf,
    path: PathBuf,
    campaign: CampaignId,
    players: [PlayerId; 2],
    characters: [CharacterId; 2],
    actors: [EntityId; 2],
    session: PlaySessionId,
    goblin: EntityId,
    opponent: Option<EntityId>,
}
impl Fixture {
    async fn new() -> Self {
        Box::pin(Self::with_source("goblin-warrior", CreatureSize::Small)).await
    }
    async fn with_source(definition: &str, size: CreatureSize) -> Self {
        Box::pin(Self::with_opponent(definition, size, false)).await
    }
    async fn with_opponent(definition: &str, size: CreatureSize, opponent: bool) -> Self {
        Box::pin(Self::with_opponent_geometry(
            definition, size, opponent, false,
        ))
        .await
    }
    async fn with_opponent_geometry(
        definition: &str,
        size: CreatureSize,
        opponent: bool,
        pc_opportunity: bool,
    ) -> Self {
        Box::pin(Self::with_geometry(
            definition,
            size,
            opponent,
            pc_opportunity,
            false,
        ))
        .await
    }
    async fn with_geometry(
        definition: &str,
        size: CreatureSize,
        opponent: bool,
        pc_opportunity: bool,
        pc_platform: bool,
    ) -> Self {
        let directory =
            std::env::temp_dir().join(format!("dmd-grapple-public-{}", CampaignId::new().0));
        std::fs::create_dir(&directory).unwrap();
        let path = directory.join("campaign.sqlite");
        let pool = open_sqlite_path(&path).await.unwrap();
        let f = Self {
            runtime: runtime(pool.clone()),
            pool,
            directory,
            path,
            campaign: CampaignId::new(),
            players: [PlayerId::new(), PlayerId::new()],
            characters: [CharacterId::new(), CharacterId::new()],
            actors: [EntityId::new(), EntityId::new()],
            session: PlaySessionId::new(),
            goblin: EntityId::new(),
            opponent: opponent.then(EntityId::new),
        };
        f.runtime
            .create_table_campaign(f.campaign, "Grapple journal", TableContract::default())
            .await
            .unwrap();
        for index in 0..2 {
            Box::pin(f.host(TableAction::AddPlayer {
                id: f.players[index],
                name: format!("Player {index}"),
            }))
            .await;
            Box::pin(f.host(TableAction::CreateCharacter {
                character_id: f.characters[index],
                entity_id: f.actors[index],
                player_id: f.players[index],
                input: creation(&format!("Character {index}")),
            }))
            .await;
        }
        let current = f.view(TableTransportChannel::Host).await;
        let catalog = f
            .runtime
            .table_creature_options(TableCreatureOptionsRequest {
                campaign_id: f.campaign,
                channel: TableTransportChannel::Host,
                revision: current.revision,
            })
            .await
            .unwrap();
        let source = catalog
            .iter()
            .find(|source| source.definition_id == definition)
            .unwrap();
        Box::pin(f.host(TableAction::CreateCreature {
            creation: Box::new(TableCreatureCreation {
                entity_id: f.goblin,
                name: "Private sentry".into(),
                definition_id: source.definition_id.clone(),
                source: source.source.clone(),
                size,
                additional_languages: if definition == "mage" {
                    vec!["dwarvish".into(), "elvish".into(), "draconic".into()]
                } else {
                    vec![]
                },
                ammunition_units: if definition == "goblin-warrior" {
                    20
                } else {
                    0
                },
                item_ids: (0..source.item_count).map(|_| ItemId::new()).collect(),
            }),
        }))
        .await;
        if let Some(actor) = f.opponent {
            let source = catalog
                .iter()
                .find(|source| source.definition_id == "goblin-warrior")
                .unwrap();
            Box::pin(f.host(TableAction::CreateCreature {
                creation: Box::new(TableCreatureCreation {
                    entity_id: actor,
                    name: "Independent sentry".into(),
                    definition_id: source.definition_id.clone(),
                    source: source.source.clone(),
                    size: CreatureSize::Small,
                    additional_languages: vec![],
                    ammunition_units: 20,
                    item_ids: (0..source.item_count).map(|_| ItemId::new()).collect(),
                }),
            }))
            .await;
        }
        Box::pin(f.host(TableAction::EnableSourceActorAccess { adopted: vec![] })).await;
        Box::pin(f.host(TableAction::SetSourceCreatureController {
            actor: f.goblin,
            controller: CreatureController::Host,
        }))
        .await;
        if let Some(actor) = f.opponent {
            Box::pin(f.host(TableAction::SetSourceCreatureController {
                actor,
                controller: CreatureController::Host,
            }))
            .await;
        }
        for index in 0..2 {
            let view = f.view(TableTransportChannel::Host).await;
            let count = view
                .characters
                .iter()
                .find(|c| c.character_id == f.characters[index])
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
        Box::pin(
            f.host(TableAction::StartSession {
                id: f.session,
                name: "Grapple session".into(),
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
        Box::pin(
            f.host(TableAction::PrepareBattlefield {
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
                        obstacles: if pc_platform {
                            vec![SpatialObstacle {
                                id: "grappler-platform".into(),
                                volume: SpatialBox {
                                    min: SpatialPoint { x: 10, y: 10, z: 0 },
                                    max: SpatialPoint {
                                        x: 20,
                                        y: 20,
                                        z: 10,
                                    },
                                },
                                blocks_movement: true,
                                blocks_sight: true,
                                observable: true,
                                cover: CoverDegree::Total,
                            }]
                        } else {
                            vec![]
                        },
                        lights: vec![],
                    },
                    characters: vec![TableCharacterPlacement {
                        character_id: f.characters[0],
                        position: SpatialPoint {
                            x: 10,
                            y: 10,
                            z: if pc_platform { 10 } else { 0 },
                        },
                        height: 12,
                        allies: vec![],
                        enemies: std::iter::once(f.goblin)
                            .chain(f.opponent.filter(|_| pc_opportunity))
                            .collect(),
                    }],
                    creatures: std::iter::once(TableCreaturePlacement {
                        actor: f.goblin,
                        public_label: "Small armored figure".into(),
                        position: SpatialPoint {
                            x: if size == CreatureSize::Huge { 60 } else { 20 },
                            y: 10,
                            z: 0,
                        },
                        height: match size {
                            CreatureSize::Huge => 40,
                            CreatureSize::Large => 20,
                            _ => 8,
                        },
                        allies: vec![],
                        enemies: std::iter::once(f.actors[0])
                            .chain(f.opponent.filter(|_| !pc_opportunity))
                            .collect(),
                    })
                    .chain(f.opponent.map(|actor| TableCreaturePlacement {
                        actor,
                        public_label: "Other guard".into(),
                        position: SpatialPoint {
                            x: if pc_opportunity { 10 } else { 30 },
                            y: if pc_opportunity { 20 } else { 10 },
                            z: 0,
                        },
                        height: 8,
                        allies: vec![],
                        enemies: vec![f.goblin],
                    }))
                    .collect(),
                    geometry_ruling: Ruling {
                        basis: RulingBasis::GmAdjudication,
                        reason: "Host establishes the visible courtyard.".into(),
                    },
                    area_grid_policy: None,
                }),
            }),
        )
        .await;
        let shared_opponent = f.opponent.filter(|_| definition == "goblin-warrior");
        let groups = vec![
            InitiativeGroup {
                actors: vec![f.actors[0]],
                request_id: RollRequestId::new(),
            },
            InitiativeGroup {
                actors: std::iter::once(f.goblin).chain(shared_opponent).collect(),
                request_id: RollRequestId::new(),
            },
        ]
        .into_iter()
        .chain(
            f.opponent
                .filter(|_| shared_opponent.is_none())
                .map(|actor| InitiativeGroup {
                    actors: vec![actor],
                    request_id: RollRequestId::new(),
                }),
        )
        .collect::<Vec<_>>();
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
                            actor: f.goblin,
                            source: TacticalSource::Creature {
                                definition_id: definition.into(),
                            },
                            surprised: false,
                        },
                    ]
                    .into_iter()
                    .chain(f.opponent.map(|actor| TacticalCombatant {
                        actor,
                        source: TacticalSource::Creature {
                            definition_id: "goblin-warrior".into(),
                        },
                        surprised: false,
                    }))
                    .collect(),
                    groups: groups.clone(),
                },
            }),
        )
        .await;
        if let Some(opponent) = shared_opponent {
            let begun = f.state().await;
            let flow = begun.encounter.as_ref().unwrap().flow.as_ref().unwrap();
            assert_eq!(groups.len(), 2);
            assert_eq!(groups[0].actors, vec![f.actors[0]]);
            assert_eq!(groups[1].actors, vec![f.goblin, opponent]);
            assert_ne!(groups[0].request_id, groups[1].request_id);
            assert_eq!(flow.initiative_groups, groups);
            let rules = begun.rules.as_ref().unwrap();
            assert!(rules.rolls.is_empty());
            let pending = rules.pending.as_ref().unwrap();
            assert_eq!(pending.request.id, groups[0].request_id);
            assert_eq!(pending.request.roller, Some(f.actors[0]));
        }
        let pc_roll = Box::pin(f.roll(f.pc(0), 18)).await;
        if shared_opponent.is_some() {
            let after_pc = f.state().await;
            let rules = after_pc.rules.as_ref().unwrap();
            assert_eq!(rules.rolls.len(), 1);
            let pending = rules.pending.as_ref().unwrap();
            assert_eq!(pending.request.id, groups[1].request_id);
            assert_eq!(pending.request.roller, Some(f.goblin));
        }
        let source_roll = Box::pin(f.roll(TableTransportChannel::Host, 2)).await;
        if let Some(opponent) = shared_opponent {
            let tied = f.state().await;
            let rules = tied.rules.as_ref().unwrap();
            assert_eq!(rules.rolls.len(), 2);
            assert!(rules.pending.is_none());
            for (index, actor, accepted_by, face, total) in [
                (0, f.actors[0], pc_roll.command_id, 18, 20),
                (1, f.goblin, source_roll.command_id, 2, 4),
            ] {
                let roll = &rules.rolls[index];
                assert_eq!(roll.request.id, groups[index].request_id);
                assert_eq!(roll.request.roller, Some(actor));
                assert_eq!(roll.request.mode, RollMode::Normal);
                assert_eq!(
                    roll.request.dice,
                    vec![DieSpec {
                        count: 1,
                        sides: 20
                    }]
                );
                assert_eq!(roll.request.modifier, 2);
                assert_eq!(roll.result.source, RollSource::Physical);
                assert_eq!(
                    roll.result.dice,
                    vec![DieResult {
                        sides: 20,
                        value: face
                    }]
                );
                assert_eq!(roll.accepted_by.id, accepted_by);
                assert_eq!(roll.resolved.total, total);
            }
            let flow = tied.encounter.as_ref().unwrap().flow.as_ref().unwrap();
            assert_eq!(flow.initiative_groups, groups);
            assert_eq!(
                flow.phase,
                TacticalPhase::InitiativeTies {
                    ties: vec![InitiativeTie {
                        total: 4,
                        actors: vec![f.goblin, opponent],
                        proposed_order: None,
                        accepted_by: vec![],
                        host_decided: false,
                    }]
                }
            );
            Box::pin(f.host(TableAction::Tactical {
                action: TacticalAction::ProposeInitiativeTie {
                    order: vec![f.goblin, opponent],
                },
            }))
            .await;
            let ready = f.state().await;
            let final_rules = ready.rules.as_ref().unwrap();
            assert_eq!(final_rules.rolls, rules.rolls);
            let flow = ready.encounter.as_ref().unwrap().flow.as_ref().unwrap();
            assert_eq!(flow.phase, TacticalPhase::Active);
            assert_eq!(flow.initiative_groups, groups);
            assert_eq!(
                flow.initiative_decisions,
                vec![InitiativeTie {
                    total: 4,
                    actors: vec![f.goblin, opponent],
                    proposed_order: Some(vec![f.goblin, opponent]),
                    accepted_by: vec![],
                    host_decided: true,
                }]
            );
            let timing = final_rules.timing.as_ref().unwrap();
            assert_eq!(timing.index, 0);
            assert_eq!(
                timing
                    .order
                    .iter()
                    .map(|entry| (entry.actor, entry.total, entry.tie_break))
                    .collect::<Vec<_>>(),
                vec![(f.actors[0], 20, 0), (f.goblin, 4, 0), (opponent, 4, 1)]
            );
        } else if f.opponent.is_some() {
            Box::pin(f.roll(TableTransportChannel::Host, 1)).await;
        }
        f
    }
    fn pc(&self, index: usize) -> TableTransportChannel {
        TableTransportChannel::Player {
            player_id: self.players[index],
            character_id: self.characters[index],
        }
    }
    async fn state(&self) -> CampaignState {
        self.runtime
            .open_campaign(self.campaign)
            .await
            .unwrap()
            .state()
            .clone()
    }
    async fn view(&self, channel: TableTransportChannel) -> TablePresentedView {
        let viewer = match channel {
            TableTransportChannel::Host => TableViewer::Host,
            TableTransportChannel::Player { player_id, .. }
            | TableTransportChannel::SourceCreature { player_id, .. } => {
                TableViewer::Player(player_id)
            }
        };
        self.runtime
            .presented_table_view(self.campaign, viewer)
            .await
            .unwrap()
    }
    async fn request(
        &self,
        channel: TableTransportChannel,
        input: TableTransportInput,
    ) -> TableTransportRequest {
        let view = self.view(channel.clone()).await;
        let activation = matches!(&input,TableTransportInput::Action(action) if matches!(action.as_ref(),TableAction::EnableGrappleAccess));
        let source = matches!(&input,TableTransportInput::Action(action) if matches!(action.as_ref(),TableAction::EnableSourceActorAccess{..}));
        let session_id = match &input {
            TableTransportInput::Action(action)
                if matches!(action.as_ref(), TableAction::StartSession { .. }) =>
            {
                Some(self.session)
            }
            _ => view
                .active_session
                .as_ref()
                .map(|session| session.session_id),
        };
        TableTransportRequest {
            version: if view.grapple.is_some() || activation {
                3
            } else if view.source_control.is_some() || source {
                2
            } else {
                1
            },
            command_id: CommandId::new(),
            campaign_id: self.campaign,
            session_id,
            channel,
            revision: view.revision,
            input,
        }
    }
    async fn send(
        &self,
        channel: TableTransportChannel,
        input: TableTransportInput,
    ) -> TableTransportRequest {
        let request = self.request(channel, input).await;
        Box::pin(self.runtime.submit_presented_table(request.clone()))
            .await
            .unwrap();
        request
    }
    async fn host(&self, action: TableAction) -> TableTransportRequest {
        Box::pin(self.send(
            TableTransportChannel::Host,
            TableTransportInput::Action(Box::new(action)),
        ))
        .await
    }
    async fn roll(&self, channel: TableTransportChannel, value: u16) -> TableTransportRequest {
        let raw = self.view(channel.clone()).await.roll.unwrap();
        let dice = raw
            .dice
            .iter()
            .flat_map(|die| {
                std::iter::repeat_n(
                    DieResult {
                        sides: die.sides,
                        value,
                    },
                    if raw.mode == RollMode::Normal {
                        usize::from(die.count)
                    } else {
                        2
                    },
                )
            })
            .collect();
        Box::pin(self.send(
            channel,
            action(TacticalAction::SubmitRoll {
                result: RollResult {
                    request_id: raw.id,
                    source: RollSource::Physical,
                    dice,
                },
            }),
        ))
        .await
    }
    async fn choose(&self, channel: TableTransportChannel, label: &str) -> TableTransportRequest {
        let view = self.view(channel.clone()).await;
        let option = view
            .grapple
            .unwrap()
            .choices
            .into_iter()
            .find(|option| option.label == label)
            .unwrap_or_else(|| panic!("missing {label}"));
        self.request(
            channel,
            TableTransportInput::GrappleChoice { handle: option.key },
        )
        .await
    }
    async fn activate(&mut self) -> TableTransportRequest {
        let request = self
            .request(
                TableTransportChannel::Host,
                TableTransportInput::Action(Box::new(TableAction::EnableGrappleAccess)),
            )
            .await;
        Box::pin(self.cold(request.clone())).await;
        request
    }
    async fn cold(&mut self, request: TableTransportRequest) {
        let before = export_campaign(&self.pool, self.campaign).await.unwrap();
        let pool = open_sqlite("sqlite::memory:").await.unwrap();
        let mirror = runtime(pool.clone());
        Box::pin(mirror.restore_campaign(&before)).await.unwrap();
        self.pool.close().await;
        self.pool = open_sqlite_path(&self.path).await.unwrap();
        self.runtime = runtime(self.pool.clone());
        let accepted = Box::pin(self.runtime.submit_presented_table(request.clone()))
            .await
            .unwrap();
        let mirrored = Box::pin(mirror.submit_presented_table(request.clone()))
            .await
            .unwrap();
        // Independently accepted projections allocate fresh random audience
        // revisions/handles. Canonical state and public outcome must agree;
        // each database must retain its own exact accepted response on retry.
        let (TableTransportResult::Accepted(actual), TableTransportResult::Accepted(other)) =
            (&accepted, &mirrored)
        else {
            panic!("expected accepted commands")
        };
        assert_eq!(actual.command_id, other.command_id);
        assert_eq!(actual.outcome, other.outcome);
        assert_eq!(
            Box::pin(mirror.submit_presented_table(request.clone()))
                .await
                .unwrap(),
            mirrored
        );
        let expected = self.state().await;
        assert_eq!(
            mirror.open_campaign(self.campaign).await.unwrap().state(),
            &expected
        );
        pool.close().await;
        self.pool.close().await;
        self.pool = open_sqlite_path(&self.path).await.unwrap();
        self.runtime = runtime(self.pool.clone());
        let saved = export_campaign(&self.pool, self.campaign).await.unwrap();
        assert_eq!(
            Box::pin(self.runtime.submit_presented_table(request.clone()))
                .await
                .unwrap(),
            accepted
        );
        let mut changed = request.clone();
        changed.input = if request.input == action(TacticalAction::Dodge) {
            action(TacticalAction::Dash {
                speed: DashSpeed::Speed,
            })
        } else {
            action(TacticalAction::Dodge)
        };
        assert_ne!(changed.input, request.input);
        assert!(
            Box::pin(self.runtime.submit_presented_table(changed))
                .await
                .is_err()
        );
        let mut after = export_campaign(&self.pool, self.campaign).await.unwrap();
        after.exported_at_utc = saved.exported_at_utc.clone();
        assert_eq!(after, saved);
        assert_eq!(
            Box::pin(self.runtime.replay_rules(self.campaign))
                .await
                .unwrap(),
            expected
        );
        self.runtime
            .query_rules(
                self.campaign,
                CommandIssuer::Player(self.players[0]),
                RulesQuery::Character {
                    actor: self.actors[0],
                },
            )
            .await
            .unwrap();
    }
    async fn reject(&self, request: TableTransportRequest) {
        let before = export_campaign(&self.pool, self.campaign).await.unwrap();
        let rows = all_rows(&self.pool).await;
        assert!(
            Box::pin(self.runtime.submit_presented_table(request))
                .await
                .is_err()
        );
        let mut after = export_campaign(&self.pool, self.campaign).await.unwrap();
        after.exported_at_utc = before.exported_at_utc.clone();
        assert_eq!(after, before);
        assert_eq!(all_rows(&self.pool).await, rows);
    }
    async fn cold_action(
        &mut self,
        channel: TableTransportChannel,
        input: TacticalAction,
    ) -> TableTransportRequest {
        let request = self.request(channel, action(input)).await;
        Box::pin(self.cold(request.clone())).await;
        request
    }
    async fn cold_roll(
        &mut self,
        channel: TableTransportChannel,
        value: u16,
    ) -> TableTransportRequest {
        let raw = self.view(channel.clone()).await.roll.unwrap();
        let dice = raw
            .dice
            .iter()
            .flat_map(|die| {
                std::iter::repeat_n(
                    DieResult {
                        sides: die.sides,
                        value,
                    },
                    if raw.mode == RollMode::Normal {
                        usize::from(die.count)
                    } else {
                        2
                    },
                )
            })
            .collect();
        Box::pin(self.cold_action(
            channel,
            TacticalAction::SubmitRoll {
                result: RollResult {
                    request_id: raw.id,
                    source: RollSource::Physical,
                    dice,
                },
            },
        ))
        .await
    }
    async fn advance_choice(&mut self) {
        let view = self.view(TableTransportChannel::Host).await;
        if view.roll.is_none()
            && let Some(choice) = view
                .tactical
                .and_then(|t| t.continuation)
                .and_then(|c| c.choices.into_iter().next())
        {
            let request = self
                .request(
                    TableTransportChannel::Host,
                    TableTransportInput::SelectWork {
                        handle: choice.handle,
                    },
                )
                .await;
            Box::pin(self.cold(request)).await;
        }
    }
    async fn establish_pc_grip(&mut self) -> GrappleId {
        let request = self
            .choose(self.pc(0), "Grapple Small armored figure with left hand")
            .await;
        Box::pin(self.cold(request)).await;
        let request = self
            .choose(TableTransportChannel::Host, "Resist Grapple with Strength")
            .await;
        Box::pin(self.cold(request)).await;
        Box::pin(self.cold_roll(TableTransportChannel::Host, 1)).await;
        let id = self
            .state()
            .await
            .rules
            .unwrap()
            .tactical_grapples
            .unwrap()
            .active[0]
            .declaration
            .id;
        let request = self
            .choose(self.pc(0), "Finish without changing equipment")
            .await;
        Box::pin(self.cold(request)).await;
        id
    }
    async fn close(self) {
        self.pool.close().await;
        let directory = self.directory.clone();
        drop(self);
        cleanup::remove_closed_directory(&directory).await.unwrap();
    }
    async fn decline_hit(
        &mut self,
        order_channel: TableTransportChannel,
        target_channel: TableTransportChannel,
    ) {
        let order = self
            .view(order_channel.clone())
            .await
            .tactical
            .unwrap()
            .hit
            .unwrap()
            .order
            .unwrap();
        let request = self
            .request(
                order_channel,
                TableTransportInput::HitResponse {
                    handle: order.key,
                    decision: Box::new(TableHitInput::Order {
                        instruction: TacticalReactionOrdering {
                            ranked: vec![],
                            unlisted: ReactionUnlistedOrder::AfterForward,
                        },
                    }),
                },
            )
            .await;
        Box::pin(self.cold(request)).await;
        let response = self
            .view(target_channel.clone())
            .await
            .tactical
            .unwrap()
            .hit
            .unwrap()
            .response
            .unwrap();
        let request = self
            .request(
                target_channel,
                TableTransportInput::HitResponse {
                    handle: response.key,
                    decision: Box::new(TableHitInput::Respond { accept: false }),
                },
            )
            .await;
        Box::pin(self.cold(request)).await;
    }
}

#[tokio::test]
async fn activation_uses_original_history_and_refuses_legacy_raw_foreign_and_unsettled_input() {
    let mut f = Box::pin(Fixture::new()).await;
    let old = f.request(f.pc(0), action(TacticalAction::Dodge)).await;
    let old_receipt = Box::pin(f.runtime.submit_presented_table(old.clone()))
        .await
        .unwrap();
    let enable = f
        .request(
            TableTransportChannel::Host,
            TableTransportInput::Action(Box::new(TableAction::EnableGrappleAccess)),
        )
        .await;
    for version in [1, 2] {
        let mut bad = enable.clone();
        bad.version = version;
        bad.command_id = CommandId::new();
        Box::pin(f.reject(bad)).await;
    }
    let mut foreign = enable.clone();
    foreign.channel = f.pc(0);
    foreign.revision = f.view(f.pc(0)).await.revision;
    Box::pin(f.reject(foreign)).await;
    Box::pin(f.cold(enable)).await;
    assert_eq!(
        Box::pin(f.runtime.submit_presented_table(old))
            .await
            .unwrap(),
        old_receipt
    );
    let current = f.state().await;
    let access = current
        .table
        .as_ref()
        .unwrap()
        .grapple_access
        .as_ref()
        .unwrap();
    assert_eq!(access.version, TableGrappleAccessVersion::OrdinaryGrappleV1);
    let pack =
        dmd_rules::RulesPack::from_json(include_str!("../../../content/srd-5.2.1/kernel.json"))
            .unwrap();
    assert!(dmd_rules::validate_state(&current, &pack).is_err());
    let raw = f
        .request(
            f.pc(0),
            action(TacticalAction::Grapple {
                target: f.goblin,
                hand: Hand::Left,
                before_change: None,
            }),
        )
        .await;
    Box::pin(f.reject(raw)).await;
    assert!(f.view(f.pc(1)).await.grapple.unwrap().choices.is_empty());
    f.close().await;
}

#[tokio::test]
async fn real_grapple_save_release_and_after_equipment_cold_replay_keep_exact_raw_and_private_handles()
 {
    let mut f = Box::pin(Fixture::new()).await;
    Box::pin(f.activate()).await;
    let offered = f
        .choose(f.pc(0), "Grapple Small armored figure with left hand")
        .await;
    let mut stolen = offered.clone();
    stolen.command_id = CommandId::new();
    stolen.channel = f.pc(1);
    stolen.revision = f.view(f.pc(1)).await.revision;
    Box::pin(f.reject(stolen)).await;
    Box::pin(f.cold(offered)).await;
    let current = f.state().await;
    let attempt = current
        .encounter
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .resolution
        .as_ref()
        .unwrap()
        .grapple
        .as_ref()
        .unwrap()
        .activity
        .as_ref()
        .unwrap();
    let GrappleActivity::Attempt(attempt) = attempt else {
        panic!("expected real attempt")
    };
    let grip = attempt.declaration.id;
    for channel in [f.pc(0), f.pc(1)] {
        let view = f.view(channel).await;
        assert!(view.roll.is_none());
        let json = serde_json::to_string(&view).unwrap();
        assert!(!json.contains(&grip.0.to_string()));
        assert!(!json.contains("Private sentry"));
    }
    let save = f
        .choose(TableTransportChannel::Host, "Resist Grapple with Strength")
        .await;
    Box::pin(f.cold(save)).await;
    let raw = f.view(TableTransportChannel::Host).await.roll.unwrap();
    assert_eq!(raw.modifier, -1);
    let rolled = f
        .request(
            TableTransportChannel::Host,
            action(TacticalAction::SubmitRoll {
                result: RollResult {
                    request_id: raw.id,
                    source: RollSource::Physical,
                    dice: vec![DieResult {
                        sides: 20,
                        value: 1,
                    }],
                },
            }),
        )
        .await;
    Box::pin(f.cold(rolled)).await;
    let live = f.state().await;
    assert!(
        live.rules
            .as_ref()
            .unwrap()
            .tactical_grapples
            .as_ref()
            .unwrap()
            .grip(grip)
            .is_some()
    );
    let finish = f.choose(f.pc(0), "Finish without changing equipment").await;
    Box::pin(f.cold(finish)).await;
    let before = f.state().await;
    let rolls = before.rules.as_ref().unwrap().rolls.clone();
    let release = f
        .choose(f.pc(0), "Release Small armored figure from left hand")
        .await;
    Box::pin(f.cold(release)).await;
    let released = f.state().await;
    assert!(released.rules.as_ref().unwrap().tactical_grapples.is_none());
    assert_eq!(released.rules.as_ref().unwrap().rolls, rolls);
    assert!(
        f.view(f.pc(0))
            .await
            .tactical
            .unwrap()
            .attack_options
            .is_some()
    );
    f.close().await;
}

#[tokio::test]
async fn withdraw_cancels_only_its_pending_request_and_never_refunds_the_paid_attack() {
    let mut f = Box::pin(Fixture::new()).await;
    Box::pin(f.activate()).await;
    let start = f
        .choose(f.pc(0), "Grapple Small armored figure with right hand")
        .await;
    Box::pin(f.cold(start)).await;
    let save = f
        .choose(TableTransportChannel::Host, "Resist Grapple with Dexterity")
        .await;
    Box::pin(f.cold(save)).await;
    let before = f.state().await;
    let request = before
        .rules
        .as_ref()
        .unwrap()
        .pending
        .as_ref()
        .unwrap()
        .request
        .id;
    let withdraw = f.choose(f.pc(0), "Withdraw this Grapple attempt").await;
    Box::pin(f.cold(withdraw)).await;
    let after = f.state().await;
    let rules = after.rules.as_ref().unwrap();
    assert!(rules.pending.is_none());
    assert!(rules.cancelled_roll_ids.contains(&request));
    assert!(!rules.rolls.iter().any(|roll| roll.request.id == request));
    assert!(rules.timing.as_ref().unwrap().action_spent);
    assert!(
        f.view(f.pc(0))
            .await
            .grapple
            .unwrap()
            .choices
            .iter()
            .all(|choice| !choice.label.starts_with("Grapple "))
    );
    f.close().await;
}

#[tokio::test]
async fn forged_activation_snapshot_audit_and_original_anchor_leave_every_destination_row_unchanged()
 {
    let mut f = Box::pin(Fixture::new()).await;
    let activation = Box::pin(f.activate()).await;
    let original = export_campaign(&f.pool, f.campaign).await.unwrap();
    for mutation in [
        "missing-marker",
        "foreign-origin",
        "missing-event",
        "old-audit",
        "anchor-marker",
    ] {
        let mut forged = original.clone();
        match mutation {
            "missing-marker" => {
                let mut state = f.state().await;
                state.table.as_mut().unwrap().grapple_access = None;
                forged.current_state.state_json = state.encode_json().unwrap();
            }
            "foreign-origin" => {
                let mut state = f.state().await;
                state
                    .table
                    .as_mut()
                    .unwrap()
                    .grapple_access
                    .as_mut()
                    .unwrap()
                    .origin
                    .id = CommandId::new();
                forged.current_state.state_json = state.encode_json().unwrap();
            }
            "missing-event" => forged
                .event_journal
                .retain(|event| event.command_id != activation.command_id.0.to_string()),
            "old-audit" => {
                forged
                    .command_audit
                    .iter_mut()
                    .find(|audit| audit.id == activation.command_id.0.to_string())
                    .unwrap()
                    .command_schema_version = 1
            }
            "anchor-marker" => {
                let anchor = forged
                    .snapshots
                    .iter_mut()
                    .min_by_key(|snapshot| snapshot.event_sequence)
                    .unwrap();
                let mut state = CampaignState::decode_json(&anchor.state_json).unwrap();
                state.table.as_mut().unwrap().grapple_access =
                    f.state().await.table.unwrap().grapple_access;
                anchor.state_json = state.encode_json().unwrap();
            }
            _ => unreachable!(),
        }
        Box::pin(hostile_destination(&forged)).await;
    }
    f.close().await;
}

fn resolution(state: &CampaignState) -> &TacticalResolution {
    state
        .encounter
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .resolution
        .as_ref()
        .unwrap()
}

fn assert_empty_attack_reads(state: &CampaignState) -> GrappleCutKey {
    let resolution = resolution(state);
    let reads = resolution.grapple.as_ref().unwrap();
    assert!(reads.proofs.is_empty());
    assert!(reads.cuts.iter().all(|cut| cut.grips.is_empty()));
    let attack = resolution.attack.as_ref().unwrap();
    let admission = reads
        .cuts
        .iter()
        .rev()
        .find(|cut| {
            cut.key.reader
                == (GrappleReader::AttackAdmission {
                    attack: attack.origin.id,
                })
        })
        .unwrap();
    let pending = resolution.pending.as_ref().unwrap();
    let issue = reads
        .cuts
        .iter()
        .find(|cut| cut.key.reader == (GrappleReader::RequestIssue { roll: pending.key }))
        .unwrap();
    assert_eq!(issue.source_attack, Some(admission.key));
    assert_eq!(issue.key.work.occurrence, pending.work.occurrence);
    dmd_domain::validate_tactical_grapple_shapes(state).unwrap();
    assert!(
        reads.validate_shape(resolution, None).is_err(),
        "legacy local shape stays strict"
    );
    admission.key
}

async fn reject_state_image(f: &Fixture, state: CampaignState) {
    let mut forged = export_campaign(&f.pool, f.campaign).await.unwrap();
    let json = serde_json::to_string(&state).unwrap();
    forged.current_state.state_json = json.clone();
    if let Some(latest) = forged
        .snapshots
        .iter_mut()
        .max_by_key(|snapshot| snapshot.event_sequence)
        && u64::try_from(latest.event_sequence).ok() == Some(state.applied_event_sequence)
    {
        latest.state_json = json;
    }
    Box::pin(hostile_destination(&forged)).await;
}

#[tokio::test]
async fn activated_no_grip_attack_records_real_empty_reads_and_refuses_markerless_or_flight_substitution()
 {
    let mut f = Box::pin(Fixture::new()).await;
    Box::pin(f.activate()).await;
    let actor = f.pc(0);
    let target = f.goblin;
    Box::pin(f.cold_action(actor.clone(), TacticalAction::UnarmedStrike { target })).await;
    let issued = f.state().await;
    assert_empty_attack_reads(&issued);
    let mut missing_marker = issued.clone();
    missing_marker.table.as_mut().unwrap().grapple_access = None;
    assert!(dmd_domain::validate_tactical_grapple_shapes(&missing_marker).is_err());
    Box::pin(reject_state_image(&f, missing_marker)).await;
    let mut missing_cut = issued.clone();
    missing_cut
        .encounter
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .resolution
        .as_mut()
        .unwrap()
        .grapple
        .as_mut()
        .unwrap()
        .cuts
        .pop();
    Box::pin(reject_state_image(&f, missing_cut)).await;
    let mut flight = issued.clone();
    flight
        .encounter
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .resolution
        .as_mut()
        .unwrap()
        .grapple
        .as_mut()
        .unwrap()
        .cuts[0]
        .key
        .reader = GrappleReader::FlightLoss { actor: target };
    assert!(dmd_domain::validate_tactical_grapple_shapes(&flight).is_err());
    Box::pin(reject_state_image(&f, flight)).await;
    Box::pin(f.cold_roll(actor, 1)).await;
    assert!(
        f.state()
            .await
            .encounter
            .unwrap()
            .flow
            .unwrap()
            .resolution
            .is_none()
    );
    f.close().await;
}

#[tokio::test]
async fn activation_rejects_an_actual_pending_attack_then_accepts_the_settled_same_encounter() {
    let mut f = Box::pin(Fixture::new()).await;
    let actor = f.pc(0);
    let target = f.goblin;
    Box::pin(f.send(
        actor.clone(),
        action(TacticalAction::UnarmedStrike { target }),
    ))
    .await;
    let enable = f
        .request(
            TableTransportChannel::Host,
            TableTransportInput::Action(Box::new(TableAction::EnableGrappleAccess)),
        )
        .await;
    Box::pin(f.reject(enable)).await;
    Box::pin(f.roll(actor, 1)).await;
    Box::pin(f.activate()).await;
    assert!(f.state().await.table.unwrap().grapple_access.is_some());
    f.close().await;
}

#[tokio::test]
async fn activated_unarmed_opportunity_keeps_empty_occurrence_read_and_spends_one_reaction() {
    let mut f = Box::pin(Fixture::new()).await;
    Box::pin(f.activate()).await;
    let pc = f.pc(0);
    let actor = f.actors[0];
    let target = f.goblin;
    Box::pin(f.cold_action(pc.clone(), TacticalAction::EndTurn)).await;
    let movement = Box::pin(f.cold_action(
        TableTransportChannel::Host,
        TacticalAction::Move {
            path: vec![TacticalMoveStep {
                destination: SpatialPoint { x: 30, y: 10, z: 0 },
                mode: MovementMode::Walk,
            }],
        },
    ))
    .await;
    let offer = f
        .view(pc.clone())
        .await
        .tactical
        .unwrap()
        .opportunity
        .unwrap();
    assert!(offer.unarmed);
    assert_eq!(offer.actor, actor);
    assert_eq!(offer.target.actor, target);
    let opportunity = Box::pin(f.cold_action(
        pc.clone(),
        TacticalAction::OpportunityAttack {
            choice: TacticalMeleeChoice::UnarmedDamage {
                ability: Ability::Strength,
            },
        },
    ))
    .await;
    let issued = f.state().await;
    let cut = assert_empty_attack_reads(&issued);
    assert_eq!(cut.work.resolution, movement.command_id);
    assert_eq!(
        cut.reader,
        GrappleReader::AttackAdmission {
            attack: opportunity.command_id
        }
    );
    assert!(
        issued
            .rules
            .as_ref()
            .unwrap()
            .timing
            .as_ref()
            .unwrap()
            .reactions_spent
            .contains(&actor)
    );
    Box::pin(f.cold_roll(pc, 1)).await;
    let final_state = f.state().await;
    assert_eq!(
        final_state
            .encounter
            .as_ref()
            .unwrap()
            .participant(target)
            .unwrap()
            .position
            .x,
        30
    );
    assert_eq!(
        final_state
            .rules
            .as_ref()
            .unwrap()
            .timing
            .as_ref()
            .unwrap()
            .reactions_spent
            .iter()
            .filter(|id| **id == actor)
            .count(),
        1
    );
    f.close().await;
}

#[tokio::test]
async fn activated_source_three_rays_keep_distinct_admission_ancestry_and_raw_ids_across_cold_replay()
 {
    let mut f = Box::pin(Fixture::with_source("adult-red-dragon", CreatureSize::Huge)).await;
    let activation = Box::pin(f.activate()).await;
    let pc = f.pc(0);
    let target = f.actors[0];
    Box::pin(f.cold_action(pc, TacticalAction::EndTurn)).await;
    let legendary = f
        .view(TableTransportChannel::Host)
        .await
        .tactical
        .unwrap()
        .legendary_action
        .unwrap();
    assert_eq!(legendary, f.goblin);
    Box::pin(f.cold_action(
        TableTransportChannel::Host,
        TacticalAction::DeclineLegendaryAction,
    ))
    .await;
    let options = f
        .view(TableTransportChannel::Host)
        .await
        .tactical
        .unwrap()
        .casting_options
        .unwrap();
    let choice = options
        .variants
        .into_iter()
        .find(|v| v.choice.spell_id == "scorching-ray")
        .unwrap()
        .choice;
    let casting = Box::pin(f.cold_action(
        TableTransportChannel::Host,
        TacticalAction::CastSpell {
            choice,
            targets: SpellTargetChoice::Entities(vec![target; 3]),
        },
    ))
    .await;
    let mut admissions = Vec::new();
    let mut requests = std::collections::HashSet::new();
    let mut advancing = casting.clone();
    for index in 0..3 {
        Box::pin(f.advance_choice()).await;
        let issued = f.state().await;
        let cut = assert_empty_attack_reads(&issued);
        let attack = resolution(&issued).attack.as_ref().unwrap();
        let pending = resolution(&issued).pending.as_ref().unwrap();
        assert_eq!(attack.origin.id, advancing.command_id);
        if index > 0 {
            assert_ne!(attack.origin.id, casting.command_id);
        }
        assert_eq!(
            cut.reader,
            GrappleReader::AttackAdmission {
                attack: attack.origin.id
            }
        );
        assert_eq!(pending.key.origin, casting.command_id);
        assert_eq!(pending.key.subject, target);
        assert_eq!(
            pending.key.request_id(),
            issued
                .rules
                .as_ref()
                .unwrap()
                .pending
                .as_ref()
                .unwrap()
                .request
                .id
        );
        let export = export_campaign(&f.pool, f.campaign).await.unwrap();
        let selected = export
            .table_transport_bindings
            .iter()
            .find(|binding| binding.meta.id == attack.origin.id)
            .unwrap();
        let selected: TableTransportRequest = serde_json::from_str(&selected.request_json).unwrap();
        // Singleton ray work advances during casting, then the preceding miss.
        // The actual accepted request owns admission; raw dice retain the cast.
        assert_eq!(selected, advancing);
        assert!(!admissions.contains(&cut));
        admissions.push(cut);
        assert!(
            requests.insert(
                issued
                    .rules
                    .as_ref()
                    .unwrap()
                    .pending
                    .as_ref()
                    .unwrap()
                    .request
                    .id
            )
        );
        assert_eq!(
            resolution(&issued)
                .grapple
                .as_ref()
                .unwrap()
                .cuts
                .iter()
                .filter(|c| matches!(c.key.reader, GrappleReader::AttackAdmission { .. }))
                .count(),
            index + 1
        );
        if index > 0 {
            let mut forged = issued.clone();
            let reads = forged
                .encounter
                .as_mut()
                .unwrap()
                .flow
                .as_mut()
                .unwrap()
                .resolution
                .as_mut()
                .unwrap()
                .grapple
                .as_mut()
                .unwrap();
            let first = reads
                .cuts
                .iter()
                .find(|c| matches!(c.key.reader, GrappleReader::AttackAdmission { .. }))
                .unwrap()
                .key;
            reads.cuts.last_mut().unwrap().source_attack = Some(first);
            Box::pin(reject_state_image(&f, forged)).await;
        }
        for mutation in [
            "advancing-origin",
            "wrong-cast",
            "wrong-target",
            "missing-admission",
        ] {
            let mut forged = issued.clone();
            let r = corrections::resolution_mut(&mut forged);
            match mutation {
                "advancing-origin" => {
                    let issue = r
                        .grapple
                        .as_mut()
                        .unwrap()
                        .cuts
                        .iter_mut()
                        .find(|read| {
                            read.key.reader == (GrappleReader::RequestIssue { roll: pending.key })
                        })
                        .unwrap();
                    let GrappleReader::RequestIssue { roll } = &mut issue.key.reader else {
                        unreachable!()
                    };
                    let substituted = if index == 0 {
                        activation.command_id
                    } else {
                        attack.origin.id
                    };
                    assert_ne!(substituted, roll.origin);
                    roll.origin = substituted;
                }
                "wrong-cast" => r.casts[0].cast.plan.origin.id = CommandId::new(),
                "wrong-target" => r.casts[0].targets[0].actor = f.goblin,
                "missing-admission" => r
                    .grapple
                    .as_mut()
                    .unwrap()
                    .cuts
                    .retain(|read| read.key != cut),
                _ => unreachable!(),
            }
            Box::pin(reject_state_image(&f, forged)).await;
        }
        advancing = Box::pin(f.cold_roll(TableTransportChannel::Host, 1)).await;
    }
    let after = f.state().await;
    assert!(after.encounter.unwrap().flow.unwrap().resolution.is_none());
    let recorded = after.rules.unwrap().rolls;
    assert!(
        requests
            .iter()
            .all(|id| recorded.iter().filter(|raw| raw.request.id == *id).count() == 1)
    );
    f.close().await;
}

#[tokio::test]
async fn real_goblin_escape_checks_use_current_source_skills_and_off_turn_release_cancels_only_escape()
 {
    for (choice, modifier) in [("Athletics", -1), ("Acrobatics", 2)] {
        let mut f = Box::pin(Fixture::new()).await;
        Box::pin(f.activate()).await;
        let grip = Box::pin(f.establish_pc_grip()).await;
        let pc = f.pc(0);
        Box::pin(f.cold_action(pc.clone(), TacticalAction::EndTurn)).await;
        let view = f.view(TableTransportChannel::Host).await;
        let offer = view
            .grapple
            .unwrap()
            .choices
            .into_iter()
            .find(|o| o.actor == f.goblin && o.label.ends_with(&format!("using {choice}")))
            .unwrap();
        let escape = f
            .request(
                TableTransportChannel::Host,
                TableTransportInput::GrappleChoice { handle: offer.key },
            )
            .await;
        Box::pin(f.cold(escape)).await;
        let before = f.state().await;
        let pending = before.rules.as_ref().unwrap().pending.as_ref().unwrap();
        assert_eq!(pending.request.modifier, modifier);
        assert!(
            before
                .rules
                .as_ref()
                .unwrap()
                .timing
                .as_ref()
                .unwrap()
                .action_spent
        );
        let request = pending.request.id;
        let accepted = before.rules.as_ref().unwrap().rolls.clone();
        let bad = f
            .request(
                TableTransportChannel::Host,
                action(TacticalAction::VoluntarilyFailSave),
            )
            .await;
        Box::pin(f.reject(bad)).await;
        let release = f
            .choose(pc, "Release Small armored figure from left hand")
            .await;
        Box::pin(f.cold(release)).await;
        let after = f.state().await;
        let rules = after.rules.as_ref().unwrap();
        assert!(rules.tactical_grapples.is_none());
        assert!(rules.cancelled_roll_ids.contains(&request));
        assert_eq!(rules.rolls, accepted);
        assert!(rules.timing.as_ref().unwrap().action_spent);
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
        assert!(
            !f.view(TableTransportChannel::Host)
                .await
                .grapple
                .unwrap()
                .choices
                .iter()
                .any(|o| o.label.contains(&grip.0.to_string()))
        );
        f.close().await;
    }
}

#[tokio::test]
async fn player_owned_goblin_genuine_save_and_after_equipment_retry_survive_controller_transfer() {
    let mut f = Box::pin(Fixture::with_opponent(
        "goblin-warrior",
        CreatureSize::Small,
        true,
    ))
    .await;
    Box::pin(f.activate()).await;
    let source = TableTransportChannel::SourceCreature {
        player_id: f.players[1],
        actor: f.goblin,
    };
    let control = f
        .request(
            TableTransportChannel::Host,
            TableTransportInput::Action(Box::new(TableAction::SetSourceCreatureController {
                actor: f.goblin,
                controller: CreatureController::Player(f.players[1]),
            })),
        )
        .await;
    Box::pin(f.cold(control)).await;
    let pc = f.pc(0);
    Box::pin(f.cold_action(pc.clone(), TacticalAction::EndTurn)).await;
    assert!(
        f.view(source.clone())
            .await
            .grapple
            .unwrap()
            .choices
            .iter()
            .all(|o| !o.label.starts_with("Grapple Character 0")),
        "player versus player body control stays closed before declaration"
    );
    let option = f
        .view(source.clone())
        .await
        .grapple
        .unwrap()
        .choices
        .into_iter()
        .find(|o| {
            o.actor == f.goblin
                && o.label.starts_with("Grapple Other guard ")
                && o.label.ends_with("with right hand")
        })
        .unwrap();
    let start = f
        .request(
            source.clone(),
            TableTransportInput::GrappleChoice { handle: option.key },
        )
        .await;
    Box::pin(f.cold(start)).await;
    let save = f
        .choose(TableTransportChannel::Host, "Resist Grapple with Strength")
        .await;
    Box::pin(f.cold(save)).await;
    Box::pin(f.cold_action(
        TableTransportChannel::Host,
        TacticalAction::VoluntarilyFailSave,
    ))
    .await;
    let finish = f
        .choose(source.clone(), "Finish without changing equipment")
        .await;
    Box::pin(f.cold(finish.clone())).await;
    let accepted = Box::pin(f.runtime.submit_presented_table(finish.clone()))
        .await
        .unwrap();
    let formerly_offered = f
        .view(source.clone())
        .await
        .grapple
        .unwrap()
        .choices
        .into_iter()
        .find(|o| o.actor == f.goblin && o.label.starts_with("Release "))
        .unwrap()
        .key;
    let control = f
        .request(
            TableTransportChannel::Host,
            TableTransportInput::Action(Box::new(TableAction::SetSourceCreatureController {
                actor: f.goblin,
                controller: CreatureController::Host,
            })),
        )
        .await;
    Box::pin(f.cold(control)).await;
    assert_eq!(
        Box::pin(f.runtime.submit_presented_table(finish))
            .await
            .unwrap(),
        accepted
    );
    let raw = f
        .request(
            source,
            TableTransportInput::GrappleChoice {
                handle: formerly_offered,
            },
        )
        .await;
    Box::pin(f.reject(raw)).await;
    assert!(f.view(f.pc(1)).await.grapple.unwrap().choices.is_empty());
    let option = f
        .view(TableTransportChannel::Host)
        .await
        .grapple
        .unwrap()
        .choices
        .into_iter()
        .find(|o| o.actor == f.goblin && o.label.starts_with("Release "))
        .unwrap();
    let release = f
        .request(
            TableTransportChannel::Host,
            TableTransportInput::GrappleChoice { handle: option.key },
        )
        .await;
    Box::pin(f.cold(release)).await;
    assert!(f.state().await.rules.unwrap().tactical_grapples.is_none());
    f.close().await;
}

#[tokio::test]
async fn current_old_mage_accepts_incoming_pc_grip_but_never_gains_a_grappling_anatomy_grant() {
    let mut f = Box::pin(Fixture::with_source("mage", CreatureSize::Medium)).await;
    Box::pin(f.activate()).await;
    Box::pin(f.establish_pc_grip()).await;
    let pc = f.pc(0);
    Box::pin(f.cold_action(pc, TacticalAction::EndTurn)).await;
    let options = f.view(TableTransportChannel::Host).await.grapple.unwrap();
    assert!(
        options
            .choices
            .iter()
            .any(|o| o.actor == f.goblin && o.label.starts_with("Escape "))
    );
    assert!(
        options
            .choices
            .iter()
            .all(|o| o.actor != f.goblin || !o.label.starts_with("Grapple "))
    );
    f.close().await;
}

#[tokio::test]
async fn actual_escape_success_and_failure_keep_paid_action_and_physical_evidence() {
    for (face, succeeded) in [(1, false), (20, true)] {
        let mut f = Box::pin(Fixture::new()).await;
        Box::pin(f.activate()).await;
        let grip = Box::pin(f.establish_pc_grip()).await;
        let pc = f.pc(0);
        Box::pin(f.cold_action(pc, TacticalAction::EndTurn)).await;
        let option = f
            .view(TableTransportChannel::Host)
            .await
            .grapple
            .unwrap()
            .choices
            .into_iter()
            .find(|o| o.actor == f.goblin && o.label.ends_with("using Acrobatics"))
            .unwrap();
        let escape = f
            .request(
                TableTransportChannel::Host,
                TableTransportInput::GrappleChoice { handle: option.key },
            )
            .await;
        Box::pin(f.cold(escape)).await;
        let before = f.state().await;
        let request = before
            .rules
            .as_ref()
            .unwrap()
            .pending
            .as_ref()
            .unwrap()
            .request
            .id;
        assert!(
            matches!(before.rules.as_ref().unwrap().pending.as_ref().unwrap().purpose,PendingPurpose::TacticalResolution{key,..} if key.role==TacticalRollRole::GrappleEscape)
        );
        Box::pin(f.cold_roll(TableTransportChannel::Host, face)).await;
        let state = f.state().await;
        let rules = state.rules.as_ref().unwrap();
        assert_eq!(
            rules
                .tactical_grapples
                .as_ref()
                .and_then(|g| g.grip(grip))
                .is_none(),
            succeeded
        );
        assert!(rules.timing.as_ref().unwrap().action_spent);
        assert!(!rules.cancelled_roll_ids.contains(&request));
        assert_eq!(
            rules
                .rolls
                .iter()
                .find(|r| r.request.id == request)
                .unwrap()
                .result
                .dice,
            vec![DieResult {
                sides: 20,
                value: face
            }]
        );
        f.close().await;
    }
}

#[tokio::test]
async fn self_only_move_retains_grip_until_actual_range_crossing_and_never_moves_the_target() {
    let mut f = Box::pin(Fixture::new()).await;
    Box::pin(f.activate()).await;
    let grip = Box::pin(f.establish_pc_grip()).await;
    let pc = f.pc(0);
    let target = f.goblin;
    let before = f.state().await;
    let target_position = before
        .encounter
        .as_ref()
        .unwrap()
        .participant(target)
        .unwrap()
        .position;
    let ordinary = f
        .request(
            pc.clone(),
            action(TacticalAction::Move {
                path: vec![TacticalMoveStep {
                    destination: SpatialPoint { x: 0, y: 10, z: 0 },
                    mode: MovementMode::Walk,
                }],
            }),
        )
        .await;
    Box::pin(f.reject(ordinary)).await;
    let movement = Box::pin(f.cold_action(
        pc,
        TacticalAction::MoveSelfOnly {
            path: vec![TacticalMoveStep {
                destination: SpatialPoint { x: 0, y: 10, z: 0 },
                mode: MovementMode::Walk,
            }],
        },
    ))
    .await;
    let suspended = f.state().await;
    assert!(
        suspended
            .rules
            .as_ref()
            .unwrap()
            .tactical_grapples
            .as_ref()
            .unwrap()
            .grip(grip)
            .is_some()
    );
    let admission = resolution(&suspended)
        .movement
        .as_ref()
        .unwrap()
        .grapple_self_only
        .as_ref()
        .unwrap();
    assert_eq!(admission.origin.id, movement.command_id);
    assert_eq!(admission.grips, vec![grip]);
    // Being held prevents travel but does not remove the target's genuine OA.
    assert_eq!(
        f.view(TableTransportChannel::Host)
            .await
            .tactical
            .unwrap()
            .opportunity
            .unwrap()
            .actor,
        target
    );
    Box::pin(f.cold_action(
        TableTransportChannel::Host,
        TacticalAction::DeclineOpportunity,
    ))
    .await;
    let after = f.state().await;
    assert!(after.rules.as_ref().unwrap().tactical_grapples.is_none());
    assert_eq!(
        after
            .encounter
            .as_ref()
            .unwrap()
            .participant(target)
            .unwrap()
            .position,
        target_position
    );
    assert_eq!(
        after
            .encounter
            .as_ref()
            .unwrap()
            .participant(f.actors[0])
            .unwrap()
            .position
            .x,
        0
    );
    assert_eq!(
        after
            .encounter
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .budget
            .movement_spent,
        10
    );
    f.close().await;
}

#[tokio::test]
async fn actual_source_critical_knockout_ends_the_incapacitated_holders_grip_after_damage_choice() {
    let mut f = Box::pin(Fixture::new()).await;
    Box::pin(f.activate()).await;
    let grip = Box::pin(f.establish_pc_grip()).await;
    let pc = f.pc(0);
    let holder = f.actors[0];
    let source = f.goblin;
    Box::pin(f.cold_action(pc.clone(), TacticalAction::EndTurn)).await;
    let weapon = f
        .state()
        .await
        .items
        .values()
        .find(|i| i.custody == Custody::Entity(source) && i.definition_id == "scimitar")
        .unwrap()
        .id;
    Box::pin(f.cold_action(
        TableTransportChannel::Host,
        TacticalAction::CreatureWeaponAttack {
            feature_id: "scimitar".into(),
            choice: CreatureWeaponUseChoice {
                weapon,
                target: holder,
                grip: WeaponGrip::OneHand(Hand::Right),
                ammunition: None,
                equipment_change: Some(AttackEquipmentChange {
                    timing: EquipmentChangeTiming::BeforeAttack,
                    operation: AttackEquipmentOperation::Equip {
                        item: weapon,
                        hand: Hand::Right,
                    },
                }),
            },
        },
    ))
    .await;
    let issued = f.state().await;
    assert_eq!(
        issued
            .rules
            .as_ref()
            .unwrap()
            .pending
            .as_ref()
            .unwrap()
            .request
            .mode,
        RollMode::Normal
    );
    assert_eq!(
        resolution(&issued).grapple.as_ref().unwrap().proofs[0]
            .declaration
            .id,
        grip
    );
    Box::pin(f.cold_roll(TableTransportChannel::Host, 20)).await;
    Box::pin(f.decline_hit(TableTransportChannel::Host, pc)).await;
    Box::pin(f.cold_roll(TableTransportChannel::Host, 6)).await;
    let paused = f.state().await;
    assert!(paused.rules.as_ref().unwrap().tactical_grapples.is_some());
    assert_eq!(
        resolution(&paused).attack.as_ref().unwrap().stage,
        TacticalAttackStage::KnockoutChoice
    );
    Box::pin(f.cold_action(
        TableTransportChannel::Host,
        TacticalAction::ChooseAttackKnockout {
            choice: KnockoutChoice::NormalDamage,
        },
    ))
    .await;
    let after = f.state().await;
    assert_eq!(after.rules.as_ref().unwrap().entities[&holder].hp, 0);
    assert!(
        dmd_rules::active_conditions(after.rules.as_ref().unwrap(), holder)
            .contains(&Condition::Unconscious)
    );
    assert!(after.rules.as_ref().unwrap().tactical_grapples.is_none());
    assert!(
        after
            .rules
            .as_ref()
            .unwrap()
            .rolls
            .starts_with(&paused.rules.as_ref().unwrap().rolls)
    );
    f.close().await;
}

#[tokio::test]
async fn actual_chimera_flight_loss_keeps_its_fall_and_issued_dice_after_owner_release() {
    let mut f = Box::pin(Fixture::with_geometry(
        "chimera",
        CreatureSize::Large,
        false,
        false,
        true,
    ))
    .await;
    Box::pin(f.activate()).await;
    let pc = f.pc(0);
    let target = f.goblin;
    Box::pin(f.cold_action(pc.clone(), TacticalAction::EndTurn)).await;
    let initial = f.state().await;
    let encounter = initial.encounter.as_ref().unwrap();
    let grappler = encounter.participant(f.actors[0]).unwrap();
    assert_eq!(
        grappler.position,
        SpatialPoint {
            x: 10,
            y: 10,
            z: 10
        }
    );
    assert_eq!(
        dmd_rules::spatial::fall_destination(encounter, grappler.entity_id).unwrap(),
        None
    );
    assert_eq!(encounter.battlefield.obstacles[0].id, "grappler-platform");
    Box::pin(f.cold_action(
        TableTransportChannel::Host,
        TacticalAction::Move {
            path: vec![
                TacticalMoveStep {
                    destination: SpatialPoint {
                        x: 20,
                        y: 10,
                        z: 10,
                    },
                    mode: MovementMode::Fly,
                },
                TacticalMoveStep {
                    destination: SpatialPoint {
                        x: 20,
                        y: 10,
                        z: 20,
                    },
                    mode: MovementMode::Fly,
                },
                TacticalMoveStep {
                    destination: SpatialPoint {
                        x: 20,
                        y: 10,
                        z: 30,
                    },
                    mode: MovementMode::Fly,
                },
                TacticalMoveStep {
                    destination: SpatialPoint {
                        x: 20,
                        y: 10,
                        z: 20,
                    },
                    mode: MovementMode::Fly,
                },
            ],
        },
    ))
    .await;
    let crossing = f
        .view(pc.clone())
        .await
        .tactical
        .unwrap()
        .opportunity
        .unwrap();
    assert_eq!(crossing.actor, f.actors[0]);
    assert_eq!(crossing.target.actor, target);
    Box::pin(f.cold_action(pc.clone(), TacticalAction::DeclineOpportunity)).await;
    assert_eq!(
        f.state()
            .await
            .encounter
            .unwrap()
            .participant(target)
            .unwrap()
            .position,
        SpatialPoint {
            x: 20,
            y: 10,
            z: 20
        }
    );
    Box::pin(f.cold_action(TableTransportChannel::Host, TacticalAction::EndTurn)).await;
    let ready = f.state().await;
    let encounter = ready.encounter.as_ref().unwrap();
    assert_eq!(
        dmd_rules::spatial::participant_distance(
            encounter.participant(f.actors[0]).unwrap(),
            encounter.participant(target).unwrap(),
        )
        .unwrap(),
        10
    );
    let attempt = f
        .choose(pc.clone(), "Grapple Small armored figure with left hand")
        .await;
    Box::pin(f.cold(attempt)).await;
    let save = f
        .choose(TableTransportChannel::Host, "Resist Grapple with Strength")
        .await;
    Box::pin(f.cold(save)).await;
    Box::pin(f.cold_roll(TableTransportChannel::Host, 1)).await;
    let airborne = f.state().await;
    let fall = resolution(&airborne).falls[0].clone();
    assert!(matches!(
        fall.cause,
        TacticalFallCause::GrappleFlightLost { .. }
    ));
    let pending = airborne
        .rules
        .as_ref()
        .unwrap()
        .pending
        .as_ref()
        .unwrap()
        .clone();
    assert!(
        matches!(pending.purpose,PendingPurpose::TacticalResolution{key,..} if key.role==TacticalRollRole::FallDamage)
    );
    assert_eq!(fall.path.from.z, 20);
    assert_eq!(fall.path.to.z, 0);
    let release = f
        .choose(pc.clone(), "Release Small armored figure from left hand")
        .await;
    Box::pin(f.cold(release)).await;
    let released = f.state().await;
    assert_eq!(resolution(&released).falls, vec![fall.clone()]);
    assert_eq!(
        released.rules.as_ref().unwrap().pending.as_ref(),
        Some(&pending)
    );
    assert!(released.rules.as_ref().unwrap().tactical_grapples.is_none());
    Box::pin(f.cold_roll(TableTransportChannel::Host, 1)).await;
    let landed = f.state().await;
    assert_eq!(
        landed
            .encounter
            .as_ref()
            .unwrap()
            .participant(target)
            .unwrap()
            .position
            .z,
        0
    );
    assert_eq!(resolution(&landed).falls.len(), 1);
    assert!(matches!(
        resolution(&landed).falls[0].stage,
        TacticalFallStage::Complete { .. }
    ));
    assert_eq!(
        landed
            .rules
            .as_ref()
            .unwrap()
            .rolls
            .iter()
            .filter(|r| r.request.id == pending.request.id)
            .count(),
        1
    );
    let finish = f.choose(pc, "Finish without changing equipment").await;
    Box::pin(f.cold(finish)).await;
    f.close().await;
}

#[tokio::test]
async fn unrelated_audience_whole_projection_stays_exact_through_private_attempt_save_and_release()
{
    let mut f = Box::pin(Fixture::new()).await;
    Box::pin(f.activate()).await;
    let outsider = f.view(f.pc(1)).await;
    let retained = f
        .runtime
        .table_view(f.campaign, TableViewer::Player(f.players[1]))
        .await
        .unwrap();
    let host_before = f
        .runtime
        .table_view(f.campaign, TableViewer::Host)
        .await
        .unwrap();
    let start = f
        .choose(f.pc(0), "Grapple Small armored figure with left hand")
        .await;
    Box::pin(f.cold(start)).await;
    Box::pin(assert_private_history(&f, &retained, &outsider)).await;
    let save = f
        .choose(TableTransportChannel::Host, "Resist Grapple with Dexterity")
        .await;
    Box::pin(f.cold(save)).await;
    Box::pin(assert_private_history(&f, &retained, &outsider)).await;
    Box::pin(f.cold_roll(TableTransportChannel::Host, 1)).await;
    Box::pin(assert_private_history(&f, &retained, &outsider)).await;
    let finish = f.choose(f.pc(0), "Finish without changing equipment").await;
    Box::pin(f.cold(finish)).await;
    Box::pin(assert_private_history(&f, &retained, &outsider)).await;
    let release = f
        .choose(f.pc(0), "Release Small armored figure from left hand")
        .await;
    Box::pin(f.cold(release)).await;
    Box::pin(assert_private_history(&f, &retained, &outsider)).await;
    let host_after = f
        .runtime
        .table_view(f.campaign, TableViewer::Host)
        .await
        .unwrap();
    assert_eq!(
        host_after.transcript.len(),
        host_before.transcript.len() + 5
    );
    f.close().await;
}

async fn assert_private_history(f: &Fixture, retained: &TableView, presented: &TablePresentedView) {
    let current = f
        .runtime
        .table_view(f.campaign, TableViewer::Player(f.players[1]))
        .await
        .unwrap();
    assert_eq!(current.transcript, retained.transcript);
    assert_eq!(current.recap, retained.recap);
    let opaque = f.view(f.pc(1)).await;
    assert_eq!(&opaque, presented);
    assert_eq!(
        current
            .transcript
            .iter()
            .map(|entry| (&entry.kind, &entry.speaker, &entry.text))
            .collect::<Vec<_>>(),
        opaque
            .transcript
            .iter()
            .map(|entry| (&entry.kind, &entry.speaker, &entry.text))
            .collect::<Vec<_>>()
    );
    assert_eq!(current.recap, opaque.recap);
}

#[tokio::test]
async fn pc_physical_attack_keeps_admitted_hand_read_after_release_before_the_reported_attack() {
    let mut f = Box::pin(Fixture::new()).await;
    Box::pin(f.activate()).await;
    let grip = Box::pin(f.establish_pc_grip()).await;
    let pc = f.pc(0);
    let actor = f.actors[0];
    let target = f.goblin;
    Box::pin(f.cold_action(pc.clone(), TacticalAction::EndTurn)).await;
    Box::pin(f.cold_action(TableTransportChannel::Host, TacticalAction::EndTurn)).await;
    let weapon = f
        .state()
        .await
        .items
        .values()
        .find(|i| i.custody == Custody::Entity(actor) && i.definition_id == "dagger")
        .unwrap()
        .id;
    Box::pin(f.cold_action(
        pc.clone(),
        TacticalAction::Attack {
            choice: WeaponUseChoice {
                weapon,
                target,
                delivery: WeaponDelivery::Melee,
                ability: Ability::Strength,
                grip: WeaponGrip::OneHand(Hand::Right),
                purpose: WeaponAttackPurpose::Normal,
                ammunition: None,
                equipment_change: Some(AttackEquipmentChange {
                    timing: EquipmentChangeTiming::BeforeAttack,
                    operation: AttackEquipmentOperation::Equip {
                        item: weapon,
                        hand: Hand::Right,
                    },
                }),
            },
        },
    ))
    .await;
    let issued = f.state().await;
    let raw = issued
        .rules
        .as_ref()
        .unwrap()
        .pending
        .as_ref()
        .unwrap()
        .clone();
    let cuts = resolution(&issued).grapple.as_ref().unwrap().cuts.clone();
    assert!(cuts.iter().all(|cut| cut.grips == vec![grip]));
    let release = f
        .choose(pc.clone(), "Release Small armored figure from left hand")
        .await;
    Box::pin(f.cold(release)).await;
    let released = f.state().await;
    assert!(released.rules.as_ref().unwrap().tactical_grapples.is_none());
    assert_eq!(
        released.rules.as_ref().unwrap().pending.as_ref(),
        Some(&raw)
    );
    assert_eq!(resolution(&released).grapple.as_ref().unwrap().cuts, cuts);
    Box::pin(f.cold_roll(pc, 1)).await;
    let after = f.state().await;
    assert_eq!(
        after
            .rules
            .as_ref()
            .unwrap()
            .tactical_inventory
            .as_ref()
            .unwrap()
            .loadout(actor)
            .unwrap()
            .hands
            .hands[1],
        HandAssignment::Item(weapon)
    );
    assert_eq!(
        after
            .rules
            .as_ref()
            .unwrap()
            .rolls
            .iter()
            .find(|r| r.request.id == raw.request.id)
            .unwrap()
            .request,
        raw.request
    );
    f.close().await;
}

#[tokio::test]
async fn genuine_intrinsic_attack_retains_the_target_relation_after_off_turn_owner_release() {
    let mut f = Box::pin(Fixture::with_source("chimera", CreatureSize::Large)).await;
    Box::pin(f.activate()).await;
    let grip = Box::pin(f.establish_pc_grip()).await;
    let pc = f.pc(0);
    let target = f.actors[0];
    Box::pin(f.cold_action(pc.clone(), TacticalAction::EndTurn)).await;
    Box::pin(f.cold_action(
        TableTransportChannel::Host,
        TacticalAction::CreatureAttack {
            target,
            feature_id: "bite".into(),
            weapon: None,
        },
    ))
    .await;
    let before = f.state().await;
    let raw = before
        .rules
        .as_ref()
        .unwrap()
        .pending
        .as_ref()
        .unwrap()
        .clone();
    assert_eq!(raw.request.mode, RollMode::Normal);
    assert!(
        resolution(&before)
            .grapple
            .as_ref()
            .unwrap()
            .cuts
            .iter()
            .all(|cut| cut.grips == vec![grip])
    );
    let release = f
        .choose(pc, "Release Small armored figure from left hand")
        .await;
    Box::pin(f.cold(release)).await;
    assert_eq!(
        f.state().await.rules.as_ref().unwrap().pending.as_ref(),
        Some(&raw)
    );
    Box::pin(f.cold_roll(TableTransportChannel::Host, 1)).await;
    assert_eq!(
        f.state()
            .await
            .rules
            .unwrap()
            .rolls
            .iter()
            .find(|r| r.request.id == raw.request.id)
            .unwrap()
            .request,
        raw.request
    );
    f.close().await;
}

#[tokio::test]
async fn activation_codec_preserves_omission_and_null_but_rejects_old_future_duplicate_and_unknown_authority()
 {
    let mut f = Box::pin(Fixture::new()).await;
    let original = export_campaign(&f.pool, f.campaign).await.unwrap();
    let anchor = CampaignState::decode_json(
        &original
            .snapshots
            .iter()
            .min_by_key(|s| s.event_sequence)
            .unwrap()
            .state_json,
    )
    .unwrap();
    let omitted = serde_json::to_value(&anchor).unwrap();
    assert!(omitted["table"].get("grapple_access").is_none());
    let mut null = omitted.clone();
    null["table"]["grapple_access"] = serde_json::Value::Null;
    assert_eq!(
        CampaignState::decode_json(&null.to_string()).unwrap(),
        anchor
    );
    Box::pin(f.activate()).await;
    let current = f.state().await;
    let encoded = serde_json::to_string(&current).unwrap();
    let duplicate = encoded.replacen(
        "\"grapple_access\":",
        "\"grapple_access\":null,\"grapple_access\":",
        1,
    );
    assert!(CampaignState::decode_json(&duplicate).is_err());
    let mut unknown = serde_json::to_value(&current).unwrap();
    unknown["table"]["grapple_access"]["version"] = serde_json::json!("UnknownGrappleV2");
    assert!(CampaignState::decode_json(&unknown.to_string()).is_err());
    let mut extra = serde_json::to_value(&current).unwrap();
    extra["table"]["grapple_access"]["authority"] = serde_json::json!(true);
    assert!(CampaignState::decode_json(&extra.to_string()).is_err());
    let codec = dmd_persistence::CampaignStateSnapshotCodec::new();
    let mut old = omitted;
    old["schema_version"] = serde_json::json!(3);
    assert_eq!(codec.decode_state(3, &old.to_string()).unwrap(), anchor);
    old["table"]["grapple_access"] = serde_json::to_value(
        current
            .table
            .as_ref()
            .unwrap()
            .grapple_access
            .as_ref()
            .unwrap(),
    )
    .unwrap();
    assert!(
        matches!(codec.decode_state(3,&old.to_string()),Err(dmd_persistence::SnapshotCodecError::MigrationFailed{message,..}) if message.contains("tactical encounter"))
    );
    let option = f
        .view(f.pc(0))
        .await
        .grapple
        .unwrap()
        .choices
        .into_iter()
        .next()
        .unwrap();
    let mut request = serde_json::to_value(
        f.request(
            f.pc(0),
            TableTransportInput::GrappleChoice { handle: option.key },
        )
        .await,
    )
    .unwrap();
    request["input"]["GrappleChoice"]["grip"] = serde_json::json!(GrappleId::from_declaration(
        CommandId::new(),
        f.actors[0],
        f.goblin,
        Hand::Left
    ));
    assert!(serde_json::from_value::<TableTransportRequest>(request).is_err());
    f.close().await;
}

#[tokio::test]
async fn actual_after_equip_then_before_stow_support_two_private_hands_without_fabricated_items() {
    let mut f = Box::pin(Fixture::new()).await;
    Box::pin(f.activate()).await;
    let pc = f.pc(0);
    let actor = f.actors[0];
    let weapon = f
        .state()
        .await
        .items
        .values()
        .find(|item| item.custody == Custody::Entity(actor) && item.definition_id == "dagger")
        .unwrap()
        .id;
    let start = f
        .choose(pc.clone(), "Grapple Small armored figure with left hand")
        .await;
    Box::pin(f.cold(start)).await;
    let save = f
        .choose(TableTransportChannel::Host, "Resist Grapple with Strength")
        .await;
    Box::pin(f.cold(save)).await;
    Box::pin(f.cold_roll(TableTransportChannel::Host, 1)).await;
    let offered = f.view(pc.clone()).await.grapple.unwrap();
    assert!(
        !offered
            .choices
            .iter()
            .any(|choice| choice.label == "After Grapple: equip in left hand Dagger")
    );
    let equip = f
        .choose(pc.clone(), "After Grapple: equip in right hand Dagger")
        .await;
    Box::pin(f.cold(equip.clone())).await;
    let first = f.state().await;
    assert_eq!(
        first
            .rules
            .as_ref()
            .unwrap()
            .tactical_inventory
            .as_ref()
            .unwrap()
            .loadout(actor)
            .unwrap()
            .hands
            .hands[1],
        HandAssignment::Item(weapon)
    );
    assert_eq!(
        first
            .rules
            .as_ref()
            .unwrap()
            .tactical_grapples
            .as_ref()
            .unwrap()
            .active
            .len(),
        1
    );
    Box::pin(f.cold_action(pc.clone(), TacticalAction::EndTurn)).await;
    Box::pin(f.cold_action(TableTransportChannel::Host, TacticalAction::EndTurn)).await;
    let outsider = f.view(f.pc(1)).await;
    let start = f
        .choose(
            pc.clone(),
            "Grapple Small armored figure with right hand; first stow Dagger",
        )
        .await;
    Box::pin(f.cold(start)).await;
    assert_eq!(f.view(f.pc(1)).await, outsider);
    let save = f
        .choose(TableTransportChannel::Host, "Resist Grapple with Strength")
        .await;
    Box::pin(f.cold(save)).await;
    Box::pin(f.cold_roll(TableTransportChannel::Host, 1)).await;
    let dual = f.state().await;
    assert_eq!(
        dual.rules
            .as_ref()
            .unwrap()
            .tactical_grapples
            .as_ref()
            .unwrap()
            .active
            .len(),
        2
    );
    assert_eq!(
        dual.rules
            .as_ref()
            .unwrap()
            .tactical_inventory
            .as_ref()
            .unwrap()
            .loadout(actor)
            .unwrap()
            .hands
            .hands,
        [HandAssignment::Free, HandAssignment::Free]
    );
    assert!(
        dual.encounter
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .resolution
            .is_none(),
        "the accepted before operation consumes this attack's equipment allowance"
    );
    assert_eq!(f.view(f.pc(1)).await, outsider);
    let options = f.view(pc.clone()).await.grapple.unwrap();
    assert_eq!(
        options
            .choices
            .iter()
            .filter(|choice| choice.label.starts_with("Release "))
            .count(),
        2
    );
    assert!(
        !options
            .choices
            .iter()
            .any(|choice| choice.label.starts_with("After Grapple:"))
    );
    let release = f
        .choose(pc.clone(), "Release Small armored figure from left hand")
        .await;
    Box::pin(f.cold(release)).await;
    assert_eq!(
        f.state()
            .await
            .rules
            .unwrap()
            .tactical_grapples
            .unwrap()
            .active
            .len(),
        1
    );
    assert_eq!(f.view(f.pc(1)).await, outsider);
    let release = f
        .choose(pc, "Release Small armored figure from right hand")
        .await;
    Box::pin(f.cold(release)).await;
    assert!(f.state().await.rules.unwrap().tactical_grapples.is_none());
    assert_eq!(f.view(f.pc(1)).await, outsider);
    f.close().await;
}

#[tokio::test]
async fn open_and_resume_replay_original_activation_when_current_and_latest_marker_are_removed() {
    let mut f = Box::pin(Fixture::new()).await;
    Box::pin(f.activate()).await;
    let original = export_campaign(&f.pool, f.campaign).await.unwrap();
    let mut missing = f.state().await;
    assert!(missing.rules.as_ref().unwrap().tactical_grapples.is_none());
    missing.table.as_mut().unwrap().grapple_access = None;
    for matching_snapshot in [false, true] {
        let mut forged = original.clone();
        forged.current_state.state_json = missing.encode_json().unwrap();
        if matching_snapshot {
            let head = forged.current_state.applied_event_sequence;
            if let Some(snapshot) = forged
                .snapshots
                .iter_mut()
                .find(|snapshot| snapshot.event_sequence == head)
            {
                snapshot.state_json = forged.current_state.state_json.clone();
            } else {
                // Hostile recovery material only: never admitted as a gameplay
                // producer or used for a positive application assertion.
                let mut snapshot = forged.snapshots[0].clone();
                snapshot.event_sequence = head;
                snapshot.state_schema_version = forged.current_state.schema_version;
                snapshot.state_json = forged.current_state.state_json.clone();
                forged.snapshots.push(snapshot);
            }
        }
        Box::pin(hostile_destination(&forged)).await;
        let path = f
            .directory
            .join(format!("hostile-open-{matching_snapshot}.sqlite"));
        let pool = open_sqlite_path(&path).await.unwrap();
        let app = runtime(pool.clone());
        app.create_table_campaign(
            CampaignId::new(),
            "Unrelated existing file campaign",
            TableContract::default(),
        )
        .await
        .unwrap();
        // The lower structural persistence API deliberately installs a hostile
        // image, so open/resume must independently enforce semantic original
        // replay even when no mutable current marker survives.
        dmd_persistence::restore_campaign(&pool, &forged)
            .await
            .unwrap();
        let before = all_rows(&pool).await;
        assert!(app.open_campaign(f.campaign).await.is_err());
        assert!(app.resume_campaign(f.campaign).await.is_err());
        assert_eq!(all_rows(&pool).await, before);
        pool.close().await;
    }
    f.close().await;
}

#[path = "support/table_grapple_public_corrections.rs"]
mod corrections;

#[path = "support/table_grapple_ground_transport.rs"]
mod ground_transport;

#[path = "support/table_grapple_inspiration.rs"]
mod inspiration;
