//! Bounded current table configuration and unresolved intent, never transcript history.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    CampaignState, CharacterId, CharacterProfile, CommandId, CommandMeta, EntityId, HouseRules,
    PlaySessionId, PlayerId, RollRequestId, SessionParticipant, TestKind, VersionedRef,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TableContract {
    pub tone: String,
    pub humor: String,
    pub tactical_preference: String,
    pub exploration_preference: String,
    pub social_preference: String,
    pub lethality: String,
    pub ruleset: VersionedRef,
    pub permitted_content: Vec<VersionedRef>,
    pub advancement: String,
    pub optional_rules: Vec<String>,
    pub house_rules: HouseRules,
    pub house_rule_notes: String,
    pub pvp_policy: String,
    pub theft_and_secrets: String,
    pub retcon_policy: String,
    pub content_boundaries: String,
    pub explanation_depth: String,
    pub experience: String,
    pub absent_player_policy: String,
}

impl Default for TableContract {
    fn default() -> Self {
        Self {
            tone: "Grounded fantasy adventure".into(),
            humor: "Welcome when it fits the table".into(),
            tactical_preference: "Balanced".into(),
            exploration_preference: "Balanced".into(),
            social_preference: "Balanced".into(),
            lethality: "Consequences follow the selected rules".into(),
            ruleset: VersionedRef {
                id: "srd-5.2".into(),
                version: "5.2.1".into(),
            },
            permitted_content: vec![VersionedRef {
                id: "srd-5.2".into(),
                version: "5.2.1".into(),
            }],
            advancement: "Milestones agreed by the table".into(),
            optional_rules: vec![],
            house_rules: HouseRules::default(),
            house_rule_notes: "No additional mechanical house rules".into(),
            pvp_policy: "Requires the affected player's explicit consent".into(),
            theft_and_secrets: "Requires the affected player's explicit consent".into(),
            retcon_policy: "Correct uncommitted intent; preserve accepted history".into(),
            content_boundaries: "Discuss and record the table's boundaries before play".into(),
            explanation_depth: "Brief, with more detail on request".into(),
            experience: "Mixed experience".into(),
            absent_player_policy: "Absent characters take no voluntary actions".into(),
        }
    }
}

