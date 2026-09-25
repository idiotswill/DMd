//! SRD5.2.1 Shield161–162 and Mage305, exercised through the public tactical reducer.
use super::*;
use dmd_rules::tactical_creature_equipment::*;
use dmd_rules::tactical_creatures::*;
use dmd_rules::tactical_defenses::effective_armor_class;

/// An isolated source-built initial image, not a claimed application creation path.
/// All response, cost, roll, turn and movement changes below are real commands.
fn source_actor(f: &mut Fixture, index: usize, definition: &str, size: CreatureSize) {
    let actor = f.actors[index];
    // The generic weapon fixture labels its attacker's loot as borrowed from
    // actor 1. This source-built initial image gives that loot to its current
    // carrier before replacing actor 1; source creation must have no old gear.
    for item in f.state.items.values_mut() {
        if item.owner == Ownership::Entity(actor) {
            let Custody::Entity(carrier) = item.custody else {
                panic!("source replacement cannot inherit another initial item");
            };
            assert_ne!(carrier, actor);
            item.owner = Ownership::Entity(carrier);
        }
    }
    f.state.characters.retain(|_, c| c.entity_id != actor);
    f.state.entities.get_mut(&actor).unwrap().kind = EntityKind::Creature;
    f.state.rules.as_mut().unwrap().entities.remove(&actor);
    let origin = f.meta(None);
    let built = build_creature(
        &f.state,
        &origin,
        actor,
        &CreatureBuildChoice {
            definition_id: definition.into(),
            size,
            additional_languages: vec![],
            hit_points: CreatureHitPointChoice::Average,
            controller: CreatureController::Player(f.players[index]),
            in_lair: false,
        },
    )
    .unwrap();
    let rules = f.state.rules.as_mut().unwrap();
    rules.entities.insert(actor, built.mechanics);
    let creatures = rules.tactical_creatures.get_or_insert_default();
    creatures.profiles.push(built.profile);
    creatures.runtime.push(built.runtime);
    let ids = creature_equipment_plan(definition, 0)
        .unwrap()
        .iter()
        .map(|_| ItemId::new())
        .collect::<Vec<_>>();
    f.state = materialize_creature_equipment(&f.state, &origin, actor, 0, &ids, &f.pack).unwrap();
    let participant = &mut f.state.encounter.as_mut().unwrap().participants[index];
    participant.size = size;
    participant.height = size.footprint_units() as u32;
    participant.movement = built.movement;
    participant.senses = built.senses;
    f.state.applied_event_sequence += 1;
}

fn mage() -> (Fixture, WeaponUseChoice) {
    let mut f = Fixture::new();
    let choice = f.arm("longsword", false, false);
    source_actor(&mut f, 1, "mage", CreatureSize::Medium);
    assert_eq!(effective_armor_class(&f.state, f.actors[1]).unwrap(), 12);
    assert!(f.rules().entities[&f.actors[1]].prepared_spells.is_empty());
    assert!(f.rules().entities[&f.actors[1]].spellcasting.is_none());
    (f, choice)
}

