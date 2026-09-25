//! Closed physical attack work. All requests and costs are derived from source data;
//! this sibling composes with the existing turn pump, never a second command queue.
mod creature;
mod intrinsic;
mod opportunity;
mod planning;
mod spell;
mod validation;
use super::turns::*;
use super::*;
use crate::tactical_definitions::WeaponMastery;
use crate::tactical_weapons::*;
pub(super) use creature::begin_creature_attack;
pub(super) use opportunity::{begin_opportunity_attack, opportunity_options_for_crossing};
pub(super) use spell::begin_spell_attack;
pub(super) fn spell_occurrence(attack: &TacticalAttack) -> Option<(u16, SpellProgramOccurrence)> {
    match attack.source {
        TacticalAttackSource::Spell { cast, at, .. } => Some((cast, at)),
        _ => None,
    }
}
pub(super) fn validate(state: &CampaignState) -> Result<(), RulesError> {
    validation::validate(state)
}

fn current(state: &CampaignState) -> Result<&TacticalAttack, RulesError> {
    resolution(state)?
        .attack
        .as_ref()
        .ok_or_else(|| invalid("attack work absent"))
}
fn current_mut(state: &mut CampaignState) -> Result<&mut TacticalAttack, RulesError> {
    resolution_mut(state)?
        .attack
        .as_mut()
        .ok_or_else(|| invalid("attack work absent"))
}
fn weapon_error(error: WeaponError) -> RulesError {
    invalid(&error.to_string())
}

