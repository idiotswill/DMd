//! Source-derived attack receipts inside the shared tactical resolution.
//! These are internal durable work, never player-supplied mechanical permissions.
use crate::{
    Ability, ActorEquipmentLoadout, CommandMeta, DamageType, DieSpec, EntityId, ItemId, RollMode,
    RollRequestId, WeaponActionWindow, WeaponAttackOutcome, WeaponUseChoice,
};
use serde::{Deserialize, Serialize};

/// Controller choice for one retained opportunity. Actor and target come from the
/// live movement window; this does not grant a reaction or a source capability.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum TacticalMeleeChoice {
    Weapon(WeaponUseChoice),
    /// The damage form of an Unarmed Strike. Grapple/Shove keep their distinct
    /// source save/size/hand choices and are not silently treated as damage here.
    UnarmedDamage {
        ability: Ability,
    },
    CreatureFeature {
        feature_id: String,
        /// Only an attack whose canonical Gear requires it accepts an ItemId.
        weapon: Option<ItemId>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AttackAmmunitionReservation {
    pub stack: ItemId,
    pub quantity_before: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AttackDamageComponent {
    pub damage_type: DamageType,
    pub dice: Vec<DieSpec>,
    pub modifier: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TacticalAttackStage {
    AttackRoll,
    DamageRoll,
    KnockoutChoice,
    Finishing,
    MasteryChoice,
}

/// Reconstructed against canonical source and original choices before damage changes
/// the target. Completed hit facts remain tied to accepted raw history and journal
/// replay; they cannot authenticate an imported after-state snapshot by themselves.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TacticalAttack {
    pub origin: CommandMeta,
    pub actor: EntityId,
    pub choice: WeaponUseChoice,
    pub window: WeaponActionWindow,
    pub equipment_before: ActorEquipmentLoadout,
    pub ammunition: Option<AttackAmmunitionReservation>,
    pub attack_modifier: i32,
    pub mode: RollMode,
    pub armor_class: i32,
    pub critical_on_hit: bool,
    pub automatic_miss: bool,
    pub damage: Vec<AttackDamageComponent>,
    pub stage: TacticalAttackStage,
    pub attack_roll: Option<RollRequestId>,
    pub damage_roll: Option<RollRequestId>,
    pub outcome: Option<WeaponAttackOutcome>,
}
