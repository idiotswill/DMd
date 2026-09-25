use super::*;
use crate::tactical_definitions::{AreaShape, EffectDescriptor, FeatureActivation, MonsterFeature};

/// Canonical supported source program. No Serde/public fields allow a caller to
/// manufacture a DC, amount, area dimension, resource or magical capability.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AreaProgram {
    actor: EntityId,
    source: CreatureSourcePin,
    feature_id: String,
    length: u32,
    ability: Ability,
    dc: u16,
    dice: Vec<DieSpec>,
    modifier: i32,
    damage_type: DamageType,
    half_on_success: bool,
    source_page: u16,
}
impl AreaProgram {
    pub fn actor(&self) -> EntityId {
        self.actor
    }
    pub fn source(&self) -> &CreatureSourcePin {
        &self.source
    }
    pub fn feature_id(&self) -> &str {
        &self.feature_id
    }
    pub fn length(&self) -> u32 {
        self.length
    }
    pub fn ability(&self) -> Ability {
        self.ability
    }
    pub fn dc(&self) -> u16 {
        self.dc
    }
    pub fn dice(&self) -> &[DieSpec] {
        &self.dice
    }
    pub fn modifier(&self) -> i32 {
        self.modifier
    }
    pub fn damage_type(&self) -> DamageType {
        self.damage_type
    }
    pub fn half_on_success(&self) -> bool {
        self.half_on_success
    }
    pub fn source_page(&self) -> u16 {
        self.source_page
    }
}

/// Reading the program is not permission to use it. Accepted admission additionally
/// runs the real creature scheduler, proves its invocation and spends its Action.
pub fn source_area_program(
    state: &CampaignState,
    actor: EntityId,
    feature_id: &str,
) -> Result<AreaProgram, RulesError> {
    let profile = state
        .rules
        .as_ref()
        .and_then(|rules| rules.tactical_creatures.as_ref())
        .and_then(|creatures| creatures.profile(actor))
        .ok_or_else(|| unavailable("an area action requires its actual source creature"))?;
    let definition = crate::tactical_creatures::source_for_profile(profile)
        .map_err(|e| invalid(e.to_string()))?;
    let named = definition
        .features
        .iter()
        .find(|f| f.id == feature_id)
        .ok_or_else(|| unavailable("this source creature has no selected area action"))?;
    let MonsterFeature::SaveArea {
        dc, area, effect, ..
    } = &named.feature
    else {
        return Err(unavailable(
            "the selected source feature is not an area save",
        ));
    };
    let (
        AreaShape::Cone { length_feet },
        EffectDescriptor::SaveDamage {
            ability,
            damage,
            half_on_success,
            push_on_failure_feet: 0,
        },
    ) = (area, effect)
    else {
        return Err(unavailable(
            "this source area requires its complete additional execution path",
        ));
    };
    if named.activation != FeatureActivation::Action || named.usage.is_some() {
        return Err(unavailable(
            "this source area requires its enclosing activation",
        ));
    }
    Ok(AreaProgram {
        actor,
        source: profile.source.clone(),
        feature_id: named.id.clone(),
        length: u32::from(*length_feet) * SPATIAL_UNITS_PER_FOOT,
        ability: *ability,
        dc: u16::from(*dc),
        dice: damage.amount.dice.clone(),
        modifier: i32::from(damage.amount.fixed),
        damage_type: damage.damage_type,
        half_on_success: *half_on_success,
        source_page: named.source_page,
    })
}
