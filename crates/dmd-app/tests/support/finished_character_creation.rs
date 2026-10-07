//! Real Finished history admits only new host-owned table creation between sessions.
use super::*;

#[tokio::test]
async fn finished_legacy_creation_preserves_owned_armor_and_cold_history() {
    Box::pin(creation_after_release(false)).await;
}

#[tokio::test]
async fn finished_pinned_creation_preserves_owned_armor_and_cold_history() {
    Box::pin(creation_after_release(true)).await;
}

async fn create_action(
    f: &Fixture,
    character: CharacterId,
    entity: EntityId,
    current: bool,
) -> TableAction {
    let mut input = creation("Later traveler");
    if current {
        input.purchases.push(EquipmentChoice {
            item_id: "glaive".into(),
            quantity: 1,
        });
        input.masteries = ["glaive".into(), "dagger".into(), "shortbow".into()];
        TableAction::CreateCharacterFromSource {
            character_id: character,
            entity_id: entity,
            player_id: f.players[0],
            source: f
                .runtime
                .character_creation_options(f.campaign)
                .await
                .unwrap()
                .source,
            input,
        }
    } else {
        TableAction::CreateCharacter {
            character_id: character,
            entity_id: entity,
            player_id: f.players[0],
            input,
        }
    }
}

async fn host_request(f: &Fixture, action: TableAction) -> TableTransportRequest {
    f.request(
        TableTransportChannel::Host,
        TableTransportInput::Action(Box::new(action)),
    )
    .await
}

async fn cold_host(f: &mut Fixture, action: TableAction) -> TableTransportRequest {
    let request = host_request(f, action).await;
    Box::pin(f.cold(request.clone())).await;
    request
}

fn retained_creation_state(before: &CampaignState, after: &CampaignState, new_actor: EntityId) {
    assert_eq!(after.encounter, before.encounter);
    assert_eq!(after.encounter_history, before.encounter_history);
    assert_eq!(after.scenes, before.scenes);
    assert_eq!(after.locations, before.locations);
    assert_eq!(after.items, before.items);
    assert_eq!(after.clock, before.clock);
    let mut old_rules = after.rules.as_ref().unwrap().clone();
    assert!(old_rules.entities.remove(&new_actor).is_some());
    assert_eq!(&old_rules, before.rules.as_ref().unwrap());
    for (id, original) in &before.characters {
        assert_eq!(after.characters.get(id), Some(original));
    }
    for (id, original) in &before.entities {
        assert_eq!(after.entities.get(id), Some(original));
    }
}

