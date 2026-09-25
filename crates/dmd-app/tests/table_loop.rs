use dmd_app::*;
use dmd_domain::*;
use dmd_persistence::{export_campaign, open_sqlite};
use dmd_rules::CharacterCreationInput;
use std::path::Path;

#[path = "support/table_area_cases.rs"]
mod table_area_cases;

#[path = "support/table_attack_cases.rs"]
mod table_attack_cases;
#[path = "support/table_casting_cases.rs"]
mod table_casting_cases;
#[path = "support/table_creature_cases.rs"]
mod table_creature_cases;
#[path = "support/table_dead_target_cases.rs"]
mod table_dead_target_cases;
#[path = "support/table_falling_cases.rs"]
mod table_falling_cases;
#[path = "support/table_medicine_cases.rs"]
mod table_medicine_cases;
#[path = "support/table_night_hag_cases.rs"]
mod table_night_hag_cases;
#[path = "support/table_oa_concentration_cases.rs"]
mod table_oa_concentration_cases;
#[path = "support/table_projection_cases.rs"]
mod table_projection_cases;
#[path = "support/table_ready_cases.rs"]
mod table_ready_cases;
#[path = "support/table_savage_cases.rs"]
mod table_savage_cases;
#[path = "support/table_shield_cases.rs"]
mod table_shield_cases;
#[path = "support/table_source_control_cases.rs"]
mod table_source_control_cases;
#[path = "support/table_tactical_cases.rs"]
mod table_tactical_cases;
#[path = "support/table_turn_core_cases.rs"]
mod table_turn_core_cases;
#[path = "support/table_unarmed_cases.rs"]
mod table_unarmed_cases;

fn input(name: &str) -> CharacterCreationInput {
    CharacterCreationInput {
        name: name.into(),
        pronouns: "they/them".into(),
        description: "A traveler".into(),
        alignment: "Neutral Good".into(),
        backstory: "A private aspiration, not established world truth".into(),
        ability_scores: [15, 14, 13, 8, 10, 12],
        background_boosts: [2, 0, 1, 0, 0, 0],
        fighter_skills: [Skill::Perception, Skill::Survival],
        human_skill: Skill::Insight,
        skilled_skills: [Skill::Acrobatics, Skill::Stealth, Skill::Investigation],
        size: CharacterSize::Medium,
        languages: ["dwarvish".into(), "elvish".into()],
        fighting_style: FightingStyle::Defense,
        gaming_set: GamingSet::Dice,
        purchases: vec![EquipmentChoice {
            item_id: "leather-armor".into(),
            quantity: 1,
        }],
        worn_armor: Some("leather-armor".into()),
        shield: false,
        masteries: ["club".into(), "dagger".into(), "shortbow".into()],
    }
}

#[path = "support/sqlite_test_cleanup.rs"]
mod sqlite_test_cleanup;

