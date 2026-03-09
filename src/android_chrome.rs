// Universal Workspace CLI (uws)
// Android and Chrome ecosystem integration
//
// Licensed under the Apache License, Version 2.0

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
        .unwrap_or_else(|_| Value::String(text));

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
