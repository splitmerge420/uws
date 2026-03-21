// src/swarm.rs
// Aluminum OS — Swarm Commander
//
// Implements `uws swarm review --batch=<n>`:
//   1. Fetches a mock batch of open PRs / dependency updates
//   2. Runs a local dry-run NPFM (Net-Positive Flourishing Metric) check on each item
//   3. Produces a single cryptographic sign-off over the approved batch
//
// Author: GitHub Copilot — Council Session 2026-03-21
// Invariants Enforced: INV-2 (Consent Gating), INV-3 (Audit Trail)

#![allow(dead_code)]

use std::time::{SystemTime, UNIX_EPOCH};

// ─── Batch Item ────────────────────────────────────────────────

/// A single PR or dependency update that belongs to the review batch.
#[derive(Debug, Clone)]
pub struct BatchItem {
    /// Numeric identifier (PR number or dependency index)
    pub id: u32,
    /// Human-readable title
    pub title: String,
    /// Kind of item
    pub kind: BatchItemKind,
    /// Whether the dry-run NPFM check passed
    pub npfm_passed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BatchItemKind {
    PullRequest,
    DependencyUpdate,
}

impl std::fmt::Display for BatchItemKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BatchItemKind::PullRequest => write!(f, "PR"),
            BatchItemKind::DependencyUpdate => write!(f, "dep"),
        }
    }
}

// ─── Review Outcome ────────────────────────────────────────────

/// Result of running `swarm review --batch=<n>`.
#[derive(Debug)]
pub struct SwarmReviewResult {
    /// Items that passed the NPFM dry-run and are approved
    pub approved: Vec<BatchItem>,
    /// Items that failed the NPFM check and are blocked
    pub blocked: Vec<BatchItem>,
    /// Cryptographic sign-off over the approved batch
    pub sign_off: String,
}

// ─── NPFM Dry-Run ──────────────────────────────────────────────

/// Run a local, offline NPFM (Net-Positive Flourishing Metric) check on a
/// batch item. Returns `true` when the item is approved.
///
/// In this V1 the check is a simple title heuristic:
/// - Items whose titles contain busywork signals (e.g. "bump version",
///   "auto-generated", "format-only", "chore") are blocked.
/// - Everything else is approved.
///
/// Replace with the full linter integration in V2.
pub fn npfm_dry_run(item: &BatchItem) -> bool {
    let lower = item.title.to_lowercase();
    let busywork_signals = [
        "bump version",
        "auto-generated",
        "format only",
        "format-only",
        "chore: ",
        "chore(",
    ];
    !busywork_signals.iter().any(|s| lower.contains(s))
}

// ─── Mock Data ─────────────────────────────────────────────────

/// Generate `count` mock batch items simulating open PRs and dependency
/// updates. In production this would call the GitHub API or read from the
/// local repository.
pub fn mock_batch(count: u32) -> Vec<BatchItem> {
    let templates = [
        ("fix: correct nil-check in executor", BatchItemKind::PullRequest),
        ("chore: bump version v0.1.1 → v0.1.2", BatchItemKind::DependencyUpdate),
        ("feat: add golden trace provenance trailer", BatchItemKind::PullRequest),
        ("refactor: extract helper function", BatchItemKind::PullRequest),
        ("chore(deps): auto-generated lock file update", BatchItemKind::DependencyUpdate),
        ("fix: validate path traversal in output-dir", BatchItemKind::PullRequest),
        ("deps: serde 1.0.228 → 1.0.229", BatchItemKind::DependencyUpdate),
        ("feat: wire swarm review CLI command", BatchItemKind::PullRequest),
        ("format-only: rustfmt pass on helpers/", BatchItemKind::PullRequest),
        ("fix: handle empty iterator in audit chain", BatchItemKind::PullRequest),
    ];

    templates
        .iter()
        .cycle()
        .take(count as usize)
        .enumerate()
        .map(|(i, (title, kind))| BatchItem {
            id: (i + 1) as u32,
            title: title.to_string(),
            kind: kind.clone(),
            npfm_passed: false, // filled in by run_swarm_review
        })
        .collect()
}

// ─── Sign-Off ──────────────────────────────────────────────────

