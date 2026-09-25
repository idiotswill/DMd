use dmd_domain::*;
use dmd_rules::{tactical::*, tactical_effects::*, tactical_inventory::*, *};
use std::collections::HashMap;
#[path = "tactical_attacks/areas.rs"]
mod areas;
#[path = "tactical_attacks/casting.rs"]
mod casting;
#[path = "tactical_attacks/creature.rs"]
mod creature;
#[path = "tactical_attacks/creature_weapon.rs"]
mod creature_weapon;
#[path = "tactical_attacks/medicine.rs"]
mod medicine;
#[path = "tactical_attacks/opportunity.rs"]
mod opportunity;
#[path = "tactical_attacks/savage.rs"]
mod savage;
#[path = "tactical_attacks/second_wind.rs"]
mod second_wind;
#[path = "tactical_attacks/shields.rs"]
mod shields;
#[path = "tactical_attacks/spell.rs"]
mod spell;
#[path = "tactical_attacks/unarmed.rs"]
mod unarmed;

struct Fixture {
    state: CampaignState,
    pack: RulesPack,
    actors: [EntityId; 2],
    players: [PlayerId; 2],
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
    fn entity_mut(&mut self, index: usize) -> &mut MechanicalEntity {
        self.state
            .rules
            .as_mut()
            .unwrap()
            .entities
            .get_mut(&self.actors[index])
            .unwrap()
    }
    fn zero_hp_target(&mut self, dead: bool) {
        self.entity_mut(1).hp = 0;
        self.entity_mut(1).prone = true;
        self.entity_mut(1).death.dead = dead;
        if dead {
            self.state
                .entities
                .get_mut(&self.actors[1])
                .unwrap()
                .existence = EntityExistence::Dead;
            self.state
                .characters
                .values_mut()
                .find(|character| character.entity_id == self.actors[1])
                .unwrap()
                .status = CharacterStatus::Dead;
        }
        validate_state(&self.state, &self.pack).unwrap();
        validate_tactical_state(&self.state).unwrap();
        assert!(self.state.validate().is_empty());
    }
    fn run(&mut self, actor: Option<usize>, action: TacticalAction) -> TacticalEvent {
        let meta = self.meta(actor);
        let before = self.state.clone();
        let transition = resolve_tactical(&self.state, &meta, &action, &self.pack).unwrap();
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
                execution: dmd_domain::TacticalExecutionVersion::ReactionsV1,
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
            self.run(Some(index), TacticalAction::SubmitRoll { result });
        }
    }
    fn loadout(&self) -> &ActorEquipmentLoadout {
        self.rules()
            .tactical_inventory
            .as_ref()
            .unwrap()
            .loadout(self.actors[0])
            .unwrap()
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

#[test]
fn source_attack_raw_damage_replays_and_preserves_same_turn_and_physical_ownership() {
    let mut f = Fixture::new();
    let choice = f.arm("longsword", false, false);
    f.begin();
    let turn = f.rules().timing.clone();
    let hp = f.rules().entities[&f.actors[1]].hp;
    let accepted = f.run(
        Some(0),
        TacticalAction::Attack {
            choice: choice.clone(),
        },
    );
    assert_eq!(f.request().modifier, 5);
    assert_eq!(
        f.request().dice,
        vec![DieSpec {
            count: 1,
            sides: 20
        }]
    );
    assert_eq!(f.request().roller, Some(f.actors[0]));
    assert_eq!(
        f.rules().timing.as_ref().unwrap().turn_number,
        turn.as_ref().unwrap().turn_number
    );
    assert!(f.rules().timing.as_ref().unwrap().action_spent);
    assert_eq!(f.flow().budget.attacks_remaining, 0);
    f.rejected(
        Some(1),
        TacticalAction::SubmitRoll {
            result: f.raw(&[15]),
        },
    );
    f.rejected(Some(0), TacticalAction::EndTurn);
    let stale = f.meta(Some(0));
    f.roll(0, &[15]);
    assert_eq!(f.request().dice, vec![DieSpec { count: 1, sides: 8 }]);
    assert_eq!(f.request().modifier, 0);
    assert_eq!(f.rules().entities[&f.actors[1]].hp, hp);
    assert!(
        resolve_tactical(
            &f.state,
            &stale,
            &TacticalAction::SubmitRoll {
                result: f.raw(&[8])
            },
            &f.pack
        )
        .is_err()
    );
    f.roll(0, &[8]);
    assert_eq!(f.rules().entities[&f.actors[1]].hp, hp - 11);
    assert!(f.flow().resolution.is_none());
    assert!(f.rules().pending.is_none());
    assert_eq!(f.flow().budget.weapon_history[0].origin, accepted.meta);
    assert_eq!(
        f.flow().budget.weapon_history[0].outcome,
        WeaponAttackOutcome::Hit {
            critical: false,
            damage_dealt: 11
        }
    );
    assert_eq!(
        f.state.items[&choice.weapon].owner,
        Ownership::Entity(f.actors[1])
    );
    f.rejected(Some(0), TacticalAction::Attack { choice });
    f.run(Some(0), TacticalAction::EndTurn);
    assert_eq!(f.rules().timing.as_ref().unwrap().turn_number, 2);
}

#[test]
fn ordinary_attack_rejects_dead_body_before_cost_but_can_damage_living_zero_hp() {
    for dead in [true, false] {
        let mut f = Fixture::new();
        let choice = f.arm("shortbow", false, true);
        let ammunition = choice.ammunition.unwrap();
        f.begin();
        f.zero_hp_target(dead);
        if dead {
            let before = f.state.clone();
            let error = resolve_tactical(
                &f.state,
                &f.meta(Some(0)),
                &TacticalAction::Attack { choice },
                &f.pack,
            )
            .unwrap_err();
            assert!(
                matches!(error, RulesError::Prerequisite(message) if message.contains("body/object"))
            );
            assert_eq!(f.state, before);
            assert_eq!(f.state.items[&ammunition].quantity, 20);
            assert!(!f.rules().timing.as_ref().unwrap().action_spent);
        } else {
            f.run(Some(0), TacticalAction::Attack { choice });
            assert_eq!(f.request().mode, RollMode::Normal);
            f.roll(0, &[15]);
            f.roll(0, &[1]);
            assert_eq!(f.rules().entities[&f.actors[1]].death.failures, 1);
            assert!(!f.rules().entities[&f.actors[1]].death.dead);
            assert!(f.flow().resolution.is_none());
            assert_eq!(f.state.items[&ammunition].quantity, 19);
        }
    }
}

#[test]
fn lethal_ordinary_attack_finishes_with_replayable_dead_target() {
    let mut f = Fixture::new();
    let choice = f.arm("shortbow", false, true);
    f.entity_mut(1).hp = 2;
    f.entity_mut(1).max_hp = 2;
    f.begin();
    f.run(Some(0), TacticalAction::Attack { choice });
    f.roll(0, &[15]);
    f.roll(0, &[6]);
    assert!(f.rules().entities[&f.actors[1]].death.dead);
    assert!(f.flow().resolution.is_none());
    assert!(f.rules().pending.is_none());
}

#[test]
fn natural_one_misses_and_twenty_doubles_only_damage_dice() {
    for face in [1, 20] {
        let mut f = Fixture::new();
        let choice = f.arm("longsword", false, false);
        f.entity_mut(1).armor = ArmorClass::Fixed(if face == 1 { 5 } else { 30 });
        f.begin();
        f.run(Some(0), TacticalAction::Attack { choice });
        f.roll(0, &[face]);
        if face == 1 {
            assert!(f.rules().pending.is_none());
            assert_eq!(f.rules().entities[&f.actors[1]].hp, 50);
            assert_eq!(
                f.flow().budget.weapon_history[0].outcome,
                WeaponAttackOutcome::Miss
            );
        } else {
            assert_eq!(f.request().dice, vec![DieSpec { count: 2, sides: 8 }]);
            f.roll(0, &[8, 8]);
            assert_eq!(f.rules().entities[&f.actors[1]].hp, 31);
        }
    }
}

#[test]
fn ammunition_is_reserved_once_and_last_unit_retains_spent_identity() {
    let mut f = Fixture::new();
    let choice = f.arm("shortbow", false, true);
    let ammo = choice.ammunition.unwrap();
    f.state.items.get_mut(&ammo).unwrap().quantity = 1;
    f.begin();
    f.run(
        Some(0),
        TacticalAction::Attack {
            choice: choice.clone(),
        },
    );
    assert_eq!(f.state.items[&ammo].quantity, 0);
    assert_eq!(f.state.items[&ammo].state, ItemState::Spent);
    let bad = RollResult {
        dice: vec![DieResult {
            sides: 20,
            value: 21,
        }],
        ..f.raw(&[1])
    };
    f.rejected(Some(0), TacticalAction::SubmitRoll { result: bad });
    f.roll(0, &[1]);
    assert_eq!(f.state.items[&ammo].quantity, 0);
    f.run(Some(0), TacticalAction::EndTurn);
    f.run(Some(1), TacticalAction::EndTurn);
    f.rejected(Some(0), TacticalAction::Attack { choice });
    assert!(!f.rules().timing.as_ref().unwrap().action_spent);
}

#[test]
fn illegal_source_choices_and_unimplemented_masteries_reject_before_any_cost() {
    let mut f = Fixture::new();
    let choice = f.arm("shortbow", true, true);
    f.begin();
    f.rejected(
        Some(0),
        TacticalAction::Attack {
            choice: choice.clone(),
        },
    ); // Vex needs a typed effect continuation.
    assert!(!f.rules().timing.as_ref().unwrap().action_spent);
    assert_eq!(f.state.items[&choice.ammunition.unwrap()].quantity, 20);
    let mut f = Fixture::new();
    let choice = f.arm("longsword", false, false);
    f.begin();
    for mutation in 0..4 {
        let mut bad = choice.clone();
        match mutation {
            0 => bad.ability = Ability::Charisma,
            1 => bad.weapon = ItemId::new(),
            2 => bad.target = EntityId::new(),
            3 => bad.ammunition = Some(choice.weapon),
            _ => unreachable!(),
        };
        f.rejected(Some(0), TacticalAction::Attack { choice: bad });
    }
    f.rejected(Some(1), TacticalAction::Attack { choice });
    assert!(!f.rules().timing.as_ref().unwrap().action_spent);
}

#[test]
fn melee_knockout_is_owned_choice_before_hp_commit_and_survives_serialization() {
    let mut f = Fixture::new();
    let choice = f.arm("longsword", false, false);
    f.entity_mut(1).hp = 3;
    f.begin();
    f.run(Some(0), TacticalAction::Attack { choice });
    f.roll(0, &[15]);
    f.roll(0, &[8]);
    assert_eq!(f.rules().entities[&f.actors[1]].hp, 3);
    assert!(f.rules().pending.is_none());
    assert_eq!(
        f.flow()
            .resolution
            .as_ref()
            .unwrap()
            .attack
            .as_ref()
            .unwrap()
            .stage,
        TacticalAttackStage::KnockoutChoice
    );
    f.rejected(
        Some(1),
        TacticalAction::ChooseAttackKnockout {
            choice: KnockoutChoice::KnockOut,
        },
    );
    f.run(
        Some(0),
        TacticalAction::ChooseAttackKnockout {
            choice: KnockoutChoice::KnockOut,
        },
    );
    assert_eq!(f.rules().entities[&f.actors[1]].hp, 1);
    assert!(!f.rules().entities[&f.actors[1]].death.dead);
    assert!(f.flow().resolution.is_none());
    let recovery = &f.rules().tactical_recovery.as_ref().unwrap()[&f.actors[1]];
    let proof = recovery.knockout_rest.as_ref().unwrap();
    assert_eq!(proof.started_at, f.state.clock.now);
    assert_eq!(
        f.rules()
            .rests
            .iter()
            .find(|r| r.actor == f.actors[1])
            .unwrap()
            .started_at,
        proof.started_at
    );
    for mutation in 0..4 {
        let mut corrupt = f.state.clone();
        match mutation {
            0 => {
                corrupt
                    .rules
                    .as_mut()
                    .unwrap()
                    .tactical_recovery
                    .as_mut()
                    .unwrap()
                    .get_mut(&f.actors[1])
                    .unwrap()
                    .knockout_rest = None
            }
            1 => corrupt.rules.as_mut().unwrap().rests.clear(),
            2 => corrupt.rules.as_mut().unwrap().rests[0].started_at.0 -= 1,
            3 => {
                corrupt
                    .rules
                    .as_mut()
                    .unwrap()
                    .tactical_recovery
                    .as_mut()
                    .unwrap()
                    .get_mut(&f.actors[1])
                    .unwrap()
                    .knockout_rest
                    .as_mut()
                    .unwrap()
                    .started_by
                    .command
                    .campaign_id = CampaignId::new()
            }
            _ => unreachable!(),
        }
        assert!(
            validate_state(&corrupt, &f.pack).is_err(),
            "rest mutation {mutation}"
        );
    }
}

#[test]
fn waking_preserves_rest_until_an_accepted_strenuous_action() {
    use dmd_rules::tactical_damage::{VitalityContext, VitalityDefenses, reduce_vitality};
    for action_kind in 0..6 {
        let mut f = Fixture::new();
        let choice = f.arm("longsword", false, false);
        f.begin();
        let actor = f.actors[0];
        // Derive this earlier recovery anchor through the real internal vitality
        // reducer. The fixture's held weapon is subsequent equipment custody;
        // this test exercises rest admission, not the separately tested drop path.
        let mut context = VitalityContext {
            origin: VitalityOrigin {
                command: f.meta(None),
                occurrence: 0,
            },
            now: f.state.clock.now,
            conditions: Default::default(),
            underwater: false,
            defenses: VitalityDefenses::default(),
            death_save_mode: RollMode::Normal,
            death_save_bonus: 0,
        };
        let knocked = reduce_vitality(
            &f.rules().entities[&actor],
            &TacticalRecovery::default(),
            &context,
            &VitalityOperation::Damage {
                packet: DamagePacket {
                    cause: DamageCause::Attack {
                        attacker: f.actors[1],
                        melee: true,
                        critical: false,
                    },
                    components: vec![DamageComponent {
                        damage_type: DamageType::Bludgeoning,
                        amounts: vec![100],
                        adjustments: vec![],
                    }],
                },
                knockout: Some(KnockoutChoice::KnockOut),
            },
        )
        .unwrap();
        f.state.applied_event_sequence += 1;
        context.origin.command = f.meta(None);
        let awake = reduce_vitality(
            &knocked.entity,
            &knocked.recovery,
            &context,
            &VitalityOperation::Heal { amount: 1 },
        )
        .unwrap();
        let rules = f.state.rules.as_mut().unwrap();
        rules.entities.insert(actor, awake.entity);
        rules
            .tactical_recovery
            .get_or_insert_default()
            .insert(actor, awake.recovery);
        rules.rests.push(RestProgress {
            actor,
            kind: RestKind::Short,
            started_at: f.state.clock.now,
        });
        f.state.applied_event_sequence += 1;
        validate_state(&f.state, &f.pack).unwrap();
        assert!(!active_conditions(f.rules(), actor).contains(&Condition::Unconscious));
        let mut illegal = choice.clone();
        illegal.ability = Ability::Intelligence;
        f.rejected(Some(0), TacticalAction::Attack { choice: illegal });
        assert!(f.rules().rests.iter().any(|r| r.actor == actor));
        let action = match action_kind {
            0 => TacticalAction::Attack { choice },
            1 => TacticalAction::Dash {
                speed: DashSpeed::Speed,
            },
            2 => TacticalAction::Disengage,
            3 => TacticalAction::Dodge,
            4 => TacticalAction::StartAttackAction,
            5 => TacticalAction::EndTurn,
            _ => unreachable!(),
        };
        f.run(Some(0), action);
        let retained = action_kind == 5;
        assert_eq!(f.rules().rests.iter().any(|r| r.actor == actor), retained);
        assert_eq!(
            f.rules().tactical_recovery.as_ref().unwrap()[&actor]
                .knockout_rest
                .is_some(),
            retained
        );
    }
}

#[test]
fn after_attack_unequip_and_thrown_custody_wait_for_resolution() {
    for thrown in [false, true] {
        let mut f = Fixture::new();
        let mut choice = f.arm("dagger", false, false);
        if thrown {
            choice.delivery = WeaponDelivery::Thrown;
        } else {
            choice.equipment_change = Some(AttackEquipmentChange {
                timing: EquipmentChangeTiming::AfterAttack,
                operation: AttackEquipmentOperation::Unequip {
                    item: choice.weapon,
                },
            });
        }
        f.begin();
        f.run(
            Some(0),
            TacticalAction::Attack {
                choice: choice.clone(),
            },
        );
        assert!(
            f.loadout()
                .hands
                .hands
                .contains(&HandAssignment::Item(choice.weapon))
        );
        f.roll(0, if thrown { &[1, 1] } else { &[1] });
        assert!(
            !f.loadout()
                .hands
                .hands
                .contains(&HandAssignment::Item(choice.weapon))
        );
        if thrown {
            assert!(matches!(
                f.state.items[&choice.weapon].custody,
                Custody::Location(_)
            ));
            assert_eq!(f.flow().ground_items[0].item, choice.weapon);
        } else {
            assert_eq!(
                f.state.items[&choice.weapon].custody,
                Custody::Entity(f.actors[0])
            );
        }
    }
}

#[test]
fn graze_is_a_real_optional_owned_damage_decision_without_another_attack_roll() {
    for accept in [false, true] {
        let mut f = Fixture::new();
        let choice = f.arm("greatsword", true, false);
        f.begin();
        f.run(Some(0), TacticalAction::Attack { choice });
        f.roll(0, &[1]);
        assert_eq!(f.rules().entities[&f.actors[1]].hp, 50);
        assert!(f.rules().pending.is_none());
        f.rejected(
            Some(1),
            TacticalAction::ChooseAttackMastery {
                choice: WeaponMasteryChoice::Graze,
            },
        );
        f.run(
            Some(0),
            TacticalAction::ChooseAttackMastery {
                choice: if accept {
                    WeaponMasteryChoice::Graze
                } else {
                    WeaponMasteryChoice::Decline
                },
            },
        );
        assert_eq!(
            f.rules().entities[&f.actors[1]].hp,
            if accept { 47 } else { 50 }
        );
        assert_eq!(f.rules().rolls.len(), 3);
        assert!(f.flow().resolution.is_none());
    }
}

#[test]
fn restored_pending_attack_rejects_forged_modifiers_equipment_outcomes_and_damage_dice() {
    let mut f = Fixture::new();
    let choice = f.arm("shortbow", false, true);
    f.begin();
    f.run(Some(0), TacticalAction::Attack { choice });
    for mutation in 0..9 {
        let mut bad = f.state.clone();
        let attack = bad
            .encounter
            .as_mut()
            .unwrap()
            .flow
            .as_mut()
            .unwrap()
            .resolution
            .as_mut()
            .unwrap()
            .attack
            .as_mut()
            .unwrap();
        match mutation {
            0 => attack.attack_modifier += 10,
            1 => attack.damage[0].modifier += 20,
            2 => attack.mode = RollMode::Advantage,
            3 => attack.armor_class -= 10,
            4 => {
                attack
                    .weapon_mut()
                    .unwrap()
                    .ammunition
                    .as_mut()
                    .unwrap()
                    .quantity_before += 1
            }
            5 => attack.weapon_mut().unwrap().equipment_before.hands = WeaponLoadout::default(),
            6 => {
                attack.outcome = Some(WeaponAttackOutcome::Hit {
                    critical: true,
                    damage_dealt: 999,
                })
            }
            7 => attack.target = f.actors[0],
            8 => attack.delivery = TacticalAttackDelivery::Melee,
            _ => unreachable!(),
        }
        assert!(
            validate_tactical_state(&bad).is_err(),
            "mutation {mutation}"
        );
    }
    f.roll(0, &[15]);
    let mut bad = f.state.clone();
    bad.rules
        .as_mut()
        .unwrap()
        .pending
        .as_mut()
        .unwrap()
        .request
        .dice[0]
        .count = 99;
    assert!(validate_tactical_state(&bad).is_err());
}

#[test]
fn weapon_damage_enters_existing_concentration_work_with_correct_roller_and_raw_history() {
    let mut f = Fixture::new();
    let choice = f.arm("longsword", false, false);
    f.entity_mut(0).heroic_inspiration = true;
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
    f.begin();
    f.run(Some(0), TacticalAction::Attack { choice });
    f.roll(0, &[15]);
    let original = f.raw(&[1]);
    f.run(
        Some(0),
        TacticalAction::SubmitRollWithInspiration {
            result: original.clone(),
            die_index: 0,
            replacement: DieResult { sides: 8, value: 8 },
        },
    );
    assert_eq!(f.rules().entities[&f.actors[1]].hp, 39);
    assert!(!f.rules().entities[&f.actors[0]].heroic_inspiration);
    assert_eq!(
        f.rules().rolls.last().unwrap().original_result,
        Some(original)
    );
    assert!(f.flow().resolution.as_ref().unwrap().attack.is_none());
    assert_eq!(f.request().roller, Some(f.actors[1]));
    assert_eq!(
        f.request().dice,
        vec![DieSpec {
            count: 1,
            sides: 20
        }]
    );
    let pending = f
        .flow()
        .resolution
        .as_ref()
        .unwrap()
        .pending
        .as_ref()
        .unwrap();
    assert!(
        matches!(pending.work.kind,TacticalWorkKind::ConcentrationSave{group:g,damage_taken:11,..} if g==group)
    );
    f.rejected(
        Some(0),
        TacticalAction::SubmitRoll {
            result: f.raw(&[1]),
        },
    );
    f.rejected(Some(0), TacticalAction::EndTurn);
    f.roll(1, &[1]);
    assert_eq!(f.rules().entities[&f.actors[1]].concentration, None);
    assert!(f.flow().resolution.is_none());
    assert_eq!(f.rules().timing.as_ref().unwrap().turn_number, 1);
}

#[test]
fn nick_and_light_share_one_extra_attack_with_source_damage_modifier_and_action_budget() {
    for nick in [false, true] {
        let mut f = Fixture::new();
        let first = f.arm("dagger", nick, false);
        let second = f.item("dagger", 1);
        f.state
            .rules
            .as_mut()
            .unwrap()
            .tactical_inventory
            .as_mut()
            .unwrap()
            .loadouts[0]
            .hands
            .hands[1] = HandAssignment::Item(second);
        f.begin();
        let trigger = f.run(
            Some(0),
            TacticalAction::Attack {
                choice: first.clone(),
            },
        );
        f.roll(0, &[1]);
        let mut extra = WeaponUseChoice {
            weapon: second,
            grip: WeaponGrip::OneHand(Hand::Right),
            purpose: if nick {
                WeaponAttackPurpose::Nick {
                    trigger: trigger.meta.id,
                }
            } else {
                WeaponAttackPurpose::LightBonus {
                    trigger: trigger.meta.id,
                }
            },
            ..first
        };
        f.run(
            Some(0),
            TacticalAction::Attack {
                choice: extra.clone(),
            },
        );
        f.roll(0, &[15]);
        f.roll(0, &[4]);
        assert_eq!(f.rules().entities[&f.actors[1]].hp, 46); // Positive Strength is not added to this extra attack.
        assert_eq!(f.rules().timing.as_ref().unwrap().bonus_action_spent, !nick);
        extra.purpose = WeaponAttackPurpose::LightBonus {
            trigger: trigger.meta.id,
        };
        f.rejected(Some(0), TacticalAction::Attack { choice: extra });
        assert_eq!(f.flow().budget.weapon_history.len(), 2);
    }
}

#[test]
fn nearby_blind_defender_does_not_impose_ranged_penalty_and_prone_cancels_advantage() {
    for prone in [false, true] {
        let mut f = Fixture::new();
        let choice = f.arm("shortbow", false, true);
        f.state.encounter.as_mut().unwrap().participants[1]
            .position
            .x = 20;
        f.entity_mut(0).prone = prone;
        f.state.rules.as_mut().unwrap().effects.push(ActiveEffect {
            id: EffectId::new(),
            source: f.actors[0],
            target: f.actors[1],
            condition: Some(Condition::Blinded),
            label: "Source blindness".into(),
            expires: Expiry::Never,
            concentration_owner: None,
        });
        f.begin();
        f.run(Some(0), TacticalAction::Attack { choice });
        assert_eq!(
            f.request().mode,
            if prone {
                RollMode::Normal
            } else {
                RollMode::Advantage
            }
        );
        f.roll(0, if prone { &[1] } else { &[1, 1] });
    }
}

#[test]
fn underwater_long_range_automatically_misses_and_still_expends_one_arrow() {
    let mut f = Fixture::new();
    let choice = f.arm("shortbow", false, true);
    let ammo = choice.ammunition.unwrap();
    let encounter = f.state.encounter.as_mut().unwrap();
    encounter.battlefield.bounds.max.x = 500;
    encounter.participants[1].position.x = 200;
    encounter.battlefield.terrain.push(TerrainVolume {
        id: "pool".into(),
        volume: SpatialBox {
            min: SpatialPoint { x: 0, y: 0, z: 0 },
            max: SpatialPoint {
                x: 500,
                y: 100,
                z: 30,
            },
        },
        difficult: false,
        observable: true,
        water: true,
        climbable: false,
        burrowable: false,
        supports_top: false,
        surface: Some("pool-water".into()),
        obscuration: Obscuration::None,
        magical_darkness: false,
    });
    f.begin();
    let dice_before = f.rules().rolls.len();
    f.run(Some(0), TacticalAction::Attack { choice });
    assert_eq!(f.rules().rolls.len(), dice_before);
    assert!(f.rules().pending.is_none());
    assert_eq!(f.rules().entities[&f.actors[1]].hp, 50);
    assert_eq!(f.state.items[&ammo].quantity, 19);
    assert_eq!(
        f.flow().budget.weapon_history[0].outcome,
        WeaponAttackOutcome::Miss
    );
}

#[test]
fn an_attack_can_equip_a_carried_weapon_before_its_roll_without_changing_ownership() {
    let mut f = Fixture::new();
    let mut choice = f.arm("longsword", false, false);
    f.state
        .rules
        .as_mut()
        .unwrap()
        .tactical_inventory
        .as_mut()
        .unwrap()
        .loadouts[0]
        .hands = WeaponLoadout::default();
    choice.equipment_change = Some(AttackEquipmentChange {
        timing: EquipmentChangeTiming::BeforeAttack,
        operation: AttackEquipmentOperation::Equip {
            item: choice.weapon,
            hand: Hand::Left,
        },
    });
    f.begin();
    f.run(
        Some(0),
        TacticalAction::Attack {
            choice: choice.clone(),
        },
    );
    assert_eq!(
        f.loadout().hands.hands[0],
        HandAssignment::Item(choice.weapon)
    );
    f.roll(0, &[1]);
    assert_eq!(
        f.loadout().hands.hands[0],
        HandAssignment::Item(choice.weapon)
    );
    assert_eq!(
        f.state.items[&choice.weapon].owner,
        Ownership::Entity(f.actors[1])
    );
}

#[test]
fn forged_graze_pause_cannot_turn_an_accepted_hit_into_a_miss() {
    let mut f = Fixture::new();
    let choice = f.arm("greatsword", true, false);
    f.begin();
    f.run(Some(0), TacticalAction::Attack { choice });
    f.roll(0, &[15]);
    let mut forged = f.state.clone();
    forged.rules.as_mut().unwrap().pending = None;
    let resolution = forged
        .encounter
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .resolution
        .as_mut()
        .unwrap();
    resolution.pending = None;
    resolution.frames.clear();
    let attack = resolution.attack.as_mut().unwrap();
    attack.stage = TacticalAttackStage::MasteryChoice;
    attack.outcome = Some(WeaponAttackOutcome::Miss);
    attack.armor_class = 999;
    assert!(validate_tactical_state(&forged).is_err());
    let mut g = Fixture::new();
    let choice = g.arm("greatsword", true, false);
    g.begin();
    g.run(Some(0), TacticalAction::Attack { choice });
    g.roll(0, &[1]);
    for field in 0..3 {
        let mut forged = g.state.clone();
        let attack = forged
            .encounter
            .as_mut()
            .unwrap()
            .flow
            .as_mut()
            .unwrap()
            .resolution
            .as_mut()
            .unwrap()
            .attack
            .as_mut()
            .unwrap();
        match field {
            0 => attack.mode = RollMode::Advantage,
            1 => attack.armor_class += 1,
            2 => attack.critical_on_hit = true,
            _ => unreachable!(),
        };
        assert!(validate_tactical_state(&forged).is_err(), "field {field}");
    }
}

#[test]
fn fixed_weapon_damage_completes_without_fabricated_damage_faces() {
    let mut f = Fixture::new();
    let choice = f.arm("blowgun", false, true);
    f.begin();
    f.run(Some(0), TacticalAction::Attack { choice });
    f.roll(0, &[20]);
    assert_eq!(f.rules().entities[&f.actors[1]].hp, 49); // Fixed 1 has no damage roll to add Dexterity to, or dice to double.
    assert!(f.rules().pending.is_none());
    assert!(f.flow().resolution.is_none());
    assert_eq!(f.rules().rolls.len(), 3);
    assert_eq!(
        f.flow().budget.weapon_history[0].outcome,
        WeaponAttackOutcome::Hit {
            critical: true,
            damage_dealt: 1
        }
    );
}

#[test]
fn hidden_truth_id_does_not_authorize_an_unlocated_target_or_spend_ammunition() {
    let mut f = Fixture::new();
    let choice = f.arm("shortbow", false, true);
    f.state
        .encounter
        .as_mut()
        .unwrap()
        .battlefield
        .ambient_light = LightLevel::Darkness;
    f.begin();
    f.rejected(
        Some(0),
        TacticalAction::Attack {
            choice: choice.clone(),
        },
    );
    assert_eq!(f.state.items[&choice.ammunition.unwrap()].quantity, 20);
    assert!(!f.rules().timing.as_ref().unwrap().action_spent);
}

#[test]
fn hidden_target_range_cannot_be_probed_through_attack_rejection() {
    let mut f = Fixture::new();
    let choice = f.arm("shortbow", false, true);
    let encounter = f.state.encounter.as_mut().unwrap();
    encounter.battlefield.ambient_light = LightLevel::Darkness;
    encounter.battlefield.bounds.max.x = 2000;
    f.begin();
    let mut errors = vec![];
    for (position, dead) in [(60, false), (1000, false), (60, true), (1000, true)] {
        if dead {
            f.zero_hp_target(true);
        }
        f.state.encounter.as_mut().unwrap().participants[1]
            .position
            .x = position;
        let before = f.state.clone();
        errors.push(
            resolve_tactical(
                &f.state,
                &f.meta(Some(0)),
                &TacticalAction::Attack {
                    choice: choice.clone(),
                },
                &f.pack,
            )
            .unwrap_err()
            .to_string(),
        );
        assert_eq!(f.state, before);
    }
    let mut absent = choice.clone();
    absent.target = EntityId::new();
    let absent_error = resolve_tactical(
        &f.state,
        &f.meta(Some(0)),
        &TacticalAction::Attack { choice: absent },
        &f.pack,
    )
    .unwrap_err()
    .to_string();
    assert!(errors.iter().all(|error| *error == errors[0]));
    assert_eq!(errors[0], absent_error);
    assert!(errors[0].contains("currently located target"));
    assert_eq!(f.state.items[&choice.ammunition.unwrap()].quantity, 20);
    assert!(!f.rules().timing.as_ref().unwrap().action_spent);
}

#[test]
fn nearby_paralyzed_target_makes_hits_critical_without_making_every_attack_hit() {
    for face in [3, 5] {
        let mut f = Fixture::new();
        let choice = f.arm("longsword", false, false);
        f.state.rules.as_mut().unwrap().effects.push(ActiveEffect {
            id: EffectId::new(),
            source: f.actors[0],
            target: f.actors[1],
            condition: Some(Condition::Paralyzed),
            label: "Source paralysis".into(),
            expires: Expiry::Never,
            concentration_owner: None,
        });
        f.begin();
        f.run(Some(0), TacticalAction::Attack { choice });
        assert_eq!(f.request().mode, RollMode::Advantage);
        f.roll(0, &[face, face]);
        if face == 3 {
            assert!(f.rules().pending.is_none());
            assert_eq!(f.rules().entities[&f.actors[1]].hp, 50);
        } else {
            assert_eq!(f.request().dice, vec![DieSpec { count: 2, sides: 8 }]);
            f.roll(0, &[8, 8]);
            assert_eq!(f.rules().entities[&f.actors[1]].hp, 31);
        }
    }
}

#[test]
fn restored_automatic_miss_cannot_be_replaced_by_a_pending_natural_twenty() {
    let mut f = Fixture::new();
    let choice = f.arm("shortbow", false, true);
    f.begin();
    f.run(Some(0), TacticalAction::Attack { choice });
    let mut forged = f.state.clone();
    let encounter = forged.encounter.as_mut().unwrap();
    encounter.battlefield.bounds.max.x = 500;
    encounter.participants[1].position.x = 200;
    encounter.battlefield.terrain.push(TerrainVolume {
        id: "pool".into(),
        volume: SpatialBox {
            min: SpatialPoint { x: 0, y: 0, z: 0 },
            max: SpatialPoint {
                x: 500,
                y: 100,
                z: 30,
            },
        },
        difficult: false,
        observable: true,
        water: true,
        climbable: false,
        burrowable: false,
        supports_top: false,
        surface: Some("pool-water".into()),
        obscuration: Obscuration::None,
        magical_darkness: false,
    });
    let attack = encounter
        .flow
        .as_mut()
        .unwrap()
        .resolution
        .as_mut()
        .unwrap()
        .attack
        .as_mut()
        .unwrap();
    attack.automatic_miss = true;
    attack.mode = RollMode::Disadvantage;
    forged
        .rules
        .as_mut()
        .unwrap()
        .pending
        .as_mut()
        .unwrap()
        .request
        .mode = RollMode::Disadvantage;
    assert!(validate_tactical_state(&forged).is_err());
    let pending = forged.rules.as_ref().unwrap().pending.as_ref().unwrap();
    let raw = RollResult {
        request_id: pending.request.id,
        source: RollSource::Physical,
        dice: vec![
            DieResult {
                sides: 20,
                value: 20
            };
            2
        ],
    };
    assert!(
        resolve_tactical(
            &forged,
            &f.meta(Some(0)),
            &TacticalAction::SubmitRoll { result: raw },
            &f.pack
        )
        .is_err()
    );
}
