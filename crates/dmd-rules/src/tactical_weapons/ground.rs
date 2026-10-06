//! Physical pickup derivation shared by activated command and read-only offers.
//! Private selection owns after-operation execution. No public commit or imported
//! accepted-state constructor can manufacture this physical preparation.
use super::*;
use crate::tactical::grapple::{execution::ReadContext, reads::AttackRead};
use crate::tactical_hands::EffectiveHands;

fn current_hands(
    state: &CampaignState,
    actor: EntityId,
    read: Option<&ReadContext<'_>>,
) -> Result<EffectiveHands, WeaponError> {
    match read {
        Some(read) => {
            require(std::ptr::eq(state, read.state()), UNAVAILABLE)?;
            EffectiveHands::current_with_read(read, actor)
        }
        None => EffectiveHands::current(
            state,
            state.rules.as_ref().ok_or_else(|| illegal(UNAVAILABLE))?,
            actor,
        ),
    }
    .map_err(|e| invalid(e.to_string()))
}

fn admitted_hands(
    state: &CampaignState,
    actor: EntityId,
    read: Option<&AttackRead<'_>>,
) -> Result<EffectiveHands, WeaponError> {
    match read {
        Some(read) => {
            require(read.actor() == actor && read.uses_hands(), UNAVAILABLE)?;
            EffectiveHands::attack(read).map_err(|e| invalid(e.to_string()))
        }
        None => current_hands(state, actor, None),
    }
}

const UNAVAILABLE: &str = "ground equipment is unavailable";

/// Visible reachable physical candidates. This owns no attack/equipment allowance.
pub struct GroundPickupOption {
    pub item: ItemId,
    pub hands: Vec<Hand>,
}

fn pickup_options(
    state: &CampaignState,
    actor: EntityId,
    pack: &RulesPack,
    hands: &EffectiveHands,
) -> Result<Vec<GroundPickupOption>, WeaponError> {
    let definitions = bundled_tactical_definitions().map_err(|e| invalid(e.to_string()))?;
    let flow = state
        .encounter
        .as_ref()
        .and_then(|e| e.flow.as_ref())
        .ok_or_else(|| illegal(UNAVAILABLE))?;
    let mut result = Vec::new();
    for ground in &flow.ground_items {
        let hands = [Hand::Left, Hand::Right]
            .into_iter()
            .filter(|hand| {
                pickup_image(
                    state,
                    actor,
                    ground.item,
                    *hand,
                    pack,
                    definitions,
                    Some(hands),
                )
                .is_ok()
            })
            .collect::<Vec<_>>();
        if !hands.is_empty() {
            result.push(GroundPickupOption {
                item: ground.item,
                hands,
            });
        }
    }
    result.sort_by_key(|option| option.item.0);
    Ok(result)
}

pub fn ground_pickup_options(
    state: &CampaignState,
    actor: EntityId,
    pack: &RulesPack,
) -> Result<Vec<GroundPickupOption>, WeaponError> {
    ground_pickup_options_inner(state, actor, pack, None)
}

pub(crate) fn ground_pickup_options_with_read(
    read: &ReadContext<'_>,
    actor: EntityId,
    pack: &RulesPack,
) -> Result<Vec<GroundPickupOption>, WeaponError> {
    ground_pickup_options_inner(read.state(), actor, pack, Some(read))
}

fn ground_pickup_options_inner(
    state: &CampaignState,
    actor: EntityId,
    pack: &RulesPack,
    read: Option<&ReadContext<'_>>,
) -> Result<Vec<GroundPickupOption>, WeaponError> {
    if !crate::tactical::attack_equipment_enabled(state) {
        return Ok(vec![]);
    }
    crate::tactical::validate_attack_equipment_state(state).map_err(|e| invalid(e.to_string()))?;
    let flow = state
        .encounter
        .as_ref()
        .and_then(|e| e.flow.as_ref())
        .unwrap();
    let rules = state.rules.as_ref().ok_or_else(|| illegal(UNAVAILABLE))?;
    let timing = rules.timing.as_ref().ok_or_else(|| illegal(UNAVAILABLE))?;
    if flow.phase != TacticalPhase::Active
        || flow.resolution.is_some()
        || rules.pending.is_some()
        || timing
            .order
            .get(timing.index)
            .is_none_or(|entry| entry.actor != actor)
        || timing.action_spent && flow.budget.attacks_remaining == 0
    {
        return Ok(vec![]);
    }
    pickup_options(state, actor, pack, &current_hands(state, actor, read)?)
}

