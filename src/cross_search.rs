// src/cross_search.rs
// Aluminum OS — Cross-Provider Search Aggregator
//
// Novel Invention #6 — Cross-Provider Fan-Out Semantic Search
//
// A single search query fans out to every connected provider simultaneously,
// aggregates results, scores them, and returns a unified ranked response.
// The user never has to know which provider holds the information.
//
// This is the "search" command surface for the entire Aluminum OS:
//   uws search query --params '{"q":"Q4 budget"}'
//   uws search query --params '{"q":"bug fix","providers":"github,linear,notion","limit":"20"}'
//
// The aggregator is pure (no I/O) — it transforms provider-specific results
// into `SearchHit` structs. The actual HTTP fan-out lives in the executor.
//
// Author: GitHub Copilot — Aluminum OS Session 2026-03-23

#![allow(dead_code)]

use std::collections::BTreeMap;

// ─── Provider identifier ──────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SearchProvider {
    GitHub,
    Slack,
    Linear,
    Notion,
    Gmail,
    MsTeams,
    GoogleDrive,
    OneDrive,
    Figma,
}

impl SearchProvider {
    /// Parse a comma-separated provider list (from --params "providers").
    pub fn parse_list(s: &str) -> Vec<SearchProvider> {
        s.split(',')
            .filter_map(|p| match p.trim() {
                "github" => Some(SearchProvider::GitHub),
                "slack" => Some(SearchProvider::Slack),
                "linear" => Some(SearchProvider::Linear),
                "notion" => Some(SearchProvider::Notion),
                "gmail" => Some(SearchProvider::Gmail),
                "ms-teams" | "teams" => Some(SearchProvider::MsTeams),
                "drive" | "google-drive" => Some(SearchProvider::GoogleDrive),
                "onedrive" | "ms-onedrive" => Some(SearchProvider::OneDrive),
                "figma" => Some(SearchProvider::Figma),
                _ => None,
            })
            .collect()
    }

    pub fn as_str(&self) -> &str {
        match self {
            SearchProvider::GitHub => "github",
            SearchProvider::Slack => "slack",
            SearchProvider::Linear => "linear",
            SearchProvider::Notion => "notion",
            SearchProvider::Gmail => "gmail",
            SearchProvider::MsTeams => "ms-teams",
            SearchProvider::GoogleDrive => "google-drive",
            SearchProvider::OneDrive => "onedrive",
            SearchProvider::Figma => "figma",
        }
    }

    /// All providers supported by the cross-search engine.
    pub fn all() -> Vec<SearchProvider> {
        vec![
            SearchProvider::GitHub,
            SearchProvider::Slack,
            SearchProvider::Linear,
            SearchProvider::Notion,
            SearchProvider::Gmail,
            SearchProvider::MsTeams,
            SearchProvider::GoogleDrive,
            SearchProvider::OneDrive,
            SearchProvider::Figma,
        ]
    }
}

// ─── Search result ────────────────────────────────────────────────────────

/// A single search hit from any provider, normalized to a common format.
#[derive(Debug, Clone)]
pub struct SearchHit {
    /// Which provider returned this result.
    pub provider: SearchProvider,
    /// The unique identifier within that provider.
    pub id: String,
    /// Human-readable title or subject.
    pub title: String,
    /// Brief preview/snippet of the content.
    pub snippet: String,
    /// Direct URL to the result.
    pub url: String,
    /// ISO 8601 timestamp of last modification.
    pub updated_at: String,
    /// Relevance score (0.0 – 1.0). Higher is more relevant.
    pub score: f64,
    /// Provider-specific metadata.
    pub metadata: BTreeMap<String, String>,
}

impl SearchHit {
    /// Serialize to a flat JSON-compatible string map for output.
    pub fn to_map(&self) -> BTreeMap<String, String> {
        let mut m = BTreeMap::new();
        m.insert("provider".to_string(), self.provider.as_str().to_string());
        m.insert("id".to_string(), self.id.clone());
        m.insert("title".to_string(), self.title.clone());
        m.insert("snippet".to_string(), self.snippet.clone());
        m.insert("url".to_string(), self.url.clone());
        m.insert("updated_at".to_string(), self.updated_at.clone());
        m.insert("score".to_string(), format!("{:.3}", self.score));
        for (k, v) in &self.metadata {
            m.insert(format!("meta.{k}"), v.clone());
        }
        m
    }
}

// ─── Search request ───────────────────────────────────────────────────────

