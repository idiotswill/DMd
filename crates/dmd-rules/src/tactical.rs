//! Versioned tactical transitions. The application supplies trusted command metadata;
//! all accepted inputs and raw dice are retained for deterministic semantic replay.
mod aftermath;
mod areas;
mod attacks;
mod casting;
mod continuations;
mod creature_bridge;
mod failed_save;
mod falling;
mod initiative;
mod medicine;
mod movement;
mod reaction_order;
mod ready;
mod second_wind;
mod shields;
mod turn_validation;
mod turns;
mod validation;
mod work_trace;
use crate::{ResolveRoll, RulesError, RulesPack};
pub use aftermath::require_aftermath_session_boundary;
pub use attacks::savage_attacker_dice;
use dmd_domain::*;
pub use failed_save::validate_failed_save;
pub use initiative::preview_initiative_circumstances;
pub use reaction_order::order_reaction_respondents;
use serde::{Deserialize, Serialize};
pub use validation::{validate_tactical_pending, validate_tactical_state};
pub use work_trace::tactical_frame_host_ordering;

pub const TACTICAL_EVENT_KIND: &str = "tactical.action_resolved";
pub const TACTICAL_EVENT_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum TacticalAction {
    ConcludeHostilities {
        cadence: AftermathCadence,
        ruling: String,
    },
    /// A deliberate, journaled transition for an already active legacy encounter.
    /// It is admitted only at an idle boundary and never alters accepted history.
    UpgradeExecution,
    Ready {
        trigger: ReadyTrigger,
        action: ReadyAction,
    },
    AbandonReady {
        actor: EntityId,
    },
    UnarmedStrike {
        target: EntityId,
    },
    FirstAid {
        target: EntityId,
        purpose: MedicinePurpose,
    },
    SubmitSavageAttacker {
        roll: SavageAttackerRoll,
    },
    SecondWind,
    DonShield {
        shield: ItemId,
        hand: Hand,
    },
    DoffShield,
    CreatureWeaponAttack {
        feature_id: String,
        choice: CreatureWeaponUseChoice,
    },
    CreatureArea {
        feature_id: String,
        aim: TacticalAreaAim,
        ordering: TacticalAreaOrdering,
    },
    CreatureAttack {
        target: EntityId,
        feature_id: String,
        weapon: Option<ItemId>,
    },
    ChooseLiquidLanding {
        choice: Option<LiquidLandingChoice>,
    },
    CastSpell {
        choice: SpellCastChoice,
        targets: SpellTargetChoice,
    },
    Move {
        path: Vec<TacticalMoveStep>,
    },
    DeclineOpportunity,
    OpportunityAttack {
        choice: TacticalMeleeChoice,
    },
    Attack {
        choice: WeaponUseChoice,
    },
    ChooseAttackKnockout {
        choice: KnockoutChoice,
    },
    ChooseAttackMastery {
        choice: WeaponMasteryChoice,
    },
    Establish {
        encounter: Box<TacticalEncounter>,
    },
    Begin {
        combatants: Vec<TacticalCombatant>,
        groups: Vec<InitiativeGroup>,
        #[serde(default, skip_serializing_if = "TacticalExecutionVersion::is_legacy")]
        execution: TacticalExecutionVersion,
    },
    SubmitRoll {
        result: RollResult,
    },
    SubmitRollWithInspiration {
        result: RollResult,
        die_index: usize,
        replacement: DieResult,
    },
    ProposeInitiativeTie {
        order: Vec<EntityId>,
    },
    AcceptInitiativeTie {
        total: i32,
    },
    ChooseTurnWork {
        occurrence: u16,
    },
    VoluntarilyFailSave,
    UseLegendaryResistance,
    DeclineLegendaryResistance,
    DeclineLegendaryAction,
    EndTurn,
    Dash {
        speed: DashSpeed,
    },
    Disengage,
    Dodge,
    StandProne,
    StartAttackAction,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TacticalOutcome {
    pub next_roll: Option<RollRequest>,
    pub awaiting_initiative_ties: bool,
    pub active_actor: Option<EntityId>,
    pub awaiting_turn_work: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TacticalEvent {
    pub meta: CommandMeta,
    pub action: TacticalAction,
    pub outcome: TacticalOutcome,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TacticalTransition {
    pub next_state: CampaignState,
    pub event: TacticalEvent,
}

fn invalid(message: &str) -> RulesError {
    RulesError::Invalid(message.into())
}
fn prerequisite(message: &str) -> RulesError {
    RulesError::Prerequisite(message.into())
}
fn privileged(meta: &CommandMeta) -> Result<(), RulesError> {
    if matches!(meta.issuer, CommandIssuer::Admin | CommandIssuer::System) && meta.actor.is_none() {
        Ok(())
    } else {
        Err(RulesError::Unauthorized)
    }
}
fn controller(state: &CampaignState, actor: EntityId) -> Option<PlayerId> {
    state
        .characters
        .values()
        .find(|c| {
            c.entity_id == actor
                && matches!(c.status, CharacterStatus::Active | CharacterStatus::Dead)
        })
        .and_then(|c| c.controlling_player_id)
        .or_else(|| {
            state
                .rules
                .as_ref()
                .and_then(|r| r.tactical_creatures.as_ref())
                .and_then(|c| c.runtime(actor))
                .and_then(|runtime| match runtime.controller {
                    CreatureController::Player(player) => Some(player),
                    _ => None,
                })
        })
}
fn authorize(state: &CampaignState, meta: &CommandMeta, actor: EntityId) -> Result<(), RulesError> {
    let accepted = match meta.issuer {
        CommandIssuer::Admin | CommandIssuer::System => {
            meta.actor.is_none() || meta.actor == Some(AgentRef::Entity(actor))
        }
        CommandIssuer::Player(id) => {
            controller(state, actor) == Some(id) && meta.actor == Some(AgentRef::Entity(actor))
        }
        CommandIssuer::Import => false,
    };
    if accepted {
        Ok(())
    } else {
        Err(RulesError::Unauthorized)
    }
}
fn encounter(state: &CampaignState) -> Result<&TacticalEncounter, RulesError> {
    state
        .encounter
        .as_ref()
        .ok_or_else(|| prerequisite("no tactical encounter"))
}
fn flow(state: &CampaignState) -> Result<&TacticalFlow, RulesError> {
    encounter(state)?
        .flow
        .as_ref()
        .ok_or_else(|| prerequisite("initiative has not begun"))
}
fn flow_mut(state: &mut CampaignState) -> Result<&mut TacticalFlow, RulesError> {
    state
        .encounter
        .as_mut()
        .and_then(|e| e.flow.as_mut())
        .ok_or_else(|| prerequisite("initiative has not begun"))
}
fn definitions() -> Result<crate::tactical_definitions::TacticalDefinitions, RulesError> {
    crate::tactical_definitions::TacticalDefinitions::from_json(
        crate::tactical_definitions::TACTICAL_DEFINITIONS_JSON,
    )
    .map_err(|e| RulesError::Incompatible(e.to_string()))
}

pub fn resolve_tactical(
    state: &CampaignState,
    meta: &CommandMeta,
    action: &TacticalAction,
    pack: &RulesPack,
) -> Result<TacticalTransition, RulesError> {
    resolve_with_policy(state, meta, action, pack, ExecutionPolicy::Live)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ExecutionPolicy {
    Live,
    Historical,
}

fn resolve_with_policy(
    state: &CampaignState,
    meta: &CommandMeta,
    action: &TacticalAction,
    pack: &RulesPack,
    policy: ExecutionPolicy,
) -> Result<TacticalTransition, RulesError> {
    if meta.campaign_id != state.campaign_id() {
        return Err(RulesError::Unauthorized);
    }
    if meta.expected_event_sequence != state.applied_event_sequence {
        return Err(RulesError::Stale);
    }
    crate::validate_state(state, pack)?;
    validate_tactical_state(state)?;
    if policy == ExecutionPolicy::Live {
        validate_live_execution(state, action)?;
    }
    let rules = state.rules.as_ref().ok_or(RulesError::Uninitialized)?;
    if rules.pending.is_some()
        && !matches!(
            action,
            TacticalAction::SubmitRoll { .. }
                | TacticalAction::SubmitSavageAttacker { .. }
                | TacticalAction::SubmitRollWithInspiration { .. }
                | TacticalAction::VoluntarilyFailSave
        )
    {
        return Err(RulesError::Pending);
    }
    let mut next = state.clone();
    match action {
        TacticalAction::ConcludeHostilities { cadence, ruling } => {
            aftermath::conclude(&mut next, meta, *cadence, ruling)?;
        }
        TacticalAction::UpgradeExecution => {
            privileged(meta)?;
            let current = flow(&next)?;
            if current.version != TacticalExecutionVersion::Legacy.flow_version()
                || current.phase != TacticalPhase::Active
                || current.resolution.is_some()
                || rules.pending.is_some()
            {
                return Err(prerequisite(
                    "execution upgrade requires a settled legacy encounter",
                ));
            }
            flow_mut(&mut next)?.version = TacticalExecutionVersion::ReactionsV1.flow_version();
        }
        TacticalAction::Ready { trigger, action } => {
            ready::declare(&mut next, meta, trigger, action)?;
        }
        TacticalAction::AbandonReady { actor } => {
            ready::abandon(&mut next, meta, *actor)?;
        }
        TacticalAction::UnarmedStrike { target } => {
            attacks::begin_unarmed(&mut next, meta, *target)?;
        }
        TacticalAction::SubmitSavageAttacker { roll } => {
            attacks::submit_savage(&mut next, meta, roll, pack)?;
        }
        TacticalAction::DonShield { shield, hand } => {
            shields::change(&mut next, meta, Some((*shield, *hand)), pack)?
        }
        TacticalAction::DoffShield => shields::change(&mut next, meta, None, pack)?,
        TacticalAction::CreatureWeaponAttack { feature_id, choice } => {
            attacks::begin_creature_weapon(&mut next, meta, feature_id, choice, pack)?
        }
        TacticalAction::CreatureArea {
            feature_id,
            aim,
            ordering,
        } => {
            areas::begin(&mut next, meta, feature_id, *aim, *ordering)?;
        }
        TacticalAction::CreatureAttack {
            target,
            feature_id,
            weapon,
        } => {
            attacks::begin_creature_attack(&mut next, meta, *target, feature_id, *weapon)?;
        }
        TacticalAction::ChooseLiquidLanding { choice } => {
            falling::choose(&mut next, meta, *choice)?
        }
        TacticalAction::CastSpell { choice, targets } => {
            casting::begin(&mut next, meta, choice, targets)?;
        }
        TacticalAction::Move { path } => movement::begin(&mut next, meta, path)?,
        TacticalAction::DeclineOpportunity => movement::decline(&mut next, meta)?,
        TacticalAction::OpportunityAttack { choice } => {
            let window = movement::selected_opportunity(&next)?.clone();
            attacks::begin_opportunity_attack(
                &mut next,
                meta,
                window.reactor,
                window.mover,
                choice,
                pack,
            )?;
        }
        TacticalAction::Attack { choice } => attacks::begin(&mut next, meta, choice, pack)?,
        TacticalAction::ChooseAttackKnockout { choice } => {
            attacks::choose_knockout(&mut next, meta, *choice)?
        }
        TacticalAction::ChooseAttackMastery { choice } => {
            attacks::choose_mastery(&mut next, meta, choice)?
        }
        TacticalAction::Establish {
            encounter: authored,
        } => {
            privileged(meta)?;
            if state.encounter.is_some() || rules.timing.is_some() || authored.flow.is_some() {
                return Err(prerequisite(
                    "an encounter already exists or carries unsolicited execution state",
                ));
            }
            let mut authored = authored.as_ref().clone();
            authored.origin = meta.clone();
            authored
                .validate(state)
                .map_err(|e| RulesError::Invalid(e.to_string()))?;
            next.encounter = Some(authored);
        }
        TacticalAction::Begin {
            combatants,
            groups,
            execution,
        } => {
            privileged(meta)?;
            if rules.entities.values().any(|e| {
                e.character_features
                    .as_ref()
                    .is_some_and(|f| f.inspiration_transfer_pending)
            }) {
                return Err(prerequisite(
                    "an Inspiration transfer awaits its controller before initiative",
                ));
            }
            if encounter(state)?.flow.is_some() || rules.timing.is_some() {
                return Err(prerequisite("initiative is already established"));
            }
            let initial = TacticalFlow {
                version: execution.flow_version(),
                origin: meta.clone(),
                combatants: combatants.clone(),
                initiative_groups: groups.clone(),
                initiative_decisions: vec![],
                phase: TacticalPhase::Initiative { next_group: 0 },
                budget: TacticalTurnBudget::default(),
                resolution: None,
                last_movement: None,
                dodges: vec![],
                save_decisions: vec![],
                ground_items: vec![],
                ready: vec![],
                aftermath: None,
            };
            next.encounter
                .as_mut()
                .ok_or_else(|| prerequisite("no encounter"))?
                .flow = Some(initial);
            validation::validate_groups(&next)?;
            let rules = next.rules.as_mut().ok_or(RulesError::Uninitialized)?;
            for c in combatants {
                if rules.entities[&c.actor].death.dead {
                    return Err(prerequisite("dead creatures cannot enter initiative"));
                }
                crate::kernel::interrupt_rest(rules, c.actor, next.clock.now);
                rules.completed_short_rests.retain(|id| *id != c.actor);
            }
            rules.permission = None;
            // Legacy/imported sheets can predate source-driven item drops. Activating
            // this encounter reconciles the current condition under this actual command;
            // it does not invent a prior injury or replace historical grant provenance.
            let unconscious = combatants
                .iter()
                .filter(|c| {
                    crate::active_conditions(rules, c.actor).contains(&Condition::Unconscious)
                })
                .map(|c| c.actor)
                .collect::<Vec<_>>();
            for actor in unconscious {
                crate::tactical_vitality_adapter::drop_held(&mut next, actor, meta)?;
            }
            initiative::issue(&mut next, meta)?;
        }
        TacticalAction::SubmitRoll { result } => {
            if flow(state)?.resolution.is_some() {
                continuations::submit(&mut next, meta, result, None)?;
            } else {
                initiative::submit(&mut next, meta, result)?;
            }
        }
        TacticalAction::SubmitRollWithInspiration {
            result,
            die_index,
            replacement,
        } => {
            if flow(state)?.resolution.is_some() {
                continuations::submit(&mut next, meta, result, Some((*die_index, *replacement)))?;
            } else {
                initiative::submit_with_inspiration(
                    &mut next,
                    meta,
                    result,
                    *die_index,
                    *replacement,
                )?;
            }
        }
        TacticalAction::ProposeInitiativeTie { order } => {
            initiative::propose_tie(&mut next, meta, order)?
        }
        TacticalAction::AcceptInitiativeTie { total } => {
            initiative::accept_tie(&mut next, meta, *total)?
        }
        TacticalAction::ChooseTurnWork { occurrence } => {
            turns::choose(&mut next, meta, *occurrence)?
        }
        TacticalAction::VoluntarilyFailSave => continuations::voluntarily_fail(&mut next, meta)?,
        TacticalAction::UseLegendaryResistance => failed_save::choose(&mut next, meta, true)?,
        TacticalAction::DeclineLegendaryResistance => failed_save::choose(&mut next, meta, false)?,
        TacticalAction::DeclineLegendaryAction => creature_bridge::decline(&mut next, meta)?,
        TacticalAction::SecondWind => second_wind::begin(&mut next, meta, pack)?,
        TacticalAction::FirstAid { target, purpose } => {
            medicine::begin(&mut next, meta, *target, *purpose)?;
        }
        TacticalAction::EndTurn
        | TacticalAction::Dash { .. }
        | TacticalAction::Disengage
        | TacticalAction::Dodge
        | TacticalAction::StandProne
        | TacticalAction::StartAttackAction => turns::core_action(&mut next, meta, action)?,
    }
    crate::validate_state(&next, pack)?;
    validate_tactical_state(&next)?;
    let rules = next.rules.as_ref().ok_or(RulesError::Uninitialized)?;
    let outcome = TacticalOutcome {
        awaiting_turn_work: next
            .encounter
            .as_ref()
            .and_then(|e| e.flow.as_ref())
            .and_then(|f| f.resolution.as_ref())
            .is_some_and(|r| r.pending.is_none()),
        next_roll: rules.pending.as_ref().map(|p| p.request.clone()),
        awaiting_initiative_ties: next
            .encounter
            .as_ref()
            .and_then(|e| e.flow.as_ref())
            .is_some_and(|f| matches!(f.phase, TacticalPhase::InitiativeTies { .. })),
        active_actor: rules
            .timing
            .as_ref()
            .and_then(|t| t.order.get(t.index))
            .map(|e| e.actor),
    };
    Ok(TacticalTransition {
        next_state: next,
        event: TacticalEvent {
            meta: meta.clone(),
            action: action.clone(),
            outcome,
        },
    })
}

pub fn replay_tactical(
    state: &CampaignState,
    event: &TacticalEvent,
    pack: &RulesPack,
) -> Result<TacticalTransition, RulesError> {
    let transition = resolve_with_policy(
        state,
        &event.meta,
        &event.action,
        pack,
        ExecutionPolicy::Historical,
    )?;
    if transition.event != *event {
        return Err(RulesError::ReplayMismatch);
    }
    Ok(transition)
}

fn validate_live_execution(
    state: &CampaignState,
    action: &TacticalAction,
) -> Result<(), RulesError> {
    if matches!(
        action,
        TacticalAction::Begin {
            execution: TacticalExecutionVersion::Legacy,
            ..
        }
    ) {
        return Err(prerequisite(
            "new initiative requires the current tactical executor",
        ));
    }
    let Some(current) = state
        .encounter
        .as_ref()
        .and_then(|encounter| encounter.flow.as_ref())
    else {
        return Ok(());
    };
    if current.version != TacticalExecutionVersion::Legacy.flow_version() {
        return Ok(());
    }
    // A saved old pause remains completable under its original semantics. Fresh
    // actions wait for the explicit idle upgrade; no source window is skipped by
    // selecting the old Begin wire shape through a new transport request.
    if matches!(
        action,
        TacticalAction::UpgradeExecution
            | TacticalAction::SubmitRoll { .. }
            | TacticalAction::SubmitSavageAttacker { .. }
            | TacticalAction::SubmitRollWithInspiration { .. }
            | TacticalAction::ProposeInitiativeTie { .. }
            | TacticalAction::AcceptInitiativeTie { .. }
            | TacticalAction::ChooseTurnWork { .. }
            | TacticalAction::VoluntarilyFailSave
            | TacticalAction::UseLegendaryResistance
            | TacticalAction::DeclineLegendaryResistance
            | TacticalAction::DeclineLegendaryAction
            | TacticalAction::DeclineOpportunity
            | TacticalAction::OpportunityAttack { .. }
            | TacticalAction::ChooseAttackKnockout { .. }
            | TacticalAction::ChooseAttackMastery { .. }
            | TacticalAction::ChooseLiquidLanding { .. }
    ) {
        return Ok(());
    }
    Err(prerequisite(
        "this legacy encounter must finish pending work and upgrade before a new action",
    ))
}
