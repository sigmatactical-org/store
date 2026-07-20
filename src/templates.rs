mod admin_page_input;
mod admin_row;
mod admin_template;
mod catalog_sku_ref;
mod form_template;
mod form_values;
mod product_detail;
mod product_template;
mod storefront_row;
mod storefront_template;
pub use admin_page_input::AdminPageInput;
pub use admin_row::AdminRow;
pub(crate) use admin_template::AdminTemplate;
pub use catalog_sku_ref::CatalogSkuRef;
pub(crate) use form_template::FormTemplate;
pub use form_values::FormValues;
pub use product_detail::ProductDetail;
pub(crate) use product_template::ProductTemplate;
pub use storefront_row::StorefrontRow;
pub(crate) use storefront_template::StorefrontTemplate;

use std::collections::HashMap;

use askama::Template;

use crate::catalog::CatalogSku;
use crate::config;
use crate::model::{Listing, format_price_cents, price_cents_to_form};
use sigma_theme::copyright_years;
use sigma_theme::nav::{Breadcrumb, SiteHeader, SiteMenuSection, site_menu};
use sigma_theme::site_nav::{AppSiteNav, render_app_site_nav};

fn page_header() -> SiteHeader {
    SiteHeader::new("Store").with_menu(site_menu(Some(SiteMenuSection::Store)))
}

fn site_nav(return_path: &str, cart_count: u32) -> Result<String, askama::Error> {
    render_app_site_nav(&AppSiteNav {
        identity_base: &config::identity_public_base_url(),
        app_base: &config::store_public_base_url(),
        contact_base: &config::contact_public_base_url(),
        cart_url: &config::cart_public_base_url(),
        cart_count,
        return_path,
        show_cart: true,
        show_contact_us: false,
        leading_html: "",
    })
}

const EXCERPT_MAX_LEN: usize = 220;

/// Timestamp rendering for the admin table (the model keeps `DateTime<Utc>`).
fn format_timestamp(timestamp: chrono::DateTime<chrono::Utc>) -> String {
    timestamp.to_rfc3339()
}

fn catalog_sku_refs(skus: &[CatalogSku]) -> Vec<CatalogSkuRef> {
    skus.iter()
        .filter(|s| s.active)
        .map(|s| CatalogSkuRef {
            id: s.id.clone(),
            sku_code: s.sku_code.clone(),
            name: s.name.clone(),
        })
        .collect()
}

/// Index SKUs by id so row building is O(n + m) instead of a scan per listing.
fn skus_by_id(skus: &[CatalogSku]) -> HashMap<&str, &CatalogSku> {
    skus.iter().map(|s| (s.id.as_str(), s)).collect()
}

/// First paragraph (or a truncated slice) of a description, for grid cards.
fn excerpt(description: &str) -> String {
    let first_paragraph = description
        .split("\n\n")
        .next()
        .unwrap_or(description)
        .trim();
    if first_paragraph.chars().count() <= EXCERPT_MAX_LEN {
        return first_paragraph.to_string();
    }
    let truncated: String = first_paragraph.chars().take(EXCERPT_MAX_LEN).collect();
    let truncated = truncated
        .rsplit_once(' ')
        .map(|(head, _)| head)
        .unwrap_or(&truncated);
    format!("{truncated}…")
}

/// Split a description into paragraphs on blank lines, for the product page.
fn description_paragraphs(description: &str) -> Vec<String> {
    description
        .split("\n\n")
        .map(str::trim)
        .filter(|p| !p.is_empty())
        .map(str::to_string)
        .collect()
}

fn storefront_rows(listings: &[Listing], skus: &[CatalogSku]) -> Vec<StorefrontRow> {
    let by_id = skus_by_id(skus);
    listings
        .iter()
        .filter(|l| l.visible)
        .filter_map(|listing| {
            let sku = by_id.get(listing.sku_id.as_str())?;
            if !sku.active {
                return None;
            }
            Some(StorefrontRow {
                product_path: crate::product_url::path(&sku.sku_code),
                name: sku.name.clone(),
                excerpt: sku.description.as_deref().map(excerpt),
                category: sku.category.clone(),
                price_display: format_price_cents(listing.price_cents),
                featured: listing.featured,
            })
        })
        .collect()
}

fn admin_rows(listings: &[Listing], skus: &[CatalogSku]) -> Vec<AdminRow> {
    let by_id = skus_by_id(skus);
    listings
        .iter()
        .map(|listing| {
            let sku = by_id.get(listing.sku_id.as_str());
            let (sku_code, name) = match sku {
                Some(s) => (s.sku_code.clone(), s.name.clone()),
                None => (listing.sku_id.clone(), "—".to_string()),
            };
            AdminRow {
                id: listing.id.clone(),
                sku_code,
                name,
                price_display: format_price_cents(listing.price_cents),
                visible_label: if listing.visible { "Yes" } else { "No" },
                featured_label: if listing.featured { "Yes" } else { "No" },
                sort_order: listing.sort_order,
                updated_at: format_timestamp(listing.updated_at),
                missing_catalog: sku.is_none(),
            }
        })
        .collect()
}

