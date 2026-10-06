//! Explicit table access to genuine source owners. Ownership remains in source runtime.
use crate::tactical_creatures::*;
use dmd_domain::*;

pub fn enabled(state: &CampaignState) -> bool {
    state
        .table
        .as_ref()
        .is_some_and(|table| table.source_actor_access.is_some())
}

pub fn presentation_version(state: &CampaignState) -> u32 {
    if super::grapple_enabled(state) {
        3
    } else if enabled(state) {
        2
    } else {
        1
    }
}

/// Reject all unresolved/held work, not merely the selected creature's local routine.
pub fn settled(state: &CampaignState) -> Result<(), String> {
    let table = state.table.as_ref().ok_or("This campaign has no table.")?;
    if table.pending.is_some()
        || table.roll_context.is_some()
        || state.rules.as_ref().is_some_and(|rules| {
            rules.pending.is_some()
                || rules
                    .tactical_effects
                    .as_ref()
                    .is_some_and(|effects| !effects.pending.is_empty())
                || rules.tactical_creatures.as_ref().is_some_and(|creatures| {
                    creatures.runtime.iter().any(|runtime| {
                        runtime.routine.is_some()
                            || runtime.recharge.iter().any(|entry| entry.pending.is_some())
                    })
                })
        })
        || state
            .encounter
            .as_ref()
            .and_then(|encounter| encounter.flow.as_ref())
            .is_some_and(|flow| {
                flow.resolution.is_some()
                    || !flow.ready.is_empty()
                    || !matches!(flow.phase, TacticalPhase::Active | TacticalPhase::Finished)
            })
    {
        return Err("Finish pending work and release or abandon held actions before changing source control.".into());
    }
    Ok(())
}

fn host_context(state: &CampaignState, meta: &CommandMeta) -> Result<(), String> {
    let table = state.table.as_ref().ok_or("This campaign has no table.")?;
    if meta.issuer != CommandIssuer::Admin
        || meta.actor.is_some()
        || meta.campaign_id != state.campaign_id()
        || meta.expected_event_sequence != state.applied_event_sequence
        || meta.session_id
            != table
                .active_session
                .as_ref()
                .map(|session| session.session_id)
    {
        return Err("Source control requires the current host and session.".into());
    }
    settled(state)
}

pub fn adoptions(state: &CampaignState) -> Result<Vec<TableSourceAdoption>, String> {
    let mut result = Vec::new();
    if let Some(creatures) = state
        .rules
        .as_ref()
        .and_then(|rules| rules.tactical_creatures.as_ref())
    {
        for runtime in &creatures.runtime {
            if let CreatureController::Player(player_id) = runtime.controller {
                let profile = creatures
                    .profile(runtime.actor)
                    .ok_or("Source profile is absent.")?;
                source_for_profile(profile).map_err(|error| error.to_string())?;
                result.push(TableSourceAdoption {
                    actor: runtime.actor,
                    player_id,
                    source: profile.source.clone(),
                    profile_origin: profile.origin.clone(),
                    control_origin: runtime.control_origin.clone(),
                });
            }
        }
    }
    result.sort_by_key(|entry| entry.actor.0);
    Ok(result)
}

pub fn activate(
    state: &CampaignState,
    meta: &CommandMeta,
    adopted: &[TableSourceAdoption],
) -> Result<CampaignState, String> {
    host_context(state, meta)?;
    if enabled(state) {
        return Err("Source creature control is already enabled.".into());
    }
    if adopted != adoptions(state)? {
        return Err("Review the complete current source-owner set before enabling control.".into());
    }
    if state.encounter.as_ref().is_some_and(|encounter| {
        encounter.flow.is_some()
            && adopted.iter().any(|adoption| {
                encounter.participant(adoption.actor).is_some()
                    && !present_player(state, meta, adoption.player_id)
            })
    }) {
        return Err("Every adopted source controller in this encounter must be present.".into());
    }
    let mut next = state.clone();
    next.table
        .as_mut()
        .ok_or("This campaign has no table.")?
        .source_actor_access = Some(Box::new(TableSourceActorAccess {
        version: TableSourceAccessVersion::SourceActorsV1,
        origin: meta.clone(),
        adopted: adopted.to_vec(),
    }));
    Ok(next)
}

