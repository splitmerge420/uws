// src/figma_provider.rs
// Aluminum OS — Figma Provider
//
// Exposes the Figma REST API through the `uws figma` command surface.
// Figma is the canonical design tool; this provider gives AI agents
// structured JSON access to files, components, comments, and exports —
// closing the design-to-code gap in the universal workspace.
//
// Command grammar:
//   uws figma <resource> <method> [--params <JSON>] [--json <JSON>] [--dry-run]
//
// Examples:
//   uws figma files get --params '{"file_key":"Abc123"}'
//   uws figma files export --params '{"file_key":"Abc123","ids":"1:2,1:3","format":"svg"}'
//   uws figma components list --params '{"file_key":"Abc123"}'
//   uws figma comments list --params '{"file_key":"Abc123"}'
//   uws figma comments post --params '{"file_key":"Abc123"}' --json '{"message":"Looks great!"}'
//   uws figma projects list --params '{"team_id":"TEAM123"}'
//   uws figma me get
//
// Authentication:
//   Set FIGMA_TOKEN or UWS_FIGMA_TOKEN in the environment.
//   Use a Figma Personal Access Token from figma.com/developers.
//
// Author: GitHub Copilot — Aluminum OS Session 2026-03-23

#![allow(dead_code)]

use std::collections::BTreeMap;

// ─── Service aliases ──────────────────────────────────────────────────────

pub const FIGMA_ALIASES: &[&str] = &["figma"];

pub fn is_figma_service(name: &str) -> bool {
    FIGMA_ALIASES.contains(&name)
}

// ─── API base ─────────────────────────────────────────────────────────────

pub const FIGMA_API_BASE: &str = "https://api.figma.com/v1";

// ─── HTTP methods ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FigmaHttpMethod {
    Get,
    Post,
    Delete,
}

impl FigmaHttpMethod {
    pub fn as_str(&self) -> &str {
        match self {
            FigmaHttpMethod::Get => "GET",
            FigmaHttpMethod::Post => "POST",
            FigmaHttpMethod::Delete => "DELETE",
        }
    }
}

// ─── Endpoint catalogue ───────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct FigmaEndpoint {
    pub resource: &'static str,
    pub method: &'static str,
    pub http_method: FigmaHttpMethod,
    pub path_template: &'static str,
    pub requires_body: bool,
    pub description: &'static str,
    pub path_params: &'static [&'static str],
}

