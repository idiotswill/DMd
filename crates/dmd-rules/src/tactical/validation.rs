use super::*;
use std::collections::{HashMap, HashSet};

pub(super) fn validate_groups(state: &CampaignState) -> Result<(), RulesError> {
    let encounter = encounter(state)?;
    let f = flow(state)?;
    if TacticalExecutionVersion::from_flow_version(f.version).is_none()
        || f.origin.campaign_id != state.campaign_id()
        || f.origin.expected_event_sequence > state.applied_event_sequence
        || f.combatants.is_empty()
        || f.combatants.len() != encounter.participants.len()
        || f.initiative_groups.is_empty()
        || f.initiative_groups.len() > f.combatants.len()
    {
        return Err(invalid("invalid bounded encounter execution state"));
    }
    privileged(&f.origin)?;
    let definitions = definitions()?;
    let rules = state.rules.as_ref().ok_or(RulesError::Uninitialized)?;
    let mut actors = HashSet::new();
    for combatant in &f.combatants {
        if !actors.insert(combatant.actor) || encounter.participant(combatant.actor).is_none() {
            return Err(invalid("encounter sources do not partition participants"));
        }
        let e = rules
            .entities
            .get(&combatant.actor)
            .ok_or_else(|| invalid("combatant lacks mechanics"))?;
        match &combatant.source {
            TacticalSource::Character => {
                if !state.characters.values().any(|c| {
                    c.entity_id == combatant.actor
                        && (c.status == CharacterStatus::Active
                            || (c.status == CharacterStatus::Dead && e.death.dead))
                }) {
                    return Err(invalid("character source requires an active character"));
                }
            }
            TacticalSource::Creature { definition_id } => {
                let source = definitions
                    .creature(definition_id)
                    .ok_or_else(|| invalid("unknown source creature"))?;
                if let Some(profile) = rules
                    .tactical_creatures
                    .as_ref()
                    .and_then(|c| c.profile(combatant.actor))
                {
                    if profile.source.definition_id != *definition_id {
                        return Err(invalid("initiative source differs from creature profile"));
                    }
                    crate::tactical_creatures::validate_creature_profile(state, profile, e)
                        .map_err(|error| invalid(&error.to_string()))?;
                    continue;
                }
                if state
                    .characters
                    .values()
                    .any(|c| c.entity_id == combatant.actor)
                    || e.character_features.is_some()
                    || e.ability_scores != source.statistics.ability_scores
                    || e.max_hp != source.statistics.hit_points
                    || e.armor != ArmorClass::Fixed(u16::from(source.statistics.armor_class))
                {
                    return Err(invalid(
                        "creature mechanics disagree with its pinned source",
                    ));
                }
            }
        }
    }
    let mut grouped = HashSet::new();
    let mut requests = HashSet::new();
    for group in &f.initiative_groups {
        if group.actors.is_empty()
            || !requests.insert(group.request_id)
            || rules.cancelled_roll_ids.contains(&group.request_id)
        {
            return Err(invalid("empty or duplicate initiative group/request"));
        }
        let mut first: Option<&TacticalCombatant> = None;
        for actor in &group.actors {
            let c = f
                .combatants
                .iter()
                .find(|c| c.actor == *actor)
                .ok_or_else(|| invalid("unknown grouped actor"))?;
            if !grouped.insert(*actor) {
                return Err(invalid("actor belongs to multiple initiative groups"));
            }
            if let Some(previous) = first {
                if matches!(c.source, TacticalSource::Character)
                    || c.source != previous.source
                    || c.surprised != previous.surprised
                    || (matches!(f.phase, TacticalPhase::Initiative { .. })
                        && (initiative::circumstances(state, *actor)?
                            != initiative::circumstances(state, previous.actor)?))
                {
                    return Err(invalid(
                        "shared initiative requires identical source and initiative circumstances",
                    ));
                }
            } else {
                first = Some(c);
            }
        }
    }
    if grouped != actors {
        return Err(invalid("initiative groups omit encounter participants"));
    }
    // Identical monsters share one initiative roll. Different surprise/condition states
    // require their own group because they have different initiative circumstances.
    for (index, left) in f
        .initiative_groups
        .iter()
        .enumerate()
        .filter(|_| matches!(f.phase, TacticalPhase::Initiative { .. }))
    {
        let l = f
            .combatants
            .iter()
            .find(|c| c.actor == left.actors[0])
            .ok_or_else(|| invalid("unknown group"))?;
        if matches!(l.source, TacticalSource::Character) {
            continue;
        }
        for right in f.initiative_groups.iter().skip(index + 1) {
            let r = f
                .combatants
                .iter()
                .find(|c| c.actor == right.actors[0])
                .ok_or_else(|| invalid("unknown group"))?;
            if l.source == r.source
                && l.surprised == r.surprised
                && initiative::circumstances(state, l.actor)?
                    == initiative::circumstances(state, r.actor)?
            {
                return Err(invalid(
                    "identical creatures must share their initiative roll",
                ));
            }
        }
    }
    Ok(())
}

