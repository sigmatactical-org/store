//! [`AdminPageInput`].

#[allow(unused_imports)]
use super::*;
use crate::catalog::CatalogSku;
use crate::identity::IdentityUser;
use crate::model::Listing;

/// Inputs for rendering the admin dashboard page.
pub struct AdminPageInput<'a> {
    pub listings: Vec<Listing>,
    pub catalog_skus: &'a [CatalogSku],
    pub catalog_configured: bool,
    pub catalog_error: Option<String>,
    pub identity_users: &'a [IdentityUser],
    pub identity_configured: bool,
    pub identity_error: Option<String>,
    pub message: Option<String>,
}
