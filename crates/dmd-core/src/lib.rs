use dmd_domain::{CampaignId, CommandId, EntityId, EventId};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameCommand {
    pub id: CommandId,
    pub campaign_id: CampaignId,
    pub actor: Option<EntityId>,
    /// Journal sequence observed when the command was constructed.
    pub expected_event_sequence: u64,
    pub kind: String,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolutionRecord {
    pub command_id: CommandId,
    pub accepted: bool,
    pub explanation: String,
    pub emitted_event_ids: Vec<EventId>,
    pub resulting_event_sequence: u64,
}

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("command is invalid: {0}")]
    InvalidCommand(String),
    #[error("command was based on event sequence {expected}, but current sequence is {actual}")]
    StaleState { expected: u64, actual: u64 },
    #[error("command campaign does not match loaded campaign")]
    CampaignMismatch,
}

pub trait CommandHandler {
    fn handle(&self, command: GameCommand) -> Result<ResolutionRecord, CoreError>;
}
