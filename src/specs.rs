//! Build spec sheets from the racer repository (vendored under `specs/`).

use pulldown_cmark::{Options, Parser, html};

const SIGMA_RACER_SKU: &str = "SIGMA-RACER";

const CHASSIS_SPEC: &str = include_str!("../specs/sigma-spec.md");
const ELECTRONICS_SPEC: &str = include_str!("../specs/electronics.md");
const EMISSIONS_SPEC: &str = include_str!("../specs/emissions_certification.md");

/// Markdown rendered to HTML for the product page (GitHub-style tables, headings, lists).
pub struct SpecDocumentView {
    pub html: String,
}

/// Load racer build specs for a storefront SKU, when available.
#[must_use]
pub fn specs_for_sku(sku_code: &str) -> Vec<SpecDocumentView> {
    if sku_code != SIGMA_RACER_SKU {
        return Vec::new();
    }
    [CHASSIS_SPEC, ELECTRONICS_SPEC, EMISSIONS_SPEC]
        .into_iter()
        .map(render_markdown_document)
        .collect()
}

fn render_markdown_document(markdown: &str) -> SpecDocumentView {
    SpecDocumentView {
        html: render_markdown_html(markdown),
    }
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
    fn sigma_racer_has_rendered_specs() {
        let docs = specs_for_sku("SIGMA-RACER");
        assert_eq!(docs.len(), 3);
        assert!(docs[0].html.contains("<table"));
        assert!(docs[0].html.contains("Powertrain"));
        assert!(docs[0].html.contains("Yamaha CP3"));
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
