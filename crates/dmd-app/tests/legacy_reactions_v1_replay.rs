//! Genuine pre-response saves: never regenerate these histories with a new executor.
use std::path::{Path, PathBuf};

use dmd_app::*;
use dmd_domain::*;
use dmd_persistence::{CampaignExport, export_campaign, open_sqlite_path};
use dmd_rules::tactical::TacticalAction;

#[path = "support/sqlite_test_cleanup.rs"]
mod sqlite_test_cleanup;

fn runtime(pool: sqlx::SqlitePool) -> CampaignRuntime {
    CampaignRuntime::from_content_root(
        pool,
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../content"),
    )
}

fn flow(state: &CampaignState) -> &TacticalFlow {
    state.encounter.as_ref().unwrap().flow.as_ref().unwrap()
}

struct Fixture {
    directory: PathBuf,
    pool: sqlx::SqlitePool,
    app: CampaignRuntime,
    original: CampaignExport,
    campaign: CampaignId,
    session: Option<PlaySessionId>,
}

impl Fixture {
    async fn restore(json: &str, events: usize, projections: usize, bindings: usize) -> Self {
        let original = CampaignExport::from_json(json).unwrap();
        assert_eq!(original.event_journal.len(), events);
        assert_eq!(original.table_projection_history.len(), projections);
        assert_eq!(original.table_transport_bindings.len(), bindings);
        let event: TableEvent =
            serde_json::from_str(&original.event_journal.last().unwrap().payload_json).unwrap();
        let campaign = event.meta.campaign_id;
        let directory = std::env::temp_dir().join(format!("dmd-v2-replay-{}", CampaignId::new().0));
        std::fs::create_dir_all(&directory).unwrap();
        let pool = open_sqlite_path(&directory.join("campaign.sqlite"))
            .await
            .unwrap();
        let app = runtime(pool.clone());
        Box::pin(app.restore_campaign(&original)).await.unwrap();
        let mut fixture = Self {
            directory,
            pool,
            app,
            original,
            campaign,
            session: event.meta.session_id,
        };
        fixture.assert_export(&fixture.original).await;
        fixture.reopen().await;
        assert_eq!(flow(&fixture.state().await).version, 2);
        // Every old envelope and response remains unchanged, including requests
        // whose original revision and tactical phase are no longer current.
        for binding in &fixture.original.table_transport_bindings {
            let request: TableTransportRequest =
                serde_json::from_str(&binding.request_json).unwrap();
            assert_eq!(
                serde_json::to_string(&request).unwrap(),
                binding.request_json
            );
            let response = Box::pin(fixture.app.submit_presented_table(request))
                .await
                .unwrap();
            assert_eq!(
                serde_json::to_string(&response).unwrap(),
                binding.response_json
            );
            fixture.assert_export(&fixture.original).await;
        }
        fixture
    }

    async fn state(&self) -> CampaignState {
        self.app
            .open_campaign(self.campaign)
            .await
            .unwrap()
            .state()
            .clone()
    }

    async fn assert_export(&self, expected: &CampaignExport) {
        let mut actual = export_campaign(&self.pool, self.campaign).await.unwrap();
        actual.exported_at_utc.clone_from(&expected.exported_at_utc);
        assert_eq!(&actual, expected);
    }

    async fn reopen(&mut self) {
        self.pool.close().await;
        self.pool = open_sqlite_path(&self.directory.join("campaign.sqlite"))
            .await
            .unwrap();
        self.app = runtime(self.pool.clone());
    }

    async fn request(
        &self,
        channel: TableTransportChannel,
        action: TacticalAction,
    ) -> TableTransportRequest {
        let viewer = match channel {
            TableTransportChannel::Host => TableViewer::Host,
            TableTransportChannel::Player { player_id, .. }
            | TableTransportChannel::SourceCreature { player_id, .. } => {
                TableViewer::Player(player_id)
            }
        };
        let view = self
            .app
            .presented_table_view(self.campaign, viewer)
            .await
            .unwrap();
        TableTransportRequest {
            // These genuinely saved tables have not activated a newer transport.
            version: 1,
            command_id: CommandId::new(),
            campaign_id: self.campaign,
            session_id: self.session,
            channel,
            revision: view.revision,
            input: TableTransportInput::Action(Box::new(TableAction::Tactical { action })),
        }
    }