pub(crate) fn after_options_with_read(
    read: &ReadContext<'_>,
    actor: EntityId,
    window: WeaponActionWindow,
    pack: &RulesPack,
) -> Result<Vec<AttackEquipmentOperation>, WeaponError> {
    after_options_inner(read.state(), actor, window, pack, Some(read))
}

fn after_options_inner(
    state: &CampaignState,
    actor: EntityId,
    window: WeaponActionWindow,
    pack: &RulesPack,
    read: Option<&ReadContext<'_>>,
) -> Result<Vec<AttackEquipmentOperation>, WeaponError> {
    let rules = state.rules.as_ref().ok_or_else(|| illegal(UNAVAILABLE))?;
    if !crate::tactical_conditions::can_act(rules, actor).map_err(|e| invalid(e.to_string()))?
        || crate::tactical_grapple_sources::ordinary_grapple_anatomy(state, actor, pack)
            .map_err(|e| invalid(e.to_string()))?
            .is_none()
    {
        return Ok(vec![]);
    }
    let loadout = rules
        .tactical_inventory
        .as_ref()
        .and_then(|i| i.loadout(actor))
        .ok_or_else(|| illegal(UNAVAILABLE))?;
    let hands = current_hands(state, actor, read)?;
    crate::tactical_inventory::validate_loadout(state, loadout)
        .map_err(|e| invalid(e.to_string()))?;
    hands
        .validate_loadout(&loadout.hands)
        .map_err(|e| invalid(e.to_string()))?;
    let definitions = bundled_tactical_definitions().map_err(|e| invalid(e.to_string()))?;
    let mut items = state
        .items
        .values()
        .filter(|item| item.custody == Custody::Entity(actor))
        .map(|item| item.id)
        .collect::<Vec<_>>();
    items.sort_by_key(|item| item.0);
    let mut result = Vec::new();
    for item in items {
        for operation in [
            AttackEquipmentOperation::Equip {
                item,
                hand: Hand::Left,
            },
            AttackEquipmentOperation::Equip {
                item,
                hand: Hand::Right,
            },
            AttackEquipmentOperation::Unequip { item },
        ] {
            if equipment::apply_operation(
                state,
                actor,
                window,
                definitions,
                &mut loadout.hands.clone(),
                operation,
                &hands,
            )
            .is_ok()
            {
                result.push(operation);
            }
        }
    }
    for pickup in pickup_options(state, actor, pack, &hands)? {
        for hand in pickup.hands {
            result.push(AttackEquipmentOperation::Pickup {
                item: pickup.item,
                hand,
            });
        }
    }
    Ok(result)
}

/// A plan view is data; only this non-Clone, non-Deserialize preparation owns the
/// exact fresh input and its internally computed physical commit candidate.
pub(crate) struct PreparedPhysicalAttack<'a> {
    original: &'a CampaignState,
    origin: CommandMeta,
    actor: EntityId,
    window: WeaponActionWindow,
    choice: WeaponUseChoice,
    candidate: Option<CampaignState>,
    plan: WeaponAttackPlan,
}

impl<'a> PreparedPhysicalAttack<'a> {
    #[cfg(test)]
    pub(crate) fn new(
        state: &'a CampaignState,
        input: &WeaponAttackInput<'_>,
    ) -> Result<Self, WeaponError> {
        Self::new_with_read(state, input, None)
    }

