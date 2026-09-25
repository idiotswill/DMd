//! Internal source-authorized scheduling. Application composes returned costs and payloads
//! with its central timing/target/continuation transaction; these are not IPC actions.
use super::*;
use crate::{ResolveRoll, active_conditions, tactical_definitions::*};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

mod capabilities;
#[cfg(test)]
mod reaction_tests;
mod resistance;
mod validation;
use capabilities::*;
pub(super) use capabilities::{available, can_act};
pub use resistance::*;
pub use validation::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreatureRechargeId {
    pub feature_id: String,
    pub request_id: RollRequestId,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatureTurnHooks {
    pub own_start: bool,
    pub recharge: Vec<String>,
    pub legendary_window: bool,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum CreatureScheduleOperation {
    ObserveTurn {
        actor: EntityId,
        turn: CreatureTurn,
        recharge_ids: Vec<CreatureRechargeId>,
    },
    SubmitRecharge {
        actor: EntityId,
        feature_id: String,
        result: RollResult,
    },
    BeginFeature {
        actor: EntityId,
        selection: CreatureFeatureSelection,
        steps: Vec<CreatureRoutineStep>,
    },
    TakeStep {
        actor: EntityId,
    },
    AbandonRoutine {
        actor: EntityId,
    },
    DeclineLegendaryAction {
        actor: EntityId,
    },
    FinishRest {
        actor: EntityId,
        receipt: CreatureRestReceipt,
    },
    LeaveCombat {
        actor: EntityId,
    },
    SetContext {
        actor: EntityId,
        controller: CreatureController,
        in_lair: bool,
    },
    /// Changes only ownership; source lair context and its original proof are retained.
    SetController {
        actor: EntityId,
        controller: CreatureController,
    },
}
impl CreatureScheduleOperation {
    fn actor(&self) -> EntityId {
        match self {
            Self::ObserveTurn { actor, .. }
            | Self::SubmitRecharge { actor, .. }
            | Self::BeginFeature { actor, .. }
            | Self::TakeStep { actor }
            | Self::AbandonRoutine { actor }
            | Self::DeclineLegendaryAction { actor }
            | Self::FinishRest { actor, .. }
            | Self::LeaveCombat { actor }
            | Self::SetController { actor, .. }
            | Self::SetContext { actor, .. } => *actor,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CreatureActionCost {
    None,
    Action,
    BonusAction,
    Reaction,
    Legendary(u8),
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatureActivationReceipt {
    pub origin: CommandMeta,
    /// This enclosing source activation owns the cost, including a Legendary
    /// Action or an already-paid Multiattack. A nested spell must not pay again.
    pub activation: FeatureActivation,
    /// SRD257 places stat-block Multiattack inside the Attack action.
    pub attack_action: bool,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatureFeaturePlan {
    pub actor: EntityId,
    /// This primitive's accepted invocation; a routine step may occur in a later
    /// command than the enclosing Attack action.
    pub invocation: CommandMeta,
    pub source: CreatureSourcePin,
    pub selection: CreatureFeatureSelection,
    /// Immutable source payload, never an after-state. Root owns targets/rolls/effect execution.
    pub feature: NamedMonsterFeature,
    pub enclosing_activation: CreatureActivationReceipt,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatureScheduleTransition {
    pub next: TacticalCreatures,
    pub cost: CreatureActionCost,
    pub activation: Option<CreatureActivationReceipt>,
    pub requests: Vec<RollRequest>,
    pub feature: Option<CreatureFeaturePlan>,
}

pub fn creature_recharge_request(
    actor: EntityId,
    source: &CreatureDefinition,
    feature_id: &str,
    id: RollRequestId,
) -> Result<RollRequest, CreatureError> {
    let f = feature(source, feature_id)?;
    if id.0.is_nil() || actor.0.is_nil() || recharge_policy(f).is_none() {
        return Err(invalid(
            "recharge requires an eligible source feature and request ID",
        ));
    }
    Ok(RollRequest {
        id,
        roller: Some(actor),
        dice: vec![DieSpec { count: 1, sides: 6 }],
        modifier: 0,
        mode: RollMode::Normal,
        visibility: RollVisibility::Secret,
        reason: format!("{}: {} recharge", source.name, f.name),
    })
}
/// Does not consume/queue anything. Root schedules these source hooks with other
/// simultaneous work; only an accepted ObserveTurn changes the persisted authority.
pub fn creature_turn_hooks(
    state: &CampaignState,
    current: &TacticalCreatures,
    actor: EntityId,
    turn: CreatureTurn,
) -> Result<CreatureTurnHooks, CreatureError> {
    validate_tactical_creatures(state, current)?;
    validate_current_turn(state, turn)?;
    let rt = current
        .runtime(actor)
        .ok_or_else(|| invalid("unknown creature runtime"))?;
    if state
        .encounter
        .as_ref()
        .is_none_or(|encounter| encounter.participant(actor).is_none())
    {
        return Err(invalid(
            "creature is not a participant in this turn's encounter",
        ));
    }
    if !after(rt.observed_turn, turn) {
        return Err(invalid("duplicate/out-of-order creature turn hook"));
    }
    let source = source_for_profile(
        current
            .profile(actor)
            .ok_or_else(|| invalid("unknown profile"))?,
    )?;
    let own_start = turn.actor == actor && turn.boundary == TurnBoundary::Start;
    Ok(CreatureTurnHooks {
        own_start,
        recharge: if own_start {
            rt.recharge
                .iter()
                .filter(|r| {
                    !r.available
                        && recharge_policy(
                            feature(source, &r.feature_id).expect("validated source"),
                        )
                        .is_some()
                })
                .map(|r| r.feature_id.clone())
                .collect()
        } else {
            vec![]
        },
        legendary_window: turn.actor != actor
            && turn.boundary == TurnBoundary::End
            && legendary_max(source, rt.in_lair) > rt.legendary_spent
            && can_act(state, actor).is_ok(),
    })
}

pub fn apply_creature_schedule(
    state: &CampaignState,
    current: &TacticalCreatures,
    meta: &CommandMeta,
    operation: &CreatureScheduleOperation,
) -> Result<CreatureScheduleTransition, CreatureError> {
    apply_schedule(state, current, meta, operation, ScheduleAuthority::Declared)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ScheduleAuthority {
    Declared,
    TurnObservation,
    ReactionWindow,
}

/// Only the live, source-verified reaction-window adapter may call this hook.
/// The controller's accepted choice remains the invocation; this does not invent
/// a System issuer or prove a trigger by accepting a serialized flag. The caller
/// atomically pays central Reaction cost and opens the source spell continuation.
#[allow(
    dead_code,
    reason = "live reaction window integration is the next owned scheduler slice"
)]
pub(crate) fn begin_creature_reaction_feature(
    state: &CampaignState,
    current: &TacticalCreatures,
    meta: &CommandMeta,
    actor: EntityId,
    selection: CreatureFeatureSelection,
) -> Result<CreatureScheduleTransition, CreatureError> {
    apply_schedule(
        state,
        current,
        meta,
        &CreatureScheduleOperation::BeginFeature {
            actor,
            selection,
            steps: vec![],
        },
        ScheduleAuthority::ReactionWindow,
    )
}

/// Called only by the central turn scheduler after its accepted command advances
/// the cursor. Preserve that exact command's issuer/actor, including Player A
/// causing source work for creature B; this is not a user-authored observation.
#[allow(
    dead_code,
    reason = "central turn adapter is integrated in a separate reviewed slice"
)]
pub(crate) fn observe_creature_turn(
    state: &CampaignState,
    current: &TacticalCreatures,
    meta: &CommandMeta,
    actor: EntityId,
    turn: CreatureTurn,
    recharge_ids: Vec<CreatureRechargeId>,
) -> Result<CreatureScheduleTransition, CreatureError> {
    apply_schedule(
        state,
        current,
        meta,
        &CreatureScheduleOperation::ObserveTurn {
            actor,
            turn,
            recharge_ids,
        },
        ScheduleAuthority::TurnObservation,
    )
}

fn apply_schedule(
    state: &CampaignState,
    current: &TacticalCreatures,
    meta: &CommandMeta,
    operation: &CreatureScheduleOperation,
    authority: ScheduleAuthority,
) -> Result<CreatureScheduleTransition, CreatureError> {
    validate_tactical_creatures(state, current)?;
    if meta.expected_event_sequence != state.applied_event_sequence {
        return Err(CreatureError::Stale);
    }
    let actor = operation.actor();
    validate_creature_origin(state, meta, actor).map_err(invalid)?;
    let profile = current
        .profile(actor)
        .ok_or_else(|| invalid("unknown creature"))?;
    let source = source_for_profile(profile)?;
    let mut next = current.clone();
    let rt = next
        .runtime
        .iter_mut()
        .find(|r| r.actor == actor)
        .ok_or_else(|| invalid("unknown creature runtime"))?;
    let mut cost = CreatureActionCost::None;
    let mut activation = None;
    let mut requests = vec![];
    let mut planned = None;
    match operation {
        CreatureScheduleOperation::ObserveTurn {
            turn, recharge_ids, ..
        } => {
            if authority != ScheduleAuthority::TurnObservation {
                privileged(meta)?;
            }
            let hooks = creature_turn_hooks(state, current, actor, *turn)?;
            if rt.recharge.iter().any(|r| r.pending.is_some()) || rt.routine.is_some() {
                return Err(CreatureError::Unavailable(
                    "unfinished creature work blocks turn advance".into(),
                ));
            }
            if recharge_ids.len() != hooks.recharge.len() {
                return Err(invalid("recharge request count differs from source hooks"));
            }
            let mut ids = HashSet::new();
            let mut features = HashSet::new();
            for id in recharge_ids {
                if !hooks.recharge.contains(&id.feature_id)
                    || !features.insert(&id.feature_id)
                    || !ids.insert(id.request_id)
                    || request_id_exists(state, current, id.request_id)
                {
                    return Err(invalid("duplicate/unknown/colliding recharge request"));
                }
                let request =
                    creature_recharge_request(actor, source, &id.feature_id, id.request_id)?;
                rt.recharge
                    .iter_mut()
                    .find(|r| r.feature_id == id.feature_id)
                    .expect("validated hook")
                    .pending = Some(CreatureRechargeTicket {
                    origin: meta.clone(),
                    turn: *turn,
                    request: request.clone(),
                });
                requests.push(request);
            }
            if hooks.own_start {
                rt.legendary_spent = 0;
                rt.used_this_own_turn.clear();
            }
            rt.observed_turn = Some(*turn);
            rt.legendary_window_spent = false;
        }
        CreatureScheduleOperation::SubmitRecharge {
            feature_id, result, ..
        } => {
            authorize(state, rt, meta)?;
            // Source creature rolls are secret; digital generation is system/host only.
            privileged(meta)?;
            let row = rt
                .recharge
                .iter_mut()
                .find(|r| &r.feature_id == feature_id)
                .ok_or_else(|| invalid("unknown recharge feature"))?;
            let ticket = row
                .pending
                .as_ref()
                .ok_or_else(|| invalid("no pending recharge"))?
                .clone();
            validate_current_turn(state, ticket.turn)?;
            let resolved = ticket
                .request
                .resolve(result)
                .map_err(|e| invalid(e.to_string()))?;
            let policy = recharge_policy(feature(source, feature_id)?)
                .ok_or_else(|| invalid("source feature has no d6 recharge"))?;
            row.available =
                (i32::from(policy.minimum)..=i32::from(policy.maximum)).contains(&resolved.total);
            row.pending = None;
            row.last_roll = Some(CreatureRechargeRecord {
                ticket,
                accepted_by: meta.clone(),
                result: result.clone(),
            });
        }
        CreatureScheduleOperation::BeginFeature {
            selection, steps, ..
        } => {
            authorize(state, rt, meta)?;
            can_act(state, actor)?;
            let f = validate_selection(source, selection)?;
            let reaction = authority == ScheduleAuthority::ReactionWindow;
            if reaction != (f.activation == FeatureActivation::Reaction) {
                return Err(CreatureError::Unauthorized);
            }
            if !reaction
                && (rt.routine.is_some() || rt.recharge.iter().any(|r| r.pending.is_some()))
            {
                return Err(CreatureError::Unavailable(
                    "creature has unfinished source work".into(),
                ));
            }
            let timing = rules(state)?
                .timing
                .as_ref()
                .ok_or_else(|| invalid("feature requires central combat timing"))?;
            let turn = if reaction {
                let encounter = state
                    .encounter
                    .as_ref()
                    .ok_or_else(|| invalid("reaction requires an encounter"))?;
                if encounter.participant(actor).is_none() {
                    return Err(invalid("reactor is not an encounter participant"));
                }
                CreatureTurn {
                    encounter_id: encounter.id,
                    actor: timing
                        .order
                        .get(timing.index)
                        .ok_or_else(|| invalid("missing active turn"))?
                        .actor,
                    number: timing.turn_number,
                    boundary: TurnBoundary::Start,
                }
            } else {
                rt.observed_turn
                    .ok_or_else(|| invalid("feature requires an observed combat boundary"))?
            };
            validate_current_turn(state, turn)?;
            cost = match f.activation {
                FeatureActivation::Reaction
                    if reaction && !timing.reactions_spent.contains(&actor) =>
                {
                    CreatureActionCost::Reaction
                }
                FeatureActivation::Action
                    if turn.actor == actor
                        && turn.boundary == TurnBoundary::Start
                        && !timing.action_spent =>
                {
                    CreatureActionCost::Action
                }
                FeatureActivation::BonusAction
                    if turn.actor == actor
                        && turn.boundary == TurnBoundary::Start
                        && !timing.bonus_action_spent =>
                {
                    CreatureActionCost::BonusAction
                }
                FeatureActivation::Legendary { cost }
                    if turn.actor != actor
                        && turn.boundary == TurnBoundary::End
                        && !rt.legendary_window_spent
                        && rt.legendary_spent.saturating_add(cost)
                            <= legendary_max(source, rt.in_lair) =>
                {
                    CreatureActionCost::Legendary(cost)
                }
                _ => {
                    return Err(CreatureError::Unavailable(
                        "source activation timing or budget unavailable".into(),
                    ));
                }
            };
            activation = Some(CreatureActivationReceipt {
                origin: meta.clone(),
                activation: f.activation,
                attack_action: f.activation == FeatureActivation::Action
                    && matches!(
                        f.feature,
                        MonsterFeature::Attack { .. }
                            | MonsterFeature::Multiattack { .. }
                            | MonsterFeature::MultiattackRoutine { .. }
                    ),
            });
            if matches!(
                f.feature,
                MonsterFeature::Multiattack { .. } | MonsterFeature::MultiattackRoutine { .. }
            ) {
                validate_steps(source, &f.id, steps)?;
                // Check every chosen primitive in a disposable resource image. Do not
                // actually spend a later breath/spell before its accepted step begins.
                let mut forecast = rt.clone();
                for step in steps {
                    spend(source, &mut forecast, &step.selection)?;
                }
                spend(source, rt, selection)?;
                rt.routine = Some(CreatureRoutineContinuation {
                    origin: meta.clone(),
                    turn,
                    feature_id: f.id.clone(),
                    steps: steps.clone(),
                    next_step: 0,
                });
            } else {
                if !steps.is_empty() {
                    return Err(invalid("primitive feature cannot carry routine steps"));
                }
                spend(source, rt, selection)?;
                planned = Some(CreatureFeaturePlan {
                    actor,
                    invocation: meta.clone(),
                    source: profile.source.clone(),
                    selection: selection.clone(),
                    feature: f.clone(),
                    enclosing_activation: CreatureActivationReceipt {
                        origin: meta.clone(),
                        activation: f.activation,
                        attack_action: f.activation == FeatureActivation::Action
                            && matches!(f.feature, MonsterFeature::Attack { .. }),
                    },
                });
            }
            if let CreatureActionCost::Legendary(n) = cost {
                rt.legendary_spent += n;
                rt.legendary_window_spent = true;
            }
        }
        CreatureScheduleOperation::TakeStep { .. } => {
            authorize(state, rt, meta)?;
            can_act(state, actor)?;
            let mut routine = rt
                .routine
                .clone()
                .ok_or_else(|| invalid("no source routine pending"))?;
            validate_current_turn(state, routine.turn)?;
            if !rules(state)?
                .timing
                .as_ref()
                .expect("validated timing")
                .action_spent
            {
                return Err(invalid("routine action was not paid by central timing"));
            }
            let step = &routine.steps[usize::from(routine.next_step)];
            spend(source, rt, &step.selection)?;
            planned = Some(CreatureFeaturePlan {
                actor,
                invocation: meta.clone(),
                source: profile.source.clone(),
                selection: step.selection.clone(),
                feature: feature(source, &step.selection.feature_id)?.clone(),
                enclosing_activation: CreatureActivationReceipt {
                    origin: routine.origin.clone(),
                    activation: FeatureActivation::Action,
                    attack_action: true,
                },
            });
            routine.next_step += 1;
            rt.routine = (usize::from(routine.next_step) < routine.steps.len()).then_some(routine);
        }
        CreatureScheduleOperation::AbandonRoutine { .. } => {
            authorize(state, rt, meta)?;
            if rt.routine.take().is_none() {
                return Err(invalid("no routine to abandon"));
            }
        }
        CreatureScheduleOperation::DeclineLegendaryAction { .. } => {
            authorize(state, rt, meta)?;
            let turn = rt
                .observed_turn
                .ok_or_else(|| invalid("no legendary end window"))?;
            validate_current_turn(state, turn)?;
            if turn.actor == actor
                || turn.boundary != TurnBoundary::End
                || rt.legendary_window_spent
                || legendary_max(source, rt.in_lair) <= rt.legendary_spent
            {
                return Err(CreatureError::Unavailable(
                    "no eligible legendary window".into(),
                ));
            }
            rt.legendary_window_spent = true;
        }
        CreatureScheduleOperation::FinishRest { receipt, .. } => {
            privileged(meta)?;
            authorize(state, rt, meta)?;
            validate_creature_origin(state, &receipt.origin, actor).map_err(invalid)?;
            privileged(&receipt.origin)?;
            if receipt.origin != *meta
                || receipt.finished_at != state.clock.now
                || rules(state)?.timing.is_some()
                || rt.routine.is_some()
                || rt.recharge.iter().any(|r| r.pending.is_some())
                || rt
                    .last_rest
                    .as_ref()
                    .is_some_and(|old| old.finished_at >= receipt.finished_at)
            {
                return Err(invalid("invalid completed source rest receipt"));
            }
            for r in &mut rt.recharge {
                r.available = true;
            }
            if receipt.kind == RestKind::Long {
                for r in &mut rt.limited_uses {
                    r.spent = 0;
                }
                rt.legendary_resistance_spent = 0;
                rt.legendary_resistance_rolls.clear();
            }
            rt.last_rest = Some(receipt.clone());
        }
        CreatureScheduleOperation::LeaveCombat { .. } => {
            privileged(meta)?;
            if rules(state)?.timing.is_some() {
                return Err(invalid(
                    "end central combat before leaving source combat cursor",
                ));
            }
            if rt.recharge.iter().any(|r| r.pending.is_some()) {
                return Err(invalid("resolve recharge before leaving combat"));
            }
            rt.routine = None;
            rt.observed_turn = None;
            rt.legendary_window_spent = false;
        }
        CreatureScheduleOperation::SetController { controller, .. } => {
            privileged(meta)?;
            if let CreatureController::Player(id) = controller
                && !state.players.contains_key(id)
            {
                return Err(invalid("unknown creature controller"));
            }
            if rt.routine.is_some() || rt.recharge.iter().any(|r| r.pending.is_some()) {
                return Err(invalid("cannot change source control during pending work"));
            }
            rt.controller = *controller;
            rt.control_origin = meta.clone();
        }
        CreatureScheduleOperation::SetContext {
            controller,
            in_lair,
            ..
        } => {
            privileged(meta)?;
            if let CreatureController::Player(id) = controller
                && !state.players.contains_key(id)
            {
                return Err(invalid("unknown creature controller"));
            }
            if rt.routine.is_some() || rt.recharge.iter().any(|r| r.pending.is_some()) {
                return Err(invalid("cannot change source control during pending work"));
            }
            rt.controller = *controller;
            rt.control_origin = meta.clone();
            rt.in_lair = *in_lair;
            rt.lair_origin = meta.clone();
        }
    }
    rt.last_operation = meta.clone();
    // Validate against the central cost the caller must apply in the same commit.
    let mut validation_image = state.clone();
    if let Some(timing) = validation_image
        .rules
        .as_mut()
        .and_then(|rules| rules.timing.as_mut())
    {
        match cost {
            CreatureActionCost::Action => timing.action_spent = true,
            CreatureActionCost::BonusAction => timing.bonus_action_spent = true,
            CreatureActionCost::Reaction => timing.reactions_spent.push(actor),
            _ => (),
        }
    }
    validate_tactical_creatures(&validation_image, &next)?;
    Ok(CreatureScheduleTransition {
        next,
        cost,
        activation,
        requests,
        feature: planned,
    })
}

fn request_id_exists(
    state: &CampaignState,
    current: &TacticalCreatures,
    id: RollRequestId,
) -> bool {
    id.0.is_nil() || state.rules.as_ref().is_some_and(|r|r.pending.as_ref().is_some_and(|p|p.request.id==id)||r.rolls.iter().any(|r|r.request.id==id)||r.cancelled_roll_ids.contains(&id))
        || current.profiles.iter().any(|p|matches!(&p.hit_points,CreatureHitPointOrigin::Rolled{request,..} if request.id==id))
        || current.runtime.iter().flat_map(|r|&r.recharge).any(|r|r.pending.as_ref().is_some_and(|p|p.request.id==id)||r.last_roll.as_ref().is_some_and(|p|p.ticket.request.id==id))
}

/// Rechecks the already-observed post-turn window after end effects have resolved.
/// Source capability only: targets/geometry are still chosen and validated separately.
pub fn creature_legendary_action_available(
    state: &CampaignState,
    current: &TacticalCreatures,
    actor: EntityId,
) -> Result<bool, CreatureError> {
    validate_tactical_creatures(state, current)?;
    let runtime = current
        .runtime(actor)
        .ok_or_else(|| invalid("unknown creature"))?;
    let profile = current
        .profile(actor)
        .ok_or_else(|| invalid("unknown creature profile"))?;
    let Some(turn) = runtime.observed_turn else {
        return Ok(false);
    };
    if turn.actor == actor
        || turn.boundary != TurnBoundary::End
        || runtime.legendary_window_spent
        || runtime.routine.is_some()
        || runtime.recharge.iter().any(|r| r.pending.is_some())
        || can_act(state, actor).is_err()
    {
        return Ok(false);
    }
    validate_current_turn(state, turn)?;
    let source = source_for_profile(profile)?;
    for f in &source.features {
        let FeatureActivation::Legendary { cost } = f.activation else {
            continue;
        };
        if runtime.legendary_spent.saturating_add(cost) > legendary_max(source, runtime.in_lair) {
            continue;
        }
        let selections = match &f.feature {
            MonsterFeature::Spellcasting { spells, .. } => spells
                .iter()
                .map(|s| CreatureFeatureSelection {
                    feature_id: f.id.clone(),
                    spell_id: Some(s.spell_id.clone()),
                    simple_action: None,
                })
                .collect::<Vec<_>>(),
            _ => vec![CreatureFeatureSelection {
                feature_id: f.id.clone(),
                spell_id: None,
                simple_action: None,
            }],
        };
        if selections
            .iter()
            .any(|s| available(source, runtime, s).is_ok())
        {
            return Ok(true);
        }
    }
    Ok(false)
}
