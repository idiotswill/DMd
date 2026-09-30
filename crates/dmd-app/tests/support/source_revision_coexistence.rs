//! Authored, uncompiled integration draft. Receiver: reconciled release flow5 + MR.
//! This is a child of legacy_shield_hit_v1_replay; the original helpers and
//! genuine capture bytes are unchanged. No synthetic positive state is accepted.
use super::*;
use dmd_rules::tactical_creatures::{
    creature_definition, creature_source_pin, source_for_actor, source_for_profile,
};
use dmd_rules::tactical_definitions::{MonsterTrait, bundled_night_hag_v2};
use dmd_rules::tactical_spells::validate_creature_spell_source;
use std::collections::BTreeMap;

fn creation_input(creation: TableCreatureCreation) -> TableTransportInput {
    TableTransportInput::Action(Box::new(TableAction::CreateCreature {
        creation: Box::new(creation),
    }))
}

async fn accept(
    f: &mut Fixture,
    channel: TableTransportChannel,
    input: TableTransportInput,
) -> TableTransportRequest {
    let view = f.view(&channel).await;
    let request = f.request(channel, &view, input);
    Box::pin(f.accept_cold(request.clone())).await;
    request
}

async fn host(f: &mut Fixture, input: TableTransportInput) -> TableTransportRequest {
    Box::pin(accept(f, TableTransportChannel::Host, input)).await
}

async fn reject_host(f: &Fixture, input: TableTransportInput) {
    let view = f.view(&TableTransportChannel::Host).await;
    Box::pin(f.reject(f.request(TableTransportChannel::Host, &view, input))).await;
}

