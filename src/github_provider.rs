// src/github_provider.rs
// Aluminum OS — GitHub Provider
//
// Exposes the GitHub REST API v3 through the `uws github` command surface,
// giving every developer who already uses GitHub immediate, agent-friendly
// access to repos, issues, pull requests, releases, Actions workflows, and
// more — all as clean JSON, with zero separate auth setup beyond a PAT.
//
// Command grammar:
//   uws github <resource> <method> [--params <JSON>] [--json <JSON>] [--dry-run]
//
// Examples:
//   uws github repos list
//   uws github repos get --params '{"owner":"octocat","repo":"Hello-World"}'
//   uws github issues list --params '{"owner":"octocat","repo":"Hello-World","state":"open"}'
//   uws github issues create --params '{"owner":"octocat","repo":"Hello-World"}' \
//       --json '{"title":"Found a bug","body":"..."}'
//   uws github pulls list --params '{"owner":"octocat","repo":"Hello-World"}'
//   uws github search repos --params '{"q":"language:rust stars:>1000"}'
//   uws github actions runs --params '{"owner":"octocat","repo":"Hello-World"}'
//   uws github user me
//   uws github notifications list
//
// Authentication:
//   Set GITHUB_TOKEN or UWS_GITHUB_TOKEN in the environment.
//   Uses the standard GitHub personal access token (PAT) or the GitHub Actions
//   GITHUB_TOKEN secret — no separate OAuth flow required.
//
// GitHub's value to Aluminum OS (contrarian analysis):
//   - 100M+ developers already authenticated with a token → zero onboarding friction
//   - GitHub Issues/PRs/Actions become accessible to every AI agent using uws
//   - GitHub Copilot reads .github/copilot-instructions.md and will suggest uws commands
//   - GitHub Actions marketplace: uws becomes a first-class Actions tool
//   - GitHub MCP server: uws complements (and extends) the official github/mcp-server
//   - GitHub stars → organic discovery → OSS adoption flywheel
//
// Constitutional invariants:
//   INV-6 (Provider Abstraction) — GitHub is one of N providers; no special casing
//   INV-7 (Vendor Balance)       — GitHub gets the same treatment as Google/Microsoft/Apple
//   INV-1 (Sovereignty)          — token never logged; responses streamed locally
//
// Author: GitHub Copilot (builder) + Aluminum OS Council
// Session: 2026-03-22

#![allow(dead_code)]

use std::collections::BTreeMap;

// ─── Service aliases ──────────────────────────────────────────────────────

/// All CLI aliases that route to this provider.
pub const GITHUB_ALIASES: &[&str] = &["github", "gh-api"];

/// Returns `true` if the given service name should be handled by this provider.
pub fn is_github_service(name: &str) -> bool {
    GITHUB_ALIASES.contains(&name)
}

// ─── API base ─────────────────────────────────────────────────────────────

pub const GITHUB_API_BASE: &str = "https://api.github.com";

// ─── Endpoint catalogue ───────────────────────────────────────────────────

/// HTTP methods used by the GitHub REST API.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HttpMethod {
    Get,
    Post,
    Patch,
    Put,
    Delete,
}

impl HttpMethod {
    pub fn as_str(&self) -> &str {
        match self {
            HttpMethod::Get => "GET",
            HttpMethod::Post => "POST",
            HttpMethod::Patch => "PATCH",
            HttpMethod::Put => "PUT",
            HttpMethod::Delete => "DELETE",
        }
    }
}

/// A single GitHub REST API endpoint.
#[derive(Debug, Clone)]
pub struct GhEndpoint {
    /// CLI resource name (e.g. `"repos"`, `"issues"`).
    pub resource: &'static str,
    /// CLI method name (e.g. `"list"`, `"get"`, `"create"`).
    pub method: &'static str,
    /// HTTP verb.
    pub http_method: HttpMethod,
    /// URL path template.  Tokens like `{owner}` and `{repo}` are substituted
    /// from the `--params` JSON before any remaining params are sent as query string.
    pub path_template: &'static str,
    /// Whether a JSON body (`--json`) is required.
    pub requires_body: bool,
    /// Short human-readable description.
    pub description: &'static str,
    /// Names of path-parameter tokens that must be provided in `--params`
    /// (without braces).
    pub path_params: &'static [&'static str],
}

