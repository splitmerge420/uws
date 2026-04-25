// crates/kintsugi-weave/src/lib.rs
//
// Kintsugi Weave — Regenerative CI Engine
//
// This crate is the thin orchestrator for the three Kintsugi Weave gates:
//
//   1. NPFM Gate           — Non-Productive Feature Multiplication detector
//                            (stub; wired in PR #7)
//   2. Provenance Validate — HITL check for GoldenTrace trailer in PR body
//                            (real check; full validation wired in PR #12)
//   3. Swarm Commander     — Swarm review summary
//                            (stub; to be wired after swarm review lands)
//
// Gate status in this iteration:
//   NPFM          — STUB    (returns score=0.0 always; see TODO below)
//   Provenance    — REAL    (checks for GoldenTrace-ID trailer in PR body)
//   Swarm         — STUB    (returns PASS always; see TODO below)
//
// None of the gates are enforcing in this iteration. The workflow produces a
// report comment only and does NOT block merge.
//
// TODO(#7):  Wire real NPFM implementation once PR #7 lands.
// TODO:      Wire real Swarm Commander once swarm review integration lands.
// TODO(#12): Wire full GoldenTrace provenance validation once PR #12 lands.

/// Result of the NPFM (Non-Productive Feature Multiplication) gate.
///
/// **STUB** — always returns `score = 0.0` until PR #7 (NPFM implementation)
/// is merged and its API is wired here.
///
/// The NPFM gate detects bloat: PRs that add many features with low constitutional
/// alignment score (measured against the 39 invariants). A score of `0.0` is clean;
/// `1.0` is maximum bloat.
#[derive(Debug, Clone)]
pub struct NpfmResult {
    /// Bloat score: 0.0 (clean) to 1.0 (maximum bloat). Stubbed at 0.0.
    pub score: f64,
    /// Human-readable verdict string.
    pub verdict: String,
    /// True when this gate is a stub rather than a real implementation.
    pub is_stub: bool,
}

/// Result of the HITL provenance validation gate.
///
/// Checks whether the PR body contains a `GoldenTrace-ID:` trailer line.
/// This is a **real** check (not a stub), but enforcement is soft (warn-only)
/// in this iteration.
///
/// TODO(#12): Replace soft-fail with enforcing mode once GoldenTrace (#12) is
/// fully wired and the gate has been operational for ≥1 sprint.
#[derive(Debug, Clone)]
pub struct ProvenanceResult {
    /// Whether a GoldenTrace-ID trailer was found.
    pub has_trailer: bool,
    /// The trailer value, if found.
    pub trailer_value: Option<String>,
    /// Human-readable status message.
    pub status: String,
    /// True when this gate is a stub rather than a real implementation.
    pub is_stub: bool,
}

/// Result of the Swarm Commander summary gate.
///
/// **STUB** — always returns `verdict = PASS` until swarm review integration
/// is implemented.
#[derive(Debug, Clone)]
pub struct SwarmResult {
    /// Swarm verdict string.
    pub verdict: String,
    /// True when this gate is a stub rather than a real implementation.
    pub is_stub: bool,
}

/// Full Kintsugi Weave report, produced by running all three gates.
#[derive(Debug, Clone)]
pub struct WeaveReport {
    /// NPFM gate result.
    pub npfm: NpfmResult,
    /// Provenance validation result.
    pub provenance: ProvenanceResult,
    /// Swarm Commander result.
    pub swarm: SwarmResult,
    /// PR number, if known.
    pub pr_number: Option<u64>,
    /// Commit SHA, if known.
    pub sha: Option<String>,
}

/// Run the NPFM gate.
///
/// **STUB** — returns a placeholder `score = 0.0` / PASS verdict.
/// Wire the real implementation here once PR #7 lands.
///
/// # TODO(#7)
/// Replace this stub with a call to the NPFM API once #7 is merged:
/// ```text
/// npfm::gate::run(pr_diff).await?
/// ```
pub fn npfm_gate() -> NpfmResult {
    // TODO(#7): Call real NPFM implementation once PR #7 lands.
    NpfmResult {
        score: 0.0,
        verdict: "PASS (stub — NPFM not yet wired; see PR #7)".to_string(),
        is_stub: true,
    }
}

