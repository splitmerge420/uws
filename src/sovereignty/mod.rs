//! Sovereignty Layer — Cultural adapters and multi-provider auth for UWS.
//!
//! This module is deliberately isolated from the core discovery-driven CLI
//! so that downstream forks (DragonSeek, JinnSeek, etc.) can take this
//! directory and the thin routing block in main.rs without touching the
//! rest of the codebase.
//!
//! # Architecture
//! ```text
//! uws translate --adapter dragonseek --content "Hello"
//!       |
//!       v
//! main.rs routes to sovereignty::handle_translate_command()
//!       |
//!       v
//! sovereignty/translate.rs -> adapter registry -> JSON output
//! ```

pub mod translate;

use crate::error::GwsError;

/// Handle `uws ms-auth` — Microsoft Graph OAuth flow.
pub async fn handle_ms_auth_command(args: &[String]) -> Result<(), GwsError> {
    // TODO: Wire to existing ms_graph auth handler once module is ready.
    // For now, print structured JSON indicating the command was recognized.
    let action = args.first().map(|s| s.as_str()).unwrap_or("status");
    let output = serde_json::json!({
        "command": "ms-auth",
        "action": action,
        "status": "not_yet_implemented",
        "message": "Microsoft Graph OAuth will be available in the next release. Use `uws auth login` for Google OAuth in the meantime."
    });
    println!("{}", serde_json::to_string_pretty(&output).unwrap());
    Ok(())
}

/// Handle `uws apple-auth` — Apple iCloud OAuth flow.
pub async fn handle_apple_auth_command(args: &[String]) -> Result<(), GwsError> {
    let action = args.first().map(|s| s.as_str()).unwrap_or("status");
    let output = serde_json::json!({
        "command": "apple-auth",
        "action": action,
        "status": "not_yet_implemented",
        "message": "Apple iCloud OAuth will be available in the next release. Use `uws auth login` for Google OAuth in the meantime."
    });
    println!("{}", serde_json::to_string_pretty(&output).unwrap());
    Ok(())
}

/// Handle `uws translate` — Cultural sovereignty adapter.
///
/// Usage:
///   uws translate --adapter <name> --content <string>
///   uws translate --adapter <name> --content-file <path>
///
/// Exactly one of --content or --content-file must be provided.
/// Output is always JSON.
pub async fn handle_translate_command(args: &[String]) -> Result<(), GwsError> {
    translate::run(args).await
}