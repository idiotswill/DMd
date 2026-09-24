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
    MIGRATOR.run(pool).await
}

pub async fn open_sqlite(database_url: &str) -> Result<SqlitePool, PersistenceError> {
    let options = SqliteConnectOptions::from_str(database_url)?
        .create_if_missing(true)
        .foreign_keys(true);

    let pool = SqlitePool::connect_with(options).await?;
    migrate_sqlite(&pool).await?;
    Ok(pool)
}
