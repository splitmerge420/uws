// src/notion_provider.rs
// Aluminum OS — Notion Provider
//
// Exposes the Notion REST API through the `uws notion` command surface.
// Notion is the documentation and knowledge management platform; this
// provider makes pages, databases, blocks, and search queryable by any
// AI agent with the same clean JSON grammar.
//
// Command grammar:
//   uws notion <resource> <method> [--params <JSON>] [--json <JSON>] [--dry-run]
//
// Examples:
//   uws notion pages get --params '{"page_id":"abc123"}'
//   uws notion pages create --json '{"parent":{"database_id":"db123"},"properties":{"Name":{"title":[{"text":{"content":"New Page"}}]}}}'
//   uws notion databases list
//   uws notion databases query --params '{"database_id":"db123"}'
//   uws notion search query --params '{"query":"meeting notes"}'
//   uws notion blocks list --params '{"block_id":"abc123"}'
//   uws notion users list
//
// Authentication:
//   Set NOTION_API_KEY or UWS_NOTION_TOKEN in the environment.
//   Requires a Notion integration token (Internal Integration Token).
//
// Author: GitHub Copilot — Aluminum OS Session 2026-03-23

#![allow(dead_code)]

use std::collections::BTreeMap;

// ─── Service aliases ──────────────────────────────────────────────────────

pub const NOTION_ALIASES: &[&str] = &["notion"];

pub fn is_notion_service(name: &str) -> bool {
    NOTION_ALIASES.contains(&name)
}

// ─── API base ─────────────────────────────────────────────────────────────

pub const NOTION_API_BASE: &str = "https://api.notion.com/v1";
pub const NOTION_API_VERSION: &str = "2022-06-28";

// ─── HTTP methods ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotionHttpMethod {
    Get,
    Post,
    Patch,
    Delete,
}

impl NotionHttpMethod {
    pub fn as_str(&self) -> &str {
        match self {
            NotionHttpMethod::Get => "GET",
            NotionHttpMethod::Post => "POST",
            NotionHttpMethod::Patch => "PATCH",
            NotionHttpMethod::Delete => "DELETE",
        }
    }
}

// ─── Endpoint catalogue ───────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct NotionEndpoint {
    pub resource: &'static str,
    pub method: &'static str,
    pub http_method: NotionHttpMethod,
    /// Path template; {page_id}, {database_id}, {block_id} are substituted from params.
    pub path_template: &'static str,
    pub requires_body: bool,
    pub description: &'static str,
    pub path_params: &'static [&'static str],
}

