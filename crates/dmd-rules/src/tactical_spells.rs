//! Internal source-derived casting primitives. The tactical scheduler applies every
//! returned obligation in the same atomic command as the returned cast record. None
//! of these operations is a public command or a second resource/turn authority.
mod binding;
mod creature;
mod execution;
mod program;
mod reservations;
mod retained;
#[cfg(test)]
mod tests;

use crate::{RulesError, ability_modifier, proficiency_bonus, tactical_definitions::*};
use dmd_domain::*;
use serde::Serialize;

pub use binding::*;
pub use creature::plan_spell_from_feature;
pub use execution::{
    spell_amount_operation, spell_amount_request, spell_condition_effect, spell_effect_expiry,
};
pub use program::compile_spell_program;
pub use reservations::validate_spell_slot_reservation;
pub use retained::{retain_spell_cast, retained_spell_binding, validate_retained_spell};

/// This is an internal fact from a source-backed material registry. It intentionally
/// cannot be deserialized as a public player request. Physical identity/custody are
/// independently checked below; display names never establish a component match.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpellMaterialFact {
    Specified {
        item: ItemId,
        spell_id: String,
        value_cp: u32,
    },
    Focus {
        item: ItemId,
    },
    ComponentPouch {
        item: ItemId,
    },
}

/// Validate physical components against the current loadout. `verbal_possible` is an
/// internal current sensory/anatomical fact (e.g. Silence), not a public permission.
/// A component registry supplies material facts; no item name or client cost is used.
pub fn validate_spell_components(
    state: &CampaignState,
    plan: &SpellCastPlan,
    verbal_possible: bool,
    material_fact: Option<&SpellMaterialFact>,
) -> Result<Option<ItemId>, RulesError> {
    validate_spell_plan(plan)?;
    let defs = definitions()?;
    let spell = defs
        .spell(&plan.choice.spell_id)
        .ok_or_else(|| invalid("spell source absent"))?;
    let rules = state.rules.as_ref().ok_or(RulesError::Uninitialized)?;
    let entity = rules
        .entities
        .get(&plan.choice.actor)
        .ok_or_else(|| invalid("caster absent"))?;
    if plan.components.verbal
        && (!verbal_possible || entity.spellcasting.as_ref().is_some_and(|c| !c.can_speak))
    {
        return Err(unavailable("verbal component is not possible"));
    }
    let loadout = rules
        .tactical_inventory
        .as_ref()
        .and_then(|i| i.loadout(plan.choice.actor));
    let free_hand = loadout.map_or_else(
        || entity.spellcasting.as_ref().is_some_and(|c| c.free_hand),
        |l| l.hands.hands.contains(&HandAssignment::Free),
    );
    if !plan.components.material {
        if material_fact.is_some() || plan.choice.material != SpellMaterialChoice::None {
            return Err(invalid("unneeded material evidence"));
        }
        if plan.components.somatic && !free_hand {
            return Err(unavailable("somatic component needs a free hand"));
        }
        return Ok(None);
    }
    let requirement = spell
        .components
        .material
        .as_ref()
        .ok_or_else(|| invalid("material requirement absent"))?;
    let fact =
        material_fact.ok_or_else(|| unavailable("source-backed material access is required"))?;
    let (item_id, substitutes) = match (plan.choice.material, fact) {
        (
            SpellMaterialChoice::Material { item },
            SpellMaterialFact::Specified {
                item: proof,
                spell_id,
                value_cp,
            },
        ) if item == *proof
            && *spell_id == spell.id
            && *value_cp >= requirement.minimum_cost_cp =>
        {
            (item, false)
        }
        (SpellMaterialChoice::Focus { item }, SpellMaterialFact::Focus { item: proof })
            if item == *proof =>
        {
            if !entity
                .spellcasting
                .as_ref()
                .is_some_and(|c| c.material_focus)
            {
                return Err(unavailable(
                    "source grant does not authorize this spellcasting focus",
                ));
            }
            (item, true)
        }
        (
            SpellMaterialChoice::ComponentPouch { item },
            SpellMaterialFact::ComponentPouch { item: proof },
        ) if item == *proof => (item, true),
        _ => {
            return Err(unavailable(
                "material selection does not match source evidence",
            ));
        }
    };
    if substitutes && (requirement.minimum_cost_cp > 0 || requirement.consumed) {
        return Err(unavailable(
            "specified costly or consumed component cannot be substituted",
        ));
    }
    let item = state
        .items
        .get(&item_id)
        .ok_or_else(|| unavailable("material item absent"))?;
    if item.campaign_id != state.campaign_id()
        || item.id != item_id
        || item.quantity == 0
        || item.custody != Custody::Entity(plan.choice.actor)
        || item.state != ItemState::Intact
    {
        return Err(unavailable(
            "material item is not intact and carried by caster",
        ));
    }
    let held = loadout.is_some_and(|l| l.hands.hands.contains(&HandAssignment::Item(item_id)));
    // SRD105: a hand holding/accessing M may also perform this spell's S component.
    let held_material_hand = held
        && !matches!(
            plan.choice.material,
            SpellMaterialChoice::ComponentPouch { .. }
        );
    if !free_hand && !held_material_hand {
        return Err(unavailable("material component needs an accessible hand"));
    }
    if matches!(plan.choice.material, SpellMaterialChoice::Focus { .. }) && !held {
        return Err(unavailable("the spellcasting focus must be held"));
    }
    Ok(requirement.consumed.then_some(item_id))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpellCastTransition {
    pub cast: SpellCast,
    pub obligations: Vec<SpellCastObligation>,
}

/// Source-selected consequences, not a general state-patch vocabulary. The root
/// consumes them through existing budget, inventory, lifecycle and work reducers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpellCastObligation {
    RequireCreatureFeatureReceipt {
        actor: EntityId,
        feature_id: String,
        spell_id: String,
        origin: CommandMeta,
    },
    SpendCastingCost {
        actor: EntityId,
        cost: SpellCastingCost,
    },
    BeginConcentration {
        group: ConcentrationGroup,
    },
    CommitExpenditure {
        actor: EntityId,
        expenditure: SpellExpenditure,
    },
    ConsumeMaterial {
        actor: EntityId,
        item: ItemId,
    },
    ActivateConcentration {
        group: EffectId,
        duration: SpellDuration,
    },
    EndConcentration {
        actor: EntityId,
        group: EffectId,
    },
    QueueProgram {
        origin: CommandMeta,
        occurrence: u16,
    },
}

