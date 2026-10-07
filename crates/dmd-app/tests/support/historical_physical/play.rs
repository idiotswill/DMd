//! Ordinary commands continue the actual saved identities; no fixture recreates them.
use super::*;
use driver::Fixture;

struct Roles {
    channels: Vec<TableTransportChannel>,
    pcs: [TableTransportChannel; 2],
    characters: [CharacterId; 2],
    actors: [EntityId; 2],
    names: [String; 2],
    target: EntityId,
    reactor: Option<EntityId>,
}
impl Roles {
    fn from(archive: &archive::Archive) -> Self {
        let terminal = archive.cuts.last().unwrap();
        let state = decoded(&terminal.after);
        let channels = terminal
            .after_views
            .iter()
            .map(|a| a.channel.clone())
            .collect::<Vec<_>>();
        let pcs = channels
            .iter()
            .filter(|c| matches!(c, TableTransportChannel::Player { .. }))
            .cloned()
            .collect::<Vec<_>>();
        let pcs: [TableTransportChannel; 2] = pcs.try_into().unwrap();
        let characters = pcs.each_ref().map(|c| match c {
            TableTransportChannel::Player { character_id, .. } => *character_id,
            _ => unreachable!(),
        });
        let actors = characters.map(|id| state.characters[&id].entity_id);
        let names = characters.map(|id| state.characters[&id].display_name.clone());
        let order = &state.rules.as_ref().unwrap().timing.as_ref().unwrap().order;
        assert_eq!(order[0].actor, actors[0]);
        let target = order[1].actor;
        let reactor = order.get(2).map(|entry| entry.actor);
        assert_eq!(reactor.is_some(), archive.name == "ground-v4");
        Self {
            channels,
            pcs,
            characters,
            actors,
            names,
            target,
            reactor,
        }
    }
    fn source(&self) -> TableTransportChannel {
        let actor = self.reactor.unwrap();
        self.channels.iter().find(|c| matches!(c,TableTransportChannel::SourceCreature { actor: id, .. } if *id==actor)).unwrap().clone()
    }
}
fn flow(state: &CampaignState) -> &TacticalFlow {
    state.encounter.as_ref().unwrap().flow.as_ref().unwrap()
}
fn timing(state: &CampaignState) -> &CombatTiming {
    state.rules.as_ref().unwrap().timing.as_ref().unwrap()
}
async fn send(
    f: &mut Fixture,
    roles: &Roles,
    channel: TableTransportChannel,
    input: TableTransportInput,
) -> TableTransportRequest {
    let request = f.request(channel, input).await;
    Box::pin(f.accept(request.clone(), &roles.channels)).await;
    request
}
async fn act(
    f: &mut Fixture,
    roles: &Roles,
    channel: TableTransportChannel,
    action: TacticalAction,
) -> TableTransportRequest {
    Box::pin(send(f, roles, channel, tactical(action))).await
}
async fn choose(
    f: &mut Fixture,
    roles: &Roles,
    channel: TableTransportChannel,
    label: &str,
) -> TableTransportRequest {
    let options = f.view(&channel).await.grapple.unwrap().choices;
    let matching = options
        .iter()
        .filter(|o| o.label == label)
        .collect::<Vec<_>>();
    assert_eq!(matching.len(), 1, "actual offer {label}: {options:?}");
    Box::pin(send(
        f,
        roles,
        channel,
        TableTransportInput::GrappleChoice {
            handle: matching[0].key,
        },
    ))
    .await
}
async fn raw(
    f: &mut Fixture,
    roles: &Roles,
    channel: TableTransportChannel,
    face: u16,
    inspired: bool,
) -> TableTransportRequest {
    let roll = f.view(&channel).await.roll.unwrap();
    assert_eq!(roll.mode, RollMode::Normal);
    assert_eq!(
        roll.dice,
        vec![DieSpec {
            count: 1,
            sides: 20
        }]
    );
    let result = RollResult {
        request_id: roll.id,
        source: RollSource::Physical,
        dice: vec![DieResult {
            sides: 20,
            value: face,
        }],
    };
    let action = if inspired {
        TacticalAction::SubmitRollWithInspiration {
            result,
            die_index: 0,
            replacement: DieResult {
                sides: 20,
                value: 20,
            },
        }
    } else {
        TacticalAction::SubmitRoll { result }
    };
    let pending = f.state().await.rules.unwrap().pending.unwrap();
    let request = f.request(channel.clone(), tactical(action)).await;
    // Grapple saves, unlike ordinary weapon attacks, require their real owner.
    if inspired {
        for other in [
            TableTransportChannel::Host,
            roles.pcs.iter().find(|c| **c != channel).unwrap().clone(),
        ] {
            let mut bad = f.request(other, request.input.clone()).await;
            assert_ne!(bad, request);
            bad.version = request.version;
            Box::pin(f.refuse(bad)).await;
        }
    }
    Box::pin(f.accept(request.clone(), &roles.channels)).await;
    let state = f.state().await;
    let accepted = state.rules.as_ref().unwrap().rolls.last().unwrap();
    assert_eq!(accepted.request, pending.request);
    assert_eq!(accepted.issued_by, pending.issued_by);
    assert_eq!(accepted.purpose, pending.purpose);
    assert_eq!(accepted.accepted_by.id, request.command_id);
    assert_eq!(accepted.result.source, RollSource::Physical);
    assert_eq!(
        accepted.result.dice,
        vec![DieResult {
            sides: 20,
            value: if inspired { 20 } else { face }
        }]
    );
    assert_eq!(
        accepted.resolved.total,
        i32::from(if inspired { 20 } else { face }) + pending.request.modifier
    );
    if inspired {
        assert_eq!(
            accepted.original_result.as_ref().unwrap().dice,
            vec![DieResult {
                sides: 20,
                value: face
            }]
        );
        assert!(
            !state.rules.as_ref().unwrap().entities[&pending.request.roller.unwrap()]
                .heroic_inspiration
        );
    }
    request
}
async fn finish(f: &mut Fixture, roles: &Roles) {
    let state = f.state().await;
    assert!(flow(&state).resolution.is_none());
    assert!(state.rules.as_ref().unwrap().pending.is_none());
    Box::pin(act(f,roles,TableTransportChannel::Host,TacticalAction::ConcludeHostilities {
        cadence:AftermathCadence::ContinueExistingOrder,
        ruling:"Participants conclude this actual settled encounter; the existing order continues until all boundaries are settled.".into(),
    })).await;
    assert_eq!(flow(&f.state().await).phase, TacticalPhase::Active);
    Box::pin(f.premature_mass()).await;
    if let Some(reactor) = roles.reactor {
        let before = f.state().await;
        assert_eq!(
            timing(&before).order[timing(&before).index].actor,
            roles.target
        );
        assert_eq!(timing(&before).reactions_spent, vec![reactor]);
        let early = f
            .request(
                TableTransportChannel::Host,
                tactical(TacticalAction::FinishEncounter),
            )
            .await;
        let error = Box::pin(f.refuse(early)).await;
        assert!(
            error.contains("spent Reactions must reach their owners' real Start"),
            "{error}"
        );
        Box::pin(act(
            f,
            roles,
            TableTransportChannel::Host,
            TacticalAction::EndTurn,
        ))
        .await;
        let after = f.state().await;
        assert_eq!(timing(&after).turn_number, timing(&before).turn_number + 1);
        assert_eq!(timing(&after).order[timing(&after).index].actor, reactor);
        assert!(timing(&after).reactions_spent.is_empty());
    } else {
        assert!(timing(&f.state().await).reactions_spent.is_empty());
    }
    Box::pin(act(
        f,
        roles,
        TableTransportChannel::Host,
        TacticalAction::FinishEncounter,
    ))
    .await;
    assert_eq!(flow(&f.state().await).phase, TacticalPhase::Finished);
}

