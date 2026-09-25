use dmd_domain::*;
use dmd_rules::{tactical_inventory::*, *};

struct Fixture {
    state: CampaignState,
    inventory: TacticalInventory,
    character: CharacterId,
    actor: EntityId,
    player: PlayerId,
    other: EntityId,
    other_player: PlayerId,
    pack: RulesPack,
}
impl Fixture {
    fn new() -> Self {
        let pack =
            RulesPack::from_json(include_str!("../../../content/srd-5.2.1/kernel.json")).unwrap();
        let contract = TableContract::default();
        let campaign = Campaign {
            id: CampaignId::new(),
            display_name: "Any campaign".into(),
            status: CampaignStatus::Active,
            world_seed: 3,
            ruleset: contract.ruleset.clone(),
            content_packs: contract.permitted_content.clone(),
        };
        let mut state = CampaignState::empty(
            campaign,
            WorldClock {
                now: WorldInstant(0),
                calendar_id: "seconds".into(),
            },
        );
        state.applied_event_sequence = 7;
        let (actor, other, character, player, other_player) = (
            EntityId::new(),
            EntityId::new(),
            CharacterId::new(),
            PlayerId::new(),
            PlayerId::new(),
        );
        for id in [player, other_player] {
            state.players.insert(
                id,
                Player {
                    id,
                    campaign_id: state.campaign_id(),
                    display_name: "Player".into(),
                },
            );
        }
        for id in [actor, other] {
            state.entities.insert(
                id,
                WorldEntity {
                    id,
                    campaign_id: state.campaign_id(),
                    display_name: "Person".into(),
                    kind: if id == actor {
                        EntityKind::Character
                    } else {
                        EntityKind::Npc
                    },
                    existence: EntityExistence::Present,
                    location_id: None,
                },
            );
        }
        let built = build_character(
            &CharacterCreationInput {
                name: "Person".into(),
                pronouns: "they/them".into(),
                description: "A guard".into(),
                alignment: "Neutral Good".into(),
                backstory: "Searching for work".into(),
                ability_scores: [15, 14, 13, 8, 10, 12],
                background_boosts: [2, 0, 1, 0, 0, 0],
                fighter_skills: [Skill::Perception, Skill::Survival],
                human_skill: Skill::Insight,
                skilled_skills: [Skill::Acrobatics, Skill::Stealth, Skill::Investigation],
                size: CharacterSize::Medium,
                languages: ["dwarvish".into(), "common-sign-language".into()],
                fighting_style: FightingStyle::Defense,
                gaming_set: GamingSet::Dice,
                purchases: [
                    ("dagger", 2),
                    ("shortbow", 1),
                    ("leather-armor", 1),
                    ("shield", 1),
                    ("arrows", 40),
                    ("rations", 5),
                    ("backpack", 2),
                    ("gaming-dice", 1),
                ]
                .into_iter()
                .map(|(id, quantity)| EquipmentChoice {
                    item_id: id.into(),
                    quantity,
                })
                .collect(),
                worn_armor: Some("leather-armor".into()),
                shield: true,
                masteries: ["club".into(), "dagger".into(), "shortbow".into()],
            },
            actor,
            &pack,
        )
        .unwrap();
        state.characters.insert(
            character,
            Character {
                id: character,
                entity_id: actor,
                campaign_id: state.campaign_id(),
                controlling_player_id: Some(player),
                display_name: built.profile.name.clone(),
                status: CharacterStatus::Active,
            },
        );
        let mut table = TableState::new(contract);
        table.character_profiles.insert(character, built.profile);
        state.table = Some(table);
        assert!(state.validate().is_empty());
        Self {
            state,
            inventory: TacticalInventory::default(),
            character,
            actor,
            player,
            other,
            other_player,
            pack,
        }
    }
    fn meta(&self) -> CommandMeta {
        CommandMeta {
            id: CommandId::new(),
            campaign_id: self.state.campaign_id(),
            session_id: None,
            issuer: CommandIssuer::Player(self.player),
            actor: Some(AgentRef::Entity(self.actor)),
            expected_event_sequence: self.state.applied_event_sequence,
        }
    }
    fn ids(&self) -> Vec<ItemId> {
        let plan = starting_equipment_plan(&self.state, self.character, &self.pack).unwrap();
        (0..plan.identity_count).map(|_| ItemId::new()).collect()
    }
    fn grant(&mut self) -> StartingEquipmentReceipt {
        let next = materialize_starting_equipment(
            &self.state,
            &self.inventory,
            &self.meta(),
            self.character,
            &self.ids(),
            &self.pack,
        )
        .unwrap();
        self.state = next.next_state;
        self.inventory = next.next_inventory;
        next.receipt
    }
    fn item(&self, definition: &str) -> ItemId {
        self.inventory
            .receipt(self.character)
            .unwrap()
            .allocations
            .iter()
            .find(|entry| entry.definition_id == definition)
            .unwrap()
            .item_ids[0]
    }
    fn valid(&self) {
        validate_tactical_inventory(&self.state, &self.inventory, &self.pack).unwrap();
    }
}

