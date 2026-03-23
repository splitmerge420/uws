// src/linear_provider.rs
// Aluminum OS — Linear Provider
//
// Exposes the Linear.app GraphQL API through the `uws linear` command surface.
// Linear is the issue tracker of choice for modern engineering teams; this
// provider makes it queryable by any AI agent through the same clean JSON
// grammar as every other uws provider.
//
// Command grammar:
//   uws linear <resource> <method> [--params <JSON>] [--json <JSON>] [--dry-run]
//
// Examples:
//   uws linear issues list
//   uws linear issues get --params '{"id":"ISSUE-123"}'
//   uws linear issues create --json '{"title":"Bug","teamId":"TEAM-1"}'
//   uws linear projects list
//   uws linear teams list
//   uws linear cycles list --params '{"teamId":"TEAM-1"}'
//   uws linear me get
//
// Authentication:
//   Set LINEAR_API_KEY or UWS_LINEAR_TOKEN in the environment.
//   Linear uses a single API key — no OAuth flow required.
//
// Note: Linear uses GraphQL. Each CLI method maps to a named GraphQL operation.
// The request builder outputs the operation name and variables; the executor
// sends them to https://api.linear.app/graphql.
//
// Author: GitHub Copilot — Aluminum OS Session 2026-03-23

#![allow(dead_code)]

use std::collections::BTreeMap;

// ─── Service aliases ──────────────────────────────────────────────────────

pub const LINEAR_ALIASES: &[&str] = &["linear"];

pub fn is_linear_service(name: &str) -> bool {
    LINEAR_ALIASES.contains(&name)
}

// ─── API base ─────────────────────────────────────────────────────────────

pub const LINEAR_API_BASE: &str = "https://api.linear.app/graphql";

// ─── Endpoint descriptor ─────────────────────────────────────────────────

/// HTTP method used by Linear (always POST for GraphQL).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinearMethod {
    Query,
    Mutation,
}

impl LinearMethod {
    pub fn as_str(&self) -> &str {
        match self {
            LinearMethod::Query => "query",
            LinearMethod::Mutation => "mutation",
        }
    }
}

#[derive(Debug, Clone)]
pub struct LinearEndpoint {
    pub resource: &'static str,
    pub method: &'static str,
    pub operation_type: LinearMethod,
    /// GraphQL operation name (used as the operation name in the query string)
    pub operation_name: &'static str,
    pub requires_body: bool,
    pub description: &'static str,
}

