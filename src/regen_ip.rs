//! Regenerative IP & Provenance Engine
//!
//! Thin orchestrator for tagging artifacts with cryptographic provenance and
//! routing attribution to maintainers/contributors.
//!
//! Deliverables in this module:
//! - `sign_artifact()` — SHA-256 hash of a file or commit ref, produces a
//!   signed (or unsigned-but-hashed) provenance record.
//! - `monetize_artifact()` — structured "monetization intent" record with
//!   attribution targets and a suggested payout split.
//!
//! ## Signing key
//! Ed25519 signing is deferred to a follow-up (see Followups in PR body).
//! For v0, if no key is configured the record is emitted with `unsigned: true`.
//!
//! ## Attribution targets
//! royalty_weight::compute_attribution() is not yet wired (that is PR #18).
//! v0 stubs attribution from Cargo.toml `[package].authors` with a TODO.

use std::io;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Semantic version of the invariant set this provenance record was produced
/// under.  Bumped whenever the signing / hashing algorithm changes.
pub const INVARIANT_SET_VERSION: &str = "v0.1.0";

/// A signed (or unsigned-but-hashed) provenance record for a single artifact.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProvenanceRecord {
    /// Hex-encoded SHA-256 digest of the artifact content.
    pub artifact_hash: String,
    /// Identity of the signer (empty string when unsigned).
    pub signer: String,
    /// Unix timestamp (seconds since epoch) when the record was produced.
    pub timestamp: u64,
    /// Hex-encoded signature bytes, or empty string when unsigned.
    pub signature: String,
    /// Version of the invariant set used to produce this record.
    pub invariant_set_version: String,
    /// `true` when no signing key was configured — hash is real but unverified.
    pub unsigned: bool,
}

/// One attribution target inside a monetization intent.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AttributionTarget {
    /// Human-readable identifier (e.g. "Dave Sheldon <splitmerge420@gmail.com>").
    pub identity: String,
    /// Fractional share of the suggested payout (0.0–1.0).
    pub weight: f64,
}

/// Structured monetization intent for a given artifact.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonetizationRecord {
    /// SHA-256 hex digest of the artifact being monetized.
    pub artifact_hash: String,
    /// Ordered list of attribution targets.
    pub attribution_targets: Vec<AttributionTarget>,
    /// Key → fractional-share map mirroring attribution_targets for quick lookup.
    pub suggested_split: std::collections::HashMap<String, f64>,
    /// Human-readable status (e.g. "stub", "pending", "resolved").
    pub status: String,
}

// ─── Internal helpers ────────────────────────────────────────────────────────

