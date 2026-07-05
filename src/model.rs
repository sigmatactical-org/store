use serde::{Deserialize, Serialize};

/// Realm user from the identity provider (Keycloak Admin API).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RealmUser {
    pub id: String,
    pub display_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OrderStatus {
    PendingDeposit,
}

/// Customer order placed from the public storefront.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Order {
    pub id: String,
    pub sku_code: String,
    pub username: String,
    pub price_cents: u64,
    pub deposit_cents: u64,
    pub status: OrderStatus,
    pub created_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateOrder {
    pub sku_code: String,
    pub username: String,
    pub price_cents: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OrderForm {
    pub username: String,
}

impl Order {
    pub fn new(input: CreateOrder) -> Self {
        let deposit_cents = deposit_cents_for_price(input.price_cents);
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            sku_code: input.sku_code,
            username: input.username.trim().to_string(),
            price_cents: input.price_cents,
            deposit_cents,
            status: OrderStatus::PendingDeposit,
            created_at: chrono::Utc::now().to_rfc3339(),
        }
    }
}

/// Deposit required at order time (50% of list price).
#[must_use]
pub fn deposit_cents_for_price(price_cents: u64) -> u64 {
    price_cents / 2
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Listing {
    pub id: String,
    pub sku_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price_cents: Option<u64>,
    pub featured: bool,
    pub visible: bool,
    pub sort_order: u32,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateListing {
    pub sku_id: String,
    pub price_cents: Option<u64>,
    #[serde(default)]
    pub featured: Option<bool>,
    #[serde(default)]
    pub visible: Option<bool>,
    #[serde(default)]
    pub sort_order: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateListing {
    pub sku_id: String,
    pub price_cents: Option<u64>,
    pub featured: bool,
    pub visible: bool,
    pub sort_order: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ListingForm {
    pub sku_id: String,
    pub price: String,
    pub featured: Option<String>,
    pub visible: Option<String>,
    pub sort_order: String,
}

impl ListingForm {
    pub fn into_create(self) -> Result<CreateListing, String> {
        Ok(CreateListing {
            sku_id: self.sku_id,
            price_cents: parse_price_cents(&self.price)?,
            featured: Some(self.featured.is_some()),
            visible: Some(self.visible.is_some()),
            sort_order: parse_sort_order(&self.sort_order)?,
        })
    }

    pub fn into_update(self) -> Result<UpdateListing, String> {
        Ok(UpdateListing {
            sku_id: self.sku_id,
            price_cents: parse_price_cents(&self.price)?,
            featured: self.featured.is_some(),
            visible: self.visible.is_some(),
            sort_order: parse_sort_order(&self.sort_order)?.unwrap_or(0),
        })
    }
}

fn parse_price_cents(value: &str) -> Result<Option<u64>, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    if trimmed.contains('.') {
        let parts: Vec<&str> = trimmed.split('.').collect();
        if parts.len() != 2 || parts[0].is_empty() {
            return Err("invalid price format".to_string());
        }
        let dollars: u64 = parts[0]
            .parse()
            .map_err(|_| "invalid price format".to_string())?;
        let cents_str = format!("{:0<2}", parts[1]);
        let cents_part = &cents_str[..2.min(cents_str.len())];
        let cents: u64 = cents_part
            .parse()
            .map_err(|_| "invalid price format".to_string())?;
        return Ok(Some(dollars * 100 + cents));
    }
    let dollars: u64 = trimmed
        .parse()
        .map_err(|_| "invalid price format".to_string())?;
    Ok(Some(dollars * 100))
}

fn parse_sort_order(value: &str) -> Result<Option<u32>, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    trimmed
        .parse()
        .map(Some)
        .map_err(|_| "sort order must be a non-negative integer".to_string())
}

impl Listing {
    pub fn new(input: CreateListing) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            sku_id: input.sku_id.trim().to_string(),
            price_cents: input.price_cents,
            featured: input.featured.unwrap_or(false),
            visible: input.visible.unwrap_or(true),
            sort_order: input.sort_order.unwrap_or(0),
            updated_at: now,
        }
    }

    pub fn apply_update(&mut self, input: UpdateListing) {
        self.sku_id = input.sku_id.trim().to_string();
        self.price_cents = input.price_cents;
        self.featured = input.featured;
        self.visible = input.visible;
        self.sort_order = input.sort_order;
        self.updated_at = chrono::Utc::now().to_rfc3339();
    }
}

#[must_use]
pub fn format_price_cents(cents: Option<u64>) -> String {
    match cents {
        Some(c) => format!("${}.{:02}", group_thousands(c / 100), c % 100),
        None => String::new(),
    }
}

/// Render a whole-dollar amount with thousands separators (e.g. `175000` -> `175,000`).
fn group_thousands(dollars: u64) -> String {
    let digits = dollars.to_string();
    let mut grouped = String::with_capacity(digits.len() + digits.len() / 3);
    for (i, ch) in digits.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            grouped.push(',');
        }
        grouped.push(ch);
    }
    grouped.chars().rev().collect()
}

#[must_use]
pub fn price_cents_to_form(cents: Option<u64>) -> String {
    match cents {
        Some(c) => format!("{:.2}", c as f64 / 100.0),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_price_cents_dollars_and_cents() {
        assert_eq!(parse_price_cents("19.99").unwrap(), Some(1999));
        assert_eq!(parse_price_cents("5").unwrap(), Some(500));
        assert_eq!(parse_price_cents("").unwrap(), None);
    }

    #[test]
    fn format_price_cents_groups_thousands() {
        assert_eq!(format_price_cents(Some(1999)), "$19.99");
        assert_eq!(format_price_cents(Some(17_500_000)), "$175,000.00");
        assert_eq!(format_price_cents(Some(100)), "$1.00");
        assert_eq!(format_price_cents(None), "");
    }

    #[test]
    fn deposit_is_half_of_list_price() {
        assert_eq!(deposit_cents_for_price(17_500_000), 8_750_000);
        assert_eq!(deposit_cents_for_price(1999), 999);
    }
}
