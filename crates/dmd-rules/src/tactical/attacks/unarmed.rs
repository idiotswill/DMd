//! SRD190 Damage option: the body uses the ordinary paid Attack action and queue.
use super::*;

/// SRD177 applies worn-armor training to this Strength test. A shield alone
/// never imposes that penalty. This does not change historical OA interpretation.
pub(super) fn untrained_armor(state: &CampaignState, actor: EntityId) -> Result<bool, RulesError> {
    let Some(item) = state
        .rules
        .as_ref()
        .and_then(|rules| rules.tactical_inventory.as_ref())
        .and_then(|inventory| inventory.loadout(actor))
        .and_then(|loadout| loadout.worn_armor)
    else {
        return Ok(false);
    };
    let armor = state
        .items
        .get(&item)
        .ok_or_else(|| invalid("Worn armor is absent."))?;
    if armor.definition_id != "leather-armor" {
        return Err(prerequisite(
            "Worn armor requires a supported source training category.",
        ));
    }
    let trained_pc = state
        .table
        .as_ref()
        .and_then(|table| {
            table
                .character_profiles
                .values()
                .find(|profile| profile.entity_id == actor)
        })
        .is_some_and(|profile| {
            profile
                .armor_training
                .iter()
                .any(|category| category == "light")
        });
    let trained_creature = state
        .rules
        .as_ref()
        .and_then(|rules| rules.tactical_creatures.as_ref())
        .and_then(|creatures| creatures.profile(actor))
        .map(|profile| {
            crate::tactical_creatures::source_for_profile(profile)
                .map(|source| source.statistics.gear.contains(&armor.definition_id))
        })
        .transpose()
        .map_err(|error| invalid(&error.to_string()))?
        .unwrap_or(false);
    Ok(!trained_pc && !trained_creature)
}

pub(in crate::tactical) fn begin(
    state: &mut CampaignState,
    meta: &CommandMeta,
    target: EntityId,
) -> Result<(), RulesError> {
    if flow(state)?.phase != TacticalPhase::Active || flow(state)?.resolution.is_some() {
        return Err(RulesError::Pending);
    }
    let actor = active(state)?;
    authorize(state, meta, actor)?;
    if actor == target {
        return Err(prerequisite(
            "Self-directed attacks require their explicit adjudication path.",
        ));
    }
    planning::admit_target(state, actor, target)?;
    super::super::falling::require_settled_before_action(state)?;
    let rules = state.rules.as_ref().ok_or(RulesError::Uninitialized)?;
    if !crate::tactical_conditions::can_act(rules, actor)? {
        return Err(prerequisite("The actor cannot take an attack."));
    }
    let mut budget = flow(state)?.budget.clone();
    let new_action = budget.attacks_remaining == 0;
    let window = if new_action {
        WeaponActionWindow {
            id: meta.id,
            kind: WeaponActionKind::AttackAction,
        }
    } else {
        budget
            .attack_window
            .ok_or_else(|| invalid("Unarmed attack opportunity is absent."))?
    };
    if window.kind != WeaponActionKind::AttackAction {
        return Err(prerequisite(
            "An Unarmed Strike needs an Attack action attack.",
        ));
    }
    let mut attack = TacticalAttack {
        origin: meta.clone(),
        actor,
        target,
        delivery: TacticalAttackDelivery::Melee,
        source: TacticalAttackSource::Unarmed {
            ability: Ability::Strength,
        },
        admission: TacticalAttackAdmission::UnarmedAction { window },
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
    let plan = intrinsic::plan(state, &attack)?;
    attack.attack_modifier = plan.modifier;
    attack.mode = plan.mode;
    attack.armor_class = plan.armor;
    attack.critical_on_hit = plan.critical;
    attack.damage = plan.damage;
    let now = state.clock.now;
    let rules = state.rules.as_mut().ok_or(RulesError::Uninitialized)?;
    if new_action {
        crate::tactical_budget::start_attack_action(rules, &mut budget, actor, 1)?;
        budget.attack_window = Some(window);
    }
    crate::tactical_budget::spend_attack(&mut budget)?;
    crate::kernel::interrupt_rest(rules, actor, now);
    let turn_number = rules
        .timing
        .as_ref()
        .ok_or_else(|| invalid("Turn absent"))?
        .turn_number;
    budget.movement_progress = None;
    budget.movement_origin = None;
    flow_mut(state)?.budget = budget;
    let work_trace = super::super::work_trace::initial(state)?;
    flow_mut(state)?.resolution = Some(Box::new(TacticalResolution {
        origin: meta.clone(),
        turn_actor: actor,
        turn_number,
        boundary: TurnBoundary::Start,
        frames: vec![],
        pending: None,
        failed_save: None,
        legendary_window: None,
        hit_review: None,
        attack: Some(attack),
        movement: None,
        casts: vec![],
        falls: vec![],
        areas: vec![],
        work_trace,
        next_occurrence: 0,
    }));
    push_frame(state, vec![TacticalWorkKind::AttackRoll])?;
    pump(state, meta)
}

pub(super) fn validate_admission(
    state: &CampaignState,
    attack: &TacticalAttack,
    window: WeaponActionWindow,
) -> Result<(), RulesError> {
    let r = resolution(state)?;
    if attack.origin != r.origin
        || attack.actor != r.turn_actor
        || attack.actor != active(state)?
        || r.boundary != TurnBoundary::Start
        || r.movement.is_some()
        || !r.casts.is_empty()
        || !r.areas.is_empty()
        || !matches!(
            attack.source,
            TacticalAttackSource::Unarmed {
                ability: Ability::Strength
            }
        )
        || window.kind != WeaponActionKind::AttackAction
        || flow(state)?.budget.attack_window != Some(window)
        || !state
            .rules
            .as_ref()
            .and_then(|rules| rules.timing.as_ref())
            .is_some_and(|timing| timing.action_spent)
    {
        return Err(invalid(
            "Unarmed attack differs from its paid own-turn Attack action.",
        ));
    }
    Ok(())
}