/// Returns the current Unix timestamp in seconds.
fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Compute a SHA-256 hex digest of arbitrary bytes.
pub fn sha256_hex(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher
        .finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

// ─── Public API ──────────────────────────────────────────────────────────────

/// Compute a provenance record for the file at `path`.
///
/// Reads the file, hashes its contents with SHA-256, and returns an
/// unsigned provenance record (`unsigned: true`).  When a signing key is
/// available (follow-up work) the `unsigned` field will be `false` and
/// `signature` will carry the Ed25519 hex bytes.
///
/// # Errors
/// Returns an `io::Error` if the file cannot be read.
pub fn sign_artifact(path: &str) -> Result<ProvenanceRecord, io::Error> {
    let data = std::fs::read(path)?;
    let artifact_hash = sha256_hex(&data);

    // TODO: wire to signing-key storage layer (follow-up: real Ed25519 signing)
    Ok(ProvenanceRecord {
        artifact_hash,
        signer: String::new(),
        timestamp: now_unix(),
        signature: String::new(),
        invariant_set_version: INVARIANT_SET_VERSION.to_string(),
        unsigned: true,
    })
}

/// Compute a provenance record for a raw content string (e.g. a commit message
/// or in-memory blob).  Useful for testing and for non-file artifacts.
// Used in unit tests and available as a public programmatic API.
#[allow(dead_code)]
pub fn sign_content(content: &[u8]) -> ProvenanceRecord {
    let artifact_hash = sha256_hex(content);
    // TODO: wire to signing-key storage layer (follow-up: real Ed25519 signing)
    ProvenanceRecord {
        artifact_hash,
        signer: String::new(),
        timestamp: now_unix(),
        signature: String::new(),
        invariant_set_version: INVARIANT_SET_VERSION.to_string(),
        unsigned: true,
    }
}

/// Produce a monetization intent record for `hash_or_path`.
///
/// If `hash_or_path` looks like a file path (contains `/` or `.`) and the
/// path exists on disk the artifact hash is derived by reading and hashing
/// the file.  Otherwise `hash_or_path` is treated as a pre-computed hex
/// digest.
///
/// Attribution targets are stubbed from `Cargo.toml` authors for v0.
///
/// # Errors
/// Returns an `io::Error` only when `hash_or_path` is a path and cannot be
/// read.
pub fn monetize_artifact(hash_or_path: &str) -> Result<MonetizationRecord, io::Error> {
    // Resolve the artifact hash.
    let artifact_hash = if looks_like_path(hash_or_path) && Path::new(hash_or_path).exists() {
        let data = std::fs::read(hash_or_path)?;
        sha256_hex(&data)
    } else {
        hash_or_path.to_string()
    };

    // TODO: wire to maintainer-resolution layer (follow-up: real attribution sourcing)
    // When royalty_weight::compute_attribution() is available (PR #18), call it here.
    // For v0, fall back to Cargo.toml authors.
    let targets = stub_attribution_targets();

    let total_weight: f64 = targets.iter().map(|t| t.weight).sum();
    let suggested_split: std::collections::HashMap<String, f64> = targets
        .iter()
        .map(|t| {
            let norm = if total_weight > 0.0 {
                t.weight / total_weight
            } else {
                0.0
            };
            (t.identity.clone(), norm)
        })
        .collect();

    Ok(MonetizationRecord {
        artifact_hash,
        attribution_targets: targets,
        suggested_split,
        status: "stub".to_string(),
    })
}

/// Heuristic: does the string look like a file path rather than a bare hash?
fn looks_like_path(s: &str) -> bool {
    s.contains('/') || s.contains('\\') || s.contains('.')
}

/// Stub attribution targets from Cargo.toml `[package].authors`.
///
/// TODO: replace with royalty_weight::compute_attribution() when available (PR #18).
fn stub_attribution_targets() -> Vec<AttributionTarget> {
    // Embed the authors list from Cargo.toml at compile time.
    const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");
    if AUTHORS.is_empty() {
        return vec![AttributionTarget {
            identity: "unknown".to_string(),
            weight: 1.0,
        }];
    }

    // Cargo encodes multiple authors separated by ":"
    let authors: Vec<&str> = AUTHORS.split(':').collect();
    let weight_each = 1.0 / authors.len() as f64;
    authors
        .iter()
        .map(|a| AttributionTarget {
            identity: a.trim().to_string(),
            weight: weight_each,
        })
        .collect()
}

// ─── CLI surface ─────────────────────────────────────────────────────────────

/// Build the `uws ip` clap command tree.
pub fn build_ip_command() -> clap::Command {
    use clap::{Arg, Command};

    Command::new("ip")
        .about("Regenerative IP & Provenance Engine — sign and monetize artifacts")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("sign")
                .about("Compute a SHA-256 provenance record for a file or content hash")
                .arg(
                    Arg::new("path")
                        .help("Path to the file to sign")
                        .required(true)
                        .index(1),
                ),
        )
        .subcommand(
            Command::new("monetize")
                .about("Emit a structured monetization intent record for an artifact")
                .arg(
                    Arg::new("hash_or_path")
                        .help("SHA-256 hex digest or path to the artifact")
                        .required(true)
                        .index(1),
                ),
        )
}

/// Handle `uws ip <subcommand> [args]`.
///
/// Returns `Ok(())` on success.  Errors are printed as JSON by the caller.
pub fn handle_ip_command(args: &[String]) -> Result<(), crate::error::GwsError> {
    let cmd = build_ip_command();

    // Prepend a fake argv[0] so clap can parse correctly.
    let parse_args: Vec<String> = std::iter::once("ip".to_string())
        .chain(args.iter().cloned())
        .collect();

    let matches = cmd.try_get_matches_from(&parse_args).map_err(|e| {
        if e.kind() == clap::error::ErrorKind::DisplayHelp
            || e.kind() == clap::error::ErrorKind::DisplayVersion
        {
            print!("{e}");
            std::process::exit(0);
        }
        crate::error::GwsError::Validation(e.to_string())
    })?;

    match matches.subcommand() {
        Some(("sign", sub)) => {
            let path = sub
                .get_one::<String>("path")
                .expect("path is required");
            let record = sign_artifact(path)
                .map_err(|e| crate::error::GwsError::Validation(format!("{e}")))?;
            let json = serde_json::to_string_pretty(&record)
                .map_err(|e| crate::error::GwsError::Validation(format!("{e}")))?;
            println!("{json}");
        }
        Some(("monetize", sub)) => {
            let hash_or_path = sub
                .get_one::<String>("hash_or_path")
                .expect("hash_or_path is required");
            let record = monetize_artifact(hash_or_path)
                .map_err(|e| crate::error::GwsError::Validation(format!("{e}")))?;
            let json = serde_json::to_string_pretty(&record)
                .map_err(|e| crate::error::GwsError::Validation(format!("{e}")))?;
            println!("{json}");
        }
        _ => unreachable!("subcommand_required enforced by clap"),
    }

    Ok(())
}

