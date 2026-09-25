use super::*;
use crate::spatial::{cover_from, participant_distance, perceive};
use crate::tactical_conditions::{AttackPerception, attack_conditions};

fn spatial(error: crate::spatial::SpatialError) -> RulesError {
    invalid(&error.to_string())
}

pub(super) fn require_located_target(
    state: &CampaignState,
    actor: EntityId,
    target: EntityId,
) -> Result<(), RulesError> {
    let encounter = encounter(state)?;
    if encounter.participant(target).is_none()
        || !perceive(encounter, state, actor, target)
            .is_ok_and(|knowledge| knowledge.precisely_located)
    {
        return Err(prerequisite(
            "choose a currently located target; an unlocated target needs a guessed-location action",
        ));
    }
    Ok(())
}

/// Only for admitting a new vitality attack. Retained source reconstruction must
/// remain valid when that same attack has killed its target.
pub(super) fn admit_target(
    state: &CampaignState,
    actor: EntityId,
    target: EntityId,
) -> Result<(), RulesError> {
    require_located_target(state, actor, target)?;
    if state
        .rules
        .as_ref()
        .and_then(|rules| rules.entities.get(&target))
        .is_some_and(|target| target.death.dead)
    {
        return Err(prerequisite(
            "a dead body requires its body/object adjudication path",
        ));
    }
    Ok(())
}

pub(super) fn weapon_plan(
    state: &CampaignState,
    meta: &CommandMeta,
    actor: EntityId,
    choice: &WeaponUseChoice,
    window: WeaponActionWindow,
    loadout: &WeaponLoadout,
    pack: &RulesPack,
) -> Result<WeaponAttackPlan, RulesError> {
    let e = encounter(state)?;
    let from = e
        .participant(actor)
        .ok_or_else(|| invalid("attacker absent"))?;
    let to = e
        .participant(choice.target)
        .ok_or_else(|| prerequisite("target is outside this encounter"))?;
    let defs = definitions()?;
    let combatant = flow(state)?
        .combatants
        .iter()
        .find(|c| c.actor == actor)
        .ok_or_else(|| invalid("attacker source absent"))?;
    let source = match &combatant.source {
        TacticalSource::Character => WeaponActorSource::Character(
            state
                .table
                .as_ref()
                .and_then(|t| t.character_profiles.values().find(|p| p.entity_id == actor))
                .ok_or_else(|| invalid("character source profile absent"))?,
        ),
        TacticalSource::Creature { definition_id } => WeaponActorSource::CreatureOrdinaryWeapon(
            defs.creature(definition_id)
                .ok_or_else(|| invalid("creature source absent"))?,
        ),
    };
    let rules = state.rules.as_ref().ok_or(RulesError::Uninitialized)?;
    let timing = rules
        .timing
        .as_ref()
        .ok_or_else(|| invalid("turn absent"))?;
    let history = flow(state)?
        .budget
        .weapon_history
        .iter()
        .filter(|r| r.origin.id != meta.id)
        .cloned()
        .collect::<Vec<_>>();
    let trigger = match choice.purpose {
        WeaponAttackPurpose::Cleave { trigger } => history
            .iter()
            .find(|r| r.origin.id == trigger)
            .and_then(|r| e.participant(r.target)),
        _ => None,
    };
    let volume = from.volume().map_err(|error| invalid(&error))?;
    let underwater = e
        .battlefield
        .terrain
        .iter()
        .any(|t| t.water && t.volume.encloses(volume));
    if !underwater
        && e.battlefield
            .terrain
            .iter()
            .any(|t| t.water && t.volume.intersects(volume))
    {
        return Err(prerequisite(
            "partial submersion needs an explicit source geometry ruling",
        ));
    }
    prepare_weapon_attack(&WeaponAttackInput {
        state,
        source,
        pack,
        definitions: &defs,
        choice,
        context: WeaponAttackContext {
            origin: meta,
            actor,
            turn_number: timing.turn_number,
            on_actor_turn: active(state)? == actor,
            window,
            distance: participant_distance(from, to).map_err(spatial)?,
            base_reach: from.reach,
            mounted: false,
            underwater,
            has_swim_speed: from.movement.swim.is_some(),
            target_is_creature: matches!(
                state.entities[&choice.target].kind,
                EntityKind::Character | EntityKind::Npc | EntityKind::Creature
            ),
            target_size: to.size,
            distance_from_trigger_target: trigger
                .map(|p| participant_distance(p, to))
                .transpose()
                .map_err(spatial)?,
        },
        loadout,
        history: &history,
    })
    .map_err(weapon_error)
}
pub(super) fn hit_facts(
    state: &CampaignState,
    actor: EntityId,
    choice: &WeaponUseChoice,
    plan: &WeaponAttackPlan,
) -> Result<(RollMode, i32, bool), RulesError> {
    hit_facts_for(
        state,
        actor,
        choice.target,
        choice.delivery != WeaponDelivery::Melee,
        !plan.disadvantage.is_empty(),
    )
}

