//! Owned semantic continuity; durable audit and presentation authentication stays in app.
use super::*;
use crate::tactical::grapple::execution::{ExecutionContext, ReadContext};
use crate::{RulesAnswer, RulesError, RulesPack, RulesQuery};

/// Only this module constructs a closed image, after a complete validated transition.
pub(crate) struct ClosedImage {
    state: CampaignState,
}
impl ClosedImage {
    pub(crate) fn state(&self) -> &CampaignState {
        &self.state
    }
}

/// An original anchor and the exact transitions that followed it. Never a saved capability.
pub struct CampaignExecution {
    image: ClosedImage,
    pack: RulesPack,
}

/// Immutable borrowing of one particular certified image, never an arbitrary matching head.
pub struct TableRead<'a> {
    image: &'a ClosedImage,
    pack: &'a RulesPack,
}

pub struct PreparedTableStep<'a> {
    owner: &'a mut CampaignExecution,
    candidate: ClosedImage,
    produced: TableProduced,
}

/// Keeps the exact prior image alive while app verifies historical audience and bindings.
pub struct AppliedTableStep<'a> {
    before: ClosedImage,
    after: &'a CampaignExecution,
    produced: TableProduced,
}

pub enum LegacyRulesEventRef<'a> {
    Rules(&'a RulesEvent),
    Tactical(&'a crate::tactical::TacticalEvent),
}

impl CampaignExecution {
    pub fn from_original_anchor(anchor: CampaignState, pack: RulesPack) -> Result<Self, String> {
        reducer::validate_table(&anchor, &pack)?;
        if has_unimplemented_grapple_records(&anchor)
            || super::grapple_enabled(&anchor)
            || anchor.encounter_history.is_some()
            || source_control::enabled(&anchor)
            || anchor.encounter.as_ref().is_some_and(|e| e.flow.is_some())
            || anchor.rules.as_ref().is_some_and(|rules| {
                rules.tactical_effects.is_some()
                    || rules.tactical_inventory.is_some()
                    || rules.tactical_recovery.is_some()
                    || rules.tactical_creatures.is_some()
            })
        {
            return Err("tactical recovery requires its original pre-tactical anchor".into());
        }
        Ok(Self {
            image: ClosedImage { state: anchor },
            pack,
        })
    }

    pub fn read(&self) -> TableRead<'_> {
        TableRead {
            image: &self.image,
            pack: &self.pack,
        }
    }

    pub fn prepare_table<'a>(
        &'a mut self,
        meta: &CommandMeta,
        operation: &TableOperation,
    ) -> Result<PreparedTableStep<'a>, String> {
        self.prepare(meta, operation, None)
    }

    pub fn prepare_table_replay<'a>(
        &'a mut self,
        meta: &CommandMeta,
        operation: &TableOperation,
        expected: ExpectedNested<'_>,
    ) -> Result<PreparedTableStep<'a>, String> {
        self.prepare(meta, operation, Some(expected))
    }

    fn prepare<'a>(
        &'a mut self,
        meta: &CommandMeta,
        operation: &TableOperation,
        expected: Option<ExpectedNested<'_>>,
    ) -> Result<PreparedTableStep<'a>, String> {
        self.read().validate()?;
        let mut next = Box::new(self.image.state.clone());
        let mut execution = if super::grapple_enabled(self.image.state())
            || matches!(operation, TableOperation::EnableGrappleAccess)
        {
            ExecutionContext::for_table(&self.image, &next, meta)
        } else {
            ExecutionContext::ordinary()
        };
        let produced = reducer::apply_operation(
            self.image.state(),
            &mut next,
            meta,
            operation,
            &self.pack,
            expected,
            &mut execution,
        )?;
        execution.validate_delta(&next).map_err(|e| e.to_string())?;
        drop(execution);
        if let Some(expected) = expected
            && (produced.rules_event.as_ref() != expected.rules_event
                || produced.tactical_event.as_ref() != expected.tactical_event)
        {
            return Err("Table source event disagrees with deterministic replay.".into());
        }
        next.applied_event_sequence = meta
            .expected_event_sequence
            .checked_add(1)
            .ok_or("table command sequence overflow")?;
        let candidate = ClosedImage { state: *next };
        TableRead {
            image: &candidate,
            pack: &self.pack,
        }
        .validate()?;
        Ok(PreparedTableStep {
            owner: self,
            candidate,
            produced,
        })
    }

    pub fn prepare_legacy_replay<'a>(
        &'a mut self,
        event: LegacyRulesEventRef<'_>,
    ) -> Result<PreparedTableStep<'a>, String> {
        if self.image.state.table.is_some() {
            return Err("raw rules/tactical event bypasses the table command boundary".into());
        }
        let (mut state, produced, meta) = match event {
            LegacyRulesEventRef::Rules(event) => {
                let transition = crate::replay(self.image.state(), event, &self.pack)
                    .map_err(|e| e.to_string())?;
                (
                    transition.next_state,
                    TableProduced {
                        message: String::new(),
                        mechanics: Some(transition.outcome),
                        rules_event: Some(transition.event),
                        tactical_event: None,
                        session_change: None,
                    },
                    &event.meta,
                )
            }
            LegacyRulesEventRef::Tactical(event) => {
                let transition =
                    crate::tactical::replay_tactical(self.image.state(), event, &self.pack)
                        .map_err(|e| e.to_string())?;
                (
                    transition.next_state,
                    TableProduced {
                        message: String::new(),
                        mechanics: None,
                        rules_event: None,
                        tactical_event: Some(transition.event),
                        session_change: None,
                    },
                    &event.meta,
                )
            }
        };
        state.applied_event_sequence = meta
            .expected_event_sequence
            .checked_add(1)
            .ok_or("table command sequence overflow")?;
        let candidate = ClosedImage { state };
        TableRead {
            image: &candidate,
            pack: &self.pack,
        }
        .validate()?;
        Ok(PreparedTableStep {
            owner: self,
            candidate,
            produced,
        })
    }

    pub fn into_state(self) -> CampaignState {
        self.image.state
    }
}

