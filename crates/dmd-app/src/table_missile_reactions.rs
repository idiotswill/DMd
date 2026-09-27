//! One private acknowledgment per committed target, with no competitor summary.
use crate::*;
use dmd_domain::*;
use std::collections::HashSet;

pub(super) fn view(
    state: &CampaignState,
    own: &HashSet<EntityId>,
    host: bool,
) -> Result<Option<Box<TableMissileView>>, String> {
    let Some(encounter) = &state.encounter else {
        return Ok(None);
    };
    let Some(flow) = &encounter.flow else {
        return Ok(None);
    };
    if !TacticalExecutionVersion::from_flow_version(flow.version)
        .is_some_and(TacticalExecutionVersion::supports_missile_shield)
    {
        return Ok(None);
    }
    let Some(resolution) = &flow.resolution else {
        return Ok(None);
    };
    let mut waiting = resolution.missiles.iter().filter(|missile| {
        matches!(
            missile.stage,
            TacticalMissileStage::Collecting | TacticalMissileStage::Selected { .. }
        )
    });
    let Some(missile) = waiting.next() else {
        return Ok(None);
    };
    if waiting.next().is_some() {
        return Err("Missile responses have more than one current window.".into());
    }
    if !host
        && !own.contains(&resolution.turn_actor)
        && !missile
            .respondents
            .iter()
            .any(|target| own.contains(&target.response.actor))
    {
        return Ok(None);
    }
    let controlled = |actor| {
        own.contains(&actor) || (host && !super::hit_reactions::player_controlled(state, actor))
    };
    let key = TacticalWorkKey {
        resolution: resolution.origin.id,
        occurrence: missile.work.occurrence,
    };
    let may_order = if missile.delegated_by.is_some() {
        host
    } else {
        controlled(resolution.turn_actor)
    };
    let order = if missile.order.is_none() && may_order {
        let observer = (!host)
            .then(|| {
                dmd_rules::spatial::project_actor_view(encounter, state, resolution.turn_actor)
                    .map_err(|error| error.to_string())
            })
            .transpose()?;
        let participants = state
            .rules
            .as_ref()
            .and_then(|rules| rules.timing.as_ref())
            .ok_or("Missile response has no initiative.")?
            .order
            .iter()
            .filter_map(|entry| {
                let label = if host || entry.actor == resolution.turn_actor {
                    encounter
                        .participant(entry.actor)
                        .map(|participant| participant.public_label.clone())
                } else {
                    observer
                        .as_ref()?
                        .contacts
                        .iter()
                        .find(|contact| {
                            contact.entity_id == entry.actor
                                && contact.status != dmd_rules::spatial::ContactStatus::Remembered
                        })
                        .map(|contact| {
                            contact
                                .label
                                .clone()
                                .unwrap_or_else(|| "Unseen creature".into())
                        })
                }?;
                Some(TableAttackTarget {
                    actor: entry.actor,
                    label,
                })
            })
            .collect();
        Some(TableHitOrder {
            key,
            actor: resolution.turn_actor,
            participants,
        })
    } else {
        None
    };
    let delegate = (missile.order.is_none()
        && missile.delegated_by.is_none()
        && controlled(resolution.turn_actor))
    .then_some(key);
    let responses = missile
        .respondents
        .iter()
        .enumerate()
        .filter_map(|(index, target)| {
            let selected = matches!(missile.stage, TacticalMissileStage::Selected { respondent }
            if usize::from(respondent) == index);
            let offered = missile.stage == TacticalMissileStage::Collecting
                && target.response.intent.is_none();
            (controlled(target.response.actor) && (selected || offered))
                .then_some((target, selected))
        })
        .map(|(target, selected)| {
            Ok(TableHitResponse {
                key,
                actor: target.response.actor,
                selected,
                shield: dmd_rules::tactical::shield_choices(state, target.response.actor)
                    .map_err(|error| error.to_string())?,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    // Retain a uniform waiting surface for owners/turn controller while others
    // decide. No stage, outstanding count, accepted set or foreign actor is sent.
    Ok(Some(Box::new(TableMissileView {
        order,
        delegate,
        responses,
    })))
}
