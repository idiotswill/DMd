//! An explicit table presentation boundary, separate from source creature ownership.
use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::{CampaignState, CommandIssuer, CommandMeta, CreatureSourcePin, EntityId, PlayerId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TableSourceAccessVersion {
    SourceActorsV1,
}

/// Historical evidence at activation, never the current ownership map. Current
/// ownership remains CreatureRuntime.controller and its authenticated control_origin.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TableSourceAdoption {
    pub actor: EntityId,
    pub player_id: PlayerId,
    pub source: CreatureSourcePin,
    pub profile_origin: CommandMeta,
    pub control_origin: CommandMeta,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TableSourceActorAccess {
    pub version: TableSourceAccessVersion,
    pub origin: CommandMeta,
    pub adopted: Vec<TableSourceAdoption>,
}

impl TableSourceActorAccess {
    pub fn validate(&self, state: &CampaignState) -> Result<(), String> {
        if self.origin.id.0.is_nil()
            || self.origin.campaign_id != state.campaign_id()
            || self.origin.expected_event_sequence > state.applied_event_sequence
            || self.origin.issuer != CommandIssuer::Admin
            || self.origin.actor.is_some()
            || self.origin.session_id.is_some_and(|id| id.0.is_nil())
        {
            return Err("invalid table source-access activation provenance".into());
        }
        let mut actors = HashSet::new();
        for adoption in &self.adopted {
            let profile = state
                .rules
                .as_ref()
                .and_then(|rules| rules.tactical_creatures.as_ref())
                .and_then(|creatures| creatures.profile(adoption.actor))
                .ok_or("adopted source profile is absent")?;
            if !actors.insert(adoption.actor)
                || !state.players.contains_key(&adoption.player_id)
                || adoption.source != profile.source
                || adoption.profile_origin != profile.origin
                || adoption.profile_origin.expected_event_sequence
                    > self.origin.expected_event_sequence
                || adoption.control_origin.expected_event_sequence
                    > self.origin.expected_event_sequence
            {
                return Err("invalid historical source-control adoption".into());
            }
            crate::validate_creature_origin(state, &adoption.control_origin, adoption.actor)?;
        }
        Ok(())
    }
}
