//! [`ProductTemplate`].

#[allow(unused_imports)]
use super::*;
use askama::Template;
use sigma_theme::nav::SiteHeader;

/// Public product detail page for a single storefront item.
#[derive(Template)]
#[template(path = "product.html")]
pub(crate) struct ProductTemplate {
    pub(crate) sku_code: String,
    pub(crate) sku_id: String,
    pub(crate) name: String,
    pub(crate) category: Option<String>,
    pub(crate) description_paragraphs: Vec<String>,
    pub(crate) price_display: String,
    pub(crate) details_url: Option<String>,
    pub(crate) site_header: SiteHeader,
    pub(crate) site_nav: String,
    pub(crate) cart_add_url: String,
    pub(crate) copyright_years: String,
}
