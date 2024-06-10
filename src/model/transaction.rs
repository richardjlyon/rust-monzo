//! Models for the transaction endpoint

use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer};
use sqlx::{Pool, Sqlite};
use std::collections::HashMap;
use tracing_log::log::{error, info};

use super::{merchant::Merchant, DatabasePool};
use crate::error::AppError as Error;

#[derive(Deserialize, Debug)]
pub struct Transactions {
    pub transactions: Vec<Transaction>,
}

#[derive(Deserialize, Debug)]
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
    pub categories: Categories,
    pub attachments: Option<Vec<Attachment>>,
}

#[derive(Deserialize, Debug)]
pub struct Categories {
    #[serde(flatten)]
    _fields: HashMap<String, i32>,
}

#[derive(Deserialize, Debug)]
pub struct Attachment {
    _id: String,
    _external_id: String,
    _file_url: String,
    _file_type: String,
    _created: DateTime<Utc>,
}

// -- Services -------------------------------------------------------------------------

pub trait TransactionService {
    async fn create_transaction(&self, tx_fc: &Transaction) -> Result<(), Error>;
    async fn delete_all_transactions(&self) -> Result<(), Error>;
}

#[derive(Debug, Clone)]
pub struct SqliteTransactionService {
    pub(crate) pool: DatabasePool,
}

impl SqliteTransactionService {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }
}

// -- Service Implementations ----------------------------------------------------------

impl TransactionService for SqliteTransactionService {
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
        if let Some(merchant_id) = transaction_merchant_id(&tx_fc) {
            if !merchant_exists(db, &merchant_id).await? {
                // ...insert the merchant
                insert_merchant(db, &tx_fc.merchant.as_ref().unwrap()).await?;
            }
        }

        let merchant_id = transaction_merchant_id(&tx_fc);

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
                "invalid date-time format: {}",
                s
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
    transaction
        .merchant
        .as_ref()
        .map_or(None, |m| Some(m.id.clone()))
}

// check if a merchant exists
async fn merchant_exists(db: &Pool<Sqlite>, merchant_id: &str) -> Result<bool, Error> {
    let merchant = sqlx::query!(
        r"
                SELECT id
                FROM merchants
                WHERE id = $1
            ",
        merchant_id,
    )
    .fetch_optional(db)
    .await?;

    Ok(merchant.is_some())
}

// insert a merchant
#[tracing::instrument(name = "Insert merchant", skip(db, merchant))]
async fn insert_merchant(db: &Pool<Sqlite>, merchant: &Merchant) -> Result<(), Error> {
    match sqlx::query!(
        r"
            INSERT INTO merchants (
                id,
                name,
                category
            )
            VALUES ($1, $2, $3)
        ",
        merchant.id,
        merchant.name,
        merchant.category,
    )
    .execute(db)
    .await
    {
        Ok(_) => {
            info!("Created merchant: {:?}", merchant.id);
            Ok(())
        }
        Err(e) => {
            error!("Failed to create merchant: {:?}", merchant.id);
            Err(Error::DbError(e.to_string()))
        }
    }
}