pub fn validate_tactical_pending(
    state: &CampaignState,
    pending: &PendingRoll,
) -> Result<(), RulesError> {
    if matches!(pending.purpose, PendingPurpose::TacticalResolution { .. }) {
        return super::turn_validation::pending(state, pending);
    }
    let PendingPurpose::TacticalInitiative {
        encounter: id,
        group_index,
    } = pending.purpose
    else {
        return Err(invalid(
            "pending roll is not a tactical initiative continuation",
        ));
    };
    let encounter = encounter(state)?;
    let f = flow(state)?;
    if id != encounter.id
        || f.phase
            != (TacticalPhase::Initiative {
                next_group: group_index,
            })
        || pending.issued_by.campaign_id != state.campaign_id()
        || pending.issued_by.expected_event_sequence > state.applied_event_sequence
        || pending.request != initiative::request(state, group_index)?
    {
        return Err(invalid(
            "pending tactical request differs from its source continuation",
        ));
    }
    Ok(())
}

fn validate_ties(
    state: &CampaignState,
    ties: &[InitiativeTie],
    require_accepted: bool,
) -> Result<(), RulesError> {
    let mut totals: HashMap<i32, HashSet<EntityId>> = HashMap::new();
    for entry in initiative::entries(state)? {
        totals.entry(entry.total).or_default().insert(entry.actor);
    }
    totals.retain(|_, actors| actors.len() > 1);
    if ties.len() != totals.len() {
        return Err(invalid("initiative ties differ from recorded rolls"));
    }
    for tie in ties {
        let actual = totals
            .remove(&tie.total)
            .ok_or_else(|| invalid("duplicate or unknown initiative tie"))?;
        if tie.actors.iter().copied().collect::<HashSet<_>>() != actual
            || tie.actors.len() != actual.len()
        {
            return Err(invalid("tie membership differs from initiative results"));
        }
        let controllers: HashSet<_> = tie
            .actors
            .iter()
            .filter_map(|id| controller(state, *id))
            .collect();
        let player_only = tie.actors.iter().all(|id| {
            controller(state, *id).is_some()
                && flow(state).is_ok_and(|f| {
                    f.combatants
                        .iter()
                        .any(|c| c.actor == *id && c.source == TacticalSource::Character)
                })
        });
        let accepted: HashSet<_> = tie.accepted_by.iter().copied().collect();
        if accepted.len() != tie.accepted_by.len()
            || !accepted.is_subset(&controllers)
            || (player_only && tie.host_decided)
            || (!player_only && !accepted.is_empty())
        {
            return Err(invalid("tie decision has invalid controller authority"));
        }
        if let Some(order) = &tie.proposed_order {
            if order.len() != actual.len()
                || order.iter().copied().collect::<HashSet<_>>() != actual
            {
                return Err(invalid("proposed tie order is not a permutation"));
            }
        } else if !accepted.is_empty() || tie.host_decided {
            return Err(invalid("tie approval lacks a proposal"));
        }
        if require_accepted
            && (tie.proposed_order.is_none()
                || (player_only && accepted != controllers)
                || (!player_only && !tie.host_decided))
        {
            return Err(invalid(
                "initiative started without all required tie decisions",
            ));
        }
    }
    Ok(())
}

