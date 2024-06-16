use sqlx::{Pool, Sqlite};
use tracing_log::log::{error, info};

use crate::error::AppErrors as Error;

use super::DatabasePool;

#[derive(Debug, Default)]
pub struct Category {
    pub id: String,
    pub name: String,
}

// -- Services -------------------------------------------------------------------------
pub trait Service {
    async fn save_category(&self, category: &Category) -> Result<(), Error>;
}

#[derive(Debug, Clone)]
pub struct SqliteCategoryService {
    pub(crate) pool: DatabasePool,
}

impl SqliteCategoryService {
    #[must_use]
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }
}

// -- Service Implementations ----------------------------------------------------------

impl Service for SqliteCategoryService {
    #[tracing::instrument(name = "Save category", skip(self, category_fc))]
    async fn save_category(&self, category_fc: &Category) -> Result<(), Error> {
        let db = self.pool.db();

        if is_duplicate_category(db, &category_fc.id).await? {
            info!("Category exists. Skipping");
            return Err(Error::Duplicate("Category already exists".to_string()));
        }

        match sqlx::query!(
            r"
                INSERT INTO categories (id, name)
                VALUES ($1, $2)
            ",
            category_fc.id,
            category_fc.name,
        )
        .execute(db)
        .await
        {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("Failed to save category: {:?}", e);
                Err(Error::DbError("Failed to save category".to_string()))
            }
        }
    }
}

// Check if a category is a duplicate
async fn is_duplicate_category(db: &Pool<Sqlite>, category_id: &str) -> Result<bool, Error> {
    let existing_category = sqlx::query!(
        r"
            SELECT id
            FROM categories
            WHERE id = $1
        ",
        category_id,
    )
    .fetch_optional(db)
    .await?;

    Ok(existing_category.is_some())
}

// -- Tests ----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use crate::tests::test::test_db;

    use super::*;

    #[tokio::test]
    async fn save_category() {
        // Arrange
        let (pool, _tmp) = test_db().await;
        let service = SqliteCategoryService::new(pool);
        let category = Category::default();

        // Act
        let result = service.save_category(&category).await;

        // Assert
        assert!(result.is_ok());
    }
}
