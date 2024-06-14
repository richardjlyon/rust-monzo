//! Models for the account endpoint

use async_trait::async_trait;
use chrono::{DateTime, TimeZone, Utc};
use serde::Deserialize;
use sqlx::{prelude::FromRow, Pool, Sqlite};
use tracing_log::log::{error, info};

use super::DatabasePool;
use crate::error::AppErrors as Error;

#[derive(Deserialize, Debug)]
pub struct Accounts {
    pub accounts: Vec<Account>,
}

#[derive(Deserialize, Debug, Default, FromRow)]
pub struct Account {
    pub id: String,
    pub closed: bool,
    pub created: DateTime<Utc>,
    pub description: String,
    pub currency: String,
    pub country_code: String,
    pub owner_type: String,
    pub account_number: String,
    pub sort_code: String,
}

// -- Services ------------------------------------------------

#[async_trait]
pub trait Service {
    async fn save_account(&self, acc_fc: &Account) -> Result<(), Error>;
    async fn read_accounts(&self) -> Result<Vec<Account>, Error>;
}

#[derive(Debug, Clone)]
pub struct SqliteAccountService {
    pub(crate) pool: DatabasePool,
}

impl SqliteAccountService {
    #[must_use]
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }
}

// -- Service Implementations ----------------------------------------------------------

#[async_trait]
impl Service for SqliteAccountService {
    #[tracing::instrument(
        name = "Creating account",
        skip(self, acc_fc),
        fields(id = %acc_fc.id)
    )]
    async fn save_account(&self, acc_fc: &Account) -> Result<(), Error> {
        let db = self.pool.db();

        if is_duplicate_account(db, &acc_fc.id).await? {
            info!("Account exists. Skipping");
            return Err(Error::Duplicate("Account already exists".to_string()));
        }

        info!("Inserting account");
        match sqlx::query!(
            r"
                INSERT INTO accounts (
                    id,
                    closed,
                    created,
                    description,
                    currency,
                    country_code,
                    owner_type,
                    account_number,
                    sort_code
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ",
            acc_fc.id,
            acc_fc.closed,
            acc_fc.created,
            acc_fc.description,
            acc_fc.currency,
            acc_fc.country_code,
            acc_fc.owner_type,
            acc_fc.account_number,
            acc_fc.sort_code,
        )
        .execute(db)
        .await
        {
            Ok(_) => {
                info!("Created account: {}", acc_fc.id);
                Ok(())
            }
            Err(e) => {
                error!("Failed to create account: {}", acc_fc.id);
                Err(Error::DbError(e.to_string()))
            }
        }
    }

    #[tracing::instrument(name = "Getting accounts", skip(self))]
    async fn read_accounts(&self) -> Result<Vec<Account>, Error> {
        let db = self.pool.db();

        let rows = sqlx::query!(
            r"
                SELECT *
                FROM accounts
            "
        )
        .fetch_all(db)
        .await;

        match rows {
            Ok(rows) => {
                info!("Read {} accounts", rows.len());
                Ok(rows
                    .into_iter()
                    .map(|row| Account {
                        id: row.id,
                        closed: row.closed,
                        created: TimeZone::from_utc_datetime(&Utc, &row.created),
                        description: row.description,
                        currency: row.currency,
                        country_code: row.country_code,
                        owner_type: row.owner_type,
                        account_number: row.account_number,
                        sort_code: row.sort_code,
                    })
                    .collect())
            }
            Err(e) => {
                error!("Failed to read transactions. Reason: {}", e.to_string());
                return Err(Error::DbError(e.to_string()));
            }
        }
    }
}

// Check if an account is a duplicate
async fn is_duplicate_account(db: &Pool<Sqlite>, acc_id: &str) -> Result<bool, Error> {
    let existing_account = sqlx::query!(
        r"
            SELECT id
            FROM accounts
            WHERE id = $1
        ",
        acc_id,
    )
    .fetch_optional(db)
    .await?;

    Ok(existing_account.is_some())
}

// -- Tests ----------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::test::test_db;

    #[tokio::test]
    async fn create_account() {
        // Arrange
        let (pool, _tmp) = test_db().await;
        let service = SqliteAccountService::new(pool);
        let acc = Account::default();

        // Act
        let result = service.save_account(&acc).await;

        // Assert
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn read_accounts() {
        // Arrange
        let (pool, _tmp) = test_db().await;
        let service = SqliteAccountService::new(pool);

        // Act
        let result = service.read_accounts().await.unwrap();

        // Assert
        assert_eq!(result.len(), 1);
    }
}
