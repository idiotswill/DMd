//! Guarded private-handler controls. The pre-Grapple capture supplies real Human
//! and creature definitions/items. Fresh executor/budget scaffolding and later
//! private invocations are synthetic internal execution, never accepted replay.
use super::*;
use crate::tactical_creature_equipment::*;
use crate::tactical_creatures::*;

pub(super) struct Fixture {
    pub state: CampaignState,
    pub pack: RulesPack,
    pub human: EntityId,
    pub target: EntityId,
}
impl Fixture {
    pub(super) fn new() -> Self {
        let mut state = crate::tactical_hands::tests::source_state();
        let pack = crate::tactical_hands::tests::pack();
        let human = crate::tactical_hands::tests::human(&state);
        let target = state
            .rules
            .as_ref()
            .unwrap()
            .tactical_creatures
            .as_ref()
            .unwrap()
            .profiles[0]
            .actor;
        // These fresh-turn/executor facts are expressly an internal scaffold.
        state
            .encounter
            .as_mut()
            .unwrap()
            .flow
            .as_mut()
            .unwrap()
            .version = 5;
        state
            .encounter
            .as_mut()
            .unwrap()
            .flow
            .as_mut()
            .unwrap()
            .budget = TacticalTurnBudget::default();
        state
            .rules
            .as_mut()
            .unwrap()
            .timing
            .as_mut()
            .unwrap()
            .action_spent = false;
        let mut f = Self {
            state,
            pack,
            human,
            target,
        };
        let origin = f.meta(None);
        let current = f
            .state
            .rules
            .as_ref()
            .unwrap()
            .tactical_creatures
            .as_ref()
            .unwrap();
        let changed = apply_creature_schedule(
            &f.state,
            current,
            &origin,
            &CreatureScheduleOperation::SetContext {
                actor: target,
                controller: CreatureController::Host,
                in_lair: false,
            },
        )
        .unwrap();
        f.state.rules.as_mut().unwrap().tactical_creatures = Some(changed.next);
        f.state.applied_event_sequence += 1;
        f
    }
    pub(super) fn meta(&self, actor: Option<EntityId>) -> CommandMeta {
        let issuer = actor
            .and_then(|a| controller(&self.state, a))
            .map_or(CommandIssuer::Admin, CommandIssuer::Player);
        CommandMeta {
            id: CommandId::new(),
            campaign_id: self.state.campaign_id(),
            session_id: self
                .state
                .rules
                .as_ref()
                .unwrap()
                .tactical_inventory
                .as_ref()
                .unwrap()
                .loadout(self.human)
                .unwrap()
                .command
                .session_id,
            issuer,
            actor: actor.map(AgentRef::Entity),
            expected_event_sequence: self.state.applied_event_sequence,
        }
    }
    fn run(
        &mut self,
        actor: Option<EntityId>,
        apply: impl FnOnce(&mut CampaignState, &CommandMeta) -> Result<(), RulesError>,
    ) -> CommandMeta {
        let meta = self.meta(actor);
        let mut next = self.state.clone();
        apply(&mut next, &meta).unwrap();
        validate(&next).unwrap();
        next.applied_event_sequence += 1;
        self.state = next;
        meta
    }
    pub(super) fn begin(
        &mut self,
        hand: Hand,
        before: Option<AttackEquipmentOperation>,
    ) -> GrappleId {
        let target = self.target;
        let pack = self.pack.clone();
        self.run(Some(self.human), |s, m| {
            begin(s, m, target, hand, before, &pack)
        });
        attempt(&self.state).unwrap().declaration.id
    }
    fn choose(&mut self, id: GrappleId, ability: GrappleSaveAbility) {
        self.run(Some(self.target), |s, m| choose_save(s, m, id, ability));
    }
    fn submit(&mut self, face: u16) {
        let p = self.state.rules.as_ref().unwrap().pending.as_ref().unwrap();
        let result = RollResult {
            request_id: p.request.id,
            source: RollSource::Physical,
            dice: vec![
                DieResult {
                    sides: 20,
                    value: face
                };
                if p.request.mode == RollMode::Normal {
                    1
                } else {
                    2
                }
            ],
        };
        self.run(Some(self.target), |s, m| {
            super::super::continuations::submit(s, m, &result, None)
        });
    }
    fn decline(&mut self, id: GrappleId) {
        let work = work_key(
            &self.state,
            attempt(&self.state).unwrap().selected.as_ref().unwrap(),
        )
        .unwrap();
        self.run(Some(self.human), |s, m| {
            decline_after_equipment(s, m, id, work)
        });
    }
    fn item(&self, definition: &str) -> ItemId {
        self.state
            .items
            .values()
            .find(|i| i.custody == Custody::Entity(self.human) && i.definition_id == definition)
            .unwrap()
            .id
    }
    fn loose_greatsword(&mut self) -> ItemId {
        // Physical loot fixture using the real weapon definition; no acquisition
        // or source grant is claimed by this isolated initial-image setup.
        let mut item = self.state.items[&self.item("dagger")].clone();
        item.id = ItemId::new();
        item.definition_id = "greatsword".into();
        let id = item.id;
        self.state.items.insert(id, item);
        id
    }
    fn replace_target(&mut self, definition: &str, size: CreatureSize) {
        let actor = self.target;
        let origin = self.meta(None);
        self.state.items.retain(|_, i| {
            i.custody != Custody::Entity(actor) && i.owner != Ownership::Entity(actor)
        });
        let rules = self.state.rules.as_mut().unwrap();
        rules.entities.remove(&actor);
        rules
            .tactical_inventory
            .as_mut()
            .unwrap()
            .loadouts
            .retain(|l| l.actor != actor);
        let creatures = rules.tactical_creatures.as_mut().unwrap();
        let observed = creatures.runtime(actor).unwrap().observed_turn;
        creatures.profiles.retain(|p| p.actor != actor);
        creatures.runtime.retain(|r| r.actor != actor);
        let source = current_creature_sources()
            .unwrap()
            .into_iter()
            .find(|s| s.id == definition)
            .unwrap();
        let pin = creature_source_pin(source).unwrap();
        let mut built = build_creature_from_source(
            &self.state,
            &origin,
            actor,
            &CreatureBuildChoice {
                definition_id: definition.into(),
                size,
                additional_languages: if definition == "mage" {
                    vec!["dwarvish".into(), "elvish".into(), "draconic".into()]
                } else {
                    vec![]
                },
                hit_points: CreatureHitPointChoice::Average,
                controller: CreatureController::Host,
                in_lair: false,
            },
            Some(&pin),
        )
        .unwrap();
        built.runtime.observed_turn = observed;
        let rules = self.state.rules.as_mut().unwrap();
        rules.entities.insert(actor, built.mechanics);
        let creatures = rules.tactical_creatures.as_mut().unwrap();
        creatures.profiles.push(built.profile);
        creatures.runtime.push(built.runtime);
        let ammunition = if definition == "goblin-warrior" {
            20
        } else {
            0
        };
        let ids = creature_equipment_plan_from_source(&pin, ammunition)
            .unwrap()
            .iter()
            .map(|_| ItemId::new())
            .collect::<Vec<_>>();
        self.state = materialize_creature_equipment(
            &self.state,
            &origin,
            actor,
            ammunition,
            &ids,
            &self.pack,
        )
        .unwrap();
        let encounter = self.state.encounter.as_mut().unwrap();
        let p = encounter
            .participants
            .iter_mut()
            .find(|p| p.entity_id == actor)
            .unwrap();
        p.size = size;
        p.height = size.footprint_units() as u32;
        p.movement = built.movement;
        p.senses = built.senses;
        encounter
            .flow
            .as_mut()
            .unwrap()
            .combatants
            .iter_mut()
            .find(|c| c.actor == actor)
            .unwrap()
            .source = TacticalSource::Creature {
            definition_id: definition.into(),
        };
        self.state.applied_event_sequence += 1;
    }
    fn paralyze_target(&mut self) {
        self.state
            .rules
            .as_mut()
            .unwrap()
            .effects
            .push(ActiveEffect {
                id: EffectId::new(),
                source: self.human,
                target: self.target,
                condition: Some(Condition::Paralyzed),
                label: "Synthetic imported condition for no-die control".into(),
                expires: Expiry::Never,
                concentration_owner: None,
            });
    }
}

