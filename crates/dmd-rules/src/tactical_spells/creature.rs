use super::*;
use crate::tactical_creatures::{CreatureFeaturePlan, source_for_profile};

/// Trusted composition hook for the exact plan just returned by the creature
/// reducer. It is not a public action and accepts no deserializable authorization
/// flag. The enclosing reducer's cost/use and these obligations must commit together.
/// Replay must reproduce that source activation before calling this hook; a retained
/// spell plan alone cannot prove a historical prepaid action or limited-use payment.
///
/// The invocation may be an NPC consequence of another actor's original command.
/// Preserve it verbatim; never replace its issuer with a fictitious System command.
pub fn plan_spell_from_feature(
    state: &CampaignState,
    feature: &CreatureFeaturePlan,
    material: SpellMaterialChoice,
    mode: SpellCastMode,
    occurrence: u16,
) -> Result<SpellCastPlan, RulesError> {
    if matches!(mode, SpellCastMode::Ready { .. })
        && (feature.enclosing_activation.attack_action
            || feature.enclosing_activation.activation != FeatureActivation::Action)
    {
        return Err(unavailable(
            "Ready cannot reuse an Attack, bonus or legendary activation",
        ));
    }
    validate_creature_origin(state, &feature.invocation, feature.actor).map_err(|e| invalid(&e))?;
    validate_creature_origin(state, &feature.enclosing_activation.origin, feature.actor)
        .map_err(|e| invalid(&e))?;
    if feature.invocation.expected_event_sequence != state.applied_event_sequence
        || feature.enclosing_activation.origin.expected_event_sequence
            > feature.invocation.expected_event_sequence
        || (feature.enclosing_activation.origin.expected_event_sequence
            == feature.invocation.expected_event_sequence
            && feature.enclosing_activation.origin != feature.invocation)
        || (feature.enclosing_activation.origin.id == feature.invocation.id
            && feature.enclosing_activation.origin != feature.invocation)
        || feature.selection.simple_action.is_some()
    {
        return Err(invalid(
            "source spell invocation or enclosing origin differs",
        ));
    }
    let profile = state
        .rules
        .as_ref()
        .and_then(|rules| rules.tactical_creatures.as_ref())
        .and_then(|creatures| creatures.profile(feature.actor))
        .ok_or_else(|| unavailable("source caster profile absent"))?;
    let source = source_for_profile(profile).map_err(|e| invalid(&e.to_string()))?;
    let canonical = source
        .features
        .iter()
        .find(|f| f.id == feature.selection.feature_id)
        .ok_or_else(|| invalid("source casting feature absent"))?;
    if feature.source != profile.source || feature.feature != *canonical {
        return Err(invalid(
            "source casting receipt differs from pinned profile",
        ));
    }
    let spell_id = feature
        .selection
        .spell_id
        .clone()
        .ok_or_else(|| invalid("source casting requires a selected spell"))?;
    let choice = SpellCastChoice {
        actor: feature.actor,
        spell_id,
        grant: SpellGrantChoice::CreatureFeature {
            feature_id: canonical.id.clone(),
        },
        resource: SpellResourceChoice::SourceFeature,
        material,
        mode,
    };
    let plan = plan_spell_cast_authorized(state, &feature.invocation, &choice, occurrence)?;
    if plan.program.source.creature_definition_id.as_ref() != Some(&profile.source.definition_id) {
        return Err(invalid(
            "encounter source differs from retained creature profile",
        ));
    }
    Ok(plan)
}
