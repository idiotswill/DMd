use super::*;
use dmd_rules::tactical_creatures::*;

fn move_away(f: &Fixture) -> TacticalAction {
    let mut destination = f
        .state
        .encounter
        .as_ref()
        .unwrap()
        .participant(f.actors[1])
        .unwrap()
        .position;
    destination.x += 10;
    TacticalAction::Move {
        path: vec![TacticalMoveStep {
            destination,
            mode: if f.rules().entities[&f.actors[1]].prone {
                MovementMode::Crawl
            } else {
                MovementMode::Walk
            },
        }],
    }
}
fn start_crossing(f: &mut Fixture) -> TacticalEvent {
    f.begin();
    f.run(Some(0), TacticalAction::EndTurn);
    f.run(Some(1), move_away(f))
}
fn creature(f: &mut Fixture, id: &str, size: CreatureSize) {
    let actor = f.actors[0];
    f.state.characters.retain(|_, c| c.entity_id != actor);
    f.state.entities.get_mut(&actor).unwrap().kind = EntityKind::Creature;
    f.state.rules.as_mut().unwrap().entities.remove(&actor);
    let built = build_creature(
        &f.state,
        &f.meta(None),
        actor,
        &CreatureBuildChoice {
            definition_id: id.into(),
            size,
            additional_languages: vec![],
            hit_points: CreatureHitPointChoice::Average,
            controller: CreatureController::Player(f.players[0]),
            in_lair: false,
        },
    )
    .unwrap();
    let r = f.state.rules.as_mut().unwrap();
    r.entities.insert(actor, built.mechanics);
    r.tactical_creatures = Some(TacticalCreatures {
        schema_version: TACTICAL_CREATURES_SCHEMA_VERSION,
        profiles: vec![built.profile],
        runtime: vec![built.runtime],
    });
    let e = f.state.encounter.as_mut().unwrap();
    e.participants[0].size = size;
    e.participants[0].height = size.footprint_units() as u32;
    e.participants[0].movement = built.movement;
    e.participants[0].senses = built.senses;
    e.participants[0].enemies = vec![f.actors[1]];
    e.participants[1].enemies = vec![actor];
    e.participants[1].position.x = 10 + size.footprint_units();
    f.entity_mut(1).max_hp = 100;
    f.entity_mut(1).hp = 100;
    f.state.applied_event_sequence += 1;
}

#[test]
fn lethal_opportunity_damage_finishes_attack_and_stops_the_original_move() {
    let mut f = Fixture::new();
    f.arm("longsword", false, false);
    f.entity_mut(1).hp = 1;
    f.entity_mut(1).max_hp = 1;
    start_crossing(&mut f);
    f.run(
        Some(0),
        TacticalAction::OpportunityAttack {
            choice: TacticalMeleeChoice::UnarmedDamage {
                ability: Ability::Strength,
            },
        },
    );
    f.roll(0, &[15]);
    f.run(
        Some(0),
        TacticalAction::ChooseAttackKnockout {
            choice: KnockoutChoice::NormalDamage,
        },
    );
    assert!(f.rules().entities[&f.actors[1]].death.dead);
    assert!(f.flow().resolution.is_none());
    assert!(f.rules().pending.is_none());
    assert_eq!(
        f.state.encounter.as_ref().unwrap().participants[1]
            .position
            .x,
        20
    );
    assert_eq!(f.flow().budget.movement_spent, 0);
    assert!(
        f.rules()
            .timing
            .as_ref()
            .unwrap()
            .reactions_spent
            .contains(&f.actors[0])
    );
}

