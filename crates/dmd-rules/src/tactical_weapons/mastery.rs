use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum WeaponEffectExpiry {
    StartOfSourceNextTurn,
    EndOfSourceNextTurn,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum WeaponMasteryConsequence {
    /// Same-type fixed damage; no added dice or damage boosts may be attached (SRD90).
    GrazeDamage {
        source: EntityId,
        target: EntityId,
        amount: u32,
        damage_type: DamageType,
    },
    PushAway {
        source: EntityId,
        target: EntityId,
        distance: u32,
    },
    /// Repeated Slow never increases this reduction; resolver uses the greatest value.
    Slow {
        source: EntityId,
        target: EntityId,
        reduction: u32,
        expires: WeaponEffectExpiry,
    },
    ToppleSave {
        source: EntityId,
        target: EntityId,
        ability: Ability,
        dc: i32,
        on_failure: Condition,
    },
    Sap {
        source: EntityId,
        target: EntityId,
        expires: WeaponEffectExpiry,
    },
    Vex {
        source: EntityId,
        target: EntityId,
        expires: WeaponEffectExpiry,
    },
    /// This is still a player choice/proposal, not authorization. Prepare it against
    /// current geometry/equipment/history before committing the continuation.
    CleaveAttack { attack: WeaponUseChoice },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum WeaponMasteryOffer {
    Graze {
        source: EntityId,
        target: EntityId,
        amount: u32,
        damage_type: DamageType,
    },
    Push {
        source: EntityId,
        target: EntityId,
        maximum: u32,
    },
    Slow {
        source: EntityId,
        target: EntityId,
    },
    Topple {
        source: EntityId,
        target: EntityId,
        dc: i32,
    },
    Cleave {
        source: EntityId,
        first_target: EntityId,
        weapon: ItemId,
        trigger: CommandId,
    },
}
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
pub struct WeaponMasteryResolution {
    pub mandatory: Vec<WeaponMasteryConsequence>,
    /// Requires the controlling player/DM's recorded choice, including explicit decline.
    pub offer: Option<WeaponMasteryOffer>,
}
/// Derived from the accepted target and the current turn's persisted receipts.
pub struct WeaponMasteryContext<'a> {
    pub target_is_creature: bool,
    pub target_size: CreatureSize,
    pub history: &'a [WeaponAttackReceipt],
}
pub fn weapon_mastery_resolution(
    plan: &WeaponAttackPlan,
    outcome: WeaponAttackOutcome,
    context: &WeaponMasteryContext<'_>,
) -> Result<WeaponMasteryResolution, WeaponError> {
    require(
        outcome != WeaponAttackOutcome::Pending,
        "mastery awaits resolved attack outcome",
    )?;
    require(
        !(plan.automatic_miss && matches!(outcome, WeaponAttackOutcome::Hit { .. })),
        "an automatic miss cannot trigger a hit mastery",
    )?;
    require(context.history.len() <= 128, "unbounded mastery history")?;
    let mut result = WeaponMasteryResolution::default();
    let Some(mastery) = plan.mastery else {
        return Ok(result);
    };
    if !context.target_is_creature {
        return Ok(result);
    }
    let source = plan.receipt.actor;
    let target = plan.receipt.target;
    if outcome == WeaponAttackOutcome::Miss {
        if mastery == WeaponMastery::Graze {
            result.offer = Some(WeaponMasteryOffer::Graze {
                source,
                target,
                amount: plan.ability_modifier.max(0) as u32,
                damage_type: plan.damage.damage_type,
            });
        }
        return Ok(result);
    }
    let WeaponAttackOutcome::Hit { damage_dealt, .. } = outcome else {
        unreachable!("pending rejected")
    };
    match mastery {
        WeaponMastery::Cleave
            if plan.receipt.delivery == WeaponDelivery::Melee
                && !matches!(plan.receipt.purpose, WeaponAttackPurpose::Cleave { .. })
                && !context.history.iter().any(|receipt| {
                    receipt.actor == source
                        && receipt.turn_number == plan.receipt.turn_number
                        && matches!(receipt.purpose, WeaponAttackPurpose::Cleave { .. })
                }) =>
        {
            result.offer = Some(WeaponMasteryOffer::Cleave {
                source,
                first_target: target,
                weapon: plan.receipt.weapon,
                trigger: plan.receipt.origin.id,
            });
        }
        WeaponMastery::Push if context.target_size.rank() <= CreatureSize::Large.rank() => {
            result.offer = Some(WeaponMasteryOffer::Push {
                source,
                target,
                maximum: 20,
            });
        }
        WeaponMastery::Sap => result.mandatory.push(WeaponMasteryConsequence::Sap {
            source,
            target,
            expires: WeaponEffectExpiry::StartOfSourceNextTurn,
        }),
        WeaponMastery::Slow if damage_dealt > 0 => {
            result.offer = Some(WeaponMasteryOffer::Slow { source, target })
        }
        WeaponMastery::Topple => {
            result.offer = Some(WeaponMasteryOffer::Topple {
                source,
                target,
                dc: 8 + plan.ability_modifier + plan.proficiency_bonus,
            })
        }
        WeaponMastery::Vex if damage_dealt > 0 => {
            result.mandatory.push(WeaponMasteryConsequence::Vex {
                source,
                target,
                expires: WeaponEffectExpiry::EndOfSourceNextTurn,
            })
        }
        _ => {}
    }
    Ok(result)
}
pub fn choose_weapon_mastery(
    offer: &WeaponMasteryOffer,
    choice: &WeaponMasteryChoice,
) -> Result<Option<WeaponMasteryConsequence>, WeaponError> {
    if *choice == WeaponMasteryChoice::Decline {
        return Ok(None);
    }
    let consequence = match (offer, choice) {
        (
            WeaponMasteryOffer::Graze {
                source,
                target,
                amount,
                damage_type,
            },
            WeaponMasteryChoice::Graze,
        ) => WeaponMasteryConsequence::GrazeDamage {
            source: *source,
            target: *target,
            amount: *amount,
            damage_type: *damage_type,
        },
        (
            WeaponMasteryOffer::Push {
                source,
                target,
                maximum,
            },
            WeaponMasteryChoice::Push { distance },
        ) => {
            require(
                *distance > 0 && distance <= maximum,
                "push exceeds the source choice limit",
            )?;
            WeaponMasteryConsequence::PushAway {
                source: *source,
                target: *target,
                distance: *distance,
            }
        }
        (WeaponMasteryOffer::Slow { source, target }, WeaponMasteryChoice::Slow) => {
            WeaponMasteryConsequence::Slow {
                source: *source,
                target: *target,
                reduction: 20,
                expires: WeaponEffectExpiry::StartOfSourceNextTurn,
            }
        }
        (WeaponMasteryOffer::Topple { source, target, dc }, WeaponMasteryChoice::Topple) => {
            WeaponMasteryConsequence::ToppleSave {
                source: *source,
                target: *target,
                dc: *dc,
                ability: Ability::Constitution,
                on_failure: Condition::Prone,
            }
        }
        (
            WeaponMasteryOffer::Cleave {
                first_target,
                weapon,
                trigger,
                ..
            },
            WeaponMasteryChoice::Cleave { attack },
        ) => {
            require(
                attack.weapon == *weapon
                    && attack.target != *first_target
                    && attack.delivery == WeaponDelivery::Melee
                    && attack.purpose == WeaponAttackPurpose::Cleave { trigger: *trigger },
                "Cleave choice must retain its exact trigger and physical weapon",
            )?;
            WeaponMasteryConsequence::CleaveAttack {
                attack: *attack.clone(),
            }
        }
        _ => return Err(illegal("choice does not match the pending mastery offer")),
    };
    Ok(Some(consequence))
}
