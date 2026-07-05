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

/// GitHub repository (`owner/name`) for SIGMA-RACER build specs markdown.
#[must_use]
pub fn racer_specs_repo() -> (String, String) {
    let value = std::env::var("STORE_RACER_SPECS_REPO")
        .unwrap_or_else(|_| "sigmatactical-org/racer".to_string());
    parse_github_repo(&value).unwrap_or_else(|| {
        ("sigmatactical-org".to_string(), "racer".to_string())
    })
}

/// Git ref (branch, tag, or commit) for racer specs.
#[must_use]
pub fn racer_specs_ref() -> String {
    std::env::var("STORE_RACER_SPECS_REF")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "main".to_string())
}

/// In-memory cache TTL for fetched racer specs (default 30 minutes).
#[must_use]
pub fn racer_specs_cache_ttl() -> std::time::Duration {
    const DEFAULT_SECS: u64 = 30 * 60;
    std::env::var("STORE_RACER_SPECS_CACHE_TTL_SECS")
        .ok()
        .and_then(|value| value.parse().ok())
        .map(std::time::Duration::from_secs)
        .unwrap_or_else(|| std::time::Duration::from_secs(DEFAULT_SECS))
}

fn parse_github_repo(value: &str) -> Option<(String, String)> {
    let value = value.trim().trim_start_matches("https://github.com/");
    let (owner, repo) = value.split_once('/')?;
    let repo = repo.trim_end_matches('/').trim_end_matches(".git");
    if owner.is_empty() || repo.is_empty() {
        return None;
    }
    Some((owner.to_string(), repo.to_string()))
}
