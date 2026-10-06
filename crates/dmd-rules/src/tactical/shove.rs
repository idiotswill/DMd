//! SRD190 ordinary Shove. The existing frame pump owns every save and consequence.
mod geometry;
mod validation;
use super::turns::*;
use super::*;

pub(super) use geometry::{choose_outcome, rule_push};
pub(super) use validation::validate;
pub(super) use validation::validate_fall;
pub(super) use validation::validate_work;

fn current(state: &CampaignState) -> Result<&TacticalShove, RulesError> {
    resolution(state)?
        .shove
        .as_deref()
        .ok_or_else(|| invalid("Shove work absent"))
}
fn current_mut(state: &mut CampaignState) -> Result<&mut TacticalShove, RulesError> {
    resolution_mut(state)?
        .shove
        .as_deref_mut()
        .ok_or_else(|| invalid("Shove work absent"))
}
fn require_execution(state: &CampaignState) -> Result<(), RulesError> {
    // Retained flow4 work keeps its paid semantics. The central live gate admits
    // fresh Shove on live flow5/7, before this shared restore guard.
    let version = flow(state)?.version;
    if version != TacticalExecutionVersion::ShieldMissileV1.flow_version()
        && version != TacticalExecutionVersion::EncounterReleaseV1.flow_version()
        && version != TacticalExecutionVersion::ReleasedTimeV1.flow_version()
    {
        return Err(prerequisite(
            "Shove requires the current encounter execution.",
        ));
    }
    Ok(())
}

/// Unlike legacy privileged routing, the new action cannot substitute Host input
/// for an actually owned body's choices, including a player-owned source creature.
pub(super) fn authorize_owner(
    state: &CampaignState,
    meta: &CommandMeta,
    actor: EntityId,
) -> Result<(), RulesError> {
    if let Some(player) = controller(state, actor) {
        if meta.issuer != CommandIssuer::Player(player)
            || meta.actor != Some(AgentRef::Entity(actor))
        {
            return Err(RulesError::Unauthorized);
        }
    } else if !host_source(state, actor) {
        return Err(prerequisite(
            "This actor requires its supported controller path.",
        ));
    }
    authorize(state, meta, actor)
}

fn host_source(state: &CampaignState, actor: EntityId) -> bool {
    state
        .rules
        .as_ref()
        .and_then(|r| r.tactical_creatures.as_ref())
        .and_then(|c| c.runtime(actor))
        .is_some_and(|r| r.controller == CreatureController::Host)
        && flow(state).is_ok_and(|f| {
            f.combatants
                .iter()
                .any(|c| c.actor == actor && matches!(c.source, TacticalSource::Creature { .. }))
        })
}

fn source_pin(
    state: &CampaignState,
    actor: EntityId,
) -> Result<Option<CreatureSourcePin>, RulesError> {
    match &flow(state)?
        .combatants
        .iter()
        .find(|c| c.actor == actor)
        .ok_or_else(|| invalid("Shove actor source absent"))?
        .source
    {
        TacticalSource::Character => {
            let profile = state
                .table
                .as_ref()
                .and_then(|t| t.character_profiles.values().find(|p| p.entity_id == actor))
                .ok_or_else(|| prerequisite("Shove needs an authenticated character profile."))?;
            let size = match profile.size {
                CharacterSize::Small => CreatureSize::Small,
                CharacterSize::Medium => CreatureSize::Medium,
            };
            if encounter(state)?
                .participant(actor)
                .is_none_or(|p| p.size != size)
            {
                return Err(invalid("Shove body size differs from character source"));
            }
            Ok(None)
        }
        TacticalSource::Creature { definition_id } => {
            let profile = state
                .rules
                .as_ref()
                .and_then(|r| r.tactical_creatures.as_ref())
                .and_then(|c| c.profile(actor))
                .ok_or_else(|| prerequisite("Shove needs an authenticated creature profile."))?;
            if profile.source.definition_id != *definition_id {
                return Err(invalid("Shove creature source differs from encounter"));
            }
            if encounter(state)?
                .participant(actor)
                .is_none_or(|p| p.size != profile.size)
            {
                return Err(invalid("Shove body size differs from creature source"));
            }
            crate::tactical_creatures::source_for_profile(profile)
                .map_err(|e| invalid(&e.to_string()))?;
            Ok(Some(profile.source.clone()))
        }
    }
}

