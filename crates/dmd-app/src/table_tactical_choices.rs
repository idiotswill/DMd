//! Presentation deliberately omits effect payloads, source identities and hidden targets.
use std::collections::HashSet;

use dmd_domain::*;

pub(super) fn continuation(
    resolution: &TacticalResolution,
    own: &HashSet<EntityId>,
    host: bool,
) -> Option<crate::TableTacticalContinuation> {
    let after_turn = resolution.frames.last().is_some_and(|frame| {
        !frame.is_empty()
            && frame
                .iter()
                .all(|work| matches!(work.kind, TacticalWorkKind::LegendaryWindow { .. }))
    });
    if !host && (after_turn || !own.contains(&resolution.turn_actor)) {
        return None;
    }
    let choices = if resolution.pending.is_none()
        && resolution.failed_save.is_none()
        && resolution.legendary_window.is_none()
    {
        resolution
            .frames
            .last()
            .filter(|frame| frame.len() > 1)
            .map(|frame| {
                frame
                    .iter()
                    .map(|work| {
                        let (subject, kind) = match &work.kind {
                            TacticalWorkKind::DeathSave { actor } => {
                                (Some(*actor), "Death saving throw")
                            }
                            TacticalWorkKind::ConcentrationSave { actor, .. } => {
                                (Some(*actor), "Concentration saving throw")
                            }
                            TacticalWorkKind::StableRecovery { actor, .. } => {
                                (Some(*actor), "Stable recovery time")
                            }
                            TacticalWorkKind::RecoverStable { actor } => {
                                (Some(*actor), "Stable recovery")
                            }
                            TacticalWorkKind::Effect { .. } => (None, "Effect consequence"),
                            TacticalWorkKind::CreatureRecharge { actor, .. } => {
                                (Some(*actor), "Ability recharge")
                            }
                            TacticalWorkKind::LegendaryWindow { actor } => {
                                (Some(*actor), "Legendary Action opportunity")
                            }
                            TacticalWorkKind::AttackRoll
                            | TacticalWorkKind::AttackDamage
                            | TacticalWorkKind::FinishAttack => (
                                resolution.attack.as_ref().map(|attack| attack.actor),
                                "Attack consequence",
                            ),
                            TacticalWorkKind::MoveSegment => (
                                resolution.movement.as_ref().map(|movement| movement.actor),
                                "Continue movement",
                            ),
                            TacticalWorkKind::MovementOpportunity { reactor } => {
                                (Some(*reactor), "Opportunity attack")
                            }
                            TacticalWorkKind::SpellProgram { cast, .. }
                            | TacticalWorkKind::FinishSpell { cast } => (
                                resolution
                                    .casts
                                    .iter()
                                    .find(|record| record.cast.plan.occurrence == *cast)
                                    .map(|record| record.cast.plan.choice.actor),
                                "Spell consequence",
                            ),
                            TacticalWorkKind::EndOccupiedSpace { actor } => {
                                (Some(*actor), "Resolve occupied space")
                            }
                        };
                        // Owning the turn grants ordering authority, not knowledge of another
                        // actor's health, concentration, hidden source, DC or location.
                        let label = if host || subject.is_some_and(|actor| own.contains(&actor)) {
                            kind
                        } else {
                            "Concurrent consequence"
                        };
                        crate::TableTacticalWorkChoice {
                            occurrence: work.occurrence,
                            label: label.into(),
                        }
                    })
                    .collect()
            })
            .unwrap_or_default()
    } else {
        vec![]
    };
    Some(crate::TableTacticalContinuation {
        actor: resolution.turn_actor,
        host_adjudication: after_turn,
        choices,
    })
}

