use dmd_app::*;
use dmd_domain::*;
use dmd_persistence::{export_campaign, open_sqlite};
use dmd_rules::CharacterCreationInput;
use std::path::Path;

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
            item_id: "leather".into(),
            quantity: 1,
        }],
        worn_armor: Some("leather".into()),
        shield: false,
        masteries: ["club".into(), "dagger".into(), "shortbow".into()],
    }
}

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
        let pool = open_sqlite("sqlite::memory:").await.unwrap();
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
            .create_table_campaign(
                f.campaign,
                "An independent campaign",
                TableContract::default(),
            )
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
                    input: input(&format!("Character {i}")),
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
