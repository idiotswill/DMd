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
                || receipt.execution != TacticalExecutionVersion::EncounterReleaseV1
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
                    flow.version != TacticalExecutionVersion::EncounterReleaseV1.flow_version()
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
