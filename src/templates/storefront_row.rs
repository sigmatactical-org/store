//! [`StorefrontRow`].

#[allow(unused_imports)]
use super::*;

/// One rendered table row.
pub struct StorefrontRow {
    pub product_path: String,
    pub name: String,
    pub excerpt: Option<String>,
    pub category: Option<String>,
    pub price_display: String,
    pub featured: bool,
}
