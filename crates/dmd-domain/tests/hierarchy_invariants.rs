use dmd_domain::{
    Campaign, CampaignId, CampaignState, CampaignStatus, Custody, ItemId, ItemInstance, ItemState,
    Location, LocationId, Ownership, StateInvariantViolation, VersionedRef, WorldClock,
    WorldInstant,
};

fn state() -> CampaignState {
    let campaign_id = CampaignId::new();
    CampaignState::empty(
        Campaign {
            id: campaign_id,
            display_name: "Hierarchy Test".into(),
            status: CampaignStatus::Active,
            world_seed: 11,
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

#[test]
fn location_parent_cycle_is_rejected() {
    let mut state = state();
    let first_id = LocationId::new();
    let second_id = LocationId::new();
    state.locations.insert(
        first_id,
        Location {
            id: first_id,
            campaign_id: state.campaign_id(),
            display_name: "First".into(),
            parent_location_id: Some(second_id),
        },
    );
    state.locations.insert(
        second_id,
        Location {
            id: second_id,
            campaign_id: state.campaign_id(),
            display_name: "Second".into(),
            parent_location_id: Some(first_id),
        },
    );

    assert!(state.validate().iter().any(|violation| matches!(
        violation,
        StateInvariantViolation::CyclicReference(kind) if kind == "location hierarchy"
    )));
}

#[test]
fn item_container_cycle_is_rejected() {
    let mut state = state();
    let first_id = ItemId::new();
    let second_id = ItemId::new();
    state.items.insert(
        first_id,
        ItemInstance {
            id: first_id,
            campaign_id: state.campaign_id(),
            definition_id: "test.container".into(),
            display_name: "First Container".into(),
            quantity: 1,
            owner: Ownership::Unowned,
            custody: Custody::Container(second_id),
            state: ItemState::Intact,
        },
    );
    state.items.insert(
        second_id,
        ItemInstance {
            id: second_id,
            campaign_id: state.campaign_id(),
            definition_id: "test.container".into(),
            display_name: "Second Container".into(),
            quantity: 1,
            owner: Ownership::Unowned,
            custody: Custody::Container(first_id),
            state: ItemState::Intact,
        },
    );

    assert!(state.validate().iter().any(|violation| matches!(
        violation,
        StateInvariantViolation::CyclicReference(kind) if kind == "item containment"
    )));
}