#[test]
fn derived_equipment_changes_preserve_other_actors_without_loosening_grants() {
    let mut f = Fixture::new();
    f.grant();
    let mut cause = f.meta();
    cause.actor = Some(AgentRef::Entity(f.other));
    cause.issuer = CommandIssuer::Player(f.other_player);
    f.inventory.loadouts[0].command = cause.clone();
    f.valid();
    assert!(validate_equipment_origin(&f.state, &cause, f.actor).is_err());
    for invalid in [
        CommandMeta {
            actor: Some(AgentRef::Entity(EntityId::new())),
            ..cause.clone()
        },
        CommandMeta {
            campaign_id: CampaignId::new(),
            ..cause.clone()
        },
        CommandMeta {
            issuer: CommandIssuer::Player(PlayerId::new()),
            ..cause.clone()
        },
        CommandMeta {
            expected_event_sequence: f.state.applied_event_sequence + 1,
            ..cause.clone()
        },
    ] {
        assert!(validate_equipment_change_origin(&f.state, &invalid, f.actor).is_err());
    }
    f.inventory.receipts[0].command = cause;
    assert!(validate_tactical_inventory(&f.state, &f.inventory, &f.pack).is_err());
}

#[test]
fn source_grants_allocate_unique_weapons_canonical_stacks_and_initial_loadout() {
    let mut f = Fixture::new();
    let before = f.state.clone();
    let meta = f.meta();
    let ids = f.ids();
    let plan = starting_equipment_plan(&f.state, f.character, &f.pack).unwrap();
    assert_eq!(plan.identity_count, 10);
    assert_eq!(
        plan.allocations
            .iter()
            .map(|entry| entry.definition.id.as_str())
            .collect::<Vec<_>>(),
        [
            "arrows",
            "backpack",
            "dagger",
            "gaming-dice",
            "leather-armor",
            "rations",
            "shield",
            "shortbow"
        ]
    );
    let first =
        materialize_starting_equipment(&f.state, &f.inventory, &meta, f.character, &ids, &f.pack)
            .unwrap();
    let repeated_resolution =
        materialize_starting_equipment(&f.state, &f.inventory, &meta, f.character, &ids, &f.pack)
            .unwrap();
    assert_eq!(
        first, repeated_resolution,
        "same accepted inputs deterministically re-resolve"
    );
    assert_eq!(f.state, before, "pure reducer must not mutate its input");
    assert_eq!(first.receipt.command, meta);
    assert_eq!(
        first.next_state.applied_event_sequence,
        before.applied_event_sequence
    );
    assert_eq!(first.next_state.table, before.table);
    assert_eq!(
        first.receipt.creation_profile,
        before.table.as_ref().unwrap().character_profiles[&f.character]
    );
    f.state = first.next_state;
    f.inventory = first.next_inventory;
    let dagger = f
        .inventory
        .receipt(f.character)
        .unwrap()
        .allocations
        .iter()
        .find(|a| a.definition_id == "dagger")
        .unwrap();
    assert_eq!(dagger.item_ids.len(), 2);
    assert_ne!(dagger.item_ids[0], dagger.item_ids[1]);
    for id in &dagger.item_ids {
        assert_eq!(f.state.items[id].quantity, 1);
    }
    assert_eq!(f.state.items[&f.item("arrows")].quantity, 40);
    assert_eq!(f.state.items[&f.item("rations")].quantity, 5);
    let loadout = f.inventory.loadout(f.actor).unwrap();
    assert_eq!(
        loadout.hands.hands,
        [HandAssignment::Item(f.item("shield")), HandAssignment::Free]
    );
    assert_eq!(loadout.worn_armor, Some(f.item("leather-armor")));
    assert_eq!(loadout.shield, Some(f.item("shield")));
    assert_eq!(f.state.items.len(), 10);
    f.valid();
}