    pub(crate) fn new_with_read(
        state: &'a CampaignState,
        input: &WeaponAttackInput<'_>,
        read: Option<&AttackRead<'_>>,
    ) -> Result<Self, WeaponError> {
        require(std::ptr::eq(state, input.state), UNAVAILABLE)?;
        if let Some(read) = read {
            require(
                std::ptr::eq(state, read.state())
                    && read.actor() == input.context.actor
                    && read.target() == input.choice.target
                    && read.uses_hands(),
                UNAVAILABLE,
            )?;
        }
        validate_equipment_intent(input)?;
        let hands = admitted_hands(state, input.context.actor, read)?;
        let (plan, candidate) = match input.choice.equipment_change {
            Some(AttackEquipmentChange {
                timing: EquipmentChangeTiming::BeforeAttack,
                operation: AttackEquipmentOperation::Pickup { item, hand },
            }) => {
                let pickup = PreparedPickup::new_with_hands(
                    state,
                    input.context.origin,
                    input.context.actor,
                    input.context.window,
                    item,
                    hand,
                    input.pack,
                    input.definitions,
                    Some(&hands),
                )?;
                let plan = pickup.calculate_plan(input, read)?;
                (plan, Some(pickup.candidate))
            }
            _ => (prepare_with_read(input, read)?, None),
        };
        Ok(Self {
            original: state,
            origin: input.context.origin.clone(),
            actor: input.context.actor,
            window: input.context.window,
            choice: input.choice.clone(),
            candidate,
            plan,
        })
    }

    pub(crate) fn plan(&self) -> &WeaponAttackPlan {
        &self.plan
    }

    /// Discard the consuming capability when a caller needs only a pure probe.
    pub(crate) fn into_plan(self) -> WeaponAttackPlan {
        self.plan
    }

    pub(crate) fn consume(
        self,
        state: &CampaignState,
        origin: &CommandMeta,
        actor: EntityId,
        window: WeaponActionWindow,
        choice: &WeaponUseChoice,
    ) -> Result<(Option<CampaignState>, WeaponAttackPlan), WeaponError> {
        require(
            std::ptr::eq(self.original, state)
                && self.origin == *origin
                && self.actor == actor
                && self.window == window
                && self.choice == *choice,
            UNAVAILABLE,
        )?;
        Ok((self.candidate, self.plan))
    }
}

/// Consistency-only reader for an actual attached paid declaration. It owns no
/// consuming capability and can never commit its local inverse to current state.
pub(crate) struct RetainedPhysicalRead<'a> {
    current: &'a CampaignState,
    attack: &'a TacticalAttack,
    before: CampaignState,
    admitted: Option<AttackRead<'a>>,
}

impl<'a> RetainedPhysicalRead<'a> {
    #[cfg(test)]
    pub(crate) fn new(
        state: &'a CampaignState,
        attack: &'a TacticalAttack,
        pack: &RulesPack,
    ) -> Result<Self, WeaponError> {
        Self::new_with_read(&ReadContext::ordinary(state), attack, pack)
    }

    pub(crate) fn new_with_read(
        read: &ReadContext<'a>,
        attack: &'a TacticalAttack,
        pack: &RulesPack,
    ) -> Result<Self, WeaponError> {
        crate::tactical::validate_paid_equipment_read_with_context(read, attack)
            .map_err(|e| invalid(e.to_string()))?;
        Self::from_checked_cut(read, attack, pack)
    }

    #[cfg(test)]
    pub(crate) fn entered_completion(
        state: &'a CampaignState,
        meta: &CommandMeta,
        pack: &RulesPack,
    ) -> Result<Self, WeaponError> {
        Self::entered_completion_with_read(&ReadContext::ordinary(state), meta, pack)
    }

    pub(crate) fn entered_completion_with_read(
        read: &ReadContext<'a>,
        meta: &CommandMeta,
        pack: &RulesPack,
    ) -> Result<Self, WeaponError> {
        let attack = crate::tactical::validate_equipment_completion_read_with_context(read, meta)
            .map_err(|e| invalid(e.to_string()))?;
        Self::from_checked_cut(read, attack, pack)
    }

    pub(crate) fn material_choice_with_read(
        read: &ReadContext<'a>,
        meta: &CommandMeta,
        pack: &RulesPack,
    ) -> Result<Self, WeaponError> {
        let attack = crate::tactical::validate_equipment_choice_read_with_context(read, meta)
            .map_err(|e| invalid(e.to_string()))?;
        Self::from_checked_cut(read, attack, pack)
    }

