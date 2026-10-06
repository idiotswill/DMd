//! Two independent continuations of one genuine pre-breath export. Physical
//! damage determines survival; neither branch patches HP, effects or recharge.
use super::*;

struct Sources {
    mage: EntityId,
    dragon: EntityId,
    armor: TacticalEffect,
}

fn point(x: i32, y: i32, z: i32) -> SpatialPoint {
    SpatialPoint { x, y, z }
}

async fn accept(
    f: &mut Fixture,
    url: &str,
    directory: &Path,
    label: &str,
    player: Option<usize>,
    declaration: TableAction,
) -> (TableTransportRequest, TableTransportResult) {
    let request = request(f, player, declaration).await;
    let response = Box::pin(durable_step(f, url, directory, label, request.clone())).await;
    (request, response)
}

fn breath_recharge(state: &CampaignState, dragon: EntityId) -> &CreatureRechargeState {
    state
        .rules
        .as_ref()
        .unwrap()
        .tactical_creatures
        .as_ref()
        .unwrap()
        .runtime(dragon)
        .unwrap()
        .recharge
        .iter()
        .find(|row| row.feature_id == "fire-breath")
        .unwrap()
}

fn assert_armor(state: &CampaignState, sources: &Sources) {
    let effects = state
        .rules
        .as_ref()
        .unwrap()
        .tactical_effects
        .as_ref()
        .unwrap();
    assert_eq!(
        effects
            .effects
            .iter()
            .find(|effect| effect.id == sources.armor.id),
        Some(&sources.armor)
    );
    assert!(
        sources.armor.conditions.is_empty(),
        "Mage Armor has raw defense, not a condition projection"
    );
    assert!(sources.armor.concentration_group.is_none());
    assert!(
        matches!(sources.armor.expires, TacticalEffectExpiry::AtTime(at) if at > state.clock.now)
    );
}

async fn end_turn(
    f: &mut Fixture,
    url: &str,
    directory: &Path,
    label: &str,
    player: Option<usize>,
    dragon: EntityId,
) {
    Box::pin(accept(
        f,
        url,
        directory,
        label,
        player,
        action(TacticalAction::EndTurn),
    ))
    .await;
    // One explicit choice at the actual other-creature End. No generic queue drain.
    if let Some(actor) = view(f, None).await.tactical.unwrap().legendary_action {
        assert_eq!(actor, dragon);
        Box::pin(accept(
            f,
            url,
            directory,
            "declined-real-legendary-window",
            None,
            action(TacticalAction::DeclineLegendaryAction),
        ))
        .await;
    }
}

fn battlefield(
    f: &Fixture,
    mage: EntityId,
    dragon: EntityId,
    location: LocationId,
) -> TableBattlefieldSetup {
    TableBattlefieldSetup {
        encounter_id: EncounterId::new(),
        scene_id: SceneId::new(),
        location_id: location,
        name: "Open ground with one creature in the breath cone".into(),
        battlefield: Battlefield {
            bounds: SpatialBox {
                min: point(0, 0, 0),
                max: point(200, 140, 80),
            },
            floor_z: 0,
            floor_surface: "stone".into(),
            ambient_light: LightLevel::Bright,
            terrain: vec![],
            obstacles: vec![],
            lights: vec![],
        },
        characters: vec![TableCharacterPlacement {
            character_id: f.characters[0],
            position: point(10, 110, 0),
            height: 12,
            allies: vec![],
            enemies: vec![],
        }],
        creatures: vec![
            TableCreaturePlacement {
                actor: mage,
                public_label: "Robed spellcaster".into(),
                position: point(70, 50, 0),
                height: 12,
                allies: vec![],
                enemies: vec![],
            },
            TableCreaturePlacement {
                actor: dragon,
                public_label: "Huge red dragon".into(),
                position: point(10, 40, 0),
                height: 40,
                allies: vec![],
                enemies: vec![],
            },
        ],
        area_grid_policy: Some(TacticalAreaGridPolicy::OccupiedCellCentersV1),
        geometry_ruling: Ruling {
            basis: RulingBasis::GmAdjudication,
            reason: "Explicit occupied cells put only the Mage east of the dragon's breath origin."
                .into(),
        },
    }
}

