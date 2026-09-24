use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use dmd_app::{CampaignRuntime, RulesContext, RulesReplayApplier, RunnableCampaignError};
use dmd_domain::*;
use dmd_persistence::{export_campaign, load_command_audit, open_sqlite, replay_campaign_to_head};
use dmd_rules::{RulesAction, RulesAnswer, RulesError, RulesOutcome, RulesPack, RulesQuery};

struct Fixture {
    directory: PathBuf,
    content: PathBuf,
    db: PathBuf,
    state: CampaignState,
    player: PlayerId,
    other_player: PlayerId,
    actor: EntityId,
    other_actor: EntityId,
}

impl Fixture {
    fn new() -> Self {
        let campaign_id = CampaignId::new();
        let directory = std::env::temp_dir().join(format!("dmd-rules-app-{}", campaign_id.0));
        let content = directory.join("content");
        fs::create_dir_all(&content).unwrap();
        let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../content/srd-5.2.1");
        for entry in fs::read_dir(source).unwrap() {
            let entry = entry.unwrap();
            if entry.file_type().unwrap().is_file() {
                fs::copy(entry.path(), content.join(entry.file_name())).unwrap();
            }
        }
        let mut state = CampaignState::empty(
            Campaign {
                id: campaign_id,
                display_name: "Unrelated rules campaign".into(),
                status: CampaignStatus::Active,
                world_seed: 73,
                ruleset: VersionedRef {
                    id: "srd-5.2".into(),
                    version: "5.2.1".into(),
                },
                content_packs: vec![],
            },
            WorldClock {
                now: WorldInstant(0),
                calendar_id: "seconds".into(),
            },
        );
        let player = PlayerId::new();
        let other_player = PlayerId::new();
        let actor = EntityId::new();
        let other_actor = EntityId::new();
        for (player_id, entity_id) in [(player, actor), (other_player, other_actor)] {
            state.players.insert(
                player_id,
                Player {
                    id: player_id,
                    campaign_id,
                    display_name: "Player".into(),
                },
            );
            state.entities.insert(
                entity_id,
                WorldEntity {
                    id: entity_id,
                    campaign_id,
                    display_name: "Same name".into(),
                    kind: EntityKind::Character,
                    existence: EntityExistence::Present,
                    location_id: None,
                },
            );
            let id = CharacterId::new();
            state.characters.insert(
                id,
                Character {
                    id,
                    entity_id,
                    campaign_id,
                    controlling_player_id: Some(player_id),
                    display_name: "Same name".into(),
                    status: CharacterStatus::Active,
                },
            );
        }
        Self {
            db: directory.join("campaign.sqlite"),
            directory,
            content,
            state,
            player,
            other_player,
            actor,
            other_actor,
        }
    }

    async fn runtime(&self) -> (sqlx::SqlitePool, CampaignRuntime) {
        let pool = open_sqlite(&format!("sqlite://{}", self.db.display()))
            .await
            .unwrap();
        let runtime = CampaignRuntime::from_content_root(pool.clone(), &self.content);
        (pool, runtime)
    }

    fn context(
        &self,
        issuer: CommandIssuer,
        actor: Option<EntityId>,
        sequence: u64,
    ) -> RulesContext {
        RulesContext {
            campaign_id: self.state.campaign_id(),
            issuer,
            actor,
            session_id: None,
            expected_event_sequence: sequence,
        }
    }

    async fn initialize(&self, runtime: &CampaignRuntime) {
        runtime.create_campaign(&self.state).await.unwrap();
        runtime
            .execute_rules(
                self.context(CommandIssuer::Admin, None, 0),
                RulesAction::Initialize {
                    entities: vec![mechanics(self.actor), mechanics(self.other_actor)],
                    house_rules: HouseRules::default(),
                    ruling: ruling(),
                },
            )
            .await
            .unwrap();
    }

    fn request(&self, id: RollRequestId, visibility: RollVisibility) -> RulesAction {
        RulesAction::RequestTest {
            actor: self.actor,
            kind: TestKind::Check {
                ability: Ability::Strength,
                skill: Some(Skill::Athletics),
            },
            dc: 15,
            visibility,
            circumstances: Circumstances::default(),
            ruling: ruling(),
            request_id: id,
        }
    }

    fn applier(&self) -> RulesReplayApplier {
        RulesReplayApplier::new(
            RulesPack::from_json(&fs::read_to_string(self.content.join("kernel.json")).unwrap())
                .unwrap(),
        )
    }

    async fn step(
        &self,
        runtime: &CampaignRuntime,
        issuer: CommandIssuer,
        actor: Option<EntityId>,
        action: RulesAction,
    ) -> dmd_app::RulesReceipt {
        let sequence = runtime
            .open_campaign(self.state.campaign_id())
            .await
            .unwrap()
            .state()
            .applied_event_sequence;
        runtime
            .execute_rules(self.context(issuer, actor, sequence), action)
            .await
            .unwrap()
    }

    async fn submit(
        &self,
        runtime: &CampaignRuntime,
        player: PlayerId,
        actor: EntityId,
        values: &[u16],
    ) -> dmd_app::RulesReceipt {
        let state = runtime
            .open_campaign(self.state.campaign_id())
            .await
            .unwrap();
        let request = &state
            .state()
            .rules
            .as_ref()
            .unwrap()
            .pending
            .as_ref()
            .unwrap()
            .request;
        let mut values = values.iter().copied();
        let dice = request
            .dice
            .iter()
            .flat_map(|spec| {
                let count = if request.mode == RollMode::Normal {
                    spec.count
                } else {
                    2
                };
                (0..count)
                    .map(|_| DieResult {
                        sides: spec.sides,
                        value: values.next().unwrap(),
                    })
                    .collect::<Vec<_>>()
            })
            .collect();
        assert!(values.next().is_none());
        self.step(
            runtime,
            CommandIssuer::Player(player),
            Some(actor),
            RulesAction::SubmitRoll {
                result: RollResult {
                    request_id: request.id,
                    source: RollSource::Physical,
                    dice,
                },
            },
        )
        .await
    }

    async fn assert_replay(&self, pool: &sqlx::SqlitePool, runtime: &CampaignRuntime) {
        let actual = runtime
            .open_campaign(self.state.campaign_id())
            .await
            .unwrap();
        assert_eq!(
            &replay_campaign_to_head(pool, self.state.campaign_id(), &self.applier())
                .await
                .unwrap(),
            actual.state()
        );
        assert_eq!(
            &runtime
                .replay_rules(self.state.campaign_id())
                .await
                .unwrap(),
            actual.state()
        );
        let backup = export_campaign(pool, self.state.campaign_id())
            .await
            .unwrap();
        let target = open_sqlite("sqlite::memory:").await.unwrap();
        let restored = CampaignRuntime::from_content_root(target.clone(), &self.content);
        assert_eq!(
            restored.restore_campaign(&backup).await.unwrap().state(),
            actual.state()
        );
        assert_eq!(
            &restored
                .replay_rules(self.state.campaign_id())
                .await
                .unwrap(),
            actual.state()
        );
        drop(restored);
        target.close().await;
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.directory);
    }
}

