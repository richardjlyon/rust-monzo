use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    SqlitePool,
};

use crate::configuration::Settings;
use crate::error::AppError as Error;

pub mod account;
pub mod balance;
pub mod merchant;
pub mod pots;
pub mod transaction;

/// A holder for a backing store. Allows swapping out implementations.
#[derive(Debug, Clone)]
pub struct DatabasePool {
    pool: SqlitePool,
}

impl DatabasePool {
    /// Constructor
    #[tracing::instrument(name = "Creating a database pool")]
    pub async fn new(path: &str, max_connections: u32) -> Result<Self, Error> {
        let options = SqliteConnectOptions::new()
            .auto_vacuum(sqlx::sqlite::SqliteAutoVacuum::Incremental)
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
            .synchronous(sqlx::sqlite::SqliteSynchronous::Normal)
            .pragma("temp_store", "memory")
            .pragma("mmap_size", "30000000000")
            .create_if_missing(true)
            .filename(path);

        let pool = SqlitePoolOptions::new()
            .max_connections(max_connections)
            .connect_with(options)
            .await?;

        // add a few pragmas

        // do a migration
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();

        Ok(DatabasePool { pool })
    }

    pub async fn new_from_config(config: Settings) -> Result<Self, Error> {
        Self::new(
            &config.database.database_path,
            config.database.max_connections,
        )
        .await
    }

    /// Returns the sqlx db pool reference
    /// (only for the model layer)
    pub fn db(&self) -> &SqlitePool {
        &self.pool
    }
}
