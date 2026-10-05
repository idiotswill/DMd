//! Constructed private mechanism inputs followed by the actual activation,
//! attack, raw and equipment producers. These are NOT accepted app histories,
//! original export replay, portable restore, native play or Ogre admission.
use super::*;
use crate::tactical_weapons::ground::RetainedPhysicalRead;

struct Fixture {
    state: CampaignState,
    pack: RulesPack,
    actor: EntityId,
    target: EntityId,
    choice: WeaponUseChoice,
}

impl Fixture {
    fn new() -> Self {
        let export: serde_json::Value = serde_json::from_str(include_str!(
            "../../../dmd-app/tests/fixtures/shield-hit-v1-selected.json"
        ))
        .unwrap();
        let mut state: CampaignState =
            serde_json::from_str(export["current_state"]["state_json"].as_str().unwrap()).unwrap();
        let attack = resolution(&state).unwrap().attack.as_ref().unwrap().clone();
        let weapon = attack.weapon().unwrap();
        let mut choice = weapon.choice.clone();
        choice.equipment_change = None;
        choice.after_equipment = None;
        // This is a declared private starting image, not a rewriting of the
        // frozen export or a claim that its earlier events produced flow5.
        let rules = state.rules.as_mut().unwrap();
        rules.pending = None;
        rules
            .rolls
            .retain(|r| !matches!(r.purpose, PendingPurpose::TacticalResolution { .. }));
        rules.timing.as_mut().unwrap().action_spent = false;
        let equipped = rules
            .tactical_inventory
            .as_mut()
            .unwrap()
            .loadouts
            .iter_mut()
            .find(|l| l.actor == attack.actor)
            .unwrap();
        *equipped = weapon.equipment_before.clone();
        equipped.hands.hands[Hand::Right.index()] = HandAssignment::Item(choice.weapon);
        let flow = flow_mut(&mut state).unwrap();
        flow.version = TacticalExecutionVersion::EncounterReleaseV1.flow_version();
        flow.resolution = None;
        flow.budget = TacticalTurnBudget::default();
        flow.ground_items.clear();
        Self {
            state,
            pack: RulesPack::from_json(include_str!("../../../../content/srd-5.2.1/kernel.json"))
                .unwrap(),
            actor: attack.actor,
            target: attack.target,
            choice,
        }
    }

    fn meta(&self, actor: EntityId) -> CommandMeta {
        CommandMeta {
            id: CommandId::new(),
            campaign_id: self.state.campaign_id(),
            session_id: self
                .state
                .table
                .as_ref()
                .unwrap()
                .active_session
                .as_ref()
                .map(|s| s.session_id),
            issuer: controller(&self.state, actor)
                .map_or(CommandIssuer::Admin, CommandIssuer::Player),
            actor: Some(AgentRef::Entity(actor)),
            expected_event_sequence: self.state.applied_event_sequence,
        }
    }
    fn host(&self) -> CommandMeta {
        let mut meta = self.meta(self.actor);
        meta.actor = None;
        meta.issuer = CommandIssuer::Admin;
        meta
    }
    fn activate(&mut self) -> CommandMeta {
        let meta = self.host();
        activate(&mut self.state, &meta, &self.pack).unwrap();
        self.state.applied_event_sequence += 1;
        meta
    }
    fn put_down(&mut self, item: ItemId) {
        let e = self.state.encounter.as_ref().unwrap();
        let actor = e.participant(self.actor).unwrap();
        let point = SpatialPoint {
            x: actor.position.x + 5,
            y: actor.position.y + 5,
            z: actor.position.z,
        };
        let location = self.state.scenes[&e.scene_id].location_id;
        let origin = self
            .state
            .rules
            .as_ref()
            .unwrap()
            .tactical_inventory
            .as_ref()
            .unwrap()
            .loadout(self.actor)
            .unwrap()
            .command
            .clone();
        self.state.items.get_mut(&item).unwrap().custody = Custody::Location(location);
        let loadout = self
            .state
            .rules
            .as_mut()
            .unwrap()
            .tactical_inventory
            .as_mut()
            .unwrap()
            .loadouts
            .iter_mut()
            .find(|l| l.actor == self.actor)
            .unwrap();
        for hand in &mut loadout.hands.hands {
            if *hand == HandAssignment::Item(item) {
                *hand = HandAssignment::Free;
            }
        }
        flow_mut(&mut self.state)
            .unwrap()
            .ground_items
            .push(TacticalGroundItem {
                item,
                position: point,
                origin,
            });
    }
    fn pickup(&mut self, item: ItemId, hand: Hand) {
        self.put_down(item);
        self.choice.equipment_change = Some(AttackEquipmentChange {
            timing: EquipmentChangeTiming::BeforeAttack,
            operation: AttackEquipmentOperation::Pickup { item, hand },
        });
    }
    fn begin(&mut self) -> CommandMeta {
        let meta = self.meta(self.actor);
        // The public command's existing atomic boundary owns its candidate. The
        // private producer used here is exactly that branch, not a new reducer.
        let mut candidate = self.state.clone();
        attacks::begin(&mut candidate, &meta, &self.choice, &self.pack).unwrap();
        candidate.applied_event_sequence += 1;
        self.state = candidate;
        meta
    }
    fn raw(&mut self, face: u16) {
        let request = &self
            .state
            .rules
            .as_ref()
            .unwrap()
            .pending
            .as_ref()
            .unwrap()
            .request;
        let count = if request.mode == RollMode::Normal {
            1
        } else {
            2
        };
        let raw = RollResult {
            request_id: request.id,
            source: RollSource::Physical,
            dice: request
                .dice
                .iter()
                .flat_map(|d| {
                    (0..d.count * count).map(move |_| DieResult {
                        sides: d.sides,
                        value: face.min(d.sides),
                    })
                })
                .collect(),
        };
        let meta = self.meta(request.roller.unwrap());
        continuations::submit(&mut self.state, &meta, &raw, None).unwrap();
        self.state.applied_event_sequence += 1;
    }
    fn validate_paid(&self) {
        validate_records(&self.state).unwrap();
        attacks::validate(&self.state).unwrap();
        super::super::turn_validation::validate(&self.state).unwrap();
    }

