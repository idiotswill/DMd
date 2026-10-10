//! Additional live reads over the actual original/current views; never a new oracle.
use super::*;

pub async fn assert_reads(f: &driver::Fixture, views: &[archive::Audience]) {
    let state = f.state().await;
    for captured in views {
        if let Some(roll) = &captured.presented.roll {
            let request = TableRollDetailsRequest {
                version: 1,
                campaign_id: f.campaign,
                channel: captured.channel.clone(),
                revision: captured.presented.revision,
                roll_id: roll.id,
            };
            let actual = Box::pin(f.runtime.table_roll_details(request))
                .await
                .map_err(|e| e.to_string());
            match captured
                .roll_options
                .as_ref()
                .expect("offered roll has its original options result")
            {
                Ok(options) => {
                    let details = actual.unwrap();
                    assert_eq!(details.version, 1);
                    assert_eq!(
                        &details.options, options,
                        "legacy channel-specific options stay literal"
                    );
                    let pending = state.rules.as_ref().unwrap().pending.as_ref().unwrap();
                    let label = match pending.purpose {
                        PendingPurpose::TacticalResolution { key, .. }
                            if key.role == TacticalRollRole::GrappleSave =>
                        {
                            assert_eq!(pending.request.reason, "Grapple saving throw");
                            assert_eq!(roll.reason, "Unsupported roll");
                            "Grapple saving throw"
                        }
                        PendingPurpose::TacticalResolution { key, .. }
                            if key.role == TacticalRollRole::GrappleEscape =>
                        {
                            assert!(
                                [
                                    "Strength (Athletics) Escape",
                                    "Dexterity (Acrobatics) Escape"
                                ]
                                .contains(&pending.request.reason.as_str())
                            );
                            assert_eq!(roll.reason, "Unsupported roll");
                            pending.request.reason.as_str()
                        }
                        _ => roll.reason.as_str(),
                    };
                    assert_eq!(details.display_reason, label);
                    assert_eq!(
                        options.heroic_inspiration,
                        dmd_rules::table::grapple_transport_enabled(&state).then(|| state
                            .rules
                            .as_ref()
                            .unwrap()
                            .entities[&pending.request.roller.unwrap()]
                            .heroic_inspiration)
                    );
                    if state.physical_facts.is_some() {
                        assert_eq!(captured.presented.physical.as_ref().unwrap().version, 5);
                        assert_eq!(captured.presented.grapple.as_ref().unwrap().version, 4);
                    }
                }
                Err(expected) => assert_eq!(
                    actual.unwrap_err(),
                    *expected,
                    "wrong actor/source read keeps its exact refusal"
                ),
            }
        } else {
            assert!(captured.roll_options.is_none());
        }
        let raw = f
            .runtime
            .table_view(f.campaign, viewer(&captured.channel))
            .await
            .unwrap();
        let presented = f.view(&captured.channel).await;
        assert_eq!(
            serde_json::to_vec(&raw).unwrap(),
            serde_json::to_vec(&captured.raw).unwrap(),
            "details never relabel the original raw DTO"
        );
        assert_eq!(
            serde_json::to_vec(&presented).unwrap(),
            serde_json::to_vec(&captured.presented).unwrap(),
            "details never change original revisions, handles, or presented DTOs"
        );
    }
    // The caller's existing complete export and all-table cell guards wrap every read.
}
