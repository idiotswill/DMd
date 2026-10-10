//! External authored draft: uncompiled/unrun; receiver needs Grapple + release5.
//! Positive states use immutable original capture and real production commands.
//! Zero new Grapple authority: source coexistence is not Grapple admission proof.
use super::*;
use dmd_rules::tactical_creatures::{
    creature_definition, creature_source_pin, source_for_actor, source_for_profile,
};
use dmd_rules::tactical_definitions::bundled_goblin_warrior_v2;
use std::collections::BTreeMap;

const ORIGINAL: &str = include_str!("../fixtures/reactions-v1-upgrade-100c7da.json");

fn image(export: &CampaignExport) -> Box<CampaignState> {
    Box::new(CampaignState::decode_json(&export.current_state.state_json).unwrap())
}

fn profile(state: &CampaignState, actor: EntityId) -> &CreatureProfile {
    state
        .rules
        .as_ref()
        .unwrap()
        .tactical_creatures
        .as_ref()
        .unwrap()
        .profile(actor)
        .unwrap()
}

fn loadout(state: &CampaignState, actor: EntityId) -> &ActorEquipmentLoadout {
    state
        .rules
        .as_ref()
        .unwrap()
        .tactical_inventory
        .as_ref()
        .unwrap()
        .loadout(actor)
        .unwrap()
}

fn tactical(action: TacticalAction) -> TableAction {
    TableAction::Tactical { action }
}

async fn view(f: &Fixture, channel: &TableTransportChannel) -> TablePresentedView {
    let viewer = match channel {
        TableTransportChannel::Host => TableViewer::Host,
        TableTransportChannel::Player { player_id, .. }
        | TableTransportChannel::SourceCreature { player_id, .. } => {
            TableViewer::Player(*player_id)
        }
    };
    f.app
        .presented_table_view(f.campaign, viewer)
        .await
        .unwrap()
}

async fn request(
    f: &Fixture,
    channel: TableTransportChannel,
    action: TableAction,
) -> TableTransportRequest {
    let shown = view(f, &channel).await;
    let activation = matches!(&action, TableAction::EnableSourceActorAccess { .. });
    TableTransportRequest {
        version: if shown.source_control.is_some() || activation {
            TABLE_SOURCE_TRANSPORT_VERSION
        } else {
            TABLE_TRANSPORT_VERSION
        },
        command_id: CommandId::new(),
        campaign_id: f.campaign,
        session_id: shown.active_session.as_ref().map(|s| s.session_id),
        revision: shown.revision,
        channel,
        input: TableTransportInput::Action(Box::new(action)),
    }
}

fn assert_prefix(original: &CampaignExport, accepted: &CampaignExport) {
    assert!(accepted.event_journal.starts_with(&original.event_journal));
    assert!(
        accepted
            .table_projection_history
            .starts_with(&original.table_projection_history)
    );
    for row in &original.command_audit {
        assert!(accepted.command_audit.contains(row));
    }
    for row in &original.event_causes {
        assert!(accepted.event_causes.contains(row));
    }
    for row in &original.snapshots {
        assert!(accepted.snapshots.contains(row));
    }
    for row in &original.table_transport_bindings {
        assert!(accepted.table_transport_bindings.contains(row));
    }
    assert_eq!(accepted.play_sessions, original.play_sessions);
    assert_eq!(
        accepted.play_session_participants,
        original.play_session_participants
    );
    assert_eq!(accepted.observations, original.observations);
}

fn assert_zero_grapple(state: &CampaignState) {
    assert!(
        !has_unimplemented_grapple_records(state),
        "this proof has no Grapple producer or authority"
    );
}

async fn accept(
    f: &mut Fixture,
    channel: TableTransportChannel,
    action: TableAction,
) -> TableTransportRequest {
    let request = request(f, channel, action).await;
    Box::pin(f.accept_cold(request.clone())).await;
    let accepted = export_campaign(&f.pool, f.campaign).await.unwrap();
    assert_prefix(&f.original, &accepted);
    assert_zero_grapple(&image(&accepted));
    request
}

async fn host(f: &mut Fixture, action: TableAction) -> TableTransportRequest {
    Box::pin(accept(f, TableTransportChannel::Host, action)).await
}

async fn reject(f: &Fixture, channel: TableTransportChannel, action: TableAction) {
    // Current view comes before the parent's whole-export no-write baseline.
    let request = request(f, channel, action).await;
    Box::pin(f.reject(request)).await;
}

async fn physical_d20(f: &mut Fixture, channel: TableTransportChannel, actor: EntityId, face: u16) {
    let shown = view(f, &channel).await;
    let roll = shown.roll.as_ref().unwrap();
    assert_eq!(roll.roller, Some(actor));
    assert_eq!(roll.mode, RollMode::Normal);
    assert_eq!(
        roll.dice,
        [DieSpec {
            count: 1,
            sides: 20
        }]
    );
    Box::pin(accept(
        f,
        channel,
        tactical(TacticalAction::SubmitRoll {
            result: RollResult {
                request_id: roll.id,
                source: RollSource::Physical,
                dice: vec![DieResult {
                    sides: 20,
                    value: face,
                }],
            },
        }),
    ))
    .await;
}

