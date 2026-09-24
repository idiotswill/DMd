use super::*;

pub(super) fn validate(state: &CampaignState) -> Result<(), RulesError> {
    let Some(resolution) = flow(state)?.resolution.as_ref() else {
        return Ok(());
    };
    let Some(attack) = &resolution.attack else {
        if resolution
            .frames
            .iter()
            .flatten()
            .chain(resolution.pending.iter().map(|p| &p.work))
            .any(|w| {
                matches!(
                    w.kind,
                    TacticalWorkKind::AttackRoll
                        | TacticalWorkKind::AttackDamage
                        | TacticalWorkKind::FinishAttack
                )
            })
        {
            return Err(invalid("attack work lacks its retained declaration"));
        }
        return Ok(());
    };
    validate_equipment_origin(state, &attack.origin, attack.actor).map_err(|e| invalid(&e))?;
    authorize(state, &attack.origin, attack.actor)?;
    opportunity::validate_admission(state, attack)?;
    if attack.automatic_miss {
        return Err(invalid(
            "an automatic miss must finish without accepting an attack roll",
        ));
    }
    match attack.stage {
        TacticalAttackStage::AttackRoll
            if attack.attack_roll.is_some()
                || attack.damage_roll.is_some()
                || attack.outcome.is_some() =>
        {
            return Err(invalid("unrolled attack retains a completed outcome"));
        }
        TacticalAttackStage::DamageRoll
            if attack.damage_roll.is_some()
                || !matches!(
                    attack.outcome,
                    Some(WeaponAttackOutcome::Hit {
                        damage_dealt: 0,
                        ..
                    })
                ) =>
        {
            return Err(invalid("unrolled damage retains a completed outcome"));
        }
        TacticalAttackStage::KnockoutChoice
            if !matches!(
                attack.outcome,
                Some(WeaponAttackOutcome::Hit {
                    damage_dealt: 0,
                    ..
                })
            ) || (!attack.damage.iter().all(|c| c.dice.is_empty())
                && attack.damage_roll.is_none()) =>
        {
            return Err(invalid("knockout choice lacks its accepted damage"));
        }
        TacticalAttackStage::MasteryChoice
            if attack.outcome != Some(WeaponAttackOutcome::Miss)
                || attack.damage_roll.is_some() =>
        {
            return Err(invalid("Graze choice lacks a miss"));
        }
        TacticalAttackStage::Finishing => {
            return Err(invalid(
                "attack completion must be pumped before persistence",
            ));
        }
        _ => (),
    }
    if attack.actor == attack.target
        || attack.damage.is_empty()
        || attack.damage.len() > 16
        || !(-1000..=1000).contains(&attack.attack_modifier)
        || !(-1000..=1000).contains(&attack.armor_class)
    {
        return Err(invalid("invalid retained attack identity/bounds"));
    }
    validate_source(state, attack)?;
    let rules = state.rules.as_ref().ok_or(RulesError::Uninitialized)?;
    if let Some(id) = attack.attack_roll {
        let record = rules
            .rolls
            .iter()
            .find(|r| r.request.id == id)
            .ok_or_else(|| invalid("accepted attack roll missing"))?;
        let PendingPurpose::TacticalResolution {
            encounter: id_encounter,
            key,
        } = record.purpose
        else {
            return Err(invalid("attack history has a different purpose"));
        };
        let work = TacticalWorkItem {
            occurrence: key.occurrence,
            kind: TacticalWorkKind::AttackRoll,
        };
        let expected = super::key(state, &work)?;
        if id_encounter != encounter(state)?.id
            || key != expected
            || key.occurrence >= resolution.next_occurrence
            || record.issued_by != attack.origin
            || record.request
                != request(state, &work, key)?.ok_or_else(|| invalid("attack request absent"))?
            || record.request.resolve(&record.result)? != record.resolved
        {
            return Err(invalid("accepted attack request differs"));
        }
        let face = record.resolved.kept_dice[0].value;
        let hit = face != 1 && (face == 20 || record.resolved.total >= attack.armor_class);
        if match attack.outcome {
            Some(WeaponAttackOutcome::Hit { critical, .. }) => {
                !hit || critical != (face == 20 || attack.critical_on_hit)
            }
            Some(WeaponAttackOutcome::Miss) => hit,
            _ => true,
        } {
            return Err(invalid("attack outcome differs from accepted die"));
        }
    } else if !attack.automatic_miss && attack.stage != TacticalAttackStage::AttackRoll {
        return Err(invalid("attack outcome lacks accepted dice"));
    }
    if let Some(id) = attack.damage_roll {
        let record = rules
            .rolls
            .iter()
            .find(|r| r.request.id == id)
            .ok_or_else(|| invalid("accepted damage roll missing"))?;
        let PendingPurpose::TacticalResolution {
            encounter: id_encounter,
            key,
        } = record.purpose
        else {
            return Err(invalid("damage history has a different purpose"));
        };
        let work = TacticalWorkItem {
            occurrence: key.occurrence,
            kind: TacticalWorkKind::AttackDamage,
        };
        let issued = attack
            .attack_roll
            .and_then(|id| rules.rolls.iter().find(|r| r.request.id == id))
            .ok_or_else(|| invalid("damage has no accepted attack"))?;
        if id_encounter != encounter(state)?.id
            || key != super::key(state, &work)?
            || key.occurrence >= resolution.next_occurrence
            || record.issued_by != issued.accepted_by
            || record.request
                != request(state, &work, key)?.ok_or_else(|| invalid("damage request absent"))?
            || record.request.resolve(&record.result)? != record.resolved
        {
            return Err(invalid("accepted damage request differs"));
        }
    }
    if let Some(pending) = &resolution.pending {
        let expected_origin = match pending.work.kind {
            TacticalWorkKind::AttackRoll => Some(&attack.origin),
            TacticalWorkKind::AttackDamage => Some(
                &rules
                    .rolls
                    .iter()
                    .find(|r| Some(r.request.id) == attack.attack_roll)
                    .ok_or_else(|| invalid("damage request lacks an attack record"))?
                    .accepted_by,
            ),
            _ => None,
        };
        if expected_origin.is_some_and(|origin| {
            rules
                .pending
                .as_ref()
                .is_none_or(|p| &p.issued_by != origin)
        }) {
            return Err(invalid(
                "attack request issuance differs from its accepted cause",
            ));
        }
    }
    if attack.stage == TacticalAttackStage::KnockoutChoice {
        let context = crate::tactical_vitality_adapter::context(
            state,
            attack.target,
            VitalityOrigin {
                command: attack.origin.clone(),
                occurrence: 0,
            },
        )?;
        let recovery = rules
            .tactical_recovery
            .as_ref()
            .and_then(|r| r.get(&attack.target))
            .cloned()
            .unwrap_or_default();
        let operation = VitalityOperation::Damage {
            packet: packet(state)?,
            knockout: None,
        };
        let preview = crate::tactical_damage::reduce_vitality(
            &rules.entities[&attack.target],
            &recovery,
            &context,
            &operation,
        )
        .map_err(|e| invalid(&e.to_string()))?;
        if !preview.awaiting_choice {
            return Err(invalid("knockout decision is not due for this damage"));
        }
    }
    let work = resolution
        .frames
        .iter()
        .flatten()
        .chain(resolution.pending.iter().map(|p| &p.work))
        .filter(|w| {
            matches!(
                w.kind,
                TacticalWorkKind::AttackRoll
                    | TacticalWorkKind::AttackDamage
                    | TacticalWorkKind::FinishAttack
            )
        })
        .collect::<Vec<_>>();
    let expected = match attack.stage {
        TacticalAttackStage::AttackRoll => Some(TacticalWorkKind::AttackRoll),
        TacticalAttackStage::DamageRoll => Some(TacticalWorkKind::AttackDamage),
        TacticalAttackStage::Finishing => Some(TacticalWorkKind::FinishAttack),
        TacticalAttackStage::KnockoutChoice | TacticalAttackStage::MasteryChoice => None,
    };
    if expected
        .as_ref()
        .is_some_and(|kind| work.len() != 1 || work[0].kind != *kind)
        || expected.is_none() && !work.is_empty()
    {
        return Err(invalid("attack stage differs from shared work"));
    }
    if matches!(
        attack.stage,
        TacticalAttackStage::KnockoutChoice | TacticalAttackStage::MasteryChoice
    ) && (resolution.pending.is_some()
        || resolution.failed_save.is_some()
        || resolution.legendary_window.is_some()
        || resolution.frames.iter().flatten().any(|w| {
            !matches!(
                w.kind,
                TacticalWorkKind::MoveSegment | TacticalWorkKind::MovementOpportunity { .. }
            )
        }))
    {
        return Err(invalid("attack decision competes with pending work"));
    }
    Ok(())
}

