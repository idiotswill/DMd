use super::*;
use dmd_persistence::{CampaignExport, CampaignStateSnapshotCodec};
use serde_json::{Value, json};

pub(super) async fn run() {
    let path = std::env::temp_dir().join(format!(
        "dmd-current-creation-history-{}.sqlite",
        CommandId::new().0
    ));
    let mut f = Box::pin(blank(&path)).await;
    Box::pin(reject_creation_inputs(&f)).await;
    let original_request = Box::pin(create(&mut f, &path, "greatsword")).await;
    let original = export_campaign(&f.pool, f.campaign).await.unwrap();
    assert!(
        state(&f).await.items.is_empty(),
        "source history must be authenticated before inventory exists"
    );
    let destination = open_sqlite("sqlite::memory:").await.unwrap();
    let destination_runtime = runtime(destination.clone());
    destination_runtime
        .create_table_campaign(
            CampaignId::new(),
            "Preserved unrelated campaign",
            TableContract::default(),
        )
        .await
        .unwrap();
    let destination_rows = all_rows(&destination).await;
    for mutation in 0..9 {
        let mut forged = original.clone();
        match mutation {
            0 => {
                let mut latest = forged.snapshots[0].clone();
                latest.event_sequence = forged.current_state.applied_event_sequence;
                latest.state_json = forged.current_state.state_json.clone();
                forged.snapshots = vec![latest];
            }
            1 => {
                let mut current: Value =
                    serde_json::from_str(&forged.current_state.state_json).unwrap();
                current["table"]["character_profiles"][f.characters[0].0.to_string()]["creation_source"]
                    ["definition_fingerprint"] = json!("0000000000000000");
                forged.current_state.state_json = serde_json::to_string(&current).unwrap();
            }
            2..=5 => {
                let row = forged
                    .event_journal
                    .iter_mut()
                    .find(|row| row.command_id == original_request.command_id.0.to_string())
                    .unwrap();
                let mut event: Value = serde_json::from_str(&row.payload_json).unwrap();
                match mutation {
                    2 => {
                        event["action"]["CreateCharacterFromSource"]["source"]["profile_id"] =
                            json!("foreign")
                    }
                    3 => {
                        event["rules_event"]["action"]["CreateCharacterFromSource"]["source"]["definition_fingerprint"] =
                            json!("0000000000000000")
                    }
                    4 => event["rules_event"] = Value::Null,
                    _ => {
                        event["rules_event"]["action"]["CreateCharacterFromSource"]["input"]["purchases"] =
                            json!([])
                    }
                }
                row.payload_json = serde_json::to_string(&event).unwrap();
            }
            6 => {
                let row = forged
                    .command_audit
                    .iter_mut()
                    .find(|row| row.id == original_request.command_id.0.to_string())
                    .unwrap();
                let mut audit: Value = serde_json::from_str(&row.payload_json).unwrap();
                assert!(audit["action"]["CreateCharacterFromSource"]["source"].is_object());
                audit["action"]["CreateCharacterFromSource"]["source"]["profile_id"] =
                    json!("foreign");
                row.payload_json = serde_json::to_string(&audit).unwrap();
            }
            7 => {
                let row = forged
                    .event_journal
                    .iter_mut()
                    .find(|row| row.command_id == original_request.command_id.0.to_string())
                    .unwrap();
                let event: Value = serde_json::from_str(&row.payload_json).unwrap();
                row.event_kind = dmd_rules::RULES_EVENT_KIND.into();
                row.payload_json = serde_json::to_string(&event["rules_event"]).unwrap();
            }
            _ => {
                let mut current: Value =
                    serde_json::from_str(&forged.current_state.state_json).unwrap();
                current["table"]["character_profiles"][f.characters[0].0.to_string()]["creation_source"] =
                    Value::Null;
                forged.current_state.state_json = serde_json::to_string(&current).unwrap();
            }
        }
        assert!(
            Box::pin(destination_runtime.restore_campaign(&forged))
                .await
                .is_err(),
            "history mutation {mutation}"
        );
        assert_eq!(
            all_rows(&destination).await,
            destination_rows,
            "history mutation {mutation}"
        );
    }
    Box::pin(old_schema_controls(
        &original,
        &destination_runtime,
        &destination,
        &destination_rows,
    ))
    .await;
    Box::pin(legacy_schema_three_absent_and_null_controls()).await;
    Box::pin(materialized_history_controls(
        &mut f,
        &path,
        &destination_runtime,
        &destination,
        &destination_rows,
    ))
    .await;
    Box::pin(destination_runtime.restore_campaign(&original))
        .await
        .unwrap();
    assert_eq!(
        destination_runtime
            .open_campaign(f.campaign)
            .await
            .unwrap()
            .state(),
        &serde_json::from_str::<CampaignState>(&original.current_state.state_json).unwrap()
    );
    // Frozen legacy creator remains valid and does not acquire the current pin.
    let old_input = action(TableAction::CreateCharacter {
        character_id: CharacterId::new(),
        entity_id: EntityId::new(),
        player_id: f.players[0],
        input: input("Legacy creation remains accepted"),
    });
    let old_request = Box::pin(step(&mut f, &path, TableTransportChannel::Host, old_input)).await;
    let TableTransportInput::Action(old_action) = old_request.input else {
        unreachable!()
    };
    let TableAction::CreateCharacter { character_id, .. } = old_action.as_ref() else {
        unreachable!()
    };
    let current = state(&f).await;
    let legacy = &current.table.as_ref().unwrap().character_profiles[character_id];
    assert!(legacy.creation_source.is_none());
    assert!(
        !serde_json::to_string(legacy)
            .unwrap()
            .contains("creation_source")
    );
    destination.close().await;
    Box::pin(close(f, &path)).await;
}

