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
pub(in crate::tactical) fn admit_target(
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
    Ok(prepare_weapon(state, meta, actor, choice, window, loadout, pack)?.into_plan())
}

pub(super) fn prepare_weapon<'a>(
    state: &'a CampaignState,
    meta: &CommandMeta,
    actor: EntityId,
    choice: &WeaponUseChoice,
    window: WeaponActionWindow,
    loadout: &WeaponLoadout,
    pack: &RulesPack,
) -> Result<crate::tactical_weapons::ground::PreparedPhysicalAttack<'a>, RulesError> {
    if is_ground_pickup(choice.equipment_change)
        || (choice.after_equipment.is_some() && flow(state)?.attack_equipment_access.is_some())
    {
        super::super::attack_equipment_access::require_origin(state, meta)?;
    }
    let calculation = calculation(state, meta, actor, choice, window, loadout, pack)?;
    crate::tactical_weapons::ground::PreparedPhysicalAttack::new(state, &calculation.input())
        .map_err(weapon_error)
}

struct PhysicalCalculation<'a> {
    state: &'a CampaignState,
    source: WeaponActorSource<'a>,
    pack: &'a RulesPack,
    definitions: &'a crate::tactical_definitions::TacticalDefinitions,
    choice: &'a WeaponUseChoice,
    context: WeaponAttackContext<'a>,
    loadout: &'a WeaponLoadout,
    history: Vec<WeaponAttackReceipt>,
}

impl PhysicalCalculation<'_> {
    fn input(&self) -> WeaponAttackInput<'_> {
        WeaponAttackInput {
            state: self.state,
            source: self.source,
            pack: self.pack,
            definitions: self.definitions,
            choice: self.choice,
            context: self.context,
            loadout: self.loadout,
            history: &self.history,
        }
    }
}

fn calculation<'a>(
    state: &'a CampaignState,
    meta: &'a CommandMeta,
    actor: EntityId,
    choice: &'a WeaponUseChoice,
    window: WeaponActionWindow,
    loadout: &'a WeaponLoadout,
    pack: &'a RulesPack,
) -> Result<PhysicalCalculation<'a>, RulesError> {
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
            crate::tactical_creatures::source_for_actor(state, actor, definition_id)
                .map_err(|e| invalid(&e.to_string()))?,
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
    Ok(PhysicalCalculation {
        state,
        source,
        pack,
        definitions: defs,
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
        history,
    })
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
        let source = crate::tactical_creatures::source_for_actor(state, actor, definition_id)
            .map_err(|e| invalid(&e.to_string()))?;
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
    let ac = super::super::hit_reactions::original_defense(state, actor, target_id)? + cover;
    Ok((condition.mode, ac, condition.critical_on_hit))
}

/// Undo only this attack's bounded, recorded reservation for source reconstruction.
/// No caller-defined after-state is replayed; accepted event history proves that this
/// original equipment/ammunition image was the one actually reserved at declaration.
pub(super) fn reconstruct(
    state: &CampaignState,
    attack: &TacticalAttack,
) -> Result<WeaponAttackPlan, RulesError> {
    // Current reservations cannot stand in for an attack's original admission.
    // The later resolver must authenticate its historical cut before enabling it.
    if has_unimplemented_grapple_records(state) {
        return Err(invalid(
            "Grapple attack reconstruction requires original admission proof",
        ));
    }
    let weapon = attack
        .weapon()
        .ok_or_else(|| invalid("attack source is not a physical weapon"))?;
    if is_ground_pickup(weapon.choice.equipment_change)
        || (weapon.choice.after_equipment.is_some()
            && flow(state)?.attack_equipment_access.is_some())
    {
        let pack =
            RulesPack::from_json(include_str!("../../../../../content/srd-5.2.1/kernel.json"))?;
        let retained =
            crate::tactical_weapons::ground::RetainedPhysicalRead::new(state, attack, &pack)
                .map_err(weapon_error)?;
        let calculation = calculation(
            retained.state(),
            &attack.origin,
            attack.actor,
            &weapon.choice,
            weapon.window,
            &weapon.equipment_before.hands,
            &pack,
        )?;
        return retained
            .calculate(&calculation.input())
            .map_err(weapon_error);
    }
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
    crate::tactical_weapons::ground::restore_before_image(state, &mut before, attack, &pack)
        .map_err(weapon_error)?;
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

/// Actual private completion callers obtain their attached declaration here.
/// Entered pump work and a persisted direct material choice have different
/// validators; neither is represented by a caller flag or a rewritten stage.
pub(super) fn reconstruct_completion(
    state: &CampaignState,
    meta: &CommandMeta,
) -> Result<WeaponAttackPlan, RulesError> {
    let attack = current(state)?;
    let weapon = attack
        .weapon()
        .ok_or_else(|| invalid("physical attack absent"))?;
    if !is_ground_pickup(weapon.choice.equipment_change)
        && !(weapon.choice.after_equipment.is_some()
            && flow(state)?.attack_equipment_access.is_some())
    {
        return reconstruct(state, attack);
    }
    if has_unimplemented_grapple_records(state) {
        return Err(invalid(
            "Grapple attack reconstruction requires original admission proof",
        ));
    }
    let pack = RulesPack::from_json(include_str!("../../../../../content/srd-5.2.1/kernel.json"))?;
    let reader = if resolution(state)?
        .work_trace
        .as_ref()
        .is_some_and(|t| t.active.is_some())
    {
        crate::tactical_weapons::ground::RetainedPhysicalRead::entered_completion(
            state, meta, &pack,
        )
    } else {
        crate::tactical_weapons::ground::RetainedPhysicalRead::material_choice(state, meta, &pack)
    }
    .map_err(weapon_error)?;
    let calculation = calculation(
        reader.state(),
        &attack.origin,
        attack.actor,
        &weapon.choice,
        weapon.window,
        &weapon.equipment_before.hands,
        &pack,
    )?;
    reader.calculate(&calculation.input()).map_err(weapon_error)
}

#[cfg(test)]
mod hand_tests {
    use super::*;

    #[test]
    fn genuine_old_before_image_survives_but_new_current_hands_cannot_authenticate_it() {
        let export: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../dmd-app/tests/fixtures/shield-hit-v1-selected.json"
        ))
        .unwrap();
        let mut state: CampaignState =
            serde_json::from_str(export["current_state"]["state_json"].as_str().unwrap()).unwrap();
        let attack = state
            .encounter
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .resolution
            .as_ref()
            .unwrap()
            .attack
            .as_ref()
            .unwrap()
            .clone();
        let weapon = attack.weapon().unwrap();
        assert_eq!(
            weapon.equipment_before.hands.hands,
            [HandAssignment::Free; 2]
        );
        let before = serde_json::to_value(&state).unwrap();
        let original = reconstruct(&state, &attack).unwrap();
        assert_eq!(
            original.loadout_for_attack.hands[Hand::Right.index()],
            HandAssignment::Item(weapon.choice.weapon)
        );
        assert_eq!(serde_json::to_value(&state).unwrap(), before);

        // Pure historical boundary control, never an accepted Grapple history:
        // a source-shaped current Attempt cannot prove this older admission.
        crate::tactical_hands::tests::install_attempt(&mut state, attack.actor, Hand::Left);
        let guarded = serde_json::to_value(&state).unwrap();
        assert!(
            reconstruct(&state, &attack)
                .unwrap_err()
                .to_string()
                .contains("requires original admission proof")
        );
        assert_eq!(serde_json::to_value(&state).unwrap(), guarded);
    }
}
