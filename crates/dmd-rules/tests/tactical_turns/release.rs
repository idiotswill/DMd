//! Isolated domain/rule fixtures. These are not genuine app captures or release
//! acceptance; the application scenarios exercise the complete durable route.
use super::*;

#[test]
fn accepted_release_preserves_source_resources_and_replacement_advances_global_turn() {
    let mut f = prepared_creature();
    f.rejected(Some(0), TacticalAction::FinishEncounter);
    let before = f.state.clone();
    let event = f.run(None, TacticalAction::FinishEncounter);
    let history = f.state.encounter_history.as_ref().unwrap();
    assert_eq!(history.completions.len(), 1);
    let receipt = history.last().unwrap().clone();
    assert_eq!(receipt.released_by, event.meta);
    assert_eq!(
        receipt.final_turn.number,
        before
            .rules
            .as_ref()
            .unwrap()
            .timing
            .as_ref()
            .unwrap()
            .turn_number
    );
    assert_eq!(f.state.clock, before.clock);
    assert_eq!(f.state.items, before.items);
    assert_eq!(
        f.state.rules.as_ref().unwrap().entities,
        before.rules.as_ref().unwrap().entities
    );
    assert_eq!(
        f.state.rules.as_ref().unwrap().tactical_inventory,
        before.rules.as_ref().unwrap().tactical_inventory
    );
    let mut source = f.rules().tactical_creatures.clone().unwrap();
    let previous = before
        .rules
        .as_ref()
        .unwrap()
        .tactical_creatures
        .as_ref()
        .unwrap();
    for (runtime, old) in source.runtime.iter_mut().zip(&previous.runtime) {
        assert!(runtime.observed_turn.is_none());
        assert!(!runtime.legendary_window_spent);
        assert_eq!(runtime.last_operation, event.meta);
        runtime.observed_turn = old.observed_turn;
        runtime.legendary_window_spent = old.legendary_window_spent;
        runtime.last_operation = old.last_operation.clone();
    }
    assert_eq!(
        &source, previous,
        "only retired cursors and their provenance change"
    );
    f.rejected(None, TacticalAction::FinishEncounter);
    let old_encounter = f.state.encounter.as_ref().unwrap().clone();
    let mut scene = f.state.scenes[&old_encounter.scene_id].clone();
    scene.id = SceneId::new();
    let mut location = f.state.locations[&scene.location_id].clone();
    location.id = LocationId::new();
    scene.location_id = location.id;
    f.state.locations.insert(location.id, location);
    // Application setup stages a fresh Closed scene in the same atomic command.
    let scene_id = scene.id;
    f.state.scenes.insert(scene_id, scene);
    let mut authored = old_encounter.clone();
    authored.id = EncounterId::new();
    authored.scene_id = scene_id;
    authored.flow = None;
    f.run(
        None,
        TacticalAction::Establish {
            encounter: Box::new(authored),
        },
    );
    assert_eq!(
        f.state.scenes[&old_encounter.scene_id].status,
        SceneStatus::Closed
    );
    assert_eq!(f.state.scenes[&scene_id].status, SceneStatus::Active);
    assert!(
        f.actors
            .iter()
            .all(|actor| f.state.entities[actor].location_id
                == Some(f.state.scenes[&scene_id].location_id))
    );
    assert_eq!(
        f.state.encounter_history.as_ref().unwrap().last(),
        Some(&receipt)
    );
    f.begin();
    assert_eq!(
        f.rules().timing.as_ref().unwrap().turn_number,
        receipt.final_turn.number + 1
    );
    assert_eq!(f.rules().timing.as_ref().unwrap().round, 1);
    f.run(Some(0), TacticalAction::StartAttackAction);
    f.rejected(None, TacticalAction::FinishEncounter);
}

fn conclude(f: &mut Fixture) {
    f.run(
        None,
        TacticalAction::ConcludeHostilities {
            cadence: AftermathCadence::ContinueExistingOrder,
            ruling: "Fighting has stopped; retain all actual consequences.".into(),
        },
    );
}

fn features() -> CharacterFeatureState {
    CharacterFeatureState {
        fighter_level: 1,
        fighting_style: FightingStyle::Archery,
        wearing_armor: false,
        human_resourceful: true,
        savage_attacker: true,
        second_wind_remaining: 1,
        inspiration_transfer_pending: false,
        savage_attacker_turn: None,
    }
}

