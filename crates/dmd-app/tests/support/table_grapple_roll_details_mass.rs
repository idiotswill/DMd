//! M changes the command version, never the independent Grapple capability or read schema.
use super::*;
use dmd_persistence::ProjectionCapability;

async fn capabilities(f: &Fixture, grapple: Option<u32>) {
    let state = f.state().await;
    assert_eq!(
        dmd_rules::table::source_control::presentation_version(&state),
        5
    );
    assert_eq!(state.schema_version, 4);
    assert_eq!(state.physical_facts.as_ref().unwrap().schema_version, 1);
    assert_eq!(dmd_rules::table::grapple_enabled(&state), grapple.is_some());
    assert_eq!(
        dmd_rules::table::grapple_transport_enabled(&state),
        grapple == Some(4)
    );
    for channel in [TableTransportChannel::Host, f.pc(0), f.pc(1)] {
        let view = f.view(channel).await;
        assert_eq!(view.physical.as_ref().unwrap().version, 5);
        assert_eq!(view.grapple.as_ref().map(|g| g.version), grapple);
        assert_eq!(view.source_control.as_ref().unwrap().version, 2);
    }
    let export = export_campaign(&f.pool, f.campaign).await.unwrap();
    assert_eq!(export.table_projection_history.last().unwrap().version, 5);
    let audit = export.command_audit.last().unwrap();
    assert_eq!(audit.command_schema_version, 6);
    // Bindings are persisted in UUID order, not acceptance chronology.
    let binding = export
        .table_transport_bindings
        .iter()
        .find(|binding| binding.meta.id.0.to_string() == audit.id)
        .unwrap();
    assert_eq!(binding.version, 5);
    let request: TableTransportRequest = serde_json::from_str(&binding.request_json).unwrap();
    assert_eq!(request.command_id, binding.meta.id);
    assert_eq!(request.version, 5);
}

async fn start_mass(source_owner: bool) -> Fixture {
    let mut f = Box::pin(Fixture::new()).await;
    if source_owner {
        Box::pin(f.host(TableAction::SetSourceCreatureController {
            actor: f.goblin,
            controller: CreatureController::Player(f.players[1]),
        }))
        .await;
    }
    let previous = f.state().await;
    let encounter = previous.encounter.as_ref().unwrap();
    assert_eq!(
        encounter.flow.as_ref().unwrap().phase,
        TacticalPhase::Active
    );
    assert!(previous.rules.as_ref().unwrap().pending.is_none());
    Box::pin(submit(&mut f, TableTransportChannel::Host, action(TacticalAction::ConcludeHostilities {
        cadence: AftermathCadence::ContinueExistingOrder,
        ruling: "The actual settled participants conclude this encounter before physical facts are enabled.".into(),
    }), 2)).await;
    Box::pin(submit(
        &mut f,
        TableTransportChannel::Host,
        action(TacticalAction::FinishEncounter),
        2,
    ))
    .await;
    assert_eq!(
        f.state().await.encounter.unwrap().flow.unwrap().phase,
        TacticalPhase::Finished
    );
    assert!(f.state().await.physical_facts.is_none());
    Box::pin(submit(
        &mut f,
        TableTransportChannel::Host,
        TableTransportInput::EnablePhysicalFacts,
        5,
    ))
    .await;
    Box::pin(capabilities(&f, None)).await;
    let pc = encounter.participant(f.actors[0]).unwrap();
    let creature = encounter.participant(f.goblin).unwrap();
    let setup = TableBattlefieldSetup {
        encounter_id: EncounterId::new(),
        scene_id: SceneId::new(),
        location_id: LocationId::new(),
        name: "The retained participants begin their next encounter".into(),
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
            height: 8,
            allies: creature.allies.clone(),
            enemies: creature.enemies.clone(),
        }],
        geometry_ruling: Ruling {
            basis: RulingBasis::GmAdjudication,
            reason: "Host establishes the retained visible geometry in a genuinely new scene."
                .into(),
        },
        area_grid_policy: None,
    };
    assert_ne!(setup.encounter_id, encounter.id);
    assert_ne!(setup.scene_id, encounter.scene_id);
    Box::pin(submit(
        &mut f,
        TableTransportChannel::Host,
        TableTransportInput::Action(Box::new(TableAction::PrepareBattlefield {
            setup: Box::new(setup),
        })),
        5,
    ))
    .await;
    let pc = f.actors[0];
    let goblin = f.goblin;
    Box::pin(submit(
        &mut f,
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
                    actor: goblin,
                    source: TacticalSource::Creature {
                        definition_id: "goblin-warrior".into(),
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
                    actors: vec![goblin],
                    request_id: RollRequestId::new(),
                },
            ],
        }),
        5,
    ))
    .await;
    let current = f.state().await;
    let old_source = previous
        .rules
        .as_ref()
        .unwrap()
        .tactical_creatures
        .as_ref()
        .unwrap();
    let source = current
        .rules
        .as_ref()
        .unwrap()
        .tactical_creatures
        .as_ref()
        .unwrap();
    assert_eq!(source.profiles, old_source.profiles);
    assert_eq!(
        source.runtime(goblin).unwrap().controller,
        old_source.runtime(goblin).unwrap().controller
    );
    assert_eq!(
        source.runtime(goblin).unwrap().control_origin,
        old_source.runtime(goblin).unwrap().control_origin
    );
    assert!(
        current
            .encounter
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .attack_equipment_access
            .is_none()
    );
    assert!(matches!(
        current
            .rules
            .as_ref()
            .unwrap()
            .pending
            .as_ref()
            .unwrap()
            .purpose,
        PendingPurpose::TacticalInitiative { .. }
    ));
    Box::pin(capabilities(&f, None)).await;
    f
}