fn resolution(f: &Fixture) -> &TacticalResolution {
    f.flow().resolution.as_deref().unwrap()
}
fn hit(f: &Fixture) -> &TacticalHitReview {
    resolution(f).hit_review.as_deref().unwrap()
}
fn hit_window(f: &Fixture) -> TacticalWorkKey {
    TacticalWorkKey {
        resolution: resolution(f).origin.id,
        occurrence: hit(f).work.occurrence,
    }
}
fn shield(f: &Fixture) -> SpellCastChoice {
    let choices = shield_choices(&f.state, f.actors[1]).unwrap();
    assert_eq!(choices.len(), 1);
    assert_eq!(choices[0].spell_id, "shield");
    assert_eq!(
        choices[0].grant,
        SpellGrantChoice::CreatureFeature {
            feature_id: "protective-magic".into(),
        }
    );
    assert_eq!(choices[0].resource, SpellResourceChoice::SourceFeature);
    choices[0].clone()
}
fn uses(f: &Fixture) -> u8 {
    f.rules()
        .tactical_creatures
        .as_ref()
        .unwrap()
        .runtime
        .iter()
        .find(|r| r.actor == f.actors[1])
        .unwrap()
        .limited_uses
        .iter()
        .filter(|u| u.feature_id == "protective-magic")
        .map(|u| {
            assert!(u.spell_id.is_none());
            u.spent
        })
        .sum()
}
fn select(f: &mut Fixture, intent_first: bool) -> TacticalWorkKey {
    let window = hit_window(f);
    let order_owner =
        hit_responses::owner_index(&f.state, &f.actors, &f.players, resolution(f).turn_actor);
    let order = TacticalAction::OrderHitResponses {
        window,
        instruction: hit_responses::forward_order(),
    };
    let respond = TacticalAction::RespondToHit {
        window,
        accept: true,
    };
    if intent_first {
        f.run(Some(1), respond);
        assert_eq!(hit(f).stage, TacticalHitReviewStage::Collecting);
        f.run(order_owner, order);
    } else {
        f.run(order_owner, order);
        assert_eq!(hit(f).stage, TacticalHitReviewStage::Collecting);
        f.run(Some(1), respond);
    }
    assert_eq!(hit(f).stage, TacticalHitReviewStage::Selected);
    window
}
fn cast(f: &mut Fixture, window: TacticalWorkKey) -> TacticalEvent {
    let choice = shield(f);
    f.run(Some(1), TacticalAction::CastHitShield { window, choice })
}
fn attack_hit(f: &mut Fixture, choice: &WeaponUseChoice, face: u16) -> TacticalEvent {
    f.run(
        Some(0),
        TacticalAction::Attack {
            choice: choice.clone(),
        },
    );
    let result = f.raw(&[face]);
    let accepted = f.run(Some(0), TacticalAction::SubmitRoll { result });
    assert_eq!(hit(f).cause, accepted.meta);
    assert_eq!(hit(f).stage, TacticalHitReviewStage::Collecting);
    assert!(f.rules().pending.is_none());
    accepted
}
fn next_round(f: &mut Fixture) {
    f.run(Some(0), TacticalAction::EndTurn);
    assert_eq!(effective_armor_class(&f.state, f.actors[1]).unwrap(), 12);
    assert!(
        !f.rules()
            .timing
            .as_ref()
            .unwrap()
            .reactions_spent
            .contains(&f.actors[1])
    );
    f.run(Some(1), TacticalAction::EndTurn);
}

#[test]
fn genuine_mage_shield_changes_hit_to_miss_in_either_arrival_order_without_rewriting_raw_facts() {
    for intent_first in [false, true] {
        let (mut f, choice) = mage();
        f.begin();
        let hp = f.rules().entities[&f.actors[1]].hp;
        let accepted = attack_hit(&mut f, &choice, 8); // 8+5 hits12, misses17.
        let raw = f.rules().rolls.last().unwrap().clone();
        let weapon = resolution(&f).attack.as_ref().unwrap().clone();
        let window = select(&mut f, intent_first);
        assert_eq!(uses(&f), 0); // Intent and ordering spend neither resource.
        assert!(
            f.rules()
                .timing
                .as_ref()
                .unwrap()
                .reactions_spent
                .is_empty()
        );
        assert_eq!(resolution(&f).attack.as_ref(), Some(&weapon));
        let casting = cast(&mut f, window);
        assert_eq!(f.rules().rolls.last(), Some(&raw));
        assert_eq!(raw.accepted_by, accepted.meta);
        assert_eq!(f.rules().entities[&f.actors[1]].hp, hp);
        assert_eq!(
            f.flow().budget.weapon_history[0].outcome,
            WeaponAttackOutcome::Miss
        );
        assert!(f.flow().resolution.is_none());
        assert!(f.rules().pending.is_none());
        assert_eq!(uses(&f), 1);
        assert!(
            f.rules()
                .timing
                .as_ref()
                .unwrap()
                .reactions_spent
                .contains(&f.actors[1])
        );
        assert_eq!(effective_armor_class(&f.state, f.actors[1]).unwrap(), 17);
        let effects = &f.rules().tactical_effects.as_ref().unwrap().effects;
        assert_eq!(effects.len(), 1);
        assert_eq!(effects[0].source.command, casting.meta);
        assert_eq!(
            effects[0].expires,
            TacticalEffectExpiry::AfterOwnerBoundaries {
                owner: f.actors[1],
                boundary: TurnBoundary::Start,
                remaining: 1,
            }
        );
        f.rejected(
            Some(1),
            TacticalAction::CastHitShield {
                window,
                choice: SpellCastChoice {
                    actor: f.actors[1],
                    spell_id: "shield".into(),
                    grant: SpellGrantChoice::CreatureFeature {
                        feature_id: "protective-magic".into(),
                    },
                    resource: SpellResourceChoice::SourceFeature,
                    material: SpellMaterialChoice::None,
                    mode: SpellCastMode::Immediate,
                },
            },
        );
        next_round(&mut f);
        assert_eq!(uses(&f), 1); // An own turn restores Reaction, never shared daily uses.
    }
}