async fn current_creation(f: &Fixture) -> TableCreatureCreation {
    let view = f.view(&TableTransportChannel::Host).await;
    let options = f
        .app
        .table_creature_options(TableCreatureOptionsRequest {
            campaign_id: f.campaign,
            channel: TableTransportChannel::Host,
            revision: view.revision,
        })
        .await
        .unwrap();
    let hags: Vec<_> = options
        .iter()
        .filter(|option| option.definition_id == "night-hag")
        .collect();
    assert_eq!(
        hags.len(),
        1,
        "current admission exposes exactly one Hag revision"
    );
    let option = hags[0];
    let pin = creature_source_pin(bundled_night_hag_v2().unwrap()).unwrap();
    assert_eq!(option.source.as_ref(), Some(&pin));
    assert_eq!(option.sizes, [CreatureSize::Medium]);
    assert_eq!(option.additional_languages, 0);
    assert!(!option.ammunition_required);
    assert_eq!(
        option.item_count, 0,
        "no invented Soul Bag, focus, or components"
    );
    TableCreatureCreation {
        entity_id: EntityId::new(),
        name: "Private corrected Hag revision".into(),
        definition_id: option.definition_id.clone(),
        source: option.source.clone(),
        size: CreatureSize::Medium,
        additional_languages: vec![],
        ammunition_units: 0,
        item_ids: vec![],
    }
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

fn assert_source_pair(state: &CampaignState, old: &CreatureProfile, new: &CreatureProfile) {
    assert_ne!(old.actor, new.actor);
    assert_ne!(old.origin.id, new.origin.id);
    assert_eq!(old.origin.campaign_id, new.origin.campaign_id);
    assert_eq!(
        profile(state, old.actor),
        old,
        "release/control/play never rewrite the old profile"
    );
    assert_eq!(
        profile(state, new.actor),
        new,
        "current admission remains the exact created profile"
    );
    let old_source = creature_definition("night-hag").unwrap();
    let new_source = bundled_night_hag_v2().unwrap();
    assert_eq!(
        old.source,
        CreatureSourcePin {
            ruleset_id: "srd-5.2".into(),
            ruleset_version: "5.2.1".into(),
            definition_id: "night-hag".into(),
            definition_fingerprint: "90405f1404e04648".into(),
        }
    );
    assert_eq!(old.source, creature_source_pin(old_source).unwrap());
    assert_eq!(new.source, creature_source_pin(new_source).unwrap());
    assert_ne!(old.source, new.source);
    assert_eq!(
        source_for_profile(profile(state, old.actor)).unwrap(),
        old_source
    );
    assert_eq!(
        source_for_profile(profile(state, new.actor)).unwrap(),
        new_source
    );
    assert_eq!(
        source_for_actor(state, old.actor, "night-hag").unwrap(),
        old_source
    );
    assert_eq!(
        source_for_actor(state, new.actor, "night-hag").unwrap(),
        new_source
    );
    assert!(!old_source.traits.contains(&MonsterTrait::MagicResistance));
    assert_eq!(new_source.traits, [MonsterTrait::MagicResistance]);
    for source in [old_source, new_source] {
        assert_eq!(
            source.statistics.saving_throw_modifiers[Ability::Constitution.index()],
            3
        );
    }
    let creatures = state
        .rules
        .as_ref()
        .unwrap()
        .tactical_creatures
        .as_ref()
        .unwrap();
    assert_eq!(
        creatures
            .profiles
            .iter()
            .filter(|p| p.source.definition_id == "night-hag")
            .count(),
        2
    );
}

// The original Hag was created by a genuine typed table command, before the
// fixture's opaque transport bindings. Retry that exact historical body as well.
async fn retry_original_hag_creation(f: &Fixture, old: &CreatureProfile) {
    let event = f
        .original
        .event_journal
        .iter()
        .find(|event| event.command_id == old.origin.id.0.to_string())
        .unwrap();
    let saved: TableEvent = serde_json::from_str(&event.payload_json).unwrap();
    let TableAction::CreateCreature { creation } = &saved.action else {
        panic!("the original profile must name an accepted CreateCreature event");
    };
    assert_eq!(creation.entity_id, old.actor);
    assert!(
        creation.source.is_none(),
        "retain the original absent-pin historical body"
    );
    assert_eq!(saved.meta, old.origin);
    let before = export_campaign(&f.pool, f.campaign).await.unwrap();
    let receipt = Box::pin(f.app.execute_table(saved.meta, saved.action))
        .await
        .unwrap();
    assert!(receipt.already_accepted);
    assert_eq!(receipt.command_id, old.origin.id);
    assert_eq!(
        receipt.event_sequence,
        u64::try_from(event.sequence).unwrap()
    );
    assert_eq!(receipt.outcome, saved.outcome);
    f.assert_export(&before).await;
}

async fn retry_created(f: &Fixture, request: &TableTransportRequest) {
    let before = export_campaign(&f.pool, f.campaign).await.unwrap();
    let binding = before
        .table_transport_bindings
        .iter()
        .find(|binding| binding.meta.id == request.command_id)
        .unwrap();
    assert_eq!(
        serde_json::to_string(request).unwrap(),
        binding.request_json
    );
    let expected: TableTransportResult = serde_json::from_str(&binding.response_json).unwrap();
    assert_eq!(
        Box::pin(f.app.submit_presented_table(request.clone()))
            .await
            .unwrap(),
        expected
    );
    f.assert_export(&before).await;
}

async fn destination_rows(pool: &sqlx::SqlitePool) -> BTreeMap<String, i64> {
    let names: Vec<String> =
        sqlx::query_scalar("SELECT name FROM sqlite_master WHERE type = 'table' ORDER BY name")
            .fetch_all(pool)
            .await
            .unwrap();
    let mut rows = BTreeMap::new();
    for name in names {
        let quoted = name.replace('"', "\"\"");
        let count: i64 = sqlx::query_scalar(&format!("SELECT COUNT(*) FROM \"{quoted}\""))
            .fetch_one(pool)
            .await
            .unwrap();
        rows.insert(name, count);
    }
    rows
}

// The extra snapshot is an exact earlier accepted current-state row. A positive
// restore proves that retaining it is legal before any adversarial copy changes.
async fn reject_revision_swaps(
    f: &Fixture,
    old: &CreatureProfile,
    new: &CreatureProfile,
    earlier: &dmd_persistence::CurrentStateRow,
) {
    let original = export_campaign(&f.pool, f.campaign).await.unwrap();
    assert!(earlier.applied_event_sequence < original.current_state.applied_event_sequence);
    let earlier_state = Box::new(CampaignState::decode_json(&earlier.state_json).unwrap());
    assert_eq!(profile(&earlier_state, old.actor), old);
    assert!(!earlier_state.entities.contains_key(&new.actor));
    let mut genuine = original.clone();
    if let Some(snapshot) = genuine
        .snapshots
        .iter()
        .find(|row| row.event_sequence == earlier.applied_event_sequence)
    {
        assert_eq!(snapshot.campaign_id, earlier.campaign_id);
        assert_eq!(snapshot.state_schema_version, earlier.schema_version);
        assert_eq!(snapshot.state_json, earlier.state_json);
    } else {
        genuine.snapshots.push(dmd_persistence::SnapshotRow {
            campaign_id: earlier.campaign_id.clone(),
            event_sequence: earlier.applied_event_sequence,
            state_schema_version: earlier.schema_version,
            state_json: earlier.state_json.clone(),
            created_at_utc: original.exported_at_utc.clone(),
        });
        genuine.snapshots.sort_by_key(|row| row.event_sequence);
    }
    for snapshot in &original.snapshots {
        assert!(
            genuine.snapshots.contains(snapshot),
            "never replace an original snapshot"
        );
    }
    let positive_path = f.directory.join("genuine-earlier-snapshot.sqlite");
    let positive_pool = open_sqlite_path(&positive_path).await.unwrap();
    let positive_app = runtime(positive_pool.clone());
    assert_eq!(
        Box::pin(positive_app.restore_campaign(&genuine))
            .await
            .unwrap()
            .state(),
        image(&original).as_ref()
    );
    let mut positive_export = export_campaign(&positive_pool, f.campaign).await.unwrap();
    positive_export
        .exported_at_utc
        .clone_from(&genuine.exported_at_utc);
    assert_eq!(positive_export, genuine);
    positive_pool.close().await;
    drop(positive_app);
    drop(positive_pool);

    // Both current profiles and a real earlier snapshot are independently
    // authenticated. The known-valid wrong pin changes no intrinsic statistic.
    for (retired, actor, wrong_pin) in [
        (false, old.actor, new.source.clone()),
        (false, new.actor, old.source.clone()),
        (true, old.actor, new.source.clone()),
    ] {
        let mut forged = genuine.clone();
        let json = if retired {
            &mut forged
                .snapshots
                .iter_mut()
                .find(|row| row.event_sequence == earlier.applied_event_sequence)
                .unwrap()
                .state_json
        } else {
            &mut forged.current_state.state_json
        };
        let mut changed = Box::new(CampaignState::decode_json(json).unwrap());
        changed
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
            .source = wrong_pin;
        source_for_profile(profile(&changed, actor)).unwrap();
        dmd_rules::tactical_creatures::validate_creature_profile(
            &changed,
            profile(&changed, actor),
            &changed.rules.as_ref().unwrap().entities[&actor],
        )
        .unwrap();
        *json = changed.encode_json().unwrap();
        assert_eq!(forged.event_journal, genuine.event_journal);
        assert_eq!(forged.command_audit, genuine.command_audit);
        assert_eq!(
            forged.table_projection_history,
            genuine.table_projection_history
        );
        assert_eq!(
            forged.table_transport_bindings,
            genuine.table_transport_bindings
        );
        if retired {
            assert_eq!(forged.current_state, genuine.current_state);
        } else {
            assert_eq!(forged.snapshots, genuine.snapshots);
        }
        let path = f
            .directory
            .join(format!("hostile-{retired}-{}.sqlite", actor.0));
        let pool = open_sqlite_path(&path).await.unwrap();
        let app = runtime(pool.clone());
        let before = destination_rows(&pool).await;
        assert!(Box::pin(app.restore_campaign(&forged)).await.is_err());
        assert_eq!(
            destination_rows(&pool).await,
            before,
            "fresh hostile restore is atomic across every table"
        );
        assert_eq!(
            Box::pin(app.restore_campaign(&genuine))
                .await
                .unwrap()
                .state(),
            image(&original).as_ref(),
            "genuine history still restores into the same rejected destination"
        );
        pool.close().await;
        drop(app);
        drop(pool);
    }
    f.assert_export(&original).await;
}

async fn complete_original_missiles(
    f: &mut Fixture,
    old: &CreatureProfile,
    target: EntityId,
) -> SpellSourcePin {
    let initial = f.state().await;
    let cast = &flow(&initial).resolution.as_ref().unwrap().casts[0];
    let origin = cast.cast.plan.origin.id;
    let original_spell = cast.cast.plan.program.source.clone();
    assert_eq!(
        original_spell.creature_definition_id.as_deref(),
        Some("night-hag")
    );
    assert_eq!(original_spell.feature_id.as_deref(), Some("spellcasting"));
    assert_eq!(original_spell.spell_id, "magic-missile");
    assert_eq!(cast.cast.plan.program.spell_level, 4);
    assert_eq!(cast.completed.len(), 2);
    assert_eq!(cast.targets.len(), 6);
    assert!(cast.targets.iter().all(|t| t.actor == target));
    validate_creature_spell_source(old, &original_spell).unwrap();
    let mut ids: HashSet<_> = initial
        .rules
        .as_ref()
        .unwrap()
        .rolls
        .iter()
        .filter(|roll| {
            matches!(&roll.purpose, PendingPurpose::TacticalResolution { key, .. }
            if key.origin == origin && key.role == TacticalRollRole::SpellAmount)
        })
        .map(|roll| {
            assert_eq!(roll.result.dice, [DieResult { sides: 4, value: 1 }]);
            roll.request.id
        })
        .collect();
    assert_eq!(ids.len(), 2);
    for dart in 2..6 {
        let before = f.state().await;
        assert_eq!(flow(&before).version, 3);
        let cast = &flow(&before).resolution.as_ref().unwrap().casts[0];
        assert_eq!(cast.cast.plan.origin.id, origin);
        assert_eq!(cast.cast.plan.program.source, original_spell);
        assert_eq!(cast.completed.len(), dart);
        assert_eq!(
            before.rules.as_ref().unwrap().entities[&target].hp,
            19 - 2 * dart as u32
        );
        let pending = before.rules.as_ref().unwrap().pending.as_ref().unwrap();
        assert_eq!(pending.request.visibility, RollVisibility::Secret);
        assert_eq!(pending.issued_by.issuer, CommandIssuer::Admin);
        assert!(ids.insert(pending.request.id));
        assert!(
            matches!(&pending.purpose, PendingPurpose::TacticalResolution { key, .. }
            if key.origin == origin && key.role == TacticalRollRole::SpellAmount && key.subject == target)
        );
        let view = f.view(&TableTransportChannel::Host).await;
        assert_eq!(
            view.roll.as_ref().unwrap().dice,
            [DieSpec { count: 1, sides: 4 }]
        );
        assert_eq!(view.roll.as_ref().unwrap().modifier, 1);
        Box::pin(host(f, faces(&view, &[1]))).await;
        let after = f.state().await;
        let raw = after.rules.as_ref().unwrap().rolls.last().unwrap();
        assert_eq!(raw.request.id, pending.request.id);
        assert_eq!(raw.issued_by, pending.issued_by);
        assert_eq!(raw.result.dice, [DieResult { sides: 4, value: 1 }]);
        assert_eq!(
            after.rules.as_ref().unwrap().entities[&target].hp,
            17 - 2 * dart as u32
        );
    }
    let after = f.state().await;
    assert_eq!(ids.len(), 6);
    assert_eq!(flow(&after).version, 3);
    assert!(flow(&after).resolution.is_none());
    assert!(after.rules.as_ref().unwrap().pending.is_none());
    assert_eq!(after.rules.as_ref().unwrap().entities[&target].hp, 7);
    assert_eq!(profile(&after, old.actor), old);
    original_spell
}

async fn enable_and_assign_hags(f: &mut Fixture, actors: [EntityId; 2]) {
    let view = f.view(&TableTransportChannel::Host).await;
    assert!(view.source_control.is_none());
    let options = f
        .app
        .table_source_control_options(TableCreatureOptionsRequest {
            campaign_id: f.campaign,
            channel: TableTransportChannel::Host,
            revision: view.revision,
        })
        .await
        .unwrap();
    assert!(!options.enabled && options.settled && options.adopted.is_empty());
    let mut request = f.request(
        TableTransportChannel::Host,
        &view,
        TableTransportInput::Action(Box::new(TableAction::EnableSourceActorAccess {
            adopted: options.adopted,
        })),
    );
    request.version = TABLE_SOURCE_TRANSPORT_VERSION;
    Box::pin(f.accept_cold(request)).await;
    for actor in actors {
        Box::pin(host(
            f,
            TableTransportInput::Action(Box::new(TableAction::SetSourceCreatureController {
                actor,
                controller: CreatureController::Host,
            })),
        ))
        .await;
    }
}

async fn second_scene(
    f: &mut Fixture,
    old: &CreatureProfile,
    new: &CreatureProfile,
    horse: EntityId,
) {
    let finished = f.state().await;
    assert_eq!(flow(&finished).phase, TacticalPhase::Finished);
    let released = finished
        .encounter_history
        .as_ref()
        .unwrap()
        .last()
        .unwrap()
        .clone();
    let old_encounter = finished.encounter.as_ref().unwrap();
    let pc = finished
        .characters
        .values()
        .find(|pc| old_encounter.participant(pc.entity_id).is_some())
        .unwrap();
    let pc_actor = pc.entity_id;
    let pc_id = pc.id;
    let point = |x, y| SpatialPoint { x, y, z: 0 };
    let setup = TableBattlefieldSetup {
        encounter_id: EncounterId::new(),
        scene_id: SceneId::new(),
        location_id: finished.scenes[&old_encounter.scene_id].location_id,
        name: "A later encounter with both retained Hag revisions".into(),
        battlefield: Battlefield {
            bounds: SpatialBox {
                min: point(0, 0),
                max: SpatialPoint {
                    x: 180,
                    y: 100,
                    z: 60,
                },
            },
            floor_z: 0,
            floor_surface: "stone".into(),
            ambient_light: LightLevel::Bright,
            terrain: vec![],
            obstacles: vec![],
            lights: vec![],
        },
        characters: vec![TableCharacterPlacement {
            character_id: pc_id,
            position: point(10, 10),
            height: 12,
            allies: vec![],
            enemies: vec![],
        }],
        creatures: vec![
            TableCreaturePlacement {
                actor: old.actor,
                public_label: "Earlier horned figure".into(),
                position: point(40, 30),
                height: 12,
                allies: vec![],
                enemies: vec![new.actor],
            },
            TableCreaturePlacement {
                actor: new.actor,
                public_label: "Later horned figure".into(),
                position: point(80, 30),
                height: 12,
                allies: vec![],
                enemies: vec![old.actor],
            },
            TableCreaturePlacement {
                actor: horse,
                public_label: "Horse".into(),
                position: point(130, 30),
                height: 16,
                allies: vec![],
                enemies: vec![],
            },
        ],
        geometry_ruling: Ruling {
            basis: RulingBasis::GmAdjudication,
            reason: "New explicit separated bodies on a bright open floor in the same location."
                .into(),
        },
        area_grid_policy: None,
    };
    let encounter_id = setup.encounter_id;
    let scene_id = setup.scene_id;
    Box::pin(host(
        f,
        TableTransportInput::Action(Box::new(TableAction::PrepareBattlefield {
            setup: Box::new(setup),
        })),
    ))
    .await;
    let combatants = [
        (
            old.actor,
            TacticalSource::Creature {
                definition_id: "night-hag".into(),
            },
        ),
        (
            new.actor,
            TacticalSource::Creature {
                definition_id: "night-hag".into(),
            },
        ),
        (
            horse,
            TacticalSource::Creature {
                definition_id: "warhorse".into(),
            },
        ),
        (pc_actor, TacticalSource::Character),
    ]
    .into_iter()
    .map(|(actor, source)| TacticalCombatant {
        actor,
        source,
        surprised: false,
    })
    .collect::<Vec<_>>();
    let prepared = f.state().await;
    for combatant in &combatants[..2] {
        assert_eq!(
            dmd_rules::tactical::preview_initiative_circumstances(&prepared, combatant).unwrap(),
            (5, RollMode::Normal),
            "each exact profile independently derives the shared initiative circumstances"
        );
    }
    // Current initiative grouping uses source ID plus actual circumstances.
    // Both source-faithful Hags have +5; roll their one genuine group, then
    // submit the required Host tie choice. Do not invent surprise to split it.
    let groups = vec![
        InitiativeGroup {
            actors: vec![old.actor, new.actor],
            request_id: RollRequestId::new(),
        },
        InitiativeGroup {
            actors: vec![horse],
            request_id: RollRequestId::new(),
        },
        InitiativeGroup {
            actors: vec![pc_actor],
            request_id: RollRequestId::new(),
        },
    ];
    Box::pin(host(
        f,
        tactical(TacticalAction::Begin {
            execution: TacticalExecutionVersion::EncounterReleaseV1,
            combatants,
            groups,
        }),
    ))
    .await;
    for (actor, value) in [(old.actor, 20), (horse, 8), (pc_actor, 1)] {
        let state = f.state().await;
        let channel = controller(&state, actor);
        let view = f.view(&channel).await;
        assert_eq!(view.roll.as_ref().unwrap().roller, Some(actor));
        assert_eq!(
            view.roll.as_ref().unwrap().dice,
            [DieSpec {
                count: 1,
                sides: 20
            }]
        );
        Box::pin(accept(f, channel, faces(&view, &[value]))).await;
    }
    let tied = f.state().await;
    assert!(
        matches!(&flow(&tied).phase, TacticalPhase::InitiativeTies { ties }
        if ties.len() == 1 && ties[0].actors.len() == 2
            && ties[0].actors.contains(&old.actor) && ties[0].actors.contains(&new.actor))
    );
    Box::pin(host(
        f,
        tactical(TacticalAction::ProposeInitiativeTie {
            order: vec![old.actor, new.actor],
        }),
    ))
    .await;
    let active = f.state().await;
    assert_eq!(active.encounter.as_ref().unwrap().id, encounter_id);
    assert_eq!(active.encounter.as_ref().unwrap().scene_id, scene_id);
    assert_eq!(
        active.scenes[&released.scene_id].status,
        SceneStatus::Closed
    );
    assert_eq!(flow(&active).version, 5);
    assert_eq!(flow(&active).phase, TacticalPhase::Active);
    assert_eq!(
        active
            .rules
            .as_ref()
            .unwrap()
            .timing
            .as_ref()
            .unwrap()
            .turn_number,
        released.final_turn.number + 1
    );
    assert_eq!(
        f.view(&TableTransportChannel::Host)
            .await
            .tactical
            .unwrap()
            .active_actor,
        Some(old.actor)
    );
    assert_eq!(
        active.rules.as_ref().unwrap().entities[&horse].hp,
        7,
        "replacement never heals an old participant"
    );
    assert_source_pair(&active, old, new);
}

// Real source feature selection, response ordering, acknowledgement, six raw
// amounts and five explicit impact choices (the last sole dart drains itself).
async fn cast_six_darts(
    f: &mut Fixture,
    caster: &CreatureProfile,
    target: &CreatureProfile,
) -> SpellSourcePin {
    let before = f.state().await;
    let hp = before.rules.as_ref().unwrap().entities[&target.actor].hp;
    let view = f.view(&TableTransportChannel::Host).await;
    assert_eq!(
        view.tactical.as_ref().unwrap().active_actor,
        Some(caster.actor)
    );
    let options = view
        .tactical
        .as_ref()
        .unwrap()
        .casting_options
        .as_ref()
        .unwrap();
    assert_eq!(options.actor, caster.actor);
    assert_eq!(options.variants.len(), 1);
    let variant = &options.variants[0];
    assert_eq!(variant.choice.spell_id, "magic-missile");
    assert_eq!(variant.choice.resource, SpellResourceChoice::SourceFeature);
    assert_eq!(
        variant.choice.grant,
        SpellGrantChoice::CreatureFeature {
            feature_id: "spellcasting".into()
        }
    );
    assert_eq!(variant.choice.material, SpellMaterialChoice::None);
    assert!(variant.targets.iter().any(|t| t.actor == target.actor));
    let paid = Box::pin(host(
        f,
        tactical(TacticalAction::CastSpell {
            choice: variant.choice.clone(),
            targets: SpellTargetChoice::Entities(vec![target.actor; 6]),
        }),
    ))
    .await;
    let committed = f.state().await;
    let cast = &flow(&committed).resolution.as_ref().unwrap().casts[0];
    let pin = cast.cast.plan.program.source.clone();
    validate_creature_spell_source(caster, &pin).unwrap();
    assert!(
        validate_creature_spell_source(target, &pin).is_err(),
        "same definition ID does not authorize another full source tuple"
    );
    assert_eq!(pin.ruleset_id, caster.source.ruleset_id);
    assert_eq!(pin.ruleset_version, caster.source.ruleset_version);
    assert_eq!(pin.creature_definition_id.as_deref(), Some("night-hag"));
    assert_eq!(pin.feature_id.as_deref(), Some("spellcasting"));
    assert_eq!(pin.spell_id, "magic-missile");
    assert_eq!(cast.cast.plan.program.spell_level, 4);
    assert_eq!(cast.cast.plan.origin.id, paid.command_id);
    assert_eq!(cast.cast.plan.choice.actor, caster.actor);
    assert_eq!(cast.targets.len(), 6);
    assert!(cast.targets.iter().all(|t| t.actor == target.actor));
    let activation = cast.creature_activation.as_ref().unwrap();
    assert_eq!(activation.origin.id, paid.command_id);
    assert_eq!(activation.activation, SpellEnclosingActivation::Action);
    assert!(!activation.attack_action);
    assert!(
        committed
            .rules
            .as_ref()
            .unwrap()
            .timing
            .as_ref()
            .unwrap()
            .action_spent
    );
    let missile = f
        .view(&TableTransportChannel::Host)
        .await
        .tactical
        .unwrap()
        .missile
        .unwrap();
    assert_eq!(missile.responses.len(), 1);
    assert_eq!(missile.responses[0].actor, target.actor);
    assert!(missile.responses[0].shield.is_empty());
    Box::pin(host(
        f,
        TableTransportInput::MissileResponse {
            handle: missile.order.unwrap().key,
            decision: Box::new(TableMissileInput::Order {
                instruction: TacticalReactionOrdering {
                    ranked: vec![],
                    unlisted: ReactionUnlistedOrder::AfterReverse,
                },
            }),
        },
    ))
    .await;
    let response = f
        .view(&TableTransportChannel::Host)
        .await
        .tactical
        .unwrap()
        .missile
        .unwrap()
        .responses
        .remove(0);
    Box::pin(host(
        f,
        TableTransportInput::MissileResponse {
            handle: response.key,
            decision: Box::new(TableMissileInput::Respond { accept: false }),
        },
    ))
    .await;
    let mut ids = HashSet::new();
    for _ in 0..6 {
        let state = f.state().await;
        let pending = state.rules.as_ref().unwrap().pending.as_ref().unwrap();
        assert!(
            matches!(&pending.purpose, PendingPurpose::TacticalResolution { key, .. }
            if key.origin == paid.command_id && key.role == TacticalRollRole::SpellAmount && key.subject == target.actor)
        );
        assert_eq!(
            flow(&state).resolution.as_ref().unwrap().casts[0]
                .cast
                .plan
                .program
                .source,
            pin
        );
        assert_eq!(
            state.rules.as_ref().unwrap().entities[&target.actor].hp,
            hp,
            "all amounts precede impacts in flow5"
        );
        let view = f.view(&TableTransportChannel::Host).await;
        let roll = view.roll.as_ref().unwrap();
        assert!(ids.insert(pending.request.id));
        assert_eq!(roll.dice, [DieSpec { count: 1, sides: 4 }]);
        assert_eq!(roll.modifier, 1);
        Box::pin(host(f, faces(&view, &[1]))).await;
        let after = f.state().await;
        let raw = after.rules.as_ref().unwrap().rolls.last().unwrap();
        assert_eq!(raw.request.id, pending.request.id);
        assert_eq!(raw.issued_by, pending.issued_by);
        assert_eq!(raw.result.dice, [DieResult { sides: 4, value: 1 }]);
    }
    assert_eq!(ids.len(), 6);
    assert_eq!(
        f.state().await.rules.as_ref().unwrap().entities[&target.actor].hp,
        hp
    );
    assert!(f.view(&TableTransportChannel::Host).await.roll.is_none());
    for remaining in (2..=6).rev() {
        let view = f.view(&TableTransportChannel::Host).await;
        let choices = &view
            .tactical
            .as_ref()
            .unwrap()
            .continuation
            .as_ref()
            .unwrap()
            .choices;
        assert_eq!(choices.len(), remaining);
        assert!(
            choices
                .iter()
                .all(|choice| choice.label.starts_with("Dart "))
        );
        let before_hp = f.state().await.rules.as_ref().unwrap().entities[&target.actor].hp;
        Box::pin(host(
            f,
            TableTransportInput::SelectWork {
                handle: choices.last().unwrap().handle,
            },
        ))
        .await;
        assert_eq!(
            f.state().await.rules.as_ref().unwrap().entities[&target.actor].hp,
            before_hp - if remaining == 2 { 4 } else { 2 }
        );
    }
    let after = f.state().await;
    assert_eq!(flow(&after).version, 5);
    assert!(flow(&after).resolution.is_none());
    let rules = after.rules.as_ref().unwrap();
    assert!(rules.pending.is_none());
    assert_eq!(rules.entities[&target.actor].hp, hp - 12);
    assert_eq!(
        rules.entities[&caster.actor].hp,
        before.rules.as_ref().unwrap().entities[&caster.actor].hp
    );
    assert!(rules.timing.as_ref().unwrap().action_spent);
    assert!(!rules.timing.as_ref().unwrap().slot_spent_this_turn);
    assert!(rules.entities[&caster.actor].resources.is_empty());
    assert!(rules.entities[&caster.actor].prepared_spells.is_empty());
    assert!(rules.entities[&caster.actor].spellcasting.is_none());
    assert!(
        rules
            .tactical_creatures
            .as_ref()
            .unwrap()
            .runtime(caster.actor)
            .unwrap()
            .limited_uses
            .is_empty(),
        "at-will source use has no fabricated slot or usage pool"
    );
    assert_eq!(profile(&after, caster.actor), caster);
    assert_eq!(profile(&after, target.actor), target);
    pin
}

async fn cold_final_restore_and_receipts(
    f: &Fixture,
    old: &CreatureProfile,
    created: &TableTransportRequest,
) {
    let accepted = export_campaign(&f.pool, f.campaign).await.unwrap();
    f.assert_prefix(&accepted);
    for snapshot in &f.original.snapshots {
        assert!(
            accepted.snapshots.contains(snapshot),
            "all original snapshot rows remain byte-for-byte retained"
        );
    }
    for cause in &f.original.event_causes {
        assert!(accepted.event_causes.contains(cause));
    }
    let path = f.directory.join("coexistence-final.sqlite");
    let pool = open_sqlite_path(&path).await.unwrap();
    let app = runtime(pool.clone());
    Box::pin(app.restore_campaign(&accepted)).await.unwrap();
    pool.close().await;
    drop(app);
    drop(pool);
    let pool = open_sqlite_path(&path).await.unwrap();
    let app = runtime(pool.clone());
    assert_eq!(
        Box::pin(app.resume_campaign(f.campaign))
            .await
            .unwrap()
            .state(),
        image(&accepted).as_ref()
    );
    for binding in &accepted.table_transport_bindings {
        let request: TableTransportRequest = serde_json::from_str(&binding.request_json).unwrap();
        let expected: TableTransportResult = serde_json::from_str(&binding.response_json).unwrap();
        assert_eq!(
            Box::pin(app.submit_presented_table(request)).await.unwrap(),
            expected
        );
    }
    let original = f
        .original
        .event_journal
        .iter()
        .find(|event| event.command_id == old.origin.id.0.to_string())
        .unwrap();
    let saved: TableEvent = serde_json::from_str(&original.payload_json).unwrap();
    let receipt = Box::pin(app.execute_table(saved.meta, saved.action))
        .await
        .unwrap();
    assert!(receipt.already_accepted);
    assert_eq!(receipt.command_id, old.origin.id);
    assert_eq!(
        receipt.event_sequence,
        u64::try_from(original.sequence).unwrap()
    );
    assert_eq!(receipt.outcome, saved.outcome);
    let mut changed = created.clone();
    changed.input = tactical(TacticalAction::Dodge);
    assert!(Box::pin(app.submit_presented_table(changed)).await.is_err());
    let mut actual = export_campaign(&pool, f.campaign).await.unwrap();
    actual.exported_at_utc.clone_from(&accepted.exported_at_utc);
    assert_eq!(
        actual, accepted,
        "all old/new exact receipts are read-only after independent cold restore"
    );
    pool.close().await;
    drop(app);
    drop(pool);
}

#[tokio::test]
async fn genuine_old_hag_and_current_hag_coexist_after_authenticated_release() {
    let mut f = Box::pin(Fixture::restore(MISSILE_PARTIAL)).await;
    let initial = f.state().await;
    let cast = &flow(&initial).resolution.as_ref().unwrap().casts[0];
    let old = profile(&initial, cast.cast.plan.choice.actor).clone();
    let horse = cast.targets[0].actor;
    assert_eq!(initial.rules.as_ref().unwrap().entities[&horse].hp, 15);
    assert_eq!(initial.rules.as_ref().unwrap().entities[&old.actor].hp, 112);
    Box::pin(f.retry_original_bindings()).await;
    Box::pin(retry_original_hag_creation(&f, &old)).await;
    // A genuine current pin cannot bypass pending work or an unreleased encounter.
    let pending_creation = current_creation(&f).await;
    Box::pin(reject_host(&f, creation_input(pending_creation))).await;
    Box::pin(reject_host(
        &f,
        tactical(TacticalAction::UpgradeExecutionTo {
            execution: TacticalExecutionVersion::EncounterReleaseV1,
        }),
    ))
    .await;
    let original_spell = Box::pin(complete_original_missiles(&mut f, &old, horse)).await;
    let active_creation = current_creation(&f).await;
    Box::pin(reject_host(&f, creation_input(active_creation))).await;
    Box::pin(host(
        &mut f,
        tactical(TacticalAction::UpgradeExecutionTo {
            execution: TacticalExecutionVersion::EncounterReleaseV1,
        }),
    ))
    .await;
    Box::pin(host(
        &mut f,
        tactical(TacticalAction::ConcludeHostilities {
            cadence: AftermathCadence::ContinueExistingOrder,
            ruling: "Host concludes this conflict; no pending work or source timing is discarded."
                .into(),
        }),
    ))
    .await;
    assert!(
        f.view(&TableTransportChannel::Host)
            .await
            .tactical
            .unwrap()
            .release
            .unwrap()
            .may_finish
    );
    let before_release = f.state().await;
    assert_eq!(before_release.items, initial.items);
    let old_inventory = before_release
        .rules
        .as_ref()
        .unwrap()
        .tactical_inventory
        .as_ref()
        .unwrap();
    let old_loadout = old_inventory.loadout(old.actor).unwrap().clone();
    assert_eq!(
        Some(&old_loadout),
        initial
            .rules
            .as_ref()
            .unwrap()
            .tactical_inventory
            .as_ref()
            .unwrap()
            .loadout(old.actor)
    );
    let released = Box::pin(host(&mut f, tactical(TacticalAction::FinishEncounter))).await;
    let finished = f.state().await;
    assert_eq!(flow(&finished).phase, TacticalPhase::Finished);
    assert_eq!(flow(&finished).version, 5);
    assert!(finished.rules.as_ref().unwrap().timing.is_none());
    assert_eq!(profile(&finished, old.actor), &old);
    assert_eq!(finished.items, before_release.items);
    assert_eq!(
        finished.rules.as_ref().unwrap().tactical_inventory,
        before_release.rules.as_ref().unwrap().tactical_inventory
    );
    // LeaveCombat legitimately retires source observed-turn/window state; it
    // must not rewrite items, physical loadout identities, or their origins.
    let earlier = export_campaign(&f.pool, f.campaign)
        .await
        .unwrap()
        .current_state;
    assert_eq!(
        finished
            .encounter_history
            .as_ref()
            .unwrap()
            .last()
            .unwrap()
            .released_by
            .id,
        released.command_id
    );
    let creation = current_creation(&f).await;
    // Valid Finished authority does not permit missing, retired, or partial pins.
    let mut invalid = vec![None, Some(old.source.clone())];
    for lane in 0..4 {
        let mut pin = creation.source.clone().unwrap();
        match lane {
            0 => pin.ruleset_id.push_str("-forged"),
            1 => pin.ruleset_version.push_str("-forged"),
            2 => pin.definition_id = "warhorse".into(),
            _ => pin.definition_fingerprint = "0000000000000000".into(),
        }
        invalid.push(Some(pin));
    }
    for source in invalid {
        let mut wrong = creation.clone();
        wrong.entity_id = EntityId::new();
        wrong.source = source;
        Box::pin(reject_host(&f, creation_input(wrong))).await;
    }
    let attending = finished
        .characters
        .values()
        .find(|pc| {
            finished
                .encounter
                .as_ref()
                .unwrap()
                .participant(pc.entity_id)
                .is_some()
        })
        .unwrap();
    let player_channel = controller(&finished, attending.entity_id);
    assert!(matches!(
        player_channel,
        TableTransportChannel::Player { .. }
    ));
    let player_view = f.view(&player_channel).await;
    Box::pin(f.reject(f.request(
        player_channel,
        &player_view,
        creation_input(creation.clone()),
    )))
    .await;
    let created = Box::pin(host(&mut f, creation_input(creation.clone()))).await;
    let current = f.state().await;
    let new = profile(&current, creation.entity_id).clone();
    assert_eq!(
        new.origin,
        CommandMeta {
            id: created.command_id,
            campaign_id: f.campaign,
            session_id: Some(f.session),
            issuer: CommandIssuer::Admin,
            actor: None,
            expected_event_sequence: finished.applied_event_sequence,
        }
    );
    assert_eq!(new.source, creation.source.clone().unwrap());
    assert_eq!(new.size, creation.size);
    assert_eq!(new.additional_languages, creation.additional_languages);
    assert_eq!(new.hit_points, CreatureHitPointOrigin::Average);
    assert_eq!(current.rules.as_ref().unwrap().entities[&new.actor].hp, 112);
    assert!(
        !current
            .items
            .values()
            .any(|item| item.custody == Custody::Entity(new.actor))
    );
    assert_source_pair(&current, &old, &new);
    assert_eq!(
        current.items, before_release.items,
        "a zero-item Hag creation changes no item or custody"
    );
    let current_inventory = current
        .rules
        .as_ref()
        .unwrap()
        .tactical_inventory
        .as_ref()
        .unwrap();
    assert_eq!(current_inventory.loadout(old.actor), Some(&old_loadout));
    assert_eq!(current_inventory.receipts, old_inventory.receipts);
    for loadout in &old_inventory.loadouts {
        assert_eq!(current_inventory.loadout(loadout.actor), Some(loadout));
    }
    assert_eq!(
        current_inventory.loadouts.len(),
        old_inventory.loadouts.len() + 1
    );
    assert!(current_inventory.loadout(new.actor).is_some());
    assert!(validate_creature_spell_source(&new, &original_spell).is_err());
    // Use the actual unrelated absent player; this read creates no transport
    // revision, so it cannot mask a write in the retry/rejection comparisons.
    let outsider = initial
        .table
        .as_ref()
        .unwrap()
        .active_session
        .as_ref()
        .unwrap()
        .participants
        .iter()
        .find(|p| p.attendance == AttendanceStatus::Absent)
        .unwrap()
        .player_id;
    let before_privacy_query = export_campaign(&f.pool, f.campaign).await.unwrap();
    let private = serde_json::to_string(
        &f.app
            .table_view(f.campaign, TableViewer::Player(outsider))
            .await
            .unwrap(),
    )
    .unwrap();
    f.assert_export(&before_privacy_query).await;
    for secret in [
        creation.name.clone(),
        new.actor.0.to_string(),
        new.source.definition_fingerprint.clone(),
        "MagicResistance".into(),
        "magic-resistance".into(),
    ] {
        assert!(
            !private.contains(&secret),
            "private source admission leaked {secret}"
        );
    }
    Box::pin(retry_original_hag_creation(&f, &old)).await;
    Box::pin(retry_created(&f, &created)).await;
    Box::pin(reject_revision_swaps(&f, &old, &new, &earlier)).await;
    Box::pin(enable_and_assign_hags(&mut f, [old.actor, new.actor])).await;
    Box::pin(retry_original_hag_creation(&f, &old)).await;
    Box::pin(retry_created(&f, &created)).await;
    Box::pin(second_scene(&mut f, &old, &new, horse)).await;
    let old_cast = Box::pin(cast_six_darts(&mut f, &old, &new)).await;
    assert_eq!(
        old_cast, original_spell,
        "old revision preserves its complete original spell tuple"
    );
    Box::pin(host(&mut f, tactical(TacticalAction::EndTurn))).await;
    assert_eq!(
        f.view(&TableTransportChannel::Host)
            .await
            .tactical
            .unwrap()
            .active_actor,
        Some(new.actor)
    );
    let new_cast = Box::pin(cast_six_darts(&mut f, &new, &old)).await;
    assert_ne!(
        old_cast, new_cast,
        "same spell/feature IDs do not collapse source revisions"
    );
    let final_state = f.state().await;
    assert_source_pair(&final_state, &old, &new);
    assert_eq!(final_state.items, initial.items);
    assert_eq!(
        final_state
            .rules
            .as_ref()
            .unwrap()
            .tactical_inventory
            .as_ref()
            .unwrap()
            .loadout(old.actor),
        Some(&old_loadout)
    );
    assert_eq!(
        final_state.rules.as_ref().unwrap().entities[&old.actor].hp,
        100
    );
    assert_eq!(
        final_state.rules.as_ref().unwrap().entities[&new.actor].hp,
        100
    );
    assert_eq!(final_state.rules.as_ref().unwrap().entities[&horse].hp, 7);
    Box::pin(cold_final_restore_and_receipts(&f, &old, &created)).await;
    Box::pin(f.finish()).await;
}