fn ruling() -> Ruling {
    Ruling {
        basis: RulingBasis::Srd { page: 7 },
        reason: "Meaningful uncertain attempt against established evidence".into(),
    }
}

fn mechanics(entity_id: EntityId) -> MechanicalEntity {
    let mut entity = MechanicalEntity::basic(entity_id);
    entity.ability_scores = [16, 14, 14, 10, 12, 10];
    entity.max_hp = 12;
    entity.hp = 12;
    entity.saving_proficiencies = BTreeSet::from([Ability::Strength]);
    entity.skill_proficiencies = BTreeMap::from([(Skill::Athletics, Proficiency::Proficient)]);
    entity.resources = BTreeMap::from([(
        "resolve".into(),
        ResourcePool {
            maximum: 2,
            remaining: 2,
            recovery: Recovery::ShortOrLongRest,
        },
    )]);
    entity.attacks.insert("club".into());
    entity.attack_proficiencies.insert("club".into());
    entity.prepared_spells = BTreeSet::from(["cure-wounds".into(), "dancing-lights".into()]);
    entity.spellcasting = Some(Spellcasting {
        ability: Ability::Wisdom,
        slot_maxima: [2, 0, 0, 0, 0, 0, 0, 0, 0],
        slots: [2, 0, 0, 0, 0, 0, 0, 0, 0],
        can_speak: true,
        free_hand: true,
        material_focus: true,
    });
    entity
}

#[tokio::test]
async fn initiative_attack_damage_reaction_and_effect_timing_use_the_durable_path() {
    let f = Fixture::new();
    let (pool, runtime) = f.runtime().await;
    f.initialize(&runtime).await;
    for (actor, player, face) in [(f.actor, f.player, 15), (f.other_actor, f.other_player, 10)] {
        f.step(
            &runtime,
            CommandIssuer::System,
            Some(actor),
            RulesAction::RequestTest {
                actor,
                kind: TestKind::Initiative,
                dc: 0,
                visibility: RollVisibility::Public,
                circumstances: Circumstances::default(),
                ruling: ruling(),
                request_id: RollRequestId::new(),
            },
        )
        .await;
        f.submit(&runtime, player, actor, &[face]).await;
    }
    f.step(
        &runtime,
        CommandIssuer::Admin,
        None,
        RulesAction::StartCombat {
            participants: vec![
                InitiativeEntry {
                    actor: f.other_actor,
                    total: 12,
                    tie_break: 1,
                },
                InitiativeEntry {
                    actor: f.actor,
                    total: 17,
                    tie_break: 0,
                },
            ],
            ruling: ruling(),
        },
    )
    .await;
    f.step(
        &runtime,
        CommandIssuer::System,
        None,
        RulesAction::AuthorizeAttack {
            actor: f.actor,
            target: f.other_actor,
            attack_id: "club".into(),
            circumstances: Circumstances::default(),
            within_five_feet: true,
            ruling: ruling(),
        },
    )
    .await;
    f.step(
        &runtime,
        CommandIssuer::Player(f.player),
        Some(f.actor),
        RulesAction::Attack {
            actor: f.actor,
            target: f.other_actor,
            attack_id: "club".into(),
            request_id: RollRequestId::new(),
        },
    )
    .await;
    let attack = f.submit(&runtime, f.player, f.actor, &[20]).await;
    assert!(matches!(
        attack.outcome,
        RulesOutcome::RollResolved {
            critical: true,
            followup: Some(_),
            ..
        }
    ));
    // Save at the damage decision boundary, close the database, then supply physical faces.
    let pending = runtime
        .open_campaign(f.state.campaign_id())
        .await
        .unwrap()
        .state()
        .clone();
    drop(runtime);
    pool.close().await;
    let (pool, runtime) = f.runtime().await;
    assert_eq!(
        runtime
            .resume_campaign(f.state.campaign_id())
            .await
            .unwrap()
            .state(),
        &pending
    );
    let damage = f.submit(&runtime, f.player, f.actor, &[3, 2]).await;
    assert!(matches!(
        damage.outcome,
        RulesOutcome::RollResolved {
            amount: Some(8),
            ..
        }
    ));
    let bonus = RulesAction::UseBonusAction {
        actor: f.actor,
        feature_id: "validated-feature".into(),
        ruling: ruling(),
    };
    f.step(
        &runtime,
        CommandIssuer::System,
        Some(f.actor),
        bonus.clone(),
    )
    .await;
    let sequence = runtime
        .open_campaign(f.state.campaign_id())
        .await
        .unwrap()
        .state()
        .applied_event_sequence;
    assert!(
        runtime
            .execute_rules(
                f.context(CommandIssuer::System, Some(f.actor), sequence),
                bonus
            )
            .await
            .is_err()
    );
    let effect_id = EffectId::new();
    f.step(
        &runtime,
        CommandIssuer::System,
        None,
        RulesAction::ApplyEffect {
            effect: ActiveEffect {
                id: effect_id,
                source: f.actor,
                target: f.other_actor,
                condition: Some(Condition::Poisoned),
                label: "Temporary poison".into(),
                expires: Expiry::AtTurn {
                    actor: f.actor,
                    boundary: TurnBoundary::End,
                    turn_number: 1,
                },
                concentration_owner: None,
            },
            ruling: ruling(),
        },
    )
    .await;
    f.step(
        &runtime,
        CommandIssuer::System,
        Some(f.other_actor),
        RulesAction::UseReaction {
            actor: f.other_actor,
            trigger: "Validated reaction opportunity".into(),
            ruling: ruling(),
        },
    )
    .await;
    let sequence = runtime
        .open_campaign(f.state.campaign_id())
        .await
        .unwrap()
        .state()
        .applied_event_sequence;
    assert!(
        runtime
            .execute_rules(
                f.context(CommandIssuer::System, Some(f.other_actor), sequence),
                RulesAction::UseReaction {
                    actor: f.other_actor,
                    trigger: "Second opportunity".into(),
                    ruling: ruling()
                }
            )
            .await
            .is_err()
    );
    f.step(
        &runtime,
        CommandIssuer::Player(f.player),
        Some(f.actor),
        RulesAction::EndTurn { actor: f.actor },
    )
    .await;
    let state = runtime.open_campaign(f.state.campaign_id()).await.unwrap();
    let rules = state.state().rules.as_ref().unwrap();
    assert_eq!(rules.entities[&f.other_actor].hp, 4);
    assert!(!rules.effects.iter().any(|e| e.id == effect_id));
    assert!(rules.timing.as_ref().unwrap().reactions_spent.is_empty());
    assert!(!rules.timing.as_ref().unwrap().bonus_action_spent);
    f.step(
        &runtime,
        CommandIssuer::Player(f.other_player),
        Some(f.other_actor),
        RulesAction::EndTurn {
            actor: f.other_actor,
        },
    )
    .await;
    assert_eq!(
        runtime
            .open_campaign(f.state.campaign_id())
            .await
            .unwrap()
            .state()
            .clock
            .now,
        WorldInstant(6)
    );
    f.step(
        &runtime,
        CommandIssuer::Admin,
        None,
        RulesAction::EndCombat { ruling: ruling() },
    )
    .await;
    f.assert_replay(&pool, &runtime).await;
    drop(runtime);
    pool.close().await;
}