#[test]
fn natural_twenty_still_hits_shield_and_damage_request_keeps_original_attack_acceptance_cause() {
    let (mut f, choice) = mage();
    f.begin();
    let hp = f.rules().entities[&f.actors[1]].hp;
    let accepted = attack_hit(&mut f, &choice, 20);
    let original = resolution(&f).attack.as_ref().unwrap().clone();
    let window = select(&mut f, false);
    let casting = cast(&mut f, window);
    let pending = f.rules().pending.as_ref().unwrap();
    assert_eq!(pending.request.dice, [DieSpec { count: 2, sides: 8 }]);
    assert_eq!(pending.issued_by, accepted.meta);
    assert_ne!(pending.issued_by, casting.meta);
    assert_eq!(pending.request.roller, Some(f.actors[0]));
    let retained = resolution(&f).attack.as_ref().unwrap();
    assert_eq!(retained.armor_class, original.armor_class);
    assert_eq!(retained.damage, original.damage);
    assert_eq!(retained.mode, original.mode);
    assert_eq!(retained.attack_modifier, original.attack_modifier);
    assert_eq!(
        retained.outcome,
        Some(WeaponAttackOutcome::Hit {
            critical: true,
            damage_dealt: 0
        })
    );
    assert!(hit(&f).completed_shield.is_some());
    let cause = f.rules().rolls.last().unwrap().clone();
    f.rejected(
        Some(1),
        TacticalAction::SubmitRoll {
            result: f.raw(&[1, 1]),
        },
    );
    f.roll(0, &[1, 1]);
    assert_eq!(f.rules().entities[&f.actors[1]].hp, hp - 5);
    assert_eq!(f.rules().rolls[f.rules().rolls.len() - 2], cause);
    assert_eq!(f.rules().rolls.last().unwrap().issued_by, accepted.meta);
    assert_eq!(
        f.rules().rolls.last().unwrap().accepted_by.issuer,
        CommandIssuer::Player(f.players[0])
    );
    assert!(f.flow().resolution.is_none());
}

#[test]
fn shield_defeated_hit_retains_the_attackers_optional_graze_without_new_damage_dice() {
    for accept in [false, true] {
        let mut f = Fixture::new();
        let choice = f.arm("greatsword", true, false);
        source_actor(&mut f, 1, "mage", CreatureSize::Medium);
        f.begin();
        let hp = f.rules().entities[&f.actors[1]].hp;
        attack_hit(&mut f, &choice, 8);
        let original = f.rules().rolls.last().unwrap().clone();
        let window = select(&mut f, false);
        cast(&mut f, window);
        assert_eq!(
            resolution(&f).attack.as_ref().unwrap().stage,
            TacticalAttackStage::MasteryChoice
        );
        assert_eq!(hit(&f).stage, TacticalHitReviewStage::Resolved);
        assert!(f.rules().pending.is_none());
        assert_eq!(f.rules().entities[&f.actors[1]].hp, hp);
        f.rejected(
            Some(1),
            TacticalAction::ChooseAttackMastery {
                choice: WeaponMasteryChoice::Graze,
            },
        );
        f.run(
            Some(0),
            TacticalAction::ChooseAttackMastery {
                choice: if accept {
                    WeaponMasteryChoice::Graze
                } else {
                    WeaponMasteryChoice::Decline
                },
            },
        );
        assert_eq!(
            f.rules().entities[&f.actors[1]].hp,
            hp - if accept { 3 } else { 0 }
        );
        assert_eq!(f.rules().rolls.last(), Some(&original));
        assert_eq!(uses(&f), 1);
        assert!(f.flow().resolution.is_none());
    }
}