async fn initiative(f: &mut Fixture, source_owner: bool, grapple: Option<u32>) {
    for (channel, face) in [
        (f.pc(0), 18),
        (
            if source_owner {
                source(f)
            } else {
                TableTransportChannel::Host
            },
            2,
        ),
    ] {
        let roll = f.view(channel.clone()).await.roll.unwrap();
        assert_eq!(roll.reason, "Initiative");
        assert_eq!(roll.mode, RollMode::Normal);
        Box::pin(submit(
            f,
            channel,
            action(TacticalAction::SubmitRoll {
                result: RollResult {
                    request_id: roll.id,
                    source: RollSource::Physical,
                    dice: vec![DieResult {
                        sides: 20,
                        value: face,
                    }],
                },
            }),
            5,
        ))
        .await;
    }
    let state = f.state().await;
    assert_eq!(
        state
            .encounter
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .phase,
        TacticalPhase::Active
    );
    assert_eq!(
        state.rules.as_ref().unwrap().timing.as_ref().unwrap().order[0].actor,
        f.actors[0]
    );
    assert!(state.rules.as_ref().unwrap().pending.is_none());
    Box::pin(capabilities(f, None)).await;
    // Real G/T admission requires this settled Active turn, never Finished.
    if let Some(version) = grapple {
        assert!(matches!(version, 3 | 4));
        Box::pin(submit(
            f,
            TableTransportChannel::Host,
            TableTransportInput::Action(Box::new(TableAction::EnableGrappleAccess)),
            5,
        ))
        .await;
        Box::pin(capabilities(f, Some(3))).await;
        if version == 4 {
            Box::pin(submit(
                f,
                TableTransportChannel::Host,
                TableTransportInput::Action(Box::new(TableAction::EnableGrappleTransport)),
                5,
            ))
            .await;
            Box::pin(capabilities(f, Some(4))).await;
        }
    }
    Box::pin(submit(
        f,
        TableTransportChannel::Host,
        action(TacticalAction::ActivateAttackEquipment),
        5,
    ))
    .await;
    assert!(
        f.state()
            .await
            .encounter
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .attack_equipment_access
            .is_some()
    );
    Box::pin(capabilities(f, grapple)).await;
}

