//! Frozen-v4 cuts reuse the real Ground fixture and unchanged cold driver.
use super::*;
use crate::v4_capture_archive::Capture;

async fn choose_v4(f: &mut Fixture, channel: TableTransportChannel, label: &str) {
    let mut command = f.choose(channel, label).await;
    command.version = 4;
    Box::pin(f.cold(command)).await;
}

#[tokio::test]
async fn capture_v4_ground_upgrade_paid_prefix_and_issued_source_raw() {
    let mut capture = Capture::new("ground-v4");
    let mut f = Box::pin(opportunity_fixture()).await;
    let old = Box::pin(f.activate()).await;
    assert_eq!(old.version, 3);
    let upgrade = request(
        &f,
        TableTransportChannel::Host,
        TableTransportInput::Action(Box::new(TableAction::EnableGrappleTransport)),
    )
    .await;
    Box::pin(capture.accept(&mut f, "v-three-before-upgrade", upgrade)).await;
    assert_eq!(f.view(f.pc(0)).await.grapple.unwrap().version, 4);
    capture
        .retained_retry(&f, "v-three-after-upgrade", &old)
        .await;
    let equipment = request(
        &f,
        TableTransportChannel::Host,
        action(TacticalAction::ActivateAttackEquipment),
    )
    .await;
    Box::pin(capture.accept(&mut f, "v-four-equipment-activation", equipment)).await;
    assert!(flow(&f.state().await).attack_equipment_access.is_some());

    let pc = f.pc(0);
    Box::pin(choose_v4(
        &mut f,
        pc.clone(),
        "Grapple Small armored figure with left hand",
    ))
    .await;
    Box::pin(choose_v4(
        &mut f,
        TableTransportChannel::Host,
        "Resist Grapple with Strength",
    ))
    .await;
    let answer = raw(&f, TableTransportChannel::Host, 1).await;
    Box::pin(f.cold(answer)).await;
    Box::pin(choose_v4(
        &mut f,
        pc.clone(),
        "Finish without changing equipment",
    ))
    .await;
    let reactor = f.opponent.unwrap();
    let source = TableTransportChannel::SourceCreature {
        player_id: f.players[1],
        actor: reactor,
    };
    let control = request(
        &f,
        TableTransportChannel::Host,
        TableTransportInput::Action(Box::new(TableAction::SetSourceCreatureController {
            actor: reactor,
            controller: CreatureController::Player(f.players[1]),
        })),
    )
    .await;
    Box::pin(capture.accept(&mut f, "before-source-controller", control)).await;
    capture.include_source(source.clone());

    // This exact two-step geometry is the existing nonempty-prefix control.
    let movement = drag(&f, vec![step(0, 10), step(0, 0)]).await;
    Box::pin(capture.accept(&mut f, "before-paid-ground-move", movement)).await;
    let selected = f.state().await;
    let prefix = resolution(&selected)
        .grapple
        .as_ref()
        .unwrap()
        .transport
        .as_ref()
        .unwrap()
        .steps
        .clone();
    assert_eq!(prefix.len(), 1);
    assert_eq!(prefix[0].holder.to, SpatialPoint { x: 0, y: 10, z: 0 });
    assert_eq!(prefix[0].target.to, SpatialPoint { x: 10, y: 10, z: 0 });
    assert_eq!(
        resolution(&selected).movement.as_ref().unwrap().next_step,
        1
    );
    assert_eq!(flow(&selected).budget.movement_spent, 20);
    assert_eq!(
        resolution(&selected)
            .movement
            .as_ref()
            .unwrap()
            .opportunity
            .as_ref()
            .unwrap()
            .reactor,
        reactor
    );
    let accept = request(
        &f,
        source.clone(),
        action(TacticalAction::OpportunityAttack {
            choice: TacticalMeleeChoice::UnarmedDamage {
                ability: Ability::Strength,
            },
        }),
    )
    .await;
    Box::pin(capture.accept(&mut f, "paid-prefix-before-opportunity", accept)).await;
    let issued = f.state().await;
    let pending = issued.rules.as_ref().unwrap().pending.clone().unwrap();
    assert_eq!(pending.request.roller, Some(reactor));
    assert!(
        issued
            .rules
            .as_ref()
            .unwrap()
            .timing
            .as_ref()
            .unwrap()
            .reactions_spent
            .contains(&reactor)
    );
    let release = release(&f).await;
    Box::pin(capture.accept(&mut f, "issued-source-raw-before-release", release)).await;
    let released = f.state().await;
    assert_eq!(
        released.rules.as_ref().unwrap().pending.as_ref(),
        Some(&pending)
    );
    assert_eq!(resolution(&released).attack, resolution(&issued).attack);
    assert_eq!(
        resolution(&released)
            .grapple
            .as_ref()
            .unwrap()
            .transport
            .as_ref()
            .unwrap()
            .steps,
        prefix
    );
    let answer = raw(&f, source, 1).await;
    Box::pin(capture.accept(&mut f, "released-source-raw-before-answer", answer)).await;
    let done = f.state().await;
    let result = flow(&done).last_movement.as_ref().unwrap();
    assert_eq!(result.reason, TacticalMovementEnd::Stopped);
    assert_eq!((result.completed_steps, result.spent_after), (1, 20));
    assert_eq!(result.endpoint, prefix[0].holder.to);
    assert_eq!(
        result.transport.as_ref().unwrap().target_endpoint,
        prefix[0].target.to
    );
    assert_eq!(
        done.rules.as_ref().unwrap().rolls.last().unwrap().request,
        pending.request
    );
    assert_eq!(
        done.rules.as_ref().unwrap().rolls.len(),
        issued.rules.as_ref().unwrap().rolls.len() + 1
    );
    assert_eq!(
        done.rules
            .as_ref()
            .unwrap()
            .timing
            .as_ref()
            .unwrap()
            .reactions_spent,
        issued
            .rules
            .as_ref()
            .unwrap()
            .timing
            .as_ref()
            .unwrap()
            .reactions_spent
    );
    assert!(flow(&done).resolution.is_none());
    let end = request(&f, pc, action(TacticalAction::EndTurn)).await;
    Box::pin(capture.accept(&mut f, "stopped-ground-before-next-turn", end)).await;
    capture
        .retained_retry(&f, "v-three-after-ground-completion", &old)
        .await;
    f.close().await;
    capture.finish(&[
        "v-three-before-upgrade",
        "v-four-equipment-activation",
        "before-source-controller",
        "before-paid-ground-move",
        "paid-prefix-before-opportunity",
        "issued-source-raw-before-release",
        "released-source-raw-before-answer",
        "stopped-ground-before-next-turn",
    ]);
}
