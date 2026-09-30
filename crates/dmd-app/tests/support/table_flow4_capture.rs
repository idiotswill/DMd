//! UNRUN diagnostic template for an isolated branch from verified flow 4 main.
//! Never merge this module or its call sites into production. No presentation,
//! request creation, mutation, retry, or continuation is performed here.
use super::*;
use serde_json::{Value, json};
use std::io::Write;

pub const HIT: &[&str] = &["flow4-hit-selected-shield"];
pub const OWNED: &[&str] = &["flow4-missile-selected-shield"];
pub const CONCENTRATION: &[&str] = &[
    "flow4-ordinary-committed-first-save",
    "flow4-missile-three-faces",
    "flow4-missile-all-faces-before-impact",
    "flow4-missile-first-concentration-child",
    "flow4-missile-penultimate-concentration-child",
];

#[derive(Debug, serde::Serialize)]
pub enum Boundary {
    HitSelected {
        attacker: EntityId,
        target: EntityId,
    },
    MissileSelected {
        caster: EntityId,
        selected: EntityId,
    },
    MissileProgress {
        caster: EntityId,
        target: EntityId,
        faces: usize,
        completed: usize,
        starting_hp: u32,
    },
    OrdinarySave {
        caster: EntityId,
        target: EntityId,
        accepted: CommandId,
    },
}

fn directory() -> std::path::PathBuf {
    let path = std::path::PathBuf::from(
        std::env::var_os("DMD_FLOW4_CAPTURE_DIR")
            .expect("explicit fresh external output directory"),
    );
    assert!(path.is_absolute() && path.is_dir());
    path
}

fn identity() -> Value {
    let value: Value =
        serde_json::from_slice(&std::fs::read(directory().join("provenance.json")).unwrap())
            .unwrap();
    // The external launcher must independently prove these values against Git.
    // The template's 940 anchors are never substituted for the actual producer.
    for field in [
        "producer_commit",
        "producer_tree",
        "diagnostic_commit",
        "diagnostic_tree",
    ] {
        let sha = value[field].as_str().expect("explicit Git identity");
        assert_eq!(sha.len(), 40);
        assert!(sha.bytes().all(|byte| byte.is_ascii_hexdigit()));
    }
    value
}

fn write_once(name: &str, bytes: &[u8]) {
    assert!(!name.contains('/') && !name.contains('\\'));
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(directory().join(name))
        .expect("capture output must not exist");
    file.write_all(bytes).unwrap();
    file.sync_all().unwrap();
}

fn write_json(name: &str, value: &Value) {
    write_once(name, &serde_json::to_vec_pretty(value).unwrap());
}

fn accepted_binding(
    export: &dmd_persistence::CampaignExport,
    command: CommandId,
) -> &dmd_persistence::TableTransportBinding {
    let found: Vec<_> = export
        .table_transport_bindings
        .iter()
        .filter(|binding| binding.meta.id == command)
        .collect();
    assert_eq!(found.len(), 1, "one original accepted transport binding");
    found[0]
}

fn assert_unpaid_shield(rules: &RulesState, actor: EntityId) {
    assert!(
        !rules
            .timing
            .as_ref()
            .unwrap()
            .reactions_spent
            .contains(&actor)
    );
    let source = rules.tactical_creatures.as_ref().unwrap();
    assert_eq!(
        source
            .runtime(actor)
            .unwrap()
            .limited_uses
            .iter()
            .filter(|usage| usage.feature_id == "protective-magic")
            .map(|usage| usize::from(usage.spent))
            .sum::<usize>(),
        0
    );
    assert_eq!(rules.entities[&actor].hp, 81);
}

