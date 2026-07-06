//! Lowercase URL paths for storefront product pages.

/// URL slug for a catalog SKU code (always lowercase).
#[must_use]
pub fn slug(sku_code: &str) -> String {
    sku_code.to_lowercase()
}

/// Storefront path for a product (`/products/{slug}`).
#[must_use]
pub fn path(sku_code: &str) -> String {
    format!("/products/{}", slug(sku_code))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slug_is_lowercase() {
        assert_eq!(slug("SIGMA-RACER"), "sigma-racer");
    }

    #[test]
    fn path_uses_lowercase_slug() {
        assert_eq!(path("SIGMA-RACER"), "/products/sigma-racer");
    }
}
