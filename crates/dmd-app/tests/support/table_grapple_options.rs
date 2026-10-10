//! Live advice uses genuine accepted history and must preserve every persisted row.
use super::*;

async fn request(f: &Fixture, channel: TableTransportChannel) -> TableGrappleOptionsRequest {
    let view = f.view(channel.clone()).await;
    TableGrappleOptionsRequest {
        version: 1,
        campaign_id: f.campaign,
        channel,
        revision: view.revision,
    }
}
async fn unchanged(f: &Fixture, before: &CampaignExport, rows: &[(String, Vec<Vec<String>>)]) {
    let mut after = export_campaign(&f.pool, f.campaign).await.unwrap();
    after.exported_at_utc = before.exported_at_utc.clone();
    assert_eq!(&after, before);
    assert_eq!(all_rows(&f.pool).await, rows);
}
async fn read(f: &Fixture, request: TableGrappleOptionsRequest) -> TableGrappleOptions {
    let original = f.view(request.channel.clone()).await;
    let before = export_campaign(&f.pool, f.campaign).await.unwrap();
    let rows = all_rows(&f.pool).await;
    let answer = Box::pin(f.runtime.table_grapple_options(request.clone()))
        .await
        .unwrap();
    assert_eq!(answer.version, 1);
    assert_eq!(answer.campaign_id, request.campaign_id);
    assert_eq!(answer.channel, request.channel);
    assert_eq!(answer.revision, request.revision);
    let offered = original
        .grapple
        .as_ref()
        .unwrap()
        .choices
        .iter()
        .map(|choice| choice.key)
        .collect::<Vec<_>>();
    assert!(answer.handles.iter().all(|handle| offered.contains(handle)));
    assert_eq!(
        answer
            .handles
            .iter()
            .collect::<std::collections::HashSet<_>>()
            .len(),
        answer.handles.len()
    );
    assert_eq!(f.view(request.channel).await, original);
    Box::pin(unchanged(f, &before, &rows)).await;
    answer
}
async fn refused(f: &Fixture, request: TableGrappleOptionsRequest) {
    let before = export_campaign(&f.pool, f.campaign).await.unwrap();
    let rows = all_rows(&f.pool).await;
    assert!(
        Box::pin(f.runtime.table_grapple_options(request))
            .await
            .is_err()
    );
    Box::pin(unchanged(f, &before, &rows)).await;
}
pub(super) async fn assert_available(f: &Fixture, accepted: &TableTransportRequest) {
    let TableTransportInput::GrappleChoice { handle } = &accepted.input else {
        panic!("expected original opaque choice")
    };
    let answer = Box::pin(read(
        f,
        TableGrappleOptionsRequest {
            version: 1,
            campaign_id: accepted.campaign_id,
            channel: accepted.channel.clone(),
            revision: accepted.revision,
        },
    ))
    .await;
    assert!(answer.handles.contains(handle));
}

