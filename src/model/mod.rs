use account::AccountForDB;
use category::Category;
use chrono::Utc;
use pot::PotResponse;
use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    SqlitePool,
};
use transaction::TransactionForDB;

use crate::configuration::Settings;
use crate::error::AppErrors as Error;

pub mod account;
pub mod balance;
pub mod category;
pub mod merchant;
pub mod pot;
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
        sqlx::migrate!("./migrations").run(&pool).await?;

        Ok(DatabasePool { pool })
    }

    /// Create a new database pool from the information in configuration
    ///
    /// # Errors
    /// Will return an error if configuration is not valid or the pool can't be created
    pub async fn new_from_config(config: Settings) -> Result<Self, Error> {
        Self::new(
            &config.database.database_path,
            config.database.max_connections,
        )
        .await
    }

    /// Returns the sqlx db pool reference
    /// (only for the model layer)
    #[must_use]
    pub fn db(&self) -> &SqlitePool {
        &self.pool
    }

    /// Seed the test database with initial data
    ///
    /// # Errors
    /// Will return an error if the seed data can't be inserted
    pub async fn seed_initial_data(&self) -> Result<(), Error> {
        let db = self.db();

        // insert account
        let account = AccountForDB {
            id: "1".to_string(),
            closed: false,
            created: Utc::now().naive_utc(),
            description: "Main Account".to_string(),
            currency: "GBP".to_string(),
            country_code: "GB".to_string(),
            owner_type: "personal".to_string(),
            account_number: "12345678".to_string(),
            sort_code: "12-34-56".to_string(),
        };

        sqlx::query!(
            r#"
            INSERT INTO accounts (
                id, closed, created, description, currency, country_code, owner_type, account_number, sort_code
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            "#,
            account.id,
            account.closed,
            account.created,
            account.description,
            account.currency,
            account.country_code,
            account.owner_type,
            account.account_number,
            account.sort_code,
        )
        .execute(db)
        .await?;

        let category = Category {
            id: "1".to_string(),
            name: "category_1".to_string(),
        };

        sqlx::query!(
            r#"
            INSERT INTO categories (id, name)
            VALUES (?1, ?2)
            "#,
            category.id,
            category.name,
        )
        .execute(db)
        .await?;

        // insert transactions

        let mut tx1 = TransactionForDB::default();
        tx1.id = "1".to_string();
        tx1.account_id = account.id.clone();
        tx1.category_id = category.id.clone();

        let mut tx2 = TransactionForDB::default();
        tx2.id = "2".to_string();
        tx2.account_id = account.id.clone();
        tx2.category_id = category.id.clone();

        for tx in vec![tx1, tx2] {
            sqlx::query!(
                r#"
                INSERT INTO transactions (id, account_id, amount, local_amount, currency, local_currency, description, created, category_id)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
                "#,
                tx.id,
                tx.account_id,
                tx.amount,
                tx.local_amount,
                tx.currency,
                tx.local_currency,
                tx.description,
                tx.created,
                tx.category_id,
            )
            .execute(db)
            .await?;
        }

        // insert pot
        let pot = PotResponse {
            id: "1".to_string(),
            name: "pot_name".to_string(),
            balance: 1234,
            currency: "GBP".to_string(),
            deleted: false,
            pot_type: "default".to_string(),
        };

        sqlx::query!(
            r#"
            INSERT INTO pots (id, name, balance, currency, deleted, pot_type)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            "#,
            pot.id,
            pot.name,
            pot.balance,
            pot.currency,
            pot.deleted,
            pot.pot_type,
        )
        .execute(db)
        .await?;

        Ok(())
    }
}
