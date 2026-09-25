use std::path::Path;

use dmd_app::{CampaignRuntime, TableAction, TableEvent};
use dmd_domain::*;
use dmd_persistence::{CampaignExport, export_campaign, open_sqlite};
use dmd_rules::tactical::TacticalAction;

fn runtime(pool: sqlx::SqlitePool) -> CampaignRuntime {
    CampaignRuntime::from_content_root(
        pool,
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../content"),
    )
}

#[tokio::test]
async fn genuine_legacy_declared_second_wind_replays_retries_and_finishes_before_upgrade() {
    // Captured from f669389's genuine file-SQLite scenario at accepted event22,
    // before ReactionsV1 existed. Do not regenerate with the current executor.
    let export = CampaignExport::from_json(include_str!(
        "fixtures/legacy-declared-second-wind-f669.json"
    ))
    .unwrap();
    let event: TableEvent =
        serde_json::from_str(&export.event_journal.last().unwrap().payload_json).unwrap();
    assert!(matches!(event.action, TableAction::Adjudicate { .. }));
    assert_eq!(
        event.tactical_event.as_ref().unwrap().action,
        TacticalAction::SecondWind
    );
    let campaign = event.meta.campaign_id;
    let pool = open_sqlite("sqlite::memory:").await.unwrap();
    let app = runtime(pool.clone());
    Box::pin(app.restore_campaign(&export)).await.unwrap();
    let before = app.open_campaign(campaign).await.unwrap().state().clone();
    assert_eq!(
        before
            .encounter
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .version,
        1
    );
    let mut exported = export_campaign(&pool, campaign).await.unwrap();
    exported.exported_at_utc = export.exported_at_utc.clone();
    assert_eq!(
        exported, export,
        "restore must preserve every historical byte"
    );
    let retried = Box::pin(app.execute_table(event.meta.clone(), event.action.clone()))
        .await
        .unwrap();
    assert!(retried.already_accepted);
    assert_eq!(retried.outcome, event.outcome);
    assert_eq!(app.open_campaign(campaign).await.unwrap().state(), &before);

    let mut host = event.meta.clone();
    host.id = CommandId::new();
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
    let pending = before.rules.as_ref().unwrap().pending.as_ref().unwrap();
    let actor = pending.request.roller.unwrap();
    let character = before
        .characters
        .values()
        .find(|pc| pc.entity_id == actor)
        .unwrap();
    let mut player = host.clone();
    player.id = CommandId::new();
    player.issuer = CommandIssuer::Player(character.controlling_player_id.unwrap());
    player.actor = Some(AgentRef::Entity(actor));
    Box::pin(app.execute_table(
        player.clone(),
        TableAction::Tactical {
            action: TacticalAction::SubmitRoll {
                result: RollResult {
                    request_id: pending.request.id,
                    source: RollSource::Physical,
                    dice: vec![DieResult {
                        sides: 10,
                        value: 7,
                    }],
                },
            },
        },
    ))
    .await
    .unwrap();
    let settled = app.open_campaign(campaign).await.unwrap().state().clone();
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
    player.id = CommandId::new();
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
}
