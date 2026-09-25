use super::*;

#[tokio::test]
async fn source_creature_gear_is_atomic_private_and_restores_without_regranting() {
    // Separate independently awaited stages keep debug poll stack frames bounded.
    // All authority, privacy, replay and physical-inventory assertions remain active.
    let directory = std::env::temp_dir().join(format!("dmd-creature-{}", CampaignId::new().0));
    std::fs::create_dir_all(&directory).unwrap();
    let database = directory.join("campaign.sqlite");
    let url = format!("sqlite://{}", database.display());
    let pool = open_sqlite(&url).await.unwrap();
    let mut f = Fixture::with_pool(TableContract::default(), pool).await;
    let (created, actor) = verify_creature_creation(&mut f, &url).await;
    verify_creature_armor(&created, actor);
    verify_player_check_after_setup(&mut f, actor).await;
    f.pool.close().await;
    drop(f);
    std::fs::remove_file(database).unwrap();
    let _ = std::fs::remove_dir(directory);
}

async fn verify_creature_creation(f: &mut Fixture, url: &str) -> (CampaignState, EntityId) {
    let actor = EntityId::new();
    let allocations =
        dmd_rules::tactical_creature_equipment::creature_equipment_plan("goblin-warrior", 20)
            .unwrap();
    let creation = TableCreatureCreation {
        entity_id: actor,
        name: "Private sentry".into(),
        definition_id: "goblin-warrior".into(),
        size: CreatureSize::Small,
        additional_languages: vec![],
        ammunition_units: 20,
        item_ids: allocations.iter().map(|_| ItemId::new()).collect(),
    };
    let action = TableAction::CreateCreature {
        creation: Box::new(creation.clone()),
    };
    let before = f
        .runtime
        .open_campaign(f.campaign)
        .await
        .unwrap()
        .state()
        .clone();
    assert!(
        f.runtime
            .execute_table(f.player_meta(0).await, action.clone())
            .await
            .is_err()
    );
    assert_eq!(
        f.runtime.open_campaign(f.campaign).await.unwrap().state(),
        &before
    );
    let mut duplicate = creation.clone();
    duplicate.item_ids[1] = duplicate.item_ids[0];
    assert!(
        f.runtime
            .execute_table(
                f.meta(CommandIssuer::Admin, None, Some(f.session)).await,
                TableAction::CreateCreature {
                    creation: Box::new(duplicate)
                }
            )
            .await
            .is_err()
    );
    assert_eq!(
        f.runtime.open_campaign(f.campaign).await.unwrap().state(),
        &before
    );
    let meta = f.meta(CommandIssuer::Admin, None, Some(f.session)).await;
    f.runtime
        .execute_table(meta.clone(), action.clone())
        .await
        .unwrap();
    // Reopen the actual file before retrying the original accepted creation command.
    f.pool.close().await;
    f.pool = open_sqlite(url).await.unwrap();
    f.runtime = CampaignRuntime::from_content_root(
        f.pool.clone(),
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../content"),
    );
    assert!(
        f.runtime
            .execute_table(meta.clone(), action.clone())
            .await
            .unwrap()
            .already_accepted
    );
    let created = f
        .runtime
        .open_campaign(f.campaign)
        .await
        .unwrap()
        .state()
        .clone();
    assert_eq!(created.items.len(), allocations.len());
    let rules = created.rules.as_ref().unwrap();
    assert_eq!(dmd_rules::armor_class(&rules.entities[&actor]), 15);
    let loadout = rules
        .tactical_inventory
        .as_ref()
        .unwrap()
        .loadout(actor)
        .unwrap();
    let shield = loadout.shield.unwrap();
    assert_eq!(created.items[&shield].custody, Custody::Entity(actor));
    let pack: dmd_rules::RulesPack =
        serde_json::from_str(include_str!("../../../../content/srd-5.2.1/kernel.json")).unwrap();
    // NPC source modifiers cannot be silently replaced by the legacy PC test formula.
    let error = dmd_rules::resolve(
        &created,
        &f.meta(CommandIssuer::Admin, None, Some(f.session)).await,
        &dmd_rules::RulesAction::RequestTest {
            actor,
            kind: TestKind::Check {
                ability: Ability::Dexterity,
                skill: Some(Skill::Stealth),
            },
            dc: 15,
            visibility: RollVisibility::Secret,
            circumstances: Circumstances::default(),
            ruling: Ruling {
                basis: RulingBasis::Srd { page: 7 },
                reason: "A meaningful uncertain attempt".into(),
            },
            request_id: RollRequestId::new(),
        },
        &pack,
    )
    .unwrap_err();
    assert!(
        matches!(error, dmd_rules::RulesError::Prerequisite(message) if message.contains("source creature tests"))
    );
    for (definition, spent) in [("scimitar", false), ("arrows", true)] {
        let mut malformed = created.clone();
        let item = malformed
            .items
            .values_mut()
            .find(|item| item.definition_id == definition)
            .unwrap();
        if spent {
            item.state = ItemState::Spent;
        } else {
            item.quantity = 2;
        }
        assert!(
            dmd_rules::validate_state(&malformed, &pack).is_err(),
            "unheld source equipment must be checked"
        );
    }
    let view = f
        .runtime
        .table_view(f.campaign, TableViewer::Player(f.players[0]))
        .await
        .unwrap();
    assert!(
        !serde_json::to_string(&view)
            .unwrap()
            .contains("Private sentry")
    );
    assert!(
        !serde_json::to_string(&view)
            .unwrap()
            .contains(&actor.0.to_string())
    );
    let export = export_campaign(&f.pool, f.campaign).await.unwrap();
    let pool = open_sqlite("sqlite::memory:").await.unwrap();
    let restored = CampaignRuntime::from_content_root(
        pool.clone(),
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../content"),
    );
    restored.restore_campaign(&export).await.unwrap();
    assert_eq!(
        restored.resume_campaign(f.campaign).await.unwrap().state(),
        &created
    );
    assert!(
        restored
            .execute_table(meta, action)
            .await
            .unwrap()
            .already_accepted
    );
    assert_eq!(
        restored.resume_campaign(f.campaign).await.unwrap().state(),
        &created
    );

    pool.close().await;
    verify_creature_restore_guards(f, &export, actor).await;
    (created, actor)
}

