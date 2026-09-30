//! Genuine flow4 exports and the original-source verified continuation suite.
//! Preserve the original bytes and semantics; never regenerate with a successor.
use std::collections::HashSet;
use std::path::{Path, PathBuf};

use dmd_app::*;
use dmd_domain::*;
use dmd_persistence::{CampaignExport, export_campaign, open_sqlite_path};
use dmd_rules::tactical::TacticalAction;

#[path = "support/sqlite_test_cleanup.rs"]
mod sqlite_test_cleanup;

const SHIELD_SELECTED: &str = "flow4-hit-selected-shield.json";
const MISSILE_SELECTED: &str = "flow4-missile-selected-shield.json";
const THREE_FACES: &str = "flow4-missile-three-faces.json";
const ALL_FACES: &str = "flow4-missile-all-faces-before-impact.json";
const FIRST_CHILD: &str = "flow4-missile-first-concentration-child.json";
const FIFTH_CHILD: &str = "flow4-missile-penultimate-concentration-child.json";
const ORDINARY_SAVE: &str = "flow4-ordinary-committed-first-save.json";
const CAPTURES: [&str; 7] = [
    SHIELD_SELECTED,
    MISSILE_SELECTED,
    THREE_FACES,
    ALL_FACES,
    FIRST_CHILD,
    FIFTH_CHILD,
    ORDINARY_SAVE,
];

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
            "required genuine flow4 capture {} is missing/unreadable: {error}; import the verified generator artifact, never skip or synthesize it",
            path.display()
        )
    });
    let export = Box::new(CampaignExport::from_json(std::str::from_utf8(&bytes).unwrap()).unwrap());
    assert_eq!(
        export.to_json().unwrap().as_bytes(),
        bytes,
        "the actual generator's exported bytes must round-trip unchanged"
    );
    assert_eq!(flow(&image(&export)).version, 4);
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

