use super::*;

fn fixture() -> Fixture {
    let mut f = Fixture::new();
    f.state.encounter.as_mut().unwrap().participants[1]
        .position
        .x = 20;
    f.begin();
    f.zero_hp_target(false);
    f
}
fn aid(f: &Fixture, purpose: MedicinePurpose) -> TacticalAction {
    TacticalAction::FirstAid {
        target: f.actors[1],
        purpose,
    }
}

#[test]
fn first_aid_pays_action_and_resolves_owned_physical_check_then_target_recovery() {
    for face in [9, 10] {
        let mut f = fixture();
        let action = aid(&f, MedicinePurpose::Stabilize);
        f.rejected(Some(1), action.clone());
        let accepted = f.run(Some(0), action.clone());
        let request = &f.rules().pending.as_ref().unwrap().request;
        assert_eq!(request.roller, Some(f.actors[0]));
        assert_eq!(request.modifier, 0);
        assert_eq!(
            request.dice,
            vec![DieSpec {
                count: 1,
                sides: 20
            }]
        );
        let timing = f.rules().timing.as_ref().unwrap();
        assert!(timing.action_spent);
        assert!(!timing.bonus_action_spent);
        assert!(timing.reactions_spent.is_empty());
        assert!(matches!(f.rules().pending.as_ref().unwrap().purpose,
            PendingPurpose::TacticalResolution { key, .. }
            if key.role == TacticalRollRole::Medicine && key.origin == accepted.meta.id
                && key.subject == f.actors[1]));
        f.rejected(Some(0), TacticalAction::VoluntarilyFailSave);
        f.rejected(Some(0), TacticalAction::UseLegendaryResistance);
        f.rejected(
            Some(1),
            TacticalAction::SubmitRoll {
                result: f.raw(&[face]),
            },
        );
        f.rejected(Some(0), TacticalAction::EndTurn);
        let raw = f.raw(&[face]);
        f.run(
            Some(0),
            TacticalAction::SubmitRoll {
                result: raw.clone(),
            },
        );
        assert_eq!(f.rules().rolls.last().unwrap().result, raw);
        assert_eq!(f.rules().entities[&f.actors[1]].hp, 0);
        assert_eq!(f.rules().entities[&f.actors[1]].death.stable, face == 10);
        assert!(active_conditions(f.rules(), f.actors[1]).contains(&Condition::Unconscious));
        if face == 10 {
            let pending = f.rules().pending.as_ref().unwrap();
            assert_eq!(pending.request.roller, Some(f.actors[1]));
            assert_eq!(pending.request.dice, vec![DieSpec { count: 1, sides: 4 }]);
            f.rejected(
                Some(0),
                TacticalAction::SubmitRoll {
                    result: f.raw(&[2]),
                },
            );
            f.roll(1, &[2]);
            let stable = f.rules().tactical_recovery.as_ref().unwrap()[&f.actors[1]]
                .stable
                .as_ref()
                .unwrap();
            assert_eq!(
                dmd_rules::tactical_damage::stable_wake_at(stable).unwrap(),
                Some(WorldInstant(7200))
            );
        }
        assert!(f.flow().resolution.is_none());
        f.rejected(Some(0), action);
        f.rejected(Some(0), TacticalAction::Dodge);
    }
}

#[test]
fn first_aid_checks_knowledge_contact_vitality_and_paid_request_without_mutation() {
    for case in 0..6 {
        let mut f = fixture();
        match case {
            0 => {
                f.state.encounter.as_mut().unwrap().participants[1]
                    .position
                    .x = 40
            }
            1 => {
                f.state
                    .encounter
                    .as_mut()
                    .unwrap()
                    .battlefield
                    .ambient_light = LightLevel::Darkness
            }
            2 => f.zero_hp_target(true),
            3 => f.entity_mut(1).hp = 1,
            4 => f.entity_mut(0).hp = 0,
            _ => {
                f.run(Some(0), TacticalAction::Dodge);
            }
        }
        f.rejected(Some(0), aid(&f, MedicinePurpose::Stabilize));
    }
    let mut f = fixture();
    f.rejected(Some(0), aid(&f, MedicinePurpose::EndKnockout));
    f.run(Some(0), aid(&f, MedicinePurpose::Stabilize));
    for case in 0..3 {
        let mut forged = f.state.clone();
        let rules = forged.rules.as_mut().unwrap();
        match case {
            0 => rules.timing.as_mut().unwrap().action_spent = false,
            1 => rules.pending.as_mut().unwrap().request.modifier += 10,
            _ => rules.pending.as_mut().unwrap().request.roller = Some(f.actors[1]),
        }
        assert!(validate_tactical_state(&forged).is_err());
    }
}

#[test]
fn medicine_uses_actual_proficiency_exhaustion_inspiration_and_opt_in_natural_extremes() {
    let mut f = fixture();
    f.entity_mut(0)
        .skill_proficiencies
        .insert(Skill::Medicine, Proficiency::Expertise);
    f.entity_mut(0).exhaustion = 1;
    f.entity_mut(0).heroic_inspiration = true;
    f.run(Some(0), aid(&f, MedicinePurpose::Stabilize));
    assert_eq!(f.rules().pending.as_ref().unwrap().request.modifier, 2);
    let original = f.raw(&[1]);
    f.run(
        Some(0),
        TacticalAction::SubmitRollWithInspiration {
            result: original.clone(),
            die_index: 0,
            replacement: DieResult {
                sides: 20,
                value: 8,
            },
        },
    );
    assert_eq!(
        f.rules().rolls.last().unwrap().original_result,
        Some(original)
    );
    assert!(!f.rules().entities[&f.actors[0]].heroic_inspiration);
    assert!(f.rules().entities[&f.actors[1]].death.stable);
    for house in [false, true] {
        let mut f = fixture();
        f.entity_mut(0).ability_scores[Ability::Wisdom.index()] = 30;
        f.state
            .rules
            .as_mut()
            .unwrap()
            .house_rules
            .ability_test_natural_extremes = house;
        f.run(Some(0), aid(&f, MedicinePurpose::Stabilize));
        f.roll(0, &[1]); // Total11 succeeds in SRD; explicit house rule makes natural1 fail.
        assert_eq!(f.rules().entities[&f.actors[1]].death.stable, !house);
        let record = f.rules().rolls.last().unwrap();
        assert_eq!(record.resolved.total, 11);
        assert_eq!(record.result.dice[0].value, 1);
    }
}

#[test]
fn genuine_melee_knockout_ends_only_after_successful_first_aid_without_healing() {
    let mut f = Fixture::new();
    let choice = f.arm("longsword", false, false);
    f.entity_mut(1).hp = 3;
    f.begin();
    f.run(Some(0), TacticalAction::Attack { choice });
    f.roll(0, &[15]);
    f.roll(0, &[8]);
    f.run(
        Some(0),
        TacticalAction::ChooseAttackKnockout {
            choice: KnockoutChoice::KnockOut,
        },
    );
    for face in [9, 10] {
        f.run(Some(0), TacticalAction::EndTurn);
        f.run(Some(1), TacticalAction::EndTurn);
        f.run(Some(0), aid(&f, MedicinePurpose::EndKnockout));
        f.roll(0, &[face]);
        assert_eq!(f.rules().entities[&f.actors[1]].hp, 1);
        assert_eq!(
            active_conditions(f.rules(), f.actors[1]).contains(&Condition::Unconscious),
            face == 9
        );
    }
    assert!(f.flow().resolution.is_none());
    f.rejected(Some(0), aid(&f, MedicinePurpose::EndKnockout));
}
