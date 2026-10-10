//! SRD92/191: one Utilize Action to don or doff a real carried shield.
use super::*;
use crate::tactical_budget::{TacticalCost, spend_cost};
use crate::tactical_inventory::{EquipmentKind, equipment_definition, validate_loadout};

#[cfg(test)]
pub(super) fn change(
    state: &mut CampaignState,
    meta: &CommandMeta,
    don: Option<(ItemId, Hand)>,
    pack: &RulesPack,
) -> Result<(), RulesError> {
    change_with_context(
        state,
        meta,
        don,
        pack,
        &mut grapple::execution::ExecutionContext::ordinary(),
    )
}

pub(super) fn change_with_context(
    state: &mut CampaignState,
    meta: &CommandMeta,
    don: Option<(ItemId, Hand)>,
    pack: &RulesPack,
    execution: &mut grapple::execution::ExecutionContext<'_>,
) -> Result<(), RulesError> {
    if flow(state)?.phase != TacticalPhase::Active || flow(state)?.resolution.is_some() {
        return Err(RulesError::Pending);
    }
    let actor = turns::active(state)?;
    authorize(state, meta, actor)?;
    let mut loadout = state
        .rules
        .as_ref()
        .and_then(|rules| rules.tactical_inventory.as_ref())
        .and_then(|inventory| inventory.loadout(actor))
        .cloned()
        .ok_or_else(|| prerequisite("shield use requires current physical equipment"))?;
    if let Some((id, hand)) = don {
        if loadout.shield.is_some() {
            return Err(prerequisite(
                "doff the current shield before donning another",
            ));
        }
        let item = state
            .items
            .get(&id)
            .ok_or_else(|| prerequisite("select a real carried shield"))?;
        if item.id != id
            || item.campaign_id != state.campaign_id()
            || item.custody != Custody::Entity(actor)
            || item.state != ItemState::Intact
            || item.quantity != 1
            || !equipment_definition(&item.definition_id)
                .is_ok_and(|definition| definition.kind == EquipmentKind::Shield)
        {
            return Err(prerequisite(
                "select one intact source shield in actor custody",
            ));
        }
        let hands = crate::tactical_hands::EffectiveHands::current_with_read(
            &execution.read(state)?,
            actor,
        )?;
        hands.validate_loadout(&loadout.hands)?;
        if !hands.can_hold(&loadout.hands, hand, id) {
            return Err(prerequisite("the selected shield hand is occupied"));
        }
        if loadout.hands.hands[1 - hand.index()] == HandAssignment::Item(id) {
            return Err(prerequisite(
                "a donned shield must occupy only its selected hand",
            ));
        }
        loadout.hands.hands[hand.index()] = HandAssignment::Item(id);
        loadout.shield = Some(id);
    } else {
        let shield = loadout
            .shield
            .take()
            .ok_or_else(|| prerequisite("there is no donned shield to remove"))?;
        for hand in &mut loadout.hands.hands {
            if *hand == HandAssignment::Item(shield) {
                *hand = HandAssignment::Free;
            }
        }
        // The removed shield stays carried/stowed. This is not a ground drop,
        // ownership transfer, consumption, or a free weapon equipment change.
    }
    loadout.command = meta.clone();
    validate_loadout(state, &loadout).map_err(|error| prerequisite(&error.to_string()))?;
    crate::tactical_hands::EffectiveHands::current_with_read(&execution.read(state)?, actor)?
        .validate_loadout(&loadout.hands)?;
    let rules = state.rules.as_mut().ok_or(RulesError::Uninitialized)?;
    spend_cost(rules, actor, TacticalCost::Action)?;
    let current = rules
        .tactical_inventory
        .as_mut()
        .and_then(|inventory| {
            inventory
                .loadouts
                .iter_mut()
                .find(|entry| entry.actor == actor)
        })
        .ok_or_else(|| invalid("shield equipment disappeared"))?;
    *current = loadout;
    refresh_armor(state, actor, pack)?;
    let now = state.clock.now;
    crate::kernel::interrupt_rest(
        state.rules.as_mut().ok_or(RulesError::Uninitialized)?,
        actor,
        now,
    );
    let budget = &mut flow_mut(state)?.budget;
    budget.movement_progress = None;
    budget.movement_origin = None;
    Ok(())
}