#[test]
fn unarmed_reaction_has_real_fixed_damage_without_invented_inventory_or_action_cost() {
    let mut f = Fixture::new();
    f.arm("longsword", false, false);
    let inventory = f.state.items.clone();
    let moving = start_crossing(&mut f);
    let accepted = f.run(
        Some(0),
        TacticalAction::OpportunityAttack {
            choice: TacticalMeleeChoice::UnarmedDamage {
                ability: Ability::Strength,
            },
        },
    );
    let r = f.flow().resolution.as_ref().unwrap();
    assert_eq!(r.origin, moving.meta);
    assert_eq!(r.attack.as_ref().unwrap().origin, accepted.meta);
    assert!(matches!(
        r.attack.as_ref().unwrap().source,
        TacticalAttackSource::Unarmed { .. }
    ));
    assert_eq!(f.request().modifier, 5);
    assert!(savage_attacker_dice(&f.state, &f.pack).is_err());
    f.roll(0, &[20]);
    assert_eq!(f.rules().entities[&f.actors[1]].hp, 46); // Fixed 1+3 does not double.
    assert!(savage_attacker_dice(&f.state, &f.pack).is_err());
    assert_eq!(f.state.items, inventory);
    assert!(f.flow().budget.weapon_history.is_empty());
    assert!(!f.rules().timing.as_ref().unwrap().action_spent);
    assert!(
        f.rules()
            .timing
            .as_ref()
            .unwrap()
            .reactions_spent
            .contains(&f.actors[0])
    );
    assert_eq!(
        f.state.encounter.as_ref().unwrap().participants[1]
            .position
            .x,
        30
    );
}

#[test]
fn malformed_reaction_choice_rejects_before_consuming_the_live_crossing() {
    let mut f = Fixture::new();
    let choice = f.arm("longsword", false, false);
    start_crossing(&mut f);
    f.rejected(
        Some(0),
        TacticalAction::OpportunityAttack {
            choice: TacticalMeleeChoice::UnarmedDamage {
                ability: Ability::Dexterity,
            },
        },
    );
    for variation in 0..4 {
        let mut bad = choice.clone();
        match variation {
            0 => bad.target = f.actors[0],
            1 => bad.delivery = WeaponDelivery::Thrown,
            2 => {
                bad.purpose = WeaponAttackPurpose::LightBonus {
                    trigger: CommandId::new(),
                }
            }
            _ => {
                bad.equipment_change = Some(AttackEquipmentChange {
                    timing: EquipmentChangeTiming::AfterAttack,
                    operation: AttackEquipmentOperation::Unequip {
                        item: choice.weapon,
                    },
                })
            }
        }
        f.rejected(
            Some(0),
            TacticalAction::OpportunityAttack {
                choice: TacticalMeleeChoice::Weapon(bad),
            },
        );
    }
    assert!(
        !f.rules()
            .timing
            .as_ref()
            .unwrap()
            .reactions_spent
            .contains(&f.actors[0])
    );
    assert!(
        f.flow()
            .resolution
            .as_ref()
            .unwrap()
            .movement
            .as_ref()
            .unwrap()
            .opportunity
            .is_some()
    );
}

#[test]
fn restored_reaction_requires_exact_crossing_response_cost_and_source() {
    let mut f = Fixture::new();
    f.arm("longsword", false, false);
    start_crossing(&mut f);
    f.run(
        Some(0),
        TacticalAction::OpportunityAttack {
            choice: TacticalMeleeChoice::UnarmedDamage {
                ability: Ability::Strength,
            },
        },
    );
    for variation in 0..10 {
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
        match variation {
            0 => r.attack.as_mut().unwrap().admission = TacticalAttackAdmission::OwnTurn,
            1 => r.movement.as_mut().unwrap().decisions[0].origin.id = CommandId::new(),
            2 => bad
                .rules
                .as_mut()
                .unwrap()
                .timing
                .as_mut()
                .unwrap()
                .reactions_spent
                .clear(),
            3 => r.attack.as_mut().unwrap().attack_modifier += 1,
            4 => {
                let TacticalAttackAdmission::Opportunity(w) =
                    &mut r.attack.as_mut().unwrap().admission
                else {
                    unreachable!()
                };
                w.to.x += 1;
            }
            5 => {
                let TacticalAttackAdmission::Opportunity(w) =
                    &mut r.attack.as_mut().unwrap().admission
                else {
                    unreachable!()
                };
                w.options.clear();
            }
            6 => {
                r.attack.as_mut().unwrap().source = TacticalAttackSource::Unarmed {
                    ability: Ability::Dexterity,
                }
            }
            _ => {
                let TacticalAttackAdmission::Opportunity(w) =
                    &mut r.attack.as_mut().unwrap().admission
                else {
                    unreachable!()
                };
                match variation {
                    7 => w.origin.campaign_id = CampaignId::new(),
                    8 => w.origin.expected_event_sequence = 0,
                    _ => w.origin.issuer = CommandIssuer::Import,
                }
            }
        }
        assert!(
            validate_tactical_state(&bad).is_err(),
            "accepted forged reaction {variation}"
        );
    }
    f.roll(0, &[1]);
}

