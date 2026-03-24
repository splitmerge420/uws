// Universal Workspace CLI (uws)
// Android and Chrome ecosystem integration
//
// Licensed under the Apache License, Version 2.0

#![allow(dead_code, unused_imports)]

use anyhow::{anyhow, Result};
use serde_json::Value;

/// Android Management API base URL
pub const ANDROID_MGMT_BASE: &str = "https://androidmanagement.googleapis.com/v1";

/// Chrome Management API base URL
pub const CHROME_MGMT_BASE: &str = "https://chromemanagement.googleapis.com/v1";

/// Chrome Policy API base URL
pub const CHROME_POLICY_BASE: &str = "https://chromepolicy.googleapis.com/v1";

/// Chrome Web Store API base URL
pub const CHROME_STORE_BASE: &str = "https://www.googleapis.com/chromewebstore/v1.1";

/// Google Messages (via Google Messages for Web / RCS Business Messaging)
pub const GOOGLE_MESSAGES_BASE: &str = "https://rcsbusinessmessaging.googleapis.com/v1";

/// Known Android and Chrome services.
pub struct AndroidChromeServiceEntry {
    pub aliases: &'static [&'static str],
    pub base_url: &'static str,
    pub description: &'static str,
    pub scopes: &'static [&'static str],
    pub ecosystem: Ecosystem,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Ecosystem {
    Android,
    Chrome,
    Both,
}

/// All supported Android and Chrome services.
pub const ANDROID_CHROME_SERVICES: &[AndroidChromeServiceEntry] = &[
    AndroidChromeServiceEntry {
        aliases: &["android-management", "android-mgmt"],
        base_url: ANDROID_MGMT_BASE,
        description: "Android Management API: enterprise device and policy management",
        scopes: &["https://www.googleapis.com/auth/androidmanagement"],
        ecosystem: Ecosystem::Android,
    },
    AndroidChromeServiceEntry {
        aliases: &["android-messages", "messages"],
        base_url: GOOGLE_MESSAGES_BASE,
        description: "Google Messages: RCS Business Messaging API",
        scopes: &["https://www.googleapis.com/auth/businessmessages"],
        ecosystem: Ecosystem::Android,
    },
    AndroidChromeServiceEntry {
        aliases: &["chrome-management", "chrome-mgmt"],
        base_url: CHROME_MGMT_BASE,
        description: "Chrome Management API: device telemetry and app reports",
        scopes: &[
            "https://www.googleapis.com/auth/chrome.management.reports.readonly",
            "https://www.googleapis.com/auth/chrome.management.telemetry.readonly",
        ],
        ecosystem: Ecosystem::Chrome,
    },
    AndroidChromeServiceEntry {
        aliases: &["chrome-policy"],
        base_url: CHROME_POLICY_BASE,
        description: "Chrome Policy API: manage Chrome policies for users and devices",
        scopes: &[
            "https://www.googleapis.com/auth/chrome.management.policy",
        ],
        ecosystem: Ecosystem::Chrome,
    },
    AndroidChromeServiceEntry {
        aliases: &["chrome-extensions", "webstore"],
        base_url: CHROME_STORE_BASE,
        description: "Chrome Web Store API: extension and app management",
        scopes: &[
            "https://www.googleapis.com/auth/chromewebstore",
        ],
        ecosystem: Ecosystem::Chrome,
    },
    AndroidChromeServiceEntry {
        aliases: &["chrome-devices", "chromeos"],
        base_url: "https://admin.googleapis.com/admin/directory/v1",
        description: "ChromeOS devices: list, manage, and deprovision Chromebooks",
        scopes: &[
            "https://www.googleapis.com/auth/admin.directory.device.chromeos",
        ],
        ecosystem: Ecosystem::Chrome,
    },
    AndroidChromeServiceEntry {
        aliases: &["android-devices"],
        base_url: "https://admin.googleapis.com/admin/directory/v1",
        description: "Android devices: list and manage Android devices in Google Workspace",
        scopes: &[
            "https://www.googleapis.com/auth/admin.directory.device.mobile",
        ],
        ecosystem: Ecosystem::Android,
    },
];

/// Resolve an Android/Chrome service alias.
pub fn resolve_android_chrome_service(name: &str) -> Option<&'static AndroidChromeServiceEntry> {
    ANDROID_CHROME_SERVICES
        .iter()
        .find(|e| e.aliases.contains(&name))
}

/// Execute an Android or Chrome API request using a Google OAuth token.
pub async fn execute_android_chrome_request(
    method: &str,
    base_url: &str,
    path: &str,
    token: &str,
    params: Option<&str>,
    body: Option<&str>,
    dry_run: bool,
) -> Result<Value> {
    let url = format!("{}{}", base_url, path);

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
            "provider": "android_chrome"
        });
        println!("{}", serde_json::to_string_pretty(&dry)?);
        return Ok(dry);
    }

    let client = reqwest::Client::new();
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
            "Android/Chrome API error {}: {}",
            status,
            text
        ));
    }

    let json: Value = serde_json::from_str(&text)
        .unwrap_or(Value::String(text));

    Ok(json)
}

/// Print Android/Chrome service listing for --help output.
pub fn print_android_chrome_services() {
    println!("ANDROID SERVICES:");
    for entry in ANDROID_CHROME_SERVICES
        .iter()
        .filter(|e| e.ecosystem == Ecosystem::Android || e.ecosystem == Ecosystem::Both)
    {
        println!("    {:<25} {}", entry.aliases[0], entry.description);
    }
    println!();
    println!("CHROME SERVICES:");
    for entry in ANDROID_CHROME_SERVICES
        .iter()
        .filter(|e| e.ecosystem == Ecosystem::Chrome || e.ecosystem == Ecosystem::Both)
    {
        println!("    {:<25} {}", entry.aliases[0], entry.description);
    }
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

/// Dispatch an Android or Chrome service command.
///
/// Uses the service's registered `base_url` and appends the `--path` override
/// (or empty string for the service root).  The HTTP method defaults to POST
/// when `--json` is present, GET otherwise.
///
/// # Examples
/// ```text
/// uws android --params '{"name":"enterprises/acme/devices"}' --path /enterprises/acme/devices
/// uws chrome-mgmt --path /customers/acme/reports/printUsage --dry-run
/// ```
pub async fn handle_android_chrome_command(service_name: &str, rest_args: &[String]) -> Result<()> {
    let entry = resolve_android_chrome_service(service_name)
        .ok_or_else(|| anyhow!("Unknown Android/Chrome service: {service_name}"))?;

    let (params, body, method_flag, dry_run, path_flag) = parse_flags(rest_args);

    let action = rest_args.iter().find(|a| !a.starts_with('-')).map(|s| s.as_str()).unwrap_or("list");

    let http_method = method_flag.unwrap_or_else(|| match action {
        "create" | "post" => "POST".to_string(),
        "update" | "patch" => "PATCH".to_string(),
        "delete" | "remove" => "DELETE".to_string(),
        _ => if body.is_some() { "POST".to_string() } else { "GET".to_string() },
    });

    let api_path = path_flag.unwrap_or_default();

    // Android/Chrome services use a Google OAuth2 token (same as other Google services)
    let token = std::env::var("GOOGLE_WORKSPACE_CLI_TOKEN")
        .or_else(|_| std::env::var("GOOGLE_OAUTH_TOKEN"))
        .unwrap_or_default();

    let result = execute_android_chrome_request(
        &http_method,
        entry.base_url,
        &api_path,
        &token,
        params.as_deref(),
        body.as_deref(),
        dry_run,
    ).await?;

    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}
