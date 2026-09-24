use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use dmd_app::{CampaignRuntime, RunnableCampaignError};
use dmd_domain::{
    Campaign, CampaignId, CampaignState, CampaignStatus, CatalogLoadError, ContentResolutionError,
    VersionedRef, WorldClock, WorldInstant,
};
use dmd_persistence::{
    CampaignPurgeAuthorization, LifecycleError, export_campaign,
    open_campaign as raw_open_campaign, open_sqlite, purge_campaign,
};

static NEXT_TEST_DIR: AtomicU64 = AtomicU64::new(0);

struct TestDir {
    path: PathBuf,
}

impl TestDir {
    fn new(label: &str) -> Self {
        let serial = NEXT_TEST_DIR.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "dmd-runnable-{label}-{}-{serial}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect("test directory should be created");
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn state(
    name: &str,
    seed: u64,
    ruleset: (&str, &str),
    content_packs: &[(&str, &str)],
) -> CampaignState {
    CampaignState::empty(
        Campaign {
            id: CampaignId::new(),
            display_name: name.into(),
            status: CampaignStatus::Active,
            world_seed: seed,
            ruleset: versioned_ref(ruleset.0, ruleset.1),
            content_packs: content_packs
                .iter()
                .map(|(id, version)| versioned_ref(id, version))
                .collect(),
        },
        WorldClock {
            now: WorldInstant(100),
            calendar_id: "test.calendar".into(),
        },
    )
}

fn versioned_ref(id: &str, version: &str) -> VersionedRef {
    VersionedRef {
        id: id.into(),
        version: version.into(),
    }
}

async fn file_pool(path: &Path) -> sqlx::SqlitePool {
    open_sqlite(&format!("sqlite://{}", path.display()))
        .await
        .expect("file-backed SQLite should open")
}

fn pack_dir(root: &Path, name: &str) -> PathBuf {
    let path = root.join(name);
    fs::create_dir_all(&path).expect("pack directory should be created");
    path
}

fn write_ruleset(root: &Path, directory: &str, id: &str, version: &str) -> PathBuf {
    write_ruleset_with_file(root, directory, id, version, None)
}

fn write_ruleset_with_file(
    root: &Path,
    directory: &str,
    id: &str,
    version: &str,
    declared_file: Option<(&str, &[u8])>,
) -> PathBuf {
    let directory = pack_dir(root, directory);
    let files = if let Some((relative, bytes)) = declared_file {
        let file_path = directory.join(relative);
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).expect("declared-file parent should be created");
        }
        fs::write(&file_path, bytes).expect("declared content should be written");
        format!(
            r#"[{{"path":"{relative}","byte_len":{},"checksum":{{"algorithm":"fnv1a64","value":"{}"}}}}]"#,
            bytes.len(),
            fnv1a64(bytes)
        )
    } else {
        "[]".into()
    };
    let manifest = format!(
        r#"{{
  "manifest_schema_version": 1,
  "content_contract_version": 1,
  "kind": "ruleset",
  "id": "{id}",
  "version": "{version}",
  "compatible_rulesets": [],
  "dependencies": [],
  "files": {files}
}}"#
    );
    fs::write(directory.join("manifest.json"), manifest)
        .expect("ruleset manifest should be written");
    directory
}

fn write_pack(
    root: &Path,
    directory: &str,
    id: &str,
    version: &str,
    compatible_ruleset: (&str, &str),
    dependencies: &[(&str, &str)],
) -> PathBuf {
    let directory = pack_dir(root, directory);
    let dependencies = dependencies
        .iter()
        .map(|(dependency_id, dependency_version)| {
            format!(r#"{{"id":"{dependency_id}","version":"{dependency_version}"}}"#)
        })
        .collect::<Vec<_>>()
        .join(",");
    let manifest = format!(
        r#"{{
  "manifest_schema_version": 1,
  "content_contract_version": 1,
  "kind": "content_pack",
  "id": "{id}",
  "version": "{version}",
  "compatible_rulesets": [{{"id":"{}","version":"{}"}}],
  "dependencies": [{dependencies}],
  "files": []
}}"#,
        compatible_ruleset.0, compatible_ruleset.1
    );
    fs::write(directory.join("manifest.json"), manifest).expect("pack manifest should be written");
    directory
}

fn fnv1a64(bytes: &[u8]) -> String {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}

