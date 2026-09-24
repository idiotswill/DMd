use std::path::{Path, PathBuf};

mod rules_restore;
mod rules_runtime;
pub use rules_runtime::*;
mod table_protocol;
pub use table_protocol::*;
mod table_engine;
mod table_runtime;

use dmd_domain::{
    CampaignId, CampaignState, CatalogLoadError, ContentCatalog, ContentResolutionError,
    ResolvedCampaignContent,
};
use dmd_persistence::{
    CampaignExport, CampaignLifecycleSummary, CampaignStateSnapshotCodec, LifecycleError,
    OpenCampaign, create_campaign, open_campaign, restore_campaign,
};
use dmd_rules::RulesPack;
use sqlx::SqlitePool;
use thiserror::Error;

/// Outermost application/runtime composition boundary for campaigns that are safe to play.
///
/// Raw `dmd-persistence` lifecycle APIs intentionally remain content-agnostic so recovery and
/// diagnostics can inspect saves whose content is currently unavailable. Callers that intend to
/// run gameplay should use this type instead: every operation reloads and validates the configured
/// local content catalog and resolves the campaign's exact persisted references before a
/// `RunnableCampaign` is returned.
#[derive(Debug, Clone)]
pub struct CampaignRuntime {
    pool: SqlitePool,
    content_roots: Vec<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunnableCampaign {
    lifecycle: CampaignLifecycleSummary,
    state: CampaignState,
    content: ResolvedCampaignContent,
}

impl RunnableCampaign {
    #[must_use]
    pub fn lifecycle(&self) -> &CampaignLifecycleSummary {
        &self.lifecycle
    }

    #[must_use]
    pub fn state(&self) -> &CampaignState {
        &self.state
    }