/// Facts come from the tactical scheduler's current frame and source component
/// resolver, never from player JSON. No serde implementation is intentional.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpellCastContext {
    pub now: WorldInstant,
    pub turn_number: u64,
    pub current_actor: EntityId,
    /// Current source can_act result, including death/incapacitation. Never player input.
    pub can_act: bool,
    pub concentration: Option<EffectId>,
}

/// Only the scheduler may advance an interrupt window after resolving its exact
/// trigger/save. A UI cannot claim a counterspell or a perceived Ready trigger.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpellCastAdvance {
    Commit,
    Countered,
    Interrupt(SpellCastingInterruption),
    ReleaseReady,
    ExpireHeld,
}

fn invalid(message: &str) -> RulesError {
    RulesError::Invalid(message.into())
}
fn unavailable(message: &str) -> RulesError {
    RulesError::Prerequisite(message.into())
}

/// Apply one already source-authorized cost on a clone. The scheduler composes this
/// with the cast record and other obligations before its single durable commit.
pub fn apply_spell_casting_cost(
    state: &CampaignState,
    actor: EntityId,
    cost: SpellCastingCost,
) -> Result<CampaignState, RulesError> {
    let mut next = state.clone();
    let rules = next.rules.as_mut().ok_or(RulesError::Uninitialized)?;
    crate::tactical_budget::spend_cost(
        rules,
        actor,
        match cost {
            SpellCastingCost::Action => crate::tactical_budget::TacticalCost::Action,
            SpellCastingCost::BonusAction => crate::tactical_budget::TacticalCost::BonusAction,
            SpellCastingCost::Reaction => crate::tactical_budget::TacticalCost::Reaction,
        },
    )?;
    Ok(next)
}

