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

//! Royalty v0 Package Attribution Weighting
//!
//! Applies the `primary_plus_equal_split` model to the dependency lineage:
//!
//! * **40%** → primary package (`splitmerge420/uws`)
//! * **60%** → split equally among the remaining core dependencies
//!
//! This is step 3 of the Royalty Oracle staircase:
//!   1. Verified lineage payload
//!   2. Lineage hash  ← already implemented in `royalty_observability`
//!   3. **Package attribution map**  ← this module
//!   4. Maintainer resolution
//!   5. Payout routing

use serde::Serialize;

use crate::royalty_observability::{compute_lineage_hash, LINEAGE_DEPS};

/// Share of the total weight assigned to the primary package.
const PRIMARY_WEIGHT: f64 = 0.40;

/// Model version string embedded in every attribution report.
const MODEL_VERSION: &str = "0.1";

/// Weighting strategy identifier.
const WEIGHTING_STRATEGY: &str = "primary_plus_equal_split";

// ─── Attribution types ────────────────────────────────────────────────────────

/// A single entry in the attribution map.
#[derive(Debug, Serialize, PartialEq)]
pub struct AttributionEntry {
    pub package: String,
    /// Fractional share of the total weight (0.0 – 1.0).
    pub weight: f64,
    /// Either `"primary"` or `"dependency"`.
    pub role: String,
}

/// The complete v0 attribution report returned by `compute_attribution`.
#[derive(Debug, Serialize)]
pub struct AttributionReport {
    pub status: String,
    pub command: String,
    pub model_version: String,
    pub lineage_hash: String,
    pub weighting_strategy: String,
    pub total_weight: f64,
    pub attribution_map: Vec<AttributionEntry>,
}

// ─── Core logic ───────────────────────────────────────────────────────────────

/// Compute the v0 `primary_plus_equal_split` attribution report.
///
/// The first entry in `LINEAGE_DEPS` is treated as the primary package (it is
/// always `("uws", …)`). All subsequent entries are dependencies and receive
/// equal shares of the remaining weight.
pub fn compute_attribution() -> AttributionReport {
    let lineage_hash = compute_lineage_hash();

    // Split deps into primary (index 0) and the rest.
    let (primary_name, _primary_ver) = LINEAGE_DEPS[0];
    let dep_entries = &LINEAGE_DEPS[1..];

    let dep_count = dep_entries.len();
    // Each dependency's share: 60% divided equally. Use round-trip f64 arithmetic;
    // precision is adequate for a display-oriented attribution report.
    let dep_weight = if dep_count > 0 {
        (1.0 - PRIMARY_WEIGHT) / dep_count as f64
    } else {
        0.0
    };

    let mut attribution_map = Vec::with_capacity(1 + dep_count);

    // Primary entry — always first in the map.
    attribution_map.push(AttributionEntry {
        package: primary_name.to_string(),
        weight: PRIMARY_WEIGHT,
        role: "primary".to_string(),
    });

    // Dependency entries — one per remaining lineage dep.
    for (name, _ver) in dep_entries {
        attribution_map.push(AttributionEntry {
            package: name.to_string(),
            weight: round_to_6_decimals(dep_weight),
            role: "dependency".to_string(),
        });
    }

    // Total weight should sum to 1.0 (floating-point: sum primary + deps * dep_weight).
    let total_weight = round_to_6_decimals(PRIMARY_WEIGHT + dep_weight * dep_count as f64);

    AttributionReport {
        status: "ok".to_string(),
        command: "weight".to_string(),
        model_version: MODEL_VERSION.to_string(),
        lineage_hash,
        weighting_strategy: WEIGHTING_STRATEGY.to_string(),
        total_weight,
        attribution_map,
    }
}

/// Round a float to 6 decimal places (sufficient precision for display).
fn round_to_6_decimals(v: f64) -> f64 {
    (v * 1_000_000.0).round() / 1_000_000.0
}

// ─── CLI handler ──────────────────────────────────────────────────────────────

