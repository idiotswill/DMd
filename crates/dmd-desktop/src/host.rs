//! Local input boundary: the renderer selects a channel; this adapter derives authority.
//! Gameplay, durable acknowledgement and content validation stay in `dmd-app`.

use std::path::PathBuf;

use dmd_app::{
    CampaignRuntime, CharacterCreationOptions, RunnableCampaignError, TableAction,
    TableCampaignSummary, TableReceipt, TableTextResult, TableView, TableViewer,
};
use dmd_domain::{
    AgentRef, CampaignId, CharacterId, CommandId, CommandIssuer, CommandMeta, PlaySessionId,
    PlayerId, TableContract, TableSituation,
};
use serde::{Deserialize, Serialize};
use tauri::{Manager, State};
use tokio::sync::OnceCell;

struct DesktopPaths {
    data: PathBuf,
    content: PathBuf,
}

pub struct DesktopHost {
    paths: Result<DesktopPaths, DesktopError>,
    runtime: OnceCell<CampaignRuntime>,
}

impl DesktopHost {
    pub fn new(app: &tauri::AppHandle) -> Self {
        let paths = (|| {
            let data = app
                .path()
                .app_data_dir()
                .map_err(|_| DesktopError::storage())?;
            let content = app
                .path()
                .resource_dir()
                .map_err(|_| DesktopError::storage())?
                .join("content");
            Ok(DesktopPaths { data, content })
        })();
        Self {
            paths,
            runtime: OnceCell::new(),
        }
    }

    async fn runtime(&self) -> Result<&CampaignRuntime, DesktopError> {
        self.runtime
            .get_or_try_init(|| async {
                let paths = self.paths.as_ref().map_err(Clone::clone)?;
                std::fs::create_dir_all(&paths.data).map_err(|_| DesktopError::storage())?;
                CampaignRuntime::open_local(paths.data.join("campaigns.sqlite"), &paths.content)
                    .await
                    .map_err(|error| match error {
                        RunnableCampaignError::Table(_) => DesktopError::storage(),
                        other => DesktopError::from(other),
                    })
            })
            .await
    }
}

/// The message is safe to show; raw database errors and storage paths stay out of UI.
#[derive(Debug, Clone, Serialize)]
pub struct DesktopError {
    code: &'static str,
    message: String,
    retryable: bool,
}

impl DesktopError {
    fn storage() -> Self {
        Self { code: "local_storage", message: "DMd could not open its local data. Check that your user data folder is writable, then retry.".into(), retryable: true }
    }

    fn selection(message: &str) -> Self {
        Self {
            code: "channel_selection",
            message: message.into(),
            retryable: false,
        }
    }
}

