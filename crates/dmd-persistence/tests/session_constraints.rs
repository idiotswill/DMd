use dmd_persistence::migrate_sqlite;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::str::FromStr;

async fn test_pool() -> sqlx::SqlitePool {
    let options = SqliteConnectOptions::from_str("sqlite::memory:")
        .expect("valid SQLite URL")
        .foreign_keys(true);
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await
        .expect("in-memory SQLite should open");
    migrate_sqlite(&pool)
        .await
        .expect("migrations should initialize");
    pool
}

#[tokio::test]
async fn persisted_session_cannot_move_between_campaigns() {
    let pool = test_pool().await;
    sqlx::query(
        "INSERT INTO play_sessions (id, campaign_id, display_name, status, started_at_world, ended_at_world) VALUES ('session-1', 'campaign-a', 'Session', 'closed', 0, 1)",
    )
    .execute(&pool)
    .await
    .expect("fixture session should insert");

    let result =
        sqlx::query("UPDATE play_sessions SET campaign_id = 'campaign-b' WHERE id = 'session-1'")
            .execute(&pool)
            .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn participant_ordinals_are_unique_within_a_session() {
    let pool = test_pool().await;
    sqlx::query(
        "INSERT INTO play_sessions (id, campaign_id, display_name, status, started_at_world, ended_at_world) VALUES ('session-1', 'campaign-a', 'Session', 'closed', 0, 1)",
    )
    .execute(&pool)
    .await
    .expect("fixture session should insert");
    sqlx::query(
        "INSERT INTO play_session_participants (session_id, ordinal, player_id, character_id, attendance) VALUES ('session-1', 0, 'player-a', NULL, 'present')",
    )
    .execute(&pool)
    .await
    .expect("first participant should insert");

    let result = sqlx::query(
        "INSERT INTO play_session_participants (session_id, ordinal, player_id, character_id, attendance) VALUES ('session-1', 0, 'player-b', NULL, 'present')",
    )
    .execute(&pool)
    .await;

    assert!(result.is_err());
}