fn pending(state: &CampaignState, role: TacticalRollRole, actor: EntityId) -> &TacticalPendingWork {
    let encounter = state.encounter.as_ref().unwrap();
    let work = encounter
        .flow
        .as_ref()
        .unwrap()
        .resolution
        .as_ref()
        .unwrap()
        .pending
        .as_ref()
        .unwrap();
    assert_eq!(work.key.role, role);
    let roll = state.rules.as_ref().unwrap().pending.as_ref().unwrap();
    assert_eq!(roll.request.roller, Some(actor));
    assert_eq!(roll.request.id, work.key.request_id());
    assert_eq!(
        roll.purpose,
        PendingPurpose::TacticalResolution {
            encounter: encounter.id,
            key: work.key,
        }
    );
    work
}

fn assert_boundary(
    state: &CampaignState,
    export: &dmd_persistence::CampaignExport,
    expected: &Boundary,
) {
    let encounter = state.encounter.as_ref().unwrap();
    let flow = encounter.flow.as_ref().unwrap();
    assert_eq!(flow.version, 4);
    assert_eq!(flow.phase, TacticalPhase::Active);
    let resolution = flow.resolution.as_ref().unwrap();
    let rules = state.rules.as_ref().unwrap();
    match *expected {
        Boundary::HitSelected { attacker, target } => {
            assert!(resolution.missiles.is_empty());
            let hit = resolution.hit_review.as_ref().unwrap();
            assert_eq!(hit.stage, TacticalHitReviewStage::Selected);
            assert_eq!(hit.respondent.as_ref().unwrap().actor, target);
            assert!(
                hit.respondent
                    .as_ref()
                    .unwrap()
                    .intent
                    .as_ref()
                    .unwrap()
                    .accepted
            );
            assert!(hit.completed_shield.is_none() && hit.selected_cast.is_none());
            let attack = resolution.attack.as_ref().unwrap();
            assert_eq!((attack.actor, attack.target), (attacker, target));
            assert_eq!(attack.origin.id, hit.attack_origin);
            assert_eq!(attack.attack_roll, Some(hit.attack_roll));
            assert!(attack.damage_roll.is_none() && rules.pending.is_none());
            let rolled = rules
                .rolls
                .iter()
                .find(|roll| roll.request.id == hit.attack_roll)
                .unwrap();
            assert_eq!(rolled.result.source, RollSource::Physical);
            assert_eq!(
                rolled.result.dice,
                vec![DieResult {
                    sides: 20,
                    value: 10
                }]
            );
            accepted_binding(export, attack.origin.id);
            accepted_binding(export, rolled.accepted_by.id);
            accepted_binding(export, hit.cause.id);
            assert_unpaid_shield(rules, target);
        }
        Boundary::MissileSelected { caster, selected } => {
            assert!(resolution.hit_review.is_none());
            assert_eq!(resolution.missiles.len(), 1);
            let missile = &resolution.missiles[0];
            let TacticalMissileStage::Selected { respondent } = missile.stage else {
                panic!("actual selected missile response required");
            };
            assert_eq!(
                missile.respondents[usize::from(respondent)].response.actor,
                selected
            );
            assert_eq!(
                resolution
                    .casts
                    .iter()
                    .find(|entry| entry.cast.plan.occurrence == missile.cast)
                    .unwrap()
                    .cast
                    .plan
                    .choice
                    .actor,
                caster
            );
            assert_eq!(missile.darts.len(), 6);
            assert!(
                missile
                    .darts
                    .iter()
                    .all(|dart| dart.amount.is_none() && dart.completed_by.is_none())
            );
            assert!(
                missile
                    .respondents
                    .iter()
                    .all(|respondent| respondent.completed_shield.is_none())
            );
            assert!(rules.pending.is_none());
            for respondent in &missile.respondents {
                assert_unpaid_shield(rules, respondent.response.actor);
            }
            assert_eq!(
                missile.order.as_ref().unwrap().instruction.ranked[0],
                selected
            );
            accepted_binding(export, missile.cause.id);
        }
        Boundary::MissileProgress {
            caster,
            target,
            faces,
            completed,
            starting_hp,
        } => {
            assert!(resolution.hit_review.is_none());
            assert_eq!(resolution.missiles.len(), 1);
            let missile = &resolution.missiles[0];
            assert_eq!(
                resolution
                    .casts
                    .iter()
                    .find(|entry| entry.cast.plan.occurrence == missile.cast)
                    .unwrap()
                    .cast
                    .plan
                    .choice
                    .actor,
                caster
            );
            assert_eq!(missile.darts.len(), 6);
            assert!(missile.darts.iter().all(|dart| dart.target == target));
            assert_eq!(
                missile
                    .darts
                    .iter()
                    .filter(|dart| dart.amount.is_some())
                    .count(),
                faces
            );
            assert_eq!(
                missile
                    .darts
                    .iter()
                    .filter(|dart| dart.completed_by.is_some())
                    .count(),
                completed
            );
            assert_eq!(
                rules.entities[&target].hp,
                starting_hp.checked_sub(2 * completed as u32).unwrap()
            );
            let mut raw_ids = std::collections::HashSet::new();
            for dart in &missile.darts {
                if let Some(key) = dart.amount {
                    assert_eq!(key.role, TacticalRollRole::SpellAmount);
                    assert!(raw_ids.insert(key.request_id()));
                    let roll = rules
                        .rolls
                        .iter()
                        .find(|roll| roll.request.id == key.request_id())
                        .unwrap();
                    assert_eq!(roll.result.source, RollSource::Physical);
                    assert_eq!(roll.result.dice, vec![DieResult { sides: 4, value: 1 }]);
                    assert_eq!(roll.request.modifier, 1);
                    accepted_binding(export, roll.accepted_by.id);
                }
                if let Some(done) = &dart.completed_by {
                    accepted_binding(export, done.id);
                }
            }
            if faces == 3 {
                assert_eq!(completed, 0);
                assert_eq!(missile.stage, TacticalMissileStage::Amounts);
                pending(state, TacticalRollRole::SpellAmount, caster);
                assert!(
                    missile
                        .darts
                        .iter()
                        .all(|dart| dart.impact_occurrence.is_none())
                );
            } else {
                assert_eq!(faces, 6);
                assert_eq!(missile.stage, TacticalMissileStage::Impacts);
                assert!(
                    missile
                        .darts
                        .iter()
                        .all(|dart| dart.impact_occurrence.is_some())
                );
                if completed == 0 {
                    assert!(rules.pending.is_none() && resolution.pending.is_none());
                } else {
                    assert!(matches!(completed, 1 | 5));
                    let work = pending(state, TacticalRollRole::Concentration, target);
                    assert_eq!(work.key.subject, target);
                    let TacticalWorkKind::ConcentrationSave {
                        actor,
                        group,
                        damage_taken,
                    } = work.work.kind
                    else {
                        panic!("real concentration child required");
                    };
                    assert_eq!((actor, damage_taken), (target, 2));
                    assert_eq!(rules.entities[&target].concentration, Some(group));
                    assert!(
                        rules
                            .tactical_effects
                            .as_ref()
                            .unwrap()
                            .groups
                            .iter()
                            .any(|entry| entry.id == group && entry.source.actor == target)
                    );
                    assert_eq!(
                        missile
                            .darts
                            .iter()
                            .filter(|dart| dart.selected_by.is_some())
                            .count(),
                        completed
                    );
                }
            }
        }
        Boundary::OrdinarySave {
            caster,
            target,
            accepted,
        } => {
            assert!(resolution.missiles.is_empty() && resolution.hit_review.is_none());
            assert_eq!(resolution.casts.len(), 1);
            let binding = &resolution.casts[0];
            let cast = &binding.cast;
            assert_eq!(cast.phase, SpellCastPhase::Committed);
            assert_eq!(cast.plan.origin.id, accepted);
            assert_eq!(cast.plan.choice.actor, caster);
            assert_eq!(cast.plan.choice.spell_id, "hold-person");
            assert_eq!(cast.plan.cost, SpellCastingCost::Action);
            assert!(cast.plan.activation_prepaid);
            let activation = binding.creature_activation.as_ref().unwrap();
            assert_eq!(activation.origin.id, accepted);
            assert_eq!(activation.activation, SpellEnclosingActivation::Action);
            assert_eq!(
                cast.plan.choice.resource,
                SpellResourceChoice::SourceFeature
            );
            assert_eq!(
                binding.selection,
                Some(SpellTargetChoice::Entities(vec![target]))
            );
            assert_eq!(
                binding.targets,
                vec![SpellBoundTarget {
                    actor: target,
                    source_type_matches: true
                }]
            );
            let pin = &cast.plan.program.source;
            let creatures = rules.tactical_creatures.as_ref().unwrap();
            let source = &creatures.profile(caster).unwrap().source;
            assert_eq!(pin.ruleset_id, source.ruleset_id);
            assert_eq!(pin.ruleset_version, source.ruleset_version);
            assert_eq!(
                pin.creature_definition_id.as_deref(),
                Some(source.definition_id.as_str())
            );
            assert_eq!(pin.spell_id, cast.plan.choice.spell_id);
            assert!(!pin.fingerprint.is_empty() && !cast.plan.program.nodes.is_empty());
            let SpellExpenditure::CreatureUse {
                ref feature_id,
                ref spell_id,
                maximum,
            } = cast.plan.expenditure
            else {
                panic!("actual limited source expenditure required");
            };
            assert_eq!(
                cast.plan.choice.grant,
                SpellGrantChoice::CreatureFeature {
                    feature_id: feature_id.clone()
                }
            );
            assert_eq!(pin.feature_id.as_deref(), Some(feature_id.as_str()));
            assert_eq!(spell_id, "hold-person");
            assert_eq!(maximum, 1);
            assert_eq!(
                creatures
                    .runtime(caster)
                    .unwrap()
                    .limited_uses
                    .iter()
                    .filter(|usage| usage.feature_id == *feature_id
                        && usage.spell_id.as_deref() == Some(spell_id.as_str()))
                    .map(|usage| usize::from(usage.spent))
                    .sum::<usize>(),
                1
            );
            assert!(rules.timing.as_ref().unwrap().action_spent);
            let group = cast.plan.concentration_group.unwrap();
            assert_eq!(rules.entities[&caster].concentration, Some(group));
            assert!(
                rules
                    .tactical_effects
                    .as_ref()
                    .unwrap()
                    .groups
                    .iter()
                    .any(|entry| entry.id == group && entry.source.actor == caster)
            );
            let work = pending(state, TacticalRollRole::SpellSave, target);
            assert_eq!(work.key.origin, accepted);
            assert_eq!(work.key.subject, target);
            assert_eq!(
                work.work.kind,
                TacticalWorkKind::SpellProgram {
                    cast: cast.plan.occurrence,
                    at: SpellProgramOccurrence { node: 0, target: 0 },
                }
            );
            let original = accepted_binding(export, accepted);
            assert_eq!(original.version, 2);
        }
    }
}

