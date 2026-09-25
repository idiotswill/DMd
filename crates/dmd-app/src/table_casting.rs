//! Private source choices, never a casting permission. Every submitted choice is
//! independently admitted by the same tactical reducer at its expected head.
use dmd_domain::*;
use dmd_rules::tactical_creatures::{
    CreatureActionCost, CreatureScheduleOperation, apply_creature_schedule, creature_definitions,
    source_for_profile,
};
use dmd_rules::tactical_definitions::MonsterFeature;
use dmd_rules::tactical_spells::*;

use crate::{TableAttackTarget, TableCastingOptions, TableCastingVariant};

pub(super) fn options(
    state: &CampaignState,
    actor: EntityId,
) -> Result<Option<TableCastingOptions>, String> {
    let Some(rules) = &state.rules else {
        return Ok(None);
    };
    let Some(entity) = rules.entities.get(&actor) else {
        return Ok(None);
    };
    let Some(encounter) = &state.encounter else {
        return Ok(None);
    };
    let definitions = creature_definitions().map_err(|error| error.to_string())?;
    let mut grants = Vec::new();
    for id in &entity.prepared_spells {
        let Some(spell) = definitions.spell(id) else {
            continue;
        };
        let resources = if spell.level == 0 {
            vec![SpellResourceChoice::Cantrip]
        } else {
            entity
                .spellcasting
                .as_ref()
                .map(|casting| {
                    (spell.level..=9)
                        .filter(|level| casting.slots[usize::from(*level - 1)] > 0)
                        .map(|level| SpellResourceChoice::Slot { level })
                        .collect()
                })
                .unwrap_or_default()
        };
        grants.push((id.clone(), SpellGrantChoice::Prepared, resources));
    }
    if let Some(profile) = rules
        .tactical_creatures
        .as_ref()
        .and_then(|creatures| creatures.profile(actor))
    {
        let source = source_for_profile(profile).map_err(|error| error.to_string())?;
        for feature in &source.features {
            if let MonsterFeature::Spellcasting { spells, .. } = &feature.feature {
                for spell in spells {
                    grants.push((
                        spell.spell_id.clone(),
                        SpellGrantChoice::CreatureFeature {
                            feature_id: feature.id.clone(),
                        },
                        vec![SpellResourceChoice::SourceFeature],
                    ));
                }
            }
        }
    }
    if grants.is_empty() {
        return Ok(None);
    }
    grants.sort_by(|a, b| a.0.cmp(&b.0));
    // This deterministic read-only identifier is never returned to the client,
    // journaled, or treated as accepted authority. Actual action IDs come from
    // the stable table request envelope. Planning never mutates the input state.
    let meta = CommandMeta {
        id: CommandId(spell_concentration_id(CommandId(encounter.id.0), actor, 0).0),
        campaign_id: state.campaign_id(),
        session_id: state
            .table
            .as_ref()
            .and_then(|table| table.active_session.as_ref())
            .map(|session| session.session_id),
        issuer: CommandIssuer::Admin,
        actor: Some(AgentRef::Entity(actor)),
        expected_event_sequence: state.applied_event_sequence,
    };
    let observed = dmd_rules::spatial::project_actor_view(encounter, state, actor)
        .map_err(|error| error.to_string())?;
    let mut targets = observed
        .contacts
        .into_iter()
        .filter(|contact| {
            contact.entity_id != actor
                && contact.status != dmd_rules::spatial::ContactStatus::Remembered
        })
        .map(|contact| TableAttackTarget {
            actor: contact.entity_id,
            label: contact.label.unwrap_or_else(|| "Located creature".into()),
        })
        .collect::<Vec<_>>();
    targets.push(TableAttackTarget {
        actor,
        label: "Self".into(),
    });
    targets.sort_by(|a, b| a.label.cmp(&b.label).then(a.actor.0.cmp(&b.actor.0)));
    let mut result = TableCastingOptions {
        actor,
        variants: vec![],
        unavailable: vec![],
    };
    for (spell_id, grant, resources) in grants {
        let spell = definitions
            .spell(&spell_id)
            .ok_or("Owned spell source is absent.")?;
        let before = result.variants.len();
        let mut unsupported = false;
        let mut materials = vec![SpellMaterialChoice::None];
        let mut items = state
            .items
            .values()
            .filter(|item| {
                item.campaign_id == state.campaign_id()
                    && item.custody == Custody::Entity(actor)
                    && item.state == ItemState::Intact
                    && item.quantity > 0
            })
            .collect::<Vec<_>>();
        items.sort_by_key(|item| item.id.0);
        for item in items {
            if dmd_rules::tactical_creature_equipment::source_spell_material(item)
                .map_err(|error| error.to_string())?
                .is_some_and(|material| material.spell_id == spell_id)
            {
                materials.push(SpellMaterialChoice::Material { item: item.id });
            }
        }
        for resource in resources {
            for material in &materials {
                let choice = SpellCastChoice {
                    actor,
                    spell_id: spell_id.clone(),
                    grant: grant.clone(),
                    resource,
                    material: *material,
                    mode: SpellCastMode::Immediate,
                };
                let Ok(plan) = preview_plan(state, &meta, &choice) else {
                    continue;
                };
                if executable_spell_kind(&plan).is_err() {
                    unsupported = true;
                    continue;
                }
                if validate_spell_slot_reservation(state, &plan, &[]).is_err() {
                    continue;
                }
                let (minimum_targets, maximum_targets, repeated_targets) =
                    match plan.program.targets {
                        SpellTargetRule::CreatureOrObject => (1, 1, false),
                        SpellTargetRule::Creatures { maximum, .. } => (1, maximum, false),
                        SpellTargetRule::Rays { count } | SpellTargetRule::Darts { count, .. } => {
                            (count, count, true)
                        }
                        _ => {
                            unsupported = true;
                            continue;
                        }
                    };
                let eligible = targets
                    .iter()
                    .filter(|target| {
                        let selection =
                            SpellTargetChoice::Entities(vec![
                                target.actor;
                                usize::from(minimum_targets)
                            ]);
                        // Type eligibility remains private. A legal wrong-type selection
                        // is still offered and later pays/resolves according to source.
                        bind_spell(state, &plan, &selection)
                            .is_ok_and(|bound| bound.consumed_material().is_none())
                    })
                    .cloned()
                    .collect::<Vec<_>>();
                if eligible.is_empty() {
                    continue;
                }
                let resource_label = match resource {
                    SpellResourceChoice::Cantrip => "cantrip".to_owned(),
                    SpellResourceChoice::Slot { level } => format!("level {level} slot"),
                    SpellResourceChoice::SourceFeature => "source ability".to_owned(),
                };
                let material_label = match material {
                    SpellMaterialChoice::Material { item } => state
                        .items
                        .get(item)
                        .map(|item| format!(", {}", item.display_name))
                        .unwrap_or_default(),
                    _ => String::new(),
                };
                result.variants.push(TableCastingVariant {
                    choice,
                    label: format!("{} — {resource_label}{material_label}", spell.name),
                    concentration: plan.program.concentration,
                    minimum_targets,
                    maximum_targets,
                    repeated_targets,
                    targets: eligible,
                });
            }
        }
        if result.variants.len() == before {
            result.unavailable.push(if unsupported {
                format!("{}: this spell cannot be cast here yet.", spell.name)
            } else {
                format!("{}: no legal cast is available. Check remaining actions and uses, free hands, armor training, specified materials and targets within range.", spell.name)
            });
        }
    }
    // A source can expose the same spell through Action and Legendary features.
    // An unavailable alternate activation must not contradict a usable variant.
    result.unavailable.retain(|reason| {
        !result.variants.iter().any(|variant| {
            definitions
                .spell(&variant.choice.spell_id)
                .is_some_and(|spell| reason.starts_with(&format!("{}:", spell.name)))
        })
    });
    result.unavailable.sort();
    result.unavailable.dedup();
    Ok(Some(result))
}

