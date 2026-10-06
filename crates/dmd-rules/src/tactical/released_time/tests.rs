//! Actual rule producers over an authored world fixture. These controls do not
//! replace original accepted journal, file-SQLite, portable or native evidence.
use super::*;
use crate::tactical_creature_equipment::*;
use crate::tactical_creatures::*;
use crate::tactical_damage::VitalityFollowup;
use crate::tactical_inventory::*;
use crate::*;
use std::collections::HashMap;
struct Fixture {
    state: CampaignState,
    pack: RulesPack,
    actors: [EntityId; 3],
    players: [PlayerId; 3],
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
                    position: point(10 + 10 * i as i32, 10, 0),
                    size: CreatureSize::Medium,
                    public_label: "Adventurer".into(),
                    height: 12,
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
        let actor = actor.filter(|index| {
            self.state
                .characters
                .values()
                .any(|c| c.entity_id == self.actors[*index])
        });
        CommandMeta {
            id: CommandId::new(),
            campaign_id: self.state.campaign_id(),
            session_id: self
                .state
                .table
                .as_ref()
                .and_then(|t| t.active_session.as_ref())
                .map(|s| s.session_id),
            issuer: actor.map_or(CommandIssuer::Admin, |i| {
                CommandIssuer::Player(self.players[i])
            }),
            actor: actor.map(|i| AgentRef::Entity(self.actors[i])),
            expected_event_sequence: self.state.applied_event_sequence,
        }
    }
    fn run(&mut self, actor: Option<usize>, action: TacticalAction) -> TacticalEvent {
        let meta = self.meta(actor);
        // Preserve the original flow-5 Begin of these isolated controls. Every
        // continuation and explicit upgrade uses the normal public producer.
        let historical = matches!(
            action,
            TacticalAction::Begin {
                execution: TacticalExecutionVersion::EncounterReleaseV1,
                ..
            }
        );
        let result = if historical {
            super::super::resolve_with_policy(
                &self.state,
                &meta,
                &action,
                &self.pack,
                super::super::ExecutionPolicy::Historical,
            )
        } else {
            resolve_tactical(&self.state, &meta, &action, &self.pack)
        }
        .unwrap();
        assert_eq!(
            replay_tactical(&self.state, &result.event, &self.pack).unwrap(),
            result
        );
        self.state = result.next_state;
        self.state.applied_event_sequence += 1;
        result.event
    }
    fn guarded(&mut self, action: TacticalAction) -> CommandMeta {
        let meta = self.meta(None);
        let next = transition(&self.state, &meta, &action, &self.pack).unwrap();
        assert_eq!(
            transition(&self.state, &meta, &action, &self.pack).unwrap(),
            next
        );
        self.state = serde_json::from_str(&next.encode_json().unwrap()).unwrap();
        self.state.applied_event_sequence += 1;
        validate(&self.state, &self.pack).unwrap();
        meta
    }
    fn human(&mut self) {
        let actor = self.actors[0];
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
            masteries: ["club".into(), "dagger".into(), "shortbow".into()],
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
        table.active_session = Some(ActiveTableSession {
            session_id: PlaySessionId::new(),
            display_name: "Offstage producer fixture".into(),
            started_at_world: self.state.clock.now,
            participants: self
                .players
                .iter()
                .map(|player_id| SessionParticipant {
                    player_id: *player_id,
                    character_id: None,
                    attendance: AttendanceStatus::Present,
                })
                .collect(),
        });
        self.state.table = Some(table);
        let built = materialize_starting_equipment(
            &self.state,
            &TacticalInventory::default(),
            &self.meta(Some(0)),
            character,
            &[],
            &self.pack,
        )
        .unwrap();
        self.state = built.next_state;
        self.state.rules.as_mut().unwrap().tactical_inventory = Some(built.next_inventory);
        self.state.applied_event_sequence += 1;
    }
    fn mage(&mut self, index: usize) {
        let actor = self.actors[index];
        self.state.characters.retain(|_, c| c.entity_id != actor);
        self.state.entities.get_mut(&actor).unwrap().kind = EntityKind::Creature;
        self.state.rules.as_mut().unwrap().entities.remove(&actor);
        let meta = self.meta(None);
        let built = build_creature(
            &self.state,
            &meta,
            actor,
            &CreatureBuildChoice {
                definition_id: "mage".into(),
                size: CreatureSize::Medium,
                additional_languages: vec!["Dwarvish".into(), "Elvish".into(), "Gnomish".into()],
                hit_points: CreatureHitPointChoice::Average,
                controller: CreatureController::Host,
                in_lair: false,
            },
        )
        .unwrap();
        let participant = &mut self.state.encounter.as_mut().unwrap().participants[index];
        participant.movement = built.movement;
        participant.senses = built.senses;
        let rules = self.state.rules.as_mut().unwrap();
        rules.entities.insert(actor, built.mechanics);
        let sources = rules
            .tactical_creatures
            .get_or_insert_with(TacticalCreatures::default);
        sources.profiles.push(built.profile);
        sources.runtime.push(built.runtime);
        let ids = creature_equipment_plan("mage", 0)
            .unwrap()
            .iter()
            .map(|_| ItemId::new())
            .collect::<Vec<_>>();
        self.state =
            materialize_creature_equipment(&self.state, &meta, actor, 0, &ids, &self.pack).unwrap();
        self.state.applied_event_sequence += 1;
    }
    fn begin(&mut self) {
        let human = self
            .state
            .characters
            .values()
            .any(|c| c.entity_id == self.actors[0]);
        let mages = if human {
            self.actors[1..].to_vec()
        } else {
            self.actors.to_vec()
        };
        let mut groups = Vec::new();
        if human {
            groups.push(InitiativeGroup {
                actors: vec![self.actors[0]],
                request_id: RollRequestId::new(),
            });
        }
        groups.push(InitiativeGroup {
            actors: mages.clone(),
            request_id: RollRequestId::new(),
        });
        self.run(
            None,
            TacticalAction::Begin {
                execution: TacticalExecutionVersion::EncounterReleaseV1,
                combatants: self
                    .actors
                    .iter()
                    .map(|actor| TacticalCombatant {
                        actor: *actor,
                        source: self
                            .state
                            .rules
                            .as_ref()
                            .unwrap()
                            .tactical_creatures
                            .as_ref()
                            .and_then(|c| c.profile(*actor))
                            .map_or(TacticalSource::Character, |p| TacticalSource::Creature {
                                definition_id: p.source.definition_id.clone(),
                            }),
                        surprised: false,
                    })
                    .collect(),
                groups,
            },
        );
        if human {
            self.roll(0, 18);
        }
        self.roll(usize::from(human), 3);
        self.run(None, TacticalAction::ProposeInitiativeTie { order: mages });
    }
    fn roll(&mut self, index: usize, value: u16) {
        let request = &self
            .state
            .rules
            .as_ref()
            .unwrap()
            .pending
            .as_ref()
            .unwrap()
            .request;
        let result = RollResult {
            request_id: request.id,
            source: RollSource::Physical,
            dice: request
                .dice
                .iter()
                .flat_map(|spec| {
                    (0..spec.count).map(move |_| DieResult {
                        sides: spec.sides,
                        value,
                    })
                })
                .collect(),
        };
        self.run(Some(index), TacticalAction::SubmitRoll { result });
    }
    fn end(&mut self, index: usize) {
        self.run(Some(index), TacticalAction::EndTurn);
    }
    fn armor(&mut self, index: usize) -> TacticalEvent {
        let actor = self.actors[index];
        let item = self
            .state
            .items
            .values()
            .find(|item| {
                item.definition_id == "spell-material:mage-armor"
                    && item.custody == Custody::Entity(actor)
            })
            .unwrap()
            .id;
        self.run(
            Some(index),
            TacticalAction::CastSpell {
                choice: SpellCastChoice {
                    actor,
                    spell_id: "mage-armor".into(),
                    grant: SpellGrantChoice::CreatureFeature {
                        feature_id: "spellcasting".into(),
                    },
                    resource: SpellResourceChoice::SourceFeature,
                    material: SpellMaterialChoice::Material { item },
                    mode: SpellCastMode::Immediate,
                },
                targets: SpellTargetChoice::Entities(vec![actor]),
            },
        )
    }
    fn release(&mut self) {
        self.run(
            None,
            TacticalAction::ConcludeHostilities {
                cadence: AftermathCadence::ContinueExistingOrder,
                ruling: "Hostilities ended; source timing remains.".into(),
            },
        );
        self.run(None, TacticalAction::FinishEncounter);
        self.guarded(TacticalAction::UpgradeExecutionTo {
            execution: TacticalExecutionVersion::ReleasedTimeV1,
        });
    }
    fn two_mages() -> Self {
        let mut f = Self::new();
        f.human();
        f.mage(1);
        f.mage(2);
        f.begin();
        f.end(0);
        f.armor(1);
        f.end(1);
        f.armor(2);
        f.release();
        f
    }
}
fn advance(seconds: u32) -> TacticalAction {
    TacticalAction::AdvanceReleasedTime {
        seconds,
        ordering: ReleasedTimeOrdering::HostSelect,
        ruling: "Wait for the retained absolute deadlines.".into(),
    }
}

