//! A selected movement crossing grants one reaction attack, never another action.
use super::*;
use crate::tactical_definitions::{
    AttackDelivery, MonsterFeature, WeaponHands, WeaponKind, WeaponProperty,
};

fn ogre_grips(
    program: &crate::tactical_creatures::OgreWeaponProgram,
    hands: &crate::tactical_hands::EffectiveHands,
    loadout: &WeaponLoadout,
    item: ItemId,
) -> Vec<WeaponGrip> {
    if program.delivery() != WeaponDelivery::Melee {
        return vec![];
    }
    match program.weapon().hands {
        WeaponHands::Two if hands.can_use_two_hands(loadout, item) => vec![WeaponGrip::TwoHands],
        WeaponHands::One => [Hand::Left, Hand::Right]
            .into_iter()
            .filter(|hand| hands.holds(loadout, *hand, item))
            .map(WeaponGrip::OneHand)
            .collect(),
        _ => vec![],
    }
}

fn held_source_grips(
    state: &CampaignState,
    actor: EntityId,
    program: &crate::tactical_creatures::OgreWeaponProgram,
    item: ItemId,
) -> Result<Vec<WeaponGrip>, RulesError> {
    if !intrinsic::held(state, actor, item)
        || state.items[&item].definition_id != program.weapon().id
    {
        return Ok(vec![]);
    }
    let rules = state.rules.as_ref().ok_or(RulesError::Uninitialized)?;
    let loadout = rules
        .tactical_inventory
        .as_ref()
        .and_then(|inventory| inventory.loadout(actor))
        .ok_or_else(|| invalid("physical source loadout absent"))?;
    let hands = crate::tactical_hands::EffectiveHands::current(state, rules, actor)?;
    hands.validate_loadout(&loadout.hands)?;
    Ok(ogre_grips(program, &hands, &loadout.hands, item))
}

/// Current physical source candidates, not permission to answer a crossing.
/// The caller must retain the selected window's feature/Item identity; response
/// admission independently checks the crossing, owner, source and physical plan.
pub fn physical_source_opportunity_grips(
    state: &CampaignState,
    actor: EntityId,
    feature_id: &str,
    item: ItemId,
) -> Result<Vec<WeaponGrip>, RulesError> {
    creature_weapon::require_ogre_execution(state)?;
    let profile = intrinsic::profile(state, actor)?;
    crate::tactical_creatures::source_for_profile(profile)
        .map_err(|error| invalid(&error.to_string()))?;
    let program = crate::tactical_creatures::ogre_weapon_program(&profile.source, feature_id)
        .map_err(|error| invalid(&error.to_string()))?;
    held_source_grips(state, actor, &program, item)
}

