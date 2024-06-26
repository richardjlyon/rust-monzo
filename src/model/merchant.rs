//! Models for the merchant endpoint

use async_trait::async_trait;
use serde::Deserialize;
use sqlx::{Pool, Sqlite};
use tracing_log::log::{error, info};

use crate::error::AppErrors as Error;

use super::DatabasePool;

#[derive(Deserialize, Debug, Default, Clone)]
pub struct Merchant {
    pub id: String,
    pub name: String,
    pub category: String,
    // pub logo: Option<String>,
    // pub address: Address,
}

#[derive(Deserialize, Debug, Default, Clone)]
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

// -- Services -------------------------------------------------------------------------

#[async_trait]
pub trait Service {
    async fn save_merchant(&self, merchant_fc: &Merchant) -> Result<String, Error>;
    async fn get_merchant(&self, merchant_id: &str) -> Result<Option<Merchant>, Error>;
}

#[derive(Debug, Clone)]
pub struct SqliteMerchantService {
    pub(crate) pool: DatabasePool,
}

impl SqliteMerchantService {
    #[must_use]
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }
}

// -- Service Implementations ----------------------------------------------------------

#[async_trait]
impl Service for SqliteMerchantService {
    #[tracing::instrument(
        name = "Create merchant",
        skip(self, merchant_fc),
        fields(tx_id = %merchant_fc.id, merchant_id = %merchant_fc.id)
    )]
    /// Save a merchant to the database returning the merchant id
    ///
    /// # Errors
    /// Will return an error if the merchant already exists or create fails
    async fn save_merchant(&self, merchant_fc: &Merchant) -> Result<String, Error> {
        let db = self.pool.db();

        if is_duplicate_merchant(db, &merchant_fc.id).await? {
            info!("Merchant exists. Skipping");
            return Err(Error::Duplicate("Merchant already exists".to_string()));
        }

        match sqlx::query!(
            r"
                INSERT INTO merchants (
                    id,
                    name,
                    category
                )
                VALUES ($1, $2, $3)
            ",
            merchant_fc.id,
            merchant_fc.name,
            merchant_fc.category,
        )
        .execute(db)
        .await
        {
            Ok(_) => {
                info!("Created merchant: {:?}", merchant_fc.id);
                Ok(merchant_fc.id.clone())
            }
            Err(e) => {
                error!("Failed to create merchant: {:?}", merchant_fc.id);
                Err(Error::DbError(e.to_string()))
            }
        }
    }

    #[tracing::instrument(name = "Get merchant")]
    async fn get_merchant(&self, merchant_id: &str) -> Result<Option<Merchant>, Error> {
        let db = self.pool.db();

        let merchant = sqlx::query_as!(
            Merchant,
            r"
                SELECT *
                FROM merchants
                WHERE id = $1
            ",
            merchant_id,
        )
        .fetch_optional(db)
        .await?;

        Ok(merchant)
    }
}

// -- Utility functions ----------------------------------------------------------------

// Check if a merchant is a duplicate
async fn is_duplicate_merchant(db: &Pool<Sqlite>, merchant_id: &str) -> Result<bool, Error> {
    let existing_merchant = sqlx::query!(
        r"
            SELECT id
            FROM merchants
            WHERE id = $1
        ",
        merchant_id,
    )
    .fetch_optional(db)
    .await?;

    Ok(existing_merchant.is_some())
}

// -- Tests ----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::test::test_db;

    #[tokio::test]
    async fn create_merchant() {
        // Arrange
        let (pool, _tmp) = test_db().await;
        let service = SqliteMerchantService::new(pool);
        let merchant = Merchant::default();

        // Act
        let result = service.save_merchant(&merchant).await;

        // Assert
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn get_merchant() {
        // Arrange
        let (pool, _tmp) = test_db().await;
        let service = SqliteMerchantService::new(pool);
        let merchant = Merchant::default();

        // Act
        service.save_merchant(&merchant).await.unwrap();
        let result = service.get_merchant(&merchant.id).await;

        // Assert
        assert!(result.is_ok());
        assert_eq!(result.unwrap().unwrap().id, merchant.id);
    }
}
