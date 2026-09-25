use super::validation::*;
use super::*;
use crate::ResolveRoll;
use std::collections::HashSet;

pub fn resolve(
    state: &CampaignState,
    meta: &CommandMeta,
    action: &RulesAction,
    pack: &RulesPack,
) -> Result<RulesTransition, RulesError> {
    if meta.campaign_id != state.campaign_id() {
        return Err(RulesError::Unauthorized);
    }
    if meta.expected_event_sequence != state.applied_event_sequence {
        return Err(RulesError::Stale);
    }
    validate_state(state, pack)?;
    if state
        .rules
        .as_ref()
        .is_some_and(|rules| rules.tactical_inventory.is_some())
        && !matches!(
            action,
            RulesAction::CreateCharacter { .. }
                | RulesAction::RequestTest { .. }
                | RulesAction::SecondWind { .. }
                | RulesAction::SubmitRoll { .. }
                | RulesAction::SubmitRollWithInspiration { .. }
                | RulesAction::CancelRoll { .. }
        )
    {
        return Err(prerequisite(
            "physical equipment requires the tactical action path",
        ));
    }
    if state.encounter.as_ref().is_some_and(|e| e.flow.is_some())
        || state
            .rules
            .as_ref()
            .is_some_and(|r| r.tactical_effects.is_some() || r.tactical_recovery.is_some())
    {
        return Err(prerequisite(
            "active tactical state requires the tactical command path",
        ));
    }
    if state.rules.as_ref().is_some_and(|r| r.pending.is_some())
        && !matches!(
            action,
            RulesAction::SubmitRoll { .. }
                | RulesAction::SubmitRollWithInspiration { .. }
                | RulesAction::SubmitSavageAttacker { .. }
                | RulesAction::CancelRoll { .. }
        )
    {
        return Err(RulesError::Pending);
    }
    if state.rules.as_ref().is_some_and(|r| {
        r.entities.values().any(|e| {
            e.character_features
                .as_ref()
                .is_some_and(|f| f.inspiration_transfer_pending)
        })
    }) && !matches!(action, RulesAction::ResolveInspirationTransfer { .. })
    {
        return Err(prerequisite(
            "an Inspiration transfer choice awaits its controller",
        ));
    }
    let mut next = state.clone();
    let outcome = if let RulesAction::CreateCharacter { entity_id, input } = action {
        authorize(state, meta, *entity_id)?;
        if !state
            .characters
            .values()
            .any(|c| c.entity_id == *entity_id && c.status == CharacterStatus::Active)
        {
            return Err(prerequisite(
                "creation requires an existing active character identity",
            ));
        }
        let built = crate::build_character(input, *entity_id, pack)?;
        if let Some(rules) = &mut next.rules {
            if rules.entities.contains_key(entity_id) {
                return Err(prerequisite("character mechanics already exist"));
            }
            rules.entities.insert(*entity_id, built.mechanics);
            rules.permission = None;
        } else {
            next.rules = Some(RulesState {
                tactical_recovery: None,
                pack_id: pack.id.clone(),
                pack_version: pack.version.clone(),
                entities: std::collections::HashMap::from([(*entity_id, built.mechanics)]),
                house_rules: state
                    .table
                    .as_ref()
                    .map_or_else(HouseRules::default, |table| {
                        table.contract.house_rules.clone()
                    }),
                effects: vec![],
                tactical_effects: None,
                tactical_inventory: None,
                tactical_creatures: None,
                pending: None,
                rolls: vec![],
                cancelled_roll_ids: vec![],
                rulings: vec![],
                timing: None,
                rests: vec![],
                completed_short_rests: vec![],
                permission: None,
            });
        }
        RulesOutcome::Changed
    } else if let RulesAction::Initialize {
        entities,
        house_rules,
        ruling,
    } = action
    {
        privileged(meta.issuer)?;
        if state.rules.is_some() {
            return Err(prerequisite("rules already initialized"));
        }
        ruling_valid(ruling, house_rules)?;
        let map = entities
            .iter()
            .map(|e| (e.entity_id, e.clone()))
            .collect::<std::collections::HashMap<_, _>>();
        if map.len() != entities.len() || map.is_empty() {
            return Err(invalid("duplicate/empty mechanical initialization"));
        }
        next.rules = Some(RulesState {
            tactical_recovery: None,
            pack_id: pack.id.clone(),
            pack_version: pack.version.clone(),
            entities: map,
            house_rules: house_rules.clone(),
            effects: vec![],
            tactical_effects: None,
            tactical_inventory: None,
            tactical_creatures: None,
            pending: None,
            rolls: vec![],
            cancelled_roll_ids: vec![],
            rulings: vec![RulingRecord {
                command: meta.clone(),
                ruling: ruling.clone(),
            }],
            timing: None,
            rests: vec![],
            completed_short_rests: vec![],
            permission: None,
        });
        RulesOutcome::Changed
    } else {
        let mut rules = next.rules.take().ok_or(RulesError::Uninitialized)?;
        let permission = rules.permission.take();
        let outcome = apply(&mut next, &mut rules, meta, action, pack, permission)?;
        next.rules = Some(rules);
        outcome
    };
    sync_deaths(&mut next);
    validate_state(&next, pack)?;
    Ok(RulesTransition {
        next_state: next,
        event: RulesEvent {
            meta: meta.clone(),
            action: action.clone(),
            outcome: outcome.clone(),
        },
        outcome,
    })
}

pub fn replay(
    state: &CampaignState,
    event: &RulesEvent,
    pack: &RulesPack,
) -> Result<RulesTransition, RulesError> {
    let transition = resolve(state, &event.meta, &event.action, pack)?;
    if transition.event != *event {
        return Err(RulesError::ReplayMismatch);
    }
    Ok(transition)
}

pub fn query(
    state: &CampaignState,
    issuer: CommandIssuer,
    query: &RulesQuery,
    pack: &RulesPack,
) -> Result<RulesAnswer, RulesError> {
    validate_state(state, pack)?;
    let rules = state.rules.as_ref().ok_or(RulesError::Uninitialized)?;
    match query {
        RulesQuery::PendingRoll => Ok(RulesAnswer::PendingRoll(
            rules
                .pending
                .as_ref()
                .filter(|p| match issuer {
                    CommandIssuer::System | CommandIssuer::Admin => true,
                    CommandIssuer::Player(_) => {
                        p.request.visibility != RollVisibility::Secret
                            && p.request
                                .roller
                                .is_some_and(|actor| owns(state, issuer, actor))
                    }
                    CommandIssuer::Import => false,
                })
                .map(|p| p.request.clone()),
        )),
        RulesQuery::Character { actor } => {
            if !owns(state, issuer, *actor) {
                return Err(RulesError::Unauthorized);
            }
            let e = entity(rules, *actor)?;
            Ok(RulesAnswer::Character {
                ability_modifiers: e.ability_scores.map(ability_modifier),
                proficiency_bonus: proficiency_bonus(e.level),
                armor_class: crate::tactical_defenses::effective_armor_class(state, *actor)?,
                hp: e.hp,
                max_hp: e.max_hp,
                temporary_hp: e.temporary_hp,
                spell_save_dc: e.spellcasting.as_ref().map(|s| {
                    8 + proficiency_bonus(e.level)
                        + ability_modifier(e.ability_scores[s.ability.index()])
                }),
            })
        }
        RulesQuery::PassivePerception {
            actor,
            circumstances,
        } => {
            if !owns(state, issuer, *actor) {
                return Err(RulesError::Unauthorized);
            }
            let e = entity(rules, *actor)?;
            let kind = TestKind::Check {
                ability: Ability::Wisdom,
                skill: Some(Skill::Perception),
            };
            let adjustment = match check_mode(rules, *actor, &kind, *circumstances) {
                RollMode::Normal => 0,
                RollMode::Advantage => 5,
                RollMode::Disadvantage => -5,
            };
            Ok(RulesAnswer::PassivePerception(
                10 + check_modifier(e, &kind) + adjustment,
            ))
        }
        RulesQuery::SupportedContent => Ok(RulesAnswer::SupportedContent {
            attacks: pack.attacks.iter().map(|a| a.id.clone()).collect(),
            spells: pack
                .spells
                .iter()
                .map(|s| SupportedSpell {
                    id: s.id.clone(),
                    support: match s.effect {
                        SpellEffect::Healing { .. } => SpellSupport::HealingWithUpcasting,
                        SpellEffect::AttackDamage { .. } => SpellSupport::CreatureAttackDamage,
                        SpellEffect::ConcentrationMarker { .. } => {
                            SpellSupport::ConcentrationDurationOnly
                        }
                    },
                })
                .collect(),
        }),
    }
}

