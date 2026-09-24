#[cfg(unix)]
use std::{
    env, fs,
    path::{Path, PathBuf},
};

use dmd_domain::ContentManifest;
#[cfg(unix)]
use dmd_domain::{
    CONTENT_MANIFEST_FILENAME, CURRENT_CONTENT_CONTRACT_VERSION,
    CURRENT_CONTENT_MANIFEST_SCHEMA_VERSION, CatalogLoadError, ChecksumAlgorithm, ContentCatalog,
    ContentChecksum, ManifestFile, ManifestKind, VersionedRef, fnv1a64_hex,
};
#[cfg(unix)]
use uuid::Uuid;

#[cfg(unix)]
struct TempContentRoot(PathBuf);

#[cfg(unix)]
impl TempContentRoot {
    fn new() -> Self {
        let path = env::temp_dir().join(format!("dmd-content-fail-closed-{}", Uuid::new_v4()));
        fs::create_dir_all(&path).expect("temp content root should be created");
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

#[cfg(unix)]
impl Drop for TempContentRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[cfg(unix)]
fn reference(id: &str, version: &str) -> VersionedRef {
    VersionedRef {
        id: id.into(),
        version: version.into(),
    }
}

#[cfg(unix)]
fn pack_with_file(path: &str, bytes: &[u8]) -> ContentManifest {
    ContentManifest {
        manifest_schema_version: CURRENT_CONTENT_MANIFEST_SCHEMA_VERSION,
        content_contract_version: CURRENT_CONTENT_CONTRACT_VERSION,
        kind: ManifestKind::ContentPack,
        id: "pack.fail-closed".into(),
        version: "1".into(),
        compatible_rulesets: vec![reference("rules.generic", "1")],
        dependencies: Vec::new(),
        files: vec![ManifestFile {
            path: path.into(),
            byte_len: bytes.len() as u64,
            checksum: ContentChecksum {
                algorithm: ChecksumAlgorithm::Fnv1a64,
                value: fnv1a64_hex(bytes),
            },
        }],
    }
}

#[test]
fn unknown_manifest_fields_fail_closed_at_schema_boundaries() {
    let cases = [
        r#"{
            "manifest_schema_version":1,
            "content_contract_version":1,
            "kind":"content_pack",
            "id":"pack.generic",
            "version":"1",
            "compatible_rulesets":[{"id":"rules.generic","version":"1"}],
            "dependecies":[]
        }"#,
        r#"{
            "manifest_schema_version":1,
            "content_contract_version":1,
            "kind":"content_pack",
            "id":"pack.generic",
            "version":"1",
            "compatible_rulesets":[{"id":"rules.generic","version":"1","versoin":"2"}]
        }"#,
        r#"{
            "manifest_schema_version":1,
            "content_contract_version":1,
            "kind":"content_pack",
            "id":"pack.generic",
            "version":"1",
            "compatible_rulesets":[{"id":"rules.generic","version":"1"}],
            "files":[{
                "path":"data.bin",
                "byte_len":0,
                "byte_lenght":0,
                "checksum":{"algorithm":"fnv1a64","value":"cbf29ce484222325"}
            }]
        }"#,
        r#"{
            "manifest_schema_version":1,
            "content_contract_version":1,
            "kind":"content_pack",
            "id":"pack.generic",
            "version":"1",
            "compatible_rulesets":[{"id":"rules.generic","version":"1"}],
            "files":[{
                "path":"data.bin",
                "byte_len":0,
                "checksum":{
                    "algorithm":"fnv1a64",
                    "value":"cbf29ce484222325",
                    "algoritm":"fnv1a64"
                }
            }]
        }"#,
    ];

    for json in cases {
        assert!(
            ContentManifest::decode_json(json).is_err(),
            "unknown manifest field should be rejected: {json}"
        );
    }
}

#[cfg(unix)]
#[test]
fn intermediate_symlink_cannot_escape_manifest_directory() {
    use std::os::unix::fs::symlink;

    let root = TempContentRoot::new();
    let pack_dir = root.path().join("pack");
    let outside_dir = root.path().join("outside");
    fs::create_dir_all(&pack_dir).unwrap();
    fs::create_dir_all(&outside_dir).unwrap();

    let bytes = b"outside content";
    fs::write(outside_dir.join("rules.json"), bytes).unwrap();
    symlink(&outside_dir, pack_dir.join("data")).unwrap();

    let manifest = pack_with_file("data/rules.json", bytes);
    let manifest_path = pack_dir.join(CONTENT_MANIFEST_FILENAME);
    fs::write(&manifest_path, manifest.encode_json_pretty().unwrap()).unwrap();

    assert!(matches!(
        ContentCatalog::load_from_roots(&[manifest_path]),
        Err(CatalogLoadError::SymlinkNotAllowed(path))
            if path == pack_dir.join("data")
    ));
}
