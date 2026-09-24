//! Durable source-derived spell plans. These records are internal authority, not
//! a deserializable permission to install arbitrary effects or bypass a controller.
use crate::{
    Ability, CommandMeta, Condition, DamageType, DieSpec, EffectId, EntityId, ItemId, SpatialPoint,
    TurnBoundary, WorldInstant,
};
use serde::{Deserialize, Serialize};

pub const TACTICAL_SPELL_SCHEMA_VERSION: u32 = 1;

pub fn spell_concentration_id(
    command: crate::CommandId,
    actor: EntityId,
    occurrence: u16,
) -> EffectId {
    let mut name = b"dmd.tactical.spell.concentration.v1".to_vec();
    name.extend_from_slice(actor.0.as_bytes());
    name.extend_from_slice(&occurrence.to_be_bytes());
    EffectId(uuid::Uuid::new_v5(&command.0, &name))
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum SpellGrantChoice {
    Prepared,
    CreatureFeature { feature_id: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpellResourceChoice {
    Cantrip,
    Slot { level: u8 },
    SourceFeature,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum SpellCastMode {
    Immediate,
    /// The scheduler establishes whether a trigger actually occurs and is perceived.
    Ready {
        trigger: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum SpellMaterialChoice {
    None,
    Material { item: ItemId },
    Focus { item: ItemId },
    ComponentPouch { item: ItemId },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpellCastChoice {
    pub actor: EntityId,
    pub spell_id: String,
    pub grant: SpellGrantChoice,
    pub resource: SpellResourceChoice,
    pub material: SpellMaterialChoice,
    pub mode: SpellCastMode,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpellSourcePin {
    pub ruleset_id: String,
    pub ruleset_version: String,
    pub spell_id: String,
    /// Canonical typed content identity, not a cryptographic signature.
    pub fingerprint: String,
    pub creature_definition_id: Option<String>,
    pub feature_id: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpellCastingCost {
    Action,
    BonusAction,
    Reaction,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum SpellExpenditure {
    None,
    Slot {
        level: u8,
    },
    /// Source creature reducer owns this counter. This is a derived obligation,
    /// never a second spell-owned limited-use pool.
    CreatureUse {
        feature_id: String,
        spell_id: String,
        maximum: u8,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpellComponentsNeeded {
    pub verbal: bool,
    pub somatic: bool,
    pub material: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum SpellDuration {
    Instantaneous,
    Seconds { seconds: u32 },
    OwnerBoundary { boundary: TurnBoundary },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpellDamageDice {
    pub dice: Vec<DieSpec>,
    pub modifier: i16,
    pub damage_type: DamageType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpellRangeLimit {
    Caster,
    Touch,
    Feet(u16),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum SpellAreaShape {
    Cone { length_feet: u16 },
    Cube { side_feet: u16 },
    Cylinder { radius_feet: u16, height_feet: u16 },
    Emanation { radius_feet: u16 },
    Line { length_feet: u16, width_feet: u16 },
    Sphere { radius_feet: u16 },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum SpellTargetRule {
    Caster,
    CreatureOrObject,
    Creatures {
        maximum: u8,
        creature_type: Option<String>,
        requires_sight: bool,
    },
    Darts {
        count: u8,
        requires_sight: bool,
    },
    LightPoints {
        maximum: u8,
        may_combine_as_medium_form: bool,
    },
    Area(SpellAreaShape),
}

/// Amount references are source-local, never arbitrary expressions or state paths.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpellDamageShare {
    SingleOccurrence,
    PerAttack,
    PerDart,
    /// SRD16: one raw damage roll for all simultaneous saving-throw targets.
    SimultaneousSavingThrows,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum SpellProgramNode {
    Heal {
        dice: Vec<DieSpec>,
        modifier: i16,
    },
    AttackDamage {
        damage: SpellDamageDice,
        share: SpellDamageShare,
        ignite_unworn_uncarried_target: bool,
    },
    SaveDamage {
        ability: Ability,
        damage: SpellDamageDice,
        half_on_success: bool,
        push_on_failure_feet: u16,
    },
    SaveCondition {
        ability: Ability,
        condition: Condition,
        repeat_at_target_end: bool,
    },
    AutomaticDamage {
        damage: SpellDamageDice,
        share: SpellDamageShare,
    },
    Condition {
        condition: Condition,
    },
    DimLights {
        radius_feet: u16,
        bonus_action_move_feet: u16,
        maximum_separation_feet: u16,
        vanish_outside_casting_range: bool,
    },
    HeavyObscuration {
        dispersed_by_strong_wind: bool,
    },
    ArmorClassBonus {
        bonus: u8,
        includes_triggering_attack: bool,
    },
    PreventSpellDamage {
        spell_id: String,
    },
    IgniteUnwornUncarriedObjects,
    PushUnsecuredObjectsEntirelyInArea {
        feet: u16,
    },
    Audible {
        feet: u16,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpellEffectProgram {
    pub schema_version: u32,
    pub source: SpellSourcePin,
    pub spell_level: u8,
    /// Source creature numbers may be explicit; never invent an NPC class level.
    pub attack_bonus: Option<i16>,
    pub save_dc: Option<u16>,
    pub ability_modifier: i16,
    pub cantrip_character_level: Option<u8>,
    pub range: SpellRangeLimit,
    pub targets: SpellTargetRule,
    pub duration: SpellDuration,
    pub concentration: bool,
    pub nodes: Vec<SpellProgramNode>,
    pub source_pages: Vec<u16>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpellCastPlan {
    pub origin: CommandMeta,
    /// Allocated by the shared resolution, not supplied by a player request.
    pub occurrence: u16,
    pub choice: SpellCastChoice,
    pub program: SpellEffectProgram,
    pub cost: SpellCastingCost,
    pub expenditure: SpellExpenditure,
    /// Exact source-feature activation owns its action and innate-use payment. The
    /// scheduler must retain/prove that receipt; a public caller cannot set this flag.
    pub activation_prepaid: bool,
    pub components: SpellComponentsNeeded,
    pub concentration_group: Option<EffectId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpellCastPhase {
    /// Casting action is spent; the casting interruption window has not closed.
    Casting,
    /// Resources are paid and the source effect program can be queued once.
    Committed,
    /// Resources are paid; a readied spell is held by Concentration.
    Held,
    Released,
    Countered,
    Expired,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpellCast {
    pub plan: SpellCastPlan,
    pub phase: SpellCastPhase,
    pub started_at: WorldInstant,
    pub started_on_turn: u64,
    pub last_operation: CommandMeta,
}

/// Binding follows casting legality and is internal to the tactical resolver. The
/// source target selector/area is reconstructed from the pinned definition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum SpellTargetChoice {
    Entities(Vec<EntityId>),
    Point(SpatialPoint),
    LightPoints(Vec<SpatialPoint>),
}
