//! [`StoreError`].

#[allow(unused_imports)]
use super::*;
use thiserror::Error;

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
