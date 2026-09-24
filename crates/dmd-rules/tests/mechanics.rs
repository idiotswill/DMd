use dmd_domain::*;
use dmd_rules::*;

fn ruling() -> Ruling {
    Ruling {
        basis: RulingBasis::GmAdjudication,
        reason: "Verified encounter context and adjudicated prerequisite".into(),
    }
}
fn pack() -> RulesPack {
    RulesPack::from_json(include_str!("../../../content/srd-5.2.1/kernel.json")).unwrap()
}
struct Table {
    state: CampaignState,
    pack: RulesPack,
    actor: EntityId,
    target: EntityId,
    player: PlayerId,
    origin: CampaignState,
    events: Vec<RulesEvent>,
}
impl Table {
    fn new() -> Self {
        Self::custom(|_, _| {})
    }
    fn custom(change: impl FnOnce(&mut MechanicalEntity, &mut MechanicalEntity)) -> Self {
        let campaign = Campaign {
            id: CampaignId::new(),
            display_name: "Kernel fixture".into(),
            status: CampaignStatus::Active,
            world_seed: 7,
            ruleset: VersionedRef {
                id: "srd-5.2".into(),
                version: "5.2.1".into(),
            },
            content_packs: vec![],
        };
        let mut state = CampaignState::empty(
            campaign,
            WorldClock {
                now: WorldInstant(0),
                calendar_id: "test".into(),
            },
        );
        let actor = EntityId::new();
        let target = EntityId::new();
        let player = PlayerId::new();
        for id in [actor, target] {
            state.entities.insert(
                id,
                WorldEntity {
                    id,
                    campaign_id: state.campaign_id(),
                    display_name: "Unnamed".into(),
                    kind: if id == actor {
                        EntityKind::Character
                    } else {
                        EntityKind::Creature
                    },
                    existence: EntityExistence::Present,
                    location_id: None,
                },
            );
        }
        state.players.insert(
            player,
            Player {
                id: player,
                campaign_id: state.campaign_id(),
                display_name: "Controller".into(),
            },
        );
        let character = Character {
            id: CharacterId::new(),
            entity_id: actor,
            campaign_id: state.campaign_id(),
            controlling_player_id: Some(player),
            display_name: "PC".into(),
            status: CharacterStatus::Active,
        };
        state.characters.insert(character.id, character);
        let mut pc = MechanicalEntity::basic(actor);
        pc.level = 5;
        pc.ability_scores = [16, 14, 12, 10, 15, 8];
        pc.max_hp = 40;
        pc.hp = 40;
        pc.hit_dice.maximum = 5;
        pc.hit_dice.remaining = 5;
        pc.attacks.extend(["dagger".into(), "shortbow".into()]);
        pc.attack_proficiencies.insert("dagger".into());
        pc.skill_proficiencies
            .insert(Skill::Perception, Proficiency::Expertise);
        pc.saving_proficiencies.insert(Ability::Constitution);
        pc.prepared_spells.extend([
            "cure-wounds".into(),
            "fire-bolt".into(),
            "dancing-lights".into(),
        ]);
        pc.spellcasting = Some(Spellcasting {
            ability: Ability::Wisdom,
            slot_maxima: [4, 3, 2, 0, 0, 0, 0, 0, 0],
            slots: [4, 3, 2, 0, 0, 0, 0, 0, 0],
            can_speak: true,
            free_hand: true,
            material_focus: true,
        });
        pc.resources.insert(
            "feature".into(),
            ResourcePool {
                maximum: 3,
                remaining: 3,
                recovery: Recovery::ShortOrLongRest,
            },
        );
        let mut npc = MechanicalEntity::basic(target);
        npc.max_hp = 30;
        npc.hp = 30;
        npc.armor = ArmorClass::Fixed(15);
        change(&mut pc, &mut npc);
        let origin = state.clone();
        let mut table = Self {
            state,
            pack: pack(),
            actor,
            target,
            player,
            origin,
            events: vec![],
        };
        table.admin(RulesAction::Initialize {
            entities: vec![pc, npc],
            house_rules: HouseRules::default(),
            ruling: ruling(),
        });
        table
    }
    fn meta(&self, issuer: CommandIssuer, actor: Option<EntityId>) -> CommandMeta {
        CommandMeta {
            id: CommandId::new(),
            campaign_id: self.state.campaign_id(),
            session_id: None,
            issuer,
            actor: actor.map(AgentRef::Entity),
            expected_event_sequence: self.state.applied_event_sequence,
        }
    }
    fn step(
        &mut self,
        issuer: CommandIssuer,
        actor: Option<EntityId>,
        action: RulesAction,
    ) -> RulesOutcome {
        let meta = self.meta(issuer, actor);
        let transition = resolve(&self.state, &meta, &action, &self.pack)
            .unwrap_or_else(|e| panic!("{action:?}: {e}"));
        assert_eq!(
            transition.next_state.applied_event_sequence,
            self.state.applied_event_sequence
        );
        self.events.push(transition.event);
        self.state = transition.next_state;
        self.state.applied_event_sequence += 1;
        transition.outcome
    }
    fn admin(&mut self, action: RulesAction) -> RulesOutcome {
        self.step(CommandIssuer::Admin, None, action)
    }
    fn player(&mut self, action: RulesAction) -> RulesOutcome {
        self.step(CommandIssuer::Player(self.player), Some(self.actor), action)
    }
    fn request(
        &mut self,
        kind: TestKind,
        dc: i32,
        c: Circumstances,
        visibility: RollVisibility,
    ) -> RollRequest {
        let actor = self.actor;
        let out = self.admin(RulesAction::RequestTest {
            actor,
            kind,
            dc,
            visibility,
            circumstances: c,
            ruling: ruling(),
            request_id: RollRequestId::new(),
        });
        match out {
            RulesOutcome::RollRequested(r) => r,
            _ => panic!("expected roll"),
        }
    }
    fn result(&self, values: &[u16], source: RollSource) -> RollResult {
        let request = &self.rules().pending.as_ref().unwrap().request;
        let sides: Vec<_> = if request.mode == RollMode::Normal {
            request
                .dice
                .iter()
                .flat_map(|s| std::iter::repeat_n(s.sides, usize::from(s.count)))
                .collect()
        } else {
            vec![20, 20]
        };
        assert_eq!(values.len(), sides.len());
        RollResult {
            request_id: request.id,
            source,
            dice: sides
                .into_iter()
                .zip(values)
                .map(|(sides, value)| DieResult {
                    sides,
                    value: *value,
                })
                .collect(),
        }
    }
    fn faces(&mut self, values: &[u16]) -> RulesOutcome {
        let result = self.result(values, RollSource::Physical);
        self.player(RulesAction::SubmitRoll { result })
    }
    fn rules(&self) -> &RulesState {
        self.state.rules.as_ref().unwrap()
    }
    fn pc(&self) -> &MechanicalEntity {
        &self.rules().entities[&self.actor]
    }
    fn target(&self) -> &MechanicalEntity {
        &self.rules().entities[&self.target]
    }
    fn attack(&mut self) {
        self.admin(RulesAction::AuthorizeAttack {
            actor: self.actor,
            target: self.target,
            attack_id: "dagger".into(),
            circumstances: Circumstances::default(),
            within_five_feet: true,
            ruling: ruling(),
        });
        self.player(RulesAction::Attack {
            actor: self.actor,
            target: self.target,
            attack_id: "dagger".into(),
            request_id: RollRequestId::new(),
        });
    }
    fn spell(&mut self, id: &str, slot: u8) {
        self.admin(RulesAction::AuthorizeSpell {
            actor: self.actor,
            target: self.target,
            spell_id: id.into(),
            circumstances: Circumstances::default(),
            within_five_feet: false,
            ruling: ruling(),
        });
        self.player(RulesAction::CastSpell {
            actor: self.actor,
            target: self.target,
            spell_id: id.into(),
            slot_level: slot,
            request_id: RollRequestId::new(),
            effect_id: EffectId::new(),
        });
    }
    fn replay(&self) {
        let mut state = self.origin.clone();
        for event in &self.events {
            let transition = replay(&state, event, &self.pack).unwrap();
            state = transition.next_state;
            state.applied_event_sequence += 1;
        }
        assert_eq!(state, self.state);
    }
    fn fails(&self, issuer: CommandIssuer, actor: Option<EntityId>, action: RulesAction) {
        let before = self.state.clone();
        assert!(resolve(&self.state, &self.meta(issuer, actor), &action, &self.pack).is_err());
        assert_eq!(before, self.state);
    }
}
fn check() -> TestKind {
    TestKind::Check {
        ability: Ability::Strength,
        skill: None,
    }
}

