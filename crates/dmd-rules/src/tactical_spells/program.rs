use super::*;

fn more_dice(dice: &mut Vec<DieSpec>, additional: &[DieSpec], times: u8) -> Result<(), RulesError> {
    for die in additional {
        let count = die
            .count
            .checked_mul(u16::from(times))
            .ok_or_else(|| invalid("spell dice overflow"))?;
        if count == 0 {
            continue;
        }
        if let Some(existing) = dice.iter_mut().find(|d| d.sides == die.sides) {
            existing.count = existing
                .count
                .checked_add(count)
                .ok_or_else(|| invalid("spell dice overflow"))?;
        } else {
            dice.push(DieSpec {
                count,
                sides: die.sides,
            });
        }
    }
    if dice.iter().map(|d| u32::from(d.count)).sum::<u32>() > 256 {
        return Err(invalid("spell program exceeds dice capacity"));
    }
    Ok(())
}
fn damage(
    formula: &crate::tactical_definitions::DamageComponent,
    upcast: Option<&Upcast>,
    extra: u8,
) -> Result<SpellDamageDice, RulesError> {
    let mut dice = formula.amount.dice.clone();
    if let Some(Upcast::ExtraDamageDie { die }) = upcast {
        more_dice(&mut dice, &[*die], extra)?;
    }
    Ok(SpellDamageDice {
        dice,
        modifier: formula.amount.fixed,
        damage_type: formula.damage_type,
    })
}

