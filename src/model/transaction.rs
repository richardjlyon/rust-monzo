//! Models for the transaction endpoint
#![allow(dead_code)]
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer};
use sqlx::{Pool, Sqlite};
use std::collections::HashMap;
use tracing_log::log::{error, info};

use super::{
    merchant::{Merchant, Service as MerchantService, SqliteMerchantService},
    DatabasePool,
};
use crate::error::AppErrors as Error;

#[derive(Deserialize, Debug)]
pub struct Transactions {
    pub transactions: Vec<Transaction>,
}

#[allow(clippy::module_name_repetitions)]
#[derive(Deserialize, Debug, Default)]
pub struct Transaction {
    pub id: String,
    pub dedupe_id: String,
    pub account_id: String,
    pub amount: i64,
    pub currency: String,
    pub local_amount: i64,
    pub local_currency: String,
    // #[serde(deserialize_with = "deserialize_optional_datetime")]
    pub created: DateTime<Utc>,
    pub description: String,
    pub amount_is_pending: bool,
    pub merchant: Option<Merchant>,
    pub notes: String,
    #[serde(deserialize_with = "deserialize_optional_datetime")]
    pub settled: Option<DateTime<Utc>>,
    #[serde(deserialize_with = "deserialize_optional_datetime")]
    pub updated: Option<DateTime<Utc>>,
    pub category: String,
    pub categories: HashMap<String, i32>,
    pub attachments: Option<Vec<Attachment>>,
}

#[derive(Deserialize, Debug)]
pub struct Attachment {
    id: String,
    external_id: String,
    file_url: String,
    file_type: String,
    created: DateTime<Utc>,
}

// -- Services -------------------------------------------------------------------------

#[async_trait]
pub trait Service {
    async fn create_transaction(&self, tx_fc: &Transaction) -> Result<(), Error>;
    async fn delete_all_transactions(&self) -> Result<(), Error>;
}

#[derive(Debug, Clone)]
pub struct SqliteTransactionService {
    pub(crate) pool: DatabasePool,
}

impl SqliteTransactionService {
    #[must_use]
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }
}

// -- Service Implementations ----------------------------------------------------------

#[async_trait]
impl Service for SqliteTransactionService {
    #[tracing::instrument(
        name = "Create transaction",
        skip(self, tx_fc),
        fields(tx_id = %tx_fc.id, acc_id = %tx_fc.account_id)
    )]
    async fn create_transaction(&self, tx_fc: &Transaction) -> Result<(), Error> {
        let db = self.pool.db();

        if is_duplicate_transaction(db, &tx_fc.id).await? {
            info!("Transaction exists. Skipping");
            return Err(Error::Duplicate("Transaction already exists".to_string()));
        }

        // if the transaction has a merchant and it doesn't exist in the db...
        if let Some(merchant_id) = transaction_merchant_id(tx_fc) {
            if !merchant_exists(&self.pool, &merchant_id).await? {
                // ...insert the merchant
                let merchant = tx_fc
                    .merchant
                    .as_ref()
                    .expect("Merchant not found in transaction");
                insert_merchant(&self.pool, merchant).await?;
            }
        }

        let merchant_id = transaction_merchant_id(tx_fc);

        info!("Inserting transaction");
        match sqlx::query!(
            r"
                INSERT INTO transactions (
                    id,
                    account_id,
                    merchant_id,
                    amount,
                    currency,
                    local_amount,
                    local_currency,
                    created,
                    description,
                    notes,
                    settled,
                    updated,
                    category
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            ",
            tx_fc.id,
            tx_fc.account_id,
            merchant_id,
            tx_fc.amount,
            tx_fc.currency,
            tx_fc.local_amount,
            tx_fc.local_currency,
            tx_fc.created,
            tx_fc.description,
            tx_fc.notes,
            tx_fc.settled,
            tx_fc.updated,
            tx_fc.category,
        )
        .execute(db)
        .await
        {
            Ok(_) => {
                info!("Created transaction: {}", tx_fc.id);
                Ok(())
            }
            Err(e) => {
                error!(
                    "Failed to create transaction: {}. Reason: {}. Account id: {}. Merchant id: {}",
                    tx_fc.id,
                    e.to_string(),
                    tx_fc.account_id,
                    merchant_id.unwrap_or("None".to_string()),
                );
                Err(Error::DbError(e.to_string()))
            }
        }
    }

    #[tracing::instrument(name = "Delete all transactions", skip(self))]
    async fn delete_all_transactions(&self) -> Result<(), Error> {
        let db = self.pool.db();

        match sqlx::query!("DELETE FROM transactions").execute(db).await {
            Ok(_) => {
                info!("Deleted all transactions");
                Ok(())
            }
            Err(e) => {
                error!("Failed to delete all transactions: {}", e.to_string());
                Err(Error::DbError(e.to_string()))
            }
        }
    }
}

// -- Utility functions ----------------------------------------------------------------

// Custom deserialization function for Option<DateTime<Utc>>
fn deserialize_optional_datetime<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt: Option<String> = Option::deserialize(deserializer)?;
    match opt.as_deref() {
        Some("") | None => Ok(None),
        Some(s) => match DateTime::parse_from_rfc3339(s) {
            Ok(dt) => Ok(Some(dt.with_timezone(&Utc))),
            Err(_) => Err(serde::de::Error::custom(format!(
                "invalid date-time format: {s}"
            ))),
        },
    }
}

// Check if a transaction is a duplicate
async fn is_duplicate_transaction(db: &Pool<Sqlite>, tx_id: &str) -> Result<bool, Error> {
    let existing_transaction = sqlx::query!(
        r"
            SELECT id
            FROM transactions
            WHERE id = $1
        ",
        tx_id,
    )
    .fetch_optional(db)
    .await?;

    Ok(existing_transaction.is_some())
}

// check if a transaction has a merchant
fn transaction_merchant_id(transaction: &Transaction) -> Option<String> {
    transaction.merchant.as_ref().map(|m| m.id.clone())
}

// check if a merchant exists
async fn merchant_exists(db: &DatabasePool, merchant_id: &str) -> Result<bool, Error> {
    let merchant_service = SqliteMerchantService::new(db.clone());
    let merchant = merchant_service.get_merchant(merchant_id).await?;

    Ok(merchant.is_some())
}

// insert a merchant
async fn insert_merchant(db: &DatabasePool, merchant: &Merchant) -> Result<(), Error> {
    let merchant_service = SqliteMerchantService::new(db.clone());
    merchant_service.create_merchant(merchant).await
}

// -- Tests ----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::test::test_db;

    #[tokio::test]
    async fn create_transaction() {
        // Arrange
        let (pool, _tmp) = test_db().await;
        let service = SqliteTransactionService::new(pool);
        let mut tx = Transaction::default();
        tx.account_id = "1".to_string();

        // Act
        let result = service.create_transaction(&tx).await;

        //Assert
        assert!(result.is_ok());
    }
}
