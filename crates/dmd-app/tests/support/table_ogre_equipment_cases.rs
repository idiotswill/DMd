//! Genuine catalog creation, finite physical source play and ordinary recovery.
//! These scenarios only change campaign state through accepted application commands.
use super::*;

struct Gear {
    actor: EntityId,
    source: CreatureSourcePin,
    greatclub: ItemId,
    javelins: Vec<ItemId>,
}

async fn hostile_source(f: &Fixture, actor: EntityId, item: ItemId) {
    let original = export_campaign(&f.pool, f.campaign).await.unwrap();
    for mutation in 0..4 {
        let mut forged = original.clone();
        let mut changed = state(f).await;
        match mutation {
            0 => changed
                .rules
                .as_mut()
                .unwrap()
                .tactical_creatures
                .as_mut()
                .unwrap()
                .profiles
                .iter_mut()
                .find(|profile| profile.actor == actor)
                .unwrap()
                .source
                .definition_fingerprint
                .push('x'),
            1 => changed.items.get_mut(&item).unwrap().quantity = 2,
            2 => {
                changed
                    .encounter
                    .as_mut()
                    .unwrap()
                    .flow
                    .as_mut()
                    .unwrap()
                    .resolution
                    .as_mut()
                    .unwrap()
                    .attack
                    .as_mut()
                    .unwrap()
                    .damage[0]
                    .dice[0]
                    .count = 1
            }
            3 => changed.items.get_mut(&item).unwrap().owner = Ownership::Unowned,
            _ => unreachable!(),
        }
        forged.current_state.state_json = changed.encode_json().unwrap();
        let sequence = i64::try_from(changed.applied_event_sequence).unwrap();
        forged
            .snapshots
            .retain(|row| row.event_sequence != sequence);
        forged.snapshots.push(dmd_persistence::SnapshotRow {
            campaign_id: forged.campaign_id.clone(),
            event_sequence: sequence,
            state_schema_version: i64::from(changed.schema_version),
            state_json: changed.encode_json().unwrap(),
            created_at_utc: forged.exported_at_utc.clone(),
        });
        forged.snapshots.sort_by_key(|row| row.event_sequence);
        Box::pin(forged_refused(&forged)).await;
    }
}

