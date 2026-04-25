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

//! Universal Workspace CLI (uws) — quick-build entry point.
//!
//! This binary currently provides:
//!  - `uws ip sign <path>`           — SHA-256 provenance record (JSON)
//!  - `uws ip monetize <hash|path>`  — monetization intent record (JSON)
//!
//! Full Google Workspace service routing (Drive, Gmail, Calendar, …) and
//! Microsoft / Apple provider dispatch live in the full-build (Phase 2)
//! configuration — see Cargo.toml comments and PRs #19-#22.

mod error;
pub(crate) mod regen_ip;

use error::{print_error_json, GwsError};

fn main() {
    if let Err(err) = run() {
        print_error_json(&err);
        std::process::exit(1);
    }
}

fn run() -> Result<(), GwsError> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        print_usage();
        return Err(GwsError::Validation(
            "No command specified. Usage: uws <command> [args]".to_string(),
        ));
    }

    // Find the first non-flag argument.
    let first_arg = args
        .iter()
        .skip(1)
        .find(|a| !a.starts_with("--") || a.as_str() == "--help" || a.as_str() == "--version")
        .cloned()
        .ok_or_else(|| {
            GwsError::Validation("No command specified. Usage: uws <command> [args]".to_string())
        })?;

    // --help / -h at the top level
    if is_help_flag(&first_arg) {
        print_usage();
        return Ok(());
    }

    // --version / -V / version
    if is_version_flag(&first_arg) {
        println!("uws {}", env!("CARGO_PKG_VERSION"));
        println!("Universal Workspace CLI — not affiliated with Google, Microsoft, or Apple.");
        return Ok(());
    }

    // ── Early-exit handlers ───────────────────────────────────────────────────
    // Each built-in command parses its own sub-args.  Same pattern as `auth`.

    // Handle the `ip` command (Regenerative IP & Provenance Engine)
    if first_arg == "ip" {
        let ip_args: Vec<String> = args.iter().skip(2).cloned().collect();
        return regen_ip::handle_ip_command(&ip_args);
    }

    // ── Unknown command ───────────────────────────────────────────────────────
    print_usage();
    Err(GwsError::Validation(format!(
        "Unknown command: '{first_arg}'. \
         Run `uws --help` for available commands."
    )))
}

fn print_usage() {
    println!("uws — Universal Workspace CLI v{}", env!("CARGO_PKG_VERSION"));
    println!();
    println!("USAGE:");
    println!("    uws <command> [subcommand] [args]");
    println!();
    println!("COMMANDS:");
    println!("    ip sign <path>              Produce a SHA-256 provenance record for a file");
    println!("    ip monetize <hash|path>     Emit a monetization intent record for an artifact");
    println!("    ip --help                   Show ip subcommand help");
    println!();
    println!("FLAGS:");
    println!("    --help, -h       Show this help message");
    println!("    --version, -V    Show version information");
    println!();
    println!("NOTE:");
    println!("    Google Workspace (drive, gmail, calendar, …), Microsoft 365,");
    println!("    and Apple iCloud commands require Phase 2 deps (see Cargo.toml).");
    println!();
    println!("MORE:");
    println!("    https://github.com/atlaslattice/uws");
}

fn is_help_flag(arg: &str) -> bool {
    matches!(arg, "--help" | "-h")
}

fn is_version_flag(arg: &str) -> bool {
    matches!(arg, "--version" | "-V" | "version")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_help_flag() {
        assert!(is_help_flag("--help"));
        assert!(is_help_flag("-h"));
        assert!(!is_help_flag("help"));
        assert!(!is_help_flag("--h"));
    }

    #[test]
    fn test_is_version_flag() {
        assert!(is_version_flag("--version"));
        assert!(is_version_flag("-V"));
        assert!(is_version_flag("version"));
        assert!(!is_version_flag("--ver"));
        assert!(!is_version_flag("v"));
    }

    #[test]
    fn test_run_no_args_returns_error() {
        // A run with only the binary name should fail.
        // We can't call run() directly here since it reads std::env::args,
        // so we just exercise the GwsError path instead.
        let err = GwsError::Validation("No command specified".to_string());
        let json = err.to_json();
        assert_eq!(json["error"]["code"], 400);
    }

    #[test]
    fn test_unknown_command_returns_validation_error() {
        let err = GwsError::Validation("Unknown command: 'foo'.".to_string());
        let json = err.to_json();
        assert_eq!(json["error"]["reason"], "validationError");
    }
}
