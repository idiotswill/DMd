//! A real source critical hit and owned physical death saves produce this corpse.
//! Its owner is required for unresolved aftermath, then unnecessary after release.
use super::*;

struct DeathHistory {
    goblin: EntityId,
    before_release: Box<CampaignState>,
    receipts: Vec<(TableTransportRequest, TableTransportResult)>,
}

async fn accept(
    f: &mut Fixture,
    url: &str,
    directory: &Path,
    label: &str,
    player: Option<usize>,
    declaration: TableAction,
) -> (TableTransportRequest, TableTransportResult) {
    let request = request(f, player, declaration).await;
    let response = Box::pin(durable_step(f, url, directory, label, request.clone())).await;
    (request, response)
}

fn survivor_session(f: &Fixture) -> TableAction {
    TableAction::StartSession {
        id: f.session,
        name: "The survivors return without the dead character's owner".into(),
        participants: vec![SessionParticipant {
            player_id: f.players[0],
            character_id: Some(f.characters[0]),
            attendance: AttendanceStatus::Present,
        }],
    }
}

fn assert_corpse_unchanged(f: &Fixture, current: &CampaignState, original: &CampaignState) {
    let corpse = f.actors[1];
    assert_eq!(
        current.rules.as_ref().unwrap().entities[&corpse],
        original.rules.as_ref().unwrap().entities[&corpse]
    );
    assert_eq!(current.entities[&corpse], original.entities[&corpse]);
    assert_eq!(
        current.characters[&f.characters[1]],
        original.characters[&f.characters[1]]
    );
    assert_eq!(
        current.characters[&f.characters[1]].status,
        CharacterStatus::Dead
    );
    assert_eq!(current.entities[&corpse].existence, EntityExistence::Dead);
    assert_eq!(current.rules.as_ref().unwrap().entities[&corpse].hp, 0);
    let corpse_items = original
        .items
        .values()
        .filter(|item| {
            item.owner == Ownership::Entity(corpse) || item.custody == Custody::Entity(corpse)
        })
        .collect::<Vec<_>>();
    assert!(
        !corpse_items.is_empty(),
        "real character equipment must be present"
    );
    for item in corpse_items {
        assert_eq!(current.items[&item.id], *item);
    }
    let old_scene = original.encounter.as_ref().unwrap().scene_id;
    assert_eq!(
        current.scenes[&old_scene].presences,
        original.scenes[&old_scene].presences
    );
    assert!(
        current
            .rules
            .as_ref()
            .unwrap()
            .rolls
            .starts_with(&original.rules.as_ref().unwrap().rolls)
    );
}

async fn dying_closed_refusal(f: &mut Fixture, url: &str, directory: &Path) {
    Box::pin(table_medicine_cases::prepare(f, false)).await;
    // This existing helper creates both PCs and a source Goblin, rolls the source
    // critical 2d6+2 = 12, and chooses ordinary damage against the real 12-HP PC.
    let dying = Box::new(state(f).await);
    let patient = &dying.rules.as_ref().unwrap().entities[&f.actors[1]];
    assert_eq!(patient.hp, 0);
    assert!(!patient.death.dead && !patient.death.stable);
    assert_eq!(patient.death.failures, 0);
    assert_eq!(
        dying
            .rules
            .as_ref()
            .unwrap()
            .timing
            .as_ref()
            .unwrap()
            .turn_number,
        4
    );
    Box::pin(accept(
        f,
        url,
        directory,
        "dying-conclusion",
        None,
        conclusion(),
    ))
    .await;
    let live_finish = request(f, None, action(TacticalAction::FinishEncounter)).await;
    let error = Box::pin(rejected_request(f, live_finish)).await;
    assert!(
        error.contains("settle every dying creature on the retained cadence"),
        "{error}"
    );
    Box::pin(accept(
        f,
        url,
        directory,
        "closed-dying-aftermath",
        None,
        TableAction::EndSession,
    ))
    .await;
    assert!(view(f, None).await.active_session.is_none());
    let closed_finish = closed_request(f, action(TacticalAction::FinishEncounter)).await;
    let error = Box::pin(rejected_request(f, closed_finish)).await;
    assert!(
        error.contains("settle every dying creature on the retained cadence"),
        "{error}"
    );
    // The failed administrative release must leave the actual old cadence resumable.
    f.session = PlaySessionId::new();
    let absent = request(f, None, survivor_session(f)).await;
    let error = Box::pin(rejected_request(f, absent)).await;
    assert!(
        error.contains("every retained character's controller"),
        "{error}"
    );
    let start = TableAction::StartSession {
        id: f.session,
        name: "Both owners resume the unresolved death saves".into(),
        participants: participants(f),
    };
    Box::pin(accept(
        f,
        url,
        directory,
        "dying-cadence-resumed",
        None,
        start,
    ))
    .await;
    assert_eq!(state(f).await.clock, dying.clock);
}

