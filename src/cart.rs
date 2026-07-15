//! Minimal client for the cart service. The public cart UI lives in the cart
//! service itself; the store only reads the live item count for its navbar
//! badge, server-side, using the shared guest-cart cookie.

mod cart_detail;
mod cart_error;
mod cart_line;
mod cart_line_detail;
pub use cart_detail::CartDetail;
pub use cart_error::CartError;
pub(crate) use cart_line::CartLine;
pub(crate) use cart_line_detail::CartLineDetail;

use crate::config;

fn base_url() -> Result<String, CartError> {
    config::cart_base_url().ok_or(CartError::NotConfigured)
}

/// Fetch a cart by id. Returns `Ok(None)` when the cart no longer exists.
pub async fn get_cart(cart_id: &str) -> Result<Option<CartDetail>, CartError> {
    let url = format!("{}carts/{cart_id}", base_url()?);
    let response =
        sigma_pg::clients::http::with_internal_auth(sigma_pg::clients::http::client().get(url))
            .send()
            .await?;
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
