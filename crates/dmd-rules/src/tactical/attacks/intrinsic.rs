//! Non-item attacks derive every number from the actor's authenticated source.
use super::*;
use crate::tactical_definitions::{
    AttackDelivery, EffectDescriptor, HitRequirement, MonsterFeature, SourceSize,
};

pub(super) struct IntrinsicPlan {
    pub modifier: i32,
    pub damage: Vec<AttackDamageComponent>,
    pub mode: RollMode,
    pub armor: i32,
    pub critical: bool,
    pub prone: bool,
}

pub(super) fn profile(
    state: &CampaignState,
    actor: EntityId,
) -> Result<&CreatureProfile, RulesError> {
    state
        .rules
        .as_ref()
        .and_then(|r| r.tactical_creatures.as_ref())
        .and_then(|c| c.profile(actor))
        .ok_or_else(|| prerequisite("a pinned creature profile is required"))
}
pub(super) fn held(state: &CampaignState, actor: EntityId, item: ItemId) -> bool {
    state.items.get(&item).is_some_and(|i| {
        i.id == item
            && i.campaign_id == state.campaign_id()
            && i.custody == Custody::Entity(actor)
            && i.quantity == 1
            && i.state == ItemState::Intact
    }) && state
        .rules
        .as_ref()
        .and_then(|r| r.tactical_inventory.as_ref())
        .and_then(|i| i.loadout(actor))
        .is_some_and(|l| l.hands.hands.contains(&HandAssignment::Item(item)))
}
fn size(source: SourceSize) -> CreatureSize {
    match source {
        SourceSize::Tiny => CreatureSize::Tiny,
        SourceSize::Small => CreatureSize::Small,
        SourceSize::Medium => CreatureSize::Medium,
        SourceSize::Large => CreatureSize::Large,
        SourceSize::Huge => CreatureSize::Huge,
        SourceSize::Gargantuan => CreatureSize::Gargantuan,
    }
}
fn damage(value: &crate::tactical_definitions::DamageComponent) -> AttackDamageComponent {
    AttackDamageComponent {
        damage_type: value.damage_type,
        dice: value.amount.dice.clone(),
        modifier: i32::from(value.amount.fixed),
    }
}

