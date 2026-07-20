//! [`CreateListing`].

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct CreateListing {
    pub sku_id: String,
    pub price_cents: Option<u64>,
    #[serde(default)]
    pub featured: Option<bool>,
    #[serde(default)]
    pub visible: Option<bool>,
    #[serde(default)]
    pub sort_order: Option<u32>,
}