fn prepared_creature() -> Fixture {
    let mut f = Fixture::new();
    f.creature(1, false);
    let origin = f.meta(None);
    f.state.rules.as_mut().unwrap().tactical_inventory = Some(TacticalInventory {
        schema_version: 1,
        receipts: vec![],
        loadouts: vec![ActorEquipmentLoadout {
            actor: f.actors[1],
            hands: WeaponLoadout::default(),
            worn_armor: None,
            shield: None,
            command: origin,
        }],
    });
    f.begin();
    conclude(&mut f);
    f
}

fn install_defense(f: &mut Fixture, ordinal: u16, expires: TacticalEffectExpiry) {
    let origin = f.meta(None);
    f.state = tactical_effect_adapter::apply_effect_operation(
        &f.state,
        &origin,
        &EffectLifecycleAction {
            step: 0,
            operation: EffectLifecycleOperation::Install {
                effects: vec![TacticalEffect {
                    id: EffectId::new(),
                    source: EffectSource {
                        definition_id: "isolated-defense-fixture".into(),
                        actor: f.actors[1],
                        command: origin.clone(),
                        ordinal,
                    },
                    established_at: None,
                    target: TacticalEffectTarget::Creature(f.actors[1]),
                    concentration_group: None,
                    expires,
                    overlap: Some(EffectOverlap {
                        key: "same-defense".into(),
                        potency: i32::from(ordinal),
                    }),
                    conditions: vec![],
                    defenses: vec![EffectDefense::BaseArmorClass {
                        base: 13,
                        ability: Ability::Dexterity,
                        ends_when_wearing_armor: true,
                    }],
                    triggers: vec![],
                }],
            },
        },
    )
    .unwrap()
    .0;
    f.state.applied_event_sequence += 1;
}

#[test]
fn release_preflight_is_read_only_and_does_not_treat_ruling_as_permission() {
    let mut f = Fixture::new();
    f.begin();
    assert!(encounter_release_preflight(&f.state).is_err());
    conclude(&mut f);
    let before = f.state.clone();
    let ready = encounter_release_preflight(&f.state).unwrap();
    assert!(ready.required_actors.is_empty());
    assert_eq!(ready.next_turn_number, 2);
    assert_eq!(f.state, before);
    f.run(Some(0), TacticalAction::StartAttackAction);
    let owed = f.state.clone();
    assert!(encounter_release_preflight(&f.state).is_err());
    assert_eq!(f.state, owed);
    f.run(Some(0), TacticalAction::EndTurn);
    encounter_release_preflight(&f.state).unwrap();
}

#[test]
fn paid_ready_and_spent_reaction_must_reach_actual_boundaries() {
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
    conclude(&mut f);
    assert!(encounter_release_preflight(&f.state).is_err());
    f.run(Some(0), TacticalAction::AbandonReady { actor: f.actors[0] });
    encounter_release_preflight(&f.state).unwrap();
    // Isolated expenditure input; production acceptance spends a real Reaction.
    f.state
        .rules
        .as_mut()
        .unwrap()
        .timing
        .as_mut()
        .unwrap()
        .reactions_spent
        .push(f.actors[0]);
    assert!(encounter_release_preflight(&f.state).is_err());
    f.run(Some(0), TacticalAction::EndTurn);
    assert!(encounter_release_preflight(&f.state).is_err());
    f.run(Some(1), TacticalAction::EndTurn);
    encounter_release_preflight(&f.state).unwrap();
}

#[test]
fn global_inspiration_and_dying_actor_are_not_hidden_by_roster() {
    let mut f = Fixture::new();
    f.begin();
    conclude(&mut f);
    let actor = EntityId::new();
    let mut offstage = MechanicalEntity::basic(actor);
    offstage.character_features = Some(features());
    offstage.heroic_inspiration = true;
    offstage
        .character_features
        .as_mut()
        .unwrap()
        .inspiration_transfer_pending = true;
    f.state
        .rules
        .as_mut()
        .unwrap()
        .entities
        .insert(actor, offstage);
    let before = f.state.clone();
    assert!(encounter_release_preflight(&f.state).is_err());
    assert_eq!(f.state, before);
    let offstage = f
        .state
        .rules
        .as_mut()
        .unwrap()
        .entities
        .get_mut(&actor)
        .unwrap();
    offstage
        .character_features
        .as_mut()
        .unwrap()
        .inspiration_transfer_pending = false;
    offstage.hp = 0;
    offstage.prone = true;
    assert!(encounter_release_preflight(&f.state).is_err());
}

