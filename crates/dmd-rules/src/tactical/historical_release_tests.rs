//! Original flow-5 release controls over isolated rule fixtures. These retain
//! their original bodies and producer semantics through the existing Historical
//! reducer; they are not genuine captured application journals. Contemporary
//! public integration scenarios separately exercise the current flow version.
use crate as dmd_rules;
use crate::{tactical::*, tactical_effects::*, *};
use dmd_domain::*;
use std::collections::HashMap;

struct Fixture {
    state: CampaignState,
    pack: RulesPack,
    actors: [EntityId; 2],
    players: [PlayerId; 2],
}
impl Fixture {
    fn new() -> Self {
        let pack = RulesPack::from_json(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../content/srd-5.2.1/kernel.json"
        )))
        .unwrap();
        let mut state = CampaignState::empty(
            Campaign {
                id: CampaignId::new(),
                display_name: "Independent source turn fixture".into(),
                status: CampaignStatus::Active,
                world_seed: 1,
                ruleset: VersionedRef {
                    id: pack.id.clone(),
                    version: pack.version.clone(),
                },
                content_packs: vec![],
            },
            WorldClock {
                now: WorldInstant(0),
                calendar_id: "seconds".into(),
            },
        );
        let actors = [EntityId::new(), EntityId::new()];
        let players = [PlayerId::new(), PlayerId::new()];
        let location = LocationId::new();
        let scene = SceneId::new();
        state.locations.insert(
            location,
            Location {
                id: location,
                campaign_id: state.campaign_id(),
                display_name: "Field".into(),
                parent_location_id: None,
            },
        );
        for (actor, player) in actors.into_iter().zip(players) {
            state.players.insert(
                player,
                Player {
                    id: player,
                    campaign_id: state.campaign_id(),
                    display_name: "Controller".into(),
                },
            );
            state.entities.insert(
                actor,
                WorldEntity {
                    id: actor,
                    campaign_id: state.campaign_id(),
                    display_name: "Adventurer".into(),
                    kind: EntityKind::Character,
                    existence: EntityExistence::Present,
                    location_id: Some(location),
                },
            );
            let id = CharacterId::new();
            state.characters.insert(
                id,
                Character {
                    id,
                    campaign_id: state.campaign_id(),
                    entity_id: actor,
                    controlling_player_id: Some(player),
                    display_name: "Adventurer".into(),
                    status: CharacterStatus::Active,
                },
            );
        }
        state.scenes.insert(
            scene,
            Scene {
                id: scene,
                campaign_id: state.campaign_id(),
                location_id: location,
                mode: SceneMode::Combat,
                status: SceneStatus::Active,
                started_at: WorldInstant(0),
                presences: actors
                    .into_iter()
                    .map(|entity_id| ScenePresence {
                        entity_id,
                        role: PresenceRole::Participant,
                    })
                    .collect(),
            },
        );
        state.rules = Some(RulesState {
            tactical_recovery: None,
            tactical_creatures: None,
            tactical_inventory: None,
            tactical_effects: None,
            pack_id: pack.id.clone(),
            pack_version: pack.version.clone(),
            entities: actors
                .into_iter()
                .map(|id| (id, MechanicalEntity::basic(id)))
                .collect::<HashMap<_, _>>(),
            house_rules: HouseRules::default(),
            effects: vec![],
            pending: None,
            rolls: vec![],
            cancelled_roll_ids: vec![],
            rulings: vec![],
            timing: None,
            rests: vec![],
            completed_short_rests: vec![],
            permission: None,
        });
        let mut f = Self {
            state,
            pack,
            actors,
            players,
        };
        let point = |x, y, z| SpatialPoint { x, y, z };
        let encounter = TacticalEncounter {
            id: EncounterId::new(),
            scene_id: scene,
            battlefield: Battlefield {
                bounds: SpatialBox {
                    min: point(0, 0, 0),
                    max: point(100, 100, 100),
                },
                floor_z: 0,
                floor_surface: "stone".into(),
                ambient_light: LightLevel::Bright,
                terrain: vec![],
                obstacles: vec![],
                lights: vec![],
            },
            participants: actors
                .into_iter()
                .enumerate()
                .map(|(i, entity_id)| TacticalParticipant {
                    entity_id,
                    position: point(10 + 30 * i as i32, 10, 0),
                    size: CreatureSize::Medium,
                    public_label: "Adventurer".into(),
                    height: 12,
                    reach: 10,
                    movement: MovementProfile {
                        walk: 60,
                        climb: None,
                        swim: None,
                        fly: Some(120),
                        burrow: None,
                        hover: false,
                    },
                    senses: Senses::default(),
                    allies: vec![],
                    enemies: vec![],
                })
                .collect(),
            knowledge: vec![],
            origin: f.meta(None),
            area_grid_policy: None,
            geometry_ruling: Ruling {
                basis: RulingBasis::GmAdjudication,
                reason: "Explicit flat field".into(),
            },
            flow: None,
        };
        f.run(
            None,
            TacticalAction::Establish {
                encounter: Box::new(encounter),
            },
        );
        f
    }

    fn meta(&self, actor: Option<usize>) -> CommandMeta {
        CommandMeta {
            id: CommandId::new(),
            campaign_id: self.state.campaign_id(),
            session_id: None,
            issuer: actor.map_or(CommandIssuer::Admin, |i| {
                CommandIssuer::Player(self.players[i])
            }),
            actor: actor.map(|i| AgentRef::Entity(self.actors[i])),
            expected_event_sequence: self.state.applied_event_sequence,
        }
    }

    fn rules(&self) -> &RulesState {
        self.state.rules.as_ref().unwrap()
    }

    fn creature(&mut self, index: usize, controlled: bool) {
        use dmd_rules::tactical_creatures::*;
        let actor = self.actors[index];
        self.state.characters.retain(|_, c| c.entity_id != actor);
        self.state.entities.get_mut(&actor).unwrap().kind = EntityKind::Creature;
        self.state.rules.as_mut().unwrap().entities.remove(&actor);
        let built = build_creature(
            &self.state,
            &self.meta(None),
            actor,
            &CreatureBuildChoice {
                definition_id: "adult-red-dragon".into(),
                size: CreatureSize::Huge,
                additional_languages: vec![],
                hit_points: CreatureHitPointChoice::Average,
                controller: if controlled {
                    CreatureController::Player(self.players[index])
                } else {
                    CreatureController::Host
                },
                in_lair: false,
            },
        )
        .unwrap();
        let participant = self
            .state
            .encounter
            .as_mut()
            .unwrap()
            .participants
            .iter_mut()
            .find(|p| p.entity_id == actor)
            .unwrap();
        participant.size = CreatureSize::Huge;
        participant.height = 30;
        participant.movement = built.movement;
        participant.senses = built.senses;
        // Keep two Huge actors disjoint when testing independent after-turn windows.
        participant.position.x = if index == 0 { 20 } else { 70 };
        self.state
            .encounter
            .as_mut()
            .unwrap()
            .battlefield
            .bounds
            .max
            .x = 120;
        let rules = self.state.rules.as_mut().unwrap();
        rules.entities.insert(actor, built.mechanics);
        let current = rules.tactical_creatures.get_or_insert(TacticalCreatures {
            schema_version: 1,
            profiles: vec![],
            runtime: vec![],
        });
        current.profiles.push(built.profile);
        current.runtime.push(built.runtime);
        self.state.applied_event_sequence += 1;
    }

    fn run(&mut self, actor: Option<usize>, action: TacticalAction) -> TacticalEvent {
        let meta = self.meta(actor);
        let before = self.state.clone();
        let transition = super::resolve_with_policy(
            &self.state,
            &meta,
            &action,
            &self.pack,
            super::ExecutionPolicy::Historical,
        )
        .unwrap();
        let event: TacticalEvent =
            serde_json::from_slice(&serde_json::to_vec(&transition.event).unwrap()).unwrap();
        assert_eq!(
            replay_tactical(&before, &event, &self.pack).unwrap(),
            transition
        );
        self.state =
            serde_json::from_slice(&serde_json::to_vec(&transition.next_state).unwrap()).unwrap();
        self.state.applied_event_sequence += 1;
        validate_state(&self.state, &self.pack).unwrap();
        validate_tactical_state(&self.state).unwrap();
        assert!(
            self.state.validate().is_empty(),
            "{:?}",
            self.state.validate()
        );
        event
    }

    fn rejected(&self, actor: Option<usize>, action: TacticalAction) {
        let before = self.state.clone();
        assert!(
            super::resolve_with_policy(
                &self.state,
                &self.meta(actor),
                &action,
                &self.pack,
                super::ExecutionPolicy::Historical
            )
            .is_err()
        );
        assert_eq!(self.state, before);
    }

    fn raw(&self, values: &[u16]) -> RollResult {
        let request = &self.rules().pending.as_ref().unwrap().request;
        RollResult {
            request_id: request.id,
            source: RollSource::Physical,
            dice: values
                .iter()
                .map(|v| DieResult {
                    sides: request.dice[0].sides,
                    value: *v,
                })
                .collect(),
        }
    }

    fn begin(&mut self) {
        self.run(
            None,
            TacticalAction::Begin {
                execution: dmd_domain::TacticalExecutionVersion::EncounterReleaseV1,
                combatants: self
                    .actors
                    .into_iter()
                    .map(|actor| TacticalCombatant {
                        actor,
                        source: self
                            .rules()
                            .tactical_creatures
                            .as_ref()
                            .and_then(|c| c.profile(actor))
                            .map_or(TacticalSource::Character, |p| TacticalSource::Creature {
                                definition_id: p.source.definition_id.clone(),
                            }),
                        surprised: false,
                    })
                    .collect(),
                groups: self
                    .actors
                    .into_iter()
                    .map(|actor| InitiativeGroup {
                        actors: vec![actor],
                        request_id: RollRequestId::new(),
                    })
                    .collect(),
            },
        );
        for (index, value) in [(0, 18), (1, 3)] {
            let values =
                if self.rules().pending.as_ref().unwrap().request.mode == RollMode::Disadvantage {
                    vec![value, value]
                } else {
                    vec![value]
                };
            let result = self.raw(&values);
            let actor = self
                .rules()
                .tactical_creatures
                .as_ref()
                .and_then(|c| c.profile(self.actors[index]))
                .is_none()
                .then_some(index);
            self.run(actor, TacticalAction::SubmitRoll { result });
        }
    }
}

