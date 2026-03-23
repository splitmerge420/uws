// src/linear_provider.rs
// Universal Workspace CLI (uws) — Linear.app integration
//
// Linear is the modern issue-tracking platform used by thousands of engineering
// teams. This module provides structured access to the Linear GraphQL API v2:
//   - Issues (list, create, update, comment, assign)
//   - Projects (list, create, archive)
//   - Teams (list, members, states)
//   - Cycles (sprints: list, active, completed)
//   - Views (saved filters)
//   - Webhooks (create/list/delete)
//
// Auth: Linear API key via UWS_LINEAR_API_KEY env var,
//       or OAuth2 app via UWS_LINEAR_CLIENT_ID / UWS_LINEAR_CLIENT_SECRET.
//
// Licensed under the Apache License, Version 2.0

#![allow(dead_code)]

use anyhow::{anyhow, Result};
use serde_json::Value;

// ─── Constants ────────────────────────────────────────────────

/// Linear GraphQL API endpoint.
pub const LINEAR_API_BASE: &str = "https://api.linear.app/graphql";

/// Linear OAuth2 authorization endpoint.
pub const LINEAR_AUTH_URL: &str = "https://linear.app/oauth/authorize";

/// Linear OAuth2 token endpoint.
pub const LINEAR_TOKEN_URL: &str = "https://api.linear.app/oauth/token";

// ─── Service Registry ─────────────────────────────────────────

/// A Linear resource accessible through `uws linear-<alias>`.
pub struct LinearServiceEntry {
    pub aliases: &'static [&'static str],
    pub description: &'static str,
    pub example_query: &'static str,
}

/// All first-class Linear resources exposed by `uws`.
pub const LINEAR_SERVICES: &[LinearServiceEntry] = &[
    LinearServiceEntry {
        aliases: &["linear-issues", "linear-issue"],
        description: "Issues: list, create, update, comment on, and assign Linear issues",
        example_query: r#"{ "query": "{ issues(first: 20) { nodes { id title state { name } assignee { name } priority } } }" }"#,
    },
    LinearServiceEntry {
        aliases: &["linear-projects", "linear-project"],
        description: "Projects: list, create, and archive Linear projects",
        example_query: r#"{ "query": "{ projects { nodes { id name state description } } }" }"#,
    },
    LinearServiceEntry {
        aliases: &["linear-teams", "linear-team"],
        description: "Teams: list teams, members, and workflow states",
        example_query: r#"{ "query": "{ teams { nodes { id name key members { nodes { name email } } } } }" }"#,
    },
    LinearServiceEntry {
        aliases: &["linear-cycles", "linear-cycle"],
        description: "Cycles (sprints): list active, completed, and upcoming cycles",
        example_query: r#"{ "query": "{ cycles(filter: { completedAt: { null: true } }) { nodes { id number name startsAt endsAt completedAt } } }" }"#,
    },
    LinearServiceEntry {
        aliases: &["linear-views", "linear-view"],
        description: "Views: list and query saved issue filters",
        example_query: r#"{ "query": "{ customViews { nodes { id name description } } }" }"#,
    },
    LinearServiceEntry {
        aliases: &["linear-webhooks", "linear-webhook"],
        description: "Webhooks: create, list, and delete Linear webhooks",
        example_query: r#"{ "query": "{ webhooks { nodes { id url enabled createdAt } } }" }"#,
    },
    LinearServiceEntry {
        aliases: &["linear-me", "linear-user"],
        description: "Current user: profile, assigned issues, and notifications",
        example_query: r#"{ "query": "{ viewer { id name email assignedIssues { nodes { id title priority } } } }" }"#,
    },
    LinearServiceEntry {
        aliases: &["linear-labels", "linear-label"],
        description: "Labels: list and manage issue labels",
        example_query: r#"{ "query": "{ issueLabels { nodes { id name color } } }" }"#,
    },
];

/// Resolve a Linear service by alias.
pub fn resolve_linear_service(name: &str) -> Option<&'static LinearServiceEntry> {
    LINEAR_SERVICES.iter().find(|e| e.aliases.contains(&name))
}

// ─── Auth Config ──────────────────────────────────────────────

/// Authentication configuration for the Linear API.
#[derive(Debug, Clone)]
pub struct LinearAuthConfig {
    /// Personal API key (simplest auth method).
    pub api_key: Option<String>,
    /// OAuth2 client ID.
    pub client_id: Option<String>,
    /// OAuth2 client secret.
    pub client_secret: Option<String>,
    /// Pre-obtained OAuth2 access token.
    pub access_token: Option<String>,
}

