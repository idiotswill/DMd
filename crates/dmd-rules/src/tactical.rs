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
pub(crate) mod grapple;
mod hit_reactions;
mod initiative;
mod medicine;
mod missiles;
mod movement;
mod reaction_order;
mod ready;
mod release;
mod second_wind;
mod shields;
mod shove;
mod turn_validation;
mod turns;
pub(crate) mod validation;
mod work_trace;
use crate::{ResolveRoll, RulesError, RulesPack};
pub use aftermath::require_aftermath_session_boundary;
pub use attacks::savage_attacker_dice;
pub(crate) use attacks::savage_attacker_dice_with_read;
use dmd_domain::*;
pub use failed_save::validate_failed_save;
pub use hit_reactions::shield_choices;
pub(crate) use hit_reactions::shield_choices_with_read;
pub use initiative::preview_initiative_circumstances;
pub use reaction_order::order_reaction_respondents;
pub use release::{
    EncounterReleaseReadiness, MAX_RELEASE_DEPENDENCIES, encounter_release_preflight,
    require_finished_encounter, retained_encounter_dependencies,
};
use serde::{Deserialize, Serialize};
pub use validation::{validate_tactical_pending, validate_tactical_state};
pub use work_trace::tactical_frame_host_ordering;

pub const TACTICAL_EVENT_KIND: &str = "tactical.action_resolved";
pub const TACTICAL_EVENT_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum TacticalAction {
    /// Retire only fully settled, explicitly concluded flow 5 timing.
    FinishEncounter,
    ConcludeHostilities {
        cadence: AftermathCadence,
        ruling: String,
    },
    /// A deliberate, journaled transition for an already active legacy encounter.
    /// It is admitted only at an idle boundary and never alters accepted history.
    UpgradeExecution,
    /// Unlike the historical unit command, this retains the exact new executor.
    UpgradeExecutionTo {
        execution: TacticalExecutionVersion,
    },
    RespondToHit {
        window: TacticalWorkKey,
        accept: bool,
    },
    OrderHitResponses {
        window: TacticalWorkKey,
        instruction: TacticalReactionOrdering,
    },
    DelegateHitResponses {
        window: TacticalWorkKey,
    },
    CastHitShield {
        window: TacticalWorkKey,
        choice: SpellCastChoice,
    },
    DeclineSelectedHitShield {
        window: TacticalWorkKey,
    },
    RespondToMissile {
        window: TacticalWorkKey,
        actor: EntityId,
        accept: bool,
    },
    OrderMissileResponses {
        window: TacticalWorkKey,
        instruction: TacticalReactionOrdering,
    },
    DelegateMissileResponses {
        window: TacticalWorkKey,
    },
    CastMissileShield {
        window: TacticalWorkKey,
        choice: SpellCastChoice,
    },
    DeclineSelectedMissileShield {
        window: TacticalWorkKey,
    },
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
    Shove {
        target: EntityId,
    },
    ChooseShoveSave {
        ability: ShoveSaveAbility,
    },
    ChooseShoveOutcome {
        choice: ShoveChoice,
    },
    RuleShovePush {
        ruling: ShoveGeometryRuling,
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
    MoveSelfOnly {
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
    Grapple {
        target: EntityId,
        hand: Hand,
        before_change: Option<AttackEquipmentOperation>,
    },
    ChooseGrappleSave {
        grip: GrappleId,
        ability: GrappleSaveAbility,
    },
    ApplyGrappleAfterEquipment {
        grip: GrappleId,
        work: TacticalWorkKey,
        operation: AttackEquipmentOperation,
    },
    DeclineGrappleAfterEquipment {
        grip: GrappleId,
        work: TacticalWorkKey,
    },
    WithdrawGrapple {
        grip: GrappleId,
    },
    EscapeGrapple {
        grip: GrappleId,
        choice: GrappleEscapeChoice,
    },
    ReleaseGrapple {
        grip: GrappleId,
    },
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
fn definitions() -> Result<&'static crate::tactical_definitions::TacticalDefinitions, RulesError> {
    crate::tactical_definitions::bundled_tactical_definitions()
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
    let mut next = state.clone();
    let event = apply_table_with_context(
        state,
        &mut next,
        meta,
        action,
        pack,
        policy == ExecutionPolicy::Historical,
        &mut grapple::execution::ExecutionContext::ordinary(),
    )?;
    Ok(TacticalTransition {
        next_state: next,
        event,
    })
}

pub(crate) fn apply_table_with_context(
    state: &CampaignState,
    next: &mut CampaignState,
    meta: &CommandMeta,
    action: &TacticalAction,
    pack: &RulesPack,
    historical: bool,
    execution: &mut grapple::execution::ExecutionContext<'_>,
) -> Result<TacticalEvent, RulesError> {
    if meta.campaign_id != state.campaign_id() {
        return Err(RulesError::Unauthorized);
    }
    if meta.expected_event_sequence != state.applied_event_sequence {
        return Err(RulesError::Stale);
    }
    execution.admit_table(next, action)?;
    crate::kernel::validate_state_with_read(&execution.read(next)?, pack)?;
    validation::validate_tactical_state_with_read(&execution.read(next)?)?;
    if !historical {
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
        && !(execution.is_owned()
            && matches!(
                action,
                TacticalAction::ReleaseGrapple { .. } | TacticalAction::WithdrawGrapple { .. }
            ))
    {
        return Err(RulesError::Pending);
    }
    dispatch(state, next, meta, action, pack, execution)?;
    crate::kernel::validate_state_with_read(&execution.read(next)?, pack)?;
    validation::validate_tactical_state_with_read(&execution.read(next)?)?;
    Ok(TacticalEvent {
        meta: meta.clone(),
        action: action.clone(),
        outcome: outcome(next)?,
    })
}

fn dispatch(
    state: &CampaignState,
    next: &mut CampaignState,
    meta: &CommandMeta,
    action: &TacticalAction,
    pack: &RulesPack,
    execution: &mut grapple::execution::ExecutionContext<'_>,
) -> Result<(), RulesError> {
    let rules = state.rules.as_ref().ok_or(RulesError::Uninitialized)?;
    match action {
        TacticalAction::Grapple {
            target,
            hand,
            before_change,
        } => grapple::begin_with_context(
            next,
            meta,
            *target,
            *hand,
            *before_change,
            pack,
            execution,
        )?,
        TacticalAction::ChooseGrappleSave { grip, ability } => {
            grapple::choose_save_with_context(next, meta, *grip, *ability, execution)?
        }
        TacticalAction::ApplyGrappleAfterEquipment {
            grip,
            work,
            operation,
        } => grapple::choose_equipment_with_context(
            next,
            meta,
            *grip,
            *work,
            Some(*operation),
            execution,
        )?,
        TacticalAction::DeclineGrappleAfterEquipment { grip, work } => {
            grapple::choose_equipment_with_context(next, meta, *grip, *work, None, execution)?
        }
        TacticalAction::WithdrawGrapple { grip } => {
            grapple::withdraw_with_context(next, meta, *grip, execution)?
        }
        TacticalAction::EscapeGrapple { grip, choice } => {
            grapple::begin_escape_with_context(next, meta, *grip, *choice, execution)?
        }
        TacticalAction::ReleaseGrapple { grip } => {
            grapple::release_with_context(next, meta, *grip, execution)?
        }
        TacticalAction::RespondToMissile {
            window,
            actor,
            accept,
        } => {
            missiles::respond(next, meta, *window, *actor, *accept, execution)?;
        }
        TacticalAction::OrderMissileResponses {
            window,
            instruction,
        } => {
            missiles::order(next, meta, *window, instruction, execution)?;
        }
        TacticalAction::DelegateMissileResponses { window } => {
            missiles::delegate(next, meta, *window)?;
        }
        TacticalAction::CastMissileShield { window, choice } => {
            missiles::cast(next, meta, *window, choice, execution)?;
        }
        TacticalAction::DeclineSelectedMissileShield { window } => {
            missiles::decline(next, meta, *window, execution)?;
        }
        TacticalAction::RespondToHit { window, accept } => {
            hit_reactions::respond_with_context(next, meta, *window, *accept, execution)?;
        }
        TacticalAction::OrderHitResponses {
            window,
            instruction,
        } => {
            hit_reactions::order_with_context(next, meta, *window, instruction, execution)?;
        }
        TacticalAction::DelegateHitResponses { window } => {
            hit_reactions::delegate(next, meta, *window)?;
        }
        TacticalAction::CastHitShield { window, choice } => {
            hit_reactions::cast_with_context(next, meta, *window, choice, execution)?;
        }
        TacticalAction::DeclineSelectedHitShield { window } => {
            hit_reactions::decline_with_context(next, meta, *window, execution)?;
        }
        TacticalAction::ConcludeHostilities { cadence, ruling } => {
            aftermath::conclude(next, meta, *cadence, ruling)?;
        }
        TacticalAction::FinishEncounter => release::finish(next, meta)?,
        TacticalAction::UpgradeExecution => {
            privileged(meta)?;
            let current = flow(next)?;
            if current.version != TacticalExecutionVersion::Legacy.flow_version()
                || current.phase != TacticalPhase::Active
                || current.resolution.is_some()
                || rules.pending.is_some()
            {
                return Err(prerequisite(
                    "execution upgrade requires a settled legacy encounter",
                ));
            }
            flow_mut(next)?.version = TacticalExecutionVersion::ReactionsV1.flow_version();
        }
        TacticalAction::UpgradeExecutionTo { execution } => {
            privileged(meta)?;
            let current = flow(next)?;
            if !matches!(
                execution,
                TacticalExecutionVersion::ShieldHitV1
                    | TacticalExecutionVersion::ShieldMissileV1
                    | TacticalExecutionVersion::EncounterReleaseV1
            ) || current.version >= execution.flow_version()
                || current.phase != TacticalPhase::Active
                || current.resolution.is_some()
                || !current.ready.is_empty()
                || rules.pending.is_some()
            {
                return Err(prerequisite(
                    "execution upgrade requires a settled encounter without held actions",
                ));
            }
            flow_mut(next)?.version = execution.flow_version();
        }
        TacticalAction::Ready { trigger, action } => {
            ready::declare(next, meta, trigger, action)?;
        }
        TacticalAction::AbandonReady { actor } => {
            ready::abandon(next, meta, *actor)?;
        }
        TacticalAction::UnarmedStrike { target } => {
            attacks::begin_unarmed_with_context(next, meta, *target, execution)?;
        }
        TacticalAction::Shove { target } => shove::begin(next, meta, *target, execution)?,
        TacticalAction::ChooseShoveSave { ability } => {
            shove::choose_save(next, meta, *ability, execution)?
        }
        TacticalAction::ChooseShoveOutcome { choice } => {
            shove::choose_outcome(next, meta, choice, execution)?
        }
        TacticalAction::RuleShovePush { ruling } => {
            shove::rule_push(next, meta, *ruling, execution)?
        }
        TacticalAction::SubmitSavageAttacker { roll } => {
            attacks::submit_savage_with_context(next, meta, roll, pack, execution)?;
        }
        TacticalAction::DonShield { shield, hand } => {
            shields::change_with_context(next, meta, Some((*shield, *hand)), pack, execution)?
        }
        TacticalAction::DoffShield => {
            shields::change_with_context(next, meta, None, pack, execution)?
        }
        TacticalAction::CreatureWeaponAttack { feature_id, choice } => {
            attacks::begin_creature_weapon_with_context(
                next, meta, feature_id, choice, pack, execution,
            )?
        }
        TacticalAction::CreatureArea {
            feature_id,
            aim,
            ordering,
        } => {
            areas::begin(next, meta, feature_id, *aim, *ordering, execution)?;
        }
        TacticalAction::CreatureAttack {
            target,
            feature_id,
            weapon,
        } => {
            attacks::begin_creature_attack_with_context(
                next, meta, *target, feature_id, *weapon, execution,
            )?;
        }
        TacticalAction::ChooseLiquidLanding { choice } => {
            falling::choose(next, meta, *choice, execution)?
        }
        TacticalAction::CastSpell { choice, targets } => {
            casting::begin(next, meta, choice, targets, execution)?;
        }
        TacticalAction::Move { path } => {
            movement::begin(next, meta, path, movement::Intent::Ordinary, execution)?
        }
        TacticalAction::MoveSelfOnly { path } => {
            movement::begin(next, meta, path, movement::Intent::SelfOnly, execution)?
        }
        TacticalAction::DeclineOpportunity => movement::decline(next, meta, execution)?,
        TacticalAction::OpportunityAttack { choice } => {
            let window = movement::selected_opportunity(next)?.clone();
            attacks::begin_opportunity_attack(
                next,
                meta,
                window.reactor,
                window.mover,
                choice,
                pack,
                execution,
            )?;
        }
        TacticalAction::Attack { choice } => {
            attacks::begin_with_context(next, meta, choice, pack, execution)?
        }
        TacticalAction::ChooseAttackKnockout { choice } => {
            attacks::choose_knockout_with_context(next, meta, *choice, execution)?
        }
        TacticalAction::ChooseAttackMastery { choice } => {
            attacks::choose_mastery_with_context(next, meta, choice, execution)?
        }
        TacticalAction::Establish {
            encounter: authored,
        } => {
            privileged(meta)?;
            if rules.timing.is_some() || authored.flow.is_some() {
                return Err(prerequisite(
                    "an encounter already exists or carries unsolicited execution state",
                ));
            }
            let replacement = state.encounter.is_some();
            if replacement {
                require_finished_encounter(state)?;
                let required = retained_encounter_dependencies(state)?;
                if authored.participants.len() > MAX_RELEASE_DEPENDENCIES
                    || required
                        .iter()
                        .any(|actor| authored.participant(*actor).is_none())
                    || state.encounter_history.as_ref().is_none_or(|history| {
                        history.contains_encounter(authored.id)
                            || history
                                .spaces
                                .iter()
                                .any(|space| space.scene_id == authored.scene_id)
                    })
                {
                    return Err(prerequisite(
                        "replacement must preserve timing dependencies with new encounter and scene identities",
                    ));
                }
            }
            let mut authored = authored.as_ref().clone();
            authored.origin = meta.clone();
            if replacement {
                let scene = next
                    .scenes
                    .get(&authored.scene_id)
                    .ok_or_else(|| invalid("replacement scene is absent"))?;
                if scene.status != SceneStatus::Closed {
                    return Err(prerequisite(
                        "stage the replacement scene before activating it",
                    ));
                }
                let location = scene.location_id;
                for participant in &authored.participants {
                    next.entities
                        .get_mut(&participant.entity_id)
                        .ok_or_else(|| invalid("replacement actor is absent"))?
                        .location_id = Some(location);
                }
            }
            crate::spatial::validate_source_placement(&authored, next)
                .map_err(|e| RulesError::Prerequisite(e.to_string()))?;
            authored
                .validate(next)
                .map_err(|e| RulesError::Invalid(e.to_string()))?;
            next.encounter = Some(authored);
            if replacement {
                let scene_id = encounter(next)?.scene_id;
                next.scenes
                    .get_mut(&scene_id)
                    .ok_or_else(|| invalid("replacement scene is absent"))?
                    .status = SceneStatus::Active;
                execution.observe_encounter_replacement(next, meta)?;
            }
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
            validation::validate_groups(next)?;
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
                crate::tactical_vitality_adapter::drop_held(next, actor, meta)?;
            }
            initiative::issue(next, meta)?;
        }
        TacticalAction::SubmitRoll { result } => {
            if flow(state)?.resolution.is_some() {
                continuations::submit_with_context(next, meta, result, None, execution)?;
            } else {
                initiative::submit(next, meta, result, execution)?;
            }
        }
        TacticalAction::SubmitRollWithInspiration {
            result,
            die_index,
            replacement,
        } => {
            if flow(state)?.resolution.is_some() {
                continuations::submit_with_context(
                    next,
                    meta,
                    result,
                    Some((*die_index, *replacement)),
                    execution,
                )?;
            } else {
                initiative::submit_with_inspiration(
                    next,
                    meta,
                    result,
                    *die_index,
                    *replacement,
                    execution,
                )?;
            }
        }
        TacticalAction::ProposeInitiativeTie { order } => {
            initiative::propose_tie(next, meta, order, execution)?
        }
        TacticalAction::AcceptInitiativeTie { total } => {
            initiative::accept_tie(next, meta, *total, execution)?
        }
        TacticalAction::ChooseTurnWork { occurrence } => {
            turns::choose_with_context(next, meta, *occurrence, execution)?
        }
        TacticalAction::VoluntarilyFailSave => {
            continuations::voluntarily_fail_with_context(next, meta, execution)?
        }
        TacticalAction::UseLegendaryResistance => {
            failed_save::choose_with_context(next, meta, true, execution)?
        }
        TacticalAction::DeclineLegendaryResistance => {
            failed_save::choose_with_context(next, meta, false, execution)?
        }
        TacticalAction::DeclineLegendaryAction => creature_bridge::decline(next, meta, execution)?,
        TacticalAction::SecondWind => second_wind::begin(next, meta, pack, execution)?,
        TacticalAction::FirstAid { target, purpose } => {
            medicine::begin(next, meta, *target, *purpose, execution)?;
        }
        TacticalAction::EndTurn
        | TacticalAction::Dash { .. }
        | TacticalAction::Disengage
        | TacticalAction::Dodge
        | TacticalAction::StandProne
        | TacticalAction::StartAttackAction => {
            turns::core_action_with_context(next, meta, action, execution)?
        }
    }
    Ok(())
}

fn outcome(next: &CampaignState) -> Result<TacticalOutcome, RulesError> {
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
    Ok(outcome)
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
    if matches!(action, TacticalAction::UpgradeExecutionTo { execution }
        if *execution != TacticalExecutionVersion::EncounterReleaseV1)
    {
        return Err(prerequisite(
            "a new targeted upgrade requires the current tactical executor",
        ));
    }
    if matches!(action, TacticalAction::Begin { execution, .. }
        if *execution != TacticalExecutionVersion::EncounterReleaseV1)
    {
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
    if current.version == TacticalExecutionVersion::EncounterReleaseV1.flow_version() {
        return Ok(());
    }
    // A saved old pause remains completable under its original semantics. Fresh
    // actions wait for the explicit idle upgrade; no source window is skipped by
    // selecting the old Begin wire shape through a new transport request.
    if matches!(
        action,
        TacticalAction::UpgradeExecution
            | TacticalAction::UpgradeExecutionTo { .. }
            | TacticalAction::AbandonReady { .. }
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
            | TacticalAction::ChooseShoveSave { .. }
            | TacticalAction::ChooseShoveOutcome { .. }
            | TacticalAction::RuleShovePush { .. }
            | TacticalAction::ChooseLiquidLanding { .. }
            | TacticalAction::RespondToHit { .. }
            | TacticalAction::OrderHitResponses { .. }
            | TacticalAction::DelegateHitResponses { .. }
            | TacticalAction::CastHitShield { .. }
            | TacticalAction::DeclineSelectedHitShield { .. }
            | TacticalAction::RespondToMissile { .. }
            | TacticalAction::OrderMissileResponses { .. }
            | TacticalAction::DelegateMissileResponses { .. }
            | TacticalAction::CastMissileShield { .. }
            | TacticalAction::DeclineSelectedMissileShield { .. }
    ) {
        return Ok(());
    }
    Err(prerequisite(
        "this legacy encounter must finish pending work and upgrade before a new action",
    ))
}
