//! Authored private producer controls. These do not activate public admission,
//! reproduce a journal, or claim application/native acceptance.
use super::*;
mod ground_opportunity_tests;

fn command(state: &CampaignState, actor: Option<EntityId>) -> CommandMeta {
    CommandMeta {
        id: CommandId::new(),
        campaign_id: state.campaign_id(),
        session_id: state.encounter.as_ref().unwrap().origin.session_id,
        issuer: actor
            .and_then(|a| controller(state, a))
            .map_or(CommandIssuer::Admin, CommandIssuer::Player),
        actor: actor.map(AgentRef::Entity),
        expected_event_sequence: state.applied_event_sequence,
    }
}
fn public(
    state: &mut CampaignState,
    pack: &RulesPack,
    actor: Option<EntityId>,
    action: TacticalAction,
) {
    let meta = command(state, actor);
    let mut next = crate::tactical::resolve_tactical(state, &meta, &action, pack)
        .unwrap()
        .next_state;
    next.applied_event_sequence = meta.expected_event_sequence.checked_add(1).unwrap();
    *state = next;
}
fn apply(
    owner: &mut GuardedGrappleExecution<'_>,
    actor: EntityId,
    action: TacticalAction,
) -> CommandMeta {
    let meta = command(owner.state(), Some(actor));
    let event = owner.apply(&meta, &action).unwrap();
    assert_eq!(event.action, action);
    assert_eq!(
        owner.state().applied_event_sequence,
        meta.expected_event_sequence + 1
    );
    meta
}
fn baseline(pack: &RulesPack) -> (CampaignState, EntityId, EntityId) {
    let mut state = crate::tactical_hands::tests::source_state();
    let human = crate::tactical_hands::tests::human(&state);
    let goblin = state
        .rules
        .as_ref()
        .unwrap()
        .tactical_creatures
        .as_ref()
        .unwrap()
        .profiles[0]
        .actor;
    public(
        &mut state,
        pack,
        None,
        TacticalAction::UpgradeExecutionTo {
            execution: TacticalExecutionVersion::EncounterReleaseV1,
        },
    );
    let meta = command(&state, None);
    let source = crate::tactical_creatures::apply_creature_schedule(
        &state,
        state
            .rules
            .as_ref()
            .unwrap()
            .tactical_creatures
            .as_ref()
            .unwrap(),
        &meta,
        &crate::tactical_creatures::CreatureScheduleOperation::SetContext {
            actor: goblin,
            controller: CreatureController::Host,
            in_lair: false,
        },
    )
    .unwrap();
    state.rules.as_mut().unwrap().tactical_creatures = Some(source.next);
    state.applied_event_sequence += 1;
    public(&mut state, pack, Some(human), TacticalAction::EndTurn);
    public(&mut state, pack, Some(goblin), TacticalAction::EndTurn);
    crate::validate_state(&state, pack).unwrap();
    crate::tactical::validate_tactical_state(&state).unwrap();
    (state, human, goblin)
}
fn pending(state: &CampaignState) -> &RollRequest {
    &state
        .rules
        .as_ref()
        .unwrap()
        .pending
        .as_ref()
        .unwrap()
        .request
}
fn faces(request: &RollRequest, value: u16) -> RollResult {
    let mut dice: Vec<_> = request
        .dice
        .iter()
        .flat_map(|spec| {
            (0..spec.count).map(|_| DieResult {
                sides: spec.sides,
                value: value.min(spec.sides),
            })
        })
        .collect();
    if request.mode != RollMode::Normal {
        assert_eq!(dice.len(), 1);
        dice.push(dice[0]);
    }
    RollResult {
        request_id: request.id,
        source: RollSource::Physical,
        dice,
    }
}
fn submit(owner: &mut GuardedGrappleExecution<'_>, value: u16) {
    let request = pending(owner.state());
    let actor = request.roller.unwrap();
    let result = faces(request, value);
    apply(owner, actor, TacticalAction::SubmitRoll { result });
}
fn grip(owner: &mut GuardedGrappleExecution<'_>, actor: EntityId, target: EntityId) -> GrappleId {
    apply(
        owner,
        actor,
        TacticalAction::Grapple {
            target,
            hand: Hand::Left,
            before_change: None,
        },
    );
    let id = attempt(owner.state()).unwrap().declaration.id;
    apply(
        owner,
        target,
        TacticalAction::ChooseGrappleSave {
            grip: id,
            ability: GrappleSaveAbility::Strength,
        },
    );
    if owner.state().rules.as_ref().unwrap().pending.is_some() {
        submit(owner, 1);
    }
    let work = work_key(
        owner.state(),
        attempt(owner.state()).unwrap().selected.as_ref().unwrap(),
    )
    .unwrap();
    apply(
        owner,
        actor,
        TacticalAction::DeclineGrappleAfterEquipment { grip: id, work },
    );
    assert!(flow(owner.state()).unwrap().resolution.is_none());
    id
}
fn item(state: &CampaignState, actor: EntityId, definition: &str) -> ItemId {
    state
        .items
        .values()
        .find(|i| i.custody == Custody::Entity(actor) && i.definition_id == definition)
        .unwrap()
        .id
}
fn dagger(state: &CampaignState, actor: EntityId, target: EntityId) -> WeaponUseChoice {
    WeaponUseChoice {
        after_equipment: None,
        weapon: item(state, actor, "dagger"),
        target,
        delivery: WeaponDelivery::Melee,
        ability: Ability::Strength,
        grip: WeaponGrip::OneHand(Hand::Right),
        purpose: WeaponAttackPurpose::Normal,
        ammunition: None,
        equipment_change: None,
    }
}
fn next_turn(owner: &mut GuardedGrappleExecution<'_>, actor: EntityId) {
    for _ in 0..8 {
        if active(owner.state()).unwrap() == actor {
            return;
        }
        let current = active(owner.state()).unwrap();
        apply(owner, current, TacticalAction::EndTurn);
        settle_boundary(owner);
    }
    panic!("real turn traversal did not reach actor");
}
fn next_round(owner: &mut GuardedGrappleExecution<'_>, actor: EntityId) {
    assert_eq!(active(owner.state()).unwrap(), actor);
    apply(owner, actor, TacticalAction::EndTurn);
    settle_boundary(owner);
    next_turn(owner, actor);
}
fn settle_boundary(owner: &mut GuardedGrappleExecution<'_>) {
    for _ in 0..32 {
        let Some(r) = flow(owner.state()).unwrap().resolution.as_ref() else {
            return;
        };
        assert!(r.attack.is_none() && r.grapple.is_none());
        if owner.state().rules.as_ref().unwrap().pending.is_some() {
            submit(owner, 1);
        } else {
            let actor = r.turn_actor;
            let occurrence = r.frames.last().unwrap()[0].occurrence;
            apply(owner, actor, TacticalAction::ChooseTurnWork { occurrence });
        }
    }
    panic!("boundary work did not settle");
}
fn decline_hit(owner: &mut GuardedGrappleExecution<'_>) {
    let Some(hit) = flow(owner.state())
        .unwrap()
        .resolution
        .as_ref()
        .and_then(|r| r.hit_review.clone())
    else {
        return;
    };
    if hit.stage != TacticalHitReviewStage::Collecting {
        return;
    }
    let window = TacticalWorkKey {
        resolution: resolution(owner.state()).unwrap().origin.id,
        occurrence: hit.work.occurrence,
    };
    if let Some(respondent) = hit.respondent.as_ref().filter(|r| r.intent.is_none()) {
        apply(
            owner,
            respondent.actor,
            TacticalAction::RespondToHit {
                window,
                accept: false,
            },
        );
    }
    if resolution(owner.state())
        .unwrap()
        .hit_review
        .as_ref()
        .unwrap()
        .order
        .is_none()
    {
        let actor = resolution(owner.state()).unwrap().turn_actor;
        apply(
            owner,
            actor,
            TacticalAction::OrderHitResponses {
                window,
                instruction: ordering(),
            },
        );
    }
}
fn ordering() -> TacticalReactionOrdering {
    TacticalReactionOrdering {
        ranked: vec![],
        unlisted: ReactionUnlistedOrder::AfterForward,
    }
}