#[test]
fn source_wolf_bite_uses_printed_bonus_and_prone_rider_without_a_fake_weapon() {
    let mut f = Fixture::new();
    creature(&mut f, "wolf", CreatureSize::Medium);
    start_crossing(&mut f);
    let items = f.state.items.clone();
    f.run(
        Some(0),
        TacticalAction::OpportunityAttack {
            choice: TacticalMeleeChoice::CreatureFeature {
                feature_id: "bite".into(),
                weapon: None,
            },
        },
    );
    assert_eq!(f.request().modifier, 4);
    f.roll(0, &[15]);
    assert_eq!(f.request().dice, [DieSpec { count: 1, sides: 6 }]);
    f.roll(0, &[4]);
    assert_eq!(f.rules().entities[&f.actors[1]].hp, 94);
    assert!(f.rules().entities[&f.actors[1]].prone);
    assert_eq!(f.state.items, items);
    assert_eq!(f.flow().budget.movement_spent, 0); // The accepted Walk cannot silently become Crawl.
    assert_eq!(
        f.state.encounter.as_ref().unwrap().participants[1]
            .position
            .x,
        20
    );
}

#[test]
fn chimera_bite_uses_extra_dice_only_when_advantage_survives_cancellation() {
    for (disadvantage, critical, resistant) in [
        (false, false, false),
        (true, false, false),
        (false, true, false),
        (false, false, true),
    ] {
        let mut f = Fixture::new();
        creature(&mut f, "chimera", CreatureSize::Large);
        f.entity_mut(1).prone = true;
        f.entity_mut(0).prone = disadvantage;
        if resistant {
            f.entity_mut(1).resistances.insert(DamageType::Piercing);
        }
        start_crossing(&mut f);
        f.run(
            Some(0),
            TacticalAction::OpportunityAttack {
                choice: TacticalMeleeChoice::CreatureFeature {
                    feature_id: "bite".into(),
                    weapon: None,
                },
            },
        );
        assert_eq!(f.request().modifier, 7);
        assert_eq!(
            f.request().mode,
            if disadvantage {
                RollMode::Normal
            } else {
                RollMode::Advantage
            }
        );
        f.roll(
            0,
            if critical {
                &[20, 20]
            } else if disadvantage {
                &[15]
            } else {
                &[15, 16]
            },
        );
        assert_eq!(
            f.request().dice,
            if critical {
                vec![
                    DieSpec { count: 4, sides: 6 },
                    DieSpec { count: 4, sides: 6 },
                ]
            } else if disadvantage {
                vec![DieSpec { count: 2, sides: 6 }]
            } else {
                vec![
                    DieSpec { count: 2, sides: 6 },
                    DieSpec { count: 2, sides: 6 },
                ]
            }
        );
        f.roll(
            0,
            if critical {
                &[1, 2, 3, 4, 5, 6, 1, 2]
            } else if disadvantage {
                &[1, 2]
            } else {
                &[1, 2, 3, 4]
            },
        );
        assert_eq!(
            f.rules().entities[&f.actors[1]].hp,
            if critical {
                72
            } else if disadvantage || resistant {
                93
            } else {
                86
            }
        );
    }
}

#[test]
fn source_reaction_cannot_claim_the_other_actors_charge_movement() {
    let mut f = Fixture::new();
    creature(&mut f, "warhorse", CreatureSize::Large);
    start_crossing(&mut f);
    f.run(
        Some(0),
        TacticalAction::OpportunityAttack {
            choice: TacticalMeleeChoice::CreatureFeature {
                feature_id: "hooves".into(),
                weapon: None,
            },
        },
    );
    assert_eq!(f.request().modifier, 6);
    f.roll(0, &[15]);
    assert_eq!(f.request().dice, [DieSpec { count: 2, sides: 4 }]);
    f.roll(0, &[1, 2]);
    assert_eq!(f.rules().entities[&f.actors[1]].hp, 93);
    assert!(!f.rules().entities[&f.actors[1]].prone);
}

