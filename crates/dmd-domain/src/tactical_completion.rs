//! Immutable encounter completion evidence and retired spatial placements.
//! Gameplay resources and physical custody remain in their existing stores.
use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::{
    AgentRef, Battlefield, CampaignState, CommandId, CommandIssuer, CommandMeta, Custody,
    EffectTurn, EncounterId, EntityId, LocationId, SceneId, SceneStatus, TacticalExecutionVersion,
    TacticalGroundItem, TacticalPhase, TurnBoundary, WorldInstant,
};

/// One accepted release, retained after its live encounter is replaced. The
/// ordered receipt chain owns global turn highwater and all-time identity reuse.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TacticalCompletion {
    pub encounter_id: EncounterId,
    pub scene_id: SceneId,
    pub location_id: LocationId,
    pub encounter_origin: CommandMeta,
    pub initiative_origin: CommandMeta,
    pub conclusion_origin: CommandMeta,
    pub released_by: CommandMeta,
    pub predecessor: Option<CommandId>,
    pub released_at: WorldInstant,
    pub final_turn: EffectTurn,
    pub execution: TacticalExecutionVersion,
}

/// Exact old geometry and loose-item positions, not a second inventory. Actor
/// identities only validate attached lights; no mutable actor resources live here.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TacticalSceneSpace {
    pub encounter_id: EncounterId,
    pub scene_id: SceneId,
    pub location_id: LocationId,
    pub release: CommandId,
    pub battlefield: Battlefield,
    pub participants: Vec<EntityId>,
    pub ground_items: Vec<TacticalGroundItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TacticalEncounterHistory {
    pub completions: Vec<TacticalCompletion>,
    pub spaces: Vec<TacticalSceneSpace>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub elapsed_intervals: Vec<crate::ReleasedElapsedReceipt>,
}

fn command_origin(state: &CampaignState, origin: &CommandMeta) -> Result<(), String> {
    if origin.id.0.is_nil()
        || origin.campaign_id != state.campaign_id()
        || origin.expected_event_sequence > state.applied_event_sequence
        || origin.session_id.is_some_and(|id| id.0.is_nil())
        || matches!(origin.issuer, CommandIssuer::Import)
        || matches!(origin.issuer, CommandIssuer::Player(id)
            if !state.players.contains_key(&id) || origin.actor.is_none())
        || origin.actor.is_some_and(|actor| match actor {
            AgentRef::Entity(id) => !state.entities.contains_key(&id),
            AgentRef::Faction(_) => true,
        })
    {
        return Err("invalid encounter completion command provenance".into());
    }
    Ok(())
}

fn host_origin(state: &CampaignState, origin: &CommandMeta) -> Result<(), String> {
    command_origin(state, origin)?;
    if !matches!(origin.issuer, CommandIssuer::Admin | CommandIssuer::System)
        || origin.actor.is_some()
    {
        return Err("encounter completion requires trusted host provenance".into());
    }
    Ok(())
}

impl TacticalEncounterHistory {
    fn validate_elapsed(&self, state: &CampaignState) -> Result<(), String> {
        let mut previous: Option<&crate::ReleasedElapsedReceipt> = None;
        let mut commands = self
            .completions
            .iter()
            .flat_map(|c| {
                [
                    &c.encounter_origin,
                    &c.initiative_origin,
                    &c.conclusion_origin,
                    &c.released_by,
                ]
            })
            .map(|m| m.id)
            .collect::<HashSet<_>>();
        for elapsed in &self.elapsed_intervals {
            host_origin(state, &elapsed.origin)?;
            host_origin(state, &elapsed.completed_by)?;
            let release = self
                .completions
                .iter()
                .rev()
                .find(|c| {
                    c.released_by.expected_event_sequence < elapsed.origin.expected_event_sequence
                })
                .ok_or("elapsed interval predates every release")?;
            if elapsed.origin.session_id.is_none()
                || elapsed.completed_by.session_id.is_none()
                || elapsed.release != release.released_by.id
                || elapsed.predecessor != previous.map(|p| p.origin.id)
                || elapsed.started_at < release.released_at
                || elapsed.target_at <= elapsed.started_at
                || elapsed.completed_at != elapsed.target_at
                || elapsed.completed_at > state.clock.now
                || !crate::valid_released_time_ruling(&elapsed.ruling)
                || elapsed.completed_by.expected_event_sequence
                    < elapsed.origin.expected_event_sequence
                || (elapsed.completed_by.expected_event_sequence
                    == elapsed.origin.expected_event_sequence
                    && elapsed.completed_by != elapsed.origin)
                || (elapsed.completed_by.id == elapsed.origin.id
                    && elapsed.completed_by != elapsed.origin)
                || !commands.insert(elapsed.origin.id)
                || (elapsed.completed_by.id != elapsed.origin.id
                    && !commands.insert(elapsed.completed_by.id))
                || previous.is_some_and(|p| {
                    p.completed_by.expected_event_sequence >= elapsed.origin.expected_event_sequence
                        || p.target_at > elapsed.started_at
                })
                || self.completions.iter().any(|c| {
                    c.encounter_origin.expected_event_sequence
                        > elapsed.origin.expected_event_sequence
                        && c.encounter_origin.expected_event_sequence
                            <= elapsed.completed_by.expected_event_sequence
                })
            {
                return Err("invalid released interval receipt chain".into());
            }
            previous = Some(elapsed);
        }
        Ok(())
    }

