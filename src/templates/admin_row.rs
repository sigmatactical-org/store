//! [`AdminRow`].

#[allow(unused_imports)]
use super::*;
use crate::model::Listing;

/// One rendered table row.
pub struct AdminRow {
    pub listing: Listing,
    pub sku_code: String,
    pub name: String,
    pub price_display: String,
    pub visible_label: String,
    pub featured_label: String,
    pub missing_catalog: bool,
}