async fn prepare(f: &mut Fixture, path: &Path) -> Gear {
    let shown = view(f, &TableTransportChannel::Host).await;
    let catalog = f
        .runtime
        .table_creature_options(TableCreatureOptionsRequest {
            campaign_id: f.campaign,
            channel: TableTransportChannel::Host,
            revision: shown.revision,
        })
        .await
        .unwrap();
    let option = catalog
        .iter()
        .find(|option| option.definition_id == "ogre")
        .unwrap();
    assert_eq!(option.item_count, 4);
    assert!(!option.ammunition_required);
    let actor = EntityId::new();
    let ids = (0..4).map(|_| ItemId::new()).collect::<Vec<_>>();
    let create = TableCreatureCreation {
        entity_id: actor,
        name: "Large giant with physical gear".into(),
        definition_id: option.definition_id.clone(),
        source: option.source.clone(),
        size: CreatureSize::Large,
        additional_languages: vec![],
        ammunition_units: 0,
        item_ids: ids.clone(),
    };
    for mutation in 0..6 {
        let mut invalid = create.clone();
        match mutation {
            0 => invalid.source = None,
            1 => invalid
                .source
                .as_mut()
                .unwrap()
                .definition_fingerprint
                .push('x'),
            2 => {
                invalid.item_ids.pop();
            }
            3 => invalid.item_ids[1] = invalid.item_ids[0],
            4 => invalid.item_ids[0] = ItemId(uuid::Uuid::nil()),
            5 => invalid.ammunition_units = 1,
            _ => unreachable!(),
        }
        let bad = request(
            f,
            TableTransportChannel::Host,
            TableTransportInput::Action(Box::new(TableAction::CreateCreature {
                creation: Box::new(invalid),
            })),
        )
        .await;
        Box::pin(rejected(f, bad)).await;
    }
    Box::pin(step(
        f,
        path,
        TableTransportChannel::Host,
        TableTransportInput::Action(Box::new(TableAction::CreateCreature {
            creation: Box::new(create),
        })),
    ))
    .await;
    let created = state(f).await;
    let items = ids.iter().map(|id| &created.items[id]).collect::<Vec<_>>();
    assert!(
        items
            .iter()
            .all(|item| item.quantity == 1 && item.custody == Custody::Entity(actor))
    );
    let greatclub = items
        .iter()
        .find(|item| item.definition_id == "greatclub")
        .unwrap()
        .id;
    let mut javelins = items
        .iter()
        .filter(|item| item.definition_id == "javelin")
        .map(|item| item.id)
        .collect::<Vec<_>>();
    javelins.sort_by_key(|id| id.0);
    assert_eq!(javelins.len(), 3);
    assert_eq!(created.rules.as_ref().unwrap().entities[&actor].hp, 68);
    let gear = Gear {
        actor,
        source: option.source.clone().unwrap(),
        greatclub,
        javelins,
    };

    let canonical = f
        .runtime
        .table_view(f.campaign, TableViewer::Host)
        .await
        .unwrap();
    let equipment = canonical
        .characters
        .iter()
        .find(|c| c.character_id == f.characters[0])
        .unwrap()
        .equipment
        .as_ref()
        .unwrap();
    let prepare_equipment = TableAction::PrepareEquipment {
        character_id: f.characters[0],
        item_ids: (0..equipment.initial_item_count)
            .map(|_| ItemId::new())
            .collect(),
    };
    Box::pin(step(
        f,
        path,
        TableTransportChannel::Host,
        TableTransportInput::Action(Box::new(prepare_equipment)),
    ))
    .await;
    let battlefield = TableAction::PrepareBattlefield { setup: Box::new(TableBattlefieldSetup {
        encounter_id: EncounterId::new(), scene_id: SceneId::new(), location_id: LocationId::new(),
        name: "Visible courtyard".into(), battlefield: Battlefield {
            bounds: SpatialBox { min: SpatialPoint { x: 0, y: 0, z: 0 }, max: SpatialPoint { x: 200, y: 100, z: 50 } },
            floor_z: 0, floor_surface: "stone".into(), ambient_light: LightLevel::Bright,
            terrain: vec![], obstacles: vec![], lights: vec![],
        }, characters: vec![TableCharacterPlacement { character_id: f.characters[0], position: SpatialPoint { x: 10, y: 10, z: 0 },
            height: 12, allies: vec![], enemies: vec![actor] }],
        creatures: vec![TableCreaturePlacement { actor, public_label: "Large armed giant".into(), position: SpatialPoint { x: 20, y: 10, z: 0 },
            height: 20, allies: vec![], enemies: vec![f.actors[0]] }],
        area_grid_policy: None, geometry_ruling: Ruling { basis: RulingBasis::GmAdjudication,
            reason: "Visible level stone courtyard with adjacent occupied cells at five-foot reach.".into() },
    }) };
    Box::pin(step(
        f,
        path,
        TableTransportChannel::Host,
        TableTransportInput::Action(Box::new(battlefield)),
    ))
    .await;
    let placed = state(f).await;
    let encounter = placed.encounter.as_ref().unwrap();
    assert_eq!(
        dmd_rules::spatial::participant_distance(
            encounter.participant(f.actors[0]).unwrap(),
            encounter.participant(actor).unwrap(),
        )
        .unwrap(),
        10
    );
    let begin = TacticalAction::Begin {
        execution: TacticalExecutionVersion::EncounterReleaseV1,
        combatants: vec![
            TacticalCombatant {
                actor: f.actors[0],
                source: TacticalSource::Character,
                surprised: false,
            },
            TacticalCombatant {
                actor,
                source: TacticalSource::Creature {
                    definition_id: "ogre".into(),
                },
                surprised: false,
            },
        ],
        groups: vec![
            InitiativeGroup {
                actors: vec![f.actors[0]],
                request_id: RollRequestId::new(),
            },
            InitiativeGroup {
                actors: vec![actor],
                request_id: RollRequestId::new(),
            },
        ],
    };
    Box::pin(step(f, path, TableTransportChannel::Host, action(begin))).await;
    Box::pin(player_raw(f, path, 18)).await;
    Box::pin(raw(f, path, TableTransportChannel::Host, 2)).await;
    Box::pin(step(
        f,
        path,
        TableTransportChannel::Host,
        action(TacticalAction::ActivateAttackEquipment),
    ))
    .await;
    gear
}

