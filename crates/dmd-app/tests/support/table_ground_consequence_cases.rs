//! Real source Shield and later consequences before Ground equipment decisions.
//! Every positive state follows accepted application commands and physical dice.
use super::*;

#[path = "table_ground_concentration_cases.rs"]
mod concentration;

fn owner(f: &Fixture, mage: EntityId) -> TableTransportChannel {
    TableTransportChannel::SourceCreature {
        player_id: f.players[1],
        actor: mage,
    }
}
fn physical_input(view: TablePresentedView, values: &[u16]) -> TableTransportInput {
    let roll = view.roll.unwrap();
    let sides = if roll.mode == RollMode::Normal {
        roll.dice
            .iter()
            .flat_map(|die| std::iter::repeat_n(die.sides, usize::from(die.count)))
            .collect::<Vec<_>>()
    } else {
        vec![20, 20]
    };
    assert_eq!(sides.len(), values.len());
    action(TacticalAction::SubmitRoll {
        result: RollResult {
            request_id: roll.id,
            source: RollSource::Physical,
            dice: sides
                .into_iter()
                .zip(values)
                .map(|(sides, value)| DieResult {
                    sides,
                    value: *value,
                })
                .collect(),
        },
    })
}

async fn prepare(f: &mut Fixture, path: &Path) -> EntityId {
    // The shared fixture initially marks player 1 absent. Establish the actual
    // controllers' attendance through accepted session commands before combat.
    f.host(TableAction::EndSession, Some(f.session)).await;
    f.session = PlaySessionId::new();
    f.host(
        TableAction::StartSession {
            id: f.session,
            name: "Agreed practice encounter".into(),
            participants: (0..2)
                .map(|index| SessionParticipant {
                    player_id: f.players[index],
                    character_id: Some(f.characters[index]),
                    attendance: AttendanceStatus::Present,
                })
                .collect(),
        },
        Some(f.session),
    )
    .await;
    let host = view(f, &TableTransportChannel::Host).await;
    let catalog = f
        .runtime
        .table_creature_options(TableCreatureOptionsRequest {
            campaign_id: f.campaign,
            channel: TableTransportChannel::Host,
            revision: host.revision,
        })
        .await
        .unwrap();
    let source = catalog
        .iter()
        .find(|source| source.definition_id == "mage")
        .unwrap();
    let mage = EntityId::new();
    f.host(
        TableAction::CreateCreature {
            creation: Box::new(TableCreatureCreation {
                entity_id: mage,
                name: "Private source Mage".into(),
                definition_id: source.definition_id.clone(),
                source: source.source.clone(),
                size: CreatureSize::Medium,
                additional_languages: vec!["dwarvish".into(), "elvish".into(), "draconic".into()],
                ammunition_units: 0,
                item_ids: (0..source.item_count).map(|_| ItemId::new()).collect(),
            }),
        },
        Some(f.session),
    )
    .await;
    let count = host
        .characters
        .iter()
        .find(|character| character.character_id == f.characters[0])
        .unwrap()
        .equipment
        .as_ref()
        .unwrap()
        .initial_item_count;
    f.host(
        TableAction::PrepareEquipment {
            character_id: f.characters[0],
            item_ids: (0..count).map(|_| ItemId::new()).collect(),
        },
        Some(f.session),
    )
    .await;
    let p = |x, y| SpatialPoint { x, y, z: 0 };
    f.host(
        TableAction::PrepareBattlefield {
            setup: Box::new(TableBattlefieldSetup {
                encounter_id: EncounterId::new(),
                scene_id: SceneId::new(),
                location_id: LocationId::new(),
                name: "Stone courtyard".into(),
                battlefield: Battlefield {
                    bounds: SpatialBox {
                        min: p(0, 0),
                        max: SpatialPoint {
                            x: 100,
                            y: 100,
                            z: 40,
                        },
                    },
                    floor_z: 0,
                    floor_surface: "stone".into(),
                    ambient_light: LightLevel::Bright,
                    terrain: vec![],
                    obstacles: vec![],
                    lights: vec![],
                },
                characters: vec![TableCharacterPlacement {
                    character_id: f.characters[0],
                    position: p(10, 10),
                    height: 12,
                    allies: vec![],
                    enemies: vec![mage],
                }],
                creatures: vec![TableCreaturePlacement {
                    actor: mage,
                    public_label: "Spellcaster".into(),
                    position: p(20, 10),
                    height: 12,
                    allies: vec![],
                    enemies: vec![f.actors[0]],
                }],
                area_grid_policy: None,
                geometry_ruling: Ruling {
                    basis: RulingBasis::GmAdjudication,
                    reason: "Visible adjacent creatures in an open courtyard.".into(),
                },
            }),
        },
        Some(f.session),
    )
    .await;
    Box::pin(step(
        f,
        path,
        TableTransportChannel::Host,
        TableTransportInput::Action(Box::new(TableAction::EnableSourceActorAccess {
            adopted: vec![],
        })),
    ))
    .await;
    let controller = CreatureController::Player(f.players[1]);
    Box::pin(step(
        f,
        path,
        TableTransportChannel::Host,
        TableTransportInput::Action(Box::new(TableAction::SetSourceCreatureController {
            actor: mage,
            controller,
        })),
    ))
    .await;
    let actors = [f.actors[0], mage];
    Box::pin(step(
        f,
        path,
        TableTransportChannel::Host,
        action(TacticalAction::Begin {
            execution: TacticalExecutionVersion::EncounterReleaseV1,
            combatants: vec![
                TacticalCombatant {
                    actor: actors[0],
                    source: TacticalSource::Character,
                    surprised: false,
                },
                TacticalCombatant {
                    actor: mage,
                    source: TacticalSource::Creature {
                        definition_id: "mage".into(),
                    },
                    surprised: false,
                },
            ],
            groups: actors
                .into_iter()
                .map(|actor| InitiativeGroup {
                    actors: vec![actor],
                    request_id: RollRequestId::new(),
                })
                .collect(),
        }),
    ))
    .await;
    let pc = player(f);
    let mage_channel = owner(f, mage);
    let input = physical_input(view(f, &pc).await, &[18]);
    Box::pin(step(f, path, pc, input)).await;
    let input = physical_input(view(f, &mage_channel).await, &[2]);
    Box::pin(step(f, path, mage_channel, input)).await;
    assert_eq!(
        state(f).await.rules.as_ref().unwrap().entities[&mage].hp,
        81
    );
    Box::pin(step(
        f,
        path,
        TableTransportChannel::Host,
        action(TacticalAction::ActivateAttackEquipment),
    ))
    .await;
    mage
}