pub const LINEAR_ENDPOINTS: &[LinearEndpoint] = &[
    // ── Issues ───────────────────────────────────────────────────────
    LinearEndpoint {
        resource: "issues",
        method: "list",
        operation_type: LinearMethod::Query,
        operation_name: "IssueList",
        requires_body: false,
        description: "List issues (optionally filtered by team, state, assignee)",
    },
    LinearEndpoint {
        resource: "issues",
        method: "get",
        operation_type: LinearMethod::Query,
        operation_name: "IssueGet",
        requires_body: false,
        description: "Get a specific issue by ID",
    },
    LinearEndpoint {
        resource: "issues",
        method: "create",
        operation_type: LinearMethod::Mutation,
        operation_name: "IssueCreate",
        requires_body: true,
        description: "Create a new issue",
    },
    LinearEndpoint {
        resource: "issues",
        method: "update",
        operation_type: LinearMethod::Mutation,
        operation_name: "IssueUpdate",
        requires_body: true,
        description: "Update an existing issue",
    },
    LinearEndpoint {
        resource: "issues",
        method: "archive",
        operation_type: LinearMethod::Mutation,
        operation_name: "IssueArchive",
        requires_body: true,
        description: "Archive an issue",
    },
    LinearEndpoint {
        resource: "issues",
        method: "assign",
        operation_type: LinearMethod::Mutation,
        operation_name: "IssueUpdate",
        requires_body: true,
        description: "Assign an issue to a user",
    },
    LinearEndpoint {
        resource: "issues",
        method: "search",
        operation_type: LinearMethod::Query,
        operation_name: "IssueSearch",
        requires_body: false,
        description: "Search issues by query string",
    },
    // ── Projects ─────────────────────────────────────────────────────
    LinearEndpoint {
        resource: "projects",
        method: "list",
        operation_type: LinearMethod::Query,
        operation_name: "ProjectList",
        requires_body: false,
        description: "List all projects",
    },
    LinearEndpoint {
        resource: "projects",
        method: "get",
        operation_type: LinearMethod::Query,
        operation_name: "ProjectGet",
        requires_body: false,
        description: "Get a specific project by ID",
    },
    LinearEndpoint {
        resource: "projects",
        method: "create",
        operation_type: LinearMethod::Mutation,
        operation_name: "ProjectCreate",
        requires_body: true,
        description: "Create a new project",
    },
    LinearEndpoint {
        resource: "projects",
        method: "update",
        operation_type: LinearMethod::Mutation,
        operation_name: "ProjectUpdate",
        requires_body: true,
        description: "Update a project",
    },
    // ── Teams ────────────────────────────────────────────────────────
    LinearEndpoint {
        resource: "teams",
        method: "list",
        operation_type: LinearMethod::Query,
        operation_name: "TeamList",
        requires_body: false,
        description: "List all teams in the organization",
    },
    LinearEndpoint {
        resource: "teams",
        method: "get",
        operation_type: LinearMethod::Query,
        operation_name: "TeamGet",
        requires_body: false,
        description: "Get a specific team by ID",
    },
    LinearEndpoint {
        resource: "teams",
        method: "members",
        operation_type: LinearMethod::Query,
        operation_name: "TeamMembers",
        requires_body: false,
        description: "List members of a team",
    },
    // ── Cycles (sprints) ─────────────────────────────────────────────
    LinearEndpoint {
        resource: "cycles",
        method: "list",
        operation_type: LinearMethod::Query,
        operation_name: "CycleList",
        requires_body: false,
        description: "List cycles (sprints) for a team",
    },
    LinearEndpoint {
        resource: "cycles",
        method: "get",
        operation_type: LinearMethod::Query,
        operation_name: "CycleGet",
        requires_body: false,
        description: "Get a specific cycle by ID",
    },
    LinearEndpoint {
        resource: "cycles",
        method: "issues",
        operation_type: LinearMethod::Query,
        operation_name: "CycleIssues",
        requires_body: false,
        description: "List issues in a cycle",
    },
    // ── Labels ───────────────────────────────────────────────────────
    LinearEndpoint {
        resource: "labels",
        method: "list",
        operation_type: LinearMethod::Query,
        operation_name: "LabelList",
        requires_body: false,
        description: "List all issue labels",
    },
    LinearEndpoint {
        resource: "labels",
        method: "create",
        operation_type: LinearMethod::Mutation,
        operation_name: "LabelCreate",
        requires_body: true,
        description: "Create a new label",
    },
    // ── Workflow states ───────────────────────────────────────────────
    LinearEndpoint {
        resource: "states",
        method: "list",
        operation_type: LinearMethod::Query,
        operation_name: "WorkflowStateList",
        requires_body: false,
        description: "List workflow states for a team",
    },
    // ── Comments ─────────────────────────────────────────────────────
    LinearEndpoint {
        resource: "comments",
        method: "list",
        operation_type: LinearMethod::Query,
        operation_name: "CommentList",
        requires_body: false,
        description: "List comments on an issue",
    },
    LinearEndpoint {
        resource: "comments",
        method: "create",
        operation_type: LinearMethod::Mutation,
        operation_name: "CommentCreate",
        requires_body: true,
        description: "Add a comment to an issue",
    },
    LinearEndpoint {
        resource: "comments",
        method: "delete",
        operation_type: LinearMethod::Mutation,
        operation_name: "CommentDelete",
        requires_body: true,
        description: "Delete a comment",
    },
    // ── Me (authenticated user) ───────────────────────────────────────
    LinearEndpoint {
        resource: "me",
        method: "get",
        operation_type: LinearMethod::Query,
        operation_name: "Viewer",
        requires_body: false,
        description: "Get the authenticated user's profile",
    },
    // ── Organization ─────────────────────────────────────────────────
    LinearEndpoint {
        resource: "org",
        method: "get",
        operation_type: LinearMethod::Query,
        operation_name: "Organization",
        requires_body: false,
        description: "Get organization details",
    },
    LinearEndpoint {
        resource: "org",
        method: "members",
        operation_type: LinearMethod::Query,
        operation_name: "OrgMembers",
        requires_body: false,
        description: "List all organization members",
    },
    // ── Attachments ──────────────────────────────────────────────────
    LinearEndpoint {
        resource: "attachments",
        method: "create",
        operation_type: LinearMethod::Mutation,
        operation_name: "AttachmentCreate",
        requires_body: true,
        description: "Attach a URL/file to an issue",
    },
    LinearEndpoint {
        resource: "attachments",
        method: "list",
        operation_type: LinearMethod::Query,
        operation_name: "AttachmentList",
        requires_body: false,
        description: "List attachments for an issue",
    },
];

