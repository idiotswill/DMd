//! Private hit decisions never project the eligibility or intent of another owner.
use crate::*;
use dmd_domain::*;
use std::collections::HashSet;

fn player_controlled(state: &CampaignState, actor: EntityId) -> bool {
    state.characters.values().any(|character| {
        character.entity_id == actor
            && character.controlling_player_id.is_some()
            && matches!(
                character.status,
                CharacterStatus::Active | CharacterStatus::Dead
            )
    }) || state
        .rules
        .as_ref()
        .and_then(|rules| rules.tactical_creatures.as_ref())
        .and_then(|creatures| creatures.runtime(actor))
        .is_some_and(|runtime| matches!(runtime.controller, CreatureController::Player(_)))
}

pub(super) fn view(
    state: &CampaignState,
    own: &HashSet<EntityId>,
    host: bool,
) -> Result<Option<Box<TableHitView>>, String> {
    let Some(encounter) = &state.encounter else {
        return Ok(None);
    };
    let Some(flow) = &encounter.flow else {
        return Ok(None);
    };
    if flow.version != TacticalExecutionVersion::ShieldHitV1.flow_version() {
        return Ok(None);
    }
    let Some(resolution) = &flow.resolution else {
        return Ok(None);
    };
    let Some(hit) = &resolution.hit_review else {
        return Ok(None);
    };
    if !matches!(
        hit.stage,
        TacticalHitReviewStage::Collecting | TacticalHitReviewStage::Selected
    ) {
        return Ok(None);
    }
    let target = hit
        .respondent
        .as_ref()
        .ok_or("Hit response lacks its target.")?
        .actor;
    let controlled = |actor| own.contains(&actor) || (host && !player_controlled(state, actor));
    // Every target acknowledges collection. This waiting surface never reflects
    // whether an eligible Shield exists or whether a private intent was accepted.
    if !host && !own.contains(&resolution.turn_actor) && !own.contains(&target) {
        return Ok(None);
    }
    let key = TacticalWorkKey {
        resolution: resolution.origin.id,
        occurrence: hit.work.occurrence,
    };
    let may_order = if hit.delegated_by.is_some() {
        host
    } else {
        controlled(resolution.turn_actor)
    };
    let order = if hit.order.is_none() && may_order {
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
            .ok_or("Hit response has no initiative.")?
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
    let delegate =
        (hit.order.is_none() && hit.delegated_by.is_none() && controlled(resolution.turn_actor))
            .then_some(key);
    let response = if controlled(target)
        && (hit.stage == TacticalHitReviewStage::Selected
            || hit
                .respondent
                .as_ref()
                .is_some_and(|target| target.intent.is_none()))
    {
        Some(TableHitResponse {
            key,
            actor: target,
            selected: hit.stage == TacticalHitReviewStage::Selected,
            shield: dmd_rules::tactical::shield_choices(state, target)
                .map_err(|error| error.to_string())?,
        })
    } else {
        None
    };
    Ok(Some(Box::new(TableHitView {
        order,
        delegate,
        response,
    })))
}