fn validate_source(state: &CampaignState, attack: &TacticalAttack) -> Result<(), RulesError> {
    let Some(weapon) = attack.weapon() else {
        let plan = intrinsic::plan(state, attack)?;
        if attack.stage == TacticalAttackStage::MasteryChoice
            || attack.automatic_miss
            || attack.attack_modifier != plan.modifier
            || attack.damage != plan.damage
            || (attack.mode, attack.armor_class, attack.critical_on_hit)
                != (plan.mode, plan.armor, plan.critical)
        {
            return Err(invalid(
                "intrinsic attack differs from canonical live source",
            ));
        }
        return Ok(());
    };
    if attack.target != weapon.choice.target
        || (attack.delivery == TacticalAttackDelivery::Melee)
            != (weapon.choice.delivery == WeaponDelivery::Melee)
        || weapon.equipment_before.actor != attack.actor
        || weapon.equipment_before.command.expected_event_sequence
            > attack.origin.expected_event_sequence
    {
        return Err(invalid(
            "attack target/delivery/equipment differs from its physical choice",
        ));
    }
    validate_equipment_change_origin(state, &weapon.equipment_before.command, attack.actor)
        .map_err(|e| invalid(&e))?;
    let plan = planning::reconstruct(state, attack)?;
    if plan
        .mastery
        .is_some_and(|m| !matches!(m, WeaponMastery::Nick | WeaponMastery::Graze))
        || (attack.stage == TacticalAttackStage::MasteryChoice
            && plan.mastery != Some(WeaponMastery::Graze))
    {
        return Err(invalid(
            "retained attack has no supported source mastery continuation",
        ));
    }
    let rules = state.rules.as_ref().ok_or(RulesError::Uninitialized)?;
    let timing = rules
        .timing
        .as_ref()
        .ok_or_else(|| invalid("attack timing absent"))?;
    if matches!(attack.admission, TacticalAttackAdmission::Opportunity(_)) {
        if weapon.window
            != (WeaponActionWindow {
                id: attack.origin.id,
                kind: WeaponActionKind::Reaction,
            })
            || weapon.choice.purpose != WeaponAttackPurpose::Normal
            || weapon.choice.delivery != WeaponDelivery::Melee
            || weapon.choice.equipment_change.is_some()
            || weapon.ammunition.is_some()
        {
            return Err(invalid(
                "reaction source differs from its one melee opportunity",
            ));
        }
    } else {
        match weapon.choice.purpose {
            WeaponAttackPurpose::Normal | WeaponAttackPurpose::Nick { .. }
                if !timing.action_spent
                    || flow(state)?.budget.attack_window != Some(weapon.window) =>
            {
                return Err(invalid("attack lacks its spent action opportunity"));
            }
            WeaponAttackPurpose::LightBonus { .. }
                if !timing.bonus_action_spent
                    || weapon.window
                        != (WeaponActionWindow {
                            id: attack.origin.id,
                            kind: WeaponActionKind::BonusAction,
                        }) =>
            {
                return Err(invalid("Light attack lacks its spent bonus action"));
            }
            WeaponAttackPurpose::Cleave { .. } => {
                return Err(invalid("Cleave continuation is not yet supported"));
            }
            _ => (),
        }
    }
    let inventory = rules
        .tactical_inventory
        .as_ref()
        .ok_or_else(|| invalid("attack equipment absent"))?;
    let equipped = inventory
        .loadout(attack.actor)
        .ok_or_else(|| invalid("attack loadout absent"))?;
    if equipped.hands != plan.loadout_for_attack
        || equipped.worn_armor != weapon.equipment_before.worn_armor
        || equipped.shield != weapon.equipment_before.shield
        || equipped.command != attack.origin
        || attack.attack_modifier != plan.attack_modifier
        || attack.automatic_miss != plan.automatic_miss
        || attack.damage
            != [AttackDamageComponent {
                damage_type: plan.damage.damage_type,
                dice: plan.damage.dice.clone(),
                modifier: plan.damage.modifier,
            }]
    {
        return Err(invalid("attack source or reserved equipment differs"));
    }
    match (&weapon.ammunition, &plan.ammunition) {
        (Some(reserved), Some(spend)) => {
            let item = state
                .items
                .get(&reserved.stack)
                .ok_or_else(|| invalid("reserved ammunition missing"))?;
            if reserved.stack != spend.stack
                || reserved.quantity_before == 0
                || reserved.quantity_before.checked_sub(spend.quantity) != Some(item.quantity)
                || item.state
                    != if item.quantity == 0 {
                        ItemState::Spent
                    } else {
                        ItemState::Intact
                    }
            {
                return Err(invalid("ammunition reservation differs"));
            }
        }
        (None, None) => (),
        _ => return Err(invalid("ammunition source differs")),
    }
    let receipts = flow(state)?
        .budget
        .weapon_history
        .iter()
        .filter(|r| r.origin.id == attack.origin.id)
        .collect::<Vec<_>>();
    if receipts.len() != 1 || *receipts[0] != plan.receipt {
        return Err(invalid("pending attack receipt differs"));
    }
    {
        let (mode, armor, critical) =
            planning::hit_facts(state, attack.actor, &weapon.choice, &plan)?;
        if (attack.mode, attack.armor_class, attack.critical_on_hit) != (mode, armor, critical) {
            return Err(invalid("pending attack circumstances differ"));
        }
    }
    Ok(())
}
