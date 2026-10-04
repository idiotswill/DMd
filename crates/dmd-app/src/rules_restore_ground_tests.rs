//! Pure export preflight refusals. These forged images are never accepted play.
use super::*;
use dmd_domain::{AttackEquipmentChange, AttackEquipmentOperation, EquipmentChangeTiming, Hand};

#[test]
fn pickup_authority_in_current_later_snapshot_or_recovery_anchor_is_refused_without_writes() {
    let original: CampaignExport = serde_json::from_str(include_str!(
        "../tests/fixtures/shield-hit-v1-selected.json"
    ))
    .unwrap();
    let mut poisoned: CampaignState =
        serde_json::from_str(&original.current_state.state_json).unwrap();
    let weapon = poisoned
        .encounter
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .resolution
        .as_mut()
        .unwrap()
        .attack
        .as_mut()
        .unwrap()
        .weapon_mut()
        .unwrap();
    weapon.choice.equipment_change = Some(AttackEquipmentChange {
        timing: EquipmentChangeTiming::BeforeAttack,
        operation: AttackEquipmentOperation::Pickup {
            item: weapon.choice.weapon,
            hand: Hand::Right,
        },
    });
    let encoded = serde_json::to_string(&poisoned).unwrap();
    let pack =
        RulesPack::from_json(include_str!("../../../content/srd-5.2.1/kernel.json")).unwrap();
    for cut in 0..3 {
        let mut export = original.clone();
        if cut == 0 {
            export.current_state.state_json = encoded.clone();
        } else {
            let mut snapshot = export.snapshots[0].clone();
            snapshot.event_sequence = i64::try_from(poisoned.applied_event_sequence).unwrap();
            snapshot.state_schema_version = i64::from(poisoned.schema_version);
            snapshot.state_json = encoded.clone();
            if cut == 2 {
                export.snapshots.clear();
            }
            export.snapshots.push(snapshot);
        }
        let before = serde_json::to_string(&export).unwrap();
        let error = validate_rules_export(&export, &pack).unwrap_err();
        assert!(
            error.contains("ground pickup execution is not enabled"),
            "cut {cut}: {error}"
        );
        assert_eq!(serde_json::to_string(&export).unwrap(), before);
    }
}
