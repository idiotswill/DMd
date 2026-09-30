//! Synthetic lifecycle invariants. Actual installed-source casting and persistence
//! are exercised separately; these fixtures do not claim a source-authorized spell.
use super::*;

fn install(f: &mut Fixture, effects: Vec<TacticalEffect>, meta: &CommandMeta) {
    f.state = tactical_effect_adapter::apply_effect_operation(
        &f.state,
        meta,
        &EffectLifecycleAction {
            step: 0,
            operation: EffectLifecycleOperation::Install { effects },
        },
    )
    .unwrap()
    .0;
    f.state.applied_event_sequence += 1;
}

fn effect(f: &Fixture, meta: &CommandMeta, ordinal: u16) -> TacticalEffect {
    TacticalEffect {
        id: EffectId::new(),
        source: EffectSource {
            definition_id: "synthetic-timing-invariant".into(),
            actor: f.actors[0],
            command: meta.clone(),
            ordinal,
        },
        established_at: None,
        target: TacticalEffectTarget::Creature(f.actors[1]),
        concentration_group: None,
        expires: TacticalEffectExpiry::Never,
        overlap: None,
        conditions: vec![],
        defenses: vec![],
        triggers: vec![],
    }
}

fn attachment(f: &Fixture) -> &TacticalEffects {
    f.rules().tactical_effects.as_ref().unwrap()
}

fn choose(f: &mut Fixture, id: EffectTicketId) {
    let occurrence = f
        .state
        .encounter
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .resolution
        .as_ref()
        .unwrap()
        .frames
        .last()
        .unwrap()
        .iter()
        .find(|work| work.kind == TacticalWorkKind::Effect { ticket: id })
        .unwrap()
        .occurrence;
    f.run(Some(0), TacticalAction::ChooseTurnWork { occurrence });
}

#[test]
fn no_due_boundary_retains_both_historical_observation_steps() {
    let mut f = Fixture::new();
    assert!(f.rules().tactical_effects.is_none());
    f.begin();
    assert_eq!(attachment(&f).last_operation.as_ref().unwrap().step, 1);
    let meta = f.meta(None);
    let mut future = effect(&f, &meta, 0);
    future.expires = TacticalEffectExpiry::AtTime(WorldInstant(60));
    install(&mut f, vec![future], &meta);
    let event = f.run(Some(0), TacticalAction::EndTurn);
    assert_eq!(
        attachment(&f).last_operation,
        Some(EffectOperationStamp {
            command: event.meta,
            step: 3,
        })
    );
    assert!(attachment(&f).pending.is_empty());
    let event = f.run(Some(1), TacticalAction::EndTurn);
    assert_eq!(f.state.clock.now, WorldInstant(6));
    assert_eq!(
        attachment(&f).last_operation,
        Some(EffectOperationStamp {
            command: event.meta,
            step: 3,
        })
    );
}

#[test]
fn no_due_source_trigger_retains_historical_time_step_in_ticket_and_raw_request() {
    let mut f = Fixture::new();
    f.begin();
    let meta = f.meta(None);
    let mut future = effect(&f, &meta, 0);
    future.expires = TacticalEffectExpiry::AtTime(WorldInstant(60));
    future.triggers.push(EffectTriggerRule {
        event: EffectTriggerEvent::Turn {
            subject: EffectSubject::Source,
            boundary: TurnBoundary::Start,
        },
        frequency: EffectTriggerFrequency::EveryOccurrence,
        payload: resistance_save(Ability::Wisdom),
    });
    install(&mut f, vec![future], &meta);
    f.run(Some(0), TacticalAction::EndTurn);
    let crossed = f.run(Some(1), TacticalAction::EndTurn);
    let ticket = &attachment(&f).pending[0];
    assert_eq!(
        ticket.id,
        EffectTicketId {
            command: crossed.meta.id,
            step: 3,
            ordinal: 0
        }
    );
    assert_eq!(
        ticket.origin,
        EffectOperationStamp {
            command: crossed.meta.clone(),
            step: 3
        }
    );
    let request = &f.rules().pending.as_ref().unwrap().request;
    assert_eq!(
        request.id,
        TacticalRollKey {
            origin: crossed.meta.id,
            role: TacticalRollRole::EffectSave,
            subject: f.actors[1],
            occurrence: 0,
        }
        .request_id()
    );
    f.roll(1, &[1]);
    assert_eq!(f.state.clock.now, WorldInstant(6));
}

#[test]
fn group_deadline_is_found_when_no_target_effect_has_a_timed_expiry() {
    let mut f = Fixture::new();
    f.begin();
    let meta = f.meta(None);
    let group = EffectId::new();
    let mut member = effect(&f, &meta, 0);
    member.concentration_group = Some(group);
    f.state = tactical_effect_adapter::apply_effect_operation(
        &f.state,
        &meta,
        &EffectLifecycleAction {
            step: 0,
            operation: EffectLifecycleOperation::BeginConcentration {
                group: ConcentrationGroup {
                    id: group,
                    source: member.source.clone(),
                    expires: TacticalEffectExpiry::AtTime(WorldInstant(6)),
                    stage: ConcentrationStage::Casting,
                },
            },
        },
    )
    .unwrap()
    .0;
    f.state = tactical_effect_adapter::apply_effect_operation(
        &f.state,
        &meta,
        &EffectLifecycleAction {
            step: 1,
            operation: EffectLifecycleOperation::Install {
                effects: vec![member],
            },
        },
    )
    .unwrap()
    .0;
    f.state.applied_event_sequence += 1;
    f.run(Some(0), TacticalAction::EndTurn);
    f.run(Some(1), TacticalAction::EndTurn);
    assert_eq!(f.state.clock.now, WorldInstant(6));
    assert_eq!(f.rules().entities[&f.actors[0]].concentration, None);
    assert!(attachment(&f).groups.is_empty());
    assert!(attachment(&f).effects.is_empty());
    assert!(attachment(&f).pending.is_empty());
}