fn source_choice(
    gear: &Gear,
    target: EntityId,
    feature: &str,
    index: usize,
) -> CreatureWeaponUseChoice {
    let weapon = if feature == "greatclub" {
        gear.greatclub
    } else {
        gear.javelins[index]
    };
    CreatureWeaponUseChoice {
        weapon,
        target,
        grip: if feature == "greatclub" {
            WeaponGrip::TwoHands
        } else {
            WeaponGrip::OneHand(Hand::Right)
        },
        ammunition: None,
        equipment_change: (feature != "javelin-thrown").then_some(AttackEquipmentChange {
            timing: EquipmentChangeTiming::BeforeAttack,
            operation: AttackEquipmentOperation::Equip {
                item: weapon,
                hand: Hand::Right,
            },
        }),
        after_equipment: (feature == "javelin-thrown")
            .then_some(AfterAttackEquipmentIntent::Choose),
    }
}

async fn finish_source(f: &mut Fixture, path: &Path, feature: &str, face: u16) {
    Box::pin(raw(f, path, TableTransportChannel::Host, face)).await;
    if face > 1 {
        Box::pin(table_hit_driver::decline_hit_responses(f)).await;
        let pending = view(f, &TableTransportChannel::Host).await.roll.unwrap();
        assert_eq!(
            pending.dice,
            [DieSpec {
                count: if face == 20 { 4 } else { 2 },
                sides: if feature == "greatclub" { 8 } else { 6 }
            }]
        );
        assert_eq!(pending.modifier, 0);
        Box::pin(raw(f, path, TableTransportChannel::Host, 1)).await;
    }
    if feature == "javelin-thrown" {
        let offered = view(f, &TableTransportChannel::Host)
            .await
            .tactical
            .unwrap()
            .attack_equipment
            .unwrap();
        Box::pin(step(
            f,
            path,
            TableTransportChannel::Host,
            TableTransportInput::AttackEquipment {
                handle: offered.key,
                choice: AttackEquipmentChoice::Decline,
            },
        ))
        .await;
    }
}

async fn one_form(f: &mut Fixture, path: &Path, feature: &str, face: u16) {
    let gear = Box::pin(prepare(f, path)).await;
    Box::pin(player_step(f, path, action(TacticalAction::EndTurn))).await;
    let before = state(f).await;
    let choice = source_choice(&gear, f.actors[0], feature, 0);
    let item = choice.weapon;
    let options = view(f, &TableTransportChannel::Host)
        .await
        .tactical
        .unwrap()
        .attack_options
        .unwrap();
    assert!(options.weapons.iter().any(|weapon| {
        weapon.item == item
            && weapon
                .source_features
                .iter()
                .any(|source| source.feature_id == feature)
    }));
    Box::pin(step(
        f,
        path,
        TableTransportChannel::Host,
        action(TacticalAction::CreatureWeaponAttack {
            feature_id: feature.into(),
            choice,
        }),
    ))
    .await;
    let paid = state(f).await;
    let attack = paid
        .encounter
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .resolution
        .as_ref()
        .unwrap()
        .attack
        .as_ref()
        .unwrap();
    assert_eq!(
        attack.damage[0].dice,
        [DieSpec {
            count: 2,
            sides: if feature == "greatclub" { 8 } else { 6 }
        }]
    );
    assert_eq!(attack.damage[0].modifier, 4);
    assert!(
        paid.rules
            .as_ref()
            .unwrap()
            .timing
            .as_ref()
            .unwrap()
            .action_spent
    );
    assert_eq!(
        paid.encounter
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .budget
            .attacks_remaining,
        0
    );
    Box::pin(hostile_source(f, gear.actor, item)).await;
    Box::pin(finish_source(f, path, feature, face)).await;
    let complete = state(f).await;
    assert_eq!(complete.items.len(), before.items.len());
    assert_eq!(complete.items[&item].owner, before.items[&item].owner);
    assert_eq!(complete.items[&item].quantity, 1);
    assert_eq!(
        complete.items[&item].custody == Custody::Entity(gear.actor),
        feature != "javelin-thrown"
    );
    assert_eq!(
        complete
            .rules
            .as_ref()
            .unwrap()
            .tactical_creatures
            .as_ref()
            .unwrap()
            .profile(gear.actor)
            .unwrap()
            .source,
        gear.source
    );
    assert!(
        complete
            .encounter
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .resolution
            .is_none()
    );
}

