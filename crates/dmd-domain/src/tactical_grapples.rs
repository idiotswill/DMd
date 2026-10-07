//! Proposed ordinary-grip authority and retained causal evidence. These records do
//! not execute a grapple or authenticate a history; rules and original replay do.
mod validation;

use crate::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const TACTICAL_GRAPPLES_SCHEMA_VERSION: u32 = 1;

/// Reviewed normalization of two ordinary usable hand roles, not an inference
/// from equipment slots, creature type, or the absence of another anatomy field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrdinaryHandAnatomy {
    TwoHandsV1,
}

impl OrdinaryHandAnatomy {
    pub const fn hands(self) -> [Hand; 2] {
        match self {
            Self::TwoHandsV1 => [Hand::Left, Hand::Right],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum GrappleAnatomyProof {
    /// Rules must reconstruct the actual admitted Human profile and mechanics.
    HumanCreationV1 { character: CharacterId },
    /// Only a fully pinned immutable definition can grant source anatomy.
    Creature {
        source: CreatureSourcePin,
        ordinary_hands: OrdinaryHandAnatomy,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct GrappleId(pub Uuid);

impl GrappleId {
    pub fn from_declaration(
        origin: CommandId,
        grappler: EntityId,
        target: EntityId,
        hand: Hand,
    ) -> Self {
        let mut bytes = b"dmd.tactical.grip.v1\0".to_vec();
        bytes.extend_from_slice(grappler.0.as_bytes());
        bytes.extend_from_slice(target.0.as_bytes());
        bytes.push(match hand {
            Hand::Left => 0,
            Hand::Right => 1,
        });
        Self(Uuid::new_v5(&origin.0, &bytes))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GrappleSaveAbility {
    Strength,
    Dexterity,
}

impl GrappleSaveAbility {
    pub const fn ability(self) -> Ability {
        match self {
            Self::Strength => Ability::Strength,
            Self::Dexterity => Ability::Dexterity,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GrappleEscapeChoice {
    Athletics,
    Acrobatics,
}

impl GrappleEscapeChoice {
    pub const fn test(self) -> (Ability, Skill) {
        match self {
            Self::Athletics => (Ability::Strength, Skill::Athletics),
            Self::Acrobatics => (Ability::Dexterity, Skill::Acrobatics),
        }
    }
}

/// Original paid declaration. Range/DC and anatomy are derived by rules, never
/// caller permissions. The target's source may be an unannotated old revision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TacticalGrappleDeclaration {
    pub id: GrappleId,
    pub origin: CommandMeta,
    pub grappler: EntityId,
    pub target: EntityId,
    pub hand: Hand,
    pub window: WeaponActionWindow,
    pub anatomy: GrappleAnatomyProof,
    pub target_source: Option<CreatureSourcePin>,
    pub grappler_from: SpatialPoint,
    pub target_from: SpatialPoint,
    /// Half-foot units, as elsewhere in physical geometry.
    pub range: u32,
    pub escape_dc: i32,
}

/// References canonical raw history or the existing no-die save decision. A
/// voluntary/automatic failure never requires a fabricated RecordedRoll.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum GrappleSaveEvidence {
    Physical {
        key: TacticalRollKey,
        accepted_by: CommandMeta,
    },
    Decision(TacticalSaveDecision),
}

impl GrappleSaveEvidence {
    pub fn key(&self) -> TacticalRollKey {
        match self {
            Self::Physical { key, .. } => *key,
            Self::Decision(decision) => decision.key,
        }
    }
    pub fn resolved_by(&self) -> &CommandMeta {
        match self {
            Self::Physical { accepted_by, .. } => accepted_by,
            Self::Decision(decision) => &decision.resolved_by,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GrappleLegendaryDecision {
    pub chosen_by: CommandMeta,
    pub use_resistance: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GrappleSaveProof {
    pub evidence: GrappleSaveEvidence,
    pub legendary: Option<GrappleLegendaryDecision>,
    pub final_success: bool,
    pub finalized_by: CommandMeta,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TacticalGrappleSave {
    pub ability: GrappleSaveAbility,
    pub chosen_by: CommandMeta,
    pub key: TacticalRollKey,
    /// None only for a genuine source automatic failure, not a natural-one roll.
    pub request: Option<RollRequest>,
    pub proof: Option<GrappleSaveProof>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TacticalGrip {
    pub declaration: TacticalGrappleDeclaration,
    pub established_by: CommandMeta,
    /// May refer to an earlier retired resolution. Paid roll origin is separate.
    pub work: TacticalWorkKey,
    pub save: TacticalGrappleSave,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TacticalGrapples {
    pub schema_version: u32,
    /// Canonical ascending grip identity. An empty live attachment is not valid.
    pub active: Vec<TacticalGrip>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TacticalGrappleAttemptStage {
    Queued,
    SaveChoice,
    Saving,
    Resolving,
    AfterEquipment,
    Complete,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum GrappleAttemptOutcome {
    Resisted {
        resolved_by: CommandMeta,
    },
    Immune {
        resolved_by: CommandMeta,
    },
    Established {
        grip: GrappleId,
    },
    Withdrawn {
        withdrawn_by: CommandMeta,
        cancelled: Option<RollRequestId>,
    },
}

/// The one Attack-action equipment allowance; the before operation is owned by
/// the paid declaration. An after decision retains its actual selected work.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GrappleEquipmentAdmission {
    pub equipment_before: ActorEquipmentLoadout,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub before_change: Option<AttackEquipmentOperation>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub after: Option<GrappleEquipmentDecision>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum GrappleEquipmentDecision {
    Declined {
        chosen_by: CommandMeta,
        work: TacticalWorkKey,
    },
    Applied {
        chosen_by: CommandMeta,
        work: TacticalWorkKey,
        operation: AttackEquipmentOperation,
        equipment_before: Box<ActorEquipmentLoadout>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TacticalGrappleAttempt {
    pub equipment: GrappleEquipmentAdmission,
    pub declaration: TacticalGrappleDeclaration,
    pub stage: TacticalGrappleAttemptStage,
    pub selected: Option<TacticalWorkItem>,
    pub save: Option<TacticalGrappleSave>,
    pub outcome: Option<GrappleAttemptOutcome>,
}

impl TacticalGrappleAttempt {
    /// The exact attempt identity lets its own admission reconstruction recognize
    /// this reservation. An unrelated action must never ignore it.
    pub fn reservation(&self) -> Option<(GrappleId, EntityId, Hand)> {
        self.outcome.is_none().then_some((
            self.declaration.id,
            self.declaration.grappler,
            self.declaration.hand,
        ))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TacticalGrappleEscapeStage {
    Queued,
    Rolling,
    Resolving,
    Complete,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum GrappleEscapeOutcome {
    Checked {
        resolved_by: CommandMeta,
        succeeded: bool,
    },
    Obsolete {
        ended_by: CommandMeta,
        cancelled: Option<RollRequestId>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TacticalGrappleEscape {
    pub origin: CommandMeta,
    pub actor: EntityId,
    pub grip: GrappleId,
    pub choice: GrappleEscapeChoice,
    pub difficulty: i32,
    pub key: TacticalRollKey,
    pub request: Option<RollRequest>,
    pub selected: Option<TacticalWorkItem>,
    pub stage: TacticalGrappleEscapeStage,
    pub outcome: Option<GrappleEscapeOutcome>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum GrappleActivity {
    Attempt(Box<TacticalGrappleAttempt>),
    Escape(Box<TacticalGrappleEscape>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum GrappleReader {
    AttackAdmission {
        attack: CommandId,
    },
    OpportunityWindow {
        attack: CommandId,
        window: CommandId,
        reactor: EntityId,
        mover: EntityId,
        step: u16,
    },
    RequestIssue {
        roll: TacticalRollKey,
    },
    FlightLoss {
        actor: EntityId,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GrappleCutKey {
    pub work: TacticalWorkKey,
    pub reader: GrappleReader,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GrappleReadCut {
    pub key: GrappleCutKey,
    pub issued_by: CommandMeta,
    /// Complete relevant relation set; rules/replay authenticate completeness.
    pub grips: Vec<GrappleId>,
    /// Direct same-attack AttackAdmission only; no recursive/request inheritance.
    pub source_attack: Option<GrappleCutKey>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum GrappleEndCause {
    Released,
    Escaped {
        roll: TacticalRollKey,
        work: TacticalWorkKey,
    },
    Incapacitated {
        work: TacticalWorkKey,
    },
    Dead {
        work: TacticalWorkKey,
    },
    OutOfRange {
        work: TacticalWorkKey,
        moved_actor: EntityId,
    },
}

impl GrappleEndCause {
    pub fn work(&self) -> Option<TacticalWorkKey> {
        match self {
            Self::Released => None,
            Self::Escaped { work, .. }
            | Self::Incapacitated { work }
            | Self::Dead { work }
            | Self::OutOfRange { work, .. } => Some(*work),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GrappleEndReceipt {
    pub grip: GrappleId,
    pub caused_by: CommandMeta,
    pub cause: GrappleEndCause,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GrappleOpportunityRefresh {
    pub work: TacticalWorkKey,
    pub movement_origin: CommandMeta,
    pub window_origin: CommandMeta,
    pub reactor: EntityId,
    pub step: u16,
    /// Unique grip ending in this resolution; the receipt owns its actual cause.
    pub ended_grip: GrappleId,
    pub previous: Vec<TacticalMeleeOption>,
    pub resulting: Vec<TacticalMeleeOption>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GrappleSelfOnlyAdmission {
    pub origin: CommandMeta,
    pub grips: Vec<GrappleId>,
}

/// Completed source evidence retained only while its attack reads still belong
/// to this resolution. It is never executable casting work or a reservation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GrappleCompletedCast {
    pub record: Box<TacticalCasting>,
    pub work: TacticalWorkKey,
    pub finished_by: CommandMeta,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TacticalGrappleResolution {
    pub activity: Option<GrappleActivity>,
    pub proofs: Vec<TacticalGrip>,
    pub cuts: Vec<GrappleReadCut>,
    pub ends: Vec<GrappleEndReceipt>,
    pub opportunity_refreshes: Vec<GrappleOpportunityRefresh>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub completed_casts: Vec<GrappleCompletedCast>,
}

impl TacticalGrappleResolution {
    pub fn is_empty(&self) -> bool {
        self.activity.is_none()
            && self.proofs.is_empty()
            && self.cuts.is_empty()
            && self.ends.is_empty()
            && self.opportunity_refreshes.is_empty()
            && self.completed_casts.is_empty()
    }
}

/// Includes orphaned retained fields so absence of live grips cannot hide new
/// provisional/historical authority from version and recovery-anchor guards.
pub fn has_tactical_grapple_attachments(state: &CampaignState) -> bool {
    state
        .rules
        .as_ref()
        .is_some_and(|r| r.tactical_grapples.is_some())
        || state
            .encounter
            .as_ref()
            .and_then(|e| e.flow.as_ref())
            .and_then(|f| f.resolution.as_ref())
            .is_some_and(|r| {
                r.grapple.is_some()
                    || r.movement
                        .as_ref()
                        .is_some_and(|m| m.grapple_self_only.is_some())
                    || r.falls
                        .iter()
                        .any(|f| matches!(f.cause, TacticalFallCause::GrappleFlightLost { .. }))
            })
}

/// Temporary source/domain checkpoint boundary. New raw roles have no accepted
/// producer yet, including when a forged history omits all other new fields.
pub fn has_unimplemented_grapple_records(state: &CampaignState) -> bool {
    fn new_role(role: TacticalRollRole) -> bool {
        matches!(
            role,
            TacticalRollRole::GrappleSave | TacticalRollRole::GrappleEscape
        )
    }
    fn role(purpose: &PendingPurpose) -> bool {
        matches!(purpose, PendingPurpose::TacticalResolution { key, .. }
            if new_role(key.role))
    }
    has_tactical_grapple_attachments(state)
        || state.rules.as_ref().is_some_and(|r| {
            r.pending.as_ref().is_some_and(|p| role(&p.purpose))
                || r.rolls.iter().any(|p| role(&p.purpose))
        })
        || state
            .encounter
            .as_ref()
            .and_then(|e| e.flow.as_ref())
            .is_some_and(|f| {
                f.save_decisions.iter().any(|d| new_role(d.key.role))
                    || f.resolution.as_ref().is_some_and(|r| {
                        r.pending.as_ref().is_some_and(|p| new_role(p.key.role))
                            || r.failed_save
                                .as_ref()
                                .is_some_and(|p| new_role(p.pending.key.role))
                    })
            })
}

/// This validates only retained shape. An explicit table marker permits empty
/// occurrence reads, while runnable saves still require original rules replay.
pub fn validate_tactical_grapple_shapes(state: &CampaignState) -> Result<(), String> {
    if !has_tactical_grapple_attachments(state) {
        return Ok(());
    }
    let flow = state
        .encounter
        .as_ref()
        .and_then(|e| e.flow.as_ref())
        .ok_or("grapple attachment requires its tactical executor")?;
    if state.schema_version != CURRENT_STATE_SCHEMA_VERSION || flow.version != 5 {
        return Err("grapple authority is not defined for this schema/executor".into());
    }
    let live = state
        .rules
        .as_ref()
        .and_then(|r| r.tactical_grapples.as_ref());
    if let Some(live) = live {
        live.validate_shape()?;
        if live
            .active
            .iter()
            .any(|grip| grip.declaration.origin.campaign_id != state.campaign_id())
        {
            return Err("live grip belongs to another campaign".into());
        }
    }
    if let Some(resolution) = &flow.resolution {
        if let Some(context) = &resolution.grapple {
            if let Some(access) = state
                .table
                .as_ref()
                .and_then(|table| table.grapple_access.as_ref())
            {
                access.validate(state)?;
                context.validate_activated_shape(resolution, live)?;
            } else {
                context.validate_shape(resolution, live)?;
            }
        } else if resolution
            .movement
            .as_ref()
            .is_some_and(|m| m.grapple_self_only.is_some())
            || resolution
                .falls
                .iter()
                .any(|f| matches!(f.cause, TacticalFallCause::GrappleFlightLost { .. }))
        {
            return Err("grapple consumer has no retained proof context".into());
        }
    }
    Ok(())
}
