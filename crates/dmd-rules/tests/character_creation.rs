use dmd_domain::*;
use dmd_rules::*;

fn pack() -> RulesPack {
    RulesPack::from_json(include_str!("../../../content/srd-5.2.1/kernel.json")).unwrap()
}
fn input() -> CharacterCreationInput {
    CharacterCreationInput {
        name: "A player character".into(),
        pronouns: "they/them".into(),
        description: "An experienced guard".into(),
        alignment: "Neutral Good".into(),
        backstory: "I claim to know the mayor; this is not established world truth".into(),
        ability_scores: [15, 14, 13, 8, 10, 12],
        background_boosts: [2, 0, 1, 0, 0, 0],
        fighter_skills: [Skill::Perception, Skill::Survival],
        human_skill: Skill::Insight,
        skilled_skills: [Skill::Acrobatics, Skill::Stealth, Skill::Investigation],
        size: CharacterSize::Small,
        languages: ["dwarvish".into(), "common-sign-language".into()],
        fighting_style: FightingStyle::Defense,
        gaming_set: GamingSet::Dice,
        purchases: [
            ("club", 1),
            ("dagger", 1),
            ("shortbow", 1),
            ("leather-armor", 1),
            ("arrows", 20),
            ("quiver", 1),
            ("gaming-dice", 1),
        ]
        .into_iter()
        .map(|(id, quantity)| EquipmentChoice {
            item_id: id.into(),
            quantity,
        })
        .collect(),
        worn_armor: Some("leather-armor".into()),
        shield: false,
        masteries: ["club".into(), "dagger".into(), "shortbow".into()],
    }
}
fn ruling() -> Ruling {
    Ruling {
        basis: RulingBasis::GmAdjudication,
        reason: "Source-checked table context".into(),
    }
}

#[test]
fn source_creation_derives_every_grant_and_profile_without_caller_totals() {
    let built = build_character(&input(), EntityId::new(), &pack()).unwrap();
    assert_eq!(built.mechanics.ability_scores, [17, 14, 14, 8, 10, 12]);
    assert_eq!(built.mechanics.max_hp, 12);
    assert_eq!(
        built.mechanics.hit_dice,
        HitDice {
            sides: 10,
            maximum: 1,
            remaining: 1
        }
    );
    assert_eq!(
        built.mechanics.saving_proficiencies,
        std::collections::BTreeSet::from([Ability::Strength, Ability::Constitution])
    );
    assert_eq!(built.mechanics.skill_proficiencies.len(), 8);
    assert_eq!(armor_class(&built.mechanics), 14);
    assert_eq!(built.profile.money_cp, 16580);
    assert_eq!(built.profile.speed_feet, 30);
    assert_eq!(
        built.profile.languages,
        vec!["common", "dwarvish", "common-sign-language"]
    );
    assert_eq!(built.profile.equipment, built.equipment);
    assert_eq!(built.profile.class_id, "fighter");
    assert_eq!(built.profile.species_id, "human");
    assert_eq!(built.profile.background_id, "soldier");
    assert!(
        built
            .profile
            .features
            .iter()
            .any(|f| f.id == "weapon-mastery" && f.execution_gate == 4)
    );
    assert!(!built.profile.features.iter().any(|f| f.id == "tough"));
    validate_character_profile(&built.profile, &pack()).unwrap();
    let mut tampered = built.profile.clone();
    tampered.money_cp += 1;
    assert!(validate_character_profile(&tampered, &pack()).is_err());
    tampered = built.profile.clone();
    tampered.features[0].source_page = 1;
    assert!(validate_character_profile(&tampered, &pack()).is_err());
    tampered = built.profile.clone();
    tampered.equipment[0].unit_cost_cp = 0;
    assert!(validate_character_profile(&tampered, &pack()).is_err());
}

