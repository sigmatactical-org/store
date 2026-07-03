use std::convert::Infallible;

use warp::http::StatusCode;
use warp::reply::Response;
use warp::{Filter, Rejection, Reply};

use crate::SharedStore;
use crate::catalog::{self, CatalogSku};
use crate::identity;
use crate::model::{CreateListing, Listing, UpdateListing};
use crate::store::StoreError;

#[derive(serde::Serialize)]
struct ErrorBody {
    error: String,
}

#[derive(serde::Serialize)]
struct StorefrontItem {
    listing: Listing,
    sku: Option<CatalogSku>,
}

fn json_error(status: StatusCode, message: impl Into<String>) -> Response {
    warp::reply::with_status(
        warp::reply::json(&ErrorBody {
            error: message.into(),
        }),
        status,
    )
    .into_response()
}

fn store_error_status(err: &StoreError) -> StatusCode {
    match err {
        StoreError::NotFound => StatusCode::NOT_FOUND,
        StoreError::SkuIdRequired | StoreError::DuplicateSkuId | StoreError::SkuNotInCatalog(_) => {
            StatusCode::BAD_REQUEST
        }
        StoreError::Io(_) | StoreError::Json(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

async fn enrich_listings(listings: Vec<Listing>) -> Vec<StorefrontItem> {
    let skus = catalog::fetch_skus().await.ok();
    listings
        .into_iter()
        .map(|listing| {
            let sku = skus
                .as_ref()
                .and_then(|all| catalog::sku_by_id(all, &listing.sku_id).cloned());
            StorefrontItem { listing, sku }
        })
        .collect()
}

pub fn routes(
    store: impl Filter<Extract = (SharedStore,), Error = Infallible> + Clone + Send + 'static,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone + Send + 'static {
    list_items(store.clone())
        .or(list_users(store.clone()))
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
        .and(store)
        .and_then(|store: SharedStore| async move {
            let listings = store.lock().await.list();
            let visible: Vec<Listing> = listings.into_iter().filter(|l| l.visible).collect();
            let items = enrich_listings(visible).await;
            Ok::<_, Rejection>(warp::reply::json(&items))
        })
}

fn list_users(
    _store: impl Filter<Extract = (SharedStore,), Error = Infallible> + Clone + Send + 'static,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone + Send + 'static {
    warp::path("users")
        .and(warp::path::end())
        .and(warp::get())
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
        .and(store)
        .and_then(|store: SharedStore| async move {
            let listings = store.lock().await.list();
            let items = enrich_listings(listings).await;
            Ok::<_, Rejection>(warp::reply::json(&items))
        })
}

fn get_listing(
    store: impl Filter<Extract = (SharedStore,), Error = Infallible> + Clone + Send + 'static,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone + Send + 'static {
    warp::path!("listings" / String)
        .and(warp::path::end())
        .and(warp::get())
        .and(store)
        .and_then(|id: String, store: SharedStore| async move {
            let store = store.lock().await;
            let Some(listing) = store.get(&id) else {
                return Err(warp::reject::not_found());
            };
            let skus = catalog::fetch_skus().await.ok();
            let sku = skus
                .as_ref()
                .and_then(|all| catalog::sku_by_id(all, &listing.sku_id).cloned());
            Ok(warp::reply::json(&StorefrontItem { listing, sku }))
        })
}

fn create_listing(
    store: impl Filter<Extract = (SharedStore,), Error = Infallible> + Clone + Send + 'static,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone + Send + 'static {
    warp::path("listings")
        .and(warp::path::end())
        .and(warp::post())
        .and(warp::body::json())
        .and(store)
        .and_then(|input: CreateListing, store: SharedStore| async move {
            if let Ok(skus) = catalog::fetch_skus().await
                && catalog::validate_sku_id(&skus, input.sku_id.trim()).is_err()
            {
                return Ok(json_error(
                    StatusCode::BAD_REQUEST,
                    format!("catalog sku not found: {}", input.sku_id.trim()),
                ));
            }
            let mut store = store.lock().await;
            let response = match store.create(input) {
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
        .and(store)
        .and_then(
            |id: String, input: UpdateListing, store: SharedStore| async move {
                if let Ok(skus) = catalog::fetch_skus().await
                    && catalog::validate_sku_id(&skus, input.sku_id.trim()).is_err()
                {
                    return Ok(json_error(
                        StatusCode::BAD_REQUEST,
                        format!("catalog sku not found: {}", input.sku_id.trim()),
                    ));
                }
                let mut store = store.lock().await;
                let response = match store.update(&id, input) {
                    Ok(listing) => warp::reply::json(&listing).into_response(),
                    Err(StoreError::NotFound) => return Err(warp::reject::not_found()),
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
        .and(store)
        .and_then(|id: String, store: SharedStore| async move {
            let mut store = store.lock().await;
            let response = match store.delete(&id) {
                Ok(()) => {
                    warp::reply::with_status(warp::reply(), StatusCode::NO_CONTENT).into_response()
                }
                Err(StoreError::NotFound) => return Err(warp::reject::not_found()),
                Err(e) => json_error(store_error_status(&e), e.to_string()),
            };
            Ok(response)
        })
}
