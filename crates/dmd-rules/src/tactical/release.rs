//! Read-only release admission. Never clear a cursor to make this scan succeed.
use super::*;
use std::collections::HashSet;

/// The actual table setup route currently admits at most 100 placements.
/// Keep this shared boundary when replacement setup is integrated.
pub const MAX_RELEASE_DEPENDENCIES: usize = 100;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncounterReleaseReadiness {
    /// Sorted only for deterministic diagnostics; not a turn/response ordering.
    pub required_actors: Vec<EntityId>,
    pub next_turn_number: u64,
}

fn expiry(expiry: &TacticalEffectExpiry, now: WorldInstant) -> Result<(), RulesError> {
    match expiry {
        TacticalEffectExpiry::AfterOwnerBoundaries { .. } => Err(prerequisite(
            "settle owner-relative effects on the retained cadence before finishing",
        )),
        TacticalEffectExpiry::AtTime(at) if *at <= now => Err(prerequisite(
            "resolve due effect deadlines before finishing the encounter",
        )),
        _ => Ok(()),
    }
}

/// Whole-campaign dependency union. Raw retained records, including suppressed
/// effects and defense-only spells, own timing; projected conditions do not.
/// Reuse this scan at Finished/session/replacement boundaries, with no mutation.
pub fn retained_encounter_dependencies(state: &CampaignState) -> Result<Vec<EntityId>, RulesError> {
    let proof = super::released_time::validation_for(state)?;
    retained_dependencies(state, false, proof.as_ref())
}