#[test]
fn real_choices_and_invalid_source_combinations_are_validated() {
    let mut alternate = input();
    alternate.ability_scores = [8, 15, 14, 10, 13, 12];
    alternate.background_boosts = [1, 1, 1, 0, 0, 0];
    alternate.fighting_style = FightingStyle::Archery;
    alternate.worn_armor = None;
    alternate.size = CharacterSize::Medium;
    let built = build_character(&alternate, EntityId::new(), &pack()).unwrap();
    assert_eq!(built.mechanics.ability_scores, [9, 16, 15, 10, 13, 12]);
    assert_eq!(armor_class(&built.mechanics), 13);
    for mutation in 0..9 {
        let mut bad = input();
        match mutation {
            0 => bad.ability_scores[0] = 18,
            1 => bad.background_boosts = [0, 0, 1, 2, 0, 0],
            2 => bad.fighter_skills[0] = Skill::Arcana,
            3 => bad.human_skill = Skill::Athletics,
            4 => bad.languages[0] = "thieves-cant".into(),
            5 => bad.masteries[0] = "greatsword".into(),
            6 => bad.purchases[0].quantity = 1000,
            7 => bad.shield = true,
            8 => bad.purchases[4].quantity = 1,
            _ => unreachable!(),
        }
        // The expensive case must exceed 205 GP, not merely add inexpensive clubs.
        if mutation == 6 {
            bad.purchases[2].quantity = 9;
        }
        assert!(
            build_character(&bad, EntityId::new(), &pack()).is_err(),
            "mutation {mutation}"
        );
    }
}

