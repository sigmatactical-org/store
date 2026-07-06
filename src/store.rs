use sqlx::PgPool;
use thiserror::Error;

use crate::model::{CreateListing, Listing, UpdateListing};

const SCHEMA: &str = "store";

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

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
struct Database {
    listings: Vec<Listing>,
}

#[derive(Debug, Clone)]
pub struct ListingsStore {
    pool: PgPool,
    db: Database,
}

impl ListingsStore {
    /// Connect to PostgreSQL and load the store snapshot.
    pub async fn connect() -> Result<Self, StoreError> {
        let pool = sigma_pg::connect().await?;
        let db: Database = sigma_pg::load_document(&pool, SCHEMA).await?;
        Ok(Self { pool, db })
    }

    /// Reset the store snapshot (tests only).
    #[cfg(test)]
    pub async fn connect_empty() -> Result<Self, StoreError> {
        let pool = sigma_pg::connect().await?;
        let db = Database::default();
        sigma_pg::save_document(&pool, SCHEMA, &db).await?;
        Ok(Self { pool, db })
    }

    async fn persist(&self) -> Result<(), StoreError> {
        sigma_pg::save_document(&self.pool, SCHEMA, &self.db).await?;
        Ok(())
    }

    #[must_use]
    pub fn list(&self) -> Vec<Listing> {
        let mut listings = self.db.listings.clone();
        listings.sort_by(|a, b| {
            a.sort_order
                .cmp(&b.sort_order)
                .then_with(|| a.sku_id.cmp(&b.sku_id))
        });
        listings
    }

    #[must_use]
    pub fn get(&self, id: &str) -> Option<Listing> {
        self.db.listings.iter().find(|l| l.id == id).cloned()
    }

    pub async fn create(&mut self, input: CreateListing) -> Result<Listing, StoreError> {
        self.validate_sku_id(&input.sku_id, None)?;
        let listing = Listing::new(input);
        self.db.listings.push(listing.clone());
        self.persist().await?;
        Ok(listing)
    }

    pub async fn update(&mut self, id: &str, input: UpdateListing) -> Result<Listing, StoreError> {
        self.validate_sku_id(&input.sku_id, Some(id))?;
        let listing = self
            .db
            .listings
            .iter_mut()
            .find(|l| l.id == id)
            .ok_or(StoreError::NotFound)?;
        listing.apply_update(input);
        let updated = listing.clone();
        self.persist().await?;
        Ok(updated)
    }

    pub async fn delete(&mut self, id: &str) -> Result<(), StoreError> {
        let index = self
            .db
            .listings
            .iter()
            .position(|l| l.id == id)
            .ok_or(StoreError::NotFound)?;
        self.db.listings.remove(index);
        self.persist().await
    }

    fn validate_sku_id(&self, sku_id: &str, except_id: Option<&str>) -> Result<(), StoreError> {
        if sku_id.trim().is_empty() {
            return Err(StoreError::SkuIdRequired);
        }
        let normalized = sku_id.trim();
        if self
            .db
            .listings
            .iter()
            .any(|l| except_id != Some(l.id.as_str()) && l.sku_id == normalized)
        {
            return Err(StoreError::DuplicateSkuId);
        }
        Ok(())
    }
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
        let mut store = test_store().await;
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
        let mut store = test_store().await;
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