#[test]
fn uniform_acknowledgment_is_owned_even_without_a_spell_and_no_other_actor_can_order() {
    for available in [false, true] {
        let (mut f, choice) = if available {
            mage()
        } else {
            let mut f = Fixture::new();
            let choice = f.arm("longsword", false, false);
            (f, choice)
        };
        f.begin();
        attack_hit(&mut f, &choice, 15);
        let window = hit_window(&f);
        assert_eq!(
            shield_choices(&f.state, f.actors[1]).unwrap().is_empty(),
            !available
        );
        assert_eq!(hit(&f).respondent.as_ref().unwrap().actor, f.actors[1]);
        for owner in [None, Some(0)] {
            f.rejected(
                owner,
                TacticalAction::RespondToHit {
                    window,
                    accept: false,
                },
            );
        }
        for owner in [None, Some(1)] {
            f.rejected(
                owner,
                TacticalAction::OrderHitResponses {
                    window,
                    instruction: hit_responses::forward_order(),
                },
            );
        }
        let stale = TacticalWorkKey {
            occurrence: window.occurrence + 1,
            ..window
        };
        f.rejected(
            Some(1),
            TacticalAction::RespondToHit {
                window: stale,
                accept: false,
            },
        );
        if !available {
            f.rejected(
                Some(1),
                TacticalAction::RespondToHit {
                    window,
                    accept: true,
                },
            );
        }
        f.run(
            Some(0),
            TacticalAction::OrderHitResponses {
                window,
                instruction: hit_responses::forward_order(),
            },
        );
        assert!(f.rules().pending.is_none()); // Ordering alone never acknowledges for target.
        f.run(
            Some(1),
            TacticalAction::RespondToHit {
                window,
                accept: false,
            },
        );
        assert_eq!(f.request().dice, [DieSpec { count: 1, sides: 8 }]);
        assert!(
            f.rules()
                .timing
                .as_ref()
                .unwrap()
                .reactions_spent
                .is_empty()
        );
        f.roll(0, &[1]);
    }
}

#[test]
fn selected_controller_can_decline_without_payment_and_host_cannot_cast_for_owned_mage() {
    let (mut f, choice) = mage();
    f.begin();
    attack_hit(&mut f, &choice, 15);
    let window = select(&mut f, true);
    let source_choice = shield(&f);
    for actor in [None, Some(0)] {
        f.rejected(
            actor,
            TacticalAction::CastHitShield {
                window,
                choice: source_choice.clone(),
            },
        );
        f.rejected(actor, TacticalAction::DeclineSelectedHitShield { window });
    }
    f.run(Some(1), TacticalAction::DeclineSelectedHitShield { window });
    assert_eq!(uses(&f), 0);
    assert!(
        f.rules()
            .timing
            .as_ref()
            .unwrap()
            .reactions_spent
            .is_empty()
    );
    assert_eq!(f.request().roller, Some(f.actors[0]));
    f.roll(0, &[1]);
}