fn apply(
    state: &mut CampaignState,
    rules: &mut RulesState,
    meta: &CommandMeta,
    action: &RulesAction,
    pack: &RulesPack,
    permission: Option<ActionPermission>,
) -> Result<RulesOutcome, RulesError> {
    let continues_rest_spending = matches!(
        action,
        RulesAction::ApplyDamage { .. }
            | RulesAction::SpendHitDie { .. }
            | RulesAction::FinishRest { .. }
            | RulesAction::InterruptRest { .. }
    ) || (matches!(
        action,
        RulesAction::SubmitRoll { .. } | RulesAction::SubmitRollWithInspiration { .. }
    ) && rules
        .pending
        .as_ref()
        .is_some_and(|p| matches!(p.purpose, PendingPurpose::RestHitDie)));
    if !continues_rest_spending {
        rules.completed_short_rests.clear();
    }
    match action {
        RulesAction::CreateCharacter { .. } => {
            Err(prerequisite("creation is handled at the entity boundary"))
        }
        RulesAction::Initialize { .. } => Err(prerequisite("already initialized")),
        RulesAction::RequestTest {
            actor,
            kind,
            dc,
            visibility,
            circumstances,
            ruling,
            request_id,
        } => {
            adjudicate(rules, meta, ruling)?;
            entity(rules, *actor)?;
            authorize(state, meta, *actor)?;
            if rules
                .tactical_creatures
                .as_ref()
                .is_some_and(|creatures| creatures.profile(*actor).is_some())
            {
                return Err(prerequisite(
                    "source creature tests require their source action path",
                ));
            }
            if !(0..=100).contains(dc) {
                return Err(invalid("test DC outside supported range"));
            }
            if matches!(kind, TestKind::Check { .. }) {
                ready(rules, *actor)?;
            }
            if matches!(kind, TestKind::Initiative) {
                interrupt_rest(rules, *actor, state.clock.now);
                rules.completed_short_rests.retain(|id| id != actor);
            }
            let e = entity(rules, *actor)?;
            if e.death.dead {
                return Err(prerequisite("dead creatures cannot roll"));
            }
            if matches!(kind, TestKind::DeathSave)
                && (e.hp != 0 || e.death.stable || !e.uses_death_saves)
            {
                return Err(prerequisite("death saving throw is not due"));
            }
            if matches!(
                kind,
                TestKind::Save {
                    ability: Ability::Strength | Ability::Dexterity
                }
            ) && [
                Condition::Paralyzed,
                Condition::Petrified,
                Condition::Stunned,
                Condition::Unconscious,
            ]
            .iter()
            .any(|c| conditions(rules, *actor).contains(c))
            {
                return Ok(RulesOutcome::AutomaticTest { success: false });
            }
            let request = RollRequest {
                id: *request_id,
                roller: Some(*actor),
                dice: vec![DieSpec {
                    count: 1,
                    sides: 20,
                }],
                modifier: check_modifier(e, kind),
                mode: check_mode(rules, *actor, kind, *circumstances),
                visibility: *visibility,
                reason: format!("{kind:?}"),
            };
            let actual_dc = if matches!(kind, TestKind::DeathSave) {
                10
            } else {
                *dc
            };
            set_pending(
                rules,
                meta,
                request,
                PendingPurpose::Test {
                    kind: kind.clone(),
                    dc: actual_dc,
                    circumstances: *circumstances,
                },
                ruling.clone(),
            )
        }
        RulesAction::AuthorizeAttack {
            actor,
            target,
            attack_id,
            circumstances,
            within_five_feet,
            ruling,
        } => {
            adjudicate(rules, meta, ruling)?;
            ready(rules, *actor)?;
            entity(rules, *target)?;
            pack.attack(attack_id)?;
            rules.permission = Some(ActionPermission {
                issued_by: meta.clone(),
                actor: *actor,
                target: *target,
                content_id: attack_id.clone(),
                spell: false,
                circumstances: *circumstances,
                within_five_feet: *within_five_feet,
                ruling: ruling.clone(),
            });
            Ok(RulesOutcome::Changed)
        }
        RulesAction::AuthorizeSpell {
            actor,
            target,
            spell_id,
            circumstances,
            within_five_feet,
            ruling,
        } => {
            adjudicate(rules, meta, ruling)?;
            ready(rules, *actor)?;
            entity(rules, *target)?;
            pack.spell(spell_id)?;
            rules.permission = Some(ActionPermission {
                issued_by: meta.clone(),
                actor: *actor,
                target: *target,
                content_id: spell_id.clone(),
                spell: true,
                circumstances: *circumstances,
                within_five_feet: *within_five_feet,
                ruling: ruling.clone(),
            });
            Ok(RulesOutcome::Changed)
        }
        RulesAction::Attack {
            actor,
            target,
            attack_id,
            request_id,
        } => {
            authorize(state, meta, *actor)?;
            ready(rules, *actor)?;
            let p = permission
                .ok_or_else(|| prerequisite("attack needs authoritative contextual permission"))?;
            permission_matches(&p, meta, *actor, *target, attack_id, false)?;
            if !entity(rules, *actor)?.attacks.contains(attack_id) {
                return Err(prerequisite("weapon is not in the actor's granted loadout"));
            }
            let attack = pack.attack(attack_id)?;
            let e = entity(rules, *actor)?;
            let modifier = attack_modifier(e, attack.ability);
            let total_modifier = modifier
                + if e.attack_proficiencies.contains(attack_id) {
                    proficiency_bonus(e.level)
                } else {
                    0
                }
                - i32::from(e.exhaustion) * 2;
            let (roll_mode, critical) = attack_context(rules, *actor, *target, &p, attack.ranged)?;
            let total_modifier = total_modifier + archery_bonus(e, attack.ranged);
            spend_action(rules, *actor)?;
            interrupt_rest(rules, *actor, state.clock.now);
            rules.completed_short_rests.retain(|id| id != actor);
            set_pending(
                rules,
                meta,
                RollRequest {
                    id: *request_id,
                    roller: Some(*actor),
                    dice: vec![DieSpec {
                        count: 1,
                        sides: 20,
                    }],
                    modifier: total_modifier,
                    mode: roll_mode,
                    visibility: RollVisibility::Public,
                    reason: format!("attack:{attack_id}"),
                },
                PendingPurpose::Attack {
                    target: *target,
                    damage: attack.damage.clone(),
                    damage_modifier: modifier,
                    damage_type: attack.damage_type,
                    automatic_critical: critical,
                    permission: p.clone(),
                },
                p.ruling,
            )
        }
        RulesAction::CastSpell {
            actor,
            target,
            spell_id,
            slot_level,
            request_id,
            effect_id,
        } => cast_spell(
            state,
            rules,
            meta,
            pack,
            (
                *actor,
                *target,
                spell_id,
                *slot_level,
                *request_id,
                *effect_id,
            ),
            permission,
        ),
        RulesAction::SubmitRoll { result } => submit(state, rules, meta, result),
        RulesAction::SecondWind { actor, request_id } => {
            authorize(state, meta, *actor)?;
            ready(rules, *actor)?;
            if let Some(t) = &mut rules.timing {
                if t.order[t.index].actor != *actor || t.bonus_action_spent {
                    return Err(prerequisite("Second Wind bonus action unavailable"));
                }
                t.bonus_action_spent = true;
            }
            let features = entity_mut(rules, *actor)?
                .character_features
                .as_mut()
                .ok_or_else(|| prerequisite("Second Wind is not granted"))?;
            if features.second_wind_remaining == 0 {
                return Err(prerequisite("Second Wind exhausted"));
            }
            features.second_wind_remaining -= 1;
            let modifier = i32::from(features.fighter_level);
            set_pending(
                rules,
                meta,
                RollRequest {
                    id: *request_id,
                    roller: Some(*actor),
                    dice: vec![DieSpec {
                        count: 1,
                        sides: 10,
                    }],
                    modifier,
                    mode: RollMode::Normal,
                    visibility: RollVisibility::Public,
                    reason: "second-wind".into(),
                },
                PendingPurpose::SecondWind,
                srd(48, "Fighter Second Wind"),
            )
        }
        RulesAction::ResolveInspirationTransfer { actor, recipient } => {
            authorize(state, meta, *actor)?;
            if !entity(rules, *actor)?
                .character_features
                .as_ref()
                .is_some_and(|f| f.inspiration_transfer_pending)
            {
                return Err(prerequisite("no Inspiration transfer is pending"));
            }
            if let Some(recipient) = recipient {
                if recipient == actor
                    || !state.characters.values().any(|c| {
                        c.entity_id == *recipient
                            && c.status == CharacterStatus::Active
                            && c.controlling_player_id.is_some()
                    })
                    || entity(rules, *recipient)?.heroic_inspiration
                    || entity(rules, *recipient)?.death.dead
                {
                    return Err(prerequisite(
                        "recipient must be another eligible group PC lacking Inspiration",
                    ));
                }
                entity_mut(rules, *recipient)?.heroic_inspiration = true;
            }
            entity_mut(rules, *actor)?
                .character_features
                .as_mut()
                .unwrap()
                .inspiration_transfer_pending = false;
            Ok(RulesOutcome::Changed)
        }
        RulesAction::SubmitSavageAttacker { roll } => {
            if roll.weapon_dice.is_some() {
                return Err(invalid("tactical weapon dice require the tactical path"));
            }
            let pending = rules.pending.as_ref().ok_or(RulesError::NoPending)?;
            let actor = pending
                .request
                .roller
                .ok_or_else(|| invalid("missing roller"))?;
            let PendingPurpose::Damage { attack_roll_id, .. } = pending.purpose else {
                return Err(prerequisite("Savage Attacker requires weapon damage"));
            };
            let attack = rules
                .rolls
                .iter()
                .find(|r| r.request.id == attack_roll_id)
                .ok_or_else(|| invalid("missing damage origin"))?;
            if !matches!(&attack.purpose, PendingPurpose::Attack { permission, .. } if !permission.spell)
            {
                return Err(prerequisite(
                    "Savage Attacker applies only to weapon damage dice",
                ));
            }
            let turn = rules
                .timing
                .as_ref()
                .ok_or_else(|| prerequisite("Savage Attacker needs recorded turn timing"))?
                .turn_number;
            let features = entity(rules, actor)?
                .character_features
                .as_ref()
                .ok_or_else(|| prerequisite("Savage Attacker is not granted"))?;
            if !features.savage_attacker || features.savage_attacker_turn == Some(turn) {
                return Err(prerequisite("Savage Attacker unavailable this turn"));
            }
            let result = savage_result(&pending.request, roll)?;
            if roll.inspiration.is_some() {
                if !entity(rules, actor)?.heroic_inspiration {
                    return Err(prerequisite("Heroic Inspiration unavailable"));
                }
                entity_mut(rules, actor)?.heroic_inspiration = false;
            }
            entity_mut(rules, actor)?
                .character_features
                .as_mut()
                .unwrap()
                .savage_attacker_turn = Some(turn);
            let outcome = submit(state, rules, meta, &result)?;
            rules
                .rolls
                .last_mut()
                .ok_or_else(|| invalid("missing damage record"))?
                .savage_attacker = Some(roll.clone());
            Ok(outcome)
        }
        RulesAction::SubmitRollWithInspiration {
            result,
            die_index,
            replacement,
        } => {
            let pending = rules.pending.as_ref().ok_or(RulesError::NoPending)?;
            pending.request.resolve(result)?;
            let actor = pending
                .request
                .roller
                .ok_or_else(|| invalid("missing roller"))?;
            let original = result
                .dice
                .get(*die_index)
                .ok_or_else(|| invalid("reroll die index out of range"))?;
            if replacement.sides != original.sides
                || replacement.value == 0
                || replacement.value > replacement.sides
            {
                return Err(invalid("invalid replacement die"));
            }
            if !entity(rules, actor)?.heroic_inspiration {
                return Err(prerequisite("Heroic Inspiration unavailable"));
            }
            entity_mut(rules, actor)?.heroic_inspiration = false;
            let mut replacement_result = result.clone();
            replacement_result.dice[*die_index] = *replacement;
            let outcome = submit(state, rules, meta, &replacement_result)?;
            rules
                .rolls
                .last_mut()
                .ok_or_else(|| invalid("missing recorded reroll"))?
                .original_result = Some(result.clone());
            Ok(outcome)
        }
        RulesAction::GrantInspiration { actor, ruling } => {
            adjudicate(rules, meta, ruling)?;
            let e = entity_mut(rules, *actor)?;
            if e.heroic_inspiration {
                return Err(prerequisite(
                    "Heroic Inspiration does not stack; designate another eligible recipient",
                ));
            }
            e.heroic_inspiration = true;
            Ok(RulesOutcome::Changed)
        }
        RulesAction::CancelRoll { ruling } => {
            adjudicate(rules, meta, ruling)?;
            let pending = rules.pending.take().ok_or(RulesError::NoPending)?;
            rules.cancelled_roll_ids.push(pending.request.id);
            if matches!(pending.purpose, PendingPurpose::Concentration { .. }) {
                end_concentration(
                    rules,
                    pending
                        .request
                        .roller
                        .ok_or_else(|| invalid("missing roller"))?,
                );
            }
            Ok(RulesOutcome::Changed)
        }
        RulesAction::ApplyDamage {
            target,
            amount,
            damage_type,
            critical,
            ruling,
        } => {
            adjudicate(rules, meta, ruling)?;
            let applied = damage(
                rules,
                *target,
                *amount,
                *damage_type,
                *critical,
                state.clock.now,
            )?;
            let followup = concentration_request(rules, *target, applied, meta, ruling.clone())?;
            Ok(RulesOutcome::Damage { applied, followup })
        }
        RulesAction::Heal {
            target,
            amount,
            ruling,
        } => {
            adjudicate(rules, meta, ruling)?;
            Ok(RulesOutcome::Healed {
                regained: heal(rules, *target, *amount)?,
            })
        }
        RulesAction::GrantTemporaryHp {
            target,
            amount,
            ruling,
        } => {
            adjudicate(rules, meta, ruling)?;
            if *amount > 1_000_000 {
                return Err(invalid("temporary hit points overflow"));
            }
            entity_mut(rules, *target)?.temporary_hp = *amount;
            Ok(RulesOutcome::Changed)
        }
        RulesAction::ApplyEffect { effect, ruling } => {
            adjudicate(rules, meta, ruling)?;
            apply_effect(rules, effect.clone())?;
            Ok(RulesOutcome::Changed)
        }
        RulesAction::RemoveEffect { effect_id, ruling } => {
            adjudicate(rules, meta, ruling)?;
            if !rules.effects.iter().any(|e| e.id == *effect_id) {
                return Err(invalid("unknown effect"));
            }
            remove_effects(rules, |e| e.id == *effect_id);
            Ok(RulesOutcome::Changed)
        }
        RulesAction::SetExhaustion {
            target,
            levels,
            ruling,
        } => {
            adjudicate(rules, meta, ruling)?;
            if *levels > 6 {
                return Err(invalid("invalid exhaustion level"));
            }
            let e = entity_mut(rules, *target)?;
            e.exhaustion = *levels;
            if *levels == 6 {
                kill(e);
                end_concentration(rules, *target);
            }
            Ok(RulesOutcome::Changed)
        }
        RulesAction::SetProne {
            target,
            prone,
            ruling,
        } => {
            adjudicate(rules, meta, ruling)?;
            if !prone && conditions(rules, *target).contains(&Condition::Unconscious) {
                return Err(prerequisite("unconscious creature cannot stand"));
            }
            entity_mut(rules, *target)?.prone = *prone;
            Ok(RulesOutcome::Changed)
        }
        RulesAction::SpendResource {
            actor,
            resource_id,
            amount,
        } => {
            authorize(state, meta, *actor)?;
            ready(rules, *actor)?;
            if *amount == 0 {
                return Err(invalid("zero resource expenditure"));
            }
            let pool = entity_mut(rules, *actor)?
                .resources
                .get_mut(resource_id)
                .ok_or_else(|| invalid("unknown resource"))?;
            pool.remaining = pool
                .remaining
                .checked_sub(*amount)
                .ok_or_else(|| prerequisite("resource exhausted"))?;
            Ok(RulesOutcome::Changed)
        }
        RulesAction::RecoverResource {
            actor,
            resource_id,
            amount,
            ruling,
        } => {
            adjudicate(rules, meta, ruling)?;
            if *amount == 0 {
                return Err(invalid("zero resource recovery"));
            }
            let pool = entity_mut(rules, *actor)?
                .resources
                .get_mut(resource_id)
                .ok_or_else(|| invalid("unknown resource"))?;
            pool.remaining = pool.remaining.saturating_add(*amount).min(pool.maximum);
            Ok(RulesOutcome::Changed)
        }
        RulesAction::StartRest {
            actor,
            kind,
            ruling,
        } => {
            adjudicate(rules, meta, ruling)?;
            ready(rules, *actor)?;
            if rules.timing.is_some() || rules.rests.iter().any(|r| r.actor == *actor) {
                return Err(prerequisite("already resting or in combat"));
            }
            if *kind == RestKind::Long
                && entity(rules, *actor)?
                    .last_long_rest_finished
                    .is_some_and(|last| state.clock.now.0.saturating_sub(last.0) < 16 * 3600)
            {
                return Err(prerequisite("must wait 16 hours after a long rest"));
            }
            rules.completed_short_rests.retain(|id| id != actor);
            rules.rests.push(RestProgress {
                actor: *actor,
                kind: *kind,
                started_at: state.clock.now,
            });
            Ok(RulesOutcome::Changed)
        }
        RulesAction::InterruptRest { actor, ruling } => {
            adjudicate(rules, meta, ruling)?;
            if !rules.rests.iter().any(|r| r.actor == *actor) {
                return Err(prerequisite("not resting"));
            }
            interrupt_rest(rules, *actor, state.clock.now);
            Ok(RulesOutcome::Changed)
        }
        RulesAction::FinishRest {
            actor,
            slept_seconds,
            ruling,
        } => finish_rest(state, rules, meta, *actor, *slept_seconds, ruling),
        RulesAction::SpendHitDie { actor, request_id } => {
            authorize(state, meta, *actor)?;
            ready(rules, *actor)?;
            if !rules.completed_short_rests.contains(actor) {
                return Err(prerequisite("hit dice require a completed short rest"));
            }
            let e = entity_mut(rules, *actor)?;
            e.hit_dice.remaining = e
                .hit_dice
                .remaining
                .checked_sub(1)
                .ok_or_else(|| prerequisite("no hit dice remaining"))?;
            let request = RollRequest {
                id: *request_id,
                roller: Some(*actor),
                dice: vec![DieSpec {
                    count: 1,
                    sides: e.hit_dice.sides,
                }],
                modifier: ability_modifier(e.ability_scores[2]),
                mode: RollMode::Normal,
                visibility: RollVisibility::Public,
                reason: "short-rest-hit-die".into(),
            };
            set_pending(
                rules,
                meta,
                request,
                PendingPurpose::RestHitDie,
                srd(187, "Spend one hit die after a short rest"),
            )
        }
        RulesAction::AdvanceTime { seconds, ruling } => {
            adjudicate(rules, meta, ruling)?;
            if *seconds == 0 || rules.timing.is_some() {
                return Err(prerequisite(
                    "time advancement requires positive noncombat duration",
                ));
            }
            state.clock.now = WorldInstant(
                state
                    .clock
                    .now
                    .0
                    .checked_add(i64::from(*seconds))
                    .ok_or_else(|| invalid("world time overflow"))?,
            );
            remove_effects(
                rules,
                |e| matches!(e.expires,Expiry::AtTime(t) if t<=state.clock.now),
            );
            Ok(RulesOutcome::Changed)
        }
        RulesAction::StartCombat {
            participants,
            ruling,
        } => start_combat(state, rules, meta, participants, ruling),
        RulesAction::EndTurn { actor } => {
            authorize(state, meta, *actor)?;
            let t = rules
                .timing
                .as_ref()
                .ok_or_else(|| prerequisite("no combat timing"))?;
            if t.order[t.index].actor != *actor {
                return Err(prerequisite("not this actor's turn"));
            }
            let end_number = t.turn_number;
            remove_effects(
                rules,
                |e| matches!(e.expires,Expiry::AtTurn{actor:a,boundary:TurnBoundary::End,turn_number:n} if a==*actor&&n<=end_number),
            );
            let t = rules
                .timing
                .as_mut()
                .ok_or_else(|| prerequisite("no combat timing"))?;
            t.index = (t.index + 1) % t.order.len();
            if t.index == 0 {
                t.round = t
                    .round
                    .checked_add(1)
                    .ok_or_else(|| invalid("round overflow"))?;
                state.clock.now.0 = state
                    .clock
                    .now
                    .0
                    .checked_add(6)
                    .ok_or_else(|| invalid("clock overflow"))?;
            }
            t.turn_number = t
                .turn_number
                .checked_add(1)
                .ok_or_else(|| invalid("turn overflow"))?;
            t.action_spent = false;
            t.bonus_action_spent = false;
            t.slot_spent_this_turn = false;
            let next = t.order[t.index].actor;
            let number = t.turn_number;
            t.reactions_spent.retain(|a| *a != next);
            remove_effects(rules, |e| {
                matches!(e.expires,Expiry::AtTurn{actor:a,boundary:TurnBoundary::Start,turn_number:n} if a==next&&n<=number)
                    || matches!(e.expires,Expiry::AtTime(t) if t<=state.clock.now)
            });
            turn_start(rules, next, meta)
        }
        RulesAction::EndCombat { ruling } => {
            adjudicate(rules, meta, ruling)?;
            if rules.timing.take().is_none() {
                return Err(prerequisite("no combat timing"));
            }
            remove_effects(rules, |e| matches!(e.expires, Expiry::AtTurn { .. }));
            for e in rules.entities.values_mut() {
                if let Some(f) = &mut e.character_features {
                    f.savage_attacker_turn = None;
                }
            }
            Ok(RulesOutcome::Changed)
        }
        RulesAction::UseReaction {
            actor,
            trigger,
            ruling,
        } => {
            adjudicate(rules, meta, ruling)?;
            ready(rules, *actor)?;
            if trigger.trim().is_empty() {
                return Err(invalid("reaction requires a supported trigger"));
            }
            let t = rules
                .timing
                .as_mut()
                .ok_or_else(|| prerequisite("reaction timing requires initiative"))?;
            if !t.order.iter().any(|e| e.actor == *actor) || t.reactions_spent.contains(actor) {
                return Err(prerequisite("reaction unavailable"));
            }
            t.reactions_spent.push(*actor);
            Ok(RulesOutcome::Changed)
        }
        RulesAction::UseBonusAction {
            actor,
            feature_id,
            ruling,
        } => {
            adjudicate(rules, meta, ruling)?;
            ready(rules, *actor)?;
            if !definitions::valid_id(feature_id) {
                return Err(invalid(
                    "bonus action requires a supported feature identifier",
                ));
            }
            let t = rules
                .timing
                .as_mut()
                .ok_or_else(|| prerequisite("bonus action timing requires initiative"))?;
            if t.order[t.index].actor != *actor || t.bonus_action_spent {
                return Err(prerequisite("bonus action unavailable"));
            }
            t.bonus_action_spent = true;
            Ok(RulesOutcome::Changed)
        }
        RulesAction::EndConcentration { actor } => {
            authorize(state, meta, *actor)?;
            if entity(rules, *actor)?.concentration.is_none() {
                return Err(prerequisite("not concentrating"));
            }
            end_concentration(rules, *actor);
            Ok(RulesOutcome::Changed)
        }
    }
}