#[test]
fn derived_modifiers_and_armor_round_negative_scores_correctly() {
    for (score, expected) in [(1, -5), (8, -1), (9, -1), (10, 0), (19, 4), (30, 10)] {
        assert_eq!(ability_modifier(score), expected);
    }
    for (level, expected) in [
        (1, 2),
        (4, 2),
        (5, 3),
        (8, 3),
        (9, 4),
        (13, 5),
        (17, 6),
        (20, 6),
    ] {
        assert_eq!(proficiency_bonus(level), expected);
    }
    let t = Table::custom(|pc, _| {
        pc.ability_scores[1] = 8;
        pc.armor = ArmorClass::Armor {
            base: 14,
            dexterity_cap: Some(2),
            shield: true,
        };
    });
    assert_eq!(armor_class(t.pc()), 15);
    assert_eq!(
        query(
            &t.state,
            CommandIssuer::Player(t.player),
            &RulesQuery::PassivePerception {
                actor: t.actor,
                circumstances: Circumstances::default()
            },
            &t.pack
        )
        .unwrap(),
        RulesAnswer::PassivePerception(18)
    );
}

#[test]
fn checks_and_saves_do_not_treat_natural_extremes_as_attacks() {
    let mut t = Table::new();
    t.request(
        check(),
        30,
        Circumstances::default(),
        RollVisibility::Public,
    );
    assert!(matches!(
        t.faces(&[20]),
        RulesOutcome::RollResolved {
            success: Some(false),
            critical: false,
            ..
        }
    ));
    t.request(
        TestKind::Save {
            ability: Ability::Constitution,
        },
        5,
        Circumstances::default(),
        RollVisibility::Public,
    );
    assert!(matches!(
        t.faces(&[1]),
        RulesOutcome::RollResolved {
            success: Some(true),
            ..
        }
    ));
    t.replay();
}

#[test]
fn poison_and_advantage_cancel_and_keep_raw_faces_when_uncancelled() {
    let mut t = Table::new();
    let effect = ActiveEffect {
        id: EffectId::new(),
        source: t.target,
        target: t.actor,
        condition: Some(Condition::Poisoned),
        label: "poison".into(),
        expires: Expiry::Never,
        concentration_owner: None,
    };
    t.admin(RulesAction::ApplyEffect {
        effect,
        ruling: ruling(),
    });
    let request = t.request(
        check(),
        12,
        Circumstances {
            advantage: true,
            disadvantage: false,
            ranged_threat: false,
        },
        RollVisibility::Public,
    );
    assert_eq!(request.mode, RollMode::Normal);
    t.faces(&[9]);
    t.request(
        check(),
        12,
        Circumstances::default(),
        RollVisibility::Public,
    );
    let result = t.faces(&[18, 3]);
    assert!(matches!(
        result,
        RulesOutcome::RollResolved {
            roll: ResolvedRoll { total: 6, .. },
            success: Some(false),
            ..
        }
    ));
    assert_eq!(t.rules().rolls.last().unwrap().resolved.raw_dice.len(), 2);
    t.replay();
}