pub(super) fn save_actor(
    pending: &PendingRoll,
    own: &HashSet<EntityId>,
    host: bool,
) -> Option<EntityId> {
    let PendingPurpose::TacticalResolution { key, .. } = &pending.purpose else {
        return None;
    };
    if !matches!(
        key.role,
        TacticalRollRole::DeathSave
            | TacticalRollRole::EffectSave
            | TacticalRollRole::Concentration
            | TacticalRollRole::SpellSave
    ) {
        return None;
    }
    pending
        .request
        .roller
        .filter(|actor| host || own.contains(actor))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_pending_savers_channel_receives_the_failure_choice() {
        let actor = EntityId::new();
        let key = TacticalRollKey {
            origin: CommandId::new(),
            role: TacticalRollRole::Concentration,
            subject: actor,
            occurrence: 6,
        };
        let mut pending = PendingRoll {
            issued_by: CommandMeta {
                id: CommandId::new(),
                campaign_id: CampaignId::new(),
                session_id: None,
                issuer: CommandIssuer::Admin,
                actor: None,
                expected_event_sequence: 7,
            },
            request: RollRequest {
                id: key.request_id(),
                roller: Some(actor),
                dice: vec![DieSpec {
                    count: 1,
                    sides: 20,
                }],
                modifier: 3,
                mode: RollMode::Normal,
                visibility: RollVisibility::Secret,
                reason: "Private source".into(),
            },
            purpose: PendingPurpose::TacticalResolution {
                encounter: EncounterId::new(),
                key,
            },
            ruling: Ruling {
                basis: RulingBasis::Srd { page: 180 },
                reason: "Concentration".into(),
            },
        };
        let own = HashSet::from([actor]);
        assert_eq!(save_actor(&pending, &own, false), Some(actor));
        assert_eq!(save_actor(&pending, &HashSet::new(), false), None);
        assert_eq!(save_actor(&pending, &HashSet::new(), true), Some(actor));
        pending.purpose = PendingPurpose::TacticalResolution {
            encounter: EncounterId::new(),
            key: TacticalRollKey {
                role: TacticalRollRole::EffectDamage,
                ..key
            },
        };
        assert_eq!(save_actor(&pending, &own, false), None);
        assert_eq!(save_actor(&pending, &HashSet::new(), true), None);
        // Amount keys name the target, while the real roller is the caster. They
        // never enable a voluntary save-failure choice for either participant.
        let caster = EntityId::new();
        pending.request.roller = Some(caster);
        pending.purpose = PendingPurpose::TacticalResolution {
            encounter: EncounterId::new(),
            key: TacticalRollKey {
                role: TacticalRollRole::SpellAmount,
                ..key
            },
        };
        assert_eq!(save_actor(&pending, &HashSet::from([caster]), false), None);
        assert_eq!(save_actor(&pending, &own, false), None);
        pending.request.roller = Some(actor);
        pending.purpose = PendingPurpose::TacticalResolution {
            encounter: EncounterId::new(),
            key: TacticalRollKey {
                role: TacticalRollRole::SpellSave,
                ..key
            },
        };
        assert_eq!(save_actor(&pending, &own, false), Some(actor));
        assert_eq!(save_actor(&pending, &HashSet::from([caster]), false), None);
    }

    #[test]
    fn ordering_authority_does_not_reveal_other_actors_consequences() {
        let own_actor = EntityId::new();
        let hidden_actor = EntityId::new();
        let hidden_group = EffectId::new();
        let origin = CommandMeta {
            id: CommandId::new(),
            campaign_id: CampaignId::new(),
            session_id: None,
            issuer: CommandIssuer::Admin,
            actor: None,
            expected_event_sequence: 8,
        };
        let mut resolution = TacticalResolution {
            origin,
            turn_actor: own_actor,
            turn_number: 4,
            boundary: TurnBoundary::Start,
            frames: vec![vec![
                TacticalWorkItem {
                    occurrence: 11,
                    kind: TacticalWorkKind::DeathSave { actor: own_actor },
                },
                TacticalWorkItem {
                    occurrence: 12,
                    kind: TacticalWorkKind::ConcentrationSave {
                        actor: hidden_actor,
                        group: hidden_group,
                        damage_taken: 37,
                    },
                },
            ]],
            pending: None,
            failed_save: None,
            legendary_window: None,
            attack: None,
            movement: None,
            casts: vec![],
            next_occurrence: 13,
        };
        let own = HashSet::from([own_actor]);
        let presented = continuation(&resolution, &own, false).unwrap();
        assert_eq!(presented.choices[0].label, "Death saving throw");
        assert_eq!(presented.choices[1].label, "Concurrent consequence");
        let json = serde_json::to_string(&presented).unwrap();
        assert!(!json.contains(&hidden_actor.0.to_string()));
        assert!(!json.contains(&hidden_group.0.to_string()));
        assert!(!json.contains("damage_taken"));
        assert!(continuation(&resolution, &HashSet::new(), false).is_none());
        assert_eq!(
            continuation(&resolution, &HashSet::new(), true)
                .unwrap()
                .choices[1]
                .label,
            "Concentration saving throw"
        );
        resolution.pending = Some(TacticalPendingWork {
            work: resolution.frames[0][0].clone(),
            key: TacticalRollKey {
                origin: resolution.origin.id,
                role: TacticalRollRole::DeathSave,
                subject: own_actor,
                occurrence: 11,
            },
        });
        assert!(
            continuation(&resolution, &own, false)
                .unwrap()
                .choices
                .is_empty()
        );
    }
}