#[test]
fn due_time_and_owner_expiry_share_turn_choice_with_suppressed_trigger_in_both_orders() {
    for expiry_first in [false, true] {
        let mut f = Fixture::new();
        f.begin();
        let meta = f.meta(None);
        let mut weak = effect(&f, &meta, 0);
        weak.overlap = Some(EffectOverlap {
            key: "synthetic-overlap".into(),
            potency: 1,
        });
        weak.triggers.push(EffectTriggerRule {
            event: EffectTriggerEvent::Turn {
                subject: EffectSubject::Source,
                boundary: TurnBoundary::Start,
            },
            frequency: EffectTriggerFrequency::EveryOccurrence,
            payload: EffectTriggerPayload::Damage {
                dice: vec![DieSpec { count: 1, sides: 6 }],
                modifier: 0,
                damage_type: DamageType::Fire,
            },
        });
        let weak_id = weak.id;
        let mut strong = effect(&f, &meta, 1);
        strong.overlap = Some(EffectOverlap {
            key: "synthetic-overlap".into(),
            potency: 2,
        });
        strong.expires = TacticalEffectExpiry::AtTime(WorldInstant(6));
        let strong_id = strong.id;
        let mut relative = effect(&f, &meta, 2);
        relative.expires = TacticalEffectExpiry::AfterOwnerBoundaries {
            owner: f.actors[0],
            boundary: TurnBoundary::Start,
            remaining: 1,
        };
        install(&mut f, vec![weak, strong, relative], &meta);
        let hp = f.rules().entities[&f.actors[1]].hp;
        f.run(Some(0), TacticalAction::EndTurn);
        let crossed = f.run(Some(1), TacticalAction::EndTurn);
        let pending = attachment(&f).pending.clone();
        assert_eq!(pending.len(), 3);
        assert_eq!(f.state.clock.now, WorldInstant(6));
        for ticket in &pending {
            assert_eq!(ticket.origin.command, crossed.meta);
            assert_eq!(ticket.id.step, 2);
            assert_eq!(ticket.origin.step, 2);
            assert_eq!(
                ticket.cause,
                EffectObservation::Turn(EffectTurn {
                    actor: f.actors[0],
                    number: 3,
                    boundary: TurnBoundary::Start,
                })
            );
        }
        let expiry = pending
            .iter()
            .find(|ticket| ticket.effect == strong_id)
            .unwrap()
            .id;
        let damage = pending
            .iter()
            .find(|ticket| ticket.effect == weak_id)
            .unwrap()
            .id;
        f.rejected(Some(0), TacticalAction::EndTurn);
        f.rejected(Some(1), TacticalAction::EndTurn);
        for observation in [EffectObservation::Time, pending[0].cause.clone()] {
            let before = f.state.clone();
            assert!(
                tactical_effect_adapter::apply_effect_operation(
                    &f.state,
                    &f.meta(None),
                    &EffectLifecycleAction {
                        step: 0,
                        operation: EffectLifecycleOperation::Observe(observation),
                    },
                )
                .is_err()
            );
            assert_eq!(f.state, before);
        }
        if expiry_first {
            choose(&mut f, expiry);
            choose(&mut f, damage);
            f.rejected(Some(0), TacticalAction::EndTurn);
            f.rejected(
                Some(1),
                TacticalAction::SubmitRoll {
                    result: f.raw(&[3]),
                },
            );
            f.roll(0, &[3]);
        } else {
            choose(&mut f, damage); // Suppressed when selected; no raw roll is issued.
            assert!(f.rules().pending.is_none());
            choose(&mut f, expiry);
        }
        assert!(attachment(&f).pending.is_empty());
        assert_eq!(attachment(&f).effects.len(), 1);
        assert_eq!(attachment(&f).effects[0].id, weak_id);
        assert_eq!(
            f.rules().entities[&f.actors[1]].hp,
            hp - if expiry_first { 3 } else { 0 }
        );
        assert_eq!(f.state.clock.now, WorldInstant(6));
    }
}

#[test]
fn suppressed_raw_deadline_expires_even_when_the_active_effect_has_no_deadline() {
    let mut f = Fixture::new();
    f.begin();
    let meta = f.meta(None);
    let mut hidden = effect(&f, &meta, 0);
    hidden.expires = TacticalEffectExpiry::AtTime(WorldInstant(6));
    hidden.overlap = Some(EffectOverlap {
        key: "synthetic-overlap".into(),
        potency: 1,
    });
    let hidden_id = hidden.id;
    let mut active = effect(&f, &meta, 1);
    active.overlap = Some(EffectOverlap {
        key: "synthetic-overlap".into(),
        potency: 2,
    });
    let active_id = active.id;
    install(&mut f, vec![hidden, active], &meta);
    assert!(!effect_applies_to(
        attachment(&f),
        &attachment(&f).effects[0],
        f.actors[1]
    ));
    f.run(Some(0), TacticalAction::EndTurn);
    f.run(Some(1), TacticalAction::EndTurn);
    assert_eq!(f.state.clock.now, WorldInstant(6));
    assert!(
        !attachment(&f)
            .effects
            .iter()
            .any(|record| record.id == hidden_id)
    );
    assert_eq!(attachment(&f).effects[0].id, active_id);
    assert!(attachment(&f).pending.is_empty());
}