struct Table {
    state: CampaignState,
    origin: CampaignState,
    actor: EntityId,
    other: EntityId,
    player: PlayerId,
    other_player: PlayerId,
    events: Vec<RulesEvent>,
}
impl Table {
    fn new() -> Self {
        let campaign = Campaign {
            id: CampaignId::new(),
            display_name: "Unrelated custom campaign".into(),
            status: CampaignStatus::Active,
            world_seed: 31,
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
                calendar_id: "seconds".into(),
            },
        );
        let (actor, other, player, other_player) = (
            EntityId::new(),
            EntityId::new(),
            PlayerId::new(),
            PlayerId::new(),
        );
        for (id, controller) in [(actor, player), (other, other_player)] {
            state.players.insert(
                controller,
                Player {
                    id: controller,
                    campaign_id: state.campaign_id(),
                    display_name: "Player".into(),
                },
            );
            state.entities.insert(
                id,
                WorldEntity {
                    id,
                    campaign_id: state.campaign_id(),
                    display_name: "Same Name".into(),
                    kind: EntityKind::Character,
                    existence: EntityExistence::Present,
                    location_id: None,
                },
            );
            let cid = CharacterId::new();
            state.characters.insert(
                cid,
                Character {
                    id: cid,
                    entity_id: id,
                    campaign_id: state.campaign_id(),
                    controlling_player_id: Some(controller),
                    display_name: "Same Name".into(),
                    status: CharacterStatus::Active,
                },
            );
        }
        let origin = state.clone();
        let mut t = Self {
            state,
            origin,
            actor,
            other,
            player,
            other_player,
            events: vec![],
        };
        t.act(
            player,
            actor,
            RulesAction::CreateCharacter {
                entity_id: actor,
                input: input(),
            },
        );
        t.act(
            other_player,
            other,
            RulesAction::CreateCharacter {
                entity_id: other,
                input: input(),
            },
        );
        t
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
        let transition = resolve(&self.state, &self.meta(issuer, actor), &action, &pack())
            .unwrap_or_else(|e| panic!("{action:?}: {e}"));
        self.state = transition.next_state;
        self.state.applied_event_sequence += 1;
        self.events.push(transition.event);
        transition.outcome
    }
    fn act(&mut self, player: PlayerId, actor: EntityId, action: RulesAction) -> RulesOutcome {
        self.step(CommandIssuer::Player(player), Some(actor), action)
    }
    fn admin(&mut self, action: RulesAction) -> RulesOutcome {
        self.step(CommandIssuer::Admin, None, action)
    }
    fn pc(&self) -> &MechanicalEntity {
        &self.state.rules.as_ref().unwrap().entities[&self.actor]
    }
    fn raw(&self, values: &[u16]) -> RollResult {
        let request = &self
            .state
            .rules
            .as_ref()
            .unwrap()
            .pending
            .as_ref()
            .unwrap()
            .request;
        let mut values = values.iter();
        RollResult {
            request_id: request.id,
            source: RollSource::Physical,
            dice: request
                .dice
                .iter()
                .flat_map(|d| {
                    (0..d.count)
                        .map(|_| DieResult {
                            sides: d.sides,
                            value: *values.next().unwrap(),
                        })
                        .collect::<Vec<_>>()
                })
                .collect(),
        }
    }
    fn submit(&mut self, values: &[u16]) -> RulesOutcome {
        let result = self.raw(values);
        let actor = self
            .state
            .rules
            .as_ref()
            .unwrap()
            .pending
            .as_ref()
            .unwrap()
            .request
            .roller
            .unwrap();
        let player = if actor == self.actor {
            self.player
        } else {
            self.other_player
        };
        self.act(player, actor, RulesAction::SubmitRoll { result })
    }
    fn rest(&mut self, kind: RestKind) {
        self.admin(RulesAction::StartRest {
            actor: self.actor,
            kind,
            ruling: ruling(),
        });
        self.admin(RulesAction::AdvanceTime {
            seconds: if kind == RestKind::Long { 28800 } else { 3600 },
            ruling: ruling(),
        });
        self.admin(RulesAction::FinishRest {
            actor: self.actor,
            slept_seconds: if kind == RestKind::Long { 21600 } else { 0 },
            ruling: ruling(),
        });
    }
    fn verify(&self) {
        let serialized = serde_json::to_string(&self.state).unwrap();
        let restored: CampaignState = serde_json::from_str(&serialized).unwrap();
        validate_state(&restored, &pack()).unwrap();
        assert_eq!(restored, self.state);
        let mut replayed = self.origin.clone();
        for event in &self.events {
            replayed = replay(&replayed, event, &pack()).unwrap().next_state;
            replayed.applied_event_sequence += 1;
        }
        assert_eq!(replayed, self.state);
    }
    fn begin_combat(&mut self) {
        for actor in [self.actor, self.other] {
            self.admin(RulesAction::RequestTest {
                actor,
                kind: TestKind::Initiative,
                dc: 0,
                visibility: RollVisibility::Public,
                circumstances: Circumstances::default(),
                ruling: ruling(),
                request_id: RollRequestId::new(),
            });
            self.submit(&[if actor == self.actor { 15 } else { 10 }]);
        }
        self.admin(RulesAction::StartCombat {
            participants: vec![
                InitiativeEntry {
                    actor: self.actor,
                    total: 17,
                    tie_break: 0,
                },
                InitiativeEntry {
                    actor: self.other,
                    total: 12,
                    tie_break: 0,
                },
            ],
            ruling: ruling(),
        });
    }
}

#[test]
fn creation_adds_a_pc_without_replacing_existing_mechanics_or_history() {
    let mut t = Table::new();
    let before = t.state.clone();
    assert!(
        resolve(
            &t.state,
            &t.meta(CommandIssuer::Player(t.player), Some(t.actor)),
            &RulesAction::CreateCharacter {
                entity_id: t.actor,
                input: input()
            },
            &pack()
        )
        .is_err()
    );
    assert_eq!(t.state, before);
    assert_eq!(t.state.rules.as_ref().unwrap().entities.len(), 2);
    t.admin(RulesAction::ApplyDamage {
        target: t.actor,
        amount: 3,
        damage_type: DamageType::Fire,
        critical: false,
        ruling: ruling(),
    });
    assert_eq!(t.pc().hp, 9);
    t.verify();
}