#[tokio::test]
async fn spells_concentration_damage_healing_and_rests_survive_reopen_and_replay() {
    let f = Fixture::new();
    let (pool, runtime) = f.runtime().await;
    f.initialize(&runtime).await;
    f.step(
        &runtime,
        CommandIssuer::System,
        None,
        RulesAction::AuthorizeSpell {
            actor: f.actor,
            target: f.actor,
            spell_id: "dancing-lights".into(),
            circumstances: Circumstances::default(),
            within_five_feet: true,
            ruling: ruling(),
        },
    )
    .await;
    f.step(
        &runtime,
        CommandIssuer::Player(f.player),
        Some(f.actor),
        RulesAction::CastSpell {
            actor: f.actor,
            target: f.actor,
            spell_id: "dancing-lights".into(),
            slot_level: 0,
            request_id: RollRequestId::new(),
            effect_id: EffectId::new(),
        },
    )
    .await;
    let damage = f
        .step(
            &runtime,
            CommandIssuer::System,
            None,
            RulesAction::ApplyDamage {
                target: f.actor,
                amount: 4,
                damage_type: DamageType::Fire,
                critical: false,
                ruling: ruling(),
            },
        )
        .await;
    assert!(matches!(
        damage.outcome,
        RulesOutcome::Damage {
            followup: Some(_),
            ..
        }
    ));
    let checkpoint = runtime
        .open_campaign(f.state.campaign_id())
        .await
        .unwrap()
        .state()
        .clone();
    drop(runtime);
    pool.close().await;
    let (pool, runtime) = f.runtime().await;
    assert_eq!(
        runtime
            .resume_campaign(f.state.campaign_id())
            .await
            .unwrap()
            .state(),
        &checkpoint
    );
    f.submit(&runtime, f.player, f.actor, &[1]).await;
    assert!(
        runtime
            .open_campaign(f.state.campaign_id())
            .await
            .unwrap()
            .state()
            .rules
            .as_ref()
            .unwrap()
            .entities[&f.actor]
            .concentration
            .is_none()
    );
    f.step(
        &runtime,
        CommandIssuer::System,
        None,
        RulesAction::AuthorizeSpell {
            actor: f.actor,
            target: f.actor,
            spell_id: "cure-wounds".into(),
            circumstances: Circumstances::default(),
            within_five_feet: true,
            ruling: ruling(),
        },
    )
    .await;
    f.step(
        &runtime,
        CommandIssuer::Player(f.player),
        Some(f.actor),
        RulesAction::CastSpell {
            actor: f.actor,
            target: f.actor,
            spell_id: "cure-wounds".into(),
            slot_level: 1,
            request_id: RollRequestId::new(),
            effect_id: EffectId::new(),
        },
    )
    .await;
    f.submit(&runtime, f.player, f.actor, &[2, 3]).await;
    f.step(
        &runtime,
        CommandIssuer::System,
        None,
        RulesAction::ApplyDamage {
            target: f.actor,
            amount: 6,
            damage_type: DamageType::Fire,
            critical: false,
            ruling: ruling(),
        },
    )
    .await;
    f.step(
        &runtime,
        CommandIssuer::Player(f.player),
        Some(f.actor),
        RulesAction::SpendResource {
            actor: f.actor,
            resource_id: "resolve".into(),
            amount: 2,
        },
    )
    .await;
    f.step(
        &runtime,
        CommandIssuer::System,
        None,
        RulesAction::StartRest {
            actor: f.actor,
            kind: RestKind::Short,
            ruling: ruling(),
        },
    )
    .await;
    f.step(
        &runtime,
        CommandIssuer::System,
        None,
        RulesAction::AdvanceTime {
            seconds: 3600,
            ruling: ruling(),
        },
    )
    .await;
    f.step(
        &runtime,
        CommandIssuer::System,
        None,
        RulesAction::FinishRest {
            actor: f.actor,
            slept_seconds: 0,
            ruling: ruling(),
        },
    )
    .await;
    f.step(
        &runtime,
        CommandIssuer::Player(f.player),
        Some(f.actor),
        RulesAction::SpendHitDie {
            actor: f.actor,
            request_id: RollRequestId::new(),
        },
    )
    .await;
    f.submit(&runtime, f.player, f.actor, &[3]).await;
    let state = runtime.open_campaign(f.state.campaign_id()).await.unwrap();
    let actor = &state.state().rules.as_ref().unwrap().entities[&f.actor];
    assert_eq!(actor.hp, 11);
    assert_eq!(actor.hit_dice.remaining, 0);
    assert_eq!(actor.resources["resolve"].remaining, 2);
    assert_eq!(actor.spellcasting.as_ref().unwrap().slots[0], 1);
    f.step(
        &runtime,
        CommandIssuer::System,
        None,
        RulesAction::StartRest {
            actor: f.actor,
            kind: RestKind::Long,
            ruling: ruling(),
        },
    )
    .await;
    f.step(
        &runtime,
        CommandIssuer::System,
        None,
        RulesAction::AdvanceTime {
            seconds: 28800,
            ruling: ruling(),
        },
    )
    .await;
    f.step(
        &runtime,
        CommandIssuer::System,
        None,
        RulesAction::FinishRest {
            actor: f.actor,
            slept_seconds: 21600,
            ruling: ruling(),
        },
    )
    .await;
    let state = runtime.open_campaign(f.state.campaign_id()).await.unwrap();
    let actor = &state.state().rules.as_ref().unwrap().entities[&f.actor];
    assert_eq!(actor.hp, 12);
    assert_eq!(actor.hit_dice.remaining, 1);
    assert_eq!(actor.spellcasting.as_ref().unwrap().slots[0], 2);
    f.assert_replay(&pool, &runtime).await;
    drop(runtime);
    pool.close().await;
}