fn begin(f: &Fixture, mage: EntityId, dragon: EntityId, player_first: bool) -> TableAction {
    let order = if player_first {
        [f.actors[0], dragon, mage]
    } else {
        [mage, dragon, f.actors[0]]
    };
    action(TacticalAction::Begin {
        execution: TacticalExecutionVersion::EncounterReleaseV1,
        combatants: order
            .into_iter()
            .map(|actor| TacticalCombatant {
                actor,
                surprised: false,
                source: if actor == mage {
                    TacticalSource::Creature {
                        definition_id: "mage".into(),
                    }
                } else if actor == dragon {
                    TacticalSource::Creature {
                        definition_id: "adult-red-dragon".into(),
                    }
                } else {
                    TacticalSource::Character
                },
            })
            .collect(),
        groups: order
            .into_iter()
            .map(|actor| InitiativeGroup {
                actors: vec![actor],
                request_id: RollRequestId::new(),
            })
            .collect(),
    })
}

async fn prepare_sources(f: &mut Fixture, url: &str, directory: &Path) -> Sources {
    let count = view(f, None)
        .await
        .characters
        .iter()
        .find(|row| row.character_id == f.characters[0])
        .unwrap()
        .equipment
        .as_ref()
        .unwrap()
        .initial_item_count;
    f.host(
        TableAction::PrepareEquipment {
            character_id: f.characters[0],
            item_ids: (0..count).map(|_| ItemId::new()).collect(),
        },
        Some(f.session),
    )
    .await;
    let mage = EntityId::new();
    let dragon = EntityId::new();
    for (actor, definition_id, size, additional_languages) in [
        (
            mage,
            "mage",
            CreatureSize::Medium,
            vec!["dwarvish".into(), "elvish".into(), "draconic".into()],
        ),
        (dragon, "adult-red-dragon", CreatureSize::Huge, vec![]),
    ] {
        let host = f
            .runtime
            .presented_table_view(f.campaign, TableViewer::Host)
            .await
            .unwrap();
        let options = f
            .runtime
            .table_creature_options(TableCreatureOptionsRequest {
                campaign_id: f.campaign,
                channel: TableTransportChannel::Host,
                revision: host.revision,
            })
            .await
            .unwrap();
        let source = options
            .iter()
            .find(|source| source.definition_id == definition_id)
            .unwrap();
        let allocation =
            dmd_rules::tactical_creature_equipment::creature_equipment_plan_from_source(
                source.source.as_ref().unwrap(),
                0,
            )
            .unwrap();
        f.host(
            TableAction::CreateCreature {
                creation: Box::new(TableCreatureCreation {
                    entity_id: actor,
                    name: format!("Private {definition_id}"),
                    definition_id: definition_id.into(),
                    source: source.source.clone(),
                    size,
                    additional_languages,
                    ammunition_units: 0,
                    item_ids: allocation.iter().map(|_| ItemId::new()).collect(),
                }),
            },
            Some(f.session),
        )
        .await;
    }
    let setup = battlefield(f, mage, dragon, LocationId::new());
    f.host(
        TableAction::PrepareBattlefield {
            setup: Box::new(setup),
        },
        Some(f.session),
    )
    .await;
    let initiative = begin(f, mage, dragon, false);
    Box::pin(accept(
        f,
        url,
        directory,
        "first-dragon-initiative",
        None,
        initiative,
    ))
    .await;
    // The dragon's source initiative modifier is +12; a physical 5 keeps its
    // total 17 below the Mage's 22 without inventing a tie-order decision.
    for (player, face) in [(None, 20), (None, 5), (Some(0), 2)] {
        Box::pin(setup_raw(f, player, &[face])).await;
    }
    assert_eq!(
        view(f, None).await.tactical.unwrap().active_actor,
        Some(mage)
    );
    let casting = view(f, None)
        .await
        .tactical
        .unwrap()
        .casting_options
        .unwrap();
    let choice = casting
        .variants
        .into_iter()
        .find(|variant| variant.choice.spell_id == "mage-armor")
        .unwrap()
        .choice;
    Box::pin(accept(
        f,
        url,
        directory,
        "real-mage-armor",
        None,
        action(TacticalAction::CastSpell {
            choice,
            targets: SpellTargetChoice::Entities(vec![mage]),
        }),
    ))
    .await;
    let armored = Box::new(state(f).await);
    assert_eq!(armored.rules.as_ref().unwrap().entities[&mage].hp, 81);
    let armor = armored
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
    assert_eq!(
        armor.expires,
        TacticalEffectExpiry::AtTime(WorldInstant(armored.clock.now.0 + 28_800))
    );
    Box::pin(end_turn(
        f,
        url,
        directory,
        "mage-end-before-breath",
        None,
        dragon,
    ))
    .await;
    let ready = Box::new(state(f).await);
    assert_eq!(
        view(f, None).await.tactical.unwrap().active_actor,
        Some(dragon)
    );
    assert_eq!(
        ready
            .rules
            .as_ref()
            .unwrap()
            .timing
            .as_ref()
            .unwrap()
            .turn_number,
        2
    );
    let recharge = breath_recharge(&ready, dragon);
    assert!(recharge.available && recharge.pending.is_none() && recharge.last_roll.is_none());
    let sources = Sources {
        mage,
        dragon,
        armor,
    };
    assert_armor(&ready, &sources);
    sources
}

