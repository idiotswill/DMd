//! Closed, read-only policy for the reviewed Ogre source. This does not authorize
//! an actor, plan an attack or override ordinary equipment damage. The physical
//! own-turn/OA adapters resolve this exact pin themselves and still
//! validate current custody, hands, costs and retained admission before use.
use super::*;
use crate::tactical_definitions::{
    AttackDelivery, DamageComponent, FeatureActivation, MonsterFeature, WeaponDefinition,
    WeaponHands, WeaponKind, WeaponProperty, bundled_ogre,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OgreWeaponProgram {
    source: CreatureSourcePin,
    feature_id: &'static str,
    weapon: &'static WeaponDefinition,
    delivery: WeaponDelivery,
    attack_bonus: i16,
    source_delivery: &'static AttackDelivery,
    printed_base_damage: &'static DamageComponent,
}

impl OgreWeaponProgram {
    pub fn source(&self) -> &CreatureSourcePin {
        &self.source
    }
    pub fn feature_id(&self) -> &'static str {
        self.feature_id
    }
    pub fn weapon(&self) -> &'static WeaponDefinition {
        self.weapon
    }
    pub fn delivery(&self) -> WeaponDelivery {
        self.delivery
    }
    pub fn ability(&self) -> Ability {
        Ability::Strength
    }
    pub fn attack_bonus(&self) -> i16 {
        self.attack_bonus
    }
    pub fn source_delivery(&self) -> &'static AttackDelivery {
        self.source_delivery
    }
    pub fn printed_base_damage(&self) -> &'static DamageComponent {
        self.printed_base_damage
    }
}

/// Source descriptor only; consuming attack adapters do not open profile admission.
pub fn ogre_weapon_program(
    pin: &CreatureSourcePin,
    feature_id: &str,
) -> Result<OgreWeaponProgram, CreatureError> {
    let source = creature_source(pin)?;
    if !std::ptr::eq(source, bundled_ogre().map_err(|e| invalid(e.to_string()))?) {
        return Err(invalid(
            "physical Ogre policy requires the exact immutable Ogre pin",
        ));
    }
    let (feature_id, weapon_id, delivery) = match feature_id {
        "greatclub" => ("greatclub", "greatclub", WeaponDelivery::Melee),
        "javelin-melee" => ("javelin-melee", "javelin", WeaponDelivery::Melee),
        "javelin-thrown" => ("javelin-thrown", "javelin", WeaponDelivery::Thrown),
        _ => return Err(invalid("unknown Ogre physical form")),
    };
    let feature = source
        .features
        .iter()
        .find(|f| f.id == feature_id)
        .ok_or_else(|| invalid("Ogre physical feature is absent"))?;
    let MonsterFeature::Attack {
        bonus,
        delivery: source_delivery,
        damage,
        extra_damage_if_attack_had_advantage,
        conditional_hits,
    } = &feature.feature
    else {
        return Err(invalid("Ogre physical form is not a printed attack"));
    };
    let weapon = creature_definitions()?
        .weapon(weapon_id)
        .ok_or_else(|| invalid("Ogre physical weapon is absent"))?;
    let shape_matches = match (weapon_id, source_delivery, delivery) {
        ("greatclub", AttackDelivery::Melee { reach_feet: 5 }, WeaponDelivery::Melee) => {
            weapon.hands == WeaponHands::Two
                && weapon.properties.contains(&WeaponProperty::TwoHanded)
        }
        ("javelin", AttackDelivery::Melee { reach_feet: 5 }, WeaponDelivery::Melee)
        | ("javelin", AttackDelivery::Ranged { .. }, WeaponDelivery::Thrown) => {
            weapon.hands == WeaponHands::One
                && weapon.properties.contains(&WeaponProperty::Thrown)
                && match source_delivery {
                    AttackDelivery::Ranged { range } => weapon.range.as_ref() == Some(range),
                    AttackDelivery::Melee { .. } => true,
                }
        }
        _ => false,
    };
    if !shape_matches
        || weapon.kind != WeaponKind::Melee
        || weapon.ammunition.is_some()
        || !source.statistics.gear.iter().any(|id| id == weapon_id)
        || source.ordinary_hands != Some(OrdinaryHandAnatomy::TwoHandsV1)
        || feature.activation != FeatureActivation::Action
        || feature.usage.is_some()
        || damage.len() != 1
        || damage[0].damage_type != weapon.damage_type
        || !extra_damage_if_attack_had_advantage.is_empty()
        || !conditional_hits.is_empty()
    {
        return Err(invalid(
            "Ogre physical source and retained weapon policy differ",
        ));
    }
    Ok(OgreWeaponProgram {
        source: pin.clone(),
        feature_id,
        weapon,
        delivery,
        attack_bonus: *bonus,
        source_delivery,
        printed_base_damage: &damage[0],
    })
}