#[test]
fn unauthorized_stale_impossible_and_pending_actions_never_mutate() {
    let mut t = Table::new();
    let action = RulesAction::RequestTest {
        actor: t.actor,
        kind: check(),
        dc: 10,
        visibility: RollVisibility::Public,
        circumstances: Circumstances::default(),
        ruling: ruling(),
        request_id: RollRequestId::new(),
    };
    t.fails(
        CommandIssuer::Player(t.player),
        Some(t.actor),
        action.clone(),
    );
    let mut meta = t.meta(CommandIssuer::Admin, None);
    meta.expected_event_sequence = 0;
    assert_eq!(
        resolve(&t.state, &meta, &action, &t.pack),
        Err(RulesError::Stale)
    );
    t.admin(action);
    t.fails(
        CommandIssuer::Admin,
        None,
        RulesAction::Heal {
            target: t.actor,
            amount: 2,
            ruling: ruling(),
        },
    );
    let invalid = t.result(&[21], RollSource::Physical);
    t.fails(
        CommandIssuer::Player(t.player),
        Some(t.actor),
        RulesAction::SubmitRoll { result: invalid },
    );
    let digital = t.result(&[10], RollSource::Digital);
    t.fails(
        CommandIssuer::Player(t.player),
        Some(t.actor),
        RulesAction::SubmitRoll {
            result: digital.clone(),
        },
    );
    t.fails(
        CommandIssuer::System,
        Some(t.target),
        RulesAction::SubmitRoll {
            result: digital.clone(),
        },
    );
    t.step(
        CommandIssuer::System,
        Some(t.actor),
        RulesAction::SubmitRoll { result: digital },
    );
    t.replay();
}

#[test]
fn attacks_require_context_critical_doubles_only_dice_and_natural_one_misses() {
    let mut t = Table::new();
    t.fails(
        CommandIssuer::Player(t.player),
        Some(t.actor),
        RulesAction::Attack {
            actor: t.actor,
            target: t.target,
            attack_id: "dagger".into(),
            request_id: RollRequestId::new(),
        },
    );
    t.attack();
    assert_eq!(t.rules().pending.as_ref().unwrap().request.modifier, 6);
    let out = t.faces(&[20]);
    assert!(matches!(
        out,
        RulesOutcome::RollResolved {
            critical: true,
            success: Some(true),
            ..
        }
    ));
    let damage = &t.rules().pending.as_ref().unwrap().request;
    assert_eq!(damage.dice, vec![DieSpec { count: 2, sides: 4 }]);
    assert_eq!(damage.modifier, 3);
    t.faces(&[1, 4]);
    assert_eq!(t.target().hp, 22);
    t.attack();
    assert!(matches!(
        t.faces(&[1]),
        RulesOutcome::RollResolved {
            success: Some(false),
            ..
        }
    ));
    assert!(t.rules().pending.is_none());
    t.replay();
}

#[test]
fn resistance_then_vulnerability_temporary_hp_and_healing_are_distinct() {
    let mut t = Table::custom(|_, target| {
        target.resistances.insert(DamageType::Fire);
        target.vulnerabilities.insert(DamageType::Fire);
        target.temporary_hp = 3;
    });
    assert_eq!(
        t.admin(RulesAction::ApplyDamage {
            target: t.target,
            amount: 5,
            damage_type: DamageType::Fire,
            critical: false,
            ruling: ruling()
        }),
        RulesOutcome::Damage {
            applied: 4,
            followup: None
        }
    );
    assert_eq!(t.target().hp, 29);
    assert_eq!(t.target().temporary_hp, 0);
    t.admin(RulesAction::GrantTemporaryHp {
        target: t.target,
        amount: 5,
        ruling: ruling(),
    });
    t.admin(RulesAction::GrantTemporaryHp {
        target: t.target,
        amount: 4,
        ruling: ruling(),
    });
    assert_eq!(t.target().temporary_hp, 4);
    assert_eq!(
        t.admin(RulesAction::Heal {
            target: t.target,
            amount: 20,
            ruling: ruling()
        }),
        RulesOutcome::Healed { regained: 1 }
    );
    assert_eq!(t.target().temporary_hp, 4);
    t.replay();
}

#[test]
fn death_saves_and_massive_damage_update_world_lifecycle() {
    let mut t = Table::new();
    t.admin(RulesAction::ApplyDamage {
        target: t.actor,
        amount: 40,
        damage_type: DamageType::Force,
        critical: false,
        ruling: ruling(),
    });
    t.request(
        TestKind::DeathSave,
        99,
        Circumstances::default(),
        RollVisibility::Public,
    );
    t.faces(&[20]);
    assert_eq!(t.pc().hp, 1);
    t.admin(RulesAction::ApplyDamage {
        target: t.actor,
        amount: 1,
        damage_type: DamageType::Force,
        critical: false,
        ruling: ruling(),
    });
    t.request(
        TestKind::DeathSave,
        10,
        Circumstances::default(),
        RollVisibility::Public,
    );
    t.faces(&[1]);
    assert_eq!(t.pc().death.failures, 2);
    t.request(
        TestKind::DeathSave,
        10,
        Circumstances::default(),
        RollVisibility::Public,
    );
    t.faces(&[2]);
    assert!(t.pc().death.dead);
    assert_eq!(t.state.entities[&t.actor].existence, EntityExistence::Dead);
    assert!(matches!(
        query(
            &t.state,
            CommandIssuer::Player(t.player),
            &RulesQuery::Character { actor: t.actor },
            &t.pack
        ),
        Ok(RulesAnswer::Character { hp: 0, .. })
    ));
    t.replay();
    let mut massive = Table::new();
    massive.admin(RulesAction::ApplyDamage {
        target: massive.target,
        amount: 60,
        damage_type: DamageType::Force,
        critical: false,
        ruling: ruling(),
    });
    assert!(massive.target().death.dead);
}