    fn from_checked_cut(
        read: &ReadContext<'a>,
        attack: &'a TacticalAttack,
        pack: &RulesPack,
    ) -> Result<Self, WeaponError> {
        let state = read.state();
        let admitted = read
            .attack_retained(attack)
            .map_err(|e| invalid(e.to_string()))?;
        let weapon = attack.weapon().ok_or_else(|| illegal(UNAVAILABLE))?;
        let current = state
            .rules
            .as_ref()
            .and_then(|r| r.tactical_inventory.as_ref())
            .and_then(|i| i.loadout(attack.actor))
            .ok_or_else(|| illegal(UNAVAILABLE))?;
        require(
            current.command == attack.origin
                && current.worn_armor == weapon.equipment_before.worn_armor
                && current.shield == weapon.equipment_before.shield,
            UNAVAILABLE,
        )?;
        let mut before = state.clone();
        before.applied_event_sequence = attack.origin.expected_event_sequence;
        match (&weapon.ammunition, weapon.choice.ammunition) {
            (Some(ammo), Some(chosen)) => {
                let item = state
                    .items
                    .get(&ammo.stack)
                    .ok_or_else(|| illegal(UNAVAILABLE))?;
                require(
                    chosen == ammo.stack
                        && ammo.quantity_before.checked_sub(1) == Some(item.quantity)
                        && item.state
                            == if item.quantity == 0 {
                                ItemState::Spent
                            } else {
                                ItemState::Intact
                            },
                    UNAVAILABLE,
                )?;
                let original = before
                    .items
                    .get_mut(&ammo.stack)
                    .ok_or_else(|| illegal(UNAVAILABLE))?;
                original.quantity = ammo.quantity_before;
                original.state = ItemState::Intact;
            }
            (None, None) => (),
            _ => return Err(illegal(UNAVAILABLE)),
        }
        let equipment = before
            .rules
            .as_mut()
            .and_then(|r| r.tactical_inventory.as_mut())
            .and_then(|i| i.loadouts.iter_mut().find(|i| i.actor == attack.actor))
            .ok_or_else(|| illegal(UNAVAILABLE))?;
        *equipment = weapon.equipment_before.clone();
        restore_before_image_with_read(state, &mut before, attack, pack, admitted.as_ref())?;
        Ok(Self {
            current: state,
            attack,
            before,
            admitted,
        })
    }

    pub(crate) fn state(&self) -> &CampaignState {
        &self.before
    }

    pub(crate) fn calculate(
        &self,
        input: &WeaponAttackInput<'_>,
    ) -> Result<WeaponAttackPlan, WeaponError> {
        let weapon = self.attack.weapon().ok_or_else(|| illegal(UNAVAILABLE))?;
        require(
            std::ptr::eq(input.state, &self.before)
                && input.context.origin == &self.attack.origin
                && input.context.actor == self.attack.actor
                && input.context.window == weapon.window
                && input.choice == &weapon.choice
                && input.loadout == &weapon.equipment_before.hands,
            UNAVAILABLE,
        )?;
        validate_equipment_intent(input)?;
        let hands = admitted_hands(&self.before, self.attack.actor, self.admitted.as_ref())?;
        let plan = if let Some(AttackEquipmentChange {
            timing: EquipmentChangeTiming::BeforeAttack,
            operation: AttackEquipmentOperation::Pickup { item, hand },
        }) = weapon.choice.equipment_change
        {
            // This is physical rederivation at a proven paid cut. It deliberately
            // does not call the fresh budget constructor or return its capability.
            PreparedPickup::derive_with_hands(
                &self.before,
                &self.attack.origin,
                self.attack.actor,
                weapon.window,
                item,
                hand,
                input.pack,
                input.definitions,
                Some(&hands),
            )?
            .calculate_plan(input, self.admitted.as_ref())?
        } else {
            prepare_with_read(input, self.admitted.as_ref())?
        };
        let flow = self
            .current
            .encounter
            .as_ref()
            .and_then(|e| e.flow.as_ref())
            .ok_or_else(|| illegal(UNAVAILABLE))?;
        let receipt = flow
            .budget
            .weapon_history
            .iter()
            .find(|r| r.origin.id == self.attack.origin.id)
            .ok_or_else(|| illegal(UNAVAILABLE))?;
        let equipment = self
            .current
            .rules
            .as_ref()
            .and_then(|r| r.tactical_inventory.as_ref())
            .and_then(|i| i.loadout(self.attack.actor))
            .ok_or_else(|| illegal(UNAVAILABLE))?;
        require(
            plan.receipt == *receipt && plan.loadout_for_attack == equipment.hands,
            UNAVAILABLE,
        )?;
        crate::tactical::validate_paid_equipment_plan(
            self.current,
            self.attack,
            &plan,
            self.admitted.as_ref(),
        )
        .map_err(|e| invalid(e.to_string()))?;
        Ok(plan)
    }
}

