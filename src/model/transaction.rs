//! Models for the transaction endpoint
#![allow(dead_code)]
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer};
use sqlx::{Pool, Sqlite};
use tracing_log::log::{error, info};

use super::{
    merchant::{Merchant, Service as MerchantService, SqliteMerchantService},
    DatabasePool,
};
use crate::error::AppErrors as Error;

#[derive(Deserialize, Debug)]
pub struct Transactions {
    pub transactions: Vec<TransactionResponse>,
}

/// Represents a transaction response from the Monzo API
#[allow(clippy::module_name_repetitions)]
#[derive(Deserialize, Debug, Default, Clone)]
pub struct TransactionResponse {
    pub id: String,
    pub account_id: String,
    pub merchant: Option<Merchant>,
    pub amount: i64,
    pub currency: String,
    pub local_amount: i64,
    pub local_currency: String,
    pub created: DateTime<Utc>,
    pub description: Option<String>,
    pub notes: Option<String>,
    #[serde(deserialize_with = "deserialize_optional_datetime")]
    pub settled: Option<DateTime<Utc>>,
    #[serde(deserialize_with = "deserialize_optional_datetime")]
    pub updated: Option<DateTime<Utc>>,
    pub category: String,
    // pub categories: HashMap<String, i32>,
    // pub attachments: Option<Vec<Attachment>>,
    // pub amount_is_pending: bool,
    // pub dedupe_id: String,
}

/// Represents an attachment from the Monzo API
#[derive(Deserialize, Debug)]
pub struct Attachment {
    id: String,
    external_id: String,
    file_url: String,
    file_type: String,
    created: DateTime<Utc>,
}

/// Represents a transaction from the database
#[derive(Debug, Default)]
pub struct Transaction {
    pub id: String,
    pub account_id: String,
    pub merchant_id: Option<String>,
    pub amount: i64,
    pub currency: String,
    pub local_amount: i64,
    pub local_currency: String,
    pub created: DateTime<Utc>,
    pub description: Option<String>,
    pub notes: Option<String>,
    pub settled: Option<DateTime<Utc>>,
    pub updated: Option<DateTime<Utc>>,
    pub category: String,
}

impl From<TransactionResponse> for Transaction {
    fn from(tx: TransactionResponse) -> Self {
        Self {
            id: tx.id,
            account_id: tx.account_id,
            merchant_id: tx.merchant.map(|m| m.id),
            amount: tx.amount,
            currency: tx.currency,
            local_amount: tx.local_amount,
            local_currency: tx.local_currency,
            created: tx.created,
            description: tx.description,
            notes: tx.notes,
            settled: tx.settled,
            updated: tx.updated,
            category: tx.category,
        }
    }
}

// -- Services -------------------------------------------------------------------------

#[async_trait]
pub trait Service {
    async fn save_transaction(&self, tx_resp: &TransactionResponse) -> Result<(), Error>;
    async fn read_transactions(&self) -> Result<Vec<Transaction>, Error>;
    async fn read_transaction(&self, tx_id: &str) -> Result<Transaction, Error>;
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
        skip(self, tx_resp),
        fields(tx_id = %tx_resp.id, acc_id = %tx_resp.account_id)
    )]
    async fn save_transaction(&self, tx_resp: &TransactionResponse) -> Result<(), Error> {
        let db = self.pool.db();

        let tx = Transaction::from(tx_resp.clone());

        if is_duplicate_transaction(db, &tx.id).await? {
            info!("Transaction exists. Skipping");
            return Err(Error::Duplicate("Transaction already exists".to_string()));
        }

        let merchant_id = insert_merchant(self.pool.clone(), &tx_resp.merchant).await?;

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
            tx.id,
            tx.account_id,
            merchant_id,
            tx.amount,
            tx.currency,
            tx.local_amount,
            tx.local_currency,
            tx.created,
            tx.description,
            tx.notes,
            tx.settled,
            tx.updated,
            tx.category,
        )
        .execute(db)
        .await
        {
            Ok(_) => {
                info!("Created transaction: {}", tx.id);
                Ok(())
            }
            Err(e) => {
                error!(
                    "Failed to create transaction: {}. Reason: {}. Account id: {}. Merchant id: {}",
                    tx.id,
                    e.to_string(),
                    tx.account_id,
                    tx.merchant_id.clone().unwrap_or("None".to_string()),
                );
                Err(Error::DbError(e.to_string()))
            }
        }
    }

    #[tracing::instrument(name = "Read transactions", skip(self))]
    async fn read_transactions(&self) -> Result<Vec<Transaction>, Error> {
        todo!("Implement read_transactions")
    }

    #[tracing::instrument(name = "Read transaction", skip(self))]
    async fn read_transaction(&self, tx_id: &str) -> Result<Transaction, Error> {
        let _db = self.pool.db();

        todo!("Implement read_transaction")

        // match sqlx::query_as!(
        //     Transaction,
        //     r"
        //         SELECT * FROM transactions WHERE id = $1
        //     ",
        //     tx_id
        // )
        // .fetch_one(db)
        // .await
        // {
        //     Ok(tx) => {
        //         info!("Read transaction: {}", tx_id);
        //         Ok(tx)
        //     }
        //     Err(e) => {
        //         error!(
        //             "Failed to read transaction: {}. Reason: {}",
        //             tx_id,
        //             e.to_string()
        //         );
        //         Err(Error::DbError(e.to_string()))
        //     }
        // }
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

/// Insert a merchant into the database if it exists and isn't a duplicate
/// Returns the merchant id if it was inserted
///
/// # Errors
/// Will return an error if a merchant could not be retrieved from the database
async fn insert_merchant(
    pool: DatabasePool,
    merchant: &Option<Merchant>,
) -> Result<Option<String>, Error> {
    if merchant.is_none() {
        return Ok(None);
    }

    let merchant_service = SqliteMerchantService::new(pool);
    let merchant = merchant.as_ref().unwrap();
    match merchant_service.save_merchant(&merchant).await {
        Ok(_) | Err(Error::Duplicate(_)) => return Ok(Some(merchant.id.clone())),
        Err(e) => return Err(e),
    }
}

// -- Tests ----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::test::test_db;

    #[tokio::test]
    async fn save_transaction() {
        // Arrange
        let (pool, _tmp) = test_db().await;
        let service = SqliteTransactionService::new(pool);
        let mut tx_resp = TransactionResponse::default();
        tx_resp.account_id = "1".to_string();

        // Act
        let result = service.save_transaction(&tx_resp).await;

        //Assert
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[ignore]
    async fn read_transactions() {}

    #[tokio::test]
    #[ignore]
    async fn read_transaction() {}
}
