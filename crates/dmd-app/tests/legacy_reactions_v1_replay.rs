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
            TableTransportChannel::Player { player_id, .. } => TableViewer::Player(player_id),
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
