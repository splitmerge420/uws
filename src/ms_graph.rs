// Universal Workspace CLI (uws)
// Microsoft 365 integration via Microsoft Graph API
//
// Licensed under the Apache License, Version 2.0

#![allow(dead_code, unused_imports)]

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Microsoft Graph API base URL
pub const MS_GRAPH_BASE: &str = "https://graph.microsoft.com/v1.0";

/// Microsoft Graph API beta URL (for preview features)
pub const MS_GRAPH_BETA: &str = "https://graph.microsoft.com/beta";

/// Microsoft OAuth2 token endpoint
pub const MS_TOKEN_URL: &str = "https://login.microsoftonline.com/{tenant}/oauth2/v2.0/token";

/// Microsoft OAuth2 authorization endpoint
pub const MS_AUTH_URL: &str = "https://login.microsoftonline.com/{tenant}/oauth2/v2.0/authorize";

/// Known Microsoft Graph services mapped to their API paths and descriptions.
pub struct MsServiceEntry {
    pub aliases: &'static [&'static str],
    pub graph_path: &'static str,
    pub description: &'static str,
    pub scopes: &'static [&'static str],
}

/// All supported Microsoft 365 services.
pub const MS_SERVICES: &[MsServiceEntry] = &[
    MsServiceEntry {
        aliases: &["ms-mail", "outlook-mail"],
        graph_path: "/me/messages",
        description: "Outlook Mail: read, send, and manage email",
        scopes: &["Mail.ReadWrite", "Mail.Send"],
    },
    MsServiceEntry {
        aliases: &["ms-calendar", "outlook-calendar"],
        graph_path: "/me/calendar",
        description: "Outlook Calendar: events, meetings, and scheduling",
        scopes: &["Calendars.ReadWrite"],
    },
    MsServiceEntry {
        aliases: &["ms-onedrive", "onedrive"],
        graph_path: "/me/drive",
        description: "OneDrive: files, folders, and sharing",
        scopes: &["Files.ReadWrite.All"],
    },
    MsServiceEntry {
        aliases: &["ms-teams", "teams"],
        graph_path: "/me/joinedTeams",
        description: "Microsoft Teams: channels, messages, and meetings",
        scopes: &["Team.ReadBasic.All", "Channel.ReadBasic.All", "Chat.ReadWrite"],
    },
    MsServiceEntry {
        aliases: &["ms-todo", "todo"],
        graph_path: "/me/todo/lists",
        description: "Microsoft To Do: task lists and tasks",
        scopes: &["Tasks.ReadWrite"],
    },
    MsServiceEntry {
        aliases: &["ms-onenote", "onenote"],
        graph_path: "/me/onenote",
        description: "OneNote: notebooks, sections, and pages",
        scopes: &["Notes.ReadWrite.All"],
    },
    MsServiceEntry {
        aliases: &["ms-contacts", "outlook-contacts"],
        graph_path: "/me/contacts",
        description: "Outlook Contacts: people and contact groups",
        scopes: &["Contacts.ReadWrite"],
    },
    MsServiceEntry {
        aliases: &["ms-sharepoint", "sharepoint"],
        graph_path: "/sites",
        description: "SharePoint: sites, lists, and document libraries",
        scopes: &["Sites.ReadWrite.All"],
    },
    MsServiceEntry {
        aliases: &["ms-planner", "planner"],
        graph_path: "/me/planner/tasks",
        description: "Microsoft Planner: plans, buckets, and tasks",
        scopes: &["Tasks.ReadWrite"],
    },
    MsServiceEntry {
        aliases: &["ms-profile", "ms-me"],
        graph_path: "/me",
        description: "Microsoft profile: user info, photo, presence",
        scopes: &["User.Read"],
    },
];

/// Resolve a Microsoft service alias to its Graph API path.
pub fn resolve_ms_service(name: &str) -> Option<&'static MsServiceEntry> {
    MS_SERVICES
        .iter()
        .find(|e| e.aliases.contains(&name))
}

/// Microsoft Graph auth configuration loaded from environment variables.
#[derive(Debug, Clone)]
pub struct MsAuthConfig {
    pub client_id: String,
    pub client_secret: Option<String>,
    pub tenant_id: String,
    pub token: Option<String>,
}

impl MsAuthConfig {
    /// Load Microsoft auth config from environment variables.
    pub fn from_env() -> Result<Self> {
        // Pre-obtained token takes highest priority
        if let Ok(token) = std::env::var("UWS_MS_TOKEN") {
            return Ok(Self {
                client_id: String::new(),
                client_secret: None,
                tenant_id: "common".to_string(),
                token: Some(token),
            });
        }

        let client_id = std::env::var("UWS_MS_CLIENT_ID")
            .map_err(|_| anyhow!("UWS_MS_CLIENT_ID not set. See: https://github.com/splitmerge420/uws#microsoft-auth"))?;

        let tenant_id = std::env::var("UWS_MS_TENANT_ID")
            .unwrap_or_else(|_| "common".to_string());

        let client_secret = std::env::var("UWS_MS_CLIENT_SECRET").ok();

        Ok(Self {
            client_id,
            client_secret,
            tenant_id,
            token: None,
        })
    }

