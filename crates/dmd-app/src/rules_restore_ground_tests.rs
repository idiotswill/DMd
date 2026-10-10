//! Pure export preflight refusals. These forged images are never accepted play.
use super::*;
use dmd_domain::{AttackEquipmentChange, AttackEquipmentOperation, EquipmentChangeTiming, Hand};

#[test]
fn after_equipment_intent_parent_or_orphan_work_is_refused_at_every_export_cut() {
    use dmd_domain::{
        AfterAttackEquipmentIntent, AttackAfterEquipmentReceipt, AttackEquipmentCause,
        AttackEquipmentCompletionParent, AttackEquipmentPause, AttackEquipmentSource,
        TacticalAttackAfterEquipment, TacticalWorkItem, TacticalWorkKey, TacticalWorkKind,
    };
    let original: CampaignExport = serde_json::from_str(include_str!(
        "../tests/fixtures/shield-hit-v1-selected.json"
    ))
    .unwrap();
    let pack =
        RulesPack::from_json(include_str!("../../../content/srd-5.2.1/kernel.json")).unwrap();
    for shape in 0..5 {
        let mut poisoned: CampaignState =
            serde_json::from_str(&original.current_state.state_json).unwrap();
        let r = poisoned
            .encounter
            .as_mut()
            .unwrap()
            .flow
            .as_mut()
            .unwrap()
            .resolution
            .as_mut()
            .unwrap();
        let attack = r.attack.as_ref().unwrap().clone();
        let work = TacticalWorkItem {
            occurrence: r.next_occurrence,
            kind: TacticalWorkKind::AttackAfterEquipment,
        };
        let work_key = TacticalWorkKey {
            resolution: r.origin.id,
            occurrence: work.occurrence,
        };
        let cause = AttackEquipmentCause {
            origin: attack.origin.clone(),
            actor: attack.actor,
            turn_number: r.turn_number,
            window: attack.weapon().unwrap().window,
            choice: attack.weapon().unwrap().choice.clone(),
            source: AttackEquipmentSource::Ordinary,
            outcome: attack.outcome.unwrap(),
            completed_by: attack.origin.clone(),
            completed_work: TacticalWorkKey {
                resolution: r.origin.id,
                occurrence: 0,
            },
        };
        match shape {
            0 => {
                r.attack
                    .as_mut()
                    .unwrap()
                    .weapon_mut()
                    .unwrap()
                    .choice
                    .after_equipment = Some(AfterAttackEquipmentIntent::Choose)
            }
            1 => {
                let work = TacticalWorkKey {
                    resolution: r.origin.id,
                    occurrence: 0,
                };
                let parent = AttackEquipmentCompletionParent {
                    work,
                    pause: AttackEquipmentPause::Knockout,
                    paused_by: attack.origin.clone(),
                    accepted_raw: attack.damage_roll,
                    suspended_outcome: attack.outcome.unwrap(),
                };
                r.attack
                    .as_mut()
                    .unwrap()
                    .weapon_mut()
                    .unwrap()
                    .after_equipment_parent = Some(parent);
            }
            2 => r.frames.push(vec![work]),
            3 => {
                r.attack_after_equipment = Some(Box::new(TacticalAttackAfterEquipment {
                    cause,
                    work,
                    selected_by: Some(attack.origin.clone()),
                }))
            }
            _ => {
                let receipt = AttackAfterEquipmentReceipt {
                    cause,
                    work: work_key,
                    selected_by: attack.origin.clone(),
                    chosen_by: attack.origin.clone(),
                    applied: None,
                };
                let flow = poisoned.encounter.as_mut().unwrap().flow.as_mut().unwrap();
                flow.resolution = None;
                flow.budget.weapon_history[0].after_equipment = Some(Box::new(receipt));
            }
        }
        let encoded = serde_json::to_string(&poisoned).unwrap();
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
                "shape {shape}, cut {cut}: {error}"
            );
            assert_eq!(serde_json::to_string(&export).unwrap(), before);
        }
    }
}

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