#[test]
fn defense_only_absolute_effect_survives_scan_but_dead_source_blocks_release() {
    let mut f = prepared_creature();
    install_defense(
        &mut f,
        0,
        TacticalEffectExpiry::AtTime(WorldInstant(28_800)),
    );
    let before = f.state.clone();
    assert_eq!(
        encounter_release_preflight(&f.state)
            .unwrap()
            .required_actors,
        vec![f.actors[1]]
    );
    assert_eq!(f.state, before);
    assert!(
        f.rules().tactical_effects.as_ref().unwrap().effects[0]
            .conditions
            .is_empty()
    );
    // Negative fixture models the nonconcentration deadline retained after death.
    f.state
        .rules
        .as_mut()
        .unwrap()
        .entities
        .get_mut(&f.actors[1])
        .unwrap()
        .death
        .dead = true;
    let blocked = f.state.clone();
    assert!(encounter_release_preflight(&f.state).is_err());
    assert_eq!(f.state, blocked, "the old timing must remain available");
}

#[test]
fn suppressed_relative_and_per_turn_clauses_still_block_release() {
    let mut f = prepared_creature();
    install_defense(&mut f, 0, TacticalEffectExpiry::Never);
    install_defense(&mut f, 1, TacticalEffectExpiry::Never);
    encounter_release_preflight(&f.state).unwrap();
    for mutation in 0..4 {
        let mut altered = f.state.clone();
        let effect = &mut altered
            .rules
            .as_mut()
            .unwrap()
            .tactical_effects
            .as_mut()
            .unwrap()
            .effects[0];
        match mutation {
            0 => {
                effect.expires = TacticalEffectExpiry::AfterOwnerBoundaries {
                    owner: f.actors[1],
                    boundary: TurnBoundary::Start,
                    remaining: 1,
                }
            }
            1 => effect.triggers.push(EffectTriggerRule {
                event: EffectTriggerEvent::Turn {
                    subject: EffectSubject::Target,
                    boundary: TurnBoundary::End,
                },
                frequency: EffectTriggerFrequency::EveryOccurrence,
                payload: EffectTriggerPayload::EndTargetEffect,
            }),
            2 => effect.triggers.push(EffectTriggerRule {
                event: EffectTriggerEvent::Damage {
                    subject: EffectSubject::Target,
                },
                frequency: EffectTriggerFrequency::OncePerTargetPerTurn {
                    key: "limit".into(),
                },
                payload: EffectTriggerPayload::EndTargetEffect,
            }),
            3 => effect.triggers.push(EffectTriggerRule {
                event: EffectTriggerEvent::ZoneContact(ZoneContact::CreatureEnters),
                frequency: EffectTriggerFrequency::EveryOccurrence,
                payload: EffectTriggerPayload::EndTargetEffect,
            }),
            _ => unreachable!(),
        }
        let before = altered.clone();
        assert!(
            encounter_release_preflight(&altered).is_err(),
            "mutation {mutation}"
        );
        assert_eq!(altered, before);
    }
}

#[test]
fn orphaned_casting_and_due_legacy_effects_are_not_quiescent() {
    let mut f = prepared_creature();
    let origin = f.meta(None);
    f.state
        .rules
        .as_mut()
        .unwrap()
        .tactical_effects
        .as_mut()
        .unwrap()
        .groups
        .push(ConcentrationGroup {
            id: EffectId::new(),
            source: EffectSource {
                definition_id: "isolated-held-cast".into(),
                actor: f.actors[1],
                command: origin,
                ordinal: 0,
            },
            expires: TacticalEffectExpiry::Never,
            stage: ConcentrationStage::Casting,
        });
    assert!(encounter_release_preflight(&f.state).is_err());
    f.state
        .rules
        .as_mut()
        .unwrap()
        .tactical_effects
        .as_mut()
        .unwrap()
        .groups
        .clear();
    for expires in [
        Expiry::AtTime(WorldInstant(0)),
        Expiry::AtTurn {
            actor: f.actors[1],
            boundary: TurnBoundary::End,
            turn_number: 2,
        },
    ] {
        let mut altered = f.state.clone();
        altered.rules.as_mut().unwrap().effects.push(ActiveEffect {
            id: EffectId::new(),
            source: f.actors[1],
            target: f.actors[1],
            condition: None,
            label: "Retained legacy deadline".into(),
            expires,
            concentration_owner: None,
        });
        assert!(encounter_release_preflight(&altered).is_err());
    }
}