/// Departure distance is derived by the movement evaluator, never a controller
/// modifier. Irrelevant reach options must not consult uncertain cover at all.
pub(in crate::tactical) fn opportunity_options_for_crossing(
    state: &CampaignState,
    actor: EntityId,
    mover: EntityId,
    after_distance: u32,
) -> Result<Vec<TacticalMeleeOption>, RulesError> {
    // A capability query must not reveal geometry behind an unknown truth ID.
    if planning::require_located_target(state, actor, mover).is_err() {
        return Ok(vec![]);
    }
    let rules = state.rules.as_ref().ok_or(RulesError::Uninitialized)?;
    if !crate::tactical_conditions::can_act(rules, actor)?
        || !crate::tactical_conditions::may_harm(rules, actor, mover)
    {
        return Ok(vec![]);
    }
    let participant = encounter(state)?
        .participant(actor)
        .ok_or_else(|| invalid("reactor absent"))?;
    let target = encounter(state)?
        .participant(mover)
        .ok_or_else(|| invalid("mover absent"))?;
    let definitions = definitions()?;
    let loadout = rules
        .tactical_inventory
        .as_ref()
        .and_then(|i| i.loadout(actor));
    let mut result = vec![TacticalMeleeOption {
        source: TacticalMeleeSource::Unarmed,
        reach: 10,
    }];
    if let Some(loadout) = loadout {
        let hands = crate::tactical_hands::EffectiveHands::current(state, rules, actor)?;
        hands.validate_loadout(&loadout.hands)?;
        let mut held = loadout
            .hands
            .hands
            .iter()
            .filter_map(|h| {
                if let HandAssignment::Item(id) = h {
                    Some(*id)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        held.sort_by_key(|id| id.0);
        held.dedup();
        for item in held {
            if !intrinsic::held(state, actor, item) {
                continue;
            }
            let Some(weapon) = definitions.weapon(&state.items[&item].definition_id) else {
                continue;
            };
            if weapon.kind != WeaponKind::Melee
                || (weapon.hands != WeaponHands::One
                    && !hands.can_use_two_hands(&loadout.hands, item))
            {
                continue;
            }
            result.push(TacticalMeleeOption {
                source: TacticalMeleeSource::Weapon { item },
                reach: participant.reach
                    + if weapon.properties.contains(&WeaponProperty::Reach) {
                        10
                    } else {
                        0
                    },
            });
        }
    }
    if let Ok(profile) = intrinsic::profile(state, actor) {
        let source = crate::tactical_creatures::source_for_profile(profile)
            .map_err(|e| invalid(&e.to_string()))?;
        let mut features = source.features.iter().collect::<Vec<_>>();
        features.sort_by(|a, b| a.id.cmp(&b.id));
        for feature in features {
            let MonsterFeature::Attack {
                delivery: AttackDelivery::Melee { reach_feet },
                ..
            } = feature.feature
            else {
                continue;
            };
            if feature.usage.is_some() {
                continue;
            }
            let required =
                crate::tactical_creature_equipment::creature_attack_gear(profile, &feature.id)?;
            if source.id == "ogre" {
                creature_weapon::require_ogre_execution(state)?;
                let program =
                    crate::tactical_creatures::ogre_weapon_program(&profile.source, &feature.id)
                        .map_err(|error| invalid(&error.to_string()))?;
                if loadout.is_some() {
                    let mut items = vec![];
                    for item in state.items.values() {
                        if !held_source_grips(state, actor, &program, item.id)?.is_empty() {
                            items.push(item.id);
                        }
                    }
                    items.sort_by_key(|item| item.0);
                    result.extend(items.into_iter().map(|item| TacticalMeleeOption {
                        source: TacticalMeleeSource::CreatureWeapon {
                            feature_id: feature.id.clone(),
                            item,
                        },
                        reach: u32::from(reach_feet) * 2,
                    }));
                }
                // Never emit the gripless historical source option for this program.
                continue;
            }
            let items = match required {
                None => vec![None],
                Some(definition) => {
                    let mut items = state
                        .items
                        .values()
                        .filter(|item| {
                            item.definition_id == definition
                                && intrinsic::held(state, actor, item.id)
                        })
                        .map(|item| Some(item.id))
                        .collect::<Vec<_>>();
                    items.sort_by_key(|id| id.map(|id| id.0));
                    items
                }
            };
            result.extend(items.into_iter().map(|weapon| TacticalMeleeOption {
                source: TacticalMeleeSource::CreatureFeature {
                    feature_id: feature.id.clone(),
                    weapon,
                },
                reach: u32::from(reach_feet) * 2,
            }));
        }
    }
    let distance = crate::spatial::participant_distance(participant, target)
        .map_err(|e| invalid(&e.to_string()))?;
    result.retain(|option| distance <= option.reach && option.reach < after_distance);
    if result.is_empty() {
        return Ok(result);
    }
    let cover = crate::spatial::cover_from(
        encounter(state)?,
        participant.center().map_err(|e| invalid(&e))?,
        target.volume().map_err(|e| invalid(&e))?,
        &[actor, mover],
    )
    .map_err(|e| invalid(&e.to_string()))?;
    if cover.requires_adjudication {
        return Err(prerequisite(
            "opportunity attack cover needs an authored geometry ruling",
        ));
    }
    if cover.degree == CoverDegree::Total {
        return Ok(vec![]);
    }
    // Unarmed, then physical ItemIds, then canonical feature and implement IDs.
    // Each input group is explicitly sorted; HashMap iteration never sets replay order.
    Ok(result)
}

pub(in crate::tactical) fn begin_opportunity_attack(
    state: &mut CampaignState,
    meta: &CommandMeta,
    actor: EntityId,
    target: EntityId,
    choice: &TacticalMeleeChoice,
    pack: &RulesPack,
) -> Result<(), RulesError> {
    if matches!(choice, TacticalMeleeChoice::CreatureWeapon { .. }) {
        creature_weapon::require_ogre_execution(state)?;
    }
    authorize(state, meta, actor)?;
    planning::admit_target(state, actor, target)?;
    let window = super::super::movement::validate_opportunity(state, actor, target)?.clone();
    let selected = match choice {
        TacticalMeleeChoice::Weapon(c) => {
            if c.target != target
                || c.delivery != WeaponDelivery::Melee
                || c.purpose != WeaponAttackPurpose::Normal
                || c.equipment_change.is_some()
                || c.after_equipment.is_some()
                || c.ammunition.is_some()
            {
                return Err(prerequisite(
                    "opportunity attack uses one currently held melee weapon without free equipment changes",
                ));
            }
            TacticalMeleeSource::Weapon { item: c.weapon }
        }
        TacticalMeleeChoice::UnarmedDamage { .. } => TacticalMeleeSource::Unarmed,
        TacticalMeleeChoice::CreatureFeature { feature_id, weapon } => {
            TacticalMeleeSource::CreatureFeature {
                feature_id: feature_id.clone(),
                weapon: *weapon,
            }
        }
        TacticalMeleeChoice::CreatureWeapon {
            feature_id, weapon, ..
        } => TacticalMeleeSource::CreatureWeapon {
            feature_id: feature_id.clone(),
            item: *weapon,
        },
    };
    if !window.options.iter().any(|o| o.source == selected) {
        return Err(prerequisite(
            "selected implement did not trigger this crossing",
        ));
    }
    let source = match choice {
        TacticalMeleeChoice::Weapon(c) => {
            TacticalAttackSource::Weapon(Box::new(TacticalWeaponAttack {
                after_equipment_parent: None,
                ground_pickup_before: None,
                choice: c.clone(),
                window: WeaponActionWindow {
                    id: meta.id,
                    kind: WeaponActionKind::Reaction,
                },
                equipment_before: state
                    .rules
                    .as_ref()
                    .and_then(|r| r.tactical_inventory.as_ref())
                    .and_then(|i| i.loadout(actor))
                    .cloned()
                    .ok_or_else(|| prerequisite("current equipment required"))?,
                ammunition: None,
            }))
        }
        TacticalMeleeChoice::UnarmedDamage { ability } => {
            TacticalAttackSource::Unarmed { ability: *ability }
        }
        TacticalMeleeChoice::CreatureFeature { feature_id, weapon } => {
            TacticalAttackSource::CreatureFeature {
                source: intrinsic::profile(state, actor)?.source.clone(),
                feature_id: feature_id.clone(),
                weapon: *weapon,
            }
        }
        TacticalMeleeChoice::CreatureWeapon {
            feature_id,
            weapon,
            grip,
        } => creature_weapon::opportunity_source(
            state, actor, target, meta, feature_id, *weapon, *grip,
        )?,
    };
    let mut attack = TacticalAttack {
        origin: meta.clone(),
        actor,
        target,
        delivery: TacticalAttackDelivery::Melee,
        source,
        admission: TacticalAttackAdmission::Opportunity(Box::new(window)),
        attack_modifier: 0,
        mode: RollMode::Normal,
        armor_class: 0,
        critical_on_hit: false,
        automatic_miss: false,
        damage: vec![],
        stage: TacticalAttackStage::AttackRoll,
        attack_roll: None,
        damage_roll: None,
        outcome: None,
    };
    let plan = if let Some(weapon) = attack.weapon() {
        let plan = planning::weapon_plan(
            state,
            meta,
            actor,
            &weapon.choice,
            weapon.window,
            &weapon.equipment_before.hands,
            pack,
        )?;
        if plan
            .mastery
            .is_some_and(|m| !matches!(m, WeaponMastery::Nick | WeaponMastery::Graze))
        {
            return Err(prerequisite(
                "this mastery requires its typed continuation before any attack cost",
            ));
        }
        let (mode, armor, critical) = planning::hit_facts(state, actor, &weapon.choice, &plan)?;
        attack.attack_modifier = plan.attack_modifier;
        attack.mode = mode;
        attack.armor_class = armor;
        attack.critical_on_hit = critical;
        attack.damage = if matches!(attack.source, TacticalAttackSource::CreatureWeapon { .. }) {
            creature_weapon::validate_source(state, &attack, &plan)?
        } else {
            vec![AttackDamageComponent {
                damage_type: plan.damage.damage_type,
                dice: plan.damage.dice.clone(),
                modifier: plan.damage.modifier,
            }]
        };
        Some(plan)
    } else {
        let plan = intrinsic::plan(state, &attack)?;
        attack.attack_modifier = plan.modifier;
        attack.mode = plan.mode;
        attack.armor_class = plan.armor;
        attack.critical_on_hit = plan.critical;
        attack.damage = plan.damage;
        None
    };
    let rules = state.rules.as_mut().ok_or(RulesError::Uninitialized)?;
    crate::tactical_budget::spend_cost(
        rules,
        actor,
        crate::tactical_budget::TacticalCost::Reaction,
    )?;
    crate::kernel::interrupt_rest(rules, actor, state.clock.now);
    if let Some(plan) = plan {
        let loadout = rules
            .tactical_inventory
            .as_mut()
            .and_then(|i| i.loadouts.iter_mut().find(|l| l.actor == actor))
            .ok_or_else(|| invalid("equipment absent"))?;
        loadout.hands = plan.loadout_for_attack;
        loadout.command = meta.clone();
        flow_mut(state)?.budget.weapon_history.push(plan.receipt);
    }
    super::super::movement::record_attack(state, meta, actor)?;
    resolution_mut(state)?.attack = Some(attack);
    push_frame(state, vec![TacticalWorkKind::AttackRoll])?;
    pump(state, meta)
}

pub(super) fn validate_admission(
    state: &CampaignState,
    attack: &TacticalAttack,
) -> Result<(), RulesError> {
    let resolution = resolution(state)?;
    match &attack.admission {
        TacticalAttackAdmission::UnarmedAction { window } => {
            super::unarmed::validate_admission(state, attack, *window)?;
        }
        TacticalAttackAdmission::OwnTurn => {
            if attack.origin != resolution.origin
                || attack.actor != resolution.turn_actor
                || resolution.movement.is_some()
                || attack.weapon().is_none()
            {
                return Err(invalid("ordinary attack differs from its own-turn origin"));
            }
        }
        TacticalAttackAdmission::CreatureAction { .. } => {
            creature::validate_admission(state, attack)?
        }
        TacticalAttackAdmission::Opportunity(window) => {
            validate_equipment_change_origin(state, &window.origin, window.reactor)
                .map_err(|e| invalid(&e))?;
            let rules = state.rules.as_ref().ok_or(RulesError::Uninitialized)?;
            let movement = resolution
                .movement
                .as_ref()
                .ok_or_else(|| invalid("reaction attack lacks movement parent"))?;
            if window.reactor != attack.actor
                || window.mover != attack.target
                || movement.actor != attack.target
                || !rules
                    .timing
                    .as_ref()
                    .is_some_and(|t| t.reactions_spent.contains(&attack.actor))
                || !movement.decisions.iter().any(|d| {
                    d.reactor == attack.actor
                        && d.origin == attack.origin
                        && d.kind == TacticalOpportunityDecisionKind::Attack
                })
                || window.origin.expected_event_sequence > attack.origin.expected_event_sequence
                || window.origin.expected_event_sequence < movement.origin.expected_event_sequence
            {
                return Err(invalid(
                    "reaction attack differs from accepted crossing/response/cost",
                ));
            }
            let mut before = state.clone();
            before
                .rules
                .as_mut()
                .unwrap()
                .timing
                .as_mut()
                .unwrap()
                .reactions_spent
                .retain(|id| *id != attack.actor);
            let r = resolution_mut(&mut before)?;
            r.attack = None;
            r.pending = None;
            r.movement.as_mut().unwrap().opportunity = Some(window.as_ref().clone());
            // Equipping a two-handed grip can change hand assignment but does not
            // change which held implement triggered. Restore the accepted before-image.
            if let Some(weapon) = attack.weapon() {
                let loadout = before
                    .rules
                    .as_mut()
                    .and_then(|r| r.tactical_inventory.as_mut())
                    .and_then(|i| i.loadouts.iter_mut().find(|l| l.actor == attack.actor))
                    .ok_or_else(|| invalid("reaction before-equipment absent"))?;
                *loadout = weapon.equipment_before.clone();
            }
            super::super::movement::validate_opportunity(&before, attack.actor, attack.target)?;
            let source = match &attack.source {
                TacticalAttackSource::Weapon(w) => TacticalMeleeSource::Weapon {
                    item: w.choice.weapon,
                },
                TacticalAttackSource::CreatureWeapon {
                    source,
                    feature_id,
                    weapon,
                } => {
                    creature_weapon::require_ogre_execution(state)?;
                    let program =
                        crate::tactical_creatures::ogre_weapon_program(source, feature_id)
                            .map_err(|error| invalid(&error.to_string()))?;
                    if program.delivery() != WeaponDelivery::Melee {
                        return Err(invalid("physical source opportunity is not a melee form"));
                    }
                    TacticalMeleeSource::CreatureWeapon {
                        feature_id: feature_id.clone(),
                        item: weapon.choice.weapon,
                    }
                }
                TacticalAttackSource::Unarmed { .. } => TacticalMeleeSource::Unarmed,
                TacticalAttackSource::CreatureFeature {
                    feature_id, weapon, ..
                } => TacticalMeleeSource::CreatureFeature {
                    feature_id: feature_id.clone(),
                    weapon: *weapon,
                },
                TacticalAttackSource::Spell { .. } => {
                    return Err(invalid("a spell is not this melee opportunity source"));
                }
            };
            if !window.options.iter().any(|o| o.source == source) {
                return Err(invalid("reaction implement differs from retained crossing"));
            }
        }
        TacticalAttackAdmission::Spell { .. } => spell::validate_admission(state, attack)?,
    }
    Ok(())
}

#[cfg(test)]
mod hand_tests {
    use super::*;

    #[test]
    fn closed_source_grip_candidates_use_held_and_supporting_hands() {
        // Pure grip composition using actual Human hand context and hypothetical
        // physical assignments, not Ogre materialization or accepted OA/Grapple play.
        let mut state = crate::tactical_hands::tests::source_state();
        let actor = crate::tactical_hands::tests::human(&state);
        let pin = crate::tactical_creatures::creature_source_pin(
            crate::tactical_definitions::bundled_ogre().unwrap(),
        )
        .unwrap();
        let greatclub = crate::tactical_creatures::ogre_weapon_program(&pin, "greatclub").unwrap();
        let javelin =
            crate::tactical_creatures::ogre_weapon_program(&pin, "javelin-melee").unwrap();
        let thrown =
            crate::tactical_creatures::ogre_weapon_program(&pin, "javelin-thrown").unwrap();
        let item = ItemId::new();
        let mut held = WeaponLoadout {
            hands: [HandAssignment::Item(item), HandAssignment::Free],
        };
        let hands = crate::tactical_hands::EffectiveHands::current(
            &state,
            state.rules.as_ref().unwrap(),
            actor,
        )
        .unwrap();
        assert_eq!(
            ogre_grips(&greatclub, &hands, &held, item),
            [WeaponGrip::TwoHands]
        );
        assert_eq!(
            ogre_grips(&javelin, &hands, &held, item),
            [WeaponGrip::OneHand(Hand::Left)]
        );
        assert!(ogre_grips(&thrown, &hands, &held, item).is_empty());
        assert!(ogre_grips(&greatclub, &hands, &WeaponLoadout::default(), item).is_empty());
        held.hands[1] = HandAssignment::Item(ItemId::new());
        assert!(ogre_grips(&greatclub, &hands, &held, item).is_empty());
        assert_eq!(
            ogre_grips(&javelin, &hands, &held, item),
            [WeaponGrip::OneHand(Hand::Left)]
        );
        held.hands[1] = HandAssignment::Free;
        crate::tactical_hands::tests::install_attempt(&mut state, actor, Hand::Right);
        let reserved = crate::tactical_hands::EffectiveHands::current(
            &state,
            state.rules.as_ref().unwrap(),
            actor,
        )
        .unwrap();
        assert!(ogre_grips(&greatclub, &reserved, &held, item).is_empty());
        assert_eq!(
            ogre_grips(&javelin, &reserved, &held, item),
            [WeaponGrip::OneHand(Hand::Left)]
        );
    }

    #[test]
    fn shared_physical_source_candidates_recheck_actual_item_and_loadout() {
        // Isolated candidate composition: genuine Human context, hypothetical
        // weapon assignment and immutable Ogre descriptor. No Ogre profile or
        // accepted acquisition/reaction/Grapple history is manufactured.
        let mut state = crate::tactical_hands::tests::source_state();
        let actor = crate::tactical_hands::tests::human(&state);
        let pin = crate::tactical_creatures::creature_source_pin(
            crate::tactical_definitions::bundled_ogre().unwrap(),
        )
        .unwrap();
        let program = crate::tactical_creatures::ogre_weapon_program(&pin, "greatclub").unwrap();
        let loadout = state
            .rules
            .as_mut()
            .unwrap()
            .tactical_inventory
            .as_mut()
            .unwrap()
            .loadouts
            .iter_mut()
            .find(|loadout| loadout.actor == actor)
            .unwrap();
        let item = loadout
            .hands
            .hands
            .iter()
            .find_map(|hand| match hand {
                HandAssignment::Item(item) => Some(*item),
                _ => None,
            })
            .unwrap();
        loadout.hands.hands = [HandAssignment::Item(item), HandAssignment::Free];
        state.items.get_mut(&item).unwrap().definition_id = "greatclub".into();
        assert_eq!(
            held_source_grips(&state, actor, &program, item).unwrap(),
            [WeaponGrip::TwoHands]
        );
        let before = state.clone();
        for corruption in 0..7 {
            let mut altered = before.clone();
            let weapon = altered.items.get_mut(&item).unwrap();
            match corruption {
                0 => weapon.quantity = 2,
                1 => weapon.state = ItemState::Damaged,
                2 => weapon.custody = Custody::Entity(EntityId::new()),
                3 => weapon.definition_id = "javelin".into(),
                4 => weapon.campaign_id = CampaignId::new(),
                5 => weapon.id = ItemId::new(),
                6 => {
                    altered
                        .rules
                        .as_mut()
                        .unwrap()
                        .tactical_inventory
                        .as_mut()
                        .unwrap()
                        .loadouts
                        .iter_mut()
                        .find(|loadout| loadout.actor == actor)
                        .unwrap()
                        .hands = WeaponLoadout::default()
                }
                _ => unreachable!(),
            }
            let expected = altered.clone();
            assert!(
                held_source_grips(&altered, actor, &program, item)
                    .unwrap()
                    .is_empty()
            );
            assert_eq!(altered, expected);
        }
        assert_eq!(state, before);
        assert!(
            held_source_grips(&state, actor, &program, ItemId::new())
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn physical_source_query_requires_current_execution_and_the_actual_full_source_pin() {
        let mut state = crate::tactical_hands::tests::source_state();
        let actor = crate::tactical_hands::tests::human(&state);
        let item = ItemId::new();
        let before = state.clone();
        assert!(
            physical_source_opportunity_grips(&state, actor, "greatclub", item)
                .unwrap_err()
                .to_string()
                .contains("current executor")
        );
        assert_eq!(state, before);
        // Explicitly synthetic negative version image, not a journaled upgrade.
        // Keep the genuine Human and old creature profiles/pins unchanged.
        state
            .encounter
            .as_mut()
            .unwrap()
            .flow
            .as_mut()
            .unwrap()
            .version = 5;
        let before = state.clone();
        assert!(
            physical_source_opportunity_grips(&state, actor, "greatclub", item)
                .unwrap_err()
                .to_string()
                .contains("pinned creature profile")
        );
        let profiles = &state
            .rules
            .as_ref()
            .unwrap()
            .tactical_creatures
            .as_ref()
            .unwrap()
            .profiles;
        assert!(!profiles.is_empty());
        for profile in profiles {
            assert!(
                physical_source_opportunity_grips(&state, profile.actor, "greatclub", item)
                    .unwrap_err()
                    .to_string()
                    .contains("exact immutable Ogre pin")
            );
        }
        assert_eq!(state, before);
    }

    #[test]
    fn current_two_hand_options_change_without_removing_unarmed_or_rewriting_selected_work() {
        // Pure current-menu control: the old genuine state supplies source and
        // perception. Glaive acquisition and the Attempt are synthetic; this is
        // not an accepted release, menu refresh or long-reach table scenario.
        let mut state = crate::tactical_hands::tests::source_state();
        let actor = crate::tactical_hands::tests::human(&state);
        let target = state
            .encounter
            .as_ref()
            .unwrap()
            .participants
            .iter()
            .find(|p| p.entity_id != actor)
            .unwrap()
            .entity_id;
        let loadout = state
            .rules
            .as_mut()
            .unwrap()
            .tactical_inventory
            .as_mut()
            .unwrap()
            .loadouts
            .iter_mut()
            .find(|l| l.actor == actor)
            .unwrap();
        let item = loadout
            .hands
            .hands
            .iter()
            .find_map(|h| match h {
                HandAssignment::Item(id) => Some(*id),
                _ => None,
            })
            .unwrap();
        loadout.hands.hands = [HandAssignment::Item(item), HandAssignment::Free];
        state.items.get_mut(&item).unwrap().definition_id = "glaive".into();
        let original = opportunity_options_for_crossing(&state, actor, target, 30).unwrap();
        assert!(
            original
                .iter()
                .any(|o| o.source == TacticalMeleeSource::Unarmed)
        );
        assert!(
            original
                .iter()
                .any(|o| o.source == (TacticalMeleeSource::Weapon { item }) && o.reach == 20)
        );
        crate::tactical_hands::tests::install_attempt(&mut state, actor, Hand::Right);
        let blocked = opportunity_options_for_crossing(&state, actor, target, 30).unwrap();
        assert_eq!(
            blocked,
            vec![TacticalMeleeOption {
                source: TacticalMeleeSource::Unarmed,
                reach: 10
            }]
        );
        let resolution = state
            .encounter
            .as_mut()
            .unwrap()
            .flow
            .as_mut()
            .unwrap()
            .resolution
            .as_mut()
            .unwrap();
        let context = resolution.grapple.as_mut().unwrap();
        let Some(GrappleActivity::Attempt(attempt)) = context.activity.as_mut() else {
            unreachable!()
        };
        attempt.stage = TacticalGrappleAttemptStage::Complete;
        attempt.outcome = Some(GrappleAttemptOutcome::Withdrawn {
            withdrawn_by: attempt.declaration.origin.clone(),
            cancelled: None,
        });
        assert_eq!(
            opportunity_options_for_crossing(&state, actor, target, 30).unwrap(),
            original
        );
        assert!(
            state
                .encounter
                .as_ref()
                .unwrap()
                .flow
                .as_ref()
                .unwrap()
                .resolution
                .as_ref()
                .unwrap()
                .grapple
                .as_ref()
                .unwrap()
                .opportunity_refreshes
                .is_empty()
        );
    }
}
