// src/slack_provider.rs
// Aluminum OS — Slack Provider
//
// Exposes the Slack Web API through the `uws slack` command surface,
// giving AI agents and developers clean, JSON-first access to Slack
// channels, messages, users, reactions, files, and more.
//
// Command grammar:
//   uws slack <resource> <method> [--params <JSON>] [--json <JSON>] [--dry-run]
//
// Examples:
//   uws slack channels list
//   uws slack channels history --params '{"channel":"C1234567890"}'
//   uws slack messages post --params '{"channel":"general"}' --json '{"text":"Hello!"}'
//   uws slack users list
//   uws slack users info --params '{"user":"U1234567890"}'
//   uws slack files list --params '{"channel":"C1234567890"}'
//   uws slack search messages --params '{"query":"budget Q4"}'
//
// Authentication:
//   Set SLACK_BOT_TOKEN or UWS_SLACK_TOKEN in the environment.
//   Requires a Slack app with appropriate OAuth scopes.
//
// Author: GitHub Copilot — Aluminum OS Session 2026-03-23

#![allow(dead_code)]

use std::collections::BTreeMap;

// ─── Service aliases ──────────────────────────────────────────────────────

pub const SLACK_ALIASES: &[&str] = &["slack"];

pub fn is_slack_service(name: &str) -> bool {
    SLACK_ALIASES.contains(&name)
}

// ─── API base ─────────────────────────────────────────────────────────────

pub const SLACK_API_BASE: &str = "https://slack.com/api";

// ─── HTTP method ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SlackMethod {
    Get,
    Post,
}

impl SlackMethod {
    pub fn as_str(&self) -> &str {
        match self {
            SlackMethod::Get => "GET",
            SlackMethod::Post => "POST",
        }
    }
}

// ─── Endpoint catalogue ───────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct SlackEndpoint {
    pub resource: &'static str,
    pub method: &'static str,
    pub http_method: SlackMethod,
    pub api_method: &'static str,
    pub requires_body: bool,
    pub description: &'static str,
}

