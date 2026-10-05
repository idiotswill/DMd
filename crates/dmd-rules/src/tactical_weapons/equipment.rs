use super::*;
use crate::tactical_hands::EffectiveHands;

pub(super) fn carried_item(
    state: &CampaignState,
    actor: EntityId,
    id: ItemId,
) -> Result<&ItemInstance, WeaponError> {
    let item = state
        .items
        .get(&id)
        .ok_or_else(|| illegal("unknown physical item"))?;
    require(
        item.id == id
            && item.campaign_id == state.campaign_id()
            && item.custody == Custody::Entity(actor)
            && item.quantity > 0
            && item.state == ItemState::Intact,
        "item is not intact and in this actor's physical custody",
    )?;
    Ok(item)
}
fn validate_loadout(
    input: &WeaponAttackInput<'_>,
    loadout: &WeaponLoadout,
) -> Result<(), WeaponError> {
    for assignment in loadout.hands {
        if let HandAssignment::Item(id) = assignment {
            carried_item(input.state, input.context.actor, id)?;
        }
    }
    Ok(())
}
fn apply_change(
    input: &WeaponAttackInput<'_>,
    loadout: &mut WeaponLoadout,
    change: AttackEquipmentOperation,
    hands: &EffectiveHands,
) -> Result<(), WeaponError> {
    require(
        input.context.window.kind == WeaponActionKind::AttackAction,
        "free equip/unequip belongs to an Attack action",
    )?;
    let id = match change {
        AttackEquipmentOperation::Pickup { .. } => {
            return Err(illegal("ground pickup requires sealed preparation"));
        }
        AttackEquipmentOperation::Equip { item, .. }
        | AttackEquipmentOperation::Unequip { item } => item,
    };
    let item = carried_item(input.state, input.context.actor, id)?;
    require(
        item.quantity == 1 && input.definitions.weapon(&item.definition_id).is_some(),
        "Attack-action equipment change is for one physical weapon",
    )?;
    match change {
        AttackEquipmentOperation::Pickup { .. } => {
            return Err(illegal("ground pickup requires sealed preparation"));
        }
        AttackEquipmentOperation::Equip { hand, .. } => {
            require(
                !loadout.hands.contains(&HandAssignment::Item(id)),
                "weapon is already held",
            )?;
            require(hands.is_free(loadout, hand), "equipping hand is occupied")?;
            loadout.hands[hand.index()] = HandAssignment::Item(id);
        }
        AttackEquipmentOperation::Unequip { .. } => {
            require(
                loadout.hands.contains(&HandAssignment::Item(id)),
                "weapon is not held",
            )?;
            for hand in &mut loadout.hands {
                if *hand == HandAssignment::Item(id) {
                    *hand = HandAssignment::Free;
                }
            }
        }
    }
    Ok(())
}
pub(super) fn before_attack(
    input: &WeaponAttackInput<'_>,
    weapon: &WeaponDefinition,
    hands: &EffectiveHands,
) -> Result<WeaponLoadout, WeaponError> {
    let mut loadout = input.loadout.clone();
    validate_loadout(input, &loadout)?;
    hands
        .validate_loadout(&loadout)
        .map_err(|error| invalid(error.to_string()))?;
    if let Some(change) = input.choice.equipment_change
        && change.timing == EquipmentChangeTiming::BeforeAttack
    {
        apply_change(input, &mut loadout, change.operation, hands)?;
    }
    apply_grip(
        input.choice,
        weapon,
        input.context.mounted,
        hands,
        &mut loadout,
    )?;
    Ok(loadout)
}

/// Shared physical grip derivation for the ordinary planner and bounded inverse.
pub(super) fn apply_grip(
    choice: &WeaponUseChoice,
    weapon: &WeaponDefinition,
    mounted: bool,
    hands: &EffectiveHands,
    loadout: &mut WeaponLoadout,
) -> Result<(), WeaponError> {
    let assignment = HandAssignment::Item(choice.weapon);
    match choice.grip {
        WeaponGrip::OneHand(hand) => {
            require(
                weapon.hands == WeaponHands::One
                    || (weapon.hands == WeaponHands::TwoUnlessMounted && mounted),
                "this weapon requires two hands to attack",
            )?;
            if loadout.hands[hand.index()] != assignment {
                require(
                    choice.delivery == WeaponDelivery::Thrown
                        && hands.is_free(loadout, hand)
                        && !loadout.hands.contains(&assignment),
                    "weapon is not held in the chosen hand",
                )?;
                // Thrown's own draw permission does not spend Attack-action equip allowance.
                loadout.hands[hand.index()] = assignment;
            }
            // Releasing a two-handed grip is explicit in the chosen one-handed grip.
            let other = 1 - hand.index();
            if loadout.hands[other] == assignment {
                loadout.hands[other] = HandAssignment::Free;
            }
        }
        WeaponGrip::TwoHands => {
            require(
                hands.can_use_two_hands(loadout, choice.weapon),
                "two-handed attack requires holding the weapon and both hands available",
            )?;
            // Two hands do not grant a larger die unless the source has Versatile
            // and this is a melee attack. Throwing still uses ordinary damage.
            loadout.hands = [assignment; 2];
        }
    }
    Ok(())
}
pub(super) fn ammunition(
    input: &WeaponAttackInput<'_>,
    weapon: &WeaponDefinition,
    loadout: &WeaponLoadout,
    hands: &EffectiveHands,
) -> Result<Option<AmmunitionExpenditure>, WeaponError> {
    let Some(required) = required_ammunition_definition(weapon) else {
        require(
            input.choice.ammunition.is_none(),
            "non-ammunition weapon received an ammunition choice",
        )?;
        return Ok(None);
    };
    let id = input
        .choice
        .ammunition
        .ok_or_else(|| illegal("choose a matching ammunition stack"))?;
    let stack = carried_item(input.state, input.context.actor, id)?;
    require(
        stack.definition_id == required && id != input.choice.weapon,
        "incorrect ammunition type",
    )?;
    if let WeaponGrip::OneHand(hand) = input.choice.grip {
        require(
            hands.is_free(
                loadout,
                if hand == Hand::Left {
                    Hand::Right
                } else {
                    Hand::Left
                },
            ),
            "loading a one-handed ammunition weapon requires a free hand",
        )?;
    }
    Ok(Some(AmmunitionExpenditure {
        stack: id,
        quantity: 1,
    }))
}
pub(super) fn after_attack(
    input: &WeaponAttackInput<'_>,
    before: &WeaponLoadout,
    hands: &EffectiveHands,
) -> Result<WeaponLoadout, WeaponError> {
    let mut loadout = before.clone();
    if input.choice.delivery == WeaponDelivery::Thrown {
        for hand in &mut loadout.hands {
            if *hand == HandAssignment::Item(input.choice.weapon) {
                *hand = HandAssignment::Free;
            }
        }
    }
    if let Some(change) = input.choice.equipment_change
        && change.timing == EquipmentChangeTiming::AfterAttack
    {
        let changed_item = match change.operation {
            AttackEquipmentOperation::Pickup { .. } => {
                return Err(illegal(
                    "ground pickup after an attack requires owned later work",
                ));
            }
            AttackEquipmentOperation::Equip { item, .. }
            | AttackEquipmentOperation::Unequip { item } => item,
        };
        require(
            !(input.choice.delivery == WeaponDelivery::Thrown
                && changed_item == input.choice.weapon),
            "the thrown weapon is no longer available to equip or unequip",
        )?;
        apply_change(input, &mut loadout, change.operation, hands)?;
    }
    Ok(loadout)
}
