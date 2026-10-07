//! Current owned options are proposals; the shared accepted producer rechecks each one.
use super::*;

pub(crate) fn action(choice: &TableGrappleChoice) -> TacticalAction {
    match choice {
        TableGrappleChoice::Attempt {
            target,
            hand,
            before_change,
        } => TacticalAction::Grapple {
            target: *target,
            hand: *hand,
            before_change: *before_change,
        },
        TableGrappleChoice::Save { grip, ability } => TacticalAction::ChooseGrappleSave {
            grip: *grip,
            ability: *ability,
        },
        TableGrappleChoice::AfterEquipment {
            grip,
            work,
            operation: Some(operation),
        } => TacticalAction::ApplyGrappleAfterEquipment {
            grip: *grip,
            work: *work,
            operation: *operation,
        },
        TableGrappleChoice::AfterEquipment {
            grip,
            work,
            operation: None,
        } => TacticalAction::DeclineGrappleAfterEquipment {
            grip: *grip,
            work: *work,
        },
        TableGrappleChoice::Withdraw { grip } => TacticalAction::WithdrawGrapple { grip: *grip },
        TableGrappleChoice::Release { grip } => TacticalAction::ReleaseGrapple { grip: *grip },
        TableGrappleChoice::Escape { grip, choice } => TacticalAction::EscapeGrapple {
            grip: *grip,
            choice: *choice,
        },
    }
}

fn equipment_candidates(
    state: &CampaignState,
    actor: EntityId,
) -> Vec<Option<AttackEquipmentOperation>> {
    let mut items = state
        .items
        .values()
        .filter(|item| {
            item.custody == Custody::Entity(actor)
                && item.state == ItemState::Intact
                && item.quantity > 0
        })
        .map(|item| item.id)
        .collect::<Vec<_>>();
    items.sort_by_key(|item| item.0);
    let mut result = vec![None];
    for item in items {
        result.push(Some(AttackEquipmentOperation::Unequip { item }));
        for hand in [Hand::Left, Hand::Right] {
            result.push(Some(AttackEquipmentOperation::Equip { item, hand }));
        }
    }
    result
}