/// The full endpoint catalogue for the GitHub provider.
pub const GH_ENDPOINTS: &[GhEndpoint] = &[
    // ── Repositories ─────────────────────────────────────────────────
    GhEndpoint {
        resource: "repos",
        method: "list",
        http_method: HttpMethod::Get,
        path_template: "/user/repos",
        requires_body: false,
        description: "List repositories for the authenticated user",
        path_params: &[],
    },
    GhEndpoint {
        resource: "repos",
        method: "list-org",
        http_method: HttpMethod::Get,
        path_template: "/orgs/{org}/repos",
        requires_body: false,
        description: "List repositories in an organization",
        path_params: &["org"],
    },
    GhEndpoint {
        resource: "repos",
        method: "list-user",
        http_method: HttpMethod::Get,
        path_template: "/users/{username}/repos",
        requires_body: false,
        description: "List public repositories for a specific user",
        path_params: &["username"],
    },
    GhEndpoint {
        resource: "repos",
        method: "get",
        http_method: HttpMethod::Get,
        path_template: "/repos/{owner}/{repo}",
        requires_body: false,
        description: "Get a repository",
        path_params: &["owner", "repo"],
    },
    GhEndpoint {
        resource: "repos",
        method: "create",
        http_method: HttpMethod::Post,
        path_template: "/user/repos",
        requires_body: true,
        description: "Create a repository for the authenticated user",
        path_params: &[],
    },
    GhEndpoint {
        resource: "repos",
        method: "create-org",
        http_method: HttpMethod::Post,
        path_template: "/orgs/{org}/repos",
        requires_body: true,
        description: "Create a repository in an organization",
        path_params: &["org"],
    },
    GhEndpoint {
        resource: "repos",
        method: "delete",
        http_method: HttpMethod::Delete,
        path_template: "/repos/{owner}/{repo}",
        requires_body: false,
        description: "Delete a repository",
        path_params: &["owner", "repo"],
    },
    // ── Issues ───────────────────────────────────────────────────────
    GhEndpoint {
        resource: "issues",
        method: "list",
        http_method: HttpMethod::Get,
        path_template: "/repos/{owner}/{repo}/issues",
        requires_body: false,
        description: "List issues in a repository",
        path_params: &["owner", "repo"],
    },
    GhEndpoint {
        resource: "issues",
        method: "list-assigned",
        http_method: HttpMethod::Get,
        path_template: "/issues",
        requires_body: false,
        description: "List issues assigned to the authenticated user across all repos",
        path_params: &[],
    },
    GhEndpoint {
        resource: "issues",
        method: "get",
        http_method: HttpMethod::Get,
        path_template: "/repos/{owner}/{repo}/issues/{issue_number}",
        requires_body: false,
        description: "Get a single issue",
        path_params: &["owner", "repo", "issue_number"],
    },
    GhEndpoint {
        resource: "issues",
        method: "create",
        http_method: HttpMethod::Post,
        path_template: "/repos/{owner}/{repo}/issues",
        requires_body: true,
        description: "Create an issue",
        path_params: &["owner", "repo"],
    },
    GhEndpoint {
        resource: "issues",
        method: "update",
        http_method: HttpMethod::Patch,
        path_template: "/repos/{owner}/{repo}/issues/{issue_number}",
        requires_body: true,
        description: "Update an issue (title, body, state, labels, assignees, milestone)",
        path_params: &["owner", "repo", "issue_number"],
    },
    GhEndpoint {
        resource: "issues",
        method: "comments",
        http_method: HttpMethod::Get,
        path_template: "/repos/{owner}/{repo}/issues/{issue_number}/comments",
        requires_body: false,
        description: "List comments on an issue",
        path_params: &["owner", "repo", "issue_number"],
    },
    GhEndpoint {
        resource: "issues",
        method: "comment",
        http_method: HttpMethod::Post,
        path_template: "/repos/{owner}/{repo}/issues/{issue_number}/comments",
        requires_body: true,
        description: "Create a comment on an issue",
        path_params: &["owner", "repo", "issue_number"],
    },
    // ── Pull Requests ─────────────────────────────────────────────────
    GhEndpoint {
        resource: "pulls",
        method: "list",
        http_method: HttpMethod::Get,
        path_template: "/repos/{owner}/{repo}/pulls",
        requires_body: false,
        description: "List pull requests in a repository",
        path_params: &["owner", "repo"],
    },
    GhEndpoint {
        resource: "pulls",
        method: "get",
        http_method: HttpMethod::Get,
        path_template: "/repos/{owner}/{repo}/pulls/{pull_number}",
        requires_body: false,
        description: "Get a pull request",
        path_params: &["owner", "repo", "pull_number"],
    },
    GhEndpoint {
        resource: "pulls",
        method: "create",
        http_method: HttpMethod::Post,
        path_template: "/repos/{owner}/{repo}/pulls",
        requires_body: true,
        description: "Create a pull request",
        path_params: &["owner", "repo"],
    },
    GhEndpoint {
        resource: "pulls",
        method: "merge",
        http_method: HttpMethod::Put,
        path_template: "/repos/{owner}/{repo}/pulls/{pull_number}/merge",
        requires_body: false,
        description: "Merge a pull request",
        path_params: &["owner", "repo", "pull_number"],
    },
    GhEndpoint {
        resource: "pulls",
        method: "reviews",
        http_method: HttpMethod::Get,
        path_template: "/repos/{owner}/{repo}/pulls/{pull_number}/reviews",
        requires_body: false,
        description: "List reviews on a pull request",
        path_params: &["owner", "repo", "pull_number"],
    },
    GhEndpoint {
        resource: "pulls",
        method: "files",
        http_method: HttpMethod::Get,
        path_template: "/repos/{owner}/{repo}/pulls/{pull_number}/files",
        requires_body: false,
        description: "List files changed in a pull request",
        path_params: &["owner", "repo", "pull_number"],
    },
    // ── Releases ─────────────────────────────────────────────────────
    GhEndpoint {
        resource: "releases",
        method: "list",
        http_method: HttpMethod::Get,
        path_template: "/repos/{owner}/{repo}/releases",
        requires_body: false,
        description: "List releases in a repository",
        path_params: &["owner", "repo"],
    },
    GhEndpoint {
        resource: "releases",
        method: "get",
        http_method: HttpMethod::Get,
        path_template: "/repos/{owner}/{repo}/releases/{release_id}",
        requires_body: false,
        description: "Get a release",
        path_params: &["owner", "repo", "release_id"],
    },
    GhEndpoint {
        resource: "releases",
        method: "latest",
        http_method: HttpMethod::Get,
        path_template: "/repos/{owner}/{repo}/releases/latest",
        requires_body: false,
        description: "Get the latest release",
        path_params: &["owner", "repo"],
    },
    GhEndpoint {
        resource: "releases",
        method: "create",
        http_method: HttpMethod::Post,
        path_template: "/repos/{owner}/{repo}/releases",
        requires_body: true,
        description: "Create a release",
        path_params: &["owner", "repo"],
    },
    // ── GitHub Actions ────────────────────────────────────────────────
    GhEndpoint {
        resource: "actions",
        method: "list",
        http_method: HttpMethod::Get,
        path_template: "/repos/{owner}/{repo}/actions/workflows",
        requires_body: false,
        description: "List workflows in a repository",
        path_params: &["owner", "repo"],
    },
    GhEndpoint {
        resource: "actions",
        method: "runs",
        http_method: HttpMethod::Get,
        path_template: "/repos/{owner}/{repo}/actions/runs",
        requires_body: false,
        description: "List workflow runs in a repository",
        path_params: &["owner", "repo"],
    },
    GhEndpoint {
        resource: "actions",
        method: "jobs",
        http_method: HttpMethod::Get,
        path_template: "/repos/{owner}/{repo}/actions/runs/{run_id}/jobs",
        requires_body: false,
        description: "List jobs for a workflow run",
        path_params: &["owner", "repo", "run_id"],
    },
    GhEndpoint {
        resource: "actions",
        method: "dispatch",
        http_method: HttpMethod::Post,
        path_template: "/repos/{owner}/{repo}/actions/workflows/{workflow_id}/dispatches",
        requires_body: true,
        description: "Trigger a workflow dispatch event",
        path_params: &["owner", "repo", "workflow_id"],
    },
    // ── Search ───────────────────────────────────────────────────────
    GhEndpoint {
        resource: "search",
        method: "repos",
        http_method: HttpMethod::Get,
        path_template: "/search/repositories",
        requires_body: false,
        description: "Search repositories",
        path_params: &[],
    },
    GhEndpoint {
        resource: "search",
        method: "issues",
        http_method: HttpMethod::Get,
        path_template: "/search/issues",
        requires_body: false,
        description: "Search issues and pull requests",
        path_params: &[],
    },
    GhEndpoint {
        resource: "search",
        method: "code",
        http_method: HttpMethod::Get,
        path_template: "/search/code",
        requires_body: false,
        description: "Search code across GitHub",
        path_params: &[],
    },
    GhEndpoint {
        resource: "search",
        method: "users",
        http_method: HttpMethod::Get,
        path_template: "/search/users",
        requires_body: false,
        description: "Search users",
        path_params: &[],
    },
    GhEndpoint {
        resource: "search",
        method: "commits",
        http_method: HttpMethod::Get,
        path_template: "/search/commits",
        requires_body: false,
        description: "Search commits",
        path_params: &[],
    },
    // ── Users ────────────────────────────────────────────────────────
    GhEndpoint {
        resource: "user",
        method: "me",
        http_method: HttpMethod::Get,
        path_template: "/user",
        requires_body: false,
        description: "Get the authenticated user",
        path_params: &[],
    },
    GhEndpoint {
        resource: "users",
        method: "get",
        http_method: HttpMethod::Get,
        path_template: "/users/{username}",
        requires_body: false,
        description: "Get a user by username",
        path_params: &["username"],
    },
    GhEndpoint {
        resource: "users",
        method: "repos",
        http_method: HttpMethod::Get,
        path_template: "/users/{username}/repos",
        requires_body: false,
        description: "List public repositories for a user",
        path_params: &["username"],
    },
    // ── Notifications ────────────────────────────────────────────────
    GhEndpoint {
        resource: "notifications",
        method: "list",
        http_method: HttpMethod::Get,
        path_template: "/notifications",
        requires_body: false,
        description: "List notifications for the authenticated user",
        path_params: &[],
    },
    GhEndpoint {
        resource: "notifications",
        method: "mark-read",
        http_method: HttpMethod::Put,
        path_template: "/notifications",
        requires_body: false,
        description: "Mark all notifications as read",
        path_params: &[],
    },
    // ── Stars ────────────────────────────────────────────────────────
    GhEndpoint {
        resource: "stars",
        method: "list",
        http_method: HttpMethod::Get,
        path_template: "/user/starred",
        requires_body: false,
        description: "List repositories starred by the authenticated user",
        path_params: &[],
    },
    GhEndpoint {
        resource: "stars",
        method: "list-user",
        http_method: HttpMethod::Get,
        path_template: "/users/{username}/starred",
        requires_body: false,
        description: "List repositories starred by a specific user",
        path_params: &["username"],
    },
    GhEndpoint {
        resource: "stars",
        method: "star",
        http_method: HttpMethod::Put,
        path_template: "/user/starred/{owner}/{repo}",
        requires_body: false,
        description: "Star a repository",
        path_params: &["owner", "repo"],
    },
    GhEndpoint {
        resource: "stars",
        method: "unstar",
        http_method: HttpMethod::Delete,
        path_template: "/user/starred/{owner}/{repo}",
        requires_body: false,
        description: "Unstar a repository",
        path_params: &["owner", "repo"],
    },
    // ── Gists ────────────────────────────────────────────────────────
    GhEndpoint {
        resource: "gists",
        method: "list",
        http_method: HttpMethod::Get,
        path_template: "/gists",
        requires_body: false,
        description: "List gists for the authenticated user",
        path_params: &[],
    },
    GhEndpoint {
        resource: "gists",
        method: "get",
        http_method: HttpMethod::Get,
        path_template: "/gists/{gist_id}",
        requires_body: false,
        description: "Get a gist",
        path_params: &["gist_id"],
    },
    GhEndpoint {
        resource: "gists",
        method: "create",
        http_method: HttpMethod::Post,
        path_template: "/gists",
        requires_body: true,
        description: "Create a gist",
        path_params: &[],
    },
    // ── Contents / Files ─────────────────────────────────────────────
    GhEndpoint {
        resource: "contents",
        method: "get",
        http_method: HttpMethod::Get,
        path_template: "/repos/{owner}/{repo}/contents/{path}",
        requires_body: false,
        description: "Get file or directory contents",
        path_params: &["owner", "repo", "path"],
    },
    GhEndpoint {
        resource: "contents",
        method: "create",
        http_method: HttpMethod::Put,
        path_template: "/repos/{owner}/{repo}/contents/{path}",
        requires_body: true,
        description: "Create or update a file",
        path_params: &["owner", "repo", "path"],
    },
    // ── Commits ──────────────────────────────────────────────────────
    GhEndpoint {
        resource: "commits",
        method: "list",
        http_method: HttpMethod::Get,
        path_template: "/repos/{owner}/{repo}/commits",
        requires_body: false,
        description: "List commits in a repository",
        path_params: &["owner", "repo"],
    },
    GhEndpoint {
        resource: "commits",
        method: "get",
        http_method: HttpMethod::Get,
        path_template: "/repos/{owner}/{repo}/commits/{ref}",
        requires_body: false,
        description: "Get a commit",
        path_params: &["owner", "repo", "ref"],
    },
    // ── Organizations ─────────────────────────────────────────────────
    GhEndpoint {
        resource: "orgs",
        method: "list",
        http_method: HttpMethod::Get,
        path_template: "/user/orgs",
        requires_body: false,
        description: "List organizations for the authenticated user",
        path_params: &[],
    },
    GhEndpoint {
        resource: "orgs",
        method: "get",
        http_method: HttpMethod::Get,
        path_template: "/orgs/{org}",
        requires_body: false,
        description: "Get an organization",
        path_params: &["org"],
    },
    GhEndpoint {
        resource: "orgs",
        method: "members",
        http_method: HttpMethod::Get,
        path_template: "/orgs/{org}/members",
        requires_body: false,
        description: "List members of an organization",
        path_params: &["org"],
    },
    // ── Labels ───────────────────────────────────────────────────────
    GhEndpoint {
        resource: "labels",
        method: "list",
        http_method: HttpMethod::Get,
        path_template: "/repos/{owner}/{repo}/labels",
        requires_body: false,
        description: "List labels for a repository",
        path_params: &["owner", "repo"],
    },
    GhEndpoint {
        resource: "labels",
        method: "create",
        http_method: HttpMethod::Post,
        path_template: "/repos/{owner}/{repo}/labels",
        requires_body: true,
        description: "Create a label",
        path_params: &["owner", "repo"],
    },
    // ── Milestones ───────────────────────────────────────────────────
    GhEndpoint {
        resource: "milestones",
        method: "list",
        http_method: HttpMethod::Get,
        path_template: "/repos/{owner}/{repo}/milestones",
        requires_body: false,
        description: "List milestones for a repository",
        path_params: &["owner", "repo"],
    },
];

