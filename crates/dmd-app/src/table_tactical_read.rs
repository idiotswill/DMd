//! Read-only mechanical affordances borrow their exact image, never a state/proof pair.
use dmd_domain::*;
use dmd_rules::{
    RulesError, table::TableRead, tactical_hands::EffectiveHands, tactical_spells::BoundSpell,
};

#[derive(Clone, Copy)]
pub(crate) enum TacticalRead<'a> {
    Ordinary(&'a CampaignState),
    Owned(&'a TableRead<'a>),
}
impl<'a> TacticalRead<'a> {
    pub(crate) fn state(self) -> &'a CampaignState {
        match self {
            Self::Ordinary(state) => state,
            Self::Owned(read) => read.state(),
        }
    }
    pub(crate) fn hands(self, actor: EntityId) -> Result<EffectiveHands, RulesError> {
        match self {
            Self::Ordinary(state) => EffectiveHands::current(
                state,
                state.rules.as_ref().ok_or(RulesError::Uninitialized)?,
                actor,
            ),
            Self::Owned(read) => read.effective_hands(actor),
        }
    }
    pub(crate) fn ground_pickup_options(
        self,
        actor: EntityId,
        pack: &dmd_rules::RulesPack,
    ) -> Result<
        Vec<dmd_rules::tactical_weapons::GroundPickupOption>,
        dmd_rules::tactical_weapons::WeaponError,
    > {
        match self {
            Self::Ordinary(state) => {
                dmd_rules::tactical_weapons::ground_pickup_options(state, actor, pack)
            }
            Self::Owned(read) => read.ground_pickup_options(actor),
        }
    }
    pub(crate) fn attack_equipment_options(
        self,
        pack: &dmd_rules::RulesPack,
    ) -> Result<Option<dmd_rules::tactical::AttackEquipmentOptions>, RulesError> {
        match self {
            Self::Ordinary(state) => dmd_rules::tactical::attack_equipment_options(state, pack),
            Self::Owned(read) => read.attack_equipment_options(),
        }
    }
    pub(crate) fn physical_source_opportunity_grips(
        self,
        actor: EntityId,
        feature: &str,
        item: ItemId,
    ) -> Result<Vec<WeaponGrip>, RulesError> {
        match self {
            Self::Ordinary(state) => {
                dmd_rules::tactical::physical_source_opportunity_grips(state, actor, feature, item)
            }
            Self::Owned(read) => read.physical_source_opportunity_grips(actor, feature, item),
        }
    }
    pub(crate) fn bind_spell(
        self,
        plan: &SpellCastPlan,
        choice: &SpellTargetChoice,
    ) -> Result<BoundSpell, RulesError> {
        match self {
            Self::Ordinary(state) => dmd_rules::tactical_spells::bind_spell(state, plan, choice),
            Self::Owned(read) => read.bind_spell(plan, choice),
        }
    }
    pub(crate) fn shield_choices(
        self,
        actor: EntityId,
    ) -> Result<Vec<SpellCastChoice>, RulesError> {
        match self {
            Self::Ordinary(state) => dmd_rules::tactical::shield_choices(state, actor),
            Self::Owned(read) => read.shield_choices(actor),
        }
    }
}
