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
            RunnableCampaignError::TableRejected(message) => Self { code: "table_action", message, retryable: false },
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

#[cfg(test)]
mod tests {
    use super::*;
    use dmd_domain::{AttendanceStatus, EntityId, SessionParticipant};
    use serde_json::json;

    #[test]
    fn only_proven_input_rejections_release_the_original_request() {
        let rejected = DesktopError::from(RunnableCampaignError::TableRejected(
            "Choose a present player.".into(),
        ));
        assert!(!rejected.retryable);
        assert_eq!(rejected.message, "Choose a present player.");
        let uncertain = DesktopError::from(RunnableCampaignError::Table(
            "database error at a private path".into(),
        ));
        assert!(uncertain.retryable);
        assert_eq!(uncertain.code, "campaign_operation");
        assert!(!uncertain.message.contains("private path"));
    }

    #[test]
    fn renderer_cannot_supply_issuer_or_actor() {
        let request = json!({
            "command_id": CommandId::new(), "campaign_id": CampaignId::new(),
            "expected_event_sequence": 0, "session_id": null, "channel": "Host",
            "action": "EndSession"
        });
        assert!(serde_json::from_value::<TableActionRequest>(request.clone()).is_ok());
        for (field, value) in [
            ("issuer", json!("System")),
            ("actor", json!({"Entity": EntityId::new()})),
        ] {
            let mut forged = request.clone();
            forged[field] = value;
            assert!(serde_json::from_value::<TableActionRequest>(forged).is_err());
        }
        let mut forged = request;
        forged["channel"] = json!({"Player": {"player_id": PlayerId::new(), "character_id": CharacterId::new(), "actor": EntityId::new()}});
        assert!(serde_json::from_value::<TableActionRequest>(forged).is_err());
    }

    #[tokio::test]
    async fn controller_selection_and_original_context_survive_session_end_retry() {
        let database =
            std::env::temp_dir().join(format!("dmd-desktop-test-{}.sqlite", CampaignId::new().0));
        let runtime = CampaignRuntime::open_local(
            &database,
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../content"),
        )
        .await
        .unwrap();
        let campaign = CampaignId::new();
        let player = PlayerId::new();
        let stranger = PlayerId::new();
        let character = CharacterId::new();
        let entity = EntityId::new();
        let session = PlaySessionId::new();
        runtime
            .create_table_campaign(campaign, "Desktop boundary test", TableContract::default())
            .await
            .unwrap();
        for (id, name) in [(player, "Controller"), (stranger, "Other player")] {
            host_action(
                &runtime,
                campaign,
                None,
                TableAction::AddPlayer {
                    id,
                    name: name.into(),
                },
            )
            .await;
        }
        // Enter source choices through the same deserialized application action accepted over IPC.
        let create = serde_json::from_value(json!({"CreateCharacter": {
            "character_id": character, "entity_id": entity, "player_id": player,
            "input": {
                "name": "Traveler", "pronouns": "they/them", "description": "A traveler",
                "alignment": "Neutral Good", "backstory": "A private aspiration",
                "ability_scores": [15,14,13,8,10,12], "background_boosts": [2,0,1,0,0,0],
                "fighter_skills": ["Perception","Survival"], "human_skill": "Insight",
                "skilled_skills": ["Acrobatics","Stealth","Investigation"], "size": "Medium",
                "languages": ["dwarvish","elvish"], "fighting_style": "Defense", "gaming_set": "Dice",
                "purchases": [{"item_id":"leather-armor","quantity":1}], "worn_armor":"leather-armor",
                "shield": false, "masteries": ["club","dagger","shortbow"]
            }
        }})).unwrap();
        host_action(&runtime, campaign, None, create).await;
        host_action(
            &runtime,
            campaign,
            Some(session),
            TableAction::StartSession {
                id: session,
                name: "An evening".into(),
                participants: vec![SessionParticipant {
                    player_id: player,
                    character_id: Some(character),
                    attendance: AttendanceStatus::Present,
                }],
            },
        )
        .await;
        let head = runtime
            .table_view(campaign, TableViewer::Host)
            .await
            .unwrap()
            .event_sequence;
        let id = CommandId::new();
        assert!(
            command_meta(
                &runtime,
                id,
                campaign,
                head,
                Some(session),
                LocalChannel::Player {
                    player_id: stranger,
                    character_id: character,
                }
            )
            .await
            .is_err()
        );
        let original = command_meta(
            &runtime,
            id,
            campaign,
            head,
            Some(session),
            LocalChannel::Player {
                player_id: player,
                character_id: character,
            },
        )
        .await
        .unwrap();
        assert_eq!(original.issuer, CommandIssuer::Player(player));
        assert_eq!(original.actor, Some(AgentRef::Entity(entity)));
        let result = runtime
            .submit_table_text(original.clone(), "How do I roll with advantage?")
            .await
            .unwrap();
        host_action(&runtime, campaign, Some(session), TableAction::EndSession).await;
        let ended_head = runtime
            .table_view(campaign, TableViewer::Host)
            .await
            .unwrap()
            .event_sequence;
        let retry = command_meta(
            &runtime,
            id,
            campaign,
            head,
            Some(session),
            LocalChannel::Player {
                player_id: player,
                character_id: character,
            },
        )
        .await
        .unwrap();
        assert_eq!(retry, original);
        assert_eq!(
            runtime
                .submit_table_text(retry, "How do I roll with advantage?")
                .await
                .unwrap(),
            result
        );
        assert_eq!(
            runtime
                .table_view(campaign, TableViewer::Host)
                .await
                .unwrap()
                .event_sequence,
            ended_head
        );
    }

    async fn host_action(
        runtime: &CampaignRuntime,
        campaign: CampaignId,
        session: Option<PlaySessionId>,
        action: TableAction,
    ) {
        let head = runtime
            .table_view(campaign, TableViewer::Host)
            .await
            .unwrap()
            .event_sequence;
        let meta = command_meta(
            runtime,
            CommandId::new(),
            campaign,
            head,
            session,
            LocalChannel::Host,
        )
        .await
        .unwrap();
        runtime.execute_table(meta, action).await.unwrap();
    }
}