/// Source-valid private scene setup, not an accepted table history. Canonical
/// builders supply every new mechanical/profile/gear fact. The old encounter is
/// actually concluded/released; replacement/initiative/Prone use real reducers.
fn with_source(
    pack: &RulesPack,
    definition: &str,
) -> (CampaignState, EntityId, EntityId, EntityId) {
    build_source_scene(pack, definition, false)
}
fn build_source_scene(
    pack: &RulesPack,
    definition: &str,
    underwater: bool,
) -> (CampaignState, EntityId, EntityId, EntityId) {
    use crate::tactical_creatures::*;
    let (mut state, human, goblin) = baseline(pack);
    public(
        &mut state,
        pack,
        None,
        TacticalAction::ConcludeHostilities {
            cadence: AftermathCadence::ContinueExistingOrder,
            ruling: "Private setup concludes the original encounter.".into(),
        },
    );
    public(&mut state, pack, None, TacticalAction::FinishEncounter);
    let actor = EntityId::new();
    let mut world = state.entities[&goblin].clone();
    world.id = actor;
    world.display_name = format!("Private canonical {definition}");
    state.entities.insert(actor, world);
    let meta = command(&state, None);
    let source = creature_definition(definition).unwrap();
    let pin = creature_source_pin(source).unwrap();
    let built = build_creature_from_source(
        &state,
        &meta,
        actor,
        &CreatureBuildChoice {
            definition_id: definition.into(),
            size: CreatureSize::Medium,
            additional_languages: if definition == "mage" {
                vec!["dwarvish".into(), "elvish".into(), "draconic".into()]
            } else {
                vec![]
            },
            hit_points: CreatureHitPointChoice::Average,
            controller: CreatureController::Host,
            in_lair: false,
        },
        Some(&pin),
    )
    .unwrap();
    let rules = state.rules.as_mut().unwrap();
    rules.entities.insert(actor, built.mechanics);
    let creatures = rules.tactical_creatures.as_mut().unwrap();
    creatures.profiles.push(built.profile);
    creatures.profiles.sort_by_key(|p| p.actor.0);
    creatures.runtime.push(built.runtime);
    creatures.runtime.sort_by_key(|r| r.actor.0);
    let allocations =
        crate::tactical_creature_equipment::creature_equipment_plan_from_source(&pin, 0).unwrap();
    state = crate::tactical_creature_equipment::materialize_creature_equipment(
        &state,
        &meta,
        actor,
        0,
        &allocations
            .iter()
            .map(|_| ItemId::new())
            .collect::<Vec<_>>(),
        pack,
    )
    .unwrap();
    state.applied_event_sequence += 1;
    let participant = TacticalParticipant {
        entity_id: actor,
        position: SpatialPoint {
            x: if underwater { 80 } else { 20 },
            y: 20,
            z: 0,
        },
        size: CreatureSize::Medium,
        public_label: "Third source actor".into(),
        height: 10,
        reach: 10,
        movement: built.movement,
        senses: built.senses,
        allies: vec![],
        enemies: vec![human, goblin],
    };
    replacement(
        state,
        pack,
        human,
        goblin,
        participant,
        TacticalSource::Creature {
            definition_id: definition.into(),
        },
        underwater,
    )
}
fn replacement(
    mut state: CampaignState,
    pack: &RulesPack,
    human: EntityId,
    goblin: EntityId,
    participant: TacticalParticipant,
    source: TacticalSource,
    underwater: bool,
) -> (CampaignState, EntityId, EntityId, EntityId) {
    let actor = participant.entity_id;
    let mut encounter = state.encounter.as_ref().unwrap().clone();
    let mut scene = state.scenes[&encounter.scene_id].clone();
    scene.id = SceneId::new();
    scene.status = SceneStatus::Closed;
    scene.presences.push(ScenePresence {
        entity_id: actor,
        role: PresenceRole::Participant,
    });
    encounter.id = EncounterId::new();
    encounter.scene_id = scene.id;
    encounter.flow = None;
    if underwater {
        encounter.battlefield.terrain.push(TerrainVolume {
            id: "private-full-water".into(),
            volume: encounter.battlefield.bounds,
            difficult: false,
            water: true,
            climbable: false,
            burrowable: false,
            supports_top: false,
            surface: None,
            obscuration: Obscuration::None,
            magical_darkness: false,
            observable: true,
        });
    }
    encounter.participants.push(participant);
    for participant in &mut encounter.participants {
        participant.enemies = [human, goblin, actor]
            .into_iter()
            .filter(|id| *id != participant.entity_id)
            .collect();
    }
    state.scenes.insert(scene.id, scene);
    public(
        &mut state,
        pack,
        None,
        TacticalAction::Establish {
            encounter: Box::new(encounter),
        },
    );
    public(
        &mut state,
        pack,
        None,
        TacticalAction::Begin {
            combatants: vec![
                TacticalCombatant {
                    actor: human,
                    source: TacticalSource::Character,
                    surprised: false,
                },
                TacticalCombatant {
                    actor: goblin,
                    source: TacticalSource::Creature {
                        definition_id: "goblin-warrior".into(),
                    },
                    surprised: false,
                },
                TacticalCombatant {
                    actor,
                    source,
                    surprised: false,
                },
            ],
            groups: [human, goblin, actor]
                .into_iter()
                .map(|id| InitiativeGroup {
                    actors: vec![id],
                    request_id: RollRequestId::new(),
                })
                .collect(),
            execution: TacticalExecutionVersion::EncounterReleaseV1,
        },
    );
    for face in [20, 10, 1] {
        let request = pending(&state);
        let roller = request.roller.unwrap();
        let result = faces(request, face);
        public(
            &mut state,
            pack,
            Some(roller),
            TacticalAction::SubmitRoll { result },
        );
    }
    assert_eq!(active(&state).unwrap(), human);
    crate::validate_state(&state, pack).unwrap();
    crate::tactical::validate_tactical_state(&state).unwrap();
    (state, human, goblin, actor)
}