async fn creation_after_release(current: bool) {
    let mut f = Box::pin(Fixture::with_source("mage", CreatureSize::Medium)).await;
    let mage = f.goblin;
    let owner = f.players[0];
    Box::pin(cold_host(
        &mut f,
        TableAction::SetSourceCreatureController {
            actor: mage,
            controller: CreatureController::Player(owner),
        },
    ))
    .await;
    let owned = TableTransportChannel::SourceCreature {
        player_id: owner,
        actor: mage,
    };
    let pc = f.pc(0);
    Box::pin(f.cold_action(pc, TacticalAction::EndTurn)).await;
    let choice = f
        .view(owned.clone())
        .await
        .tactical
        .unwrap()
        .casting_options
        .into_iter()
        .flat_map(|options| options.variants)
        .find(|variant| variant.choice.actor == mage && variant.choice.spell_id == "mage-armor")
        .unwrap()
        .choice;
    Box::pin(f.cold_action(
        owned.clone(),
        TacticalAction::CastSpell {
            choice,
            targets: SpellTargetChoice::Entities(vec![mage]),
        },
    ))
    .await;
    let cast = f.state().await;
    let armor = cast
        .rules
        .as_ref()
        .unwrap()
        .tactical_effects
        .as_ref()
        .unwrap()
        .effects
        .iter()
        .find(|effect| effect.source.actor == mage)
        .unwrap()
        .clone();
    assert!(matches!(armor.expires, TacticalEffectExpiry::AtTime(_)));
    assert_eq!(
        dmd_rules::tactical_defenses::effective_armor_class(&cast, mage).unwrap(),
        15
    );

    let character = CharacterId::new();
    let entity = EntityId::new();
    let action = create_action(&f, character, entity, current).await;
    let active_request = host_request(&f, action.clone()).await;
    let old_revision = active_request.revision;
    Box::pin(f.reject(active_request)).await;
    Box::pin(cold_host(
        &mut f,
        TableAction::Tactical {
            action: TacticalAction::ConcludeHostilities {
                cadence: AftermathCadence::ContinueExistingOrder,
                ruling: "The combatants have ended hostilities; the accepted armor remains.".into(),
            },
        },
    ))
    .await;
    let active_aftermath = host_request(&f, action.clone()).await;
    Box::pin(f.reject(active_aftermath)).await;
    Box::pin(cold_host(&mut f, TableAction::EndSession)).await;
    let closed_aftermath = host_request(&f, action.clone()).await;
    Box::pin(f.reject(closed_aftermath)).await;
    Box::pin(cold_host(
        &mut f,
        TableAction::Tactical {
            action: TacticalAction::FinishEncounter,
        },
    ))
    .await;
    let before = f.state().await;
    dmd_rules::tactical::require_finished_encounter(&before).unwrap();
    assert!(before.table.as_ref().unwrap().active_session.is_none());
    assert_eq!(
        before
            .rules
            .as_ref()
            .unwrap()
            .tactical_effects
            .as_ref()
            .unwrap()
            .effects,
        vec![armor.clone()]
    );
    assert_eq!(
        before
            .rules
            .as_ref()
            .unwrap()
            .tactical_creatures
            .as_ref()
            .unwrap()
            .runtime(mage)
            .unwrap()
            .controller,
        CreatureController::Player(owner)
    );

    let request = host_request(&f, action.clone()).await;
    assert!(request.session_id.is_none());
    let mut stale = request.clone();
    stale.revision = old_revision;
    Box::pin(f.reject(stale)).await;
    let mut wrong_session = request.clone();
    wrong_session.session_id = Some(f.session);
    Box::pin(f.reject(wrong_session)).await;
    for channel in [f.pc(0), owned.clone()] {
        let wrong_owner = f
            .request(
                channel,
                TableTransportInput::Action(Box::new(action.clone())),
            )
            .await;
        Box::pin(f.reject(wrong_owner)).await;
    }
    let mut wrong_identity = action.clone();
    match &mut wrong_identity {
        TableAction::CreateCharacter { player_id, .. }
        | TableAction::CreateCharacterFromSource { player_id, .. } => *player_id = PlayerId::new(),
        _ => unreachable!(),
    }
    let wrong_identity = host_request(&f, wrong_identity).await;
    Box::pin(f.reject(wrong_identity)).await;
    let mut reused_identity = action.clone();
    match &mut reused_identity {
        TableAction::CreateCharacter { entity_id, .. }
        | TableAction::CreateCharacterFromSource { entity_id, .. } => *entity_id = f.actors[0],
        _ => unreachable!(),
    }
    let reused_identity = host_request(&f, reused_identity).await;
    Box::pin(f.reject(reused_identity)).await;
    if let TableAction::CreateCharacterFromSource { source, .. } = &action {
        let mut wrong_pin = action.clone();
        if let TableAction::CreateCharacterFromSource { source, .. } = &mut wrong_pin {
            source.definition_fingerprint = "0000000000000000".into();
        }
        assert_ne!(source.definition_fingerprint, "0000000000000000");
        let wrong_pin = host_request(&f, wrong_pin).await;
        Box::pin(f.reject(wrong_pin)).await;
    }

    // A genuine Finished phase does not grant the standalone ordinary kernel access.
    let pack =
        dmd_rules::RulesPack::from_json(include_str!("../../../../content/srd-5.2.1/kernel.json"))
            .unwrap();
    let meta = CommandMeta {
        id: CommandId::new(),
        campaign_id: f.campaign,
        session_id: None,
        issuer: CommandIssuer::Admin,
        actor: None,
        expected_event_sequence: before.applied_event_sequence,
    };
    for rule_action in [
        dmd_rules::RulesAction::CreateCharacter {
            entity_id: entity,
            input: creation("Later traveler"),
        },
        dmd_rules::RulesAction::SecondWind {
            actor: f.actors[0],
            request_id: RollRequestId::new(),
        },
    ] {
        let error = dmd_rules::resolve(&before, &meta, &rule_action, &pack).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("active tactical state requires the tactical command path")
        );
    }
    for mutation in 0..3 {
        let mut forged = before.clone();
        match mutation {
            0 => forged.encounter_history = None,
            1 => {
                forged
                    .encounter
                    .as_mut()
                    .unwrap()
                    .flow
                    .as_mut()
                    .unwrap()
                    .version = 3
            }
            _ => {
                forged.encounter_history.as_mut().unwrap().completions[0]
                    .released_by
                    .id = CommandId::new()
            }
        }
        Box::pin(reject_state_image(&f, forged)).await;
    }

    let original = export_campaign(&f.pool, f.campaign).await.unwrap();
    Box::pin(f.cold(request.clone())).await;
    let after = f.state().await;
    retained_creation_state(&before, &after, entity);
    assert_eq!(
        after.characters[&character].controlling_player_id,
        Some(owner)
    );
    assert_eq!(
        after.table.as_ref().unwrap().character_profiles[&character]
            .creation_source
            .is_some(),
        current
    );
    let saved = export_campaign(&f.pool, f.campaign).await.unwrap();
    assert!(saved.event_journal.starts_with(&original.event_journal));
    assert!(
        saved
            .table_projection_history
            .starts_with(&original.table_projection_history)
    );
    for binding in &original.table_transport_bindings {
        assert!(saved.table_transport_bindings.contains(binding));
    }

    // The original exact response also survives independent file import and reopen.
    let destination = f.directory.join("independent-created.sqlite");
    let mirror_pool = open_sqlite_path(&destination).await.unwrap();
    let mirror = runtime(mirror_pool.clone());
    Box::pin(mirror.restore_campaign(&saved)).await.unwrap();
    let accepted = Box::pin(f.runtime.submit_presented_table(request.clone()))
        .await
        .unwrap();
    assert_eq!(
        Box::pin(mirror.submit_presented_table(request.clone()))
            .await
            .unwrap(),
        accepted
    );
    mirror_pool.close().await;
    let mirror_pool = open_sqlite_path(&destination).await.unwrap();
    let mirror = runtime(mirror_pool.clone());
    assert_eq!(
        Box::pin(mirror.replay_rules(f.campaign)).await.unwrap(),
        after
    );
    assert_eq!(
        Box::pin(mirror.submit_presented_table(request.clone()))
            .await
            .unwrap(),
        accepted
    );
    let rows = all_rows(&mirror_pool).await;
    let mut changed = request.clone();
    let TableTransportInput::Action(changed_action) = &mut changed.input else {
        unreachable!()
    };
    match changed_action.as_mut() {
        TableAction::CreateCharacter { input, .. }
        | TableAction::CreateCharacterFromSource { input, .. } => input.name.push_str(" changed"),
        _ => unreachable!(),
    }
    assert_ne!(changed.input, request.input);
    assert!(
        Box::pin(mirror.submit_presented_table(changed))
            .await
            .is_err()
    );
    assert_eq!(all_rows(&mirror_pool).await, rows);
    mirror_pool.close().await;

    let view = f.view(TableTransportChannel::Host).await;
    let count = view
        .characters
        .iter()
        .find(|pc| pc.character_id == character)
        .unwrap()
        .equipment
        .as_ref()
        .unwrap()
        .initial_item_count;
    Box::pin(cold_host(
        &mut f,
        TableAction::PrepareEquipment {
            character_id: character,
            item_ids: (0..count).map(|_| ItemId::new()).collect(),
        },
    ))
    .await;
    let prepared = f.state().await;
    assert_eq!(prepared.encounter_history, before.encounter_history);
    assert_eq!(prepared.encounter, before.encounter);
    for (id, item) in &before.items {
        assert_eq!(prepared.items.get(id), Some(item));
    }
    if current {
        assert!(
            prepared
                .items
                .values()
                .any(|item| item.custody == Custody::Entity(entity)
                    && item.definition_id == "glaive")
        );
    }
    f.session = PlaySessionId::new();
    let session = f.session;
    Box::pin(cold_host(
        &mut f,
        TableAction::StartSession {
            id: session,
            name: "A later session with the new character".into(),
            participants: vec![SessionParticipant {
                player_id: owner,
                character_id: Some(character),
                attendance: AttendanceStatus::Present,
            }],
        },
    ))
    .await;
    let started = f.state().await;
    assert_eq!(started.encounter_history, before.encounter_history);
    assert_eq!(
        started
            .rules
            .as_ref()
            .unwrap()
            .tactical_effects
            .as_ref()
            .unwrap()
            .effects,
        vec![armor]
    );
    assert_eq!(
        started
            .rules
            .as_ref()
            .unwrap()
            .tactical_creatures
            .as_ref()
            .unwrap()
            .runtime(mage)
            .unwrap()
            .controller,
        CreatureController::Player(owner)
    );
    let another = create_action(&f, CharacterId::new(), EntityId::new(), current).await;
    let active_finished = host_request(&f, another).await;
    Box::pin(f.reject(active_finished)).await;
    let rows = all_rows(&f.pool).await;
    assert_eq!(
        Box::pin(f.runtime.submit_presented_table(request))
            .await
            .unwrap(),
        accepted
    );
    assert_eq!(all_rows(&f.pool).await, rows);
    f.close().await;
}

