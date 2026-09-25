use dmd_domain::*;
use dmd_rules::{tactical::*, tactical_effects::*, *};
use std::collections::HashMap;

fn resistance_save(ability: Ability) -> EffectTriggerPayload {
    EffectTriggerPayload::SavingThrow {
        ability,
        dc: 30,
        on_success: EffectSaveEnd::TargetEffect,
        on_failure: EffectSaveEnd::None,
    }
}

#[test]
fn occupied_end_space_uses_actual_volume_size_exceptions_and_prone_immunity() {
    for (own, other, immune, overlap, prone) in [
        (
            CreatureSize::Medium,
            CreatureSize::Medium,
            false,
            true,
            true,
        ),
        (CreatureSize::Tiny, CreatureSize::Medium, false, true, false),
        (
            CreatureSize::Large,
            CreatureSize::Medium,
            false,
            true,
            false,
        ),
        (CreatureSize::Medium, CreatureSize::Large, false, true, true),
        (
            CreatureSize::Medium,
            CreatureSize::Medium,
            true,
            true,
            false,
        ),
        (
            CreatureSize::Medium,
            CreatureSize::Medium,
            false,
            false,
            false,
        ),
    ] {
        let mut f = Fixture::new();
        f.begin();
        let e = f.state.encounter.as_mut().unwrap();
        e.participants[0].size = own;
        e.participants[1].size = other;
        e.participants[1].position = e.participants[0].position;
        if !overlap {
            // Exact touching edges are distinct spaces, including on a grid.
            e.participants[1].position.x = e.participants[0].volume().unwrap().max.x;
        }
        if immune {
            f.entity_mut(0)
                .condition_immunities
                .insert(Condition::Prone);
        }
        f.run(Some(0), TacticalAction::EndTurn);
        assert_eq!(f.rules().entities[&f.actors[0]].prone, prone);
    }
}

#[test]
fn occupied_space_consequence_shares_end_order_and_rejects_foreign_or_duplicate_work() {
    let mut f = Fixture::new();
    f.effect_at(
        1,
        EffectTriggerPayload::SavingThrow {
            ability: Ability::Wisdom,
            dc: 5,
            on_success: EffectSaveEnd::TargetEffect,
            on_failure: EffectSaveEnd::None,
        },
        vec![],
        TurnBoundary::End,
    );
    f.begin();
    let e = f.state.encounter.as_mut().unwrap();
    e.participants[1].position = e.participants[0].position;
    f.run(Some(0), TacticalAction::EndTurn);
    assert!(!f.rules().entities[&f.actors[0]].prone);
    let work = f
        .resolution()
        .frames
        .last()
        .unwrap()
        .iter()
        .find(|w| matches!(w.kind, TacticalWorkKind::EndOccupiedSpace { .. }))
        .unwrap()
        .clone();
    let choice = TacticalAction::ChooseTurnWork {
        occurrence: work.occurrence,
    };
    f.rejected(Some(1), choice.clone());
    for duplicate in [false, true] {
        let mut corrupt = f.state.clone();
        let r = corrupt
            .encounter
            .as_mut()
            .unwrap()
            .flow
            .as_mut()
            .unwrap()
            .resolution
            .as_mut()
            .unwrap();
        if duplicate {
            let mut copied = work.clone();
            copied.occurrence = r.next_occurrence;
            r.next_occurrence += 1;
            r.frames.last_mut().unwrap().push(copied);
        } else {
            r.frames
                .last_mut()
                .unwrap()
                .iter_mut()
                .find(|w| w.occurrence == work.occurrence)
                .unwrap()
                .kind = TacticalWorkKind::EndOccupiedSpace { actor: f.actors[1] };
        }
        assert!(validate_tactical_state(&corrupt).is_err());
    }
    f.run(Some(0), choice);
    assert!(f.rules().entities[&f.actors[0]].prone);
    assert_eq!(
        f.rules().pending.as_ref().unwrap().request.roller,
        Some(f.actors[1])
    );
    f.roll(1, &[10]);
    assert_eq!(f.rules().timing.as_ref().unwrap().index, 1);
}

#[test]
fn effect_save_and_legendary_resistance_share_the_explicit_natural_extremes_policy() {
    for (face, dc, raw_success) in [(1, 8, true), (20, 30, false)] {
        for house in [false, true] {
            let mut f = Fixture::new();
            f.creature(1, true);
            f.state
                .rules
                .as_mut()
                .unwrap()
                .house_rules
                .ability_test_natural_extremes = house;
            let effect = f.effect(
                1,
                EffectTriggerPayload::SavingThrow {
                    ability: Ability::Wisdom,
                    dc,
                    on_success: EffectSaveEnd::TargetEffect,
                    on_failure: EffectSaveEnd::None,
                },
                vec![],
            );
            f.begin();
            // The printed Wisdom save is +7: 1 totals8; 20 totals27.
            assert_eq!(f.rules().pending.as_ref().unwrap().request.modifier, 7);
            assert_eq!(
                matches!(
                    f.rules().pending.as_ref().unwrap().ruling.basis,
                    RulingBasis::HouseRule { .. }
                ),
                house
            );
            f.roll(1, &[face]);
            let success = if house { face == 20 } else { raw_success };
            let failed = f
                .state
                .encounter
                .as_ref()
                .unwrap()
                .flow
                .as_ref()
                .unwrap()
                .resolution
                .as_ref()
                .and_then(|r| r.failed_save.as_ref());
            assert_eq!(failed.is_some(), !success);
            assert_eq!(f.rules().rolls.last().unwrap().result.dice[0].value, face);
            if !success {
                f.run(Some(1), TacticalAction::UseLegendaryResistance);
                assert_eq!(f.creature_runtime(1).legendary_resistance_spent, 1);
            } else {
                f.rejected(Some(1), TacticalAction::UseLegendaryResistance);
                assert_eq!(f.creature_runtime(1).legendary_resistance_spent, 0);
            }
            assert!(
                !f.rules()
                    .tactical_effects
                    .as_ref()
                    .unwrap()
                    .effects
                    .iter()
                    .any(|e| e.id == effect)
            );
        }
    }
}

