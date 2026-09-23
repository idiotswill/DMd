use dmd_domain::{
    Campaign, CampaignId, CampaignState, CampaignStatus, EntityExistence, EntityId, EntityKind,
    Location, LocationId, PresenceRole, Scene, SceneId, SceneMode, ScenePresence, SceneStatus,
    StateInvariantViolation, VersionedRef, WorldClock, WorldEntity, WorldInstant,
};

fn state() -> CampaignState {
    let campaign_id = CampaignId::new();
    CampaignState::empty(
        Campaign {
            id: campaign_id,
            display_name: "Location Test".into(),
            status: CampaignStatus::Active,
            world_seed: 7,
            ruleset: VersionedRef {
                id: "test.rules".into(),
                version: "1".into(),
            },
            content_packs: vec![],
        },
        WorldClock {
            now: WorldInstant(0),
            calendar_id: "test.calendar".into(),
        },
    )
}

fn location(state: &mut CampaignState, name: &str) -> LocationId {
    let location = Location {
        id: LocationId::new(),
        campaign_id: state.campaign_id(),
        display_name: name.into(),
        parent_location_id: None,
    };
    let id = location.id;
    state.locations.insert(id, location);
    id
}

fn entity(state: &mut CampaignState, location_id: LocationId) -> EntityId {
    let entity = WorldEntity {
        id: EntityId::new(),
        campaign_id: state.campaign_id(),
        display_name: "Actor".into(),
        kind: EntityKind::Npc,
        existence: EntityExistence::Present,
        location_id: Some(location_id),
    };
    let id = entity.id;
    state.entities.insert(id, entity);
    id
}

fn active_scene(state: &mut CampaignState, location_id: LocationId, entity_id: EntityId) {
    let scene = Scene {
        id: SceneId::new(),
        campaign_id: state.campaign_id(),
        location_id,
        mode: SceneMode::Social,
        status: SceneStatus::Active,
        started_at: state.clock.now,
        presences: vec![ScenePresence {
            entity_id,
            role: PresenceRole::Participant,
        }],
    };
    state.scenes.insert(scene.id, scene);
}

#[test]
fn active_participant_must_match_scene_location() {
    let mut state = state();
    let street = location(&mut state, "Street");
    let tavern = location(&mut state, "Tavern");
    let actor = entity(&mut state, street);
    active_scene(&mut state, tavern, actor);

    assert!(state.validate().iter().any(|violation| matches!(
        violation,
        StateInvariantViolation::SceneLocationMismatch { entity_id, .. } if *entity_id == actor
    )));
}

#[test]
fn entity_cannot_participate_in_two_active_scenes() {
    let mut state = state();
    let square = location(&mut state, "Square");
    let actor = entity(&mut state, square);
    active_scene(&mut state, square, actor);
    active_scene(&mut state, square, actor);

    assert!(state.validate().iter().any(|violation| matches!(
        violation,
        StateInvariantViolation::MultipleActiveSceneParticipation(entity_id) if *entity_id == actor
    )));
}