#[test]
fn every_new_command_is_denied_in_live_and_historical_dispatch() {
    let f = Fixture::new();
    let id = GrappleId::from_declaration(CommandId::new(), f.human, f.target, Hand::Left);
    let work = TacticalWorkKey {
        resolution: CommandId::new(),
        occurrence: 0,
    };
    let actions = [
        TacticalAction::Grapple {
            target: f.target,
            hand: Hand::Left,
            before_change: None,
        },
        TacticalAction::ChooseGrappleSave {
            grip: id,
            ability: GrappleSaveAbility::Strength,
        },
        TacticalAction::ApplyGrappleAfterEquipment {
            grip: id,
            work,
            operation: AttackEquipmentOperation::Unequip {
                item: f.item("dagger"),
            },
        },
        TacticalAction::DeclineGrappleAfterEquipment { grip: id, work },
        TacticalAction::WithdrawGrapple { grip: id },
        TacticalAction::EscapeGrapple {
            grip: id,
            choice: GrappleEscapeChoice::Athletics,
        },
        TacticalAction::ReleaseGrapple { grip: id },
    ];
    for version in 1..=5 {
        let mut state = f.state.clone();
        state
            .encounter
            .as_mut()
            .unwrap()
            .flow
            .as_mut()
            .unwrap()
            .version = version;
        let before = serde_json::to_vec(&state).unwrap();
        for action in &actions {
            for policy in [ExecutionPolicy::Live, ExecutionPolicy::Historical] {
                let error = super::super::resolve_with_policy(
                    &state,
                    &f.meta(Some(f.human)),
                    action,
                    &f.pack,
                    policy,
                )
                .unwrap_err();
                assert!(error.to_string().contains(if version == 5 {
                    "not enabled"
                } else {
                    "release execution"
                }));
                assert_eq!(serde_json::to_vec(&state).unwrap(), before);
            }
        }
    }
}

