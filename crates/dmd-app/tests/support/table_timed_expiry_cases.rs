//! Genuine installed-source creation, physical saves, natural rounds and file SQLite.
//! No fabricated mechanics, elapsed-time command or database mutation sets the deadline.
use super::*;

fn preserve_prefix(
    before: &dmd_persistence::CampaignExport,
    after: &dmd_persistence::CampaignExport,
) {
    assert_eq!(
        &after.event_journal[..before.event_journal.len()],
        before.event_journal.as_slice()
    );
    assert_eq!(
        &after.command_audit[..before.command_audit.len()],
        before.command_audit.as_slice()
    );
    assert_eq!(
        &after.table_projection_history[..before.table_projection_history.len()],
        before.table_projection_history.as_slice()
    );
    // Bindings are exported by command UUID, not acceptance sequence. A new
    // accepted UUID can sort before an old row without changing that old receipt.
    for binding in &before.table_transport_bindings {
        assert_eq!(
            after
                .table_transport_bindings
                .iter()
                .find(|candidate| candidate.meta.id == binding.meta.id),
            Some(binding)
        );
    }
}

#[tokio::test]
async fn source_hold_person_deadline_survives_natural_rounds_cold_choice_retry_and_restore() {
    let directory = std::env::temp_dir().join(format!("dmd-hold-deadline-{}", CampaignId::new().0));
    std::fs::create_dir_all(&directory).unwrap();
    let path = directory.join("campaign.sqlite");
    let pool = dmd_persistence::open_sqlite_path(&path).await.unwrap();
    let mut f = Box::pin(Fixture::with_pool(TableContract::default(), pool)).await;
    let sources = Box::pin(prepare_sources(&mut f, true, true)).await;
    for index in [0, 1, 2] {
        Box::pin(accept(
            &f,
            sources.channel(index),
            tactical(TacticalAction::EndTurn),
        ))
        .await;
    }
    let choice = view(&f, &sources.channel(3))
        .await
        .tactical
        .unwrap()
        .casting_options
        .unwrap()
        .variants
        .into_iter()
        .find(|variant| variant.choice.spell_id == "hold-person")
        .unwrap()
        .choice;
    let cast = request(
        &f,
        sources.channel(3),
        tactical(TacticalAction::CastSpell {
            choice,
            targets: SpellTargetChoice::Entities(vec![sources.actors[2]]),
        }),
    )
    .await;
    Box::pin(cold_step(&mut f, &path, "hold-initial-save", cast)).await;
    let initial = view(&f, &sources.channel(2)).await;
    assert_eq!(initial.roll.as_ref().unwrap().modifier, 4); // Installed Mage Wisdom save.
    let failed = raw(initial, 1);
    Box::pin(accept(&f, sources.channel(2), failed)).await;
    let paid_state = Box::pin(state(&f)).await;
    let paid_rules = paid_state.rules.as_ref().unwrap();
    let group = paid_rules.entities[&sources.actors[3]]
        .concentration
        .unwrap();
    assert_eq!(paid_state.clock.now, WorldInstant(0));
    assert_eq!(
        paid_rules.tactical_effects.as_ref().unwrap().groups[0].expires,
        TacticalEffectExpiry::AtTime(WorldInstant(60))
    );
    let paid = Box::new(export_campaign(&f.pool, f.campaign).await.unwrap());
    let mut repeated = 0;
    let mut end_commands = 0;
    let crossed = loop {
        end_commands += 1;
        assert!(
            end_commands <= 40,
            "ten four-actor rounds must reach the deadline"
        );
        let current = Box::pin(state(&f)).await;
        let timing = current.rules.as_ref().unwrap().timing.as_ref().unwrap();
        let active = timing.order[timing.index].actor;
        let index = sources
            .actors
            .iter()
            .position(|actor| *actor == active)
            .unwrap();
        let at_deadline = current.clock.now == WorldInstant(54) && index == 3;
        let end = request(
            &f,
            sources.channel(index),
            tactical(TacticalAction::EndTurn),
        )
        .await;
        if at_deadline {
            Box::pin(cold_step(
                &mut f,
                &path,
                "hold-deadline-boundary",
                end.clone(),
            ))
            .await;
            break end;
        }
        assert!(matches!(
            Box::pin(f.runtime.submit_presented_table(end))
                .await
                .unwrap(),
            TableTransportResult::Accepted(_)
        ));
        if let Some(roll) = view(&f, &sources.channel(2)).await.roll {
            assert_eq!(roll.modifier, 4);
            assert_eq!(index, 2);
            assert!(view(&f, &sources.channel(0)).await.roll.is_none());
            let input = raw(view(&f, &sources.channel(2)).await, 1);
            if repeated == 0 {
                let foreign = request(&f, sources.channel(0), input.clone()).await;
                Box::pin(unchanged(&f, foreign)).await;
                let wrong_actor = request(&f, sources.channel(3), input.clone()).await;
                Box::pin(unchanged(&f, wrong_actor)).await;
                let pending_end =
                    request(&f, sources.channel(2), tactical(TacticalAction::EndTurn)).await;
                Box::pin(unchanged(&f, pending_end)).await;
            }
            let save = request(&f, sources.channel(2), input).await;
            if current.clock.now == WorldInstant(54) {
                Box::pin(cold_step(&mut f, &path, "hold-last-repeat-save", save)).await;
            } else {
                assert!(matches!(
                    Box::pin(f.runtime.submit_presented_table(save))
                        .await
                        .unwrap(),
                    TableTransportResult::Accepted(_)
                ));
            }
            repeated += 1;
        }
        let continued = Box::pin(state(&f)).await;
        assert!(continued.clock.now < WorldInstant(60));
        assert_eq!(
            continued.rules.as_ref().unwrap().entities[&sources.actors[3]].concentration,
            Some(group)
        );
        assert!(
            dmd_rules::active_conditions(continued.rules.as_ref().unwrap(), sources.actors[2])
                .contains(&Condition::Paralyzed)
        );
    };
    assert_eq!(repeated, 9);
    let due = Box::pin(state(&f)).await;
    assert_eq!(due.clock.now, WorldInstant(60));
    assert_eq!(
        due.rules.as_ref().unwrap().timing.as_ref().unwrap().round,
        11
    );
    assert_eq!(
        due.rules
            .as_ref()
            .unwrap()
            .tactical_effects
            .as_ref()
            .unwrap()
            .pending
            .len(),
        2
    );
    assert!(due.rules.as_ref().unwrap().pending.is_none());
    let snapshot = Box::new(export_campaign(&f.pool, f.campaign).await.unwrap());
    preserve_prefix(&paid, &snapshot);
    let presented = view(&f, &sources.channel(0))
        .await
        .tactical
        .unwrap()
        .continuation
        .unwrap();
    assert_eq!(presented.actor, sources.actors[0]);
    assert_eq!(presented.choices.len(), 2);
    assert!(
        view(&f, &sources.channel(2))
            .await
            .tactical
            .unwrap()
            .continuation
            .is_none_or(|continuation| continuation.choices.is_empty())
    );
    let selected = TableTransportInput::SelectWork {
        handle: presented.choices[0].handle,
    };
    let foreign = request(&f, sources.channel(2), selected.clone()).await;
    Box::pin(unchanged(&f, foreign)).await;
    let wrong_actor = request(&f, sources.channel(1), selected.clone()).await;
    Box::pin(unchanged(&f, wrong_actor)).await;
    let mut stale = crossed;
    stale.command_id = CommandId::new();
    Box::pin(unchanged(&f, stale)).await;
    let selection = request(&f, sources.channel(0), selected).await;
    Box::pin(cold_step(&mut f, &path, "hold-choose-expiry", selection)).await;

    let completed = Box::pin(state(&f)).await;
    let final_rules = completed.rules.as_ref().unwrap();
    assert_eq!(completed.clock.now, WorldInstant(60));
    assert_eq!(final_rules.entities[&sources.actors[3]].concentration, None);
    assert!(
        !dmd_rules::active_conditions(final_rules, sources.actors[2])
            .contains(&Condition::Paralyzed)
    );
    assert!(
        final_rules
            .tactical_effects
            .as_ref()
            .unwrap()
            .groups
            .is_empty()
    );
    assert!(
        final_rules
            .tactical_effects
            .as_ref()
            .unwrap()
            .effects
            .is_empty()
    );
    assert!(
        final_rules
            .tactical_effects
            .as_ref()
            .unwrap()
            .pending
            .is_empty()
    );
    assert!(
        completed
            .encounter
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .resolution
            .is_none()
    );
    assert_eq!(
        &final_rules.rolls[..paid_rules.rolls.len()],
        paid_rules.rolls.as_slice()
    );
    assert_eq!(final_rules.rolls.len(), paid_rules.rolls.len() + 9);
    for actor in sources.actors {
        assert_eq!(
            final_rules.entities[&actor].hp,
            paid_rules.entities[&actor].hp
        );
        assert_eq!(
            final_rules.entities[&actor].resources,
            paid_rules.entities[&actor].resources
        );
        assert_eq!(
            final_rules
                .tactical_creatures
                .as_ref()
                .unwrap()
                .runtime(actor)
                .unwrap()
                .limited_uses,
            paid_rules
                .tactical_creatures
                .as_ref()
                .unwrap()
                .runtime(actor)
                .unwrap()
                .limited_uses
        );
    }
    let finished = Box::new(export_campaign(&f.pool, f.campaign).await.unwrap());
    preserve_prefix(&snapshot, &finished);
    // cold_step already restored the completed export independently and retried
    // the exact accepted selection without changing any row or prior roll.
    Box::pin(reopen(&mut f, &path)).await;
    assert_eq!(*Box::pin(state(&f)).await, *completed);
    f.pool.close().await;
    drop(f);
    sqlite_test_cleanup::remove_closed_directory(&directory)
        .await
        .unwrap();
}