    async fn reject(&self, request: TableTransportRequest) {
        let before = export_campaign(&self.pool, self.campaign).await.unwrap();
        assert!(
            Box::pin(self.app.submit_presented_table(request))
                .await
                .is_err()
        );
        self.assert_export(&before).await;
    }

    async fn accept_cold(&mut self, request: TableTransportRequest) {
        let before = export_campaign(&self.pool, self.campaign).await.unwrap();
        let mirror_path = self
            .directory
            .join(format!("mirror-{}.sqlite", request.command_id.0));
        let mirror_pool = open_sqlite_path(&mirror_path).await.unwrap();
        let mirror = runtime(mirror_pool.clone());
        Box::pin(mirror.restore_campaign(&before)).await.unwrap();
        self.reopen().await;
        let response = Box::pin(self.app.submit_presented_table(request.clone()))
            .await
            .unwrap();
        Box::pin(mirror.submit_presented_table(request.clone()))
            .await
            .unwrap();
        assert_eq!(
            mirror.open_campaign(self.campaign).await.unwrap().state(),
            &self.state().await
        );
        mirror_pool.close().await;
        drop(mirror);
        drop(mirror_pool);
        let accepted = export_campaign(&self.pool, self.campaign).await.unwrap();
        assert!(
            accepted
                .event_journal
                .starts_with(&self.original.event_journal)
        );
        assert!(
            accepted
                .table_projection_history
                .starts_with(&self.original.table_projection_history)
        );
        self.reopen().await;
        assert_eq!(
            Box::pin(self.app.submit_presented_table(request))
                .await
                .unwrap(),
            response
        );
        self.assert_export(&accepted).await;
        // Final portable recovery must authenticate the new continuation too.
        let restored_pool = open_sqlite_path(
            &self
                .directory
                .join(format!("restored-{}.sqlite", CommandId::new().0)),
        )
        .await
        .unwrap();
        let restored = runtime(restored_pool.clone());
        Box::pin(restored.restore_campaign(&accepted))
            .await
            .unwrap();
        assert_eq!(
            restored.open_campaign(self.campaign).await.unwrap().state(),
            &self.state().await
        );
        let mut restored_export = export_campaign(&restored_pool, self.campaign)
            .await
            .unwrap();
        restored_export
            .exported_at_utc
            .clone_from(&accepted.exported_at_utc);
        assert_eq!(restored_export, accepted);
        restored_pool.close().await;
        drop(restored);
        drop(restored_pool);
    }

    async fn close(self) {
        self.pool.close().await;
        let directory = self.directory.clone();
        drop(self);
        sqlite_test_cleanup::remove_closed_file(&directory.join("campaign.sqlite"))
            .await
            .unwrap();
        sqlite_test_cleanup::remove_closed_directory(&directory)
            .await
            .unwrap();
    }
}

fn owner(state: &CampaignState, actor: EntityId) -> (PlayerId, TableTransportChannel) {
    let character = state
        .characters
        .values()
        .find(|pc| pc.entity_id == actor)
        .unwrap();
    let player = character.controlling_player_id.unwrap();
    (
        player,
        TableTransportChannel::Player {
            player_id: player,
            character_id: character.id,
        },
    )
}

fn foreign(state: &CampaignState, actor: EntityId) -> TableTransportChannel {
    let character = state
        .characters
        .values()
        .find(|pc| pc.entity_id != actor)
        .unwrap();
    TableTransportChannel::Player {
        player_id: character.controlling_player_id.unwrap(),
        character_id: character.id,
    }
}

