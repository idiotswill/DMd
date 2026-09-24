//! Real SQLite/application recovery and authority interactions for tactical initiative.
use super::*;
use dmd_rules::tactical::TacticalAction;

fn fixture() -> (Fixture, TacticalEncounter) {
    let mut f = Fixture::new();
    let location = LocationId::new();
    let scene = SceneId::new();
    f.state.locations.insert(
        location,
        Location {
            id: location,
            campaign_id: f.state.campaign_id(),
            display_name: "Open clearing".into(),
            parent_location_id: None,
        },
    );
    for actor in [f.actor, f.other_actor] {
        f.state.entities.get_mut(&actor).unwrap().location_id = Some(location);
    }
    f.state.scenes.insert(
        scene,
        Scene {
            id: scene,
            campaign_id: f.state.campaign_id(),
            location_id: location,
            mode: SceneMode::Combat,
            status: SceneStatus::Active,
            started_at: WorldInstant(0),
            presences: [f.actor, f.other_actor]
                .into_iter()
                .map(|entity_id| ScenePresence {
                    entity_id,
                    role: PresenceRole::Participant,
                })
                .collect(),
        },
    );
    let point = |x, y, z| SpatialPoint { x, y, z };
    let encounter = TacticalEncounter {
        id: EncounterId::new(),
        scene_id: scene,
        battlefield: Battlefield {
            bounds: SpatialBox {
                min: point(0, 0, 0),
                max: point(100, 100, 40),
            },
            floor_z: 0,
            floor_surface: "grass".into(),
            ambient_light: LightLevel::Bright,
            terrain: vec![],
            obstacles: vec![],
            lights: vec![],
        },
        participants: [(f.actor, 10), (f.other_actor, 50)]
            .into_iter()
            .map(|(entity_id, x)| TacticalParticipant {
                entity_id,
                public_label: "Traveler".into(),
                position: point(x, 10, 0),
                size: CreatureSize::Medium,
                height: 12,
                reach: 10,
                movement: MovementProfile {
                    walk: 60,
                    climb: None,
                    swim: None,
                    fly: None,
                    burrow: None,
                    hover: false,
                },
                senses: Senses::default(),
                allies: vec![],
                enemies: vec![],
            })
            .collect(),
        knowledge: vec![],
        origin: CommandMeta {
            id: CommandId::new(),
            campaign_id: f.state.campaign_id(),
            session_id: None,
            issuer: CommandIssuer::Admin,
            actor: None,
            expected_event_sequence: 0,
        },
        geometry_ruling: ruling(),
        flow: None,
    };
    (f, encounter)
}

async fn step(
    f: &Fixture,
    runtime: &CampaignRuntime,
    issuer: CommandIssuer,
    actor: Option<EntityId>,
    action: TacticalAction,
) -> dmd_app::TacticalReceipt {
    let seq = runtime
        .open_campaign(f.state.campaign_id())
        .await
        .unwrap()
        .state()
        .applied_event_sequence;
    runtime
        .execute_tactical(f.context(issuer, actor, seq), action)
        .await
        .unwrap()
}
async fn establish(f: &Fixture, runtime: &CampaignRuntime, encounter: TacticalEncounter) {
    step(
        f,
        runtime,
        CommandIssuer::Admin,
        None,
        TacticalAction::Establish {
            encounter: Box::new(encounter),
        },
    )
    .await;
}
fn begin_action(f: &Fixture, surprised: bool) -> TacticalAction {
    TacticalAction::Begin {
        combatants: vec![
            TacticalCombatant {
                actor: f.actor,
                source: TacticalSource::Character,
                surprised,
            },
            TacticalCombatant {
                actor: f.other_actor,
                source: TacticalSource::Character,
                surprised: false,
            },
        ],
        groups: [f.actor, f.other_actor]
            .into_iter()
            .map(|actor| InitiativeGroup {
                actors: vec![actor],
                request_id: RollRequestId::new(),
            })
            .collect(),
    }
}
async fn result(f: &Fixture, runtime: &CampaignRuntime, values: &[u16]) -> RollResult {
    let state = runtime.open_campaign(f.state.campaign_id()).await.unwrap();
    let request = &state
        .state()
        .rules
        .as_ref()
        .unwrap()
        .pending
        .as_ref()
        .unwrap()
        .request;
    RollResult {
        request_id: request.id,
        source: RollSource::Physical,
        dice: values
            .iter()
            .map(|value| DieResult {
                sides: 20,
                value: *value,
            })
            .collect(),
    }
}
async fn submit(
    f: &Fixture,
    runtime: &CampaignRuntime,
    player: PlayerId,
    actor: EntityId,
    values: &[u16],
) {
    let result = result(f, runtime, values).await;
    step(
        f,
        runtime,
        CommandIssuer::Player(player),
        Some(actor),
        TacticalAction::SubmitRoll { result },
    )
    .await;
}
async fn rejected_unchanged(
    f: &Fixture,
    pool: &sqlx::SqlitePool,
    runtime: &CampaignRuntime,
    issuer: CommandIssuer,
    actor: Option<EntityId>,
    action: TacticalAction,
) {
    let before = export_campaign(pool, f.state.campaign_id()).await.unwrap();
    let sequence = runtime
        .open_campaign(f.state.campaign_id())
        .await
        .unwrap()
        .state()
        .applied_event_sequence;
    assert!(
        runtime
            .execute_tactical(f.context(issuer, actor, sequence), action)
            .await
            .is_err()
    );
    let mut after = export_campaign(pool, f.state.campaign_id()).await.unwrap();
    after.exported_at_utc = before.exported_at_utc.clone();
    assert_eq!(after, before);
}