async fn actual_death(f: &mut Fixture, url: &str, directory: &Path) -> DeathHistory {
    Box::pin(accept(
        f,
        url,
        directory,
        "first-owned-death-save",
        Some(0),
        action(TacticalAction::EndTurn),
    ))
    .await;
    let pending = view(f, Some(1)).await.roll.unwrap();
    assert_eq!(pending.reason, "Death saving throw");
    assert_eq!(
        pending.dice,
        vec![DieSpec {
            count: 1,
            sides: 20
        }]
    );
    Box::pin(rejected(f, None, TableAction::EndSession)).await;
    Box::pin(rejected(f, None, action(TacticalAction::FinishEncounter))).await;
    let first = raw_action(f, Some(1), &[1]).await;
    Box::pin(rejected(f, Some(0), first.clone())).await;
    let first = Box::pin(accept(
        f,
        url,
        directory,
        "physical-death-one",
        Some(1),
        first,
    ))
    .await;
    let failed = Box::new(state(f).await);
    assert_eq!(
        failed.rules.as_ref().unwrap().entities[&f.actors[1]]
            .death
            .failures,
        2
    );
    assert!(
        !failed.rules.as_ref().unwrap().entities[&f.actors[1]]
            .death
            .dead
    );
    assert!(failed.rules.as_ref().unwrap().pending.is_none());
    let premature = request(f, None, action(TacticalAction::FinishEncounter)).await;
    let error = Box::pin(rejected_request(f, premature)).await;
    assert!(error.contains("settle every dying creature"), "{error}");
    Box::pin(direct(f, Some(1), TacticalAction::EndTurn)).await;
    Box::pin(direct(f, None, TacticalAction::EndTurn)).await;
    Box::pin(accept(
        f,
        url,
        directory,
        "second-owned-death-save",
        Some(0),
        action(TacticalAction::EndTurn),
    ))
    .await;
    assert_eq!(
        view(f, Some(1)).await.roll.unwrap().reason,
        "Death saving throw"
    );
    let second = raw_action(f, Some(1), &[2]).await;
    let second = Box::pin(accept(
        f,
        url,
        directory,
        "physical-death-two",
        Some(1),
        second,
    ))
    .await;
    let dead = Box::new(state(f).await);
    let patient = &dead.rules.as_ref().unwrap().entities[&f.actors[1]];
    assert_eq!(patient.hp, 0);
    assert_eq!(
        (patient.death.successes, patient.death.failures),
        (0, 0),
        "the third failure kills the creature and clears its pending-save counters"
    );
    assert!(patient.death.dead);
    assert_eq!(dead.entities[&f.actors[1]].existence, EntityExistence::Dead);
    assert_eq!(
        dead.characters[&f.characters[1]].status,
        CharacterStatus::Dead
    );
    let deaths = dead
        .rules
        .as_ref()
        .unwrap()
        .rolls
        .iter()
        .filter(|record| {
            record.request.roller == Some(f.actors[1])
                && matches!(&record.purpose, PendingPurpose::TacticalResolution { key, .. }
                if key.role == TacticalRollRole::DeathSave)
        })
        .collect::<Vec<_>>();
    assert_eq!(deaths.len(), 2);
    assert_eq!(
        deaths[0].result.dice,
        vec![DieResult {
            sides: 20,
            value: 1
        }]
    );
    assert_eq!(
        deaths[1].result.dice,
        vec![DieResult {
            sides: 20,
            value: 2
        }]
    );
    assert_eq!(deaths[0].accepted_by.id, first.0.command_id);
    assert_eq!(deaths[1].accepted_by.id, second.0.command_id);
    // Retain the dead participant while its actual bound owner ends this turn.
    Box::pin(accept(
        f,
        url,
        directory,
        "dead-turn-ended",
        Some(1),
        action(TacticalAction::EndTurn),
    ))
    .await;
    let before_release = Box::new(state(f).await);
    let timing = before_release
        .rules
        .as_ref()
        .unwrap()
        .timing
        .as_ref()
        .unwrap();
    assert_eq!(timing.turn_number, 9);
    let goblin = timing.order[timing.index].actor;
    assert!(!f.actors.contains(&goblin));
    assert!(
        before_release
            .encounter
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .resolution
            .is_none()
    );
    assert!(
        view(f, None)
            .await
            .tactical
            .unwrap()
            .release
            .unwrap()
            .may_finish
    );
    assert_corpse_unchanged(f, &before_release, &dead);
    DeathHistory {
        goblin,
        before_release,
        receipts: vec![first, second],
    }
}

