use std::collections::{BTreeMap, BTreeSet};

use serde::Deserialize;

const LEDGER: &str = include_str!("../../../docs/rules/rules-coverage-ledger.json");
const INVENTORY: &str = include_str!("../../../docs/rules/srd-5.2.1-inventory.json");
const SOURCE: &str = include_str!("../../../content/srd-5.2.1/source.json");
const NOTICE: &str = include_str!("../../../content/srd-5.2.1/NOTICE.md");
const SOURCE_SHA256: &str = "8974902d109d6e63672d7c490bde9ccf052410503d9cfa768237154fbc5e3d87";

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Ledger {
    schema_version: u32,
    ruleset_id: String,
    version: String,
    source: String,
    inventory: String,
    families: Vec<Family>,
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Family {
    id: String,
    family: String,
    primary_gate: u8,
    source_pages: Vec<u16>,
    legal_to_ship: bool,
    status: String,
    mechanical_tests: Vec<String>,
    production_integration: Vec<String>,
    player_acceptance: Vec<String>,
    scope: String,
    final_acceptance_gate: u8,
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Inventory {
    schema_version: u32,
    ruleset_id: String,
    version: String,
    source_sha256: String,
    chapters: Vec<Chapter>,
    catalogs: BTreeMap<String, Vec<Entry>>,
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Chapter {
    title: String,
    first_page: u16,
    last_page: u16,
    family_ids: Vec<String>,
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Entry {
    name: String,
    page: u16,
    family_id: String,
}

fn fixtures() -> (Ledger, Inventory) {
    (
        serde_json::from_str(LEDGER).expect("valid ledger JSON"),
        serde_json::from_str(INVENTORY).expect("valid inventory JSON"),
    )
}

fn validate(ledger: &Ledger, inventory: &Inventory) -> Result<(), String> {
    if ledger.schema_version != 1
        || inventory.schema_version != 1
        || ledger.ruleset_id != "srd-5.2"
        || inventory.ruleset_id != ledger.ruleset_id
        || ledger.version != "5.2.1"
        || inventory.version != ledger.version
        || inventory.source_sha256 != SOURCE_SHA256
        || ledger.source != "../../content/srd-5.2.1/source.json"
        || ledger.inventory != "srd-5.2.1-inventory.json"
    {
        return Err("source identity mismatch".into());
    }
    let mut families = BTreeMap::new();
    for family in &ledger.families {
        if family.id.is_empty()
            || families.insert(family.id.as_str(), family).is_some()
            || family.family.trim().is_empty()
            || family.scope.trim().is_empty()
            || !(2..=6).contains(&family.primary_gate)
            || family.final_acceptance_gate != 14
            || !family.legal_to_ship
            || family.source_pages.is_empty()
            || family.source_pages.iter().any(|p| !(1..=364).contains(p))
        {
            return Err(format!("invalid or duplicate family {}", family.id));
        }
        let rank = match family.status.as_str() {
            "planned" | "intentionally_deferred" | "implementing" | "implemented" => 0,
            "mechanically_tested" => 1,
            "production_integrated" => 2,
            "player_accepted" => 3,
            _ => return Err(format!("invalid status for {}", family.id)),
        };
        for (required, evidence) in [
            (rank >= 1, &family.mechanical_tests),
            (rank >= 2, &family.production_integration),
            (rank >= 3, &family.player_acceptance),
        ] {
            if (required && evidence.is_empty()) || evidence.iter().any(|e| e.trim().is_empty()) {
                return Err(format!("missing completion evidence for {}", family.id));
            }
        }
    }
    let mut referenced = BTreeSet::new();
    let mut pages = BTreeSet::new();
    let mut titles = BTreeSet::new();
    for chapter in &inventory.chapters {
        if chapter.title.trim().is_empty()
            || !titles.insert(&chapter.title)
            || chapter.first_page > chapter.last_page
            || !(1..=364).contains(&chapter.first_page)
            || !(1..=364).contains(&chapter.last_page)
            || chapter.family_ids.is_empty()
        {
            return Err("invalid source chapter".into());
        }
        for page in chapter.first_page..=chapter.last_page {
            if !pages.insert(page) {
                return Err("overlapping source chapters".into());
            }
        }
        let mut chapter_families = BTreeSet::new();
        for id in &chapter.family_ids {
            if !families.contains_key(id.as_str()) || !chapter_families.insert(id) {
                return Err(format!("unassigned/duplicate chapter family {id}"));
            }
            referenced.insert(id.as_str());
        }
    }
    let expected_pages: BTreeSet<_> = [1].into_iter().chain(5..=364).collect();
    if pages != expected_pages || inventory.chapters.len() != 14 {
        return Err("source chapter coverage gap".into());
    }
    let expected_catalogs = BTreeMap::from([
        ("glossary", (155, 176, 191)),
        ("spells", (338, 107, 175)),
        ("magic_items", (258, 209, 253)),
        ("monsters", (330, 258, 364)),
        ("feats", (17, 87, 88)),
        ("classes", (12, 28, 82)),
        ("subclasses", (12, 28, 82)),
        ("backgrounds", (4, 83, 83)),
        ("species", (9, 84, 86)),
    ]);
    if inventory.catalogs.len() != expected_catalogs.len() {
        return Err("missing or unknown source catalog".into());
    }
    for (name, (count, first, last)) in expected_catalogs {
        let entries = inventory.catalogs.get(name).ok_or("missing catalog")?;
        if entries.len() != count {
            return Err(format!("source catalog count changed: {name}"));
        }
        let mut names = BTreeSet::new();
        for entry in entries {
            if entry.name.trim().is_empty()
                || !names.insert(entry.name.as_str())
                || !(first..=last).contains(&entry.page)
                || !families.contains_key(entry.family_id.as_str())
            {
                return Err(format!("invalid/unassigned {name} entry {}", entry.name));
            }
            referenced.insert(entry.family_id.as_str());
        }
    }
    if referenced != families.keys().copied().collect() {
        return Err("family not grounded in the source inventory".into());
    }
    Ok(())
}

#[test]
fn pinned_source_has_complete_owned_inventory() {
    let (ledger, inventory) = fixtures();
    assert_eq!(validate(&ledger, &inventory), Ok(()));
    let source: serde_json::Value = serde_json::from_str(SOURCE).unwrap();
    assert_eq!(source["ruleset_id"], ledger.ruleset_id);
    assert_eq!(source["version"], ledger.version);
    assert_eq!(source["license"], "CC-BY-4.0");
    assert_eq!(
        source["source_url"],
        "https://media.dndbeyond.com/compendium-images/srd/5.2/SRD_CC_v5.2.1.pdf"
    );
    assert_eq!(source["landing_url"], "https://www.dndbeyond.com/srd");
    assert_eq!(
        source["license_url"],
        "https://creativecommons.org/licenses/by/4.0/legalcode"
    );
    assert_eq!(source["publisher"], "Wizards of the Coast LLC");
    assert_eq!(source["sha256"], SOURCE_SHA256);
    assert_eq!(source["byte_len"], 6_031_375);
    assert_eq!(source["page_count"], 364);
    assert_eq!(source["published"], "2025-05-01");
    assert_eq!(source["notice"], "NOTICE.md");
    assert!(NOTICE.contains(concat!(
        "This work includes material from the System Reference Document 5.2.1 ",
        "(\"SRD 5.2.1\") by Wizards of the Coast LLC, available at ",
        "https://www.dndbeyond.com/srd. The SRD 5.2.1 is licensed under the ",
        "Creative Commons Attribution 4.0 International License, available at ",
        "https://creativecommons.org/licenses/by/4.0/legalcode."
    )));
    assert!(NOTICE.contains("DMd adaptations:"));
}

#[test]
fn missing_owner_or_source_member_is_rejected() {
    let (mut ledger, mut inventory) = fixtures();
    ledger.families.remove(0);
    assert!(validate(&ledger, &inventory).is_err());
    let (ledger, _) = fixtures();
    inventory.catalogs.get_mut("spells").unwrap().pop();
    assert!(validate(&ledger, &inventory).is_err());
    let (_, mut inventory) = fixtures();
    inventory.catalogs.get_mut("glossary").unwrap()[0].family_id = "unassigned".into();
    assert!(validate(&ledger, &inventory).is_err());
    let (mut ledger, inventory) = fixtures();
    ledger.families[0].primary_gate = 0;
    assert!(validate(&ledger, &inventory).is_err());
}

#[test]
fn duplicate_ownership_or_source_gaps_are_rejected() {
    let (mut ledger, inventory) = fixtures();
    ledger.families.push(ledger.families[0].clone());
    assert!(validate(&ledger, &inventory).is_err());
    let (ledger, mut inventory) = fixtures();
    inventory.chapters[1].first_page += 1;
    assert!(validate(&ledger, &inventory).is_err());
    let (_, mut inventory) = fixtures();
    let entries = inventory.catalogs.get_mut("monsters").unwrap();
    entries[1] = entries[0].clone();
    assert!(validate(&ledger, &inventory).is_err());
}

#[test]
fn completion_needs_mechanical_application_and_player_evidence() {
    for status in [
        "mechanically_tested",
        "production_integrated",
        "player_accepted",
    ] {
        let (mut ledger, inventory) = fixtures();
        let family = &mut ledger.families[0];
        family.status = status.into();
        family.mechanical_tests.clear();
        family.production_integration.clear();
        family.player_acceptance.clear();
        assert!(validate(&ledger, &inventory).is_err());
    }
    let (mut ledger, inventory) = fixtures();
    let family = &mut ledger.families[0];
    family.status = "player_accepted".into();
    family.mechanical_tests = vec!["test path and exact verified head".into()];
    family.production_integration =
        vec!["real application scenario and exact verified head".into()];
    family.player_acceptance = vec!["human acceptance report and build".into()];
    assert_eq!(validate(&ledger, &inventory), Ok(()));
}

#[test]
fn completion_stages_reject_missing_evidence_even_when_other_stages_have_it() {
    for (status, missing) in [
        ("mechanically_tested", "mechanical"),
        ("production_integrated", "mechanical"),
        ("production_integrated", "production"),
        ("player_accepted", "mechanical"),
        ("player_accepted", "production"),
        ("player_accepted", "player"),
    ] {
        let (mut ledger, inventory) = fixtures();
        let family = &mut ledger.families[0];
        family.status = status.into();
        family.mechanical_tests = vec!["mechanical test and verified head".into()];
        family.production_integration = vec!["application scenario and verified head".into()];
        family.player_acceptance = vec!["human acceptance report and build".into()];
        match missing {
            "mechanical" => family.mechanical_tests.clear(),
            "production" => family.production_integration.clear(),
            "player" => family.player_acceptance.clear(),
            _ => unreachable!(),
        }
        assert!(
            validate(&ledger, &inventory).is_err(),
            "{status}: {missing}"
        );
    }
}
