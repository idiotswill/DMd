//! Source program leaves consumed by the shared scheduler. No function here creates
//! a queue, chooses a controller outcome, or accepts arbitrary damage/effect fields.
use super::*;
use crate::ResolveRoll;

fn occurrence(
    record: &TacticalCasting,
    occurrence: SpellProgramOccurrence,
) -> Result<&SpellBoundTarget, RulesError> {
    validate_retained_spell(record)?;
    if !matches!(
        record.cast.phase,
        SpellCastPhase::Committed | SpellCastPhase::Released
    ) || occurrence.node != 0
        || record.completed.contains(&occurrence)
    {
        return Err(invalid(
            "spell program occurrence is not awaiting execution",
        ));
    }
    record
        .targets
        .get(usize::from(occurrence.target))
        .ok_or_else(|| invalid("spell target occurrence is outside the bound selection"))
}

pub fn spell_amount_request(
    record: &TacticalCasting,
    at: SpellProgramOccurrence,
    id: RollRequestId,
    visibility: RollVisibility,
) -> Result<RollRequest, RulesError> {
    occurrence(record, at)?;
    if id.0.is_nil() {
        return Err(invalid("spell amount request identity is absent"));
    }
    let (dice, modifier, reason) = match record.cast.plan.program.nodes.as_slice() {
        [SpellProgramNode::Heal { dice, modifier }] => (dice, *modifier, "Spell healing"),
        [
            SpellProgramNode::AutomaticDamage {
                damage,
                share: SpellDamageShare::PerDart | SpellDamageShare::SingleOccurrence,
            },
        ] => (&damage.dice, damage.modifier, "Spell damage"),
        _ => {
            return Err(invalid(
                "spell occurrence is not an independent damage or healing roll",
            ));
        }
    };
    Ok(RollRequest {
        id,
        roller: Some(record.cast.plan.choice.actor),
        dice: dice.clone(),
        modifier: i32::from(modifier),
        mode: RollMode::Normal,
        visibility,
        reason: reason.into(),
    })
}

/// The shared continuation first authenticates and journals this raw result, then
/// applies the returned ordinary vitality operation and its nested consequences.
pub fn spell_amount_operation(
    state: &CampaignState,
    record: &TacticalCasting,
    at: SpellProgramOccurrence,
    id: RollRequestId,
    result: &RollResult,
) -> Result<Option<VitalityOperation>, RulesError> {
    let total = spell_amount_request(record, at, id, RollVisibility::Public)?
        .resolve(result)?
        .total
        .max(0) as u32;
    let target = occurrence(record, at)?;
    let entity = state
        .rules
        .as_ref()
        .and_then(|rules| rules.entities.get(&target.actor))
        .ok_or_else(|| invalid("spell amount target is absent"))?;
    // Invalid/dead targets do not refund an accepted cast or invoke the ordinary
    // vitality reducer's forbidden revival path. The caller records no effect.
    if entity.death.dead
        || !target.source_type_matches
        || (matches!(
            record.cast.plan.program.nodes.as_slice(),
            [SpellProgramNode::AutomaticDamage { .. }]
        ) && crate::tactical_defenses::prevents_spell_damage(
            state,
            target.actor,
            &record.cast.plan.program.source.spell_id,
        ))
    {
        return Ok(None);
    }
    match record.cast.plan.program.nodes.as_slice() {
        [SpellProgramNode::Heal { .. }] => Ok(Some(VitalityOperation::Heal { amount: total })),
        [SpellProgramNode::AutomaticDamage { damage, .. }] => Ok(Some(VitalityOperation::Damage {
            packet: DamagePacket {
                // Magic Missile is not an attack and cannot manufacture a critical
                // hit's two failures at 0 HP or a melee knockout permission.
                cause: DamageCause::Other,
                components: vec![dmd_domain::DamageComponent {
                    damage_type: damage.damage_type,
                    amounts: vec![total],
                    adjustments: vec![],
                }],
            },
            knockout: None,
        })),
        _ => Err(invalid("spell amount source changed")),
    }
}

