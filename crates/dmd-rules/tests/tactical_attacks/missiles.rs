//! Source-built initial images exercise the public reducer and replay on each
//! transition. Actual table creation/transport/SQLite evidence remains separate.
use super::*;

fn hag(mage_target: bool) -> Fixture {
    let mut f = Fixture::new();
    hit_shield::source_actor(&mut f, 0, "night-hag", CreatureSize::Medium);
    if mage_target {
        hit_shield::source_actor(&mut f, 1, "mage", CreatureSize::Medium);
    }
    f.begin();
    f
}

fn cast(f: &mut Fixture, targets: Vec<EntityId>) {
    f.run(
        Some(0),
        TacticalAction::CastSpell {
            choice: SpellCastChoice {
                actor: f.actors[0],
                spell_id: "magic-missile".into(),
                grant: SpellGrantChoice::CreatureFeature {
                    feature_id: "spellcasting".into(),
                },
                resource: SpellResourceChoice::SourceFeature,
                material: SpellMaterialChoice::None,
                mode: SpellCastMode::Immediate,
            },
            targets: SpellTargetChoice::Entities(targets),
        },
    );
    assert!(f.rules().pending.is_none());
    assert!(f.rules().timing.as_ref().unwrap().action_spent);
    assert_eq!(missile(f).stage, TacticalMissileStage::Collecting);
}

fn missile(f: &Fixture) -> &TacticalMissile {
    &f.flow().resolution.as_ref().unwrap().missiles[0]
}

fn window(f: &Fixture) -> TacticalWorkKey {
    TacticalWorkKey {
        resolution: f.flow().resolution.as_ref().unwrap().origin.id,
        occurrence: missile(f).work.occurrence,
    }
}

fn order(f: &mut Fixture) {
    f.run(
        Some(0),
        TacticalAction::OrderMissileResponses {
            window: window(f),
            instruction: hit_responses::forward_order(),
        },
    );
}

fn respond(f: &mut Fixture, actor: usize, accept: bool) {
    f.run(
        Some(actor),
        TacticalAction::RespondToMissile {
            window: window(f),
            actor: f.actors[actor],
            accept,
        },
    );
}

fn amounts(f: &mut Fixture, values: &[u16]) {
    let hp = f.actors.map(|actor| f.rules().entities[&actor].hp);
    for (index, face) in values.iter().enumerate() {
        assert_eq!(f.request().dice, vec![DieSpec { count: 1, sides: 4 }]);
        assert_eq!(f.request().modifier, 1);
        f.roll(0, &[*face]);
        assert_eq!(f.actors.map(|actor| f.rules().entities[&actor].hp), hp);
        assert_eq!(
            missile(f)
                .darts
                .iter()
                .filter(|dart| dart.amount.is_some())
                .count(),
            index + 1
        );
        assert!(
            missile(f)
                .darts
                .iter()
                .all(|dart| dart.completed_by.is_none())
        );
    }
    assert_eq!(missile(f).stage, TacticalMissileStage::Impacts);
    assert!(f.rules().pending.is_none());
    assert_eq!(
        f.flow()
            .resolution
            .as_ref()
            .unwrap()
            .frames
            .last()
            .unwrap()
            .len(),
        values.len()
    );
}

fn choose(f: &mut Fixture, index: usize) {
    let occurrence = missile(f).darts[index].impact_occurrence.unwrap();
    f.run(Some(0), TacticalAction::ChooseTurnWork { occurrence });
}

fn drain(f: &mut Fixture, reverse: bool) {
    while f.flow().resolution.is_some() {
        assert!(f.rules().pending.is_none());
        let next = missile(f)
            .darts
            .iter()
            .enumerate()
            .filter(|(_, dart)| dart.completed_by.is_none())
            .map(|(index, _)| index)
            .collect::<Vec<_>>();
        choose(
            f,
            if reverse {
                *next.last().unwrap()
            } else {
                next[0]
            },
        );
    }
}