#[test]
fn concentration_requires_a_save_uses_damage_before_temporary_hp_and_expires() {
    let mut t = Table::new();
    t.spell("dancing-lights", 0);
    let effect = t.pc().concentration.unwrap();
    t.admin(RulesAction::GrantTemporaryHp {
        target: t.actor,
        amount: 80,
        ruling: ruling(),
    });
    t.admin(RulesAction::ApplyDamage {
        target: t.actor,
        amount: 70,
        damage_type: DamageType::Force,
        critical: false,
        ruling: ruling(),
    });
    assert_eq!(t.pc().hp, 40);
    assert!(matches!(
        t.rules().pending.as_ref().unwrap().purpose,
        PendingPurpose::Concentration { dc: 30, .. }
    ));
    t.faces(&[20]);
    assert!(t.pc().concentration.is_none());
    assert!(!t.rules().effects.iter().any(|e| e.id == effect));
    t.spell("dancing-lights", 0);
    let previous = t.pc().concentration;
    t.spell("dancing-lights", 0);
    assert_ne!(previous, t.pc().concentration);
    assert_eq!(t.rules().effects.len(), 1);
    t.admin(RulesAction::AdvanceTime {
        seconds: 60,
        ruling: ruling(),
    });
    assert!(t.pc().concentration.is_none());
    assert!(t.rules().effects.is_empty());
    t.replay();
}

#[test]
fn spellcasting_derives_healing_upcast_and_cantrip_damage_with_slot_cost() {
    let mut t = Table::custom(|_, target| target.hp = 10);
    t.spell("cure-wounds", 2);
    assert_eq!(t.pc().spellcasting.as_ref().unwrap().slots[1], 2);
    assert_eq!(t.rules().pending.as_ref().unwrap().request.modifier, 2);
    t.faces(&[1, 2, 3, 4]);
    assert_eq!(t.target().hp, 22);
    t.spell("fire-bolt", 0);
    t.faces(&[20]);
    assert_eq!(
        t.rules().pending.as_ref().unwrap().request.dice,
        vec![DieSpec {
            count: 4,
            sides: 10
        }]
    );
    assert_eq!(t.rules().pending.as_ref().unwrap().request.modifier, 0);
    t.faces(&[1, 1, 1, 1]);
    assert_eq!(t.target().hp, 18);
    t.replay();
}

#[test]
fn spell_components_and_slots_fail_without_cost_or_mutation() {
    let mut t = Table::custom(|pc, _| pc.spellcasting.as_mut().unwrap().can_speak = false);
    t.admin(RulesAction::AuthorizeSpell {
        actor: t.actor,
        target: t.target,
        spell_id: "cure-wounds".into(),
        circumstances: Circumstances::default(),
        within_five_feet: false,
        ruling: ruling(),
    });
    t.fails(
        CommandIssuer::Player(t.player),
        Some(t.actor),
        RulesAction::CastSpell {
            actor: t.actor,
            target: t.target,
            spell_id: "cure-wounds".into(),
            slot_level: 1,
            request_id: RollRequestId::new(),
            effect_id: EffectId::new(),
        },
    );
}

#[test]
fn secret_rolls_and_queries_preserve_visibility_and_do_not_change_state() {
    let mut t = Table::new();
    t.request(
        check(),
        15,
        Circumstances::default(),
        RollVisibility::Secret,
    );
    let before = t.state.clone();
    assert_eq!(
        query(
            &t.state,
            CommandIssuer::Player(t.player),
            &RulesQuery::PendingRoll,
            &t.pack
        )
        .unwrap(),
        RulesAnswer::PendingRoll(None)
    );
    assert!(matches!(
        query(
            &t.state,
            CommandIssuer::Admin,
            &RulesQuery::PendingRoll,
            &t.pack
        )
        .unwrap(),
        RulesAnswer::PendingRoll(Some(_))
    ));
    assert_eq!(before, t.state);
    let physical = t.result(&[15], RollSource::Physical);
    t.fails(
        CommandIssuer::Player(t.player),
        Some(t.actor),
        RulesAction::SubmitRoll { result: physical },
    );
    let result = t.result(&[15], RollSource::Digital);
    t.step(
        CommandIssuer::System,
        None,
        RulesAction::SubmitRoll { result },
    );
    t.replay();
}

#[test]
fn inspiration_replaces_one_die_before_resolution_and_records_both_inputs() {
    let mut t = Table::new();
    t.admin(RulesAction::GrantInspiration {
        actor: t.actor,
        ruling: ruling(),
    });
    t.request(
        check(),
        16,
        Circumstances {
            advantage: true,
            disadvantage: false,
            ranged_threat: false,
        },
        RollVisibility::Public,
    );
    let result = t.result(&[2, 15], RollSource::Physical);
    let outcome = t.player(RulesAction::SubmitRollWithInspiration {
        result: result.clone(),
        die_index: 1,
        replacement: DieResult {
            sides: 20,
            value: 4,
        },
    });
    assert!(matches!(
        outcome,
        RulesOutcome::RollResolved {
            success: Some(false),
            roll: ResolvedRoll { total: 7, .. },
            ..
        }
    ));
    assert!(!t.pc().heroic_inspiration);
    assert_eq!(
        t.rules().rolls.last().unwrap().original_result,
        Some(result)
    );
    t.replay();
}