#[test]
fn second_wind_preserves_raw_pending_rolls_and_partial_rest_recovery() {
    let mut t = Table::new();
    t.admin(RulesAction::ApplyDamage {
        target: t.actor,
        amount: 9,
        damage_type: DamageType::Fire,
        critical: false,
        ruling: ruling(),
    });
    for die in [2, 3] {
        let outcome = t.act(
            t.player,
            t.actor,
            RulesAction::SecondWind {
                actor: t.actor,
                request_id: RollRequestId::new(),
            },
        );
        assert!(matches!(
            outcome,
            RulesOutcome::RollRequested(RollRequest { modifier: 1, .. })
        ));
        t.verify();
        t.submit(&[die]);
    }
    assert_eq!(t.pc().hp, 10);
    assert_eq!(
        t.pc()
            .character_features
            .as_ref()
            .unwrap()
            .second_wind_remaining,
        0
    );
    let before = t.state.clone();
    assert!(
        resolve(
            &t.state,
            &t.meta(CommandIssuer::Player(t.player), Some(t.actor)),
            &RulesAction::SecondWind {
                actor: t.actor,
                request_id: RollRequestId::new()
            },
            &pack()
        )
        .is_err()
    );
    assert_eq!(t.state, before);
    t.rest(RestKind::Short);
    assert_eq!(
        t.pc()
            .character_features
            .as_ref()
            .unwrap()
            .second_wind_remaining,
        1
    );
    t.rest(RestKind::Long);
    assert_eq!(
        t.pc()
            .character_features
            .as_ref()
            .unwrap()
            .second_wind_remaining,
        2
    );
    assert!(t.pc().heroic_inspiration);
    t.verify();
}

#[test]
fn inspiration_overflow_waits_for_its_controller_and_survives_serialization() {
    let mut t = Table::new();
    t.admin(RulesAction::GrantInspiration {
        actor: t.actor,
        ruling: ruling(),
    });
    t.rest(RestKind::Long);
    assert!(
        t.pc()
            .character_features
            .as_ref()
            .unwrap()
            .inspiration_transfer_pending
    );
    t.verify();
    let before = t.state.clone();
    for (issuer, actor, action) in [
        (
            CommandIssuer::Player(t.other_player),
            t.actor,
            RulesAction::ResolveInspirationTransfer {
                actor: t.actor,
                recipient: Some(t.other),
            },
        ),
        (
            CommandIssuer::Player(t.player),
            t.actor,
            RulesAction::ResolveInspirationTransfer {
                actor: t.actor,
                recipient: Some(t.actor),
            },
        ),
        (
            CommandIssuer::Player(t.player),
            t.actor,
            RulesAction::SecondWind {
                actor: t.actor,
                request_id: RollRequestId::new(),
            },
        ),
    ] {
        assert!(resolve(&t.state, &t.meta(issuer, Some(actor)), &action, &pack()).is_err());
    }
    assert_eq!(t.state, before);
    t.act(
        t.player,
        t.actor,
        RulesAction::ResolveInspirationTransfer {
            actor: t.actor,
            recipient: Some(t.other),
        },
    );
    assert!(t.state.rules.as_ref().unwrap().entities[&t.other].heroic_inspiration);
    assert!(
        !t.pc()
            .character_features
            .as_ref()
            .unwrap()
            .inspiration_transfer_pending
    );
    t.verify();
}