#[tokio::test]
async fn create_open_restart_and_restore_use_the_runnable_boundary() {
    let test = TestDir::new("happy-path");
    let content = test.path().join("content");
    fs::create_dir_all(&content).expect("content root should exist");
    write_ruleset(&content, "rules", "rules.alpha", "1");
    write_pack(
        &content,
        "pack",
        "pack.alpha",
        "1",
        ("rules.alpha", "1"),
        &[],
    );

    let db = test.path().join("campaign.sqlite");
    let pool = file_pool(&db).await;
    let runtime = CampaignRuntime::from_content_root(pool.clone(), &content);
    let expected = state("Alpha", 11, ("rules.alpha", "1"), &[("pack.alpha", "1")]);

    let created = runtime
        .create_campaign(&expected)
        .await
        .expect("resolved campaign should create");
    assert_eq!(created.state(), &expected);
    assert_eq!(created.content().ruleset.manifest.id, "rules.alpha");
    assert_eq!(created.content().content_packs.len(), 1);

    let opened = runtime
        .open_campaign(expected.campaign_id())
        .await
        .expect("resolved campaign should open");
    assert_eq!(opened.state(), &expected);
    let resumed = runtime
        .resume_campaign(expected.campaign_id())
        .await
        .expect("resume should share runnable resolution path");
    assert_eq!(resumed.state(), &expected);

    let export = export_campaign(&pool, expected.campaign_id())
        .await
        .expect("raw export should succeed");
    drop(runtime);
    drop(pool);

    let restarted_pool = file_pool(&db).await;
    let restarted = CampaignRuntime::from_content_root(restarted_pool.clone(), &content);
    assert_eq!(
        restarted
            .open_campaign(expected.campaign_id())
            .await
            .expect("restart should reopen through content resolution")
            .state(),
        &expected
    );
    drop(restarted);
    drop(restarted_pool);

    let restored_db = test.path().join("restored.sqlite");
    let restored_pool = file_pool(&restored_db).await;
    let restored_runtime = CampaignRuntime::from_content_root(restored_pool.clone(), &content);
    let restored = restored_runtime
        .restore_campaign(&export)
        .await
        .expect("resolved export should restore");
    assert_eq!(restored.state(), &expected);
    assert_eq!(
        raw_open_campaign(&restored_pool, expected.campaign_id())
            .await
            .expect("raw open should work")
            .state,
        expected
    );
}

#[tokio::test]
async fn unresolved_create_fails_before_persistence() {
    let test = TestDir::new("create-preflight");
    let content = test.path().join("content");
    fs::create_dir_all(&content).expect("content root should exist");
    let pool = file_pool(&test.path().join("campaign.sqlite")).await;
    let runtime = CampaignRuntime::from_content_root(pool.clone(), &content);
    let campaign = state("Missing", 22, ("rules.missing", "1"), &[]);

    let error = runtime
        .create_campaign(&campaign)
        .await
        .expect_err("missing ruleset must reject runnable create");
    assert!(matches!(
        error,
        RunnableCampaignError::Content(ContentResolutionError::MissingManifest { .. })
    ));
    assert!(matches!(
        raw_open_campaign(&pool, campaign.campaign_id()).await,
        Err(LifecycleError::CampaignNotFound)
    ));
}

#[tokio::test]
async fn wrong_version_after_persistence_blocks_runnable_open_but_not_raw_recovery() {
    let test = TestDir::new("wrong-version");
    let content = test.path().join("content");
    fs::create_dir_all(&content).expect("content root should exist");
    let installed_v1 = write_ruleset(&content, "rules-v1", "rules.alpha", "1");
    let pool = file_pool(&test.path().join("campaign.sqlite")).await;
    let runtime = CampaignRuntime::from_content_root(pool.clone(), &content);
    let campaign = state("Pinned", 33, ("rules.alpha", "1"), &[]);
    runtime
        .create_campaign(&campaign)
        .await
        .expect("initial exact content should create");

    fs::remove_dir_all(installed_v1).expect("old ruleset should be removed");
    write_ruleset(&content, "rules-v2", "rules.alpha", "2");

    let error = runtime
        .open_campaign(campaign.campaign_id())
        .await
        .expect_err("installed newer version must not substitute");
    assert!(matches!(
        error,
        RunnableCampaignError::Content(ContentResolutionError::VersionUnavailable { .. })
    ));
    assert_eq!(
        raw_open_campaign(&pool, campaign.campaign_id())
            .await
            .expect("raw recovery read should remain available")
            .state,
        campaign
    );
}

