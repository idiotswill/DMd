use super::*;
use dmd_rules::tactical_creature_equipment::*;
use dmd_rules::tactical_creatures::*;

fn prepared() -> Fixture {
    let mut f = Fixture::new();
    let origin = f.meta(Some(0));
    let actor = f.actors[0];
    let e = f.entity_mut(0);
    // An imported mechanical caster exercises the same authenticated prepared
    // grant used by the source planner, without inventing a source monster level.
    e.level = 5;
    e.hit_dice.maximum = 5;
    e.hit_dice.remaining = 5;
    e.ability_scores[Ability::Wisdom.index()] = 16;
    e.prepared_spells.insert("fire-bolt".into());
    e.spellcasting = Some(Spellcasting {
        ability: Ability::Wisdom,
        slot_maxima: [2; 9],
        slots: [2; 9],
        can_speak: true,
        free_hand: true,
        material_focus: false,
    });
    f.state.rules.as_mut().unwrap().tactical_inventory = Some(TacticalInventory {
        loadouts: vec![ActorEquipmentLoadout {
            actor,
            hands: WeaponLoadout::default(),
            worn_armor: None,
            shield: None,
            command: origin,
        }],
        ..Default::default()
    });
    f.entity_mut(1).max_hp = 100;
    f.entity_mut(1).hp = 100;
    f.state.applied_event_sequence += 1;
    f
}

fn dragon() -> Fixture {
    let mut f = Fixture::new();
    let actor = f.actors[0];
    f.state.characters.retain(|_, c| c.entity_id != actor);
    f.state.entities.get_mut(&actor).unwrap().kind = EntityKind::Creature;
    f.state.rules.as_mut().unwrap().entities.remove(&actor);
    let origin = f.meta(None);
    let built = build_creature(
        &f.state,
        &origin,
        actor,
        &CreatureBuildChoice {
            definition_id: "adult-red-dragon".into(),
            size: CreatureSize::Huge,
            additional_languages: vec![],
            hit_points: CreatureHitPointChoice::Average,
            controller: CreatureController::Player(f.players[0]),
            in_lair: false,
        },
    )
    .unwrap();
    let rules = f.state.rules.as_mut().unwrap();
    rules.entities.insert(actor, built.mechanics);
    rules.tactical_creatures = Some(TacticalCreatures {
        schema_version: TACTICAL_CREATURES_SCHEMA_VERSION,
        profiles: vec![built.profile],
        runtime: vec![built.runtime],
    });
    let ids = creature_equipment_plan("adult-red-dragon", 0)
        .unwrap()
        .iter()
        .map(|_| ItemId::new())
        .collect::<Vec<_>>();
    f.state = materialize_creature_equipment(&f.state, &origin, actor, 0, &ids, &f.pack).unwrap();
    let e = f.state.encounter.as_mut().unwrap();
    e.participants[0].size = CreatureSize::Huge;
    e.participants[0].height = 30;
    e.participants[0].movement = built.movement;
    e.participants[0].senses = built.senses;
    e.participants[1].position.x = 60;
    f.entity_mut(1).max_hp = 100;
    f.entity_mut(1).hp = 100;
    f.state.applied_event_sequence += 1;
    f
}

fn cast(f: &Fixture, source: bool, targets: Vec<EntityId>) -> TacticalAction {
    TacticalAction::CastSpell {
        choice: SpellCastChoice {
            actor: f.actors[0],
            spell_id: if source { "scorching-ray" } else { "fire-bolt" }.into(),
            grant: if source {
                SpellGrantChoice::CreatureFeature {
                    feature_id: "spellcasting".into(),
                }
            } else {
                SpellGrantChoice::Prepared
            },
            resource: if source {
                SpellResourceChoice::SourceFeature
            } else {
                SpellResourceChoice::Cantrip
            },
            material: SpellMaterialChoice::None,
            mode: SpellCastMode::Immediate,
        },
        targets: SpellTargetChoice::Entities(targets),
    }
}

fn pending_attack(f: &Fixture) -> &TacticalAttack {
    f.flow()
        .resolution
        .as_ref()
        .unwrap()
        .attack
        .as_ref()
        .unwrap()
}

