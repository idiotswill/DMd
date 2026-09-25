//! Pure SRD 5.2.1 mechanical transitions. No persistence, providers, or ambient randomness.
mod definitions;
mod engine;
mod request_integrity;
mod validation;
pub use definitions::*;
use dmd_domain::*;
pub(crate) use engine::interrupt_rest;
pub use engine::{query, replay, resolve};
use serde::{Deserialize, Serialize};
use thiserror::Error;
pub(crate) use validation::savage_result;
pub use validation::{ability_modifier, armor_class, proficiency_bonus, validate_state};
pub use validation::{check_modifier as test_modifier, conditions as active_conditions};

/// SRD 5.2.1 standard skill abilities; a host may establish a different ability for a check.
pub const STANDARD_SKILL_ABILITIES: [(Skill, Ability); 18] = [
    (Skill::Acrobatics, Ability::Dexterity),
    (Skill::AnimalHandling, Ability::Wisdom),
    (Skill::Arcana, Ability::Intelligence),
    (Skill::Athletics, Ability::Strength),
    (Skill::Deception, Ability::Charisma),
    (Skill::History, Ability::Intelligence),
    (Skill::Insight, Ability::Wisdom),
    (Skill::Intimidation, Ability::Charisma),
    (Skill::Investigation, Ability::Intelligence),
    (Skill::Medicine, Ability::Wisdom),
    (Skill::Nature, Ability::Intelligence),
    (Skill::Perception, Ability::Wisdom),
    (Skill::Performance, Ability::Charisma),
    (Skill::Persuasion, Ability::Charisma),
    (Skill::Religion, Ability::Intelligence),
    (Skill::SleightOfHand, Ability::Dexterity),
    (Skill::Stealth, Ability::Dexterity),
    (Skill::Survival, Ability::Wisdom),
];