/// Run the HITL provenance validation gate.
///
/// Scans `pr_body` for a `GoldenTrace-ID:` trailer line. The check is
/// intentionally **soft** (warn-only) in this iteration — a missing trailer
/// produces a warning but does not block merge.
///
/// # TODO(#12)
/// Switch from soft-fail to enforcing mode once GoldenTrace (#12) is fully
/// wired and the gate has been operational for ≥1 sprint.
///
/// # Example
/// ```
/// use kintsugi_weave::provenance_validate;
///
/// let body = "Fixes #42\n\nGoldenTrace-ID: gt-abc123\n";
/// let result = provenance_validate(body);
/// assert!(result.has_trailer);
/// assert_eq!(result.trailer_value.as_deref(), Some("gt-abc123"));
/// ```
pub fn provenance_validate(pr_body: &str) -> ProvenanceResult {
    const TRAILER_PREFIX: &str = "GoldenTrace-ID:";

    let mut found: Option<String> = None;
    for line in pr_body.lines() {
        let trimmed = line.trim();
        if let Some(value) = trimmed.strip_prefix(TRAILER_PREFIX) {
            let val = value.trim().to_string();
            if !val.is_empty() {
                found = Some(val);
                break;
            }
        }
    }

    match found {
        Some(ref val) => ProvenanceResult {
            has_trailer: true,
            trailer_value: found.clone(),
            status: format!("PASS — GoldenTrace-ID found: {val}"),
            is_stub: false,
        },
        None => ProvenanceResult {
            has_trailer: false,
            trailer_value: None,
            status: "WARN — no GoldenTrace-ID trailer found (soft fail; see PR #12)".to_string(),
            is_stub: false,
        },
    }
}

/// Run the Swarm Commander summary gate.
///
/// **STUB** — returns a placeholder PASS verdict until the Swarm Commander
/// integration is implemented.
///
/// # TODO
/// Wire real Swarm Commander once swarm review integration lands:
/// ```text
/// swarm::commander::summarise(pr_number).await?
/// ```
pub fn swarm_commander_summary() -> SwarmResult {
    // TODO: Call real Swarm Commander once swarm review integration is implemented.
    SwarmResult {
        verdict: "PASS (stub — Swarm Commander not yet wired)".to_string(),
        is_stub: true,
    }
}

/// Run all three gates and produce a full [`WeaveReport`].
///
/// This is the primary entry point for the CI workflow.
///
/// # Arguments
/// * `pr_body`   — full text of the PR description
/// * `pr_number` — PR number, used for display
/// * `sha`       — commit SHA, used for display
pub fn run_all(pr_body: &str, pr_number: Option<u64>, sha: Option<String>) -> WeaveReport {
    WeaveReport {
        npfm: npfm_gate(),
        provenance: provenance_validate(pr_body),
        swarm: swarm_commander_summary(),
        pr_number,
        sha,
    }
}