pub const NOTION_ENDPOINTS: &[NotionEndpoint] = &[
    // ── Pages ────────────────────────────────────────────────────────
    NotionEndpoint {
        resource: "pages",
        method: "get",
        http_method: NotionHttpMethod::Get,
        path_template: "/pages/{page_id}",
        requires_body: false,
        description: "Retrieve a page by ID",
        path_params: &["page_id"],
    },
    NotionEndpoint {
        resource: "pages",
        method: "create",
        http_method: NotionHttpMethod::Post,
        path_template: "/pages",
        requires_body: true,
        description: "Create a new page",
        path_params: &[],
    },
    NotionEndpoint {
        resource: "pages",
        method: "update",
        http_method: NotionHttpMethod::Patch,
        path_template: "/pages/{page_id}",
        requires_body: true,
        description: "Update page properties",
        path_params: &["page_id"],
    },
    NotionEndpoint {
        resource: "pages",
        method: "archive",
        http_method: NotionHttpMethod::Patch,
        path_template: "/pages/{page_id}",
        requires_body: true,
        description: "Archive (soft-delete) a page",
        path_params: &["page_id"],
    },
    NotionEndpoint {
        resource: "pages",
        method: "properties",
        http_method: NotionHttpMethod::Get,
        path_template: "/pages/{page_id}/properties/{property_id}",
        requires_body: false,
        description: "Get a page property value",
        path_params: &["page_id", "property_id"],
    },
    // ── Databases ────────────────────────────────────────────────────
    NotionEndpoint {
        resource: "databases",
        method: "list",
        http_method: NotionHttpMethod::Post,
        path_template: "/search",
        requires_body: false,
        description: "List databases accessible to the integration",
        path_params: &[],
    },
    NotionEndpoint {
        resource: "databases",
        method: "get",
        http_method: NotionHttpMethod::Get,
        path_template: "/databases/{database_id}",
        requires_body: false,
        description: "Get a database by ID",
        path_params: &["database_id"],
    },
    NotionEndpoint {
        resource: "databases",
        method: "create",
        http_method: NotionHttpMethod::Post,
        path_template: "/databases",
        requires_body: true,
        description: "Create a new database",
        path_params: &[],
    },
    NotionEndpoint {
        resource: "databases",
        method: "update",
        http_method: NotionHttpMethod::Patch,
        path_template: "/databases/{database_id}",
        requires_body: true,
        description: "Update a database schema",
        path_params: &["database_id"],
    },
    NotionEndpoint {
        resource: "databases",
        method: "query",
        http_method: NotionHttpMethod::Post,
        path_template: "/databases/{database_id}/query",
        requires_body: false,
        description: "Query a database for pages matching filters",
        path_params: &["database_id"],
    },
    // ── Blocks ───────────────────────────────────────────────────────
    NotionEndpoint {
        resource: "blocks",
        method: "get",
        http_method: NotionHttpMethod::Get,
        path_template: "/blocks/{block_id}",
        requires_body: false,
        description: "Get a block by ID",
        path_params: &["block_id"],
    },
    NotionEndpoint {
        resource: "blocks",
        method: "list",
        http_method: NotionHttpMethod::Get,
        path_template: "/blocks/{block_id}/children",
        requires_body: false,
        description: "List the children of a block (page content)",
        path_params: &["block_id"],
    },
    NotionEndpoint {
        resource: "blocks",
        method: "append",
        http_method: NotionHttpMethod::Patch,
        path_template: "/blocks/{block_id}/children",
        requires_body: true,
        description: "Append blocks to a page or block",
        path_params: &["block_id"],
    },
    NotionEndpoint {
        resource: "blocks",
        method: "update",
        http_method: NotionHttpMethod::Patch,
        path_template: "/blocks/{block_id}",
        requires_body: true,
        description: "Update a block's content",
        path_params: &["block_id"],
    },
    NotionEndpoint {
        resource: "blocks",
        method: "delete",
        http_method: NotionHttpMethod::Delete,
        path_template: "/blocks/{block_id}",
        requires_body: false,
        description: "Delete (archive) a block",
        path_params: &["block_id"],
    },
    // ── Users ────────────────────────────────────────────────────────
    NotionEndpoint {
        resource: "users",
        method: "list",
        http_method: NotionHttpMethod::Get,
        path_template: "/users",
        requires_body: false,
        description: "List all users in the workspace",
        path_params: &[],
    },
    NotionEndpoint {
        resource: "users",
        method: "get",
        http_method: NotionHttpMethod::Get,
        path_template: "/users/{user_id}",
        requires_body: false,
        description: "Get a user by ID",
        path_params: &["user_id"],
    },
    NotionEndpoint {
        resource: "users",
        method: "me",
        http_method: NotionHttpMethod::Get,
        path_template: "/users/me",
        requires_body: false,
        description: "Get the authenticated bot user",
        path_params: &[],
    },
    // ── Search ───────────────────────────────────────────────────────
    NotionEndpoint {
        resource: "search",
        method: "query",
        http_method: NotionHttpMethod::Post,
        path_template: "/search",
        requires_body: false,
        description: "Search all pages and databases in the workspace",
        path_params: &[],
    },
    // ── Comments ─────────────────────────────────────────────────────
    NotionEndpoint {
        resource: "comments",
        method: "list",
        http_method: NotionHttpMethod::Get,
        path_template: "/comments",
        requires_body: false,
        description: "List comments on a page or block",
        path_params: &[],
    },
    NotionEndpoint {
        resource: "comments",
        method: "create",
        http_method: NotionHttpMethod::Post,
        path_template: "/comments",
        requires_body: true,
        description: "Add a comment to a page or block",
        path_params: &[],
    },
];

