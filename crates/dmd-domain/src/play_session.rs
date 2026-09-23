use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::{CampaignId, CharacterId, PlaySessionId, PlayerId, WorldInstant};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlaySessionStatus {
    Active,
    Closed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AttendanceStatus {
    Present,
    Absent,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionParticipant {
    pub player_id: PlayerId,
    /// None is valid during character creation, after a death, or for a table participant
    /// who is present without controlling a PC in this session.
    pub character_id: Option<CharacterId>,
    pub attendance: AttendanceStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlaySession {
    pub id: PlaySessionId,
    pub campaign_id: CampaignId,
    pub display_name: String,
    pub status: PlaySessionStatus,
    /// In-world time at which play begins/resumes. Wall-clock timestamps belong to telemetry.
    pub started_at_world: WorldInstant,
    pub ended_at_world: Option<WorldInstant>,
    pub participants: Vec<SessionParticipant>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlaySessionShapeViolation {
    DuplicatePlayer(PlayerId),
    ActiveSessionHasEndTime,
    ClosedSessionMissingEndTime,
    EndBeforeStart,
}

impl PlaySession {
    #[must_use]
    pub fn validate_shape(&self) -> Vec<PlaySessionShapeViolation> {
        let mut violations = Vec::new();
        let mut players = HashSet::new();

        for participant in &self.participants {
            if !players.insert(participant.player_id) {
                violations.push(PlaySessionShapeViolation::DuplicatePlayer(
                    participant.player_id,
                ));
            }
        }

        match (self.status, self.ended_at_world) {
            (PlaySessionStatus::Active, Some(_)) => {
                violations.push(PlaySessionShapeViolation::ActiveSessionHasEndTime);
            }
            (PlaySessionStatus::Closed, None) => {
                violations.push(PlaySessionShapeViolation::ClosedSessionMissingEndTime);
            }
            _ => {}
        }

        if let Some(ended_at) = self.ended_at_world {
            if ended_at < self.started_at_world {
                violations.push(PlaySessionShapeViolation::EndBeforeStart);
            }
        }

        violations
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn absence_is_session_attendance_not_character_lifecycle() {
        let participant = SessionParticipant {
            player_id: PlayerId::new(),
            character_id: Some(CharacterId::new()),
            attendance: AttendanceStatus::Absent,
        };

        assert_eq!(participant.attendance, AttendanceStatus::Absent);
    }

    #[test]
    fn active_session_has_no_end_time() {
        let session = PlaySession {
            id: PlaySessionId::new(),
            campaign_id: CampaignId::new(),
            display_name: "Test Session".into(),
            status: PlaySessionStatus::Active,
            started_at_world: WorldInstant(10),
            ended_at_world: None,
            participants: vec![],
        };

        assert!(session.validate_shape().is_empty());
    }
}
