//! [`FormTemplate`].

use askama::Template;
use sigma_theme::nav::SiteHeader;

use super::CatalogSkuRef;

#[derive(Template)]
#[template(path = "form.html")]
pub(crate) struct FormTemplate {
    /// `Some` when editing an existing listing (drives the form action).
    pub(crate) listing_id: Option<String>,
    pub(crate) sku_id: String,
    pub(crate) price: String,
    pub(crate) featured: bool,
    pub(crate) visible: bool,
    pub(crate) sort_order: String,
    pub(crate) catalog_skus: Vec<CatalogSkuRef>,
    pub(crate) error: Option<String>,
    pub(crate) site_header: SiteHeader,
    pub(crate) site_nav: String,
    pub(crate) copyright_years: String,
}