/// Rebind only a real current offer with the identical canonical capability.
/// This is a fresh command, never an alteration of a stored original retry.
async fn next(f: &mut Fixture, roles: &Roles, cut: &archive::Cut) {
    let mut input = cut.request.input.clone();
    let current = f.export().await;
    let offered = f.view(&cut.request.channel).await;
    let old = latest(&cut.before, audience(&cut.request.channel));
    let now = latest(&current, audience(&cut.request.channel));
    let key = |id: uuid::Uuid| {
        let capability = &old
            .handles
            .iter()
            .find(|h| h.opaque == id)
            .unwrap()
            .capability;
        now.handles
            .iter()
            .find(|h| &h.capability == capability)
            .unwrap()
            .opaque
    };
    match &mut input {
        TableTransportInput::GrappleChoice { handle }
        | TableTransportInput::InspirationTransfer { handle } => handle.0 = key(handle.0),
        TableTransportInput::MoveGrappled { option, .. } => option.0 = key(option.0),
        TableTransportInput::Action(action) => match action.as_mut() {
            TableAction::Tactical {
                action:
                    TacticalAction::SubmitRoll { result }
                    | TacticalAction::SubmitRollWithInspiration { result, .. },
            } => {
                // A fresh earlier attack has a genuinely new canonical raw ID.
                // Select the actual owned current offer; do not transplant the
                // original producer's canonical roll or work ancestry.
                let roll = offered.roll.as_ref().unwrap();
                let original = cut
                    .before_views
                    .iter()
                    .find(|v| v.channel == cut.request.channel)
                    .unwrap()
                    .presented
                    .roll
                    .as_ref()
                    .unwrap();
                assert_eq!(roll.dice, original.dice);
                assert_eq!(roll.mode, original.mode);
                assert_eq!(roll.reason, original.reason);
                result.request_id = roll.id;
            }
            _ => {}
        },
        _ => panic!("unreviewed archived continuation shape"),
    }
    Box::pin(send(f, roles, cut.request.channel.clone(), input)).await;
}
pub async fn old_pending(archive: &archive::Archive) {
    let roles = Roles::from(archive);
    let cut = &archive.cuts[3];
    let mut f = Box::pin(Fixture::restore(&cut.before)).await;
    if roles.reactor.is_some() {
        for cut in &archive.cuts[3..] {
            Box::pin(next(&mut f, &roles, cut)).await;
        }
        assert_ground_terminal(&f.state().await, &roles);
    } else {
        Box::pin(next(&mut f, &roles, cut)).await;
        Box::pin(act(
            &mut f,
            &roles,
            roles.pcs[0].clone(),
            TacticalAction::EndTurn,
        ))
        .await;
        Box::pin(save(&mut f, &roles, 0)).await;
        Box::pin(raw(&mut f, &roles, roles.pcs[0].clone(), 1, true)).await;
        Box::pin(choose(
            &mut f,
            &roles,
            TableTransportChannel::Host,
            "Finish without changing equipment",
        ))
        .await;
        Box::pin(act(
            &mut f,
            &roles,
            TableTransportChannel::Host,
            TacticalAction::EndTurn,
        ))
        .await;
    }
    Box::pin(f.retained(&archive.retained)).await;
    Box::pin(finish(&mut f, &roles)).await;
    Box::pin(f.close()).await;
}

