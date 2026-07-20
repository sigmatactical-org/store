//! [`StorefrontTemplate`].

use askama::Template;
use sigma_theme::nav::SiteHeader;

use super::StorefrontRow;

/// Public storefront home page: visible, catalog-backed listings only.
#[derive(Template)]
#[template(path = "index.html")]
pub(crate) struct StorefrontTemplate {
    pub(crate) storefront_items: Vec<StorefrontRow>,
    pub(crate) site_header: SiteHeader,
    pub(crate) site_nav: String,
    pub(crate) copyright_years: String,
}
