//! Pure typed table operations. Application envelopes and language stay in dmd-app.
use crate::{CharacterCreationInput, RulesEvent, RulesOutcome};
use dmd_domain::*;
use serde::{Deserialize, Serialize};
pub(crate) mod execution;
pub mod reducer;
pub use execution::{
    AppliedTableStep, CampaignExecution, LegacyRulesEventRef, PreparedTableStep, TableRead,
};
mod battlefield;
mod creatures;
mod equipment;
mod grapple_access;
mod inspiration;
pub use inspiration::award_ruling as inspiration_award_ruling;
pub mod source_control;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TableOperation {
    AwardHeroicInspiration {
        character_id: CharacterId,
        reason: String,
    },
    EnableGrappleAccess,
    EnableGrappleTransport,
    UpdateContract {
        contract: TableContract,
    },
    AddPlayer {
        id: PlayerId,
        name: String,
    },
    CreateCharacter {
        character_id: CharacterId,
        entity_id: EntityId,
        player_id: PlayerId,
        input: CharacterCreationInput,
    },
    PrepareEquipment {
        character_id: CharacterId,
        item_ids: Vec<ItemId>,
    },
    CreateCreature {
        creation: Box<TableCreatureCreation>,
    },
    /// Explicitly adopt the complete existing source-owner set at a settled boundary.
    EnableSourceActorAccess {
        adopted: Vec<TableSourceAdoption>,
    },
    SetSourceCreatureController {
        actor: EntityId,
        controller: CreatureController,
    },
    Tactical {
        action: crate::tactical::TacticalAction,
    },
    PrepareBattlefield {
        setup: Box<TableBattlefieldSetup>,
    },
    StartSession {
        id: PlaySessionId,
        name: String,
        participants: Vec<SessionParticipant>,
    },
    EndSession,
    SetSituation {
        situation: TableSituation,
    },
    Declare {
        text: String,
        intent: TableIntent,
    },
    Correct {
        pending_id: CommandId,
        revision: u32,
        text: String,
        intent: TableIntent,
    },
    CancelDecision {
        pending_id: CommandId,
        revision: u32,
    },
    Adjudicate {
        pending_id: CommandId,
        revision: u32,
        request_id: RollRequestId,
    },
    SubmitPhysical {
        request_id: RollRequestId,
        faces: Vec<u16>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TableSessionChange {
    Start {
        session: PlaySession,
    },
    Replace {
        expected: PlaySession,
        next: PlaySession,
    },
}

/// Expected source children from the original app envelope; absence of both still means replay.
#[derive(Clone, Copy)]
pub struct ExpectedNested<'a> {
    pub rules_event: Option<&'a RulesEvent>,
    pub tactical_event: Option<&'a crate::tactical::TacticalEvent>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableProduced {
    pub message: String,
    pub mechanics: Option<RulesOutcome>,
    pub rules_event: Option<RulesEvent>,
    pub tactical_event: Option<crate::tactical::TacticalEvent>,
    pub session_change: Option<TableSessionChange>,
}
pub struct TableOperationTransition {
    pub state: CampaignState,
    pub produced: TableProduced,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TableCreatureCreation {
    pub entity_id: EntityId,
    pub name: String,
    pub definition_id: String,
    /// Absence is frozen V1 replay, never a current-catalog default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<CreatureSourcePin>,
    pub size: CreatureSize,
    pub additional_languages: Vec<String>,
    pub ammunition_units: u16,
    pub item_ids: Vec<ItemId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TableBattlefieldSetup {
    pub encounter_id: EncounterId,
    pub scene_id: SceneId,
    pub location_id: LocationId,
    pub name: String,
    pub battlefield: Battlefield,
    pub characters: Vec<TableCharacterPlacement>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub creatures: Vec<TableCreaturePlacement>,
    pub geometry_ruling: Ruling,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub area_grid_policy: Option<TacticalAreaGridPolicy>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TableCreaturePlacement {
    pub actor: EntityId,
    pub public_label: String,
    pub position: SpatialPoint,
    pub height: u32,
    pub allies: Vec<EntityId>,
    pub enemies: Vec<EntityId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TableCharacterPlacement {
    pub character_id: CharacterId,
    pub position: SpatialPoint,
    /// Explicit physical geometry, measured in half-feet; not a mechanical bonus.
    pub height: u32,
    pub allies: Vec<EntityId>,
    pub enemies: Vec<EntityId>,
}

/// A protocol marker is only a routing fact. It never creates a closed read.
pub fn grapple_enabled(state: &CampaignState) -> bool {
    state
        .table
        .as_ref()
        .is_some_and(|table| table.grapple_access.is_some())
}

pub fn grapple_action(choice: &TableGrappleChoice) -> crate::tactical::TacticalAction {
    crate::tactical::grapple::table_action(choice)
}

pub fn grapple_transport_enabled(state: &CampaignState) -> bool {
    state
        .table
        .as_ref()
        .and_then(|t| t.grapple_access.as_ref())
        .is_some_and(|access| access.ground_transport.is_some())
}
