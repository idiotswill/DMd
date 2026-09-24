//! Trusted application composition for durable rules commands and read-only queries.
//!
//! Input adapters establish `RulesContext`; language/provider proposals supply only the action.
//! Every call reopens campaign content and state. No cached capability authorizes a later write.

use std::fs;

use dmd_core::GameCommand;
use dmd_domain::{
    AgentRef, CampaignId, CampaignState, CommandId, CommandIssuer, CommandMeta, DieResult,
    EntityId, EventId, EventSource, PendingEvent, PlaySessionId, ResolvedCampaignContent, RollMode,
    RollRequestId, RollResult, RollSource, fnv1a64_hex,
};
use dmd_persistence::{
    CommitReceipt, ReplayApplyError, ReplayEventApplier, StoredJournalEvent,
    commit_campaign_transition,
};
use dmd_rules::{RulesAction, RulesAnswer, RulesEvent, RulesOutcome, RulesPack, RulesQuery};
use rand::Rng;

use crate::{CampaignRuntime, RunnableCampaignError};

pub use dmd_rules::{RULES_EVENT_KIND, RULES_EVENT_VERSION};

/// Values supplied by the trusted local session/input channel, never by an interpreted proposal.
#[derive(Debug, Clone)]
pub struct RulesContext {
    pub campaign_id: CampaignId,
    pub issuer: CommandIssuer,
    pub actor: Option<EntityId>,
    pub session_id: Option<PlaySessionId>,
    pub expected_event_sequence: u64,
}

#[derive(Debug)]
pub struct RulesReceipt {
    pub commit: CommitReceipt,
    pub outcome: RulesOutcome,
}

impl CampaignRuntime {
    /// Reconstruct authoritative mechanics from immutable snapshots and recorded inputs using
    /// the campaign's installed content. This does not replace current state or write history.
    pub async fn replay_rules(
        &self,
        campaign_id: CampaignId,
    ) -> Result<CampaignState, RunnableCampaignError> {
        let runnable = self.open_campaign(campaign_id).await?;
        let pack = load_rules_pack(runnable.content())?;
        let applier = RulesReplayApplier::new(pack.clone());
        let state = dmd_persistence::replay_campaign_to_head(&self.pool, campaign_id, &applier)
            .await
            .map_err(|error| RunnableCampaignError::Replay(Box::new(error)))?;
        crate::table_engine::validate_table(&state, &pack).map_err(RunnableCampaignError::Table)?;
        Ok(state)
    }

    /// Resolve and atomically commit one typed action. A failed validation writes nothing.
    pub async fn execute_rules(
        &self,
        context: RulesContext,
        action: RulesAction,
    ) -> Result<RulesReceipt, RunnableCampaignError> {
        let runnable = self.open_campaign(context.campaign_id).await?;
        if runnable.state().table.is_some() {
            return Err(RunnableCampaignError::Table(
                "This campaign uses the table command path so pending decisions and mechanics commit together.".into(),
            ));
        }
        let pack = load_rules_pack(runnable.content())?;
        let meta = CommandMeta {
            id: CommandId::new(),
            campaign_id: context.campaign_id,
            session_id: context.session_id,
            issuer: context.issuer,
            actor: context.actor.map(AgentRef::Entity),
            expected_event_sequence: context.expected_event_sequence,
        };
        let mut transition = dmd_rules::resolve(runnable.state(), &meta, &action, &pack)?;
        // The kernel is sequence-neutral; persistence allocates the journal position. Its
        // atomic commit validates this proposed one-event image against the locked head.
        transition.next_state.applied_event_sequence =
            meta.expected_event_sequence.checked_add(1).ok_or_else(|| {
                RunnableCampaignError::RulesContent("journal sequence exhausted".into())
            })?;
        let command = GameCommand {
            meta: meta.clone(),
            payload: action,
        };
        let payload = command
            .encode_payload("rules.action", 1)
            .map_err(|error| RunnableCampaignError::RulesContent(error.to_string()))?;
        let event = PendingEvent {
            id: EventId::new(),
            occurred_at: transition.next_state.clock.now,
            source: EventSource::RuleResolution,
            actor: meta.actor,
            caused_by_event_ids: vec![],
            payload: transition.event,
        }
        .encode(RULES_EVENT_KIND, RULES_EVENT_VERSION)
        .map_err(|error| RunnableCampaignError::RulesContent(error.to_string()))?;
        let explanation = serde_json::to_string(&transition.outcome)
            .map_err(|error| RunnableCampaignError::RulesContent(error.to_string()))?;
        let commit = commit_campaign_transition(
            &self.pool,
            &meta,
            &payload,
            &transition.next_state,
            &[event],
            &explanation,
        )
        .await
        .map_err(|error| RunnableCampaignError::Journal(Box::new(error)))?;
        Ok(RulesReceipt {
            commit,
            outcome: transition.outcome,
        })
    }