pub(crate) fn assign(
    state: &CampaignState,
    meta: &CommandMeta,
    actor: EntityId,
    controller: CreatureController,
) -> Result<CampaignState, String> {
    host_context(state, meta)?;
    if !enabled(state) {
        return Err("Enable source creature control before assigning an actor.".into());
    }
    let current = state
        .rules
        .as_ref()
        .and_then(|rules| rules.tactical_creatures.as_ref())
        .ok_or("No source creatures are available.")?;
    let profile = current
        .profile(actor)
        .ok_or("Select an authenticated source creature.")?;
    source_for_profile(profile).map_err(|error| error.to_string())?;
    if let CreatureController::Player(player) = controller
        && state.encounter.as_ref().is_some_and(|encounter| {
            encounter.flow.is_some() && encounter.participant(actor).is_some()
        })
        && !present_player(state, meta, player)
    {
        return Err(
            "The new source controller must be present in this encounter's session.".into(),
        );
    }
    let transition = apply_creature_schedule(
        state,
        current,
        meta,
        &CreatureScheduleOperation::SetController { actor, controller },
    )
    .map_err(|error| error.to_string())?;
    let mut next = state.clone();
    next.rules
        .as_mut()
        .ok_or("Mechanical state is absent.")?
        .tactical_creatures = Some(transition.next);
    Ok(next)
}

pub fn owns_source(state: &CampaignState, player: PlayerId, actor: EntityId) -> bool {
    enabled(state)
        && state
            .rules
            .as_ref()
            .and_then(|rules| rules.tactical_creatures.as_ref())
            .is_some_and(|creatures| {
                creatures
                    .runtime(actor)
                    .is_some_and(|runtime| runtime.controller == CreatureController::Player(player))
                    && creatures
                        .profile(actor)
                        .is_some_and(|profile| source_for_profile(profile).is_ok())
            })
}

pub fn attending_source(state: &CampaignState, meta: &CommandMeta) -> Result<EntityId, String> {
    let (CommandIssuer::Player(player), Some(AgentRef::Entity(actor))) = (meta.issuer, meta.actor)
    else {
        return Err("Select the attending controller and their source creature.".into());
    };
    if !present_player(state, meta, player) || !owns_source(state, player, actor) {
        return Err("Select the attending controller and their source creature.".into());
    }
    Ok(actor)
}

fn present_player(state: &CampaignState, meta: &CommandMeta, player: PlayerId) -> bool {
    state
        .table
        .as_ref()
        .and_then(|table| table.active_session.as_ref())
        .is_some_and(|session| {
            Some(session.session_id) == meta.session_id
                && session.participants.iter().any(|participant| {
                    participant.player_id == player
                        && participant.attendance == AttendanceStatus::Present
                })
        })
}

pub fn has_owned_source(state: &CampaignState, player: PlayerId) -> bool {
    state
        .rules
        .as_ref()
        .and_then(|rules| rules.tactical_creatures.as_ref())
        .is_some_and(|creatures| {
            creatures
                .runtime
                .iter()
                .any(|runtime| owns_source(state, player, runtime.actor))
        })
}