#[test]
fn selected_shield_revalidates_the_exact_spell_grant_and_resource_before_any_payment() {
    let (mut f, choice) = mage();
    f.begin();
    attack_hit(&mut f, &choice, 15);
    let source_choice = shield(&f);
    let waiting = hit_window(&f);
    f.rejected(
        Some(1),
        TacticalAction::CastHitShield {
            window: waiting,
            choice: source_choice.clone(),
        },
    );
    let window = select(&mut f, false);
    for case in 0..5 {
        let mut wrong = source_choice.clone();
        match case {
            0 => wrong.spell_id = "counterspell".into(),
            1 => wrong.grant = SpellGrantChoice::Prepared,
            2 => wrong.resource = SpellResourceChoice::Slot { level: 1 },
            3 => wrong.actor = f.actors[0],
            4 => {
                wrong.mode = SpellCastMode::Ready {
                    trigger: "A later hit".into(),
                }
            }
            _ => unreachable!(),
        }
        f.rejected(
            Some(1),
            TacticalAction::CastHitShield {
                window,
                choice: wrong,
            },
        );
        assert_eq!(uses(&f), 0);
        assert!(
            f.rules()
                .timing
                .as_ref()
                .unwrap()
                .reactions_spent
                .is_empty()
        );
    }
    cast(&mut f, window);
    assert_eq!(uses(&f), 1);
    f.roll(0, &[1]); // 15+5 still hits17, so the originally issued damage remains.
}

#[test]
fn fixed_unarmed_damage_after_shield_keeps_original_roll_and_current_effect_executor() {
    let (mut f, _) = mage();
    f.begin();
    let hp = f.rules().entities[&f.actors[1]].hp;
    f.run(
        Some(0),
        TacticalAction::UnarmedStrike {
            target: f.actors[1],
        },
    );
    f.roll(0, &[20]);
    let original = f.rules().rolls.last().unwrap().clone();
    let window = select(&mut f, true);
    let casting = cast(&mut f, window);
    assert_eq!(f.rules().entities[&f.actors[1]].hp, hp - 4);
    assert_eq!(f.rules().rolls.last(), Some(&original));
    assert!(f.rules().pending.is_none()); // A critical fixed1+Strength invents no damage dice.
    assert!(f.flow().resolution.is_none());
    let effects = f.rules().tactical_effects.as_ref().unwrap();
    assert_eq!(effects.effects[0].source.command, casting.meta);
    assert_eq!(
        effects.last_operation.as_ref().unwrap().command,
        casting.meta
    );
    assert_ne!(casting.meta, original.accepted_by);
}

#[test]
fn spent_reaction_blocks_a_second_shield_but_the_existing_defense_and_second_hit_continue_remain() {
    let mut f = Fixture::new();
    let choice = f.arm("club", false, false);
    let other = f.item("club", 1);
    f.state
        .rules
        .as_mut()
        .unwrap()
        .tactical_inventory
        .as_mut()
        .unwrap()
        .loadouts[0]
        .hands
        .hands[1] = HandAssignment::Item(other);
    source_actor(&mut f, 1, "mage", CreatureSize::Medium);
    f.begin();
    let first = f.run(
        Some(0),
        TacticalAction::Attack {
            choice: choice.clone(),
        },
    );
    f.roll(0, &[8]);
    let window = select(&mut f, false);
    cast(&mut f, window);
    let mut extra = choice;
    extra.weapon = other;
    extra.grip = WeaponGrip::OneHand(Hand::Right);
    extra.purpose = WeaponAttackPurpose::LightBonus {
        trigger: first.meta.id,
    };
    f.run(Some(0), TacticalAction::Attack { choice: extra });
    f.roll(0, &[15]);
    assert_eq!(resolution(&f).attack.as_ref().unwrap().armor_class, 17);
    assert!(shield_choices(&f.state, f.actors[1]).unwrap().is_empty());
    f.rejected(
        Some(1),
        TacticalAction::RespondToHit {
            window: hit_window(&f),
            accept: true,
        },
    );
    f.decline_hit_responses();
    assert_eq!(uses(&f), 1);
    f.roll(0, &[1]);
    assert_eq!(effective_armor_class(&f.state, f.actors[1]).unwrap(), 17);
}