async fn materialized_history_controls(
    f: &mut Fixture,
    path: &Path,
    target: &CampaignRuntime,
    pool: &sqlx::SqlitePool,
    unchanged: &[(String, Vec<Vec<String>>)],
) {
    let host = view(f, &TableTransportChannel::Host).await;
    let count = host
        .characters
        .iter()
        .find(|character| character.character_id == f.characters[0])
        .unwrap()
        .equipment
        .as_ref()
        .unwrap()
        .initial_item_count;
    let equipment = action(TableAction::PrepareEquipment {
        character_id: f.characters[0],
        item_ids: (0..count).map(|_| ItemId::new()).collect(),
    });
    Box::pin(step(f, path, TableTransportChannel::Host, equipment)).await;
    let original = export_campaign(&f.pool, f.campaign).await.unwrap();
    for mode in 0..3 {
        let mut forged = original.clone();
        let mut current: Value = serde_json::from_str(&forged.current_state.state_json).unwrap();
        match mode {
            0 => {
                current["rules"]["tactical_inventory"]["receipts"][0]["creation_profile"]["creation_source"]
                    ["definition_fingerprint"] = json!("0000000000000000")
            }
            1 => {
                current["rules"]["tactical_inventory"]["receipts"][0]["source"]["profile_id"] =
                    json!("human-fighter-soldier-level-1")
            }
            _ => current["rules"]["entities"][f.actors[0].0.to_string()]["hp"] = json!(11),
        }
        forged.current_state.state_json = serde_json::to_string(&current).unwrap();
        let mut latest = forged.snapshots[0].clone();
        latest.event_sequence = forged.current_state.applied_event_sequence;
        latest.state_json = forged.current_state.state_json.clone();
        forged.snapshots.push(latest);
        assert!(
            Box::pin(target.restore_campaign(&forged)).await.is_err(),
            "materialized history {mode}"
        );
        assert_eq!(all_rows(pool).await.as_slice(), unchanged);
    }
}

async fn reject_creation_inputs(f: &Fixture) {
    let source = f
        .runtime
        .character_creation_options(f.campaign)
        .await
        .unwrap()
        .source;
    let creation = TableAction::CreateCharacterFromSource {
        character_id: f.characters[0],
        entity_id: f.actors[0],
        player_id: f.players[0],
        source,
        input: input("Current candidate"),
    };
    let valid = serde_json::to_value(&creation).unwrap();
    for mode in 0..5 {
        let mut invalid = creation.clone();
        let TableAction::CreateCharacterFromSource {
            input,
            character_id,
            entity_id,
            ..
        } = &mut invalid
        else {
            unreachable!()
        };
        match mode {
            0 => input.purchases.push(EquipmentChoice {
                item_id: "greatsword".into(),
                quantity: 5,
            }),
            1 => input.purchases.push(input.purchases[0].clone()),
            2 => input.purchases.push(EquipmentChoice {
                item_id: "arrows".into(),
                quantity: 1,
            }),
            3 => *character_id = CharacterId(Default::default()),
            _ => *entity_id = EntityId(Default::default()),
        }
        let invalid = request(f, TableTransportChannel::Host, action(invalid)).await;
        Box::pin(reject(f, invalid)).await;
    }
    for mode in 0..4 {
        let mut malformed = valid.clone();
        let body = malformed["CreateCharacterFromSource"]
            .as_object_mut()
            .unwrap();
        match mode {
            0 => {
                body.remove("source");
            }
            1 => {
                body.insert("source".into(), Value::Null);
            }
            2 => {
                body.get_mut("source").unwrap()["definition_fingerprint"] = json!("NOT-A-PIN");
            }
            _ => {
                body.get_mut("source").unwrap()["unexpected"] = json!(true);
            }
        }
        let bytes = serde_json::to_string(&malformed).unwrap();
        if let Ok(action_value) = serde_json::from_str::<TableAction>(&bytes) {
            let invalid = request(f, TableTransportChannel::Host, action(action_value)).await;
            Box::pin(reject(f, invalid)).await;
        } else {
            assert!(
                mode != 2,
                "wrong fingerprint is typed and must be refused by the application"
            );
        }
    }
    let TableAction::CreateCharacterFromSource { source, .. } = &creation else {
        unreachable!()
    };
    let source_json = serde_json::to_string(source).unwrap();
    let serialized = serde_json::to_string(&creation).unwrap();
    for replacement in [
        format!("\"source\":null,\"source\":{source_json}"),
        format!("\"source\":{source_json},\"source\":null"),
    ] {
        let duplicate = serialized.replace(&format!("\"source\":{source_json}"), &replacement);
        assert!(serde_json::from_str::<TableAction>(&duplicate).is_err());
    }
    let mut old = input("Unsupported legacy purchase");
    old.purchases.push(EquipmentChoice {
        item_id: "greatsword".into(),
        quantity: 1,
    });
    let old = request(
        f,
        TableTransportChannel::Host,
        action(TableAction::CreateCharacter {
            character_id: f.characters[0],
            entity_id: f.actors[0],
            player_id: f.players[0],
            input: old,
        }),
    )
    .await;
    Box::pin(reject(f, old)).await;
}

