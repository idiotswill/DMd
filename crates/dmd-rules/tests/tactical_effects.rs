use dmd_domain::*;
use dmd_rules::tactical_effects::*;
use std::collections::HashMap;

struct Fixture {
    campaign: CampaignState,
    effects: TacticalEffects,
    caster: EntityId,
    target: EntityId,
    other: EntityId,
}

impl Fixture {
    fn new() -> Self {
        let mut campaign = CampaignState::empty(
            Campaign {
                id: CampaignId::new(),
                display_name: "Lifecycle fixture".into(),
                status: CampaignStatus::Active,
                world_seed: 1,
                ruleset: VersionedRef {
                    id: "srd-5.2".into(),
                    version: "5.2.1".into(),
                },
                content_packs: Vec::new(),
            },
            WorldClock {
                now: WorldInstant(0),
                calendar_id: "test".into(),
            },
        );
        let ids = [EntityId::new(), EntityId::new(), EntityId::new()];
        for id in ids {
            campaign.entities.insert(
                id,
                WorldEntity {
                    id,
                    campaign_id: campaign.campaign_id(),
                    display_name: "Creature".into(),
                    kind: EntityKind::Creature,
                    existence: EntityExistence::Present,
                    location_id: None,
                },
            );
        }
        campaign.rules = Some(RulesState {
            tactical_recovery: None,
            pack_id: "srd-5.2".into(),
            pack_version: "5.2.1".into(),
            entities: ids
                .into_iter()
                .map(|id| (id, MechanicalEntity::basic(id)))
                .collect::<HashMap<_, _>>(),
            house_rules: HouseRules::default(),
            effects: Vec::new(),
            tactical_effects: None,
            tactical_inventory: None,
            tactical_creatures: None,
            pending: None,
            rolls: Vec::new(),
            cancelled_roll_ids: Vec::new(),
            rulings: Vec::new(),
            timing: None,
            rests: Vec::new(),
            completed_short_rests: Vec::new(),
            permission: None,
        });
        Self {
            campaign,
            effects: TacticalEffects::default(),
            caster: ids[0],
            target: ids[1],
            other: ids[2],
        }
    }

    fn meta(&self) -> CommandMeta {
        CommandMeta {
            id: CommandId::new(),
            campaign_id: self.campaign.campaign_id(),
            session_id: None,
            issuer: CommandIssuer::System,
            actor: None,
            expected_event_sequence: self.campaign.applied_event_sequence,
        }
    }

    fn source(&self, meta: &CommandMeta, definition: &str) -> EffectSource {
        EffectSource {
            definition_id: definition.into(),
            actor: self.caster,
            command: meta.clone(),
            ordinal: 0,
        }
    }

    fn effect(&self, source: EffectSource, target: EntityId) -> TacticalEffect {
        TacticalEffect {
            id: EffectId::new(),
            source,
            established_at: None,
            target: TacticalEffectTarget::Creature(target),
            concentration_group: None,
            expires: TacticalEffectExpiry::Never,
            overlap: None,
            conditions: vec![EffectCondition {
                id: EffectId::new(),
                condition: Condition::Paralyzed,
            }],
            triggers: Vec::new(),
        }
    }

    fn run_with(
        &mut self,
        meta: &CommandMeta,
        operation: EffectLifecycleOperation,
    ) -> EffectLifecycleTransition {
        let action = EffectLifecycleAction { step: 0, operation };
        let before = self.effects.clone();
        let result = apply_effect_lifecycle(&self.campaign, &self.effects, meta, &action).unwrap();
        // Every scenario checks re-resolution and a serialized interrupted state, not just
        // equality with a hard-coded implementation result.
        let decoded: TacticalEffects =
            serde_json::from_str(&serde_json::to_string(&before).unwrap()).unwrap();
        assert_eq!(
            apply_effect_lifecycle(&self.campaign, &decoded, meta, &action).unwrap(),
            result
        );
        assert_eq!(
            self.effects, before,
            "pure resolution must not mutate its input"
        );
        self.effects = result.next_effects.clone();
        self.campaign.applied_event_sequence += 1;
        self.effects.validate(&self.campaign).unwrap();
        result
    }

    fn run(&mut self, operation: EffectLifecycleOperation) -> EffectLifecycleTransition {
        self.run_with(&self.meta(), operation)
    }

    fn observe(&mut self, event: EffectObservation) -> EffectLifecycleTransition {
        self.run(EffectLifecycleOperation::Observe(event))
    }

    fn resolve(
        &mut self,
        id: EffectTicketId,
        outcome: EffectTriggerResolution,
    ) -> EffectLifecycleTransition {
        self.run(EffectLifecycleOperation::ResolveTrigger {
            ticket: id,
            outcome,
        })
    }

    fn turn(
        &mut self,
        actor: EntityId,
        number: u64,
        boundary: TurnBoundary,
    ) -> EffectLifecycleTransition {
        self.campaign.rules.as_mut().unwrap().timing = Some(CombatTiming {
            order: vec![InitiativeEntry {
                actor,
                total: 10,
                tie_break: 0,
            }],
            index: 0,
            round: 1,
            turn_number: number,
            action_spent: false,
            bonus_action_spent: false,
            slot_spent_this_turn: false,
            reactions_spent: Vec::new(),
        });
        self.observe(EffectObservation::Turn(EffectTurn {
            actor,
            number,
            boundary,
        }))
    }

    fn group(&mut self) -> ConcentrationGroup {
        let meta = self.meta();
        let group = ConcentrationGroup {
            id: EffectId::new(),
            source: self.source(&meta, "hold-person"),
            expires: TacticalEffectExpiry::AtTime(WorldInstant(60)),
            stage: ConcentrationStage::Casting,
        };
        self.run_with(
            &meta,
            EffectLifecycleOperation::BeginConcentration {
                group: group.clone(),
            },
        );
        group
    }
}

