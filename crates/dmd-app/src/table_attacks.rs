//! Read-only physical choices. The accepted action revalidates every source and fact.
use dmd_domain::*;
use dmd_rules::tactical_definitions::{
    TACTICAL_DEFINITIONS_JSON, TacticalDefinitions, WeaponDefinition, WeaponHands, WeaponKind,
    WeaponMastery, WeaponProperty,
};

pub(super) fn options(
    state: &CampaignState,
    actor: EntityId,
) -> Result<Option<crate::TableAttackOptions>, String> {
    let Some(encounter) = &state.encounter else {
        return Ok(None);
    };
    let Some(loadout) = state
        .rules
        .as_ref()
        .and_then(|rules| rules.tactical_inventory.as_ref())
        .and_then(|inventory| inventory.loadout(actor))
    else {
        return Ok(None);
    };
    let definitions = TacticalDefinitions::from_json(TACTICAL_DEFINITIONS_JSON)
        .map_err(|error| error.to_string())?;
    let usable = |item: &&ItemInstance| {
        item.custody == Custody::Entity(actor)
            && item.state == ItemState::Intact
            && item.quantity > 0
    };
    let mut weapons = Vec::new();
    for item in state.items.values().filter(usable) {
        let Some(weapon) = definitions.weapon(&item.definition_id) else {
            continue;
        };
        let deliveries = match weapon.kind {
            WeaponKind::Melee => {
                let mut choices = vec![WeaponDelivery::Melee];
                if weapon.properties.contains(&WeaponProperty::Thrown) {
                    choices.push(WeaponDelivery::Thrown);
                }
                choices
            }
            WeaponKind::Ranged if weapon.ammunition.is_some() => vec![WeaponDelivery::Shot],
            WeaponKind::Ranged => vec![WeaponDelivery::Thrown],
        };
        let abilities = if weapon.properties.contains(&WeaponProperty::Finesse) {
            vec![Ability::Strength, Ability::Dexterity]
        } else {
            vec![match weapon.kind {
                WeaponKind::Melee => Ability::Strength,
                WeaponKind::Ranged => Ability::Dexterity,
            }]
        };
        let mut grips = match weapon.hands {
            WeaponHands::One => vec![
                WeaponGrip::OneHand(Hand::Left),
                WeaponGrip::OneHand(Hand::Right),
            ],
            // The current source resolver has no mounted action authority yet.
            WeaponHands::Two | WeaponHands::TwoUnlessMounted => vec![WeaponGrip::TwoHands],
        };
        if weapon.versatile_damage.is_some() {
            grips.push(WeaponGrip::TwoHands);
        }
        let ammunition_id = dmd_rules::tactical_weapons::required_ammunition_definition(weapon);
        let mut ammunition = state
            .items
            .values()
            .filter(usable)
            .filter(|candidate| Some(candidate.definition_id.as_str()) == ammunition_id)
            .map(|candidate| crate::TableItemView {
                id: candidate.id,
                name: dmd_rules::tactical_inventory::equipment_definition(&candidate.definition_id)
                    .map(|source| source.display_name.clone())
                    .unwrap_or_else(|_| "Ammunition".into()),
                quantity: candidate.quantity,
            })
            .collect::<Vec<_>>();
        ammunition.sort_by_key(|stack| stack.id.0);
        weapons.push(crate::TableWeaponChoice {
            item: item.id,
            name: weapon.name.clone(),
            deliveries,
            abilities,
            grips,
            purposes: purposes(state, actor, item.id, weapon, &definitions),
            ammunition_required: ammunition_id.is_some(),
            ammunition,
            source_features: source_features(state, actor, item)?,
        });
    }
    weapons.sort_by(|a, b| a.name.cmp(&b.name).then(a.item.0.cmp(&b.item.0)));
    // Target knowledge belongs to the acting creature, even if one human controls
    // several observers. The host's full map is not this actor's perception.
    let observed = dmd_rules::spatial::project_actor_view(encounter, state, actor)
        .map_err(|error| error.to_string())?;
    let targets = observed
        .contacts
        .into_iter()
        .filter(|contact| {
            contact.entity_id != actor
                && contact.status != dmd_rules::spatial::ContactStatus::Remembered
        })
        .map(|contact| crate::TableAttackTarget {
            actor: contact.entity_id,
            label: contact.label.unwrap_or_else(|| "Located creature".into()),
        })
        .collect();
    Ok(Some(crate::TableAttackOptions {
        actor,
        hands: loadout.hands.clone(),
        weapons,
        targets,
    }))
}