fn with_second_human(pack: &RulesPack) -> (CampaignState, EntityId, EntityId, EntityId) {
    let (mut state, human, goblin) = baseline(pack);
    public(
        &mut state,
        pack,
        None,
        TacticalAction::ConcludeHostilities {
            cadence: AftermathCadence::ContinueExistingOrder,
            ruling: "Private setup concludes the original encounter.".into(),
        },
    );
    public(&mut state, pack, None, TacticalAction::FinishEncounter);
    let character = state
        .characters
        .values()
        .find(|c| c.entity_id != human)
        .unwrap();
    let character_id = character.id;
    let actor = character.entity_id;
    let plan =
        crate::tactical_inventory::starting_equipment_plan(&state, character_id, pack).unwrap();
    let meta = command(&state, None);
    let grant = crate::tactical_inventory::materialize_starting_equipment(
        &state,
        state
            .rules
            .as_ref()
            .unwrap()
            .tactical_inventory
            .as_ref()
            .unwrap(),
        &meta,
        character_id,
        &(0..plan.identity_count)
            .map(|_| ItemId::new())
            .collect::<Vec<_>>(),
        pack,
    )
    .unwrap();
    state = grant.next_state;
    state.rules.as_mut().unwrap().tactical_inventory = Some(grant.next_inventory);
    state.applied_event_sequence += 1;
    let mut participant = state
        .encounter
        .as_ref()
        .unwrap()
        .participant(human)
        .unwrap()
        .clone();
    participant.entity_id = actor;
    participant.position = SpatialPoint { x: 20, y: 20, z: 0 };
    participant.public_label = "Second canonical Human".into();
    replacement(
        state,
        pack,
        human,
        goblin,
        participant,
        TacticalSource::Character,
        false,
    )
}

#[test]
fn two_real_holders_are_both_incoming_and_attacking_one_still_has_disadvantage() {
    let pack = crate::tactical_hands::tests::pack();
    let (state, human, goblin, second) = with_second_human(&pack);
    let mut owner = GuardedGrappleExecution::new(state, &pack).unwrap();
    let first = grip(&mut owner, human, goblin);
    next_turn(&mut owner, second);
    let other = grip(&mut owner, second, goblin);
    next_turn(&mut owner, goblin);
    let action = scimitar(owner.state(), goblin, human);
    apply(&mut owner, goblin, action);
    let mut ids = vec![first, other];
    ids.sort_by_key(|id| id.0);
    assert_admission(&owner, &ids);
    assert_eq!(pending(owner.state()).mode, RollMode::Disadvantage);
    let request = pending(owner.state()).clone();
    apply(
        &mut owner,
        second,
        TacticalAction::ReleaseGrapple { grip: other },
    );
    assert_eq!(pending(owner.state()), &request);
    submit(&mut owner, 1);
    next_round(&mut owner, goblin);
    let action = scimitar(owner.state(), goblin, human);
    apply(&mut owner, goblin, action);
    assert_admission(&owner, &[first]);
    assert_eq!(pending(owner.state()).mode, RollMode::Normal);
}

fn prone_third(pack: &RulesPack) -> (CampaignState, EntityId, EntityId, EntityId) {
    let (mut state, human, goblin, mage) = with_source(pack, "mage");
    public(
        &mut state,
        pack,
        Some(human),
        TacticalAction::Shove { target: mage },
    );
    public(
        &mut state,
        pack,
        Some(mage),
        TacticalAction::ChooseShoveSave {
            ability: ShoveSaveAbility::Strength,
        },
    );
    let result = faces(pending(&state), 1);
    public(
        &mut state,
        pack,
        Some(mage),
        TacticalAction::SubmitRoll { result },
    );
    public(
        &mut state,
        pack,
        Some(human),
        TacticalAction::ChooseShoveOutcome {
            choice: ShoveChoice::Prone,
        },
    );
    assert!(state.rules.as_ref().unwrap().entities[&mage].prone);
    for actor in [human, goblin, mage] {
        public(&mut state, pack, Some(actor), TacticalAction::EndTurn);
    }
    (state, human, goblin, mage)
}
fn scimitar(state: &CampaignState, goblin: EntityId, target: EntityId) -> TacticalAction {
    let weapon = item(state, goblin, "scimitar");
    let equipped = admission::loadout(state, goblin)
        .unwrap()
        .hands
        .hands
        .contains(&HandAssignment::Item(weapon));
    TacticalAction::CreatureWeaponAttack {
        feature_id: "scimitar".into(),
        choice: CreatureWeaponUseChoice {
            after_equipment: None,
            weapon,
            target,
            grip: WeaponGrip::OneHand(Hand::Right),
            ammunition: None,
            equipment_change: (!equipped).then_some(AttackEquipmentChange {
                timing: EquipmentChangeTiming::BeforeAttack,
                operation: AttackEquipmentOperation::Equip {
                    item: weapon,
                    hand: Hand::Right,
                },
            }),
        },
    }
}

#[test]
fn printed_goblin_cancellation_keeps_base_damage_across_each_real_release_wait() {
    let pack = crate::tactical_hands::tests::pack();
    for release_at in 0..3 {
        let (state, human, goblin, mage) = prone_third(&pack);
        let mut owner = GuardedGrappleExecution::new(state, &pack).unwrap();
        let id = grip(&mut owner, human, goblin);
        apply(&mut owner, human, TacticalAction::EndTurn);
        let action = scimitar(owner.state(), goblin, mage);
        apply(&mut owner, goblin, action);
        assert_admission(&owner, &[id]);
        let admitted = resolution(owner.state())
            .unwrap()
            .attack
            .as_ref()
            .unwrap()
            .clone();
        assert_eq!(admitted.mode, RollMode::Normal);
        assert_eq!(admitted.damage.len(), 1);
        if release_at == 0 {
            apply(
                &mut owner,
                human,
                TacticalAction::ReleaseGrapple { grip: id },
            );
        }
        submit(&mut owner, 19);
        if release_at == 1 {
            apply(
                &mut owner,
                human,
                TacticalAction::ReleaseGrapple { grip: id },
            );
        }
        decline_hit(&mut owner);
        let request = pending(owner.state()).clone();
        if release_at == 2 {
            apply(
                &mut owner,
                human,
                TacticalAction::ReleaseGrapple { grip: id },
            );
        }
        assert_eq!(pending(owner.state()), &request);
        let attack = resolution(owner.state()).unwrap().attack.as_ref().unwrap();
        assert_eq!(attack.mode, admitted.mode);
        assert_eq!(attack.damage, admitted.damage);
        submit(&mut owner, 1);
        assert!(flow(owner.state()).unwrap().resolution.is_none());
        next_round(&mut owner, goblin);
        let action = scimitar(owner.state(), goblin, mage);
        apply(&mut owner, goblin, action);
        let attack = resolution(owner.state()).unwrap().attack.as_ref().unwrap();
        assert_eq!(attack.mode, RollMode::Advantage);
        assert_eq!(attack.damage.len(), 2);
        assert!(resolution(owner.state()).unwrap().grapple.is_none());
    }
}

