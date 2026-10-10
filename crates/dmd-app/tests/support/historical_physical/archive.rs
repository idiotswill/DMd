//! Immutable original-build inputs. These checks precede every application restore.
use super::*;
use serde::de::{MapAccess, SeqAccess, Visitor};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

pub const PRODUCER: &str = "969627f90d2a44643c7f33d9b1a205500337976f";
const TREE: &str = "84c74d1f3372df7913fad5eecbd13775f6b1e4ac";
const BASE: &str = "58696ac1d0c55ef7f71cfb546fb92e97747ee437";
const EXECUTABLE: &str = "30107621ee41679a86a12b7ff3897c68947057bc964b3890d5a86e1274d5f709";
const RECEIPT: &str = "f0313a56f1d5eea573b3f4bc3e7b4bad7ab66dfee9416ac734305d2450efda78";
const GROUND: &[&str] = &[
    "v-three-before-upgrade",
    "v-four-equipment-activation",
    "before-source-controller",
    "before-paid-ground-move",
    "paid-prefix-before-opportunity",
    "issued-source-raw-before-release",
    "released-source-raw-before-answer",
    "stopped-ground-before-next-turn",
];
const INSPIRATION: &[&str] = &[
    "before-ground-equipment",
    "before-first-host-award",
    "first-award-before-duplicate",
    "pending-owner-transfer",
    "pending-physical-inspiration-reroll",
    "consumed-reroll-before-equipment-finish",
    "settled-reroll-before-next-turn",
];

