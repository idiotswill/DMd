use super::*;

pub fn validate_tactical_creatures(
    state: &CampaignState,
    current: &TacticalCreatures,
) -> Result<(), CreatureError> {
    current.validate(state).map_err(invalid)?;
    let rules = rules(state)?;
    let mut roll_ids = HashSet::new();
    for profile in &current.profiles {
        let entity = rules
            .entities
            .get(&profile.actor)
            .ok_or_else(|| invalid("profile lacks creature mechanics"))?;
        validate_creature_profile(state, profile, entity)?;
        let rt = current
            .runtime(profile.actor)
            .ok_or_else(|| invalid("profile lacks runtime"))?;
        let source = source_for_profile(profile)?;
        privileged(&rt.control_origin)?;
        privileged(&rt.lair_origin)?;
        let expected = profile::initial_recharge(source);
        if expected.len() != rt.recharge.len()
            || expected
                .iter()
                .zip(&rt.recharge)
                .any(|(a, b)| a.feature_id != b.feature_id)
        {
            return Err(invalid("creature recharge table differs from source"));
        }
        let expected = profile::initial_limited_uses(source);
        if expected.len() != rt.limited_uses.len()
            || expected
                .iter()
                .zip(&rt.limited_uses)
                .any(|(a, b)| a.feature_id != b.feature_id || a.spell_id != b.spell_id)
        {
            return Err(invalid("creature limited-use table differs from source"));
        }
        for row in &rt.limited_uses {
            if row.spent > long_rest_limit(source, row)? {
                return Err(invalid("creature use counter exceeds source"));
            }
        }
        let mut used = HashSet::new();
        for id in &rt.used_this_own_turn {
            if !used.insert(id)
                || feature(source, id)?.usage != Some(FeatureUsage::OnceUntilOwnTurnStart)
            {
                return Err(invalid("unknown/duplicate per-own-turn source usage"));
            }
        }
        if rt.legendary_spent > legendary_max(source, true)
            || rt.legendary_resistance_spent > resistance_max(source, true)
            || rt.legendary_resistance_rolls.len() != usize::from(rt.legendary_resistance_spent)
            || rt.legendary_resistance_rolls.iter().any(|id| id.0.is_nil())
            || rt
                .legendary_resistance_rolls
                .iter()
                .collect::<HashSet<_>>()
                .len()
                != rt.legendary_resistance_rolls.len()
        {
            return Err(invalid("invalid legendary source budget"));
        }
        if rt.legendary_window_spent
            && !rt
                .observed_turn
                .is_some_and(|turn| turn.actor != rt.actor && turn.boundary == TurnBoundary::End)
        {
            return Err(invalid("legendary use has no other-turn end window"));
        }
        for row in &rt.recharge {
            for ticket in row
                .pending
                .iter()
                .chain(row.last_roll.iter().map(|r| &r.ticket))
            {
                validate_creature_origin(state, &ticket.origin, rt.actor).map_err(invalid)?;
                if ticket.turn.actor != rt.actor
                    || ticket.turn.boundary != TurnBoundary::Start
                    || ticket.request
                        != creature_recharge_request(
                            rt.actor,
                            source,
                            &row.feature_id,
                            ticket.request.id,
                        )?
                    || !roll_ids.insert(ticket.request.id)
                {
                    return Err(invalid("invalid/duplicate retained recharge request"));
                }
            }
            if let Some(ticket) = &row.pending
                && (row.available || rt.observed_turn != Some(ticket.turn))
            {
                return Err(invalid("pending recharge not tied to spent own-start hook"));
            }
            if let Some(record) = &row.last_roll {
                privileged(&record.accepted_by)?;
                if record
                    .accepted_by
                    .actor
                    .is_some_and(|a| a != AgentRef::Entity(rt.actor))
                    || record.accepted_by.expected_event_sequence
                        < record.ticket.origin.expected_event_sequence
                {
                    return Err(invalid("recharge acceptance provenance differs"));
                }
                record
                    .ticket
                    .request
                    .resolve(&record.result)
                    .map_err(|e| invalid(e.to_string()))?;
                if row
                    .pending
                    .as_ref()
                    .is_some_and(|p| p.turn == record.ticket.turn)
                {
                    return Err(invalid("same own start recharges twice"));
                }
            }
        }
        if let Some(routine) = &rt.routine {
            authorize(state, rt, &routine.origin)?;
            if routine.turn.actor != rt.actor
                || routine.turn.boundary != TurnBoundary::Start
                || rt.observed_turn != Some(routine.turn)
            {
                return Err(invalid("routine has wrong source turn"));
            }
            validate_steps(source, &routine.feature_id, &routine.steps)?;
            validate_current_turn(state, routine.turn)?;
            if !rules
                .timing
                .as_ref()
                .is_some_and(|timing| timing.action_spent)
            {
                return Err(invalid("source routine has no paid central action"));
            }
        }
        if let Some(rest) = &rt.last_rest {
            privileged(&rest.origin)?;
        }
    }
    Ok(())
}
