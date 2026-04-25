//! `uws ip` — Regenerative IP & Provenance Engine CLI commands.
//!
//! # Subcommands
//! - `uws ip sign`     — sign HEAD with a provenance trailer (or `--dry-run`)
//! - `uws ip monetize` — emit the Ledger registration payload
//!
//! # Entry point
//! Called from `src/main.rs` via the early-exit pattern used by `auth` and
//! `schema`.

use crate::error::GwsError;
use crate::ledger::provenance::{ProvenanceTrailer, RevenueSplit, MIN_HUMAN_WEIGHT};

/// Dispatch `uws ip <subcommand> [flags]`.
pub async fn handle_ip_command(args: &[String]) -> Result<(), GwsError> {
    let sub = args.first().map(|s| s.as_str()).unwrap_or("--help");

    match sub {
        "sign" => handle_sign(&args[1..]).await,
        "monetize" => handle_monetize(&args[1..]).await,
        "--help" | "-h" | "help" => {
            print_ip_help();
            Ok(())
        }
        other => Err(GwsError::Validation(format!(
            "Unknown `uws ip` subcommand: '{other}'. Run `uws ip --help` for usage."
        ))),
    }
}

// ─── uws ip sign ───────────────────────────────────────────────────────────

/// Handle `uws ip sign [--dry-run] [--human-weight N] [--ai-weight N]
///                     [--human-share N] [--ai-share N] [--author EMAIL]`.
async fn handle_sign(args: &[String]) -> Result<(), GwsError> {
    let dry_run = args.iter().any(|a| a == "--dry-run");
    let human_weight = parse_flag_f64(args, "--human-weight").unwrap_or(0.7);
    let ai_weight = parse_flag_f64(args, "--ai-weight").unwrap_or(1.0 - human_weight);
    let human_share = parse_flag_f64(args, "--human-share").unwrap_or(human_weight);
    let ai_share = parse_flag_f64(args, "--ai-share").unwrap_or(ai_weight);
    let author = parse_flag_str(args, "--author");

    // Print help if requested
    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_sign_help();
        return Ok(());
    }

    // Validate human_weight before building the revenue split, so error
    // messages are ordered sensibly.
    if human_weight < MIN_HUMAN_WEIGHT {
        return Err(GwsError::Validation(format!(
            "--human-weight ({human_weight}) is below the minimum allowed value \
             ({MIN_HUMAN_WEIGHT}).  All IP requires at least {MIN_HUMAN_WEIGHT} \
             human contribution."
        )));
    }

    let revenue_split =
        RevenueSplit::new(human_share, ai_share).map_err(GwsError::Validation)?;

    // Resolve HEAD commit SHA (best-effort; None if not in a git repo)
    let commit_sha = resolve_head_sha();

    let signed_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let trailer = ProvenanceTrailer::new(
        human_weight,
        ai_weight,
        revenue_split,
        signed_at,
        commit_sha,
        author,
    )
    .map_err(GwsError::Validation)?;

    if dry_run {
        // Print the trailer JSON and exit without touching git state.
        let json = trailer.to_json().map_err(GwsError::Validation)?;
        println!("{json}");
        return Ok(());
    }

    // Attach as a git note on HEAD.
    attach_git_note(&trailer)?;
    eprintln!("✓ Provenance trailer attached to HEAD via git notes.");
    Ok(())
}

// ─── uws ip monetize ───────────────────────────────────────────────────────

/// Handle `uws ip monetize [--dry-run] [--human-weight N] [--ai-weight N]
///                          [--human-share N] [--ai-share N] [--author EMAIL]`.
async fn handle_monetize(args: &[String]) -> Result<(), GwsError> {
    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_monetize_help();
        return Ok(());
    }

    let human_weight = parse_flag_f64(args, "--human-weight").unwrap_or(0.7);
    let ai_weight = parse_flag_f64(args, "--ai-weight").unwrap_or(1.0 - human_weight);
    let human_share = parse_flag_f64(args, "--human-share").unwrap_or(human_weight);
    let ai_share = parse_flag_f64(args, "--ai-share").unwrap_or(ai_weight);
    let author = parse_flag_str(args, "--author");

    if human_weight < MIN_HUMAN_WEIGHT {
        return Err(GwsError::Validation(format!(
            "--human-weight ({human_weight}) is below the minimum allowed value \
             ({MIN_HUMAN_WEIGHT})."
        )));
    }

    let revenue_split =
        RevenueSplit::new(human_share, ai_share).map_err(GwsError::Validation)?;
    let commit_sha = resolve_head_sha();
    let signed_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let trailer = ProvenanceTrailer::new(
        human_weight,
        ai_weight,
        revenue_split,
        signed_at,
        commit_sha,
        author,
    )
    .map_err(GwsError::Validation)?;

    let payload = trailer
        .to_monetize_payload()
        .map_err(GwsError::Validation)?;

    let payload_str =
        serde_json::to_string_pretty(&payload).map_err(|e| GwsError::Validation(e.to_string()))?;

    // Gate live Ledger API calls behind UWS_LEDGER_URL.
    match std::env::var("UWS_LEDGER_URL") {
        Ok(url) if !url.is_empty() => {
            eprintln!("UWS_LEDGER_URL is set ({url}) — live Ledger integration is not yet\
             implemented in this release.  The payload below is ready to POST:\n");
            println!("{payload_str}");
        }
        _ => {
            println!("{payload_str}");
            eprintln!(
                "\n(Set UWS_LEDGER_URL to enable live registration once the Ledger API is available.)"
            );
        }
    }

    Ok(())
}

