//! Additive current-read coverage over original production-created histories.
use super::*;

async fn request(f: &Fixture, channel: TableTransportChannel) -> TableIntrinsicAttackRequest {
    let view = f.view(channel.clone()).await;
    TableIntrinsicAttackRequest {
        version: 1,
        campaign_id: f.campaign,
        channel,
        revision: view.revision,
        actor: f.goblin,
    }
}

async fn unchanged(f: &Fixture, before: &CampaignExport, rows: &[(String, Vec<Vec<String>>)]) {
    let mut after = export_campaign(&f.pool, f.campaign).await.unwrap();
    after.exported_at_utc = before.exported_at_utc.clone();
    assert_eq!(&after, before);
    assert_eq!(all_rows(&f.pool).await, rows);
}

async fn refused(f: &Fixture, request: TableIntrinsicAttackRequest) {
    let before = export_campaign(&f.pool, f.campaign).await.unwrap();
    let rows = all_rows(&f.pool).await;
    assert!(matches!(
        Box::pin(f.runtime.table_intrinsic_attack_options(request)).await,
        Err(RunnableCampaignError::TableRejected(_))
    ));
    Box::pin(unchanged(f, &before, &rows)).await;
}

async fn read(f: &Fixture, request: TableIntrinsicAttackRequest) -> TableIntrinsicAttackOptions {
    let before = export_campaign(&f.pool, f.campaign).await.unwrap();
    let rows = all_rows(&f.pool).await;
    let result = Box::pin(f.runtime.table_intrinsic_attack_options(request.clone()))
        .await
        .unwrap();
    assert_eq!(result.version, 1);
    assert_eq!(result.revision, request.revision);
    assert_eq!(result.actor, request.actor);
    assert!(
        result
            .features
            .windows(2)
            .all(|pair| pair[0].feature_id < pair[1].feature_id)
    );
    assert!(result.targets.iter().all(|target| target.actor != f.goblin));
    assert!(
        result
            .targets
            .iter()
            .all(|target| target.actor != f.actors[1])
    );
    assert_eq!(
        Box::pin(f.runtime.table_intrinsic_attack_options(request))
            .await
            .unwrap(),
        result
    );
    Box::pin(unchanged(f, &before, &rows)).await;
    result
}

async fn submit_at(
    f: &mut Fixture,
    channel: TableTransportChannel,
    input: TableTransportInput,
    version: u32,
) {
    let mut request = f.request(channel, input).await;
    request.version = version;
    if let TableTransportInput::Action(action) = &request.input
        && let TableAction::StartSession { id, .. } = action.as_ref()
    {
        request.session_id = Some(*id);
    }
    Box::pin(f.cold(request)).await;
}