    pub fn last(&self) -> Option<&TacticalCompletion> {
        self.completions.last()
    }

    pub fn contains_encounter(&self, encounter: EncounterId) -> bool {
        self.completions
            .iter()
            .any(|receipt| receipt.encounter_id == encounter)
    }

    pub fn next_turn_number(&self) -> Result<u64, String> {
        self.last().map_or(Ok(1), |receipt| {
            receipt
                .final_turn
                .number
                .checked_add(1)
                .ok_or_else(|| "encounter turn highwater is exhausted".into())
        })
    }

    /// Structural checks do not authenticate history. Strict journal replay must
    /// reconstruct every receipt/space from the accepted release's prior state.
    pub fn validate(&self, state: &CampaignState) -> Result<(), String> {
        if self.completions.is_empty() || self.spaces.len() != self.completions.len() {
            return Err("completion history requires one scene space per receipt".into());
        }
        let mut encounters = HashSet::new();
        let mut scenes = HashSet::new();
        let mut commands = HashSet::new();
        let mut previous: Option<&TacticalCompletion> = None;
        let mut placed = HashSet::new();
        for (receipt, space) in self.completions.iter().zip(&self.spaces) {
            let origins = [
                &receipt.encounter_origin,
                &receipt.initiative_origin,
                &receipt.conclusion_origin,
                &receipt.released_by,
            ];
            for origin in origins {
                host_origin(state, origin)?;
                if !commands.insert(origin.id) {
                    return Err("completion command identity is reused".into());
                }
            }
            if origins
                .windows(2)
                .any(|pair| pair[0].expected_event_sequence >= pair[1].expected_event_sequence)
                || receipt.encounter_id.0.is_nil()
                || !encounters.insert(receipt.encounter_id)
                || !receipt.execution.supports_release()
                || receipt.final_turn.number == 0
                || receipt.final_turn.number == u64::MAX
                || receipt.final_turn.boundary != TurnBoundary::Start
                || !state.entities.contains_key(&receipt.final_turn.actor)
                || receipt.released_at > state.clock.now
                || receipt.predecessor != previous.map(|prior| prior.released_by.id)
                || previous.is_some_and(|prior| {
                    receipt.encounter_origin.expected_event_sequence
                        <= prior.released_by.expected_event_sequence
                        || receipt.final_turn.number <= prior.final_turn.number
                        || receipt.released_at < prior.released_at
                })
            {
                return Err("invalid completion identity, origin chain or turn highwater".into());
            }
            let scene = state
                .scenes
                .get(&receipt.scene_id)
                .ok_or("completed encounter scene is absent")?;
            if scene.location_id != receipt.location_id
                || scene.campaign_id != state.campaign_id()
                || scene.status != SceneStatus::Closed
                || !scenes.insert(receipt.scene_id)
                || !state.locations.contains_key(&receipt.location_id)
                || space.encounter_id != receipt.encounter_id
                || space.scene_id != receipt.scene_id
                || space.location_id != receipt.location_id
                || space.release != receipt.released_by.id
                || space.participants.is_empty()
                || space.participants.len() > crate::MAX_TACTICAL_PARTICIPANTS
                || space
                    .participants
                    .iter()
                    .any(|id| !state.entities.contains_key(id))
                || !space.participants.contains(&receipt.final_turn.actor)
            {
                return Err("retired scene identity differs from completion".into());
            }
            let actors = space.participants.iter().copied().collect::<HashSet<_>>();
            if actors.len() != space.participants.len() {
                return Err("duplicate retired scene participant".into());
            }
            space.battlefield.validate_for_participants(&actors)?;
            for ground in &space.ground_items {
                command_origin(state, &ground.origin)?;
                ground.position.validate()?;
                if !placed.insert(ground.item)
                    || !space.battlefield.bounds.contains(ground.position)
                    || ground.origin.expected_event_sequence
                        >= receipt.released_by.expected_event_sequence
                    || state
                        .items
                        .get(&ground.item)
                        .is_none_or(|item| item.custody != Custody::Location(space.location_id))
                {
                    return Err(
                        "retired item placement has invalid bounds, origin or custody".into(),
                    );
                }
            }
            previous = Some(receipt);
        }
        self.validate_elapsed(state)?;
        if let Some(upgrade) = state
            .encounter
            .as_ref()
            .and_then(|e| e.flow.as_ref())
            .and_then(|f| f.released_time_upgrade.as_ref())
        {
            host_origin(state, &upgrade.origin)?;
            if upgrade.origin.session_id.is_none()
                || state
                    .encounter
                    .as_ref()
                    .and_then(|e| e.flow.as_ref())
                    .is_none_or(|f| {
                        f.phase != TacticalPhase::Finished
                            || f.version != TacticalExecutionVersion::ReleasedTimeV1.flow_version()
                    })
            {
                return Err("released upgrade is outside its Finished flow".into());
            }
        }
        let current = state
            .encounter
            .as_ref()
            .ok_or("completed encounter must remain attached until atomic replacement")?;
        let last = self.last().ok_or("completion history is empty")?;
        if current
            .flow
            .as_ref()
            .is_some_and(|flow| flow.phase == TacticalPhase::Finished)
            && current.id != last.encounter_id
        {
            return Err("Finished encounter is not the latest authenticated completion".into());
        }
        if self.contains_encounter(current.id) {
            if current.id != last.encounter_id
                || current.origin != last.encounter_origin
                || current.scene_id != last.scene_id
                || current.flow.as_ref().is_none_or(|flow| {
                    !matches_release_execution(flow, last)
                        || flow.phase != TacticalPhase::Finished
                        || flow.origin != last.initiative_origin
                        || flow
                            .aftermath
                            .as_ref()
                            .is_none_or(|aftermath| aftermath.origin != last.conclusion_origin)
                })
            {
                return Err("live encounter reuses a completed encounter identity".into());
            }
        } else if current.origin.expected_event_sequence <= last.released_by.expected_event_sequence
            || scenes.contains(&current.scene_id)
        {
            return Err(
                "replacement encounter predates its predecessor or reuses a retired scene".into(),
            );
        }
        if current.flow.as_ref().is_some_and(|flow| {
            flow.ground_items
                .iter()
                .any(|ground| !placed.insert(ground.item))
        }) {
            return Err("item is placed in both live and retired scene spaces".into());
        }
        Ok(())
    }
}

fn matches_release_execution(flow: &crate::TacticalFlow, receipt: &TacticalCompletion) -> bool {
    if flow.version == receipt.execution.flow_version() {
        return flow.released_time_upgrade.is_none();
    }
    flow.version == TacticalExecutionVersion::ReleasedTimeV1.flow_version()
        && receipt.execution == TacticalExecutionVersion::EncounterReleaseV1
        && flow.released_time_upgrade.as_ref().is_some_and(|upgrade| {
            upgrade.release == receipt.released_by.id
                && upgrade.from == TacticalExecutionVersion::EncounterReleaseV1
                && upgrade.to == TacticalExecutionVersion::ReleasedTimeV1
                && upgrade.origin.expected_event_sequence
                    > receipt.released_by.expected_event_sequence
        })
}
