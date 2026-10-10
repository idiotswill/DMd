//! Private ownership mechanism controls from actual shared producers after the
//! existing authored source baseline. Public cold replay has its separate test.
use super::*;

fn table(owner: &mut GuardedGrappleExecution<'_>, operation: crate::table::TableOperation) {
    let meta = command(owner.state(), None);
    let mut candidate = Box::new(owner.state().clone());
    let mut execution = ExecutionContext {
        mass: None,
        guarded: Some(GuardedCommand {
            predecessor: owner.state(),
            candidate: std::ptr::from_ref(candidate.as_ref()),
            command: &meta,
            produced: ProducedEvidence::default(),
        }),
    };
    crate::table::reducer::apply_operation(
        owner.state(),
        &mut candidate,
        &meta,
        &operation,
        owner.pack,
        None,
        &mut execution,
    )
    .unwrap();
    execution.validate_delta(&candidate).unwrap();
    drop(execution);
    candidate.applied_event_sequence += 1;
    owner.state = *candidate;
}

fn tactical(owner: &mut GuardedGrappleExecution<'_>, actor: EntityId, action: TacticalAction) {
    let meta = command(owner.state(), Some(actor));
    let mut candidate = Box::new(owner.state().clone());
    let mut execution = ExecutionContext {
        mass: None,
        guarded: Some(GuardedCommand {
            predecessor: owner.state(),
            candidate: std::ptr::from_ref(candidate.as_ref()),
            command: &meta,
            produced: ProducedEvidence::default(),
        }),
    };
    crate::tactical::apply_table_with_context(
        owner.state(),
        &mut candidate,
        &meta,
        &action,
        owner.pack,
        false,
        &mut execution,
    )
    .unwrap();
    execution.validate_delta(&candidate).unwrap();
    drop(execution);
    candidate.applied_event_sequence += 1;
    owner.state = *candidate;
}

fn choice() -> TacticalMeleeChoice {
    TacticalMeleeChoice::UnarmedDamage {
        ability: Ability::Strength,
    }
}

struct Fixture<'a> {
    owner: GuardedGrappleExecution<'a>,
    selected: CampaignState,
    holder: EntityId,
    reactor: EntityId,
    grip: GrappleId,
}
fn fixture(pack: &RulesPack) -> Fixture<'_> {
    let (state, holder, target, reactor) = with_source(pack, "mage");
    let mut owner = GuardedGrappleExecution::new(state, pack).unwrap();
    table(
        &mut owner,
        crate::table::TableOperation::EnableGrappleAccess,
    );
    let grip = grip(&mut owner, holder, target);
    table(
        &mut owner,
        crate::table::TableOperation::EnableGrappleTransport,
    );
    tactical(
        &mut owner,
        holder,
        TacticalAction::MoveGrappled {
            grip,
            path: vec![TacticalMoveStep {
                destination: SpatialPoint { x: 0, y: 0, z: 0 },
                mode: MovementMode::Walk,
            }],
        },
    );
    let selected = owner.state().clone();
    assert_eq!(
        resolution(&selected)
            .unwrap()
            .movement
            .as_ref()
            .unwrap()
            .opportunity
            .as_ref()
            .unwrap()
            .reactor,
        reactor
    );
    tactical(
        &mut owner,
        reactor,
        TacticalAction::OpportunityAttack { choice: choice() },
    );
    Fixture {
        owner,
        selected,
        holder,
        reactor,
        grip,
    }
}