async fn fork_file(
    source: &Fixture,
    export: &dmd_persistence::CampaignExport,
    path: &Path,
) -> Fixture {
    let pool = dmd_persistence::open_sqlite_path(path).await.unwrap();
    let runtime = runtime(pool.clone());
    Box::pin(runtime.restore_campaign(export)).await.unwrap();
    pool.close().await;
    let pool = dmd_persistence::open_sqlite_path(path).await.unwrap();
    let runtime = super::runtime(pool.clone());
    Box::pin(runtime.resume_campaign(source.campaign))
        .await
        .unwrap();
    Fixture {
        runtime,
        pool,
        campaign: source.campaign,
        players: source.players,
        characters: source.characters,
        actors: source.actors,
        session: source.session,
    }
}

async fn physical_breath(
    f: &mut Fixture,
    url: &str,
    directory: &Path,
    sources: &Sources,
    face: u16,
) {
    let options = view(f, None).await.tactical.unwrap().area_options.unwrap();
    assert_eq!(options.actor, sources.dragon);
    let feature = options
        .variants
        .iter()
        .find(|variant| variant.feature_id == "fire-breath")
        .unwrap();
    let breath = action(TacticalAction::CreatureArea {
        feature_id: feature.feature_id.clone(),
        aim: TacticalAreaAim {
            origin: point(40, 55, 6),
            toward: point(160, 55, 6),
            include_origin: false,
        },
        ordering: TacticalAreaOrdering::Host,
    });
    Box::pin(rejected(f, Some(0), breath.clone())).await;
    Box::pin(accept(
        f,
        url,
        directory,
        "genuine-dragon-breath",
        None,
        breath,
    ))
    .await;
    let pending = Box::new(state(f).await);
    let resolution = pending
        .encounter
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .resolution
        .as_ref()
        .unwrap();
    assert_eq!(resolution.areas.len(), 1);
    assert_eq!(resolution.areas[0].targets.len(), 1);
    assert_eq!(resolution.areas[0].targets[0].actor, sources.mage);
    assert_eq!(
        resolution.pending.as_ref().unwrap().key.role,
        TacticalRollRole::AreaDamage
    );
    assert!(!breath_recharge(&pending, sources.dragon).available);
    assert_eq!(
        view(f, None).await.roll.unwrap().dice,
        vec![DieSpec {
            count: 17,
            sides: 6
        }]
    );
    let damage = raw_action(f, None, &[face; 17]).await;
    Box::pin(accept(
        f,
        url,
        directory,
        "physical-breath-damage",
        None,
        damage,
    ))
    .await;
    let saving = Box::new(state(f).await);
    let pending = saving
        .encounter
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .resolution
        .as_ref()
        .unwrap()
        .pending
        .as_ref()
        .unwrap();
    assert_eq!(pending.key.role, TacticalRollRole::AreaSave);
    assert_eq!(pending.key.subject, sources.mage);
    assert_eq!(
        saving.rules.as_ref().unwrap().entities[&sources.mage].hp,
        81
    );
    let failed_save = raw_action(f, None, &[1]).await;
    Box::pin(accept(
        f,
        url,
        directory,
        "physical-mage-failed-save",
        None,
        failed_save,
    ))
    .await;
    let resolved = Box::new(state(f).await);
    assert!(
        resolved
            .encounter
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .resolution
            .is_none()
    );
    assert_armor(&resolved, sources);
    assert_eq!(
        resolved.rules.as_ref().unwrap().entities[&sources.mage].hp,
        if face == 1 { 64 } else { 0 }
    );
    assert_eq!(
        resolved.rules.as_ref().unwrap().entities[&sources.mage]
            .death
            .dead,
        face == 6
    );
}