struct Fixture {
    runtime: CampaignRuntime,
    pool: sqlx::SqlitePool,
    campaign: CampaignId,
    players: [PlayerId; 2],
    characters: [CharacterId; 2],
    actors: [EntityId; 2],
    session: PlaySessionId,
}
impl Fixture {
    async fn new() -> Self {
        Box::pin(Self::with_contract(TableContract::default())).await
    }
    async fn with_contract(contract: TableContract) -> Self {
        Box::pin(Self::with_creation(contract, None)).await
    }
    async fn with_creation(
        contract: TableContract,
        first_character: Option<CharacterCreationInput>,
    ) -> Self {
        let pool = open_sqlite("sqlite::memory:").await.unwrap();
        // Keep nested setup phases on the heap so each caller's scenario does not
        // carry another copy of the complete async creation/transaction frame.
        Box::pin(Self::with_creation_pool(contract, first_character, pool)).await
    }
    async fn with_pool(contract: TableContract, pool: sqlx::SqlitePool) -> Self {
        Box::pin(Self::with_creation_pool(contract, None, pool)).await
    }
    async fn with_creation_pool(
        contract: TableContract,
        first_character: Option<CharacterCreationInput>,
        pool: sqlx::SqlitePool,
    ) -> Self {
        let runtime = CampaignRuntime::from_content_root(
            pool.clone(),
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../content"),
        );
        let mut f = Self {
            runtime,
            pool,
            campaign: CampaignId::new(),
            players: [PlayerId::new(), PlayerId::new()],
            characters: [CharacterId::new(), CharacterId::new()],
            actors: [EntityId::new(), EntityId::new()],
            session: PlaySessionId::new(),
        };
        f.runtime
            .create_table_campaign(f.campaign, "An independent campaign", contract)
            .await
            .unwrap();
        for i in 0..2 {
            f.host(
                TableAction::AddPlayer {
                    id: f.players[i],
                    name: format!("Player {i}"),
                },
                None,
            )
            .await;
            f.host(
                TableAction::CreateCharacter {
                    character_id: f.characters[i],
                    entity_id: f.actors[i],
                    player_id: f.players[i],
                    input: if i == 0 {
                        first_character
                            .clone()
                            .unwrap_or_else(|| input("Character 0"))
                    } else {
                        input("Character 1")
                    },
                },
                None,
            )
            .await;
        }
        f.host(
            TableAction::StartSession {
                id: f.session,
                name: "First session".into(),
                participants: vec![
                    SessionParticipant {
                        player_id: f.players[0],
                        character_id: Some(f.characters[0]),
                        attendance: AttendanceStatus::Present,
                    },
                    SessionParticipant {
                        player_id: f.players[1],
                        character_id: Some(f.characters[1]),
                        attendance: AttendanceStatus::Absent,
                    },
                ],
            },
            Some(f.session),
        )
        .await;
        f.host(
            TableAction::SetSituation {
                situation: TableSituation {
                    title: "Rocky pass".into(),
                    description: "A ledge blocks the route.".into(),
                    challenges: vec![TableChallenge {
                        id: "ledge".into(),
                        title: "Scale the ledge".into(),
                        description: "The stone offers difficult handholds.".into(),
                        phrases: vec!["climb".into(), "scale".into()],
                        kind: TestKind::Check {
                            ability: Ability::Strength,
                            skill: Some(Skill::Athletics),
                        },
                        dc: 15,
                        success: "You reach the upper path.".into(),
                        failure: "You remain below the ledge.".into(),
                        resolution: None,
                    }],
                },
            },
            Some(f.session),
        )
        .await;
        f
    }
    async fn meta(
        &self,
        issuer: CommandIssuer,
        actor: Option<EntityId>,
        session_id: Option<PlaySessionId>,
    ) -> CommandMeta {
        CommandMeta {
            id: CommandId::new(),
            campaign_id: self.campaign,
            session_id,
            issuer,
            actor: actor.map(AgentRef::Entity),
            expected_event_sequence: self
                .runtime
                .open_campaign(self.campaign)
                .await
                .unwrap()
                .state()
                .applied_event_sequence,
        }
    }
    async fn host(&mut self, action: TableAction, session: Option<PlaySessionId>) -> TableReceipt {
        let meta = self.meta(CommandIssuer::Admin, None, session).await;
        self.runtime.execute_table(meta, action).await.unwrap()
    }
    async fn player_meta(&self, index: usize) -> CommandMeta {
        self.meta(
            CommandIssuer::Player(self.players[index]),
            Some(self.actors[index]),
            Some(self.session),
        )
        .await
    }
}

#[tokio::test]
async fn alternatives_and_unsupported_extra_actions_wait_without_spending_resources() {
    let f = Fixture::new().await;
    for text in [
        "I use Second Wind or drink a potion",
        "I climb the ledge or wait here",
        "I use Second Wind and attack the guard",
    ] {
        f.runtime
            .submit_table_text(f.player_meta(0).await, text)
            .await
            .unwrap();
        let before = f
            .runtime
            .open_campaign(f.campaign)
            .await
            .unwrap()
            .state()
            .clone();
        let pending = before.table.as_ref().unwrap().pending.as_ref().unwrap();
        assert!(matches!(pending.intent, TableIntent::Unresolved { .. }));
        let action = TableAction::Adjudicate {
            pending_id: pending.id,
            revision: pending.revision,
            request_id: RollRequestId::new(),
        };
        assert!(matches!(
            f.runtime
                .execute_table(
                    f.meta(CommandIssuer::Admin, None, Some(f.session)).await,
                    action
                )
                .await,
            Err(RunnableCampaignError::TableRejected(_))
        ));
        assert_eq!(
            &before,
            f.runtime.open_campaign(f.campaign).await.unwrap().state()
        );
        f.runtime
            .execute_table(
                f.player_meta(0).await,
                TableAction::CancelDecision {
                    pending_id: pending.id,
                    revision: pending.revision,
                },
            )
            .await
            .unwrap();
    }
}