#[tokio::test]
async fn initiative_physical_faces_resume_after_restart_and_replay_restore_exactly() {
    let (f, encounter) = fixture();
    let (pool, runtime) = f.runtime().await;
    f.initialize(&runtime).await;
    establish(&f, &runtime, encounter).await;
    let start = step(
        &f,
        &runtime,
        CommandIssuer::Admin,
        None,
        begin_action(&f, true),
    )
    .await;
    assert_eq!(
        start.outcome.next_roll.as_ref().unwrap().mode,
        RollMode::Disadvantage
    );
    let pending_result = result(&f, &runtime, &[12, 4]).await;
    rejected_unchanged(
        &f,
        &pool,
        &runtime,
        CommandIssuer::Player(f.other_player),
        Some(f.other_actor),
        TacticalAction::SubmitRoll {
            result: pending_result,
        },
    )
    .await;
    f.assert_replay(&pool, &runtime).await;
    submit(&f, &runtime, f.player, f.actor, &[12, 4]).await;
    f.assert_replay(&pool, &runtime).await;
    let pending = runtime
        .open_campaign(f.state.campaign_id())
        .await
        .unwrap()
        .state()
        .clone();
    drop(runtime);
    pool.close().await;
    let (pool, runtime) = f.runtime().await;
    assert_eq!(
        runtime
            .resume_campaign(f.state.campaign_id())
            .await
            .unwrap()
            .state(),
        &pending
    );
    submit(&f, &runtime, f.other_player, f.other_actor, &[17]).await;
    let final_state = runtime.open_campaign(f.state.campaign_id()).await.unwrap();
    let rules = final_state.state().rules.as_ref().unwrap();
    assert_eq!(rules.timing.as_ref().unwrap().order[0].actor, f.other_actor);
    assert_eq!(
        rules.rolls[0]
            .resolved
            .raw_dice
            .iter()
            .map(|d| d.value)
            .collect::<Vec<_>>(),
        vec![12, 4]
    );
    assert_eq!(rules.rolls[0].resolved.kept_dice[0].value, 4);
    f.assert_replay(&pool, &runtime).await;
    let mut corrupt = export_campaign(&pool, f.state.campaign_id()).await.unwrap();
    corrupt
        .event_journal
        .last_mut()
        .unwrap()
        .event_schema_version += 1;
    let target = open_sqlite("sqlite::memory:").await.unwrap();
    let restore = CampaignRuntime::from_content_root(target.clone(), &f.content);
    assert!(restore.restore_campaign(&corrupt).await.is_err());
    assert!(
        dmd_persistence::open_campaign(&target, f.state.campaign_id())
            .await
            .is_err()
    );
    drop((runtime, restore));
    pool.close().await;
    target.close().await;
}

