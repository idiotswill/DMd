use super::*;
use crate::WorldInstant;

fn captured_wire() -> String {
    let export: serde_json::Value = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../dmd-app/tests/fixtures/shield-hit-v1-selected.json"
    )))
    .unwrap();
    let state = export["current_state"]["state_json"].as_str().unwrap();
    let start = state.find("\"resolution\":{").unwrap() + "\"resolution\":".len();
    let mut depth = 0;
    let mut quoted = false;
    let mut escaped = false;
    for (offset, ch) in state[start..].char_indices() {
        if quoted {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                quoted = false;
            }
        } else {
            match ch {
                '"' => quoted = true,
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        return state[start..=start + offset].into();
                    }
                }
                _ => {}
            }
        }
    }
    panic!("captured resolution is unterminated")
}
#[test]
fn original_captured_turn_wire_remains_byte_exact() {
    let original = captured_wire();
    let cursor: TacticalResolution = serde_json::from_str(&original).unwrap();
    assert!(cursor.turn_context().is_ok());
    assert_eq!(serde_json::to_string(&cursor).unwrap(), original);
    assert!(!original.contains("released_interval"));
}
#[test]
fn turn_wire_rejects_null_partial_mixed_duplicate_and_unknown_context() {
    let original = captured_wire();
    for field in ["turn_actor", "turn_number", "boundary"] {
        let mut value: serde_json::Value = serde_json::from_str(&original).unwrap();
        value.as_object_mut().unwrap().remove(field);
        assert!(serde_json::from_value::<TacticalResolution>(value).is_err());
        let mut value: serde_json::Value = serde_json::from_str(&original).unwrap();
        value[field] = serde_json::Value::Null;
        assert!(serde_json::from_value::<TacticalResolution>(value).is_err());
        let duplicate = original.replacen('{', &format!("{{\"{field}\":null,"), 1);
        assert!(serde_json::from_str::<TacticalResolution>(&duplicate).is_err());
    }
    for extra in [
        "\"released_interval\":null,",
        "\"future_context\":{},",
        "\"released_interval\":null,\"released_interval\":null,",
    ] {
        assert!(
            serde_json::from_str::<TacticalResolution>(&original.replacen(
                '{',
                &format!("{{{extra}"),
                1
            ))
            .is_err()
        );
    }
}
#[test]
fn released_wire_omits_turn_fields_and_rejects_mixed_turn_authority() {
    let mut cursor: TacticalResolution = serde_json::from_str(&captured_wire()).unwrap();
    cursor.context =
        TacticalResolutionContext::ReleasedInterval(Box::new(crate::ReleasedElapsedContext {
            release: cursor.origin.id,
            started_at: WorldInstant(0),
            target_at: WorldInstant(1),
            progress_at: WorldInstant(0),
            ordering: crate::ReleasedTimeOrdering::HostSelect,
            ruling: "Wait.".into(),
            predecessor: None,
            batches: vec![],
        }));
    let wire = serde_json::to_string(&cursor).unwrap();
    let decoded: TacticalResolution = serde_json::from_str(&wire).unwrap();
    assert_eq!(decoded, cursor);
    let value: serde_json::Value = serde_json::from_str(&wire).unwrap();
    for field in ["turn_actor", "turn_number", "boundary"] {
        assert!(value.get(field).is_none());
    }
    for extra in [
        "\"turn_number\":1,",
        "\"turn_actor\":null,",
        "\"released_interval\":null,",
    ] {
        assert!(
            serde_json::from_str::<TacticalResolution>(&wire.replacen(
                '{',
                &format!("{{{extra}"),
                1
            ))
            .is_err()
        );
    }
}
#[test]
fn version_seven_preserves_five_and_rejects_reserved_six_and_future_versions() {
    for version in 1..=5 {
        assert_eq!(
            crate::TacticalExecutionVersion::from_flow_version(version)
                .unwrap()
                .flow_version(),
            version
        );
    }
    for version in [0, 6, 8, u32::MAX] {
        assert!(crate::TacticalExecutionVersion::from_flow_version(version).is_none());
    }
    let released = crate::TacticalExecutionVersion::from_flow_version(7).unwrap();
    assert!(
        released.retains_work_ancestry()
            && released.supports_hit_shield()
            && released.supports_missile_shield()
            && released.supports_release()
            && released.supports_released_time()
    );
    assert!(!crate::TacticalExecutionVersion::EncounterReleaseV1.supports_released_time());
}