impl LinearAuthConfig {
    /// Load Linear auth config from environment variables.
    ///
    /// Priority: `UWS_LINEAR_API_KEY` > `UWS_LINEAR_TOKEN` > OAuth2 client creds.
    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("UWS_LINEAR_API_KEY").ok();
        let access_token = std::env::var("UWS_LINEAR_TOKEN").ok();
        let client_id = std::env::var("UWS_LINEAR_CLIENT_ID").ok();
        let client_secret = std::env::var("UWS_LINEAR_CLIENT_SECRET").ok();

        if api_key.is_none() && access_token.is_none() && client_id.is_none() {
            return Err(anyhow!(
                "Linear credentials not found. Set UWS_LINEAR_API_KEY (recommended) or \
                 UWS_LINEAR_TOKEN / UWS_LINEAR_CLIENT_ID. \
                 See: https://github.com/splitmerge420/uws#linear-auth"
            ));
        }

        Ok(Self {
            api_key,
            client_id,
            client_secret,
            access_token,
        })
    }

    /// Returns the best available authorization header value.
    pub fn auth_header(&self) -> Option<String> {
        if let Some(key) = &self.api_key {
            return Some(key.clone()); // Linear uses the raw key, not "Bearer"
        }
        if let Some(token) = &self.access_token {
            return Some(format!("Bearer {token}"));
        }
        None
    }

    /// Returns true if any credential is available.
    pub fn is_configured(&self) -> bool {
        self.api_key.is_some()
            || self.access_token.is_some()
            || self.client_id.is_some()
    }
}

// ─── GraphQL Request Executor ─────────────────────────────────

/// Execute a Linear GraphQL query or mutation.
///
/// `query_json` must be a JSON string with at least a `"query"` field and
/// optionally a `"variables"` field.
pub async fn execute_linear_request(
    query_json: &str,
    auth: &str,
    dry_run: bool,
) -> Result<Value> {
    if dry_run {
        let dry = serde_json::json!({
            "dry_run": true,
            "url": LINEAR_API_BASE,
            "method": "POST",
            "body": query_json,
            "provider": "linear"
        });
        println!("{}", serde_json::to_string_pretty(&dry)?);
        return Ok(dry);
    }

    let client = reqwest::Client::new();
    let resp = client
        .post(LINEAR_API_BASE)
        .header("Content-Type", "application/json")
        .header("Authorization", auth)
        .body(query_json.to_string())
        .send()
        .await?;

    let status = resp.status();
    let text = resp.text().await?;

    if !status.is_success() {
        return Err(anyhow!("Linear API error {}: {}", status, text));
    }

    // GraphQL errors appear inside the 200 OK response body
    let json: Value = serde_json::from_str(&text)
        .unwrap_or_else(|_| Value::String(text));

    if let Some(errors) = json.get("errors") {
        return Err(anyhow!("Linear GraphQL errors: {}", errors));
    }

    Ok(json)
}

// ─── Preset Queries ───────────────────────────────────────────

/// Return the GraphQL query to fetch the authenticated user's assigned issues.
pub fn query_my_issues(limit: u32) -> String {
    serde_json::json!({
        "query": format!(
            r#"{{
                viewer {{
                    id name email
                    assignedIssues(first: {limit}) {{
                        nodes {{
                            id identifier title priority
                            state {{ name type }}
                            team {{ name key }}
                            createdAt updatedAt
                        }}
                    }}
                }}
            }}"#
        )
    })
    .to_string()
}

/// Return the GraphQL query to list all teams.
pub fn query_teams() -> String {
    serde_json::json!({
        "query": r#"{
            teams {
                nodes {
                    id name key description
                    members { nodes { id name email } }
                }
            }
        }"#
    })
    .to_string()
}

/// Return the GraphQL mutation to create a new issue.
pub fn mutation_create_issue(title: &str, team_id: &str, description: Option<&str>) -> String {
    let desc_field = description
        .map(|d| format!(r#"description: "{}""#, d.replace('"', "\\\"")))
        .unwrap_or_default();
    serde_json::json!({
        "query": format!(
            r#"mutation {{
                issueCreate(input: {{
                    title: "{title}"
                    teamId: "{team_id}"
                    {desc_field}
                }}) {{
                    success
                    issue {{ id identifier title url }}
                }}
            }}"#
        )
    })
    .to_string()
}

