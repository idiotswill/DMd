use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct WorldInstant(pub i64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct WorldDuration(pub i64);

impl WorldInstant {
    #[must_use]
    pub const fn advance(self, duration: WorldDuration) -> Self {
        Self(self.0 + duration.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorldClock {
    pub now: WorldInstant,
    /// Presentation/calendar interpretation belongs to content, not to core time arithmetic.
    pub calendar_id: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn world_time_advances_without_calendar_assumptions() {
        assert_eq!(WorldInstant(10).advance(WorldDuration(7)), WorldInstant(17));
    }
}
