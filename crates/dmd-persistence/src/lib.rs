pub mod journal_store;
pub mod lifecycle;
pub mod observation_store;
pub mod projection_store;
pub mod session_store;
pub mod snapshot_replay;

pub use journal_store::*;
pub use lifecycle::*;
pub use observation_store::*;
pub use projection_store::*;
pub use session_store::*;
pub use snapshot_replay::*;

use sqlx::{
    SqlitePool,
    migrate::{MigrateError, Migrator},
    sqlite::SqliteConnectOptions,
};
use std::str::FromStr;
use thiserror::Error;

pub static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

#[derive(Debug, Error)]
pub enum PersistenceError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Migration(#[from] MigrateError),
}

pub async fn migrate_sqlite(pool: &SqlitePool) -> Result<(), MigrateError> {
    // SQLx normally commits each migration separately. Keep the complete upgrade atomic:
    // later state-schema preflight failures must also roll back earlier pending migrations.
    // SQLite's nested SQLx migration transactions become savepoints inside this transaction.
    let mut transaction = pool.begin().await?;
    // The shipped SQL migrations must keep their checksums. Newly recognized optional
    // authority must be rejected before an old row can be relabeled as schema 4.
    let has_current: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'campaign_state_current')",
    ).fetch_one(&mut *transaction).await?;
    if has_current {
        let legacy: Vec<String> = sqlx::query_scalar(
            "SELECT state_json FROM campaign_state_current WHERE schema_version < 4",
        )
        .fetch_all(&mut *transaction)
        .await?;
        for json in legacy {
            snapshot_replay::preflight_legacy_authority(&json)
                .map_err(|error| MigrateError::Execute(sqlx::Error::Protocol(error)))?;
        }
    }
    // Use the already acquired connection. `run` adds an Acquire lifetime that
    // prevents this future satisfying Send at the native desktop command boundary.
    MIGRATOR.run_direct(&mut *transaction).await?;
    transaction.commit().await?;
    Ok(())
}

pub async fn open_sqlite(database_url: &str) -> Result<SqlitePool, PersistenceError> {
    let options = SqliteConnectOptions::from_str(database_url)?
        .create_if_missing(true)
        .foreign_keys(true);

    open_with_options(options).await
}

/// Native paths are not parsed as URLs, so spaces, `?`, `#` and Unicode remain file names.
pub async fn open_sqlite_path(
    path: impl AsRef<std::path::Path>,
) -> Result<SqlitePool, PersistenceError> {
    open_with_options(
        SqliteConnectOptions::new()
            .filename(path)
            .create_if_missing(true)
            .foreign_keys(true),
    )
    .await
}

async fn open_with_options(options: SqliteConnectOptions) -> Result<SqlitePool, PersistenceError> {
    let pool = SqlitePool::connect_with(options).await?;
    migrate_sqlite(&pool).await?;
    Ok(pool)
}
