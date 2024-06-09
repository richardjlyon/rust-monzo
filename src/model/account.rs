//! Models for the account endpoint

use chrono::{DateTime, Utc};
use serde::Deserialize;

use super::DatabasePool;
use crate::error::AppError as Error;

#[derive(Deserialize, Debug)]
pub struct Accounts {
    pub accounts: Vec<Account>,
}

#[derive(Deserialize, Debug)]
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

impl AccountService for SqliteAccountService {
    async fn create_account(&self, acc_fc: &Account) -> Result<(), Error> {
        let db = self.pool.db();

        sqlx::query!(
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
        .await?;

        Ok(())
    }
}
