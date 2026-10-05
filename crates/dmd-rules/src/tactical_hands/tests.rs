//! Pure composition controls. Synthetic grip/Attempt records never pass the
//! runnable checkpoint guard and are not accepted histories or gameplay proof.
use super::*;

pub(crate) fn source_state() -> CampaignState {
    let export: serde_json::Value = serde_json::from_str(include_str!(
        "../../../dmd-app/tests/fixtures/reactions-v1-upgrade-100c7da.json"
    ))
    .unwrap();
    serde_json::from_str(export["current_state"]["state_json"].as_str().unwrap()).unwrap()
}

pub(crate) fn pack() -> RulesPack {
    RulesPack::from_json(include_str!("../../../../content/srd-5.2.1/kernel.json")).unwrap()
}

pub(crate) fn human(state: &CampaignState) -> EntityId {
    state
        .table
        .as_ref()
        .unwrap()
        .character_profiles
        .values()
        .map(|profile| profile.entity_id)
        .filter(|actor| {
            state
                .rules
                .as_ref()
                .unwrap()
                .tactical_inventory
                .as_ref()
                .unwrap()
                .loadout(*actor)
                .is_some()
        })
        .min_by_key(|actor| actor.0)
        .unwrap()
}

pub(crate) fn install_attempt(
    state: &mut CampaignState,
    actor: EntityId,
    hand: Hand,
) -> TacticalGrappleDeclaration {
    let target = state
        .encounter
        .as_ref()
        .and_then(|encounter| {
            encounter
                .participants
                .iter()
                .map(|p| p.entity_id)
                .filter(|id| *id != actor && state.entities.contains_key(id))
                .min_by_key(|id| id.0)
        })
        .or_else(|| {
            state
                .entities
                .values()
                .filter(|entity| {
                    entity.id != actor
                        && matches!(
                            entity.kind,
                            EntityKind::Character | EntityKind::Npc | EntityKind::Creature
                        )
                })
                .map(|entity| entity.id)
                .min_by_key(|id| id.0)
        })
        .unwrap();
    let origin = CommandMeta {
        id: CommandId::new(),
        campaign_id: state.campaign_id(),
        session_id: None,
        issuer: CommandIssuer::Admin,
        actor: Some(AgentRef::Entity(actor)),
        expected_event_sequence: state.applied_event_sequence,
    };
    let declaration = TacticalGrappleDeclaration {
        id: GrappleId::from_declaration(origin.id, actor, target, hand),
        window: WeaponActionWindow {
            id: origin.id,
            kind: WeaponActionKind::AttackAction,
        },
        anatomy: ordinary_grapple_anatomy(state, actor, &pack())
            .unwrap()
            .unwrap(),
        origin: origin.clone(),
        grappler: actor,
        target,
        hand,
        target_source: None,
        grappler_from: SpatialPoint { x: 0, y: 0, z: 0 },
        target_from: SpatialPoint { x: 10, y: 0, z: 0 },
        range: 10,
        escape_dc: 13,
    };
    // Only the shape/source hand seam is under test. This synthetic container
    // has no accepted command, geometry, payment, work producer or replay proof.
    if state.encounter.is_none() {
        state.encounter = source_state().encounter;
    }
    let flow = state.encounter.as_mut().unwrap().flow.as_mut().unwrap();
    flow.version = 5;
    flow.resolution = Some(Box::new(TacticalResolution {
        origin,
        turn_actor: actor,
        turn_number: 1,
        boundary: TurnBoundary::Start,
        frames: vec![],
        pending: None,
        failed_save: None,
        legendary_window: None,
        attack: None,
        shove: None,
        hit_review: None,
        movement: None,
        casts: vec![],
        missiles: vec![],
        falls: vec![],
        areas: vec![],
        work_trace: None,
        next_occurrence: 0,
        grapple: Some(Box::new(TacticalGrappleResolution {
            activity: Some(GrappleActivity::Attempt(Box::new(TacticalGrappleAttempt {
                equipment: GrappleEquipmentAdmission {
                    equipment_before: ActorEquipmentLoadout {
                        actor,
                        hands: WeaponLoadout::default(),
                        worn_armor: None,
                        shield: None,
                        command: declaration.origin.clone(),
                    },
                    before_change: None,
                    after: None,
                },
                declaration: declaration.clone(),
                stage: TacticalGrappleAttemptStage::SaveChoice,
                selected: None,
                save: None,
                outcome: None,
            }))),
            proofs: vec![],
            cuts: vec![],
            ends: vec![],
            opportunity_refreshes: vec![],
        })),
    }));
    declaration
}

