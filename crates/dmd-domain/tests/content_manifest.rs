use std::{env, fs, path::{Path, PathBuf}};

use dmd_domain::{
    CONTENT_MANIFEST_FILENAME, CURRENT_CONTENT_CONTRACT_VERSION,
    CURRENT_CONTENT_MANIFEST_SCHEMA_VERSION, Campaign, CampaignId, CampaignStatus,
    CatalogLoadError, ChecksumAlgorithm, ContentCatalog, ContentChecksum, ContentManifest,
    ContentResolutionError, ManifestFile, ManifestKind, ManifestViolation, VersionedRef,
    fnv1a64_hex,
};
use uuid::Uuid;

struct TempContentRoot(PathBuf);

impl TempContentRoot {
    fn new() -> Self {
        let path = env::temp_dir().join(format!("dmd-content-manifest-{}", Uuid::new_v4()));
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
        display_name: "Generic test campaign".into(),
        status: CampaignStatus::Active,
        world_seed: 7,
        ruleset,
        content_packs,
    }
}

fn install(root: &Path, directory: &str, manifest: &ContentManifest) -> PathBuf {
    let directory = root.join(directory);
    fs::create_dir_all(&directory).expect("manifest directory should be created");
    let path = directory.join(CONTENT_MANIFEST_FILENAME);
    fs::write(
        &path,
        manifest
            .encode_json_pretty()
            .expect("manifest should encode"),
    )
    .expect("manifest should be written");
    path
}

#[test]
fn unrelated_rulesets_and_content_packs_resolve_without_hardcoded_campaign_assumptions() {
    let root = TempContentRoot::new();
    install(root.path(), "rules-alpha", &ruleset("rules.alpha", "1.0.0"));
    install(root.path(), "rules-beta", &ruleset("rules.beta", "9.4"));
    install(
        root.path(),
        "forest-pack",
        &pack("world.forest", "2", reference("rules.alpha", "1.0.0")),
    );
    install(
        root.path(),
        "space-pack",
        &pack("world.space", "3", reference("rules.beta", "9.4")),
    );
    let catalog = ContentCatalog::load_from_roots(&[root.path().to_path_buf()])
        .expect("catalog should load");

    let alpha = campaign(
        reference("rules.alpha", "1.0.0"),
        vec![reference("world.forest", "2")],
    );
    let beta = campaign(
        reference("rules.beta", "9.4"),
        vec![reference("world.space", "3")],
    );

    assert_eq!(
        catalog
            .resolve_campaign(&alpha)
            .unwrap()
            .content_packs
            .len(),
        1
    );
    assert_eq!(
        catalog.resolve_campaign(&beta).unwrap().content_packs.len(),
        1
    );
    assert!(matches!(
        catalog.resolve_campaign(&campaign(
            reference("rules.alpha", "1.0.0"),
            vec![reference("world.space", "3")]
        )),
        Err(ContentResolutionError::IncompatibleRuleset { .. })
    ));
}

#[test]
fn exact_version_is_required_and_never_substituted() {
    let root = TempContentRoot::new();
    install(root.path(), "rules", &ruleset("rules.generic", "1.0"));
    let catalog = ContentCatalog::load_from_roots(&[root.path().to_path_buf()]).unwrap();
    let error = catalog
        .resolve_campaign(&campaign(reference("rules.generic", "2.0"), Vec::new()))
        .unwrap_err();

    assert_eq!(
        error,
        ContentResolutionError::VersionUnavailable {
            kind: ManifestKind::Ruleset,
            id: "rules.generic".into(),
            requested: "2.0".into(),
            available: vec!["1.0".into()],
        }
    );
}

#[test]
fn missing_manifest_is_explicit() {
    let root = TempContentRoot::new();
    let catalog = ContentCatalog::load_from_roots(&[root.path().to_path_buf()]).unwrap();
    assert_eq!(
        catalog
            .resolve_campaign(&campaign(reference("rules.missing", "1"), Vec::new()))
            .unwrap_err(),
        ContentResolutionError::MissingManifest {
            kind: ManifestKind::Ruleset,
            id: "rules.missing".into(),
        }
    );
}

#[test]
fn kind_mismatch_is_explicit() {
    let root = TempContentRoot::new();
    install(
        root.path(),
        "pack",
        &pack("shared.id", "1", reference("rules.any", "1")),
    );
    let catalog = ContentCatalog::load_from_roots(&[root.path().to_path_buf()]).unwrap();

    assert!(matches!(
        catalog.resolve_campaign(&campaign(reference("shared.id", "1"), Vec::new())),
        Err(ContentResolutionError::KindMismatch {
            expected: ManifestKind::Ruleset,
            actual: ManifestKind::ContentPack,
            ..
        })
    ));
}

