//! Genuine flow3 captures. The unchanged generator creates these files; never
//! manufacture them from typed state or regenerate them with a newer executor.
use std::collections::HashSet;
use std::path::{Path, PathBuf};

use dmd_app::*;
use dmd_domain::*;
use dmd_persistence::{CampaignExport, export_campaign, open_sqlite_path};
use dmd_rules::tactical::TacticalAction;

#[path = "support/sqlite_test_cleanup.rs"]
mod sqlite_test_cleanup;

const MISSILE_FIRST: &str = "shield-hit-v1-missile-after-0-darts.json";
const MISSILE_PARTIAL: &str = "shield-hit-v1-missile-after-2-darts.json";
const SHIELD_SELECTED: &str = "shield-hit-v1-selected.json";
const SHIELD_DAMAGE: &str = "shield-hit-v1-post-cast-damage.json";

fn runtime(pool: sqlx::SqlitePool) -> CampaignRuntime {
    CampaignRuntime::from_content_root(
        pool,
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../content"),
    )
}

fn image(export: &CampaignExport) -> Box<CampaignState> {
    Box::new(CampaignState::decode_json(&export.current_state.state_json).unwrap())
}

fn flow(state: &CampaignState) -> &TacticalFlow {
    state.encounter.as_ref().unwrap().flow.as_ref().unwrap()
}

fn capture(name: &str) -> Box<CampaignExport> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name);
    let bytes = std::fs::read(&path).unwrap_or_else(|error| {
        panic!(
            "required genuine flow3 capture {} is missing/unreadable: {error}; import the verified generator artifact, never skip or synthesize it",
            path.display()
        )
    });
    let export = Box::new(CampaignExport::from_json(std::str::from_utf8(&bytes).unwrap()).unwrap());
    assert_eq!(
        export.to_json().unwrap().as_bytes(),
        bytes,
        "the actual generator's exported bytes must round-trip unchanged"
    );
    assert_eq!(flow(&image(&export)).version, 3);
    assert!(!export.event_journal.is_empty());
    assert!(!export.table_projection_history.is_empty());
    assert!(!export.table_transport_bindings.is_empty());
    export
}

fn tactical(action: TacticalAction) -> TableTransportInput {
    TableTransportInput::Action(Box::new(TableAction::Tactical { action }))
}

fn viewer(channel: &TableTransportChannel) -> TableViewer {
    match *channel {
        TableTransportChannel::Host => TableViewer::Host,
        TableTransportChannel::Player { player_id, .. }
        | TableTransportChannel::SourceCreature { player_id, .. } => TableViewer::Player(player_id),
    }
}

fn controller(state: &CampaignState, actor: EntityId) -> TableTransportChannel {
    if let Some(pc) = state.characters.values().find(|pc| pc.entity_id == actor) {
        return TableTransportChannel::Player {
            player_id: pc.controlling_player_id.unwrap(),
            character_id: pc.id,
        };
    }
    match state
        .rules
        .as_ref()
        .unwrap()
        .tactical_creatures
        .as_ref()
        .unwrap()
        .runtime(actor)
        .unwrap()
        .controller
    {
        CreatureController::Player(player_id) => {
            TableTransportChannel::SourceCreature { player_id, actor }
        }
        // These historical autonomous actors' secret rolls were issued and
        // submitted by Admin/Host. Keep the controller and original channel.
        CreatureController::Host | CreatureController::Autonomous => TableTransportChannel::Host,
    }
}

fn faces(view: &TablePresentedView, values: &[u16]) -> TableTransportInput {
    let roll = view.roll.as_ref().unwrap();
    assert_eq!(roll.mode, RollMode::Normal);
    let sides = roll
        .dice
        .iter()
        .flat_map(|die| std::iter::repeat_n(die.sides, usize::from(die.count)))
        .collect::<Vec<_>>();
    assert_eq!(sides.len(), values.len());
    tactical(TacticalAction::SubmitRoll {
        result: RollResult {
            request_id: roll.id,
            source: RollSource::Physical,
            dice: sides
                .into_iter()
                .zip(values)
                .map(|(sides, value)| DieResult {
                    sides,
                    value: *value,
                })
                .collect(),
        },
    })
}

