//! Regenerative IP Provenance Engine — core types and logic.
//!
//! # Overview
//! Every commit to a `uws`-managed workspace carries a `ProvenanceTrailer`
//! that records:
//! - how much of the creative work was done by a human (`human_weight`)
//! - how much was AI-assisted (`ai_weight`)
//! - how revenue derived from this work should be split (`RevenueSplit`)
//! - a tamper-evident signature over the canonical JSON of the trailer
//!   (`HitlSignature`)
//!
//! # Minimum human weight
//! `MIN_HUMAN_WEIGHT` (0.01) is enforced: a fully-autonomous AI commit is
//! not permitted.  This guards the regenerative-IP guarantee that a human
//! remains "in the loop" for every piece of attributed intellectual property.
//!
//! # Signature algorithm (current)
//! SHA-256 is computed over the *canonical JSON* of `ProvenanceTrailerCore`
//! (the fields that constitute the attestation, **excluding** the signature
//! itself).  The hex digest is stored in `HitlSignature::digest`.
//!
//! # Upgrade path
//! Replace `algorithm: "sha256-canonical-json"` with `"ed25519"` (RFC 8032)
//! or `"ml-dsa-44"` (FIPS 204 / ML-DSA level 2) and swap the hash call for
//! an asymmetric signing operation using the `ed25519-dalek` or `pqcrypto`
//! crates.  The `HitlSignature` struct is already designed for that: add a
//! `pubkey` field and sign the canonical JSON bytes instead of hashing them.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Minimum human contribution weight.  Enforced in [`ProvenanceTrailer::new`].
pub const MIN_HUMAN_WEIGHT: f64 = 0.01;

// ─── Revenue Split ─────────────────────────────────────────────────────────

/// Describes how revenue derived from IP attributed to this commit should be
/// distributed among contributors.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RevenueSplit {
    /// Fraction (0.0–1.0) allocated to human contributor(s).
    pub human_share: f64,
    /// Fraction (0.0–1.0) allocated to the AI/tool provider.
    pub ai_share: f64,
    /// Fraction (0.0–1.0) held in the regenerative commons pool.
    pub commons_share: f64,
}

impl RevenueSplit {
    /// Create a split from human and AI shares.
    ///
    /// The remainder (1.0 − human − ai) becomes the commons share.
    ///
    /// Returns `Err` if any share is negative or the total exceeds 1.0.
    pub fn new(human_share: f64, ai_share: f64) -> Result<Self, String> {
        if human_share < 0.0 || ai_share < 0.0 {
            return Err("Revenue shares must be non-negative".to_string());
        }
        let commons_share = 1.0 - human_share - ai_share;
        if commons_share < -1e-9 {
            return Err(format!(
                "human_share ({human_share}) + ai_share ({ai_share}) exceeds 1.0"
            ));
        }
        Ok(Self {
            human_share,
            ai_share,
            commons_share: commons_share.max(0.0),
        })
    }
}

// ─── HITL Signature ────────────────────────────────────────────────────────

/// A tamper-evident signature over the canonical JSON of a provenance trailer.
///
/// Current algorithm: SHA-256 over the canonical JSON bytes of
/// [`ProvenanceTrailerCore`].  See the module-level doc for the Ed25519 /
/// ML-DSA upgrade path.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HitlSignature {
    /// Identifies the signing algorithm.
    ///
    /// Current value: `"sha256-canonical-json"`.
    /// Future values: `"ed25519"`, `"ml-dsa-44"`.
    pub algorithm: String,
    /// Hex-encoded signature (or hash digest for the current SHA-256 mode).
    pub digest: String,
}

// ─── Canonical fields (signed) ─────────────────────────────────────────────

/// The subset of trailer fields that are covered by the signature.
///
/// Serialised with `serde_json` in key-sorted order to produce a stable
/// canonical JSON string.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProvenanceTrailerCore {
    /// Fraction of creative work attributable to a human (0.01–1.0).
    pub human_weight: f64,
    /// Fraction of creative work attributable to AI (0.0–0.99).
    pub ai_weight: f64,
    /// Revenue distribution spec.
    pub revenue_split: RevenueSplit,
    /// Unix timestamp (seconds) of the signing event.
    pub signed_at: u64,
    /// Git commit SHA that this trailer is attached to, if known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit_sha: Option<String>,
    /// Free-form author identifier (e.g. email, GitHub handle).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
}

// ─── Full Trailer ──────────────────────────────────────────────────────────

/// A signed provenance record for a single commit.
///
/// Attach to a git commit with:
/// ```text
/// uws ip sign
/// ```
/// or read the canonical JSON with:
/// ```text
/// uws ip sign --dry-run
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProvenanceTrailer {
    /// The fields that are covered by the signature.
    #[serde(flatten)]
    pub core: ProvenanceTrailerCore,
    /// Tamper-evident signature over the canonical JSON of `core`.
    pub signature: HitlSignature,
}

