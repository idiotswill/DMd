//! Durable turn work is source-derived by rules, never a public state-patch command.
use crate::{
    CommandId, CommandMeta, EffectId, EffectTicketId, EntityId, RollRequestId, TurnBoundary,
    VitalityOrigin,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TacticalRollRole {
    DeathSave,
    EffectSave,
    EffectDamage,
    Concentration,
    StableRecovery,
    CreatureRecharge,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TacticalRollKey {
    /// Original accepted boundary/attack command, retained across later submissions.
    pub origin: CommandId,
    pub role: TacticalRollRole,
    /// A damage roll can be rolled by its source while this identifies its victim.
    pub subject: EntityId,
    pub occurrence: u16,
}
impl TacticalRollKey {
    /// Fixed binary namespace and role tags are replay semantics. Never use random UUIDs
    /// or a platform-dependent hash for requests created inside an accepted transition.
    pub fn request_id(self) -> RollRequestId {
        let tag = match self.role {
            TacticalRollRole::DeathSave => 1,
            TacticalRollRole::EffectSave => 2,
            TacticalRollRole::EffectDamage => 3,
            TacticalRollRole::Concentration => 4,
            TacticalRollRole::StableRecovery => 5,
            TacticalRollRole::CreatureRecharge => 6,
        };
        let mut bytes = b"dmd.tactical.roll.v1\0".to_vec();
        bytes.push(tag);
        bytes.extend_from_slice(self.subject.0.as_bytes());
        bytes.extend_from_slice(&self.occurrence.to_be_bytes());
        RollRequestId(Uuid::new_v5(&self.origin.0, &bytes))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum TacticalWorkKind {
    DeathSave {
        actor: EntityId,
    },
    Effect {
        ticket: EffectTicketId,
    },
    ConcentrationSave {
        actor: EntityId,
        group: EffectId,
        damage_taken: u32,
    },
    StableRecovery {
        actor: EntityId,
        origin: VitalityOrigin,
    },
    RecoverStable {
        actor: EntityId,
    },
    CreatureRecharge {
        actor: EntityId,
        feature_id: String,
    },
    /// Post-End opportunity, after End effects. Does not spend a source use until
    /// a feature is actually selected; the creature's controller may decline.
    LegendaryWindow {
        actor: EntityId,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TacticalWorkItem {
    /// Unique within this resolution, allocated deterministically when work is queued.
    pub occurrence: u16,
    pub kind: TacticalWorkKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TacticalPendingWork {
    pub work: TacticalWorkItem,
    pub key: TacticalRollKey,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TacticalFailedSave {
    pub pending: TacticalPendingWork,
    pub issued_by: CommandMeta,
    pub resolved_by: CommandMeta,
    /// None records an automatic or voluntarily chosen failure. A source feature
    /// changes the outcome, never these accepted physical/digital die faces.
    pub result: Option<crate::RollResult>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TacticalLegendaryWindow {
    pub work: TacticalWorkItem,
    pub origin: CommandMeta,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TacticalResolution {
    pub origin: CommandMeta,
    pub turn_actor: EntityId,
    pub turn_number: u64,
    pub boundary: TurnBoundary,
    /// Last frame is current. Nested damage/concentration consequences finish before
    /// resuming the remaining parent boundary's simultaneous work.
    pub frames: Vec<Vec<TacticalWorkItem>>,
    pub pending: Option<TacticalPendingWork>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failed_save: Option<TacticalFailedSave>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub legendary_window: Option<TacticalLegendaryWindow>,
    pub next_occurrence: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TacticalSaveFailure {
    Automatic,
    Voluntary,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TacticalSaveDecision {
    pub key: TacticalRollKey,
    pub issued_by: CommandMeta,
    pub resolved_by: CommandMeta,
    /// No die was rolled. Do not manufacture a RecordedRoll or a natural-1 face.
    pub failure: TacticalSaveFailure,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TacticalDodge {
    pub actor: EntityId,
    pub origin: CommandMeta,
    pub declared_on_turn: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TacticalGroundItem {
    pub item: crate::ItemId,
    pub position: crate::SpatialPoint,
    pub origin: CommandMeta,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn request_identity_survives_serialization_and_separates_roles_subjects_and_occurrences() {
        let key = TacticalRollKey {
            origin: CommandId::new(),
            role: TacticalRollRole::DeathSave,
            subject: EntityId::new(),
            occurrence: 0,
        };
        let restored: TacticalRollKey =
            serde_json::from_slice(&serde_json::to_vec(&key).unwrap()).unwrap();
        assert_eq!(key.request_id(), restored.request_id());
        assert_ne!(
            key.request_id(),
            TacticalRollKey {
                role: TacticalRollRole::Concentration,
                ..key
            }
            .request_id()
        );
        assert_ne!(
            key.request_id(),
            TacticalRollKey {
                occurrence: 1,
                ..key
            }
            .request_id()
        );
        assert_ne!(
            key.request_id(),
            TacticalRollKey {
                subject: EntityId::new(),
                ..key
            }
            .request_id()
        );
        assert_ne!(
            key.request_id(),
            TacticalRollKey {
                origin: CommandId::new(),
                ..key
            }
            .request_id()
        );
    }
}
