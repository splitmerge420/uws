// src/github_provider.rs — Universal Workspace CLI (uws)
// GitHub as a first-class uws provider
//
// Surfaces the GitHub REST API through the same uws grammar:
//   uws github-issues list --params '{"owner":"acme","repo":"api","state":"open"}'
//   uws github-pulls list  --params '{"owner":"acme","repo":"api"}'
//   uws github-actions list --params '{"owner":"acme","repo":"api"}'
//   uws github-search list --params '{"q":"JanusRouter language:rust"}'
//   uws github-releases list --params '{"owner":"acme","repo":"api"}'
//
// Authentication: GITHUB_TOKEN environment variable (Personal Access Token or
// workflow token injected automatically in GitHub Actions).
//
// Licensed under the Apache License, Version 2.0

#![allow(dead_code, unused_imports)]

use anyhow::{anyhow, Result};
use serde_json::Value;

/// GitHub REST API base URL
pub const GITHUB_API_BASE: &str = "https://api.github.com";

/// GitHub GraphQL API endpoint
pub const GITHUB_GRAPHQL_URL: &str = "https://api.github.com/graphql";

/// GitHub Models inference endpoint (compatible with OpenAI SDK)
pub const GITHUB_MODELS_URL: &str = "https://models.inference.ai.azure.com";

/// GitHub Copilot Chat endpoint (GitHub Copilot API)
pub const GITHUB_COPILOT_CHAT_URL: &str = "https://api.githubcopilot.com/chat/completions";

/// Environment variable carrying the GitHub token
pub const GITHUB_TOKEN_ENV: &str = "GITHUB_TOKEN";

// ─── Service Registry ─────────────────────────────────────────

/// A GitHub service registered in the uws command surface.
pub struct GitHubServiceEntry {
    /// Command aliases, e.g. &["github-issues", "gh-issues"]
    pub aliases: &'static [&'static str],
    /// The REST API path template, e.g. "/repos/{owner}/{repo}/issues"
    pub api_path: &'static str,
    /// Human-readable description
    pub description: &'static str,
    /// Required GitHub token scopes
    pub scopes: &'static [&'static str],
    /// Whether this endpoint supports pagination via Link headers
    pub paginated: bool,
}

/// All uws-exposed GitHub services.
pub const GITHUB_SERVICES: &[GitHubServiceEntry] = &[
    // ── Issues ──────────────────────────────────────────────────
    GitHubServiceEntry {
        aliases: &["github-issues", "gh-issues"],
        api_path: "/repos/{owner}/{repo}/issues",
        description: "GitHub Issues: list, get, create, update, close",
        scopes: &["repo"],
        paginated: true,
    },
    // ── Pull Requests ────────────────────────────────────────────
    GitHubServiceEntry {
        aliases: &["github-pulls", "github-prs", "gh-pulls"],
        api_path: "/repos/{owner}/{repo}/pulls",
        description: "GitHub Pull Requests: list, get, create, merge",
        scopes: &["repo"],
        paginated: true,
    },
    // ── Actions ──────────────────────────────────────────────────
    GitHubServiceEntry {
        aliases: &["github-actions", "gh-actions"],
        api_path: "/repos/{owner}/{repo}/actions/runs",
        description: "GitHub Actions: list runs, get logs, trigger workflows",
        scopes: &["actions:read"],
        paginated: true,
    },
    // ── Releases ─────────────────────────────────────────────────
    GitHubServiceEntry {
        aliases: &["github-releases", "gh-releases"],
        api_path: "/repos/{owner}/{repo}/releases",
        description: "GitHub Releases: list, get, create, upload assets",
        scopes: &["contents:read"],
        paginated: true,
    },
    // ── Code Search ──────────────────────────────────────────────
    GitHubServiceEntry {
        aliases: &["github-search", "gh-search"],
        api_path: "/search/code",
        description: "GitHub Code Search: search across all public repositories",
        scopes: &[],
        paginated: true,
    },
    // ── Repositories ─────────────────────────────────────────────
    GitHubServiceEntry {
        aliases: &["github-repos", "gh-repos"],
        api_path: "/user/repos",
        description: "GitHub Repositories: list, create, get, fork, delete",
        scopes: &["repo"],
        paginated: true,
    },
    // ── GitHub Models (AI inference) ─────────────────────────────
    GitHubServiceEntry {
        aliases: &["github-models", "gh-models"],
        api_path: "/marketplace/models",
        description: "GitHub Models: list available AI models, run inference",
        scopes: &["models:read"],
        paginated: false,
    },
    // ── Notifications ────────────────────────────────────────────
    GitHubServiceEntry {
        aliases: &["github-notifications", "gh-notifications"],
        api_path: "/notifications",
        description: "GitHub Notifications: list, mark as read, manage subscriptions",
        scopes: &["notifications"],
        paginated: true,
    },
    // ── Gists ────────────────────────────────────────────────────
    GitHubServiceEntry {
        aliases: &["github-gists", "gh-gists"],
        api_path: "/gists",
        description: "GitHub Gists: list, create, update, fork, star",
        scopes: &["gist"],
        paginated: true,
    },
];