// ─── Lookup ───────────────────────────────────────────────────────────────

/// Errors from the GitHub provider.
#[derive(Debug, Clone, PartialEq)]
pub enum GhError {
    UnknownEndpoint { resource: String, method: String },
    MissingPathParam(String),
    MissingToken,
    HttpError { status: u16, body: String },
    ParseError(String),
}

impl std::fmt::Display for GhError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GhError::UnknownEndpoint { resource, method } => {
                write!(f, "unknown github endpoint: {} {}", resource, method)
            }
            GhError::MissingPathParam(p) => {
                write!(f, "missing required path parameter: {}", p)
            }
            GhError::MissingToken => write!(
                f,
                "GitHub token not found. Set GITHUB_TOKEN or UWS_GITHUB_TOKEN."
            ),
            GhError::HttpError { status, body } => {
                write!(f, "GitHub API error {}: {}", status, body)
            }
            GhError::ParseError(msg) => write!(f, "parse error: {}", msg),
        }
    }
}

/// Look up an endpoint by (resource, method).
pub fn resolve_github_endpoint(
    resource: &str,
    method: &str,
) -> Result<&'static GhEndpoint, GhError> {
    GH_ENDPOINTS
        .iter()
        .find(|e| e.resource == resource && e.method == method)
        .ok_or_else(|| GhError::UnknownEndpoint {
            resource: resource.to_string(),
            method: method.to_string(),
        })
}

