use super::*;
use dmd_rules::tactical_defenses::{effective_armor_class, prevents_spell_damage};

fn queried(f: &Fixture) -> CampaignState {
    let mut state = f.campaign.clone();
    state.rules.as_mut().unwrap().tactical_effects = Some(f.effects.clone());
    state
}

fn defense(f: &Fixture, source: &str, clauses: Vec<EffectDefense>) -> TacticalEffect {
    let mut effect = f.effect(f.source(&f.meta(), source), f.target);
    effect.conditions.clear();
    effect.defenses = clauses;
    effect.overlap = Some(EffectOverlap {
        key: source.into(),
        potency: 0,
    });
    effect
}

#[test]
fn timed_source_defenses_do_not_change_equipment_or_stack_identical_spells() {
    let mut f = Fixture::new();
    let original = f.campaign.rules.as_ref().unwrap().entities[&f.target]
        .armor
        .clone();
    let mut armor = defense(
        &f,
        "mage-armor",
        vec![EffectDefense::BaseArmorClass {
            base: 13,
            ability: Ability::Dexterity,
            ends_when_wearing_armor: true,
        }],
    );
    armor.expires = TacticalEffectExpiry::AtTime(WorldInstant(28_800));
    f.run_with(
        &armor.source.command.clone(),
        EffectLifecycleOperation::Install {
            effects: vec![armor],
        },
    );
    for _ in 0..2 {
        let mut shield = defense(
            &f,
            "shield",
            vec![
                EffectDefense::ArmorClassBonus { bonus: 5 },
                EffectDefense::PreventSpellDamage {
                    spell_id: "magic-missile".into(),
                },
            ],
        );
        shield.expires = TacticalEffectExpiry::AtTime(WorldInstant(6));
        f.run_with(
            &shield.source.command.clone(),
            EffectLifecycleOperation::Install {
                effects: vec![shield],
            },
        );
    }
    let state = queried(&f);
    let before = serde_json::to_vec(&state).unwrap();
    assert_eq!(effective_armor_class(&state, f.target).unwrap(), 18);
    assert!(prevents_spell_damage(&state, f.target, "magic-missile"));
    assert!(!prevents_spell_damage(&state, f.target, "fire-bolt"));
    assert_eq!(
        state.rules.as_ref().unwrap().entities[&f.target].armor,
        original
    );
    assert_eq!(serde_json::to_vec(&state).unwrap(), before);
    f.campaign.clock.now = WorldInstant(6);
    let due = f.observe(EffectObservation::Time);
    assert_eq!(due.triggers.len(), 2);
    for ticket in due.triggers {
        f.resolve(ticket.id, EffectTriggerResolution::Apply);
    }
    assert_eq!(effective_armor_class(&queried(&f), f.target).unwrap(), 13);
    assert!(!prevents_spell_damage(
        &queried(&f),
        f.target,
        "magic-missile"
    ));
    f.campaign.clock.now = WorldInstant(28_800);
    let due = f.observe(EffectObservation::Time);
    assert_eq!(due.triggers.len(), 1);
    f.resolve(due.triggers[0].id, EffectTriggerResolution::Apply);
    assert_eq!(effective_armor_class(&queried(&f), f.target).unwrap(), 10);
}