#[test]
fn rests_recover_resources_and_hit_dice_without_free_healing_or_repeated_long_rest() {
    let mut t = Table::custom(|pc, _| {
        pc.hp = 10;
        pc.exhaustion = 2;
    });
    t.player(RulesAction::SpendResource {
        actor: t.actor,
        resource_id: "feature".into(),
        amount: 3,
    });
    t.fails(
        CommandIssuer::Player(t.player),
        Some(t.actor),
        RulesAction::SpendResource {
            actor: t.actor,
            resource_id: "feature".into(),
            amount: 1,
        },
    );
    t.admin(RulesAction::StartRest {
        actor: t.actor,
        kind: RestKind::Short,
        ruling: ruling(),
    });
    t.fails(
        CommandIssuer::Admin,
        None,
        RulesAction::FinishRest {
            actor: t.actor,
            slept_seconds: 0,
            ruling: ruling(),
        },
    );
    t.admin(RulesAction::AdvanceTime {
        seconds: 3600,
        ruling: ruling(),
    });
    t.admin(RulesAction::FinishRest {
        actor: t.actor,
        slept_seconds: 0,
        ruling: ruling(),
    });
    assert_eq!(t.pc().hp, 10);
    assert_eq!(t.pc().resources["feature"].remaining, 3);
    t.player(RulesAction::SpendHitDie {
        actor: t.actor,
        request_id: RollRequestId::new(),
    });
    t.faces(&[6]);
    assert_eq!(t.pc().hp, 17);
    assert_eq!(t.pc().hit_dice.remaining, 4);
    t.admin(RulesAction::StartRest {
        actor: t.actor,
        kind: RestKind::Long,
        ruling: ruling(),
    });
    t.admin(RulesAction::AdvanceTime {
        seconds: 28800,
        ruling: ruling(),
    });
    t.admin(RulesAction::FinishRest {
        actor: t.actor,
        slept_seconds: 21600,
        ruling: ruling(),
    });
    assert_eq!(t.pc().hp, 40);
    assert_eq!(t.pc().hit_dice.remaining, 5);
    assert_eq!(t.pc().exhaustion, 1);
    t.fails(
        CommandIssuer::Admin,
        None,
        RulesAction::StartRest {
            actor: t.actor,
            kind: RestKind::Long,
            ruling: ruling(),
        },
    );
    t.replay();
}

#[test]
fn attack_and_damage_interrupt_rest_and_invalid_context_cannot_be_reused() {
    let mut t = Table::new();
    t.admin(RulesAction::StartRest {
        actor: t.actor,
        kind: RestKind::Short,
        ruling: ruling(),
    });
    t.attack();
    assert!(t.rules().rests.is_empty());
    t.faces(&[1]);
    t.fails(
        CommandIssuer::Admin,
        None,
        RulesAction::FinishRest {
            actor: t.actor,
            slept_seconds: 0,
            ruling: ruling(),
        },
    );
    t.admin(RulesAction::StartRest {
        actor: t.target,
        kind: RestKind::Short,
        ruling: ruling(),
    });
    t.admin(RulesAction::ApplyDamage {
        target: t.target,
        amount: 1,
        damage_type: DamageType::Force,
        critical: false,
        ruling: ruling(),
    });
    assert!(t.rules().rests.is_empty());
    t.replay();
}

#[test]
fn initiative_orders_recorded_rolls_and_refreshes_reactions_and_action_budget() {
    let mut t = Table::new();
    t.request(
        TestKind::Initiative,
        0,
        Circumstances::default(),
        RollVisibility::Public,
    );
    t.faces(&[12]);
    t.admin(RulesAction::RequestTest {
        actor: t.target,
        kind: TestKind::Initiative,
        dc: 0,
        visibility: RollVisibility::Public,
        circumstances: Circumstances::default(),
        ruling: ruling(),
        request_id: RollRequestId::new(),
    });
    let result = t.result(&[10], RollSource::Digital);
    t.step(
        CommandIssuer::System,
        None,
        RulesAction::SubmitRoll { result },
    );
    t.admin(RulesAction::StartCombat {
        participants: vec![
            InitiativeEntry {
                actor: t.target,
                total: 10,
                tie_break: 0,
            },
            InitiativeEntry {
                actor: t.actor,
                total: 14,
                tie_break: 0,
            },
        ],
        ruling: ruling(),
    });
    assert_eq!(t.rules().timing.as_ref().unwrap().order[0].actor, t.actor);
    let expiring = EffectId::new();
    t.admin(RulesAction::ApplyEffect {
        effect: ActiveEffect {
            id: expiring,
            source: t.actor,
            target: t.actor,
            condition: None,
            label: "turn marker".into(),
            expires: Expiry::AtTurn {
                actor: t.actor,
                boundary: TurnBoundary::Start,
                turn_number: 3,
            },
            concentration_owner: None,
        },
        ruling: ruling(),
    });
    t.fails(
        CommandIssuer::Admin,
        None,
        RulesAction::ApplyEffect {
            effect: ActiveEffect {
                id: EffectId::new(),
                source: t.actor,
                target: t.actor,
                condition: None,
                label: "wrong actor for turn".into(),
                expires: Expiry::AtTurn {
                    actor: t.actor,
                    boundary: TurnBoundary::Start,
                    turn_number: 2,
                },
                concentration_owner: None,
            },
            ruling: ruling(),
        },
    );
    t.admin(RulesAction::UseBonusAction {
        actor: t.actor,
        feature_id: "adjudicated-feature".into(),
        ruling: ruling(),
    });
    t.fails(
        CommandIssuer::Admin,
        None,
        RulesAction::UseBonusAction {
            actor: t.actor,
            feature_id: "adjudicated-feature".into(),
            ruling: ruling(),
        },
    );
    t.admin(RulesAction::UseReaction {
        actor: t.actor,
        trigger: "authoritative trigger".into(),
        ruling: ruling(),
    });
    t.fails(
        CommandIssuer::Admin,
        None,
        RulesAction::UseReaction {
            actor: t.actor,
            trigger: "duplicate".into(),
            ruling: ruling(),
        },
    );
    t.attack();
    t.faces(&[1]);
    t.admin(RulesAction::AuthorizeAttack {
        actor: t.actor,
        target: t.target,
        attack_id: "dagger".into(),
        circumstances: Circumstances::default(),
        within_five_feet: true,
        ruling: ruling(),
    });
    t.fails(
        CommandIssuer::Player(t.player),
        Some(t.actor),
        RulesAction::Attack {
            actor: t.actor,
            target: t.target,
            attack_id: "dagger".into(),
            request_id: RollRequestId::new(),
        },
    );
    t.player(RulesAction::EndTurn { actor: t.actor });
    t.admin(RulesAction::EndTurn { actor: t.target });
    assert_eq!(t.rules().timing.as_ref().unwrap().round, 2);
    assert!(!t.rules().timing.as_ref().unwrap().bonus_action_spent);
    assert!(!t.rules().effects.iter().any(|e| e.id == expiring));
    assert!(
        t.rules()
            .timing
            .as_ref()
            .unwrap()
            .reactions_spent
            .is_empty()
    );
    assert_eq!(t.state.clock.now, WorldInstant(6));
    t.replay();
}