#[test]
fn intrinsic_wolf_uses_same_admission_and_release_keeps_actual_request() {
    let pack = crate::tactical_hands::tests::pack();
    let (state, human, goblin, wolf) = with_source(&pack, "wolf");
    let mut owner = GuardedGrappleExecution::new(state, &pack).unwrap();
    let id = grip(&mut owner, human, wolf);
    next_turn(&mut owner, wolf);
    apply(
        &mut owner,
        wolf,
        TacticalAction::CreatureAttack {
            target: goblin,
            feature_id: "bite".into(),
            weapon: None,
        },
    );
    assert_admission(&owner, &[id]);
    assert_eq!(pending(owner.state()).mode, RollMode::Disadvantage);
    let request = pending(owner.state()).clone();
    apply(
        &mut owner,
        human,
        TacticalAction::ReleaseGrapple { grip: id },
    );
    assert_eq!(pending(owner.state()), &request);
    submit(&mut owner, 1);
    assert!(flow(owner.state()).unwrap().resolution.is_none());
    next_round(&mut owner, wolf);
    apply(
        &mut owner,
        wolf,
        TacticalAction::CreatureAttack {
            target: goblin,
            feature_id: "bite".into(),
            weapon: None,
        },
    );
    assert_eq!(pending(owner.state()).mode, RollMode::Normal);
}

#[test]
fn actual_selected_shield_keeps_attack_cut_and_distinguishes_issue_time_from_damage_cause() {
    let pack = crate::tactical_hands::tests::pack();
    let (state, human, goblin, mage) = prone_third(&pack);
    let mut owner = GuardedGrappleExecution::new(state, &pack).unwrap();
    let id = grip(&mut owner, human, goblin);
    apply(&mut owner, human, TacticalAction::EndTurn);
    let action = scimitar(owner.state(), goblin, mage);
    apply(&mut owner, goblin, action);
    submit(&mut owner, 20);
    let r = resolution(owner.state()).unwrap();
    let hit = r.hit_review.as_ref().unwrap();
    let cause = hit.cause.clone();
    let window = TacticalWorkKey {
        resolution: r.origin.id,
        occurrence: hit.work.occurrence,
    };
    apply(
        &mut owner,
        mage,
        TacticalAction::RespondToHit {
            window,
            accept: true,
        },
    );
    apply(
        &mut owner,
        goblin,
        TacticalAction::OrderHitResponses {
            window,
            instruction: ordering(),
        },
    );
    apply(
        &mut owner,
        human,
        TacticalAction::ReleaseGrapple { grip: id },
    );
    let choice = SpellCastChoice {
        actor: mage,
        spell_id: "shield".into(),
        grant: SpellGrantChoice::CreatureFeature {
            feature_id: "protective-magic".into(),
        },
        resource: SpellResourceChoice::SourceFeature,
        material: SpellMaterialChoice::None,
        mode: SpellCastMode::Immediate,
    };
    let cast = apply(
        &mut owner,
        mage,
        TacticalAction::CastHitShield { window, choice },
    );
    let r = resolution(owner.state()).unwrap();
    assert!(r.hit_review.as_ref().unwrap().completed_shield.is_some());
    assert_eq!(r.attack.as_ref().unwrap().mode, RollMode::Normal);
    assert_eq!(r.attack.as_ref().unwrap().damage.len(), 1);
    let raw = owner
        .state()
        .rules
        .as_ref()
        .unwrap()
        .pending
        .as_ref()
        .unwrap();
    assert_eq!(raw.issued_by, cause);
    let c = r.grapple.as_ref().unwrap();
    assert!(c.cuts.iter().any(|cut| matches!(cut.key.reader, GrappleReader::RequestIssue { roll } if roll.role == TacticalRollRole::AttackDamage) && cut.issued_by == cast));
    assert_admission(&owner, &[id]);
    submit(&mut owner, 1);
}

#[test]
fn actual_damage_that_breaks_holder_rolls_back_the_entire_command() {
    let pack = crate::tactical_hands::tests::pack();
    let (state, human, goblin) = baseline(&pack);
    let mut owner = GuardedGrappleExecution::new(state, &pack).unwrap();
    grip(&mut owner, human, goblin);
    apply(&mut owner, human, TacticalAction::EndTurn);
    let action = scimitar(owner.state(), goblin, human);
    apply(&mut owner, goblin, action);
    submit(&mut owner, 20);
    decline_hit(&mut owner);
    let result = faces(pending(owner.state()), 6);
    apply(&mut owner, goblin, TacticalAction::SubmitRoll { result });
    assert_eq!(
        resolution(owner.state())
            .unwrap()
            .attack
            .as_ref()
            .unwrap()
            .stage,
        TacticalAttackStage::KnockoutChoice
    );
    let before = owner.state().clone();
    let meta = command(&before, Some(goblin));
    assert!(
        owner
            .apply(
                &meta,
                &TacticalAction::ChooseAttackKnockout {
                    choice: KnockoutChoice::KnockOut
                }
            )
            .is_err()
    );
    assert_eq!(owner.state(), &before);
}

