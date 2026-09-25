use super::*;
use dmd_rules::tactical_creature_equipment::*;
use dmd_rules::tactical_creatures::*;

fn imported_healer() -> Fixture {
    let mut f = Fixture::new();
    let entity = f.entity_mut(0);
    entity.ability_scores[Ability::Wisdom.index()] = 16;
    entity.prepared_spells.insert("cure-wounds".into());
    entity.spellcasting = Some(Spellcasting {
        ability: Ability::Wisdom,
        slot_maxima: [2, 0, 0, 0, 0, 0, 0, 0, 0],
        slots: [2, 0, 0, 0, 0, 0, 0, 0, 0],
        can_speak: true,
        free_hand: true,
        material_focus: false,
    });
    let loadout = ActorEquipmentLoadout {
        actor: f.actors[0],
        hands: WeaponLoadout::default(),
        worn_armor: None,
        shield: None,
        command: f.meta(Some(0)),
    };
    f.state.rules.as_mut().unwrap().tactical_inventory = Some(TacticalInventory {
        loadouts: vec![loadout],
        ..TacticalInventory::default()
    });
    f.state.encounter.as_mut().unwrap().participants[1]
        .position
        .x = 20;
    f.entity_mut(1).max_hp = 20;
    f.entity_mut(1).hp = 2;
    f.state.applied_event_sequence += 1;
    f.begin();
    f
}

fn heal(f: &Fixture) -> TacticalAction {
    TacticalAction::CastSpell {
        choice: SpellCastChoice {
            actor: f.actors[0],
            spell_id: "cure-wounds".into(),
            grant: SpellGrantChoice::Prepared,
            resource: SpellResourceChoice::Slot { level: 1 },
            material: SpellMaterialChoice::None,
            mode: SpellCastMode::Immediate,
        },
        targets: SpellTargetChoice::Entities(vec![f.actors[1]]),
    }
}

#[test]
fn healing_pays_once_then_resumes_caster_dice_without_changing_target_control() {
    let mut f = imported_healer();
    let accepted = f.run(Some(0), heal(&f));
    assert_eq!(f.request().dice, [DieSpec { count: 2, sides: 8 }]);
    assert_eq!(f.request().roller, Some(f.actors[0]));
    assert_eq!(f.request().modifier, 3);
    assert!(f.rules().timing.as_ref().unwrap().action_spent);
    assert_eq!(
        f.rules().entities[&f.actors[0]]
            .spellcasting
            .as_ref()
            .unwrap()
            .slots[0],
        1
    );
    let resolution = f.flow().resolution.as_ref().unwrap();
    assert_eq!(resolution.casts.len(), 1);
    assert_eq!(resolution.casts[0].cast.plan.origin, accepted.meta);
    let raw = f.raw(&[2, 4]);
    f.rejected(
        Some(1),
        TacticalAction::SubmitRoll {
            result: raw.clone(),
        },
    );
    f.run(
        Some(0),
        TacticalAction::SubmitRoll {
            result: raw.clone(),
        },
    );
    assert_eq!(f.rules().entities[&f.actors[1]].hp, 11);
    assert!(f.flow().resolution.is_none());
    assert_eq!(
        f.rules().entities[&f.actors[0]]
            .spellcasting
            .as_ref()
            .unwrap()
            .slots[0],
        1
    );
    f.rejected(Some(0), TacticalAction::SubmitRoll { result: raw });
}

#[test]
fn casting_rejects_range_ready_and_wrong_source_before_payment() {
    let f = imported_healer();
    let mut action = heal(&f);
    if let TacticalAction::CastSpell { choice, .. } = &mut action {
        choice.mode = SpellCastMode::Ready {
            trigger: "The gate opens".into(),
        };
    }
    f.rejected(Some(0), action);
    let mut remote = f;
    remote.state.encounter.as_mut().unwrap().participants[1]
        .position
        .x = 70;
    remote.rejected(Some(0), heal(&remote));
    let mut wrong = heal(&remote);
    if let TacticalAction::CastSpell { choice, .. } = &mut wrong {
        choice.grant = SpellGrantChoice::CreatureFeature {
            feature_id: "spellcasting".into(),
        };
        choice.resource = SpellResourceChoice::SourceFeature;
    }
    remote.rejected(Some(0), wrong);
    assert_eq!(
        remote.rules().entities[&remote.actors[0]]
            .spellcasting
            .as_ref()
            .unwrap()
            .slots[0],
        2
    );
    assert!(!remote.rules().timing.as_ref().unwrap().action_spent);
}