async fn dead_mage_refusal(f: &mut Fixture, url: &str, directory: &Path, sources: &Sources) {
    Box::pin(physical_breath(f, url, directory, sources, 6)).await;
    Box::pin(accept(
        f,
        url,
        directory,
        "dead-mage-conclusion",
        None,
        conclusion(),
    ))
    .await;
    let before = Box::new(state(f).await);
    assert_eq!(
        before.entities[&sources.mage].existence,
        EntityExistence::Dead
    );
    assert!(
        before.rules.as_ref().unwrap().entities[&sources.mage]
            .concentration
            .is_none()
    );
    let finish = request(f, None, action(TacticalAction::FinishEncounter)).await;
    let error = Box::pin(rejected_request(f, finish)).await;
    assert!(
        error.contains("a retained timing dependency cannot enter the next initiative"),
        "{error}"
    );
    assert_eq!(state(f).await.encounter, before.encounter);
    assert!(state(f).await.encounter_history.is_none());
    Box::pin(accept(
        f,
        url,
        directory,
        "dead-mage-closed-session",
        None,
        TableAction::EndSession,
    ))
    .await;
    let closed = closed_request(f, action(TacticalAction::FinishEncounter)).await;
    let error = Box::pin(rejected_request(f, closed)).await;
    assert!(
        error.contains("a retained timing dependency cannot enter the next initiative"),
        "{error}"
    );
    // Resume the real old cadence; a failed release neither strands nor resurrects it.
    f.session = PlaySessionId::new();
    let resume = TableAction::StartSession {
        id: f.session,
        name: "The blocked old cadence resumes".into(),
        participants: vec![SessionParticipant {
            player_id: f.players[0],
            character_id: Some(f.characters[0]),
            attendance: AttendanceStatus::Present,
        }],
    };
    Box::pin(accept(
        f,
        url,
        directory,
        "dead-mage-cadence-resume",
        None,
        resume,
    ))
    .await;
    Box::pin(end_turn(
        f,
        url,
        directory,
        "dead-mage-old-turn-progress",
        None,
        sources.dragon,
    ))
    .await;
    let continued = Box::new(state(f).await);
    assert_eq!(
        continued.encounter.as_ref().unwrap().id,
        before.encounter.as_ref().unwrap().id
    );
    assert_eq!(
        continued
            .rules
            .as_ref()
            .unwrap()
            .timing
            .as_ref()
            .unwrap()
            .turn_number,
        3
    );
    assert_eq!(
        view(f, None).await.tactical.unwrap().active_actor,
        Some(f.actors[0])
    );
    assert!(
        continued.rules.as_ref().unwrap().entities[&sources.mage]
            .death
            .dead
    );
    assert_armor(&continued, sources);
}

