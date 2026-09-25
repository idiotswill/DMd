//! Read-only defense composition. Equipment remains the authoritative ordinary AC;
//! timed source clauses remain in the existing effect lifecycle and never rewrite it.
use crate::{RulesError, ability_modifier, armor_class, tactical_effects::effect_applies_to};
use dmd_domain::*;

fn invalid(message: &str) -> RulesError {
    RulesError::Invalid(message.into())
}

pub fn wearing_armor(state: &CampaignState, actor: EntityId) -> bool {
    state
        .rules
        .as_ref()
        .and_then(|rules| rules.tactical_inventory.as_ref())
        .and_then(|inventory| inventory.loadout(actor))
        .is_some_and(|loadout| loadout.worn_armor.is_some())
}

/// Shield training and physical donning are independent of an unarmored formula.
/// In particular a shield is not worn armor for Mage Armor's ending clause.
pub fn trained_shield_bonus(state: &CampaignState, actor: EntityId) -> Result<i32, RulesError> {
    let rules = state.rules.as_ref().ok_or(RulesError::Uninitialized)?;
    let shield = rules
        .tactical_inventory
        .as_ref()
        .and_then(|inventory| inventory.loadout(actor))
        .and_then(|loadout| loadout.shield);
    let Some(shield) = shield else { return Ok(0) };
    if state
        .items
        .get(&shield)
        .is_none_or(|item| item.definition_id != "shield")
    {
        return Err(invalid(
            "defense query found an unsupported shield identity",
        ));
    }
    let trained = if let Some(profile) = rules
        .tactical_creatures
        .as_ref()
        .and_then(|creatures| creatures.profile(actor))
    {
        crate::tactical_creatures::source_for_profile(profile)
            .map_err(|error| RulesError::Invalid(error.to_string()))?
            .statistics
            .gear
            .iter()
            .any(|id| id == "shield")
    } else {
        state
            .table
            .as_ref()
            .and_then(|table| {
                table
                    .character_profiles
                    .values()
                    .find(|profile| profile.entity_id == actor)
            })
            .is_some_and(|profile| {
                profile
                    .armor_training
                    .iter()
                    .any(|training| training == "shield")
            })
    };
    Ok(if trained { 2 } else { 0 })
}

/// This first source formula path admits an ordinary unarmored baseline only.
/// A creature with a competing natural/class formula needs its explicit choice;
/// selecting the maximum here would silently make that player's decision.
pub fn ordinary_unarmored_formula(
    state: &CampaignState,
    actor: EntityId,
) -> Result<bool, RulesError> {
    let entity = state
        .rules
        .as_ref()
        .ok_or(RulesError::Uninitialized)?
        .entities
        .get(&actor)
        .ok_or_else(|| invalid("defense target mechanics absent"))?;
    Ok(!wearing_armor(state, actor)
        && armor_class(entity)
            == 10
                + ability_modifier(entity.ability_scores[Ability::Dexterity.index()])
                + trained_shield_bonus(state, actor)?)
}

pub fn effective_armor_class(state: &CampaignState, actor: EntityId) -> Result<i32, RulesError> {
    let rules = state.rules.as_ref().ok_or(RulesError::Uninitialized)?;
    let entity = rules
        .entities
        .get(&actor)
        .ok_or_else(|| invalid("defense target mechanics absent"))?;
    let mut base = armor_class(entity);
    let mut formula = None;
    let mut bonus = 0i32;
    if let Some(effects) = &rules.tactical_effects {
        for effect in effects
            .effects
            .iter()
            .filter(|effect| effect_applies_to(effects, effect, actor))
        {
            for defense in &effect.defenses {
                match defense {
                    EffectDefense::BaseArmorClass {
                        base,
                        ability,
                        ends_when_wearing_armor,
                    } => {
                        if *ends_when_wearing_armor && wearing_armor(state, actor) {
                            continue;
                        }
                        let candidate = i32::from(*base)
                            + ability_modifier(entity.ability_scores[ability.index()])
                            + trained_shield_bonus(state, actor)?;
                        if formula.is_some_and(|previous| previous != candidate) {
                            return Err(invalid(
                                "competing armor formulas require an explicit source choice",
                            ));
                        }
                        formula = Some(candidate);
                    }
                    EffectDefense::ArmorClassBonus { bonus: addition } => {
                        bonus = bonus
                            .checked_add(i32::from(*addition))
                            .ok_or_else(|| invalid("armor bonus overflow"))?;
                    }
                    EffectDefense::PreventSpellDamage { .. } => (),
                }
            }
        }
    }
    if let Some(formula) = formula {
        base = formula;
    }
    base.checked_add(bonus)
        .ok_or_else(|| invalid("effective armor overflow"))
}

pub fn prevents_spell_damage(state: &CampaignState, actor: EntityId, spell_id: &str) -> bool {
    state.rules.as_ref().and_then(|rules| rules.tactical_effects.as_ref())
        .is_some_and(|effects| effects.effects.iter().any(|effect| {
            effect_applies_to(effects, effect, actor) && effect.defenses.iter().any(|defense| {
                matches!(defense, EffectDefense::PreventSpellDamage { spell_id: blocked } if blocked == spell_id)
            })
        }))
}
