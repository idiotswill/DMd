use super::*;
use dmd_rules::tactical_creature_equipment::*;
use dmd_rules::tactical_creatures::*;

fn source(f: &mut Fixture, actor: EntityId, id: &str, player: PlayerId) {
    f.state.characters.retain(|_, c| c.entity_id != actor);
    f.state.entities.get_mut(&actor).unwrap().kind = EntityKind::Creature;
    f.state.rules.as_mut().unwrap().entities.remove(&actor);
    let origin = f.meta(None);
    let definition = creature_definition(id).unwrap();
    let built = build_creature(
        &f.state,
        &origin,
        actor,
        &CreatureBuildChoice {
            definition_id: id.into(),
            size: source_size(definition.statistics.size),
            additional_languages: vec![],
            hit_points: CreatureHitPointChoice::Average,
            controller: CreatureController::Player(player),
            in_lair: false,
        },
    )
    .unwrap();
    let participant = f
        .state
        .encounter
        .as_mut()
        .unwrap()
        .participants
        .iter_mut()
        .find(|p| p.entity_id == actor)
        .unwrap();
    participant.size = built.profile.size;
    participant.movement = built.movement;
    participant.senses = built.senses;
    let rules = f.state.rules.as_mut().unwrap();
    rules.entities.insert(actor, built.mechanics);
    let creatures = rules.tactical_creatures.get_or_insert_default();
    creatures.profiles.push(built.profile);
    creatures.runtime.push(built.runtime);
    let ids = creature_equipment_plan(id, 0)
        .unwrap()
        .iter()
        .map(|_| ItemId::new())
        .collect::<Vec<_>>();
    f.state = materialize_creature_equipment(&f.state, &origin, actor, 0, &ids, &f.pack).unwrap();
    f.state.applied_event_sequence += 1;
}
fn fixture(id: &str, two_targets: bool) -> (Fixture, Option<EntityId>) {
    let mut f = Fixture::new();
    let actor = f.actors[0];
    let player = f.players[0];
    source(&mut f, actor, id, player);
    let encounter = f.state.encounter.as_mut().unwrap();
    encounter.battlefield.bounds.max = SpatialPoint {
        x: 200,
        y: 200,
        z: 100,
    };
    encounter.area_grid_policy = Some(TacticalAreaGridPolicy::OccupiedCellCentersV1);
    encounter.participants[0].position = SpatialPoint { x: 10, y: 40, z: 0 };
    encounter.participants[0].height = 10;
    encounter.participants[1].position = SpatialPoint { x: 50, y: 40, z: 0 };
    encounter.participants[1].height = 10;
    let third = if two_targets {
        let id = EntityId::new();
        let mut participant = encounter.participants[1].clone();
        participant.entity_id = id;
        participant.position.y = 50;
        encounter.participants.push(participant);
        let character_id = CharacterId::new();
        f.state.characters.insert(
            character_id,
            Character {
                id: character_id,
                campaign_id: f.state.campaign_id(),
                entity_id: id,
                controlling_player_id: Some(f.players[1]),
                display_name: "Other adventurer".into(),
                status: CharacterStatus::Active,
            },
        );
        let mut entity = f.state.entities[&f.actors[1]].clone();
        entity.id = id;
        f.state.entities.insert(id, entity);
        f.state
            .rules
            .as_mut()
            .unwrap()
            .entities
            .insert(id, MechanicalEntity::basic(id));
        f.state
            .scenes
            .values_mut()
            .next()
            .unwrap()
            .presences
            .push(ScenePresence {
                entity_id: id,
                role: PresenceRole::Participant,
            });
        Some(id)
    } else {
        None
    };
    for target in [Some(f.actors[1]), third].into_iter().flatten() {
        let entity = f
            .state
            .rules
            .as_mut()
            .unwrap()
            .entities
            .get_mut(&target)
            .unwrap();
        entity.max_hp = 100;
        entity.hp = 100;
    }
    (f, third)
}
fn begin(f: &mut Fixture) {
    let actors = f
        .state
        .encounter
        .as_ref()
        .unwrap()
        .participants
        .iter()
        .map(|p| p.entity_id)
        .collect::<Vec<_>>();
    f.run(
        None,
        TacticalAction::Begin {
            combatants: actors
                .iter()
                .map(|actor| TacticalCombatant {
                    actor: *actor,
                    surprised: false,
                    source: f
                        .rules()
                        .tactical_creatures
                        .as_ref()
                        .and_then(|c| c.profile(*actor))
                        .map_or(TacticalSource::Character, |p| TacticalSource::Creature {
                            definition_id: p.source.definition_id.clone(),
                        }),
                })
                .collect(),
            groups: actors
                .iter()
                .map(|actor| InitiativeGroup {
                    actors: vec![*actor],
                    request_id: RollRequestId::new(),
                })
                .collect(),
        },
    );
    for value in [18, 5, 1].into_iter().take(actors.len()) {
        raw(f, &[value]);
    }
    assert_eq!(
        f.rules().timing.as_ref().unwrap().order[0].actor,
        f.actors[0]
    );
}
fn raw(f: &mut Fixture, values: &[u16]) {
    let result = f.raw(values);
    f.run(None, TacticalAction::SubmitRoll { result });
}
fn aim(f: &Fixture) -> TacticalAreaAim {
    let p = f
        .state
        .encounter
        .as_ref()
        .unwrap()
        .participant(f.actors[0])
        .unwrap();
    TacticalAreaAim {
        origin: SpatialPoint {
            x: p.volume().unwrap().max.x,
            y: 50,
            z: 5,
        },
        toward: SpatialPoint {
            x: 180,
            y: 50,
            z: 5,
        },
        include_origin: false,
    }
}
fn action(f: &Fixture) -> TacticalAction {
    TacticalAction::CreatureArea {
        feature_id: "fire-breath".into(),
        aim: aim(f),
        ordering: TacticalAreaOrdering::DelegateToHost,
    }
}
fn record(f: &Fixture) -> &TacticalArea {
    &f.flow().resolution.as_ref().unwrap().areas[0]
}
fn choose(f: &mut Fixture, actor: EntityId, save: bool) {
    let r = f.flow().resolution.as_ref().unwrap();
    let work = r
        .frames
        .last()
        .unwrap()
        .iter()
        .find(|work| {
            let target = match work.kind {
                TacticalWorkKind::AreaSave { target, .. } if save => target,
                TacticalWorkKind::ApplyAreaDamage { target, .. } if !save => target,
                _ => return false,
            };
            r.areas[0].targets[usize::from(target)].actor == actor
        })
        .unwrap()
        .occurrence;
    f.run(None, TacticalAction::ChooseTurnWork { occurrence: work });
}
fn focus(f: &mut Fixture, actor: EntityId) -> EffectId {
    // Imported legitimate concentration state isolates damage obligations. This
    // does not grant any new spell to the supported player creation catalog.
    let meta = f.meta(None);
    let id = EffectId::new();
    f.state = tactical_effect_adapter::apply_effect_operation(
        &f.state,
        &meta,
        &EffectLifecycleAction {
            step: 0,
            operation: EffectLifecycleOperation::BeginConcentration {
                group: ConcentrationGroup {
                    id,
                    source: EffectSource {
                        definition_id: "imported-source-focus".into(),
                        actor,
                        command: meta.clone(),
                        ordinal: 0,
                    },
                    expires: TacticalEffectExpiry::Never,
                    stage: ConcentrationStage::Casting,
                },
            },
        },
    )
    .unwrap()
    .0;
    f.state.applied_event_sequence += 1;
    id
}