// Hold Person needs a distinct source Humanoid: the captured Goblin is Fey.
// The held actor is a second canonical Host Cultist; two Player-owned Humans
// would require the still-closed consent path. Build every source/gear fact.
fn with_held_cultist_and_caster(
    pack: &RulesPack,
) -> (CampaignState, EntityId, EntityId, EntityId, EntityId) {
    use crate::tactical_creatures::*;
    let (mut state, human, goblin, cultist) = with_source(pack, "cultist-fanatic");
    public(
        &mut state,
        pack,
        None,
        TacticalAction::ConcludeHostilities {
            cadence: AftermathCadence::ContinueExistingOrder,
            ruling: "Private setup adds a second canonical Cultist to a fresh encounter.".into(),
        },
    );
    public(&mut state, pack, None, TacticalAction::FinishEncounter);
    let held = EntityId::new();
    let mut world = state.entities[&goblin].clone();
    world.id = held;
    world.display_name = "Private canonical held Cultist".into();
    state.entities.insert(held, world);
    let meta = command(&state, None);
    let source = creature_definition("cultist-fanatic").unwrap();
    assert_eq!(
        source.statistics.creature_type,
        crate::tactical_definitions::CreatureType::Humanoid
    );
    let pin = creature_source_pin(source).unwrap();
    let built = build_creature_from_source(
        &state,
        &meta,
        held,
        &CreatureBuildChoice {
            definition_id: "cultist-fanatic".into(),
            size: CreatureSize::Medium,
            additional_languages: vec![],
            hit_points: CreatureHitPointChoice::Average,
            controller: CreatureController::Host,
            in_lair: false,
        },
        Some(&pin),
    )
    .unwrap();
    let rules = state.rules.as_mut().unwrap();
    rules.entities.insert(held, built.mechanics);
    let creatures = rules.tactical_creatures.as_mut().unwrap();
    creatures.profiles.push(built.profile);
    creatures.profiles.sort_by_key(|p| p.actor.0);
    creatures.runtime.push(built.runtime);
    creatures.runtime.sort_by_key(|r| r.actor.0);
    let allocations =
        crate::tactical_creature_equipment::creature_equipment_plan_from_source(&pin, 0).unwrap();
    state = crate::tactical_creature_equipment::materialize_creature_equipment(
        &state,
        &meta,
        held,
        0,
        &allocations
            .iter()
            .map(|_| ItemId::new())
            .collect::<Vec<_>>(),
        pack,
    )
    .unwrap();
    state.applied_event_sequence += 1;
    let mut encounter = state.encounter.as_ref().unwrap().clone();
    let mut scene = state.scenes[&encounter.scene_id].clone();
    scene.id = SceneId::new();
    scene.status = SceneStatus::Closed;
    scene.presences.push(ScenePresence {
        entity_id: held,
        role: PresenceRole::Participant,
    });
    encounter.id = EncounterId::new();
    encounter.scene_id = scene.id;
    encounter.flow = None;
    encounter.participants.push(TacticalParticipant {
        entity_id: held,
        position: SpatialPoint { x: 20, y: 0, z: 0 },
        size: CreatureSize::Medium,
        public_label: "Second source Cultist".into(),
        height: 10,
        reach: 10,
        movement: built.movement,
        senses: built.senses,
        allies: vec![],
        enemies: vec![],
    });
    let actors = [human, goblin, held, cultist];
    for participant in &mut encounter.participants {
        participant.enemies = actors
            .into_iter()
            .filter(|id| *id != participant.entity_id)
            .collect();
    }
    state.scenes.insert(scene.id, scene);
    public(
        &mut state,
        pack,
        None,
        TacticalAction::Establish {
            encounter: Box::new(encounter),
        },
    );
    public(
        &mut state,
        pack,
        None,
        TacticalAction::Begin {
            combatants: actors
                .into_iter()
                .map(|actor| TacticalCombatant {
                    actor,
                    source: if actor == human {
                        TacticalSource::Character
                    } else if actor == goblin {
                        TacticalSource::Creature {
                            definition_id: "goblin-warrior".into(),
                        }
                    } else {
                        TacticalSource::Creature {
                            definition_id: "cultist-fanatic".into(),
                        }
                    },
                    surprised: false,
                })
                .collect(),
            groups: [vec![human], vec![goblin], vec![held, cultist]]
                .into_iter()
                .map(|actors| InitiativeGroup {
                    actors,
                    request_id: RollRequestId::new(),
                })
                .collect(),
            execution: TacticalExecutionVersion::EncounterReleaseV1,
        },
    );
    for (actor, face) in [human, goblin, held].into_iter().zip([20, 10, 5]) {
        let request = pending(&state);
        assert_eq!(request.roller, Some(actor));
        let result = faces(request, face);
        public(
            &mut state,
            pack,
            Some(actor),
            TacticalAction::SubmitRoll { result },
        );
    }
    public(
        &mut state,
        pack,
        None,
        TacticalAction::ProposeInitiativeTie {
            order: vec![held, cultist],
        },
    );
    assert_eq!(
        state.rules.as_ref().unwrap().timing.as_ref().unwrap().order,
        vec![
            InitiativeEntry {
                actor: human,
                total: 22,
                tie_break: 0
            },
            InitiativeEntry {
                actor: goblin,
                total: 12,
                tie_break: 0
            },
            InitiativeEntry {
                actor: held,
                total: 7,
                tie_break: 0
            },
            InitiativeEntry {
                actor: cultist,
                total: 7,
                tie_break: 1
            },
        ]
    );
    assert_eq!(active(&state).unwrap(), human);
    crate::validate_state(&state, pack).unwrap();
    crate::tactical::validate_tactical_state(&state).unwrap();
    (state, human, goblin, held, cultist)
}

#[test]
fn actual_concentration_child_retains_completed_attack_evidence_through_release() {
    let pack = crate::tactical_hands::tests::pack();
    let (mut state, human, goblin, held, cultist) = with_held_cultist_and_caster(&pack);
    for actor in [human, goblin, held] {
        assert_eq!(active(&state).unwrap(), actor);
        public(&mut state, &pack, Some(actor), TacticalAction::EndTurn);
    }
    assert_eq!(active(&state).unwrap(), cultist);
    let material = item(&state, cultist, "spell-material:hold-person");
    public(
        &mut state,
        &pack,
        Some(cultist),
        TacticalAction::CastSpell {
            choice: SpellCastChoice {
                actor: cultist,
                spell_id: "hold-person".into(),
                grant: SpellGrantChoice::CreatureFeature {
                    feature_id: "spellcasting".into(),
                },
                resource: SpellResourceChoice::SourceFeature,
                material: SpellMaterialChoice::Material { item: material },
                mode: SpellCastMode::Immediate,
            },
            targets: SpellTargetChoice::Entities(vec![held]),
        },
    );
    assert_eq!(pending(&state).roller, Some(held));
    assert_eq!(
        resolution(&state)
            .unwrap()
            .pending
            .as_ref()
            .unwrap()
            .key
            .role,
        TacticalRollRole::SpellSave
    );
    let result = faces(pending(&state), 1);
    public(
        &mut state,
        &pack,
        Some(held),
        TacticalAction::SubmitRoll { result },
    );
    assert!(
        state.rules.as_ref().unwrap().entities[&cultist]
            .concentration
            .is_some()
    );
    let group = state.rules.as_ref().unwrap().entities[&cultist]
        .concentration
        .unwrap();
    assert!(
        crate::active_conditions(state.rules.as_ref().unwrap(), held)
            .contains(&Condition::Paralyzed)
    );
    public(&mut state, &pack, Some(cultist), TacticalAction::EndTurn);
    let mut owner = GuardedGrappleExecution::new(state, &pack).unwrap();
    let inherited = owner.state().rules.as_ref().unwrap().rolls.clone();
    let id = grip(&mut owner, human, held);
    assert_eq!(
        flow(owner.state())
            .unwrap()
            .save_decisions
            .last()
            .unwrap()
            .failure,
        TacticalSaveFailure::Automatic
    );
    let decision = flow(owner.state()).unwrap().save_decisions.last().unwrap();
    assert_eq!(decision.key.subject, held);
    assert_eq!(decision.key.role, TacticalRollRole::GrappleSave);
    assert_eq!(owner.state().rules.as_ref().unwrap().rolls, inherited);
    next_round(&mut owner, human);
    assert!(
        crate::active_conditions(owner.state().rules.as_ref().unwrap(), held)
            .contains(&Condition::Paralyzed)
    );
    assert_eq!(
        owner.state().rules.as_ref().unwrap().entities[&cultist].concentration,
        Some(group)
    );
    let choice = dagger(owner.state(), human, cultist);
    apply(&mut owner, human, TacticalAction::Attack { choice });
    submit(&mut owner, 19);
    decline_hit(&mut owner);
    submit(&mut owner, 1);
    let r = resolution(owner.state()).unwrap();
    assert!(r.attack.is_none() && r.hit_review.is_none());
    let child = r.pending.as_ref().unwrap();
    assert_eq!(child.key.role, TacticalRollRole::Concentration);
    let root = reads::attack_root(owner.state()).unwrap();
    assert!(reads::in_lineage(owner.state(), child.work.occurrence, root.work.occurrence).unwrap());
    let node = r
        .work_trace
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .find(|n| n.work == child.work)
        .unwrap();
    assert!(
        r.work_trace
            .as_ref()
            .unwrap()
            .nodes
            .iter()
            .any(|parent| Some(parent.work.occurrence) == node.parent
                && parent.work.kind == TacticalWorkKind::AttackDamage)
    );
    let tuple = (
        owner.state().rules.as_ref().unwrap().pending.clone(),
        r.pending.clone(),
        r.frames.clone(),
    );
    assert_admission(&owner, &[id]);
    apply(
        &mut owner,
        human,
        TacticalAction::ReleaseGrapple { grip: id },
    );
    let r = resolution(owner.state()).unwrap();
    assert_eq!(
        (
            owner.state().rules.as_ref().unwrap().pending.clone(),
            r.pending.clone(),
            r.frames.clone()
        ),
        tuple
    );
    assert_admission(&owner, &[id]);
    submit(&mut owner, 20);
    assert!(flow(owner.state()).unwrap().resolution.is_none());
    assert!(GuardedGrappleExecution::new(owner.state().clone(), &pack).is_err());
}

