//! Genuine V1 capture continuation plus current Mage V2 creation. No positive state injection.
use super::*;
use dmd_rules::tactical_creatures::{creature_definition, creature_source_pin, source_for_profile};
use dmd_rules::tactical_definitions::bundled_mage_v2;
#[path = "mage_source_store.rs"]
mod store;

fn action(action: TableAction) -> TableTransportInput {
    TableTransportInput::Action(Box::new(action))
}

fn profile(state: &CampaignState, actor: EntityId) -> &CreatureProfile {
    state
        .rules
        .as_ref()
        .unwrap()
        .tactical_creatures
        .as_ref()
        .unwrap()
        .profile(actor)
        .unwrap()
}

async fn accept_action(
    f: &mut Fixture,
    channel: TableTransportChannel,
    input: TableTransportInput,
) -> TableTransportRequest {
    let shown = f.view(&channel).await;
    let request = f.request(channel, &shown, input);
    Box::pin(f.accept_cold(request.clone())).await;
    request
}

async fn host(f: &mut Fixture, declaration: TableAction) -> TableTransportRequest {
    Box::pin(accept_action(
        f,
        TableTransportChannel::Host,
        action(declaration),
    ))
    .await
}

async fn all_views(f: &Fixture) -> Vec<TablePresentedView> {
    let mut viewers = vec![TableViewer::Host];
    let state = f.state().await;
    let mut players = state.players.keys().copied().collect::<Vec<_>>();
    players.sort_by_key(|id| id.0);
    viewers.extend(players.into_iter().map(TableViewer::Player));
    let mut views = Vec::new();
    for viewer in viewers {
        views.push(
            f.app
                .presented_table_view(f.campaign, viewer)
                .await
                .unwrap(),
        );
    }
    views
}

async fn unchanged_rejection(f: &Fixture, request: TableTransportRequest) {
    let views = all_views(f).await;
    let before = export_campaign(&f.pool, f.campaign).await.unwrap();
    let rows = store::typed_rows(&f.pool).await;
    assert!(
        Box::pin(f.app.submit_presented_table(request))
            .await
            .is_err()
    );
    f.assert_export(&before).await;
    assert_eq!(all_views(f).await, views);
    assert_eq!(store::typed_rows(&f.pool).await, rows);
}

async fn current_creation(f: &Fixture, size: CreatureSize) -> TableCreatureCreation {
    let shown = f.view(&TableTransportChannel::Host).await;
    let options = f
        .app
        .table_creature_options(TableCreatureOptionsRequest {
            campaign_id: f.campaign,
            channel: TableTransportChannel::Host,
            revision: shown.revision,
        })
        .await
        .unwrap();
    let mages = options
        .iter()
        .filter(|option| option.definition_id == "mage")
        .collect::<Vec<_>>();
    assert_eq!(mages.len(), 1);
    let option = mages[0];
    let pin = creature_source_pin(bundled_mage_v2().unwrap()).unwrap();
    assert_eq!(option.source.as_ref(), Some(&pin));
    assert_eq!(option.item_count, 2);
    assert!(!option.ammunition_required);
    assert_eq!(option.sizes, [CreatureSize::Medium, CreatureSize::Small]);
    assert_eq!(option.abilities, ["Spellcasting", "Protective Magic"]);
    assert!(!option.omitted_features.is_empty());
    TableCreatureCreation {
        entity_id: EntityId::new(),
        name: "Private current source".into(),
        definition_id: "mage".into(),
        source: Some(pin),
        size,
        additional_languages: vec!["dwarvish".into(), "elvish".into(), "draconic".into()],
        ammunition_units: 0,
        item_ids: vec![ItemId::new(), ItemId::new()],
    }
}

