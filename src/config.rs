use sigma_pg::clients::http::{env_url, normalize_base_url};

/// Read an optional base URL from `var` (trimmed, trailing slash normalized).
fn env_url_opt(var: &str) -> Option<String> {
    std::env::var(var)
        .ok()
        .filter(|s| !s.trim().is_empty())
        .map(|s| normalize_base_url(&s))
}

/// Base URL of the catalog service (e.g. `http://127.0.0.1:8081/`).
#[must_use]
pub fn catalog_base_url() -> Option<String> {
    env_url_opt("STORE_CATALOG_BASE_URL")
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
    env_url_opt("STORE_CART_BASE_URL")
}

/// Public base URL of the cart service, where the browser is sent to add items
/// and view the cart (e.g. `http://127.0.0.1:8084/`).
#[must_use]
pub fn cart_public_base_url() -> String {
    env_url("STORE_CART_PUBLIC_URL", "http://127.0.0.1:8084/")
}

/// Browser origin of the cart service for CSP `form-action` (no trailing slash).
#[must_use]
pub fn cart_public_origin() -> String {
    cart_public_base_url().trim_end_matches('/').to_string()
}

/// Public base URL of the contact service for the storefront contact form.
#[must_use]
pub fn contact_public_base_url() -> String {
    env_url("STORE_CONTACT_PUBLIC_URL", "http://127.0.0.1:8083/")
}

/// OIDC issuer URL for the identity provider (Keycloak realm URL).
#[must_use]
pub fn identity_issuer_url() -> Option<String> {
    std::env::var("STORE_IDENTITY_ISSUER_URL")
        .ok()
        .filter(|s| !s.trim().is_empty())
}

/// Service-account client id for Keycloak Admin API access.
#[must_use]
pub fn identity_client_id() -> Option<String> {
    std::env::var("STORE_IDENTITY_CLIENT_ID")
        .ok()
        .filter(|s| !s.trim().is_empty())
}

/// Service-account client secret for Keycloak Admin API access.
#[must_use]
pub fn identity_client_secret() -> Option<String> {
    std::env::var("STORE_IDENTITY_CLIENT_SECRET")
        .ok()
        .filter(|s| !s.trim().is_empty())
}

/// Whether identity integration is configured.
#[must_use]
pub fn identity_configured() -> bool {
    identity_issuer_url().is_some()
        && identity_client_id().is_some()
        && identity_client_secret().is_some()
}

/// Public base URL of the identity BFF (e.g. `http://127.0.0.1:3000/`).
#[must_use]
pub fn identity_public_base_url() -> String {
    env_url("STORE_IDENTITY_PUBLIC_URL", "http://127.0.0.1:3000/")
}

/// Browser origin of the identity BFF for CSP `connect-src` (no trailing slash).
#[must_use]
pub fn identity_public_origin() -> String {
    identity_public_base_url().trim_end_matches('/').to_string()
}

/// Canonical public URL of this store (e.g. `http://127.0.0.1:8082/`).
#[must_use]
pub fn store_public_base_url() -> String {
    env_url("STORE_PUBLIC_BASE_URL", "http://127.0.0.1:8082/")
}

/// Public base URL of the info service for product detail links.
#[must_use]
pub fn info_public_base_url() -> String {
    env_url("STORE_INFO_PUBLIC_URL", "http://127.0.0.1:8080/")
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