#[tokio::test]
async fn player_only_tie_requires_every_controller_and_reproposal_resets_agreement() {
    let (f, encounter) = fixture();
    let (pool, runtime) = f.runtime().await;
    f.initialize(&runtime).await;
    establish(&f, &runtime, encounter).await;
    step(
        &f,
        &runtime,
        CommandIssuer::Admin,
        None,
        begin_action(&f, false),
    )
    .await;
    submit(&f, &runtime, f.player, f.actor, &[10]).await;
    submit(&f, &runtime, f.other_player, f.other_actor, &[10]).await;
    rejected_unchanged(
        &f,
        &pool,
        &runtime,
        CommandIssuer::Admin,
        None,
        TacticalAction::ProposeInitiativeTie {
            order: vec![f.actor, f.other_actor],
        },
    )
    .await;
    step(
        &f,
        &runtime,
        CommandIssuer::Player(f.player),
        Some(f.actor),
        TacticalAction::ProposeInitiativeTie {
            order: vec![f.actor, f.other_actor],
        },
    )
    .await;
    step(
        &f,
        &runtime,
        CommandIssuer::Player(f.other_player),
        Some(f.other_actor),
        TacticalAction::ProposeInitiativeTie {
            order: vec![f.other_actor, f.actor],
        },
    )
    .await;
    assert!(
        runtime
            .open_campaign(f.state.campaign_id())
            .await
            .unwrap()
            .state()
            .rules
            .as_ref()
            .unwrap()
            .timing
            .is_none()
    );
    f.assert_replay(&pool, &runtime).await;
    let state = runtime.open_campaign(f.state.campaign_id()).await.unwrap();
    let TacticalPhase::InitiativeTies { ties } = &state
        .state()
        .encounter
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .phase
    else {
        panic!("tie expected")
    };
    assert_eq!(ties[0].accepted_by, vec![f.other_player]);
    let total = ties[0].total;
    rejected_unchanged(
        &f,
        &pool,
        &runtime,
        CommandIssuer::Player(f.other_player),
        Some(f.other_actor),
        TacticalAction::AcceptInitiativeTie { total },
    )
    .await;
    step(
        &f,
        &runtime,
        CommandIssuer::Player(f.player),
        Some(f.actor),
        TacticalAction::AcceptInitiativeTie { total },
    )
    .await;
    assert_eq!(
        runtime
            .open_campaign(f.state.campaign_id())
            .await
            .unwrap()
            .state()
            .rules
            .as_ref()
            .unwrap()
            .timing
            .as_ref()
            .unwrap()
            .order[0]
            .actor,
        f.other_actor
    );
    f.assert_replay(&pool, &runtime).await;
    drop(runtime);
    pool.close().await;
}

#[tokio::test]
async fn tactical_initiative_inspiration_preserves_original_faces_and_spends_once() {
    let (f, encounter) = fixture();
    let (pool, runtime) = f.runtime().await;
    f.initialize(&runtime).await;
    f.step(
        &runtime,
        CommandIssuer::Admin,
        None,
        RulesAction::GrantInspiration {
            actor: f.actor,
            ruling: ruling(),
        },
    )
    .await;
    establish(&f, &runtime, encounter).await;
    step(
        &f,
        &runtime,
        CommandIssuer::Admin,
        None,
        begin_action(&f, true),
    )
    .await;
    let original = result(&f, &runtime, &[18, 2]).await;
    let action = TacticalAction::SubmitRollWithInspiration {
        result: original.clone(),
        die_index: 1,
        replacement: DieResult {
            sides: 20,
            value: 14,
        },
    };
    rejected_unchanged(
        &f,
        &pool,
        &runtime,
        CommandIssuer::Player(f.other_player),
        Some(f.other_actor),
        action.clone(),
    )
    .await;
    step(
        &f,
        &runtime,
        CommandIssuer::Player(f.player),
        Some(f.actor),
        action.clone(),
    )
    .await;
    let state = runtime.open_campaign(f.state.campaign_id()).await.unwrap();
    let rules = state.state().rules.as_ref().unwrap();
    assert!(!rules.entities[&f.actor].heroic_inspiration);
    assert_eq!(rules.rolls[0].original_result, Some(original));
    assert_eq!(rules.rolls[0].resolved.kept_dice[0].value, 14);
    rejected_unchanged(
        &f,
        &pool,
        &runtime,
        CommandIssuer::Player(f.player),
        Some(f.actor),
        action,
    )
    .await;
    f.assert_replay(&pool, &runtime).await;
    drop(runtime);
    pool.close().await;
}

