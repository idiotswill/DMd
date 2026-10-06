//! Pure category/composition controls. The synthetic Hold Person -> Fiend rows
//! below are deliberately NOT accepted gameplay or positive MR history evidence.
use super::*;
use crate::tactical_conditions::{DodgeContext, TestDisposition, save_conditions};
use crate::tactical_creatures::*;

fn fixture() -> (
    CampaignState,
    CommandMeta,
    EntityId,
    EntityId,
    SpellCastPlan,
) {
    let mut state = CampaignState::empty(
        Campaign {
            id: CampaignId::new(),
            display_name: "Private category mechanism".into(),
            status: CampaignStatus::Active,
            world_seed: 1,
            ruleset: VersionedRef {
                id: "srd-5.2".into(),
                version: "5.2.1".into(),
            },
            content_packs: vec![],
        },
        WorldClock {
            now: WorldInstant(100),
            calendar_id: "seconds".into(),
        },
    );
    state.applied_event_sequence = 10;
    let meta = CommandMeta {
        id: CommandId::new(),
        campaign_id: state.campaign_id(),
        session_id: None,
        issuer: CommandIssuer::Admin,
        actor: None,
        expected_event_sequence: 10,
    };
    let caster = EntityId::new();
    state.entities.insert(
        caster,
        WorldEntity {
            id: caster,
            campaign_id: state.campaign_id(),
            display_name: "Caster".into(),
            kind: EntityKind::Character,
            existence: EntityExistence::Present,
            location_id: None,
        },
    );
    let mut entity = MechanicalEntity::basic(caster);
    entity.level = 5;
    entity.ability_scores[Ability::Wisdom.index()] = 16;
    entity.prepared_spells.insert("hold-person".into());
    entity.spellcasting = Some(Spellcasting {
        ability: Ability::Wisdom,
        slot_maxima: [4; 9],
        slots: [4; 9],
        can_speak: true,
        free_hand: true,
        material_focus: true,
    });
    state.rules = Some(RulesState {
        pack_id: "srd-5.2".into(),
        pack_version: "5.2.1".into(),
        entities: [(caster, entity)].into(),
        house_rules: HouseRules::default(),
        effects: vec![],
        tactical_effects: None,
        tactical_inventory: None,
        tactical_recovery: None,
        tactical_creatures: Some(TacticalCreatures::default()),
        pending: None,
        rolls: vec![],
        cancelled_roll_ids: vec![],
        rulings: vec![],
        timing: None,
        rests: vec![],
        completed_short_rests: vec![],
        permission: None,
    });
    let current = add_source(
        &mut state,
        &meta,
        crate::tactical_definitions::bundled_night_hag_v2().unwrap(),
        CreatureSize::Medium,
    );
    let old = add_source(
        &mut state,
        &meta,
        creature_definition("night-hag").unwrap(),
        CreatureSize::Medium,
    );
    let plan = crate::tactical_spells::plan_spell_cast(
        &state,
        &meta,
        &SpellCastChoice {
            actor: caster,
            spell_id: "hold-person".into(),
            grant: SpellGrantChoice::Prepared,
            resource: SpellResourceChoice::Slot { level: 2 },
            material: SpellMaterialChoice::None,
            mode: SpellCastMode::Immediate,
        },
    )
    .unwrap();
    (state, meta, current, old, plan)
}

fn add_source(
    state: &mut CampaignState,
    meta: &CommandMeta,
    source: &crate::tactical_definitions::CreatureDefinition,
    size: CreatureSize,
) -> EntityId {
    let actor = EntityId::new();
    state.entities.insert(
        actor,
        WorldEntity {
            id: actor,
            campaign_id: state.campaign_id(),
            display_name: "Unrelated name".into(),
            kind: EntityKind::Creature,
            existence: EntityExistence::Present,
            location_id: None,
        },
    );
    let built = build_creature_from_source(
        state,
        meta,
        actor,
        &CreatureBuildChoice {
            definition_id: source.id.clone(),
            size,
            additional_languages: vec![],
            hit_points: CreatureHitPointChoice::Average,
            controller: CreatureController::Autonomous,
            in_lair: false,
        },
        Some(&creature_source_pin(source).unwrap()),
    )
    .unwrap();
    let rules = state.rules.as_mut().unwrap();
    rules.entities.insert(actor, built.mechanics);
    let creatures = rules.tactical_creatures.as_mut().unwrap();
    creatures.profiles.push(built.profile);
    creatures.runtime.push(built.runtime);
    actor
}

