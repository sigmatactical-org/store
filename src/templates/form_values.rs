//! [`FormValues`].

#[allow(unused_imports)]
use super::*;

/// Prefilled field values for the edit/create form.
pub struct FormValues {
    pub sku_id: String,
    pub price: String,
    pub featured: bool,
    pub visible: bool,
    pub sort_order: String,
}