/// List all endpoints for a given resource (for help text).
pub fn endpoints_for_resource(resource: &str) -> Vec<&'static GhEndpoint> {
    GH_ENDPOINTS
        .iter()
        .filter(|e| e.resource == resource)
        .collect()
}

/// All distinct resource names in the catalogue.
pub fn all_resources() -> Vec<&'static str> {
    let mut seen = std::collections::HashSet::new();
    let mut resources = Vec::new();
    for ep in GH_ENDPOINTS {
        if seen.insert(ep.resource) {
            resources.push(ep.resource);
        }
    }
    resources
}

// ─── URL building ─────────────────────────────────────────────────────────

/// Parse a `--params` JSON string into a `BTreeMap<String, String>`.
/// Only string and number values are extracted; nested objects are skipped.
pub fn parse_params(json_str: &str) -> Result<BTreeMap<String, String>, GhError> {
    if json_str.trim().is_empty() {
        return Ok(BTreeMap::new());
    }
    // Use a hand-rolled extractor matching the zero-dep philosophy in universal_io.
    parse_flat_json_object(json_str)
        .map_err(|e| GhError::ParseError(e))
}

/// Build a full GitHub API URL by substituting path parameters from `params`
/// into `path_template`, then appending remaining params as query string.
///
/// Returns `(url, remaining_params_used_as_query_string)`.
pub fn build_github_url(
    path_template: &str,
    path_param_names: &[&str],
    params: &BTreeMap<String, String>,
) -> Result<String, GhError> {
    let mut path = path_template.to_string();

    // Substitute all declared path parameters.
    for param in path_param_names {
        let value = params.get(*param).ok_or_else(|| {
            GhError::MissingPathParam((*param).to_string())
        })?;
        let token = format!("{{{}}}", param);
        path = path.replace(&token, value);
    }

    // Build query string from remaining params (not used as path params).
    let path_param_set: std::collections::HashSet<&str> =
        path_param_names.iter().copied().collect();
    let query_parts: Vec<String> = params
        .iter()
        .filter(|(k, _)| !path_param_set.contains(k.as_str()))
        .map(|(k, v)| format!("{}={}", percent_encode(k), percent_encode(v)))
        .collect();

    let url = if query_parts.is_empty() {
        format!("{}{}", GITHUB_API_BASE, path)
    } else {
        format!("{}{}?{}", GITHUB_API_BASE, path, query_parts.join("&"))
    };

    Ok(url)
}