async fn retry_original_creation(f: &Fixture, old: &CreatureProfile) {
    let event = f
        .original
        .event_journal
        .iter()
        .find(|e| e.command_id == old.origin.id.0.to_string())
        .unwrap();
    let saved: TableEvent = serde_json::from_str(&event.payload_json).unwrap();
    let TableAction::CreateCreature { creation } = &saved.action else {
        panic!("original source creation missing")
    };
    assert_eq!(creation.entity_id, old.actor);
    assert!(
        creation.source.is_none(),
        "historical absent pin is never rewritten"
    );
    assert_eq!(saved.meta, old.origin);
    let before = export_campaign(&f.pool, f.campaign).await.unwrap();
    let receipt = Box::pin(f.app.execute_table(saved.meta, saved.action))
        .await
        .unwrap();
    assert!(receipt.already_accepted);
    assert_eq!(receipt.command_id, old.origin.id);
    assert_eq!(
        receipt.event_sequence,
        u64::try_from(event.sequence).unwrap()
    );
    assert_eq!(receipt.outcome, saved.outcome);
    f.assert_export(&before).await;
}

async fn current_creation(f: &Fixture) -> TableCreatureCreation {
    let shown = view(f, &TableTransportChannel::Host).await;
    let options = f
        .app
        .table_creature_options(TableCreatureOptionsRequest {
            campaign_id: f.campaign,
            channel: TableTransportChannel::Host,
            revision: shown.revision,
        })
        .await
        .unwrap();
    let options: Vec<_> = options
        .iter()
        .filter(|o| o.definition_id == "goblin-warrior")
        .collect();
    assert_eq!(options.len(), 1);
    let option = options[0];
    let pin = creature_source_pin(bundled_goblin_warrior_v2().unwrap()).unwrap();
    assert_eq!(option.source.as_ref(), Some(&pin));
    assert_eq!(option.sizes, [CreatureSize::Small]);
    assert_eq!(option.additional_languages, 0);
    assert!(option.ammunition_required);
    assert_eq!(option.item_count, 5);
    TableCreatureCreation {
        entity_id: EntityId::new(),
        name: "Private current Goblin revision".into(),
        definition_id: option.definition_id.clone(),
        source: option.source.clone(),
        size: CreatureSize::Small,
        additional_languages: vec![],
        ammunition_units: 20,
        item_ids: (0..option.item_count).map(|_| ItemId::new()).collect(),
    }
}

fn creation(creation: TableCreatureCreation) -> TableAction {
    TableAction::CreateCreature {
        creation: Box::new(creation),
    }
}

fn assert_pair(state: &CampaignState, old: &CreatureProfile, new: &CreatureProfile) {
    assert_eq!(profile(state, old.actor), old);
    assert_eq!(profile(state, new.actor), new);
    assert_ne!(old.actor, new.actor);
    assert_ne!(old.origin.id, new.origin.id);
    assert_eq!(old.origin.campaign_id, new.origin.campaign_id);
    let v1 = creature_definition("goblin-warrior").unwrap();
    let v2 = bundled_goblin_warrior_v2().unwrap();
    assert_eq!(old.source.definition_fingerprint, "b1b3d06ef838810e");
    assert_eq!(old.source, creature_source_pin(v1).unwrap());
    assert_eq!(new.source, creature_source_pin(v2).unwrap());
    assert_ne!(old.source, new.source);
    assert_eq!(source_for_profile(profile(state, old.actor)).unwrap(), v1);
    assert_eq!(source_for_profile(profile(state, new.actor)).unwrap(), v2);
    assert_eq!(
        source_for_actor(state, old.actor, "goblin-warrior").unwrap(),
        v1
    );
    assert_eq!(
        source_for_actor(state, new.actor, "goblin-warrior").unwrap(),
        v2
    );
    assert_eq!(v1.ordinary_hands, None);
    assert_eq!(v2.ordinary_hands, Some(OrdinaryHandAnatomy::TwoHandsV1));
    let mut comparison_only = v2.clone();
    comparison_only.ordinary_hands = None;
    assert_eq!(
        &comparison_only, v1,
        "only the reviewed immutable anatomy annotation differs"
    );
    assert_eq!(
        state
            .rules
            .as_ref()
            .unwrap()
            .tactical_creatures
            .as_ref()
            .unwrap()
            .profiles
            .len(),
        2
    );
    assert_zero_grapple(state);
}

fn item(state: &CampaignState, actor: EntityId, definition: &str) -> ItemId {
    let items: Vec<_> = state
        .items
        .values()
        .filter(|i| i.definition_id == definition && i.custody == Custody::Entity(actor))
        .collect();
    assert_eq!(items.len(), 1);
    items[0].id
}

fn assert_original_paid_state(state: &CampaignState, original: &CampaignState) {
    assert_eq!(
        state.clock, original.clock,
        "no proposed transition wraps a round or advances time"
    );
    for (id, mechanics) in &original.rules.as_ref().unwrap().entities {
        assert_eq!(
            &state.rules.as_ref().unwrap().entities[id],
            mechanics,
            "no healing, resource reset, or fabricated mechanics"
        );
    }
    for (id, original_item) in &original.items {
        assert_eq!(&state.items[id], original_item);
    }
    assert!(
        state
            .rules
            .as_ref()
            .unwrap()
            .rolls
            .starts_with(&original.rules.as_ref().unwrap().rolls)
    );
    assert_eq!(
        state
            .rules
            .as_ref()
            .unwrap()
            .tactical_inventory
            .as_ref()
            .unwrap()
            .receipts,
        original
            .rules
            .as_ref()
            .unwrap()
            .tactical_inventory
            .as_ref()
            .unwrap()
            .receipts
    );
    assert_zero_grapple(state);
}

