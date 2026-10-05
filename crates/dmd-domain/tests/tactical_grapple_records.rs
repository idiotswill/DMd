//! Synthetic structural controls. These fixtures are not accepted Grapple history;
//! the source/domain checkpoint must still refuse them at the runnable boundary.
use dmd_domain::*;

fn meta(sequence: u64) -> CommandMeta {
    CommandMeta {
        id: CommandId::new(),
        campaign_id: CampaignId(uuid::Uuid::from_u128(1)),
        session_id: None,
        issuer: CommandIssuer::Admin,
        actor: None,
        expected_event_sequence: sequence,
    }
}

fn grip() -> TacticalGrip {
    let origin = meta(1);
    let chosen = meta(2);
    let target = EntityId::new();
    let grappler = EntityId::new();
    let key = TacticalRollKey {
        origin: origin.id,
        role: TacticalRollRole::GrappleSave,
        subject: target,
        occurrence: 4,
    };
    TacticalGrip {
        declaration: TacticalGrappleDeclaration {
            id: GrappleId::from_declaration(origin.id, grappler, target, Hand::Left),
            window: WeaponActionWindow {
                id: origin.id,
                kind: WeaponActionKind::AttackAction,
            },
            origin,
            grappler,
            target,
            hand: Hand::Left,
            anatomy: GrappleAnatomyProof::HumanCreationV1 {
                character: CharacterId::new(),
            },
            target_source: None,
            grappler_from: SpatialPoint { x: 10, y: 10, z: 0 },
            target_from: SpatialPoint { x: 20, y: 10, z: 0 },
            range: 10,
            escape_dc: 13,
        },
        established_by: chosen.clone(),
        // A prior retired resolution is intentionally not today's consumer root.
        work: TacticalWorkKey {
            resolution: CommandId::new(),
            occurrence: 4,
        },
        save: TacticalGrappleSave {
            ability: GrappleSaveAbility::Strength,
            chosen_by: chosen.clone(),
            key,
            request: None,
            proof: Some(GrappleSaveProof {
                evidence: GrappleSaveEvidence::Decision(TacticalSaveDecision {
                    key,
                    issued_by: chosen.clone(),
                    resolved_by: chosen.clone(),
                    failure: TacticalSaveFailure::Automatic,
                }),
                legendary: None,
                final_success: false,
                finalized_by: chosen,
            }),
        },
    }
}

fn live(grip: TacticalGrip) -> TacticalGrapples {
    TacticalGrapples {
        schema_version: TACTICAL_GRAPPLES_SCHEMA_VERSION,
        active: vec![grip],
    }
}

fn resolution() -> TacticalResolution {
    TacticalResolution {
        grapple: None,
        origin: meta(5),
        turn_actor: EntityId::new(),
        turn_number: 1,
        boundary: TurnBoundary::Start,
        frames: vec![],
        pending: None,
        failed_save: None,
        legendary_window: None,
        attack: None,
        shove: None,
        hit_review: None,
        movement: None,
        casts: vec![],
        missiles: vec![],
        falls: vec![],
        areas: vec![],
        work_trace: Some(TacticalWorkTrace {
            nodes: vec![
                TacticalWorkNode {
                    work: TacticalWorkItem {
                        occurrence: 1,
                        kind: TacticalWorkKind::AttackRoll,
                    },
                    parent: None,
                },
                TacticalWorkNode {
                    work: TacticalWorkItem {
                        occurrence: 2,
                        kind: TacticalWorkKind::AttackDamage,
                    },
                    parent: Some(1),
                },
            ],
            active: None,
        }),
        next_occurrence: 3,
    }
}

fn context(proof: TacticalGrip) -> TacticalGrappleResolution {
    TacticalGrappleResolution {
        activity: None,
        proofs: vec![proof],
        cuts: vec![],
        ends: vec![],
        opportunity_refreshes: vec![],
    }
}

