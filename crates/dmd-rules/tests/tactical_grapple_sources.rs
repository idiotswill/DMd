//! Source/shape checkpoint controls, not Grapple gameplay acceptance. The genuine
//! old export is read without altering its original source pins or journal bytes.
use dmd_domain::*;
use dmd_rules::{
    tactical_creatures::*, tactical_definitions::*,
    tactical_grapple_sources::ordinary_grapple_anatomy, *,
};

fn old_state() -> (serde_json::Value, CampaignState) {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../dmd-app/tests/fixtures/reactions-v1-upgrade-100c7da.json");
    let export: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let raw: serde_json::Value =
        serde_json::from_str(export["current_state"]["state_json"].as_str().unwrap()).unwrap();
    let state = serde_json::from_value(raw.clone()).unwrap();
    (raw, state)
}

fn pack() -> RulesPack {
    RulesPack::from_json(include_str!("../../../content/srd-5.2.1/kernel.json")).unwrap()
}

#[test]
fn immutable_goblin_revision_changes_only_the_reviewed_typed_anatomy() {
    let old = creature_definition("goblin-warrior").unwrap();
    let new = bundled_goblin_warrior_v2().unwrap();
    assert!(old.ordinary_hands.is_none());
    assert_eq!(
        creature_definition_fingerprint(old).unwrap(),
        "b1b3d06ef838810e"
    );
    let mut normalized = old.clone();
    normalized.ordinary_hands = Some(OrdinaryHandAnatomy::TwoHandsV1);
    assert_eq!(&normalized, new);
    assert_eq!(
        serde_json::to_value(old).unwrap().get("ordinary_hands"),
        None
    );
    let old_pin = creature_source_pin(old).unwrap();
    let new_pin = creature_source_pin(new).unwrap();
    assert_ne!(old_pin, new_pin);
    assert_eq!(creature_source(&old_pin).unwrap(), old);
    assert_eq!(creature_source(&new_pin).unwrap(), new);
    let all = immutable_creature_sources().unwrap();
    assert_eq!(all.iter().filter(|s| s.id == "goblin-warrior").count(), 2);
    let current = current_creature_sources().unwrap();
    assert_eq!(
        current
            .iter()
            .filter(|s| s.id == "goblin-warrior")
            .copied()
            .collect::<Vec<_>>(),
        vec![new]
    );
    let ogre_pin = creature_source_pin(bundled_ogre().unwrap()).unwrap();
    assert_eq!(
        current
            .iter()
            .filter(|source| creature_source_pin(source).unwrap() == ogre_pin)
            .count(),
        1
    );
    let mage_pin = creature_source_pin(bundled_mage_v2().unwrap()).unwrap();
    assert_eq!(
        current
            .iter()
            .filter(|source| creature_source_pin(source).unwrap() == mage_pin)
            .count(),
        1
    );
    for source in &current {
        let pin = creature_source_pin(source).unwrap();
        if pin == new_pin || pin == ogre_pin || pin == mage_pin {
            assert_eq!(source.ordinary_hands, Some(OrdinaryHandAnatomy::TwoHandsV1));
        } else {
            assert!(source.ordinary_hands.is_none());
        }
    }
    let mut forged = new_pin;
    forged.definition_fingerprint = old_pin.definition_fingerprint;
    assert_eq!(
        creature_source(&forged).unwrap(),
        old,
        "a full old pin still means the old source"
    );
    forged.definition_fingerprint = "0000000000000000".into();
    assert!(creature_source(&forged).is_err());
}

#[test]
fn genuine_old_state_keeps_its_source_and_optional_absence_while_human_query_reconstructs() {
    let (raw, mut state) = old_state();
    assert_eq!(serde_json::to_value(&state).unwrap(), raw);
    validate_state(&state, &pack()).unwrap();
    assert!(!has_tactical_grapple_attachments(&state));
    assert!(!has_unimplemented_grapple_records(&state));
    let goblin = state
        .rules
        .as_ref()
        .unwrap()
        .tactical_creatures
        .as_ref()
        .unwrap()
        .profiles
        .iter()
        .find(|p| p.source.definition_id == "goblin-warrior")
        .unwrap()
        .actor;
    assert_eq!(
        ordinary_grapple_anatomy(&state, goblin, &pack()).unwrap(),
        None
    );
    let (&character, profile) = state
        .table
        .as_ref()
        .unwrap()
        .character_profiles
        .iter()
        .next()
        .unwrap();
    let actor = profile.entity_id;
    assert_eq!(
        ordinary_grapple_anatomy(&state, actor, &pack()).unwrap(),
        Some(GrappleAnatomyProof::HumanCreationV1 { character })
    );
    // Hostile local shape controls, not accepted commands or a new saved history.
    state
        .table
        .as_mut()
        .unwrap()
        .character_profiles
        .get_mut(&character)
        .unwrap()
        .features
        .clear();
    assert!(ordinary_grapple_anatomy(&state, actor, &pack()).is_err());
    state
        .table
        .as_mut()
        .unwrap()
        .character_profiles
        .remove(&character);
    assert_eq!(
        ordinary_grapple_anatomy(&state, actor, &pack()).unwrap(),
        None
    );
}