fn preview_plan(
    state: &CampaignState,
    meta: &CommandMeta,
    choice: &SpellCastChoice,
) -> Result<SpellCastPlan, dmd_rules::RulesError> {
    let (plan, cost) = match &choice.grant {
        SpellGrantChoice::Prepared => {
            let plan = plan_spell_cast_at(state, meta, choice, 0)?;
            let cost = plan.cost;
            (plan, cost)
        }
        SpellGrantChoice::CreatureFeature { feature_id } => {
            let current = state
                .rules
                .as_ref()
                .and_then(|rules| rules.tactical_creatures.as_ref())
                .ok_or(dmd_rules::RulesError::Uninitialized)?;
            let step = apply_creature_schedule(
                state,
                current,
                meta,
                &CreatureScheduleOperation::BeginFeature {
                    actor: choice.actor,
                    selection: CreatureFeatureSelection {
                        feature_id: feature_id.clone(),
                        spell_id: Some(choice.spell_id.clone()),
                        simple_action: None,
                    },
                    steps: vec![],
                },
            )
            .map_err(|error| dmd_rules::RulesError::Invalid(error.to_string()))?;
            let cost = match step.cost {
                CreatureActionCost::Action => SpellCastingCost::Action,
                CreatureActionCost::BonusAction => SpellCastingCost::BonusAction,
                _ => {
                    return Err(dmd_rules::RulesError::Prerequisite(
                        "Spell needs its enclosing activation.".into(),
                    ));
                }
            };
            let feature = step.feature.as_ref().ok_or_else(|| {
                dmd_rules::RulesError::Invalid("Source feature is not a spell.".into())
            })?;
            (
                plan_spell_from_feature(
                    state,
                    feature,
                    choice.material,
                    SpellCastMode::Immediate,
                    0,
                )?,
                cost,
            )
        }
    };
    // This helper applies the exact authoritative action-budget rule to a clone.
    // Source counters above also remain in the discarded preview transition.
    apply_spell_casting_cost(state, choice.actor, cost)?;
    Ok(plan)
}