fn verify_creature_armor(created: &CampaignState, actor: EntityId) {
    let shield = created
        .rules
        .as_ref()
        .unwrap()
        .tactical_inventory
        .as_ref()
        .unwrap()
        .loadout(actor)
        .unwrap()
        .shield
        .unwrap();
    // The source profile remains immutable while live custody governs shield AC.
    // This isolated query test is not a journaled loss or a replacement for the
    // unconscious-drop integration scenario in the turn scheduler.
    let mut lost = created.clone();
    let inventory = lost
        .rules
        .as_mut()
        .unwrap()
        .tactical_inventory
        .as_mut()
        .unwrap();
    let loadout = inventory
        .loadouts
        .iter_mut()
        .find(|l| l.actor == actor)
        .unwrap();
    loadout.shield = None;
    loadout.hands = WeaponLoadout::default();
    lost.items.get_mut(&shield).unwrap().custody = Custody::Missing;
    let profile = lost
        .rules
        .as_ref()
        .unwrap()
        .tactical_creatures
        .as_ref()
        .unwrap()
        .profile(actor)
        .unwrap()
        .clone();
    assert!(
        dmd_rules::tactical_creatures::validate_creature_profile(
            &lost,
            &profile,
            &lost.rules.as_ref().unwrap().entities[&actor]
        )
        .is_err()
    );
    let armor =
        dmd_rules::tactical_creature_equipment::creature_current_armor(&lost, &profile).unwrap();
    lost.rules
        .as_mut()
        .unwrap()
        .entities
        .get_mut(&actor)
        .unwrap()
        .armor = armor;
    assert_eq!(
        dmd_rules::armor_class(&lost.rules.as_ref().unwrap().entities[&actor]),
        13
    );
    dmd_rules::tactical_creatures::validate_creature_profile(
        &lost,
        &profile,
        &lost.rules.as_ref().unwrap().entities[&actor],
    )
    .unwrap();
    assert_eq!(
        dmd_rules::tactical_creature_equipment::creature_attack_gear(&profile, "scimitar").unwrap(),
        Some("scimitar")
    );
}

async fn verify_player_check_after_setup(f: &mut Fixture, actor: EntityId) {
    let before = f
        .runtime
        .open_campaign(f.campaign)
        .await
        .unwrap()
        .state()
        .clone();
    for (text, face) in [("I climb the ledge", 12), ("I use Second Wind", 7)] {
        f.runtime
            .submit_table_text(f.player_meta(0).await, text)
            .await
            .unwrap();
        let pending = f
            .runtime
            .table_view(f.campaign, TableViewer::Player(f.players[0]))
            .await
            .unwrap()
            .pending
            .unwrap();
        let request_id = RollRequestId::new();
        f.host(
            TableAction::Adjudicate {
                pending_id: pending.id,
                revision: pending.revision,
                request_id,
            },
            Some(f.session),
        )
        .await;
        verify_incomplete_table_state_rejected(f).await;
        let receipt = f
            .runtime
            .execute_table(
                f.player_meta(0).await,
                TableAction::SubmitPhysical {
                    request_id,
                    faces: vec![face],
                },
            )
            .await
            .unwrap();
        assert!(receipt.outcome.mechanics.is_some());
    }
    let after = f
        .runtime
        .open_campaign(f.campaign)
        .await
        .unwrap()
        .state()
        .clone();
    assert!(
        after.table.as_ref().unwrap().situation.challenges[0]
            .resolution
            .as_ref()
            .unwrap()
            .success
    );
    assert_eq!(
        after.rules.as_ref().unwrap().tactical_creatures,
        before.rules.as_ref().unwrap().tactical_creatures
    );
    assert_eq!(
        after.rules.as_ref().unwrap().entities[&actor],
        before.rules.as_ref().unwrap().entities[&actor]
    );
    assert_eq!(after.items, before.items);
    assert_eq!(after.rules.as_ref().unwrap().rolls.len(), 2);
    let export = export_campaign(&f.pool, f.campaign).await.unwrap();
    let pool = open_sqlite("sqlite::memory:").await.unwrap();
    let runtime = CampaignRuntime::from_content_root(
        pool.clone(),
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../content"),
    );
    runtime.restore_campaign(&export).await.unwrap();
    assert_eq!(
        runtime.resume_campaign(f.campaign).await.unwrap().state(),
        &after
    );
    pool.close().await;
}