#[test]
fn malformed_packs_states_and_replay_outcomes_fail_closed() {
    let mut p = pack();
    p.version = "unknown".into();
    assert!(p.validate().is_err());
    assert!(RulesPack::from_json("{\"id\":\"srd-5.2\"}").is_err());
    let mut value = serde_json::to_value(pack()).unwrap();
    value["arbitrary_behavior"] = true.into();
    assert!(RulesPack::from_json(&value.to_string()).is_err());
    let mut t = Table::new();
    t.request(
        check(),
        10,
        Circumstances::default(),
        RollVisibility::Public,
    );
    t.faces(&[8]);
    let mut event = t.events[0].clone();
    event.outcome = RulesOutcome::AutomaticTest { success: true };
    assert_eq!(
        replay(&t.origin, &event, &t.pack),
        Err(RulesError::ReplayMismatch)
    );
    let json = t.state.encode_json().unwrap();
    assert_eq!(CampaignState::decode_json(&json).unwrap(), t.state);
    let mut broken = t.state.clone();
    broken
        .rules
        .as_mut()
        .unwrap()
        .entities
        .get_mut(&t.actor)
        .unwrap()
        .hp = 99;
    assert!(validate_state(&broken, &t.pack).is_err());
    assert!(serde_json::from_str::<RulesEvent>("{\"action\":{\"Unknown\":{}}}").is_err());
    t.replay();
}

#[test]
fn cancelled_roll_ids_cannot_accept_old_faces_under_a_new_modifier() {
    let mut t = Table::new();
    let request = t.request(
        check(),
        10,
        Circumstances::default(),
        RollVisibility::Public,
    );
    t.admin(RulesAction::CancelRoll { ruling: ruling() });
    t.fails(
        CommandIssuer::Admin,
        None,
        RulesAction::RequestTest {
            actor: t.actor,
            kind: check(),
            dc: 20,
            visibility: RollVisibility::Public,
            circumstances: Circumstances::default(),
            ruling: ruling(),
            request_id: request.id,
        },
    );
}

#[test]
fn restored_requests_rederive_dice_modifiers_targets_and_causes() {
    let mut t = Table::new();
    t.request(
        check(),
        12,
        Circumstances::default(),
        RollVisibility::Public,
    );
    let mut bad = t.state.clone();
    bad.rules
        .as_mut()
        .unwrap()
        .pending
        .as_mut()
        .unwrap()
        .request
        .modifier = 999;
    assert!(validate_state(&bad, &t.pack).is_err());
    let mut bad = t.state.clone();
    bad.rules
        .as_mut()
        .unwrap()
        .pending
        .as_mut()
        .unwrap()
        .purpose = PendingPurpose::Concentration {
        dc: 10,
        damage_taken: 4,
    };
    assert!(validate_state(&bad, &t.pack).is_err());
    let mut bad = t.state.clone();
    bad.rules
        .as_mut()
        .unwrap()
        .pending
        .as_mut()
        .unwrap()
        .purpose = PendingPurpose::Test {
        kind: TestKind::DeathSave,
        dc: 10,
        circumstances: Circumstances::default(),
    };
    assert!(validate_state(&bad, &t.pack).is_err());
    let mut bad = t.state.clone();
    let pending = bad.rules.as_mut().unwrap().pending.as_mut().unwrap();
    pending.purpose = PendingPurpose::Damage {
        target: t.target,
        damage_type: DamageType::Force,
        critical: false,
        attack_roll_id: RollRequestId::new(),
    };
    assert!(validate_state(&bad, &t.pack).is_err());
    t.faces(&[10]);
    t.attack();
    t.faces(&[20]);
    let mut bad = t.state.clone();
    let r = bad.rules.as_mut().unwrap();
    r.pending.as_mut().unwrap().request.dice[0].count = 200;
    if let PendingPurpose::Attack { damage, .. } = &mut r.rolls.last_mut().unwrap().purpose {
        damage[0].count = 100;
    }
    assert!(validate_state(&bad, &t.pack).is_err());
    let mut bad = t.state.clone();
    bad.rules
        .as_mut()
        .unwrap()
        .pending
        .as_mut()
        .unwrap()
        .issued_by
        .issuer = CommandIssuer::Import;
    assert!(validate_state(&bad, &t.pack).is_err());
    t.faces(&[2, 2]);
    t.replay();
}

#[test]
fn restored_permissions_and_ruling_provenance_cannot_gain_privilege() {
    let mut t = Table::new();
    t.admin(RulesAction::AuthorizeAttack {
        actor: t.actor,
        target: t.target,
        attack_id: "dagger".into(),
        circumstances: Circumstances::default(),
        within_five_feet: true,
        ruling: ruling(),
    });
    let mut bad = t.state.clone();
    bad.rules
        .as_mut()
        .unwrap()
        .permission
        .as_mut()
        .unwrap()
        .issued_by
        .issuer = CommandIssuer::Player(t.player);
    assert!(validate_state(&bad, &t.pack).is_err());
    let mut bad = t.state.clone();
    bad.rules.as_mut().unwrap().rulings[0]
        .command
        .expected_event_sequence = u64::MAX;
    assert!(validate_state(&bad, &t.pack).is_err());
    let mut bad = t.state.clone();
    let old = bad.rules.as_ref().unwrap().rulings[0].clone();
    bad.rules.as_mut().unwrap().rulings.push(old);
    assert!(validate_state(&bad, &t.pack).is_err());
    let mut bad = t.state.clone();
    bad.rules
        .as_mut()
        .unwrap()
        .entities
        .get_mut(&t.actor)
        .unwrap()
        .hp = 0;
    assert!(validate_state(&bad, &t.pack).is_err());
}