#[test]
fn missiles_collect_all_faces_before_hp_and_retain_reverse_occurrence_order() {
    let mut f = hag(false);
    f.entity_mut(1).hp = 100;
    f.entity_mut(1).max_hp = 100;
    let targets = vec![f.actors[1]; 6];
    cast(&mut f, targets);
    assert_eq!(missile(&f).respondents.len(), 1);
    order(&mut f);
    assert!(f.rules().pending.is_none());
    respond(&mut f, 1, false);
    amounts(&mut f, &[1, 2, 3, 4, 1, 2]);
    let original = missile(&f)
        .darts
        .iter()
        .map(|dart| dart.amount.unwrap())
        .collect::<Vec<_>>();
    let occurrence = missile(&f).darts[5].impact_occurrence.unwrap();
    f.rejected(None, TacticalAction::ChooseTurnWork { occurrence });
    f.rejected(Some(1), TacticalAction::ChooseTurnWork { occurrence });
    choose(&mut f, 5);
    assert_eq!(f.rules().entities[&f.actors[1]].hp, 97);
    assert!(missile(&f).darts[5].completed_by.is_some());
    assert!(
        missile(&f).darts[..5]
            .iter()
            .all(|dart| dart.completed_by.is_none())
    );
    assert_eq!(
        missile(&f)
            .darts
            .iter()
            .map(|dart| dart.amount.unwrap())
            .collect::<Vec<_>>(),
        original
    );
    drain(&mut f, true);
    assert_eq!(f.rules().entities[&f.actors[1]].hp, 81);
}

#[test]
fn missiles_shield_spends_once_but_retains_every_prevented_face() {
    for intent_first in [false, true] {
        let mut f = hag(true);
        let target = f.actors[1];
        cast(&mut f, vec![target; 6]);
        let action = TacticalAction::RespondToMissile {
            window: window(&f),
            actor: target,
            accept: true,
        };
        f.rejected(None, action.clone());
        f.rejected(Some(0), action);
        if intent_first {
            respond(&mut f, 1, true);
            order(&mut f);
        } else {
            order(&mut f);
            respond(&mut f, 1, true);
        }
        assert_eq!(
            missile(&f).stage,
            TacticalMissileStage::Selected { respondent: 0 }
        );
        assert!(
            f.rules()
                .timing
                .as_ref()
                .unwrap()
                .reactions_spent
                .is_empty()
        );
        let choice = shield_choices(&f.state, target).unwrap().remove(0);
        f.run(
            Some(1),
            TacticalAction::CastMissileShield {
                window: window(&f),
                choice,
            },
        );
        assert_eq!(
            f.rules().timing.as_ref().unwrap().reactions_spent,
            vec![target]
        );
        assert!(missile(&f).respondents[0].completed_shield.is_some());
        amounts(&mut f, &[4; 6]);
        drain(&mut f, false);
        assert_eq!(f.rules().entities[&target].hp, 81);
        let spent: u8 = f
            .rules()
            .tactical_creatures
            .as_ref()
            .unwrap()
            .runtime(target)
            .unwrap()
            .limited_uses
            .iter()
            .filter(|use_| use_.feature_id == "protective-magic")
            .map(|use_| use_.spent)
            .sum();
        assert_eq!(spent, 1);
    }
}

#[test]
fn missiles_selected_decline_and_response_delegation_do_not_transfer_impact_authority() {
    let mut f = hag(true);
    let target = f.actors[1];
    cast(&mut f, vec![target; 6]);
    f.run(
        Some(0),
        TacticalAction::DelegateMissileResponses { window: window(&f) },
    );
    respond(&mut f, 1, true);
    f.run(
        None,
        TacticalAction::OrderMissileResponses {
            window: window(&f),
            instruction: hit_responses::forward_order(),
        },
    );
    f.rejected(
        None,
        TacticalAction::DeclineSelectedMissileShield { window: window(&f) },
    );
    f.run(
        Some(1),
        TacticalAction::DeclineSelectedMissileShield { window: window(&f) },
    );
    assert!(
        f.rules()
            .timing
            .as_ref()
            .unwrap()
            .reactions_spent
            .is_empty()
    );
    amounts(&mut f, &[1; 6]);
    let occurrence = missile(&f).darts[0].impact_occurrence.unwrap();
    f.rejected(None, TacticalAction::ChooseTurnWork { occurrence });
    drain(&mut f, false);
    assert_eq!(f.rules().entities[&target].hp, 69);
}

