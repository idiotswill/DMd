//! Accepted physical facts. Structural data is not a capability to admit a fact.
use crate::*;
use serde::{Deserialize, Serialize};

pub const PHYSICAL_FACTS_VERSION: u32 = 1;
pub const MICRO_POUNDS_PER_POUND: u64 = 1_000_000;
pub const CUSTOM_LOAD_DEFINITION: &str = "dmd:personal-load-v1";
pub const CURRENCY_LOT_DEFINITION: &str = "dmd:currency-lot-v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PhysicalCatalogPin {
    pub schema_version: u32,
    pub catalog_id: String,
    pub ruleset_id: String,
    pub ruleset_version: String,
    pub definition_fingerprint: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PhysicalMass(pub u64);

impl PhysicalMass {
    pub fn parse_pounds(value: &str) -> Result<Self, String> {
        if value.is_empty()
            || value.len() > 27
            || !value.bytes().all(|b| b.is_ascii_digit() || b == b'.')
        {
            return Err(
                "Enter pounds as a positive decimal with at most six decimal places.".into(),
            );
        }
        let (whole, fraction) = value.split_once('.').unwrap_or((value, ""));
        if whole.is_empty()
            || fraction.len() > 6
            || value.ends_with('.')
            || (whole.len() > 1 && whole.starts_with('0'))
        {
            return Err("Enter a bounded decimal number of pounds.".into());
        }
        let whole = whole
            .parse::<u128>()
            .map_err(|_| "Pounds exceed the supported range.")?;
        let fraction = if fraction.is_empty() {
            0
        } else {
            fraction
                .parse::<u128>()
                .map_err(|_| "Invalid decimal pounds.")?
                .checked_mul(
                    10u128
                        .pow(6 - u32::try_from(fraction.len()).map_err(|_| "Invalid precision.")?),
                )
                .ok_or("Pounds overflow.")?
        };
        let units = whole
            .checked_mul(u128::from(MICRO_POUNDS_PER_POUND))
            .and_then(|n| n.checked_add(fraction))
            .ok_or("Pounds overflow.")?;
        let units = u64::try_from(units).map_err(|_| "Pounds exceed the supported range.")?;
        if units == 0 {
            return Err(
                "A physical mass must be positive; mark an unknown value explicitly.".into(),
            );
        }
        Ok(Self(units))
    }

    #[must_use]
    pub fn pounds(self) -> String {
        let whole = self.0 / MICRO_POUNDS_PER_POUND;
        let remainder = self.0 % MICRO_POUNDS_PER_POUND;
        if remainder == 0 {
            whole.to_string()
        } else {
            format!("{whole}.{:06}", remainder)
                .trim_end_matches('0')
                .to_owned()
        }
    }

    pub fn times(self, quantity: u32) -> Result<Self, String> {
        u64::try_from(u128::from(self.0) * u128::from(quantity))
            .map(Self)
            .map_err(|_| "Physical mass overflow.".into())
    }

