//! Session-bound encounter setup and viewer-specific presentation.
use dmd_domain::*;
use dmd_rules::{RulesPack, tactical::*};

use crate::{TableBattlefieldSetup, table_engine::table};

#[path = "table_attacks.rs"]
mod attacks;
#[path = "table_casting.rs"]
mod casting;
#[path = "table_tactical_choices.rs"]
mod choices;
#[path = "table_movement.rs"]
mod movement;
#[path = "table_shields.rs"]
mod shields;

pub(crate) fn view(
    state: &CampaignState,
    viewer: &crate::TableViewer,
) -> Result<Option<crate::TableTacticalView>, String> {
    let Some(encounter) = &state.encounter else {
        return Ok(None);
    };
    let host = matches!(viewer, crate::TableViewer::Host);
    let own = state.characters.values().filter(|character| {
        matches!(viewer, crate::TableViewer::Player(player) if character.controlling_player_id == Some(*player))
            && matches!(character.status, CharacterStatus::Active | CharacterStatus::Dead)
            && encounter.participant(character.entity_id).is_some()
    }).map(|character| character.entity_id).collect::<std::collections::HashSet<_>>();
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
        encounter_id: encounter.id,
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
            .and_then(|resolution| choices::continuation(resolution, &own, host)),
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
                attacks::options(state, actor)?
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
                shields::options(state, actor)?
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
                casting::options(state, actor)?
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
                movement::options(state, actor)?
            }
            _ => None,
        },
        opportunity: flow
            .and_then(|flow| flow.resolution.as_ref())
            .and_then(|resolution| resolution.movement.as_ref())
            .and_then(|movement| movement.opportunity.as_ref())
            .filter(|window| host || own.contains(&window.reactor))
            .map(|window| movement::opportunity(state, window))
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

pub(crate) fn prepare(
    state: &CampaignState,
    meta: &CommandMeta,
    setup: &TableBattlefieldSetup,
    pack: &RulesPack,
) -> Result<TacticalTransition, String> {
    bounded_text(&setup.name, 200)?;
    if state.encounter.is_some()
        || state.scenes.contains_key(&setup.scene_id)
        || setup.location_id.0.is_nil()
        || setup.scene_id.0.is_nil()
        || setup.encounter_id.0.is_nil()
        || setup.characters.is_empty()
        || setup.characters.len().saturating_add(setup.creatures.len()) > 100
    {
        return Err("Choose a new encounter and scene with at least one character.".into());
    }
    let mut next = state.clone();
    if let Some(location) = next.locations.get(&setup.location_id) {
        if location.campaign_id != meta.campaign_id {
            return Err("The location belongs to another campaign.".into());
        }
    } else {
        next.locations.insert(
            setup.location_id,
            Location {
                id: setup.location_id,
                campaign_id: meta.campaign_id,
                display_name: setup.name.clone(),
                parent_location_id: None,
            },
        );
    }
    let mut participants = Vec::new();
    for placement in &setup.characters {
        let character = state
            .characters
            .get(&placement.character_id)
            .ok_or("Unknown encounter character.")?;
        let player = character
            .controlling_player_id
            .ok_or("An encounter character requires a controller.")?;
        let session = meta
            .session_id
            .ok_or("Encounter setup requires an active session.")?;
        table(state)?.validate_attendance(session, player, placement.character_id)?;
        let profile = table(state)?
            .character_profiles
            .get(&placement.character_id)
            .ok_or("The character has no source creation profile.")?;
        if state
            .rules
            .as_ref()
            .and_then(|r| r.tactical_inventory.as_ref())
            .and_then(|inventory| inventory.receipt(placement.character_id))
            .is_none()
        {
            return Err(
                "Prepare each character's starting equipment before encounter setup.".into(),
            );
        }
        participants.push(TacticalParticipant {
            entity_id: character.entity_id,
            public_label: character.display_name.clone(),
            position: placement.position,
            size: match profile.size {
                CharacterSize::Small => CreatureSize::Small,
                CharacterSize::Medium => CreatureSize::Medium,
            },
            height: placement.height,
            reach: 10,
            movement: MovementProfile {
                walk: u32::from(profile.speed_feet) * 2,
                climb: None,
                swim: None,
                fly: None,
                burrow: None,
                hover: false,
            },
            senses: Senses::default(),
            allies: placement.allies.clone(),
            enemies: placement.enemies.clone(),
        });
        next.entities
            .get_mut(&character.entity_id)
            .ok_or("Character entity is absent.")?
            .location_id = Some(setup.location_id);
    }
    for placement in &setup.creatures {
        bounded_text(&placement.public_label, 200)?;
        let world = state
            .entities
            .get(&placement.actor)
            .ok_or("Unknown source creature.")?;
        let profile = state
            .rules
            .as_ref()
            .and_then(|r| r.tactical_creatures.as_ref())
            .and_then(|creatures| creatures.profile(placement.actor))
            .ok_or("Creature source profile is absent.")?;
        if world.existence != EntityExistence::Present
            || state
                .rules
                .as_ref()
                .and_then(|r| r.tactical_inventory.as_ref())
                .and_then(|i| i.loadout(placement.actor))
                .is_none()
        {
            return Err("Place a living source creature with its prepared equipment.".into());
        }
        let source = dmd_rules::tactical_creatures::source_for_profile(profile)
            .map_err(|e| e.to_string())?;
        let reach = source
            .features
            .iter()
            .filter_map(|feature| match &feature.feature {
                dmd_rules::tactical_definitions::MonsterFeature::Attack {
                    delivery: dmd_rules::tactical_definitions::AttackDelivery::Melee { reach_feet },
                    ..
                } => Some(u32::from(*reach_feet) * 2),
                _ => None,
            })
            .max()
            .unwrap_or(10);
        participants.push(TacticalParticipant {
            entity_id: placement.actor,
            public_label: placement.public_label.clone(),
            position: placement.position,
            size: profile.size,
            height: placement.height,
            reach,
            movement: dmd_rules::tactical_creatures::creature_movement(source),
            senses: dmd_rules::tactical_creatures::creature_senses(source),
            allies: placement.allies.clone(),
            enemies: placement.enemies.clone(),
        });
        next.entities
            .get_mut(&placement.actor)
            .ok_or("Creature entity is absent.")?
            .location_id = Some(setup.location_id);
    }
    next.scenes.insert(
        setup.scene_id,
        Scene {
            id: setup.scene_id,
            campaign_id: meta.campaign_id,
            location_id: setup.location_id,
            mode: SceneMode::Combat,
            status: SceneStatus::Active,
            started_at: state.clock.now,
            presences: participants
                .iter()
                .map(|p| ScenePresence {
                    entity_id: p.entity_id,
                    role: PresenceRole::Participant,
                })
                .collect(),
        },
    );
    let encounter = TacticalEncounter {
        id: setup.encounter_id,
        scene_id: setup.scene_id,
        battlefield: setup.battlefield.clone(),
        participants,
        knowledge: vec![],
        origin: meta.clone(),
        geometry_ruling: setup.geometry_ruling.clone(),
        flow: None,
    };
    resolve_tactical(
        &next,
        meta,
        &TacticalAction::Establish {
            encounter: Box::new(encounter),
        },
        pack,
    )
    .map_err(|error| error.to_string())
}