fn refresh_armor(
    state: &mut CampaignState,
    actor: EntityId,
    pack: &RulesPack,
) -> Result<(), RulesError> {
    let rules = state.rules.as_ref().ok_or(RulesError::Uninitialized)?;
    let armor = if let Some(profile) = rules
        .tactical_creatures
        .as_ref()
        .and_then(|creatures| creatures.profile(actor))
    {
        crate::tactical_creature_equipment::creature_current_armor(state, profile)?
    } else {
        let profile = state
            .table
            .as_ref()
            .and_then(|table| {
                table
                    .character_profiles
                    .values()
                    .find(|p| p.entity_id == actor)
            })
            .ok_or_else(|| prerequisite("shield use requires authenticated training authority"))?;
        let entity = rules
            .entities
            .get(&actor)
            .ok_or_else(|| invalid("actor absent"))?;
        crate::validate_character_intrinsics(profile, entity, pack)?;
        let loadout = rules
            .tactical_inventory
            .as_ref()
            .and_then(|inventory| inventory.loadout(actor))
            .ok_or_else(|| invalid("shield equipment absent"))?;
        let base = match loadout.worn_armor {
            None => 10,
            Some(id)
                if state
                    .items
                    .get(&id)
                    .is_some_and(|item| item.definition_id == "leather-armor") =>
            {
                11
            }
            Some(_) => {
                return Err(prerequisite(
                    "current armor needs its source interpretation",
                ));
            }
        };
        ArmorClass::Armor {
            base,
            dexterity_cap: None,
            shield: loadout.shield.is_some()
                && profile.armor_training.iter().any(|kind| kind == "shield"),
        }
    };
    state
        .rules
        .as_mut()
        .ok_or(RulesError::Uninitialized)?
        .entities
        .get_mut(&actor)
        .ok_or_else(|| invalid("actor absent"))?
        .armor = armor;
    Ok(())
}

#[cfg(test)]
mod hand_tests {
    use super::*;

    #[test]
    fn real_shield_collision_precedes_cost_and_doff_preserves_the_other_reservation() {
        // Internal change control, not command acceptance: source authority is
        // genuine, but the grip, available Action and shield transfer are fixtures.
        let mut state = crate::tactical_hands::tests::source_state();
        let actor = crate::tactical_hands::tests::human(&state);
        let shield = state
            .items
            .values()
            .find(|item| item.definition_id == "shield")
            .unwrap()
            .id;
        state.items.get_mut(&shield).unwrap().custody = Custody::Entity(actor);
        let loadout = state
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
        loadout.hands.hands = [HandAssignment::Free; 2];
        let declaration =
            crate::tactical_hands::tests::install_attempt(&mut state, actor, Hand::Right);
        let meta = declaration.origin.clone();
        state
            .encounter
            .as_mut()
            .unwrap()
            .flow
            .as_mut()
            .unwrap()
            .resolution = None;
        let rules = state.rules.as_mut().unwrap();
        rules.timing.as_mut().unwrap().action_spent = false;
        rules.tactical_grapples = Some(TacticalGrapples {
            schema_version: 1,
            active: vec![crate::tactical_hands::tests::live(declaration)],
        });
        let before = serde_json::to_value(&state).unwrap();
        assert!(
            change(
                &mut state,
                &meta,
                Some((shield, Hand::Right)),
                &crate::tactical_hands::tests::pack()
            )
            .unwrap_err()
            .to_string()
            .contains("selected shield hand is occupied")
        );
        assert_eq!(serde_json::to_value(&state).unwrap(), before);
        change(
            &mut state,
            &meta,
            Some((shield, Hand::Left)),
            &crate::tactical_hands::tests::pack(),
        )
        .unwrap();
        assert!(
            state
                .rules
                .as_ref()
                .unwrap()
                .timing
                .as_ref()
                .unwrap()
                .action_spent
        );
        assert_eq!(
            state
                .rules
                .as_ref()
                .unwrap()
                .tactical_inventory
                .as_ref()
                .unwrap()
                .loadout(actor)
                .unwrap()
                .shield,
            Some(shield)
        );

        state
            .rules
            .as_mut()
            .unwrap()
            .timing
            .as_mut()
            .unwrap()
            .action_spent = false;
        change(
            &mut state,
            &meta,
            None,
            &crate::tactical_hands::tests::pack(),
        )
        .unwrap();
        let rules = state.rules.as_ref().unwrap();
        let loadout = rules
            .tactical_inventory
            .as_ref()
            .unwrap()
            .loadout(actor)
            .unwrap();
        assert_eq!(loadout.hands.hands, [HandAssignment::Free; 2]);
        assert_eq!(loadout.shield, None);
        assert_eq!(state.items[&shield].custody, Custody::Entity(actor));
        let hands = crate::tactical_hands::EffectiveHands::current(&state, rules, actor).unwrap();
        assert!(hands.is_free(&loadout.hands, Hand::Left));
        assert!(!hands.is_free(&loadout.hands, Hand::Right));
        assert!(rules.timing.as_ref().unwrap().action_spent);
    }
}
