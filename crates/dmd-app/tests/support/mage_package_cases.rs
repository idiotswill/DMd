//! Package integrity only. No Mage creation or gameplay is exercised here.
use super::*;
#[path = "mage_source_store.rs"]
mod store;

#[tokio::test]
async fn immutable_mage_v2_package_refusals_preserve_full_populated_store_and_repair_recovers() {
    let _ = dmd_rules::tactical_definitions::bundled_mage_v2().unwrap();
    for mode in ["missing", "undeclared", "changed", "rehashed"] {
        let f = Fixture::new();
        let (pool, runtime) = f.runtime().await;
        f.initialize(&runtime).await;
        let before = export_campaign(&pool, f.state.campaign_id()).await.unwrap();
        let rows_before = store::typed_rows(&pool).await;
        let manifest_path = f.content.join("manifest.json");
        let original_manifest = fs::read(&manifest_path).unwrap();
        let source_path = f.content.join("mage-v2.json");
        let original_source = fs::read(&source_path).unwrap();
        let mut manifest: serde_json::Value = serde_json::from_slice(&original_manifest).unwrap();
        let files = manifest["files"].as_array_mut().unwrap();
        match mode {
            "missing" => fs::remove_file(&source_path).unwrap(),
            "undeclared" => files.retain(|file| file["path"] != "mage-v2.json"),
            _ => {
                let mut source: serde_json::Value =
                    serde_json::from_slice(&original_source).unwrap();
                source["statistics"]["hit_points"] = serde_json::json!(82);
                let bytes = serde_json::to_vec_pretty(&source).unwrap();
                fs::write(&source_path, &bytes).unwrap();
                if mode == "rehashed" {
                    let file = files
                        .iter_mut()
                        .find(|file| file["path"] == "mage-v2.json")
                        .unwrap();
                    file["byte_len"] = serde_json::json!(bytes.len());
                    file["checksum"]["value"] = serde_json::json!(fnv1a64_hex(&bytes));
                }
            }
        }
        fs::write(
            &manifest_path,
            serde_json::to_vec_pretty(&manifest).unwrap(),
        )
        .unwrap();
        assert!(
            runtime
                .query_rules(
                    f.state.campaign_id(),
                    CommandIssuer::Admin,
                    RulesQuery::PendingRoll
                )
                .await
                .is_err(),
            "{mode} source must not authorize a query"
        );
        assert!(
            runtime
                .execute_rules(
                    f.context(CommandIssuer::System, None, 1),
                    RulesAction::AdvanceTime {
                        seconds: 1,
                        ruling: ruling(),
                    }
                )
                .await
                .is_err(),
            "{mode} source must not authorize a command"
        );
        let mut after = export_campaign(&pool, f.state.campaign_id()).await.unwrap();
        after.exported_at_utc = before.exported_at_utc.clone();
        assert_eq!(
            after, before,
            "{mode} package failure must leave the full store unchanged"
        );

        assert_eq!(
            store::typed_rows(&pool).await,
            rows_before,
            "every typed row must remain exact"
        );

        // Restore only the real package, preserving the same runtime and campaign.
        fs::write(&source_path, &original_source).unwrap();
        fs::write(&manifest_path, &original_manifest).unwrap();
        runtime
            .query_rules(
                f.state.campaign_id(),
                CommandIssuer::Admin,
                RulesQuery::PendingRoll,
            )
            .await
            .unwrap();
        let receipt = runtime
            .execute_rules(
                f.context(CommandIssuer::System, None, 1),
                RulesAction::AdvanceTime {
                    seconds: 1,
                    ruling: ruling(),
                },
            )
            .await
            .unwrap();
        assert_eq!(receipt.outcome, RulesOutcome::Changed);
        let recovered = runtime.open_campaign(f.state.campaign_id()).await.unwrap();
        assert_eq!(recovered.state().applied_event_sequence, 2);
        assert_eq!(recovered.state().clock.now, WorldInstant(1));
        let current_mages = dmd_rules::tactical_creatures::current_creature_sources()
            .unwrap()
            .into_iter()
            .filter(|source| source.id == "mage")
            .map(dmd_rules::tactical_creatures::creature_source_pin)
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(
            current_mages,
            vec![
                dmd_rules::tactical_creatures::creature_source_pin(
                    dmd_rules::tactical_definitions::bundled_mage_v2().unwrap()
                )
                .unwrap()
            ]
        );
        drop((recovered, runtime));
        pool.close().await;
    }
}
