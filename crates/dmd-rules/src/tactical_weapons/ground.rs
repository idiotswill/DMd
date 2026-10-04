//! Guarded physical foundation. The public planner/commands deny every Pickup.
//! This module derives private candidates; it exposes no commit or accepted state.
use super::*;
use crate::tactical_hands::EffectiveHands;

const UNAVAILABLE: &str = "ground equipment is unavailable";

#[cfg(test)]
#[path = "ground_tests.rs"]
mod tests;

struct PreparedPickup<'a> {
    original: &'a CampaignState,
    candidate: CampaignState,
    before: AttackGroundPickupBefore,
    hand: Hand,
    origin: CommandMeta,
    window: WeaponActionWindow,
}

impl<'a> PreparedPickup<'a> {
    #[allow(clippy::too_many_arguments)]
    fn new(
        state: &'a CampaignState,
        origin: &CommandMeta,
        actor: EntityId,
        window: WeaponActionWindow,
        item_id: ItemId,
        hand: Hand,
        pack: &RulesPack,
        definitions: &TacticalDefinitions,
    ) -> Result<Self, WeaponError> {
        // All failures have one non-disclosing shape, including unknown/foreign IDs.
        let flow = state
            .encounter
            .as_ref()
            .and_then(|e| e.flow.as_ref())
            .ok_or_else(|| illegal(UNAVAILABLE))?;
        let timing = state
            .rules
            .as_ref()
            .and_then(|r| r.timing.as_ref())
            .ok_or_else(|| illegal(UNAVAILABLE))?;
        require(
            flow.version == TacticalExecutionVersion::EncounterReleaseV1.flow_version()
                && flow.phase == TacticalPhase::Active
                && timing
                    .order
                    .get(timing.index)
                    .is_some_and(|entry| entry.actor == actor)
                && window.kind == WeaponActionKind::AttackAction
                && (if timing.action_spent {
                    flow.budget.attacks_remaining > 0 && flow.budget.attack_window == Some(window)
                } else {
                    window.id == origin.id
                })
                && !has_unimplemented_ground_records(state),
            UNAVAILABLE,
        )?;
        Self::derive(
            state,
            origin,
            actor,
            window,
            item_id,
            hand,
            pack,
            definitions,
        )
        .map_err(|_| illegal(UNAVAILABLE))
    }

    #[allow(clippy::too_many_arguments)]
    fn derive(
        state: &'a CampaignState,
        origin: &CommandMeta,
        actor: EntityId,
        window: WeaponActionWindow,
        item_id: ItemId,
        hand: Hand,
        pack: &RulesPack,
        definitions: &TacticalDefinitions,
    ) -> Result<Self, WeaponError> {
        require(
            origin.campaign_id == state.campaign_id()
                && origin.expected_event_sequence == state.applied_event_sequence
                && !origin.id.0.is_nil(),
            UNAVAILABLE,
        )?;
        crate::tactical::authorize(state, origin, actor).map_err(|_| illegal(UNAVAILABLE))?;
        let encounter = state
            .encounter
            .as_ref()
            .ok_or_else(|| illegal(UNAVAILABLE))?;
        let flow = encounter
            .flow
            .as_ref()
            .ok_or_else(|| illegal(UNAVAILABLE))?;
        let rules = state.rules.as_ref().ok_or_else(|| illegal(UNAVAILABLE))?;
        require(window.kind == WeaponActionKind::AttackAction, UNAVAILABLE)?;
        require(
            crate::tactical_conditions::can_act(rules, actor).map_err(|_| illegal(UNAVAILABLE))?,
            UNAVAILABLE,
        )?;
        require(
            crate::tactical_grapple_sources::ordinary_grapple_anatomy(state, actor, pack)
                .map_err(|_| illegal(UNAVAILABLE))?
                .is_some(),
            UNAVAILABLE,
        )?;
        let scene = state
            .scenes
            .get(&encounter.scene_id)
            .ok_or_else(|| illegal(UNAVAILABLE))?;
        require(
            scene.status == SceneStatus::Active && scene.campaign_id == state.campaign_id(),
            UNAVAILABLE,
        )?;
        let mut records = flow
            .ground_items
            .iter()
            .enumerate()
            .filter(|(_, entry)| entry.item == item_id);
        let (ground_index, ground) = records.next().ok_or_else(|| illegal(UNAVAILABLE))?;
        require(records.next().is_none(), UNAVAILABLE)?;
        require(
            crate::spatial::object_point_access(encounter, state, actor, ground.position)
                .map_err(|_| illegal(UNAVAILABLE))?,
            UNAVAILABLE,
        )?;
        let item = state
            .items
            .get(&item_id)
            .ok_or_else(|| illegal(UNAVAILABLE))?;
        require(
            item.id == item_id
                && item.campaign_id == state.campaign_id()
                && item.custody == Custody::Location(scene.location_id)
                && item.state == ItemState::Intact
                && item.quantity == 1
                && definitions.weapon(&item.definition_id).is_some()
                && ground.origin.campaign_id == state.campaign_id()
                && ground.origin.expected_event_sequence < origin.expected_event_sequence
                && ground.origin.id != origin.id,
            UNAVAILABLE,
        )?;
        let inventory = rules
            .tactical_inventory
            .as_ref()
            .ok_or_else(|| illegal(UNAVAILABLE))?;
        let equipment = inventory
            .loadout(actor)
            .ok_or_else(|| illegal(UNAVAILABLE))?;
        require(
            inventory.loadouts.iter().all(|loadout| {
                !loadout.hands.hands.contains(&HandAssignment::Item(item_id))
                    && loadout.worn_armor != Some(item_id)
                    && loadout.shield != Some(item_id)
            }),
            UNAVAILABLE,
        )?;
        let hands =
            EffectiveHands::current(state, rules, actor).map_err(|_| illegal(UNAVAILABLE))?;
        hands
            .validate_loadout(&equipment.hands)
            .map_err(|_| illegal(UNAVAILABLE))?;
        require(hands.is_free(&equipment.hands, hand), UNAVAILABLE)?;
        let before = AttackGroundPickupBefore {
            item: item.clone(),
            ground: ground.clone(),
            ground_index: u32::try_from(ground_index).map_err(|_| illegal(UNAVAILABLE))?,
            encounter: encounter.id,
            scene: encounter.scene_id,
            location: scene.location_id,
            equipment: equipment.clone(),
        };
        let mut candidate = state.clone();
        candidate.items.get_mut(&item_id).unwrap().custody = Custody::Entity(actor);
        candidate
            .encounter
            .as_mut()
            .unwrap()
            .flow
            .as_mut()
            .unwrap()
            .ground_items
            .retain(|record| record.item != item_id);
        candidate
            .rules
            .as_mut()
            .unwrap()
            .tactical_inventory
            .as_mut()
            .unwrap()
            .loadouts
            .iter_mut()
            .find(|loadout| loadout.actor == actor)
            .unwrap()
            .hands
            .hands[hand.index()] = HandAssignment::Item(item_id);
        Ok(Self {
            original: state,
            candidate,
            before,
            hand,
            origin: origin.clone(),
            window,
        })
    }