#[test]
fn concentration_target_escape_preserves_other_targets_and_replacement_ends_the_group() {
    let mut f = Fixture::new();
    let group = f.group();
    let mut first = f.effect(group.source.clone(), f.target);
    first.concentration_group = Some(group.id);
    first.triggers.push(EffectTriggerRule {
        event: EffectTriggerEvent::Turn {
            subject: EffectSubject::Target,
            boundary: TurnBoundary::End,
        },
        frequency: EffectTriggerFrequency::EveryOccurrence,
        payload: EffectTriggerPayload::SavingThrow {
            ability: Ability::Wisdom,
            dc: 15,
            on_success: EffectSaveEnd::TargetEffect,
            on_failure: EffectSaveEnd::None,
        },
    });
    let mut second = f.effect(group.source.clone(), f.other);
    second.concentration_group = Some(group.id);
    let (first_id, second_id) = (first.id, second.id);
    f.run(EffectLifecycleOperation::Install {
        effects: vec![first, second],
    });
    assert_eq!(
        f.effects.concentration_bindings(),
        vec![(f.caster, group.id)]
    );
    assert!(
        active_effect_views(&f.effects)
            .iter()
            .all(|e| e.concentration_owner.is_none())
    );
    f.turn(f.target, 1, TurnBoundary::Start);
    let observed = f.turn(f.target, 1, TurnBoundary::End);
    assert_eq!(
        f.effects.effects.len(),
        2,
        "observation does not decide a save"
    );
    let saved = f.resolve(
        observed.triggers[0].id,
        EffectTriggerResolution::SavingThrow { success: true },
    );
    assert!(saved.ended.iter().any(|e| e.id == first_id && !e.group));
    assert_eq!(f.effects.effects[0].id, second_id);
    assert!(f.effects.group_for_owner(f.caster).is_some());
    let next = f.group();
    assert_ne!(next.id, group.id);
    assert!(
        f.effects.effects.is_empty(),
        "starting the next cast ends the old spell immediately"
    );
    assert_eq!(f.effects.groups[0].stage, ConcentrationStage::Casting);
}

#[test]
fn committed_held_cast_replaces_only_its_hold_expiry_before_target_installation() {
    let mut f = Fixture::new();
    let meta = f.meta();
    let group = ConcentrationGroup {
        id: EffectId::new(),
        source: f.source(&meta, "hold-person"),
        expires: TacticalEffectExpiry::AfterOwnerBoundaries {
            owner: f.caster,
            boundary: TurnBoundary::Start,
            remaining: 1,
        },
        stage: ConcentrationStage::Casting,
    };
    f.run_with(
        &meta,
        EffectLifecycleOperation::BeginConcentration {
            group: group.clone(),
        },
    );
    f.campaign.clock.now = WorldInstant(12);
    f.run(EffectLifecycleOperation::SetCastingDuration {
        group: group.id,
        source: group.source.clone(),
        expires: TacticalEffectExpiry::AtTime(WorldInstant(72)),
    });
    assert_eq!(f.effects.groups[0].id, group.id);
    assert_eq!(f.effects.groups[0].source, group.source);
    assert_eq!(f.effects.groups[0].stage, ConcentrationStage::Casting);
    assert!(f.effects.effects.is_empty());
    let mut effect = f.effect(group.source.clone(), f.target);
    effect.concentration_group = Some(group.id);
    effect.expires = TacticalEffectExpiry::AtTime(WorldInstant(72));
    f.run(EffectLifecycleOperation::Install {
        effects: vec![effect],
    });
    assert_eq!(f.effects.groups[0].stage, ConcentrationStage::Active);
    let caster = f.caster;
    assert!(f.turn(caster, 3, TurnBoundary::Start).triggers.is_empty());
    assert!(f.effects.group_for_owner(caster).is_some());
    f.campaign.clock.now = WorldInstant(72);
    let due = f.observe(EffectObservation::Time);
    let expiry = due
        .triggers
        .iter()
        .find(|ticket| ticket.payload == EffectTriggerPayload::ExpireConcentrationGroup)
        .unwrap();
    f.resolve(expiry.id, EffectTriggerResolution::Apply);
    assert!(f.effects.groups.is_empty());
    assert!(f.effects.effects.is_empty());
    assert!(f.effects.pending.is_empty());
}

#[test]
fn casting_duration_cannot_refresh_active_or_due_groups_or_replace_source() {
    let mut f = Fixture::new();
    let group = f.group();
    let mut wrong_source = group.source.clone();
    wrong_source.actor = f.other;
    for (id, source, expires) in [
        (
            EffectId::new(),
            group.source.clone(),
            TacticalEffectExpiry::AtTime(WorldInstant(90)),
        ),
        (
            group.id,
            wrong_source,
            TacticalEffectExpiry::AtTime(WorldInstant(90)),
        ),
        (
            group.id,
            group.source.clone(),
            TacticalEffectExpiry::AtTime(WorldInstant(0)),
        ),
        (group.id, group.source.clone(), TacticalEffectExpiry::Never),
    ] {
        let before = f.effects.clone();
        assert!(
            apply_effect_lifecycle(
                &f.campaign,
                &f.effects,
                &f.meta(),
                &EffectLifecycleAction {
                    step: 0,
                    operation: EffectLifecycleOperation::SetCastingDuration {
                        group: id,
                        source,
                        expires
                    },
                }
            )
            .is_err()
        );
        assert_eq!(before, f.effects);
    }
    let mut active = f.effect(group.source.clone(), f.target);
    active.concentration_group = Some(group.id);
    f.run(EffectLifecycleOperation::Install {
        effects: vec![active],
    });
    let operation = EffectLifecycleOperation::SetCastingDuration {
        group: group.id,
        source: group.source.clone(),
        expires: TacticalEffectExpiry::AtTime(WorldInstant(90)),
    };
    let before = f.effects.clone();
    assert!(
        apply_effect_lifecycle(
            &f.campaign,
            &f.effects,
            &f.meta(),
            &EffectLifecycleAction { step: 0, operation }
        )
        .is_err()
    );
    assert_eq!(before, f.effects);
    let next = f.group();
    f.campaign.clock.now = WorldInstant(60);
    let expired = f.effects.clone();
    assert!(
        apply_effect_lifecycle(
            &f.campaign,
            &f.effects,
            &f.meta(),
            &EffectLifecycleAction {
                step: 0,
                operation: EffectLifecycleOperation::SetCastingDuration {
                    group: next.id,
                    source: next.source.clone(),
                    expires: TacticalEffectExpiry::AtTime(WorldInstant(90)),
                },
            }
        )
        .is_err()
    );
    assert_eq!(expired, f.effects);
    assert!(!f.observe(EffectObservation::Time).triggers.is_empty());
    let before = f.effects.clone();
    assert!(
        apply_effect_lifecycle(
            &f.campaign,
            &f.effects,
            &f.meta(),
            &EffectLifecycleAction {
                step: 0,
                operation: EffectLifecycleOperation::SetCastingDuration {
                    group: next.id,
                    source: next.source,
                    expires: TacticalEffectExpiry::AtTime(WorldInstant(90))
                },
            }
        )
        .is_err()
    );
    assert_eq!(before, f.effects);
}