#[tokio::test]
async fn observation_storage_failure_keeps_request_retryable_and_recovers_exactly_once() {
    let f = Fixture::new().await;
    let meta = f.player_meta(0).await;
    sqlx::query("CREATE TRIGGER reject_observation BEFORE INSERT ON session_observations BEGIN SELECT RAISE(FAIL, 'temporary write failure'); END")
        .execute(&f.pool).await.unwrap();
    assert!(matches!(
        f.runtime
            .submit_table_text(meta.clone(), "How do I roll?")
            .await,
        Err(RunnableCampaignError::Table(_))
    ));
    sqlx::query("DROP TRIGGER reject_observation")
        .execute(&f.pool)
        .await
        .unwrap();
    let accepted = f
        .runtime
        .submit_table_text(meta.clone(), "How do I roll?")
        .await
        .unwrap();
    let retried = f
        .runtime
        .submit_table_text(meta.clone(), "How do I roll?")
        .await
        .unwrap();
    assert_eq!(accepted, retried);
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM session_observations")
        .fetch_one(&f.pool)
        .await
        .unwrap();
    assert_eq!(count, 1);
    assert!(matches!(
        f.runtime.submit_table_text(meta, "Different input").await,
        Err(RunnableCampaignError::TableRejected(_))
    ));
}

#[tokio::test]
async fn unavailable_receipt_lookup_never_claims_a_prior_action_was_rejected() {
    let f = Fixture::new().await;
    let meta = f.player_meta(0).await;
    let action = TableAction::Declare {
        text: "I climb the wall".into(),
    };
    let receipt = f
        .runtime
        .execute_table(meta.clone(), action.clone())
        .await
        .unwrap();
    sqlx::query("ALTER TABLE command_audit RENAME TO temporarily_unavailable_audit")
        .execute(&f.pool)
        .await
        .unwrap();
    assert!(matches!(
        f.runtime.execute_table(meta.clone(), action.clone()).await,
        Err(RunnableCampaignError::Table(_))
    ));
    sqlx::query("ALTER TABLE temporarily_unavailable_audit RENAME TO command_audit")
        .execute(&f.pool)
        .await
        .unwrap();
    let retried = f.runtime.execute_table(meta, action).await.unwrap();
    assert!(retried.already_accepted);
    assert_eq!(receipt.command_id, retried.command_id);
    assert_eq!(receipt.event_sequence, retried.event_sequence);
    assert_eq!(receipt.outcome, retried.outcome);
}

