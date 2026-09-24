use super::*;
use crate::ResolveRoll;
use std::collections::HashSet;

pub fn ability_modifier(score: u8) -> i32 {
    (i32::from(score) - 10).div_euclid(2)
}
pub fn proficiency_bonus(level: u8) -> i32 {
    2 + (i32::from(level.saturating_sub(1)) / 4)
}
pub fn armor_class(entity: &MechanicalEntity) -> i32 {
    match entity.armor {
        ArmorClass::Fixed(ac) => i32::from(ac),
        ArmorClass::HeavyArmor { base, shield } => i32::from(base) + if shield { 2 } else { 0 },
        ArmorClass::Armor {
            base,
            dexterity_cap,
            shield,
        } => {
            i32::from(base)
                + dexterity_cap.map_or(ability_modifier(entity.ability_scores[1]), |cap| {
                    ability_modifier(entity.ability_scores[1]).min(i32::from(cap))
                })
                + if shield { 2 } else { 0 }
        }
    }
}
pub(super) fn entity(rules: &RulesState, id: EntityId) -> Result<&MechanicalEntity, RulesError> {
    rules
        .entities
        .get(&id)
        .ok_or_else(|| invalid("unknown mechanical entity"))
}
pub(super) fn entity_mut(
    rules: &mut RulesState,
    id: EntityId,
) -> Result<&mut MechanicalEntity, RulesError> {
    rules
        .entities
        .get_mut(&id)
        .ok_or_else(|| invalid("unknown mechanical entity"))
}
pub(super) fn privileged(issuer: CommandIssuer) -> Result<(), RulesError> {
    if matches!(issuer, CommandIssuer::System | CommandIssuer::Admin) {
        Ok(())
    } else {
        Err(RulesError::Unauthorized)
    }
}
pub(super) fn owns(state: &CampaignState, issuer: CommandIssuer, id: EntityId) -> bool {
    match issuer {
        CommandIssuer::System | CommandIssuer::Admin => true,
        CommandIssuer::Player(player) => {
            state.players.contains_key(&player)
                && state
                    .characters
                    .values()
                    .any(|c| c.entity_id == id && c.controlling_player_id == Some(player))
        }
        CommandIssuer::Import => false,
    }
}
pub(super) fn authorize(
    state: &CampaignState,
    meta: &CommandMeta,
    id: EntityId,
) -> Result<(), RulesError> {
    if matches!(meta.issuer, CommandIssuer::Player(_))
        && !state
            .characters
            .values()
            .any(|c| c.entity_id == id && c.status == CharacterStatus::Active)
    {
        return Err(RulesError::Unauthorized);
    }
    if !owns(state, meta.issuer, id)
        || (matches!(meta.issuer, CommandIssuer::Player(_))
            && meta.actor != Some(AgentRef::Entity(id)))
        || (meta.actor.is_some() && meta.actor != Some(AgentRef::Entity(id)))
    {
        return Err(RulesError::Unauthorized);
    }
    Ok(())
}
pub(super) fn ruling_valid(ruling: &Ruling, houses: &HouseRules) -> Result<(), RulesError> {
    if ruling.reason.trim().is_empty() || ruling.reason.len() > 2000 {
        return Err(invalid("ruling requires a bounded explanation"));
    }
    match &ruling.basis {
        RulingBasis::Srd { page } if !(1..=364).contains(page) => {
            Err(invalid("invalid SRD source page"))
        }
        RulingBasis::HouseRule { id }
            if id != "ability-test-natural-extremes" || !houses.ability_test_natural_extremes =>
        {
            Err(invalid("house ruling refers to a disabled/unknown rule"))
        }
        _ => Ok(()),
    }
}
pub(super) fn conditions(rules: &RulesState, id: EntityId) -> HashSet<Condition> {
    let mut result: HashSet<_> = rules
        .effects
        .iter()
        .filter(|e| e.target == id)
        .filter_map(|e| e.condition)
        .collect();
    if rules.entities.get(&id).is_some_and(|e| e.hp == 0) {
        result.insert(Condition::Unconscious);
    }
    if result.contains(&Condition::Unconscious) {
        result.insert(Condition::Prone);
    }
    if rules.entities.get(&id).is_some_and(|e| e.prone) {
        result.insert(Condition::Prone);
    }
    if result.contains(&Condition::Petrified) {
        result.remove(&Condition::Poisoned);
    }
    if [
        Condition::Unconscious,
        Condition::Paralyzed,
        Condition::Petrified,
        Condition::Stunned,
    ]
    .iter()
    .any(|c| result.contains(c))
    {
        result.insert(Condition::Incapacitated);
    }
    result
}
pub(super) fn ready(rules: &RulesState, id: EntityId) -> Result<(), RulesError> {
    let e = entity(rules, id)?;
    if e.death.dead || conditions(rules, id).contains(&Condition::Incapacitated) {
        return Err(prerequisite("actor cannot act while dead or incapacitated"));
    }
    Ok(())
}
pub(super) fn mode(c: Circumstances) -> RollMode {
    match (c.advantage, c.disadvantage) {
        (true, false) => RollMode::Advantage,
        (false, true) => RollMode::Disadvantage,
        _ => RollMode::Normal,
    }
}
pub(super) fn check_modifier(e: &MechanicalEntity, kind: &TestKind) -> i32 {
    let base = match kind {
        TestKind::Check { ability, skill } => {
            ability_modifier(e.ability_scores[ability.index()])
                + skill
                    .and_then(|s| e.skill_proficiencies.get(&s))
                    .map_or(0, |p| {
                        proficiency_bonus(e.level)
                            * if *p == Proficiency::Expertise { 2 } else { 1 }
                    })
        }
        TestKind::Save { ability } => {
            ability_modifier(e.ability_scores[ability.index()])
                + if e.saving_proficiencies.contains(ability) {
                    proficiency_bonus(e.level)
                } else {
                    0
                }
        }
        TestKind::Initiative => ability_modifier(e.ability_scores[Ability::Dexterity.index()]),
        TestKind::DeathSave => 0,
    };
    base - i32::from(e.exhaustion) * 2
}
pub(super) fn check_mode(
    rules: &RulesState,
    actor: EntityId,
    kind: &TestKind,
    mut c: Circumstances,
) -> RollMode {
    let cs = conditions(rules, actor);
    if matches!(kind, TestKind::Initiative) {
        c.advantage |= cs.contains(&Condition::Invisible);
        c.disadvantage |= cs.contains(&Condition::Incapacitated);
    }
    if matches!(kind, TestKind::Check { .. } | TestKind::Initiative)
        && cs.contains(&Condition::Poisoned)
    {
        c.disadvantage = true;
    }
    if matches!(
        kind,
        TestKind::Save {
            ability: Ability::Dexterity
        }
    ) && cs.contains(&Condition::Restrained)
    {
        c.disadvantage = true;
    }
    mode(c)
}
pub(super) fn validate_entity(
    e: &MechanicalEntity,
    state: &CampaignState,
    pack: &RulesPack,
) -> Result<(), RulesError> {
    let world = state
        .entities
        .get(&e.entity_id)
        .ok_or_else(|| invalid("mechanics requires an existing world entity"))?;
    if world.campaign_id != state.campaign_id()
        || matches!(
            world.existence,
            EntityExistence::Destroyed | EntityExistence::Missing
        )
    {
        return Err(invalid("mechanical entity is unavailable in this campaign"));
    }
    if !(1..=20).contains(&e.level)
        || e.ability_scores.iter().any(|s| !(1..=30).contains(s))
        || e.max_hp == 0
        || e.max_hp > 1_000_000
        || e.hp > e.max_hp
        || e.temporary_hp > 1_000_000
        || e.exhaustion > 6
    {
        return Err(invalid(
            "invalid level, ability score, hit points or exhaustion",
        ));
    }
    if e.death.successes > 2
        || e.death.failures > 2
        || (e.death.dead && (e.hp != 0 || world.existence != EntityExistence::Dead))
        || (!e.death.dead && world.existence == EntityExistence::Dead)
        || (e.exhaustion == 6 && !e.death.dead)
        || (e.hp > 0 && (e.death.stable || e.death.successes != 0 || e.death.failures != 0))
        || (e.death.stable && (e.death.successes != 0 || e.death.failures != 0))
    {
        return Err(invalid("inconsistent death state"));
    }
    if (e.hp == 0 && !e.death.dead && (!e.prone || !e.uses_death_saves))
        || (e.death.dead && (e.death.stable || e.death.successes != 0 || e.death.failures != 0))
    {
        return Err(invalid("inconsistent zero-HP/death conditions"));
    }
    match e.armor {
        ArmorClass::HeavyArmor { base, .. } if !(1..=30).contains(&base) => {
            return Err(invalid("invalid heavy armor class"));
        }
        ArmorClass::Fixed(ac) if !(1..=40).contains(&ac) => {
            return Err(invalid("invalid fixed armor class"));
        }
        ArmorClass::Armor {
            base,
            dexterity_cap,
            ..
        } if !(1..=30).contains(&base) || dexterity_cap.is_some_and(|v| !(0..=10).contains(&v)) => {
            return Err(invalid("invalid armor calculation"));
        }
        _ => (),
    }
    if ![6, 8, 10, 12].contains(&e.hit_dice.sides)
        || e.hit_dice.maximum != e.level
        || e.hit_dice.remaining > e.hit_dice.maximum
    {
        return Err(invalid("invalid hit dice pool"));
    }
    if e.resources
        .iter()
        .any(|(id, p)| !definitions::valid_id(id) || p.maximum == 0 || p.remaining > p.maximum)
    {
        return Err(invalid("invalid resource pool"));
    }
    for id in &e.attacks {
        pack.attack(id)?;
    }
    if !e.attack_proficiencies.is_subset(&e.attacks) {
        return Err(invalid("weapon proficiency outside granted loadout"));
    }
    for id in &e.prepared_spells {
        pack.spell(id)?;
    }
    if let Some(s) = &e.spellcasting {
        if s.slots
            .iter()
            .zip(s.slot_maxima)
            .any(|(current, max)| *current > max || max > 20)
        {
            return Err(invalid("invalid spell slots"));
        }
    } else if !e.prepared_spells.is_empty() {
        return Err(invalid("prepared spells without spellcasting"));
    }
    Ok(())
}
pub fn validate_state(state: &CampaignState, pack: &RulesPack) -> Result<(), RulesError> {
    if state.schema_version != CURRENT_STATE_SCHEMA_VERSION {
        return Err(RulesError::Incompatible(
            "unsupported campaign state schema".into(),
        ));
    }
    pack.validate()?;
    if state.campaign.ruleset.id != pack.id || state.campaign.ruleset.version != pack.version {
        return Err(RulesError::Incompatible(
            "campaign rules pin does not match kernel".into(),
        ));
    }
    let Some(rules) = &state.rules else {
        return Ok(());
    };
    if rules.pack_id != pack.id || rules.pack_version != pack.version {
        return Err(RulesError::Incompatible(
            "mechanical state rules pin mismatch".into(),
        ));
    }
    if rules.entities.is_empty() {
        return Err(invalid("empty mechanical state"));
    }
    for (id, e) in &rules.entities {
        if *id != e.entity_id {
            return Err(invalid("mechanical entity key mismatch"));
        }
        validate_entity(e, state, pack)?;
    }
    let mut effects = HashSet::new();
    for effect in &rules.effects {
        entity(rules, effect.source)?;
        let target = entity(rules, effect.target)?;
        if !effects.insert(effect.id)
            || effect.label.trim().is_empty()
            || effect.label.len() > 200
            || effect
                .condition
                .is_some_and(|c| target.condition_immunities.contains(&c))
        {
            return Err(invalid("invalid active effect"));
        }
        if let Some(owner) = effect.concentration_owner
            && entity(rules, owner)?.concentration != Some(effect.id)
        {
            return Err(invalid("orphaned concentration effect"));
        }
        if effect.condition == Some(Condition::Unconscious) && !target.prone {
            return Err(invalid("Unconscious creature must remain Prone"));
        }
        match effect.expires {
            Expiry::AtTime(time) if time <= state.clock.now => {
                return Err(invalid("expired time effect"));
            }
            Expiry::AtTurn {
                actor,
                turn_number,
                boundary,
            } => {
                entity(rules, actor)?;
                let t = rules
                    .timing
                    .as_ref()
                    .ok_or_else(|| invalid("turn expiry outside initiative"))?;
                if t.order.is_empty()
                    || t.index >= t.order.len()
                    || turn_number < t.turn_number
                    || (turn_number == t.turn_number && boundary == TurnBoundary::Start)
                {
                    return Err(invalid("invalid turn expiry"));
                }
                let index = (t.index
                    + ((turn_number - t.turn_number) % t.order.len() as u64) as usize)
                    % t.order.len();
                if t.order[index].actor != actor {
                    return Err(invalid(
                        "turn expiry actor does not match the specified turn",
                    ));
                }
            }
            _ => (),
        }
    }
    for e in rules.entities.values() {
        if let Some(effect_id) = e.concentration
            && (!rules
                .effects
                .iter()
                .any(|x| x.id == effect_id && x.concentration_owner == Some(e.entity_id))
                || conditions(rules, e.entity_id).contains(&Condition::Incapacitated)
                || e.death.dead)
        {
            return Err(invalid("invalid concentration owner"));
        }
    }
    let mut rolls = HashSet::new();
    for roll in &rules.rolls {
        let roller = roll
            .request
            .roller
            .ok_or_else(|| invalid("recorded roll without actor"))?;
        entity(rules, roller)?;
        match &roll.purpose {
            PendingPurpose::Test { .. }
            | PendingPurpose::Attack { .. }
            | PendingPurpose::Concentration { .. }
                if roll.request.dice
                    != [DieSpec {
                        count: 1,
                        sides: 20,
                    }] =>
            {
                return Err(invalid("recorded d20 test has invalid dice"));
            }
            _ => (),
        }
        match &roll.purpose {
            PendingPurpose::Attack { target, .. }
            | PendingPurpose::Damage { target, .. }
            | PendingPurpose::Healing { target, .. } => {
                entity(rules, *target)?;
            }
            _ => (),
        }
        request_integrity::command(state, &roll.issued_by)?;
        request_integrity::command(state, &roll.accepted_by)?;
        if roll.accepted_by.expected_event_sequence <= roll.issued_by.expected_event_sequence
            || roll
                .accepted_by
                .actor
                .is_some_and(|a| Some(a) != roll.request.roller.map(AgentRef::Entity))
        {
            return Err(invalid("invalid roll acceptance provenance"));
        }
        if !rolls.insert(roll.request.id) || roll.request.resolve(&roll.result)? != roll.resolved {
            return Err(invalid("invalid recorded roll"));
        }
        if let Some(original) = &roll.original_result {
            roll.request.resolve(original)?;
            if original.request_id != roll.result.request_id
                || original.source != roll.result.source
                || original
                    .dice
                    .iter()
                    .zip(&roll.result.dice)
                    .filter(|(a, b)| a != b)
                    .count()
                    > 1
            {
                return Err(invalid("invalid inspiration reroll record"));
            }
        }
        if roll.result.source == RollSource::Digital
            || roll.request.visibility == RollVisibility::Secret
        {
            privileged(roll.accepted_by.issuer)?;
        }
    }
    for id in &rules.cancelled_roll_ids {
        if !rolls.insert(*id) {
            return Err(invalid("duplicate/colliding cancelled roll identifier"));
        }
    }
    if let Some(p) = &rules.pending {
        p.request.validate()?;
        if rolls.contains(&p.request.id) || p.request.roller.is_none() {
            return Err(invalid("invalid pending request identity"));
        }
        entity(
            rules,
            p.request.roller.ok_or_else(|| invalid("missing roller"))?,
        )?;
        ruling_valid(&p.ruling, &rules.house_rules)?;
        request_integrity::pending(state, rules, p, pack)?;
    }
    let mut ruling_commands = HashSet::new();
    for r in &rules.rulings {
        request_integrity::command(state, &r.command)?;
        privileged(r.command.issuer)?;
        if !ruling_commands.insert(r.command.id) {
            return Err(invalid("duplicate ruling command"));
        }
        ruling_valid(&r.ruling, &rules.house_rules)?;
    }
    if let Some(t) = &rules.timing {
        let mut actors = HashSet::new();
        if t.order.is_empty() || t.index >= t.order.len() || t.round == 0 || t.turn_number == 0 {
            return Err(invalid("invalid initiative cursor"));
        }
        for x in &t.order {
            entity(rules, x.actor)?;
            if !actors.insert(x.actor) {
                return Err(invalid("duplicate initiative actor"));
            }
        }
        let mut reactions = HashSet::new();
        for id in &t.reactions_spent {
            if !actors.contains(id) || !reactions.insert(id) {
                return Err(invalid("invalid reaction budget"));
            }
        }
    }
    let mut rests = HashSet::new();
    for r in &rules.rests {
        entity(rules, r.actor)?;
        if !rests.insert(r.actor) || r.started_at > state.clock.now || rules.timing.is_some() {
            return Err(invalid("invalid rest state"));
        }
    }
    let mut completed = HashSet::new();
    for id in &rules.completed_short_rests {
        entity(rules, *id)?;
        if !completed.insert(*id) {
            return Err(invalid("duplicate rest completion"));
        }
    }
    if let Some(p) = &rules.permission {
        request_integrity::permission(state, rules, p)?;
        if state
            .applied_event_sequence
            .saturating_sub(p.issued_by.expected_event_sequence)
            > 1
        {
            return Err(invalid("stale action permission"));
        }
        entity(rules, p.actor)?;
        entity(rules, p.target)?;
        ruling_valid(&p.ruling, &rules.house_rules)?;
        if p.spell {
            pack.spell(&p.content_id)?;
        } else {
            pack.attack(&p.content_id)?;
        }
    }
    Ok(())
}