/// Handle `uws royalty <subcommand>` invocations.
///
/// Currently supported subcommands:
/// * `weight` — print the v0 attribution report as JSON
pub fn handle_royalty_command(args: &[String]) {
    // args[0] = binary, args[1] = "royalty", args[2] = subcommand
    let subcmd = args.get(2).map(|s| s.as_str()).unwrap_or("weight");

    match subcmd {
        "weight" | "--help" | "-h" => {
            if subcmd == "--help" || subcmd == "-h" {
                println!("Usage: uws royalty weight");
                println!();
                println!("Compute the v0 package attribution map for the current lineage.");
                println!("Strategy: primary_plus_equal_split");
                println!("  Primary package: 40%");
                println!("  Each dependency:  60% / N dependencies");
                return;
            }
            let report = compute_attribution();
            match serde_json::to_string_pretty(&report) {
                Ok(json) => println!("{json}"),
                Err(e) => eprintln!("royalty weight: serialization error: {e}"),
            }
        }
        other => {
            eprintln!("uws royalty: unknown subcommand '{other}'");
            eprintln!("Available subcommands: weight");
            std::process::exit(1);
        }
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attribution_total_weight_is_one() {
        let report = compute_attribution();
        // Total should be 1.0 (within floating-point tolerance).
        assert!(
            (report.total_weight - 1.0).abs() < 1e-9,
            "total_weight should be 1.0, got {}",
            report.total_weight
        );
    }

    #[test]
    fn attribution_map_sums_to_one() {
        let report = compute_attribution();
        let sum: f64 = report.attribution_map.iter().map(|e| e.weight).sum();
        assert!(
            (sum - 1.0).abs() < 1e-9,
            "attribution_map weights must sum to 1.0, got {sum}"
        );
    }

    #[test]
    fn primary_entry_is_first_and_has_correct_weight() {
        let report = compute_attribution();
        let primary = report
            .attribution_map
            .first()
            .expect("attribution_map must not be empty");
        assert_eq!(primary.role, "primary");
        assert_eq!(primary.package, "uws");
        assert!(
            (primary.weight - 0.40).abs() < 1e-9,
            "primary weight must be 0.40, got {}",
            primary.weight
        );
    }

    #[test]
    fn all_deps_have_equal_weight() {
        let report = compute_attribution();
        let dep_entries: Vec<_> = report
            .attribution_map
            .iter()
            .filter(|e| e.role == "dependency")
            .collect();
        assert!(
            !dep_entries.is_empty(),
            "must have at least one dependency entry"
        );
        let first_weight = dep_entries[0].weight;
        for entry in &dep_entries {
            assert!(
                (entry.weight - first_weight).abs() < 1e-9,
                "all dependency weights must be equal; {} has {}, expected {}",
                entry.package,
                entry.weight,
                first_weight
            );
        }
    }

    #[test]
    fn dep_count_matches_lineage_deps_minus_one() {
        let report = compute_attribution();
        let dep_count = report
            .attribution_map
            .iter()
            .filter(|e| e.role == "dependency")
            .count();
        assert_eq!(
            dep_count,
            LINEAGE_DEPS.len() - 1,
            "should have one dependency entry per non-primary lineage dep"
        );
    }

    #[test]
    fn report_has_correct_metadata() {
        let report = compute_attribution();
        assert_eq!(report.status, "ok");
        assert_eq!(report.command, "weight");
        assert_eq!(report.model_version, "0.1");
        assert_eq!(report.weighting_strategy, "primary_plus_equal_split");
        // lineage_hash is a 64-char hex SHA-256
        assert_eq!(report.lineage_hash.len(), 64);
        assert!(report.lineage_hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn report_serializes_to_valid_json() {
        let report = compute_attribution();
        let json = serde_json::to_string(&report).expect("serialization must succeed");
        // Must contain the key fields
        assert!(json.contains("\"status\":\"ok\""));
        assert!(json.contains("\"command\":\"weight\""));
        assert!(json.contains("\"weighting_strategy\":\"primary_plus_equal_split\""));
        assert!(json.contains("\"attribution_map\""));
    }
}