async fn begin_mass_encounter(f: &mut Fixture, owner: TableTransportChannel) {
    let original = f.state().await;
    let encounter = original.encounter.as_ref().unwrap();
    Box::pin(submit_at(
        f,
        TableTransportChannel::Host,
        action(TacticalAction::ConcludeHostilities {
            cadence: AftermathCadence::ContinueExistingOrder,
            ruling: "The participants settle this encounter before enabling physical facts.".into(),
        }),
        4,
    ))
    .await;
    Box::pin(submit_at(
        f,
        TableTransportChannel::Host,
        TableTransportInput::Action(Box::new(TableAction::EndSession)),
        4,
    ))
    .await;
    let closed_session = request(f, owner.clone()).await;
    Box::pin(refused(f, closed_session)).await;
    let participants = (0..2)
        .map(|index| SessionParticipant {
            player_id: f.players[index],
            character_id: Some(f.characters[index]),
            attendance: AttendanceStatus::Present,
        })
        .collect();
    Box::pin(submit_at(
        f,
        TableTransportChannel::Host,
        TableTransportInput::Action(Box::new(TableAction::StartSession {
            id: PlaySessionId::new(),
            name: "Resumed source ownership".into(),
            participants,
        })),
        4,
    ))
    .await;
    Box::pin(submit_at(
        f,
        TableTransportChannel::Host,
        action(TacticalAction::FinishEncounter),
        4,
    ))
    .await;
    Box::pin(submit_at(
        f,
        TableTransportChannel::Host,
        TableTransportInput::EnablePhysicalFacts,
        5,
    ))
    .await;
    let pc = encounter.participant(f.actors[0]).unwrap();
    let creature = encounter.participant(f.goblin).unwrap();
    let setup = TableBattlefieldSetup {
        encounter_id: EncounterId::new(),
        scene_id: SceneId::new(),
        location_id: LocationId::new(),
        name: "A new encounter with the retained participants".into(),
        battlefield: encounter.battlefield.clone(),
        characters: vec![TableCharacterPlacement {
            character_id: f.characters[0],
            position: pc.position,
            height: 12,
            allies: pc.allies.clone(),
            enemies: pc.enemies.clone(),
        }],
        creatures: vec![TableCreaturePlacement {
            actor: f.goblin,
            public_label: "Small armored figure".into(),
            position: creature.position,
            height: 20,
            allies: creature.allies.clone(),
            enemies: creature.enemies.clone(),
        }],
        geometry_ruling: Ruling {
            basis: RulingBasis::GmAdjudication,
            reason: "The Host establishes the retained visible geometry in a new scene.".into(),
        },
        area_grid_policy: None,
    };
    assert_ne!(setup.encounter_id, encounter.id);
    Box::pin(submit_at(
        f,
        TableTransportChannel::Host,
        TableTransportInput::Action(Box::new(TableAction::PrepareBattlefield {
            setup: Box::new(setup),
        })),
        5,
    ))
    .await;
    let actor = f.goblin;
    let pc = f.actors[0];
    Box::pin(submit_at(
        f,
        TableTransportChannel::Host,
        action(TacticalAction::Begin {
            execution: TacticalExecutionVersion::EncounterReleaseV1,
            combatants: vec![
                TacticalCombatant {
                    actor: pc,
                    source: TacticalSource::Character,
                    surprised: false,
                },
                TacticalCombatant {
                    actor,
                    source: TacticalSource::Creature {
                        definition_id: "chimera".into(),
                    },
                    surprised: false,
                },
            ],
            groups: vec![
                InitiativeGroup {
                    actors: vec![pc],
                    request_id: RollRequestId::new(),
                },
                InitiativeGroup {
                    actors: vec![actor],
                    request_id: RollRequestId::new(),
                },
            ],
        }),
        5,
    ))
    .await;
    for (channel, value) in [(f.pc(0), 18), (owner, 2)] {
        let roll = f.view(channel.clone()).await.roll.unwrap();
        assert_eq!(roll.mode, RollMode::Normal);
        Box::pin(submit_at(
            f,
            channel,
            action(TacticalAction::SubmitRoll {
                result: RollResult {
                    request_id: roll.id,
                    source: RollSource::Physical,
                    dice: vec![DieResult { sides: 20, value }],
                },
            }),
            5,
        ))
        .await;
    }
    let pc = f.pc(0);
    Box::pin(submit_at(f, pc, action(TacticalAction::EndTurn), 5)).await;
}