pub(crate) fn live(declaration: TacticalGrappleDeclaration) -> TacticalGrip {
    let chosen = CommandMeta {
        id: CommandId::new(),
        actor: None,
        expected_event_sequence: declaration.origin.expected_event_sequence + 1,
        ..declaration.origin.clone()
    };
    let key = TacticalRollKey {
        origin: declaration.origin.id,
        role: TacticalRollRole::GrappleSave,
        subject: declaration.target,
        occurrence: 0,
    };
    TacticalGrip {
        declaration,
        established_by: chosen.clone(),
        work: TacticalWorkKey {
            resolution: CommandId::new(),
            occurrence: 0,
        },
        save: TacticalGrappleSave {
            ability: GrappleSaveAbility::Strength,
            chosen_by: chosen.clone(),
            key,
            request: None,
            proof: Some(GrappleSaveProof {
                evidence: GrappleSaveEvidence::Decision(TacticalSaveDecision {
                    key,
                    issued_by: chosen.clone(),
                    resolved_by: chosen.clone(),
                    failure: TacticalSaveFailure::Automatic,
                }),
                legendary: None,
                final_success: false,
                finalized_by: chosen,
            }),
        },
    }
}

#[test]
fn provisional_live_incoming_and_ended_authority_have_distinct_current_occupancy() {
    let mut state = source_state();
    let actor = human(&state);
    let declaration = install_attempt(&mut state, actor, Hand::Right);
    let physical = WeaponLoadout {
        hands: [HandAssignment::Free; 2],
    };
    let hands = EffectiveHands::current(&state, state.rules.as_ref().unwrap(), actor).unwrap();
    assert!(hands.is_free(&physical, Hand::Left));
    assert!(!hands.is_free(&physical, Hand::Right));
    assert!(state.rules.as_ref().unwrap().tactical_grapples.is_none());
    let original = serde_json::to_value(&state).unwrap();
    assert!(
        crate::validate_state(&state, &pack())
            .unwrap_err()
            .to_string()
            .contains("Grapple execution is not enabled")
    );
    assert_eq!(serde_json::to_value(&state).unwrap(), original);

    let grip = live(declaration);
    state
        .encounter
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .resolution = None;
    state.rules.as_mut().unwrap().tactical_grapples = Some(TacticalGrapples {
        schema_version: 1,
        active: vec![grip.clone()],
    });
    assert_eq!(
        EffectiveHands::current(&state, state.rules.as_ref().unwrap(), actor).unwrap(),
        hands
    );
    let incoming = EffectiveHands::current(
        &state,
        state.rules.as_ref().unwrap(),
        grip.declaration.target,
    )
    .unwrap();
    assert!(!incoming.has_reservation());

    // Keep a structurally valid ended proof in a distinct consumer container.
    install_attempt(&mut state, actor, Hand::Left);
    state.rules.as_mut().unwrap().tactical_grapples = None;
    let context = state
        .encounter
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .resolution
        .as_mut()
        .unwrap()
        .grapple
        .as_mut()
        .unwrap();
    context.activity = None;
    context.proofs.push(grip.clone());
    context.ends.push(GrappleEndReceipt {
        grip: grip.declaration.id,
        caused_by: CommandMeta {
            id: CommandId::new(),
            expected_event_sequence: grip.established_by.expected_event_sequence + 1,
            ..grip.established_by.clone()
        },
        cause: GrappleEndCause::Released,
    });
    let ended = EffectiveHands::current(&state, state.rules.as_ref().unwrap(), actor).unwrap();
    assert!(!ended.has_reservation());
    assert!(ended.is_free(&physical, Hand::Right));
}