// ─── Error type ──────────────────────────────────────────────────────────

#[derive(Debug)]
pub enum NotionError {
    EndpointNotFound { resource: String, method: String },
    TokenNotFound,
    ParseError(String),
}

impl std::fmt::Display for NotionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NotionError::EndpointNotFound { resource, method } => {
                write!(f, "Unknown Notion endpoint: {resource} {method}")
            }
            NotionError::TokenNotFound => write!(
                f,
                "Notion token not found. Set NOTION_API_KEY or UWS_NOTION_TOKEN."
            ),
            NotionError::ParseError(s) => write!(f, "Parse error: {s}"),
        }
    }
}

// ─── Token resolution ─────────────────────────────────────────────────────

pub fn resolve_notion_token() -> Option<String> {
    std::env::var("UWS_NOTION_TOKEN")
        .ok()
        .or_else(|| std::env::var("NOTION_API_KEY").ok())
}

// ─── Path parameter substitution ─────────────────────────────────────────

/// Substitute `{param}` tokens in a path template from `params`.
/// Returns `(url, remaining_query_params)`.
pub fn build_url(
    path_template: &str,
    params: &BTreeMap<String, String>,
) -> (String, BTreeMap<String, String>) {
    let mut path = path_template.to_string();
    let mut remaining = BTreeMap::new();

    for (k, v) in params {
        let token = format!("{{{k}}}");
        if path.contains(&token) {
            path = path.replace(&token, v);
        } else {
            remaining.insert(k.clone(), v.clone());
        }
    }

    let full_url = format!("{NOTION_API_BASE}{path}");
    (full_url, remaining)
}

// ─── Request builder ──────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct NotionRequest {
    pub http_method: String,
    pub url: String,
    pub query_params: BTreeMap<String, String>,
    pub body: Option<BTreeMap<String, String>>,
    pub auth_header: String,
    pub notion_version: String,
    pub dry_run: bool,
}

pub fn build_request(
    endpoint: &NotionEndpoint,
    params: BTreeMap<String, String>,
    body: Option<BTreeMap<String, String>>,
    token: &str,
    dry_run: bool,
) -> NotionRequest {
    let (url, query_params) = build_url(endpoint.path_template, &params);
    let auth_token = if dry_run { "[REDACTED]".to_string() } else { token.to_string() };

    NotionRequest {
        http_method: endpoint.http_method.as_str().to_string(),
        url,
        query_params,
        body,
        auth_header: format!("Bearer {auth_token}"),
        notion_version: NOTION_API_VERSION.to_string(),
        dry_run,
    }
}

// ─── Dry-run output ──────────────────────────────────────────────────────

pub fn format_dry_run(req: &NotionRequest, endpoint: &NotionEndpoint) -> String {
    let mut parts = vec![
        format!("\"method\": \"{}\"", req.http_method),
        format!("\"url\": \"{}\"", req.url),
        format!("\"notion-version\": \"{}\"", req.notion_version),
        format!("\"description\": \"{}\"", endpoint.description),
    ];
    if !req.query_params.is_empty() {
        let ps: Vec<String> = req.query_params.iter()
            .map(|(k, v)| format!("\"{k}\": \"{v}\""))
            .collect();
        parts.push(format!("\"queryParams\": {{{}}}", ps.join(", ")));
    }
    if let Some(b) = &req.body {
        let bs: Vec<String> = b.iter().map(|(k, v)| format!("\"{k}\": \"{v}\"")).collect();
        parts.push(format!("\"body\": {{{}}}", bs.join(", ")));
    }
    format!("{{\n  {}\n}}", parts.join(",\n  "))
}

// ─── Endpoint lookup ──────────────────────────────────────────────────────

pub fn find_endpoint(resource: &str, method: &str) -> Option<&'static NotionEndpoint> {
    NOTION_ENDPOINTS
        .iter()
        .find(|e| e.resource == resource && e.method == method)
}

