//! Synthetic local evidence controls, never accepted history or source grants.
//! The unchanged application three-ray test owns the actual producer/replay proof.
use super::*;

fn completed_source() -> CampaignState {
    let mut state = crate::tactical_hands::tests::source_state();
    let actor = crate::tactical_hands::tests::human(&state);
    let declaration = crate::tactical_hands::tests::install_attempt(&mut state, actor, Hand::Left);
    let origin = declaration.origin;
    let target = declaration.target;
    // Authored mechanics for a source-validator unit fixture only. No replay,
    // owner or application accepts this modified historical state.
    let entity = state
        .rules
        .as_mut()
        .unwrap()
        .entities
        .get_mut(&actor)
        .unwrap();
    entity.prepared_spells.insert("scorching-ray".into());
    entity.spellcasting = Some(Spellcasting {
        ability: Ability::Wisdom,
        slot_maxima: [4; 9],
        slots: [4; 9],
        can_speak: true,
        free_hand: true,
        material_focus: true,
    });
    let plan = crate::tactical_spells::plan_spell_cast_at(
        &state,
        &origin,
        &SpellCastChoice {
            actor,
            spell_id: "scorching-ray".into(),
            grant: SpellGrantChoice::Prepared,
            resource: SpellResourceChoice::Slot { level: 2 },
            material: SpellMaterialChoice::None,
            mode: SpellCastMode::Immediate,
        },
        0,
    )
    .unwrap();
    let record = TacticalCasting {
        cast: Box::new(SpellCast {
            plan,
            phase: SpellCastPhase::Committed,
            started_at: state.clock.now,
            started_on_turn: 1,
            last_operation: origin.clone(),
        }),
        selection: Some(SpellTargetChoice::Entities(vec![target; 3])),
        targets: vec![
            SpellBoundTarget {
                actor: target,
                source_type_matches: true
            };
            3
        ],
        consumed_material: None,
        creature_activation: None,
        completed: (0..3)
            .map(|target| SpellProgramOccurrence { node: 0, target })
            .collect(),
    };
    crate::tactical_spells::validate_retained_spell(&record).unwrap();
    let mut finished = origin.clone();
    finished.id = CommandId::new();
    finished.expected_event_sequence += 2;
    state.applied_event_sequence = finished.expected_event_sequence;
    let r = resolution_mut(&mut state).unwrap();
    r.next_occurrence = 8;
    let mut trace = TacticalWorkTrace::default();
    trace.nodes.push(TacticalWorkNode {
        work: TacticalWorkItem {
            occurrence: 1,
            kind: TacticalWorkKind::FinishSpell { cast: 0 },
        },
        parent: None,
    });
    for target in 0..3 {
        trace.nodes.push(TacticalWorkNode {
            work: TacticalWorkItem {
                occurrence: target + 2,
                kind: TacticalWorkKind::SpellProgram {
                    cast: 0,
                    at: SpellProgramOccurrence { node: 0, target },
                },
            },
            parent: None,
        });
    }
    let context = r.grapple.as_mut().unwrap();
    context.activity = None;
    for target in 0..3 {
        trace.nodes.push(TacticalWorkNode {
            work: TacticalWorkItem {
                occurrence: target + 5,
                kind: TacticalWorkKind::AttackRoll,
            },
            parent: Some(target + 2),
        });
        context.cuts.push(GrappleReadCut {
            key: GrappleCutKey {
                work: TacticalWorkKey {
                    resolution: origin.id,
                    occurrence: target + 5,
                },
                reader: GrappleReader::AttackAdmission { attack: origin.id },
            },
            issued_by: origin.clone(),
            grips: vec![],
            source_attack: None,
        });
    }
    context.completed_casts.push(GrappleCompletedCast {
        record: Box::new(record),
        work: TacticalWorkKey {
            resolution: origin.id,
            occurrence: 1,
        },
        finished_by: finished,
    });
    r.work_trace = Some(trace);
    state
}

#[test]
fn completed_source_rejects_forged_program_target_finish_and_live_work() {
    let state = completed_source();
    validate(&state).unwrap(); // Local source consistency, not execution authority.
    for case in [
        "program",
        "target",
        "cast",
        "finish-key",
        "finish-parent",
        "unfinished",
        "duplicate",
        "live-cast",
        "queued",
        "queued-attack",
        "orphan",
        "finish-origin",
        "finish-before-read",
    ] {
        let mut changed = state.clone();
        let r = resolution_mut(&mut changed).unwrap();
        let c = r.grapple.as_mut().unwrap();
        match case {
            "program" => c.completed_casts[0]
                .record
                .cast
                .plan
                .program
                .source
                .fingerprint
                .push('x'),
            "target" => c.completed_casts[0].record.targets[0].actor = EntityId::new(),
            "cast" => c.completed_casts[0].record.cast.plan.occurrence += 1,
            "finish-key" => c.completed_casts[0].work.occurrence = 5,
            "finish-parent" => r.work_trace.as_mut().unwrap().nodes[0].parent = Some(2),
            "unfinished" => {
                c.completed_casts[0].record.completed.pop();
            }
            "duplicate" => c.completed_casts.push(c.completed_casts[0].clone()),
            "live-cast" => r.casts.push(*c.completed_casts[0].record.clone()),
            "queued" => r
                .frames
                .push(vec![r.work_trace.as_ref().unwrap().nodes[0].work.clone()]),
            "queued-attack" => r
                .frames
                .push(vec![r.work_trace.as_ref().unwrap().nodes[4].work.clone()]),
            "orphan" => c.cuts.clear(),
            "finish-origin" => c.completed_casts[0].finished_by.campaign_id = CampaignId::new(),
            "finish-before-read" => c.cuts[0].issued_by.expected_event_sequence += 3,
            _ => unreachable!(),
        }
        assert!(validate(&changed).is_err(), "accepted forged {case}");
    }
}

#[test]
fn completion_requires_entered_finish_and_empty_receipts_keep_legacy_bytes() {
    let mut state = completed_source();
    let r = resolution_mut(&mut state).unwrap();
    let receipt = r.grapple.as_mut().unwrap().completed_casts.pop().unwrap();
    r.casts.push(*receipt.record.clone());
    let empty = r.grapple.as_ref().unwrap();
    let json = serde_json::to_value(empty).unwrap();
    assert!(json.get("completed_casts").is_none());
    assert_eq!(
        serde_json::from_value::<TacticalGrappleResolution>(json).unwrap(),
        **empty
    );
    let before = state.clone();
    assert!(retain(&mut state, &receipt.finished_by, &receipt.record).is_err());
    assert_eq!(state, before);
    resolution_mut(&mut state)
        .unwrap()
        .work_trace
        .as_mut()
        .unwrap()
        .active = Some(5);
    let before = state.clone();
    assert!(retain(&mut state, &receipt.finished_by, &receipt.record).is_err());
    assert_eq!(state, before);
}