pub async fn capture(
    f: &Fixture,
    label: &str,
    expected: Boundary,
    roles: Value,
    prospective: Option<&TableTransportRequest>,
) {
    assert!(
        HIT.iter()
            .chain(OWNED)
            .chain(CONCENTRATION)
            .any(|known| *known == label)
    );
    let provenance = identity();
    let exported = Box::new(export_campaign(&f.pool, f.campaign).await.unwrap());
    let state = Box::new(CampaignState::decode_json(&exported.current_state.state_json).unwrap());
    assert_eq!(exported.campaign_id, f.campaign.0.to_string());
    assert_eq!(
        state
            .table
            .as_ref()
            .unwrap()
            .active_session
            .as_ref()
            .unwrap()
            .session_id,
        f.session
    );
    assert_boundary(&state, &exported, &expected);
    if let Some(next) = prospective {
        assert_eq!(next.version, 2);
        assert_eq!(next.campaign_id, f.campaign);
        assert_eq!(next.session_id, Some(f.session));
        assert!(
            !exported
                .table_transport_bindings
                .iter()
                .any(|binding| binding.meta.id == next.command_id)
        );
    }
    let rules = state.rules.as_ref().unwrap();
    let flow = state.encounter.as_ref().unwrap().flow.as_ref().unwrap();
    // This is a diagnostic inventory, not a reconstructed campaign/export.
    // Complete original history stays in the untouched official JSON below.
    let evidence = json!({
        "status": "raw candidate; producer and original-source baseline acceptance pending",
        "label": label, "provenance": provenance,
        "campaign_id": f.campaign, "session_id": f.session,
        "event_sequence": exported.current_state.applied_event_sequence,
        "exported_at_utc": exported.exported_at_utc,
        "executor": flow.version, "expected_boundary": expected,
        "roles": roles, "pending": rules.pending, "resolution": flow.resolution,
        "turn_budget": flow.budget, "timing": rules.timing,
        "actor_mechanics": rules.entities, "source_profiles_and_runtime": rules.tactical_creatures,
        "effects": rules.tactical_effects, "raw_rolls": rules.rolls,
        "active_session": state.table.as_ref().unwrap().active_session,
        "counts": {"events": exported.event_journal.len(), "audit": exported.command_audit.len(),
            "causes": exported.event_causes.len(), "snapshots": exported.snapshots.len(),
            "projections": exported.table_projection_history.len(), "bindings": exported.table_transport_bindings.len()},
        "prospective_unaccepted_request_json": prospective.map(|next| serde_json::to_string(next).unwrap()),
        "accepted_prefix_bindings": exported.table_transport_bindings,
    });
    let bytes = exported.to_json().unwrap();
    write_once(&format!("{label}.json"), bytes.as_bytes());
    write_json(&format!("{label}.evidence.json"), &evidence);
}