async fn verify_creature_restore_guards(
    f: &Fixture,
    export: &dmd_persistence::CampaignExport,
    actor: EntityId,
) {
    let unrelated = export
        .event_journal
        .iter()
        .find_map(|row| {
            let event: TableEvent = serde_json::from_str(&row.payload_json).ok()?;
            matches!(event.action, TableAction::AddPlayer { .. }).then_some(event.meta)
        })
        .unwrap();
    let pool = open_sqlite("sqlite::memory:").await.unwrap();
    let runtime = CampaignRuntime::from_content_root(
        pool.clone(),
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../content"),
    );
    for corrupt in [
        "source",
        "profile_origin",
        "control_origin",
        "lair_origin",
        "last_operation",
        "hp",
        "ammunition",
        "anchor",
    ] {
        let mut altered = export.clone();
        let mut state = CampaignState::decode_json(&altered.current_state.state_json).unwrap();
        let rules = state.rules.as_mut().unwrap();
        let creatures = rules.tactical_creatures.as_mut().unwrap();
        match corrupt {
            "source" => creatures.profiles[0].source.definition_id = "wolf".into(),
            "profile_origin" => creatures.profiles[0].origin.id = CommandId::new(),
            "control_origin" => creatures.runtime[0].control_origin = unrelated.clone(),
            "lair_origin" => creatures.runtime[0].lair_origin = unrelated.clone(),
            "last_operation" => creatures.runtime[0].last_operation = unrelated.clone(),
            "hp" => rules.entities.get_mut(&actor).unwrap().hp -= 1,
            "ammunition" => {
                state
                    .items
                    .values_mut()
                    .find(|item| item.definition_id == "arrows")
                    .unwrap()
                    .quantity += 1
            }
            "anchor" => {}
            _ => unreachable!(),
        }
        altered.current_state.state_json = state.encode_json().unwrap();
        if corrupt == "anchor" {
            altered.snapshots = vec![dmd_persistence::SnapshotRow {
                campaign_id: altered.current_state.campaign_id.clone(),
                event_sequence: altered.current_state.applied_event_sequence,
                state_schema_version: altered.current_state.schema_version,
                state_json: altered.current_state.state_json.clone(),
                created_at_utc: "2026-09-25 00:00:00".into(),
            }];
        }
        assert!(
            runtime.restore_campaign(&altered).await.is_err(),
            "{corrupt} must fail closed"
        );
        assert!(
            dmd_persistence::open_campaign(&pool, f.campaign)
                .await
                .is_err(),
            "{corrupt} must leave no partial restored campaign"
        );
    }
    pool.close().await;
}

async fn verify_incomplete_table_state_rejected(f: &Fixture) {
    let mut export = export_campaign(&f.pool, f.campaign).await.unwrap();
    let mut malformed = CampaignState::decode_json(&export.current_state.state_json).unwrap();
    assert!(malformed.table.as_ref().unwrap().roll_context.is_some());
    malformed.rules.as_mut().unwrap().pending = None;
    assert!(malformed.validate_references().is_empty());
    assert!(
        malformed
            .validate()
            .iter()
            .any(|error| matches!(error, StateInvariantViolation::InvalidTableState(_)))
    );
    export.current_state.state_json = malformed.encode_json().unwrap();
    let pool = open_sqlite("sqlite::memory:").await.unwrap();
    let runtime = CampaignRuntime::from_content_root(
        pool.clone(),
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../content"),
    );
    assert!(runtime.restore_campaign(&export).await.is_err());
    assert!(
        dmd_persistence::open_campaign(&pool, f.campaign)
            .await
            .is_err()
    );
    pool.close().await;
}
