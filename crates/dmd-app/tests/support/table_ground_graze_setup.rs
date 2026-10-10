//! Separate current-source producers; no legacy fixture semantics are changed.
use super::*;
use std::path::PathBuf;

async fn create_step(f: &mut Fixture, path: &Path, value: TableAction) {
    let shown = view(f, &TableTransportChannel::Host).await;
    let session_id = match &value {
        TableAction::StartSession { id, .. } => Some(*id),
        _ => shown
            .active_session
            .as_ref()
            .map(|session| session.session_id),
    };
    let request = TableTransportRequest {
        version: TABLE_TRANSPORT_VERSION,
        command_id: CommandId::new(),
        campaign_id: f.campaign,
        session_id,
        channel: TableTransportChannel::Host,
        revision: shown.revision,
        input: TableTransportInput::Action(Box::new(value)),
    };
    Box::pin(cold_step(f, path, request)).await;
}

pub(super) async fn current_fixture(definition: &str) -> (Fixture, PathBuf, PathBuf, ItemId) {
    let directory = std::env::temp_dir().join(format!("dmd-ground-graze-{}", CampaignId::new().0));
    std::fs::create_dir_all(&directory).unwrap();
    let path = directory.join("campaign.sqlite");
    let pool = dmd_persistence::open_sqlite_path(&path).await.unwrap();
    let mut f = Fixture {
        runtime: runtime(pool.clone()),
        pool,
        campaign: CampaignId::new(),
        players: [PlayerId::new(), PlayerId::new()],
        characters: [CharacterId::new(), CharacterId::new()],
        actors: [EntityId::new(), EntityId::new()],
        session: PlaySessionId::new(),
    };
    f.runtime
        .create_table_campaign(
            f.campaign,
            "Genuine Graze campaign",
            TableContract::default(),
        )
        .await
        .unwrap();
    let options = f
        .runtime
        .character_creation_options(f.campaign)
        .await
        .unwrap();
    assert_eq!(options.catalog.items.len(), 18);
    for index in 0..2 {
        let add = TableAction::AddPlayer {
            id: f.players[index],
            name: format!("Player {index}"),
        };
        Box::pin(create_step(&mut f, &path, add)).await;
        let mut selected = input(&format!("Current character {index}"));
        if index == 0 {
            selected.purchases.push(EquipmentChoice {
                item_id: definition.into(),
                quantity: 1,
            });
            selected.masteries = ["greatsword".into(), "glaive".into(), "dagger".into()];
        }
        let create = TableAction::CreateCharacterFromSource {
            character_id: f.characters[index],
            entity_id: f.actors[index],
            player_id: f.players[index],
            source: options.source.clone(),
            input: selected,
        };
        Box::pin(create_step(&mut f, &path, create)).await;
    }
    let created = state(&f).await;
    assert!(created.items.is_empty());
    let profile = &created.table.as_ref().unwrap().character_profiles[&f.characters[0]];
    assert_eq!(profile.creation_source.as_ref(), Some(&options.source));
    assert_eq!(
        profile.money_cp,
        20500 - 1000 - if definition == "glaive" { 2000 } else { 5000 }
    );
    assert!(
        created.rules.as_ref().unwrap().entities[&f.actors[0]]
            .attacks
            .is_empty()
    );
    for character in f.characters {
        let shown = view(&f, &TableTransportChannel::Host).await;
        let count = shown
            .characters
            .iter()
            .find(|row| row.character_id == character)
            .unwrap()
            .equipment
            .as_ref()
            .unwrap()
            .initial_item_count;
        Box::pin(create_step(
            &mut f,
            &path,
            TableAction::PrepareEquipment {
                character_id: character,
                item_ids: (0..count).map(|_| ItemId::new()).collect(),
            },
        ))
        .await;
    }
    let materialized = state(&f).await;
    let weapons = materialized
        .items
        .values()
        .filter(|item| {
            item.custody == Custody::Entity(f.actors[0]) && item.definition_id == definition
        })
        .collect::<Vec<_>>();
    assert_eq!(weapons.len(), 1);
    assert_eq!(weapons[0].quantity, 1);
    let item = weapons[0].id;
    let receipt = materialized
        .rules
        .as_ref()
        .unwrap()
        .tactical_inventory
        .as_ref()
        .unwrap()
        .receipt(f.characters[0])
        .unwrap();
    assert_eq!(
        receipt.source.profile_id,
        "human-fighter-soldier-level-1-physical-v1"
    );
    assert_eq!(&receipt.creation_profile, profile);
    assert_eq!(
        hands(&materialized, f.actors[0]).hands,
        [HandAssignment::Free; 2]
    );
    let start = TableAction::StartSession {
        id: f.session,
        name: "Both characters present".into(),
        participants: (0..2)
            .map(|index| SessionParticipant {
                player_id: f.players[index],
                character_id: Some(f.characters[index]),
                attendance: AttendanceStatus::Present,
            })
            .collect(),
    };
    Box::pin(create_step(&mut f, &path, start)).await;
    (f, directory, path, item)
}

