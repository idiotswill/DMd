//! Serializable authoritative mechanics. Resolution belongs to dmd-rules.
use crate::{CommandMeta, EffectId, EntityId, ResolvedRoll, RollRequest, RollResult, WorldInstant};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashMap};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Ability {
    Strength,
    Dexterity,
    Constitution,
    Intelligence,
    Wisdom,
    Charisma,
}
impl Ability {
    pub const ALL: [Self; 6] = [
        Self::Strength,
        Self::Dexterity,
        Self::Constitution,
        Self::Intelligence,
        Self::Wisdom,
        Self::Charisma,
    ];
    pub const fn index(self) -> usize {
        self as usize
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Skill {
    Acrobatics,
    AnimalHandling,
    Arcana,
    Athletics,
    Deception,
    History,
    Insight,
    Intimidation,
    Investigation,
    Medicine,
    Nature,
    Perception,
    Performance,
    Persuasion,
    Religion,
    SleightOfHand,
    Stealth,
    Survival,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Proficiency {
    Proficient,
    Expertise,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum ArmorClass {
    Fixed(u16),
    HeavyArmor {
        base: u16,
        shield: bool,
    },
    Armor {
        base: u16,
        dexterity_cap: Option<i16>,
        shield: bool,
    },
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum DamageType {
    Acid,
    Bludgeoning,
    Cold,
    Fire,
    Force,
    Lightning,
    Necrotic,
    Piercing,
    Poison,
    Psychic,
    Radiant,
    Slashing,
    Thunder,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Condition {
    Blinded,
    Charmed,
    Deafened,
    Frightened,
    Grappled,
    Incapacitated,
    Invisible,
    Paralyzed,
    Petrified,
    Poisoned,
    Prone,
    Restrained,
    Stunned,
    Unconscious,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Recovery {
    ShortOrLongRest,
    LongRest,
    Never,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResourcePool {
    pub maximum: u16,
    pub remaining: u16,
    pub recovery: Recovery,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HitDice {
    pub sides: u16,
    pub maximum: u8,
    pub remaining: u8,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Spellcasting {
    pub ability: Ability,
    pub slot_maxima: [u8; 9],
    pub slots: [u8; 9],
    pub can_speak: bool,
    pub free_hand: bool,
    pub material_focus: bool,
}
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeathState {
    pub successes: u8,
    pub failures: u8,
    pub stable: bool,
    pub dead: bool,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MechanicalEntity {
    pub entity_id: EntityId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub character_features: Option<crate::CharacterFeatureState>,
    pub level: u8,
    pub ability_scores: [u8; 6],
    pub armor: ArmorClass,
    pub max_hp: u32,
    pub hp: u32,
    pub temporary_hp: u32,
    pub hit_dice: HitDice,
    pub death: DeathState,
    pub uses_death_saves: bool,
    pub exhaustion: u8,
    pub heroic_inspiration: bool,
    pub prone: bool,
    pub saving_proficiencies: BTreeSet<Ability>,
    pub skill_proficiencies: BTreeMap<Skill, Proficiency>,
    pub attacks: BTreeSet<String>,
    pub attack_proficiencies: BTreeSet<String>,
    pub prepared_spells: BTreeSet<String>,
    pub spellcasting: Option<Spellcasting>,
    pub resources: BTreeMap<String, ResourcePool>,
    pub resistances: BTreeSet<DamageType>,
    pub vulnerabilities: BTreeSet<DamageType>,
    pub damage_immunities: BTreeSet<DamageType>,
    pub condition_immunities: BTreeSet<Condition>,
    pub concentration: Option<EffectId>,
    pub last_long_rest_finished: Option<WorldInstant>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TurnBoundary {
    Start,
    End,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Expiry {
    Never,
    AtTime(WorldInstant),
    AtTurn {
        actor: EntityId,
        boundary: TurnBoundary,
        turn_number: u64,
    },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActiveEffect {
    pub id: EffectId,
    pub source: EntityId,
    pub target: EntityId,
    pub condition: Option<Condition>,
    pub label: String,
    pub expires: Expiry,
    pub concentration_owner: Option<EntityId>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InitiativeEntry {
    pub actor: EntityId,
    pub total: i32,
    pub tie_break: u16,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CombatTiming {
    pub order: Vec<InitiativeEntry>,
    pub index: usize,
    pub round: u32,
    pub turn_number: u64,
    pub action_spent: bool,
    pub bonus_action_spent: bool,
    pub slot_spent_this_turn: bool,
    pub reactions_spent: Vec<EntityId>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RestKind {
    Short,
    Long,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RestProgress {
    pub actor: EntityId,
    pub kind: RestKind,
    pub started_at: WorldInstant,
}
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HouseRules {
    pub ability_test_natural_extremes: bool,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum RulingBasis {
    Srd { page: u16 },
    HouseRule { id: String },
    GmAdjudication,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Ruling {
    pub basis: RulingBasis,
    pub reason: String,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Circumstances {
    pub advantage: bool,
    pub disadvantage: bool,
    /// A hostile creature within 5 feet can see the attacker and is not incapacitated.
    pub ranged_threat: bool,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum TestKind {
    Check {
        ability: Ability,
        skill: Option<Skill>,
    },
    Save {
        ability: Ability,
    },
    Initiative,
    DeathSave,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum PendingPurpose {
    Test {
        kind: TestKind,
        dc: i32,
        circumstances: Circumstances,
    },
    Attack {
        target: EntityId,
        damage: Vec<crate::DieSpec>,
        damage_modifier: i32,
        damage_type: DamageType,
        automatic_critical: bool,
        permission: ActionPermission,
    },
    Damage {
        target: EntityId,
        damage_type: DamageType,
        critical: bool,
        attack_roll_id: crate::RollRequestId,
    },
    Healing {
        target: EntityId,
        spell_id: String,
        slot_level: u8,
        permission: ActionPermission,
    },
    Concentration {
        dc: i32,
        damage_taken: u32,
    },
    RestHitDie,
    SecondWind,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PendingRoll {
    pub issued_by: CommandMeta,
    pub request: RollRequest,
    pub purpose: PendingPurpose,
    pub ruling: Ruling,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecordedRoll {
    pub issued_by: CommandMeta,
    pub accepted_by: CommandMeta,
    pub original_result: Option<RollResult>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub savage_attacker: Option<crate::SavageAttackerRoll>,
    pub request: RollRequest,
    pub result: RollResult,
    pub resolved: ResolvedRoll,
    pub purpose: PendingPurpose,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RulingRecord {
    pub command: CommandMeta,
    pub ruling: Ruling,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActionPermission {
    pub issued_by: CommandMeta,
    pub actor: EntityId,
    pub target: EntityId,
    pub content_id: String,
    pub spell: bool,
    pub circumstances: Circumstances,
    pub within_five_feet: bool,
    pub ruling: Ruling,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RulesState {
    pub pack_id: String,
    pub pack_version: String,
    pub entities: HashMap<EntityId, MechanicalEntity>,
    pub house_rules: HouseRules,
    pub effects: Vec<ActiveEffect>,
    pub pending: Option<PendingRoll>,
    pub rolls: Vec<RecordedRoll>,
    pub cancelled_roll_ids: Vec<crate::RollRequestId>,
    pub rulings: Vec<RulingRecord>,
    pub timing: Option<CombatTiming>,
    pub rests: Vec<RestProgress>,
    pub completed_short_rests: Vec<EntityId>,
    /// One-use authoritative context, invalidated by every unrelated mechanics action.
    pub permission: Option<ActionPermission>,
}

impl MechanicalEntity {
    /// Neutral validated starting sheet for an existing world entity. Character creation and
    /// progression features are applied by trusted typed import/adjudication, not this default.
    pub fn basic(entity_id: EntityId) -> Self {
        Self {
            entity_id,
            character_features: None,
            level: 1,
            ability_scores: [10; 6],
            armor: ArmorClass::Armor {
                base: 10,
                dexterity_cap: None,
                shield: false,
            },
            max_hp: 8,
            hp: 8,
            temporary_hp: 0,
            hit_dice: HitDice {
                sides: 8,
                maximum: 1,
                remaining: 1,
            },
            death: DeathState::default(),
            uses_death_saves: true,
            exhaustion: 0,
            heroic_inspiration: false,
            prone: false,
            saving_proficiencies: BTreeSet::new(),
            skill_proficiencies: BTreeMap::new(),
            attacks: BTreeSet::new(),
            attack_proficiencies: BTreeSet::new(),
            prepared_spells: BTreeSet::new(),
            spellcasting: None,
            resources: BTreeMap::new(),
            resistances: BTreeSet::new(),
            vulnerabilities: BTreeSet::new(),
            damage_immunities: BTreeSet::new(),
            condition_immunities: BTreeSet::new(),
            concentration: None,
            last_long_rest_finished: None,
        }
    }
}