#[tokio::test]
async fn normal_scene_corrects_pending_accepts_raw_faces_and_retries_without_duplicates() {
    let mut f = Fixture::new().await;
    let before = f
        .runtime
        .open_campaign(f.campaign)
        .await
        .unwrap()
        .state()
        .clone();
    let question = f.player_meta(0).await;
    let response = f
        .runtime
        .submit_table_text(question.clone(), "How do I roll with advantage?")
        .await
        .unwrap();
    assert!(matches!(response, TableTextResult::Observed(_)));
    let mut changed_actor = question.clone();
    changed_actor.actor = Some(AgentRef::Entity(f.actors[1]));
    assert!(
        f.runtime
            .submit_table_text(changed_actor, "How do I roll with advantage?")
            .await
            .is_err()
    );
    assert_eq!(
        before,
        *f.runtime.open_campaign(f.campaign).await.unwrap().state()
    );
    f.runtime
        .submit_table_text(f.player_meta(0).await, "I climb the ledge")
        .await
        .unwrap();
    let original = f
        .runtime
        .table_view(f.campaign, TableViewer::Player(f.players[0]))
        .await
        .unwrap()
        .pending
        .unwrap();
    f.runtime
        .submit_table_text(f.player_meta(0).await, "Actually I ask about a rope")
        .await
        .unwrap();
    let revised = f
        .runtime
        .table_view(f.campaign, TableViewer::Player(f.players[0]))
        .await
        .unwrap()
        .pending
        .unwrap();
    assert_eq!(original.id, revised.id);
    assert_eq!(revised.revision, 1);
    assert!(matches!(revised.intent, TableIntent::Unresolved { .. }));
    assert!(
        f.runtime
            .execute_table(
                f.meta(CommandIssuer::Admin, None, Some(f.session)).await,
                TableAction::Adjudicate {
                    pending_id: original.id,
                    revision: original.revision,
                    request_id: RollRequestId::new()
                }
            )
            .await
            .is_err()
    );
    f.runtime
        .submit_table_text(
            f.player_meta(0).await,
            "Actually I carefully scale the ledge",
        )
        .await
        .unwrap();
    let pending = f
        .runtime
        .table_view(f.campaign, TableViewer::Player(f.players[0]))
        .await
        .unwrap()
        .pending
        .unwrap();
    let request_id = RollRequestId::new();
    f.host(
        TableAction::Adjudicate {
            pending_id: pending.id,
            revision: pending.revision,
            request_id,
        },
        Some(f.session),
    )
    .await;
    let requested = f
        .runtime
        .table_view(f.campaign, TableViewer::Player(f.players[0]))
        .await
        .unwrap();
    assert_eq!(requested.roll.as_ref().unwrap().modifier, 5);
    assert_eq!(
        requested.roll.as_ref().unwrap().reason,
        "Strength (Athletics) check"
    );
    let before_invalid = f
        .runtime
        .open_campaign(f.campaign)
        .await
        .unwrap()
        .state()
        .clone();
    assert!(
        f.runtime
            .execute_table(
                f.player_meta(0).await,
                TableAction::SubmitPhysical {
                    request_id,
                    faces: vec![21]
                }
            )
            .await
            .is_err()
    );
    assert_eq!(
        before_invalid,
        *f.runtime.open_campaign(f.campaign).await.unwrap().state()
    );
    let meta = f.player_meta(0).await;
    let action = TableAction::SubmitPhysical {
        request_id,
        faces: vec![12],
    };
    let receipt = f
        .runtime
        .execute_table(meta.clone(), action.clone())
        .await
        .unwrap();
    assert!(receipt.outcome.message.contains("17"));
    assert!(receipt.outcome.message.contains("upper path"));
    let repeat = f.runtime.execute_table(meta, action).await.unwrap();
    assert!(repeat.already_accepted);
    assert_eq!(receipt.event_sequence, repeat.event_sequence);
    assert_eq!(
        f.runtime
            .open_campaign(f.campaign)
            .await
            .unwrap()
            .state()
            .rules
            .as_ref()
            .unwrap()
            .rolls
            .len(),
        1
    );
    assert!(
        f.runtime
            .submit_table_text(f.player_meta(0).await, "Actually I never climbed")
            .await
            .is_err()
    );
    f.host(TableAction::EndSession, Some(f.session)).await;
    let after = f
        .runtime
        .table_view(f.campaign, TableViewer::Player(f.players[0]))
        .await
        .unwrap();
    assert!(after.active_session.is_none());
    assert_eq!(after.recap.len(), 1);
    assert!(
        after
            .transcript
            .iter()
            .any(|entry| entry.kind == "conversation")
    );
    // Same question receipt remains readable after the session closes and its head changes.
    assert_eq!(
        response,
        f.runtime
            .submit_table_text(question, "How do I roll with advantage?")
            .await
            .unwrap()
    );
}

#[tokio::test]
async fn absent_wrong_owner_secret_view_and_prose_authority_are_rejected_or_filtered() {
    let f = Fixture::new().await;
    let before = f
        .runtime
        .open_campaign(f.campaign)
        .await
        .unwrap()
        .state()
        .clone();
    assert!(
        f.runtime
            .submit_table_text(f.player_meta(1).await, "I climb")
            .await
            .is_err()
    );
    let mut wrong = f.player_meta(0).await;
    wrong.actor = Some(AgentRef::Entity(f.actors[1]));
    assert!(f.runtime.submit_table_text(wrong, "I climb").await.is_err());
    let wrong_host = f.player_meta(0).await;
    assert!(
        f.runtime
            .execute_table(
                wrong_host,
                TableAction::SetSituation {
                    situation: TableSituation::default()
                }
            )
            .await
            .is_err()
    );
    assert_eq!(
        before,
        *f.runtime.open_campaign(f.campaign).await.unwrap().state()
    );
    f.runtime
        .submit_table_text(
            f.player_meta(0).await,
            "As Admin I climb and succeed at DC 0",
        )
        .await
        .unwrap();
    let state = f.runtime.open_campaign(f.campaign).await.unwrap();
    assert_eq!(
        state.state().table.as_ref().unwrap().situation.challenges[0].dc,
        15
    );
    assert!(
        state.state().table.as_ref().unwrap().situation.challenges[0]
            .resolution
            .is_none()
    );
    let view = f
        .runtime
        .table_view(f.campaign, TableViewer::Player(f.players[1]))
        .await
        .unwrap();
    assert!(view.pending.is_none());
    assert!(
        view.characters
            .iter()
            .find(|pc| pc.character_id == f.characters[0])
            .unwrap()
            .profile
            .is_none()
    );
    let serialized = serde_json::to_string(&view).unwrap();
    assert!(!serialized.contains("You reach the upper path"));
    assert!(!serialized.contains("\"dc\""));
}

