//! Released interval evidence for the existing tactical continuation stack.
//! These records are consistency data. Accepted command replay authenticates them.
use crate::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReleasedTimeOrdering {
    HostSelect,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReleasedTimeUpgrade {
    pub origin: CommandMeta,
    pub release: CommandId,
    pub from: TacticalExecutionVersion,
    pub to: TacticalExecutionVersion,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum ReleasedDeadlineSource {
    Effect {
        effect: EffectId,
        source: EffectSource,
        established_at: EffectOperationStamp,
        group: Option<EffectId>,
    },
    Group {
        group: EffectId,
        source: EffectSource,
    },
    Legacy {
        effect: EffectId,
        source: EntityId,
        target: EntityId,
        concentration_owner: Option<EntityId>,
    },
    Stable {
        actor: EntityId,
        origin: VitalityOrigin,
        stabilized_at: WorldInstant,
        roll: RollRequestId,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReleasedDeadlineBinding {
    pub source: ReleasedDeadlineSource,
    pub due_at: WorldInstant,
    pub work: TacticalWorkItem,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum ReleasedWorkOutcome {
    Applied,
    CancelledBy { work: TacticalWorkKey },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReleasedWorkCompletion {
    pub work: TacticalWorkKey,
    pub completed_by: CommandMeta,
    pub outcome: ReleasedWorkOutcome,
}

/// Immutable source/partition evidence, not a second executable work queue.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReleasedDeadlineBatch {
    pub ordinal: u16,
    pub at: WorldInstant,
    pub observed: EffectOperationStamp,
    pub bindings: Vec<ReleasedDeadlineBinding>,
    pub completions: Vec<ReleasedWorkCompletion>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReleasedElapsedContext {
    pub release: CommandId,
    pub started_at: WorldInstant,
    pub target_at: WorldInstant,
    pub progress_at: WorldInstant,
    pub ordering: ReleasedTimeOrdering,
    pub ruling: String,
    pub predecessor: Option<CommandId>,
    pub batches: Vec<ReleasedDeadlineBatch>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReleasedElapsedReceipt {
    pub origin: CommandMeta,
    pub release: CommandId,
    pub predecessor: Option<CommandId>,
    pub started_at: WorldInstant,
    pub target_at: WorldInstant,
    pub ordering: ReleasedTimeOrdering,
    pub ruling: String,
    pub completed_by: CommandMeta,
    pub completed_at: WorldInstant,
}

pub fn valid_released_time_ruling(ruling: &str) -> bool {
    !ruling.is_empty()
        && ruling.trim() == ruling
        && ruling.len() <= 2000
        && !ruling.chars().any(char::is_control)
}