/// Borrowed, internally derived operation. Only the selected-work token can
/// create one for execution; receipt inversion below rederives physical data.
pub(crate) struct PreparedAfter<'a> {
    original: &'a CampaignState,
    candidate: CampaignState,
    image: AttackEquipmentApplied,
}

impl PreparedAfter<'_> {
    pub(crate) fn consume(
        self,
        state: &CampaignState,
    ) -> Result<(CampaignState, AttackEquipmentApplied), WeaponError> {
        require(std::ptr::eq(self.original, state), UNAVAILABLE)?;
        Ok((self.candidate, self.image))
    }
}

#[cfg(test)]
pub(crate) fn prepare_after<'a>(
    selected: &crate::tactical::attack_equipment::SelectedEquipment<'a>,
    operation: AttackEquipmentOperation,
    pack: &RulesPack,
) -> Result<PreparedAfter<'a>, WeaponError> {
    derive_after(
        selected.state(),
        selected.origin(),
        selected.actor(),
        selected.window(),
        operation,
        pack,
        None,
    )
    .map_err(|_| illegal(UNAVAILABLE))
}

pub(crate) fn prepare_after_with_read<'a>(
    selected: &crate::tactical::attack_equipment::SelectedEquipment<'a>,
    operation: AttackEquipmentOperation,
    pack: &RulesPack,
    read: &ReadContext<'_>,
) -> Result<PreparedAfter<'a>, WeaponError> {
    let hands = current_hands(selected.state(), selected.actor(), Some(read))?;
    derive_after(
        selected.state(),
        selected.origin(),
        selected.actor(),
        selected.window(),
        operation,
        pack,
        Some(&hands),
    )
}

/// Verify the sealed selected operation's pure inverse before committing it to
/// the existing execution candidate. The physical copy never becomes a read owner.
pub(crate) fn verify_after_inverse(
    selected: &crate::tactical::attack_equipment::SelectedEquipment<'_>,
    candidate: &CampaignState,
    receipt: &AttackAfterEquipmentReceipt,
    pack: &RulesPack,
    read: &ReadContext<'_>,
) -> Result<(), WeaponError> {
    require(
        receipt.chosen_by == *selected.origin()
            && receipt.cause.actor == selected.actor()
            && receipt.cause.window == selected.window(),
        UNAVAILABLE,
    )?;
    let hands = current_hands(selected.state(), selected.actor(), Some(read))?;
    let restored = restore_after_image_inner(candidate, receipt, pack, Some(&hands))?;
    require(restored == *selected.state(), UNAVAILABLE)
}