#[test]
fn automatic_voluntary_and_physical_evidence_remain_distinct() {
    let automatic = grip();
    automatic.validate_shape().unwrap();
    let json = serde_json::to_string(&automatic).unwrap();
    assert_eq!(
        serde_json::from_str::<TacticalGrip>(&json).unwrap(),
        automatic
    );
    assert!(!json.contains("original_result"));
    let mut voluntary = automatic.clone();
    voluntary.save.request = Some(RollRequest {
        id: voluntary.save.key.request_id(),
        roller: Some(voluntary.declaration.target),
        dice: vec![DieSpec {
            count: 1,
            sides: 20,
        }],
        modifier: 2,
        mode: RollMode::Normal,
        visibility: RollVisibility::Secret,
        reason: "Synthetic source save".into(),
    });
    let GrappleSaveEvidence::Decision(decision) =
        &mut voluntary.save.proof.as_mut().unwrap().evidence
    else {
        unreachable!()
    };
    decision.failure = TacticalSaveFailure::Voluntary;
    voluntary.validate_shape().unwrap();
    let mut bad = voluntary.clone();
    bad.save.request = None;
    assert!(bad.validate_shape().is_err());
    let mut bad = automatic.clone();
    bad.save.request = voluntary.save.request.clone();
    assert!(bad.validate_shape().is_err());
    let mut physical = voluntary;
    physical.save.proof.as_mut().unwrap().evidence = GrappleSaveEvidence::Physical {
        key: physical.save.key,
        accepted_by: physical.established_by.clone(),
    };
    physical.validate_shape().unwrap();
    let mut bad = physical;
    bad.save.request.as_mut().unwrap().roller = Some(bad.declaration.grappler);
    assert!(bad.validate_shape().is_err());
    let mut resisted = automatic;
    let proof = resisted.save.proof.as_mut().unwrap();
    let choice = meta(3);
    proof.legendary = Some(GrappleLegendaryDecision {
        chosen_by: choice.clone(),
        use_resistance: true,
    });
    proof.final_success = true;
    proof.finalized_by = choice;
    resisted.save.validate_shape(&resisted.declaration).unwrap();
    assert!(
        resisted.validate_shape().is_err(),
        "a resisted attempt is never a live grip"
    );
}

#[test]
fn pending_attempt_reserves_its_exact_hand_without_any_live_attachment() {
    let g = grip();
    let r = resolution();
    let attempt = TacticalGrappleAttempt {
        declaration: g.declaration.clone(),
        stage: TacticalGrappleAttemptStage::SaveChoice,
        selected: None,
        save: None,
        outcome: None,
    };
    assert_eq!(
        attempt.reservation(),
        Some((g.declaration.id, g.declaration.grappler, Hand::Left))
    );
    let mut c = context(g.clone());
    c.proofs.clear();
    c.activity = Some(GrappleActivity::Attempt(Box::new(attempt)));
    assert!(!c.is_empty());
    c.validate_shape(&r, None).unwrap();
    assert!(c.validate_shape(&r, Some(&live(g))).is_err());
    let GrappleActivity::Attempt(attempt) = c.activity.as_mut().unwrap() else {
        unreachable!()
    };
    attempt.stage = TacticalGrappleAttemptStage::Complete;
    attempt.outcome = Some(GrappleAttemptOutcome::Withdrawn {
        withdrawn_by: meta(6),
        cancelled: None,
    });
    assert_eq!(attempt.reservation(), None);
    c.validate_shape(&r, None).unwrap();
}

