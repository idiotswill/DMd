use dmd_domain::{AgentRef, CampaignId, CommandId, EventId, PlaySessionId};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandMeta {
    pub id: CommandId,
    pub campaign_id: CampaignId,
    /// None is valid for between-session simulation, imports, and maintenance commands.
    pub session_id: Option<PlaySessionId>,
    pub actor: Option<AgentRef>,
    /// Journal sequence observed when the command was constructed.
    pub expected_event_sequence: u64,
}

/// A typed request to mutate or resolve authoritative game state.
///
/// Untrusted language-model/provider output must be parsed into a concrete `C` before it can
/// cross this boundary. Core handlers must not dispatch on string command names or inspect
/// arbitrary JSON payloads.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GameCommand<C> {
    pub meta: CommandMeta,
    pub payload: C,
}

impl<C> GameCommand<C> {
    #[must_use]
    pub fn map_payload<U>(self, f: impl FnOnce(C) -> U) -> GameCommand<U> {
        GameCommand {
            meta: self.meta,
            payload: f(self.payload),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

pub trait CommandHandler<C> {
    fn handle(&self, command: GameCommand<C>) -> Result<ResolutionRecord, CoreError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    struct TestMove {
        steps: u32,
    }

    #[test]
    fn typed_command_mapping_preserves_authority_metadata() {
        let command_id = CommandId::new();
        let campaign_id = CampaignId::new();
        let command = GameCommand {
            meta: CommandMeta {
                id: command_id,
                campaign_id,
                session_id: None,
                actor: None,
                expected_event_sequence: 41,
            },
            payload: TestMove { steps: 3 },
        };

        let mapped = command.map_payload(|payload| payload.steps);

        assert_eq!(mapped.meta.id, command_id);
        assert_eq!(mapped.meta.campaign_id, campaign_id);
        assert_eq!(mapped.meta.expected_event_sequence, 41);
        assert_eq!(mapped.payload, 3);
    }
}