fn derive_after<'a>(
    state: &'a CampaignState,
    origin: &CommandMeta,
    actor: EntityId,
    window: WeaponActionWindow,
    operation: AttackEquipmentOperation,
    pack: &RulesPack,
    admitted: Option<&EffectiveHands>,
) -> Result<PreparedAfter<'a>, WeaponError> {
    let definitions = bundled_tactical_definitions().map_err(|e| invalid(e.to_string()))?;
    require(
        origin.campaign_id == state.campaign_id()
            && origin.expected_event_sequence == state.applied_event_sequence
            && !origin.id.0.is_nil()
            && window.kind == WeaponActionKind::AttackAction,
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
    require(
        flow.version == TacticalExecutionVersion::EncounterReleaseV1.flow_version()
            && flow.phase == TacticalPhase::Active
            && state
                .scenes
                .get(&encounter.scene_id)
                .is_some_and(|s| s.status == SceneStatus::Active),
        UNAVAILABLE,
    )?;
    let rules = state.rules.as_ref().ok_or_else(|| illegal(UNAVAILABLE))?;
    require(
        crate::tactical_conditions::can_act(rules, actor).map_err(|_| illegal(UNAVAILABLE))?
            && crate::tactical_grapple_sources::ordinary_grapple_anatomy(state, actor, pack)
                .map_err(|_| illegal(UNAVAILABLE))?
                .is_some(),
        UNAVAILABLE,
    )?;
    let before = rules
        .tactical_inventory
        .as_ref()
        .and_then(|i| i.loadout(actor))
        .ok_or_else(|| illegal(UNAVAILABLE))?
        .clone();
    let (mut candidate, ground_before) =
        if let AttackEquipmentOperation::Pickup { item, hand } = operation {
            // Same physical derivation as before Pickup, with a different sealed
            // admission owner. No fresh-attack budget or caller bypass flag enters.
            let prepared = PreparedPickup::derive_with_hands(
                state,
                origin,
                actor,
                window,
                item,
                hand,
                pack,
                definitions,
                admitted,
            )?;
            (prepared.candidate, Some(prepared.before))
        } else {
            let ordinary;
            let hands = if let Some(hands) = admitted {
                hands
            } else {
                ordinary = current_hands(state, actor, None)?;
                &ordinary
            };
            crate::tactical_inventory::validate_loadout(state, &before)
                .map_err(|_| illegal(UNAVAILABLE))?;
            hands
                .validate_loadout(&before.hands)
                .map_err(|_| illegal(UNAVAILABLE))?;
            let mut loadout = before.hands.clone();
            equipment::apply_operation(
                state,
                actor,
                window,
                definitions,
                &mut loadout,
                operation,
                hands,
            )?;
            let mut candidate = state.clone();
            candidate
                .rules
                .as_mut()
                .unwrap()
                .tactical_inventory
                .as_mut()
                .unwrap()
                .loadouts
                .iter_mut()
                .find(|l| l.actor == actor)
                .unwrap()
                .hands = loadout;
            (candidate, None)
        };
    candidate
        .rules
        .as_mut()
        .unwrap()
        .tactical_inventory
        .as_mut()
        .unwrap()
        .loadouts
        .iter_mut()
        .find(|l| l.actor == actor)
        .unwrap()
        .command = origin.clone();
    Ok(PreparedAfter {
        original: state,
        candidate,
        image: AttackEquipmentApplied {
            operation,
            equipment_before: before,
            ground_before,
        },
    })
}

/// Immediate decision-cut inverse only. This is physical consistency evidence,
/// not command authority or a means to reconstruct today's state from old history.
#[cfg(test)]
pub(crate) fn restore_after_image(
    current: &CampaignState,
    receipt: &AttackAfterEquipmentReceipt,
    pack: &RulesPack,
) -> Result<CampaignState, WeaponError> {
    restore_after_image_inner(current, receipt, pack, None)
}

fn restore_after_image_inner(
    current: &CampaignState,
    receipt: &AttackAfterEquipmentReceipt,
    pack: &RulesPack,
    hands: Option<&EffectiveHands>,
) -> Result<CampaignState, WeaponError> {
    crate::tactical::attack_equipment::validate_inverse_context(current, receipt)
        .map_err(|_| illegal(UNAVAILABLE))?;
    let Some(image) = receipt.applied.as_deref() else {
        return Ok(current.clone());
    };
    let actor = receipt.cause.actor;
    require(
        image.equipment_before.actor == actor
            && receipt.chosen_by.campaign_id == current.campaign_id()
            && receipt.chosen_by.expected_event_sequence == current.applied_event_sequence,
        UNAVAILABLE,
    )?;
    let mut before = current.clone();
    let loadout = before
        .rules
        .as_mut()
        .and_then(|r| r.tactical_inventory.as_mut())
        .and_then(|i| i.loadouts.iter_mut().find(|l| l.actor == actor))
        .ok_or_else(|| illegal(UNAVAILABLE))?;
    *loadout = image.equipment_before.clone();
    match (image.operation, &image.ground_before) {
        (AttackEquipmentOperation::Pickup { item, .. }, Some(ground)) => {
            require(
                ground.item.id == item
                    && ground.ground.item == item
                    && ground.equipment == image.equipment_before,
                UNAVAILABLE,
            )?;
            if item == receipt.cause.choice.weapon
                && receipt.cause.choice.delivery == WeaponDelivery::Thrown
            {
                require(
                    ground.ground.origin == receipt.cause.completed_by,
                    UNAVAILABLE,
                )?;
            }
            let encounter = before
                .encounter
                .as_mut()
                .ok_or_else(|| illegal(UNAVAILABLE))?;
            require(
                encounter.id == ground.encounter
                    && encounter.scene_id == ground.scene
                    && before
                        .scenes
                        .get(&ground.scene)
                        .is_some_and(|s| s.location_id == ground.location),
                UNAVAILABLE,
            )?;
            let flow = encounter
                .flow
                .as_mut()
                .ok_or_else(|| illegal(UNAVAILABLE))?;
            let index = usize::try_from(ground.ground_index).map_err(|_| illegal(UNAVAILABLE))?;
            require(
                index <= flow.ground_items.len()
                    && !flow.ground_items.iter().any(|g| g.item == item),
                UNAVAILABLE,
            )?;
            flow.ground_items.insert(index, ground.ground.clone());
            before.items.insert(item, ground.item.clone());
        }
        (AttackEquipmentOperation::Pickup { .. }, None) | (_, Some(_)) => {
            return Err(illegal(UNAVAILABLE));
        }
        (_, None) => {}
    }
    let prepared = derive_after(
        &before,
        &receipt.chosen_by,
        actor,
        receipt.cause.window,
        image.operation,
        pack,
        hands,
    )?;
    require(
        prepared.image == *image && prepared.candidate == *current,
        UNAVAILABLE,
    )?;
    Ok(before)
}