pub fn validate_tactical_state(state: &CampaignState) -> Result<(), RulesError> {
    let Some(encounter) = &state.encounter else {
        return Ok(());
    };
    encounter
        .validate(state)
        .map_err(|e| RulesError::Invalid(e.to_string()))?;
    crate::spatial::validate_physical_positions(encounter)
        .map_err(|e| RulesError::Invalid(e.to_string()))?;
    let Some(f) = &encounter.flow else {
        return Ok(());
    };
    validate_groups(state)?;
    let rules = state.rules.as_ref().ok_or(RulesError::Uninitialized)?;
    let completed = match &f.phase {
        TacticalPhase::Initiative { next_group } => *next_group,
        _ => f.initiative_groups.len(),
    };
    if completed > f.initiative_groups.len() {
        return Err(invalid("initiative cursor is out of bounds"));
    }
    for (index, group) in f.initiative_groups.iter().enumerate() {
        let roll = rules
            .rolls
            .iter()
            .find(|r| r.request.id == group.request_id);
        if index < completed {
            let roll =
                roll.ok_or_else(|| invalid("completed initiative group has no raw roll history"))?;
            if roll.purpose
                != (PendingPurpose::TacticalInitiative {
                    encounter: encounter.id,
                    group_index: index,
                })
                || roll.request.roller != group.actors.first().copied()
                || roll.request.dice
                    != [DieSpec {
                        count: 1,
                        sides: 20,
                    }]
            {
                return Err(invalid(
                    "initiative history belongs to a different continuation",
                ));
            }
        } else if roll.is_some() {
            return Err(invalid("future initiative group has already been rolled"));
        }
    }
    match &f.phase {
        TacticalPhase::Initiative { next_group } => {
            if *next_group >= f.initiative_groups.len()
                || rules.timing.is_some()
                || !f.initiative_decisions.is_empty()
            {
                return Err(invalid("invalid initiative phase"));
            }
            validate_tactical_pending(
                state,
                rules
                    .pending
                    .as_ref()
                    .ok_or_else(|| invalid("initiative lacks its next roll"))?,
            )?;
        }
        TacticalPhase::InitiativeTies { ties } => {
            if rules.timing.is_some()
                || rules.pending.is_some()
                || !f.initiative_decisions.is_empty()
            {
                return Err(invalid("invalid tie decision phase"));
            }
            validate_ties(state, ties, false)?;
        }
        TacticalPhase::Active => {
            validate_ties(state, &f.initiative_decisions, true)?;
            let timing = rules
                .timing
                .as_ref()
                .ok_or_else(|| invalid("active encounter lacks initiative"))?;
            let entries = initiative::entries(state)?;
            if timing.order.len() != entries.len()
                || timing.order.iter().any(|e| {
                    !entries
                        .iter()
                        .any(|r| r.actor == e.actor && r.total == e.total)
                })
            {
                return Err(invalid("active initiative differs from accepted raw rolls"));
            }
            for pair in timing.order.windows(2) {
                if pair[0].total < pair[1].total {
                    return Err(invalid("initiative is not in descending order"));
                }
                if pair[0].total == pair[1].total {
                    let order = f
                        .initiative_decisions
                        .iter()
                        .find(|t| t.total == pair[0].total)
                        .and_then(|t| t.proposed_order.as_ref())
                        .ok_or_else(|| invalid("active initiative tie lacks accepted order"))?;
                    let left = order.iter().position(|id| *id == pair[0].actor);
                    let right = order.iter().position(|id| *id == pair[1].actor);
                    if left.is_none() || right.is_none() || left >= right {
                        return Err(invalid("active initiative reverses accepted tie order"));
                    }
                }
            }
        }
        TacticalPhase::Finished => {
            if rules.timing.is_some() || rules.pending.is_some() {
                return Err(invalid(
                    "finished encounter retains an outstanding turn/roll",
                ));
            }
        }
    }
    super::aftermath::validate(state)?;
    super::turn_validation::validate(state)
}