fn focus(f: &mut Fixture) -> EffectId {
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
                        definition_id: "retained-source-focus".into(),
                        actor: f.actors[1],
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
fn spell_attack_critical_uses_source_scaling_raw_faces_and_no_physical_reservation() {
    let mut f = prepared();
    f.begin();
    let before = f.state.clone();
    let slots = f.rules().entities[&f.actors[0]]
        .spellcasting
        .as_ref()
        .unwrap()
        .slots;
    let event = f.run(Some(0), cast(&f, false, vec![f.actors[1]]));
    assert_eq!(f.request().roller, Some(f.actors[0]));
    assert_eq!(f.request().modifier, 6); // Wisdom +3 and level-5 PB +3.
    assert!(matches!(
        pending_attack(&f).source,
        TacticalAttackSource::Spell { .. }
    ));
    assert!(pending_attack(&f).weapon().is_none());
    assert_eq!(pending_attack(&f).origin, event.meta);
    assert!(f.rules().timing.as_ref().unwrap().action_spent);
    assert!(f.flow().budget.weapon_history.is_empty());
    assert_eq!(f.state.items, before.items);
    f.rejected(
        Some(1),
        TacticalAction::SubmitRoll {
            result: f.raw(&[20]),
        },
    );
    f.rejected(
        Some(0),
        TacticalAction::SubmitRoll {
            result: f.raw(&[21]),
        },
    );
    let mut forged_event = event.clone();
    forged_event.meta.actor = Some(AgentRef::Entity(f.actors[1]));
    assert!(replay_tactical(&before, &forged_event, &f.pack).is_err());
    f.roll(0, &[20]);
    assert_eq!(
        f.request().dice,
        vec![DieSpec {
            count: 4,
            sides: 10
        }]
    );
    f.rejected(
        Some(0),
        TacticalAction::SubmitRoll {
            result: f.raw(&[10, 10]),
        },
    );
    f.roll(0, &[10, 10, 10, 10]);
    assert_eq!(f.rules().entities[&f.actors[1]].hp, 60);
    assert!(f.flow().resolution.is_none());
    assert_eq!(f.state.items, before.items);
    assert_eq!(
        f.rules().entities[&f.actors[0]]
            .spellcasting
            .as_ref()
            .unwrap()
            .slots,
        slots
    );
    assert!(f.flow().budget.weapon_history.is_empty());
}

#[test]
fn spell_attack_conditions_exhaustion_and_natural_one_are_source_derived() {
    let mut f = prepared();
    f.entity_mut(0).exhaustion = 1;
    f.entity_mut(0).prone = true;
    f.begin();
    f.run(Some(0), cast(&f, false, vec![f.actors[1]]));
    assert_eq!(f.request().modifier, 4);
    assert_eq!(f.request().mode, RollMode::Disadvantage);
    f.roll(0, &[20, 1]);
    assert_eq!(f.rules().entities[&f.actors[1]].hp, 100);
    assert!(f.rules().pending.is_none());
    assert!(f.flow().resolution.is_none());
}

#[test]
fn source_permitted_self_target_keeps_the_actual_caster_and_completes_damage() {
    let mut f = prepared();
    f.entity_mut(0).max_hp = 100;
    f.entity_mut(0).hp = 100;
    f.begin();
    f.run(Some(0), cast(&f, false, vec![f.actors[0]]));
    assert_eq!(pending_attack(&f).actor, pending_attack(&f).target);
    assert_eq!(f.request().roller, Some(f.actors[0]));
    assert_eq!(f.request().mode, RollMode::Normal);
    f.roll(0, &[15]);
    f.roll(0, &[3, 4]);
    assert_eq!(f.rules().entities[&f.actors[0]].hp, 93);
    assert_eq!(f.rules().entities[&f.actors[1]].hp, 100);
    assert!(f.flow().resolution.is_none());
}

#[test]
fn source_rays_keep_distinct_rolls_and_resume_after_another_players_concentration_save() {
    let mut f = dragon();
    let group = focus(&mut f);
    f.begin();
    let items = f.state.items.clone();
    let event = f.run(Some(0), cast(&f, true, vec![f.actors[1]; 3]));
    let first = f.request().id;
    assert_eq!(f.request().modifier, 12); // Printed dragon spell attack, never level0 PB.
    f.roll(0, &[15]);
    assert_eq!(f.request().dice, vec![DieSpec { count: 2, sides: 6 }]);
    f.roll(0, &[3, 4]);
    assert_eq!(f.rules().entities[&f.actors[1]].hp, 93);
    assert!(f.flow().resolution.as_ref().unwrap().attack.is_none());
    assert_eq!(
        f.flow().resolution.as_ref().unwrap().casts[0].completed,
        vec![SpellProgramOccurrence { node: 0, target: 0 }]
    );
    assert_eq!(f.request().roller, Some(f.actors[1]));
    assert!(
        matches!(f.flow().resolution.as_ref().unwrap().pending.as_ref().unwrap().work.kind,
        TacticalWorkKind::ConcentrationSave { group: actual, .. } if actual == group)
    );
    let saved = f.run(
        Some(1),
        TacticalAction::SubmitRoll {
            result: f.raw(&[1]),
        },
    );
    let second = f.request().id;
    assert_ne!(first, second);
    assert_eq!(pending_attack(&f).origin, saved.meta);
    assert_eq!(f.rules().pending.as_ref().unwrap().issued_by, saved.meta);
    assert_eq!(
        pending_attack(&f).admission,
        TacticalAttackAdmission::Spell {
            casting_origin: event.meta.clone()
        }
    );
    assert_eq!(f.request().roller, Some(f.actors[0]));
    f.roll(0, &[1]); // Missed second ray still completes exactly this occurrence.
    let third = f.request().id;
    assert_ne!(third, second);
    assert_ne!(third, first);
    f.roll(0, &[20]);
    assert_eq!(f.request().dice, vec![DieSpec { count: 4, sides: 6 }]);
    f.roll(0, &[1, 2, 3, 4]);
    assert_eq!(f.rules().entities[&f.actors[1]].hp, 83);
    assert!(f.flow().resolution.is_none());
    assert!(f.rules().timing.as_ref().unwrap().action_spent);
    assert!(!f.rules().timing.as_ref().unwrap().bonus_action_spent);
    assert!(f.flow().budget.weapon_history.is_empty());
    assert_eq!(f.state.items, items);
    let ray_rolls = f
        .rules()
        .rolls
        .iter()
        .filter(|r| {
            matches!(r.purpose,
        PendingPurpose::TacticalResolution { key, .. } if key.role == TacticalRollRole::Attack)
        })
        .collect::<Vec<_>>();
    assert_eq!(ray_rolls.len(), 3);
    for roll in ray_rolls {
        let PendingPurpose::TacticalResolution { key, .. } = roll.purpose else {
            panic!()
        };
        assert_eq!(key.origin, event.meta.id);
    }
}

#[test]
fn later_rays_to_a_target_killed_by_the_first_complete_without_unfinishable_damage() {
    let mut f = dragon();
    f.entity_mut(1).max_hp = 3;
    f.entity_mut(1).hp = 3;
    f.begin();
    f.run(Some(0), cast(&f, true, vec![f.actors[1]; 3]));
    f.roll(0, &[20]);
    f.roll(0, &[6, 6, 6, 6]);
    assert!(f.rules().entities[&f.actors[1]].death.dead);
    assert!(f.rules().pending.is_none());
    assert!(f.flow().resolution.is_none());
    assert_eq!(
        f.rules()
            .rolls
            .iter()
            .filter(|r| matches!(r.purpose,
        PendingPurpose::TacticalResolution { key, .. } if key.role == TacticalRollRole::Attack))
            .count(),
        1
    );
    assert!(f.rules().timing.as_ref().unwrap().action_spent);
}

#[test]
fn restored_spell_attack_rejects_source_ordinals_facts_and_cause_forgeries() {
    let mut f = dragon();
    f.begin();
    f.run(Some(0), cast(&f, true, vec![f.actors[1]; 3]));
    for mutation in 0..11 {
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
        let a = r.attack.as_mut().unwrap();
        match mutation {
            0 => a.attack_modifier += 1,
            1 => a.damage[0].dice[0].count += 1,
            2 => a.armor_class += 1,
            3 => a.mode = RollMode::Disadvantage,
            4 => a.critical_on_hit = !a.critical_on_hit,
            5 => a.admission = TacticalAttackAdmission::OwnTurn,
            6 => a.origin.campaign_id = CampaignId::new(),
            7 => {
                let TacticalAttackSource::Spell { at, .. } = &mut a.source else {
                    panic!()
                };
                at.target = 1;
            }
            8 => {
                let TacticalAttackSource::Spell { cast, .. } = &mut a.source else {
                    panic!()
                };
                *cast += 1;
            }
            9 => {
                let TacticalAttackSource::Spell { source, .. } = &mut a.source else {
                    panic!()
                };
                source.spell_id = "fire-bolt".into();
            }
            10 => r.casts[0]
                .completed
                .push(SpellProgramOccurrence { node: 0, target: 0 }),
            _ => unreachable!(),
        }
        assert!(
            validate_tactical_state(&corrupt).is_err(),
            "spell forgery {mutation}"
        );
    }
    f.roll(0, &[20]);
    let mut corrupt = f.state.clone();
    corrupt
        .rules
        .as_mut()
        .unwrap()
        .pending
        .as_mut()
        .unwrap()
        .request
        .dice[0]
        .count = 2;
    assert!(validate_state(&corrupt, &f.pack).is_err());
}