async fn closed_release_and_survivor_session(
    f: &mut Fixture,
    url: &str,
    directory: &Path,
    history: &DeathHistory,
) {
    Box::pin(accept(
        f,
        url,
        directory,
        "closed-dead-aftermath",
        None,
        TableAction::EndSession,
    ))
    .await;
    f.session = PlaySessionId::new();
    let premature = request(f, None, survivor_session(f)).await;
    let error = Box::pin(rejected_request(f, premature)).await;
    assert!(
        error.contains("every retained character's controller"),
        "{error}"
    );
    let before = Box::new(export_campaign(&f.pool, f.campaign).await.unwrap());
    let finish = closed_request(f, action(TacticalAction::FinishEncounter)).await;
    assert_eq!(finish.session_id, None);
    Box::pin(durable_step(
        f,
        url,
        directory,
        "dead-closed-release",
        finish,
    ))
    .await;
    let after = Box::new(export_campaign(&f.pool, f.campaign).await.unwrap());
    assert_eq!(
        after.play_sessions, before.play_sessions,
        "release creates no attendance session"
    );
    assert_eq!(
        after.play_session_participants,
        before.play_session_participants
    );
    let finished = Box::new(state(f).await);
    assert!(finished.table.as_ref().unwrap().active_session.is_none());
    assert_eq!(finished.clock, history.before_release.clock);
    assert_eq!(finished.items, history.before_release.items);
    assert_eq!(
        finished.rules.as_ref().unwrap().entities,
        history.before_release.rules.as_ref().unwrap().entities
    );
    assert_eq!(
        finished.rules.as_ref().unwrap().tactical_inventory,
        history
            .before_release
            .rules
            .as_ref()
            .unwrap()
            .tactical_inventory
    );
    assert_corpse_unchanged(f, &finished, &history.before_release);
    let old_scene = history.before_release.encounter.as_ref().unwrap().scene_id;
    assert_eq!(finished.scenes[&old_scene].status, SceneStatus::Closed);
    assert!(
        view(f, None)
            .await
            .tactical
            .unwrap()
            .release
            .unwrap()
            .required_actors
            .is_empty()
    );
    let start = survivor_session(f);
    Box::pin(accept(
        f,
        url,
        directory,
        "survivor-only-session",
        None,
        start,
    ))
    .await;
    let current = Box::new(state(f).await);
    let actual = current
        .table
        .as_ref()
        .unwrap()
        .active_session
        .as_ref()
        .unwrap();
    assert_eq!(actual.session_id, f.session);
    assert_eq!(
        actual.participants,
        vec![SessionParticipant {
            player_id: f.players[0],
            character_id: Some(f.characters[0]),
            attendance: AttendanceStatus::Present,
        }]
    );
    assert_corpse_unchanged(f, &current, &history.before_release);
}