// Read-only, at the original scenario's successful tail before closing its pool.
// Records the one real eventual receipt for an already-created prospective input.
// A new baseline must query for fresh capabilities rather than replay this suffix.
pub async fn seal_receipts(f: &Fixture, labels: &[&str]) {
    let export = Box::new(export_campaign(&f.pool, f.campaign).await.unwrap());
    for label in labels {
        let evidence: Value = serde_json::from_slice(
            &std::fs::read(directory().join(format!("{label}.evidence.json"))).unwrap(),
        )
        .unwrap();
        assert_eq!(
            evidence["campaign_id"],
            serde_json::to_value(f.campaign).unwrap()
        );
        let receipt = evidence["prospective_unaccepted_request_json"]
            .as_str()
            .map(|json| {
                let request: TableTransportRequest = serde_json::from_str(json).unwrap();
                let binding = accepted_binding(&export, request.command_id);
                assert_eq!(
                    serde_json::from_str::<TableTransportRequest>(&binding.request_json).unwrap(),
                    request
                );
                serde_json::to_value(binding).unwrap()
            });
        write_json(
            &format!("{label}.later-receipt.json"),
            &json!({
                "status": "real later receipt, not part of the frozen accepted prefix",
                "binding": receipt,
                "final_sequence": export.current_state.applied_event_sequence,
            }),
        );
    }
}

// Called only after all unchanged scenario assertions and filesystem cleanup.
// A passing full test log and external hash manifest are still required.
pub fn producer_finished(producer: &str, labels: &[&str]) {
    for label in labels {
        for suffix in ["json", "evidence.json", "later-receipt.json"] {
            assert!(directory().join(format!("{label}.{suffix}")).is_file());
        }
    }
    write_json(
        &format!("{producer}.producer-finished.json"),
        &json!({
            "producer": producer, "labels": labels, "provenance": identity(),
            "status": "scenario reached original successful end; baseline acceptance still pending",
        }),
    );
}
