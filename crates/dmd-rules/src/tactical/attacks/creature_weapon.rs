//! Printed source weapon actions retain the ordinary physical reservation/receipt.
use super::*;
use crate::tactical_creatures::{
    CreatureActionCost, CreatureScheduleOperation, apply_creature_schedule,
};
use crate::tactical_definitions::{
    AttackDelivery, FeatureActivation, MonsterFeature, WeaponKind, WeaponProperty,
};

struct SourceWeaponFacts {
    pin: CreatureSourcePin,
    delivery: WeaponDelivery,
    bonus: i32,
    base: AttackDamageComponent,
    advantage_damage: Vec<AttackDamageComponent>,
    reach: Option<u32>,
    abilities: Vec<Ability>,
}

pub(super) struct PreparedCreatureWeapon {
    source: CreatureSourcePin,
    feature_id: String,
    pub(super) next_creatures: TacticalCreatures,
}

impl PreparedCreatureWeapon {
    pub(super) fn attack_source(&self, weapon: Box<TacticalWeaponAttack>) -> TacticalAttackSource {
        TacticalAttackSource::CreatureWeapon {
            source: self.source.clone(),
            feature_id: self.feature_id.clone(),
            weapon,
        }
    }

    pub(super) fn damage(
        &self,
        state: &CampaignState,
        actor: EntityId,
        plan: &WeaponAttackPlan,
        mode: RollMode,
    ) -> Result<Vec<AttackDamageComponent>, RulesError> {
        let facts = facts(state, actor, &self.feature_id, plan.receipt.weapon)?;
        if facts.pin != self.source {
            return Err(invalid("source weapon pin differs"));
        }
        require_matching_plan(state, actor, &facts, plan)?;
        Ok(source_damage(&facts, mode))
    }
}

fn component(source: &crate::tactical_definitions::DamageComponent) -> AttackDamageComponent {
    AttackDamageComponent {
        damage_type: source.damage_type,
        dice: source.amount.dice.clone(),
        modifier: i32::from(source.amount.fixed),
    }
}

fn facts(
    state: &CampaignState,
    actor: EntityId,
    feature_id: &str,
    item: ItemId,
) -> Result<SourceWeaponFacts, RulesError> {
    let profile = intrinsic::profile(state, actor)?;
    let source = crate::tactical_creatures::source_for_profile(profile)
        .map_err(|e| invalid(&e.to_string()))?;
    let required = crate::tactical_creature_equipment::creature_attack_gear(profile, feature_id)?
        .ok_or_else(|| {
        prerequisite("intrinsic source attack does not use a physical weapon")
    })?;
    if state
        .items
        .get(&item)
        .is_none_or(|item| item.definition_id != required)
    {
        return Err(prerequisite(
            "select the actual weapon required by this source feature",
        ));
    }
    let feature = source
        .features
        .iter()
        .find(|feature| feature.id == feature_id)
        .ok_or_else(|| invalid("source weapon feature absent"))?;
    let MonsterFeature::Attack {
        bonus,
        delivery,
        damage,
        extra_damage_if_attack_had_advantage,
        conditional_hits,
    } = &feature.feature
    else {
        return Err(prerequisite("source feature is not a weapon attack"));
    };
    if feature.activation != FeatureActivation::Action
        || feature.usage.is_some()
        || damage.len() != 1
        || !conditional_hits.is_empty()
    {
        return Err(prerequisite(
            "source weapon needs its complete activation or rider continuation",
        ));
    }
    let definitions = definitions()?;
    let weapon = definitions
        .weapon(required)
        .ok_or_else(|| invalid("source physical weapon absent"))?;
    let (delivery, reach) = match delivery {
        AttackDelivery::Melee { reach_feet } if weapon.kind == WeaponKind::Melee => {
            (WeaponDelivery::Melee, Some(u32::from(*reach_feet) * 2))
        }
        AttackDelivery::Ranged { range }
            if weapon.range.as_ref() == Some(range)
                && required_ammunition_definition(weapon).is_some() =>
        {
            (WeaponDelivery::Shot, None)
        }
        _ => {
            return Err(prerequisite(
                "printed source delivery needs its physical range adapter",
            ));
        }
    };
    let abilities = if weapon.kind == WeaponKind::Melee {
        if weapon.properties.contains(&WeaponProperty::Finesse) {
            vec![Ability::Strength, Ability::Dexterity]
        } else {
            vec![Ability::Strength]
        }
    } else {
        vec![Ability::Dexterity]
    };
    Ok(SourceWeaponFacts {
        pin: profile.source.clone(),
        delivery,
        bonus: i32::from(*bonus),
        base: component(&damage[0]),
        advantage_damage: extra_damage_if_attack_had_advantage
            .iter()
            .map(component)
            .collect(),
        reach,
        abilities,
    })
}

/// Normal-weapon calculations must faithfully represent the printed base facts.
/// They never substitute an ordinary formula for a discrepant stat-block attack.
fn require_matching_plan(
    state: &CampaignState,
    actor: EntityId,
    facts: &SourceWeaponFacts,
    plan: &WeaponAttackPlan,
) -> Result<(), RulesError> {
    let entity = state
        .rules
        .as_ref()
        .and_then(|rules| rules.entities.get(&actor))
        .ok_or_else(|| invalid("source attacker mechanics absent"))?;
    if plan.attack_modifier != facts.bonus - i32::from(entity.exhaustion) * 2
        || plan.damage.damage_type != facts.base.damage_type
        || plan.damage.dice != facts.base.dice
        || plan.damage.modifier != facts.base.modifier
        || facts.reach.is_some_and(|reach| plan.reach != reach)
        || plan.mastery.is_some()
        || plan.receipt.purpose != WeaponAttackPurpose::Normal
        || plan.receipt.delivery != facts.delivery
        || !facts.abilities.contains(&plan.receipt.ability)
    {
        return Err(prerequisite(
            "printed source facts need a different physical attack adapter",
        ));
    }
    Ok(())
}