async fn survivor_encounter(f: &mut Fixture, url: &str, directory: &Path, history: &DeathHistory) {
    let finished = Box::new(state(f).await);
    let mut setup = replacement(f, &finished, history.goblin);
    setup.name = "Survivors at the next battlefield".into();
    setup.creatures[0].public_label = "Armored sentry".into();
    setup.creatures[0].position = SpatialPoint { x: 30, y: 10, z: 0 };
    setup.creatures[0].height = 8;
    let mut corpse_setup = setup.clone();
    corpse_setup.characters.push(TableCharacterPlacement {
        character_id: f.characters[1],
        position: SpatialPoint { x: 20, y: 30, z: 0 },
        height: 12,
        allies: vec![f.actors[0]],
        enemies: vec![history.goblin],
    });
    let invalid = request(
        f,
        None,
        TableAction::PrepareBattlefield {
            setup: Box::new(corpse_setup),
        },
    )
    .await;
    let error = Box::pin(rejected_request(f, invalid)).await;
    assert!(
        error.contains("A dead character cannot enter a new encounter"),
        "{error}"
    );
    Box::pin(accept(
        f,
        url,
        directory,
        "survivor-battlefield",
        None,
        TableAction::PrepareBattlefield {
            setup: Box::new(setup),
        },
    ))
    .await;
    let actor = f.actors[0];
    let begin = action(TacticalAction::Begin {
        execution: TacticalExecutionVersion::EncounterReleaseV1,
        combatants: vec![
            TacticalCombatant {
                actor,
                source: TacticalSource::Character,
                surprised: false,
            },
            TacticalCombatant {
                actor: history.goblin,
                source: TacticalSource::Creature {
                    definition_id: "goblin-warrior".into(),
                },
                surprised: false,
            },
        ],
        groups: [actor, history.goblin]
            .into_iter()
            .map(|actor| InitiativeGroup {
                actors: vec![actor],
                request_id: RollRequestId::new(),
            })
            .collect(),
    });
    Box::pin(accept(
        f,
        url,
        directory,
        "survivor-initiative",
        None,
        begin,
    ))
    .await;
    for (player, faces, label) in [
        (Some(0), [20], "survivor-physical-initiative"),
        (None, [2], "surviving-source-initiative"),
    ] {
        let raw = raw_action(f, player, &faces).await;
        Box::pin(accept(f, url, directory, label, player, raw)).await;
    }
    let active = Box::new(state(f).await);
    let timing = active.rules.as_ref().unwrap().timing.as_ref().unwrap();
    assert_eq!(timing.turn_number, 10);
    assert_eq!(timing.round, 1);
    assert_eq!(timing.order[timing.index].actor, actor);
    assert!(timing.order.iter().all(|entry| entry.actor != f.actors[1]));
    assert!(
        active
            .encounter
            .as_ref()
            .unwrap()
            .participant(f.actors[1])
            .is_none()
    );
    assert_corpse_unchanged(f, &active, &history.before_release);
    let movement = action(TacticalAction::Move {
        path: vec![TacticalMoveStep {
            destination: SpatialPoint { x: 20, y: 10, z: 0 },
            mode: MovementMode::Walk,
        }],
    });
    Box::pin(rejected(f, Some(1), movement.clone())).await;
    Box::pin(accept(
        f,
        url,
        directory,
        "survivor-actual-movement",
        Some(0),
        movement,
    ))
    .await;
    let played = Box::new(state(f).await);
    assert_eq!(
        played
            .encounter
            .as_ref()
            .unwrap()
            .participant(actor)
            .unwrap()
            .position
            .x,
        20
    );
    assert_corpse_unchanged(f, &played, &history.before_release);
    let before = Box::new(export_campaign(&f.pool, f.campaign).await.unwrap());
    for (original, response) in &history.receipts {
        assert_eq!(
            serde_json::to_vec(
                &Box::pin(f.runtime.submit_presented_table(original.clone()))
                    .await
                    .unwrap()
            )
            .unwrap(),
            serde_json::to_vec(response).unwrap()
        );
        let mut changed = original.clone();
        let TableTransportInput::Action(declaration) = &mut changed.input else {
            panic!("death raw is an action")
        };
        let TableAction::Tactical {
            action: TacticalAction::SubmitRoll { result },
        } = declaration.as_mut()
        else {
            panic!("original physical death save body is retained")
        };
        result.dice[0].value = 20;
        assert!(
            Box::pin(f.runtime.submit_presented_table(changed))
                .await
                .is_err()
        );
    }
    let mut after = Box::new(export_campaign(&f.pool, f.campaign).await.unwrap());
    after.exported_at_utc = before.exported_at_utc.clone();
    assert_eq!(
        after, before,
        "old death-save retries preserve the corpse and survivor session exactly"
    );
}

#[tokio::test]
async fn genuine_death_saves_remain_owed_until_death_then_release_allows_survivor_only_play() {
    let directory = std::env::temp_dir().join(format!("dmd-release-death-{}", CampaignId::new().0));
    std::fs::create_dir_all(&directory).unwrap();
    let url = format!(
        "sqlite://{}",
        directory
            .join("campaign.sqlite")
            .to_string_lossy()
            .replace('\\', "/")
    );
    let pool = open_sqlite(&url).await.unwrap();
    let mut f = Box::pin(Fixture::with_pool(TableContract::default(), pool)).await;
    Box::pin(dying_closed_refusal(&mut f, &url, &directory)).await;
    let history = Box::pin(actual_death(&mut f, &url, &directory)).await;
    Box::pin(closed_release_and_survivor_session(
        &mut f, &url, &directory, &history,
    ))
    .await;
    Box::pin(survivor_encounter(&mut f, &url, &directory, &history)).await;
    f.pool.close().await;
    drop(f);
    sqlite_test_cleanup::remove_closed_directory(&directory)
        .await
        .unwrap();
}