    pub fn plus(self, other: Self) -> Result<Self, String> {
        u64::try_from(u128::from(self.0) + u128::from(other.0))
            .map(Self)
            .map_err(|_| "Physical mass overflow.".into())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PhysicalCoins {
    pub cp: u32,
    pub sp: u32,
    pub ep: u32,
    pub gp: u32,
    pub pp: u32,
}
impl PhysicalCoins {
    pub fn totals(self) -> Result<(u32, u32), String> {
        let counts = [self.cp, self.sp, self.ep, self.gp, self.pp];
        let count: u128 = counts.iter().map(|n| u128::from(*n)).sum();
        let value: u128 = counts
            .iter()
            .zip([1u32, 10, 50, 100, 1000])
            .map(|(n, v)| u128::from(*n) * u128::from(v))
            .sum();
        Ok((
            u32::try_from(count).map_err(|_| "Too many coins.")?,
            u32::try_from(value).map_err(|_| "Coin value overflow.")?,
        ))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PhysicalSubject {
    Body(EntityId),
    Item(ItemId),
    Coverage(EntityId),
    Currency(CharacterId),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum PhysicalItemMass {
    Unknown,
    /// Host classification of an ordinary catalog object or an explicit subtype/condition.
    Catalog {
        condition: Option<String>,
    },
    Unit {
        mass: PhysicalMass,
        condition: Option<String>,
    },
    Gross {
        mass: PhysicalMass,
    },
    /// Explicit rejection of ordinary catalog applicability for this physical object.
    NonstandardUnit {
        description: String,
        mass: PhysicalMass,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum PhysicalFactValue {
    Body {
        identity: String,
        mass: Option<PhysicalMass>,
    },
    Item {
        definition_id: String,
        quantity: u32,
        state: ItemState,
        contents: Vec<ItemId>,
        mass: PhysicalItemMass,
    },
    /// All specifically recorded additional payload is now ordinary physical items.
    Coverage {
        identity: String,
        allocations: String,
        complete: bool,
    },
    Currency {
        identity: String,
        value_cp: u32,
        item: ItemId,
        coins: PhysicalCoins,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PhysicalFact {
    pub subject: PhysicalSubject,
    pub origin: CommandMeta,
    pub previous: Option<CommandId>,
    pub value: PhysicalFactValue,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PhysicalFacts {
    pub schema_version: u32,
    pub catalog: PhysicalCatalogPin,
    pub origin: CommandMeta,
    pub records: Vec<PhysicalFact>,
}
impl PhysicalFacts {
    #[must_use]
    pub fn get(&self, subject: PhysicalSubject) -> Option<&PhysicalFact> {
        self.records.iter().find(|record| record.subject == subject)
    }
}

/// Opaque presentation binds this exact current subject, source and previous fact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum PhysicalFactOffer {
    Enable {
        catalog: PhysicalCatalogPin,
    },
    Body {
        actor: EntityId,
        identity: String,
        previous: Option<CommandId>,
    },
    Item {
        item: ItemInstance,
        contents: Vec<ItemId>,
        previous: Option<CommandId>,
    },
    Coverage {
        actor: EntityId,
        identity: String,
        allocations: String,
        previous: Option<CommandId>,
    },
    PersonalItem {
        actor: EntityId,
        identity: String,
    },
    Currency {
        character: CharacterId,
        identity: String,
        value_cp: u32,
        lot: Option<ItemInstance>,
        previous: Option<CommandId>,
    },
}

/// User-entered fields never carry a catalog amount, authoritative total or new ItemId.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum PhysicalFactInput {
    Enable,
    Body {
        pounds: Option<String>,
        reason: String,
    },
    Item {
        choice: PhysicalItemInput,
        reason: String,
    },
    Coverage {
        complete: bool,
        reason: String,
    },
    PersonalItem {
        name: String,
        pounds: String,
        separate_from_listed: bool,
        reason: String,
    },
    Currency {
        coins: PhysicalCoins,
        reason: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum PhysicalItemInput {
    Unknown,
    Catalog {
        condition: Option<String>,
    },
    Unit {
        pounds: String,
        condition: Option<String>,
    },
    Gross {
        pounds: String,
    },
    NonstandardUnit {
        description: String,
        pounds: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PhysicalFactAcceptance {
    pub offer: PhysicalFactOffer,
    pub input: PhysicalFactInput,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PhysicalLoad {
    pub items: Vec<PhysicalItemLoad>,
    pub known_pounds: String,
    pub complete: bool,
    pub unresolved: Vec<String>,
    pub body_and_load_pounds: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PhysicalItemLoad {
    pub item: ItemId,
    pub name: String,
    pub quantity: u32,
    pub pounds: Option<String>,
    pub basis: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_fractional_units_coin_counts_and_largest_mass_round_trip_without_rounding() {
        for text in [
            "0.000001",
            "0.02",
            "0.05",
            "0.075",
            "0.2",
            "180.25",
            "18446744073709.551615",
        ] {
            let mass = PhysicalMass::parse_pounds(text).unwrap();
            assert_eq!(mass.pounds(), text);
        }
        assert_eq!(
            PhysicalMass::parse_pounds("0.075")
                .unwrap()
                .times(40)
                .unwrap()
                .pounds(),
            "3"
        );
        let gold = PhysicalCoins {
            cp: 0,
            sp: 0,
            ep: 0,
            gp: 1,
            pp: 0,
        };
        let copper = PhysicalCoins {
            cp: 100,
            sp: 0,
            ep: 0,
            gp: 0,
            pp: 0,
        };
        assert_eq!(gold.totals().unwrap(), (1, 100));
        assert_eq!(copper.totals().unwrap(), (100, 100));
        assert_eq!(
            PhysicalMass(20_000)
                .times(copper.totals().unwrap().0)
                .unwrap()
                .pounds(),
            "2"
        );
    }
    #[test]
    fn unknown_is_not_zero_and_invalid_or_overflowing_physical_arithmetic_refuses() {
        for text in [
            "",
            "0",
            "0.000000",
            "-1",
            "+1",
            " 1",
            "1 ",
            ".5",
            "1.",
            "01",
            "1e3",
            "1.0000001",
            "1.2.3",
            "18446744073709.551616",
        ] {
            assert!(PhysicalMass::parse_pounds(text).is_err(), "{text}");
        }
        assert!(PhysicalMass(u64::MAX).times(2).is_err());
        assert!(PhysicalMass(u64::MAX).plus(PhysicalMass(1)).is_err());
        assert!(
            PhysicalCoins {
                cp: u32::MAX,
                sp: 1,
                ep: 0,
                gp: 0,
                pp: 0
            }
            .totals()
            .is_err()
        );
        assert!(
            PhysicalCoins {
                cp: 0,
                sp: 0,
                ep: 0,
                gp: 0,
                pp: u32::MAX
            }
            .totals()
            .is_err()
        );
    }
}