// ─── Error type ──────────────────────────────────────────────────────────

#[derive(Debug)]
pub enum LinearError {
    EndpointNotFound { resource: String, method: String },
    TokenNotFound,
    ParseError(String),
}

impl std::fmt::Display for LinearError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LinearError::EndpointNotFound { resource, method } => {
                write!(f, "Unknown Linear endpoint: {resource} {method}")
            }
            LinearError::TokenNotFound => write!(
                f,
                "Linear API key not found. Set LINEAR_API_KEY or UWS_LINEAR_TOKEN."
            ),
            LinearError::ParseError(s) => write!(f, "Parse error: {s}"),
        }
    }
}

// ─── Token resolution ─────────────────────────────────────────────────────

pub fn resolve_linear_token() -> Option<String> {
    std::env::var("UWS_LINEAR_TOKEN")
        .ok()
        .or_else(|| std::env::var("LINEAR_API_KEY").ok())
}

// ─── Endpoint lookup ──────────────────────────────────────────────────────

pub fn find_endpoint(resource: &str, method: &str) -> Option<&'static LinearEndpoint> {
    LINEAR_ENDPOINTS
        .iter()
        .find(|e| e.resource == resource && e.method == method)
}

// ─── Request builder ──────────────────────────────────────────────────────

/// A fully-constructed Linear GraphQL request (no I/O).
#[derive(Debug, Clone)]
pub struct LinearRequest {
    pub url: String,
    pub operation_type: String,
    pub operation_name: String,
    pub variables: BTreeMap<String, String>,
    pub auth_header: String,
    pub dry_run: bool,
}

/// Build a Linear GraphQL request from CLI inputs.
pub fn build_request(
    endpoint: &LinearEndpoint,
    params: BTreeMap<String, String>,
    body: Option<BTreeMap<String, String>>,
    token: &str,
    dry_run: bool,
) -> LinearRequest {
    // Merge params and body into variables
    let mut variables: BTreeMap<String, String> = params;
    if let Some(b) = body {
        variables.extend(b);
    }

    let auth_token = if dry_run { "[REDACTED]".to_string() } else { token.to_string() };

    LinearRequest {
        url: LINEAR_API_BASE.to_string(),
        operation_type: endpoint.operation_type.as_str().to_string(),
        operation_name: endpoint.operation_name.to_string(),
        variables,
        auth_header: format!("Bearer {auth_token}"),
        dry_run,
    }
}

// ─── Dry-run output ──────────────────────────────────────────────────────

pub fn format_dry_run(req: &LinearRequest, endpoint: &LinearEndpoint) -> String {
    let mut parts = vec![
        format!("\"url\": \"{}\"", req.url),
        format!("\"operationType\": \"{}\"", req.operation_type),
        format!("\"operationName\": \"{}\"", req.operation_name),
        format!("\"description\": \"{}\"", endpoint.description),
    ];
    if !req.variables.is_empty() {
        let vs: Vec<String> = req.variables.iter()
            .map(|(k, v)| format!("\"{k}\": \"{v}\""))
            .collect();
        parts.push(format!("\"variables\": {{{}}}", vs.join(", ")));
    }
    format!("{{\n  {}\n}}", parts.join(",\n  "))
}