#[test]
fn source_collision_and_detached_rules_cannot_manufacture_free_hands() {
    let mut state = source_state();
    let actor = human(&state);
    let declaration = install_attempt(&mut state, actor, Hand::Right);
    let hands = EffectiveHands::current(&state, state.rules.as_ref().unwrap(), actor).unwrap();
    let shield = ItemId::new();
    let physical = WeaponLoadout {
        hands: [HandAssignment::Free, HandAssignment::Item(shield)],
    };
    assert!(hands.validate_loadout(&physical).is_err());
    assert!(!hands.can_hold(&physical, Hand::Right, shield));
    // Doff/unequip clearing an Item never clears the derived relation.
    let doffed = WeaponLoadout {
        hands: [HandAssignment::Free; 2],
    };
    assert!(!hands.can_hold(&doffed, Hand::Right, shield));
    assert!(hands.can_hold(&doffed, Hand::Left, shield));

    state.rules.as_mut().unwrap().tactical_grapples = Some(TacticalGrapples {
        schema_version: 1,
        active: vec![live(declaration)],
    });
    assert!(
        EffectiveHands::current(&state, state.rules.as_ref().unwrap(), actor).is_err(),
        "live/provisional collision"
    );
    state.rules.as_mut().unwrap().tactical_grapples = None;
    let detached = state.rules.take().unwrap();
    assert!(EffectiveHands::current(&state, &detached, actor).is_err());
    state.rules = Some(detached);
    let context = state
        .encounter
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .resolution
        .as_mut()
        .unwrap()
        .grapple
        .as_mut()
        .unwrap();
    let Some(GrappleActivity::Attempt(attempt)) = context.activity.as_mut() else {
        unreachable!()
    };
    attempt.declaration.anatomy = GrappleAnatomyProof::HumanCreationV1 {
        character: CharacterId::new(),
    };
    assert!(EffectiveHands::current(&state, state.rules.as_ref().unwrap(), actor).is_err());
}

#[test]
fn old_source_and_absent_fields_do_not_acquire_new_anatomy_requirements() {
    let state = source_state();
    let before = serde_json::to_value(&state).unwrap();
    let rules = state.rules.as_ref().unwrap();
    for actor in rules.entities.keys() {
        assert!(
            !EffectiveHands::current(&state, rules, *actor)
                .unwrap()
                .has_reservation()
        );
    }
    assert_eq!(serde_json::to_value(&state).unwrap(), before);
    // Even a bad new-anatomy profile is not consulted by old hand readers.
    let mut old = state;
    let actor = human(&old);
    old.table
        .as_mut()
        .unwrap()
        .character_profiles
        .values_mut()
        .find(|p| p.entity_id == actor)
        .unwrap()
        .features
        .clear();
    assert!(ordinary_grapple_anatomy(&old, actor, &pack()).is_err());
    assert!(
        !EffectiveHands::current(&old, old.rules.as_ref().unwrap(), actor)
            .unwrap()
            .has_reservation()
    );
}

#[test]
fn unknown_anatomy_cannot_turn_a_default_free_slot_into_a_reserved_source_hand() {
    let mut state = source_state();
    let actor = human(&state);
    install_attempt(&mut state, actor, Hand::Right);
    let profile = state
        .rules
        .as_ref()
        .unwrap()
        .tactical_creatures
        .as_ref()
        .unwrap()
        .profiles
        .iter()
        .find(|p| p.source.definition_id == "goblin-warrior")
        .unwrap()
        .clone();
    assert_eq!(
        ordinary_grapple_anatomy(&state, profile.actor, &pack()).unwrap(),
        None
    );
    let context = state
        .encounter
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .resolution
        .as_mut()
        .unwrap()
        .grapple
        .as_mut()
        .unwrap();
    let Some(GrappleActivity::Attempt(attempt)) = context.activity.as_mut() else {
        unreachable!()
    };
    let declaration = &mut attempt.declaration;
    declaration.grappler = profile.actor;
    declaration.target = actor;
    declaration.origin.actor = Some(AgentRef::Entity(profile.actor));
    declaration.id = GrappleId::from_declaration(
        declaration.origin.id,
        profile.actor,
        actor,
        declaration.hand,
    );
    declaration.anatomy = GrappleAnatomyProof::Creature {
        source: profile.source,
        ordinary_hands: OrdinaryHandAnatomy::TwoHandsV1,
    };
    assert!(EffectiveHands::current(&state, state.rules.as_ref().unwrap(), profile.actor).is_err());
}

