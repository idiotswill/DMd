//! The supported SRD 5.2.1 creation profile; choices become validated grants, never caller totals.
use crate::{RulesError, RulesPack, ability_modifier};
use dmd_domain::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CharacterCreationInput {
    pub name: String,
    pub pronouns: String,
    pub description: String,
    pub alignment: String,
    pub backstory: String,
    pub ability_scores: [u8; 6],
    pub background_boosts: [u8; 6],
    pub fighter_skills: [Skill; 2],
    pub human_skill: Skill,
    pub skilled_skills: [Skill; 3],
    pub size: CharacterSize,
    pub languages: [String; 2],
    pub fighting_style: FightingStyle,
    pub gaming_set: GamingSet,
    pub purchases: Vec<EquipmentChoice>,
    pub worn_armor: Option<String>,
    pub shield: bool,
    pub masteries: [String; 3],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuiltCharacter {
    pub mechanics: MechanicalEntity,
    pub profile: CharacterProfile,
    /// Identical to the profile's starting equipment; callers can instantiate inventory identities.
    pub equipment: Vec<CharacterEquipment>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StarterItem {
    pub id: String,
    pub name: String,
    pub unit_cost_cp: u32,
    pub purchase_multiple: u16,
    pub source_page: u16,
    pub weapon: bool,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StarterCatalog {
    pub schema_version: u32,
    pub ruleset_id: String,
    pub version: String,
    pub profile_id: String,
    pub starting_money_cp: u32,
    pub source_pages: Vec<u16>,
    pub scope: String,
    pub items: Vec<StarterItem>,
}
pub fn starter_catalog() -> StarterCatalog {
    // Embedded bytes and distributed bytes are the same tracked, integrity-declared source asset.
    serde_json::from_str(include_str!(
        "../../../content/srd-5.2.1/character-creation.json"
    ))
    .expect("source-reviewed starter catalog is valid JSON")
}
/// Fighter level-one selections (SRD48), independent of owned/starting equipment.
pub fn fighter_mastery_choices() -> Result<Vec<String>, RulesError> {
    let catalog = crate::tactical_definitions::TacticalDefinitions::from_json(
        crate::tactical_definitions::TACTICAL_DEFINITIONS_JSON,
    )
    .map_err(|error| RulesError::Invalid(error.to_string()))?;
    let mut choices: Vec<_> = catalog
        .weapons
        .into_iter()
        .map(|weapon| weapon.id)
        .collect();
    choices.sort_unstable();
    Ok(choices)
}
pub const FIGHTER_SKILLS: [Skill; 9] = [
    Skill::Acrobatics,
    Skill::AnimalHandling,
    Skill::Athletics,
    Skill::History,
    Skill::Insight,
    Skill::Intimidation,
    Skill::Persuasion,
    Skill::Perception,
    Skill::Survival,
];
pub const STANDARD_LANGUAGES: [&str; 9] = [
    "common-sign-language",
    "draconic",
    "dwarvish",
    "elvish",
    "giant",
    "gnomish",
    "goblin",
    "halfling",
    "orc",
];
pub const ALIGNMENTS: [&str; 9] = [
    "Lawful Good",
    "Neutral Good",
    "Chaotic Good",
    "Lawful Neutral",
    "Neutral",
    "Chaotic Neutral",
    "Lawful Evil",
    "Neutral Evil",
    "Chaotic Evil",
];
/// Reconstruct all immutable source grants/prices from the profile's retained choices.
pub fn validate_character_profile(
    profile: &CharacterProfile,
    pack: &RulesPack,
) -> Result<(), RulesError> {
    character_from_profile(profile, pack).map(|_| ())
}

/// Check retained source grants while allowing only the sheet's mutable play state.
/// New equipment/advancement rules must explicitly extend this boundary when supported.
pub fn validate_character_mechanics(
    profile: &CharacterProfile,
    entity: &MechanicalEntity,
    pack: &RulesPack,
) -> Result<(), RulesError> {
    validate_character_components(profile, entity, pack, false)
}

/// Validate immutable source grants without treating starting armor as live inventory.
/// The caller must independently derive and validate `armor` and `wearing_armor` from
/// current equipment before using AC. Legacy kernel attack registrations remain source
/// creation data; the tactical weapon planner derives weapon availability from ItemIds.
pub fn validate_character_intrinsics(
    profile: &CharacterProfile,
    entity: &MechanicalEntity,
    pack: &RulesPack,
) -> Result<(), RulesError> {
    validate_character_components(profile, entity, pack, true)
}

fn validate_character_components(
    profile: &CharacterProfile,
    entity: &MechanicalEntity,
    pack: &RulesPack,
    live_equipment: bool,
) -> Result<(), RulesError> {
    let mut expected = character_from_profile(profile, pack)?.mechanics;
    if live_equipment {
        expected.armor = entity.armor.clone();
    }
    expected.hp = entity.hp;
    expected.temporary_hp = entity.temporary_hp;
    expected.hit_dice.remaining = entity.hit_dice.remaining;
    expected.death = entity.death.clone();
    expected.exhaustion = entity.exhaustion;
    expected.heroic_inspiration = entity.heroic_inspiration;
    expected.prone = entity.prone;
    expected.concentration = entity.concentration;
    expected.last_long_rest_finished = entity.last_long_rest_finished;
    let actual_features = entity
        .character_features
        .as_ref()
        .ok_or_else(|| invalid("character is missing its source feature grants"))?;
    let features = expected
        .character_features
        .as_mut()
        .expect("source features");
    features.second_wind_remaining = actual_features.second_wind_remaining;
    features.inspiration_transfer_pending = actual_features.inspiration_transfer_pending;
    features.savage_attacker_turn = actual_features.savage_attacker_turn;
    if live_equipment {
        features.wearing_armor = actual_features.wearing_armor;
    }
    if expected != *entity {
        return Err(invalid(
            "character source choices disagree with its mechanical sheet",
        ));
    }
    Ok(())
}

fn character_from_profile(
    profile: &CharacterProfile,
    pack: &RulesPack,
) -> Result<BuiltCharacter, RulesError> {
    if profile.languages.len() != 3
        || profile.languages[0] != "common"
        || profile.tool_proficiencies.len() != 1
    {
        return Err(invalid("invalid profile languages or tool grants"));
    }
    let gaming_set = [
        GamingSet::Dice,
        GamingSet::Dragonchess,
        GamingSet::PlayingCards,
        GamingSet::ThreeDragonAnte,
    ]
    .into_iter()
    .find(|g| g.id() == profile.tool_proficiencies[0])
    .ok_or_else(|| invalid("invalid Soldier gaming-set grant"))?;
    let input = CharacterCreationInput {
        name: profile.name.clone(),
        pronouns: profile.pronouns.clone(),
        description: profile.description.clone(),
        alignment: profile.alignment.clone(),
        backstory: profile.backstory.clone(),
        ability_scores: profile.base_ability_scores,
        background_boosts: profile.background_boosts,
        fighter_skills: profile.fighter_skills,
        human_skill: profile.human_skill,
        skilled_skills: profile.skilled_skills,
        size: profile.size,
        languages: [profile.languages[1].clone(), profile.languages[2].clone()],
        fighting_style: profile.fighting_style,
        gaming_set,
        purchases: profile
            .equipment
            .iter()
            .map(|e| EquipmentChoice {
                item_id: e.item_id.clone(),
                quantity: e.quantity,
            })
            .collect(),
        worn_armor: profile.worn_armor.clone(),
        shield: profile.shield,
        masteries: profile.masteries.clone(),
    };
    let built = build_character(&input, profile.entity_id, pack)?;
    if built.profile != *profile {
        return Err(invalid(
            "profile derived grants/equipment differ from source creation",
        ));
    }
    Ok(built)
}
fn invalid(message: &str) -> RulesError {
    RulesError::Invalid(message.into())
}

pub fn build_character(
    input: &CharacterCreationInput,
    entity_id: EntityId,
    pack: &RulesPack,
) -> Result<BuiltCharacter, RulesError> {
    pack.validate()?;
    if !ALIGNMENTS.contains(&input.alignment.as_str()) {
        return Err(invalid("choose a source alignment"));
    }
    for (value, maximum, required) in [
        (&input.name, 100, true),
        (&input.pronouns, 100, false),
        (&input.description, 2000, false),
        (&input.alignment, 100, true),
        (&input.backstory, 8000, false),
    ] {
        if value.len() > maximum || (required && value.trim().is_empty()) || value.contains('\0') {
            return Err(invalid("character text is empty, oversized or invalid"));
        }
    }
    let mut scores = input.ability_scores;
    scores.sort();
    if scores != [8, 10, 12, 13, 14, 15] {
        return Err(invalid("assign the exact standard array"));
    }
    let boosts = input.background_boosts;
    let mut physical = [boosts[0], boosts[1], boosts[2]];
    physical.sort();
    if boosts[3..] != [0, 0, 0] || (physical != [0, 1, 2] && physical != [1, 1, 1]) {
        return Err(invalid(
            "Soldier boosts must be +2/+1 or +1 each among Strength, Dexterity, Constitution",
        ));
    }
    if input
        .fighter_skills
        .iter()
        .any(|s| !FIGHTER_SKILLS.contains(s))
    {
        return Err(invalid("Fighter skill choice is outside the class list"));
    }
    let skills: Vec<_> = [Skill::Athletics, Skill::Intimidation]
        .into_iter()
        .chain(input.fighter_skills)
        .chain([input.human_skill])
        .chain(input.skilled_skills)
        .collect();
    let unique: BTreeSet<_> = skills.iter().copied().collect();
    if unique.len() != skills.len() {
        return Err(invalid(
            "choose distinct new skill proficiencies; bonuses never stack",
        ));
    }
    if input.languages[0] == input.languages[1]
        || input
            .languages
            .iter()
            .any(|s| !STANDARD_LANGUAGES.contains(&s.as_str()))
    {
        return Err(invalid(
            "choose two distinct standard languages in addition to Common",
        ));
    }
    let masteries: BTreeSet<_> = input.masteries.iter().map(String::as_str).collect();
    let choices = fighter_mastery_choices()?;
    if masteries.len() != 3
        || masteries
            .iter()
            .any(|id| !choices.iter().any(|choice| choice == id))
    {
        return Err(invalid(
            "choose three distinct Simple or Martial weapon mastery grants",
        ));
    }
    let catalog = starter_catalog();
    let mut purchased = BTreeMap::new();
    let mut equipment = Vec::new();
    let mut spent = 0u32;
    for choice in &input.purchases {
        let item = catalog
            .items
            .iter()
            .find(|i| i.id == choice.item_id)
            .ok_or_else(|| invalid("item is not in the supported source catalog"))?;
        if choice.quantity == 0
            || choice.quantity > 1000
            || choice.quantity % item.purchase_multiple != 0
            || purchased.insert(item.id.clone(), choice.quantity).is_some()
        {
            return Err(invalid(
                "invalid, duplicated or non-package equipment quantity",
            ));
        }
        spent = spent
            .checked_add(item.unit_cost_cp * u32::from(choice.quantity))
            .ok_or_else(|| invalid("equipment total overflow"))?;
        equipment.push(CharacterEquipment {
            item_id: item.id.clone(),
            display_name: item.name.clone(),
            quantity: choice.quantity,
            unit_cost_cp: item.unit_cost_cp,
            source_page: item.source_page,
        });
    }
    let money_cp = catalog
        .starting_money_cp
        .checked_sub(spent)
        .ok_or_else(|| invalid("purchases exceed the source starting gold"))?;
    if input
        .worn_armor
        .as_ref()
        .is_some_and(|a| a != "leather-armor" || !purchased.contains_key(a))
        || (input.shield && !purchased.contains_key("shield"))
    {
        return Err(invalid("equipped armor or shield is not owned/supported"));
    }
    equipment.sort_by(|a, b| a.item_id.cmp(&b.item_id));
    let mut mechanics = MechanicalEntity::basic(entity_id);
    for (index, score) in input.ability_scores.iter().enumerate() {
        mechanics.ability_scores[index] = score + boosts[index];
    }
    mechanics.max_hp = (10 + ability_modifier(mechanics.ability_scores[2])) as u32;
    mechanics.hp = mechanics.max_hp;
    mechanics.hit_dice = HitDice {
        sides: 10,
        maximum: 1,
        remaining: 1,
    };
    mechanics.saving_proficiencies = BTreeSet::from([Ability::Strength, Ability::Constitution]);
    mechanics.skill_proficiencies = unique
        .into_iter()
        .map(|s| (s, Proficiency::Proficient))
        .collect();
    mechanics.armor = ArmorClass::Armor {
        base: if input.worn_armor.is_some() { 11 } else { 10 },
        dexterity_cap: None,
        shield: input.shield,
    };
    mechanics.attacks = purchased
        .keys()
        .filter(|id| catalog.items.iter().any(|i| &i.id == *id && i.weapon))
        .cloned()
        .collect();
    mechanics.attack_proficiencies = mechanics.attacks.clone();
    for id in &mechanics.attacks {
        pack.attack(id)?;
    }
    mechanics.character_features = Some(CharacterFeatureState {
        fighter_level: 1,
        fighting_style: input.fighting_style,
        wearing_armor: input.worn_armor.is_some(),
        human_resourceful: true,
        savage_attacker: true,
        second_wind_remaining: 2,
        inspiration_transfer_pending: false,
        savage_attacker_turn: None,
    });
    let features = [
        ("fighting-style", 47, 3, "Defense/Archery bonuses"),
        (
            "second-wind",
            48,
            3,
            "Raw healing die and partial/full rest recovery",
        ),
        (
            "weapon-mastery",
            48,
            4,
            "Three grants; mastery execution remains Gate 4",
        ),
        (
            "resourceful",
            86,
            3,
            "Long-rest Inspiration and optional controller-selected transfer",
        ),
        ("skillful", 86, 3, "One selected skill proficiency"),
        (
            "versatile",
            86,
            3,
            "Skilled is the supported extra Origin feat",
        ),
        (
            "skilled",
            87,
            3,
            "Three selected skill proficiencies; tool choices remain a later extension",
        ),
        (
            "savage-attacker",
            87,
            3,
            "Once-per-turn weapon damage dice choice",
        ),
    ]
    .into_iter()
    .map(
        |(id, source_page, execution_gate, scope)| CharacterFeatureGrant {
            id: id.into(),
            source_page,
            execution_gate,
            scope: scope.into(),
        },
    )
    .collect();
    let profile = CharacterProfile {
        entity_id,
        name: input.name.trim().into(),
        pronouns: input.pronouns.clone(),
        description: input.description.clone(),
        alignment: input.alignment.clone(),
        backstory: input.backstory.clone(),
        class_id: "fighter".into(),
        species_id: "human".into(),
        background_id: "soldier".into(),
        level: 1,
        experience_points: 0,
        size: input.size,
        speed_feet: 30,
        base_ability_scores: input.ability_scores,
        background_boosts: boosts,
        fighter_skills: input.fighter_skills,
        human_skill: input.human_skill,
        skilled_skills: input.skilled_skills,
        languages: std::iter::once("common".into())
            .chain(input.languages.clone())
            .collect(),
        fighting_style: input.fighting_style,
        features,
        masteries: input.masteries.clone(),
        tool_proficiencies: vec![input.gaming_set.id().into()],
        armor_training: vec![
            "light".into(),
            "medium".into(),
            "heavy".into(),
            "shield".into(),
        ],
        weapon_proficiencies: vec!["simple".into(), "martial".into()],
        equipment: equipment.clone(),
        worn_armor: input.worn_armor.clone(),
        shield: input.shield,
        money_cp,
    };
    Ok(BuiltCharacter {
        mechanics,
        profile,
        equipment,
    })
}
