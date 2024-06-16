//! Models for the pot endpoint

use async_trait::async_trait;
use serde::Deserialize;
use sqlx::{Pool, Sqlite};
use tracing_log::log::{error, info};

use crate::error::AppErrors as Error;

use super::DatabasePool;

#[derive(Deserialize, Debug)]
pub struct Pots {
    pub pots: Vec<Pot>,
}

#[derive(Deserialize, Debug, Default)]
pub struct Pot {
    pub id: String,
    pub name: String,
    pub balance: i64,
    pub currency: String,
    pub deleted: bool,
    #[serde(rename = "type")]
    pub pot_type: String,
}

// -- Services -------------------------------------------------------------------------

#[async_trait]
pub trait Service {
    async fn save_pot(&self, pot_fc: &Pot) -> Result<(), Error>;
    async fn read_pots(&self) -> Result<Vec<Pot>, Error>;
    async fn read_pot(&self, pot_id: &str) -> Result<Option<Pot>, Error>;
}

#[derive(Debug, Clone)]
pub struct SqlitePotService {
    pub(crate) pool: DatabasePool,
}

impl SqlitePotService {
    #[must_use]
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }
}

// -- Service Implementations ----------------------------------------------------------

#[async_trait]
impl Service for SqlitePotService {
    #[tracing::instrument(
        name = "Save pot",
        skip(self, pot_fc),
        fields(tx_id = %pot_fc.id, merchant_id = %pot_fc.id)
    )]
    async fn save_pot(&self, pot_fc: &Pot) -> Result<(), Error> {
        let db = self.pool.db();

        if is_duplicate_pot(db, &pot_fc.id).await? {
            info!("Pot exists. Skipping");
            return Err(Error::Duplicate("Pot already exists".to_string()));
        }

        match sqlx::query!(
            r"
                INSERT INTO pots (
                    id,
                    name,
                    balance,
                    currency,
                    deleted,
                    pot_type
                )
                VALUES ($1, $2, $3, $4, $5, $6)
            ",
            pot_fc.id,
            pot_fc.name,
            pot_fc.balance,
            pot_fc.currency,
            pot_fc.deleted,
            pot_fc.pot_type,
        )
        .execute(db)
        .await
        {
            Ok(_) => {
                info!("Created pot: {:?}", pot_fc.id);
                Ok(())
            }
            Err(e) => {
                error!("Failed to create pot: {:?}", pot_fc.id);
                Err(Error::DbError(e.to_string()))
            }
        }
    }

    #[tracing::instrument(name = "Get pots")]
    async fn read_pots(&self) -> Result<Vec<Pot>, Error> {
        let db = self.pool.db();

        let pots = sqlx::query_as!(
            Pot,
            r"
                SELECT *
                FROM pots
            ",
        )
        .fetch_all(db)
        .await;

        match pots {
            Ok(pots) => Ok(pots),
            Err(e) => {
                error!("Failed to get pots: {:?}", e);
                Err(Error::DbError(e.to_string()))
            }
        }
    }

    #[tracing::instrument(name = "Get pot")]
    async fn read_pot(&self, pot_id: &str) -> Result<Option<Pot>, Error> {
        let db = self.pool.db();

        let pot = sqlx::query_as!(
            Pot,
            r"
                SELECT *
                FROM pots
                WHERE id = $1
            ",
            pot_id,
        )
        .fetch_optional(db)
        .await?;

        Ok(pot)
    }
}

// -- Utility functions ----------------------------------------------------------------

// Check if a merchant is a duplicate
async fn is_duplicate_pot(db: &Pool<Sqlite>, pot_id: &str) -> Result<bool, Error> {
    let existing_pot = sqlx::query!(
        r"
            SELECT id
            FROM pots
            WHERE id = $1
        ",
        pot_id,
    )
    .fetch_optional(db)
    .await?;

    Ok(existing_pot.is_some())
}

// -- Tests ---------------------------------------------------

#[cfg(test)]
mod tests {
    use crate::tests::test::test_db;

    use super::*;

    #[tokio::test]
    async fn create_pot() {
        // Arrange
        let (pool, _tmp) = test_db().await;
        let service = SqlitePotService::new(pool);
        let pot = Pot::default();

        // Act
        let result = service.save_pot(&pot).await;

        // Assert
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn read_pots() {
        // Arrange
        let (pool, _tmp) = test_db().await;
        let service = SqlitePotService::new(pool);

        // Act
        let result = service.read_pots().await;

        // Assert
        assert_eq!(result.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn read_pot() {
        // Arrange
        let (pool, _tmp) = test_db().await;
        let service = SqlitePotService::new(pool);
        let pot_id = "1".to_string();

        // Act
        let result = service.read_pot(&pot_id).await.unwrap().unwrap();

        // Assert
        assert_eq!(result.id, pot_id);
    }
}