#[tokio::test]
async fn genuine_v2_attack_roll_finishes_without_new_response_windows() {
    let mut f = Box::pin(Fixture::restore(
        include_str!("fixtures/reactions-v1-attackroll-fb83.json"),
        13,
        8,
        1,
    ))
    .await;
    let before = f.state().await;
    let attack = flow(&before)
        .resolution
        .as_ref()
        .unwrap()
        .attack
        .as_ref()
        .unwrap();
    assert_eq!(attack.stage, TacticalAttackStage::AttackRoll);
    assert!(
        flow(&before)
            .resolution
            .as_ref()
            .unwrap()
            .work_trace
            .is_some()
    );
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
    let (actor, target) = (attack.actor, attack.target);
    assert_eq!(before.rules.as_ref().unwrap().entities[&target].hp, 10);
    let (player, channel) = owner(&before, actor);
    let view = f
        .app
        .presented_table_view(f.campaign, TableViewer::Player(player))
        .await
        .unwrap();
    let roll = view.roll.unwrap();
    let action = TacticalAction::SubmitRoll {
        result: RollResult {
            request_id: roll.id,
            source: RollSource::Physical,
            dice: vec![DieResult {
                sides: 20,
                value: 20,
            }],
        },
    };
    let unauthorized = f.request(foreign(&before, actor), action.clone()).await;
    Box::pin(f.reject(unauthorized)).await;
    let accepted = f.request(channel, action).await;
    Box::pin(f.accept_cold(accepted)).await;
    let after = f.state().await;
    assert_eq!(flow(&after).version, 2);
    assert!(flow(&after).resolution.is_none());
    let rules = after.rules.as_ref().unwrap();
    assert!(rules.pending.is_none());
    assert_eq!(rules.entities[&target].hp, 6);
    assert_eq!(
        rules.rolls.len(),
        before.rules.as_ref().unwrap().rolls.len() + 1
    );
    assert_eq!(
        rules.rolls.last().unwrap().result.dice,
        vec![DieResult {
            sides: 20,
            value: 20
        }]
    );
    f.close().await;
}

#[tokio::test]
async fn genuine_v2_knockout_choice_finishes_without_replacing_accepted_dice() {
    let mut f = Box::pin(Fixture::restore(
        include_str!("fixtures/reactions-v1-knockoutchoice-fb83.json"),
        22,
        17,
        8,
    ))
    .await;
    let before = f.state().await;
    let attack = flow(&before)
        .resolution
        .as_ref()
        .unwrap()
        .attack
        .as_ref()
        .unwrap();
    assert_eq!(attack.stage, TacticalAttackStage::KnockoutChoice);
    let (actor, target) = (attack.actor, attack.target);
    assert_eq!(before.rules.as_ref().unwrap().entities[&target].hp, 2);
    let (_, channel) = owner(&before, actor);
    let action = TacticalAction::ChooseAttackKnockout {
        choice: KnockoutChoice::KnockOut,
    };
    let unauthorized = f.request(foreign(&before, actor), action.clone()).await;
    Box::pin(f.reject(unauthorized)).await;
    let accepted = f.request(channel, action).await;
    Box::pin(f.accept_cold(accepted)).await;
    let after = f.state().await;
    assert_eq!(flow(&after).version, 2);
    assert!(flow(&after).resolution.is_none());
    let rules = after.rules.as_ref().unwrap();
    assert!(rules.pending.is_none());
    assert_eq!(rules.entities[&target].hp, 1);
    assert!(
        rules.tactical_recovery.as_ref().unwrap()[&target]
            .knockout
            .is_some()
    );
    assert_eq!(rules.rolls, before.rules.as_ref().unwrap().rolls);
    f.close().await;
}