    #[must_use]
    pub fn content(&self) -> &ResolvedCampaignContent {
        &self.content
    }
}

#[derive(Debug, Error)]
pub enum RunnableCampaignError {
    #[error("{0}")]
    Table(String),
    #[error("content catalog could not be loaded: {0}")]
    Catalog(#[from] CatalogLoadError),
    #[error("campaign content could not be resolved: {0}")]
    Content(#[from] ContentResolutionError),
    #[error(transparent)]
    Lifecycle(#[from] LifecycleError),
    #[error("campaign export current state could not be decoded: {0}")]
    ExportStateDecode(String),
    #[error(transparent)]
    Rules(#[from] dmd_rules::RulesError),
    #[error("rules content is unavailable or incompatible: {0}")]
    RulesContent(String),
    #[error(transparent)]
    Journal(Box<dmd_persistence::JournalStoreError>),
    #[error(transparent)]
    Replay(Box<dmd_persistence::SnapshotReplayError>),
}

impl CampaignRuntime {
    #[must_use]
    pub fn new(pool: SqlitePool, content_roots: impl IntoIterator<Item = PathBuf>) -> Self {
        Self {
            pool,
            content_roots: content_roots.into_iter().collect(),
        }
    }

    #[must_use]
    pub fn from_content_root(pool: SqlitePool, content_root: impl AsRef<Path>) -> Self {
        Self::new(pool, [content_root.as_ref().to_path_buf()])
    }

    pub async fn create_campaign(
        &self,
        state: &CampaignState,
    ) -> Result<RunnableCampaign, RunnableCampaignError> {
        let catalog = self.load_catalog()?;
        let content = catalog.resolve_campaign(&state.campaign)?;
        let pack = Self::validate_rules(state, &content)?;

        let raw = create_campaign(&self.pool, state).await?;
        Self::make_runnable(raw, &catalog, pack.as_ref())
    }

    pub async fn open_campaign(
        &self,
        campaign_id: CampaignId,
    ) -> Result<RunnableCampaign, RunnableCampaignError> {
        let raw = open_campaign(&self.pool, campaign_id).await?;
        let catalog = self.load_catalog()?;
        let content = catalog.resolve_campaign(&raw.state.campaign)?;
        let pack = Self::validate_rules(&raw.state, &content)?;
        Self::make_runnable(raw, &catalog, pack.as_ref())
    }

    pub async fn resume_campaign(
        &self,
        campaign_id: CampaignId,
    ) -> Result<RunnableCampaign, RunnableCampaignError> {
        self.open_campaign(campaign_id).await
    }

    pub async fn restore_campaign(
        &self,
        export: &CampaignExport,
    ) -> Result<RunnableCampaign, RunnableCampaignError> {
        let upgraded = export.upgraded()?;
        let preflight_state = CampaignState::decode_json(&upgraded.current_state.state_json)
            .map_err(|error| RunnableCampaignError::ExportStateDecode(error.to_string()))?;
        let catalog = self.load_catalog()?;
        let content = catalog.resolve_campaign(&preflight_state.campaign)?;
        let mut pack = Self::validate_rules(&preflight_state, &content)?;
        if Self::has_rules_history(&upgraded, &preflight_state)? {
            if pack.is_none() {
                pack = Some(rules_runtime::load_rules_pack(&content)?);
            }
            rules_restore::validate_rules_export(
                &upgraded,
                pack.as_ref().expect("rules pack loaded"),
            )
            .map_err(RunnableCampaignError::RulesContent)?;
        }

        // Content preflight happens before raw restore opens its write transaction. Persistence then
        // revalidates the full export and restores atomically. Check its returned state against
        // the already validated content snapshot, without a fallible file read after the commit.
        let raw = restore_campaign(&self.pool, &upgraded).await?;
        Self::make_runnable(raw, &catalog, pack.as_ref())
    }

    fn load_catalog(&self) -> Result<ContentCatalog, CatalogLoadError> {
        ContentCatalog::load_from_roots(&self.content_roots)
    }

    fn make_runnable(
        raw: OpenCampaign,
        catalog: &ContentCatalog,
        pack: Option<&RulesPack>,
    ) -> Result<RunnableCampaign, RunnableCampaignError> {
        // Catalog resolution is pure: file integrity was checked before the write. A later
        // operation reloads content normally, so this reuse cannot authorize a future command.
        let content = catalog.resolve_campaign(&raw.state.campaign)?;
        if let Some(pack) = pack {
            table_engine::validate_table(&raw.state, pack).map_err(RunnableCampaignError::Table)?;
        } else if Self::uses_rules(&raw.state) {
            return Err(RunnableCampaignError::RulesContent(
                "returned state requires rules that were not validated before the operation".into(),
            ));
        }
        Ok(RunnableCampaign {
            lifecycle: raw.lifecycle,
            state: raw.state,
            content,
        })
    }

    fn validate_rules(
        state: &CampaignState,
        content: &ResolvedCampaignContent,
    ) -> Result<Option<RulesPack>, RunnableCampaignError> {
        if Self::uses_rules(state) {
            let pack = rules_runtime::load_rules_pack(content)?;
            table_engine::validate_table(state, &pack).map_err(RunnableCampaignError::Table)?;
            return Ok(Some(pack));
        }
        Ok(None)
    }

    fn uses_rules(state: &CampaignState) -> bool {
        state.rules.is_some() || state.campaign.ruleset.id == "srd-5.2"
    }

    fn has_rules_history(
        export: &CampaignExport,
        current: &CampaignState,
    ) -> Result<bool, RunnableCampaignError> {
        let mut found = Self::uses_rules(current)
            || export
                .command_audit
                .iter()
                .any(|audit| audit.command_kind.starts_with("rules."))
            || export
                .event_journal
                .iter()
                .any(|event| event.event_kind.starts_with("rules."));
        let codec = CampaignStateSnapshotCodec::new();
        for snapshot in &export.snapshots {
            let version = u32::try_from(snapshot.state_schema_version)
                .map_err(|error| RunnableCampaignError::ExportStateDecode(error.to_string()))?;
            let state = codec
                .decode_state(version, &snapshot.state_json)
                .map_err(|error| RunnableCampaignError::ExportStateDecode(error.to_string()))?;
            found |= Self::uses_rules(&state);
        }
        Ok(found)
    }
}
