use dmd_domain::*;
use dmd_rules::{spatial, tactical::*, tactical_effects::*, tactical_inventory::*, *};
use std::collections::HashMap;

#[path = "tactical_movement/falling.rs"]
mod falling;

struct Fixture {
    state: CampaignState,
    pack: RulesPack,
    actors: [EntityId; 3],
    players: [PlayerId; 3],
}
impl Fixture {
    fn new() -> Self {
        let pack =
            RulesPack::from_json(include_str!("../../../content/srd-5.2.1/kernel.json")).unwrap();
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
        let actors = [EntityId::new(), EntityId::new(), EntityId::new()];
        let players = [PlayerId::new(), PlayerId::new(), PlayerId::new()];
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
    fn entity_mut(&mut self, index: usize) -> &mut MechanicalEntity {
        self.state
            .rules
            .as_mut()
            .unwrap()
            .entities
            .get_mut(&self.actors[index])
            .unwrap()
    }
    fn run(&mut self, actor: Option<usize>, action: TacticalAction) -> TacticalEvent {
        let meta = self.meta(actor);
        self.run_meta(&meta, &action)
    }
    fn run_meta(&mut self, meta: &CommandMeta, action: &TacticalAction) -> TacticalEvent {
        let before = self.state.clone();
        let transition = resolve_tactical(&self.state, meta, action, &self.pack).unwrap();
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
        assert!(resolve_tactical(&self.state, &self.meta(actor), &action, &self.pack).is_err());
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
    fn roll(&mut self, actor: usize, values: &[u16]) {
        let result = self.raw(values);
        self.run(Some(actor), TacticalAction::SubmitRoll { result });
    }
    fn begin(&mut self) {
        self.run(
            None,
            TacticalAction::Begin {
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
        for (index, value) in [(0, 18), (1, 3), (2, 1)] {
            let values = if self.rules().pending.as_ref().unwrap().request.mode != RollMode::Normal
            {
                vec![value, value]
            } else {
                vec![value]
            };
            let result = self.raw(&values);
            self.run(Some(index), TacticalAction::SubmitRoll { result });
        }
    }
    fn arm(&mut self, weapon: &str, mastery: bool, ranged: bool) -> WeaponUseChoice {
        let actor = self.actors[0];
        let mut masteries = vec![];
        if mastery {
            masteries.push(weapon.to_owned());
        }
        for id in ["club", "dagger", "shortbow", "longsword"] {
            if id != weapon && !masteries.iter().any(|x| x == id) && masteries.len() < 3 {
                masteries.push(id.into());
            }
        }
        let input = CharacterCreationInput {
            name: "Adventurer".into(),
            pronouns: "they/them".into(),
            description: "A guard".into(),
            alignment: "Neutral Good".into(),
            backstory: "A traveling guard".into(),
            ability_scores: [15, 14, 13, 8, 10, 12],
            background_boosts: [2, 0, 1, 0, 0, 0],
            fighter_skills: [Skill::Perception, Skill::Survival],
            human_skill: Skill::Insight,
            skilled_skills: [Skill::Acrobatics, Skill::Stealth, Skill::Investigation],
            size: CharacterSize::Medium,
            languages: ["dwarvish".into(), "common-sign-language".into()],
            fighting_style: FightingStyle::Defense,
            gaming_set: GamingSet::Dice,
            purchases: vec![],
            worn_armor: None,
            shield: false,
            masteries: masteries.try_into().unwrap(),
        };
        let built = build_character(&input, actor, &self.pack).unwrap();
        let character = self
            .state
            .characters
            .values()
            .find(|c| c.entity_id == actor)
            .unwrap()
            .id;
        self.state
            .rules
            .as_mut()
            .unwrap()
            .entities
            .insert(actor, built.mechanics);
        self.state.campaign.content_packs = TableContract::default().permitted_content.clone();
        let mut table = TableState::new(TableContract::default());
        table.character_profiles.insert(character, built.profile);
        self.state.table = Some(table);
        let materialized = materialize_starting_equipment(
            &self.state,
            &TacticalInventory::default(),
            &self.meta(Some(0)),
            character,
            &[],
            &self.pack,
        )
        .unwrap();
        self.state = materialized.next_state;
        self.state.rules.as_mut().unwrap().tactical_inventory = Some(materialized.next_inventory);
        self.state.applied_event_sequence += 1;
        // Real borrowed loot follows the same physical identity/custody rules as a
        // materialized starting weapon; its ownership is deliberately somebody else's.
        let weapon_id = self.item(weapon, 1);
        let definitions = dmd_rules::tactical_definitions::TacticalDefinitions::from_json(
            dmd_rules::tactical_definitions::TACTICAL_DEFINITIONS_JSON,
        )
        .unwrap();
        let definition = definitions.weapon(weapon).unwrap();
        let grip = if definition.hands == dmd_rules::tactical_definitions::WeaponHands::One {
            WeaponGrip::OneHand(Hand::Left)
        } else {
            WeaponGrip::TwoHands
        };
        let ammunition = dmd_rules::tactical_weapons::required_ammunition_definition(definition)
            .map(|id| self.item(id, 20));
        let meta = self.meta(Some(0));
        let loadout = &mut self
            .state
            .rules
            .as_mut()
            .unwrap()
            .tactical_inventory
            .as_mut()
            .unwrap()
            .loadouts[0];
        loadout.hands = WeaponLoadout {
            hands: [
                HandAssignment::Item(weapon_id),
                if grip == WeaponGrip::TwoHands {
                    HandAssignment::Item(weapon_id)
                } else {
                    HandAssignment::Free
                },
            ],
        };
        loadout.command = meta;
        self.state.applied_event_sequence += 1;
        self.entity_mut(1).max_hp = 50;
        self.entity_mut(1).hp = 50;
        self.state.encounter.as_mut().unwrap().participants[1]
            .position
            .x = if ranged { 60 } else { 20 };
        self.state.encounter.as_mut().unwrap().participants[0].enemies = vec![self.actors[1]];
        self.state.encounter.as_mut().unwrap().participants[1].enemies = vec![actor];
        WeaponUseChoice {
            weapon: weapon_id,
            target: self.actors[1],
            delivery: if ranged {
                WeaponDelivery::Shot
            } else {
                WeaponDelivery::Melee
            },
            ability: if ranged {
                Ability::Dexterity
            } else {
                Ability::Strength
            },
            grip,
            purpose: WeaponAttackPurpose::Normal,
            ammunition,
            equipment_change: None,
        }
    }
    fn item(&mut self, definition: &str, quantity: u32) -> ItemId {
        let id = ItemId::new();
        self.state.items.insert(
            id,
            ItemInstance {
                id,
                campaign_id: self.state.campaign_id(),
                definition_id: definition.into(),
                display_name: "Borrowed equipment".into(),
                quantity,
                owner: Ownership::Entity(self.actors[1]),
                custody: Custody::Entity(self.actors[0]),
                state: ItemState::Intact,
            },
        );
        id
    }
    fn flow(&self) -> &TacticalFlow {
        self.state
            .encounter
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
    }
    fn request(&self) -> &RollRequest {
        &self.rules().pending.as_ref().unwrap().request
    }
}

fn point(x: i32, y: i32, z: i32) -> SpatialPoint {
    SpatialPoint { x, y, z }
}
fn walk(points: &[SpatialPoint]) -> TacticalAction {
    TacticalAction::Move {
        path: points
            .iter()
            .map(|destination| TacticalMoveStep {
                destination: *destination,
                mode: MovementMode::Walk,
            })
            .collect(),
    }
}
impl Fixture {
    fn position(&self, actor: usize) -> SpatialPoint {
        self.state
            .encounter
            .as_ref()
            .unwrap()
            .participant(self.actors[actor])
            .unwrap()
            .position
    }
    fn movement(&self) -> &TacticalMovement {
        self.flow()
            .resolution
            .as_ref()
            .unwrap()
            .movement
            .as_ref()
            .unwrap()
    }
    fn begin_mover_turn(&mut self) {
        self.begin();
        self.run(Some(0), TacticalAction::EndTurn);
        assert_eq!(
            self.rules().timing.as_ref().unwrap().order[1].actor,
            self.actors[1]
        );
    }
}

#[test]
fn ordinary_movement_commits_real_segments_and_difficult_terrain_without_an_action() {
    let mut f = Fixture::new();
    f.state
        .encounter
        .as_mut()
        .unwrap()
        .battlefield
        .terrain
        .push(TerrainVolume {
            id: "rubble".into(),
            volume: SpatialBox {
                min: point(20, 0, 0),
                max: point(30, 100, 20),
            },
            difficult: true,
            observable: true,
            water: false,
            climbable: false,
            burrowable: false,
            supports_top: false,
            surface: None,
            obscuration: Obscuration::None,
            magical_darkness: false,
        });
    f.begin();
    f.run(Some(0), walk(&[point(20, 10, 0), point(30, 10, 0)]));
    assert_eq!(f.position(0), point(30, 10, 0));
    assert_eq!(f.flow().budget.movement_spent, 30);
    assert!(!f.rules().timing.as_ref().unwrap().action_spent);
    assert!(f.flow().resolution.is_none());
    f.rejected(Some(1), walk(&[point(50, 10, 0)]));
    f.rejected(Some(0), walk(&[point(30, 10, 0)]));
    f.rejected(
        Some(0),
        TacticalAction::Move {
            path: vec![TacticalMoveStep {
                destination: point(60, 10, 0),
                mode: MovementMode::Teleport,
            }],
        },
    );
}

#[test]
fn encountered_obstruction_and_exhaustion_keep_the_accepted_prefix() {
    let mut f = Fixture::new();
    f.state
        .encounter
        .as_mut()
        .unwrap()
        .battlefield
        .obstacles
        .push(SpatialObstacle {
            id: "wall".into(),
            volume: SpatialBox {
                min: point(35, 0, 0),
                max: point(36, 30, 30),
            },
            blocks_movement: true,
            blocks_sight: true,
            observable: true,
            cover: CoverDegree::Total,
        });
    f.begin();
    let event = f.run(Some(0), walk(&[point(20, 10, 0), point(30, 10, 0)]));
    assert_eq!(f.position(0), point(20, 10, 0));
    assert_eq!(f.flow().budget.movement_spent, 10);
    let result = f.flow().last_movement.as_ref().unwrap();
    assert_eq!(result.original, event.meta);
    assert_eq!(result.cause, event.meta);
    assert_eq!(result.reason, TacticalMovementEnd::Stopped);
    assert_eq!((result.completed_steps, result.requested_steps), (1, 2));
    f.run(
        Some(0),
        walk(&[
            point(20, 20, 0),
            point(20, 30, 0),
            point(20, 40, 0),
            point(20, 50, 0),
            point(20, 60, 0),
            point(20, 70, 0),
        ]),
    );
    assert_eq!(f.position(0), point(20, 60, 0));
    assert_eq!(f.flow().budget.movement_spent, 60);
    let result = f.flow().last_movement.as_ref().unwrap();
    assert_eq!((result.spent_before, result.spent_after), (10, 60));
    assert_eq!((result.completed_steps, result.requested_steps), (5, 6));
    assert_eq!(result.reason, TacticalMovementEnd::Stopped);
    assert!(!f.rules().timing.as_ref().unwrap().action_spent);
}

#[test]
fn hidden_solid_near_far_or_absent_cannot_reject_the_same_travel_attempt() {
    let mut f = Fixture::new();
    let encounter = f.state.encounter.as_mut().unwrap();
    encounter.participants[1].position = point(0, 60, 0);
    encounter.participants[2].position = point(0, 80, 0);
    f.begin();
    let initial = f.state.clone();
    let meta = f.meta(Some(0));
    let action = walk(&[
        point(20, 10, 0),
        point(30, 10, 0),
        point(40, 10, 0),
        point(50, 10, 0),
    ]);
    let view =
        spatial::project_actor_view(initial.encounter.as_ref().unwrap(), &initial, f.actors[0])
            .unwrap();
    let mut first_event = None;
    for (blocker, end, cost, count) in [
        (Some(25), 10, 0, 0),
        (Some(45), 30, 20, 2),
        (None, 50, 40, 4),
    ] {
        f.state = initial.clone();
        if let Some(x) = blocker {
            f.state
                .encounter
                .as_mut()
                .unwrap()
                .battlefield
                .obstacles
                .push(SpatialObstacle {
                    id: "secret-pressure-screen".into(),
                    volume: SpatialBox {
                        min: point(x, 0, 0),
                        max: point(x + 1, 30, 30),
                    },
                    blocks_movement: true,
                    blocks_sight: false,
                    observable: false,
                    cover: CoverDegree::None,
                });
        }
        assert_eq!(
            spatial::project_actor_view(f.state.encounter.as_ref().unwrap(), &f.state, f.actors[0])
                .unwrap(),
            view
        );
        let event = f.run_meta(&meta, &action);
        if let Some(expected) = &first_event {
            assert_eq!(&event, expected);
        } else {
            first_event = Some(event);
        }
        assert_eq!(f.position(0), point(end, 10, 0));
        assert_eq!(f.flow().budget.movement_spent, cost);
        let result = f.flow().last_movement.as_ref().unwrap();
        assert_eq!(result.completed_steps, count);
        assert_eq!(result.endpoint, f.position(0));
        assert_eq!(
            result.reason,
            if blocker.is_some() {
                TacticalMovementEnd::Stopped
            } else {
                TacticalMovementEnd::Completed
            }
        );
        assert!(
            !serde_json::to_string(result)
                .unwrap()
                .contains("secret-pressure-screen")
        );
    }
}

#[test]
fn an_unseen_creature_stops_only_the_crossing_that_reaches_its_space() {
    let mut f = Fixture::new();
    f.state.encounter.as_mut().unwrap().participants[2].position = point(0, 80, 0);
    f.state.rules.as_mut().unwrap().effects.push(ActiveEffect {
        id: EffectId::new(),
        source: f.actors[1],
        target: f.actors[1],
        condition: Some(Condition::Invisible),
        label: "Source invisibility".into(),
        expires: Expiry::Never,
        concentration_owner: None,
    });
    f.begin();
    let initial = f.state.clone();
    let meta = f.meta(Some(0));
    let action = walk(&[
        point(20, 10, 0),
        point(30, 10, 0),
        point(40, 10, 0),
        point(50, 10, 0),
    ]);
    for (other, end, cost) in [(point(40, 10, 0), 30, 20), (point(0, 60, 0), 50, 40)] {
        f.state = initial.clone();
        f.state.encounter.as_mut().unwrap().participants[1].position = other;
        let view =
            spatial::project_actor_view(f.state.encounter.as_ref().unwrap(), &f.state, f.actors[0])
                .unwrap();
        assert!(
            !view
                .contacts
                .iter()
                .any(|contact| contact.entity_id == f.actors[1])
        );
        let knowledge = f.state.encounter.as_ref().unwrap().knowledge.clone();
        f.run_meta(&meta, &action);
        assert_eq!(f.position(0), point(end, 10, 0));
        assert_eq!(f.flow().budget.movement_spent, cost);
        assert_eq!(f.state.encounter.as_ref().unwrap().knowledge, knowledge);
        assert!(
            !serde_json::to_string(f.flow().last_movement.as_ref().unwrap())
                .unwrap()
                .contains(&f.actors[1].0.to_string())
        );
    }
}

#[test]
fn hidden_difficult_terrain_changes_only_cost_of_segments_actually_traversed() {
    let mut f = Fixture::new();
    f.state.encounter.as_mut().unwrap().participants[1].position = point(0, 60, 0);
    f.state.encounter.as_mut().unwrap().participants[2].position = point(0, 80, 0);
    f.begin();
    let initial = f.state.clone();
    let meta = f.meta(Some(0));
    let action = walk(&[
        point(20, 10, 0),
        point(30, 10, 0),
        point(40, 10, 0),
        point(50, 10, 0),
    ]);
    for difficult in [false, true] {
        f.state = initial.clone();
        if difficult {
            f.state
                .encounter
                .as_mut()
                .unwrap()
                .battlefield
                .terrain
                .push(TerrainVolume {
                    id: "concealed-slowing-floor".into(),
                    volume: SpatialBox {
                        min: point(20, 0, 0),
                        max: point(60, 30, 20),
                    },
                    difficult: true,
                    observable: false,
                    water: false,
                    climbable: false,
                    burrowable: false,
                    supports_top: false,
                    surface: None,
                    obscuration: Obscuration::None,
                    magical_darkness: false,
                });
        }
        f.run_meta(&meta, &action);
        assert_eq!(f.position(0), point(if difficult { 40 } else { 50 }, 10, 0));
        assert_eq!(
            f.flow().budget.movement_spent,
            if difficult { 60 } else { 40 }
        );
        assert_eq!(
            f.flow().last_movement.as_ref().unwrap().reason,
            if difficult {
                TacticalMovementEnd::Stopped
            } else {
                TacticalMovementEnd::Completed
            }
        );
    }
}

#[test]
fn an_unexpected_stop_preserves_legal_transit_into_an_allys_space() {
    let mut f = Fixture::new();
    let encounter = f.state.encounter.as_mut().unwrap();
    encounter.participants[0].allies = vec![f.actors[1]];
    encounter.participants[1].position = point(20, 10, 0);
    encounter.battlefield.obstacles.push(SpatialObstacle {
        id: "concealed-screen".into(),
        volume: SpatialBox {
            min: point(35, 0, 0),
            max: point(36, 30, 30),
        },
        blocks_movement: true,
        blocks_sight: false,
        observable: false,
        cover: CoverDegree::None,
    });
    f.begin();
    f.run(Some(0), walk(&[point(20, 10, 0), point(30, 10, 0)]));
    assert_eq!(f.position(0), f.position(1));
    assert_eq!(f.flow().budget.movement_spent, 10);
    assert_eq!(
        f.flow().last_movement.as_ref().unwrap().reason,
        TacticalMovementEnd::Stopped
    );
    // This stop was involuntary. SRD14's shared-space Prone consequence is due only
    // if the actor ends its turn here; a later legal move can still leave the space.
    let stopped = f.state.clone();
    f.run(Some(0), walk(&[point(20, 20, 0)]));
    assert_eq!(f.position(0), point(20, 20, 0));
    assert_eq!(f.flow().budget.movement_spent, 20);
    assert!(!active_conditions(f.rules(), f.actors[0]).contains(&Condition::Prone));
    f.state = stopped;
    f.run(Some(0), TacticalAction::EndTurn);
    assert!(active_conditions(f.rules(), f.actors[0]).contains(&Condition::Prone));
    assert_eq!(
        f.flow().last_movement.as_ref().unwrap().endpoint,
        point(20, 10, 0)
    );
}

#[test]
fn relevant_uncertain_opportunity_cover_stops_at_departure_without_rolling_back_prefix() {
    let mut f = Fixture::new();
    f.arm("longsword", false, false);
    let encounter = f.state.encounter.as_mut().unwrap();
    encounter.battlefield.obstacles.push(SpatialObstacle {
        id: "unseen-partial-reaction-screen".into(),
        volume: SpatialBox {
            min: point(20, 0, 0),
            max: point(21, 100, 100),
        },
        blocks_movement: false,
        blocks_sight: false,
        observable: false,
        cover: CoverDegree::Total,
    });
    let assessed = spatial::cover_from(
        encounter,
        encounter.participants[0].center().unwrap(),
        encounter.participants[1].volume().unwrap(),
        &f.actors,
    )
    .unwrap();
    assert!(assessed.requires_adjudication);
    f.begin_mover_turn();
    let event = f.run(Some(1), walk(&[point(20, 20, 0), point(30, 20, 0)]));
    // The first segment stays in reach, so its unrelated cover uncertainty is
    // irrelevant. The second would depart; do not cross a possible OA or undo step1.
    assert_eq!(f.position(1), point(20, 20, 0));
    assert_eq!(f.flow().budget.movement_spent, 10);
    assert!(f.flow().resolution.is_none());
    assert!(
        !f.rules()
            .timing
            .as_ref()
            .unwrap()
            .reactions_spent
            .contains(&f.actors[0])
    );
    let result = f.flow().last_movement.as_ref().unwrap();
    assert_eq!(result.original, event.meta);
    assert_eq!(result.reason, TacticalMovementEnd::Stopped);
    assert_eq!(result.completed_steps, 1);
    assert!(
        !serde_json::to_string(result)
            .unwrap()
            .contains("unseen-partial-reaction-screen")
    );
}

#[test]
fn malformed_and_unavailable_source_intent_still_reject_atomically() {
    let mut f = Fixture::new();
    f.state.encounter.as_mut().unwrap().participants[0]
        .movement
        .fly = None;
    f.begin();
    for path in [
        vec![],
        vec![TacticalMoveStep {
            destination: point(30, 10, 0),
            mode: MovementMode::Walk,
        }],
        vec![TacticalMoveStep {
            destination: point(11, 10, 0),
            mode: MovementMode::Walk,
        }],
        vec![
            TacticalMoveStep {
                destination: point(20, 10, 0),
                mode: MovementMode::Walk
            };
            1025
        ],
        vec![TacticalMoveStep {
            destination: point(10, 10, 10),
            mode: MovementMode::Fly,
        }],
        vec![TacticalMoveStep {
            destination: point(20, 10, 0),
            mode: MovementMode::Burrow,
        }],
        vec![TacticalMoveStep {
            destination: point(20, 10, 0),
            mode: MovementMode::Teleport,
        }],
    ] {
        f.rejected(Some(0), TacticalAction::Move { path });
    }
    f.entity_mut(0).prone = true;
    f.rejected(Some(0), walk(&[point(20, 10, 0)]));
    assert!(f.flow().last_movement.is_none());
    assert_eq!(f.flow().budget.movement_spent, 0);
}

#[test]
fn completed_receipt_survives_turn_reset_and_rejects_incoherent_anchor_claims() {
    let mut f = Fixture::new();
    f.begin();
    f.run(Some(0), walk(&[point(10, 20, 0), point(10, 30, 0)]));
    let result = f.flow().last_movement.clone();
    f.run(Some(0), TacticalAction::EndTurn);
    assert_eq!(f.flow().budget.movement_spent, 0);
    assert_eq!(f.flow().last_movement, result);
    for mutation in 0..9 {
        let mut corrupt = f.state.clone();
        let result = corrupt
            .encounter
            .as_mut()
            .unwrap()
            .flow
            .as_mut()
            .unwrap()
            .last_movement
            .as_mut()
            .unwrap();
        match mutation {
            0 => result.original.actor = Some(AgentRef::Entity(f.actors[1])),
            1 => result.cause.campaign_id = CampaignId::new(),
            2 => result.completed_steps += 1,
            3 => result.reason = TacticalMovementEnd::Stopped,
            4 => result.spent_after = 0,
            5 => result.spent_after = 100,
            6 => result.endpoint.x = 500,
            7 => result.turn_number = u64::MAX,
            8 => result.cause.id = CommandId::new(),
            _ => unreachable!(),
        }
        assert!(
            validate_tactical_state(&corrupt).is_err(),
            "result mutation {mutation}"
        );
    }
    // Explicit completed-combat snapshot, not a public EndCombat operation claim.
    // A later inactive state must still validate the retained historical receipt.
    f.state.rules.as_mut().unwrap().timing = None;
    f.state
        .encounter
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .phase = TacticalPhase::Finished;
    validate_tactical_state(&f.state).unwrap();
    f.state
        .encounter
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .last_movement
        .as_mut()
        .unwrap()
        .spent_after = 0;
    assert!(validate_tactical_state(&f.state).is_err());
}

#[test]
fn initiative_cannot_smuggle_a_structurally_coherent_movement_receipt() {
    let mut f = Fixture::new();
    f.run(
        None,
        TacticalAction::Begin {
            combatants: f
                .actors
                .into_iter()
                .map(|actor| TacticalCombatant {
                    actor,
                    source: TacticalSource::Character,
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
        },
    );
    validate_tactical_state(&f.state).unwrap();
    let original = f.meta(Some(0));
    f.state
        .encounter
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .last_movement = Some(TacticalMovementResult {
        original: original.clone(),
        cause: original,
        actor: f.actors[0],
        turn_number: 1,
        start: point(10, 10, 0),
        endpoint: point(20, 10, 0),
        requested_steps: 1,
        completed_steps: 1,
        spent_before: 0,
        spent_after: 10,
        reason: TacticalMovementEnd::Completed,
    });
    assert!(validate_tactical_state(&f.state).is_err());
}

#[test]
fn opportunity_decline_keeps_position_and_cost_uncommitted_until_actual_response() {
    let mut f = Fixture::new();
    f.arm("longsword", false, false);
    f.begin_mover_turn();
    let accepted = f.run(Some(1), walk(&[point(30, 10, 0)]));
    assert_eq!(f.position(1), point(20, 10, 0));
    assert_eq!(f.flow().budget.movement_spent, 0);
    assert_eq!(f.movement().origin, accepted.meta);
    let window = f.movement().opportunity.as_ref().unwrap();
    assert_eq!((window.reactor, window.mover), (f.actors[0], f.actors[1]));
    assert_eq!(
        (window.from, window.to),
        (point(20, 10, 0), point(30, 10, 0))
    );
    f.rejected(Some(1), TacticalAction::DeclineOpportunity);
    f.rejected(Some(1), TacticalAction::EndTurn);
    f.run(Some(0), TacticalAction::DeclineOpportunity);
    assert_eq!(f.position(1), point(30, 10, 0));
    assert_eq!(f.flow().budget.movement_spent, 10);
    assert!(
        !f.rules()
            .timing
            .as_ref()
            .unwrap()
            .reactions_spent
            .contains(&f.actors[0])
    );
    assert!(f.flow().resolution.is_none());
    assert!(matches!(
        resolve_tactical(&f.state, &accepted.meta, &accepted.action, &f.pack),
        Err(RulesError::Stale)
    ));
}

#[test]
fn opportunity_raw_attack_and_damage_finish_above_the_original_movement_cursor() {
    let mut f = Fixture::new();
    let choice = f.arm("longsword", false, false);
    f.begin_mover_turn();
    let turn = f.rules().timing.as_ref().unwrap().turn_number;
    let moving = f.run(Some(1), walk(&[point(30, 10, 0)]));
    let reacting = f.run(
        Some(0),
        TacticalAction::OpportunityAttack {
            choice: TacticalMeleeChoice::Weapon(choice),
        },
    );
    let resolution = f.flow().resolution.as_ref().unwrap();
    assert_eq!(resolution.origin, moving.meta);
    assert_eq!(resolution.attack.as_ref().unwrap().origin, reacting.meta);
    assert_eq!(f.request().roller, Some(f.actors[0]));
    assert_eq!(f.position(1), point(20, 10, 0));
    assert_eq!(f.flow().budget.movement_spent, 0);
    assert!(
        f.rules()
            .timing
            .as_ref()
            .unwrap()
            .reactions_spent
            .contains(&f.actors[0])
    );
    f.rejected(
        Some(1),
        TacticalAction::SubmitRoll {
            result: f.raw(&[15]),
        },
    );
    f.roll(0, &[15]);
    assert_eq!(f.position(1), point(20, 10, 0));
    assert_eq!(f.request().dice, [DieSpec { count: 1, sides: 8 }]);
    f.roll(0, &[1]);
    assert_eq!(f.rules().entities[&f.actors[1]].hp, 46);
    assert_eq!(f.position(1), point(30, 10, 0));
    assert_eq!(f.flow().budget.movement_spent, 10);
    assert_eq!(f.rules().timing.as_ref().unwrap().turn_number, turn);
    assert!(!f.rules().timing.as_ref().unwrap().action_spent);
    assert!(f.flow().resolution.is_none());
}

#[test]
fn knocking_out_the_mover_stops_the_uncommitted_path_without_refunding_the_reaction() {
    let mut f = Fixture::new();
    let choice = f.arm("longsword", false, false);
    f.entity_mut(1).hp = 3;
    f.begin_mover_turn();
    let original = f.run(Some(1), walk(&[point(30, 10, 0), point(40, 10, 0)]));
    f.run(
        Some(0),
        TacticalAction::OpportunityAttack {
            choice: TacticalMeleeChoice::Weapon(choice),
        },
    );
    f.roll(0, &[15]);
    f.roll(0, &[8]);
    assert_eq!(f.position(1), point(20, 10, 0));
    assert_eq!(f.rules().entities[&f.actors[1]].hp, 3);
    f.rejected(
        Some(1),
        TacticalAction::ChooseAttackKnockout {
            choice: KnockoutChoice::KnockOut,
        },
    );
    let cause = f.run(
        Some(0),
        TacticalAction::ChooseAttackKnockout {
            choice: KnockoutChoice::KnockOut,
        },
    );
    let result = f.flow().last_movement.as_ref().unwrap();
    assert_eq!(result.original, original.meta);
    assert_eq!(result.cause, cause.meta);
    assert_eq!(result.reason, TacticalMovementEnd::Interrupted);
    assert_eq!((result.completed_steps, result.spent_after), (0, 0));
    assert_eq!(f.rules().entities[&f.actors[1]].hp, 1);
    assert!(active_conditions(f.rules(), f.actors[1]).contains(&Condition::Unconscious));
    assert_eq!(f.position(1), point(20, 10, 0));
    assert_eq!(f.flow().budget.movement_spent, 0);
    assert!(
        f.rules()
            .timing
            .as_ref()
            .unwrap()
            .reactions_spent
            .contains(&f.actors[0])
    );
    assert!(f.flow().resolution.is_none());
    f.run(Some(1), TacticalAction::EndTurn);
}

#[test]
fn weapon_reach_and_unarmed_reach_offer_distinct_crossings_without_generic_reach_permissions() {
    let mut f = Fixture::new();
    let choice = f.arm("halberd", false, false);
    f.begin_mover_turn();
    f.run(Some(1), walk(&[point(30, 10, 0), point(40, 10, 0)]));
    let window = f.movement().opportunity.as_ref().unwrap();
    assert!(window.options.iter().all(|o| o.reach == 10));
    assert!(
        window
            .options
            .iter()
            .any(|o| o.source == TacticalMeleeSource::Unarmed)
    );
    f.run(Some(0), TacticalAction::DeclineOpportunity);
    assert_eq!(f.position(1), point(30, 10, 0));
    assert_eq!(f.flow().budget.movement_spent, 10);
    let window = f.movement().opportunity.as_ref().unwrap();
    assert!(window.options.iter().any(|o| o.source
        == TacticalMeleeSource::Weapon {
            item: choice.weapon
        }
        && o.reach == 20));
    assert!(
        window
            .options
            .iter()
            .all(|o| o.source != TacticalMeleeSource::Unarmed)
    );
    f.run(Some(0), TacticalAction::DeclineOpportunity);
    assert_eq!(f.position(1), point(40, 10, 0));
    assert_eq!(f.flow().budget.movement_spent, 20);
}

#[test]
fn disengage_and_unseen_mover_do_not_create_an_opportunity_from_hidden_truth() {
    for disengage in [false, true] {
        let mut f = Fixture::new();
        f.arm("longsword", false, false);
        if !disengage {
            f.state
                .encounter
                .as_mut()
                .unwrap()
                .battlefield
                .ambient_light = LightLevel::Darkness;
        }
        f.begin_mover_turn();
        if disengage {
            f.run(Some(1), TacticalAction::Disengage);
        }
        f.run(Some(1), walk(&[point(30, 10, 0)]));
        assert_eq!(f.position(1), point(30, 10, 0));
        assert_eq!(f.flow().budget.movement_spent, 10);
        assert!(f.flow().resolution.is_none());
        assert!(
            !f.rules()
                .timing
                .as_ref()
                .unwrap()
                .reactions_spent
                .contains(&f.actors[0])
        );
    }
}

#[test]
fn malformed_saved_crossing_cannot_change_reactor_reach_cost_or_cursor() {
    let mut f = Fixture::new();
    f.arm("longsword", false, false);
    f.begin_mover_turn();
    f.run(Some(1), walk(&[point(30, 10, 0)]));
    for mutation in 0..10 {
        let mut corrupt = f.state.clone();
        let movement = corrupt
            .encounter
            .as_mut()
            .unwrap()
            .flow
            .as_mut()
            .unwrap()
            .resolution
            .as_mut()
            .unwrap()
            .movement
            .as_mut()
            .unwrap();
        match mutation {
            0 => movement.next_step = 1,
            1 => movement.initial_spent = 10,
            2 => movement.opportunity.as_mut().unwrap().reactor = f.actors[1],
            3 => movement.opportunity.as_mut().unwrap().to = point(40, 10, 0),
            4 => movement.opportunity.as_mut().unwrap().options[0].reach = 1000,
            5 => movement.initial_progress.walked_runup = 20,
            6 => movement.origin.actor = Some(AgentRef::Entity(f.actors[0])),
            7 => movement.offered.clear(),
            8 => movement.opportunity.as_mut().unwrap().origin.campaign_id = CampaignId::new(),
            9 => {
                movement
                    .opportunity
                    .as_mut()
                    .unwrap()
                    .origin
                    .expected_event_sequence = movement
                    .origin
                    .expected_event_sequence
                    .checked_sub(1)
                    .unwrap();
            }
            _ => unreachable!(),
        }
        assert!(
            validate_tactical_state(&corrupt).is_err(),
            "mutation {mutation}"
        );
    }
}

#[test]
fn concentration_failure_finishes_before_the_mover_commits_its_crossing() {
    let mut f = Fixture::new();
    let choice = f.arm("longsword", false, false);
    let meta = f.meta(None);
    let group = EffectId::new();
    f.state = tactical_effect_adapter::apply_effect_operation(
        &f.state,
        &meta,
        &EffectLifecycleAction {
            step: 0,
            operation: EffectLifecycleOperation::BeginConcentration {
                group: ConcentrationGroup {
                    id: group,
                    source: EffectSource {
                        definition_id: "retained-source-focus".into(),
                        actor: f.actors[1],
                        command: meta.clone(),
                        ordinal: 0,
                    },
                    expires: TacticalEffectExpiry::Never,
                    stage: ConcentrationStage::Casting,
                },
            },
        },
    )
    .unwrap()
    .0;
    f.state.applied_event_sequence += 1;
    f.begin_mover_turn();
    let original = f.run(Some(1), walk(&[point(30, 10, 0)]));
    f.run(
        Some(0),
        TacticalAction::OpportunityAttack {
            choice: TacticalMeleeChoice::Weapon(choice),
        },
    );
    f.roll(0, &[15]);
    f.roll(0, &[1]);
    assert_eq!(f.position(1), point(20, 10, 0));
    assert_eq!(f.flow().budget.movement_spent, 0);
    assert!(f.flow().resolution.as_ref().unwrap().attack.is_none());
    assert_eq!(f.request().roller, Some(f.actors[1]));
    assert!(
        matches!(f.flow().resolution.as_ref().unwrap().pending.as_ref().unwrap().work.kind,
        TacticalWorkKind::ConcentrationSave { actor, group: source, damage_taken: 4 } if actor == f.actors[1] && source == group)
    );
    f.rejected(Some(0), TacticalAction::VoluntarilyFailSave);
    let cause = f.run(Some(1), TacticalAction::VoluntarilyFailSave);
    assert_eq!(f.rules().entities[&f.actors[1]].concentration, None);
    assert_eq!(f.position(1), point(30, 10, 0));
    assert_eq!(f.flow().budget.movement_spent, 10);
    let result = f.flow().last_movement.as_ref().unwrap();
    assert_eq!(result.original, original.meta);
    assert_eq!(result.cause, cause.meta);
    assert_eq!(result.reason, TacticalMovementEnd::Completed);
}

#[test]
fn current_turn_controller_orders_independent_opportunities_but_cannot_decline_them() {
    let mut f = Fixture::new();
    f.arm("longsword", false, false);
    let third = &mut f.state.encounter.as_mut().unwrap().participants[2];
    third.position = point(10, 20, 0);
    third.enemies = vec![f.actors[1]];
    f.begin_mover_turn();
    f.run(Some(1), walk(&[point(30, 10, 0)]));
    assert!(f.movement().opportunity.is_none());
    let work = f.flow().resolution.as_ref().unwrap().frames.last().unwrap();
    assert_eq!(work.len(), 2);
    let chosen = work
        .iter()
        .find(|w| {
            matches!(w.kind,
        TacticalWorkKind::MovementOpportunity { reactor } if reactor == f.actors[2])
        })
        .unwrap()
        .occurrence;
    f.rejected(
        Some(0),
        TacticalAction::ChooseTurnWork { occurrence: chosen },
    );
    f.run(
        Some(1),
        TacticalAction::ChooseTurnWork { occurrence: chosen },
    );
    assert_eq!(
        f.movement().opportunity.as_ref().unwrap().reactor,
        f.actors[2]
    );
    f.rejected(Some(1), TacticalAction::DeclineOpportunity);
    f.run(Some(2), TacticalAction::DeclineOpportunity);
    assert_eq!(
        f.movement().opportunity.as_ref().unwrap().reactor,
        f.actors[0]
    );
    assert_eq!(f.position(1), point(20, 10, 0));
    f.run(Some(0), TacticalAction::DeclineOpportunity);
    assert_eq!(f.position(1), point(30, 10, 0));
    assert!(
        f.rules()
            .timing
            .as_ref()
            .unwrap()
            .reactions_spent
            .is_empty()
    );
}
