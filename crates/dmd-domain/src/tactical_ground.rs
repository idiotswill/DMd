//! Additive equipment evidence. Private producers do not grant public or restore
//! authority; original accepted-command replay must authenticate these records.
use crate::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AttackEquipmentAccessVersion {
    GroundEquipmentV1,
}

/// Explicit encounter-local opt-in. Absence preserves every old flow5 projection.
/// Only original accepted-command replay can authenticate this origin.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TacticalAttackEquipmentAccess {
    pub version: AttackEquipmentAccessVersion,
    pub origin: CommandMeta,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AfterAttackEquipmentIntent {
    Choose,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AttackEquipmentPause {
    Knockout,
    Graze,
}

/// Actual entered work at an opted-in attack's material pause, not a prediction
/// of future completion. Original command replay must authenticate this evidence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AttackEquipmentCompletionParent {
    pub work: TacticalWorkKey,
    pub pause: AttackEquipmentPause,
    pub paused_by: CommandMeta,
    pub accepted_raw: Option<RollRequestId>,
    pub suspended_outcome: WeaponAttackOutcome,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum AttackEquipmentSource {
    Ordinary,
    Creature {
        source: CreatureSourcePin,
        feature_id: String,
    },
}

/// Declaration/completion evidence for an earned allowance. No caller supplies
/// this record, a remaining-allowance flag or numerical attack mechanics.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AttackEquipmentCause {
    pub origin: CommandMeta,
    pub actor: EntityId,
    pub turn_number: u64,
    pub window: WeaponActionWindow,
    pub choice: WeaponUseChoice,
    pub source: AttackEquipmentSource,
    pub outcome: WeaponAttackOutcome,
    pub completed_by: CommandMeta,
    pub completed_work: TacticalWorkKey,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TacticalAttackAfterEquipment {
    pub cause: AttackEquipmentCause,
    pub work: TacticalWorkItem,
    /// None means queued; only actual selection creates an owned material wait.
    pub selected_by: Option<CommandMeta>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AttackEquipmentChoice {
    Decline,
    Apply(AttackEquipmentOperation),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AttackEquipmentApplied {
    pub operation: AttackEquipmentOperation,
    pub equipment_before: ActorEquipmentLoadout,
    pub ground_before: Option<AttackGroundPickupBefore>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AttackAfterEquipmentReceipt {
    pub cause: AttackEquipmentCause,
    pub work: TacticalWorkKey,
    pub selected_by: CommandMeta,
    pub chosen_by: CommandMeta,
    /// None is an explicit Decline, not an unanswered operation.
    pub applied: Option<Box<AttackEquipmentApplied>>,
}

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
        .is_some_and(|flow| flow.attack_equipment_access.is_some())
        || has_attack_equipment_records(state)
}

/// Record inventory without rollout policy. Activation does not make a retained
/// receipt or an orphan frame disappear from this descriptive query.
pub fn has_attack_equipment_records(state: &CampaignState) -> bool {
    state
        .encounter
        .as_ref()
        .and_then(|e| e.flow.as_ref())
        .is_some_and(|flow| {
            flow.budget
                .weapon_history
                .iter()
                .any(|r| r.ground_pickup_before.is_some() || r.after_equipment.is_some())
                || flow.resolution.as_ref().is_some_and(|r| {
                    r.attack_after_equipment.is_some()
                        || r.frames
                            .iter()
                            .flatten()
                            .chain(r.pending.iter().map(|p| &p.work))
                            .chain(r.failed_save.iter().map(|p| &p.pending.work))
                            .chain(
                                r.work_trace
                                    .iter()
                                    .flat_map(|t| t.nodes.iter().map(|n| &n.work)),
                            )
                            .any(|w| w.kind == TacticalWorkKind::AttackAfterEquipment)
                })
                || flow
                    .resolution
                    .as_ref()
                    .and_then(|r| r.attack.as_ref())
                    .and_then(TacticalAttack::weapon)
                    .is_some_and(|w| {
                        w.ground_pickup_before.is_some()
                            || w.after_equipment_parent.is_some()
                            || w.choice.after_equipment.is_some()
                            || is_ground_pickup(w.choice.equipment_change)
                    })
        })
}
