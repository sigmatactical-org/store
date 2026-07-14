//! [`AdminTemplate`].

#[allow(unused_imports)]
use super::*;
use crate::identity::IdentityUser;
use askama::Template;
use sigma_theme::nav::SiteHeader;

/// Internal admin dashboard: listing management + identity users + config status.
#[derive(Template)]
#[template(path = "admin.html")]
pub(crate) struct AdminTemplate {
    pub(crate) admin_rows: Vec<AdminRow>,
    pub(crate) catalog_configured: bool,
    pub(crate) catalog_error: Option<String>,
    pub(crate) identity_users: Vec<IdentityUser>,
    pub(crate) identity_configured: bool,
    pub(crate) identity_error: Option<String>,
    pub(crate) message: Option<String>,
    pub(crate) site_header: SiteHeader,
    pub(crate) site_nav: String,
    pub(crate) copyright_years: String,
}