// ─── Authentication helper ────────────────────────────────────

/// Returns the GitHub token from the environment, in priority order:
/// 1. GITHUB_TOKEN (set by GitHub Actions automatically)
/// 2. GH_TOKEN (set by the gh CLI)
/// 3. GITHUB_PAT (legacy variable name)
///
/// Returns None if no token is found.
pub fn github_token() -> Option<String> {
    for var in &["GITHUB_TOKEN", "GH_TOKEN", "GITHUB_PAT"] {
        if let Ok(v) = std::env::var(var) {
            if !v.is_empty() {
                return Some(v);
            }
        }
    }
    None
}

/// Returns whether GitHub authentication is available.
pub fn is_authenticated() -> bool {
    github_token().is_some()
}

/// Format a GitHub REST API Authorization header value.
pub fn auth_header(token: &str) -> String {
    format!("Bearer {}", token)
}

/// Build a GitHub REST API URL, substituting {owner}, {repo}, {number} etc.
pub fn build_url(path_template: &str, params: &[(&str, &str)]) -> String {
    let mut url = format!("{}{}", GITHUB_API_BASE, path_template);
    for (key, value) in params {
        url = url.replace(&format!("{{{}}}", key), value);
    }
    url
}

// ─── Service resolver ─────────────────────────────────────────

/// Resolve a GitHub service alias to its entry.
pub fn resolve_github_service(name: &str) -> Option<&'static GitHubServiceEntry> {
    GITHUB_SERVICES.iter().find(|e| e.aliases.contains(&name))
}

// ─── CLI dispatcher ───────────────────────────────────────────

/// Parse shared CLI flags (--params, --json, --method, --dry-run, --path).
fn parse_flags(
    args: &[String],
) -> (Option<String>, Option<String>, Option<String>, bool, Option<String>) {
    let mut params: Option<String> = None;
    let mut body: Option<String> = None;
    let mut method: Option<String> = None;
    let mut dry_run = false;
    let mut path: Option<String> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--params" if i + 1 < args.len() => { params = Some(args[i + 1].clone()); i += 2; }
            "--json"   if i + 1 < args.len() => { body   = Some(args[i + 1].clone()); i += 2; }
            "--method" if i + 1 < args.len() => { method = Some(args[i + 1].clone()); i += 2; }
            "--path"   if i + 1 < args.len() => { path   = Some(args[i + 1].clone()); i += 2; }
            "--dry-run" => { dry_run = true; i += 1; }
            _ => { i += 1; }
        }
    }
    (params, body, method, dry_run, path)
}

/// Execute a GitHub REST API request and print the JSON result.
async fn execute_github_request(
    http_method: &str,
    url: &str,
    token: Option<&str>,
    params: Option<&str>,
    body: Option<&str>,
    dry_run: bool,
) -> Result<()> {
    if dry_run {
        let dry = serde_json::json!({
            "dry_run": true,
            "method": http_method,
            "url": url,
            "params": params,
            "body": body,
            "provider": "github"
        });
        println!("{}", serde_json::to_string_pretty(&dry)?);
        return Ok(());
    }

    let client = reqwest::Client::new();

    let mut query: Vec<(String, String)> = Vec::new();
    if let Some(p) = params {
        if let Ok(obj) = serde_json::from_str::<serde_json::Map<String, Value>>(p) {
            for (k, v) in obj {
                let val = match &v {
                    Value::String(s) => s.clone(),
                    other => other.to_string(),
                };
                query.push((k, val));
            }
        }
    }

    let mut req = match http_method.to_uppercase().as_str() {
        "GET"    => client.get(url),
        "POST"   => client.post(url),
        "PATCH"  => client.patch(url),
        "PUT"    => client.put(url),
        "DELETE" => client.delete(url),
        _        => return Err(anyhow!("Unsupported HTTP method: {http_method}")),
    };

    req = req
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .header("User-Agent", "uws-cli/1.0")
        .query(&query);

    if let Some(tok) = token {
        req = req.bearer_auth(tok);
    }

    if let Some(b) = body {
        req = req
            .header("Content-Type", "application/json")
            .body(b.to_string());
    }

    let resp = req.send().await?;
    let status = resp.status();
    let text = resp.text().await?;

    if !status.is_success() {
        return Err(anyhow!("GitHub API error {}: {}", status, text));
    }

    // Pretty-print if JSON, raw otherwise
    if let Ok(json) = serde_json::from_str::<Value>(&text) {
        println!("{}", serde_json::to_string_pretty(&json)?);
    } else {
        println!("{text}");
    }
    Ok(())
}