#[test]
fn area_source_cost_recharge_and_single_raw_amount_are_atomic_and_replayed() {
    for (id, count, sides, modifier) in [
        ("chimera", 7, 8, 0),
        ("young-red-dragon", 16, 6, 0),
        ("adult-red-dragon", 17, 6, 0),
    ] {
        let (mut f, _) = fixture(id, false);
        f.entity_mut(1).ability_scores[Ability::Dexterity.index()] = 18;
        begin(&mut f);
        let before = f.state.clone();
        f.rejected(Some(1), action(&f));
        let mut no_policy = f.state.clone();
        no_policy.encounter.as_mut().unwrap().area_grid_policy = None;
        assert!(resolve_tactical(&no_policy, &f.meta(Some(0)), &action(&f), &f.pack).is_err());
        assert_eq!(before, f.state);
        f.run(Some(0), action(&f));
        assert_eq!(f.request().dice, vec![DieSpec { count, sides }]);
        assert_eq!(f.request().modifier, modifier);
        assert_eq!(f.request().roller, Some(f.actors[0]));
        assert!(f.rules().timing.as_ref().unwrap().action_spent);
        assert!(
            !f.rules()
                .tactical_creatures
                .as_ref()
                .unwrap()
                .runtime(f.actors[0])
                .unwrap()
                .recharge
                .iter()
                .find(|r| r.feature_id == "fire-breath")
                .unwrap()
                .available
        );
        f.rejected(
            Some(1),
            TacticalAction::SubmitRoll {
                result: f.raw(&vec![1; count as usize]),
            },
        );
        raw(&mut f, &vec![1; count as usize]);
        assert_eq!(f.request().roller, Some(f.actors[1]));
        raw(&mut f, &[20]);
        assert_eq!(
            f.rules().entities[&f.actors[1]].hp,
            100 - u32::from(count / 2)
        );
        assert!(f.flow().resolution.is_none());
        assert_eq!(f.rules().rolls.iter().filter(|r| matches!(r.purpose, PendingPurpose::TacticalResolution { key, .. } if key.role == TacticalRollRole::AreaDamage)).count(), 1);
        assert!(f.flow().budget.weapon_history.is_empty());
        assert_eq!(f.flow().budget.attacks_remaining, 0);
        f.rejected(Some(0), action(&f));
    }
}