// ─── Unit tests ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_notion_service() {
        assert!(is_notion_service("notion"));
        assert!(!is_notion_service("slack"));
        assert!(!is_notion_service("linear"));
    }

    #[test]
    fn test_endpoint_catalogue_nonempty() {
        assert!(!NOTION_ENDPOINTS.is_empty());
    }

    #[test]
    fn test_find_pages_get() {
        let ep = find_endpoint("pages", "get").unwrap();
        assert_eq!(ep.http_method, NotionHttpMethod::Get);
        assert!(ep.path_template.contains("{page_id}"));
    }

    #[test]
    fn test_find_databases_query() {
        let ep = find_endpoint("databases", "query").unwrap();
        assert_eq!(ep.http_method, NotionHttpMethod::Post);
        assert!(ep.path_template.contains("{database_id}"));
    }

    #[test]
    fn test_find_unknown_returns_none() {
        assert!(find_endpoint("nonexistent", "list").is_none());
    }

    #[test]
    fn test_build_url_substitutes_path_params() {
        let mut params = BTreeMap::new();
        params.insert("page_id".to_string(), "abc123".to_string());
        params.insert("extra".to_string(), "value".to_string());
        let (url, remaining) = build_url("/pages/{page_id}", &params);
        assert_eq!(url, "https://api.notion.com/v1/pages/abc123");
        assert_eq!(remaining.get("extra").map(|s| s.as_str()), Some("value"));
        assert!(!remaining.contains_key("page_id"));
    }

    #[test]
    fn test_build_url_no_substitution_needed() {
        let params = BTreeMap::new();
        let (url, remaining) = build_url("/users", &params);
        assert_eq!(url, "https://api.notion.com/v1/users");
        assert!(remaining.is_empty());
    }

    #[test]
    fn test_build_request_sets_notion_version() {
        let ep = find_endpoint("users", "list").unwrap();
        let req = build_request(ep, BTreeMap::new(), None, "secret_token", false);
        assert_eq!(req.notion_version, "2022-06-28");
    }

    #[test]
    fn test_build_request_dry_run_redacts_token() {
        let ep = find_endpoint("pages", "create").unwrap();
        let req = build_request(ep, BTreeMap::new(), None, "secret_notion_token", true);
        assert!(req.auth_header.contains("[REDACTED]"));
        assert!(!req.auth_header.contains("secret_notion_token"));
    }

    #[test]
    fn test_format_dry_run() {
        let ep = find_endpoint("search", "query").unwrap();
        let mut params = BTreeMap::new();
        params.insert("query".to_string(), "meeting notes".to_string());
        let req = build_request(ep, params, None, "token", true);
        let out = format_dry_run(&req, ep);
        assert!(out.contains("2022-06-28"));
        assert!(out.contains("search"));
    }

    #[test]
    fn test_blocks_endpoints_present() {
        assert!(find_endpoint("blocks", "get").is_some());
        assert!(find_endpoint("blocks", "list").is_some());
        assert!(find_endpoint("blocks", "append").is_some());
        assert!(find_endpoint("blocks", "delete").is_some());
    }

    #[test]
    fn test_error_display() {
        let e = NotionError::TokenNotFound;
        assert!(e.to_string().contains("NOTION_API_KEY"));
        let e2 = NotionError::EndpointNotFound {
            resource: "pages".to_string(),
            method: "xyz".to_string(),
        };
        assert!(e2.to_string().contains("pages xyz"));
    }

    #[test]
    fn test_notion_api_base() {
        assert_eq!(NOTION_API_BASE, "https://api.notion.com/v1");
    }

    #[test]
    fn test_unique_resource_method_pairs() {
        let mut seen = std::collections::HashSet::new();
        for ep in NOTION_ENDPOINTS {
            let key = format!("{}/{}", ep.resource, ep.method);
            assert!(seen.insert(key.clone()), "Duplicate endpoint: {key}");
        }
    }

    #[test]
    fn test_comments_endpoints_present() {
        assert!(find_endpoint("comments", "list").is_some());
        assert!(find_endpoint("comments", "create").is_some());
    }
}