#[tokio::test]
async fn supported_manifest_is_not_sufficient_for_an_unsupported_kernel_version() {
    let mut f = Fixture::new();
    let manifest_path = f.content.join("manifest.json");
    let mut manifest: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&manifest_path).unwrap()).unwrap();
    manifest["version"] = serde_json::json!("5.2.2");
    fs::write(manifest_path, serde_json::to_vec_pretty(&manifest).unwrap()).unwrap();
    f.state.campaign.ruleset.version = "5.2.2".into();
    let (pool, runtime) = f.runtime().await;
    assert!(matches!(
        runtime.create_campaign(&f.state).await,
        Err(RunnableCampaignError::RulesContent(_))
    ));
    assert!(
        dmd_persistence::open_campaign(&pool, f.state.campaign_id())
            .await
            .is_err()
    );
    drop(runtime);
    pool.close().await;
}

#[tokio::test]
async fn rehashed_or_undeclared_tactical_catalog_cannot_authorize_campaign_mutation() {
    for undeclared in [false, true] {
        let f = Fixture::new();
        let (pool, runtime) = f.runtime().await;
        f.initialize(&runtime).await;
        let before = export_campaign(&pool, f.state.campaign_id()).await.unwrap();
        let manifest_path = f.content.join("manifest.json");
        let mut manifest: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&manifest_path).unwrap()).unwrap();
        let files = manifest["files"].as_array_mut().unwrap();
        if undeclared {
            files.retain(|file| file["path"] != "tactical.json");
        } else {
            // Even equivalent JSON with a correctly recomputed checksum is not the
            // exact catalog whose executable definitions this build implements.
            let path = f.content.join("tactical.json");
            let mut bytes = fs::read(&path).unwrap();
            bytes.push(b' ');
            fs::write(path, &bytes).unwrap();
            let tactical = files
                .iter_mut()
                .find(|file| file["path"] == "tactical.json")
                .unwrap();
            tactical["byte_len"] = serde_json::json!(bytes.len());
            tactical["checksum"]["value"] = serde_json::json!(fnv1a64_hex(&bytes));
        }
        fs::write(manifest_path, serde_json::to_vec_pretty(&manifest).unwrap()).unwrap();
        assert!(matches!(
            runtime
                .execute_rules(
                    f.context(CommandIssuer::System, None, 1),
                    RulesAction::AdvanceTime {
                        seconds: 1,
                        ruling: ruling()
                    },
                )
                .await,
            Err(RunnableCampaignError::RulesContent(_))
        ));
        let mut after = export_campaign(&pool, f.state.campaign_id()).await.unwrap();
        after.exported_at_utc = before.exported_at_utc.clone();
        assert_eq!(after, before);
        drop(runtime);
        pool.close().await;
    }
}

#[tokio::test]
async fn invalid_mechanical_initialization_and_stale_competing_action_leave_no_partial_writes() {
    let f = Fixture::new();
    let (pool, runtime) = f.runtime().await;
    let unrelated = Fixture::new();
    unrelated.initialize(&runtime).await;
    let unrelated_before = export_campaign(&pool, unrelated.state.campaign_id())
        .await
        .unwrap();
    runtime.create_campaign(&f.state).await.unwrap();
    let before = export_campaign(&pool, f.state.campaign_id()).await.unwrap();
    let mut invalid = mechanics(f.actor);
    invalid.hp = invalid.max_hp + 1;
    assert!(
        runtime
            .execute_rules(
                f.context(CommandIssuer::Admin, None, 0),
                RulesAction::Initialize {
                    entities: vec![invalid],
                    house_rules: HouseRules::default(),
                    ruling: ruling()
                }
            )
            .await
            .is_err()
    );
    assert_eq!(
        export_campaign(&pool, f.state.campaign_id())
            .await
            .unwrap()
            .current_state,
        before.current_state
    );
    runtime
        .execute_rules(
            f.context(CommandIssuer::Admin, None, 0),
            RulesAction::Initialize {
                entities: vec![mechanics(f.actor)],
                house_rules: HouseRules::default(),
                ruling: ruling(),
            },
        )
        .await
        .unwrap();
    let context = f.context(CommandIssuer::Player(f.player), Some(f.actor), 1);
    runtime
        .execute_rules(
            context.clone(),
            RulesAction::SpendResource {
                actor: f.actor,
                resource_id: "resolve".into(),
                amount: 1,
            },
        )
        .await
        .unwrap();
    assert!(matches!(
        runtime
            .execute_rules(
                context,
                RulesAction::SpendResource {
                    actor: f.actor,
                    resource_id: "resolve".into(),
                    amount: 1
                }
            )
            .await,
        Err(RunnableCampaignError::Rules(RulesError::Stale))
    ));
    let state = runtime.open_campaign(f.state.campaign_id()).await.unwrap();
    assert_eq!(
        state.state().rules.as_ref().unwrap().entities[&f.actor].resources["resolve"].remaining,
        1
    );
    assert_eq!(state.state().applied_event_sequence, 2);
    let unrelated_after = export_campaign(&pool, unrelated.state.campaign_id())
        .await
        .unwrap();
    assert_eq!(
        unrelated_after.current_state,
        unrelated_before.current_state
    );
    assert_eq!(
        unrelated_after.event_journal,
        unrelated_before.event_journal
    );
    assert_eq!(
        unrelated_after.command_audit,
        unrelated_before.command_audit
    );
    drop(runtime);
    pool.close().await;
}