/// Parameters for a cross-provider search.
#[derive(Debug, Clone)]
pub struct SearchRequest {
    pub query: String,
    pub providers: Vec<SearchProvider>,
    pub limit_per_provider: usize,
    pub total_limit: usize,
}

impl SearchRequest {
    /// Build a search request from CLI `--params` map.
    ///
    /// Recognized keys:
    /// - `q` or `query` — the search query (required)
    /// - `providers` — comma-separated list (default: all)
    /// - `limit` — max total results (default: 50)
    /// - `limit_per_provider` — max results per provider (default: 10)
    pub fn from_params(params: &BTreeMap<String, String>) -> Result<SearchRequest, String> {
        let query = params
            .get("q")
            .or_else(|| params.get("query"))
            .cloned()
            .ok_or_else(|| "Missing required param: 'q' (search query)".to_string())?;

        if query.trim().is_empty() {
            return Err("Search query 'q' cannot be empty".to_string());
        }

        let providers = match params.get("providers") {
            Some(p) => SearchProvider::parse_list(p),
            None => SearchProvider::all(),
        };

        let limit_per_provider = params
            .get("limit_per_provider")
            .and_then(|s| s.parse().ok())
            .unwrap_or(10usize);

        let total_limit = params
            .get("limit")
            .and_then(|s| s.parse().ok())
            .unwrap_or(50usize);

        Ok(SearchRequest {
            query,
            providers,
            limit_per_provider,
            total_limit,
        })
    }
}

// ─── Result aggregation ───────────────────────────────────────────────────

/// Aggregate and rank search hits from multiple providers.
///
/// Deduplicates by URL, sorts by score descending, then applies `total_limit`.
pub fn aggregate_results(mut hits: Vec<SearchHit>, total_limit: usize) -> Vec<SearchHit> {
    // Deduplicate by URL
    let mut seen_urls = std::collections::HashSet::new();
    hits.retain(|h| seen_urls.insert(h.url.clone()));

    // Sort by score descending, then by provider (stable)
    hits.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.provider.cmp(&b.provider))
    });

    hits.truncate(total_limit);
    hits
}

/// Build a per-provider sub-query URL for the given provider.
/// This is the fan-out step — each provider gets its own query URL.
pub fn build_provider_query_url(provider: &SearchProvider, query: &str, limit: usize) -> String {
    let encoded_q = query.replace(' ', "+");
    match provider {
        SearchProvider::GitHub => {
            format!("https://api.github.com/search/repositories?q={encoded_q}&per_page={limit}")
        }
        SearchProvider::Slack => {
            format!("https://slack.com/api/search.all?query={encoded_q}&count={limit}")
        }
        SearchProvider::Linear => {
            // Linear uses GraphQL; return the endpoint URL — variables are in the body
            "https://api.linear.app/graphql".to_string()
        }
        SearchProvider::Notion => {
            "https://api.notion.com/v1/search".to_string()
        }
        SearchProvider::Gmail => {
            format!("https://www.googleapis.com/gmail/v1/users/me/messages?q={encoded_q}&maxResults={limit}")
        }
        SearchProvider::MsTeams => {
            "https://graph.microsoft.com/v1.0/search/query".to_string()
        }
        SearchProvider::GoogleDrive => {
            format!("https://www.googleapis.com/drive/v3/files?q=fullText+contains+%27{encoded_q}%27&pageSize={limit}")
        }
        SearchProvider::OneDrive => {
            format!("https://graph.microsoft.com/v1.0/me/drive/root/search(q='{encoded_q}')?$top={limit}")
        }
        SearchProvider::Figma => {
            format!("https://api.figma.com/v1/search?query={encoded_q}")
        }
    }
}

