//! Models for the transaction endpoint

use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;
use tracing_log::log::{error, info};

use super::DatabasePool;
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
pub struct Merchant {
    pub id: String,
    pub name: String,
    pub category: String,
    pub logo: Option<String>,
    pub address: Address,
}

#[derive(Deserialize, Debug)]
pub struct Address {
    pub short_formatted: String,
    pub formatted: String,
    pub city: String,
    pub latitude: f64,
    pub longitude: f64,
    pub address: String,
    pub region: String,
    pub country: String,
    pub postcode: String,
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
        name = "Creating a transaction",
        skip(self, tx_fc),
        fields(tx_id = %tx_fc.id, acc_id = %tx_fc.account_id)
    )]
    async fn create_transaction(&self, tx_fc: &Transaction) -> Result<(), Error> {
        let db = self.pool.db();

        // return if the transaction already exists
        info!("Checking if Transaction already exists: {}", tx_fc.id);
        let existing_transaction = sqlx::query!(
            r"
                SELECT id
                FROM transactions
                WHERE id = $1
            ",
            tx_fc.id,
        )
        .fetch_optional(db)
        .await?;

        if existing_transaction.is_some() {
            info!("Skipped existing Transaction: {}", tx_fc.id);
            return Err(Error::Duplicate("Transaction already exists".to_string()));
        }

        // insert the merchant if there is one and it doesn't already exist
        info!("Checking if transaction has merchant: {:?}", tx_fc);
        let merchant_id = tx_fc.merchant.as_ref().map_or(None, |m| Some(m.id.clone()));

        if merchant_id.is_some() {
            let merchant_id = merchant_id.clone().unwrap();
            info!(
                "Has merchant. Checking if Merchant exists in db: {}",
                merchant_id
            );

            match sqlx::query!(
                r"
                    SELECT id
                    FROM merchants
                    WHERE id = $1
                ",
                merchant_id,
            )
            .fetch_optional(db)
            .await
            {
                Ok(Some(_)) => (),
                Ok(None) => {
                    info!("Inserting Merchant : {:?}", merchant_id);
                    // insert the account
                    match sqlx::query!(
                        r"
                            INSERT INTO merchants (
                                id,
                                name,
                                category
                            )
                            VALUES ($1, $2, $3)
                        ",
                        tx_fc.merchant.as_ref().unwrap().id,
                        tx_fc.merchant.as_ref().unwrap().name,
                        tx_fc.merchant.as_ref().unwrap().category,
                    )
                    .execute(db)
                    .await
                    {
                        Ok(_) => {
                            info!("Created merchant: {:?}", merchant_id);
                            ()
                        }
                        Err(e) => {
                            error!("Failed to create merchant: {:?}", merchant_id);
                            return Err(Error::DbError(e.to_string()));
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to check if merchant exists: {:?}", merchant_id);
                    return Err(Error::DbError(e.to_string()));
                }
            }
        }

        // insert the transaction

        info!("Inserting transaction : {}", tx_fc.id);

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
                    merchant_id.unwrap_or("None".to_string(),)
                );
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