fn assert_pair(state: &CampaignState, old: &CreatureProfile, new: &CreatureProfile) {
    assert_eq!(profile(state, old.actor), old);
    assert_eq!(profile(state, new.actor), new);
    assert_eq!(
        source_for_profile(old).unwrap(),
        creature_definition("mage").unwrap()
    );
    assert_eq!(source_for_profile(new).unwrap(), bundled_mage_v2().unwrap());
    assert_ne!(old.source, new.source);
    let pack =
        dmd_rules::RulesPack::from_json(include_str!("../../../../content/srd-5.2.1/kernel.json"))
            .unwrap();
    assert_eq!(
        dmd_rules::tactical_grapple_sources::ordinary_grapple_anatomy(state, old.actor, &pack)
            .unwrap(),
        None
    );
    assert_eq!(
        dmd_rules::tactical_grapple_sources::ordinary_grapple_anatomy(state, new.actor, &pack)
            .unwrap(),
        Some(GrappleAnatomyProof::Creature {
            source: new.source.clone(),
            ordinary_hands: OrdinaryHandAnatomy::TwoHandsV1
        })
    );
    let mut items = state
        .items
        .values()
        .filter(|item| item.custody == Custody::Entity(new.actor))
        .map(|item| {
            assert_eq!(item.campaign_id, state.campaign_id());
            assert_eq!(item.state, ItemState::Intact);
            (item.definition_id.as_str(), item.quantity)
        })
        .collect::<Vec<_>>();
    items.sort();
    assert_eq!(items, [("spell-material:mage-armor", 1), ("wand", 1)]);
    let loadout = state
        .rules
        .as_ref()
        .unwrap()
        .tactical_inventory
        .as_ref()
        .unwrap()
        .loadout(new.actor)
        .unwrap();
    assert_eq!(
        loadout.hands.hands,
        [HandAssignment::Free, HandAssignment::Free]
    );
    assert_eq!(state.rules.as_ref().unwrap().entities[&new.actor].hp, 81);
    assert_eq!(
        dmd_rules::tactical_defenses::effective_armor_class(state, new.actor).unwrap(),
        12
    );
}

