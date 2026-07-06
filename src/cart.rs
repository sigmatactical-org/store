//! Minimal client for the cart service. The public cart UI lives in the cart
//! service itself; the store only reads the live item count for its navbar
//! badge, server-side, using the shared guest-cart cookie.

use serde::Deserialize;
use thiserror::Error;

use crate::config;

#[derive(Debug, Error)]
pub enum CartError {
    #[error("cart integration is not configured (set STORE_CART_BASE_URL)")]
    NotConfigured,
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("cart request failed: {0}")]
    Request(String),
}

#[derive(Debug, Clone, Deserialize)]
struct CartLine {
    quantity: u32,
}

#[derive(Debug, Clone, Deserialize)]
struct CartLineDetail {
    line: CartLine,
}

/// Enriched cart returned by the cart service (`{ cart, user, lines }`).
#[derive(Debug, Clone, Deserialize)]
pub struct CartDetail {
    #[serde(default)]
    lines: Vec<CartLineDetail>,
}

impl CartDetail {
    /// Total number of items (summed line quantities) for the nav badge.
    #[must_use]
    pub fn item_count(&self) -> u32 {
        self.lines.iter().map(|l| l.line.quantity).sum()
    }
}

fn base_url() -> Result<String, CartError> {
    config::cart_base_url().ok_or(CartError::NotConfigured)
}

/// Fetch a cart by id. Returns `Ok(None)` when the cart no longer exists.
pub async fn get_cart(cart_id: &str) -> Result<Option<CartDetail>, CartError> {
    let url = format!("{}carts/{cart_id}", base_url()?);
    let response = reqwest::Client::new().get(url).send().await?;
    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return Ok(None);
    }
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(CartError::Request(format!("{status}: {body}")));
    }
    Ok(Some(response.json().await?))
}
