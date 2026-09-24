use super::*;
use std::collections::BTreeMap;

pub(super) fn request(
    state: &CampaignState,
    group_index: usize,
) -> Result<RollRequest, RulesError> {
    let f = flow(state)?;
    let group = f
        .initiative_groups
        .get(group_index)
        .ok_or_else(|| invalid("initiative group is absent"))?;
    let actor = *group
        .actors
        .first()
        .ok_or_else(|| invalid("empty initiative group"))?;
    let (modifier, mode) = circumstances(state, actor)?;
    Ok(RollRequest {
        id: group.request_id,
        roller: Some(actor),
        dice: vec![DieSpec {
            count: 1,
            sides: 20,
        }],
        modifier,
        mode,
        visibility: if controller(state, actor).is_some() {
            RollVisibility::Public
        } else {
            RollVisibility::Secret
        },
        reason: "Initiative".into(),
    })
}

pub(super) fn circumstances(
    state: &CampaignState,
    actor: EntityId,
) -> Result<(i32, RollMode), RulesError> {
    let combatant = flow(state)?
        .combatants
        .iter()
        .find(|c| c.actor == actor)
        .ok_or_else(|| invalid("unknown initiative actor"))?;
    preview_initiative_circumstances(state, combatant)
}

/// Read-only setup preview; the authoritative Begin transition repeats these checks.
pub fn preview_initiative_circumstances(
    state: &CampaignState,
    combatant: &TacticalCombatant,
) -> Result<(i32, RollMode), RulesError> {
    let actor = combatant.actor;
    let rules = state.rules.as_ref().ok_or(RulesError::Uninitialized)?;
    let mechanics = rules
        .entities
        .get(&actor)
        .ok_or_else(|| invalid("missing initiative mechanics"))?;
    let modifier = match &combatant.source {
        TacticalSource::Character => crate::test_modifier(mechanics, &TestKind::Initiative),
        TacticalSource::Creature { definition_id } => {
            i32::from(
                definitions()?
                    .creature(definition_id)
                    .ok_or_else(|| invalid("unknown creature definition"))?
                    .statistics
                    .initiative_modifier,
            ) - i32::from(mechanics.exhaustion) * 2
        }
    };
    let conditions = crate::active_conditions(rules, actor);
    let advantage = conditions.contains(&Condition::Invisible);
    let mut frightened_in_sight = false;
    for effect in crate::tactical_effect_adapter::condition_effects(rules)
        .filter(|e| e.target == actor && e.condition == Some(Condition::Frightened))
    {
        if encounter(state)?.participant(effect.source).is_some() {
            frightened_in_sight |=
                crate::spatial::perceive(encounter(state)?, state, actor, effect.source)
                    .map_err(|e| RulesError::Invalid(e.to_string()))?
                    .sees;
        }
    }
    let disadvantage = combatant.surprised
        || frightened_in_sight
        || conditions.contains(&Condition::Incapacitated)
        || conditions.contains(&Condition::Poisoned);
    let mode = match (advantage, disadvantage) {
        (true, false) => RollMode::Advantage,
        (false, true) => RollMode::Disadvantage,
        _ => RollMode::Normal,
    };
    Ok((modifier, mode))
}

pub(super) fn submit_with_inspiration(
    state: &mut CampaignState,
    meta: &CommandMeta,
    original: &RollResult,
    die_index: usize,
    replacement: DieResult,
) -> Result<(), RulesError> {
    let pending = state
        .rules
        .as_ref()
        .and_then(|r| r.pending.as_ref())
        .ok_or(RulesError::NoPending)?;
    validate_tactical_pending(state, pending)?;
    pending.request.resolve(original)?;
    let actor = pending
        .request
        .roller
        .ok_or_else(|| invalid("missing initiative roller"))?;
    authorize(state, meta, actor)?;
    let die = original
        .dice
        .get(die_index)
        .ok_or_else(|| invalid("reroll die index out of range"))?;
    if replacement.sides != die.sides
        || replacement.value == 0
        || replacement.value > replacement.sides
    {
        return Err(invalid("invalid replacement die"));
    }
    let mechanics = state
        .rules
        .as_mut()
        .and_then(|r| r.entities.get_mut(&actor))
        .ok_or(RulesError::Uninitialized)?;
    if !mechanics.heroic_inspiration {
        return Err(prerequisite("Heroic Inspiration unavailable"));
    }
    mechanics.heroic_inspiration = false;
    let mut result = original.clone();
    result.dice[die_index] = replacement;
    submit(state, meta, &result)?;
    state
        .rules
        .as_mut()
        .and_then(|r| r.rolls.last_mut())
        .ok_or_else(|| invalid("missing accepted initiative reroll"))?
        .original_result = Some(original.clone());
    Ok(())
}

