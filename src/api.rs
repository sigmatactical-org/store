mod storefront_item;
pub(crate) use storefront_item::StorefrontItem;

use std::convert::Infallible;

use sigma_pg::api::{internal_auth, json_error, store_error_status};
use warp::http::StatusCode;
use warp::reply::Response;
use warp::{Filter, Rejection, Reply};

use crate::SharedStore;
use crate::catalog::{self, CatalogSku};
use crate::identity;
use crate::model::{CreateListing, Listing, UpdateListing};
use crate::store::StoreError;

/// Merge listings with their catalog SKUs (missing/unfetched SKUs stay `None`).
fn enrich_listings(listings: Vec<Listing>, skus: Option<&[CatalogSku]>) -> Vec<StorefrontItem> {
    listings
        .into_iter()
        .map(|listing| {
            let sku = skus.and_then(|all| catalog::sku_by_id(all, &listing.sku_id).cloned());
            StorefrontItem { listing, sku }
        })
        .collect()
}

async fn require_catalog_sku(sku_id: &str) -> Result<(), Response> {
    catalog::require_active_sku(sku_id).await.map_err(|e| {
        json_error(
            StatusCode::BAD_REQUEST,
            format!("catalog validation failed: {e}"),
        )
    })
}

/// Build this module's routes.
pub fn routes(
    store: impl Filter<Extract = (SharedStore,), Error = Infallible> + Clone + Send + 'static,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone + Send + 'static {
    list_items(store.clone())
        .or(list_users())
        .or(list_listings(store.clone()))
        .or(get_listing(store.clone()))
        .or(create_listing(store.clone()))
        .or(update_listing(store.clone()))
        .or(delete_listing(store))
}

fn list_items(
    store: impl Filter<Extract = (SharedStore,), Error = Infallible> + Clone + Send + 'static,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone + Send + 'static {
    warp::path("items")
        .and(warp::path::end())
        .and(warp::get())
        .and(internal_auth())
        .and(store)
        .and_then(|store: SharedStore| async move {
            let (listings, skus) = tokio::join!(store.list(), catalog::fetch_skus());
            let listings = match listings {
                Ok(listings) => listings,
                Err(e) => return Ok(json_error(store_error_status(&e), e.to_string())),
            };
            let visible: Vec<Listing> = listings.into_iter().filter(|l| l.visible).collect();
            let skus = skus.ok();
            let items = enrich_listings(visible, skus.as_deref().map(Vec::as_slice));
            Ok::<_, Rejection>(warp::reply::json(&items).into_response())
        })
}

fn list_users() -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone + Send + 'static
{
    warp::path("users")
        .and(warp::path::end())
        .and(warp::get())
        .and(internal_auth())
        .and_then(|| async move {
            let response = match identity::fetch_users().await {
                Ok(users) => warp::reply::json(&users).into_response(),
                Err(e) => json_error(
                    if matches!(e, identity::IdentityError::NotConfigured) {
                        StatusCode::SERVICE_UNAVAILABLE
                    } else {
                        StatusCode::BAD_GATEWAY
                    },
                    e.to_string(),
                ),
            };
            Ok::<_, Rejection>(response)
        })
}

fn list_listings(
    store: impl Filter<Extract = (SharedStore,), Error = Infallible> + Clone + Send + 'static,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone + Send + 'static {
    warp::path("listings")
        .and(warp::path::end())
        .and(warp::get())
        .and(internal_auth())
        .and(store)
        .and_then(|store: SharedStore| async move {
            let (listings, skus) = tokio::join!(store.list(), catalog::fetch_skus());
            let listings = match listings {
                Ok(listings) => listings,
                Err(e) => return Ok(json_error(store_error_status(&e), e.to_string())),
            };
            let skus = skus.ok();
            let items = enrich_listings(listings, skus.as_deref().map(Vec::as_slice));
            Ok::<_, Rejection>(warp::reply::json(&items).into_response())
        })
}

fn get_listing(
    store: impl Filter<Extract = (SharedStore,), Error = Infallible> + Clone + Send + 'static,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone + Send + 'static {
    warp::path!("listings" / String)
        .and(warp::path::end())
        .and(warp::get())
        .and(internal_auth())
        .and(store)
        .and_then(|id: String, store: SharedStore| async move {
            let listing = match store.get(&id).await {
                Ok(Some(listing)) => listing,
                Ok(None) => return Err(warp::reject::not_found()),
                Err(e) => return Ok(json_error(store_error_status(&e), e.to_string())),
            };
            let sku = catalog::fetch_sku(&listing.sku_id).await.ok().flatten();
            Ok(warp::reply::json(&StorefrontItem { listing, sku }).into_response())
        })
}

fn create_listing(
    store: impl Filter<Extract = (SharedStore,), Error = Infallible> + Clone + Send + 'static,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone + Send + 'static {
    warp::path("listings")
        .and(warp::path::end())
        .and(warp::post())
        .and(warp::body::json())
        .and(internal_auth())
        .and(store)
        .and_then(|input: CreateListing, store: SharedStore| async move {
            if let Err(resp) = require_catalog_sku(input.sku_id.trim()).await {
                return Ok(resp);
            }
            let response = match store.create(input).await {
                Ok(listing) => {
                    warp::reply::with_status(warp::reply::json(&listing), StatusCode::CREATED)
                        .into_response()
                }
                Err(e) => json_error(store_error_status(&e), e.to_string()),
            };
            Ok::<_, Rejection>(response)
        })
}

fn update_listing(
    store: impl Filter<Extract = (SharedStore,), Error = Infallible> + Clone + Send + 'static,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone + Send + 'static {
    warp::path!("listings" / String)
        .and(warp::path::end())
        .and(warp::put())
        .and(warp::body::json())
        .and(internal_auth())
        .and(store)
        .and_then(
            |id: String, input: UpdateListing, store: SharedStore| async move {
                if let Err(resp) = require_catalog_sku(input.sku_id.trim()).await {
                    return Ok(resp);
                }
                let response = match store.update(&id, input).await {
                    Ok(listing) => warp::reply::json(&listing).into_response(),
                    Err(StoreError::NotFound(_)) => return Err(warp::reject::not_found()),
                    Err(e) => json_error(store_error_status(&e), e.to_string()),
                };
                Ok(response)
            },
        )
}

fn delete_listing(
    store: impl Filter<Extract = (SharedStore,), Error = Infallible> + Clone + Send + 'static,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone + Send + 'static {
    warp::path!("listings" / String)
        .and(warp::path::end())
        .and(warp::delete())
        .and(internal_auth())
        .and(store)
        .and_then(|id: String, store: SharedStore| async move {
            let response = match store.delete(&id).await {
                Ok(()) => {
                    warp::reply::with_status(warp::reply(), StatusCode::NO_CONTENT).into_response()
                }
                Err(StoreError::NotFound(_)) => return Err(warp::reject::not_found()),
                Err(e) => json_error(store_error_status(&e), e.to_string()),
            };
            Ok(response)
        })
}