#[test]
fn public_released_commands_replay_exact_events_and_reject_changed_outcomes() {
    let mut f = Fixture::new();
    f.human();
    f.mage(1);
    f.mage(2);
    f.begin(); // Original isolated5 prefix; current commands explicitly upgrade.
    f.run(
        None,
        TacticalAction::UpgradeExecutionTo {
            execution: TacticalExecutionVersion::ReleasedTimeV1,
        },
    );
    f.end(0);
    f.armor(1);
    f.end(1);
    f.armor(2);
    f.run(
        None,
        TacticalAction::ConcludeHostilities {
            cadence: AftermathCadence::ContinueExistingOrder,
            ruling: "Retain real source effects after the last turn.".into(),
        },
    );
    f.run(None, TacticalAction::FinishEncounter);
    let before = f.state.clone();
    let event = f.run(None, advance(28_860));
    assert!(event.outcome.awaiting_turn_work);
    assert!(event.outcome.active_actor.is_none());
    crate::validate_state(&f.state, &f.pack).unwrap();
    validate_tactical_state(&f.state).unwrap();
    let mut forged = event.clone();
    forged.outcome.awaiting_turn_work = false;
    assert!(replay_tactical(&before, &forged, &f.pack).is_err());
    let occurrence = resolution(&f.state).unwrap().frames.last().unwrap()[1].occurrence;
    let paused = f.state.clone();
    let completed = f.run(None, TacticalAction::ChooseTurnWork { occurrence });
    assert!(!completed.outcome.awaiting_turn_work);
    assert!(completed.outcome.active_actor.is_none());
    assert_eq!(f.state.clock.now, WorldInstant(28_860));
    assert_eq!(
        f.state
            .encounter_history
            .as_ref()
            .unwrap()
            .elapsed_intervals
            .last()
            .unwrap()
            .origin,
        event.meta
    );
    let mut forged = completed;
    forged.outcome.active_actor = Some(f.actors[0]);
    assert!(replay_tactical(&paused, &forged, &f.pack).is_err());
}

#[test]
fn real_mage_equal_deadlines_pause_then_resume_original_target_in_either_order() {
    for select in [0, 1] {
        let mut f = Fixture::two_mages();
        let release = f
            .state
            .encounter_history
            .as_ref()
            .unwrap()
            .last()
            .unwrap()
            .clone();
        let before = f.state.clone();
        let origin = f.guarded(advance(28_860));
        assert_eq!(f.state.clock.now, WorldInstant(28_800));
        let r = resolution(&f.state).unwrap();
        assert!(r.turn_context().is_err());
        assert_eq!(r.frames[0].len(), 2);
        assert_eq!(
            r.released_interval().unwrap().target_at,
            WorldInstant(28_860)
        );
        assert!(r.released_interval().unwrap().batches[0].bindings.iter().all(|b| matches!(&b.source, ReleasedDeadlineSource::Effect { source, .. } if source.definition_id == "mage-armor")));
        let work = r.frames[0][select].occurrence;
        let selected = f.guarded(TacticalAction::ChooseTurnWork { occurrence: work });
        assert!(flow(&f.state).unwrap().resolution.is_none());
        assert_eq!(f.state.clock.now, WorldInstant(28_860));
        assert!(effects(&f.state).unwrap().effects.is_empty());
        assert_eq!(
            f.state.rules.as_ref().unwrap().rolls,
            before.rules.as_ref().unwrap().rolls
        );
        assert_eq!(f.state.items, before.items);
        assert_eq!(
            f.state.rules.as_ref().unwrap().tactical_creatures,
            before.rules.as_ref().unwrap().tactical_creatures
        );
        let history = f.state.encounter_history.as_ref().unwrap();
        assert_eq!(history.last(), Some(&release));
        assert_eq!(history.elapsed_intervals[0].origin, origin);
        assert_eq!(history.elapsed_intervals[0].completed_by, selected);
        assert_eq!(
            history.elapsed_intervals[0].completed_at,
            WorldInstant(28_860)
        );
    }
}

