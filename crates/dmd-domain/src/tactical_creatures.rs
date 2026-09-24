//! Source identity and limited-use history for creatures, independent of encounters.
//! Source reconstruction and execution belong to rules; journal authentication to app.
use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::{
    AgentRef, CampaignState, CommandIssuer, CommandMeta, CreatureSize, EncounterId, EntityId,
    PlayerId, RestKind, RollRequest, RollResult, TurnBoundary, WorldInstant,
};

pub const TACTICAL_CREATURES_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreatureSourcePin {
    pub ruleset_id: String,
    pub ruleset_version: String,
    pub definition_id: String,
    /// FNV1a64 over the typed canonical definition JSON, a content identity, not a signature.
    pub definition_fingerprint: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum CreatureHitPointOrigin {
    Average,
    Rolled {
        request: RollRequest,
        result: RollResult,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CreatureController {
    Autonomous,
    Host,
    Player(PlayerId),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreatureProfile {
    pub actor: EntityId,
    pub origin: CommandMeta,
    pub source: CreatureSourcePin,
    pub size: CreatureSize,
    pub additional_languages: Vec<String>,
    pub hit_points: CreatureHitPointOrigin,
}

/// A reference to the authoritative central turn, never a second turn counter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreatureTurn {
    pub encounter_id: EncounterId,
    pub actor: EntityId,
    pub number: u64,
    pub boundary: TurnBoundary,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreatureRechargeTicket {
    pub origin: CommandMeta,
    pub turn: CreatureTurn,
    pub request: RollRequest,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreatureRechargeRecord {
    pub ticket: CreatureRechargeTicket,
    pub accepted_by: CommandMeta,
    pub result: RollResult,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreatureRechargeState {
    pub feature_id: String,
    pub available: bool,
    pub pending: Option<CreatureRechargeTicket>,
    pub last_roll: Option<CreatureRechargeRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreatureLimitedUse {
    pub feature_id: String,
    pub spell_id: Option<String>,
    pub spent: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CreatureSimpleAction {
    Dash,
    Disengage,
    Dodge,
    Hide,
}

/// Selects only source-defined capabilities. Contains no caller modifiers/DCs/damage.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreatureFeatureSelection {
    pub feature_id: String,
    pub spell_id: Option<String>,
    pub simple_action: Option<CreatureSimpleAction>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreatureRoutineStep {
    /// Source slot identity; the chosen order need not be source listing order.
    pub slot: u8,
    pub selection: CreatureFeatureSelection,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreatureRoutineContinuation {
    pub origin: CommandMeta,
    pub turn: CreatureTurn,
    pub feature_id: String,
    pub steps: Vec<CreatureRoutineStep>,
    pub next_step: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreatureRestReceipt {
    /// The already accepted completed-rest command, authenticated by application replay.
    pub origin: CommandMeta,
    pub kind: RestKind,
    pub finished_at: WorldInstant,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreatureRuntime {
    pub actor: EntityId,
    pub controller: CreatureController,
    pub control_origin: CommandMeta,
    pub in_lair: bool,
    pub lair_origin: CommandMeta,
    pub recharge: Vec<CreatureRechargeState>,
    pub limited_uses: Vec<CreatureLimitedUse>,
    /// Reset at this creature's own start, not each global turn or encounter load.
    pub used_this_own_turn: Vec<String>,
    pub legendary_spent: u8,
    pub legendary_resistance_spent: u8,
    /// Last source hook observed, used only for idempotence/ordering of creature work.
    pub observed_turn: Option<CreatureTurn>,
    /// Only one Legendary Action at an eligible other-creature end boundary.
    pub legendary_window_spent: bool,
    pub routine: Option<CreatureRoutineContinuation>,
    pub last_rest: Option<CreatureRestReceipt>,
    pub last_operation: CommandMeta,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TacticalCreatures {
    pub schema_version: u32,
    pub profiles: Vec<CreatureProfile>,
    pub runtime: Vec<CreatureRuntime>,
}
impl Default for TacticalCreatures {
    fn default() -> Self {
        Self {
            schema_version: TACTICAL_CREATURES_SCHEMA_VERSION,
            profiles: vec![],
            runtime: vec![],
        }
    }
}
impl TacticalCreatures {
    pub fn profile(&self, actor: EntityId) -> Option<&CreatureProfile> {
        self.profiles.iter().find(|profile| profile.actor == actor)
    }
    pub fn runtime(&self, actor: EntityId) -> Option<&CreatureRuntime> {
        self.runtime.iter().find(|runtime| runtime.actor == actor)
    }

    /// Structural validation. Also run source/rules validation before making a save runnable.
    pub fn validate(&self, state: &CampaignState) -> Result<(), String> {
        if self.schema_version != TACTICAL_CREATURES_SCHEMA_VERSION
            || self.profiles.len() > state.entities.len()
            || self.runtime.len() != self.profiles.len()
        {
            return Err("invalid creature authority version/count".into());
        }
        let mut actors = HashSet::new();
        for profile in &self.profiles {
            if !actors.insert(profile.actor)
                || !valid_label(&profile.source.definition_id)
                || profile.source.ruleset_id != state.campaign.ruleset.id
                || profile.source.ruleset_version != state.campaign.ruleset.version
                || profile.source.definition_fingerprint.len() != 16
                || !profile
                    .source
                    .definition_fingerprint
                    .bytes()
                    .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
                || profile.additional_languages.len() > 32
                || profile
                    .additional_languages
                    .iter()
                    .any(|language| !valid_label(language))
            {
                return Err("invalid creature profile/source identity".into());
            }
            validate_creature_origin(state, &profile.origin, profile.actor)?;
            if !matches!(
                profile.origin.issuer,
                CommandIssuer::System | CommandIssuer::Admin
            ) || profile
                .origin
                .actor
                .is_some_and(|actor| actor != AgentRef::Entity(profile.actor))
            {
                return Err(
                    "creature source creation requires trusted system/host provenance".into(),
                );
            }
        }
        let mut runtime_actors = HashSet::new();
        for runtime in &self.runtime {
            if !actors.contains(&runtime.actor)
                || !runtime_actors.insert(runtime.actor)
                || runtime.recharge.len() > 64
                || runtime.limited_uses.len() > 128
                || runtime.used_this_own_turn.len() > 64
            {
                return Err("invalid creature runtime identity/count".into());
            }
            for origin in [
                &runtime.control_origin,
                &runtime.lair_origin,
                &runtime.last_operation,
            ] {
                validate_creature_origin(state, origin, runtime.actor)?;
            }
            if let CreatureController::Player(id) = runtime.controller
                && !state.players.contains_key(&id)
            {
                return Err("unknown creature controller".into());
            }
            if let Some(turn) = runtime.observed_turn {
                validate_turn_reference(state, turn)?;
            }
            for recharge in &runtime.recharge {
                if !valid_label(&recharge.feature_id) {
                    return Err("invalid recharge feature".into());
                }
                if let Some(ticket) = &recharge.pending {
                    validate_creature_origin(state, &ticket.origin, runtime.actor)?;
                    validate_turn_reference(state, ticket.turn)?;
                }
                if let Some(record) = &recharge.last_roll {
                    validate_creature_origin(state, &record.ticket.origin, runtime.actor)?;
                    validate_creature_origin(state, &record.accepted_by, runtime.actor)?;
                    validate_turn_reference(state, record.ticket.turn)?;
                }
            }
            if let Some(routine) = &runtime.routine {
                validate_creature_origin(state, &routine.origin, runtime.actor)?;
                validate_turn_reference(state, routine.turn)?;
                if routine.steps.is_empty()
                    || routine.steps.len() > 20
                    || usize::from(routine.next_step) >= routine.steps.len()
                    || !valid_label(&routine.feature_id)
                    || routine.steps.iter().any(|step| {
                        !valid_label(&step.selection.feature_id)
                            || step
                                .selection
                                .spell_id
                                .as_ref()
                                .is_some_and(|id| !valid_label(id))
                    })
                {
                    return Err("invalid pending source routine".into());
                }
            }
            if let Some(rest) = &runtime.last_rest {
                validate_creature_origin(state, &rest.origin, runtime.actor)?;
                if rest.finished_at > state.clock.now {
                    return Err("creature rest is in the future".into());
                }
            }
        }
        Ok(())
    }
}

fn valid_label(value: &str) -> bool {
    !value.trim().is_empty() && value.len() <= 160 && !value.chars().any(char::is_control)
}
fn validate_turn_reference(state: &CampaignState, turn: CreatureTurn) -> Result<(), String> {
    if turn.encounter_id.0.is_nil() || turn.number == 0 || !state.entities.contains_key(&turn.actor)
    {
        return Err("invalid retained creature turn reference".into());
    }
    Ok(())
}
pub fn validate_creature_origin(
    state: &CampaignState,
    origin: &CommandMeta,
    actor: EntityId,
) -> Result<(), String> {
    if actor.0.is_nil()
        || !state
            .entities
            .get(&actor)
            .is_some_and(|entity| entity.id == actor && entity.campaign_id == state.campaign_id())
        || origin.id.0.is_nil()
        || origin.campaign_id != state.campaign_id()
        || origin.expected_event_sequence > state.applied_event_sequence
        || origin.session_id.is_some_and(|id| id.0.is_nil())
        || origin.actor.is_some_and(|subject| match subject {
            AgentRef::Entity(id) => !state.entities.contains_key(&id),
            AgentRef::Faction(_) => true,
        })
        || matches!(origin.issuer, CommandIssuer::Import)
        || matches!(origin.issuer, CommandIssuer::Player(id) if !state.players.contains_key(&id) || origin.actor.is_none())
    {
        return Err("invalid retained creature command provenance".into());
    }
    Ok(())
}