#[test]
fn actual_weapon_before_operation_frees_only_its_paid_hand() {
    let mut f = Fixture::new();
    let sword = f.loose_greatsword();
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
        .find(|l| l.actor == f.human)
        .unwrap();
    loadout.hands.hands = [HandAssignment::Item(sword); 2];
    let original = serde_json::to_vec(&f.state).unwrap();
    let meta = f.meta(Some(f.human));
    assert!(begin(&mut f.state, &meta, f.target, Hand::Left, None, &f.pack).is_err());
    assert_eq!(serde_json::to_vec(&f.state).unwrap(), original);
    let id = f.begin(
        Hand::Left,
        Some(AttackEquipmentOperation::Unequip { item: sword }),
    );
    let a = attempt(&f.state).unwrap();
    assert_eq!(a.declaration.id, id);
    assert_eq!(
        a.equipment.equipment_before.hands.hands,
        [HandAssignment::Item(sword); 2]
    );
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
    admission::validate_attempt_admission(&f.state).unwrap();
    let hands = crate::tactical_hands::EffectiveHands::current(
        &f.state,
        f.state.rules.as_ref().unwrap(),
        f.human,
    )
    .unwrap();
    assert!(!hands.is_free(
        &admission::loadout(&f.state, f.human).unwrap().hands,
        Hand::Left
    ));
    assert!(hands.is_free(
        &admission::loadout(&f.state, f.human).unwrap().hands,
        Hand::Right
    ));
    let mut forged = f.state.clone();
    attempt_mut(&mut forged).unwrap().declaration.hand = Hand::Right;
    assert!(admission::validate_attempt_admission(&forged).is_err());
    let mut forged = f.state.clone();
    forged.items.get_mut(&sword).unwrap().quantity = 0;
    assert!(admission::validate_attempt_admission(&forged).is_err());
    assert!(
        crate::validate_state(&f.state, &f.pack)
            .unwrap_err()
            .to_string()
            .contains("not enabled")
    );
}

#[test]
fn successful_grip_release_during_after_equipment_retains_exact_outcome_and_end() {
    let mut f = Fixture::new();
    let sword = f.loose_greatsword();
    let id = f.begin(Hand::Left, None);
    f.choose(id, GrappleSaveAbility::Dexterity);
    assert_eq!(
        f.state
            .rules
            .as_ref()
            .unwrap()
            .pending
            .as_ref()
            .unwrap()
            .request
            .modifier,
        2
    );
    f.submit(1);
    assert_eq!(
        attempt(&f.state).unwrap().outcome,
        Some(GrappleAttemptOutcome::Established { grip: id })
    );
    assert!(
        admission::validate_attempt_admission(&f.state).is_err(),
        "completed live hand cannot be excluded"
    );
    validate(&f.state).unwrap();
    let work = work_key(
        &f.state,
        attempt(&f.state).unwrap().selected.as_ref().unwrap(),
    )
    .unwrap();
    let meta = f.meta(Some(f.human));
    let before = serde_json::to_vec(&f.state).unwrap();
    assert!(
        apply_after_equipment(
            &mut f.state,
            &meta,
            id,
            work,
            AttackEquipmentOperation::Equip {
                item: sword,
                hand: Hand::Left
            }
        )
        .is_err()
    );
    assert_eq!(serde_json::to_vec(&f.state).unwrap(), before);
    f.run(Some(f.human), |s, m| release(s, m, id));
    assert_eq!(
        attempt(&f.state).unwrap().outcome,
        Some(GrappleAttemptOutcome::Established { grip: id })
    );
    assert_eq!(context(&f.state).unwrap().proofs.len(), 1);
    assert_eq!(
        context(&f.state).unwrap().ends[0].cause,
        GrappleEndCause::Released
    );
    f.run(Some(f.human), |s, m| {
        apply_after_equipment(
            s,
            m,
            id,
            work,
            AttackEquipmentOperation::Equip {
                item: sword,
                hand: Hand::Left,
            },
        )
    });
    assert!(flow(&f.state).unwrap().resolution.is_none());
    assert!(f.state.rules.as_ref().unwrap().tactical_grapples.is_none());
    assert_eq!(
        admission::loadout(&f.state, f.human).unwrap().hands.hands[0],
        HandAssignment::Item(sword)
    );
    assert!(
        crate::tactical_hands::EffectiveHands::current(
            &f.state,
            f.state.rules.as_ref().unwrap(),
            f.human
        )
        .is_err(),
        "retired raw authentication remains guarded"
    );
}

