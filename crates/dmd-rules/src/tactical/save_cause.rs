//! Private source evidence for Magic Resistance. No wire flag, roll reason or
//! damage type can grant this circumstance. Original semantic replay remains
//! the authority for retained source installation and its historical numbers.
use super::*;
use crate::tactical_areas::AreaProgram;
use crate::tactical_definitions::{
    EffectDescriptor, EffectDuration, MonsterTrait, RepeatSave, bundled_tactical_definitions,
};
#[cfg(test)]
mod tests;

pub(super) enum SaveCause<'a> {
    Spell(&'a SpellCastPlan),
    Repeated(&'a ScheduledEffectTrigger),
    Area(&'a AreaProgram),
    Concentration,
}

pub(super) fn circumstances(
    state: &CampaignState,
    actor: EntityId,
    ability: Ability,
    cause: SaveCause<'_>,
) -> Result<Circumstances, RulesError> {
    let profile = state
        .rules
        .as_ref()
        .and_then(|rules| rules.tactical_creatures.as_ref())
        .and_then(|creatures| creatures.profile(actor));
    let resistant = profile
        .map(crate::tactical_creatures::source_for_profile)
        .transpose()
        .map_err(|e| invalid(&e.to_string()))?
        .is_some_and(|source| source.traits.contains(&MonsterTrait::MagicResistance));
    // The old source generation had no such trait. Do not add new source
    // restrictions or modes to old requests, including opaque legacy effects.
    if !resistant {
        return Ok(Circumstances::default());
    }
    let magical = match cause {
        SaveCause::Spell(plan) => {
            crate::tactical_spells::validate_spell_plan(plan)?;
            if let Some(caster) = state
                .rules
                .as_ref()
                .and_then(|rules| rules.tactical_creatures.as_ref())
                .and_then(|creatures| creatures.profile(plan.choice.actor))
            {
                crate::tactical_spells::validate_creature_spell_source(
                    caster,
                    &plan.program.source,
                )?;
            }
            if !matches!(plan.program.nodes.as_slice(),
                [SpellProgramNode::SaveCondition { ability: source_ability, .. }]
                if *source_ability == ability)
            {
                return Err(invalid(
                    "saving throw differs from its admitted spell clause",
                ));
            }
            true
        }
        SaveCause::Repeated(ticket) => {
            repeated_spell(state, actor, ability, ticket)?;
            true
        }
        SaveCause::Area(program) => {
            // These exact immutable source clauses are breath, not spells or
            // phenomena labelled magical (SRD185,273,318-319). A new revision
            // or producer requires an explicit source audit, not a default.
            let admitted = ["chimera", "young-red-dragon", "adult-red-dragon"];
            if program.ability() != ability
                || program.feature_id() != "fire-breath"
                || !admitted.contains(&program.source().definition_id.as_str())
                || crate::tactical_creatures::creature_source_pin(
                    crate::tactical_creatures::creature_definition(&program.source().definition_id)
                        .map_err(|e| invalid(&e.to_string()))?,
                )
                .map_err(|e| invalid(&e.to_string()))?
                    != *program.source()
            {
                return Err(invalid("saving throw source category is not supported"));
            }
            false
        }
        SaveCause::Concentration => false,
    };
    Ok(Circumstances {
        advantage: magical,
        ..Circumstances::default()
    })
}

fn repeated_spell(
    state: &CampaignState,
    actor: EntityId,
    ability: Ability,
    ticket: &ScheduledEffectTrigger,
) -> Result<(), RulesError> {
    let attachment = state
        .rules
        .as_ref()
        .and_then(|rules| rules.tactical_effects.as_ref())
        .ok_or_else(|| invalid("repeated save lacks its retained effect"))?;
    let effect = attachment
        .effects
        .iter()
        .find(|effect| effect.id == ticket.effect)
        .ok_or_else(|| invalid("repeated save lacks its retained effect"))?;
    let definitions = bundled_tactical_definitions().map_err(|e| invalid(&e.to_string()))?;
    let spell = definitions
        .spell(&effect.source.definition_id)
        .ok_or_else(|| invalid("repeated save source category is not supported"))?;
    let [
        EffectDescriptor::SaveCondition {
            ability: source_ability,
            condition,
            repeat: Some(RepeatSave::EndOfTargetsTurnEndsOnSuccess),
        },
    ] = spell.effects.as_slice()
    else {
        return Err(invalid("repeated save lacks its canonical spell clause"));
    };
    let EffectTriggerPayload::SavingThrow { dc, .. } = ticket.payload else {
        return Err(invalid("effect ticket is not a saving throw"));
    };
    let canonical = EffectTriggerRule {
        event: EffectTriggerEvent::Turn {
            subject: EffectSubject::Target,
            boundary: TurnBoundary::End,
        },
        frequency: EffectTriggerFrequency::EveryOccurrence,
        payload: EffectTriggerPayload::SavingThrow {
            ability: *source_ability,
            dc,
            on_success: EffectSaveEnd::TargetEffect,
            on_failure: EffectSaveEnd::None,
        },
    };
    let EffectDuration::Seconds { concentration, .. } = spell.duration else {
        return Err(invalid("repeated spell duration is not supported"));
    };
    // The retained casting validator admits at most 128 target occurrences.
    // This proves only local identity shape; original replay proves selection.
    let identities_match = (0..128).any(|target| {
        let at = SpellProgramOccurrence { node: 0, target };
        effect.id
            == spell_program_effect_id(
                effect.source.command.id,
                effect.source.actor,
                effect.source.ordinal,
                at,
                false,
            )
            && matches!(effect.conditions.as_slice(), [view] if view.id == spell_program_effect_id(
                effect.source.command.id, effect.source.actor, effect.source.ordinal, at, true,
            ))
    });
    if *source_ability != ability
        || !identities_match
        || !attachment.pending.contains(ticket)
        || ticket.target != actor
        || ticket.source != effect.source
        || ticket.rule_index != Some(0)
        || ticket.payload != canonical.payload
        || effect.triggers.as_slice() != [canonical]
        || effect.target != TacticalEffectTarget::Creature(actor)
        || !matches!(ticket.cause, EffectObservation::Turn(EffectTurn {
            actor: target, boundary: TurnBoundary::End, ..
        }) if target == actor)
        || !matches!(effect.conditions.as_slice(), [view] if view.condition == *condition)
        || !effect.defenses.is_empty()
        || effect
            .overlap
            .as_ref()
            .is_none_or(|overlap| overlap.key != spell.id || overlap.potency != i32::from(dc))
        || !matches!(effect.expires, TacticalEffectExpiry::AtTime(_))
        || concentration != effect.concentration_group.is_some()
        || effect.concentration_group.is_some_and(|id| {
            id != spell_concentration_id(
                effect.source.command.id,
                effect.source.actor,
                effect.source.ordinal,
            ) || attachment
                .groups
                .iter()
                .find(|group| group.id == id)
                .is_none_or(|group| {
                    group.source != effect.source
                        || group.expires != effect.expires
                        || group.stage != ConcentrationStage::Active
                })
        })
    {
        return Err(invalid(
            "repeated save differs from its retained canonical spell clause",
        ));
    }
    // Do not recompute the DC or duration from today's caster, recheck casting
    // permission, or require a discarded TacticalCasting record. The original
    // application replay authenticates these values and deterministic identities.
    Ok(())
}
