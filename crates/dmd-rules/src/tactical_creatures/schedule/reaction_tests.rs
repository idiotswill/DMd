//! Source hook tests: the live reaction-window adapter remains the trigger authority.
use super::*;

fn selection(spell: &str) -> CreatureFeatureSelection {
    CreatureFeatureSelection {
        feature_id: "protective-magic".into(),
        spell_id: Some(spell.into()),
        simple_action: None,
    }
}

struct Fixture {
    state: CampaignState,
    creatures: TacticalCreatures,
    actor: EntityId,
    other: EntityId,
}

#[test]
fn declared_begin_cannot_claim_a_reaction_even_for_host_or_its_controller() {
    let mut f = Fixture::new("mage");
    let player = PlayerId::new();
    f.state.players.insert(
        player,
        Player {
            id: player,
            campaign_id: f.state.campaign_id(),
            display_name: "Controller".into(),
        },
    );
    f.creatures.runtime[0].controller = CreatureController::Player(player);
    for issuer in [
        CommandIssuer::Admin,
        CommandIssuer::System,
        CommandIssuer::Player(player),
    ] {
        let mut meta = f.meta();
        meta.issuer = issuer;
        meta.actor = Some(AgentRef::Entity(f.actor));
        let before = (f.state.clone(), f.creatures.clone());
        let result = apply_creature_schedule(
            &f.state,
            &f.creatures,
            &meta,
            &CreatureScheduleOperation::BeginFeature {
                actor: f.actor,
                selection: selection("shield"),
                steps: vec![],
            },
        );
        assert_eq!(result.unwrap_err(), CreatureError::Unauthorized);
        assert_eq!((f.state.clone(), f.creatures.clone()), before);
    }
}

#[test]
fn genuine_reaction_hook_preserves_issuer_origin_and_pays_one_shared_daily_pool() {
    let mut f = Fixture::new("mage");
    let player = PlayerId::new();
    f.state.players.insert(
        player,
        Player {
            id: player,
            campaign_id: f.state.campaign_id(),
            display_name: "Controller".into(),
        },
    );
    f.creatures.runtime[0].controller = CreatureController::Player(player);
    // Reacting does not require waiting for one's first observed Start hook.
    let timing = f.state.rules.as_mut().unwrap().timing.as_mut().unwrap();
    timing.index = 1;
    timing.turn_number = 2;
    timing.action_spent = true;
    timing.bonus_action_spent = true;
    assert!(f.creatures.runtime[0].observed_turn.is_none());
    for (index, spell) in ["shield", "counterspell", "shield"].into_iter().enumerate() {
        let mut meta = f.meta();
        meta.issuer = CommandIssuer::Player(player);
        meta.actor = Some(AgentRef::Entity(f.actor));
        let before_timing = f.state.rules.as_ref().unwrap().timing.clone();
        let transition = begin_creature_reaction_feature(
            &f.state,
            &f.creatures,
            &meta,
            f.actor,
            selection(spell),
        )
        .unwrap();
        assert_eq!(transition.cost, CreatureActionCost::Reaction);
        assert!(transition.requests.is_empty());
        let plan = transition.feature.unwrap();
        assert_eq!(plan.invocation, meta);
        assert_eq!(plan.enclosing_activation.origin, meta);
        assert_eq!(
            plan.enclosing_activation.activation,
            FeatureActivation::Reaction
        );
        assert!(!plan.enclosing_activation.attack_action);
        assert_eq!(plan.source, f.creatures.profiles[0].source);
        assert_eq!(plan.selection, selection(spell));
        assert_eq!(transition.next.runtime[0].limited_uses.len(), 1);
        assert_eq!(
            transition.next.runtime[0].limited_uses[0].spent,
            (index + 1) as u8
        );
        assert_eq!(
            f.state.rules.as_ref().unwrap().timing,
            before_timing,
            "caller owns the atomic central Reaction payment"
        );
        f.creatures =
            serde_json::from_slice(&serde_json::to_vec(&transition.next).unwrap()).unwrap();
        validate_tactical_creatures(&f.state, &f.creatures).unwrap();
        f.state.applied_event_sequence += 1;
        // Model the caller's actual budget commit, then a later refreshed turn.
        f.state
            .rules
            .as_mut()
            .unwrap()
            .timing
            .as_mut()
            .unwrap()
            .reactions_spent
            .push(f.actor);
        assert!(
            begin_creature_reaction_feature(
                &f.state,
                &f.creatures,
                &f.meta(),
                f.actor,
                selection("shield")
            )
            .is_err()
        );
        let timing = f.state.rules.as_mut().unwrap().timing.as_mut().unwrap();
        timing.reactions_spent.clear();
        timing.turn_number += 2;
        timing.round += 1;
    }
    let before = f.creatures.clone();
    assert!(
        begin_creature_reaction_feature(
            &f.state,
            &f.creatures,
            &f.meta(),
            f.actor,
            selection("counterspell")
        )
        .is_err()
    );
    assert_eq!(f.creatures, before);
    // Source per-day means Long Rest; a Short Rest cannot refresh the shared pool.
    f.state.rules.as_mut().unwrap().timing = None;
    for (kind, time, spent) in [(RestKind::Short, 3600, 3), (RestKind::Long, 32400, 0)] {
        f.state.clock.now = WorldInstant(time);
        let meta = f.meta();
        f.creatures = apply_creature_schedule(
            &f.state,
            &f.creatures,
            &meta,
            &CreatureScheduleOperation::FinishRest {
                actor: f.actor,
                receipt: CreatureRestReceipt {
                    origin: meta.clone(),
                    kind,
                    finished_at: f.state.clock.now,
                },
            },
        )
        .unwrap()
        .next;
        assert_eq!(f.creatures.runtime[0].limited_uses[0].spent, spent);
    }
}

