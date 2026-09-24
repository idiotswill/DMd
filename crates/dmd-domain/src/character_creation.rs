//! Source-backed character identity and creation records, not arbitrary mechanical totals.
use crate::{EntityId, Skill};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CharacterSize {
    Small,
    Medium,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FightingStyle {
    Defense,
    Archery,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GamingSet {
    Dice,
    Dragonchess,
    PlayingCards,
    ThreeDragonAnte,
}
impl GamingSet {
    pub const fn id(self) -> &'static str {
        match self {
            Self::Dice => "gaming-dice",
            Self::Dragonchess => "dragonchess",
            Self::PlayingCards => "playing-cards",
            Self::ThreeDragonAnte => "three-dragon-ante",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EquipmentChoice {
    pub item_id: String,
    pub quantity: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CharacterEquipment {
    pub item_id: String,
    pub display_name: String,
    pub quantity: u16,
    pub unit_cost_cp: u32,
    pub source_page: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CharacterFeatureGrant {
    pub id: String,
    pub source_page: u16,
    /// Source grant is present; this records the gate owning complete execution.
    pub execution_gate: u8,
    pub scope: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CharacterProfile {
    pub entity_id: EntityId,
    pub name: String,
    pub pronouns: String,
    pub description: String,
    pub alignment: String,
    /// Player-authored narrative. Never accepted as world facts by character creation.
    pub backstory: String,
    pub class_id: String,
    pub species_id: String,
    pub background_id: String,
    pub level: u8,
    pub experience_points: u32,
    pub size: CharacterSize,
    pub speed_feet: u16,
    pub base_ability_scores: [u8; 6],
    pub background_boosts: [u8; 6],
    pub fighter_skills: [Skill; 2],
    pub human_skill: Skill,
    pub skilled_skills: [Skill; 3],
    pub languages: Vec<String>,
    pub fighting_style: FightingStyle,
    pub features: Vec<CharacterFeatureGrant>,
    pub masteries: [String; 3],
    pub tool_proficiencies: Vec<String>,
    pub armor_training: Vec<String>,
    pub weapon_proficiencies: Vec<String>,
    pub equipment: Vec<CharacterEquipment>,
    pub worn_armor: Option<String>,
    pub shield: bool,
    pub money_cp: u32,
}

/// Opt-in typed level-one feature grants. Absent on unchanged historical imported sheets.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CharacterFeatureState {
    pub fighter_level: u8,
    pub fighting_style: FightingStyle,
    pub wearing_armor: bool,
    pub human_resourceful: bool,
    pub savage_attacker: bool,
    pub second_wind_remaining: u8,
    /// A source-triggered optional transfer awaits this PC's controller, not a guessed recipient.
    pub inspiration_transfer_pending: bool,
    /// Combat turn used, cleared when combat ends. No free repeated use in the same turn.
    pub savage_attacker_turn: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DamageRollChoice {
    First,
    Second,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SavageInspiration {
    pub roll: DamageRollChoice,
    pub die_index: usize,
    pub replacement: crate::DieResult,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SavageAttackerRoll {
    pub first: crate::RollResult,
    pub second: crate::RollResult,
    pub chosen: DamageRollChoice,
    pub inspiration: Option<SavageInspiration>,
}