impl ProvenanceTrailer {
    /// Build and sign a new trailer.
    ///
    /// # Errors
    /// Returns `Err` if `human_weight < MIN_HUMAN_WEIGHT` or if
    /// `human_weight + ai_weight > 1.0`.
    pub fn new(
        human_weight: f64,
        ai_weight: f64,
        revenue_split: RevenueSplit,
        signed_at: u64,
        commit_sha: Option<String>,
        author: Option<String>,
    ) -> Result<Self, String> {
        if human_weight < MIN_HUMAN_WEIGHT {
            return Err(format!(
                "human_weight ({human_weight}) is below MIN_HUMAN_WEIGHT ({MIN_HUMAN_WEIGHT})"
            ));
        }
        if human_weight + ai_weight > 1.0 + 1e-9 {
            return Err(format!(
                "human_weight ({human_weight}) + ai_weight ({ai_weight}) exceeds 1.0"
            ));
        }

        let core = ProvenanceTrailerCore {
            human_weight,
            ai_weight,
            revenue_split,
            signed_at,
            commit_sha,
            author,
        };

        let signature = Self::sign(&core)?;
        Ok(Self { core, signature })
    }

    /// Compute the SHA-256 signature over the canonical JSON of `core`.
    fn sign(core: &ProvenanceTrailerCore) -> Result<HitlSignature, String> {
        let canonical =
            canonical_json(core).map_err(|e| format!("failed to serialise core: {e}"))?;
        let mut hasher = Sha256::new();
        hasher.update(canonical.as_bytes());
        let digest = format!("{:x}", hasher.finalize());
        Ok(HitlSignature {
            algorithm: "sha256-canonical-json".to_string(),
            digest,
        })
    }

    /// Verify the embedded signature against the core fields.
    ///
    /// Returns `Ok(true)` if the signature is valid, `Ok(false)` if it has
    /// been tampered with, or `Err` if the core cannot be serialised.
    pub fn verify(&self) -> Result<bool, String> {
        let expected = Self::sign(&self.core)?;
        Ok(expected.digest == self.signature.digest)
    }

    /// Serialise the full trailer to a pretty-printed JSON string.
    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|e| format!("JSON serialisation error: {e}"))
    }

    /// Return the trailer as a compact single-line JSON string suitable for
    /// embedding in a git note (no embedded newlines).
    ///
    /// All special characters are already escaped by `serde_json`.
    pub fn to_git_note(&self) -> Result<String, String> {
        serde_json::to_string(self).map_err(|e| format!("JSON serialisation error: {e}"))
    }

    /// Build the registration payload for the Ledger API (`uws ip monetize`).
    pub fn to_monetize_payload(&self) -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({
            "schema_version": "1",
            "provenance": self,
            "registration_type": "ip_commit",
        }))
    }
}

// ─── Canonical JSON helper ─────────────────────────────────────────────────

/// Produce a *canonical* JSON string for `value` by sorting object keys.
///
/// This ensures the signature is stable regardless of field insertion order.
fn canonical_json<T: Serialize>(value: &T) -> Result<String, serde_json::Error> {
    let v = serde_json::to_value(value)?;
    Ok(sort_json_keys(&v).to_string())
}

/// Recursively sort JSON object keys so the representation is deterministic.
fn sort_json_keys(value: &serde_json::Value) -> serde_json::Value {
    use serde_json::Value;
    match value {
        Value::Object(map) => {
            let mut sorted: serde_json::Map<String, Value> = serde_json::Map::new();
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();
            for k in keys {
                sorted.insert(k.clone(), sort_json_keys(&map[k]));
            }
            Value::Object(sorted)
        }
        Value::Array(arr) => Value::Array(arr.iter().map(sort_json_keys).collect()),
        other => other.clone(),
    }
}

