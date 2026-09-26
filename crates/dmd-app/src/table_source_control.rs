//! Explicit table access to genuine source owners. Ownership remains in source runtime.
use dmd_domain::*;
use dmd_rules::{RulesPack, tactical_creatures::*};

pub(crate) fn enabled(state: &CampaignState) -> bool {
    state
        .table
        .as_ref()
        .is_some_and(|table| table.source_actor_access.is_some())
}

pub(crate) fn presentation_version(state: &CampaignState) -> u32 {
    if enabled(state) { 2 } else { 1 }
}

/// Reject all unresolved/held work, not merely the selected creature's local routine.
pub(crate) fn settled(state: &CampaignState) -> Result<(), String> {
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

pub(crate) fn adoptions(state: &CampaignState) -> Result<Vec<TableSourceAdoption>, String> {
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

pub(crate) fn activate(
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
    pack: &RulesPack,
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
    dmd_rules::validate_state(&next, pack).map_err(|error| error.to_string())?;
    Ok(next)
}

pub(crate) fn owns_source(state: &CampaignState, player: PlayerId, actor: EntityId) -> bool {
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

pub(crate) fn attending_source(
    state: &CampaignState,
    meta: &CommandMeta,
) -> Result<EntityId, String> {
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

pub(crate) fn has_owned_source(state: &CampaignState, player: PlayerId) -> bool {
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
pub(crate) fn authorize_tactical(
    state: &CampaignState,
    meta: &CommandMeta,
    action: &dmd_rules::tactical::TacticalAction,
) -> Result<(), String> {
    use dmd_rules::tactical::TacticalAction as A;
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
                execution: TacticalExecutionVersion::ShieldHitV1,
                combatants,
                ..
            } => combatants.iter().any(|combatant| owned(combatant.actor)),
            A::UpgradeExecutionTo {
                execution: TacticalExecutionVersion::ShieldHitV1,
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
        // These retain source monster/mixed-tie and explicitly delegated ordering.
        A::Establish { .. }
        | A::Begin { .. }
        | A::ConcludeHostilities { .. }
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
        A::OrderHitResponses { .. } => resolution.and_then(|resolution| {
            (!resolution
                .hit_review
                .as_ref()
                .is_some_and(|hit| hit.delegated_by.is_some()))
            .then_some(resolution.turn_actor)
        }),
        A::ChooseTurnWork { .. } => match resolution {
            Some(resolution)
                if !dmd_rules::tactical::tactical_frame_host_ordering(resolution)
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
        | A::FirstAid { .. }
        | A::SecondWind
        | A::DonShield { .. }
        | A::DoffShield
        | A::CreatureWeaponAttack { .. }
        | A::CreatureArea { .. }
        | A::CreatureAttack { .. }
        | A::Move { .. }
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

pub(crate) fn visible_actors(
    state: &CampaignState,
    viewer: &crate::TableViewer,
) -> Result<Vec<crate::TableControlledSourceActor>, String> {
    let mut result = Vec::new();
    if let Some(rules) = &state.rules
        && let Some(creatures) = &rules.tactical_creatures
    {
        for profile in &creatures.profiles {
            let runtime = creatures
                .runtime(profile.actor)
                .ok_or("Source runtime is absent.")?;
            if !matches!(viewer, crate::TableViewer::Host)
                && !matches!(viewer, crate::TableViewer::Player(player) if owns_source(state, *player, profile.actor))
            {
                continue;
            }
            source_for_profile(profile).map_err(|error| error.to_string())?;
            let world = state
                .entities
                .get(&profile.actor)
                .ok_or("Source entity is absent.")?;
            let entity = rules
                .entities
                .get(&profile.actor)
                .ok_or("Source mechanics are absent.")?;
            result.push(crate::TableControlledSourceActor {
                actor: profile.actor,
                name: world.display_name.clone(),
                definition_id: profile.source.definition_id.clone(),
                controller: runtime.controller,
                hp: entity.hp,
                max_hp: entity.max_hp,
            });
        }
    }
    result.sort_by_key(|actor| actor.actor.0);
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_hit_executor_requires_source_access_without_reinterpreting_old_admission() {
        use dmd_rules::tactical::TacticalAction as A;
        // A qualified typed-state fixture, not a claim of historical table-owned
        // source commands. The original export and its journals remain untouched.
        let export = dmd_persistence::CampaignExport::from_json(include_str!(
            "../tests/fixtures/reactions-v1-upgrade-100c7da.json"
        ))
        .unwrap();
        let mut state = CampaignState::decode_json(&export.current_state.state_json).unwrap();
        let session = state
            .table
            .as_ref()
            .unwrap()
            .active_session
            .as_ref()
            .unwrap();
        let player = session
            .participants
            .iter()
            .find(|participant| participant.attendance == AttendanceStatus::Present)
            .unwrap()
            .player_id;
        let creatures = state
            .rules
            .as_ref()
            .unwrap()
            .tactical_creatures
            .as_ref()
            .unwrap();
        let profile = &creatures.profiles[0];
        let actor = profile.actor;
        let definition_id = profile.source.definition_id.clone();
        let origin = CommandMeta {
            id: CommandId::new(),
            campaign_id: state.campaign_id(),
            session_id: Some(session.session_id),
            issuer: CommandIssuer::Admin,
            actor: None,
            expected_event_sequence: state.applied_event_sequence,
        };
        let transition = apply_creature_schedule(
            &state,
            creatures,
            &origin,
            &CreatureScheduleOperation::SetContext {
                actor,
                controller: CreatureController::Player(player),
                in_lair: false,
            },
        )
        .unwrap();
        state.rules.as_mut().unwrap().tactical_creatures = Some(transition.next);
        state.applied_event_sequence += 1;
        let meta = CommandMeta {
            id: CommandId::new(),
            expected_event_sequence: state.applied_event_sequence,
            ..origin
        };
        let begin = |execution| A::Begin {
            execution,
            combatants: vec![TacticalCombatant {
                actor,
                source: TacticalSource::Creature {
                    definition_id: definition_id.clone(),
                },
                surprised: false,
            }],
            groups: vec![InitiativeGroup {
                actors: vec![actor],
                request_id: RollRequestId::new(),
            }],
        };
        let before = state.clone();
        let upgrade = A::UpgradeExecutionTo {
            execution: TacticalExecutionVersion::ShieldHitV1,
        };
        for action in [
            begin(TacticalExecutionVersion::ShieldHitV1),
            upgrade.clone(),
        ] {
            assert_eq!(
                authorize_tactical(&state, &meta, &action).unwrap_err(),
                "Enable source creature control before starting or upgrading this encounter."
            );
            let player_meta = CommandMeta {
                issuer: CommandIssuer::Player(player),
                ..meta.clone()
            };
            assert!(
                authorize_tactical(&state, &player_meta, &action).is_ok(),
                "the ordinary nonprivileged rules gate must refuse without a source-access oracle"
            );
        }
        for action in [
            begin(TacticalExecutionVersion::Legacy),
            begin(TacticalExecutionVersion::ReactionsV1),
            A::UpgradeExecution,
        ] {
            assert!(authorize_tactical(&state, &meta, &action).is_ok());
        }
        assert_eq!(state, before);
        let activated = activate(&state, &meta, &adoptions(&state).unwrap()).unwrap();
        assert!(authorize_tactical(&activated, &meta, &upgrade).is_ok());
        assert!(
            authorize_tactical(
                &activated,
                &meta,
                &begin(TacticalExecutionVersion::ShieldHitV1)
            )
            .is_ok()
        );
    }

    #[test]
    fn typed_existing_owner_requires_attendance_when_activating_a_running_encounter() {
        // The source-owner changes here are explicitly synthetic typed-state
        // setup, not a claim that an old table binary accepted this ownership.
        let export = dmd_persistence::CampaignExport::from_json(include_str!(
            "../tests/fixtures/reactions-v1-upgrade-100c7da.json"
        ))
        .unwrap();
        let baseline = CampaignState::decode_json(&export.current_state.state_json).unwrap();
        settled(&baseline).unwrap();
        let session = baseline
            .table
            .as_ref()
            .unwrap()
            .active_session
            .as_ref()
            .unwrap();
        let creatures = baseline
            .rules
            .as_ref()
            .unwrap()
            .tactical_creatures
            .as_ref()
            .unwrap();
        let actor = creatures.profiles[0].actor;
        assert!(
            baseline
                .encounter
                .as_ref()
                .unwrap()
                .participant(actor)
                .is_some()
        );
        for attendance in [AttendanceStatus::Absent, AttendanceStatus::Present] {
            let player = session
                .participants
                .iter()
                .find(|p| p.attendance == attendance)
                .unwrap()
                .player_id;
            let origin = CommandMeta {
                id: CommandId::new(),
                campaign_id: baseline.campaign_id(),
                session_id: Some(session.session_id),
                issuer: CommandIssuer::Admin,
                actor: None,
                expected_event_sequence: baseline.applied_event_sequence,
            };
            let transition = apply_creature_schedule(
                &baseline,
                creatures,
                &origin,
                &CreatureScheduleOperation::SetContext {
                    actor,
                    controller: CreatureController::Player(player),
                    in_lair: false,
                },
            )
            .unwrap();
            let mut state = baseline.clone();
            state.rules.as_mut().unwrap().tactical_creatures = Some(transition.next);
            state.applied_event_sequence += 1;
            let activation = CommandMeta {
                id: CommandId::new(),
                expected_event_sequence: state.applied_event_sequence,
                ..origin
            };
            let before = state.clone();
            let result = activate(&state, &activation, &adoptions(&state).unwrap());
            if attendance == AttendanceStatus::Absent {
                assert_eq!(
                    result.unwrap_err(),
                    "Every adopted source controller in this encounter must be present."
                );
            } else {
                assert!(enabled(&result.unwrap()));
            }
            assert_eq!(state, before);
        }
    }

    #[test]
    fn typed_preexisting_source_owner_does_not_silently_enable_v2_projection() {
        // This is a qualified pure typed-state test, not a captured Player-source
        // table corpus or a claim that SetContext had a shipped table endpoint.
        let export = dmd_persistence::CampaignExport::from_json(include_str!(
            "../tests/fixtures/legacy-savage-f960.json"
        ))
        .unwrap();
        let mut state = CampaignState::decode_json(&export.current_state.state_json).unwrap();
        let creatures = state
            .rules
            .as_ref()
            .unwrap()
            .tactical_creatures
            .as_ref()
            .unwrap();
        let actor = creatures.profiles[0].actor;
        let player = *state.players.keys().next().unwrap();
        let meta = CommandMeta {
            id: CommandId::new(),
            campaign_id: state.campaign_id(),
            session_id: state
                .table
                .as_ref()
                .unwrap()
                .active_session
                .as_ref()
                .map(|session| session.session_id),
            issuer: CommandIssuer::Admin,
            actor: None,
            expected_event_sequence: state.applied_event_sequence,
        };
        let transition = apply_creature_schedule(
            &state,
            creatures,
            &meta,
            &CreatureScheduleOperation::SetContext {
                actor,
                controller: CreatureController::Player(player),
                in_lair: false,
            },
        )
        .unwrap();
        state.rules.as_mut().unwrap().tactical_creatures = Some(transition.next);
        state.applied_event_sequence += 1;
        let pack =
            RulesPack::from_json(include_str!("../../../content/srd-5.2.1/kernel.json")).unwrap();
        let v1 = crate::table_projection::project_table_v1(
            &state,
            crate::TableViewer::Player(player),
            &pack,
            &[],
            &[],
            None,
        )
        .unwrap();
        let current = crate::table_projection::project_table(
            &state,
            crate::TableViewer::Player(player),
            &pack,
            &[],
            &[],
            None,
        )
        .unwrap();
        assert_eq!(current, v1);
        assert!(current.source_control.is_none());
        assert!(
            !serde_json::to_string(&current)
                .unwrap()
                .contains("source_control")
        );
        let adoption = adoptions(&state).unwrap();
        assert_eq!(adoption.len(), 1);
        assert_eq!(adoption[0].actor, actor);
        assert_eq!(adoption[0].control_origin, meta);
        assert_eq!(
            adoption[0].source,
            state
                .rules
                .as_ref()
                .unwrap()
                .tactical_creatures
                .as_ref()
                .unwrap()
                .profile(actor)
                .unwrap()
                .source
        );
        let activation = CommandMeta {
            id: CommandId::new(),
            expected_event_sequence: state.applied_event_sequence,
            ..meta
        };
        assert!(
            activate(&state, &activation, &adoption).is_err(),
            "the genuine captured pending attack still blocks activation"
        );
        let proof = TableSourceActorAccess {
            version: TableSourceAccessVersion::SourceActorsV1,
            origin: activation,
            adopted: adoption,
        };
        proof.validate(&state).unwrap();
        let mut wrong = proof.clone();
        wrong.adopted[0].source.definition_id = "mage".into();
        assert!(wrong.validate(&state).is_err());
        let mut wrong = proof.clone();
        wrong.adopted[0].control_origin.campaign_id = CampaignId::new();
        assert!(wrong.validate(&state).is_err());
        let mut wrong = proof.clone();
        wrong.adopted.push(proof.adopted[0].clone());
        assert!(wrong.validate(&state).is_err());
    }
}