#[test]
fn imported_prepared_spell_outside_the_pinned_kernel_is_rejected_unchanged() {
    let mut f = imported_healer();
    // Tactical content is not a new grant or a replacement kernel pack. Even an
    // imported prepared list must pass the immutable kernel's source validator.
    f.entity_mut(0)
        .prepared_spells
        .insert("magic-missile".into());
    let action = TacticalAction::CastSpell {
        choice: SpellCastChoice {
            actor: f.actors[0],
            spell_id: "magic-missile".into(),
            grant: SpellGrantChoice::Prepared,
            resource: SpellResourceChoice::Slot { level: 1 },
            material: SpellMaterialChoice::None,
            mode: SpellCastMode::Immediate,
        },
        targets: SpellTargetChoice::Entities(vec![f.actors[1]; 3]),
    };
    f.rejected(Some(0), action);
    assert_eq!(
        f.rules().entities[&f.actors[0]]
            .spellcasting
            .as_ref()
            .unwrap()
            .slots[0],
        2
    );
    assert!(!f.rules().timing.as_ref().unwrap().action_spent);
}

#[test]
fn retained_spell_partition_rejects_omitted_duplicate_and_foreign_work() {
    let mut f = imported_healer();
    f.run(Some(0), heal(&f));
    for tamper in 0..5 {
        let mut corrupt = f.state.clone();
        let r = corrupt
            .encounter
            .as_mut()
            .unwrap()
            .flow
            .as_mut()
            .unwrap()
            .resolution
            .as_mut()
            .unwrap();
        match tamper {
            0 => r.frames.clear(),
            1 => r.casts[0]
                .completed
                .push(SpellProgramOccurrence { node: 0, target: 0 }),
            2 => r.casts[0].cast.plan.occurrence = r.next_occurrence,
            3 => r.casts[0].cast.plan.origin.actor = Some(AgentRef::Entity(f.actors[1])),
            _ => {
                // Canonical equal-head metadata still cannot invent a different
                // accepted cast command from the actual resolution origin.
                let cast = &mut r.casts[0].cast;
                cast.plan.origin.id = CommandId::new();
                cast.last_operation = cast.plan.origin.clone();
                dmd_rules::tactical_spells::validate_spell_cast(cast).unwrap();
            }
        }
        assert!(validate_tactical_state(&corrupt).is_err());
    }
    let mut lost = f.state.clone();
    lost.encounter
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .resolution
        .as_mut()
        .unwrap()
        .casts
        .clear();
    assert!(validate_tactical_state(&lost).is_err());
}

fn fanatic() -> (Fixture, SpellCastChoice) {
    let mut f = Fixture::new();
    f.arm("club", false, false); // Genuine Human target with its source profile.
    let actor = f.actors[1];
    // The generic weapon fixture makes this second actor the lender. This case
    // creates a new source creature instead; the Human owns its existing club.
    for item in f.state.items.values_mut() {
        if item.owner == Ownership::Entity(actor) {
            item.owner = Ownership::Entity(f.actors[0]);
        }
    }
    f.state
        .characters
        .retain(|_, character| character.entity_id != actor);
    f.state.entities.get_mut(&actor).unwrap().kind = EntityKind::Creature;
    f.state.rules.as_mut().unwrap().entities.remove(&actor);
    let meta = f.meta(None);
    let built = build_creature(
        &f.state,
        &meta,
        actor,
        &CreatureBuildChoice {
            definition_id: "cultist-fanatic".into(),
            size: CreatureSize::Medium,
            additional_languages: vec![],
            hit_points: CreatureHitPointChoice::Average,
            controller: CreatureController::Player(f.players[1]),
            in_lair: false,
        },
    )
    .unwrap();
    let participant = &mut f.state.encounter.as_mut().unwrap().participants[1];
    participant.size = CreatureSize::Medium;
    participant.height = 12;
    participant.movement = built.movement;
    participant.senses = built.senses;
    f.state
        .rules
        .as_mut()
        .unwrap()
        .entities
        .insert(actor, built.mechanics);
    f.state.rules.as_mut().unwrap().tactical_creatures = Some(TacticalCreatures {
        schema_version: TACTICAL_CREATURES_SCHEMA_VERSION,
        profiles: vec![built.profile],
        runtime: vec![built.runtime],
    });
    let allocations = creature_equipment_plan("cultist-fanatic", 0).unwrap();
    let ids = allocations
        .iter()
        .map(|_| ItemId::new())
        .collect::<Vec<_>>();
    f.state = materialize_creature_equipment(&f.state, &meta, actor, 0, &ids, &f.pack).unwrap();
    let item = f
        .state
        .items
        .values()
        .find(|item| item.definition_id == "spell-material:hold-person")
        .unwrap()
        .id;
    f.state.applied_event_sequence += 1;
    f.begin();
    f.run(Some(0), TacticalAction::EndTurn);
    (
        f,
        SpellCastChoice {
            actor,
            spell_id: "hold-person".into(),
            grant: SpellGrantChoice::CreatureFeature {
                feature_id: "spellcasting".into(),
            },
            resource: SpellResourceChoice::SourceFeature,
            material: SpellMaterialChoice::Material { item },
            mode: SpellCastMode::Immediate,
        },
    )
}

