use askama::Template;

use crate::catalog::CatalogSku;
use crate::config;
use crate::model::{Listing, RealmUser, format_price_cents, price_cents_to_form};
use sigma_cart_nav::render_cart_nav;
use sigma_identity_nav::{auth_links, render_auth_nav};
use sigma_theme::copyright_years;

/// Public storefront home page: visible, catalog-backed listings only.
#[derive(Template)]
#[template(path = "index.html")]
struct StorefrontTemplate {
    storefront_items: Vec<StorefrontRow>,
    cart_nav: String,
    auth_nav: String,
    contact_us_url: String,
    copyright_years: String,
}

/// Internal admin dashboard: listing management + identity users + config status.
#[derive(Template)]
#[template(path = "admin.html")]
struct AdminTemplate {
    admin_rows: Vec<AdminRow>,
    catalog_configured: bool,
    catalog_error: Option<String>,
    identity_users: Vec<RealmUser>,
    identity_configured: bool,
    identity_error: Option<String>,
    message: Option<String>,
    copyright_years: String,
}

/// Public product detail page for a single storefront item.
#[derive(Template)]
#[template(path = "product.html")]
struct ProductTemplate {
    sku_code: String,
    sku_id: String,
    name: String,
    category: Option<String>,
    description_paragraphs: Vec<String>,
    price_display: String,
    details_url: Option<String>,
    cart_nav: String,
    cart_add_url: String,
    auth_nav: String,
    contact_us_url: String,
    copyright_years: String,
}

#[derive(Template)]
#[template(path = "form.html")]
struct FormTemplate {
    listing: Option<Listing>,
    sku_id: String,
    price: String,
    featured: bool,
    visible: bool,
    sort_order: String,
    catalog_skus: Vec<CatalogSkuRef>,
    error: Option<String>,
    copyright_years: String,
}

pub struct StorefrontRow {
    pub sku_code: String,
    pub name: String,
    pub excerpt: Option<String>,
    pub category: Option<String>,
    pub price_display: String,
    pub featured: bool,
}

pub struct AdminRow {
    pub listing: Listing,
    pub sku_code: String,
    pub name: String,
    pub price_display: String,
    pub visible_label: String,
    pub featured_label: String,
    pub missing_catalog: bool,
}

pub struct CatalogSkuRef {
    pub id: String,
    pub sku_code: String,
    pub name: String,
}

pub struct FormValues {
    pub sku_id: String,
    pub price: String,
    pub featured: bool,
    pub visible: bool,
    pub sort_order: String,
}

const EXCERPT_MAX_LEN: usize = 220;

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

fn resolve_sku<'a>(skus: &'a [CatalogSku], sku_id: &str) -> Option<&'a CatalogSku> {
    skus.iter().find(|s| s.id == sku_id)
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
    listings
        .iter()
        .filter(|l| l.visible)
        .filter_map(|listing| {
            let sku = resolve_sku(skus, &listing.sku_id)?;
            if !sku.active {
                return None;
            }
            Some(StorefrontRow {
                sku_code: sku.sku_code.clone(),
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
    listings
        .iter()
        .map(|listing| {
            let sku = resolve_sku(skus, &listing.sku_id);
            let (sku_code, name) = match sku {
                Some(s) => (s.sku_code.clone(), s.name.clone()),
                None => (listing.sku_id.clone(), "—".to_string()),
            };
            AdminRow {
                listing: listing.clone(),
                sku_code,
                name,
                price_display: format_price_cents(listing.price_cents),
                visible_label: if listing.visible {
                    "Yes".to_string()
                } else {
                    "No".to_string()
                },
                featured_label: if listing.featured {
                    "Yes".to_string()
                } else {
                    "No".to_string()
                },
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
    FormTemplate {
        listing,
        sku_id: values.sku_id,
        price: values.price,
        featured: values.featured,
        visible: values.visible,
        sort_order: values.sort_order,
        catalog_skus: catalog_sku_refs(catalog_skus),
        error,
        copyright_years: copyright_years(),
    }
    .render()
}

/// # Errors
///
/// Returns [`askama::Error`] when template rendering fails.
pub fn render_storefront_html(
    listings: Vec<Listing>,
    catalog_skus: &[CatalogSku],
    cart_count: u32,
) -> Result<String, askama::Error> {
    let links = auth_links(
        &config::identity_public_base_url(),
        &config::store_public_base_url(),
        &config::contact_public_base_url(),
        "/",
    );
    StorefrontTemplate {
        storefront_items: storefront_rows(&listings, catalog_skus),
        cart_nav: render_cart_nav(&config::cart_public_base_url(), cart_count)?,
        auth_nav: render_auth_nav(&links)?,
        contact_us_url: links.contact_us_url,
        copyright_years: copyright_years(),
    }
    .render()
}

/// Inputs for rendering the admin dashboard page.
pub struct AdminPageInput<'a> {
    pub listings: Vec<Listing>,
    pub catalog_skus: &'a [CatalogSku],
    pub catalog_configured: bool,
    pub catalog_error: Option<String>,
    pub identity_users: &'a [RealmUser],
    pub identity_configured: bool,
    pub identity_error: Option<String>,
    pub message: Option<String>,
}

/// # Errors
///
/// Returns [`askama::Error`] when template rendering fails.
pub fn render_admin_html(input: AdminPageInput<'_>) -> Result<String, askama::Error> {
    AdminTemplate {
        admin_rows: admin_rows(&input.listings, input.catalog_skus),
        catalog_configured: input.catalog_configured,
        catalog_error: input.catalog_error,
        identity_users: input.identity_users.to_vec(),
        identity_configured: input.identity_configured,
        identity_error: input.identity_error,
        message: input.message,
        copyright_years: copyright_years(),
    }
    .render()
}

/// A single storefront item resolved for the product detail page.
pub struct ProductDetail {
    pub sku_code: String,
    pub sku_id: String,
    pub name: String,
    pub category: Option<String>,
    pub description: Option<String>,
    pub price_display: String,
}

/// Resolve a visible, active product by its catalog SKU code.
#[must_use]
pub fn find_product(
    sku_code: &str,
    listings: &[Listing],
    skus: &[CatalogSku],
) -> Option<ProductDetail> {
    let sku = skus.iter().find(|s| s.active && s.sku_code == sku_code)?;
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
    let return_path = format!("/products/{}", product.sku_code);
    let links = auth_links(
        &config::identity_public_base_url(),
        &config::store_public_base_url(),
        &config::contact_public_base_url(),
        &return_path,
    );
    let cart_url = config::cart_public_base_url();
    let cart_add_url = format!("{cart_url}add");
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
        cart_nav: render_cart_nav(&cart_url, cart_count)?,
        cart_add_url,
        auth_nav: render_auth_nav(&links)?,
        contact_us_url: links.contact_us_url,
        copyright_years: copyright_years(),
    }
    .render()
}

/// # Errors
///
/// Returns [`askama::Error`] when template rendering fails.
pub fn render_form_html(
    _listings: Vec<Listing>,
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
    _listings: Vec<Listing>,
    catalog_skus: &[CatalogSku],
    listing: Option<Listing>,
    error: Option<String>,
    values: FormValues,
) -> Result<String, askama::Error> {
    render_form(catalog_skus, listing, error, values)
}
