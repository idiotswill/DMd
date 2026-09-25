use dmd_domain::*;
use dmd_rules::{
    spatial::{ActorTacticalView, ContactStatus, TacticalCellView, TacticalContactView},
    tactical_creatures::*,
};

struct Fixture {
    state: CampaignState,
    creatures: TacticalCreatures,
    actor: EntityId,
    other: EntityId,
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
                additional_languages: vec![],
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
            tactical_recovery: None,
            tactical_effects: None,
            tactical_inventory: None,
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
            flow: None,
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
    fn op(&mut self, op: CreatureScheduleOperation) -> CreatureScheduleTransition {
        let result =
            apply_creature_schedule(&self.state, &self.creatures, &self.meta(), &op).unwrap();
        self.creatures = result.next.clone();
        result
    }
    fn turn(
        &mut self,
        actor: EntityId,
        number: u64,
        boundary: TurnBoundary,
    ) -> CreatureScheduleTransition {
        let timing = self.state.rules.as_mut().unwrap().timing.as_mut().unwrap();
        timing.turn_number = number;
        timing.index = usize::from(actor == self.other);
        if boundary == TurnBoundary::Start {
            timing.action_spent = false;
            timing.bonus_action_spent = false;
        }
        let turn = CreatureTurn {
            encounter_id: self.state.encounter.as_ref().unwrap().id,
            actor,
            number,
            boundary,
        };
        let hooks = creature_turn_hooks(&self.state, &self.creatures, self.actor, turn).unwrap();
        self.op(CreatureScheduleOperation::ObserveTurn {
            actor: self.actor,
            turn,
            recharge_ids: hooks
                .recharge
                .into_iter()
                .map(|feature_id| CreatureRechargeId {
                    feature_id,
                    request_id: RollRequestId::new(),
                })
                .collect(),
        })
    }
    fn begin(
        &mut self,
        id: &str,
        spell: Option<&str>,
        steps: Vec<CreatureRoutineStep>,
    ) -> CreatureScheduleTransition {
        let t = self.op(CreatureScheduleOperation::BeginFeature {
            actor: self.actor,
            selection: selection(id, spell),
            steps,
        });
        match t.cost {
            CreatureActionCost::Action => {
                self.state
                    .rules
                    .as_mut()
                    .unwrap()
                    .timing
                    .as_mut()
                    .unwrap()
                    .action_spent = true
            }
            CreatureActionCost::BonusAction => {
                self.state
                    .rules
                    .as_mut()
                    .unwrap()
                    .timing
                    .as_mut()
                    .unwrap()
                    .bonus_action_spent = true
            }
            _ => (),
        };
        t
    }
    fn reject(&self, op: CreatureScheduleOperation) {
        let before = self.creatures.clone();
        assert!(apply_creature_schedule(&self.state, &self.creatures, &self.meta(), &op).is_err());
        assert_eq!(before, self.creatures);
    }
    fn next_own_start(&mut self) -> CreatureScheduleTransition {
        let n = self
            .state
            .rules
            .as_ref()
            .unwrap()
            .timing
            .as_ref()
            .unwrap()
            .turn_number;
        self.turn(self.actor, n, TurnBoundary::End);
        self.turn(self.other, n + 1, TurnBoundary::Start);
        self.turn(self.other, n + 1, TurnBoundary::End);
        self.turn(self.actor, n + 2, TurnBoundary::Start)
    }
}
fn selection(id: &str, spell: Option<&str>) -> CreatureFeatureSelection {
    CreatureFeatureSelection {
        feature_id: id.into(),
        spell_id: spell.map(str::to_owned),
        simple_action: None,
    }
}
fn step(slot: u8, id: &str, spell: Option<&str>) -> CreatureRoutineStep {
    CreatureRoutineStep {
        slot,
        selection: selection(id, spell),
    }
}

