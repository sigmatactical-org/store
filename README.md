# sigma-shop

Shop storefront for Sigma Tactical Group. Pulls product SKUs from the catalog service, manages shop listings locally, and exposes a server-rendered web UI plus JSON API.

Repository: https://github.com/sigmatactical-org/shop

Shared site chrome comes from [sigma-theme](https://github.com/sigmatactical-org/sigma-theme). Product data comes from [sigma-catalog](https://github.com/sigmatactical-org/catalog). User directory comes from [sigma-identity](https://github.com/sigmatactical-org/identity) (Keycloak Admin API).

## Features

- **Catalog integration** — fetch SKUs from sigma-catalog at request time
- **Identity integration** — fetch realm users from the identity provider (Keycloak Admin API)
- **Shop listings** — link catalog SKUs to the storefront with price, visibility, featured flag, and sort order
- **Web UI** — browse the storefront and manage listings
- **JSON API** — programmatic CRUD for integration behind [sigma-identity](https://github.com/sigmatactical-org/identity)

## Configuration

| Variable | Purpose |
|----------|---------|
| `PORT` | Listen port (default `8080`) |
| `SHOP_DATA_PATH` | JSON database path (default `data/shop.json`) |
| `SHOP_CATALOG_BASE_URL` | Catalog service root (e.g. `http://127.0.0.1:8080/`) |
| `SHOP_IDENTITY_ISSUER_URL` | OIDC issuer / realm URL (e.g. `http://127.0.0.1:8101/realms/multcorp`) |
| `SHOP_IDENTITY_CLIENT_ID` | Service-account client id for Admin API |
| `SHOP_IDENTITY_CLIENT_SECRET` | Service-account client secret |

Catalog integration requires a running sigma-catalog instance. SKU definitions are managed in catalog; shop only controls how those SKUs appear on the storefront.

Identity integration requires a Keycloak client with **service accounts enabled** and the **view-users** role on **realm-management**. In the dev realm, you can reuse the `identity` client credentials and assign that role to `service-account-identity`.

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

Run catalog and shop on different ports:

```bash
# Terminal 1 — catalog (default 8080)
cd commerce/catalog && ./scripts/prepare-local.sh && cargo run -p sigma-catalog

# Terminal 2 — shop
cd commerce/shop && ./scripts/prepare-local.sh
export SHOP_CATALOG_BASE_URL=http://127.0.0.1:8080/
export PORT=8082
cargo run -p sigma-shop
```

From the sigma workspace (commerce nested workspace):

```bash
for svc in catalog shop; do (cd "commerce/$svc" && ./scripts/prepare-local.sh); done
(cd commerce && SHOP_CATALOG_BASE_URL=http://127.0.0.1:8080/ PORT=8082 cargo run -p sigma-shop)
```

Open http://localhost:8082

## Docker

Release is in **`.github/workflows/release.yml`** when configured. Locally:

```bash
./scripts/docker-build.sh
docker build -f Dockerfile build/image
```

Mount a volume at `/app/data` (or set `SHOP_DATA_PATH`) so listing data persists across restarts.

## License

MIT OR Apache-2.0