#[test]
fn concentration_uses_the_same_opt_in_policy_before_source_resistance_or_group_loss() {
    for house in [false, true] {
        let mut f = Fixture::new();
        f.creature(1, true);
        f.state
            .rules
            .as_mut()
            .unwrap()
            .house_rules
            .ability_test_natural_extremes = house;
        let meta = f.meta(None);
        let group = EffectId::new();
        f.state = tactical_effect_adapter::apply_effect_operation(
            &f.state,
            &meta,
            &EffectLifecycleAction {
                step: 0,
                operation: EffectLifecycleOperation::BeginConcentration {
                    group: ConcentrationGroup {
                        id: group,
                        source: EffectSource {
                            definition_id: "test-focus".into(),
                            actor: f.actors[1],
                            command: meta.clone(),
                            ordinal: 0,
                        },
                        expires: TacticalEffectExpiry::Never,
                        stage: ConcentrationStage::Casting,
                    },
                },
            },
        )
        .unwrap()
        .0;
        f.state.applied_event_sequence += 1;
        f.effect(
            1,
            EffectTriggerPayload::Damage {
                dice: vec![DieSpec { count: 1, sides: 6 }],
                modifier: 55,
                damage_type: DamageType::Acid,
            },
            vec![],
        );
        f.begin();
        f.roll(0, &[5]);
        assert_eq!(
            f.rules().pending.as_ref().unwrap().request.roller,
            Some(f.actors[1])
        );
        assert_eq!(f.rules().pending.as_ref().unwrap().request.modifier, 7);
        assert_eq!(
            matches!(
                f.rules().pending.as_ref().unwrap().ruling.basis,
                RulingBasis::HouseRule { .. }
            ),
            house
        );
        f.roll(1, &[20]); // Printed +7 totals27 against the actual damage DC30.
        assert_eq!(f.rules().rolls.last().unwrap().result.dice[0].value, 20);
        assert_eq!(f.rules().rolls.last().unwrap().resolved.total, 27);
        if house {
            assert!(
                f.state
                    .encounter
                    .as_ref()
                    .unwrap()
                    .flow
                    .as_ref()
                    .unwrap()
                    .resolution
                    .is_none()
            );
            assert_eq!(f.rules().entities[&f.actors[1]].concentration, Some(group));
            f.rejected(Some(1), TacticalAction::UseLegendaryResistance);
        } else {
            assert!(f.resolution().failed_save.is_some());
            assert_eq!(f.rules().entities[&f.actors[1]].concentration, Some(group));
            f.run(Some(1), TacticalAction::DeclineLegendaryResistance);
            assert_eq!(f.rules().entities[&f.actors[1]].concentration, None);
        }
        assert_eq!(f.creature_runtime(1).legendary_resistance_spent, 0);
    }
}

#[test]
fn legendary_resistance_pauses_before_consequences_preserves_raw_faces_and_creature_authority() {
    let mut f = Fixture::new();
    f.creature(1, true);
    let effect = f.effect(1, resistance_save(Ability::Wisdom), vec![]);
    f.begin();
    f.roll(1, &[1]);
    let failed = f.resolution().failed_save.clone().unwrap();
    assert_eq!(failed.result.as_ref().unwrap().dice[0].value, 1);
    assert_eq!(f.creature_runtime(1).legendary_resistance_spent, 0);
    assert!(
        f.rules()
            .tactical_effects
            .as_ref()
            .unwrap()
            .effects
            .iter()
            .any(|e| e.id == effect)
    );
    f.rejected(Some(0), TacticalAction::UseLegendaryResistance);
    f.rejected(Some(0), TacticalAction::DeclineLegendaryResistance);
    f.rejected(Some(0), TacticalAction::EndTurn);
    let stale = f.meta(Some(1));
    f.run(Some(1), TacticalAction::UseLegendaryResistance);
    assert_eq!(f.creature_runtime(1).legendary_resistance_spent, 1);
    assert_eq!(
        f.creature_runtime(1).legendary_resistance_rolls,
        vec![failed.pending.key.request_id()]
    );
    assert!(
        !f.rules()
            .tactical_effects
            .as_ref()
            .unwrap()
            .effects
            .iter()
            .any(|e| e.id == effect)
    );
    assert_eq!(
        f.rules().rolls.last().unwrap().result,
        failed.result.unwrap()
    );
    assert!(
        resolve_tactical(
            &f.state,
            &stale,
            &TacticalAction::UseLegendaryResistance,
            &f.pack
        )
        .is_err()
    );
    f.rejected(Some(1), TacticalAction::UseLegendaryResistance);
}

#[test]
fn legendary_resistance_handles_voluntary_and_automatic_failure_without_fictional_rolls() {
    for automatic in [false, true] {
        let mut f = Fixture::new();
        f.creature(1, true);
        let effect = f.effect(
            1,
            resistance_save(if automatic {
                Ability::Strength
            } else {
                Ability::Wisdom
            }),
            if automatic {
                vec![Condition::Stunned]
            } else {
                vec![]
            },
        );
        f.begin();
        let count = f.rules().rolls.len();
        if !automatic {
            f.run(Some(1), TacticalAction::VoluntarilyFailSave);
        }
        assert!(
            f.resolution()
                .failed_save
                .as_ref()
                .unwrap()
                .result
                .is_none()
        );
        assert_eq!(f.rules().rolls.len(), count);
        f.run(Some(1), TacticalAction::UseLegendaryResistance);
        assert_eq!(f.rules().rolls.len(), count);
        assert_eq!(f.creature_runtime(1).legendary_resistance_spent, 1);
        assert!(
            !f.rules()
                .tactical_effects
                .as_ref()
                .unwrap()
                .effects
                .iter()
                .any(|e| e.id == effect)
        );
    }
}

#[test]
fn declining_resistance_applies_failure_without_spending_and_rejects_forged_pause() {
    let mut f = Fixture::new();
    f.creature(1, true);
    let effect = f.effect(1, resistance_save(Ability::Wisdom), vec![]);
    f.begin();
    f.roll(1, &[1]);
    for mutation in 0..5 {
        let mut corrupt = f.state.clone();
        let r = corrupt
            .encounter
            .as_mut()
            .unwrap()
            .flow
            .as_mut()
            .unwrap()
            .resolution
            .as_mut()
            .unwrap();
        let failed = r.failed_save.as_mut().unwrap();
        match mutation {
            0 => failed.result.as_mut().unwrap().dice[0].value = 20,
            1 => failed.result = None,
            2 => failed.pending.key.occurrence += 1,
            3 => failed.resolved_by.issuer = CommandIssuer::Player(f.players[0]),
            4 => {
                r.failed_save = None;
            }
            _ => unreachable!(),
        }
        assert!(
            validate_tactical_state(&corrupt).is_err(),
            "mutation {mutation}"
        );
    }
    f.run(Some(1), TacticalAction::DeclineLegendaryResistance);
    assert_eq!(f.creature_runtime(1).legendary_resistance_spent, 0);
    assert!(f.creature_runtime(1).legendary_resistance_rolls.is_empty());
    assert!(
        f.rules()
            .tactical_effects
            .as_ref()
            .unwrap()
            .effects
            .iter()
            .any(|e| e.id == effect)
    );
    assert!(
        f.rules()
            .tactical_effects
            .as_ref()
            .unwrap()
            .pending
            .is_empty()
    );
}