/// Compile source clauses into typed executable work descriptions. This does not
/// authorize arbitrary programs: production callers reconstruct it with plan_spell_cast
/// and replay compares the canonical result to the retained source-derived record.
pub fn compile_spell_program(
    spell: &TacticalSpellDefinition,
    source: SpellSourcePin,
    spell_level: u8,
    ability_modifier: i16,
    attack_bonus: Option<i16>,
    save_dc: Option<u16>,
    cantrip_character_level: Option<u8>,
) -> Result<SpellEffectProgram, RulesError> {
    if spell_level > 9
        || spell_level < spell.level
        || (spell.level == 0 && spell_level != 0)
        || source.spell_id != spell.id
        || !(-5..=10).contains(&ability_modifier)
        || cantrip_character_level.is_some_and(|level| !(1..=20).contains(&level))
    {
        return Err(invalid("invalid source spell program context"));
    }
    let extra = spell_level - spell.level;
    let mut nodes = Vec::new();
    for effect in &spell.effects {
        let node = match effect {
            EffectDescriptor::SaveCommand { ability, choices } => {
                if save_dc.is_none() {
                    return Err(invalid("source save spell lacks a DC"));
                }
                SpellProgramNode::SaveCommand {
                    ability: *ability,
                    choices: choices
                        .iter()
                        .map(|choice| match choice {
                            CommandWord::Approach => SpellCommandWord::Approach,
                            CommandWord::Drop => SpellCommandWord::Drop,
                            CommandWord::Flee => SpellCommandWord::Flee,
                            CommandWord::Grovel => SpellCommandWord::Grovel,
                            CommandWord::Halt => SpellCommandWord::Halt,
                        })
                        .collect(),
                }
            }
            EffectDescriptor::Healing {
                dice,
                add_spellcasting_modifier,
            } => {
                let mut dice = dice.clone();
                if let Some(Upcast::ExtraHealingDice { dice: additional }) = &spell.upcast {
                    more_dice(&mut dice, additional, extra)?;
                }
                SpellProgramNode::Heal {
                    dice,
                    modifier: if *add_spellcasting_modifier {
                        ability_modifier
                    } else {
                        0
                    },
                }
            }
            EffectDescriptor::RangedSpellAttack {
                damage: value,
                extra_die_at_levels,
                ignite_unworn_uncarried_target,
            } => {
                if attack_bonus.is_none() {
                    return Err(invalid("source attack spell lacks an attack modifier"));
                }
                let mut damage = damage(value, spell.upcast.as_ref(), extra)?;
                if !extra_die_at_levels.is_empty() {
                    let level = cantrip_character_level.ok_or_else(|| {
                        unavailable("source cantrip scaling level is not represented")
                    })?;
                    let count = extra_die_at_levels
                        .iter()
                        .filter(|&&at| level >= at)
                        .count() as u8;
                    more_dice(&mut damage.dice, &value.amount.dice, count)?;
                }
                SpellProgramNode::AttackDamage {
                    damage,
                    share: SpellDamageShare::PerAttack,
                    ignite_unworn_uncarried_target: *ignite_unworn_uncarried_target,
                }
            }
            EffectDescriptor::SaveDamage {
                ability,
                damage: value,
                half_on_success,
                push_on_failure_feet,
            } => {
                if save_dc.is_none() {
                    return Err(invalid("source save spell lacks a DC"));
                }
                SpellProgramNode::SaveDamage {
                    ability: *ability,
                    damage: damage(value, spell.upcast.as_ref(), extra)?,
                    share: SpellDamageShare::SimultaneousSavingThrows,
                    half_on_success: *half_on_success,
                    push_on_failure_feet: *push_on_failure_feet,
                }
            }
            EffectDescriptor::SaveCondition {
                ability,
                condition,
                repeat,
            } => {
                if save_dc.is_none() {
                    return Err(invalid("source save spell lacks a DC"));
                }
                SpellProgramNode::SaveCondition {
                    ability: *ability,
                    condition: *condition,
                    repeat_at_target_end: repeat.is_some(),
                }
            }
            EffectDescriptor::Damage { damage: value } => SpellProgramNode::AutomaticDamage {
                damage: damage(value, spell.upcast.as_ref(), extra)?,
                share: if matches!(spell.targets, TargetSelection::Darts { .. }) {
                    SpellDamageShare::PerDart
                } else {
                    SpellDamageShare::SingleOccurrence
                },
            },
            EffectDescriptor::Condition { condition } => SpellProgramNode::Condition {
                condition: *condition,
            },
            EffectDescriptor::MovableDimLights {
                radius_feet,
                bonus_action_move_feet,
                maximum_separation_feet,
                vanish_outside_casting_range,
            } => SpellProgramNode::DimLights {
                radius_feet: *radius_feet,
                bonus_action_move_feet: *bonus_action_move_feet,
                maximum_separation_feet: *maximum_separation_feet,
                vanish_outside_casting_range: *vanish_outside_casting_range,
            },
            EffectDescriptor::HeavilyObscured {
                dispersed_by_strong_wind,
            } => SpellProgramNode::HeavyObscuration {
                dispersed_by_strong_wind: *dispersed_by_strong_wind,
            },
            EffectDescriptor::ArmorClassBonus {
                bonus,
                includes_triggering_attack,
            } => SpellProgramNode::ArmorClassBonus {
                bonus: *bonus,
                includes_triggering_attack: *includes_triggering_attack,
            },
            EffectDescriptor::BaseArmorClass {
                base,
                ability,
                ends_when_wearing_armor,
            } => SpellProgramNode::BaseArmorClass {
                base: *base,
                ability: *ability,
                ends_when_wearing_armor: *ends_when_wearing_armor,
            },
            EffectDescriptor::InterruptSpellCasting {
                ability,
                preserve_spell_slot,
            } => SpellProgramNode::InterruptSpellCasting {
                ability: *ability,
                preserve_spell_slot: *preserve_spell_slot,
            },
            EffectDescriptor::PreventSpellDamage { spell_id } => {
                SpellProgramNode::PreventSpellDamage {
                    spell_id: spell_id.clone(),
                }
            }
            EffectDescriptor::IgniteUnwornUncarriedObjects => {
                SpellProgramNode::IgniteUnwornUncarriedObjects
            }
            EffectDescriptor::PushUnsecuredObjectsEntirelyInArea { feet } => {
                SpellProgramNode::PushUnsecuredObjectsEntirelyInArea { feet: *feet }
            }
            EffectDescriptor::Audible { feet } => SpellProgramNode::Audible { feet: *feet },
        };
        nodes.push(node);
    }
    if nodes.is_empty() || nodes.len() > 32 {
        return Err(invalid("spell program node count invalid"));
    }
    let (duration, concentration) = match spell.duration {
        EffectDuration::Instantaneous => (SpellDuration::Instantaneous, false),
        EffectDuration::Seconds {
            seconds,
            concentration,
        } => (SpellDuration::Seconds { seconds }, concentration),
        EffectDuration::UntilStartOfCastersNextTurn => (
            SpellDuration::OwnerBoundary {
                boundary: TurnBoundary::Start,
            },
            false,
        ),
    };
    let range = match spell.range {
        SpellRange::Caster => SpellRangeLimit::Caster,
        SpellRange::Touch => SpellRangeLimit::Touch,
        SpellRange::Distance { feet } => SpellRangeLimit::Feet(feet),
    };
    let targets = match &spell.targets {
        TargetSelection::Caster => SpellTargetRule::Caster,
        TargetSelection::CreatureOrObject => SpellTargetRule::CreatureOrObject,
        TargetSelection::Creatures {
            maximum,
            creature_type,
            requires_sight,
        } => {
            let count = if let Some(Upcast::AdditionalTargets { count }) = spell.upcast {
                count
            } else {
                0
            };
            SpellTargetRule::Creatures {
                maximum: maximum
                    .checked_add(
                        count
                            .checked_mul(extra)
                            .ok_or_else(|| invalid("target count overflow"))?,
                    )
                    .ok_or_else(|| invalid("target count overflow"))?,
                creature_type: creature_type.map(|kind| format!("{kind:?}")),
                requires_sight: *requires_sight,
            }
        }
        TargetSelection::Darts {
            count,
            requires_sight,
        } => {
            let added = if let Some(Upcast::AdditionalDarts { count }) = spell.upcast {
                count
            } else {
                0
            };
            SpellTargetRule::Darts {
                count: count
                    .checked_add(
                        added
                            .checked_mul(extra)
                            .ok_or_else(|| invalid("dart count overflow"))?,
                    )
                    .ok_or_else(|| invalid("dart count overflow"))?,
                requires_sight: *requires_sight,
            }
        }
        TargetSelection::Rays { count } => {
            let added = if let Some(Upcast::AdditionalRays { count }) = spell.upcast {
                count
            } else {
                0
            };
            SpellTargetRule::Rays {
                count: count
                    .checked_add(
                        added
                            .checked_mul(extra)
                            .ok_or_else(|| invalid("ray count overflow"))?,
                    )
                    .ok_or_else(|| invalid("ray count overflow"))?,
            }
        }
        TargetSelection::LightPoints {
            maximum,
            may_combine_as_medium_form,
        } => SpellTargetRule::LightPoints {
            maximum: *maximum,
            may_combine_as_medium_form: *may_combine_as_medium_form,
        },
        TargetSelection::Area { shape } => {
            let added = if let Some(Upcast::AdditionalRadius { feet }) = spell.upcast {
                feet.checked_mul(u16::from(extra))
                    .ok_or_else(|| invalid("area radius overflow"))?
            } else {
                0
            };
            SpellTargetRule::Area(match *shape {
                AreaShape::Cone { length_feet } => SpellAreaShape::Cone { length_feet },
                AreaShape::Cube { side_feet } => SpellAreaShape::Cube { side_feet },
                AreaShape::Cylinder {
                    radius_feet,
                    height_feet,
                } => SpellAreaShape::Cylinder {
                    radius_feet: radius_feet
                        .checked_add(added)
                        .ok_or_else(|| invalid("area radius overflow"))?,
                    height_feet,
                },
                AreaShape::Emanation { radius_feet } => SpellAreaShape::Emanation {
                    radius_feet: radius_feet
                        .checked_add(added)
                        .ok_or_else(|| invalid("area radius overflow"))?,
                },
                AreaShape::Line {
                    length_feet,
                    width_feet,
                } => SpellAreaShape::Line {
                    length_feet,
                    width_feet,
                },
                AreaShape::Sphere { radius_feet } => SpellAreaShape::Sphere {
                    radius_feet: radius_feet
                        .checked_add(added)
                        .ok_or_else(|| invalid("area radius overflow"))?,
                },
            })
        }
    };
    Ok(SpellEffectProgram {
        schema_version: TACTICAL_SPELL_SCHEMA_VERSION,
        source,
        spell_level,
        attack_bonus,
        save_dc,
        ability_modifier,
        cantrip_character_level,
        range,
        targets,
        duration,
        concentration,
        nodes,
        source_pages: spell.source_pages.clone(),
    })
}