pub fn sha(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

/// serde_json::Value normally accepts the last duplicate key. Evidence may not.
struct Unique(Value);
impl<'de> Deserialize<'de> for Unique {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Strict;
        impl<'de> Visitor<'de> for Strict {
            type Value = Unique;
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("JSON without duplicate object keys")
            }
            fn visit_bool<E: serde::de::Error>(self, v: bool) -> Result<Unique, E> {
                Ok(Unique(v.into()))
            }
            fn visit_i64<E: serde::de::Error>(self, v: i64) -> Result<Unique, E> {
                Ok(Unique(v.into()))
            }
            fn visit_u64<E: serde::de::Error>(self, v: u64) -> Result<Unique, E> {
                Ok(Unique(v.into()))
            }
            fn visit_f64<E: serde::de::Error>(self, v: f64) -> Result<Unique, E> {
                serde_json::Number::from_f64(v)
                    .map(|n| Unique(Value::Number(n)))
                    .ok_or_else(|| E::custom("nonfinite JSON number"))
            }
            fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Unique, E> {
                Ok(Unique(v.into()))
            }
            fn visit_string<E: serde::de::Error>(self, v: String) -> Result<Unique, E> {
                Ok(Unique(v.into()))
            }
            fn visit_unit<E: serde::de::Error>(self) -> Result<Unique, E> {
                Ok(Unique(Value::Null))
            }
            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Unique, A::Error> {
                let mut values = Vec::new();
                while let Some(Unique(value)) = seq.next_element()? {
                    values.push(value);
                }
                Ok(Unique(Value::Array(values)))
            }
            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Unique, A::Error> {
                let mut values = serde_json::Map::new();
                while let Some((key, Unique(value))) = map.next_entry::<String, Unique>()? {
                    if values.insert(key.clone(), value).is_some() {
                        return Err(serde::de::Error::custom(format!(
                            "duplicate JSON key: {key}"
                        )));
                    }
                }
                Ok(Unique(Value::Object(values)))
            }
        }
        deserializer.deserialize_any(Strict)
    }
}
pub fn strict(bytes: &[u8]) -> Value {
    serde_json::from_slice::<Unique>(bytes)
        .expect("archive JSON must have unique keys")
        .0
}
fn decode<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> T {
    strict(bytes);
    serde_json::from_slice(bytes).expect("typed original archive")
}
fn safe_relative(path: &str) {
    assert!(!path.is_empty(), "empty archive path");
    for part in path.split('/') {
        assert!(
            !part.is_empty() && part != "." && part != "..",
            "archive path traversal"
        );
        assert!(
            part.bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b)),
            "archive path syntax"
        );
    }
}
fn regular(path: &Path) {
    let meta = std::fs::symlink_metadata(path).expect("archive path exists");
    assert!(!meta.file_type().is_symlink(), "archive symlink");
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        assert_eq!(meta.file_attributes() & 0x400, 0, "archive reparse point");
    }
}
fn files(root: &Path, directory: &Path, out: &mut BTreeMap<String, Vec<u8>>) {
    regular(directory);
    for entry in std::fs::read_dir(directory).unwrap() {
        let path = entry.unwrap().path();
        regular(&path);
        if path.is_dir() {
            files(root, &path, out);
        } else {
            assert!(path.is_file(), "archive entry is not a regular file");
            let name = path
                .strip_prefix(root)
                .unwrap()
                .to_str()
                .unwrap()
                .replace('\\', "/");
            safe_relative(&name);
            assert!(
                std::fs::metadata(&path).unwrap().len() < 32 * 1024 * 1024,
                "archive file bound"
            );
            assert!(out.insert(name, std::fs::read(path).unwrap()).is_none());
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Audience {
    pub channel: TableTransportChannel,
    pub raw: TableView,
    pub presented: TablePresentedView,
    pub roll_options: Option<Result<TableRollOptions, String>>,
}
pub struct Cut {
    pub label: String,
    pub before: CampaignExport,
    pub before_views: Vec<Audience>,
    pub after: CampaignExport,
    pub after_views: Vec<Audience>,
    pub request: TableTransportRequest,
    pub response: TableTransportResult,
    pub response_bytes: Vec<u8>,
    pub event: TableEvent,
    pub binding: TableTransportBinding,
}
pub struct Archive {
    pub name: String,
    pub cuts: Vec<Cut>,
    pub retained: Vec<TableTransportBinding>,
}
fn manifest_pin(name: &str) -> &'static str {
    match name {
        "ground-v4" => "b421b38b3a10d2003b323532163a56fdda928b66ad5a7e138c4bf73571c9cdcb",
        "inspiration-recipient-v4" => {
            "7aa3878c3f65d71f0ad6f2b978b13dd58056a1e6aaac25e67dce023e266b57f1"
        }
        "inspiration-decline-v4" => {
            "b52f5116dda336241b8784ef575a4945b1ea28c481b31c1aad91921df6c978d5"
        }
        _ => panic!("unknown original route"),
    }
}
fn verify_files(bundle: &BTreeMap<String, Vec<u8>>, declared: &Value, prefix: &str) {
    for (path, pin) in declared.as_object().unwrap() {
        safe_relative(path);
        let bytes = bundle
            .get(&format!("{prefix}{path}"))
            .expect("missing original file");
        assert_eq!(
            bytes.len() as u64,
            pin["bytes"].as_u64().unwrap(),
            "archive length"
        );
        assert_eq!(
            sha(bytes),
            pin["sha256"].as_str().unwrap(),
            "archive SHA256"
        );
    }
}
pub fn load(name: &str) -> Archive {
    let root =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/physical-facts-v4-969");
    let mut bundle = BTreeMap::new();
    files(&root, &root, &mut bundle);
    let receipt_bytes = &bundle["provenance/receipt.json"];
    assert_eq!(sha(receipt_bytes), RECEIPT, "original provenance receipt");
    let receipt = strict(receipt_bytes);
    assert_eq!(receipt["schema_version"], 1);
    assert_eq!(
        receipt["status"],
        "AUDITED_ORIGINAL_CAPTURE_IMPORTED_UNCHANGED_CONSUMER_UNRUN"
    );
    assert_eq!(receipt["producer"]["head"], PRODUCER);
    assert_eq!(receipt["producer"]["tree"], TREE);
    assert_eq!(receipt["producer_baseline"], BASE);
    assert_eq!(receipt["capture_executable"]["sha256"], EXECUTABLE);
    let original = &receipt["original_payload_and_manifest_files"];
    let provenance = &receipt["copied_provenance_files"];
    assert_eq!(original.as_object().unwrap().len(), 191);
    assert_eq!(provenance.as_object().unwrap().len(), 40);
    verify_files(&bundle, original, "");
    verify_files(&bundle, provenance, "provenance/");
    let expected: BTreeSet<_> = original
        .as_object()
        .unwrap()
        .keys()
        .cloned()
        .chain(
            provenance
                .as_object()
                .unwrap()
                .keys()
                .map(|p| format!("provenance/{p}")),
        )
        .chain(["provenance/receipt.json".to_string()])
        .collect();
    assert_eq!(
        bundle.keys().cloned().collect::<BTreeSet<_>>(),
        expected,
        "whole archive inventory"
    );
    assert_eq!(
        expected
            .iter()
            .map(|s| s.to_ascii_lowercase())
            .collect::<BTreeSet<_>>()
            .len(),
        expected.len(),
        "case aliases"
    );
    let route: BTreeMap<_, _> = bundle
        .into_iter()
        .filter_map(|(path, bytes)| {
            path.strip_prefix(&format!("{name}/"))
                .map(|relative| (relative.to_string(), bytes))
        })
        .collect();
    validate_route(name, &route, manifest_pin(name))
}

fn nested_json(export: &CampaignExport) {
    strict(export.current_state.state_json.as_bytes());
    for row in &export.snapshots {
        strict(row.state_json.as_bytes());
    }
    for row in &export.event_journal {
        strict(row.payload_json.as_bytes());
    }
    for row in &export.command_audit {
        strict(row.payload_json.as_bytes());
    }
    for row in &export.table_transport_bindings {
        strict(row.request_json.as_bytes());
        strict(row.response_json.as_bytes());
    }
}
fn validate_route(name: &str, bundle: &BTreeMap<String, Vec<u8>>, pin: &str) -> Archive {
    let bytes = bundle.get("manifest.json").expect("missing route manifest");
    assert_eq!(sha(bytes), pin, "route manifest pin");
    let m = strict(bytes);
    assert_eq!(m["scenario"], name);
    assert_eq!(m["schema_version"], 1);
    assert_eq!(m["status"], "COMPLETE_GENUINE_APPLICATION_CAPTURES");
    assert_eq!(m["source"]["head"], PRODUCER);
    assert_eq!(m["source"]["tree"], TREE);
    assert_eq!(m["source"]["base"], BASE);
    assert_eq!(m["source"]["compiled_capture_bytes_match"], true);
    assert_eq!(m["source"]["entries"].as_array().unwrap().len(), 727);
    let labels = if name == "ground-v4" {
        GROUND
    } else {
        INSPIRATION
    };
    let records = m["cuts"].as_array().unwrap();
    assert_eq!(records.len(), labels.len(), "cut count");
    let mut names = BTreeSet::from(["manifest.json".to_string()]);
    for a in m["artifacts"].as_array().unwrap() {
        let p = a["path"].as_str().unwrap();
        safe_relative(p);
        assert!(!p.contains('/'), "nested route artifact");
        assert!(names.insert(p.to_string()), "duplicate artifact");
        let bytes = bundle.get(p).expect("missing artifact");
        assert_eq!(
            bytes.len() as u64,
            a["bytes"].as_u64().unwrap(),
            "artifact length"
        );
        assert_eq!(sha(bytes), a["sha256"].as_str().unwrap(), "artifact SHA256");
    }
    assert_eq!(names, bundle.keys().cloned().collect(), "route inventory");
    let mut known = BTreeSet::from(["manifest.json".to_string()]);
    let mut commands = BTreeSet::new();
    let mut cuts = Vec::new();
    for (index, (record, label)) in records.iter().zip(labels).enumerate() {
        assert_eq!(record["label"], *label, "ordered original labels");
        let stem = format!("{index:02}-{label}");
        let mut get = |suffix: &str| {
            let file = format!("{stem}-{suffix}.json");
            known.insert(file.clone());
            bundle.get(&file).expect("known cut artifact").as_slice()
        };
        let before: CampaignExport = decode(get("before-export"));
        let after: CampaignExport = decode(get("expected-after-export"));
        let before_views = decode(get("before-audiences"));
        let after_views = decode(get("expected-after-audiences"));
        let request_bytes = get("next-request");
        let request: TableTransportRequest = decode(request_bytes);
        assert_eq!(
            serde_json::to_vec(&request).unwrap(),
            request_bytes,
            "literal original request"
        );
        let response_bytes = get("accepted-response").to_vec();
        let response: TableTransportResult = decode(&response_bytes);
        assert_eq!(
            serde_json::to_vec(&response).unwrap(),
            response_bytes,
            "literal original response"
        );
        let event_bytes = get("accepted-event");
        let event: TableEvent = decode(event_bytes);
        let binding: TableTransportBinding = decode(get("binding"));
        assert!(
            commands.insert(request.command_id.0),
            "duplicate cut command"
        );
        assert_eq!(request.version, 4);
        assert_eq!(record["version"], 4);
        assert_eq!(record["command"], request.command_id.0.to_string());
        assert_eq!(
            before.current_state.applied_event_sequence as u64,
            record["before_sequence"].as_u64().unwrap()
        );
        assert_eq!(
            after.current_state.applied_event_sequence,
            before.current_state.applied_event_sequence + 1
        );
        assert_eq!(
            after.current_state.applied_event_sequence as u64,
            record["after_sequence"].as_u64().unwrap()
        );
        assert_eq!(binding.meta, event.meta, "binding event meta");
        assert_eq!(binding.meta.id, request.command_id);
        assert_eq!(
            binding.request_json.as_bytes(),
            request_bytes,
            "binding request bytes"
        );
        assert_eq!(
            binding.response_json.as_bytes(),
            response_bytes,
            "binding response bytes"
        );
        assert_eq!(binding.audience, audience(&request.channel));
        assert_eq!(binding.version, 4);
        assert_eq!(
            binding.projection_ordinal,
            after.table_projection_history.last().unwrap().ordinal
        );
        assert_eq!(
            binding.acceptance,
            TransportAcceptance::Command {
                resulting_event_sequence: after.current_state.applied_event_sequence as u64
            }
        );
        assert_eq!(
            before
                .table_transport_bindings
                .iter()
                .filter(|b| b.meta.id == request.command_id)
                .count(),
            0
        );
        assert_eq!(
            after
                .table_transport_bindings
                .iter()
                .filter(|b| b.meta.id == request.command_id)
                .collect::<Vec<_>>(),
            vec![&binding]
        );
        let row = after
            .event_journal
            .iter()
            .find(|r| r.command_id == request.command_id.0.to_string())
            .unwrap();
        assert_eq!(
            row.payload_json.as_bytes(),
            event_bytes,
            "journal original event bytes"
        );
        let audit = after
            .command_audit
            .iter()
            .find(|r| r.id == request.command_id.0.to_string())
            .unwrap();
        let payload = strict(audit.payload_json.as_bytes());
        assert_eq!(payload["request"], serde_json::to_value(&request).unwrap());
        assert_eq!(
            payload["action"],
            serde_json::to_value(&event.action).unwrap()
        );
        assert_eq!(audit.command_schema_version, 5);
        nested_json(&before);
        nested_json(&after);
        super::compare::prefix(&before, &after);
        cuts.push(Cut {
            label: (*label).into(),
            before,
            before_views,
            after,
            after_views,
            request,
            response,
            response_bytes,
            event,
            binding,
        });
    }
    let retained_names: &[&str] = if name == "ground-v4" {
        &["v-three-after-upgrade", "v-three-after-ground-completion"]
    } else {
        &[
            "first-award-after-transfer",
            "first-award-at-end",
            "duplicate-at-end",
            "transfer-at-end",
            "reroll-at-end",
        ]
    };
    let final_export = &cuts.last().unwrap().after;
    let retained = retained_names
        .iter()
        .map(|label| {
            let path = format!("retained-{label}.json");
            known.insert(path.clone());
            let binding: TableTransportBinding = decode(&bundle[&path]);
            assert!(final_export.table_transport_bindings.contains(&binding));
            let request: TableTransportRequest = decode(binding.request_json.as_bytes());
            assert_eq!(
                serde_json::to_string(&request).unwrap(),
                binding.request_json
            );
            binding
        })
        .collect();
    assert_eq!(known, names, "exact route artifact roles");
    for pair in cuts.windows(2) {
        super::compare::prefix(&pair[0].after, &pair[1].before);
    }
    Archive {
        name: name.into(),
        cuts,
        retained,
    }
}

fn failed_at(label: &str, expected: &str, test: impl FnOnce() + std::panic::UnwindSafe) {
    let error = std::panic::catch_unwind(test).expect_err(label);
    let message = error
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| error.downcast_ref::<&str>().copied())
        .unwrap_or("non-string panic");
    assert!(
        message.contains(expected),
        "{label} rejected at another layer: {message}"
    );
}
/// Negative copies may update their envelope hashes solely to reach a named deeper
/// loader check. They never replace the hard-pinned positive receipt or any file.
pub fn packaging_negatives(name: &str) {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/physical-facts-v4-969")
        .join(name);
    let mut original = BTreeMap::new();
    files(&root, &root, &mut original);
    let first = if name == "ground-v4" {
        "00-v-three-before-upgrade"
    } else {
        "00-before-ground-equipment"
    };
    let artifact = format!("{first}-binding.json");
    let validate = |bundle: &BTreeMap<String, Vec<u8>>| {
        validate_route(
            name,
            bundle,
            &sha(bundle.get("manifest.json").expect("manifest")),
        );
    };
    let mut missing = original.clone();
    missing.remove(&artifact);
    failed_at("missing artifact", "missing artifact", || {
        validate(&missing)
    });
    let mut extra = original.clone();
    extra.insert("unexpected.json".into(), b"{}".to_vec());
    failed_at("extra artifact", "route inventory", || validate(&extra));
    let mut corrupt = original.clone();
    corrupt.get_mut(&artifact).unwrap()[0] = b'!';
    failed_at("payload bytes", "artifact SHA256", || validate(&corrupt));
    let mut duplicate = original.clone();
    let bytes = duplicate.get_mut("manifest.json").unwrap();
    bytes.splice(1..1, b"\"schema_version\":1,".iter().copied());
    failed_at("duplicate JSON field", "duplicate JSON key", || {
        validate(&duplicate)
    });
    for (field, value) in [
        ("status", Value::from("NOT_COMPLETE")),
        ("schema_version", Value::from(77)),
        ("scenario", Value::from("foreign-route")),
    ] {
        let mut changed = original.clone();
        let mut manifest = strict(&changed["manifest.json"]);
        assert_ne!(manifest[field], value);
        manifest[field] = value;
        changed.insert(
            "manifest.json".into(),
            serde_json::to_vec(&manifest).unwrap(),
        );
        failed_at(field, "assertion", || validate(&changed));
    }
    for field in ["head", "tree", "base"] {
        let mut changed = original.clone();
        let mut m = strict(&changed["manifest.json"]);
        m["source"][field] = "foreign-source".into();
        changed.insert("manifest.json".into(), serde_json::to_vec(&m).unwrap());
        failed_at(field, "foreign-source", || validate(&changed));
    }
    for (field, value) in [
        ("label", Value::from("foreign-label")),
        ("version", Value::from(5)),
        ("after_sequence", Value::from(9000)),
    ] {
        let mut changed = original.clone();
        let mut m = strict(&changed["manifest.json"]);
        assert_ne!(m["cuts"][0][field], value);
        m["cuts"][0][field] = value;
        changed.insert("manifest.json".into(), serde_json::to_vec(&m).unwrap());
        failed_at(field, "assertion", || validate(&changed));
    }
    let mut count = original.clone();
    let mut m = strict(&count["manifest.json"]);
    m["cuts"].as_array_mut().unwrap().pop();
    count.insert("manifest.json".into(), serde_json::to_vec(&m).unwrap());
    failed_at("missing cut", "cut count", || validate(&count));
    for path in [
        "../escape.json",
        "C:/escape.json",
        "alias:stream",
        "nested/file.json",
    ] {
        let mut changed = original.clone();
        let mut m = strict(&changed["manifest.json"]);
        m["artifacts"][0]["path"] = path.into();
        changed.insert("manifest.json".into(), serde_json::to_vec(&m).unwrap());
        failed_at(
            path,
            if path == "nested/file.json" {
                "nested route artifact"
            } else {
                "archive path"
            },
            || validate(&changed),
        );
    }
    let mut repeated = original.clone();
    let mut m = strict(&repeated["manifest.json"]);
    let a = m["artifacts"][0].clone();
    m["artifacts"].as_array_mut().unwrap().push(a);
    repeated.insert("manifest.json".into(), serde_json::to_vec(&m).unwrap());
    failed_at("duplicate artifact", "duplicate artifact", || {
        validate(&repeated)
    });
    let mut alias = original.clone();
    alias.insert(artifact.to_ascii_uppercase(), original[&artifact].clone());
    failed_at("case alias", "route inventory", || validate(&alias));
    for (field, value) in [("request_json", "{}"), ("response_json", "{}")] {
        let mut changed = original.clone();
        let mut b = strict(&changed[&artifact]);
        assert_ne!(b[field], value);
        b[field] = value.into();
        let bytes = serde_json::to_vec(&b).unwrap();
        let mut m = strict(&changed["manifest.json"]);
        let a = m["artifacts"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|a| a["path"] == artifact)
            .unwrap();
        a["bytes"] = (bytes.len() as u64).into();
        a["sha256"] = sha(&bytes).into();
        changed.insert(artifact.clone(), bytes);
        changed.insert("manifest.json".into(), serde_json::to_vec(&m).unwrap());
        failed_at(
            field,
            if field == "request_json" {
                "binding request bytes"
            } else {
                "binding response bytes"
            },
            || validate(&changed),
        );
    }
    eprintln!("historical969 {name}: strict loader packaging/duplicate/cross-file negatives PASS");
}
