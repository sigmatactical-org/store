mod create_listing;
mod listing;
mod listing_form;
mod update_listing;
pub use create_listing::CreateListing;
pub use listing::Listing;
pub use listing_form::ListingForm;
pub use update_listing::UpdateListing;

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
}