#[tokio::test]
async fn inspiration_advantage_passive_queries_and_house_rulings_are_durable() {
    let f = Fixture::new();
    let (pool, runtime) = f.runtime().await;
    runtime.create_campaign(&f.state).await.unwrap();
    f.step(
        &runtime,
        CommandIssuer::Admin,
        None,
        RulesAction::Initialize {
            entities: vec![mechanics(f.actor), mechanics(f.other_actor)],
            house_rules: HouseRules {
                ability_test_natural_extremes: true,
            },
            ruling: ruling(),
        },
    )
    .await;
    f.step(
        &runtime,
        CommandIssuer::System,
        None,
        RulesAction::GrantInspiration {
            actor: f.actor,
            ruling: ruling(),
        },
    )
    .await;
    f.step(
        &runtime,
        CommandIssuer::System,
        None,
        RulesAction::ApplyEffect {
            effect: ActiveEffect {
                id: EffectId::new(),
                source: f.other_actor,
                target: f.actor,
                condition: Some(Condition::Poisoned),
                label: "Poison".into(),
                expires: Expiry::AtTime(WorldInstant(60)),
                concentration_owner: None,
            },
            ruling: ruling(),
        },
    )
    .await;
    let before = export_campaign(&pool, f.state.campaign_id()).await.unwrap();
    for (circumstances, expected) in [
        (Circumstances::default(), 6),
        (
            Circumstances {
                advantage: true,
                disadvantage: false,
                ranged_threat: false,
            },
            11,
        ),
    ] {
        assert_eq!(
            runtime
                .query_rules(
                    f.state.campaign_id(),
                    CommandIssuer::Player(f.player),
                    RulesQuery::PassivePerception {
                        actor: f.actor,
                        circumstances
                    }
                )
                .await
                .unwrap(),
            RulesAnswer::PassivePerception(expected)
        );
    }
    assert_eq!(
        export_campaign(&pool, f.state.campaign_id())
            .await
            .unwrap()
            .event_journal,
        before.event_journal
    );
    let id = RollRequestId::new();
    f.step(
        &runtime,
        CommandIssuer::System,
        Some(f.actor),
        f.request(id, RollVisibility::Public),
    )
    .await;
    let receipt = f
        .step(
            &runtime,
            CommandIssuer::Player(f.player),
            Some(f.actor),
            RulesAction::SubmitRollWithInspiration {
                result: RollResult {
                    request_id: id,
                    source: RollSource::Physical,
                    dice: vec![
                        DieResult {
                            sides: 20,
                            value: 1,
                        },
                        DieResult {
                            sides: 20,
                            value: 17,
                        },
                    ],
                },
                die_index: 0,
                replacement: DieResult {
                    sides: 20,
                    value: 20,
                },
            },
        )
        .await;
    assert!(
        matches!(receipt.outcome, RulesOutcome::RollResolved { ref roll, success: Some(true), .. }
        if roll.total == 22)
    );
    assert!(
        !runtime
            .open_campaign(f.state.campaign_id())
            .await
            .unwrap()
            .state()
            .rules
            .as_ref()
            .unwrap()
            .entities[&f.actor]
            .heroic_inspiration
    );
    f.step(
        &runtime,
        CommandIssuer::System,
        None,
        RulesAction::AdvanceTime {
            seconds: 60,
            ruling: ruling(),
        },
    )
    .await;
    f.step(
        &runtime,
        CommandIssuer::System,
        Some(f.actor),
        RulesAction::RequestTest {
            actor: f.actor,
            kind: TestKind::Check {
                ability: Ability::Strength,
                skill: Some(Skill::Athletics),
            },
            dc: 100,
            visibility: RollVisibility::Public,
            circumstances: Circumstances::default(),
            ruling: Ruling {
                basis: RulingBasis::HouseRule {
                    id: "ability-test-natural-extremes".into(),
                },
                reason: "Explicit campaign option permits a natural twenty to succeed".into(),
            },
            request_id: RollRequestId::new(),
        },
    )
    .await;
    assert!(matches!(
        f.submit(&runtime, f.player, f.actor, &[20]).await.outcome,
        RulesOutcome::RollResolved {
            success: Some(true),
            ..
        }
    ));
    let state = runtime.open_campaign(f.state.campaign_id()).await.unwrap();
    assert!(
        state
            .state()
            .rules
            .as_ref()
            .unwrap()
            .rulings
            .iter()
            .any(|record| matches!(record.ruling.basis, RulingBasis::HouseRule { .. }))
    );
    f.assert_replay(&pool, &runtime).await;
    drop(runtime);
    pool.close().await;
}

fn physical(id: RollRequestId, value: u16) -> RulesAction {
    RulesAction::SubmitRoll {
        result: RollResult {
            request_id: id,
            source: RollSource::Physical,
            dice: vec![DieResult { sides: 20, value }],
        },
    }
}

#[tokio::test]
async fn rules_restore_rejects_semantic_corruption_before_installing_any_rows() {
    let f = Fixture::new();
    let (pool, runtime) = f.runtime().await;
    f.initialize(&runtime).await;
    let initialized = runtime
        .open_campaign(f.state.campaign_id())
        .await
        .unwrap()
        .state()
        .clone();
    let id = RollRequestId::new();
    f.step(
        &runtime,
        CommandIssuer::System,
        Some(f.actor),
        f.request(id, RollVisibility::Public),
    )
    .await;
    let original = export_campaign(&pool, f.state.campaign_id()).await.unwrap();
    for mutation in 0..11 {
        let mut export = original.clone();
        match mutation {
            0 => {
                let mut state =
                    CampaignState::decode_json(&export.current_state.state_json).unwrap();
                state
                    .rules
                    .as_mut()
                    .unwrap()
                    .pending
                    .as_mut()
                    .unwrap()
                    .request
                    .modifier = 999;
                export.current_state.state_json = state.encode_json().unwrap();
            }
            1 => {
                let mut state = initialized.clone();
                state
                    .rules
                    .as_mut()
                    .unwrap()
                    .entities
                    .get_mut(&f.actor)
                    .unwrap()
                    .hp -= 1;
                export.snapshots.push(dmd_persistence::SnapshotRow {
                    campaign_id: export.campaign_id.clone(),
                    event_sequence: 1,
                    state_schema_version: i64::from(CURRENT_STATE_SCHEMA_VERSION),
                    state_json: state.encode_json().unwrap(),
                    created_at_utc: export.exported_at_utc.clone(),
                });
            }
            2 => export.command_audit[0].issuer_kind = "system".into(),
            3 => {
                let mut event: dmd_rules::RulesEvent =
                    serde_json::from_str(&export.event_journal[0].payload_json).unwrap();
                event.outcome = RulesOutcome::Healed { regained: 1 };
                export.event_journal[0].payload_json = serde_json::to_string(&event).unwrap();
            }
            4 => {
                let mut state =
                    CampaignState::decode_json(&export.current_state.state_json).unwrap();
                state
                    .rules
                    .as_mut()
                    .unwrap()
                    .entities
                    .get_mut(&f.actor)
                    .unwrap()
                    .hp -= 1;
                export.current_state.state_json = state.encode_json().unwrap();
            }
            5 => export.event_journal[0].event_kind = "rules.future_action".into(),
            6 => export.event_journal[0].event_schema_version = 2,
            7 => {
                let mut state =
                    CampaignState::decode_json(&export.current_state.state_json).unwrap();
                state.rules.as_mut().unwrap().rulings[0].command.id = CommandId::new();
                export.current_state.state_json = state.encode_json().unwrap();
            }
            8..=10 => {
                // Re-labeling a rules history as a generic campaign must not bypass preflight.
                let mut state =
                    CampaignState::decode_json(&export.current_state.state_json).unwrap();
                state.rules = None;
                state.campaign.ruleset.id = "generic-test".into();
                export.current_state.state_json = state.encode_json().unwrap();
                let path = f.content.join("manifest.json");
                let mut manifest: serde_json::Value =
                    serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
                manifest["id"] = serde_json::json!("generic-test");
                fs::write(path, serde_json::to_vec(&manifest).unwrap()).unwrap();
                if mutation >= 9 {
                    for event in &mut export.event_journal {
                        event.event_kind = "generic.changed".into();
                    }
                }
                if mutation == 10 {
                    for audit in &mut export.command_audit {
                        audit.command_kind = "generic.action".into();
                    }
                }
            }
            _ => unreachable!(),
        }
        // The outer persistence format is valid; only the rules owner can detect these defects.
        assert!(export.upgraded().is_ok(), "structural fixture {mutation}");
        let target = open_sqlite("sqlite::memory:").await.unwrap();
        let restored = CampaignRuntime::from_content_root(target.clone(), &f.content);
        assert!(
            restored.restore_campaign(&export).await.is_err(),
            "mutation {mutation}"
        );
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM campaign_state_current")
            .fetch_one(&target)
            .await
            .unwrap();
        assert_eq!(count, 0);
        drop(restored);
        target.close().await;
    }
    assert_eq!(
        export_campaign(&pool, f.state.campaign_id())
            .await
            .unwrap()
            .current_state,
        original.current_state
    );
    drop(runtime);
    pool.close().await;
}