    /// Returns true if a pre-obtained token is available.
    pub fn has_token(&self) -> bool {
        self.token.is_some()
    }
}

/// Execute a Microsoft Graph API request.
pub async fn execute_graph_request(
    method: &str,
    path: &str,
    token: &str,
    params: Option<&str>,
    body: Option<&str>,
    dry_run: bool,
) -> Result<Value> {
    let client = reqwest::Client::new();

    // Build the full URL
    let base = if path.starts_with("/beta/") {
        MS_GRAPH_BETA
    } else {
        MS_GRAPH_BASE
    };
    let url = format!("{}{}", base, path);

    // Parse query params if provided
    let mut query_params: Vec<(String, String)> = Vec::new();
    if let Some(p) = params {
        if let Ok(obj) = serde_json::from_str::<serde_json::Map<String, Value>>(p) {
            for (k, v) in obj {
                let val = match &v {
                    Value::String(s) => s.clone(),
                    other => other.to_string(),
                };
                query_params.push((k, val));
            }
        }
    }

    if dry_run {
        let dry = serde_json::json!({
            "dry_run": true,
            "method": method,
            "url": url,
            "params": params,
            "body": body,
            "provider": "microsoft_graph"
        });
        println!("{}", serde_json::to_string_pretty(&dry)?);
        return Ok(dry);
    }

    let mut req = match method.to_uppercase().as_str() {
        "GET" => client.get(&url),
        "POST" => client.post(&url),
        "PATCH" => client.patch(&url),
        "PUT" => client.put(&url),
        "DELETE" => client.delete(&url),
        _ => return Err(anyhow!("Unsupported HTTP method: {}", method)),
    };

    req = req
        .bearer_auth(token)
        .header("Content-Type", "application/json")
        .query(&query_params);

    if let Some(b) = body {
        req = req.body(b.to_string());
    }

    let resp = req.send().await?;
    let status = resp.status();
    let text = resp.text().await?;

    if !status.is_success() {
        return Err(anyhow!(
            "Microsoft Graph API error {}: {}",
            status,
            text
        ));
    }

    let json: Value = serde_json::from_str(&text)
        .unwrap_or(Value::String(text));

    Ok(json)
}

pub async fn handle_ms_auth_command(args: &[String]) -> Result<()> {
    let subcommand = args.first().map(|s| s.as_str()).unwrap_or("help");

    match subcommand {
        "setup" => {
            println!("Microsoft 365 Authentication Setup");
            println!("====================================");
            println!();
            println!("1. Go to: https://portal.azure.com/#blade/Microsoft_AAD_RegisteredApps/ApplicationsListBlade");
            println!("2. Click 'New registration'");
            println!("3. Name: 'uws CLI', Supported account types: 'Personal Microsoft accounts'");
            println!("4. Redirect URI: http://localhost:8765/callback");
            println!("5. Copy the Application (client) ID");
            println!("6. Under 'Certificates & secrets', create a new client secret");
            println!("7. Under 'API permissions', add the scopes you need:");
            println!("   - Mail.ReadWrite, Mail.Send");
            println!("   - Calendars.ReadWrite");
            println!("   - Files.ReadWrite.All");
            println!("   - Team.ReadBasic.All");
            println!("   - Tasks.ReadWrite");
            println!("   - Notes.ReadWrite.All");
            println!("   - Contacts.ReadWrite");
            println!();
            println!("Then set environment variables:");
            println!("   export UWS_MS_CLIENT_ID=<your-client-id>");
            println!("   export UWS_MS_CLIENT_SECRET=<your-client-secret>");
            println!("   export UWS_MS_TENANT_ID=common");
            println!();
            println!("Then run: uws ms-auth login");
        }
        "login" => {
            let config = MsAuthConfig::from_env()?;
            let tenant = &config.tenant_id;
            let client_id = &config.client_id;
            let scopes = "offline_access Mail.ReadWrite Calendars.ReadWrite Files.ReadWrite.All Tasks.ReadWrite Notes.ReadWrite.All Contacts.ReadWrite";
            let auth_url = format!(
                "https://login.microsoftonline.com/{}/oauth2/v2.0/authorize?client_id={}&response_type=code&redirect_uri=http://localhost:8765/callback&scope={}",
                tenant, client_id, urlencoding(scopes)
            );
            println!("Open this URL in your browser to authenticate:");
            println!();
            println!("{}", auth_url);
            println!();
            println!("After approving, paste the 'code' parameter from the redirect URL here.");
            println!("Then exchange it with: uws ms-auth exchange <code>");
        }
        "status" => {
            match MsAuthConfig::from_env() {
                Ok(cfg) if cfg.has_token() => {
                    println!("{{\"status\": \"authenticated\", \"method\": \"token\", \"provider\": \"microsoft\"}}");
                }
                Ok(_) => {
                    println!("{{\"status\": \"configured\", \"method\": \"oauth\", \"provider\": \"microsoft\"}}");
                }
                Err(e) => {
                    println!("{{\"status\": \"unauthenticated\", \"error\": \"{}\", \"provider\": \"microsoft\"}}", e);
                }
            }
        }
        _ => {
            println!("Usage: uws ms-auth <subcommand>");
            println!();
            println!("Subcommands:");
            println!("  setup    Print step-by-step Azure app registration guide");
            println!("  login    Generate OAuth2 authorization URL");
            println!("  status   Check current authentication status");
        }
    }
    Ok(())
}