async fn host_cannot_answer(f: &Fixture, owner: &TableTransportChannel) {
    let export = export_campaign(&f.pool, f.campaign).await.unwrap();
    let rows = all_rows(&f.pool).await;
    let pending = f.state().await.rules.unwrap().pending.unwrap();
    let host = f.view(TableTransportChannel::Host).await;
    let own = f.view(owner.clone()).await;
    let host_roll = host.roll.as_ref().unwrap();
    assert_ne!(host_roll.id, own.roll.as_ref().unwrap().id);
    for view in [&host, &own] {
        let projection = export
            .table_projection_history
            .iter()
            .rev()
            .flat_map(|p| &p.changes)
            .find(|c| c.revision == view.revision)
            .unwrap();
        let handle = projection
            .handles
            .iter()
            .find(|h| h.opaque == view.roll.as_ref().unwrap().id.0)
            .unwrap();
        assert_eq!(
            handle.capability,
            ProjectionCapability::Roll {
                canonical: pending.request.id
            }
        );
    }
    let mut request = f
        .request(
            TableTransportChannel::Host,
            action(TacticalAction::SubmitRoll {
                result: RollResult {
                    request_id: host_roll.id,
                    source: RollSource::Physical,
                    dice: vec![DieResult {
                        sides: 20,
                        value: 20,
                    }],
                },
            }),
        )
        .await;
    request.version = 5;
    assert!(
        matches!(Box::pin(f.runtime.submit_presented_table(request)).await,
        Err(RunnableCampaignError::TableRejected(message)) if message == "issuer is not authorized for this action")
    );
    Box::pin(unchanged(f, &export, &rows)).await;
}

async fn matrix(source_owner: bool, capability: u32) {
    for escape in [false, true] {
        for choice in if escape {
            ["Athletics", "Acrobatics"]
        } else {
            ["Strength", "Dexterity"]
        } {
            let mut f = Box::pin(start_mass(source_owner)).await;
            Box::pin(initiative(&mut f, source_owner, Some(capability))).await;
            let inspired = capability == 4 && !source_owner;
            if inspired {
                let character_id = f.characters[0];
                Box::pin(submit(
                    &mut f,
                    TableTransportChannel::Host,
                    TableTransportInput::Action(Box::new(TableAction::AwardHeroicInspiration {
                        character_id,
                        reason: "For cooperating with a companion in this actual new encounter."
                            .into(),
                    })),
                    5,
                ))
                .await;
            }
            let owner = Box::pin(pending_save(
                &mut f,
                source_owner,
                if escape { "Strength" } else { choice },
                5,
            ))
            .await;
            let label = if escape {
                Box::pin(submit(
                    &mut f,
                    owner.clone(),
                    action(TacticalAction::VoluntarilyFailSave),
                    5,
                ))
                .await;
                let attacker = if source_owner {
                    f.pc(0)
                } else {
                    TableTransportChannel::Host
                };
                Box::pin(choose(
                    &mut f,
                    attacker.clone(),
                    "Finish without changing equipment",
                    5,
                ))
                .await;
                Box::pin(submit(&mut f, attacker, action(TacticalAction::EndTurn), 5)).await;
                let offer = f
                    .view(owner.clone())
                    .await
                    .grapple
                    .unwrap()
                    .choices
                    .into_iter()
                    .find(|o| o.label.ends_with(&format!("using {choice}")))
                    .unwrap();
                Box::pin(submit(
                    &mut f,
                    owner.clone(),
                    TableTransportInput::GrappleChoice { handle: offer.key },
                    5,
                ))
                .await;
                if choice == "Athletics" {
                    "Strength (Athletics) Escape"
                } else {
                    "Dexterity (Acrobatics) Escape"
                }
            } else {
                "Grapple saving throw"
            };
            let state = f.state().await;
            let pending = state.rules.as_ref().unwrap().pending.clone().unwrap();
            assert!(
                matches!(&pending.purpose, PendingPurpose::TacticalResolution { key, .. }
                if key.role == if escape { TacticalRollRole::GrappleEscape } else { TacticalRollRole::GrappleSave })
            );
            Box::pin(capabilities(&f, Some(capability))).await;
            // This unchanged helper expects the independent G capability, not command5.
            let read = Box::pin(read_pending(
                &mut f,
                owner.clone(),
                label,
                capability,
                inspired,
            ))
            .await;
            Box::pin(host_cannot_answer(&f, &owner)).await;
            let accepted = Box::pin(physical(&mut f, owner, 5, inspired)).await;
            assert_eq!(accepted.version, 5);
            let after = f.state().await;
            let rules = after.rules.as_ref().unwrap();
            let paid = rules.rolls.last().unwrap();
            assert_eq!(
                rules.rolls.len(),
                state.rules.as_ref().unwrap().rolls.len() + 1
            );
            assert_eq!(paid.request, pending.request);
            assert_eq!(paid.purpose, pending.purpose);
            assert_eq!(paid.issued_by, pending.issued_by);
            assert_eq!(paid.accepted_by.id, accepted.command_id);
            assert_eq!(paid.result.source, RollSource::Physical);
            assert_eq!(
                paid.result.dice,
                vec![DieResult {
                    sides: 20,
                    value: 20
                }]
            );
            assert_eq!(paid.resolved.total, 20 + pending.request.modifier);
            assert_eq!(paid.original_result.is_some(), inspired);
            if inspired {
                assert_eq!(
                    paid.original_result.as_ref().unwrap().source,
                    RollSource::Physical
                );
                assert_eq!(
                    paid.original_result.as_ref().unwrap().dice,
                    vec![DieResult {
                        sides: 20,
                        value: 1
                    }]
                );
                assert!(!rules.entities[&f.actors[0]].heroic_inspiration);
            }
            Box::pin(refused(&f, read)).await;
            Box::pin(capabilities(&f, Some(capability))).await;
            if !escape {
                let attacker = if source_owner {
                    f.pc(0)
                } else {
                    TableTransportChannel::Host
                };
                Box::pin(choose(
                    &mut f,
                    attacker,
                    "Finish without changing equipment",
                    5,
                ))
                .await;
            }
            f.close().await;
        }
    }
}