async fn finite_and_recover(f: &mut Fixture, path: &Path) {
    let gear = Box::pin(prepare(f, path)).await;
    Box::pin(player_step(f, path, action(TacticalAction::EndTurn))).await;
    let initial = state(f).await;
    for index in 0..3 {
        let choice = source_choice(&gear, f.actors[0], "javelin-thrown", index);
        Box::pin(step(
            f,
            path,
            TableTransportChannel::Host,
            action(TacticalAction::CreatureWeaponAttack {
                feature_id: "javelin-thrown".into(),
                choice,
            }),
        ))
        .await;
        Box::pin(finish_source(
            f,
            path,
            "javelin-thrown",
            if index == 1 { 12 } else { 1 },
        ))
        .await;
        if index < 2 {
            Box::pin(next_source_turn(f, path)).await;
        }
    }
    let spent = state(f).await;
    assert!(
        gear.javelins
            .iter()
            .all(|id| matches!(spent.items[id].custody, Custody::Location(_)))
    );
    assert_eq!(
        spent
            .encounter
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .ground_items
            .iter()
            .filter(|g| gear.javelins.contains(&g.item))
            .count(),
        3
    );
    Box::pin(next_source_turn(f, path)).await;
    let choice = source_choice(&gear, f.actors[0], "javelin-thrown", 0);
    let forbidden = request(
        f,
        TableTransportChannel::Host,
        action(TacticalAction::CreatureWeaponAttack {
            feature_id: "javelin-thrown".into(),
            choice,
        }),
    )
    .await;
    Box::pin(rejected(f, forbidden)).await;
    Box::pin(step(
        f,
        path,
        TableTransportChannel::Host,
        action(TacticalAction::EndTurn),
    ))
    .await;
    let item = gear.javelins[0];
    let mut choice = throw_choice(item, gear.actor);
    choice.delivery = WeaponDelivery::Melee;
    choice.equipment_change = Some(AttackEquipmentChange {
        timing: EquipmentChangeTiming::BeforeAttack,
        operation: AttackEquipmentOperation::Pickup {
            item,
            hand: Hand::Right,
        },
    });
    Box::pin(player_step(
        f,
        path,
        action(TacticalAction::Attack { choice }),
    ))
    .await;
    let paid = state(f).await;
    assert_eq!(paid.items[&item].custody, Custody::Entity(f.actors[0]));
    assert_eq!(paid.items[&item].owner, initial.items[&item].owner);
    let attack = paid
        .encounter
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .resolution
        .as_ref()
        .unwrap()
        .attack
        .as_ref()
        .unwrap();
    assert!(matches!(attack.source, TacticalAttackSource::Weapon(_)));
    assert_eq!(attack.damage[0].dice, [DieSpec { count: 1, sides: 6 }]);
    assert_eq!(attack.damage[0].modifier, 3);
    Box::pin(player_raw(f, path, 12)).await;
    Box::pin(table_hit_driver::decline_hit_responses(f)).await;
    assert_eq!(
        view(f, &player(f)).await.roll.unwrap().dice,
        [DieSpec { count: 1, sides: 6 }]
    );
    Box::pin(player_raw(f, path, 1)).await;
    let complete = state(f).await;
    assert_eq!(complete.items.len(), initial.items.len());
    assert_eq!(
        complete.rules.as_ref().unwrap().entities[&gear.actor].hp,
        64
    );
}

