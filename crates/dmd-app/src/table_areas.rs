//! Source-controller choices only. No target projection, geometry query, victim
//! count, private resistance, or damage preview enters this DTO.
use crate::{TableAreaOptions, TableAreaVariant};
use dmd_domain::*;
use dmd_rules::tactical_creatures::{
    CreatureActionCost, CreatureScheduleOperation, apply_creature_schedule, source_for_profile,
};
use dmd_rules::tactical_definitions::MonsterFeature;

pub(super) fn options(
    state: &CampaignState,
    actor: EntityId,
) -> Result<Option<TableAreaOptions>, String> {
    let Some(rules) = &state.rules else {
        return Ok(None);
    };
    let Some(creatures) = &rules.tactical_creatures else {
        return Ok(None);
    };
    let Some(profile) = creatures.profile(actor) else {
        return Ok(None);
    };
    let source = source_for_profile(profile).map_err(|e| e.to_string())?;
    let features = source
        .features
        .iter()
        .filter(|feature| matches!(feature.feature, MonsterFeature::SaveArea { .. }))
        .collect::<Vec<_>>();
    if features.is_empty() {
        return Ok(None);
    }
    let encounter = state
        .encounter
        .as_ref()
        .ok_or("Area source lacks a battlefield.")?;
    let participant = encounter
        .participant(actor)
        .ok_or("Area source is outside the battlefield.")?;
    let runtime = creatures
        .runtime(actor)
        .ok_or("Area source runtime is absent.")?;
    let controller = match runtime.controller {
        CreatureController::Player(player) => Some(player),
        _ => None,
    };
    let mut result = TableAreaOptions {
        actor,
        controller,
        source_space: participant.volume()?,
        variants: vec![],
        unavailable: vec![],
    };
    if encounter.area_grid_policy.is_none() {
        result.unavailable.push(
            "The host must choose how areas affect this map before this ability can be used."
                .into(),
        );
        return Ok(Some(result));
    }
    if dmd_rules::active_conditions(rules, actor).contains(&Condition::Charmed) {
        result.unavailable.push(
            "Area abilities while Charmed need an adjudication this table does not support yet."
                .into(),
        );
        return Ok(Some(result));
    }
    let meta = CommandMeta {
        id: CommandId(encounter.id.0),
        campaign_id: state.campaign_id(),
        session_id: state
            .table
            .as_ref()
            .and_then(|table| table.active_session.as_ref())
            .map(|session| session.session_id),
        issuer: controller.map_or(CommandIssuer::Admin, CommandIssuer::Player),
        actor: Some(AgentRef::Entity(actor)),
        expected_event_sequence: state.applied_event_sequence,
    };
    let mut budget_preview = rules.clone();
    if dmd_rules::tactical_budget::spend_cost(
        &mut budget_preview,
        actor,
        dmd_rules::tactical_budget::TacticalCost::Action,
    )
    .is_err()
    {
        result
            .unavailable
            .push("This creature cannot spend an Action now.".into());
        return Ok(Some(result));
    }
    for feature in features {
        let Ok(program) = dmd_rules::tactical_areas::source_area_program(state, actor, &feature.id)
        else {
            result
                .unavailable
                .push(format!("{} cannot be resolved here yet.", feature.name));
            continue;
        };
        let available = apply_creature_schedule(
            state,
            creatures,
            &meta,
            &CreatureScheduleOperation::BeginFeature {
                actor,
                selection: CreatureFeatureSelection {
                    feature_id: feature.id.clone(),
                    spell_id: None,
                    simple_action: None,
                },
                steps: vec![],
            },
        )
        .is_ok_and(|step| step.cost == CreatureActionCost::Action && step.feature.is_some());
        if available {
            result.variants.push(TableAreaVariant {
                feature_id: feature.id.clone(),
                label: feature.name.clone(),
                length_feet: program.length() / SPATIAL_UNITS_PER_FOOT,
            });
        } else {
            result.unavailable.push(format!(
                "{} is unavailable until its source use is restored.",
                feature.name
            ));
        }
    }
    Ok(Some(result))
}
