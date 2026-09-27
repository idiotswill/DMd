use super::*;

fn ready_move() -> TacticalAction {
    TacticalAction::Ready {
        trigger: ReadyTrigger::MovementFinished {
            subject: ReadySubject::AnyOther,
        },
        action: ReadyAction::Move,
    }
}

fn declarations(state: &CampaignState) -> &[TacticalReady] {
    &state
        .encounter
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .ready
}

#[test]
fn ready_abandonment_is_owned_off_turn_and_never_refunds_or_clears_other_declarations() {
    let mut f = Fixture::new();
    f.begin();
    f.run(Some(0), ready_move());
    let abandon = TacticalAction::AbandonReady { actor: f.actors[0] };
    f.rejected(Some(1), abandon.clone());
    f.rejected(None, abandon.clone()); // Host authority does not replace the player's choice.
    let timing = f.rules().timing.clone();
    let effects = f.rules().tactical_effects.clone();
    let before = f.state.clone();
    let event = f.run(Some(0), abandon.clone());
    let mut replayed = replay_tactical(&before, &event, &f.pack)
        .unwrap()
        .next_state;
    replayed.applied_event_sequence += 1;
    assert_eq!(replayed, f.state);
    assert!(declarations(&f.state).is_empty());
    assert_eq!(f.rules().timing, timing);
    assert_eq!(f.rules().tactical_effects, effects);
    f.rejected(Some(0), ready_move());
    f.rejected(Some(0), abandon.clone());
    f.run(Some(0), TacticalAction::EndTurn);
    f.run(Some(1), TacticalAction::EndTurn);
    f.run(Some(0), ready_move());
    f.run(Some(0), TacticalAction::EndTurn);
    f.run(Some(1), ready_move());
    let timing = f.rules().timing.clone();
    let clock = f.state.clock.clone();
    f.run(Some(0), abandon);
    assert_eq!(declarations(&f.state).len(), 1);
    assert_eq!(declarations(&f.state)[0].actor, f.actors[1]);
    assert_eq!(f.rules().timing, timing);
    assert_eq!(f.state.clock, clock);
}

#[test]
fn ready_abandonment_waits_for_another_actors_pending_dice() {
    let mut f = Fixture::new();
    f.begin();
    f.run(Some(0), ready_move());
    f.entity_mut(1).hp = 0;
    f.entity_mut(1).prone = true;
    f.run(Some(0), TacticalAction::EndTurn);
    assert!(f.rules().pending.is_some());
    f.rejected(Some(0), TacticalAction::AbandonReady { actor: f.actors[0] });
    assert_eq!(declarations(&f.state).len(), 1);
}

#[test]
fn ready_pays_action_only_survives_another_turn_and_expires_before_own_start() {
    let mut f = Fixture::new();
    f.begin();
    f.rejected(Some(1), ready_move());
    let event = f.run(Some(0), ready_move());
    assert!(f.rules().timing.as_ref().unwrap().action_spent);
    assert!(
        f.rules()
            .timing
            .as_ref()
            .unwrap()
            .reactions_spent
            .is_empty()
    );
    assert_eq!(declarations(&f.state)[0].origin, event.meta);
    f.rejected(Some(0), TacticalAction::Dodge);
    f.rejected(Some(0), ready_move());
    f.run(Some(0), TacticalAction::EndTurn);
    assert_eq!(declarations(&f.state).len(), 1);
    f.run(Some(1), TacticalAction::Dodge);
    assert_eq!(declarations(&f.state)[0].actor, f.actors[0]);
    f.run(Some(1), TacticalAction::EndTurn);
    assert!(declarations(&f.state).is_empty());
    assert!(!f.rules().timing.as_ref().unwrap().action_spent);
    f.run(Some(0), ready_move());
}

#[test]
fn ready_rejects_forged_controller_expired_or_unpaid_declarations_after_round_trip() {
    let mut f = Fixture::new();
    f.begin();
    f.run(Some(0), ready_move());
    for case in 0..6 {
        let mut forged = f.state.clone();
        let flow = forged.encounter.as_mut().unwrap().flow.as_mut().unwrap();
        match case {
            0 => flow.ready[0].actor = f.actors[1],
            1 => flow.ready[0].declared_on_turn = 0,
            2 => flow.ready.push(flow.ready[0].clone()),
            3 => flow.ready[0].origin.actor = Some(AgentRef::Entity(f.actors[1])),
            4 => {
                forged
                    .rules
                    .as_mut()
                    .unwrap()
                    .timing
                    .as_mut()
                    .unwrap()
                    .action_spent = false
            }
            5 => flow.phase = TacticalPhase::Finished,
            _ => unreachable!(),
        }
        let restored: CampaignState =
            serde_json::from_slice(&serde_json::to_vec(&forged).unwrap()).unwrap();
        assert!(validate_tactical_state(&restored).is_err(), "case {case}");
    }
}