#[test]
fn guarded_interval_rejects_public_authority_and_candidate_splicing_atomically() {
    let mut f = Fixture::two_mages();
    f.guarded(advance(28_860));
    let original = f.state.clone();
    crate::validate_state(&original, &f.pack).unwrap();
    validate_tactical_state(&original).unwrap();
    assert!(resolve_tactical(&original, &f.meta(None), &advance(1), &f.pack).is_err());
    let proof = ReleasedValidation::derive(&original).unwrap();
    let cloned = original.clone();
    assert!(!proof.binds(&cloned));
    assert!(crate::kernel::validate_released_state(&cloned, &f.pack, &proof).is_err());
    for mutation in 0..8 {
        let mut forged = original.clone();
        let r = resolution_mut(&mut forged).unwrap();
        match mutation {
            0 => r.frames[0].pop().map(|_| ()).unwrap(),
            1 => {
                let duplicate = r.frames[0][0].clone();
                r.frames[0].push(duplicate);
            }
            2 => r.frames[0][0].occurrence += 100,
            3 => r.released_interval_mut().unwrap().target_at = WorldInstant(1),
            4 => r.released_interval_mut().unwrap().release = CommandId::new(),
            5 => r.released_interval_mut().unwrap().batches[0].observed.step += 1,
            6 => r.work_trace.as_mut().unwrap().active = Some(0),
            7 => r.frames[0][0].kind = TacticalWorkKind::DeathSave { actor: f.actors[0] },
            _ => unreachable!(),
        }
        assert!(validate(&forged, &f.pack).is_err(), "mutation {mutation}");
    }
    let occurrence = resolution(&f.state).unwrap().frames[0][0].occurrence;
    for (case, mut meta) in [f.meta(Some(0)), f.meta(None), f.meta(None)]
        .into_iter()
        .enumerate()
    {
        if case == 1 {
            meta.session_id = None;
        }
        if case == 2 {
            meta.session_id = Some(PlaySessionId::new());
        }
        assert!(
            transition(
                &f.state,
                &meta,
                &TacticalAction::ChooseTurnWork { occurrence },
                &f.pack
            )
            .is_err()
        );
    }
    assert_eq!(f.state, original);
}

#[test]
fn three_real_mages_keep_two_siblings_after_one_choice_without_reobserving_time() {
    let mut f = Fixture::new();
    f.state.campaign.content_packs = TableContract::default().permitted_content.clone();
    let mut table = TableState::new(TableContract::default());
    table.active_session = Some(ActiveTableSession {
        session_id: PlaySessionId::new(),
        display_name: "Three Mage fixture".into(),
        started_at_world: WorldInstant(0),
        participants: f
            .players
            .iter()
            .map(|p| SessionParticipant {
                player_id: *p,
                character_id: None,
                attendance: AttendanceStatus::Present,
            })
            .collect(),
    });
    f.state.table = Some(table);
    for index in 0..3 {
        f.mage(index);
    }
    f.begin();
    f.armor(0);
    f.end(0);
    f.armor(1);
    f.end(1);
    f.armor(2);
    f.release();
    let first = f.guarded(advance(28_860));
    let r = resolution(&f.state).unwrap();
    let observed = r.released_interval().unwrap().batches[0].observed.clone();
    assert_eq!(observed.command, first);
    let occurrence = r.frames[0][0].occurrence;
    let selected = f.guarded(TacticalAction::ChooseTurnWork { occurrence });
    let r = resolution(&f.state).unwrap();
    assert_eq!(r.frames[0].len(), 2);
    let batch = &r.released_interval().unwrap().batches[0];
    assert_eq!(batch.observed, observed);
    assert_eq!(batch.completions.len(), 1);
    assert_eq!(batch.completions[0].completed_by, selected);
    assert_eq!(r.next_occurrence, 3);
    assert_eq!(f.state.clock.now, WorldInstant(28_800));
    let occurrence = r.frames[0][1].occurrence;
    f.guarded(TacticalAction::ChooseTurnWork { occurrence });
    assert_eq!(f.state.clock.now, WorldInstant(28_860));
    assert!(flow(&f.state).unwrap().resolution.is_none());
}

#[test]
fn real_later_round_recast_reveals_old_deadline_then_keeps_new_defense_until_its_own_expiry() {
    let mut f = Fixture::new();
    f.human();
    f.mage(1);
    f.mage(2);
    f.begin();
    f.end(0);
    f.armor(1);
    f.end(1);
    f.end(2);
    f.end(0);
    assert_eq!(f.state.clock.now, WorldInstant(6));
    f.armor(1);
    f.release();
    let actor = f.actors[1];
    assert_eq!(effects(&f.state).unwrap().effects.len(), 2);
    f.guarded(advance(28_794));
    assert_eq!(f.state.clock.now, WorldInstant(28_800));
    assert_eq!(effects(&f.state).unwrap().effects.len(), 1);
    assert_eq!(
        crate::tactical_defenses::effective_armor_class(&f.state, actor).unwrap(),
        15
    );
    let first = f
        .state
        .encounter_history
        .as_ref()
        .unwrap()
        .elapsed_intervals[0]
        .origin
        .id;
    f.guarded(advance(6));
    assert!(effects(&f.state).unwrap().effects.is_empty());
    assert_eq!(
        crate::tactical_defenses::effective_armor_class(&f.state, actor).unwrap(),
        12
    );
    assert_eq!(
        f.state
            .encounter_history
            .as_ref()
            .unwrap()
            .elapsed_intervals[1]
            .predecessor,
        Some(first)
    );
}

#[test]
fn real_damage_medicine_and_accepted_d4_wake_once_after_release() {
    let mut f = Fixture::new();
    f.human();
    f.mage(1);
    f.mage(2);
    f.begin();
    f.end(0);
    let target = f.actors[0];
    let meta = f.meta(None);
    let hp = f.state.rules.as_ref().unwrap().entities[&target].hp;
    // Invoke the actual private damage consequence over real built mechanics;
    // this is not an accepted attack capture and does not discharge that proof.
    let damage = crate::tactical_vitality_adapter::apply(
        &mut f.state,
        target,
        VitalityOrigin {
            command: meta.clone(),
            occurrence: 0,
        },
        &VitalityOperation::Damage {
            packet: DamagePacket {
                cause: DamageCause::Other,
                components: vec![DamageComponent {
                    damage_type: DamageType::Bludgeoning,
                    amounts: vec![hp],
                    adjustments: vec![],
                }],
            },
            knockout: None,
        },
    )
    .unwrap();
    for followup in damage.followups {
        match followup {
            VitalityFollowup::DropHeldItems => {
                crate::tactical_vitality_adapter::drop_held(&mut f.state, target, &meta).unwrap()
            }
            VitalityFollowup::InterruptRest => crate::kernel::interrupt_rest(
                f.state.rules.as_mut().unwrap(),
                target,
                f.state.clock.now,
            ),
            _ => panic!("unexpected damage followup"),
        }
    }
    f.state.applied_event_sequence += 1;
    f.run(
        Some(1),
        TacticalAction::FirstAid {
            target,
            purpose: MedicinePurpose::Stabilize,
        },
    );
    f.roll(1, 20);
    assert!(
        matches!(f.state.rules.as_ref().unwrap().pending.as_ref().unwrap().purpose, PendingPurpose::TacticalResolution { key: TacticalRollKey { role: TacticalRollRole::StableRecovery, subject, .. }, .. } if subject == target)
    );
    f.roll(0, 2);
    let stable = f
        .state
        .rules
        .as_ref()
        .unwrap()
        .tactical_recovery
        .as_ref()
        .unwrap()[&target]
        .stable
        .clone()
        .unwrap();
    let rolls = f.state.rules.as_ref().unwrap().rolls.clone();
    f.release();
    f.guarded(advance(7_199));
    assert_eq!(f.state.rules.as_ref().unwrap().entities[&target].hp, 0);
    f.guarded(advance(1));
    assert_eq!(f.state.rules.as_ref().unwrap().entities[&target].hp, 1);
    assert!(
        !crate::active_conditions(f.state.rules.as_ref().unwrap(), target)
            .contains(&Condition::Unconscious)
    );
    assert_eq!(f.state.rules.as_ref().unwrap().rolls, rolls);
    assert!(
        f.state
            .rules
            .as_ref()
            .unwrap()
            .tactical_recovery
            .as_ref()
            .unwrap()
            .get(&target)
            .is_none_or(|r| r.stable.is_none())
    );
    assert_eq!(
        stable.delay_roll.unwrap().dice,
        vec![DieResult { sides: 4, value: 2 }]
    );
    f.guarded(advance(1));
    assert_eq!(f.state.rules.as_ref().unwrap().entities[&target].hp, 1);
}

