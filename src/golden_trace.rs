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

//! GoldenTrace provenance trailer.
//!
//! Appends a git-trailer-style block to commit messages or PR descriptions
//! when commands are run. Format:
//!
//! ```text
//! GoldenTrace: <hash> <timestamp> <invariant-set-version>
//! ```
//!
//! The hash is a SHA-256 digest of the content being annotated (command args,
//! PR ref, or description body), truncated to 16 hex characters for brevity.
//!
//! # Integration points
//! Wire this into commit-message formatters, PR description writers, and
//! audit-chain entries. The `format_trailer` function is intentionally
//! standalone so it can be called from any writer without an async context.

use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};

/// Current invariant-set version tag embedded in every trailer.
pub const INVARIANT_SET_VERSION: &str = "inv-set/v0";

/// A GoldenTrace provenance record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GoldenTrace {
    /// Truncated SHA-256 hex digest (16 chars) of the annotated content.
    pub hash: String,
    /// UTC timestamp in RFC 3339 format.
    pub timestamp: String,
    /// Invariant-set version tag.
    pub invariant_set_version: String,
}

impl GoldenTrace {
    /// Build a new `GoldenTrace` from arbitrary content, using the current UTC time.
    ///
    /// `content` is hashed with SHA-256; the first 16 hex characters are stored.
    pub fn new(content: &str) -> Self {
        Self::with_timestamp(content, Utc::now())
    }

    /// Build a `GoldenTrace` with an explicit timestamp (useful in tests).
    pub fn with_timestamp(content: &str, ts: DateTime<Utc>) -> Self {
        let hash = compute_hash(content);
        GoldenTrace {
            hash,
            timestamp: ts.to_rfc3339(),
            invariant_set_version: INVARIANT_SET_VERSION.to_string(),
        }
    }

    /// Return the single-line trailer string.
    ///
    /// ```text
    /// GoldenTrace: a1b2c3d4e5f6a7b8 2026-04-25T04:00:00+00:00 inv-set/v0
    /// ```
    pub fn to_trailer(&self) -> String {
        format!(
            "GoldenTrace: {} {} {}",
            self.hash, self.timestamp, self.invariant_set_version
        )
    }

    /// Append the trailer to an existing message, separated by a blank line.
    ///
    /// Follows the git-trailer convention: trailers go after a blank line at
    /// the end of the message body.
    pub fn append_to_message(&self, message: &str) -> String {
        let trimmed = message.trim_end();
        format!("{}\n\n{}\n", trimmed, self.to_trailer())
    }
}

/// Compute a truncated SHA-256 hex digest (first 16 hex chars) of `content`.
pub fn compute_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    let result = hasher.finalize();
    // Each byte is 2 hex chars; 8 bytes → 16 hex chars
    result
        .iter()
        .take(8)
        .map(|b| format!("{b:02x}"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn fixed_ts() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 4, 25, 4, 0, 0).unwrap()
    }

    #[test]
    fn test_hash_length() {
        let h = compute_hash("hello world");
        assert_eq!(h.len(), 16, "hash should be 16 hex characters");
    }

    #[test]
    fn test_hash_is_hex() {
        let h = compute_hash("test content");
        assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_deterministic() {
        let h1 = compute_hash("same content");
        let h2 = compute_hash("same content");
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_trailer_format() {
        let gt = GoldenTrace::with_timestamp("test", fixed_ts());
        let t = gt.to_trailer();
        assert!(t.starts_with("GoldenTrace: "), "trailer must start with key");
        // hash (16 chars) + space + timestamp + space + version
        let parts: Vec<&str> = t.splitn(4, ' ').collect();
        assert_eq!(parts[0], "GoldenTrace:");
        assert_eq!(parts[1].len(), 16);
        assert!(parts[2].contains("2026-04-25"));
        assert_eq!(parts[3], INVARIANT_SET_VERSION);
    }

    #[test]
    fn test_append_to_message() {
        let gt = GoldenTrace::with_timestamp("body", fixed_ts());
        let msg = "feat: add something\n\nThis is the body.";
        let result = gt.append_to_message(msg);
        assert!(result.contains("\n\nGoldenTrace:"), "must have blank line before trailer");
        assert!(result.ends_with('\n'), "must end with newline");
    }

    #[test]
    fn test_append_strips_trailing_whitespace() {
        let gt = GoldenTrace::with_timestamp("x", fixed_ts());
        let msg = "fix: typo   \n\n  ";
        let result = gt.append_to_message(msg);
        // The original trailing whitespace should be stripped
        assert!(!result.contains("   \n\nGoldenTrace:"));
    }
}
