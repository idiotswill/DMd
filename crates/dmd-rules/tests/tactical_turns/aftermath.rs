use super::*;

fn conclude() -> TacticalAction {
    TacticalAction::ConcludeHostilities {
        cadence: AftermathCadence::ContinueExistingOrder,
        ruling: "Keep the existing cadence while ongoing effects settle.".into(),
    }
}

#[test]
fn conclusion_preserves_paid_ready_and_every_existing_gameplay_field() {
    let mut f = Fixture::new();
    f.begin();
    f.run(
        Some(0),
        TacticalAction::Ready {
            trigger: ReadyTrigger::MovementFinished {
                subject: ReadySubject::AnyOther,
            },
            action: ReadyAction::Move,
        },
    );
    let before = f.state.clone();
    f.rejected(Some(0), conclude());
    f.rejected(Some(1), conclude());
    let accepted = f.run(None, conclude());
    let record = f
        .state
        .encounter
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .aftermath
        .as_ref()
        .unwrap();
    assert_eq!(record.origin, accepted.meta);
    assert_eq!(record.concluded_at, before.clock.now);
    assert_eq!(record.concluded_on_turn.actor, f.actors[0]);
    let mut stripped = f.state.clone();
    stripped
        .encounter
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .aftermath = None;
    stripped.applied_event_sequence = before.applied_event_sequence;
    assert_eq!(
        stripped, before,
        "conclusion cannot spend/refund, expire, heal or advance"
    );
    require_aftermath_session_boundary(&f.state).unwrap();
    f.rejected(None, conclude());
    f.run(Some(0), TacticalAction::EndTurn);
    assert_eq!(
        f.state
            .encounter
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .ready
            .len(),
        1
    );
    f.run(Some(1), TacticalAction::EndTurn);
    assert!(
        f.state
            .encounter
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .ready
            .is_empty()
    );
    assert_eq!(f.state.clock.now, WorldInstant(6));
    require_aftermath_session_boundary(&f.state).unwrap();
}

#[test]
fn conclusion_and_session_pause_reject_pending_death_save_without_choosing_it() {
    let mut f = Fixture::new();
    f.begin();
    assert!(require_aftermath_session_boundary(&f.state).is_err());
    // Isolated rules fixture; the table scenario creates the dying PC through
    // an actual source weapon attack instead of changing HP.
    f.entity_mut(1).hp = 0;
    f.entity_mut(1).prone = true;
    f.run(None, conclude());
    f.run(Some(0), TacticalAction::EndTurn);
    assert!(f.rules().pending.is_some());
    f.rejected(None, conclude());
    assert!(require_aftermath_session_boundary(&f.state).is_err());
    f.roll(1, &[10]);
    assert_eq!(f.rules().entities[&f.actors[1]].death.successes, 1);
    assert_eq!(f.rules().entities[&f.actors[1]].hp, 0);
    require_aftermath_session_boundary(&f.state).unwrap();
}

#[test]
fn conclusion_requires_explicit_valid_policy_and_authentic_bounded_origin() {
    let mut f = Fixture::new();
    f.begin();
    for ruling in [String::new(), " trailing ".into(), "x".repeat(2_001)] {
        f.rejected(
            None,
            TacticalAction::ConcludeHostilities {
                cadence: AftermathCadence::ContinueExistingOrder,
                ruling,
            },
        );
    }
    assert!(
        serde_json::from_value::<TacticalAction>(serde_json::json!({
            "ConcludeHostilities": {"ruling":"No implicit policy"}
        }))
        .is_err()
    );
    f.run(None, conclude());
    for case in 0..6 {
        let mut corrupted = f.state.clone();
        let flow = corrupted.encounter.as_mut().unwrap().flow.as_mut().unwrap();
        let record = flow.aftermath.as_mut().unwrap();
        match case {
            0 => record.origin.issuer = CommandIssuer::Player(f.players[0]),
            1 => record.concluded_at = WorldInstant(1),
            2 => record.concluded_on_turn.number = 0,
            3 => record.concluded_on_turn.actor = EntityId::new(),
            4 => record.concluded_on_turn.boundary = TurnBoundary::End,
            5 => flow.phase = TacticalPhase::Finished,
            _ => unreachable!(),
        }
        let decoded: CampaignState =
            serde_json::from_slice(&serde_json::to_vec(&corrupted).unwrap()).unwrap();
        assert!(validate_tactical_state(&decoded).is_err(), "case {case}");
    }
}

#[test]
fn absent_conclusion_keeps_historical_flow_bytes_unchanged() {
    let mut f = Fixture::new();
    f.begin();
    let flow = f.state.encounter.as_ref().unwrap().flow.as_ref().unwrap();
    let value = serde_json::to_value(flow).unwrap();
    assert!(value.get("aftermath").is_none());
    let restored: TacticalFlow = serde_json::from_value(value.clone()).unwrap();
    assert!(restored.aftermath.is_none());
    assert_eq!(serde_json::to_value(restored).unwrap(), value);
}

#[test]
fn historical_reactions_aftermath_replays_and_upgrades_without_changing_its_cadence() {
    let mut f = Fixture::new();
    f.begin();
    // Isolated typed rules fixture for the historical admission boundary. This
    // is not a genuine persisted capture or a replacement for application replay.
    f.state
        .encounter
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .version = TacticalExecutionVersion::ReactionsV1.flow_version();
    f.rejected(None, conclude());
    let event = TacticalEvent {
        meta: f.meta(None),
        action: conclude(),
        outcome: TacticalOutcome {
            next_roll: None,
            awaiting_initiative_ties: false,
            active_actor: Some(f.actors[0]),
            awaiting_turn_work: false,
        },
    };
    let historical = replay_tactical(&f.state, &event, &f.pack).unwrap();
    assert_eq!(historical.event, event);
    f.state = historical.next_state;
    f.state.applied_event_sequence += 1;
    require_aftermath_session_boundary(&f.state).unwrap();
    let before = f.state.clone();
    f.run(
        None,
        TacticalAction::UpgradeExecutionTo {
            execution: TacticalExecutionVersion::ShieldHitV1,
        },
    );
    require_aftermath_session_boundary(&f.state).unwrap();
    let mut compared = f.state.clone();
    let version = &mut compared
        .encounter
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .version;
    assert_eq!(
        *version,
        TacticalExecutionVersion::ShieldHitV1.flow_version()
    );
    *version = TacticalExecutionVersion::ReactionsV1.flow_version();
    compared.applied_event_sequence = before.applied_event_sequence;
    assert_eq!(
        compared, before,
        "upgrade preserves the complete aftermath state"
    );
    f.run(Some(0), TacticalAction::EndTurn);
    require_aftermath_session_boundary(&f.state).unwrap();
}