#[test]
fn one_hand_one_grip_and_canonical_nonempty_authority_are_enforced() {
    let g = grip();
    let mut value = live(g.clone());
    value.validate_shape().unwrap();
    value.active.push(g.clone());
    assert!(value.validate_shape().is_err());
    value.active.clear();
    assert!(value.validate_shape().is_err());
    let mut other = grip();
    other.declaration.grappler = g.declaration.grappler;
    other.declaration.id = GrappleId::from_declaration(
        other.declaration.origin.id,
        other.declaration.grappler,
        other.declaration.target,
        other.declaration.hand,
    );
    value.active = vec![g, other];
    value.active.sort_by_key(|g| g.declaration.id.0);
    assert!(
        value.validate_shape().is_err(),
        "different targets still cannot reuse one hand"
    );
}

#[test]
fn nested_work_and_paid_attack_origins_are_distinct_and_inheritance_is_direct() {
    let g = grip();
    let r = resolution();
    let admission = meta(6);
    let attack_key = GrappleCutKey {
        work: TacticalWorkKey {
            resolution: r.origin.id,
            occurrence: 1,
        },
        reader: GrappleReader::AttackAdmission {
            attack: admission.id,
        },
    };
    let roll = TacticalRollKey {
        origin: admission.id,
        role: TacticalRollRole::AttackDamage,
        subject: g.declaration.target,
        occurrence: 2,
    };
    let request_key = GrappleCutKey {
        work: TacticalWorkKey {
            resolution: r.origin.id,
            occurrence: 2,
        },
        reader: GrappleReader::RequestIssue { roll },
    };
    assert_ne!(roll.origin, request_key.work.resolution);
    let mut c = context(g.clone());
    c.cuts = vec![
        GrappleReadCut {
            key: attack_key,
            issued_by: admission,
            grips: vec![g.declaration.id],
            source_attack: None,
        },
        GrappleReadCut {
            key: request_key,
            issued_by: meta(8),
            grips: vec![g.declaration.id],
            source_attack: Some(attack_key),
        },
    ];
    // Damage may issue after release but belongs to the earlier admitted attack.
    c.ends.push(GrappleEndReceipt {
        grip: g.declaration.id,
        caused_by: meta(7),
        cause: GrappleEndCause::Released,
    });
    c.validate_shape(&r, None).unwrap();
    for edit in 0..5 {
        let mut bad = c.clone();
        match edit {
            0 => bad.cuts[1].source_attack = Some(request_key),
            1 => bad.cuts[0].source_attack = Some(request_key),
            2 => bad.cuts[1].key.work.resolution = roll.origin,
            3 => {
                bad.cuts[0].key.reader = GrappleReader::AttackAdmission {
                    attack: CommandId::new(),
                }
            }
            _ => bad.cuts[1].source_attack = None,
        }
        assert!(bad.validate_shape(&r, None).is_err(), "mutation {edit}");
    }
}

#[test]
fn same_command_ending_needs_a_real_later_causal_child() {
    let mut g = grip();
    let r = resolution();
    g.established_by = r.origin.clone();
    g.work = TacticalWorkKey {
        resolution: r.origin.id,
        occurrence: 1,
    };
    let mut c = context(g.clone());
    c.cuts.push(GrappleReadCut {
        key: GrappleCutKey {
            work: g.work,
            reader: GrappleReader::FlightLoss {
                actor: g.declaration.target,
            },
        },
        issued_by: r.origin.clone(),
        grips: vec![g.declaration.id],
        source_attack: None,
    });
    c.ends.push(GrappleEndReceipt {
        grip: g.declaration.id,
        caused_by: r.origin.clone(),
        cause: GrappleEndCause::OutOfRange {
            work: TacticalWorkKey {
                resolution: r.origin.id,
                occurrence: 2,
            },
            moved_actor: g.declaration.target,
        },
    });
    c.validate_shape(&r, None).unwrap();
    let mut bad = r.clone();
    bad.work_trace.as_mut().unwrap().nodes[1].parent = None;
    assert!(c.validate_shape(&bad, None).is_err());
    c.ends[0].cause = GrappleEndCause::Released;
    assert!(c.validate_shape(&r, None).is_err());
}