#[test]
fn registry_preserves_five_ammunition_identities_without_expanding_starter_shop() {
    let weapons = tactical_definitions::TacticalDefinitions::from_json(
        tactical_definitions::TACTICAL_DEFINITIONS_JSON,
    )
    .unwrap();
    assert_eq!(weapons.weapons.len(), 38);
    for weapon in weapons.weapons {
        let definition = equipment_definition(&weapon.id).unwrap();
        assert_eq!(definition.kind, EquipmentKind::Weapon);
        assert_eq!(definition.stacking, ItemStacking::Individual);
        assert_eq!(definition.source_page, weapon.source_page);
    }
    for (id, kind) in [
        ("arrows", AmmunitionStackKind::Arrows),
        ("bolts", AmmunitionStackKind::Bolts),
        ("needles", AmmunitionStackKind::Needles),
        ("firearm-bullets", AmmunitionStackKind::FirearmBullets),
        ("sling-bullets", AmmunitionStackKind::SlingBullets),
    ] {
        let definition = equipment_definition(id).unwrap();
        assert_eq!(definition.kind, EquipmentKind::Ammunition(kind));
        assert_eq!(definition.stacking, ItemStacking::Stack);
        assert_eq!(definition.source_page, 96);
    }
    assert_eq!(starter_catalog().items.len(), 16);
    assert!(
        !starter_catalog()
            .items
            .iter()
            .any(|entry| entry.id == "firearm-bullets")
    );
    assert!(equipment_definition("Arrows").is_err());
    assert!(equipment_definition("bullets").is_err());
}

#[test]
fn spent_missing_destroyed_and_transferred_items_never_reopen_the_grant() {
    let mut f = Fixture::new();
    let receipt = f.grant();
    let arrows = f.item("arrows");
    let dagger = f.item("dagger");
    let bow = f.item("shortbow");
    let item = f.state.items.get_mut(&arrows).unwrap();
    item.quantity = 0;
    item.state = ItemState::Spent;
    let item = f.state.items.get_mut(&dagger).unwrap();
    item.custody = Custody::Destroyed;
    item.state = ItemState::Destroyed;
    let item = f.state.items.get_mut(&bow).unwrap();
    item.custody = Custody::Missing;
    let backpack = f.item("backpack");
    let item = f.state.items.get_mut(&backpack).unwrap();
    item.owner = Ownership::Entity(f.other);
    item.custody = Custody::Entity(f.other);
    f.valid();
    assert_eq!(f.inventory.receipt(f.character), Some(&receipt));
    let before = (f.state.clone(), f.inventory.clone());
    assert_eq!(
        materialize_starting_equipment(
            &f.state,
            &f.inventory,
            &f.meta(),
            f.character,
            &f.ids(),
            &f.pack
        ),
        Err(InventoryError::AlreadyProvisioned(f.character))
    );
    assert_eq!((f.state.clone(), f.inventory.clone()), before);
    for item in f.state.items.values_mut() {
        item.custody = Custody::Missing;
        item.quantity = 0;
        item.state = ItemState::Spent;
    }
    let loadout = &mut f.inventory.loadouts[0];
    loadout.hands = WeaponLoadout::default();
    loadout.worn_armor = None;
    loadout.shield = None;
    f.valid();
    assert!(matches!(
        materialize_starting_equipment(
            &f.state,
            &f.inventory,
            &f.meta(),
            f.character,
            &f.ids(),
            &f.pack
        ),
        Err(InventoryError::AlreadyProvisioned(_))
    ));
    f.state.items.remove(&arrows);
    assert!(
        validate_tactical_inventory(&f.state, &f.inventory, &f.pack).is_err(),
        "deleted tombstone is corruption"
    );
}