fn retained_dependencies(
    state: &CampaignState,
    allow_new_initiative: bool,
    released: Option<&super::released_time::ReleasedValidation<'_>>,
) -> Result<Vec<EntityId>, RulesError> {
    let rules = state.rules.as_ref().ok_or(RulesError::Uninitialized)?;
    let admitted_initiative = allow_new_initiative
        && rules.pending.as_ref().is_some_and(|pending| {
            state.encounter.as_ref().is_some_and(|encounter| {
                matches!(pending.purpose, PendingPurpose::TacticalInitiative { encounter: id, .. } if id == encounter.id)
                    && encounter.flow.as_ref().is_some_and(|flow| matches!(flow.phase, TacticalPhase::Initiative { .. }))
            })
        });
    if (rules.pending.is_some() && !admitted_initiative)
        || rules.permission.is_some()
        || state
            .table
            .as_ref()
            .is_some_and(|table| table.pending.is_some() || table.roll_context.is_some())
        || rules.entities.values().any(|entity| {
            entity
                .character_features
                .as_ref()
                .is_some_and(|features| features.inspiration_transfer_pending)
        })
    {
        return Err(prerequisite(
            "resolve outstanding table, dice and Inspiration choices before finishing",
        ));
    }
    if rules.tactical_creatures.as_ref().is_some_and(|creatures| {
        creatures.runtime.iter().any(|runtime| {
            runtime.routine.is_some() || runtime.recharge.iter().any(|row| row.pending.is_some())
        })
    }) {
        return Err(prerequisite(
            "finish source routines and recharge rolls before finishing the encounter",
        ));
    }
    // This is independent of the Active-only central-work cursor validator.
    super::creature_bridge::validate_recharge_history(state)?;
    let mut dependencies = HashSet::new();
    for effect in &rules.effects {
        match effect.expires {
            Expiry::AtTurn { .. } => {
                return Err(prerequisite(
                    "settle turn-relative effects before finishing",
                ));
            }
            Expiry::AtTime(at)
                if at <= state.clock.now
                    && !released.is_some_and(|p| p.legacy(state, effect.id)) =>
            {
                return Err(prerequisite(
                    "resolve due legacy effect deadlines before finishing",
                ));
            }
            _ => {}
        }
        dependencies.extend([effect.source, effect.target]);
        dependencies.extend(effect.concentration_owner);
    }
    if let Some(effects) = &rules.tactical_effects {
        if effects
            .pending
            .iter()
            .any(|ticket| !released.is_some_and(|p| p.ticket(state, ticket.id)))
        {
            return Err(prerequisite("resolve scheduled effects before finishing"));
        }
        for group in &effects.groups {
            if group.stage == ConcentrationStage::Casting {
                return Err(prerequisite(
                    "finish the retained concentration cast before finishing",
                ));
            }
            if !released.is_some_and(|p| p.effect(state, group.id, true)) {
                expiry(&group.expires, state.clock.now)?;
            }
            dependencies.insert(group.source.actor);
        }
        for effect in &effects.effects {
            if !released.is_some_and(|p| p.effect(state, effect.id, false)) {
                expiry(&effect.expires, state.clock.now)?;
            }
            if effect.triggers.iter().any(|trigger| {
                matches!(
                    trigger.event,
                    EffectTriggerEvent::Turn { .. } | EffectTriggerEvent::ZoneContact(_)
                ) || matches!(
                    trigger.frequency,
                    EffectTriggerFrequency::OncePerTargetPerTurn { .. }
                )
            }) {
                return Err(prerequisite(
                    "settle retained turn-triggered effects before finishing",
                ));
            }
            dependencies.insert(effect.source.actor);
            match effect.target {
                TacticalEffectTarget::Creature(actor) => {
                    dependencies.insert(actor);
                }
                TacticalEffectTarget::Zone { .. } => {
                    return Err(prerequisite(
                        "retained zone effects require their existing scene",
                    ));
                }
            }
        }
    }
    for entity in rules.entities.values() {
        if entity.hp == 0 && !entity.death.dead && !entity.death.stable && entity.uses_death_saves {
            return Err(prerequisite(
                "settle every dying creature on the retained cadence",
            ));
        }
        if entity.concentration.is_some() {
            dependencies.insert(entity.entity_id);
        }
    }
    if let Some(recoveries) = &rules.tactical_recovery {
        for (actor, recovery) in recoveries {
            if let Some(stable) = &recovery.stable {
                let wake = crate::tactical_damage::stable_wake_at(stable)
                    .map_err(|error| invalid(&error.to_string()))?;
                if wake.is_none_or(|at| at <= state.clock.now)
                    && !released.is_some_and(|p| p.stable(state, *actor))
                {
                    return Err(prerequisite(
                        "resolve recovery dice and due waking before finishing",
                    ));
                }
                dependencies.insert(*actor);
            }
            if recovery.knockout.is_some() || recovery.knockout_rest.is_some() {
                dependencies.insert(*actor);
            }
            if let Some(rest) = &recovery.knockout_rest {
                let due = rest
                    .started_at
                    .0
                    .checked_add(3600)
                    .ok_or_else(|| invalid("knockout rest deadline overflow"))?;
                if due <= state.clock.now.0 {
                    return Err(prerequisite(
                        "resolve completed knockout recovery before finishing",
                    ));
                }
            }
        }
    }
    dependencies.extend(rules.rests.iter().map(|rest| rest.actor));
    if dependencies.len() > MAX_RELEASE_DEPENDENCIES {
        return Err(prerequisite(
            "retained timing dependencies exceed the supported next battlefield",
        ));
    }
    if dependencies.len() == MAX_RELEASE_DEPENDENCIES
        && !dependencies.iter().any(|actor| {
            state.characters.values().any(|character| character.entity_id == *actor)
                || (state.table.as_ref().is_some_and(|table| table.source_actor_access.is_some())
                    && rules.tactical_creatures.as_ref().and_then(|creatures| creatures.runtime(*actor))
                        .is_some_and(|runtime| matches!(runtime.controller, CreatureController::Player(player) if state.players.contains_key(&player))))
        })
    {
        return Err(prerequisite("retained dependencies leave no placement for the required character or owned-source seat"));
    }
    let mut dependencies = dependencies.into_iter().collect::<Vec<_>>();
    dependencies.sort_by_key(|actor| actor.0);
    for actor in &dependencies {
        require_future_placement(state, *actor)?;
    }
    if released.is_some_and(|proof| proof.interval(state)) {
        require_retained_participants(state, &dependencies)?;
    }
    Ok(dependencies)
}

/// Released elapsed work requires actual retained geometry for the entire raw
/// dependency union. Future placement eligibility alone does not provide it.
pub(super) fn require_retained_participants(
    state: &CampaignState,
    dependencies: &[EntityId],
) -> Result<(), RulesError> {
    let current = encounter(state)?;
    if dependencies
        .iter()
        .any(|actor| current.participant(*actor).is_none())
    {
        return Err(prerequisite(
            "released time requires every retained dependency's actual participant position",
        ));
    }
    Ok(())
}