// ─── Git helpers ────────────────────────────────────────────────────────────

/// Resolve the current HEAD commit SHA using `git rev-parse HEAD`.
///
/// Returns `None` if not in a git repository or git is unavailable.
fn resolve_head_sha() -> Option<String> {
    std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

/// Attach the trailer to the HEAD commit via `git notes add`.
///
/// The trailer is serialised as a single-line JSON string (serde_json escapes
/// any embedded special characters, so it is safe to pass via a shell
/// argument).
fn attach_git_note(trailer: &ProvenanceTrailer) -> Result<(), GwsError> {
    let note = trailer.to_git_note().map_err(GwsError::Validation)?;

    // Use -f to overwrite any existing uws-ip note on HEAD.
    let status = std::process::Command::new("git")
        .args(["notes", "--ref=uws-ip", "add", "-f", "-m", &note])
        .status()
        .map_err(|e| GwsError::Validation(format!("Failed to run `git notes add`: {e}")))?;

    if !status.success() {
        return Err(GwsError::Validation(
            "`git notes add` exited with a non-zero status. \
             Are you inside a git repository?"
                .to_string(),
        ));
    }
    Ok(())
}

// ─── Flag parsers ───────────────────────────────────────────────────────────

fn parse_flag_f64(args: &[String], flag: &str) -> Option<f64> {
    args.windows(2)
        .find(|w| w[0] == flag)
        .and_then(|w| w[1].parse::<f64>().ok())
}

fn parse_flag_str(args: &[String], flag: &str) -> Option<String> {
    args.windows(2)
        .find(|w| w[0] == flag)
        .map(|w| w[1].clone())
}

// ─── Help text ──────────────────────────────────────────────────────────────

fn print_ip_help() {
    println!("uws ip — Regenerative IP & Provenance Engine");
    println!();
    println!("USAGE:");
    println!("    uws ip <subcommand> [flags]");
    println!();
    println!("SUBCOMMANDS:");
    println!("    sign       Sign HEAD with a provenance trailer (or --dry-run to preview)");
    println!("    monetize   Emit the Ledger registration payload");
    println!();
    println!("Run `uws ip <subcommand> --help` for subcommand-specific flags.");
}

fn print_sign_help() {
    println!("uws ip sign — Sign the current HEAD commit with a provenance trailer");
    println!();
    println!("USAGE:");
    println!("    uws ip sign [flags]");
    println!();
    println!("FLAGS:");
    println!("    --dry-run              Print the trailer JSON without modifying git state");
    println!("    --human-weight <N>     Human contribution fraction (default: 0.7, min: {MIN_HUMAN_WEIGHT})");
    println!("    --ai-weight <N>        AI contribution fraction   (default: 1 − human-weight)");
    println!("    --human-share <N>      Human revenue share        (default: human-weight)");
    println!("    --ai-share <N>         AI revenue share           (default: ai-weight)");
    println!("    --author <EMAIL>       Author identifier          (default: none)");
    println!();
    println!("NOTES:");
    println!("    The trailer is attached as a git note in the 'uws-ip' ref namespace.");
    println!("    Retrieve with: git notes --ref=uws-ip show HEAD");
}

fn print_monetize_help() {
    println!("uws ip monetize — Emit the Ledger registration payload");
    println!();
    println!("USAGE:");
    println!("    uws ip monetize [flags]");
    println!();
    println!("FLAGS:");
    println!("    --human-weight <N>     Human contribution fraction (default: 0.7, min: {MIN_HUMAN_WEIGHT})");
    println!("    --ai-weight <N>        AI contribution fraction   (default: 1 − human-weight)");
    println!("    --human-share <N>      Human revenue share        (default: human-weight)");
    println!("    --ai-share <N>         AI revenue share           (default: ai-weight)");
    println!("    --author <EMAIL>       Author identifier          (default: none)");
    println!();
    println!("ENVIRONMENT:");
    println!("    UWS_LEDGER_URL         When set, the payload is displayed ready to POST.");
    println!("                           Live Ledger integration is a planned follow-up.");
}
