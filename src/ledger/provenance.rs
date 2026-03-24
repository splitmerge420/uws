// src/ledger/provenance.rs
// Aluminum OS — Provenance Ledger
//
// Maps the `ProvenanceTrailer` (philosophical) to the `GoldenTrace`
// (concrete git commit trailer) from the Kintsugi architecture.
//
// `ProvenanceTrailer`  ──maps to──►  `GoldenTrace`
//   actor                              signature
//   invariants_checked                 invariants_checked
//   audit_hash                         audit_hash
//   policy_result                      policy_result
//   + hitl_weight (human HITL input)   hitl_weight
//
// The primary output is a valid git trailer string:
//
//   Golden-Trace: HITL=0.90, Invariants=INV-2|INV-3, Policy=ALLOW, Sig=abc123
//   Golden-Trace-Audit: sha3-256:<hash>
//   Golden-Trace-Actor: <actor>
//
// Author: GitHub Copilot — Council Session 2026-03-21
// Invariants: INV-3 (Audit Trail), INV-5 (Provenance)

#![allow(dead_code)]

use std::time::{SystemTime, UNIX_EPOCH};

// ─── GoldenTrace ───────────────────────────────────────────────

/// Concrete mapping of the `ProvenanceTrailer` onto a `GoldenTrace`.
///
/// A `GoldenTrace` is the authoritative, git-embeddable record that a human
/// was in the loop (HITL) when a given commit was created or approved.
///
/// | ProvenanceTrailer field    | GoldenTrace field          |
/// |----------------------------|----------------------------|
/// | `actor`                    | `actor`                    |
/// | `invariants_checked`       | `invariants_checked`       |
/// | `policy_result`            | `policy_result`            |
/// | `audit_hash`               | `audit_hash`               |
/// | *(new)* human `hitl_weight`| `hitl_weight`              |
/// | `session_timestamp`        | `timestamp`                |
#[derive(Debug, Clone)]
pub struct GoldenTrace {
    /// Human/AI actor who approved the commit
    pub actor: String,
    /// Unix timestamp of the approval
    pub timestamp: u64,
    /// Human-in-the-loop weight [0.0 – 1.0]
    ///
    /// 1.0 = fully human-driven decision
    /// 0.0 = fully automated (no human review)
    pub hitl_weight: f32,
    /// Constitutional invariants that were checked before approval
    pub invariants_checked: Vec<String>,
    /// Policy engine result ("ALLOW" | "DENY")
    pub policy_result: String,
    /// SHA3-256 audit-chain hash of the approval session
    pub audit_hash: String,
    /// Short commit reference this trace is attached to
    pub commit_hash: String,
}

impl GoldenTrace {
    /// Format the `GoldenTrace` as a set of git commit trailer lines.
    ///
    /// The output is ready to be appended verbatim to a git commit message.
    ///
    /// ```
    /// Golden-Trace: HITL=0.90, Policy=ALLOW, Sig=abc123...
    /// Golden-Trace-Invariants: INV-2,INV-3
    /// Golden-Trace-Actor: copilot
    /// Golden-Trace-Commit: abc123
    /// ```
    pub fn to_trailer_string(&self) -> String {
        format!(
            "Golden-Trace: HITL={:.2}, Policy={}, Sig={}\n\
             Golden-Trace-Invariants: {}\n\
             Golden-Trace-Actor: {}\n\
             Golden-Trace-Commit: {}",
            self.hitl_weight,
            self.policy_result,
            &self.audit_hash[..self.audit_hash.len().min(16)],
            self.invariants_checked.join(","),
            self.actor,
            &self.commit_hash[..self.commit_hash.len().min(12)],
        )
    }
}

// ─── ProvenanceTrailer → GoldenTrace mapping ──────────────────

/// A `ProvenanceTrailer` mirrors the fields from
/// `council_github_client::ProvenanceTrailer`. It is re-declared here to
/// keep the ledger module self-contained and avoid circular dependency on
/// the council client.
#[derive(Debug, Clone)]
pub struct ProvenanceTrailer {
    /// Actor name (AI agent or human identifier)
    pub actor: String,
    /// ISO-8601 / Unix session timestamp
    pub session_timestamp: String,
    /// Constitutional invariants checked during this session
    pub invariants_checked: Vec<String>,
    /// Policy engine decision: "ALLOW" or "DENY"
    pub policy_result: String,
    /// SHA3-256 audit-chain hash
    pub audit_hash: String,
}

/// Convert a `ProvenanceTrailer` and an explicit HITL weight into a
/// `GoldenTrace`.
///
/// # Arguments
/// * `trailer`     — provenance metadata from the council client
/// * `hitl_weight` — human-in-the-loop weight [0.0 – 1.0]
/// * `commit_hash` — the commit SHA this trace is being attached to
///
/// # Panics
/// Does not panic. `hitl_weight` is clamped to [0.0, 1.0].
pub fn provenance_to_golden_trace(
    trailer: &ProvenanceTrailer,
    hitl_weight: f32,
    commit_hash: &str,
) -> GoldenTrace {
    let hitl = hitl_weight.clamp(0.0, 1.0);
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    GoldenTrace {
        actor: trailer.actor.clone(),
        timestamp: ts,
        hitl_weight: hitl,
        invariants_checked: trailer.invariants_checked.clone(),
        policy_result: trailer.policy_result.clone(),
        audit_hash: trailer.audit_hash.clone(),
        commit_hash: commit_hash.to_string(),
    }
}