impl TableContract {
    pub fn validate(&self) -> Result<(), String> {
        for value in [
            &self.tone,
            &self.humor,
            &self.tactical_preference,
            &self.exploration_preference,
            &self.social_preference,
            &self.lethality,
            &self.advancement,
            &self.house_rule_notes,
            &self.pvp_policy,
            &self.theft_and_secrets,
            &self.retcon_policy,
            &self.content_boundaries,
            &self.explanation_depth,
            &self.experience,
            &self.absent_player_policy,
        ] {
            bounded_text(value, 4000)?;
        }
        if self.ruleset.id != "srd-5.2"
            || self.ruleset.version != "5.2.1"
            || self.permitted_content != vec![self.ruleset.clone()]
            || !self.optional_rules.is_empty()
        {
            return Err("unsupported table content or optional rule selection".into());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActiveTableSession {
    pub session_id: PlaySessionId,
    pub display_name: String,
    pub started_at_world: crate::WorldInstant,
    pub participants: Vec<SessionParticipant>,
}

impl ActiveTableSession {
    pub fn as_session(&self, campaign_id: crate::CampaignId) -> crate::PlaySession {
        crate::PlaySession {
            id: self.session_id,
            campaign_id,
            display_name: self.display_name.clone(),
            status: crate::PlaySessionStatus::Active,
            started_at_world: self.started_at_world,
            ended_at_world: None,
            participants: self.participants.clone(),
        }
    }
}

/// A player proposal. It never supplies a DC, modifier, authority or successful outcome.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum TableIntent {
    Check {
        kind: TestKind,
        goal: String,
        challenge_id: Option<String>,
    },
    SecondWind,
    Unresolved {
        question: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PendingTableDecision {
    pub id: CommandId,
    pub session_id: PlaySessionId,
    pub player_id: PlayerId,
    pub character_id: CharacterId,
    pub actor: EntityId,
    pub origin: CommandMeta,
    pub revision: u32,
    pub text: String,
    pub intent: TableIntent,
}

/// Links a table declaration to the single authoritative rules.pending request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TableRollContext {
    pub request_id: RollRequestId,
    pub session_id: PlaySessionId,
    pub player_id: PlayerId,
    pub character_id: CharacterId,
    pub actor: EntityId,
    pub challenge_id: Option<String>,
    pub declaration: String,
}

/// Host-established context. The player's text cannot create a challenge or change its DC.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TableChallenge {
    pub id: String,
    pub title: String,
    pub description: String,
    pub phrases: Vec<String>,
    pub kind: TestKind,
    pub dc: u16,
    pub success: String,
    pub failure: String,
    pub resolution: Option<ChallengeResolution>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChallengeResolution {
    pub actor: EntityId,
    pub request_id: RollRequestId,
    pub success: bool,
    pub total: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TableSituation {
    pub title: String,
    pub description: String,
    pub challenges: Vec<TableChallenge>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TableState {
    pub contract: TableContract,
    pub character_profiles: HashMap<CharacterId, CharacterProfile>,
    pub active_session: Option<ActiveTableSession>,
    pub pending: Option<PendingTableDecision>,
    pub roll_context: Option<TableRollContext>,
    pub situation: TableSituation,
    /// Absent keeps the original PC-only table presentation and transport semantics.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_actor_access: Option<crate::TableSourceActorAccess>,
}

impl TableState {
    #[must_use]
    pub fn new(contract: TableContract) -> Self {
        Self {
            contract,
            character_profiles: HashMap::new(),
            active_session: None,
            pending: None,
            roll_context: None,
            situation: TableSituation::default(),
            source_actor_access: None,
        }
    }

    pub fn validate(&self, state: &CampaignState) -> Result<(), String> {
        self.contract.validate()?;
        if let Some(access) = &self.source_actor_access {
            access.validate(state)?;
        }
        if self.contract.ruleset != state.campaign.ruleset {
            return Err("table and campaign rules identities disagree".into());
        }
        if state
            .rules
            .as_ref()
            .is_some_and(|rules| rules.house_rules != self.contract.house_rules)
        {
            return Err("table contract and applied house rules disagree".into());
        }
        for (id, profile) in &self.character_profiles {
            let character = state.characters.get(id).ok_or("unknown table character")?;
            if character.entity_id != profile.entity_id || character.display_name != profile.name {
                return Err("table character profile identity disagrees".into());
            }
            bounded_text(&profile.name, 200)?;
        }
        if let Some(session) = &self.active_session {
            bounded_text(&session.display_name, 200)?;
            if session.started_at_world > state.clock.now {
                return Err("table session starts after current time".into());
            }
            let candidate = session.as_session(state.campaign_id());
            if !candidate.validate_against_state(state).is_empty() {
                return Err("invalid current table session binding".into());
            }
            for participant in &session.participants {
                if let Some(character_id) = participant.character_id {
                    let pc = state
                        .characters
                        .get(&character_id)
                        .ok_or("unknown session character")?;
                    validate_owner(state, participant.player_id, character_id, pc.entity_id)?;
                }
            }
        }
        if self.pending.is_some() && self.roll_context.is_some() {
            return Err(
                "a table decision and a roll cannot both own the active declaration".into(),
            );
        }
        if let Some(pending) = &self.pending {
            validate_owner(
                state,
                pending.player_id,
                pending.character_id,
                pending.actor,
            )?;
            bounded_text(&pending.text, 8000)?;
            self.validate_attendance(pending.session_id, pending.player_id, pending.character_id)?;
            match &pending.intent {
                TableIntent::Check {
                    goal, challenge_id, ..
                } => {
                    bounded_text(goal, 8000)?;
                    self.validate_challenge_id(challenge_id)?;
                }
                TableIntent::Unresolved { question } => bounded_text(question, 4000)?,
                TableIntent::SecondWind => (),
            }
            if pending.origin.campaign_id != state.campaign_id()
                || pending.origin.expected_event_sequence > state.applied_event_sequence
                || pending.origin.issuer != crate::CommandIssuer::Player(pending.player_id)
                || pending.origin.actor != Some(crate::AgentRef::Entity(pending.actor))
                || pending.origin.session_id != Some(pending.session_id)
            {
                return Err("invalid table decision provenance".into());
            }
        }
        if let Some(context) = &self.roll_context {
            validate_owner(
                state,
                context.player_id,
                context.character_id,
                context.actor,
            )?;
            bounded_text(&context.declaration, 8000)?;
            self.validate_attendance(context.session_id, context.player_id, context.character_id)?;
            self.validate_challenge_id(&context.challenge_id)?;
            let pending = state
                .rules
                .as_ref()
                .and_then(|rules| rules.pending.as_ref())
                .ok_or("table roll context has no authoritative request")?;
            if pending.request.id != context.request_id
                || pending.request.roller != Some(context.actor)
                || pending.issued_by.session_id != Some(context.session_id)
                || pending.request.visibility != crate::RollVisibility::Public
            {
                return Err("table roll context points to a different request".into());
            }
            if let Some(id) = &context.challenge_id {
                let challenge = self
                    .situation
                    .challenges
                    .iter()
                    .find(|challenge| &challenge.id == id)
                    .ok_or("unknown pending challenge")?;
                if challenge.resolution.is_some()
                    || !matches!(&pending.purpose, crate::PendingPurpose::Test {kind,dc,..} if kind==&challenge.kind && *dc==i32::from(challenge.dc))
                {
                    return Err(
                        "table roll context disagrees with the established challenge".into(),
                    );
                }
            }
        }
        if self.situation.title.len() > 200
            || self.situation.description.len() > 16000
            || self.situation.challenges.len() > 100
        {
            return Err("table situation exceeds supported bounds".into());
        }
        let mut ids = std::collections::HashSet::new();
        for challenge in &self.situation.challenges {
            bounded_text(&challenge.id, 100)?;
            bounded_text(&challenge.title, 200)?;
            bounded_text(&challenge.description, 4000)?;
            bounded_text(&challenge.success, 4000)?;
            bounded_text(&challenge.failure, 4000)?;
            if !ids.insert(&challenge.id)
                || challenge.dc > 30
                || challenge.phrases.len() > 20
                || !matches!(challenge.kind, TestKind::Check { .. })
            {
                return Err("invalid table challenge".into());
            }
            for phrase in &challenge.phrases {
                bounded_text(phrase, 200)?;
            }
            if let Some(resolution) = &challenge.resolution
                && !state.entities.contains_key(&resolution.actor)
            {
                return Err("challenge resolution references a missing actor".into());
            }
        }
        Ok(())
    }

    fn validate_challenge_id(&self, id: &Option<String>) -> Result<(), String> {
        if let Some(id) = id
            && !self
                .situation
                .challenges
                .iter()
                .any(|challenge| &challenge.id == id)
        {
            return Err("declaration references an unknown challenge".into());
        }
        Ok(())
    }

    pub fn validate_attendance(
        &self,
        session_id: PlaySessionId,
        player_id: PlayerId,
        character_id: CharacterId,
    ) -> Result<(), String> {
        if !self.active_session.as_ref().is_some_and(|session| {
            session.session_id == session_id
                && session.participants.iter().any(|p| {
                    p.player_id == player_id
                        && p.character_id == Some(character_id)
                        && p.attendance == crate::AttendanceStatus::Present
                })
        }) {
            return Err("the character's controller must be present in the active session".into());
        }
        Ok(())
    }
}

fn validate_owner(
    state: &CampaignState,
    player: PlayerId,
    character: CharacterId,
    actor: EntityId,
) -> Result<(), String> {
    if !state.players.contains_key(&player)
        || !state
            .characters
            .get(&character)
            .is_some_and(|pc| pc.entity_id == actor && pc.controlling_player_id == Some(player))
    {
        return Err("table declaration owner or actor is inconsistent".into());
    }
    Ok(())
}

pub fn bounded_text(value: &str, maximum: usize) -> Result<(), String> {
    if value.trim().is_empty() || value.len() > maximum || value.contains('\0') {
        return Err("text is empty, too long or contains an invalid character".into());
    }
    Ok(())
}
