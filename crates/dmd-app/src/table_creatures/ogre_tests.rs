//! Current full-pin creation and original-event replay, with malformed input controls.
use super::*;
use crate::{TableAction, TableEvent, TableOutcome, table_engine};
use dmd_rules::RulesPack;

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
fn current_ogre_creation_replays_exact_gear_and_refuses_missing_or_forged_pins() {
    let (state, meta, pack) = fixture();
    let original = serde_json::to_value(&state).unwrap();
    let source = dmd_rules::tactical_definitions::bundled_ogre().unwrap();
    let pin = creature_source_pin(source).unwrap();
    assert!(
        current_catalog()
            .unwrap()
            .iter()
            .any(|option| option.definition_id == "ogre"
                && option.source == Some(pin.clone())
                && option.item_count == 4
                && !option.ammunition_required)
    );
    let mut wrong_pin = pin.clone();
    wrong_pin.definition_fingerprint.push('x');
    for supplied in [None, Some(wrong_pin)] {
        let action = TableAction::CreateCreature {
            creation: Box::new(crate::TableCreatureCreation {
                entity_id: EntityId::new(),
                name: "Unverified Ogre".into(),
                definition_id: "ogre".into(),
                source: supplied.clone(),
                size: CreatureSize::Large,
                additional_languages: vec![],
                ammunition_units: 0,
                item_ids: (0..4).map(|_| ItemId::new()).collect(),
            }),
        };
        let live = match table_engine::resolve_table(&state, &meta, &action, &pack) {
            Ok(_) => panic!("missing or forged pin reached live creation"),
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
            Ok(_) => panic!("missing or forged pin reached historical creation"),
            Err(error) => error,
        };
        assert!(
            replay.contains(if supplied.is_some() {
                "creature source pin or definition fingerprint differs"
            } else {
                "unknown source creature"
            }),
            "{replay}"
        );
        assert_eq!(serde_json::to_value(&state).unwrap(), original);
    }
    let actor = EntityId::new();
    let ids = (0..4).map(|_| ItemId::new()).collect::<Vec<_>>();
    let create = crate::TableCreatureCreation {
        entity_id: actor,
        name: "Physical Ogre".into(),
        definition_id: "ogre".into(),
        source: Some(pin.clone()),
        size: CreatureSize::Large,
        additional_languages: vec![],
        ammunition_units: 0,
        item_ids: ids.clone(),
    };
    for mutation in 0..4 {
        let mut invalid = create.clone();
        match mutation {
            0 => {
                invalid.item_ids.pop();
            }
            1 => invalid.item_ids[1] = invalid.item_ids[0],
            2 => invalid.item_ids[0] = ItemId(uuid::Uuid::nil()),
            3 => invalid.ammunition_units = 1,
            _ => unreachable!(),
        }
        assert!(
            table_engine::resolve_table(
                &state,
                &meta,
                &TableAction::CreateCreature {
                    creation: Box::new(invalid)
                },
                &pack
            )
            .is_err()
        );
        assert_eq!(serde_json::to_value(&state).unwrap(), original);
    }
    let created = table_engine::resolve_table(
        &state,
        &meta,
        &TableAction::CreateCreature {
            creation: Box::new(create),
        },
        &pack,
    )
    .unwrap();
    assert_eq!(
        table_engine::replay_table(&state, &created.event, &pack)
            .unwrap()
            .state,
        created.state
    );
    let rules = created.state.rules.as_ref().unwrap();
    assert_eq!(
        rules
            .tactical_creatures
            .as_ref()
            .unwrap()
            .profile(actor)
            .unwrap()
            .source,
        pin
    );
    assert_eq!(rules.entities[&actor].hp, 68);
    assert_eq!(dmd_rules::armor_class(&rules.entities[&actor]), 11);
    assert_eq!(
        ids.iter()
            .filter(|id| created.state.items[*id].definition_id == "greatclub")
            .count(),
        1
    );
    assert_eq!(
        ids.iter()
            .filter(|id| created.state.items[*id].definition_id == "javelin")
            .count(),
        3
    );
    assert!(ids.iter().all(|id| created.state.items[id].quantity == 1
        && created.state.items[id].custody == Custody::Entity(actor)));
    assert_eq!(serde_json::to_value(&state).unwrap(), original);
    // The original old-source positive and its replay equality remain unchanged.
    let wolf = creature_definition("wolf").unwrap();
    let action = TableAction::CreateCreature {
        creation: Box::new(crate::TableCreatureCreation {
            entity_id: EntityId::new(),
            name: "Ordinary wolf".into(),
            definition_id: "wolf".into(),
            source: Some(creature_source_pin(wolf).unwrap()),
            size: CreatureSize::Medium,
            additional_languages: vec![],
            ammunition_units: 0,
            item_ids: vec![],
        }),
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