#[cfg(test)]
#[path = "ground_tests.rs"]
mod tests;

// Physical eligibility shared by offers and sealed command preparation. It has
// no command, payment or consuming capability and does not mutate campaign state.
fn pickup_image(
    state: &CampaignState,
    actor: EntityId,
    item_id: ItemId,
    hand: Hand,
    pack: &RulesPack,
    definitions: &TacticalDefinitions,
    admitted: Option<&EffectiveHands>,
) -> Result<AttackGroundPickupBefore, WeaponError> {
    let encounter = state
        .encounter
        .as_ref()
        .ok_or_else(|| illegal(UNAVAILABLE))?;
    let flow = encounter
        .flow
        .as_ref()
        .ok_or_else(|| illegal(UNAVAILABLE))?;
    let rules = state.rules.as_ref().ok_or_else(|| illegal(UNAVAILABLE))?;
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
            && ground.origin.expected_event_sequence < state.applied_event_sequence,
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
    let ordinary;
    let hands = if let Some(hands) = admitted {
        hands
    } else {
        ordinary = current_hands(state, actor, None)?;
        &ordinary
    };
    hands
        .validate_loadout(&equipment.hands)
        .map_err(|_| illegal(UNAVAILABLE))?;
    require(hands.is_free(&equipment.hands, hand), UNAVAILABLE)?;
    Ok(AttackGroundPickupBefore {
        item: item.clone(),
        ground: ground.clone(),
        ground_index: u32::try_from(ground_index).map_err(|_| illegal(UNAVAILABLE))?,
        encounter: encounter.id,
        scene: encounter.scene_id,
        location: scene.location_id,
        equipment: equipment.clone(),
    })
}

struct PreparedPickup<'a> {
    original: &'a CampaignState,
    candidate: CampaignState,
    before: AttackGroundPickupBefore,
    hand: Hand,
    origin: CommandMeta,
    window: WeaponActionWindow,
}

