// src/kintsugi_weave.rs
//
// Kintsugi Weave — Regenerative CI Engine (root lib re-export shim)
//
// This module re-exports the gate types and functions from the standalone
// `crates/kintsugi-weave` crate for library consumers that import the root
// `uws` crate directly.
//
// The canonical implementation lives in `crates/kintsugi-weave/src/lib.rs`.
// The standalone CLI binary lives in `crates/kintsugi-weave/src/main.rs`.
//
// Gate status:
//   NPFM Gate       — STUB (wire in PR #7)
//   Provenance      — REAL check, soft-fail (enforcing mode in PR #12)
//   Swarm Commander — STUB (wire after swarm review integration)
//
// None of the gates are enforcing in this iteration. The workflow produces a
// report comment only and does NOT block merge.

/// Result of the NPFM (Non-Productive Feature Multiplication) gate.
///
/// **STUB** — always returns `score = 0.0` until PR #7 is merged.
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
/// TODO(#12): Switch from soft-fail to enforcing mode once GoldenTrace (#12)
/// is fully wired.
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
/// **STUB** — always returns PASS until swarm review integration is implemented.
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
/// **STUB** — returns score `0.0` / PASS verdict. Wire real implementation
/// here once PR #7 lands.
///
/// # TODO(#7)
/// Replace this stub with a call to the NPFM API:
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
/// Scans `pr_body` for a `GoldenTrace-ID:` trailer line. A missing trailer
/// is a soft-fail (warn-only) in this iteration — it does not block merge.
///
/// # TODO(#12)
/// Switch from soft-fail to enforcing mode once GoldenTrace (#12) is fully
/// wired and the gate has been operational for ≥1 sprint.
///
/// # Example
/// ```
/// use uws::kintsugi_weave::provenance_validate;
///
/// let body = "Fixes #42\n\nGoldenTrace-ID: gt-abc123\n";
/// let result = provenance_validate(body);
/// assert!(result.has_trailer);
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
/// **STUB** — returns PASS always until swarm review integration is implemented.
pub fn swarm_commander_summary() -> SwarmResult {
    // TODO: Call real Swarm Commander once swarm review integration is implemented.
    SwarmResult {
        verdict: "PASS (stub — Swarm Commander not yet wired)".to_string(),
        is_stub: true,
    }
}

/// Run all three gates and produce a full [`WeaveReport`].
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
        let body = "No trailer here.\n";
        let result = provenance_validate(body);
        assert!(!result.has_trailer);
        assert!(result.status.contains("WARN"));
    }

    #[test]
    fn test_provenance_validate_empty_value() {
        let body = "GoldenTrace-ID:\n";
        let result = provenance_validate(body);
        assert!(!result.has_trailer);
    }

    #[test]
    fn test_swarm_commander_stub() {
        let result = swarm_commander_summary();
        assert!(result.verdict.contains("stub"));
        assert!(result.is_stub);
    }

    #[test]
    fn test_run_all_produces_report() {
        let report = run_all(
            "Fix\n\nGoldenTrace-ID: gt-test\n",
            Some(42),
            Some("abc123".to_string()),
        );
        assert!(report.provenance.has_trailer);
        assert!(report.npfm.is_stub);
        assert!(report.swarm.is_stub);
    }

    #[test]
    fn test_format_comment_with_trailer() {
        let report = run_all("Fix\n\nGoldenTrace-ID: gt-x\n", Some(1), None);
        let comment = format_comment(&report);
        assert!(comment.contains("Kintsugi Weave Report"));
        assert!(comment.contains("PR #1"));
        assert!(comment.contains("GoldenTrace-ID found"));
        assert!(comment.contains("merge not blocked"));
    }

    #[test]
    fn test_format_comment_warns_without_trailer() {
        let report = run_all("No trailer", None, None);
        let comment = format_comment(&report);
        assert!(comment.contains("⚠️") || comment.contains("WARN"));
    }
}