pub(super) fn issue(state: &mut CampaignState, meta: &CommandMeta) -> Result<(), RulesError> {
    let TacticalPhase::Initiative { next_group } = flow(state)?.phase else {
        return Err(invalid("initiative request outside initiative phase"));
    };
    let request = request(state, next_group)?;
    let encounter = encounter(state)?.id;
    let rules = state.rules.as_mut().ok_or(RulesError::Uninitialized)?;
    if rules.pending.is_some()
        || rules.rolls.iter().any(|r| r.request.id == request.id)
        || rules.cancelled_roll_ids.contains(&request.id)
    {
        return Err(invalid("initiative request identity already used"));
    }
    rules.pending = Some(PendingRoll {
        issued_by: meta.clone(),
        request,
        purpose: PendingPurpose::TacticalInitiative {
            encounter,
            group_index: next_group,
        },
        ruling: Ruling {
            basis: RulingBasis::Srd { page: 13 },
            reason: "Roll once per identical creature group; surprise applies disadvantage.".into(),
        },
    });
    Ok(())
}

pub(super) fn submit(
    state: &mut CampaignState,
    meta: &CommandMeta,
    result: &RollResult,
) -> Result<(), RulesError> {
    let pending = state
        .rules
        .as_ref()
        .and_then(|r| r.pending.as_ref())
        .ok_or(RulesError::NoPending)?
        .clone();
    validate_tactical_pending(state, &pending)?;
    let actor = pending
        .request
        .roller
        .ok_or_else(|| invalid("initiative roller is missing"))?;
    authorize(state, meta, actor)?;
    if (result.source == RollSource::Digital
        || pending.request.visibility == RollVisibility::Secret)
        && !matches!(meta.issuer, CommandIssuer::Admin | CommandIssuer::System)
    {
        return Err(RulesError::Unauthorized);
    }
    let resolved = pending.request.resolve(result)?;
    let rules = state.rules.as_mut().ok_or(RulesError::Uninitialized)?;
    rules.rolls.push(RecordedRoll {
        issued_by: pending.issued_by,
        accepted_by: meta.clone(),
        original_result: None,
        savage_attacker: None,
        request: pending.request,
        result: result.clone(),
        resolved,
        purpose: pending.purpose,
    });
    rules.pending = None;
    let f = flow_mut(state)?;
    let TacticalPhase::Initiative { next_group } = &mut f.phase else {
        return Err(invalid("not rolling initiative"));
    };
    *next_group += 1;
    if *next_group < f.initiative_groups.len() {
        return issue(state, meta);
    }
    let mut totals = BTreeMap::<i32, Vec<EntityId>>::new();
    for entry in entries(state)? {
        totals.entry(entry.total).or_default().push(entry.actor);
    }
    let ties = totals
        .into_iter()
        .filter(|(_, actors)| actors.len() > 1)
        .map(|(total, actors)| InitiativeTie {
            total,
            actors,
            proposed_order: None,
            accepted_by: vec![],
            host_decided: false,
        })
        .collect();
    flow_mut(state)?.phase = TacticalPhase::InitiativeTies { ties };
    finish_if_agreed(state, meta)
}

pub(super) fn entries(state: &CampaignState) -> Result<Vec<InitiativeEntry>, RulesError> {
    let rules = state.rules.as_ref().ok_or(RulesError::Uninitialized)?;
    let mut entries = Vec::new();
    for group in &flow(state)?.initiative_groups {
        let roll = rules
            .rolls
            .iter()
            .find(|r| r.request.id == group.request_id)
            .ok_or_else(|| invalid("initiative roll history is incomplete"))?;
        for actor in &group.actors {
            entries.push(InitiativeEntry {
                actor: *actor,
                total: roll.resolved.total,
                tie_break: 0,
            });
        }
    }
    Ok(entries)
}