#[test]
fn all_area_saves_precede_damage_then_each_target_has_its_own_concentration_children() {
    let (mut f, third) = fixture("chimera", true);
    let first = f.actors[1];
    let third = third.unwrap();
    f.entity_mut(1).resistances.insert(DamageType::Fire);
    f.entity_mut(1).temporary_hp = 20;
    f.state
        .rules
        .as_mut()
        .unwrap()
        .entities
        .get_mut(&third)
        .unwrap()
        .vulnerabilities
        .insert(DamageType::Fire);
    begin(&mut f);
    let first_group = focus(&mut f, first);
    let third_group = focus(&mut f, third);
    f.run(Some(0), action(&f));
    raw(&mut f, &[8, 7, 6, 5, 3, 1, 1]); // One 31-point amount for both victims.
    let amount = record(&f).damage.unwrap();
    choose(&mut f, first, true);
    f.roll(1, &[15]);
    assert_eq!(f.rules().entities[&first].hp, 100);
    assert_eq!(f.rules().entities[&third].hp, 100);
    assert_eq!(f.request().roller, Some(third));
    raw(&mut f, &[1]);
    assert!(
        record(&f)
            .targets
            .iter()
            .all(|t| t.save.is_some() && t.applied_by.is_none())
    );
    choose(&mut f, first, false);
    assert_eq!(f.rules().entities[&first].temporary_hp, 13); // floor(floor(31/2)/2).
    assert_eq!(f.rules().entities[&first].hp, 100);
    assert_eq!(f.request().roller, Some(first));
    assert_eq!(
        f.flow()
            .resolution
            .as_ref()
            .unwrap()
            .pending
            .as_ref()
            .unwrap()
            .key
            .role,
        TacticalRollRole::Concentration
    );
    raw(&mut f, &[10]);
    assert_eq!(f.rules().entities[&first].concentration, Some(first_group));
    assert_eq!(f.rules().entities[&third].hp, 38);
    assert_eq!(f.request().roller, Some(third));
    raw(&mut f, &[20]); // RAW 20 is below DC30.
    assert_eq!(f.rules().entities[&third].concentration, None);
    assert!(
        !f.rules()
            .tactical_effects
            .as_ref()
            .unwrap()
            .groups
            .iter()
            .any(|g| g.id == third_group)
    );
    assert!(f.flow().resolution.is_none());
    assert_eq!(
        f.rules()
            .rolls
            .iter()
            .filter(|r| r.request.id == amount.request_id())
            .count(),
        1
    );
}

#[test]
fn area_house_natural_extremes_and_raw_default_use_the_same_save_outcome_policy() {
    for house in [false, true] {
        let (mut f, _) = fixture("chimera", false);
        f.state
            .rules
            .as_mut()
            .unwrap()
            .house_rules
            .ability_test_natural_extremes = house;
        f.entity_mut(1).exhaustion = 3; // Natural20 total14 against DC15.
        begin(&mut f);
        f.run(Some(0), action(&f));
        raw(&mut f, &[1; 7]);
        assert_eq!(f.request().modifier, -6);
        raw(&mut f, &[20]);
        assert_eq!(
            f.rules().entities[&f.actors[1]].hp,
            if house { 97 } else { 93 }
        );
    }
}