impl<'a> TableRead<'a> {
    pub fn state(&self) -> &'a CampaignState {
        self.image.state()
    }
    pub fn pack(&self) -> &'a RulesPack {
        self.pack
    }
    pub(crate) fn context(&self) -> ReadContext<'a> {
        ReadContext::closed(self.image)
    }
    pub fn validate(&self) -> Result<(), String> {
        reducer::validate_table_with_read(&self.context(), self.pack)
    }
    pub fn query(
        &self,
        issuer: CommandIssuer,
        query: &RulesQuery,
    ) -> Result<RulesAnswer, RulesError> {
        crate::kernel::query_with_read(&self.context(), issuer, query, self.pack)
    }
    pub fn savage_attacker_dice(&self) -> Result<usize, RulesError> {
        crate::tactical::savage_attacker_dice_with_read(&self.context(), self.pack)
    }
    pub fn effective_hands(
        &self,
        actor: EntityId,
    ) -> Result<crate::tactical_hands::EffectiveHands, RulesError> {
        crate::tactical_hands::EffectiveHands::current_with_read(&self.context(), actor)
    }
    pub fn ground_pickup_options(
        &self,
        actor: EntityId,
    ) -> Result<
        Vec<crate::tactical_weapons::GroundPickupOption>,
        crate::tactical_weapons::WeaponError,
    > {
        crate::tactical_weapons::ground::ground_pickup_options_with_read(
            &self.context(),
            actor,
            self.pack,
        )
    }
    pub fn attack_equipment_options(
        &self,
    ) -> Result<Option<crate::tactical::AttackEquipmentOptions>, RulesError> {
        crate::tactical::attack_equipment_options_with_read(&self.context(), self.pack)
    }
    pub fn physical_source_opportunity_grips(
        &self,
        actor: EntityId,
        feature: &str,
        item: ItemId,
    ) -> Result<Vec<WeaponGrip>, RulesError> {
        crate::tactical::physical_source_opportunity_grips_with_read(
            &self.context(),
            actor,
            feature,
            item,
        )
    }
    pub fn bind_spell(
        &self,
        plan: &SpellCastPlan,
        choice: &SpellTargetChoice,
    ) -> Result<crate::tactical_spells::BoundSpell, RulesError> {
        crate::tactical_spells::bind_spell_with_read(&self.context(), plan, choice)
    }
    pub fn shield_choices(&self, actor: EntityId) -> Result<Vec<SpellCastChoice>, RulesError> {
        crate::tactical::shield_choices_with_read(&self.context(), actor)
    }
    pub fn grapple_choices(
        &self,
        issuer: CommandIssuer,
    ) -> Result<Vec<TableGrappleOffer>, RulesError> {
        crate::tactical::grapple::table_choices(&self.context(), issuer, self.pack)
    }
}

impl PreparedTableStep<'_> {
    pub fn before(&self) -> TableRead<'_> {
        self.owner.read()
    }
    pub fn after(&self) -> TableRead<'_> {
        TableRead {
            image: &self.candidate,
            pack: &self.owner.pack,
        }
    }
    pub fn produced(&self) -> &TableProduced {
        &self.produced
    }
}
impl<'a> PreparedTableStep<'a> {
    pub fn commit(self) -> AppliedTableStep<'a> {
        let Self {
            owner,
            candidate,
            produced,
        } = self;
        let before = std::mem::replace(&mut owner.image, candidate);
        AppliedTableStep {
            before,
            after: owner,
            produced,
        }
    }
}
impl AppliedTableStep<'_> {
    pub fn before(&self) -> TableRead<'_> {
        TableRead {
            image: &self.before,
            pack: &self.after.pack,
        }
    }
    pub fn after(&self) -> TableRead<'_> {
        self.after.read()
    }
    pub fn produced(&self) -> &TableProduced {
        &self.produced
    }
}