/// Common visibility/condition/cover facts. Intrinsic and spell adapters provide
/// their source-derived delivery and disadvantage without invented weapon IDs.
pub(super) fn hit_facts_for(
    state: &CampaignState,
    actor: EntityId,
    target_id: EntityId,
    ranged: bool,
    source_disadvantage: bool,
) -> Result<(RollMode, i32, bool), RulesError> {
    let e = encounter(state)?;
    let from = e
        .participant(actor)
        .ok_or_else(|| invalid("attacker absent"))?;
    let target = e
        .participant(target_id)
        .ok_or_else(|| invalid("target absent"))?;
    let seen = perceive(e, state, actor, target_id).map_err(spatial)?;
    if !seen.precisely_located {
        return Err(prerequisite(
            "an unlocated target requires an explicit guessed-location attack",
        ));
    }
    let reverse = perceive(e, state, target_id, actor).map_err(spatial)?;
    let rules = state.rules.as_ref().ok_or(RulesError::Uninitialized)?;
    let mut feared = false;
    for effect in crate::tactical_effect_adapter::condition_effects(rules)
        .filter(|f| f.target == actor && f.condition == Some(Condition::Frightened))
    {
        if e.participant(effect.source).is_some()
            && perceive(e, state, actor, effect.source)
                .map_err(spatial)?
                .sees
        {
            feared = true;
        }
    }
    let mut threat = false;
    for enemy in &from.enemies {
        let Some(other) = e.participant(*enemy) else {
            continue;
        };
        if participant_distance(from, other).map_err(spatial)? <= 10
            && crate::tactical_conditions::can_act(rules, *enemy)?
            && perceive(e, state, *enemy, actor).map_err(spatial)?.sees
        {
            threat = true;
        }
    }
    let mut source_advantage = false;
    if let Some(TacticalSource::Creature { definition_id }) = flow(state)?
        .combatants
        .iter()
        .find(|c| c.actor == actor)
        .map(|c| &c.source)
    {
        let definitions = definitions()?;
        let source = definitions
            .creature(definition_id)
            .ok_or_else(|| invalid("attacker source absent"))?;
        for property in &source.traits {
            if let crate::tactical_definitions::MonsterTrait::PackTactics { ally_distance_feet } =
                property
            {
                for ally in &from.allies {
                    let Some(other) = e.participant(*ally) else {
                        continue;
                    };
                    if *ally != actor
                        && participant_distance(other, target).map_err(spatial)?
                            <= u32::from(*ally_distance_feet) * 2
                        && crate::tactical_conditions::can_act(rules, *ally)?
                    {
                        source_advantage = true;
                    }
                }
            }
        }
    }
    let condition = attack_conditions(
        rules,
        actor,
        target_id,
        ranged,
        AttackPerception {
            attacker_sees_target: seen.sees,
            target_sees_attacker: reverse.sees,
            fear_source_in_sight: feared,
            within_five_feet: participant_distance(from, target).map_err(spatial)? <= 10,
            hostile_ranged_threat: threat,
        },
        Circumstances {
            advantage: source_advantage,
            disadvantage: source_disadvantage,
            ..Circumstances::default()
        },
        dodge_context(state, target_id)?,
    )?;
    let cover = cover_from(
        e,
        from.center().map_err(|error| invalid(&error))?,
        target.volume().map_err(|error| invalid(&error))?,
        &[actor, target_id],
    )
    .map_err(spatial)?
    .armor_and_dexterity_bonus()
    .map_err(spatial)?;
    let ac = crate::armor_class(
        rules
            .entities
            .get(&target_id)
            .ok_or_else(|| invalid("target mechanics absent"))?,
    ) + cover;
    Ok((condition.mode, ac, condition.critical_on_hit))
}

/// Undo only this attack's bounded, recorded reservation for source reconstruction.
/// No caller-defined after-state is replayed; accepted event history proves that this
/// original equipment/ammunition image was the one actually reserved at declaration.
pub(super) fn reconstruct(
    state: &CampaignState,
    attack: &TacticalAttack,
) -> Result<WeaponAttackPlan, RulesError> {
    let weapon = attack
        .weapon()
        .ok_or_else(|| invalid("attack source is not a physical weapon"))?;
    let mut before = state.clone();
    before.applied_event_sequence = attack.origin.expected_event_sequence;
    if let Some(ammo) = &weapon.ammunition {
        let stack = before
            .items
            .get_mut(&ammo.stack)
            .ok_or_else(|| invalid("reserved ammunition absent"))?;
        stack.quantity = ammo.quantity_before;
        stack.state = ItemState::Intact;
    }
    let rules = before.rules.as_mut().ok_or(RulesError::Uninitialized)?;
    let current = rules
        .tactical_inventory
        .as_mut()
        .and_then(|i| i.loadouts.iter_mut().find(|l| l.actor == attack.actor))
        .ok_or_else(|| invalid("reserved equipment absent"))?;
    *current = weapon.equipment_before.clone();
    let pack = RulesPack::from_json(include_str!("../../../../../content/srd-5.2.1/kernel.json"))?;
    weapon_plan(
        &before,
        &attack.origin,
        attack.actor,
        &weapon.choice,
        weapon.window,
        &weapon.equipment_before.hands,
        &pack,
    )
}