#[test]
fn raw_recovery_die_and_due_wake_are_checked_across_the_campaign() {
    let mut f = prepared_creature();
    let origin = f.meta(None);
    let actor = f.actors[1];
    let stable = StableRecovery {
        origin: VitalityOrigin {
            command: origin,
            occurrence: 0,
        },
        stabilized_at: f.state.clock.now,
        delay_roll: None,
    };
    f.state
        .rules
        .as_mut()
        .unwrap()
        .tactical_recovery
        .get_or_insert_default()
        .insert(
            actor,
            TacticalRecovery {
                stable: Some(stable),
                ..TacticalRecovery::default()
            },
        );
    assert!(encounter_release_preflight(&f.state).is_err());
    let raw = RollResult {
        request_id: RollRequestId::new(),
        source: RollSource::Physical,
        dice: vec![DieResult { sides: 4, value: 1 }],
    };
    f.state
        .rules
        .as_mut()
        .unwrap()
        .tactical_recovery
        .as_mut()
        .unwrap()
        .get_mut(&actor)
        .unwrap()
        .stable
        .as_mut()
        .unwrap()
        .delay_roll = Some(raw.clone());
    let before = f.state.clone();
    assert_eq!(
        retained_encounter_dependencies(&f.state).unwrap(),
        vec![actor]
    );
    assert_eq!(f.state, before);
    f.state.clock.now = WorldInstant(3600);
    assert!(encounter_release_preflight(&f.state).is_err());
    assert_eq!(
        f.rules().tactical_recovery.as_ref().unwrap()[&actor]
            .stable
            .as_ref()
            .unwrap()
            .delay_roll,
        Some(raw)
    );
}

/// A structural negative-test fixture. Never a portable
/// capture or a claim that an app command has authenticated these added records.
fn isolated_finished(f: &mut Fixture) {
    encounter_release_preflight(&f.state).unwrap();
    let origin = f.meta(None);
    let encounter = f.state.encounter.as_ref().unwrap();
    let flow = encounter.flow.as_ref().unwrap();
    let timing = f.rules().timing.as_ref().unwrap();
    let location = f.state.scenes[&encounter.scene_id].location_id;
    let receipt = TacticalCompletion {
        encounter_id: encounter.id,
        scene_id: encounter.scene_id,
        location_id: location,
        encounter_origin: encounter.origin.clone(),
        initiative_origin: flow.origin.clone(),
        conclusion_origin: flow.aftermath.as_ref().unwrap().origin.clone(),
        released_by: origin,
        predecessor: None,
        released_at: f.state.clock.now,
        final_turn: EffectTurn {
            actor: timing.order[timing.index].actor,
            number: timing.turn_number,
            boundary: TurnBoundary::Start,
        },
        execution: TacticalExecutionVersion::EncounterReleaseV1,
    };
    let space = TacticalSceneSpace {
        encounter_id: encounter.id,
        scene_id: encounter.scene_id,
        location_id: location,
        release: receipt.released_by.id,
        battlefield: encounter.battlefield.clone(),
        participants: encounter
            .participants
            .iter()
            .map(|participant| participant.entity_id)
            .collect(),
        ground_items: flow.ground_items.clone(),
    };
    f.state.scenes.get_mut(&receipt.scene_id).unwrap().status = SceneStatus::Closed;
    let flow = f.state.encounter.as_mut().unwrap().flow.as_mut().unwrap();
    flow.version = 5;
    flow.phase = TacticalPhase::Finished;
    flow.budget = TacticalTurnBudget::default();
    flow.ground_items.clear();
    let rules = f.state.rules.as_mut().unwrap();
    rules.timing = None;
    let effects = rules.tactical_effects.as_mut().unwrap();
    effects.turn = None;
    effects.trigger_uses.clear();
    if let Some(creatures) = &mut rules.tactical_creatures {
        for runtime in &mut creatures.runtime {
            runtime.observed_turn = None;
            runtime.legendary_window_spent = false;
        }
    }
    f.state.encounter_history = Some(Box::new(TacticalEncounterHistory {
        completions: vec![receipt],
        spaces: vec![space],
    }));
    f.state.applied_event_sequence += 1;
}