/// Version-two table input assigns voluntary source decisions to their actual
/// controller. The generic trusted rules API remains unchanged for old replay.
pub fn authorize_tactical(
    state: &CampaignState,
    meta: &CommandMeta,
    action: &crate::tactical::TacticalAction,
) -> Result<(), String> {
    use crate::tactical::TacticalAction as A;
    if !matches!(meta.issuer, CommandIssuer::Admin | CommandIssuer::System) {
        return Ok(());
    }
    if !enabled(state) {
        // A new executor cannot strand an existing Player-owned source behind
        // a transport that has never been explicitly activated for its owner.
        // Keep all historical Begin and unit 1-to-2 upgrade semantics intact.
        let owned = |actor| {
            state
                .rules
                .as_ref()
                .and_then(|rules| rules.tactical_creatures.as_ref())
                .and_then(|creatures| creatures.runtime(actor))
                .is_some_and(|runtime| matches!(runtime.controller, CreatureController::Player(_)))
        };
        let needs_access = match action {
            A::Begin {
                execution:
                    TacticalExecutionVersion::ShieldHitV1
                    | TacticalExecutionVersion::ShieldMissileV1
                    | TacticalExecutionVersion::EncounterReleaseV1,
                combatants,
                ..
            } => combatants.iter().any(|combatant| owned(combatant.actor)),
            A::UpgradeExecutionTo {
                execution:
                    TacticalExecutionVersion::ShieldHitV1
                    | TacticalExecutionVersion::ShieldMissileV1
                    | TacticalExecutionVersion::EncounterReleaseV1,
            } => state.encounter.as_ref().is_some_and(|encounter| {
                encounter
                    .participants
                    .iter()
                    .any(|participant| owned(participant.entity_id))
            }),
            _ => false,
        };
        return if needs_access {
            Err(
                "Enable source creature control before starting or upgrading this encounter."
                    .into(),
            )
        } else {
            Ok(())
        };
    }
    let rules = state.rules.as_ref().ok_or("Mechanical state is absent.")?;
    if let A::Begin { combatants, .. } = action {
        for combatant in combatants {
            if let Some(CreatureController::Player(player)) = rules
                .tactical_creatures
                .as_ref()
                .and_then(|creatures| creatures.runtime(combatant.actor))
                .map(|runtime| runtime.controller)
                && !present_player(state, meta, player)
            {
                return Err(
                    "Every source controller must be present before initiative begins.".into(),
                );
            }
        }
    }
    let resolution = state
        .encounter
        .as_ref()
        .and_then(|encounter| encounter.flow.as_ref())
        .and_then(|flow| flow.resolution.as_deref());
    let active = rules
        .timing
        .as_ref()
        .and_then(|timing| timing.order.get(timing.index))
        .map(|entry| entry.actor);
    let actor = match action {
        A::Grapple { .. }
        | A::ChooseGrappleSave { .. }
        | A::ApplyGrappleAfterEquipment { .. }
        | A::DeclineGrappleAfterEquipment { .. }
        | A::WithdrawGrapple { .. }
        | A::EscapeGrapple { .. }
        | A::ReleaseGrapple { .. }
            if !super::grapple_enabled(state) =>
        {
            return Err("Grapple commands are not enabled.".into());
        }
        A::Grapple { .. } | A::EscapeGrapple { .. } => active,
        A::ChooseGrappleSave { grip, .. } => resolution
            .and_then(|r| r.grapple.as_ref())
            .and_then(|c| c.activity.as_ref())
            .and_then(|activity| match activity {
                GrappleActivity::Attempt(attempt) if attempt.declaration.id == *grip => {
                    Some(attempt.declaration.target)
                }
                _ => None,
            }),
        A::ApplyGrappleAfterEquipment { grip, .. }
        | A::DeclineGrappleAfterEquipment { grip, .. }
        | A::WithdrawGrapple { grip } => resolution
            .and_then(|r| r.grapple.as_ref())
            .and_then(|c| c.activity.as_ref())
            .and_then(|activity| match activity {
                GrappleActivity::Attempt(attempt) if attempt.declaration.id == *grip => {
                    Some(attempt.declaration.grappler)
                }
                _ => None,
            }),
        A::ReleaseGrapple { grip } => rules
            .tactical_grapples
            .as_ref()
            .and_then(|live| live.grip(*grip))
            .map(|live| live.declaration.grappler),
        // These retain source monster/mixed-tie and explicitly delegated ordering.
        A::Establish { .. }
        | A::Begin { .. }
        | A::ConcludeHostilities { .. }
        | A::FinishEncounter
        | A::UpgradeExecution
        | A::UpgradeExecutionTo { .. }
        | A::ProposeInitiativeTie { .. }
        | A::AcceptInitiativeTie { .. } => None,
        A::RespondToHit { .. } | A::CastHitShield { .. } | A::DeclineSelectedHitShield { .. } => {
            resolution
                .and_then(|resolution| resolution.hit_review.as_ref())
                .and_then(|hit| hit.respondent.as_ref())
                .map(|respondent| respondent.actor)
        }
        A::DelegateHitResponses { .. } => resolution.map(|resolution| resolution.turn_actor),
        A::RespondToMissile { window, actor, .. } => resolution
            .filter(|resolution| resolution.origin.id == window.resolution)
            .and_then(|resolution| {
                resolution
                    .missiles
                    .iter()
                    .find(|missile| missile.work.occurrence == window.occurrence)
            })
            .and_then(|missile| {
                missile
                    .respondents
                    .iter()
                    .find(|target| target.response.actor == *actor)
            })
            .map(|target| target.response.actor),
        A::CastMissileShield { window, .. } | A::DeclineSelectedMissileShield { window } => {
            resolution
                .filter(|resolution| resolution.origin.id == window.resolution)
                .and_then(|resolution| {
                    resolution
                        .missiles
                        .iter()
                        .find(|missile| missile.work.occurrence == window.occurrence)
                })
                .and_then(|missile| match missile.stage {
                    TacticalMissileStage::Selected { respondent } => {
                        missile.respondents.get(usize::from(respondent))
                    }
                    _ => None,
                })
                .map(|target| target.response.actor)
        }
        A::DelegateMissileResponses { .. } => resolution.map(|resolution| resolution.turn_actor),
        A::OrderMissileResponses { window, .. } => resolution.and_then(|resolution| {
            let delegated = resolution.origin.id == window.resolution
                && resolution
                    .missiles
                    .iter()
                    .find(|missile| missile.work.occurrence == window.occurrence)
                    .is_some_and(|missile| missile.delegated_by.is_some());
            (!delegated).then_some(resolution.turn_actor)
        }),
        A::OrderHitResponses { .. } => resolution.and_then(|resolution| {
            (!resolution
                .hit_review
                .as_ref()
                .is_some_and(|hit| hit.delegated_by.is_some()))
            .then_some(resolution.turn_actor)
        }),
        A::ChooseTurnWork { .. } => match resolution {
            Some(resolution)
                if !crate::tactical::tactical_frame_host_ordering(resolution)
                    .map_err(|error| error.to_string())? =>
            {
                Some(resolution.turn_actor)
            }
            _ => None,
        },
        A::SubmitRoll { .. }
        | A::SubmitRollWithInspiration { .. }
        | A::SubmitSavageAttacker { .. } => rules
            .pending
            .as_ref()
            .filter(|pending| pending.request.visibility == RollVisibility::Public)
            .and_then(|pending| pending.request.roller),
        A::VoluntarilyFailSave => rules
            .pending
            .as_ref()
            .and_then(|pending| pending.request.roller),
        A::UseLegendaryResistance | A::DeclineLegendaryResistance => resolution
            .and_then(|resolution| resolution.failed_save.as_ref())
            .map(|failed| failed.pending.key.subject),
        A::DeclineLegendaryAction => resolution
            .and_then(|resolution| resolution.legendary_window.as_ref())
            .and_then(|window| match window.work.kind {
                TacticalWorkKind::LegendaryWindow { actor } => Some(actor),
                _ => None,
            }),
        A::ChooseAttackKnockout { .. } | A::ChooseAttackMastery { .. } => resolution
            .and_then(|resolution| resolution.attack.as_ref())
            .map(|attack| attack.actor),
        A::ChooseShoveSave { .. } => resolution.and_then(|r| r.shove.as_ref()).map(|s| s.target),
        A::ChooseShoveOutcome { .. } => resolution.and_then(|r| r.shove.as_ref()).map(|s| s.actor),
        A::RuleShovePush { .. } => None,
        A::DeclineOpportunity | A::OpportunityAttack { .. } => resolution
            .and_then(|resolution| resolution.movement.as_ref())
            .and_then(|movement| movement.opportunity.as_ref())
            .map(|window| window.reactor),
        A::ChooseLiquidLanding { .. } => resolution
            .and_then(|resolution| {
                resolution
                    .falls
                    .iter()
                    .find(|fall| fall.stage == TacticalFallStage::LandingChoice)
            })
            .map(|fall| fall.actor),
        A::CastSpell { choice, .. } => Some(choice.actor),
        A::AbandonReady { actor } => Some(*actor),
        A::Ready { .. }
        | A::UnarmedStrike { .. }
        | A::Shove { .. }
        | A::FirstAid { .. }
        | A::SecondWind
        | A::DonShield { .. }
        | A::DoffShield
        | A::CreatureWeaponAttack { .. }
        | A::CreatureArea { .. }
        | A::CreatureAttack { .. }
        | A::Move { .. }
        | A::MoveSelfOnly { .. }
        | A::Attack { .. }
        | A::EndTurn
        | A::Dash { .. }
        | A::Disengage
        | A::Dodge
        | A::StandProne
        | A::StartAttackAction => active,
    };
    if actor.is_some_and(|actor| {
        rules
            .tactical_creatures
            .as_ref()
            .and_then(|creatures| creatures.runtime(actor))
            .is_some_and(|runtime| matches!(runtime.controller, CreatureController::Player(_)))
    }) {
        return Err(
            "This source creature's player must make the decision or report its public dice."
                .into(),
        );
    }
    Ok(())
}