async fn opportunity(f: &mut Fixture, path: &Path, feature: &str) {
    let gear = Box::pin(prepare(f, path)).await;
    Box::pin(player_step(f, path, action(TacticalAction::EndTurn))).await;
    let choice = source_choice(&gear, f.actors[0], feature, 0);
    let item = choice.weapon;
    let grip = choice.grip;
    Box::pin(step(
        f,
        path,
        TableTransportChannel::Host,
        action(TacticalAction::CreatureWeaponAttack {
            feature_id: feature.into(),
            choice,
        }),
    ))
    .await;
    Box::pin(finish_source(f, path, feature, 1)).await;
    Box::pin(step(
        f,
        path,
        TableTransportChannel::Host,
        action(TacticalAction::EndTurn),
    ))
    .await;
    Box::pin(player_step(
        f,
        path,
        action(TacticalAction::Move {
            path: vec![TacticalMoveStep {
                destination: SpatialPoint { x: 0, y: 10, z: 0 },
                mode: MovementMode::Walk,
            }],
        }),
    ))
    .await;
    let shown = view(f, &TableTransportChannel::Host)
        .await
        .tactical
        .unwrap()
        .opportunity
        .unwrap();
    assert_eq!(shown.actor, gear.actor);
    assert!(
        shown
            .physical_source_weapons
            .iter()
            .any(|offer| offer.item == item
                && offer.feature_id == feature
                && offer.grips.contains(&grip))
    );
    let before = state(f).await;
    for bad in [
        TacticalMeleeChoice::CreatureFeature {
            feature_id: feature.into(),
            weapon: Some(item),
        },
        TacticalMeleeChoice::CreatureWeapon {
            feature_id: "javelin-thrown".into(),
            weapon: item,
            grip,
        },
    ] {
        let forbidden = request(
            f,
            TableTransportChannel::Host,
            action(TacticalAction::OpportunityAttack { choice: bad }),
        )
        .await;
        Box::pin(rejected(f, forbidden)).await;
    }
    Box::pin(step(
        f,
        path,
        TableTransportChannel::Host,
        action(TacticalAction::OpportunityAttack {
            choice: TacticalMeleeChoice::CreatureWeapon {
                feature_id: feature.into(),
                weapon: item,
                grip,
            },
        }),
    ))
    .await;
    let paid = state(f).await;
    let timing = paid.rules.as_ref().unwrap().timing.as_ref().unwrap();
    assert_eq!(
        timing.action_spent,
        before
            .rules
            .as_ref()
            .unwrap()
            .timing
            .as_ref()
            .unwrap()
            .action_spent
    );
    assert_eq!(timing.reactions_spent, vec![gear.actor]);
    let attack = paid
        .encounter
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .resolution
        .as_ref()
        .unwrap()
        .attack
        .as_ref()
        .unwrap();
    let weapon = attack.weapon().unwrap();
    assert_eq!(weapon.window.kind, WeaponActionKind::Reaction);
    assert!(
        weapon.choice.equipment_change.is_none()
            && weapon.choice.after_equipment.is_none()
            && weapon.ground_pickup_before.is_none()
    );
    assert_eq!(paid.items, before.items);
    Box::pin(finish_source(f, path, feature, 20)).await;
    assert!(
        state(f)
            .await
            .encounter
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .resolution
            .is_none()
    );
}