#[tokio::test]
async fn advisory_intrinsic_choice_uses_original_attack_and_survives_release_restore_and_retry() {
    let mut f = Box::pin(Fixture::with_source("chimera", CreatureSize::Large)).await;
    Box::pin(f.activate()).await;
    let wrong_turn = request(&f, TableTransportChannel::Host).await;
    Box::pin(refused(&f, wrong_turn)).await;
    let grip = Box::pin(f.establish_pc_grip()).await;
    let pc = f.pc(0);
    Box::pin(f.cold_action(pc.clone(), TacticalAction::EndTurn)).await;
    let host = request(&f, TableTransportChannel::Host).await;
    let original_view = f.view(TableTransportChannel::Host).await;
    let choices = Box::pin(read(&f, host.clone())).await;
    assert_eq!(f.view(TableTransportChannel::Host).await, original_view);
    let bite = choices
        .features
        .iter()
        .find(|choice| choice.feature_id == "bite")
        .unwrap();
    assert_eq!(bite.label, "Bite");
    assert!(
        choices
            .targets
            .iter()
            .any(|target| target.actor == f.actors[0])
    );
    let target = f.actors[0];
    for (feature_id, target, weapon) in [
        ("fire-breath", target, None),
        ("foreign-feature", target, None),
        ("bite", f.actors[1], None),
        ("bite", target, Some(ItemId::new())),
    ] {
        let bad = f
            .request(
                TableTransportChannel::Host,
                action(TacticalAction::CreatureAttack {
                    target,
                    feature_id: feature_id.into(),
                    weapon,
                }),
            )
            .await;
        Box::pin(f.reject(bad)).await;
    }
    let accepted = Box::pin(f.cold_action(
        TableTransportChannel::Host,
        TacticalAction::CreatureAttack {
            target,
            feature_id: bite.feature_id.clone(),
            weapon: None,
        },
    ))
    .await;
    let before = f.state().await;
    let raw = before.rules.as_ref().unwrap().pending.clone().unwrap();
    assert!(
        resolution(&before)
            .grapple
            .as_ref()
            .unwrap()
            .cuts
            .iter()
            .all(|cut| cut.grips == vec![grip])
    );
    let pending = request(&f, TableTransportChannel::Host).await;
    Box::pin(refused(&f, pending)).await;
    let release = f
        .choose(pc, "Release Small armored figure from left hand")
        .await;
    Box::pin(f.cold(release)).await;
    assert_eq!(
        f.state().await.rules.as_ref().unwrap().pending.as_ref(),
        Some(&raw)
    );
    Box::pin(f.cold_roll(TableTransportChannel::Host, 1)).await;
    let after = f.state().await;
    let rules = after.rules.as_ref().unwrap();
    assert_eq!(
        rules
            .rolls
            .iter()
            .find(|roll| roll.request.id == raw.request.id)
            .unwrap()
            .request,
        raw.request
    );
    assert!(rules.timing.as_ref().unwrap().action_spent);
    assert!(matches!(
        Box::pin(f.runtime.submit_presented_table(accepted))
            .await
            .unwrap(),
        TableTransportResult::Accepted(_)
    ));
    Box::pin(refused(&f, host)).await;

    // Reuse this genuine settled history for source ownership and a fresh turn.
    let actor = f.goblin;
    let player = f.players[1];
    Box::pin(f.host(TableAction::SetSourceCreatureController {
        actor,
        controller: CreatureController::Player(player),
    }))
    .await;
    let owner = TableTransportChannel::SourceCreature {
        player_id: player,
        actor,
    };
    Box::pin(f.cold_action(owner.clone(), TacticalAction::EndTurn)).await;
    let pc = f.pc(0);
    Box::pin(f.cold_action(pc, TacticalAction::EndTurn)).await;
    let owned = request(&f, owner.clone()).await;
    let owned_choices = Box::pin(read(&f, owned.clone())).await;
    assert_eq!(owned_choices.features, choices.features);
    let host_now = request(&f, TableTransportChannel::Host).await;
    Box::pin(refused(&f, host_now)).await;
    let pc_same_human = request(&f, f.pc(1)).await;
    assert_eq!(pc_same_human.revision, owned.revision);
    Box::pin(refused(&f, pc_same_human)).await;
    let mut wrong_selected = owned.clone();
    wrong_selected.channel = TableTransportChannel::SourceCreature {
        player_id: player,
        actor: f.actors[1],
    };
    Box::pin(refused(&f, wrong_selected)).await;
    let unrelated = request(
        &f,
        TableTransportChannel::SourceCreature {
            player_id: f.players[0],
            actor,
        },
    )
    .await;
    Box::pin(refused(&f, unrelated)).await;
    let mut bad_version = owned.clone();
    bad_version.version = 2;
    Box::pin(refused(&f, bad_version)).await;

    // A current audience revision never grants a former controller permission.
    Box::pin(f.host(TableAction::SetSourceCreatureController {
        actor,
        controller: CreatureController::Player(f.players[0]),
    }))
    .await;
    let former = request(&f, owner.clone()).await;
    Box::pin(refused(&f, former)).await;
    let new_owner = request(
        &f,
        TableTransportChannel::SourceCreature {
            player_id: f.players[0],
            actor,
        },
    )
    .await;
    assert_eq!(
        Box::pin(read(&f, new_owner)).await.features,
        choices.features
    );
    Box::pin(f.host(TableAction::SetSourceCreatureController {
        actor,
        controller: CreatureController::Player(player),
    }))
    .await;
    let owned = request(&f, owner.clone()).await;
    let owned_choices = Box::pin(read(&f, owned.clone())).await;

    // An independent portable restore preserves original audience bytes and choices.
    let export = export_campaign(&f.pool, f.campaign).await.unwrap();
    let pool = open_sqlite("sqlite::memory:").await.unwrap();
    let mirror = runtime(pool.clone());
    Box::pin(mirror.restore_campaign(&export)).await.unwrap();
    let rows = all_rows(&pool).await;
    assert_eq!(
        Box::pin(mirror.table_intrinsic_attack_options(owned.clone()))
            .await
            .unwrap(),
        owned_choices
    );
    assert_eq!(all_rows(&pool).await, rows);
    pool.close().await;

    // The actual source player's choice uses the existing durable command too.
    Box::pin(f.cold_action(
        owner.clone(),
        TacticalAction::CreatureAttack {
            target,
            feature_id: "bite".into(),
            weapon: None,
        },
    ))
    .await;
    Box::pin(f.cold_roll(owner.clone(), 1)).await;
    let exhausted = request(&f, owner.clone()).await;
    Box::pin(refused(&f, exhausted)).await;
    Box::pin(f.cold_action(owner.clone(), TacticalAction::EndTurn)).await;
    let pc = f.pc(0);
    Box::pin(f.cold_action(pc, TacticalAction::EndTurn)).await;

    // Distinct opt-ins, same real history: no synthetic G/M current state.
    let mut upgrade = f
        .request(
            TableTransportChannel::Host,
            TableTransportInput::Action(Box::new(TableAction::EnableGrappleTransport)),
        )
        .await;
    upgrade.version = 4;
    Box::pin(f.cold(upgrade)).await;
    let g4 = request(&f, owner.clone()).await;
    assert_eq!(Box::pin(read(&f, g4)).await.features, choices.features);
    Box::pin(begin_mass_encounter(&mut f, owner.clone())).await;
    let mg4 = request(&f, owner).await;
    assert_eq!(Box::pin(read(&f, mg4)).await.features, choices.features);
    f.close().await;
}