// ─── Unit tests ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_linear_service() {
        assert!(is_linear_service("linear"));
        assert!(!is_linear_service("slack"));
        assert!(!is_linear_service("github"));
    }

    #[test]
    fn test_endpoint_catalogue_nonempty() {
        assert!(!LINEAR_ENDPOINTS.is_empty());
    }

    #[test]
    fn test_find_issues_list() {
        let ep = find_endpoint("issues", "list").unwrap();
        assert_eq!(ep.operation_name, "IssueList");
        assert_eq!(ep.operation_type, LinearMethod::Query);
    }

    #[test]
    fn test_find_issues_create_is_mutation() {
        let ep = find_endpoint("issues", "create").unwrap();
        assert_eq!(ep.operation_type, LinearMethod::Mutation);
        assert!(ep.requires_body);
    }

    #[test]
    fn test_find_unknown_returns_none() {
        assert!(find_endpoint("nonexistent", "list").is_none());
    }

    #[test]
    fn test_build_request_url_is_graphql() {
        let ep = find_endpoint("teams", "list").unwrap();
        let req = build_request(ep, BTreeMap::new(), None, "lin_api_token", false);
        assert_eq!(req.url, "https://api.linear.app/graphql");
    }

    #[test]
    fn test_build_request_merges_params_and_body() {
        let ep = find_endpoint("issues", "update").unwrap();
        let mut params = BTreeMap::new();
        params.insert("id".to_string(), "ISSUE-123".to_string());
        let mut body = BTreeMap::new();
        body.insert("title".to_string(), "New title".to_string());
        let req = build_request(ep, params, Some(body), "token", false);
        assert_eq!(req.variables["id"], "ISSUE-123");
        assert_eq!(req.variables["title"], "New title");
    }

    #[test]
    fn test_dry_run_redacts_token() {
        let ep = find_endpoint("issues", "create").unwrap();
        let req = build_request(ep, BTreeMap::new(), None, "lin_api_secret", true);
        assert!(req.auth_header.contains("[REDACTED]"));
        assert!(!req.auth_header.contains("lin_api_secret"));
    }

    #[test]
    fn test_format_dry_run_output() {
        let ep = find_endpoint("cycles", "list").unwrap();
        let mut params = BTreeMap::new();
        params.insert("teamId".to_string(), "TEAM-1".to_string());
        let req = build_request(ep, params, None, "token", true);
        let out = format_dry_run(&req, ep);
        assert!(out.contains("CycleList"));
        assert!(out.contains("TEAM-1"));
    }

    #[test]
    fn test_all_resources_present() {
        let resources = ["issues", "projects", "teams", "cycles", "labels", "states", "comments", "me", "org"];
        for r in &resources {
            assert!(
                LINEAR_ENDPOINTS.iter().any(|e| e.resource == *r),
                "Resource {r} missing from endpoint catalogue"
            );
        }
    }

    #[test]
    fn test_error_display_token_not_found() {
        let e = LinearError::TokenNotFound;
        assert!(e.to_string().contains("LINEAR_API_KEY"));
    }

    #[test]
    fn test_error_display_endpoint_not_found() {
        let e = LinearError::EndpointNotFound {
            resource: "issues".to_string(),
            method: "frobnicate".to_string(),
        };
        assert!(e.to_string().contains("issues frobnicate"));
    }

    #[test]
    fn test_linear_api_constant() {
        assert_eq!(LINEAR_API_BASE, "https://api.linear.app/graphql");
    }

    #[test]
    fn test_unique_resource_method_pairs() {
        let mut seen = std::collections::HashSet::new();
        for ep in LINEAR_ENDPOINTS {
            let key = format!("{}/{}", ep.resource, ep.method);
            assert!(seen.insert(key.clone()), "Duplicate endpoint: {key}");
        }
    }

    #[test]
    fn test_attachments_endpoints_present() {
        assert!(find_endpoint("attachments", "create").is_some());
        assert!(find_endpoint("attachments", "list").is_some());
    }
}
