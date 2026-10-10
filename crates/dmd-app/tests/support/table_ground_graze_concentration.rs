//! Actual source Hold Person, not an injected concentration group.
use super::*;

async fn courtyard(f: &mut Fixture, path: &Path) -> EntityId {
    let cultist = Box::pin(current_source(
        f,
        path,
        "cultist-fanatic",
        CreatureSize::Medium,
    ))
    .await;
    let setup = TableBattlefieldSetup {
        encounter_id: EncounterId::new(),
        scene_id: SceneId::new(),
        location_id: LocationId::new(),
        name: "Source caster in the courtyard".into(),
        area_grid_policy: None,
        battlefield: floor(),
        characters: vec![
            TableCharacterPlacement {
                character_id: f.characters[0],
                position: point(10, 10, 0),
                height: 12,
                allies: vec![f.actors[1]],
                enemies: vec![cultist],
            },
            TableCharacterPlacement {
                character_id: f.characters[1],
                position: point(10, 40, 0),
                height: 12,
                allies: vec![f.actors[0]],
                enemies: vec![cultist],
            },
        ],
        creatures: vec![TableCreaturePlacement {
            actor: cultist,
            public_label: "Robed traveler".into(),
            position: point(20, 10, 0),
            height: 12,
            allies: vec![],
            enemies: f.actors.to_vec(),
        }],
        geometry_ruling: Ruling {
            basis: RulingBasis::GmAdjudication,
            reason:
                "Level illuminated courtyard, adjacent attacker and caster, visible distant ally."
                    .into(),
        },
    };
    Box::pin(battlefield(f, path, setup)).await;
    let combatants = vec![
        TacticalCombatant {
            actor: f.actors[0],
            source: TacticalSource::Character,
            surprised: false,
        },
        TacticalCombatant {
            actor: cultist,
            source: TacticalSource::Creature {
                definition_id: "cultist-fanatic".into(),
            },
            surprised: false,
        },
        TacticalCombatant {
            actor: f.actors[1],
            source: TacticalSource::Character,
            surprised: false,
        },
    ];
    Box::pin(begin(f, path, combatants)).await;
    Box::pin(player_raw(f, path, 18)).await;
    Box::pin(raw(f, path, TableTransportChannel::Host, 10)).await;
    let second = other_player(f);
    Box::pin(raw(f, path, second, 1)).await;
    Box::pin(activate(f, path)).await;
    cultist
}

async fn concentrate(f: &mut Fixture, path: &Path, cultist: EntityId, item: ItemId) -> EffectId {
    Box::pin(equip_prior_miss(f, path, item, cultist)).await;
    Box::pin(player_step(f, path, action(TacticalAction::EndTurn))).await;
    let variant = view(f, &TableTransportChannel::Host)
        .await
        .tactical
        .unwrap()
        .casting_options
        .unwrap()
        .variants
        .into_iter()
        .find(|v| v.choice.spell_id == "hold-person")
        .unwrap();
    let SpellMaterialChoice::Material { item: material } = variant.choice.material else {
        panic!("the actual source's material component must be offered")
    };
    assert_eq!(
        state(f).await.items[&material].custody,
        Custody::Entity(cultist)
    );
    let target = f.actors[1];
    let cast = Box::pin(step(
        f,
        path,
        TableTransportChannel::Host,
        action(TacticalAction::CastSpell {
            choice: variant.choice,
            targets: SpellTargetChoice::Entities(vec![target]),
        }),
    ))
    .await;
    let second = other_player(f);
    Box::pin(raw(f, path, second.clone(), 1)).await;
    let focused = state(f).await;
    let rules = focused.rules.as_ref().unwrap();
    let group = rules
        .tactical_effects
        .as_ref()
        .unwrap()
        .group_for_owner(cultist)
        .unwrap();
    assert_eq!(group.source.command.id, cast.command_id);
    assert_eq!(group.source.actor, cultist);
    let group_id = group.id;
    assert!(dmd_rules::active_conditions(rules, target).contains(&Condition::Paralyzed));
    assert!(rules.timing.as_ref().unwrap().action_spent);
    Box::pin(step(
        f,
        path,
        TableTransportChannel::Host,
        action(TacticalAction::EndTurn),
    ))
    .await;
    Box::pin(step(
        f,
        path,
        second.clone(),
        action(TacticalAction::EndTurn),
    ))
    .await;
    Box::pin(raw(f, path, second, 1)).await;
    let next = state(f).await;
    let rules = next.rules.as_ref().unwrap();
    let timing = rules.timing.as_ref().unwrap();
    assert_eq!(timing.order[timing.index].actor, f.actors[0]);
    assert_eq!(
        rules
            .tactical_effects
            .as_ref()
            .unwrap()
            .group_for_owner(cultist)
            .unwrap()
            .id,
        group_id
    );
    assert!(dmd_rules::active_conditions(rules, target).contains(&Condition::Paralyzed));
    group_id
}