#[tokio::test]
async fn current_grapple_options_keep_a_genuinely_paid_attack_slot_and_reject_stale_identity_without_writes()
 {
    let mut f = Box::pin(Fixture::new()).await;
    Box::pin(f.activate()).await;
    let pc = f.pc(0);
    let first = f
        .choose(pc.clone(), "Grapple Small armored figure with right hand")
        .await;
    Box::pin(assert_available(&f, &first)).await;
    let valid = request(&f, pc.clone()).await;
    let other = request(&f, f.pc(1)).await;
    let host = request(&f, TableTransportChannel::Host).await;
    assert!(Box::pin(read(&f, other.clone())).await.handles.is_empty());
    assert!(Box::pin(read(&f, host)).await.handles.is_empty());
    let mut wrong_version = valid.clone();
    wrong_version.version = 2;
    let mut wrong_revision = valid.clone();
    wrong_revision.revision = other.revision;
    let mut wrong_pc = valid.clone();
    wrong_pc.channel = TableTransportChannel::Player {
        player_id: f.players[0],
        character_id: f.characters[1],
    };
    let mut nonexistent_pc = valid.clone();
    nonexistent_pc.channel = TableTransportChannel::Player {
        player_id: f.players[0],
        character_id: CharacterId::new(),
    };
    let mut wrong_source = valid.clone();
    wrong_source.channel = TableTransportChannel::SourceCreature {
        player_id: f.players[0],
        actor: f.goblin,
    };
    let mut wrong_campaign = valid.clone();
    wrong_campaign.campaign_id = CampaignId::new();
    for rejected in [
        wrong_version,
        wrong_revision,
        wrong_pc,
        nonexistent_pc,
        wrong_source,
        wrong_campaign,
    ] {
        Box::pin(refused(&f, rejected)).await;
    }
    // Fighter1's real accepted action grants one unspent ordinary attack.
    Box::pin(f.cold_action(pc.clone(), TacticalAction::StartAttackAction)).await;
    Box::pin(refused(&f, valid)).await;
    let paid = f.state().await;
    assert!(
        paid.rules
            .as_ref()
            .unwrap()
            .timing
            .as_ref()
            .unwrap()
            .action_spent
    );
    assert_eq!(
        paid.encounter
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .budget
            .attacks_remaining,
        1
    );
    let paid_choice = f
        .choose(pc, "Grapple Small armored figure with right hand")
        .await;
    Box::pin(assert_available(&f, &paid_choice)).await;
    Box::pin(f.cold(paid_choice)).await;
    let used = f.state().await;
    assert!(
        used.rules
            .as_ref()
            .unwrap()
            .timing
            .as_ref()
            .unwrap()
            .action_spent
    );
    assert_eq!(
        used.encounter
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .budget
            .attacks_remaining,
        0
    );
    assert_eq!(
        used.rules.as_ref().unwrap().rolls,
        paid.rules.as_ref().unwrap().rolls
    );
    f.close().await;
}

#[tokio::test]
async fn current_grapple_options_preserve_save_withdraw_and_after_equipment_controls_without_refunding()
 {
    let mut f = Box::pin(Fixture::new()).await;
    Box::pin(f.activate()).await;
    let start = f
        .choose(f.pc(0), "Grapple Small armored figure with right hand")
        .await;
    Box::pin(f.cold(start)).await;
    let save = f
        .choose(TableTransportChannel::Host, "Resist Grapple with Dexterity")
        .await;
    Box::pin(assert_available(&f, &save)).await;
    Box::pin(f.cold(save)).await;
    let withdraw = f.choose(f.pc(0), "Withdraw this Grapple attempt").await;
    Box::pin(assert_available(&f, &withdraw)).await;
    Box::pin(f.cold(withdraw)).await;
    let after = f.state().await;
    let finish = f.choose(f.pc(0), "Finish without changing equipment").await;
    Box::pin(assert_available(&f, &finish)).await;
    Box::pin(f.cold(finish)).await;
    let settled = f.state().await;
    assert_eq!(
        settled.rules.as_ref().unwrap().timing,
        after.rules.as_ref().unwrap().timing
    );
    assert_eq!(
        settled
            .encounter
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .budget,
        after
            .encounter
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .budget
    );
    assert!(
        Box::pin(read(&f, request(&f, f.pc(0)).await))
            .await
            .handles
            .is_empty()
    );
    f.close().await;
}

