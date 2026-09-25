//! Isolated projection snapshots, not journaled acquisition or equipment actions.
//! The characters and starting gear are created through the real table runtime.
//! Additional source weapons represent already-acquired physical campaign items.
use super::*;
use crate::{CampaignRuntime, TableAction, TableContract, TableViewer};
use dmd_rules::tactical_weapons::*;
use std::path::Path;

fn creation(name: &str) -> dmd_rules::CharacterCreationInput {
    dmd_rules::CharacterCreationInput {
        name: name.into(),
        pronouns: "they/them".into(),
        description: "A source-created traveler".into(),
        alignment: "Neutral Good".into(),
        backstory: "".into(),
        ability_scores: [15, 14, 13, 8, 10, 12],
        background_boosts: [2, 0, 1, 0, 0, 0],
        fighter_skills: [Skill::Perception, Skill::Survival],
        human_skill: Skill::Insight,
        skilled_skills: [Skill::Acrobatics, Skill::Stealth, Skill::Investigation],
        size: CharacterSize::Medium,
        languages: ["dwarvish".into(), "elvish".into()],
        fighting_style: FightingStyle::Defense,
        gaming_set: GamingSet::Dice,
        purchases: ["leather-armor", "dagger", "shield"]
            .into_iter()
            .map(|item_id| EquipmentChoice {
                item_id: item_id.into(),
                quantity: 1,
            })
            .collect(),
        worn_armor: Some("leather-armor".into()),
        shield: false,
        masteries: ["club".into(), "dagger".into(), "shortbow".into()],
    }
}

async fn execute(runtime: &CampaignRuntime, campaign: CampaignId, action: TableAction) {
    let sequence = runtime
        .open_campaign(campaign)
        .await
        .unwrap()
        .state()
        .applied_event_sequence;
    runtime
        .execute_table(
            CommandMeta {
                id: CommandId::new(),
                campaign_id: campaign,
                session_id: None,
                issuer: CommandIssuer::Admin,
                actor: None,
                expected_event_sequence: sequence,
            },
            action,
        )
        .await
        .unwrap();
}

async fn source_snapshot() -> (CampaignState, [EntityId; 2]) {
    let pool = dmd_persistence::open_sqlite("sqlite::memory:")
        .await
        .unwrap();
    let runtime = CampaignRuntime::from_content_root(
        pool.clone(),
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../content"),
    );
    let campaign = CampaignId::new();
    runtime
        .create_table_campaign(
            campaign,
            "Projection source fixture",
            TableContract::default(),
        )
        .await
        .unwrap();
    let actors = [EntityId::new(), EntityId::new()];
    let characters = [CharacterId::new(), CharacterId::new()];
    for index in 0..2 {
        let player = PlayerId::new();
        execute(
            &runtime,
            campaign,
            TableAction::AddPlayer {
                id: player,
                name: format!("Controller {index}"),
            },
        )
        .await;
        execute(
            &runtime,
            campaign,
            TableAction::CreateCharacter {
                character_id: characters[index],
                entity_id: actors[index],
                player_id: player,
                input: creation(&format!("Traveler {index}")),
            },
        )
        .await;
    }
    let view = runtime
        .table_view(campaign, TableViewer::Host)
        .await
        .unwrap();
    let count = view
        .characters
        .iter()
        .find(|character| character.character_id == characters[0])
        .unwrap()
        .equipment
        .as_ref()
        .unwrap()
        .initial_item_count;
    execute(
        &runtime,
        campaign,
        TableAction::PrepareEquipment {
            character_id: characters[0],
            item_ids: (0..count).map(|_| ItemId::new()).collect(),
        },
    )
    .await;
    let state = runtime
        .open_campaign(campaign)
        .await
        .unwrap()
        .state()
        .clone();
    pool.close().await;
    (state, actors)
}