/// The next supported table placement must be possible before timing is removed.
/// Actual attendance and selected geometry still belong to the later setup command.
fn require_future_placement(state: &CampaignState, actor: EntityId) -> Result<(), RulesError> {
    let rules = state.rules.as_ref().ok_or(RulesError::Uninitialized)?;
    let retired_scene = state.encounter.as_ref().map(|encounter| encounter.scene_id);
    if state.scenes.values().any(|scene| {
        Some(scene.id) != retired_scene
            && scene.status == SceneStatus::Active
            && scene.presences.iter().any(|presence| {
                presence.entity_id == actor && presence.role == PresenceRole::Participant
            })
    }) {
        return Err(prerequisite(
            "a retained dependency still belongs to another active scene",
        ));
    }
    if state
        .entities
        .get(&actor)
        .is_none_or(|world| world.existence != EntityExistence::Present)
        || rules
            .entities
            .get(&actor)
            .is_none_or(|entity| entity.death.dead)
    {
        return Err(prerequisite(
            "a retained timing dependency cannot enter the next initiative",
        ));
    }
    let inventory = rules
        .tactical_inventory
        .as_ref()
        .ok_or_else(|| prerequisite("prepare retained actors' equipment before finishing"))?;
    if inventory.loadout(actor).is_none() {
        return Err(prerequisite(
            "retained actor lacks equipment for the next battlefield",
        ));
    }
    if let Some(character) = state
        .characters
        .values()
        .find(|character| character.entity_id == actor)
    {
        if character.status != CharacterStatus::Active
            || character
                .controlling_player_id
                .is_none_or(|id| !state.players.contains_key(&id))
            || state
                .table
                .as_ref()
                .is_none_or(|table| !table.character_profiles.contains_key(&character.id))
            || inventory.receipt(character.id).is_none()
        {
            return Err(prerequisite(
                "retained character cannot enter the supported next battlefield",
            ));
        }
    } else {
        let creatures = rules
            .tactical_creatures
            .as_ref()
            .ok_or_else(|| prerequisite("retained actor lacks a supported creature source"))?;
        let profile = creatures
            .profile(actor)
            .ok_or_else(|| prerequisite("retained actor lacks a supported creature source"))?;
        if creatures.runtime(actor).is_none() {
            return Err(prerequisite("retained creature runtime is absent"));
        }
        crate::tactical_creatures::validate_creature_profile(
            state,
            profile,
            &rules.entities[&actor],
        )
        .map_err(|error| invalid(&error.to_string()))?;
    }
    Ok(())
}

/// Shared by eventual live release, capability derivation, closed legacy upgrade
/// and strict replay. It authorizes no command and writes no gameplay state.
pub fn encounter_release_preflight(
    state: &CampaignState,
) -> Result<EncounterReleaseReadiness, RulesError> {
    let proof = super::released_time::validation_for(state)?;
    encounter_release_preflight_with_released(state, proof.as_ref())
}
pub(super) fn encounter_release_preflight_with_released(
    state: &CampaignState,
    released: Option<&super::released_time::ReleasedValidation<'_>>,
) -> Result<EncounterReleaseReadiness, RulesError> {
    let current = flow(state)?;
    if current.phase != TacticalPhase::Active || current.aftermath.is_none() {
        return Err(prerequisite(
            "conclude hostilities before finishing encounter timing",
        ));
    }
    if current.resolution.is_some()
        || !current.ready.is_empty()
        || !current.dodges.is_empty()
        || current.budget.disengaged.is_some()
        || current.budget.movement_progress.is_some()
        || current.budget.attacks_remaining != 0
        || current.budget.attack_window.is_some()
    {
        return Err(prerequisite(
            "settle held actions, attacks and movement before finishing",
        ));
    }
    super::aftermath::validate_with_released(state, released)?;
    let rules = state.rules.as_ref().ok_or(RulesError::Uninitialized)?;
    let timing = rules
        .timing
        .as_ref()
        .ok_or_else(|| invalid("release lacks retained initiative timing"))?;
    if !timing.reactions_spent.is_empty() {
        return Err(prerequisite(
            "spent Reactions must reach their owners' real Start before finishing",
        ));
    }
    let next_turn_number = timing
        .turn_number
        .checked_add(1)
        .ok_or_else(|| invalid("encounter turn highwater is exhausted"))?;
    if let Some(history) = &state.encounter_history {
        history.validate(state).map_err(|error| invalid(&error))?;
        if history.contains_encounter(encounter(state)?.id)
            || history
                .last()
                .is_some_and(|last| timing.turn_number <= last.final_turn.number)
        {
            return Err(invalid(
                "release reuses a completed encounter or turn epoch",
            ));
        }
    }
    super::falling::require_settled_before_action(state)?;
    let required_actors = retained_dependencies(state, false, released)?;
    Ok(EncounterReleaseReadiness {
        required_actors,
        next_turn_number,
    })
}

