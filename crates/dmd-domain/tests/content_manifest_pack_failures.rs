use std::{
    env, fs,
    path::{Path, PathBuf},
};

use dmd_domain::{
    CONTENT_MANIFEST_FILENAME, CURRENT_CONTENT_CONTRACT_VERSION,
    CURRENT_CONTENT_MANIFEST_SCHEMA_VERSION, Campaign, CampaignId, CampaignStatus,
    CatalogLoadError, ChecksumAlgorithm, ContentCatalog, ContentChecksum, ContentManifest,
    ContentResolutionError, ManifestFile, ManifestKind, VersionedRef, fnv1a64_hex,
};
use uuid::Uuid;

struct TempContentRoot(PathBuf);

impl TempContentRoot {
    fn new() -> Self {
        let path = env::temp_dir().join(format!("dmd-content-pack-failures-{}", Uuid::new_v4()));
        fs::create_dir_all(&path).expect("temp content root should be created");
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempContentRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn reference(id: &str, version: &str) -> VersionedRef {
    VersionedRef {
        id: id.into(),
        version: version.into(),
    }
}

fn ruleset(id: &str, version: &str) -> ContentManifest {
    ContentManifest {
        manifest_schema_version: CURRENT_CONTENT_MANIFEST_SCHEMA_VERSION,
        content_contract_version: CURRENT_CONTENT_CONTRACT_VERSION,
        kind: ManifestKind::Ruleset,
        id: id.into(),
        version: version.into(),
        compatible_rulesets: Vec::new(),
        dependencies: Vec::new(),
        files: Vec::new(),
    }
}

fn pack(id: &str, version: &str, ruleset: VersionedRef) -> ContentManifest {
    ContentManifest {
        manifest_schema_version: CURRENT_CONTENT_MANIFEST_SCHEMA_VERSION,
        content_contract_version: CURRENT_CONTENT_CONTRACT_VERSION,
        kind: ManifestKind::ContentPack,
        id: id.into(),
        version: version.into(),
        compatible_rulesets: vec![ruleset],
        dependencies: Vec::new(),
        files: Vec::new(),
    }
}

fn campaign(ruleset: VersionedRef, content_packs: Vec<VersionedRef>) -> Campaign {
    Campaign {
        id: CampaignId::new(),
        display_name: "Generic pack failure campaign".into(),
        status: CampaignStatus::Active,
        world_seed: 19,
        ruleset,
        content_packs,
    }
}

fn install(root: &Path, directory: &str, manifest: &ContentManifest) {
    let directory = root.join(directory);
    fs::create_dir_all(&directory).expect("manifest directory should be created");
    fs::write(
        directory.join(CONTENT_MANIFEST_FILENAME),
        manifest
            .encode_json_pretty()
            .expect("manifest should encode"),
    )
    .expect("manifest should be written");
}

#[test]
fn missing_content_pack_is_explicit() {
    let root = TempContentRoot::new();
    let rules = reference("rules.generic", "1");
    install(root.path(), "rules", &ruleset(&rules.id, &rules.version));
    let catalog = ContentCatalog::load_from_roots(&[root.path().to_path_buf()]).unwrap();

    assert_eq!(
        catalog
            .resolve_campaign(&campaign(rules, vec![reference("pack.missing", "1")]))
            .unwrap_err(),
        ContentResolutionError::MissingManifest {
            kind: ManifestKind::ContentPack,
            id: "pack.missing".into(),
        }
    );
}

#[test]
fn wrong_content_pack_version_is_explicit() {
    let root = TempContentRoot::new();
    let rules = reference("rules.generic", "1");
    install(root.path(), "rules", &ruleset(&rules.id, &rules.version));
    install(
        root.path(),
        "pack",
        &pack("pack.generic", "1", rules.clone()),
    );
    let catalog = ContentCatalog::load_from_roots(&[root.path().to_path_buf()]).unwrap();

    assert_eq!(
        catalog
            .resolve_campaign(&campaign(rules, vec![reference("pack.generic", "2")]))
            .unwrap_err(),
        ContentResolutionError::VersionUnavailable {
            kind: ManifestKind::ContentPack,
            id: "pack.generic".into(),
            requested: "2".into(),
            available: vec!["1".into()],
        }
    );
}

#[test]
fn duplicate_content_pack_id_in_campaign_is_explicit() {
    let root = TempContentRoot::new();
    let rules = reference("rules.generic", "1");
    install(root.path(), "rules", &ruleset(&rules.id, &rules.version));
    install(
        root.path(),
        "pack-v1",
        &pack("pack.generic", "1", rules.clone()),
    );
    install(
        root.path(),
        "pack-v2",
        &pack("pack.generic", "2", rules.clone()),
    );
    let catalog = ContentCatalog::load_from_roots(&[root.path().to_path_buf()]).unwrap();

    assert_eq!(
        catalog
            .resolve_campaign(&campaign(
                rules,
                vec![reference("pack.generic", "1"), reference("pack.generic", "2")],
            ))
            .unwrap_err(),
        ContentResolutionError::DuplicateCampaignPackId("pack.generic".into())
    );
}

#[test]
fn missing_declared_content_file_fails_catalog_load() {
    let root = TempContentRoot::new();
    let mut manifest = pack("pack.files", "1", reference("rules.generic", "1"));
    manifest.files.push(ManifestFile {
        path: "missing.bin".into(),
        byte_len: 0,
        checksum: ContentChecksum {
            algorithm: ChecksumAlgorithm::Fnv1a64,
            value: fnv1a64_hex(&[]),
        },
    });
    install(root.path(), "pack", &manifest);

    assert!(matches!(
        ContentCatalog::load_from_roots(&[root.path().to_path_buf()]),
        Err(CatalogLoadError::MissingContentFile { .. })
    ));
}
