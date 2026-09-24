//! Pairwise tactical consequences of SRD 5.2.1 conditions (pp. 176–191).
//! The encounter resolver supplies sensory facts derived from authoritative geometry.
use crate::{RulesError, active_conditions};
use dmd_domain::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AttackPerception {
    pub attacker_sees_target: bool,
    pub target_sees_attacker: bool,
    pub fear_source_in_sight: bool,
    pub within_five_feet: bool,
    pub hostile_ranged_threat: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AttackConditions {
    pub mode: RollMode,
    /// A hit, not merely an attack declaration, becomes critical in these circumstances.
    pub critical_on_hit: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DodgeContext {
    pub declared: bool,
    /// Derived current Speed, including Exhaustion and non-condition effects.
    pub effective_speed_units: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestDisposition {
    Roll(RollMode),
    AutomaticFailure,
}

fn entity(rules: &RulesState, actor: EntityId) -> Result<&MechanicalEntity, RulesError> {
    rules.entities.get(&actor).ok_or_else(|| {
        RulesError::Invalid("tactical condition query references missing mechanics".into())
    })
}

fn mode(advantage: bool, disadvantage: bool) -> RollMode {
    match (advantage, disadvantage) {
        (true, false) => RollMode::Advantage,
        (false, true) => RollMode::Disadvantage,
        _ => RollMode::Normal,
    }
}

/// Charmed forbids attacks and other harmful abilities/effects against the charmer.
pub fn may_harm(rules: &RulesState, actor: EntityId, target: EntityId) -> bool {
    !rules.effects.iter().any(|effect| {
        effect.target == actor
            && effect.source == target
            && effect.condition == Some(Condition::Charmed)
    })
}

pub fn can_act(rules: &RulesState, actor: EntityId) -> Result<bool, RulesError> {
    let entity = entity(rules, actor)?;
    Ok(!entity.death.dead && !active_conditions(rules, actor).contains(&Condition::Incapacitated))
}

/// Darkness, blindsight and invisibility are pairwise; a global Invisible/Blinded flag
/// must not override the already-derived fact that a particular opponent can see you.
pub fn attack_conditions(
    rules: &RulesState,
    actor: EntityId,
    target: EntityId,
    ranged: bool,
    perception: AttackPerception,
    other: Circumstances,
    dodge: DodgeContext,
) -> Result<AttackConditions, RulesError> {
    entity(rules, target)?;
    if !can_act(rules, actor)? || !may_harm(rules, actor, target) {
        return Err(RulesError::Prerequisite(
            "the actor cannot make this attack".into(),
        ));
    }
    let own = active_conditions(rules, actor);
    let theirs = active_conditions(rules, target);
    let mut advantage = other.advantage || !perception.target_sees_attacker;
    let mut disadvantage = other.disadvantage || !perception.attacker_sees_target;
    disadvantage |= own.contains(&Condition::Poisoned)
        || own.contains(&Condition::Prone)
        || own.contains(&Condition::Restrained)
        || (own.contains(&Condition::Frightened) && perception.fear_source_in_sight)
        || (ranged && perception.hostile_ranged_threat);
    disadvantage |= rules.effects.iter().any(|effect| {
        effect.target == actor
            && effect.condition == Some(Condition::Grappled)
            && effect.source != target
    });
    advantage |= [
        Condition::Paralyzed,
        Condition::Petrified,
        Condition::Restrained,
        Condition::Stunned,
        Condition::Unconscious,
    ]
    .iter()
    .any(|condition| theirs.contains(condition));
    if theirs.contains(&Condition::Prone) {
        advantage |= perception.within_five_feet;
        disadvantage |= !perception.within_five_feet;
    }
    // A Dodge benefit requires sight of the attacker and ends if incapacitated or Speed 0.
    disadvantage |= dodge.declared
        && dodge.effective_speed_units > 0
        && perception.target_sees_attacker
        && !theirs.contains(&Condition::Incapacitated)
        && !theirs.contains(&Condition::Grappled)
        && !theirs.contains(&Condition::Restrained);
    Ok(AttackConditions {
        mode: mode(advantage, disadvantage),
        critical_on_hit: perception.within_five_feet
            && (theirs.contains(&Condition::Paralyzed) || theirs.contains(&Condition::Unconscious)),
    })
}

/// The caller separately establishes whether a task is possible from available evidence.
/// Being unable to perform the required sensory task cannot be cured by a high die face.
pub fn check_conditions(
    rules: &RulesState,
    actor: EntityId,
    sight_required_but_unavailable: bool,
    hearing_required: bool,
    fear_source_in_sight: bool,
    other: Circumstances,
) -> Result<TestDisposition, RulesError> {
    entity(rules, actor)?;
    let conditions = active_conditions(rules, actor);
    if sight_required_but_unavailable
        || (hearing_required && conditions.contains(&Condition::Deafened))
        || conditions.contains(&Condition::Unconscious)
    {
        return Ok(TestDisposition::AutomaticFailure);
    }
    Ok(TestDisposition::Roll(mode(
        other.advantage,
        other.disadvantage
            || conditions.contains(&Condition::Poisoned)
            || (conditions.contains(&Condition::Frightened) && fear_source_in_sight),
    )))
}

pub fn save_conditions(
    rules: &RulesState,
    actor: EntityId,
    ability: Ability,
    other: Circumstances,
    dodge: DodgeContext,
) -> Result<TestDisposition, RulesError> {
    entity(rules, actor)?;
    let conditions = active_conditions(rules, actor);
    if matches!(ability, Ability::Strength | Ability::Dexterity)
        && [
            Condition::Paralyzed,
            Condition::Petrified,
            Condition::Stunned,
            Condition::Unconscious,
        ]
        .iter()
        .any(|condition| conditions.contains(condition))
    {
        return Ok(TestDisposition::AutomaticFailure);
    }
    let dexterity = ability == Ability::Dexterity;
    let dodge = dodge.declared
        && dodge.effective_speed_units > 0
        && dexterity
        && !conditions.contains(&Condition::Incapacitated)
        && !conditions.contains(&Condition::Grappled)
        && !conditions.contains(&Condition::Restrained);
    Ok(TestDisposition::Roll(mode(
        other.advantage || dodge,
        other.disadvantage || (dexterity && conditions.contains(&Condition::Restrained)),
    )))
}

/// All speeds are affected by Exhaustion; Grappled/Restrained prevent any increase.
/// Lengths use the spatial model's half-foot units.
pub fn effective_speed(
    rules: &RulesState,
    actor: EntityId,
    base_units: u32,
) -> Result<u32, RulesError> {
    let entity = entity(rules, actor)?;
    let conditions = active_conditions(rules, actor);
    if entity.death.dead
        || [
            Condition::Grappled,
            Condition::Restrained,
            Condition::Paralyzed,
            Condition::Petrified,
            Condition::Unconscious,
        ]
        .iter()
        .any(|condition| conditions.contains(condition))
    {
        return Ok(0);
    }
    Ok(base_units.saturating_sub(u32::from(entity.exhaustion) * 10))
}

/// Speechlessness is an Incapacitated clause, not a Deafened clause.
pub fn can_speak(rules: &RulesState, actor: EntityId) -> Result<bool, RulesError> {
    let entity = entity(rules, actor)?;
    Ok(!entity.death.dead && !active_conditions(rules, actor).contains(&Condition::Incapacitated))
}