#[test]
fn charmer_or_total_cover_crossing_does_not_offer_an_unusable_attack() {
    for charm in [false, true] {
        let mut f = Fixture::new();
        f.arm("longsword", false, false);
        if charm {
            f.state.rules.as_mut().unwrap().effects.push(ActiveEffect {
                id: EffectId::new(),
                source: f.actors[1],
                target: f.actors[0],
                condition: Some(Condition::Charmed),
                label: "Charmed by mover".into(),
                expires: Expiry::Never,
                concentration_owner: None,
            });
        } else {
            f.state
                .encounter
                .as_mut()
                .unwrap()
                .battlefield
                .obstacles
                .push(SpatialObstacle {
                    id: "transparent-total-barrier".into(),
                    volume: SpatialBox {
                        min: SpatialPoint { x: 18, y: 0, z: 0 },
                        max: SpatialPoint {
                            x: 19,
                            y: 100,
                            z: 100,
                        },
                    },
                    blocks_movement: false,
                    blocks_sight: false,
                    observable: true,
                    cover: CoverDegree::Total,
                });
        }
        start_crossing(&mut f);
        assert!(f.flow().resolution.is_none());
        assert!(
            f.rules()
                .timing
                .as_ref()
                .unwrap()
                .reactions_spent
                .is_empty()
        );
        assert_eq!(
            f.state.encounter.as_ref().unwrap().participants[1]
                .position
                .x,
            30
        );
    }
}

#[test]
fn distant_enemy_with_uncertain_cover_cannot_block_unrelated_movement() {
    let mut f = Fixture::new();
    f.arm("longsword", false, false);
    let e = f.state.encounter.as_mut().unwrap();
    e.participants[1].position.x = 70;
    e.battlefield.obstacles.push(SpatialObstacle {
        id: "distant-partial-barrier".into(),
        volume: SpatialBox {
            min: SpatialPoint { x: 70, y: 0, z: 0 },
            max: SpatialPoint {
                x: 71,
                y: 100,
                z: 100,
            },
        },
        blocks_movement: false,
        blocks_sight: false,
        observable: true,
        cover: CoverDegree::Total,
    });
    let assessed = dmd_rules::spatial::cover_from(
        e,
        e.participants[0].center().unwrap(),
        e.participants[1].volume().unwrap(),
        &f.actors,
    )
    .unwrap();
    assert!(assessed.requires_adjudication); // The ambiguity is real, but no reach can cross it.
    start_crossing(&mut f);
    assert!(f.flow().resolution.is_none());
    assert!(
        f.rules()
            .timing
            .as_ref()
            .unwrap()
            .reactions_spent
            .is_empty()
    );
    assert_eq!(
        f.state.encounter.as_ref().unwrap().participants[1]
            .position
            .x,
        80
    );
    assert_eq!(f.flow().budget.movement_spent, 10);
}

#[test]
fn movement_within_reach_does_not_consult_uncertain_opportunity_cover() {
    let mut f = Fixture::new();
    f.arm("longsword", false, false);
    let e = f.state.encounter.as_mut().unwrap();
    e.battlefield.obstacles.push(SpatialObstacle {
        id: "partial-reaction-barrier".into(),
        volume: SpatialBox {
            min: SpatialPoint { x: 20, y: 0, z: 0 },
            max: SpatialPoint {
                x: 21,
                y: 100,
                z: 100,
            },
        },
        blocks_movement: false,
        blocks_sight: false,
        observable: false,
        cover: CoverDegree::Total,
    });
    let assessed = dmd_rules::spatial::cover_from(
        e,
        e.participants[0].center().unwrap(),
        e.participants[1].volume().unwrap(),
        &f.actors,
    )
    .unwrap();
    assert!(assessed.requires_adjudication);
    let mut destination = e.participants[1].position;
    destination.y += 10;
    let mut after = e.participants[1].clone();
    after.position = destination;
    assert!(dmd_rules::spatial::participant_distance(&e.participants[0], &after).unwrap() <= 10);
    f.begin();
    f.run(Some(0), TacticalAction::EndTurn);
    f.run(
        Some(1),
        TacticalAction::Move {
            path: vec![TacticalMoveStep {
                destination,
                mode: MovementMode::Walk,
            }],
        },
    );
    assert!(f.flow().resolution.is_none());
    assert!(
        f.rules()
            .timing
            .as_ref()
            .unwrap()
            .reactions_spent
            .is_empty()
    );
    assert_eq!(f.flow().budget.movement_spent, 10);
    assert_eq!(
        f.state.encounter.as_ref().unwrap().participants[1].position,
        destination
    );
}