#[test]
fn genuine_three_use_pool_exhausts_across_turns_but_still_requires_fourth_target_continue() {
    let (mut f, choice) = mage();
    f.begin();
    for spent in 1..=3 {
        attack_hit(&mut f, &choice, 8);
        let window = select(&mut f, spent % 2 == 0);
        cast(&mut f, window);
        assert_eq!(uses(&f), spent);
        next_round(&mut f);
    }
    attack_hit(&mut f, &choice, 8);
    assert!(shield_choices(&f.state, f.actors[1]).unwrap().is_empty());
    f.rejected(
        Some(1),
        TacticalAction::RespondToHit {
            window: hit_window(&f),
            accept: true,
        },
    );
    assert!(f.rules().pending.is_none());
    f.decline_hit_responses();
    assert_eq!(uses(&f), 3);
    f.roll(0, &[1]);
}

#[test]
fn actual_hands_and_incapacitation_block_casting_but_not_owned_cost_free_continue() {
    for incapacitated in [false, true] {
        let (mut f, choice) = mage();
        let actor = f.actors[1];
        if incapacitated {
            f.state.rules.as_mut().unwrap().effects.push(ActiveEffect {
                id: EffectId::new(),
                source: actor,
                target: actor,
                condition: Some(Condition::Incapacitated),
                label: "Existing source condition".into(),
                expires: Expiry::Never,
                concentration_owner: None,
            });
        } else {
            let dagger = f.item("dagger", 1);
            f.state.items.get_mut(&dagger).unwrap().custody = Custody::Entity(actor);
            let wand = f
                .state
                .items
                .values()
                .find(|item| item.definition_id == "wand" && item.custody == Custody::Entity(actor))
                .unwrap()
                .id;
            let meta = f.meta(Some(1));
            let loadout = f
                .state
                .rules
                .as_mut()
                .unwrap()
                .tactical_inventory
                .as_mut()
                .unwrap()
                .loadouts
                .iter_mut()
                .find(|l| l.actor == actor)
                .unwrap();
            loadout.hands.hands = [HandAssignment::Item(wand), HandAssignment::Item(dagger)];
            loadout.command = meta;
            f.state.applied_event_sequence += 1;
        }
        f.begin();
        attack_hit(&mut f, &choice, 15);
        assert!(shield_choices(&f.state, actor).unwrap().is_empty());
        f.rejected(
            Some(1),
            TacticalAction::RespondToHit {
                window: hit_window(&f),
                accept: true,
            },
        );
        f.decline_hit_responses();
        assert_eq!(uses(&f), 0);
        assert!(
            !f.rules()
                .timing
                .as_ref()
                .unwrap()
                .reactions_spent
                .contains(&actor)
        );
        f.roll(0, &[1]);
    }
}

#[test]
fn shield_opportunity_child_preserves_original_move_and_reaction_damage_cause() {
    let (mut f, choice) = mage();
    f.begin();
    f.run(Some(0), TacticalAction::EndTurn);
    let moving = f.run(
        Some(1),
        TacticalAction::Move {
            path: vec![TacticalMoveStep {
                destination: SpatialPoint { x: 30, y: 10, z: 0 },
                mode: MovementMode::Walk,
            }],
        },
    );
    let attack = f.run(
        Some(0),
        TacticalAction::OpportunityAttack {
            choice: TacticalMeleeChoice::Weapon(choice),
        },
    );
    let result = f.raw(&[20]);
    let rolled = f.run(Some(0), TacticalAction::SubmitRoll { result });
    assert_eq!(resolution(&f).origin, moving.meta);
    assert_eq!(resolution(&f).attack.as_ref().unwrap().origin, attack.meta);
    assert_eq!(resolution(&f).turn_actor, f.actors[1]);
    let retained_move = resolution(&f).movement.clone();
    let window = hit_window(&f);
    f.rejected(
        Some(0),
        TacticalAction::OrderHitResponses {
            window,
            instruction: hit_responses::forward_order(),
        },
    );
    let window = select(&mut f, false);
    cast(&mut f, window);
    assert_eq!(resolution(&f).origin, moving.meta);
    assert_eq!(resolution(&f).movement, retained_move);
    assert_eq!(f.rules().pending.as_ref().unwrap().issued_by, rolled.meta);
    assert_eq!(
        f.state.encounter.as_ref().unwrap().participants[1]
            .position
            .x,
        20
    );
    f.roll(0, &[1, 1]);
    assert!(f.flow().resolution.is_none());
    assert_eq!(
        f.state.encounter.as_ref().unwrap().participants[1]
            .position
            .x,
        30
    );
    assert_eq!(f.flow().budget.movement_spent, 10);
    assert!(
        f.rules()
            .timing
            .as_ref()
            .unwrap()
            .reactions_spent
            .contains(&f.actors[0])
    );
    assert!(
        f.rules()
            .timing
            .as_ref()
            .unwrap()
            .reactions_spent
            .contains(&f.actors[1])
    );
}