fn conclude(f: &mut Fixture) {
    f.run(
        None,
        TacticalAction::ConcludeHostilities {
            cadence: AftermathCadence::ContinueExistingOrder,
            ruling: "Fighting has stopped; retain all actual consequences.".into(),
        },
    );
}

fn prepared_creature() -> Fixture {
    let mut f = Fixture::new();
    f.creature(1, false);
    let origin = f.meta(None);
    f.state.rules.as_mut().unwrap().tactical_inventory = Some(TacticalInventory {
        schema_version: 1,
        receipts: vec![],
        loadouts: vec![ActorEquipmentLoadout {
            actor: f.actors[1],
            hands: WeaponLoadout::default(),
            worn_armor: None,
            shield: None,
            command: origin,
        }],
    });
    f.begin();
    conclude(&mut f);
    f
}

fn install_defense(f: &mut Fixture, ordinal: u16, expires: TacticalEffectExpiry) {
    let origin = f.meta(None);
    f.state = tactical_effect_adapter::apply_effect_operation(
        &f.state,
        &origin,
        &EffectLifecycleAction {
            step: 0,
            operation: EffectLifecycleOperation::Install {
                effects: vec![TacticalEffect {
                    id: EffectId::new(),
                    source: EffectSource {
                        definition_id: "isolated-defense-fixture".into(),
                        actor: f.actors[1],
                        command: origin.clone(),
                        ordinal,
                    },
                    established_at: None,
                    target: TacticalEffectTarget::Creature(f.actors[1]),
                    concentration_group: None,
                    expires,
                    overlap: Some(EffectOverlap {
                        key: "same-defense".into(),
                        potency: i32::from(ordinal),
                    }),
                    conditions: vec![],
                    defenses: vec![EffectDefense::BaseArmorClass {
                        base: 13,
                        ability: Ability::Dexterity,
                        ends_when_wearing_armor: true,
                    }],
                    triggers: vec![],
                }],
            },
        },
    )
    .unwrap()
    .0;
    f.state.applied_event_sequence += 1;
}

