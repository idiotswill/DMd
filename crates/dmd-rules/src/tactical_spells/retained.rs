//! Source reconstruction for a retained cast. Shape checks do not replace the
//! application's accepted-event replay or current queue/reference validation.
use super::*;
use crate::tactical_creatures::CreatureFeaturePlan;
use std::collections::HashSet;

pub fn retain_spell_cast(
    cast: SpellCast,
    bound: &BoundSpell,
    selection: SpellTargetChoice,
    creature: Option<&CreatureFeaturePlan>,
) -> Result<TacticalCasting, RulesError> {
    if bound.plan() != &cast.plan {
        return Err(invalid(
            "retained cast differs from its admitted source plan",
        ));
    }
    let creature_activation = match (&cast.plan.choice.grant, creature) {
        (SpellGrantChoice::Prepared, None) => None,
        (SpellGrantChoice::CreatureFeature { feature_id }, Some(feature))
            if feature.actor == cast.plan.choice.actor
                && feature.invocation == cast.plan.origin
                && feature.selection.feature_id == *feature_id
                && feature.selection.spell_id.as_ref() == Some(&cast.plan.choice.spell_id)
                && feature.source.definition_id
                    == cast
                        .plan
                        .program
                        .source
                        .creature_definition_id
                        .clone()
                        .unwrap_or_default() =>
        {
            Some(SpellCreatureActivation {
                origin: feature.enclosing_activation.origin.clone(),
                activation: match feature.enclosing_activation.activation {
                    FeatureActivation::Action => SpellEnclosingActivation::Action,
                    FeatureActivation::BonusAction => SpellEnclosingActivation::BonusAction,
                    FeatureActivation::Reaction => SpellEnclosingActivation::Reaction,
                    FeatureActivation::Legendary { .. } => SpellEnclosingActivation::Legendary,
                },
                attack_action: feature.enclosing_activation.attack_action,
            })
        }
        _ => return Err(invalid("retained cast lacks its exact source activation")),
    };
    let record = TacticalCasting {
        cast: Box::new(cast),
        selection: Some(selection),
        targets: bound
            .targets()
            .iter()
            .map(|target| SpellBoundTarget {
                actor: target.actor(),
                source_type_matches: target.valid_type(),
            })
            .collect(),
        consumed_material: bound.consumed_material(),
        creature_activation,
        completed: vec![],
    };
    validate_retained_spell(&record)?;
    Ok(record)
}

pub fn validate_retained_spell(record: &TacticalCasting) -> Result<(), RulesError> {
    let cast = &record.cast;
    validate_spell_cast(cast)?;
    executable_spell_kind(&cast.plan)?;
    if record.targets.len() > 128 || record.completed.len() > 128 {
        return Err(invalid("retained spell target capacity exceeded"));
    }
    match (&cast.plan.choice.grant, &record.creature_activation) {
        (SpellGrantChoice::Prepared, None) => (),
        (SpellGrantChoice::CreatureFeature { .. }, Some(activation)) => {
            let original = &cast.plan.origin;
            if activation.origin.id.0.is_nil()
                || activation.origin.campaign_id != original.campaign_id
                || activation.origin.expected_event_sequence > original.expected_event_sequence
                || (activation.origin.id == original.id && activation.origin != *original)
                || (activation.origin.expected_event_sequence == original.expected_event_sequence
                    && activation.origin != *original)
                || matches!(cast.plan.choice.mode, SpellCastMode::Ready { .. })
                    && (activation.attack_action
                        || activation.activation != SpellEnclosingActivation::Action)
            {
                return Err(invalid("invalid enclosing spell activation provenance"));
            }
        }
        _ => return Err(invalid("retained spell activation ownership differs")),
    }
    let source = definitions()?;
    let source = source
        .spell(&cast.plan.choice.spell_id)
        .ok_or_else(|| invalid("retained spell source is absent"))?;
    let material_consumed = source
        .components
        .material
        .as_ref()
        .is_some_and(|m| m.consumed)
        && cast.plan.components.material;
    let expected_material = if material_consumed {
        match cast.plan.choice.material {
            SpellMaterialChoice::Material { item } => Some(item),
            _ => {
                return Err(invalid(
                    "consumed material lacks an exact physical identity",
                ));
            }
        }
    } else {
        None
    };
    if record.consumed_material != expected_material {
        return Err(invalid("retained consumed material differs from source"));
    }
    let Some(SpellTargetChoice::Entities(actors)) = &record.selection else {
        // The first executor admits immediate creature programs. Ready's genuine
        // release binding is added with its authoritative trigger integration.
        return Err(unavailable(
            "retained spell lacks its executable creature selection",
        ));
    };
    if actors.is_empty()
        || actors.len() != record.targets.len()
        || actors
            .iter()
            .zip(&record.targets)
            .any(|(id, bound)| id.0.is_nil() || *id != bound.actor)
    {
        return Err(invalid("retained spell target order or identity differs"));
    }
    let (min, max, repeats, restricted_type) = match &cast.plan.program.targets {
        SpellTargetRule::Caster => (1, 1, false, false),
        SpellTargetRule::CreatureOrObject => (1, 1, false, false),
        SpellTargetRule::Creatures {
            maximum,
            creature_type,
            ..
        } => (1, usize::from(*maximum), false, creature_type.is_some()),
        SpellTargetRule::Darts { count, .. } | SpellTargetRule::Rays { count } => {
            (usize::from(*count), usize::from(*count), true, false)
        }
        _ => {
            return Err(unavailable(
                "retained target shape has no complete executor",
            ));
        }
    };
    if actors.len() < min
        || actors.len() > max
        || (!repeats && actors.iter().collect::<HashSet<_>>().len() != actors.len())
        || (!restricted_type
            && record
                .targets
                .iter()
                .any(|target| !target.source_type_matches))
    {
        return Err(invalid(
            "retained target facts disagree with source selector",
        ));
    }
    if matches!(cast.plan.program.targets, SpellTargetRule::Caster)
        && actors.as_slice() != [cast.plan.choice.actor]
    {
        return Err(invalid("retained self spell has a foreign target"));
    }
    if matches!(
        cast.plan.program.nodes.as_slice(),
        [SpellProgramNode::BaseArmorClass { .. }]
    ) && actors.as_slice() != [cast.plan.choice.actor]
    {
        return Err(invalid(
            "retained armor formula exceeds its admitted self-target path",
        ));
    }
    let mut completed = HashSet::new();
    if record.completed.iter().any(|occurrence| {
        occurrence.node != 0
            || usize::from(occurrence.target) >= actors.len()
            || !completed.insert(*occurrence)
    }) || (!record.completed.is_empty()
        && !matches!(
            cast.phase,
            SpellCastPhase::Committed | SpellCastPhase::Released
        ))
    {
        return Err(invalid("invalid or duplicate completed spell occurrence"));
    }
    Ok(())
}

/// Reconstruct a sealed proof only from a source-checked retained record. Historical
/// position/material access is not re-decided after a legitimate interrupt or payment.
pub fn retained_spell_binding(record: &TacticalCasting) -> Result<BoundSpell, RulesError> {
    validate_retained_spell(record)?;
    Ok(BoundSpell::from_retained(
        record,
        executable_spell_kind(&record.cast.plan)?,
    ))
}