#[test]
fn source_fanatic_hold_uses_real_material_and_counter_then_repeats_target_end_save() {
    let (mut f, choice) = fanatic();
    let action = TacticalAction::CastSpell {
        choice: choice.clone(),
        targets: SpellTargetChoice::Entities(vec![f.actors[0]]),
    };
    let mut wrong = action.clone();
    if let TacticalAction::CastSpell { choice, .. } = &mut wrong {
        choice.material = SpellMaterialChoice::None;
    }
    f.rejected(Some(1), wrong);
    f.run(Some(1), action.clone());
    assert!(f.rules().entities[&f.actors[1]].spellcasting.is_none());
    assert!(f.rules().timing.as_ref().unwrap().action_spent);
    assert_eq!(f.request().roller, Some(f.actors[0]));
    let group = f.rules().entities[&f.actors[1]].concentration.unwrap();
    f.run(Some(0), TacticalAction::VoluntarilyFailSave);
    assert!(active_conditions(f.rules(), f.actors[0]).contains(&Condition::Paralyzed));
    assert_eq!(f.rules().entities[&f.actors[1]].concentration, Some(group));
    assert!(f.flow().resolution.is_none());
    f.run(Some(1), TacticalAction::EndTurn);
    f.run(Some(0), TacticalAction::EndTurn);
    assert_eq!(f.request().roller, Some(f.actors[0]));
    f.roll(0, &[20]);
    assert!(!active_conditions(f.rules(), f.actors[0]).contains(&Condition::Paralyzed));
    assert_eq!(f.rules().entities[&f.actors[1]].concentration, None);
    f.rejected(Some(1), action); // Its one source use remains spent after condition ends.
}

#[test]
fn spell_save_extremes_follow_only_the_explicit_table_policy_and_preserve_raw_twenty() {
    for house in [false, true] {
        let (mut f, choice) = fanatic();
        f.entity_mut(0).exhaustion = 5;
        f.state
            .rules
            .as_mut()
            .unwrap()
            .house_rules
            .ability_test_natural_extremes = house;
        f.state
            .table
            .as_mut()
            .unwrap()
            .contract
            .house_rules
            .ability_test_natural_extremes = house;
        f.run(
            Some(1),
            TacticalAction::CastSpell {
                choice,
                targets: SpellTargetChoice::Entities(vec![f.actors[0]]),
            },
        );
        assert_eq!(f.request().modifier, -10);
        assert_eq!(
            matches!(
                f.rules().pending.as_ref().unwrap().ruling.basis,
                RulingBasis::HouseRule { .. }
            ),
            house
        );
        f.roll(0, &[20]); // Total10 fails the source DC12 under ordinary rules.
        assert_eq!(
            active_conditions(f.rules(), f.actors[0]).contains(&Condition::Paralyzed),
            !house
        );
        let raw = f.rules().rolls.last().unwrap();
        assert_eq!(raw.result.dice[0].value, 20);
        assert_eq!(raw.resolved.total, 10);
    }
}

