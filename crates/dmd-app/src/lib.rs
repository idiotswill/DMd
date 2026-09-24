use std::path::{Path, PathBuf};

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
    pub lifecycle: CampaignLifecycleSummary,
    pub state: CampaignState,
    pub content: ResolvedCampaignContent,
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
        catalog.resolve_campaign(&state.campaign)?;

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
        let preflight_state = CampaignState::decode_json(&export.current_state.state_json)
            .map_err(|error| RunnableCampaignError::ExportStateDecode(error.to_string()))?;
        let catalog = self.load_catalog()?;
        catalog.resolve_campaign(&preflight_state.campaign)?;

        // Content preflight happens before raw restore opens its write transaction. Persistence then
        // revalidates the full export and restores atomically; resolution is repeated on the exact
        // state returned from persistence before it can cross the runnable boundary.
        let raw = restore_campaign(&self.pool, export).await?;
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
        Ok(RunnableCampaign {
            lifecycle: raw.lifecycle,
            state: raw.state,
            content,
        })
    }
}