fn isolated_finished(f: &mut Fixture) {
    encounter_release_preflight(&f.state).unwrap();
    let origin = f.meta(None);
    let encounter = f.state.encounter.as_ref().unwrap();
    let flow = encounter.flow.as_ref().unwrap();
    let timing = f.rules().timing.as_ref().unwrap();
    let location = f.state.scenes[&encounter.scene_id].location_id;
    let receipt = TacticalCompletion {
        encounter_id: encounter.id,
        scene_id: encounter.scene_id,
        location_id: location,
        encounter_origin: encounter.origin.clone(),
        initiative_origin: flow.origin.clone(),
        conclusion_origin: flow.aftermath.as_ref().unwrap().origin.clone(),
        released_by: origin,
        predecessor: None,
        released_at: f.state.clock.now,
        final_turn: EffectTurn {
            actor: timing.order[timing.index].actor,
            number: timing.turn_number,
            boundary: TurnBoundary::Start,
        },
        execution: TacticalExecutionVersion::EncounterReleaseV1,
    };
    let space = TacticalSceneSpace {
        encounter_id: encounter.id,
        scene_id: encounter.scene_id,
        location_id: location,
        release: receipt.released_by.id,
        battlefield: encounter.battlefield.clone(),
        participants: encounter
            .participants
            .iter()
            .map(|participant| participant.entity_id)
            .collect(),
        ground_items: flow.ground_items.clone(),
    };
    f.state.scenes.get_mut(&receipt.scene_id).unwrap().status = SceneStatus::Closed;
    let flow = f.state.encounter.as_mut().unwrap().flow.as_mut().unwrap();
    flow.version = 5;
    flow.phase = TacticalPhase::Finished;
    flow.budget = TacticalTurnBudget::default();
    flow.ground_items.clear();
    let rules = f.state.rules.as_mut().unwrap();
    rules.timing = None;
    let effects = rules.tactical_effects.as_mut().unwrap();
    effects.turn = None;
    effects.trigger_uses.clear();
    if let Some(creatures) = &mut rules.tactical_creatures {
        for runtime in &mut creatures.runtime {
            runtime.observed_turn = None;
            runtime.legendary_window_spent = false;
        }
    }
    f.state.encounter_history = Some(Box::new(TacticalEncounterHistory {
        elapsed_intervals: vec![],
        completions: vec![receipt],
        spaces: vec![space],
    }));
    f.state.applied_event_sequence += 1;
}