fn isolated_replacement(state: &mut CampaignState, origin: CommandMeta) {
    let old_scene = state.encounter.as_ref().unwrap().scene_id;
    let mut scene = state.scenes[&old_scene].clone();
    scene.id = SceneId::new();
    scene.status = SceneStatus::Active;
    let encounter = state.encounter.as_mut().unwrap();
    encounter.id = EncounterId::new();
    encounter.scene_id = scene.id;
    encounter.origin = origin;
    encounter.flow = None;
    state.scenes.insert(scene.id, scene);
    state.applied_event_sequence += 1;
}

#[test]
fn savage_marker_requires_completion_highwater_in_each_no_timing_phase() {
    let mut f = Fixture::new();
    f.entity_mut(0).character_features = Some(features());
    f.begin();
    f.entity_mut(0)
        .character_features
        .as_mut()
        .unwrap()
        .savage_attacker_turn = Some(1);
    conclude(&mut f);
    isolated_finished(&mut f);
    validate_state(&f.state, &f.pack).unwrap();
    assert_eq!(
        f.state
            .encounter_history
            .as_ref()
            .unwrap()
            .next_turn_number()
            .unwrap(),
        2
    );
    let mut missing = f.state.clone();
    missing.encounter_history = None;
    assert!(validate_state(&missing, &f.pack).is_err());
    let mut future = f.state.clone();
    future
        .rules
        .as_mut()
        .unwrap()
        .entities
        .get_mut(&f.actors[0])
        .unwrap()
        .character_features
        .as_mut()
        .unwrap()
        .savage_attacker_turn = Some(2);
    assert!(validate_state(&future, &f.pack).is_err());
    // The same marker survives replacement setup without an initiative cursor.
    let origin = f.meta(None);
    isolated_replacement(&mut f.state, origin);
    validate_state(&f.state, &f.pack).unwrap();
    // This is an isolated structural input, not an authenticated app capture.
    // Continue it through the actual current initiative command.
    let action = TacticalAction::Begin {
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
        execution: TacticalExecutionVersion::EncounterReleaseV1,
    };
    f.run(None, action);
    validate_state(&f.state, &f.pack).unwrap();
    assert!(f.rules().timing.is_none());
    assert!(f.rules().pending.is_some());
    f.roll(0, &[18]);
    f.roll(1, &[3]);
    assert_eq!(f.rules().timing.as_ref().unwrap().turn_number, 2);
    assert_eq!(f.rules().timing.as_ref().unwrap().round, 1);
    assert_eq!(
        f.rules().entities[&f.actors[0]]
            .character_features
            .as_ref()
            .unwrap()
            .savage_attacker_turn,
        Some(1)
    );
}

#[test]
fn finished_source_recharge_still_matches_authoritative_raw_history() {
    let mut f = Fixture::new();
    f.creature(1, false);
    f.creature_runtime_mut(1).recharge[0].available = false;
    f.creature_runtime_mut(1).legendary_spent = 3;
    f.begin();
    f.run(Some(0), TacticalAction::EndTurn);
    let result = f.raw(&[6]);
    f.run(None, TacticalAction::SubmitRoll { result });
    conclude(&mut f);
    isolated_finished(&mut f);
    validate_state(&f.state, &f.pack).unwrap();
    let mut forged = f.state.clone();
    forged
        .rules
        .as_mut()
        .unwrap()
        .tactical_creatures
        .as_mut()
        .unwrap()
        .runtime[0]
        .recharge[0]
        .last_roll
        .as_mut()
        .unwrap()
        .result
        .dice[0]
        .value = 1;
    assert!(validate_tactical_state(&forged).is_err());
    // The proof is also required after replacement setup has no flow at all.
    let origin = f.meta(None);
    for state in [&mut f.state, &mut forged] {
        isolated_replacement(state, origin.clone());
    }
    validate_tactical_state(&f.state).unwrap();
    assert!(validate_tactical_state(&forged).is_err());
}