fn place_snapshot(state: &mut CampaignState, actors: [EntityId; 2]) {
    let location = LocationId::new();
    let scene = SceneId::new();
    state.locations.insert(
        location,
        Location {
            id: location,
            campaign_id: state.campaign_id(),
            display_name: "Visible field".into(),
            parent_location_id: None,
        },
    );
    state.scenes.insert(
        scene,
        Scene {
            id: scene,
            campaign_id: state.campaign_id(),
            location_id: location,
            mode: SceneMode::Combat,
            status: SceneStatus::Active,
            started_at: state.clock.now,
            presences: actors
                .into_iter()
                .map(|entity_id| ScenePresence {
                    entity_id,
                    role: PresenceRole::Participant,
                })
                .collect(),
        },
    );
    for actor in actors {
        state.entities.get_mut(&actor).unwrap().location_id = Some(location);
    }
    let origin = state
        .rules
        .as_ref()
        .unwrap()
        .tactical_inventory
        .as_ref()
        .unwrap()
        .loadout(actors[0])
        .unwrap()
        .command
        .clone();
    state.encounter = Some(TacticalEncounter {
        id: EncounterId::new(),
        scene_id: scene,
        battlefield: Battlefield {
            bounds: SpatialBox {
                min: point(0, 0),
                max: SpatialPoint {
                    x: 60,
                    y: 40,
                    z: 40,
                },
            },
            floor_z: 0,
            floor_surface: "stone".into(),
            ambient_light: LightLevel::Bright,
            terrain: vec![],
            obstacles: vec![],
            lights: vec![],
        },
        participants: actors
            .into_iter()
            .enumerate()
            .map(|(index, entity_id)| TacticalParticipant {
                entity_id,
                public_label: format!("Visible traveler {index}"),
                position: point(10 + index as i32 * 10, 10),
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
        origin,
        geometry_ruling: Ruling {
            basis: RulingBasis::GmAdjudication,
            reason: "Isolated immutable projection snapshot; no gameplay event claimed".into(),
        },
        flow: None,
    });
    state.encounter.as_ref().unwrap().validate(state).unwrap();
}

fn point(x: i32, y: i32) -> SpatialPoint {
    SpatialPoint { x, y, z: 0 }
}

fn item(state: &CampaignState, actor: EntityId, definition: &str) -> ItemId {
    state
        .items
        .values()
        .find(|item| item.custody == Custody::Entity(actor) && item.definition_id == definition)
        .unwrap()
        .id
}

#[tokio::test]
async fn opportunity_two_handed_projection_agrees_with_reaction_source_planning() {
    let (mut original, actors) = source_snapshot().await;
    place_snapshot(&mut original, actors);
    let pack =
        dmd_rules::RulesPack::from_json(include_str!("../../../content/srd-5.2.1/kernel.json"))
            .unwrap();
    let definitions = dmd_rules::tactical_definitions::TacticalDefinitions::from_json(
        dmd_rules::tactical_definitions::TACTICAL_DEFINITIONS_JSON,
    )
    .unwrap();
    for definition in ["greatsword", "quarterstaff"] {
        let source = dmd_rules::tactical_inventory::equipment_definition(definition).unwrap();
        let id = ItemId::new();
        original.items.insert(
            id,
            ItemInstance {
                id,
                campaign_id: original.campaign_id(),
                definition_id: definition.into(),
                display_name: source.display_name.clone(),
                quantity: 1,
                owner: Ownership::Entity(actors[0]),
                custody: Custody::Entity(actors[0]),
                state: ItemState::Intact,
            },
        );
    }
    for definition in ["greatsword", "quarterstaff"] {
        let weapon = item(&original, actors[0], definition);
        let shield = item(&original, actors[0], "shield");
        let dagger = item(&original, actors[0], "dagger");
        for (hands, allowed) in [
            ([HandAssignment::Item(weapon), HandAssignment::Free], true),
            ([HandAssignment::Free, HandAssignment::Item(weapon)], true),
            ([HandAssignment::Item(weapon); 2], true),
            (
                [HandAssignment::Item(weapon), HandAssignment::Item(shield)],
                false,
            ),
            (
                [HandAssignment::Item(weapon), HandAssignment::Item(dagger)],
                false,
            ),
            ([HandAssignment::Free; 2], false),
        ] {
            let mut state = original.clone();
            state
                .rules
                .as_mut()
                .unwrap()
                .tactical_inventory
                .as_mut()
                .unwrap()
                .loadouts
                .iter_mut()
                .find(|loadout| loadout.actor == actors[0])
                .unwrap()
                .hands
                .hands = hands;
            let loadout = state
                .rules
                .as_ref()
                .unwrap()
                .tactical_inventory
                .as_ref()
                .unwrap()
                .loadout(actors[0])
                .unwrap();
            dmd_rules::tactical_inventory::validate_loadout(&state, loadout).unwrap();
            let profile = state
                .table
                .as_ref()
                .unwrap()
                .character_profiles
                .values()
                .find(|profile| profile.entity_id == actors[0])
                .unwrap();
            let meta = CommandMeta {
                id: CommandId::new(),
                campaign_id: state.campaign_id(),
                session_id: None,
                issuer: CommandIssuer::Admin,
                actor: Some(AgentRef::Entity(actors[0])),
                expected_event_sequence: state.applied_event_sequence,
            };
            let choice = WeaponUseChoice {
                weapon,
                target: actors[1],
                delivery: WeaponDelivery::Melee,
                ability: Ability::Strength,
                grip: WeaponGrip::TwoHands,
                purpose: WeaponAttackPurpose::Normal,
                ammunition: None,
                equipment_change: None,
            };
            let plan = prepare_weapon_attack(&WeaponAttackInput {
                state: &state,
                source: WeaponActorSource::Character(profile),
                pack: &pack,
                definitions: &definitions,
                choice: &choice,
                loadout: &loadout.hands,
                history: &[],
                context: WeaponAttackContext {
                    origin: &meta,
                    actor: actors[0],
                    turn_number: 1,
                    on_actor_turn: false,
                    window: WeaponActionWindow {
                        id: meta.id,
                        kind: WeaponActionKind::Reaction,
                    },
                    distance: 10,
                    base_reach: 10,
                    mounted: false,
                    underwater: false,
                    has_swim_speed: false,
                    target_is_creature: true,
                    target_size: CreatureSize::Medium,
                    distance_from_trigger_target: None,
                },
            });
            assert_eq!(
                plan.is_ok(),
                allowed,
                "source plan: {definition}, {hands:?}"
            );
            if let Ok(plan) = &plan {
                assert_eq!(
                    plan.damage.dice,
                    vec![if definition == "greatsword" {
                        DieSpec { count: 2, sides: 6 }
                    } else {
                        DieSpec { count: 1, sides: 8 }
                    }],
                    "the offered grip must produce the source two-handed damage"
                );
                assert_eq!(plan.receipt.window.kind, WeaponActionKind::Reaction);
            }
            let before = state.clone();
            let projection = opportunity(
                &state,
                &TacticalOpportunityWindow {
                    origin: loadout.command.clone(),
                    reactor: actors[0],
                    mover: actors[1],
                    step_index: 0,
                    from: point(20, 10),
                    to: point(30, 10),
                    options: vec![TacticalMeleeOption {
                        source: TacticalMeleeSource::Weapon { item: weapon },
                        reach: 10,
                    }],
                },
            )
            .unwrap();
            let weapons = projection.weapons.unwrap();
            assert_eq!(weapons.targets.len(), 1);
            assert_eq!(weapons.targets[0].actor, actors[1]);
            let offered = weapons.weapons.iter().find(|option| option.item == weapon);
            assert_eq!(
                offered.is_some_and(|option| option.grips.contains(&WeaponGrip::TwoHands)),
                plan.is_ok(),
                "projection/source agreement: {definition}, {hands:?}"
            );
            if definition == "quarterstaff" && hands[0] == HandAssignment::Item(weapon) {
                assert!(
                    offered
                        .unwrap()
                        .grips
                        .contains(&WeaponGrip::OneHand(Hand::Left)),
                    "a blocked second hand does not erase legal one-handed Versatile use"
                );
            }
            assert_eq!(
                state, before,
                "read-only projection must not ready the second hand"
            );
        }
    }
}