#[test]
fn new_source_builder_supplies_anatomy_without_inventing_hands_for_old_or_air_sources() {
    let (_, mut state) = old_state();
    let actor = EntityId::new();
    state.entities.insert(
        actor,
        WorldEntity {
            id: actor,
            campaign_id: state.campaign_id(),
            display_name: "Arbitrary source name".into(),
            kind: EntityKind::Creature,
            existence: EntityExistence::Present,
            location_id: None,
        },
    );
    let origin = CommandMeta {
        id: CommandId::new(),
        campaign_id: state.campaign_id(),
        session_id: None,
        issuer: CommandIssuer::Admin,
        actor: None,
        expected_event_sequence: state.applied_event_sequence,
    };
    let new_pin = creature_source_pin(bundled_goblin_warrior_v2().unwrap()).unwrap();
    let choice = CreatureBuildChoice {
        definition_id: "goblin-warrior".into(),
        size: CreatureSize::Small,
        additional_languages: vec![],
        hit_points: CreatureHitPointChoice::Average,
        controller: CreatureController::Host,
        in_lair: false,
    };
    let built =
        build_creature_from_source(&state, &origin, actor, &choice, Some(&new_pin)).unwrap();
    // Isolated pure source-adapter installation. No journal/production coexistence
    // acceptance is claimed; that requires root's release5 table path.
    let rules = state.rules.as_mut().unwrap();
    rules.entities.insert(actor, built.mechanics);
    let creatures = rules.tactical_creatures.as_mut().unwrap();
    creatures.profiles.push(built.profile);
    creatures.runtime.push(built.runtime);
    assert_eq!(
        ordinary_grapple_anatomy(&state, actor, &pack()).unwrap(),
        Some(GrappleAnatomyProof::Creature {
            source: new_pin,
            ordinary_hands: OrdinaryHandAnatomy::TwoHandsV1,
        })
    );
    assert!(bundled_air_elemental().unwrap().ordinary_hands.is_none());
    assert!(
        creature_definition("wolf")
            .unwrap()
            .ordinary_hands
            .is_none()
    );
}

#[test]
fn source_domain_checkpoint_refuses_live_orphaned_and_raw_only_injection() {
    let (_, state) = old_state();
    let before = serde_json::to_value(&state).unwrap();
    for mutation in 0..4 {
        let mut forged = state.clone();
        match mutation {
            0 => {
                forged.rules.as_mut().unwrap().tactical_grapples = Some(TacticalGrapples {
                    schema_version: 1,
                    active: vec![],
                })
            }
            1 => {
                // Deliberately forged orphan consumer, with no live grip field.
                // Detection must not depend on a new producer having run first.
                let flow = forged.encounter.as_mut().unwrap().flow.as_mut().unwrap();
                flow.resolution = Some(Box::new(TacticalResolution {
                    attack_after_equipment: None,
                    grapple: Some(Box::new(TacticalGrappleResolution {
                        transport: None,
                        activity: None,
                        proofs: vec![],
                        cuts: vec![],
                        ends: vec![],
                        opportunity_refreshes: vec![],
                    })),
                    origin: flow.origin.clone(),
                    turn_actor: flow.combatants[0].actor,
                    turn_number: 1,
                    boundary: TurnBoundary::Start,
                    frames: vec![],
                    pending: None,
                    failed_save: None,
                    legendary_window: None,
                    attack: None,
                    shove: None,
                    hit_review: None,
                    movement: None,
                    casts: vec![],
                    missiles: vec![],
                    falls: vec![],
                    areas: vec![],
                    work_trace: None,
                    next_occurrence: 0,
                }));
            }
            2 => {
                let roll = forged
                    .rules
                    .as_mut()
                    .unwrap()
                    .rolls
                    .iter_mut()
                    .find(|r| matches!(r.purpose, PendingPurpose::TacticalResolution { .. }))
                    .unwrap();
                let PendingPurpose::TacticalResolution { key, .. } = &mut roll.purpose else {
                    unreachable!()
                };
                key.role = TacticalRollRole::GrappleSave;
            }
            _ => {
                let flow = forged.encounter.as_mut().unwrap().flow.as_mut().unwrap();
                flow.save_decisions.push(TacticalSaveDecision {
                    key: TacticalRollKey {
                        origin: flow.origin.id,
                        role: TacticalRollRole::GrappleSave,
                        subject: flow.combatants[0].actor,
                        occurrence: 0,
                    },
                    issued_by: flow.origin.clone(),
                    resolved_by: flow.origin.clone(),
                    failure: TacticalSaveFailure::Automatic,
                });
            }
        }
        assert!(has_unimplemented_grapple_records(&forged));
        assert!(validate_state(&forged, &pack()).is_err());
    }
    assert_eq!(serde_json::to_value(&state).unwrap(), before);
}
