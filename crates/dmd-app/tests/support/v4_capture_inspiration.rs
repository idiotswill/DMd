//! Genuine current Physical PC creation, awards, optional transfer and raw reroll.
use super::*;
use crate::v4_capture_archive::Capture;

#[tokio::test]
async fn capture_v4_inspiration_recipient_and_physical_reroll() {
    Box::pin(run(true)).await;
}

#[tokio::test]
async fn capture_v4_inspiration_decline_and_physical_reroll() {
    Box::pin(run(false)).await;
}

async fn run(give: bool) {
    let mut capture = Capture::new(if give {
        "inspiration-recipient-v4"
    } else {
        "inspiration-decline-v4"
    });
    let mut f = Box::pin(Fixture::with_physical_weapon("glaive")).await;
    enable(&mut f).await;
    let equipment = request(
        &f,
        TableTransportChannel::Host,
        action(TacticalAction::ActivateAttackEquipment),
    )
    .await;
    Box::pin(capture.accept(&mut f, "before-ground-equipment", equipment)).await;
    let initial = f.state().await;
    let receipt = initial
        .rules
        .as_ref()
        .unwrap()
        .tactical_inventory
        .as_ref()
        .unwrap()
        .receipt(f.characters[0])
        .unwrap();
    assert_eq!(
        receipt.source.profile_id,
        "human-fighter-soldier-level-1-physical-v1"
    );
    assert!(receipt.creation_profile.creation_source.is_some());
    assert!(!has_inspiration(&initial, f.actors[0]));
    assert!(!has_inspiration(&initial, f.actors[1]));
    // PC 1 is genuinely uninvolved until an actual gift is accepted. The
    // controller alias in the Ground route is deliberately not used for this.
    let private = f
        .runtime
        .table_view(f.campaign, TableViewer::Player(f.players[1]))
        .await
        .unwrap();
    let unrelated = f.view(f.pc(1)).await;
    let unrelated_bytes = serde_json::to_vec(&unrelated).unwrap();
    let first = request(
        &f,
        TableTransportChannel::Host,
        award(f.characters[0], REASON),
    )
    .await;
    Box::pin(capture.accept(&mut f, "before-first-host-award", first.clone())).await;
    private_unchanged(&f, &private, &unrelated).await;
    assert_eq!(
        serde_json::to_vec(&f.view(f.pc(1)).await).unwrap(),
        unrelated_bytes
    );
    let awarded = f.state().await;
    assert!(has_inspiration(&awarded, f.actors[0]));
    assert_eq!(
        awarded
            .rules
            .as_ref()
            .unwrap()
            .rulings
            .last()
            .unwrap()
            .command
            .id,
        first.command_id
    );
    let duplicate = request(&f, TableTransportChannel::Host, excess(f.characters[0])).await;
    Box::pin(capture.accept(&mut f, "first-award-before-duplicate", duplicate.clone())).await;
    private_unchanged(&f, &private, &unrelated).await;
    assert_eq!(
        serde_json::to_vec(&f.view(f.pc(1)).await).unwrap(),
        unrelated_bytes
    );
    let choosing = f.state().await;
    assert!(has_inspiration(&choosing, f.actors[0]) && pending(&choosing, f.actors[0]));
    assert!(!has_inspiration(&choosing, f.actors[1]));
    assert_eq!(
        choosing
            .table
            .as_ref()
            .unwrap()
            .inspiration_transfer
            .as_ref()
            .unwrap()
            .origin
            .id,
        duplicate.command_id
    );
    let choice = decision(
        &f,
        0,
        if give {
            "Give to Character 1"
        } else {
            "Decline the extra Inspiration"
        },
    )
    .await;
    Box::pin(capture.accept(&mut f, "pending-owner-transfer", choice.clone())).await;
    if !give {
        private_unchanged(&f, &private, &unrelated).await;
        assert_eq!(
            serde_json::to_vec(&f.view(f.pc(1)).await).unwrap(),
            unrelated_bytes
        );
    }
    let transferred = f.state().await;
    assert!(
        transferred
            .table
            .as_ref()
            .unwrap()
            .inspiration_transfer
            .is_none()
    );
    assert!(has_inspiration(&transferred, f.actors[0]) && !pending(&transferred, f.actors[0]));
    assert_eq!(has_inspiration(&transferred, f.actors[1]), give);
    assert!(!pending(&transferred, f.actors[1]));
    capture
        .retained_retry(&f, "first-award-after-transfer", &first)
        .await;

    // Existing helper produces a real Goblin attempt and an owned PC save.
    target_pc(&mut f, "Strength").await;
    let saving = f.state().await;
    let raw = saving.rules.as_ref().unwrap().pending.as_ref().unwrap();
    assert_eq!(raw.request.roller, Some(f.actors[0]));
    assert_eq!(raw.request.modifier, 5);
    assert!(
        matches!(raw.purpose, PendingPurpose::TacticalResolution { key, .. }
        if key.role == TacticalRollRole::GrappleSave)
    );
    assert_eq!(options(&f, f.pc(0)).await.heroic_inspiration, Some(true));
    let reroll = inspired(&f, 1, 20).await;
    Box::pin(capture.accept(
        &mut f,
        "pending-physical-inspiration-reroll",
        reroll.clone(),
    ))
    .await;
    let consumed = f.state().await;
    let rules = consumed.rules.as_ref().unwrap();
    let recorded = rules.rolls.last().unwrap();
    assert_eq!(recorded.request, raw.request);
    assert_eq!(recorded.issued_by, raw.issued_by);
    assert_eq!(recorded.purpose, raw.purpose);
    assert_eq!(recorded.accepted_by.id, reroll.command_id);
    assert_eq!(
        recorded.accepted_by.issuer,
        CommandIssuer::Player(f.players[0])
    );
    assert_eq!(
        recorded.original_result,
        Some(RollResult {
            request_id: raw.request.id,
            source: RollSource::Physical,
            dice: vec![DieResult {
                sides: 20,
                value: 1
            }],
        })
    );
    assert_eq!(recorded.result.request_id, raw.request.id);
    assert_eq!(
        recorded.result.dice,
        vec![DieResult {
            sides: 20,
            value: 20
        }]
    );
    assert_eq!(recorded.result.source, RollSource::Physical);
    assert_eq!(recorded.resolved.total, 25);
    assert!(!has_inspiration(&consumed, f.actors[0]));
    assert_eq!(
        rules.rolls.len(),
        saving.rules.as_ref().unwrap().rolls.len() + 1
    );
    let mut finish = f
        .choose(
            TableTransportChannel::Host,
            "Finish without changing equipment",
        )
        .await;
    finish.version = 4;
    Box::pin(capture.accept(&mut f, "consumed-reroll-before-equipment-finish", finish)).await;
    assert!(
        f.state()
            .await
            .encounter
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .resolution
            .is_none()
    );
    let end = request(
        &f,
        TableTransportChannel::Host,
        action(TacticalAction::EndTurn),
    )
    .await;
    Box::pin(capture.accept(&mut f, "settled-reroll-before-next-turn", end)).await;
    for (label, original) in [
        ("first-award-at-end", first),
        ("duplicate-at-end", duplicate),
        ("transfer-at-end", choice),
        ("reroll-at-end", reroll),
    ] {
        capture.retained_retry(&f, label, &original).await;
    }
    f.close().await;
    capture.finish(&[
        "before-ground-equipment",
        "before-first-host-award",
        "first-award-before-duplicate",
        "pending-owner-transfer",
        "pending-physical-inspiration-reroll",
        "consumed-reroll-before-equipment-finish",
        "settled-reroll-before-next-turn",
    ]);
}