fn isolated_replacement(state: &mut CampaignState, origin: CommandMeta) {
    let old_scene = state.encounter.as_ref().unwrap().scene_id;
    let mut scene = state.scenes[&old_scene].clone();
    scene.id = SceneId::new();
    scene.status = SceneStatus::Active;
    let encounter = state.encounter.as_mut().unwrap();
    encounter.id = EncounterId::new();
    encounter.scene_id = scene.id;
    encounter.origin = origin;
    encounter.flow = None;
    state.scenes.insert(scene.id, scene);
    state.applied_event_sequence += 1;
}

#[test]
fn accepted_release_preserves_source_resources_and_replacement_advances_global_turn() {
    let mut f = prepared_creature();
    f.rejected(Some(0), TacticalAction::FinishEncounter);
    let before = f.state.clone();
    let event = f.run(None, TacticalAction::FinishEncounter);
    let history = f.state.encounter_history.as_ref().unwrap();
    assert_eq!(history.completions.len(), 1);
    let receipt = history.last().unwrap().clone();
    assert_eq!(receipt.released_by, event.meta);
    assert_eq!(
        receipt.final_turn.number,
        before
            .rules
            .as_ref()
            .unwrap()
            .timing
            .as_ref()
            .unwrap()
            .turn_number
    );
    assert_eq!(f.state.clock, before.clock);
    assert_eq!(f.state.items, before.items);
    assert_eq!(
        f.state.rules.as_ref().unwrap().entities,
        before.rules.as_ref().unwrap().entities
    );
    assert_eq!(
        f.state.rules.as_ref().unwrap().tactical_inventory,
        before.rules.as_ref().unwrap().tactical_inventory
    );
    let mut source = f.rules().tactical_creatures.clone().unwrap();
    let previous = before
        .rules
        .as_ref()
        .unwrap()
        .tactical_creatures
        .as_ref()
        .unwrap();
    for (runtime, old) in source.runtime.iter_mut().zip(&previous.runtime) {
        assert!(runtime.observed_turn.is_none());
        assert!(!runtime.legendary_window_spent);
        assert_eq!(runtime.last_operation, event.meta);
        runtime.observed_turn = old.observed_turn;
        runtime.legendary_window_spent = old.legendary_window_spent;
        runtime.last_operation = old.last_operation.clone();
    }
    assert_eq!(
        &source, previous,
        "only retired cursors and their provenance change"
    );
    f.rejected(None, TacticalAction::FinishEncounter);
    let old_encounter = f.state.encounter.as_ref().unwrap().clone();
    let mut scene = f.state.scenes[&old_encounter.scene_id].clone();
    scene.id = SceneId::new();
    let mut location = f.state.locations[&scene.location_id].clone();
    location.id = LocationId::new();
    scene.location_id = location.id;
    f.state.locations.insert(location.id, location);
    // Application setup stages a fresh Closed scene in the same atomic command.
    let scene_id = scene.id;
    f.state.scenes.insert(scene_id, scene);
    let mut authored = old_encounter.clone();
    authored.id = EncounterId::new();
    authored.scene_id = scene_id;
    authored.flow = None;
    f.run(
        None,
        TacticalAction::Establish {
            encounter: Box::new(authored),
        },
    );
    assert_eq!(
        f.state.scenes[&old_encounter.scene_id].status,
        SceneStatus::Closed
    );
    assert_eq!(f.state.scenes[&scene_id].status, SceneStatus::Active);
    assert!(
        f.actors
            .iter()
            .all(|actor| f.state.entities[actor].location_id
                == Some(f.state.scenes[&scene_id].location_id))
    );
    assert_eq!(
        f.state.encounter_history.as_ref().unwrap().last(),
        Some(&receipt)
    );
    f.begin();
    assert_eq!(
        f.rules().timing.as_ref().unwrap().turn_number,
        receipt.final_turn.number + 1
    );
    assert_eq!(f.rules().timing.as_ref().unwrap().round, 1);
    f.run(Some(0), TacticalAction::StartAttackAction);
    f.rejected(None, TacticalAction::FinishEncounter);
}

