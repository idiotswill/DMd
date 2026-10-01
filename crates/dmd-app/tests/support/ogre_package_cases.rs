//! Package integrity only. No Ogre creation or gameplay is exercised here.
use super::*;

#[tokio::test]
async fn immutable_ogre_package_is_required_after_cache_warmup_and_repair_recovers() {
    let _ = dmd_rules::tactical_definitions::bundled_ogre().unwrap();
    for mode in ["missing", "undeclared", "changed", "rehashed"] {
        let f = Fixture::new();
        let (pool, runtime) = f.runtime().await;
        f.initialize(&runtime).await;
        let before = export_campaign(&pool, f.state.campaign_id()).await.unwrap();
        let manifest_path = f.content.join("manifest.json");
        let original_manifest = fs::read(&manifest_path).unwrap();
        let source_path = f.content.join("ogre-v1.json");
        let original_source = fs::read(&source_path).unwrap();
        let mut manifest: serde_json::Value = serde_json::from_slice(&original_manifest).unwrap();
        let files = manifest["files"].as_array_mut().unwrap();
        match mode {
            "missing" => fs::remove_file(&source_path).unwrap(),
            "undeclared" => files.retain(|file| file["path"] != "ogre-v1.json"),
            _ => {
                let mut source: serde_json::Value =
                    serde_json::from_slice(&original_source).unwrap();
                source["statistics"]["hit_points"] = serde_json::json!(69);
                let bytes = serde_json::to_vec_pretty(&source).unwrap();
                fs::write(&source_path, &bytes).unwrap();
                if mode == "rehashed" {
                    let file = files
                        .iter_mut()
                        .find(|file| file["path"] == "ogre-v1.json")
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
        assert!(
            dmd_rules::tactical_creatures::current_creature_sources()
                .unwrap()
                .iter()
                .all(|source| source.id != "ogre")
        );
        drop((recovered, runtime));
        pool.close().await;
    }
}
