// Universal Workspace CLI (uws)
// Apple ecosystem integration via Apple APIs and iCloud
//
// Licensed under the Apache License, Version 2.0

#![allow(dead_code, unused_imports)]

use anyhow::{anyhow, Result};
use serde_json::Value;

/// Apple iCloud base URL
pub const APPLE_ICLOUD_BASE: &str = "https://p01-caldav.icloud.com";

/// Apple Sign In token endpoint
pub const APPLE_TOKEN_URL: &str = "https://appleid.apple.com/auth/token";

/// Apple Sign In authorization endpoint
pub const APPLE_AUTH_URL: &str = "https://appleid.apple.com/auth/authorize";

/// Known Apple ecosystem services.
pub struct AppleServiceEntry {
    pub aliases: &'static [&'static str],
    pub protocol: AppleProtocol,
    pub description: &'static str,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AppleProtocol {
    /// CalDAV for Calendar and Reminders
    CalDAV,
    /// CardDAV for Contacts
    CardDAV,
    /// iCloud Drive (CloudKit / private API)
    CloudKit,
    /// Apple Sign In (OAuth2-based)
    SignIn,
}

/// All supported Apple ecosystem services.
pub const APPLE_SERVICES: &[AppleServiceEntry] = &[
    AppleServiceEntry {
        aliases: &["apple-calendar", "ical"],
        protocol: AppleProtocol::CalDAV,
        description: "iCloud Calendar: events, reminders, and shared calendars (CalDAV)",
    },
    AppleServiceEntry {
        aliases: &["apple-reminders", "reminders"],
        protocol: AppleProtocol::CalDAV,
        description: "Reminders: task lists and reminders (CalDAV VTODO)",
    },
    AppleServiceEntry {
        aliases: &["apple-contacts", "icloud-contacts"],
        protocol: AppleProtocol::CardDAV,
        description: "iCloud Contacts: people, groups, and vCards (CardDAV)",
    },
    AppleServiceEntry {
        aliases: &["apple-drive", "icloud-drive"],
        protocol: AppleProtocol::CloudKit,
        description: "iCloud Drive: files and folders (CloudKit)",
    },
    AppleServiceEntry {
        aliases: &["apple-notes", "icloud-notes"],
        protocol: AppleProtocol::CloudKit,
        description: "iCloud Notes: notes and folders (CloudKit)",
    },
    AppleServiceEntry {
        aliases: &["apple-signin", "apple-auth"],
        protocol: AppleProtocol::SignIn,
        description: "Sign in with Apple: OAuth2 identity and token management",
    },
];

/// Resolve an Apple service alias.
pub fn resolve_apple_service(name: &str) -> Option<&'static AppleServiceEntry> {
    APPLE_SERVICES
        .iter()
        .find(|e| e.aliases.contains(&name))
}

/// Apple auth configuration loaded from environment variables.
#[derive(Debug, Clone)]
pub struct AppleAuthConfig {
    pub client_id: Option<String>,
    pub team_id: Option<String>,
    pub key_id: Option<String>,
    pub private_key_file: Option<String>,
    /// iCloud app-specific password (for CalDAV/CardDAV)
    pub app_password: Option<String>,
    /// Apple ID username (email)
    pub apple_id: Option<String>,
}

impl AppleAuthConfig {
    /// Load Apple auth config from environment variables.
    pub fn from_env() -> Self {
        Self {
            client_id: std::env::var("UWS_APPLE_CLIENT_ID").ok(),
            team_id: std::env::var("UWS_APPLE_TEAM_ID").ok(),
            key_id: std::env::var("UWS_APPLE_KEY_ID").ok(),
            private_key_file: std::env::var("UWS_APPLE_PRIVATE_KEY_FILE").ok(),
            app_password: std::env::var("UWS_APPLE_APP_PASSWORD").ok(),
            apple_id: std::env::var("UWS_APPLE_ID").ok(),
        }
    }

    /// Returns true if CalDAV/CardDAV credentials are available.
    pub fn has_caldav_credentials(&self) -> bool {
        self.apple_id.is_some() && self.app_password.is_some()
    }
}