fn source_damage(facts: &SourceWeaponFacts, mode: RollMode) -> Vec<AttackDamageComponent> {
    let mut result = vec![facts.base.clone()];
    if mode == RollMode::Advantage {
        result.extend(facts.advantage_damage.clone());
    }
    result
}

pub(in crate::tactical) fn begin_creature_weapon(
    state: &mut CampaignState,
    meta: &CommandMeta,
    feature_id: &str,
    selected: &CreatureWeaponUseChoice,
    pack: &RulesPack,
) -> Result<(), RulesError> {
    if flow(state)?.phase != TacticalPhase::Active || flow(state)?.resolution.is_some() {
        return Err(RulesError::Pending);
    }
    let actor = active(state)?;
    authorize(state, meta, actor)?;
    planning::require_located_target(state, actor, selected.target)?;
    // This source path resolves creature vitality, not object/body durability.
    // Check only at admission: a committed attack can itself kill this target.
    if state
        .rules
        .as_ref()
        .and_then(|rules| rules.entities.get(&selected.target))
        .is_some_and(|target| target.death.dead)
    {
        return Err(prerequisite(
            "a dead body requires its body/object adjudication path",
        ));
    }
    let facts = facts(state, actor, feature_id, selected.weapon)?;
    let current = state
        .rules
        .as_ref()
        .and_then(|rules| rules.tactical_creatures.as_ref())
        .ok_or_else(|| prerequisite("source creature attachment is required"))?;
    let source_step = apply_creature_schedule(
        state,
        current,
        meta,
        &CreatureScheduleOperation::BeginFeature {
            actor,
            selection: CreatureFeatureSelection {
                feature_id: feature_id.into(),
                spell_id: None,
                simple_action: None,
            },
            steps: vec![],
        },
    )
    .map_err(|e| prerequisite(&e.to_string()))?;
    let feature = source_step
        .feature
        .as_ref()
        .ok_or_else(|| prerequisite("source feature needs enclosing work"))?;
    if source_step.cost != CreatureActionCost::Action
        || feature.invocation != *meta
        || feature.enclosing_activation.origin != *meta
        || !feature.enclosing_activation.attack_action
        || feature.enclosing_activation.activation != FeatureActivation::Action
        || feature.source != facts.pin
    {
        return Err(prerequisite(
            "source weapon lacks its one Action activation",
        ));
    }
    let equipment = state
        .rules
        .as_ref()
        .and_then(|rules| rules.tactical_inventory.as_ref())
        .and_then(|inventory| inventory.loadout(actor))
        .ok_or_else(|| prerequisite("source weapon requires current physical equipment"))?;
    let window = WeaponActionWindow {
        id: meta.id,
        kind: WeaponActionKind::AttackAction,
    };
    let mut selected_choice = None;
    let mut last_error = None;
    for ability in &facts.abilities {
        let choice = WeaponUseChoice {
            weapon: selected.weapon,
            target: selected.target,
            delivery: facts.delivery,
            ability: *ability,
            grip: selected.grip,
            purpose: WeaponAttackPurpose::Normal,
            ammunition: selected.ammunition,
            equipment_change: selected.equipment_change,
        };
        match planning::weapon_plan(state, meta, actor, &choice, window, &equipment.hands, pack)
            .and_then(|plan| require_matching_plan(state, actor, &facts, &plan))
        {
            Ok(()) => {
                selected_choice = Some(choice);
                break;
            }
            Err(error) => last_error = Some(error),
        }
    }
    let choice = selected_choice
        .ok_or_else(|| last_error.unwrap_or_else(|| prerequisite("no source weapon plan")))?;
    super::begin_with_source(
        state,
        meta,
        &choice,
        pack,
        Some(PreparedCreatureWeapon {
            source: facts.pin,
            feature_id: feature_id.into(),
            next_creatures: source_step.next,
        }),
    )
}

pub(super) fn validate_source(
    state: &CampaignState,
    attack: &TacticalAttack,
    plan: &WeaponAttackPlan,
) -> Result<Vec<AttackDamageComponent>, RulesError> {
    let TacticalAttackSource::CreatureWeapon {
        source,
        feature_id,
        weapon,
    } = &attack.source
    else {
        return Err(invalid("retained attack is not a source physical weapon"));
    };
    let facts = facts(state, attack.actor, feature_id, weapon.choice.weapon)?;
    if source != &facts.pin
        || !matches!(
            attack.admission,
            TacticalAttackAdmission::CreatureAction { approach: None }
        )
        || weapon.window
            != (WeaponActionWindow {
                id: attack.origin.id,
                kind: WeaponActionKind::AttackAction,
            })
    {
        return Err(invalid("source weapon pin or Action admission differs"));
    }
    require_matching_plan(state, attack.actor, &facts, plan)?;
    let (mode, _, _) = planning::hit_facts(state, attack.actor, &weapon.choice, plan)?;
    Ok(source_damage(&facts, mode))
}