#[test]
fn raw_recovery_die_and_due_wake_are_checked_across_the_campaign() {
    let mut f = prepared_creature();
    let origin = f.meta(None);
    let actor = f.actors[1];
    let stable = StableRecovery {
        origin: VitalityOrigin {
            command: origin,
            occurrence: 0,
        },
        stabilized_at: f.state.clock.now,
        delay_roll: None,
    };
    f.state
        .rules
        .as_mut()
        .unwrap()
        .tactical_recovery
        .get_or_insert_default()
        .insert(
            actor,
            TacticalRecovery {
                stable: Some(stable),
                ..TacticalRecovery::default()
            },
        );
    assert!(encounter_release_preflight(&f.state).is_err());
    let raw = RollResult {
        request_id: RollRequestId::new(),
        source: RollSource::Physical,
        dice: vec![DieResult { sides: 4, value: 1 }],
    };
    f.state
        .rules
        .as_mut()
        .unwrap()
        .tactical_recovery
        .as_mut()
        .unwrap()
        .get_mut(&actor)
        .unwrap()
        .stable
        .as_mut()
        .unwrap()
        .delay_roll = Some(raw.clone());
    let before = f.state.clone();
    assert_eq!(
        retained_encounter_dependencies(&f.state).unwrap(),
        vec![actor]
    );
    assert_eq!(f.state, before);
    f.state.clock.now = WorldInstant(3600);
    assert!(encounter_release_preflight(&f.state).is_err());
    assert_eq!(
        f.rules().tactical_recovery.as_ref().unwrap()[&actor]
            .stable
            .as_ref()
            .unwrap()
            .delay_roll,
        Some(raw)
    );
}