#[test]
fn exact_trait_changes_only_authenticated_spell_category_and_uses_normal_composition() {
    let (mut state, meta, current, old, plan) = fixture();
    let magical = circumstances(&state, current, Ability::Wisdom, SaveCause::Spell(&plan)).unwrap();
    assert!(magical.advantage && !magical.disadvantage);
    assert_eq!(
        circumstances(&state, old, Ability::Wisdom, SaveCause::Spell(&plan)).unwrap(),
        Circumstances::default()
    );
    assert_eq!(
        circumstances(
            &state,
            current,
            Ability::Constitution,
            SaveCause::Concentration
        )
        .unwrap(),
        Circumstances::default()
    );
    let chimera = add_source(
        &mut state,
        &meta,
        creature_definition("chimera").unwrap(),
        CreatureSize::Large,
    );
    let breath =
        crate::tactical_areas::source_area_program(&state, chimera, "fire-breath").unwrap();
    assert_eq!(
        circumstances(
            &state,
            current,
            Ability::Dexterity,
            SaveCause::Area(&breath)
        )
        .unwrap(),
        Circumstances::default()
    );
    assert!(circumstances(&state, current, Ability::Wisdom, SaveCause::Area(&breath)).is_err());
    let mut forged = plan.clone();
    forged.program.source.fingerprint.push('0');
    assert!(circumstances(&state, current, Ability::Wisdom, SaveCause::Spell(&forged)).is_err());
    // Old actors retain the old path; their pre-existing source validators still run.
    assert_eq!(
        circumstances(&state, old, Ability::Wisdom, SaveCause::Spell(&forged)).unwrap(),
        Circumstances::default()
    );
    let rules = state.rules.as_mut().unwrap();
    assert_eq!(
        save_conditions(
            rules,
            current,
            Ability::Wisdom,
            magical,
            DodgeContext::default()
        )
        .unwrap(),
        TestDisposition::Roll(RollMode::Advantage)
    );
    assert_eq!(
        save_conditions(
            rules,
            current,
            Ability::Wisdom,
            Circumstances {
                disadvantage: true,
                ..magical
            },
            DodgeContext::default()
        )
        .unwrap(),
        TestDisposition::Roll(RollMode::Normal)
    );
    // Composition is generic; this is not a claim that Hold Person has a Dex save.
    rules.effects.push(ActiveEffect {
        id: EffectId::new(),
        source: plan.choice.actor,
        target: current,
        condition: Some(Condition::Restrained),
        label: "Synthetic composition".into(),
        expires: Expiry::Never,
        concentration_owner: None,
    });
    assert_eq!(
        save_conditions(
            rules,
            current,
            Ability::Dexterity,
            magical,
            DodgeContext::default()
        )
        .unwrap(),
        TestDisposition::Roll(RollMode::Normal)
    );
    rules.effects[0].condition = Some(Condition::Paralyzed);
    assert_eq!(
        save_conditions(
            rules,
            current,
            Ability::Dexterity,
            magical,
            DodgeContext::default()
        )
        .unwrap(),
        TestDisposition::AutomaticFailure
    );
}

fn synthetic_repeated(
    state: &mut CampaignState,
    target: EntityId,
    plan: &SpellCastPlan,
) -> ScheduledEffectTrigger {
    let source = EffectSource {
        definition_id: "hold-person".into(),
        actor: plan.choice.actor,
        command: plan.origin.clone(),
        ordinal: plan.occurrence,
    };
    let at = SpellProgramOccurrence { node: 0, target: 3 };
    let id = spell_program_effect_id(source.command.id, source.actor, source.ordinal, at, false);
    let view = spell_program_effect_id(source.command.id, source.actor, source.ordinal, at, true);
    let group = spell_concentration_id(source.command.id, source.actor, source.ordinal);
    let payload = EffectTriggerPayload::SavingThrow {
        ability: Ability::Wisdom,
        dc: 14,
        on_success: EffectSaveEnd::TargetEffect,
        on_failure: EffectSaveEnd::None,
    };
    let origin = EffectOperationStamp {
        command: CommandMeta {
            id: CommandId::new(),
            expected_event_sequence: 12,
            ..plan.origin.clone()
        },
        step: 0,
    };
    let ticket = ScheduledEffectTrigger {
        id: EffectTicketId {
            command: origin.command.id,
            step: 0,
            ordinal: 0,
        },
        effect: id,
        rule_index: Some(0),
        source: source.clone(),
        target,
        payload: payload.clone(),
        cause: EffectObservation::Turn(EffectTurn {
            actor: target,
            number: 5,
            boundary: TurnBoundary::End,
        }),
        origin,
    };
    state.rules.as_mut().unwrap().tactical_effects = Some(TacticalEffects {
        groups: vec![ConcentrationGroup {
            id: group,
            source: source.clone(),
            expires: TacticalEffectExpiry::AtTime(WorldInstant(160)),
            stage: ConcentrationStage::Active,
        }],
        effects: vec![TacticalEffect {
            id,
            source,
            established_at: Some(EffectOperationStamp {
                command: CommandMeta {
                    id: CommandId::new(),
                    expected_event_sequence: 11,
                    ..plan.origin.clone()
                },
                step: 0,
            }),
            target: TacticalEffectTarget::Creature(target),
            concentration_group: Some(group),
            expires: TacticalEffectExpiry::AtTime(WorldInstant(160)),
            overlap: Some(EffectOverlap {
                key: "hold-person".into(),
                potency: 14,
            }),
            conditions: vec![EffectCondition {
                id: view,
                condition: Condition::Paralyzed,
            }],
            defenses: vec![],
            triggers: vec![EffectTriggerRule {
                event: EffectTriggerEvent::Turn {
                    subject: EffectSubject::Target,
                    boundary: TurnBoundary::End,
                },
                frequency: EffectTriggerFrequency::EveryOccurrence,
                payload,
            }],
        }],
        pending: vec![ticket.clone()],
        ..TacticalEffects::default()
    });
    ticket
}

