# sigma-store

[![CI](https://github.com/sigmatactical-org/store/actions/workflows/ci.yml/badge.svg)](https://github.com/sigmatactical-org/store/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)
[![MSRV](https://img.shields.io/badge/MSRV-1.97.0-blue.svg)](https://www.rust-lang.org)

Public storefront for Sigma Tactical Group. Pulls product SKUs from the catalog service, manages storefront listings locally, and exposes a customer-facing web UI, an internal admin UI, plus a JSON API.

Repository: https://github.com/sigmatactical-org/store

Shared site chrome comes from [sigma-theme](https://github.com/sigmatactical-org/sigma-theme). Product data comes from [sigma-catalog](https://github.com/sigmatactical-org/catalog). User directory comes from [sigma-identity](https://github.com/sigmatactical-org/identity) (Keycloak Admin API).

## Public vs internal

- **Public** (`sigmatactical.store`): `GET /` (storefront browse) and `GET /products/{sku_code}` (product detail). The cart itself is a separate public service — [sigma-cart](https://github.com/sigmatactical-org/cart) — that this store adds items to. No admin data is rendered on these pages.
- **Internal / admin only**: `GET /admin` (listing management + identity users) and the `/listings/*` CRUD pages. These are not linked from the public pages and, like [sigma-catalog](https://github.com/sigmatactical-org/catalog), [sigma-cart](https://github.com/sigmatactical-org/cart), and [sigma-contact](https://github.com/sigmatactical-org/contact), are intended to be reached only through the [sigma-identity](https://github.com/sigmatactical-org/identity) authenticated proxy in production — not exposed on the public domain.

## Features

- **Catalog integration** — fetch SKUs from sigma-catalog at request time
- **Cart integration** — Add to cart from the product page posts directly to the public [sigma-cart](https://github.com/sigmatactical-org/cart) service, which owns the cart UI; the store shows a live item count in the navbar (read server-side from the shared `sigma_cart` cookie)
- **Identity integration** — fetch realm users from the identity provider (Keycloak Admin API)
- **Storefront listings** — link catalog SKUs to the storefront with price, visibility, featured flag, and sort order
- **Public web UI** — browse the storefront and view product detail pages
- **Admin web UI** — manage listings and look up identity users
- **JSON API** — programmatic CRUD for integration behind [sigma-identity](https://github.com/sigmatactical-org/identity)

## Configuration

| Variable | Purpose |
|----------|---------|
| `PORT` | Listen port (default `8080`) |
| `DATABASE_URL` | PostgreSQL connection URL (default `postgres://sigma:sigma@127.0.0.1:5432/sigma`) |
| `STORE_CATALOG_BASE_URL` | Catalog service root (e.g. `http://127.0.0.1:8080/`) |
| `STORE_CART_BASE_URL` | Cart service root over the mesh (e.g. `http://127.0.0.1:8084/`); enables the server-side navbar badge count |
| `STORE_CART_PUBLIC_URL` | Public cart service URL the browser is sent to for Add to cart and the cart link (default `http://127.0.0.1:8084/`) |
| `STORE_IDENTITY_ISSUER_URL` | OIDC issuer / realm URL (e.g. `http://127.0.0.1:8101/realms/multcorp`) |
| `STORE_IDENTITY_CLIENT_ID` | Service-account client id for Admin API |
| `STORE_IDENTITY_CLIENT_SECRET` | Service-account client secret |
| `STORE_IDENTITY_PUBLIC_URL` | Public identity BFF base URL for sign-in (e.g. `http://127.0.0.1:3000/`) |
| `STORE_PUBLIC_BASE_URL` | Canonical store URL for login return (default `http://127.0.0.1:8082/`) |
| `STORE_INFO_PUBLIC_URL` | Public info service base URL for product Details links (default `http://127.0.0.1:8080/`) |

The **Sign in** button on public pages links to `{STORE_IDENTITY_PUBLIC_URL}/auth/login` with `app_uri` and `redirect_uri` set to the store. Add the store origin to identity's `IDENTITY_LOGIN_REDIRECT_APP_URIS` and `IDENTITY_REGISTRATION_RETURN_URIS` (e.g. `http://localhost:8082/*`).

SIGMA-RACER **Details** links to `{STORE_INFO_PUBLIC_URL}products/sigma-racer` on [sigma-info](https://github.com/sigmatactical-org/info), which hosts the tabbed build specifications fetched from [racer](https://github.com/sigmatactical-org/racer).

Shoppers **Add to cart** from a product page. The button posts directly to the public [sigma-cart](https://github.com/sigmatactical-org/cart) service (`STORE_CART_PUBLIC_URL`), which owns the cart UI, the guest cart (tracked by a shared `sigma_cart` cookie), and the deposit-to-reserve checkout. The store reads that same cookie server-side (via `STORE_CART_BASE_URL`) to show a live item count in the navbar; when it is unset, the badge is simply hidden. For the cookie to be shared, the store and cart must sit on the same registrable domain (sibling subdomains in prod; the same `localhost` in dev).

Identity integration requires a Keycloak client with **service accounts enabled** and the **view-users** role on **realm-management**. In the dev realm, run `platform/scripts/seed-keycloak-dev-users.sh` to grant `view-users` on `service-account-identity`.

## Data model

Each **listing** references a catalog SKU by id:

- `sku_id` — catalog SKU id (required, one listing per SKU)
- `price_cents` — optional display price in cents
- `visible` — show on the public storefront
- `featured` — highlight on the storefront
- `sort_order` — display order (lower first)

## API

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/items` | Visible storefront items (listings merged with catalog SKUs) |
| `GET` | `/users` | Realm users from identity (requires identity config) |
| `GET` | `/listings` | List all listings (admin) |
| `GET` | `/listings/{id}` | Get one listing |
| `POST` | `/listings` | Create listing (JSON) |
| `PUT` | `/listings/{id}` | Update listing |
| `DELETE` | `/listings/{id}` | Delete listing |

Example create listing:

```json
{
  "sku_id": "<catalog-sku-uuid>",
  "price_cents": 1999,
  "featured": true,
  "visible": true,
  "sort_order": 0
}
```

### Behind sigma-identity

Point identity at this service, for example:

```bash
IDENTITY_PROXY_TARGET=http://127.0.0.1:8080/
```

Browser clients call `/api/listings` on the identity host (with session + CSRF); identity forwards the request with a Bearer token attached.

## Development

Run catalog and store on different ports:

```bash
# Terminal 1 — catalog (default 8080)
cd sigma/it/catalog && ./scripts/prepare-local.sh && cargo run -p sigma-catalog

# Terminal 2 — cart (default 8084); point it back at the store for prices
cd sigma/it/cart && ./scripts/prepare-local.sh
export CART_CATALOG_BASE_URL=http://127.0.0.1:8080/
export CART_STORE_BASE_URL=http://127.0.0.1:8082/
export CART_STORE_PUBLIC_URL=http://127.0.0.1:8082/
export CART_PUBLIC_BASE_URL=http://127.0.0.1:8084/
PORT=8084 cargo run -p sigma-cart

# Terminal 3 — store
cd sigma/it/store && ./scripts/prepare-local.sh
export STORE_CATALOG_BASE_URL=http://127.0.0.1:8080/
export STORE_CART_BASE_URL=http://127.0.0.1:8084/
export STORE_CART_PUBLIC_URL=http://127.0.0.1:8084/
export STORE_IDENTITY_PUBLIC_URL=http://127.0.0.1:3000/
export STORE_PUBLIC_BASE_URL=http://127.0.0.1:8082/
export PORT=8082
cargo run -p sigma-store
```

From the sigma workspace (`sigma/it/`):

```bash
cd sigma/it && ./scripts/prepare-commerce-local.sh
STORE_CATALOG_BASE_URL=http://127.0.0.1:8080/ PORT=8082 cargo run -p sigma-store
```

Open http://localhost:8082

### Seed the first product (Sigma Racer)

With catalog and store both running locally, create the Sigma Racer SKU and listing:

```bash
CATALOG_URL=http://127.0.0.1:8080 STORE_URL=http://127.0.0.1:8082 \
  ../scripts/seed-sigma-racer.sh
```

## Docker

Release is in **`.github/workflows/release.yml`** when configured. Locally:

```bash
./scripts/docker-build.sh
docker build -f Dockerfile build/image
```

Data is stored in the shared PostgreSQL `store` schema (`store.listings` JSONB table). Postgres runs in the [platform](https://github.com/sigmatactical-org/platform) kind stack — port-forward for local `cargo run`:

```bash
cd platform && ./scripts/postgres-dev.sh port-forward-bg && ./scripts/postgres-dev.sh migrate
```

## Brand & artwork

© Sigma Tactical Group. **All rights reserved.**

The Sigma Tactical Group name, logos, marks, artwork, and visual identity are **proprietary**. They are not covered by this repository's source-code license. See [BRANDING.md](BRANDING.md).

## License

MIT OR Apache-2.0 for **source code** only. Branding remains proprietary.