#[test]
fn retired_proofs_do_not_reserve_hands_and_cannot_replace_live_authority() {
    let g = grip();
    let r = resolution();
    let mut c = context(g.clone());
    c.ends.push(GrappleEndReceipt {
        grip: g.declaration.id,
        caused_by: meta(6),
        cause: GrappleEndCause::Released,
    });
    c.validate_shape(&r, None).unwrap();
    assert!(c.activity.is_none());
    assert!(c.validate_shape(&r, Some(&live(g))).is_err());
    c.ends.clear();
    assert!(c.validate_shape(&r, None).is_err());
}

#[test]
fn ended_grip_proof_serves_self_only_route_live_menu_and_causal_fall_consumers() {
    let mut g = grip();
    let mut r = resolution();
    g.established_by = r.origin.clone();
    g.work = TacticalWorkKey {
        resolution: r.origin.id,
        occurrence: 1,
    };
    let window_origin = meta(6);
    let refresh_work = TacticalWorkKey {
        resolution: r.origin.id,
        occurrence: 3,
    };
    r.work_trace.as_mut().unwrap().nodes.push(TacticalWorkNode {
        work: TacticalWorkItem {
            occurrence: 3,
            kind: TacticalWorkKind::MovementOpportunity {
                reactor: g.declaration.grappler,
            },
        },
        parent: Some(1),
    });
    r.next_occurrence = 4;
    let unarmed = TacticalMeleeOption {
        source: TacticalMeleeSource::Unarmed,
        reach: 10,
    };
    // Pure planner shape: current source/starter acquisition does not establish
    // a genuine two-handed melee-weapon menu-change scenario yet.
    let weapon = TacticalMeleeOption {
        source: TacticalMeleeSource::Weapon {
            item: ItemId::new(),
        },
        reach: 20,
    };
    let options = vec![unarmed.clone(), weapon];
    r.movement = Some(Box::new(TacticalMovement {
        grapple_self_only: None,
        origin: r.origin.clone(),
        actor: g.declaration.target,
        path: vec![],
        initial_position: g.declaration.grappler_from,
        initial_spent: 0,
        initial_progress: TacticalMovementProgress::default(),
        initial_progress_origin: None,
        next_step: 0,
        traversed: vec![],
        offered: vec![],
        decisions: vec![],
        opportunity: Some(TacticalOpportunityWindow {
            origin: window_origin.clone(),
            reactor: g.declaration.grappler,
            mover: g.declaration.target,
            step_index: 0,
            from: g.declaration.target_from,
            to: SpatialPoint { x: 40, y: 10, z: 0 },
            options: options.clone(),
        }),
    }));
    let mut c = context(g.clone());
    let cut = GrappleCutKey {
        work: g.work,
        reader: GrappleReader::FlightLoss {
            actor: g.declaration.target,
        },
    };
    c.cuts.push(GrappleReadCut {
        key: cut,
        issued_by: g.established_by.clone(),
        grips: vec![g.declaration.id],
        source_attack: None,
    });
    c.ends.push(GrappleEndReceipt {
        grip: g.declaration.id,
        caused_by: meta(7),
        cause: GrappleEndCause::Released,
    });
    c.opportunity_refreshes.push(GrappleOpportunityRefresh {
        work: refresh_work,
        movement_origin: r.origin.clone(),
        window_origin,
        reactor: g.declaration.grappler,
        step: 0,
        ended_grip: g.declaration.id,
        previous: vec![unarmed],
        resulting: options,
    });
    r.falls.push(TacticalFall {
        origin: r.origin.clone(),
        actor: g.declaration.target,
        cause: TacticalFallCause::GrappleFlightLost {
            grip: g.declaration.id,
            consequence: g.established_by.clone(),
            work: g.work,
            cut,
        },
        path: SpatialFall {
            from: SpatialPoint {
                x: 20,
                y: 10,
                z: 40,
            },
            to: g.declaration.target_from,
            surface: FallSurface::Floor,
        },
        stage: TacticalFallStage::Queued,
    });
    c.validate_shape(&r, None).unwrap();
    // Separate synthetic movement reader: the holder's admitted self-only route
    // has no opportunity against itself, and survives the relation's ending.
    let mut route = r.clone();
    let movement = route.movement.as_mut().unwrap();
    movement.actor = g.declaration.grappler;
    movement.opportunity = None;
    movement.grapple_self_only = Some(GrappleSelfOnlyAdmission {
        origin: movement.origin.clone(),
        grips: vec![g.declaration.id],
    });
    let mut route_context = c.clone();
    route_context.opportunity_refreshes.clear();
    route_context.validate_shape(&route, None).unwrap();
    let mut bad = route.clone();
    bad.movement
        .as_mut()
        .unwrap()
        .grapple_self_only
        .as_mut()
        .unwrap()
        .origin = meta(8);
    assert!(route_context.validate_shape(&bad, None).is_err());
    route.movement.as_mut().unwrap().actor = EntityId::new();
    assert!(route_context.validate_shape(&route, None).is_err());
    for edit in 0..3 {
        let mut bad = r.clone();
        match edit {
            0 => bad
                .movement
                .as_mut()
                .unwrap()
                .opportunity
                .as_mut()
                .unwrap()
                .options
                .clear(),
            1 => bad.falls[0].actor = EntityId::new(),
            _ => {
                let TacticalFallCause::GrappleFlightLost { work, .. } = &mut bad.falls[0].cause
                else {
                    unreachable!()
                };
                work.occurrence = 2;
            }
        }
        assert!(
            c.validate_shape(&bad, None).is_err(),
            "consumer mutation {edit}"
        );
    }
    let mut bad = c.clone();
    bad.proofs.clear();
    assert!(bad.validate_shape(&r, None).is_err());
    let mut bad = c;
    let mut broken = bad.opportunity_refreshes[0].clone();
    broken.previous.clear();
    bad.opportunity_refreshes.push(broken);
    assert!(bad.validate_shape(&r, None).is_err());
}