#[test]
fn withdrawal_before_or_during_raw_preserves_one_equipment_allowance() {
    for before_used in [false, true] {
        for issued in [false, true] {
            let mut f = Fixture::new();
            let operation = before_used.then_some(AttackEquipmentOperation::Unequip {
                item: f.item("dagger"),
            });
            let id = f.begin(
                if before_used { Hand::Right } else { Hand::Left },
                operation,
            );
            if issued {
                f.choose(id, GrappleSaveAbility::Strength);
            }
            let pending = f
                .state
                .rules
                .as_ref()
                .unwrap()
                .pending
                .as_ref()
                .map(|p| p.request.id);
            let raw_count = f.state.rules.as_ref().unwrap().rolls.len();
            f.run(Some(f.human), |s, m| withdraw(s, m, id));
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
            assert_eq!(f.state.rules.as_ref().unwrap().rolls.len(), raw_count);
            if let Some(request) = pending {
                assert_eq!(
                    f.state
                        .rules
                        .as_ref()
                        .unwrap()
                        .cancelled_roll_ids
                        .iter()
                        .filter(|x| **x == request)
                        .count(),
                    1
                );
            }
            if before_used {
                assert!(flow(&f.state).unwrap().resolution.is_none());
            } else {
                assert_eq!(
                    attempt(&f.state).unwrap().stage,
                    TacticalGrappleAttemptStage::AfterEquipment
                );
                f.decline(id);
            }
        }
    }
}

#[test]
fn actual_no_lr_automatic_and_voluntary_failures_use_the_common_private_pump() {
    for automatic in [false, true] {
        let mut f = Fixture::new();
        if automatic {
            f.paralyze_target();
        }
        let count = f.state.rules.as_ref().unwrap().rolls.len();
        let id = f.begin(Hand::Left, None);
        f.choose(id, GrappleSaveAbility::Strength);
        if !automatic {
            f.run(Some(f.target), |s, m| {
                super::super::continuations::voluntarily_fail(s, m)
            });
        }
        let save = attempt(&f.state).unwrap().save.as_ref().unwrap();
        assert!(matches!(
            save.proof.as_ref().unwrap().evidence,
            GrappleSaveEvidence::Decision(_)
        ));
        assert_eq!(save.request.is_none(), automatic);
        assert_eq!(f.state.rules.as_ref().unwrap().rolls.len(), count);
        assert_eq!(
            flow(&f.state)
                .unwrap()
                .save_decisions
                .last()
                .unwrap()
                .failure,
            if automatic {
                TacticalSaveFailure::Automatic
            } else {
                TacticalSaveFailure::Voluntary
            }
        );
        assert_eq!(
            attempt(&f.state).unwrap().stage,
            TacticalGrappleAttemptStage::AfterEquipment
        );
        f.decline(id);
    }
}

#[test]
fn resistance_and_actual_air_immunity_keep_paid_outcomes_and_owned_decline() {
    for air in [false, true] {
        let mut f = Fixture::new();
        if air {
            f.replace_target("air-elemental", CreatureSize::Large);
        }
        let id = f.begin(Hand::Left, None);
        f.choose(id, GrappleSaveAbility::Strength);
        f.submit(if air { 1 } else { 20 });
        assert!(
            matches!(
                attempt(&f.state).unwrap().outcome,
                Some(GrappleAttemptOutcome::Immune { .. })
            ) == air
        );
        assert!(f.state.rules.as_ref().unwrap().tactical_grapples.is_none());
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
        f.decline(id);
    }
}

