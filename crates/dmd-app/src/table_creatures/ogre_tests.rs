//! Negative admission controls, not a created Ogre or accepted Ogre history.
use super::*;
use crate::{TableAction, TableEvent, TableOutcome, table_engine};

fn fixture() -> (CampaignState, CommandMeta, RulesPack) {
    let mut state = CampaignState::empty(
        Campaign {
            id: CampaignId::new(),
            display_name: "Source admission control".into(),
            status: CampaignStatus::Active,
            world_seed: 73,
            ruleset: VersionedRef {
                id: "srd-5.2".into(),
                version: "5.2.1".into(),
            },
            content_packs: vec![],
        },
        WorldClock {
            now: WorldInstant(0),
            calendar_id: "seconds".into(),
        },
    );
    let actor = EntityId::new();
    state.entities.insert(
        actor,
        WorldEntity {
            id: actor,
            campaign_id: state.campaign_id(),
            display_name: "Existing neutral entity".into(),
            kind: EntityKind::Npc,
            existence: EntityExistence::Present,
            location_id: None,
        },
    );
    let mut meta = CommandMeta {
        id: CommandId::new(),
        campaign_id: state.campaign_id(),
        session_id: None,
        issuer: CommandIssuer::Admin,
        actor: None,
        expected_event_sequence: 0,
    };
    let pack =
        RulesPack::from_json(include_str!("../../../../content/srd-5.2.1/kernel.json")).unwrap();
    // Use actual initialization rather than fabricating a source profile or grant.
    state = dmd_rules::resolve(
        &state,
        &meta,
        &dmd_rules::RulesAction::Initialize {
            entities: vec![MechanicalEntity::basic(actor)],
            house_rules: HouseRules::default(),
            ruling: Ruling {
                basis: RulingBasis::Srd { page: 7 },
                reason: "Initialize the admission control".into(),
            },
        },
        &pack,
    )
    .unwrap()
    .next_state;
    state.table = Some(TableState::new(TableContract::default()));
    meta.id = CommandId::new();
    meta.expected_event_sequence = state.applied_event_sequence;
    table_engine::validate_table(&state, &pack).unwrap();
    (state, meta, pack)
}

#[test]
fn registry_only_ogre_refuses_live_and_historical_creation_without_mutation() {
    let (state, meta, pack) = fixture();
    let original = serde_json::to_value(&state).unwrap();
    let source = dmd_rules::tactical_definitions::bundled_ogre().unwrap();
    let pin = creature_source_pin(source).unwrap();
    assert!(
        current_catalog()
            .unwrap()
            .iter()
            .all(|option| option.definition_id != "ogre")
    );
    for supplied in [None, Some(pin)] {
        let action = TableAction::CreateCreature {
            creation: crate::TableCreatureCreation {
                entity_id: EntityId::new(),
                name: "Unadmitted Ogre".into(),
                definition_id: "ogre".into(),
                source: supplied.clone(),
                size: CreatureSize::Large,
                additional_languages: vec![],
                ammunition_units: 0,
                item_ids: (0..4).map(|_| ItemId::new()).collect(),
            },
        };
        let live = match table_engine::resolve_table(&state, &meta, &action, &pack) {
            Ok(_) => panic!("unadmitted Ogre reached live creation"),
            Err(error) => error,
        };
        assert!(
            live.contains(if supplied.is_some() {
                "not an exact current admission"
            } else {
                "exact current creature source"
            }),
            "{live}"
        );
        // Deliberately forged event. No claim that an old Ogre producer existed.
        let event = TableEvent {
            meta: meta.clone(),
            action,
            outcome: TableOutcome {
                message: "Host preparation recorded.".into(),
                mechanics: None,
            },
            rules_event: None,
            tactical_event: None,
        };
        let replay = match table_engine::replay_table(&state, &event, &pack) {
            Ok(_) => panic!("unadmitted Ogre reached historical creation"),
            Err(error) => error,
        };
        assert!(
            replay.contains(if supplied.is_some() {
                "Ogre creation is not admitted"
            } else {
                "unknown source creature"
            }),
            "{replay}"
        );
        assert_eq!(serde_json::to_value(&state).unwrap(), original);
    }
    // The same valid table still admits a real old source; refusal is Ogre-specific.
    let wolf = creature_definition("wolf").unwrap();
    let action = TableAction::CreateCreature {
        creation: crate::TableCreatureCreation {
            entity_id: EntityId::new(),
            name: "Ordinary wolf".into(),
            definition_id: "wolf".into(),
            source: Some(creature_source_pin(wolf).unwrap()),
            size: CreatureSize::Medium,
            additional_languages: vec![],
            ammunition_units: 0,
            item_ids: vec![],
        },
    };
    let accepted = table_engine::resolve_table(&state, &meta, &action, &pack).unwrap();
    assert_eq!(
        table_engine::replay_table(&state, &accepted.event, &pack)
            .unwrap()
            .state,
        accepted.state
    );
    assert_eq!(serde_json::to_value(&state).unwrap(), original);
}