#[test]
fn issued_ground_opportunity_requires_exact_predecessor_or_observed_producer() {
    let pack = crate::tactical_hands::tests::pack();
    let mut f = fixture(&pack);
    let before = f.owner.state().clone();
    let meta = command(&before, Some(f.holder));
    let candidate = Box::new(before.clone());
    let owned = ExecutionContext {
        mass: None,
        guarded: Some(GuardedCommand {
            predecessor: &before,
            candidate: std::ptr::from_ref(candidate.as_ref()),
            command: &meta,
            produced: ProducedEvidence::default(),
        }),
    };
    let attack = resolution(&candidate).unwrap().attack.as_ref().unwrap();
    assert!(
        owned
            .read(&candidate)
            .unwrap()
            .accepted_ground_opportunity(attack)
            .unwrap()
    );
    assert!(owned.validate_delta(&candidate).is_ok());
    let foreign = candidate.as_ref().clone();
    assert!(owned.read(&foreign).is_err());
    assert!(
        ReadContext::ordinary(&candidate)
            .accepted_ground_opportunity(attack)
            .is_err()
    );
    let unobserved = ExecutionContext {
        mass: None,
        guarded: Some(GuardedCommand {
            predecessor: &f.selected,
            candidate: std::ptr::from_ref(candidate.as_ref()),
            command: &attack.origin,
            produced: ProducedEvidence::default(),
        }),
    };
    assert!(
        unobserved
            .read(&candidate)
            .unwrap()
            .accepted_ground_opportunity(attack)
            .is_err()
    );
    assert!(unobserved.validate_delta(&candidate).is_err());

    let issued_request = pending(&before).clone();
    tactical(
        &mut f.owner,
        f.holder,
        TacticalAction::ReleaseGrapple { grip: f.grip },
    );
    assert_eq!(pending(f.owner.state()), &issued_request);
    tactical(
        &mut f.owner,
        f.reactor,
        TacticalAction::SubmitRoll {
            result: faces(&issued_request, 1),
        },
    );
    assert!(flow(f.owner.state()).unwrap().resolution.is_none());
    assert_eq!(
        flow(f.owner.state())
            .unwrap()
            .last_movement
            .as_ref()
            .unwrap()
            .reason,
        TacticalMovementEnd::Stopped
    );
}

#[test]
fn changed_crossing_source_response_and_work_cannot_borrow_the_predecessor_certificate() {
    let pack = crate::tactical_hands::tests::pack();
    let f = fixture(&pack);
    let before = f.owner.state().clone();
    for case in 0..12 {
        let mut candidate = Box::new(before.clone());
        let meta = command(&before, Some(f.holder));
        let owned = ExecutionContext {
            mass: None,
            guarded: Some(GuardedCommand {
                predecessor: &before,
                candidate: std::ptr::from_ref(candidate.as_ref()),
                command: &meta,
                produced: ProducedEvidence::default(),
            }),
        };
        let r = resolution_mut(&mut candidate).unwrap();
        match case {
            0 => {
                if let TacticalAttackAdmission::Opportunity(w) =
                    &mut r.attack.as_mut().unwrap().admission
                {
                    w.origin.id = CommandId::new();
                }
            }
            1 => {
                if let TacticalAttackAdmission::Opportunity(w) =
                    &mut r.attack.as_mut().unwrap().admission
                {
                    w.from.x += 1;
                }
            }
            2 => {
                if let TacticalAttackAdmission::Opportunity(w) =
                    &mut r.attack.as_mut().unwrap().admission
                {
                    w.options[0].reach += 1;
                }
            }
            3 => {
                r.attack.as_mut().unwrap().source = TacticalAttackSource::Unarmed {
                    ability: Ability::Dexterity,
                }
            }
            4 => r.movement.as_mut().unwrap().decisions[0].origin.id = CommandId::new(),
            5 => r.movement.as_mut().unwrap().next_step += 1,
            6 => {
                r.grapple
                    .as_mut()
                    .unwrap()
                    .transport
                    .as_mut()
                    .unwrap()
                    .admission
                    .target_from
                    .x += 1
            }
            7 => r.grapple.as_mut().unwrap().cuts[0].issued_by.id = CommandId::new(),
            8 => {
                let root = reads::attack_root(&candidate).unwrap().work.occurrence;
                node_mut(&mut candidate, root).parent = None;
            }
            9 => r.attack.as_mut().unwrap().damage[0].modifier += 1,
            10 => {
                let response = r.movement.as_ref().unwrap().decisions[0].clone();
                r.movement.as_mut().unwrap().decisions.push(response);
            }
            _ => r.grapple.as_mut().unwrap().proofs[0].declaration.origin.id = CommandId::new(),
        }
        let attack = resolution(&candidate).unwrap().attack.as_ref().unwrap();
        assert!(
            owned
                .read(&candidate)
                .unwrap()
                .accepted_ground_opportunity(attack)
                .is_err(),
            "reader accepted mutation {case}"
        );
        assert!(
            owned.validate_delta(&candidate).is_err(),
            "delta accepted mutation {case}"
        );
    }
    assert_eq!(f.owner.state(), &before);
}

fn node_mut(state: &mut CampaignState, occurrence: u16) -> &mut TacticalWorkNode {
    resolution_mut(state)
        .unwrap()
        .work_trace
        .as_mut()
        .unwrap()
        .nodes
        .iter_mut()
        .find(|n| n.work.occurrence == occurrence)
        .unwrap()
}

