use super::*;
use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AttackAbility {
    Strength,
    Dexterity,
    Finesse,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AttackDefinition {
    pub id: String,
    pub ability: AttackAbility,
    pub damage: Vec<DieSpec>,
    pub damage_type: DamageType,
    pub ranged: bool,
    pub source_page: u16,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum SpellEffect {
    Healing {
        dice: Vec<DieSpec>,
        extra_dice_per_slot: Vec<DieSpec>,
    },
    AttackDamage {
        dice: Vec<DieSpec>,
        damage_type: DamageType,
        cantrip_scaling: bool,
    },
    /// Concentration/duration authority only; geometry/illumination waits for Gate 4.
    ConcentrationMarker { duration_seconds: u32 },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpellDefinition {
    pub id: String,
    pub level: u8,
    pub verbal: bool,
    pub somatic: bool,
    pub material: bool,
    pub effect: SpellEffect,
    pub source_page: u16,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RulesPack {
    pub id: String,
    pub version: String,
    pub kernel_schema_version: u32,
    pub attacks: Vec<AttackDefinition>,
    pub spells: Vec<SpellDefinition>,
}
impl RulesPack {
    pub fn from_json(json: &str) -> Result<Self, RulesError> {
        let pack: Self = serde_json::from_str(json)
            .map_err(|e| invalid(format!("rules definition JSON: {e}")))?;
        pack.validate()?;
        Ok(pack)
    }
    pub fn validate(&self) -> Result<(), RulesError> {
        if self.id != "srd-5.2" || self.version != "5.2.1" || self.kernel_schema_version != 1 {
            return Err(RulesError::Incompatible(
                "supported kernel is srd-5.2@5.2.1 schema 1".into(),
            ));
        }
        if self.attacks.is_empty() || self.spells.is_empty() {
            return Err(invalid("empty rules definitions"));
        }
        let mut ids = BTreeSet::new();
        for a in &self.attacks {
            if !valid_id(&a.id) || !ids.insert(&a.id) || !(1..=364).contains(&a.source_page) {
                return Err(invalid("invalid/duplicate attack definition"));
            }
            validate_dice(&a.damage)?;
        }
        ids.clear();
        for s in &self.spells {
            if !valid_id(&s.id)
                || !ids.insert(&s.id)
                || s.level > 9
                || !(1..=364).contains(&s.source_page)
            {
                return Err(invalid("invalid/duplicate spell definition"));
            }
            match &s.effect {
                SpellEffect::Healing {
                    dice,
                    extra_dice_per_slot,
                } => {
                    validate_dice(dice)?;
                    validate_dice(extra_dice_per_slot)?;
                    if s.level == 0 {
                        return Err(invalid("healing cantrip is unsupported"));
                    }
                }
                SpellEffect::AttackDamage {
                    dice,
                    cantrip_scaling,
                    ..
                } => {
                    validate_dice(dice)?;
                    if s.level != 0 || !cantrip_scaling {
                        return Err(invalid(
                            "only scaling attack cantrips are supported in this kernel",
                        ));
                    }
                }
                SpellEffect::ConcentrationMarker { duration_seconds } => {
                    if *duration_seconds == 0 || *duration_seconds > 86400 {
                        return Err(invalid("invalid spell duration"));
                    }
                }
            }
        }
        Ok(())
    }
    pub fn attack(&self, id: &str) -> Result<&AttackDefinition, RulesError> {
        self.attacks
            .iter()
            .find(|a| a.id == id)
            .ok_or_else(|| prerequisite("attack definition is unavailable"))
    }
    pub fn spell(&self, id: &str) -> Result<&SpellDefinition, RulesError> {
        self.spells
            .iter()
            .find(|a| a.id == id)
            .ok_or_else(|| prerequisite("spell definition is unavailable"))
    }
}
pub(super) fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 100
        && id
            .bytes()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || b"-._".contains(&c))
}
pub(super) fn validate_dice(dice: &[DieSpec]) -> Result<(), RulesError> {
    if dice.is_empty()
        || dice.len() > 10
        || dice.iter().any(|d| {
            d.count == 0 || d.count > 100 || ![4, 6, 8, 10, 12, 20, 100].contains(&d.sides)
        })
    {
        return Err(invalid("invalid content dice"));
    }
    Ok(())
}