    fn plan(self, input: &WeaponAttackInput<'_>) -> Result<WeaponAttackPlan, WeaponError> {
        require(std::ptr::eq(self.original, input.state), UNAVAILABLE)?;
        require(input.loadout == &self.before.equipment.hands, UNAVAILABLE)?;
        require(
            input.context.origin == &self.origin
                && input.context.actor == self.before.equipment.actor
                && input.context.window == self.window
                && input.context.on_actor_turn
                && input.choice.purpose == WeaponAttackPurpose::Normal
                && input.choice.equipment_change
                    == Some(AttackEquipmentChange {
                        timing: EquipmentChangeTiming::BeforeAttack,
                        operation: AttackEquipmentOperation::Pickup {
                            item: self.before.item.id,
                            hand: self.hand,
                        },
                    }),
            UNAVAILABLE,
        )?;
        // Only this sealed local working choice consumes Pickup. The actual
        // declaration remains unchanged and its original image enters the receipt.
        let mut working = input.choice.clone();
        working.equipment_change = None;
        let loadout = self
            .candidate
            .rules
            .as_ref()
            .unwrap()
            .tactical_inventory
            .as_ref()
            .unwrap()
            .loadout(input.context.actor)
            .unwrap();
        let mut plan = prepare_weapon_attack_inner(&WeaponAttackInput {
            state: &self.candidate,
            choice: &working,
            source: input.source,
            pack: input.pack,
            definitions: input.definitions,
            context: input.context,
            loadout: &loadout.hands,
            history: input.history,
        })?;
        plan.receipt.ground_pickup_before = Some(self.before);
        Ok(plan)
    }
}

pub(super) fn prepare(input: &WeaponAttackInput<'_>) -> Result<WeaponAttackPlan, WeaponError> {
    let Some(change) = input.choice.equipment_change else {
        return prepare_weapon_attack_inner(input);
    };
    let AttackEquipmentOperation::Pickup { item, hand } = change.operation else {
        return prepare_weapon_attack_inner(input);
    };
    require(
        change.timing == EquipmentChangeTiming::BeforeAttack
            && input.choice.purpose == WeaponAttackPurpose::Normal
            && input.context.on_actor_turn,
        UNAVAILABLE,
    )?;
    PreparedPickup::new(
        input.state,
        input.context.origin,
        input.context.actor,
        input.context.window,
        item,
        hand,
        input.pack,
        input.definitions,
    )?
    .plan(input)
}

