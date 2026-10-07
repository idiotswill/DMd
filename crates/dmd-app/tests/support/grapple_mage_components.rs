//! Current immutable source, real outgoing grip and physical component access.
use super::*;

#[tokio::test]
async fn current_mage_keeps_one_grip_while_casting_with_its_accessible_material() {
    Box::pin(material_case(false)).await;
}

#[tokio::test]
async fn current_mage_two_real_grips_block_material_until_release_and_wand_never_grants_focus() {
    Box::pin(material_case(true)).await;
}

async fn armor_choices(f: &Fixture) -> Vec<SpellCastChoice> {
    f.view(TableTransportChannel::Host)
        .await
        .tactical
        .unwrap()
        .casting_options
        .into_iter()
        .flat_map(|options| options.variants)
        .filter(|variant| variant.choice.spell_id == "mage-armor")
        .map(|variant| variant.choice)
        .collect()
}

async fn release(f: &mut Fixture, label: &str) {
    let request = f.choose(TableTransportChannel::Host, label).await;
    Box::pin(f.cold(request)).await;
}

async fn next_mage_turn(f: &mut Fixture) {
    Box::pin(f.cold_action(TableTransportChannel::Host, TacticalAction::EndTurn)).await;
    if f.opponent.is_some() {
        Box::pin(f.cold_action(TableTransportChannel::Host, TacticalAction::EndTurn)).await;
    }
    let pc = f.pc(0);
    Box::pin(f.cold_action(pc, TacticalAction::EndTurn)).await;
    let state = f.state().await;
    let timing = state.rules.as_ref().unwrap().timing.as_ref().unwrap();
    assert_eq!(timing.order[timing.index].actor, f.goblin);
}