pub(crate) fn choices(
    read: &execution::ReadContext<'_>,
    issuer: CommandIssuer,
    pack: &RulesPack,
) -> Result<Vec<TableGrappleOffer>, RulesError> {
    let state = read.state();
    if !crate::table::grapple_enabled(state) {
        return Ok(vec![]);
    }
    read.require_guarded("Grapple choices require original table history")?;
    let Some(session) = state
        .table
        .as_ref()
        .and_then(|table| table.active_session.as_ref())
    else {
        return Ok(vec![]);
    };
    if let CommandIssuer::Player(player) = issuer
        && !session.participants.iter().any(|participant| {
            participant.player_id == player && participant.attendance == AttendanceStatus::Present
        })
    {
        return Ok(vec![]);
    }
    let Some(current_flow) = state
        .encounter
        .as_ref()
        .and_then(|encounter| encounter.flow.as_ref())
    else {
        return Ok(vec![]);
    };
    if current_flow.version != TacticalExecutionVersion::EncounterReleaseV1.flow_version()
        || current_flow.phase != TacticalPhase::Active
    {
        return Ok(vec![]);
    }
    let rules = state.rules.as_ref().ok_or(RulesError::Uninitialized)?;
    let mut actors = encounter(state)?
        .participants
        .iter()
        .map(|p| p.entity_id)
        .collect::<Vec<_>>();
    actors.sort_by_key(|actor| actor.0);
    let mut result = vec![];
    for actor in actors.iter().copied() {
        let meta = CommandMeta {
            // Pure preview identity; no accepted declaration or roll ever uses it.
            id: CommandId(uuid::Uuid::new_v5(
                &state.campaign_id().0,
                actor.0.as_bytes(),
            )),
            campaign_id: state.campaign_id(),
            session_id: Some(session.session_id),
            issuer,
            actor: matches!(issuer, CommandIssuer::Player(_)).then_some(AgentRef::Entity(actor)),
            expected_event_sequence: state.applied_event_sequence,
        };
        if super::super::shove::authorize_owner(state, &meta, actor).is_err() {
            continue;
        }
        if let CommandIssuer::Player(player) = issuer
            && rules
                .tactical_creatures
                .as_ref()
                .and_then(|creatures| creatures.runtime(actor))
                .is_some()
            && !crate::table::source_control::owns_source(state, player, actor)
        {
            continue;
        }
        let mut add = |choice| result.push(TableGrappleOffer { actor, choice });
        if let Some(live) = &rules.tactical_grapples {
            for grip in &live.active {
                if grip.declaration.grappler == actor {
                    add(TableGrappleChoice::Release {
                        grip: grip.declaration.id,
                    });
                }
                if grip.declaration.target == actor
                    && active(state)? == actor
                    && current_flow.resolution.is_none()
                    && rules.pending.is_none()
                    && rules
                        .timing
                        .as_ref()
                        .is_some_and(|timing| !timing.action_spent)
                    && crate::tactical_conditions::can_act(rules, actor)?
                    && super::super::falling::require_settled_before_action(state).is_ok()
                {
                    for choice in [
                        GrappleEscapeChoice::Athletics,
                        GrappleEscapeChoice::Acrobatics,
                    ] {
                        add(TableGrappleChoice::Escape {
                            grip: grip.declaration.id,
                            choice,
                        });
                    }
                }
            }
        }
        if let Ok(attempt) = attempt(state) {
            let grip = attempt.declaration.id;
            if attempt.declaration.grappler == actor
                && attempt.reservation().is_some()
                && (attempt.selected.is_some()
                    || resolution(state)?.pending.is_some()
                    || resolution(state)?.failed_save.is_some())
            {
                add(TableGrappleChoice::Withdraw { grip });
            }
            if attempt.declaration.target == actor
                && attempt.stage == TacticalGrappleAttemptStage::SaveChoice
            {
                for ability in [GrappleSaveAbility::Strength, GrappleSaveAbility::Dexterity] {
                    add(TableGrappleChoice::Save { grip, ability });
                }
            }
            if attempt.declaration.grappler == actor
                && attempt.stage == TacticalGrappleAttemptStage::AfterEquipment
                && attempt.equipment.before_change.is_none()
                && attempt.equipment.after.is_none()
                && let Some(selected) = &attempt.selected
            {
                let work = work_key(state, selected)?;
                let loadout = admission::loadout(state, actor)?;
                let hands = crate::tactical_hands::EffectiveHands::current_with_read(read, actor)?;
                for operation in equipment_candidates(state, actor) {
                    if let Some(operation) = operation {
                        let mut proposed = loadout.hands.clone();
                        if crate::tactical_weapons::apply_attack_equipment_operation(
                            state,
                            actor,
                            attempt.declaration.window,
                            definitions()?,
                            &mut proposed,
                            operation,
                            &hands,
                        )
                        .is_err()
                        {
                            continue;
                        }
                    }
                    add(TableGrappleChoice::AfterEquipment {
                        grip,
                        work,
                        operation,
                    });
                }
            }
        }
        if active(state)? == actor && current_flow.resolution.is_none() && rules.pending.is_none() {
            for target in actors.iter().copied().filter(|target| *target != actor) {
                for hand in [Hand::Left, Hand::Right] {
                    for before_change in equipment_candidates(state, actor) {
                        if admission::plan_attempt_with_read(
                            read,
                            &meta,
                            target,
                            hand,
                            before_change,
                            pack,
                        )
                        .is_ok()
                        {
                            add(TableGrappleChoice::Attempt {
                                target,
                                hand,
                                before_change,
                            });
                        }
                    }
                }
            }
        }
    }
    Ok(result)
}
