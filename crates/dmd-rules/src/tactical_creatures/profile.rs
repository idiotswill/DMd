use std::collections::HashSet;

use super::*;
use crate::{ResolveRoll, ability_modifier, tactical_definitions::*};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CreatureHitPointChoice {
    Average,
    Rolled {
        request_id: RollRequestId,
        result: RollResult,
    },
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatureBuildChoice {
    pub definition_id: String,
    pub size: CreatureSize,
    pub additional_languages: Vec<String>,
    pub hit_points: CreatureHitPointChoice,
    pub controller: CreatureController,
    pub in_lair: bool,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuiltCreature {
    pub profile: CreatureProfile,
    pub runtime: CreatureRuntime,
    pub mechanics: MechanicalEntity,
    pub movement: MovementProfile,
    pub senses: Senses,
}

pub fn build_creature(
    state: &CampaignState,
    meta: &CommandMeta,
    actor: EntityId,
    choice: &CreatureBuildChoice,
) -> Result<BuiltCreature, CreatureError> {
    if meta.expected_event_sequence != state.applied_event_sequence {
        return Err(CreatureError::Stale);
    }
    validate_creature_origin(state, meta, actor).map_err(invalid)?;
    if !matches!(meta.issuer, CommandIssuer::System | CommandIssuer::Admin)
        || meta
            .actor
            .is_some_and(|subject| subject != AgentRef::Entity(actor))
        || state.characters.values().any(|pc| pc.entity_id == actor)
    {
        return Err(CreatureError::Unauthorized);
    }
    let world = &state.entities[&actor];
    if world.existence != EntityExistence::Present
        || !matches!(world.kind, EntityKind::Npc | EntityKind::Creature)
        || state
            .rules
            .as_ref()
            .is_some_and(|rules| rules.entities.contains_key(&actor))
    {
        return Err(invalid(
            "source creation requires a new living NPC/creature mechanical identity",
        ));
    }
    if let CreatureController::Player(id) = choice.controller
        && !state.players.contains_key(&id)
    {
        return Err(invalid("unknown controller"));
    }
    let definitions = creature_definitions()?;
    if state.campaign.ruleset.id != definitions.ruleset_id
        || state.campaign.ruleset.version != definitions.ruleset_version
    {
        return Err(invalid("creature source and campaign ruleset differ"));
    }
    let source = creature_definition(&choice.definition_id)?;
    let hit_points = match &choice.hit_points {
        CreatureHitPointChoice::Average => CreatureHitPointOrigin::Average,
        CreatureHitPointChoice::Rolled { request_id, result } => {
            let request = creature_hit_point_request(actor, source, *request_id)?;
            request
                .resolve(result)
                .map_err(|error| invalid(error.to_string()))?;
            CreatureHitPointOrigin::Rolled {
                request,
                result: result.clone(),
            }
        }
    };
    let profile = CreatureProfile {
        actor,
        origin: meta.clone(),
        source: CreatureSourcePin {
            ruleset_id: definitions.ruleset_id.clone(),
            ruleset_version: definitions.ruleset_version.clone(),
            definition_id: source.id.clone(),
            definition_fingerprint: creature_definition_fingerprint(source)?,
        },
        size: choice.size,
        additional_languages: choice.additional_languages.clone(),
        hit_points,
    };
    validate_profile_choices(&profile, source)?;
    let mechanics = initial_creature_mechanics(&profile)?;
    let runtime = CreatureRuntime {
        actor,
        controller: choice.controller,
        control_origin: meta.clone(),
        in_lair: choice.in_lair,
        lair_origin: meta.clone(),
        recharge: initial_recharge(source),
        limited_uses: initial_limited_uses(source),
        used_this_own_turn: vec![],
        legendary_spent: 0,
        legendary_resistance_spent: 0,
        observed_turn: None,
        legendary_window_spent: false,
        routine: None,
        last_rest: None,
        last_operation: meta.clone(),
    };
    Ok(BuiltCreature {
        profile,
        runtime,
        mechanics,
        movement: creature_movement(source),
        senses: creature_senses(source),
    })
}

pub fn creature_hit_point_request(
    actor: EntityId,
    source: &CreatureDefinition,
    id: RollRequestId,
) -> Result<RollRequest, CreatureError> {
    if actor.0.is_nil() || id.0.is_nil() {
        return Err(invalid("creature HP roll needs non-nil identities"));
    }
    let request = RollRequest {
        id,
        roller: Some(actor),
        dice: source.statistics.hit_point_formula.dice.clone(),
        modifier: i32::from(source.statistics.hit_point_formula.fixed),
        mode: RollMode::Normal,
        visibility: RollVisibility::Secret,
        reason: format!("{} initial hit points", source.name),
    };
    request
        .validate()
        .map_err(|error| invalid(error.to_string()))?;
    Ok(request)
}

fn validate_profile_choices(
    profile: &CreatureProfile,
    source: &CreatureDefinition,
) -> Result<(), CreatureError> {
    if !source
        .statistics
        .allowed_sizes
        .iter()
        .any(|size| source_size(*size) == profile.size)
        || profile.additional_languages.len() != usize::from(source.statistics.additional_languages)
    {
        return Err(invalid("creature size/language choices differ from source"));
    }
    let mut languages = source
        .statistics
        .languages
        .iter()
        .map(|language| language.to_lowercase())
        .collect::<HashSet<_>>();
    for language in &profile.additional_languages {
        if language.trim().is_empty()
            || language.len() > 160
            || language.chars().any(char::is_control)
            || !languages.insert(language.to_lowercase())
        {
            return Err(invalid("invalid or duplicate additional creature language"));
        }
    }
    if let CreatureHitPointOrigin::Rolled { request, result } = &profile.hit_points {
        if *request != creature_hit_point_request(profile.actor, source, request.id)? {
            return Err(invalid("creature HP request differs from source"));
        }
        let total = request
            .resolve(result)
            .map_err(|error| invalid(error.to_string()))?
            .total;
        if !(1..=1_000_000).contains(&total) {
            return Err(invalid("rolled creature HP outside live bounds"));
        }
    }
    Ok(())
}

/// Baseline construction only; callers preserve live HP/death/effects on restore.
pub fn initial_creature_mechanics(
    profile: &CreatureProfile,
) -> Result<MechanicalEntity, CreatureError> {
    let source = source_for_profile(profile)?;
    validate_profile_choices(profile, source)?;
    let statistics = &source.statistics;
    let [die] = statistics.hit_point_formula.dice.as_slice() else {
        return Err(invalid("creature HD requires one source pool"));
    };
    if ![4, 6, 8, 10, 12, 20].contains(&die.sides)
        || die.count == 0
        || die.count > u16::from(u8::MAX)
    {
        return Err(invalid(
            "source creature Hit Dice are outside supported pool bounds",
        ));
    }
    let max_hp = match &profile.hit_points {
        CreatureHitPointOrigin::Average => statistics.hit_points,
        CreatureHitPointOrigin::Rolled { request, result } => {
            request
                .resolve(result)
                .map_err(|error| invalid(error.to_string()))?
                .total as u32
        }
    };
    let mut entity = MechanicalEntity::basic(profile.actor);
    entity.level = 0; // Explicit non-PC sentinel. Never an inferred CR/HD character level.
    entity.ability_scores = statistics.ability_scores;
    entity.armor = ArmorClass::Fixed(u16::from(statistics.armor_class));
    entity.max_hp = max_hp;
    entity.hp = max_hp;
    entity.hit_dice = HitDice {
        sides: die.sides,
        maximum: die.count as u8,
        remaining: die.count as u8,
    };
    entity.uses_death_saves = false;
    entity.resistances = statistics.damage_resistances.iter().copied().collect();
    entity.vulnerabilities = statistics.damage_vulnerabilities.iter().copied().collect();
    entity.damage_immunities = statistics.damage_immunities.iter().copied().collect();
    entity.condition_immunities = statistics.condition_immunities.iter().copied().collect();
    // Explicit source test/attack/spell modifiers are queried through this module.
    // Legacy PC grant/proficiency/slot tables remain empty, so they cannot fabricate them.
    entity.saving_proficiencies.clear();
    entity.skill_proficiencies.clear();
    entity.attacks.clear();
    entity.attack_proficiencies.clear();
    entity.prepared_spells.clear();
    entity.spellcasting = None;
    entity.resources.clear();
    Ok(entity)
}

pub fn validate_creature_profile(
    state: &CampaignState,
    profile: &CreatureProfile,
    entity: &MechanicalEntity,
) -> Result<(), CreatureError> {
    validate_creature_origin(state, &profile.origin, profile.actor).map_err(invalid)?;
    if !matches!(
        profile.origin.issuer,
        CommandIssuer::System | CommandIssuer::Admin
    ) || profile
        .origin
        .actor
        .is_some_and(|actor| actor != AgentRef::Entity(profile.actor))
        || profile.source.ruleset_id != state.campaign.ruleset.id
        || profile.source.ruleset_version != state.campaign.ruleset.version
        || state
            .characters
            .values()
            .any(|character| character.entity_id == profile.actor)
    {
        return Err(invalid("invalid creature profile origin or PC identity"));
    }
    let initial = initial_creature_mechanics(profile)?;
    let source = source_for_profile(profile)?;
    if entity.entity_id != profile.actor
        || entity.level != 0
        || entity.character_features.is_some()
        || entity.ability_scores != initial.ability_scores
        || entity.armor != initial.armor
        || entity.hit_dice.sides != initial.hit_dice.sides
        || entity.hit_dice.maximum != initial.hit_dice.maximum
        || entity.hit_dice.remaining > entity.hit_dice.maximum
        || entity.resistances != initial.resistances
        || entity.vulnerabilities != initial.vulnerabilities
        || entity.damage_immunities != initial.damage_immunities
        || entity.condition_immunities != initial.condition_immunities
        || !entity.saving_proficiencies.is_empty()
        || !entity.skill_proficiencies.is_empty()
        || !entity.attacks.is_empty()
        || !entity.attack_proficiencies.is_empty()
        || !entity.prepared_spells.is_empty()
        || entity.spellcasting.is_some()
        || !entity.resources.is_empty()
        || (source.statistics.exhaustion_immune && entity.exhaustion != 0)
        || entity.max_hp > 1_000_000
        || entity.hp > entity.max_hp
        || entity.temporary_hp > 1_000_000
        || (entity.max_hp == 0 && !entity.death.dead)
    {
        return Err(invalid(
            "current creature intrinsic statistics differ from source",
        ));
    }
    Ok(())
}

/// Exact stat-block modifier (including its explicit skill adjustments), plus the
/// existing exhaustion penalty. Contextual advantage/effects are resolved elsewhere.
pub fn creature_test_modifier(
    profile: &CreatureProfile,
    entity: &MechanicalEntity,
    kind: &TestKind,
) -> Result<i32, CreatureError> {
    if entity.entity_id != profile.actor {
        return Err(invalid("creature test actor differs"));
    }
    let source = source_for_profile(profile)?;
    let stats = &source.statistics;
    let modifier = match kind {
        TestKind::Initiative => i32::from(stats.initiative_modifier),
        TestKind::Save { ability } => i32::from(stats.saving_throw_modifiers[ability.index()]),
        TestKind::Check { ability, skill } => {
            let base = ability_modifier(stats.ability_scores[ability.index()]);
            if let Some(skill) = skill
                && let Some(explicit) = stats.skills.iter().find(|entry| entry.skill == *skill)
            {
                let standard = crate::STANDARD_SKILL_ABILITIES
                    .iter()
                    .find(|(candidate, _)| candidate == skill)
                    .ok_or_else(|| invalid("unknown skill ability"))?
                    .1;
                base + i32::from(explicit.modifier)
                    - ability_modifier(stats.ability_scores[standard.index()])
            } else {
                base
            }
        }
        TestKind::DeathSave => 0,
    };
    Ok(modifier - i32::from(entity.exhaustion) * 2)
}
pub fn creature_proficiency_bonus(profile: &CreatureProfile) -> Result<u8, CreatureError> {
    Ok(source_for_profile(profile)?.statistics.proficiency_bonus)
}
pub fn creature_movement(source: &CreatureDefinition) -> MovementProfile {
    let s = &source.statistics.speeds;
    let units = |feet: u16| u32::from(feet) * 2;
    let optional = |feet: u16| (feet > 0).then(|| units(feet));
    MovementProfile {
        walk: units(s.walk),
        climb: optional(s.climb),
        swim: optional(s.swim),
        fly: optional(s.fly),
        burrow: optional(s.burrow),
        hover: s.hover,
    }
}
pub fn creature_senses(source: &CreatureDefinition) -> Senses {
    let s = &source.statistics.senses;
    let units = |feet: u16| u32::from(feet) * 2;
    Senses {
        darkvision: units(s.darkvision),
        blindsight: units(s.blindsight),
        tremorsense: units(s.tremorsense),
        truesight: units(s.truesight),
    }
}
pub fn source_size(size: SourceSize) -> CreatureSize {
    match size {
        SourceSize::Tiny => CreatureSize::Tiny,
        SourceSize::Small => CreatureSize::Small,
        SourceSize::Medium => CreatureSize::Medium,
        SourceSize::Large => CreatureSize::Large,
        SourceSize::Huge => CreatureSize::Huge,
        SourceSize::Gargantuan => CreatureSize::Gargantuan,
    }
}
pub(super) fn initial_recharge(source: &CreatureDefinition) -> Vec<CreatureRechargeState> {
    source
        .features
        .iter()
        .filter(|feature| matches!(feature.feature, MonsterFeature::SaveArea { .. }))
        .map(|feature| CreatureRechargeState {
            feature_id: feature.id.clone(),
            available: true,
            pending: None,
            last_roll: None,
        })
        .collect()
}
pub(super) fn initial_limited_uses(source: &CreatureDefinition) -> Vec<CreatureLimitedUse> {
    let mut uses = vec![];
    for feature in &source.features {
        if let MonsterFeature::Spellcasting { spells, .. } = &feature.feature {
            for spell in spells
                .iter()
                .filter(|spell| spell.uses_per_long_rest.is_some())
            {
                uses.push(CreatureLimitedUse {
                    feature_id: feature.id.clone(),
                    spell_id: Some(spell.spell_id.clone()),
                    spent: 0,
                });
            }
        }
    }
    uses
}