fn adjudicate(
    rules: &mut RulesState,
    meta: &CommandMeta,
    ruling: &Ruling,
) -> Result<(), RulesError> {
    privileged(meta.issuer)?;
    ruling_valid(ruling, &rules.house_rules)?;
    rules.rulings.push(RulingRecord {
        command: meta.clone(),
        ruling: ruling.clone(),
    });
    Ok(())
}
fn srd(page: u16, reason: &str) -> Ruling {
    Ruling {
        basis: RulingBasis::Srd { page },
        reason: reason.into(),
    }
}
fn permission_matches(
    p: &ActionPermission,
    meta: &CommandMeta,
    actor: EntityId,
    target: EntityId,
    id: &str,
    spell: bool,
) -> Result<(), RulesError> {
    if p.issued_by.expected_event_sequence.checked_add(1) != Some(meta.expected_event_sequence) {
        return Err(prerequisite("contextual permission is stale"));
    }
    if p.actor != actor || p.target != target || p.content_id != id || p.spell != spell {
        return Err(prerequisite(
            "context permission does not match selected action",
        ));
    }
    Ok(())
}
fn set_pending(
    rules: &mut RulesState,
    meta: &CommandMeta,
    request: RollRequest,
    purpose: PendingPurpose,
    ruling: Ruling,
) -> Result<RulesOutcome, RulesError> {
    request.validate()?;
    if rules.pending.is_some() {
        return Err(RulesError::Pending);
    }
    if rules.rolls.iter().any(|r| r.request.id == request.id)
        || rules.cancelled_roll_ids.contains(&request.id)
    {
        return Err(invalid("roll request identifier already used"));
    }
    rules.pending = Some(PendingRoll {
        issued_by: meta.clone(),
        request: request.clone(),
        purpose,
        ruling,
    });
    Ok(RulesOutcome::RollRequested(request))
}
fn spend_action(rules: &mut RulesState, actor: EntityId) -> Result<(), RulesError> {
    if let Some(t) = &mut rules.timing {
        if t.order[t.index].actor != actor || t.action_spent {
            return Err(prerequisite(
                "action unavailable outside own turn or already spent",
            ));
        }
        t.action_spent = true;
    }
    Ok(())
}
pub(super) fn attack_modifier(e: &MechanicalEntity, ability: AttackAbility) -> i32 {
    match ability {
        AttackAbility::Strength => ability_modifier(e.ability_scores[0]),
        AttackAbility::Dexterity => ability_modifier(e.ability_scores[1]),
        AttackAbility::Finesse => {
            ability_modifier(e.ability_scores[0]).max(ability_modifier(e.ability_scores[1]))
        }
    }
}
pub(super) fn attack_context(
    rules: &RulesState,
    actor: EntityId,
    target: EntityId,
    p: &ActionPermission,
    ranged: bool,
) -> Result<(RollMode, bool), RulesError> {
    if entity(rules, target)?.death.dead {
        return Err(prerequisite("target is dead"));
    }
    let attacker = conditions(rules, actor);
    let defender = conditions(rules, target);
    let mut c = p.circumstances;
    if rules
        .effects
        .iter()
        .any(|e| e.target == actor && e.source == target && e.condition == Some(Condition::Charmed))
    {
        return Err(prerequisite("charmed creature cannot attack its charmer"));
    }
    if [
        Condition::Blinded,
        Condition::Poisoned,
        Condition::Prone,
        Condition::Restrained,
    ]
    .iter()
    .any(|x| attacker.contains(x))
    {
        c.disadvantage = true;
    }
    if [
        Condition::Blinded,
        Condition::Paralyzed,
        Condition::Petrified,
        Condition::Restrained,
        Condition::Stunned,
        Condition::Unconscious,
    ]
    .iter()
    .any(|x| defender.contains(x))
    {
        c.advantage = true;
    }
    if defender.contains(&Condition::Prone) {
        if p.within_five_feet {
            c.advantage = true;
        } else {
            c.disadvantage = true;
        }
    }
    if ranged && p.circumstances.ranged_threat {
        c.disadvantage = true;
    }
    let critical = p.within_five_feet
        && (defender.contains(&Condition::Paralyzed) || defender.contains(&Condition::Unconscious));
    Ok((mode(c), critical))
}