    fn physical_input<'a>(
        &'a self,
        meta: &'a CommandMeta,
    ) -> crate::tactical_weapons::WeaponAttackInput<'a> {
        use crate::tactical_weapons::{WeaponActorSource, WeaponAttackContext, WeaponAttackInput};
        let rules = self.state.rules.as_ref().unwrap();
        let e = self.state.encounter.as_ref().unwrap();
        let actor = e.participant(self.actor).unwrap();
        let target = e.participant(self.target).unwrap();
        WeaponAttackInput {
            state: &self.state,
            source: WeaponActorSource::Character(
                self.state
                    .table
                    .as_ref()
                    .unwrap()
                    .character_profiles
                    .values()
                    .find(|p| p.entity_id == self.actor)
                    .unwrap(),
            ),
            pack: &self.pack,
            definitions: definitions().unwrap(),
            choice: &self.choice,
            context: WeaponAttackContext {
                origin: meta,
                actor: self.actor,
                turn_number: rules.timing.as_ref().unwrap().turn_number,
                on_actor_turn: active(&self.state).unwrap() == self.actor,
                window: WeaponActionWindow {
                    id: meta.id,
                    kind: WeaponActionKind::AttackAction,
                },
                distance: crate::spatial::participant_distance(actor, target).unwrap(),
                base_reach: actor.reach,
                mounted: false,
                underwater: false,
                has_swim_speed: actor.movement.swim.is_some(),
                target_is_creature: true,
                target_size: target.size,
                distance_from_trigger_target: None,
            },
            loadout: &rules
                .tactical_inventory
                .as_ref()
                .unwrap()
                .loadout(self.actor)
                .unwrap()
                .hands,
            history: &flow(&self.state).unwrap().budget.weapon_history,
        }
    }

    fn decline_hit(&mut self) {
        let resolution = resolution(&self.state).unwrap();
        let window = TacticalWorkKey {
            resolution: resolution.origin.id,
            occurrence: resolution.hit_review.as_ref().unwrap().work.occurrence,
        };
        let meta = self.meta(self.actor);
        hit_reactions::order(
            &mut self.state,
            &meta,
            window,
            &TacticalReactionOrdering {
                ranked: vec![],
                unlisted: ReactionUnlistedOrder::AfterForward,
            },
        )
        .unwrap();
        self.state.applied_event_sequence += 1;
        let meta = self.meta(self.target);
        hit_reactions::respond(&mut self.state, &meta, window, false).unwrap();
        self.state.applied_event_sequence += 1;
    }

    fn source(shot: bool) -> Self {
        use crate::tactical_creature_equipment::{
            creature_current_armor, creature_equipment_plan, materialize_creature_equipment,
        };
        use crate::tactical_creatures::{
            CreatureBuildChoice, CreatureHitPointChoice, build_creature,
        };
        let mut f = Self::new();
        let target_source = flow(&f.state)
            .unwrap()
            .combatants
            .iter()
            .find(|c| c.actor == f.target)
            .unwrap()
            .source
            .clone();
        let old = f.actor;
        let actor = EntityId::new();
        let mut entity = f.state.entities[&old].clone();
        entity.id = actor;
        entity.kind = EntityKind::Creature;
        f.state.entities.insert(actor, entity);
        let origin = f.host();
        let built = build_creature(
            &f.state,
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
        )
        .unwrap();
        let rules = f.state.rules.as_mut().unwrap();
        rules.entities.insert(actor, built.mechanics);
        let creatures = rules.tactical_creatures.get_or_insert_default();
        creatures.profiles.push(built.profile);
        creatures.runtime.push(built.runtime);
        rules.timing = None;
        rules.rolls.clear();
        rules.tactical_effects.as_mut().unwrap().turn = None;
        let ids = creature_equipment_plan("goblin-warrior", 3)
            .unwrap()
            .iter()
            .map(|_| ItemId::new())
            .collect::<Vec<_>>();
        f.state =
            materialize_creature_equipment(&f.state, &origin, actor, 3, &ids, &f.pack).unwrap();
        let equipment = f
            .state
            .rules
            .as_mut()
            .unwrap()
            .tactical_inventory
            .as_mut()
            .unwrap()
            .loadouts
            .iter_mut()
            .find(|l| l.actor == actor)
            .unwrap();
        if shot {
            // Declared ordinary baseline with the shield already doffed, not an
            // extra unpaid Doff Shield action granted by this attack producer.
            equipment.shield = None;
            equipment.hands = WeaponLoadout::default();
            equipment.command = origin.clone();
            let profile = f
                .state
                .rules
                .as_ref()
                .unwrap()
                .tactical_creatures
                .as_ref()
                .unwrap()
                .profile(actor)
                .unwrap();
            let armor = creature_current_armor(&f.state, profile).unwrap();
            f.state
                .rules
                .as_mut()
                .unwrap()
                .entities
                .get_mut(&actor)
                .unwrap()
                .armor = armor;
        }
        let e = f.state.encounter.as_mut().unwrap();
        e.flow = None;
        let p = e
            .participants
            .iter_mut()
            .find(|p| p.entity_id == old)
            .unwrap();
        p.entity_id = actor;
        p.size = CreatureSize::Small;
        p.height = 10;
        p.movement = built.movement;
        p.senses = built.senses;
        let target = e
            .participants
            .iter_mut()
            .find(|p| p.entity_id == f.target)
            .unwrap();
        target.enemies = vec![actor];
        if shot {
            target.position.x = 60;
            e.battlefield.bounds.max.x = e.battlefield.bounds.max.x.max(100);
        }
        let scene = e.scene_id;
        f.state
            .scenes
            .get_mut(&scene)
            .unwrap()
            .presences
            .iter_mut()
            .find(|p| p.entity_id == old)
            .unwrap()
            .entity_id = actor;
        f.actor = actor;
        f.choice.weapon = f
            .state
            .items
            .values()
            .find(|i| {
                i.custody == Custody::Entity(actor)
                    && i.definition_id == if shot { "shortbow" } else { "scimitar" }
            })
            .unwrap()
            .id;
        f.choice.delivery = if shot {
            WeaponDelivery::Shot
        } else {
            WeaponDelivery::Melee
        };
        f.choice.grip = if shot {
            WeaponGrip::TwoHands
        } else {
            WeaponGrip::OneHand(Hand::Right)
        };
        f.choice.ammunition = shot.then(|| {
            f.state
                .items
                .values()
                .find(|i| i.custody == Custody::Entity(actor) && i.definition_id == "arrows")
                .unwrap()
                .id
        });
        f.choice.ability = Ability::Dexterity;
        f.state.applied_event_sequence += 1;
        let begin = TacticalAction::Begin {
            execution: TacticalExecutionVersion::EncounterReleaseV1,
            combatants: vec![
                TacticalCombatant {
                    actor,
                    source: TacticalSource::Creature {
                        definition_id: "goblin-warrior".into(),
                    },
                    surprised: false,
                },
                TacticalCombatant {
                    actor: f.target,
                    source: target_source,
                    surprised: false,
                },
            ],
            groups: [actor, f.target]
                .into_iter()
                .map(|actor| InitiativeGroup {
                    actors: vec![actor],
                    request_id: RollRequestId::new(),
                })
                .collect(),
        };
        f.state = resolve_tactical(&f.state, &f.host(), &begin, &f.pack)
            .unwrap()
            .next_state;
        f.state.applied_event_sequence += 1;
        for face in [18, 3] {
            let request = f
                .state
                .rules
                .as_ref()
                .unwrap()
                .pending
                .as_ref()
                .unwrap()
                .request
                .clone();
            let result = RollResult {
                request_id: request.id,
                source: RollSource::Physical,
                dice: vec![DieResult {
                    sides: 20,
                    value: face,
                }],
            };
            f.state = resolve_tactical(
                &f.state,
                &f.meta(request.roller.unwrap()),
                &TacticalAction::SubmitRoll { result },
                &f.pack,
            )
            .unwrap()
            .next_state;
            f.state.applied_event_sequence += 1;
        }
        assert_eq!(active(&f.state).unwrap(), actor);
        f
    }

    fn begin_source(&mut self) -> CommandMeta {
        let meta = self.meta(self.actor);
        let selected = CreatureWeaponUseChoice {
            after_equipment: self.choice.after_equipment,
            weapon: self.choice.weapon,
            target: self.target,
            grip: self.choice.grip,
            ammunition: self.choice.ammunition,
            equipment_change: self.choice.equipment_change,
        };
        let mut candidate = self.state.clone();
        let feature = if self.choice.delivery == WeaponDelivery::Shot {
            "shortbow"
        } else {
            "scimitar"
        };
        attacks::begin_creature_weapon(&mut candidate, &meta, feature, &selected, &self.pack)
            .unwrap();
        candidate.applied_event_sequence += 1;
        self.state = candidate;
        meta
    }

    fn borrowed(&mut self, definition: &str) {
        let item = ItemId::new();
        let mut physical = self.state.items[&self.choice.weapon].clone();
        physical.id = item;
        physical.definition_id = definition.into();
        physical.owner = Ownership::Entity(self.target);
        self.state.items.insert(item, physical);
        self.choice.weapon = item;
        self.state
            .rules
            .as_mut()
            .unwrap()
            .tactical_inventory
            .as_mut()
            .unwrap()
            .loadouts
            .iter_mut()
            .find(|l| l.actor == self.actor)
            .unwrap()
            .hands = WeaponLoadout::default();
    }

    fn grant_graze(&mut self) {
        let input = crate::CharacterCreationInput {
            name: "Equipment reader".into(),
            pronouns: "they/them".into(),
            description: "A traveler".into(),
            alignment: "Neutral Good".into(),
            backstory: "A guard".into(),
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
            masteries: ["greatsword".into(), "dagger".into(), "shortbow".into()],
        };
        let built = crate::build_character(&input, self.actor, &self.pack).unwrap();
        let character = self
            .state
            .characters
            .values()
            .find(|c| c.entity_id == self.actor)
            .unwrap()
            .id;
        self.state
            .table
            .as_mut()
            .unwrap()
            .character_profiles
            .insert(character, built.profile);
        let origin = self.meta(self.actor);
        let rules = self.state.rules.as_mut().unwrap();
        rules.entities.insert(self.actor, built.mechanics);
        let inventory = rules.tactical_inventory.as_mut().unwrap();
        inventory.receipts.retain(|r| r.actor != self.actor);
        inventory.loadouts.retain(|l| l.actor != self.actor);
        let provisioned = crate::tactical_inventory::materialize_starting_equipment(
            &self.state,
            self.state
                .rules
                .as_ref()
                .unwrap()
                .tactical_inventory
                .as_ref()
                .unwrap(),
            &origin,
            character,
            &[],
            &self.pack,
        )
        .unwrap();
        self.state = provisioned.next_state;
        self.state.rules.as_mut().unwrap().tactical_inventory = Some(provisioned.next_inventory);
        self.state.applied_event_sequence += 1;
        self.borrowed("greatsword");
        self.choice.grip = WeaponGrip::TwoHands;
    }
}

