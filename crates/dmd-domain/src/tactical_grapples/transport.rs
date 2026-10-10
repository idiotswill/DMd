//! Retained evidence for one grounded coupled movement, never movement permission.
use crate::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GrappleTransportKind {
    GroundDragV1,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GrappleTransportAdmission {
    pub origin: CommandMeta,
    pub kind: GrappleTransportKind,
    pub grip: GrappleId,
    pub holder: EntityId,
    pub target: EntityId,
    pub holder_from: SpatialPoint,
    pub target_from: SpatialPoint,
    pub path: Vec<TacticalMoveStep>,
    pub spent_before: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GrappleBodyDisplacement {
    pub actor: EntityId,
    pub from: SpatialPoint,
    pub to: SpatialPoint,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GrappleTransportStepReceipt {
    pub work: TacticalWorkKey,
    pub step: u16,
    pub cause: CommandMeta,
    pub holder: GrappleBodyDisplacement,
    pub target: GrappleBodyDisplacement,
    pub mode: MovementMode,
    pub ordinary_cost: u32,
    pub haul_cost: u32,
    pub holder_size: CreatureSize,
    pub target_size: CreatureSize,
}

impl GrappleTransportStepReceipt {
    pub fn total_cost(&self) -> Option<u32> {
        self.ordinary_cost.checked_add(self.haul_cost)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GrappleTransportStop {
    pub cause: CommandMeta,
    pub ended_grip: GrappleId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GrappleTransportHistory {
    pub admission: GrappleTransportAdmission,
    pub steps: Vec<GrappleTransportStepReceipt>,
    pub stop: Option<GrappleTransportStop>,
}

/// Coarse final evidence survives retirement of the detailed shared resolution.
/// Original typed action replay authenticates the sums and source relation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GrappleTransportResult {
    pub kind: GrappleTransportKind,
    pub grip: GrappleId,
    pub target: EntityId,
    pub target_start: SpatialPoint,
    pub target_endpoint: SpatialPoint,
    pub ordinary_cost: u32,
    pub haul_cost: u32,
}

impl GrappleTransportHistory {
    pub(crate) fn validate_shape(
        &self,
        resolution: &TacticalResolution,
        context: &TacticalGrappleResolution,
    ) -> Result<(), String> {
        let admission = &self.admission;
        let proof = context
            .proofs
            .iter()
            .find(|p| p.declaration.id == admission.grip)
            .ok_or("ground drag lost its original grip proof")?;
        if admission.origin != resolution.origin
            || admission.holder != resolution.turn_actor
            || admission.holder != proof.declaration.grappler
            || admission.target != proof.declaration.target
            || admission.path.is_empty()
            || admission.path.len() > 1024
            || self.steps.len() > admission.path.len()
            || admission.spent_before > 10_000
            || context.activity.is_some()
        {
            return Err("ground drag admission differs from its actual resolution".into());
        }
        admission.holder_from.validate()?;
        admission.target_from.validate()?;
        let mut prior = admission.holder_from;
        for step in &admission.path {
            step.destination.validate()?;
            let distance = prior
                .x
                .abs_diff(step.destination.x)
                .max(prior.y.abs_diff(step.destination.y));
            if !matches!(step.mode, MovementMode::Walk | MovementMode::Crawl)
                || step.destination.z != admission.holder_from.z
                || distance == 0
                || distance > 10
            {
                return Err("ground drag path is not bounded horizontal movement".into());
            }
            prior = step.destination;
        }
        let mut holder = admission.holder_from;
        let mut target = admission.target_from;
        let mut cause = &admission.origin;
        let mut spent = admission.spent_before;
        let mut previous_work = None;
        for (index, step) in self.steps.iter().enumerate() {
            let path = &admission.path[index];
            let node = resolution
                .work_trace
                .as_ref()
                .and_then(|trace| {
                    trace
                        .nodes
                        .iter()
                        .find(|node| node.work.occurrence == step.work.occurrence)
                })
                .ok_or("ground drag step has no entered work")?;
            let dx = i64::from(step.holder.to.x) - i64::from(holder.x);
            let dy = i64::from(step.holder.to.y) - i64::from(holder.y);
            let distance = u32::try_from(dx.abs().max(dy.abs()))
                .map_err(|_| "ground drag distance overflow")?;
            let haul = if step.target_size == CreatureSize::Tiny
                || step.holder_size.rank() - step.target_size.rank() >= 2
            {
                0
            } else {
                distance
            };
            if step.work.resolution != resolution.origin.id
                || node.work.kind != TacticalWorkKind::MoveSegment
                || previous_work.is_some_and(|old| step.work.occurrence <= old)
                || usize::from(step.step) != index
                || step.holder.actor != admission.holder
                || step.target.actor != admission.target
                || step.holder.from != holder
                || step.target.from != target
                || step.holder.to != path.destination
                || step.mode != path.mode
                || !matches!(step.mode, MovementMode::Walk | MovementMode::Crawl)
                || step.holder.to.z != holder.z
                || step.target.to.z != target.z
                || i64::from(step.target.to.x) - i64::from(target.x) != dx
                || i64::from(step.target.to.y) - i64::from(target.y) != dy
                || distance == 0
                || distance > 10
                || step.ordinary_cost < distance
                || step.ordinary_cost > 30
                || step.ordinary_cost % distance != 0
                || step.ordinary_cost / distance > 3
                || step.haul_cost != haul
                || step.cause.campaign_id != cause.campaign_id
                || step.cause.expected_event_sequence < cause.expected_event_sequence
                || (step.cause.expected_event_sequence == cause.expected_event_sequence
                    && &step.cause != cause)
                || (step.cause.id == cause.id && &step.cause != cause)
            {
                return Err(
                    "ground drag receipt differs from its paired step and source work".into(),
                );
            }
            step.holder.to.validate()?;
            step.target.to.validate()?;
            spent = spent
                .checked_add(step.total_cost().ok_or("ground drag cost overflow")?)
                .filter(|spent| *spent <= 10_000)
                .ok_or("ground drag expenditure exceeds its bound")?;
            holder = step.holder.to;
            target = step.target.to;
            cause = &step.cause;
            previous_work = Some(step.work.occurrence);
        }
        if let Some(movement) = &resolution.movement
            && (movement.origin != admission.origin
                || movement.actor != admission.holder
                || movement.path != admission.path
                || movement.initial_position != admission.holder_from
                || movement.initial_spent != admission.spent_before
                || movement.grapple_self_only.is_some()
                || movement.traversed.len() != self.steps.len()
                || movement.traversed.iter().zip(&self.steps).any(|(a, b)| {
                    a.cause != b.cause
                        || a.from != b.holder.from
                        || a.to != b.holder.to
                        || a.mode != b.mode
                        || Some(a.cost) != b.total_cost()
                }))
        {
            return Err("ground drag and ordinary movement receipts differ".into());
        }
        if let Some(stop) = &self.stop
            && (stop.cause.expected_event_sequence < cause.expected_event_sequence
                || (stop.cause.expected_event_sequence == cause.expected_event_sequence
                    && &stop.cause != cause))
        {
            return Err("ground drag moved after its selected relation ended".into());
        }
        let end = context.ends.iter().find(|end| end.grip == admission.grip);
        if self
            .stop
            .as_ref()
            .map(|stop| (stop.ended_grip, &stop.cause))
            != end.map(|end| (end.grip, &end.caused_by))
        {
            return Err("ground drag stop differs from its actual relation ending".into());
        }
        Ok(())
    }
}
