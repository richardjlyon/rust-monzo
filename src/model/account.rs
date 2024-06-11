//! Models for the account endpoint

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use sqlx::{Pool, Sqlite};
use tracing_log::log::{error, info};

use super::DatabasePool;
use crate::error::AppError as Error;

#[derive(Deserialize, Debug)]
pub struct Accounts {
    pub accounts: Vec<Account>,
}

#[derive(Deserialize, Debug, Default)]
pub struct Account {
    pub id: String,
    pub closed: bool,
    pub created: DateTime<Utc>,
    pub description: String,
    pub owner_type: String,
    pub account_number: String,
    pub sort_code: String,
}

// -- Services ------------------------------------------------

#[async_trait]
pub trait AccountService {
    async fn create_account(&self, acc_fc: &Account) -> Result<(), Error>;
}

#[derive(Debug, Clone)]
pub struct SqliteAccountService {
    pub(crate) pool: DatabasePool,
}

impl SqliteAccountService {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }
}

// -- Service Implementations ----------------------------------------------------------

#[async_trait]
impl AccountService for SqliteAccountService {
    #[tracing::instrument(
        name = "Creating account",
        skip(self, acc_fc),
        fields(id = %acc_fc.id)
    )]
    async fn create_account(&self, acc_fc: &Account) -> Result<(), Error> {
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
                    owner_type,
                    account_number,
                    sort_code
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7)
            ",
            acc_fc.id,
            acc_fc.closed,
            acc_fc.created,
            acc_fc.description,
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
    async fn create_accoun_workst() {
        // Arrange
        let (pool, _tmp) = test_db().await;
        let service = SqliteAccountService::new(pool);
        let acc = Account::default();

        // Act
        let result = service.create_account(&acc).await;

        // Assert
        assert!(result.is_ok());
    }
}
