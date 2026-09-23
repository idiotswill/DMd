use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::{CampaignId, CampaignState, CharacterId, PlaySessionId, PlayerId, WorldInstant};

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
    DuplicateCharacter(CharacterId),
    ActiveSessionHasEndTime,
    ClosedSessionMissingEndTime,
    EndBeforeStart,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlaySessionReferenceViolation {
    Shape(PlaySessionShapeViolation),
    CampaignMismatch {
        expected: CampaignId,
        actual: CampaignId,
    },
    MissingPlayer(PlayerId),
    MissingCharacter(CharacterId),
}

impl PlaySession {
    #[must_use]
    pub fn validate_shape(&self) -> Vec<PlaySessionShapeViolation> {
        let mut violations = Vec::new();
        let mut players = HashSet::new();
        let mut characters = HashSet::new();

        for participant in &self.participants {
            if !players.insert(participant.player_id) {
                violations.push(PlaySessionShapeViolation::DuplicatePlayer(
                    participant.player_id,
                ));
            }
            if let Some(character_id) = participant.character_id
                && !characters.insert(character_id)
            {
                violations.push(PlaySessionShapeViolation::DuplicateCharacter(character_id));
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

        if let Some(ended_at) = self.ended_at_world
            && ended_at < self.started_at_world
        {
            violations.push(PlaySessionShapeViolation::EndBeforeStart);
        }

        violations
    }

    #[must_use]
    pub fn validate_against_state(
        &self,
        state: &CampaignState,
    ) -> Vec<PlaySessionReferenceViolation> {
        let mut violations = self
            .validate_shape()
            .into_iter()
            .map(PlaySessionReferenceViolation::Shape)
            .collect::<Vec<_>>();

        if self.campaign_id != state.campaign_id() {
            violations.push(PlaySessionReferenceViolation::CampaignMismatch {
                expected: state.campaign_id(),
                actual: self.campaign_id,
            });
        }

        for participant in &self.participants {
            if !state.players.contains_key(&participant.player_id) {
                violations.push(PlaySessionReferenceViolation::MissingPlayer(
                    participant.player_id,
                ));
            }
            if let Some(character_id) = participant.character_id
                && !state.characters.contains_key(&character_id)
            {
                violations.push(PlaySessionReferenceViolation::MissingCharacter(
                    character_id,
                ));
            }
        }

        violations
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Campaign, CampaignStatus, VersionedRef, WorldClock};

    fn empty_state() -> CampaignState {
        let campaign_id = CampaignId::new();
        CampaignState::empty(
            Campaign {
                id: campaign_id,
                display_name: "Session Validation Test".into(),
                status: CampaignStatus::Active,
                world_seed: 1,
                ruleset: VersionedRef {
                    id: "test.rules".into(),
                    version: "1".into(),
                },
                content_packs: vec![],
            },
            WorldClock {
                now: WorldInstant(10),
                calendar_id: "test.calendar".into(),
            },
        )
    }

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

    #[test]
    fn one_character_cannot_be_assigned_to_two_players() {
        let character_id = CharacterId::new();
        let session = PlaySession {
            id: PlaySessionId::new(),
            campaign_id: CampaignId::new(),
            display_name: "Test Session".into(),
            status: PlaySessionStatus::Active,
            started_at_world: WorldInstant(10),
            ended_at_world: None,
            participants: vec![
                SessionParticipant {
                    player_id: PlayerId::new(),
                    character_id: Some(character_id),
                    attendance: AttendanceStatus::Present,
                },
                SessionParticipant {
                    player_id: PlayerId::new(),
                    character_id: Some(character_id),
                    attendance: AttendanceStatus::Present,
                },
            ],
        };

        assert!(session.validate_shape().iter().any(|violation| matches!(
            violation,
            PlaySessionShapeViolation::DuplicateCharacter(id) if *id == character_id
        )));
    }

    #[test]
    fn session_references_are_checked_against_world_state() {
        let state = empty_state();
        let missing_player = PlayerId::new();
        let missing_character = CharacterId::new();
        let session = PlaySession {
            id: PlaySessionId::new(),
            campaign_id: state.campaign_id(),
            display_name: "Test Session".into(),
            status: PlaySessionStatus::Active,
            started_at_world: state.clock.now,
            ended_at_world: None,
            participants: vec![SessionParticipant {
                player_id: missing_player,
                character_id: Some(missing_character),
                attendance: AttendanceStatus::Present,
            }],
        };

        let violations = session.validate_against_state(&state);
        assert!(violations.iter().any(|violation| matches!(
            violation,
            PlaySessionReferenceViolation::MissingPlayer(id) if *id == missing_player
        )));
        assert!(violations.iter().any(|violation| matches!(
            violation,
            PlaySessionReferenceViolation::MissingCharacter(id) if *id == missing_character
        )));
    }
}