#[test]
fn ready_trigger_text_is_bounded_and_named_hidden_subject_does_not_probe_position() {
    let mut f = Fixture::new();
    f.begin();
    for text in [String::new(), "a".repeat(513), "door\nopens".into()] {
        f.rejected(
            Some(0),
            TacticalAction::Ready {
                trigger: ReadyTrigger::Adjudicated { description: text },
                action: ReadyAction::Attack,
            },
        );
    }
    f.state.rules.as_mut().unwrap().effects.push(ActiveEffect {
        id: EffectId::new(),
        source: f.actors[1],
        target: f.actors[0],
        condition: Some(Condition::Blinded),
        label: "Fixture obscuration".into(),
        expires: Expiry::Never,
        concentration_owner: None,
    });
    let before = f.state.clone();
    let meta = f.meta(Some(0));
    for target in [f.actors[1], EntityId::new()] {
        let action = TacticalAction::Ready {
            trigger: ReadyTrigger::AttackFinished {
                subject: ReadySubject::Creature(target),
            },
            action: ReadyAction::Attack,
        };
        let accepted = resolve_tactical(&f.state, &meta, &action, &f.pack).unwrap();
        assert!(
            accepted
                .next_state
                .rules
                .as_ref()
                .unwrap()
                .timing
                .as_ref()
                .unwrap()
                .action_spent
        );
        assert_eq!(declarations(&accepted.next_state).len(), 1);
        assert!(
            accepted
                .next_state
                .encounter
                .as_ref()
                .unwrap()
                .flow
                .as_ref()
                .unwrap()
                .resolution
                .is_none()
        );
    }
    assert_eq!(f.state, before);
    // A generally perceptible circumstance is admitted without looking up a
    // remote actor. It still needs a real witnessed milestone before release.
    f.run(Some(0), ready_move());
}

#[test]
fn legacy_begin_replays_exactly_but_cannot_be_selected_live_and_upgrades_only_when_idle() {
    let mut f = Fixture::new();
    let action = TacticalAction::Begin {
        execution: TacticalExecutionVersion::ShieldHitV1,
        combatants: f
            .actors
            .into_iter()
            .map(|actor| TacticalCombatant {
                actor,
                source: TacticalSource::Character,
                surprised: false,
            })
            .collect(),
        groups: f
            .actors
            .into_iter()
            .map(|actor| InitiativeGroup {
                actors: vec![actor],
                request_id: RollRequestId::new(),
            })
            .collect(),
    };
    let meta = f.meta(None);
    let current = resolve_tactical(&f.state, &meta, &action, &f.pack).unwrap();
    let mut historical_event = current.event.clone();
    let TacticalAction::Begin { execution, .. } = &mut historical_event.action else {
        unreachable!()
    };
    *execution = TacticalExecutionVersion::Legacy;
    let wire = serde_json::to_value(&historical_event).unwrap();
    assert!(wire["action"]["Begin"].get("execution").is_none());
    let restored: TacticalEvent = serde_json::from_value(wire.clone()).unwrap();
    assert_eq!(serde_json::to_value(&restored).unwrap(), wire);
    assert!(resolve_tactical(&f.state, &meta, &restored.action, &f.pack).is_err());
    let legacy = replay_tactical(&f.state, &restored, &f.pack).unwrap();
    let mut expected = current.next_state.clone();
    expected
        .encounter
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .version = 1;
    assert_eq!(legacy.next_state, expected);
    f.state = legacy.next_state;
    f.state.applied_event_sequence += 1;
    f.rejected(None, TacticalAction::UpgradeExecution);
    // Actual old pending raw requests remain usable before the durable upgrade.
    f.roll(0, &[18]);
    f.roll(1, &[3]);
    f.rejected(Some(0), TacticalAction::Dodge);
    f.rejected(Some(0), TacticalAction::UpgradeExecution);
    let before = f.state.clone();
    f.run(None, TacticalAction::UpgradeExecution);
    let mut upgraded = f.state.clone();
    upgraded.applied_event_sequence = before.applied_event_sequence;
    upgraded
        .encounter
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .version = 1;
    assert_eq!(upgraded, before);
    f.rejected(None, TacticalAction::UpgradeExecution);
    assert_eq!(
        f.state
            .encounter
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .version,
        2
    );
    f.rejected(Some(0), ready_move());
    f.run(
        None,
        TacticalAction::UpgradeExecutionTo {
            execution: TacticalExecutionVersion::ShieldHitV1,
        },
    );
    f.run(Some(0), ready_move());
}
