use super::*;
use crate::{CharacterCreationInput, RulesAction, build_character, resolve};

fn creation(weapon: &str) -> CharacterCreationInput {
    let mut masteries = vec![weapon.to_owned()];
    for id in ["club", "dagger", "shortbow"] {
        if !masteries.iter().any(|value| value == id) && masteries.len() < 3 {
            masteries.push(id.into());
        }
    }
    CharacterCreationInput {
        name: "Weapon tester".into(),
        pronouns: "they/them".into(),
        description: "Guard".into(),
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
    }
}

struct Fixture {
    state: CampaignState,
    profile: CharacterProfile,
    pack: RulesPack,
    definitions: TacticalDefinitions,
    meta: CommandMeta,
    actor: EntityId,
    target: EntityId,
    second: EntityId,
    choice: WeaponUseChoice,
    loadout: WeaponLoadout,
    history: Vec<WeaponAttackReceipt>,
    window: WeaponActionWindow,
    on_turn: bool,
    distance: u32,
    underwater: bool,
    swim: bool,
    mounted: bool,
    trigger_distance: Option<u32>,
}
impl Fixture {
    fn new(weapon_id: &str) -> Self {
        let pack = RulesPack::from_json(include_str!("../../../../content/srd-5.2.1/kernel.json"))
            .unwrap();
        let definitions = TacticalDefinitions::from_json(TACTICAL_DEFINITIONS_JSON).unwrap();
        let mut state = CampaignState::empty(
            Campaign {
                id: CampaignId::new(),
                display_name: "Weapon source interactions".into(),
                status: CampaignStatus::Active,
                world_seed: 9,
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
        let (actor, target, second) = (EntityId::new(), EntityId::new(), EntityId::new());
        for id in [actor, target, second] {
            state.entities.insert(
                id,
                WorldEntity {
                    id,
                    campaign_id: state.campaign_id(),
                    display_name: "An actor".into(),
                    kind: EntityKind::Character,
                    existence: EntityExistence::Present,
                    location_id: None,
                },
            );
        }
        let character_id = CharacterId::new();
        state.characters.insert(
            character_id,
            Character {
                id: character_id,
                entity_id: actor,
                campaign_id: state.campaign_id(),
                controlling_player_id: None,
                display_name: "Weapon tester".into(),
                status: CharacterStatus::Active,
            },
        );
        let mut meta = CommandMeta {
            id: CommandId::new(),
            campaign_id: state.campaign_id(),
            session_id: None,
            issuer: CommandIssuer::Admin,
            actor: Some(AgentRef::Entity(actor)),
            expected_event_sequence: 0,
        };
        let input = creation(weapon_id);
        let profile = build_character(&input, actor, &pack).unwrap().profile;
        state = resolve(
            &state,
            &meta,
            &RulesAction::CreateCharacter {
                entity_id: actor,
                input,
            },
            &pack,
        )
        .unwrap()
        .next_state;
        state.applied_event_sequence = 1;
        meta.id = CommandId::new();
        meta.expected_event_sequence = 1;
        let weapon = definitions.weapon(weapon_id).unwrap();
        let item = ItemId::new();
        let delivery = if weapon.kind == WeaponKind::Melee {
            WeaponDelivery::Melee
        } else if weapon.ammunition.is_some() {
            WeaponDelivery::Shot
        } else {
            WeaponDelivery::Thrown
        };
        let grip = if weapon.hands == WeaponHands::One {
            WeaponGrip::OneHand(Hand::Left)
        } else {
            WeaponGrip::TwoHands
        };
        let ammunition = required_ammunition_definition(weapon).map(|definition| {
            let id = ItemId::new();
            state.items.insert(
                id,
                ItemInstance {
                    id,
                    campaign_id: state.campaign_id(),
                    definition_id: definition.into(),
                    display_name: "Projectiles".into(),
                    quantity: 10,
                    owner: Ownership::Entity(actor),
                    custody: Custody::Entity(actor),
                    state: ItemState::Intact,
                },
            );
            id
        });
        state.items.insert(
            item,
            ItemInstance {
                id: item,
                campaign_id: state.campaign_id(),
                definition_id: weapon_id.into(),
                display_name: "Borrowed weapon".into(),
                quantity: 1,
                owner: Ownership::Entity(target),
                custody: Custody::Entity(actor),
                state: ItemState::Intact,
            },
        );
        let loadout = WeaponLoadout {
            hands: [
                HandAssignment::Item(item),
                if grip == WeaponGrip::TwoHands {
                    HandAssignment::Item(item)
                } else {
                    HandAssignment::Free
                },
            ],
        };
        Self {
            state,
            profile,
            pack,
            definitions,
            meta,
            actor,
            target,
            second,
            choice: WeaponUseChoice {
                weapon: item,
                target,
                delivery,
                ability: if delivery == WeaponDelivery::Melee {
                    Ability::Strength
                } else {
                    Ability::Dexterity
                },
                grip,
                purpose: WeaponAttackPurpose::Normal,
                ammunition,
                equipment_change: None,
            },
            loadout,
            history: vec![],
            window: WeaponActionWindow {
                id: CommandId::new(),
                kind: WeaponActionKind::AttackAction,
            },
            on_turn: true,
            distance: 10,
            underwater: false,
            swim: false,
            mounted: false,
            trigger_distance: None,
        }
    }
    fn input(&self) -> WeaponAttackInput<'_> {
        WeaponAttackInput {
            state: &self.state,
            source: WeaponActorSource::Character(&self.profile),
            pack: &self.pack,
            definitions: &self.definitions,
            choice: &self.choice,
            loadout: &self.loadout,
            history: &self.history,
            context: WeaponAttackContext {
                origin: &self.meta,
                actor: self.actor,
                turn_number: 1,
                on_actor_turn: self.on_turn,
                window: self.window,
                distance: self.distance,
                base_reach: 10,
                mounted: self.mounted,
                underwater: self.underwater,
                has_swim_speed: self.swim,
                target_is_creature: true,
                target_size: CreatureSize::Medium,
                distance_from_trigger_target: self.trigger_distance,
            },
        }
    }
    fn plan(&self) -> WeaponAttackPlan {
        prepare_weapon_attack(&self.input()).unwrap()
    }
    fn accept(&mut self, outcome: WeaponAttackOutcome) -> CommandId {
        let mut receipt = self.plan().receipt;
        receipt.outcome = outcome;
        let id = receipt.origin.id;
        self.history.push(receipt);
        self.state.applied_event_sequence += 1;
        self.meta.id = CommandId::new();
        self.meta.expected_event_sequence = self.state.applied_event_sequence;
        id
    }
    fn duplicate_weapon(&mut self) -> ItemId {
        let mut copy = self.state.items[&self.choice.weapon].clone();
        copy.id = ItemId::new();
        let id = copy.id;
        self.state.items.insert(id, copy);
        id
    }
    fn rebuild(&mut self, input: CharacterCreationInput) {
        let built = build_character(&input, self.actor, &self.pack).unwrap();
        self.profile = built.profile;
        self.state
            .rules
            .as_mut()
            .unwrap()
            .entities
            .insert(self.actor, built.mechanics);
    }
}

#[test]
fn every_source_weapon_uses_validated_character_grants_and_physical_inventory() {
    let definitions = TacticalDefinitions::from_json(TACTICAL_DEFINITIONS_JSON).unwrap();
    assert_eq!(definitions.weapons.len(), 38);
    for weapon in &definitions.weapons {
        let fixture = Fixture::new(&weapon.id);
        let before = fixture.state.clone();
        let plan = fixture.plan();
        assert_eq!(plan.mastery, Some(weapon.mastery), "{}", weapon.id);
        assert!(plan.proficient);
        assert_eq!(plan.receipt.weapon, fixture.choice.weapon);
        assert_eq!(plan.ammunition.is_some(), weapon.ammunition.is_some());
        assert_eq!(fixture.state, before);
        assert_eq!(
            serde_json::to_value(&plan).unwrap(),
            serde_json::to_value(fixture.plan()).unwrap()
        );
    }
}

#[test]
fn finesse_thrown_and_archery_use_source_ability_and_classification() {
    let mut fixture = Fixture::new("dagger");
    assert_eq!(
        (
            fixture.plan().attack_modifier,
            fixture.plan().damage.modifier
        ),
        (5, 3)
    );
    fixture.choice.ability = Ability::Dexterity;
    assert_eq!(
        (
            fixture.plan().attack_modifier,
            fixture.plan().damage.modifier
        ),
        (4, 2)
    );
    fixture.choice.delivery = WeaponDelivery::Thrown;
    fixture.loadout = WeaponLoadout::default();
    fixture.window.kind = WeaponActionKind::Reaction;
    let plan = fixture.plan();
    assert_eq!(plan.thrown_weapon, Some(fixture.choice.weapon));
    assert_eq!(plan.loadout_after_attack, WeaponLoadout::default());
    let mut source = creation("dagger");
    source.fighting_style = FightingStyle::Archery;
    fixture.rebuild(source);
    assert_eq!(fixture.plan().attack_modifier, 4); // Melee weapon thrown: not Archery.
    let mut dart = Fixture::new("dart");
    let mut source = creation("dart");
    source.fighting_style = FightingStyle::Archery;
    dart.rebuild(source);
    assert_eq!(dart.plan().attack_modifier, 6); // Ranged classification, even when thrown.
    dart.choice.ability = Ability::Strength;
    assert_eq!(dart.plan().attack_modifier, 7);
    dart.choice.ability = Ability::Wisdom;
    assert!(prepare_weapon_attack(&dart.input()).is_err());
    let mut spear = Fixture::new("spear");
    spear.choice.delivery = WeaponDelivery::Thrown;
    assert_eq!(spear.plan().attack_modifier, 5);
    spear.choice.ability = Ability::Dexterity;
    assert!(prepare_weapon_attack(&spear.input()).is_err());
}

#[test]
fn versatile_critical_and_fixed_blowgun_damage_keep_source_distinctions() {
    let mut staff = Fixture::new("quarterstaff");
    assert_eq!(
        staff.plan().damage.dice,
        vec![DieSpec { count: 1, sides: 6 }]
    );
    staff.choice.grip = WeaponGrip::TwoHands;
    assert_eq!(
        staff.plan().damage.dice,
        vec![DieSpec { count: 1, sides: 8 }]
    );
    let mut spear = Fixture::new("spear");
    spear.choice.grip = WeaponGrip::TwoHands;
    spear.choice.delivery = WeaponDelivery::Thrown;
    assert_eq!(
        spear.plan().damage.dice,
        vec![DieSpec { count: 1, sides: 6 }]
    );
    let sword = Fixture::new("greatsword")
        .plan()
        .damage
        .for_critical()
        .unwrap();
    assert_eq!(sword.dice, vec![DieSpec { count: 4, sides: 6 }]);
    assert_eq!(sword.modifier, 3);
    let blowgun = Fixture::new("blowgun").plan().damage;
    assert_eq!(blowgun.fixed_amount(), Some(1));
    assert_eq!(blowgun.for_critical().unwrap(), blowgun);
    let bad = WeaponDamageProfile {
        dice: vec![DieSpec {
            count: u16::MAX,
            sides: 6,
        }],
        modifier: 0,
        damage_type: DamageType::Slashing,
    };
    assert!(bad.for_critical().is_err());
}

#[test]
fn reach_range_and_underwater_boundaries_do_not_refund_automatic_misses() {
    let mut bow = Fixture::new("shortbow");
    bow.distance = 160;
    assert!(bow.plan().disadvantage.is_empty());
    bow.distance = 161;
    assert_eq!(bow.plan().disadvantage, vec![WeaponDisadvantage::LongRange]);
    bow.underwater = true;
    let plan = bow.plan();
    assert!(plan.automatic_miss);
    assert_eq!(plan.ammunition.unwrap().quantity, 1);
    bow.distance = 160;
    assert!(!bow.plan().automatic_miss);
    assert_eq!(
        bow.plan().disadvantage,
        vec![WeaponDisadvantage::UnderwaterRanged]
    );
    bow.distance = 641;
    assert!(prepare_weapon_attack(&bow.input()).is_err());
    let mut crossbow = Fixture::new("light-crossbow");
    crossbow.underwater = true;
    assert!(
        crossbow
            .plan()
            .disadvantage
            .contains(&WeaponDisadvantage::UnderwaterRanged)
    );
    let mut dagger = Fixture::new("dagger");
    dagger.underwater = true;
    assert!(dagger.plan().disadvantage.is_empty()); // Piercing, no swim speed needed.
    dagger.choice.delivery = WeaponDelivery::Thrown;
    dagger.distance = 41;
    assert!(dagger.plan().automatic_miss);
    let mut axe = Fixture::new("handaxe");
    axe.underwater = true;
    assert_eq!(
        axe.plan().disadvantage,
        vec![WeaponDisadvantage::UnderwaterMelee]
    );
    axe.swim = true;
    assert!(axe.plan().disadvantage.is_empty());
    let mut whip = Fixture::new("whip");
    whip.distance = 20;
    assert_eq!(whip.plan().reach, 20);
    whip.distance = 21;
    assert!(prepare_weapon_attack(&whip.input()).is_err());
}

#[test]
fn mounted_lance_waives_two_hands_but_never_heavy_strength_requirement() {
    let mut lance = Fixture::new("lance");
    let mut source = creation("lance");
    source.ability_scores = [8, 15, 14, 10, 13, 12];
    source.background_boosts = [1, 1, 1, 0, 0, 0];
    lance.rebuild(source);
    lance.choice.grip = WeaponGrip::OneHand(Hand::Left);
    assert!(prepare_weapon_attack(&lance.input()).is_err());
    lance.mounted = true;
    assert!(
        lance
            .plan()
            .disadvantage
            .contains(&WeaponDisadvantage::HeavyAbilityRequirement)
    );
    assert_eq!(lance.plan().damage.modifier, -1);
    let mut bow = Fixture::new("longbow");
    let mut source = creation("longbow");
    source.ability_scores = [15, 12, 13, 8, 10, 14];
    bow.rebuild(source.clone());
    assert!(
        bow.plan()
            .disadvantage
            .contains(&WeaponDisadvantage::HeavyAbilityRequirement)
    );
    source.background_boosts = [1, 1, 1, 0, 0, 0]; // Dexterity 13 is sufficient.
    bow.rebuild(source);
    assert!(
        !bow.plan()
            .disadvantage
            .contains(&WeaponDisadvantage::HeavyAbilityRequirement)
    );
}

#[test]
fn custody_identity_loading_hand_and_single_equipment_change_are_enforced() {
    let mut fixture = Fixture::new("hand-crossbow");
    let shield = fixture.duplicate_weapon();
    fixture.state.items.get_mut(&shield).unwrap().definition_id = "shield".into();
    fixture.loadout.hands[1] = HandAssignment::Item(shield);
    assert!(prepare_weapon_attack(&fixture.input()).is_err());
    fixture.loadout.hands[1] = HandAssignment::Free;
    fixture.plan();
    for mutation in 0..5 {
        let mut broken = Fixture::new("dagger");
        let item = broken.state.items.get_mut(&broken.choice.weapon).unwrap();
        match mutation {
            0 => item.id = ItemId::new(),
            1 => item.quantity = 2,
            2 => item.custody = Custody::Entity(broken.target),
            3 => item.state = ItemState::Spent,
            _ => item.campaign_id = CampaignId::new(),
        }
        assert!(prepare_weapon_attack(&broken.input()).is_err());
    }
    let mut fixture = Fixture::new("dagger");
    let other = fixture.duplicate_weapon();
    fixture.choice.equipment_change = Some(AttackEquipmentChange {
        timing: EquipmentChangeTiming::BeforeAttack,
        operation: AttackEquipmentOperation::Equip {
            item: other,
            hand: Hand::Right,
        },
    });
    let plan = fixture.plan();
    assert_eq!(
        plan.loadout_for_attack.hands[1],
        HandAssignment::Item(other)
    );
    fixture.window.kind = WeaponActionKind::Reaction;
    assert!(prepare_weapon_attack(&fixture.input()).is_err());
    fixture.window.kind = WeaponActionKind::AttackAction;
    fixture.choice.equipment_change = Some(AttackEquipmentChange {
        timing: EquipmentChangeTiming::AfterAttack,
        operation: AttackEquipmentOperation::Unequip {
            item: fixture.choice.weapon,
        },
    });
    let plan = fixture.plan();
    assert_ne!(plan.loadout_for_attack, WeaponLoadout::default());
    assert_eq!(plan.loadout_after_attack, WeaponLoadout::default());
}

#[test]
fn ammunition_types_empty_stacks_and_loading_share_actual_action_opportunity() {
    for (weapon, ammo) in [
        ("shortbow", "arrows"),
        ("hand-crossbow", "bolts"),
        ("blowgun", "needles"),
        ("sling", "sling-bullets"),
        ("pistol", "firearm-bullets"),
        ("musket", "firearm-bullets"),
    ] {
        let fixture = Fixture::new(weapon);
        assert_eq!(
            fixture.state.items[&fixture.choice.ammunition.unwrap()].definition_id,
            ammo
        );
    }
    let mut crossbow = Fixture::new("light-crossbow");
    crossbow.accept(WeaponAttackOutcome::Miss);
    assert!(prepare_weapon_attack(&crossbow.input()).is_err());
    crossbow.window = WeaponActionWindow {
        id: CommandId::new(),
        kind: WeaponActionKind::Reaction,
    };
    crossbow.plan();
    crossbow.window.kind = WeaponActionKind::BonusAction;
    crossbow.plan();
    let id = crossbow.choice.ammunition.unwrap();
    crossbow.state.items.get_mut(&id).unwrap().definition_id = "arrows".into();
    assert!(prepare_weapon_attack(&crossbow.input()).is_err());
    crossbow.state.items.get_mut(&id).unwrap().definition_id = "bolts".into();
    crossbow.state.items.get_mut(&id).unwrap().quantity = 0;
    assert!(prepare_weapon_attack(&crossbow.input()).is_err());
    assert_eq!(recoverable_ammunition(5), 2);
}

#[test]
fn light_and_nick_share_one_extra_attack_and_require_distinct_physical_weapons() {
    let mut fixture = Fixture::new("dagger");
    let trigger = fixture.accept(WeaponAttackOutcome::Miss); // Hitting is unnecessary.
    fixture.choice.purpose = WeaponAttackPurpose::Nick { trigger };
    assert!(prepare_weapon_attack(&fixture.input()).is_err());
    let other = fixture.duplicate_weapon();
    fixture.choice.weapon = other;
    fixture.loadout.hands[0] = HandAssignment::Item(other);
    assert_eq!(fixture.plan().damage.modifier, 0);
    fixture.accept(WeaponAttackOutcome::Miss);
    fixture.window = WeaponActionWindow {
        id: CommandId::new(),
        kind: WeaponActionKind::BonusAction,
    };
    fixture.choice.purpose = WeaponAttackPurpose::LightBonus { trigger };
    assert!(prepare_weapon_attack(&fixture.input()).is_err());
    for mutation in 0..3 {
        let mut fixture = Fixture::new("dagger");
        if mutation == 0 {
            fixture.window.kind = WeaponActionKind::OtherAction;
        }
        if mutation == 1 {
            fixture.on_turn = false;
        }
        let trigger = fixture.accept(WeaponAttackOutcome::Miss);
        fixture.choice.weapon = fixture.duplicate_weapon();
        fixture.loadout.hands[0] = HandAssignment::Item(fixture.choice.weapon);
        fixture.choice.purpose = WeaponAttackPurpose::Nick { trigger };
        if mutation == 2 {
            fixture.window.id = CommandId::new();
        }
        assert!(prepare_weapon_attack(&fixture.input()).is_err());
    }
    let mut weak = Fixture::new("dagger");
    let mut source = creation("dagger");
    source.ability_scores = [8, 15, 14, 10, 13, 12];
    source.background_boosts = [1, 1, 1, 0, 0, 0];
    weak.rebuild(source);
    let trigger = weak.accept(WeaponAttackOutcome::Miss);
    weak.choice.weapon = weak.duplicate_weapon();
    weak.loadout.hands[0] = HandAssignment::Item(weak.choice.weapon);
    weak.choice.purpose = WeaponAttackPurpose::LightBonus { trigger };
    weak.window = WeaponActionWindow {
        id: CommandId::new(),
        kind: WeaponActionKind::BonusAction,
    };
    assert_eq!(weak.plan().damage.modifier, -1);
}

#[test]
fn cleave_requires_same_weapon_reach_second_target_and_hit_even_off_turn() {
    let mut fixture = Fixture::new("halberd");
    fixture.on_turn = false;
    fixture.window.kind = WeaponActionKind::Reaction;
    let trigger = fixture.accept(WeaponAttackOutcome::Hit {
        critical: false,
        damage_dealt: 0,
    });
    fixture.choice.purpose = WeaponAttackPurpose::Cleave { trigger };
    fixture.trigger_distance = Some(10);
    assert!(prepare_weapon_attack(&fixture.input()).is_err()); // Same target.
    fixture.choice.target = fixture.second;
    fixture.distance = 20;
    assert_eq!(fixture.plan().damage.modifier, 0);
    fixture.trigger_distance = Some(11);
    assert!(prepare_weapon_attack(&fixture.input()).is_err());
    fixture.trigger_distance = Some(10);
    fixture.distance = 21;
    assert!(prepare_weapon_attack(&fixture.input()).is_err());
    fixture.distance = 20;
    let original = fixture.choice.weapon;
    fixture.choice.weapon = fixture.duplicate_weapon();
    fixture.loadout.hands = [HandAssignment::Item(fixture.choice.weapon); 2];
    assert!(prepare_weapon_attack(&fixture.input()).is_err());
    fixture.choice.weapon = original;
    fixture.loadout.hands = [HandAssignment::Item(original); 2];
    fixture.accept(WeaponAttackOutcome::Miss);
    assert!(prepare_weapon_attack(&fixture.input()).is_err()); // Once per global turn.
}

fn mastery(
    fixture: &Fixture,
    outcome: WeaponAttackOutcome,
    size: CreatureSize,
) -> WeaponMasteryResolution {
    weapon_mastery_resolution(
        &fixture.plan(),
        outcome,
        &WeaponMasteryContext {
            target_is_creature: true,
            target_size: size,
            history: &fixture.history,
        },
    )
    .unwrap()
}

#[test]
fn mastery_zero_damage_triggers_optional_choices_and_expiry_match_source() {
    let hit_zero = WeaponAttackOutcome::Hit {
        critical: false,
        damage_dealt: 0,
    };
    let hit = WeaponAttackOutcome::Hit {
        critical: false,
        damage_dealt: 1,
    };
    for weapon in ["club", "shortbow"] {
        // Slow and Vex need damage, not merely a hit.
        assert_eq!(
            mastery(&Fixture::new(weapon), hit_zero, CreatureSize::Medium),
            WeaponMasteryResolution::default()
        );
    }
    let sap = mastery(&Fixture::new("mace"), hit_zero, CreatureSize::Medium);
    assert!(matches!(
        sap.mandatory.as_slice(),
        [WeaponMasteryConsequence::Sap {
            expires: WeaponEffectExpiry::StartOfSourceNextTurn,
            ..
        }]
    ));
    let vex = mastery(&Fixture::new("shortbow"), hit, CreatureSize::Medium);
    assert!(matches!(
        vex.mandatory.as_slice(),
        [WeaponMasteryConsequence::Vex {
            expires: WeaponEffectExpiry::EndOfSourceNextTurn,
            ..
        }]
    ));
    let slow = mastery(&Fixture::new("club"), hit, CreatureSize::Medium)
        .offer
        .unwrap();
    assert!(matches!(
        choose_weapon_mastery(&slow, &WeaponMasteryChoice::Slow).unwrap(),
        Some(WeaponMasteryConsequence::Slow {
            reduction: 20,
            expires: WeaponEffectExpiry::StartOfSourceNextTurn,
            ..
        })
    ));
    assert_eq!(
        choose_weapon_mastery(&slow, &WeaponMasteryChoice::Decline).unwrap(),
        None
    );
    assert!(choose_weapon_mastery(&slow, &WeaponMasteryChoice::Topple).is_err());
    let topple = mastery(
        &Fixture::new("quarterstaff"),
        hit_zero,
        CreatureSize::Gargantuan,
    )
    .offer
    .unwrap();
    assert!(matches!(
        choose_weapon_mastery(&topple, &WeaponMasteryChoice::Topple).unwrap(),
        Some(WeaponMasteryConsequence::ToppleSave {
            dc: 13,
            ability: Ability::Constitution,
            on_failure: Condition::Prone,
            ..
        })
    ));
    let push = mastery(&Fixture::new("greatclub"), hit_zero, CreatureSize::Large)
        .offer
        .unwrap();
    assert!(
        choose_weapon_mastery(&push, &WeaponMasteryChoice::Push { distance: 20 })
            .unwrap()
            .is_some()
    );
    for distance in [0, 21, u32::MAX] {
        assert!(choose_weapon_mastery(&push, &WeaponMasteryChoice::Push { distance }).is_err());
    }
    assert!(
        mastery(&Fixture::new("greatclub"), hit, CreatureSize::Huge)
            .offer
            .is_none()
    );
    let graze = mastery(
        &Fixture::new("greatsword"),
        WeaponAttackOutcome::Miss,
        CreatureSize::Medium,
    )
    .offer
    .unwrap();
    assert!(matches!(
        choose_weapon_mastery(&graze, &WeaponMasteryChoice::Graze).unwrap(),
        Some(WeaponMasteryConsequence::GrazeDamage {
            amount: 3,
            damage_type: DamageType::Slashing,
            ..
        })
    ));
}

#[test]
fn mastery_grants_cleave_choice_and_automatic_miss_are_not_forgeable_permissions() {
    let mut fixture = Fixture::new("halberd");
    let offer = mastery(
        &fixture,
        WeaponAttackOutcome::Hit {
            critical: false,
            damage_dealt: 0,
        },
        CreatureSize::Medium,
    )
    .offer
    .unwrap();
    let mut attack = fixture.choice.clone();
    attack.target = fixture.second;
    attack.purpose = WeaponAttackPurpose::Cleave {
        trigger: fixture.meta.id,
    };
    assert!(
        choose_weapon_mastery(
            &offer,
            &WeaponMasteryChoice::Cleave {
                attack: Box::new(attack.clone())
            }
        )
        .unwrap()
        .is_some()
    );
    attack.weapon = fixture.duplicate_weapon();
    assert!(
        choose_weapon_mastery(
            &offer,
            &WeaponMasteryChoice::Cleave {
                attack: Box::new(attack)
            }
        )
        .is_err()
    );
    fixture.rebuild(creation("dagger")); // Valid profile, but it grants no Halberd mastery.
    assert!(fixture.plan().mastery.is_none());
    assert_eq!(
        mastery(&fixture, WeaponAttackOutcome::Miss, CreatureSize::Medium),
        WeaponMasteryResolution::default()
    );
    let mut bow = Fixture::new("shortbow");
    bow.underwater = true;
    bow.distance = 161;
    assert!(
        weapon_mastery_resolution(
            &bow.plan(),
            WeaponAttackOutcome::Hit {
                critical: false,
                damage_dealt: 1
            },
            &WeaponMasteryContext {
                target_is_creature: true,
                target_size: CreatureSize::Medium,
                history: &[]
            }
        )
        .is_err()
    );
}

#[test]
fn stale_unsupported_and_unbounded_inputs_fail_without_mutation() {
    for mutation in 0..5 {
        let mut fixture = Fixture::new("dagger");
        match mutation {
            0 => fixture.meta.expected_event_sequence = 0,
            1 => fixture.meta.actor = Some(AgentRef::Entity(fixture.target)),
            2 => fixture.distance = u32::MAX,
            3 => {
                fixture
                    .state
                    .rules
                    .as_mut()
                    .unwrap()
                    .entities
                    .get_mut(&fixture.actor)
                    .unwrap()
                    .death
                    .dead = true
            }
            _ => fixture.profile.masteries[0] = "invented-weapon".into(),
        }
        let before = fixture.state.clone();
        assert!(prepare_weapon_attack(&fixture.input()).is_err());
        assert_eq!(fixture.state, before);
    }
    let mut fixture = Fixture::new("dagger");
    fixture.accept(WeaponAttackOutcome::Miss);
    fixture.history.push(fixture.history[0].clone());
    assert!(prepare_weapon_attack(&fixture.input()).is_err());
    fixture.history.truncate(1);
    fixture.history[0].origin.campaign_id = CampaignId::new();
    assert!(prepare_weapon_attack(&fixture.input()).is_err());
}

#[test]
fn ordinary_creature_weapon_use_retains_source_training_without_stat_block_riders() {
    let mut fixture = Fixture::new("scimitar");
    let creature = fixture
        .definitions
        .creature("goblin-warrior")
        .unwrap()
        .clone();
    fixture
        .state
        .rules
        .as_mut()
        .unwrap()
        .entities
        .get_mut(&fixture.actor)
        .unwrap()
        .ability_scores = creature.statistics.ability_scores;
    fixture.choice.ability = Ability::Dexterity;
    let mut input = fixture.input();
    input.source = WeaponActorSource::CreatureOrdinaryWeapon(&creature);
    let plan = prepare_weapon_attack(&input).unwrap();
    assert_eq!(plan.attack_modifier, 4);
    assert_eq!(plan.damage.dice, vec![DieSpec { count: 1, sides: 6 }]);
    assert_eq!(plan.damage.modifier, 2);
    assert!(plan.mastery.is_none());
    let mut forged = creature.clone();
    forged.statistics.proficiency_bonus = 5;
    input.source = WeaponActorSource::CreatureOrdinaryWeapon(&forged);
    assert!(prepare_weapon_attack(&input).is_err());
    let mut looted = Fixture::new("greatsword");
    looted
        .state
        .rules
        .as_mut()
        .unwrap()
        .entities
        .get_mut(&looted.actor)
        .unwrap()
        .ability_scores = creature.statistics.ability_scores;
    let mut input = looted.input();
    input.source = WeaponActorSource::CreatureOrdinaryWeapon(&creature);
    let plan = prepare_weapon_attack(&input).unwrap();
    assert!(!plan.proficient);
    assert_eq!(plan.attack_modifier, -1);
    assert!(
        plan.disadvantage
            .contains(&WeaponDisadvantage::HeavyAbilityRequirement)
    );
}

#[test]
fn accepted_history_cannot_change_actor_action_identity_or_global_turn() {
    for mutation in 0..5 {
        let mut fixture = Fixture::new("dagger");
        fixture.accept(WeaponAttackOutcome::Miss);
        match mutation {
            0 => fixture.history[0].origin.actor = Some(AgentRef::Entity(fixture.target)),
            1 => fixture.history[0].turn_number = 2,
            2 => fixture.window.kind = WeaponActionKind::OtherAction,
            3 => fixture.history[0].origin.id = fixture.meta.id,
            _ => fixture.history = vec![fixture.history[0].clone(); 129],
        }
        assert!(
            prepare_weapon_attack(&fixture.input()).is_err(),
            "mutation {mutation}"
        );
    }
}

#[test]
fn exhaustion_applies_once_to_attack_test_but_never_damage_or_topple_dc() {
    let mut fixture = Fixture::new("quarterstaff");
    for levels in 0..6 {
        fixture
            .state
            .rules
            .as_mut()
            .unwrap()
            .entities
            .get_mut(&fixture.actor)
            .unwrap()
            .exhaustion = levels;
        let plan = fixture.plan();
        assert_eq!(plan.attack_modifier, 5 - i32::from(levels) * 2);
        assert_eq!(plan.damage.modifier, 3);
        let offer = mastery(
            &fixture,
            WeaponAttackOutcome::Hit {
                critical: false,
                damage_dealt: 0,
            },
            CreatureSize::Medium,
        )
        .offer
        .unwrap();
        assert!(matches!(offer, WeaponMasteryOffer::Topple { dc: 13, .. }));
    }
    fixture
        .state
        .rules
        .as_mut()
        .unwrap()
        .entities
        .get_mut(&fixture.actor)
        .unwrap()
        .exhaustion = 6;
    assert!(prepare_weapon_attack(&fixture.input()).is_err());
}

#[test]
fn live_armor_is_separate_from_immutable_weapon_grants() {
    let mut fixture = Fixture::new("dagger");
    let mechanics = fixture
        .state
        .rules
        .as_mut()
        .unwrap()
        .entities
        .get_mut(&fixture.actor)
        .unwrap();
    mechanics.armor = ArmorClass::Armor {
        base: 11,
        dexterity_cap: None,
        shield: false,
    };
    mechanics.character_features.as_mut().unwrap().wearing_armor = true;
    // The tactical planner does not authorize this armor or produce AC. The inventory
    // adapter must validate it; immutable creation grants still control the weapon.
    assert!(
        crate::validate_character_mechanics(&fixture.profile, mechanics, &fixture.pack).is_err()
    );
    crate::validate_character_intrinsics(&fixture.profile, mechanics, &fixture.pack).unwrap();
    assert_eq!(fixture.plan().attack_modifier, 5);
    let mechanics = fixture
        .state
        .rules
        .as_mut()
        .unwrap()
        .entities
        .get_mut(&fixture.actor)
        .unwrap();
    mechanics.ability_scores[0] += 1;
    assert!(
        crate::validate_character_intrinsics(&fixture.profile, mechanics, &fixture.pack).is_err()
    );
    assert!(prepare_weapon_attack(&fixture.input()).is_err());
}

#[test]
fn nick_and_cleave_inherit_attack_equipment_window_without_creating_an_action() {
    let mut nick = Fixture::new("dagger");
    let trigger = nick.accept(WeaponAttackOutcome::Miss);
    nick.choice.weapon = nick.duplicate_weapon();
    nick.loadout.hands[0] = HandAssignment::Item(nick.choice.weapon);
    nick.choice.purpose = WeaponAttackPurpose::Nick { trigger };
    nick.choice.equipment_change = Some(AttackEquipmentChange {
        timing: EquipmentChangeTiming::AfterAttack,
        operation: AttackEquipmentOperation::Unequip {
            item: nick.choice.weapon,
        },
    });
    assert_eq!(nick.plan().receipt.window, nick.history[0].window);
    assert_eq!(nick.plan().loadout_after_attack, WeaponLoadout::default());
    for reaction in [false, true] {
        let mut cleave = Fixture::new("halberd");
        if reaction {
            cleave.window.kind = WeaponActionKind::Reaction;
            cleave.on_turn = false;
        }
        let trigger = cleave.accept(WeaponAttackOutcome::Hit {
            critical: false,
            damage_dealt: 0,
        });
        cleave.choice.target = cleave.second;
        cleave.trigger_distance = Some(10);
        cleave.choice.purpose = WeaponAttackPurpose::Cleave { trigger };
        cleave.choice.equipment_change = Some(AttackEquipmentChange {
            timing: EquipmentChangeTiming::AfterAttack,
            operation: AttackEquipmentOperation::Unequip {
                item: cleave.choice.weapon,
            },
        });
        if reaction {
            assert!(prepare_weapon_attack(&cleave.input()).is_err());
        } else {
            assert_eq!(cleave.plan().receipt.window, cleave.history[0].window);
            assert_eq!(cleave.plan().loadout_after_attack, WeaponLoadout::default());
        }
    }
}