pub(super) fn begin(
    state: &mut CampaignState,
    meta: &CommandMeta,
    choice: &WeaponUseChoice,
    pack: &RulesPack,
) -> Result<(), RulesError> {
    if flow(state)?.phase != TacticalPhase::Active || flow(state)?.resolution.is_some() {
        return Err(RulesError::Pending);
    }
    let actor = active(state)?;
    authorize(state, meta, actor)?;
    if actor == choice.target {
        return Err(prerequisite(
            "self-directed attacks require their explicit adjudication path",
        ));
    }
    // A retained truth ID is not permission to query a hidden creature's range.
    // Check actor knowledge before any target geometry or source range planning.
    planning::require_located_target(state, actor, choice.target)?;
    let rules = state.rules.as_ref().ok_or(RulesError::Uninitialized)?;
    let timing = rules
        .timing
        .as_ref()
        .ok_or_else(|| invalid("attack timing absent"))?;
    let budget = &flow(state)?.budget;
    let (window, new_action) = match choice.purpose {
        WeaponAttackPurpose::Normal if budget.attacks_remaining > 0 => (
            budget
                .attack_window
                .ok_or_else(|| invalid("attack opportunity missing"))?,
            false,
        ),
        WeaponAttackPurpose::Normal if !timing.action_spent => (
            WeaponActionWindow {
                id: meta.id,
                kind: WeaponActionKind::AttackAction,
            },
            true,
        ),
        WeaponAttackPurpose::LightBonus { .. } => (
            WeaponActionWindow {
                id: meta.id,
                kind: WeaponActionKind::BonusAction,
            },
            false,
        ),
        WeaponAttackPurpose::Nick { .. } => (
            budget
                .attack_window
                .ok_or_else(|| prerequisite("Nick requires its original Attack action"))?,
            false,
        ),
        _ => return Err(prerequisite("no matching attack opportunity")),
    };
    let equipment = rules
        .tactical_inventory
        .as_ref()
        .and_then(|i| i.loadout(actor))
        .cloned()
        .ok_or_else(|| prerequisite("materialized current equipment is required"))?;
    let plan = planning::weapon_plan(state, meta, actor, choice, window, &equipment.hands, pack)?;
    // A bounded source slice must fail BEFORE any cost when it cannot finish a rider.
    if plan
        .mastery
        .is_some_and(|m| !matches!(m, WeaponMastery::Nick | WeaponMastery::Graze))
    {
        return Err(prerequisite(
            "this weapon mastery requires the pending typed modifier/movement continuation",
        ));
    }
    let (mode, armor_class, critical_on_hit) = planning::hit_facts(state, actor, choice, &plan)?;
    let ammunition = plan
        .ammunition
        .as_ref()
        .map(|spend| AttackAmmunitionReservation {
            stack: spend.stack,
            quantity_before: state.items[&spend.stack].quantity,
        });
    let attack = TacticalAttack {
        origin: meta.clone(),
        actor,
        target: choice.target,
        delivery: if choice.delivery == WeaponDelivery::Melee {
            TacticalAttackDelivery::Melee
        } else {
            TacticalAttackDelivery::Ranged
        },
        source: TacticalAttackSource::Weapon(Box::new(TacticalWeaponAttack {
            choice: choice.clone(),
            window,
            equipment_before: equipment,
            ammunition,
        })),
        admission: TacticalAttackAdmission::OwnTurn,
        attack_modifier: plan.attack_modifier,
        mode,
        armor_class,
        critical_on_hit,
        automatic_miss: plan.automatic_miss,
        damage: vec![AttackDamageComponent {
            damage_type: plan.damage.damage_type,
            dice: plan.damage.dice.clone(),
            modifier: plan.damage.modifier,
        }],
        stage: TacticalAttackStage::AttackRoll,
        attack_roll: None,
        damage_roll: None,
        outcome: None,
    };
    let mut budget = flow(state)?.budget.clone();
    budget.movement_progress = None;
    budget.movement_origin = None;
    let rules = state.rules.as_mut().ok_or(RulesError::Uninitialized)?;
    match choice.purpose {
        WeaponAttackPurpose::Normal => {
            if new_action {
                crate::tactical_budget::start_attack_action(rules, &mut budget, actor, 1)?;
                budget.attack_window = Some(window);
            }
            crate::tactical_budget::spend_attack(&mut budget)?;
        }
        WeaponAttackPurpose::LightBonus { .. } => crate::tactical_budget::spend_cost(
            rules,
            actor,
            crate::tactical_budget::TacticalCost::BonusAction,
        )?,
        WeaponAttackPurpose::Nick { .. } => (),
        WeaponAttackPurpose::Cleave { .. } => unreachable!("rejected above"),
    }
    budget.weapon_history.push(plan.receipt);
    // Healing may wake a knocked-out combatant without ending the mandatory
    // Short Rest. Attacking is strenuous activity; only accepting the action
    // interrupts that rest, not inspecting or rejecting an attack proposal.
    crate::kernel::interrupt_rest(rules, actor, state.clock.now);
    let loadout = rules
        .tactical_inventory
        .as_mut()
        .and_then(|i| i.loadouts.iter_mut().find(|l| l.actor == actor))
        .ok_or_else(|| invalid("equipment absent"))?;
    loadout.hands = plan.loadout_for_attack;
    loadout.command = meta.clone();
    if let Some(spend) = plan.ammunition {
        let stack = state
            .items
            .get_mut(&spend.stack)
            .ok_or_else(|| invalid("ammunition absent"))?;
        stack.quantity = stack
            .quantity
            .checked_sub(spend.quantity)
            .ok_or_else(|| invalid("ammunition underflow"))?;
        if stack.quantity == 0 {
            stack.state = ItemState::Spent;
        }
    }
    let timing = state
        .rules
        .as_ref()
        .and_then(|r| r.timing.as_ref())
        .ok_or_else(|| invalid("timing absent"))?;
    let number = timing.turn_number;
    flow_mut(state)?.budget = budget;
    flow_mut(state)?.resolution = Some(Box::new(TacticalResolution {
        origin: meta.clone(),
        turn_actor: actor,
        turn_number: number,
        boundary: TurnBoundary::Start,
        frames: vec![],
        pending: None,
        failed_save: None,
        legendary_window: None,
        attack: Some(attack),
        movement: None,
        casts: vec![],
        next_occurrence: 0,
    }));
    push_frame(state, vec![TacticalWorkKind::AttackRoll])?;
    pump(state, meta)
}