#[test]
fn negative_only_changed_cuts_source_and_ancestry_cannot_replace_owned_attack_evidence() {
    let pack = crate::tactical_hands::tests::pack();
    let (state, human, goblin, mage) = prone_third(&pack);
    let mut owner = GuardedGrappleExecution::new(state, &pack).unwrap();
    let id = grip(&mut owner, human, goblin);
    apply(&mut owner, human, TacticalAction::EndTurn);
    let action = scimitar(owner.state(), goblin, mage);
    apply(&mut owner, goblin, action);
    let before = owner.state().clone();
    for case in 0..9 {
        // Deliberately inaccessible mechanism negatives. No fabricated image is
        // accepted by the owner or used as a positive producer/setup state.
        let mut candidate = Box::new(before.clone());
        let meta = command(&before, Some(goblin));
        let execution = ExecutionContext {
            mass: None,
            guarded: Some(GuardedCommand {
                predecessor: &before,
                candidate: std::ptr::from_ref(candidate.as_ref()),
                command: &meta,
                produced: ProducedEvidence::default(),
            }),
        };
        let r = resolution_mut(&mut candidate).unwrap();
        let c = r.grapple.as_mut().unwrap();
        match case {
            0 => {
                c.cuts.remove(0);
            }
            1 => {
                c.cuts.push(c.cuts[1].clone());
            }
            2 => {
                c.cuts[0].grips.clear();
            }
            3 => {
                c.proofs[0].declaration.grappler = mage;
            }
            4 => {
                c.cuts[1].source_attack = Some(c.cuts[1].key);
            }
            5 => {
                c.cuts[1].issued_by.expected_event_sequence -= 1;
            }
            6 => {
                r.attack.as_mut().unwrap().damage[0].modifier += 1;
            }
            7 => {
                r.work_trace.as_mut().unwrap().nodes[0].parent = Some(0);
            }
            _ => {
                c.proofs[0].declaration.id =
                    GrappleId::from_declaration(CommandId::new(), human, goblin, Hand::Left);
            }
        }
        let result = execution
            .validate_delta(&candidate)
            .and_then(|()| {
                crate::kernel::validate_state_with_read(&execution.read(&candidate)?, &pack)
            })
            .and_then(|()| {
                crate::tactical::validation::validate_tactical_state_with_read(
                    &execution.read(&candidate)?,
                )
            });
        assert!(result.is_err(), "negative case {case} must reject");
        assert_eq!(owner.state(), &before);
    }
    assert_eq!(context(owner.state()).unwrap().proofs[0].declaration.id, id);
}
fn assert_admission(owner: &GuardedGrappleExecution<'_>, ids: &[GrappleId]) {
    let c = context(owner.state()).unwrap();
    assert!(c.activity.is_none());
    assert_eq!(
        c.proofs
            .iter()
            .map(|g| g.declaration.id)
            .collect::<Vec<_>>(),
        ids
    );
    let r = resolution(owner.state()).unwrap();
    let root = reads::attack_root(owner.state()).unwrap();
    assert_eq!(
        c.cuts[0].key.work,
        work_key(owner.state(), &root.work).unwrap()
    );
    assert_eq!(
        c.cuts[0].key.reader,
        GrappleReader::AttackAdmission {
            attack: r.origin.id
        }
    );
    for cut in &c.cuts[1..] {
        assert_eq!(cut.grips, ids);
        assert_eq!(cut.source_attack, Some(c.cuts[0].key));
    }
    assert_eq!(r.work_trace.as_ref().unwrap().active, None);
}

#[test]
fn ordinary_holder_attack_release_keeps_request_then_later_attack_has_current_free_hands() {
    let pack = crate::tactical_hands::tests::pack();
    let (state, human, goblin) = baseline(&pack);
    let mut owner = GuardedGrappleExecution::new(state, &pack).unwrap();
    let id = grip(&mut owner, human, goblin);
    next_round(&mut owner, human);
    let choice = dagger(owner.state(), human, goblin);
    apply(&mut owner, human, TacticalAction::Attack { choice });
    assert_admission(&owner, &[id]);
    let request = pending(owner.state()).clone();
    let attack = resolution(owner.state()).unwrap().attack.clone();
    apply(
        &mut owner,
        human,
        TacticalAction::ReleaseGrapple { grip: id },
    );
    assert_eq!(pending(owner.state()), &request);
    assert_eq!(resolution(owner.state()).unwrap().attack, attack);
    assert_admission(&owner, &[id]);
    assert_eq!(context(owner.state()).unwrap().ends.len(), 1);
    submit(&mut owner, 1);
    assert!(flow(owner.state()).unwrap().resolution.is_none());
    let inherited = owner.state().rules.as_ref().unwrap().rolls.clone();
    assert!(GuardedGrappleExecution::new(owner.state().clone(), &pack).is_err());
    next_round(&mut owner, human);
    let mut choice = dagger(owner.state(), human, goblin);
    choice.equipment_change = Some(AttackEquipmentChange {
        timing: EquipmentChangeTiming::AfterAttack,
        operation: AttackEquipmentOperation::Unequip {
            item: choice.weapon,
        },
    });
    apply(&mut owner, human, TacticalAction::Attack { choice });
    assert!(resolution(owner.state()).unwrap().grapple.is_none());
    submit(&mut owner, 1);
    assert!(
        owner
            .state()
            .rules
            .as_ref()
            .unwrap()
            .rolls
            .starts_with(&inherited)
    );
    assert_eq!(
        admission::loadout(owner.state(), human).unwrap().hands,
        WeaponLoadout::default()
    );
}