#[test]
fn interval_refuses_bad_bounds_ruling_wrong_executor_and_unsupported_retained_conditions() {
    let f = Fixture::two_mages();
    for action in [
        advance(0),
        TacticalAction::AdvanceReleasedTime {
            seconds: 1,
            ordering: ReleasedTimeOrdering::HostSelect,
            ruling: "bad\nline".into(),
        },
        TacticalAction::UpgradeExecutionTo {
            execution: TacticalExecutionVersion::ReleasedTimeV1,
        },
    ] {
        let before = f.state.clone();
        assert!(transition(&f.state, &f.meta(None), &action, &f.pack).is_err());
        assert_eq!(f.state, before);
    }
    for version in [1, 2, 3, 4, 6, 8] {
        let mut forged = f.state.clone();
        flow_mut(&mut forged).unwrap().version = version;
        assert!(transition(&forged, &f.meta(None), &advance(1), &f.pack).is_err());
    }
    let mut condition = f.state.clone();
    condition
        .rules
        .as_mut()
        .unwrap()
        .tactical_effects
        .as_mut()
        .unwrap()
        .effects[0]
        .conditions
        .push(EffectCondition {
            id: EffectId::new(),
            condition: Condition::Invisible,
        });
    assert!(transition(&condition, &f.meta(None), &advance(1), &f.pack).is_err());
    let mut rest = f.state.clone();
    rest.rules.as_mut().unwrap().rests.push(RestProgress {
        actor: f.actors[0],
        kind: RestKind::Short,
        started_at: rest.clock.now,
    });
    assert!(transition(&rest, &f.meta(None), &advance(1), &f.pack).is_err());
}

#[test]
fn isolated_group_fixture_records_exact_cleanup_cancellations_before_pruning() {
    for group_first in [true, false] {
        let mut f = Fixture::new();
        f.human();
        f.mage(1);
        f.mage(2);
        f.begin();
        f.end(0);
        f.armor(1);
        f.end(1);
        f.armor(2);
        let meta = f.meta(None);
        let group = EffectId::new();
        let source = EffectSource {
            definition_id: "isolated-condition-free-group-fixture".into(),
            actor: f.actors[1],
            command: meta.clone(),
            ordinal: 0,
        };
        let members = [EffectId::new(), EffectId::new()];
        for (step, operation) in [
            EffectLifecycleOperation::BeginConcentration {
                group: ConcentrationGroup {
                    id: group,
                    source: source.clone(),
                    expires: TacticalEffectExpiry::AtTime(WorldInstant(28_800)),
                    stage: ConcentrationStage::Casting,
                },
            },
            EffectLifecycleOperation::Install {
                effects: members
                    .iter()
                    .enumerate()
                    .map(|(index, id)| TacticalEffect {
                        id: *id,
                        source: source.clone(),
                        established_at: None,
                        target: TacticalEffectTarget::Creature(f.actors[index + 1]),
                        concentration_group: Some(group),
                        expires: TacticalEffectExpiry::AtTime(WorldInstant(28_800)),
                        overlap: None,
                        conditions: vec![],
                        defenses: vec![],
                        triggers: vec![],
                    })
                    .collect(),
            },
        ]
        .into_iter()
        .enumerate()
        {
            f.state = crate::tactical_effect_adapter::apply_effect_operation(
                &f.state,
                &meta,
                &EffectLifecycleAction {
                    step: step as u16,
                    operation,
                },
            )
            .unwrap()
            .0;
        }
        f.state.applied_event_sequence += 1;
        f.release();
        f.guarded(advance(28_860));
        assert_eq!(resolution(&f.state).unwrap().frames[0].len(), 5);
        let selected_ids = if group_first {
            vec![group]
        } else {
            members.to_vec()
        };
        for id in selected_ids {
            let ticket = effects(&f.state)
                .unwrap()
                .pending
                .iter()
                .find(|ticket| ticket.effect == id)
                .unwrap()
                .id;
            let occurrence = resolution(&f.state).unwrap().frames[0]
                .iter()
                .find(|work| work.kind == TacticalWorkKind::Effect { ticket })
                .unwrap()
                .occurrence;
            f.guarded(TacticalAction::ChooseTurnWork { occurrence });
        }
        let r = resolution(&f.state).unwrap();
        assert_eq!(r.frames[0].len(), 2);
        let batch = &r.released_interval().unwrap().batches[0];
        assert_eq!(batch.completions.len(), 3);
        assert_eq!(
            batch
                .completions
                .iter()
                .filter(|c| matches!(c.outcome, ReleasedWorkOutcome::CancelledBy { .. }))
                .count(),
            if group_first { 2 } else { 1 }
        );
        for completion in &batch.completions {
            if let ReleasedWorkOutcome::CancelledBy { work } = completion.outcome {
                assert!(batch.completions.iter().any(|cause| cause.work == work
                    && cause.outcome == ReleasedWorkOutcome::Applied
                    && cause.completed_by == completion.completed_by));
            }
        }
        assert_eq!(
            f.state.rules.as_ref().unwrap().entities[&f.actors[1]].concentration,
            None
        );
        let occurrence = r.frames[0][0].occurrence;
        f.guarded(TacticalAction::ChooseTurnWork { occurrence });
        assert_eq!(f.state.clock.now, WorldInstant(28_860));
    }
}