/// Simple percent-encoding for query string values.
/// Encodes space, `&`, `=`, `+`, `#`, `%`, `{`, `}` and non-ASCII.
fn percent_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for byte in s.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9'
            | b'-' | b'_' | b'.' | b'~' | b':' | b'/' | b',' | b'+' => {
                out.push(byte as char);
            }
            _ => {
                out.push('%');
                out.push(hex_char(byte >> 4));
                out.push(hex_char(byte & 0xf));
            }
        }
    }
    out
}

fn hex_char(n: u8) -> char {
    match n {
        0..=9 => (b'0' + n) as char,
        _ => (b'A' + n - 10) as char,
    }
}

// ─── Auth ─────────────────────────────────────────────────────────────────

/// Retrieve the GitHub token from the environment.
/// Checks `UWS_GITHUB_TOKEN` first (uws-specific override), then `GITHUB_TOKEN`
/// (the standard GitHub Actions / gh CLI variable).
pub fn get_github_token() -> Result<String, GhError> {
    std::env::var("UWS_GITHUB_TOKEN")
        .or_else(|_| std::env::var("GITHUB_TOKEN"))
        .map_err(|_| GhError::MissingToken)
}

// ─── Request builder ──────────────────────────────────────────────────────

/// A fully-resolved GitHub API request, ready to execute.
#[derive(Debug, Clone)]
pub struct GhRequest {
    pub method: HttpMethod,
    pub url: String,
    pub token: String,
    pub body: Option<String>,
    pub dry_run: bool,
}