#[test]
fn empty_and_unknown_record_shapes_fail_without_changing_absent_resolution_json() {
    let r = resolution();
    let value = serde_json::to_value(&r).unwrap();
    assert!(value.get("grapple").is_none());
    let decoded: TacticalResolution = serde_json::from_value(value.clone()).unwrap();
    assert_eq!(serde_json::to_value(decoded).unwrap(), value);
    let empty = TacticalGrappleResolution {
        activity: None,
        proofs: vec![],
        cuts: vec![],
        ends: vec![],
        opportunity_refreshes: vec![],
    };
    assert!(empty.validate_shape(&r, None).is_err());
    let mut forged = serde_json::to_value(grip()).unwrap();
    forged["trusted_roll_mode"] = serde_json::json!("Advantage");
    assert!(serde_json::from_value::<TacticalGrip>(forged).is_err());
}

#[test]
fn new_roll_roles_use_only_the_reviewed_append_only_tags() {
    let origin = CommandId(uuid::Uuid::from_u128(7));
    let subject = EntityId(uuid::Uuid::from_u128(9));
    for (role, tag) in [
        (TacticalRollRole::ShoveSave, 18),
        (TacticalRollRole::GrappleSave, 19),
        (TacticalRollRole::GrappleEscape, 20),
    ] {
        let key = TacticalRollKey {
            origin,
            subject,
            role,
            occurrence: 3,
        };
        let mut bytes = b"dmd.tactical.roll.v1\0".to_vec();
        bytes.push(tag);
        bytes.extend_from_slice(subject.0.as_bytes());
        bytes.extend_from_slice(&3u16.to_be_bytes());
        assert_eq!(
            key.request_id(),
            RollRequestId(uuid::Uuid::new_v5(&origin.0, &bytes))
        );
    }
}
