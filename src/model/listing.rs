//! [`Listing`].

#[allow(unused_imports)]
use super::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Listing {
    pub id: String,
    pub sku_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price_cents: Option<u64>,
    pub featured: bool,
    pub visible: bool,
    pub sort_order: u32,
    pub updated_at: String,
}
impl Listing {
    /// New Listing from a create request.
    pub fn new(input: CreateListing) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            sku_id: input.sku_id.trim().to_string(),
            price_cents: input.price_cents,
            featured: input.featured.unwrap_or(false),
            visible: input.visible.unwrap_or(true),
            sort_order: input.sort_order.unwrap_or(0),
            updated_at: now,
        }
    }

    /// Apply a partial update in place.
    pub fn apply_update(&mut self, input: UpdateListing) {
        self.sku_id = input.sku_id.trim().to_string();
        self.price_cents = input.price_cents;
        self.featured = input.featured;
        self.visible = input.visible;
        self.sort_order = input.sort_order;
        self.updated_at = chrono::Utc::now().to_rfc3339();
    }
}
