//! Explicit test-controller choices for one accepted hit, never a queue drain.
use super::*;

fn controller(state: &CampaignState, actor: EntityId) -> (TableViewer, TableTransportChannel) {
    if let Some(character) = state
        .characters
        .values()
        .find(|character| character.entity_id == actor)
        && let Some(player_id) = character.controlling_player_id
    {
        return (
            TableViewer::Player(player_id),
            TableTransportChannel::Player {
                player_id,
                character_id: character.id,
            },
        );
    }
    assert!(
        !state
            .rules
            .as_ref()
            .and_then(|rules| rules.tactical_creatures.as_ref())
            .and_then(|creatures| creatures.runtime(actor))
            .is_some_and(|runtime| matches!(runtime.controller, CreatureController::Player(_))),
        "Assigned source actors require the PR43 SourceCreature channel, never Host substitution"
    );
    (TableViewer::Host, TableTransportChannel::Host)
}

/// Choose the fixture's explicit order, then let the actual target decline.
/// Each decision is a separate accepted command using that owner's current DTO.
pub(super) async fn decline_hit_responses(f: &Fixture) {
    let (
        sequence,
        order_viewer,
        order_channel,
        target_viewer,
        target_channel,
        target,
        cause,
        rolls,
        reactions,
    ) = {
        let opened = f.runtime.open_campaign(f.campaign).await.unwrap();
        let state = opened.state();
        let rules = state.rules.as_ref().unwrap();
        let resolution = state
            .encounter
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .resolution
            .as_ref()
            .unwrap();
        let hit = resolution
            .hit_review
            .as_ref()
            .expect("an actual hit must open collection");
        assert_eq!(hit.stage, TacticalHitReviewStage::Collecting);
        assert!(hit.order.is_none());
        assert!(hit.delegated_by.is_none());
        let respondent = hit.respondent.as_ref().unwrap();
        assert!(respondent.intent.is_none());
        let (order_viewer, order_channel) = controller(state, resolution.turn_actor);
        let (target_viewer, target_channel) = controller(state, respondent.actor);
        (
            state.applied_event_sequence,
            order_viewer,
            order_channel,
            target_viewer,
            target_channel,
            respondent.actor,
            hit.cause.clone(),
            rules.rolls.clone(),
            rules.timing.as_ref().unwrap().reactions_spent.clone(),
        )
    };
    let order_view = Box::pin(f.runtime.presented_table_view(f.campaign, order_viewer))
        .await
        .unwrap();
    let order = order_view
        .tactical
        .as_ref()
        .unwrap()
        .hit
        .as_ref()
        .unwrap()
        .order
        .as_ref()
        .expect("the actual turn controller must own ordering");
    Box::pin(f.runtime.submit_presented_table(TableTransportRequest {
        version: TABLE_TRANSPORT_VERSION,
        command_id: CommandId::new(),
        campaign_id: f.campaign,
        session_id: Some(f.session),
        channel: order_channel,
        revision: order_view.revision,
        input: TableTransportInput::HitResponse {
            handle: order.key,
            decision: Box::new(TableHitInput::Order {
                instruction: TacticalReactionOrdering {
                    ranked: vec![],
                    unlisted: ReactionUnlistedOrder::AfterForward,
                },
            }),
        },
    }))
    .await
    .unwrap();
    let target_view = Box::pin(f.runtime.presented_table_view(f.campaign, target_viewer))
        .await
        .unwrap();
    let response = target_view
        .tactical
        .as_ref()
        .unwrap()
        .hit
        .as_ref()
        .unwrap()
        .response
        .as_ref()
        .expect("even an ineligible target owns its acknowledgment");
    assert_eq!(response.actor, target);
    assert!(!response.selected);
    Box::pin(f.runtime.submit_presented_table(TableTransportRequest {
        version: TABLE_TRANSPORT_VERSION,
        command_id: CommandId::new(),
        campaign_id: f.campaign,
        session_id: Some(f.session),
        channel: target_channel,
        revision: target_view.revision,
        input: TableTransportInput::HitResponse {
            handle: response.key,
            decision: Box::new(TableHitInput::Respond { accept: false }),
        },
    }))
    .await
    .unwrap();
    let opened = f.runtime.open_campaign(f.campaign).await.unwrap();
    let state = opened.state();
    assert_eq!(state.applied_event_sequence, sequence + 2);
    let rules = state.rules.as_ref().unwrap();
    assert_eq!(
        rules.rolls, rolls,
        "coordination must not fabricate physical dice"
    );
    assert_eq!(rules.timing.as_ref().unwrap().reactions_spent, reactions);
    if let Some(resolution) = state
        .encounter
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .resolution
        .as_ref()
    {
        assert!(!resolution.hit_review.as_ref().is_some_and(|hit| matches!(
            hit.stage,
            TacticalHitReviewStage::Collecting | TacticalHitReviewStage::Selected
        )));
        if resolution
            .pending
            .as_ref()
            .is_some_and(|pending| pending.key.role == TacticalRollRole::AttackDamage)
        {
            assert_eq!(
                rules.pending.as_ref().unwrap().issued_by,
                cause,
                "damage remains caused by the original raw hit, not the decline"
            );
        }
    }
}

/// Opt in only at an expected successful attack roll. The underlying submit still
/// executes exactly one command, including at every miss/initiative/damage call.
pub(super) async fn roll_then_decline(f: &Fixture, host: bool, values: &[u16]) {
    Box::pin(table_attack_cases::submit(f, host, values)).await;
    Box::pin(decline_hit_responses(f)).await;
}