/// The application may use a Finished encounter only after all retained proof
/// and dependencies have been checked. An arbitrary historical Finished flag
/// never opens a session, preparation or replacement exception.
pub fn require_finished_encounter(state: &CampaignState) -> Result<(), RulesError> {
    let current = flow(state)?;
    if !TacticalExecutionVersion::from_flow_version(current.version)
        .is_some_and(TacticalExecutionVersion::supports_release)
        || current.phase != TacticalPhase::Finished
        || current.resolution.is_some()
    {
        return Err(prerequisite(
            "finish the existing encounter before preparing another",
        ));
    }
    validate_history(state)
}

pub(super) fn finish(state: &mut CampaignState, meta: &CommandMeta) -> Result<(), RulesError> {
    privileged(meta)?;
    let execution = TacticalExecutionVersion::from_flow_version(flow(state)?.version)
        .filter(|v| v.supports_release())
        .ok_or_else(|| prerequisite("continue this saved encounter before finishing it"))?;
    if execution.supports_released_time() {
        super::released_time::release_preflight(state)?;
    } else {
        encounter_release_preflight(state)?;
    }
    let current = encounter(state)?;
    let f = flow(state)?;
    let timing = state
        .rules
        .as_ref()
        .and_then(|rules| rules.timing.as_ref())
        .ok_or_else(|| invalid("release lacks timing"))?;
    let scene = state
        .scenes
        .get(&current.scene_id)
        .ok_or_else(|| invalid("release scene is absent"))?;
    let receipt = TacticalCompletion {
        encounter_id: current.id,
        scene_id: current.scene_id,
        location_id: scene.location_id,
        encounter_origin: current.origin.clone(),
        initiative_origin: f.origin.clone(),
        conclusion_origin: f
            .aftermath
            .as_ref()
            .ok_or_else(|| invalid("release conclusion is absent"))?
            .origin
            .clone(),
        released_by: meta.clone(),
        predecessor: state
            .encounter_history
            .as_ref()
            .and_then(|history| history.last())
            .map(|receipt| receipt.released_by.id),
        released_at: state.clock.now,
        final_turn: EffectTurn {
            actor: turns::active(state)?,
            number: timing.turn_number,
            boundary: TurnBoundary::Start,
        },
        execution,
    };
    let space = TacticalSceneSpace {
        encounter_id: current.id,
        scene_id: current.scene_id,
        location_id: scene.location_id,
        release: meta.id,
        battlefield: current.battlefield.clone(),
        participants: current
            .participants
            .iter()
            .map(|actor| actor.entity_id)
            .collect(),
        ground_items: f.ground_items.clone(),
    };
    // Only now may the existing lifecycle helpers retire cursors. The full
    // preflight above proved they cannot erase a pending routine or consequence.
    state
        .rules
        .as_mut()
        .ok_or(RulesError::Uninitialized)?
        .timing = None;
    if state
        .rules
        .as_ref()
        .is_some_and(|rules| rules.tactical_effects.is_some())
    {
        let (next, ended) = crate::tactical_effect_adapter::apply_effect_operation(
            state,
            meta,
            &crate::tactical_effects::EffectLifecycleAction {
                step: 0,
                operation: crate::tactical_effects::EffectLifecycleOperation::LeaveCombat,
            },
        )?;
        if !ended.is_empty() {
            return Err(invalid("release unexpectedly ended a retained effect"));
        }
        *state = next;
    }
    let actors = state
        .rules
        .as_ref()
        .and_then(|rules| rules.tactical_creatures.as_ref())
        .map(|creatures| {
            creatures
                .runtime
                .iter()
                .map(|runtime| runtime.actor)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    for actor in actors {
        let current = state
            .rules
            .as_ref()
            .and_then(|rules| rules.tactical_creatures.as_ref())
            .ok_or_else(|| invalid("release lost source creature attachment"))?;
        let transition = crate::tactical_creatures::apply_creature_schedule(
            state,
            current,
            meta,
            &crate::tactical_creatures::CreatureScheduleOperation::LeaveCombat { actor },
        )
        .map_err(|error| invalid(&error.to_string()))?;
        state
            .rules
            .as_mut()
            .ok_or(RulesError::Uninitialized)?
            .tactical_creatures = Some(transition.next);
    }
    state
        .scenes
        .get_mut(&receipt.scene_id)
        .ok_or_else(|| invalid("release scene disappeared"))?
        .status = SceneStatus::Closed;
    let f = flow_mut(state)?;
    f.phase = TacticalPhase::Finished;
    f.budget = TacticalTurnBudget::default();
    f.ground_items.clear();
    let history = state
        .encounter_history
        .get_or_insert_with(|| Box::new(TacticalEncounterHistory::default()));
    history.completions.push(receipt);
    history.spaces.push(space);
    Ok(())
}

/// New completion attachments remain checked before the inactive/no-flow early
/// returns. No historical flow without such an attachment gains an exception.
pub(super) fn require_completion_history(state: &CampaignState) -> Result<(), RulesError> {
    if state.encounter_history.is_none()
        && state
            .encounter
            .as_ref()
            .and_then(|encounter| encounter.flow.as_ref())
            .is_some_and(|flow| {
                TacticalExecutionVersion::from_flow_version(flow.version)
                    .is_some_and(TacticalExecutionVersion::supports_release)
                    && flow.phase == TacticalPhase::Finished
            })
    {
        return Err(invalid(
            "Finished encounter lacks authenticated completion history",
        ));
    }
    Ok(())
}

pub(super) fn validate_history(state: &CampaignState) -> Result<(), RulesError> {
    let proof = super::released_time::validation_for(state)?;
    validate_history_with_released(state, proof.as_ref())
}
pub(super) fn validate_history_with_released(
    state: &CampaignState,
    released: Option<&super::released_time::ReleasedValidation<'_>>,
) -> Result<(), RulesError> {
    require_completion_history(state)?;
    let Some(history) = &state.encounter_history else {
        return Ok(());
    };
    history.validate(state).map_err(|error| invalid(&error))?;
    super::creature_bridge::validate_recharge_history(state)?;
    let current = encounter(state)?;
    let last = history
        .last()
        .ok_or_else(|| invalid("completion history is empty"))?;
    let rules = state.rules.as_ref().ok_or(RulesError::Uninitialized)?;
    if current
        .flow
        .as_ref()
        .is_some_and(|flow| flow.phase == TacticalPhase::Finished)
    {
        let flow = flow(state)?;
        let space = history
            .spaces
            .last()
            .ok_or_else(|| invalid("completed scene space is absent"))?;
        let record = flow
            .aftermath
            .as_ref()
            .ok_or_else(|| invalid("completed aftermath is absent"))?;
        if rules.timing.is_some()
            || (flow.resolution.is_some() && !released.is_some_and(|p| p.interval(state)))
            || !flow.ready.is_empty()
            || !flow.dodges.is_empty()
            || !flow.ground_items.is_empty()
            || flow.budget != TacticalTurnBudget::default()
            || rules
                .tactical_effects
                .as_ref()
                .is_some_and(|effects| effects.turn.is_some() || !effects.trigger_uses.is_empty())
            || rules.tactical_creatures.as_ref().is_some_and(|creatures| {
                creatures.runtime.iter().any(|runtime| {
                    runtime.observed_turn.is_some() || runtime.legendary_window_spent
                })
            })
            || current.battlefield != space.battlefield
            || current
                .participants
                .iter()
                .map(|participant| participant.entity_id)
                .collect::<Vec<_>>()
                != space.participants
            || record.concluded_at > last.released_at
            || record.concluded_on_turn.number == 0
            || record.concluded_on_turn.number > last.final_turn.number
            || record.concluded_on_turn.boundary != TurnBoundary::Start
            || !space.participants.contains(&record.concluded_on_turn.actor)
            || state
                .scenes
                .get(&last.scene_id)
                .is_none_or(|scene| scene.status != SceneStatus::Closed)
        {
            return Err(invalid(
                "Finished encounter differs from its authenticated release boundary",
            ));
        }
        retained_dependencies(state, false, released)?;
    } else {
        if rules
            .timing
            .as_ref()
            .is_some_and(|timing| timing.turn_number <= last.final_turn.number)
        {
            return Err(invalid(
                "replacement initiative reuses a completed turn epoch",
            ));
        }
        if current.flow.as_ref().is_none_or(|flow| {
            matches!(
                flow.phase,
                TacticalPhase::Initiative { .. } | TacticalPhase::InitiativeTies { .. }
            )
        }) {
            let required = retained_dependencies(state, true, released)?;
            if current.participants.len() > MAX_RELEASE_DEPENDENCIES
                || required
                    .iter()
                    .any(|actor| current.participant(*actor).is_none())
            {
                return Err(invalid(
                    "replacement battlefield omits a retained timing dependency or exceeds placement capacity",
                ));
            }
        }
    }
    Ok(())
}
