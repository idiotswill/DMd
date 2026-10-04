//! Constructed initial images, then actual private producers. These are not
//! admitted app histories, original-journal replay or portable restore evidence.
use super::*;
use crate::tactical_creatures::{CreatureBuildChoice, CreatureHitPointChoice, build_creature};
use crate::tactical_definitions::{WeaponHands, bundled_tactical_definitions};
use crate::tactical_effects::*;

struct Fixture {
    state: CampaignState,
    pack: RulesPack,
    actor: EntityId,
    target: EntityId,
    choice: WeaponUseChoice,
    session: Option<PlaySessionId>,
}
impl Fixture {
    fn new(weapon: &str, mastery: bool) -> Self {
        let export: serde_json::Value = serde_json::from_str(include_str!(
            "../../../dmd-app/tests/fixtures/shield-hit-v1-selected.json"
        ))
        .unwrap();
        let mut state: CampaignState =
            serde_json::from_str(export["current_state"]["state_json"].as_str().unwrap()).unwrap();
        let attack = resolution(&state).unwrap().attack.as_ref().unwrap().clone();
        let actor = attack.actor;
        let target = attack.target;
        let session = attack.origin.session_id;
        let pack = RulesPack::from_json(include_str!("../../../../content/srd-5.2.1/kernel.json"))
            .unwrap();
        // Build a genuine canonical Fighter baseline with real selected mastery
        // grants. No source definition is edited to manufacture a positive case.
        let first = if mastery { weapon } else { "club" };
        let input = crate::CharacterCreationInput {
            name: "Equipment controller".into(),
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
            masteries: [first.into(), "dagger".into(), "shortbow".into()],
        };
        let built = crate::build_character(&input, actor, &pack).unwrap();
        let character = state
            .characters
            .values()
            .find(|c| c.entity_id == actor)
            .unwrap()
            .id;
        state
            .table
            .as_mut()
            .unwrap()
            .character_profiles
            .insert(character, built.profile);
        // Reconstitute this constructed Fighter baseline through the actual
        // grant producer. Frozen exports and their accepted grants are untouched.
        let grant = CommandMeta {
            id: CommandId::new(),
            campaign_id: state.campaign_id(),
            session_id: session,
            issuer: CommandIssuer::Player(controller(&state, actor).unwrap()),
            actor: Some(AgentRef::Entity(actor)),
            expected_event_sequence: state.applied_event_sequence,
        };
        let rules = state.rules.as_mut().unwrap();
        rules.entities.insert(actor, built.mechanics);
        let inventory = rules.tactical_inventory.as_mut().unwrap();
        inventory.receipts.retain(|r| r.actor != actor);
        inventory.loadouts.retain(|l| l.actor != actor);
        let provisioned = crate::tactical_inventory::materialize_starting_equipment(
            &state,
            state
                .rules
                .as_ref()
                .unwrap()
                .tactical_inventory
                .as_ref()
                .unwrap(),
            &grant,
            character,
            &[],
            &pack,
        )
        .unwrap();
        state = provisioned.next_state;
        state.rules.as_mut().unwrap().tactical_inventory = Some(provisioned.next_inventory);
        state.applied_event_sequence += 1;
        let definition = bundled_tactical_definitions()
            .unwrap()
            .weapon(weapon)
            .unwrap();
        let grip = if definition.hands == WeaponHands::One {
            WeaponGrip::OneHand(Hand::Right)
        } else {
            WeaponGrip::TwoHands
        };
        // Ordinary borrowed equipment in the constructed baseline; existing
        // physical Item definitions are not rewritten.
        let item = ItemId::new();
        state.items.insert(
            item,
            ItemInstance {
                id: item,
                campaign_id: state.campaign_id(),
                definition_id: weapon.into(),
                display_name: "Borrowed physical weapon".into(),
                quantity: 1,
                owner: Ownership::Entity(target),
                custody: Custody::Entity(actor),
                state: ItemState::Intact,
            },
        );
        let ammunition =
            crate::tactical_weapons::required_ammunition_definition(definition).map(|definition| {
                let id = ItemId::new();
                state.items.insert(
                    id,
                    ItemInstance {
                        id,
                        campaign_id: state.campaign_id(),
                        definition_id: definition.into(),
                        display_name: "Physical ammunition".into(),
                        quantity: 20,
                        owner: Ownership::Entity(actor),
                        custody: Custody::Entity(actor),
                        state: ItemState::Intact,
                    },
                );
                id
            });
        let rules = state.rules.as_mut().unwrap();
        rules.pending = None;
        rules
            .rolls
            .retain(|r| !matches!(r.purpose, PendingPurpose::TacticalResolution { .. }));
        rules.timing.as_mut().unwrap().action_spent = false;
        let loadout = rules
            .tactical_inventory
            .as_mut()
            .unwrap()
            .loadouts
            .iter_mut()
            .find(|l| l.actor == actor)
            .unwrap();
        loadout.worn_armor = None;
        loadout.shield = None;
        loadout.hands.hands = if grip == WeaponGrip::TwoHands {
            [HandAssignment::Item(item); 2]
        } else {
            [HandAssignment::Free, HandAssignment::Item(item)]
        };
        let f = flow_mut(&mut state).unwrap();
        f.version = TacticalExecutionVersion::EncounterReleaseV1.flow_version();
        f.phase = TacticalPhase::Active;
        f.resolution = None;
        f.budget = TacticalTurnBudget::default();
        f.ground_items.clear();
        let choice = WeaponUseChoice {
            after_equipment: Some(AfterAttackEquipmentIntent::Choose),
            weapon: item,
            target,
            delivery: if ammunition.is_some() {
                WeaponDelivery::Shot
            } else {
                WeaponDelivery::Melee
            },
            ability: if ammunition.is_some() {
                Ability::Dexterity
            } else {
                Ability::Strength
            },
            grip,
            purpose: WeaponAttackPurpose::Normal,
            ammunition,
            equipment_change: None,
        };
        Self {
            state,
            pack,
            actor,
            target,
            choice,
            session,
        }
    }
    fn meta(&self, actor: EntityId) -> CommandMeta {
        CommandMeta {
            id: CommandId::new(),
            campaign_id: self.state.campaign_id(),
            session_id: self.session,
            issuer: controller(&self.state, actor)
                .map_or(CommandIssuer::Admin, CommandIssuer::Player),
            actor: Some(AgentRef::Entity(actor)),
            expected_event_sequence: self.state.applied_event_sequence,
        }
    }
    fn advance(&mut self) {
        self.state.applied_event_sequence += 1;
    }
    fn begin(&mut self) -> CommandMeta {
        let meta = self.meta(self.actor);
        attacks::begin(&mut self.state, &meta, &self.choice, &self.pack).unwrap();
        self.advance();
        meta
    }
    fn raw(&mut self, face: u16) -> CommandMeta {
        let pending = self.state.rules.as_ref().unwrap().pending.as_ref().unwrap();
        let request = &pending.request;
        let count = if request.mode == RollMode::Normal {
            1
        } else {
            2
        };
        let dice = request
            .dice
            .iter()
            .flat_map(|die| {
                (0..die.count * count).map(move |_| DieResult {
                    sides: die.sides,
                    value: face.min(die.sides),
                })
            })
            .collect();
        let raw = RollResult {
            request_id: request.id,
            source: RollSource::Physical,
            dice,
        };
        let meta = self.meta(request.roller.unwrap());
        continuations::submit(&mut self.state, &meta, &raw, None).unwrap();
        self.advance();
        meta
    }
    fn decline_hit(&mut self) {
        let hit = resolution(&self.state)
            .unwrap()
            .hit_review
            .as_ref()
            .unwrap()
            .clone();
        let window = TacticalWorkKey {
            resolution: resolution(&self.state).unwrap().origin.id,
            occurrence: hit.work.occurrence,
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
        self.advance();
        let meta = self.meta(self.target);
        hit_reactions::respond(&mut self.state, &meta, window, false).unwrap();
        self.advance();
    }
    fn selected(&self) -> &TacticalAttackAfterEquipment {
        resolution(&self.state)
            .unwrap()
            .attack_after_equipment
            .as_deref()
            .unwrap()
    }
    fn decision(&mut self, choice: AttackEquipmentChoice) -> CommandMeta {
        let meta = self.meta(self.actor);
        let work = key(&self.state, self.selected().work.occurrence).unwrap();
        choose(&mut self.state, &meta, work, choice, &self.pack).unwrap();
        self.advance();
        meta
    }
    fn reject(&mut self, meta: CommandMeta, work: TacticalWorkKey, choice: AttackEquipmentChoice) {
        let before = self.state.clone();
        assert!(choose(&mut self.state, &meta, work, choice, &self.pack).is_err());
        assert_eq!(
            self.state, before,
            "rejected equipment decision must preserve the entire cut"
        );
    }
    fn source_flyer(&mut self) {
        // A real source-built new Chimera supplies non-Hover flight. The captured
        // Mage is not edited into a new creature or granted invented mechanics.
        let actor = EntityId::new();
        let mut world = self.state.entities[&self.target].clone();
        world.id = actor;
        self.state.entities.insert(actor, world);
        let mut meta = self.meta(actor);
        meta.issuer = CommandIssuer::Admin;
        let built = build_creature(
            &self.state,
            &meta,
            actor,
            &CreatureBuildChoice {
                definition_id: "chimera".into(),
                size: CreatureSize::Large,
                additional_languages: vec![],
                hit_points: CreatureHitPointChoice::Average,
                controller: CreatureController::Host,
                in_lair: false,
            },
        )
        .unwrap();
        let rules = self.state.rules.as_mut().unwrap();
        rules.entities.insert(actor, built.mechanics);
        rules.entities.get_mut(&actor).unwrap().hp = 1; // A wounded real source baseline.
        let creatures = rules.tactical_creatures.get_or_insert_default();
        creatures.profiles.push(built.profile);
        creatures.runtime.push(built.runtime);
        for entry in &mut rules.timing.as_mut().unwrap().order {
            if entry.actor == self.target {
                entry.actor = actor;
            }
        }
        let e = self.state.encounter.as_mut().unwrap();
        // Keep the real elevated bodies inside the constructed battlefield.
        // The ledge and the flyer's fall distance remain twenty feet.
        e.battlefield.bounds.max.z = 60;
        let target = e
            .participants
            .iter_mut()
            .find(|p| p.entity_id == self.target)
            .unwrap();
        target.entity_id = actor;
        target.size = CreatureSize::Large;
        target.height = 20;
        target.movement = built.movement;
        target.senses = built.senses;
        target.position = SpatialPoint {
            x: 30,
            y: 10,
            z: 40,
        };
        let attacker = e
            .participants
            .iter_mut()
            .find(|p| p.entity_id == self.actor)
            .unwrap();
        attacker.position.z = 40;
        attacker.enemies = vec![actor];
        e.flow
            .as_mut()
            .unwrap()
            .combatants
            .iter_mut()
            .find(|c| c.actor == self.target)
            .unwrap()
            .actor = actor;
        e.flow
            .as_mut()
            .unwrap()
            .combatants
            .iter_mut()
            .find(|c| c.actor == actor)
            .unwrap()
            .source = TacticalSource::Creature {
            definition_id: "chimera".into(),
        };
        e.battlefield.obstacles.push(SpatialObstacle {
            id: "supporting-ledge".into(),
            volume: SpatialBox {
                min: SpatialPoint { x: 0, y: 0, z: 0 },
                max: SpatialPoint {
                    x: 20,
                    y: 30,
                    z: 40,
                },
            },
            blocks_movement: true,
            blocks_sight: true,
            observable: true,
            cover: CoverDegree::Total,
        });
        let scene = e.scene_id;
        self.state
            .scenes
            .get_mut(&scene)
            .unwrap()
            .presences
            .iter_mut()
            .find(|p| p.entity_id == self.target)
            .unwrap()
            .entity_id = actor;
        self.target = actor;
        self.choice.target = actor;
        self.advance();
        let encounter = self.state.encounter.as_ref().unwrap();
        crate::spatial::validate_encounter(encounter, &self.state).unwrap();
        assert!(
            crate::spatial::perceive(encounter, &self.state, self.actor, self.target)
                .unwrap()
                .precisely_located
        );
    }
}

#[test]
fn private_miss_produces_one_selected_work_then_explicit_decline_without_repeating_attack() {
    let mut f = Fixture::new("javelin", false);
    let origin = f.begin();
    let completed = f.raw(1);
    let record = f.selected().clone();
    assert_eq!(record.cause.origin, origin);
    assert_eq!(record.cause.completed_by, completed);
    assert_eq!(record.selected_by, Some(completed));
    assert_eq!(
        node(&f.state, record.cause.completed_work)
            .unwrap()
            .work
            .kind,
        TacticalWorkKind::FinishAttack
    );
    assert_eq!(
        node(&f.state, key(&f.state, record.work.occurrence).unwrap())
            .unwrap()
            .parent,
        Some(record.cause.completed_work.occurrence)
    );
    assert!(resolution(&f.state).unwrap().attack.is_none());
    assert!(
        resolution(&f.state)
            .unwrap()
            .work_trace
            .as_ref()
            .unwrap()
            .active
            .is_none()
    );
    assert!(f.state.rules.as_ref().unwrap().pending.is_none());
    assert_eq!(flow(&f.state).unwrap().budget.attacks_remaining, 0);
    validate(&f.state).unwrap();
    let rolls = f.state.rules.as_ref().unwrap().rolls.clone();
    let chosen = f.decision(AttackEquipmentChoice::Decline);
    assert!(flow(&f.state).unwrap().resolution.is_none());
    let final_record = flow(&f.state).unwrap().budget.weapon_history[0]
        .after_equipment
        .as_ref()
        .unwrap();
    assert_eq!(final_record.chosen_by, chosen);
    assert!(final_record.applied.is_none());
    assert_eq!(f.state.rules.as_ref().unwrap().rolls, rolls);
    assert_eq!(flow(&f.state).unwrap().budget.weapon_history.len(), 1);
}

#[test]
fn private_actual_throw_then_same_item_pickup_uses_completed_ground_and_spent_allowance() {
    let mut f = Fixture::new("javelin", false);
    f.choice.delivery = WeaponDelivery::Thrown;
    f.begin();
    let completed = f.raw(1);
    let before = f.state.clone();
    let ground = flow(&before).unwrap().ground_items[0].clone();
    assert_eq!(ground.origin, completed);
    assert_eq!(ground.item, f.choice.weapon);
    assert_eq!(flow(&before).unwrap().budget.attacks_remaining, 0);
    let chosen = f.decision(AttackEquipmentChoice::Apply(
        AttackEquipmentOperation::Pickup {
            item: f.choice.weapon,
            hand: Hand::Right,
        },
    ));
    let item = &f.state.items[&f.choice.weapon];
    assert_eq!(item.custody, Custody::Entity(f.actor));
    assert_eq!(item.owner, before.items[&f.choice.weapon].owner);
    assert_eq!(item.quantity, before.items[&f.choice.weapon].quantity);
    assert_eq!(
        item.definition_id,
        before.items[&f.choice.weapon].definition_id
    );
    assert!(flow(&f.state).unwrap().ground_items.is_empty());
    let receipt = flow(&f.state).unwrap().budget.weapon_history[0]
        .after_equipment
        .as_ref()
        .unwrap();
    assert_eq!(receipt.chosen_by, chosen);
    assert_eq!(
        receipt
            .applied
            .as_ref()
            .unwrap()
            .ground_before
            .as_ref()
            .unwrap()
            .ground,
        ground
    );
    assert_eq!(
        f.state.rules.as_ref().unwrap().entities,
        before.rules.as_ref().unwrap().entities
    );
    assert_eq!(
        f.state.rules.as_ref().unwrap().rolls,
        before.rules.as_ref().unwrap().rolls
    );
}

#[test]
fn private_rolled_hit_and_fixed_damage_finish_before_equipment_selection() {
    for fixed in [false, true] {
        let mut f = Fixture::new(if fixed { "blowgun" } else { "javelin" }, false);
        let hp = f.state.rules.as_ref().unwrap().entities[&f.target].hp;
        f.begin();
        f.raw(20);
        f.decline_hit();
        if !fixed {
            f.raw(1);
        }
        assert_eq!(
            node(&f.state, f.selected().cause.completed_work)
                .unwrap()
                .work
                .kind,
            TacticalWorkKind::AttackDamage
        );
        assert!(f.state.rules.as_ref().unwrap().entities[&f.target].hp < hp);
        if fixed {
            assert_eq!(
                f.state.rules.as_ref().unwrap().entities[&f.target].hp,
                hp - 1
            );
            assert!(!f.state.rules.as_ref().unwrap().rolls.iter().any(|r| matches!(r.purpose,PendingPurpose::TacticalResolution { key, .. } if key.role == TacticalRollRole::AttackDamage)));
            assert_eq!(f.state.items[&f.choice.ammunition.unwrap()].quantity, 19);
        }
        let rolls = f.state.rules.as_ref().unwrap().rolls.clone();
        f.decision(AttackEquipmentChoice::Apply(
            AttackEquipmentOperation::Unequip {
                item: f.choice.weapon,
            },
        ));
        assert_eq!(f.state.rules.as_ref().unwrap().rolls, rolls);
        assert_eq!(flow(&f.state).unwrap().budget.weapon_history.len(), 1);
    }
}

#[test]
fn private_knockout_resumes_the_actual_damage_parent_and_rejects_sibling_forgery() {
    let mut f = Fixture::new("javelin", false);
    f.state
        .rules
        .as_mut()
        .unwrap()
        .entities
        .get_mut(&f.target)
        .unwrap()
        .hp = 1;
    f.begin();
    f.raw(20);
    f.decline_hit();
    f.raw(1);
    let parent = resolution(&f.state)
        .unwrap()
        .attack
        .as_ref()
        .unwrap()
        .weapon()
        .unwrap()
        .after_equipment_parent
        .clone()
        .unwrap();
    assert_eq!(parent.pause, AttackEquipmentPause::Knockout);
    assert!(
        resolution(&f.state)
            .unwrap()
            .attack_after_equipment
            .is_none()
    );
    let mut wrong = f.state.clone();
    resolution_mut(&mut wrong)
        .unwrap()
        .attack
        .as_mut()
        .unwrap()
        .weapon_mut()
        .unwrap()
        .after_equipment_parent
        .as_mut()
        .unwrap()
        .work
        .occurrence = 0;
    assert!(validate_pause(&wrong).is_err());
    let meta = f.meta(f.actor);
    attacks::choose_knockout(&mut f.state, &meta, KnockoutChoice::KnockOut).unwrap();
    f.advance();
    assert_eq!(f.selected().cause.completed_work, parent.work);
    assert_eq!(f.selected().cause.completed_by, meta);
    assert!(
        resolution(&f.state)
            .unwrap()
            .work_trace
            .as_ref()
            .unwrap()
            .active
            .is_none()
    );
    assert_eq!(f.state.rules.as_ref().unwrap().entities[&f.target].hp, 1);
    f.decision(AttackEquipmentChoice::Decline);
}

#[test]
fn private_real_graze_apply_and_decline_keep_finish_parent_without_fabricated_raw() {
    for accept in [false, true] {
        let mut f = Fixture::new("greatsword", true);
        let hp = f.state.rules.as_ref().unwrap().entities[&f.target].hp;
        f.begin();
        f.raw(1);
        let attack = resolution(&f.state).unwrap().attack.as_ref().unwrap();
        let parent = attack
            .weapon()
            .unwrap()
            .after_equipment_parent
            .clone()
            .unwrap();
        assert_eq!(parent.pause, AttackEquipmentPause::Graze);
        assert_eq!(parent.accepted_raw, attack.attack_roll);
        assert!(
            resolution(&f.state)
                .unwrap()
                .attack_after_equipment
                .is_none()
        );
        let meta = f.meta(f.actor);
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
        f.advance();
        assert_eq!(f.selected().cause.completed_work, parent.work);
        assert_eq!(f.selected().cause.completed_by, meta);
        assert_eq!(
            f.state.rules.as_ref().unwrap().entities[&f.target].hp,
            if accept { hp - 3 } else { hp }
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
        f.decision(AttackEquipmentChoice::Decline);
    }
}

#[test]
fn private_direct_knockout_flight_loss_is_a_child_of_retained_damage_before_equipment() {
    let mut f = Fixture::new("javelin", false);
    f.source_flyer();
    f.begin();
    f.raw(20);
    f.decline_hit();
    f.raw(1);
    let parent = resolution(&f.state)
        .unwrap()
        .attack
        .as_ref()
        .unwrap()
        .weapon()
        .unwrap()
        .after_equipment_parent
        .clone()
        .unwrap();
    let meta = f.meta(f.actor);
    attacks::choose_knockout(&mut f.state, &meta, KnockoutChoice::KnockOut).unwrap();
    f.advance();
    let r = resolution(&f.state).unwrap();
    let after = r.attack_after_equipment.as_ref().unwrap();
    assert!(
        after.selected_by.is_none(),
        "queued after work must not interrupt falling"
    );
    let trace = r.work_trace.as_ref().unwrap();
    let begin = trace
        .nodes
        .iter()
        .find(|n| matches!(n.work.kind, TacticalWorkKind::BeginFall { .. }))
        .unwrap();
    assert_eq!(begin.parent, Some(parent.work.occurrence));
    assert!(begin.work.occurrence > after.work.occurrence);
    assert!(trace.active.is_none());
    assert!(r.frames.iter().any(|frame| frame.contains(&after.work)));
    assert!(matches!(
        r.pending.as_ref().unwrap().work.kind,
        TacticalWorkKind::FallDamage { .. }
    ));
    f.raw(1);
    assert!(f.selected().selected_by.is_some());
    assert_eq!(
        f.state
            .encounter
            .as_ref()
            .unwrap()
            .participant(f.target)
            .unwrap()
            .position
            .z,
        0
    );
    f.decision(AttackEquipmentChoice::Decline);
}

#[test]
fn private_automatic_underwater_miss_completes_without_any_attack_raw_or_pause_parent() {
    let mut f = Fixture::new("javelin", false);
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
    let rolls = f.state.rules.as_ref().unwrap().rolls.clone();
    f.begin();
    assert_eq!(f.selected().cause.outcome, WeaponAttackOutcome::Miss);
    assert_eq!(
        node(&f.state, f.selected().cause.completed_work)
            .unwrap()
            .work
            .kind,
        TacticalWorkKind::FinishAttack
    );
    assert_eq!(f.state.rules.as_ref().unwrap().rolls, rolls);
    assert!(resolution(&f.state).unwrap().attack.is_none());
    f.decision(AttackEquipmentChoice::Decline);
}

#[test]
fn private_invalid_apply_is_atomic_and_decline_does_not_require_living_actor_or_item() {
    let mut f = Fixture::new("javelin", false);
    f.choice.delivery = WeaponDelivery::Thrown;
    f.begin();
    f.raw(1);
    let work = key(&f.state, f.selected().work.occurrence).unwrap();
    let op = AttackEquipmentChoice::Apply(AttackEquipmentOperation::Pickup {
        item: f.choice.weapon,
        hand: Hand::Right,
    });
    let good = f.state.clone();
    for case in 0..5 {
        f.state = good.clone();
        match case {
            0 => {
                f.state.items.get_mut(&f.choice.weapon).unwrap().custody =
                    Custody::Entity(f.target);
            }
            1 => {
                flow_mut(&mut f.state).unwrap().ground_items[0].position.x += 200;
            }
            2 => {
                f.state
                    .encounter
                    .as_mut()
                    .unwrap()
                    .battlefield
                    .ambient_light = LightLevel::Darkness;
            }
            3 => {
                f.state
                    .rules
                    .as_mut()
                    .unwrap()
                    .entities
                    .get_mut(&f.actor)
                    .unwrap()
                    .hp = 0;
            }
            _ => {
                f.state.items.get_mut(&f.choice.weapon).unwrap().quantity = 2;
            }
        }
        f.reject(f.meta(f.actor), work, op);
    }
    f.state = good;
    let actor = f
        .state
        .rules
        .as_mut()
        .unwrap()
        .entities
        .get_mut(&f.actor)
        .unwrap();
    actor.hp = 0;
    actor.death.dead = true;
    f.decision(AttackEquipmentChoice::Decline);
    assert!(flow(&f.state).unwrap().resolution.is_none());
}

#[test]
fn private_choice_rejects_wrong_authority_head_key_and_duplicate_work_without_mutation() {
    let mut f = Fixture::new("javelin", false);
    f.begin();
    f.raw(1);
    let work = key(&f.state, f.selected().work.occurrence).unwrap();
    for case in 0..7 {
        let mut meta = f.meta(f.actor);
        let mut bad = work;
        match case {
            0 => meta.actor = Some(AgentRef::Entity(f.target)),
            1 => meta.campaign_id = CampaignId::new(),
            2 => meta.expected_event_sequence -= 1,
            3 => meta.session_id = Some(PlaySessionId::new()),
            4 => bad.resolution = CommandId::new(),
            5 => meta.issuer = CommandIssuer::Admin,
            _ => {
                meta.issuer = CommandIssuer::Admin;
                meta.actor = None;
            }
        }
        f.reject(meta, bad, AttackEquipmentChoice::Decline);
    }
    let good = f.state.clone();
    let duplicate = f.selected().work.clone();
    resolution_mut(&mut f.state)
        .unwrap()
        .frames
        .push(vec![duplicate]);
    f.reject(f.meta(f.actor), work, AttackEquipmentChoice::Decline);
    f.state = good;
    let meta = f.decision(AttackEquipmentChoice::Decline);
    f.reject(meta, work, AttackEquipmentChoice::Decline);
}

#[test]
fn private_inverse_round_trip_requires_exact_selected_cause_and_preserves_unrelated_state() {
    let mut f = Fixture::new("javelin", false);
    f.choice.delivery = WeaponDelivery::Thrown;
    f.begin();
    f.raw(1);
    let meta = f.meta(f.actor);
    let record = f.selected().clone();
    let token = SelectedEquipment {
        state: &f.state,
        origin: meta.clone(),
        actor: f.actor,
        window: record.cause.window,
    };
    let operation = AttackEquipmentOperation::Pickup {
        item: f.choice.weapon,
        hand: Hand::Right,
    };
    let prepared =
        crate::tactical_weapons::ground::prepare_after(&token, operation, &f.pack).unwrap();
    let (candidate, image) = prepared.consume(&f.state).unwrap();
    let receipt = AttackAfterEquipmentReceipt {
        cause: record.cause,
        work: key(&f.state, record.work.occurrence).unwrap(),
        selected_by: record.selected_by.unwrap(),
        chosen_by: meta,
        applied: Some(Box::new(image)),
    };
    assert_eq!(
        crate::tactical_weapons::ground::restore_after_image(&candidate, &receipt, &f.pack)
            .unwrap(),
        f.state
    );
    for case in 0..5 {
        let mut bad = receipt.clone();
        match case {
            0 => bad.work.occurrence += 1,
            1 => bad.cause.completed_work.occurrence += 1,
            2 => {
                bad.applied
                    .as_mut()
                    .unwrap()
                    .ground_before
                    .as_mut()
                    .unwrap()
                    .ground
                    .origin
                    .id = CommandId::new()
            }
            3 => {
                bad.applied
                    .as_mut()
                    .unwrap()
                    .ground_before
                    .as_mut()
                    .unwrap()
                    .ground_index += 1
            }
            _ => {
                bad.applied.as_mut().unwrap().equipment_before.hands.hands[0] =
                    HandAssignment::Item(f.choice.weapon)
            }
        }
        assert!(
            crate::tactical_weapons::ground::restore_after_image(&candidate, &bad, &f.pack)
                .is_err(),
            "forged inverse {case}"
        );
    }
    let clone = f.state.clone();
    assert!(
        crate::tactical_weapons::ground::prepare_after(&token, operation, &f.pack)
            .unwrap()
            .consume(&clone)
            .is_err()
    );
}

#[test]
fn private_pending_selected_and_final_states_remain_publicly_closed_under_both_policies() {
    let mut f = Fixture::new("javelin", false);
    f.begin();
    let pending = f.state.clone();
    f.raw(1);
    let selected = f.state.clone();
    let work = key(&f.state, f.selected().work.occurrence).unwrap();
    f.decision(AttackEquipmentChoice::Decline);
    let final_state = f.state.clone();
    for image in [pending, selected, final_state] {
        for version in [1, 2, 3, 4, 5, 6, u32::MAX] {
            let mut state = image.clone();
            flow_mut(&mut state).unwrap().version = version;
            assert!(has_unimplemented_ground_records(&state));
            assert!(crate::validate_state(&state, &f.pack).is_err());
            assert!(validate_tactical_state(&state).is_err());
            for policy in [ExecutionPolicy::Live, ExecutionPolicy::Historical] {
                let mut meta = f.meta(f.actor);
                meta.expected_event_sequence = state.applied_event_sequence;
                let before = state.clone();
                assert!(
                    resolve_with_policy(
                        &state,
                        &meta,
                        &TacticalAction::ChooseAttackEquipment {
                            work,
                            choice: AttackEquipmentChoice::Decline
                        },
                        &f.pack,
                        policy
                    )
                    .is_err()
                );
                assert_eq!(state, before);
            }
        }
    }
}

#[test]
fn private_graze_death_fall_keeps_finish_parent_and_never_selects_equipment_early() {
    let mut f = Fixture::new("greatsword", true);
    f.source_flyer();
    f.begin();
    f.raw(1);
    let parent = resolution(&f.state)
        .unwrap()
        .attack
        .as_ref()
        .unwrap()
        .weapon()
        .unwrap()
        .after_equipment_parent
        .clone()
        .unwrap();
    let meta = f.meta(f.actor);
    attacks::choose_mastery(&mut f.state, &meta, &WeaponMasteryChoice::Graze).unwrap();
    f.advance();
    let r = resolution(&f.state).unwrap();
    let begin = r
        .work_trace
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .find(|n| matches!(n.work.kind, TacticalWorkKind::BeginFall { .. }))
        .unwrap();
    assert_eq!(begin.parent, Some(parent.work.occurrence));
    assert_eq!(f.selected().cause.completed_work, parent.work);
    assert!(r.work_trace.as_ref().unwrap().active.is_none());
    assert!(
        f.state.rules.as_ref().unwrap().entities[&f.target]
            .death
            .dead
    );
    assert_eq!(
        f.state
            .encounter
            .as_ref()
            .unwrap()
            .participant(f.target)
            .unwrap()
            .position
            .z,
        0
    );
    f.decision(AttackEquipmentChoice::Decline);
}

#[test]
fn private_concentration_child_finishes_before_after_selection() {
    let mut f = Fixture::new("javelin", false);
    let source = f.meta(f.target);
    let group = EffectId::new();
    f.state = crate::tactical_effect_adapter::apply_effect_operation(
        &f.state,
        &source,
        &EffectLifecycleAction {
            step: 0,
            operation: EffectLifecycleOperation::BeginConcentration {
                group: ConcentrationGroup {
                    id: group,
                    source: EffectSource {
                        definition_id: "retained-source-focus".into(),
                        actor: f.target,
                        command: source.clone(),
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
    f.advance();
    f.begin();
    f.raw(20);
    f.decline_hit();
    f.raw(1);
    let r = resolution(&f.state).unwrap();
    let record = r.attack_after_equipment.as_ref().unwrap();
    assert!(record.selected_by.is_none());
    let work = &r.pending.as_ref().unwrap().work;
    assert!(
        matches!(work.kind,TacticalWorkKind::ConcentrationSave {actor,group:g,..} if actor==f.target && g==group)
    );
    assert_eq!(
        node(&f.state, key(&f.state, work.occurrence).unwrap())
            .unwrap()
            .parent,
        Some(record.cause.completed_work.occurrence)
    );
    f.raw(1);
    assert!(f.selected().selected_by.is_some());
    assert!(
        f.state.rules.as_ref().unwrap().entities[&f.target]
            .concentration
            .is_none()
    );
    f.decision(AttackEquipmentChoice::Decline);
}

#[test]
fn private_unconscious_drop_happens_before_the_completed_attack_equipment_choice() {
    let mut f = Fixture::new("javelin", false);
    let item = ItemId::new();
    let origin = f.meta(f.target);
    f.state.items.insert(
        item,
        ItemInstance {
            id: item,
            campaign_id: f.state.campaign_id(),
            definition_id: "club".into(),
            display_name: "Held physical weapon".into(),
            quantity: 1,
            owner: Ownership::Entity(f.target),
            custody: Custody::Entity(f.target),
            state: ItemState::Intact,
        },
    );
    let inventory = f
        .state
        .rules
        .as_mut()
        .unwrap()
        .tactical_inventory
        .as_mut()
        .unwrap();
    inventory.loadouts.retain(|l| l.actor != f.target);
    inventory.loadouts.push(ActorEquipmentLoadout {
        actor: f.target,
        hands: WeaponLoadout {
            hands: [HandAssignment::Item(item), HandAssignment::Free],
        },
        worn_armor: None,
        shield: None,
        command: origin,
    });
    f.state
        .rules
        .as_mut()
        .unwrap()
        .entities
        .get_mut(&f.target)
        .unwrap()
        .hp = 1;
    f.advance();
    f.begin();
    f.raw(20);
    f.decline_hit();
    f.raw(1);
    let meta = f.meta(f.actor);
    attacks::choose_knockout(&mut f.state, &meta, KnockoutChoice::KnockOut).unwrap();
    f.advance();
    assert!(f.selected().selected_by.is_some());
    assert!(matches!(f.state.items[&item].custody, Custody::Location(_)));
    assert_eq!(
        flow(&f.state)
            .unwrap()
            .ground_items
            .iter()
            .find(|g| g.item == item)
            .unwrap()
            .origin,
        meta
    );
    assert_eq!(
        f.state
            .rules
            .as_ref()
            .unwrap()
            .tactical_inventory
            .as_ref()
            .unwrap()
            .loadout(f.target)
            .unwrap()
            .hands
            .hands,
        [HandAssignment::Free; 2]
    );
    f.decision(AttackEquipmentChoice::Decline);
    assert!(matches!(f.state.items[&item].custody, Custody::Location(_)));
}

#[test]
fn private_actual_selected_allowance_can_equip_a_different_carried_weapon() {
    let mut f = Fixture::new("javelin", false);
    let other = ItemId::new();
    let mut item = f.state.items[&f.choice.weapon].clone();
    item.id = other;
    item.definition_id = "club".into();
    f.state.items.insert(other, item);
    f.begin();
    f.raw(1);
    f.decision(AttackEquipmentChoice::Apply(
        AttackEquipmentOperation::Equip {
            item: other,
            hand: Hand::Left,
        },
    ));
    assert_eq!(
        f.state
            .rules
            .as_ref()
            .unwrap()
            .tactical_inventory
            .as_ref()
            .unwrap()
            .loadout(f.actor)
            .unwrap()
            .hands
            .hands,
        [
            HandAssignment::Item(other),
            HandAssignment::Item(f.choice.weapon)
        ]
    );
    assert_eq!(flow(&f.state).unwrap().budget.attacks_remaining, 0);
}

#[test]
fn private_no_intent_preserves_absent_fields_and_old_completion_without_a_pause() {
    let mut f = Fixture::new("javelin", false);
    f.choice.after_equipment = None;
    let encoded = serde_json::to_value(&f.choice).unwrap();
    assert!(encoded.get("after_equipment").is_none());
    assert_eq!(
        serde_json::to_value(serde_json::from_value::<WeaponUseChoice>(encoded.clone()).unwrap())
            .unwrap(),
        encoded
    );
    f.begin();
    let attack = resolution(&f.state).unwrap().attack.as_ref().unwrap();
    assert!(attack.weapon().unwrap().after_equipment_parent.is_none());
    assert!(
        serde_json::to_value(attack.weapon().unwrap())
            .unwrap()
            .get("after_equipment_parent")
            .is_none()
    );
    assert!(
        serde_json::to_value(resolution(&f.state).unwrap())
            .unwrap()
            .get("attack_after_equipment")
            .is_none()
    );
    f.raw(1);
    assert!(flow(&f.state).unwrap().resolution.is_none());
    assert!(
        serde_json::to_value(&flow(&f.state).unwrap().budget.weapon_history[0])
            .unwrap()
            .get("after_equipment")
            .is_none()
    );
}

#[test]
fn private_new_intent_conflicts_and_non_normal_purposes_refuse_before_payment() {
    for case in 0..3 {
        let mut f = Fixture::new("javelin", false);
        match case {
            0 => {
                f.choice.equipment_change = Some(AttackEquipmentChange {
                    timing: EquipmentChangeTiming::BeforeAttack,
                    operation: AttackEquipmentOperation::Unequip {
                        item: f.choice.weapon,
                    },
                })
            }
            1 => {
                f.choice.equipment_change = Some(AttackEquipmentChange {
                    timing: EquipmentChangeTiming::AfterAttack,
                    operation: AttackEquipmentOperation::Unequip {
                        item: f.choice.weapon,
                    },
                })
            }
            _ => {
                f.choice.purpose = WeaponAttackPurpose::Cleave {
                    trigger: CommandId::new(),
                }
            }
        }
        let before = f.state.clone();
        let meta = f.meta(f.actor);
        assert!(attacks::begin(&mut f.state, &meta, &f.choice, &f.pack).is_err());
        assert_eq!(f.state, before);
    }
}

#[test]
fn public_after_intent_in_ordinary_source_opportunity_and_cleave_is_always_closed() {
    let f = Fixture::new("javelin", false);
    let actions = [
        TacticalAction::Attack {
            choice: f.choice.clone(),
        },
        TacticalAction::OpportunityAttack {
            choice: TacticalMeleeChoice::Weapon(f.choice.clone()),
        },
        TacticalAction::CreatureWeaponAttack {
            feature_id: "javelin-melee".into(),
            choice: CreatureWeaponUseChoice {
                after_equipment: f.choice.after_equipment,
                weapon: f.choice.weapon,
                target: f.target,
                grip: f.choice.grip,
                ammunition: None,
                equipment_change: None,
            },
        },
        TacticalAction::ChooseAttackMastery {
            choice: WeaponMasteryChoice::Cleave {
                attack: Box::new(f.choice.clone()),
            },
        },
    ];
    for version in [1, 2, 3, 4, 5, 6, u32::MAX] {
        let mut state = f.state.clone();
        flow_mut(&mut state).unwrap().version = version;
        for policy in [ExecutionPolicy::Live, ExecutionPolicy::Historical] {
            for action in &actions {
                assert!(action_uses_ground_pickup(action));
                let before = state.clone();
                assert!(
                    resolve_with_policy(&state, &f.meta(f.actor), action, &f.pack, policy).is_err()
                );
                assert_eq!(state, before);
            }
        }
    }
}

#[test]
fn private_selected_equipment_cannot_be_raw_dice_simultaneous_work_or_selected_twice() {
    let mut f = Fixture::new("javelin", false);
    f.begin();
    f.raw(1);
    let work = f.selected().work.clone();
    let meta = f.meta(f.actor);
    let before = f.state.clone();
    assert!(continuations::key(&f.state, &work).is_err());
    assert!(!work_trace::tactical_frame_host_ordering(resolution(&f.state).unwrap()).unwrap());
    assert!(turns::choose(&mut f.state, &meta, work.occurrence).is_err());
    assert_eq!(f.state, before);
    assert!(continuations::start(&mut f.state, &meta, work).is_err());
    assert_eq!(f.state, before);
}

#[test]
fn private_selected_cause_rejects_changed_outcome_choice_parent_and_resolution() {
    let mut f = Fixture::new("javelin", false);
    f.begin();
    f.raw(1);
    let good = f.state.clone();
    let work = key(&f.state, f.selected().work.occurrence).unwrap();
    for field in 0..4 {
        f.state = good.clone();
        let record = resolution_mut(&mut f.state)
            .unwrap()
            .attack_after_equipment
            .as_mut()
            .unwrap();
        match field {
            0 => {
                record.cause.outcome = WeaponAttackOutcome::Hit {
                    critical: false,
                    damage_dealt: 1,
                }
            }
            1 => {
                record.cause.choice.equipment_change = Some(AttackEquipmentChange {
                    timing: EquipmentChangeTiming::BeforeAttack,
                    operation: AttackEquipmentOperation::Unequip {
                        item: f.choice.weapon,
                    },
                })
            }
            2 => record.cause.completed_work.occurrence = 0,
            _ => record.cause.completed_work.resolution = CommandId::new(),
        }
        f.reject(f.meta(f.actor), work, AttackEquipmentChoice::Decline);
    }
}

#[test]
fn private_graze_rejects_same_kind_retired_sibling_without_entering_or_mutating_it() {
    let mut f = Fixture::new("greatsword", true);
    f.begin();
    f.raw(1);
    let r = resolution_mut(&mut f.state).unwrap();
    let actual = r
        .attack
        .as_ref()
        .unwrap()
        .weapon()
        .unwrap()
        .after_equipment_parent
        .as_ref()
        .unwrap()
        .work;
    let trace = r.work_trace.as_mut().unwrap();
    let mut sibling = trace
        .nodes
        .iter()
        .find(|n| n.work.occurrence == actual.occurrence)
        .unwrap()
        .clone();
    assert_eq!(sibling.work.kind, TacticalWorkKind::FinishAttack);
    sibling.work.occurrence = r.next_occurrence;
    r.next_occurrence += 1;
    let counterfeit = TacticalWorkKey {
        resolution: actual.resolution,
        occurrence: sibling.work.occurrence,
    };
    trace.nodes.push(sibling);
    r.attack
        .as_mut()
        .unwrap()
        .weapon_mut()
        .unwrap()
        .after_equipment_parent
        .as_mut()
        .unwrap()
        .work = counterfeit;
    // This is a same-kind, same-ancestor, allocation-ordered DAG counterfeit,
    // not the existing wrong-kind or malformed-trace negative.
    work_trace::validate(&f.state).unwrap();
    let before = f.state.clone();
    let meta = f.meta(f.actor);
    assert!(validate_pause(&f.state).is_err());
    assert!(attacks::choose_mastery(&mut f.state, &meta, &WeaponMasteryChoice::Graze).is_err());
    assert_eq!(f.state, before);
    assert!(
        resolution(&f.state)
            .unwrap()
            .work_trace
            .as_ref()
            .unwrap()
            .active
            .is_none()
    );
}

#[test]
fn private_after_trace_inventory_rejects_retired_extra_and_orphan_nodes_atomically() {
    let mut f = Fixture::new("javelin", false);
    f.begin();
    f.raw(1);
    let work = key(&f.state, f.selected().work.occurrence).unwrap();
    let good = f.state.clone();
    for orphan in [false, true] {
        f.state = good.clone();
        let r = resolution_mut(&mut f.state).unwrap();
        if orphan {
            r.attack_after_equipment = None;
        } else {
            let trace = r.work_trace.as_mut().unwrap();
            let mut extra = trace
                .nodes
                .iter()
                .find(|n| n.work.occurrence == work.occurrence)
                .unwrap()
                .clone();
            extra.work.occurrence = r.next_occurrence;
            r.next_occurrence += 1;
            trace.nodes.push(extra);
        }
        work_trace::validate(&f.state).unwrap();
        assert!(validate(&f.state).is_err());
        f.reject(f.meta(f.actor), work, AttackEquipmentChoice::Decline);
    }
}

#[test]
fn private_printed_physical_source_propagates_intent_and_retains_exact_pin_and_feature() {
    use crate::tactical_creature_equipment::{
        creature_equipment_plan, materialize_creature_equipment,
    };
    let mut f = Fixture::new("javelin", false);
    let old = f.actor;
    let actor = EntityId::new();
    let mut world = f.state.entities[&old].clone();
    world.id = actor;
    world.kind = EntityKind::Creature;
    f.state.entities.insert(actor, world);
    let mut origin = f.meta(actor);
    origin.issuer = CommandIssuer::Admin;
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
    let pin = built.profile.source.clone();
    let rules = f.state.rules.as_mut().unwrap();
    rules.entities.insert(actor, built.mechanics);
    let creatures = rules.tactical_creatures.get_or_insert_default();
    creatures.profiles.push(built.profile);
    creatures.runtime.push(built.runtime);
    for entry in &mut rules.timing.as_mut().unwrap().order {
        if entry.actor == old {
            entry.actor = actor;
        }
    }
    rules
        .tactical_effects
        .as_mut()
        .unwrap()
        .turn
        .as_mut()
        .unwrap()
        .actor = actor;
    let ids = creature_equipment_plan("goblin-warrior", 20)
        .unwrap()
        .iter()
        .map(|_| ItemId::new())
        .collect::<Vec<_>>();
    f.state = materialize_creature_equipment(&f.state, &origin, actor, 20, &ids, &f.pack).unwrap();
    let arrows = f
        .state
        .items
        .values()
        .filter(|i| i.custody == Custody::Entity(actor) && i.definition_id == "arrows")
        .collect::<Vec<_>>();
    assert_eq!(arrows.len(), 1);
    let arrows_before = arrows[0].clone();
    assert_eq!(arrows_before.quantity, 20);
    let weapon = f
        .state
        .items
        .values()
        .find(|i| i.custody == Custody::Entity(actor) && i.definition_id == "scimitar")
        .unwrap()
        .id;
    let loadout = f
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
    assert_eq!(
        loadout.hands.hands[Hand::Right.index()],
        HandAssignment::Free
    );
    loadout.hands.hands[Hand::Right.index()] = HandAssignment::Item(weapon);
    let e = f.state.encounter.as_mut().unwrap();
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
    e.participants
        .iter_mut()
        .find(|p| p.entity_id == f.target)
        .unwrap()
        .enemies = vec![actor];
    let c = e
        .flow
        .as_mut()
        .unwrap()
        .combatants
        .iter_mut()
        .find(|c| c.actor == old)
        .unwrap();
    c.actor = actor;
    c.source = TacticalSource::Creature {
        definition_id: "goblin-warrior".into(),
    };
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
    f.choice.weapon = weapon;
    f.advance();
    let observed = crate::tactical_creatures::observe_creature_turn(
        &f.state,
        f.state
            .rules
            .as_ref()
            .unwrap()
            .tactical_creatures
            .as_ref()
            .unwrap(),
        &f.meta(actor),
        actor,
        CreatureTurn {
            encounter_id: f.state.encounter.as_ref().unwrap().id,
            actor,
            number: f
                .state
                .rules
                .as_ref()
                .unwrap()
                .timing
                .as_ref()
                .unwrap()
                .turn_number,
            boundary: TurnBoundary::Start,
        },
        vec![],
    )
    .unwrap();
    f.state.rules.as_mut().unwrap().tactical_creatures = Some(observed.next);
    f.advance();
    let choice = CreatureWeaponUseChoice {
        after_equipment: Some(AfterAttackEquipmentIntent::Choose),
        weapon,
        target: f.target,
        grip: WeaponGrip::OneHand(Hand::Right),
        ammunition: None,
        equipment_change: None,
    };
    let meta = f.meta(actor);
    attacks::begin_creature_weapon(&mut f.state, &meta, "scimitar", &choice, &f.pack).unwrap();
    f.advance();
    assert_eq!(
        resolution(&f.state)
            .unwrap()
            .attack
            .as_ref()
            .unwrap()
            .weapon()
            .unwrap()
            .choice
            .after_equipment,
        choice.after_equipment
    );
    f.raw(1);
    assert_eq!(
        f.selected().cause.source,
        AttackEquipmentSource::Creature {
            source: pin,
            feature_id: "scimitar".into()
        }
    );
    assert_eq!(f.selected().cause.origin, meta);
    f.decision(AttackEquipmentChoice::Decline);
    assert_eq!(f.state.items[&arrows_before.id], arrows_before);
}