#[test]
fn private_actual_activation_changes_only_the_omitted_encounter_record() {
    let mut f = Fixture::new();
    let original = f.state.clone();
    assert!(
        serde_json::to_value(flow(&f.state).unwrap())
            .unwrap()
            .get("attack_equipment_access")
            .is_none()
    );
    let origin = f.activate();
    let mut expected = original;
    flow_mut(&mut expected).unwrap().attack_equipment_access =
        Some(Box::new(TacticalAttackEquipmentAccess {
            version: AttackEquipmentAccessVersion::GroundEquipmentV1,
            origin,
        }));
    expected.applied_event_sequence += 1;
    assert_eq!(f.state, expected);
    validate_records(&f.state).unwrap();
    let saved = serde_json::to_string(&f.state).unwrap();
    assert_eq!(
        serde_json::from_str::<CampaignState>(&saved).unwrap(),
        f.state
    );
    let before = f.state.clone();
    let repeat = f.host();
    assert!(activate(&mut f.state, &repeat, &f.pack).is_err());
    assert_eq!(f.state, before);
}

#[test]
fn private_activation_rejects_nonhost_wrong_session_paid_pending_and_old_executor_atomically() {
    for case in 0..7 {
        let mut f = Fixture::new();
        let mut meta = f.host();
        match case {
            0 => meta = f.meta(f.actor),
            1 => meta.session_id = Some(PlaySessionId::new()),
            2 => {
                f.state
                    .rules
                    .as_mut()
                    .unwrap()
                    .timing
                    .as_mut()
                    .unwrap()
                    .action_spent = true
            }
            3 => {
                flow_mut(&mut f.state).unwrap().version =
                    TacticalExecutionVersion::ShieldHitV1.flow_version()
            }
            4 => meta.expected_event_sequence += 1,
            5 => meta.actor = Some(AgentRef::Entity(f.actor)),
            6 => {
                f.begin();
                meta = f.host();
            }
            _ => unreachable!(),
        }
        let original = f.state.clone();
        assert!(
            activate(&mut f.state, &meta, &f.pack).is_err(),
            "case {case}"
        );
        assert_eq!(f.state, original, "case {case}");
    }
}