/// Dispatch a GitHub service command.
///
/// # Routing
/// - `list`   → GET  `{api_path}` with optional `--params`
/// - `get`    → GET  `{api_path}` with optional `--params`
/// - `create` → POST `{api_path}` with `--json` body
/// - `update` → PATCH `{api_path}` with `--json` body
/// - `delete` → DELETE `{api_path}`
/// - `--path <PATH>` overrides the full API path
/// - `--method <VERB>` overrides the inferred HTTP method
///
/// # Examples
/// ```text
/// uws github-issues list --params '{"owner":"acme","repo":"api","state":"open"}'
/// uws github-pulls create --json '{"title":"Fix bug","head":"fix/issue-42","base":"main"}'
/// uws github-search list --params '{"q":"JanusRouter language:rust"}'
/// ```
pub async fn handle_github_command(service_name: &str, rest_args: &[String]) -> Result<()> {
    let entry = resolve_github_service(service_name)
        .ok_or_else(|| anyhow!("Unknown GitHub service: {service_name}"))?;

    let (params, body, method_flag, dry_run, path_flag) = parse_flags(rest_args);

    // First positional (non-flag) arg is the action alias
    let action = rest_args.iter().find(|a| !a.starts_with('-')).map(|s| s.as_str()).unwrap_or("list");

    let http_method = method_flag.unwrap_or_else(|| match action {
        "create" | "post" | "send" => "POST".to_string(),
        "update" | "patch"         => "PATCH".to_string(),
        "delete" | "remove"        => "DELETE".to_string(),
        "put"                      => "PUT".to_string(),
        _  => if body.is_some() { "POST".to_string() } else { "GET".to_string() },
    });

    let api_path = path_flag.unwrap_or_else(|| entry.api_path.to_string());
    let url = format!("{GITHUB_API_BASE}{api_path}");

    let token = github_token();
    execute_github_request(
        &http_method,
        &url,
        token.as_deref(),
        params.as_deref(),
        body.as_deref(),
        dry_run,
    ).await
}

// ─── Tests ────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_services_have_aliases() {
        for svc in GITHUB_SERVICES {
            assert!(
                !svc.aliases.is_empty(),
                "Service with path {} has no aliases",
                svc.api_path
            );
            // All aliases must start with "github-" or "gh-"
            for alias in svc.aliases {
                assert!(
                    alias.starts_with("github-") || alias.starts_with("gh-"),
                    "Alias '{}' should start with 'github-' or 'gh-'",
                    alias
                );
            }
        }
    }

    #[test]
    fn test_build_url_substitutes_owner_and_repo() {
        let url = build_url(
            "/repos/{owner}/{repo}/issues",
            &[("owner", "acme"), ("repo", "api")],
        );
        assert_eq!(url, "https://api.github.com/repos/acme/api/issues");
    }

    #[test]
    fn test_build_url_leaves_unsubstituted_params() {
        let url = build_url("/repos/{owner}/{repo}/issues", &[("owner", "acme")]);
        // {repo} is not substituted — stays as-is
        assert!(url.contains("{repo}"));
    }

    #[test]
    fn test_auth_header_format() {
        let header = auth_header("ghp_testtoken123");
        assert_eq!(header, "Bearer ghp_testtoken123");
    }

    #[test]
    fn test_resolve_known_service() {
        assert!(resolve_github_service("github-issues").is_some());
        assert!(resolve_github_service("gh-issues").is_some());
        assert!(resolve_github_service("github-pulls").is_some());
        assert!(resolve_github_service("github-actions").is_some());
        assert!(resolve_github_service("unknown-service").is_none());
    }
}
