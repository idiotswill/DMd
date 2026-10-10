//! Audience formatting delegates source authorization to the pure table reducer.
use dmd_domain::*;
#[cfg(test)]
use dmd_rules::RulesPack;
pub(crate) use dmd_rules::table::source_control::*;
use dmd_rules::tactical_creatures::*;

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
    fn guarded_grapple_commands_refuse_enabled_privileged_source_authorization() {
        use dmd_rules::tactical::TacticalAction as A;
        // Synthetic compile/guard fixture: reuse captured state and the real
        // activation helper, but these new command/work/item identities are not
        // admitted actions or an accepted Grapple journal. No database is opened.
        let export = dmd_persistence::CampaignExport::from_json(include_str!(
            "../tests/fixtures/reactions-v1-upgrade-100c7da.json"
        ))
        .unwrap();
        let baseline = CampaignState::decode_json(&export.current_state.state_json).unwrap();
        let session = baseline
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
        let holder = baseline
            .rules
            .as_ref()
            .unwrap()
            .tactical_creatures
            .as_ref()
            .unwrap()
            .profiles[0]
            .actor;
        let target = baseline
            .encounter
            .as_ref()
            .unwrap()
            .participants
            .iter()
            .find(|participant| participant.entity_id != holder)
            .unwrap()
            .entity_id;
        let meta = CommandMeta {
            id: CommandId::new(),
            campaign_id: baseline.campaign_id(),
            session_id: Some(session.session_id),
            issuer: CommandIssuer::Admin,
            actor: None,
            expected_event_sequence: baseline.applied_event_sequence,
        };
        assert!(!enabled(&baseline));
        let state = activate(&baseline, &meta, &adoptions(&baseline).unwrap()).unwrap();
        assert!(enabled(&state));
        assert!(state.rules.is_some());
        let before = state.clone();
        let grip = GrappleId::from_declaration(meta.id, holder, target, Hand::Left);
        let work = TacticalWorkKey {
            resolution: CommandId::new(),
            occurrence: 0,
        };
        let actions = [
            A::Grapple {
                target,
                hand: Hand::Left,
                before_change: None,
            },
            A::ChooseGrappleSave {
                grip,
                ability: GrappleSaveAbility::Strength,
            },
            A::ApplyGrappleAfterEquipment {
                grip,
                work,
                operation: AttackEquipmentOperation::Unequip {
                    item: ItemId::new(),
                },
            },
            A::DeclineGrappleAfterEquipment { grip, work },
            A::WithdrawGrapple { grip },
            A::EscapeGrapple {
                grip,
                choice: GrappleEscapeChoice::Athletics,
            },
            A::ReleaseGrapple { grip },
        ];
        for action in &actions {
            for issuer in [CommandIssuer::Admin, CommandIssuer::System] {
                let privileged = CommandMeta {
                    issuer,
                    ..meta.clone()
                };
                assert_eq!(
                    authorize_tactical(&state, &privileged, action).unwrap_err(),
                    "Grapple commands are not enabled."
                );
                assert!(authorize_tactical(&baseline, &privileged, action).is_ok());
            }
            let player_meta = CommandMeta {
                issuer: CommandIssuer::Player(player),
                ..meta.clone()
            };
            assert!(authorize_tactical(&state, &player_meta, action).is_ok());
        }
        // Those legacy/nonprivileged early returns preserve routing only;
        // the unchanged rules gate still refuses every public Grapple action.
        assert_eq!(state, before);
    }

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
            begin(TacticalExecutionVersion::ShieldMissileV1),
            A::UpgradeExecutionTo {
                execution: TacticalExecutionVersion::ShieldMissileV1,
            },
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
                &A::UpgradeExecutionTo {
                    execution: TacticalExecutionVersion::ShieldMissileV1,
                }
            )
            .is_ok()
        );
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
