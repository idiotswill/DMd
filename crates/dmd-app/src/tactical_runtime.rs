//! Tactical application commands share the campaign's atomic journal and restore path.
use crate::{CampaignRuntime, RulesContext, RunnableCampaignError};
use dmd_core::GameCommand;
use dmd_domain::*;
use dmd_persistence::{CommitReceipt, commit_campaign_transition};
use dmd_rules::tactical::*;

#[derive(Debug)]
pub struct TacticalReceipt {
    pub commit: CommitReceipt,
    pub outcome: TacticalOutcome,
}

impl CampaignRuntime {
    /// Input adapters establish the issuer, actor and session; proposals supply only
    /// the typed action. Table campaigns must pass through their session boundary.
    pub async fn execute_tactical(
        &self,
        context: RulesContext,
        action: TacticalAction,
    ) -> Result<TacticalReceipt, RunnableCampaignError> {
        let runnable = self.open_campaign(context.campaign_id).await?;
        if runnable.state().table.is_some() {
            return Err(RunnableCampaignError::Table(
                "Table campaigns require the table encounter command path.".into(),
            ));
        }
        let pack = crate::rules_runtime::load_rules_pack(runnable.content())?;
        let meta = CommandMeta {
            id: CommandId::new(),
            campaign_id: context.campaign_id,
            session_id: context.session_id,
            issuer: context.issuer,
            actor: context.actor.map(AgentRef::Entity),
            expected_event_sequence: context.expected_event_sequence,
        };
        let mut transition = resolve_tactical(runnable.state(), &meta, &action, &pack)?;
        transition.next_state.applied_event_sequence =
            meta.expected_event_sequence.checked_add(1).ok_or_else(|| {
                RunnableCampaignError::RulesContent("journal sequence exhausted".into())
            })?;
        let payload = GameCommand {
            meta: meta.clone(),
            payload: action,
        }
        .encode_payload("tactical.action", 1)
        .map_err(|e| RunnableCampaignError::RulesContent(e.to_string()))?;
        let outcome = transition.event.outcome.clone();
        let event = PendingEvent {
            id: EventId::new(),
            occurred_at: transition.next_state.clock.now,
            source: EventSource::RuleResolution,
            actor: meta.actor,
            caused_by_event_ids: vec![],
            payload: transition.event,
        }
        .encode(TACTICAL_EVENT_KIND, TACTICAL_EVENT_VERSION)
        .map_err(|e| RunnableCampaignError::RulesContent(e.to_string()))?;
        let explanation = serde_json::to_string(&outcome)
            .map_err(|e| RunnableCampaignError::RulesContent(e.to_string()))?;
        let commit = commit_campaign_transition(
            &self.pool,
            &meta,
            &payload,
            &transition.next_state,
            &[event],
            &explanation,
        )
        .await
        .map_err(|e| RunnableCampaignError::Journal(Box::new(e)))?;
        Ok(TacticalReceipt { commit, outcome })
    }
}
