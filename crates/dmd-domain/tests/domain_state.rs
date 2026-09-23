use dmd_domain::{
    Campaign, CampaignId, CampaignState, CampaignStatus, Character, CharacterId, CharacterStatus,
    Claim, ClaimId, EntityExistence, EntityId, EntityKind, EventId, FactValue, Location,
    LocationId, Player, PlayerId, PresenceRole, Proposition, Scene, SceneId, SceneMode,
    ScenePresence, SceneStatus, SubjectRef, VersionedRef, WorldClock, WorldEntity, WorldInstant,
};

fn new_state() -> CampaignState {
    let campaign_id = CampaignId::new();
    CampaignState::empty(
        Campaign {
            id: campaign_id,
            display_name: "Unrelated Test World".into(),
            status: CampaignStatus::Active,
            world_seed: 12345,
            ruleset: VersionedRef {
                id: "test.ruleset".into(),
                version: "1".into(),
            },
            content_packs: vec![],
        },
        WorldClock {
            now: WorldInstant(100),
            calendar_id: "test.calendar".into(),
        },
    )
}

fn add_location(state: &mut CampaignState, name: &str) -> LocationId {
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

fn add_character(state: &mut CampaignState, name: &str, status: CharacterStatus) -> EntityId {
    let player = Player {
        id: PlayerId::new(),
        campaign_id: state.campaign_id(),
        display_name: format!("{name} player"),
    };
    let player_id = player.id;
    state.players.insert(player_id, player);

    let entity = WorldEntity {
        id: EntityId::new(),
        campaign_id: state.campaign_id(),
        display_name: name.into(),
        kind: EntityKind::Character,
        existence: match status {
            CharacterStatus::Dead => EntityExistence::Dead,
            CharacterStatus::Retired => EntityExistence::Retired,
            CharacterStatus::Active | CharacterStatus::Absent => EntityExistence::Present,
        },
    };
    let entity_id = entity.id;
    state.entities.insert(entity_id, entity);

    let character = Character {
        id: CharacterId::new(),
        entity_id,
        campaign_id: state.campaign_id(),
        controlling_player_id: Some(player_id),
        display_name: name.into(),
        status,
    };
    state.characters.insert(character.id, character);
    entity_id
}

#[test]
fn split_party_can_have_multiple_simultaneous_scenes() {
    let mut state = new_state();
    let north = add_location(&mut state, "North Room");
    let south = add_location(&mut state, "South Room");
    let first = add_character(&mut state, "First", CharacterStatus::Active);
    let second = add_character(&mut state, "Second", CharacterStatus::Active);

    for (location_id, entity_id) in [(north, first), (south, second)] {
        let scene = Scene {
            id: SceneId::new(),
            campaign_id: state.campaign_id(),
            location_id,
            mode: SceneMode::Exploration,
            status: SceneStatus::Active,
            started_at: state.clock.now,
            presences: vec![ScenePresence {
                entity_id,
                role: PresenceRole::Participant,
            }],
        };
        state.scenes.insert(scene.id, scene);
    }

    assert_eq!(state.scenes.len(), 2);
    assert!(state.validate().is_empty());
}

#[test]
fn dead_character_remains_valid_historical_state() {
    let mut state = new_state();
    let entity_id = add_character(&mut state, "Fallen Hero", CharacterStatus::Dead);

    assert_eq!(state.entities[&entity_id].existence, EntityExistence::Dead);
    assert!(state.validate().is_empty());
}

#[test]
fn a_claim_does_not_become_world_truth() {
    let mut state = new_state();
    let location_id = add_location(&mut state, "Old Bridge");
    let witness = WorldEntity {
        id: EntityId::new(),
        campaign_id: state.campaign_id(),
        display_name: "Witness".into(),
        kind: EntityKind::Npc,
        existence: EntityExistence::Present,
    };
    let witness_id = witness.id;
    state.entities.insert(witness_id, witness);

    let claim = Claim {
        id: ClaimId::new(),
        campaign_id: state.campaign_id(),
        speaker: witness_id,
        proposition: Proposition {
            subject: SubjectRef::Location(location_id),
            predicate: "destroyed".into(),
            value: FactValue::Boolean(true),
        },
        made_at: state.clock.now,
        source_event_id: EventId::new(),
    };
    state.claims.insert(claim.id, claim);

    assert_eq!(state.claims.len(), 1);
    assert!(state.facts.is_empty());
    assert!(state.validate().is_empty());
}

#[test]
fn campaign_state_snapshot_round_trips_without_identity_loss() {
    let mut state = new_state();
    let location_id = add_location(&mut state, "Harbor District");
    let entity_id = add_character(&mut state, "Traveler", CharacterStatus::Active);
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
    state.applied_event_sequence = 17;

    assert!(state.validate().is_empty());

    let encoded = serde_json::to_string(&state).expect("campaign snapshot should serialize");
    let decoded: CampaignState =
        serde_json::from_str(&encoded).expect("campaign snapshot should deserialize");

    assert_eq!(decoded, state);
    assert_eq!(decoded.applied_event_sequence, 17);
    assert!(decoded.validate().is_empty());
}