#[test]
fn synthetic_retained_clause_needs_no_live_cast_but_rejects_altered_source_shape() {
    let (mut state, _, current, old, plan) = fixture();
    let ticket = synthetic_repeated(&mut state, current, &plan);
    assert!(
        state.encounter.is_none(),
        "no retained casting record is used"
    );
    state.clock.now = WorldInstant(130);
    state
        .rules
        .as_mut()
        .unwrap()
        .entities
        .get_mut(&plan.choice.actor)
        .unwrap()
        .ability_scores[Ability::Wisdom.index()] = 8;
    assert!(
        circumstances(
            &state,
            current,
            Ability::Wisdom,
            SaveCause::Repeated(&ticket)
        )
        .unwrap()
        .advantage
    );
    for mutation in 0..16 {
        let mut changed = state.clone();
        let attachment = changed
            .rules
            .as_mut()
            .unwrap()
            .tactical_effects
            .as_mut()
            .unwrap();
        let effect = &mut attachment.effects[0];
        match mutation {
            0 => effect.id = EffectId::new(),
            1 => effect.conditions[0].id = EffectId::new(),
            2 => effect.conditions[0].condition = Condition::Restrained,
            3 => effect.conditions.push(effect.conditions[0].clone()),
            4 => effect
                .defenses
                .push(EffectDefense::ArmorClassBonus { bonus: 1 }),
            5 => effect.triggers.push(effect.triggers[0].clone()),
            6 => {
                effect.triggers[0].event = EffectTriggerEvent::Turn {
                    subject: EffectSubject::Source,
                    boundary: TurnBoundary::End,
                }
            }
            7 => {
                effect.triggers[0].event = EffectTriggerEvent::Turn {
                    subject: EffectSubject::Target,
                    boundary: TurnBoundary::Start,
                }
            }
            8 => {
                effect.triggers[0].frequency = EffectTriggerFrequency::OncePerTargetPerTurn {
                    key: "forged".into(),
                }
            }
            9 => effect.overlap.as_mut().unwrap().key = "other".into(),
            10 => effect.overlap.as_mut().unwrap().potency += 1,
            11 => effect.expires = TacticalEffectExpiry::Never,
            12 => effect.concentration_group = None,
            13 => attachment.groups[0].stage = ConcentrationStage::Casting,
            14 => attachment.groups[0].expires = TacticalEffectExpiry::AtTime(WorldInstant(170)),
            _ => effect.source.definition_id = "command".into(),
        }
        assert!(
            circumstances(
                &changed,
                current,
                Ability::Wisdom,
                SaveCause::Repeated(&ticket)
            )
            .is_err(),
            "mutation {mutation}"
        );
        assert_eq!(
            circumstances(&changed, old, Ability::Wisdom, SaveCause::Repeated(&ticket)).unwrap(),
            Circumstances::default()
        );
    }
    for mutation in 0..7 {
        let mut changed = state.clone();
        let mut bad = ticket.clone();
        match mutation {
            0 => bad.rule_index = Some(1),
            1 => bad.source.ordinal += 1,
            2 => bad.target = old,
            3 => bad.cause = EffectObservation::Time,
            4 => {
                bad.payload = EffectTriggerPayload::SavingThrow {
                    ability: Ability::Dexterity,
                    dc: 14,
                    on_success: EffectSaveEnd::TargetEffect,
                    on_failure: EffectSaveEnd::None,
                }
            }
            5 => {
                bad.payload = EffectTriggerPayload::SavingThrow {
                    ability: Ability::Wisdom,
                    dc: 14,
                    on_success: EffectSaveEnd::ConcentrationGroup,
                    on_failure: EffectSaveEnd::None,
                }
            }
            _ => {
                bad.payload = EffectTriggerPayload::SavingThrow {
                    ability: Ability::Wisdom,
                    dc: 14,
                    on_success: EffectSaveEnd::TargetEffect,
                    on_failure: EffectSaveEnd::TargetEffect,
                }
            }
        }
        changed
            .rules
            .as_mut()
            .unwrap()
            .tactical_effects
            .as_mut()
            .unwrap()
            .pending = vec![bad.clone()];
        assert!(
            circumstances(
                &changed,
                current,
                Ability::Wisdom,
                SaveCause::Repeated(&bad)
            )
            .is_err(),
            "ticket mutation {mutation}"
        );
    }
}