fn player_only(state: &CampaignState, tie: &InitiativeTie) -> bool {
    tie.actors.iter().all(|id| {
        controller(state, *id).is_some()
            && flow(state).is_ok_and(|f| {
                f.combatants
                    .iter()
                    .any(|c| c.actor == *id && c.source == TacticalSource::Character)
            })
    })
}
fn approved(state: &CampaignState, tie: &InitiativeTie) -> bool {
    tie.proposed_order.is_some()
        && if player_only(state, tie) {
            tie.actors
                .iter()
                .all(|id| controller(state, *id).is_some_and(|p| tie.accepted_by.contains(&p)))
        } else {
            tie.host_decided
        }
}
pub(super) fn propose_tie(
    state: &mut CampaignState,
    meta: &CommandMeta,
    order: &[EntityId],
) -> Result<(), RulesError> {
    let TacticalPhase::InitiativeTies { ties } = &flow(state)?.phase else {
        return Err(prerequisite("no initiative ties await decisions"));
    };
    let index = ties
        .iter()
        .position(|t| {
            order.len() == t.actors.len()
                && t.actors
                    .iter()
                    .all(|id| order.iter().filter(|x| *x == id).count() == 1)
        })
        .ok_or_else(|| invalid("order must name every actor in exactly one tie"))?;
    let tie = &ties[index];
    let players = player_only(state, tie);
    if players {
        let CommandIssuer::Player(player) = meta.issuer else {
            return Err(RulesError::Unauthorized);
        };
        if !tie.actors.iter().any(|id| {
            controller(state, *id) == Some(player) && meta.actor == Some(AgentRef::Entity(*id))
        }) {
            return Err(RulesError::Unauthorized);
        }
    } else {
        privileged(meta)?;
    }
    let TacticalPhase::InitiativeTies { ties } = &mut flow_mut(state)?.phase else {
        unreachable!()
    };
    ties[index].proposed_order = Some(order.to_vec());
    ties[index].accepted_by = if let CommandIssuer::Player(player) = meta.issuer {
        vec![player]
    } else {
        vec![]
    };
    ties[index].host_decided = !players;
    finish_if_agreed(state, meta)
}
pub(super) fn accept_tie(
    state: &mut CampaignState,
    meta: &CommandMeta,
    total: i32,
) -> Result<(), RulesError> {
    let CommandIssuer::Player(player) = meta.issuer else {
        return Err(RulesError::Unauthorized);
    };
    let TacticalPhase::InitiativeTies { ties } = &flow(state)?.phase else {
        return Err(prerequisite("no initiative ties await decisions"));
    };
    let index = ties
        .iter()
        .position(|t| t.total == total)
        .ok_or_else(|| invalid("unknown initiative tie"))?;
    let tie = &ties[index];
    if !player_only(state, tie)
        || tie.proposed_order.is_none()
        || tie.accepted_by.contains(&player)
        || !tie.actors.iter().any(|id| {
            controller(state, *id) == Some(player) && meta.actor == Some(AgentRef::Entity(*id))
        })
    {
        return Err(RulesError::Unauthorized);
    }
    let TacticalPhase::InitiativeTies { ties } = &mut flow_mut(state)?.phase else {
        unreachable!()
    };
    ties[index].accepted_by.push(player);
    finish_if_agreed(state, meta)
}
fn finish_if_agreed(state: &mut CampaignState, meta: &CommandMeta) -> Result<(), RulesError> {
    let TacticalPhase::InitiativeTies { ties } = &flow(state)?.phase else {
        return Err(invalid("initiative tie phase is missing"));
    };
    if ties.iter().any(|t| !approved(state, t)) {
        return Ok(());
    }
    let mut order = entries(state)?;
    for entry in &mut order {
        if let Some(tie) = ties.iter().find(|t| t.total == entry.total) {
            entry.tie_break = tie
                .proposed_order
                .as_ref()
                .and_then(|o| o.iter().position(|a| *a == entry.actor))
                .ok_or_else(|| invalid("accepted tie omits a combatant"))?
                as u16;
        }
    }
    order.sort_by(|a, b| b.total.cmp(&a.total).then(a.tie_break.cmp(&b.tie_break)));
    let accepted_ties = ties.clone();
    state
        .rules
        .as_mut()
        .ok_or(RulesError::Uninitialized)?
        .timing = Some(CombatTiming {
        order,
        index: 0,
        round: 1,
        turn_number: 1,
        action_spent: false,
        bonus_action_spent: false,
        slot_spent_this_turn: false,
        reactions_spent: vec![],
    });
    flow_mut(state)?.initiative_decisions = accepted_ties;
    flow_mut(state)?.phase = TacticalPhase::Active;
    super::turns::begin_boundary(state, meta, TurnBoundary::Start)
}