#[test]
fn history_is_absent_on_old_wire_and_rejects_reused_origins_and_highwater() {
    let mut f = Fixture::new();
    f.begin();
    let old = serde_json::to_value(&f.state).unwrap();
    assert!(old.get("encounter_history").is_none());
    assert_eq!(
        serde_json::to_value(serde_json::from_value::<CampaignState>(old.clone()).unwrap())
            .unwrap(),
        old
    );
    conclude(&mut f);
    isolated_finished(&mut f);
    for mutation in 0..8 {
        let mut forged = f.state.clone();
        let history = forged.encounter_history.as_mut().unwrap();
        match mutation {
            0 => history.completions[0].predecessor = Some(CommandId::new()),
            1 => history.completions[0].final_turn.number = u64::MAX,
            2 => {
                history.completions[0].released_by =
                    history.completions[0].conclusion_origin.clone()
            }
            3 => history.spaces[0].release = CommandId::new(),
            4 => history.spaces[0].battlefield.bounds.max.x += 1,
            5 => {
                history.completions.push(history.completions[0].clone());
                history.spaces.push(history.spaces[0].clone());
            }
            6 => {
                let id = history.completions[0].scene_id;
                forged.scenes.get_mut(&id).unwrap().status = SceneStatus::Active;
            }
            7 => {
                // Pass the ordinary newer-origin/fresh-scene predecessor guards;
                // only the exact Finished-to-latest-receipt binding rejects this.
                let finished = forged.encounter.as_ref().unwrap().flow.clone();
                isolated_replacement(&mut forged, f.meta(None));
                forged.encounter.as_mut().unwrap().flow = finished;
            }
            _ => unreachable!(),
        }
        if mutation == 7 {
            assert_eq!(
                forged.encounter_history.as_ref().unwrap().validate(&forged),
                Err("Finished encounter is not the latest authenticated completion".into())
            );
        } else {
            assert!(
                validate_tactical_state(&forged).is_err(),
                "mutation {mutation}"
            );
        }
    }
}

#[test]
fn finished_without_receipt_and_conclusion_is_rejected_independently_of_savage() {
    let mut f = Fixture::new();
    f.begin();
    conclude(&mut f);
    isolated_finished(&mut f);
    f.state.encounter_history = None;
    f.state
        .encounter
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .aftermath = None;
    assert!(validate_tactical_state(&f.state).is_err());
    assert!(!f.state.validate().is_empty());
}

#[test]
fn retired_item_placement_keeps_original_cause_and_cannot_move_with_replacement() {
    let mut f = Fixture::new();
    f.begin();
    conclude(&mut f);
    let item = ItemId::new();
    let original_drop = f.meta(None);
    let encounter = f.state.encounter.as_ref().unwrap();
    let location = f.state.scenes[&encounter.scene_id].location_id;
    let position = encounter.participants[0].position;
    f.state.items.insert(
        item,
        ItemInstance {
            id: item,
            campaign_id: f.state.campaign_id(),
            definition_id: "dagger".into(),
            display_name: "Unit fixture dropped dagger".into(),
            quantity: 1,
            owner: Ownership::Entity(f.actors[0]),
            custody: Custody::Location(location),
            state: ItemState::Intact,
        },
    );
    let placement = TacticalGroundItem {
        item,
        position,
        origin: original_drop.clone(),
    };
    f.state
        .encounter
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .ground_items
        .push(placement.clone());
    f.state.applied_event_sequence += 1;
    isolated_finished(&mut f);
    validate_tactical_state(&f.state).unwrap();
    assert_eq!(
        f.state.encounter_history.as_ref().unwrap().spaces[0].ground_items,
        vec![placement.clone()]
    );
    for mutation in 0..3 {
        let mut forged = f.state.clone();
        match mutation {
            0 => forged
                .encounter
                .as_mut()
                .unwrap()
                .flow
                .as_mut()
                .unwrap()
                .ground_items
                .push(placement.clone()),
            1 => forged.items.get_mut(&item).unwrap().custody = Custody::Entity(f.actors[0]),
            2 => {
                forged.encounter_history.as_mut().unwrap().spaces[0].ground_items[0]
                    .position
                    .x = 10_000
            }
            _ => unreachable!(),
        }
        assert!(validate_tactical_state(&forged).is_err());
    }
    let origin = f.meta(None);
    isolated_replacement(&mut f.state, origin);
    f.state.encounter.as_mut().unwrap().battlefield.bounds.max.x = 200;
    validate_tactical_state(&f.state).unwrap();
    assert_eq!(
        f.state.encounter_history.as_ref().unwrap().spaces[0].ground_items[0].origin,
        original_drop
    );
    assert_eq!(
        f.state.encounter_history.as_ref().unwrap().spaces[0].ground_items[0].position,
        position
    );
    assert_eq!(f.state.items[&item].custody, Custody::Location(location));
    // A new scene may share a location; it cannot overwrite the retained scene.
    let old_scene = f.state.encounter_history.as_ref().unwrap().spaces[0].scene_id;
    f.state.encounter.as_mut().unwrap().scene_id = old_scene;
    assert!(validate_tactical_state(&f.state).is_err());
}

