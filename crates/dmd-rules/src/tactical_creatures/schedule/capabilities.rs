use super::*;

pub(super) fn feature<'a>(
    source: &'a CreatureDefinition,
    id: &str,
) -> Result<&'a NamedMonsterFeature, CreatureError> {
    source
        .features
        .iter()
        .find(|f| f.id == id)
        .ok_or_else(|| invalid("unknown source feature"))
}
pub(super) fn recharge_policy(f: &NamedMonsterFeature) -> Option<Recharge> {
    match (&f.feature, f.usage) {
        (MonsterFeature::SaveArea { recharge, .. }, _) => Some(*recharge),
        (_, Some(FeatureUsage::Recharge(recharge))) => Some(recharge),
        _ => None,
    }
}
pub(super) fn long_rest_limit(
    source: &CreatureDefinition,
    record: &CreatureLimitedUse,
) -> Result<u8, CreatureError> {
    let f = feature(source, &record.feature_id)?;
    if let Some(id) = &record.spell_id {
        if let MonsterFeature::Spellcasting { spells, .. } = &f.feature {
            return spells
                .iter()
                .find(|s| &s.spell_id == id)
                .and_then(|s| s.uses_per_long_rest)
                .ok_or_else(|| invalid("unknown limited spell"));
        }
    } else if let Some(FeatureUsage::PerLongRest { uses }) = f.usage {
        return Ok(uses);
    }
    Err(invalid("unknown limited source capability"))
}
pub(super) fn selection_reference(selection: &CreatureFeatureSelection) -> MonsterFeatureReference {
    MonsterFeatureReference {
        feature_id: selection.feature_id.clone(),
        spell_id: selection.spell_id.clone(),
        simple_action: selection.simple_action.map(|a| match a {
            CreatureSimpleAction::Dash => BasicAction::Dash,
            CreatureSimpleAction::Disengage => BasicAction::Disengage,
            CreatureSimpleAction::Dodge => BasicAction::Dodge,
            CreatureSimpleAction::Hide => BasicAction::Hide,
        }),
    }
}
pub(super) fn validate_selection<'a>(
    source: &'a CreatureDefinition,
    s: &CreatureFeatureSelection,
) -> Result<&'a NamedMonsterFeature, CreatureError> {
    let f = feature(source, &s.feature_id)?;
    match &f.feature {
        MonsterFeature::Spellcasting { spells, .. } => {
            if s.simple_action.is_some()
                || !spells
                    .iter()
                    .any(|spell| Some(&spell.spell_id) == s.spell_id.as_ref())
            {
                return Err(invalid("choose one source innate spell"));
            }
        }
        MonsterFeature::BasicActionChoice { options } => {
            if s.spell_id.is_some()
                || !selection_reference(s)
                    .simple_action
                    .is_some_and(|a| options.contains(&a))
            {
                return Err(invalid("choose one source basic action"));
            }
        }
        _ => {
            if s.spell_id.is_some() || s.simple_action.is_some() {
                return Err(invalid("feature has no spell/basic-action choice"));
            }
        }
    }
    Ok(f)
}
pub(super) fn validate_steps(
    source: &CreatureDefinition,
    id: &str,
    steps: &[CreatureRoutineStep],
) -> Result<(), CreatureError> {
    let f = feature(source, id)?;
    let mut slots = HashSet::new();
    match &f.feature {
        MonsterFeature::Multiattack {
            count,
            attack_options,
        } => {
            if steps.len() != usize::from(*count) {
                return Err(invalid("wrong Multiattack occurrence count"));
            }
            for step in steps {
                if step.slot >= *count
                    || !slots.insert(step.slot)
                    || !attack_options.contains(&step.selection.feature_id)
                    || step.selection.spell_id.is_some()
                    || step.selection.simple_action.is_some()
                {
                    return Err(invalid("invalid Multiattack attack/slot"));
                }
            }
        }
        MonsterFeature::MultiattackRoutine {
            slots: source_slots,
            limits,
        } => {
            if steps.len() != source_slots.len() {
                return Err(invalid("wrong mixed Multiattack occurrence count"));
            }
            for step in steps {
                if !slots.insert(step.slot)
                    || !source_slots
                        .get(usize::from(step.slot))
                        .is_some_and(|slot| {
                            slot.options.contains(&selection_reference(&step.selection))
                        })
                {
                    return Err(invalid("invalid mixed Multiattack slot/substitution"));
                }
            }
            for limit in limits {
                if steps
                    .iter()
                    .filter(|s| selection_reference(&s.selection) == limit.selection)
                    .count()
                    > usize::from(limit.maximum)
                {
                    return Err(invalid("Multiattack shared substitution limit exceeded"));
                }
            }
        }
        _ => return Err(invalid("source feature is not a routine")),
    }
    for step in steps {
        validate_selection(source, &step.selection)?;
    }
    Ok(())
}
pub(in crate::tactical_creatures) fn available(
    source: &CreatureDefinition,
    rt: &CreatureRuntime,
    s: &CreatureFeatureSelection,
) -> Result<(), CreatureError> {
    let f = validate_selection(source, s)?;
    if rt
        .recharge
        .iter()
        .any(|r| r.feature_id == f.id && (!r.available || r.pending.is_some()))
        || rt.used_this_own_turn.contains(&f.id)
    {
        return Err(CreatureError::Unavailable(
            "feature requires recharge/own-turn refresh".into(),
        ));
    }
    for limited in rt
        .limited_uses
        .iter()
        .filter(|r| r.feature_id == f.id && (r.spell_id.is_none() || r.spell_id == s.spell_id))
    {
        if limited.spent >= long_rest_limit(source, limited)? {
            return Err(CreatureError::Unavailable(
                "source per-rest uses exhausted".into(),
            ));
        }
    }
    Ok(())
}
pub(super) fn spend(
    source: &CreatureDefinition,
    rt: &mut CreatureRuntime,
    s: &CreatureFeatureSelection,
) -> Result<(), CreatureError> {
    available(source, rt, s)?;
    let f = validate_selection(source, s)?;
    if let Some(r) = rt.recharge.iter_mut().find(|r| r.feature_id == f.id) {
        r.available = false;
    }
    for limited in rt
        .limited_uses
        .iter_mut()
        .filter(|r| r.feature_id == f.id && (r.spell_id.is_none() || r.spell_id == s.spell_id))
    {
        limited.spent += 1;
    }
    if f.usage == Some(FeatureUsage::OnceUntilOwnTurnStart) {
        rt.used_this_own_turn.push(f.id.clone());
    }
    Ok(())
}
pub(super) fn rules(state: &CampaignState) -> Result<&RulesState, CreatureError> {
    state
        .rules
        .as_ref()
        .ok_or_else(|| invalid("creature execution requires rules state"))
}
pub(in crate::tactical_creatures) fn can_act(
    state: &CampaignState,
    actor: EntityId,
) -> Result<(), CreatureError> {
    let rules = rules(state)?;
    let entity = rules
        .entities
        .get(&actor)
        .ok_or_else(|| invalid("missing creature mechanics"))?;
    if entity.death.dead
        || entity.hp == 0
        || active_conditions(rules, actor).contains(&Condition::Incapacitated)
    {
        return Err(CreatureError::Unavailable(
            "creature cannot take actions".into(),
        ));
    }
    Ok(())
}
pub(super) fn authorize(
    state: &CampaignState,
    rt: &CreatureRuntime,
    meta: &CommandMeta,
) -> Result<(), CreatureError> {
    validate_creature_origin(state, meta, rt.actor).map_err(invalid)?;
    if meta
        .actor
        .is_some_and(|actor| actor != AgentRef::Entity(rt.actor))
    {
        return Err(CreatureError::Unauthorized);
    }
    match meta.issuer {
        CommandIssuer::System | CommandIssuer::Admin => Ok(()),
        CommandIssuer::Player(player)
            if rt.controller == CreatureController::Player(player)
                && meta.actor == Some(AgentRef::Entity(rt.actor)) =>
        {
            Ok(())
        }
        _ => Err(CreatureError::Unauthorized),
    }
}
pub(super) fn privileged(meta: &CommandMeta) -> Result<(), CreatureError> {
    if matches!(meta.issuer, CommandIssuer::System | CommandIssuer::Admin) {
        Ok(())
    } else {
        Err(CreatureError::Unauthorized)
    }
}
pub(super) fn validate_current_turn(
    state: &CampaignState,
    turn: CreatureTurn,
) -> Result<(), CreatureError> {
    let encounter = state
        .encounter
        .as_ref()
        .ok_or_else(|| invalid("no encounter for creature turn"))?;
    let timing = rules(state)?
        .timing
        .as_ref()
        .ok_or_else(|| invalid("no central combat timing"))?;
    if turn.encounter_id != encounter.id
        || turn.number != timing.turn_number
        || timing
            .order
            .get(timing.index)
            .is_none_or(|entry| entry.actor != turn.actor)
        || encounter.participant(turn.actor).is_none()
    {
        return Err(invalid("creature hook does not match central turn"));
    }
    Ok(())
}
pub(super) fn after(old: Option<CreatureTurn>, new: CreatureTurn) -> bool {
    match old {
        None => new.boundary == TurnBoundary::Start,
        Some(old) => {
            old.encounter_id == new.encounter_id
                && ((old.number == new.number
                    && old.actor == new.actor
                    && old.boundary == TurnBoundary::Start
                    && new.boundary == TurnBoundary::End)
                    || (old.number.checked_add(1) == Some(new.number)
                        && old.boundary == TurnBoundary::End
                        && new.boundary == TurnBoundary::Start))
        }
    }
}
pub(super) fn legendary_max(source: &CreatureDefinition, in_lair: bool) -> u8 {
    source
        .legendary_budget
        .as_ref()
        .map_or(0, |b| if in_lair { b.uses_in_lair } else { b.uses })
}
pub(super) fn resistance_max(source: &CreatureDefinition, in_lair: bool) -> u8 {
    source
        .traits
        .iter()
        .find_map(|t| match t {
            MonsterTrait::LegendaryResistance {
                uses_per_long_rest,
                uses_in_lair,
            } => Some(if in_lair {
                *uses_in_lair
            } else {
                *uses_per_long_rest
            }),
            _ => None,
        })
        .unwrap_or(0)
}
