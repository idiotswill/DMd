//! The installed sidecar cannot supply a caller-rehashed replacement source.
use super::*;
#[tokio::test]
async fn physical_mass_sidecar_missing_undeclared_or_rehashed_source_refuses_without_campaign_mutation()
 {
    for mode in ["missing", "undeclared", "rehashed-mass"] {
        let f = Fixture::new();
        let (pool, runtime) = f.runtime().await;
        f.initialize(&runtime).await;
        let before = export_campaign(&pool, f.state.campaign_id()).await.unwrap();
        let manifest_path = f.content.join("manifest.json");
        let source_path = f.content.join("equipment-mass-v1.json");
        let original_manifest = fs::read(&manifest_path).unwrap();
        let original_source = fs::read(&source_path).unwrap();
        let mut manifest: serde_json::Value = serde_json::from_slice(&original_manifest).unwrap();
        let files = manifest["files"].as_array_mut().unwrap();
        match mode {
            "missing" => fs::remove_file(&source_path).unwrap(),
            "undeclared" => files.retain(|file| file["path"] != "equipment-mass-v1.json"),
            _ => {
                let mut source: serde_json::Value =
                    serde_json::from_slice(&original_source).unwrap();
                source["entries"][0]["mass"]["Intrinsic"]["mass"] = serde_json::json!(1);
                let bytes = serde_json::to_vec_pretty(&source).unwrap();
                fs::write(&source_path, &bytes).unwrap();
                let file = files
                    .iter_mut()
                    .find(|file| file["path"] == "equipment-mass-v1.json")
                    .unwrap();
                file["byte_len"] = serde_json::json!(bytes.len());
                file["checksum"]["value"] = serde_json::json!(fnv1a64_hex(&bytes));
            }
        }
        fs::write(
            &manifest_path,
            serde_json::to_vec_pretty(&manifest).unwrap(),
        )
        .unwrap();
        assert!(
            runtime
                .execute_rules(
                    f.context(CommandIssuer::System, None, 1),
                    RulesAction::AdvanceTime {
                        seconds: 1,
                        ruling: ruling()
                    }
                )
                .await
                .is_err()
        );
        let mut after = export_campaign(&pool, f.state.campaign_id()).await.unwrap();
        after.exported_at_utc = before.exported_at_utc.clone();
        assert_eq!(
            after, before,
            "invalid source {mode} mutated the saved campaign"
        );
        fs::write(source_path, original_source).unwrap();
        fs::write(manifest_path, original_manifest).unwrap();
        runtime
            .query_rules(
                f.state.campaign_id(),
                CommandIssuer::Admin,
                RulesQuery::PendingRoll,
            )
            .await
            .unwrap();
        drop(runtime);
        pool.close().await;
    }
}