#[tokio::test]
async fn source_candidates_keep_conditional_and_multicomponent_actions_and_exclude_physical_ogre() {
    for (definition, feature) in [
        ("warhorse", Some("hooves")),
        ("young-red-dragon", Some("rend")),
        ("air-elemental", Some("thunderous-slam")),
        ("ogre", None),
    ] {
        let mut f = Box::pin(Fixture::with_source(definition, CreatureSize::Large)).await;
        // Host may direct an Autonomous source as well as a Host-controlled one.
        let actor = f.goblin;
        Box::pin(f.host(TableAction::SetSourceCreatureController {
            actor,
            controller: CreatureController::Autonomous,
        }))
        .await;
        let pc = f.pc(0);
        Box::pin(f.cold_action(pc, TacticalAction::EndTurn)).await;
        let current = request(&f, TableTransportChannel::Host).await;
        let choices = Box::pin(read(&f, current)).await;
        if let Some(feature) = feature {
            let offered = choices
                .features
                .iter()
                .find(|choice| choice.feature_id == feature)
                .unwrap();
            let target = f.actors[0];
            Box::pin(f.cold_action(
                TableTransportChannel::Host,
                TacticalAction::CreatureAttack {
                    target,
                    feature_id: offered.feature_id.clone(),
                    weapon: None,
                },
            ))
            .await;
            if definition == "young-red-dragon" {
                assert_eq!(
                    resolution(&f.state().await)
                        .attack
                        .as_ref()
                        .unwrap()
                        .damage
                        .len(),
                    2
                );
            }
            Box::pin(f.cold_roll(TableTransportChannel::Host, 1)).await;
        } else {
            assert!(choices.features.is_empty());
        }
        f.close().await;
    }
}

#[test]
fn intrinsic_read_schema_is_separate_strict_and_has_no_command_or_proof_fields() {
    let request = TableIntrinsicAttackRequest {
        version: 1,
        campaign_id: CampaignId::new(),
        channel: TableTransportChannel::Host,
        revision: dmd_persistence::ProjectionRevision(RollRequestId::new().0),
        actor: EntityId::new(),
    };
    let value = serde_json::to_value(&request).unwrap();
    assert_eq!(value.as_object().unwrap().len(), 5);
    for key in [
        "command_id",
        "expected_event_sequence",
        "state",
        "proof",
        "feature_id",
        "target",
        "session_id",
    ] {
        let mut changed = value.clone();
        changed[key] = serde_json::json!("not-authority");
        assert!(serde_json::from_value::<TableIntrinsicAttackRequest>(changed).is_err());
    }
    let result = TableIntrinsicAttackOptions {
        version: 1,
        revision: request.revision,
        actor: request.actor,
        features: vec![],
        targets: vec![],
    };
    let mut changed = serde_json::to_value(result).unwrap();
    changed["armor_class"] = serde_json::json!(10);
    assert!(serde_json::from_value::<TableIntrinsicAttackOptions>(changed).is_err());
}