async fn failed_recharge_before_release(
    f: &mut Fixture,
    url: &str,
    directory: &Path,
    sources: &Sources,
) -> (
    CreatureRechargeRecord,
    TableTransportRequest,
    TableTransportResult,
) {
    Box::pin(physical_breath(f, url, directory, sources, 1)).await;
    Box::pin(end_turn(
        f,
        url,
        directory,
        "spent-dragon-end",
        None,
        sources.dragon,
    ))
    .await;
    Box::pin(end_turn(
        f,
        url,
        directory,
        "first-pc-end",
        Some(0),
        sources.dragon,
    ))
    .await;
    Box::pin(end_turn(
        f,
        url,
        directory,
        "second-mage-end",
        None,
        sources.dragon,
    ))
    .await;
    let pending = Box::new(state(f).await);
    let ticket = breath_recharge(&pending, sources.dragon)
        .pending
        .as_ref()
        .unwrap();
    assert_eq!(ticket.turn.number, 5);
    assert_eq!(ticket.turn.actor, sources.dragon);
    assert_eq!(ticket.turn.boundary, TurnBoundary::Start);
    assert_eq!(
        view(f, None).await.roll.unwrap().dice,
        vec![DieSpec { count: 1, sides: 6 }]
    );
    let raw = raw_action(f, None, &[1]).await;
    Box::pin(rejected(f, Some(0), raw.clone())).await;
    let (request, response) = Box::pin(accept(
        f,
        url,
        directory,
        "actual-failed-recharge",
        None,
        raw,
    ))
    .await;
    let failed = Box::new(state(f).await);
    let recharge = breath_recharge(&failed, sources.dragon);
    assert!(!recharge.available && recharge.pending.is_none());
    let record = recharge.last_roll.as_ref().unwrap().clone();
    assert_eq!(record.accepted_by.id, request.command_id);
    assert_eq!(record.result.dice, vec![DieResult { sides: 6, value: 1 }]);
    assert_eq!(
        failed
            .rules
            .as_ref()
            .unwrap()
            .rolls
            .iter()
            .find(|row| row.request.id == record.ticket.request.id)
            .unwrap()
            .result,
        record.result
    );
    (record, request, response)
}