#[test]
fn strongest_overlap_is_target_local_and_older_cast_returns_when_newer_ends() {
    let mut f = Fixture::new();
    let meta = f.meta();
    let mut weak = f.effect(f.source(&meta, "test-blessing"), f.target);
    weak.overlap = Some(EffectOverlap {
        key: "test-blessing".into(),
        potency: 1,
    });
    let weak_id = weak.id;
    let weak_view = weak.conditions[0].id;
    f.run_with(
        &meta,
        EffectLifecycleOperation::Install {
            effects: vec![weak],
        },
    );
    let meta = f.meta();
    let mut strong = f.effect(f.source(&meta, "test-blessing"), f.target);
    strong.overlap = Some(EffectOverlap {
        key: "test-blessing".into(),
        potency: 2,
    });
    let strong_id = strong.id;
    let other = f.effect(f.source(&meta, "unrelated"), f.other);
    f.run_with(
        &meta,
        EffectLifecycleOperation::Install {
            effects: vec![strong, other],
        },
    );
    assert_eq!(active_effect_views(&f.effects).len(), 2);
    assert!(
        !active_effect_views(&f.effects)
            .iter()
            .any(|e| e.id == weak_view)
    );
    f.run(EffectLifecycleOperation::EndEffect {
        effect: strong_id,
        reason: EffectEndReason::Dispelled,
    });
    assert!(
        active_effect_views(&f.effects)
            .iter()
            .any(|e| e.id == weak_view)
    );
    assert!(f.effects.effects.iter().any(|e| e.id == weak_id));
}

#[test]
fn simultaneous_expiry_and_trigger_wait_for_the_recorded_order() {
    let mut f = Fixture::new();
    let meta = f.meta();
    let mut older = f.effect(f.source(&meta, "ongoing-zone"), f.target);
    older.overlap = Some(EffectOverlap {
        key: "overlap".into(),
        potency: 1,
    });
    older.triggers.push(EffectTriggerRule {
        event: EffectTriggerEvent::Turn {
            subject: EffectSubject::Source,
            boundary: TurnBoundary::Start,
        },
        frequency: EffectTriggerFrequency::EveryOccurrence,
        payload: EffectTriggerPayload::Damage {
            dice: vec![DieSpec { count: 1, sides: 6 }],
            modifier: 0,
            damage_type: DamageType::Fire,
        },
    });
    f.run_with(
        &meta,
        EffectLifecycleOperation::Install {
            effects: vec![older],
        },
    );
    let meta = f.meta();
    let mut newer = f.effect(f.source(&meta, "ongoing-zone"), f.target);
    newer.overlap = Some(EffectOverlap {
        key: "overlap".into(),
        potency: 1,
    });
    newer.expires = TacticalEffectExpiry::AfterOwnerBoundaries {
        owner: f.caster,
        boundary: TurnBoundary::Start,
        remaining: 1,
    };
    f.run_with(
        &meta,
        EffectLifecycleOperation::Install {
            effects: vec![newer],
        },
    );
    let observed = f.turn(f.caster, 1, TurnBoundary::Start);
    assert_eq!(observed.triggers.len(), 2);
    assert!(observed.ended.is_empty());
    let expiry = observed
        .triggers
        .iter()
        .find(|t| t.payload == EffectTriggerPayload::ExpireTargetEffect)
        .unwrap()
        .id;
    let damage = observed
        .triggers
        .iter()
        .find(|t| matches!(t.payload, EffectTriggerPayload::Damage { .. }))
        .unwrap()
        .clone();
    assert!(!trigger_is_applicable(&f.effects, &damage).unwrap());
    let mut alternate = Fixture {
        campaign: f.campaign.clone(),
        effects: f.effects.clone(),
        caster: f.caster,
        target: f.target,
        other: f.other,
    };
    alternate.resolve(damage.id, EffectTriggerResolution::SkipInactive);
    alternate.resolve(expiry, EffectTriggerResolution::Apply);
    assert!(alternate.effects.pending.is_empty());
    f.resolve(expiry, EffectTriggerResolution::Apply);
    assert!(
        trigger_is_applicable(&f.effects, &damage).unwrap(),
        "older effect resumes before its selected consequence"
    );
    let applied = f.resolve(damage.id, EffectTriggerResolution::Apply);
    assert_eq!(applied.selected_trigger_applies, Some(true));
}

#[test]
fn owner_relative_expiry_ignores_other_creatures_boundaries() {
    let mut f = Fixture::new();
    let meta = f.meta();
    let mut effect = f.effect(f.source(&meta, "shield"), f.target);
    effect.expires = TacticalEffectExpiry::AfterOwnerBoundaries {
        owner: f.caster,
        boundary: TurnBoundary::Start,
        remaining: 1,
    };
    f.run_with(
        &meta,
        EffectLifecycleOperation::Install {
            effects: vec![effect],
        },
    );
    assert!(f.turn(f.other, 1, TurnBoundary::Start).triggers.is_empty());
    assert!(f.turn(f.other, 1, TurnBoundary::End).triggers.is_empty());
    let start = f.turn(f.caster, 2, TurnBoundary::Start);
    assert_eq!(
        f.effects.effects.len(),
        1,
        "duration awaits accepted simultaneous ordering"
    );
    f.resolve(start.triggers[0].id, EffectTriggerResolution::Apply);
    assert!(f.effects.effects.is_empty());
}

#[test]
fn positive_damage_ends_only_affected_target_and_zero_damage_does_not_wake_it() {
    let mut f = Fixture::new();
    let meta = f.meta();
    let mut asleep = f.effect(f.source(&meta, "hypnotic-pattern"), f.target);
    asleep.triggers.push(EffectTriggerRule {
        event: EffectTriggerEvent::Damage {
            subject: EffectSubject::Target,
        },
        frequency: EffectTriggerFrequency::EveryOccurrence,
        payload: EffectTriggerPayload::EndTargetEffect,
    });
    let other = f.effect(f.source(&meta, "hypnotic-pattern"), f.other);
    f.run_with(
        &meta,
        EffectLifecycleOperation::Install {
            effects: vec![asleep, other],
        },
    );
    assert!(
        f.observe(EffectObservation::Damage {
            source: Some(f.caster),
            target: f.target,
            amount: 0
        })
        .triggers
        .is_empty()
    );
    let damage = f.observe(EffectObservation::Damage {
        source: Some(f.caster),
        target: f.target,
        amount: 1,
    });
    assert_eq!(f.effects.effects.len(), 2);
    f.resolve(damage.triggers[0].id, EffectTriggerResolution::Apply);
    assert_eq!(f.effects.effects.len(), 1);
    assert_eq!(
        f.effects.effects[0].target,
        TacticalEffectTarget::Creature(f.other)
    );
}