pub const SLACK_ENDPOINTS: &[SlackEndpoint] = &[
    // ── Conversations (channels) ─────────────────────────────────────
    SlackEndpoint {
        resource: "channels",
        method: "list",
        http_method: SlackMethod::Get,
        api_method: "conversations.list",
        requires_body: false,
        description: "List all channels in the workspace",
    },
    SlackEndpoint {
        resource: "channels",
        method: "info",
        http_method: SlackMethod::Get,
        api_method: "conversations.info",
        requires_body: false,
        description: "Get information about a channel",
    },
    SlackEndpoint {
        resource: "channels",
        method: "history",
        http_method: SlackMethod::Get,
        api_method: "conversations.history",
        requires_body: false,
        description: "Get message history for a channel",
    },
    SlackEndpoint {
        resource: "channels",
        method: "members",
        http_method: SlackMethod::Get,
        api_method: "conversations.members",
        requires_body: false,
        description: "List members of a channel",
    },
    SlackEndpoint {
        resource: "channels",
        method: "join",
        http_method: SlackMethod::Post,
        api_method: "conversations.join",
        requires_body: true,
        description: "Join a channel",
    },
    SlackEndpoint {
        resource: "channels",
        method: "create",
        http_method: SlackMethod::Post,
        api_method: "conversations.create",
        requires_body: true,
        description: "Create a new channel",
    },
    SlackEndpoint {
        resource: "channels",
        method: "archive",
        http_method: SlackMethod::Post,
        api_method: "conversations.archive",
        requires_body: true,
        description: "Archive a channel",
    },
    // ── Messages ─────────────────────────────────────────────────────
    SlackEndpoint {
        resource: "messages",
        method: "post",
        http_method: SlackMethod::Post,
        api_method: "chat.postMessage",
        requires_body: true,
        description: "Post a message to a channel or user",
    },
    SlackEndpoint {
        resource: "messages",
        method: "update",
        http_method: SlackMethod::Post,
        api_method: "chat.update",
        requires_body: true,
        description: "Update an existing message",
    },
    SlackEndpoint {
        resource: "messages",
        method: "delete",
        http_method: SlackMethod::Post,
        api_method: "chat.delete",
        requires_body: true,
        description: "Delete a message",
    },
    SlackEndpoint {
        resource: "messages",
        method: "schedule",
        http_method: SlackMethod::Post,
        api_method: "chat.scheduleMessage",
        requires_body: true,
        description: "Schedule a message to be sent later",
    },
    SlackEndpoint {
        resource: "messages",
        method: "permalink",
        http_method: SlackMethod::Get,
        api_method: "chat.getPermalink",
        requires_body: false,
        description: "Get a permanent link to a message",
    },
    // ── Thread replies ───────────────────────────────────────────────
    SlackEndpoint {
        resource: "threads",
        method: "replies",
        http_method: SlackMethod::Get,
        api_method: "conversations.replies",
        requires_body: false,
        description: "Get replies in a thread",
    },
    SlackEndpoint {
        resource: "threads",
        method: "reply",
        http_method: SlackMethod::Post,
        api_method: "chat.postMessage",
        requires_body: true,
        description: "Reply to a thread (set thread_ts in body)",
    },
    // ── Users ────────────────────────────────────────────────────────
    SlackEndpoint {
        resource: "users",
        method: "list",
        http_method: SlackMethod::Get,
        api_method: "users.list",
        requires_body: false,
        description: "List all users in the workspace",
    },
    SlackEndpoint {
        resource: "users",
        method: "info",
        http_method: SlackMethod::Get,
        api_method: "users.info",
        requires_body: false,
        description: "Get information about a user",
    },
    SlackEndpoint {
        resource: "users",
        method: "profile",
        http_method: SlackMethod::Get,
        api_method: "users.profile.get",
        requires_body: false,
        description: "Get a user's profile",
    },
    SlackEndpoint {
        resource: "users",
        method: "presence",
        http_method: SlackMethod::Get,
        api_method: "users.getPresence",
        requires_body: false,
        description: "Get a user's presence status",
    },
    // ── Reactions ────────────────────────────────────────────────────
    SlackEndpoint {
        resource: "reactions",
        method: "add",
        http_method: SlackMethod::Post,
        api_method: "reactions.add",
        requires_body: true,
        description: "Add an emoji reaction to a message",
    },
    SlackEndpoint {
        resource: "reactions",
        method: "remove",
        http_method: SlackMethod::Post,
        api_method: "reactions.remove",
        requires_body: true,
        description: "Remove an emoji reaction from a message",
    },
    SlackEndpoint {
        resource: "reactions",
        method: "get",
        http_method: SlackMethod::Get,
        api_method: "reactions.get",
        requires_body: false,
        description: "Get reactions for a message",
    },
    // ── Files ────────────────────────────────────────────────────────
    SlackEndpoint {
        resource: "files",
        method: "list",
        http_method: SlackMethod::Get,
        api_method: "files.list",
        requires_body: false,
        description: "List files in the workspace",
    },
    SlackEndpoint {
        resource: "files",
        method: "info",
        http_method: SlackMethod::Get,
        api_method: "files.info",
        requires_body: false,
        description: "Get information about a file",
    },
    SlackEndpoint {
        resource: "files",
        method: "delete",
        http_method: SlackMethod::Post,
        api_method: "files.delete",
        requires_body: true,
        description: "Delete a file",
    },
    SlackEndpoint {
        resource: "files",
        method: "share",
        http_method: SlackMethod::Post,
        api_method: "files.sharedPublicURL",
        requires_body: true,
        description: "Share a file publicly",
    },
    // ── Search ───────────────────────────────────────────────────────
    SlackEndpoint {
        resource: "search",
        method: "messages",
        http_method: SlackMethod::Get,
        api_method: "search.messages",
        requires_body: false,
        description: "Search messages in the workspace",
    },
    SlackEndpoint {
        resource: "search",
        method: "files",
        http_method: SlackMethod::Get,
        api_method: "search.files",
        requires_body: false,
        description: "Search files in the workspace",
    },
    SlackEndpoint {
        resource: "search",
        method: "all",
        http_method: SlackMethod::Get,
        api_method: "search.all",
        requires_body: false,
        description: "Search all content in the workspace",
    },
    // ── DMs / IMs ────────────────────────────────────────────────────
    SlackEndpoint {
        resource: "dm",
        method: "open",
        http_method: SlackMethod::Post,
        api_method: "conversations.open",
        requires_body: true,
        description: "Open a direct message channel",
    },
    SlackEndpoint {
        resource: "dm",
        method: "history",
        http_method: SlackMethod::Get,
        api_method: "conversations.history",
        requires_body: false,
        description: "Get DM history with a user",
    },
    // ── Workspace ────────────────────────────────────────────────────
    SlackEndpoint {
        resource: "workspace",
        method: "info",
        http_method: SlackMethod::Get,
        api_method: "team.info",
        requires_body: false,
        description: "Get information about the workspace/team",
    },
    SlackEndpoint {
        resource: "workspace",
        method: "emoji",
        http_method: SlackMethod::Get,
        api_method: "emoji.list",
        requires_body: false,
        description: "List custom emoji in the workspace",
    },
    // ── Status ───────────────────────────────────────────────────────
    SlackEndpoint {
        resource: "status",
        method: "get",
        http_method: SlackMethod::Get,
        api_method: "users.profile.get",
        requires_body: false,
        description: "Get the authenticated user's status",
    },
    SlackEndpoint {
        resource: "status",
        method: "set",
        http_method: SlackMethod::Post,
        api_method: "users.profile.set",
        requires_body: true,
        description: "Set the authenticated user's status",
    },
    // ── Reminders ────────────────────────────────────────────────────
    SlackEndpoint {
        resource: "reminders",
        method: "add",
        http_method: SlackMethod::Post,
        api_method: "reminders.add",
        requires_body: true,
        description: "Create a reminder",
    },
    SlackEndpoint {
        resource: "reminders",
        method: "list",
        http_method: SlackMethod::Get,
        api_method: "reminders.list",
        requires_body: false,
        description: "List all reminders",
    },
    SlackEndpoint {
        resource: "reminders",
        method: "delete",
        http_method: SlackMethod::Post,
        api_method: "reminders.delete",
        requires_body: true,
        description: "Delete a reminder",
    },
    // ── Bookmarks ────────────────────────────────────────────────────
    SlackEndpoint {
        resource: "bookmarks",
        method: "list",
        http_method: SlackMethod::Get,
        api_method: "bookmarks.list",
        requires_body: false,
        description: "List bookmarks in a channel",
    },
    SlackEndpoint {
        resource: "bookmarks",
        method: "add",
        http_method: SlackMethod::Post,
        api_method: "bookmarks.add",
        requires_body: true,
        description: "Add a bookmark to a channel",
    },
];