#[test]
fn unarmed_fixed_damage_uses_admission_and_creates_no_damage_raw_or_issue_cut() {
    let pack = crate::tactical_hands::tests::pack();
    let (state, human, goblin) = baseline(&pack);
    let mut owner = GuardedGrappleExecution::new(state, &pack).unwrap();
    let id = grip(&mut owner, human, goblin);
    apply(&mut owner, human, TacticalAction::EndTurn);
    apply(
        &mut owner,
        goblin,
        TacticalAction::UnarmedStrike { target: human },
    );
    assert_admission(&owner, &[id]);
    assert_eq!(
        pending(owner.state()).mode,
        RollMode::Normal,
        "attacking sole holder is exempt"
    );
    // The genuine captured Goblin has Strength 8: 1 + (-1) is zero damage.
    assert_eq!(
        owner.state().rules.as_ref().unwrap().entities[&goblin].ability_scores
            [Ability::Strength.index()],
        8
    );
    assert_eq!(
        resolution(owner.state())
            .unwrap()
            .attack
            .as_ref()
            .unwrap()
            .damage,
        vec![AttackDamageComponent {
            damage_type: DamageType::Bludgeoning,
            dice: vec![],
            modifier: 0,
        }]
    );
    let origin = resolution(owner.state()).unwrap().origin.id;
    let before = owner.state().rules.as_ref().unwrap().entities[&human].hp;
    apply(
        &mut owner,
        human,
        TacticalAction::ReleaseGrapple { grip: id },
    );
    submit(&mut owner, 20);
    decline_hit(&mut owner);
    assert!(flow(owner.state()).unwrap().resolution.is_none());
    assert_eq!(
        owner.state().rules.as_ref().unwrap().entities[&human].hp,
        before
    );
    assert_eq!(owner.state().rules.as_ref().unwrap().rolls.iter().filter(|raw| matches!(raw.purpose, PendingPurpose::TacticalResolution { key, .. } if key.origin == origin)).count(), 1);
}

#[test]
fn damage_release_and_real_knockout_complete_current_equipment_once() {
    let pack = crate::tactical_hands::tests::pack();
    let (state, human, goblin) = baseline(&pack);
    let mut owner = GuardedGrappleExecution::new(state, &pack).unwrap();
    let id = grip(&mut owner, human, goblin);
    next_round(&mut owner, human);
    let mut choice = dagger(owner.state(), human, goblin);
    let weapon = choice.weapon;
    choice.equipment_change = Some(AttackEquipmentChange {
        timing: EquipmentChangeTiming::AfterAttack,
        operation: AttackEquipmentOperation::Unequip { item: weapon },
    });
    apply(&mut owner, human, TacticalAction::Attack { choice });
    submit(&mut owner, 19);
    decline_hit(&mut owner);
    let request = pending(owner.state()).clone();
    apply(
        &mut owner,
        human,
        TacticalAction::ReleaseGrapple { grip: id },
    );
    assert_eq!(pending(owner.state()), &request);
    submit(&mut owner, 1);
    assert_eq!(
        resolution(owner.state())
            .unwrap()
            .attack
            .as_ref()
            .unwrap()
            .stage,
        TacticalAttackStage::KnockoutChoice
    );
    let nodes = resolution(owner.state())
        .unwrap()
        .work_trace
        .as_ref()
        .unwrap()
        .nodes
        .clone();
    assert!(
        nodes
            .iter()
            .any(|n| n.work.kind == TacticalWorkKind::AttackDamage)
    );
    let occurrence = resolution(owner.state()).unwrap().next_occurrence;
    let now = owner.state().clock.now;
    let knockout = apply(
        &mut owner,
        human,
        TacticalAction::ChooseAttackKnockout {
            choice: KnockoutChoice::KnockOut,
        },
    );
    assert!(flow(owner.state()).unwrap().resolution.is_none());
    assert!(
        !admission::loadout(owner.state(), human)
            .unwrap()
            .hands
            .hands
            .contains(&HandAssignment::Item(weapon))
    );
    assert_eq!(owner.state().items[&weapon].custody, Custody::Entity(human));
    let rules = owner.state().rules.as_ref().unwrap();
    let target = &rules.entities[&goblin];
    assert_eq!(target.hp, 1);
    assert!(target.prone);
    assert_eq!(target.death, DeathState::default());
    assert!(crate::active_conditions(rules, goblin).contains(&Condition::Unconscious));
    let origin = VitalityOrigin {
        command: knockout.clone(),
        occurrence,
    };
    assert_eq!(
        rules.tactical_recovery.as_ref().unwrap()[&goblin],
        TacticalRecovery {
            knockout: Some(KnockoutRecovery {
                origin: origin.clone(),
                inflicted_at: now,
                short_rest_started_at: Some(now),
            }),
            stable: None,
            knockout_rest: Some(KnockoutRestAuthorization {
                knockout_origin: origin.clone(),
                started_by: origin,
                started_at: now,
            }),
        }
    );
    assert_eq!(
        rules
            .rests
            .iter()
            .filter(|rest| rest.actor == goblin)
            .collect::<Vec<_>>(),
        vec![&RestProgress {
            actor: goblin,
            kind: RestKind::Short,
            started_at: now,
        }]
    );
    assert_eq!(
        admission::loadout(owner.state(), human).unwrap().command,
        knockout
    );
    assert_eq!(flow(owner.state()).unwrap().budget.weapon_history.len(), 1);
}

#[test]
fn savage_actual_new_damage_observation_preserves_inherited_grapple_raw() {
    let pack = crate::tactical_hands::tests::pack();
    let (state, human, goblin) = baseline(&pack);
    let mut owner = GuardedGrappleExecution::new(state, &pack).unwrap();
    grip(&mut owner, human, goblin);
    let inherited = owner.state().rules.as_ref().unwrap().rolls.clone();
    next_round(&mut owner, human);
    let choice = dagger(owner.state(), human, goblin);
    apply(&mut owner, human, TacticalAction::Attack { choice });
    submit(&mut owner, 19);
    decline_hit(&mut owner);
    let request = pending(owner.state()).clone();
    let roll = SavageAttackerRoll {
        weapon_dice: Some(1),
        first: faces(&request, 1),
        second: faces(&request, 4),
        chosen: DamageRollChoice::First,
        inspiration: None,
    };
    apply(
        &mut owner,
        human,
        TacticalAction::SubmitSavageAttacker { roll: roll.clone() },
    );
    assert!(
        owner
            .state()
            .rules
            .as_ref()
            .unwrap()
            .rolls
            .starts_with(&inherited)
    );
    assert_eq!(
        owner
            .state()
            .rules
            .as_ref()
            .unwrap()
            .rolls
            .last()
            .unwrap()
            .savage_attacker,
        Some(roll)
    );
    apply(
        &mut owner,
        human,
        TacticalAction::ChooseAttackKnockout {
            choice: KnockoutChoice::KnockOut,
        },
    );
}