#[test]
fn area_automatic_and_voluntary_failure_never_invent_raw_save_faces() {
    for automatic in [false, true] {
        let (mut f, _) = fixture("chimera", false);
        begin(&mut f);
        if automatic {
            let target = f.actors[1];
            let source = f.actors[0];
            f.state.rules.as_mut().unwrap().effects.push(ActiveEffect {
                id: EffectId::new(),
                source,
                target,
                condition: Some(Condition::Paralyzed),
                label: "Imported paralysis".into(),
                expires: Expiry::Never,
                concentration_owner: None,
            });
        }
        f.run(Some(0), action(&f));
        raw(&mut f, &[1; 7]);
        if !automatic {
            f.run(Some(1), TacticalAction::VoluntarilyFailSave);
        }
        assert_eq!(f.rules().entities[&f.actors[1]].hp, 93);
        assert_eq!(f.rules().rolls.iter().filter(|r| matches!(r.purpose, PendingPurpose::TacticalResolution { key, .. } if key.role == TacticalRollRole::AreaSave)).count(), 0);
        assert_eq!(
            f.flow().save_decisions.last().unwrap().failure,
            if automatic {
                TacticalSaveFailure::Automatic
            } else {
                TacticalSaveFailure::Voluntary
            }
        );
        assert!(f.flow().resolution.is_none());
    }
}

#[test]
fn area_partition_source_and_raw_evidence_corruption_reject_without_mutation() {
    let (mut f, _) = fixture("chimera", true);
    begin(&mut f);
    f.run(Some(0), action(&f));
    raw(&mut f, &[1; 7]);
    for mutation in 0..7 {
        let mut bad = f.state.clone();
        let r = bad
            .encounter
            .as_mut()
            .unwrap()
            .flow
            .as_mut()
            .unwrap()
            .resolution
            .as_mut()
            .unwrap();
        match mutation {
            0 => {
                r.frames.last_mut().unwrap().pop();
            }
            1 => {
                let w = r.frames.last().unwrap()[0].clone();
                r.frames.last_mut().unwrap().push(w);
            }
            2 => r.areas[0].source.pin.definition_id = "adult-red-dragon".into(),
            3 => r.areas[0].geometry_origin.id = CommandId::new(),
            4 => r.areas[0].damage.as_mut().unwrap().occurrence = 0,
            5 => {
                let t = r.areas[0].targets[0].clone();
                r.areas[0].targets[1] = t;
            }
            6 => {
                let phase = r
                    .frames
                    .iter()
                    .position(|frame| {
                        frame.iter().any(|work| {
                            matches!(work.kind, TacticalWorkKind::BeginAreaDamage { .. })
                        })
                    })
                    .unwrap();
                let saves = r
                    .frames
                    .iter()
                    .position(|frame| {
                        frame
                            .iter()
                            .any(|work| matches!(work.kind, TacticalWorkKind::AreaSave { .. }))
                    })
                    .unwrap();
                r.frames.swap(phase, saves);
            }
            _ => unreachable!(),
        }
        assert!(validate_tactical_state(&bad).is_err(), "forgery {mutation}");
    }
}

#[test]
fn area_absent_and_hidden_victims_do_not_change_source_amount_request() {
    for hidden in [false, true] {
        let (mut f, _) = fixture("chimera", false);
        if hidden {
            f.state
                .encounter
                .as_mut()
                .unwrap()
                .battlefield
                .ambient_light = LightLevel::Darkness;
        } else {
            f.state.encounter.as_mut().unwrap().participants[1]
                .position
                .x = 150;
        }
        begin(&mut f);
        f.run(Some(0), action(&f));
        assert_eq!(f.request().dice, vec![DieSpec { count: 7, sides: 8 }]);
        assert_eq!(f.request().reason, "Damage from the accepted area effect");
        raw(&mut f, &[1; 7]);
        if hidden {
            assert_eq!(f.request().roller, Some(f.actors[1]));
            raw(&mut f, &[15]);
        }
        assert!(f.flow().resolution.is_none());
    }
}

