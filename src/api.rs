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
        StoreError::SkuIdRequired
        | StoreError::DuplicateSkuId
        | StoreError::SkuNotInCatalog(_)
        | StoreError::InvalidInput(_) => StatusCode::BAD_REQUEST,
        StoreError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

fn internal_auth()
-> impl Filter<Extract = (Option<String>, Option<String>), Error = Rejection> + Clone {
    warp::header::optional::<String>("authorization")
        .and(warp::header::optional::<String>("x-sigma-internal-token"))
}

fn ensure_internal(
    authorization: Option<String>,
    internal_token: Option<String>,
) -> Result<(), Rejection> {
    if sigma_pg::clients::internal::authorize_internal(
        authorization.as_deref(),
        internal_token.as_deref(),
    ) {
        Ok(())
    } else {
        Err(warp::reject::not_found())
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

async fn require_catalog_sku(sku_id: &str) -> Result<(), Response> {
    catalog::require_active_sku(sku_id).await.map_err(|e| {
        json_error(
            StatusCode::BAD_REQUEST,
            format!("catalog validation failed: {e}"),
        )
    })
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
        .and(internal_auth())
        .and(store)
        .and_then(
            |authorization, internal_token, store: SharedStore| async move {
                ensure_internal(authorization, internal_token)?;
                let listings = match store.list().await {
                    Ok(listings) => listings,
                    Err(e) => return Ok(json_error(store_error_status(&e), e.to_string())),
                };
                let visible: Vec<Listing> = listings.into_iter().filter(|l| l.visible).collect();
                let items = enrich_listings(visible).await;
                Ok::<_, Rejection>(warp::reply::json(&items).into_response())
            },
        )
}

fn list_users(
    _store: impl Filter<Extract = (SharedStore,), Error = Infallible> + Clone + Send + 'static,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone + Send + 'static {
    warp::path("users")
        .and(warp::path::end())
        .and(warp::get())
        .and(internal_auth())
        .and_then(|authorization, internal_token| async move {
            ensure_internal(authorization, internal_token)?;
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
        .and_then(
            |authorization, internal_token, store: SharedStore| async move {
                ensure_internal(authorization, internal_token)?;
                let listings = match store.list().await {
                    Ok(listings) => listings,
                    Err(e) => return Ok(json_error(store_error_status(&e), e.to_string())),
                };
                let items = enrich_listings(listings).await;
                Ok::<_, Rejection>(warp::reply::json(&items).into_response())
            },
        )
}

fn get_listing(
    store: impl Filter<Extract = (SharedStore,), Error = Infallible> + Clone + Send + 'static,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone + Send + 'static {
    warp::path!("listings" / String)
        .and(warp::path::end())
        .and(warp::get())
        .and(internal_auth())
        .and(store)
        .and_then(
            |id: String, authorization, internal_token, store: SharedStore| async move {
                ensure_internal(authorization, internal_token)?;
                let listing = match store.get(&id).await {
                    Ok(Some(listing)) => listing,
                    Ok(None) => return Err(warp::reject::not_found()),
                    Err(e) => return Ok(json_error(store_error_status(&e), e.to_string())),
                };
                let skus = catalog::fetch_skus().await.ok();
                let sku = skus
                    .as_ref()
                    .and_then(|all| catalog::sku_by_id(all, &listing.sku_id).cloned());
                Ok(warp::reply::json(&StorefrontItem { listing, sku }).into_response())
            },
        )
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
        .and_then(
            |input: CreateListing, authorization, internal_token, store: SharedStore| async move {
                ensure_internal(authorization, internal_token)?;
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
            },
        )
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
            |id: String,
             input: UpdateListing,
             authorization,
             internal_token,
             store: SharedStore| async move {
                ensure_internal(authorization, internal_token)?;
                if let Err(resp) = require_catalog_sku(input.sku_id.trim()).await {
                    return Ok(resp);
                }
                let response = match store.update(&id, input).await {
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
        .and(internal_auth())
        .and(store)
        .and_then(
            |id: String, authorization, internal_token, store: SharedStore| async move {
                ensure_internal(authorization, internal_token)?;
                let response = match store.delete(&id).await {
                    Ok(()) => warp::reply::with_status(warp::reply(), StatusCode::NO_CONTENT)
                        .into_response(),
                    Err(StoreError::NotFound) => return Err(warp::reject::not_found()),
                    Err(e) => json_error(store_error_status(&e), e.to_string()),
                };
                Ok(response)
            },
        )
}
