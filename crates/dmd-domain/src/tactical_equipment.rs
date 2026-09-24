//! Durable weapon identities and accepted action history; execution belongs to rules.
use crate::{Ability, CommandId, CommandMeta, EntityId, ItemId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Hand {
    Left,
    Right,
}
impl Hand {
    pub const fn index(self) -> usize {
        match self {
            Self::Left => 0,
            Self::Right => 1,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WeaponGrip {
    OneHand(Hand),
    TwoHands,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum HandAssignment {
    #[default]
    Free,
    Item(ItemId),
}
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WeaponLoadout {
    /// Left, right. A two-handed grip references the same physical ItemId twice.
    pub hands: [HandAssignment; 2],
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WeaponDelivery {
    Melee,
    Thrown,
    Shot,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WeaponActionKind {
    AttackAction,
    /// A source-defined attack from another action, including monster Multiattack.
    OtherAction,
    BonusAction,
    Reaction,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WeaponActionWindow {
    /// Identity of the accepted action opportunity, not each individual attack.
    pub id: CommandId,
    pub kind: WeaponActionKind,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WeaponAttackPurpose {
    Normal,
    LightBonus { trigger: CommandId },
    Nick { trigger: CommandId },
    Cleave { trigger: CommandId },
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EquipmentChangeTiming {
    BeforeAttack,
    AfterAttack,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AttackEquipmentOperation {
    Equip { item: ItemId, hand: Hand },
    Unequip { item: ItemId },
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AttackEquipmentChange {
    pub timing: EquipmentChangeTiming,
    pub operation: AttackEquipmentOperation,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WeaponUseChoice {
    pub weapon: ItemId,
    pub target: EntityId,
    pub delivery: WeaponDelivery,
    pub ability: Ability,
    pub grip: WeaponGrip,
    pub purpose: WeaponAttackPurpose,
    /// Physical ammunition stack selected by the controller, if the weapon needs it.
    pub ammunition: Option<ItemId>,
    pub equipment_change: Option<AttackEquipmentChange>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WeaponAttackOutcome {
    Pending,
    Miss,
    Hit { critical: bool, damage_dealt: u32 },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WeaponAttackReceipt {
    pub origin: CommandMeta,
    pub actor: EntityId,
    pub turn_number: u64,
    pub on_actor_turn: bool,
    pub window: WeaponActionWindow,
    pub weapon: ItemId,
    pub definition_id: String,
    pub target: EntityId,
    pub delivery: WeaponDelivery,
    pub ability: Ability,
    pub grip: WeaponGrip,
    pub ammunition: Option<ItemId>,
    pub purpose: WeaponAttackPurpose,
    pub outcome: WeaponAttackOutcome,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WeaponMasteryChoice {
    Decline,
    Graze,
    Push {
        distance: u32,
    },
    Slow,
    Topple,
    /// The selected extra attack must pass ordinary weapon preparation before commit.
    Cleave {
        attack: Box<WeaponUseChoice>,
    },
}