async fn no_after_card(f: &Fixture) {
    for channel in [player(f), TableTransportChannel::Host] {
        assert!(
            view(f, &channel)
                .await
                .tactical
                .unwrap()
                .attack_equipment
                .is_none(),
            "equipment cannot be selected before the real response or damage child"
        );
    }
}

async fn premature_after(f: &Fixture) {
    Box::pin(no_after_card(f)).await;
    let before = state(f).await;
    let resolution = before
        .encounter
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .resolution
        .as_ref()
        .unwrap();
    let queued = resolution.attack_after_equipment.as_ref().unwrap();
    assert!(queued.selected_by.is_none());
    let work = TacticalWorkKey {
        resolution: resolution.origin.id,
        occurrence: queued.work.occurrence,
    };
    for input in [
        action(TacticalAction::ChooseAttackEquipment {
            work,
            choice: AttackEquipmentChoice::Decline,
        }),
        TableTransportInput::AttackEquipment {
            handle: work.resolution,
            choice: AttackEquipmentChoice::Decline,
        },
        TableTransportInput::SelectWork {
            handle: work.resolution,
        },
    ] {
        let wrong = request(f, player(f), input).await;
        Box::pin(rejected(f, wrong)).await;
        assert_eq!(state(f).await, before);
    }
    Box::pin(hostile_populated_destination(f)).await;
}

