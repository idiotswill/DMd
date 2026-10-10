//! Atomic source NPC construction, shared by setup UI and authoritative table replay.
use dmd_domain::*;
use dmd_rules::tactical_creatures::*;

pub(crate) fn view(
    state: &CampaignState,
    host: bool,
) -> Result<Option<crate::TableCreatureSetupView>, String> {
    if !host {
        return Ok(None);
    }
    // Immutable presentation v1 wire image, captured from genuine f9602a8 history.
    // Current creation affordances belong to table_creature_options, not old digests.
    let catalog = serde_json::from_str(include_str!("table_creature_catalog_v1.json"))
        .map_err(|e| e.to_string())?;
    let mut creatures = Vec::new();
    if let Some(rules) = &state.rules
        && let Some(profiles) = &rules.tactical_creatures
    {
        for profile in &profiles.profiles {
            let world = state
                .entities
                .get(&profile.actor)
                .ok_or("Creature entity is absent.")?;
            let entity = rules
                .entities
                .get(&profile.actor)
                .ok_or("Creature mechanics are absent.")?;
            creatures.push(crate::TableCreatureView {
                actor: profile.actor,
                name: world.display_name.clone(),
                definition_id: profile.source.definition_id.clone(),
                size: profile.size,
                hp: entity.hp,
                max_hp: entity.max_hp,
            });
        }
    }
    creatures.sort_by_key(|creature| creature.actor.0);
    Ok(Some(crate::TableCreatureSetupView { catalog, creatures }))
}

pub(crate) fn current_catalog() -> Result<Vec<crate::TableCreatureOption>, String> {
    let definitions = creature_definitions().map_err(|e| e.to_string())?;
    current_creature_sources()
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(|source| {
            let pin = creature_source_pin(source).map_err(|e| e.to_string())?;
            let ammunition_required = source.statistics.gear.iter().any(|id| {
                definitions
                    .weapon(id)
                    .is_some_and(|weapon| weapon.ammunition.is_some())
            });
            let plan = dmd_rules::tactical_creature_equipment::creature_equipment_plan_from_source(
                &pin,
                if ammunition_required { 1 } else { 0 },
            )
            .map_err(|e| e.to_string())?;
            Ok(crate::TableCreatureOption {
                definition_id: source.id.clone(),
                source: Some(pin),
                name: source.name.clone(),
                sizes: source
                    .statistics
                    .allowed_sizes
                    .iter()
                    .map(|size| source_size(*size))
                    .collect(),
                additional_languages: source.statistics.additional_languages,
                ammunition_required,
                item_count: plan.len(),
                abilities: source
                    .features
                    .iter()
                    .map(|feature| feature.name.clone())
                    .collect(),
                omitted_features: match &source.coverage {
                    dmd_rules::tactical_definitions::DefinitionCoverage::CompleteStatBlock => {
                        vec![]
                    }
                    dmd_rules::tactical_definitions::DefinitionCoverage::SelectedFeatures {
                        omitted,
                    } => omitted.clone(),
                },
                execution_limits: creature_execution_limits(source),
            })
        })
        .collect::<Result<Vec<_>, String>>()
}

#[cfg(test)]
mod ogre_tests;

#[cfg(test)]
mod source_wire_tests {
    use super::*;
    #[test]
    fn historical_picker_and_absent_creation_pin_keep_their_wire_shape() {
        let original = include_str!("table_creature_catalog_v1.json");
        let options: Vec<crate::TableCreatureOption> = serde_json::from_str(original).unwrap();
        assert!(
            options
                .iter()
                .all(|o| o.source.is_none() && o.execution_limits.is_empty())
        );
        assert_eq!(
            serde_json::to_string_pretty(&options).unwrap(),
            original.replace("\r\n", "\n").trim_end()
        );
        let json = serde_json::json!({"entity_id":EntityId::new(),"name":"Legacy","definition_id":"wolf","size":"Medium","additional_languages":[],"ammunition_units":0,"item_ids":[]});
        let creation: crate::TableCreatureCreation = serde_json::from_value(json.clone()).unwrap();
        assert!(creation.source.is_none());
        assert_eq!(serde_json::to_value(creation).unwrap(), json);
        let current = current_catalog().unwrap();
        assert!(current.iter().all(|o| o.source.is_some()));
        assert!(
            current
                .iter()
                .any(|o| o.definition_id == "air-elemental" && !o.execution_limits.is_empty())
        );
        assert!(!options.iter().any(|o| o.definition_id == "air-elemental"));
    }
}