#[test]
fn isolated_legacy_absolute_deadline_uses_shared_work_and_exact_pointer_cleanup() {
    let mut f = Fixture::new();
    f.human();
    f.mage(1);
    f.mage(2);
    f.begin();
    let id = EffectId::new();
    let actor = f.actors[1];
    // Existing ActiveEffect has no installation-command record. This explicitly
    // isolated legacy fixture is not represented as accepted historical input.
    f.state.rules.as_mut().unwrap().effects.push(ActiveEffect {
        id,
        source: actor,
        target: actor,
        condition: None,
        label: "Isolated legacy absolute deadline".into(),
        expires: Expiry::AtTime(WorldInstant(10)),
        concentration_owner: Some(actor),
    });
    f.state
        .rules
        .as_mut()
        .unwrap()
        .entities
        .get_mut(&actor)
        .unwrap()
        .concentration = Some(id);
    f.release();
    let before = f.state.clone();
    f.guarded(advance(11));
    assert_eq!(f.state.clock.now, WorldInstant(11));
    assert!(f.state.rules.as_ref().unwrap().effects.is_empty());
    assert_eq!(
        f.state.rules.as_ref().unwrap().entities[&actor].concentration,
        None
    );
    assert_eq!(
        f.state.rules.as_ref().unwrap().rolls,
        before.rules.as_ref().unwrap().rolls
    );
}

#[test]
fn explicit_active_five_to_seven_upgrade_produces_new_seven_release_without_rewriting_old_history()
{
    let mut f = Fixture::new();
    f.human();
    f.mage(1);
    f.mage(2);
    f.begin();
    f.guarded(TacticalAction::UpgradeExecutionTo {
        execution: TacticalExecutionVersion::ReleasedTimeV1,
    });
    assert!(flow(&f.state).unwrap().released_time_upgrade.is_none());
    f.guarded(TacticalAction::ConcludeHostilities {
        cadence: AftermathCadence::ContinueExistingOrder,
        ruling: "Hostilities ended at the real settled boundary.".into(),
    });
    f.guarded(TacticalAction::FinishEncounter);
    assert_eq!(
        f.state
            .encounter_history
            .as_ref()
            .unwrap()
            .last()
            .unwrap()
            .execution,
        TacticalExecutionVersion::ReleasedTimeV1
    );
    f.guarded(advance(60));
    assert_eq!(f.state.clock.now, WorldInstant(60));
    assert!(flow(&f.state).unwrap().resolution.is_none());
}

#[test]
fn suppressed_condition_revelation_is_refused_before_any_elapsed_mutation() {
    let mut f = Fixture::new();
    f.human();
    f.mage(1);
    f.mage(2);
    f.begin();
    f.end(0);
    f.armor(1);
    f.end(1);
    f.end(2);
    f.end(0);
    f.armor(1);
    f.release();
    let old = &mut f
        .state
        .rules
        .as_mut()
        .unwrap()
        .tactical_effects
        .as_mut()
        .unwrap()
        .effects[0];
    old.conditions.push(EffectCondition {
        id: EffectId::new(),
        condition: Condition::Invisible,
    });
    // Negative forged-payload control: the later real cast suppresses this older
    // overlapping record. Scanning only the active projection would miss it.
    assert!(
        !crate::active_conditions(f.state.rules.as_ref().unwrap(), f.actors[1])
            .contains(&Condition::Invisible)
    );
    let before = f.state.clone();
    assert!(transition(&f.state, &f.meta(None), &advance(28_860), &f.pack).is_err());
    assert_eq!(f.state, before);
}

#[test]
fn isolated_129_deadline_fixture_rejects_capacity_before_advancing_time() {
    let mut f = Fixture::new();
    f.human();
    f.mage(1);
    f.mage(2);
    f.begin();
    let meta = f.meta(None);
    let effects = (0..129u16)
        .map(|ordinal| TacticalEffect {
            id: EffectId::new(),
            source: EffectSource {
                definition_id: "isolated-deadline-capacity-fixture".into(),
                actor: f.actors[1],
                command: meta.clone(),
                ordinal,
            },
            established_at: None,
            target: TacticalEffectTarget::Creature(f.actors[1]),
            concentration_group: None,
            expires: TacticalEffectExpiry::AtTime(WorldInstant(100 + i64::from(ordinal))),
            overlap: None,
            conditions: vec![],
            defenses: vec![],
            triggers: vec![],
        })
        .collect();
    f.state = crate::tactical_effect_adapter::apply_effect_operation(
        &f.state,
        &meta,
        &EffectLifecycleAction {
            step: 0,
            operation: EffectLifecycleOperation::Install { effects },
        },
    )
    .unwrap()
    .0;
    f.state.applied_event_sequence += 1;
    f.release();
    let before = f.state.clone();
    assert!(transition(&f.state, &f.meta(None), &advance(229), &f.pack).is_err());
    assert_eq!(f.state, before);
}

fn three_mages_after_one_expiry() -> Fixture {
    let mut f = Fixture::new();
    f.state.campaign.content_packs = TableContract::default().permitted_content.clone();
    let mut table = TableState::new(TableContract::default());
    table.active_session = Some(ActiveTableSession {
        session_id: PlaySessionId::new(),
        display_name: "Completed deadline provenance fixture".into(),
        started_at_world: WorldInstant(0),
        participants: f
            .players
            .iter()
            .map(|player_id| SessionParticipant {
                player_id: *player_id,
                character_id: None,
                attendance: AttendanceStatus::Present,
            })
            .collect(),
    });
    f.state.table = Some(table);
    for index in 0..3 {
        f.mage(index);
    }
    f.begin();
    f.armor(0);
    f.end(0);
    f.armor(1);
    f.end(1);
    f.armor(2);
    f.release();
    f.guarded(advance(28_860));
    let occurrence = resolution(&f.state).unwrap().frames[0][0].occurrence;
    f.guarded(TacticalAction::ChooseTurnWork { occurrence });
    assert_eq!(resolution(&f.state).unwrap().frames[0].len(), 2);
    f
}

fn assert_forged_completion_refused(f: &Fixture, forged: &CampaignState) {
    let before = forged.clone();
    assert!(validate(forged, &f.pack).is_err());
    let occurrence = resolution(forged).unwrap().frames[0][0].occurrence;
    assert!(
        transition(
            forged,
            &f.meta(None),
            &TacticalAction::ChooseTurnWork { occurrence },
            &f.pack
        )
        .is_err()
    );
    assert_eq!(*forged, before);
}

