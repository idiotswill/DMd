use dmd_domain::CampaignId;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameCommand {
    pub campaign_id: CampaignId,
    pub kind: String,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolutionRecord {
    pub accepted: bool,
    pub explanation: String,
}

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("command is invalid: {0}")]
    InvalidCommand(String),
}

pub trait CommandHandler {
    fn handle(&self, command: GameCommand) -> Result<ResolutionRecord, CoreError>;
}
