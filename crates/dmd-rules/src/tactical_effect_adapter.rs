//! Attachment of lifecycle authority to ordinary mechanics. These are internal derived
//! operations, never direct player commands; source legality belongs to the resolver.
use crate::{RulesError, active_conditions, tactical_effects::*};
use dmd_domain::*;
use std::collections::HashSet;

/// Unified ephemeral condition views retain the actual charmer/grappler/fear source.
pub fn condition_effects(rules: &RulesState) -> impl Iterator<Item = ActiveEffect> + '_ {
    rules.effects.iter().cloned().chain(
        rules
            .tactical_effects
            .as_ref()
            .into_iter()
            .flat_map(active_effect_views),
    )
}

pub fn validate_effect_attachment(state: &CampaignState) -> Result<(), RulesError> {
    let Some(rules) = &state.rules else {
        return Ok(());
    };
    let Some(effects) = &rules.tactical_effects else {
        return Ok(());
    };
    effects.validate(state).map_err(RulesError::Invalid)?;
    let mut ids: HashSet<_> = rules.effects.iter().map(|effect| effect.id).collect();
    for id in effects
        .groups
        .iter()
        .map(|g| g.id)
        .chain(effects.effects.iter().map(|e| e.id))
        .chain(
            effects
                .effects
                .iter()
                .flat_map(|e| e.conditions.iter().map(|c| c.id)),
        )
    {
        if !ids.insert(id) {
            return Err(RulesError::Invalid(
                "lifecycle and legacy effect identities overlap".into(),
            ));
        }
    }
    for group in &effects.groups {
        let owner = &rules.entities[&group.source.actor];
        if owner.concentration != Some(group.id)
            || owner.death.dead
            || active_conditions(rules, owner.entity_id).contains(&Condition::Incapacitated)
        {
            return Err(RulesError::Invalid(
                "invalid tactical concentration binding".into(),
            ));
        }
    }
    // Suppression must not hide an impossible retained condition: otherwise ending
    // the stronger effect could fail and strand both source instances forever.
    for effect in &effects.effects {
        if let TacticalEffectTarget::Creature(target) = effect.target
            && effect.conditions.iter().any(|condition| {
                rules.entities[&target]
                    .condition_immunities
                    .contains(&condition.condition)
            })
        {
            return Err(RulesError::Invalid(
                "grouped condition conflicts with target immunity".into(),
            ));
        }
    }
    for view in active_effect_views(effects) {
        let target = &rules.entities[&view.target];
        if view.condition == Some(Condition::Unconscious)
            && !target.condition_immunities.contains(&Condition::Prone)
            && !target.prone
        {
            return Err(RulesError::Invalid(
                "invalid tactical condition consequence".into(),
            ));
        }
    }
    Ok(())
}

/// Applies an already source-authorized operation atomically, synchronizing the single
/// concentration pointer. Becoming Incapacitated breaks concentration immediately,
/// including when removing an overlapping effect reveals another incapacitating effect.
pub fn apply_effect_operation(
    state: &CampaignState,
    meta: &CommandMeta,
    action: &EffectLifecycleAction,
) -> Result<(CampaignState, Vec<EndedTacticalEffect>), RulesError> {
    let rules = state.rules.as_ref().ok_or(RulesError::Uninitialized)?;
    let current = rules.tactical_effects.clone().unwrap_or_default();
    let transition = apply_effect_lifecycle(state, &current, meta, action)
        .map_err(|e| RulesError::Invalid(e.to_string()))?;
    let mut next = state.clone();
    let mut ended = transition.ended;
    install(&mut next, &current, transition.next_effects);
    let mut step = action.step;
    loop {
        let rules = next.rules.as_ref().expect("rules retained");
        let owner = rules
            .entities
            .values()
            .filter(|entity| {
                entity.concentration.is_some()
                    && (entity.death.dead
                        || active_conditions(rules, entity.entity_id)
                            .contains(&Condition::Incapacitated))
            })
            .map(|entity| entity.entity_id)
            .min_by_key(|id| id.0);
        let Some(owner) = owner else { break };
        let current = rules
            .tactical_effects
            .clone()
            .expect("attachment installed");
        if current.group_for_owner(owner).is_some() {
            step = step
                .checked_add(1)
                .ok_or_else(|| RulesError::Invalid("effect operation limit".into()))?;
            let transition = apply_effect_lifecycle(
                &next,
                &current,
                meta,
                &EffectLifecycleAction {
                    step,
                    operation: EffectLifecycleOperation::EndConcentration {
                        owner,
                        reason: EffectEndReason::ConcentrationBroken,
                    },
                },
            )
            .map_err(|e| RulesError::Invalid(e.to_string()))?;
            ended.extend(transition.ended);
            install(&mut next, &current, transition.next_effects);
        } else {
            let rules = next.rules.as_mut().expect("rules retained");
            rules
                .effects
                .retain(|effect| effect.concentration_owner != Some(owner));
            rules
                .entities
                .get_mut(&owner)
                .expect("known owner")
                .concentration = None;
        }
    }
    validate_effect_attachment(&next)?;
    Ok((next, ended))
}

fn install(state: &mut CampaignState, previous: &TacticalEffects, next: TacticalEffects) {
    let rules = state.rules.as_mut().expect("validated rules");
    for (owner, id) in previous.concentration_bindings() {
        let entity = rules.entities.get_mut(&owner).expect("validated owner");
        if entity.concentration == Some(id) {
            entity.concentration = None;
        }
    }
    for (owner, id) in next.concentration_bindings() {
        // Starting a new grouped spell also replaces legacy concentration, exactly once.
        rules
            .effects
            .retain(|effect| effect.concentration_owner != Some(owner));
        rules
            .entities
            .get_mut(&owner)
            .expect("validated owner")
            .concentration = Some(id);
    }
    rules.tactical_effects = Some(next);
    let unconscious: Vec<_> = rules
        .tactical_effects
        .as_ref()
        .into_iter()
        .flat_map(active_effect_views)
        .filter(|effect| effect.condition == Some(Condition::Unconscious))
        .filter(|effect| {
            !rules.entities[&effect.target]
                .condition_immunities
                .contains(&Condition::Prone)
        })
        .map(|effect| effect.target)
        .collect();
    for actor in unconscious {
        rules
            .entities
            .get_mut(&actor)
            .expect("validated target")
            .prone = true;
    }
}
