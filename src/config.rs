/// PostgreSQL connection URL (shared Sigma database).
#[must_use]
pub fn database_url() -> String {
    std::env::var("DATABASE_URL").unwrap_or_else(|_| sigma_pg::DEFAULT_DATABASE_URL.to_string())
}

/// Base URL of the catalog service (e.g. `http://127.0.0.1:8081/`).
#[must_use]
pub fn catalog_base_url() -> Option<String> {
    std::env::var("STORE_CATALOG_BASE_URL")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .map(|s| {
            let mut url = s.trim().to_string();
            if !url.ends_with('/') {
                url.push('/');
            }
            url
        })
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
    std::env::var("STORE_CART_BASE_URL")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .map(|s| normalize_base_url(&s))
}

/// Whether cart integration is configured.
#[must_use]
pub fn cart_configured() -> bool {
    cart_base_url().is_some()
}

/// Public base URL of the cart service, where the browser is sent to add items
/// and view the cart (e.g. `http://127.0.0.1:8084/`).
#[must_use]
pub fn cart_public_base_url() -> String {
    std::env::var("STORE_CART_PUBLIC_URL")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .map(|s| normalize_base_url(&s))
        .unwrap_or_else(|| "http://127.0.0.1:8084/".to_string())
}

/// Browser origin of the cart service for CSP `form-action` (no trailing slash).
#[must_use]
pub fn cart_public_origin() -> String {
    cart_public_base_url().trim_end_matches('/').to_string()
}

/// Public base URL of the contact service for the storefront contact form.
#[must_use]
pub fn contact_public_base_url() -> String {
    std::env::var("STORE_CONTACT_PUBLIC_URL")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .map(|s| normalize_base_url(&s))
        .unwrap_or_else(|| "http://127.0.0.1:8083/".to_string())
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
    std::env::var("STORE_IDENTITY_PUBLIC_URL")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .map(|s| normalize_base_url(&s))
        .unwrap_or_else(|| "http://127.0.0.1:3000/".to_string())
}

/// Canonical public URL of this store (e.g. `http://127.0.0.1:8082/`).
#[must_use]
pub fn store_public_base_url() -> String {
    std::env::var("STORE_PUBLIC_BASE_URL")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .map(|s| normalize_base_url(&s))
        .unwrap_or_else(|| "http://127.0.0.1:8082/".to_string())
}

/// Public base URL of the info service for product detail links.
#[must_use]
pub fn info_public_base_url() -> String {
    std::env::var("STORE_INFO_PUBLIC_URL")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .map(|s| normalize_base_url(&s))
        .unwrap_or_else(|| "http://127.0.0.1:8080/".to_string())
}

/// External details page URL for a storefront SKU, when available.
#[must_use]
pub fn product_details_url(sku_code: &str) -> Option<String> {
    if sku_code == "SIGMA-RACER" {
        Some(format!(
            "{}/products/sigma-racer",
            info_public_base_url().trim_end_matches('/')
        ))
    } else {
        None
    }
}

fn normalize_base_url(url: &str) -> String {
    let mut url = url.trim().to_string();
    if !url.ends_with('/') {
        url.push('/');
    }
    url
}

/// Browser origin of the identity BFF for CSP `connect-src` (no trailing slash).
#[must_use]
pub fn identity_public_origin() -> String {
    identity_public_base_url().trim_end_matches('/').to_string()
}