#[test]
fn current_quantity_and_owner_can_change_but_live_loadout_requires_custody() {
    let mut f = Fixture::new();
    f.grant();
    let dagger = f.item("dagger");
    let arrows = f.item("arrows");
    f.state.items.get_mut(&arrows).unwrap().quantity = 120; // A later source-authorized refill/merge.
    f.state.items.get_mut(&dagger).unwrap().owner = Ownership::Entity(f.other);
    f.inventory.loadouts[0].hands.hands[1] = HandAssignment::Item(dagger);
    f.valid(); // Borrowing does not require legal ownership.
    f.state.items.get_mut(&dagger).unwrap().custody = Custody::Entity(f.other);
    assert!(validate_tactical_inventory(&f.state, &f.inventory, &f.pack).is_err());
    f.inventory.loadouts[0].hands.hands[1] = HandAssignment::Free;
    let mut command = f.meta();
    command.issuer = CommandIssuer::System;
    command.actor = Some(AgentRef::Entity(f.other));
    f.inventory.loadouts.push(ActorEquipmentLoadout {
        actor: f.other,
        hands: WeaponLoadout {
            hands: [HandAssignment::Item(dagger), HandAssignment::Free],
        },
        worn_armor: None,
        shield: None,
        command,
    });
    f.valid(); // Source character's receipt remains, NPC now carries the weapon.
}

#[test]
fn identities_and_authority_fail_atomically() {
    let f = Fixture::new();
    let original = (f.state.clone(), f.inventory.clone());
    let valid_ids = f.ids();
    for mutation in 0..10 {
        let mut meta = f.meta();
        let mut ids = valid_ids.clone();
        match mutation {
            0 => meta.expected_event_sequence -= 1,
            1 => meta.campaign_id = CampaignId::new(),
            2 => meta.issuer = CommandIssuer::Player(f.other_player),
            3 => meta.actor = Some(AgentRef::Entity(f.other)),
            4 => meta.issuer = CommandIssuer::Import,
            5 => {
                ids.pop();
            }
            6 => ids.push(ItemId::new()),
            7 => ids[1] = ids[0],
            8 => ids[0].0 = Default::default(),
            9 => meta.id.0 = Default::default(),
            _ => unreachable!(),
        }
        assert!(
            materialize_starting_equipment(
                &f.state,
                &f.inventory,
                &meta,
                f.character,
                &ids,
                &f.pack
            )
            .is_err(),
            "case {mutation}"
        );
        assert_eq!((f.state.clone(), f.inventory.clone()), original);
    }
    let mut foreign = f.state.clone();
    foreign.items.insert(
        valid_ids[0],
        ItemInstance {
            id: valid_ids[0],
            campaign_id: f.state.campaign_id(),
            definition_id: "other.source".into(),
            display_name: "Unrelated".into(),
            quantity: 1,
            owner: Ownership::Unowned,
            custody: Custody::Entity(f.other),
            state: ItemState::Intact,
        },
    );
    assert_eq!(
        materialize_starting_equipment(
            &foreign,
            &f.inventory,
            &f.meta(),
            f.character,
            &valid_ids,
            &f.pack
        ),
        Err(InventoryError::IdentityCollision(valid_ids[0]))
    );
    let mut meta = f.meta();
    meta.issuer = CommandIssuer::Admin;
    meta.actor = Some(AgentRef::Entity(f.other));
    assert!(
        materialize_starting_equipment(
            &f.state,
            &f.inventory,
            &meta,
            f.character,
            &valid_ids,
            &f.pack
        )
        .is_err()
    );
}

