//! Source definitions for tactical execution; loading a definition does not execute it.
//! Kept separate from the version-1 kernel pack so historical replay keeps its meaning.
use dmd_domain::{Ability, Condition, DamageType, DieSpec, Skill};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use thiserror::Error;

pub const TACTICAL_DEFINITIONS_JSON: &str =
    include_str!("../../../content/srd-5.2.1/tactical.json");

#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("invalid tactical definitions: {0}")]
pub struct DefinitionError(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TacticalDefinitions {
    pub schema_version: u32,
    pub ruleset_id: String,
    pub ruleset_version: String,
    pub weapon_properties: Vec<PropertyDefinition>,
    pub masteries: Vec<MasteryDefinition>,
    pub weapons: Vec<WeaponDefinition>,
    pub spells: Vec<TacticalSpellDefinition>,
    pub creatures: Vec<CreatureDefinition>,
}

/// SRD pp89–90. Range is represented explicitly as well as by the source property tag.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum WeaponProperty {
    /// Spend one matching projectile; one-handed loading needs a free hand. Recover
    /// half (rounded down) after spending one minute following the fight (SRD89).
    Ammunition,
    /// Player chooses Strength or Dexterity; use the same modifier for attack/damage.
    Finesse,
    /// Disadvantage below Strength 13 for melee, Dexterity 13 for ranged (SRD89).
    Heavy,
    /// On-turn Attack enables one later bonus attack with another Light weapon;
    /// omit a positive ability damage modifier but retain a negative one (SRD89).
    Light,
    /// At most one projectile per action, bonus action or reaction, not per turn.
    Loading,
    /// Disadvantage beyond normal range; no attack beyond long range.
    Range,
    /// Add five feet to attacks and Opportunity Attacks using this weapon.
    Reach,
    /// Drawing is part of the attack; thrown melee weapons retain melee ability choice.
    Thrown,
    /// Both hands are required when attacking, subject to the Lance mounted exception.
    TwoHanded,
    /// The alternate damage applies to a melee attack using two hands.
    Versatile,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PropertyDefinition {
    pub property: WeaponProperty,
    pub source_page: u16,
}
/// Execution must apply the complete source clause, including optional choices and expiry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum WeaponMastery {
    /// Once per turn after a melee hit: optional attack on another creature within
    /// five feet of that target and within reach; omit positive ability damage.
    Cleave,
    /// On a miss: optional same-type damage equal to the attack ability modifier;
    /// only an increase to that modifier can increase this damage.
    Graze,
    /// Light's extra attack may be part of Attack instead of a bonus action, once/turn.
    Nick,
    /// A hit may push a Large-or-smaller creature up to ten feet straight away.
    Push,
    /// A hit gives disadvantage on the next attack before the attacker's next turn.
    Sap,
    /// A damaging hit may reduce speed ten feet until the attacker's next turn;
    /// repeated Slow hits do not increase that reduction.
    Slow,
    /// A hit may force Constitution DC 8 + attack ability + PB; failure makes Prone.
    Topple,
    /// A damaging hit grants advantage on the next attack against that creature
    /// before the end of the attacker's next turn.
    Vex,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MasteryDefinition {
    pub mastery: WeaponMastery,
    pub source_page: u16,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WeaponCategory {
    Simple,
    Martial,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WeaponKind {
    Melee,
    Ranged,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WeaponHands {
    One,
    Two,
    TwoUnlessMounted,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AmmunitionKind {
    Arrow,
    Bolt,
    Bullet,
    Needle,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WeaponRange {
    pub normal_feet: u16,
    pub long_feet: u16,
}
/// A fixed amount has no dice. This preserves the Blowgun's fixed 1 damage (SRD16/91).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DamageFormula {
    pub dice: Vec<DieSpec>,
    pub fixed: i16,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WeaponDefinition {
    pub id: String,
    pub name: String,
    pub category: WeaponCategory,
    pub kind: WeaponKind,
    pub damage: DamageFormula,
    pub damage_type: DamageType,
    pub properties: Vec<WeaponProperty>,
    pub range: Option<WeaponRange>,
    pub ammunition: Option<AmmunitionKind>,
    pub hands: WeaponHands,
    pub reach_bonus_feet: u16,
    pub versatile_damage: Option<DamageFormula>,
    pub mastery: WeaponMastery,
    pub source_page: u16,
}

/// Definition-local source categories; runtime geometry remains owned by dmd-domain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SourceSize {
    Tiny,
    Small,
    Medium,
    Large,
    Huge,
    Gargantuan,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CreatureType {
    Aberration,
    Beast,
    Celestial,
    Construct,
    Dragon,
    Elemental,
    Fey,
    Fiend,
    Giant,
    Humanoid,
    Monstrosity,
    Ooze,
    Plant,
    Undead,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum AreaShape {
    Cone { length_feet: u16 },
    Cube { side_feet: u16 },
    Cylinder { radius_feet: u16, height_feet: u16 },
    Emanation { radius_feet: u16 },
    Line { length_feet: u16, width_feet: u16 },
    Sphere { radius_feet: u16 },
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum SpellRange {
    Caster,
    Touch,
    Distance { feet: u16 },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum TargetSelection {
    Caster,
    CreatureOrObject,
    LightPoints {
        maximum: u8,
        may_combine_as_medium_form: bool,
    },
    Creatures {
        maximum: u8,
        creature_type: Option<CreatureType>,
        requires_sight: bool,
    },
    Area {
        shape: AreaShape,
    },
    /// Each dart has a chosen visible creature; several may have the same target.
    Darts {
        count: u8,
        requires_sight: bool,
    },
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReactionTrigger {
    HitByAttackOrTargetedByMagicMissile,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum CastingTime {
    Action,
    BonusAction,
    Reaction { trigger: ReactionTrigger },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpellComponents {
    pub verbal: bool,
    pub somatic: bool,
    pub material: Option<MaterialComponent>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MaterialComponent {
    pub description: String,
    pub minimum_cost_cp: u32,
    pub consumed: bool,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum EffectDuration {
    Instantaneous,
    Seconds { seconds: u32, concentration: bool },
    UntilStartOfCastersNextTurn,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RepeatSave {
    EndOfTargetsTurnEndsOnSuccess,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DamageComponent {
    pub amount: DamageFormula,
    pub damage_type: DamageType,
}
/// Declarative effects, not permission to execute them. SaveDamage rolls once for all
/// simultaneously affected targets; each target applies its own save and damage traits.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum EffectDescriptor {
    Healing {
        dice: Vec<DieSpec>,
        add_spellcasting_modifier: bool,
    },
    RangedSpellAttack {
        damage: DamageComponent,
        extra_die_at_levels: Vec<u8>,
        ignite_unworn_uncarried_target: bool,
    },
    MovableDimLights {
        radius_feet: u16,
        bonus_action_move_feet: u16,
        maximum_separation_feet: u16,
        vanish_outside_casting_range: bool,
    },
    SaveDamage {
        ability: Ability,
        damage: DamageComponent,
        half_on_success: bool,
        push_on_failure_feet: u16,
    },
    SaveCondition {
        ability: Ability,
        condition: Condition,
        repeat: Option<RepeatSave>,
    },
    Damage {
        damage: DamageComponent,
    },
    Condition {
        condition: Condition,
    },
    HeavilyObscured {
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
pub enum Upcast {
    ExtraDamageDie { die: DieSpec },
    ExtraHealingDice { dice: Vec<DieSpec> },
    AdditionalTargets { count: u8 },
    AdditionalDarts { count: u8 },
    AdditionalRadius { feet: u16 },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TacticalSpellDefinition {
    pub id: String,
    pub name: String,
    pub level: u8,
    pub casting_time: CastingTime,
    pub range: SpellRange,
    pub targets: TargetSelection,
    pub components: SpellComponents,
    pub duration: EffectDuration,
    pub effects: Vec<EffectDescriptor>,
    pub upcast: Option<Upcast>,
    pub source_pages: Vec<u16>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Speeds {
    pub walk: u16,
    pub burrow: u16,
    pub climb: u16,
    pub fly: u16,
    pub swim: u16,
    pub hover: bool,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SenseRanges {
    pub blindsight: u16,
    pub darkvision: u16,
    pub tremorsense: u16,
    pub truesight: u16,
    pub passive_perception: i16,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SkillModifier {
    pub skill: Skill,
    pub modifier: i16,
}
/// Source stat blocks have explicit attack/save/initiative values; do not infer monster
/// PB from a fabricated player level or treat CR as level.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreatureStatistics {
    pub size: SourceSize,
    pub allowed_sizes: Vec<SourceSize>,
    pub creature_type: CreatureType,
    pub creature_tags: Vec<String>,
    pub alignment: String,
    pub armor_class: u8,
    pub hit_points: u32,
    pub hit_point_formula: DamageFormula,
    pub ability_scores: [u8; 6],
    pub saving_throw_modifiers: [i16; 6],
    pub initiative_modifier: i16,
    pub proficiency_bonus: u8,
    pub challenge_rating: String,
    pub experience_points: u32,
    pub speeds: Speeds,
    pub skills: Vec<SkillModifier>,
    pub senses: SenseRanges,
    pub damage_resistances: Vec<DamageType>,
    pub damage_vulnerabilities: Vec<DamageType>,
    pub damage_immunities: Vec<DamageType>,
    pub condition_immunities: Vec<Condition>,
    pub exhaustion_immune: bool,
    pub languages: Vec<String>,
    pub additional_languages: u8,
    pub can_speak: bool,
    pub gear: Vec<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum MonsterTrait {
    /// The ally must be another creature and must not be Incapacitated (SRD364).
    PackTactics { ally_distance_feet: u16 },
    LegendaryResistance {
        uses_per_long_rest: u8,
        uses_in_lair: u8,
    },
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum AttackDelivery {
    Melee { reach_feet: u16 },
    Ranged { range: WeaponRange },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum HitRequirement {
    TargetSizeAtMost {
        size: SourceSize,
    },
    /// Required straight movement toward this target immediately before this hit.
    Charge {
        straight_feet: u16,
        target_size_at_most: SourceSize,
    },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConditionalHitEffect {
    pub requirement: HitRequirement,
    pub effects: Vec<EffectDescriptor>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BasicAction {
    Dash,
    Disengage,
    Dodge,
    Hide,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Recharge {
    /// SRD257: one use; roll at own turn start, or recover on Short/Long Rest.
    pub die_sides: u16,
    pub minimum: u16,
    pub maximum: u16,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum MonsterFeature {
    Attack {
        bonus: i16,
        delivery: AttackDelivery,
        damage: Vec<DamageComponent>,
        extra_damage_if_attack_had_advantage: Vec<DamageComponent>,
        conditional_hits: Vec<ConditionalHitEffect>,
    },
    Multiattack {
        count: u8,
        attack_options: Vec<String>,
    },
    BasicActionChoice {
        options: Vec<BasicAction>,
    },
    Spellcasting {
        ability: Ability,
        save_dc: Option<u8>,
        attack_bonus: Option<i16>,
        spells: Vec<InnateSpell>,
    },
    SaveArea {
        dc: u8,
        area: AreaShape,
        effect: EffectDescriptor,
        recharge: Recharge,
    },
    /// Movement still provokes normally; this is not teleportation or Disengage.
    MoveThenAttack {
        speed_divisor: u8,
        attack_id: String,
    },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InnateSpell {
    pub spell_id: String,
    pub cast_level: u8,
    /// None means at will; "per day" in the SRD recovers on a Long Rest (p186).
    pub uses_per_long_rest: Option<u8>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum FeatureActivation {
    Action,
    BonusAction,
    Legendary { cost: u8 },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NamedMonsterFeature {
    pub id: String,
    pub name: String,
    pub activation: FeatureActivation,
    pub feature: MonsterFeature,
    pub source_page: u16,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LegendaryBudget {
    /// SRD257: one action after another creature's turn; unavailable while
    /// Incapacitated; refresh all uses at the start of the monster's own turn.
    pub uses: u8,
    pub uses_in_lair: u8,
}
/// Complete refers only to represented stat-block clauses, never execution support.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum DefinitionCoverage {
    CompleteStatBlock,
    SelectedFeatures { omitted: Vec<String> },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreatureDefinition {
    pub id: String,
    pub name: String,
    pub coverage: DefinitionCoverage,
    pub statistics: CreatureStatistics,
    pub traits: Vec<MonsterTrait>,
    pub features: Vec<NamedMonsterFeature>,
    pub legendary_budget: Option<LegendaryBudget>,
    pub source_pages: Vec<u16>,
}

impl TacticalDefinitions {
    pub fn from_json(json: &str) -> Result<Self, DefinitionError> {
        let definitions: Self =
            serde_json::from_str(json).map_err(|e| DefinitionError(format!("JSON: {e}")))?;
        definitions.validate()?;
        Ok(definitions)
    }

    pub fn weapon(&self, id: &str) -> Option<&WeaponDefinition> {
        self.weapons.iter().find(|w| w.id == id)
    }
    pub fn spell(&self, id: &str) -> Option<&TacticalSpellDefinition> {
        self.spells.iter().find(|s| s.id == id)
    }
    pub fn creature(&self, id: &str) -> Option<&CreatureDefinition> {
        self.creatures.iter().find(|c| c.id == id)
    }

    pub fn validate(&self) -> Result<(), DefinitionError> {
        ensure(
            self.schema_version == 1
                && self.ruleset_id == "srd-5.2"
                && self.ruleset_version == "5.2.1",
            "unsupported schema or source pin",
        )?;
        let properties: BTreeSet<_> = self.weapon_properties.iter().map(|p| p.property).collect();
        ensure(
            properties.len() == 10 && self.weapon_properties.len() == 10,
            "all ten source properties must be defined exactly once",
        )?;
        for p in &self.weapon_properties {
            page(p.source_page)?;
        }
        let masteries: BTreeSet<_> = self.masteries.iter().map(|m| m.mastery).collect();
        ensure(
            masteries.len() == 8 && self.masteries.len() == 8,
            "all eight source masteries must be defined exactly once",
        )?;
        for m in &self.masteries {
            page(m.source_page)?;
        }
        ensure(
            !self.weapons.is_empty() && !self.spells.is_empty() && !self.creatures.is_empty(),
            "empty definition family",
        )?;
        unique_ids(self.weapons.iter().map(|w| w.id.as_str()))?;
        unique_ids(self.spells.iter().map(|s| s.id.as_str()))?;
        unique_ids(self.creatures.iter().map(|c| c.id.as_str()))?;
        for weapon in &self.weapons {
            weapon.validate()?;
        }
        for spell in &self.spells {
            spell.validate(self)?;
        }
        for creature in &self.creatures {
            creature.validate(self)?;
        }
        Ok(())
    }
}

impl WeaponDefinition {
    fn validate(&self) -> Result<(), DefinitionError> {
        label(&self.name)?;
        page(self.source_page)?;
        formula(&self.damage)?;
        let props: BTreeSet<_> = self.properties.iter().copied().collect();
        ensure(
            props.len() == self.properties.len(),
            "duplicate weapon property",
        )?;
        let has = |p| props.contains(&p);
        ensure(
            has(WeaponProperty::Ammunition) == self.ammunition.is_some(),
            "ammunition mismatch",
        )?;
        ensure(
            has(WeaponProperty::Range) == self.range.is_some(),
            "range property mismatch",
        )?;
        ensure(
            self.range.is_some()
                == (has(WeaponProperty::Ammunition) || has(WeaponProperty::Thrown)),
            "range must belong to ammunition or thrown attack",
        )?;
        ensure(
            !has(WeaponProperty::Ammunition) || self.kind == WeaponKind::Ranged,
            "ammunition weapon must be ranged",
        )?;
        ensure(
            self.kind != WeaponKind::Ranged || self.range.is_some(),
            "ranged weapon lacks range",
        )?;
        ensure(
            !has(WeaponProperty::Loading) || has(WeaponProperty::Ammunition),
            "loading needs ammunition",
        )?;
        ensure(
            !(has(WeaponProperty::Thrown) && has(WeaponProperty::Ammunition)),
            "ambiguous ranged delivery",
        )?;
        if let Some(r) = self.range {
            weapon_range(r)?;
        }
        ensure(
            has(WeaponProperty::TwoHanded) == (self.hands != WeaponHands::One),
            "hand requirement mismatch",
        )?;
        ensure(
            self.hands != WeaponHands::TwoUnlessMounted
                || (self.kind == WeaponKind::Melee && self.id == "lance"),
            "unsupported mounted exception",
        )?;
        ensure(
            self.reach_bonus_feet == if has(WeaponProperty::Reach) { 5 } else { 0 },
            "reach property mismatch",
        )?;
        ensure(
            !has(WeaponProperty::Reach) || self.kind == WeaponKind::Melee,
            "ranged weapon cannot have reach",
        )?;
        ensure(
            has(WeaponProperty::Versatile) == self.versatile_damage.is_some(),
            "versatile mismatch",
        )?;
        if let Some(d) = &self.versatile_damage {
            formula(d)?;
            ensure(
                self.kind == WeaponKind::Melee && self.hands == WeaponHands::One,
                "invalid versatile weapon",
            )?;
        }
        ensure(
            !has(WeaponProperty::Light) || self.hands == WeaponHands::One,
            "light weapon needs one hand",
        )?;
        Ok(())
    }
}

impl TacticalSpellDefinition {
    fn validate(&self, pack: &TacticalDefinitions) -> Result<(), DefinitionError> {
        label(&self.name)?;
        pages(&self.source_pages)?;
        ensure(self.level <= 9, "invalid spell level")?;
        if let SpellRange::Distance { feet } = self.range {
            distance(feet)?;
        }
        match self.targets {
            TargetSelection::Caster => ensure(
                self.range == SpellRange::Caster,
                "self target needs self range",
            )?,
            TargetSelection::CreatureOrObject => {}
            TargetSelection::LightPoints { maximum, .. } => {
                ensure((1..=20).contains(&maximum), "invalid light count")?
            }
            TargetSelection::Creatures { maximum, .. } => {
                ensure((1..=100).contains(&maximum), "invalid target count")?
            }
            TargetSelection::Area { shape } => area(shape)?,
            TargetSelection::Darts {
                count,
                requires_sight,
            } => {
                ensure(
                    (1..=100).contains(&count) && requires_sight,
                    "invalid dart selection",
                )?;
                ensure(
                    self.effects
                        .iter()
                        .filter(|e| matches!(e, EffectDescriptor::Damage { .. }))
                        .count()
                        == 1,
                    "darts need one per-dart damage formula",
                )?;
            }
        }
        if let Some(m) = &self.components.material {
            label(&m.description)?;
        }
        if let EffectDuration::Seconds { seconds, .. } = self.duration {
            ensure((1..=604_800).contains(&seconds), "invalid duration")?;
        }
        effects(&self.effects)?;
        for e in &self.effects {
            match e {
                EffectDescriptor::PreventSpellDamage { spell_id } => {
                    ensure(pack.spell(spell_id).is_some(), "unknown prevented spell")?
                }
                EffectDescriptor::SaveCondition {
                    repeat: Some(_), ..
                } => ensure(
                    self.duration != EffectDuration::Instantaneous,
                    "ongoing save needs duration",
                )?,
                EffectDescriptor::HeavilyObscured { .. }
                | EffectDescriptor::IgniteUnwornUncarriedObjects
                | EffectDescriptor::PushUnsecuredObjectsEntirelyInArea { .. } => ensure(
                    matches!(self.targets, TargetSelection::Area { .. }),
                    "spatial effect needs area",
                )?,
                EffectDescriptor::MovableDimLights { .. } => ensure(
                    matches!(self.targets, TargetSelection::LightPoints { .. }),
                    "lights need point selection",
                )?,
                EffectDescriptor::RangedSpellAttack {
                    extra_die_at_levels,
                    ..
                } => ensure(
                    extra_die_at_levels.is_empty()
                        || (self.level == 0 && extra_die_at_levels == &[5, 11, 17]),
                    "invalid cantrip scaling",
                )?,
                EffectDescriptor::ArmorClassBonus {
                    includes_triggering_attack: true,
                    ..
                } => ensure(
                    matches!(self.casting_time, CastingTime::Reaction { .. }),
                    "triggering attack needs reaction",
                )?,
                _ => {}
            }
        }
        if let Some(upcast) = &self.upcast {
            ensure(self.level > 0, "slot upcasting is not cantrip scaling")?;
            match upcast {
                Upcast::ExtraDamageDie { die } => {
                    dice(std::slice::from_ref(die))?;
                    ensure(
                        self.effects.iter().any(|e| {
                            matches!(
                                e,
                                EffectDescriptor::SaveDamage { .. }
                                    | EffectDescriptor::Damage { .. }
                            )
                        }),
                        "damage upcast lacks damage",
                    )?;
                }
                Upcast::ExtraHealingDice { dice: extra } => {
                    dice(extra)?;
                    ensure(
                        !extra.is_empty()
                            && self
                                .effects
                                .iter()
                                .any(|e| matches!(e, EffectDescriptor::Healing { .. })),
                        "healing upcast mismatch",
                    )?;
                }
                Upcast::AdditionalTargets { count } => ensure(
                    *count > 0 && matches!(self.targets, TargetSelection::Creatures { .. }),
                    "target upcast mismatch",
                )?,
                Upcast::AdditionalDarts { count } => ensure(
                    *count > 0 && matches!(self.targets, TargetSelection::Darts { .. }),
                    "dart upcast mismatch",
                )?,
                Upcast::AdditionalRadius { feet } => {
                    distance(*feet)?;
                    ensure(
                        matches!(
                            self.targets,
                            TargetSelection::Area {
                                shape: AreaShape::Sphere { .. }
                            }
                        ),
                        "radius upcast mismatch",
                    )?;
                }
            }
        }
        Ok(())
    }
}

impl CreatureDefinition {
    fn validate(&self, pack: &TacticalDefinitions) -> Result<(), DefinitionError> {
        label(&self.name)?;
        pages(&self.source_pages)?;
        if let DefinitionCoverage::SelectedFeatures { omitted } = &self.coverage {
            ensure(
                !omitted.is_empty(),
                "partial creature must list omitted clauses",
            )?;
            unique_ids(omitted.iter().map(String::as_str))?;
        }
        let s = &self.statistics;
        label(&s.alignment)?;
        for tag in &s.creature_tags {
            label(tag)?;
        }
        ensure(
            s.allowed_sizes.contains(&s.size)
                && s.allowed_sizes.iter().collect::<BTreeSet<_>>().len() == s.allowed_sizes.len(),
            "invalid allowed sizes",
        )?;
        ensure(
            (1..=30).contains(&s.armor_class) && (1..=1_000_000).contains(&s.hit_points),
            "invalid AC or HP",
        )?;
        formula(&s.hit_point_formula)?;
        ensure(
            s.ability_scores.iter().all(|n| (1..=30).contains(n)),
            "invalid creature ability",
        )?;
        ensure(
            s.saving_throw_modifiers
                .iter()
                .chain([&s.initiative_modifier])
                .all(|n| (-10..=40).contains(n)),
            "invalid creature modifier",
        )?;
        ensure(
            (2..=9).contains(&s.proficiency_bonus),
            "invalid monster proficiency",
        )?;
        ensure(
            matches!(s.challenge_rating.as_str(), "0" | "1/8" | "1/4" | "1/2")
                || s.challenge_rating
                    .parse::<u8>()
                    .is_ok_and(|v| (1..=30).contains(&v)),
            "invalid challenge rating",
        )?;
        for n in [
            s.speeds.walk,
            s.speeds.burrow,
            s.speeds.climb,
            s.speeds.fly,
            s.speeds.swim,
            s.senses.blindsight,
            s.senses.darkvision,
            s.senses.tremorsense,
            s.senses.truesight,
        ] {
            ensure(n <= 1_000, "unbounded speed or sense")?;
        }
        ensure(!s.speeds.hover || s.speeds.fly > 0, "hover needs flight")?;
        ensure(
            (0..=50).contains(&s.senses.passive_perception),
            "invalid passive perception",
        )?;
        let mut skills = BTreeSet::new();
        for skill in &s.skills {
            ensure(
                skills.insert(skill.skill) && (-10..=40).contains(&skill.modifier),
                "duplicate or invalid skill",
            )?;
        }
        for values in [
            &s.damage_resistances,
            &s.damage_vulnerabilities,
            &s.damage_immunities,
        ] {
            ensure(
                values.iter().collect::<BTreeSet<_>>().len() == values.len(),
                "duplicate damage trait",
            )?;
        }
        ensure(
            s.condition_immunities.iter().collect::<BTreeSet<_>>().len()
                == s.condition_immunities.len(),
            "duplicate condition immunity",
        )?;
        for language in &s.languages {
            label(language)?;
        }
        unique_ids(s.gear.iter().map(String::as_str))?;
        unique_ids(self.features.iter().map(|f| f.id.as_str()))?;
        ensure(
            !self.features.is_empty(),
            "creature has no defined features",
        )?;
        if let Some(b) = &self.legendary_budget {
            ensure(
                (1..=10).contains(&b.uses) && b.uses_in_lair >= b.uses && b.uses_in_lair <= 10,
                "invalid legendary budget",
            )?;
        }
        for t in &self.traits {
            match t {
                MonsterTrait::PackTactics { ally_distance_feet } => distance(*ally_distance_feet)?,
                MonsterTrait::LegendaryResistance {
                    uses_per_long_rest,
                    uses_in_lair,
                } => ensure(
                    *uses_per_long_rest > 0
                        && *uses_in_lair >= *uses_per_long_rest
                        && *uses_in_lair <= 10,
                    "invalid legendary resistance",
                )?,
            }
        }
        ensure(
            s.additional_languages <= 10,
            "invalid language choice count",
        )?;
        for f in &self.features {
            self.validate_feature(f, pack)?;
        }
        Ok(())
    }

    fn validate_feature(
        &self,
        f: &NamedMonsterFeature,
        pack: &TacticalDefinitions,
    ) -> Result<(), DefinitionError> {
        label(&f.name)?;
        page(f.source_page)?;
        ensure(
            self.source_pages.contains(&f.source_page),
            "feature page outside creature source",
        )?;
        if let FeatureActivation::Legendary { cost } = f.activation {
            ensure(
                self.legendary_budget
                    .as_ref()
                    .is_some_and(|b| cost > 0 && cost <= b.uses),
                "legendary feature lacks valid budget",
            )?;
        }
        let attack_exists = |id: &str| {
            self.features
                .iter()
                .any(|a| a.id == id && matches!(a.feature, MonsterFeature::Attack { .. }))
        };
        match &f.feature {
            MonsterFeature::Attack {
                bonus,
                delivery,
                damage,
                extra_damage_if_attack_had_advantage,
                conditional_hits,
            } => {
                ensure((-10..=40).contains(bonus), "invalid attack bonus")?;
                match delivery {
                    AttackDelivery::Melee { reach_feet } => distance(*reach_feet)?,
                    AttackDelivery::Ranged { range } => weapon_range(*range)?,
                }
                ensure(!damage.is_empty(), "attack lacks damage")?;
                for d in damage.iter().chain(extra_damage_if_attack_had_advantage) {
                    formula(&d.amount)?;
                }
                for c in conditional_hits {
                    if let HitRequirement::Charge { straight_feet, .. } = c.requirement {
                        distance(straight_feet)?;
                    }
                    effects(&c.effects)?;
                }
            }
            MonsterFeature::Multiattack {
                count,
                attack_options,
            } => {
                ensure(
                    (1..=20).contains(count) && !attack_options.is_empty(),
                    "invalid multiattack count",
                )?;
                unique_ids(attack_options.iter().map(String::as_str))?;
                ensure(
                    attack_options.iter().all(|id| attack_exists(id)),
                    "multiattack references missing/non-attack feature",
                )?;
                ensure(
                    f.activation == FeatureActivation::Action,
                    "multiattack requires action",
                )?;
            }
            MonsterFeature::BasicActionChoice { options } => {
                ensure(!options.is_empty(), "empty action choice")?;
                ensure(
                    options
                        .iter()
                        .enumerate()
                        .all(|(i, option)| !options[..i].contains(option)),
                    "duplicate action choice",
                )?;
            }
            MonsterFeature::Spellcasting {
                save_dc,
                attack_bonus,
                spells,
                ..
            } => {
                ensure(
                    save_dc.is_none_or(|dc| (1..=30).contains(&dc))
                        && attack_bonus.is_none_or(|b| (-10..=40).contains(&b))
                        && !spells.is_empty(),
                    "invalid spellcasting metadata",
                )?;
                unique_ids(spells.iter().map(|s| s.spell_id.as_str()))?;
                for spell in spells {
                    let definition = pack
                        .spell(&spell.spell_id)
                        .ok_or_else(|| DefinitionError("unknown innate spell".into()))?;
                    ensure(
                        (definition.level..=9).contains(&spell.cast_level)
                            && (definition.level != 0 || spell.cast_level == 0)
                            && spell
                                .uses_per_long_rest
                                .is_none_or(|uses| (1..=20).contains(&uses)),
                        "invalid innate spell use",
                    )?;
                }
            }
            MonsterFeature::SaveArea {
                dc,
                area: shape,
                effect,
                recharge,
            } => {
                ensure((1..=30).contains(dc), "invalid save DC")?;
                area(*shape)?;
                effects(std::slice::from_ref(effect))?;
                ensure(
                    matches!(
                        effect,
                        EffectDescriptor::SaveDamage { .. }
                            | EffectDescriptor::SaveCondition { .. }
                    ),
                    "save area needs save effect",
                )?;
                ensure(
                    recharge.die_sides == 6
                        && (1..=6).contains(&recharge.minimum)
                        && recharge.maximum == 6
                        && recharge.minimum <= recharge.maximum,
                    "invalid d6 recharge interval",
                )?;
            }
            MonsterFeature::MoveThenAttack {
                speed_divisor,
                attack_id,
            } => {
                ensure(
                    (1..=4).contains(speed_divisor) && attack_exists(attack_id),
                    "invalid move/attack reference",
                )?;
            }
        }
        Ok(())
    }
}

fn ensure(condition: bool, message: &str) -> Result<(), DefinitionError> {
    if condition {
        Ok(())
    } else {
        Err(DefinitionError(message.into()))
    }
}
fn label(s: &str) -> Result<(), DefinitionError> {
    ensure(
        !s.trim().is_empty() && s.len() <= 300 && !s.chars().any(char::is_control),
        "invalid label",
    )
}
fn unique_ids<'a>(ids: impl IntoIterator<Item = &'a str>) -> Result<(), DefinitionError> {
    let mut seen = BTreeSet::new();
    for id in ids {
        ensure(
            !id.is_empty()
                && id.len() <= 100
                && id
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
                && seen.insert(id),
            "invalid or duplicate identifier",
        )?;
    }
    Ok(())
}
fn page(page: u16) -> Result<(), DefinitionError> {
    ensure((1..=364).contains(&page), "invalid source page")
}
fn pages(pages: &[u16]) -> Result<(), DefinitionError> {
    ensure(
        !pages.is_empty() && pages.windows(2).all(|w| w[0] < w[1]),
        "source pages must be sorted and unique",
    )?;
    for p in pages {
        page(*p)?;
    }
    Ok(())
}
fn distance(n: u16) -> Result<(), DefinitionError> {
    ensure((1..=5_280).contains(&n), "invalid distance")
}
fn weapon_range(r: WeaponRange) -> Result<(), DefinitionError> {
    distance(r.normal_feet)?;
    distance(r.long_feet)?;
    ensure(r.normal_feet <= r.long_feet, "inverted range")
}
fn dice(dice: &[DieSpec]) -> Result<(), DefinitionError> {
    ensure(
        dice.len() <= 8
            && dice.iter().all(|d| {
                (1..=100).contains(&d.count) && [4, 6, 8, 10, 12, 20, 100].contains(&d.sides)
            }),
        "invalid dice",
    )
}
fn formula(f: &DamageFormula) -> Result<(), DefinitionError> {
    dice(&f.dice)?;
    ensure(
        (-100..=1_000).contains(&f.fixed) && (!f.dice.is_empty() || f.fixed > 0),
        "invalid fixed/dice formula",
    )
}
fn area(shape: AreaShape) -> Result<(), DefinitionError> {
    match shape {
        AreaShape::Cone { length_feet }
        | AreaShape::Cube {
            side_feet: length_feet,
        }
        | AreaShape::Emanation {
            radius_feet: length_feet,
        }
        | AreaShape::Sphere {
            radius_feet: length_feet,
        } => distance(length_feet),
        AreaShape::Cylinder {
            radius_feet,
            height_feet,
        } => {
            distance(radius_feet)?;
            distance(height_feet)
        }
        AreaShape::Line {
            length_feet,
            width_feet,
        } => {
            distance(length_feet)?;
            distance(width_feet)
        }
    }
}
fn effects(es: &[EffectDescriptor]) -> Result<(), DefinitionError> {
    ensure(
        !es.is_empty() && es.len() <= 20,
        "empty or excessive effect list",
    )?;
    for e in es {
        match e {
            EffectDescriptor::Healing { dice: healing, .. } => {
                dice(healing)?;
                ensure(!healing.is_empty(), "healing requires dice")?;
            }
            EffectDescriptor::RangedSpellAttack { damage, .. } => formula(&damage.amount)?,
            EffectDescriptor::MovableDimLights {
                radius_feet,
                bonus_action_move_feet,
                maximum_separation_feet,
                ..
            } => {
                distance(*radius_feet)?;
                distance(*bonus_action_move_feet)?;
                distance(*maximum_separation_feet)?;
            }
            EffectDescriptor::SaveDamage {
                damage,
                push_on_failure_feet,
                ..
            } => {
                formula(&damage.amount)?;
                if *push_on_failure_feet > 0 {
                    distance(*push_on_failure_feet)?;
                }
            }
            EffectDescriptor::Damage { damage } => formula(&damage.amount)?,
            EffectDescriptor::ArmorClassBonus { bonus, .. } => {
                ensure((1..=10).contains(bonus), "invalid AC bonus")?
            }
            EffectDescriptor::PreventSpellDamage { spell_id } => unique_ids([spell_id.as_str()])?,
            EffectDescriptor::PushUnsecuredObjectsEntirelyInArea { feet }
            | EffectDescriptor::Audible { feet } => distance(*feet)?,
            _ => {}
        }
    }
    Ok(())
}