#[test]
fn missiles_concentration_children_drain_between_committed_darts() {
    let mut f = hag(false);
    f.entity_mut(1).hp = 100;
    f.entity_mut(1).max_hp = 100;
    let target = f.actors[1];
    let group = spell::focus(&mut f);
    cast(&mut f, vec![target; 6]);
    respond(&mut f, 1, false);
    order(&mut f);
    amounts(&mut f, &[4; 6]);
    choose(&mut f, 5);
    assert_eq!(f.rules().entities[&target].hp, 95);
    let pending = f
        .flow()
        .resolution
        .as_ref()
        .unwrap()
        .pending
        .as_ref()
        .unwrap();
    assert!(
        matches!(pending.work.kind, TacticalWorkKind::ConcentrationSave { actor, group: actual, damage_taken: 5 } if actor == target && actual == group)
    );
    let occurrence = missile(&f).darts[0].impact_occurrence.unwrap();
    f.rejected(Some(0), TacticalAction::ChooseTurnWork { occurrence });
    f.roll(1, &[1]);
    assert!(f.rules().entities[&target].concentration.is_none());
    drain(&mut f, false);
    assert_eq!(f.rules().entities[&target].hp, 70);
}

#[test]
fn missiles_committed_impacts_survive_caster_death_and_dead_targets_are_no_effects() {
    let mut f = hag(false);
    // Bounded vitality initial image: a real source caster with 1 HP. Three
    // self-targeted committed strikes precede the target's three strikes.
    f.entity_mut(0).hp = 1;
    f.entity_mut(1).max_hp = 100;
    f.entity_mut(1).hp = 100;
    let [caster, target] = f.actors;
    cast(&mut f, vec![caster, target, caster, target, caster, target]);
    respond(&mut f, 0, false);
    respond(&mut f, 1, false);
    order(&mut f);
    amounts(&mut f, &[4; 6]);
    choose(&mut f, 0);
    // Source monsters die at zero by their retained death policy; the caster's
    // later two occurrences still exist and finish as authenticated no-effects.
    assert!(f.rules().entities[&caster].death.dead);
    choose(&mut f, 2);
    assert!(missile(&f).darts[2].completed_by.is_some());
    assert_eq!(f.rules().entities[&caster].hp, 0);
    drain(&mut f, false);
    assert_eq!(f.rules().entities[&target].hp, 85);
    assert_eq!(f.rules().entities[&caster].hp, 0);
}

#[test]
fn missiles_reject_forged_targets_raw_keys_barriers_and_retired_occurrences() {
    let mut f = hag(false);
    let target = f.actors[1];
    cast(&mut f, vec![target; 6]);
    order(&mut f);
    respond(&mut f, 1, false);
    amounts(&mut f, &[1; 6]);
    choose(&mut f, 5);
    for alteration in 0..9 {
        let mut bad = f.state.clone();
        let r = bad
            .encounter
            .as_mut()
            .unwrap()
            .flow
            .as_mut()
            .unwrap()
            .resolution
            .as_mut()
            .unwrap();
        match alteration {
            0 => r.missiles[0].darts[0].target = f.actors[0],
            1 => r.missiles[0].darts[0].amount = r.missiles[0].darts[1].amount,
            2 => {
                r.missiles[0].darts.remove(0);
            }
            3 => {
                r.missiles[0].darts[0].impact_occurrence = r.missiles[0].darts[1].impact_occurrence
            }
            4 => {
                r.frames.last_mut().unwrap().pop();
            }
            5 => r.missiles[0].darts[5].completed_by = None,
            6 => r.missiles.clear(),
            7 => {
                r.work_trace
                    .as_mut()
                    .unwrap()
                    .nodes
                    .iter_mut()
                    .find(|node| {
                        node.work.occurrence == r.missiles[0].darts[5].impact_occurrence.unwrap()
                    })
                    .unwrap()
                    .parent = None
            }
            _ => {
                r.missiles[0].respondents[0]
                    .response
                    .intent
                    .as_mut()
                    .unwrap()
                    .origin
                    .issuer = CommandIssuer::Admin
            }
        };
        assert!(
            validate_tactical_state(&bad).is_err(),
            "accepted alteration {alteration}"
        );
    }
}
