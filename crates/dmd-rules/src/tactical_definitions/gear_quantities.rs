//! Preserve duplicate-key evidence instead of silently keeping a map's last value.
use std::collections::BTreeMap;

use serde::Deserializer;
use serde::de::{Error, MapAccess, Visitor};

pub(super) fn deserialize<'de, D>(deserializer: D) -> Result<BTreeMap<String, u32>, D::Error>
where
    D: Deserializer<'de>,
{
    struct Quantities;
    impl<'de> Visitor<'de> for Quantities {
        type Value = BTreeMap<String, u32>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str("a gear quantity map with unique definition keys")
        }

        fn visit_map<A>(self, mut entries: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'de>,
        {
            let mut quantities = BTreeMap::new();
            while let Some((id, quantity)) = entries.next_entry::<String, u32>()? {
                if quantities.insert(id, quantity).is_some() {
                    return Err(A::Error::custom("duplicate gear quantity definition"));
                }
            }
            Ok(quantities)
        }
    }
    deserializer.deserialize_map(Quantities)
}
