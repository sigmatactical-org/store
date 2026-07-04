//! Sigma Store: public storefront and internal admin UI for catalog listings.

mod api;
mod auth_links;
mod catalog;
pub mod config;
mod identity;
mod model;
mod specs;
pub mod store;
mod templates;
mod web;

use std::convert::Infallible;
use std::sync::Arc;

use tokio::sync::Mutex;
use warp::Filter;
use warp::Reply;

pub use model::{CreateListing, Listing, RealmUser, UpdateListing};

/// Shared mutable listings store handle.
pub type SharedStore = Arc<Mutex<store::ListingsStore>>;

/// Resolve listen address from **`PORT`** (default **8080**).
#[must_use]
pub fn listen_socket_addr_from_env() -> std::net::SocketAddr {
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(8080);
    SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), port)
}

fn with_store(
    store: SharedStore,
) -> impl Filter<Extract = (SharedStore,), Error = Infallible> + Clone {
    warp::any().map(move || store.clone())
}

fn content_security_policy() -> String {
    let identity_origin = config::identity_public_origin();
    format!(
        "default-src 'self'; base-uri 'self'; object-src 'none'; frame-ancestors 'none'; \
         img-src 'self' data:; style-src 'self' 'unsafe-inline'; script-src 'self'; \
         font-src 'self'; connect-src 'self' {identity_origin}; form-action 'self'"
    )
}

/// Site routes: web UI, JSON API, `/up`, theme static assets, error recovery.
pub fn routes(
    store: store::ListingsStore,
) -> impl Filter<Extract = (impl Reply,), Error = Infallible> + Clone + Send + 'static {
    use warp::reply::with::header;

    let store = Arc::new(Mutex::new(store));

    warp::path("up")
        .and(warp::get())
        .map(|| warp::reply::with_status("up", warp::http::StatusCode::OK))
        .or(web::routes(with_store(store.clone())))
        .or(api::routes(with_store(store)))
        .or(sigma_theme::warp::static_files())
        .or(sigma_theme::warp::favicon())
        .recover(sigma_theme::warp::handle_rejection)
        .with(header("content-security-policy", content_security_policy()))
        .with(header("x-content-type-options", "nosniff"))
        .with(header("x-frame-options", "DENY"))
        .with(header("referrer-policy", "strict-origin-when-cross-origin"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use warp::http::StatusCode;

    async fn test_store() -> store::ListingsStore {
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
            .reply(&routes(test_store().await))
            .await;
        assert_eq!(res.status(), StatusCode::SERVICE_UNAVAILABLE);
    }
}