fn assert_presented_roll(view: &TablePresentedView, canonical: &RollRequest) {
    let shown = view.roll.as_ref().unwrap();
    assert_ne!(
        shown.id, canonical.id,
        "the public request uses an opaque capability"
    );
    assert_eq!(shown.roller, canonical.roller);
    assert_eq!(shown.dice, canonical.dice);
    assert_eq!(shown.modifier, canonical.modifier);
    assert_eq!(shown.mode, canonical.mode);
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
        let directory = std::env::temp_dir().join(format!("dmd-flow4-{}", CampaignId::new().0));
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
                Box::pin(self.app.submit_presented_table(request.clone()))
                    .await
                    .unwrap(),
                response,
                "all original envelopes recover before current owner/revision/phase admission"
            );
            self.assert_export(&before).await;
            let mut changed = request;
            changed.input = if matches!(&changed.input,
                TableTransportInput::Action(action)
                if matches!(&**action, TableAction::Tactical { action: TacticalAction::Dodge }))
            {
                tactical(TacticalAction::Disengage)
            } else {
                tactical(TacticalAction::Dodge)
            };
            assert_ne!(
                serde_json::to_string(&changed).unwrap(),
                binding.request_json
            );
            Box::pin(self.reject(changed)).await;
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
        for cause in &self.original.event_causes {
            assert!(accepted.event_causes.contains(cause));
        }
        for snapshot in &self.original.snapshots {
            assert!(accepted.snapshots.contains(snapshot));
        }
        for observation in &self.original.observations {
            assert!(accepted.observations.contains(observation));
        }
        assert_eq!(accepted.play_sessions, self.original.play_sessions);
        assert_eq!(
            accepted.play_session_participants,
            self.original.play_session_participants
        );
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
async fn genuine_flow4_selected_owned_shield_finishes_as_a_miss_without_replacing_dice() {
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
    assert_eq!(flow(&after).version, 4);
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

fn missile_record(state: &CampaignState) -> &TacticalMissile {
    let resolution = flow(state).resolution.as_ref().unwrap();
    assert_eq!(resolution.missiles.len(), 1);
    &resolution.missiles[0]
}

fn missile_cast(state: &CampaignState) -> &TacticalCasting {
    let occurrence = missile_record(state).cast;
    flow(state)
        .resolution
        .as_ref()
        .unwrap()
        .casts
        .iter()
        .find(|cast| cast.cast.plan.occurrence == occurrence)
        .unwrap()
}

fn pending_key(state: &CampaignState) -> TacticalRollKey {
    match &state
        .rules
        .as_ref()
        .unwrap()
        .pending
        .as_ref()
        .unwrap()
        .purpose
    {
        PendingPurpose::TacticalResolution { key, .. } => *key,
        _ => panic!("the genuine pause requires a tactical raw request"),
    }
}

fn assert_boundary(name: &str, state: &CampaignState) {
    assert_eq!(flow(state).version, 4);
    let resolution = flow(state).resolution.as_ref().unwrap();
    match name {
        SHIELD_SELECTED => {
            let hit = resolution.hit_review.as_ref().unwrap();
            assert_eq!(hit.stage, TacticalHitReviewStage::Selected);
            assert!(hit.completed_shield.is_none());
            assert!(state.rules.as_ref().unwrap().pending.is_none());
        }
        ORDINARY_SAVE => {
            assert!(resolution.missiles.is_empty() && resolution.hit_review.is_none());
            assert_eq!(resolution.casts.len(), 1);
            let casting = &resolution.casts[0];
            assert_eq!(casting.cast.phase, SpellCastPhase::Committed);
            assert_eq!(casting.cast.plan.choice.spell_id, "hold-person");
            assert_eq!(casting.targets.len(), 1);
            assert!(casting.targets[0].source_type_matches);
            assert!(casting.completed.is_empty());
            let key = pending_key(state);
            assert_eq!(key.role, TacticalRollRole::SpellSave);
            assert_eq!(key.origin, casting.cast.plan.origin.id);
            assert_eq!(key.subject, casting.targets[0].actor);
        }
        _ => {
            let missile = missile_record(state);
            assert_eq!(missile.darts.len(), 6);
            let (amounts, completed, stage) = match name {
                MISSILE_SELECTED => (0, 0, TacticalMissileStage::Selected { respondent: 1 }),
                THREE_FACES => (3, 0, TacticalMissileStage::Amounts),
                ALL_FACES => (6, 0, TacticalMissileStage::Impacts),
                FIRST_CHILD => (6, 1, TacticalMissileStage::Impacts),
                FIFTH_CHILD => (6, 5, TacticalMissileStage::Impacts),
                _ => panic!("unrecognized required capture"),
            };
            assert_eq!(missile.stage, stage);
            assert_eq!(
                missile
                    .darts
                    .iter()
                    .filter(|dart| dart.amount.is_some())
                    .count(),
                amounts
            );
            assert_eq!(
                missile
                    .darts
                    .iter()
                    .filter(|dart| dart.completed_by.is_some())
                    .count(),
                completed
            );
            if matches!(name, FIRST_CHILD | FIFTH_CHILD) {
                assert_eq!(pending_key(state).role, TacticalRollRole::Concentration);
            }
            if name == THREE_FACES {
                assert_eq!(pending_key(state).role, TacticalRollRole::SpellAmount);
            }
        }
    }
}

#[tokio::test]
async fn seven_original_flow4_exports_restore_cold_and_retry_every_original_binding() {
    for name in CAPTURES {
        let f = Box::pin(Fixture::restore(name)).await;
        assert_boundary(name, &*f.state().await);
        Box::pin(f.retry_original_bindings()).await;
        f.assert_export(&f.original).await;
        f.close().await;
    }
}

async fn continue_missile(name: &str) {
    let mut f = Box::pin(Fixture::restore(name)).await;
    let initial = f.state().await;
    assert_boundary(name, &initial);
    let original = missile_record(&initial).clone();
    let cast = missile_cast(&initial);
    assert_eq!(cast.cast.plan.choice.spell_id, "magic-missile");
    assert_eq!(cast.cast.plan.program.spell_level, 4);
    let caster = cast.cast.plan.choice.actor;
    let cast_origin = cast.cast.plan.origin.id;
    let targets = original
        .darts
        .iter()
        .map(|dart| dart.target)
        .collect::<HashSet<_>>();
    let shielded = name == MISSILE_SELECTED;
    let mut steps = 0;
    loop {
        let before = f.state().await;
        if flow(&before).resolution.is_none() {
            break;
        }
        assert_eq!(flow(&before).version, 4);
        let missile = missile_record(&before);
        assert_eq!(missile.cause, original.cause);
        assert_eq!(missile.work, original.work);
        assert_eq!(missile.order, original.order);
        for (old, current) in original.darts.iter().zip(&missile.darts) {
            if old.amount.is_some() {
                assert_eq!(current.amount, old.amount);
            }
            if old.selected_by.is_some() {
                assert_eq!(current.selected_by, old.selected_by);
            }
            if old.completed_by.is_some() {
                assert_eq!(current.completed_by, old.completed_by);
            }
        }
        assert_eq!(
            missile
                .darts
                .iter()
                .map(|dart| (dart.at, dart.target))
                .collect::<Vec<_>>(),
            original
                .darts
                .iter()
                .map(|dart| (dart.at, dart.target))
                .collect::<Vec<_>>()
        );
        assert_eq!(missile_cast(&before).cast.plan, cast.cast.plan);
        let (channel, view, input) = if let Some(pending) = &before.rules.as_ref().unwrap().pending
        {
            let key = pending_key(&before);
            let actor = pending.request.roller.unwrap();
            let channel = controller(&before, actor);
            let view = f.view(&channel).await;
            assert_presented_roll(&view, &pending.request);
            let face = match key.role {
                TacticalRollRole::SpellAmount => {
                    assert_eq!(actor, caster);
                    assert_eq!(missile.stage, TacticalMissileStage::Amounts);
                    assert!(missile.darts.iter().all(|dart| dart.completed_by.is_none()));
                    for target in &targets {
                        assert_eq!(
                            before.rules.as_ref().unwrap().entities[target].hp,
                            initial.rules.as_ref().unwrap().entities[target].hp
                        );
                    }
                    1
                }
                TacticalRollRole::Concentration => {
                    assert!(!shielded);
                    let outer = f.view(&controller(&before, caster)).await;
                    assert!(
                        outer
                            .tactical
                            .as_ref()
                            .unwrap()
                            .continuation
                            .as_ref()
                            .unwrap()
                            .choices
                            .is_empty()
                    );
                    20
                }
                _ => panic!("unexpected new raw role in the original missile continuation"),
            };
            let input = faces(&view, &[face]);
            (channel, view, input)
        } else if let TacticalMissileStage::Selected { respondent } = missile.stage {
            assert!(shielded);
            let actor = missile.respondents[usize::from(respondent)].response.actor;
            let channel = controller(&before, actor);
            let view = f.view(&channel).await;
            let selected = view
                .tactical
                .as_ref()
                .unwrap()
                .missile
                .as_ref()
                .unwrap()
                .responses
                .iter()
                .find(|response| response.actor == actor)
                .unwrap();
            assert!(selected.selected);
            let choice = selected.shield[0].clone();
            assert_eq!(choice.spell_id, "shield");
            assert_eq!(choice.resource, SpellResourceChoice::SourceFeature);
            let input = TableTransportInput::MissileResponse {
                handle: selected.key,
                decision: Box::new(TableMissileInput::Cast { choice }),
            };
            (channel, view, input)
        } else {
            assert_eq!(missile.stage, TacticalMissileStage::Impacts);
            assert!(missile.darts.iter().all(|dart| dart.amount.is_some()));
            let channel = controller(&before, caster);
            let view = f.view(&channel).await;
            let continuation = view
                .tactical
                .as_ref()
                .unwrap()
                .continuation
                .as_ref()
                .unwrap();
            assert!(
                continuation.choices.len() >= 2,
                "last singleton must execute automatically"
            );
            let input = TableTransportInput::SelectWork {
                handle: continuation.choices.last().unwrap().handle,
            };
            (channel, view, input)
        };
        // Host cannot borrow any owned source decision or raw request. The
        // foreign projection is taken before the complete no-write snapshot.
        assert!(matches!(
            channel,
            TableTransportChannel::SourceCreature { .. }
        ));
        let foreign = f.view(&TableTransportChannel::Host).await;
        Box::pin(f.reject(f.request(TableTransportChannel::Host, &foreign, input.clone()))).await;
        let request = f.request(channel, &view, input);
        let command = request.command_id;
        let last_singleton = before
            .rules
            .as_ref()
            .unwrap()
            .pending
            .as_ref()
            .filter(|_| pending_key(&before).role == TacticalRollRole::Concentration)
            .and_then(|_| {
                let remaining = missile
                    .darts
                    .iter()
                    .filter(|dart| dart.completed_by.is_none())
                    .collect::<Vec<_>>();
                (remaining.len() == 1).then(|| remaining[0].at)
            });
        Box::pin(f.accept_cold(request)).await;
        if let Some(pending) = &before.rules.as_ref().unwrap().pending {
            let after = f.state().await;
            let rules = after.rules.as_ref().unwrap();
            let saved = rules
                .rolls
                .iter()
                .find(|roll| roll.request.id == pending.request.id)
                .unwrap();
            assert_eq!(saved.request, pending.request);
            assert_eq!(saved.result.request_id, pending.request.id);
            assert_eq!(saved.accepted_by.id, command);
            assert!(
                rules
                    .rolls
                    .starts_with(&before.rules.as_ref().unwrap().rolls)
            );
        }
        if let Some(at) = last_singleton {
            let after = f.state().await;
            let final_dart = missile_record(&after)
                .darts
                .iter()
                .find(|dart| dart.at == at)
                .unwrap();
            assert!(final_dart.selected_by.is_none());
            assert_eq!(final_dart.completed_by.as_ref().unwrap().id, command);
            assert_eq!(pending_key(&after).role, TacticalRollRole::Concentration);
            assert_ne!(
                after
                    .rules
                    .as_ref()
                    .unwrap()
                    .pending
                    .as_ref()
                    .unwrap()
                    .request
                    .id,
                before
                    .rules
                    .as_ref()
                    .unwrap()
                    .pending
                    .as_ref()
                    .unwrap()
                    .request
                    .id
            );
        }
        steps += 1;
        assert!(steps <= 20, "bounded six-dart continuation did not settle");
    }
    let completed = f.state().await;
    let rules = completed.rules.as_ref().unwrap();
    assert_eq!(flow(&completed).version, 4);
    assert!(rules.pending.is_none());
    assert!(
        rules
            .rolls
            .starts_with(&initial.rules.as_ref().unwrap().rolls)
    );
    let amounts = rules
        .rolls
        .iter()
        .filter(|roll| {
            matches!(&roll.purpose,
        PendingPurpose::TacticalResolution { key, .. }
        if key.origin==cast_origin && key.role==TacticalRollRole::SpellAmount)
        })
        .collect::<Vec<_>>();
    assert_eq!(amounts.len(), 6);
    assert_eq!(
        amounts
            .iter()
            .map(|roll| roll.request.id)
            .collect::<HashSet<_>>()
            .len(),
        6
    );
    for amount in amounts {
        assert_eq!(amount.result.source, RollSource::Physical);
        assert_eq!(amount.result.dice, [DieResult { sides: 4, value: 1 }]);
    }
    for target in targets {
        if shielded {
            assert_eq!(rules.entities[&target].hp, 81);
            assert_eq!(
                protective_uses(&completed, target),
                protective_uses(&initial, target) + 1
            );
            assert!(
                rules
                    .timing
                    .as_ref()
                    .unwrap()
                    .reactions_spent
                    .contains(&target)
            );
        } else {
            let pending = original
                .darts
                .iter()
                .filter(|dart| dart.target == target && dart.completed_by.is_none())
                .count() as u32;
            assert_eq!(
                rules.entities[&target].hp,
                initial.rules.as_ref().unwrap().entities[&target].hp - 2 * pending
            );
            assert_eq!(
                rules.entities[&target].concentration,
                initial.rules.as_ref().unwrap().entities[&target].concentration
            );
            assert!(rules.entities[&target].concentration.is_some());
        }
    }
    assert_eq!(
        rules
            .tactical_creatures
            .as_ref()
            .unwrap()
            .runtime(caster)
            .unwrap()
            .limited_uses,
        initial
            .rules
            .as_ref()
            .unwrap()
            .tactical_creatures
            .as_ref()
            .unwrap()
            .runtime(caster)
            .unwrap()
            .limited_uses
    );
    assert!(rules.timing.as_ref().unwrap().action_spent);
    Box::pin(f.finish()).await;
}

#[tokio::test]
async fn selected_owned_flow4_missile_shields_pay_once_then_prevent_all_six_darts() {
    Box::pin(continue_missile(MISSILE_SELECTED)).await;
}
#[tokio::test]
async fn three_flow4_faces_continue_through_barrier_and_six_concentration_children() {
    Box::pin(continue_missile(THREE_FACES)).await;
}
#[tokio::test]
async fn all_flow4_faces_keep_original_impacts_and_concentration_children() {
    Box::pin(continue_missile(ALL_FACES)).await;
}
#[tokio::test]
async fn first_flow4_concentration_child_precedes_the_remaining_five_darts() {
    Box::pin(continue_missile(FIRST_CHILD)).await;
}
#[tokio::test]
async fn fifth_flow4_child_resumes_the_last_automatic_singleton_and_its_save() {
    Box::pin(continue_missile(FIFTH_CHILD)).await;
}

#[tokio::test]
async fn committed_flow4_hold_person_keeps_its_paid_source_and_original_first_save() {
    let mut f = Box::pin(Fixture::restore(ORDINARY_SAVE)).await;
    let before = f.state().await;
    assert_boundary(ORDINARY_SAVE, &before);
    let casting = &flow(&before).resolution.as_ref().unwrap().casts[0];
    let caster = casting.cast.plan.choice.actor;
    let target = casting.targets[0].actor;
    let origin = casting.cast.plan.origin.id;
    let pending = before.rules.as_ref().unwrap().pending.as_ref().unwrap();
    let original_request = pending.request.clone();
    assert_eq!(original_request.roller, Some(target));
    assert!(
        before.rules.as_ref().unwrap().entities[&caster]
            .concentration
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
    let channel = controller(&before, target);
    let owned = f.view(&channel).await;
    assert_presented_roll(&owned, &original_request);
    let input = faces(&owned, &[1]);
    for foreign in [TableTransportChannel::Host, controller(&before, caster)] {
        let projected = f.view(&foreign).await;
        Box::pin(f.reject(f.request(foreign, &projected, input.clone()))).await;
    }
    Box::pin(f.accept_cold(f.request(channel, &owned, input))).await;
    let after = f.state().await;
    assert_eq!(flow(&after).version, 4);
    assert!(flow(&after).resolution.is_none());
    let rules = after.rules.as_ref().unwrap();
    assert!(rules.pending.is_none());
    assert!(dmd_rules::active_conditions(rules, target).contains(&Condition::Paralyzed));
    assert_eq!(
        rules.entities[&caster].concentration,
        before.rules.as_ref().unwrap().entities[&caster].concentration
    );
    assert_eq!(
        rules
            .tactical_creatures
            .as_ref()
            .unwrap()
            .runtime(caster)
            .unwrap()
            .limited_uses,
        before
            .rules
            .as_ref()
            .unwrap()
            .tactical_creatures
            .as_ref()
            .unwrap()
            .runtime(caster)
            .unwrap()
            .limited_uses
    );
    let raw = rules.rolls.last().unwrap();
    assert_eq!(raw.request, original_request);
    assert_eq!(raw.result.source, RollSource::Physical);
    assert_eq!(
        raw.result.dice,
        [DieResult {
            sides: 20,
            value: 1
        }]
    );
    assert!(
        matches!(&raw.purpose,PendingPurpose::TacticalResolution { key,.. }
        if key.role==TacticalRollRole::SpellSave && key.origin==origin && key.subject==target)
    );
    assert!(
        rules
            .rolls
            .starts_with(&before.rules.as_ref().unwrap().rolls)
    );
    Box::pin(f.finish()).await;
}