#[test]
fn actual_dragon_size_and_deferred_dodge_are_atomic_refusals() {
    let mut f = Fixture::new();
    f.replace_target("adult-red-dragon", CreatureSize::Huge);
    let meta = f.meta(Some(f.human));
    let before = serde_json::to_vec(&f.state).unwrap();
    assert!(begin(&mut f.state, &meta, f.target, Hand::Left, None, &f.pack).is_err());
    assert_eq!(serde_json::to_vec(&f.state).unwrap(), before);
    let mut f = Fixture::new();
    let origin = f.meta(Some(f.target));
    flow_mut(&mut f.state).unwrap().dodges.push(TacticalDodge {
        actor: f.target,
        origin,
        declared_on_turn: 1,
    });
    let meta = f.meta(Some(f.human));
    let before = serde_json::to_vec(&f.state).unwrap();
    assert!(
        begin(&mut f.state, &meta, f.target, Hand::Left, None, &f.pack)
            .unwrap_err()
            .to_string()
            .contains("Dodge")
    );
    assert_eq!(serde_json::to_vec(&f.state).unwrap(), before);
}

#[test]
fn selected_work_and_source_forgery_never_bypass_the_provisional_admission() {
    let mut f = Fixture::new();
    let id = f.begin(Hand::Left, None);
    let mut forged = f.state.clone();
    resolution_mut(&mut forged)
        .unwrap()
        .work_trace
        .as_mut()
        .unwrap()
        .nodes[0]
        .parent = Some(0);
    assert!(validate(&forged).is_err());
    let mut forged = f.state.clone();
    attempt_mut(&mut forged)
        .unwrap()
        .equipment
        .equipment_before
        .command
        .id = CommandId::new();
    assert!(admission::validate_attempt_admission(&forged).is_err());
    let before = serde_json::to_vec(&f.state).unwrap();
    let foreign = f.meta(Some(f.human));
    assert!(choose_save(&mut f.state, &foreign, id, GrappleSaveAbility::Strength).is_err());
    assert_eq!(serde_json::to_vec(&f.state).unwrap(), before);
    let meta = f.meta(Some(f.human));
    assert!(super::super::turns::choose(&mut f.state, &meta, 0).is_err());
    assert_eq!(serde_json::to_vec(&f.state).unwrap(), before);
}

#[test]
fn old_source_cannot_grapple_but_current_goblin_can_grapple_owned_human() {
    for current in [false, true] {
        let mut f = Fixture::new();
        if current {
            f.replace_target("goblin-warrior", CreatureSize::Small);
        }
        f.run(Some(f.human), |s, m| {
            super::super::turns::core_action(s, m, &TacticalAction::EndTurn)
        });
        let meta = f.meta(Some(f.target));
        let before = serde_json::to_vec(&f.state).unwrap();
        let result = begin(&mut f.state, &meta, f.human, Hand::Right, None, &f.pack);
        if !current {
            assert!(result.is_err());
            assert_eq!(serde_json::to_vec(&f.state).unwrap(), before);
            continue;
        }
        result.unwrap();
        validate(&f.state).unwrap();
        f.state.applied_event_sequence += 1;
        let id = attempt(&f.state).unwrap().declaration.id;
        f.run(Some(f.human), |s, m| {
            choose_save(s, m, id, GrappleSaveAbility::Dexterity)
        });
        let p = f.state.rules.as_ref().unwrap().pending.as_ref().unwrap();
        assert_eq!(p.request.modifier, 2);
        let result = RollResult {
            request_id: p.request.id,
            source: RollSource::Physical,
            dice: vec![DieResult {
                sides: 20,
                value: 1,
            }],
        };
        f.run(Some(f.human), |s, m| {
            super::super::continuations::submit(s, m, &result, None)
        });
        let work = work_key(
            &f.state,
            attempt(&f.state).unwrap().selected.as_ref().unwrap(),
        )
        .unwrap();
        f.run(Some(f.target), |s, m| {
            decline_after_equipment(s, m, id, work)
        });
        f.run(Some(f.target), |s, m| {
            super::super::turns::core_action(s, m, &TacticalAction::EndTurn)
        });
        f.run(Some(f.human), |s, m| {
            begin_escape(s, m, id, GrappleEscapeChoice::Acrobatics)
        });
        let e = escape(&f.state).unwrap();
        assert_eq!(
            e.request.as_ref().unwrap().modifier,
            4,
            "actual Human Acrobatics proficiency differs from Dexterity save"
        );
        let id_request = e.key.request_id();
        let before = serde_json::to_vec(&f.state).unwrap();
        let meta = f.meta(Some(f.human));
        assert!(super::super::continuations::voluntarily_fail(&mut f.state, &meta).is_err());
        assert_eq!(serde_json::to_vec(&f.state).unwrap(), before);
        f.run(Some(f.target), |s, m| release(s, m, id));
        assert!(flow(&f.state).unwrap().resolution.is_none());
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
        assert_eq!(
            f.state
                .rules
                .as_ref()
                .unwrap()
                .cancelled_roll_ids
                .iter()
                .filter(|x| **x == id_request)
                .count(),
            1
        );
    }
}

