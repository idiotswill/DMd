//! Genuine current purchases and Graze continuations before owned equipment.
use super::final_helpers::{
    atomic_rejection, atomic_retry, close_fixture, current_source, hands, hostile_activation_rows,
    hostile_private_records, other_player,
};
use super::*;

#[path = "table_ground_graze_concentration.rs"]
mod concentration_child;
#[path = "table_ground_graze_fall.rs"]
mod death_fall;
#[path = "table_ground_graze_helpers.rs"]
mod helpers;
#[path = "table_ground_graze_setup.rs"]
mod setup_current;
use helpers::*;
use setup_current::*;

#[tokio::test]
async fn genuine_current_weapon_graze_apply_or_decline_precedes_owned_after_equipment() {
    for definition in ["greatsword", "glaive"] {
        for (face, graze) in [(1, true), (2, true), (2, false)] {
            for unequip in [false, true] {
                Box::pin(basic_case(definition, face, graze, unequip)).await;
            }
        }
    }
}

async fn basic_case(definition: &str, face: u16, graze: bool, unequip: bool) {
    let (mut f, directory, path, item) = Box::pin(current_fixture(definition)).await;
    Box::pin(basic_encounter(&mut f, &path)).await;
    let target = f.actors[1];
    Box::pin(equip_prior_miss(&mut f, &path, item, target)).await;
    Box::pin(player_step(&mut f, &path, action(TacticalAction::EndTurn))).await;
    let second = other_player(&f);
    Box::pin(step(&mut f, &path, second, action(TacticalAction::EndTurn))).await;
    let before = state(&f).await;
    assert_eq!(before.rules.as_ref().unwrap().entities[&target].hp, 12);
    let cut = Box::pin(start_graze(&mut f, &path, item, target, face)).await;
    let decision = Box::pin(choose_graze(&mut f, &path, graze)).await;
    let selected = state(&f).await;
    let rules = selected.rules.as_ref().unwrap();
    assert_eq!(rules.entities[&target].hp, 12 - if graze { 3 } else { 0 });
    assert_eq!(rules.rolls, cut.paused.rules.as_ref().unwrap().rolls);
    assert_eq!(
        rules.rolls.len(),
        before.rules.as_ref().unwrap().rolls.len() + 1
    );
    assert_eq!(selected.items, before.items);
    assert_eq!(
        hands(&selected, f.actors[0]).hands,
        [HandAssignment::Item(item); 2]
    );
    let resolution = resolution(&selected);
    assert!(resolution.pending.is_none());
    assert!(resolution.failed_save.is_none());
    assert!(resolution.falls.is_empty());
    let after = resolution.attack_after_equipment.as_ref().unwrap();
    assert_eq!(after.selected_by.as_ref().unwrap().id, decision.command_id);
    assert_eq!(after.cause.completed_by.id, decision.command_id);
    assert_eq!(after.cause.completed_work, cut.parent.work);
    let hostile = definition == "greatsword" && face == 1 && graze && !unequip;
    if hostile {
        Box::pin(hostile_activation_rows(&f)).await;
    }
    Box::pin(finish_equipment(
        &mut f, &path, item, &cut, unequip, hostile,
    ))
    .await;
    Box::pin(close_fixture(f, &directory)).await;
}

#[tokio::test]
async fn genuine_graze_concentration_child_finishes_before_after_equipment() {
    for succeeds in [false, true] {
        for unequip in [false, true] {
            Box::pin(concentration_child::run(succeeds, unequip)).await;
        }
    }
}

#[tokio::test]
async fn genuine_graze_target_death_fall_retains_finish_parent_and_owned_decline() {
    Box::pin(death_fall::run()).await;
}