#[test]
fn zone_entry_and_turn_trigger_share_a_limit_per_target_per_turn_not_per_round() {
    let mut f = Fixture::new();
    f.turn(f.caster, 1, TurnBoundary::Start);
    let meta = f.meta();
    let mut zone = f.effect(f.source(&meta, "spirit-guardians"), f.target);
    zone.target = TacticalEffectTarget::Zone {
        occupants: Vec::new(),
    };
    zone.conditions.clear();
    let payload = EffectTriggerPayload::Damage {
        dice: vec![DieSpec { count: 3, sides: 8 }],
        modifier: 0,
        damage_type: DamageType::Radiant,
    };
    for event in [
        EffectTriggerEvent::ZoneContact(ZoneContact::CreatureEnters),
        EffectTriggerEvent::ZoneContact(ZoneContact::ZoneEntersCreature),
        EffectTriggerEvent::Turn {
            subject: EffectSubject::Target,
            boundary: TurnBoundary::End,
        },
    ] {
        zone.triggers.push(EffectTriggerRule {
            event,
            frequency: EffectTriggerFrequency::OncePerTargetPerTurn {
                key: "damage".into(),
            },
            payload: payload.clone(),
        });
    }
    let id = zone.id;
    f.run_with(
        &meta,
        EffectLifecycleOperation::Install {
            effects: vec![zone],
        },
    );
    let enter = f.observe(EffectObservation::Zone {
        effect: id,
        target: f.target,
        contact: ZoneContact::CreatureEnters,
    });
    f.resolve(enter.triggers[0].id, EffectTriggerResolution::Apply);
    f.observe(EffectObservation::Zone {
        effect: id,
        target: f.target,
        contact: ZoneContact::CreatureLeaves,
    });
    let enter = f.observe(EffectObservation::Zone {
        effect: id,
        target: f.target,
        contact: ZoneContact::ZoneEntersCreature,
    });
    assert!(!trigger_is_applicable(&f.effects, &enter.triggers[0]).unwrap());
    f.resolve(enter.triggers[0].id, EffectTriggerResolution::SkipInactive);
    let other = f.observe(EffectObservation::Zone {
        effect: id,
        target: f.other,
        contact: ZoneContact::CreatureEnters,
    });
    assert!(trigger_is_applicable(&f.effects, &other.triggers[0]).unwrap());
    f.resolve(other.triggers[0].id, EffectTriggerResolution::Apply);
    f.turn(f.caster, 1, TurnBoundary::End);
    f.turn(f.target, 2, TurnBoundary::Start);
    let end = f.turn(f.target, 2, TurnBoundary::End);
    assert!(trigger_is_applicable(&f.effects, &end.triggers[0]).unwrap());
    f.resolve(end.triggers[0].id, EffectTriggerResolution::Apply);
    assert_eq!(f.effects.trigger_uses.len(), 1);
}

#[test]
fn time_expiry_removes_a_whole_concentration_group_after_resume() {
    let mut f = Fixture::new();
    let group = f.group();
    let mut effect = f.effect(group.source, f.target);
    effect.concentration_group = Some(group.id);
    let meta = f.meta();
    let mut zone = f.effect(f.source(&meta, "unrelated-zone"), f.other);
    zone.target = TacticalEffectTarget::Zone {
        occupants: Vec::new(),
    };
    zone.conditions.clear();
    let zone_id = zone.id;
    f.run_with(
        &meta,
        EffectLifecycleOperation::Install {
            effects: vec![effect, zone],
        },
    );
    f.campaign.clock.now = WorldInstant(60);
    // Wall-clock expiry can become due during another zone's geometry observation.
    let due = f.observe(EffectObservation::Zone {
        effect: zone_id,
        target: f.other,
        contact: ZoneContact::CreatureEnters,
    });
    f.effects = serde_json::from_str(&serde_json::to_string(&f.effects).unwrap()).unwrap();
    assert_eq!(f.effects.pending, due.triggers);
    f.resolve(due.triggers[0].id, EffectTriggerResolution::Apply);
    assert!(f.effects.groups.is_empty() && f.effects.pending.is_empty());
    assert_eq!(f.effects.effects.len(), 1);
    assert_eq!(f.effects.effects[0].id, zone_id);
}

#[test]
fn invalid_and_duplicate_operations_leave_input_unchanged() {
    let mut f = Fixture::new();
    let meta = f.meta();
    let effect = f.effect(f.source(&meta, "effect"), f.target);
    let action = EffectLifecycleAction {
        step: 0,
        operation: EffectLifecycleOperation::Install {
            effects: vec![effect.clone(), effect],
        },
    };
    let before = f.effects.clone();
    assert!(apply_effect_lifecycle(&f.campaign, &f.effects, &meta, &action).is_err());
    assert_eq!(f.effects, before);
    let op = EffectLifecycleOperation::Observe(EffectObservation::Time);
    f.run_with(&meta, op.clone());
    assert!(
        apply_effect_lifecycle(
            &f.campaign,
            &f.effects,
            &meta,
            &EffectLifecycleAction {
                step: 0,
                operation: op.clone()
            }
        )
        .is_err()
    );
    let mut foreign = f.meta();
    foreign.campaign_id = CampaignId::new();
    assert!(
        apply_effect_lifecycle(
            &f.campaign,
            &f.effects,
            &foreign,
            &EffectLifecycleAction {
                step: 0,
                operation: op
            }
        )
        .is_err()
    );
}