pub const FIGMA_ENDPOINTS: &[FigmaEndpoint] = &[
    // ── Files ────────────────────────────────────────────────────────
    FigmaEndpoint {
        resource: "files",
        method: "get",
        http_method: FigmaHttpMethod::Get,
        path_template: "/files/{file_key}",
        requires_body: false,
        description: "Get a Figma file by key",
        path_params: &["file_key"],
    },
    FigmaEndpoint {
        resource: "files",
        method: "nodes",
        http_method: FigmaHttpMethod::Get,
        path_template: "/files/{file_key}/nodes",
        requires_body: false,
        description: "Get specific nodes within a file",
        path_params: &["file_key"],
    },
    FigmaEndpoint {
        resource: "files",
        method: "images",
        http_method: FigmaHttpMethod::Get,
        path_template: "/files/{file_key}/images",
        requires_body: false,
        description: "Get image references from a file",
        path_params: &["file_key"],
    },
    FigmaEndpoint {
        resource: "files",
        method: "export",
        http_method: FigmaHttpMethod::Get,
        path_template: "/images/{file_key}",
        requires_body: false,
        description: "Export nodes as images (svg, png, jpg, pdf)",
        path_params: &["file_key"],
    },
    FigmaEndpoint {
        resource: "files",
        method: "versions",
        http_method: FigmaHttpMethod::Get,
        path_template: "/files/{file_key}/versions",
        requires_body: false,
        description: "Get version history of a file",
        path_params: &["file_key"],
    },
    FigmaEndpoint {
        resource: "files",
        method: "styles",
        http_method: FigmaHttpMethod::Get,
        path_template: "/files/{file_key}/styles",
        requires_body: false,
        description: "List styles defined in a file",
        path_params: &["file_key"],
    },
    // ── Components ───────────────────────────────────────────────────
    FigmaEndpoint {
        resource: "components",
        method: "list",
        http_method: FigmaHttpMethod::Get,
        path_template: "/files/{file_key}/components",
        requires_body: false,
        description: "List all components in a file",
        path_params: &["file_key"],
    },
    FigmaEndpoint {
        resource: "components",
        method: "get",
        http_method: FigmaHttpMethod::Get,
        path_template: "/components/{component_key}",
        requires_body: false,
        description: "Get a component by key",
        path_params: &["component_key"],
    },
    FigmaEndpoint {
        resource: "components",
        method: "sets",
        http_method: FigmaHttpMethod::Get,
        path_template: "/files/{file_key}/component_sets",
        requires_body: false,
        description: "List component sets in a file",
        path_params: &["file_key"],
    },
    // ── Comments ─────────────────────────────────────────────────────
    FigmaEndpoint {
        resource: "comments",
        method: "list",
        http_method: FigmaHttpMethod::Get,
        path_template: "/files/{file_key}/comments",
        requires_body: false,
        description: "List all comments in a file",
        path_params: &["file_key"],
    },
    FigmaEndpoint {
        resource: "comments",
        method: "post",
        http_method: FigmaHttpMethod::Post,
        path_template: "/files/{file_key}/comments",
        requires_body: true,
        description: "Post a comment to a file",
        path_params: &["file_key"],
    },
    FigmaEndpoint {
        resource: "comments",
        method: "delete",
        http_method: FigmaHttpMethod::Delete,
        path_template: "/files/{file_key}/comments/{comment_id}",
        requires_body: false,
        description: "Delete a comment",
        path_params: &["file_key", "comment_id"],
    },
    FigmaEndpoint {
        resource: "comments",
        method: "reactions",
        http_method: FigmaHttpMethod::Get,
        path_template: "/files/{file_key}/comments/{comment_id}/reactions",
        requires_body: false,
        description: "List reactions on a comment",
        path_params: &["file_key", "comment_id"],
    },
    // ── Projects ─────────────────────────────────────────────────────
    FigmaEndpoint {
        resource: "projects",
        method: "list",
        http_method: FigmaHttpMethod::Get,
        path_template: "/teams/{team_id}/projects",
        requires_body: false,
        description: "List projects in a team",
        path_params: &["team_id"],
    },
    FigmaEndpoint {
        resource: "projects",
        method: "files",
        http_method: FigmaHttpMethod::Get,
        path_template: "/projects/{project_id}/files",
        requires_body: false,
        description: "List files in a project",
        path_params: &["project_id"],
    },
    // ── Teams ────────────────────────────────────────────────────────
    FigmaEndpoint {
        resource: "teams",
        method: "styles",
        http_method: FigmaHttpMethod::Get,
        path_template: "/teams/{team_id}/styles",
        requires_body: false,
        description: "List published styles in a team library",
        path_params: &["team_id"],
    },
    FigmaEndpoint {
        resource: "teams",
        method: "components",
        http_method: FigmaHttpMethod::Get,
        path_template: "/teams/{team_id}/components",
        requires_body: false,
        description: "List published components in a team library",
        path_params: &["team_id"],
    },
    // ── Me ───────────────────────────────────────────────────────────
    FigmaEndpoint {
        resource: "me",
        method: "get",
        http_method: FigmaHttpMethod::Get,
        path_template: "/me",
        requires_body: false,
        description: "Get the authenticated user's profile",
        path_params: &[],
    },
    // ── Webhooks ─────────────────────────────────────────────────────
    FigmaEndpoint {
        resource: "webhooks",
        method: "list",
        http_method: FigmaHttpMethod::Get,
        path_template: "/v2/webhooks",
        requires_body: false,
        description: "List webhooks for the current team",
        path_params: &[],
    },
    FigmaEndpoint {
        resource: "webhooks",
        method: "create",
        http_method: FigmaHttpMethod::Post,
        path_template: "/v2/webhooks",
        requires_body: true,
        description: "Create a webhook",
        path_params: &[],
    },
    FigmaEndpoint {
        resource: "webhooks",
        method: "delete",
        http_method: FigmaHttpMethod::Delete,
        path_template: "/v2/webhooks/{webhook_id}",
        requires_body: false,
        description: "Delete a webhook",
        path_params: &["webhook_id"],
    },
];

// ─── Error type ──────────────────────────────────────────────────────────

#[derive(Debug)]
pub enum FigmaError {
    EndpointNotFound { resource: String, method: String },
    TokenNotFound,
    ParseError(String),
}

impl std::fmt::Display for FigmaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FigmaError::EndpointNotFound { resource, method } => {
                write!(f, "Unknown Figma endpoint: {resource} {method}")
            }
            FigmaError::TokenNotFound => write!(
                f,
                "Figma token not found. Set FIGMA_TOKEN or UWS_FIGMA_TOKEN."
            ),
            FigmaError::ParseError(s) => write!(f, "Parse error: {s}"),
        }
    }
}

// ─── Token resolution ─────────────────────────────────────────────────────

pub fn resolve_figma_token() -> Option<String> {
    std::env::var("UWS_FIGMA_TOKEN")
        .ok()
        .or_else(|| std::env::var("FIGMA_TOKEN").ok())
}

// ─── Path parameter substitution ─────────────────────────────────────────

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

    // If path starts with /v2 use absolute; else prefix with FIGMA_API_BASE
    let url = if path.starts_with("/v2") {
        format!("https://api.figma.com{path}")
    } else {
        format!("{FIGMA_API_BASE}{path}")
    };
    (url, remaining)
}