#[test]
fn recharge_follows_own_start_preserves_triggering_player_and_raw_history_before_actions() {
    let mut f = Fixture::new();
    f.creature(1, false);
    f.creature_runtime_mut(1).recharge[0].available = false;
    // Exhausted legendary actions provide no after-turn pause; own Start refreshes them.
    f.creature_runtime_mut(1).legendary_spent = 3;
    f.begin();
    assert!(f.rules().pending.is_none());
    let end = f.run(Some(0), TacticalAction::EndTurn);
    let pending = f.rules().pending.clone().unwrap();
    let PendingPurpose::TacticalResolution { key, .. } = pending.purpose else {
        panic!("expected source recharge")
    };
    assert_eq!(key.role, TacticalRollRole::CreatureRecharge);
    assert_eq!(key.origin, end.meta.id);
    assert_eq!(pending.request.dice, vec![DieSpec { count: 1, sides: 6 }]);
    assert_eq!(pending.request.visibility, RollVisibility::Secret);
    assert_eq!(f.creature_runtime(1).legendary_spent, 0);
    assert_eq!(
        f.creature_runtime(1).recharge[0]
            .pending
            .as_ref()
            .unwrap()
            .origin,
        end.meta
    );
    f.rejected(None, TacticalAction::StartAttackAction);
    f.rejected(None, TacticalAction::VoluntarilyFailSave);
    let raw = f.raw(&[6]);
    f.rejected(
        Some(0),
        TacticalAction::SubmitRoll {
            result: raw.clone(),
        },
    );
    let event = f.run(
        None,
        TacticalAction::SubmitRoll {
            result: raw.clone(),
        },
    );
    let record = f.creature_runtime(1).recharge[0]
        .last_roll
        .as_ref()
        .unwrap();
    assert_eq!(record.result, raw);
    assert_eq!(record.accepted_by, event.meta);
    assert_eq!(record.ticket.origin, end.meta);
    assert!(f.creature_runtime(1).recharge[0].available);
    assert_eq!(f.rules().rolls.last().unwrap().result, raw);
    f.run(None, TacticalAction::StartAttackAction);
}

#[test]
fn restored_recharge_cannot_omit_work_change_origin_or_forge_accepted_history() {
    let mut f = Fixture::new();
    f.creature(1, false);
    f.creature_runtime_mut(1).recharge[0].available = false;
    f.creature_runtime_mut(1).legendary_spent = 3;
    f.begin();
    f.run(Some(0), TacticalAction::EndTurn);
    for mutation in 0..4 {
        let mut corrupt = f.state.clone();
        match mutation {
            0 => {
                corrupt
                    .encounter
                    .as_mut()
                    .unwrap()
                    .flow
                    .as_mut()
                    .unwrap()
                    .resolution = None
            }
            1 => {
                corrupt
                    .rules
                    .as_mut()
                    .unwrap()
                    .tactical_creatures
                    .as_mut()
                    .unwrap()
                    .runtime[0]
                    .recharge[0]
                    .pending
                    .as_mut()
                    .unwrap()
                    .origin
                    .id = CommandId::new()
            }
            2 => {
                corrupt
                    .rules
                    .as_mut()
                    .unwrap()
                    .tactical_creatures
                    .as_mut()
                    .unwrap()
                    .runtime[0]
                    .observed_turn
                    .as_mut()
                    .unwrap()
                    .number += 1
            }
            3 => {
                corrupt
                    .rules
                    .as_mut()
                    .unwrap()
                    .pending
                    .as_mut()
                    .unwrap()
                    .request
                    .modifier = 1
            }
            _ => unreachable!(),
        }
        assert!(
            validate_tactical_state(&corrupt).is_err(),
            "mutation {mutation}"
        );
    }
    let result = f.raw(&[1]);
    f.run(None, TacticalAction::SubmitRoll { result });
    assert!(!f.creature_runtime(1).recharge[0].available);
    let mut corrupt = f.state.clone();
    corrupt
        .rules
        .as_mut()
        .unwrap()
        .tactical_creatures
        .as_mut()
        .unwrap()
        .runtime[0]
        .recharge[0]
        .last_roll
        .as_mut()
        .unwrap()
        .result
        .dice[0]
        .value = 6;
    assert!(validate_tactical_state(&corrupt).is_err());
}

#[test]
fn legendary_window_waits_until_end_effects_and_disengage_expiry_then_controller_declines() {
    let mut f = Fixture::new();
    f.creature(1, true);
    f.effect_at(
        1,
        EffectTriggerPayload::SavingThrow {
            ability: Ability::Wisdom,
            dc: 1,
            on_success: EffectSaveEnd::TargetEffect,
            on_failure: EffectSaveEnd::None,
        },
        vec![Condition::Stunned],
        TurnBoundary::End,
    );
    f.begin();
    f.run(Some(0), TacticalAction::Disengage);
    f.run(Some(0), TacticalAction::EndTurn);
    assert!(f.resolution().legendary_window.is_none());
    assert!(f.rules().pending.is_some());
    f.rejected(Some(1), TacticalAction::DeclineLegendaryAction);
    f.roll(1, &[20]);
    assert!(f.resolution().legendary_window.is_some());
    assert!(
        f.state
            .encounter
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .budget
            .disengaged
            .is_none()
    );
    assert_eq!(f.rules().timing.as_ref().unwrap().turn_number, 1);
    assert!(!active_conditions(f.rules(), f.actors[1]).contains(&Condition::Stunned));
    f.rejected(Some(0), TacticalAction::DeclineLegendaryAction);
    f.rejected(Some(0), TacticalAction::EndTurn);
    f.run(Some(1), TacticalAction::DeclineLegendaryAction);
    assert_eq!(f.rules().timing.as_ref().unwrap().turn_number, 2);
    assert_eq!(f.creature_runtime(1).legendary_spent, 0);
    f.rejected(Some(1), TacticalAction::DeclineLegendaryAction);
}