#[test]
fn real_goblin_weapon_definitions_share_the_derived_hand_planner() {
    use crate::tactical_creatures::*;
    use crate::tactical_definitions::*;
    use crate::tactical_weapons::*;
    for weapon_id in ["scimitar", "shortbow"] {
        let mut state = source_state();
        let actor = EntityId::new();
        let target = human(&state);
        state.entities.insert(
            actor,
            WorldEntity {
                id: actor,
                campaign_id: state.campaign_id(),
                display_name: "Source planner control".into(),
                kind: EntityKind::Creature,
                existence: EntityExistence::Present,
                location_id: None,
            },
        );
        let source = bundled_goblin_warrior_v2().unwrap();
        let source_pin = creature_source_pin(source).unwrap();
        let origin = CommandMeta {
            id: CommandId::new(),
            campaign_id: state.campaign_id(),
            session_id: None,
            issuer: CommandIssuer::Admin,
            actor: Some(AgentRef::Entity(actor)),
            expected_event_sequence: state.applied_event_sequence,
        };
        let built = build_creature_from_source(
            &state,
            &origin,
            actor,
            &CreatureBuildChoice {
                definition_id: "goblin-warrior".into(),
                size: CreatureSize::Small,
                additional_languages: vec![],
                hit_points: CreatureHitPointChoice::Average,
                controller: CreatureController::Host,
                in_lair: false,
            },
            Some(&source_pin),
        )
        .unwrap();
        let rules = state.rules.as_mut().unwrap();
        rules.entities.insert(actor, built.mechanics);
        let creatures = rules.tactical_creatures.as_mut().unwrap();
        creatures.profiles.push(built.profile);
        creatures.runtime.push(built.runtime);
        let weapon = ItemId::new();
        let arrows = ItemId::new();
        for (id, definition, quantity) in [(weapon, weapon_id, 1), (arrows, "arrows", 20)] {
            state.items.insert(
                id,
                ItemInstance {
                    id,
                    campaign_id: state.campaign_id(),
                    definition_id: definition.into(),
                    display_name: "Physical planner item".into(),
                    quantity,
                    owner: Ownership::Entity(actor),
                    custody: Custody::Entity(actor),
                    state: ItemState::Intact,
                },
            );
        }
        let bow = weapon_id == "shortbow";
        let choice = WeaponUseChoice {
            weapon,
            target,
            delivery: if bow {
                WeaponDelivery::Shot
            } else {
                WeaponDelivery::Melee
            },
            ability: Ability::Dexterity,
            grip: if bow {
                WeaponGrip::TwoHands
            } else {
                WeaponGrip::OneHand(Hand::Left)
            },
            purpose: WeaponAttackPurpose::Normal,
            ammunition: bow.then_some(arrows),
            equipment_change: None,
        };
        let physical = WeaponLoadout {
            hands: [HandAssignment::Item(weapon), HandAssignment::Free],
        };
        let pack = pack();
        let definitions = bundled_tactical_definitions().unwrap();
        let prepare = |state: &CampaignState| {
            prepare_weapon_attack(&WeaponAttackInput {
                state,
                source: WeaponActorSource::CreatureOrdinaryWeapon(source),
                pack: &pack,
                definitions,
                choice: &choice,
                loadout: &physical,
                history: &[],
                context: WeaponAttackContext {
                    origin: &origin,
                    actor,
                    turn_number: 1,
                    on_actor_turn: true,
                    window: WeaponActionWindow {
                        id: origin.id,
                        kind: WeaponActionKind::AttackAction,
                    },
                    distance: 10,
                    base_reach: 10,
                    mounted: false,
                    underwater: false,
                    has_swim_speed: false,
                    target_is_creature: true,
                    target_size: CreatureSize::Medium,
                    distance_from_trigger_target: None,
                },
            })
        };
        let original = prepare(&state).unwrap();
        install_attempt(&mut state, actor, Hand::Right);
        if bow {
            assert!(
                prepare(&state)
                    .unwrap_err()
                    .to_string()
                    .contains("both hands available")
            );
        } else {
            assert_eq!(prepare(&state).unwrap(), original);
        }
    }
}