#[test]
fn mage_armor_ending_observation_requires_actual_armor_and_cannot_reactivate_after_doffing() {
    let mut f = Fixture::new();
    let mut armor = defense(
        &f,
        "mage-armor",
        vec![EffectDefense::BaseArmorClass {
            base: 13,
            ability: Ability::Dexterity,
            ends_when_wearing_armor: true,
        }],
    );
    armor.triggers.push(EffectTriggerRule {
        event: EffectTriggerEvent::ArmorWorn {
            subject: EffectSubject::Target,
        },
        frequency: EffectTriggerFrequency::EveryOccurrence,
        payload: EffectTriggerPayload::EndTargetEffect,
    });
    f.run_with(
        &armor.source.command.clone(),
        EffectLifecycleOperation::Install {
            effects: vec![armor],
        },
    );
    let operation = EffectLifecycleAction {
        step: 0,
        operation: EffectLifecycleOperation::Observe(EffectObservation::ArmorWorn {
            target: f.target,
        }),
    };
    assert!(apply_effect_lifecycle(&f.campaign, &f.effects, &f.meta(), &operation).is_err());
    // The fixture represents the authoritative equipment transition immediately
    // before its lifecycle observation; it does not claim a public don-armor action.
    let meta = f.meta();
    f.campaign.rules.as_mut().unwrap().tactical_inventory = Some(TacticalInventory {
        loadouts: vec![ActorEquipmentLoadout {
            actor: f.target,
            hands: WeaponLoadout::default(),
            worn_armor: Some(ItemId::new()),
            shield: None,
            command: meta,
        }],
        ..TacticalInventory::default()
    });
    f.campaign
        .rules
        .as_mut()
        .unwrap()
        .entities
        .get_mut(&f.target)
        .unwrap()
        .armor = ArmorClass::Fixed(11);
    assert_eq!(effective_armor_class(&queried(&f), f.target).unwrap(), 11);
    let due = f.observe(EffectObservation::ArmorWorn { target: f.target });
    assert_eq!(due.triggers.len(), 1);
    f.resolve(due.triggers[0].id, EffectTriggerResolution::Apply);
    f.campaign
        .rules
        .as_mut()
        .unwrap()
        .tactical_inventory
        .as_mut()
        .unwrap()
        .loadouts[0]
        .worn_armor = None;
    f.campaign
        .rules
        .as_mut()
        .unwrap()
        .entities
        .get_mut(&f.target)
        .unwrap()
        .armor = ArmorClass::Fixed(10);
    assert_eq!(effective_armor_class(&queried(&f), f.target).unwrap(), 10);
    assert!(f.effects.effects.is_empty());
}

#[test]
fn old_effect_wire_omits_empty_defenses_and_rejects_duplicate_defense_clauses() {
    let mut f = Fixture::new();
    let plain = f.effect(f.source(&f.meta(), "existing"), f.target);
    let encoded = serde_json::to_value(&plain).unwrap();
    assert!(encoded.get("defenses").is_none());
    assert_eq!(
        serde_json::from_value::<TacticalEffect>(encoded).unwrap(),
        plain
    );
    let duplicate = defense(
        &f,
        "shield",
        vec![EffectDefense::ArmorClassBonus { bonus: 5 }; 2],
    );
    let before = f.effects.clone();
    assert!(
        apply_effect_lifecycle(
            &f.campaign,
            &f.effects,
            &duplicate.source.command.clone(),
            &EffectLifecycleAction {
                step: 0,
                operation: EffectLifecycleOperation::Install {
                    effects: vec![duplicate]
                },
            }
        )
        .is_err()
    );
    assert_eq!(f.effects, before);
    // A genuine owner boundary, rather than any other actor's start, ends Shield.
    let mut shield = defense(
        &f,
        "shield",
        vec![EffectDefense::ArmorClassBonus { bonus: 5 }],
    );
    shield.expires = TacticalEffectExpiry::AfterOwnerBoundaries {
        owner: f.target,
        boundary: TurnBoundary::Start,
        remaining: 1,
    };
    f.run_with(
        &shield.source.command.clone(),
        EffectLifecycleOperation::Install {
            effects: vec![shield],
        },
    );
    assert!(f.turn(f.other, 1, TurnBoundary::Start).triggers.is_empty());
    assert!(f.turn(f.other, 1, TurnBoundary::End).triggers.is_empty());
    let due = f.turn(f.target, 2, TurnBoundary::Start);
    assert_eq!(due.triggers.len(), 1);
    f.resolve(due.triggers[0].id, EffectTriggerResolution::Apply);
    assert_eq!(effective_armor_class(&queried(&f), f.target).unwrap(), 10);
}