#[test]
fn matching_imported_gear_requires_reconciliation_including_borrowed_contained_and_lost() {
    for case in 0..4 {
        let mut f = Fixture::new();
        let id = ItemId::new();
        let mut item = ItemInstance {
            id,
            campaign_id: f.state.campaign_id(),
            definition_id: "dagger".into(),
            display_name: "Renamed knife".into(),
            quantity: 1,
            owner: Ownership::Entity(f.other),
            custody: Custody::Entity(f.other),
            state: ItemState::Intact,
        };
        match case {
            0 => item.custody = Custody::Entity(f.actor), // Borrowed imported item.
            1 => {
                item.owner = Ownership::Entity(f.actor);
                item.custody = Custody::Missing;
            }
            2 => {
                item.owner = Ownership::Entity(f.actor);
                item.custody = Custody::Destroyed;
                item.state = ItemState::Destroyed;
            }
            3 => {
                let container = ItemId::new();
                item.custody = Custody::Container(container);
                f.state.items.insert(
                    container,
                    ItemInstance {
                        id: container,
                        campaign_id: f.state.campaign_id(),
                        definition_id: "custom.container".into(),
                        display_name: "Bag".into(),
                        quantity: 1,
                        owner: Ownership::Unowned,
                        custody: Custody::Entity(f.actor),
                        state: ItemState::Intact,
                    },
                );
            }
            _ => unreachable!(),
        }
        f.state.items.insert(id, item);
        let before = (f.state.clone(), f.inventory.clone());
        assert_eq!(
            materialize_starting_equipment(
                &f.state,
                &f.inventory,
                &f.meta(),
                f.character,
                &f.ids(),
                &f.pack
            ),
            Err(InventoryError::ReconciliationRequired(vec![id]))
        );
        assert_eq!((f.state, f.inventory), before);
    }
    let mut f = Fixture::new();
    let id = ItemId::new();
    f.state.items.insert(
        id,
        ItemInstance {
            id,
            campaign_id: f.state.campaign_id(),
            definition_id: "dagger".into(),
            display_name: "Another person's dagger".into(),
            quantity: 1,
            owner: Ownership::Entity(f.other),
            custody: Custody::Entity(f.other),
            state: ItemState::Intact,
        },
    );
    f.grant();
    f.valid(); // Other actors' gear is not automatically this PC's old grant.
}

#[test]
fn corrupt_retained_grants_fail_closed_even_at_an_initial_snapshot() {
    let mut original = Fixture::new();
    original.grant();
    for mutation in 0..16 {
        let mut state = original.state.clone();
        let mut inventory = original.inventory.clone();
        match mutation {
            0 => inventory.schema_version += 1,
            1 => inventory.receipts[0].source.profile_id = "unsupported".into(),
            2 => inventory.receipts[0].source.ruleset_version = "5.2.2".into(),
            3 => inventory.receipts[0].creation_profile.money_cp += 1,
            4 => inventory.receipts[0].allocations[0].initial_quantity += 1,
            5 => {
                inventory.receipts[0].allocations[1].item_ids.pop();
            }
            6 => inventory.receipts[0].allocations.swap(0, 1),
            7 => {
                inventory.receipts[0].allocations[1].item_ids[1] =
                    inventory.receipts[0].allocations[1].item_ids[0]
            }
            8 => inventory.receipts[0].command.expected_event_sequence += 1,
            9 => inventory.receipts[0].command.issuer = CommandIssuer::Import,
            10 => inventory.receipts.push(inventory.receipts[0].clone()),
            11 => inventory.loadouts.push(inventory.loadouts[0].clone()),
            12 => {
                state
                    .items
                    .get_mut(&original.item("dagger"))
                    .unwrap()
                    .quantity = 2
            }
            13 => inventory.loadouts[0].worn_armor = Some(original.item("dagger")),
            14 => inventory.loadouts[0].hands = WeaponLoadout::default(),
            15 => {
                inventory.receipts[0].creation_profile.money_cp += 1;
                state
                    .table
                    .as_mut()
                    .unwrap()
                    .character_profiles
                    .get_mut(&original.character)
                    .unwrap()
                    .money_cp += 1;
            }
            _ => unreachable!(),
        }
        assert!(
            validate_tactical_inventory(&state, &inventory, &original.pack).is_err(),
            "corruption {mutation}"
        );
    }
}

