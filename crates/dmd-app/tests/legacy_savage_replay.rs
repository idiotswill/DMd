use std::path::Path;

use dmd_app::*;
use dmd_domain::*;
use dmd_persistence::{CampaignExport, export_campaign, open_sqlite};
use dmd_rules::tactical::TacticalAction;

#[path = "support/sqlite_test_cleanup.rs"]
mod sqlite_test_cleanup;

fn runtime(pool: sqlx::SqlitePool) -> CampaignRuntime {
    CampaignRuntime::from_content_root(
        pool,
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../content"),
    )
}

#[tokio::test]
async fn genuine_legacy_savage_pause_finishes_owned_raw_sets_before_upgrade_and_retries_after_it() {
    let export =
        CampaignExport::from_json(include_str!("fixtures/legacy-savage-f960.json")).unwrap();
    let event: TableEvent =
        serde_json::from_str(&export.event_journal.last().unwrap().payload_json).unwrap();
    assert!(matches!(
        event.tactical_event.as_ref().unwrap().action,
        TacticalAction::SubmitRoll { .. }
    ));
    let campaign = event.meta.campaign_id;
    let directory = std::env::temp_dir().join(format!("dmd-legacy-savage-{}", CampaignId::new().0));
    std::fs::create_dir_all(&directory).unwrap();
    let url = format!(
        "sqlite://{}",
        directory
            .join("campaign.sqlite")
            .to_string_lossy()
            .replace('\\', "/")
    );
    let mut pool = open_sqlite(&url).await.unwrap();
    let mut app = runtime(pool.clone());
    Box::pin(app.restore_campaign(&export)).await.unwrap();
    let before = app.open_campaign(campaign).await.unwrap().state().clone();
    let flow = before.encounter.as_ref().unwrap().flow.as_ref().unwrap();
    assert_eq!(flow.version, 1);
    let attack = flow.resolution.as_ref().unwrap().attack.as_ref().unwrap();
    assert_eq!(attack.stage, TacticalAttackStage::DamageRoll);
    let actor = attack.actor;
    let target = attack.target;
    let character = before
        .characters
        .values()
        .find(|pc| pc.entity_id == actor)
        .unwrap();
    let owner = character.controlling_player_id.unwrap();
    let mut copied = export_campaign(&pool, campaign).await.unwrap();
    copied.exported_at_utc = export.exported_at_utc.clone();
    assert_eq!(
        copied, export,
        "preserve the actual historical presentation and journal bytes"
    );
    let original_retry = Box::pin(app.execute_table(event.meta.clone(), event.action.clone()))
        .await
        .unwrap();
    assert!(original_retry.already_accepted);
    assert_eq!(original_retry.outcome, event.outcome);
    assert_eq!(app.open_campaign(campaign).await.unwrap().state(), &before);

    pool.close().await;
    pool = open_sqlite(&url).await.unwrap();
    app = runtime(pool.clone());
    let view = app
        .presented_table_view(campaign, TableViewer::Player(owner))
        .await
        .unwrap();
    let pending = view.roll.as_ref().unwrap();
    assert_eq!(pending.dice, vec![DieSpec { count: 2, sides: 4 }]);
    let channel = TableTransportChannel::Player {
        player_id: owner,
        character_id: character.id,
    };
    let options = app
        .table_roll_options(TableRollOptionsRequest {
            campaign_id: campaign,
            channel: channel.clone(),
            revision: view.revision,
            roll_id: pending.id,
        })
        .await
        .unwrap();
    assert_eq!(options.savage_attacker.unwrap().weapon_dice, 2);
    let raw = |values: [u16; 2]| RollResult {
        request_id: pending.id,
        source: RollSource::Physical,
        dice: values
            .into_iter()
            .map(|value| DieResult { sides: 4, value })
            .collect(),
    };
    let first = raw([1, 2]);
    let second = raw([4, 4]);
    let request = TableTransportRequest {
        version: TABLE_TRANSPORT_VERSION,
        command_id: CommandId::new(),
        campaign_id: campaign,
        session_id: event.meta.session_id,
        channel,
        revision: view.revision,
        input: TableTransportInput::Action(Box::new(TableAction::Tactical {
            action: TacticalAction::SubmitSavageAttacker {
                roll: SavageAttackerRoll {
                    weapon_dice: Some(2),
                    first: first.clone(),
                    second: second.clone(),
                    chosen: DamageRollChoice::First,
                    inspiration: None,
                },
            },
        })),
    };
    let foreign = before
        .characters
        .values()
        .find(|pc| pc.entity_id != actor)
        .unwrap();
    let foreign_owner = foreign.controlling_player_id.unwrap();
    let mut forged = request.clone();
    forged.command_id = CommandId::new();
    forged.channel = TableTransportChannel::Player {
        player_id: foreign_owner,
        character_id: foreign.id,
    };
    forged.revision = app
        .presented_table_view(campaign, TableViewer::Player(foreign_owner))
        .await
        .unwrap()
        .revision;
    assert!(Box::pin(app.submit_presented_table(forged)).await.is_err());
    assert_eq!(app.open_campaign(campaign).await.unwrap().state(), &before);
    let mut host = event.meta.clone();
    host.id = CommandId::new();
    host.issuer = CommandIssuer::Admin;
    host.actor = None;
    host.expected_event_sequence = before.applied_event_sequence;
    assert!(
        Box::pin(app.execute_table(
            host.clone(),
            TableAction::Tactical {
                action: TacticalAction::UpgradeExecution,
            }
        ))
        .await
        .is_err()
    );

    let accepted = Box::pin(app.submit_presented_table(request.clone()))
        .await
        .unwrap();
    let settled = app.open_campaign(campaign).await.unwrap().state().clone();
    let rules = settled.rules.as_ref().unwrap();
    let roll = rules.rolls.last().unwrap();
    let savage = roll.savage_attacker.as_ref().unwrap();
    assert_eq!(savage.first.dice, first.dice);
    assert_eq!(savage.second.dice, second.dice);
    assert_eq!(savage.chosen, DamageRollChoice::First);
    assert_eq!(roll.result.dice, first.dice);
    assert_eq!(
        rules.entities[&target].hp,
        before.rules.as_ref().unwrap().entities[&target].hp - 6
    );
    assert_eq!(
        rules.entities[&actor]
            .character_features
            .as_ref()
            .unwrap()
            .savage_attacker_turn,
        Some(1)
    );
    assert!(rules.pending.is_none());
    assert!(
        settled
            .encounter
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .resolution
            .is_none()
    );
    pool.close().await;
    pool = open_sqlite(&url).await.unwrap();
    app = runtime(pool.clone());
    assert_eq!(
        Box::pin(app.submit_presented_table(request.clone()))
            .await
            .unwrap(),
        accepted
    );
    let mut player = host.clone();
    player.id = CommandId::new();
    player.issuer = CommandIssuer::Player(owner);
    player.actor = Some(AgentRef::Entity(actor));
    player.expected_event_sequence = settled.applied_event_sequence;
    assert!(
        Box::pin(app.execute_table(
            player,
            TableAction::Tactical {
                action: TacticalAction::EndTurn,
            }
        ))
        .await
        .is_err()
    );
    host.id = CommandId::new();
    host.expected_event_sequence = settled.applied_event_sequence;
    Box::pin(app.execute_table(
        host,
        TableAction::Tactical {
            action: TacticalAction::UpgradeExecution,
        },
    ))
    .await
    .unwrap();
    let upgraded = app.open_campaign(campaign).await.unwrap().state().clone();
    assert_eq!(upgraded.rules, settled.rules);
    let mut expected_encounter = settled.encounter.clone().unwrap();
    expected_encounter.flow.as_mut().unwrap().version = 2;
    assert_eq!(upgraded.encounter, Some(expected_encounter));
    assert_eq!(
        Box::pin(app.submit_presented_table(request)).await.unwrap(),
        accepted
    );
    let resumed = export_campaign(&pool, campaign).await.unwrap();
    let mirror_pool = open_sqlite("sqlite::memory:").await.unwrap();
    let mirror = runtime(mirror_pool.clone());
    Box::pin(mirror.restore_campaign(&resumed)).await.unwrap();
    assert_eq!(
        mirror.open_campaign(campaign).await.unwrap().state(),
        &upgraded
    );
    mirror_pool.close().await;
    pool.close().await;
    drop(app);
    drop(pool);
    sqlite_test_cleanup::remove_closed_directory(&directory)
        .await
        .unwrap();
}