pub fn spell_effect_expiry(
    record: &TacticalCasting,
    now: WorldInstant,
) -> Result<TacticalEffectExpiry, RulesError> {
    validate_retained_spell(record)?;
    if now < record.cast.started_at {
        return Err(invalid("spell effect cannot precede casting"));
    }
    match record.cast.plan.program.duration {
        SpellDuration::Seconds { seconds } => Ok(TacticalEffectExpiry::AtTime(WorldInstant(
            now.0
                .checked_add(i64::from(seconds))
                .ok_or_else(|| invalid("spell duration overflow"))?,
        ))),
        SpellDuration::OwnerBoundary { boundary } => {
            Ok(TacticalEffectExpiry::AfterOwnerBoundaries {
                owner: record.cast.plan.choice.actor,
                boundary,
                remaining: 1,
            })
        }
        SpellDuration::Instantaneous => {
            Err(invalid("instantaneous spell has no ongoing effect expiry"))
        }
    }
}

/// Canonical casting plan supplies the only duration, source and group identity.
/// Call after Commit/ReleaseReady, before installing any failed-save target effect.
pub fn spell_casting_duration_operation(
    record: &TacticalCasting,
    now: WorldInstant,
) -> Result<crate::tactical_effects::EffectLifecycleOperation, RulesError> {
    validate_retained_spell(record)?;
    let plan = &record.cast.plan;
    if !matches!(
        record.cast.phase,
        SpellCastPhase::Committed | SpellCastPhase::Released
    ) || !plan.program.concentration
    {
        return Err(invalid(
            "only a committed concentration program sets its duration",
        ));
    }
    Ok(
        crate::tactical_effects::EffectLifecycleOperation::SetCastingDuration {
            group: plan
                .concentration_group
                .ok_or_else(|| invalid("concentration program has no group"))?,
            source: EffectSource {
                definition_id: plan.program.source.spell_id.clone(),
                actor: plan.choice.actor,
                command: plan.origin.clone(),
                ordinal: plan.occurrence,
            },
            expires: spell_effect_expiry(record, now)?,
        },
    )
}

fn effect_id(
    record: &TacticalCasting,
    at: SpellProgramOccurrence,
    condition_view: bool,
) -> EffectId {
    let plan = &record.cast.plan;
    spell_program_effect_id(
        plan.origin.id,
        plan.choice.actor,
        plan.occurrence,
        at,
        condition_view,
    )
}

pub fn spell_defense_effect(
    state: &CampaignState,
    record: &TacticalCasting,
    at: SpellProgramOccurrence,
) -> Result<Option<TacticalEffect>, RulesError> {
    let target = occurrence(record, at)?;
    let plan = &record.cast.plan;
    if executable_spell_kind(plan)? != ExecutableSpellKind::Defense {
        return Err(invalid("spell occurrence has no source defense program"));
    }
    let mut defenses = Vec::new();
    let mut triggers = Vec::new();
    for node in &plan.program.nodes {
        match node {
            SpellProgramNode::BaseArmorClass {
                base,
                ability,
                ends_when_wearing_armor,
            } => {
                if *ends_when_wearing_armor
                    && crate::tactical_defenses::wearing_armor(state, target.actor)
                {
                    return Ok(None);
                }
                defenses.push(EffectDefense::BaseArmorClass {
                    base: *base,
                    ability: *ability,
                    ends_when_wearing_armor: *ends_when_wearing_armor,
                });
                if *ends_when_wearing_armor {
                    triggers.push(EffectTriggerRule {
                        event: EffectTriggerEvent::ArmorWorn {
                            subject: EffectSubject::Target,
                        },
                        frequency: EffectTriggerFrequency::EveryOccurrence,
                        payload: EffectTriggerPayload::EndTargetEffect,
                    });
                }
            }
            SpellProgramNode::ArmorClassBonus { bonus, .. } => {
                defenses.push(EffectDefense::ArmorClassBonus { bonus: *bonus });
            }
            SpellProgramNode::PreventSpellDamage { spell_id } => {
                defenses.push(EffectDefense::PreventSpellDamage {
                    spell_id: spell_id.clone(),
                });
            }
            _ => return Err(invalid("unsupported clause in source defense program")),
        }
    }
    Ok(Some(TacticalEffect {
        id: effect_id(record, at, false),
        source: EffectSource {
            definition_id: plan.program.source.spell_id.clone(),
            actor: plan.choice.actor,
            command: plan.origin.clone(),
            ordinal: plan.occurrence,
        },
        established_at: None,
        target: TacticalEffectTarget::Creature(target.actor),
        concentration_group: plan
            .program
            .concentration
            .then_some(plan.concentration_group)
            .flatten(),
        expires: spell_effect_expiry(record, state.clock.now)?,
        overlap: Some(EffectOverlap {
            key: plan.program.source.spell_id.clone(),
            potency: 0,
        }),
        conditions: vec![],
        defenses,
        triggers,
    }))
}