/// Prepared-slot authority stays in MechanicalEntity and the shared turn budget.
/// CreatureUse is deliberately rejected here: the source creature reducer owns it.
pub fn apply_spell_expenditure(
    state: &CampaignState,
    actor: EntityId,
    expenditure: &SpellExpenditure,
) -> Result<CampaignState, RulesError> {
    if !state
        .rules
        .as_ref()
        .is_some_and(|rules| rules.entities.contains_key(&actor))
    {
        return Err(unavailable("caster mechanics absent"));
    }
    let mut next = state.clone();
    let level = match *expenditure {
        SpellExpenditure::None => return Ok(next),
        SpellExpenditure::Slot { level } if (1..=9).contains(&level) => level,
        SpellExpenditure::CreatureUse { .. } => {
            return Err(unavailable("source creature reducer must commit this use"));
        }
        _ => return Err(invalid("invalid spell expenditure")),
    };
    let rules = next.rules.as_mut().ok_or(RulesError::Uninitialized)?;
    let casting = rules
        .entities
        .get_mut(&actor)
        .and_then(|e| e.spellcasting.as_mut())
        .ok_or_else(|| unavailable("caster has no spell slots"))?;
    let slot = &mut casting.slots[usize::from(level - 1)];
    *slot = slot
        .checked_sub(1)
        .ok_or_else(|| unavailable("selected slot is exhausted"))?;
    let timing = rules
        .timing
        .as_mut()
        .ok_or_else(|| unavailable("spell slot requires an active tactical turn"))?;
    let budget = &mut next
        .encounter
        .as_mut()
        .and_then(|e| e.flow.as_mut())
        .ok_or_else(|| unavailable("spell slot lacks tactical flow"))?
        .budget;
    crate::tactical_budget::spend_slot_turn(timing, budget, actor)?;
    Ok(next)
}

fn definitions() -> Result<TacticalDefinitions, RulesError> {
    TacticalDefinitions::from_json(TACTICAL_DEFINITIONS_JSON)
        .map_err(|e| RulesError::Incompatible(e.to_string()))
}

fn fingerprint(value: &impl Serialize) -> Result<String, RulesError> {
    let bytes = serde_json::to_vec(value).map_err(|_| invalid("source serialization failed"))?;
    let hash = bytes
        .into_iter()
        .fold(0xcbf29ce484222325_u64, |hash, byte| {
            (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3)
        });
    Ok(format!("fnv1a64:{hash:016x}"))
}

fn source_pin(
    defs: &TacticalDefinitions,
    spell: &TacticalSpellDefinition,
    creature: Option<&CreatureDefinition>,
    feature: Option<&NamedMonsterFeature>,
) -> Result<SpellSourcePin, RulesError> {
    Ok(SpellSourcePin {
        ruleset_id: defs.ruleset_id.clone(),
        ruleset_version: defs.ruleset_version.clone(),
        spell_id: spell.id.clone(),
        fingerprint: fingerprint(&(spell, creature, feature))?,
        creature_definition_id: creature.map(|c| c.id.clone()),
        feature_id: feature.map(|f| f.id.clone()),
    })
}

fn source_components(
    defs: &TacticalDefinitions,
    spell: &TacticalSpellDefinition,
    pin: &SpellSourcePin,
) -> SpellComponentsNeeded {
    let waivers = pin
        .creature_definition_id
        .as_ref()
        .and_then(|id| defs.creature(id))
        .and_then(|creature| {
            creature
                .features
                .iter()
                .find(|f| Some(&f.id) == pin.feature_id.as_ref())
        })
        .and_then(|feature| feature.spell_component_waivers);
    SpellComponentsNeeded {
        verbal: spell.components.verbal && !waivers.is_some_and(|w| w.verbal),
        somatic: spell.components.somatic && !waivers.is_some_and(|w| w.somatic),
        material: spell.components.material.is_some() && !waivers.is_some_and(|w| w.material),
    }
}

fn authorize(state: &CampaignState, meta: &CommandMeta, actor: EntityId) -> Result<(), RulesError> {
    if meta.id.0.is_nil()
        || meta.campaign_id != state.campaign_id()
        || meta.expected_event_sequence != state.applied_event_sequence
        || !state.entities.contains_key(&actor)
    {
        return Err(invalid("spell command has stale or foreign authority"));
    }
    let allowed = match meta.issuer {
        CommandIssuer::Admin | CommandIssuer::System => {
            meta.actor.is_none() || meta.actor == Some(AgentRef::Entity(actor))
        }
        CommandIssuer::Player(player) => {
            meta.actor == Some(AgentRef::Entity(actor))
                && state.characters.values().any(|c| {
                    c.entity_id == actor
                        && c.status == CharacterStatus::Active
                        && c.controlling_player_id == Some(player)
                })
        }
        CommandIssuer::Import => false,
    };
    if allowed {
        Ok(())
    } else {
        Err(RulesError::Unauthorized)
    }
}