pub(super) fn key(
    state: &CampaignState,
    work: &TacticalWorkItem,
) -> Result<TacticalRollKey, RulesError> {
    let attack = current(state)?;
    let role = match work.kind {
        TacticalWorkKind::AttackRoll => TacticalRollRole::Attack,
        TacticalWorkKind::AttackDamage => TacticalRollRole::AttackDamage,
        _ => return Err(invalid("not attack dice work")),
    };
    Ok(TacticalRollKey {
        origin: match &attack.admission {
            TacticalAttackAdmission::Spell { casting_origin } => casting_origin.id,
            _ => attack.origin.id,
        },
        role,
        subject: attack.target,
        occurrence: work.occurrence,
    })
}
pub(super) fn request(
    state: &CampaignState,
    work: &TacticalWorkItem,
    key: TacticalRollKey,
) -> Result<Option<RollRequest>, RulesError> {
    let attack = current(state)?;
    let (dice, modifier, mode, reason) = match work.kind {
        TacticalWorkKind::AttackRoll => (
            vec![DieSpec {
                count: 1,
                sides: 20,
            }],
            attack.attack_modifier,
            attack.mode,
            "Attack",
        ),
        TacticalWorkKind::AttackDamage => {
            let critical = matches!(
                attack.outcome,
                Some(WeaponAttackOutcome::Hit { critical: true, .. })
            );
            let dice = attack
                .damage
                .iter()
                .flat_map(|c| c.dice.iter())
                .map(|d| {
                    Ok(DieSpec {
                        count: d
                            .count
                            .checked_mul(if critical { 2 } else { 1 })
                            .ok_or_else(|| invalid("critical dice overflow"))?,
                        sides: d.sides,
                    })
                })
                .collect::<Result<Vec<_>, RulesError>>()?;
            if dice.is_empty() {
                return Ok(None);
            }
            (dice, 0, RollMode::Normal, "Attack damage")
        }
        _ => return Err(invalid("not attack dice work")),
    };
    Ok(Some(RollRequest {
        id: key.request_id(),
        roller: Some(attack.actor),
        dice,
        modifier,
        mode,
        visibility: if controller(state, attack.actor).is_some() {
            RollVisibility::Public
        } else {
            RollVisibility::Secret
        },
        reason: reason.into(),
    }))
}

pub(super) fn start(
    state: &mut CampaignState,
    meta: &CommandMeta,
    work: &TacticalWorkItem,
) -> Result<bool, RulesError> {
    match work.kind {
        TacticalWorkKind::AttackRoll if current(state)?.automatic_miss => {
            let attack = current_mut(state)?;
            attack.outcome = Some(WeaponAttackOutcome::Miss);
            attack.stage = TacticalAttackStage::Finishing;
            push_frame(state, vec![TacticalWorkKind::FinishAttack])?;
            Ok(true)
        }
        TacticalWorkKind::AttackDamage
            if current(state)?.damage.iter().all(|c| c.dice.is_empty()) =>
        {
            apply_damage(state, meta, work.occurrence, None)?;
            Ok(true)
        }
        TacticalWorkKind::FinishAttack => {
            finish(state, meta)?;
            Ok(true)
        }
        _ => Ok(false),
    }
}
pub(super) fn resolved(
    state: &mut CampaignState,
    meta: &CommandMeta,
    pending: &TacticalPendingWork,
    result: &RollResult,
) -> Result<(), RulesError> {
    match pending.work.kind {
        TacticalWorkKind::AttackRoll => {
            let request = request(state, &pending.work, pending.key)?
                .ok_or_else(|| invalid("attack request absent"))?;
            let roll = request.resolve(result)?;
            let face = roll.kept_dice[0].value;
            let attack = current_mut(state)?;
            attack.attack_roll = Some(result.request_id);
            let hit = face != 1 && (face == 20 || roll.total >= attack.armor_class);
            attack.outcome = Some(if hit {
                WeaponAttackOutcome::Hit {
                    critical: face == 20 || attack.critical_on_hit,
                    damage_dealt: 0,
                }
            } else {
                WeaponAttackOutcome::Miss
            });
            attack.stage = if hit {
                TacticalAttackStage::DamageRoll
            } else {
                TacticalAttackStage::Finishing
            };
            push_frame(
                state,
                vec![if hit {
                    TacticalWorkKind::AttackDamage
                } else {
                    TacticalWorkKind::FinishAttack
                }],
            )?;
        }
        TacticalWorkKind::AttackDamage => {
            current_mut(state)?.damage_roll = Some(result.request_id);
            apply_damage(state, meta, pending.work.occurrence, None)?;
        }
        _ => return Err(invalid("not an attack continuation")),
    }
    Ok(())
}