#[test]
fn multiple_after_turn_opportunities_require_host_order_and_each_creatures_explicit_choice() {
    use dmd_rules::tactical_creatures::*;
    let mut f = Fixture::new();
    f.creature(1, false);
    let third = EntityId::new();
    let mut world = f.state.entities[&f.actors[1]].clone();
    world.id = third;
    f.state.entities.insert(third, world);
    f.state
        .scenes
        .values_mut()
        .next()
        .unwrap()
        .presences
        .push(ScenePresence {
            entity_id: third,
            role: PresenceRole::Participant,
        });
    let built = build_creature(
        &f.state,
        &f.meta(None),
        third,
        &CreatureBuildChoice {
            definition_id: "adult-red-dragon".into(),
            size: CreatureSize::Huge,
            additional_languages: vec![],
            hit_points: CreatureHitPointChoice::Average,
            controller: CreatureController::Host,
            in_lair: false,
        },
    )
    .unwrap();
    let mut participant = f.state.encounter.as_ref().unwrap().participants[1].clone();
    participant.entity_id = third;
    participant.position.y = 60;
    f.state
        .encounter
        .as_mut()
        .unwrap()
        .participants
        .push(participant);
    let rules = f.state.rules.as_mut().unwrap();
    rules.entities.insert(third, built.mechanics);
    let creatures = rules.tactical_creatures.as_mut().unwrap();
    creatures.profiles.push(built.profile);
    creatures.runtime.push(built.runtime);
    f.state.applied_event_sequence += 1;
    let actors = [f.actors[0], f.actors[1], third];
    f.run(
        None,
        TacticalAction::Begin {
            combatants: actors
                .into_iter()
                .enumerate()
                .map(|(i, actor)| TacticalCombatant {
                    actor,
                    source: if i == 0 {
                        TacticalSource::Character
                    } else {
                        TacticalSource::Creature {
                            definition_id: "adult-red-dragon".into(),
                        }
                    },
                    surprised: false,
                })
                .collect(),
            groups: vec![
                InitiativeGroup {
                    actors: vec![f.actors[0]],
                    request_id: RollRequestId::new(),
                },
                InitiativeGroup {
                    actors: vec![f.actors[1], third],
                    request_id: RollRequestId::new(),
                },
            ],
        },
    );
    for value in [18, 3] {
        let result = f.raw(&[value]);
        f.run(None, TacticalAction::SubmitRoll { result });
    }
    f.run(
        None,
        TacticalAction::ProposeInitiativeTie {
            order: vec![f.actors[1], third],
        },
    );
    f.run(Some(0), TacticalAction::EndTurn);
    assert_eq!(f.resolution().frames.last().unwrap().len(), 2);
    let occurrence = f
        .resolution()
        .frames
        .last()
        .unwrap()
        .iter()
        .find(|w| matches!(w.kind,TacticalWorkKind::LegendaryWindow {actor} if actor==third))
        .unwrap()
        .occurrence;
    f.rejected(Some(0), TacticalAction::ChooseTurnWork { occurrence });
    f.run(None, TacticalAction::ChooseTurnWork { occurrence });
    assert!(
        matches!(f.resolution().legendary_window.as_ref().unwrap().work.kind,TacticalWorkKind::LegendaryWindow {actor} if actor==third)
    );
    let mut corrupt = f.state.clone();
    corrupt
        .encounter
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .resolution
        .as_mut()
        .unwrap()
        .legendary_window = None;
    assert!(validate_tactical_state(&corrupt).is_err());
    f.run(None, TacticalAction::DeclineLegendaryAction);
    assert!(
        matches!(f.resolution().legendary_window.as_ref().unwrap().work.kind,TacticalWorkKind::LegendaryWindow {actor} if actor==f.actors[1])
    );
    f.run(None, TacticalAction::DeclineLegendaryAction);
    assert_eq!(f.rules().timing.as_ref().unwrap().turn_number, 2);
    assert_eq!(
        f.rules()
            .tactical_creatures
            .as_ref()
            .unwrap()
            .runtime(third)
            .unwrap()
            .legendary_spent,
        0
    );
}