#[test]
fn dependency_scan_uses_actual_collective_setup_capacity_and_scene_authority() {
    let mut f = prepared_creature();
    install_defense(
        &mut f,
        0,
        TacticalEffectExpiry::AtTime(WorldInstant(28_800)),
    );
    let mut other_scene = f.state.scenes.values().next().unwrap().clone();
    other_scene.id = SceneId::new();
    other_scene
        .presences
        .retain(|presence| presence.entity_id == f.actors[1]);
    f.state.scenes.insert(other_scene.id, other_scene);
    assert!(
        encounter_release_preflight(&f.state)
            .unwrap_err()
            .to_string()
            .contains("another active scene")
    );
    let mut many = Fixture::new();
    many.begin();
    conclude(&mut many);
    for _ in 0..=MAX_RELEASE_DEPENDENCIES {
        let actor = EntityId::new();
        many.state
            .rules
            .as_mut()
            .unwrap()
            .effects
            .push(ActiveEffect {
                id: EffectId::new(),
                source: actor,
                target: actor,
                condition: None,
                label: "Unit fixture absolute dependency".into(),
                expires: Expiry::AtTime(WorldInstant(28_800)),
                concentration_owner: None,
            });
    }
    assert!(
        retained_encounter_dependencies(&many.state)
            .unwrap_err()
            .to_string()
            .contains("exceed")
    );
    many.state.rules.as_mut().unwrap().effects.pop();
    assert!(
        retained_encounter_dependencies(&many.state)
            .unwrap_err()
            .to_string()
            .contains("required character or owned-source seat")
    );
    assert_eq!(
        MAX_TACTICAL_PARTICIPANTS, 128,
        "historical domain capacity is unchanged"
    );
}

#[test]
fn replacement_setup_and_pending_initiative_cannot_omit_retained_dependencies() {
    let mut f = prepared_creature();
    install_defense(
        &mut f,
        0,
        TacticalEffectExpiry::AtTime(WorldInstant(28_800)),
    );
    let effects_before = f.rules().tactical_effects.as_ref().unwrap().effects.clone();
    isolated_finished(&mut f);
    let origin = f.meta(None);
    isolated_replacement(&mut f.state, origin);
    validate_tactical_state(&f.state).unwrap();
    let mut omitted = f.state.clone();
    omitted
        .encounter
        .as_mut()
        .unwrap()
        .participants
        .retain(|participant| participant.entity_id != f.actors[1]);
    assert!(
        validate_tactical_state(&omitted)
            .unwrap_err()
            .to_string()
            .contains("retained timing dependency")
    );
    let action = TacticalAction::Begin {
        combatants: f
            .actors
            .into_iter()
            .map(|actor| TacticalCombatant {
                actor,
                source: f
                    .rules()
                    .tactical_creatures
                    .as_ref()
                    .and_then(|creatures| creatures.profile(actor))
                    .map_or(TacticalSource::Character, |profile| {
                        TacticalSource::Creature {
                            definition_id: profile.source.definition_id.clone(),
                        }
                    }),
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
        execution: TacticalExecutionVersion::EncounterReleaseV1,
    };
    f.run(None, action);
    validate_state(&f.state, &f.pack).unwrap();
    assert!(f.rules().pending.is_some());
    assert_eq!(
        f.rules().tactical_effects.as_ref().unwrap().effects,
        effects_before
    );
    let mut omitted = f.state.clone();
    omitted
        .encounter
        .as_mut()
        .unwrap()
        .participants
        .retain(|participant| participant.entity_id != f.actors[1]);
    assert!(
        validate_tactical_state(&omitted)
            .unwrap_err()
            .to_string()
            .contains("retained timing dependency")
    );
}