#[test]
fn shield_spell_child_preserves_each_ray_and_original_cast_after_completed_review_is_retired() {
    let mut f = Fixture::new();
    source_actor(&mut f, 0, "adult-red-dragon", CreatureSize::Huge);
    source_actor(&mut f, 1, "mage", CreatureSize::Medium);
    f.state.encounter.as_mut().unwrap().participants[1]
        .position
        .x = 60;
    f.begin();
    let casting = f.run(
        Some(0),
        TacticalAction::CastSpell {
            choice: SpellCastChoice {
                actor: f.actors[0],
                spell_id: "scorching-ray".into(),
                grant: SpellGrantChoice::CreatureFeature {
                    feature_id: "spellcasting".into(),
                },
                resource: SpellResourceChoice::SourceFeature,
                material: SpellMaterialChoice::None,
                mode: SpellCastMode::Immediate,
            },
            targets: SpellTargetChoice::Entities(vec![f.actors[1]; 3]),
        },
    );
    f.roll(0, &[20]);
    let hit_cause = f.rules().rolls.last().unwrap().accepted_by.clone();
    let window = select(&mut f, false);
    cast(&mut f, window);
    assert_eq!(resolution(&f).origin, casting.meta);
    assert_eq!(resolution(&f).casts.len(), 1); // Original three-ray cast survives child removal.
    assert_eq!(resolution(&f).casts[0].cast.plan.origin, casting.meta);
    assert_eq!(f.rules().pending.as_ref().unwrap().issued_by, hit_cause);
    f.roll(0, &[1, 1, 1, 1]);
    assert!(resolution(&f).hit_review.is_none());
    assert_eq!(
        f.request().dice,
        [DieSpec {
            count: 1,
            sides: 20
        }]
    );
    let next_ray_origin = f.rules().rolls.last().unwrap().accepted_by.clone();
    assert_eq!(
        resolution(&f).attack.as_ref().unwrap().origin,
        next_ray_origin
    );
    assert_ne!(next_ray_origin, casting.meta);
    f.roll(0, &[20]);
    let second_hit = f.rules().rolls.last().unwrap().clone();
    assert_eq!(second_hit.issued_by, next_ray_origin);
    let PendingPurpose::TacticalResolution { key, .. } = second_hit.purpose else {
        panic!("spell ray lacks tactical key");
    };
    assert_eq!(key.origin, casting.meta.id);
    assert!(shield_choices(&f.state, f.actors[1]).unwrap().is_empty());
    f.decline_hit_responses();
    assert_eq!(
        f.rules().pending.as_ref().unwrap().issued_by,
        second_hit.accepted_by
    );
    f.roll(0, &[1, 1, 1, 1]);
    f.roll(0, &[1]);
    assert!(f.flow().resolution.is_none());
    assert_eq!(uses(&f), 1);
}

fn assert_corrupt(state: CampaignState, pack: &RulesPack, case: usize) {
    let restored: CampaignState =
        serde_json::from_slice(&serde_json::to_vec(&state).unwrap()).unwrap();
    assert!(
        validate_state(&restored, pack).is_err() || validate_tactical_state(&restored).is_err(),
        "case {case}"
    );
}