#[test]
fn combat_bonus_budget_and_savage_attacker_keep_source_choices_and_replay() {
    let mut t = Table::new();
    t.begin_combat();
    t.act(
        t.player,
        t.actor,
        RulesAction::SecondWind {
            actor: t.actor,
            request_id: RollRequestId::new(),
        },
    );
    t.submit(&[1]);
    assert!(
        resolve(
            &t.state,
            &t.meta(CommandIssuer::Player(t.player), Some(t.actor)),
            &RulesAction::SecondWind {
                actor: t.actor,
                request_id: RollRequestId::new()
            },
            &pack()
        )
        .is_err()
    );
    t.admin(RulesAction::AuthorizeAttack {
        actor: t.actor,
        target: t.other,
        attack_id: "club".into(),
        circumstances: Circumstances::default(),
        within_five_feet: true,
        ruling: ruling(),
    });
    t.act(
        t.player,
        t.actor,
        RulesAction::Attack {
            actor: t.actor,
            target: t.other,
            attack_id: "club".into(),
            request_id: RollRequestId::new(),
        },
    );
    t.submit(&[20]);
    let raw = SavageAttackerRoll {
        first: t.raw(&[4, 4]),
        second: t.raw(&[1, 2]),
        chosen: DamageRollChoice::Second,
        inspiration: None,
    };
    let outcome = t.act(
        t.player,
        t.actor,
        RulesAction::SubmitSavageAttacker { roll: raw.clone() },
    );
    assert!(matches!(
        outcome,
        RulesOutcome::RollResolved {
            amount: Some(6),
            critical: true,
            ..
        }
    ));
    assert_eq!(t.state.rules.as_ref().unwrap().entities[&t.other].hp, 6);
    assert_eq!(
        t.pc()
            .character_features
            .as_ref()
            .unwrap()
            .savage_attacker_turn,
        Some(1)
    );
    assert_eq!(
        t.state
            .rules
            .as_ref()
            .unwrap()
            .rolls
            .last()
            .unwrap()
            .savage_attacker,
        Some(raw)
    );
    t.verify();
    let mut forged = t.state.clone();
    forged
        .rules
        .as_mut()
        .unwrap()
        .rolls
        .last_mut()
        .unwrap()
        .savage_attacker
        .as_mut()
        .unwrap()
        .chosen = DamageRollChoice::First;
    assert!(validate_state(&forged, &pack()).is_err());
    t.admin(RulesAction::EndCombat { ruling: ruling() });
    assert_eq!(
        t.pc()
            .character_features
            .as_ref()
            .unwrap()
            .savage_attacker_turn,
        None
    );
    t.verify();
}

#[test]
fn archery_bonus_reconstructs_outstanding_weapon_request() {
    let mut t = Table::new();
    // A distinct campaign whose source creation selected Archery; replay starts before creation.
    t.state = t.origin.clone();
    t.events.clear();
    let mut choice = input();
    choice.fighting_style = FightingStyle::Archery;
    t.act(
        t.player,
        t.actor,
        RulesAction::CreateCharacter {
            entity_id: t.actor,
            input: choice,
        },
    );
    t.act(
        t.other_player,
        t.other,
        RulesAction::CreateCharacter {
            entity_id: t.other,
            input: input(),
        },
    );
    t.admin(RulesAction::AuthorizeAttack {
        actor: t.actor,
        target: t.other,
        attack_id: "shortbow".into(),
        circumstances: Circumstances::default(),
        within_five_feet: false,
        ruling: ruling(),
    });
    let out = t.act(
        t.player,
        t.actor,
        RulesAction::Attack {
            actor: t.actor,
            target: t.other,
            attack_id: "shortbow".into(),
            request_id: RollRequestId::new(),
        },
    );
    assert!(matches!(
        out,
        RulesOutcome::RollRequested(RollRequest { modifier: 6, .. })
    ));
    t.verify();
    t.submit(&[12]);
    t.submit(&[3]);
    t.verify();
}

#[test]
fn starter_catalog_retains_exact_source_identity_and_prices() {
    let catalog = starter_catalog();
    assert_eq!(
        (
            catalog.schema_version,
            catalog.ruleset_id.as_str(),
            catalog.version.as_str()
        ),
        (1, "srd-5.2", "5.2.1")
    );
    assert_eq!(catalog.profile_id, "human-fighter-soldier-level-1");
    assert_eq!(catalog.starting_money_cp, 20500); // Fighter C155GP + Soldier B50GP.
    assert_eq!(
        catalog
            .items
            .iter()
            .map(|i| &i.id)
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        catalog.items.len()
    );
    for (id, cp, page, multiple) in [
        ("club", 10, 91, 1),
        ("dagger", 200, 91, 1),
        ("shortbow", 2500, 91, 1),
        ("leather-armor", 1000, 92, 1),
        ("shield", 1000, 92, 1),
        ("arrows", 5, 96, 20),
        ("gaming-dice", 10, 94, 1),
        ("playing-cards", 50, 94, 1),
    ] {
        let item = catalog.items.iter().find(|i| i.id == id).unwrap();
        assert_eq!(
            (item.unit_cost_cp, item.source_page, item.purchase_multiple),
            (cp, page, multiple),
            "{id}"
        );
    }
    let notice = include_str!("../../../content/srd-5.2.1/NOTICE.md");
    assert!(notice.contains("character-creation.json"));
    assert!(notice.contains("Creative Commons Attribution 4.0"));
}