/// Execute a CalDAV request against iCloud Calendar.
pub async fn execute_caldav_request(
    method: &str,
    path: &str,
    apple_id: &str,
    app_password: &str,
    body: Option<&str>,
    dry_run: bool,
) -> Result<Value> {
    let url = format!("{}{}", APPLE_ICLOUD_BASE, path);

    if dry_run {
        let dry = serde_json::json!({
            "dry_run": true,
            "method": method,
            "url": url,
            "body": body,
            "provider": "apple_caldav"
        });
        println!("{}", serde_json::to_string_pretty(&dry)?);
        return Ok(dry);
    }

    let client = reqwest::Client::new();
    let mut req = match method.to_uppercase().as_str() {
        "GET" | "PROPFIND" | "REPORT" => client.request(
            reqwest::Method::from_bytes(method.as_bytes())
                .unwrap_or(reqwest::Method::GET),
            &url,
        ),
        "PUT" => client.put(&url),
        "DELETE" => client.delete(&url),
        _ => return Err(anyhow!("Unsupported CalDAV method: {}", method)),
    };

    req = req
        .basic_auth(apple_id, Some(app_password))
        .header("Content-Type", "text/xml; charset=utf-8");

    if let Some(b) = body {
        req = req.body(b.to_string());
    }

    let resp = req.send().await?;
    let status = resp.status();
    let text = resp.text().await?;

    if !status.is_success() && status.as_u16() != 207 {
        // 207 Multi-Status is normal for CalDAV PROPFIND/REPORT
        return Err(anyhow!("Apple CalDAV error {}: {}", status, text));
    }

    // Return raw XML wrapped in JSON for consistent output
    Ok(serde_json::json!({
        "status": status.as_u16(),
        "provider": "apple_caldav",
        "raw": text
    }))
}

/// Execute a CardDAV request against iCloud Contacts.
pub async fn execute_carddav_request(
    method: &str,
    path: &str,
    apple_id: &str,
    app_password: &str,
    body: Option<&str>,
    dry_run: bool,
) -> Result<Value> {
    let base = "https://contacts.icloud.com";
    let url = format!("{}{}", base, path);

    if dry_run {
        let dry = serde_json::json!({
            "dry_run": true,
            "method": method,
            "url": url,
            "body": body,
            "provider": "apple_carddav"
        });
        println!("{}", serde_json::to_string_pretty(&dry)?);
        return Ok(dry);
    }

    let client = reqwest::Client::new();
    let req = client
        .request(
            reqwest::Method::from_bytes(method.as_bytes())
                .unwrap_or(reqwest::Method::GET),
            &url,
        )
        .basic_auth(apple_id, Some(app_password))
        .header("Content-Type", "text/xml; charset=utf-8");

    let req = if let Some(b) = body {
        req.body(b.to_string())
    } else {
        req
    };

    let resp = req.send().await?;
    let status = resp.status();
    let text = resp.text().await?;

    Ok(serde_json::json!({
        "status": status.as_u16(),
        "provider": "apple_carddav",
        "raw": text
    }))
}

/// Handle the `apple-auth` command flow.
pub async fn handle_apple_auth_command(args: &[String]) -> Result<()> {
    let subcommand = args.first().map(|s| s.as_str()).unwrap_or("help");

    match subcommand {
        "setup" => {
            println!("Apple Ecosystem Authentication Setup");
            println!("=====================================");
            println!();
            println!("Option A: iCloud CalDAV/CardDAV (Calendar, Contacts, Reminders)");
            println!("----------------------------------------------------------------");
            println!("1. Go to: https://appleid.apple.com/account/manage");
            println!("2. Under 'Sign-In and Security', click 'App-Specific Passwords'");
            println!("3. Generate a new password for 'uws CLI'");
            println!("4. Set environment variables:");
            println!("   export UWS_APPLE_ID=your@apple.com");
            println!("   export UWS_APPLE_APP_PASSWORD=xxxx-xxxx-xxxx-xxxx");
            println!();
            println!("Option B: Sign in with Apple (OAuth2 for apps)");
            println!("------------------------------------------------");
            println!("1. Go to: https://developer.apple.com/account/resources/identifiers/list/serviceId");
            println!("2. Register a Services ID (e.g., com.yourapp.uws)");
            println!("3. Enable 'Sign In with Apple' and configure domains/redirect URIs");
            println!("4. Create a private key under Keys > Sign In with Apple");
            println!("5. Download the .p8 key file");
            println!("6. Set environment variables:");
            println!("   export UWS_APPLE_CLIENT_ID=com.yourapp.uws");
            println!("   export UWS_APPLE_TEAM_ID=XXXXXXXXXX");
            println!("   export UWS_APPLE_KEY_ID=XXXXXXXXXX");
            println!("   export UWS_APPLE_PRIVATE_KEY_FILE=/path/to/AuthKey_XXXXXXXXXX.p8");
        }
        "status" => {
            let cfg = AppleAuthConfig::from_env();
            if cfg.has_caldav_credentials() {
                println!("{{\"status\": \"authenticated\", \"method\": \"app_password\", \"provider\": \"apple\", \"apple_id\": \"{}\"}}",
                    cfg.apple_id.as_deref().unwrap_or(""));
            } else if cfg.client_id.is_some() {
                println!("{{\"status\": \"configured\", \"method\": \"sign_in_with_apple\", \"provider\": \"apple\"}}");
            } else {
                println!("{{\"status\": \"unauthenticated\", \"provider\": \"apple\", \"hint\": \"Run: uws apple-auth setup\"}}");
            }
        }
        _ => {
            println!("Usage: uws apple-auth <subcommand>");
            println!();
            println!("Subcommands:");
            println!("  setup    Print step-by-step Apple authentication guide");
            println!("  status   Check current authentication status");
        }
    }
    Ok(())
}

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

