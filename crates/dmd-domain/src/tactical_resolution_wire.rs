//! Explicit compatibility adapter: old Turn fields retain their order and omission.
//! Typed decoding rejects duplicate, mixed, null, partial and unknown context.
use super::*;
use serde::{Deserializer, Serializer, ser::SerializeStruct};

enum Present<T> {
    Missing,
    Value(T),
}
impl<T> Default for Present<T> {
    fn default() -> Self {
        Self::Missing
    }
}
impl<'de, T: Deserialize<'de>> Deserialize<'de> for Present<T> {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        T::deserialize(d).map(Self::Value)
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Wire {
    origin: CommandMeta,
    #[serde(default)]
    turn_actor: Present<EntityId>,
    #[serde(default)]
    turn_number: Present<u64>,
    #[serde(default)]
    boundary: Present<TurnBoundary>,
    frames: Vec<Vec<TacticalWorkItem>>,
    pending: Option<TacticalPendingWork>,
    #[serde(default)]
    failed_save: Option<TacticalFailedSave>,
    #[serde(default)]
    legendary_window: Option<TacticalLegendaryWindow>,
    #[serde(default)]
    attack: Option<crate::TacticalAttack>,
    #[serde(default)]
    hit_review: Option<Box<crate::TacticalHitReview>>,
    #[serde(default)]
    movement: Option<Box<crate::TacticalMovement>>,
    #[serde(default)]
    casts: Vec<crate::TacticalCasting>,
    #[serde(default)]
    missiles: Vec<crate::TacticalMissile>,
    #[serde(default)]
    falls: Vec<crate::TacticalFall>,
    #[serde(default)]
    areas: Vec<crate::TacticalArea>,
    #[serde(default)]
    work_trace: Option<crate::TacticalWorkTrace>,
    next_occurrence: u16,
    #[serde(default)]
    released_interval: Present<Box<crate::ReleasedElapsedContext>>,
}

impl Serialize for TacticalResolution {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut count = 4 + match &self.context {
            TacticalResolutionContext::Turn(_) => 3,
            TacticalResolutionContext::ReleasedInterval(_) => 1,
        };
        count += usize::from(self.failed_save.is_some());
        count += usize::from(self.legendary_window.is_some());
        count += usize::from(self.attack.is_some());
        count += usize::from(self.hit_review.is_some());
        count += usize::from(self.movement.is_some());
        count += usize::from(self.work_trace.is_some());
        count += usize::from(!self.casts.is_empty());
        count += usize::from(!self.missiles.is_empty());
        count += usize::from(!self.falls.is_empty());
        count += usize::from(!self.areas.is_empty());
        let mut wire = serializer.serialize_struct("TacticalResolution", count)?;
        wire.serialize_field("origin", &self.origin)?;
        match &self.context {
            TacticalResolutionContext::Turn(turn) => {
                wire.serialize_field("turn_actor", &turn.actor)?;
                wire.serialize_field("turn_number", &turn.number)?;
                wire.serialize_field("boundary", &turn.boundary)?;
            }
            TacticalResolutionContext::ReleasedInterval(interval) => {
                wire.serialize_field("released_interval", interval)?
            }
        }
        wire.serialize_field("frames", &self.frames)?;
        wire.serialize_field("pending", &self.pending)?;
        if self.failed_save.is_some() {
            wire.serialize_field("failed_save", &self.failed_save)?;
        }
        if self.legendary_window.is_some() {
            wire.serialize_field("legendary_window", &self.legendary_window)?;
        }
        if self.attack.is_some() {
            wire.serialize_field("attack", &self.attack)?;
        }
        if self.hit_review.is_some() {
            wire.serialize_field("hit_review", &self.hit_review)?;
        }
        if self.movement.is_some() {
            wire.serialize_field("movement", &self.movement)?;
        }
        if !self.casts.is_empty() {
            wire.serialize_field("casts", &self.casts)?;
        }
        if !self.missiles.is_empty() {
            wire.serialize_field("missiles", &self.missiles)?;
        }
        if !self.falls.is_empty() {
            wire.serialize_field("falls", &self.falls)?;
        }
        if !self.areas.is_empty() {
            wire.serialize_field("areas", &self.areas)?;
        }
        if self.work_trace.is_some() {
            wire.serialize_field("work_trace", &self.work_trace)?;
        }
        wire.serialize_field("next_occurrence", &self.next_occurrence)?;
        wire.end()
    }
}

impl<'de> Deserialize<'de> for TacticalResolution {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let wire = Wire::deserialize(deserializer)?;
        let context = match (
            wire.turn_actor,
            wire.turn_number,
            wire.boundary,
            wire.released_interval,
        ) {
            (
                Present::Value(actor),
                Present::Value(number),
                Present::Value(boundary),
                Present::Missing,
            ) => TacticalResolutionContext::Turn(TacticalTurnContext {
                actor,
                number,
                boundary,
            }),
            (Present::Missing, Present::Missing, Present::Missing, Present::Value(interval)) => {
                TacticalResolutionContext::ReleasedInterval(interval)
            }
            _ => {
                return Err(serde::de::Error::custom(
                    "resolution requires exactly one complete context",
                ));
            }
        };
        Ok(Self {
            context,
            origin: wire.origin,
            frames: wire.frames,
            pending: wire.pending,
            failed_save: wire.failed_save,
            legendary_window: wire.legendary_window,
            attack: wire.attack,
            hit_review: wire.hit_review,
            movement: wire.movement,
            casts: wire.casts,
            missiles: wire.missiles,
            falls: wire.falls,
            areas: wire.areas,
            work_trace: wire.work_trace,
            next_occurrence: wire.next_occurrence,
        })
    }
}
