use std::{
    collections::{BTreeMap, BTreeSet, HashSet},
    error::Error,
    fmt, fs,
    path::{Component, Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{Campaign, VersionedRef};

pub const CURRENT_CONTENT_MANIFEST_SCHEMA_VERSION: u32 = 1;
pub const CURRENT_CONTENT_CONTRACT_VERSION: u32 = 1;
pub const CONTENT_MANIFEST_FILENAME: &str = "manifest.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ManifestKind {
    Ruleset,
    ContentPack,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ManifestIdentity {
    pub kind: ManifestKind,
    pub id: String,
    pub version: String,
}

impl ManifestIdentity {
    #[must_use]
    pub fn new(kind: ManifestKind, id: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            kind,
            id: id.into(),
            version: version.into(),
        }
    }
}

impl fmt::Display for ManifestIdentity {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{:?}:{}@{}", self.kind, self.id, self.version)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChecksumAlgorithm {
    Fnv1a64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContentChecksum {
    pub algorithm: ChecksumAlgorithm,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManifestFile {
    pub path: String,
    pub byte_len: u64,
    pub checksum: ContentChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContentManifest {
    pub manifest_schema_version: u32,
    pub content_contract_version: u32,
    pub kind: ManifestKind,
    pub id: String,
    pub version: String,
    #[serde(default)]
    pub compatible_rulesets: Vec<VersionedRef>,
    #[serde(default)]
    pub dependencies: Vec<VersionedRef>,
    #[serde(default)]
    pub files: Vec<ManifestFile>,
}

impl ContentManifest {
    #[must_use]
    pub fn identity(&self) -> ManifestIdentity {
        ManifestIdentity::new(self.kind, self.id.clone(), self.version.clone())
    }

    pub fn decode_json(value: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(value)
    }

    pub fn encode_json_pretty(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    #[must_use]
    pub fn validate(&self) -> Vec<ManifestViolation> {
        let mut violations = Vec::new();

        if self.manifest_schema_version != CURRENT_CONTENT_MANIFEST_SCHEMA_VERSION {
            violations.push(ManifestViolation::UnsupportedManifestSchema {
                actual: self.manifest_schema_version,
                supported: CURRENT_CONTENT_MANIFEST_SCHEMA_VERSION,
            });
        }
        if self.content_contract_version != CURRENT_CONTENT_CONTRACT_VERSION {
            violations.push(ManifestViolation::IncompatibleContentContract {
                actual: self.content_contract_version,
                supported: CURRENT_CONTENT_CONTRACT_VERSION,
            });
        }
        if !valid_identity_token(&self.id) {
            violations.push(ManifestViolation::InvalidId(self.id.clone()));
        }
        if !valid_identity_token(&self.version) {
            violations.push(ManifestViolation::InvalidVersion(self.version.clone()));
        }

        match self.kind {
            ManifestKind::Ruleset => {
                if !self.compatible_rulesets.is_empty() {
                    violations.push(ManifestViolation::RulesetDeclaresCompatibleRulesets);
                }
                if !self.dependencies.is_empty() {
                    violations.push(ManifestViolation::RulesetDeclaresPackDependencies);
                }
            }
            ManifestKind::ContentPack => {
                if self.compatible_rulesets.is_empty() {
                    violations.push(ManifestViolation::ContentPackMissingRulesetCompatibility);
                }
            }
        }

        validate_versioned_refs(
            "compatible_rulesets",
            &self.compatible_rulesets,
            &mut violations,
        );
        validate_versioned_refs("dependencies", &self.dependencies, &mut violations);

        let mut file_paths = HashSet::new();
        for file in &self.files {
            if !safe_relative_path(&file.path) {
                violations.push(ManifestViolation::UnsafeFilePath(file.path.clone()));
            }
            if !file_paths.insert(file.path.clone()) {
                violations.push(ManifestViolation::DuplicateFilePath(file.path.clone()));
            }
            match file.checksum.algorithm {
                ChecksumAlgorithm::Fnv1a64 => {
                    if !valid_fnv1a64(&file.checksum.value) {
                        violations.push(ManifestViolation::InvalidChecksum {
                            path: file.path.clone(),
                            algorithm: file.checksum.algorithm,
                        });
                    }
                }
            }
        }

        violations
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManifestViolation {
    UnsupportedManifestSchema {
        actual: u32,
        supported: u32,
    },
    IncompatibleContentContract {
        actual: u32,
        supported: u32,
    },
    InvalidId(String),
    InvalidVersion(String),
    RulesetDeclaresCompatibleRulesets,
    RulesetDeclaresPackDependencies,
    ContentPackMissingRulesetCompatibility,
    InvalidReference {
        field: &'static str,
        id: String,
        version: String,
    },
    DuplicateReference {
        field: &'static str,
        id: String,
        version: String,
    },
    UnsafeFilePath(String),
    DuplicateFilePath(String),
    InvalidChecksum {
        path: String,
        algorithm: ChecksumAlgorithm,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CatalogLoadError {
    Io {
        path: PathBuf,
        message: String,
    },
    SymlinkNotAllowed(PathBuf),
    MalformedManifest {
        path: PathBuf,
        message: String,
    },
    InvalidManifest {
        path: PathBuf,
        violations: Vec<ManifestViolation>,
    },
    DuplicateIdentity {
        identity: ManifestIdentity,
        first_path: PathBuf,
        second_path: PathBuf,
    },
    MissingContentFile {
        identity: ManifestIdentity,
        path: PathBuf,
    },
    NonRegularContentFile {
        identity: ManifestIdentity,
        path: PathBuf,
    },
    ContentFileLengthMismatch {
        identity: ManifestIdentity,
        path: PathBuf,
        expected: u64,
        actual: u64,
    },
    ContentFileChecksumMismatch {
        identity: ManifestIdentity,
        path: PathBuf,
        expected: String,
        actual: String,
    },
}

impl fmt::Display for CatalogLoadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, message } => {
                write!(formatter, "I/O error at {}: {message}", path.display())
            }
            Self::SymlinkNotAllowed(path) => {
                write!(
                    formatter,
                    "content discovery does not follow symlink {}",
                    path.display()
                )
            }
            Self::MalformedManifest { path, message } => {
                write!(
                    formatter,
                    "malformed content manifest {}: {message}",
                    path.display()
                )
            }
            Self::InvalidManifest { path, violations } => write!(
                formatter,
                "invalid content manifest {}: {violations:?}",
                path.display()
            ),
            Self::DuplicateIdentity {
                identity,
                first_path,
                second_path,
            } => write!(
                formatter,
                "duplicate content manifest {identity} at {} and {}",
                first_path.display(),
                second_path.display()
            ),
            Self::MissingContentFile { identity, path } => {
                write!(
                    formatter,
                    "{identity} is missing declared file {}",
                    path.display()
                )
            }
            Self::NonRegularContentFile { identity, path } => write!(
                formatter,
                "{identity} declared non-regular file {}",
                path.display()
            ),
            Self::ContentFileLengthMismatch {
                identity,
                path,
                expected,
                actual,
            } => write!(
                formatter,
                "{identity} file {} length mismatch: expected {expected}, got {actual}",
                path.display()
            ),
            Self::ContentFileChecksumMismatch {
                identity,
                path,
                expected,
                actual,
            } => write!(
                formatter,
                "{identity} file {} checksum mismatch: expected {expected}, got {actual}",
                path.display()
            ),
        }
    }
}

impl Error for CatalogLoadError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContentResolutionError {
    MissingManifest {
        kind: ManifestKind,
        id: String,
    },
    VersionUnavailable {
        kind: ManifestKind,
        id: String,
        requested: String,
        available: Vec<String>,
    },
    KindMismatch {
        expected: ManifestKind,
        id: String,
        version: String,
        actual: ManifestKind,
    },
    DuplicateCampaignPackId(String),
    IncompatibleRuleset {
        pack: ManifestIdentity,
        ruleset: VersionedRef,
    },
    MissingDependency {
        pack: ManifestIdentity,
        dependency: VersionedRef,
    },
}

impl fmt::Display for ContentResolutionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingManifest { kind, id } => {
                write!(formatter, "no installed {kind:?} manifest with id {id}")
            }
            Self::VersionUnavailable {
                kind,
                id,
                requested,
                available,
            } => write!(
                formatter,
                "installed {kind:?} {id} does not provide exact version {requested}; available: {available:?}"
            ),
            Self::KindMismatch {
                expected,
                id,
                version,
                actual,
            } => write!(
                formatter,
                "manifest {id}@{version} has kind {actual:?}, expected {expected:?}"
            ),
            Self::DuplicateCampaignPackId(id) => {
                write!(
                    formatter,
                    "campaign references content pack id {id} more than once"
                )
            }
            Self::IncompatibleRuleset { pack, ruleset } => write!(
                formatter,
                "content pack {pack} is not compatible with ruleset {}@{}",
                ruleset.id, ruleset.version
            ),
            Self::MissingDependency { pack, dependency } => write!(
                formatter,
                "content pack {pack} requires campaign content pack {}@{}",
                dependency.id, dependency.version
            ),
        }
    }
}

impl Error for ContentResolutionError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstalledManifest {
    pub manifest: ContentManifest,
    pub manifest_path: PathBuf,
}

#[derive(Debug, Clone, Default)]
pub struct ContentCatalog {
    manifests: BTreeMap<ManifestIdentity, InstalledManifest>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedCampaignContent {
    pub ruleset: InstalledManifest,
    pub content_packs: Vec<InstalledManifest>,
}

impl ContentCatalog {
    pub fn load_from_roots(roots: &[PathBuf]) -> Result<Self, CatalogLoadError> {
        let mut catalog = Self::default();
        let mut manifest_paths = Vec::new();

        for root in roots {
            collect_manifest_paths(root, &mut manifest_paths)?;
        }
        manifest_paths.sort();

        for path in manifest_paths {
            let json = fs::read_to_string(&path).map_err(|error| io_error(&path, error))?;
            let manifest = ContentManifest::decode_json(&json).map_err(|error| {
                CatalogLoadError::MalformedManifest {
                    path: path.clone(),
                    message: error.to_string(),
                }
            })?;
            let violations = manifest.validate();
            if !violations.is_empty() {
                return Err(CatalogLoadError::InvalidManifest { path, violations });
            }

            verify_manifest_files(&manifest, &path)?;
            let identity = manifest.identity();
            if let Some(existing) = catalog.manifests.get(&identity) {
                return Err(CatalogLoadError::DuplicateIdentity {
                    identity,
                    first_path: existing.manifest_path.clone(),
                    second_path: path,
                });
            }
            catalog.manifests.insert(
                identity,
                InstalledManifest {
                    manifest,
                    manifest_path: path,
                },
            );
        }

        Ok(catalog)
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.manifests.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.manifests.is_empty()
    }

    pub fn resolve_campaign(
        &self,
        campaign: &Campaign,
    ) -> Result<ResolvedCampaignContent, ContentResolutionError> {
        let ruleset = self.resolve_exact(ManifestKind::Ruleset, &campaign.ruleset)?;
        let mut seen_pack_ids = HashSet::new();
        let requested_packs: BTreeSet<_> = campaign
            .content_packs
            .iter()
            .map(|reference| (reference.id.clone(), reference.version.clone()))
            .collect();
        let mut content_packs = Vec::with_capacity(campaign.content_packs.len());

        for reference in &campaign.content_packs {
            if !seen_pack_ids.insert(reference.id.clone()) {
                return Err(ContentResolutionError::DuplicateCampaignPackId(
                    reference.id.clone(),
                ));
            }
            let installed = self.resolve_exact(ManifestKind::ContentPack, reference)?;
            let compatible = installed
                .manifest
                .compatible_rulesets
                .iter()
                .any(|candidate| candidate == &campaign.ruleset);
            if !compatible {
                return Err(ContentResolutionError::IncompatibleRuleset {
                    pack: installed.manifest.identity(),
                    ruleset: campaign.ruleset.clone(),
                });
            }
            for dependency in &installed.manifest.dependencies {
                if !requested_packs.contains(&(dependency.id.clone(), dependency.version.clone())) {
                    return Err(ContentResolutionError::MissingDependency {
                        pack: installed.manifest.identity(),
                        dependency: dependency.clone(),
                    });
                }
            }
            content_packs.push(installed.clone());
        }

        Ok(ResolvedCampaignContent {
            ruleset: ruleset.clone(),
            content_packs,
        })
    }

    fn resolve_exact(
        &self,
        kind: ManifestKind,
        reference: &VersionedRef,
    ) -> Result<&InstalledManifest, ContentResolutionError> {
        let identity = ManifestIdentity::new(kind, reference.id.clone(), reference.version.clone());
        if let Some(installed) = self.manifests.get(&identity) {
            return Ok(installed);
        }

        let other_kind = match kind {
            ManifestKind::Ruleset => ManifestKind::ContentPack,
            ManifestKind::ContentPack => ManifestKind::Ruleset,
        };
        let other_identity =
            ManifestIdentity::new(other_kind, reference.id.clone(), reference.version.clone());
        if self.manifests.contains_key(&other_identity) {
            return Err(ContentResolutionError::KindMismatch {
                expected: kind,
                id: reference.id.clone(),
                version: reference.version.clone(),
                actual: other_kind,
            });
        }

        let available: Vec<_> = self
            .manifests
            .keys()
            .filter(|candidate| candidate.kind == kind && candidate.id == reference.id)
            .map(|candidate| candidate.version.clone())
            .collect();
        if !available.is_empty() {
            return Err(ContentResolutionError::VersionUnavailable {
                kind,
                id: reference.id.clone(),
                requested: reference.version.clone(),
                available,
            });
        }

        Err(ContentResolutionError::MissingManifest {
            kind,
            id: reference.id.clone(),
        })
    }
}

fn validate_versioned_refs(
    field: &'static str,
    references: &[VersionedRef],
    violations: &mut Vec<ManifestViolation>,
) {
    let mut seen = HashSet::new();
    for reference in references {
        if !valid_identity_token(&reference.id) || !valid_identity_token(&reference.version) {
            violations.push(ManifestViolation::InvalidReference {
                field,
                id: reference.id.clone(),
                version: reference.version.clone(),
            });
        }
        if !seen.insert((reference.id.clone(), reference.version.clone())) {
            violations.push(ManifestViolation::DuplicateReference {
                field,
                id: reference.id.clone(),
                version: reference.version.clone(),
            });
        }
    }
}

fn valid_identity_token(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-' | b'+'))
}

fn safe_relative_path(value: &str) -> bool {
    if value.is_empty() {
        return false;
    }
    let path = Path::new(value);
    !path.is_absolute()
        && path
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
}

fn valid_fnv1a64(value: &str) -> bool {
    value.len() == 16
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn collect_manifest_paths(
    root: &Path,
    manifest_paths: &mut Vec<PathBuf>,
) -> Result<(), CatalogLoadError> {
    let metadata = fs::symlink_metadata(root).map_err(|error| io_error(root, error))?;
    if metadata.file_type().is_symlink() {
        return Err(CatalogLoadError::SymlinkNotAllowed(root.to_path_buf()));
    }
    if metadata.is_file() {
        if root.file_name().and_then(|name| name.to_str()) == Some(CONTENT_MANIFEST_FILENAME) {
            manifest_paths.push(root.to_path_buf());
        }
        return Ok(());
    }
    if !metadata.is_dir() {
        return Ok(());
    }

    let mut entries: Vec<_> = fs::read_dir(root)
        .map_err(|error| io_error(root, error))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| io_error(root, error))?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let path = entry.path();
        let file_type = entry.file_type().map_err(|error| io_error(&path, error))?;
        if file_type.is_symlink() {
            return Err(CatalogLoadError::SymlinkNotAllowed(path));
        }
        if file_type.is_dir() {
            collect_manifest_paths(&path, manifest_paths)?;
        } else if file_type.is_file()
            && path.file_name().and_then(|name| name.to_str()) == Some(CONTENT_MANIFEST_FILENAME)
        {
            manifest_paths.push(path);
        }
    }
    Ok(())
}

fn verify_manifest_files(
    manifest: &ContentManifest,
    manifest_path: &Path,
) -> Result<(), CatalogLoadError> {
    let identity = manifest.identity();
    let base = manifest_path.parent().unwrap_or_else(|| Path::new("."));
    for declared in &manifest.files {
        let path = base.join(&declared.path);
        let metadata = match fs::symlink_metadata(&path) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Err(CatalogLoadError::MissingContentFile {
                    identity: identity.clone(),
                    path,
                });
            }
            Err(error) => return Err(io_error(&path, error)),
        };
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            return Err(CatalogLoadError::NonRegularContentFile {
                identity: identity.clone(),
                path,
            });
        }
        if metadata.len() != declared.byte_len {
            return Err(CatalogLoadError::ContentFileLengthMismatch {
                identity: identity.clone(),
                path,
                expected: declared.byte_len,
                actual: metadata.len(),
            });
        }
        let bytes = fs::read(&path).map_err(|error| io_error(&path, error))?;
        let actual = fnv1a64_hex(&bytes);
        if actual != declared.checksum.value {
            return Err(CatalogLoadError::ContentFileChecksumMismatch {
                identity: identity.clone(),
                path,
                expected: declared.checksum.value.clone(),
                actual,
            });
        }
    }
    Ok(())
}

fn io_error(path: &Path, error: std::io::Error) -> CatalogLoadError {
    CatalogLoadError::Io {
        path: path.to_path_buf(),
        message: error.to_string(),
    }
}

#[must_use]
pub fn fnv1a64_hex(bytes: &[u8]) -> String {
    const OFFSET_BASIS: u64 = 0xcbf29ce484222325;
    const PRIME: u64 = 0x00000100000001b3;
    let mut hash = OFFSET_BASIS;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(PRIME);
    }
    format!("{hash:016x}")
}