#[test]
fn savage_attacker_records_inspiration_and_rejects_invalid_or_repeated_choices() {
    let mut t = Table::new();
    t.begin_combat();
    t.admin(RulesAction::GrantInspiration {
        actor: t.actor,
        ruling: ruling(),
    });
    t.admin(RulesAction::AuthorizeAttack {
        actor: t.actor,
        target: t.other,
        attack_id: "club".into(),
        circumstances: Circumstances::default(),
        within_five_feet: true,
        ruling: ruling(),
    });
    t.act(
        t.player,
        t.actor,
        RulesAction::Attack {
            actor: t.actor,
            target: t.other,
            attack_id: "club".into(),
            request_id: RollRequestId::new(),
        },
    );
    t.submit(&[20]);
    let choice = SavageAttackerRoll {
        first: t.raw(&[1, 1]),
        second: t.raw(&[3, 3]),
        chosen: DamageRollChoice::First,
        inspiration: Some(SavageInspiration {
            roll: DamageRollChoice::First,
            die_index: 0,
            replacement: DieResult { sides: 4, value: 4 },
        }),
    };
    let before = t.state.clone();
    for mutation in 0..4 {
        let mut bad = choice.clone();
        match mutation {
            0 => bad.second.request_id = RollRequestId::new(),
            1 => bad.second.dice[0].value = 5,
            2 => bad.second.source = RollSource::Digital,
            3 => bad.inspiration.as_mut().unwrap().die_index = 7,
            _ => unreachable!(),
        }
        assert!(
            resolve(
                &t.state,
                &t.meta(CommandIssuer::Player(t.player), Some(t.actor)),
                &RulesAction::SubmitSavageAttacker { roll: bad },
                &pack()
            )
            .is_err()
        );
        assert_eq!(t.state, before);
    }
    let mut already_used = t.state.clone();
    already_used
        .rules
        .as_mut()
        .unwrap()
        .entities
        .get_mut(&t.actor)
        .unwrap()
        .character_features
        .as_mut()
        .unwrap()
        .savage_attacker_turn = Some(1);
    assert!(
        resolve(
            &already_used,
            &t.meta(CommandIssuer::Player(t.player), Some(t.actor)),
            &RulesAction::SubmitSavageAttacker {
                roll: choice.clone()
            },
            &pack()
        )
        .is_err()
    );
    let outcome = t.act(
        t.player,
        t.actor,
        RulesAction::SubmitSavageAttacker {
            roll: choice.clone(),
        },
    );
    assert!(matches!(
        outcome,
        RulesOutcome::RollResolved {
            amount: Some(8),
            ..
        }
    ));
    assert!(!t.pc().heroic_inspiration);
    let saved = t.state.rules.as_ref().unwrap().rolls.last().unwrap();
    assert_eq!(
        saved.result.dice,
        vec![
            DieResult { sides: 4, value: 4 },
            DieResult { sides: 4, value: 1 }
        ]
    );
    assert_eq!(saved.savage_attacker, Some(choice));
    t.verify();
}

#[test]
fn inspiration_transfer_can_be_declined_without_inventing_a_recipient() {
    let mut t = Table::new();
    t.admin(RulesAction::GrantInspiration {
        actor: t.actor,
        ruling: ruling(),
    });
    t.rest(RestKind::Long);
    t.act(
        t.player,
        t.actor,
        RulesAction::ResolveInspirationTransfer {
            actor: t.actor,
            recipient: None,
        },
    );
    assert!(t.pc().heroic_inspiration);
    assert!(!t.state.rules.as_ref().unwrap().entities[&t.other].heroic_inspiration);
    t.verify();
}