async fn shield(f: &mut Fixture, path: &Path, mage: EntityId) -> CommandId {
    Box::pin(no_after_card(f)).await;
    let channel = owner(f, mage);
    let offered = view(f, &channel)
        .await
        .tactical
        .unwrap()
        .hit
        .unwrap()
        .response
        .unwrap();
    assert!(!offered.selected);
    assert_eq!(offered.shield.len(), 1);
    let wrong_kind = request(
        f,
        player(f),
        TableTransportInput::AttackEquipment {
            handle: offered.key,
            choice: AttackEquipmentChoice::Decline,
        },
    )
    .await;
    Box::pin(rejected(f, wrong_kind)).await;
    let accept = request(
        f,
        channel.clone(),
        TableTransportInput::HitResponse {
            handle: offered.key,
            decision: Box::new(TableHitInput::Respond { accept: true }),
        },
    )
    .await;
    // Arrival of another controller's private answer must not change this
    // player's complete DTO, revision or transcript, even with queued equipment.
    let unrelated = view(f, &player(f)).await;
    Box::pin(cold_step(f, path, accept)).await;
    assert_eq!(view(f, &player(f)).await, unrelated);
    Box::pin(no_after_card(f)).await;
    let ordering = view(f, &player(f))
        .await
        .tactical
        .unwrap()
        .hit
        .unwrap()
        .order
        .unwrap();
    Box::pin(player_step(
        f,
        path,
        TableTransportInput::HitResponse {
            handle: ordering.key,
            decision: Box::new(TableHitInput::Order {
                instruction: TacticalReactionOrdering {
                    ranked: vec![],
                    unlisted: ReactionUnlistedOrder::AfterForward,
                },
            }),
        },
    ))
    .await;
    let selected = view(f, &channel)
        .await
        .tactical
        .unwrap()
        .hit
        .unwrap()
        .response
        .unwrap();
    assert!(selected.selected);
    let choice = selected.shield[0].clone();
    assert_eq!(choice.spell_id, "shield");
    assert_eq!(
        choice.grant,
        SpellGrantChoice::CreatureFeature {
            feature_id: "protective-magic".into(),
        }
    );
    let cast = request(
        f,
        channel,
        TableTransportInput::HitResponse {
            handle: selected.key,
            decision: Box::new(TableHitInput::Cast { choice }),
        },
    )
    .await;
    for foreign in [player(f), TableTransportChannel::Host] {
        let mut stolen = cast.clone();
        stolen.command_id = CommandId::new();
        stolen.revision = view(f, &foreign).await.revision;
        stolen.channel = foreign;
        Box::pin(rejected(f, stolen)).await;
    }
    Box::pin(cold_step(f, path, cast.clone())).await;
    let after = state(f).await;
    let rules = after.rules.as_ref().unwrap();
    assert!(
        rules
            .timing
            .as_ref()
            .unwrap()
            .reactions_spent
            .contains(&mage)
    );
    assert_eq!(
        rules
            .tactical_creatures
            .as_ref()
            .unwrap()
            .runtime(mage)
            .unwrap()
            .limited_uses
            .iter()
            .find(|usage| usage.feature_id == "protective-magic" && usage.spell_id.is_none())
            .unwrap()
            .spent,
        1
    );
    assert_eq!(
        dmd_rules::tactical_defenses::effective_armor_class(&after, mage).unwrap(),
        17
    );
    cast.command_id
}

// The destination already has unrelated accepted history; a refusal must not
// remove or edit any existing row, including its projection and audit records.
async fn hostile_populated_destination(f: &Fixture) {
    let export = export_campaign(&f.pool, f.campaign).await.unwrap();
    let pool = open_sqlite("sqlite::memory:").await.unwrap();
    let unrelated = Box::pin(Fixture::with_pool(TableContract::default(), pool.clone())).await;
    assert_ne!(unrelated.campaign, f.campaign);
    let _ = view(&unrelated, &TableTransportChannel::Host).await;
    let before = all_destination_rows(&pool).await;
    for mutation in 0..3 {
        let mut changed = state(f).await;
        let flow = changed.encounter.as_mut().unwrap().flow.as_mut().unwrap();
        if mutation == 0 {
            flow.attack_equipment_access.as_mut().unwrap().origin.id = CommandId::new();
        } else if mutation == 1 {
            flow.attack_equipment_access = None;
        } else {
            flow.attack_equipment_access
                .as_mut()
                .unwrap()
                .origin
                .campaign_id = unrelated.campaign;
        }
        let mut forged = export.clone();
        forged.current_state.state_json = changed.encode_json().unwrap();
        forged.snapshots.retain(|row| {
            row.event_sequence != i64::try_from(changed.applied_event_sequence).unwrap()
        });
        forged.snapshots.push(dmd_persistence::SnapshotRow {
            campaign_id: forged.campaign_id.clone(),
            event_sequence: i64::try_from(changed.applied_event_sequence).unwrap(),
            state_schema_version: i64::from(changed.schema_version),
            state_json: changed.encode_json().unwrap(),
            created_at_utc: forged.exported_at_utc.clone(),
        });
        forged.snapshots.sort_by_key(|row| row.event_sequence);
        assert!(
            Box::pin(runtime(pool.clone()).restore_campaign(&forged))
                .await
                .is_err()
        );
        assert_eq!(all_destination_rows(&pool).await, before);
    }
    pool.close().await;
}