#[test]
fn canonical_creature_spell_cannot_be_reassigned_to_an_actor_without_its_source() {
    let (mut f, choice) = fanatic();
    f.run(
        Some(1),
        TacticalAction::CastSpell {
            choice,
            targets: SpellTargetChoice::Entities(vec![f.actors[0]]),
        },
    );
    let mut corrupt = f.state.clone();
    let record = &mut corrupt
        .encounter
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .resolution
        .as_mut()
        .unwrap()
        .casts[0];
    let original = &mut record.cast.plan.origin;
    original.actor = Some(AgentRef::Entity(f.actors[0]));
    original.issuer = CommandIssuer::Player(f.players[0]);
    record.cast.last_operation = original.clone();
    record.creature_activation.as_mut().unwrap().origin = original.clone();
    record.cast.plan.choice.actor = f.actors[0];
    record.cast.plan.concentration_group = Some(spell_concentration_id(
        original.id,
        f.actors[0],
        record.cast.plan.occurrence,
    ));
    // The standalone record remains canonical: its program really is Hold Person
    // from the Cultist source. The attached Human actor has no such source grant.
    dmd_rules::tactical_spells::validate_retained_spell(record).unwrap();
    let error = validate_tactical_state(&corrupt).unwrap_err().to_string();
    assert!(
        error.contains("retained source caster profile is absent"),
        "{error}"
    );
}

#[test]
fn accepted_leveled_cast_interrupts_source_rest_but_cantrip_and_rejection_preserve_it() {
    use dmd_rules::tactical_damage::{VitalityContext, VitalityDefenses, reduce_vitality};
    for cantrip in [false, true] {
        let mut f = imported_healer();
        let actor = f.actors[0];
        f.entity_mut(0).prepared_spells.insert("fire-bolt".into());
        let mut context = VitalityContext {
            origin: VitalityOrigin {
                command: f.meta(None),
                occurrence: 0,
            },
            now: f.state.clock.now,
            conditions: Default::default(),
            underwater: false,
            defenses: VitalityDefenses::default(),
            death_save_mode: RollMode::Normal,
            death_save_bonus: 0,
        };
        let knocked = reduce_vitality(
            &f.rules().entities[&actor],
            &TacticalRecovery::default(),
            &context,
            &VitalityOperation::Damage {
                packet: DamagePacket {
                    cause: DamageCause::Attack {
                        attacker: f.actors[1],
                        melee: true,
                        critical: false,
                    },
                    components: vec![DamageComponent {
                        damage_type: DamageType::Bludgeoning,
                        amounts: vec![100],
                        adjustments: vec![],
                    }],
                },
                knockout: Some(KnockoutChoice::KnockOut),
            },
        )
        .unwrap();
        f.state.applied_event_sequence += 1;
        context.origin.command = f.meta(None);
        let awake = reduce_vitality(
            &knocked.entity,
            &knocked.recovery,
            &context,
            &VitalityOperation::Heal { amount: 1 },
        )
        .unwrap();
        let rules = f.state.rules.as_mut().unwrap();
        rules.entities.insert(actor, awake.entity);
        rules
            .tactical_recovery
            .get_or_insert_default()
            .insert(actor, awake.recovery);
        rules.rests.push(RestProgress {
            actor,
            kind: RestKind::Short,
            started_at: f.state.clock.now,
        });
        f.state.applied_event_sequence += 1;
        validate_state(&f.state, &f.pack).unwrap();
        let mut action = heal(&f);
        if let TacticalAction::CastSpell { choice, .. } = &mut action
            && cantrip
        {
            choice.spell_id = "fire-bolt".into();
            choice.resource = SpellResourceChoice::Cantrip;
        }
        let mut rejected = action.clone();
        if let TacticalAction::CastSpell { choice, .. } = &mut rejected {
            choice.mode = SpellCastMode::Ready {
                trigger: "The gate opens".into(),
            };
        }
        f.rejected(Some(0), rejected);
        assert!(f.rules().rests.iter().any(|rest| rest.actor == actor));
        f.run(Some(0), action);
        assert_eq!(
            f.rules().rests.iter().any(|rest| rest.actor == actor),
            cantrip
        );
        assert_eq!(
            f.rules().tactical_recovery.as_ref().unwrap()[&actor]
                .knockout_rest
                .is_some(),
            cantrip
        );
        f.roll(0, if cantrip { &[1, 1] } else { &[2, 3] });
        assert!(f.flow().resolution.is_none());
    }
}