#[test]
fn restored_corruption_cannot_forge_trigger_payload_source_or_target() {
    let mut f = Fixture::new();
    let meta = f.meta();
    let mut effect = f.effect(f.source(&meta, "hold-person"), f.target);
    effect.triggers.push(EffectTriggerRule {
        event: EffectTriggerEvent::Turn {
            subject: EffectSubject::Target,
            boundary: TurnBoundary::End,
        },
        frequency: EffectTriggerFrequency::EveryOccurrence,
        payload: EffectTriggerPayload::SavingThrow {
            ability: Ability::Wisdom,
            dc: 15,
            on_success: EffectSaveEnd::TargetEffect,
            on_failure: EffectSaveEnd::None,
        },
    });
    f.run_with(
        &meta,
        EffectLifecycleOperation::Install {
            effects: vec![effect],
        },
    );
    f.turn(f.target, 1, TurnBoundary::Start);
    f.turn(f.target, 1, TurnBoundary::End);
    for corruption in 0..6 {
        let mut bad = f.effects.clone();
        match corruption {
            0 => bad.pending[0].target = f.other,
            1 => bad.pending[0].source.definition_id = "another-spell".into(),
            2 => bad.pending[0].payload = EffectTriggerPayload::EndConcentrationGroup,
            3 => bad.pending[0].origin.command.campaign_id = CampaignId::new(),
            4 => bad.pending[0].rule_index = Some(9),
            _ => bad.pending.push(bad.pending[0].clone()),
        }
        assert!(
            bad.validate(&f.campaign).is_err(),
            "corruption {corruption}"
        );
    }
}

#[test]
fn duplicate_boundaries_and_unknown_serialized_fields_fail_closed() {
    let mut f = Fixture::new();
    f.turn(f.caster, 1, TurnBoundary::Start);
    let before = f.effects.clone();
    let action = EffectLifecycleAction {
        step: 0,
        operation: EffectLifecycleOperation::Observe(EffectObservation::Turn(EffectTurn {
            actor: f.caster,
            number: 1,
            boundary: TurnBoundary::Start,
        })),
    };
    assert!(apply_effect_lifecycle(&f.campaign, &f.effects, &f.meta(), &action).is_err());
    assert_eq!(f.effects, before);
    let mut json = serde_json::to_value(&f.effects).unwrap();
    json["future_effect_action"] = serde_json::json!({"damage": 999});
    assert!(serde_json::from_value::<TacticalEffects>(json).is_err());
}

#[test]
fn identical_damage_occurrences_each_queue_their_own_source_relative_trigger() {
    let mut f = Fixture::new();
    let meta = f.meta();
    let mut effect = f.effect(f.source(&meta, "damage-response"), f.target);
    effect.triggers.push(EffectTriggerRule {
        event: EffectTriggerEvent::Damage {
            subject: EffectSubject::Source,
        },
        frequency: EffectTriggerFrequency::EveryOccurrence,
        payload: EffectTriggerPayload::Damage {
            dice: vec![DieSpec { count: 1, sides: 4 }],
            modifier: 0,
            damage_type: DamageType::Psychic,
        },
    });
    f.run_with(
        &meta,
        EffectLifecycleOperation::Install {
            effects: vec![effect],
        },
    );
    let event = EffectObservation::Damage {
        source: Some(f.other),
        target: f.caster,
        amount: 3,
    };
    let first = f.observe(event.clone());
    let second = f.observe(event);
    assert_eq!(
        f.effects.pending.len(),
        2,
        "two damage instances are two source triggers"
    );
    assert_ne!(first.triggers[0].id, second.triggers[0].id);
    assert_eq!(
        first.triggers[0].target, f.target,
        "source damage resolves on the bound target"
    );
    f.resolve(first.triggers[0].id, EffectTriggerResolution::Apply);
    f.resolve(second.triggers[0].id, EffectTriggerResolution::Apply);
}

#[test]
fn ending_combat_preserves_a_lasting_spell_and_allows_a_new_initiative_counter() {
    let mut f = Fixture::new();
    let group = f.group();
    let mut effect = f.effect(group.source, f.target);
    effect.concentration_group = Some(group.id);
    f.run(EffectLifecycleOperation::Install {
        effects: vec![effect],
    });
    f.turn(f.caster, 1, TurnBoundary::Start);
    f.turn(f.caster, 1, TurnBoundary::End);
    let action = EffectLifecycleAction {
        step: 0,
        operation: EffectLifecycleOperation::LeaveCombat,
    };
    assert!(apply_effect_lifecycle(&f.campaign, &f.effects, &f.meta(), &action).is_err());
    f.campaign.rules.as_mut().unwrap().timing = None;
    f.run(EffectLifecycleOperation::LeaveCombat);
    assert!(f.effects.turn.is_none());
    assert!(f.effects.group_for_owner(f.caster).is_some());
    assert_eq!(f.effects.effects.len(), 1);
    f.turn(f.other, 1, TurnBoundary::Start);
    assert_eq!(f.effects.turn.unwrap().actor, f.other);
}

#[test]
fn expiry_ticket_target_is_validated_and_repeated_observations_do_not_duplicate_expiry() {
    let mut f = Fixture::new();
    let meta = f.meta();
    let mut effect = f.effect(f.source(&meta, "timed-effect"), f.target);
    effect.expires = TacticalEffectExpiry::AtTime(WorldInstant(1));
    f.run_with(
        &meta,
        EffectLifecycleOperation::Install {
            effects: vec![effect],
        },
    );
    f.campaign.clock.now = WorldInstant(1);
    let first = f.observe(EffectObservation::Time);
    let mut corrupt = f.effects.clone();
    corrupt.pending[0].target = f.other;
    assert!(corrupt.validate(&f.campaign).is_err());
    let second = f.observe(EffectObservation::Damage {
        source: None,
        target: f.target,
        amount: 0,
    });
    assert!(second.triggers.is_empty());
    assert_eq!(f.effects.pending.len(), 1);
    f.resolve(first.triggers[0].id, EffectTriggerResolution::Apply);
}