#[test]
fn activated_public_command_kernel_planner_and_replay_share_actual_pickup_transition() {
    let mut f = Fixture::new();
    f.pickup(f.choice.weapon, Hand::Right);
    let before = f.state.clone();
    let activation = resolve_tactical(
        &before,
        &f.host(),
        &TacticalAction::ActivateAttackEquipment,
        &f.pack,
    )
    .unwrap();
    assert_eq!(
        replay_tactical(&before, &activation.event, &f.pack)
            .unwrap()
            .next_state,
        activation.next_state
    );
    f.state = activation.next_state;
    f.state.applied_event_sequence += 1;
    crate::validate_state(&f.state, &f.pack).unwrap();
    validate_tactical_state(&f.state).unwrap();
    crate::tactical_weapons::prepare_weapon_attack(&f.physical_input(&f.meta(f.actor))).unwrap();
    let action = TacticalAction::Attack {
        choice: f.choice.clone(),
    };
    let meta = f.meta(f.actor);
    let accepted = resolve_tactical(&f.state, &meta, &action, &f.pack).unwrap();
    let historical = resolve_with_policy(
        &f.state,
        &meta,
        &action,
        &f.pack,
        ExecutionPolicy::Historical,
    )
    .unwrap();
    assert_eq!(accepted.next_state, historical.next_state);
    assert_eq!(
        replay_tactical(&f.state, &accepted.event, &f.pack)
            .unwrap()
            .next_state,
        accepted.next_state
    );
    assert_eq!(
        accepted.next_state.items[&f.choice.weapon].custody,
        Custody::Entity(f.actor)
    );
    assert!(
        accepted
            .next_state
            .rules
            .as_ref()
            .unwrap()
            .timing
            .as_ref()
            .unwrap()
            .action_spent
    );
    crate::validate_state(&accepted.next_state, &f.pack).unwrap();
    validate_tactical_state(&accepted.next_state).unwrap();
}

#[test]
fn activated_public_admission_refuses_missing_malformed_and_foreign_marker_atomically() {
    for case in 0..4 {
        let mut f = Fixture::new();
        f.pickup(f.choice.weapon, Hand::Right);
        f.activate();
        let marker = &mut flow_mut(&mut f.state).unwrap().attack_equipment_access;
        match case {
            0 => *marker = None,
            1 => marker.as_mut().unwrap().origin.campaign_id = CampaignId::new(),
            2 => marker.as_mut().unwrap().origin.actor = Some(AgentRef::Entity(f.actor)),
            3 => marker.as_mut().unwrap().origin.expected_event_sequence = 0,
            _ => unreachable!(),
        }
        let original = f.state.clone();
        for policy in [ExecutionPolicy::Live, ExecutionPolicy::Historical] {
            assert!(
                resolve_with_policy(
                    &f.state,
                    &f.meta(f.actor),
                    &TacticalAction::Attack {
                        choice: f.choice.clone(),
                    },
                    &f.pack,
                    policy
                )
                .is_err(),
                "case {case}"
            );
        }
        assert!(
            crate::tactical_weapons::prepare_weapon_attack(&f.physical_input(&f.meta(f.actor)))
                .is_err()
        );
        assert_eq!(f.state, original);
    }
}

#[test]
fn private_fresh_preparation_cannot_be_consumed_at_a_cloned_state_or_changed_command() {
    use crate::tactical_weapons::ground::PreparedPhysicalAttack;
    let mut f = Fixture::new();
    f.pickup(f.choice.weapon, Hand::Right);
    f.activate();
    let meta = f.meta(f.actor);
    let input = f.physical_input(&meta);
    let original = f.state.clone();
    let prepared = PreparedPhysicalAttack::new(&f.state, &input).unwrap();
    assert!(
        prepared
            .consume(&original, &meta, f.actor, input.context.window, &f.choice)
            .is_err()
    );
    let prepared = PreparedPhysicalAttack::new(&f.state, &input).unwrap();
    let mut changed = meta.clone();
    changed.id = CommandId::new();
    assert!(
        prepared
            .consume(&f.state, &changed, f.actor, input.context.window, &f.choice)
            .is_err()
    );
    assert_eq!(f.state, original);
}

