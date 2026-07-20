use std::sync::Arc;

pub use sigma_pg::clients::catalog::{CatalogError, CatalogSku, sku_by_id, validate_sku_id};
use sigma_pg::clients::http;

/// Pull all SKUs from the catalog service. sigma-pg TTL-caches the list per
/// process (30s, keyed by base URL), so hot paths can call this freely.
pub async fn fetch_skus() -> Result<Arc<Vec<CatalogSku>>, CatalogError> {
    sigma_pg::clients::catalog::fetch_skus(crate::config::catalog_base_url().as_deref()).await
}

/// Fetch a single SKU by id (`GET /skus/{id}`). Returns `Ok(None)` when the
/// SKU does not exist.
///
/// Local helper: sigma-pg's catalog client only offers the full-list
/// `fetch_skus` (API gap).
pub async fn fetch_sku(id: &str) -> Result<Option<CatalogSku>, CatalogError> {
    let base = crate::config::catalog_base_url().ok_or(CatalogError::NotConfigured)?;
    let url = format!("{base}skus/{id}");
    let response = http::with_internal_auth(http::client().get(url))
        .send()
        .await?;
    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return Ok(None);
    }
    let response = http::ensure_success(response)
        .await
        .map_err(CatalogError::Request)?;
    Ok(Some(response.json().await?))
}

/// Fail-closed SKU validation for mutations when catalog integration is configured.
pub async fn require_active_sku(sku_id: &str) -> Result<(), CatalogError> {
    if !crate::config::catalog_configured() {
        return Ok(());
    }
    let skus = fetch_skus().await?;
    validate_sku_id(&skus, sku_id.trim())
}
