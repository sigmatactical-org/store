use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use thiserror::Error;

use crate::model::{CreateListing, Listing, UpdateListing};

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("listing not found")]
    NotFound,
    #[error("sku_id is required")]
    SkuIdRequired,
    #[error("listing for this catalog sku already exists")]
    DuplicateSkuId,
    #[error("catalog sku not found: {0}")]
    SkuNotInCatalog(String),
    #[error("database error: {0}")]
    Database(#[from] anyhow::Error),
    #[error("{0}")]
    InvalidInput(String),
}

impl From<sqlx::Error> for StoreError {
    fn from(err: sqlx::Error) -> Self {
        Self::Database(err.into())
    }
}

#[derive(Debug, Clone)]
pub struct ListingsStore {
    pool: PgPool,
}

impl ListingsStore {
    pub async fn connect() -> Result<Self, StoreError> {
        let pool = sigma_pg::connect_as("store").await?;
        Ok(Self { pool })
    }

    #[cfg(test)]
    pub async fn connect_empty() -> Result<Self, StoreError> {
        let store = Self::connect().await?;
        sqlx::query("TRUNCATE store.listings")
            .execute(&store.pool)
            .await?;
        Ok(store)
    }

    #[must_use]
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub async fn list(&self) -> Result<Vec<Listing>, StoreError> {
        let rows = sqlx::query(
            "SELECT id, sku_id, price_cents, featured, visible, sort_order, updated_at \
             FROM store.listings ORDER BY sort_order, sku_id",
        )
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(row_to_listing).collect()
    }

    pub async fn get(&self, id: &str) -> Result<Option<Listing>, StoreError> {
        let row = sqlx::query(
            "SELECT id, sku_id, price_cents, featured, visible, sort_order, updated_at \
             FROM store.listings WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        row.map(row_to_listing).transpose()
    }

    pub async fn create(&self, input: CreateListing) -> Result<Listing, StoreError> {
        self.validate_sku_id(&input.sku_id, None).await?;
        let listing = Listing::new(input);
        sqlx::query(
            "INSERT INTO store.listings (id, sku_id, price_cents, featured, visible, sort_order, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7)",
        )
        .bind(&listing.id)
        .bind(&listing.sku_id)
        .bind(listing.price_cents.map(|p| p as i64))
        .bind(listing.featured)
        .bind(listing.visible)
        .bind(listing.sort_order as i32)
        .bind(parse_ts(&listing.updated_at)?)
        .execute(&self.pool)
        .await?;
        Ok(listing)
    }

    pub async fn update(&self, id: &str, input: UpdateListing) -> Result<Listing, StoreError> {
        self.validate_sku_id(&input.sku_id, Some(id)).await?;
        let mut listing = self.get(id).await?.ok_or(StoreError::NotFound)?;
        listing.apply_update(input);
        sqlx::query(
            "UPDATE store.listings SET sku_id = $2, price_cents = $3, featured = $4, visible = $5, \
             sort_order = $6, updated_at = $7 WHERE id = $1",
        )
        .bind(id)
        .bind(&listing.sku_id)
        .bind(listing.price_cents.map(|p| p as i64))
        .bind(listing.featured)
        .bind(listing.visible)
        .bind(listing.sort_order as i32)
        .bind(parse_ts(&listing.updated_at)?)
        .execute(&self.pool)
        .await?;
        Ok(listing)
    }

    pub async fn delete(&self, id: &str) -> Result<(), StoreError> {
        let result = sqlx::query("DELETE FROM store.listings WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        if result.rows_affected() == 0 {
            return Err(StoreError::NotFound);
        }
        Ok(())
    }

    async fn validate_sku_id(
        &self,
        sku_id: &str,
        except_id: Option<&str>,
    ) -> Result<(), StoreError> {
        if sku_id.trim().is_empty() {
            return Err(StoreError::SkuIdRequired);
        }
        let normalized = sku_id.trim();
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(
                SELECT 1 FROM store.listings
                WHERE sku_id = $1
                  AND ($2::text IS NULL OR id <> $2)
             )",
        )
        .bind(normalized)
        .bind(except_id)
        .fetch_one(&self.pool)
        .await?;
        if exists {
            return Err(StoreError::DuplicateSkuId);
        }
        Ok(())
    }
}

fn row_to_listing(row: sqlx::postgres::PgRow) -> Result<Listing, StoreError> {
    let price_cents: Option<i64> = row.get("price_cents");
    Ok(Listing {
        id: row.get("id"),
        sku_id: row.get("sku_id"),
        price_cents: price_cents.map(|p| p as u64),
        featured: row.get("featured"),
        visible: row.get("visible"),
        sort_order: row.get::<i32, _>("sort_order") as u32,
        updated_at: row.get::<DateTime<Utc>, _>("updated_at").to_rfc3339(),
    })
}

fn parse_ts(value: &str) -> Result<DateTime<Utc>, StoreError> {
    value
        .parse::<DateTime<Utc>>()
        .map_err(|e| StoreError::InvalidInput(format!("invalid timestamp: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn test_store() -> ListingsStore {
        ListingsStore::connect_empty()
            .await
            .expect("PostgreSQL required for tests")
    }

    #[tokio::test]
    async fn create_listing() {
        let store = test_store().await;
        let listing = store
            .create(CreateListing {
                sku_id: "sku-abc".to_string(),
                price_cents: Some(1999),
                featured: Some(true),
                visible: Some(true),
                sort_order: Some(10),
            })
            .await
            .unwrap();
        assert_eq!(listing.sku_id, "sku-abc");
        assert_eq!(listing.price_cents, Some(1999));
    }

    #[tokio::test]
    async fn reject_duplicate_sku_id() {
        let store = test_store().await;
        store
            .create(CreateListing {
                sku_id: "sku-abc".to_string(),
                price_cents: None,
                featured: None,
                visible: None,
                sort_order: None,
            })
            .await
            .unwrap();
        let err = store
            .create(CreateListing {
                sku_id: "sku-abc".to_string(),
                price_cents: None,
                featured: None,
                visible: None,
                sort_order: None,
            })
            .await
            .unwrap_err();
        assert!(matches!(err, StoreError::DuplicateSkuId));
    }
}