fn opposition(state: &CampaignState, actor: EntityId, target: EntityId) -> Result<(), RulesError> {
    let e = encounter(state)?;
    let a = e
        .participant(actor)
        .ok_or_else(|| invalid("Shove actor absent"))?;
    let t = e
        .participant(target)
        .ok_or_else(|| invalid("Shove target absent"))?;
    let owned_a = controller(state, actor).is_some();
    let owned_t = controller(state, target).is_some();
    if (owned_a && owned_t)
        || (!owned_a && !host_source(state, actor))
        || (!owned_t && !host_source(state, target))
        || !a.enemies.contains(&target)
        || !t.enemies.contains(&actor)
        || a.allies.contains(&target)
        || t.allies.contains(&actor)
    {
        return Err(prerequisite(
            "This Shove requires a supported opposing controller relationship; other cases need explicit table consent.",
        ));
    }
    Ok(())
}

fn difficulty(state: &CampaignState, actor: EntityId) -> Result<i32, RulesError> {
    let rules = state.rules.as_ref().ok_or(RulesError::Uninitialized)?;
    let entity = rules
        .entities
        .get(&actor)
        .ok_or_else(|| invalid("Shove mechanics absent"))?;
    let proficiency = if source_pin(state, actor)?.is_some() {
        let profile = rules
            .tactical_creatures
            .as_ref()
            .and_then(|c| c.profile(actor))
            .ok_or_else(|| invalid("Shove source profile absent"))?;
        i32::from(
            crate::tactical_creatures::creature_proficiency_bonus(profile)
                .map_err(|e| invalid(&e.to_string()))?,
        )
    } else {
        crate::proficiency_bonus(entity.level)
    };
    Ok(8 + crate::ability_modifier(entity.ability_scores[Ability::Strength.index()]) + proficiency)
}

fn cover_bonus(
    state: &CampaignState,
    actor: EntityId,
    target: EntityId,
) -> Result<i32, RulesError> {
    let e = encounter(state)?;
    let a = e
        .participant(actor)
        .ok_or_else(|| invalid("Shove actor absent"))?;
    let t = e
        .participant(target)
        .ok_or_else(|| invalid("Shove target absent"))?;
    crate::spatial::cover_from(
        e,
        a.position,
        t.volume().map_err(|e| invalid(&e))?,
        &[actor, target],
    )
    .and_then(|cover| cover.armor_and_dexterity_bonus())
    .map_err(|_| prerequisite("The selected body action needs supported direct reach and cover."))
}

pub(super) fn begin(
    state: &mut CampaignState,
    meta: &CommandMeta,
    target: EntityId,
) -> Result<(), RulesError> {
    require_execution(state)?;
    if flow(state)?.phase != TacticalPhase::Active || flow(state)?.resolution.is_some() {
        return Err(RulesError::Pending);
    }
    let actor = active(state)?;
    authorize_owner(state, meta, actor)?;
    if actor == target {
        return Err(prerequisite(
            "Self-directed Shove needs its explicit adjudication path.",
        ));
    }
    // Location first: private source traits cannot become a guessed-target oracle.
    super::attacks::admit_body_target(state, actor, target)?;
    opposition(state, actor, target)?;
    super::falling::require_settled_before_action(state)?;
    let rules = state.rules.as_ref().ok_or(RulesError::Uninitialized)?;
    if !crate::tactical_conditions::can_act(rules, actor)?
        || !crate::tactical_conditions::may_harm(rules, actor, target)
    {
        return Err(prerequisite(
            "The actor cannot take this harmful body action.",
        ));
    }
    let e = encounter(state)?;
    let a = e
        .participant(actor)
        .ok_or_else(|| invalid("Shove actor absent"))?;
    let t = e
        .participant(target)
        .ok_or_else(|| invalid("Shove target absent"))?;
    if crate::spatial::participant_distance(a, t).map_err(|e| invalid(&e.to_string()))? > 10
        || t.size.rank() > a.size.rank() + 1
    {
        return Err(prerequisite(
            "The target is outside this body's Shove reach or size limit.",
        ));
    }
    cover_bonus(state, actor, target)?;
    let actor_source = source_pin(state, actor)?;
    let target_source = source_pin(state, target)?;
    let actor_from = a.position;
    let target_from = t.position;
    let difficulty = difficulty(state, actor)?;
    let mut budget = flow(state)?.budget.clone();
    let new_action = budget.attacks_remaining == 0;
    let window = if new_action {
        WeaponActionWindow {
            id: meta.id,
            kind: WeaponActionKind::AttackAction,
        }
    } else {
        budget
            .attack_window
            .ok_or_else(|| invalid("Shove attack opportunity absent"))?
    };
    if window.kind != WeaponActionKind::AttackAction {
        return Err(prerequisite("Shove needs an Attack action attack."));
    }
    let now = state.clock.now;
    let rules = state.rules.as_mut().ok_or(RulesError::Uninitialized)?;
    if new_action {
        crate::tactical_budget::start_attack_action(rules, &mut budget, actor, 1)?;
        budget.attack_window = Some(window);
    }
    crate::tactical_budget::spend_attack(&mut budget)?;
    crate::kernel::interrupt_rest(rules, actor, now);
    let turn_number = rules
        .timing
        .as_ref()
        .ok_or_else(|| invalid("Turn absent"))?
        .turn_number;
    budget.movement_progress = None;
    budget.movement_origin = None;
    flow_mut(state)?.budget = budget;
    let work_trace = super::work_trace::initial(state)?;
    flow_mut(state)?.resolution = Some(Box::new(TacticalResolution {
        origin: meta.clone(),
        context: TacticalResolutionContext::Turn(TacticalTurnContext {
            actor,
            number: turn_number,
            boundary: TurnBoundary::Start,
        }),
        frames: vec![],
        pending: None,
        failed_save: None,
        legendary_window: None,
        attack: None,
        hit_review: None,
        movement: None,
        casts: vec![],
        missiles: vec![],
        falls: vec![],
        areas: vec![],
        work_trace,
        next_occurrence: 0,
        shove: Some(Box::new(TacticalShove {
            origin: meta.clone(),
            actor,
            target,
            window,
            actor_source,
            target_source,
            actor_from,
            target_from,
            difficulty,
            stage: TacticalShoveStage::Queued,
            selected: None,
            save: None,
            push: None,
            effect: None,
        })),
    }));
    push_frame(state, vec![TacticalWorkKind::BeginShove])?;
    pump(state, meta)
}