    /// Pure rules questions never construct a command or call persistence's commit path.
    pub async fn query_rules(
        &self,
        campaign_id: CampaignId,
        viewer: CommandIssuer,
        query: RulesQuery,
    ) -> Result<RulesAnswer, RunnableCampaignError> {
        let runnable = self.open_campaign(campaign_id).await?;
        let pack = load_rules_pack(runnable.content())?;
        Ok(dmd_rules::query(runnable.state(), viewer, &query, &pack)?)
    }

    /// Generate runtime dice for a saved request. Only trusted runtime/admin callers may supply
    /// digital input; player clients submit physical faces through the ordinary action boundary.
    pub async fn roll_digitally(
        &self,
        context: RulesContext,
        request_id: RollRequestId,
    ) -> Result<RulesReceipt, RunnableCampaignError> {
        if !matches!(context.issuer, CommandIssuer::System | CommandIssuer::Admin) {
            return Err(dmd_rules::RulesError::Unauthorized.into());
        }
        let runnable = self.open_campaign(context.campaign_id).await?;
        if runnable.state().applied_event_sequence != context.expected_event_sequence {
            return Err(dmd_rules::RulesError::Stale.into());
        }
        let pending = runnable
            .state()
            .rules
            .as_ref()
            .and_then(|rules| rules.pending.as_ref())
            .filter(|pending| pending.request.id == request_id)
            .ok_or(dmd_rules::RulesError::NoPending)?;
        let request = &pending.request;
        let dice = {
            let mut rng = rand::thread_rng();
            request
                .dice
                .iter()
                .flat_map(|spec| {
                    let count = if request.mode == RollMode::Normal {
                        spec.count
                    } else {
                        2
                    };
                    (0..count)
                        .map(|_| DieResult {
                            sides: spec.sides,
                            value: rng.gen_range(1..=spec.sides),
                        })
                        .collect::<Vec<_>>()
                })
                .collect()
        };
        // Reopening inside execute_rules repeats content, authority and sequence validation. A
        // concurrent write rejects this result; never apply it against a different pending roll.
        self.execute_rules(
            context,
            RulesAction::SubmitRoll {
                result: RollResult {
                    request_id,
                    source: RollSource::Digital,
                    dice,
                },
            },
        )
        .await
    }
}

/// Read only declared, integrity-checked rules data; do not treat a manifest label as executable
/// rules support. The second checksum covers the actual bytes decoded here, after catalog load.
pub(crate) fn load_rules_pack(
    content: &ResolvedCampaignContent,
) -> Result<RulesPack, RunnableCampaignError> {
    let installed = &content.ruleset;
    if installed.manifest.id != "srd-5.2" || installed.manifest.version != "5.2.1" {
        return Err(RunnableCampaignError::RulesContent(
            "no implemented kernel for the campaign's exact ruleset version".into(),
        ));
    }
    let declared = installed
        .manifest
        .files
        .iter()
        .find(|file| file.path == "kernel.json")
        .ok_or_else(|| RunnableCampaignError::RulesContent("kernel.json is not declared".into()))?;
    let base = installed.manifest_path.parent().ok_or_else(|| {
        RunnableCampaignError::RulesContent("rules manifest has no parent directory".into())
    })?;
    let creation = installed
        .manifest
        .files
        .iter()
        .find(|file| file.path == "character-creation.json")
        .ok_or_else(|| {
            RunnableCampaignError::RulesContent("character-creation.json is not declared".into())
        })?;
    let creation_bytes = fs::read(base.join("character-creation.json"))
        .map_err(|error| RunnableCampaignError::RulesContent(error.to_string()))?;
    if creation_bytes.len() as u64 != creation.byte_len
        || fnv1a64_hex(&creation_bytes) != creation.checksum.value
        || creation_bytes != include_bytes!("../../../content/srd-5.2.1/character-creation.json")
    {
        return Err(RunnableCampaignError::RulesContent("installed character creation catalog does not match this exact supported rules version".into()));
    }
    let tactical = installed
        .manifest
        .files
        .iter()
        .find(|file| file.path == "tactical.json")
        .ok_or_else(|| {
            RunnableCampaignError::RulesContent("tactical.json is not declared".into())
        })?;
    let tactical_bytes = fs::read(base.join("tactical.json"))
        .map_err(|error| RunnableCampaignError::RulesContent(error.to_string()))?;
    if tactical_bytes.len() as u64 != tactical.byte_len
        || fnv1a64_hex(&tactical_bytes) != tactical.checksum.value
        || tactical_bytes != include_bytes!("../../../content/srd-5.2.1/tactical.json")
    {
        return Err(RunnableCampaignError::RulesContent(
            "installed tactical catalog does not match this exact supported rules version".into(),
        ));
    }
    let bytes = fs::read(base.join("kernel.json"))
        .map_err(|error| RunnableCampaignError::RulesContent(error.to_string()))?;
    if bytes.len() as u64 != declared.byte_len || fnv1a64_hex(&bytes) != declared.checksum.value {
        return Err(RunnableCampaignError::RulesContent(
            "kernel.json changed after catalog validation".into(),
        ));
    }
    let json = std::str::from_utf8(&bytes)
        .map_err(|error| RunnableCampaignError::RulesContent(error.to_string()))?;
    Ok(RulesPack::from_json(json)?)
}