#[test]
fn source_reaction_hook_rejects_spent_incapacitated_foreign_or_wrong_capability_unchanged() {
    for case in 0..9 {
        let mut f = Fixture::new("mage");
        let mut meta = f.meta();
        let mut chosen = selection("shield");
        match case {
            0 => f
                .state
                .rules
                .as_mut()
                .unwrap()
                .timing
                .as_mut()
                .unwrap()
                .reactions_spent
                .push(f.actor),
            1 => {
                f.state
                    .rules
                    .as_mut()
                    .unwrap()
                    .entities
                    .get_mut(&f.actor)
                    .unwrap()
                    .hp = 0
            }
            2 => meta.expected_event_sequence -= 1,
            3 => meta.actor = Some(AgentRef::Entity(f.other)),
            4 => {
                chosen = CreatureFeatureSelection {
                    feature_id: "spellcasting".into(),
                    spell_id: Some("mage-armor".into()),
                    simple_action: None,
                }
            }
            5 => chosen.spell_id = Some("fireball".into()),
            6 => f
                .state
                .encounter
                .as_mut()
                .unwrap()
                .participants
                .retain(|p| p.entity_id != f.actor),
            7 => f.state.rules.as_mut().unwrap().effects.push(ActiveEffect {
                id: EffectId::new(),
                source: f.other,
                target: f.actor,
                condition: Some(Condition::Stunned),
                label: "Stunned".into(),
                expires: Expiry::Never,
                concentration_owner: None,
            }),
            _ => {
                let player = PlayerId::new();
                f.state.players.insert(
                    player,
                    Player {
                        id: player,
                        campaign_id: f.state.campaign_id(),
                        display_name: "Other player".into(),
                    },
                );
                meta.issuer = CommandIssuer::Player(player);
                meta.actor = Some(AgentRef::Entity(f.actor));
            }
        }
        let before = (f.state.clone(), f.creatures.clone());
        assert!(
            begin_creature_reaction_feature(&f.state, &f.creatures, &meta, f.actor, chosen)
                .is_err(),
            "case {case}"
        );
        assert_eq!((f.state, f.creatures), before);
    }
}
impl Fixture {
    fn new(id: &str) -> Self {
        let actor = EntityId::new();
        let other = EntityId::new();
        let mut state = CampaignState::empty(
            Campaign {
                id: CampaignId::new(),
                display_name: "Source scheduling".into(),
                status: CampaignStatus::Active,
                world_seed: 1,
                ruleset: VersionedRef {
                    id: "srd-5.2".into(),
                    version: "5.2.1".into(),
                },
                content_packs: vec![],
            },
            WorldClock {
                now: WorldInstant(0),
                calendar_id: "seconds".into(),
            },
        );
        state.applied_event_sequence = 10;
        for entity in [actor, other] {
            state.entities.insert(
                entity,
                WorldEntity {
                    id: entity,
                    campaign_id: state.campaign_id(),
                    display_name: "Creature".into(),
                    kind: EntityKind::Creature,
                    existence: EntityExistence::Present,
                    location_id: None,
                },
            );
        }
        let meta = CommandMeta {
            id: CommandId::new(),
            campaign_id: state.campaign_id(),
            session_id: None,
            issuer: CommandIssuer::Admin,
            actor: None,
            expected_event_sequence: 10,
        };
        let definition = creature_definition(id).unwrap();
        let built = build_creature(
            &state,
            &meta,
            actor,
            &CreatureBuildChoice {
                definition_id: id.into(),
                size: source_size(definition.statistics.size),
                additional_languages: (0..definition.statistics.additional_languages)
                    .map(|i| format!("Chosen language {i}"))
                    .collect(),
                hit_points: CreatureHitPointChoice::Average,
                controller: CreatureController::Autonomous,
                in_lair: false,
            },
        )
        .unwrap();
        state.rules = Some(RulesState {
            pack_id: "srd-5.2".into(),
            pack_version: "5.2.1".into(),
            entities: [
                (actor, built.mechanics),
                (other, MechanicalEntity::basic(other)),
            ]
            .into_iter()
            .collect(),
            house_rules: HouseRules::default(),
            effects: vec![],
            tactical_effects: None,
            tactical_inventory: None,
            tactical_recovery: None,
            tactical_creatures: None,
            pending: None,
            rolls: vec![],
            cancelled_roll_ids: vec![],
            rulings: vec![],
            timing: Some(CombatTiming {
                order: vec![
                    InitiativeEntry {
                        actor,
                        total: 10,
                        tie_break: 0,
                    },
                    InitiativeEntry {
                        actor: other,
                        total: 1,
                        tie_break: 1,
                    },
                ],
                index: 0,
                round: 1,
                turn_number: 1,
                action_spent: false,
                bonus_action_spent: false,
                slot_spent_this_turn: false,
                reactions_spent: vec![],
            }),
            rests: vec![],
            completed_short_rests: vec![],
            permission: None,
        });
        let participant = |entity_id| TacticalParticipant {
            entity_id,
            position: SpatialPoint { x: 0, y: 0, z: 0 },
            size: CreatureSize::Medium,
            public_label: "Creature".into(),
            height: 10,
            reach: 10,
            movement: MovementProfile {
                walk: 60,
                climb: None,
                swim: None,
                fly: None,
                burrow: None,
                hover: false,
            },
            senses: Senses::default(),
            allies: vec![],
            enemies: vec![],
        };
        state.encounter = Some(TacticalEncounter {
            id: EncounterId::new(),
            scene_id: SceneId::new(),
            battlefield: Battlefield {
                bounds: SpatialBox {
                    min: SpatialPoint { x: 0, y: 0, z: 0 },
                    max: SpatialPoint {
                        x: 100,
                        y: 100,
                        z: 100,
                    },
                },
                floor_z: 0,
                floor_surface: "stone".into(),
                ambient_light: LightLevel::Bright,
                terrain: vec![],
                obstacles: vec![],
                lights: vec![],
            },
            participants: vec![participant(actor), participant(other)],
            knowledge: vec![],
            origin: meta,
            geometry_ruling: Ruling {
                basis: RulingBasis::GmAdjudication,
                reason: "fixture".into(),
            },
            flow: None,
        });
        Self {
            state,
            creatures: TacticalCreatures {
                schema_version: 1,
                profiles: vec![built.profile],
                runtime: vec![built.runtime],
            },
            actor,
            other,
        }
    }
    fn meta(&self) -> CommandMeta {
        CommandMeta {
            id: CommandId::new(),
            campaign_id: self.state.campaign_id(),
            session_id: None,
            issuer: CommandIssuer::Admin,
            actor: None,
            expected_event_sequence: self.state.applied_event_sequence,
        }
    }
}