// ─── Unit tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    // ── Hash determinism ──────────────────────────────────────────────────────

    #[test]
    fn hash_is_deterministic_for_same_input() {
        let h1 = sha256_hex(b"hello world");
        let h2 = sha256_hex(b"hello world");
        assert_eq!(h1, h2, "SHA-256 must be deterministic");
    }

    #[test]
    fn hash_differs_for_different_inputs() {
        let h1 = sha256_hex(b"hello");
        let h2 = sha256_hex(b"world");
        assert_ne!(h1, h2);
    }

    #[test]
    fn hash_is_64_hex_chars() {
        let h = sha256_hex(b"test");
        assert_eq!(h.len(), 64, "SHA-256 hex digest must be 64 characters");
    }

    #[test]
    fn hash_known_value() {
        // Verified against: python3 -c "import hashlib; print(hashlib.sha256(b'abc').hexdigest())"
        // and: printf 'abc' | sha256sum
        let h = sha256_hex(b"abc");
        assert_eq!(
            h,
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
            "SHA-256 of 'abc' must match known vector"
        );
    }

    // ── sign_content JSON shape ───────────────────────────────────────────────

    #[test]
    fn provenance_record_has_required_fields() {
        let record = sign_content(b"test content");
        // Serialise to JSON and check all required keys are present.
        let v: serde_json::Value = serde_json::to_value(&record).unwrap();
        assert!(v.get("artifact_hash").is_some());
        assert!(v.get("signer").is_some());
        assert!(v.get("timestamp").is_some());
        assert!(v.get("signature").is_some());
        assert!(v.get("invariant_set_version").is_some());
        assert!(v.get("unsigned").is_some());
    }

    #[test]
    fn provenance_record_unsigned_fallback() {
        let record = sign_content(b"some data");
        assert!(record.unsigned, "v0 records must carry unsigned: true");
        assert!(
            record.signature.is_empty(),
            "unsigned record must have empty signature"
        );
        assert!(
            record.signer.is_empty(),
            "unsigned record must have empty signer"
        );
    }

    #[test]
    fn provenance_record_hash_is_deterministic_for_same_content() {
        let r1 = sign_content(b"deterministic");
        let r2 = sign_content(b"deterministic");
        assert_eq!(
            r1.artifact_hash, r2.artifact_hash,
            "same content must produce same artifact_hash"
        );
    }

    #[test]
    fn provenance_record_invariant_set_version_matches_constant() {
        let record = sign_content(b"v");
        assert_eq!(record.invariant_set_version, INVARIANT_SET_VERSION);
    }

    // ── sign_artifact (file I/O) ──────────────────────────────────────────────

    #[test]
    fn sign_artifact_reads_file_and_hashes() {
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        tmp.write_all(b"file content for signing").unwrap();
        let path = tmp.path().to_str().unwrap().to_string();

        let record = sign_artifact(&path).unwrap();
        assert_eq!(record.artifact_hash, sha256_hex(b"file content for signing"));
        assert!(record.unsigned);
    }

    #[test]
    fn sign_artifact_missing_file_returns_error() {
        let result = sign_artifact("/nonexistent/path/does_not_exist.bin");
        assert!(result.is_err(), "missing file must return an error");
    }

    // ── monetize_artifact JSON shape ─────────────────────────────────────────

    #[test]
    fn monetization_record_has_required_fields() {
        let record = monetize_artifact("deadbeef1234").unwrap();
        let v: serde_json::Value = serde_json::to_value(&record).unwrap();
        assert!(v.get("artifact_hash").is_some());
        assert!(v.get("attribution_targets").is_some());
        assert!(v.get("suggested_split").is_some());
        assert!(v.get("status").is_some());
    }

    #[test]
    fn monetization_record_status_is_stub() {
        let record = monetize_artifact("abc123").unwrap();
        assert_eq!(record.status, "stub");
    }

    #[test]
    fn monetization_record_hash_passthrough() {
        let hash = "aabbccdd1122";
        let record = monetize_artifact(hash).unwrap();
        assert_eq!(record.artifact_hash, hash);
    }

    #[test]
    fn monetization_record_from_file() {
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        tmp.write_all(b"monetize this").unwrap();
        let path = tmp.path().to_str().unwrap().to_string();

        let record = monetize_artifact(&path).unwrap();
        let expected_hash = sha256_hex(b"monetize this");
        assert_eq!(record.artifact_hash, expected_hash);
    }

    #[test]
    fn monetization_split_weights_sum_to_one() {
        let record = monetize_artifact("somehash").unwrap();
        let total: f64 = record.suggested_split.values().sum();
        // floating point: allow small epsilon
        assert!(
            (total - 1.0).abs() < 1e-9 || record.attribution_targets.is_empty(),
            "suggested_split weights must sum to 1.0, got {total}"
        );
    }
}