#[test]
fn removing_an_attack_or_its_entire_resolution_requires_one_actual_completion() {
    let pack = crate::tactical_hands::tests::pack();
    let f = fixture(&pack);
    let before = f.owner.state().clone();
    for entire in [false, true] {
        let mut candidate = Box::new(before.clone());
        let meta = command(&before, Some(f.holder));
        let owned = ExecutionContext {
            mass: None,
            guarded: Some(GuardedCommand {
                predecessor: &before,
                candidate: std::ptr::from_ref(candidate.as_ref()),
                command: &meta,
                produced: ProducedEvidence::default(),
            }),
        };
        if entire {
            flow_mut(&mut candidate).unwrap().resolution = None;
        } else {
            resolution_mut(&mut candidate).unwrap().attack = None;
        }
        assert!(owned.validate_delta(&candidate).is_err());
    }
    let mut candidate = Box::new(before.clone());
    let meta = command(&before, Some(f.reactor));
    let mut owned = ExecutionContext {
        mass: None,
        guarded: Some(GuardedCommand {
            predecessor: &before,
            candidate: std::ptr::from_ref(candidate.as_ref()),
            command: &meta,
            produced: ProducedEvidence::default(),
        }),
    };
    // Actual physical miss completes the child and its movement parent first.
    crate::tactical::dispatch(
        &before,
        &mut candidate,
        &meta,
        &TacticalAction::SubmitRoll {
            result: faces(pending(&before), 1),
        },
        &pack,
        &mut owned,
    )
    .unwrap();
    assert!(flow(&candidate).unwrap().resolution.is_none());
    owned.validate_delta(&candidate).unwrap();
    // Deliberately private duplicate-observation negative, never an adopted state.
    let completed = owned
        .guarded
        .as_ref()
        .unwrap()
        .produced
        .completed_opportunities[0]
        .clone();
    owned
        .guarded
        .as_mut()
        .unwrap()
        .produced
        .completed_opportunities
        .push(completed);
    assert!(owned.validate_delta(&candidate).is_err());
}

#[test]
fn live_token_is_candidate_and_choice_bound_before_observation() {
    let pack = crate::tactical_hands::tests::pack();
    let f = fixture(&pack);
    let original = &f.selected;
    let meta = resolution(f.owner.state())
        .unwrap()
        .attack
        .as_ref()
        .unwrap()
        .origin
        .clone();
    let mut candidate = Box::new(original.clone());
    let mut owned = ExecutionContext {
        mass: None,
        guarded: Some(GuardedCommand {
            predecessor: original,
            candidate: std::ptr::from_ref(candidate.as_ref()),
            command: &meta,
            produced: ProducedEvidence::default(),
        }),
    };
    let token = owned
        .admit_opportunity(&candidate, &meta, f.reactor, f.holder, &choice())
        .unwrap();
    let foreign = Box::new(original.clone());
    let mut foreign_context = ExecutionContext {
        mass: None,
        guarded: Some(GuardedCommand {
            predecessor: original,
            candidate: std::ptr::from_ref(foreign.as_ref()),
            command: &meta,
            produced: ProducedEvidence::default(),
        }),
    };
    assert!(
        foreign_context
            .observe_opportunity(&foreign, token)
            .is_err()
    );
    let token = owned
        .admit_opportunity(&candidate, &meta, f.reactor, f.holder, &choice())
        .unwrap();
    crate::tactical::dispatch(
        original,
        &mut candidate,
        &meta,
        &TacticalAction::OpportunityAttack { choice: choice() },
        &pack,
        &mut owned,
    )
    .unwrap();
    let attack = resolution(&candidate).unwrap().attack.as_ref().unwrap();
    assert!(
        owned
            .read(&candidate)
            .unwrap()
            .accepted_ground_opportunity(attack)
            .unwrap()
    );
    // Remove the genuine observation only in this hostile candidate control.
    owned
        .guarded
        .as_mut()
        .unwrap()
        .produced
        .opportunities
        .clear();
    resolution_mut(&mut candidate)
        .unwrap()
        .attack
        .as_mut()
        .unwrap()
        .source = TacticalAttackSource::Unarmed {
        ability: Ability::Dexterity,
    };
    assert!(owned.observe_opportunity(&candidate, token).is_err());
    assert!(owned.validate_delta(&candidate).is_err());
}