async fn mass(f: &mut Fixture, roles: &Roles, archive: &archive::Archive) {
    let before = f.state().await;
    assert!(before.physical_facts.is_none());
    let old = f.export().await;
    let activation = Box::pin(send(
        f,
        roles,
        TableTransportChannel::Host,
        TableTransportInput::EnablePhysicalFacts,
    ))
    .await;
    assert_eq!(activation.version, 5);
    let after = f.state().await;
    assert_eq!(after.schema_version, 4);
    assert_eq!(after.physical_facts.as_ref().unwrap().schema_version, 1);
    assert_eq!(
        after.table.as_ref().unwrap().grapple_access,
        before.table.as_ref().unwrap().grapple_access
    );
    assert_eq!(
        after.rules.as_ref().unwrap().tactical_creatures,
        before.rules.as_ref().unwrap().tactical_creatures
    );
    assert_eq!(after.table, before.table);
    assert_eq!(after.rules, before.rules);
    assert_eq!(after.encounter, before.encounter);
    assert_eq!(after.encounter_history, before.encounter_history);
    assert_eq!(after.items, before.items);
    let saved = f.export().await;
    compare::prefix(&old, &saved);
    assert_eq!(
        saved.command_audit.last().unwrap().command_schema_version,
        6
    );
    assert_eq!(saved.event_journal.last().unwrap().event_schema_version, 1);
    assert_eq!(saved.table_projection_history.last().unwrap().version, 5);
    for channel in &roles.channels {
        let v = f.view(channel).await;
        assert_eq!(v.physical.as_ref().unwrap().version, 5);
        assert_eq!(v.grapple.as_ref().unwrap().version, 4);
        if let Some(source) = v.source_control {
            assert_eq!(source.version, 2);
        }
        if !matches!(channel, TableTransportChannel::Host) {
            assert!(v.physical.unwrap().controls.is_empty());
        }
    }
    Box::pin(f.retained(&archive.retained)).await;
    // Genuine older accepted requests remain retries, while a distinct fresh v4
    // command using current context cannot write a mass-enabled campaign.
    let mut stale = f
        .request(
            TableTransportChannel::Host,
            TableTransportInput::Action(Box::new(TableAction::AwardHeroicInspiration {
                character_id: roles.characters[0],
                reason: "A distinct fresh obsolete-version envelope must be refused.".into(),
            })),
        )
        .await;
    stale.version = 4;
    Box::pin(f.refuse(stale)).await;
    let host = f.view(&TableTransportChannel::Host).await;
    let fact = host
        .physical
        .unwrap()
        .controls
        .into_iter()
        .find(|c| c.kind == "body" && c.label.contains(&roles.names[0]))
        .unwrap();
    let input=TableTransportInput::PhysicalFact { handle:fact.key,input:PhysicalFactInput::Body {
        pounds:None,reason:"The historical character's unladen body has not been measured; its mass remains explicitly unknown.".into(),
    }};
    for channel in &roles.pcs {
        let bad = f.request(channel.clone(), input.clone()).await;
        Box::pin(f.refuse(bad)).await;
    }
    Box::pin(send(f, roles, TableTransportChannel::Host, input)).await;
    let measured = f.state().await;
    let entry = measured
        .physical_facts
        .as_ref()
        .unwrap()
        .get(PhysicalSubject::Body(roles.actors[0]))
        .unwrap();
    assert!(matches!(
        entry.value,
        PhysicalFactValue::Body { mass: None, .. }
    ));
    Box::pin(negatives::post_mass(f, &old)).await;
    Box::pin(f.retained(&archive.retained)).await;
}
async fn battlefield(f: &mut Fixture, roles: &Roles) {
    let state = f.state().await;
    assert_eq!(flow(&state).phase, TacticalPhase::Finished);
    let ground = roles.reactor.is_some();
    let pc_indices = if ground { vec![0] } else { vec![0, 1] };
    let source_actors = std::iter::once(roles.target)
        .chain(roles.reactor)
        .collect::<Vec<_>>();
    let point = |x, y| SpatialPoint { x, y, z: 0 };
    let setup = TableBattlefieldSetup {
        encounter_id: EncounterId::new(),
        scene_id: SceneId::new(),
        location_id: LocationId::new(),
        name: "Historical participants continue on the courtyard".into(),
        battlefield: Battlefield {
            bounds: SpatialBox {
                min: point(0, 0),
                max: SpatialPoint {
                    x: 100,
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
        characters: pc_indices
            .iter()
            .map(|&i| TableCharacterPlacement {
                character_id: roles.characters[i],
                position: point(if i == 0 { 10 } else { 30 }, 10),
                height: 12,
                allies: vec![],
                enemies: source_actors.clone(),
            })
            .collect(),
        creatures: source_actors
            .iter()
            .map(|&actor| TableCreaturePlacement {
                actor,
                public_label: if actor == roles.target {
                    "Small armored figure"
                } else {
                    "Independent reactor"
                }
                .into(),
                position: if actor == roles.target {
                    point(20, 10)
                } else {
                    point(10, 20)
                },
                height: 8,
                allies: vec![],
                enemies: pc_indices.iter().map(|&i| roles.actors[i]).collect(),
            })
            .collect(),
        geometry_ruling: Ruling {
            basis: RulingBasis::GmAdjudication,
            reason:
                "Host establishes these visible level positions for the same retained participants."
                    .into(),
        },
        area_grid_policy: None,
    };
    let mut groups = pc_indices
        .iter()
        .map(|&i| InitiativeGroup {
            actors: vec![roles.actors[i]],
            request_id: RollRequestId::new(),
        })
        .collect::<Vec<_>>();
    groups.push(InitiativeGroup {
        actors: source_actors.clone(),
        request_id: RollRequestId::new(),
    });
    let creatures = state
        .rules
        .as_ref()
        .unwrap()
        .tactical_creatures
        .as_ref()
        .unwrap();
    let combatants = pc_indices
        .iter()
        .map(|&i| TacticalCombatant {
            actor: roles.actors[i],
            source: TacticalSource::Character,
            surprised: false,
        })
        .chain(source_actors.iter().map(|&actor| {
            TacticalCombatant {
                actor,
                source: TacticalSource::Creature {
                    definition_id: creatures
                        .profile(actor)
                        .unwrap()
                        .source
                        .definition_id
                        .clone(),
                },
                surprised: false,
            }
        }))
        .collect();
    Box::pin(send(
        f,
        roles,
        TableTransportChannel::Host,
        TableTransportInput::Action(Box::new(TableAction::PrepareBattlefield {
            setup: Box::new(setup),
        })),
    ))
    .await;
    Box::pin(act(
        f,
        roles,
        TableTransportChannel::Host,
        TacticalAction::Begin {
            execution: TacticalExecutionVersion::EncounterReleaseV1,
            combatants,
            groups: groups.clone(),
        },
    ))
    .await;
    assert_eq!(flow(&f.state().await).initiative_groups, groups);
    for &i in &pc_indices {
        Box::pin(raw(
            f,
            roles,
            roles.pcs[i].clone(),
            if i == 0 { 18 } else { 10 },
            false,
        ))
        .await;
    }
    Box::pin(raw(f, roles, TableTransportChannel::Host, 2, false)).await;
    if ground {
        Box::pin(act(
            f,
            roles,
            TableTransportChannel::Host,
            TacticalAction::ProposeInitiativeTie {
                order: source_actors,
            },
        ))
        .await;
    }
    let ready = f.state().await;
    assert_eq!(
        ready.table.as_ref().unwrap().character_profiles,
        state.table.as_ref().unwrap().character_profiles
    );
    assert_eq!(
        ready.table.as_ref().unwrap().source_actor_access,
        state.table.as_ref().unwrap().source_actor_access
    );
    let ready_sources = ready
        .rules
        .as_ref()
        .unwrap()
        .tactical_creatures
        .as_ref()
        .unwrap();
    assert_eq!(ready_sources.profiles, creatures.profiles);
    for actor in std::iter::once(roles.target).chain(roles.reactor) {
        let before = creatures.runtime(actor).unwrap();
        let after = ready_sources.runtime(actor).unwrap();
        assert_eq!(after.controller, before.controller);
        assert_eq!(after.control_origin, before.control_origin);
    }
    let encounter = ready.encounter.as_ref().unwrap();
    for &i in &pc_indices {
        let pc = encounter.participant(roles.actors[i]).unwrap();
        let target = encounter.participant(roles.target).unwrap();
        assert_eq!(
            dmd_rules::spatial::participant_distance(pc, target).unwrap(),
            10
        );
        assert!(pc.enemies.contains(&roles.target));
        assert!(target.enemies.contains(&roles.actors[i]));
    }
    assert_eq!(flow(&ready).phase, TacticalPhase::Active);
    assert_eq!(
        timing(&ready).order[timing(&ready).index].actor,
        roles.actors[0]
    );
    assert_eq!(
        timing(&ready)
            .order
            .iter()
            .map(|e| e.actor)
            .collect::<Vec<_>>(),
        if ground {
            vec![roles.actors[0], roles.target, roles.reactor.unwrap()]
        } else {
            vec![roles.actors[0], roles.actors[1], roles.target]
        }
    );
    assert!(flow(&ready).attack_equipment_access.is_none());
    Box::pin(act(
        f,
        roles,
        TableTransportChannel::Host,
        TacticalAction::ActivateAttackEquipment,
    ))
    .await;
    assert!(flow(&f.state().await).attack_equipment_access.is_some());
}
fn assert_ground_terminal(state: &CampaignState, roles: &Roles) {
    let result = flow(state).last_movement.as_ref().unwrap();
    assert_eq!(result.reason, TacticalMovementEnd::Stopped);
    assert_eq!((result.completed_steps, result.spent_after), (1, 20));
    assert_eq!(result.endpoint, SpatialPoint { x: 0, y: 10, z: 0 });
    assert_eq!(
        result.transport.as_ref().unwrap().target_endpoint,
        SpatialPoint { x: 10, y: 10, z: 0 }
    );
    assert!(
        timing(state)
            .reactions_spent
            .contains(&roles.reactor.unwrap())
    );
    assert!(flow(state).resolution.is_none());
    assert!(state.rules.as_ref().unwrap().pending.is_none());
}
async fn ground(f: &mut Fixture, roles: &Roles) {
    Box::pin(choose(
        f,
        roles,
        roles.pcs[0].clone(),
        "Grapple Small armored figure with left hand",
    ))
    .await;
    Box::pin(choose(
        f,
        roles,
        TableTransportChannel::Host,
        "Resist Grapple with Strength",
    ))
    .await;
    Box::pin(raw(f, roles, TableTransportChannel::Host, 1, false)).await;
    assert_eq!(
        f.state()
            .await
            .rules
            .unwrap()
            .tactical_grapples
            .unwrap()
            .active
            .len(),
        1
    );
    Box::pin(choose(
        f,
        roles,
        roles.pcs[0].clone(),
        "Finish without changing equipment",
    ))
    .await;
    let option = f
        .view(&roles.pcs[0])
        .await
        .grapple
        .unwrap()
        .ground_drag
        .into_iter()
        .find(|o| o.actor == roles.actors[0])
        .unwrap();
    Box::pin(send(
        f,
        roles,
        roles.pcs[0].clone(),
        TableTransportInput::MoveGrappled {
            option: option.key,
            path: vec![
                TacticalMoveStep {
                    destination: SpatialPoint { x: 0, y: 10, z: 0 },
                    mode: MovementMode::Walk,
                },
                TacticalMoveStep {
                    destination: SpatialPoint { x: 0, y: 0, z: 0 },
                    mode: MovementMode::Walk,
                },
            ],
        },
    ))
    .await;
    let selected = f.state().await;
    let prefix = flow(&selected)
        .resolution
        .as_ref()
        .unwrap()
        .grapple
        .as_ref()
        .unwrap()
        .transport
        .as_ref()
        .unwrap()
        .steps
        .clone();
    assert_eq!(prefix.len(), 1);
    assert_eq!(flow(&selected).budget.movement_spent, 20);
    assert_eq!(
        flow(&selected)
            .resolution
            .as_ref()
            .unwrap()
            .movement
            .as_ref()
            .unwrap()
            .next_step,
        1
    );
    Box::pin(act(
        f,
        roles,
        roles.source(),
        TacticalAction::OpportunityAttack {
            choice: TacticalMeleeChoice::UnarmedDamage {
                ability: Ability::Strength,
            },
        },
    ))
    .await;
    let issued = f.state().await;
    let pending = issued.rules.as_ref().unwrap().pending.clone().unwrap();
    assert_eq!(pending.request.roller, roles.reactor);
    let release = f
        .view(&roles.pcs[0])
        .await
        .grapple
        .unwrap()
        .choices
        .into_iter()
        .find(|o| o.label.starts_with("Release "))
        .unwrap();
    Box::pin(send(
        f,
        roles,
        roles.pcs[0].clone(),
        TableTransportInput::GrappleChoice {
            handle: release.key,
        },
    ))
    .await;
    let released = f.state().await;
    assert_eq!(
        released.rules.as_ref().unwrap().pending.as_ref(),
        Some(&pending)
    );
    let resolution = flow(&released).resolution.as_ref().unwrap();
    assert_eq!(
        resolution.attack,
        flow(&issued).resolution.as_ref().unwrap().attack
    );
    assert_eq!(
        resolution.frames,
        flow(&issued).resolution.as_ref().unwrap().frames
    );
    assert_eq!(
        resolution
            .grapple
            .as_ref()
            .unwrap()
            .transport
            .as_ref()
            .unwrap()
            .steps,
        prefix
    );
    assert_eq!(timing(&released), timing(&issued));
    assert_eq!(released.items, issued.items);
    let view = f.view(&roles.source()).await;
    let result = RollResult {
        request_id: view.roll.unwrap().id,
        source: RollSource::Physical,
        dice: vec![DieResult {
            sides: 20,
            value: 1,
        }],
    };
    let bad = f
        .request(
            roles.pcs[1].clone(),
            tactical(TacticalAction::SubmitRoll { result }),
        )
        .await;
    Box::pin(f.refuse(bad)).await;
    Box::pin(raw(f, roles, roles.source(), 1, false)).await;
    let done = f.state().await;
    assert_ground_terminal(&done, roles);
    assert_eq!(
        done.rules.as_ref().unwrap().rolls.len(),
        issued.rules.as_ref().unwrap().rolls.len() + 1
    );
    assert_eq!(timing(&done), timing(&issued));
    assert_eq!(done.items, issued.items);
}
async fn save(f: &mut Fixture, roles: &Roles, index: usize) {
    assert_eq!(
        timing(&f.state().await).order[timing(&f.state().await).index].actor,
        roles.target
    );
    Box::pin(choose(
        f,
        roles,
        TableTransportChannel::Host,
        &format!("Grapple {} with right hand", roles.names[index]),
    ))
    .await;
    Box::pin(choose(
        f,
        roles,
        roles.pcs[index].clone(),
        "Resist Grapple with Strength",
    ))
    .await;
    let pending = f.state().await.rules.unwrap().pending.unwrap();
    assert_eq!(pending.request.roller, Some(roles.actors[index]));
    assert!(
        matches!(pending.purpose,PendingPurpose::TacticalResolution {key,..} if key.role==TacticalRollRole::GrappleSave)
    );
}
async fn award(f: &mut Fixture, roles: &Roles, recipient: bool) {
    let pc1 = f.views(&[roles.pcs[1].clone()]).await;
    assert!(!f.state().await.rules.unwrap().entities[&roles.actors[0]].heroic_inspiration);
    for action in [
        TableAction::AwardHeroicInspiration {
            character_id: roles.characters[0],
            reason: "For the participant's careful cooperation in continuing the saved encounter."
                .into(),
        },
        TableAction::AwardExcessInspiration {
            character_id: roles.characters[0],
            reason: "For returning to help a companion during this new play.".into(),
        },
    ] {
        Box::pin(send(
            f,
            roles,
            TableTransportChannel::Host,
            TableTransportInput::Action(Box::new(action)),
        ))
        .await;
        let now = f.views(&[roles.pcs[1].clone()]).await;
        assert_eq!(
            now[0].presented, pc1[0].presented,
            "private award must not offer a revision/handle oracle"
        );
        let mut raw = now[0].raw.clone();
        raw.event_sequence = pc1[0].raw.event_sequence;
        assert_eq!(raw, pc1[0].raw);
    }
    let transfer = f.view(&roles.pcs[0]).await.inspiration_transfer.unwrap();
    let label = if recipient {
        format!("Give to {}", roles.names[1])
    } else {
        "Decline the extra Inspiration".into()
    };
    let choice = transfer
        .choices
        .iter()
        .find(|c| c.label == label)
        .unwrap_or_else(|| panic!("missing {label}: {transfer:?}"));
    let input = TableTransportInput::InspirationTransfer { handle: choice.key };
    for channel in [TableTransportChannel::Host, roles.pcs[1].clone()] {
        let bad = f.request(channel, input.clone()).await;
        Box::pin(f.refuse(bad)).await;
    }
    Box::pin(send(f, roles, roles.pcs[0].clone(), input)).await;
    assert_eq!(
        f.state().await.rules.unwrap().entities[&roles.actors[1]].heroic_inspiration,
        recipient
    );
    if !recipient {
        assert_eq!(f.view(&roles.pcs[1]).await, pc1[0].presented);
    }
}
async fn to_goblin(f: &mut Fixture, roles: &Roles) {
    for i in 0..2 {
        assert_eq!(
            timing(&f.state().await).order[timing(&f.state().await).index].actor,
            roles.actors[i]
        );
        Box::pin(act(f, roles, roles.pcs[i].clone(), TacticalAction::EndTurn)).await;
    }
    assert_eq!(
        timing(&f.state().await).order[timing(&f.state().await).index].actor,
        roles.target
    );
}
async fn recipient_still_ineligible(f: &Fixture, roles: &Roles) {
    let original = f.export().await;
    let cells = driver::rows(&f.pool).await;
    let mut probe = Box::pin(Fixture::restore(&original)).await;
    assert!(probe.state().await.rules.unwrap().entities[&roles.actors[1]].heroic_inspiration);
    for action in [
        TableAction::AwardHeroicInspiration {character_id:roles.characters[0],reason:"Independent genuine eligibility branch with the historical recipient gift still held.".into()},
        TableAction::AwardExcessInspiration {character_id:roles.characters[0],reason:"Check the actual owner offer before the old recipient gift has been spent.".into()},
    ] {Box::pin(send(&mut probe,roles,TableTransportChannel::Host,TableTransportInput::Action(Box::new(action)))).await;}
    let choices = probe
        .view(&roles.pcs[0])
        .await
        .inspiration_transfer
        .unwrap()
        .choices;
    assert_eq!(choices.len(), 1);
    assert_eq!(choices[0].label, "Decline the extra Inspiration");
    Box::pin(send(
        &mut probe,
        roles,
        roles.pcs[0].clone(),
        TableTransportInput::InspirationTransfer {
            handle: choices[0].key,
        },
    ))
    .await;
    assert!(probe.state().await.rules.unwrap().entities[&roles.actors[1]].heroic_inspiration);
    Box::pin(probe.close()).await;
    compare::exact_export(&original, &f.export().await);
    assert_eq!(driver::rows(&f.pool).await, cells);
}
pub async fn new_version(archive: &archive::Archive) {
    let roles = Roles::from(archive);
    let terminal = &archive.cuts.last().unwrap().after;
    let mut f = Box::pin(Fixture::restore(terminal)).await;
    Box::pin(finish(&mut f, &roles)).await;
    Box::pin(mass(&mut f, &roles, archive)).await;
    if roles.reactor.is_some() {
        Box::pin(battlefield(&mut f, &roles)).await;
        Box::pin(ground(&mut f, &roles)).await;
    } else {
        let recipient = archive.name == "inspiration-recipient-v4";
        assert_eq!(
            f.state().await.rules.unwrap().entities[&roles.actors[1]].heroic_inspiration,
            recipient
        );
        if recipient {
            Box::pin(recipient_still_ineligible(&f, &roles)).await;
        }
        if !recipient {
            Box::pin(award(&mut f, &roles, false)).await;
        }
        Box::pin(battlefield(&mut f, &roles)).await;
        Box::pin(to_goblin(&mut f, &roles)).await;
        if recipient {
            // The old gift is a real resource, not something a new fixture clears.
            assert!(f.state().await.rules.unwrap().entities[&roles.actors[1]].heroic_inspiration);
            Box::pin(save(&mut f, &roles, 1)).await;
            Box::pin(raw(&mut f, &roles, roles.pcs[1].clone(), 1, true)).await;
            Box::pin(choose(
                &mut f,
                &roles,
                TableTransportChannel::Host,
                "Finish without changing equipment",
            ))
            .await;
            Box::pin(award(&mut f, &roles, true)).await;
            Box::pin(act(
                &mut f,
                &roles,
                TableTransportChannel::Host,
                TacticalAction::EndTurn,
            ))
            .await;
            Box::pin(to_goblin(&mut f, &roles)).await;
        }
        Box::pin(save(&mut f, &roles, 0)).await;
        Box::pin(raw(&mut f, &roles, roles.pcs[0].clone(), 1, true)).await;
        Box::pin(choose(
            &mut f,
            &roles,
            TableTransportChannel::Host,
            "Finish without changing equipment",
        ))
        .await;
        let state = f.state().await;
        assert!(!state.rules.as_ref().unwrap().entities[&roles.actors[0]].heroic_inspiration);
        assert_eq!(
            state.rules.as_ref().unwrap().entities[&roles.actors[1]].heroic_inspiration,
            recipient
        );
        assert!(flow(&state).resolution.is_none());
    }
    compare::prefix(terminal, &f.export().await);
    Box::pin(f.retained(&archive.retained)).await;
    Box::pin(f.close()).await;
}
