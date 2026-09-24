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
    CreateCreature {
        creation: Box<TableCreatureCreation>,
    },
    Tactical {
        action: dmd_rules::tactical::TacticalAction,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tactical_event: Option<dmd_rules::tactical::TacticalEvent>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum TableViewer {
    Host,
    Player(PlayerId),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TableCreatureCreation {
    pub entity_id: EntityId,
    pub name: String,
    pub definition_id: String,
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
    pub tactical: Option<TableTacticalView>,
    pub creature_setup: Option<TableCreatureSetupView>,
    pub situation_title: String,
    pub situation_description: String,
    pub transcript: Vec<TableTranscriptEntry>,
    pub recap: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TableCreatureSetupView {
    pub catalog: Vec<TableCreatureOption>,
    pub creatures: Vec<TableCreatureView>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TableCreatureOption {
    pub definition_id: String,
    pub name: String,
    pub sizes: Vec<CreatureSize>,
    pub additional_languages: u8,
    pub ammunition_required: bool,
    pub item_count: usize,
    pub abilities: Vec<String>,
    pub omitted_features: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TableCreatureView {
    pub actor: EntityId,
    pub name: String,
    pub definition_id: String,
    pub size: CreatureSize,
    pub hp: u32,
    pub max_hp: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TableTacticalView {
    pub encounter_id: EncounterId,
    pub round: Option<u32>,
    pub active_actor: Option<EntityId>,
    pub phase: String,
    /// Full map geometry and participant truth are local-host-only.
    pub battlefield: Option<Battlefield>,
    pub participants: Vec<TacticalParticipant>,
    pub combatant_sources: Vec<TableCombatantSource>,
    pub observers: Vec<dmd_rules::spatial::ActorTacticalView>,
    pub initiative: Vec<TableInitiativeView>,
    pub ties: Vec<InitiativeTie>,
    pub budget: Option<TableTacticalBudget>,
    /// Only the controlling viewer or host receives the current ordered-work choice.
    pub continuation: Option<TableTacticalContinuation>,
    /// A saving throw's controller may choose failure before reporting any dice.
    pub may_fail_save: Option<EntityId>,
    pub legendary_resistance: Option<EntityId>,
    pub legendary_action: Option<EntityId>,
    /// Physical choices for an authorized current actor; no target combat statistics.
    pub attack_options: Option<TableAttackOptions>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TableAttackOptions {
    pub actor: EntityId,
    pub hands: WeaponLoadout,
    pub weapons: Vec<TableWeaponChoice>,
    pub targets: Vec<TableAttackTarget>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TableWeaponChoice {
    pub item: ItemId,
    pub name: String,
    pub deliveries: Vec<WeaponDelivery>,
    pub abilities: Vec<Ability>,
    pub grips: Vec<WeaponGrip>,
    pub ammunition_required: bool,
    pub ammunition: Vec<TableItemView>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TableAttackTarget {
    pub actor: EntityId,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TableCombatantSource {
    pub actor: EntityId,
    pub source: TacticalSource,
    pub initiative_modifier: i32,
    pub normal_mode: RollMode,
    pub surprised_mode: RollMode,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TableTacticalContinuation {
    pub actor: EntityId,
    pub host_adjudication: bool,
    pub choices: Vec<TableTacticalWorkChoice>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TableTacticalWorkChoice {
    pub occurrence: u16,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TableTacticalBudget {
    pub movement_spent: u32,
    pub attacks_remaining: u8,
    pub action_spent: bool,
    pub bonus_action_spent: bool,
    pub reaction_available: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TableInitiativeView {
    pub actor: EntityId,
    pub label: String,
    pub total: Option<i32>,
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