impl<'a> PreparedPickup<'a> {
    // Prior private mechanism controls keep their ordinary input surface. These
    // adapters are absent from production and never construct an owned read.
    #[cfg(test)]
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
        Self::new_with_hands(
            state,
            origin,
            actor,
            window,
            item_id,
            hand,
            pack,
            definitions,
            None,
        )
    }

    #[cfg(test)]
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
        Self::derive_with_hands(
            state,
            origin,
            actor,
            window,
            item_id,
            hand,
            pack,
            definitions,
            None,
        )
    }

    #[cfg(test)]
    fn plan(self, input: &WeaponAttackInput<'_>) -> Result<WeaponAttackPlan, WeaponError> {
        self.calculate_plan(input, None)
    }
    #[allow(clippy::too_many_arguments)]
    fn new_with_hands(
        state: &'a CampaignState,
        origin: &CommandMeta,
        actor: EntityId,
        window: WeaponActionWindow,
        item_id: ItemId,
        hand: Hand,
        pack: &RulesPack,
        definitions: &TacticalDefinitions,
        hands: Option<&EffectiveHands>,
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
                && (flow.attack_equipment_access.is_some() || !has_attack_equipment_records(state)),
            UNAVAILABLE,
        )?;
        Self::derive_with_hands(
            state,
            origin,
            actor,
            window,
            item_id,
            hand,
            pack,
            definitions,
            hands,
        )
        .map_err(|_| illegal(UNAVAILABLE))
    }

    #[allow(clippy::too_many_arguments)]
    fn derive_with_hands(
        state: &'a CampaignState,
        origin: &CommandMeta,
        actor: EntityId,
        window: WeaponActionWindow,
        item_id: ItemId,
        hand: Hand,
        pack: &RulesPack,
        definitions: &TacticalDefinitions,
        hands: Option<&EffectiveHands>,
    ) -> Result<Self, WeaponError> {
        require(
            origin.campaign_id == state.campaign_id()
                && origin.expected_event_sequence == state.applied_event_sequence
                && !origin.id.0.is_nil(),
            UNAVAILABLE,
        )?;
        crate::tactical::authorize(state, origin, actor).map_err(|_| illegal(UNAVAILABLE))?;
        require(window.kind == WeaponActionKind::AttackAction, UNAVAILABLE)?;
        let before = pickup_image(state, actor, item_id, hand, pack, definitions, hands)?;
        require(before.ground.origin.id != origin.id, UNAVAILABLE)?;
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

    fn calculate_plan(
        &self,
        input: &WeaponAttackInput<'_>,
        read: Option<&AttackRead<'_>>,
    ) -> Result<WeaponAttackPlan, WeaponError> {
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
        let mut plan = prepare_weapon_attack_inner(
            &WeaponAttackInput {
                state: &self.candidate,
                choice: &working,
                source: input.source,
                pack: input.pack,
                definitions: input.definitions,
                context: input.context,
                loadout: &loadout.hands,
                history: input.history,
            },
            read,
        )?;
        plan.receipt.ground_pickup_before = Some(self.before.clone());
        Ok(plan)
    }
}

pub(super) fn prepare(input: &WeaponAttackInput<'_>) -> Result<WeaponAttackPlan, WeaponError> {
    prepare_with_read(input, None)
}

fn prepare_with_read(
    input: &WeaponAttackInput<'_>,
    read: Option<&AttackRead<'_>>,
) -> Result<WeaponAttackPlan, WeaponError> {
    let Some(change) = input.choice.equipment_change else {
        return prepare_weapon_attack_inner(input, read);
    };
    let AttackEquipmentOperation::Pickup { item, hand } = change.operation else {
        return prepare_weapon_attack_inner(input, read);
    };
    require(
        change.timing == EquipmentChangeTiming::BeforeAttack
            && input.choice.purpose == WeaponAttackPurpose::Normal
            && input.context.on_actor_turn,
        UNAVAILABLE,
    )?;
    PreparedPickup::new_with_hands(
        input.state,
        input.context.origin,
        input.context.actor,
        input.context.window,
        item,
        hand,
        input.pack,
        input.definitions,
        Some(&admitted_hands(input.state, input.context.actor, read)?),
    )?
    .calculate_plan(input, read)
}

/// Bounded inverse for the pickup image. Full attack reconstruction and original
/// command replay remain required; this evidence cannot authenticate itself.
/// Public replay authenticates this image against its accepted originating action.
#[cfg(test)]
pub(crate) fn restore_before_image(
    current: &CampaignState,
    before: &mut CampaignState,
    attack: &TacticalAttack,
    pack: &RulesPack,
) -> Result<(), WeaponError> {
    restore_before_image_with_read(current, before, attack, pack, None)
}

fn restore_before_image_with_read(
    current: &CampaignState,
    before: &mut CampaignState,
    attack: &TacticalAttack,
    pack: &RulesPack,
    read: Option<&AttackRead<'_>>,
) -> Result<(), WeaponError> {
    if let Some(read) = read {
        require(
            std::ptr::eq(current, read.state())
                && read.actor() == attack.actor
                && read.target() == attack.target,
            UNAVAILABLE,
        )?;
    }
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
    let hands = admitted_hands(before, attack.actor, read)?;
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
    let prepared = PreparedPickup::derive_with_hands(
        &reconstructed,
        &attack.origin,
        attack.actor,
        weapon.window,
        item,
        hand,
        pack,
        definitions,
        Some(&hands),
    )?;
    require(
        prepared.before == *image && prepared.hand == hand,
        UNAVAILABLE,
    )?;
    *before = reconstructed;
    Ok(())
}