#[test]
fn restored_cancelled_and_reroll_records_reject_inconsistent_input() {
    let mut t = Table::new();
    t.admin(RulesAction::GrantInspiration {
        actor: t.actor,
        ruling: ruling(),
    });
    t.request(
        check(),
        12,
        Circumstances {
            advantage: true,
            ..Circumstances::default()
        },
        RollVisibility::Public,
    );
    let result = t.result(&[3, 4], RollSource::Physical);
    t.player(RulesAction::SubmitRollWithInspiration {
        result,
        die_index: 0,
        replacement: DieResult {
            sides: 20,
            value: 10,
        },
    });
    let mut bad = t.state.clone();
    bad.rules
        .as_mut()
        .unwrap()
        .rolls
        .last_mut()
        .unwrap()
        .original_result
        .as_mut()
        .unwrap()
        .dice[1]
        .value = 5;
    assert!(validate_state(&bad, &t.pack).is_err());
    let mut bad = t.state.clone();
    let id = bad.rules.as_ref().unwrap().rolls.last().unwrap().request.id;
    bad.rules.as_mut().unwrap().cancelled_roll_ids.push(id);
    assert!(validate_state(&bad, &t.pack).is_err());
    let mut bad = t.state.clone();
    bad.rules
        .as_mut()
        .unwrap()
        .rolls
        .last_mut()
        .unwrap()
        .accepted_by
        .actor = Some(AgentRef::Entity(t.target));
    assert!(validate_state(&bad, &t.pack).is_err());
}

#[test]
fn close_spell_attacks_use_explicit_threat_and_target_distance() {
    let mut t = Table::new();
    t.admin(RulesAction::ApplyEffect {
        effect: ActiveEffect {
            id: EffectId::new(),
            source: t.actor,
            target: t.target,
            condition: Some(Condition::Paralyzed),
            label: "paralysis".into(),
            expires: Expiry::Never,
            concentration_owner: None,
        },
        ruling: ruling(),
    });
    t.admin(RulesAction::AuthorizeSpell {
        actor: t.actor,
        target: t.target,
        spell_id: "fire-bolt".into(),
        circumstances: Circumstances::default(),
        within_five_feet: true,
        ruling: ruling(),
    });
    t.player(RulesAction::CastSpell {
        actor: t.actor,
        target: t.target,
        spell_id: "fire-bolt".into(),
        slot_level: 0,
        request_id: RollRequestId::new(),
        effect_id: EffectId::new(),
    });
    assert_eq!(
        t.rules().pending.as_ref().unwrap().request.mode,
        RollMode::Advantage
    );
    assert!(matches!(
        t.faces(&[15, 12]),
        RulesOutcome::RollResolved { critical: true, .. }
    ));
    t.faces(&[1, 1, 1, 1]);
    t.replay();
    let mut blinded = Table::new();
    blinded.admin(RulesAction::ApplyEffect {
        effect: ActiveEffect {
            id: EffectId::new(),
            source: blinded.actor,
            target: blinded.target,
            condition: Some(Condition::Blinded),
            label: "blindness".into(),
            expires: Expiry::Never,
            concentration_owner: None,
        },
        ruling: ruling(),
    });
    for (ranged_threat, expected_mode) in [(false, RollMode::Advantage), (true, RollMode::Normal)] {
        blinded.admin(RulesAction::AuthorizeSpell {
            actor: blinded.actor,
            target: blinded.target,
            spell_id: "fire-bolt".into(),
            circumstances: Circumstances {
                ranged_threat,
                ..Circumstances::default()
            },
            within_five_feet: true,
            ruling: ruling(),
        });
        blinded.player(RulesAction::CastSpell {
            actor: blinded.actor,
            target: blinded.target,
            spell_id: "fire-bolt".into(),
            slot_level: 0,
            request_id: RollRequestId::new(),
            effect_id: EffectId::new(),
        });
        assert_eq!(
            blinded.rules().pending.as_ref().unwrap().request.mode,
            expected_mode
        );
        // The adjacent blinded target cannot see the caster. Only a separately
        // adjudicated seeing enemy cancels the advantage against that target.
        blinded.faces(if ranged_threat { &[1] } else { &[1, 1] });
    }
    blinded.replay();
}

#[test]
fn healing_preserves_prone_and_heavy_armor_ignores_negative_dexterity() {
    let mut t = Table::custom(|pc, _| {
        pc.ability_scores[1] = 8;
        pc.armor = ArmorClass::HeavyArmor {
            base: 16,
            shield: true,
        };
    });
    assert_eq!(armor_class(t.pc()), 18);
    t.admin(RulesAction::ApplyDamage {
        target: t.actor,
        amount: 40,
        damage_type: DamageType::Force,
        critical: false,
        ruling: ruling(),
    });
    t.admin(RulesAction::Heal {
        target: t.actor,
        amount: 2,
        ruling: ruling(),
    });
    assert!(t.pc().prone);
    t.admin(RulesAction::SetProne {
        target: t.actor,
        prone: false,
        ruling: ruling(),
    });
    assert!(!t.pc().prone);
    t.replay();
}

