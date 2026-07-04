//! Build spec sheets from the racer repository (vendored under `specs/`).

use pulldown_cmark::{Options, Parser, html};

const SIGMA_RACER_SKU: &str = "SIGMA-RACER";

const CHASSIS_SPEC: &str = include_str!("../specs/sigma-spec.md");
const ELECTRONICS_SPEC: &str = include_str!("../specs/electronics.md");
const EMISSIONS_SPEC: &str = include_str!("../specs/emissions_certification.md");

struct SpecSource {
    id: &'static str,
    label: &'static str,
    markdown: &'static str,
}

const RACER_SPEC_SOURCES: &[SpecSource] = &[
    SpecSource {
        id: "chassis",
        label: "Chassis",
        markdown: CHASSIS_SPEC,
    },
    SpecSource {
        id: "electronics",
        label: "Electronics",
        markdown: ELECTRONICS_SPEC,
    },
    SpecSource {
        id: "emissions",
        label: "Emissions",
        markdown: EMISSIONS_SPEC,
    },
];

/// One racer repo document rendered for the product page.
pub struct SpecDocumentView {
    pub id: String,
    pub label: String,
    pub html: String,
}

/// Load racer build specs for a storefront SKU, when available.
#[must_use]
pub fn specs_for_sku(sku_code: &str) -> Vec<SpecDocumentView> {
    if sku_code != SIGMA_RACER_SKU {
        return Vec::new();
    }
    RACER_SPEC_SOURCES
        .iter()
        .map(|source| SpecDocumentView {
            id: source.id.to_string(),
            label: source.label.to_string(),
            html: render_markdown_html(source.markdown),
        })
        .collect()
}

fn render_markdown_html(markdown: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_STRIKETHROUGH);

    let parser = Parser::new_ext(markdown, options);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    html_output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sigma_racer_has_three_tabbed_specs() {
        let docs = specs_for_sku("SIGMA-RACER");
        assert_eq!(docs.len(), 3);
        assert_eq!(docs[0].id, "chassis");
        assert_eq!(docs[0].label, "Chassis");
        assert_eq!(docs[1].id, "electronics");
        assert_eq!(docs[2].id, "emissions");
        assert!(docs[0].html.contains("<table"));
        assert!(docs[0].html.contains("Powertrain"));
        assert!(docs[1].html.contains("STM32"));
    }

    #[test]
    fn other_skus_have_no_specs() {
        assert!(specs_for_sku("OTHER").is_empty());
    }

    #[test]
    fn renders_markdown_table() {
        let md = "## Test\n\n| Item | Spec |\n|---|---|\n| Engine | Yamaha |\n";
        let html = render_markdown_html(md);
        assert!(html.contains("<table"));
        assert!(html.contains("Engine"));
        assert!(html.contains("Yamaha"));
    }
}