#[test]
fn area_source_legendary_resistance_changes_only_the_save_outcome() {
    for use_resistance in [false, true] {
        let (mut f, _) = fixture("chimera", false);
        let target = f.actors[1];
        let player = f.players[1];
        source(&mut f, target, "adult-red-dragon", player);
        begin(&mut f);
        let hp = f.rules().entities[&target].hp;
        f.run(Some(0), action(&f));
        raw(&mut f, &[1; 7]);
        f.roll(1, &[1]);
        let failed = f
            .flow()
            .resolution
            .as_ref()
            .unwrap()
            .failed_save
            .as_ref()
            .unwrap();
        let key = failed.pending.key;
        assert_eq!(key.role, TacticalRollRole::AreaSave);
        assert_eq!(failed.result.as_ref().unwrap().dice[0].value, 1);
        f.rejected(Some(0), TacticalAction::UseLegendaryResistance);
        f.run(
            Some(1),
            if use_resistance {
                TacticalAction::UseLegendaryResistance
            } else {
                TacticalAction::DeclineLegendaryResistance
            },
        );
        let runtime = f
            .rules()
            .tactical_creatures
            .as_ref()
            .unwrap()
            .runtime(target)
            .unwrap();
        assert_eq!(runtime.legendary_resistance_spent, u8::from(use_resistance));
        assert_eq!(
            runtime
                .legendary_resistance_rolls
                .contains(&key.request_id()),
            use_resistance
        );
        assert_eq!(f.rules().entities[&target].hp, hp); // Actual source Fire immunity.
        assert!(f.flow().resolution.is_none());
    }
}

#[test]
fn area_save_cover_is_derived_once_from_the_host_policy_and_changes_the_result() {
    let (mut f, _) = fixture("chimera", false);
    f.state
        .encounter
        .as_mut()
        .unwrap()
        .battlefield
        .obstacles
        .push(SpatialObstacle {
            id: "authored-low-wall".into(),
            volume: SpatialBox {
                min: SpatialPoint { x: 40, y: 0, z: 0 },
                max: SpatialPoint {
                    x: 41,
                    y: 100,
                    z: 30,
                },
            },
            blocks_movement: false,
            blocks_sight: false,
            observable: true,
            cover: CoverDegree::Half,
        });
    begin(&mut f);
    f.run(Some(0), action(&f));
    raw(&mut f, &[1; 7]);
    assert_eq!(f.request().modifier, 2);
    f.roll(1, &[13]);
    assert_eq!(f.rules().entities[&f.actors[1]].hp, 97);
}

#[test]
fn optional_area_policy_preserves_absent_wire_bytes_and_rejects_unknown_values() {
    let f = Fixture::new();
    let encounter = f.state.encounter.as_ref().unwrap();
    let old = serde_json::to_value(encounter).unwrap();
    assert!(old.get("area_grid_policy").is_none());
    let decoded: TacticalEncounter = serde_json::from_value(old.clone()).unwrap();
    assert_eq!(decoded.area_grid_policy, None);
    assert_eq!(serde_json::to_value(decoded).unwrap(), old);
    let mut unknown = old;
    unknown["area_grid_policy"] = serde_json::json!("TrustCallerTargets");
    assert!(serde_json::from_value::<TacticalEncounter>(unknown).is_err());
}

#[test]
fn charmed_area_failure_is_uniform_before_aim_and_never_omits_the_charmer() {
    let mut messages = vec![];
    for hidden in [false, true] {
        let (mut f, _) = fixture("chimera", false);
        begin(&mut f);
        let source = f.actors[1];
        let actor = f.actors[0];
        f.state.rules.as_mut().unwrap().effects.push(ActiveEffect {
            id: EffectId::new(),
            source,
            target: actor,
            condition: Some(Condition::Charmed),
            label: "Imported sourced charm".into(),
            expires: Expiry::Never,
            concentration_owner: None,
        });
        if hidden {
            f.state
                .encounter
                .as_mut()
                .unwrap()
                .battlefield
                .ambient_light = LightLevel::Darkness;
            f.state.encounter.as_mut().unwrap().participants[1]
                .position
                .x = 150;
        }
        let before = f.state.clone();
        let error = resolve_tactical(&f.state, &f.meta(Some(0)), &action(&f), &f.pack).unwrap_err();
        let RulesError::Prerequisite(message) = error else {
            panic!("unexpected error: {error:?}");
        };
        messages.push(message);
        assert_eq!(before, f.state);
        assert!(!f.rules().timing.as_ref().unwrap().action_spent);
    }
    assert_eq!(messages[0], messages[1]);
    assert!(messages[0].contains("Charmed"));
}