#[tokio::test]
async fn unfinished_creation_refusal_preserves_real_pending_grapple_and_controller_resume() {
    let mut f = Box::pin(Fixture::new()).await;
    Box::pin(f.activate()).await;
    let start = f
        .choose(f.pc(0), "Grapple Small armored figure with left hand")
        .await;
    Box::pin(f.cold(start)).await;
    let save = f
        .choose(TableTransportChannel::Host, "Resist Grapple with Strength")
        .await;
    Box::pin(f.cold(save)).await;
    let pending = f.state().await.rules.unwrap().pending.unwrap();
    for current in [false, true] {
        let action = create_action(&f, CharacterId::new(), EntityId::new(), current).await;
        let request = host_request(&f, action).await;
        Box::pin(f.reject(request)).await;
        assert_eq!(f.state().await.rules.unwrap().pending.unwrap(), pending);
    }
    let end = host_request(&f, TableAction::EndSession).await;
    Box::pin(f.reject(end)).await;
    Box::pin(f.cold_roll(TableTransportChannel::Host, 1)).await;
    let finish = f.choose(f.pc(0), "Finish without changing equipment").await;
    Box::pin(f.cold(finish)).await;
    let after = f.state().await;
    let grip = &after
        .rules
        .as_ref()
        .unwrap()
        .tactical_grapples
        .as_ref()
        .unwrap()
        .active[0];
    assert_eq!(grip.declaration.grappler, f.actors[0]);
    assert_eq!(grip.declaration.target, f.goblin);
    assert!(
        after
            .rules
            .as_ref()
            .unwrap()
            .rolls
            .iter()
            .any(|roll| roll.request.id == pending.request.id)
    );
    f.close().await;
}
