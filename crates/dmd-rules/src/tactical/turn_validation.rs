use super::turns::*;
use super::*;
use std::collections::HashSet;

fn provenance(
    state: &CampaignState,
    meta: &CommandMeta,
    subject: EntityId,
) -> Result<(), RulesError> {
    validate_equipment_change_origin(state, meta, subject).map_err(|e| invalid(&e))
}
pub(super) fn pending(state: &CampaignState, pending: &PendingRoll) -> Result<(), RulesError> {
    let PendingPurpose::TacticalResolution { encounter: id, key } = pending.purpose else {
        return Err(invalid("not a tactical resolution request"));
    };
    let r = resolution(state)?;
    let p = r
        .pending
        .as_ref()
        .ok_or_else(|| invalid("request lacks selected work"))?;
    if id != encounter(state)?.id
        || p.key != key
        || key != super::continuations::key(state, &p.work)?
        || pending.request
            != super::continuations::request(state, &p.work, key)?
                .ok_or_else(|| invalid("automatic failure has no request"))?
        || pending.ruling
            != super::continuations::ruling(
                key.role,
                &state
                    .rules
                    .as_ref()
                    .ok_or(RulesError::Uninitialized)?
                    .house_rules,
            )
        || pending.issued_by.expected_event_sequence < r.origin.expected_event_sequence
    {
        return Err(invalid(
            "tactical pending request differs from retained source work",
        ));
    }
    provenance(state, &pending.issued_by, key.subject)?;
    selected_applicable(state, &p.work)?;
    Ok(())
}

pub(super) fn selected_applicable(
    state: &CampaignState,
    work: &TacticalWorkItem,
) -> Result<(), RulesError> {
    if let TacticalWorkKind::Effect { ticket } = work.kind {
        let t = super::continuations::ticket(state, ticket)?;
        if !crate::tactical_effects::trigger_is_applicable(effects(state)?, t)
            .map_err(|e| invalid(&e.to_string()))?
        {
            return Err(invalid("pending source effect is inactive"));
        }
    }
    if let TacticalWorkKind::ConcentrationSave { actor, group, .. } = work.kind
        && state
            .rules
            .as_ref()
            .and_then(|r| r.entities.get(&actor))
            .is_none_or(|e| e.concentration != Some(group))
    {
        return Err(invalid("selected concentration save lost its source group"));
    }
    Ok(())
}

fn validate_work(
    state: &CampaignState,
    r: &TacticalResolution,
    work: &TacticalWorkItem,
) -> Result<(), RulesError> {
    let rules = state.rules.as_ref().ok_or(RulesError::Uninitialized)?;
    if work.occurrence >= r.next_occurrence {
        return Err(invalid("future work occurrence"));
    }
    let actor = match &work.kind {
        TacticalWorkKind::SpellProgram { .. } | TacticalWorkKind::FinishSpell { .. } => {
            super::casting::validate_work(state, work)?
        }
        TacticalWorkKind::MoveSegment => {
            r.movement
                .as_ref()
                .ok_or_else(|| invalid("movement work lacks accepted intent"))?
                .actor
        }
        TacticalWorkKind::MovementOpportunity { reactor } => *reactor,
        TacticalWorkKind::AttackRoll
        | TacticalWorkKind::AttackDamage
        | TacticalWorkKind::FinishAttack => {
            r.attack
                .as_ref()
                .ok_or_else(|| invalid("attack work lacks declaration"))?
                .target
        }
        TacticalWorkKind::DeathSave { actor } => {
            if *actor != r.turn_actor || r.boundary != TurnBoundary::Start {
                return Err(invalid("death save outside owner's start boundary"));
            }
            *actor
        }
        TacticalWorkKind::ConcentrationSave {
            actor,
            damage_taken,
            ..
        } => {
            if *damage_taken == 0 || *damage_taken > 1_000_000 {
                return Err(invalid("invalid concentration damage occurrence"));
            }
            *actor
        }
        TacticalWorkKind::StableRecovery { actor, origin } => {
            provenance(state, &origin.command, *actor)?;
            let recovery = rules
                .tactical_recovery
                .as_ref()
                .and_then(|records| records.get(actor))
                .and_then(|r| r.stable.as_ref())
                .ok_or_else(|| invalid("stable recovery lacks its cause"))?;
            if recovery.origin != *origin || recovery.delay_roll.is_some() {
                return Err(invalid("stable recovery is stale or already rolled"));
            }
            *actor
        }
        TacticalWorkKind::RecoverStable { actor } => *actor,
        TacticalWorkKind::EndOccupiedSpace { actor } => {
            if *actor != r.turn_actor || r.boundary != TurnBoundary::End {
                return Err(invalid(
                    "occupied-space consequence is outside owner's End boundary",
                ));
            }
            *actor
        }
        TacticalWorkKind::CreatureRecharge { actor, feature_id } => {
            let ticket = super::creature_bridge::recharge(state, *actor, feature_id)?;
            if *actor != r.turn_actor
                || r.boundary != TurnBoundary::Start
                || ticket.origin != r.origin
                || ticket.turn.encounter_id != encounter(state)?.id
                || ticket.turn.actor != r.turn_actor
                || ticket.turn.number != r.turn_number
                || ticket.turn.boundary != r.boundary
                || ticket.request.id != super::continuations::key(state, work)?.request_id()
            {
                return Err(invalid(
                    "recharge is not attached to its source own-start occurrence",
                ));
            }
            *actor
        }
        TacticalWorkKind::LegendaryWindow { actor } => {
            let profile = rules
                .tactical_creatures
                .as_ref()
                .and_then(|c| c.profile(*actor))
                .ok_or_else(|| invalid("legendary window lacks a source profile"))?;
            let source = crate::tactical_creatures::source_for_profile(profile)
                .map_err(|e| invalid(&e.to_string()))?;
            if *actor == r.turn_actor
                || r.boundary != TurnBoundary::End
                || source.legendary_budget.is_none()
            {
                return Err(invalid(
                    "legendary window outside another creature's completed turn",
                ));
            }
            *actor
        }
        TacticalWorkKind::Effect { ticket } => {
            let t = super::continuations::ticket(state, *ticket)?;
            if t.origin.command.expected_event_sequence < r.origin.expected_event_sequence {
                return Err(invalid("effect ticket predates this source resolution"));
            }
            t.target
        }
    };
    if !rules.entities.contains_key(&actor) || encounter(state)?.participant(actor).is_none() {
        return Err(invalid("work references absent encounter actor"));
    }
    Ok(())
}