#[test]
fn receipt_roundtrip_keeps_original_command_and_ignores_later_controller_transfer() {
    let mut f = Fixture::new();
    let receipt = f.grant();
    f.state
        .characters
        .get_mut(&f.character)
        .unwrap()
        .controlling_player_id = Some(f.other_player);
    f.valid();
    let json = serde_json::to_string(&f.inventory).unwrap();
    let restored: TacticalInventory = serde_json::from_str(&json).unwrap();
    assert_eq!(restored, f.inventory);
    assert_eq!(restored.receipts[0].command, receipt.command);
    validate_tactical_inventory(&f.state, &restored, &f.pack).unwrap();
    let mut malformed = serde_json::to_value(&restored).unwrap();
    malformed["receipts"][0]["arbitrary_grant"] = serde_json::json!(999);
    assert!(serde_json::from_value::<TacticalInventory>(malformed).is_err());
}

#[test]
fn profile_plan_rejects_unsupported_source_and_forged_starting_grants() {
    let f = Fixture::new();
    let mut state = f.state.clone();
    state.campaign.ruleset.version = "5.2.2".into();
    assert!(starting_equipment_plan(&state, f.character, &f.pack).is_err());
    let mut state = f.state.clone();
    state
        .table
        .as_mut()
        .unwrap()
        .character_profiles
        .get_mut(&f.character)
        .unwrap()
        .equipment[0]
        .quantity += 1;
    assert!(starting_equipment_plan(&state, f.character, &f.pack).is_err());
    let mut state = f.state.clone();
    state
        .table
        .as_mut()
        .unwrap()
        .character_profiles
        .get_mut(&f.character)
        .unwrap()
        .equipment[0]
        .item_id = "firearm-bullets".into();
    assert!(starting_equipment_plan(&state, f.character, &f.pack).is_err());
    assert!(starting_equipment_plan(&f.state, CharacterId::new(), &f.pack).is_err());
}

#[test]
fn buying_no_gear_still_creates_a_durable_one_time_receipt() {
    let mut f = Fixture::new();
    let profile = f
        .state
        .table
        .as_mut()
        .unwrap()
        .character_profiles
        .get_mut(&f.character)
        .unwrap();
    profile.equipment.clear();
    profile.worn_armor = None;
    profile.shield = false;
    profile.money_cp = 20500;
    validate_character_profile(profile, &f.pack).unwrap();
    assert!(f.ids().is_empty());
    f.grant();
    f.valid();
    assert!(f.state.items.is_empty());
    assert!(f.inventory.receipts[0].allocations.is_empty());
    assert_eq!(f.inventory.loadouts[0].hands, WeaponLoadout::default());
    assert!(matches!(
        materialize_starting_equipment(
            &f.state,
            &f.inventory,
            &f.meta(),
            f.character,
            &[],
            &f.pack
        ),
        Err(InventoryError::AlreadyProvisioned(_))
    ));
}