impl From<RunnableCampaignError> for DesktopError {
    fn from(error: RunnableCampaignError) -> Self {
        match error {
            RunnableCampaignError::Table(message) => Self { code: "table_action", message, retryable: false },
            RunnableCampaignError::Catalog(_) | RunnableCampaignError::Content(_)
                | RunnableCampaignError::RulesContent(_) => Self {
                    code: "campaign_content",
                    message: "This campaign's required content is unavailable or incompatible. Reinstall the matching DMd package or restore its original content files, then retry.".into(),
                    retryable: true,
                },
            _ => Self {
                code: "campaign_operation",
                message: "DMd could not confirm this operation. Your original request has been retained; retry it before starting another action.".into(),
                retryable: true,
            },
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopStatus {
    app_name: &'static str,
    version: &'static str,
    runtime_ready: bool,
    message: &'static str,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateCampaignRequest {
    id: CampaignId,
    name: String,
    contract: TableContract,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OpenCampaignRequest {
    campaign_id: CampaignId,
    viewer: TableViewer,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CampaignRequest {
    campaign_id: CampaignId,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub enum LocalChannel {
    Host,
    Player {
        player_id: PlayerId,
        character_id: CharacterId,
    },
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TableActionRequest {
    command_id: CommandId,
    campaign_id: CampaignId,
    expected_event_sequence: u64,
    session_id: Option<PlaySessionId>,
    channel: LocalChannel,
    action: TableAction,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TableTextRequest {
    command_id: CommandId,
    campaign_id: CampaignId,
    expected_event_sequence: u64,
    session_id: Option<PlaySessionId>,
    channel: LocalChannel,
    text: String,
}

async fn command_meta(
    runtime: &CampaignRuntime,
    command_id: CommandId,
    campaign_id: CampaignId,
    expected_event_sequence: u64,
    session_id: Option<PlaySessionId>,
    channel: LocalChannel,
) -> Result<CommandMeta, DesktopError> {
    let (issuer, actor) = match channel {
        LocalChannel::Host => (CommandIssuer::Admin, None),
        LocalChannel::Player {
            player_id,
            character_id,
        } => {
            // Use persistent identity, not current attendance: accepted request retries after a
            // session ends must reconstruct the original metadata for application idempotency.
            let view = runtime
                .table_view(campaign_id, TableViewer::Player(player_id))
                .await?;
            let character = view
                .characters
                .iter()
                .find(|character| {
                    character.character_id == character_id && character.player_id == Some(player_id)
                })
                .ok_or_else(|| {
                    DesktopError::selection("Select a character controlled by this player.")
                })?;
            (
                CommandIssuer::Player(player_id),
                Some(AgentRef::Entity(character.entity_id)),
            )
        }
    };
    Ok(CommandMeta {
        id: command_id,
        campaign_id,
        session_id,
        issuer,
        actor,
        expected_event_sequence,
    })
}

#[tauri::command]
pub async fn desktop_status(host: State<'_, DesktopHost>) -> Result<DesktopStatus, DesktopError> {
    host.runtime().await?;
    Ok(DesktopStatus {
        app_name: "DMd",
        version: env!("CARGO_PKG_VERSION"),
        runtime_ready: true,
        message: "Your local table is ready.",
    })
}

#[tauri::command]
pub fn desktop_default_contract() -> TableContract {
    TableContract::default()
}

#[tauri::command]
pub async fn desktop_list_campaigns(
    host: State<'_, DesktopHost>,
) -> Result<Vec<TableCampaignSummary>, DesktopError> {
    Ok(host.runtime().await?.list_table_campaigns().await?)
}

#[tauri::command]
pub async fn desktop_create_campaign(
    host: State<'_, DesktopHost>,
    request: CreateCampaignRequest,
) -> Result<TableView, DesktopError> {
    Ok(host
        .runtime()
        .await?
        .create_table_campaign(request.id, &request.name, request.contract)
        .await?)
}

#[tauri::command]
pub async fn desktop_open_campaign(
    host: State<'_, DesktopHost>,
    request: OpenCampaignRequest,
) -> Result<TableView, DesktopError> {
    Ok(host
        .runtime()
        .await?
        .table_view(request.campaign_id, request.viewer)
        .await?)
}

#[tauri::command]
pub async fn desktop_creation_options(
    host: State<'_, DesktopHost>,
    request: CampaignRequest,
) -> Result<CharacterCreationOptions, DesktopError> {
    Ok(host
        .runtime()
        .await?
        .character_creation_options(request.campaign_id)
        .await?)
}

#[tauri::command]
pub async fn desktop_host_situation(
    host: State<'_, DesktopHost>,
    request: CampaignRequest,
) -> Result<TableSituation, DesktopError> {
    Ok(host
        .runtime()
        .await?
        .read_host_situation(request.campaign_id)
        .await?)
}

#[tauri::command]
pub async fn desktop_table_action(
    host: State<'_, DesktopHost>,
    request: TableActionRequest,
) -> Result<TableReceipt, DesktopError> {
    let runtime = host.runtime().await?;
    let meta = command_meta(
        runtime,
        request.command_id,
        request.campaign_id,
        request.expected_event_sequence,
        request.session_id,
        request.channel,
    )
    .await?;
    Ok(runtime.execute_table(meta, request.action).await?)
}

#[tauri::command]
pub async fn desktop_table_text(
    host: State<'_, DesktopHost>,
    request: TableTextRequest,
) -> Result<TableTextResult, DesktopError> {
    if matches!(request.channel, LocalChannel::Host) {
        return Err(DesktopError::selection(
            "Select a present player and their character before speaking at the table.",
        ));
    }
    let runtime = host.runtime().await?;
    let meta = command_meta(
        runtime,
        request.command_id,
        request.campaign_id,
        request.expected_event_sequence,
        request.session_id,
        request.channel,
    )
    .await?;
    Ok(runtime.submit_table_text(meta, &request.text).await?)
}