#[test]
fn trigger_fanout_stops_at_capacity_without_changing_authoritative_input() {
    let mut f = Fixture::new();
    let mut occupants = vec![f.caster, f.target, f.other];
    while occupants.len() < 128 {
        let id = EntityId::new();
        f.campaign.entities.insert(
            id,
            WorldEntity {
                id,
                campaign_id: f.campaign.campaign_id(),
                display_name: "Participant".into(),
                kind: EntityKind::Creature,
                existence: EntityExistence::Present,
                location_id: None,
            },
        );
        f.campaign
            .rules
            .as_mut()
            .unwrap()
            .entities
            .insert(id, MechanicalEntity::basic(id));
        occupants.push(id);
    }
    let meta = f.meta();
    let mut zones = Vec::new();
    for _ in 0..(MAX_PENDING_EFFECT_TRIGGERS / (16 * occupants.len()) + 1) {
        let mut effect = f.effect(f.source(&meta, "source-aura"), f.target);
        effect.conditions.clear();
        effect.target = TacticalEffectTarget::Zone {
            occupants: occupants.clone(),
        };
        effect.triggers = vec![
            EffectTriggerRule {
                event: EffectTriggerEvent::Damage {
                    subject: EffectSubject::Source
                },
                frequency: EffectTriggerFrequency::EveryOccurrence,
                payload: EffectTriggerPayload::Damage {
                    dice: vec![DieSpec { count: 1, sides: 4 }],
                    modifier: 0,
                    damage_type: DamageType::Fire,
                },
            };
            16
        ];
        zones.push(effect);
    }
    f.run_with(&meta, EffectLifecycleOperation::Install { effects: zones });
    let before = f.effects.clone();
    let result = apply_effect_lifecycle(
        &f.campaign,
        &f.effects,
        &f.meta(),
        &EffectLifecycleAction {
            step: 0,
            operation: EffectLifecycleOperation::Observe(EffectObservation::Damage {
                source: Some(f.other),
                target: f.caster,
                amount: 1,
            }),
        },
    );
    assert!(result.unwrap_err().0.contains("capacity"));
    assert_eq!(f.effects, before);
}

#[test]
fn a_per_turn_trigger_without_initiative_rejects_before_queuing_an_unresolvable_ticket() {
    let mut f = Fixture::new();
    let meta = f.meta();
    let mut effect = f.effect(f.source(&meta, "limited-response"), f.target);
    effect.triggers.push(EffectTriggerRule {
        event: EffectTriggerEvent::Damage {
            subject: EffectSubject::Target,
        },
        frequency: EffectTriggerFrequency::OncePerTargetPerTurn {
            key: "response".into(),
        },
        payload: EffectTriggerPayload::EndTargetEffect,
    });
    f.run_with(
        &meta,
        EffectLifecycleOperation::Install {
            effects: vec![effect],
        },
    );
    let before = f.effects.clone();
    let operation = EffectLifecycleOperation::Observe(EffectObservation::Damage {
        source: None,
        target: f.target,
        amount: 1,
    });
    assert!(
        apply_effect_lifecycle(
            &f.campaign,
            &f.effects,
            &f.meta(),
            &EffectLifecycleAction { step: 0, operation }
        )
        .is_err()
    );
    assert_eq!(f.effects, before);
}

#[test]
fn equally_potent_delayed_effect_uses_installation_recency_and_retains_casting_origin() {
    let mut f = Fixture::new();
    let cast = f.meta();
    let held_source = f.source(&cast, "overlapping-spell");
    // A previously accepted cast exists before another effect is installed.
    f.run_with(
        &cast,
        EffectLifecycleOperation::Observe(EffectObservation::Time),
    );
    let newer_cast = f.meta();
    let mut immediate = f.effect(f.source(&newer_cast, "overlapping-spell"), f.target);
    immediate.overlap = Some(EffectOverlap {
        key: "same-spell".into(),
        potency: 1,
    });
    f.run_with(
        &newer_cast,
        EffectLifecycleOperation::Install {
            effects: vec![immediate],
        },
    );
    let mut released = f.effect(held_source, f.target);
    released.overlap = Some(EffectOverlap {
        key: "same-spell".into(),
        potency: 1,
    });
    let released_view = released.conditions[0].id;
    f.run(EffectLifecycleOperation::Install {
        effects: vec![released],
    });
    assert_eq!(active_effect_views(&f.effects)[0].id, released_view);
    assert_eq!(f.effects.effects[1].source.command, cast);
    let mut corrupt = f.effects.clone();
    corrupt.effects[1].established_at = None;
    assert!(corrupt.validate(&f.campaign).is_err());
}

impl Fixture {
    fn attached(&mut self, operation: EffectLifecycleOperation) -> Vec<EndedTacticalEffect> {
        let meta = match &operation {
            EffectLifecycleOperation::Install { effects } => effects
                .iter()
                .find(|effect| {
                    effect.source.command.expected_event_sequence
                        == self.campaign.applied_event_sequence
                })
                .map(|effect| effect.source.command.clone())
                .unwrap_or_else(|| self.meta()),
            _ => self.meta(),
        };
        let action = EffectLifecycleAction { step: 0, operation };
        let before = self.campaign.clone();
        let result =
            dmd_rules::tactical_effect_adapter::apply_effect_operation(&before, &meta, &action)
                .unwrap();
        let decoded = CampaignState::decode_json(&before.encode_json().unwrap()).unwrap();
        assert_eq!(
            result,
            dmd_rules::tactical_effect_adapter::apply_effect_operation(&decoded, &meta, &action)
                .unwrap()
        );
        assert_eq!(self.campaign, before);
        self.campaign = result.0;
        self.campaign.applied_event_sequence += 1;
        dmd_rules::tactical_effect_adapter::validate_effect_attachment(&self.campaign).unwrap();
        assert!(self.campaign.validate().is_empty());
        result.1
    }
}