fn source_features(
    state: &CampaignState,
    actor: EntityId,
    item: &ItemInstance,
) -> Result<Vec<crate::TableCreatureAttackChoice>, String> {
    use dmd_rules::tactical_definitions::{FeatureActivation, MonsterFeature};
    let Some(rules) = &state.rules else {
        return Ok(vec![]);
    };
    if rules
        .timing
        .as_ref()
        .is_none_or(|timing| timing.action_spent)
        || !dmd_rules::tactical_conditions::can_act(rules, actor).map_err(|e| e.to_string())?
    {
        return Ok(vec![]);
    }
    let Some(profile) = rules
        .tactical_creatures
        .as_ref()
        .and_then(|c| c.profile(actor))
    else {
        return Ok(vec![]);
    };
    let source =
        dmd_rules::tactical_creatures::source_for_profile(profile).map_err(|e| e.to_string())?;
    let mut choices = Vec::new();
    for feature in &source.features {
        if feature.activation != FeatureActivation::Action || feature.usage.is_some() {
            continue;
        }
        let MonsterFeature::Attack {
            damage,
            conditional_hits,
            ..
        } = &feature.feature
        else {
            continue;
        };
        if damage.len() != 1 || !conditional_hits.is_empty() {
            continue;
        }
        let required =
            dmd_rules::tactical_creature_equipment::creature_attack_gear(profile, &feature.id)
                .map_err(|e| e.to_string())?;
        if required == Some(item.definition_id.as_str()) {
            choices.push(crate::TableCreatureAttackChoice {
                feature_id: feature.id.clone(),
                label: feature.name.clone(),
                weapon: Some(item.id),
            });
        }
    }
    choices.sort_by(|a, b| a.feature_id.cmp(&b.feature_id));
    Ok(choices)
}

/// Read-only opportunities from accepted history, never a new attack allowance.
fn purposes(
    state: &CampaignState,
    actor: EntityId,
    item: ItemId,
    weapon: &WeaponDefinition,
    definitions: &TacticalDefinitions,
) -> Vec<WeaponAttackPurpose> {
    let Some(flow) = state
        .encounter
        .as_ref()
        .and_then(|encounter| encounter.flow.as_ref())
    else {
        return vec![];
    };
    let Some(timing) = state.rules.as_ref().and_then(|rules| rules.timing.as_ref()) else {
        return vec![];
    };
    let budget = &flow.budget;
    let mut choices = vec![];
    if !timing.action_spent || budget.attacks_remaining > 0 {
        choices.push(WeaponAttackPurpose::Normal);
    }
    if !weapon.properties.contains(&WeaponProperty::Light)
        || budget.weapon_history.iter().any(|prior| {
            prior.actor == actor
                && matches!(
                    prior.purpose,
                    WeaponAttackPurpose::LightBonus { .. } | WeaponAttackPurpose::Nick { .. }
                )
        })
    {
        return choices;
    }
    let triggers = budget.weapon_history.iter().filter(|prior| {
        prior.actor == actor
            && prior.turn_number == timing.turn_number
            && prior.on_actor_turn
            && prior.window.kind == WeaponActionKind::AttackAction
            && prior.purpose == WeaponAttackPurpose::Normal
            && prior.outcome != WeaponAttackOutcome::Pending
            && prior.weapon != item
            && definitions
                .weapon(&prior.definition_id)
                .is_some_and(|source| source.properties.contains(&WeaponProperty::Light))
    });
    if let Some(prior) = triggers
        .clone()
        .next()
        .filter(|_| !timing.bonus_action_spent)
    {
        choices.push(WeaponAttackPurpose::LightBonus {
            trigger: prior.origin.id,
        });
    }
    let nick = weapon.mastery == WeaponMastery::Nick
        && state
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
                    .features
                    .iter()
                    .any(|feature| feature.id == "weapon-mastery")
                    && profile.masteries.contains(&weapon.id)
            });
    if let Some(prior) = triggers
        .clone()
        .find(|prior| Some(prior.window) == budget.attack_window)
        .filter(|_| nick)
    {
        choices.push(WeaponAttackPurpose::Nick {
            trigger: prior.origin.id,
        });
    }
    choices
}