struct Fixture {
    state: CampaignState,
    pack: RulesPack,
    actors: [EntityId; 2],
    players: [PlayerId; 2],
}
impl Fixture {
    fn new() -> Self {
        let pack =
            RulesPack::from_json(include_str!("../../../content/srd-5.2.1/kernel.json")).unwrap();
        let mut state = CampaignState::empty(
            Campaign {
                id: CampaignId::new(),
                display_name: "Independent source turn fixture".into(),
                status: CampaignStatus::Active,
                world_seed: 1,
                ruleset: VersionedRef {
                    id: pack.id.clone(),
                    version: pack.version.clone(),
                },
                content_packs: vec![],
            },
            WorldClock {
                now: WorldInstant(0),
                calendar_id: "seconds".into(),
            },
        );
        let actors = [EntityId::new(), EntityId::new()];
        let players = [PlayerId::new(), PlayerId::new()];
        let location = LocationId::new();
        let scene = SceneId::new();
        state.locations.insert(
            location,
            Location {
                id: location,
                campaign_id: state.campaign_id(),
                display_name: "Field".into(),
                parent_location_id: None,
            },
        );
        for (actor, player) in actors.into_iter().zip(players) {
            state.players.insert(
                player,
                Player {
                    id: player,
                    campaign_id: state.campaign_id(),
                    display_name: "Controller".into(),
                },
            );
            state.entities.insert(
                actor,
                WorldEntity {
                    id: actor,
                    campaign_id: state.campaign_id(),
                    display_name: "Adventurer".into(),
                    kind: EntityKind::Character,
                    existence: EntityExistence::Present,
                    location_id: Some(location),
                },
            );
            let id = CharacterId::new();
            state.characters.insert(
                id,
                Character {
                    id,
                    campaign_id: state.campaign_id(),
                    entity_id: actor,
                    controlling_player_id: Some(player),
                    display_name: "Adventurer".into(),
                    status: CharacterStatus::Active,
                },
            );
        }
        state.scenes.insert(
            scene,
            Scene {
                id: scene,
                campaign_id: state.campaign_id(),
                location_id: location,
                mode: SceneMode::Combat,
                status: SceneStatus::Active,
                started_at: WorldInstant(0),
                presences: actors
                    .into_iter()
                    .map(|entity_id| ScenePresence {
                        entity_id,
                        role: PresenceRole::Participant,
                    })
                    .collect(),
            },
        );
        state.rules = Some(RulesState {
            tactical_recovery: None,
            tactical_creatures: None,
            tactical_inventory: None,
            tactical_effects: None,
            pack_id: pack.id.clone(),
            pack_version: pack.version.clone(),
            entities: actors
                .into_iter()
                .map(|id| (id, MechanicalEntity::basic(id)))
                .collect::<HashMap<_, _>>(),
            house_rules: HouseRules::default(),
            effects: vec![],
            pending: None,
            rolls: vec![],
            cancelled_roll_ids: vec![],
            rulings: vec![],
            timing: None,
            rests: vec![],
            completed_short_rests: vec![],
            permission: None,
        });
        let mut f = Self {
            state,
            pack,
            actors,
            players,
        };
        let point = |x, y, z| SpatialPoint { x, y, z };
        let encounter = TacticalEncounter {
            id: EncounterId::new(),
            scene_id: scene,
            battlefield: Battlefield {
                bounds: SpatialBox {
                    min: point(0, 0, 0),
                    max: point(100, 100, 100),
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
                .map(|(i, entity_id)| TacticalParticipant {
                    entity_id,
                    position: point(10 + 30 * i as i32, 10, 0),
                    size: CreatureSize::Medium,
                    public_label: "Adventurer".into(),
                    height: 12,
                    reach: 10,
                    movement: MovementProfile {
                        walk: 60,
                        climb: None,
                        swim: None,
                        fly: Some(120),
                        burrow: None,
                        hover: false,
                    },
                    senses: Senses::default(),
                    allies: vec![],
                    enemies: vec![],
                })
                .collect(),
            knowledge: vec![],
            origin: f.meta(None),
            geometry_ruling: Ruling {
                basis: RulingBasis::GmAdjudication,
                reason: "Explicit flat field".into(),
            },
            flow: None,
        };
        f.run(
            None,
            TacticalAction::Establish {
                encounter: Box::new(encounter),
            },
        );
        f
    }
    fn meta(&self, actor: Option<usize>) -> CommandMeta {
        CommandMeta {
            id: CommandId::new(),
            campaign_id: self.state.campaign_id(),
            session_id: None,
            issuer: actor.map_or(CommandIssuer::Admin, |i| {
                CommandIssuer::Player(self.players[i])
            }),
            actor: actor.map(|i| AgentRef::Entity(self.actors[i])),
            expected_event_sequence: self.state.applied_event_sequence,
        }
    }
    fn rules(&self) -> &RulesState {
        self.state.rules.as_ref().unwrap()
    }
    fn entity_mut(&mut self, index: usize) -> &mut MechanicalEntity {
        self.state
            .rules
            .as_mut()
            .unwrap()
            .entities
            .get_mut(&self.actors[index])
            .unwrap()
    }
    fn creature(&mut self, index: usize, controlled: bool) {
        use dmd_rules::tactical_creatures::*;
        let actor = self.actors[index];
        self.state.characters.retain(|_, c| c.entity_id != actor);
        self.state.entities.get_mut(&actor).unwrap().kind = EntityKind::Creature;
        self.state.rules.as_mut().unwrap().entities.remove(&actor);
        let built = build_creature(
            &self.state,
            &self.meta(None),
            actor,
            &CreatureBuildChoice {
                definition_id: "adult-red-dragon".into(),
                size: CreatureSize::Huge,
                additional_languages: vec![],
                hit_points: CreatureHitPointChoice::Average,
                controller: if controlled {
                    CreatureController::Player(self.players[index])
                } else {
                    CreatureController::Host
                },
                in_lair: false,
            },
        )
        .unwrap();
        let participant = self
            .state
            .encounter
            .as_mut()
            .unwrap()
            .participants
            .iter_mut()
            .find(|p| p.entity_id == actor)
            .unwrap();
        participant.size = CreatureSize::Huge;
        participant.height = 30;
        participant.movement = built.movement;
        participant.senses = built.senses;
        // Keep two Huge actors disjoint when testing independent after-turn windows.
        participant.position.x = if index == 0 { 20 } else { 70 };
        self.state
            .encounter
            .as_mut()
            .unwrap()
            .battlefield
            .bounds
            .max
            .x = 120;
        let rules = self.state.rules.as_mut().unwrap();
        rules.entities.insert(actor, built.mechanics);
        let current = rules.tactical_creatures.get_or_insert(TacticalCreatures {
            schema_version: 1,
            profiles: vec![],
            runtime: vec![],
        });
        current.profiles.push(built.profile);
        current.runtime.push(built.runtime);
        self.state.applied_event_sequence += 1;
    }
    fn resolution(&self) -> &TacticalResolution {
        self.state
            .encounter
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .resolution
            .as_ref()
            .unwrap()
    }
    fn creature_runtime(&self, index: usize) -> &CreatureRuntime {
        self.rules()
            .tactical_creatures
            .as_ref()
            .unwrap()
            .runtime(self.actors[index])
            .unwrap()
    }
    fn creature_runtime_mut(&mut self, index: usize) -> &mut CreatureRuntime {
        self.state
            .rules
            .as_mut()
            .unwrap()
            .tactical_creatures
            .as_mut()
            .unwrap()
            .runtime
            .iter_mut()
            .find(|r| r.actor == self.actors[index])
            .unwrap()
    }
    fn run(&mut self, actor: Option<usize>, action: TacticalAction) -> TacticalEvent {
        let meta = self.meta(actor);
        let before = self.state.clone();
        let transition = resolve_tactical(&self.state, &meta, &action, &self.pack).unwrap();
        let event: TacticalEvent =
            serde_json::from_slice(&serde_json::to_vec(&transition.event).unwrap()).unwrap();
        assert_eq!(
            replay_tactical(&before, &event, &self.pack).unwrap(),
            transition
        );
        self.state =
            serde_json::from_slice(&serde_json::to_vec(&transition.next_state).unwrap()).unwrap();
        self.state.applied_event_sequence += 1;
        validate_state(&self.state, &self.pack).unwrap();
        validate_tactical_state(&self.state).unwrap();
        assert!(
            self.state.validate().is_empty(),
            "{:?}",
            self.state.validate()
        );
        event
    }
    fn rejected(&self, actor: Option<usize>, action: TacticalAction) {
        let before = self.state.clone();
        assert!(resolve_tactical(&self.state, &self.meta(actor), &action, &self.pack).is_err());
        assert_eq!(self.state, before);
    }
    fn raw(&self, values: &[u16]) -> RollResult {
        let request = &self.rules().pending.as_ref().unwrap().request;
        RollResult {
            request_id: request.id,
            source: RollSource::Physical,
            dice: values
                .iter()
                .map(|v| DieResult {
                    sides: request.dice[0].sides,
                    value: *v,
                })
                .collect(),
        }
    }
    fn roll(&mut self, actor: usize, values: &[u16]) {
        let result = self.raw(values);
        self.run(Some(actor), TacticalAction::SubmitRoll { result });
    }
    fn begin(&mut self) {
        self.run(
            None,
            TacticalAction::Begin {
                combatants: self
                    .actors
                    .into_iter()
                    .map(|actor| TacticalCombatant {
                        actor,
                        source: self
                            .rules()
                            .tactical_creatures
                            .as_ref()
                            .and_then(|c| c.profile(actor))
                            .map_or(TacticalSource::Character, |p| TacticalSource::Creature {
                                definition_id: p.source.definition_id.clone(),
                            }),
                        surprised: false,
                    })
                    .collect(),
                groups: self
                    .actors
                    .into_iter()
                    .map(|actor| InitiativeGroup {
                        actors: vec![actor],
                        request_id: RollRequestId::new(),
                    })
                    .collect(),
            },
        );
        for (index, value) in [(0, 18), (1, 3)] {
            let values =
                if self.rules().pending.as_ref().unwrap().request.mode == RollMode::Disadvantage {
                    vec![value, value]
                } else {
                    vec![value]
                };
            let result = self.raw(&values);
            let actor = self
                .rules()
                .tactical_creatures
                .as_ref()
                .and_then(|c| c.profile(self.actors[index]))
                .is_none()
                .then_some(index);
            self.run(actor, TacticalAction::SubmitRoll { result });
        }
    }
    fn effect(
        &mut self,
        target: usize,
        payload: EffectTriggerPayload,
        conditions: Vec<Condition>,
    ) -> EffectId {
        self.effect_at(target, payload, conditions, TurnBoundary::Start)
    }
    fn effect_at(
        &mut self,
        target: usize,
        payload: EffectTriggerPayload,
        conditions: Vec<Condition>,
        boundary: TurnBoundary,
    ) -> EffectId {
        let meta = self.meta(None);
        let id = EffectId::new();
        let effect = TacticalEffect {
            id,
            source: EffectSource {
                definition_id: "test-source".into(),
                actor: self.actors[0],
                command: meta.clone(),
                ordinal: 0,
            },
            established_at: None,
            target: TacticalEffectTarget::Creature(self.actors[target]),
            concentration_group: None,
            expires: TacticalEffectExpiry::Never,
            overlap: None,
            conditions: conditions
                .into_iter()
                .map(|condition| EffectCondition {
                    id: EffectId::new(),
                    condition,
                })
                .collect(),
            triggers: vec![EffectTriggerRule {
                event: EffectTriggerEvent::Turn {
                    subject: EffectSubject::Source,
                    boundary,
                },
                frequency: EffectTriggerFrequency::EveryOccurrence,
                payload,
            }],
        };
        self.state = tactical_effect_adapter::apply_effect_operation(
            &self.state,
            &meta,
            &EffectLifecycleAction {
                step: 0,
                operation: EffectLifecycleOperation::Install {
                    effects: vec![effect],
                },
            },
        )
        .unwrap()
        .0;
        self.state.applied_event_sequence += 1;
        id
    }
    fn choice_for_target(&self, index: usize) -> u16 {
        let r = self
            .state
            .encounter
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .resolution
            .as_ref()
            .unwrap();
        r.frames
            .last()
            .unwrap()
            .iter()
            .find(|work| match work.kind {
                TacticalWorkKind::Effect { ticket } => self
                    .rules()
                    .tactical_effects
                    .as_ref()
                    .unwrap()
                    .pending
                    .iter()
                    .any(|t| t.id == ticket && t.target == self.actors[index]),
                _ => false,
            })
            .unwrap()
            .occurrence
    }
}

#[test]
fn first_turn_death_save_precedes_actions_survives_restart_and_rejects_foreign_stale_input() {
    let mut f = Fixture::new();
    f.entity_mut(0).hp = 0;
    f.entity_mut(0).prone = true;
    f.begin();
    assert!(matches!(
        f.rules().pending.as_ref().unwrap().purpose,
        PendingPurpose::TacticalResolution {
            key: TacticalRollKey {
                role: TacticalRollRole::DeathSave,
                ..
            },
            ..
        }
    ));
    f.rejected(
        Some(0),
        TacticalAction::Dash {
            speed: DashSpeed::Speed,
        },
    );
    let result = f.raw(&[20]);
    f.rejected(
        Some(1),
        TacticalAction::SubmitRoll {
            result: result.clone(),
        },
    );
    let stale = f.meta(Some(0));
    f.roll(0, &[20]);
    assert_eq!(f.rules().entities[&f.actors[0]].hp, 1);
    assert!(f.rules().entities[&f.actors[0]].prone);
    assert!(matches!(
        resolve_tactical(
            &f.state,
            &stale,
            &TacticalAction::SubmitRoll { result },
            &f.pack
        ),
        Err(RulesError::Stale)
    ));
    f.run(Some(0), TacticalAction::StandProne);
    assert_eq!(
        f.state
            .encounter
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .budget
            .movement_spent,
        30
    );
}

#[test]
fn death_natural_one_then_voluntary_failure_kills_without_fabricated_dice() {
    let mut f = Fixture::new();
    f.entity_mut(0).hp = 0;
    f.entity_mut(0).prone = true;
    f.begin();
    f.roll(0, &[1]);
    assert_eq!(f.rules().entities[&f.actors[0]].death.failures, 2);
    f.run(Some(0), TacticalAction::EndTurn);
    f.run(Some(1), TacticalAction::EndTurn);
    let count = f.rules().rolls.len();
    f.run(Some(0), TacticalAction::VoluntarilyFailSave);
    assert_eq!(f.rules().rolls.len(), count);
    assert!(f.rules().entities[&f.actors[0]].death.dead);
    assert_eq!(
        f.state.entities[&f.actors[0]].existence,
        EntityExistence::Dead
    );
    assert_eq!(
        f.state
            .encounter
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .save_decisions
            .len(),
        1
    );
}

#[test]
fn stabilization_requests_raw_d4_and_retains_source_cause_and_wake_time() {
    let mut f = Fixture::new();
    f.entity_mut(0).hp = 0;
    f.entity_mut(0).prone = true;
    f.entity_mut(0).death.successes = 2;
    f.begin();
    f.roll(0, &[10]);
    assert_eq!(
        f.rules().pending.as_ref().unwrap().request.dice,
        vec![DieSpec { count: 1, sides: 4 }]
    );
    f.rejected(Some(0), TacticalAction::VoluntarilyFailSave);
    f.roll(0, &[4]);
    let recovery = &f.rules().tactical_recovery.as_ref().unwrap()[&f.actors[0]];
    assert_eq!(
        dmd_rules::tactical_damage::stable_wake_at(recovery.stable.as_ref().unwrap()).unwrap(),
        Some(WorldInstant(14400))
    );
    f.run(Some(0), TacticalAction::EndTurn);
    f.run(Some(1), TacticalAction::EndTurn);
    assert!(f.rules().pending.is_none());
    assert_eq!(f.state.clock.now, WorldInstant(6));
}

#[test]
fn turn_budgets_round_time_selected_dash_speed_dodge_and_attack_window_are_single_authority() {
    let mut f = Fixture::new();
    f.begin();
    f.rejected(Some(1), TacticalAction::EndTurn);
    f.run(
        Some(0),
        TacticalAction::Dash {
            speed: DashSpeed::Fly,
        },
    );
    let flow = f.state.encounter.as_ref().unwrap().flow.as_ref().unwrap();
    assert_eq!(
        tactical_budget::movement_remaining(
            &flow.budget,
            DashSpeed::Speed,
            &f.state.encounter.as_ref().unwrap().participants[0].movement
        )
        .unwrap(),
        60
    );
    assert_eq!(
        tactical_budget::movement_remaining(
            &flow.budget,
            DashSpeed::Fly,
            &f.state.encounter.as_ref().unwrap().participants[0].movement
        )
        .unwrap(),
        240
    );
    f.rejected(Some(0), TacticalAction::Dodge);
    f.run(Some(0), TacticalAction::EndTurn);
    assert_eq!(f.state.clock.now, WorldInstant(0));
    f.run(Some(1), TacticalAction::Dodge);
    f.run(Some(1), TacticalAction::EndTurn);
    assert_eq!(f.state.clock.now, WorldInstant(6));
    assert_eq!(
        f.state
            .encounter
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .dodges
            .len(),
        1
    );
    let event = f.run(Some(0), TacticalAction::StartAttackAction);
    let budget = &f
        .state
        .encounter
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .budget;
    assert_eq!(budget.attacks_remaining, 1);
    assert_eq!(budget.attack_window.unwrap().id, event.meta.id);
    f.run(Some(0), TacticalAction::EndTurn);
    assert!(
        f.state
            .encounter
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .dodges
            .is_empty()
    );
    f.run(Some(1), TacticalAction::Disengage);
    assert!(
        f.state
            .encounter
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .budget
            .disengaged
            .is_some()
    );
}

#[test]
fn simultaneous_effect_order_and_independent_roller_authority_survive_each_serialized_pause() {
    let mut f = Fixture::new();
    let save = || EffectTriggerPayload::SavingThrow {
        ability: Ability::Wisdom,
        dc: 12,
        on_success: EffectSaveEnd::TargetEffect,
        on_failure: EffectSaveEnd::TargetEffect,
    };
    f.effect(0, save(), vec![]);
    f.effect(1, save(), vec![]);
    f.begin();
    let chosen = f.choice_for_target(1);
    assert!(f.rules().pending.is_none());
    f.rejected(
        Some(1),
        TacticalAction::ChooseTurnWork { occurrence: chosen },
    );
    f.run(
        Some(0),
        TacticalAction::ChooseTurnWork { occurrence: chosen },
    );
    assert_eq!(
        f.rules().pending.as_ref().unwrap().request.roller,
        Some(f.actors[1])
    );
    let result = f.raw(&[14]);
    f.rejected(Some(0), TacticalAction::SubmitRoll { result });
    f.run(Some(1), TacticalAction::VoluntarilyFailSave);
    assert_eq!(
        f.rules().pending.as_ref().unwrap().request.roller,
        Some(f.actors[0])
    );
    f.roll(0, &[14]);
    assert!(f.rules().pending.is_none());
    assert!(
        f.rules()
            .tactical_effects
            .as_ref()
            .unwrap()
            .effects
            .is_empty()
    );
}

#[test]
fn automatic_strength_save_failure_has_no_roll_or_controller_fabrication() {
    let mut f = Fixture::new();
    f.effect(
        1,
        EffectTriggerPayload::SavingThrow {
            ability: Ability::Strength,
            dc: 10,
            on_success: EffectSaveEnd::None,
            on_failure: EffectSaveEnd::TargetEffect,
        },
        vec![Condition::Stunned],
    );
    f.begin();
    assert!(f.rules().pending.is_none());
    assert_eq!(f.rules().rolls.len(), 2);
    let decisions = &f
        .state
        .encounter
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .save_decisions;
    assert_eq!(decisions.len(), 1);
    assert_eq!(decisions[0].failure, TacticalSaveFailure::Automatic);
}

#[test]
fn effect_damage_preserves_inspiration_raw_faces_and_queues_target_concentration() {
    let mut f = Fixture::new();
    f.entity_mut(0).heroic_inspiration = true;
    f.entity_mut(1).max_hp = 50;
    f.entity_mut(1).hp = 50;
    let meta = f.meta(None);
    let group = EffectId::new();
    f.state = tactical_effect_adapter::apply_effect_operation(
        &f.state,
        &meta,
        &EffectLifecycleAction {
            step: 0,
            operation: EffectLifecycleOperation::BeginConcentration {
                group: ConcentrationGroup {
                    id: group,
                    source: EffectSource {
                        definition_id: "test-focus".into(),
                        actor: f.actors[1],
                        command: meta.clone(),
                        ordinal: 0,
                    },
                    expires: TacticalEffectExpiry::Never,
                    stage: ConcentrationStage::Casting,
                },
            },
        },
    )
    .unwrap()
    .0;
    let source = f.rules().tactical_effects.as_ref().unwrap().groups[0]
        .source
        .clone();
    f.state = tactical_effect_adapter::apply_effect_operation(
        &f.state,
        &meta,
        &EffectLifecycleAction {
            step: 1,
            operation: EffectLifecycleOperation::Install {
                effects: vec![TacticalEffect {
                    id: EffectId::new(),
                    source,
                    established_at: None,
                    target: TacticalEffectTarget::Creature(f.actors[1]),
                    concentration_group: Some(group),
                    expires: TacticalEffectExpiry::Never,
                    overlap: None,
                    conditions: vec![],
                    triggers: vec![],
                }],
            },
        },
    )
    .unwrap()
    .0;
    f.state.applied_event_sequence += 1;
    f.effect(
        1,
        EffectTriggerPayload::Damage {
            dice: vec![DieSpec { count: 1, sides: 6 }],
            modifier: 0,
            damage_type: DamageType::Fire,
        },
        vec![],
    );
    f.begin();
    let original = f.raw(&[1]);
    f.run(
        Some(0),
        TacticalAction::SubmitRollWithInspiration {
            result: original.clone(),
            die_index: 0,
            replacement: DieResult { sides: 6, value: 4 },
        },
    );
    assert_eq!(f.rules().entities[&f.actors[1]].hp, 46);
    assert!(!f.rules().entities[&f.actors[0]].heroic_inspiration);
    assert_eq!(
        f.rules().rolls.last().unwrap().original_result,
        Some(original)
    );
    assert_eq!(
        f.rules().pending.as_ref().unwrap().request.roller,
        Some(f.actors[1])
    );
    let mut corrupt = f.state.clone();
    let work = &mut corrupt
        .encounter
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .resolution
        .as_mut()
        .unwrap()
        .pending
        .as_mut()
        .unwrap()
        .work;
    let TacticalWorkKind::ConcentrationSave { group, .. } = &mut work.kind else {
        panic!("expected concentration")
    };
    *group = EffectId::new();
    assert!(validate_tactical_state(&corrupt).is_err());
    f.roll(1, &[9]);
    assert_eq!(f.rules().entities[&f.actors[1]].concentration, None);
}

#[test]
fn restored_pending_modifier_role_occurrence_origin_and_missing_work_are_rejected() {
    let mut f = Fixture::new();
    f.entity_mut(0).hp = 0;
    f.entity_mut(0).prone = true;
    f.begin();
    for mutation in 0..5 {
        let mut bad = f.state.clone();
        match mutation {
            0 => {
                bad.rules
                    .as_mut()
                    .unwrap()
                    .pending
                    .as_mut()
                    .unwrap()
                    .request
                    .modifier += 1
            }
            1 => {
                if let PendingPurpose::TacticalResolution { key, .. } = &mut bad
                    .rules
                    .as_mut()
                    .unwrap()
                    .pending
                    .as_mut()
                    .unwrap()
                    .purpose
                {
                    key.role = TacticalRollRole::StableRecovery;
                }
            }
            2 => {
                bad.encounter
                    .as_mut()
                    .unwrap()
                    .flow
                    .as_mut()
                    .unwrap()
                    .resolution
                    .as_mut()
                    .unwrap()
                    .pending
                    .as_mut()
                    .unwrap()
                    .work
                    .occurrence += 1
            }
            3 => {
                bad.encounter
                    .as_mut()
                    .unwrap()
                    .flow
                    .as_mut()
                    .unwrap()
                    .resolution
                    .as_mut()
                    .unwrap()
                    .origin
                    .id = CommandId::new()
            }
            _ => {
                bad.encounter
                    .as_mut()
                    .unwrap()
                    .flow
                    .as_mut()
                    .unwrap()
                    .resolution = None
            }
        }
        assert!(
            validate_state(&bad, &f.pack).is_err() || validate_tactical_state(&bad).is_err(),
            "mutation {mutation}"
        );
    }
}

#[test]
fn stunned_and_incapacitated_can_stand_but_cannot_spend_actions() {
    for condition in [Condition::Stunned, Condition::Incapacitated] {
        let mut f = Fixture::new();
        f.entity_mut(0).prone = true;
        let source = f.actors[1];
        let actor = f.actors[0];
        f.state.rules.as_mut().unwrap().effects.push(ActiveEffect {
            id: EffectId::new(),
            source,
            target: actor,
            label: "Source condition".into(),
            condition: Some(condition),
            concentration_owner: None,
            expires: Expiry::Never,
        });
        f.begin();
        f.run(Some(0), TacticalAction::StandProne);
        assert!(!f.rules().entities[&actor].prone);
        f.rejected(
            Some(0),
            TacticalAction::Dash {
                speed: DashSpeed::Speed,
            },
        );
        f.run(Some(0), TacticalAction::EndTurn);
    }
}

#[test]
fn damage_to_unconscious_drops_physical_weapon_and_shield_with_actual_cause() {
    let mut f = Fixture::new();
    let actor = f.actors[1];
    let sword = ItemId::new();
    let shield = ItemId::new();
    let meta = f.meta(None);
    for (id, definition) in [(sword, "longsword"), (shield, "shield")] {
        f.state.items.insert(
            id,
            ItemInstance {
                id,
                campaign_id: f.state.campaign_id(),
                definition_id: definition.into(),
                display_name: definition.into(),
                quantity: 1,
                owner: Ownership::Entity(actor),
                custody: Custody::Entity(actor),
                state: ItemState::Intact,
            },
        );
    }
    f.state.rules.as_mut().unwrap().tactical_inventory = Some(TacticalInventory {
        schema_version: 1,
        receipts: vec![],
        loadouts: vec![ActorEquipmentLoadout {
            actor,
            hands: WeaponLoadout {
                hands: [HandAssignment::Item(sword), HandAssignment::Item(shield)],
            },
            worn_armor: None,
            shield: Some(shield),
            command: meta,
        }],
    });
    f.entity_mut(1).armor = ArmorClass::Armor {
        base: 10,
        dexterity_cap: None,
        shield: true,
    };
    f.entity_mut(1).hp = 2;
    f.effect(
        1,
        EffectTriggerPayload::Damage {
            dice: vec![DieSpec { count: 1, sides: 6 }],
            modifier: 0,
            damage_type: DamageType::Force,
        },
        vec![],
    );
    f.begin();
    f.roll(0, &[3]);
    assert_eq!(f.rules().entities[&actor].hp, 0);
    let loadout = f
        .rules()
        .tactical_inventory
        .as_ref()
        .unwrap()
        .loadout(actor)
        .unwrap();
    assert_eq!(loadout.hands, WeaponLoadout::default());
    assert_eq!(loadout.shield, None);
    assert_eq!(loadout.command.actor, Some(AgentRef::Entity(f.actors[0])));
    let ground = &f
        .state
        .encounter
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .ground_items;
    assert_eq!(ground.len(), 2);
    for item in [sword, shield] {
        assert!(matches!(f.state.items[&item].custody, Custody::Location(_)));
        assert_eq!(f.state.items[&item].owner, Ownership::Entity(actor));
    }
}

#[test]
fn round_clock_overflow_fails_atomically_after_end_work() {
    let mut f = Fixture::new();
    f.begin();
    f.run(Some(0), TacticalAction::EndTurn);
    f.state.clock.now = WorldInstant(i64::MAX - 1);
    f.rejected(Some(1), TacticalAction::EndTurn);
}

#[test]
fn same_command_end_and_next_start_never_reuse_a_save_occurrence_identity() {
    let mut f = Fixture::new();
    f.effect(
        1,
        EffectTriggerPayload::SavingThrow {
            ability: Ability::Strength,
            dc: 10,
            on_success: EffectSaveEnd::None,
            on_failure: EffectSaveEnd::None,
        },
        vec![Condition::Stunned],
    );
    let effect = &mut f
        .state
        .rules
        .as_mut()
        .unwrap()
        .tactical_effects
        .as_mut()
        .unwrap()
        .effects[0];
    let mut start = effect.triggers[0].clone();
    effect.triggers[0].event = EffectTriggerEvent::Turn {
        subject: EffectSubject::Source,
        boundary: TurnBoundary::End,
    };
    start.event = EffectTriggerEvent::Turn {
        subject: EffectSubject::Target,
        boundary: TurnBoundary::Start,
    };
    effect.triggers.push(start);
    f.begin();
    assert!(
        f.state
            .encounter
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .save_decisions
            .is_empty()
    );
    let event = f.run(Some(0), TacticalAction::EndTurn);
    let decisions = &f
        .state
        .encounter
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .save_decisions;
    assert_eq!(decisions.len(), 2);
    assert_eq!(decisions[0].key.origin, event.meta.id);
    assert_eq!(decisions[1].key.origin, event.meta.id);
    assert_ne!(decisions[0].key.request_id(), decisions[1].key.request_id());
}

#[test]
fn begin_reconciles_legacy_unconscious_held_items_without_invented_injury_origin() {
    let mut f = Fixture::new();
    let actor = f.actors[0];
    let item = ItemId::new();
    let prior = f.meta(None);
    f.state.items.insert(
        item,
        ItemInstance {
            id: item,
            campaign_id: f.state.campaign_id(),
            definition_id: "dagger".into(),
            display_name: "Dagger".into(),
            quantity: 1,
            owner: Ownership::Entity(actor),
            custody: Custody::Entity(actor),
            state: ItemState::Intact,
        },
    );
    f.state.rules.as_mut().unwrap().tactical_inventory = Some(TacticalInventory {
        schema_version: 1,
        receipts: vec![],
        loadouts: vec![ActorEquipmentLoadout {
            actor,
            hands: WeaponLoadout {
                hands: [HandAssignment::Item(item), HandAssignment::Free],
            },
            worn_armor: None,
            shield: None,
            command: prior,
        }],
    });
    f.entity_mut(0).hp = 0;
    f.entity_mut(0).prone = true;
    let event = f.run(
        None,
        TacticalAction::Begin {
            combatants: f
                .actors
                .into_iter()
                .map(|actor| TacticalCombatant {
                    actor,
                    source: TacticalSource::Character,
                    surprised: false,
                })
                .collect(),
            groups: f
                .actors
                .into_iter()
                .map(|actor| InitiativeGroup {
                    actors: vec![actor],
                    request_id: RollRequestId::new(),
                })
                .collect(),
        },
    );
    assert_eq!(
        f.rules()
            .tactical_inventory
            .as_ref()
            .unwrap()
            .loadout(actor)
            .unwrap()
            .command,
        event.meta
    );
    let ground = &f
        .state
        .encounter
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .ground_items;
    assert_eq!(ground.len(), 1);
    assert_eq!(ground[0].origin, event.meta);
    assert!(matches!(f.state.items[&item].custody, Custody::Location(_)));
    assert_eq!(f.rules().entities[&actor].hp, 0);
}
