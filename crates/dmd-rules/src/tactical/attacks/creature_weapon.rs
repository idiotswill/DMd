//! Printed source weapon actions retain the ordinary physical reservation/receipt.
use super::*;
use crate::tactical_creatures::{
    CreatureActionCost, CreatureScheduleOperation, OgreWeaponProgram, apply_creature_schedule,
    ogre_weapon_program,
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
    /// Only this sealed exact source may differ in base dice. Old sources retain
    /// their strict complete-formula comparison.
    ogre: Option<OgreWeaponProgram>,
}

pub(in crate::tactical) fn require_ogre_execution(state: &CampaignState) -> Result<(), RulesError> {
    if flow(state)?.version != TacticalExecutionVersion::EncounterReleaseV1.flow_version() {
        return Err(prerequisite(
            "physical Ogre attacks require the current executor",
        ));
    }
    Ok(())
}

fn ogre_facts(program: OgreWeaponProgram) -> SourceWeaponFacts {
    SourceWeaponFacts {
        pin: program.source().clone(),
        delivery: program.delivery(),
        bonus: i32::from(program.attack_bonus()),
        base: component(program.printed_base_damage()),
        advantage_damage: vec![],
        reach: match program.source_delivery() {
            AttackDelivery::Melee { reach_feet } => Some(u32::from(*reach_feet) * 2),
            AttackDelivery::Ranged { .. } => None,
        },
        abilities: vec![program.ability()],
        ogre: Some(program),
    }
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
    if source.id == "ogre" {
        // This is a producer and retained-source boundary under either policy,
        // independent of the top-level Live-only legacy-action allowlist.
        require_ogre_execution(state)?;
        return ogre_weapon_program(&profile.source, feature_id)
            .map(ogre_facts)
            .map_err(|error| invalid(&error.to_string()));
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
        ogre: None,
    })
}

/// Normal-weapon calculations remain ordinary. Only the sealed Ogre program may
/// substitute printed base dice after all remaining source facts match exactly.
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
    require_matching_facts(facts, plan, entity.exhaustion)
}

fn require_matching_facts(
    facts: &SourceWeaponFacts,
    plan: &WeaponAttackPlan,
    exhaustion: u8,
) -> Result<(), RulesError> {
    let expected_dice = match &facts.ogre {
        Some(program) => {
            let grip_matches = match program.weapon().hands {
                crate::tactical_definitions::WeaponHands::Two => {
                    plan.receipt.grip == WeaponGrip::TwoHands
                }
                crate::tactical_definitions::WeaponHands::One => {
                    matches!(plan.receipt.grip, WeaponGrip::OneHand(_))
                }
                _ => false,
            };
            if plan.receipt.definition_id != program.weapon().id
                || !grip_matches
                || plan.ammunition.is_some()
                || plan.receipt.ammunition.is_some()
                || plan.thrown_weapon
                    != (program.delivery() == WeaponDelivery::Thrown).then_some(plan.receipt.weapon)
            {
                return Err(prerequisite(
                    "Ogre form differs from its physical weapon plan",
                ));
            }
            &program.weapon().damage.dice
        }
        None => &facts.base.dice,
    };
    if plan.attack_modifier != facts.bonus - i32::from(exhaustion) * 2
        || plan.damage.damage_type != facts.base.damage_type
        || &plan.damage.dice != expected_dice
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

/// New physical OA is bounded to exact Ogre melee forms. It does not invoke the
/// Action scheduler or spend anything; the shared OA caller owns Reaction cost.
pub(super) fn opportunity_source(
    state: &CampaignState,
    actor: EntityId,
    target: EntityId,
    meta: &CommandMeta,
    feature_id: &str,
    item: ItemId,
    grip: WeaponGrip,
) -> Result<TacticalAttackSource, RulesError> {
    require_ogre_execution(state)?;
    let facts = facts(state, actor, feature_id, item)?;
    if facts.ogre.is_none() || facts.delivery != WeaponDelivery::Melee {
        return Err(prerequisite(
            "physical source opportunity requires an Ogre melee form",
        ));
    }
    let equipment = state
        .rules
        .as_ref()
        .and_then(|rules| rules.tactical_inventory.as_ref())
        .and_then(|inventory| inventory.loadout(actor))
        .cloned()
        .ok_or_else(|| prerequisite("current physical equipment required"))?;
    Ok(TacticalAttackSource::CreatureWeapon {
        source: facts.pin,
        feature_id: feature_id.into(),
        weapon: Box::new(TacticalWeaponAttack {
            choice: WeaponUseChoice {
                weapon: item,
                target,
                delivery: WeaponDelivery::Melee,
                ability: Ability::Strength,
                grip,
                purpose: WeaponAttackPurpose::Normal,
                ammunition: None,
                equipment_change: None,
            },
            window: WeaponActionWindow {
                id: meta.id,
                kind: WeaponActionKind::Reaction,
            },
            equipment_before: equipment,
            ammunition: None,
        }),
    })
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
    planning::admit_target(state, actor, selected.target)?;
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
    let expected_kind = match &attack.admission {
        TacticalAttackAdmission::CreatureAction { approach: None } => {
            WeaponActionKind::AttackAction
        }
        TacticalAttackAdmission::Opportunity(_)
            if facts.ogre.is_some()
                && facts.delivery == WeaponDelivery::Melee
                && weapon.choice.equipment_change.is_none()
                && weapon.choice.ammunition.is_none()
                && weapon.ammunition.is_none() =>
        {
            WeaponActionKind::Reaction
        }
        _ => {
            return Err(invalid(
                "source weapon lacks its exact Action or physical Reaction admission",
            ));
        }
    };
    if source != &facts.pin
        || weapon.window
            != (WeaponActionWindow {
                id: attack.origin.id,
                kind: expected_kind,
            })
    {
        return Err(invalid("source weapon pin or physical admission differs"));
    }
    require_matching_plan(state, attack.actor, &facts, plan)?;
    let (mode, _, _) = planning::hit_facts(state, attack.actor, &weapon.choice, plan)?;
    Ok(source_damage(&facts, mode))
}

#[cfg(test)]
mod ogre_tests;