pub(super) async fn run(succeeds: bool, unequip: bool) {
    let (mut f, directory, path, item) = Box::pin(current_fixture("greatsword")).await;
    let cultist = Box::pin(courtyard(&mut f, &path)).await;
    let group = Box::pin(concentrate(&mut f, &path, cultist, item)).await;
    let before = state(&f).await;
    let hp = before.rules.as_ref().unwrap().entities[&cultist].hp;
    let source = before
        .rules
        .as_ref()
        .unwrap()
        .tactical_creatures
        .as_ref()
        .unwrap()
        .profile(cultist)
        .unwrap()
        .source
        .clone();
    let cut = Box::pin(start_graze(&mut f, &path, item, cultist, 1)).await;
    let mastery = Box::pin(choose_graze(&mut f, &path, true)).await;
    let queued = state(&f).await;
    let rules = queued.rules.as_ref().unwrap();
    assert_eq!(rules.entities[&cultist].hp, hp - 3);
    assert_eq!(rules.rolls, cut.paused.rules.as_ref().unwrap().rolls);
    assert_eq!(
        rules
            .tactical_effects
            .as_ref()
            .unwrap()
            .group_for_owner(cultist)
            .unwrap()
            .id,
        group
    );
    assert!(dmd_rules::active_conditions(rules, f.actors[1]).contains(&Condition::Paralyzed));
    let r = resolution(&queued);
    assert!(r.attack.is_none());
    let after = r.attack_after_equipment.as_ref().unwrap();
    assert!(after.selected_by.is_none());
    assert_eq!(after.cause.origin.id, cut.attack.command_id);
    assert_eq!(after.cause.completed_by.id, mastery.command_id);
    assert_eq!(after.cause.completed_work, cut.parent.work);
    let pending = r.pending.as_ref().unwrap();
    assert_eq!(pending.key.role, TacticalRollRole::Concentration);
    assert_eq!(pending.key.subject, cultist);
    assert_eq!(pending.key.origin, cut.attack.command_id);
    assert!(
        matches!(pending.work.kind, TacticalWorkKind::ConcentrationSave { actor, group: actual, .. }
        if actor == cultist && actual == group)
    );
    let trace = r.work_trace.as_ref().unwrap();
    let node = trace
        .nodes
        .iter()
        .find(|node| node.work == pending.work)
        .unwrap();
    assert_eq!(node.parent, Some(cut.parent.work.occurrence));
    assert!(node.work.occurrence > after.work.occurrence);
    assert!(trace.active.is_none());
    assert_eq!(
        rules.pending.as_ref().unwrap().issued_by.id,
        mastery.command_id
    );
    Box::pin(no_after_card(&f)).await;
    for channel in [player(&f), other_player(&f)] {
        let shown = view(&f, &channel).await;
        assert!(shown.roll.is_none());
        assert!(shown.tactical.unwrap().attack_equipment.is_none());
    }
    let premature = request(
        &f,
        player(&f),
        action(TacticalAction::ChooseAttackEquipment {
            work: TacticalWorkKey {
                resolution: cut.attack.command_id,
                occurrence: after.work.occurrence,
            },
            choice: AttackEquipmentChoice::Decline,
        }),
    )
    .await;
    Box::pin(atomic_rejection(&f, premature)).await;
    let shown = view(&f, &TableTransportChannel::Host).await;
    let roll = shown.roll.as_ref().unwrap();
    assert_eq!(roll.roller, Some(cultist));
    assert_eq!(roll.mode, RollMode::Normal);
    assert_eq!(
        roll.dice,
        vec![DieSpec {
            count: 1,
            sides: 20
        }]
    );
    let save_input = physical_input(shown, &[if succeeds { 20 } else { 1 }]);
    let wrong_owner = request(&f, player(&f), save_input.clone()).await;
    Box::pin(atomic_rejection(&f, wrong_owner)).await;
    let save = Box::pin(step(&mut f, &path, TableTransportChannel::Host, save_input)).await;
    let completed = state(&f).await;
    let final_rules = completed.rules.as_ref().unwrap();
    assert_eq!(final_rules.entities[&cultist].hp, hp - 3);
    assert_eq!(
        final_rules.entities[&cultist].concentration.is_some(),
        succeeds
    );
    assert_eq!(
        final_rules
            .tactical_effects
            .as_ref()
            .unwrap()
            .group_for_owner(cultist)
            .is_some(),
        succeeds
    );
    assert_eq!(
        dmd_rules::active_conditions(final_rules, f.actors[1]).contains(&Condition::Paralyzed),
        succeeds
    );
    assert_eq!(
        final_rules
            .tactical_creatures
            .as_ref()
            .unwrap()
            .profile(cultist)
            .unwrap()
            .source,
        source
    );
    assert_eq!(final_rules.rolls.len(), rules.rolls.len() + 1);
    assert_eq!(
        &final_rules.rolls[..rules.rolls.len()],
        rules.rolls.as_slice()
    );
    assert_eq!(
        final_rules.rolls.last().unwrap().accepted_by.id,
        save.command_id
    );
    assert_eq!(completed.items, before.items);
    let selected = resolution(&completed);
    assert!(selected.pending.is_none());
    assert!(
        selected.failed_save.is_none(),
        "the actual Cultist has no Legendary Resistance"
    );
    let final_after = selected.attack_after_equipment.as_ref().unwrap();
    assert_eq!(final_after.cause, after.cause);
    assert_eq!(
        final_after.selected_by.as_ref().unwrap().id,
        save.command_id
    );
    Box::pin(finish_equipment(
        &mut f,
        &path,
        item,
        &cut,
        unequip,
        !succeeds && !unequip,
    ))
    .await;
    Box::pin(close_fixture(f, &directory)).await;
}