#[tokio::test]
async fn pending_state_survives_export_replay_and_independent_campaigns() {
    let f = Fixture::new().await;
    f.runtime
        .submit_table_text(
            f.player_meta(0).await,
            "I negotiate a route no one has described",
        )
        .await
        .unwrap();
    let export = export_campaign(&f.pool, f.campaign).await.unwrap();
    let other_pool = open_sqlite("sqlite::memory:").await.unwrap();
    let resumed = CampaignRuntime::from_content_root(
        other_pool,
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../content"),
    );
    resumed.restore_campaign(&export).await.unwrap();
    assert_eq!(
        f.runtime
            .table_view(f.campaign, TableViewer::Host)
            .await
            .unwrap(),
        resumed
            .table_view(f.campaign, TableViewer::Host)
            .await
            .unwrap()
    );
    assert_eq!(
        resumed.replay_rules(f.campaign).await.unwrap(),
        *resumed.open_campaign(f.campaign).await.unwrap().state()
    );
    let second = CampaignId::new();
    resumed
        .create_table_campaign(second, "Unrelated second world", TableContract::default())
        .await
        .unwrap();
    assert!(
        resumed
            .table_view(second, TableViewer::Host)
            .await
            .unwrap()
            .pending
            .is_none()
    );
    assert!(
        resumed
            .table_view(f.campaign, TableViewer::Host)
            .await
            .unwrap()
            .pending
            .is_some()
    );
}

#[tokio::test]
async fn supported_profiles_require_their_authoritative_mechanics_and_feature_grants() {
    let f = Fixture::new().await;
    let state = f
        .runtime
        .open_campaign(f.campaign)
        .await
        .unwrap()
        .state()
        .clone();
    let mut missing = state.clone();
    missing
        .rules
        .as_mut()
        .unwrap()
        .entities
        .remove(&f.actors[0]);
    assert!(
        matches!(f.runtime.create_campaign(&missing).await,Err(RunnableCampaignError::Table(message)) if message.contains("mechanical sheet"))
    );
    let mut missing_grants = state.clone();
    missing_grants
        .rules
        .as_mut()
        .unwrap()
        .entities
        .get_mut(&f.actors[0])
        .unwrap()
        .character_features = None;
    assert!(
        matches!(f.runtime.create_campaign(&missing_grants).await,Err(RunnableCampaignError::Table(message)) if message.contains("feature grants"))
    );
    // Earliest imported/backfilled anchors have no earlier creation event to replay.
    // Their source profile must still constrain every immutable mechanical grant.
    let changes: [fn(&mut MechanicalEntity); 10] = [
        |entity| {
            entity.max_hp = 100;
            entity.hp = 100;
        },
        |entity| entity.hit_dice.sides = 12,
        |entity| entity.hit_dice.maximum = 2,
        |entity| {
            entity.saving_proficiencies.insert(Ability::Wisdom);
        },
        |entity| {
            entity
                .skill_proficiencies
                .insert(Skill::Athletics, Proficiency::Expertise);
        },
        |entity| {
            entity.armor = ArmorClass::Armor {
                base: 18,
                dexterity_cap: Some(0),
                shield: true,
            }
        },
        |entity| {
            entity.attacks.insert("club".into());
        },
        |entity| {
            entity.attack_proficiencies.insert("club".into());
        },
        |entity| {
            entity.resistances.insert(DamageType::Fire);
        },
        |entity| entity.uses_death_saves = false,
    ];
    for (index, change) in changes.into_iter().enumerate() {
        let mut malformed = state.clone();
        change(
            malformed
                .rules
                .as_mut()
                .unwrap()
                .entities
                .get_mut(&f.actors[0])
                .unwrap(),
        );
        assert!(
            matches!(
                f.runtime.create_campaign(&malformed).await,
                Err(RunnableCampaignError::Table(_))
            ),
            "source grant mutation {index} must fail before persistence"
        );
    }
}

