//! [`ListingForm`].

#[allow(unused_imports)]
use super::*;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct ListingForm {
    pub sku_id: String,
    pub price: String,
    pub featured: Option<String>,
    pub visible: Option<String>,
    pub sort_order: String,
}
impl ListingForm {
    /// Validate the form into a create request.
    pub fn into_create(self) -> Result<CreateListing, String> {
        Ok(CreateListing {
            sku_id: self.sku_id,
            price_cents: parse_price_cents(&self.price)?,
            featured: Some(self.featured.is_some()),
            visible: Some(self.visible.is_some()),
            sort_order: parse_sort_order(&self.sort_order)?,
        })
    }

    /// Validate the form into an update request.
    pub fn into_update(self) -> Result<UpdateListing, String> {
        Ok(UpdateListing {
            sku_id: self.sku_id,
            price_cents: parse_price_cents(&self.price)?,
            featured: self.featured.is_some(),
            visible: self.visible.is_some(),
            sort_order: parse_sort_order(&self.sort_order)?.unwrap_or(0),
        })
    }
}
