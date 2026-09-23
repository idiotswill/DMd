use dmd_domain::{
    AgentRef, Belief, BeliefBasis, BeliefConfidence, BeliefId, Campaign, CampaignId, CampaignState,
    CampaignStatus, Claim, ClaimId, ClaimSource, Custody, EventId, Fact, FactId, FactProvenance,
    FactValue, Faction, FactionId, FactionStatus, ItemId, ItemInstance, ItemState, KnowledgeHolder,
    KnowledgeId, KnowledgeRecord, KnowledgeTarget, Location, LocationId, Ownership, Proposition,
    StateInvariantViolation, SubjectRef, VersionedRef, WorldClock, WorldInstant,
};

fn state() -> CampaignState {
    let campaign_id = CampaignId::new();
    CampaignState::empty(
        Campaign {
            id: campaign_id,
            display_name: "Information Test".into(),
            status: CampaignStatus::Active,
            world_seed: 9,
            ruleset: VersionedRef {
                id: "test.rules".into(),
                version: "1".into(),
            },
            content_packs: vec![],
        },
        WorldClock {
            now: WorldInstant(20),
            calendar_id: "test.calendar".into(),
        },
    )
}

fn location(state: &mut CampaignState) -> LocationId {
    let location = Location {
        id: LocationId::new(),
        campaign_id: state.campaign_id(),
        display_name: "Archive".into(),
        parent_location_id: None,
    };
    let id = location.id;
    state.locations.insert(id, location);
    id
}

fn faction(state: &mut CampaignState) -> FactionId {
    let faction = Faction {
        id: FactionId::new(),
        campaign_id: state.campaign_id(),
        display_name: "Guild".into(),
        status: FactionStatus::Active,
    };
    let id = faction.id;
    state.factions.insert(id, faction);
    id
}

#[test]
fn document_can_source_claim_without_known_author() {
    let mut state = state();
    let archive = location(&mut state);
    let ledger = ItemInstance {
        id: ItemId::new(),
        campaign_id: state.campaign_id(),
        definition_id: "test.document".into(),
        display_name: "Ledger".into(),
        quantity: 1,
        owner: Ownership::Unowned,
        custody: Custody::Location(archive),
        state: ItemState::Intact,
    };
    let ledger_id = ledger.id;
    state.items.insert(ledger_id, ledger);

    let claim = Claim {
        id: ClaimId::new(),
        campaign_id: state.campaign_id(),
        source: ClaimSource::Item(ledger_id),
        proposition: Proposition {
            subject: SubjectRef::Location(archive),
            predicate: "was_used_by_smugglers".into(),
            value: FactValue::Boolean(true),
        },
        made_at: state.clock.now,
        source_event_id: EventId::new(),
    };
    state.claims.insert(claim.id, claim);

    assert!(state.validate().is_empty());
    assert!(state.facts.is_empty());
}

#[test]
fn faction_can_hold_established_knowledge() {
    let mut state = state();
    let faction_id = faction(&mut state);
    let fact = Fact {
        id: FactId::new(),
        campaign_id: state.campaign_id(),
        proposition: Proposition {
            subject: SubjectRef::Faction(faction_id),
            predicate: "knows_safe_route".into(),
            value: FactValue::Boolean(true),
        },
        valid_from: state.clock.now,
        valid_until: None,
        provenance: FactProvenance::InitialWorldState,
        source_event_id: None,
    };
    let fact_id = fact.id;
    state.facts.insert(fact_id, fact);
    state.knowledge.push(KnowledgeRecord {
        id: KnowledgeId::new(),
        campaign_id: state.campaign_id(),
        holder: KnowledgeHolder::Agent(AgentRef::Faction(faction_id)),
        target: KnowledgeTarget::Fact(fact_id),
        acquired_at: state.clock.now,
        source_event_id: EventId::new(),
    });

    assert!(state.validate().is_empty());
}

#[test]
fn belief_basis_must_reference_materialized_fact_or_claim() {
    let mut state = state();
    let faction_id = faction(&mut state);
    let belief = Belief {
        id: BeliefId::new(),
        campaign_id: state.campaign_id(),
        holder: AgentRef::Faction(faction_id),
        proposition: Proposition {
            subject: SubjectRef::Faction(faction_id),
            predicate: "rival_is_preparing_attack".into(),
            value: FactValue::Boolean(true),
        },
        confidence: BeliefConfidence::Possible,
        basis: vec![BeliefBasis::Claim(ClaimId::new())],
        updated_at: state.clock.now,
    };
    state.beliefs.insert(belief.id, belief);

    assert!(state.validate().iter().any(|violation| matches!(
        violation,
        StateInvariantViolation::MissingReference { owner, target }
            if owner == "belief basis" && target == "claim"
    )));
}