#[tokio::test]
async fn incompatible_pack_and_missing_dependency_fail_explicitly() {
    let test = TestDir::new("pack-errors");
    let content = test.path().join("content");
    fs::create_dir_all(&content).expect("content root should exist");
    write_ruleset(&content, "rules-a", "rules.alpha", "1");
    write_ruleset(&content, "rules-b", "rules.beta", "1");
    write_pack(
        &content,
        "incompatible",
        "pack.incompatible",
        "1",
        ("rules.beta", "1"),
        &[],
    );
    write_pack(
        &content,
        "dependent",
        "pack.dependent",
        "1",
        ("rules.alpha", "1"),
        &[("pack.required", "1")],
    );
    write_pack(
        &content,
        "required",
        "pack.required",
        "1",
        ("rules.alpha", "1"),
        &[],
    );

    let pool = file_pool(&test.path().join("campaign.sqlite")).await;
    let runtime = CampaignRuntime::from_content_root(pool, &content);

    let incompatible = state(
        "Incompatible",
        44,
        ("rules.alpha", "1"),
        &[("pack.incompatible", "1")],
    );
    assert!(matches!(
        runtime
            .create_campaign(&incompatible)
            .await
            .expect_err("incompatible pack must fail"),
        RunnableCampaignError::Content(ContentResolutionError::IncompatibleRuleset { .. })
    ));

    let missing_dependency = state(
        "Missing dependency",
        45,
        ("rules.alpha", "1"),
        &[("pack.dependent", "1")],
    );
    assert!(matches!(
        runtime
            .create_campaign(&missing_dependency)
            .await
            .expect_err("missing dependency must fail"),
        RunnableCampaignError::Content(ContentResolutionError::MissingDependency { .. })
    ));
}

#[tokio::test]
async fn corrupt_local_content_is_a_catalog_failure_while_raw_state_remains_available() {
    let test = TestDir::new("corrupt-content");
    let content = test.path().join("content");
    fs::create_dir_all(&content).expect("content root should exist");
    let rules_dir = write_ruleset_with_file(
        &content,
        "rules",
        "rules.alpha",
        "1",
        Some(("rules.dat", b"known-good-rules")),
    );
    let pool = file_pool(&test.path().join("campaign.sqlite")).await;
    let runtime = CampaignRuntime::from_content_root(pool.clone(), &content);
    let campaign = state("Corruptible", 55, ("rules.alpha", "1"), &[]);
    runtime
        .create_campaign(&campaign)
        .await
        .expect("valid declared file should create");

    fs::write(rules_dir.join("rules.dat"), b"changed-rules")
        .expect("declared file should be corrupted");
    let error = runtime
        .open_campaign(campaign.campaign_id())
        .await
        .expect_err("checksum mismatch must block runnable open");
    assert!(matches!(
        error,
        RunnableCampaignError::Catalog(CatalogLoadError::ContentFileLengthMismatch { .. })
            | RunnableCampaignError::Catalog(CatalogLoadError::ContentFileChecksumMismatch { .. })
    ));
    assert_eq!(
        raw_open_campaign(&pool, campaign.campaign_id())
            .await
            .expect("raw recovery should still read state")
            .state,
        campaign
    );
}

#[tokio::test]
async fn restore_content_preflight_fails_before_any_target_rows_commit() {
    let test = TestDir::new("restore-preflight");
    let source_content = test.path().join("source-content");
    fs::create_dir_all(&source_content).expect("content root should exist");
    write_ruleset(&source_content, "rules", "rules.alpha", "1");
    let source_pool = file_pool(&test.path().join("source.sqlite")).await;
    let source_runtime = CampaignRuntime::from_content_root(source_pool.clone(), &source_content);
    let campaign = state("Portable", 66, ("rules.alpha", "1"), &[]);
    source_runtime
        .create_campaign(&campaign)
        .await
        .expect("source should create");
    let export = export_campaign(&source_pool, campaign.campaign_id())
        .await
        .expect("source should export");

    let empty_content = test.path().join("empty-content");
    fs::create_dir_all(&empty_content).expect("empty content root should exist");
    let target_pool = file_pool(&test.path().join("target.sqlite")).await;
    let target_runtime = CampaignRuntime::from_content_root(target_pool.clone(), &empty_content);
    assert!(matches!(
        target_runtime
            .restore_campaign(&export)
            .await
            .expect_err("unresolved restore must fail before persistence mutation"),
        RunnableCampaignError::Content(ContentResolutionError::MissingManifest { .. })
    ));
    assert!(matches!(
        raw_open_campaign(&target_pool, campaign.campaign_id()).await,
        Err(LifecycleError::CampaignNotFound)
    ));
}