#[tokio::test]
async fn persistence_session_rejection_preserves_resolved_rules_state_and_history() {
    let f = Fixture::new();
    let (pool, runtime) = f.runtime().await;
    f.initialize(&runtime).await;
    let before = export_campaign(&pool, f.state.campaign_id()).await.unwrap();
    let missing_session = PlaySessionId::new();
    let mut context = f.context(CommandIssuer::Player(f.player), Some(f.actor), 1);
    context.session_id = Some(missing_session);
    let rejected = runtime
        .execute_rules(
            context,
            RulesAction::SpendResource {
                actor: f.actor,
                resource_id: "resolve".into(),
                amount: 1,
            },
        )
        .await;
    assert!(matches!(rejected,
        Err(RunnableCampaignError::Journal(error))
        if matches!(*error, dmd_persistence::JournalStoreError::MissingSession(id) if id == missing_session)
    ));
    let after = export_campaign(&pool, f.state.campaign_id()).await.unwrap();
    assert_eq!(after.current_state, before.current_state);
    assert_eq!(after.command_audit, before.command_audit);
    assert_eq!(after.event_journal, before.event_journal);
    assert_eq!(after.snapshots, before.snapshots);
    drop(runtime);
    pool.close().await;
}

#[tokio::test]
async fn create_and_restore_reuse_preflight_content_after_waiting_for_database() {
    use std::{
        future::{Future, poll_fn},
        task::Poll,
    };

    for restore in [false, true] {
        let f = Fixture::new();
        let (source_pool, source_runtime) = f.runtime().await;
        f.initialize(&source_runtime).await;
        let backup = export_campaign(&source_pool, f.state.campaign_id())
            .await
            .unwrap();
        let pool = open_sqlite("sqlite::memory:").await.unwrap();
        let runtime = CampaignRuntime::from_content_root(pool.clone(), &f.content);
        let mut connections = Vec::new();
        for _ in 0..pool.options().get_max_connections() {
            connections.push(pool.acquire().await.unwrap());
        }
        let mut operation = Box::pin(async {
            if restore {
                runtime.restore_campaign(&backup).await
            } else {
                runtime.create_campaign(&f.state).await
            }
        });
        // Poll exactly through synchronous content preflight and block on the exhausted pool.
        // This gives a deterministic content change during the persistence await, without sleeps.
        poll_fn(|cx| {
            assert!(operation.as_mut().poll(cx).is_pending());
            Poll::Ready(())
        })
        .await;
        fs::write(f.content.join("kernel.json"), b"{}").unwrap();
        drop(connections);
        let completed = operation
            .await
            .expect("validated operation must report its commit");
        assert_eq!(completed.state().campaign_id(), f.state.campaign_id());
        let persisted = dmd_persistence::open_campaign(&pool, f.state.campaign_id())
            .await
            .unwrap();
        assert_eq!(completed.state(), &persisted.state);
        // Reusing preflight bytes for this response grants no capability to a later operation.
        assert!(runtime.open_campaign(f.state.campaign_id()).await.is_err());
        drop((runtime, source_runtime));
        pool.close().await;
        source_pool.close().await;
    }
}

#[tokio::test]
async fn legacy_rules_campaign_restore_upgrades_before_mechanical_preflight() {
    let f = Fixture::new();
    let (pool, runtime) = f.runtime().await;
    runtime.create_campaign(&f.state).await.unwrap();
    let mut export = export_campaign(&pool, f.state.campaign_id()).await.unwrap();
    fn legacy(json: &str) -> String {
        let mut value: serde_json::Value = serde_json::from_str(json).unwrap();
        value["schema_version"] = serde_json::json!(1);
        value.as_object_mut().unwrap().remove("rules");
        value.as_object_mut().unwrap().remove("table");
        serde_json::to_string(&value).unwrap()
    }
    export.state_schema_version = 1;
    export.format_version = 1;
    export.lifecycle.state_schema_version = 1;
    export.current_state.schema_version = 1;
    export.current_state.state_json = legacy(&export.current_state.state_json);
    for snapshot in &mut export.snapshots {
        snapshot.state_schema_version = 1;
        snapshot.state_json = legacy(&snapshot.state_json);
    }
    let target = open_sqlite("sqlite::memory:").await.unwrap();
    let restored = CampaignRuntime::from_content_root(target.clone(), &f.content);
    let runnable = restored.restore_campaign(&export).await.unwrap();
    assert_eq!(
        runnable.state().schema_version,
        CURRENT_STATE_SCHEMA_VERSION
    );
    assert!(runnable.state().rules.is_none());
    assert!(runnable.state().table.is_none());
    f.step(
        &restored,
        CommandIssuer::Admin,
        None,
        RulesAction::Initialize {
            entities: vec![mechanics(f.actor)],
            house_rules: HouseRules::default(),
            ruling: ruling(),
        },
    )
    .await;
    assert_eq!(
        export_campaign(&target, f.state.campaign_id())
            .await
            .unwrap()
            .snapshots,
        export.snapshots
    );
    f.assert_replay(&target, &restored).await;
    drop((runtime, restored));
    pool.close().await;
    target.close().await;
}