#[test]
fn completed_real_expiry_and_trace_cannot_claim_other_work_or_another_time_ticket() {
    let f = three_mages_after_one_expiry();
    validate(&f.state, &f.pack).unwrap();
    for mutation in 0..5 {
        let mut forged = f.state.clone();
        let r = resolution_mut(&mut forged).unwrap();
        let batch = &mut r.released_interval_mut().unwrap().batches[0];
        let occurrence = batch.completions[0].work.occurrence;
        let binding = batch
            .bindings
            .iter_mut()
            .find(|binding| binding.work.occurrence == occurrence)
            .unwrap();
        match mutation {
            0 => binding.work.kind = TacticalWorkKind::DeathSave { actor: f.actors[0] },
            1..=3 => {
                let TacticalWorkKind::Effect { ticket } = &mut binding.work.kind else {
                    panic!("real expiry")
                };
                match mutation {
                    1 => ticket.command = CommandId::new(),
                    2 => ticket.step += 1,
                    _ => ticket.ordinal += 1,
                }
            }
            4 => {
                binding.source = ReleasedDeadlineSource::Legacy {
                    effect: EffectId::new(),
                    source: f.actors[0],
                    target: f.actors[0],
                    concentration_owner: None,
                }
            }
            _ => unreachable!(),
        }
        let changed = binding.work.clone();
        r.work_trace.as_mut().unwrap().nodes[usize::from(occurrence)].work = changed;
        assert_forged_completion_refused(&f, &forged);
    }
}

fn install_isolated_legacy(
    f: &mut Fixture,
    source: EntityId,
    target: EntityId,
    owner: Option<EntityId>,
    at: WorldInstant,
) -> EffectId {
    let id = EffectId::new();
    let rules = f.state.rules.as_mut().unwrap();
    rules.effects.push(ActiveEffect {
        id,
        source,
        target,
        condition: None,
        label: "Isolated legacy deadline validation fixture".into(),
        expires: Expiry::AtTime(at),
        concentration_owner: owner,
    });
    if let Some(owner) = owner {
        rules.entities.get_mut(&owner).unwrap().concentration = Some(id);
    }
    id
}

#[test]
fn completed_legacy_binding_keeps_its_exact_effect_id_after_removal() {
    let mut f = Fixture::two_mages();
    let actor = f.actors[0];
    let effect = install_isolated_legacy(&mut f, actor, actor, None, WorldInstant(28_800));
    validate(&f.state, &f.pack).unwrap();
    f.guarded(advance(28_860));
    let occurrence = resolution(&f.state).unwrap().frames[0]
        .iter()
        .find(|w| w.kind == TacticalWorkKind::ExpireLegacyEffect { effect })
        .unwrap()
        .occurrence;
    f.guarded(TacticalAction::ChooseTurnWork { occurrence });
    assert_eq!(resolution(&f.state).unwrap().frames[0].len(), 2);
    let mut forged = f.state.clone();
    let r = resolution_mut(&mut forged).unwrap();
    let binding = r.released_interval_mut().unwrap().batches[0]
        .bindings
        .iter_mut()
        .find(|b| b.work.occurrence == occurrence)
        .unwrap();
    binding.work.kind = TacticalWorkKind::ExpireLegacyEffect {
        effect: EffectId::new(),
    };
    let changed = binding.work.clone();
    r.work_trace.as_mut().unwrap().nodes[usize::from(occurrence)].work = changed;
    assert_forged_completion_refused(&f, &forged);
}

fn released_with_real_stable_recovery() -> Fixture {
    let mut f = Fixture::new();
    f.human();
    f.mage(1);
    f.mage(2);
    f.begin();
    f.end(0);
    let target = f.actors[0];
    let meta = f.meta(None);
    let hp = f.state.rules.as_ref().unwrap().entities[&target].hp;
    // As in the original control, damage is the actual private adapter over real
    // mechanics, not an accepted attack history. Medicine and d4 are real inputs.
    let damage = crate::tactical_vitality_adapter::apply(
        &mut f.state,
        target,
        VitalityOrigin {
            command: meta.clone(),
            occurrence: 0,
        },
        &VitalityOperation::Damage {
            packet: DamagePacket {
                cause: DamageCause::Other,
                components: vec![DamageComponent {
                    damage_type: DamageType::Bludgeoning,
                    amounts: vec![hp],
                    adjustments: vec![],
                }],
            },
            knockout: None,
        },
    )
    .unwrap();
    for followup in damage.followups {
        match followup {
            VitalityFollowup::DropHeldItems => {
                crate::tactical_vitality_adapter::drop_held(&mut f.state, target, &meta).unwrap()
            }
            VitalityFollowup::InterruptRest => crate::kernel::interrupt_rest(
                f.state.rules.as_mut().unwrap(),
                target,
                f.state.clock.now,
            ),
            _ => panic!("unexpected damage followup"),
        }
    }
    f.state.applied_event_sequence += 1;
    f.run(
        Some(1),
        TacticalAction::FirstAid {
            target,
            purpose: MedicinePurpose::Stabilize,
        },
    );
    f.roll(1, 20);
    f.roll(0, 2);
    f.release();
    f
}

#[test]
fn completed_real_stable_wake_cannot_change_its_actor_in_retained_work() {
    let mut f = released_with_real_stable_recovery();
    for actor in [f.actors[1], f.actors[2]] {
        install_isolated_legacy(&mut f, actor, actor, None, WorldInstant(7_200));
    }
    validate(&f.state, &f.pack).unwrap();
    f.guarded(advance(7_260));
    let actor = f.actors[0];
    let occurrence = resolution(&f.state).unwrap().frames[0]
        .iter()
        .find(|w| w.kind == TacticalWorkKind::RecoverStable { actor })
        .unwrap()
        .occurrence;
    f.guarded(TacticalAction::ChooseTurnWork { occurrence });
    assert_eq!(f.state.rules.as_ref().unwrap().entities[&actor].hp, 1);
    assert_eq!(resolution(&f.state).unwrap().frames[0].len(), 2);
    let mut forged = f.state.clone();
    let r = resolution_mut(&mut forged).unwrap();
    let binding = r.released_interval_mut().unwrap().batches[0]
        .bindings
        .iter_mut()
        .find(|b| b.work.occurrence == occurrence)
        .unwrap();
    binding.work.kind = TacticalWorkKind::RecoverStable { actor: f.actors[1] };
    let changed = binding.work.clone();
    r.work_trace.as_mut().unwrap().nodes[usize::from(occurrence)].work = changed;
    assert_forged_completion_refused(&f, &forged);
}

fn isolated_outside_dependency(case: usize) -> Fixture {
    let mut f = Fixture::new();
    f.human();
    f.mage(1);
    f.mage(2);
    let outside = f.actors[2];
    // Authored geometry before the real Begin: the prepared third actor is not
    // placed. This is a legacy-boundary fixture, not an accepted capture edit.
    f.state
        .encounter
        .as_mut()
        .unwrap()
        .participants
        .retain(|p| p.entity_id != outside);
    f.run(
        None,
        TacticalAction::Begin {
            execution: TacticalExecutionVersion::EncounterReleaseV1,
            combatants: vec![
                TacticalCombatant {
                    actor: f.actors[0],
                    source: TacticalSource::Character,
                    surprised: false,
                },
                TacticalCombatant {
                    actor: f.actors[1],
                    source: TacticalSource::Creature {
                        definition_id: "mage".into(),
                    },
                    surprised: false,
                },
            ],
            groups: f.actors[..2]
                .iter()
                .map(|actor| InitiativeGroup {
                    actors: vec![*actor],
                    request_id: RollRequestId::new(),
                })
                .collect(),
        },
    );
    f.roll(0, 18);
    f.roll(1, 3);
    let inside = f.actors[1];
    let (source, target, owner) = match case {
        0 => (outside, inside, None),
        1 => (inside, outside, None),
        2 => (inside, inside, Some(outside)),
        _ => unreachable!(),
    };
    install_isolated_legacy(&mut f, source, target, owner, WorldInstant(10));
    f.release();
    f
}

