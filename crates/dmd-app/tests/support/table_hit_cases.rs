//! Genuine source ownership, physical attack, private response and cold transport.
use super::*;
use dmd_rules::tactical::TacticalAction;

fn runtime(pool: sqlx::SqlitePool) -> CampaignRuntime {
    CampaignRuntime::from_content_root(
        pool,
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../content"),
    )
}
fn action(action: TacticalAction) -> TableTransportInput {
    TableTransportInput::Action(Box::new(TableAction::Tactical { action }))
}
fn attacker(f: &Fixture) -> TableTransportChannel {
    TableTransportChannel::Player {
        player_id: f.players[0],
        character_id: f.characters[0],
    }
}
fn owner(f: &Fixture, mage: EntityId) -> TableTransportChannel {
    TableTransportChannel::SourceCreature {
        player_id: f.players[1],
        actor: mage,
    }
}
async fn state(f: &Fixture) -> CampaignState {
    f.runtime
        .open_campaign(f.campaign)
        .await
        .unwrap()
        .state()
        .clone()
}
async fn view(f: &Fixture, channel: &TableTransportChannel) -> TablePresentedView {
    let viewer = match channel {
        TableTransportChannel::Host => TableViewer::Host,
        TableTransportChannel::Player { player_id, .. }
        | TableTransportChannel::SourceCreature { player_id, .. } => {
            TableViewer::Player(*player_id)
        }
    };
    f.runtime
        .presented_table_view(f.campaign, viewer)
        .await
        .unwrap()
}
async fn request(
    f: &Fixture,
    channel: TableTransportChannel,
    input: TableTransportInput,
) -> TableTransportRequest {
    TableTransportRequest {
        version: 2,
        command_id: CommandId::new(),
        campaign_id: f.campaign,
        session_id: Some(f.session),
        revision: view(f, &channel).await.revision,
        channel,
        input,
    }
}
async fn reopen(f: &mut Fixture, path: &Path) {
    f.pool.close().await;
    f.pool = dmd_persistence::open_sqlite_path(path).await.unwrap();
    f.runtime = runtime(f.pool.clone());
    Box::pin(f.runtime.resume_campaign(f.campaign))
        .await
        .unwrap();
}
async fn unchanged(f: &Fixture, request: TableTransportRequest) {
    let before = export_campaign(&f.pool, f.campaign).await.unwrap();
    assert!(
        Box::pin(f.runtime.submit_presented_table(request))
            .await
            .is_err()
    );
    let mut after = export_campaign(&f.pool, f.campaign).await.unwrap();
    after.exported_at_utc = before.exported_at_utc.clone();
    assert_eq!(after, before);
}

