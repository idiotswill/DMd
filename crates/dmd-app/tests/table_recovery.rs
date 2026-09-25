use std::path::Path;

use dmd_app::*;
use dmd_domain::*;
use dmd_persistence::{CampaignExport, SnapshotRow, export_campaign, open_sqlite};
use dmd_rules::{CharacterCreationInput, RulesAction, RulesOutcome};

fn input() -> CharacterCreationInput {
    CharacterCreationInput {
        name: "Recovery character".into(),
        pronouns: "they/them".into(),
        description: "A traveler".into(),
        alignment: "Neutral Good".into(),
        backstory: "Personal history".into(),
        ability_scores: [15, 14, 13, 8, 10, 12],
        background_boosts: [2, 0, 1, 0, 0, 0],
        fighter_skills: [Skill::Perception, Skill::Survival],
        human_skill: Skill::Insight,
        skilled_skills: [Skill::Acrobatics, Skill::Stealth, Skill::Investigation],
        size: CharacterSize::Medium,
        languages: ["dwarvish".into(), "elvish".into()],
        fighting_style: FightingStyle::Defense,
        gaming_set: GamingSet::Dice,
        purchases: vec![],
        worn_armor: None,
        shield: false,
        masteries: ["club".into(), "dagger".into(), "shortbow".into()],
    }
}

async fn runtime() -> (sqlx::SqlitePool, CampaignRuntime) {
    let pool = open_sqlite("sqlite::memory:").await.unwrap();
    let runtime = CampaignRuntime::from_content_root(
        pool.clone(),
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../content"),
    );
    (pool, runtime)
}

struct Fixture {
    pool: sqlx::SqlitePool,
    runtime: CampaignRuntime,
    campaign: CampaignId,
    player: PlayerId,
    character: CharacterId,
    actor: EntityId,
    session: PlaySessionId,
}

impl Fixture {
    async fn new() -> Self {
        let (pool, runtime) = runtime().await;
        let f = Self {
            pool,
            runtime,
            campaign: CampaignId::new(),
            player: PlayerId::new(),
            character: CharacterId::new(),
            actor: EntityId::new(),
            session: PlaySessionId::new(),
        };
        f.runtime
            .create_table_campaign(f.campaign, "Recovery table", TableContract::default())
            .await
            .unwrap();
        for action in [
            TableAction::AddPlayer {
                id: f.player,
                name: "Attending player".into(),
            },
            TableAction::CreateCharacter {
                character_id: f.character,
                entity_id: f.actor,
                player_id: f.player,
                input: input(),
            },
            TableAction::StartSession {
                id: f.session,
                name: "Recovery session".into(),
                participants: vec![SessionParticipant {
                    player_id: f.player,
                    character_id: Some(f.character),
                    attendance: AttendanceStatus::Present,
                }],
            },
            TableAction::SetSituation {
                situation: TableSituation {
                    title: "Ridge".into(),
                    description: "A steep ridge blocks the path.".into(),
                    challenges: vec![TableChallenge {
                        id: "ridge".into(),
                        title: "Scale the ridge".into(),
                        description: "Rough handholds rise above you.".into(),
                        phrases: vec!["climb".into()],
                        kind: TestKind::Check {
                            ability: Ability::Strength,
                            skill: Some(Skill::Athletics),
                        },
                        dc: 15,
                        success: "You reach the top.".into(),
                        failure: "You remain below.".into(),
                        resolution: None,
                    }],
                },
            },
        ] {
            f.execute(action, false).await;
        }
        f
    }

    async fn meta(&self, player: bool, starting: bool) -> CommandMeta {
        let state = self.runtime.open_campaign(self.campaign).await.unwrap();
        CommandMeta {
            id: CommandId::new(),
            campaign_id: self.campaign,
            session_id: if starting {
                Some(self.session)
            } else {
                state
                    .state()
                    .table
                    .as_ref()
                    .unwrap()
                    .active_session
                    .as_ref()
                    .map(|s| s.session_id)
            },
            issuer: if player {
                CommandIssuer::Player(self.player)
            } else {
                CommandIssuer::Admin
            },
            actor: player.then_some(AgentRef::Entity(self.actor)),
            expected_event_sequence: state.state().applied_event_sequence,
        }
    }