pub(super) fn waiting(state: &CampaignState) -> bool {
    state
        .encounter
        .as_ref()
        .and_then(|e| e.flow.as_ref())
        .and_then(|f| f.resolution.as_ref())
        .and_then(|r| r.shove.as_ref())
        .is_some_and(|s| s.selected.is_some())
}

pub(super) fn is_work(kind: &TacticalWorkKind) -> bool {
    matches!(
        kind,
        TacticalWorkKind::BeginShove
            | TacticalWorkKind::ShoveSave
            | TacticalWorkKind::ChooseShoveOutcome
            | TacticalWorkKind::FinishShove
    )
}

pub(super) fn start(
    state: &mut CampaignState,
    work: &TacticalWorkItem,
) -> Result<bool, RulesError> {
    match work.kind {
        TacticalWorkKind::BeginShove => {
            let s = current_mut(state)?;
            if s.stage != TacticalShoveStage::Queued {
                return Err(invalid("Shove already begun"));
            }
            s.stage = TacticalShoveStage::SaveChoice;
            s.selected = Some(work.clone());
        }
        TacticalWorkKind::ChooseShoveOutcome => {
            let s = current_mut(state)?;
            if s.stage != TacticalShoveStage::Saving
                || s.save.as_ref().and_then(|s| s.final_success) != Some(false)
            {
                return Err(invalid("Shove outcome has no final failed save"));
            }
            s.stage = TacticalShoveStage::OutcomeChoice;
            s.selected = Some(work.clone());
        }
        TacticalWorkKind::FinishShove => {
            if current(state)?.stage != TacticalShoveStage::Resolving {
                return Err(invalid("Shove cleanup out of order"));
            }
            // Keep receipts until the common pump drops this fully settled resolution.
        }
        _ => return Ok(false),
    }
    Ok(true)
}

pub(super) fn key(
    state: &CampaignState,
    work: &TacticalWorkItem,
) -> Result<TacticalRollKey, RulesError> {
    if work.kind != TacticalWorkKind::ShoveSave {
        return Err(invalid("Shove choice has no raw roll"));
    }
    let s = current(state)?;
    Ok(TacticalRollKey {
        origin: s.origin.id,
        role: TacticalRollRole::ShoveSave,
        subject: s.target,
        occurrence: work.occurrence,
    })
}