#[tokio::test]
async fn selected_house_rule_is_inherited_by_creation_and_explained_as_a_house_rule() {
    let mut contract = TableContract::default();
    contract.house_rules.ability_test_natural_extremes = true;
    contract.house_rule_notes = "Natural 1 and 20 determine ability test results.".into();
    let f = Fixture::with_contract(contract.clone()).await;
    assert_eq!(
        f.runtime
            .open_campaign(f.campaign)
            .await
            .unwrap()
            .state()
            .rules
            .as_ref()
            .unwrap()
            .house_rules,
        contract.house_rules
    );
    let reply = f
        .runtime
        .submit_table_text(f.player_meta(0).await, "How do I roll?")
        .await
        .unwrap();
    assert!(
        matches!(reply,TableTextResult::Observed(body) if body.answer.contains("Your table explicitly enabled the house rule"))
    );
}

#[tokio::test]
async fn campaign_creation_retry_recovers_existing_progress_without_reinitializing() {
    let f = Fixture::new().await;
    let before = f
        .runtime
        .table_view(f.campaign, TableViewer::Host)
        .await
        .unwrap();
    let retried = f
        .runtime
        .create_table_campaign(
            f.campaign,
            "An independent campaign",
            TableContract::default(),
        )
        .await
        .unwrap();
    assert_eq!(before, retried);
    assert!(
        f.runtime
            .create_table_campaign(
                f.campaign,
                "A different creation request",
                TableContract::default()
            )
            .await
            .is_err()
    );
    assert_eq!(
        before,
        f.runtime
            .table_view(f.campaign, TableViewer::Host)
            .await
            .unwrap()
    );
}

#[tokio::test]
async fn second_wind_pending_roll_resources_and_transcript_survive_database_reopen() {
    let f = Fixture::new().await;
    f.runtime
        .submit_table_text(f.player_meta(0).await, "I use Second Wind")
        .await
        .unwrap();
    let pending = f
        .runtime
        .table_view(f.campaign, TableViewer::Host)
        .await
        .unwrap()
        .pending
        .unwrap();
    let request_id = RollRequestId::new();
    let meta = f.meta(CommandIssuer::Admin, None, Some(f.session)).await;
    let action = TableAction::Adjudicate {
        pending_id: pending.id,
        revision: pending.revision,
        request_id,
    };
    f.runtime
        .execute_table(meta.clone(), action.clone())
        .await
        .unwrap();
    assert!(
        f.runtime
            .execute_table(meta, action)
            .await
            .unwrap()
            .already_accepted
    );
    let expected = f
        .runtime
        .table_view(f.campaign, TableViewer::Player(f.players[0]))
        .await
        .unwrap();
    assert_eq!(
        expected.roll.as_ref().unwrap().reason,
        "Second Wind healing"
    );
    assert_eq!(
        expected
            .characters
            .iter()
            .find(|pc| pc.character_id == f.characters[0])
            .unwrap()
            .second_wind_remaining,
        Some(1)
    );
    let export = export_campaign(&f.pool, f.campaign).await.unwrap();
    let directory = std::env::temp_dir().join(format!("dmd-table-reopen-{}", f.campaign.0));
    std::fs::create_dir(&directory).unwrap();
    let database = format!("sqlite:{}", directory.join("campaign.sqlite").display());
    let content = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../content");
    let pool = open_sqlite(&database).await.unwrap();
    let restored = CampaignRuntime::from_content_root(pool.clone(), &content);
    restored.restore_campaign(&export).await.unwrap();
    drop(restored);
    pool.close().await;
    let reopened_pool = open_sqlite(&database).await.unwrap();
    let reopened = CampaignRuntime::from_content_root(reopened_pool.clone(), content);
    assert_eq!(
        expected,
        reopened
            .table_view(f.campaign, TableViewer::Player(f.players[0]))
            .await
            .unwrap()
    );
    let meta = f.player_meta(0).await;
    let result = reopened
        .execute_table(
            meta,
            TableAction::SubmitPhysical {
                request_id,
                faces: vec![6],
            },
        )
        .await
        .unwrap();
    assert!(result.outcome.message.contains("total 7"));
    assert!(
        reopened
            .table_view(f.campaign, TableViewer::Host)
            .await
            .unwrap()
            .roll
            .is_none()
    );
    // The old direct entrypoint cannot bypass a table campaign's pending-action composition.
    let state = reopened.open_campaign(f.campaign).await.unwrap();
    assert!(
        reopened
            .execute_rules(
                RulesContext {
                    campaign_id: f.campaign,
                    issuer: CommandIssuer::Admin,
                    actor: None,
                    session_id: Some(f.session),
                    expected_event_sequence: state.state().applied_event_sequence
                },
                dmd_rules::RulesAction::AdvanceTime {
                    seconds: 1,
                    ruling: Ruling {
                        basis: RulingBasis::GmAdjudication,
                        reason: "Bypass attempt".into()
                    }
                }
            )
            .await
            .is_err()
    );
    drop(state);
    drop(reopened);
    reopened_pool.close().await;
    drop(reopened_pool);
    drop(pool);
    sqlite_test_cleanup::remove_closed_directory(&directory)
        .await
        .unwrap();
}