async fn material_case(two_grips: bool) {
    let mut f = Box::pin(Fixture::with_opponent(
        "mage",
        CreatureSize::Medium,
        two_grips,
    ))
    .await;
    Box::pin(f.activate()).await;
    let mage = f.goblin;
    let original = f.state().await;
    let profile = original
        .rules
        .as_ref()
        .unwrap()
        .tactical_creatures
        .as_ref()
        .unwrap()
        .profile(mage)
        .unwrap()
        .clone();
    let pin = dmd_rules::tactical_creatures::creature_source_pin(
        dmd_rules::tactical_definitions::bundled_mage_v2().unwrap(),
    )
    .unwrap();
    assert_eq!(profile.source, pin);
    assert_ne!(pin.definition_fingerprint, "af0f81ba7833b9c4");
    let carried = |definition: &str| {
        original
            .items
            .values()
            .find(|item| item.custody == Custody::Entity(mage) && item.definition_id == definition)
            .unwrap()
            .id
    };
    let wand = carried("wand");
    let material = carried("spell-material:mage-armor");
    let pc = f.pc(0);
    Box::pin(f.cold_action(pc.clone(), TacticalAction::EndTurn)).await;
    let original_choice = armor_choices(&f)
        .await
        .into_iter()
        .find(|choice| choice.material == SpellMaterialChoice::Material { item: material })
        .unwrap();
    assert_eq!(original_choice.actor, mage);
    assert!(
        armor_choices(&f)
            .await
            .iter()
            .all(|choice| !matches!(choice.material, SpellMaterialChoice::Focus { .. }))
    );

    let start = f
        .choose(
            TableTransportChannel::Host,
            "Grapple Character 0 with left hand",
        )
        .await;
    Box::pin(f.cold(start.clone())).await;
    let save = f.choose(pc.clone(), "Resist Grapple with Dexterity").await;
    Box::pin(f.cold(save)).await;
    let rolled = Box::pin(f.cold_roll(pc.clone(), 1)).await;
    let held = f.state().await;
    let grips = &held
        .rules
        .as_ref()
        .unwrap()
        .tactical_grapples
        .as_ref()
        .unwrap()
        .active;
    assert_eq!(grips.len(), 1);
    let grip = grips[0].clone();
    assert_eq!(grip.declaration.origin.id, start.command_id);
    assert_eq!(grip.declaration.grappler, mage);
    assert_eq!(grip.declaration.target, f.actors[0]);
    assert_eq!(grip.declaration.hand, Hand::Left);
    assert_eq!(
        grip.declaration.anatomy,
        GrappleAnatomyProof::Creature {
            source: pin.clone(),
            ordinary_hands: OrdinaryHandAnatomy::TwoHandsV1,
        }
    );
    assert_eq!(grip.established_by.id, rolled.command_id);
    assert_eq!(grip.save.ability, GrappleSaveAbility::Dexterity);
    let choices = f
        .view(TableTransportChannel::Host)
        .await
        .grapple
        .unwrap()
        .choices;
    assert!(choices.iter().all(|choice| {
        !choice
            .label
            .starts_with("After Grapple: equip in left hand")
    }));
    assert!(choices.iter().all(|choice| !choice.label.ends_with("Wand")));
    let finish = f
        .choose(
            TableTransportChannel::Host,
            "Finish without changing equipment",
        )
        .await;
    Box::pin(f.cold(finish)).await;
    Box::pin(next_mage_turn(&mut f)).await;
    let mut expected_grips = vec![grip.clone()];
    if two_grips {
        let start = f
            .choose(
                TableTransportChannel::Host,
                "Grapple Other guard with right hand",
            )
            .await;
        Box::pin(f.cold(start.clone())).await;
        let save = f
            .choose(TableTransportChannel::Host, "Resist Grapple with Strength")
            .await;
        Box::pin(f.cold(save)).await;
        let roll = Box::pin(f.cold_roll(TableTransportChannel::Host, 1)).await;
        let second = f
            .state()
            .await
            .rules
            .unwrap()
            .tactical_grapples
            .unwrap()
            .active
            .into_iter()
            .find(|live| live.declaration.hand == Hand::Right)
            .unwrap();
        assert_eq!(second.declaration.origin.id, start.command_id);
        assert_eq!(second.established_by.id, roll.command_id);
        assert_eq!(second.declaration.grappler, mage);
        assert_eq!(second.declaration.target, f.opponent.unwrap());
        assert_eq!(second.declaration.anatomy, grip.declaration.anatomy);
        expected_grips.push(second);
        expected_grips.sort_by_key(|live| live.declaration.id.0);
        let finish = f
            .choose(
                TableTransportChannel::Host,
                "Finish without changing equipment",
            )
            .await;
        Box::pin(f.cold(finish)).await;
        Box::pin(next_mage_turn(&mut f)).await;
    }

    let before_cast = f.state().await;
    let loadout = before_cast
        .rules
        .as_ref()
        .unwrap()
        .tactical_inventory
        .as_ref()
        .unwrap()
        .loadout(mage)
        .unwrap();
    assert_eq!(loadout.hands.hands, [HandAssignment::Free; 2]);
    assert_eq!(
        before_cast
            .rules
            .as_ref()
            .unwrap()
            .tactical_grapples
            .as_ref()
            .unwrap()
            .active,
        expected_grips
    );
    // Empty physical slots are still reserved by live grips. The Mage's real
    // carried Wand also supplies no class-specific focus permission.
    let mut forbidden_focus = original_choice.clone();
    forbidden_focus.material = SpellMaterialChoice::Focus { item: wand };
    let request = f
        .request(
            TableTransportChannel::Host,
            action(TacticalAction::CastSpell {
                choice: forbidden_focus,
                targets: SpellTargetChoice::Entities(vec![mage]),
            }),
        )
        .await;
    Box::pin(f.reject(request)).await;
    let wrong_owner = f
        .request(
            pc,
            action(TacticalAction::CastSpell {
                choice: original_choice.clone(),
                targets: SpellTargetChoice::Entities(vec![mage]),
            }),
        )
        .await;
    Box::pin(f.reject(wrong_owner)).await;
    if two_grips {
        assert!(armor_choices(&f).await.is_empty());
        let blocked = f
            .request(
                TableTransportChannel::Host,
                action(TacticalAction::CastSpell {
                    choice: original_choice.clone(),
                    targets: SpellTargetChoice::Entities(vec![mage]),
                }),
            )
            .await;
        Box::pin(f.reject(blocked)).await;
        Box::pin(release(&mut f, "Release Character 0 from left hand")).await;
        let remaining = f
            .state()
            .await
            .rules
            .unwrap()
            .tactical_grapples
            .unwrap()
            .active;
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].declaration.hand, Hand::Right);
        assert_eq!(remaining[0].declaration.target, f.opponent.unwrap());
    }
    let choice = armor_choices(&f)
        .await
        .into_iter()
        .find(|choice| choice == &original_choice)
        .unwrap();
    let outsider = f.view(f.pc(1)).await;
    let private = f
        .runtime
        .table_view(f.campaign, TableViewer::Player(f.players[1]))
        .await
        .unwrap();
    let before = f.state().await;
    Box::pin(f.cold_action(
        TableTransportChannel::Host,
        TacticalAction::CastSpell {
            choice,
            targets: SpellTargetChoice::Entities(vec![mage]),
        },
    ))
    .await;
    let after = f.state().await;
    assert_eq!(
        dmd_rules::tactical_defenses::effective_armor_class(&after, mage).unwrap(),
        15
    );
    assert_eq!(after.items, before.items);
    assert_eq!(
        after.rules.as_ref().unwrap().rolls,
        before.rules.as_ref().unwrap().rolls
    );
    assert_eq!(
        after.rules.as_ref().unwrap().tactical_grapples,
        before.rules.as_ref().unwrap().tactical_grapples
    );
    assert_eq!(
        after
            .rules
            .as_ref()
            .unwrap()
            .tactical_creatures
            .as_ref()
            .unwrap()
            .profile(mage),
        Some(&profile)
    );
    Box::pin(assert_private_history(&f, &private, &outsider)).await;
    if !two_grips {
        assert_eq!(
            after
                .rules
                .as_ref()
                .unwrap()
                .tactical_grapples
                .as_ref()
                .unwrap()
                .active,
            vec![grip]
        );
        Box::pin(release(&mut f, "Release Character 0 from left hand")).await;
    } else {
        Box::pin(release(&mut f, "Release Other guard from right hand")).await;
    }
    assert!(f.state().await.rules.unwrap().tactical_grapples.is_none());
    f.close().await;
}