/// Reconstructs the grant and all numeric spell parameters from authoritative state
/// and the embedded canonical content. Target/area binding and component access are
/// separate source prerequisites which the scheduler must complete before begin_cast.
pub fn plan_spell_cast(
    state: &CampaignState,
    meta: &CommandMeta,
    choice: &SpellCastChoice,
) -> Result<SpellCastPlan, RulesError> {
    plan_spell_cast_at(state, meta, choice, 0)
}

/// Nested work must use the shared resolution's checked occurrence allocation.
/// The zero-ordinal convenience function is only for a direct initial invocation.
pub fn plan_spell_cast_at(
    state: &CampaignState,
    meta: &CommandMeta,
    choice: &SpellCastChoice,
    occurrence: u16,
) -> Result<SpellCastPlan, RulesError> {
    if !matches!(choice.grant, SpellGrantChoice::Prepared) {
        return Err(unavailable(
            "source creatures require an authenticated feature activation",
        ));
    }
    authorize(state, meta, choice.actor)?;
    plan_spell_cast_authorized(state, meta, choice, occurrence)
}

fn plan_spell_cast_authorized(
    state: &CampaignState,
    meta: &CommandMeta,
    choice: &SpellCastChoice,
    occurrence: u16,
) -> Result<SpellCastPlan, RulesError> {
    if occurrence >= 32_768 {
        return Err(invalid("spell occurrence exceeds resolution capacity"));
    }
    let defs = definitions()?;
    let rules = state.rules.as_ref().ok_or(RulesError::Uninitialized)?;
    if rules.pack_id != defs.ruleset_id || rules.pack_version != defs.ruleset_version {
        return Err(RulesError::Incompatible(
            "tactical spell source pin differs".into(),
        ));
    }
    let entity = rules
        .entities
        .get(&choice.actor)
        .ok_or_else(|| invalid("caster mechanics absent"))?;
    if !crate::tactical_conditions::can_act(rules, choice.actor)? {
        return Err(unavailable("caster cannot act"));
    }
    let spell = defs
        .spell(&choice.spell_id)
        .ok_or_else(|| unavailable("spell is not in supported source content"))?;
    let (level, modifier, attack_bonus, save_dc, expenditure, pin, cantrip_level) = match &choice
        .grant
    {
        SpellGrantChoice::Prepared => {
            if !entity.prepared_spells.contains(&choice.spell_id) {
                return Err(unavailable("spell is not prepared"));
            }
            let casting = entity
                .spellcasting
                .as_ref()
                .ok_or_else(|| unavailable("caster lacks Spellcasting"))?;
            let (level, expenditure) = match choice.resource {
                SpellResourceChoice::Cantrip if spell.level == 0 => (0, SpellExpenditure::None),
                SpellResourceChoice::Slot { level }
                    if spell.level > 0 && level >= spell.level && level <= 9 =>
                {
                    if casting.slots[usize::from(level - 1)] == 0 {
                        return Err(unavailable("the selected spell slot is exhausted"));
                    }
                    (level, SpellExpenditure::Slot { level })
                }
                _ => return Err(unavailable("payment does not match prepared spell")),
            };
            if !(1..=20).contains(&entity.level) {
                return Err(invalid("invalid caster level"));
            }
            let modifier = ability_modifier(entity.ability_scores[casting.ability.index()]) as i16;
            let attack = modifier + proficiency_bonus(entity.level) as i16;
            (
                level,
                modifier,
                Some(attack),
                Some((8 + attack) as u16),
                expenditure,
                source_pin(&defs, spell, None, None)?,
                Some(entity.level),
            )
        }
        SpellGrantChoice::CreatureFeature { feature_id } => {
            if choice.resource != SpellResourceChoice::SourceFeature {
                return Err(unavailable(
                    "source creature casting does not expend invented PC slots",
                ));
            }
            let source = state
                .encounter
                .as_ref()
                .and_then(|e| e.flow.as_ref())
                .and_then(|f| f.combatants.iter().find(|c| c.actor == choice.actor))
                .ok_or_else(|| unavailable("source creature encounter grant absent"))?;
            let TacticalSource::Creature { definition_id } = &source.source else {
                return Err(unavailable("actor has no source creature grant"));
            };
            let creature = defs
                .creature(definition_id)
                .ok_or_else(|| invalid("creature source absent"))?;
            let feature = creature
                .features
                .iter()
                .find(|f| f.id == *feature_id)
                .ok_or_else(|| unavailable("source feature absent"))?;
            let MonsterFeature::Spellcasting {
                ability,
                save_dc,
                attack_bonus,
                spells,
            } = &feature.feature
            else {
                return Err(unavailable("selected creature feature is not Spellcasting"));
            };
            let grant = spells
                .iter()
                .find(|s| s.spell_id == spell.id)
                .ok_or_else(|| unavailable("spell is not granted by that source feature"))?;
            let modifier =
                ability_modifier(creature.statistics.ability_scores[ability.index()]) as i16;
            let expenditure = grant
                .uses_per_long_rest
                .map_or(SpellExpenditure::None, |maximum| {
                    SpellExpenditure::CreatureUse {
                        feature_id: feature_id.clone(),
                        spell_id: spell.id.clone(),
                        maximum,
                    }
                });
            (
                grant.cast_level,
                modifier,
                *attack_bonus,
                save_dc.map(u16::from),
                expenditure,
                source_pin(&defs, spell, Some(creature), Some(feature))?,
                None,
            )
        }
    };
    let program = compile_spell_program(
        spell,
        pin,
        level,
        modifier,
        attack_bonus,
        save_dc,
        cantrip_level,
    )?;
    let cost = match spell.casting_time {
        CastingTime::Action => SpellCastingCost::Action,
        CastingTime::BonusAction => SpellCastingCost::BonusAction,
        CastingTime::Reaction { .. } => SpellCastingCost::Reaction,
    };
    let ready = matches!(choice.mode, SpellCastMode::Ready { .. });
    if let SpellCastMode::Ready { trigger } = &choice.mode
        && (cost != SpellCastingCost::Action
            || trigger.trim().is_empty()
            || trigger.len() > 500
            || trigger.chars().any(char::is_control))
    {
        return Err(unavailable(
            "Ready requires an action spell and a bounded perceivable trigger",
        ));
    }
    let components = source_components(&defs, spell, &program.source);
    if !components.material && choice.material != SpellMaterialChoice::None {
        return Err(invalid("unneeded material component selection"));
    }
    let group = (program.concentration || ready)
        .then(|| spell_concentration_id(meta.id, choice.actor, occurrence));
    Ok(SpellCastPlan {
        origin: meta.clone(),
        occurrence,
        choice: choice.clone(),
        program,
        cost,
        expenditure,
        activation_prepaid: matches!(choice.grant, SpellGrantChoice::CreatureFeature { .. }),
        components,
        concentration_group: group,
    })
}