#[test]
fn collecting_hit_rejects_forged_cause_controller_work_cover_or_skipped_stage_after_serialization()
{
    let (mut f, choice) = mage();
    f.begin();
    attack_hit(&mut f, &choice, 20);
    for case in 0..10 {
        let mut state = f.state.clone();
        let r = state
            .encounter
            .as_mut()
            .unwrap()
            .flow
            .as_mut()
            .unwrap()
            .resolution
            .as_mut()
            .unwrap();
        let hit = r.hit_review.as_mut().unwrap();
        match case {
            0 => r.hit_review = None,
            1 => hit.cause.issuer = CommandIssuer::Player(f.players[1]),
            2 => hit.respondent.as_mut().unwrap().actor = f.actors[0],
            3 => hit.cover_bonus = 2,
            4 => hit.work.occurrence += 1,
            5 => hit.attack_roll = f.rules().rolls[0].request.id,
            6 => hit.stage = TacticalHitReviewStage::Selected,
            7 => {
                let duplicate = r.frames.last().unwrap()[0].clone();
                r.frames.last_mut().unwrap().push(duplicate);
            }
            8 => {
                let resume = r.frames.last().unwrap()[0].occurrence;
                r.work_trace
                    .as_mut()
                    .unwrap()
                    .nodes
                    .iter_mut()
                    .find(|n| n.work.occurrence == resume)
                    .unwrap()
                    .parent = None;
            }
            9 => hit.respondent = None,
            _ => unreachable!(),
        }
        assert_corrupt(state, &f.pack, case);
    }
}

#[test]
fn completed_shield_damage_pause_rejects_forged_effect_source_cost_original_facts_or_damage_cause()
{
    let (mut f, choice) = mage();
    f.begin();
    attack_hit(&mut f, &choice, 20);
    let window = select(&mut f, true);
    let casting = cast(&mut f, window);
    for case in 0..13 {
        let mut state = f.state.clone();
        let r = state
            .encounter
            .as_mut()
            .unwrap()
            .flow
            .as_mut()
            .unwrap()
            .resolution
            .as_mut()
            .unwrap();
        let hit = r.hit_review.as_mut().unwrap();
        match case {
            0 => r.hit_review = None,
            1 => hit.completed_shield = None,
            2 => {
                hit.completed_shield
                    .as_mut()
                    .unwrap()
                    .cast
                    .plan
                    .choice
                    .actor = f.actors[0]
            }
            3 => hit.completed_shield.as_mut().unwrap().completed.clear(),
            4 => {
                hit.respondent
                    .as_mut()
                    .unwrap()
                    .intent
                    .as_mut()
                    .unwrap()
                    .origin
                    .issuer = CommandIssuer::Admin
            }
            5 => hit.order.as_mut().unwrap().origin.issuer = CommandIssuer::Player(f.players[1]),
            6 => {
                state
                    .rules
                    .as_mut()
                    .unwrap()
                    .timing
                    .as_mut()
                    .unwrap()
                    .reactions_spent
                    .remove(&f.actors[1]);
            }
            7 => {
                state
                    .rules
                    .as_mut()
                    .unwrap()
                    .tactical_effects
                    .as_mut()
                    .unwrap()
                    .effects[0]
                    .defenses[0] = EffectDefense::ArmorClassBonus { bonus: 6 };
            }
            8 => {
                state
                    .rules
                    .as_mut()
                    .unwrap()
                    .tactical_effects
                    .as_mut()
                    .unwrap()
                    .effects[0]
                    .expires = TacticalEffectExpiry::Never;
            }
            9 => {
                state
                    .rules
                    .as_mut()
                    .unwrap()
                    .tactical_effects
                    .as_mut()
                    .unwrap()
                    .effects
                    .clear();
            }
            10 => {
                state
                    .rules
                    .as_mut()
                    .unwrap()
                    .pending
                    .as_mut()
                    .unwrap()
                    .issued_by = casting.meta.clone();
            }
            11 => {
                r.attack.as_mut().unwrap().armor_class += 5;
            }
            12 => {
                state
                    .rules
                    .as_mut()
                    .unwrap()
                    .tactical_effects
                    .as_mut()
                    .unwrap()
                    .effects[0]
                    .established_at
                    .as_mut()
                    .unwrap()
                    .command = hit.cause.clone();
            }
            _ => unreachable!(),
        };
        assert_corrupt(state, &f.pack, case);
    }
}