#[tokio::test]
async fn actual_m_g3_pc_save_and_escape_details_preserve_capabilities_and_physical_retry() {
    Box::pin(matrix(false, 3)).await;
}
#[tokio::test]
async fn actual_m_g4_pc_save_and_escape_details_preserve_owned_inspiration_and_retry() {
    Box::pin(matrix(false, 4)).await;
}
#[tokio::test]
async fn actual_m_g3_source_save_and_escape_details_preserve_owned_source_and_retry() {
    Box::pin(matrix(true, 3)).await;
}
#[tokio::test]
async fn actual_m_g4_source_save_and_escape_details_preserve_owned_source_and_retry() {
    Box::pin(matrix(true, 4)).await;
}
#[tokio::test]
async fn actual_mass_only_initiative_details_do_not_grant_grapple_or_inspiration() {
    let mut f = Box::pin(start_mass(false)).await;
    let request = details_request(&f, f.pc(0)).await;
    let before = export_campaign(&f.pool, f.campaign).await.unwrap();
    let rows = all_rows(&f.pool).await;
    let original = f.view(f.pc(0)).await;
    let options = Box::pin(f.runtime.table_roll_options(TableRollOptionsRequest {
        campaign_id: request.campaign_id,
        channel: request.channel.clone(),
        revision: request.revision,
        roll_id: request.roll_id,
    }))
    .await
    .unwrap();
    let answer = Box::pin(f.runtime.table_roll_details(request.clone()))
        .await
        .unwrap();
    assert_eq!(answer.version, 1);
    assert_eq!(answer.options, options);
    assert_eq!(
        serde_json::to_value(&options).unwrap(),
        serde_json::json!({"savage_attacker":null})
    );
    assert_eq!(answer.display_reason, "Initiative");
    assert_eq!(original.roll.as_ref().unwrap().reason, "Initiative");
    assert_eq!(f.view(f.pc(0)).await, original);
    Box::pin(unchanged(&f, &before, &rows)).await;
    f.pool.close().await;
    f.pool = open_sqlite_path(&f.path).await.unwrap();
    f.runtime = runtime(f.pool.clone());
    assert_eq!(
        Box::pin(f.runtime.table_roll_details(request.clone()))
            .await
            .unwrap(),
        answer
    );
    Box::pin(unchanged(&f, &before, &rows)).await;
    let pool = open_sqlite("sqlite::memory:").await.unwrap();
    let mirror = runtime(pool.clone());
    Box::pin(mirror.restore_campaign(&before)).await.unwrap();
    let cells = all_rows(&pool).await;
    assert_eq!(
        Box::pin(mirror.table_roll_details(request.clone()))
            .await
            .unwrap(),
        answer
    );
    let mut portable = export_campaign(&pool, f.campaign).await.unwrap();
    portable.exported_at_utc = before.exported_at_utc.clone();
    assert_eq!(portable, before);
    assert_eq!(all_rows(&pool).await, cells);
    pool.close().await;
    Box::pin(initiative(&mut f, false, None)).await;
    Box::pin(refused(&f, request)).await;
    Box::pin(capabilities(&f, None)).await;
    assert!(f.view(f.pc(0)).await.inspiration_transfer.is_none());
    f.close().await;
}