fn cast_spell(
    state: &CampaignState,
    rules: &mut RulesState,
    meta: &CommandMeta,
    pack: &RulesPack,
    args: (EntityId, EntityId, &str, u8, RollRequestId, EffectId),
    permission: Option<ActionPermission>,
) -> Result<RulesOutcome, RulesError> {
    let (actor, target, id, slot, request_id, effect_id) = args;
    authorize(state, meta, actor)?;
    ready(rules, actor)?;
    entity(rules, target)?;
    let p = permission
        .ok_or_else(|| prerequisite("spell needs authoritative contextual permission"))?;
    permission_matches(&p, meta, actor, target, id, true)?;
    let definition = pack.spell(id)?;
    let e = entity(rules, actor)?;
    if !e.prepared_spells.contains(id) {
        return Err(prerequisite("spell is not prepared/known"));
    }
    let casting = e
        .spellcasting
        .as_ref()
        .ok_or_else(|| prerequisite("actor cannot cast"))?;
    if (definition.verbal && !casting.can_speak)
        || (definition.somatic && !casting.free_hand)
        || (definition.material && !casting.material_focus)
    {
        return Err(prerequisite("required spell components unavailable"));
    }
    if (definition.level == 0 && slot != 0)
        || (definition.level > 0 && (slot < definition.level || slot > 9))
    {
        return Err(prerequisite("invalid spell slot level"));
    }
    if slot > 0 && casting.slots[usize::from(slot) - 1] == 0 {
        return Err(prerequisite("spell slot exhausted"));
    }
    if slot > 0
        && rules
            .timing
            .as_ref()
            .is_some_and(|t| t.slot_spent_this_turn)
    {
        return Err(prerequisite("one spell slot expenditure per turn"));
    }
    let modifier = ability_modifier(e.ability_scores[casting.ability.index()]);
    let level = e.level;
    let spell_attack = modifier + proficiency_bonus(level) - i32::from(e.exhaustion) * 2;
    let attack_context_result = if matches!(definition.effect, SpellEffect::AttackDamage { .. }) {
        Some(attack_context(rules, actor, target, &p, true)?)
    } else {
        None
    };
    spend_action(rules, actor)?;
    rules.completed_short_rests.retain(|x| *x != actor);
    if slot > 0 {
        entity_mut(rules, actor)?
            .spellcasting
            .as_mut()
            .ok_or_else(|| prerequisite("missing spellcasting"))?
            .slots[usize::from(slot) - 1] -= 1;
        if let Some(t) = &mut rules.timing {
            t.slot_spent_this_turn = true;
        }
        interrupt_rest(rules, actor, state.clock.now);
    }
    match &definition.effect {
        SpellEffect::Healing {
            dice,
            extra_dice_per_slot,
        } => {
            if entity(rules, target)?.death.dead {
                return Err(prerequisite("healing cannot restore the dead"));
            }
            let mut healing = dice.clone();
            for _ in definition.level..slot {
                healing.extend(extra_dice_per_slot.clone());
            }
            set_pending(
                rules,
                meta,
                RollRequest {
                    id: request_id,
                    roller: Some(actor),
                    dice: healing,
                    modifier,
                    mode: RollMode::Normal,
                    visibility: RollVisibility::Public,
                    reason: format!("spell:{id}"),
                },
                PendingPurpose::Healing {
                    target,
                    spell_id: id.into(),
                    slot_level: slot,
                    permission: p.clone(),
                },
                p.ruling,
            )
        }
        SpellEffect::AttackDamage {
            dice,
            damage_type,
            cantrip_scaling,
        } => {
            let mut damage = dice.clone();
            let multiplier = if *cantrip_scaling {
                1 + u16::from(level >= 5) + u16::from(level >= 11) + u16::from(level >= 17)
            } else {
                1
            };
            for d in &mut damage {
                d.count = d
                    .count
                    .checked_mul(multiplier)
                    .ok_or_else(|| invalid("damage dice overflow"))?;
            }
            let (roll_mode, critical) =
                attack_context_result.ok_or_else(|| invalid("missing attack context"))?;
            set_pending(
                rules,
                meta,
                RollRequest {
                    id: request_id,
                    roller: Some(actor),
                    dice: vec![DieSpec {
                        count: 1,
                        sides: 20,
                    }],
                    modifier: spell_attack,
                    mode: roll_mode,
                    visibility: RollVisibility::Public,
                    reason: format!("spell-attack:{id}"),
                },
                PendingPurpose::Attack {
                    target,
                    damage,
                    damage_modifier: 0,
                    damage_type: *damage_type,
                    automatic_critical: critical,
                    permission: p.clone(),
                },
                p.ruling,
            )
        }
        SpellEffect::ConcentrationMarker { duration_seconds } => {
            end_concentration(rules, actor);
            let expires = Expiry::AtTime(WorldInstant(
                state
                    .clock
                    .now
                    .0
                    .checked_add(i64::from(*duration_seconds))
                    .ok_or_else(|| invalid("duration overflow"))?,
            ));
            apply_effect(
                rules,
                ActiveEffect {
                    id: effect_id,
                    source: actor,
                    target,
                    condition: None,
                    label: format!("spell:{id}"),
                    expires,
                    concentration_owner: Some(actor),
                },
            )?;
            Ok(RulesOutcome::Changed)
        }
    }
}