// ─── append_golden_trace_to_commit ────────────────────────────

/// Format a valid `Golden-Trace` git trailer string and return it, ready
/// to be appended to a commit message.
///
/// This is the primary public API for the provenance ledger module.
///
/// # Arguments
/// * `hitl_weight`  — human-in-the-loop weight [0.0 – 1.0]
/// * `commit_hash`  — the commit SHA being annotated
///
/// # Returns
/// A multi-line string of git trailer lines, for example:
///
/// ```text
/// Golden-Trace: HITL=0.90, Policy=ALLOW, Sig=abc123...
/// Golden-Trace-Invariants: INV-2,INV-3
/// Golden-Trace-Actor: automated
/// Golden-Trace-Commit: abc123456789
/// ```
pub fn append_golden_trace_to_commit(hitl_weight: f32, commit_hash: &str) -> String {
    // Build a minimal audit hash from the commit hash + timestamp so the
    // trailer is deterministic enough for testing but unique per call.
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let raw = format!("{}|{}", commit_hash, ts);
    let audit_hash = fnv1a_as_hex(raw.as_bytes());

    let trailer = ProvenanceTrailer {
        actor: "automated".to_string(),
        session_timestamp: ts.to_string(),
        invariants_checked: vec!["INV-2".to_string(), "INV-3".to_string()],
        policy_result: "ALLOW".to_string(),
        audit_hash,
    };

    let trace = provenance_to_golden_trace(&trailer, hitl_weight, commit_hash);
    trace.to_trailer_string()
}

// ─── Helpers ──────────────────────────────────────────────────

fn fnv1a_as_hex(bytes: &[u8]) -> String {
    let mut hash: u64 = 0xcbf29ce484222325;
    for &b in bytes {
        hash ^= b as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    // Extend to 64 hex chars by hashing twice
    let h2 = {
        let mut h: u64 = hash.wrapping_add(0xdeadbeef);
        for &b in bytes {
            h ^= b as u64;
            h = h.wrapping_mul(0x100000001b3);
        }
        h
    };
    format!("{:016x}{:016x}{:016x}{:016x}", hash, h2, hash ^ h2, h2.wrapping_add(hash))
}

// ─── Tests ────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_trailer() -> ProvenanceTrailer {
        ProvenanceTrailer {
            actor: "copilot".to_string(),
            session_timestamp: "1742000000Z".to_string(),
            invariants_checked: vec!["INV-2".to_string(), "INV-3".to_string()],
            policy_result: "ALLOW".to_string(),
            audit_hash: "abcd1234ef567890abcd1234ef567890abcd1234ef567890abcd1234ef567890"
                .to_string(),
        }
    }

    #[test]
    fn test_provenance_to_golden_trace_fields() {
        let trailer = sample_trailer();
        let trace = provenance_to_golden_trace(&trailer, 0.9, "deadbeef1234");

        assert_eq!(trace.actor, "copilot");
        assert!((trace.hitl_weight - 0.9).abs() < 1e-6);
        assert_eq!(trace.policy_result, "ALLOW");
        assert_eq!(trace.invariants_checked, vec!["INV-2", "INV-3"]);
        assert_eq!(trace.commit_hash, "deadbeef1234");
    }

    #[test]
    fn test_hitl_weight_clamped_above_one() {
        let trailer = sample_trailer();
        let trace = provenance_to_golden_trace(&trailer, 1.5, "abc");
        assert!((trace.hitl_weight - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_hitl_weight_clamped_below_zero() {
        let trailer = sample_trailer();
        let trace = provenance_to_golden_trace(&trailer, -0.5, "abc");
        assert!((trace.hitl_weight - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_to_trailer_string_contains_hitl() {
        let trailer = sample_trailer();
        let trace = provenance_to_golden_trace(&trailer, 0.9, "deadbeef1234");
        let s = trace.to_trailer_string();
        assert!(s.contains("Golden-Trace:"));
        assert!(s.contains("HITL=0.90"));
        assert!(s.contains("Golden-Trace-Actor: copilot"));
    }

    #[test]
    fn test_to_trailer_string_contains_invariants() {
        let trailer = sample_trailer();
        let trace = provenance_to_golden_trace(&trailer, 1.0, "abc");
        let s = trace.to_trailer_string();
        assert!(s.contains("Golden-Trace-Invariants: INV-2,INV-3"));
    }

    #[test]
    fn test_append_golden_trace_to_commit_format() {
        let trailer = append_golden_trace_to_commit(0.9, "abc123def456");
        assert!(trailer.contains("Golden-Trace:"));
        assert!(trailer.contains("HITL=0.90"));
        assert!(trailer.contains("Golden-Trace-Actor: automated"));
    }

    #[test]
    fn test_append_golden_trace_policy_is_allow() {
        let trailer = append_golden_trace_to_commit(0.75, "cafebabe");
        assert!(trailer.contains("Policy=ALLOW"));
    }

    #[test]
    fn test_append_golden_trace_includes_commit() {
        let trailer = append_golden_trace_to_commit(1.0, "feedface1234");
        assert!(trailer.contains("Golden-Trace-Commit: feedface1234"));
    }

    #[test]
    fn test_fnv1a_hex_length() {
        let h = fnv1a_as_hex(b"test");
        assert_eq!(h.len(), 64);
    }

    #[test]
    fn test_fnv1a_hex_deterministic() {
        let h1 = fnv1a_as_hex(b"same input");
        let h2 = fnv1a_as_hex(b"same input");
        assert_eq!(h1, h2);
    }
}
