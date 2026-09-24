pub mod journal_store;
pub mod lifecycle;
pub mod projection_store;
pub mod session_store;
pub mod snapshot_replay;

pub use journal_store::*;
pub use lifecycle::*;
pub use projection_store::*;
pub use session_store::*;
pub use snapshot_replay::*;

pub async fn open_sqlite(database_url: &str) -> Result<sqlx::SqlitePool, sqlx::Error> {
    use std::str::FromStr;

    use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

    let options = SqliteConnectOptions::from_str(database_url)?.foreign_keys(true);
    SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await
}

pub async fn migrate_sqlite(pool: &sqlx::SqlitePool) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!("./migrations").run(pool).await
}
