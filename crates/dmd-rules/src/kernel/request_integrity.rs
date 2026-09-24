//! Restore/import checks reconstruct outstanding requests from retained typed causes.
use super::validation::*;
use super::*;

pub(super) fn command(state: &CampaignState, meta: &CommandMeta) -> Result<(), RulesError> {
    if meta.campaign_id != state.campaign_id()
        || meta.expected_event_sequence > state.applied_event_sequence
        || matches!(meta.issuer, CommandIssuer::Import)
    {
        return Err(invalid("invalid rules command provenance"));
    }
    if let CommandIssuer::Player(id) = meta.issuer {
        if !state.players.contains_key(&id) {
            return Err(invalid("unknown rules issuer"));
        }
    }
    if let Some(actor) = meta.actor {
        match actor {
            AgentRef::Entity(id) if state.entities.contains_key(&id) => (),
            _ => return Err(invalid("rules command references an invalid actor")),
        }
    }
    Ok(())
}
pub(super) fn permission(
    state: &CampaignState,
    rules: &RulesState,
    p: &ActionPermission,
) -> Result<(), RulesError> {
    command(state, &p.issued_by)?;
    privileged(p.issued_by.issuer)?;
    entity(rules, p.actor)?;
    entity(rules, p.target)?;
    ruling_valid(&p.ruling, &rules.house_rules)?;
    if !rules
        .rulings
        .iter()
        .any(|r| r.command == p.issued_by && r.ruling == p.ruling)
    {
        return Err(invalid(
            "action permission lacks matching privileged ruling provenance",
        ));
    }
    Ok(())
}
fn request_equals(
    request: &RollRequest,
    dice: &[DieSpec],
    modifier: i32,
    mode: RollMode,
) -> Result<(), RulesError> {
    if request.dice != dice || request.modifier != modifier || request.mode != mode {
        return Err(invalid(
            "outstanding request differs from its typed mechanical cause",
        ));
    }
    Ok(())
}
fn same_permission(
    p: &ActionPermission,
    pending: &PendingRoll,
    target: EntityId,
    spell: bool,
) -> Result<(), RulesError> {
    if pending.request.roller != Some(p.actor)
        || p.target != target
        || p.spell != spell
        || p.issued_by.expected_event_sequence.checked_add(1)
            != Some(pending.issued_by.expected_event_sequence)
    {
        return Err(invalid(
            "outstanding request has stale/mismatched contextual permission",
        ));
    }
    Ok(())
}
pub(super) fn pending(
    state: &CampaignState,
    rules: &RulesState,
    p: &PendingRoll,
    pack: &RulesPack,
) -> Result<(), RulesError> {
    command(state, &p.issued_by)?;
    if state
        .applied_event_sequence
        .saturating_sub(p.issued_by.expected_event_sequence)
        > 1
    {
        return Err(invalid(
            "pending request was not suspended at the current rules command",
        ));
    }
    let actor = p.request.roller.ok_or_else(|| invalid("missing roller"))?;
    let e = entity(rules, actor)?;
    let d20 = [DieSpec {
        count: 1,
        sides: 20,
    }];
    match &p.purpose {
        PendingPurpose::Test {
            kind,
            dc,
            circumstances,
        } => {
            if !matches!(kind, TestKind::DeathSave) {
                privileged(p.issued_by.issuer)?;
            }
            if !(0..=100).contains(dc) || e.death.dead {
                return Err(invalid("invalid pending test"));
            }
            if matches!(kind, TestKind::DeathSave)
                && (*dc != 10 || e.hp != 0 || e.death.stable || !e.uses_death_saves)
            {
                return Err(invalid("death save is not due"));
            }
            request_equals(
                &p.request,
                &d20,
                check_modifier(e, kind),
                check_mode(rules, actor, kind, *circumstances),
            )?;
        }
        PendingPurpose::Attack {
            target,
            damage,
            damage_modifier,
            damage_type,
            automatic_critical,
            permission: auth,
        } => {
            permission(state, rules, auth)?;
            same_permission(auth, p, *target, auth.spell)?;
            ready(rules, actor)?;
            let (dice, modifier, kind, ranged, attack_modifier) = if auth.spell {
                let s = pack.spell(&auth.content_id)?;
                if !e.prepared_spells.contains(&s.id) {
                    return Err(invalid("pending spell no longer prepared"));
                }
                let casting = e
                    .spellcasting
                    .as_ref()
                    .ok_or_else(|| invalid("missing spellcasting"))?;
                let SpellEffect::AttackDamage {
                    dice,
                    damage_type,
                    cantrip_scaling,
                } = &s.effect
                else {
                    return Err(invalid("pending spell is not an attack"));
                };
                let mut damage = dice.clone();
                let multiplier = if *cantrip_scaling {
                    1 + u16::from(e.level >= 5)
                        + u16::from(e.level >= 11)
                        + u16::from(e.level >= 17)
                } else {
                    1
                };
                for d in &mut damage {
                    d.count = d
                        .count
                        .checked_mul(multiplier)
                        .ok_or_else(|| invalid("damage dice overflow"))?;
                }
                (
                    damage,
                    0,
                    *damage_type,
                    true,
                    ability_modifier(e.ability_scores[casting.ability.index()])
                        + proficiency_bonus(e.level)
                        - i32::from(e.exhaustion) * 2,
                )
            } else {
                let a = pack.attack(&auth.content_id)?;
                if !e.attacks.contains(&a.id) {
                    return Err(invalid("pending weapon unavailable"));
                }
                let modifier = engine::attack_modifier(e, a.ability);
                (
                    a.damage.clone(),
                    modifier,
                    a.damage_type,
                    a.ranged,
                    modifier
                        + if e.attack_proficiencies.contains(&a.id) {
                            proficiency_bonus(e.level)
                        } else {
                            0
                        }
                        - i32::from(e.exhaustion) * 2,
                )
            };
            let (roll_mode, critical) =
                engine::attack_context(rules, actor, *target, auth, ranged)?;
            if *damage != dice
                || *damage_modifier != modifier
                || *damage_type != kind
                || *automatic_critical != critical
            {
                return Err(invalid("pending attack damage disagrees with content"));
            }
            request_equals(&p.request, &d20, attack_modifier, roll_mode)?;
        }
        PendingPurpose::Damage {
            target,
            damage_type,
            critical,
            attack_roll_id,
        } => {
            let previous = rules
                .rolls
                .last()
                .ok_or_else(|| invalid("damage lacks a preceding attack"))?;
            if previous.request.id != *attack_roll_id
                || previous.accepted_by != p.issued_by
                || previous.request.roller != Some(actor)
            {
                return Err(invalid(
                    "damage is not caused by the immediately preceding accepted attack",
                ));
            }
            let PendingPurpose::Attack {
                target: original_target,
                damage,
                damage_modifier,
                damage_type: original_type,
                automatic_critical,
                ..
            } = &previous.purpose
            else {
                return Err(invalid("damage origin is not an attack"));
            };
            let face = previous
                .resolved
                .kept_dice
                .first()
                .ok_or_else(|| invalid("attack has no die"))?
                .value;
            if *target != *original_target
                || *damage_type != *original_type
                || face == 1
                || (face != 20 && previous.resolved.total < armor_class(entity(rules, *target)?))
                || *critical != (face == 20 || *automatic_critical)
            {
                return Err(invalid("damage contradicts its attack outcome"));
            }
            let mut dice = damage.clone();
            if *critical {
                for d in &mut dice {
                    d.count = d
                        .count
                        .checked_mul(2)
                        .ok_or_else(|| invalid("critical dice overflow"))?;
                }
            }
            request_equals(&p.request, &dice, *damage_modifier, RollMode::Normal)?;
        }
        PendingPurpose::Healing {
            target,
            spell_id,
            slot_level,
            permission: auth,
        } => {
            permission(state, rules, auth)?;
            same_permission(auth, p, *target, true)?;
            if auth.content_id != *spell_id {
                return Err(invalid("healing permission differs from spell"));
            }
            let s = pack.spell(spell_id)?;
            let SpellEffect::Healing {
                dice,
                extra_dice_per_slot,
            } = &s.effect
            else {
                return Err(invalid("healing origin is not healing magic"));
            };
            if *slot_level < s.level
                || *slot_level > 9
                || !e.prepared_spells.contains(spell_id)
                || entity(rules, *target)?.death.dead
            {
                return Err(invalid("invalid pending healing spell"));
            }
            let casting = e
                .spellcasting
                .as_ref()
                .ok_or_else(|| invalid("missing spellcasting"))?;
            let mut dice = dice.clone();
            for _ in s.level..*slot_level {
                dice.extend(extra_dice_per_slot.clone());
            }
            request_equals(
                &p.request,
                &dice,
                ability_modifier(e.ability_scores[casting.ability.index()]),
                RollMode::Normal,
            )?;
        }
        PendingPurpose::Concentration { dc, damage_taken } => {
            if e.concentration.is_none()
                || *damage_taken == 0
                || *dc != ((*damage_taken / 2).clamp(10, 30) as i32)
            {
                return Err(invalid("concentration save has no current cause"));
            }
            let kind = TestKind::Save {
                ability: Ability::Constitution,
            };
            request_equals(
                &p.request,
                &d20,
                check_modifier(e, &kind),
                check_mode(rules, actor, &kind, Circumstances::default()),
            )?;
        }
        PendingPurpose::RestHitDie => {
            if !rules.completed_short_rests.contains(&actor) || e.hp == 0 || e.death.dead {
                return Err(invalid("hit die healing outside completed rest"));
            }
            request_equals(
                &p.request,
                &[DieSpec {
                    count: 1,
                    sides: e.hit_dice.sides,
                }],
                ability_modifier(e.ability_scores[2]),
                RollMode::Normal,
            )?;
        }
    }
    Ok(())
}