// ─── Error type ──────────────────────────────────────────────────────────

#[derive(Debug)]
pub enum SlackError {
    EndpointNotFound { resource: String, method: String },
    TokenNotFound,
    ParseError(String),
}

impl std::fmt::Display for SlackError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SlackError::EndpointNotFound { resource, method } => {
                write!(f, "Unknown Slack endpoint: {resource} {method}")
            }
            SlackError::TokenNotFound => write!(
                f,
                "Slack token not found. Set SLACK_BOT_TOKEN or UWS_SLACK_TOKEN."
            ),
            SlackError::ParseError(s) => write!(f, "Parse error: {s}"),
        }
    }
}

// ─── Token resolution ─────────────────────────────────────────────────────

/// Resolve the Slack bot token from environment variables.
pub fn resolve_slack_token() -> Option<String> {
    std::env::var("UWS_SLACK_TOKEN")
        .ok()
        .or_else(|| std::env::var("SLACK_BOT_TOKEN").ok())
}

// ─── Endpoint lookup ──────────────────────────────────────────────────────

pub fn find_endpoint(resource: &str, method: &str) -> Option<&'static SlackEndpoint> {
    SLACK_ENDPOINTS
        .iter()
        .find(|e| e.resource == resource && e.method == method)
}

// ─── Request builder ──────────────────────────────────────────────────────

/// Represents a fully constructed Slack API request (no I/O).
#[derive(Debug, Clone)]
pub struct SlackRequest {
    pub http_method: String,
    pub url: String,
    pub params: BTreeMap<String, String>,
    pub body: Option<BTreeMap<String, String>>,
    pub auth_header: String,
    pub dry_run: bool,
}

/// Build a Slack API request from CLI inputs.
///
/// # Arguments
/// * `endpoint` - The matched endpoint descriptor.
/// * `params` - Query/path parameters from `--params`.
/// * `body` - Request body fields from `--json`.
/// * `token` - Slack bot token.
/// * `dry_run` - If true, the token is redacted in the returned request.
pub fn build_request(
    endpoint: &SlackEndpoint,
    params: BTreeMap<String, String>,
    body: Option<BTreeMap<String, String>>,
    token: &str,
    dry_run: bool,
) -> SlackRequest {
    let url = format!("{}/{}", SLACK_API_BASE, endpoint.api_method);
    let auth_token = if dry_run { "[REDACTED]".to_string() } else { token.to_string() };

    SlackRequest {
        http_method: endpoint.http_method.as_str().to_string(),
        url,
        params,
        body,
        auth_header: format!("Bearer {auth_token}"),
        dry_run,
    }
}

// ─── Dry-run output ──────────────────────────────────────────────────────

pub fn format_dry_run(req: &SlackRequest, endpoint: &SlackEndpoint) -> String {
    let mut parts = vec![
        format!("\"method\": \"{}\"", req.http_method),
        format!("\"url\": \"{}\"", req.url),
        format!("\"description\": \"{}\"", endpoint.description),
    ];
    if !req.params.is_empty() {
        let ps: Vec<String> = req.params.iter().map(|(k, v)| format!("\"{k}\": \"{v}\"")).collect();
        parts.push(format!("\"params\": {{{}}}", ps.join(", ")));
    }
    if let Some(b) = &req.body {
        let bs: Vec<String> = b.iter().map(|(k, v)| format!("\"{k}\": \"{v}\"")).collect();
        parts.push(format!("\"body\": {{{}}}", bs.join(", ")));
    }
    format!("{{\n  {}\n}}", parts.join(",\n  "))
}

