use dmd_domain::*;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MassCatalog {
    pub schema_version: u32,
    pub catalog_id: String,
    pub ruleset_id: String,
    pub ruleset_version: String,
    pub source_pdf_sha256: String,
    pub entries: Vec<MassEntry>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MassEntry {
    pub definition_id: String,
    pub source_pages: Vec<u16>,
    pub quantity_unit: String,
    pub currently_materializable: bool,
    pub mass: CatalogMass,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum CatalogMass {
    Intrinsic {
        mass: PhysicalMass,
    },
    ConditionalGross {
        condition: String,
        mass: PhysicalMass,
    },
    Subtype {
        variants: Vec<MassVariant>,
    },
    Unquantified,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MassVariant {
    pub id: String,
    pub mass: Option<PhysicalMass>,
}
pub fn catalog() -> &'static MassCatalog {
    static CATALOG: OnceLock<MassCatalog> = OnceLock::new();
    CATALOG.get_or_init(|| {
        serde_json::from_str(include_str!(
            "../../../../content/srd-5.2.1/equipment-mass-v1.json"
        ))
        .expect("reviewed mass catalog is valid")
    })
}
pub fn fingerprint(value: &impl Serialize) -> Result<String, String> {
    let bytes = serde_json::to_vec(value).map_err(|e| e.to_string())?;
    let hash = bytes.iter().fold(0xcbf29ce484222325u64, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x100000001b3)
    });
    Ok(format!("{hash:016x}"))
}
pub fn pin() -> Result<PhysicalCatalogPin, String> {
    let source = catalog();
    Ok(PhysicalCatalogPin {
        schema_version: source.schema_version,
        catalog_id: source.catalog_id.clone(),
        ruleset_id: source.ruleset_id.clone(),
        ruleset_version: source.ruleset_version.clone(),
        definition_fingerprint: fingerprint(source)?,
    })
}
pub fn entry(id: &str) -> Option<&'static MassEntry> {
    catalog()
        .entries
        .iter()
        .find(|entry| entry.definition_id == id)
}
pub fn amount(id: &str, condition: Option<&str>) -> Result<Option<PhysicalMass>, String> {
    Ok(
        match &entry(id)
            .ok_or("This object has no printed mass entry.")?
            .mass
        {
            CatalogMass::Intrinsic { mass } if condition.is_none() => Some(*mass),
            CatalogMass::Unquantified if condition.is_none() => None,
            CatalogMass::ConditionalGross {
                condition: required,
                mass,
            } if condition == Some(required.as_str()) => Some(*mass),
            CatalogMass::Subtype { variants } => {
                variants
                    .iter()
                    .find(|v| Some(v.id.as_str()) == condition)
                    .ok_or("Choose the actual symbol form.")?
                    .mass
            }
            _ => return Err("This physical condition is not in the printed source.".into()),
        },
    )
}