/// Bounded inverse for the pickup image. Full attack reconstruction and original
/// command replay remain required; this evidence cannot authenticate itself.
/// No public producer can reach a receipt carrying this image in this checkpoint.
pub(crate) fn restore_before_image(
    current: &CampaignState,
    before: &mut CampaignState,
    attack: &TacticalAttack,
    pack: &RulesPack,
) -> Result<(), WeaponError> {
    let weapon = attack.weapon().ok_or_else(|| illegal(UNAVAILABLE))?;
    let Some(image) = &weapon.ground_pickup_before else {
        require(
            !is_ground_pickup(weapon.choice.equipment_change),
            UNAVAILABLE,
        )?;
        return Ok(());
    };
    let Some(AttackEquipmentChange {
        timing: EquipmentChangeTiming::BeforeAttack,
        operation: AttackEquipmentOperation::Pickup { item, hand },
    }) = weapon.choice.equipment_change
    else {
        return Err(illegal(UNAVAILABLE));
    };
    require(
        item == image.item.id
            && image.ground.item == item
            && image.equipment == weapon.equipment_before
            && image.equipment.actor == attack.actor
            && weapon.choice.purpose == WeaponAttackPurpose::Normal
            && matches!(
                attack.admission,
                TacticalAttackAdmission::OwnTurn
                    | TacticalAttackAdmission::CreatureAction { approach: None }
            ),
        UNAVAILABLE,
    )?;
    let encounter = current
        .encounter
        .as_ref()
        .ok_or_else(|| illegal(UNAVAILABLE))?;
    let flow = encounter
        .flow
        .as_ref()
        .ok_or_else(|| illegal(UNAVAILABLE))?;
    require(
        encounter.id == image.encounter
            && encounter.scene_id == image.scene
            && current
                .scenes
                .get(&image.scene)
                .is_some_and(|s| s.location_id == image.location)
            && !flow.ground_items.iter().any(|entry| entry.item == item),
        UNAVAILABLE,
    )?;
    let mut expected_item = image.item.clone();
    expected_item.custody = Custody::Entity(attack.actor);
    require(
        current.items.get(&item) == Some(&expected_item),
        UNAVAILABLE,
    )?;
    let equipment = current
        .rules
        .as_ref()
        .and_then(|r| r.tactical_inventory.as_ref())
        .and_then(|i| i.loadout(attack.actor))
        .ok_or_else(|| illegal(UNAVAILABLE))?;
    let mut expected_equipment = image.equipment.clone();
    expected_equipment.command = attack.origin.clone();
    expected_equipment.hands.hands[hand.index()] = HandAssignment::Item(item);
    let definitions = bundled_tactical_definitions().map_err(|e| invalid(e.to_string()))?;
    let selected_item = current
        .items
        .get(&weapon.choice.weapon)
        .ok_or_else(|| illegal(UNAVAILABLE))?;
    let definition = definitions
        .weapon(&selected_item.definition_id)
        .ok_or_else(|| illegal(UNAVAILABLE))?;
    let hands = EffectiveHands::current(
        before,
        before.rules.as_ref().ok_or_else(|| illegal(UNAVAILABLE))?,
        attack.actor,
    )
    .map_err(|_| illegal(UNAVAILABLE))?;
    equipment::apply_grip(
        &weapon.choice,
        definition,
        false,
        &hands,
        &mut expected_equipment.hands,
    )?;
    require(equipment == &expected_equipment, UNAVAILABLE)?;
    // Stage the whole inverse locally. Any failed rederivation leaves the caller's
    // reconstruction unchanged as well as never touching authoritative current.
    let mut reconstructed = before.clone();
    reconstructed.items.insert(item, image.item.clone());
    let ground = &mut reconstructed
        .encounter
        .as_mut()
        .ok_or_else(|| illegal(UNAVAILABLE))?
        .flow
        .as_mut()
        .ok_or_else(|| illegal(UNAVAILABLE))?
        .ground_items;
    let index = usize::try_from(image.ground_index).map_err(|_| illegal(UNAVAILABLE))?;
    require(
        index <= ground.len() && !ground.iter().any(|entry| entry.item == item),
        UNAVAILABLE,
    )?;
    ground.insert(index, image.ground.clone());
    // The physical inverse rederives physical preparation only, not a new attack
    // allowance from the already-paid current budget. Original command replay is
    // required for that admission cut. No constructor capable of producing a new
    // plan or mutating current state is exposed by this inverse.
    let prepared = PreparedPickup::derive(
        &reconstructed,
        &attack.origin,
        attack.actor,
        weapon.window,
        item,
        hand,
        pack,
        definitions,
    )?;
    require(
        prepared.before == *image && prepared.hand == hand,
        UNAVAILABLE,
    )?;
    *before = reconstructed;
    Ok(())
}
