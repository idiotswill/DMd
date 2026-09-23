use serde::{Deserialize, Serialize};

use crate::{AgentRef, CampaignId, CommandId, PlaySessionId, PlayerId};

/// Trusted authority context attached by the application/input layer.
///
/// This is shared domain metadata because persistence must preserve the authority principal
/// independently of any in-world actor. Language/STT providers do not establish this value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommandIssuer {
    Player(PlayerId),
    System,
    Admin,
    Import,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandMeta {
    pub id: CommandId,
    pub campaign_id: CampaignId,
    /// None is valid for between-session simulation, imports, and maintenance commands.
    pub session_id: Option<PlaySessionId>,
    /// Trusted authority principal. This is distinct from the in-world actor.
    pub issuer: CommandIssuer,
    /// Entity/faction attempting the in-world action, if the command has one.
    pub actor: Option<AgentRef>,
    /// Journal sequence observed when the command was constructed.
    pub expected_event_sequence: u64,
}