#[tokio::test]
async fn genuine_v2_paid_ready_is_owned_even_without_a_pending_resolution() {
    let mut f = Box::pin(Fixture::restore(
        include_str!("fixtures/reactions-v1-paid-ready-fb83.json"),
        13,
        8,
        1,
    ))
    .await;
    let before = f.state().await;
    let old_flow = flow(&before);
    assert!(old_flow.resolution.is_none());
    assert!(before.rules.as_ref().unwrap().pending.is_none());
    assert_eq!(old_flow.ready.len(), 1);
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
    let actor = old_flow.ready[0].actor;
    let (_, channel) = owner(&before, actor);
    let foreign_channel = foreign(&before, actor);
    let TableTransportChannel::Player {
        player_id: unrelated,
        ..
    } = foreign_channel
    else {
        panic!("genuine fixture requires another player");
    };
    let unrelated_before = f
        .app
        .presented_table_view(f.campaign, TableViewer::Player(unrelated))
        .await
        .unwrap();
    let action = TacticalAction::AbandonReady { actor };
    for unauthorized in [TableTransportChannel::Host, foreign_channel] {
        let request = f.request(unauthorized, action.clone()).await;
        Box::pin(f.reject(request)).await;
    }
    let accepted = f.request(channel, action).await;
    Box::pin(f.accept_cold(accepted)).await;
    let after = f.state().await;
    assert_eq!(after.rules, before.rules);
    assert_eq!(after.clock, before.clock);
    let mut expected_encounter = before.encounter.clone().unwrap();
    expected_encounter.flow.as_mut().unwrap().ready.clear();
    assert_eq!(after.encounter, Some(expected_encounter));
    assert_eq!(
        f.app
            .presented_table_view(f.campaign, TableViewer::Player(unrelated))
            .await
            .unwrap(),
        unrelated_before
    );
    let retained = export_campaign(&f.pool, f.campaign).await.unwrap();
    let original_request: TableTransportRequest =
        serde_json::from_str(&f.original.table_transport_bindings[0].request_json).unwrap();
    let old_response = Box::pin(f.app.submit_presented_table(original_request))
        .await
        .unwrap();
    assert_eq!(
        serde_json::to_string(&old_response).unwrap(),
        f.original.table_transport_bindings[0].response_json
    );
    f.assert_export(&retained).await;
    f.close().await;
}

#[tokio::test]
async fn genuine_original_upgrade_keeps_its_one_to_two_result_and_accepted_responses() {
    let f = Box::pin(Fixture::restore(
        include_str!("fixtures/reactions-v1-upgrade-100c7da.json"),
        16,
        11,
        2,
    ))
    .await;
    let original =
        CampaignExport::from_json(include_str!("fixtures/legacy-savage-f960.json")).unwrap();
    assert!(
        f.original
            .event_journal
            .starts_with(&original.event_journal)
    );
    assert!(
        f.original
            .table_projection_history
            .starts_with(&original.table_projection_history)
    );
    assert_eq!(f.original.snapshots, original.snapshots);
    for audit in &original.command_audit {
        assert!(f.original.command_audit.contains(audit));
    }
    let event: TableEvent =
        serde_json::from_str(&f.original.event_journal.last().unwrap().payload_json).unwrap();
    assert_eq!(
        event.action,
        TableAction::Tactical {
            action: TacticalAction::UpgradeExecution,
        }
    );
    assert_eq!(
        event.tactical_event.unwrap().action,
        TacticalAction::UpgradeExecution
    );
    let state = f.state().await;
    assert_eq!(state.applied_event_sequence, 16);
    assert_eq!(flow(&state).version, 2);
    assert!(flow(&state).resolution.is_none());
    assert!(flow(&state).ready.is_empty());
    let rules = state.rules.as_ref().unwrap();
    assert!(rules.pending.is_none());
    assert!(rules.timing.as_ref().unwrap().action_spent);
    let record = rules.rolls.last().unwrap();
    let savage = record.savage_attacker.as_ref().unwrap();
    let faces = |values: &[u16]| {
        values
            .iter()
            .map(|&value| DieResult { sides: 4, value })
            .collect::<Vec<_>>()
    };
    assert_eq!(savage.first.dice, faces(&[1, 2]));
    assert_eq!(savage.second.dice, faces(&[4, 4]));
    assert_eq!(savage.chosen, DamageRollChoice::First);
    assert_eq!(record.result.dice, savage.first.dice);

    // The original journaled unit action permanently means Legacy1-to2. Once
    // already at2 it must not become an implicit upgrade to any newer executor.
    let fresh = f
        .request(
            TableTransportChannel::Host,
            TacticalAction::UpgradeExecution,
        )
        .await;
    Box::pin(f.reject(fresh)).await;
    f.close().await;
}