#[tokio::test]
async fn equipment_preparation_is_exactly_once_private_and_replayable() {
    let directory = std::env::temp_dir().join(format!("dmd-equipment-{}", CampaignId::new().0));
    std::fs::create_dir_all(&directory).unwrap();
    let database = directory.join("campaign.sqlite");
    let url = format!("sqlite://{}", database.display());
    let pool = open_sqlite(&url).await.unwrap();
    let mut f = Fixture::with_pool(TableContract::default(), pool).await;
    let view = f
        .runtime
        .table_view(f.campaign, TableViewer::Host)
        .await
        .unwrap();
    let equipment = view
        .characters
        .iter()
        .find(|c| c.character_id == f.characters[0])
        .unwrap()
        .equipment
        .as_ref()
        .unwrap();
    assert!(!equipment.prepared);
    let ids = (0..equipment.initial_item_count)
        .map(|_| ItemId::new())
        .collect::<Vec<_>>();
    let action = TableAction::PrepareEquipment {
        character_id: f.characters[0],
        item_ids: ids.clone(),
    };
    let player = f.player_meta(0).await;
    assert!(
        f.runtime
            .execute_table(player, action.clone())
            .await
            .is_err()
    );
    let meta = f.meta(CommandIssuer::Admin, None, Some(f.session)).await;
    let receipt = f
        .runtime
        .execute_table(meta.clone(), action.clone())
        .await
        .unwrap();
    // Lose the original process and retry its exact command against the reopened file.
    f.pool.close().await;
    f.pool = open_sqlite(&url).await.unwrap();
    f.runtime = CampaignRuntime::from_content_root(
        f.pool.clone(),
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../content"),
    );
    let repeated = f.runtime.execute_table(meta, action.clone()).await.unwrap();
    assert!(repeated.already_accepted);
    assert_eq!(receipt.event_sequence, repeated.event_sequence);
    let accepted = f
        .runtime
        .open_campaign(f.campaign)
        .await
        .unwrap()
        .state()
        .clone();
    assert_eq!(accepted.items.len(), ids.len());
    let pack =
        dmd_rules::RulesPack::from_json(include_str!("../../../content/srd-5.2.1/kernel.json"))
            .unwrap();
    let bypass_meta = f.meta(CommandIssuer::Admin, None, Some(f.session)).await;
    let ruling = Ruling {
        basis: RulingBasis::GmAdjudication,
        reason: "Legacy bypass attempt".into(),
    };
    for bypass in [
        dmd_rules::RulesAction::ApplyDamage {
            target: f.actors[0],
            amount: 100,
            damage_type: DamageType::Bludgeoning,
            critical: false,
            ruling: ruling.clone(),
        },
        dmd_rules::RulesAction::Heal {
            target: f.actors[0],
            amount: 1,
            ruling: ruling.clone(),
        },
        dmd_rules::RulesAction::AuthorizeAttack {
            actor: f.actors[0],
            target: f.actors[1],
            attack_id: "club".into(),
            circumstances: Circumstances::default(),
            within_five_feet: true,
            ruling,
        },
    ] {
        let error = dmd_rules::resolve(&accepted, &bypass_meta, &bypass, &pack).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("physical equipment requires the tactical action path")
        );
    }
    assert!(
        ids.iter()
            .all(|id| accepted.items[id].custody == Custody::Entity(f.actors[0]))
    );
    let newer = f.meta(CommandIssuer::Admin, None, Some(f.session)).await;
    assert!(f.runtime.execute_table(newer, action).await.is_err());
    assert_eq!(
        f.runtime.open_campaign(f.campaign).await.unwrap().state(),
        &accepted
    );
    let own = f
        .runtime
        .table_view(f.campaign, TableViewer::Player(f.players[0]))
        .await
        .unwrap();
    assert!(
        own.characters
            .iter()
            .find(|c| c.character_id == f.characters[0])
            .unwrap()
            .equipment
            .as_ref()
            .unwrap()
            .prepared
    );
    assert!(
        own.characters
            .iter()
            .find(|c| c.character_id == f.characters[1])
            .unwrap()
            .equipment
            .is_none()
    );
    let export = export_campaign(&f.pool, f.campaign).await.unwrap();
    let target = open_sqlite("sqlite::memory:").await.unwrap();
    let restored = CampaignRuntime::from_content_root(
        target.clone(),
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../content"),
    );
    restored.restore_campaign(&export).await.unwrap();
    assert_eq!(
        restored.resume_campaign(f.campaign).await.unwrap().state(),
        &accepted
    );
    let mut corrupted = export.clone();
    let mut current = CampaignState::decode_json(&corrupted.current_state.state_json).unwrap();
    current
        .rules
        .as_mut()
        .unwrap()
        .tactical_inventory
        .as_mut()
        .unwrap()
        .receipts[0]
        .command
        .id = CommandId::new();
    corrupted.current_state.state_json = current.encode_json().unwrap();
    let bad_pool = open_sqlite("sqlite::memory:").await.unwrap();
    let bad = CampaignRuntime::from_content_root(
        bad_pool.clone(),
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../content"),
    );
    assert!(bad.restore_campaign(&corrupted).await.is_err());
    assert!(
        dmd_persistence::open_campaign(&bad_pool, f.campaign)
            .await
            .is_err()
    );
    // A genuine but unrelated host command cannot be relabeled as a physical grant.
    let unrelated = export
        .event_journal
        .iter()
        .find_map(|row| {
            let event: TableEvent = serde_json::from_str(&row.payload_json).ok()?;
            matches!(event.action, TableAction::AddPlayer { .. }).then_some(event.meta)
        })
        .unwrap();
    let mut unrelated_origin = export.clone();
    let mut state = CampaignState::decode_json(&unrelated_origin.current_state.state_json).unwrap();
    let inventory = state
        .rules
        .as_mut()
        .unwrap()
        .tactical_inventory
        .as_mut()
        .unwrap();
    inventory.receipts[0].command = unrelated.clone();
    inventory.loadouts[0].command = unrelated;
    unrelated_origin.current_state.state_json = state.encode_json().unwrap();
    assert!(bad.restore_campaign(&unrelated_origin).await.is_err());
    assert!(
        dmd_persistence::open_campaign(&bad_pool, f.campaign)
            .await
            .is_err()
    );
    // A newly backfilled anchor cannot launder materialized authority out of its replay.
    let mut missing_anchor = export.clone();
    missing_anchor.snapshots = vec![dmd_persistence::SnapshotRow {
        campaign_id: missing_anchor.current_state.campaign_id.clone(),
        event_sequence: missing_anchor.current_state.applied_event_sequence,
        state_schema_version: missing_anchor.current_state.schema_version,
        state_json: missing_anchor.current_state.state_json.clone(),
        created_at_utc: "2026-09-25 00:00:00".into(),
    }];
    let error = bad.restore_campaign(&missing_anchor).await.unwrap_err();
    assert!(error.to_string().contains("original pre-tactical anchor"));
    assert!(
        dmd_persistence::open_campaign(&bad_pool, f.campaign)
            .await
            .is_err()
    );
    target.close().await;
    bad_pool.close().await;
    f.pool.close().await;
    drop(f);
    sqlite_test_cleanup::remove_closed_file(&database)
        .await
        .unwrap();
    // SQLite may retain journal sidecars until pool handles finish dropping.
    let _ = std::fs::remove_dir(directory);
}