#[test]
fn area_does_not_bind_a_cone_before_unresolved_physical_falling() {
    let (mut f, _) = fixture("chimera", false);
    begin(&mut f);
    f.state.encounter.as_mut().unwrap().participants[1]
        .movement
        .fly = None;
    f.state.encounter.as_mut().unwrap().participants[1]
        .position
        .z = 40;
    let before = f.state.clone();
    let error = resolve_tactical(&f.state, &f.meta(Some(0)), &action(&f), &f.pack).unwrap_err();
    // The existing whole-state validator catches an impossible idle landing even
    // before area admission's defensive preflight can run.
    assert!(
        matches!(&error, RulesError::Invalid(message) if message.contains("queued landing")),
        "{error:?}"
    );
    assert_eq!(f.state, before);
}

#[test]
fn actual_area_damage_pumps_new_falling_before_idle_without_rebinding_its_old_victim() {
    let (mut f, third) = fixture("chimera", true);
    let first = f.actors[1];
    let third = third.unwrap();
    f.state.encounter.as_mut().unwrap().participants[0].height = 30;
    for participant in &mut f.state.encounter.as_mut().unwrap().participants[1..] {
        participant.position.z = 20;
    }
    f.entity_mut(1).hp = 1;
    begin(&mut f);
    focus(&mut f, third);
    let mut selected = aim(&f);
    selected.origin.z = 25;
    selected.toward.z = 25;
    f.run(
        Some(0),
        TacticalAction::CreatureArea {
            feature_id: "fire-breath".into(),
            aim: selected,
            ordering: TacticalAreaOrdering::DelegateToHost,
        },
    );
    raw(&mut f, &[1; 7]);
    choose(&mut f, first, true);
    f.roll(1, &[1]);
    raw(&mut f, &[1]);
    choose(&mut f, first, false);
    assert_eq!(f.rules().entities[&first].hp, 0);
    assert_eq!(
        f.flow()
            .resolution
            .as_ref()
            .unwrap()
            .pending
            .as_ref()
            .unwrap()
            .key
            .role,
        TacticalRollRole::FallDamage
    );
    raw(&mut f, &[1]);
    // Falling finishes before the remaining target's concentration pause. The
    // original source area retains its actual admitted volume after displacement.
    assert_eq!(
        record(&f)
            .targets
            .iter()
            .find(|target| target.actor == first)
            .unwrap()
            .volume
            .min
            .z,
        20
    );
    assert_eq!(
        f.state
            .encounter
            .as_ref()
            .unwrap()
            .participant(first)
            .unwrap()
            .position
            .z,
        0
    );
    assert_eq!(f.request().roller, Some(third));
    assert_eq!(
        f.flow()
            .resolution
            .as_ref()
            .unwrap()
            .pending
            .as_ref()
            .unwrap()
            .key
            .role,
        TacticalRollRole::Concentration
    );
    raw(&mut f, &[10]);
    assert!(f.flow().resolution.is_none());
    assert_eq!(f.rules().entities[&first].death.failures, 1);
}

#[test]
fn area_damage_at_zero_hp_is_one_death_failure_without_attack_critical_or_knockout() {
    let (mut f, _) = fixture("chimera", false);
    begin(&mut f);
    f.entity_mut(1).hp = 0;
    f.entity_mut(1).prone = true;
    f.run(Some(0), action(&f));
    raw(&mut f, &[1; 7]);
    let target = &f.rules().entities[&f.actors[1]];
    assert_eq!(target.hp, 0);
    assert_eq!(target.death.failures, 1);
    assert!(!target.death.dead);
    assert!(f.flow().resolution.is_none());
}

#[test]
fn completed_area_save_must_reconstruct_its_source_request_before_other_saves_finish() {
    let (mut f, _) = fixture("chimera", true);
    begin(&mut f);
    f.run(Some(0), action(&f));
    raw(&mut f, &[1; 7]);
    let first = f.actors[1];
    choose(&mut f, first, true);
    f.roll(1, &[10]);
    let save = record(&f)
        .targets
        .iter()
        .find(|target| target.actor == first)
        .unwrap()
        .save
        .as_ref()
        .unwrap()
        .clone();
    let mut bad = f.state.clone();
    let roll = bad
        .rules
        .as_mut()
        .unwrap()
        .rolls
        .iter_mut()
        .find(|roll| roll.request.id == save.key.request_id())
        .unwrap();
    roll.request.modifier += 10;
    roll.resolved = roll.request.resolve(&roll.result).unwrap();
    let area = &mut bad
        .encounter
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .resolution
        .as_mut()
        .unwrap()
        .areas[0];
    area.targets
        .iter_mut()
        .find(|target| target.actor == first)
        .unwrap()
        .save
        .as_mut()
        .unwrap()
        .succeeded = true;
    // Arithmetic and canonical key remain self-consistent; source request proof
    // still rejects the fabricated modifier while the second save is outstanding.
    assert!(
        matches!(validate_tactical_state(&bad), Err(RulesError::Invalid(message)) if message.contains("source request"))
    );
}

