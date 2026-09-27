//! Explicit fixture policy, never a production response default.
use dmd_domain::*;

pub struct DeclineHit {
    pub window: TacticalWorkKey,
    pub target: EntityId,
    pub turn_actor: EntityId,
    pub needs_order: bool,
    pub needs_response: bool,
}

pub fn pending(state: &CampaignState) -> Option<DeclineHit> {
    let resolution = state
        .encounter
        .as_ref()?
        .flow
        .as_ref()?
        .resolution
        .as_ref()?;
    let hit = resolution.hit_review.as_ref()?;
    if hit.stage == TacticalHitReviewStage::Resolved {
        return None;
    }
    assert_eq!(hit.stage, TacticalHitReviewStage::Collecting);
    let respondent = hit.respondent.as_ref().expect("every hit needs its target");
    assert!(
        respondent
            .intent
            .as_ref()
            .is_none_or(|intent| !intent.accepted)
    );
    Some(DeclineHit {
        window: TacticalWorkKey {
            resolution: resolution.origin.id,
            occurrence: hit.work.occurrence,
        },
        target: respondent.actor,
        turn_actor: resolution.turn_actor,
        needs_order: hit.order.is_none(),
        needs_response: respondent.intent.is_none(),
    })
}

pub fn owner_index(
    state: &CampaignState,
    actors: &[EntityId],
    players: &[PlayerId],
    actor: EntityId,
) -> Option<usize> {
    let index = actors.iter().position(|id| *id == actor).unwrap();
    let player = state
        .rules
        .as_ref()
        .and_then(|rules| rules.tactical_creatures.as_ref())
        .and_then(|creatures| {
            creatures
                .runtime
                .iter()
                .find(|runtime| runtime.actor == actor)
        })
        .map(|runtime| match runtime.controller {
            CreatureController::Player(player) => Some(player),
            CreatureController::Autonomous | CreatureController::Host => None,
        })
        .unwrap_or_else(|| {
            state
                .characters
                .values()
                .find(|c| c.entity_id == actor)
                .unwrap()
                .controlling_player_id
        });
    player.map(|player| {
        assert_eq!(
            players[index], player,
            "fixture actor must use its real owner"
        );
        index
    })
}

pub fn forward_order() -> TacticalReactionOrdering {
    TacticalReactionOrdering {
        ranked: vec![],
        unlisted: ReactionUnlistedOrder::AfterForward,
    }
}

pub fn assert_settled(state: &CampaignState, window: TacticalWorkKey) {
    if let Some(resolution) = state
        .encounter
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .resolution
        .as_ref()
        && let Some(hit) = resolution.hit_review.as_ref()
    {
        assert_eq!(resolution.origin.id, window.resolution);
        assert_eq!(hit.work.occurrence, window.occurrence);
        assert_eq!(hit.stage, TacticalHitReviewStage::Resolved);
    }
}