fn values_from_listing(listing: &Listing) -> FormValues {
    FormValues {
        sku_id: listing.sku_id.clone(),
        price: price_cents_to_form(listing.price_cents),
        featured: listing.featured,
        visible: listing.visible,
        sort_order: listing.sort_order.to_string(),
    }
}

fn default_form_values() -> FormValues {
    FormValues {
        sku_id: String::new(),
        price: String::new(),
        featured: false,
        visible: true,
        sort_order: String::new(),
    }
}

fn render_form(
    catalog_skus: &[CatalogSku],
    listing: Option<Listing>,
    error: Option<String>,
    values: FormValues,
) -> Result<String, askama::Error> {
    let return_path = listing
        .as_ref()
        .map(|entry| format!("/admin/listings/{}/edit", entry.id))
        .unwrap_or_else(|| "/admin/listings/new".to_string());
    let form_crumb = if listing.is_some() {
        "Edit listing"
    } else {
        "New listing"
    };
    FormTemplate {
        listing_id: listing.map(|entry| entry.id),
        sku_id: values.sku_id,
        price: values.price,
        featured: values.featured,
        visible: values.visible,
        sort_order: values.sort_order,
        catalog_skus: catalog_sku_refs(catalog_skus),
        error,
        site_header: page_header()
            .with_breadcrumb(Breadcrumb::link("/admin", "Admin"))
            .with_breadcrumb(Breadcrumb::current(form_crumb)),
        site_nav: site_nav(&return_path, 0)?,
        copyright_years: copyright_years(),
    }
    .render()
}

/// # Errors
///
/// Returns [`askama::Error`] when template rendering fails.
pub fn render_storefront_html(
    listings: &[Listing],
    catalog_skus: &[CatalogSku],
    cart_count: u32,
) -> Result<String, askama::Error> {
    StorefrontTemplate {
        storefront_items: storefront_rows(listings, catalog_skus),
        site_header: page_header(),
        site_nav: site_nav("/", cart_count)?,
        copyright_years: copyright_years(),
    }
    .render()
}

/// # Errors
///
/// Returns [`askama::Error`] when template rendering fails.
pub fn render_admin_html(input: AdminPageInput<'_>) -> Result<String, askama::Error> {
    AdminTemplate {
        admin_rows: admin_rows(input.listings, input.catalog_skus),
        catalog_configured: input.catalog_configured,
        catalog_error: input.catalog_error,
        identity_users: input.identity_users.to_vec(),
        identity_configured: input.identity_configured,
        identity_error: input.identity_error,
        message: input.message,
        site_header: page_header().with_breadcrumb(Breadcrumb::current("Admin")),
        site_nav: site_nav("/admin", 0)?,
        copyright_years: copyright_years(),
    }
    .render()
}

/// Resolve a visible, active product by its catalog SKU code.
#[must_use]
pub fn find_product(
    sku_code: &str,
    listings: &[Listing],
    skus: &[CatalogSku],
) -> Option<ProductDetail> {
    let needle = crate::product_url::slug(sku_code);
    let sku = skus
        .iter()
        .find(|s| s.active && crate::product_url::slug(&s.sku_code) == needle)?;
    let listing = listings.iter().find(|l| l.visible && l.sku_id == sku.id)?;
    Some(ProductDetail {
        sku_code: sku.sku_code.clone(),
        sku_id: sku.id.clone(),
        name: sku.name.clone(),
        category: sku.category.clone(),
        description: sku.description.clone(),
        price_display: format_price_cents(listing.price_cents),
    })
}

/// # Errors
///
/// Returns [`askama::Error`] when template rendering fails.
pub fn render_product_html(
    product: ProductDetail,
    cart_count: u32,
) -> Result<String, askama::Error> {
    let return_path = crate::product_url::path(&product.sku_code);
    let cart_url = config::cart_public_base_url();
    let cart_add_url = format!("{cart_url}add");
    let product_name = product.name.clone();
    ProductTemplate {
        sku_code: product.sku_code.clone(),
        sku_id: product.sku_id,
        name: product.name,
        category: product.category,
        description_paragraphs: product
            .description
            .as_deref()
            .map(description_paragraphs)
            .unwrap_or_default(),
        price_display: product.price_display,
        details_url: config::product_details_url(&product.sku_code),
        site_header: page_header()
            .with_breadcrumb(Breadcrumb::link("/", "Store"))
            .with_breadcrumb(Breadcrumb::current(product_name)),
        site_nav: site_nav(&return_path, cart_count)?,
        cart_add_url,
        copyright_years: copyright_years(),
    }
    .render()
}

/// # Errors
///
/// Returns [`askama::Error`] when template rendering fails.
pub fn render_form_html(
    catalog_skus: &[CatalogSku],
    listing: Option<Listing>,
    error: Option<String>,
) -> Result<String, askama::Error> {
    let values = listing
        .as_ref()
        .map(values_from_listing)
        .unwrap_or_else(default_form_values);
    render_form(catalog_skus, listing, error, values)
}

/// # Errors
///
/// Returns [`askama::Error`] when template rendering fails.
pub fn render_form_html_with_values(
    catalog_skus: &[CatalogSku],
    listing: Option<Listing>,
    error: Option<String>,
    values: FormValues,
) -> Result<String, askama::Error> {
    render_form(catalog_skus, listing, error, values)
}