pub(super) fn validate(state: &CampaignState) -> Result<(), RulesError> {
    let f = flow(state)?;
    let rules = state.rules.as_ref().ok_or(RulesError::Uninitialized)?;
    if f.phase != TacticalPhase::Active {
        if f.resolution.is_some()
            || !f.dodges.is_empty()
            || f.budget != TacticalTurnBudget::default()
        {
            return Err(invalid("inactive encounter retains turn work/budgets"));
        }
        return Ok(());
    }
    let timing = rules
        .timing
        .as_ref()
        .ok_or_else(|| invalid("active encounter lacks timing"))?;
    let actor = active(state)?;
    let effect_turn = effects(state)?
        .turn
        .ok_or_else(|| invalid("active encounter lacks initialized first-turn cursor"))?;
    if effect_turn.actor != actor || effect_turn.number != timing.turn_number {
        return Err(invalid("effect cursor differs from the authoritative turn"));
    }
    if let Some(r) = &f.resolution {
        provenance(state, &r.origin, actor)?;
        if r.turn_actor != actor
            || r.turn_number != timing.turn_number
            || r.boundary != effect_turn.boundary
            || r.frames.len() > 128
            || r.next_occurrence > 32_768
        {
            return Err(invalid("invalid turn continuation cursor/capacity"));
        }
        let mut occurrences = HashSet::new();
        let mut ticket_ids = HashSet::new();
        let mut occupied_space_count = 0;
        if usize::from(r.pending.is_some())
            + usize::from(r.failed_save.is_some())
            + usize::from(r.legendary_window.is_some())
            + usize::from(r.movement.as_ref().is_some_and(|m| m.opportunity.is_some()))
            > 1
        {
            return Err(invalid("multiple selected tactical continuations"));
        }
        for (index, frame) in r.frames.iter().enumerate() {
            if frame
                .iter()
                .any(|w| matches!(w.kind, TacticalWorkKind::LegendaryWindow { .. }))
                && (index != 0
                    || !frame
                        .iter()
                        .all(|w| matches!(w.kind, TacticalWorkKind::LegendaryWindow { .. })))
            {
                return Err(invalid("after-turn work must follow all End consequences"));
            }
        }
        for work in r
            .frames
            .iter()
            .flatten()
            .chain(r.pending.iter().map(|p| &p.work))
            .chain(r.failed_save.iter().map(|f| &f.pending.work))
            .chain(r.legendary_window.iter().map(|w| &w.work))
        {
            if !occurrences.insert(work.occurrence) {
                return Err(invalid("duplicate consequence occurrence"));
            }
            validate_work(state, r, work)?;
            if matches!(work.kind, TacticalWorkKind::EndOccupiedSpace { .. }) {
                occupied_space_count += 1;
                if occupied_space_count > 1 {
                    return Err(invalid("duplicate occupied-space End consequence"));
                }
            }
            if let TacticalWorkKind::Effect { ticket } = work.kind
                && !ticket_ids.insert(ticket)
            {
                return Err(invalid("duplicate effect ticket in work"));
            }
        }
        if effects(state)?
            .pending
            .iter()
            .any(|t| !ticket_ids.contains(&t.id))
        {
            return Err(invalid(
                "effect ticket omitted from authoritative continuation",
            ));
        }
        if let Some(window) = &r.legendary_window {
            if r.frames.len() > 1
                || r.frames
                    .iter()
                    .flatten()
                    .any(|w| !matches!(w.kind, TacticalWorkKind::LegendaryWindow { .. }))
            {
                return Err(invalid(
                    "legendary opportunity preceded unfinished End effects",
                ));
            }
            super::creature_bridge::validate_window(state, window)?;
        } else if let Some(failed) = &r.failed_save {
            super::failed_save::validate_failed_save(state, failed)?;
        } else if r.pending.is_some() {
            pending(
                state,
                rules
                    .pending
                    .as_ref()
                    .ok_or_else(|| invalid("selected work lacks dice request"))?,
            )?;
        } else if r.movement.as_ref().is_some_and(|m| m.opportunity.is_some()) {
            if rules.pending.is_some() {
                return Err(invalid("movement opportunity has competing dice"));
            }
        } else if r.attack.as_ref().is_some_and(|a| {
            matches!(
                a.stage,
                TacticalAttackStage::KnockoutChoice | TacticalAttackStage::MasteryChoice
            )
        }) {
            if rules.pending.is_some() {
                return Err(invalid("attack decision has competing dice"));
            }
        } else if rules.pending.is_some() || r.frames.last().is_none_or(|frame| frame.len() < 2) {
            return Err(invalid(
                "continuation was not suspended at a material choice",
            ));
        }
    } else if rules.pending.is_some()
        || !effects(state)?.pending.is_empty()
        || effect_turn.boundary != TurnBoundary::Start
    {
        return Err(invalid(
            "unattached pending work or unfinished end boundary",
        ));
    }
    super::creature_bridge::validate(state)?;
    super::attacks::validate(state)?;
    super::movement::validate(state)?;
    super::casting::validate(state)?;
    if f.budget.dash_grants.len() > 20
        || f.budget.attacks_remaining > 20
        || f.budget.weapon_history.len() > 512
        || (f.budget.attacks_remaining > 0
            && (f.budget.attack_window.is_none() || !timing.action_spent))
        || f.budget.attack_window.is_some_and(|window| {
            window.id.0.is_nil() || window.kind != WeaponActionKind::AttackAction
        })
    {
        return Err(invalid("invalid turn expenditure state"));
    }
    let mut grants = HashSet::new();
    for dash in &f.budget.dash_grants {
        if dash.origin.0.is_nil() || !grants.insert(dash.origin) || !timing.action_spent {
            return Err(invalid("invalid Dash grant"));
        }
        crate::tactical_budget::movement_remaining(&f.budget, dash.speed, &speeds(state, actor)?)?;
    }
    if let Some(origin) = &f.budget.disengaged {
        provenance(state, origin, actor)?;
        authorize(state, origin, actor)?;
        if !timing.action_spent {
            return Err(invalid("Disengage without expenditure"));
        }
    }
    let mut dodgers = HashSet::new();
    for dodge in &f.dodges {
        provenance(state, &dodge.origin, dodge.actor)?;
        authorize(state, &dodge.origin, dodge.actor)?;
        if !dodgers.insert(dodge.actor)
            || dodge.declared_on_turn == 0
            || dodge.declared_on_turn > timing.turn_number
            || (dodge.actor == actor && dodge.declared_on_turn != timing.turn_number)
            || speeds(state, dodge.actor)?.walk == 0
            || !crate::tactical_conditions::can_act(rules, dodge.actor)?
        {
            return Err(invalid("invalid or expired Dodge stance"));
        }
    }
    let mut decisions = HashSet::new();
    for decision in &f.save_decisions {
        provenance(state, &decision.issued_by, decision.key.subject)?;
        provenance(state, &decision.resolved_by, decision.key.subject)?;
        if !decisions.insert(decision.key.request_id())
            || !matches!(
                decision.key.role,
                TacticalRollRole::DeathSave
                    | TacticalRollRole::EffectSave
                    | TacticalRollRole::Concentration
                    | TacticalRollRole::SpellSave
            )
            || decision.resolved_by.expected_event_sequence
                < decision.issued_by.expected_event_sequence
            || rules
                .rolls
                .iter()
                .any(|r| r.request.id == decision.key.request_id())
        {
            return Err(invalid("invalid non-rolled saving throw decision"));
        }
        if decision.failure == TacticalSaveFailure::Voluntary {
            authorize(state, &decision.resolved_by, decision.key.subject)?;
            if !rules
                .cancelled_roll_ids
                .contains(&decision.key.request_id())
            {
                return Err(invalid("voluntary failure lacks retired request identity"));
            }
        } else if decision.issued_by != decision.resolved_by
            || decision.key.role == TacticalRollRole::DeathSave
        {
            return Err(invalid("invalid automatic saving throw failure"));
        }
    }
    let mut items = HashSet::new();
    let location = state
        .scenes
        .get(&encounter(state)?.scene_id)
        .ok_or_else(|| invalid("encounter scene absent"))?
        .location_id;
    for ground in &f.ground_items {
        if !items.insert(ground.item)
            || state
                .items
                .get(&ground.item)
                .is_none_or(|item| item.custody != Custody::Location(location))
            || !encounter(state)?
                .battlefield
                .bounds
                .contains(ground.position)
        {
            return Err(invalid("invalid ground item position/custody"));
        }
        provenance(state, &ground.origin, actor)?;
    }
    Ok(())
}