fn packet(state: &CampaignState) -> Result<DamagePacket, RulesError> {
    let attack = current(state)?;
    let critical = matches!(
        attack.outcome,
        Some(WeaponAttackOutcome::Hit { critical: true, .. })
    );
    let rolls = state.rules.as_ref().ok_or(RulesError::Uninitialized)?;
    let raw = attack
        .damage_roll
        .map(|id| {
            rolls
                .rolls
                .iter()
                .find(|r| r.request.id == id)
                .ok_or_else(|| invalid("damage history absent"))
        })
        .transpose()?;
    let mut faces = raw.into_iter().flat_map(|r| r.resolved.kept_dice.iter());
    let mut components: Vec<DamageComponent> = Vec::new();
    for component in &attack.damage {
        let mut amount = i64::from(component.modifier);
        for die in &component.dice {
            for _ in 0..die.count * if critical { 2 } else { 1 } {
                let face = faces.next().ok_or_else(|| invalid("damage face missing"))?;
                if face.sides != die.sides {
                    return Err(invalid("damage die differs"));
                }
                amount += i64::from(face.value);
            }
        }
        let amount = u32::try_from(amount.max(0)).map_err(|_| invalid("damage amount overflow"))?;
        if let Some(existing) = components
            .iter_mut()
            .find(|c| c.damage_type == component.damage_type)
        {
            // One hit can add several source dice pools of the same type. Sum
            // them within that type before a single resistance/vulnerability step.
            existing.amounts.push(amount);
        } else {
            components.push(DamageComponent {
                damage_type: component.damage_type,
                amounts: vec![amount],
                adjustments: vec![],
            });
        }
    }
    if faces.next().is_some() {
        return Err(invalid("surplus damage faces"));
    }
    Ok(DamagePacket {
        cause: DamageCause::Attack {
            attacker: attack.actor,
            melee: attack.delivery == TacticalAttackDelivery::Melee,
            critical,
        },
        components,
    })
}
fn apply_damage(
    state: &mut CampaignState,
    meta: &CommandMeta,
    occurrence: u16,
    choice: Option<KnockoutChoice>,
) -> Result<(), RulesError> {
    let attack = current(state)?.clone();
    let packet = packet(state)?;
    let operation = VitalityOperation::Damage {
        packet,
        knockout: choice,
    };
    let context = crate::tactical_vitality_adapter::context(
        state,
        attack.target,
        VitalityOrigin {
            command: meta.clone(),
            occurrence,
        },
    )?;
    let rules = state.rules.as_ref().ok_or(RulesError::Uninitialized)?;
    let recovery = rules
        .tactical_recovery
        .as_ref()
        .and_then(|r| r.get(&attack.target))
        .cloned()
        .unwrap_or_default();
    let preview = crate::tactical_damage::reduce_vitality(
        &rules.entities[&attack.target],
        &recovery,
        &context,
        &operation,
    )
    .map_err(|e| invalid(&e.to_string()))?;
    if preview.awaiting_choice {
        current_mut(state)?.stage = TacticalAttackStage::KnockoutChoice;
        return Ok(());
    }
    let outcome = WeaponAttackOutcome::Hit {
        critical: matches!(
            attack.outcome,
            Some(WeaponAttackOutcome::Hit { critical: true, .. })
        ),
        damage_dealt: preview.outcome.damage_taken,
    };
    let plan = attack
        .weapon()
        .map(|_| planning::reconstruct(state, &attack))
        .transpose()?;
    // Finish this attack's physical equipment/history atomically with its damage,
    // before pumping resulting concentration/effect work. Those consequences may
    // incapacitate the attacker and drop equipment; never re-equip it afterward.
    complete(state, meta, &attack, plan.as_ref(), outcome)?;
    super::continuations::apply_vitality(
        state,
        meta,
        attack.target,
        occurrence,
        operation,
        Some(attack.actor),
    )
}
pub(super) fn choose_knockout(
    state: &mut CampaignState,
    meta: &CommandMeta,
    choice: KnockoutChoice,
) -> Result<(), RulesError> {
    let attack = current(state)?.clone();
    if attack.stage != TacticalAttackStage::KnockoutChoice {
        return Err(prerequisite("no knockout choice is due"));
    }
    authorize(state, meta, attack.actor)?;
    let occurrence = resolution(state)?.next_occurrence;
    if occurrence >= 32_768 {
        return Err(invalid("attack occurrence capacity"));
    }
    resolution_mut(state)?.next_occurrence += 1;
    apply_damage(state, meta, occurrence, Some(choice))?;
    pump(state, meta)
}
fn finish(state: &mut CampaignState, meta: &CommandMeta) -> Result<(), RulesError> {
    let attack = current(state)?.clone();
    let outcome = attack
        .outcome
        .ok_or_else(|| invalid("attack outcome absent"))?;
    let plan = attack
        .weapon()
        .map(|_| planning::reconstruct(state, &attack))
        .transpose()?;
    if plan
        .as_ref()
        .is_some_and(|p| p.mastery == Some(WeaponMastery::Graze))
        && outcome == WeaponAttackOutcome::Miss
    {
        current_mut(state)?.stage = TacticalAttackStage::MasteryChoice;
        return Ok(());
    }
    complete(state, meta, &attack, plan.as_ref(), outcome)
}
fn complete(
    state: &mut CampaignState,
    meta: &CommandMeta,
    attack: &TacticalAttack,
    plan: Option<&WeaponAttackPlan>,
    outcome: WeaponAttackOutcome,
) -> Result<(), RulesError> {
    if let Some(plan) = plan {
        let rules = state.rules.as_mut().ok_or(RulesError::Uninitialized)?;
        let loadout = rules
            .tactical_inventory
            .as_mut()
            .and_then(|i| i.loadouts.iter_mut().find(|l| l.actor == attack.actor))
            .ok_or_else(|| invalid("equipment absent"))?;
        loadout.hands = plan.loadout_after_attack.clone();
        loadout.command = meta.clone();
        if let Some(item) = plan.thrown_weapon {
            let encounter = encounter(state)?;
            let position = encounter
                .participant(attack.target)
                .ok_or_else(|| invalid("target absent"))?
                .position;
            let location = state
                .scenes
                .get(&encounter.scene_id)
                .ok_or_else(|| invalid("scene absent"))?
                .location_id;
            state
                .items
                .get_mut(&item)
                .ok_or_else(|| invalid("thrown item absent"))?
                .custody = Custody::Location(location);
            flow_mut(state)?
                .ground_items
                .retain(|entry| entry.item != item);
            flow_mut(state)?.ground_items.push(TacticalGroundItem {
                item,
                position,
                origin: meta.clone(),
            });
        }
        let receipt = flow_mut(state)?
            .budget
            .weapon_history
            .iter_mut()
            .find(|r| r.origin.id == attack.origin.id)
            .ok_or_else(|| invalid("attack receipt absent"))?;
        receipt.outcome = outcome;
    } else if matches!(attack.source, TacticalAttackSource::Spell { .. }) {
        spell::complete(state, attack)?;
    } else {
        intrinsic::complete(state, attack, outcome)?;
    }
    resolution_mut(state)?.attack = None;
    Ok(())
}
pub(super) fn choose_mastery(
    state: &mut CampaignState,
    meta: &CommandMeta,
    choice: &WeaponMasteryChoice,
) -> Result<(), RulesError> {
    let attack = current(state)?.clone();
    if attack.stage != TacticalAttackStage::MasteryChoice {
        return Err(prerequisite("no mastery choice is due"));
    }
    authorize(state, meta, attack.actor)?;
    let plan = planning::reconstruct(state, &attack)?;
    let target = encounter(state)?
        .participant(attack.target)
        .ok_or_else(|| invalid("target absent"))?;
    let mastery = weapon_mastery_resolution(
        &plan,
        WeaponAttackOutcome::Miss,
        &WeaponMasteryContext {
            target_is_creature: true,
            target_size: target.size,
            history: &flow(state)?.budget.weapon_history,
        },
    )
    .map_err(weapon_error)?;
    let consequence = choose_weapon_mastery(
        mastery
            .offer
            .as_ref()
            .ok_or_else(|| invalid("mastery offer absent"))?,
        choice,
    )
    .map_err(weapon_error)?;
    complete(state, meta, &attack, Some(&plan), WeaponAttackOutcome::Miss)?;
    if let Some(WeaponMasteryConsequence::GrazeDamage {
        source,
        target,
        amount,
        damage_type,
    }) = consequence
    {
        let occurrence = resolution(state)?.next_occurrence;
        if occurrence >= 32_768 {
            return Err(invalid("attack occurrence capacity"));
        }
        resolution_mut(state)?.next_occurrence += 1;
        super::continuations::apply_vitality(
            state,
            meta,
            target,
            occurrence,
            VitalityOperation::Damage {
                packet: DamagePacket {
                    cause: DamageCause::Graze {
                        attacker: source,
                        ability_modifier: plan.ability_modifier,
                    },
                    components: vec![DamageComponent {
                        damage_type,
                        amounts: vec![amount],
                        adjustments: vec![],
                    }],
                },
                knockout: None,
            },
            Some(source),
        )?;
    }
    pump(state, meta)
}