fn command_valid(
    plan: &SpellCastPlan,
    previous: &CommandMeta,
    meta: &CommandMeta,
) -> Result<(), RulesError> {
    if meta.id.0.is_nil()
        || matches!(meta.issuer, CommandIssuer::Import)
        || meta.campaign_id != plan.origin.campaign_id
        || meta.expected_event_sequence < previous.expected_event_sequence
        || (meta.expected_event_sequence == previous.expected_event_sequence && *meta != *previous)
        || (meta.id == previous.id && *meta != *previous)
        || (meta.id == plan.origin.id && *meta != plan.origin)
    {
        return Err(invalid(
            "cast continuation has foreign, future-origin or changed metadata",
        ));
    }
    Ok(())
}

/// Structural/source validation complements journal replay; it does not authenticate
/// a forged initial snapshot. Prepared scores/level are proven by replaying the original
/// plan against its pre-cast state, rather than trusting this retained derived record.
pub fn validate_spell_plan(plan: &SpellCastPlan) -> Result<(), RulesError> {
    if plan.origin.id.0.is_nil()
        || plan.origin.campaign_id.0.is_nil()
        || plan.choice.actor.0.is_nil()
        || matches!(plan.origin.issuer, CommandIssuer::Import)
    {
        return Err(invalid("retained spell authority is invalid"));
    }
    let defs = definitions()?;
    let spell = defs
        .spell(&plan.choice.spell_id)
        .ok_or_else(|| invalid("retained spell source absent"))?;
    let program = &plan.program;
    if !(-5..=10).contains(&program.ability_modifier) || plan.occurrence >= 32_768 {
        return Err(invalid("retained spell parameter exceeds source bounds"));
    }
    let (pin, expenditure) = match &plan.choice.grant {
        SpellGrantChoice::Prepared => {
            let level = program
                .cantrip_character_level
                .ok_or_else(|| invalid("prepared cast level absent"))?;
            if !(1..=20).contains(&level) {
                return Err(invalid("prepared cast level invalid"));
            }
            let attack = program.ability_modifier + proficiency_bonus(level) as i16;
            if program.attack_bonus != Some(attack) || program.save_dc != Some((8 + attack) as u16)
            {
                return Err(invalid("prepared cast modifiers disagree"));
            }
            let payment = match plan.choice.resource {
                SpellResourceChoice::Cantrip if spell.level == 0 && program.spell_level == 0 => {
                    SpellExpenditure::None
                }
                SpellResourceChoice::Slot { level }
                    if spell.level > 0 && level == program.spell_level =>
                {
                    SpellExpenditure::Slot { level }
                }
                _ => return Err(invalid("retained prepared payment differs")),
            };
            (source_pin(&defs, spell, None, None)?, payment)
        }
        SpellGrantChoice::CreatureFeature { feature_id } => {
            let creature = program
                .source
                .creature_definition_id
                .as_ref()
                .and_then(|id| defs.creature(id))
                .ok_or_else(|| invalid("retained creature source absent"))?;
            let feature = creature
                .features
                .iter()
                .find(|f| f.id == *feature_id)
                .ok_or_else(|| invalid("retained source feature absent"))?;
            let MonsterFeature::Spellcasting {
                ability,
                save_dc,
                attack_bonus,
                spells,
            } = &feature.feature
            else {
                return Err(invalid("retained grant is not Spellcasting"));
            };
            let grant = spells
                .iter()
                .find(|s| s.spell_id == spell.id)
                .ok_or_else(|| invalid("retained spell grant absent"))?;
            if plan.choice.resource != SpellResourceChoice::SourceFeature
                || program.spell_level != grant.cast_level
                || program.ability_modifier
                    != ability_modifier(creature.statistics.ability_scores[ability.index()]) as i16
                || program.attack_bonus != *attack_bonus
                || program.save_dc != save_dc.map(u16::from)
                || program.cantrip_character_level.is_some()
            {
                return Err(invalid("retained source creature numbers differ"));
            }
            (
                source_pin(&defs, spell, Some(creature), Some(feature))?,
                grant
                    .uses_per_long_rest
                    .map_or(SpellExpenditure::None, |maximum| {
                        SpellExpenditure::CreatureUse {
                            feature_id: feature_id.clone(),
                            spell_id: spell.id.clone(),
                            maximum,
                        }
                    }),
            )
        }
    };
    if pin != program.source
        || expenditure != plan.expenditure
        || plan.activation_prepaid
            != matches!(plan.choice.grant, SpellGrantChoice::CreatureFeature { .. })
    {
        return Err(invalid("retained source identity or expenditure differs"));
    }
    let expected = compile_spell_program(
        spell,
        pin,
        program.spell_level,
        program.ability_modifier,
        program.attack_bonus,
        program.save_dc,
        program.cantrip_character_level,
    )?;
    if expected != *program {
        return Err(invalid(
            "retained effect program differs from canonical source",
        ));
    }
    let cost = match spell.casting_time {
        CastingTime::Action => SpellCastingCost::Action,
        CastingTime::BonusAction => SpellCastingCost::BonusAction,
        CastingTime::Reaction { .. } => SpellCastingCost::Reaction,
    };
    let ready = match &plan.choice.mode {
        SpellCastMode::Immediate => false,
        SpellCastMode::Ready { trigger }
            if cost == SpellCastingCost::Action
                && !trigger.trim().is_empty()
                && trigger.len() <= 500
                && !trigger.chars().any(char::is_control) =>
        {
            true
        }
        _ => return Err(invalid("retained Ready choice is invalid")),
    };
    if plan.cost != cost
        || plan.components != source_components(&defs, spell, &program.source)
        || (!plan.components.material && plan.choice.material != SpellMaterialChoice::None)
        || plan.concentration_group
            != (program.concentration || ready)
                .then(|| spell_concentration_id(plan.origin.id, plan.choice.actor, plan.occurrence))
    {
        return Err(invalid("retained casting requirements differ from source"));
    }
    Ok(())
}

