//! [`ProductDetail`].

#[allow(unused_imports)]
use super::*;

/// A single storefront item resolved for the product detail page.
pub struct ProductDetail {
    pub sku_code: String,
    pub sku_id: String,
    pub name: String,
    pub category: Option<String>,
    pub description: Option<String>,
    pub price_display: String,
}
