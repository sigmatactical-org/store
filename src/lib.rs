//! Sigma Store: public storefront and internal admin UI for catalog listings.

#![forbid(unsafe_code)]

mod api;
mod catalog;
pub mod config;
mod identity;
mod model;
mod product_url;
pub mod store;
mod templates;
mod web;

use std::convert::Infallible;
use std::sync::Arc;

use warp::Filter;
use warp::Reply;
use warp::http::header::{HeaderName, HeaderValue};

pub use model::{CreateListing, Listing, UpdateListing};

/// Shared listings store handle (`PgPool` is internally concurrent).
pub type SharedStore = Arc<store::ListingsStore>;

/// Connect to PostgreSQL and serve the site until a shutdown signal arrives.
///
/// # Errors
///
/// Returns an error when the database connection or binding the listen
/// address fails.
pub async fn run() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let store = store::ListingsStore::connect().await?;
    let addr = sigma_theme::warp::listen_addr_from_env();
    sigma_theme::warp::serve("Sigma Store", addr, routes(store)).await?;
    Ok(())
}

fn with_store(
    store: SharedStore,
) -> impl Filter<Extract = (SharedStore,), Error = Infallible> + Clone {
    warp::any().map(move || store.clone())
}

/// Local CSP: `sigma_theme::warp::security_headers` cannot extend
/// `form-action` (the add-to-cart form posts cross-origin to the cart
/// service) and hard-codes `style-src` without `'unsafe-inline'`.
fn content_security_policy() -> String {
    let identity_origin = config::identity_public_origin();
    let cart_origin = config::cart_public_origin();
    format!(
        "default-src 'self'; base-uri 'self'; object-src 'none'; frame-ancestors 'none'; \
         img-src 'self' data:; style-src 'self' 'unsafe-inline'; script-src 'self'; \
         font-src 'self'; connect-src 'self' {identity_origin}; form-action 'self' {cart_origin}"
    )
}

/// Local CSP plus the shared security header set (see
/// [`sigma_theme::SECURITY_HEADERS`]).
fn security_header_map() -> warp::http::HeaderMap {
    let mut map = warp::http::HeaderMap::new();
    map.insert(
        warp::http::header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_str(&content_security_policy()).expect("valid CSP header value"),
    );
    for (name, value) in sigma_theme::SECURITY_HEADERS {
        map.insert(
            HeaderName::from_static(name),
            HeaderValue::from_static(value),
        );
    }
    map
}

/// Site routes: web UI, JSON API, `/up`, theme static assets, error recovery.
pub fn routes(
    store: store::ListingsStore,
) -> impl Filter<Extract = (impl Reply,), Error = Infallible> + Clone + Send + 'static {
    let health_pool = Arc::new(store.pool().clone());
    let store = Arc::new(store);

    sigma_theme::warp::site_routes(
        web::routes(with_store(store.clone())).or(api::routes(with_store(store))),
        sigma_pg::health::warp::health_routes("store", Some(health_pool)),
    )
    .with(warp::reply::with::headers(security_header_map()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use warp::http::StatusCode;

    async fn test_store() -> store::ListingsStore {
        sigma_pg::clients::internal::ensure_test_internal_token();
        store::ListingsStore::connect_empty()
            .await
            .expect("PostgreSQL required for tests")
    }

    #[tokio::test]
    async fn up_returns_ok() {
        let res = warp::test::request()
            .method("GET")
            .path("/up")
            .reply(&routes(test_store().await))
            .await;
        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn index_lists_store() {
        let res = warp::test::request()
            .method("GET")
            .path("/")
            .reply(&routes(test_store().await))
            .await;
        assert_eq!(res.status(), StatusCode::OK);
        let body = std::str::from_utf8(res.body()).unwrap();
        assert!(body.contains("Sigma Store"));
    }

    #[test]
    fn csp_allows_identity_status_fetch() {
        let csp = content_security_policy();
        assert!(
            csp.contains("connect-src 'self' http://127.0.0.1:3000"),
            "csp should allow identity origin, got: {csp}"
        );
    }

    #[test]
    fn csp_allows_cart_form_action() {
        let csp = content_security_policy();
        assert!(
            csp.contains("form-action 'self' http://127.0.0.1:8084"),
            "csp should allow posting forms to the cart origin, got: {csp}"
        );
    }

    #[tokio::test]
    async fn responses_carry_shared_security_headers() {
        let res = warp::test::request()
            .method("GET")
            .path("/up")
            .reply(&routes(test_store().await))
            .await;
        for (name, value) in sigma_theme::SECURITY_HEADERS {
            assert_eq!(res.headers().get(*name).unwrap(), value, "header {name}");
        }
    }

    #[tokio::test]
    async fn admin_page_renders() {
        let res = warp::test::request()
            .method("GET")
            .path("/admin")
            .reply(&routes(test_store().await))
            .await;
        assert_eq!(res.status(), StatusCode::OK);
        let body = std::str::from_utf8(res.body()).unwrap();
        assert!(body.contains("Manage listings"));
    }

    #[tokio::test]
    async fn product_page_not_found_for_unknown_sku() {
        let res = warp::test::request()
            .method("GET")
            .path("/products/does-not-exist")
            .reply(&routes(test_store().await))
            .await;
        assert_eq!(res.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn api_lists_empty_listings() {
        let res = warp::test::request()
            .method("GET")
            .path("/listings")
            .header("accept", "application/json")
            .header(
                "x-sigma-internal-token",
                sigma_pg::clients::internal::TEST_INTERNAL_TOKEN,
            )
            .reply(&routes(test_store().await))
            .await;
        assert_eq!(res.status(), StatusCode::OK);
        let body: Vec<serde_json::Value> = serde_json::from_slice(res.body()).unwrap();
        assert!(body.is_empty());
    }

    #[tokio::test]
    async fn api_create_listing() {
        let res = warp::test::request()
            .method("POST")
            .path("/listings")
            .header("content-type", "application/json")
            .header("x-sigma-internal-token", sigma_pg::clients::internal::TEST_INTERNAL_TOKEN)
            .body(
                r#"{"sku_id":"abc-123","price_cents":1999,"featured":true,"visible":true,"sort_order":0}"#,
            )
            .reply(&routes(test_store().await))
            .await;
        assert_eq!(res.status(), StatusCode::CREATED);
        let listing: Listing = serde_json::from_slice(res.body()).unwrap();
        assert_eq!(listing.sku_id, "abc-123");
        assert_eq!(listing.price_cents, Some(1999));
    }

    #[tokio::test]
    async fn api_lists_empty_items() {
        let res = warp::test::request()
            .method("GET")
            .path("/items")
            .header("accept", "application/json")
            .header(
                "x-sigma-internal-token",
                sigma_pg::clients::internal::TEST_INTERNAL_TOKEN,
            )
            .reply(&routes(test_store().await))
            .await;
        assert_eq!(res.status(), StatusCode::OK);
        let body: Vec<serde_json::Value> = serde_json::from_slice(res.body()).unwrap();
        assert!(body.is_empty());
    }

    #[tokio::test]
    async fn api_users_not_configured() {
        let res = warp::test::request()
            .method("GET")
            .path("/users")
            .header("accept", "application/json")
            .header(
                "x-sigma-internal-token",
                sigma_pg::clients::internal::TEST_INTERNAL_TOKEN,
            )
            .reply(&routes(test_store().await))
            .await;
        assert_eq!(res.status(), StatusCode::SERVICE_UNAVAILABLE);
    }
}
