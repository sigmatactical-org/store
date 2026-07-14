use std::convert::Infallible;

use warp::http::StatusCode;
use warp::{Filter, Rejection, Reply};

use crate::SharedStore;
use crate::cart;
use crate::catalog;
use crate::identity;
use crate::model::ListingForm;
use crate::store::StoreError;
use crate::templates::{self, FormValues};

/// Cookie tying a browser to its guest cart. Owned by the cart service and
/// shared with the store (same host in dev, shared parent domain in prod) so
/// the store can render a live item count.
const CART_COOKIE: &str = "sigma_cart";

/// Build this module's routes.
pub fn routes(
    store: impl Filter<Extract = (SharedStore,), Error = Infallible> + Clone + Send + 'static,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone + Send + 'static {
    storefront_page(store.clone())
        .or(product_page(store.clone()))
        .or(admin_page(store.clone()))
        .or(new_listing_page(store.clone()))
        .or(create_listing_form(store.clone()))
        .or(edit_listing_page(store.clone()))
        .or(update_listing_form(store.clone()))
        .or(delete_listing_form(store))
}

/// Extract the guest cart id from the request `Cookie` header, if present.
fn cart_id_from_cookie(cookie_header: Option<&str>) -> Option<String> {
    cookie_header?.split(';').find_map(|pair| {
        let (name, value) = pair.split_once('=')?;
        (name.trim() == CART_COOKIE)
            .then(|| value.trim().to_string())
            .filter(|v| !v.is_empty())
    })
}

/// Total item count for the nav cart badge (0 when there is no live cart).
async fn cart_count(cookie_header: Option<&str>) -> u32 {
    if !crate::config::cart_configured() {
        return 0;
    }
    let Some(cart_id) = cart_id_from_cookie(cookie_header) else {
        return 0;
    };
    match cart::get_cart(&cart_id).await {
        Ok(Some(detail)) => detail.item_count(),
        _ => 0,
    }
}

/// Public storefront: `GET /`. Visible, catalog-backed listings only.
fn storefront_page(
    store: impl Filter<Extract = (SharedStore,), Error = Infallible> + Clone + Send + 'static,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone + Send + 'static {
    warp::path::end()
        .and(warp::get())
        .and(warp::header::optional::<String>("cookie"))
        .and(store)
        .and_then(|cookie: Option<String>, store: SharedStore| async move {
            let listings = store.list().await.map_err(|_| warp::reject::not_found())?;
            let catalog_skus = catalog::fetch_skus().await.unwrap_or_default();
            let count = cart_count(cookie.as_deref()).await;
            templates::render_storefront_html(listings, &catalog_skus, count)
                .map(warp::reply::html)
                .map_err(|_| warp::reject::not_found())
        })
}

/// Public product detail page: `GET /products/{sku_code}`.
fn product_page(
    store: impl Filter<Extract = (SharedStore,), Error = Infallible> + Clone + Send + 'static,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone + Send + 'static {
    warp::path!("products" / String)
        .and(warp::get())
        .and(warp::header::optional::<String>("cookie"))
        .and(store)
        .and_then(
            |sku_code: String, cookie: Option<String>, store: SharedStore| async move {
                let listings = store.list().await.map_err(|_| warp::reject::not_found())?;
                let catalog_skus = catalog::fetch_skus().await.unwrap_or_default();
                let Some(product) = templates::find_product(&sku_code, &listings, &catalog_skus)
                else {
                    return Err(warp::reject::not_found());
                };
                let count = cart_count(cookie.as_deref()).await;
                templates::render_product_html(product, count)
                    .map(warp::reply::html)
                    .map_err(|_| warp::reject::not_found())
            },
        )
}

/// Internal admin dashboard: `GET /admin`. Intended to be reached only through
/// the sigma-identity authenticated proxy in production.
fn admin_page(
    store: impl Filter<Extract = (SharedStore,), Error = Infallible> + Clone + Send + 'static,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone + Send + 'static {
    warp::path("admin")
        .and(warp::path::end())
        .and(warp::get())
        .and(store)
        .and_then(|store: SharedStore| async move {
            let listings = store.list().await.map_err(|_| warp::reject::not_found())?;
            let catalog_result = catalog::fetch_skus().await;
            let (catalog_skus, catalog_error) = match catalog_result {
                Ok(skus) => (Some(skus), None),
                Err(e) => (None, Some(e.to_string())),
            };
            let identity_result = identity::fetch_users().await;
            let (identity_users, identity_error) = match identity_result {
                Ok(users) => (Some(users), None),
                Err(e) if crate::config::identity_configured() => (None, Some(e.to_string())),
                Err(_) => (None, None),
            };
            templates::render_admin_html(templates::AdminPageInput {
                listings,
                catalog_skus: catalog_skus.as_deref().unwrap_or(&[]),
                catalog_configured: crate::config::catalog_configured(),
                catalog_error,
                identity_users: identity_users.as_deref().unwrap_or(&[]),
                identity_configured: crate::config::identity_configured(),
                identity_error,
                message: None,
            })
            .map(warp::reply::html)
            .map_err(|_| warp::reject::not_found())
        })
}

fn new_listing_page(
    store: impl Filter<Extract = (SharedStore,), Error = Infallible> + Clone + Send + 'static,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone + Send + 'static {
    warp::path("listings")
        .and(warp::path("new"))
        .and(warp::path::end())
        .and(warp::get())
        .and(store)
        .and_then(|store: SharedStore| async move {
            let listings = store.list().await.map_err(|_| warp::reject::not_found())?;
            let catalog_skus = catalog::fetch_skus().await.unwrap_or_default();
            templates::render_form_html(listings, &catalog_skus, None, None)
                .map(warp::reply::html)
                .map_err(|_| warp::reject::not_found())
        })
}

fn create_listing_form(
    store: impl Filter<Extract = (SharedStore,), Error = Infallible> + Clone + Send + 'static,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone + Send + 'static {
    warp::path("listings")
        .and(warp::path::end())
        .and(warp::post())
        .and(warp::body::form())
        .and(store)
        .and_then(|form: ListingForm, store: SharedStore| async move {
            let listings = store.list().await.map_err(|_| warp::reject::not_found())?;
            let catalog_skus = catalog::fetch_skus().await.unwrap_or_default();
            let values = form_to_values(&form);
            let response = match form.into_create() {
                Ok(input) => {
                    if let Err(e) = catalog::require_active_sku(&input.sku_id).await {
                        render_form_error(
                            listings,
                            &catalog_skus,
                            None,
                            values,
                            invalid_input(format!("catalog validation failed: {e}")),
                        )
                    } else {
                        match store.create(input).await {
                            Ok(_) => {
                                warp::redirect::redirect(warp::http::Uri::from_static("/admin"))
                                    .into_response()
                            }
                            Err(e) => render_form_error(listings, &catalog_skus, None, values, e),
                        }
                    }
                }
                Err(e) => {
                    render_form_error(listings, &catalog_skus, None, values, invalid_input(e))
                }
            };
            Ok::<_, Rejection>(response)
        })
}

fn edit_listing_page(
    store: impl Filter<Extract = (SharedStore,), Error = Infallible> + Clone + Send + 'static,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone + Send + 'static {
    warp::path!("listings" / String / "edit")
        .and(warp::get())
        .and(store)
        .and_then(|id: String, store: SharedStore| async move {
            let listing = match store.get(&id).await {
                Ok(Some(listing)) => listing,
                Ok(None) => return Err(warp::reject::not_found()),
                Err(_) => return Err(warp::reject::not_found()),
            };
            let listings = store.list().await.map_err(|_| warp::reject::not_found())?;
            let catalog_skus = catalog::fetch_skus().await.unwrap_or_default();
            templates::render_form_html(listings, &catalog_skus, Some(listing), None)
                .map(warp::reply::html)
                .map_err(|_| warp::reject::not_found())
        })
}

fn update_listing_form(
    store: impl Filter<Extract = (SharedStore,), Error = Infallible> + Clone + Send + 'static,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone + Send + 'static {
    warp::path!("listings" / String / "edit")
        .and(warp::post())
        .and(warp::body::form())
        .and(store)
        .and_then(
            |id: String, form: ListingForm, store: SharedStore| async move {
                let listings = store.list().await.map_err(|_| warp::reject::not_found())?;
                let catalog_skus = catalog::fetch_skus().await.unwrap_or_default();
                let values = form_to_values(&form);
                let response = match form.into_update() {
                    Ok(input) => {
                        if let Err(e) = catalog::require_active_sku(&input.sku_id).await {
                            let listing = store.get(&id).await.ok().flatten();
                            render_form_error(
                                listings,
                                &catalog_skus,
                                listing,
                                values,
                                invalid_input(format!("catalog validation failed: {e}")),
                            )
                        } else {
                            match store.update(&id, input).await {
                                Ok(_) => {
                                    warp::redirect::redirect(warp::http::Uri::from_static("/admin"))
                                        .into_response()
                                }
                                Err(e) => {
                                    let listing = store.get(&id).await.ok().flatten();
                                    render_form_error(listings, &catalog_skus, listing, values, e)
                                }
                            }
                        }
                    }
                    Err(e) => {
                        let listing = store.get(&id).await.ok().flatten();
                        render_form_error(
                            listings,
                            &catalog_skus,
                            listing,
                            values,
                            invalid_input(e),
                        )
                    }
                };
                Ok::<_, Rejection>(response)
            },
        )
}

fn delete_listing_form(
    store: impl Filter<Extract = (SharedStore,), Error = Infallible> + Clone + Send + 'static,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone + Send + 'static {
    warp::path!("listings" / String / "delete")
        .and(warp::post())
        .and(store)
        .and_then(|id: String, store: SharedStore| async move {
            let catalog_skus = catalog::fetch_skus().await.unwrap_or_default();
            match store.delete(&id).await {
                Ok(()) => Ok(
                    warp::redirect::redirect(warp::http::Uri::from_static("/admin"))
                        .into_response(),
                ),
                Err(StoreError::NotFound) => Err(warp::reject::not_found()),
                Err(e) => {
                    let listings = store.list().await.unwrap_or_default();
                    templates::render_admin_html(templates::AdminPageInput {
                        listings,
                        catalog_skus: &catalog_skus,
                        catalog_configured: crate::config::catalog_configured(),
                        catalog_error: None,
                        identity_users: &[],
                        identity_configured: crate::config::identity_configured(),
                        identity_error: None,
                        message: Some(format!("Delete failed: {e}")),
                    })
                    .map(|html| warp::reply::html(html).into_response())
                    .map_err(|_| warp::reject::not_found())
                }
            }
        })
}

fn form_to_values(form: &ListingForm) -> FormValues {
    FormValues {
        sku_id: form.sku_id.clone(),
        price: form.price.clone(),
        featured: form.featured.is_some(),
        visible: form.visible.is_some(),
        sort_order: form.sort_order.clone(),
    }
}

fn invalid_input(message: String) -> StoreError {
    StoreError::InvalidInput(message)
}

fn render_form_error(
    listings: Vec<crate::model::Listing>,
    catalog_skus: &[catalog::CatalogSku],
    listing: Option<crate::model::Listing>,
    values: FormValues,
    err: StoreError,
) -> warp::reply::Response {
    let message = err.to_string();
    match templates::render_form_html_with_values(
        listings,
        catalog_skus,
        listing,
        Some(message),
        values,
    ) {
        Ok(html) => warp::reply::with_status(warp::reply::html(html), StatusCode::BAD_REQUEST)
            .into_response(),
        Err(_) => warp::reply::with_status(warp::reply(), StatusCode::INTERNAL_SERVER_ERROR)
            .into_response(),
    }
}
