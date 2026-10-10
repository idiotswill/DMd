use super::{TableBattlefieldSetup, reducer::table};
use crate::tactical::*;
use dmd_domain::*;

pub(crate) fn prepare(
    state: &CampaignState,
    next: &mut CampaignState,
    meta: &CommandMeta,
    setup: &TableBattlefieldSetup,
) -> Result<TacticalAction, String> {
    bounded_text(&setup.name, 200)?;
    let replacement = state.encounter.is_some();
    if replacement {
        require_finished_encounter(state).map_err(|error| error.to_string())?;
    }
    if state.scenes.contains_key(&setup.scene_id)
        || setup.location_id.0.is_nil()
        || setup.scene_id.0.is_nil()
        || setup.encounter_id.0.is_nil()
        || setup.characters.is_empty()
            && !(crate::table::source_control::enabled(state)
                && setup.creatures.iter().any(|placement| {
                    table(state)
                        .ok()
                        .and_then(|table| table.active_session.as_ref())
                        .is_some_and(|session| {
                            session.participants.iter().any(|participant| {
                                participant.attendance == AttendanceStatus::Present
                                    && crate::table::source_control::owns_source(
                                        state,
                                        participant.player_id,
                                        placement.actor,
                                    )
                            })
                        })
                }))
        || setup.characters.len().saturating_add(setup.creatures.len()) > 100
    {
        return Err("Choose a new encounter and scene with at least one character.".into());
    }
    if replacement
        && state
            .encounter_history
            .as_ref()
            .is_some_and(|history| history.contains_encounter(setup.encounter_id))
    {
        return Err("Choose a new encounter identity; completed encounters remain saved.".into());
    }
    if let Some(location) = next.locations.get(&setup.location_id) {
        if location.campaign_id != meta.campaign_id {
            return Err("The location belongs to another campaign.".into());
        }
    } else {
        next.locations.insert(
            setup.location_id,
            Location {
                id: setup.location_id,
                campaign_id: meta.campaign_id,
                display_name: setup.name.clone(),
                parent_location_id: None,
            },
        );
    }
    let mut participants = Vec::new();
    for placement in &setup.characters {
        let character = state
            .characters
            .get(&placement.character_id)
            .ok_or("Unknown encounter character.")?;
        let player = character
            .controlling_player_id
            .ok_or("An encounter character requires a controller.")?;
        if replacement
            && (character.status != CharacterStatus::Active
                || state
                    .rules
                    .as_ref()
                    .and_then(|rules| rules.entities.get(&character.entity_id))
                    .is_none_or(|entity| entity.death.dead))
        {
            return Err("A dead character cannot enter a new encounter.".into());
        }
        let session = meta
            .session_id
            .ok_or("Encounter setup requires an active session.")?;
        table(state)?.validate_attendance(session, player, placement.character_id)?;
        let profile = table(state)?
            .character_profiles
            .get(&placement.character_id)
            .ok_or("The character has no source creation profile.")?;
        if state
            .rules
            .as_ref()
            .and_then(|r| r.tactical_inventory.as_ref())
            .and_then(|inventory| inventory.receipt(placement.character_id))
            .is_none()
        {
            return Err(
                "Prepare each character's starting equipment before encounter setup.".into(),
            );
        }
        participants.push(TacticalParticipant {
            entity_id: character.entity_id,
            public_label: character.display_name.clone(),
            position: placement.position,
            size: match profile.size {
                CharacterSize::Small => CreatureSize::Small,
                CharacterSize::Medium => CreatureSize::Medium,
            },
            height: placement.height,
            reach: 10,
            movement: MovementProfile {
                walk: u32::from(profile.speed_feet) * 2,
                climb: None,
                swim: None,
                fly: None,
                burrow: None,
                hover: false,
            },
            senses: Senses::default(),
            allies: placement.allies.clone(),
            enemies: placement.enemies.clone(),
        });
        if !replacement {
            next.entities
                .get_mut(&character.entity_id)
                .ok_or("Character entity is absent.")?
                .location_id = Some(setup.location_id);
        }
    }
    for placement in &setup.creatures {
        bounded_text(&placement.public_label, 200)?;
        let world = state
            .entities
            .get(&placement.actor)
            .ok_or("Unknown source creature.")?;
        let profile = state
            .rules
            .as_ref()
            .and_then(|r| r.tactical_creatures.as_ref())
            .and_then(|creatures| creatures.profile(placement.actor))
            .ok_or("Creature source profile is absent.")?;
        if world.existence != EntityExistence::Present
            || replacement
                && state
                    .rules
                    .as_ref()
                    .and_then(|rules| rules.entities.get(&placement.actor))
                    .is_none_or(|entity| entity.death.dead)
            || state
                .rules
                .as_ref()
                .and_then(|r| r.tactical_inventory.as_ref())
                .and_then(|i| i.loadout(placement.actor))
                .is_none()
        {
            return Err("Place a living source creature with its prepared equipment.".into());
        }
        let source =
            crate::tactical_creatures::source_for_profile(profile).map_err(|e| e.to_string())?;
        let reach = source
            .features
            .iter()
            .filter_map(|feature| match &feature.feature {
                crate::tactical_definitions::MonsterFeature::Attack {
                    delivery: crate::tactical_definitions::AttackDelivery::Melee { reach_feet },
                    ..
                } => Some(u32::from(*reach_feet) * 2),
                _ => None,
            })
            .max()
            .unwrap_or(10);
        participants.push(TacticalParticipant {
            entity_id: placement.actor,
            public_label: placement.public_label.clone(),
            position: placement.position,
            size: profile.size,
            height: placement.height,
            reach,
            movement: crate::tactical_creatures::creature_movement(source),
            senses: crate::tactical_creatures::creature_senses(source),
            allies: placement.allies.clone(),
            enemies: placement.enemies.clone(),
        });
        if !replacement {
            next.entities
                .get_mut(&placement.actor)
                .ok_or("Creature entity is absent.")?
                .location_id = Some(setup.location_id);
        }
    }
    next.scenes.insert(
        setup.scene_id,
        Scene {
            id: setup.scene_id,
            campaign_id: meta.campaign_id,
            location_id: setup.location_id,
            mode: SceneMode::Combat,
            // A replacement is staged closed until the rules transition installs
            // it atomically. The retained old scene stays closed throughout.
            status: if replacement {
                SceneStatus::Closed
            } else {
                SceneStatus::Active
            },
            started_at: state.clock.now,
            presences: participants
                .iter()
                .map(|p| ScenePresence {
                    entity_id: p.entity_id,
                    role: PresenceRole::Participant,
                })
                .collect(),
        },
    );
    let encounter = TacticalEncounter {
        id: setup.encounter_id,
        scene_id: setup.scene_id,
        battlefield: setup.battlefield.clone(),
        participants,
        knowledge: vec![],
        origin: meta.clone(),
        area_grid_policy: setup.area_grid_policy,
        geometry_ruling: setup.geometry_ruling.clone(),
        flow: None,
    };
    Ok(TacticalAction::Establish {
        encounter: Box::new(encounter),
    })
}