#[test]
fn duplicate_manifest_identity_fails_catalog_load() {
    let root = TempContentRoot::new();
    let manifest = ruleset("rules.same", "1");
    install(root.path(), "first", &manifest);
    install(root.path(), "second", &manifest);

    assert!(matches!(
        ContentCatalog::load_from_roots(&[root.path().to_path_buf()]),
        Err(CatalogLoadError::DuplicateIdentity { .. })
    ));
}

#[test]
fn incompatible_engine_content_contract_fails_catalog_load() {
    let root = TempContentRoot::new();
    let mut manifest = ruleset("rules.future", "1");
    manifest.content_contract_version = CURRENT_CONTENT_CONTRACT_VERSION + 1;
    install(root.path(), "future", &manifest);

    assert!(matches!(
        ContentCatalog::load_from_roots(&[root.path().to_path_buf()]),
        Err(CatalogLoadError::InvalidManifest { violations, .. })
            if violations.iter().any(|violation| matches!(
                violation,
                ManifestViolation::IncompatibleContentContract { .. }
            ))
    ));
}

#[test]
fn malformed_manifest_fails_catalog_load() {
    let root = TempContentRoot::new();
    let directory = root.path().join("bad");
    fs::create_dir_all(&directory).unwrap();
    fs::write(
        directory.join(CONTENT_MANIFEST_FILENAME),
        "{ definitely-not-json",
    )
    .unwrap();

    assert!(matches!(
        ContentCatalog::load_from_roots(&[root.path().to_path_buf()]),
        Err(CatalogLoadError::MalformedManifest { .. })
    ));
}

#[test]
fn corrupted_declared_file_fails_catalog_load() {
    let root = TempContentRoot::new();
    let directory = root.path().join("pack");
    fs::create_dir_all(&directory).unwrap();
    let content_path = directory.join("data.bin");
    fs::write(&content_path, b"expected bytes").unwrap();

    let mut manifest = pack("world.integrity", "1", reference("rules.generic", "1"));
    manifest.files.push(ManifestFile {
        path: "data.bin".into(),
        byte_len: 14,
        checksum: ContentChecksum {
            algorithm: ChecksumAlgorithm::Fnv1a64,
            value: fnv1a64_hex(b"expected bytes"),
        },
    });
    install(root.path(), "pack", &manifest);
    fs::write(&content_path, b"changed bytes!").unwrap();

    assert!(matches!(
        ContentCatalog::load_from_roots(&[root.path().to_path_buf()]),
        Err(CatalogLoadError::ContentFileChecksumMismatch { .. })
            | Err(CatalogLoadError::ContentFileLengthMismatch { .. })
    ));
}

#[test]
fn content_pack_dependency_must_be_exactly_present_in_campaign() {
    let root = TempContentRoot::new();
    let rules = reference("rules.generic", "1");
    install(root.path(), "rules", &ruleset(&rules.id, &rules.version));
    install(root.path(), "base", &pack("pack.base", "1", rules.clone()));
    let mut addon = pack("pack.addon", "1", rules.clone());
    addon.dependencies.push(reference("pack.base", "1"));
    install(root.path(), "addon", &addon);
    let catalog = ContentCatalog::load_from_roots(&[root.path().to_path_buf()]).unwrap();

    assert!(matches!(
        catalog.resolve_campaign(&campaign(rules.clone(), vec![reference("pack.addon", "1")])),
        Err(ContentResolutionError::MissingDependency { .. })
    ));
    assert!(
        catalog
            .resolve_campaign(&campaign(
                rules,
                vec![reference("pack.base", "1"), reference("pack.addon", "1")]
            ))
            .is_ok()
    );
}

#[test]
fn unsafe_declared_paths_are_rejected_before_file_access() {
    let manifest = ContentManifest {
        files: vec![ManifestFile {
            path: "../outside.bin".into(),
            byte_len: 0,
            checksum: ContentChecksum {
                algorithm: ChecksumAlgorithm::Fnv1a64,
                value: fnv1a64_hex(&[]),
            },
        }],
        ..pack("pack.safe", "1", reference("rules.generic", "1"))
    };

    assert!(
        manifest
            .validate()
            .contains(&ManifestViolation::UnsafeFilePath("../outside.bin".into()))
    );
}