#[test]
fn mixed_multiattack_reserves_action_but_spends_replacement_only_when_step_starts() {
    let mut f = Fixture::new("chimera");
    f.turn(f.actor, 1, TurnBoundary::Start);
    let t = f.begin(
        "multiattack",
        None,
        vec![
            step(1, "bite", None),
            step(0, "ram", None),
            step(2, "fire-breath", None),
        ],
    );
    assert_eq!(t.cost, CreatureActionCost::Action);
    let enclosing = t.activation.unwrap();
    assert!(enclosing.attack_action);
    assert!(t.feature.is_none());
    assert!(f.creatures.runtime[0].recharge[0].available);
    for id in ["bite", "ram"] {
        let t = f.op(CreatureScheduleOperation::TakeStep { actor: f.actor });
        assert_eq!(t.cost, CreatureActionCost::None);
        assert!(t.activation.is_none());
        let plan = t.feature.unwrap();
        assert_eq!(plan.selection.feature_id, id);
        assert_eq!(plan.enclosing_activation, enclosing);
        assert_ne!(plan.invocation.id, enclosing.origin.id);
        assert!(f.creatures.runtime[0].recharge[0].available);
    }
    let restored: TacticalCreatures =
        serde_json::from_slice(&serde_json::to_vec(&f.creatures).unwrap()).unwrap();
    assert_eq!(restored, f.creatures);
    let t = f.op(CreatureScheduleOperation::TakeStep { actor: f.actor });
    assert_eq!(t.feature.unwrap().selection.feature_id, "fire-breath");
    assert!(!f.creatures.runtime[0].recharge[0].available);
    assert!(f.creatures.runtime[0].routine.is_none());
    f.reject(CreatureScheduleOperation::TakeStep { actor: f.actor });
}
#[test]
fn abandoned_or_incapacitated_multiattack_preserves_unused_breath() {
    let mut f = Fixture::new("chimera");
    f.turn(f.actor, 1, TurnBoundary::Start);
    f.begin(
        "multiattack",
        None,
        vec![
            step(0, "ram", None),
            step(1, "bite", None),
            step(2, "fire-breath", None),
        ],
    );
    f.state
        .rules
        .as_mut()
        .unwrap()
        .entities
        .get_mut(&f.actor)
        .unwrap()
        .hp = 0;
    f.reject(CreatureScheduleOperation::TakeStep { actor: f.actor });
    f.op(CreatureScheduleOperation::AbandonRoutine { actor: f.actor });
    assert!(f.creatures.runtime[0].recharge[0].available);
    assert!(
        f.state
            .rules
            .as_ref()
            .unwrap()
            .timing
            .as_ref()
            .unwrap()
            .action_spent
    );
}
#[test]
fn invalid_source_routine_count_slot_and_substitution_are_atomic() {
    let mut f = Fixture::new("chimera");
    f.turn(f.actor, 1, TurnBoundary::Start);
    for steps in [
        vec![step(0, "ram", None)],
        vec![
            step(0, "ram", None),
            step(0, "ram", None),
            step(2, "claw", None),
        ],
        vec![
            step(0, "fire-breath", None),
            step(1, "bite", None),
            step(2, "claw", None),
        ],
    ] {
        f.reject(CreatureScheduleOperation::BeginFeature {
            actor: f.actor,
            selection: selection("multiattack", None),
            steps,
        });
    }
    let mut d = Fixture::new("adult-red-dragon");
    d.turn(d.actor, 1, TurnBoundary::Start);
    d.reject(CreatureScheduleOperation::BeginFeature {
        actor: d.actor,
        selection: selection("multiattack", None),
        steps: vec![
            step(0, "spellcasting", Some("scorching-ray")),
            step(1, "spellcasting", Some("scorching-ray")),
            step(2, "rend", None),
        ],
    });
    d.begin(
        "multiattack",
        None,
        vec![
            step(2, "spellcasting", Some("scorching-ray")),
            step(0, "rend", None),
            step(1, "rend", None),
        ],
    );
    assert_eq!(
        d.op(CreatureScheduleOperation::TakeStep { actor: d.actor })
            .feature
            .unwrap()
            .selection
            .spell_id
            .as_deref(),
        Some("scorching-ray")
    );
}
#[test]
fn recharge_raw_faces_are_secret_persisted_and_only_once_at_own_start() {
    let mut f = Fixture::new("young-red-dragon");
    f.turn(f.actor, 1, TurnBoundary::Start);
    f.begin("fire-breath", None, vec![]);
    let t = f.next_own_start();
    assert_eq!(t.requests.len(), 1);
    let request = t.requests[0].clone();
    assert_eq!(request.visibility, RollVisibility::Secret);
    assert_eq!(request.modifier, 0);
    f.reject(CreatureScheduleOperation::BeginFeature {
        actor: f.actor,
        selection: selection("rend", None),
        steps: vec![],
    });
    let mut result = RollResult {
        request_id: request.id,
        source: RollSource::Physical,
        dice: vec![DieResult { sides: 6, value: 4 }],
    };
    f.op(CreatureScheduleOperation::SubmitRecharge {
        actor: f.actor,
        feature_id: "fire-breath".into(),
        result: result.clone(),
    });
    assert!(!f.creatures.runtime[0].recharge[0].available);
    f.reject(CreatureScheduleOperation::SubmitRecharge {
        actor: f.actor,
        feature_id: "fire-breath".into(),
        result: result.clone(),
    });
    let t = f.next_own_start();
    result.request_id = t.requests[0].id;
    result.dice[0].value = 5;
    f.op(CreatureScheduleOperation::SubmitRecharge {
        actor: f.actor,
        feature_id: "fire-breath".into(),
        result: result.clone(),
    });
    assert!(f.creatures.runtime[0].recharge[0].available);
    assert_eq!(
        f.creatures.runtime[0].recharge[0]
            .last_roll
            .as_ref()
            .unwrap()
            .result,
        result
    );
}
#[test]
fn recharge_rejects_forged_faces_ids_actor_and_restored_request() {
    let mut f = Fixture::new("young-red-dragon");
    f.turn(f.actor, 1, TurnBoundary::Start);
    f.begin("fire-breath", None, vec![]);
    let request = f.next_own_start().requests[0].clone();
    let result = RollResult {
        request_id: request.id,
        source: RollSource::Digital,
        dice: vec![DieResult { sides: 6, value: 6 }],
    };
    for kind in 0..3 {
        let mut corrupt = result.clone();
        match kind {
            0 => corrupt.request_id = RollRequestId::new(),
            1 => corrupt.dice[0].value = 7,
            _ => corrupt.dice.push(DieResult { sides: 6, value: 6 }),
        };
        f.reject(CreatureScheduleOperation::SubmitRecharge {
            actor: f.actor,
            feature_id: "fire-breath".into(),
            result: corrupt,
        });
    }
    let mut meta = f.meta();
    meta.actor = Some(AgentRef::Entity(f.other));
    assert!(
        apply_creature_schedule(
            &f.state,
            &f.creatures,
            &meta,
            &CreatureScheduleOperation::SubmitRecharge {
                actor: f.actor,
                feature_id: "fire-breath".into(),
                result
            }
        )
        .is_err()
    );
    let mut corrupt = f.creatures.clone();
    corrupt.runtime[0].recharge[0]
        .pending
        .as_mut()
        .unwrap()
        .request
        .modifier = 99;
    assert!(validate_tactical_creatures(&f.state, &corrupt).is_err());
}
#[test]
fn legendary_one_per_other_turn_feature_limit_and_own_start_refresh_are_distinct() {
    let mut f = Fixture::new("adult-red-dragon");
    f.turn(f.actor, 1, TurnBoundary::Start);
    f.reject(CreatureScheduleOperation::BeginFeature {
        actor: f.actor,
        selection: selection("fiery-rays", Some("scorching-ray")),
        steps: vec![],
    });
    f.turn(f.actor, 1, TurnBoundary::End);
    f.turn(f.other, 2, TurnBoundary::Start);
    f.turn(f.other, 2, TurnBoundary::End);
    assert_eq!(
        f.begin("fiery-rays", Some("scorching-ray"), vec![]).cost,
        CreatureActionCost::Legendary(1)
    );
    f.reject(CreatureScheduleOperation::BeginFeature {
        actor: f.actor,
        selection: selection("pounce", None),
        steps: vec![],
    });
    // A third participant's next end opens another window, but not another use of Fiery Rays.
    let third = EntityId::new();
    let mut world = f.state.entities[&f.other].clone();
    world.id = third;
    f.state.entities.insert(third, world);
    let mut participant = f.state.encounter.as_ref().unwrap().participants[1].clone();
    participant.entity_id = third;
    f.state
        .encounter
        .as_mut()
        .unwrap()
        .participants
        .push(participant);
    f.state
        .rules
        .as_mut()
        .unwrap()
        .timing
        .as_mut()
        .unwrap()
        .order[1]
        .actor = third;
    f.other = third;
    f.turn(third, 3, TurnBoundary::Start);
    f.turn(third, 3, TurnBoundary::End);
    f.reject(CreatureScheduleOperation::BeginFeature {
        actor: f.actor,
        selection: selection("fiery-rays", Some("scorching-ray")),
        steps: vec![],
    });
    f.begin("commanding-presence", Some("command"), vec![]);
    assert_eq!(f.creatures.runtime[0].legendary_spent, 2);
    f.turn(f.actor, 4, TurnBoundary::Start);
    assert_eq!(f.creatures.runtime[0].legendary_spent, 0);
    assert!(f.creatures.runtime[0].used_this_own_turn.is_empty());
}
#[test]
fn incapacitated_legendary_actions_are_blocked_but_resistance_is_not_an_action() {
    let mut f = Fixture::new("adult-red-dragon");
    f.turn(f.actor, 1, TurnBoundary::Start);
    f.turn(f.actor, 1, TurnBoundary::End);
    f.turn(f.other, 2, TurnBoundary::Start);
    f.turn(f.other, 2, TurnBoundary::End);
    f.state.rules.as_mut().unwrap().effects.push(ActiveEffect {
        id: EffectId::new(),
        source: f.other,
        target: f.actor,
        condition: Some(Condition::Stunned),
        label: "Stunned".into(),
        expires: Expiry::Never,
        concentration_owner: None,
    });
    f.reject(CreatureScheduleOperation::BeginFeature {
        actor: f.actor,
        selection: selection("pounce", None),
        steps: vec![],
    });
    assert!(creature_legendary_resistance_available(&f.state, &f.creatures, f.actor).unwrap());
}
#[test]
fn source_innate_per_rest_limits_and_recharge_persist_across_encounter_restart() {
    let mut f = Fixture::new("cultist-fanatic");
    f.turn(f.actor, 1, TurnBoundary::Start);
    f.begin("spellcasting", Some("hold-person"), vec![]);
    f.next_own_start();
    f.reject(CreatureScheduleOperation::BeginFeature {
        actor: f.actor,
        selection: selection("spellcasting", Some("hold-person")),
        steps: vec![],
    });
    f.state.rules.as_mut().unwrap().timing = None;
    f.op(CreatureScheduleOperation::LeaveCombat { actor: f.actor });
    f.state.clock.now = WorldInstant(3600);
    let meta = f.meta();
    let receipt = CreatureRestReceipt {
        origin: meta.clone(),
        kind: RestKind::Short,
        finished_at: f.state.clock.now,
    };
    let result = apply_creature_schedule(
        &f.state,
        &f.creatures,
        &meta,
        &CreatureScheduleOperation::FinishRest {
            actor: f.actor,
            receipt,
        },
    )
    .unwrap();
    f.creatures = result.next;
    assert_eq!(f.creatures.runtime[0].limited_uses[0].spent, 1);
    f.state.clock.now = WorldInstant(32400);
    let meta = f.meta();
    let receipt = CreatureRestReceipt {
        origin: meta.clone(),
        kind: RestKind::Long,
        finished_at: f.state.clock.now,
    };
    f.creatures = apply_creature_schedule(
        &f.state,
        &f.creatures,
        &meta,
        &CreatureScheduleOperation::FinishRest {
            actor: f.actor,
            receipt,
        },
    )
    .unwrap()
    .next;
    assert_eq!(f.creatures.runtime[0].limited_uses[0].spent, 0);
}
#[test]
fn stale_foreign_and_corrupt_source_authority_fail_closed() {
    let mut f = Fixture::new("chimera");
    f.turn(f.actor, 1, TurnBoundary::Start);
    let operation = CreatureScheduleOperation::BeginFeature {
        actor: f.actor,
        selection: selection("bite", None),
        steps: vec![],
    };
    for case in 0..4 {
        let mut meta = f.meta();
        match case {
            0 => meta.expected_event_sequence -= 1,
            1 => meta.campaign_id = CampaignId::new(),
            2 => meta.issuer = CommandIssuer::Import,
            _ => meta.actor = Some(AgentRef::Entity(f.other)),
        };
        assert!(apply_creature_schedule(&f.state, &f.creatures, &meta, &operation).is_err());
    }
    for case in 0..5 {
        let mut current = f.creatures.clone();
        match case {
            0 => current.runtime[0].recharge.clear(),
            1 => current.runtime[0].legendary_spent = 1,
            2 => current.runtime[0].used_this_own_turn.push("bite".into()),
            3 => current.runtime[0].limited_uses.push(CreatureLimitedUse {
                feature_id: "bite".into(),
                spell_id: None,
                spent: 0,
            }),
            _ => current.profiles[0].source.definition_fingerprint = "0000000000000000".into(),
        };
        assert!(validate_tactical_creatures(&f.state, &current).is_err());
    }
    f.state.entities.get_mut(&f.actor).unwrap().kind = EntityKind::Object;
    assert!(validate_tactical_creatures(&f.state, &f.creatures).is_err());
}
fn view(f: &Fixture, status: ContactStatus) -> ActorTacticalView {
    ActorTacticalView {
        observer: f.actor,
        position: Some(SpatialPoint { x: 0, y: 0, z: 0 }),
        contacts: vec![TacticalContactView {
            entity_id: f.other,
            label: Some("Known threat".into()),
            position: SpatialPoint { x: 10, y: 0, z: 0 },
            status,
            modality: PerceptionModality::Sight,
        }],
        cells: vec![TacticalCellView {
            position: SpatialPoint { x: -10, y: 0, z: 0 },
            difficult: false,
            blocked: false,
            currently_seen: true,
        }],
    }
}
fn morale() -> NpcMorale {
    NpcMorale {
        retreat_at_or_below_percent: 25,
        willing_to_surrender: true,
        willing_to_parley: true,
    }
}
#[test]
fn npc_policy_cannot_target_a_remembered_or_unrepresented_creature() {
    let f = Fixture::new("chimera");
    let caps = npc_capabilities(&f.state, &f.creatures, f.actor).unwrap();
    let relation = vec![KnownRelation {
        entity: f.other,
        disposition: KnownDisposition::Threat,
    }];
    let seen = propose_npc_intents(
        &view(&f, ContactStatus::Seen),
        &caps,
        &relation,
        NpcGoal::DefeatKnownThreats,
        morale(),
    )
    .unwrap();
    assert!(
        matches!(&seen[0].intent,NpcIntent::UseFeature{selection,..} if selection.feature_id=="fire-breath")
    );
    let remembered = propose_npc_intents(
        &view(&f, ContactStatus::Remembered),
        &caps,
        &relation,
        NpcGoal::DefeatKnownThreats,
        morale(),
    )
    .unwrap();
    assert!(
        remembered
            .iter()
            .all(|p| !matches!(p.intent, NpcIntent::UseFeature { .. }))
    );
    assert!(remembered.iter().any(|p| matches!(
        p.intent,
        NpcIntent::Investigate {
            last_known_position: SpatialPoint { x: 10, .. }
        }
    )));
    assert!(
        propose_npc_intents(
            &view(&f, ContactStatus::Seen),
            &caps,
            &[KnownRelation {
                entity: EntityId::new(),
                disposition: KnownDisposition::Threat
            }],
            NpcGoal::DefeatKnownThreats,
            morale()
        )
        .is_err()
    );
}
#[test]
fn npc_morale_uses_own_health_and_known_cells_without_automatic_player_control() {
    let mut f = Fixture::new("chimera");
    f.state
        .rules
        .as_mut()
        .unwrap()
        .entities
        .get_mut(&f.actor)
        .unwrap()
        .hp = 10;
    let caps = npc_capabilities(&f.state, &f.creatures, f.actor).unwrap();
    let knowledge = view(&f, ContactStatus::Seen);
    let relation = vec![KnownRelation {
        entity: f.other,
        disposition: KnownDisposition::Threat,
    }];
    let proposals = propose_npc_intents(
        &knowledge,
        &caps,
        &relation,
        NpcGoal::DefeatKnownThreats,
        morale(),
    )
    .unwrap();
    assert!(matches!(
        proposals[0].intent,
        NpcIntent::Retreat {
            toward_known_cell: SpatialPoint { x: -10, .. }
        }
    ));
    assert!(
        proposals
            .iter()
            .any(|p| p.intent == NpcIntent::OfferSurrender)
    );
    assert!(
        proposals
            .iter()
            .any(|p| p.intent == NpcIntent::OfferParley { spoken: false })
    );
    f.state
        .rules
        .as_mut()
        .unwrap()
        .entities
        .get_mut(&f.other)
        .unwrap()
        .hp = 0;
    assert_eq!(
        proposals,
        propose_npc_intents(
            &knowledge,
            &caps,
            &relation,
            NpcGoal::DefeatKnownThreats,
            morale()
        )
        .unwrap()
    );
    f.creatures.runtime[0].controller = CreatureController::Host;
    assert_eq!(
        npc_capabilities(&f.state, &f.creatures, f.actor).unwrap_err(),
        CreatureError::Unauthorized
    );
}
#[test]
fn npc_approaches_a_far_known_threat_without_targeting_it_beyond_source_reach() {
    let f = Fixture::new("wolf");
    let caps = npc_capabilities(&f.state, &f.creatures, f.actor).unwrap();
    let mut knowledge = view(&f, ContactStatus::Seen);
    knowledge.contacts[0].position.x = 80;
    knowledge.cells[0].position.x = 10;
    let relations = vec![KnownRelation {
        entity: f.other,
        disposition: KnownDisposition::Threat,
    }];
    let proposals = propose_npc_intents(
        &knowledge,
        &caps,
        &relations,
        NpcGoal::DefeatKnownThreats,
        morale(),
    )
    .unwrap();
    assert!(
        proposals
            .iter()
            .all(|p| !matches!(p.intent, NpcIntent::UseFeature { .. }))
    );
    assert!(proposals.iter().any(|p| matches!(
        p.intent,
        NpcIntent::Approach {
            toward_known_cell: SpatialPoint { x: 10, .. }
        }
    )));
    knowledge.cells[0].currently_seen = false;
    assert!(
        propose_npc_intents(
            &knowledge,
            &caps,
            &relations,
            NpcGoal::DefeatKnownThreats,
            morale()
        )
        .unwrap()
        .iter()
        .all(|p| !matches!(p.intent, NpcIntent::Approach { .. }))
    );
}
#[test]
fn a_source_bonus_action_has_a_distinct_central_cost() {
    let mut f = Fixture::new("goblin-warrior");
    f.turn(f.actor, 1, TurnBoundary::Start);
    let selection = CreatureFeatureSelection {
        feature_id: "nimble-escape".into(),
        spell_id: None,
        simple_action: Some(CreatureSimpleAction::Disengage),
    };
    let result = f.op(CreatureScheduleOperation::BeginFeature {
        actor: f.actor,
        selection: selection.clone(),
        steps: vec![],
    });
    assert_eq!(result.cost, CreatureActionCost::BonusAction);
    assert_eq!(
        result.feature.unwrap().enclosing_activation.activation,
        dmd_rules::tactical_definitions::FeatureActivation::BonusAction
    );
    f.state
        .rules
        .as_mut()
        .unwrap()
        .timing
        .as_mut()
        .unwrap()
        .bonus_action_spent = true;
    f.reject(CreatureScheduleOperation::BeginFeature {
        actor: f.actor,
        selection,
        steps: vec![],
    });
    assert_eq!(
        f.begin("scimitar", None, vec![]).cost,
        CreatureActionCost::Action
    );
}
#[test]
fn declined_legendary_window_closes_without_spending_and_next_window_rechecks_effects() {
    let mut f = Fixture::new("adult-red-dragon");
    f.turn(f.actor, 1, TurnBoundary::Start);
    assert!(!creature_legendary_action_available(&f.state, &f.creatures, f.actor).unwrap());
    f.turn(f.actor, 1, TurnBoundary::End);
    f.turn(f.other, 2, TurnBoundary::Start);
    f.turn(f.other, 2, TurnBoundary::End);
    assert!(creature_legendary_action_available(&f.state, &f.creatures, f.actor).unwrap());
    f.op(CreatureScheduleOperation::DeclineLegendaryAction { actor: f.actor });
    assert_eq!(f.creatures.runtime[0].legendary_spent, 0);
    assert!(!creature_legendary_action_available(&f.state, &f.creatures, f.actor).unwrap());
    f.reject(CreatureScheduleOperation::BeginFeature {
        actor: f.actor,
        selection: selection("pounce", None),
        steps: vec![],
    });
    f.turn(f.actor, 3, TurnBoundary::Start);
    f.turn(f.actor, 3, TurnBoundary::End);
    f.turn(f.other, 4, TurnBoundary::Start);
    f.turn(f.other, 4, TurnBoundary::End);
    assert!(creature_legendary_action_available(&f.state, &f.creatures, f.actor).unwrap());
    f.state
        .rules
        .as_mut()
        .unwrap()
        .entities
        .get_mut(&f.actor)
        .unwrap()
        .hp = 0;
    assert!(!creature_legendary_action_available(&f.state, &f.creatures, f.actor).unwrap());
}
