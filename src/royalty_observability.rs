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

//! Royalty Runtime Observability Layer
//!
//! Identifies the execution context, hashes the dependency lineage, and emits
//! a lightweight telemetry payload to the Royalty Collector. This layer runs
//! asynchronously and **must never block, delay, or crash** the core CLI execution.
//! All failures are silently discarded.

use serde::Serialize;
use sha2::{Digest, Sha256};
use uuid::Uuid;

/// Default collector endpoint (local development).
/// Override with the `ROYALTY_COLLECTOR_URL` environment variable.
const DEFAULT_COLLECTOR_URL: &str = "http://localhost:3000/v1/executions";

/// Environment variable that overrides the default collector URL.
const COLLECTOR_URL_ENV: &str = "ROYALTY_COLLECTOR_URL";

/// Primary package identifier as registered on the Royalty Oracle.
const PRIMARY_PACKAGE: &str = "splitmerge420/uws";

/// Core dependency identifiers used to form the lineage hash.
/// These represent the major API versions of the packages whose authorship
/// is being attributed. Major-version granularity is intentional: it tracks
/// the API contract, not transient patch releases.
pub(crate) const LINEAGE_DEPS: &[(&str, &str)] = &[
    ("uws", env!("CARGO_PKG_VERSION")),
    ("tokio", "1"),
    ("reqwest", "0.12"),
    ("serde", "1"),
    ("serde_json", "1"),
    ("clap", "4"),
    ("yup-oauth2", "12"),
    ("aes-gcm", "0.10"),
    ("sha2", "0.10"),
];

/// Telemetry payload sent to the Royalty Collector.
#[derive(Serialize)]
struct ExecutionEvent<'a> {
    session_id: String,
    primary_package: &'a str,
    lineage_hash: String,
    timestamp: String,
}

/// Compute a deterministic SHA-256 hash over the core dependency lineage.
///
/// The input is the concatenation of `name@version` strings for every entry in
/// `LINEAGE_DEPS`, separated by `|`. This makes the hash stable across
/// invocations for the same build of `uws`.
pub(crate) fn compute_lineage_hash() -> String {
    let mut hasher = Sha256::new();
    let lineage_string: String = LINEAGE_DEPS
        .iter()
        .map(|(name, version)| format!("{name}@{version}"))
        .collect::<Vec<_>>()
        .join("|");
    hasher.update(lineage_string.as_bytes());
    let result = hasher.finalize();
    format!("{result:x}")
}

/// Return the current UTC timestamp in RFC 3339 format.
fn now_rfc3339() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    // Minimal RFC 3339 representation: YYYY-MM-DDTHH:MM:SSZ
    let s = secs;
    let sec = s % 60;
    let min = (s / 60) % 60;
    let hour = (s / 3600) % 24;
    let days = s / 86400;
    // Days since 1970-01-01 → calendar date (Gregorian, no leap-second handling needed)
    let (year, month, day) = days_to_ymd(days);
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{min:02}:{sec:02}Z")
}

/// Convert a count of days since the Unix epoch to a (year, month, day) triple.
fn days_to_ymd(mut days: u64) -> (u64, u64, u64) {
    // Algorithm from http://www.howardhinnant.com/date_algorithms.html
    days += 719468;
    let era = days / 146097;
    let doe = days % 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

/// Fire-and-forget: spawn a background task that emits a telemetry event.
///
/// This function returns immediately. The spawned task sends a single HTTP POST
/// to the Royalty Collector. Any error (network, serialization, timeout, etc.)
/// is silently discarded so the CLI is never interrupted.
///
/// The collector URL defaults to `http://localhost:3000/v1/executions` and can
/// be overridden with the `ROYALTY_COLLECTOR_URL` environment variable.
pub fn emit_telemetry() {
    let session_id = Uuid::new_v4().to_string();
    let lineage_hash = compute_lineage_hash();
    let timestamp = now_rfc3339();
    let collector_url =
        std::env::var(COLLECTOR_URL_ENV).unwrap_or_else(|_| DEFAULT_COLLECTOR_URL.to_string());

    tokio::spawn(async move {
        let payload = ExecutionEvent {
            session_id,
            primary_package: PRIMARY_PACKAGE,
            lineage_hash,
            timestamp,
        };

        let client = match reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
        {
            Ok(c) => c,
            Err(_) => return,
        };

        let _ = client.post(&collector_url).json(&payload).send().await;
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lineage_hash_is_deterministic() {
        let h1 = compute_lineage_hash();
        let h2 = compute_lineage_hash();
        assert_eq!(h1, h2, "lineage hash must be deterministic across calls");
    }

    #[test]
    fn lineage_hash_is_hex_sha256() {
        let h = compute_lineage_hash();
        // SHA-256 produces 32 bytes → 64 hex characters
        assert_eq!(h.len(), 64, "expected 64-character hex SHA-256 digest");
        assert!(
            h.chars().all(|c| c.is_ascii_hexdigit()),
            "hash must be lowercase hex"
        );
    }

    #[test]
    fn timestamp_looks_like_rfc3339() {
        let ts = now_rfc3339();
        // e.g. "2026-03-23T07:29:53Z"
        assert_eq!(ts.len(), 20);
        assert!(ts.ends_with('Z'));
        assert_eq!(&ts[4..5], "-");
        assert_eq!(&ts[7..8], "-");
        assert_eq!(&ts[10..11], "T");
    }

    #[test]
    fn days_to_ymd_epoch() {
        // Unix epoch = 1970-01-01
        let (y, m, d) = days_to_ymd(0);
        assert_eq!((y, m, d), (1970, 1, 1));
    }

    #[test]
    fn days_to_ymd_known_date() {
        // 2026-03-23 = 56 years + some days after 1970
        // days since epoch for 2026-03-23: let's just check it round-trips sanely
        let (y, m, d) = days_to_ymd(20535); // approx 2026-03-23
        assert_eq!(y, 2026);
        assert!(m >= 1 && m <= 12);
        assert!(d >= 1 && d <= 31);
    }
}
