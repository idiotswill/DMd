//! A selected movement crossing grants one reaction attack, never another action.
use super::*;
use crate::tactical_definitions::{
    AttackDelivery, MonsterFeature, WeaponHands, WeaponKind, WeaponProperty,
};

/// Departure distance is derived by the movement evaluator, never a controller
/// modifier. Irrelevant reach options must not consult uncertain cover at all.
pub(in crate::tactical) fn opportunity_options_for_crossing(
    state: &CampaignState,
    actor: EntityId,
    mover: EntityId,
    after_distance: u32,
) -> Result<Vec<TacticalMeleeOption>, RulesError> {
    // A capability query must not reveal geometry behind an unknown truth ID.
    if planning::require_located_target(state, actor, mover).is_err() {
        return Ok(vec![]);
    }
    let rules = state.rules.as_ref().ok_or(RulesError::Uninitialized)?;
    if !crate::tactical_conditions::can_act(rules, actor)?
        || !crate::tactical_conditions::may_harm(rules, actor, mover)
    {
        return Ok(vec![]);
    }
    let participant = encounter(state)?
        .participant(actor)
        .ok_or_else(|| invalid("reactor absent"))?;
    let target = encounter(state)?
        .participant(mover)
        .ok_or_else(|| invalid("mover absent"))?;
    let definitions = definitions()?;
    let loadout = rules
        .tactical_inventory
        .as_ref()
        .and_then(|i| i.loadout(actor));
    let mut result = vec![TacticalMeleeOption {
        source: TacticalMeleeSource::Unarmed,
        reach: 10,
    }];
    if let Some(loadout) = loadout {
        let mut held = loadout
            .hands
            .hands
            .iter()
            .filter_map(|h| {
                if let HandAssignment::Item(id) = h {
                    Some(*id)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        held.sort_by_key(|id| id.0);
        held.dedup();
        for item in held {
            if !intrinsic::held(state, actor, item) {
                continue;
            }
            let Some(weapon) = definitions.weapon(&state.items[&item].definition_id) else {
                continue;
            };
            if weapon.kind != WeaponKind::Melee
                || (weapon.hands != WeaponHands::One
                    && !loadout
                        .hands
                        .hands
                        .iter()
                        .all(|h| *h == HandAssignment::Item(item) || *h == HandAssignment::Free))
            {
                continue;
            }
            result.push(TacticalMeleeOption {
                source: TacticalMeleeSource::Weapon { item },
                reach: participant.reach
                    + if weapon.properties.contains(&WeaponProperty::Reach) {
                        10
                    } else {
                        0
                    },
            });
        }
    }
    if let Ok(profile) = intrinsic::profile(state, actor) {
        let source = crate::tactical_creatures::source_for_profile(profile)
            .map_err(|e| invalid(&e.to_string()))?;
        let mut features = source.features.iter().collect::<Vec<_>>();
        features.sort_by(|a, b| a.id.cmp(&b.id));
        for feature in features {
            let MonsterFeature::Attack {
                delivery: AttackDelivery::Melee { reach_feet },
                ..
            } = feature.feature
            else {
                continue;
            };
            if feature.usage.is_some() {
                continue;
            }
            let required =
                crate::tactical_creature_equipment::creature_attack_gear(profile, &feature.id)?;
            let items = match required {
                None => vec![None],
                Some(definition) => {
                    let mut items = state
                        .items
                        .values()
                        .filter(|item| {
                            item.definition_id == definition
                                && intrinsic::held(state, actor, item.id)
                        })
                        .map(|item| Some(item.id))
                        .collect::<Vec<_>>();
                    items.sort_by_key(|id| id.map(|id| id.0));
                    items
                }
            };
            result.extend(items.into_iter().map(|weapon| TacticalMeleeOption {
                source: TacticalMeleeSource::CreatureFeature {
                    feature_id: feature.id.clone(),
                    weapon,
                },
                reach: u32::from(reach_feet) * 2,
            }));
        }
    }
    let distance = crate::spatial::participant_distance(participant, target)
        .map_err(|e| invalid(&e.to_string()))?;
    result.retain(|option| distance <= option.reach && option.reach < after_distance);
    if result.is_empty() {
        return Ok(result);
    }
    let cover = crate::spatial::cover_from(
        encounter(state)?,
        participant.center().map_err(|e| invalid(&e))?,
        target.volume().map_err(|e| invalid(&e))?,
        &[actor, mover],
    )
    .map_err(|e| invalid(&e.to_string()))?;
    if cover.requires_adjudication {
        return Err(prerequisite(
            "opportunity attack cover needs an authored geometry ruling",
        ));
    }
    if cover.degree == CoverDegree::Total {
        return Ok(vec![]);
    }
    // Unarmed, then physical ItemIds, then canonical feature and implement IDs.
    // Each input group is explicitly sorted; HashMap iteration never sets replay order.
    Ok(result)
}

pub(in crate::tactical) fn begin_opportunity_attack(
    state: &mut CampaignState,
    meta: &CommandMeta,
    actor: EntityId,
    target: EntityId,
    choice: &TacticalMeleeChoice,
    pack: &RulesPack,
) -> Result<(), RulesError> {
    authorize(state, meta, actor)?;
    planning::require_located_target(state, actor, target)?;
    let window = super::super::movement::validate_opportunity(state, actor, target)?.clone();
    let selected = match choice {
        TacticalMeleeChoice::Weapon(c) => {
            if c.target != target
                || c.delivery != WeaponDelivery::Melee
                || c.purpose != WeaponAttackPurpose::Normal
                || c.equipment_change.is_some()
                || c.ammunition.is_some()
            {
                return Err(prerequisite(
                    "opportunity attack uses one currently held melee weapon without free equipment changes",
                ));
            }
            TacticalMeleeSource::Weapon { item: c.weapon }
        }
        TacticalMeleeChoice::UnarmedDamage { .. } => TacticalMeleeSource::Unarmed,
        TacticalMeleeChoice::CreatureFeature { feature_id, weapon } => {
            TacticalMeleeSource::CreatureFeature {
                feature_id: feature_id.clone(),
                weapon: *weapon,
            }
        }
    };
    if !window.options.iter().any(|o| o.source == selected) {
        return Err(prerequisite(
            "selected implement did not trigger this crossing",
        ));
    }
    let source = match choice {
        TacticalMeleeChoice::Weapon(c) => {
            TacticalAttackSource::Weapon(Box::new(TacticalWeaponAttack {
                choice: c.clone(),
                window: WeaponActionWindow {
                    id: meta.id,
                    kind: WeaponActionKind::Reaction,
                },
                equipment_before: state
                    .rules
                    .as_ref()
                    .and_then(|r| r.tactical_inventory.as_ref())
                    .and_then(|i| i.loadout(actor))
                    .cloned()
                    .ok_or_else(|| prerequisite("current equipment required"))?,
                ammunition: None,
            }))
        }
        TacticalMeleeChoice::UnarmedDamage { ability } => {
            TacticalAttackSource::Unarmed { ability: *ability }
        }
        TacticalMeleeChoice::CreatureFeature { feature_id, weapon } => {
            TacticalAttackSource::CreatureFeature {
                source: intrinsic::profile(state, actor)?.source.clone(),
                feature_id: feature_id.clone(),
                weapon: *weapon,
            }
        }
    };
    let mut attack = TacticalAttack {
        origin: meta.clone(),
        actor,
        target,
        delivery: TacticalAttackDelivery::Melee,
        source,
        admission: TacticalAttackAdmission::Opportunity(Box::new(window)),
        attack_modifier: 0,
        mode: RollMode::Normal,
        armor_class: 0,
        critical_on_hit: false,
        automatic_miss: false,
        damage: vec![],
        stage: TacticalAttackStage::AttackRoll,
        attack_roll: None,
        damage_roll: None,
        outcome: None,
    };
    let plan = if let Some(weapon) = attack.weapon() {
        let plan = planning::weapon_plan(
            state,
            meta,
            actor,
            &weapon.choice,
            weapon.window,
            &weapon.equipment_before.hands,
            pack,
        )?;
        if plan
            .mastery
            .is_some_and(|m| !matches!(m, WeaponMastery::Nick | WeaponMastery::Graze))
        {
            return Err(prerequisite(
                "this mastery requires its typed continuation before any attack cost",
            ));
        }
        let (mode, armor, critical) = planning::hit_facts(state, actor, &weapon.choice, &plan)?;
        attack.attack_modifier = plan.attack_modifier;
        attack.mode = mode;
        attack.armor_class = armor;
        attack.critical_on_hit = critical;
        attack.damage = vec![AttackDamageComponent {
            damage_type: plan.damage.damage_type,
            dice: plan.damage.dice.clone(),
            modifier: plan.damage.modifier,
        }];
        Some(plan)
    } else {
        let plan = intrinsic::plan(state, &attack)?;
        attack.attack_modifier = plan.modifier;
        attack.mode = plan.mode;
        attack.armor_class = plan.armor;
        attack.critical_on_hit = plan.critical;
        attack.damage = plan.damage;
        None
    };
    let rules = state.rules.as_mut().ok_or(RulesError::Uninitialized)?;
    crate::tactical_budget::spend_cost(
        rules,
        actor,
        crate::tactical_budget::TacticalCost::Reaction,
    )?;
    crate::kernel::interrupt_rest(rules, actor, state.clock.now);
    if let Some(plan) = plan {
        let loadout = rules
            .tactical_inventory
            .as_mut()
            .and_then(|i| i.loadouts.iter_mut().find(|l| l.actor == actor))
            .ok_or_else(|| invalid("equipment absent"))?;
        loadout.hands = plan.loadout_for_attack;
        loadout.command = meta.clone();
        flow_mut(state)?.budget.weapon_history.push(plan.receipt);
    }
    super::super::movement::record_attack(state, meta, actor)?;
    resolution_mut(state)?.attack = Some(attack);
    push_frame(state, vec![TacticalWorkKind::AttackRoll])?;
    pump(state, meta)
}

pub(super) fn validate_admission(
    state: &CampaignState,
    attack: &TacticalAttack,
) -> Result<(), RulesError> {
    let resolution = resolution(state)?;
    match &attack.admission {
        TacticalAttackAdmission::OwnTurn => {
            if attack.origin != resolution.origin
                || attack.actor != resolution.turn_actor
                || resolution.movement.is_some()
                || attack.weapon().is_none()
            {
                return Err(invalid("ordinary attack differs from its own-turn origin"));
            }
        }
        TacticalAttackAdmission::Opportunity(window) => {
            validate_equipment_change_origin(state, &window.origin, window.reactor)
                .map_err(|e| invalid(&e))?;
            let rules = state.rules.as_ref().ok_or(RulesError::Uninitialized)?;
            let movement = resolution
                .movement
                .as_ref()
                .ok_or_else(|| invalid("reaction attack lacks movement parent"))?;
            if window.reactor != attack.actor
                || window.mover != attack.target
                || movement.actor != attack.target
                || !rules
                    .timing
                    .as_ref()
                    .is_some_and(|t| t.reactions_spent.contains(&attack.actor))
                || !movement.decisions.iter().any(|d| {
                    d.reactor == attack.actor
                        && d.origin == attack.origin
                        && d.kind == TacticalOpportunityDecisionKind::Attack
                })
                || window.origin.expected_event_sequence > attack.origin.expected_event_sequence
                || window.origin.expected_event_sequence < movement.origin.expected_event_sequence
            {
                return Err(invalid(
                    "reaction attack differs from accepted crossing/response/cost",
                ));
            }
            let mut before = state.clone();
            before
                .rules
                .as_mut()
                .unwrap()
                .timing
                .as_mut()
                .unwrap()
                .reactions_spent
                .retain(|id| *id != attack.actor);
            let r = resolution_mut(&mut before)?;
            r.attack = None;
            r.pending = None;
            r.movement.as_mut().unwrap().opportunity = Some(window.as_ref().clone());
            // Equipping a two-handed grip can change hand assignment but does not
            // change which held implement triggered. Restore the accepted before-image.
            if let Some(weapon) = attack.weapon() {
                let loadout = before
                    .rules
                    .as_mut()
                    .and_then(|r| r.tactical_inventory.as_mut())
                    .and_then(|i| i.loadouts.iter_mut().find(|l| l.actor == attack.actor))
                    .ok_or_else(|| invalid("reaction before-equipment absent"))?;
                *loadout = weapon.equipment_before.clone();
            }
            super::super::movement::validate_opportunity(&before, attack.actor, attack.target)?;
            let source = match &attack.source {
                TacticalAttackSource::Weapon(w) => TacticalMeleeSource::Weapon {
                    item: w.choice.weapon,
                },
                TacticalAttackSource::Unarmed { .. } => TacticalMeleeSource::Unarmed,
                TacticalAttackSource::CreatureFeature {
                    feature_id, weapon, ..
                } => TacticalMeleeSource::CreatureFeature {
                    feature_id: feature_id.clone(),
                    weapon: *weapon,
                },
                TacticalAttackSource::Spell { .. } => {
                    return Err(invalid("a spell is not this melee opportunity source"));
                }
            };
            if !window.options.iter().any(|o| o.source == source) {
                return Err(invalid("reaction implement differs from retained crossing"));
            }
        }
        TacticalAttackAdmission::Spell { .. } => spell::validate_admission(state, attack)?,
    }
    Ok(())
}