#[test]
fn attached_unconsciousness_respects_prone_immunity_and_keeps_legacy_semantics() {
    for immune in [false, true] {
        let mut f = Fixture::new();
        if immune {
            f.campaign
                .rules
                .as_mut()
                .unwrap()
                .entities
                .get_mut(&f.target)
                .unwrap()
                .condition_immunities
                .insert(Condition::Prone);
        }
        let mut effect = f.effect(f.source(&f.meta(), "source-unconsciousness"), f.target);
        effect.conditions[0].condition = Condition::Unconscious;
        let id = effect.id;
        f.attached(EffectLifecycleOperation::Install {
            effects: vec![effect],
        });
        let pack =
            dmd_rules::RulesPack::from_json(include_str!("../../../content/srd-5.2.1/kernel.json"))
                .unwrap();
        dmd_rules::validate_state(&f.campaign, &pack).unwrap();
        let rules = f.campaign.rules.as_ref().unwrap();
        let conditions = dmd_rules::active_conditions(rules, f.target);
        assert!(conditions.contains(&Condition::Unconscious));
        assert!(conditions.contains(&Condition::Incapacitated));
        assert_eq!(conditions.contains(&Condition::Prone), !immune);
        assert_eq!(rules.entities[&f.target].prone, !immune);
        assert_eq!(
            dmd_rules::tactical_conditions::effective_speed(rules, f.target, 60).unwrap(),
            0
        );
        assert!(rules.effects.is_empty());
        f.attached(EffectLifecycleOperation::EndEffect {
            effect: id,
            reason: EffectEndReason::Dismissed,
        });
        dmd_rules::validate_state(&f.campaign, &pack).unwrap();
        let rules = f.campaign.rules.as_ref().unwrap();
        assert!(!dmd_rules::active_conditions(rules, f.target).contains(&Condition::Unconscious));
        assert_eq!(
            rules.entities[&f.target].prone, !immune,
            "ending the condition never stands a creature up"
        );
    }

    // Legacy event-v1 consequences are preserved even when an empty new attachment exists.
    let mut f = Fixture::new();
    let rules = f.campaign.rules.as_mut().unwrap();
    let target = rules.entities.get_mut(&f.target).unwrap();
    target.condition_immunities.insert(Condition::Prone);
    target.prone = true;
    rules.effects.push(ActiveEffect {
        id: EffectId::new(),
        source: f.caster,
        target: f.target,
        condition: Some(Condition::Unconscious),
        label: "Historical effect".into(),
        expires: Expiry::Never,
        concentration_owner: None,
    });
    let historical = dmd_rules::active_conditions(rules, f.target);
    assert!(historical.contains(&Condition::Prone));
    rules.tactical_effects = Some(TacticalEffects::default());
    assert_eq!(historical, dmd_rules::active_conditions(rules, f.target));
    rules.tactical_recovery = Some(Default::default());
    assert_eq!(historical, dmd_rules::active_conditions(rules, f.target));
    // Query-only legacy interpretation is independent of the persisted posture.
    // This malformed snapshot is not accepted as valid state; it distinguishes
    // the historical derivation from merely re-reading the prone boolean above.
    rules.entities.get_mut(&f.target).unwrap().prone = false;
    assert!(dmd_rules::active_conditions(rules, f.target).contains(&Condition::Prone));
}

#[test]
fn attachment_rejects_unconscious_immunity_and_missing_required_prone_atomically() {
    let mut f = Fixture::new();
    f.campaign
        .rules
        .as_mut()
        .unwrap()
        .entities
        .get_mut(&f.target)
        .unwrap()
        .condition_immunities
        .insert(Condition::Unconscious);
    let mut effect = f.effect(f.source(&f.meta(), "source-unconsciousness"), f.target);
    effect.conditions[0].condition = Condition::Unconscious;
    let before = f.campaign.clone();
    let action = EffectLifecycleAction {
        step: 0,
        operation: EffectLifecycleOperation::Install {
            effects: vec![effect.clone()],
        },
    };
    assert!(
        dmd_rules::tactical_effect_adapter::apply_effect_operation(
            &f.campaign,
            &effect.source.command,
            &action
        )
        .is_err()
    );
    assert_eq!(f.campaign, before);
    f.campaign
        .rules
        .as_mut()
        .unwrap()
        .entities
        .get_mut(&f.target)
        .unwrap()
        .condition_immunities
        .clear();
    f.attached(action.operation);
    f.campaign
        .rules
        .as_mut()
        .unwrap()
        .entities
        .get_mut(&f.target)
        .unwrap()
        .prone = false;
    assert!(dmd_rules::tactical_effect_adapter::validate_effect_attachment(&f.campaign).is_err());
    let pack =
        dmd_rules::RulesPack::from_json(include_str!("../../../content/srd-5.2.1/kernel.json"))
            .unwrap();
    assert!(dmd_rules::validate_state(&f.campaign, &pack).is_err());
}

#[test]
fn suppressed_unconsciousness_reappears_through_the_attachment_without_forcing_immune_prone() {
    let mut f = Fixture::new();
    f.campaign
        .rules
        .as_mut()
        .unwrap()
        .entities
        .get_mut(&f.target)
        .unwrap()
        .condition_immunities
        .insert(Condition::Prone);
    let mut strong = f.effect(f.source(&f.meta(), "overlapping-condition"), f.target);
    strong.conditions[0].condition = Condition::Poisoned;
    strong.overlap = Some(EffectOverlap {
        key: "same-source".into(),
        potency: 2,
    });
    let id = strong.id;
    f.attached(EffectLifecycleOperation::Install {
        effects: vec![strong],
    });
    let mut weak = f.effect(f.source(&f.meta(), "overlapping-condition"), f.target);
    weak.conditions[0].condition = Condition::Unconscious;
    weak.overlap = Some(EffectOverlap {
        key: "same-source".into(),
        potency: 1,
    });
    f.attached(EffectLifecycleOperation::Install {
        effects: vec![weak],
    });
    assert!(
        !dmd_rules::active_conditions(f.campaign.rules.as_ref().unwrap(), f.target)
            .contains(&Condition::Unconscious)
    );
    f.attached(EffectLifecycleOperation::EndEffect {
        effect: id,
        reason: EffectEndReason::Expired,
    });
    let rules = f.campaign.rules.as_ref().unwrap();
    assert!(dmd_rules::active_conditions(rules, f.target).contains(&Condition::Unconscious));
    assert!(!rules.entities[&f.target].prone);
    assert!(!dmd_rules::active_conditions(rules, f.target).contains(&Condition::Prone));
}

#[test]
fn suppressed_immune_condition_cannot_be_installed_and_strand_later_removal() {
    let mut f = Fixture::new();
    f.campaign
        .rules
        .as_mut()
        .unwrap()
        .entities
        .get_mut(&f.target)
        .unwrap()
        .condition_immunities
        .insert(Condition::Unconscious);
    let mut strong = f.effect(f.source(&f.meta(), "overlapping-condition"), f.target);
    strong.conditions[0].condition = Condition::Poisoned;
    strong.overlap = Some(EffectOverlap {
        key: "same-source".into(),
        potency: 2,
    });
    let strong_id = strong.id;
    f.attached(EffectLifecycleOperation::Install {
        effects: vec![strong],
    });
    let mut weak = f.effect(f.source(&f.meta(), "overlapping-condition"), f.target);
    weak.conditions[0].condition = Condition::Unconscious;
    weak.overlap = Some(EffectOverlap {
        key: "same-source".into(),
        potency: 1,
    });
    let before = f.campaign.clone();
    let meta = weak.source.command.clone();
    let operation = EffectLifecycleAction {
        step: 0,
        operation: EffectLifecycleOperation::Install {
            effects: vec![weak],
        },
    };
    assert!(
        dmd_rules::tactical_effect_adapter::apply_effect_operation(&before, &meta, &operation)
            .is_err()
    );
    assert_eq!(before, f.campaign);
    f.attached(EffectLifecycleOperation::EndEffect {
        effect: strong_id,
        reason: EffectEndReason::Expired,
    });
    let rules = f.campaign.rules.as_ref().unwrap();
    assert!(rules.tactical_effects.as_ref().unwrap().effects.is_empty());
    assert!(dmd_rules::active_conditions(rules, f.target).is_empty());
}