/// Return the GraphQL mutation to create a comment on an issue.
pub fn mutation_create_comment(issue_id: &str, body: &str) -> String {
    let escaped = body.replace('"', "\\\"");
    serde_json::json!({
        "query": format!(
            r#"mutation {{
                commentCreate(input: {{
                    issueId: "{issue_id}"
                    body: "{escaped}"
                }}) {{
                    success
                    comment {{ id body createdAt }}
                }}
            }}"#
        )
    })
    .to_string()
}

/// Return the GraphQL query to list active cycles across all teams.
pub fn query_active_cycles() -> String {
    serde_json::json!({
        "query": r#"{
            cycles(filter: { completedAt: { null: true } isActive: { eq: true } }) {
                nodes {
                    id number name
                    startsAt endsAt
                    team { name key }
                    issues { nodes { id title state { name } } }
                }
            }
        }"#
    })
    .to_string()
}

// ─── Auth Command Handler ─────────────────────────────────────

/// Handle the `linear-auth` subcommand tree.
pub async fn handle_linear_auth_command(args: &[String]) -> Result<()> {
    let subcommand = args.first().map(|s| s.as_str()).unwrap_or("help");

    match subcommand {
        "setup" => {
            println!("Linear Authentication Setup");
            println!("============================");
            println!();
            println!("Option A — Personal API Key (recommended for personal use):");
            println!("  1. Go to: https://linear.app/settings/api");
            println!("  2. Click 'Create key', name it 'uws CLI'");
            println!("  3. Copy the key");
            println!("  4. export UWS_LINEAR_API_KEY=lin_api_...");
            println!();
            println!("Option B — OAuth2 App (recommended for team use):");
            println!("  1. Go to: https://linear.app/settings/api (OAuth2 Applications section)");
            println!("  2. Create a new OAuth2 application named 'uws CLI'");
            println!("  3. Set redirect URI: http://localhost:8766/callback");
            println!("  4. Copy the Client ID and Client Secret");
            println!("  5. export UWS_LINEAR_CLIENT_ID=<client-id>");
            println!("  6. export UWS_LINEAR_CLIENT_SECRET=<client-secret>");
            println!();
            println!("Then run: uws linear-auth login");
        }
        "login" => {
            match LinearAuthConfig::from_env() {
                Ok(cfg) if cfg.api_key.is_some() => {
                    println!("{{\"status\": \"authenticated\", \"method\": \"api_key\", \"provider\": \"linear\"}}");
                }
                Ok(cfg) if cfg.client_id.is_some() => {
                    let client_id = cfg.client_id.as_deref().unwrap_or("");
                    let scopes = "read,write,issues:create";
                    println!("Open this URL in your browser:");
                    println!();
                    println!(
                        "{LINEAR_AUTH_URL}?client_id={client_id}&redirect_uri=http://localhost:8766/callback&response_type=code&scope={scopes}&actor=user"
                    );
                    println!();
                    println!("After approving, run: uws linear-auth exchange <code>");
                }
                Ok(_) => {
                    println!("No credentials configured. Run: uws linear-auth setup");
                }
                Err(e) => {
                    println!(
                        "{{\"status\": \"unauthenticated\", \"error\": \"{e}\", \"provider\": \"linear\"}}"
                    );
                }
            }
        }
        "status" => {
            match LinearAuthConfig::from_env() {
                Ok(cfg) => {
                    let method = if cfg.api_key.is_some() {
                        "api_key"
                    } else if cfg.access_token.is_some() {
                        "token"
                    } else {
                        "oauth_configured"
                    };
                    println!("{{\"status\": \"authenticated\", \"method\": \"{method}\", \"provider\": \"linear\"}}");
                }
                Err(e) => {
                    println!(
                        "{{\"status\": \"unauthenticated\", \"error\": \"{e}\", \"provider\": \"linear\"}}"
                    );
                }
            }
        }
        _ => {
            println!("Usage: uws linear-auth <subcommand>");
            println!();
            println!("Subcommands:");
            println!("  setup    Print step-by-step Linear API key / OAuth2 setup guide");
            println!("  login    Authenticate with Linear");
            println!("  status   Check current authentication status");
        }
    }
    Ok(())
}

