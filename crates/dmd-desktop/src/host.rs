//! Local input boundary: the renderer selects a channel; this adapter derives authority.
//! Gameplay, durable acknowledgement and content validation stay in `dmd-app`.

use std::path::PathBuf;

use dmd_app::{
    CampaignRuntime, CharacterCreationOptions, RunnableCampaignError, TableAction,
    TableCampaignSummary, TablePresentedView, TableViewer,
};
use dmd_domain::{CampaignId, CommandId, PlaySessionId, TableContract, TableSituation};
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

pub type LocalChannel = dmd_app::TableTransportChannel;

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
) -> Result<TablePresentedView, DesktopError> {
    let runtime = host.runtime().await?;
    runtime
        .create_table_campaign(request.id, &request.name, request.contract)
        .await?;
    Ok(runtime
        .presented_table_view(request.id, TableViewer::Host)
        .await?)
}
#[tauri::command]
pub async fn desktop_open_campaign(
    host: State<'_, DesktopHost>,
    request: OpenCampaignRequest,
) -> Result<TablePresentedView, DesktopError> {
    Ok(host
        .runtime()
        .await?
        .presented_table_view(request.campaign_id, request.viewer)
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
) -> Result<dmd_app::LegacyTableAcknowledgement, DesktopError> {
    Ok(host
        .runtime()
        .await?
        .recover_legacy_table_request(dmd_app::LegacyTableRequest {
            command_id: request.command_id,
            campaign_id: request.campaign_id,
            expected_event_sequence: request.expected_event_sequence,
            session_id: request.session_id,
            channel: request.channel,
            input: dmd_app::LegacyTableInput::Action(Box::new(request.action)),
        })
        .await?)
}

#[tauri::command]
pub async fn desktop_table_text(
    host: State<'_, DesktopHost>,
    request: TableTextRequest,
) -> Result<dmd_app::LegacyTableAcknowledgement, DesktopError> {
    Ok(host
        .runtime()
        .await?
        .recover_legacy_table_request(dmd_app::LegacyTableRequest {
            command_id: request.command_id,
            campaign_id: request.campaign_id,
            expected_event_sequence: request.expected_event_sequence,
            session_id: request.session_id,
            channel: request.channel,
            input: dmd_app::LegacyTableInput::Text { text: request.text },
        })
        .await?)
}

#[tauri::command]
pub async fn desktop_submit_table(
    host: State<'_, DesktopHost>,
    request: dmd_app::TableTransportRequest,
) -> Result<dmd_app::TableTransportResult, DesktopError> {
    Ok(host
        .runtime()
        .await?
        .submit_presented_table(request)
        .await?)
}

#[tauri::command]
pub async fn desktop_roll_options(
    host: State<'_, DesktopHost>,
    request: dmd_app::TableRollOptionsRequest,
) -> Result<dmd_app::TableRollOptions, DesktopError> {
    Ok(host.runtime().await?.table_roll_options(request).await?)
}
#[cfg(test)]
mod tests {
    use super::*;
    use dmd_app::{TABLE_TRANSPORT_VERSION, TableTransportInput, TableTransportRequest};
    use dmd_domain::{AttendanceStatus, CharacterId, EntityId, PlayerId, SessionParticipant};
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
        let modern = json!({
            "version": TABLE_TRANSPORT_VERSION,
            "command_id": CommandId::new(), "campaign_id": CampaignId::new(),
            "revision": CommandId::new(), "session_id": null, "channel": "Host",
            "input": {"Action": "EndSession"}
        });
        assert!(serde_json::from_value::<TableTransportRequest>(modern.clone()).is_ok());
        for (field, value) in [
            ("issuer", json!("System")),
            ("actor", json!({"Entity": EntityId::new()})),
            ("expected_event_sequence", json!(0)),
        ] {
            let mut forged = modern.clone();
            forged[field] = value;
            assert!(serde_json::from_value::<TableTransportRequest>(forged).is_err());
        }
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
        let stranger_view = runtime
            .presented_table_view(campaign, TableViewer::Player(stranger))
            .await
            .unwrap();
        let mut original = TableTransportRequest {
            version: TABLE_TRANSPORT_VERSION,
            command_id: CommandId::new(),
            campaign_id: campaign,
            session_id: Some(session),
            channel: LocalChannel::Player {
                player_id: stranger,
                character_id: character,
            },
            revision: stranger_view.revision,
            input: TableTransportInput::Text {
                text: "How do I roll with advantage?".into(),
            },
        };
        assert!(
            matches!(runtime.submit_presented_table(original.clone()).await,
            Err(RunnableCampaignError::TableRejected(message)) if message.contains("controlled by this player"))
        );
        original.channel = LocalChannel::Player {
            player_id: player,
            character_id: character,
        };
        original.revision = runtime
            .presented_table_view(campaign, TableViewer::Player(player))
            .await
            .unwrap()
            .revision;
        let result = runtime
            .submit_presented_table(original.clone())
            .await
            .unwrap();
        let rendered = serde_json::to_string(&result).unwrap();
        assert!(!rendered.contains("event_sequence"));
        assert!(!rendered.contains("issuer"));
        host_action(&runtime, campaign, Some(session), TableAction::EndSession).await;
        let ended_head = runtime
            .presented_table_view(campaign, TableViewer::Host)
            .await
            .unwrap()
            .diagnostics
            .unwrap()
            .canonical_event_sequence;
        assert_eq!(
            runtime.submit_presented_table(original).await.unwrap(),
            result
        );
        assert_eq!(
            runtime
                .presented_table_view(campaign, TableViewer::Host)
                .await
                .unwrap()
                .diagnostics
                .unwrap()
                .canonical_event_sequence,
            ended_head
        );
    }

    async fn host_action(
        runtime: &CampaignRuntime,
        campaign: CampaignId,
        session: Option<PlaySessionId>,
        action: TableAction,
    ) {
        let view = runtime
            .presented_table_view(campaign, TableViewer::Host)
            .await
            .unwrap();
        runtime
            .submit_presented_table(TableTransportRequest {
                version: TABLE_TRANSPORT_VERSION,
                command_id: CommandId::new(),
                campaign_id: campaign,
                session_id: session,
                channel: LocalChannel::Host,
                revision: view.revision,
                input: TableTransportInput::Action(Box::new(action)),
            })
            .await
            .unwrap();
    }
}