async fn dropped_greatclub(f: &mut Fixture, path: &Path) {
    let gear = Box::pin(prepare(f, path)).await;
    Box::pin(player_step(f, path, action(TacticalAction::EndTurn))).await;
    let choice = source_choice(&gear, f.actors[0], "greatclub", 0);
    Box::pin(step(
        f,
        path,
        TableTransportChannel::Host,
        action(TacticalAction::CreatureWeaponAttack {
            feature_id: "greatclub".into(),
            choice,
        }),
    ))
    .await;
    Box::pin(finish_source(f, path, "greatclub", 1)).await;
    Box::pin(step(
        f,
        path,
        TableTransportChannel::Host,
        action(TacticalAction::EndTurn),
    ))
    .await;
    let initial = state(f).await;
    let dagger = initial
        .items
        .values()
        .find(|item| item.custody == Custody::Entity(f.actors[0]) && item.definition_id == "dagger")
        .unwrap()
        .id;
    for index in 0..7 {
        let choice = WeaponUseChoice {
            weapon: dagger,
            target: gear.actor,
            delivery: WeaponDelivery::Melee,
            ability: Ability::Strength,
            grip: WeaponGrip::OneHand(Hand::Right),
            purpose: WeaponAttackPurpose::Normal,
            ammunition: None,
            equipment_change: (index == 0).then_some(AttackEquipmentChange {
                timing: EquipmentChangeTiming::BeforeAttack,
                operation: AttackEquipmentOperation::Equip {
                    item: dagger,
                    hand: Hand::Right,
                },
            }),
            after_equipment: (index == 6).then_some(AfterAttackEquipmentIntent::Choose),
        };
        Box::pin(player_step(
            f,
            path,
            action(TacticalAction::Attack { choice }),
        ))
        .await;
        Box::pin(player_raw(f, path, 20)).await;
        Box::pin(table_hit_driver::decline_hit_responses(f)).await;
        assert_eq!(
            view(f, &player(f)).await.roll.unwrap().dice,
            [DieSpec { count: 2, sides: 4 }]
        );
        Box::pin(player_raw(f, path, 4)).await;
        if index < 6 {
            Box::pin(next_player_turn(f, path)).await;
        }
    }
    let at_choice = view(f, &player(f)).await.tactical.unwrap();
    assert_eq!(
        at_choice.attack_decision.unwrap().kind,
        TableAttackDecisionKind::Knockout
    );
    assert!(at_choice.attack_equipment.is_none());
    Box::pin(player_step(
        f,
        path,
        action(TacticalAction::ChooseAttackKnockout {
            choice: KnockoutChoice::KnockOut,
        }),
    ))
    .await;
    let dropped = state(f).await;
    assert_eq!(dropped.rules.as_ref().unwrap().entities[&gear.actor].hp, 1);
    assert!(matches!(
        dropped.items[&gear.greatclub].custody,
        Custody::Location(_)
    ));
    let shown = view(f, &player(f))
        .await
        .tactical
        .unwrap()
        .attack_equipment
        .unwrap();
    let operation = AttackEquipmentOperation::Unequip { item: dagger };
    assert!(
        shown
            .operations
            .iter()
            .any(|offer| offer.operation == operation)
    );
    Box::pin(player_step(
        f,
        path,
        TableTransportInput::AttackEquipment {
            handle: shown.key,
            choice: AttackEquipmentChoice::Apply(operation),
        },
    ))
    .await;
    Box::pin(next_player_turn(f, path)).await;
    Box::pin(player_step(
        f,
        path,
        action(TacticalAction::FirstAid {
            target: gear.actor,
            purpose: MedicinePurpose::EndKnockout,
        }),
    ))
    .await;
    Box::pin(player_raw(f, path, 20)).await;
    Box::pin(player_step(f, path, action(TacticalAction::EndTurn))).await;
    Box::pin(step(
        f,
        path,
        TableTransportChannel::Host,
        action(TacticalAction::StandProne),
    ))
    .await;
    Box::pin(step(
        f,
        path,
        TableTransportChannel::Host,
        action(TacticalAction::EndTurn),
    ))
    .await;
    let choice = WeaponUseChoice {
        weapon: gear.greatclub,
        target: gear.actor,
        delivery: WeaponDelivery::Melee,
        ability: Ability::Strength,
        grip: WeaponGrip::TwoHands,
        purpose: WeaponAttackPurpose::Normal,
        ammunition: None,
        equipment_change: Some(AttackEquipmentChange {
            timing: EquipmentChangeTiming::BeforeAttack,
            operation: AttackEquipmentOperation::Pickup {
                item: gear.greatclub,
                hand: Hand::Right,
            },
        }),
        after_equipment: None,
    };
    Box::pin(player_step(
        f,
        path,
        action(TacticalAction::Attack { choice }),
    ))
    .await;
    let paid = state(f).await;
    let attack = paid
        .encounter
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .resolution
        .as_ref()
        .unwrap()
        .attack
        .as_ref()
        .unwrap();
    assert_eq!(attack.damage[0].dice, [DieSpec { count: 1, sides: 8 }]);
    assert_eq!(attack.damage[0].modifier, 3);
    assert!(matches!(attack.source, TacticalAttackSource::Weapon(_)));
    assert_eq!(
        paid.items[&gear.greatclub].custody,
        Custody::Entity(f.actors[0])
    );
    assert_eq!(
        paid.items[&gear.greatclub].owner,
        initial.items[&gear.greatclub].owner
    );
    Box::pin(player_raw(f, path, 12)).await;
    Box::pin(table_hit_driver::decline_hit_responses(f)).await;
    assert_eq!(
        view(f, &player(f)).await.roll.unwrap().dice,
        [DieSpec { count: 1, sides: 8 }]
    );
    Box::pin(player_raw(f, path, 1)).await;
    Box::pin(player_step(
        f,
        path,
        action(TacticalAction::ChooseAttackKnockout {
            choice: KnockoutChoice::KnockOut,
        }),
    ))
    .await;
    let finished = state(f).await;
    assert_eq!(finished.items.len(), initial.items.len());
    assert_eq!(finished.items[&gear.greatclub].quantity, 1);
    assert!(
        finished
            .encounter
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .resolution
            .is_none()
    );
}