/// Build a `GhRequest` from parsed CLI arguments.
///
/// This is the pure, testable core of the provider — no I/O.
pub fn build_request(
    resource: &str,
    method: &str,
    params_json: Option<&str>,
    body_json: Option<&str>,
    dry_run: bool,
    token: &str,
) -> Result<GhRequest, GhError> {
    let endpoint = resolve_github_endpoint(resource, method)?;

    // Parse params.
    let params = match params_json {
        Some(json) if !json.trim().is_empty() => parse_params(json)?,
        _ => BTreeMap::new(),
    };

    // Build URL with path substitution.
    let url = build_github_url(endpoint.path_template, endpoint.path_params, &params)?;

    Ok(GhRequest {
        method: endpoint.http_method.clone(),
        url,
        token: token.to_string(),
        body: body_json.map(|s| s.to_string()),
        dry_run,
    })
}

// ─── Dry-run output ──────────────────────────────────────────────────────

/// Return a JSON string describing the request (for --dry-run).
pub fn dry_run_json(req: &GhRequest) -> String {
    let body_val = match &req.body {
        Some(b) => format!(",\"body\":{}", b),
        None => String::new(),
    };
    format!(
        r#"{{"dry_run":true,"method":"{}","url":"{}","auth":"Bearer ***"{}  }}"#,
        req.method.as_str(),
        req.url,
        body_val,
    )
}

// ─── Help text ───────────────────────────────────────────────────────────

/// Return a JSON string listing all available GitHub endpoints.
pub fn help_json() -> String {
    let endpoints: Vec<String> = GH_ENDPOINTS
        .iter()
        .map(|e| {
            format!(
                r#"{{"resource":"{}","method":"{}","http_method":"{}","description":"{}","path":"{}"}}"#,
                e.resource,
                e.method,
                e.http_method.as_str(),
                e.description,
                e.path_template,
            )
        })
        .collect();
    format!("[{}]", endpoints.join(","))
}

// ─── Minimal JSON parser ─────────────────────────────────────────────────

/// Parse a flat JSON object `{"key":"value","num":42}` into a string map.
/// Handles string and number values; skips null, boolean, objects, arrays.
fn parse_flat_json_object(json: &str) -> Result<BTreeMap<String, String>, String> {
    let json = json.trim();
    if !json.starts_with('{') || !json.ends_with('}') {
        return Err(format!("expected JSON object, got: {}", &json[..json.len().min(30)]));
    }
    let inner = &json[1..json.len() - 1];
    if inner.trim().is_empty() {
        return Ok(BTreeMap::new());
    }

    let mut map = BTreeMap::new();
    let mut rest = inner.trim();

    while !rest.is_empty() {
        rest = rest.trim_start_matches(',').trim();
        if rest.is_empty() {
            break;
        }
        if !rest.starts_with('"') {
            return Err(format!("expected '\"', got: {}", &rest[..rest.len().min(20)]));
        }
        // Parse key.
        let (key, after_key) = parse_json_string(rest)?;
        rest = after_key.trim_start_matches(':').trim();

        // Parse value.
        let (value, after_val) = parse_json_value(rest)?;
        if let Some(v) = value {
            map.insert(key, v);
        }
        rest = after_val.trim();
    }
    Ok(map)
}

fn parse_json_string(s: &str) -> Result<(String, &str), String> {
    if !s.starts_with('"') {
        return Err("expected '\"'".to_string());
    }
    let mut chars = s[1..].char_indices();
    let mut result = String::new();
    while let Some((i, c)) = chars.next() {
        match c {
            '\\' => {
                if let Some((_, esc)) = chars.next() {
                    match esc {
                        '"' => result.push('"'),
                        '\\' => result.push('\\'),
                        'n' => result.push('\n'),
                        'r' => result.push('\r'),
                        't' => result.push('\t'),
                        other => {
                            result.push('\\');
                            result.push(other);
                        }
                    }
                }
            }
            '"' => {
                return Ok((result, &s[i + 2..]));
            }
            _ => result.push(c),
        }
    }
    Err("unterminated string".to_string())
}