async fn carry_recharge(
    f: &mut Fixture,
    url: &str,
    directory: &Path,
    sources: &Sources,
    failed: &CreatureRechargeRecord,
) {
    Box::pin(accept(
        f,
        url,
        directory,
        "recharge-conclusion",
        None,
        conclusion(),
    ))
    .await;
    let before = Box::new(state(f).await);
    let original_roll = before
        .rules
        .as_ref()
        .unwrap()
        .rolls
        .iter()
        .find(|roll| roll.request.id == failed.ticket.request.id)
        .unwrap()
        .clone();
    assert_eq!(original_roll.request, failed.ticket.request);
    assert_eq!(original_roll.accepted_by, failed.accepted_by);
    assert_eq!(original_roll.result, failed.result);
    Box::pin(accept(
        f,
        url,
        directory,
        "recharge-finished",
        None,
        action(TacticalAction::FinishEncounter),
    ))
    .await;
    let finished = Box::new(state(f).await);
    assert_eq!(finished.clock, before.clock);
    assert_eq!(finished.items, before.items);
    assert_eq!(
        finished.rules.as_ref().unwrap().rolls,
        before.rules.as_ref().unwrap().rolls
    );
    assert_eq!(
        finished.rules.as_ref().unwrap().entities,
        before.rules.as_ref().unwrap().entities
    );
    assert_eq!(
        breath_recharge(&finished, sources.dragon),
        breath_recharge(&before, sources.dragon)
    );
    assert_eq!(
        finished
            .rules
            .as_ref()
            .unwrap()
            .tactical_creatures
            .as_ref()
            .unwrap()
            .runtime(sources.mage)
            .unwrap()
            .limited_uses,
        before
            .rules
            .as_ref()
            .unwrap()
            .tactical_creatures
            .as_ref()
            .unwrap()
            .runtime(sources.mage)
            .unwrap()
            .limited_uses
    );
    assert_armor(&finished, sources);
    let receipt = finished.encounter_history.as_ref().unwrap().last().unwrap();
    assert_eq!(receipt.final_turn.number, 5);
    let setup = battlefield(
        f,
        sources.mage,
        sources.dragon,
        finished.scenes[&receipt.scene_id].location_id,
    );
    Box::pin(accept(
        f,
        url,
        directory,
        "recharge-replacement",
        None,
        TableAction::PrepareBattlefield {
            setup: Box::new(setup),
        },
    ))
    .await;
    let initiative = begin(f, sources.mage, sources.dragon, true);
    Box::pin(accept(
        f,
        url,
        directory,
        "recharge-second-initiative",
        None,
        initiative,
    ))
    .await;
    let pending = Box::new(state(f).await);
    assert_eq!(
        breath_recharge(&pending, sources.dragon).last_roll.as_ref(),
        Some(failed)
    );
    assert!(!breath_recharge(&pending, sources.dragon).available);
    assert!(breath_recharge(&pending, sources.dragon).pending.is_none());
    assert_eq!(
        pending.rules.as_ref().unwrap().rolls,
        before.rules.as_ref().unwrap().rolls,
        "setup and requested initiative cannot replace the original raw proof"
    );
    for (player, face, label) in [
        (Some(0), 20, "second-pc-initiative"),
        (None, 5, "second-dragon-initiative"),
        (None, 2, "second-mage-initiative"),
    ] {
        let raw = raw_action(f, player, &[face]).await;
        Box::pin(accept(f, url, directory, label, player, raw)).await;
    }
    let first = Box::new(state(f).await);
    assert_eq!(
        first
            .rules
            .as_ref()
            .unwrap()
            .timing
            .as_ref()
            .unwrap()
            .turn_number,
        6
    );
    assert_eq!(
        first.rules.as_ref().unwrap().timing.as_ref().unwrap().round,
        1
    );
    assert_eq!(first.rules.as_ref().unwrap().entities[&sources.mage].hp, 64);
    assert_eq!(
        view(f, None).await.tactical.unwrap().active_actor,
        Some(f.actors[0])
    );
    assert_eq!(
        breath_recharge(&first, sources.dragon).last_roll.as_ref(),
        Some(failed)
    );
    assert!(!breath_recharge(&first, sources.dragon).available);
    assert!(breath_recharge(&first, sources.dragon).pending.is_none());
    assert_eq!(
        first
            .rules
            .as_ref()
            .unwrap()
            .rolls
            .iter()
            .find(|roll| roll.request.id == failed.ticket.request.id),
        Some(&original_roll)
    );
    assert_armor(&first, sources);
    Box::pin(end_turn(
        f,
        url,
        directory,
        "new-pc-end-before-dragon-start",
        Some(0),
        sources.dragon,
    ))
    .await;
    let due = Box::new(state(f).await);
    let recharge = breath_recharge(&due, sources.dragon);
    assert_eq!(recharge.last_roll.as_ref(), Some(failed));
    assert!(!recharge.available);
    let ticket = recharge.pending.as_ref().unwrap();
    assert_eq!(ticket.turn.number, 7);
    assert_eq!(ticket.turn.actor, sources.dragon);
    assert_eq!(ticket.turn.boundary, TurnBoundary::Start);
    assert_ne!(ticket.turn.encounter_id, failed.ticket.turn.encounter_id);
    assert_ne!(ticket.request.id, failed.ticket.request.id);
    let raw = raw_action(f, None, &[6]).await;
    let (successful_request, _) = Box::pin(accept(
        f,
        url,
        directory,
        "new-own-start-physical-recharge",
        None,
        raw,
    ))
    .await;
    let refreshed = Box::new(state(f).await);
    assert!(breath_recharge(&refreshed, sources.dragon).available);
    assert!(
        breath_recharge(&refreshed, sources.dragon)
            .pending
            .is_none()
    );
    assert_eq!(
        breath_recharge(&refreshed, sources.dragon)
            .last_roll
            .as_ref()
            .unwrap()
            .result
            .dice,
        vec![DieResult { sides: 6, value: 6 }]
    );
    assert_eq!(
        breath_recharge(&refreshed, sources.dragon)
            .last_roll
            .as_ref()
            .unwrap()
            .accepted_by
            .id,
        successful_request.command_id
    );
    assert_eq!(
        refreshed
            .rules
            .as_ref()
            .unwrap()
            .rolls
            .iter()
            .find(|row| row.request.id == failed.ticket.request.id),
        Some(&original_roll),
        "new own-Start success preserves the complete old failed RecordedRoll"
    );
    assert_armor(&refreshed, sources);
}