pub(super) fn point(x: i32, y: i32, z: i32) -> SpatialPoint {
    SpatialPoint { x, y, z }
}
pub(super) fn floor() -> Battlefield {
    Battlefield {
        bounds: SpatialBox {
            min: point(0, 0, 0),
            max: point(100, 100, 40),
        },
        floor_z: 0,
        floor_surface: "stone".into(),
        ambient_light: LightLevel::Bright,
        terrain: vec![],
        obstacles: vec![],
        lights: vec![],
    }
}
pub(super) async fn battlefield(f: &mut Fixture, path: &Path, setup: TableBattlefieldSetup) {
    Box::pin(step(
        f,
        path,
        TableTransportChannel::Host,
        TableTransportInput::Action(Box::new(TableAction::PrepareBattlefield {
            setup: Box::new(setup),
        })),
    ))
    .await;
}
pub(super) async fn begin(f: &mut Fixture, path: &Path, combatants: Vec<TacticalCombatant>) {
    let groups = combatants
        .iter()
        .map(|c| InitiativeGroup {
            actors: vec![c.actor],
            request_id: RollRequestId::new(),
        })
        .collect();
    Box::pin(step(
        f,
        path,
        TableTransportChannel::Host,
        action(TacticalAction::Begin {
            execution: TacticalExecutionVersion::EncounterReleaseV1,
            combatants,
            groups,
        }),
    ))
    .await;
}
pub(super) async fn activate(f: &mut Fixture, path: &Path) {
    assert!(
        state(f)
            .await
            .encounter
            .unwrap()
            .flow
            .unwrap()
            .attack_equipment_access
            .is_none()
    );
    let activation = Box::pin(step(
        f,
        path,
        TableTransportChannel::Host,
        action(TacticalAction::ActivateAttackEquipment),
    ))
    .await;
    assert_eq!(
        state(f)
            .await
            .encounter
            .unwrap()
            .flow
            .unwrap()
            .attack_equipment_access
            .unwrap()
            .origin
            .id,
        activation.command_id
    );
}
pub(super) async fn basic_encounter(f: &mut Fixture, path: &Path) {
    let setup = TableBattlefieldSetup {
        encounter_id: EncounterId::new(),
        scene_id: SceneId::new(),
        location_id: LocationId::new(),
        name: "Illuminated practice floor".into(),
        area_grid_policy: None,
        battlefield: floor(),
        characters: (0..2)
            .map(|index| TableCharacterPlacement {
                character_id: f.characters[index],
                position: point(10 + index as i32 * 10, 10, 0),
                height: 12,
                allies: vec![],
                enemies: vec![f.actors[1 - index]],
            })
            .collect(),
        creatures: vec![],
        geometry_ruling: Ruling {
            basis: RulingBasis::GmAdjudication,
            reason: "A level bright floor and adjacent occupied cells with known opposition."
                .into(),
        },
    };
    Box::pin(battlefield(f, path, setup)).await;
    let combatants = f
        .actors
        .iter()
        .map(|actor| TacticalCombatant {
            actor: *actor,
            source: TacticalSource::Character,
            surprised: false,
        })
        .collect();
    Box::pin(begin(f, path, combatants)).await;
    Box::pin(player_raw(f, path, 18)).await;
    let second = other_player(f);
    Box::pin(raw(f, path, second, 2)).await;
    Box::pin(activate(f, path)).await;
}