/// Produce a deterministic, portable sign-off string over the approved item
/// IDs and the current Unix timestamp.
///
/// Format: `SwarmSignOff: batch=<ids>, ts=<unix>, sig=<hex>`
///
/// The hex signature is an FNV-1a 64-bit hash — adequate for an offline
/// dry-run sign-off. Replace with a real Ed25519 signature in production.
pub fn compute_sign_off(approved_ids: &[u32]) -> String {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let payload = format!("batch={:?},ts={}", approved_ids, ts);
    let sig = fnv1a_64(payload.as_bytes());

    let ids_str = approved_ids
        .iter()
        .map(|id| id.to_string())
        .collect::<Vec<_>>()
        .join(",");

    format!(
        "SwarmSignOff: batch=[{}], ts={}, sig={:016x}",
        ids_str, ts, sig
    )
}

fn fnv1a_64(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for &b in bytes {
        hash ^= b as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

// ─── Main Entry Point ──────────────────────────────────────────

/// Run `uws swarm review --batch=<count>`.
///
/// 1. Generates (or fetches) `count` batch items.
/// 2. Runs the NPFM dry-run on each item.
/// 3. Splits into approved / blocked sets.
/// 4. Computes a single sign-off over the approved set.
/// 5. Returns a `SwarmReviewResult`.
pub fn run_swarm_review(batch_size: u32) -> SwarmReviewResult {
    let mut items = mock_batch(batch_size);

    // Apply NPFM dry-run to each item
    for item in &mut items {
        item.npfm_passed = npfm_dry_run(item);
    }

    let (approved, blocked): (Vec<_>, Vec<_>) =
        items.into_iter().partition(|i| i.npfm_passed);

    let approved_ids: Vec<u32> = approved.iter().map(|i| i.id).collect();
    let sign_off = compute_sign_off(&approved_ids);

    SwarmReviewResult {
        approved,
        blocked,
        sign_off,
    }
}

/// Format a `SwarmReviewResult` as a human-readable / JSON-compatible string.
///
/// This is the output printed to stdout when `uws swarm review` is invoked.
pub fn format_review_result(result: &SwarmReviewResult) -> String {
    let mut out = String::new();

    out.push_str("{\n");
    out.push_str(&format!(
        "  \"approved_count\": {},\n",
        result.approved.len()
    ));
    out.push_str(&format!(
        "  \"blocked_count\": {},\n",
        result.blocked.len()
    ));

    out.push_str("  \"approved\": [\n");
    for (i, item) in result.approved.iter().enumerate() {
        out.push_str(&format!(
            "    {{\"id\": {}, \"kind\": \"{}\", \"title\": \"{}\", \"npfm\": \"PASS\"}}",
            item.id,
            item.kind,
            item.title.replace('"', "\\\""),
        ));
        if i < result.approved.len() - 1 {
            out.push(',');
        }
        out.push('\n');
    }
    out.push_str("  ],\n");

    out.push_str("  \"blocked\": [\n");
    for (i, item) in result.blocked.iter().enumerate() {
        out.push_str(&format!(
            "    {{\"id\": {}, \"kind\": \"{}\", \"title\": \"{}\", \"npfm\": \"FAIL\"}}",
            item.id,
            item.kind,
            item.title.replace('"', "\\\""),
        ));
        if i < result.blocked.len() - 1 {
            out.push(',');
        }
        out.push('\n');
    }
    out.push_str("  ],\n");

    out.push_str(&format!("  \"sign_off\": \"{}\"\n", result.sign_off));
    out.push_str("}\n");

    out
}

/// CLI entry point called from `src/main.rs` when `first_arg == "swarm"`.
///
/// Parses the remaining args for `review --batch=<n>` and executes the
/// review, printing the result to stdout.
///
/// Returns `Ok(())` on success, `Err(String)` with a usage message on bad
/// input.
pub fn handle_swarm_command(args: &[String]) -> Result<(), String> {
    // Expect: swarm review [--batch=<n> | --batch <n>]
    let subcommand = args.first().map(|s| s.as_str()).unwrap_or("");
    if subcommand != "review" {
        return Err(format!(
            "Unknown swarm subcommand '{}'. Usage: uws swarm review --batch=<n>",
            subcommand
        ));
    }

    let batch_size = parse_batch_size(&args[1..])?;
    let result = run_swarm_review(batch_size);
    print!("{}", format_review_result(&result));
    Ok(())
}

/// Parse `--batch=<n>` or `--batch <n>` from the remaining args.
fn parse_batch_size(args: &[String]) -> Result<u32, String> {
    let mut i = 0;
    while i < args.len() {
        let a = &args[i];
        if let Some(val) = a.strip_prefix("--batch=") {
            return val
                .parse::<u32>()
                .map_err(|_| format!("--batch value must be a positive integer, got '{}'", val));
        }
        if a == "--batch" {
            let val = args
                .get(i + 1)
                .ok_or_else(|| "--batch requires a value".to_string())?;
            return val
                .parse::<u32>()
                .map_err(|_| format!("--batch value must be a positive integer, got '{}'", val));
        }
        i += 1;
    }
    // Default batch size if --batch not supplied
    Ok(10)
}

// ─── Tests ────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_batch_count() {
        let items = mock_batch(5);
        assert_eq!(items.len(), 5);
    }

    #[test]
    fn test_mock_batch_ids_are_sequential() {
        let items = mock_batch(3);
        for (i, item) in items.iter().enumerate() {
            assert_eq!(item.id, (i + 1) as u32);
        }
    }

    #[test]
    fn test_npfm_blocks_busywork() {
        let busywork = BatchItem {
            id: 1,
            title: "chore: bump version".to_string(),
            kind: BatchItemKind::DependencyUpdate,
            npfm_passed: false,
        };
        assert!(!npfm_dry_run(&busywork));
    }

    #[test]
    fn test_npfm_approves_meaningful_work() {
        let meaningful = BatchItem {
            id: 2,
            title: "fix: correct nil-check in executor".to_string(),
            kind: BatchItemKind::PullRequest,
            npfm_passed: false,
        };
        assert!(npfm_dry_run(&meaningful));
    }

    #[test]
    fn test_run_swarm_review_partitions_correctly() {
        let result = run_swarm_review(10);
        // All items should be classified
        assert_eq!(
            result.approved.len() + result.blocked.len(),
            10
        );
        // Approved items all passed NPFM
        for item in &result.approved {
            assert!(item.npfm_passed);
        }
        // Blocked items all failed NPFM
        for item in &result.blocked {
            assert!(!item.npfm_passed);
        }
    }

    #[test]
    fn test_sign_off_format() {
        let sign_off = compute_sign_off(&[1, 2, 3]);
        assert!(sign_off.starts_with("SwarmSignOff: batch=[1,2,3]"));
        assert!(sign_off.contains("sig="));
    }

    #[test]
    fn test_sign_off_empty_batch() {
        let sign_off = compute_sign_off(&[]);
        assert!(sign_off.starts_with("SwarmSignOff: batch=[]"));
    }

    #[test]
    fn test_parse_batch_size_equals() {
        let args: Vec<String> = vec!["--batch=5".to_string()];
        assert_eq!(parse_batch_size(&args).unwrap(), 5);
    }

    #[test]
    fn test_parse_batch_size_space() {
        let args: Vec<String> = vec!["--batch".to_string(), "7".to_string()];
        assert_eq!(parse_batch_size(&args).unwrap(), 7);
    }

    #[test]
    fn test_parse_batch_size_default() {
        let args: Vec<String> = vec![];
        assert_eq!(parse_batch_size(&args).unwrap(), 10);
    }

    #[test]
    fn test_parse_batch_size_invalid() {
        let args: Vec<String> = vec!["--batch=abc".to_string()];
        assert!(parse_batch_size(&args).is_err());
    }

    #[test]
    fn test_handle_swarm_command_unknown_subcommand() {
        let args: Vec<String> = vec!["unknown".to_string()];
        assert!(handle_swarm_command(&args).is_err());
    }

    #[test]
    fn test_handle_swarm_command_review() {
        let args: Vec<String> = vec!["review".to_string(), "--batch=3".to_string()];
        assert!(handle_swarm_command(&args).is_ok());
    }

    #[test]
    fn test_format_review_result_contains_keys() {
        let result = run_swarm_review(4);
        let output = format_review_result(&result);
        assert!(output.contains("approved_count"));
        assert!(output.contains("blocked_count"));
        assert!(output.contains("sign_off"));
    }
}