#[test]
fn holding_an_improvised_object_does_not_make_it_source_armor_or_a_weapon() {
    let mut f = Fixture::new();
    f.grant();
    let id = ItemId::new();
    f.state.items.insert(
        id,
        ItemInstance {
            id,
            campaign_id: f.state.campaign_id(),
            definition_id: "world.chair".into(),
            display_name: "Chair".into(),
            quantity: 1,
            owner: Ownership::Entity(f.other),
            custody: Custody::Entity(f.actor),
            state: ItemState::Intact,
        },
    );
    f.inventory.loadouts[0].hands.hands[1] = HandAssignment::Item(id);
    f.valid();
    assert!(
        equipment_definition("world.chair").is_err(),
        "holding does not authorize source weapon mechanics"
    );
    f.inventory.loadouts[0].shield = None;
    f.inventory.loadouts[0].hands.hands = [HandAssignment::Item(id); 2];
    f.valid();
    f.inventory.loadouts[0].worn_armor = Some(id);
    assert!(validate_tactical_inventory(&f.state, &f.inventory, &f.pack).is_err());
    f.inventory.loadouts[0].worn_armor = None;
    f.inventory.loadouts[0].hands.hands[0] = HandAssignment::Free;
    f.inventory.loadouts[0].shield = Some(id);
    assert!(validate_tactical_inventory(&f.state, &f.inventory, &f.pack).is_err());
    f.inventory.loadouts[0].shield = None;
    f.state.items.get_mut(&id).unwrap().quantity = 0;
    assert!(validate_tactical_inventory(&f.state, &f.inventory, &f.pack).is_err());
    f.state.items.get_mut(&id).unwrap().quantity = 1;
    f.state.items.get_mut(&id).unwrap().custody = Custody::Entity(f.other);
    assert!(validate_tactical_inventory(&f.state, &f.inventory, &f.pack).is_err());
}

#[test]
fn loose_npc_gear_requires_valid_physical_state_without_a_pc_grant_or_loadout() {
    let mut f = Fixture::new();
    let id = ItemId::new();
    f.state.items.insert(
        id,
        ItemInstance {
            id,
            campaign_id: f.state.campaign_id(),
            definition_id: "scimitar".into(),
            display_name: "Recovered gear".into(),
            quantity: 1,
            owner: Ownership::Entity(f.other),
            custody: Custody::Entity(f.other),
            state: ItemState::Intact,
        },
    );
    assert!(f.inventory.receipts.is_empty());
    assert!(f.inventory.loadouts.is_empty());
    f.valid();
    for (definition, quantity, item_state) in [
        ("scimitar", 2, ItemState::Intact),
        ("arrows", 0, ItemState::Intact),
        ("arrows", 4, ItemState::Spent),
        ("holy-symbol", 1, ItemState::Custom("ready".into())),
        ("spell-material:hold-person", 0, ItemState::Damaged),
    ] {
        let item = f.state.items.get_mut(&id).unwrap();
        item.definition_id = definition.into();
        item.quantity = quantity;
        item.state = item_state;
        assert!(
            validate_tactical_inventory(&f.state, &f.inventory, &f.pack).is_err(),
            "loose {definition} escaped physical validation"
        );
    }
    let item = f.state.items.get_mut(&id).unwrap();
    item.definition_id = "arrows".into();
    item.quantity = 0;
    item.state = ItemState::Spent;
    f.valid();
    let item = f.state.items.get_mut(&id).unwrap();
    item.definition_id = "world.sealed-letter".into();
    item.quantity = 1;
    item.state = ItemState::Custom("sealed".into());
    f.valid(); // Source equipment rules do not redefine unrelated campaign objects.
}

#[test]
fn inventory_still_rejects_foreign_item_ownership_and_custody_references() {
    let mut fixture = Fixture::new();
    fixture.grant();
    for corruption in 0..4 {
        let mut state = fixture.state.clone();
        let item = state.items.get_mut(&fixture.item("dagger")).unwrap();
        match corruption {
            0 => item.campaign_id = CampaignId::new(),
            1 => item.owner = Ownership::Entity(EntityId::new()),
            2 => item.custody = Custody::Entity(EntityId::new()),
            3 => item.custody = Custody::Container(ItemId::new()),
            _ => unreachable!(),
        }
        assert!(!state.validate_references().is_empty());
        assert!(validate_tactical_inventory(&state, &fixture.inventory, &fixture.pack).is_err());
    }
}