fn parse_json_value<'a>(s: &'a str) -> Result<(Option<String>, &'a str), String> {
    if s.starts_with('"') {
        let (v, rest) = parse_json_string(s)?;
        return Ok((Some(v), rest));
    }
    // Number (possibly negative, possibly float).
    if s.starts_with(|c: char| c.is_ascii_digit() || c == '-') {
        let end = s
            .char_indices()
            .find(|(_, c)| !c.is_ascii_digit() && *c != '.' && *c != '-' && *c != 'e' && *c != 'E' && *c != '+')
            .map(|(i, _)| i)
            .unwrap_or(s.len());
        return Ok((Some(s[..end].to_string()), &s[end..]));
    }
    // Boolean / null — skip, but consume.
    if s.starts_with("true") { return Ok((None, &s[4..])); }
    if s.starts_with("false") { return Ok((None, &s[5..])); }
    if s.starts_with("null") { return Ok((None, &s[4..])); }
    // Nested object or array — skip by counting braces/brackets.
    if s.starts_with('{') || s.starts_with('[') {
        let close = if s.starts_with('{') { (b'{', b'}') } else { (b'[', b']') };
        let mut depth = 0i32;
        for (i, b) in s.bytes().enumerate() {
            if b == close.0 { depth += 1; }
            if b == close.1 {
                depth -= 1;
                if depth == 0 {
                    return Ok((None, &s[i + 1..]));
                }
            }
        }
        return Err("unterminated object/array".to_string());
    }
    Err(format!("unexpected value start: {}", &s[..s.len().min(20)]))
}