    async fn execute(&self, action: TableAction, player: bool) {
        self.runtime
            .execute_table(
                self.meta(player, matches!(action, TableAction::StartSession { .. }))
                    .await,
                action,
            )
            .await
            .unwrap();
    }

    async fn export(&self) -> CampaignExport {
        export_campaign(&self.pool, self.campaign).await.unwrap()
    }

    async fn request(&self) -> RollRequestId {
        let state = self.runtime.open_campaign(self.campaign).await.unwrap();
        let pending = state
            .state()
            .table
            .as_ref()
            .unwrap()
            .pending
            .as_ref()
            .unwrap();
        let request_id = RollRequestId::new();
        self.execute(
            TableAction::Adjudicate {
                pending_id: pending.id,
                revision: pending.revision,
                request_id,
            },
            false,
        )
        .await;
        request_id
    }
}

async fn rejects_before_writing(export: &CampaignExport, label: &str) {
    export
        .upgraded()
        .unwrap_or_else(|error| panic!("{label} must reach semantic preflight: {error}"));
    let (pool, runtime) = runtime().await;
    assert!(
        runtime.restore_campaign(export).await.is_err(),
        "accepted {label}"
    );
    assert!(
        dmd_persistence::list_campaigns(&pool)
            .await
            .unwrap()
            .is_empty(),
        "wrote {label}"
    );
    for table in [
        "event_journal",
        "command_audit",
        "play_sessions",
        "session_observations",
    ] {
        let count: i64 = sqlx::query_scalar(&format!("SELECT COUNT(*) FROM {table}"))
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 0, "{label} partly wrote {table}");
    }
}

fn current(export: &CampaignExport) -> CampaignState {
    CampaignState::decode_json(&export.current_state.state_json).unwrap()
}
fn snapshot(export: &CampaignExport) -> SnapshotRow {
    SnapshotRow {
        campaign_id: export.campaign_id.clone(),
        event_sequence: export.current_state.applied_event_sequence,
        state_schema_version: export.current_state.schema_version,
        state_json: export.current_state.state_json.clone(),
        created_at_utc: "2026-09-24 00:00:00".into(),
    }
}

#[tokio::test]
async fn new_effect_authority_requires_real_history_and_the_original_anchor() {
    let f = Fixture::new().await;
    let export = f.export().await;
    for corruption in 0..3 {
        let mut forged = export.clone();
        let mut state = current(&forged);
        let mut effects = TacticalEffects::default();
        if corruption == 2 {
            // Validly shaped metadata is not evidence that an effect operation occurred.
            effects.last_operation = Some(EffectOperationStamp {
                command: f.meta(false, false).await,
                step: 0,
            });
        }
        state.rules.as_mut().unwrap().tactical_effects = Some(effects);
        assert!(state.validate().is_empty());
        forged.current_state.state_json = state.encode_json().unwrap();
        if corruption == 1 {
            // A matching later anchor must not authenticate its own tactical authority.
            forged.snapshots = vec![snapshot(&forged)];
        }
        rejects_before_writing(&forged, &format!("invented effect authority {corruption}")).await;
    }
    let (target, restored) = runtime().await;
    restored.restore_campaign(&export).await.unwrap();
    assert_eq!(
        restored.open_campaign(f.campaign).await.unwrap().state(),
        f.runtime.open_campaign(f.campaign).await.unwrap().state()
    );
    target.close().await;
    f.pool.close().await;
}

