//! Application-defined commands: language supplies proposals, the input channel supplies authority.
use dmd_domain::*;
use dmd_rules::{CharacterCreationInput, RulesEvent, RulesOutcome};
use serde::{Deserialize, Serialize};

pub const TABLE_EVENT_KIND: &str = "table.action_resolved";
pub const TABLE_EVENT_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum TableAction {
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
    },
    Correct {
        pending_id: CommandId,
        revision: u32,
        text: String,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TableOutcome {
    /// A player-safe account of accepted table activity. Never raw hidden context.
    pub message: String,
    pub mechanics: Option<RulesOutcome>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TableEvent {
    pub meta: CommandMeta,
    pub action: TableAction,
    pub outcome: TableOutcome,
    pub rules_event: Option<RulesEvent>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum TableViewer {
    Host,
    Player(PlayerId),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TableCampaignSummary {
    pub id: CampaignId,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TableCharacterView {
    pub character_id: CharacterId,
    pub player_id: Option<PlayerId>,
    pub entity_id: EntityId,
    pub name: String,
    /// Private character details are present only for the controller or local host.
    pub profile: Option<CharacterProfile>,
    pub sheet: Option<dmd_rules::RulesAnswer>,
    pub second_wind_remaining: Option<u8>,
    pub details: Option<TableSheetDetails>,
    pub equipment: Option<TableEquipmentView>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TableEquipmentView {
    pub prepared: bool,
    pub initial_item_count: usize,
    pub items: Vec<TableItemView>,
    pub worn_armor: Option<ItemId>,
    pub shield: Option<ItemId>,
    pub hands: WeaponLoadout,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TableItemView {
    pub id: ItemId,
    pub name: String,
    pub quantity: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TableSheetDetails {
    pub ability_scores: [u8; 6],
    pub hit_dice: HitDice,
    pub heroic_inspiration: bool,
    pub saving_throws: Vec<TableSaveBonus>,
    pub skills: Vec<TableSkillBonus>,
    pub conditions: Vec<Condition>,
    pub exhaustion: u8,
    pub death: DeathState,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TableSaveBonus {
    pub ability: Ability,
    pub modifier: i32,
    pub proficient: bool,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TableSkillBonus {
    pub skill: Skill,
    pub ability: Ability,
    pub modifier: i32,
    pub proficiency: Option<Proficiency>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TableTranscriptEntry {
    pub id: String,
    pub event_sequence: u64,
    pub kind: String,
    pub speaker: String,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TableView {
    pub campaign_id: CampaignId,
    pub name: String,
    pub event_sequence: u64,
    pub contract: TableContract,
    pub players: Vec<Player>,
    pub characters: Vec<TableCharacterView>,
    pub active_session: Option<ActiveTableSession>,
    pub pending: Option<PendingTableDecision>,
    pub roll: Option<RollRequest>,
    pub situation_title: String,
    pub situation_description: String,
    pub transcript: Vec<TableTranscriptEntry>,
    pub recap: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TableObservationBody {
    pub meta: CommandMeta,
    pub text: String,
    pub answer: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TableTextResult {
    Accepted(TableReceipt),
    Observed(TableObservationBody),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CharacterCreationOptions {
    pub catalog: dmd_rules::StarterCatalog,
    pub fighter_skills: Vec<Skill>,
    pub fighter_masteries: Vec<String>,
    pub standard_languages: Vec<String>,
    pub alignments: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TableReceipt {
    pub command_id: CommandId,
    pub event_sequence: u64,
    pub outcome: TableOutcome,
    pub already_accepted: bool,
}
