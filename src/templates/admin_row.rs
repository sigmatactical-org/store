//! [`AdminRow`].

/// One rendered table row.
pub struct AdminRow {
    pub id: String,
    pub sku_code: String,
    pub name: String,
    pub price_display: String,
    pub visible_label: &'static str,
    pub featured_label: &'static str,
    pub sort_order: u32,
    pub updated_at: String,
    pub missing_catalog: bool,
}