#[tokio::test]
async fn actual_ogre_three_printed_forms_cover_miss_hit_critical_and_cold_physical_requests() {
    for feature in ["greatclub", "javelin-melee", "javelin-thrown"] {
        for face in [1, 12, 20] {
            let directory =
                std::env::temp_dir().join(format!("dmd-ogre-form-{}", CampaignId::new().0));
            std::fs::create_dir_all(&directory).unwrap();
            let path = directory.join("campaign.sqlite");
            let pool = dmd_persistence::open_sqlite_path(&path).await.unwrap();
            let mut f = Box::pin(Fixture::with_pool(TableContract::default(), pool)).await;
            Box::pin(one_form(&mut f, &path, feature, face)).await;
            f.pool.close().await;
            drop(f);
            sqlite_test_cleanup::remove_closed_directory(&directory)
                .await
                .unwrap();
        }
    }
}

#[tokio::test]
async fn actual_three_javelins_are_finite_and_another_actor_recovers_same_item_for_ordinary_damage()
{
    let directory = std::env::temp_dir().join(format!("dmd-ogre-finite-{}", CampaignId::new().0));
    std::fs::create_dir_all(&directory).unwrap();
    let path = directory.join("campaign.sqlite");
    let pool = dmd_persistence::open_sqlite_path(&path).await.unwrap();
    let mut f = Box::pin(Fixture::with_pool(TableContract::default(), pool)).await;
    Box::pin(finite_and_recover(&mut f, &path)).await;
    f.pool.close().await;
    drop(f);
    sqlite_test_cleanup::remove_closed_directory(&directory)
        .await
        .unwrap();
}

#[tokio::test]
async fn actual_ogre_held_greatclub_and_javelin_melee_opportunities_pay_only_one_reaction() {
    for feature in ["greatclub", "javelin-melee"] {
        let directory = std::env::temp_dir().join(format!("dmd-ogre-oa-{}", CampaignId::new().0));
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join("campaign.sqlite");
        let pool = dmd_persistence::open_sqlite_path(&path).await.unwrap();
        let mut f = Box::pin(Fixture::with_pool(TableContract::default(), pool)).await;
        Box::pin(opportunity(&mut f, &path, feature)).await;
        f.pool.close().await;
        drop(f);
        sqlite_test_cleanup::remove_closed_directory(&directory)
            .await
            .unwrap();
    }
}

#[tokio::test]
async fn real_knockout_drop_and_first_aid_allow_another_actor_to_use_the_same_greatclub_for_ordinary_damage()
 {
    let directory = std::env::temp_dir().join(format!(
        "dmd-ogre-greatclub-recovery-{}",
        CampaignId::new().0
    ));
    std::fs::create_dir_all(&directory).unwrap();
    let path = directory.join("campaign.sqlite");
    let pool = dmd_persistence::open_sqlite_path(&path).await.unwrap();
    let mut creation = input("Physical recovery actor");
    creation.purchases.push(EquipmentChoice {
        item_id: "dagger".into(),
        quantity: 1,
    });
    let mut f = Box::pin(Fixture::with_creation_pool(
        TableContract::default(),
        Some(creation),
        pool,
    ))
    .await;
    Box::pin(dropped_greatclub(&mut f, &path)).await;
    f.pool.close().await;
    drop(f);
    sqlite_test_cleanup::remove_closed_directory(&directory)
        .await
        .unwrap();
}