/// Persistence orchestrates sequencing/integrity; this adapter owns typed rules replay semantics.
pub struct RulesReplayApplier {
    pack: RulesPack,
}

impl RulesReplayApplier {
    pub fn new(pack: RulesPack) -> Self {
        Self { pack }
    }
}

impl ReplayEventApplier for RulesReplayApplier {
    fn apply(
        &self,
        state: &mut CampaignState,
        event: &StoredJournalEvent,
    ) -> Result<(), ReplayApplyError> {
        let next = match (event.payload.kind.as_str(), event.payload.schema_version) {
            (RULES_EVENT_KIND, RULES_EVENT_VERSION) => {
                if state.table.is_some() {
                    return Err(ReplayApplyError::InvalidTransition(
                        "raw rules event bypasses the table command boundary".into(),
                    ));
                }
                let rules_event: RulesEvent = serde_json::from_str(&event.payload.json)
                    .map_err(|error| invalid_replay_payload(event, error))?;
                validate_replay_envelope(event, &rules_event.meta)?;
                dmd_rules::replay(state, &rules_event, &self.pack)
                    .map_err(|error| ReplayApplyError::InvalidTransition(error.to_string()))?
                    .next_state
            }
            (crate::TABLE_EVENT_KIND, crate::TABLE_EVENT_VERSION) => {
                let table_event: crate::TableEvent = serde_json::from_str(&event.payload.json)
                    .map_err(|error| invalid_replay_payload(event, error))?;
                validate_replay_envelope(event, &table_event.meta)?;
                crate::table_engine::replay_table(state, &table_event, &self.pack)
                    .map_err(ReplayApplyError::InvalidTransition)?
                    .state
            }
            _ => {
                return Err(ReplayApplyError::UnsupportedEvent {
                    kind: event.payload.kind.clone(),
                    schema_version: event.payload.schema_version,
                });
            }
        };
        if next.clock.now != event.meta.occurred_at {
            return Err(ReplayApplyError::InvalidTransition(
                "gameplay event time does not match its journal envelope".into(),
            ));
        }
        crate::table_engine::validate_table(&next, &self.pack)
            .map_err(ReplayApplyError::InvalidTransition)?;
        *state = next;
        Ok(())
    }
}

fn invalid_replay_payload(
    event: &StoredJournalEvent,
    error: serde_json::Error,
) -> ReplayApplyError {
    ReplayApplyError::InvalidPayload {
        kind: event.payload.kind.clone(),
        schema_version: event.payload.schema_version,
        message: error.to_string(),
    }
}

fn validate_replay_envelope(
    event: &StoredJournalEvent,
    meta: &CommandMeta,
) -> Result<(), ReplayApplyError> {
    if event.meta.campaign_id != meta.campaign_id
        || event.meta.command_id != Some(meta.id)
        || event.meta.actor != meta.actor
        || event.meta.session_id != meta.session_id
        || event.meta.source != EventSource::RuleResolution
        || Some(event.meta.sequence) != meta.expected_event_sequence.checked_add(1)
    {
        return Err(ReplayApplyError::InvalidTransition(
            "gameplay event authority metadata does not match its journal envelope".into(),
        ));
    }
    Ok(())
}