/// Dispatch an Apple service command.
///
/// Routing is protocol-aware:
/// - CalDAV services (`apple-calendar`, `apple-reminders`) → `execute_caldav_request`
/// - CardDAV services (`apple-contacts`) → `execute_carddav_request`
/// - CloudKit and SignIn services → print a not-yet-implemented notice
///
/// The HTTP method defaults to PROPFIND for CalDAV/CardDAV discovery, PUT for
/// creates, and DELETE for deletes.  Use `--method` to override.
///
/// # Examples
/// ```text
/// uws apple-calendar --path /dav/1234/calendars/ --dry-run
/// uws apple-contacts --path /carddav/1234/contacts/ --method PROPFIND
/// ```
pub async fn handle_apple_command(service_name: &str, rest_args: &[String]) -> Result<()> {
    let entry = resolve_apple_service(service_name)
        .ok_or_else(|| anyhow!("Unknown Apple service: {service_name}"))?;

    let config = AppleAuthConfig::from_env();
    let (params, body, method_flag, dry_run, path_flag) = parse_flags(rest_args);

    let action = rest_args.iter().find(|a| !a.starts_with('-')).map(|s| s.as_str()).unwrap_or("list");
    let http_method = method_flag.unwrap_or_else(|| match action {
        "create" | "put"   => "PUT".to_string(),
        "delete" | "remove" => "DELETE".to_string(),
        "report"            => "REPORT".to_string(),
        _ => "PROPFIND".to_string(),
    });

    let api_path = path_flag.unwrap_or_else(|| "/".to_string());

    // For dry-run, skip auth requirements and just show request shape
    if dry_run {
        let info = serde_json::json!({
            "dry_run": true,
            "service": service_name,
            "protocol": format!("{:?}", entry.protocol),
            "method": http_method,
            "path": api_path,
            "body": body,
            "provider": "apple"
        });
        println!("{}", serde_json::to_string_pretty(&info)?);
        return Ok(());
    }

    match entry.protocol {
        AppleProtocol::CalDAV => {
            let apple_id = config.apple_id
                .as_deref()
                .ok_or_else(|| anyhow!("UWS_APPLE_ID not set"))?;
            let app_password = config.app_password
                .as_deref()
                .ok_or_else(|| anyhow!("UWS_APPLE_APP_PASSWORD not set"))?;
            let result = execute_caldav_request(
                &http_method, &api_path, apple_id, app_password, body.as_deref(), dry_run,
            ).await?;
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        AppleProtocol::CardDAV => {
            let apple_id = config.apple_id
                .as_deref()
                .ok_or_else(|| anyhow!("UWS_APPLE_ID not set"))?;
            let app_password = config.app_password
                .as_deref()
                .ok_or_else(|| anyhow!("UWS_APPLE_APP_PASSWORD not set"))?;
            let _ = params; // CardDAV doesn't use query params
            let result = execute_carddav_request(
                &http_method, &api_path, apple_id, app_password, body.as_deref(), dry_run,
            ).await?;
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        AppleProtocol::CloudKit | AppleProtocol::SignIn => {
            let info = serde_json::json!({
                "service": service_name,
                "protocol": format!("{:?}", entry.protocol),
                "status": "not_yet_implemented",
                "note": "CloudKit and Sign in with Apple require native SDK integration. Use UWS_APPLE_CLIENT_ID / UWS_APPLE_TEAM_ID / UWS_APPLE_KEY_ID / UWS_APPLE_PRIVATE_KEY_FILE env vars.",
                "dry_run": dry_run,
                "path": api_path,
            });
            println!("{}", serde_json::to_string_pretty(&info)?);
        }
    }
    Ok(())
}