// ─── Tests ────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_linear_service_issues() {
        let e = resolve_linear_service("linear-issues").unwrap();
        assert!(e.aliases.contains(&"linear-issues"));
        assert!(e.description.contains("Issues"));
    }

    #[test]
    fn test_resolve_linear_service_alias() {
        // Short alias works
        let e = resolve_linear_service("linear-issue").unwrap();
        assert!(e.aliases.contains(&"linear-issue"));
    }

    #[test]
    fn test_resolve_linear_service_unknown_returns_none() {
        assert!(resolve_linear_service("not-a-service").is_none());
    }

    #[test]
    fn test_all_services_have_unique_primary_alias() {
        let mut seen = std::collections::HashSet::new();
        for svc in LINEAR_SERVICES {
            let primary = svc.aliases[0];
            assert!(seen.insert(primary), "Duplicate alias: {primary}");
        }
    }

    #[test]
    fn test_linear_auth_config_no_env_returns_error() {
        // Unset relevant vars (best effort — may already be unset in CI)
        std::env::remove_var("UWS_LINEAR_API_KEY");
        std::env::remove_var("UWS_LINEAR_TOKEN");
        std::env::remove_var("UWS_LINEAR_CLIENT_ID");
        let result = LinearAuthConfig::from_env();
        assert!(result.is_err());
    }

    #[test]
    fn test_linear_auth_config_api_key() {
        std::env::set_var("UWS_LINEAR_API_KEY", "lin_api_test_key_123");
        let cfg = LinearAuthConfig::from_env().unwrap();
        assert!(cfg.is_configured());
        assert_eq!(cfg.auth_header(), Some("lin_api_test_key_123".to_string()));
        std::env::remove_var("UWS_LINEAR_API_KEY");
    }

    #[test]
    fn test_linear_auth_config_token() {
        std::env::remove_var("UWS_LINEAR_API_KEY");
        std::env::set_var("UWS_LINEAR_TOKEN", "tok_abc123");
        let cfg = LinearAuthConfig::from_env().unwrap();
        assert_eq!(cfg.auth_header(), Some("Bearer tok_abc123".to_string()));
        std::env::remove_var("UWS_LINEAR_TOKEN");
    }

    #[test]
    fn test_query_my_issues_valid_json() {
        let q = query_my_issues(10);
        let v: serde_json::Value = serde_json::from_str(&q).unwrap();
        assert!(v.get("query").is_some());
        let query_str = v["query"].as_str().unwrap();
        assert!(query_str.contains("assignedIssues"));
    }

    #[test]
    fn test_query_teams_valid_json() {
        let q = query_teams();
        let v: serde_json::Value = serde_json::from_str(&q).unwrap();
        assert!(v["query"].as_str().unwrap().contains("members"));
    }

    #[test]
    fn test_mutation_create_issue_valid_json() {
        let m = mutation_create_issue("Fix login bug", "team-uuid-123", Some("Details here"));
        let v: serde_json::Value = serde_json::from_str(&m).unwrap();
        let q = v["query"].as_str().unwrap();
        assert!(q.contains("issueCreate"));
        assert!(q.contains("Fix login bug"));
        assert!(q.contains("Details here"));
    }

    #[test]
    fn test_mutation_create_issue_no_description() {
        let m = mutation_create_issue("Refactor auth", "team-uuid-456", None);
        let v: serde_json::Value = serde_json::from_str(&m).unwrap();
        let q = v["query"].as_str().unwrap();
        assert!(q.contains("Refactor auth"));
        assert!(!q.contains("description:"));
    }

    #[test]
    fn test_mutation_create_comment_escapes_quotes() {
        let m = mutation_create_comment("issue-123", r#"She said "hello""#);
        let v: serde_json::Value = serde_json::from_str(&m).unwrap();
        let q = v["query"].as_str().unwrap();
        assert!(q.contains("commentCreate"));
        // Escaped quotes should not break JSON parsing
        assert!(q.contains("hello"));
    }

    #[test]
    fn test_query_active_cycles_valid_json() {
        let q = query_active_cycles();
        let v: serde_json::Value = serde_json::from_str(&q).unwrap();
        assert!(v["query"].as_str().unwrap().contains("cycles"));
    }

    #[test]
    fn test_all_services_have_descriptions() {
        for svc in LINEAR_SERVICES {
            assert!(!svc.description.is_empty(), "Service {:?} missing description", svc.aliases[0]);
        }
    }

    #[test]
    fn test_all_services_have_example_queries() {
        for svc in LINEAR_SERVICES {
            let v: serde_json::Value = serde_json::from_str(svc.example_query)
                .unwrap_or_else(|_| panic!("Invalid example_query JSON for {}", svc.aliases[0]));
            assert!(v.get("query").is_some(), "Missing 'query' field in example for {}", svc.aliases[0]);
        }
    }
}
