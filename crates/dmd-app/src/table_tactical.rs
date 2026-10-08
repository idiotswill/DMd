//! Session-bound encounter setup and viewer-specific presentation.
use dmd_domain::*;
use dmd_rules::RulesPack;
use dmd_rules::tactical::*;

#[path = "table_areas.rs"]
mod areas;
#[path = "table_attacks.rs"]
mod attacks;
#[path = "table_casting.rs"]
mod casting;
#[path = "table_tactical_choices.rs"]
mod choices;
#[path = "table_hit_reactions.rs"]
mod hit_reactions;
#[path = "table_missile_reactions.rs"]
mod missile_reactions;
#[path = "table_movement.rs"]
mod movement;
#[path = "table_shields.rs"]
mod shields;

#[path = "table_tactical_read.rs"]
mod read;
pub(crate) use read::TacticalRead;
#[path = "table_grapple.rs"]
mod grapple;
pub(crate) use grapple::view as grapple_view;

pub(crate) fn view_read(
    read: TacticalRead<'_>,
    viewer: &crate::TableViewer,
    source_access: bool,
    pack: &RulesPack,
) -> Result<Option<crate::TableTacticalView>, String> {
    let state = read.state();
    let Some(encounter) = &state.encounter else {
        return Ok(None);
    };
    let host = matches!(viewer, crate::TableViewer::Host);
    let mut own = state.characters.values().filter(|character| {
        matches!(viewer, crate::TableViewer::Player(player) if character.controlling_player_id == Some(*player))
            && matches!(character.status, CharacterStatus::Active | CharacterStatus::Dead)
            && encounter.participant(character.entity_id).is_some()
    }).map(|character| character.entity_id).collect::<std::collections::HashSet<_>>();
    if source_access && let crate::TableViewer::Player(player) = viewer {
        own.extend(
            encounter
                .participants
                .iter()
                .map(|participant| participant.entity_id)
                .filter(|actor| crate::table_source_control::owns_source(state, *player, *actor)),
        );
    }
    let mut observers = own
        .iter()
        .map(|actor| {
            dmd_rules::spatial::project_actor_view(encounter, state, *actor)
                .map_err(|error| error.to_string())
        })
        .collect::<Result<Vec<_>, _>>()?;
    observers.sort_by_key(|view| view.observer.0);
    let known = observers
        .iter()
        .flat_map(|view| view.contacts.iter())
        .filter(|contact| contact.status != dmd_rules::spatial::ContactStatus::Remembered)
        .map(|contact| contact.entity_id)
        .chain(own.iter().copied())
        .collect::<std::collections::HashSet<_>>();
    let timing = state.rules.as_ref().and_then(|rules| rules.timing.as_ref());
    let active = timing
        .and_then(|timing| timing.order.get(timing.index))
        .map(|entry| entry.actor);
    let initiative = timing
        .map(|timing| {
            timing
                .order
                .iter()
                .filter(|entry| host || known.contains(&entry.actor))
                .map(|entry| {
                    let label = if host || own.contains(&entry.actor) {
                        encounter
                            .participant(entry.actor)
                            .map(|p| p.public_label.clone())
                            .unwrap_or_else(|| "Combatant".into())
                    } else {
                        observers
                            .iter()
                            .flat_map(|view| &view.contacts)
                            .find(|contact| contact.entity_id == entry.actor)
                            .and_then(|contact| contact.label.clone())
                            .unwrap_or_else(|| "Unseen creature".into())
                    };
                    crate::TableInitiativeView {
                        actor: entry.actor,
                        label,
                        total: (host || own.contains(&entry.actor)).then_some(entry.total),
                    }
                })
                .collect()
        })
        .unwrap_or_default();
    let flow = encounter.flow.as_ref();
    let phase = match flow.map(|flow| &flow.phase) {
        None => "setup",
        Some(TacticalPhase::Initiative { .. }) => "initiative",
        Some(TacticalPhase::InitiativeTies { .. }) => "ties",
        Some(TacticalPhase::Active) => "active",
        Some(TacticalPhase::Finished) => "finished",
    }
    .to_owned();
    let ties = if let Some(TacticalPhase::InitiativeTies { ties }) = flow.map(|flow| &flow.phase) {
        ties.iter()
            .filter(|tie| {
                host || tie.actors.iter().any(|actor| own.contains(actor))
                    && tie.actors.iter().all(|actor| {
                        state
                            .characters
                            .values()
                            .any(|character| character.entity_id == *actor)
                    })
            })
            .cloned()
            .collect()
    } else {
        vec![]
    };
    Ok(Some(crate::TableTacticalView {
        equipment_enabled: (host && attack_equipment_enabled(state)).then_some(true),
        attack_equipment: read
            .attack_equipment_options(pack)
            .map_err(|e| e.to_string())?
            .filter(|offer| host || own.contains(&offer.actor))
            .map(|offer| crate::TableAttackEquipmentView {
                key: offer.work,
                actor: offer.actor,
                operations: offer
                    .operations
                    .into_iter()
                    .map(|operation| {
                        let (item, verb, hand) = match operation {
                            AttackEquipmentOperation::Equip { item, hand } => {
                                (item, "Ready", Some(hand))
                            }
                            AttackEquipmentOperation::Pickup { item, hand } => {
                                (item, "Pick up", Some(hand))
                            }
                            AttackEquipmentOperation::Unequip { item } => (item, "Put away", None),
                        };
                        let name = state
                            .items
                            .get(&item)
                            .and_then(|i| {
                                dmd_rules::tactical_inventory::equipment_definition(
                                    &i.definition_id,
                                )
                                .ok()
                            })
                            .map(|d| d.display_name.clone())
                            .unwrap_or_else(|| "Weapon".into());
                        let label = match hand {
                            Some(Hand::Left) => format!("{verb} {name} · left hand"),
                            Some(Hand::Right) => format!("{verb} {name} · right hand"),
                            None => format!("{verb} {name}"),
                        };
                        crate::TableEquipmentOperation { operation, label }
                    })
                    .collect(),
                may_decline: true,
            }),
        shove: flow.and_then(|f| f.resolution.as_deref()).and_then(|r| {
            let shove = r.shove.as_deref()?;
            let selected = shove.selected.as_ref()?;
            let actor = if shove.stage == TacticalShoveStage::SaveChoice {
                shove.target
            } else {
                shove.actor
            };
            let player_controlled = state
                .characters
                .values()
                .any(|c| c.entity_id == actor && c.controlling_player_id.is_some())
                || state
                    .rules
                    .as_ref()
                    .and_then(|r| r.tactical_creatures.as_ref())
                    .and_then(|c| c.runtime(actor))
                    .is_some_and(|r| matches!(r.controller, CreatureController::Player(_)));
            let visible = if shove.stage == TacticalShoveStage::PushReview {
                host
            } else if host {
                !player_controlled
            } else {
                own.contains(&actor)
            };
            visible.then_some(crate::TableShoveView {
                key: TacticalWorkKey {
                    resolution: r.origin.id,
                    occurrence: selected.occurrence,
                },
                actor,
                stage: shove.stage,
                from: (shove.stage != TacticalShoveStage::SaveChoice).then_some(shove.target_from),
                destination: shove.push.as_ref().map(|p| p.destination),
            })
        }),
        encounter_id: encounter.id,
        aftermath: flow
            .and_then(|flow| flow.aftermath.as_ref())
            .map(|aftermath| crate::TableAftermathView {
                cadence: aftermath.cadence,
                host_ruling: host.then(|| aftermath.ruling.clone()),
                may_pause_session: host && require_aftermath_session_boundary(state).is_ok(),
            }),
        release: if host
            && flow.is_some_and(|flow| {
                flow.version == TacticalExecutionVersion::EncounterReleaseV1.flow_version()
            }) {
            if flow.is_some_and(|flow| flow.phase == TacticalPhase::Finished) {
                Some(crate::TableEncounterReleaseView {
                    may_finish: false,
                    blocker: None,
                    required_actors: finished_encounter_dependencies(state)
                        .map_err(|error| error.to_string())?,
                })
            } else {
                let readiness = encounter_release_preflight(state);
                Some(crate::TableEncounterReleaseView {
                    may_finish: readiness.is_ok(),
                    blocker: readiness.as_ref().err().map(ToString::to_string),
                    required_actors: readiness
                        .map(|ready| ready.required_actors)
                        .unwrap_or_default(),
                })
            }
        } else {
            None
        },
        execution: encounter
            .flow
            .as_ref()
            .and_then(|flow| TacticalExecutionVersion::from_flow_version(flow.version))
            .filter(|execution| !execution.is_legacy()),
        hit: hit_reactions::view(read, &own, host)?,
        missile: missile_reactions::view(read, &own, host)?,
        ready: flow
            .into_iter()
            .flat_map(|flow| &flow.ready)
            .filter(|ready| host || own.contains(&ready.actor))
            .map(|ready| {
                let player_controlled = state.characters.values().any(|character| {
                    character.entity_id == ready.actor
                        && character.controlling_player_id.is_some()
                        && matches!(
                            character.status,
                            CharacterStatus::Active | CharacterStatus::Dead
                        )
                }) || state
                    .rules
                    .as_ref()
                    .and_then(|rules| rules.tactical_creatures.as_ref())
                    .and_then(|creatures| creatures.runtime(ready.actor))
                    .is_some_and(|runtime| {
                        matches!(runtime.controller, CreatureController::Player(_))
                    });
                crate::TableReadyView {
                    actor: ready.actor,
                    action: match ready.action {
                        ReadyAction::Attack => "Attack",
                        ReadyAction::Move => "Movement",
                        ReadyAction::Spell { .. } => "Spell",
                    }
                    .into(),
                    may_abandon: if host {
                        !player_controlled
                    } else {
                        own.contains(&ready.actor)
                    },
                }
            })
            .collect(),
        round: timing.map(|timing| timing.round),
        active_actor: active.filter(|actor| host || known.contains(actor)),
        phase,
        battlefield: host.then(|| encounter.battlefield.clone()),
        participants: if host {
            encounter.participants.clone()
        } else {
            vec![]
        },
        combatant_sources: if host {
            encounter
                .participants
                .iter()
                .map(|participant| {
                    let source = state
                        .rules
                        .as_ref()
                        .and_then(|r| r.tactical_creatures.as_ref())
                        .and_then(|c| c.profile(participant.entity_id))
                        .map(|profile| TacticalSource::Creature {
                            definition_id: profile.source.definition_id.clone(),
                        })
                        .unwrap_or(TacticalSource::Character);
                    let mut combatant = TacticalCombatant {
                        actor: participant.entity_id,
                        source: source.clone(),
                        surprised: false,
                    };
                    let (initiative_modifier, normal_mode) =
                        preview_initiative_circumstances(state, &combatant)
                            .map_err(|e| e.to_string())?;
                    combatant.surprised = true;
                    let (_, surprised_mode) = preview_initiative_circumstances(state, &combatant)
                        .map_err(|e| e.to_string())?;
                    Ok(crate::TableCombatantSource {
                        actor: participant.entity_id,
                        source,
                        initiative_modifier,
                        normal_mode,
                        surprised_mode,
                    })
                })
                .collect::<Result<Vec<_>, String>>()?
        } else {
            vec![]
        },
        observers,
        initiative,
        ties,
        continuation: flow
            .and_then(|flow| flow.resolution.as_ref())
            .and_then(|resolution| {
                choices::continuation_read(read, resolution, &own, host, Some(encounter))
            }),
        may_fail_save: state
            .rules
            .as_ref()
            .and_then(|rules| rules.pending.as_ref())
            .and_then(|pending| choices::save_actor(pending, &own, host)),
        legendary_resistance: flow
            .and_then(|flow| flow.resolution.as_ref())
            .and_then(|resolution| resolution.failed_save.as_ref())
            .map(|failed| failed.pending.key.subject)
            .filter(|actor| host || own.contains(actor)),
        legendary_action: flow
            .and_then(|flow| flow.resolution.as_ref())
            .and_then(|resolution| resolution.legendary_window.as_ref())
            .and_then(|window| match window.work.kind {
                TacticalWorkKind::LegendaryWindow { actor } => Some(actor),
                _ => None,
            })
            .filter(|actor| host || own.contains(actor)),
        attack_options: match active.filter(|actor| host || own.contains(actor)) {
            Some(actor)
                if flow.is_some_and(|flow| {
                    flow.phase == TacticalPhase::Active && flow.resolution.is_none()
                }) && state
                    .rules
                    .as_ref()
                    .is_some_and(|rules| rules.pending.is_none()) =>
            {
                attacks::options_with_ground(read, actor, pack)?
            }
            _ => None,
        },
        shield_options: match active.filter(|actor| host || own.contains(actor)) {
            Some(actor)
                if flow.is_some_and(|flow| {
                    flow.phase == TacticalPhase::Active && flow.resolution.is_none()
                }) && state
                    .rules
                    .as_ref()
                    .is_some_and(|rules| rules.pending.is_none()) =>
            {
                shields::options(read, actor)?
            }
            _ => None,
        },
        area_options: match active.filter(|actor| host || own.contains(actor)) {
            Some(actor)
                if flow.is_some_and(|flow| {
                    flow.phase == TacticalPhase::Active && flow.resolution.is_none()
                }) && state
                    .rules
                    .as_ref()
                    .is_some_and(|rules| rules.pending.is_none()) =>
            {
                areas::options(state, actor)?
            }
            _ => None,
        },
        casting_options: match active.filter(|actor| host || own.contains(actor)) {
            Some(actor)
                if flow.is_some_and(|flow| {
                    flow.phase == TacticalPhase::Active && flow.resolution.is_none()
                }) && state
                    .rules
                    .as_ref()
                    .is_some_and(|rules| rules.pending.is_none()) =>
            {
                if source_access {
                    casting::options_v2(read, actor)?
                } else {
                    casting::options(read, actor)?
                }
            }
            _ => None,
        },
        movement_options: match active.filter(|actor| host || own.contains(actor)) {
            Some(actor)
                if flow.is_some_and(|flow| {
                    flow.phase == TacticalPhase::Active && flow.resolution.is_none()
                }) && state
                    .rules
                    .as_ref()
                    .is_some_and(|rules| rules.pending.is_none()) =>
            {
                movement::options(read, actor)?
            }
            _ => None,
        },
        opportunity: flow
            .and_then(|flow| flow.resolution.as_ref())
            .and_then(|resolution| resolution.movement.as_ref())
            .and_then(|movement| movement.opportunity.as_ref())
            .filter(|window| host || own.contains(&window.reactor))
            .map(|window| movement::opportunity_read(read, window))
            .transpose()?,
        liquid_landing: flow
            .and_then(|flow| flow.resolution.as_ref())
            .and_then(|resolution| {
                resolution
                    .falls
                    .iter()
                    .find(|fall| fall.stage == TacticalFallStage::LandingChoice)
            })
            .filter(|fall| host || own.contains(&fall.actor))
            .map(|fall| crate::TableLiquidLandingView { actor: fall.actor }),
        attack_decision: flow
            .and_then(|flow| flow.resolution.as_ref())
            .and_then(|resolution| resolution.attack.as_ref())
            .filter(|attack| host || own.contains(&attack.actor))
            .and_then(|attack| {
                let kind = match attack.stage {
                    TacticalAttackStage::KnockoutChoice => crate::TableAttackDecisionKind::Knockout,
                    TacticalAttackStage::MasteryChoice => crate::TableAttackDecisionKind::Graze,
                    _ => return None,
                };
                Some(crate::TableAttackDecision {
                    actor: attack.actor,
                    kind,
                })
            }),
        budget: flow
            .filter(|_| host || active.is_some_and(|actor| own.contains(&actor)))
            .and_then(|flow| {
                timing.map(|timing| crate::TableTacticalBudget {
                    movement_spent: flow.budget.movement_spent,
                    attacks_remaining: flow.budget.attacks_remaining,
                    action_spent: timing.action_spent,
                    bonus_action_spent: timing.bonus_action_spent,
                    reaction_available: active
                        .is_some_and(|actor| !timing.reactions_spent.contains(&actor)),
                })
            }),
    }))
}