#[test]
fn private_selected_pickup_commits_same_item_and_paid_reader_preserves_entire_state() {
    let mut f = Fixture::new();
    f.pickup(f.choice.weapon, Hand::Right);
    f.activate();
    let original_item = f.state.items[&f.choice.weapon].clone();
    let before = f.state.clone();
    let origin = f.begin();
    f.validate_paid();
    let attack = resolution(&f.state).unwrap().attack.as_ref().unwrap();
    let image = attack
        .weapon()
        .unwrap()
        .ground_pickup_before
        .as_ref()
        .unwrap();
    assert_eq!(image.item, original_item);
    assert_eq!(
        image,
        flow(&f.state).unwrap().budget.weapon_history[0]
            .ground_pickup_before
            .as_ref()
            .unwrap()
    );
    let mut expected_item = original_item;
    expected_item.custody = Custody::Entity(f.actor);
    assert_eq!(f.state.items[&f.choice.weapon], expected_item);
    assert!(flow(&f.state).unwrap().ground_items.is_empty());
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
    assert_eq!(flow(&f.state).unwrap().budget.attacks_remaining, 0);
    assert_eq!(
        flow(&f.state).unwrap().budget.attack_window.unwrap().id,
        origin.id
    );
    let paid = f.state.clone();
    RetainedPhysicalRead::new(&f.state, attack, &f.pack).unwrap();
    assert_eq!(f.state, paid);
    assert_eq!(
        before.items[&f.choice.weapon].custody,
        Custody::Location(image.location)
    );
    let detached = attack.clone();
    assert!(RetainedPhysicalRead::new(&f.state, &detached, &f.pack).is_err());
    f.raw(1);
    validate_records(&f.state).unwrap();
    assert!(flow(&f.state).unwrap().resolution.is_none());
    // A completed historical image does not require today's custody to match.
    f.state.items.get_mut(&f.choice.weapon).unwrap().custody = Custody::Entity(f.target);
    validate_records(&f.state).unwrap();
}

#[test]
fn private_different_pickup_keeps_selected_weapon_damage_and_both_item_identities() {
    let mut f = Fixture::new();
    let other = ItemId::new();
    let mut item = f.state.items[&f.choice.weapon].clone();
    item.id = other;
    item.owner = Ownership::Entity(f.target);
    f.state.items.insert(other, item.clone());
    f.pickup(other, Hand::Left);
    f.activate();
    f.begin();
    f.validate_paid();
    let attack = resolution(&f.state).unwrap().attack.as_ref().unwrap();
    assert_eq!(attack.weapon().unwrap().choice.weapon, f.choice.weapon);
    assert_eq!(
        attack
            .weapon()
            .unwrap()
            .ground_pickup_before
            .as_ref()
            .unwrap()
            .item
            .id,
        other
    );
    assert_eq!(attack.damage[0].dice, vec![DieSpec { count: 1, sides: 4 }]);
    assert_eq!(f.state.items[&other].owner, item.owner);
}

#[test]
fn private_pickup_refuses_wrong_grip_unknown_target_and_conflicting_intent_before_commit() {
    for case in 0..4 {
        let mut f = Fixture::new();
        f.pickup(f.choice.weapon, Hand::Right);
        f.activate();
        match case {
            0 => f.choice.grip = WeaponGrip::OneHand(Hand::Left),
            1 => f.choice.target = EntityId::new(),
            2 => f.choice.after_equipment = Some(AfterAttackEquipmentIntent::Choose),
            3 => f.choice.ammunition = Some(ItemId::new()),
            _ => unreachable!(),
        }
        let original = f.state.clone();
        let meta = f.meta(f.actor);
        assert!(
            attacks::begin(&mut f.state, &meta, &f.choice, &f.pack).is_err(),
            "case {case}"
        );
        assert_eq!(f.state, original, "case {case}");
    }
}

#[test]
fn private_activation_lineage_rejects_forged_host_actor_session_executor_and_future_origin() {
    let mut f = Fixture::new();
    f.activate();
    for case in 0..5 {
        let mut state = f.state.clone();
        let applied = state.applied_event_sequence;
        let flow = flow_mut(&mut state).unwrap();
        let access = flow.attack_equipment_access.as_mut().unwrap();
        match case {
            0 => access.origin.issuer = f.meta(f.actor).issuer,
            1 => access.origin.actor = Some(AgentRef::Entity(f.actor)),
            2 => access.origin.session_id = None,
            3 => flow.version = TacticalExecutionVersion::ShieldMissileV1.flow_version(),
            4 => access.origin.expected_event_sequence = applied + 1,
            _ => unreachable!(),
        }
        let before = state.clone();
        assert!(validate_records(&state).is_err(), "case {case}");
        assert_eq!(state, before);
    }
}

#[test]
fn private_source_pickup_late_grip_refusal_preserves_finite_items_and_source_activation() {
    let mut f = Fixture::source(true);
    f.pickup(f.choice.weapon, Hand::Right);
    f.activate();
    let selected = CreatureWeaponUseChoice {
        after_equipment: None,
        weapon: f.choice.weapon,
        target: f.target,
        grip: WeaponGrip::OneHand(Hand::Right),
        ammunition: f.choice.ammunition,
        equipment_change: f.choice.equipment_change,
    };
    let original = f.state.clone();
    let meta = f.meta(f.actor);
    assert!(
        attacks::begin_creature_weapon(&mut f.state, &meta, "shortbow", &selected, &f.pack)
            .is_err()
    );
    assert_eq!(f.state, original);
}

#[test]
fn private_paid_reader_rejects_wrong_activation_duplicate_receipt_payment_raw_and_image() {
    let mut f = Fixture::new();
    f.pickup(f.choice.weapon, Hand::Right);
    f.activate();
    f.begin();
    for case in 0..7 {
        let mut state = f.state.clone();
        match case {
            0 => flow_mut(&mut state).unwrap().attack_equipment_access = None,
            1 => {
                let receipt = flow(&state).unwrap().budget.weapon_history[0].clone();
                flow_mut(&mut state)
                    .unwrap()
                    .budget
                    .weapon_history
                    .push(receipt);
            }
            2 => {
                state
                    .rules
                    .as_mut()
                    .unwrap()
                    .timing
                    .as_mut()
                    .unwrap()
                    .action_spent = false
            }
            3 => {
                state
                    .rules
                    .as_mut()
                    .unwrap()
                    .pending
                    .as_mut()
                    .unwrap()
                    .issued_by
                    .id = CommandId::new()
            }
            4 => state.items.get_mut(&f.choice.weapon).unwrap().quantity += 1,
            5 => {
                resolution_mut(&mut state)
                    .unwrap()
                    .attack
                    .as_mut()
                    .unwrap()
                    .weapon_mut()
                    .unwrap()
                    .ground_pickup_before
                    .as_mut()
                    .unwrap()
                    .ground_index += 1
            }
            6 => {
                let seq = state.applied_event_sequence;
                flow_mut(&mut state)
                    .unwrap()
                    .attack_equipment_access
                    .as_mut()
                    .unwrap()
                    .origin
                    .expected_event_sequence = seq;
            }
            _ => unreachable!(),
        }
        let original = state.clone();
        let attack = resolution(&state).unwrap().attack.as_ref().unwrap();
        assert!(
            RetainedPhysicalRead::new(&state, attack, &f.pack).is_err(),
            "case {case}"
        );
        assert_eq!(state, original, "case {case}");
    }
}