fn urlencoding(s: &str) -> String {
    percent_encoding::utf8_percent_encode(s, percent_encoding::NON_ALPHANUMERIC).to_string()
}

/// Parsed CLI flags: (params, body, method_override, dry_run, path_override, format).
type ProviderFlags = (Option<String>, Option<String>, Option<String>, bool, Option<String>, Option<String>);

/// Parse shared CLI flags from a slice of string args.
/// Extracts: --params, --json, --method, --dry-run, --path, --format.
/// Returns (params, body, method_override, dry_run, path_override, format).
fn parse_provider_flags(
    args: &[String],
) -> ProviderFlags {
    let mut params: Option<String> = None;
    let mut body: Option<String> = None;
    let mut method: Option<String> = None;
    let mut dry_run = false;
    let mut path: Option<String> = None;
    let mut format: Option<String> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--params" if i + 1 < args.len() => { params = Some(args[i + 1].clone()); i += 2; }
            "--json"   if i + 1 < args.len() => { body   = Some(args[i + 1].clone()); i += 2; }
            "--method" if i + 1 < args.len() => { method = Some(args[i + 1].clone()); i += 2; }
            "--path"   if i + 1 < args.len() => { path   = Some(args[i + 1].clone()); i += 2; }
            "--format" if i + 1 < args.len() => { format = Some(args[i + 1].clone()); i += 2; }
            "--dry-run" => { dry_run = true; i += 1; }
            _ => { i += 1; }
        }
    }
    (params, body, method, dry_run, path, format)
}

/// Dispatch a Microsoft 365 service command.
///
/// Routing rules:
/// - `--path <PATH>` overrides the default API path for this service.
/// - `--method <VERB>` overrides the inferred HTTP method.
/// - HTTP method defaults to POST when `--json` is present, GET otherwise.
/// - Common action aliases: `list` → GET, `create` → POST, `delete` → DELETE,
///   `update` → PATCH, `get` → GET.
///
/// # Examples
/// ```text
/// uws ms-mail messages list --params '{"$top":10}'
/// uws ms-mail --path /me/messages/sendMail --json '{"message":{...}}' --dry-run
/// ```
pub async fn handle_ms_command(service_name: &str, rest_args: &[String]) -> Result<()> {
    let entry = resolve_ms_service(service_name)
        .ok_or_else(|| anyhow!("Unknown Microsoft service: {service_name}"))?;

    let (params, body, method_flag, dry_run, path_flag, _fmt) = parse_provider_flags(rest_args);

    // Derive HTTP method: explicit flag > body present > action alias > GET
    let action = rest_args.iter().find(|a| !a.starts_with('-')).map(|s| s.as_str()).unwrap_or("");
    let http_method = method_flag.clone().unwrap_or_else(|| match action {
        "create" | "send" | "post"   => "POST".to_string(),
        "update" | "patch"           => "PATCH".to_string(),
        "delete" | "remove"          => "DELETE".to_string(),
        "put"                        => "PUT".to_string(),
        _  => if body.is_some() { "POST".to_string() } else { "GET".to_string() },
    });

    // API path: explicit --path > service default
    let api_path = path_flag.unwrap_or_else(|| entry.graph_path.to_string());

    // For dry-run, skip auth requirements and show request shape directly
    if dry_run {
        let info = serde_json::json!({
            "dry_run": true,
            "method": http_method,
            "url": format!("{}{}", MS_GRAPH_BASE, api_path),
            "params": params,
            "body": body,
            "provider": "microsoft_graph"
        });
        println!("{}", serde_json::to_string_pretty(&info)?);
        return Ok(());
    }

    let config = MsAuthConfig::from_env()?;
    let token = config.token.as_deref().unwrap_or("");

    let result = execute_graph_request(
        &http_method,
        &api_path,
        token,
        params.as_deref(),
        body.as_deref(),
        dry_run,
    ).await?;

    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}