#[tokio::test]
async fn current_grapple_options_bind_same_audience_source_ownership_and_keep_free_off_turn_release()
 {
    let mut f = Box::pin(Fixture::with_opponent(
        "goblin-warrior",
        CreatureSize::Small,
        true,
    ))
    .await;
    Box::pin(f.activate()).await;
    let source = TableTransportChannel::SourceCreature {
        player_id: f.players[1],
        actor: f.goblin,
    };
    let control = f
        .request(
            TableTransportChannel::Host,
            TableTransportInput::Action(Box::new(TableAction::SetSourceCreatureController {
                actor: f.goblin,
                controller: CreatureController::Player(f.players[1]),
            })),
        )
        .await;
    Box::pin(f.cold(control)).await;
    let pc = f.pc(0);
    Box::pin(f.cold_action(pc, TacticalAction::EndTurn)).await;
    let view = f.view(source.clone()).await;
    let option = view
        .grapple
        .as_ref()
        .unwrap()
        .choices
        .iter()
        .find(|choice| {
            choice.actor == f.goblin
                && choice.label.starts_with("Grapple Other guard ")
                && choice.label.ends_with("with right hand")
        })
        .unwrap()
        .key;
    let initial = request(&f, source.clone()).await;
    assert!(
        Box::pin(read(&f, initial.clone()))
            .await
            .handles
            .contains(&option)
    );
    // The same audience revision cannot substitute its PC for its selected source.
    let same_audience_pc = request(&f, f.pc(1)).await;
    assert_eq!(same_audience_pc.revision, initial.revision);
    assert!(
        Box::pin(read(&f, same_audience_pc))
            .await
            .handles
            .is_empty()
    );
    assert!(
        Box::pin(read(&f, request(&f, TableTransportChannel::Host).await))
            .await
            .handles
            .is_empty()
    );
    let start = f
        .request(
            source.clone(),
            TableTransportInput::GrappleChoice { handle: option },
        )
        .await;
    Box::pin(f.cold(start)).await;
    let save = f
        .choose(TableTransportChannel::Host, "Resist Grapple with Strength")
        .await;
    Box::pin(assert_available(&f, &save)).await;
    Box::pin(f.cold(save)).await;
    Box::pin(f.cold_action(
        TableTransportChannel::Host,
        TacticalAction::VoluntarilyFailSave,
    ))
    .await;
    let finish = f
        .choose(source.clone(), "Finish without changing equipment")
        .await;
    Box::pin(assert_available(&f, &finish)).await;
    Box::pin(f.cold(finish)).await;
    let settled = f.state().await;
    assert!(
        settled
            .rules
            .as_ref()
            .unwrap()
            .timing
            .as_ref()
            .unwrap()
            .action_spent
    );
    assert_eq!(
        settled
            .encounter
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .budget
            .attacks_remaining,
        0
    );
    let source_view = f.view(source.clone()).await;
    let release = source_view
        .grapple
        .as_ref()
        .unwrap()
        .choices
        .iter()
        .find(|choice| choice.actor == f.goblin && choice.label.starts_with("Release "))
        .unwrap()
        .key;
    let owned_request = request(&f, source.clone()).await;
    let available = Box::pin(read(&f, owned_request.clone())).await;
    assert!(available.handles.contains(&release));
    assert!(
        source_view
            .grapple
            .as_ref()
            .unwrap()
            .choices
            .iter()
            .filter(|choice| choice.label.starts_with("Grapple "))
            .all(|choice| !available.handles.contains(&choice.key))
    );
    let control = f
        .request(
            TableTransportChannel::Host,
            TableTransportInput::Action(Box::new(TableAction::SetSourceCreatureController {
                actor: f.goblin,
                controller: CreatureController::Host,
            })),
        )
        .await;
    Box::pin(f.cold(control)).await;
    Box::pin(refused(&f, owned_request)).await;
    Box::pin(refused(&f, request(&f, source).await)).await;
    assert!(
        Box::pin(read(&f, request(&f, f.pc(1)).await))
            .await
            .handles
            .is_empty()
    );
    Box::pin(f.cold_action(TableTransportChannel::Host, TacticalAction::EndTurn)).await;
    let off_turn = f.state().await;
    let timing = off_turn.rules.as_ref().unwrap().timing.as_ref().unwrap();
    assert_ne!(timing.order[timing.index].actor, f.goblin);
    let host_view = f.view(TableTransportChannel::Host).await;
    let release = host_view
        .grapple
        .unwrap()
        .choices
        .into_iter()
        .find(|choice| choice.actor == f.goblin && choice.label.starts_with("Release "))
        .unwrap()
        .key;
    let accepted = f
        .request(
            TableTransportChannel::Host,
            TableTransportInput::GrappleChoice { handle: release },
        )
        .await;
    Box::pin(assert_available(&f, &accepted)).await;
    Box::pin(f.cold(accepted)).await;
    let released = f.state().await;
    assert_eq!(
        released.rules.as_ref().unwrap().timing,
        off_turn.rules.as_ref().unwrap().timing
    );
    assert_eq!(
        released
            .encounter
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .budget,
        off_turn
            .encounter
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .budget
    );
    assert!(released.rules.as_ref().unwrap().tactical_grapples.is_none());
    f.close().await;
}