pub const RULES_EVENT_KIND: &str = "rules.action_resolved";
pub const RULES_EVENT_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum RulesAction {
    CreateCharacter {
        entity_id: EntityId,
        input: crate::CharacterCreationInput,
    },
    Initialize {
        entities: Vec<MechanicalEntity>,
        house_rules: HouseRules,
        ruling: Ruling,
    },
    RequestTest {
        actor: EntityId,
        kind: TestKind,
        dc: i32,
        visibility: RollVisibility,
        circumstances: Circumstances,
        ruling: Ruling,
        request_id: RollRequestId,
    },
    /// Privileged contextual permission records range/visibility/etc until Gate 4 spatial truth.
    AuthorizeAttack {
        actor: EntityId,
        target: EntityId,
        attack_id: String,
        circumstances: Circumstances,
        within_five_feet: bool,
        ruling: Ruling,
    },
    Attack {
        actor: EntityId,
        target: EntityId,
        attack_id: String,
        request_id: RollRequestId,
    },
    AuthorizeSpell {
        actor: EntityId,
        target: EntityId,
        spell_id: String,
        circumstances: Circumstances,
        within_five_feet: bool,
        ruling: Ruling,
    },
    CastSpell {
        actor: EntityId,
        target: EntityId,
        spell_id: String,
        slot_level: u8,
        request_id: RollRequestId,
        effect_id: EffectId,
    },
    SubmitRoll {
        result: RollResult,
    },
    SecondWind {
        actor: EntityId,
        request_id: RollRequestId,
    },
    ResolveInspirationTransfer {
        actor: EntityId,
        recipient: Option<EntityId>,
    },
    SubmitSavageAttacker {
        roll: SavageAttackerRoll,
    },
    SubmitRollWithInspiration {
        result: RollResult,
        die_index: usize,
        replacement: DieResult,
    },
    GrantInspiration {
        actor: EntityId,
        ruling: Ruling,
    },
    CancelRoll {
        ruling: Ruling,
    },
    ApplyDamage {
        target: EntityId,
        amount: u32,
        damage_type: DamageType,
        critical: bool,
        ruling: Ruling,
    },
    Heal {
        target: EntityId,
        amount: u32,
        ruling: Ruling,
    },
    GrantTemporaryHp {
        target: EntityId,
        amount: u32,
        ruling: Ruling,
    },
    ApplyEffect {
        effect: ActiveEffect,
        ruling: Ruling,
    },
    RemoveEffect {
        effect_id: EffectId,
        ruling: Ruling,
    },
    SetExhaustion {
        target: EntityId,
        levels: u8,
        ruling: Ruling,
    },
    SetProne {
        target: EntityId,
        prone: bool,
        ruling: Ruling,
    },
    SpendResource {
        actor: EntityId,
        resource_id: String,
        amount: u16,
    },
    RecoverResource {
        actor: EntityId,
        resource_id: String,
        amount: u16,
        ruling: Ruling,
    },
    StartRest {
        actor: EntityId,
        kind: RestKind,
        ruling: Ruling,
    },
    InterruptRest {
        actor: EntityId,
        ruling: Ruling,
    },
    FinishRest {
        actor: EntityId,
        slept_seconds: u32,
        ruling: Ruling,
    },
    SpendHitDie {
        actor: EntityId,
        request_id: RollRequestId,
    },
    AdvanceTime {
        seconds: u32,
        ruling: Ruling,
    },
    StartCombat {
        participants: Vec<InitiativeEntry>,
        ruling: Ruling,
    },
    EndTurn {
        actor: EntityId,
    },
    EndCombat {
        ruling: Ruling,
    },
    UseReaction {
        actor: EntityId,
        trigger: String,
        ruling: Ruling,
    },
    UseBonusAction {
        actor: EntityId,
        feature_id: String,
        ruling: Ruling,
    },
    EndConcentration {
        actor: EntityId,
    },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RulesEvent {
    pub meta: CommandMeta,
    pub action: RulesAction,
    pub outcome: RulesOutcome,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RulesTransition {
    pub next_state: CampaignState,
    pub event: RulesEvent,
    pub outcome: RulesOutcome,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum RulesOutcome {
    Changed,
    AutomaticTest {
        success: bool,
    },
    RollRequested(RollRequest),
    RollResolved {
        roll: ResolvedRoll,
        success: Option<bool>,
        critical: bool,
        amount: Option<u32>,
        followup: Option<RollRequest>,
    },
    Damage {
        applied: u32,
        followup: Option<RollRequest>,
    },
    Healed {
        regained: u32,
    },
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RulesQuery {
    PendingRoll,
    Character {
        actor: EntityId,
    },
    PassivePerception {
        actor: EntityId,
        circumstances: Circumstances,
    },
    SupportedContent,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RulesAnswer {
    PendingRoll(Option<RollRequest>),
    Character {
        ability_modifiers: [i32; 6],
        proficiency_bonus: i32,
        armor_class: i32,
        hp: u32,
        max_hp: u32,
        temporary_hp: u32,
        spell_save_dc: Option<i32>,
    },
    PassivePerception(i32),
    SupportedContent {
        attacks: Vec<String>,
        spells: Vec<SupportedSpell>,
    },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SupportedSpell {
    pub id: String,
    pub support: SpellSupport,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpellSupport {
    HealingWithUpcasting,
    CreatureAttackDamage,
    ConcentrationDurationOnly,
}
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum RulesError {
    #[error("rules pack is incompatible: {0}")]
    Incompatible(String),
    #[error("invalid mechanics: {0}")]
    Invalid(String),
    #[error("issuer is not authorized for this action")]
    Unauthorized,
    #[error("command observes a stale journal sequence")]
    Stale,
    #[error("another roll awaits resolution")]
    Pending,
    #[error("no matching pending roll")]
    NoPending,
    #[error("rules are not initialized")]
    Uninitialized,
    #[error("a prerequisite or action budget is not satisfied: {0}")]
    Prerequisite(String),
    #[error("replayed outcome disagrees with deterministic resolution")]
    ReplayMismatch,
    #[error(transparent)]
    Roll(#[from] crate::RollError),
}
fn invalid(message: impl Into<String>) -> RulesError {
    RulesError::Invalid(message.into())
}
fn prerequisite(message: impl Into<String>) -> RulesError {
    RulesError::Prerequisite(message.into())
}