/// Called only for a failed source save. None represents invalid creature type,
/// condition immunity, or a concentration group already lost during nested work.
/// Those private causes must not be sent to the player's result explanation.
pub fn spell_condition_effect(
    state: &CampaignState,
    record: &TacticalCasting,
    at: SpellProgramOccurrence,
) -> Result<Option<TacticalEffect>, RulesError> {
    let target = occurrence(record, at)?;
    let [
        SpellProgramNode::SaveCondition {
            ability,
            condition,
            repeat_at_target_end,
        },
    ] = record.cast.plan.program.nodes.as_slice()
    else {
        return Err(invalid(
            "spell occurrence has no failed-save condition program",
        ));
    };
    let rules = state.rules.as_ref().ok_or(RulesError::Uninitialized)?;
    let entity = rules
        .entities
        .get(&target.actor)
        .ok_or_else(|| invalid("spell effect target is absent"))?;
    if !target.source_type_matches
        || entity.death.dead
        || entity.condition_immunities.contains(condition)
    {
        return Ok(None);
    }
    let plan = &record.cast.plan;
    let source = EffectSource {
        definition_id: plan.program.source.spell_id.clone(),
        actor: plan.choice.actor,
        command: plan.origin.clone(),
        ordinal: plan.occurrence,
    };
    if let Some(id) = plan.concentration_group
        && rules
            .tactical_effects
            .as_ref()
            .and_then(|e| e.group_for_owner(plan.choice.actor))
            .is_none_or(|group| group.id != id || group.source != source)
    {
        return Ok(None);
    }
    let dc = plan
        .program
        .save_dc
        .ok_or_else(|| invalid("spell saving throw DC is absent"))?;
    Ok(Some(TacticalEffect {
        id: effect_id(record, at, false),
        source,
        established_at: None,
        target: TacticalEffectTarget::Creature(target.actor),
        concentration_group: plan.concentration_group,
        expires: spell_effect_expiry(record, state.clock.now)?,
        // The represented condition is otherwise identical. A harder ending save
        // is its potency; upcast extra-target count is not a stronger condition.
        overlap: Some(EffectOverlap {
            key: plan.program.source.spell_id.clone(),
            potency: i32::from(dc),
        }),
        conditions: vec![EffectCondition {
            id: effect_id(record, at, true),
            condition: *condition,
        }],
        defenses: vec![],
        triggers: if *repeat_at_target_end {
            vec![EffectTriggerRule {
                event: EffectTriggerEvent::Turn {
                    subject: EffectSubject::Target,
                    boundary: TurnBoundary::End,
                },
                frequency: EffectTriggerFrequency::EveryOccurrence,
                payload: EffectTriggerPayload::SavingThrow {
                    ability: *ability,
                    dc,
                    on_success: EffectSaveEnd::TargetEffect,
                    on_failure: EffectSaveEnd::None,
                },
            }]
        } else {
            vec![]
        },
    }))
}