#[test]
fn private_actual_goblin_scimitar_pickup_keeps_source_pin_through_hit_and_damage() {
    let mut f = Fixture::source(false);
    f.pickup(f.choice.weapon, Hand::Right);
    f.activate();
    let source = f
        .state
        .rules
        .as_ref()
        .unwrap()
        .tactical_creatures
        .as_ref()
        .unwrap()
        .profile(f.actor)
        .unwrap()
        .source
        .clone();
    let origin = f.begin_source();
    f.validate_paid();
    let attack = resolution(&f.state).unwrap().attack.as_ref().unwrap();
    assert!(
        matches!(&attack.source, TacticalAttackSource::CreatureWeapon { source: pin, feature_id, .. } if pin == &source && feature_id == "scimitar")
    );
    assert_eq!(attack.origin, origin);
    assert_eq!(attack.attack_modifier, 4);
    assert_eq!(attack.damage[0].dice, vec![DieSpec { count: 1, sides: 6 }]);
    f.raw(20);
    f.validate_paid();
    f.decline_hit();
    f.validate_paid();
    f.raw(1);
    validate_records(&f.state).unwrap();
    assert_eq!(flow(&f.state).unwrap().budget.weapon_history.len(), 1);
}

#[test]
fn private_actual_goblin_shortbow_pickup_reconstructs_paid_finite_ammunition() {
    let mut f = Fixture::source(true);
    f.pickup(f.choice.weapon, Hand::Right);
    f.activate();
    let arrows = f.choice.ammunition.unwrap();
    assert_eq!(f.state.items[&arrows].quantity, 3);
    f.begin_source();
    f.validate_paid();
    assert_eq!(f.state.items[&arrows].quantity, 2);
    f.raw(1);
    validate_records(&f.state).unwrap();
    assert_eq!(f.state.items[&arrows].quantity, 2);
    assert_eq!(flow(&f.state).unwrap().budget.weapon_history.len(), 1);
}

#[test]
fn private_activated_after_choice_declines_after_invalid_apply_without_second_payment() {
    let mut f = Fixture::new();
    f.choice.after_equipment = Some(AfterAttackEquipmentIntent::Choose);
    f.activate();
    f.begin();
    f.validate_paid();
    f.raw(1);
    validate_records(&f.state).unwrap();
    let selected = resolution(&f.state)
        .unwrap()
        .attack_after_equipment
        .as_ref()
        .unwrap();
    let work = TacticalWorkKey {
        resolution: resolution(&f.state).unwrap().origin.id,
        occurrence: selected.work.occurrence,
    };
    let meta = f.meta(f.actor);
    let before = f.state.clone();
    assert!(
        attack_equipment::choose(
            &mut f.state,
            &meta,
            work,
            AttackEquipmentChoice::Apply(AttackEquipmentOperation::Pickup {
                item: ItemId::new(),
                hand: Hand::Left
            }),
            &f.pack
        )
        .is_err()
    );
    assert_eq!(f.state, before);
    attack_equipment::choose(
        &mut f.state,
        &meta,
        work,
        AttackEquipmentChoice::Decline,
        &f.pack,
    )
    .unwrap();
    f.state.applied_event_sequence += 1;
    validate_records(&f.state).unwrap();
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
    assert_eq!(flow(&f.state).unwrap().budget.weapon_history.len(), 1);
    assert!(
        flow(&f.state).unwrap().budget.weapon_history[0]
            .after_equipment
            .as_ref()
            .unwrap()
            .applied
            .is_none()
    );
}

#[test]
fn private_pickup_automatic_miss_reads_actual_entered_finish_without_fabricating_raw() {
    let mut f = Fixture::new();
    f.pickup(f.choice.weapon, Hand::Right);
    f.choice.delivery = WeaponDelivery::Thrown;
    let e = f.state.encounter.as_mut().unwrap();
    e.battlefield.bounds.max.x = 500;
    e.participants
        .iter_mut()
        .find(|p| p.entity_id == f.target)
        .unwrap()
        .position
        .x = 100;
    e.battlefield.terrain.push(TerrainVolume {
        id: "water".into(),
        volume: e.battlefield.bounds,
        difficult: false,
        observable: true,
        water: true,
        climbable: false,
        burrowable: false,
        supports_top: false,
        surface: Some("water".into()),
        obscuration: Obscuration::None,
        magical_darkness: false,
    });
    f.activate();
    let rolls = f.state.rules.as_ref().unwrap().rolls.clone();
    f.begin();
    validate_records(&f.state).unwrap();
    assert_eq!(f.state.rules.as_ref().unwrap().rolls, rolls);
    assert!(flow(&f.state).unwrap().resolution.is_none());
    assert_eq!(
        flow(&f.state).unwrap().budget.weapon_history[0].outcome,
        WeaponAttackOutcome::Miss
    );
    assert_eq!(
        flow(&f.state).unwrap().ground_items[0].item,
        f.choice.weapon
    );
}

#[test]
fn private_pickup_fixed_damage_completes_without_a_damage_roll_or_second_ammo_charge() {
    let mut f = Fixture::new();
    f.borrowed("blowgun");
    f.pickup(f.choice.weapon, Hand::Right);
    let ammo = ItemId::new();
    let mut stack = f.state.items[&f.choice.weapon].clone();
    stack.id = ammo;
    stack.definition_id = "needles".into();
    stack.quantity = 3;
    stack.custody = Custody::Entity(f.actor);
    f.state.items.insert(ammo, stack);
    f.choice.delivery = WeaponDelivery::Shot;
    f.choice.ability = Ability::Dexterity;
    f.choice.ammunition = Some(ammo);
    f.activate();
    f.begin();
    f.validate_paid();
    assert!(
        resolution(&f.state)
            .unwrap()
            .attack
            .as_ref()
            .unwrap()
            .damage
            .iter()
            .all(|d| d.dice.is_empty())
    );
    f.raw(20);
    f.decline_hit();
    validate_records(&f.state).unwrap();
    assert!(flow(&f.state).unwrap().resolution.is_none());
    assert_eq!(f.state.items[&ammo].quantity, 2);
    assert!(f.state.rules.as_ref().unwrap().rolls.iter().all(|r| !matches!(r.purpose, PendingPurpose::TacticalResolution { key, .. } if key.role == TacticalRollRole::AttackDamage)));
}