pub fn validate_spell_cast(cast: &SpellCast) -> Result<(), RulesError> {
    validate_spell_plan(&cast.plan)?;
    command_valid(&cast.plan, &cast.plan.origin, &cast.last_operation)?;
    if cast.started_at.0 < 0
        || cast.started_on_turn == 0
        || cast.plan.origin.id.0.is_nil()
        || (matches!(
            cast.phase,
            SpellCastPhase::Held | SpellCastPhase::Released | SpellCastPhase::Expired
        ) && !matches!(cast.plan.choice.mode, SpellCastMode::Ready { .. }))
        || (cast.phase == SpellCastPhase::Committed
            && !matches!(cast.plan.choice.mode, SpellCastMode::Immediate))
        || (cast.phase == SpellCastPhase::Interrupted(SpellCastingInterruption::ConcentrationLost)
            && cast.plan.concentration_group.is_none())
    {
        return Err(invalid("retained cast phase or origin is invalid"));
    }
    Ok(())
}

/// This internal primitive must run only after the source resolver validates target,
/// components, current budget and trigger eligibility. Returned obligations are atomic:
/// persisting only the cast would falsely claim action/concentration/resource changes.
pub fn begin_cast(
    plan: &SpellCastPlan,
    context: &SpellCastContext,
) -> Result<SpellCastTransition, RulesError> {
    validate_spell_plan(plan)?;
    if plan.origin.id.0.is_nil() || context.turn_number == 0 || context.now.0 < 0 {
        return Err(invalid("invalid cast origin or turn"));
    }
    if !context.can_act {
        return Err(unavailable("caster cannot begin an action"));
    }
    if ((!plan.activation_prepaid && plan.cost != SpellCastingCost::Reaction)
        || matches!(plan.choice.mode, SpellCastMode::Ready { .. }))
        && context.current_actor != plan.choice.actor
    {
        return Err(unavailable("casting requires the caster's turn"));
    }
    let mut obligations = match &plan.choice.grant {
        SpellGrantChoice::Prepared => vec![SpellCastObligation::SpendCastingCost {
            actor: plan.choice.actor,
            cost: plan.cost,
        }],
        SpellGrantChoice::CreatureFeature { feature_id } => {
            vec![SpellCastObligation::RequireCreatureFeatureReceipt {
                actor: plan.choice.actor,
                feature_id: feature_id.clone(),
                spell_id: plan.choice.spell_id.clone(),
                origin: plan.origin.clone(),
            }]
        }
    };
    if let Some(id) = plan.concentration_group {
        obligations.push(SpellCastObligation::BeginConcentration {
            group: ConcentrationGroup {
                id,
                source: EffectSource {
                    definition_id: plan.program.source.spell_id.clone(),
                    actor: plan.choice.actor,
                    command: plan.origin.clone(),
                    ordinal: plan.occurrence,
                },
                expires: if matches!(plan.choice.mode, SpellCastMode::Ready { .. }) {
                    TacticalEffectExpiry::AfterOwnerBoundaries {
                        owner: plan.choice.actor,
                        boundary: TurnBoundary::Start,
                        remaining: 1,
                    }
                } else {
                    TacticalEffectExpiry::Never
                },
                stage: ConcentrationStage::Casting,
            },
        });
    }
    Ok(SpellCastTransition {
        cast: SpellCast {
            plan: plan.clone(),
            phase: SpellCastPhase::Casting,
            started_at: context.now,
            started_on_turn: context.turn_number,
            last_operation: plan.origin.clone(),
        },
        obligations,
    })
}