#[tokio::test]
async fn initiative_interrupts_rest_and_visible_fear_changes_the_roll() {
    let (mut f, encounter) = fixture();
    let outsider = EntityId::new();
    f.state.entities.insert(
        outsider,
        WorldEntity {
            id: outsider,
            campaign_id: f.state.campaign_id(),
            display_name: "Distant traveler".into(),
            kind: EntityKind::Creature,
            existence: EntityExistence::Present,
            location_id: None,
        },
    );
    let (pool, runtime) = f.runtime().await;
    runtime.create_campaign(&f.state).await.unwrap();
    f.step(
        &runtime,
        CommandIssuer::Admin,
        None,
        RulesAction::Initialize {
            entities: vec![
                mechanics(f.actor),
                mechanics(f.other_actor),
                mechanics(outsider),
            ],
            house_rules: HouseRules::default(),
            ruling: ruling(),
        },
    )
    .await;
    f.step(
        &runtime,
        CommandIssuer::Admin,
        None,
        RulesAction::StartRest {
            actor: outsider,
            kind: RestKind::Short,
            ruling: ruling(),
        },
    )
    .await;
    f.step(
        &runtime,
        CommandIssuer::Admin,
        None,
        RulesAction::StartRest {
            actor: f.actor,
            kind: RestKind::Short,
            ruling: ruling(),
        },
    )
    .await;
    f.step(
        &runtime,
        CommandIssuer::Admin,
        None,
        RulesAction::ApplyEffect {
            effect: ActiveEffect {
                id: EffectId::new(),
                source: f.other_actor,
                target: f.actor,
                condition: Some(Condition::Frightened),
                label: "Fear source".into(),
                expires: Expiry::Never,
                concentration_owner: None,
            },
            ruling: ruling(),
        },
    )
    .await;
    establish(&f, &runtime, encounter).await;
    let receipt = step(
        &f,
        &runtime,
        CommandIssuer::Admin,
        None,
        begin_action(&f, false),
    )
    .await;
    assert_eq!(
        receipt.outcome.next_roll.unwrap().mode,
        RollMode::Disadvantage
    );
    assert_eq!(
        runtime
            .open_campaign(f.state.campaign_id())
            .await
            .unwrap()
            .state()
            .rules
            .as_ref()
            .unwrap()
            .rests
            .iter()
            .map(|r| r.actor)
            .collect::<Vec<_>>(),
        vec![outsider]
    );
    submit(&f, &runtime, f.player, f.actor, &[5, 13]).await;
    submit(&f, &runtime, f.other_player, f.other_actor, &[17]).await;
    f.assert_replay(&pool, &runtime).await;
    drop(runtime);
    pool.close().await;
}