async fn shield_case(
    f: &mut Fixture,
    path: &Path,
    before_pickup: bool,
    critical: bool,
    apply: bool,
) {
    let mage = Box::pin(prepare(f, path)).await;
    let initial = state(f).await;
    let item = initial
        .items
        .values()
        .find(|i| i.custody == Custody::Entity(f.actors[0]) && i.definition_id == "dagger")
        .unwrap()
        .clone();
    let mut setup_attack = throw_choice(item.id, mage);
    if !before_pickup {
        setup_attack.delivery = WeaponDelivery::Melee;
        setup_attack.equipment_change = Some(AttackEquipmentChange {
            timing: EquipmentChangeTiming::BeforeAttack,
            operation: AttackEquipmentOperation::Equip {
                item: item.id,
                hand: Hand::Right,
            },
        });
    }
    Box::pin(player_step(
        f,
        path,
        action(TacticalAction::Attack {
            choice: setup_attack,
        }),
    ))
    .await;
    Box::pin(player_raw(f, path, 1)).await;
    Box::pin(player_step(f, path, action(TacticalAction::EndTurn))).await;
    let mage_channel = owner(f, mage);
    Box::pin(step(f, path, mage_channel, action(TacticalAction::EndTurn))).await;
    let ready = state(f).await;
    assert_eq!(ready.rules.as_ref().unwrap().entities[&mage].hp, 81);
    assert_eq!(
        ready.items[&item.id].custody == Custody::Entity(f.actors[0]),
        !before_pickup
    );
    let mut choice = throw_choice(item.id, mage);
    choice.delivery = WeaponDelivery::Melee;
    if before_pickup {
        choice.equipment_change = Some(AttackEquipmentChange {
            timing: EquipmentChangeTiming::BeforeAttack,
            operation: AttackEquipmentOperation::Pickup {
                item: item.id,
                hand: Hand::Right,
            },
        });
    } else {
        choice.after_equipment = Some(AfterAttackEquipmentIntent::Choose);
    }
    let attack = Box::pin(player_step(
        f,
        path,
        action(TacticalAction::Attack { choice }),
    ))
    .await;
    let paid = state(f).await;
    assert_eq!(paid.items.len(), initial.items.len());
    assert_eq!(paid.items[&item.id].owner, item.owner);
    assert_eq!(paid.items[&item.id].custody, Custody::Entity(f.actors[0]));
    assert!(
        paid.rules
            .as_ref()
            .unwrap()
            .timing
            .as_ref()
            .unwrap()
            .action_spent
    );
    Box::pin(hostile_populated_destination(f)).await;
    let face = if critical { 20 } else { 10 };
    let input = physical_input(view(f, &player(f)).await, &[face]);
    let hit = Box::pin(player_step(f, path, input)).await;
    let response = Box::pin(shield(f, path, mage)).await;
    let shielded = state(f).await;
    assert_eq!(shielded.rules.as_ref().unwrap().entities[&mage].hp, 81);
    if critical {
        Box::pin(no_after_card(f)).await;
        let pending = shielded.rules.as_ref().unwrap().pending.as_ref().unwrap();
        assert_eq!(pending.issued_by.id, hit.command_id);
        assert_ne!(pending.issued_by.id, response);
        assert_eq!(pending.request.mode, RollMode::Normal);
        assert_eq!(pending.request.dice, vec![DieSpec { count: 2, sides: 4 }]);
        assert_eq!(pending.request.modifier, 3);
        Box::pin(hostile_populated_destination(f)).await;
        let input = physical_input(view(f, &player(f)).await, &[4, 4]);
        Box::pin(player_step(f, path, input)).await;
    } else {
        assert!(shielded.rules.as_ref().unwrap().pending.is_none());
    }
    let completed_damage = state(f).await;
    assert_eq!(
        completed_damage.rules.as_ref().unwrap().entities[&mage].hp,
        if critical { 70 } else { 81 }
    );
    if before_pickup {
        Box::pin(no_after_card(f)).await;
        assert!(
            completed_damage
                .encounter
                .as_ref()
                .unwrap()
                .flow
                .as_ref()
                .unwrap()
                .resolution
                .is_none()
        );
        assert_eq!(completed_damage.items[&item.id].owner, item.owner);
        assert_eq!(
            completed_damage.items[&item.id].custody,
            Custody::Entity(f.actors[0])
        );
        assert_eq!(completed_damage.items.len(), initial.items.len());
        return;
    }
    let selected = view(f, &player(f))
        .await
        .tactical
        .unwrap()
        .attack_equipment
        .unwrap();
    let resolution = completed_damage
        .encounter
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .resolution
        .as_ref()
        .unwrap();
    assert!(resolution.attack.is_none());
    assert_eq!(
        resolution
            .attack_after_equipment
            .as_ref()
            .unwrap()
            .cause
            .origin
            .id,
        attack.command_id
    );
    let operation = AttackEquipmentOperation::Unequip { item: item.id };
    assert!(
        selected
            .operations
            .iter()
            .any(|offer| offer.operation == operation)
    );
    Box::pin(hostile_populated_destination(f)).await;
    Box::pin(hostile_capabilities(f, selected.key)).await;
    let mage_view = view(f, &owner(f, mage)).await;
    assert!(
        mage_view
            .tactical
            .as_ref()
            .unwrap()
            .attack_equipment
            .is_none()
    );
    let finish = Box::pin(player_step(
        f,
        path,
        TableTransportInput::AttackEquipment {
            handle: selected.key,
            choice: if apply {
                AttackEquipmentChoice::Apply(operation)
            } else {
                AttackEquipmentChoice::Decline
            },
        },
    ))
    .await;
    assert_eq!(view(f, &owner(f, mage)).await, mage_view);
    let final_state = state(f).await;
    assert!(
        final_state
            .encounter
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .resolution
            .is_none()
    );
    assert_eq!(final_state.items, completed_damage.items);
    assert_eq!(
        final_state.rules.as_ref().unwrap().rolls,
        completed_damage.rules.as_ref().unwrap().rolls
    );
    assert_eq!(
        final_state.rules.as_ref().unwrap().timing,
        completed_damage.rules.as_ref().unwrap().timing
    );
    let hands = view(f, &player(f))
        .await
        .tactical
        .unwrap()
        .attack_options
        .unwrap()
        .hands;
    assert_eq!(hands.hands.contains(&HandAssignment::Item(item.id)), !apply);
    Box::pin(hostile_envelope(f, finish.command_id)).await;
    Box::pin(hostile_populated_destination(f)).await;
}

#[tokio::test]
async fn genuine_owned_shield_finishes_before_same_item_pickup_or_after_equipment_with_cold_replay()
{
    for (before_pickup, critical, apply) in [
        (true, false, false),
        (true, true, false),
        (false, false, false),
        (false, false, true),
        (false, true, false),
        (false, true, true),
    ] {
        let directory =
            std::env::temp_dir().join(format!("dmd-ground-shield-{}", CampaignId::new().0));
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join("campaign.sqlite");
        let pool = dmd_persistence::open_sqlite_path(&path).await.unwrap();
        let mut creation = input("Equipment actor");
        creation.purchases.push(EquipmentChoice {
            item_id: "dagger".into(),
            quantity: 1,
        });
        let contract = TableContract {
            pvp_policy: "Both players consent to the practice encounter and controlled Mage."
                .into(),
            ..TableContract::default()
        };
        let mut f = Box::pin(Fixture::with_creation_pool(contract, Some(creation), pool)).await;
        Box::pin(shield_case(&mut f, &path, before_pickup, critical, apply)).await;
        f.pool.close().await;
        drop(f);
        sqlite_test_cleanup::remove_closed_directory(&directory)
            .await
            .unwrap();
    }
}
