//! [`StorefrontItem`].

#[allow(unused_imports)]
use super::*;
use crate::catalog::CatalogSku;
use crate::model::Listing;

#[derive(serde::Serialize)]
pub(crate) struct StorefrontItem {
    pub(crate) listing: Listing,
    pub(crate) sku: Option<CatalogSku>,
}
