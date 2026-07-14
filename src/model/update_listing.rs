//! [`UpdateListing`].

#[allow(unused_imports)]
use super::*;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateListing {
    pub sku_id: String,
    pub price_cents: Option<u64>,
    pub featured: bool,
    pub visible: bool,
    pub sort_order: u32,
}
