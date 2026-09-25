//! Desktop envelopes contain presentation revisions and typed intent, never canonical heads.
use crate::*;
use dmd_domain::*;
use dmd_persistence::{
    ProjectionAudience, ProjectionCapability, ProjectionHandle, ProjectionRevision,
};
use serde::{Deserialize, Serialize};

pub const TABLE_TRANSPORT_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum TableTransportChannel {
    Host,
    Player {
        player_id: PlayerId,
        character_id: CharacterId,
    },
}
impl TableTransportChannel {
    pub(crate) fn audience(&self) -> ProjectionAudience {
        match self {
            Self::Host => ProjectionAudience::Host,
            Self::Player { player_id, .. } => ProjectionAudience::Player(*player_id),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum TableTransportInput {
    Action(Box<TableAction>),
    Text { text: String },
    SelectWork { handle: CommandId },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TableTransportRequest {
    pub version: u32,
    pub command_id: CommandId,
    pub campaign_id: CampaignId,
    pub session_id: Option<PlaySessionId>,
    pub channel: TableTransportChannel,
    pub revision: ProjectionRevision,
    pub input: TableTransportInput,
}

/// Current host creation choices do not alter immutable presentation v1 bytes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TableCreatureOptionsRequest {
    pub campaign_id: CampaignId,
    pub channel: TableTransportChannel,
    pub revision: ProjectionRevision,
}

/// Read-only live roll affordances are separate from immutable presentation v1.
/// They are tied to the owned opaque request; accepting them still rederives rules.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TableRollOptionsRequest {
    pub campaign_id: CampaignId,
    pub channel: TableTransportChannel,
    pub revision: ProjectionRevision,
    pub roll_id: RollRequestId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TableRollOptions {
    pub savage_attacker: Option<TableSavageAttackerOption>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TableSavageAttackerOption {
    pub weapon_dice: usize,
    pub heroic_inspiration: bool,
}

/// Only recovery of an already accepted v1 request uses this old numeric field.
/// This envelope is never converted into a newly accepted game command.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LegacyTableRequest {
    pub command_id: CommandId,
    pub campaign_id: CampaignId,
    pub expected_event_sequence: u64,
    pub session_id: Option<PlaySessionId>,
    pub channel: TableTransportChannel,
    pub input: LegacyTableInput,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LegacyTableInput {
    Action(Box<TableAction>),
    Text { text: String },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LegacyTableAcknowledgement {
    Accepted {
        command_id: CommandId,
        outcome: TablePresentedOutcome,
    },
    Observed {
        command_id: CommandId,
        text: String,
        answer: String,
    },
}

/// Retained in canonical table.action@2 alongside its derived game action. This marker
/// makes missing transport history distinguishable from genuinely legacy acceptance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct TransportedTableAction {
    pub request: TableTransportRequest,
    pub action: TableAction,
}

/// Observation@2 has the same immutable answer/provenance semantics as v1, with the
/// original audience-revision request retained for independent transport validation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct TransportedTableObservation {
    pub request: TableTransportRequest,
    pub body: TableObservationBody,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TablePresentedOutcome {
    pub message: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TablePresentedReceipt {
    pub command_id: CommandId,
    pub revision: ProjectionRevision,
    pub outcome: TablePresentedOutcome,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TablePresentedObservation {
    pub command_id: CommandId,
    pub revision: ProjectionRevision,
    pub text: String,
    pub answer: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TableTransportResult {
    Accepted(TablePresentedReceipt),
    Observed(TablePresentedObservation),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TablePresentedPending {
    pub id: CommandId,
    pub session_id: PlaySessionId,
    pub player_id: PlayerId,
    pub character_id: CharacterId,
    pub actor: EntityId,
    pub revision: u32,
    pub text: String,
    pub intent: TableIntent,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TablePresentedTranscriptEntry {
    pub id: String,
    pub kind: String,
    pub speaker: String,
    pub text: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TablePresentedWorkChoice {
    pub handle: CommandId,
    pub label: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TableHostDiagnostics {
    pub canonical_event_sequence: u64,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TablePresentedView {
    pub revision: ProjectionRevision,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diagnostics: Option<TableHostDiagnostics>,
    pub campaign_id: CampaignId,
    pub name: String,
    pub contract: TableContract,
    pub players: Vec<Player>,
    pub characters: Vec<TableCharacterView>,
    pub active_session: Option<ActiveTableSession>,
    pub pending: Option<TablePresentedPending>,
    pub roll: Option<RollRequest>,
    pub roll_channel: Option<TableRollChannel>,
    pub tactical: Option<TableTacticalView<TablePresentedWorkChoice>>,
    pub creature_setup: Option<TableCreatureSetupView>,
    pub situation_title: String,
    pub situation_description: String,
    pub transcript: Vec<TablePresentedTranscriptEntry>,
    pub recap: Vec<String>,
}

pub(crate) fn presented_view(
    raw: TableView,
    audience: ProjectionAudience,
    revision: ProjectionRevision,
    handles: &[ProjectionHandle],
    work_origin: Option<CommandId>,
) -> Result<TablePresentedView, String> {
    let handle = |capability: &ProjectionCapability| {
        let mut found = handles
            .iter()
            .filter(|entry| &entry.capability == capability);
        let result = found
            .next()
            .ok_or("missing presentation capability")?
            .opaque;
        if found.next().is_some() {
            return Err("duplicate presentation capability");
        }
        Ok(result)
    };
    let mut roll = raw.roll;
    if let Some(roll) = &mut roll {
        roll.id = RollRequestId(handle(&ProjectionCapability::Roll { canonical: roll.id })?);
    }
    let tactical = raw
        .tactical
        .map(|tactical| {
            let TableTacticalView {
                encounter_id,
                aftermath,
                execution,
                ready,
                round,
                active_actor,
                phase,
                battlefield,
                participants,
                combatant_sources,
                observers,
                initiative,
                ties,
                budget,
                continuation,
                may_fail_save,
                legendary_resistance,
                legendary_action,
                attack_options,
                casting_options,
                attack_decision,
                movement_options,
                opportunity,
                liquid_landing,
                shield_options,
                area_options,
            } = tactical;
            let continuation = continuation
                .map(|value| {
                    let choices = value
                        .choices
                        .into_iter()
                        .map(|choice| {
                            let origin =
                                work_origin.ok_or("work choices lack a resolution origin")?;
                            Ok(TablePresentedWorkChoice {
                                handle: CommandId(handle(&ProjectionCapability::Work {
                                    origin,
                                    occurrence: choice.occurrence,
                                })?),
                                label: choice.label,
                            })
                        })
                        .collect::<Result<Vec<_>, &str>>()?;
                    Ok::<_, &str>(TableTacticalContinuation {
                        actor: value.actor,
                        host_adjudication: value.host_adjudication,
                        choices,
                    })
                })
                .transpose()?;
            Ok::<_, &str>(TableTacticalView {
                encounter_id,
                aftermath,
                execution,
                ready,
                round,
                active_actor,
                phase,
                battlefield,
                participants,
                combatant_sources,
                observers,
                initiative,
                ties,
                budget,
                continuation,
                may_fail_save,
                legendary_resistance,
                legendary_action,
                attack_options,
                casting_options,
                attack_decision,
                movement_options,
                opportunity,
                liquid_landing,
                shield_options,
                area_options,
            })
        })
        .transpose()
        .map_err(str::to_owned)?;
    Ok(TablePresentedView {
        revision,
        diagnostics: matches!(audience, ProjectionAudience::Host).then_some(TableHostDiagnostics {
            canonical_event_sequence: raw.event_sequence,
        }),
        campaign_id: raw.campaign_id,
        name: raw.name,
        contract: raw.contract,
        players: raw.players,
        characters: raw.characters,
        active_session: raw.active_session,
        pending: raw.pending.map(|p| TablePresentedPending {
            id: p.id,
            session_id: p.session_id,
            player_id: p.player_id,
            character_id: p.character_id,
            actor: p.actor,
            revision: p.revision,
            text: p.text,
            intent: p.intent,
        }),
        roll,
        roll_channel: raw.roll_channel,
        tactical,
        creature_setup: raw.creature_setup,
        situation_title: raw.situation_title,
        situation_description: raw.situation_description,
        transcript: raw
            .transcript
            .into_iter()
            .map(|entry| TablePresentedTranscriptEntry {
                id: entry.id,
                kind: entry.kind,
                speaker: entry.speaker,
                text: entry.text,
            })
            .collect(),
        recap: raw.recap,
    })
}
