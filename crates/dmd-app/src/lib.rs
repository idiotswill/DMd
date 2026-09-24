use std::path::{Path, PathBuf};

mod rules_runtime;
mod rules_restore;
pub use rules_runtime::*;

use dmd_domain::{
    CampaignId, CampaignState, CatalogLoadError, ContentCatalog, ContentResolutionError,
    ResolvedCampaignContent,
};
use dmd_persistence::{
    CampaignExport, CampaignLifecycleSummary, LifecycleError, OpenCampaign, create_campaign,
    open_campaign, restore_campaign,
};
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
        Self::validate_rules(state, &content)?;

        let raw = create_campaign(&self.pool, state).await?;
        Self::make_runnable(raw, &catalog)
    }

    pub async fn open_campaign(
        &self,
        campaign_id: CampaignId,
    ) -> Result<RunnableCampaign, RunnableCampaignError> {
        let raw = open_campaign(&self.pool, campaign_id).await?;
        let catalog = self.load_catalog()?;
        Self::make_runnable(raw, &catalog)
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
        Self::validate_rules(&preflight_state, &content)?;
        if preflight_state.rules.is_some()
            || preflight_state.campaign.ruleset.id == "srd-5.2"
            || upgraded.event_journal.iter().any(|event| event.event_kind.starts_with("rules."))
        {
            let pack = rules_runtime::load_rules_pack(&content)?;
            rules_restore::validate_rules_export(&upgraded, &pack)
                .map_err(RunnableCampaignError::RulesContent)?;
        }

        // Content preflight happens before raw restore opens its write transaction. Persistence then
        // revalidates the full export and restores atomically; resolution is repeated on the exact
        // state returned from persistence before it can cross the runnable boundary.
        let raw = restore_campaign(&self.pool, &upgraded).await?;
        Self::make_runnable(raw, &catalog)
    }

    fn load_catalog(&self) -> Result<ContentCatalog, CatalogLoadError> {
        ContentCatalog::load_from_roots(&self.content_roots)
    }

    fn make_runnable(
        raw: OpenCampaign,
        catalog: &ContentCatalog,
    ) -> Result<RunnableCampaign, RunnableCampaignError> {
        let content = catalog.resolve_campaign(&raw.state.campaign)?;
        Self::validate_rules(&raw.state, &content)?;
        Ok(RunnableCampaign {
            lifecycle: raw.lifecycle,
            state: raw.state,
            content,
        })
    }

    fn validate_rules(
        state: &CampaignState,
        content: &ResolvedCampaignContent,
    ) -> Result<(), RunnableCampaignError> {
        if state.rules.is_some() || state.campaign.ruleset.id == "srd-5.2" {
            let pack = rules_runtime::load_rules_pack(content)?;
            dmd_rules::validate_state(state, &pack)?;
        }
        Ok(())
    }
}