// Exactly one accepted command. Original opaque capabilities come from the same
// exported prefix. Fresh independent receipt revisions need not have equal UUIDs.
async fn cold_step(
    f: &mut Fixture,
    path: &Path,
    request: TableTransportRequest,
) -> TableTransportResult {
    let exported = export_campaign(&f.pool, f.campaign).await.unwrap();
    let pool = open_sqlite("sqlite::memory:").await.unwrap();
    let mirror = runtime(pool.clone());
    Box::pin(mirror.restore_campaign(&exported)).await.unwrap();
    Box::pin(reopen(f, path)).await;
    let response = Box::pin(f.runtime.submit_presented_table(request.clone()))
        .await
        .unwrap();
    let independent = Box::pin(mirror.submit_presented_table(request.clone()))
        .await
        .unwrap();
    let (TableTransportResult::Accepted(first), TableTransportResult::Accepted(second)) =
        (&response, &independent)
    else {
        panic!("accepted action required");
    };
    assert_eq!(first.outcome, second.outcome);
    assert_eq!(
        mirror.open_campaign(f.campaign).await.unwrap().state(),
        &state(f).await
    );
    assert_eq!(
        Box::pin(mirror.submit_presented_table(request.clone()))
            .await
            .unwrap(),
        independent
    );
    let mirrored = export_campaign(&pool, f.campaign).await.unwrap();
    let recovered_pool = open_sqlite("sqlite::memory:").await.unwrap();
    Box::pin(runtime(recovered_pool.clone()).restore_campaign(&mirrored))
        .await
        .unwrap();
    recovered_pool.close().await;
    pool.close().await;
    Box::pin(reopen(f, path)).await;
    let saved = export_campaign(&f.pool, f.campaign).await.unwrap();
    assert_eq!(
        Box::pin(f.runtime.submit_presented_table(request.clone()))
            .await
            .unwrap(),
        response
    );
    let mut changed = request;
    changed.input = action(TacticalAction::Dodge);
    assert!(
        Box::pin(f.runtime.submit_presented_table(changed))
            .await
            .is_err()
    );
    let mut after = export_campaign(&f.pool, f.campaign).await.unwrap();
    after.exported_at_utc = saved.exported_at_utc.clone();
    assert_eq!(after, saved);
    response
}
async fn step(
    f: &mut Fixture,
    path: &Path,
    channel: TableTransportChannel,
    input: TableTransportInput,
) -> CommandId {
    let request = request(f, channel, input).await;
    let id = request.command_id;
    Box::pin(cold_step(f, path, request)).await;
    id
}
fn raw(view: TablePresentedView, values: &[u16]) -> TableTransportInput {
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
    Box::pin(step(
        f,
        path,
        TableTransportChannel::Host,
        TableTransportInput::Action(Box::new(TableAction::SetSourceCreatureController {
            actor: mage,
            controller: CreatureController::Player(f.players[1]),
        })),
    ))
    .await;
    let actors = [f.actors[0], mage];
    Box::pin(step(
        f,
        path,
        TableTransportChannel::Host,
        action(TacticalAction::Begin {
            execution: TacticalExecutionVersion::ShieldHitV1,
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
    let pc = attacker(f);
    let mage_channel = owner(f, mage);
    let input = raw(view(f, &pc).await, &[18]);
    Box::pin(step(f, path, pc, input)).await;
    let input = raw(view(f, &mage_channel).await, &[2]);
    Box::pin(step(f, path, mage_channel, input)).await;
    assert_eq!(
        state(f).await.rules.as_ref().unwrap().entities[&mage].hp,
        81
    );
    mage
}

async fn hit(f: &mut Fixture, path: &Path, mage: EntityId, face: u16) -> CommandId {
    let channel = attacker(f);
    let options = view(f, &channel)
        .await
        .tactical
        .unwrap()
        .attack_options
        .unwrap();
    let weapon = options
        .weapons
        .iter()
        .find(|weapon| weapon.name == "Dagger")
        .unwrap()
        .item;
    let equipped = options
        .hands
        .hands
        .iter()
        .any(|hand| *hand == HandAssignment::Item(weapon));
    Box::pin(step(
        f,
        path,
        channel.clone(),
        action(TacticalAction::Attack {
            choice: WeaponUseChoice {
                weapon,
                target: mage,
                delivery: WeaponDelivery::Melee,
                ability: Ability::Strength,
                grip: WeaponGrip::OneHand(Hand::Right),
                purpose: WeaponAttackPurpose::Normal,
                ammunition: None,
                equipment_change: (!equipped).then_some(AttackEquipmentChange {
                    timing: EquipmentChangeTiming::BeforeAttack,
                    operation: AttackEquipmentOperation::Equip {
                        item: weapon,
                        hand: Hand::Right,
                    },
                }),
            },
        }),
    ))
    .await;
    let dice = raw(view(f, &channel).await, &[face]);
    Box::pin(step(f, path, channel, dice)).await
}

async fn order(f: &mut Fixture, path: &Path) {
    let channel = attacker(f);
    let key = view(f, &channel)
        .await
        .tactical
        .unwrap()
        .hit
        .unwrap()
        .order
        .unwrap()
        .key;
    Box::pin(step(
        f,
        path,
        channel,
        TableTransportInput::HitResponse {
            handle: key,
            decision: Box::new(TableHitInput::Order {
                instruction: TacticalReactionOrdering {
                    ranked: vec![],
                    unlisted: ReactionUnlistedOrder::AfterForward,
                },
            }),
        },
    ))
    .await;
}
async fn offer(f: &mut Fixture, path: &Path, mage: EntityId) {
    let channel = owner(f, mage);
    let owned = view(f, &channel)
        .await
        .tactical
        .unwrap()
        .hit
        .unwrap()
        .response
        .unwrap();
    assert!(!owned.selected);
    assert_eq!(owned.shield.len(), 1);
    let request = request(
        f,
        channel,
        TableTransportInput::HitResponse {
            handle: owned.key,
            decision: Box::new(TableHitInput::Respond { accept: true }),
        },
    )
    .await;
    let mut foreign = request.clone();
    foreign.command_id = CommandId::new();
    foreign.channel = attacker(f);
    foreign.revision = view(f, &foreign.channel).await.revision;
    unchanged(f, foreign).await;
    let mut host = request.clone();
    host.command_id = CommandId::new();
    host.channel = TableTransportChannel::Host;
    host.revision = view(f, &host.channel).await.revision;
    assert!(
        view(f, &host.channel)
            .await
            .tactical
            .unwrap()
            .hit
            .unwrap()
            .response
            .is_none()
    );
    unchanged(f, host).await;
    let mut wrong_role = request.clone();
    wrong_role.command_id = CommandId::new();
    wrong_role.input = TableTransportInput::HitResponse {
        handle: owned.key,
        decision: Box::new(TableHitInput::Delegate),
    };
    unchanged(f, wrong_role).await;
    let before = view(f, &attacker(f)).await;
    let rules_before = state(f).await.rules.unwrap();
    Box::pin(cold_step(f, path, request)).await;
    assert_eq!(view(f, &attacker(f)).await, before); // Entire DTO, revision and transcript.
    let rules_after = state(f).await.rules.unwrap();
    assert_eq!(rules_after.timing, rules_before.timing);
    assert_eq!(
        rules_after.tactical_creatures,
        rules_before.tactical_creatures
    );
    assert_eq!(rules_after.rolls, rules_before.rolls);
}
async fn cast(f: &mut Fixture, path: &Path, mage: EntityId) -> CommandId {
    let channel = owner(f, mage);
    let owned = view(f, &channel)
        .await
        .tactical
        .unwrap()
        .hit
        .unwrap()
        .response
        .unwrap();
    assert!(owned.selected);
    let choice = owned.shield[0].clone();
    assert_eq!(choice.spell_id, "shield");
    assert_eq!(
        choice.grant,
        SpellGrantChoice::CreatureFeature {
            feature_id: "protective-magic".into()
        }
    );
    let before = state(f).await;
    let window = before
        .encounter
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .resolution
        .as_ref()
        .unwrap();
    let canonical = TacticalWorkKey {
        resolution: window.origin.id,
        occurrence: window.hit_review.as_ref().unwrap().work.occurrence,
    };
    let bypass = request(
        f,
        channel.clone(),
        action(TacticalAction::CastHitShield {
            window: canonical,
            choice: choice.clone(),
        }),
    )
    .await;
    unchanged(f, bypass).await;
    Box::pin(step(
        f,
        path,
        channel,
        TableTransportInput::HitResponse {
            handle: owned.key,
            decision: Box::new(TableHitInput::Cast { choice }),
        },
    ))
    .await
}

async fn reject_changed_hit_images(f: &Fixture) {
    let original = export_campaign(&f.pool, f.campaign).await.unwrap();
    for mutation in 0..4 {
        let mut changed = state(f).await;
        let resolution = changed
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
            0 => resolution.hit_review = None,
            1 => resolution.hit_review.as_mut().unwrap().cover_bonus += 2,
            2 => resolution.hit_review.as_mut().unwrap().cause = resolution.origin.clone(),
            3 => {
                resolution
                    .hit_review
                    .as_mut()
                    .unwrap()
                    .completed_shield
                    .as_mut()
                    .unwrap()
                    .cast
                    .plan
                    .choice
                    .actor = f.actors[0]
            }
            _ => unreachable!(),
        }
        let mut forged = original.clone();
        forged.current_state.state_json = changed.encode_json().unwrap();
        let pool = open_sqlite("sqlite::memory:").await.unwrap();
        assert!(
            Box::pin(runtime(pool.clone()).restore_campaign(&forged))
                .await
                .is_err(),
            "changed hit image {mutation}"
        );
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM campaign_state_current")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 0);
        pool.close().await;
    }
}

#[tokio::test]
async fn owned_source_shield_reopens_each_decision_preserves_attack_cause_and_rejects_changed_hits()
{
    let path = std::env::temp_dir().join(format!("dmd-hit-{}.sqlite", CampaignId::new().0));
    let pool = dmd_persistence::open_sqlite_path(&path).await.unwrap();
    let mut creation = input("Attacker");
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
    let mage = Box::pin(prepare(&mut f, &path)).await;
    // First response arrives before ordering; no cost or unrelated visible change.
    Box::pin(hit(&mut f, &path, mage, 10)).await;
    Box::pin(offer(&mut f, &path, mage)).await;
    Box::pin(order(&mut f, &path)).await;
    Box::pin(cast(&mut f, &path, mage)).await;
    let after = state(&f).await;
    assert_eq!(after.rules.as_ref().unwrap().entities[&mage].hp, 81);
    assert!(
        after
            .encounter
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .resolution
            .is_none()
    );
    assert_eq!(
        dmd_rules::tactical_defenses::effective_armor_class(&after, mage).unwrap(),
        17
    );
    assert!(
        after
            .rules
            .as_ref()
            .unwrap()
            .timing
            .as_ref()
            .unwrap()
            .reactions_spent
            .contains(&mage)
    );
    let pc = attacker(&f);
    Box::pin(step(&mut f, &path, pc, action(TacticalAction::EndTurn))).await;
    assert_eq!(
        dmd_rules::tactical_defenses::effective_armor_class(&state(&f).await, mage).unwrap(),
        12
    );
    let mage_channel = owner(&f, mage);
    Box::pin(step(
        &mut f,
        &path,
        mage_channel,
        action(TacticalAction::EndTurn),
    ))
    .await;
    // Opposite arrival order; natural20 still uses the original physical damage.
    let hit_cause = Box::pin(hit(&mut f, &path, mage, 20)).await;
    Box::pin(order(&mut f, &path)).await;
    Box::pin(offer(&mut f, &path, mage)).await;
    let response_cause = Box::pin(cast(&mut f, &path, mage)).await;
    let pending = state(&f).await;
    assert_eq!(
        pending
            .rules
            .as_ref()
            .unwrap()
            .pending
            .as_ref()
            .unwrap()
            .issued_by
            .id,
        hit_cause
    );
    assert_ne!(hit_cause, response_cause);
    assert_eq!(pending.rules.as_ref().unwrap().entities[&mage].hp, 81);
    let source = pending
        .rules
        .as_ref()
        .unwrap()
        .tactical_creatures
        .as_ref()
        .unwrap()
        .runtime(mage)
        .unwrap();
    assert_eq!(
        source
            .limited_uses
            .iter()
            .find(|usage| usage.feature_id == "protective-magic" && usage.spell_id.is_none())
            .unwrap()
            .spent,
        2
    );
    Box::pin(reject_changed_hit_images(&f)).await;
    let pc = attacker(&f);
    let dice = raw(view(&f, &pc).await, &[1, 2]);
    Box::pin(step(&mut f, &path, pc, dice)).await;
    let final_state = state(&f).await;
    assert_eq!(final_state.rules.as_ref().unwrap().entities[&mage].hp, 75);
    assert_eq!(
        final_state
            .rules
            .as_ref()
            .unwrap()
            .rolls
            .last()
            .unwrap()
            .result
            .dice,
        vec![
            DieResult { sides: 4, value: 1 },
            DieResult { sides: 4, value: 2 }
        ]
    );
    f.pool.close().await;
    drop(f);
    sqlite_test_cleanup::remove_closed_file(&path)
        .await
        .unwrap();
}
