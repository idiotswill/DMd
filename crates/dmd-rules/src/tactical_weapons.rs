//! Pure source-backed weapon plans. These are not player-authorized commands.
//! The encounter resolver supplies validated truth and commits resources/receipts atomically.
mod equipment;
mod history;
mod mastery;
#[cfg(test)]
mod tests;
use crate::tactical_definitions::*;
use crate::{RulesPack, ability_modifier, proficiency_bonus, validate_character_intrinsics};
use dmd_domain::*;
pub use mastery::*;
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum WeaponError {
    #[error("invalid weapon context: {0}")]
    Invalid(String),
    #[error("weapon use is not legal: {0}")]
    Illegal(String),
}
fn invalid(message: impl Into<String>) -> WeaponError {
    WeaponError::Invalid(message.into())
}
fn illegal(message: impl Into<String>) -> WeaponError {
    WeaponError::Illegal(message.into())
}
fn require(condition: bool, message: &str) -> Result<(), WeaponError> {
    if condition {
        Ok(())
    } else {
        Err(illegal(message))
    }
}

/// Source identity is established by the encounter resolver, never a player flag.
pub enum WeaponActorSource<'a> {
    Character(&'a CharacterProfile),
    /// Ordinary use of equipment. Stat-block features must use their explicit attack
    /// bonus, damage and riders in the creature resolver, not this ordinary profile.
    CreatureOrdinaryWeapon(&'a CreatureDefinition),
}
/// Runtime-derived geometry/timing facts. Intentionally not Deserialize.
pub struct WeaponAttackContext<'a> {
    pub origin: &'a CommandMeta,
    pub actor: EntityId,
    pub turn_number: u64,
    pub on_actor_turn: bool,
    pub window: WeaponActionWindow,
    /// All distances use the spatial module's half-foot units.
    pub distance: u32,
    pub base_reach: u32,
    pub mounted: bool,
    pub underwater: bool,
    pub has_swim_speed: bool,
    pub target_is_creature: bool,
    pub target_size: CreatureSize,
    /// Distance from a Cleave trigger's target to the chosen secondary target.
    pub distance_from_trigger_target: Option<u32>,
}
pub struct WeaponAttackInput<'a> {
    pub state: &'a CampaignState,
    pub source: WeaponActorSource<'a>,
    pub pack: &'a RulesPack,
    pub definitions: &'a TacticalDefinitions,
    pub choice: &'a WeaponUseChoice,
    pub context: WeaponAttackContext<'a>,
    pub loadout: &'a WeaponLoadout,
    /// Current global turn's accepted receipts, including off-turn reactions.
    pub history: &'a [WeaponAttackReceipt],
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum WeaponDisadvantage {
    HeavyAbilityRequirement,
    LongRange,
    UnderwaterMelee,
    UnderwaterRanged,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WeaponDamageProfile {
    pub dice: Vec<DieSpec>,
    pub modifier: i32,
    pub damage_type: DamageType,
}
impl WeaponDamageProfile {
    pub fn for_critical(&self) -> Result<Self, WeaponError> {
        let mut result = self.clone();
        for die in &mut result.dice {
            die.count = die
                .count
                .checked_mul(2)
                .ok_or_else(|| invalid("critical dice overflow"))?;
        }
        Ok(result)
    }
    /// Fixed weapon damage has no RollRequest; source modifiers never turn it negative.
    pub fn fixed_amount(&self) -> Option<u32> {
        self.dice.is_empty().then(|| self.modifier.max(0) as u32)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AmmunitionExpenditure {
    pub stack: ItemId,
    pub quantity: u32,
}
/// Persist the originating choice and rederive this plan on replay. A serialized plan
/// is evidence, not authority to bypass current state validation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WeaponAttackPlan {
    pub receipt: WeaponAttackReceipt,
    /// Ability, trained proficiency, Archery and Exhaustion are already included.
    /// Apply spatial/effect advantage and other active effects separately, never add
    /// the same source bonuses or Exhaustion penalty again at the dice boundary.
    pub attack_modifier: i32,
    pub ability_modifier: i32,
    pub proficiency_bonus: i32,
    pub proficient: bool,
    pub damage: WeaponDamageProfile,
    pub disadvantage: Vec<WeaponDisadvantage>,
    pub automatic_miss: bool,
    pub mastery: Option<WeaponMastery>,
    pub reach: u32,
    /// Apply this before the roll and before presenting a durable pending request.
    pub loadout_for_attack: WeaponLoadout,
    pub ammunition: Option<AmmunitionExpenditure>,
    /// Apply only after the attack's resolution, not while awaiting its roll.
    pub loadout_after_attack: WeaponLoadout,
    /// A thrown physical weapon leaves custody; it is not destroyed or silently replenished.
    pub thrown_weapon: Option<ItemId>,
}

/// Source ammunition table SRD96; firearm and sling bullets are distinct physical items.
pub fn required_ammunition_definition(weapon: &WeaponDefinition) -> Option<&'static str> {
    match weapon.ammunition? {
        AmmunitionKind::Arrow => Some("arrows"),
        AmmunitionKind::Bolt => Some("bolts"),
        AmmunitionKind::Needle => Some("needles"),
        AmmunitionKind::Bullet if weapon.id == "sling" => Some("sling-bullets"),
        AmmunitionKind::Bullet => Some("firearm-bullets"),
    }
}
/// Caller records the one-minute post-fight recovery and prevents repeating it (SRD89).
pub fn recoverable_ammunition(expended: u32) -> u32 {
    expended / 2
}

pub fn prepare_weapon_attack(
    input: &WeaponAttackInput<'_>,
) -> Result<WeaponAttackPlan, WeaponError> {
    let WeaponAttackInput {
        state,
        choice,
        context,
        definitions,
        ..
    } = input;
    definitions
        .validate()
        .map_err(|error| invalid(error.to_string()))?;
    input
        .pack
        .validate()
        .map_err(|error| invalid(error.to_string()))?;
    require(
        context.origin.campaign_id == state.campaign_id()
            && context.origin.expected_event_sequence == state.applied_event_sequence,
        "foreign or stale weapon command",
    )?;
    let entity = state
        .rules
        .as_ref()
        .and_then(|rules| rules.entities.get(&context.actor))
        .ok_or_else(|| invalid("missing mechanical actor"))?;
    require(
        state.entities.contains_key(&choice.target),
        "unknown weapon target",
    )?;
    require(
        context.distance <= 20_000
            && context.base_reach > 0
            && context.base_reach <= 4000
            && context
                .distance_from_trigger_target
                .is_none_or(|distance| distance <= 20_000),
        "unbounded spatial weapon context",
    )?;
    require(
        !entity.death.dead
            && entity.exhaustion < 6
            && entity.hp > 0
            && !crate::active_conditions(state.rules.as_ref().unwrap(), context.actor)
                .contains(&Condition::Incapacitated),
        "actor cannot attack while dead or incapacitated",
    )?;
    if let Some(actor) = &context.origin.actor {
        let matches = match actor {
            AgentRef::Entity(id) => *id == context.actor,
            _ => false,
        };
        require(matches, "weapon command actor mismatch")?;
    }
    let item = equipment::carried_item(state, context.actor, choice.weapon)?;
    require(
        item.quantity == 1,
        "each physical weapon requires an individual ItemId",
    )?;
    let weapon = definitions
        .weapon(&item.definition_id)
        .ok_or_else(|| illegal("item is not a source weapon"))?;
    let has = |property| weapon.properties.contains(&property);
    match choice.delivery {
        WeaponDelivery::Melee => require(
            weapon.kind == WeaponKind::Melee,
            "improvised melee use needs its own source adjudication",
        )?,
        WeaponDelivery::Thrown => {
            require(has(WeaponProperty::Thrown), "weapon has no Thrown property")?
        }
        WeaponDelivery::Shot => require(
            has(WeaponProperty::Ammunition),
            "weapon does not fire ammunition",
        )?,
    }
    let default_ability = if weapon.kind == WeaponKind::Melee {
        Ability::Strength
    } else {
        Ability::Dexterity
    };
    require(
        choice.ability == default_ability
            || (has(WeaponProperty::Finesse)
                && matches!(choice.ability, Ability::Strength | Ability::Dexterity)),
        "weapon ability is not a permitted source choice",
    )?;
    let score = entity.ability_scores[match choice.ability {
        Ability::Strength => 0,
        Ability::Dexterity => 1,
        _ => unreachable!("validated ability"),
    }];
    let modifier = ability_modifier(score);
    let (proficient, pb, archery, mastery) = match &input.source {
        WeaponActorSource::Character(profile) => {
            require(
                profile.entity_id == context.actor,
                "source profile belongs to another actor",
            )?;
            validate_character_intrinsics(profile, entity, input.pack)
                .map_err(|error| invalid(error.to_string()))?;
            let category = match weapon.category {
                WeaponCategory::Simple => "simple",
                WeaponCategory::Martial => "martial",
            };
            let proficient = profile
                .weapon_proficiencies
                .iter()
                .any(|id| id == category || id == &weapon.id);
            let mastery = profile
                .features
                .iter()
                .any(|feature| feature.id == "weapon-mastery")
                && profile.masteries.contains(&weapon.id);
            (
                proficient,
                proficiency_bonus(entity.level),
                profile.fighting_style == FightingStyle::Archery,
                mastery.then_some(weapon.mastery),
            )
        }
        WeaponActorSource::CreatureOrdinaryWeapon(creature) => {
            require(
                definitions.creature(&creature.id) == Some(*creature),
                "creature source differs from pinned catalog",
            )?;
            require(
                entity.ability_scores == creature.statistics.ability_scores,
                "creature abilities differ from source",
            )?;
            (
                creature.statistics.gear.contains(&weapon.id),
                i32::from(creature.statistics.proficiency_bonus),
                false,
                None,
            )
        }
    };
    history::validate(input, weapon, mastery)?;
    let loadout_for_attack = equipment::before_attack(input, weapon)?;
    let ammunition = equipment::ammunition(input, weapon, &loadout_for_attack)?;
    let loadout_after_attack = equipment::after_attack(input, &loadout_for_attack)?;
    let reach = context
        .base_reach
        .checked_add(u32::from(weapon.reach_bonus_feet) * 2)
        .ok_or_else(|| invalid("weapon reach overflow"))?;
    let mut disadvantage = Vec::new();
    let mut automatic_miss = false;
    if choice.delivery == WeaponDelivery::Melee {
        require(context.distance <= reach, "target is beyond weapon reach")?;
        if context.underwater
            && !context.has_swim_speed
            && weapon.damage_type != DamageType::Piercing
        {
            disadvantage.push(WeaponDisadvantage::UnderwaterMelee);
        }
    } else {
        let range = weapon
            .range
            .ok_or_else(|| invalid("ranged weapon lacks source range"))?;
        require(
            context.distance <= u32::from(range.long_feet) * 2,
            "target is beyond weapon long range",
        )?;
        let long = context.distance > u32::from(range.normal_feet) * 2;
        if context.underwater {
            automatic_miss = long;
            disadvantage.push(WeaponDisadvantage::UnderwaterRanged);
        } else if long {
            disadvantage.push(WeaponDisadvantage::LongRange);
        }
    }
    if has(WeaponProperty::Heavy)
        && entity.ability_scores[if weapon.kind == WeaponKind::Melee {
            0
        } else {
            1
        }] < 13
    {
        disadvantage.push(WeaponDisadvantage::HeavyAbilityRequirement);
    }
    let formula = if choice.delivery == WeaponDelivery::Melee && choice.grip == WeaponGrip::TwoHands
    {
        weapon.versatile_damage.as_ref().unwrap_or(&weapon.damage)
    } else {
        &weapon.damage
    };
    let omit_positive = !matches!(choice.purpose, WeaponAttackPurpose::Normal);
    let damage_modifier = i32::from(formula.fixed)
        + if formula.dice.is_empty() {
            0
        } else if omit_positive {
            modifier.min(0)
        } else {
            modifier
        };
    let receipt = WeaponAttackReceipt {
        origin: context.origin.clone(),
        actor: context.actor,
        turn_number: context.turn_number,
        on_actor_turn: context.on_actor_turn,
        window: context.window,
        weapon: choice.weapon,
        definition_id: weapon.id.clone(),
        target: choice.target,
        delivery: choice.delivery,
        ability: choice.ability,
        grip: choice.grip,
        ammunition: choice.ammunition,
        purpose: choice.purpose,
        outcome: WeaponAttackOutcome::Pending,
    };
    Ok(WeaponAttackPlan {
        receipt,
        attack_modifier: modifier
            + if proficient { pb } else { 0 }
            + if archery && weapon.kind == WeaponKind::Ranged {
                2
            } else {
                0
            }
            - i32::from(entity.exhaustion) * 2,
        ability_modifier: modifier,
        proficiency_bonus: pb,
        proficient,
        damage: WeaponDamageProfile {
            dice: formula.dice.clone(),
            modifier: damage_modifier,
            damage_type: weapon.damage_type,
        },
        disadvantage,
        automatic_miss,
        mastery,
        reach,
        loadout_for_attack,
        ammunition,
        loadout_after_attack,
        thrown_weapon: (choice.delivery == WeaponDelivery::Thrown).then_some(choice.weapon),
    })
}