/// Format a [`WeaveReport`] as a GitHub PR comment (Markdown).
///
/// The comment includes a summary table and per-gate details. It is
/// deliberately labelled "report only — merge not blocked" to make the
/// non-enforcing nature explicit.
///
/// # Example
/// ```
/// use kintsugi_weave::{run_all, format_comment};
///
/// let report = run_all("Fix things\n\nGoldenTrace-ID: gt-test\n", Some(99), None);
/// let comment = format_comment(&report);
/// assert!(comment.contains("Kintsugi Weave Report"));
/// assert!(comment.contains("GoldenTrace-ID found"));
/// ```
pub fn format_comment(report: &WeaveReport) -> String {
    let npfm_icon = if report.npfm.verdict.starts_with("PASS") {
        "✅"
    } else {
        "❌"
    };
    let prov_icon = if report.provenance.has_trailer {
        "✅"
    } else {
        "⚠️"
    };
    let swarm_icon = if report.swarm.verdict.starts_with("PASS") {
        "✅"
    } else {
        "❌"
    };

    let npfm_stub = if report.npfm.is_stub { " *(stub)*" } else { "" };
    let swarm_stub = if report.swarm.is_stub {
        " *(stub)*"
    } else {
        ""
    };

    let pr_label = match report.pr_number {
        Some(n) => format!("PR #{n}"),
        None => "unknown PR".to_string(),
    };
    let sha_label = match &report.sha {
        Some(s) => format!(" @ `{}`", &s[..s.len().min(8)]),
        None => String::new(),
    };

    format!(
        r#"## 🧵 Kintsugi Weave Report — {pr_label}{sha_label}

> **Constitutional health check — report only, merge not blocked.**

| Gate | Status | Notes |
|------|--------|-------|
| NPFM Gate{npfm_stub} | {npfm_icon} score `{npfm_score:.2}` | {npfm_verdict} |
| Provenance (GoldenTrace) | {prov_icon} | {prov_status} |
| Swarm Commander{swarm_stub} | {swarm_icon} | {swarm_verdict} |

### Gate Details

**NPFM Gate**{npfm_stub}
- Score: `{npfm_score:.2}` (0.0 = clean, 1.0 = maximum bloat)
- TODO(#7): Wire real NPFM implementation once PR #7 lands.

**Provenance Validation**
- {prov_status}
- TODO(#12): Switch to enforcing mode once GoldenTrace (#12) is fully wired.

**Swarm Commander**{swarm_stub}
- {swarm_verdict}
- TODO: Wire real Swarm Commander once swarm review integration lands.

---
*Generated by `kintsugi-weave` — Kintsugi Weave CI engine (non-enforcing).*
"#,
        pr_label = pr_label,
        sha_label = sha_label,
        npfm_icon = npfm_icon,
        npfm_score = report.npfm.score,
        npfm_verdict = report.npfm.verdict,
        npfm_stub = npfm_stub,
        prov_icon = prov_icon,
        prov_status = report.provenance.status,
        swarm_icon = swarm_icon,
        swarm_verdict = report.swarm.verdict,
        swarm_stub = swarm_stub,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_npfm_gate_stub() {
        let result = npfm_gate();
        assert_eq!(result.score, 0.0);
        assert!(result.verdict.contains("stub"));
        assert!(result.is_stub);
    }

    #[test]
    fn test_provenance_validate_found() {
        let body = "Fixes #42\n\nGoldenTrace-ID: gt-abc123\n";
        let result = provenance_validate(body);
        assert!(result.has_trailer);
        assert_eq!(result.trailer_value.as_deref(), Some("gt-abc123"));
        assert!(result.status.contains("PASS"));
        assert!(!result.is_stub);
    }

    #[test]
    fn test_provenance_validate_missing() {
        let body = "Fixes #42\n\nNo trailer here.\n";
        let result = provenance_validate(body);
        assert!(!result.has_trailer);
        assert!(result.trailer_value.is_none());
        assert!(result.status.contains("WARN"));
        assert!(!result.is_stub);
    }

    #[test]
    fn test_provenance_validate_empty_value_not_matched() {
        // A line with just the prefix but no value is not matched
        let body = "GoldenTrace-ID:\n";
        let result = provenance_validate(body);
        assert!(!result.has_trailer);
    }

    #[test]
    fn test_provenance_validate_with_leading_whitespace() {
        let body = "  GoldenTrace-ID: gt-padded  \n";
        let result = provenance_validate(body);
        assert!(result.has_trailer);
        assert_eq!(result.trailer_value.as_deref(), Some("gt-padded"));
    }

    #[test]
    fn test_swarm_commander_stub() {
        let result = swarm_commander_summary();
        assert!(result.verdict.contains("stub"));
        assert!(result.is_stub);
    }

    #[test]
    fn test_run_all_produces_report() {
        let body = "Fixes #1\n\nGoldenTrace-ID: gt-run-test\n";
        let report = run_all(body, Some(42), Some("abc1234567890".to_string()));
        assert_eq!(report.pr_number, Some(42));
        assert!(report.provenance.has_trailer);
        assert!(report.npfm.is_stub);
        assert!(report.swarm.is_stub);
    }

    #[test]
    fn test_format_comment_contains_expected_sections() {
        let report = run_all(
            "Fix things\n\nGoldenTrace-ID: gt-test\n",
            Some(99),
            Some("deadbeef0000".to_string()),
        );
        let comment = format_comment(&report);
        assert!(comment.contains("Kintsugi Weave Report"));
        assert!(comment.contains("PR #99"));
        assert!(comment.contains("GoldenTrace-ID found"));
        assert!(comment.contains("NPFM Gate"));
        assert!(comment.contains("Swarm Commander"));
        assert!(comment.contains("merge not blocked"));
    }

    #[test]
    fn test_format_comment_warn_when_no_trailer() {
        let report = run_all("No trailer here", None, None);
        let comment = format_comment(&report);
        assert!(comment.contains("⚠️"));
        assert!(comment.contains("WARN"));
    }
}