#[test]
fn private_before_pickup_knockout_uses_the_actual_persisted_choice_without_entered_node() {
    let mut f = Fixture::new();
    f.pickup(f.choice.weapon, Hand::Right);
    f.state
        .rules
        .as_mut()
        .unwrap()
        .entities
        .get_mut(&f.target)
        .unwrap()
        .hp = 1;
    f.activate();
    f.begin();
    f.raw(20);
    f.decline_hit();
    f.raw(1);
    f.validate_paid();
    assert_eq!(
        resolution(&f.state).unwrap().attack.as_ref().unwrap().stage,
        TacticalAttackStage::KnockoutChoice
    );
    assert!(
        resolution(&f.state)
            .unwrap()
            .work_trace
            .as_ref()
            .unwrap()
            .active
            .is_none()
    );
    let meta = f.meta(f.actor);
    assert!(RetainedPhysicalRead::entered_completion(&f.state, &meta, &f.pack).is_err());
    attacks::choose_knockout(&mut f.state, &meta, KnockoutChoice::KnockOut).unwrap();
    f.state.applied_event_sequence += 1;
    validate_records(&f.state).unwrap();
    assert!(flow(&f.state).unwrap().resolution.is_none());
}

#[test]
fn private_before_pickup_graze_choice_keeps_true_pause_without_inventing_an_entered_parent() {
    for accept in [false, true] {
        let mut f = Fixture::new();
        f.grant_graze();
        f.pickup(f.choice.weapon, Hand::Right);
        f.activate();
        f.begin();
        f.raw(1);
        f.validate_paid();
        assert_eq!(
            resolution(&f.state).unwrap().attack.as_ref().unwrap().stage,
            TacticalAttackStage::MasteryChoice
        );
        let meta = f.meta(f.actor);
        assert!(RetainedPhysicalRead::entered_completion(&f.state, &meta, &f.pack).is_err());
        attacks::choose_mastery(
            &mut f.state,
            &meta,
            &if accept {
                WeaponMasteryChoice::Graze
            } else {
                WeaponMasteryChoice::Decline
            },
        )
        .unwrap();
        f.state.applied_event_sequence += 1;
        validate_records(&f.state).unwrap();
        assert!(flow(&f.state).unwrap().resolution.is_none());
        assert_eq!(flow(&f.state).unwrap().budget.weapon_history.len(), 1);
    }
}

#[test]
fn private_activated_printed_after_intent_uses_the_same_source_activation_and_actual_choice() {
    let mut f = Fixture::source(false);
    f.state
        .rules
        .as_mut()
        .unwrap()
        .tactical_inventory
        .as_mut()
        .unwrap()
        .loadouts
        .iter_mut()
        .find(|l| l.actor == f.actor)
        .unwrap()
        .hands
        .hands[Hand::Right.index()] = HandAssignment::Item(f.choice.weapon);
    f.choice.after_equipment = Some(AfterAttackEquipmentIntent::Choose);
    f.activate();
    let original = f.begin_source();
    f.validate_paid();
    f.raw(1);
    validate_records(&f.state).unwrap();
    let resolution = resolution(&f.state).unwrap();
    let selected = resolution.attack_after_equipment.as_ref().unwrap();
    assert_eq!(selected.cause.origin, original);
    assert!(
        matches!(&selected.cause.source, AttackEquipmentSource::Creature { source, feature_id } if source.definition_id == "goblin-warrior" && feature_id == "scimitar")
    );
    let work = TacticalWorkKey {
        resolution: resolution.origin.id,
        occurrence: selected.work.occurrence,
    };
    let meta = f.meta(f.actor);
    attack_equipment::choose(
        &mut f.state,
        &meta,
        work,
        AttackEquipmentChoice::Decline,
        &f.pack,
    )
    .unwrap();
    f.state.applied_event_sequence += 1;
    validate_records(&f.state).unwrap();
    assert_eq!(flow(&f.state).unwrap().budget.weapon_history.len(), 1);
}

// Each corruption starts from a real private paid producer cut. None of these
// copied negative images is an accepted history or a reader authority source.
fn paid_pending_fixture(damage: bool) -> Fixture {
    let mut f = Fixture::new();
    f.pickup(f.choice.weapon, Hand::Right);
    f.activate();
    f.begin();
    if damage {
        f.raw(20);
        f.decline_hit();
    }
    assert_eq!(
        resolution(&f.state).unwrap().attack.as_ref().unwrap().stage,
        if damage {
            TacticalAttackStage::DamageRoll
        } else {
            TacticalAttackStage::AttackRoll
        }
    );
    let before = f.state.clone();
    let attack = resolution(&f.state).unwrap().attack.as_ref().unwrap();
    RetainedPhysicalRead::new(&f.state, attack, &f.pack).unwrap();
    assert_eq!(f.state, before);
    f
}

fn reject_paid_pending(state: CampaignState, pack: &RulesPack) {
    let before = state.clone();
    let attack = resolution(&state).unwrap().attack.as_ref().unwrap();
    assert!(RetainedPhysicalRead::new(&state, attack, pack).is_err());
    assert_eq!(
        state, before,
        "reader refusal must preserve the complete paid cut"
    );
}

#[test]
fn private_paid_pending_request_id_is_bound_before_inverse() {
    for damage in [false, true] {
        let f = paid_pending_fixture(damage);
        let mut state = f.state.clone();
        state
            .rules
            .as_mut()
            .unwrap()
            .pending
            .as_mut()
            .unwrap()
            .request
            .id = RollRequestId::new();
        reject_paid_pending(state, &f.pack);
    }
}