#[tokio::test]
async fn identical_creatures_share_source_initiative_and_host_resolves_mixed_ties() {
    let (mut f, mut encounter) = fixture();
    let third = EntityId::new();
    f.state
        .characters
        .retain(|_, c| c.entity_id != f.other_actor);
    f.state.entities.get_mut(&f.other_actor).unwrap().kind = EntityKind::Creature;
    let mut third_world = f.state.entities[&f.other_actor].clone();
    third_world.id = third;
    f.state.entities.insert(third, third_world);
    f.state
        .scenes
        .get_mut(&encounter.scene_id)
        .unwrap()
        .presences
        .push(ScenePresence {
            entity_id: third,
            role: PresenceRole::Participant,
        });
    let mut third_token = encounter.participants[1].clone();
    third_token.entity_id = third;
    third_token.position.y = 60;
    encounter.participants.push(third_token);
    for participant in encounter.participants.iter_mut().skip(1) {
        participant.size = CreatureSize::Large;
    }
    let definitions = dmd_rules::tactical_definitions::TacticalDefinitions::from_json(
        dmd_rules::tactical_definitions::TACTICAL_DEFINITIONS_JSON,
    )
    .unwrap();
    let source = definitions.creature("young-red-dragon").unwrap();
    let creature = |actor| {
        let mut e = MechanicalEntity::basic(actor);
        e.ability_scores = source.statistics.ability_scores;
        e.max_hp = source.statistics.hit_points;
        e.hp = e.max_hp;
        e.armor = ArmorClass::Fixed(u16::from(source.statistics.armor_class));
        e
    };
    let (pool, runtime) = f.runtime().await;
    runtime.create_campaign(&f.state).await.unwrap();
    f.step(
        &runtime,
        CommandIssuer::Admin,
        None,
        RulesAction::Initialize {
            entities: vec![mechanics(f.actor), creature(f.other_actor), creature(third)],
            house_rules: HouseRules::default(),
            ruling: ruling(),
        },
    )
    .await;
    establish(&f, &runtime, encounter).await;
    let combatants = vec![
        TacticalCombatant {
            actor: f.actor,
            source: TacticalSource::Character,
            surprised: false,
        },
        TacticalCombatant {
            actor: f.other_actor,
            source: TacticalSource::Creature {
                definition_id: source.id.clone(),
            },
            surprised: false,
        },
        TacticalCombatant {
            actor: third,
            source: TacticalSource::Creature {
                definition_id: source.id.clone(),
            },
            surprised: false,
        },
    ];
    rejected_unchanged(
        &f,
        &pool,
        &runtime,
        CommandIssuer::Admin,
        None,
        TacticalAction::Begin {
            combatants: combatants.clone(),
            groups: [f.actor, f.other_actor, third]
                .into_iter()
                .map(|actor| InitiativeGroup {
                    actors: vec![actor],
                    request_id: RollRequestId::new(),
                })
                .collect(),
        },
    )
    .await;
    step(
        &f,
        &runtime,
        CommandIssuer::Admin,
        None,
        TacticalAction::Begin {
            combatants,
            groups: vec![
                InitiativeGroup {
                    actors: vec![f.actor],
                    request_id: RollRequestId::new(),
                },
                InitiativeGroup {
                    actors: vec![f.other_actor, third],
                    request_id: RollRequestId::new(),
                },
            ],
        },
    )
    .await;
    submit(&f, &runtime, f.player, f.actor, &[10]).await;
    let state = runtime.open_campaign(f.state.campaign_id()).await.unwrap();
    let request = &state
        .state()
        .rules
        .as_ref()
        .unwrap()
        .pending
        .as_ref()
        .unwrap()
        .request;
    assert_eq!(request.modifier, 4); // Source initiative, despite Dexterity 10.
    assert_eq!(request.visibility, RollVisibility::Secret);
    let mut rolled = result(&f, &runtime, &[8]).await;
    rolled.source = RollSource::Digital;
    rejected_unchanged(
        &f,
        &pool,
        &runtime,
        CommandIssuer::Player(f.player),
        Some(f.actor),
        TacticalAction::SubmitRoll {
            result: rolled.clone(),
        },
    )
    .await;
    step(
        &f,
        &runtime,
        CommandIssuer::System,
        None,
        TacticalAction::SubmitRoll { result: rolled },
    )
    .await;
    rejected_unchanged(
        &f,
        &pool,
        &runtime,
        CommandIssuer::Player(f.player),
        Some(f.actor),
        TacticalAction::ProposeInitiativeTie {
            order: vec![third, f.actor, f.other_actor],
        },
    )
    .await;
    step(
        &f,
        &runtime,
        CommandIssuer::Admin,
        None,
        TacticalAction::ProposeInitiativeTie {
            order: vec![third, f.actor, f.other_actor],
        },
    )
    .await;
    let state = runtime.open_campaign(f.state.campaign_id()).await.unwrap();
    assert_eq!(state.state().rules.as_ref().unwrap().rolls.len(), 2);
    assert_eq!(
        state
            .state()
            .rules
            .as_ref()
            .unwrap()
            .timing
            .as_ref()
            .unwrap()
            .order[0]
            .actor,
        third
    );
    f.assert_replay(&pool, &runtime).await;
    drop(runtime);
    pool.close().await;
}
