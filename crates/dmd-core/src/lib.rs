pub use dmd_domain::{CommandIssuer, CommandMeta};

use dmd_domain::{CommandId, EventId, RecordCodecError, SerializedRecord};
use serde::{Deserialize, Serialize};
use thiserror::Error;

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

impl<C: Serialize> GameCommand<C> {
    /// Serialize an already-typed command payload for durable audit storage.
    ///
    /// The provider layer cannot use this to bypass typing: it first has to construct `C` through
    /// the application/core validation boundary.
    pub fn encode_payload(
        &self,
        kind: impl Into<String>,
        schema_version: u32,
    ) -> Result<SerializedRecord, RecordCodecError> {
        SerializedRecord::encode(kind, schema_version, &self.payload)
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
    use dmd_domain::{CampaignId, PlayerId};

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    struct TestMove {
        steps: u32,
    }

    fn command() -> GameCommand<TestMove> {
        GameCommand {
            meta: CommandMeta {
                id: CommandId::new(),
                campaign_id: CampaignId::new(),
                session_id: None,
                issuer: CommandIssuer::Player(PlayerId::new()),
                actor: None,
                expected_event_sequence: 41,
            },
            payload: TestMove { steps: 3 },
        }
    }

    #[test]
    fn typed_command_mapping_preserves_authority_metadata() {
        let command = command();
        let command_id = command.meta.id;
        let campaign_id = command.meta.campaign_id;
        let issuer = command.meta.issuer;

        let mapped = command.map_payload(|payload| payload.steps);

        assert_eq!(mapped.meta.id, command_id);
        assert_eq!(mapped.meta.campaign_id, campaign_id);
        assert_eq!(mapped.meta.issuer, issuer);
        assert_eq!(mapped.meta.expected_event_sequence, 41);
        assert_eq!(mapped.payload, 3);
    }

    #[test]
    fn typed_command_payload_can_be_encoded_for_audit() {
        let command = command();
        let encoded = command
            .encode_payload("test.move", 1)
            .expect("typed command should encode");
        let decoded: TestMove = encoded.decode().expect("payload should decode");

        assert_eq!(encoded.kind(), "test.move");
        assert_eq!(encoded.schema_version(), 1);
        assert_eq!(decoded, command.payload);
    }
}