fn submit(
    state: &CampaignState,
    rules: &mut RulesState,
    meta: &CommandMeta,
    result: &RollResult,
) -> Result<RulesOutcome, RulesError> {
    let pending = rules.pending.clone().ok_or(RulesError::NoPending)?;
    let actor = pending
        .request
        .roller
        .ok_or_else(|| invalid("pending roll without actor"))?;
    if result.source == RollSource::Digital || pending.request.visibility == RollVisibility::Secret
    {
        privileged(meta.issuer)?;
    }
    authorize(state, meta, actor)?;
    let roll = pending.request.resolve(result)?;
    rules.pending = None;
    rules.rolls.push(RecordedRoll {
        issued_by: pending.issued_by.clone(),
        accepted_by: meta.clone(),
        original_result: None,
        savage_attacker: None,
        request: pending.request.clone(),
        result: result.clone(),
        resolved: roll.clone(),
        purpose: pending.purpose.clone(),
    });
    let (mut success, mut critical, mut amount) = (None, false, None);
    match &pending.purpose {
        PendingPurpose::TacticalInitiative { .. } | PendingPurpose::TacticalResolution { .. } => {
            return Err(prerequisite(
                "tactical rolls require their recorded continuation",
            ));
        }
        PendingPurpose::Test { kind, dc, .. } => {
            let face = roll.kept_dice[0].value;
            if matches!(kind, TestKind::DeathSave) {
                let e = entity_mut(rules, actor)?;
                if face == 20 {
                    e.hp = 1;
                    e.death = DeathState::default();
                    success = Some(true);
                } else {
                    let passed = roll.total >= 10;
                    success = Some(passed);
                    if face == 1 {
                        e.death.failures = e.death.failures.saturating_add(2);
                    } else if passed {
                        e.death.successes += 1;
                    } else {
                        e.death.failures += 1;
                    }
                    if e.death.failures >= 3 {
                        kill(e);
                    } else if e.death.successes >= 3 {
                        e.death = DeathState {
                            stable: true,
                            ..DeathState::default()
                        };
                    }
                }
            } else if !matches!(kind, TestKind::Initiative) {
                success = Some(crate::test_outcome::ability_test_success(
                    &roll,
                    *dc,
                    &rules.house_rules,
                )?);
            }
        }
        PendingPurpose::Attack {
            target,
            damage,
            damage_modifier,
            damage_type,
            automatic_critical,
            ..
        } => {
            let face = roll.kept_dice[0].value;
            let hit =
                face == 20 || (face != 1 && roll.total >= armor_class(entity(rules, *target)?));
            success = Some(hit);
            critical = hit && (face == 20 || *automatic_critical);
            if hit {
                let mut dice = damage.clone();
                if critical {
                    for d in &mut dice {
                        d.count = d
                            .count
                            .checked_mul(2)
                            .ok_or_else(|| invalid("critical dice overflow"))?;
                    }
                }
                set_pending(
                    rules,
                    meta,
                    RollRequest {
                        id: RollRequestId(meta.id.0),
                        roller: Some(actor),
                        dice,
                        modifier: *damage_modifier,
                        mode: RollMode::Normal,
                        visibility: pending.request.visibility,
                        reason: "attack-damage".into(),
                    },
                    PendingPurpose::Damage {
                        target: *target,
                        damage_type: *damage_type,
                        critical,
                        attack_roll_id: pending.request.id,
                    },
                    pending.ruling.clone(),
                )?;
            }
        }
        PendingPurpose::Damage {
            target,
            damage_type,
            critical: is_critical,
            ..
        } => {
            critical = *is_critical;
            let applied = damage(
                rules,
                *target,
                roll.total.max(0) as u32,
                *damage_type,
                critical,
                state.clock.now,
            )?;
            amount = Some(applied);
            concentration_request(rules, *target, applied, meta, pending.ruling.clone())?;
        }
        PendingPurpose::Healing { target, .. } => {
            amount = Some(heal(rules, *target, roll.total.max(0) as u32)?);
        }
        PendingPurpose::Concentration { dc, .. } => {
            let passed = roll.total >= *dc;
            success = Some(passed);
            if !passed {
                end_concentration(rules, actor);
            }
        }
        PendingPurpose::RestHitDie => {
            amount = Some(heal(rules, actor, roll.total.max(1) as u32)?);
        }
        PendingPurpose::SecondWind => {
            amount = Some(heal(rules, actor, roll.total.max(0) as u32)?);
        }
    }
    Ok(RulesOutcome::RollResolved {
        roll,
        success,
        critical,
        amount,
        followup: rules.pending.as_ref().map(|p| p.request.clone()),
    })
}