#[test]
fn completed_request_and_automatic_proofs_cannot_authenticate_themselves() {
    let mut f = Fixture::new();
    let id = f.begin(Hand::Left, None);
    f.choose(id, GrappleSaveAbility::Strength);
    f.submit(1);
    let key = attempt(&f.state).unwrap().save.as_ref().unwrap().key;
    let mut forged = f.state.clone();
    let save = attempt_mut(&mut forged).unwrap().save.as_mut().unwrap();
    save.request.as_mut().unwrap().modifier -= 7;
    let changed = save.clone();
    context_mut(&mut forged).unwrap().proofs[0].save = changed.clone();
    forged
        .rules
        .as_mut()
        .unwrap()
        .tactical_grapples
        .as_mut()
        .unwrap()
        .active[0]
        .save = changed.clone();
    let raw = forged
        .rules
        .as_mut()
        .unwrap()
        .rolls
        .iter_mut()
        .find(|r| r.request.id == key.request_id())
        .unwrap();
    raw.request = changed.request.unwrap();
    raw.resolved = raw.request.resolve(&raw.result).unwrap();
    assert!(
        validate(&forged)
            .unwrap_err()
            .to_string()
            .contains("actual source")
    );

    let mut f = Fixture::new();
    f.paralyze_target();
    let id = f.begin(Hand::Left, None);
    f.choose(id, GrappleSaveAbility::Strength);
    let mut forged = f.state.clone();
    forged
        .rules
        .as_mut()
        .unwrap()
        .effects
        .retain(|e| e.condition != Some(Condition::Paralyzed));
    assert!(
        validate(&forged)
            .unwrap_err()
            .to_string()
            .contains("actual source")
    );
    let mut forged = f.state.clone();
    flow_mut(&mut forged).unwrap().save_decisions.clear();
    assert!(validate(&forged).is_err());
    let mut forged = f.state.clone();
    let a = attempt_mut(&mut forged).unwrap();
    a.stage = TacticalGrappleAttemptStage::Saving;
    a.selected = None;
    a.outcome = None;
    a.save.as_mut().unwrap().proof = None;
    context_mut(&mut forged).unwrap().proofs.clear();
    forged.rules.as_mut().unwrap().tactical_grapples = None;
    assert!(
        validate(&forged).is_err(),
        "bare unresolved no-die pause has no matching LR tuple"
    );
}

#[test]
fn source_save_choices_and_owned_inspiration_preserve_original_faces() {
    let mut f = Fixture::new();
    f.replace_target("mage", CreatureSize::Medium);
    let id = f.begin(Hand::Left, None);
    let declaration = &attempt(&f.state).unwrap().declaration;
    let key = TacticalRollKey {
        origin: declaration.origin.id,
        role: TacticalRollRole::GrappleSave,
        subject: f.target,
        occurrence: 1,
    };
    let strength = saves::expected_save(&f.state, declaration, GrappleSaveAbility::Strength, key)
        .unwrap()
        .unwrap();
    let dexterity = saves::expected_save(&f.state, declaration, GrappleSaveAbility::Dexterity, key)
        .unwrap()
        .unwrap();
    assert_eq!((strength.modifier, dexterity.modifier), (-1, 2));
    f.choose(id, GrappleSaveAbility::Dexterity);
    // Explicit initial resource scaffold, consumed by the actual shared submit.
    f.state
        .rules
        .as_mut()
        .unwrap()
        .entities
        .get_mut(&f.target)
        .unwrap()
        .heroic_inspiration = true;
    let pending = f.state.rules.as_ref().unwrap().pending.as_ref().unwrap();
    let result = RollResult {
        request_id: pending.request.id,
        source: RollSource::Physical,
        dice: vec![DieResult {
            sides: 20,
            value: 1,
        }],
    };
    let before = serde_json::to_vec(&f.state).unwrap();
    let foreign = f.meta(Some(f.human));
    assert!(
        super::super::continuations::submit(
            &mut f.state,
            &foreign,
            &result,
            Some((
                0,
                DieResult {
                    sides: 20,
                    value: 20
                }
            ))
        )
        .is_err()
    );
    assert_eq!(serde_json::to_vec(&f.state).unwrap(), before);
    f.run(Some(f.target), |s, m| {
        super::super::continuations::submit(
            s,
            m,
            &result,
            Some((
                0,
                DieResult {
                    sides: 20,
                    value: 20,
                },
            )),
        )
    });
    let raw = f.state.rules.as_ref().unwrap().rolls.last().unwrap();
    assert_eq!(raw.original_result, Some(result));
    assert_eq!(raw.result.dice[0].value, 20);
    assert!(!f.state.rules.as_ref().unwrap().entities[&f.target].heroic_inspiration);
    assert!(matches!(
        attempt(&f.state).unwrap().outcome,
        Some(GrappleAttemptOutcome::Resisted { .. })
    ));
    f.decline(id);
}