pub(super) fn plan(
    state: &CampaignState,
    attack: &TacticalAttack,
) -> Result<IntrinsicPlan, RulesError> {
    planning::require_located_target(state, attack.actor, attack.target)?;
    let rules = state.rules.as_ref().ok_or(RulesError::Uninitialized)?;
    let entity = rules
        .entities
        .get(&attack.actor)
        .ok_or_else(|| invalid("attacker mechanics absent"))?;
    let encounter = encounter(state)?;
    let actor = encounter
        .participant(attack.actor)
        .ok_or_else(|| invalid("attacker absent"))?;
    let target = encounter
        .participant(attack.target)
        .ok_or_else(|| invalid("target absent"))?;
    let distance =
        crate::spatial::participant_distance(actor, target).map_err(|e| invalid(&e.to_string()))?;
    let (mode, armor, critical) =
        planning::hit_facts_for(state, attack.actor, attack.target, false, false)?;
    let mut prone = false;
    let (modifier, components, reach) = match &attack.source {
        TacticalAttackSource::Unarmed { ability } => {
            if *ability != Ability::Strength
                || !matches!(attack.admission, TacticalAttackAdmission::Opportunity(_))
            {
                return Err(prerequisite(
                    "this source grants only Strength Unarmed Strikes",
                ));
            }
            let proficiency = if let Ok(profile) = profile(state, attack.actor) {
                i32::from(
                    crate::tactical_creatures::creature_proficiency_bonus(profile)
                        .map_err(|e| invalid(&e.to_string()))?,
                )
            } else {
                if !flow(state)?
                    .combatants
                    .iter()
                    .any(|c| c.actor == attack.actor && c.source == TacticalSource::Character)
                {
                    return Err(prerequisite(
                        "unarmed attack requires an authenticated actor source",
                    ));
                }
                crate::proficiency_bonus(entity.level)
            };
            let strength =
                crate::ability_modifier(entity.ability_scores[Ability::Strength.index()]);
            (
                strength + proficiency - i32::from(entity.exhaustion) * 2,
                vec![AttackDamageComponent {
                    damage_type: DamageType::Bludgeoning,
                    dice: vec![],
                    modifier: 1 + strength,
                }],
                10,
            )
        }
        TacticalAttackSource::CreatureFeature {
            source,
            feature_id,
            weapon,
        } => {
            let profile = profile(state, attack.actor)?;
            if source != &profile.source {
                return Err(invalid("attack creature source pin differs"));
            }
            let definition = crate::tactical_creatures::source_for_profile(profile)
                .map_err(|e| invalid(&e.to_string()))?;
            let feature = definition
                .features
                .iter()
                .find(|f| f.id == *feature_id)
                .ok_or_else(|| invalid("source attack absent"))?;
            if feature.usage.is_some() {
                return Err(prerequisite(
                    "limited source attack needs its feature-use continuation",
                ));
            }
            let MonsterFeature::Attack {
                bonus,
                delivery: AttackDelivery::Melee { reach_feet },
                damage: base,
                extra_damage_if_attack_had_advantage,
                conditional_hits,
            } = &feature.feature
            else {
                return Err(prerequisite(
                    "this continuation requires a source melee attack",
                ));
            };
            let required =
                crate::tactical_creature_equipment::creature_attack_gear(profile, feature_id)?;
            match (required, weapon) {
                (Some(id), Some(item))
                    if held(state, attack.actor, *item)
                        && state.items[item].definition_id == id => {}
                (None, None) => (),
                _ => {
                    return Err(prerequisite(
                        "source attack requires its actual held implement",
                    ));
                }
            }
            let mut components: Vec<_> = base.iter().map(damage).collect();
            if mode == RollMode::Advantage {
                components.extend(extra_damage_if_attack_had_advantage.iter().map(damage));
            }
            for conditional in conditional_hits {
                let applies = match conditional.requirement {
                    HitRequirement::TargetSizeAtMost { size: maximum } => {
                        target.size <= size(maximum)
                    }
                    HitRequirement::Charge {
                        straight_feet,
                        target_size_at_most,
                    } => {
                        target.size <= size(target_size_at_most)
                            && creature::approach_distance(state, attack)?
                                .is_some_and(|distance| distance >= u32::from(straight_feet) * 2)
                    }
                };
                for effect in &conditional.effects {
                    match effect {
                        EffectDescriptor::Condition {
                            condition: Condition::Prone,
                        } => prone |= applies,
                        EffectDescriptor::Damage { damage: extra } => {
                            if applies {
                                components.push(damage(extra));
                            }
                        }
                        _ => {
                            return Err(prerequisite(
                                "source hit effect needs its typed continuation",
                            ));
                        }
                    }
                }
            }
            (
                i32::from(*bonus) - i32::from(entity.exhaustion) * 2,
                components,
                u32::from(*reach_feet) * 2,
            )
        }
        TacticalAttackSource::Weapon(_)
        | TacticalAttackSource::CreatureWeapon { .. }
        | TacticalAttackSource::Spell { .. } => {
            return Err(invalid("attack is not an intrinsic melee source"));
        }
    };
    if distance > reach
        || attack.delivery != TacticalAttackDelivery::Melee
        || !matches!(
            attack.admission,
            TacticalAttackAdmission::Opportunity(_)
                | TacticalAttackAdmission::CreatureAction { .. }
        )
    {
        return Err(prerequisite(
            "intrinsic melee attack differs from its retained source admission",
        ));
    }
    Ok(IntrinsicPlan {
        modifier,
        damage: components,
        mode,
        armor,
        critical,
        prone,
    })
}

pub(super) fn complete(
    state: &mut CampaignState,
    attack: &TacticalAttack,
    outcome: WeaponAttackOutcome,
) -> Result<(), RulesError> {
    let plan = plan(state, attack)?;
    if matches!(outcome, WeaponAttackOutcome::Hit { .. }) && plan.prone {
        let target = state
            .rules
            .as_mut()
            .ok_or(RulesError::Uninitialized)?
            .entities
            .get_mut(&attack.target)
            .ok_or_else(|| invalid("target mechanics absent"))?;
        if !target.condition_immunities.contains(&Condition::Prone) {
            target.prone = true;
        }
    }
    Ok(())
}