async fn aftermath_request(
    f: &Fixture,
    session: PlaySessionId,
    channel: TableTransportChannel,
    action: TableAction,
) -> TableTransportRequest {
    let viewer = match channel {
        TableTransportChannel::Host => TableViewer::Host,
        TableTransportChannel::Player { player_id, .. }
        | TableTransportChannel::SourceCreature { player_id, .. } => TableViewer::Player(player_id),
    };
    let view = f
        .app
        .presented_table_view(f.campaign, viewer)
        .await
        .unwrap();
    assert!(view.source_control.is_some());
    TableTransportRequest {
        version: TABLE_SOURCE_TRANSPORT_VERSION,
        command_id: CommandId::new(),
        campaign_id: f.campaign,
        session_id: Some(session),
        revision: view.revision,
        channel,
        input: TableTransportInput::Action(Box::new(action)),
    }
}

#[tokio::test]
async fn genuine_closed_v2_aftermath_upgrades_and_finishes_without_inventing_a_session() {
    let json = include_str!("fixtures/reactions-v1-aftermath-be544.json");
    let mut f = Box::pin(Fixture::restore(json, 15, 10, 9)).await;
    let original = Box::new(f.state().await);
    assert!(original.table.as_ref().unwrap().active_session.is_none());
    let mage = flow(&original).combatants[0].actor;
    let CreatureController::Player(player) = original
        .rules
        .as_ref()
        .unwrap()
        .tactical_creatures
        .as_ref()
        .unwrap()
        .runtime(mage)
        .unwrap()
        .controller
    else {
        panic!("genuine fixture must retain its actual source controller")
    };
    let upgrade = TacticalAction::UpgradeExecutionTo {
        execution: TacticalExecutionVersion::EncounterReleaseV1,
    };
    let mut request = aftermath_request(
        &f,
        PlaySessionId::new(),
        TableTransportChannel::Host,
        TableAction::Tactical {
            action: upgrade.clone(),
        },
    )
    .await;
    // A made-up or formerly closed session binding cannot authorize the exception.
    Box::pin(f.reject(request.clone())).await;
    request.session_id = None;
    let mut player_request = request.clone();
    player_request.command_id = CommandId::new();
    player_request.channel = TableTransportChannel::SourceCreature {
        player_id: player,
        actor: mage,
    };
    player_request.revision = f
        .app
        .presented_table_view(f.campaign, TableViewer::Player(player))
        .await
        .unwrap()
        .revision;
    Box::pin(f.reject(player_request)).await;
    for action in [
        TacticalAction::EndTurn,
        TacticalAction::FinishEncounter,
        TacticalAction::UpgradeExecution,
        TacticalAction::UpgradeExecutionTo {
            execution: TacticalExecutionVersion::ShieldMissileV1,
        },
    ] {
        let mut wrong = request.clone();
        wrong.command_id = CommandId::new();
        wrong.input = TableTransportInput::Action(Box::new(TableAction::Tactical { action }));
        Box::pin(f.reject(wrong)).await;
    }
    Box::pin(f.accept_cold(request.clone())).await;
    let upgraded = Box::new(f.state().await);
    assert_eq!(flow(&upgraded).version, 5);
    let mut compared = upgraded.clone();
    compared
        .encounter
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .version = 2;
    compared.applied_event_sequence = original.applied_event_sequence;
    assert_eq!(
        compared, original,
        "closed upgrade changes only explicit execution and sequence"
    );
    let mut finish = aftermath_request(
        &f,
        PlaySessionId::new(),
        TableTransportChannel::Host,
        TableAction::Tactical {
            action: TacticalAction::FinishEncounter,
        },
    )
    .await;
    finish.session_id = None;
    Box::pin(f.accept_cold(finish.clone())).await;
    let finished = Box::new(f.state().await);
    assert_eq!(flow(&finished).phase, TacticalPhase::Finished);
    assert!(finished.table.as_ref().unwrap().active_session.is_none());
    assert_eq!(finished.clock, original.clock);
    assert_eq!(finished.items, original.items);
    let current = finished.rules.as_ref().unwrap();
    let old = original.rules.as_ref().unwrap();
    assert_eq!(current.entities, old.entities);
    assert_eq!(current.tactical_inventory, old.tactical_inventory);
    assert_eq!(
        current.tactical_effects.as_ref().unwrap().effects,
        old.tactical_effects.as_ref().unwrap().effects
    );
    assert_eq!(
        current
            .tactical_creatures
            .as_ref()
            .unwrap()
            .runtime(mage)
            .unwrap()
            .limited_uses,
        old.tactical_creatures
            .as_ref()
            .unwrap()
            .runtime(mage)
            .unwrap()
            .limited_uses
    );
    let saved = export_campaign(&f.pool, f.campaign).await.unwrap();
    assert_eq!(saved.play_sessions, f.original.play_sessions);
    assert_eq!(
        saved.play_session_participants,
        f.original.play_session_participants
    );
    // Original accepted source requests retain their literal response bytes even
    // after an unrelated administrative session-free boundary has been accepted.
    for binding in &f.original.table_transport_bindings {
        let original_request: TableTransportRequest =
            serde_json::from_str(&binding.request_json).unwrap();
        assert_eq!(
            serde_json::to_string(
                &Box::pin(f.app.submit_presented_table(original_request))
                    .await
                    .unwrap()
            )
            .unwrap(),
            binding.response_json
        );
    }
    f.assert_export(&saved).await;
    let mut changed = request;
    changed.input = TableTransportInput::Action(Box::new(TableAction::Tactical {
        action: TacticalAction::FinishEncounter,
    }));
    Box::pin(f.reject(changed)).await;
    f.close().await;
}

