//! Build spec sheets fetched from the racer GitHub repository at request time.

use std::sync::OnceLock;
use std::time::{Duration, Instant};

use pulldown_cmark::{Options, Parser, html};
use serde::Deserialize;
use thiserror::Error;
use tokio::sync::RwLock;

use crate::config;

const SIGMA_RACER_SKU: &str = "SIGMA-RACER";

/// Preferred tab order; unknown documents are appended alphabetically by label.
const SPEC_TAB_ORDER: &[&str] = &[
    "overview",
    "build",
    "engine",
    "chassis",
    "bodywork",
    "electrical",
    "electronics",
    "efi",
    "emissions",
];

/// One racer repo document rendered for the product page.
#[derive(Debug, Clone)]
pub struct SpecDocumentView {
    pub id: String,
    pub label: String,
    pub html: String,
}

#[derive(Debug, Error)]
enum SpecsError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("racer specs request failed: {0}")]
    Request(String),
}

#[derive(Debug, Deserialize)]
struct GithubContentEntry {
    name: String,
    download_url: Option<String>,
}

struct CacheState {
    documents: Option<Vec<SpecDocumentView>>,
    fetched_at: Option<Instant>,
}

impl CacheState {
    const fn empty() -> Self {
        Self {
            documents: None,
            fetched_at: None,
        }
    }

    fn is_fresh(&self, ttl: Duration) -> bool {
        self.documents
            .as_ref()
            .is_some_and(|_| self.fetched_at.is_some_and(|at| at.elapsed() < ttl))
    }
}

struct SpecsCache {
    client: reqwest::Client,
    state: RwLock<CacheState>,
}

impl SpecsCache {
    fn global() -> &'static SpecsCache {
        static CACHE: OnceLock<SpecsCache> = OnceLock::new();
        CACHE.get_or_init(|| SpecsCache {
            client: reqwest::Client::new(),
            state: RwLock::new(CacheState::empty()),
        })
    }
}

/// Load racer build specs for a storefront SKU, when available.
pub async fn specs_for_sku(sku_code: &str) -> Vec<SpecDocumentView> {
    if sku_code != SIGMA_RACER_SKU {
        return Vec::new();
    }
    let (owner, repo) = config::racer_specs_repo();

    let cache = SpecsCache::global();
    let ttl = config::racer_specs_cache_ttl();

    {
        let state = cache.state.read().await;
        if state.is_fresh(ttl) {
            return state.documents.clone().unwrap_or_default();
        }
    }

    match fetch_racer_specs(&cache.client, &owner, &repo, &config::racer_specs_ref()).await {
        Ok(documents) => {
            let mut state = cache.state.write().await;
            state.documents = Some(documents.clone());
            state.fetched_at = Some(Instant::now());
            documents
        }
        Err(_) => {
            let state = cache.state.read().await;
            state.documents.clone().unwrap_or_default()
        }
    }
}

async fn fetch_racer_specs(
    client: &reqwest::Client,
    owner: &str,
    repo: &str,
    git_ref: &str,
) -> Result<Vec<SpecDocumentView>, SpecsError> {
    let list_url = format!("https://api.github.com/repos/{owner}/{repo}/contents/?ref={git_ref}");
    let response = client
        .get(&list_url)
        .header("accept", "application/vnd.github+json")
        .header("user-agent", "sigma-store")
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(SpecsError::Request(format!("list {status}: {body}")));
    }

    let entries: Vec<GithubContentEntry> = response.json().await?;
    let mut sources = Vec::new();

    for entry in entries {
        if !entry.name.ends_with(".md") {
            continue;
        }
        let Some(download_url) = entry.download_url else {
            continue;
        };
        let markdown = client
            .get(download_url)
            .header("user-agent", "sigma-store")
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;
        sources.push(SpecSource {
            id: spec_id_from_filename(&entry.name),
            label: spec_label_from_filename(&entry.name),
            markdown,
        });
    }

    sources.sort_by(compare_spec_sources);
    Ok(sources
        .into_iter()
        .map(|source| SpecDocumentView {
            id: source.id,
            label: source.label,
            html: render_markdown_html(&source.markdown),
        })
        .collect())
}

struct SpecSource {
    id: String,
    label: String,
    markdown: String,
}

fn spec_id_from_filename(filename: &str) -> String {
    match filename {
        "README.md" => "overview".to_string(),
        "emissions_certification.md" => "emissions".to_string(),
        other => other.trim_end_matches(".md").to_string(),
    }
}

fn spec_label_from_filename(filename: &str) -> String {
    match filename {
        "README.md" => "Overview".to_string(),
        "emissions_certification.md" => "Emissions".to_string(),
        other => title_from_snake_case(other.trim_end_matches(".md")),
    }
}

fn title_from_snake_case(value: &str) -> String {
    value
        .split('_')
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn compare_spec_sources(left: &SpecSource, right: &SpecSource) -> std::cmp::Ordering {
    let left_rank = SPEC_TAB_ORDER
        .iter()
        .position(|id| *id == left.id)
        .unwrap_or(usize::MAX);
    let right_rank = SPEC_TAB_ORDER
        .iter()
        .position(|id| *id == right.id)
        .unwrap_or(usize::MAX);
    left_rank
        .cmp(&right_rank)
        .then_with(|| left.label.cmp(&right.label))
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
    fn maps_repo_filenames_to_tab_ids() {
        assert_eq!(spec_id_from_filename("README.md"), "overview");
        assert_eq!(spec_id_from_filename("chassis.md"), "chassis");
        assert_eq!(
            spec_id_from_filename("emissions_certification.md"),
            "emissions"
        );
        assert_eq!(spec_label_from_filename("efi.md"), "Efi");
    }

    #[tokio::test]
    async fn other_skus_have_no_specs_without_fetch() {
        assert!(specs_for_sku("OTHER").await.is_empty());
    }

    #[test]
    fn renders_markdown_table() {
        let md = "## Test\n\n| Item | Spec |\n|---|---|\n| Engine | Yamaha |\n";
        let html = render_markdown_html(md);
        assert!(html.contains("<table"));
        assert!(html.contains("Engine"));
        assert!(html.contains("Yamaha"));
    }

    #[test]
    fn orders_known_tabs_before_unknown_alphabetically() {
        let mut sources = vec![
            SpecSource {
                id: "zzz".into(),
                label: "Zzz".into(),
                markdown: String::new(),
            },
            SpecSource {
                id: "engine".into(),
                label: "Engine".into(),
                markdown: String::new(),
            },
            SpecSource {
                id: "overview".into(),
                label: "Overview".into(),
                markdown: String::new(),
            },
        ];
        sources.sort_by(compare_spec_sources);
        assert_eq!(
            sources.iter().map(|s| s.id.as_str()).collect::<Vec<_>>(),
            vec!["overview", "engine", "zzz"]
        );
    }
}