#[test]
fn otherwise_valid_outside_source_target_or_owner_is_refused_before_time_moves() {
    for case in 0..3 {
        let f = isolated_outside_dependency(case);
        validate(&f.state, &f.pack).unwrap();
        let dependencies =
            super::super::release::retained_encounter_dependencies(&f.state).unwrap();
        assert!(dependencies.contains(&f.actors[2]));
        assert!(
            encounter(&f.state)
                .unwrap()
                .participant(f.actors[2])
                .is_none()
        );
        let before = f.state.clone();
        let error = transition(&f.state, &f.meta(None), &advance(11), &f.pack).unwrap_err();
        assert!(
            matches!(error, RulesError::Prerequisite(ref why) if why.contains("every retained dependency"))
        );
        assert_eq!(f.state, before);
    }
}

fn liquid_volume() -> TerrainVolume {
    TerrainVolume {
        id: "authored-liquid".into(),
        volume: SpatialBox {
            min: SpatialPoint { x: 0, y: 0, z: 0 },
            max: SpatialPoint {
                x: 100,
                y: 100,
                z: 40,
            },
        },
        difficult: false,
        observable: true,
        water: true,
        climbable: false,
        burrowable: false,
        supports_top: false,
        surface: Some("water".into()),
        obscuration: Obscuration::None,
        magical_darkness: false,
    }
}

fn set_isolated_released_geometry(f: &mut Fixture, at: i32, liquid: bool, solid: bool) {
    // Geometry fixture only. Both existing authorities agree; no native/replay
    // geometry provenance is claimed by this private admission control.
    let encounter = f.state.encounter.as_mut().unwrap();
    for participant in &mut encounter.participants {
        participant.position.z = at;
    }
    if liquid {
        encounter.battlefield.terrain.push(liquid_volume());
    }
    if solid {
        encounter.battlefield.obstacles.push(SpatialObstacle {
            id: "authored-solid-support".into(),
            volume: SpatialBox {
                min: SpatialPoint { x: 0, y: 0, z: 0 },
                max: SpatialPoint {
                    x: 100,
                    y: 100,
                    z: at,
                },
            },
            blocks_movement: true,
            blocks_sight: true,
            observable: true,
            cover: CoverDegree::Total,
        });
    }
    f.state
        .encounter_history
        .as_mut()
        .unwrap()
        .spaces
        .last_mut()
        .unwrap()
        .battlefield = encounter.battlefield.clone();
}

#[test]
fn liquid_suspension_and_surface_are_not_ground_support_even_without_a_fall() {
    for stable in [false, true] {
        for at in [20, 40] {
            let mut f = if stable {
                released_with_real_stable_recovery()
            } else {
                Fixture::two_mages()
            };
            set_isolated_released_geometry(&mut f, at, true, false);
            validate(&f.state, &f.pack).unwrap();
            for actor in f.actors {
                assert!(
                    crate::spatial::fall_destination(encounter(&f.state).unwrap(), actor)
                        .unwrap()
                        .is_none()
                );
                assert!(
                    crate::spatial::flight_loss_fall(encounter(&f.state).unwrap(), &f.state, actor)
                        .unwrap()
                        .is_none()
                );
            }
            let before = f.state.clone();
            let error = transition(&f.state, &f.meta(None), &advance(28_860), &f.pack).unwrap_err();
            assert!(
                matches!(error, RulesError::Prerequisite(ref why) if why.contains("ground-supported"))
            );
            assert_eq!(f.state, before);
        }
    }
}

#[test]
fn real_elapsed_sources_still_advance_on_actual_floor_and_solid_top_support() {
    for at in [0, 20] {
        let mut f = Fixture::two_mages();
        set_isolated_released_geometry(&mut f, at, false, at != 0);
        validate(&f.state, &f.pack).unwrap();
        let before = f.state.clone();
        f.guarded(advance(10));
        assert_eq!(f.state.clock.now, WorldInstant(10));
        assert_eq!(f.state.items, before.items);
        assert_eq!(effects(&f.state).unwrap(), effects(&before).unwrap());
        assert_eq!(
            f.state.encounter.as_ref().unwrap().participants,
            before.encounter.as_ref().unwrap().participants
        );
    }
}

#[test]
fn cancelled_group_children_keep_their_allowed_work_and_time_ticket_shape() {
    let mut f = Fixture::two_mages();
    let meta = f.meta(None);
    let group = EffectId::new();
    let source = EffectSource {
        definition_id: "isolated-cancelled-binding-fixture".into(),
        actor: f.actors[1],
        command: meta.clone(),
        ordinal: 0,
    };
    // Isolated lifecycle installation, not a new admitted spell/profile or an
    // accepted released-time command. Actual expiry and cleanup use the producer.
    for (step, operation) in [
        EffectLifecycleOperation::BeginConcentration {
            group: ConcentrationGroup {
                id: group,
                source: source.clone(),
                expires: TacticalEffectExpiry::AtTime(WorldInstant(28_800)),
                stage: ConcentrationStage::Casting,
            },
        },
        EffectLifecycleOperation::Install {
            effects: [f.actors[1], f.actors[2]]
                .iter()
                .map(|actor| TacticalEffect {
                    id: EffectId::new(),
                    source: source.clone(),
                    established_at: None,
                    target: TacticalEffectTarget::Creature(*actor),
                    concentration_group: Some(group),
                    expires: TacticalEffectExpiry::AtTime(WorldInstant(28_800)),
                    overlap: None,
                    conditions: vec![],
                    defenses: vec![],
                    triggers: vec![],
                })
                .collect(),
        },
    ]
    .into_iter()
    .enumerate()
    {
        f.state = crate::tactical_effect_adapter::apply_effect_operation(
            &f.state,
            &meta,
            &EffectLifecycleAction {
                step: step as u16,
                operation,
            },
        )
        .unwrap()
        .0;
    }
    f.state.applied_event_sequence += 1;
    validate(&f.state, &f.pack).unwrap();
    f.guarded(advance(28_860));
    let batch = &resolution(&f.state)
        .unwrap()
        .released_interval()
        .unwrap()
        .batches[0];
    let occurrence = batch
        .bindings
        .iter()
        .find(|b| {
            matches!(b.source,
        ReleasedDeadlineSource::Group { group: id, .. } if id == group)
        })
        .unwrap()
        .work
        .occurrence;
    f.guarded(TacticalAction::ChooseTurnWork { occurrence });
    let r = resolution(&f.state).unwrap();
    assert_eq!(r.frames[0].len(), 2);
    let cancelled = r.released_interval().unwrap().batches[0]
        .completions
        .iter()
        .filter(|c| matches!(c.outcome, ReleasedWorkOutcome::CancelledBy { .. }))
        .map(|c| c.work.occurrence)
        .collect::<Vec<_>>();
    assert_eq!(cancelled.len(), 2);
    for occurrence in cancelled {
        for change_kind in [false, true] {
            let mut forged = f.state.clone();
            let r = resolution_mut(&mut forged).unwrap();
            let binding = r.released_interval_mut().unwrap().batches[0]
                .bindings
                .iter_mut()
                .find(|b| b.work.occurrence == occurrence)
                .unwrap();
            if change_kind {
                binding.work.kind = TacticalWorkKind::RecoverStable { actor: f.actors[0] };
            } else {
                let TacticalWorkKind::Effect { ticket } = &mut binding.work.kind else {
                    panic!("expiry child")
                };
                ticket.command = CommandId::new();
            }
            let changed = binding.work.clone();
            r.work_trace.as_mut().unwrap().nodes[usize::from(occurrence)].work = changed;
            assert_forged_completion_refused(&f, &forged);
        }
    }
}