// ─── Unit tests ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_provider_list() {
        let providers = SearchProvider::parse_list("github,slack,linear");
        assert_eq!(providers.len(), 3);
        assert!(providers.contains(&SearchProvider::GitHub));
        assert!(providers.contains(&SearchProvider::Slack));
        assert!(providers.contains(&SearchProvider::Linear));
    }

    #[test]
    fn test_parse_provider_list_unknown_skipped() {
        let providers = SearchProvider::parse_list("github,unknown_provider,slack");
        assert_eq!(providers.len(), 2);
    }

    #[test]
    fn test_all_providers_list() {
        let all = SearchProvider::all();
        assert_eq!(all.len(), 9);
    }

    #[test]
    fn test_search_request_from_params_basic() {
        let mut params = BTreeMap::new();
        params.insert("q".to_string(), "budget Q4".to_string());
        let req = SearchRequest::from_params(&params).unwrap();
        assert_eq!(req.query, "budget Q4");
        assert_eq!(req.providers.len(), 9); // all providers
        assert_eq!(req.total_limit, 50);
    }

    #[test]
    fn test_search_request_custom_providers() {
        let mut params = BTreeMap::new();
        params.insert("q".to_string(), "open issues".to_string());
        params.insert("providers".to_string(), "github,linear".to_string());
        params.insert("limit".to_string(), "20".to_string());
        let req = SearchRequest::from_params(&params).unwrap();
        assert_eq!(req.providers.len(), 2);
        assert_eq!(req.total_limit, 20);
    }

    #[test]
    fn test_search_request_missing_query_returns_error() {
        let params = BTreeMap::new();
        assert!(SearchRequest::from_params(&params).is_err());
    }

    #[test]
    fn test_search_request_empty_query_returns_error() {
        let mut params = BTreeMap::new();
        params.insert("q".to_string(), "  ".to_string());
        assert!(SearchRequest::from_params(&params).is_err());
    }

    #[test]
    fn test_aggregate_deduplicates_by_url() {
        let hit = SearchHit {
            provider: SearchProvider::GitHub,
            id: "1".to_string(),
            title: "Test".to_string(),
            snippet: "snippet".to_string(),
            url: "https://example.com/1".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
            score: 0.9,
            metadata: BTreeMap::new(),
        };
        let hits = vec![hit.clone(), hit];
        let aggregated = aggregate_results(hits, 100);
        assert_eq!(aggregated.len(), 1);
    }

    #[test]
    fn test_aggregate_sorts_by_score_descending() {
        let make_hit = |score: f64, url: &str| SearchHit {
            provider: SearchProvider::GitHub,
            id: url.to_string(),
            title: "T".to_string(),
            snippet: "S".to_string(),
            url: url.to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
            score,
            metadata: BTreeMap::new(),
        };
        let hits = vec![make_hit(0.3, "a"), make_hit(0.9, "b"), make_hit(0.6, "c")];
        let aggregated = aggregate_results(hits, 10);
        assert!(aggregated[0].score >= aggregated[1].score);
        assert!(aggregated[1].score >= aggregated[2].score);
    }

    #[test]
    fn test_aggregate_respects_total_limit() {
        let hits: Vec<SearchHit> = (0..20)
            .map(|i| SearchHit {
                provider: SearchProvider::GitHub,
                id: i.to_string(),
                title: format!("Hit {i}"),
                snippet: "s".to_string(),
                url: format!("https://example.com/{i}"),
                updated_at: "2026-01-01T00:00:00Z".to_string(),
                score: i as f64 / 20.0,
                metadata: BTreeMap::new(),
            })
            .collect();
        let aggregated = aggregate_results(hits, 5);
        assert_eq!(aggregated.len(), 5);
    }

    #[test]
    fn test_search_hit_to_map() {
        let mut meta = BTreeMap::new();
        meta.insert("repo".to_string(), "octocat/Hello-World".to_string());
        let hit = SearchHit {
            provider: SearchProvider::GitHub,
            id: "123".to_string(),
            title: "Hello World".to_string(),
            snippet: "A repo".to_string(),
            url: "https://github.com/octocat/Hello-World".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
            score: 0.95,
            metadata: meta,
        };
        let map = hit.to_map();
        assert_eq!(map["provider"], "github");
        assert_eq!(map["score"], "0.950");
        assert!(map.contains_key("meta.repo"));
    }

    #[test]
    fn test_build_provider_query_url_github() {
        let url = build_provider_query_url(&SearchProvider::GitHub, "hello world", 10);
        assert!(url.contains("api.github.com/search"));
        assert!(url.contains("hello+world"));
        assert!(url.contains("per_page=10"));
    }

    #[test]
    fn test_build_provider_query_url_notion() {
        let url = build_provider_query_url(&SearchProvider::Notion, "notes", 5);
        assert_eq!(url, "https://api.notion.com/v1/search");
    }

    #[test]
    fn test_provider_as_str() {
        assert_eq!(SearchProvider::GitHub.as_str(), "github");
        assert_eq!(SearchProvider::MsTeams.as_str(), "ms-teams");
        assert_eq!(SearchProvider::GoogleDrive.as_str(), "google-drive");
    }
}