#[test]
fn initiative_conditions_and_rest_interruptions_follow_srd() {
    let mut t = Table::new();
    t.admin(RulesAction::StartRest {
        actor: t.actor,
        kind: RestKind::Short,
        ruling: ruling(),
    });
    t.request(
        TestKind::Initiative,
        0,
        Circumstances::default(),
        RollVisibility::Public,
    );
    assert!(t.rules().rests.is_empty());
    t.faces(&[12]);
    let invisible_id = EffectId::new();
    for (condition, effect_id, expected_mode) in [
        (Condition::Invisible, invisible_id, RollMode::Advantage),
        (
            Condition::Incapacitated,
            EffectId::new(),
            RollMode::Disadvantage,
        ),
        (Condition::Invisible, EffectId::new(), RollMode::Normal),
    ] {
        t.admin(RulesAction::ApplyEffect {
            effect: ActiveEffect {
                id: effect_id,
                source: t.target,
                target: t.actor,
                condition: Some(condition),
                label: "timing condition".into(),
                expires: Expiry::Never,
                concentration_owner: None,
            },
            ruling: ruling(),
        });
        let request = t.request(
            TestKind::Initiative,
            0,
            Circumstances::default(),
            RollVisibility::Public,
        );
        assert_eq!(request.mode, expected_mode);
        t.faces(if expected_mode == RollMode::Normal {
            &[12]
        } else {
            &[3, 12]
        });
        if effect_id == invisible_id {
            t.admin(RulesAction::RemoveEffect {
                effect_id,
                ruling: ruling(),
            });
        }
    }
    t.replay();
}

#[test]
fn zero_damage_does_not_interrupt_rest_and_old_rest_cannot_heal_later() {
    let mut t = Table::custom(|pc, _| {
        pc.hp = 10;
        pc.damage_immunities.insert(DamageType::Fire);
    });
    t.admin(RulesAction::StartRest {
        actor: t.actor,
        kind: RestKind::Short,
        ruling: ruling(),
    });
    t.admin(RulesAction::ApplyDamage {
        target: t.actor,
        amount: 40,
        damage_type: DamageType::Fire,
        critical: false,
        ruling: ruling(),
    });
    assert_eq!(t.rules().rests.len(), 1);
    t.admin(RulesAction::AdvanceTime {
        seconds: 3600,
        ruling: ruling(),
    });
    t.admin(RulesAction::FinishRest {
        actor: t.actor,
        slept_seconds: 0,
        ruling: ruling(),
    });
    t.admin(RulesAction::AdvanceTime {
        seconds: 1,
        ruling: ruling(),
    });
    t.fails(
        CommandIssuer::Player(t.player),
        Some(t.actor),
        RulesAction::SpendHitDie {
            actor: t.actor,
            request_id: RollRequestId::new(),
        },
    );
    t.replay();
}

#[test]
fn petrification_rejects_poisoned_and_expiry_is_tied_to_real_turn_boundaries() {
    let mut t = Table::new();
    t.admin(RulesAction::ApplyEffect {
        effect: ActiveEffect {
            id: EffectId::new(),
            source: t.target,
            target: t.actor,
            condition: Some(Condition::Petrified),
            label: "stone".into(),
            expires: Expiry::Never,
            concentration_owner: None,
        },
        ruling: ruling(),
    });
    t.fails(
        CommandIssuer::Admin,
        None,
        RulesAction::ApplyEffect {
            effect: ActiveEffect {
                id: EffectId::new(),
                source: t.target,
                target: t.actor,
                condition: Some(Condition::Poisoned),
                label: "poison".into(),
                expires: Expiry::Never,
                concentration_owner: None,
            },
            ruling: ruling(),
        },
    );
    t.fails(
        CommandIssuer::Admin,
        None,
        RulesAction::ApplyEffect {
            effect: ActiveEffect {
                id: EffectId::new(),
                source: t.target,
                target: t.actor,
                condition: None,
                label: "invalid turn".into(),
                expires: Expiry::AtTurn {
                    actor: t.actor,
                    boundary: TurnBoundary::End,
                    turn_number: 1,
                },
                concentration_owner: None,
            },
            ruling: ruling(),
        },
    );
    t.replay();
}

#[test]
fn house_rules_are_explicit_disabled_by_default_and_replayable() {
    let mut t = Table::new();
    let entities = t.rules().entities.values().cloned().collect();
    t.state = t.origin.clone();
    t.events.clear();
    t.admin(RulesAction::Initialize {
        entities,
        house_rules: HouseRules {
            ability_test_natural_extremes: true,
        },
        ruling: ruling(),
    });
    t.admin(RulesAction::RequestTest {
        actor: t.actor,
        kind: check(),
        dc: 100,
        visibility: RollVisibility::Public,
        circumstances: Circumstances::default(),
        ruling: Ruling {
            basis: RulingBasis::HouseRule {
                id: "ability-test-natural-extremes".into(),
            },
            reason: "Campaign explicitly enabled natural extremes".into(),
        },
        request_id: RollRequestId::new(),
    });
    assert!(matches!(
        t.faces(&[20]),
        RulesOutcome::RollResolved {
            success: Some(true),
            critical: false,
            ..
        }
    ));
    t.replay();
    let baseline = Table::new();
    baseline.fails(
        CommandIssuer::Admin,
        None,
        RulesAction::Heal {
            target: baseline.actor,
            amount: 1,
            ruling: Ruling {
                basis: RulingBasis::HouseRule {
                    id: "unknown-house-rule".into(),
                },
                reason: "unregistered".into(),
            },
        },
    );
}

#[test]
fn raw_dice_and_support_metadata_reject_hidden_behavior_and_name_partial_spells() {
    assert!(serde_json::from_str::<DieResult>(r#"{"sides":20,"value":10,"total":999}"#).is_err());
    let t = Table::new();
    let RulesAnswer::SupportedContent { spells, .. } = query(
        &t.state,
        CommandIssuer::Player(t.player),
        &RulesQuery::SupportedContent,
        &t.pack,
    )
    .unwrap() else {
        panic!("wrong query result")
    };
    assert!(spells.contains(&SupportedSpell {
        id: "dancing-lights".into(),
        support: SpellSupport::ConcentrationDurationOnly
    }));
}
