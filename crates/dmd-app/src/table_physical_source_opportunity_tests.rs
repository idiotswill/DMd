//! Synthetic DTO/parser controls only; no Ogre creation, gameplay or journal is invented.
use crate::*;
use dmd_domain::*;
use dmd_rules::tactical::TacticalAction;
use serde_json::json;

#[test]
fn physical_source_opportunity_field_preserves_legacy_absent_bytes() {
    let actor = EntityId::new();
    let target = EntityId::new();
    let legacy = format!(
        "{{\"actor\":\"{}\",\"target\":{{\"actor\":\"{}\",\"label\":\"Located creature\"}},\"weapons\":null,\"unarmed\":true,\"features\":[{{\"feature_id\":\"bite\",\"label\":\"Bite\",\"weapon\":null}}]}}",
        actor.0, target.0
    );
    let mut view: TableOpportunityView = serde_json::from_str(&legacy).unwrap();
    assert!(view.physical_source_weapons.is_empty());
    assert_eq!(serde_json::to_string(&view).unwrap(), legacy);
    let old_features = view.features.clone();
    view.physical_source_weapons
        .push(TablePhysicalSourceWeaponChoice {
            feature_id: "greatclub".into(),
            item: ItemId::new(),
            label: "Greatclub".into(),
            weapon_name: "Greatclub".into(),
            grips: vec![WeaponGrip::TwoHands],
        });
    let encoded = serde_json::to_value(&view).unwrap();
    assert_eq!(
        encoded["physical_source_weapons"].as_array().unwrap().len(),
        1
    );
    assert_eq!(
        serde_json::from_value::<TableOpportunityView>(encoded).unwrap(),
        view
    );
    assert_eq!(view.features, old_features);
    view.physical_source_weapons.clear();
    assert_eq!(serde_json::to_string(&view).unwrap(), legacy);
}

#[test]
fn physical_source_choice_uses_the_desktop_request_parser_without_extra_authority() {
    let weapon = ItemId::new();
    let actor = EntityId::new();
    // These wire envelopes prove only parsing. Channel ownership, session/revision
    // validation and Ogre admission still run in the actual runtime on acceptance.
    for (version, channel) in [
        (TABLE_TRANSPORT_VERSION, json!("Host")),
        (
            TABLE_SOURCE_TRANSPORT_VERSION,
            json!({"SourceCreature": {"player_id": PlayerId::new(), "actor": actor}}),
        ),
    ] {
        for (feature, grip) in [
            ("greatclub", WeaponGrip::TwoHands),
            ("javelin-melee", WeaponGrip::OneHand(Hand::Left)),
        ] {
            let wire = json!({
                "version": version,
                "command_id": CommandId::new(),
                "campaign_id": CampaignId::new(),
                "session_id": PlaySessionId::new(),
                "channel": channel,
                "revision": uuid::Uuid::new_v4(),
                "input": {"Action": {"Tactical": {"action": {"OpportunityAttack": {"choice": {
                    "CreatureWeapon": {"feature_id": feature, "weapon": weapon, "grip": grip}
                }}}}}}
            });
            // This is the exact typed input to dmd-desktop::host::desktop_submit_table.
            let parsed: TableTransportRequest = serde_json::from_value(wire.clone()).unwrap();
            let TableTransportInput::Action(action) = &parsed.input else {
                panic!("action absent")
            };
            assert_eq!(
                action.as_ref(),
                &TableAction::Tactical {
                    action: TacticalAction::OpportunityAttack {
                        choice: TacticalMeleeChoice::CreatureWeapon {
                            feature_id: feature.into(),
                            weapon,
                            grip
                        },
                    },
                }
            );
            assert_eq!(serde_json::to_value(parsed).unwrap(), wire);
            for field in [
                "target",
                "actor",
                "source",
                "damage",
                "cost",
                "ammunition",
                "equipment_change",
                "item",
            ] {
                let mut forged = wire.clone();
                forged["input"]["Action"]["Tactical"]["action"]["OpportunityAttack"]["choice"]["CreatureWeapon"]
                    [field] = json!("caller supplied");
                assert!(
                    serde_json::from_value::<TableTransportRequest>(forged).is_err(),
                    "{field}"
                );
            }
            let mut missing_item = wire;
            missing_item["input"]["Action"]["Tactical"]["action"]["OpportunityAttack"]["choice"]
                ["CreatureWeapon"]["weapon"] = json!(null);
            assert!(serde_json::from_value::<TableTransportRequest>(missing_item).is_err());
        }
    }
}