#[test]
fn paused_interval_still_requires_geometry_for_a_future_raw_dependency() {
    let mut f = isolated_outside_dependency(0);
    // Establish a real paused batch using isolated legacy deadlines on placed
    // actors. The later mutation must not hide behind the current due partition.
    f.state.rules.as_mut().unwrap().effects.clear();
    for actor in [f.actors[0], f.actors[1]] {
        install_isolated_legacy(&mut f, actor, actor, None, WorldInstant(10));
    }
    validate(&f.state, &f.pack).unwrap();
    f.guarded(advance(11));
    assert_eq!(resolution(&f.state).unwrap().frames[0].len(), 2);
    let mut forged = f.state.clone();
    let outside = f.actors[2];
    forged.rules.as_mut().unwrap().effects.push(ActiveEffect {
        id: EffectId::new(),
        source: outside,
        target: outside,
        condition: None,
        label: "Forged future outside dependency".into(),
        expires: Expiry::AtTime(WorldInstant(20)),
        concentration_owner: None,
    });
    assert!(forged.validate().is_empty());
    let before = forged.clone();
    let error = validate(&forged, &f.pack).unwrap_err();
    assert!(
        matches!(error, RulesError::Prerequisite(ref why) if why.contains("every retained dependency"))
    );
    let occurrence = resolution(&forged).unwrap().frames[0][0].occurrence;
    assert!(
        transition(
            &forged,
            &f.meta(None),
            &TacticalAction::ChooseTurnWork { occurrence },
            &f.pack
        )
        .is_err()
    );
    assert_eq!(forged, before);
}

#[test]
fn released_interval_refuses_shove_attachment_work_and_commands_without_turn_authority() {
    let mut f = Fixture::two_mages();
    f.run(None, advance(28_860));
    let original = f.state.clone();
    validate_tactical_state(&original).unwrap();
    crate::validate_state(&original, &f.pack).unwrap();
    let r = resolution(&original).unwrap();
    assert!(r.turn_context().is_err());
    assert!(r.released_interval().is_some());
    assert!(r.shove.is_none());
    assert!(original.rules.as_ref().unwrap().timing.is_none());
    let occurrence = r.frames[0][0].occurrence;

    // Hostile structural attachment only, never an accepted Shove producer.
    // The explicit interval validator must reject the field independently of
    // the ordinary Turn/Shove validator (which released work does not enter).
    let mut forged = original.clone();
    resolution_mut(&mut forged).unwrap().shove = Some(Box::new(TacticalShove {
        origin: r.origin.clone(),
        actor: f.actors[0],
        target: f.actors[1],
        window: WeaponActionWindow {
            id: r.origin.id,
            kind: WeaponActionKind::AttackAction,
        },
        actor_source: None,
        target_source: None,
        actor_from: SpatialPoint { x: 10, y: 10, z: 0 },
        target_from: SpatialPoint { x: 20, y: 10, z: 0 },
        difficulty: 13,
        stage: TacticalShoveStage::SaveChoice,
        selected: None,
        save: None,
        push: None,
        effect: None,
    }));
    let encoded = forged.encode_json().unwrap();
    let forged = CampaignState::decode_json(&encoded).unwrap();
    assert!(
        ReleasedValidation::derive(&forged)
            .err()
            .unwrap()
            .to_string()
            .contains("invalid persisted released interval boundary")
    );
    assert!(
        resolve_tactical(
            &forged,
            &f.meta(None),
            &TacticalAction::ChooseTurnWork { occurrence },
            &f.pack
        )
        .is_err()
    );
    assert_eq!(forged.encode_json().unwrap(), encoded);
    for kind in [
        TacticalWorkKind::BeginShove,
        TacticalWorkKind::ShoveSave,
        TacticalWorkKind::ChooseShoveOutcome,
        TacticalWorkKind::FinishShove,
    ] {
        let mut forged = original.clone();
        resolution_mut(&mut forged).unwrap().frames[0][0].kind = kind;
        let before = forged.clone();
        assert!(validate_tactical_state(&forged).is_err());
        assert!(
            resolve_tactical(
                &forged,
                &f.meta(None),
                &TacticalAction::ChooseTurnWork { occurrence },
                &f.pack
            )
            .is_err()
        );
        assert_eq!(forged, before);
    }
    for action in [
        TacticalAction::Shove {
            target: f.actors[1],
        },
        TacticalAction::ChooseShoveSave {
            ability: ShoveSaveAbility::Strength,
        },
        TacticalAction::ChooseShoveOutcome {
            choice: ShoveChoice::Prone,
        },
        TacticalAction::RuleShovePush {
            ruling: ShoveGeometryRuling::CommitExactPush,
        },
    ] {
        for meta in [f.meta(None), f.meta(Some(0))] {
            assert!(resolve_tactical(&original, &meta, &action, &f.pack).is_err());
        }
    }
    assert_eq!(f.state, original);
    let completed = f.run(None, TacticalAction::ChooseTurnWork { occurrence });
    assert!(completed.outcome.active_actor.is_none());
    assert!(!completed.outcome.awaiting_turn_work);
    assert_eq!(f.state.clock.now, WorldInstant(28_860));
    assert!(flow(&f.state).unwrap().resolution.is_none());
    assert!(f.state.rules.as_ref().unwrap().timing.is_none());
    assert!(effects(&f.state).unwrap().effects.is_empty());
}