async fn reject_profile_swaps_in_populated_destination(
    f: &Fixture,
    old: &CreatureProfile,
    new: &CreatureProfile,
) {
    let genuine = export_campaign(&f.pool, f.campaign).await.unwrap();
    let path = f
        .directory
        .join(format!("mage-populated-destination-{}.sqlite", new.actor.0));
    let pool = open_sqlite_path(&path).await.unwrap();
    let app = runtime(pool.clone());
    // This is the normal new-campaign producer, not a patched imported history.
    let other = CampaignState::empty(
        Campaign {
            id: CampaignId::new(),
            display_name: "Existing independent destination campaign".into(),
            status: CampaignStatus::Active,
            world_seed: 381,
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
    app.create_campaign(&other).await.unwrap();
    let other_before = export_campaign(&pool, other.campaign_id()).await.unwrap();
    let rows = store::typed_rows(&pool).await;
    assert!(rows.values().any(|values| !values.is_empty()));
    for (actor, wrong) in [
        (old.actor, new.source.clone()),
        (new.actor, old.source.clone()),
    ] {
        let mut forged = genuine.clone();
        let mut state = CampaignState::decode_json(&forged.current_state.state_json).unwrap();
        state
            .rules
            .as_mut()
            .unwrap()
            .tactical_creatures
            .as_mut()
            .unwrap()
            .profiles
            .iter_mut()
            .find(|p| p.actor == actor)
            .unwrap()
            .source = wrong;
        source_for_profile(profile(&state, actor)).unwrap();
        // Both source revisions are valid. Only original replay proves this actor's revision.
        dmd_rules::tactical_creatures::validate_creature_profile(
            &state,
            profile(&state, actor),
            &state.rules.as_ref().unwrap().entities[&actor],
        )
        .unwrap();
        forged.current_state.state_json = state.encode_json().unwrap();
        assert_eq!(forged.event_journal, genuine.event_journal);
        assert_eq!(forged.snapshots, genuine.snapshots);
        assert_eq!(
            forged.table_projection_history,
            genuine.table_projection_history
        );
        assert_eq!(
            forged.table_transport_bindings,
            genuine.table_transport_bindings
        );
        assert!(Box::pin(app.restore_campaign(&forged)).await.is_err());
        assert_eq!(store::typed_rows(&pool).await, rows);
        let mut other_after = export_campaign(&pool, other.campaign_id()).await.unwrap();
        other_after
            .exported_at_utc
            .clone_from(&other_before.exported_at_utc);
        assert_eq!(other_after, other_before);
    }
    // The genuine complete history must still restore into that same populated destination.
    assert_eq!(
        Box::pin(app.restore_campaign(&genuine))
            .await
            .unwrap()
            .state(),
        image(&genuine).as_ref()
    );
    let mut restored = export_campaign(&pool, f.campaign).await.unwrap();
    restored
        .exported_at_utc
        .clone_from(&genuine.exported_at_utc);
    assert_eq!(restored, genuine);
    pool.close().await;
    drop(app);
    drop(pool);
    let pool = open_sqlite_path(&path).await.unwrap();
    let app = runtime(pool.clone());
    assert_pair(
        Box::pin(app.resume_campaign(f.campaign))
            .await
            .unwrap()
            .state(),
        old,
        new,
    );
    pool.close().await;
    drop(app);
    drop(pool);
}

#[tokio::test]
async fn genuine_original_mage_shield_and_current_mage_revisions_coexist_with_cold_retries() {
    let mut f = Box::pin(Fixture::restore(SHIELD_SELECTED)).await;
    let original = f.state().await;
    let resolution = flow(&original).resolution.as_ref().unwrap();
    let hit = resolution.hit_review.as_ref().unwrap();
    assert_eq!(hit.stage, TacticalHitReviewStage::Selected);
    let old_actor = hit.respondent.as_ref().unwrap().actor;
    let old = profile(&original, old_actor).clone();
    assert_eq!(old.source.definition_fingerprint, "af0f81ba7833b9c4");
    let channel = controller(&original, old_actor);
    let TableTransportChannel::SourceCreature { player_id, .. } = channel else {
        panic!("original Mage has its genuine player owner");
    };
    let channel = TableTransportChannel::SourceCreature {
        player_id,
        actor: old_actor,
    };
    Box::pin(f.retry_original_bindings()).await;
    let shown = f.view(&channel).await;
    let response = shown
        .tactical
        .as_ref()
        .unwrap()
        .hit
        .as_ref()
        .unwrap()
        .response
        .as_ref()
        .unwrap();
    let choice = response.shield[0].clone();
    assert_eq!(choice.spell_id, "shield");
    assert_eq!(protective_uses(&original, old_actor), 0);
    let request = f.request(
        channel,
        &shown,
        TableTransportInput::HitResponse {
            handle: response.key,
            decision: Box::new(TableHitInput::Cast { choice }),
        },
    );
    Box::pin(f.accept_cold(request)).await;
    let settled = f.state().await;
    assert!(flow(&settled).resolution.is_none());
    assert_eq!(profile(&settled, old_actor), &old);
    assert_eq!(
        settled.rules.as_ref().unwrap().rolls,
        original.rules.as_ref().unwrap().rolls
    );
    assert_eq!(settled.rules.as_ref().unwrap().entities[&old_actor].hp, 81);
    assert_eq!(protective_uses(&settled, old_actor), 1);
    assert_eq!(
        dmd_rules::tactical_defenses::effective_armor_class(&settled, old_actor).unwrap(),
        17
    );
    Box::pin(host(
        &mut f,
        TableAction::Tactical {
            action: TacticalAction::UpgradeExecutionTo {
                execution: TacticalExecutionVersion::EncounterReleaseV1,
            },
        },
    ))
    .await;
    let upgraded = f.state().await;
    let timing = upgraded.rules.as_ref().unwrap().timing.as_ref().unwrap();
    let current = timing.order[timing.index].actor;
    Box::pin(accept_action(
        &mut f,
        controller(&upgraded, current),
        tactical(TacticalAction::EndTurn),
    ))
    .await;
    Box::pin(host(
        &mut f,
        TableAction::Tactical {
            action: TacticalAction::ConcludeHostilities {
                cadence: AftermathCadence::ContinueExistingOrder,
                ruling:
                    "The original paid Shield and owner turn have settled; conclude this conflict."
                        .into(),
            },
        },
    ))
    .await;
    dmd_rules::tactical::encounter_release_preflight(&*f.state().await).unwrap();
    Box::pin(host(
        &mut f,
        TableAction::Tactical {
            action: TacticalAction::FinishEncounter,
        },
    ))
    .await;
    assert_eq!(flow(&*f.state().await).phase, TacticalPhase::Finished);
    let historical_items = f.state().await.items.clone();
    let mut created = Vec::new();
    for size in [CreatureSize::Small, CreatureSize::Medium] {
        let creation = current_creation(&f, size).await;
        let mut wrong_sources = vec![None, Some(old.source.clone())];
        for field in 0..4 {
            let mut pin = creation.source.clone().unwrap();
            match field {
                0 => pin.ruleset_id.push_str("-changed"),
                1 => pin.ruleset_version.push_str("-changed"),
                2 => pin.definition_id.push_str("-changed"),
                _ => pin.definition_fingerprint.push_str("-changed"),
            }
            wrong_sources.push(Some(pin));
        }
        for source in wrong_sources {
            let mut wrong = creation.clone();
            wrong.source = source;
            let shown = f.view(&TableTransportChannel::Host).await;
            let request = f.request(
                TableTransportChannel::Host,
                &shown,
                action(TableAction::CreateCreature {
                    creation: Box::new(wrong),
                }),
            );
            Box::pin(unchanged_rejection(&f, request)).await;
        }
        let request = Box::pin(host(
            &mut f,
            TableAction::CreateCreature {
                creation: Box::new(creation.clone()),
            },
        ))
        .await;
        let state = f.state().await;
        let new = profile(&state, creation.entity_id).clone();
        assert_eq!(new.origin.id, request.command_id);
        assert_eq!(new.source, creation.source.clone().unwrap());
        assert_eq!(new.size, size);
        assert_eq!(new.hit_points, CreatureHitPointOrigin::Average);
        assert_pair(&state, &old, &new);
        for (id, item) in &historical_items {
            assert_eq!(state.items.get(id), Some(item));
        }
        let views = all_views(&f).await;
        let before = export_campaign(&f.pool, f.campaign).await.unwrap();
        let rows = store::typed_rows(&f.pool).await;
        let accepted: TableTransportResult = serde_json::from_str(
            &before
                .table_transport_bindings
                .iter()
                .find(|binding| {
                    let saved: TableTransportRequest =
                        serde_json::from_str(&binding.request_json).unwrap();
                    saved.command_id == request.command_id
                })
                .unwrap()
                .response_json,
        )
        .unwrap();
        assert_eq!(
            Box::pin(f.app.submit_presented_table(request.clone()))
                .await
                .unwrap(),
            accepted
        );
        f.assert_export(&before).await;
        assert_eq!(all_views(&f).await, views);
        assert_eq!(store::typed_rows(&f.pool).await, rows);
        let mut changed = request;
        changed.input = action(TableAction::CreateCreature {
            creation: Box::new(TableCreatureCreation {
                name: "Changed accepted body".into(),
                ..creation
            }),
        });
        Box::pin(unchanged_rejection(&f, changed)).await;
        Box::pin(host(
            &mut f,
            TableAction::SetSourceCreatureController {
                actor: new.actor,
                controller: CreatureController::Player(player_id),
            },
        ))
        .await;
        f.reopen().await;
        assert_pair(&*f.state().await, &old, &new);
        Box::pin(reject_profile_swaps_in_populated_destination(
            &f, &old, &new,
        ))
        .await;
        created.push(new);
    }
    let final_state = f.state().await;
    assert_eq!(created.len(), 2);
    assert_ne!(created[0].actor, created[1].actor);
    assert_eq!(created[0].source, created[1].source);
    for new in &created {
        assert_pair(&final_state, &old, new);
    }
    Box::pin(f.finish()).await;
}
