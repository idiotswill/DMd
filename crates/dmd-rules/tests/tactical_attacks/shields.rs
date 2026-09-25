use super::creature_weapon::{action, choice, equipped_source, goblin, item, roll};
use super::*;

fn next_turn(f: &mut Fixture) {
    f.run(Some(0), TacticalAction::EndTurn);
    f.run(Some(1), TacticalAction::EndTurn);
}

#[test]
fn source_goblin_pays_to_doff_then_shoots_and_stows_then_pays_to_don() {
    let mut f = goblin();
    f.state.encounter.as_mut().unwrap().participants[1]
        .position
        .x = 60;
    let shield = item(&f, "shield");
    let arrows = item(&f, "arrows");
    let items = f.state.items.clone();
    f.begin();
    assert_eq!(armor_class(&f.rules().entities[&f.actors[0]]), 15);
    let doff = f.run(Some(0), TacticalAction::DoffShield);
    assert_eq!(f.loadout().command, doff.meta);
    assert_eq!(f.loadout().shield, None);
    assert_eq!(f.loadout().hands, WeaponLoadout::default());
    assert_eq!(f.state.items, items);
    assert_eq!(armor_class(&f.rules().entities[&f.actors[0]]), 13);
    let selected = choice(&f, "shortbow");
    let before = f.state.clone();
    assert!(
        resolve_tactical(
            &f.state,
            &f.meta(Some(0)),
            &action("shortbow", selected.clone()),
            &f.pack
        )
        .is_err()
    );
    assert_eq!(f.state, before);
    next_turn(&mut f);
    f.run(Some(0), action("shortbow", selected));
    roll(&mut f, 0, &[15]);
    roll(&mut f, 0, &[2]);
    assert_eq!(f.state.items[&arrows].quantity, 2);
    next_turn(&mut f);
    let before = f.state.clone();
    assert!(
        resolve_tactical(
            &f.state,
            &f.meta(Some(0)),
            &TacticalAction::DonShield {
                shield,
                hand: Hand::Left
            },
            &f.pack
        )
        .is_err()
    );
    assert_eq!(f.state, before, "both bow hands stay occupied");
    let mut stow = choice(&f, "shortbow");
    stow.equipment_change = Some(AttackEquipmentChange {
        timing: EquipmentChangeTiming::AfterAttack,
        operation: AttackEquipmentOperation::Unequip { item: stow.weapon },
    });
    f.run(Some(0), action("shortbow", stow));
    roll(&mut f, 0, &[1]);
    assert_eq!(f.loadout().hands, WeaponLoadout::default());
    assert_eq!(f.state.items[&arrows].quantity, 1);
    next_turn(&mut f);
    let don = f.run(
        Some(0),
        TacticalAction::DonShield {
            shield,
            hand: Hand::Right,
        },
    );
    assert_eq!(f.loadout().command, don.meta);
    assert_eq!(f.loadout().shield, Some(shield));
    assert_eq!(
        f.loadout().hands.hands,
        [HandAssignment::Free, HandAssignment::Item(shield)]
    );
    assert_eq!(armor_class(&f.rules().entities[&f.actors[0]]), 15);
    assert!(f.rules().timing.as_ref().unwrap().action_spent);
}

#[test]
fn shield_authority_pending_and_spent_action_rejections_preserve_physical_state() {
    let mut f = goblin();
    f.begin();
    let original = f.state.clone();
    for mut meta in [f.meta(Some(1)), f.meta(Some(0)), f.meta(Some(0))]
        .into_iter()
        .enumerate()
    {
        match meta.0 {
            1 => meta.1.expected_event_sequence -= 1,
            2 => meta.1.campaign_id = CampaignId::new(),
            _ => (),
        }
        assert!(resolve_tactical(&f.state, &meta.1, &TacticalAction::DoffShield, &f.pack).is_err());
    }
    assert_eq!(f.state, original);
    let selected = choice(&f, "scimitar");
    f.run(Some(0), action("scimitar", selected));
    let pending = f.state.clone();
    assert!(matches!(
        resolve_tactical(
            &f.state,
            &f.meta(Some(0)),
            &TacticalAction::DoffShield,
            &f.pack
        ),
        Err(RulesError::Pending)
    ));
    assert_eq!(f.state, pending);
    roll(&mut f, 0, &[1]);
    let finished = f.state.clone();
    assert!(
        resolve_tactical(
            &f.state,
            &f.meta(Some(0)),
            &TacticalAction::DoffShield,
            &f.pack
        )
        .is_err()
    );
    assert_eq!(f.state, finished);
}

#[test]
fn don_requires_an_intact_actual_shield_and_available_hand_but_not_ownership() {
    let mut f = goblin();
    let shield = item(&f, "shield");
    f.begin();
    f.run(Some(0), TacticalAction::DoffShield);
    next_turn(&mut f);
    for mutation in 0..5 {
        let mut bad = f.state.clone();
        let item = bad.items.get_mut(&shield).unwrap();
        match mutation {
            0 => item.custody = Custody::Entity(f.actors[1]),
            1 => item.state = ItemState::Destroyed,
            2 => item.quantity = 2,
            3 => item.definition_id = "dagger".into(),
            4 => item.custody = Custody::Missing,
            _ => unreachable!(),
        }
        let before = bad.clone();
        assert!(
            resolve_tactical(
                &bad,
                &f.meta(Some(0)),
                &TacticalAction::DonShield {
                    shield,
                    hand: Hand::Left
                },
                &f.pack
            )
            .is_err()
        );
        assert_eq!(bad, before);
    }
    f.state.items.get_mut(&shield).unwrap().owner = Ownership::Entity(f.actors[1]);
    f.run(
        Some(0),
        TacticalAction::DonShield {
            shield,
            hand: Hand::Left,
        },
    );
    assert_eq!(f.state.items[&shield].owner, Ownership::Entity(f.actors[1]));
    let before = f.state.clone();
    assert!(
        resolve_tactical(
            &f.state,
            &f.meta(Some(0)),
            &TacticalAction::DonShield {
                shield,
                hand: Hand::Left
            },
            &f.pack
        )
        .is_err()
    );
    assert_eq!(f.state, before);
}

#[test]
fn untrained_source_creature_can_don_borrowed_shield_without_armor_bonus() {
    let mut f = equipped_source("cultist-fanatic", 0);
    let shield = ItemId::new();
    f.state.items.insert(
        shield,
        ItemInstance {
            id: shield,
            campaign_id: f.state.campaign_id(),
            definition_id: "shield".into(),
            display_name: "Borrowed shield".into(),
            quantity: 1,
            owner: Ownership::Entity(f.actors[1]),
            custody: Custody::Entity(f.actors[0]),
            state: ItemState::Intact,
        },
    );
    f.begin();
    let initial = armor_class(&f.rules().entities[&f.actors[0]]);
    f.run(
        Some(0),
        TacticalAction::DonShield {
            shield,
            hand: Hand::Left,
        },
    );
    assert_eq!(f.loadout().shield, Some(shield));
    assert_eq!(armor_class(&f.rules().entities[&f.actors[0]]), initial);
    next_turn(&mut f);
    f.run(Some(0), TacticalAction::DoffShield);
    assert_eq!(armor_class(&f.rules().entities[&f.actors[0]]), initial);
}
