//! SRD92/191: one Utilize Action to don or doff a real carried shield.
use super::*;
use crate::tactical_budget::{TacticalCost, spend_cost};
use crate::tactical_inventory::{EquipmentKind, equipment_definition, validate_loadout};

pub(super) fn change(
    state: &mut CampaignState,
    meta: &CommandMeta,
    don: Option<(ItemId, Hand)>,
    pack: &RulesPack,
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
        if loadout.hands.hands[hand.index()] != HandAssignment::Free
            && loadout.hands.hands[hand.index()] != HandAssignment::Item(id)
        {
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