#[test]
fn source_escape_failure_success_and_idle_release_preserve_the_paid_action() {
    for succeeds in [false, true] {
        let mut f = Fixture::new();
        let id = f.begin(Hand::Left, None);
        f.choose(id, GrappleSaveAbility::Strength);
        f.submit(1);
        f.decline(id);
        assert!(flow(&f.state).unwrap().resolution.is_none());
        f.run(Some(f.human), |s, m| {
            super::super::turns::core_action(s, m, &TacticalAction::EndTurn)
        });
        f.run(Some(f.target), |s, m| {
            begin_escape(s, m, id, GrappleEscapeChoice::Acrobatics)
        });
        assert_eq!(
            escape(&f.state).unwrap().request.as_ref().unwrap().modifier,
            2
        );
        let mut athletics = escape(&f.state).unwrap().clone();
        athletics.choice = GrappleEscapeChoice::Athletics;
        assert_eq!(
            saves::escape_request(&f.state, &athletics)
                .unwrap()
                .modifier,
            -1
        );

        let before = serde_json::to_vec(&f.state).unwrap();
        let foreign = f.meta(Some(f.target));
        assert!(release(&mut f.state, &foreign, id).is_err());
        assert_eq!(serde_json::to_vec(&f.state).unwrap(), before);
        f.submit(if succeeds { 20 } else { 1 });
        assert!(flow(&f.state).unwrap().resolution.is_none());
        assert_eq!(
            f.state.rules.as_ref().unwrap().tactical_grapples.is_none(),
            succeeds
        );
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
        if !succeeds {
            f.run(Some(f.human), |s, m| release(s, m, id));
            assert!(f.state.rules.as_ref().unwrap().tactical_grapples.is_none());
        }
    }
}

#[test]
fn release_refuses_an_unimplemented_consumer_and_stale_commands_atomically() {
    let mut f = Fixture::new();
    let id = f.begin(Hand::Left, None);
    f.choose(id, GrappleSaveAbility::Strength);
    f.submit(1);
    let stale = f.meta(Some(f.human));
    f.state.applied_event_sequence += 1;
    let before = serde_json::to_vec(&f.state).unwrap();
    assert!(release(&mut f.state, &stale, id).is_err());
    assert_eq!(serde_json::to_vec(&f.state).unwrap(), before);
    // A real work kind in an explicitly forged mixed-consumer image. It is a
    // refusal control, not an admitted movement/attack positive.
    resolution_mut(&mut f.state)
        .unwrap()
        .frames
        .push(vec![TacticalWorkItem {
            occurrence: 99,
            kind: TacticalWorkKind::AttackRoll,
        }]);
    let meta = f.meta(Some(f.human));
    let before = serde_json::to_vec(&f.state).unwrap();
    assert!(
        release(&mut f.state, &meta, id)
            .unwrap_err()
            .to_string()
            .contains("temporal consumer")
    );
    assert_eq!(serde_json::to_vec(&f.state).unwrap(), before);
}

#[test]
fn second_paid_grip_and_selected_escape_leave_the_other_incoming_relation() {
    let mut f = Fixture::new();
    let dagger = f.item("dagger");
    let first = f.begin(Hand::Left, None);
    f.choose(first, GrappleSaveAbility::Strength);
    f.submit(1);
    let work = work_key(
        &f.state,
        attempt(&f.state).unwrap().selected.as_ref().unwrap(),
    )
    .unwrap();
    f.run(Some(f.human), |s, m| {
        apply_after_equipment(
            s,
            m,
            first,
            work,
            AttackEquipmentOperation::Unequip { item: dagger },
        )
    });
    f.run(Some(f.human), |s, m| {
        super::super::turns::core_action(s, m, &TacticalAction::EndTurn)
    });
    f.run(Some(f.target), |s, m| {
        super::super::turns::core_action(s, m, &TacticalAction::EndTurn)
    });
    let second = f.begin(Hand::Right, None);
    f.choose(second, GrappleSaveAbility::Strength);
    f.submit(1);
    f.decline(second);
    assert_eq!(
        f.state
            .rules
            .as_ref()
            .unwrap()
            .tactical_grapples
            .as_ref()
            .unwrap()
            .active
            .len(),
        2
    );
    f.run(Some(f.human), |s, m| {
        super::super::turns::core_action(s, m, &TacticalAction::EndTurn)
    });
    f.run(Some(f.target), |s, m| {
        begin_escape(s, m, second, GrappleEscapeChoice::Acrobatics)
    });
    f.submit(20);
    let live = f
        .state
        .rules
        .as_ref()
        .unwrap()
        .tactical_grapples
        .as_ref()
        .unwrap();
    assert_eq!(live.active.len(), 1);
    assert_eq!(live.active[0].declaration.id, first);
    let hands = crate::tactical_hands::EffectiveHands::current(
        &f.state,
        f.state.rules.as_ref().unwrap(),
        f.human,
    )
    .unwrap();
    assert!(hands.is_reserved(Hand::Left));
    assert!(!hands.is_reserved(Hand::Right));
}