#[tokio::test]
async fn pending_decision_anchors_require_exact_declared_text_revision_and_origin() {
    let f = Fixture::new().await;
    f.execute(
        TableAction::Declare {
            text: "I climb the ridge".into(),
        },
        true,
    )
    .await;
    let export = f.export().await;
    let (_, resumed) = runtime().await;
    resumed.restore_campaign(&export).await.unwrap();
    assert_eq!(
        resumed.replay_rules(f.campaign).await.unwrap(),
        current(&export)
    );
    for mutation in 0..3 {
        let mut forged = export.clone();
        let mut state = current(&forged);
        let pending = state.table.as_mut().unwrap().pending.as_mut().unwrap();
        match mutation {
            0 => pending.origin.id = CommandId::new(),
            1 => pending.revision += 1,
            2 => pending.text = "An invented declaration".into(),
            _ => unreachable!(),
        }
        forged.current_state.state_json = state.encode_json().unwrap();
        forged.snapshots.push(snapshot(&forged));
        rejects_before_writing(&forged, &format!("pending mutation {mutation}")).await;
    }
    // Even a backfilled current anchor checks available declaration/audit consistency.
    let mut backfilled = export.clone();
    backfilled.snapshots = vec![snapshot(&backfilled)];
    let (_, resumed) = runtime().await;
    resumed.restore_campaign(&backfilled).await.unwrap();
    let mut forged_authority = backfilled.clone();
    let row = &mut forged_authority.event_journal[0];
    let mut event: TableEvent = serde_json::from_str(&row.payload_json).unwrap();
    event.meta.issuer = CommandIssuer::Player(f.player);
    row.payload_json = serde_json::to_string(&event).unwrap();
    let audit = forged_authority
        .command_audit
        .iter_mut()
        .find(|audit| audit.id == row.command_id)
        .unwrap();
    audit.issuer_kind = "player".into();
    audit.issuer_player_id = Some(f.player.0.to_string());
    rejects_before_writing(&forged_authority, "pre-anchor forged host authority").await;
    let mut state = current(&backfilled);
    state.table.as_mut().unwrap().pending.as_mut().unwrap().text =
        "Another invented declaration".into();
    backfilled.current_state.state_json = state.encode_json().unwrap();
    backfilled.snapshots[0].state_json = backfilled.current_state.state_json.clone();
    rejects_before_writing(&backfilled, "backfilled pending text").await;
}

