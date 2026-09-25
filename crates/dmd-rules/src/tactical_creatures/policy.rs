//! Bounded proposals from actor-visible knowledge. No world/encounter aggregate is
//! accepted by the policy function and no proposal is an accepted command.
use super::*;
use crate::{
    spatial::{ActorTacticalView, ContactStatus},
    tactical_definitions::*,
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NpcCapabilities {
    actor: EntityId,
    definition: String,
    hp: u32,
    max_hp: u32,
    footprint: u32,
    can_speak: bool,
    can_act: bool,
    selections: Vec<CreatureFeatureSelection>,
}
/// Trusted projection boundary: inspects only this source creature's state. It
/// deliberately rejects PCs and all host/player-controlled creatures.
pub fn npc_capabilities(
    state: &CampaignState,
    current: &TacticalCreatures,
    actor: EntityId,
) -> Result<NpcCapabilities, CreatureError> {
    validate_tactical_creatures(state, current)?;
    let profile = current
        .profile(actor)
        .ok_or_else(|| invalid("unknown NPC profile"))?;
    let runtime = current
        .runtime(actor)
        .ok_or_else(|| invalid("unknown NPC runtime"))?;
    if runtime.controller != CreatureController::Autonomous
        || state.characters.values().any(|pc| pc.entity_id == actor)
    {
        return Err(CreatureError::Unauthorized);
    }
    let source = source_for_profile(profile)?;
    let mechanics = state
        .rules
        .as_ref()
        .and_then(|r| r.entities.get(&actor))
        .ok_or_else(|| invalid("missing NPC mechanics"))?;
    let mut selections = vec![];
    for feature in &source.features {
        let timing = state.rules.as_ref().and_then(|r| r.timing.as_ref());
        let permitted = match runtime.observed_turn {
            None => matches!(
                feature.activation,
                FeatureActivation::Action | FeatureActivation::BonusAction
            ),
            Some(turn) if turn.actor == actor && turn.boundary == TurnBoundary::Start => {
                match feature.activation {
                    FeatureActivation::Action => timing.is_some_and(|t| !t.action_spent),
                    FeatureActivation::BonusAction => timing.is_some_and(|t| !t.bonus_action_spent),
                    FeatureActivation::Legendary { .. } | FeatureActivation::Reaction => false,
                }
            }
            Some(turn) if turn.actor != actor && turn.boundary == TurnBoundary::End => {
                match feature.activation {
                    FeatureActivation::Legendary { cost } => {
                        creature_legendary_action_available(state, current, actor)?
                            && source.legendary_budget.as_ref().is_some_and(|budget| {
                                runtime.legendary_spent.saturating_add(cost)
                                    <= if runtime.in_lair {
                                        budget.uses_in_lair
                                    } else {
                                        budget.uses
                                    }
                            })
                    }
                    _ => false,
                }
            }
            _ => false,
        };
        if !permitted {
            continue;
        }
        let alternatives = match &feature.feature {
            MonsterFeature::Spellcasting { spells, .. } => spells
                .iter()
                .map(|s| CreatureFeatureSelection {
                    feature_id: feature.id.clone(),
                    spell_id: Some(s.spell_id.clone()),
                    simple_action: None,
                })
                .collect(),
            MonsterFeature::BasicActionChoice { options } => options
                .iter()
                .map(|a| CreatureFeatureSelection {
                    feature_id: feature.id.clone(),
                    spell_id: None,
                    simple_action: Some(match a {
                        BasicAction::Dash => CreatureSimpleAction::Dash,
                        BasicAction::Disengage => CreatureSimpleAction::Disengage,
                        BasicAction::Dodge => CreatureSimpleAction::Dodge,
                        BasicAction::Hide => CreatureSimpleAction::Hide,
                    }),
                })
                .collect(),
            _ => vec![CreatureFeatureSelection {
                feature_id: feature.id.clone(),
                spell_id: None,
                simple_action: None,
            }],
        };
        for selection in alternatives {
            if schedule::available(source, runtime, &selection).is_ok() {
                selections.push(selection);
            }
        }
    }
    Ok(NpcCapabilities {
        actor,
        definition: source.id.clone(),
        hp: mechanics.hp,
        max_hp: mechanics.max_hp,
        footprint: profile.size.footprint_units() as u32,
        can_speak: source.statistics.can_speak,
        can_act: schedule::can_act(state, actor).is_ok(),
        selections,
    })
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NpcGoal {
    HoldPosition,
    ProtectKnownAllies,
    DefeatKnownThreats,
    Escape,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NpcMorale {
    pub retreat_at_or_below_percent: u8,
    pub willing_to_surrender: bool,
    pub willing_to_parley: bool,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KnownDisposition {
    Ally,
    Threat,
    Neutral,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KnownRelation {
    pub entity: EntityId,
    pub disposition: KnownDisposition,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NpcReason {
    SourceSpecialAbility,
    SourceRoutine,
    KnownThreat,
    LastKnownPosition,
    ReachKnownThreat,
    MoraleThreshold,
    EscapeGoal,
    NoKnownThreat,
    CannotAct,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum NpcIntent {
    UseFeature {
        selection: CreatureFeatureSelection,
        known_targets: Vec<EntityId>,
    },
    Investigate {
        last_known_position: SpatialPoint,
    },
    Retreat {
        toward_known_cell: SpatialPoint,
    },
    Approach {
        toward_known_cell: SpatialPoint,
    },
    OfferSurrender,
    OfferParley {
        spoken: bool,
    },
    Dodge,
    Wait,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NpcProposal {
    pub intent: NpcIntent,
    pub reason: NpcReason,
}

/// Order is a preference, not mandatory behavior. Source special/routine choices
/// precede ordinary attacks (SRD255). Root obtains all remaining targets, routine
/// choices and geometry, and authoritatively validates the chosen proposal.
pub fn propose_npc_intents(
    view: &ActorTacticalView,
    capabilities: &NpcCapabilities,
    relations: &[KnownRelation],
    goal: NpcGoal,
    morale: NpcMorale,
) -> Result<Vec<NpcProposal>, CreatureError> {
    if view.observer != capabilities.actor
        || view.contacts.len() > 128
        || view.cells.len() > 4096
        || relations.len() > 128
        || morale.retreat_at_or_below_percent > 100
    {
        return Err(invalid("invalid bounded NPC knowledge/morale input"));
    }
    let mut known = HashSet::new();
    for contact in &view.contacts {
        if contact.entity_id == view.observer || !known.insert(contact.entity_id) {
            return Err(invalid("duplicate/self knowledge contact"));
        }
    }
    let mut assigned = HashSet::new();
    for relation in relations {
        if !known.contains(&relation.entity) || !assigned.insert(relation.entity) {
            return Err(invalid("relation is not actor-known or is duplicated"));
        }
    }
    if !capabilities.can_act {
        return Ok(vec![NpcProposal {
            intent: NpcIntent::Wait,
            reason: NpcReason::CannotAct,
        }]);
    }
    let source = creature_definition(&capabilities.definition)?;
    let mut threats = view
        .contacts
        .iter()
        .filter(|c| {
            relations
                .iter()
                .any(|r| r.entity == c.entity_id && r.disposition == KnownDisposition::Threat)
        })
        .collect::<Vec<_>>();
    threats.sort_by_key(|contact| contact.entity_id.0);
    let present = threats
        .iter()
        .filter(|c| c.status != ContactStatus::Remembered)
        .copied()
        .collect::<Vec<_>>();
    let nearest = |point: SpatialPoint| {
        present
            .iter()
            .map(|contact| distance(point, contact.position))
            .min()
    };
    let low = capabilities.max_hp > 0
        && u64::from(capabilities.hp) * 100
            <= u64::from(capabilities.max_hp) * u64::from(morale.retreat_at_or_below_percent);
    let mut proposals = vec![];
    if goal == NpcGoal::Escape || low {
        let reason = if goal == NpcGoal::Escape {
            NpcReason::EscapeGoal
        } else {
            NpcReason::MoraleThreshold
        };
        for selection in capabilities.selections.iter().filter(|selection| {
            matches!(
                selection.simple_action,
                Some(CreatureSimpleAction::Dash | CreatureSimpleAction::Disengage)
            )
        }) {
            proposals.push(NpcProposal {
                intent: NpcIntent::UseFeature {
                    selection: selection.clone(),
                    known_targets: vec![],
                },
                reason: reason.clone(),
            });
        }
        if let Some(position) = view.position {
            let mut cells = view
                .cells
                .iter()
                .filter(|cell| {
                    cell.currently_seen
                        && !cell.blocked
                        && cell.position != position
                        && distance(position, cell.position) <= 10
                })
                .collect::<Vec<_>>();
            // A policy only proposes one locally known step; authoritative movement
            // checks footprints, surfaces, reach and opportunity windows separately.
            cells.sort_by_key(|cell| {
                (
                    std::cmp::Reverse(
                        present
                            .iter()
                            .map(|c| distance(cell.position, c.position))
                            .min()
                            .unwrap_or(0),
                    ),
                    cell.position.x,
                    cell.position.y,
                    cell.position.z,
                )
            });
            if let Some(cell) = cells.first() {
                proposals.push(NpcProposal {
                    intent: NpcIntent::Retreat {
                        toward_known_cell: cell.position,
                    },
                    reason: reason.clone(),
                });
            }
        }
        if morale.willing_to_surrender && !present.is_empty() {
            proposals.push(NpcProposal {
                intent: NpcIntent::OfferSurrender,
                reason: reason.clone(),
            });
        }
        if morale.willing_to_parley && !present.is_empty() {
            proposals.push(NpcProposal {
                intent: NpcIntent::OfferParley {
                    spoken: capabilities.can_speak,
                },
                reason,
            });
        }
    }
    if goal != NpcGoal::Escape && !present.is_empty() {
        let mut selections = capabilities.selections.iter().collect::<Vec<_>>();
        selections.sort_by_key(|selection| {
            let f = source
                .features
                .iter()
                .find(|f| f.id == selection.feature_id)
                .expect("sealed capabilities");
            let priority = match f.feature {
                MonsterFeature::SaveArea { .. } | MonsterFeature::Spellcasting { .. } => 0,
                MonsterFeature::Multiattack { .. } | MonsterFeature::MultiattackRoutine { .. } => 1,
                _ => 2,
            };
            (
                priority,
                selection.feature_id.clone(),
                selection.spell_id.clone(),
            )
        });
        for selection in selections {
            if proposals.len() >= 30 {
                break;
            }
            let f = source
                .features
                .iter()
                .find(|f| f.id == selection.feature_id)
                .expect("sealed capabilities");
            let reason = match f.feature {
                MonsterFeature::SaveArea { .. } | MonsterFeature::Spellcasting { .. } => {
                    NpcReason::SourceSpecialAbility
                }
                MonsterFeature::Multiattack { .. } | MonsterFeature::MultiattackRoutine { .. } => {
                    NpcReason::SourceRoutine
                }
                _ => NpcReason::KnownThreat,
            };
            // A remembered contact is never used as a current target. Sight-dependent
            // spells are conservative: only seen contacts are suggested.
            let candidates = present
                .iter()
                .filter(|c| {
                    (selection.spell_id.is_none() || c.status == ContactStatus::Seen)
                        && view.position.is_some_and(|position| {
                            distance(position, c.position)
                                <= source_reach(source, selection)
                                    .saturating_add(u64::from(capabilities.footprint))
                        })
                })
                .map(|c| c.entity_id)
                .collect::<Vec<_>>();
            if !candidates.is_empty() {
                proposals.push(NpcProposal {
                    intent: NpcIntent::UseFeature {
                        selection: selection.clone(),
                        known_targets: candidates,
                    },
                    reason,
                });
            }
        }
    }
    if !matches!(goal, NpcGoal::HoldPosition | NpcGoal::Escape)
        && !present.is_empty()
        && let Some(position) = view.position
        && let Some(current_distance) = nearest(position)
    {
        let mut cells = view
            .cells
            .iter()
            .filter(|cell| {
                cell.currently_seen
                    && !cell.blocked
                    && cell.position != position
                    && distance(position, cell.position) <= 10
                    && nearest(cell.position).is_some_and(|n| n < current_distance)
            })
            .collect::<Vec<_>>();
        cells.sort_by_key(|cell| {
            (
                nearest(cell.position),
                cell.position.x,
                cell.position.y,
                cell.position.z,
            )
        });
        if let Some(cell) = cells.first() {
            proposals.push(NpcProposal {
                intent: NpcIntent::Approach {
                    toward_known_cell: cell.position,
                },
                reason: NpcReason::ReachKnownThreat,
            });
        }
    }
    if present.is_empty()
        && goal != NpcGoal::HoldPosition
        && goal != NpcGoal::Escape
        && let Some(contact) = threats.first()
    {
        proposals.push(NpcProposal {
            intent: NpcIntent::Investigate {
                last_known_position: contact.position,
            },
            reason: NpcReason::LastKnownPosition,
        });
    }
    proposals.push(NpcProposal {
        intent: NpcIntent::Dodge,
        reason: if threats.is_empty() {
            NpcReason::NoKnownThreat
        } else {
            NpcReason::KnownThreat
        },
    });
    Ok(proposals)
}
fn source_reach(source: &CreatureDefinition, selection: &CreatureFeatureSelection) -> u64 {
    let Some(feature) = source
        .features
        .iter()
        .find(|f| f.id == selection.feature_id)
    else {
        return 0;
    };
    let feet = match &feature.feature {
        MonsterFeature::Attack { delivery, .. } => match delivery {
            AttackDelivery::Melee { reach_feet } => *reach_feet,
            AttackDelivery::Ranged { range } => range.long_feet,
        },
        MonsterFeature::Spellcasting { .. } => selection
            .spell_id
            .as_ref()
            .and_then(|id| creature_definitions().ok()?.spell(id))
            .map_or(0, |spell| match spell.range {
                SpellRange::Caster => 0,
                SpellRange::Touch => 5,
                SpellRange::Distance { feet } => feet,
            }),
        MonsterFeature::SaveArea { area, .. } => match area {
            AreaShape::Cone { length_feet } | AreaShape::Line { length_feet, .. } => *length_feet,
            AreaShape::Cube { side_feet } => *side_feet,
            AreaShape::Cylinder { radius_feet, .. }
            | AreaShape::Emanation { radius_feet }
            | AreaShape::Sphere { radius_feet } => *radius_feet,
        },
        MonsterFeature::Multiattack { attack_options, .. } => {
            return attack_options
                .iter()
                .map(|id| {
                    source_reach(
                        source,
                        &CreatureFeatureSelection {
                            feature_id: id.clone(),
                            spell_id: None,
                            simple_action: None,
                        },
                    )
                })
                .max()
                .unwrap_or(0);
        }
        MonsterFeature::MultiattackRoutine { slots, .. } => {
            return slots
                .iter()
                .flat_map(|slot| &slot.options)
                .map(|choice| {
                    source_reach(
                        source,
                        &CreatureFeatureSelection {
                            feature_id: choice.feature_id.clone(),
                            spell_id: choice.spell_id.clone(),
                            simple_action: None,
                        },
                    )
                })
                .max()
                .unwrap_or(0);
        }
        MonsterFeature::MoveThenAttack { attack_id, .. } => {
            return source_reach(
                source,
                &CreatureFeatureSelection {
                    feature_id: attack_id.clone(),
                    spell_id: None,
                    simple_action: None,
                },
            );
        }
        _ => 0,
    };
    u64::from(feet) * 2
}
fn distance(a: SpatialPoint, b: SpatialPoint) -> u64 {
    [
        (i64::from(a.x) - i64::from(b.x)).unsigned_abs(),
        (i64::from(a.y) - i64::from(b.y)).unsigned_abs(),
        (i64::from(a.z) - i64::from(b.z)).unsigned_abs(),
    ]
    .into_iter()
    .max()
    .unwrap_or(0)
}
