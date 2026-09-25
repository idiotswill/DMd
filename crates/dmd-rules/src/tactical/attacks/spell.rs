//! A sealed source spell occurrence joins the existing attack work. The casting
//! driver owns source admission, payment and ordering; this leaf owns live hit facts.
use super::*;
use crate::tactical_spells::{
    SpellAttackOccurrence, retained_spell_binding, spell_attack_occurrence,
};

fn record(state: &CampaignState, cast: u16) -> Result<&TacticalCasting, RulesError> {
    let mut matches = resolution(state)?
        .casts
        .iter()
        .filter(|record| record.cast.plan.occurrence == cast);
    let record = matches
        .next()
        .ok_or_else(|| invalid("spell attack lacks its retained cast"))?;
    if matches.next().is_some() {
        return Err(invalid("spell attack cast identity is ambiguous"));
    }
    Ok(record)
}

fn proof(
    state: &CampaignState,
    cast: u16,
    at: SpellProgramOccurrence,
) -> Result<SpellAttackOccurrence, RulesError> {
    let record = record(state, cast)?;
    let bound = retained_spell_binding(record)?;
    spell_attack_occurrence(&record.cast, &bound, at.node, at.target)
}

/// This source path binds living creatures at cast admission. A later ray can
/// find that a previous occurrence killed its target; finish it without invoking
/// ordinary vitality on the dead or manufacturing another damage request.
fn applicable(state: &CampaignState, proof: &SpellAttackOccurrence) -> Result<bool, RulesError> {
    let rules = state.rules.as_ref().ok_or(RulesError::Uninitialized)?;
    let possible = rules
        .entities
        .get(&proof.target())
        .is_some_and(|target| !target.death.dead)
        && state.entities.get(&proof.target()).is_some_and(|target| {
            matches!(
                target.kind,
                EntityKind::Character | EntityKind::Npc | EntityKind::Creature
            )
        })
        && state
            .encounter
            .as_ref()
            .is_some_and(|encounter| encounter.participant(proof.target()).is_some());
    Ok(possible
        && crate::tactical_conditions::can_act(rules, proof.actor())?
        && crate::tactical_conditions::may_harm(rules, proof.actor(), proof.target()))
}

fn fill_facts(
    state: &CampaignState,
    attack: &mut TacticalAttack,
    proof: &SpellAttackOccurrence,
) -> Result<(), RulesError> {
    planning::require_located_target(state, proof.actor(), proof.target())?;
    let rules = state.rules.as_ref().ok_or(RulesError::Uninitialized)?;
    let caster = rules
        .entities
        .get(&proof.actor())
        .ok_or_else(|| invalid("spell attacker mechanics are absent"))?;
    // The currently executable AttackDamage source node is created only from
    // RangedSpellAttack. Future melee source programs need an explicit delivery.
    let (mode, armor, critical) =
        planning::hit_facts_for(state, proof.actor(), proof.target(), true, false)?;
    attack.delivery = TacticalAttackDelivery::Ranged;
    attack.attack_modifier =
        i32::from(proof.intrinsic_attack_bonus()) - i32::from(caster.exhaustion) * 2;
    attack.damage = vec![AttackDamageComponent {
        damage_type: proof.damage().damage_type,
        dice: proof.damage().dice.clone(),
        modifier: i32::from(proof.damage().modifier),
    }];
    attack.mode = mode;
    attack.armor_class = armor;
    attack.critical_on_hit = critical;
    Ok(())
}