// ─── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_split() -> RevenueSplit {
        RevenueSplit::new(0.7, 0.2).expect("valid split")
    }

    // ── MIN_HUMAN_WEIGHT enforcement ─────────────────────────────────────

    #[test]
    fn rejects_zero_human_weight() {
        let err = ProvenanceTrailer::new(0.0, 0.8, make_split(), 0, None, None)
            .expect_err("should fail");
        assert!(
            err.contains("MIN_HUMAN_WEIGHT"),
            "error should mention MIN_HUMAN_WEIGHT, got: {err}"
        );
    }

    #[test]
    fn rejects_human_weight_below_minimum() {
        let err =
            ProvenanceTrailer::new(0.005, 0.8, make_split(), 0, None, None).expect_err("should fail");
        assert!(err.contains("MIN_HUMAN_WEIGHT"));
    }

    #[test]
    fn accepts_minimum_human_weight() {
        let t = ProvenanceTrailer::new(MIN_HUMAN_WEIGHT, 0.5, make_split(), 42, None, None)
            .expect("should succeed at boundary");
        assert!((t.core.human_weight - MIN_HUMAN_WEIGHT).abs() < f64::EPSILON);
    }

    // ── Weight sum constraint ─────────────────────────────────────────────

    #[test]
    fn rejects_weights_exceeding_one() {
        let err = ProvenanceTrailer::new(0.6, 0.5, make_split(), 0, None, None)
            .expect_err("should fail");
        assert!(err.contains("exceeds 1.0"), "got: {err}");
    }

    // ── Serialisation round-trip ──────────────────────────────────────────

    #[test]
    fn serialisation_round_trip() {
        let original = ProvenanceTrailer::new(
            0.6,
            0.3,
            make_split(),
            1_700_000_000,
            Some("abc123".to_string()),
            Some("alice@example.com".to_string()),
        )
        .expect("valid trailer");

        let json = original.to_json().expect("serialises");
        let recovered: ProvenanceTrailer =
            serde_json::from_str(&json).expect("deserialises");

        assert_eq!(original.core.human_weight, recovered.core.human_weight);
        assert_eq!(original.core.ai_weight, recovered.core.ai_weight);
        assert_eq!(original.signature.digest, recovered.signature.digest);
        assert_eq!(original.signature.algorithm, "sha256-canonical-json");
    }

    // ── Signature verification ────────────────────────────────────────────

    #[test]
    fn valid_signature_verifies() {
        let t = ProvenanceTrailer::new(0.5, 0.4, make_split(), 99, None, None)
            .expect("valid");
        assert!(t.verify().expect("verify should not error"));
    }

    #[test]
    fn tampered_core_fails_verification() {
        let mut t = ProvenanceTrailer::new(0.5, 0.4, make_split(), 99, None, None)
            .expect("valid");
        // Tamper with a core field after signing
        t.core.human_weight = 0.99;
        assert!(!t.verify().expect("verify should not error"));
    }

    // ── JSON escaping (git notes safety) ─────────────────────────────────

    #[test]
    fn git_note_contains_no_raw_newlines() {
        let t = ProvenanceTrailer::new(
            0.8,
            0.1,
            make_split(),
            0,
            Some("sha\ninjection".to_string()),
            None,
        )
        .expect("valid");
        let note = t.to_git_note().expect("serialises");
        // serde_json escapes \n as \\n; the resulting string must be a single line
        assert!(!note.contains('\n'), "git note must be single-line: {note}");
    }

    #[test]
    fn git_note_escapes_special_characters() {
        let t = ProvenanceTrailer::new(
            0.5,
            0.3,
            make_split(),
            0,
            None,
            Some(r#"alice "the hacker" <alice@example.com>"#.to_string()),
        )
        .expect("valid");
        let note = t.to_git_note().expect("serialises");
        // Quotes inside string values must be backslash-escaped by JSON
        assert!(note.contains(r#"\""#), "JSON must escape quotes: {note}");
    }

    // ── Revenue split ─────────────────────────────────────────────────────

    #[test]
    fn revenue_split_commons_is_remainder() {
        let s = RevenueSplit::new(0.6, 0.3).expect("valid");
        let total = s.human_share + s.ai_share + s.commons_share;
        assert!((total - 1.0).abs() < 1e-9, "shares must sum to 1.0, got {total}");
    }

    #[test]
    fn revenue_split_rejects_negative_shares() {
        assert!(RevenueSplit::new(-0.1, 0.5).is_err());
        assert!(RevenueSplit::new(0.5, -0.1).is_err());
    }

    #[test]
    fn revenue_split_rejects_overflow() {
        assert!(RevenueSplit::new(0.7, 0.4).is_err());
    }

    // ── Canonical JSON stability ──────────────────────────────────────────

    #[test]
    fn canonical_json_is_stable_across_calls() {
        let split = make_split();
        let core = ProvenanceTrailerCore {
            human_weight: 0.7,
            ai_weight: 0.2,
            revenue_split: split,
            signed_at: 42,
            commit_sha: None,
            author: None,
        };
        let a = canonical_json(&core).expect("ok");
        let b = canonical_json(&core).expect("ok");
        assert_eq!(a, b);
    }

    #[test]
    fn canonical_json_keys_are_sorted() {
        let split = make_split();
        let core = ProvenanceTrailerCore {
            human_weight: 0.3,
            ai_weight: 0.5,
            revenue_split: split,
            signed_at: 0,
            commit_sha: Some("abc".to_string()),
            author: Some("bob".to_string()),
        };
        let json_str = canonical_json(&core).expect("ok");
        // Parse back and check key ordering: earlier keys should appear before later ones
        // in sorted order. We simply verify that "ai_weight" appears before "human_weight".
        let ai_pos = json_str.find("\"ai_weight\"").expect("has ai_weight");
        let human_pos = json_str.find("\"human_weight\"").expect("has human_weight");
        assert!(ai_pos < human_pos, "keys should be sorted: ai < human");
    }
}