#[tokio::test]
async fn genuine_v2_source_aftermath_resumes_and_upgrades_without_retiming_armor() {
    let json = include_str!("fixtures/reactions-v1-aftermath-be544.json");
    let mut f = Box::pin(Fixture::restore(json, 15, 10, 9)).await;
    assert_eq!(f.original.to_json().unwrap(), json);
    let original = Box::new(f.state().await);
    let resumed = Box::pin(f.app.resume_campaign(f.campaign)).await.unwrap();
    assert_eq!(resumed.state(), original.as_ref());
    drop(resumed);
    assert!(original.table.as_ref().unwrap().active_session.is_none());
    assert_eq!(flow(&original).combatants.len(), 1);
    let mage = flow(&original).combatants[0].actor;
    let source = original
        .rules
        .as_ref()
        .unwrap()
        .tactical_creatures
        .as_ref()
        .unwrap()
        .runtime(mage)
        .unwrap();
    let CreatureController::Player(player) = source.controller else {
        panic!("the original source-only capture must retain its real player owner");
    };
    let limited_uses = source.limited_uses.clone();
    let armor = original
        .rules
        .as_ref()
        .unwrap()
        .tactical_effects
        .as_ref()
        .unwrap()
        .effects
        .iter()
        .find(|effect| !effect.defenses.is_empty())
        .unwrap()
        .clone();
    let other_pc = original
        .characters
        .values()
        .find(|pc| pc.controlling_player_id.is_some_and(|id| id != player))
        .unwrap();
    let other_player = other_pc.controlling_player_id.unwrap();
    let session = PlaySessionId::new();
    let absent = Box::pin(aftermath_request(
        &f,
        session,
        TableTransportChannel::Host,
        TableAction::StartSession {
            id: session,
            name: "Absent retained source owner".into(),
            participants: vec![
                SessionParticipant {
                    player_id: player,
                    character_id: None,
                    attendance: AttendanceStatus::Absent,
                },
                SessionParticipant {
                    player_id: other_player,
                    character_id: Some(other_pc.id),
                    attendance: AttendanceStatus::Present,
                },
            ],
        },
    ))
    .await;
    Box::pin(f.reject(absent)).await;
    let start = Box::pin(aftermath_request(
        &f,
        session,
        TableTransportChannel::Host,
        TableAction::StartSession {
            id: session,
            name: "Historical Mage aftermath".into(),
            participants: vec![SessionParticipant {
                player_id: player,
                character_id: None,
                attendance: AttendanceStatus::Present,
            }],
        },
    ))
    .await;
    Box::pin(f.accept_cold(start)).await;
    let started = Box::new(f.state().await);
    let mut compared = started.clone();
    compared.table.as_mut().unwrap().active_session = None;
    compared.applied_event_sequence = original.applied_event_sequence;
    assert_eq!(
        compared, original,
        "session resume changes only its binding and sequence"
    );
    drop(compared);
    let source_channel = TableTransportChannel::SourceCreature {
        player_id: player,
        actor: mage,
    };
    let premature = Box::pin(aftermath_request(
        &f,
        session,
        source_channel.clone(),
        TableAction::Tactical {
            action: TacticalAction::EndTurn,
        },
    ))
    .await;
    Box::pin(f.reject(premature)).await;
    let upgrade = Box::pin(aftermath_request(
        &f,
        session,
        TableTransportChannel::Host,
        TableAction::Tactical {
            action: TacticalAction::UpgradeExecutionTo {
                execution: TacticalExecutionVersion::EncounterReleaseV1,
            },
        },
    ))
    .await;
    Box::pin(f.accept_cold(upgrade)).await;
    let upgraded = Box::new(f.state().await);
    assert_eq!(flow(&upgraded).version, 5);
    let mut compared = upgraded.clone();
    compared
        .encounter
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .version = 2;
    compared.applied_event_sequence = started.applied_event_sequence;
    assert_eq!(
        compared, started,
        "upgrade changes only the executor and sequence"
    );
    drop(compared);
    for channel in [
        TableTransportChannel::Host,
        TableTransportChannel::SourceCreature {
            player_id: other_player,
            actor: mage,
        },
    ] {
        let forbidden = Box::pin(aftermath_request(
            &f,
            session,
            channel,
            TableAction::Tactical {
                action: TacticalAction::EndTurn,
            },
        ))
        .await;
        Box::pin(f.reject(forbidden)).await;
    }
    let end = Box::pin(aftermath_request(
        &f,
        session,
        source_channel,
        TableAction::Tactical {
            action: TacticalAction::EndTurn,
        },
    ))
    .await;
    Box::pin(f.accept_cold(end)).await;
    let continued = Box::new(f.state().await);
    let rules = continued.rules.as_ref().unwrap();
    assert_eq!(
        rules.timing.as_ref().unwrap().turn_number,
        upgraded
            .rules
            .as_ref()
            .unwrap()
            .timing
            .as_ref()
            .unwrap()
            .turn_number
            + 1
    );
    assert_eq!(continued.clock.now, WorldInstant(upgraded.clock.now.0 + 6));
    assert_eq!(flow(&continued).aftermath, flow(&original).aftermath);
    assert!(
        rules
            .tactical_effects
            .as_ref()
            .unwrap()
            .effects
            .contains(&armor)
    );
    assert_eq!(
        rules
            .tactical_creatures
            .as_ref()
            .unwrap()
            .runtime(mage)
            .unwrap()
            .limited_uses,
        limited_uses
    );
    assert_eq!(
        rules.entities[&mage].hp,
        original.rules.as_ref().unwrap().entities[&mage].hp
    );
    assert_eq!(continued.items, original.items);
    let final_export = export_campaign(&f.pool, f.campaign).await.unwrap();
    for binding in &f.original.table_transport_bindings {
        let request: TableTransportRequest = serde_json::from_str(&binding.request_json).unwrap();
        let response = Box::pin(f.app.submit_presented_table(request))
            .await
            .unwrap();
        assert_eq!(
            serde_json::to_string(&response).unwrap(),
            binding.response_json
        );
        assert!(final_export.table_transport_bindings.contains(binding));
    }
    f.assert_export(&final_export).await;
    let mut changed: TableTransportRequest =
        serde_json::from_str(&f.original.table_transport_bindings[0].request_json).unwrap();
    changed.input = TableTransportInput::Action(Box::new(TableAction::Tactical {
        action: TacticalAction::Dodge,
    }));
    Box::pin(f.reject(changed)).await;
    f.close().await;
}