pub(in crate::tactical) fn begin_spell_attack(
    state: &mut CampaignState,
    causal_meta: &CommandMeta,
    occurrence: &SpellAttackOccurrence,
) -> Result<(), RulesError> {
    let r = resolution(state)?;
    if r.attack.is_some() || r.pending.is_some() || r.failed_save.is_some() {
        return Err(RulesError::Pending);
    }
    let at = SpellProgramOccurrence {
        node: occurrence.node_ordinal(),
        target: occurrence.target_ordinal(),
    };
    let canonical = proof(state, occurrence.cast_occurrence(), at)?;
    if canonical != *occurrence {
        return Err(invalid(
            "spell attack proof differs from the live retained cast",
        ));
    }
    validate_equipment_change_origin(state, causal_meta, occurrence.actor())
        .map_err(|e| invalid(&e))?;
    if causal_meta.expected_event_sequence != state.applied_event_sequence
        || causal_meta.expected_event_sequence < r.origin.expected_event_sequence
        || causal_meta.expected_event_sequence < occurrence.origin().expected_event_sequence
        || (causal_meta.expected_event_sequence == occurrence.origin().expected_event_sequence
            && causal_meta != occurrence.origin())
    {
        return Err(RulesError::Stale);
    }
    if !applicable(state, occurrence)? {
        return super::super::casting::complete_occurrence(state, occurrence.cast_occurrence(), at);
    }
    let mut attack = TacticalAttack {
        origin: causal_meta.clone(),
        actor: occurrence.actor(),
        target: occurrence.target(),
        delivery: TacticalAttackDelivery::Ranged,
        source: TacticalAttackSource::Spell {
            source: occurrence.source().clone(),
            cast: occurrence.cast_occurrence(),
            at,
        },
        admission: TacticalAttackAdmission::Spell {
            casting_origin: occurrence.origin().clone(),
        },
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
    fill_facts(state, &mut attack, occurrence)?;
    // No action, spell slot, feature use, ammunition or equipment changes occur
    // here. The single casting transaction has already reserved its own costs.
    resolution_mut(state)?.attack = Some(attack);
    push_frame(state, vec![TacticalWorkKind::AttackRoll])?;
    // The caller is the shared pump; recursively pumping here would erase its
    // parent cast before the caller resumes processing that same work item.
    Ok(())
}

pub(super) fn validate_admission(
    state: &CampaignState,
    attack: &TacticalAttack,
) -> Result<(), RulesError> {
    let TacticalAttackAdmission::Spell { casting_origin } = &attack.admission else {
        return Err(invalid("spell attack lacks its casting admission"));
    };
    let TacticalAttackSource::Spell { source, cast, at } = &attack.source else {
        return Err(invalid("spell admission has a different attack source"));
    };
    let occurrence = proof(state, *cast, *at)?;
    validate_equipment_change_origin(state, casting_origin, attack.actor)
        .map_err(|e| invalid(&e))?;
    if occurrence.origin() != casting_origin
        || occurrence.actor() != attack.actor
        || occurrence.target() != attack.target
        || occurrence.source() != source
        || casting_origin.expected_event_sequence > attack.origin.expected_event_sequence
        || (casting_origin.expected_event_sequence == attack.origin.expected_event_sequence
            && *casting_origin != attack.origin)
        || attack.origin.expected_event_sequence < resolution(state)?.origin.expected_event_sequence
    {
        return Err(invalid(
            "spell attack source/cause differs from retained casting",
        ));
    }
    Ok(())
}

pub(super) fn validate_source(
    state: &CampaignState,
    attack: &TacticalAttack,
) -> Result<(), RulesError> {
    validate_admission(state, attack)?;
    let TacticalAttackSource::Spell { cast, at, .. } = &attack.source else {
        return Err(invalid("not a spell attack source"));
    };
    let occurrence = proof(state, *cast, *at)?;
    if !applicable(state, &occurrence)?
        || matches!(
            attack.stage,
            TacticalAttackStage::KnockoutChoice | TacticalAttackStage::MasteryChoice
        )
        || attack.automatic_miss
    {
        return Err(invalid("spell attack retains an inapplicable source stage"));
    }
    let mut derived = attack.clone();
    fill_facts(state, &mut derived, &occurrence)?;
    if derived != *attack {
        return Err(invalid(
            "spell attack differs from live source-derived hit facts",
        ));
    }
    Ok(())
}

pub(super) fn complete(
    state: &mut CampaignState,
    attack: &TacticalAttack,
) -> Result<(), RulesError> {
    let TacticalAttackSource::Spell { cast, at, .. } = &attack.source else {
        return Err(invalid("not a spell attack completion"));
    };
    // Mark first: damage may enqueue a save owned by another actor. Restarting
    // after that save must resume the next ray, never reapply this occurrence.
    super::super::casting::complete_occurrence(state, *cast, *at)
}
