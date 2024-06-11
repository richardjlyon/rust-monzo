//! Models for the merchant endpoint

use async_trait::async_trait;
use serde::Deserialize;
use sqlx::{Pool, Sqlite};
use tracing_log::log::{error, info};

use crate::error::AppError as Error;

use super::DatabasePool;

#[derive(Deserialize, Debug, Default)]
pub struct Merchant {
    pub id: String,
    pub name: String,
    pub category: String,
    // pub logo: Option<String>,
    // pub address: Address,
}

#[derive(Deserialize, Debug, Default)]
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
pub trait MerchantService {
    async fn create_merchant(&self, merchant_fc: &Merchant) -> Result<(), Error>;
    async fn get_merchant(&self, merchant_id: &str) -> Result<Merchant, Error>;
    async fn delete_all_merchants(&self) -> Result<(), Error>;
}

#[derive(Debug, Clone)]
pub struct SqliteMerchantService {
    pub(crate) pool: DatabasePool,
}

impl SqliteMerchantService {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }
}

// -- Service Implementations ----------------------------------------------------------

#[async_trait]
impl MerchantService for SqliteMerchantService {
    #[tracing::instrument(
        name = "Create merchant",
        skip(self, merchant_fc),
        fields(tx_id = %merchant_fc.id, merchant_id = %merchant_fc.id)
    )]
    async fn create_merchant(&self, merchant_fc: &Merchant) -> Result<(), Error> {
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
                Ok(())
            }
            Err(e) => {
                error!("Failed to create merchant: {:?}", merchant_fc.id);
                Err(Error::DbError(e.to_string()))
            }
        }
    }

    #[tracing::instrument(name = "Get merchant")]
    async fn get_merchant(&self, merchant_id: &str) -> Result<Merchant, Error> {
        let db = self.pool.db();

        match sqlx::query_as!(
            Merchant,
            r"
                SELECT id, name, category
                FROM merchants
                WHERE id = $1
            ",
            merchant_id,
        )
        .fetch_one(db)
        .await
        {
            Ok(merchant) => {
                info!("Got merchant id: {}", merchant.id);
                Ok(merchant)
            }
            Err(e) => {
                error!("Failed to get merchant: {}", e.to_string());
                Err(Error::DbError(e.to_string()))
            }
        }
    }

    #[tracing::instrument(name = "Delete all merchants")]
    async fn delete_all_merchants(&self) -> Result<(), Error> {
        let db = self.pool.db();

        match sqlx::query!("DELETE FROM merchants").execute(db).await {
            Ok(_) => {
                info!("Deleted all merchants");
                Ok(())
            }
            Err(e) => {
                error!("Failed to delete all merchants: {}", e.to_string());
                Err(Error::DbError(e.to_string()))
            }
        }
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
    async fn test_create_merchant() {
        // Arrange
        let (pool, _tmp) = test_db().await;
        let service = SqliteMerchantService::new(pool);
        let merchant = Merchant::default();

        // Act
        let result = service.create_merchant(&merchant).await;

        // Assert
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_merchant() {
        // Arrange
        let (pool, _tmp) = test_db().await;
        let service = SqliteMerchantService::new(pool);
        let merchant = Merchant::default();

        // Act
        service.create_merchant(&merchant).await.unwrap();
        let result = service.get_merchant(&merchant.id).await;

        // Assert
        assert!(result.is_ok());
        assert_eq!(result.unwrap().unwrap().id, merchant.id);
    }

    #[tokio::test]
    async fn test_delete_all_merchants() {
        // Arrange
        let (pool, _tmp) = test_db().await;
        let service = SqliteMerchantService::new(pool);
        let merchant = Merchant::default();

        //Act
        service.create_merchant(&merchant).await.unwrap();
        let result = service.delete_all_merchants().await;

        // Assert
        assert!(result.is_ok());
    }
}