// ─── Unit tests ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_slack_service() {
        assert!(is_slack_service("slack"));
        assert!(!is_slack_service("github"));
        assert!(!is_slack_service("gmail"));
    }

    #[test]
    fn test_endpoint_catalogue_nonempty() {
        assert!(!SLACK_ENDPOINTS.is_empty());
    }

    #[test]
    fn test_find_channels_list() {
        let ep = find_endpoint("channels", "list").unwrap();
        assert_eq!(ep.api_method, "conversations.list");
        assert_eq!(ep.http_method, SlackMethod::Get);
    }

    #[test]
    fn test_find_messages_post() {
        let ep = find_endpoint("messages", "post").unwrap();
        assert_eq!(ep.api_method, "chat.postMessage");
        assert!(ep.requires_body);
    }

    #[test]
    fn test_find_unknown_returns_none() {
        assert!(find_endpoint("nonexistent", "list").is_none());
    }

    #[test]
    fn test_build_request_url() {
        let ep = find_endpoint("channels", "history").unwrap();
        let mut params = BTreeMap::new();
        params.insert("channel".to_string(), "C1234567890".to_string());
        let req = build_request(ep, params, None, "xoxb-test-token", false);
        assert_eq!(req.url, "https://slack.com/api/conversations.history");
        assert_eq!(req.http_method, "GET");
    }

    #[test]
    fn test_build_request_dry_run_redacts_token() {
        let ep = find_endpoint("messages", "post").unwrap();
        let params = BTreeMap::new();
        let mut body = BTreeMap::new();
        body.insert("channel".to_string(), "general".to_string());
        body.insert("text".to_string(), "Hello!".to_string());
        let req = build_request(ep, params, Some(body), "xoxb-real-token", true);
        assert!(req.auth_header.contains("[REDACTED]"));
        assert!(!req.auth_header.contains("xoxb-real-token"));
    }

    #[test]
    fn test_format_dry_run_output() {
        let ep = find_endpoint("search", "messages").unwrap();
        let mut params = BTreeMap::new();
        params.insert("query".to_string(), "budget Q4".to_string());
        let req = build_request(ep, params, None, "token", true);
        let out = format_dry_run(&req, ep);
        assert!(out.contains("search.messages"));
        assert!(out.contains("budget Q4"));
    }

    #[test]
    fn test_all_post_endpoints_require_body() {
        for ep in SLACK_ENDPOINTS {
            if ep.http_method == SlackMethod::Post && !ep.requires_body {
                // Some POST endpoints allow empty body — just verify they're intentional
                // by checking the api_method is not a mutation
                assert!(
                    ep.api_method.contains("list") || ep.api_method.contains("get"),
                    "POST endpoint {} {} should require body",
                    ep.resource,
                    ep.method
                );
            }
        }
    }

    #[test]
    fn test_error_display_token_not_found() {
        let e = SlackError::TokenNotFound;
        assert!(e.to_string().contains("SLACK_BOT_TOKEN"));
    }

    #[test]
    fn test_error_display_endpoint_not_found() {
        let e = SlackError::EndpointNotFound {
            resource: "foo".to_string(),
            method: "bar".to_string(),
        };
        assert!(e.to_string().contains("foo bar"));
    }

    #[test]
    fn test_search_endpoints_present() {
        assert!(find_endpoint("search", "messages").is_some());
        assert!(find_endpoint("search", "files").is_some());
        assert!(find_endpoint("search", "all").is_some());
    }

    #[test]
    fn test_users_endpoints_present() {
        assert!(find_endpoint("users", "list").is_some());
        assert!(find_endpoint("users", "info").is_some());
    }

    #[test]
    fn test_reactions_endpoints_present() {
        assert!(find_endpoint("reactions", "add").is_some());
        assert!(find_endpoint("reactions", "remove").is_some());
    }

    #[test]
    fn test_slack_api_base_constant() {
        assert_eq!(SLACK_API_BASE, "https://slack.com/api");
    }

    #[test]
    fn test_unique_resource_method_pairs() {
        let mut seen = std::collections::HashSet::new();
        for ep in SLACK_ENDPOINTS {
            let key = format!("{}/{}", ep.resource, ep.method);
            assert!(seen.insert(key.clone()), "Duplicate endpoint: {key}");
        }
    }

    #[test]
    fn test_reminders_endpoints_present() {
        assert!(find_endpoint("reminders", "add").is_some());
        assert!(find_endpoint("reminders", "list").is_some());
        assert!(find_endpoint("reminders", "delete").is_some());
    }

    #[test]
    fn test_workspace_endpoints_present() {
        assert!(find_endpoint("workspace", "info").is_some());
        assert!(find_endpoint("workspace", "emoji").is_some());
    }
}