#[test]
fn public_import_and_unsupported_owner_actions_stay_atomic_after_real_attack_admission() {
    let pack = crate::tactical_hands::tests::pack();
    let (state, human, goblin) = baseline(&pack);
    let mut owner = GuardedGrappleExecution::new(state, &pack).unwrap();
    grip(&mut owner, human, goblin);
    let before = owner.state().clone();
    let meta = command(owner.state(), Some(human));
    assert!(
        owner
            .apply(&meta, &TacticalAction::Move { path: vec![] })
            .is_err()
    );
    assert_eq!(owner.state(), &before);
    next_round(&mut owner, human);
    let choice = dagger(owner.state(), human, goblin);
    apply(&mut owner, human, TacticalAction::Attack { choice });
    let before = owner.state().clone();
    let meta = command(&before, Some(human));
    let result = faces(pending(&before), 1);
    assert!(
        owner
            .apply(
                &meta,
                &TacticalAction::SubmitRollWithInspiration {
                    result: result.clone(),
                    die_index: 0,
                    replacement: DieResult {
                        sides: 20,
                        value: 20
                    },
                }
            )
            .is_err(),
        "this real source actor has no Inspiration grant"
    );
    assert_eq!(owner.state(), &before);
    for policy in [ExecutionPolicy::Live, ExecutionPolicy::Historical] {
        assert!(
            crate::tactical::resolve_with_policy(
                &before,
                &meta,
                &TacticalAction::SubmitRoll {
                    result: result.clone()
                },
                &pack,
                policy
            )
            .is_err()
        );
    }
    let restored = serde_json::from_str(&serde_json::to_string(&before).unwrap()).unwrap();
    assert!(GuardedGrappleExecution::new(restored, &pack).is_err());
    let mut stale = meta;
    stale.expected_event_sequence -= 1;
    assert!(
        owner
            .apply(&stale, &TacticalAction::SubmitRoll { result })
            .is_err()
    );
    assert_eq!(owner.state(), &before);
}

#[test]
fn actual_non_graze_underwater_automatic_miss_spends_and_drops_without_a_raw_request() {
    let pack = crate::tactical_hands::tests::pack();
    let (state, human, goblin, mage) = build_source_scene(&pack, "mage", true);
    let mut owner = GuardedGrappleExecution::new(state, &pack).unwrap();
    grip(&mut owner, human, goblin);
    next_round(&mut owner, human);
    let mut choice = dagger(owner.state(), human, mage);
    choice.delivery = WeaponDelivery::Thrown;
    let weapon = choice.weapon;
    let inherited = owner.state().rules.as_ref().unwrap().rolls.clone();
    apply(&mut owner, human, TacticalAction::Attack { choice });
    assert!(flow(owner.state()).unwrap().resolution.is_none());
    assert!(owner.state().rules.as_ref().unwrap().pending.is_none());
    assert_eq!(owner.state().rules.as_ref().unwrap().rolls, inherited);
    assert!(
        owner
            .state()
            .rules
            .as_ref()
            .unwrap()
            .timing
            .as_ref()
            .unwrap()
            .action_spent
    );
    assert_eq!(
        flow(owner.state()).unwrap().budget.weapon_history[0].outcome,
        WeaponAttackOutcome::Miss
    );
    assert_eq!(
        flow(owner.state())
            .unwrap()
            .ground_items
            .iter()
            .filter(|g| g.item == weapon)
            .count(),
        1
    );
    assert!(
        !admission::loadout(owner.state(), human)
            .unwrap()
            .hands
            .hands
            .contains(&HandAssignment::Item(weapon))
    );
}

#[test]
fn thrown_completion_after_release_uses_current_hands_and_one_finite_ground_item() {
    let pack = crate::tactical_hands::tests::pack();
    let (state, human, goblin) = baseline(&pack);
    let mut owner = GuardedGrappleExecution::new(state, &pack).unwrap();
    let id = grip(&mut owner, human, goblin);
    next_round(&mut owner, human);
    let mut choice = dagger(owner.state(), human, goblin);
    choice.delivery = WeaponDelivery::Thrown;
    let weapon = choice.weapon;
    apply(&mut owner, human, TacticalAction::Attack { choice });
    apply(
        &mut owner,
        human,
        TacticalAction::ReleaseGrapple { grip: id },
    );
    submit(&mut owner, 1);
    assert!(flow(owner.state()).unwrap().resolution.is_none());
    assert_eq!(
        admission::loadout(owner.state(), human).unwrap().hands,
        WeaponLoadout::default()
    );
    let ground = &flow(owner.state()).unwrap().ground_items;
    assert_eq!(ground.iter().filter(|g| g.item == weapon).count(), 1);
    assert_eq!(
        ground.iter().find(|g| g.item == weapon).unwrap().position,
        encounter(owner.state())
            .unwrap()
            .participant(goblin)
            .unwrap()
            .position
    );
    assert!(matches!(
        owner.state().items[&weapon].custody,
        Custody::Location(_)
    ));
}

#[test]
fn unrelated_unarmed_wait_has_no_outgoing_hand_cut_and_release_preserves_its_tuple() {
    let pack = crate::tactical_hands::tests::pack();
    let (state, human, goblin, second) = with_second_human(&pack);
    let mut owner = GuardedGrappleExecution::new(state, &pack).unwrap();
    let id = grip(&mut owner, human, goblin);
    next_turn(&mut owner, second);
    apply(
        &mut owner,
        second,
        TacticalAction::UnarmedStrike { target: human },
    );
    let r = resolution(owner.state()).unwrap();
    assert!(r.grapple.is_none());
    let tuple = (
        owner.state().rules.as_ref().unwrap().pending.clone(),
        r.pending.clone(),
        r.attack.clone(),
        r.frames.clone(),
    );
    apply(
        &mut owner,
        human,
        TacticalAction::ReleaseGrapple { grip: id },
    );
    let r = resolution(owner.state()).unwrap();
    assert!(r.grapple.is_none());
    assert_eq!(
        (
            owner.state().rules.as_ref().unwrap().pending.clone(),
            r.pending.clone(),
            r.attack.clone(),
            r.frames.clone()
        ),
        tuple
    );
    submit(&mut owner, 1);
}