pub fn advance_cast(
    cast: &SpellCast,
    meta: &CommandMeta,
    context: &SpellCastContext,
    operation: SpellCastAdvance,
) -> Result<SpellCastTransition, RulesError> {
    validate_spell_cast(cast)?;
    command_valid(&cast.plan, &cast.last_operation, meta)?;
    if context.now < cast.started_at || context.turn_number < cast.started_on_turn {
        return Err(invalid("invalid cast time, turn or payment phase"));
    }
    if cast.phase == SpellCastPhase::Casting && context.turn_number != cast.started_on_turn {
        return Err(invalid(
            "an unresolved casting window cannot cross a turn boundary",
        ));
    }
    let mut next = cast.clone();
    let mut obligations = Vec::new();
    match operation {
        SpellCastAdvance::Commit if cast.phase == SpellCastPhase::Casting => {
            if !context.can_act {
                return Err(unavailable("caster can no longer complete the action"));
            }
            if let Some(group) = cast.plan.concentration_group
                && context.concentration != Some(group)
            {
                return Err(unavailable("casting concentration was lost"));
            }
            if !cast.plan.activation_prepaid {
                obligations.push(SpellCastObligation::CommitExpenditure {
                    actor: cast.plan.choice.actor,
                    expenditure: cast.plan.expenditure.clone(),
                });
            }
            if matches!(cast.plan.choice.mode, SpellCastMode::Ready { .. }) {
                next.phase = SpellCastPhase::Held;
            } else {
                next.phase = SpellCastPhase::Committed;
                release_program(cast, &mut obligations);
            }
        }
        SpellCastAdvance::Countered if cast.phase == SpellCastPhase::Casting => {
            next.phase = SpellCastPhase::Countered;
            end_owned_group(cast, context, &mut obligations);
        }
        SpellCastAdvance::Interrupt(reason) if cast.phase == SpellCastPhase::Casting => {
            let supported = match reason {
                SpellCastingInterruption::ConcentrationLost => {
                    cast.plan.concentration_group.is_some()
                        && context.concentration != cast.plan.concentration_group
                }
                SpellCastingInterruption::Incapacitated => !context.can_act,
            };
            if !supported {
                return Err(unavailable("casting interruption is not present"));
            }
            // SRD105/179: this source loss ends the spell. Counterspell (120) is
            // the represented explicit exception to ordinary slot expenditure.
            // Longer casting times have their own exception, outside this reducer.
            if !cast.plan.activation_prepaid {
                obligations.push(SpellCastObligation::CommitExpenditure {
                    actor: cast.plan.choice.actor,
                    expenditure: cast.plan.expenditure.clone(),
                });
            }
            next.phase = SpellCastPhase::Interrupted(reason);
            end_owned_group(cast, context, &mut obligations);
        }
        SpellCastAdvance::ReleaseReady if cast.phase == SpellCastPhase::Held => {
            if !context.can_act {
                return Err(unavailable("caster cannot take the release reaction"));
            }
            if context.current_actor == cast.plan.choice.actor
                && context.turn_number > cast.started_on_turn
            {
                return Err(unavailable("held spell has reached the caster's next turn"));
            }
            if context.concentration != cast.plan.concentration_group {
                return Err(unavailable("held spell concentration was lost"));
            }
            obligations.push(SpellCastObligation::SpendCastingCost {
                actor: cast.plan.choice.actor,
                cost: SpellCastingCost::Reaction,
            });
            next.phase = SpellCastPhase::Released;
            release_program(cast, &mut obligations);
        }
        SpellCastAdvance::ExpireHeld if cast.phase == SpellCastPhase::Held => {
            if context.concentration == cast.plan.concentration_group
                && !(context.current_actor == cast.plan.choice.actor
                    && context.turn_number > cast.started_on_turn)
            {
                return Err(unavailable(
                    "held spell has not expired or lost concentration",
                ));
            }
            next.phase = SpellCastPhase::Expired;
            end_owned_group(cast, context, &mut obligations);
        }
        _ => {
            return Err(unavailable(
                "cast operation does not match its durable phase",
            ));
        }
    }
    next.last_operation = meta.clone();
    Ok(SpellCastTransition {
        cast: next,
        obligations,
    })
}

fn end_owned_group(
    cast: &SpellCast,
    context: &SpellCastContext,
    obligations: &mut Vec<SpellCastObligation>,
) {
    if let Some(group) = cast.plan.concentration_group
        && context.concentration == Some(group)
    {
        obligations.push(SpellCastObligation::EndConcentration {
            actor: cast.plan.choice.actor,
            group,
        });
    }
}
fn release_program(cast: &SpellCast, obligations: &mut Vec<SpellCastObligation>) {
    if let Some(group) = cast.plan.concentration_group {
        if cast.plan.program.concentration {
            obligations.push(SpellCastObligation::ActivateConcentration {
                group,
                duration: cast.plan.program.duration,
            });
        } else {
            obligations.push(SpellCastObligation::EndConcentration {
                actor: cast.plan.choice.actor,
                group,
            });
        }
    }
    obligations.push(SpellCastObligation::QueueProgram {
        origin: cast.plan.origin.clone(),
        occurrence: cast.plan.occurrence,
    });
}
