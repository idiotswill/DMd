//! Derived current occupancy, never a second loadout or Grapple admission.
//! The kernel still refuses every new Grapple record until its resolver and
//! original replay exist. Source/shape checks here do not authenticate history.
use crate::{RulesError, RulesPack, tactical_grapple_sources::ordinary_grapple_anatomy};
use dmd_domain::*;

#[cfg(test)]
pub(crate) mod tests;

/// Read-only composition with an explicitly supplied physical loadout. Private
/// fields prevent callers from supplying a purportedly trusted reservation mask.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectiveHands {
    actor: EntityId,
    reserved: [Option<GrappleId>; 2],
}

impl EffectiveHands {
    /// Actual current rules are explicit because reducers may temporarily take
    /// them out of CampaignState. Missing source context never means free hands.
    pub fn current(
        state: &CampaignState,
        rules: &RulesState,
        actor: EntityId,
    ) -> Result<Self, RulesError> {
        Self::current_inner(state, rules, actor, true, None)
    }

    pub(crate) fn current_with_read(
        read: &crate::tactical::grapple::execution::ReadContext<'_>,
        actor: EntityId,
    ) -> Result<Self, RulesError> {
        let state = read.state();
        let rules = state.rules.as_ref().ok_or(RulesError::Uninitialized)?;
        Self::current_inner(state, rules, actor, true, Some(read))
    }

    pub(crate) fn attack(
        read: &crate::tactical::grapple::reads::AttackRead<'_>,
    ) -> Result<Self, RulesError> {
        let actor = read.actor();
        let mut result = Self {
            actor,
            reserved: [None; 2],
        };
        if !read.uses_hands() {
            return Ok(result);
        }
        let outgoing: Vec<_> = read
            .grips()
            .iter()
            .filter(|g| g.declaration.grappler == actor)
            .collect();
        if outgoing.is_empty() {
            return Ok(result);
        }
        let pack = RulesPack::from_json(include_str!("../../../content/srd-5.2.1/kernel.json"))?;
        let anatomy = ordinary_grapple_anatomy(read.state(), actor, &pack)?
            .ok_or_else(|| invalid("admitted hand has no actual source anatomy"))?;
        for grip in outgoing {
            let d = &grip.declaration;
            if d.anatomy != anatomy || result.reserved[d.hand.index()].replace(d.id).is_some() {
                return Err(invalid("admitted hand source or reservation differs"));
            }
        }
        Ok(result)
    }

    fn current_inner(
        state: &CampaignState,
        rules: &RulesState,
        actor: EntityId,
        durable: bool,
        read: Option<&crate::tactical::grapple::execution::ReadContext<'_>>,
    ) -> Result<Self, RulesError> {
        let mut result = Self {
            actor,
            reserved: [None; 2],
        };
        if rules.tactical_grapples.is_none()
            && !has_new_roll_authority(rules)
            && !has_unimplemented_grapple_records(state)
        {
            // Preserve old actions, including unannotated source anatomy. Do not
            // query new source capability or change old error ordering here.
            return Ok(result);
        }
        if state
            .rules
            .as_ref()
            .is_none_or(|stored| !std::ptr::eq(stored, rules))
        {
            return Err(invalid(
                "new hand authority requires its actual attached source context",
            ));
        }
        if !has_tactical_grapple_attachments(state) {
            if let Some(read) = read {
                read.require_guarded("orphaned Grapple roll authority has no hand context")?;
                // Only the actual owner supplies this read. Completed history
                // reserves no current hand; a standalone raw image still fails.
                return Ok(result);
            }
            return Err(invalid(
                "orphaned Grapple roll authority has no hand context",
            ));
        }
        if durable {
            validate_tactical_grapple_shapes(state).map_err(invalid)?;
        }
        let mut declarations = Vec::new();
        if let Some(live) = &rules.tactical_grapples {
            declarations.extend(live.active.iter().map(|grip| &grip.declaration));
        }
        if let Some(GrappleActivity::Attempt(attempt)) = state
            .encounter
            .as_ref()
            .and_then(|e| e.flow.as_ref())
            .and_then(|f| f.resolution.as_ref())
            .and_then(|r| r.grapple.as_ref())
            .and_then(|g| g.activity.as_ref())
            && attempt.reservation().is_some()
        {
            declarations.push(&attempt.declaration);
        }
        // Retained proofs and incoming relations never reserve current hands.
        // No own-attempt exclusion or historical cut is accepted by this live API.
        let outgoing = declarations
            .into_iter()
            .filter(|declaration| declaration.grappler == actor)
            .collect::<Vec<_>>();
        if outgoing.is_empty() {
            return Ok(result);
        }
        let pack = RulesPack::from_json(include_str!("../../../content/srd-5.2.1/kernel.json"))?;
        let anatomy = ordinary_grapple_anatomy(state, actor, &pack)?
            .ok_or_else(|| invalid("reserved hand has no authenticated ordinary anatomy"))?;
        for declaration in outgoing {
            if declaration.origin.campaign_id != state.campaign_id()
                || declaration.anatomy != anatomy
            {
                return Err(invalid(
                    "reserved hand source differs from its actual actor",
                ));
            }
            if result.reserved[declaration.hand.index()]
                .replace(declaration.id)
                .is_some()
            {
                return Err(invalid(
                    "one hand has multiple live or provisional reservations",
                ));
            }
        }
        Ok(result)
    }