#[test]
fn private_paid_pending_complete_template_and_ruling_are_bound_before_inverse() {
    for damage in [false, true] {
        let f = paid_pending_fixture(damage);
        for case in 0..7 {
            let mut state = f.state.clone();
            let pending = state.rules.as_mut().unwrap().pending.as_mut().unwrap();
            match case {
                0 => pending.request.modifier += 1,
                1 => pending.request.dice[0].count += 1,
                2 => {
                    pending.request.mode = if pending.request.mode == RollMode::Normal {
                        RollMode::Advantage
                    } else {
                        RollMode::Normal
                    }
                }
                3 => pending.request.roller = Some(f.target),
                4 => {
                    pending.request.visibility =
                        if pending.request.visibility == RollVisibility::Public {
                            RollVisibility::Secret
                        } else {
                            RollVisibility::Public
                        }
                }
                5 => pending.request.reason.push_str(" altered"),
                6 => pending.ruling.reason.push_str(" altered"),
                _ => unreachable!(),
            }
            reject_paid_pending(state, &f.pack);
        }
    }
}

#[test]
fn private_paid_pending_purpose_is_bound_before_inverse() {
    for damage in [false, true] {
        let f = paid_pending_fixture(damage);
        for case in 0..3 {
            let mut state = f.state.clone();
            let mut key = resolution(&state).unwrap().pending.as_ref().unwrap().key;
            let encounter = encounter(&state).unwrap().id;
            let purpose = match case {
                0 => PendingPurpose::RestHitDie,
                1 => PendingPurpose::TacticalResolution {
                    encounter: EncounterId::new(),
                    key,
                },
                2 => {
                    key.subject = f.actor;
                    PendingPurpose::TacticalResolution { encounter, key }
                }
                _ => unreachable!(),
            };
            state
                .rules
                .as_mut()
                .unwrap()
                .pending
                .as_mut()
                .unwrap()
                .purpose = purpose;
            reject_paid_pending(state, &f.pack);
        }
    }
}

#[test]
fn private_paid_pending_selected_key_is_bound_to_actual_work_before_inverse() {
    for damage in [false, true] {
        let f = paid_pending_fixture(damage);
        for coherently_copied in [false, true] {
            let mut state = f.state.clone();
            let selected = resolution_mut(&mut state)
                .unwrap()
                .pending
                .as_mut()
                .unwrap();
            selected.key.origin = CommandId::new();
            let key = selected.key;
            if coherently_copied {
                let encounter = encounter(&state).unwrap().id;
                let pending = state.rules.as_mut().unwrap().pending.as_mut().unwrap();
                pending.purpose = PendingPurpose::TacticalResolution { encounter, key };
                pending.request.id = key.request_id();
            }
            reject_paid_pending(state, &f.pack);
        }
    }
}

#[test]
fn private_paid_pending_exact_trace_node_is_required_before_inverse() {
    for damage in [false, true] {
        let f = paid_pending_fixture(damage);
        for case in 0..4 {
            let mut state = f.state.clone();
            let r = resolution_mut(&mut state).unwrap();
            let work = r.pending.as_ref().unwrap().work.clone();
            let trace = r.work_trace.as_mut().unwrap();
            let index = trace.nodes.iter().position(|n| n.work == work).unwrap();
            match case {
                0 => {
                    trace.nodes.remove(index);
                }
                1 => trace.nodes[index].work.kind = TacticalWorkKind::FinishAttack,
                2 => trace.nodes.push(trace.nodes[index].clone()),
                3 => {
                    let occurrence = r.next_occurrence;
                    r.next_occurrence += 1;
                    trace.nodes.push(TacticalWorkNode {
                        work: TacticalWorkItem {
                            occurrence,
                            kind: work.kind,
                        },
                        parent: trace.nodes[index].parent,
                    });
                }
                _ => unreachable!(),
            }
            reject_paid_pending(state, &f.pack);
        }
    }
}

#[test]
fn private_paid_pending_issued_cause_is_bound_at_attack_and_damage_cuts() {
    for damage in [false, true] {
        let f = paid_pending_fixture(damage);
        let mut state = f.state.clone();
        state
            .rules
            .as_mut()
            .unwrap()
            .pending
            .as_mut()
            .unwrap()
            .issued_by
            .id = CommandId::new();
        reject_paid_pending(state, &f.pack);
    }
}

#[test]
fn private_paid_pending_surfaces_follow_the_actual_persisted_stage() {
    for damage in [false, true] {
        let f = paid_pending_fixture(damage);
        for case in 0..3 {
            let mut state = f.state.clone();
            match case {
                0 => state.rules.as_mut().unwrap().pending = None,
                1 => resolution_mut(&mut state).unwrap().pending = None,
                2 => {
                    state.rules.as_mut().unwrap().pending = None;
                    let r = resolution_mut(&mut state).unwrap();
                    let work = r.pending.take().unwrap().work;
                    r.frames.push(vec![work]);
                }
                _ => unreachable!(),
            }
            reject_paid_pending(state, &f.pack);
        }
    }
    let mut f = paid_pending_fixture(false);
    let previous_raw = f.state.rules.as_ref().unwrap().pending.clone();
    let previous_work = resolution(&f.state).unwrap().pending.clone();
    f.raw(20);
    assert_eq!(
        resolution(&f.state).unwrap().attack.as_ref().unwrap().stage,
        TacticalAttackStage::HitReview
    );
    assert!(f.state.rules.as_ref().unwrap().pending.is_none());
    assert!(resolution(&f.state).unwrap().pending.is_none());
    let attack = resolution(&f.state).unwrap().attack.as_ref().unwrap();
    RetainedPhysicalRead::new(&f.state, attack, &f.pack).unwrap();
    for case in 0..3 {
        let mut state = f.state.clone();
        if case != 1 {
            state.rules.as_mut().unwrap().pending = previous_raw.clone();
        }
        if case != 0 {
            resolution_mut(&mut state).unwrap().pending = previous_work.clone();
        }
        reject_paid_pending(state, &f.pack);
    }
}
