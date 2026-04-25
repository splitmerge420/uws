// Copyright 2026 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! uws — Universal Workspace CLI (constitutional quick-build entry point)
//!
//! This binary is the quick-build mode entry point that wires the
//! Zero Trust Integration Gate into CLI dispatch.  The full provider
//! dispatch (Google Discovery routing, Microsoft Graph, Apple CalDAV,
//! Android Management) lives in the product-fork binary and requires
//! external dependencies (tokio, clap, reqwest, serde_json) that are
//! beyond the scope of this constitutional library crate.
//!
//! ## Wired call sites (Zero Trust Gate)
//! - `uws ms-auth login` → `IntegrationKind::ProviderAuth`  ← **example wired here**
//!
//! ## TODO: remaining call sites to adopt the gate
//! - `uws auth login`         → `IntegrationKind::ProviderAuth`
//! - `uws apple-auth`         → `IntegrationKind::ProviderAuth`
//! - `uws ms-auth exchange`   → `IntegrationKind::NetworkEgress`
//! - `uws <service> * *`      → `IntegrationKind::NetworkEgress`  (executor::execute_method)
//! - `--output-dir <path>`    → `IntegrationKind::FileWrite`       (validate::validate_safe_output_dir)
//! - `--upload <path>`        → `IntegrationKind::FileWrite`       (executor upload path)

use uws::zero_trust::{GateContext, GateDecision, GatePolicy, IntegrationKind, ZeroTrustGate};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let service = args.get(1).map(|s| s.as_str()).unwrap_or("");
    let subcommand = args.get(2).map(|s| s.as_str()).unwrap_or("");

    match (service, subcommand) {
        ("ms-auth", "login") => {
            handle_ms_auth_login();
        }
        ("ms-auth", _) => {
            eprintln!("Usage: uws ms-auth login");
            std::process::exit(1);
        }
        // TODO: wire gate for `uws auth login`       → IntegrationKind::ProviderAuth
        // TODO: wire gate for `uws apple-auth`       → IntegrationKind::ProviderAuth
        // TODO: wire gate for `uws ms-auth exchange` → IntegrationKind::NetworkEgress
        // TODO: wire gate for all provider calls     → IntegrationKind::NetworkEgress
        //       in executor::execute_method before the reqwest call
        // TODO: wire gate for --output-dir flag      → IntegrationKind::FileWrite
        //       in validate::validate_safe_output_dir
        // TODO: wire gate for --upload flag          → IntegrationKind::FileWrite
        //       in executor upload path
        _ => {
            print_usage();
        }
    }
}

/// Handle `uws ms-auth login`.
///
/// This is the **example wired call site** for the Zero Trust Integration Gate.
/// The gate runs BEFORE the OAuth2 authorization URL is generated or any
/// network connection is made.
fn handle_ms_auth_login() {
    // ── Zero Trust Gate ───────────────────────────────────────────────────────
    // All provider auth flows must pass the gate before proceeding.
    // Policy is loaded from the environment or defaults to permissive for the
    // quick-build mode.  In a hardened deployment, replace GatePolicy::permissive()
    // with a policy loaded from a config file or env var.
    let policy = gate_policy_from_env();
    let gate = ZeroTrustGate::new(policy);

    let ctx = GateContext {
        kind: IntegrationKind::ProviderAuth,
        resource: "microsoft/oauth2",
        actor: "uws-cli",
    };

    match gate.check(&ctx) {
        GateDecision::Allow => {}
        GateDecision::Block(reason) => {
            eprintln!(
                "{{\"error\":\"ZERO_TRUST_BLOCK\",\"reason\":\"{reason}\"}}",
                reason = reason.replace('"', "'"),
            );
            std::process::exit(1);
        }
    }
    // ── Gate passed ───────────────────────────────────────────────────────────

    // TODO: Full OAuth2 flow (requires UWS_MS_CLIENT_ID / reqwest / tokio).
    // In the full product binary (Phase 2), replace this with MsAuthConfig::from_env()
    // and the actual OAuth2 authorization URL generation.
    println!(
        "{{\"status\":\"pending\",\
         \"message\":\"Zero Trust Gate: ALLOW — ms-auth login OAuth2 flow not yet wired in quick-build mode. Set UWS_MS_CLIENT_ID and run the full binary.\"}}"
    );
}

/// Build a `GatePolicy` from the environment.
///
/// | Env var                    | Effect                          |
/// |----------------------------|---------------------------------|
/// | `UWS_ZERO_TRUST_MODE=deny` | Block all integrations          |
/// | (not set / anything else)  | Permissive (all integrations OK)|
fn gate_policy_from_env() -> GatePolicy {
    match std::env::var("UWS_ZERO_TRUST_MODE")
        .as_deref()
        .unwrap_or("")
    {
        "deny" => GatePolicy::deny_all(),
        _ => GatePolicy::permissive(),
    }
}

fn print_usage() {
    println!("uws — Universal Workspace CLI");
    println!();
    println!("USAGE:");
    println!("    uws <service> <command> [args]");
    println!();
    println!("WIRED SERVICES (quick-build mode):");
    println!("    ms-auth login    Authenticate with Microsoft 365 (via Zero Trust Gate)");
    println!();
    println!("ENVIRONMENT:");
    println!("    UWS_ZERO_TRUST_MODE=deny   Block all external integrations (deny_all policy)");
    println!("    UWS_MS_CLIENT_ID           Azure app client ID (required for ms-auth login)");
    println!("    UWS_ZERO_TRUST_AUDIT_LOG   Path for Zero Trust audit log file");
    println!();
    println!("NOTE:");
    println!("    This is the constitutional quick-build entry point.");
    println!("    The full provider dispatch binary is in the product fork.");
    println!("    See AGENTS.md and invariants/README.md for governance details.");
}
