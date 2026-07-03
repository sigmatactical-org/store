use std::path::{Path, PathBuf};

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
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
struct Database {
    listings: Vec<Listing>,
}

#[derive(Debug, Clone)]
pub struct ListingsStore {
    path: PathBuf,
    db: Database,
}

impl ListingsStore {
    /// Load or initialize the store database at `path`.
    pub fn load(path: impl AsRef<Path>) -> Result<Self, StoreError> {
        let path = path.as_ref().to_path_buf();
        let db = if path.exists() {
            let bytes = std::fs::read(&path)?;
            serde_json::from_slice(&bytes)?
        } else {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            Database::default()
        };
        Ok(Self { path, db })
    }

    fn save(&self) -> Result<(), StoreError> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let bytes = serde_json::to_vec_pretty(&self.db)?;
        std::fs::write(&self.path, bytes)?;
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

    pub fn create(&mut self, input: CreateListing) -> Result<Listing, StoreError> {
        self.validate_sku_id(&input.sku_id, None)?;
        let listing = Listing::new(input);
        self.db.listings.push(listing.clone());
        self.save()?;
        Ok(listing)
    }

    pub fn update(&mut self, id: &str, input: UpdateListing) -> Result<Listing, StoreError> {
        self.validate_sku_id(&input.sku_id, Some(id))?;
        let listing = self
            .db
            .listings
            .iter_mut()
            .find(|l| l.id == id)
            .ok_or(StoreError::NotFound)?;
        listing.apply_update(input);
        let updated = listing.clone();
        self.save()?;
        Ok(updated)
    }

    pub fn delete(&mut self, id: &str) -> Result<(), StoreError> {
        let index = self
            .db
            .listings
            .iter()
            .position(|l| l.id == id)
            .ok_or(StoreError::NotFound)?;
        self.db.listings.remove(index);
        self.save()
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
    use tempfile::TempDir;

    fn test_store() -> (ListingsStore, TempDir) {
        let dir = TempDir::new().unwrap();
        let store = ListingsStore::load(dir.path().join("store.json")).unwrap();
        (store, dir)
    }

    #[test]
    fn create_listing() {
        let (mut store, _dir) = test_store();
        let listing = store
            .create(CreateListing {
                sku_id: "sku-abc".to_string(),
                price_cents: Some(1999),
                featured: Some(true),
                visible: Some(true),
                sort_order: Some(10),
            })
            .unwrap();
        assert_eq!(listing.sku_id, "sku-abc");
        assert_eq!(listing.price_cents, Some(1999));
    }

    #[test]
    fn reject_duplicate_sku_id() {
        let (mut store, _dir) = test_store();
        store
            .create(CreateListing {
                sku_id: "sku-abc".to_string(),
                price_cents: None,
                featured: None,
                visible: None,
                sort_order: None,
            })
            .unwrap();
        let err = store
            .create(CreateListing {
                sku_id: "sku-abc".to_string(),
                price_cents: None,
                featured: None,
                visible: None,
                sort_order: None,
            })
            .unwrap_err();
        assert!(matches!(err, StoreError::DuplicateSkuId));
    }
}