// ─── Request builder ──────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct FigmaRequest {
    pub http_method: String,
    pub url: String,
    pub query_params: BTreeMap<String, String>,
    pub body: Option<BTreeMap<String, String>>,
    pub auth_header: String,
    pub dry_run: bool,
}

pub fn build_request(
    endpoint: &FigmaEndpoint,
    params: BTreeMap<String, String>,
    body: Option<BTreeMap<String, String>>,
    token: &str,
    dry_run: bool,
) -> FigmaRequest {
    let (url, query_params) = build_url(endpoint.path_template, &params);
    let auth_token = if dry_run { "[REDACTED]".to_string() } else { token.to_string() };

    FigmaRequest {
        http_method: endpoint.http_method.as_str().to_string(),
        url,
        query_params,
        body,
        auth_header: format!("Bearer {auth_token}"),
        dry_run,
    }
}

// ─── Endpoint lookup ──────────────────────────────────────────────────────

pub fn find_endpoint(resource: &str, method: &str) -> Option<&'static FigmaEndpoint> {
    FIGMA_ENDPOINTS
        .iter()
        .find(|e| e.resource == resource && e.method == method)
}

// ─── Unit tests ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_figma_service() {
        assert!(is_figma_service("figma"));
        assert!(!is_figma_service("slack"));
        assert!(!is_figma_service("notion"));
    }

    #[test]
    fn test_endpoint_catalogue_nonempty() {
        assert!(!FIGMA_ENDPOINTS.is_empty());
    }

    #[test]
    fn test_find_files_get() {
        let ep = find_endpoint("files", "get").unwrap();
        assert_eq!(ep.http_method, FigmaHttpMethod::Get);
        assert!(ep.path_template.contains("{file_key}"));
    }

    #[test]
    fn test_find_comments_post() {
        let ep = find_endpoint("comments", "post").unwrap();
        assert_eq!(ep.http_method, FigmaHttpMethod::Post);
        assert!(ep.requires_body);
    }

    #[test]
    fn test_find_unknown_returns_none() {
        assert!(find_endpoint("nonexistent", "get").is_none());
    }

    #[test]
    fn test_build_url_substitutes_path_params() {
        let mut params = BTreeMap::new();
        params.insert("file_key".to_string(), "Abc123".to_string());
        params.insert("format".to_string(), "svg".to_string());
        let (url, remaining) = build_url("/files/{file_key}", &params);
        assert_eq!(url, "https://api.figma.com/v1/files/Abc123");
        assert_eq!(remaining.get("format").map(|s| s.as_str()), Some("svg"));
    }

    #[test]
    fn test_build_url_v2_prefix() {
        let params = BTreeMap::new();
        let (url, _) = build_url("/v2/webhooks", &params);
        assert!(url.starts_with("https://api.figma.com/v2/"));
    }

    #[test]
    fn test_build_request_dry_run_redacts_token() {
        let ep = find_endpoint("files", "get").unwrap();
        let mut params = BTreeMap::new();
        params.insert("file_key".to_string(), "key1".to_string());
        let req = build_request(ep, params, None, "figma_real_token", true);
        assert!(req.auth_header.contains("[REDACTED]"));
        assert!(!req.auth_header.contains("figma_real_token"));
    }

    #[test]
    fn test_components_endpoints_present() {
        assert!(find_endpoint("components", "list").is_some());
        assert!(find_endpoint("components", "get").is_some());
        assert!(find_endpoint("components", "sets").is_some());
    }

    #[test]
    fn test_projects_endpoints_present() {
        assert!(find_endpoint("projects", "list").is_some());
        assert!(find_endpoint("projects", "files").is_some());
    }

    #[test]
    fn test_error_display() {
        let e = FigmaError::TokenNotFound;
        assert!(e.to_string().contains("FIGMA_TOKEN"));
        let e2 = FigmaError::EndpointNotFound {
            resource: "files".to_string(),
            method: "xyz".to_string(),
        };
        assert!(e2.to_string().contains("files xyz"));
    }

    #[test]
    fn test_figma_api_base() {
        assert_eq!(FIGMA_API_BASE, "https://api.figma.com/v1");
    }

    #[test]
    fn test_unique_resource_method_pairs() {
        let mut seen = std::collections::HashSet::new();
        for ep in FIGMA_ENDPOINTS {
            let key = format!("{}/{}", ep.resource, ep.method);
            assert!(seen.insert(key.clone()), "Duplicate endpoint: {key}");
        }
    }

    #[test]
    fn test_webhooks_endpoints_present() {
        assert!(find_endpoint("webhooks", "list").is_some());
        assert!(find_endpoint("webhooks", "create").is_some());
        assert!(find_endpoint("webhooks", "delete").is_some());
    }
}
