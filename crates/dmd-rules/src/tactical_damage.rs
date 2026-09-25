//! Source-derived vitality planner/reducer. This module never authorizes a command,
//! spends an action, mutates an effect group, generates dice or writes a journal.
//! The caller must atomically apply the returned entity/recovery and all follow-ups.
use crate::{ResolveRoll, test_modifier};
use dmd_domain::*;
use std::collections::{BTreeSet, HashSet};
use thiserror::Error;

pub const MAX_VITALITY_AMOUNT: u32 = 1_000_000;
const MAX_COMPONENT_AMOUNTS: usize = 64;
const MAX_ADJUSTMENTS: usize = 32;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum VitalityError {
    #[error("invalid tactical vitality input: {0}")]
    Invalid(&'static str),
    #[error("tactical vitality prerequisite failed: {0}")]
    Prerequisite(&'static str),
}

/// Facts derived by the enclosing source/condition resolver, never client assertions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VitalityContext {
    pub origin: VitalityOrigin,
    pub now: WorldInstant,
    /// Unified current conditions, including external spell/monster effects.
    pub conditions: HashSet<Condition>,
    pub underwater: bool,
    /// Conditional source effects add target defenses without mutating intrinsic traits.
    pub defenses: VitalityDefenses,
    /// Source-derived circumstances/bonuses, in addition to the entity's exhaustion.
    pub death_save_mode: RollMode,
    pub death_save_bonus: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct VitalityDefenses {
    pub immunities: BTreeSet<DamageType>,
    pub resistances: BTreeSet<DamageType>,
    pub vulnerabilities: BTreeSet<DamageType>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VitalityFollowup {
    ChooseKnockout {
        attacker: EntityId,
    },
    ChooseTemporaryHitPoints {
        existing: u32,
        offered: u32,
    },
    /// The occurrence is the transition's origin, not a deduplicated damage total.
    ConcentrationSave {
        group: EffectId,
        damage_taken: u32,
        dc: u8,
    },
    EndConcentration {
        group: EffectId,
    },
    /// Source Unconscious consequences; caller drops actual held ItemIds atomically.
    DropHeldItems,
    InterruptRest,
    StartKnockoutShortRest {
        started_at: WorldInstant,
    },
    /// Caller creates/persists the request and collects a raw d4 without choosing a face.
    StableRecoveryRoll {
        origin: VitalityOrigin,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct VitalityOutcome {
    pub damage_taken: u32,
    pub temporary_hp_lost: u32,
    pub hp_lost: u32,
    pub hp_regained: u32,
    pub stabilized: bool,
    pub knocked_out: bool,
    pub died: bool,
    pub death_save_succeeded: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VitalityTransition {
    pub origin: VitalityOrigin,
    pub entity: MechanicalEntity,
    pub recovery: TacticalRecovery,
    pub outcome: VitalityOutcome,
    pub followups: Vec<VitalityFollowup>,
    /// True means no mutation was made; resume the original operation with its choice.
    pub awaiting_choice: bool,
}

fn invalid(message: &'static str) -> VitalityError {
    VitalityError::Invalid(message)
}
fn prerequisite(message: &'static str) -> VitalityError {
    VitalityError::Prerequisite(message)
}

fn validate_origin(origin: &VitalityOrigin, current: &VitalityOrigin) -> Result<(), VitalityError> {
    let source = &origin.command;
    let head = &current.command;
    if source.id.0.is_nil()
        || source.campaign_id.0.is_nil()
        || source.session_id.is_some_and(|id| id.0.is_nil())
        || source.campaign_id != head.campaign_id
        || source.expected_event_sequence > head.expected_event_sequence
        || (source.expected_event_sequence == head.expected_event_sequence && source != head)
        || (source.id == head.id && (source != head || origin.occurrence > current.occurrence))
    {
        return Err(invalid("recovery origin is not a valid accepted command"));
    }
    Ok(())
}

/// Structural/source checks for restored records; does not alter old kernel validation.
pub fn validate_recovery(
    entity: &MechanicalEntity,
    recovery: &TacticalRecovery,
    context: &VitalityContext,
) -> Result<(), VitalityError> {
    validate_origin(&context.origin, &context.origin)?;
    if entity.entity_id.0.is_nil()
        || entity.max_hp > MAX_VITALITY_AMOUNT
        || entity.hp > entity.max_hp
        || entity.temporary_hp > MAX_VITALITY_AMOUNT
        || entity.exhaustion > 6
        || (entity.exhaustion == 6 && !entity.death.dead)
        || context.death_save_bonus.unsigned_abs() > 1000
        || (entity.max_hp == 0 && !entity.death.dead)
        || entity.death.successes > 2
        || entity.death.failures > 2
        || (entity.death.dead
            && (entity.hp != 0
                || entity.death.stable
                || entity.death.successes != 0
                || entity.death.failures != 0))
        || (entity.hp > 0 && entity.death != DeathState::default())
        || (entity.death.stable && (entity.death.successes != 0 || entity.death.failures != 0))
        || (entity.hp == 0
            && !entity.death.dead
            && (!entity.uses_death_saves
                || (!entity.prone
                    && !entity
                        .condition_immunities
                        .contains(&Condition::Unconscious)
                    && !entity.condition_immunities.contains(&Condition::Prone))))
    {
        return Err(invalid("inconsistent HP, death, or source modifier bounds"));
    }
    if let Some(knockout) = &recovery.knockout {
        validate_origin(&knockout.origin, &context.origin)?;
        if entity.death.dead
            || (!entity.prone && !entity.condition_immunities.contains(&Condition::Prone))
            || knockout.inflicted_at > context.now
            || knockout.short_rest_started_at.is_some_and(|start| {
                start < knockout.inflicted_at || start > context.now || entity.hp == 0
            })
        {
            return Err(invalid("invalid knockout recovery"));
        }
    }
    if let Some(rest) = &recovery.knockout_rest {
        validate_origin(&rest.knockout_origin, &rest.started_by)?;
        validate_origin(&rest.started_by, &context.origin)?;
        if entity.death.dead
            || entity.hp == 0
            || rest.started_at > context.now
            || recovery.knockout.as_ref().is_some_and(|knockout| {
                knockout.origin != rest.knockout_origin
                    || knockout.short_rest_started_at != Some(rest.started_at)
                    || rest.started_at < knockout.inflicted_at
            })
        {
            return Err(invalid("invalid knockout rest authorization"));
        }
    }
    if recovery
        .knockout
        .as_ref()
        .is_some_and(|knockout| knockout.short_rest_started_at.is_some())
        && recovery.knockout_rest.is_none()
    {
        return Err(invalid("knockout rest lacks its source authorization"));
    }
    if let Some(stable) = &recovery.stable {
        validate_origin(&stable.origin, &context.origin)?;
        if !entity.death.stable
            || entity.hp != 0
            || entity.death.dead
            || stable.stabilized_at > context.now
        {
            return Err(invalid("invalid stable recovery"));
        }
        if let Some(result) = &stable.delay_roll {
            let delay = stable_recovery_request(entity.entity_id, result.request_id)?;
            delay
                .resolve(result)
                .map_err(|_| invalid("invalid stable recovery die"))?;
            stable_wake_at(stable)?;
        }
    }
    Ok(())
}

/// Only this module's knockout cause; callers union it with all other condition sources.
pub fn recovery_conditions(
    entity: &MechanicalEntity,
    recovery: &TacticalRecovery,
) -> HashSet<Condition> {
    if recovery.knockout.is_some()
        && !entity
            .condition_immunities
            .contains(&Condition::Unconscious)
    {
        [
            Condition::Unconscious,
            Condition::Incapacitated,
            Condition::Prone,
        ]
        .into_iter()
        .filter(|condition| !entity.condition_immunities.contains(condition))
        .collect()
    } else {
        HashSet::new()
    }
}

fn incapacitated(
    entity: &MechanicalEntity,
    recovery: &TacticalRecovery,
    context: &VitalityContext,
) -> bool {
    if entity.death.dead {
        return true;
    }
    if entity
        .condition_immunities
        .contains(&Condition::Incapacitated)
    {
        return false;
    }
    entity.death.dead
        || (entity.hp == 0
            && !entity
                .condition_immunities
                .contains(&Condition::Unconscious))
        || (recovery.knockout.is_some()
            && !entity
                .condition_immunities
                .contains(&Condition::Unconscious))
        || [
            Condition::Incapacitated,
            Condition::Unconscious,
            Condition::Paralyzed,
            Condition::Petrified,
            Condition::Stunned,
        ]
        .iter()
        .any(|c| context.conditions.contains(c))
}

/// Adjustments, then immunity/resistance/vulnerability (SRD17). Dice have already been
/// resolved by the attack/spell planner: this must NOT double critical damage again.
pub fn damage_taken(
    entity: &MechanicalEntity,
    context: &VitalityContext,
    packet: &DamagePacket,
) -> Result<u32, VitalityError> {
    if packet.components.is_empty() || packet.components.len() > 13 {
        return Err(invalid("damage requires one to thirteen typed components"));
    }
    if matches!(packet.cause, DamageCause::Attack { attacker, .. } | DamageCause::Graze { attacker, .. } if attacker.0.is_nil())
    {
        return Err(invalid("attack damage requires its attacker"));
    }
    let graze = if let DamageCause::Graze {
        ability_modifier, ..
    } = packet.cause
    {
        if !(-5..=10).contains(&ability_modifier)
            || packet.components.len() != 1
            || packet.components[0].amounts != [ability_modifier.max(0) as u32]
        {
            return Err(invalid(
                "Graze damage must equal only its source ability modifier",
            ));
        }
        true
    } else {
        false
    };
    let mut types = BTreeSet::new();
    let mut total = 0u32;
    for component in &packet.components {
        if !types.insert(component.damage_type)
            || component.amounts.is_empty()
            || component.amounts.len() > MAX_COMPONENT_AMOUNTS
            || component.adjustments.len() > MAX_ADJUSTMENTS
        {
            return Err(invalid(
                "duplicate damage type or unbounded damage component",
            ));
        }
        let mut amount = 0i64;
        for value in &component.amounts {
            if *value > MAX_VITALITY_AMOUNT {
                return Err(invalid("damage amount outside bounds"));
            }
            amount += i64::from(*value);
        }
        if amount > i64::from(MAX_VITALITY_AMOUNT) {
            return Err(invalid("summed damage outside bounds"));
        }
        for adjustment in &component.adjustments {
            match *adjustment {
                DamageAdjustment::Add(value) => {
                    if graze && value > 0 {
                        return Err(invalid("Graze cannot add damage bonuses"));
                    }
                    if i64::from(value).abs() > i64::from(MAX_VITALITY_AMOUNT) {
                        return Err(invalid("damage adjustment outside bounds"));
                    }
                    amount += i64::from(value);
                }
                DamageAdjustment::Multiply {
                    numerator,
                    denominator,
                } => {
                    if denominator == 0 || numerator > 1000 || denominator > 1000 {
                        return Err(invalid("damage multiplier outside bounds"));
                    }
                    if graze && numerator > denominator {
                        return Err(invalid("Graze cannot increase through damage multipliers"));
                    }
                    amount = (amount * i64::from(numerator)).div_euclid(i64::from(denominator));
                }
            }
            if amount.abs() > i64::from(MAX_VITALITY_AMOUNT) {
                return Err(invalid("adjusted damage outside bounds"));
            }
        }
        let mut amount = amount.max(0) as u32;
        if entity.damage_immunities.contains(&component.damage_type)
            || context.defenses.immunities.contains(&component.damage_type)
        {
            amount = 0;
        }
        if entity.resistances.contains(&component.damage_type)
            || context
                .defenses
                .resistances
                .contains(&component.damage_type)
            || context.conditions.contains(&Condition::Petrified)
            || (context.underwater && component.damage_type == DamageType::Fire)
        {
            amount /= 2;
        }
        // Graze's specific restriction overrides increases from generic vulnerability.
        if !graze
            && (entity.vulnerabilities.contains(&component.damage_type)
                || context
                    .defenses
                    .vulnerabilities
                    .contains(&component.damage_type))
        {
            amount *= 2;
        }
        total = total
            .checked_add(amount)
            .filter(|v| *v <= MAX_VITALITY_AMOUNT)
            .ok_or_else(|| invalid("final damage outside bounds"))?;
    }
    Ok(total)
}

fn request(
    actor: EntityId,
    id: RollRequestId,
    sides: u16,
    modifier: i32,
    mode: RollMode,
    reason: &str,
) -> Result<RollRequest, VitalityError> {
    if id.0.is_nil() || actor.0.is_nil() {
        return Err(invalid("roll requires non-nil identifiers"));
    }
    Ok(RollRequest {
        id,
        roller: Some(actor),
        dice: vec![DieSpec { count: 1, sides }],
        modifier,
        mode,
        visibility: RollVisibility::Public,
        reason: reason.into(),
    })
}

pub fn death_save_request(
    entity: &MechanicalEntity,
    context: &VitalityContext,
    id: RollRequestId,
) -> Result<RollRequest, VitalityError> {
    death_save_eligible(entity)?;
    if context.death_save_bonus.unsigned_abs() > 1000 {
        return Err(invalid("death save bonus outside bounds"));
    }
    request(
        entity.entity_id,
        id,
        20,
        test_modifier(entity, &TestKind::DeathSave) + context.death_save_bonus,
        context.death_save_mode,
        "death-save",
    )
}

fn death_save_eligible(entity: &MechanicalEntity) -> Result<(), VitalityError> {
    if entity.hp != 0 || entity.death.dead || entity.death.stable || !entity.uses_death_saves {
        return Err(prerequisite(
            "death save requires an unstable living creature at zero HP",
        ));
    }
    Ok(())
}

pub fn stable_recovery_request(
    actor: EntityId,
    id: RollRequestId,
) -> Result<RollRequest, VitalityError> {
    request(actor, id, 4, 0, RollMode::Normal, "stable-recovery-hours")
}

pub fn stable_wake_at(stable: &StableRecovery) -> Result<Option<WorldInstant>, VitalityError> {
    let Some(roll) = &stable.delay_roll else {
        return Ok(None);
    };
    if roll.dice.len() != 1 || roll.dice[0].sides != 4 || !(1..=4).contains(&roll.dice[0].value) {
        return Err(invalid("stable recovery requires raw 1d4"));
    }
    stable
        .stabilized_at
        .0
        .checked_add(i64::from(roll.dice[0].value) * 3600)
        .map(|time| Some(WorldInstant(time)))
        .ok_or_else(|| invalid("stable recovery time overflow"))
}

fn stabilize(next: &mut VitalityTransition, context: &VitalityContext) {
    next.entity.death = DeathState {
        stable: true,
        ..DeathState::default()
    };
    next.recovery.stable = Some(StableRecovery {
        origin: context.origin.clone(),
        stabilized_at: context.now,
        delay_roll: None,
    });
    next.outcome.stabilized = true;
    next.followups.push(VitalityFollowup::StableRecoveryRoll {
        origin: context.origin.clone(),
    });
}
fn kill(next: &mut VitalityTransition) {
    next.entity.hp = 0;
    next.entity.death = DeathState {
        dead: true,
        ..DeathState::default()
    };
    next.recovery = TacticalRecovery::default();
    if !next.followups.contains(&VitalityFollowup::InterruptRest) {
        next.followups.push(VitalityFollowup::InterruptRest);
    }
    next.outcome.died = true;
}
fn heal(next: &mut VitalityTransition, amount: u32) {
    let regained = amount.min(next.entity.max_hp - next.entity.hp);
    next.entity.hp += regained;
    next.outcome.hp_regained = regained;
    if regained > 0 {
        next.entity.death = DeathState::default();
        next.recovery.knockout = None;
        next.recovery.stable = None;
    }
}

/// Internal pure transition: inputs remain unchanged on both rejection and success.
pub fn reduce_vitality(
    entity: &MechanicalEntity,
    recovery: &TacticalRecovery,
    context: &VitalityContext,
    operation: &VitalityOperation,
) -> Result<VitalityTransition, VitalityError> {
    validate_recovery(entity, recovery, context)?;
    if entity.death.dead && !matches!(operation, VitalityOperation::ReconcileConditions) {
        return Err(prerequisite(
            "ordinary vitality operations cannot affect or revive the dead",
        ));
    }
    let mut next = VitalityTransition {
        origin: context.origin.clone(),
        entity: entity.clone(),
        recovery: recovery.clone(),
        outcome: VitalityOutcome::default(),
        followups: Vec::new(),
        awaiting_choice: false,
    };
    match operation {
        VitalityOperation::Damage { packet, knockout } => {
            let amount = damage_taken(entity, context, packet)?;
            let hp_damage = amount.saturating_sub(entity.temporary_hp);
            let attacker = match packet.cause {
                DamageCause::Attack {
                    attacker,
                    melee: true,
                    ..
                } if entity.hp > 0 && hp_damage >= entity.hp => Some(attacker),
                _ => None,
            };
            if *knockout == Some(KnockoutChoice::KnockOut) && attacker.is_none() {
                return Err(prerequisite(
                    "knockout requires a melee hit reducing positive HP to zero",
                ));
            }
            if let (Some(attacker), None) = (attacker, knockout) {
                next.followups
                    .push(VitalityFollowup::ChooseKnockout { attacker });
                next.awaiting_choice = true;
                return Ok(next);
            }
            next.outcome.damage_taken = amount;
            if amount > 0 {
                next.outcome.temporary_hp_lost = amount.min(entity.temporary_hp);
                next.entity.temporary_hp -= next.outcome.temporary_hp_lost;
                next.followups.push(VitalityFollowup::InterruptRest);
                next.recovery.stable = None;
                next.recovery.knockout_rest = None;
                if let Some(knockout) = &mut next.recovery.knockout {
                    knockout.short_rest_started_at = None;
                }
                if *knockout == Some(KnockoutChoice::KnockOut) {
                    next.entity.hp = 1;
                    next.entity.death = DeathState::default();
                    next.outcome.knocked_out = true;
                    if !entity
                        .condition_immunities
                        .contains(&Condition::Unconscious)
                    {
                        if !entity.condition_immunities.contains(&Condition::Prone) {
                            next.entity.prone = true;
                        }
                        next.recovery.knockout = Some(KnockoutRecovery {
                            origin: context.origin.clone(),
                            inflicted_at: context.now,
                            short_rest_started_at: Some(context.now),
                        });
                        next.recovery.knockout_rest = Some(KnockoutRestAuthorization {
                            knockout_origin: context.origin.clone(),
                            started_by: context.origin.clone(),
                            started_at: context.now,
                        });
                        next.followups.push(VitalityFollowup::DropHeldItems);
                        next.followups
                            .push(VitalityFollowup::StartKnockoutShortRest {
                                started_at: context.now,
                            });
                    }
                } else if entity.hp == 0 {
                    next.entity.death.stable = false;
                    let critical =
                        matches!(packet.cause, DamageCause::Attack { critical: true, .. });
                    let failures = entity.death.failures + if critical { 2 } else { 1 };
                    if amount >= entity.max_hp || failures >= 3 {
                        kill(&mut next);
                    } else {
                        next.entity.death.failures = failures;
                    }
                } else if hp_damage >= entity.hp {
                    next.entity.hp = 0;
                    if !entity
                        .condition_immunities
                        .contains(&Condition::Unconscious)
                    {
                        if !entity.condition_immunities.contains(&Condition::Prone) {
                            next.entity.prone = true;
                        }
                        next.followups.push(VitalityFollowup::DropHeldItems);
                    }
                    if !entity.uses_death_saves || hp_damage - entity.hp >= entity.max_hp {
                        kill(&mut next);
                    }
                } else {
                    next.entity.hp -= hp_damage;
                }
                next.outcome.hp_lost = entity.hp - next.entity.hp;
                if let Some(group) = entity.concentration
                    && !incapacitated(&next.entity, &next.recovery, context)
                {
                    next.followups.push(VitalityFollowup::ConcentrationSave {
                        group,
                        damage_taken: amount,
                        dc: (amount / 2).clamp(10, 30) as u8,
                    });
                }
            }
        }
        VitalityOperation::Heal { amount } => {
            if *amount > MAX_VITALITY_AMOUNT {
                return Err(invalid("healing outside bounds"));
            }
            heal(&mut next, *amount);
        }
        VitalityOperation::TemporaryHitPoints { amount, choice } => {
            if *amount > MAX_VITALITY_AMOUNT {
                return Err(invalid("temporary HP outside bounds"));
            }
            if entity.temporary_hp > 0 && choice.is_none() {
                next.followups
                    .push(VitalityFollowup::ChooseTemporaryHitPoints {
                        existing: entity.temporary_hp,
                        offered: *amount,
                    });
                next.awaiting_choice = true;
                return Ok(next);
            }
            if *choice != Some(TemporaryHpChoice::KeepExisting) {
                next.entity.temporary_hp = *amount;
            }
        }
        VitalityOperation::Medicine { purpose, .. }
        | VitalityOperation::MedicineOutcome { purpose, .. } => {
            let succeeded = match operation {
                VitalityOperation::Medicine { total, .. } => {
                    if !(-10_000..=10_000).contains(total) {
                        return Err(invalid("Medicine total outside bounds"));
                    }
                    *total >= 10
                }
                VitalityOperation::MedicineOutcome { succeeded, .. } => *succeeded,
                _ => unreachable!(),
            };
            match purpose {
                MedicinePurpose::EndKnockout => {
                    if recovery.knockout.is_none() {
                        return Err(prerequisite("no knockout condition to end"));
                    }
                    if succeeded {
                        next.recovery.knockout = None;
                    }
                }
                MedicinePurpose::Stabilize => {
                    if entity.hp != 0 || entity.death.stable {
                        return Err(prerequisite("no unstable zero-HP creature to stabilize"));
                    }
                    if succeeded {
                        stabilize(&mut next, context);
                    }
                }
            }
        }
        VitalityOperation::DeathSave { request_id, result } => {
            let roll = death_save_request(entity, context, *request_id)?
                .resolve(result)
                .map_err(|_| invalid("invalid raw death save dice"))?;
            let face = roll.kept_dice[0].value;
            if face == 20 {
                heal(&mut next, 1);
                next.outcome.death_save_succeeded = Some(true);
            } else {
                let passed = face != 1 && roll.total >= 10;
                next.outcome.death_save_succeeded = Some(passed);
                if face == 1 {
                    next.entity.death.failures += 2;
                } else if passed {
                    next.entity.death.successes += 1;
                } else {
                    next.entity.death.failures += 1;
                }
                if next.entity.death.failures >= 3 {
                    kill(&mut next);
                } else if next.entity.death.successes >= 3 {
                    stabilize(&mut next, context);
                }
            }
        }
        VitalityOperation::FailDeathSave => {
            death_save_eligible(entity)?;
            next.outcome.death_save_succeeded = Some(false);
            next.entity.death.failures += 1;
            if next.entity.death.failures >= 3 {
                kill(&mut next);
            }
        }
        VitalityOperation::SucceedDeathSave => {
            death_save_eligible(entity)?;
            next.outcome.death_save_succeeded = Some(true);
            next.entity.death.successes += 1;
            if next.entity.death.successes >= 3 {
                stabilize(&mut next, context);
            }
        }
        VitalityOperation::Stabilize => {
            if entity.hp != 0 || entity.death.stable {
                return Err(prerequisite(
                    "stabilization requires an unstable creature at zero HP",
                ));
            }
            stabilize(&mut next, context);
        }
        VitalityOperation::StableRecoveryRoll {
            origin,
            request_id,
            result,
        } => {
            let stable = next
                .recovery
                .stable
                .as_mut()
                .ok_or_else(|| prerequisite("no stable recovery awaiting a die"))?;
            if stable.delay_roll.is_some() || stable.origin != *origin {
                return Err(prerequisite(
                    "stable recovery origin changed or die already accepted",
                ));
            }
            stable_recovery_request(entity.entity_id, *request_id)?
                .resolve(result)
                .map_err(|_| invalid("invalid raw stable recovery die"))?;
            stable.delay_roll = Some(result.clone());
            stable_wake_at(stable)?;
        }
        VitalityOperation::RecoverStable => {
            let stable = recovery
                .stable
                .as_ref()
                .ok_or_else(|| prerequisite("no remaining stable recovery"))?;
            let wake = stable_wake_at(stable)?
                .ok_or_else(|| prerequisite("stable recovery still requires its raw d4"))?;
            if context.now < wake {
                return Err(prerequisite("stable recovery time has not elapsed"));
            }
            heal(&mut next, 1);
        }
        VitalityOperation::InterruptKnockoutRest => {
            next.recovery.knockout_rest = None;
            if let Some(knockout) = &mut next.recovery.knockout {
                knockout.short_rest_started_at = None;
            }
        }
        VitalityOperation::StartKnockoutRest => {
            if entity.hp == 0 {
                return Err(prerequisite("short rest requires at least one HP"));
            }
            let knockout = next
                .recovery
                .knockout
                .as_mut()
                .ok_or_else(|| prerequisite("no knockout rest to start"))?;
            if knockout.short_rest_started_at.is_some() {
                return Err(prerequisite("knockout short rest already started"));
            }
            knockout.short_rest_started_at = Some(context.now);
            next.recovery.knockout_rest = Some(KnockoutRestAuthorization {
                knockout_origin: knockout.origin.clone(),
                started_by: context.origin.clone(),
                started_at: context.now,
            });
            next.followups
                .push(VitalityFollowup::StartKnockoutShortRest {
                    started_at: context.now,
                });
        }
        VitalityOperation::CompleteShortRest => {
            if recovery.knockout.is_some() && recovery.knockout_rest.is_none() {
                return Err(prerequisite("knockout rest was interrupted"));
            }
            if let Some(rest) = &recovery.knockout_rest
                && context
                    .now
                    .0
                    .checked_sub(rest.started_at.0)
                    .is_none_or(|elapsed| elapsed < 3600)
            {
                return Err(prerequisite(
                    "knockout rest requires one uninterrupted hour",
                ));
            }
            next.recovery.knockout = None;
            next.recovery.knockout_rest = None;
        }
        VitalityOperation::CompleteLongRest => {
            if recovery.knockout_rest.is_some() {
                return Err(prerequisite(
                    "the source-authorized rest is Short, not Long",
                ));
            }
            next.entity.temporary_hp = 0;
        }
        VitalityOperation::SetMaximumHitPoints { maximum } => {
            if *maximum > MAX_VITALITY_AMOUNT {
                return Err(invalid("maximum HP outside bounds"));
            }
            next.entity.max_hp = *maximum;
            next.entity.hp = next.entity.hp.min(*maximum);
            next.outcome.hp_lost = entity.hp - next.entity.hp;
            if *maximum == 0 {
                kill(&mut next);
            }
        }
        VitalityOperation::ReconcileConditions => {}
    }
    if incapacitated(&next.entity, &next.recovery, context)
        && let Some(group) = entity.concentration
    {
        next.followups
            .push(VitalityFollowup::EndConcentration { group });
    }
    validate_recovery(&next.entity, &next.recovery, context)?;
    Ok(next)
}

#[cfg(test)]
mod tests;
