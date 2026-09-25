//! Presentation deliberately omits effect payloads, source identities and hidden targets.
use std::collections::HashSet;

use dmd_domain::*;

pub(super) fn continuation(
    resolution: &TacticalResolution,
    own: &HashSet<EntityId>,
    host: bool,
    encounter: Option<&TacticalEncounter>,
) -> Option<crate::TableTacticalContinuation> {
    if !resolution.areas.is_empty() && !host {
        // Always one generic owned invocation indicator and zero work cards,
        // independent of private target count, phase and nested consequences.
        return own
            .contains(&resolution.turn_actor)
            .then_some(crate::TableTacticalContinuation {
                actor: resolution.turn_actor,
                host_adjudication: true,
                choices: vec![],
            });
    }
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
        && !resolution
            .falls
            .iter()
            .any(|fall| fall.stage == TacticalFallStage::LandingChoice)
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
                            TacticalWorkKind::Medicine { actor, .. } => {
                                (Some(*actor), "First aid check")
                            }
                            TacticalWorkKind::SecondWind { actor, .. } => {
                                (Some(*actor), "Second Wind healing")
                            }
                            TacticalWorkKind::AreaDamageRoll { .. } => {
                                (None, "Shared area damage roll")
                            }
                            TacticalWorkKind::AreaSave { area, target } => {
                                (area_target(resolution, *area, *target), "Area saving throw")
                            }
                            TacticalWorkKind::BeginAreaDamage { .. }
                            | TacticalWorkKind::FinishArea { .. } => {
                                (None, "Area damage consequence")
                            }
                            TacticalWorkKind::ApplyAreaDamage { area, target } => (
                                area_target(resolution, *area, *target),
                                "Area damage consequence",
                            ),
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
                            TacticalWorkKind::BeginFall { fall }
                            | TacticalWorkKind::LiquidLandingCheck { fall }
                            | TacticalWorkKind::FallDamage { fall } => (
                                resolution
                                    .falls
                                    .get(usize::from(*fall))
                                    .map(|fall| fall.actor),
                                "Falling consequence",
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
                            label: if host {
                                subject
                                    .and_then(|actor| encounter?.participant(actor))
                                    .map_or_else(
                                        || label.into(),
                                        |p| format!("{label}: {}", p.public_label),
                                    )
                            } else {
                                label.into()
                            },
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
        host_adjudication: after_turn || !resolution.areas.is_empty(),
        choices,
    })
}

fn area_target(resolution: &TacticalResolution, area: u16, target: u16) -> Option<EntityId> {
    resolution
        .areas
        .iter()
        .find(|record| record.occurrence == area)
        .and_then(|record| record.targets.get(usize::from(target)))
        .map(|target| target.actor)
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
            | TacticalRollRole::AreaSave
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
            falls: vec![],
            areas: vec![],
            next_occurrence: 13,
        };
        let own = HashSet::from([own_actor]);
        let presented = continuation(&resolution, &own, false, None).unwrap();
        assert_eq!(presented.choices[0].label, "Death saving throw");
        assert_eq!(presented.choices[1].label, "Concurrent consequence");
        let json = serde_json::to_string(&presented).unwrap();
        assert!(!json.contains(&hidden_actor.0.to_string()));
        assert!(!json.contains(&hidden_group.0.to_string()));
        assert!(!json.contains("damage_taken"));
        assert!(continuation(&resolution, &HashSet::new(), false, None).is_none());
        assert_eq!(
            continuation(&resolution, &HashSet::new(), true, None)
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
            continuation(&resolution, &own, false, None)
                .unwrap()
                .choices
                .is_empty()
        );
    }

    #[test]
    fn delegated_area_cards_are_identical_for_empty_hidden_and_nested_work() {
        let actor = EntityId::new();
        let hidden = EntityId::new();
        let origin = CommandMeta {
            id: CommandId::new(),
            campaign_id: CampaignId::new(),
            session_id: None,
            issuer: CommandIssuer::Player(PlayerId::new()),
            actor: Some(AgentRef::Entity(actor)),
            expected_event_sequence: 8,
        };
        let mut resolution = TacticalResolution {
            origin: origin.clone(),
            turn_actor: actor,
            turn_number: 1,
            boundary: TurnBoundary::Start,
            frames: vec![],
            pending: None,
            failed_save: None,
            legendary_window: None,
            attack: None,
            movement: None,
            casts: vec![],
            falls: vec![],
            next_occurrence: 100,
            areas: vec![TacticalArea {
                occurrence: 0,
                source: TacticalAreaSource {
                    actor,
                    pin: CreatureSourcePin {
                        ruleset_id: "srd-5.2".into(),
                        ruleset_version: "5.2.1".into(),
                        definition_id: "chimera".into(),
                        definition_fingerprint: "projection-only".into(),
                    },
                    feature_id: "fire-breath".into(),
                    invocation: origin.clone(),
                    enclosing_origin: origin.clone(),
                },
                ordering: TacticalAreaOrdering::DelegateToHost,
                aim: TacticalAreaAim {
                    origin: SpatialPoint { x: 0, y: 0, z: 0 },
                    toward: SpatialPoint { x: 10, y: 0, z: 0 },
                    include_origin: false,
                },
                policy: TacticalAreaGridPolicy::OccupiedCellCentersV1,
                geometry_origin: origin,
                targets: vec![],
                damage: None,
                stage: TacticalAreaStage::DamageRoll,
            }],
        };
        let own = HashSet::from([actor]);
        let expected = continuation(&resolution, &own, false, None).unwrap();
        assert!(expected.host_adjudication);
        assert!(expected.choices.is_empty());
        for count in [0, 1, 2, 20] {
            resolution.frames = vec![
                (0..count)
                    .map(|occurrence| TacticalWorkItem {
                        occurrence,
                        kind: TacticalWorkKind::ConcentrationSave {
                            actor: hidden,
                            group: EffectId::new(),
                            damage_taken: 37,
                        },
                    })
                    .collect(),
            ];
            for stage in [
                TacticalAreaStage::DamageRoll,
                TacticalAreaStage::SavingThrows,
                TacticalAreaStage::ApplyingDamage,
                TacticalAreaStage::Complete,
            ] {
                resolution.areas[0].stage = stage;
                assert_eq!(
                    continuation(&resolution, &own, false, None).unwrap(),
                    expected
                );
                assert!(continuation(&resolution, &HashSet::from([hidden]), false, None).is_none());
                let host = continuation(&resolution, &HashSet::new(), true, None).unwrap();
                assert_eq!(
                    host.choices.len(),
                    if count > 1 { usize::from(count) } else { 0 }
                );
            }
        }
        assert!(
            !serde_json::to_string(&expected)
                .unwrap()
                .contains(&hidden.0.to_string())
        );
        // Clearing this invocation restores the original controller boundary.
        resolution.areas.clear();
        assert!(
            !continuation(&resolution, &own, false, None)
                .unwrap()
                .host_adjudication
        );
    }
}