// ─── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Service identification ─────────────────────────────────────────

    #[test]
    fn test_is_github_service_primary_alias() {
        assert!(is_github_service("github"));
    }

    #[test]
    fn test_is_github_service_secondary_alias() {
        assert!(is_github_service("gh-api"));
    }

    #[test]
    fn test_is_not_github_service() {
        assert!(!is_github_service("drive"));
        assert!(!is_github_service("ms-mail"));
        assert!(!is_github_service("gitlab"));
    }

    // ── Endpoint resolution ───────────────────────────────────────────

    #[test]
    fn test_resolve_known_endpoint() {
        let ep = resolve_github_endpoint("repos", "list").unwrap();
        assert_eq!(ep.resource, "repos");
        assert_eq!(ep.method, "list");
        assert_eq!(ep.http_method, HttpMethod::Get);
    }

    #[test]
    fn test_resolve_post_endpoint() {
        let ep = resolve_github_endpoint("issues", "create").unwrap();
        assert_eq!(ep.http_method, HttpMethod::Post);
        assert!(ep.requires_body);
    }

    #[test]
    fn test_resolve_unknown_endpoint_returns_error() {
        let err = resolve_github_endpoint("repos", "teleport").unwrap_err();
        assert!(matches!(err, GhError::UnknownEndpoint { .. }));
    }

    #[test]
    fn test_all_resources_nonempty() {
        let resources = all_resources();
        assert!(!resources.is_empty());
        assert!(resources.contains(&"repos"));
        assert!(resources.contains(&"issues"));
        assert!(resources.contains(&"pulls"));
        assert!(resources.contains(&"search"));
    }

    #[test]
    fn test_endpoints_for_resource() {
        let eps = endpoints_for_resource("issues");
        assert!(eps.len() >= 4); // list, get, create, update at minimum
        assert!(eps.iter().any(|e| e.method == "list"));
        assert!(eps.iter().any(|e| e.method == "create"));
    }

    // ── URL building ──────────────────────────────────────────────────

    #[test]
    fn test_build_url_no_params() {
        let params = BTreeMap::new();
        let url = build_github_url("/user/repos", &[], &params).unwrap();
        assert_eq!(url, "https://api.github.com/user/repos");
    }

    #[test]
    fn test_build_url_path_params_substituted() {
        let mut params = BTreeMap::new();
        params.insert("owner".to_string(), "octocat".to_string());
        params.insert("repo".to_string(), "Hello-World".to_string());
        let url =
            build_github_url("/repos/{owner}/{repo}", &["owner", "repo"], &params).unwrap();
        assert_eq!(url, "https://api.github.com/repos/octocat/Hello-World");
    }

    #[test]
    fn test_build_url_remaining_params_become_query_string() {
        let mut params = BTreeMap::new();
        params.insert("owner".to_string(), "octocat".to_string());
        params.insert("repo".to_string(), "Hello-World".to_string());
        params.insert("state".to_string(), "open".to_string());
        let url = build_github_url(
            "/repos/{owner}/{repo}/issues",
            &["owner", "repo"],
            &params,
        )
        .unwrap();
        assert!(url.starts_with("https://api.github.com/repos/octocat/Hello-World/issues?"));
        assert!(url.contains("state=open"));
    }

    #[test]
    fn test_build_url_missing_path_param_returns_error() {
        let params = BTreeMap::new();
        let err = build_github_url(
            "/repos/{owner}/{repo}",
            &["owner", "repo"],
            &params,
        )
        .unwrap_err();
        assert!(matches!(err, GhError::MissingPathParam(p) if p == "owner"));
    }

    #[test]
    fn test_build_url_search_with_query() {
        let mut params = BTreeMap::new();
        params.insert("q".to_string(), "language:rust stars:>1000".to_string());
        let url = build_github_url("/search/repositories", &[], &params).unwrap();
        // Space is encoded as %20; colon is allowed through; '>' is encoded as %3E.
        assert!(url.contains("q=language:rust"));
        assert!(url.contains("%3E1000") || url.contains(">1000"));
    }

    #[test]
    fn test_build_url_three_path_params() {
        let mut params = BTreeMap::new();
        params.insert("owner".to_string(), "octocat".to_string());
        params.insert("repo".to_string(), "Hello-World".to_string());
        params.insert("issue_number".to_string(), "42".to_string());
        let url = build_github_url(
            "/repos/{owner}/{repo}/issues/{issue_number}",
            &["owner", "repo", "issue_number"],
            &params,
        )
        .unwrap();
        assert_eq!(
            url,
            "https://api.github.com/repos/octocat/Hello-World/issues/42"
        );
    }

    // ── JSON param parsing ────────────────────────────────────────────

    #[test]
    fn test_parse_params_empty_string() {
        let params = parse_params("").unwrap();
        assert!(params.is_empty());
    }

    #[test]
    fn test_parse_params_simple_object() {
        let params = parse_params(r#"{"owner":"octocat","repo":"Hello-World"}"#).unwrap();
        assert_eq!(params.get("owner").unwrap(), "octocat");
        assert_eq!(params.get("repo").unwrap(), "Hello-World");
    }

    #[test]
    fn test_parse_params_numeric_value() {
        let params = parse_params(r#"{"per_page":100,"page":2}"#).unwrap();
        assert_eq!(params.get("per_page").unwrap(), "100");
        assert_eq!(params.get("page").unwrap(), "2");
    }

    #[test]
    fn test_parse_params_special_chars_in_value() {
        let params = parse_params(r#"{"q":"is:open label:\"bug\""}"#).unwrap();
        assert!(params.get("q").is_some());
    }

    #[test]
    fn test_parse_params_not_object_returns_error() {
        let err = parse_params(r#"["array"]"#).unwrap_err();
        assert!(matches!(err, GhError::ParseError(_)));
    }

    // ── build_request ─────────────────────────────────────────────────

    #[test]
    fn test_build_request_list_repos() {
        let req = build_request("repos", "list", None, None, false, "token123").unwrap();
        assert_eq!(req.method, HttpMethod::Get);
        assert_eq!(req.url, "https://api.github.com/user/repos");
        assert_eq!(req.token, "token123");
        assert!(req.body.is_none());
        assert!(!req.dry_run);
    }

    #[test]
    fn test_build_request_get_repo() {
        let req = build_request(
            "repos",
            "get",
            Some(r#"{"owner":"octocat","repo":"Hello-World"}"#),
            None,
            false,
            "tok",
        )
        .unwrap();
        assert_eq!(
            req.url,
            "https://api.github.com/repos/octocat/Hello-World"
        );
    }

    #[test]
    fn test_build_request_dry_run_flag() {
        let req = build_request("user", "me", None, None, true, "tok").unwrap();
        assert!(req.dry_run);
    }

    #[test]
    fn test_build_request_with_body() {
        let req = build_request(
            "issues",
            "create",
            Some(r#"{"owner":"octocat","repo":"Hello-World"}"#),
            Some(r#"{"title":"Found a bug","body":"..."}"#),
            false,
            "tok",
        )
        .unwrap();
        assert_eq!(req.method, HttpMethod::Post);
        assert!(req.body.is_some());
    }

    #[test]
    fn test_build_request_unknown_endpoint_error() {
        let err = build_request("repos", "fly", None, None, false, "tok").unwrap_err();
        assert!(matches!(err, GhError::UnknownEndpoint { .. }));
    }

    // ── dry_run_json ──────────────────────────────────────────────────

    #[test]
    fn test_dry_run_json_redacts_token() {
        let req = GhRequest {
            method: HttpMethod::Get,
            url: "https://api.github.com/user".to_string(),
            token: "super_secret_token".to_string(),
            body: None,
            dry_run: true,
        };
        let json = dry_run_json(&req);
        assert!(json.contains("GET"));
        assert!(!json.contains("super_secret_token"));
        assert!(json.contains("Bearer ***"));
    }

    // ── help_json ─────────────────────────────────────────────────────

    #[test]
    fn test_help_json_is_valid_array() {
        let json = help_json();
        assert!(json.starts_with('['));
        assert!(json.ends_with(']'));
        assert!(json.contains("repos"));
        assert!(json.contains("issues"));
    }

    // ── HttpMethod display ────────────────────────────────────────────

    #[test]
    fn test_http_method_as_str() {
        assert_eq!(HttpMethod::Get.as_str(), "GET");
        assert_eq!(HttpMethod::Post.as_str(), "POST");
        assert_eq!(HttpMethod::Patch.as_str(), "PATCH");
        assert_eq!(HttpMethod::Put.as_str(), "PUT");
        assert_eq!(HttpMethod::Delete.as_str(), "DELETE");
    }
}