fn derive_request(
    state: &CampaignState,
    ability: ShoveSaveAbility,
    key: TacticalRollKey,
) -> Result<Option<RollRequest>, RulesError> {
    let s = current(state)?;
    let rules = state.rules.as_ref().ok_or(RulesError::Uninitialized)?;
    let circumstance = Circumstances {
        advantage: false,
        disadvantage: super::attacks::body_untrained_armor(state, s.target)?,
        ..Circumstances::default()
    };
    let disposition = crate::tactical_conditions::save_conditions(
        rules,
        s.target,
        ability.ability(),
        circumstance,
        dodge_context(state, s.target)?,
    )?;
    let crate::tactical_conditions::TestDisposition::Roll(mode) = disposition else {
        return Ok(None);
    };
    let modifier = super::continuations::save_modifier(state, s.target, ability.ability())?
        + if ability == ShoveSaveAbility::Dexterity {
            cover_bonus(state, s.actor, s.target)?
        } else {
            0
        };
    Ok(Some(RollRequest {
        id: key.request_id(),
        roller: Some(s.target),
        dice: vec![DieSpec {
            count: 1,
            sides: 20,
        }],
        modifier,
        mode,
        visibility: super::continuations::visibility(state, s.target),
        reason: "Shove saving throw".into(),
    }))
}

pub(super) fn choose_save(
    state: &mut CampaignState,
    meta: &CommandMeta,
    ability: ShoveSaveAbility,
) -> Result<(), RulesError> {
    require_execution(state)?;
    let s = current(state)?;
    if s.stage != TacticalShoveStage::SaveChoice {
        return Err(RulesError::Pending);
    }
    authorize_owner(state, meta, s.target)?;
    let selected = s
        .selected
        .clone()
        .ok_or_else(|| invalid("Shove save choice lacks work"))?;
    let key = TacticalRollKey {
        origin: s.origin.id,
        role: TacticalRollRole::ShoveSave,
        subject: s.target,
        occurrence: resolution(state)?.next_occurrence,
    };
    let request = derive_request(state, ability, key)?;
    let previous = super::work_trace::enter(state, &selected)?;
    let s = current_mut(state)?;
    s.selected = None;
    s.stage = TacticalShoveStage::Saving;
    s.save = Some(TacticalShoveSave {
        ability,
        chosen_by: meta.clone(),
        key,
        request,
        final_success: None,
        resolved_by: None,
    });
    push_frame(state, vec![TacticalWorkKind::ShoveSave])?;
    super::work_trace::leave(state, previous)?;
    pump(state, meta)
}

pub(super) fn request(
    state: &CampaignState,
    work: &TacticalWorkItem,
    key: TacticalRollKey,
) -> Result<Option<RollRequest>, RulesError> {
    if self::key(state, work)? != key {
        return Err(invalid("Shove raw identity differs"));
    }
    let s = current(state)?;
    let save = s
        .save
        .as_ref()
        .ok_or_else(|| invalid("Shove saving ability absent"))?;
    if save.key != key {
        return Err(invalid("Shove saving occurrence differs"));
    }
    if save.final_success.is_none() && derive_request(state, save.ability, key)? != save.request {
        return Err(invalid(
            "Shove request differs from its current source facts",
        ));
    }
    Ok(save.request.clone())
}

pub(super) fn save_failed(
    state: &CampaignState,
    pending: &TacticalPendingWork,
    result: Option<&RollResult>,
) -> Result<bool, RulesError> {
    let Some(result) = result else {
        return Ok(true);
    };
    let request = request(state, &pending.work, pending.key)?
        .ok_or_else(|| invalid("Automatic Shove save has no dice"))?;
    Ok(!crate::test_outcome::ability_test_success(
        &request.resolve(result)?,
        current(state)?.difficulty,
        &state
            .rules
            .as_ref()
            .ok_or(RulesError::Uninitialized)?
            .house_rules,
    )?)
}

pub(super) fn finish_save(
    state: &mut CampaignState,
    meta: &CommandMeta,
    pending: &TacticalPendingWork,
    result: Option<&RollResult>,
    forced_success: bool,
) -> Result<(), RulesError> {
    let succeeded = forced_success || !save_failed(state, pending, result)?;
    let s = current_mut(state)?;
    let save = s
        .save
        .as_mut()
        .ok_or_else(|| invalid("Shove save absent"))?;
    save.final_success = Some(succeeded);
    save.resolved_by = Some(meta.clone());
    if succeeded {
        s.stage = TacticalShoveStage::Resolving;
    }
    push_frame(
        state,
        vec![if succeeded {
            TacticalWorkKind::FinishShove
        } else {
            TacticalWorkKind::ChooseShoveOutcome
        }],
    )
}