#[tokio::test]
async fn actual_dragon_recharge_survives_release_and_dead_mage_armor_blocks_the_other_history() {
    let directory =
        std::env::temp_dir().join(format!("dmd-release-recharge-{}", CampaignId::new().0));
    std::fs::create_dir_all(&directory).unwrap();
    let url = |name: &str| {
        format!(
            "sqlite://{}",
            directory.join(name).to_string_lossy().replace('\\', "/")
        )
    };
    let original_url = url("original.sqlite");
    let pool = open_sqlite(&original_url).await.unwrap();
    let mut original = Box::pin(Fixture::with_pool(TableContract::default(), pool)).await;
    let sources = Box::pin(prepare_sources(&mut original, &original_url, &directory)).await;
    let export = Box::new(
        export_campaign(&original.pool, original.campaign)
            .await
            .unwrap(),
    );
    let dead_url = url("dead-mage.sqlite");
    let mut dead = Box::pin(fork_file(
        &original,
        &export,
        &directory.join("dead-mage.sqlite"),
    ))
    .await;
    let alive_url = url("surviving-mage.sqlite");
    let mut alive = Box::pin(fork_file(
        &original,
        &export,
        &directory.join("surviving-mage.sqlite"),
    ))
    .await;
    original.pool.close().await;
    drop(original);
    Box::pin(dead_mage_refusal(
        &mut dead, &dead_url, &directory, &sources,
    ))
    .await;
    dead.pool.close().await;
    drop(dead);
    let (failed, request, response) = Box::pin(failed_recharge_before_release(
        &mut alive, &alive_url, &directory, &sources,
    ))
    .await;
    Box::pin(carry_recharge(
        &mut alive, &alive_url, &directory, &sources, &failed,
    ))
    .await;
    let before = Box::new(export_campaign(&alive.pool, alive.campaign).await.unwrap());
    assert_eq!(
        serde_json::to_vec(
            &Box::pin(alive.runtime.submit_presented_table(request.clone()))
                .await
                .unwrap()
        )
        .unwrap(),
        serde_json::to_vec(&response).unwrap()
    );
    let mut changed = request;
    let TableTransportInput::Action(declaration) = &mut changed.input else {
        panic!("recharge raw is an action")
    };
    let TableAction::Tactical {
        action: TacticalAction::SubmitRoll { result },
    } = declaration.as_mut()
    else {
        panic!("original recharge raw remains intact")
    };
    result.dice[0].value = 6;
    assert!(
        Box::pin(alive.runtime.submit_presented_table(changed))
            .await
            .is_err()
    );
    let mut after = Box::new(export_campaign(&alive.pool, alive.campaign).await.unwrap());
    after.exported_at_utc = before.exported_at_utc.clone();
    assert_eq!(
        after, before,
        "old failed recharge retry cannot rewrite the later successful source turn"
    );
    alive.pool.close().await;
    drop(alive);
    sqlite_test_cleanup::remove_closed_directory(&directory)
        .await
        .unwrap();
}