#[tokio::test]
async fn one_campaigns_missing_content_does_not_make_an_unrelated_campaign_unrunnable() {
    let test = TestDir::new("isolation");
    let content = test.path().join("content");
    fs::create_dir_all(&content).expect("content root should exist");
    let alpha_dir = write_ruleset(&content, "rules-alpha", "rules.alpha", "1");
    write_ruleset(&content, "rules-beta", "rules.beta", "1");
    let pool = file_pool(&test.path().join("campaign.sqlite")).await;
    let runtime = CampaignRuntime::from_content_root(pool.clone(), &content);
    let alpha = state("Alpha", 77, ("rules.alpha", "1"), &[]);
    let beta = state("Beta", 88, ("rules.beta", "1"), &[]);
    runtime
        .create_campaign(&alpha)
        .await
        .expect("alpha should create");
    runtime
        .create_campaign(&beta)
        .await
        .expect("beta should create");

    fs::remove_dir_all(alpha_dir).expect("alpha content should be removed");
    assert!(matches!(
        runtime
            .open_campaign(alpha.campaign_id())
            .await
            .expect_err("alpha should no longer be runnable"),
        RunnableCampaignError::Content(ContentResolutionError::MissingManifest { .. })
    ));
    assert_eq!(
        runtime
            .open_campaign(beta.campaign_id())
            .await
            .expect("beta should remain runnable")
            .state(),
        &beta
    );
    assert_eq!(
        raw_open_campaign(&pool, alpha.campaign_id())
            .await
            .expect("raw alpha recovery should remain available")
            .state,
        alpha
    );
}

#[tokio::test]
async fn malformed_unrelated_manifest_fails_catalog_load_explicitly() {
    let test = TestDir::new("catalog-load");
    let content = test.path().join("content");
    fs::create_dir_all(&content).expect("content root should exist");
    write_ruleset(&content, "rules", "rules.alpha", "1");
    let pool = file_pool(&test.path().join("campaign.sqlite")).await;
    let runtime = CampaignRuntime::from_content_root(pool.clone(), &content);
    let campaign = state("Catalog", 99, ("rules.alpha", "1"), &[]);
    runtime
        .create_campaign(&campaign)
        .await
        .expect("initial catalog should load");

    let malformed = pack_dir(&content, "malformed");
    fs::write(malformed.join("manifest.json"), "{not-json")
        .expect("malformed manifest should be written");
    assert!(matches!(
        runtime
            .open_campaign(campaign.campaign_id())
            .await
            .expect_err("catalog construction failure must be explicit"),
        RunnableCampaignError::Catalog(CatalogLoadError::MalformedManifest { .. })
    ));

    // Raw persistence remains a deliberate diagnostic/recovery route even when catalog loading is
    // impossible. It is not a runnable-campaign return type.
    let raw = raw_open_campaign(&pool, campaign.campaign_id())
        .await
        .expect("raw read should not load content");
    assert_eq!(raw.state, campaign);

    // Keep the purge API reachable only as raw administration in this integration test, proving
    // the application wrapper did not replace persistence recovery/administration surfaces.
    fs::remove_dir_all(malformed).expect("malformed fixture should be removed");
    let backup = export_campaign(&pool, campaign.campaign_id())
        .await
        .expect("raw export should remain valid");
    purge_campaign(
        &pool,
        campaign.campaign_id(),
        &CampaignPurgeAuthorization::admin("test cleanup through raw administration"),
        &backup,
    )
    .await
    .expect("raw administrative purge should remain available");
}

#[tokio::test]
async fn table_observation_lineage_cannot_be_hidden_by_generic_state_images() {
    use dmd_domain::{
        CommandIssuer, NewSessionObservation, ObservationAudience, ObservationId,
        SessionObservation,
    };
    let test = TestDir::new("table-lineage");
    let content = test.path().join("content");
    fs::create_dir_all(&content).unwrap();
    write_ruleset(&content, "rules", "rules.generic", "1");
    let pool = file_pool(&test.path().join("source.sqlite")).await;
    let source = CampaignRuntime::from_content_root(pool.clone(), &content);
    let campaign = state("Generic image", 22, ("rules.generic", "1"), &[]);
    source.create_campaign(&campaign).await.unwrap();
    let mut export = export_campaign(&pool, campaign.campaign_id())
        .await
        .unwrap();
    export.observations.push(SessionObservation {
        ordinal: 1,
        record: NewSessionObservation {
            id: ObservationId::new(),
            campaign_id: campaign.campaign_id(),
            session_id: None,
            issuer: CommandIssuer::Admin,
            audience: ObservationAudience::Host,
            observed_event_sequence: 0,
            kind: "table.conversation".into(),
            payload_schema_version: 1,
            payload_json: "{}".into(),
        },
    });
    let destination_pool = file_pool(&test.path().join("destination.sqlite")).await;
    let destination = CampaignRuntime::from_content_root(destination_pool.clone(), content);
    assert!(
        matches!(destination.restore_campaign(&export).await,Err(RunnableCampaignError::RulesContent(message)) if message.contains("no implemented kernel"))
    );
    assert!(
        dmd_persistence::list_campaigns(&destination_pool)
            .await
            .unwrap()
            .is_empty()
    );
}