fn damage(
    rules: &mut RulesState,
    target: EntityId,
    amount: u32,
    damage_type: DamageType,
    critical: bool,
    now: WorldInstant,
) -> Result<u32, RulesError> {
    if amount > 1_000_000 {
        return Err(invalid("damage outside supported range"));
    }
    let petrified = conditions(rules, target).contains(&Condition::Petrified);
    let e = entity_mut(rules, target)?;
    if e.death.dead {
        return Err(prerequisite("target is dead"));
    }
    let mut applied = if e.damage_immunities.contains(&damage_type) {
        0
    } else {
        amount
    };
    if petrified || e.resistances.contains(&damage_type) {
        applied /= 2;
    }
    if e.vulnerabilities.contains(&damage_type) {
        applied = applied
            .checked_mul(2)
            .ok_or_else(|| invalid("damage overflow"))?;
    }
    if applied == 0 {
        return Ok(0);
    }
    let hp_damage = applied.saturating_sub(e.temporary_hp);
    e.temporary_hp = e.temporary_hp.saturating_sub(applied);
    if e.hp == 0 {
        e.death.stable = false;
        e.death.failures = e
            .death
            .failures
            .saturating_add(if critical { 2 } else { 1 });
        if applied >= e.max_hp || e.death.failures >= 3 {
            kill(e);
        }
    } else if hp_damage >= e.hp {
        let remaining = hp_damage - e.hp;
        e.hp = 0;
        e.prone = true;
        if !e.uses_death_saves || remaining >= e.max_hp {
            kill(e);
        }
    } else {
        e.hp -= hp_damage;
    }
    let incap = e.hp == 0 || e.death.dead;
    rules.completed_short_rests.retain(|id| *id != target);
    interrupt_rest(rules, target, now);
    if incap {
        end_concentration(rules, target);
    }
    Ok(applied)
}
fn heal(rules: &mut RulesState, target: EntityId, amount: u32) -> Result<u32, RulesError> {
    if amount > 1_000_000 {
        return Err(invalid("healing outside supported range"));
    }
    let e = entity_mut(rules, target)?;
    if e.death.dead {
        return Err(prerequisite("healing cannot revive a dead creature"));
    }
    let regained = amount.min(e.max_hp - e.hp);
    e.hp += regained;
    if e.hp > 0 {
        e.death = DeathState::default();
    }
    Ok(regained)
}
fn kill(e: &mut MechanicalEntity) {
    e.hp = 0;
    e.death = DeathState {
        dead: true,
        ..DeathState::default()
    };
}
fn concentration_request(
    rules: &mut RulesState,
    target: EntityId,
    damage: u32,
    meta: &CommandMeta,
    ruling: Ruling,
) -> Result<Option<RollRequest>, RulesError> {
    if damage == 0 || entity(rules, target)?.concentration.is_none() {
        return Ok(None);
    }
    let e = entity(rules, target)?;
    let kind = TestKind::Save {
        ability: Ability::Constitution,
    };
    let dc = (damage / 2).clamp(10, 30) as i32;
    let request = RollRequest {
        id: RollRequestId(meta.id.0),
        roller: Some(target),
        dice: vec![DieSpec {
            count: 1,
            sides: 20,
        }],
        modifier: check_modifier(e, &kind),
        mode: check_mode(rules, target, &kind, Circumstances::default()),
        visibility: RollVisibility::Public,
        reason: "concentration-save".into(),
    };
    set_pending(
        rules,
        meta,
        request.clone(),
        PendingPurpose::Concentration {
            dc,
            damage_taken: damage,
        },
        ruling,
    )?;
    Ok(Some(request))
}
fn apply_effect(rules: &mut RulesState, effect: ActiveEffect) -> Result<(), RulesError> {
    entity(rules, effect.source)?;
    let target = entity(rules, effect.target)?;
    if rules.effects.iter().any(|e| e.id == effect.id) {
        return Err(invalid("duplicate effect identifier"));
    }
    if effect.condition == Some(Condition::Poisoned)
        && conditions(rules, effect.target).contains(&Condition::Petrified)
    {
        return Err(prerequisite("petrified creature is immune to Poisoned"));
    }
    if effect
        .condition
        .is_some_and(|c| target.condition_immunities.contains(&c))
    {
        return Err(prerequisite("target immune to condition"));
    }
    if let Some(owner) = effect.concentration_owner {
        ready(rules, owner)?;
        end_concentration(rules, owner);
        entity_mut(rules, owner)?.concentration = Some(effect.id);
    }
    let target = effect.target;
    if effect.condition == Some(Condition::Unconscious) {
        entity_mut(rules, target)?.prone = true;
    }
    rules.effects.push(effect);
    if conditions(rules, target).contains(&Condition::Incapacitated) {
        end_concentration(rules, target);
    }
    Ok(())
}
fn remove_effects(rules: &mut RulesState, remove: impl Fn(&ActiveEffect) -> bool) {
    let removed: HashSet<_> = rules
        .effects
        .iter()
        .filter(|e| remove(e))
        .map(|e| e.id)
        .collect();
    rules.effects.retain(|e| !removed.contains(&e.id));
    for e in rules.entities.values_mut() {
        if e.concentration.is_some_and(|id| removed.contains(&id)) {
            e.concentration = None;
        }
    }
}
fn end_concentration(rules: &mut RulesState, actor: EntityId) {
    remove_effects(rules, |e| e.concentration_owner == Some(actor));
    if let Some(e) = rules.entities.get_mut(&actor) {
        e.concentration = None;
    }
}
fn sync_deaths(state: &mut CampaignState) {
    if let Some(rules) = &state.rules {
        for e in rules.entities.values().filter(|e| e.death.dead) {
            if let Some(world) = state.entities.get_mut(&e.entity_id) {
                world.existence = EntityExistence::Dead;
            }
            for c in state
                .characters
                .values_mut()
                .filter(|c| c.entity_id == e.entity_id)
            {
                c.status = CharacterStatus::Dead;
            }
        }
    }
}
fn recover(e: &mut MechanicalEntity, kind: RestKind) {
    if let Some(f) = &mut e.character_features {
        f.second_wind_remaining = if kind == RestKind::Long {
            2
        } else {
            f.second_wind_remaining.saturating_add(1).min(2)
        };
    }
    for pool in e.resources.values_mut() {
        if pool.recovery == Recovery::ShortOrLongRest
            || (kind == RestKind::Long && pool.recovery == Recovery::LongRest)
        {
            pool.remaining = pool.maximum;
        }
    }
}
pub(crate) fn interrupt_rest(rules: &mut RulesState, actor: EntityId, now: WorldInstant) {
    if rules.rests.iter().any(|r| {
        r.actor == actor && r.kind == RestKind::Long && now.0.saturating_sub(r.started_at.0) >= 3600
    }) {
        if let Some(e) = rules.entities.get_mut(&actor) {
            recover(e, RestKind::Short);
        }
        if !rules.completed_short_rests.contains(&actor) {
            rules.completed_short_rests.push(actor);
        }
    }
    rules.rests.retain(|r| r.actor != actor);
    if let Some(recovery) = rules
        .tactical_recovery
        .as_mut()
        .and_then(|records| records.get_mut(&actor))
    {
        recovery.knockout_rest = None;
        if let Some(knockout) = &mut recovery.knockout {
            knockout.short_rest_started_at = None;
        }
    }
}
fn finish_rest(
    state: &CampaignState,
    rules: &mut RulesState,
    meta: &CommandMeta,
    actor: EntityId,
    slept: u32,
    ruling: &Ruling,
) -> Result<RulesOutcome, RulesError> {
    adjudicate(rules, meta, ruling)?;
    let rest = rules
        .rests
        .iter()
        .find(|r| r.actor == actor)
        .cloned()
        .ok_or_else(|| prerequisite("no active rest"))?;
    let elapsed = state.clock.now.0.saturating_sub(rest.started_at.0);
    let minimum = if rest.kind == RestKind::Long {
        8 * 3600
    } else {
        3600
    };
    if elapsed < minimum
        || i64::from(slept) > elapsed
        || (rest.kind == RestKind::Long
            && (slept < 6 * 3600 || elapsed - i64::from(slept) > 2 * 3600))
    {
        return Err(prerequisite("rest duration/sleep requirement unmet"));
    }
    let e = entity_mut(rules, actor)?;
    if e.hp == 0 || e.death.dead {
        return Err(prerequisite("rest participant is at zero hit points"));
    }
    recover(e, rest.kind);
    if rest.kind == RestKind::Long {
        e.hp = e.max_hp;
        e.temporary_hp = 0;
        e.hit_dice.remaining = e.hit_dice.maximum;
        e.exhaustion = e.exhaustion.saturating_sub(1);
        e.last_long_rest_finished = Some(state.clock.now);
        if let Some(c) = &mut e.spellcasting {
            c.slots = c.slot_maxima;
        }
        if let Some(f) = &mut e.character_features
            && f.human_resourceful
        {
            if e.heroic_inspiration {
                f.inspiration_transfer_pending = true;
            } else {
                e.heroic_inspiration = true;
            }
        }
    } else if !rules.completed_short_rests.contains(&actor) {
        rules.completed_short_rests.push(actor);
    }
    rules.rests.retain(|r| r.actor != actor);
    Ok(RulesOutcome::Changed)
}
fn start_combat(
    state: &CampaignState,
    rules: &mut RulesState,
    meta: &CommandMeta,
    participants: &[InitiativeEntry],
    ruling: &Ruling,
) -> Result<RulesOutcome, RulesError> {
    adjudicate(rules, meta, ruling)?;
    if rules.timing.is_some() || participants.is_empty() {
        return Err(prerequisite("combat already started or empty"));
    }
    let mut ids = HashSet::new();
    let mut ties = HashSet::new();
    for p in participants {
        let e = entity(rules, p.actor)?;
        if e.death.dead || !ids.insert(p.actor) || !ties.insert((p.total, p.tie_break)) {
            return Err(invalid("invalid combat participant/tie order"));
        }
        let latest = rules.rolls.iter().rev().find(|r| {
            r.request.roller == Some(p.actor)
                && matches!(
                    r.purpose,
                    PendingPurpose::Test {
                        kind: TestKind::Initiative,
                        ..
                    }
                )
        });
        if latest.is_none_or(|r| r.resolved.total != p.total) {
            return Err(prerequisite(
                "initiative total must match a recorded initiative roll",
            ));
        }
        interrupt_rest(rules, p.actor, state.clock.now);
    }
    let mut order = participants.to_vec();
    order.sort_by(|a, b| b.total.cmp(&a.total).then(a.tie_break.cmp(&b.tie_break)));
    let first = order[0].actor;
    rules.timing = Some(CombatTiming {
        order,
        index: 0,
        round: 1,
        turn_number: 1,
        action_spent: false,
        bonus_action_spent: false,
        slot_spent_this_turn: false,
        reactions_spent: vec![],
    });
    rules.rests.clear();
    rules.completed_short_rests.clear();
    turn_start(rules, first, meta)
}
fn turn_start(
    rules: &mut RulesState,
    actor: EntityId,
    meta: &CommandMeta,
) -> Result<RulesOutcome, RulesError> {
    let e = entity(rules, actor)?;
    if e.hp == 0 && !e.death.dead && !e.death.stable && e.uses_death_saves {
        let kind = TestKind::DeathSave;
        let request = RollRequest {
            id: RollRequestId(meta.id.0),
            roller: Some(actor),
            dice: vec![DieSpec {
                count: 1,
                sides: 20,
            }],
            modifier: check_modifier(e, &kind),
            mode: RollMode::Normal,
            visibility: RollVisibility::Public,
            reason: "death-save".into(),
        };
        set_pending(
            rules,
            meta,
            request,
            PendingPurpose::Test {
                kind,
                dc: 10,
                circumstances: Circumstances::default(),
            },
            srd(18, "Death saving throw at the start of a turn"),
        )
    } else {
        Ok(RulesOutcome::Changed)
    }
}