#[test]
fn attached_concentration_projects_sources_and_replaces_the_whole_group() {
    let mut f = Fixture::new();
    let group = ConcentrationGroup {
        id: EffectId::new(),
        source: f.source(&f.meta(), "charm-person"),
        expires: TacticalEffectExpiry::Never,
        stage: ConcentrationStage::Casting,
    };
    // Begin needs its source command to be exactly the accepted operation.
    let meta = group.source.command.clone();
    f.campaign = dmd_rules::tactical_effect_adapter::apply_effect_operation(
        &f.campaign,
        &meta,
        &EffectLifecycleAction {
            step: 0,
            operation: EffectLifecycleOperation::BeginConcentration {
                group: group.clone(),
            },
        },
    )
    .unwrap()
    .0;
    f.campaign.applied_event_sequence += 1;
    let mut charm = f.effect(group.source.clone(), f.target);
    charm.conditions[0].condition = Condition::Charmed;
    charm.concentration_group = Some(group.id);
    let mut fear = f.effect(group.source.clone(), f.other);
    fear.conditions[0].condition = Condition::Frightened;
    fear.concentration_group = Some(group.id);
    f.attached(EffectLifecycleOperation::Install {
        effects: vec![charm, fear],
    });
    let rules = f.campaign.rules.as_ref().unwrap();
    assert!(
        rules.effects.is_empty(),
        "no duplicated persistent condition views"
    );
    assert!(!dmd_rules::tactical_conditions::may_harm(
        rules, f.target, f.caster
    ));
    assert!(dmd_rules::active_conditions(rules, f.other).contains(&Condition::Frightened));
    assert_eq!(rules.entities[&f.caster].concentration, Some(group.id));
    let meta = f.meta();
    let replacement = ConcentrationGroup {
        id: EffectId::new(),
        source: f.source(&meta, "bless"),
        expires: TacticalEffectExpiry::Never,
        stage: ConcentrationStage::Casting,
    };
    let (next, ended) = dmd_rules::tactical_effect_adapter::apply_effect_operation(
        &f.campaign,
        &meta,
        &EffectLifecycleAction {
            step: 0,
            operation: EffectLifecycleOperation::BeginConcentration {
                group: replacement.clone(),
            },
        },
    )
    .unwrap();
    assert_eq!(ended.len(), 3);
    let rules = next.rules.as_ref().unwrap();
    assert!(dmd_rules::tactical_conditions::may_harm(
        rules, f.target, f.caster
    ));
    assert!(!dmd_rules::active_conditions(rules, f.other).contains(&Condition::Frightened));
    assert_eq!(
        rules.entities[&f.caster].concentration,
        Some(replacement.id)
    );
}

#[test]
fn attached_incapacitation_breaks_concentration_but_preserves_unrelated_conditions() {
    let mut f = Fixture::new();
    let meta = f.meta();
    let group = ConcentrationGroup {
        id: EffectId::new(),
        source: f.source(&meta, "bless"),
        expires: TacticalEffectExpiry::Never,
        stage: ConcentrationStage::Casting,
    };
    f.campaign = dmd_rules::tactical_effect_adapter::apply_effect_operation(
        &f.campaign,
        &meta,
        &EffectLifecycleAction {
            step: 0,
            operation: EffectLifecycleOperation::BeginConcentration {
                group: group.clone(),
            },
        },
    )
    .unwrap()
    .0;
    f.campaign.applied_event_sequence += 1;
    let mut blessed = f.effect(group.source.clone(), f.target);
    blessed.conditions.clear();
    blessed.concentration_group = Some(group.id);
    f.attached(EffectLifecycleOperation::Install {
        effects: vec![blessed],
    });
    let mut paralysis = f.effect(f.source(&f.meta(), "hold-person"), f.caster);
    paralysis.source.actor = f.other;
    let paralysis_id = paralysis.id;
    let ended = f.attached(EffectLifecycleOperation::Install {
        effects: vec![paralysis],
    });
    assert!(
        ended
            .iter()
            .any(|e| e.id == group.id && e.reason == EffectEndReason::ConcentrationBroken)
    );
    let rules = f.campaign.rules.as_ref().unwrap();
    assert_eq!(rules.entities[&f.caster].concentration, None);
    assert!(dmd_rules::active_conditions(rules, f.caster).contains(&Condition::Incapacitated));
    assert_eq!(rules.tactical_effects.as_ref().unwrap().effects.len(), 1);
    assert_eq!(
        rules.tactical_effects.as_ref().unwrap().effects[0].id,
        paralysis_id
    );
}

#[test]
fn attachment_rejects_cross_authority_identity_collision_without_mutation() {
    let mut f = Fixture::new();
    let mut effect = f.effect(f.source(&f.meta(), "test-condition"), f.target);
    effect.conditions[0].condition = Condition::Poisoned;
    f.campaign
        .rules
        .as_mut()
        .unwrap()
        .effects
        .push(ActiveEffect {
            id: effect.conditions[0].id,
            source: f.caster,
            target: f.other,
            condition: Some(Condition::Poisoned),
            label: "Existing".into(),
            expires: Expiry::Never,
            concentration_owner: None,
        });
    let before = f.campaign.clone();
    let meta = effect.source.command.clone();
    assert!(
        dmd_rules::tactical_effect_adapter::apply_effect_operation(
            &f.campaign,
            &meta,
            &EffectLifecycleAction {
                step: 0,
                operation: EffectLifecycleOperation::Install {
                    effects: vec![effect]
                }
            }
        )
        .is_err()
    );
    assert_eq!(f.campaign, before);
}