async fn enable_and_assign(f: &mut Fixture, actors: [EntityId; 2]) {
    let shown = view(f, &TableTransportChannel::Host).await;
    assert!(shown.source_control.is_none());
    let options = f
        .app
        .table_source_control_options(TableCreatureOptionsRequest {
            campaign_id: f.campaign,
            channel: TableTransportChannel::Host,
            revision: shown.revision,
        })
        .await
        .unwrap();
    assert!(!options.enabled && options.settled && options.adopted.is_empty());
    Box::pin(host(
        f,
        TableAction::EnableSourceActorAccess {
            adopted: options.adopted,
        },
    ))
    .await;
    for actor in actors {
        Box::pin(host(
            f,
            TableAction::SetSourceCreatureController {
                actor,
                controller: CreatureController::Host,
            },
        ))
        .await;
    }
}

async fn second_scene(
    f: &mut Fixture,
    old: &CreatureProfile,
    new: &CreatureProfile,
    pc_actor: EntityId,
) {
    let before = f.state().await;
    let released = before
        .encounter_history
        .as_ref()
        .unwrap()
        .last()
        .unwrap()
        .clone();
    let pc = before
        .characters
        .values()
        .find(|pc| pc.entity_id == pc_actor)
        .unwrap();
    let point = |x, y| SpatialPoint { x, y, z: 0 };
    let setup = TableBattlefieldSetup {
        encounter_id: EncounterId::new(),
        scene_id: SceneId::new(),
        location_id: released.location_id,
        name: "A later scene with both Goblin revisions".into(),
        battlefield: Battlefield {
            bounds: SpatialBox {
                min: point(0, 0),
                max: SpatialPoint {
                    x: 120,
                    y: 100,
                    z: 60,
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
            character_id: pc.id,
            position: point(10, 10),
            height: 12,
            allies: vec![],
            enemies: vec![],
        }],
        creatures: vec![
            TableCreaturePlacement {
                actor: old.actor,
                public_label: "Earlier Goblin".into(),
                position: point(40, 30),
                height: 6,
                allies: vec![],
                enemies: vec![new.actor],
            },
            TableCreaturePlacement {
                actor: new.actor,
                public_label: "Later Goblin".into(),
                position: point(50, 30),
                height: 6,
                allies: vec![],
                enemies: vec![old.actor],
            },
        ],
        geometry_ruling: Ruling {
            basis: RulingBasis::GmAdjudication,
            reason: "Host places two adjacent nonoverlapping Small bodies on a bright open grid."
                .into(),
        },
        area_grid_policy: None,
    };
    let encounter_id = setup.encounter_id;
    Box::pin(host(
        f,
        TableAction::PrepareBattlefield {
            setup: Box::new(setup),
        },
    ))
    .await;
    let combatants = vec![
        TacticalCombatant {
            actor: old.actor,
            source: TacticalSource::Creature {
                definition_id: "goblin-warrior".into(),
            },
            surprised: false,
        },
        TacticalCombatant {
            actor: new.actor,
            source: TacticalSource::Creature {
                definition_id: "goblin-warrior".into(),
            },
            surprised: false,
        },
        TacticalCombatant {
            actor: pc_actor,
            source: TacticalSource::Character,
            surprised: false,
        },
    ];
    let prepared = f.state().await;
    for combatant in &combatants[..2] {
        assert_eq!(
            dmd_rules::tactical::preview_initiative_circumstances(&prepared, combatant).unwrap(),
            (2, RollMode::Normal)
        );
    }
    Box::pin(host(
        f,
        tactical(TacticalAction::Begin {
            execution: TacticalExecutionVersion::EncounterReleaseV1,
            combatants,
            groups: vec![
                InitiativeGroup {
                    actors: vec![old.actor, new.actor],
                    request_id: RollRequestId::new(),
                },
                InitiativeGroup {
                    actors: vec![pc_actor],
                    request_id: RollRequestId::new(),
                },
            ],
        }),
    ))
    .await;
    Box::pin(physical_d20(f, TableTransportChannel::Host, old.actor, 20)).await;
    let (_, pc_channel) = owner(&f.state().await, pc_actor);
    Box::pin(physical_d20(f, pc_channel, pc_actor, 1)).await;
    let tied = f.state().await;
    assert!(
        matches!(&flow(&tied).phase, TacticalPhase::InitiativeTies { ties }
        if ties.len() == 1 && ties[0].total == 22 && ties[0].actors.len() == 2
            && ties[0].actors.contains(&old.actor) && ties[0].actors.contains(&new.actor))
    );
    Box::pin(host(
        f,
        tactical(TacticalAction::ProposeInitiativeTie {
            order: vec![old.actor, new.actor],
        }),
    ))
    .await;
    let active = f.state().await;
    assert_eq!(flow(&active).version, 5);
    assert_eq!(flow(&active).phase, TacticalPhase::Active);
    assert_eq!(active.encounter.as_ref().unwrap().id, encounter_id);
    assert_eq!(
        active.scenes[&released.scene_id].status,
        SceneStatus::Closed
    );
    assert_eq!(
        active
            .rules
            .as_ref()
            .unwrap()
            .timing
            .as_ref()
            .unwrap()
            .turn_number,
        released.final_turn.number + 1
    );
    assert_eq!(
        view(f, &TableTransportChannel::Host)
            .await
            .tactical
            .unwrap()
            .active_actor,
        Some(old.actor)
    );
    assert_pair(&active, old, new);
}

async fn scimitar_miss(
    f: &mut Fixture,
    attacker: &CreatureProfile,
    target: &CreatureProfile,
) -> CreatureSourcePin {
    let before = Box::new(f.state().await);
    let original_loadout = loadout(&before, attacker.actor).clone();
    let scimitar = item(&before, attacker.actor, "scimitar");
    let shield = item(&before, attacker.actor, "shield");
    assert_eq!(original_loadout.shield, Some(shield));
    assert_eq!(
        original_loadout.hands.hands,
        [HandAssignment::Item(shield), HandAssignment::Free]
    );
    assert_eq!(
        view(f, &TableTransportChannel::Host)
            .await
            .tactical
            .unwrap()
            .active_actor,
        Some(attacker.actor)
    );
    let accepted = Box::pin(host(
        f,
        tactical(TacticalAction::CreatureWeaponAttack {
            feature_id: "scimitar".into(),
            choice: CreatureWeaponUseChoice {
                after_equipment: None,
                weapon: scimitar,
                target: target.actor,
                grip: WeaponGrip::OneHand(Hand::Right),
                ammunition: None,
                equipment_change: Some(AttackEquipmentChange {
                    timing: EquipmentChangeTiming::BeforeAttack,
                    operation: AttackEquipmentOperation::Equip {
                        item: scimitar,
                        hand: Hand::Right,
                    },
                }),
            },
        }),
    ))
    .await;
    let pending = Box::new(f.state().await);
    let attack = flow(&pending)
        .resolution
        .as_ref()
        .unwrap()
        .attack
        .as_ref()
        .unwrap();
    assert_eq!(attack.origin.id, accepted.command_id);
    assert_eq!(attack.actor, attacker.actor);
    assert_eq!(attack.target, target.actor);
    assert_eq!(attack.attack_modifier, 4);
    assert_eq!(attack.mode, RollMode::Normal);
    assert_eq!(attack.stage, TacticalAttackStage::AttackRoll);
    assert_eq!(
        attack.damage,
        [AttackDamageComponent {
            damage_type: DamageType::Slashing,
            dice: vec![DieSpec { count: 1, sides: 6 }],
            modifier: 2
        }]
    );
    let TacticalAttackSource::CreatureWeapon {
        source,
        feature_id,
        weapon,
    } = &attack.source
    else {
        panic!("ordinary item-only attack would not prove full creature source lookup")
    };
    assert_eq!(source, &attacker.source);
    assert_ne!(source, &target.source);
    assert_eq!(feature_id, "scimitar");
    assert_eq!(weapon.equipment_before, original_loadout);
    assert_eq!(weapon.choice.ability, Ability::Dexterity);
    assert_eq!(weapon.choice.weapon, scimitar);
    assert!(weapon.ammunition.is_none());
    assert_eq!(
        loadout(&pending, attacker.actor).hands.hands,
        [HandAssignment::Item(shield), HandAssignment::Item(scimitar)]
    );
    assert_eq!(loadout(&pending, attacker.actor).shield, Some(shield));
    assert_eq!(pending.items, before.items);
    assert!(
        pending
            .rules
            .as_ref()
            .unwrap()
            .timing
            .as_ref()
            .unwrap()
            .action_spent
    );
    assert_eq!(flow(&pending).budget.attacks_remaining, 0);
    let receipt = flow(&pending).budget.weapon_history.last().unwrap().clone();
    assert_eq!(receipt.origin.id, accepted.command_id);
    assert_eq!(receipt.actor, attacker.actor);
    assert_eq!(receipt.weapon, scimitar);
    assert_eq!(receipt.purpose, WeaponAttackPurpose::Normal);
    assert_eq!(receipt.outcome, WeaponAttackOutcome::Pending);
    let canonical_roll = pending
        .rules
        .as_ref()
        .unwrap()
        .pending
        .as_ref()
        .unwrap()
        .request
        .id;
    let source = source.clone();
    // accept_cold already restored/reopened the actual paid pending source attack.
    Box::pin(physical_d20(
        f,
        TableTransportChannel::Host,
        attacker.actor,
        1,
    ))
    .await;
    let after = Box::new(f.state().await);
    assert!(flow(&after).resolution.is_none());
    assert!(after.rules.as_ref().unwrap().pending.is_none());
    let recorded = after.rules.as_ref().unwrap().rolls.last().unwrap();
    assert_eq!(recorded.request.id, canonical_roll);
    assert_eq!(recorded.request.mode, RollMode::Normal);
    assert_eq!(recorded.request.modifier, 4);
    assert_eq!(recorded.result.source, RollSource::Physical);
    assert_eq!(
        recorded.result.dice,
        [DieResult {
            sides: 20,
            value: 1
        }]
    );
    let mut expected = receipt;
    expected.outcome = WeaponAttackOutcome::Miss;
    assert_eq!(
        flow(&after).budget.weapon_history.last().unwrap(),
        &expected
    );
    assert_eq!(after.items, before.items);
    assert_eq!(
        after.rules.as_ref().unwrap().entities,
        before.rules.as_ref().unwrap().entities
    );
    assert!(
        after
            .rules
            .as_ref()
            .unwrap()
            .timing
            .as_ref()
            .unwrap()
            .action_spent
    );
    assert_eq!(flow(&after).budget.attacks_remaining, 0);
    source
}

// Negative restore helpers and final original-receipt recovery follow below.

async fn retry_created(f: &Fixture, request: &TableTransportRequest) {
    let before = export_campaign(&f.pool, f.campaign).await.unwrap();
    let binding = before
        .table_transport_bindings
        .iter()
        .find(|binding| binding.meta.id == request.command_id)
        .unwrap();
    assert_eq!(
        serde_json::to_string(request).unwrap(),
        binding.request_json
    );
    let expected: TableTransportResult = serde_json::from_str(&binding.response_json).unwrap();
    assert_eq!(
        Box::pin(f.app.submit_presented_table(request.clone()))
            .await
            .unwrap(),
        expected
    );
    f.assert_export(&before).await;
}

async fn destination_rows(pool: &sqlx::SqlitePool) -> BTreeMap<String, i64> {
    let names: Vec<String> =
        sqlx::query_scalar("SELECT name FROM sqlite_master WHERE type = 'table' ORDER BY name")
            .fetch_all(pool)
            .await
            .unwrap();
    let mut rows = BTreeMap::new();
    for name in names {
        let quoted = name.replace('"', "\"\"");
        let count: i64 = sqlx::query_scalar(&format!("SELECT COUNT(*) FROM \"{quoted}\""))
            .fetch_one(pool)
            .await
            .unwrap();
        rows.insert(name, count);
    }
    rows
}

// The extra snapshot is an exact earlier accepted current-state row. A positive
// restore proves that retaining it is legal before any adversarial copy changes.
async fn reject_revision_swaps(
    f: &Fixture,
    old: &CreatureProfile,
    new: &CreatureProfile,
    earlier: &dmd_persistence::CurrentStateRow,
) {
    let original = export_campaign(&f.pool, f.campaign).await.unwrap();
    assert!(earlier.applied_event_sequence < original.current_state.applied_event_sequence);
    let earlier_state = Box::new(CampaignState::decode_json(&earlier.state_json).unwrap());
    assert_eq!(profile(&earlier_state, old.actor), old);
    assert!(!earlier_state.entities.contains_key(&new.actor));
    let mut genuine = original.clone();
    if let Some(snapshot) = genuine
        .snapshots
        .iter()
        .find(|row| row.event_sequence == earlier.applied_event_sequence)
    {
        assert_eq!(snapshot.campaign_id, earlier.campaign_id);
        assert_eq!(snapshot.state_schema_version, earlier.schema_version);
        assert_eq!(snapshot.state_json, earlier.state_json);
    } else {
        genuine.snapshots.push(dmd_persistence::SnapshotRow {
            campaign_id: earlier.campaign_id.clone(),
            event_sequence: earlier.applied_event_sequence,
            state_schema_version: earlier.schema_version,
            state_json: earlier.state_json.clone(),
            created_at_utc: original.exported_at_utc.clone(),
        });
        genuine.snapshots.sort_by_key(|row| row.event_sequence);
    }
    for snapshot in &original.snapshots {
        assert!(
            genuine.snapshots.contains(snapshot),
            "never replace an original snapshot"
        );
    }
    let positive_path = f.directory.join("genuine-earlier-snapshot.sqlite");
    let positive_pool = open_sqlite_path(&positive_path).await.unwrap();
    let positive_app = runtime(positive_pool.clone());
    assert_eq!(
        Box::pin(positive_app.restore_campaign(&genuine))
            .await
            .unwrap()
            .state(),
        image(&original).as_ref()
    );
    let mut positive_export = export_campaign(&positive_pool, f.campaign).await.unwrap();
    positive_export
        .exported_at_utc
        .clone_from(&genuine.exported_at_utc);
    assert_eq!(positive_export, genuine);
    positive_pool.close().await;
    drop(positive_app);
    drop(positive_pool);

    // Both current profiles and a real earlier snapshot are independently
    // authenticated. The known-valid wrong pin changes no intrinsic statistic.
    for (retired, actor, wrong_pin) in [
        (false, old.actor, new.source.clone()),
        (false, new.actor, old.source.clone()),
        (true, old.actor, new.source.clone()),
    ] {
        let mut forged = genuine.clone();
        let json = if retired {
            &mut forged
                .snapshots
                .iter_mut()
                .find(|row| row.event_sequence == earlier.applied_event_sequence)
                .unwrap()
                .state_json
        } else {
            &mut forged.current_state.state_json
        };
        let mut changed = Box::new(CampaignState::decode_json(json).unwrap());
        changed
            .rules
            .as_mut()
            .unwrap()
            .tactical_creatures
            .as_mut()
            .unwrap()
            .profiles
            .iter_mut()
            .find(|p| p.actor == actor)
            .unwrap()
            .source = wrong_pin;
        source_for_profile(profile(&changed, actor)).unwrap();
        dmd_rules::tactical_creatures::validate_creature_profile(
            &changed,
            profile(&changed, actor),
            &changed.rules.as_ref().unwrap().entities[&actor],
        )
        .unwrap();
        *json = changed.encode_json().unwrap();
        assert_eq!(forged.event_journal, genuine.event_journal);
        assert_eq!(forged.command_audit, genuine.command_audit);
        assert_eq!(forged.event_causes, genuine.event_causes);
        assert_eq!(forged.play_sessions, genuine.play_sessions);
        assert_eq!(
            forged.play_session_participants,
            genuine.play_session_participants
        );
        assert_eq!(forged.observations, genuine.observations);
        assert_eq!(
            forged.table_projection_history,
            genuine.table_projection_history
        );
        assert_eq!(
            forged.table_transport_bindings,
            genuine.table_transport_bindings
        );
        if retired {
            assert_eq!(forged.current_state, genuine.current_state);
        } else {
            assert_eq!(forged.snapshots, genuine.snapshots);
        }
        let path = f
            .directory
            .join(format!("hostile-{retired}-{}.sqlite", actor.0));
        let pool = open_sqlite_path(&path).await.unwrap();
        let app = runtime(pool.clone());
        let before = destination_rows(&pool).await;
        assert!(Box::pin(app.restore_campaign(&forged)).await.is_err());
        assert_eq!(
            destination_rows(&pool).await,
            before,
            "fresh hostile restore is atomic across every table"
        );
        assert_eq!(
            Box::pin(app.restore_campaign(&genuine))
                .await
                .unwrap()
                .state(),
            image(&original).as_ref(),
            "genuine history still restores into the same rejected destination"
        );
        let mut restored_export = export_campaign(&pool, f.campaign).await.unwrap();
        restored_export
            .exported_at_utc
            .clone_from(&genuine.exported_at_utc);
        assert_eq!(restored_export, genuine);
        pool.close().await;
        drop(app);
        drop(pool);
    }
    f.assert_export(&original).await;
}

async fn cold_final_restore_and_receipts(
    f: &Fixture,
    old: &CreatureProfile,
    created: &TableTransportRequest,
) {
    let accepted = export_campaign(&f.pool, f.campaign).await.unwrap();
    assert_prefix(&f.original, &accepted);
    for snapshot in &f.original.snapshots {
        assert!(
            accepted.snapshots.contains(snapshot),
            "all original snapshot rows remain byte-for-byte retained"
        );
    }
    for cause in &f.original.event_causes {
        assert!(accepted.event_causes.contains(cause));
    }
    let path = f.directory.join("coexistence-final.sqlite");
    let pool = open_sqlite_path(&path).await.unwrap();
    let app = runtime(pool.clone());
    Box::pin(app.restore_campaign(&accepted)).await.unwrap();
    pool.close().await;
    drop(app);
    drop(pool);
    let pool = open_sqlite_path(&path).await.unwrap();
    let app = runtime(pool.clone());
    assert_eq!(
        Box::pin(app.resume_campaign(f.campaign))
            .await
            .unwrap()
            .state(),
        image(&accepted).as_ref()
    );
    for binding in &accepted.table_transport_bindings {
        let request: TableTransportRequest = serde_json::from_str(&binding.request_json).unwrap();
        assert_eq!(
            serde_json::to_string(&request).unwrap(),
            binding.request_json
        );
        let expected: TableTransportResult = serde_json::from_str(&binding.response_json).unwrap();
        let response = Box::pin(app.submit_presented_table(request)).await.unwrap();
        assert_eq!(response, expected);
        assert_eq!(
            serde_json::to_string(&response).unwrap(),
            binding.response_json
        );
    }
    let original = f
        .original
        .event_journal
        .iter()
        .find(|event| event.command_id == old.origin.id.0.to_string())
        .unwrap();
    let saved: TableEvent = serde_json::from_str(&original.payload_json).unwrap();
    let receipt = Box::pin(app.execute_table(saved.meta, saved.action))
        .await
        .unwrap();
    assert!(receipt.already_accepted);
    assert_eq!(receipt.command_id, old.origin.id);
    assert_eq!(
        receipt.event_sequence,
        u64::try_from(original.sequence).unwrap()
    );
    assert_eq!(receipt.outcome, saved.outcome);
    let mut changed = created.clone();
    changed.input = TableTransportInput::Action(Box::new(tactical(TacticalAction::Dodge)));
    assert!(Box::pin(app.submit_presented_table(changed)).await.is_err());
    let mut actual = export_campaign(&pool, f.campaign).await.unwrap();
    actual.exported_at_utc.clone_from(&accepted.exported_at_utc);
    assert_eq!(
        actual, accepted,
        "all old/new exact receipts are read-only after independent cold restore"
    );
    pool.close().await;
    drop(app);
    drop(pool);
}

#[tokio::test]
async fn genuine_old_goblin_and_current_goblin_coexist_after_release_and_source_attacks() {
    let mut f = Box::pin(Fixture::restore(ORIGINAL, 16, 11, 2)).await;
    let original = Box::new(f.state().await);
    assert_eq!(original.applied_event_sequence, 16);
    assert_eq!(flow(&original).version, 2);
    assert_eq!(flow(&original).phase, TacticalPhase::Active);
    assert!(flow(&original).resolution.is_none());
    assert!(flow(&original).ready.is_empty());
    assert!(original.rules.as_ref().unwrap().pending.is_none());
    assert_eq!(flow(&original).budget.attacks_remaining, 0);
    assert!(flow(&original).budget.attack_window.is_some());
    let timing = original.rules.as_ref().unwrap().timing.as_ref().unwrap();
    assert_eq!(timing.turn_number, 1);
    assert_eq!(timing.round, 1);
    assert!(timing.action_spent && timing.reactions_spent.is_empty());
    let pc = timing.order[timing.index].actor;
    let (_, pc_channel) = owner(&original, pc);
    let old = original
        .rules
        .as_ref()
        .unwrap()
        .tactical_creatures
        .as_ref()
        .unwrap()
        .profiles[0]
        .clone();
    assert_eq!(
        old.source,
        creature_source_pin(creature_definition("goblin-warrior").unwrap()).unwrap()
    );
    assert_eq!(old.source.definition_fingerprint, "b1b3d06ef838810e");
    assert_eq!(original.rules.as_ref().unwrap().entities[&old.actor].hp, 4);
    assert_eq!(original.rules.as_ref().unwrap().entities[&pc].hp, 12);
    let old_runtime = original
        .rules
        .as_ref()
        .unwrap()
        .tactical_creatures
        .as_ref()
        .unwrap()
        .runtime(old.actor)
        .unwrap();
    assert_eq!(old_runtime.controller, CreatureController::Autonomous);
    assert_zero_grapple(&original);
    Box::pin(retry_original_creation(&f, &old)).await;

    // The actual original pending Savage work was already completed by the
    // original producer. Restore retries both genuine bindings; no new roll is due.
    let savage = original
        .rules
        .as_ref()
        .unwrap()
        .rolls
        .last()
        .unwrap()
        .savage_attacker
        .as_ref()
        .unwrap();
    assert_eq!(
        savage
            .first
            .dice
            .iter()
            .map(|d| d.value)
            .collect::<Vec<_>>(),
        [1, 2]
    );
    assert_eq!(
        savage
            .second
            .dice
            .iter()
            .map(|d| d.value)
            .collect::<Vec<_>>(),
        [4, 4]
    );
    assert_eq!(savage.chosen, DamageRollChoice::First);
    Box::pin(reject(
        &f,
        pc_channel.clone(),
        tactical(TacticalAction::EndTurn),
    ))
    .await;
    Box::pin(reject(
        &f,
        TableTransportChannel::Host,
        tactical(TacticalAction::UpgradeExecution),
    ))
    .await;
    let premature = current_creation(&f).await;
    Box::pin(reject(&f, TableTransportChannel::Host, creation(premature))).await;

    Box::pin(host(
        &mut f,
        tactical(TacticalAction::UpgradeExecutionTo {
            execution: TacticalExecutionVersion::EncounterReleaseV1,
        }),
    ))
    .await;
    let upgraded = Box::new(f.state().await);
    assert_eq!(flow(&upgraded).version, 5);
    assert!(flow(&upgraded).budget.attack_window.is_some());
    let mut comparison_only = upgraded.clone();
    comparison_only
        .encounter
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .version = 2;
    comparison_only.applied_event_sequence = original.applied_event_sequence;
    assert_eq!(
        comparison_only, original,
        "targeted upgrade changes no paid state or body"
    );
    Box::pin(accept(
        &mut f,
        pc_channel.clone(),
        tactical(TacticalAction::EndTurn),
    ))
    .await;
    let settled = Box::new(f.state().await);
    assert!(flow(&settled).resolution.is_none());
    assert!(flow(&settled).budget.attack_window.is_none());
    assert!(flow(&settled).budget.weapon_history.is_empty());
    let timing = settled.rules.as_ref().unwrap().timing.as_ref().unwrap();
    assert_eq!(timing.turn_number, 2);
    assert_eq!(timing.round, 1);
    assert_eq!(timing.order[timing.index].actor, old.actor);
    assert_original_paid_state(&settled, &original);
    Box::pin(host(&mut f, tactical(TacticalAction::ConcludeHostilities {
        cadence: AftermathCadence::ContinueExistingOrder,
        ruling: "Host concludes the original conflict after its actual paid work and owner turn have settled.".into(),
    }))).await;
    dmd_rules::tactical::encounter_release_preflight(&f.state().await).unwrap();
    Box::pin(host(&mut f, tactical(TacticalAction::FinishEncounter))).await;
    let finished = Box::new(f.state().await);
    assert_eq!(flow(&finished).phase, TacticalPhase::Finished);
    assert!(finished.rules.as_ref().unwrap().timing.is_none());
    assert_eq!(finished.items, original.items);
    assert_eq!(
        finished.rules.as_ref().unwrap().tactical_inventory,
        original.rules.as_ref().unwrap().tactical_inventory
    );
    assert_eq!(profile(&finished, old.actor), &old);
    assert_eq!(
        finished.table.as_ref().unwrap().active_session,
        original.table.as_ref().unwrap().active_session
    );
    let receipt = finished.encounter_history.as_ref().unwrap().last().unwrap();
    assert_eq!(receipt.final_turn.actor, old.actor);
    assert_eq!(receipt.final_turn.number, 2);
    assert_eq!(
        receipt.execution,
        TacticalExecutionVersion::EncounterReleaseV1
    );
    assert_eq!(
        finished.scenes[&receipt.scene_id].status,
        SceneStatus::Closed
    );
    assert_original_paid_state(&finished, &original);
    let earlier = export_campaign(&f.pool, f.campaign)
        .await
        .unwrap()
        .current_state;

    let selected = current_creation(&f).await;
    let mut bad_sources = vec![None, Some(old.source.clone())];
    for field in 0..4 {
        let mut pin = selected.source.clone().unwrap();
        match field {
            0 => pin.ruleset_id.push_str("-changed"),
            1 => pin.ruleset_version.push_str("-changed"),
            2 => pin.definition_id.push_str("-changed"),
            _ => pin.definition_fingerprint.push_str("-changed"),
        }
        bad_sources.push(Some(pin));
    }
    for source in bad_sources {
        let mut wrong = selected.clone();
        wrong.source = source;
        Box::pin(reject(&f, TableTransportChannel::Host, creation(wrong))).await;
    }
    let mut zero_ammo = selected.clone();
    zero_ammo.ammunition_units = 0;
    Box::pin(reject(&f, TableTransportChannel::Host, creation(zero_ammo))).await;
    Box::pin(reject(&f, pc_channel, creation(selected.clone()))).await;
    let created = Box::pin(host(&mut f, creation(selected.clone()))).await;
    let coexistence = Box::new(f.state().await);
    let new = profile(&coexistence, selected.entity_id).clone();
    assert_eq!(new.origin.id, created.command_id);
    assert_eq!(new.source, selected.source.clone().unwrap());
    assert_eq!(new.hit_points, CreatureHitPointOrigin::Average);
    assert_eq!(
        coexistence.rules.as_ref().unwrap().entities[&new.actor].hp,
        10
    );
    assert_pair(&coexistence, &old, &new);
    assert_original_paid_state(&coexistence, &original);
    assert_eq!(coexistence.items.len(), original.items.len() + 5);
    assert_eq!(
        loadout(&coexistence, old.actor),
        loadout(&original, old.actor)
    );
    for entry in original
        .rules
        .as_ref()
        .unwrap()
        .tactical_inventory
        .as_ref()
        .unwrap()
        .loadouts
        .iter()
    {
        assert_eq!(loadout(&coexistence, entry.actor), entry);
    }
    for (definition, quantity) in [
        ("leather-armor", 1),
        ("shield", 1),
        ("scimitar", 1),
        ("shortbow", 1),
        ("arrows", 20),
    ] {
        let id = item(&coexistence, new.actor, definition);
        let actual = &coexistence.items[&id];
        assert!(selected.item_ids.contains(&id));
        assert_eq!(actual.quantity, quantity);
        assert_eq!(actual.owner, Ownership::Entity(new.actor));
        assert_eq!(actual.custody, Custody::Entity(new.actor));
    }
    // Three deliberately forged negative copies; no forged positive state.
    Box::pin(reject_revision_swaps(&f, &old, &new, &earlier)).await;
    Box::pin(enable_and_assign(&mut f, [old.actor, new.actor])).await;
    Box::pin(retry_original_creation(&f, &old)).await;
    Box::pin(retry_created(&f, &created)).await;
    Box::pin(second_scene(&mut f, &old, &new, pc)).await;
    let first_source = Box::pin(scimitar_miss(&mut f, &old, &new)).await;
    Box::pin(host(&mut f, tactical(TacticalAction::EndTurn))).await;
    let second_source = Box::pin(scimitar_miss(&mut f, &new, &old)).await;
    assert_eq!(first_source, old.source);
    assert_eq!(second_source, new.source);
    assert_ne!(first_source, second_source);
    let final_state = Box::new(f.state().await);
    assert_pair(&final_state, &old, &new);
    assert_original_paid_state(&final_state, &original);
    assert_eq!(final_state.items, coexistence.items);
    assert_eq!(
        final_state.rules.as_ref().unwrap().entities[&old.actor].hp,
        4
    );
    assert_eq!(
        final_state.rules.as_ref().unwrap().entities[&new.actor].hp,
        10
    );
    for actor in [old.actor, new.actor] {
        assert_eq!(
            final_state.items[&item(&final_state, actor, "arrows")].quantity,
            20
        );
        let runtime = final_state
            .rules
            .as_ref()
            .unwrap()
            .tactical_creatures
            .as_ref()
            .unwrap()
            .runtime(actor)
            .unwrap();
        assert_eq!(runtime.controller, CreatureController::Host);
        assert!(runtime.limited_uses.is_empty());
    }
    Box::pin(cold_final_restore_and_receipts(&f, &old, &created)).await;
    f.close().await;
}
