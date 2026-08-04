//! Environment-driven configuration (service URLs, optional integrations).
//!
//! Required values are declared in the [`sigma_config::service!`] block and
//! checked by [`validate_with`] at startup; optional integrations return
//! `None` when they are not configured for this environment.

sigma_config::service! {
    prefix = "STORE";
    role = "store";
    urls {
        /// Public base URL of the cart service, where the browser is sent to add items
        /// and view the cart.
        cart_public_base_url = "CART_PUBLIC_URL" => "http://127.0.0.1:8084/";
        /// Public base URL of the contact service for the storefront contact form.
        contact_public_base_url = "CONTACT_PUBLIC_URL" => "http://127.0.0.1:8083/";
        /// Public base URL of the identity BFF.
        identity_public_base_url = "IDENTITY_PUBLIC_URL" => "http://127.0.0.1:3000/";
        /// Canonical public URL of this store.
        store_public_base_url = "PUBLIC_BASE_URL" => "http://127.0.0.1:8082/";
        /// Public base URL of the info service for product detail links.
        info_public_base_url = "INFO_PUBLIC_URL" => "http://127.0.0.1:8080/";
    }
}

/// Browser origin of the cart service for CSP `form-action` (no trailing slash).
#[must_use]
pub fn cart_public_origin() -> String {
    sigma_config::origin_of(&cart_public_base_url())
}

/// Browser origin of the identity BFF for CSP `connect-src` (no trailing slash).
#[must_use]
pub fn identity_public_origin() -> String {
    sigma_config::origin_of(&identity_public_base_url())
}

/// Base URL of the catalog service (e.g. `http://127.0.0.1:8081/`).
#[must_use]
pub fn catalog_base_url() -> Option<String> {
    SERVICE.opt_url("CATALOG_BASE_URL")
}

/// Whether catalog integration is configured.
#[must_use]
pub fn catalog_configured() -> bool {
    catalog_base_url().is_some()
}

/// Base URL of the cart service over the mesh, used server-side to read the
/// live item count for the navbar badge (e.g. `http://127.0.0.1:8084/`).
#[must_use]
pub fn cart_base_url() -> Option<String> {
    SERVICE.opt_url("CART_BASE_URL")
}

/// OIDC issuer URL for the identity provider (Keycloak realm URL).
#[must_use]
pub fn identity_issuer_url() -> Option<String> {
    SERVICE.opt_str("IDENTITY_ISSUER_URL")
}

/// Service-account client id for Keycloak Admin API access.
#[must_use]
pub fn identity_client_id() -> Option<String> {
    SERVICE.opt_str("IDENTITY_CLIENT_ID")
}

/// Service-account client secret for Keycloak Admin API access.
#[must_use]
pub fn identity_client_secret() -> Option<String> {
    SERVICE.opt_str("IDENTITY_CLIENT_SECRET")
}

/// Whether identity integration is configured.
#[must_use]
pub fn identity_configured() -> bool {
    identity_issuer_url().is_some()
        && identity_client_id().is_some()
        && identity_client_secret().is_some()
}

/// External details page URL for a storefront SKU, when available.
#[must_use]
pub fn product_details_url(sku_code: &str) -> Option<String> {
    if sku_code.eq_ignore_ascii_case("sigma-racer") {
        Some(format!(
            "{}/products/sigma-racer",
            info_public_base_url().trim_end_matches('/')
        ))
    } else {
        None
    }
}