#[test]
fn actual_non_hover_flight_loss_is_refused_before_private_payment() {
    let mut f = Fixture::new();
    f.replace_target("young-red-dragon", CreatureSize::Large);
    // Explicit initial geometry scaffold for the installed non-Hover source.
    f.state
        .encounter
        .as_mut()
        .unwrap()
        .participants
        .iter_mut()
        .find(|p| p.entity_id == f.target)
        .unwrap()
        .position
        .z = 10;
    let meta = f.meta(Some(f.human));
    let before = serde_json::to_vec(&f.state).unwrap();
    assert!(begin(&mut f.state, &meta, f.target, Hand::Left, None, &f.pack).is_err());
    assert_eq!(serde_json::to_vec(&f.state).unwrap(), before);
}

#[test]
fn prepayment_source_item_reach_and_action_refusals_leave_the_exact_input() {
    for case in 0..5 {
        let mut f = Fixture::new();
        let dagger = f.item("dagger");
        match case {
            0 => f.state.items.get_mut(&dagger).unwrap().custody = Custody::Entity(f.target),
            1 => f.state.items.get_mut(&dagger).unwrap().state = ItemState::Damaged,
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
                f.state
                    .encounter
                    .as_mut()
                    .unwrap()
                    .participants
                    .iter_mut()
                    .find(|p| p.entity_id == f.target)
                    .unwrap()
                    .position
                    .x = 70
            }
            _ => {
                f.state
                    .rules
                    .as_mut()
                    .unwrap()
                    .tactical_creatures
                    .as_mut()
                    .unwrap()
                    .profiles[0]
                    .source
                    .definition_fingerprint = "0000000000000000".into()
            }
        }
        let meta = f.meta(Some(f.human));
        let before = serde_json::to_vec(&f.state).unwrap();
        assert!(
            begin(
                &mut f.state,
                &meta,
                f.target,
                Hand::Right,
                Some(AttackEquipmentOperation::Unequip { item: dagger }),
                &f.pack
            )
            .is_err(),
            "case {case}"
        );
        assert_eq!(serde_json::to_vec(&f.state).unwrap(), before, "case {case}");
    }
}

#[test]
fn source_dexterity_cover_and_condition_mode_feed_the_actual_request() {
    let mut f = Fixture::new();
    // Explicit initial imported condition/geometry; actual source statistics and
    // physical two-d20 continuation are exercised after this scaffold.
    f.state
        .rules
        .as_mut()
        .unwrap()
        .entities
        .get_mut(&f.target)
        .unwrap()
        .exhaustion = 2;
    f.state.rules.as_mut().unwrap().effects.push(ActiveEffect {
        id: EffectId::new(),
        source: f.human,
        target: f.target,
        condition: Some(Condition::Restrained),
        label: "Synthetic initial restraint".into(),
        expires: Expiry::Never,
        concentration_owner: None,
    });
    f.state
        .encounter
        .as_mut()
        .unwrap()
        .battlefield
        .obstacles
        .push(SpatialObstacle {
            id: "authored-low-cover".into(),
            volume: SpatialBox {
                min: SpatialPoint { x: 18, y: 0, z: 0 },
                max: SpatialPoint {
                    x: 19,
                    y: 30,
                    z: 15,
                },
            },
            blocks_movement: false,
            blocks_sight: false,
            observable: true,
            cover: CoverDegree::Half,
        });
    let id = f.begin(Hand::Left, None);
    f.choose(id, GrappleSaveAbility::Dexterity);
    let p = f.state.rules.as_ref().unwrap().pending.as_ref().unwrap();
    assert_eq!(
        p.request.modifier, 0,
        "Dexterity2, Exhaustion-4, Half cover+2"
    );
    assert_eq!(p.request.mode, RollMode::Disadvantage);
    f.submit(1);
    assert_eq!(
        f.state
            .rules
            .as_ref()
            .unwrap()
            .rolls
            .last()
            .unwrap()
            .result
            .dice
            .len(),
        2
    );
    f.decline(id);
}
