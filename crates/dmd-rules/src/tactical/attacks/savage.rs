//! SRD87: repeat the source weapon dice, preserving all other damage dice.
use super::*;

/// Read-only source availability. The application must separately require the
/// audience's actual pending roll capability before exposing these private facts.
pub fn savage_attacker_dice(state: &CampaignState, pack: &RulesPack) -> Result<usize, RulesError> {
    let rules = state.rules.as_ref().ok_or(RulesError::Uninitialized)?;
    let pending = rules.pending.as_ref().ok_or(RulesError::NoPending)?;
    super::super::validation::validate_tactical_pending(state, pending)?;
    let attack = current(state)?;
    if !matches!(attack.source, TacticalAttackSource::Weapon(_))
        || attack.stage != TacticalAttackStage::DamageRoll
        || !matches!(attack.outcome, Some(WeaponAttackOutcome::Hit { .. }))
        || !matches!(pending.purpose, PendingPurpose::TacticalResolution { key, .. }
            if key.role == TacticalRollRole::AttackDamage && key.subject == attack.target)
    {
        return Err(prerequisite(
            "Savage Attacker requires a confirmed weapon hit",
        ));
    }
    let profile = state
        .table
        .as_ref()
        .and_then(|table| {
            table
                .character_profiles
                .values()
                .find(|p| p.entity_id == attack.actor)
        })
        .ok_or_else(|| prerequisite("Savage Attacker requires its created source profile"))?;
    let entity = rules
        .entities
        .get(&attack.actor)
        .ok_or_else(|| invalid("attacker absent"))?;
    crate::validate_character_intrinsics(profile, entity, pack)?;
    let features = entity
        .character_features
        .as_ref()
        .ok_or_else(|| prerequisite("Savage Attacker is not granted"))?;
    let turn = rules
        .timing
        .as_ref()
        .ok_or_else(|| invalid("attack turn absent"))?
        .turn_number;
    if !features.savage_attacker || features.savage_attacker_turn == Some(turn) {
        return Err(prerequisite("Savage Attacker is unavailable this turn"));
    }
    let plan = planning::reconstruct(state, attack)?;
    let critical = matches!(
        attack.outcome,
        Some(WeaponAttackOutcome::Hit { critical: true, .. })
    );
    let count = plan
        .damage
        .dice
        .iter()
        .map(|die| usize::from(die.count))
        .sum::<usize>()
        * if critical { 2 } else { 1 };
    if count == 0 {
        return Err(prerequisite("this weapon has no damage dice"));
    }
    Ok(count)
}

pub(in crate::tactical) fn submit(
    state: &mut CampaignState,
    meta: &CommandMeta,
    roll: &SavageAttackerRoll,
    pack: &RulesPack,
) -> Result<(), RulesError> {
    let pending = state
        .rules
        .as_ref()
        .and_then(|rules| rules.pending.as_ref())
        .ok_or(RulesError::NoPending)?
        .clone();
    let actor = pending
        .request
        .roller
        .ok_or_else(|| invalid("damage roller absent"))?;
    authorize(state, meta, actor)?;
    let count = savage_attacker_dice(state, pack)?;
    if roll.weapon_dice != Some(count) {
        return Err(invalid(
            "Savage Attacker dice differ from the source weapon",
        ));
    }
    let selected = crate::kernel::savage_result(&pending.request, roll)?;
    let rules = state.rules.as_mut().ok_or(RulesError::Uninitialized)?;
    let turn = rules
        .timing
        .as_ref()
        .ok_or_else(|| invalid("attack turn absent"))?
        .turn_number;
    let entity = rules
        .entities
        .get_mut(&actor)
        .ok_or_else(|| invalid("attacker absent"))?;
    if roll.inspiration.is_some() {
        if !entity.heroic_inspiration {
            return Err(prerequisite("Heroic Inspiration unavailable"));
        }
        entity.heroic_inspiration = false;
    }
    entity
        .character_features
        .as_mut()
        .unwrap()
        .savage_attacker_turn = Some(turn);
    super::super::continuations::submit(state, meta, &selected, None)?;
    state
        .rules
        .as_mut()
        .unwrap()
        .rolls
        .iter_mut()
        .find(|record| record.request.id == pending.request.id)
        .ok_or_else(|| invalid("accepted Savage Attacker damage record absent"))?
        .savage_attacker = Some(roll.clone());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pools() -> (RollRequest, SavageAttackerRoll) {
        let request = RollRequest {
            id: RollRequestId::new(),
            roller: Some(EntityId::new()),
            dice: vec![
                DieSpec { count: 2, sides: 6 },
                DieSpec { count: 1, sides: 8 },
            ],
            modifier: 3,
            mode: RollMode::Normal,
            visibility: RollVisibility::Public,
            reason: "Damage".into(),
        };
        let raw = |values: [u16; 3]| RollResult {
            request_id: request.id,
            source: RollSource::Physical,
            dice: [6, 6, 8]
                .into_iter()
                .zip(values)
                .map(|(sides, value)| DieResult { sides, value })
                .collect(),
        };
        let roll = SavageAttackerRoll {
            weapon_dice: Some(2),
            first: raw([1, 2, 5]),
            second: raw([6, 6, 5]),
            chosen: DamageRollChoice::First,
            inspiration: None,
        };
        (request, roll)
    }

    #[test]
    fn only_weapon_dice_repeat_and_shared_inspiration_changes_one_actual_die() {
        let (request, mut roll) = pools();
        assert_eq!(
            crate::kernel::savage_result(&request, &roll).unwrap(),
            roll.first
        );
        roll.inspiration = Some(SavageInspiration {
            roll: DamageRollChoice::First,
            die_index: 2,
            replacement: DieResult { sides: 8, value: 8 },
        });
        for chosen in [DamageRollChoice::First, DamageRollChoice::Second] {
            roll.chosen = chosen;
            let result = crate::kernel::savage_result(&request, &roll).unwrap();
            assert_eq!(result.dice[2].value, 8);
            assert_eq!(
                result.dice[0].value,
                if chosen == DamageRollChoice::First {
                    1
                } else {
                    6
                }
            );
        }
        roll.inspiration.as_mut().unwrap().die_index = 0;
        roll.inspiration.as_mut().unwrap().replacement = DieResult { sides: 6, value: 4 };
        let result = crate::kernel::savage_result(&request, &roll).unwrap();
        assert_eq!(result, roll.second); // The unchosen first set's reroll is still retained.
    }

    #[test]
    fn extra_dice_cannot_be_changed_and_historical_wire_keeps_its_old_shape() {
        let (request, roll) = pools();
        for variation in 0..4 {
            let mut bad = roll.clone();
            match variation {
                0 => bad.second.dice[2].value = 8,
                1 => bad.weapon_dice = Some(0),
                2 => bad.weapon_dice = Some(4),
                _ => bad.second.dice[0].value = 7,
            }
            assert!(crate::kernel::savage_result(&request, &bad).is_err());
        }
        let mut legacy = roll;
        legacy.weapon_dice = None;
        let wire = serde_json::to_value(&legacy).unwrap();
        assert!(wire.get("weapon_dice").is_none());
        assert_eq!(
            serde_json::from_value::<SavageAttackerRoll>(wire).unwrap(),
            legacy
        );
        legacy.second.dice[2].value = 8;
        legacy.chosen = DamageRollChoice::Second;
        assert_eq!(
            crate::kernel::savage_result(&request, &legacy).unwrap(),
            legacy.second
        );
    }
}
