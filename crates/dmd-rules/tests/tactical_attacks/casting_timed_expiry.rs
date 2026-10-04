//! Source-built Hold Person and ordinary physical dice/turns, with no clock edits.
//! These remain pure rules fixtures; file-SQLite coverage owns product creation.
use super::*;

fn casting_fixture(target_ends_round: bool) -> (Fixture, SpellCastChoice) {
    let (mut f, choice) = prepared_source_caster("cultist-fanatic", "hold-person", vec![]);
    if !target_ends_round {
        f.begin();
        f.run(Some(0), TacticalAction::EndTurn);
        return (f, choice);
    }
    f.run(
        None,
        TacticalAction::Begin {
            execution: TacticalExecutionVersion::EncounterReleaseV1,
            combatants: vec![
                TacticalCombatant {
                    actor: f.actors[0],
                    source: TacticalSource::Character,
                    surprised: false,
                },
                TacticalCombatant {
                    actor: f.actors[1],
                    source: TacticalSource::Creature {
                        definition_id: "cultist-fanatic".into(),
                    },
                    surprised: false,
                },
            ],
            groups: f
                .actors
                .into_iter()
                .map(|actor| InitiativeGroup {
                    actors: vec![actor],
                    request_id: RollRequestId::new(),
                })
                .collect(),
        },
    );
    f.roll(0, &[3]);
    f.roll(1, &[18]);
    assert_eq!(
        f.rules().timing.as_ref().unwrap().order[0].actor,
        f.actors[1]
    );
    (f, choice)
}

#[test]
fn real_hold_deadline_continues_from_end_command_or_final_owned_raw_save() {
    for target_ends_round in [false, true] {
        let (mut f, choice) = casting_fixture(target_ends_round);
        f.run(
            Some(1),
            TacticalAction::CastSpell {
                choice: choice.clone(),
                targets: SpellTargetChoice::Entities(vec![f.actors[0]]),
            },
        );
        f.roll(0, &[1]);
        let group = f.rules().entities[&f.actors[1]].concentration.unwrap();
        let group_record = f.rules().tactical_effects.as_ref().unwrap().groups[0].clone();
        assert_eq!(
            group_record.expires,
            TacticalEffectExpiry::AtTime(WorldInstant(60))
        );
        let paid = f
            .rules()
            .tactical_creatures
            .as_ref()
            .unwrap()
            .runtime
            .clone();
        let original_rolls = f.rules().rolls.clone();
        // Invalid retained input is rejected before the adapter can erase its group.
        let genuine = f.state.clone();
        f.entity_mut(1).hp = 0;
        f.entity_mut(1).prone = true;
        assert!(validate_state(&f.state, &f.pack).is_err());
        f.rejected(Some(1), TacticalAction::EndTurn);
        f.state = genuine;

        let mut repeated = 0;
        let mut end_commands = 0;
        let crossing = loop {
            end_commands += 1;
            assert!(
                end_commands <= 20,
                "ten two-actor rounds must reach the deadline"
            );
            let timing = f.rules().timing.as_ref().unwrap();
            let actor = timing.order[timing.index].actor;
            let index = usize::from(actor == f.actors[1]);
            let before_time = f.state.clock.now;
            let ended = f.run(Some(index), TacticalAction::EndTurn);
            let completed = if f.rules().pending.is_some() {
                let pending = f.rules().pending.as_ref().unwrap();
                assert_eq!(pending.request.roller, Some(f.actors[0]));
                assert!(matches!(
                    pending.purpose,
                    PendingPurpose::TacticalResolution {
                        key: TacticalRollKey {
                            role: TacticalRollRole::EffectSave,
                            ..
                        },
                        ..
                    }
                ));
                f.rejected(
                    Some(1),
                    TacticalAction::SubmitRoll {
                        result: f.raw(&[1]),
                    },
                );
                f.rejected(Some(index), TacticalAction::EndTurn);
                repeated += 1;
                f.run(
                    Some(0),
                    TacticalAction::SubmitRoll {
                        result: f.raw(&[1]),
                    },
                )
            } else {
                ended
            };
            if f.state.clock.now == WorldInstant(60) {
                assert_eq!(before_time, WorldInstant(54));
                break completed;
            }
            assert!(f.state.clock.now < WorldInstant(60));
            assert_eq!(f.rules().entities[&f.actors[1]].concentration, Some(group));
            assert!(active_conditions(f.rules(), f.actors[0]).contains(&Condition::Paralyzed));
        };
        assert_eq!(repeated, if target_ends_round { 10 } else { 9 });
        assert_eq!(
            matches!(crossing.action, TacticalAction::SubmitRoll { .. }),
            target_ends_round
        );
        let effects = f.rules().tactical_effects.as_ref().unwrap();
        assert_eq!(effects.pending.len(), 2);
        assert!(
            effects
                .pending
                .iter()
                .all(|ticket| ticket.origin.command == crossing.meta)
        );
        assert!(effects.pending.iter().all(|ticket| matches!(
            ticket.cause,
            EffectObservation::Turn(EffectTurn {
                boundary: TurnBoundary::Start,
                ..
            })
        )));
        let ticket = effects
            .pending
            .iter()
            .find(|ticket| ticket.payload == EffectTriggerPayload::ExpireConcentrationGroup)
            .unwrap()
            .id;
        let occurrence = f
            .flow()
            .resolution
            .as_ref()
            .unwrap()
            .frames
            .last()
            .unwrap()
            .iter()
            .find(|work| work.kind == TacticalWorkKind::Effect { ticket })
            .unwrap()
            .occurrence;
        let owner = usize::from(
            f.flow()
                .resolution
                .as_ref()
                .unwrap()
                .turn_context()
                .expect("turn fixture")
                .actor
                == f.actors[1],
        );
        f.rejected(
            Some(1 - owner),
            TacticalAction::ChooseTurnWork { occurrence },
        );
        f.run(Some(owner), TacticalAction::ChooseTurnWork { occurrence });
        assert!(f.flow().resolution.is_none());
        assert!(
            f.rules()
                .tactical_effects
                .as_ref()
                .unwrap()
                .pending
                .is_empty()
        );
        assert!(
            f.rules()
                .tactical_effects
                .as_ref()
                .unwrap()
                .groups
                .is_empty()
        );
        assert!(
            f.rules()
                .tactical_effects
                .as_ref()
                .unwrap()
                .effects
                .is_empty()
        );
        assert_eq!(f.rules().entities[&f.actors[1]].concentration, None);
        assert!(!active_conditions(f.rules(), f.actors[0]).contains(&Condition::Paralyzed));
        assert_eq!(
            &f.rules().rolls[..original_rolls.len()],
            original_rolls.as_slice()
        );
        // Feature use remains spent; only turn hooks may change unrelated runtime cursors.
        let spent = &paid[0].limited_uses;
        assert_eq!(
            &f.rules().tactical_creatures.as_ref().unwrap().runtime[0].limited_uses,
            spent
        );
        f.rejected(
            Some(1),
            TacticalAction::CastSpell {
                choice,
                targets: SpellTargetChoice::Entities(vec![f.actors[0]]),
            },
        );
        assert_eq!(f.state.clock.now, WorldInstant(60));
    }
}