struct Fixture {
    directory: PathBuf,
    pool: sqlx::SqlitePool,
    app: CampaignRuntime,
    original: Box<CampaignExport>,
    campaign: CampaignId,
    session: PlaySessionId,
}

impl Fixture {
    async fn restore(name: &str) -> Self {
        let original = capture(name);
        let (campaign, session) = {
            let state = image(&original);
            (
                state.campaign_id(),
                state
                    .table
                    .as_ref()
                    .unwrap()
                    .active_session
                    .as_ref()
                    .unwrap()
                    .session_id,
            )
        };
        let directory = std::env::temp_dir().join(format!("dmd-flow3-{}", CampaignId::new().0));
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
            session,
        };
        fixture.assert_export(&fixture.original).await;
        fixture.reopen().await;
        fixture.assert_export(&fixture.original).await;
        fixture
    }

    async fn state(&self) -> Box<CampaignState> {
        image(&export_campaign(&self.pool, self.campaign).await.unwrap())
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
        Box::pin(self.app.resume_campaign(self.campaign))
            .await
            .unwrap();
    }

    async fn view(&self, channel: &TableTransportChannel) -> TablePresentedView {
        self.app
            .presented_table_view(self.campaign, viewer(channel))
            .await
            .unwrap()
    }

    fn request(
        &self,
        channel: TableTransportChannel,
        view: &TablePresentedView,
        input: TableTransportInput,
    ) -> TableTransportRequest {
        TableTransportRequest {
            version: if view.source_control.is_some() {
                TABLE_SOURCE_TRANSPORT_VERSION
            } else {
                TABLE_TRANSPORT_VERSION
            },
            command_id: CommandId::new(),
            campaign_id: self.campaign,
            session_id: Some(self.session),
            channel,
            revision: view.revision,
            input,
        }
    }

    async fn retry_original_bindings(&self) {
        let before = export_campaign(&self.pool, self.campaign).await.unwrap();
        for binding in &self.original.table_transport_bindings {
            let request: TableTransportRequest =
                serde_json::from_str(&binding.request_json).unwrap();
            let response: TableTransportResult =
                serde_json::from_str(&binding.response_json).unwrap();
            assert_eq!(request.version, binding.version);
            assert_eq!(
                serde_json::to_string(&request).unwrap(),
                binding.request_json
            );
            assert_eq!(
                serde_json::to_string(&response).unwrap(),
                binding.response_json
            );
            assert_eq!(
                Box::pin(self.app.submit_presented_table(request))
                    .await
                    .unwrap(),
                response,
                "all original envelopes recover before current owner/revision/phase admission"
            );
        }
        // Check the whole batch against every durable row; inspection alone does
        // not need an additional campaign reopen after each exact receipt lookup.
        self.assert_export(&before).await;
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

    fn assert_prefix(&self, accepted: &CampaignExport) {
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
        for binding in &self.original.table_transport_bindings {
            assert!(accepted.table_transport_bindings.contains(binding));
        }
        for audit in &self.original.command_audit {
            assert!(accepted.command_audit.contains(audit));
        }
    }

    async fn accept_cold(&mut self, request: TableTransportRequest) {
        let before = export_campaign(&self.pool, self.campaign).await.unwrap();
        let mirror_path = self
            .directory
            .join(format!("mirror-{}.sqlite", request.command_id.0));
        let mirror_pool = open_sqlite_path(&mirror_path).await.unwrap();
        let mirror = runtime(mirror_pool.clone());
        Box::pin(mirror.restore_campaign(&before)).await.unwrap();
        mirror_pool.close().await;
        drop(mirror);
        drop(mirror_pool);
        let mirror_pool = open_sqlite_path(&mirror_path).await.unwrap();
        let mirror = runtime(mirror_pool.clone());
        Box::pin(mirror.resume_campaign(self.campaign))
            .await
            .unwrap();
        self.reopen().await;
        let response = Box::pin(self.app.submit_presented_table(request.clone()))
            .await
            .unwrap();
        let independent = Box::pin(mirror.submit_presented_table(request.clone()))
            .await
            .unwrap();
        match (&response, &independent) {
            (TableTransportResult::Accepted(left), TableTransportResult::Accepted(right)) => {
                assert_eq!(left.command_id, right.command_id);
                assert_eq!(left.outcome, right.outcome);
            }
            _ => panic!("both actual continuation commands must be accepted"),
        }
        let accepted = export_campaign(&self.pool, self.campaign).await.unwrap();
        let mirrored = export_campaign(&mirror_pool, self.campaign).await.unwrap();
        assert_eq!(image(&accepted), image(&mirrored));
        self.assert_prefix(&accepted);
        // Independent histories may issue different fresh opaque revisions. Each
        // must retry its own actual response, never borrow the other database's.
        assert_eq!(
            Box::pin(mirror.submit_presented_table(request.clone()))
                .await
                .unwrap(),
            independent
        );
        let mut mirror_retry = export_campaign(&mirror_pool, self.campaign).await.unwrap();
        mirror_retry
            .exported_at_utc
            .clone_from(&mirrored.exported_at_utc);
        assert_eq!(mirror_retry, mirrored);
        mirror_pool.close().await;
        drop(mirror);
        drop(mirror_pool);
        self.reopen().await;
        assert_eq!(
            Box::pin(self.app.submit_presented_table(request.clone()))
                .await
                .unwrap(),
            response
        );
        let mut changed = request;
        changed.input = tactical(TacticalAction::Dodge);
        assert!(
            Box::pin(self.app.submit_presented_table(changed))
                .await
                .is_err()
        );
        self.assert_export(&accepted).await;
    }

    async fn finish(self) {
        self.retry_original_bindings().await;
        let accepted = export_campaign(&self.pool, self.campaign).await.unwrap();
        self.assert_prefix(&accepted);
        let restored_pool = open_sqlite_path(&self.directory.join("completed-restore.sqlite"))
            .await
            .unwrap();
        let restored = runtime(restored_pool.clone());
        Box::pin(restored.restore_campaign(&accepted))
            .await
            .unwrap();
        let mut exported = export_campaign(&restored_pool, self.campaign)
            .await
            .unwrap();
        exported
            .exported_at_utc
            .clone_from(&accepted.exported_at_utc);
        assert_eq!(exported, accepted);
        restored_pool.close().await;
        drop(restored);
        drop(restored_pool);
        self.close().await;
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

#[tokio::test]
async fn genuine_flow3_capture_baseline_roundtrips_and_retries_original_bindings() {
    for (name, transport) in [
        (MISSILE_FIRST, TABLE_TRANSPORT_VERSION),
        (MISSILE_PARTIAL, TABLE_TRANSPORT_VERSION),
        (SHIELD_SELECTED, TABLE_SOURCE_TRANSPORT_VERSION),
        (SHIELD_DAMAGE, TABLE_SOURCE_TRANSPORT_VERSION),
    ] {
        let f = Box::pin(Fixture::restore(name)).await;
        let state = f.state().await;
        assert_eq!(
            state.table.as_ref().unwrap().source_actor_access.is_some(),
            transport == TABLE_SOURCE_TRANSPORT_VERSION
        );
        assert_eq!(
            f.original
                .table_transport_bindings
                .iter()
                .map(|binding| binding.version)
                .max(),
            Some(transport)
        );
        Box::pin(f.retry_original_bindings()).await;
        let mut changed: TableTransportRequest =
            serde_json::from_str(&f.original.table_transport_bindings[0].request_json).unwrap();
        changed.input = tactical(TacticalAction::Dodge);
        Box::pin(f.reject(changed)).await;
        f.assert_export(&f.original).await;
        f.close().await;
    }
}

async fn continue_missiles(name: &str, completed: usize) {
    let mut f = Box::pin(Fixture::restore(name)).await;
    let initial = f.state().await;
    let original_cast = &flow(&initial).resolution.as_ref().unwrap().casts[0];
    let caster = original_cast.cast.plan.choice.actor;
    let target = original_cast.targets[0].actor;
    let origin = original_cast.cast.plan.origin.id;
    assert_eq!(
        original_cast.cast.plan.program.source.spell_id,
        "magic-missile"
    );
    assert_eq!(original_cast.cast.plan.program.spell_level, 4);
    assert_eq!(
        original_cast
            .cast
            .plan
            .program
            .source
            .creature_definition_id
            .as_deref(),
        Some("night-hag")
    );
    assert_eq!(original_cast.targets.len(), 6);
    assert!(
        original_cast
            .targets
            .iter()
            .all(|entry| entry.actor == target)
    );
    assert_eq!(original_cast.completed.len(), completed);
    let channel = controller(&initial, caster);
    assert_eq!(channel, TableTransportChannel::Host);
    assert_eq!(
        initial
            .rules
            .as_ref()
            .unwrap()
            .tactical_creatures
            .as_ref()
            .unwrap()
            .runtime(caster)
            .unwrap()
            .controller,
        CreatureController::Autonomous
    );
    let mut raw_ids = initial
        .rules
        .as_ref()
        .unwrap()
        .rolls
        .iter()
        .filter(|roll| {
            matches!(&roll.purpose, PendingPurpose::TacticalResolution { key, .. }
            if key.origin == origin && key.role == TacticalRollRole::SpellAmount)
        })
        .map(|roll| {
            assert_eq!(roll.result.dice, [DieResult { sides: 4, value: 1 }]);
            roll.request.id
        })
        .collect::<HashSet<_>>();
    assert_eq!(raw_ids.len(), completed);
    for dart in completed..6 {
        let before = f.state().await;
        assert_eq!(flow(&before).version, 3);
        let resolution = flow(&before).resolution.as_ref().unwrap();
        assert!(resolution.hit_review.is_none());
        assert_eq!(resolution.casts[0].cast.plan.origin.id, origin);
        assert_eq!(resolution.casts[0].completed.len(), dart);
        let rules = before.rules.as_ref().unwrap();
        assert_eq!(rules.entities[&target].hp, 19 - 2 * dart as u32);
        let pending = rules.pending.as_ref().unwrap();
        assert_eq!(pending.request.visibility, RollVisibility::Secret);
        assert_eq!(pending.issued_by.issuer, CommandIssuer::Admin);
        assert!(raw_ids.insert(pending.request.id));
        assert!(
            matches!(&pending.purpose, PendingPurpose::TacticalResolution { key, .. }
            if key.origin == origin && key.role == TacticalRollRole::SpellAmount && key.subject == target)
        );
        let view = f.view(&channel).await;
        assert!(view.source_control.is_none());
        assert!(view.tactical.as_ref().unwrap().hit.is_none());
        let roll = view.roll.as_ref().unwrap();
        assert_eq!(roll.dice, [DieSpec { count: 1, sides: 4 }]);
        assert_eq!(roll.modifier, 1);
        let request = f.request(channel.clone(), &view, faces(&view, &[1]));
        Box::pin(f.accept_cold(request)).await;
        let after = f.state().await;
        assert_eq!(
            after.rules.as_ref().unwrap().entities[&target].hp,
            17 - 2 * dart as u32
        );
        let accepted_raw = after.rules.as_ref().unwrap().rolls.last().unwrap();
        assert_eq!(accepted_raw.request.id, pending.request.id);
        assert_eq!(accepted_raw.result.dice, [DieResult { sides: 4, value: 1 }]);
    }
    let final_state = f.state().await;
    let rules = final_state.rules.as_ref().unwrap();
    assert_eq!(raw_ids.len(), 6);
    assert_eq!(flow(&final_state).version, 3);
    assert!(flow(&final_state).resolution.is_none());
    assert!(rules.pending.is_none());
    assert_eq!(rules.entities[&target].hp, 7);
    assert_eq!(rules.entities[&caster].hp, 112);
    assert!(rules.entities[&caster].resources.is_empty());
    assert!(rules.entities[&caster].prepared_spells.is_empty());
    assert!(rules.entities[&caster].spellcasting.is_none());
    assert!(
        rules
            .tactical_creatures
            .as_ref()
            .unwrap()
            .runtime(caster)
            .unwrap()
            .limited_uses
            .is_empty()
    );
    assert!(rules.timing.as_ref().unwrap().action_spent);
    assert!(!rules.timing.as_ref().unwrap().slot_spent_this_turn);
    assert!(
        rules
            .rolls
            .starts_with(&initial.rules.as_ref().unwrap().rolls)
    );
    Box::pin(f.finish()).await;
}

#[tokio::test]
async fn genuine_flow3_missile_before_first_face_finishes_sequentially_without_target_windows() {
    Box::pin(continue_missiles(MISSILE_FIRST, 0)).await;
}

#[tokio::test]
async fn genuine_flow3_missile_after_two_faces_retains_hp15_and_finishes_at_hp7() {
    Box::pin(continue_missiles(MISSILE_PARTIAL, 2)).await;
}

fn protective_uses(state: &CampaignState, actor: EntityId) -> u32 {
    state
        .rules
        .as_ref()
        .unwrap()
        .tactical_creatures
        .as_ref()
        .unwrap()
        .runtime(actor)
        .unwrap()
        .limited_uses
        .iter()
        .find(|usage| usage.feature_id == "protective-magic" && usage.spell_id.is_none())
        .unwrap()
        .spent
        .into()
}

#[tokio::test]
async fn genuine_flow3_selected_owned_shield_finishes_as_a_miss_without_replacing_dice() {
    let mut f = Box::pin(Fixture::restore(SHIELD_SELECTED)).await;
    let before = f.state().await;
    let resolution = flow(&before).resolution.as_ref().unwrap();
    let hit = resolution.hit_review.as_ref().unwrap();
    assert_eq!(hit.stage, TacticalHitReviewStage::Selected);
    assert!(hit.completed_shield.is_none());
    let actor = hit.respondent.as_ref().unwrap().actor;
    let attack = resolution.attack.as_ref().unwrap();
    assert_eq!(attack.target, actor);
    let attack_raw = before
        .rules
        .as_ref()
        .unwrap()
        .rolls
        .iter()
        .find(|roll| roll.request.id == hit.attack_roll)
        .unwrap();
    assert_eq!(
        attack_raw.result.dice,
        [DieResult {
            sides: 20,
            value: 10
        }]
    );
    assert_eq!(before.rules.as_ref().unwrap().entities[&actor].hp, 81);
    assert!(before.rules.as_ref().unwrap().pending.is_none());
    assert_eq!(protective_uses(&before, actor), 0);
    let channel = controller(&before, actor);
    assert!(matches!(
        channel,
        TableTransportChannel::SourceCreature { .. }
    ));
    let owned = f.view(&channel).await;
    let response = owned
        .tactical
        .as_ref()
        .unwrap()
        .hit
        .as_ref()
        .unwrap()
        .response
        .as_ref()
        .unwrap();
    assert!(response.selected);
    assert_eq!(response.actor, actor);
    let choice = response.shield[0].clone();
    assert_eq!(choice.spell_id, "shield");
    assert_eq!(choice.resource, SpellResourceChoice::SourceFeature);
    assert_eq!(
        choice.grant,
        SpellGrantChoice::CreatureFeature {
            feature_id: "protective-magic".into()
        }
    );
    let input = TableTransportInput::HitResponse {
        handle: response.key,
        decision: Box::new(TableHitInput::Cast {
            choice: choice.clone(),
        }),
    };
    for foreign in [
        TableTransportChannel::Host,
        controller(&before, attack.actor),
    ] {
        let view = f.view(&foreign).await;
        let request = f.request(foreign, &view, input.clone());
        Box::pin(f.reject(request)).await;
    }
    let bypass = f.request(
        channel.clone(),
        &owned,
        tactical(TacticalAction::CastHitShield {
            window: TacticalWorkKey {
                resolution: resolution.origin.id,
                occurrence: hit.work.occurrence,
            },
            choice,
        }),
    );
    Box::pin(f.reject(bypass)).await;
    let request = f.request(channel, &owned, input);
    assert_eq!(request.version, TABLE_SOURCE_TRANSPORT_VERSION);
    Box::pin(f.accept_cold(request)).await;
    let after = f.state().await;
    assert_eq!(flow(&after).version, 3);
    assert!(flow(&after).resolution.is_none());
    let rules = after.rules.as_ref().unwrap();
    assert!(rules.pending.is_none());
    assert_eq!(rules.entities[&actor].hp, 81);
    assert_eq!(rules.rolls, before.rules.as_ref().unwrap().rolls);
    assert_eq!(protective_uses(&after, actor), 1);
    assert!(
        rules
            .timing
            .as_ref()
            .unwrap()
            .reactions_spent
            .contains(&actor)
    );
    assert_eq!(
        dmd_rules::tactical_defenses::effective_armor_class(&after, actor).unwrap(),
        17
    );
    Box::pin(f.finish()).await;
}

#[tokio::test]
async fn genuine_flow3_post_shield_natural20_damage_keeps_original_cause_and_hp75() {
    let mut f = Box::pin(Fixture::restore(SHIELD_DAMAGE)).await;
    let before = f.state().await;
    let resolution = flow(&before).resolution.as_ref().unwrap();
    let attack = resolution.attack.as_ref().unwrap();
    let hit = resolution.hit_review.as_ref().unwrap();
    assert_eq!(hit.stage, TacticalHitReviewStage::Resolved);
    let shield = hit.completed_shield.as_ref().unwrap();
    let rules = before.rules.as_ref().unwrap();
    let pending = rules.pending.as_ref().unwrap();
    let cause = pending.issued_by.clone();
    let raw_id = pending.request.id;
    assert_eq!(cause, hit.cause);
    assert_ne!(cause.id, shield.cast.plan.origin.id);
    // The pending request exists before any accepted damage record. The attack
    // stores damage_roll only when its original physical faces are accepted.
    assert!(attack.damage_roll.is_none());
    assert!(!rules.rolls.iter().any(|roll| roll.request.id == raw_id));
    let damage_work = resolution.pending.as_ref().unwrap();
    assert_eq!(damage_work.work.kind, TacticalWorkKind::AttackDamage);
    assert_eq!(
        damage_work.key,
        TacticalRollKey {
            origin: attack.origin.id,
            role: TacticalRollRole::AttackDamage,
            subject: attack.target,
            occurrence: damage_work.work.occurrence,
        }
    );
    assert_eq!(raw_id, damage_work.key.request_id());
    assert!(
        matches!(&pending.purpose, PendingPurpose::TacticalResolution { key, .. }
        if *key == damage_work.key)
    );
    let attack_raw = rules
        .rolls
        .iter()
        .find(|roll| roll.request.id == hit.attack_roll)
        .unwrap();
    assert_eq!(attack_raw.accepted_by, cause);
    assert_eq!(
        attack_raw.result.dice,
        [DieResult {
            sides: 20,
            value: 20
        }]
    );
    assert_eq!(rules.entities[&attack.target].hp, 81);
    assert_eq!(protective_uses(&before, attack.target), 2);
    let channel = controller(&before, attack.actor);
    assert!(matches!(channel, TableTransportChannel::Player { .. }));
    let view = f.view(&channel).await;
    let input = faces(&view, &[1, 2]);
    let request = f.request(channel, &view, input.clone());
    assert_eq!(request.version, TABLE_SOURCE_TRANSPORT_VERSION);
    let defender = controller(&before, attack.target);
    let defender_view = f.view(&defender).await;
    Box::pin(f.reject(f.request(defender, &defender_view, input))).await;
    Box::pin(f.accept_cold(request)).await;
    let after = f.state().await;
    assert_eq!(flow(&after).version, 3);
    assert!(flow(&after).resolution.is_none());
    let rules = after.rules.as_ref().unwrap();
    assert!(rules.pending.is_none());
    assert_eq!(rules.entities[&attack.target].hp, 75);
    assert_eq!(protective_uses(&after, attack.target), 2);
    assert!(
        rules
            .rolls
            .starts_with(&before.rules.as_ref().unwrap().rolls)
    );
    let damage = rules.rolls.last().unwrap();
    assert_eq!(damage.request.id, raw_id);
    assert_eq!(damage.issued_by, cause);
    assert_eq!(
        damage.result.dice,
        [
            DieResult { sides: 4, value: 1 },
            DieResult { sides: 4, value: 2 }
        ]
    );
    Box::pin(f.finish()).await;
}

#[path = "support/source_revision_coexistence.rs"]
mod source_revision_coexistence;
