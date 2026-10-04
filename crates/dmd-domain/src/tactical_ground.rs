//! Additive pickup evidence. No producer or retained authority is enabled yet.
use crate::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AttackGroundPickupBefore {
    pub item: ItemInstance,
    pub ground: TacticalGroundItem,
    /// Preserve exact bounded vector order during the local inverse.
    pub ground_index: u32,
    pub encounter: EncounterId,
    pub scene: SceneId,
    pub location: LocationId,
    pub equipment: ActorEquipmentLoadout,
}

pub fn is_ground_pickup(change: Option<AttackEquipmentChange>) -> bool {
    change.is_some_and(|change| matches!(change.operation, AttackEquipmentOperation::Pickup { .. }))
}

/// Check both encoded choices and receipts: neither may become authority at an
/// earliest restore anchor, through a historical command, or after attack retirement.
pub fn has_unimplemented_ground_records(state: &CampaignState) -> bool {
    state
        .encounter
        .as_ref()
        .and_then(|e| e.flow.as_ref())
        .is_some_and(|flow| {
            flow.budget
                .weapon_history
                .iter()
                .any(|r| r.ground_pickup_before.is_some())
                || flow
                    .resolution
                    .as_ref()
                    .and_then(|r| r.attack.as_ref())
                    .and_then(TacticalAttack::weapon)
                    .is_some_and(|w| {
                        w.ground_pickup_before.is_some()
                            || is_ground_pickup(w.choice.equipment_change)
                    })
        })
}