#[tokio::test]
async fn pending_physical_roll_survives_shutdown_and_replay_matches_committed_mechanics() {
    let f = Fixture::new();
    let (pool, runtime) = f.runtime().await;
    f.initialize(&runtime).await;
    let id = RollRequestId::new();
    let requested = runtime
        .execute_rules(
            f.context(CommandIssuer::System, Some(f.actor), 1),
            f.request(id, RollVisibility::Public),
        )
        .await
        .unwrap();
    assert!(
        matches!(requested.outcome, RulesOutcome::RollRequested(ref request)
        if request.id == id && request.modifier == 5)
    );
    let before = runtime
        .open_campaign(f.state.campaign_id())
        .await
        .unwrap()
        .state()
        .clone();
    drop(runtime);
    pool.close().await;
    let (pool, runtime) = f.runtime().await;
    assert_eq!(
        runtime
            .resume_campaign(f.state.campaign_id())
            .await
            .unwrap()
            .state(),
        &before
    );
    let result = runtime
        .execute_rules(
            f.context(CommandIssuer::Player(f.player), Some(f.actor), 2),
            physical(id, 13),
        )
        .await
        .unwrap();
    assert!(
        matches!(result.outcome, RulesOutcome::RollResolved { ref roll, success: Some(true), .. }
        if roll.total == 18 && roll.source == RollSource::Physical)
    );
    let audit = load_command_audit(&pool, result.commit.command_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(audit.meta.issuer, CommandIssuer::Player(f.player));
    assert_eq!(audit.meta.actor, Some(AgentRef::Entity(f.actor)));
    let after = runtime
        .open_campaign(f.state.campaign_id())
        .await
        .unwrap()
        .state()
        .clone();
    let replayed = replay_campaign_to_head(&pool, f.state.campaign_id(), &f.applier())
        .await
        .unwrap();
    assert_eq!(replayed, after);
    assert!(after.rules.as_ref().unwrap().pending.is_none());
    drop(runtime);
    pool.close().await;
}

#[tokio::test]
async fn rejected_rolls_and_questions_leave_durable_state_and_history_unchanged() {
    let f = Fixture::new();
    let (pool, runtime) = f.runtime().await;
    f.initialize(&runtime).await;
    let id = RollRequestId::new();
    runtime
        .execute_rules(
            f.context(CommandIssuer::System, Some(f.actor), 1),
            f.request(id, RollVisibility::Private),
        )
        .await
        .unwrap();
    let baseline = export_campaign(&pool, f.state.campaign_id()).await.unwrap();
    let cases = [
        (
            f.context(CommandIssuer::Player(f.other_player), Some(f.actor), 2),
            physical(id, 13),
        ),
        (
            f.context(CommandIssuer::Player(f.player), Some(f.actor), 1),
            physical(id, 13),
        ),
        (
            f.context(CommandIssuer::Player(f.player), Some(f.actor), 2),
            physical(id, 21),
        ),
        (
            f.context(CommandIssuer::Player(f.player), Some(f.actor), 2),
            physical(RollRequestId::new(), 13),
        ),
        (
            f.context(CommandIssuer::Player(f.player), Some(f.actor), 2),
            RulesAction::Heal {
                target: f.actor,
                amount: 100,
                ruling: ruling(),
            },
        ),
    ];
    for (context, action) in cases {
        assert!(runtime.execute_rules(context, action).await.is_err());
        let now = export_campaign(&pool, f.state.campaign_id()).await.unwrap();
        assert_eq!(now.current_state, baseline.current_state);
        assert_eq!(now.command_audit, baseline.command_audit);
        assert_eq!(now.event_journal, baseline.event_journal);
    }
    let answer = runtime
        .query_rules(
            f.state.campaign_id(),
            CommandIssuer::Player(f.player),
            RulesQuery::Character { actor: f.actor },
        )
        .await
        .unwrap();
    assert!(matches!(
        answer,
        RulesAnswer::Character {
            armor_class: 12,
            proficiency_bonus: 2,
            ..
        }
    ));
    assert!(
        runtime
            .query_rules(
                f.state.campaign_id(),
                CommandIssuer::Player(f.other_player),
                RulesQuery::Character { actor: f.actor }
            )
            .await
            .is_err()
    );
    let now = export_campaign(&pool, f.state.campaign_id()).await.unwrap();
    assert_eq!(now.current_state, baseline.current_state);
    assert_eq!(now.event_journal, baseline.event_journal);
    runtime
        .execute_rules(
            f.context(CommandIssuer::Player(f.player), Some(f.actor), 2),
            physical(id, 13),
        )
        .await
        .unwrap();
    let accepted = export_campaign(&pool, f.state.campaign_id()).await.unwrap();
    assert!(
        runtime
            .execute_rules(
                f.context(CommandIssuer::Player(f.player), Some(f.actor), 3),
                physical(id, 13)
            )
            .await
            .is_err()
    );
    assert_eq!(
        export_campaign(&pool, f.state.campaign_id())
            .await
            .unwrap()
            .event_journal,
        accepted.event_journal
    );
    drop(runtime);
    pool.close().await;
}

#[tokio::test]
async fn secret_digital_rolls_do_not_leak_and_replay_never_rerolls() {
    let f = Fixture::new();
    let (pool, runtime) = f.runtime().await;
    f.initialize(&runtime).await;
    let id = RollRequestId::new();
    runtime
        .execute_rules(
            f.context(CommandIssuer::System, Some(f.actor), 1),
            f.request(id, RollVisibility::Secret),
        )
        .await
        .unwrap();
    assert_eq!(
        runtime
            .query_rules(
                f.state.campaign_id(),
                CommandIssuer::Player(f.player),
                RulesQuery::PendingRoll
            )
            .await
            .unwrap(),
        RulesAnswer::PendingRoll(None)
    );
    assert!(
        runtime
            .execute_rules(
                f.context(CommandIssuer::Player(f.player), Some(f.actor), 2),
                physical(id, 20)
            )
            .await
            .is_err()
    );
    assert!(
        runtime
            .roll_digitally(
                f.context(CommandIssuer::Player(f.player), Some(f.actor), 2),
                id
            )
            .await
            .is_err()
    );
    let before = export_campaign(&pool, f.state.campaign_id()).await.unwrap();
    assert!(
        runtime
            .roll_digitally(f.context(CommandIssuer::System, Some(f.other_actor), 2), id)
            .await
            .is_err()
    );
    assert_eq!(
        export_campaign(&pool, f.state.campaign_id())
            .await
            .unwrap()
            .event_journal,
        before.event_journal
    );
    runtime
        .roll_digitally(f.context(CommandIssuer::System, Some(f.actor), 2), id)
        .await
        .unwrap();
    let state = runtime
        .open_campaign(f.state.campaign_id())
        .await
        .unwrap()
        .state()
        .clone();
    assert_eq!(
        state
            .rules
            .as_ref()
            .unwrap()
            .rolls
            .last()
            .unwrap()
            .result
            .source,
        RollSource::Digital
    );
    for _ in 0..3 {
        assert_eq!(
            replay_campaign_to_head(&pool, f.state.campaign_id(), &f.applier())
                .await
                .unwrap(),
            state
        );
    }
    drop(runtime);
    pool.close().await;
}

#[tokio::test]
async fn dead_character_remains_readable_to_its_controller_and_cannot_act() {
    let f = Fixture::new();
    let (pool, runtime) = f.runtime().await;
    f.initialize(&runtime).await;
    f.step(
        &runtime,
        CommandIssuer::System,
        None,
        RulesAction::ApplyDamage {
            target: f.actor,
            amount: 30,
            damage_type: DamageType::Fire,
            critical: false,
            ruling: ruling(),
        },
    )
    .await;
    assert!(matches!(
        runtime
            .query_rules(
                f.state.campaign_id(),
                CommandIssuer::Player(f.player),
                RulesQuery::Character { actor: f.actor }
            )
            .await
            .unwrap(),
        RulesAnswer::Character { hp: 0, .. }
    ));
    let before = export_campaign(&pool, f.state.campaign_id()).await.unwrap();
    assert!(
        runtime
            .execute_rules(
                f.context(CommandIssuer::Player(f.player), Some(f.actor), 2),
                RulesAction::SpendResource {
                    actor: f.actor,
                    resource_id: "resolve".into(),
                    amount: 1
                }
            )
            .await
            .is_err()
    );
    assert_eq!(
        export_campaign(&pool, f.state.campaign_id())
            .await
            .unwrap()
            .event_journal,
        before.event_journal
    );
    f.assert_replay(&pool, &runtime).await;
    drop(runtime);
    pool.close().await;
}

#[tokio::test]
async fn replay_adapter_rejects_tampered_envelopes_and_outcomes_without_changing_state() {
    use dmd_persistence::{ReplayEventApplier, load_journal_events};
    let f = Fixture::new();
    let (pool, runtime) = f.runtime().await;
    f.initialize(&runtime).await;
    let event = load_journal_events(&pool, f.state.campaign_id(), 0)
        .await
        .unwrap()
        .remove(0);
    for mutation in 0..9 {
        let mut bad = event.clone();
        match mutation {
            0 => bad.payload.kind = "unknown".into(),
            1 => bad.payload.schema_version += 1,
            2 => bad.meta.command_id = Some(CommandId::new()),
            3 => bad.meta.actor = Some(AgentRef::Entity(f.other_actor)),
            4 => bad.meta.session_id = Some(PlaySessionId::new()),
            5 => bad.meta.sequence += 1,
            6 => bad.meta.occurred_at = WorldInstant(1),
            7 => bad.meta.source = EventSource::Import,
            8 => {
                let mut payload: dmd_rules::RulesEvent =
                    serde_json::from_str(&bad.payload.json).unwrap();
                payload.outcome = RulesOutcome::Healed { regained: 9 };
                bad.payload.json = serde_json::to_string(&payload).unwrap();
            }
            _ => unreachable!(),
        }
        let mut state = f.state.clone();
        assert!(
            f.applier().apply(&mut state, &bad).is_err(),
            "mutation {mutation}"
        );
        assert_eq!(state, f.state);
    }
    f.assert_replay(&pool, &runtime).await;
    drop(runtime);
    pool.close().await;
}

#[tokio::test]
async fn pending_rules_export_restores_and_content_changes_fail_before_mutation() {
    let f = Fixture::new();
    let (pool, runtime) = f.runtime().await;
    f.initialize(&runtime).await;
    let id = RollRequestId::new();
    runtime
        .execute_rules(
            f.context(CommandIssuer::System, Some(f.actor), 1),
            f.request(id, RollVisibility::Public),
        )
        .await
        .unwrap();
    let backup = export_campaign(&pool, f.state.campaign_id()).await.unwrap();
    let target = open_sqlite(&format!(
        "sqlite://{}",
        f.directory.join("restore.sqlite").display()
    ))
    .await
    .unwrap();
    let restored_runtime = CampaignRuntime::from_content_root(target.clone(), &f.content);
    let restored = restored_runtime.restore_campaign(&backup).await.unwrap();
    assert_eq!(
        restored
            .state()
            .rules
            .as_ref()
            .unwrap()
            .pending
            .as_ref()
            .unwrap()
            .request
            .id,
        id
    );
    restored_runtime
        .execute_rules(
            f.context(CommandIssuer::Player(f.player), Some(f.actor), 2),
            physical(id, 15),
        )
        .await
        .unwrap();
    // Restored campaign is in another database; resolving there cannot consume the source request.
    assert_eq!(
        export_campaign(&pool, f.state.campaign_id())
            .await
            .unwrap()
            .current_state,
        backup.current_state
    );
    fs::write(f.content.join("kernel.json"), b"{}").unwrap();
    assert!(
        runtime
            .execute_rules(
                f.context(CommandIssuer::Player(f.player), Some(f.actor), 2),
                physical(id, 13)
            )
            .await
            .is_err()
    );
    assert!(
        runtime
            .query_rules(
                f.state.campaign_id(),
                CommandIssuer::Admin,
                RulesQuery::PendingRoll
            )
            .await
            .is_err()
    );
    assert_eq!(
        export_campaign(&pool, f.state.campaign_id())
            .await
            .unwrap()
            .current_state,
        backup.current_state
    );
    let empty = open_sqlite("sqlite::memory:").await.unwrap();
    let unavailable = CampaignRuntime::from_content_root(empty.clone(), &f.content);
    assert!(unavailable.restore_campaign(&backup).await.is_err());
    assert!(
        dmd_persistence::open_campaign(&empty, f.state.campaign_id())
            .await
            .is_err()
    );
    drop((runtime, restored_runtime, unavailable));
    pool.close().await;
    target.close().await;
    empty.close().await;
}