#[test]
fn dependency_scan_uses_actual_collective_setup_capacity_and_scene_authority() {
    let mut f = prepared_creature();
    install_defense(
        &mut f,
        0,
        TacticalEffectExpiry::AtTime(WorldInstant(28_800)),
    );
    let mut other_scene = f.state.scenes.values().next().unwrap().clone();
    other_scene.id = SceneId::new();
    other_scene
        .presences
        .retain(|presence| presence.entity_id == f.actors[1]);
    f.state.scenes.insert(other_scene.id, other_scene);
    assert!(
        encounter_release_preflight(&f.state)
            .unwrap_err()
            .to_string()
            .contains("another active scene")
    );
    let mut many = Fixture::new();
    many.begin();
    conclude(&mut many);
    for _ in 0..=MAX_RELEASE_DEPENDENCIES {
        let actor = EntityId::new();
        many.state
            .rules
            .as_mut()
            .unwrap()
            .effects
            .push(ActiveEffect {
                id: EffectId::new(),
                source: actor,
                target: actor,
                condition: None,
                label: "Unit fixture absolute dependency".into(),
                expires: Expiry::AtTime(WorldInstant(28_800)),
                concentration_owner: None,
            });
    }
    assert!(
        retained_encounter_dependencies(&many.state)
            .unwrap_err()
            .to_string()
            .contains("exceed")
    );
    many.state.rules.as_mut().unwrap().effects.pop();
    assert!(
        retained_encounter_dependencies(&many.state)
            .unwrap_err()
            .to_string()
            .contains("required character or owned-source seat")
    );
    assert_eq!(
        MAX_TACTICAL_PARTICIPANTS, 128,
        "historical domain capacity is unchanged"
    );
}

#[test]
fn replacement_setup_and_pending_initiative_cannot_omit_retained_dependencies() {
    let mut f = prepared_creature();
    install_defense(
        &mut f,
        0,
        TacticalEffectExpiry::AtTime(WorldInstant(28_800)),
    );
    let effects_before = f.rules().tactical_effects.as_ref().unwrap().effects.clone();
    isolated_finished(&mut f);
    let origin = f.meta(None);
    isolated_replacement(&mut f.state, origin);
    validate_tactical_state(&f.state).unwrap();
    let mut omitted = f.state.clone();
    omitted
        .encounter
        .as_mut()
        .unwrap()
        .participants
        .retain(|participant| participant.entity_id != f.actors[1]);
    assert!(
        validate_tactical_state(&omitted)
            .unwrap_err()
            .to_string()
            .contains("retained timing dependency")
    );
    let action = TacticalAction::Begin {
        combatants: f
            .actors
            .into_iter()
            .map(|actor| TacticalCombatant {
                actor,
                source: f
                    .rules()
                    .tactical_creatures
                    .as_ref()
                    .and_then(|creatures| creatures.profile(actor))
                    .map_or(TacticalSource::Character, |profile| {
                        TacticalSource::Creature {
                            definition_id: profile.source.definition_id.clone(),
                        }
                    }),
                surprised: false,
            })
            .collect(),
        groups: f
            .actors
            .into_iter()
            .map(|actor| InitiativeGroup {
                actors: vec![actor],
                request_id: RollRequestId::new(),
            })
            .collect(),
        execution: TacticalExecutionVersion::EncounterReleaseV1,
    };
    f.run(None, action);
    validate_state(&f.state, &f.pack).unwrap();
    assert!(f.rules().pending.is_some());
    assert_eq!(
        f.rules().tactical_effects.as_ref().unwrap().effects,
        effects_before
    );
    let mut omitted = f.state.clone();
    omitted
        .encounter
        .as_mut()
        .unwrap()
        .participants
        .retain(|participant| participant.entity_id != f.actors[1]);
    assert!(
        validate_tactical_state(&omitted)
            .unwrap_err()
            .to_string()
            .contains("retained timing dependency")
    );
}