async fn old_schema_controls(
    original: &CampaignExport,
    runtime: &CampaignRuntime,
    pool: &sqlx::SqlitePool,
    unchanged: &[(String, Vec<Vec<String>>)],
) {
    let codec = CampaignStateSnapshotCodec::new();
    for schema in 1..=3 {
        for shadow in 0..4 {
            let mut legacy: Value =
                serde_json::from_str(&original.current_state.state_json).unwrap();
            legacy["schema_version"] = json!(schema);
            legacy.as_object_mut().unwrap().remove("encounter");
            let mut bytes = serde_json::to_string(&legacy).unwrap();
            let pin = dmd_rules::current_character_creation_source().unwrap();
            let serialized = serde_json::to_string(&pin).unwrap();
            // JSON Value sorts object fields; locate the exact stored source text.
            let canonical_value =
                serde_json::to_string(&serde_json::to_value(&pin).unwrap()).unwrap();
            let needle = format!("\"creation_source\":{canonical_value}");
            let replacement = match shadow {
                0 => needle.clone(),
                1 => format!("\"creation_source\":{serialized},\"creation_source\":null"),
                2 => format!("\"creation_source\":null,\"creation_source\":{serialized}"),
                _ => "\"creation_source\":{}".into(),
            };
            assert!(bytes.contains(&needle));
            bytes = bytes.replace(&needle, &replacement);
            let error = codec.decode_state(schema, &bytes).unwrap_err().to_string();
            assert!(
                error.contains("future character creation") || error.contains("duplicate field"),
                "schema {schema}, shadow {shadow}: {error}"
            );
            let mut forged = original.clone();
            forged.state_schema_version = schema;
            forged.current_state.schema_version = i64::from(schema);
            forged.current_state.state_json = bytes;
            assert!(Box::pin(runtime.restore_campaign(&forged)).await.is_err());
            assert_eq!(all_rows(pool).await.as_slice(), unchanged);
            // A poisoned legacy image is negative input only. Migration must
            // reject it atomically even when the SQL schema is already current.
            let poisoned = open_sqlite("sqlite::memory:").await.unwrap();
            let candidate = driver::runtime(poisoned.clone());
            Box::pin(candidate.restore_campaign(original))
                .await
                .unwrap();
            sqlx::query("UPDATE campaign_state_current SET schema_version = ?, state_json = ? WHERE campaign_id = ?")
                .bind(i64::from(schema)).bind(&forged.current_state.state_json).bind(&original.campaign_id)
                .execute(&poisoned).await.unwrap();
            let before = all_rows(&poisoned).await;
            assert!(dmd_persistence::migrate_sqlite(&poisoned).await.is_err());
            assert_eq!(all_rows(&poisoned).await, before);
            poisoned.close().await;
        }
    }
}

async fn legacy_schema_three_absent_and_null_controls() {
    let legacy = Box::pin(Fixture::new()).await;
    let current = state(&legacy).await;
    let mut value = serde_json::to_value(&current).unwrap();
    value["schema_version"] = json!(3);
    value.as_object_mut().unwrap().remove("encounter");
    let codec = CampaignStateSnapshotCodec::new();
    let absent = serde_json::to_string(&value).unwrap();
    assert!(!absent.contains("creation_source"));
    assert_eq!(codec.decode_state(3, &absent).unwrap(), current);
    for profile in value["table"]["character_profiles"]
        .as_object_mut()
        .unwrap()
        .values_mut()
    {
        profile["creation_source"] = Value::Null;
    }
    let explicit_null = serde_json::to_string(&value).unwrap();
    assert_eq!(codec.decode_state(3, &explicit_null).unwrap(), current);
    let (key, profile) = value["table"]["character_profiles"]
        .as_object()
        .unwrap()
        .iter()
        .next()
        .unwrap();
    let profile_json = serde_json::to_string(profile).unwrap();
    let duplicated = explicit_null.replace(
        &format!("\"{key}\":{profile_json}"),
        &format!("\"{key}\":{profile_json},\"{key}\":{profile_json}"),
    );
    assert_ne!(duplicated, explicit_null);
    assert!(codec.decode_state(3, &duplicated).is_err());
    legacy.pool.close().await;
}