#[tokio::test]
async fn composed_roll_outcomes_audits_and_closed_session_history_restore_exactly() {
    let f = Fixture::new().await;
    let query_meta = f.meta(true, false).await;
    f.runtime
        .submit_table_text(query_meta, "What can I see?")
        .await
        .unwrap();
    f.execute(
        TableAction::Declare {
            text: "I climb the ridge".into(),
        },
        true,
    )
    .await;
    let request = f.request().await;
    let pending_export = f.export().await;
    let (_, pending_resume) = runtime().await;
    pending_resume
        .restore_campaign(&pending_export)
        .await
        .unwrap();
    assert_eq!(
        pending_resume.replay_rules(f.campaign).await.unwrap(),
        current(&pending_export)
    );
    f.execute(
        TableAction::SubmitPhysical {
            request_id: request,
            faces: vec![15],
        },
        true,
    )
    .await;
    f.execute(TableAction::EndSession, false).await;
    let export = f.export().await;
    let (_, resumed) = runtime().await;
    resumed.restore_campaign(&export).await.unwrap();
    assert_eq!(
        resumed.replay_rules(f.campaign).await.unwrap(),
        current(&export)
    );
    assert_eq!(
        resumed
            .table_view(f.campaign, TableViewer::Host)
            .await
            .unwrap(),
        f.runtime
            .table_view(f.campaign, TableViewer::Host)
            .await
            .unwrap()
    );
    let mut forged_anchor = export.clone();
    let mut intermediate = current(&pending_export);
    intermediate.table.as_mut().unwrap().contract.tone = "An unrecorded table agreement".into();
    let mut historical = snapshot(&pending_export);
    historical.state_json = intermediate.encode_json().unwrap();
    forged_anchor.snapshots.push(historical);
    rejects_before_writing(&forged_anchor, "altered intermediate table snapshot").await;
    let roll_index = export
        .event_journal
        .iter()
        .position(|row| {
            matches!(
                serde_json::from_str::<TableEvent>(&row.payload_json)
                    .unwrap()
                    .action,
                TableAction::SubmitPhysical { .. }
            )
        })
        .unwrap();
    for mutation in 0..12 {
        let mut forged = export.clone();
        let row = &mut forged.event_journal[roll_index];
        let mut event: TableEvent = serde_json::from_str(&row.payload_json).unwrap();
        let audit = forged
            .command_audit
            .iter_mut()
            .find(|audit| audit.id == row.command_id)
            .unwrap();
        match mutation {
            0 => event.rules_event.as_mut().unwrap().meta.issuer = CommandIssuer::Admin,
            1 => event.outcome.mechanics = Some(RulesOutcome::Healed { regained: 99 }),
            2 => {
                event.outcome.message = "An invented success".into();
                audit.resolution_explanation = serde_json::to_string(&event.outcome).unwrap();
            }
            3 => {
                audit.resolution_explanation = serde_json::to_string(&TableOutcome {
                    message: "Altered audit".into(),
                    mechanics: event.outcome.mechanics.clone(),
                })
                .unwrap()
            }
            4 => {
                if let RulesAction::SubmitRoll { result } =
                    &mut event.rules_event.as_mut().unwrap().action
                {
                    result.source = RollSource::Digital;
                }
            }
            5 => row.event_schema_version = 2,
            6 => forged.play_sessions[0].display_name = "Invented closed session".into(),
            7 => forged.play_sessions[0].ended_at_world = Some(50),
            8 => forged.play_session_participants[0].attendance = "absent".into(),
            9 => {
                let body = &mut forged.observations[0].record;
                let mut value: serde_json::Value =
                    serde_json::from_str(&body.payload_json).unwrap();
                value["meta"]["expected_event_sequence"] = serde_json::json!(0);
                body.payload_json = value.to_string();
            }
            10 => forged.observations[0].record.kind = "table.unsupported".into(),
            11 => {
                event.rules_event = None;
                event.outcome.mechanics = None;
                audit.resolution_explanation = serde_json::to_string(&event.outcome).unwrap();
            }
            _ => unreachable!(),
        }
        forged.event_journal[roll_index].payload_json = serde_json::to_string(&event).unwrap();
        rejects_before_writing(&forged, &format!("completed mutation {mutation}")).await;
    }
    // The available pre-anchor envelope must still reject unrelated nested authority/actions.
    let mut backfilled = export.clone();
    backfilled.snapshots = vec![snapshot(&backfilled)];
    let (_, resumed) = runtime().await;
    resumed.restore_campaign(&backfilled).await.unwrap();
    let row = &mut backfilled.event_journal[roll_index];
    let mut event: TableEvent = serde_json::from_str(&row.payload_json).unwrap();
    event.rules_event.as_mut().unwrap().action = RulesAction::EndConcentration { actor: f.actor };
    row.payload_json = serde_json::to_string(&event).unwrap();
    rejects_before_writing(&backfilled, "pre-anchor unrelated nested action").await;
}

#[tokio::test]
async fn a_raw_rules_event_cannot_bypass_an_active_table() {
    let f = Fixture::new().await;
    let before = f.export().await;
    let state = current(&before);
    let pack =
        dmd_rules::RulesPack::from_json(include_str!("../../../content/srd-5.2.1/kernel.json"))
            .unwrap();
    let meta = f.meta(false, false).await;
    let action = RulesAction::SetProne {
        target: f.actor,
        prone: true,
        ruling: Ruling {
            basis: RulingBasis::GmAdjudication,
            reason: "Unsupported direct table bypass".into(),
        },
    };
    let transition = dmd_rules::resolve(&state, &meta, &action, &pack).unwrap();
    let mut next = transition.next_state;
    next.applied_event_sequence += 1;
    let event = PendingEvent {
        id: EventId::new(),
        occurred_at: next.clock.now,
        source: EventSource::RuleResolution,
        actor: meta.actor,
        caused_by_event_ids: vec![],
        payload: transition.event,
    }
    .encode(RULES_EVENT_KIND, RULES_EVENT_VERSION)
    .unwrap();
    dmd_persistence::commit_campaign_transition(
        &f.pool,
        &meta,
        &SerializedRecord::encode("rules.action", 1, &action).unwrap(),
        &next,
        &[event],
        &serde_json::to_string(&transition.outcome).unwrap(),
    )
    .await
    .unwrap();
    let export = f.export().await;
    assert!(f.runtime.replay_rules(f.campaign).await.is_err());
    rejects_before_writing(&export, "direct mechanics bypass").await;
}