#[test]
fn area_ordering_requires_explicit_controller_consent_before_geometry_or_cost() {
    let (mut f, _) = fixture("chimera", true);
    begin(&mut f);
    let accepted = action(&f);
    let mut missing = serde_json::to_value(&accepted).unwrap();
    missing["CreatureArea"]
        .as_object_mut()
        .unwrap()
        .remove("ordering");
    assert!(serde_json::from_value::<TacticalAction>(missing).is_err());
    f.rejected(None, accepted.clone());
    f.rejected(Some(1), accepted.clone());
    let mut hostile = accepted.clone();
    if let TacticalAction::CreatureArea { ordering, aim, .. } = &mut hostile {
        *ordering = TacticalAreaOrdering::Host;
        aim.origin.x = i32::MAX;
    }
    let error = resolve_tactical(&f.state, &f.meta(Some(0)), &hostile, &f.pack).unwrap_err();
    assert!(
        error.to_string().contains("explicitly authorize"),
        "{error}"
    );
    f.rejected(Some(0), hostile);
    let event = f.run(Some(0), accepted);
    assert_eq!(record(&f).source.invocation, event.meta);
    assert_eq!(record(&f).ordering, TacticalAreaOrdering::DelegateToHost);
    let mut forged = f.state.clone();
    forged
        .encounter
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .resolution
        .as_mut()
        .unwrap()
        .areas[0]
        .ordering = TacticalAreaOrdering::Host;
    assert!(validate_tactical_state(&forged).is_err());
}

#[test]
fn delegated_area_ordering_keeps_each_save_with_its_actual_controller() {
    let (mut f, _) = fixture("chimera", true);
    begin(&mut f);
    f.run(Some(0), action(&f));
    f.roll(0, &[1; 7]);
    let resolution = f.flow().resolution.as_ref().unwrap();
    let occurrence = resolution
        .frames
        .last()
        .unwrap()
        .iter()
        .find(|work| {
            matches!(work.kind, TacticalWorkKind::AreaSave { target, .. }
            if resolution.areas[0].targets[usize::from(target)].actor == f.actors[1])
        })
        .unwrap()
        .occurrence;
    f.rejected(Some(0), TacticalAction::ChooseTurnWork { occurrence });
    f.rejected(Some(1), TacticalAction::ChooseTurnWork { occurrence });
    f.run(None, TacticalAction::ChooseTurnWork { occurrence });
    let raw = f.raw(&[10]);
    f.rejected(Some(0), TacticalAction::SubmitRoll { result: raw });
    f.roll(1, &[10]);
    assert_eq!(
        record(&f)
            .targets
            .iter()
            .filter(|target| target.save.is_some())
            .count(),
        1
    );
    assert_eq!(record(&f).ordering, TacticalAreaOrdering::DelegateToHost);
}

#[test]
fn host_source_area_uses_host_ordering_without_player_consent_fabrication() {
    let (mut f, _) = fixture("chimera", false);
    f.state
        .rules
        .as_mut()
        .unwrap()
        .tactical_creatures
        .as_mut()
        .unwrap()
        .runtime
        .iter_mut()
        .find(|runtime| runtime.actor == f.actors[0])
        .unwrap()
        .controller = CreatureController::Host;
    begin(&mut f);
    f.rejected(None, action(&f));
    f.run(
        None,
        TacticalAction::CreatureArea {
            feature_id: "fire-breath".into(),
            aim: aim(&f),
            ordering: TacticalAreaOrdering::Host,
        },
    );
    assert_eq!(record(&f).ordering, TacticalAreaOrdering::Host);
    raw(&mut f, &[1; 7]);
    f.roll(1, &[10]);
    assert!(f.flow().resolution.is_none());
}