    /// Immediate, provisional-only consumer. The excluded owned view never
    /// escapes and cannot be cloned/cached by its caller. Source/payment checks
    /// mint the borrowed token inside the same admission reconstruction.
    pub(crate) fn validate_attempt_equipment(
        proof: &crate::tactical::grapple::admission::AuthenticatedAttemptAdmission<'_>,
    ) -> Result<(), RulesError> {
        let state = proof.state();
        let attempt = proof.attempt();
        let attached = state
            .encounter
            .as_ref()
            .and_then(|e| e.flow.as_ref())
            .and_then(|f| f.resolution.as_ref())
            .and_then(|r| r.grapple.as_ref())
            .and_then(|g| g.activity.as_ref());
        if !matches!(attached, Some(GrappleActivity::Attempt(actual)) if std::ptr::eq(actual.as_ref(), attempt))
            || attempt.reservation().is_none()
        {
            return Err(invalid(
                "own-Attempt proof is not this attached provisional record",
            ));
        }
        let rules = state.rules.as_ref().ok_or(RulesError::Uninitialized)?;
        let d = &attempt.declaration;
        let mut hands = Self::current_inner(state, rules, d.grappler, false, None)?;
        if hands.reserved[d.hand.index()] != Some(d.id) {
            return Err(invalid("own-Attempt hand reservation differs"));
        }
        hands.reserved[d.hand.index()] = None;
        let mut physical = attempt.equipment.equipment_before.clone();
        crate::tactical_inventory::validate_loadout(state, &physical)
            .map_err(|e| invalid(e.to_string()))?;
        hands.validate_loadout(&physical.hands)?;
        if let Some(operation) = attempt.equipment.before_change {
            let definitions = crate::tactical_definitions::bundled_tactical_definitions()
                .map_err(|e| invalid(e.to_string()))?;
            crate::tactical_weapons::apply_attack_equipment_operation(
                state,
                d.grappler,
                d.window,
                definitions,
                &mut physical.hands,
                operation,
                &hands,
            )
            .map_err(|e| invalid(e.to_string()))?;
            physical.command = d.origin.clone();
        }
        if !hands.is_free(&physical.hands, d.hand)
            || rules
                .tactical_inventory
                .as_ref()
                .and_then(|i| i.loadout(d.grappler))
                != Some(&physical)
        {
            return Err(invalid("own-Attempt physical admission differs"));
        }
        Ok(())
    }

    pub fn actor(&self) -> EntityId {
        self.actor
    }

    pub fn has_reservation(&self) -> bool {
        self.reserved.iter().any(Option::is_some)
    }

    pub fn is_reserved(&self, hand: Hand) -> bool {
        self.reserved[hand.index()].is_some()
    }

    pub fn is_free(&self, physical: &WeaponLoadout, hand: Hand) -> bool {
        self.reserved[hand.index()].is_none()
            && physical.hands[hand.index()] == HandAssignment::Free
    }

    pub fn can_hold(&self, physical: &WeaponLoadout, hand: Hand, item: ItemId) -> bool {
        self.reserved[hand.index()].is_none()
            && (physical.hands[hand.index()] == HandAssignment::Free
                || physical.hands[hand.index()] == HandAssignment::Item(item))
    }

    pub fn holds(&self, physical: &WeaponLoadout, hand: Hand, item: ItemId) -> bool {
        self.reserved[hand.index()].is_none()
            && physical.hands[hand.index()] == HandAssignment::Item(item)
    }

    pub fn can_use_two_hands(&self, physical: &WeaponLoadout, item: ItemId) -> bool {
        physical.hands.contains(&HandAssignment::Item(item))
            && [Hand::Left, Hand::Right]
                .into_iter()
                .all(|hand| self.can_hold(physical, hand, item))
    }

    /// Kept outside tactical_inventory::validate_loadout: anatomy validation
    /// already reaches that physical-only primitive through source armor.
    pub fn validate_loadout(&self, physical: &WeaponLoadout) -> Result<(), RulesError> {
        if [Hand::Left, Hand::Right].into_iter().any(|hand| {
            self.reserved[hand.index()].is_some()
                && physical.hands[hand.index()] != HandAssignment::Free
        }) {
            return Err(invalid("physical item overlaps a reserved hand"));
        }
        Ok(())
    }
}

fn has_new_roll_authority(rules: &RulesState) -> bool {
    let new_role = |purpose: &PendingPurpose| {
        matches!(purpose,
        PendingPurpose::TacticalResolution { key, .. }
            if matches!(key.role, TacticalRollRole::GrappleSave | TacticalRollRole::GrappleEscape))
    };
    rules.pending.as_ref().is_some_and(|p| new_role(&p.purpose))
        || rules.rolls.iter().any(|r| new_role(&r.purpose))
}

fn invalid(message: impl Into<String>) -> RulesError {
    RulesError::Invalid(message.into())
}
