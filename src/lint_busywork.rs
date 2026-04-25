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

//! `uws lint` — code and workflow quality linters.
//!
//! # Subcommands
//! - `uws lint busywork [--dir <path>]` — scan for AntiBusyworkFactor (ABF)
//!   patterns: work that generates activity without meaningful value.
//!
//! # AntiBusyworkFactor (ABF) v0 ruleset
//!
//! Rules shipped in this release:
//!
//! | ID | Name | Description |
//! |----|------|-------------|
//! | ABF-001 | pure-version-bump | Commit messages that are nothing but a version number change |
//! | ABF-002 | whitespace-only-diff | Large PRs (>50 files) where nearly all changes are whitespace |
//! | ABF-003 | no-op-rename | Files renamed with no content changes |
//! | ABF-004 | changelog-only | PR that only touches CHANGELOG / CHANGES files |
//! | ABF-005 | auto-generated-noise | Commits whose message indicates automated bulk generation |
//!
//! Additional rules can be registered by implementing the `BusyworkRule` trait.

use crate::error::GwsError;
use clap::{Arg, ArgMatches, Command};
use serde_json::json;
use std::path::{Path, PathBuf};

// ─── Rule metadata ────────────────────────────────────────────────────────────

/// Severity of a busywork finding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Severity {
    Warning,
    Error,
    Info,
}

impl Severity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Severity::Warning => "warning",
            Severity::Error => "error",
            Severity::Info => "info",
        }
    }
}

/// A single busywork finding.
#[derive(Debug, Clone)]
pub struct Finding {
    /// Rule ID (e.g. "ABF-001").
    pub rule_id: String,
    /// Human-readable rule name.
    pub rule_name: String,
    /// Severity level.
    pub severity: Severity,
    /// What triggered this finding.
    pub detail: String,
    /// Optional file or commit reference.
    pub location: Option<String>,
}

impl Finding {
    fn to_json(&self) -> serde_json::Value {
        json!({
            "rule_id": self.rule_id,
            "rule_name": self.rule_name,
            "severity": self.severity.as_str(),
            "detail": self.detail,
            "location": self.location,
        })
    }
}

// ─── Rule trait ───────────────────────────────────────────────────────────────

/// Trait for a single busywork detection rule.
pub trait BusyworkRule: Send + Sync {
    /// Rule identifier (e.g. "ABF-001").
    fn id(&self) -> &'static str;
    /// Human-readable name.
    fn name(&self) -> &'static str;
    /// Check a scan input and return any findings.
    fn check(&self, input: &ScanInput) -> Vec<Finding>;
}

// ─── Scan input ───────────────────────────────────────────────────────────────

/// Input passed to every rule.
pub struct ScanInput {
    /// Commit messages found in the target directory's git log (if available).
    pub commit_messages: Vec<String>,
    /// List of file paths changed in the last relevant operation (e.g. a PR diff).
    pub changed_files: Vec<String>,
    /// Raw diff lines (may be empty when running outside a git context).
    pub diff_lines: Vec<String>,
}

// ─── Built-in rules (v0 starter ruleset) ─────────────────────────────────────

/// ABF-001 — Pure version-bump commit message.
///
/// Fires when a commit message matches patterns like:
/// - "bump version to 1.2.3"
/// - "v1.2.3"
/// - "release 1.2.3"
/// - "1.2.3" (standalone semver)
struct RulePureVersionBump;

impl BusyworkRule for RulePureVersionBump {
    fn id(&self) -> &'static str {
        "ABF-001"
    }
    fn name(&self) -> &'static str {
        "pure-version-bump"
    }
    fn check(&self, input: &ScanInput) -> Vec<Finding> {
        input
            .commit_messages
            .iter()
            .filter(|msg| is_pure_version_bump(msg))
            .map(|msg| Finding {
                rule_id: self.id().to_string(),
                rule_name: self.name().to_string(),
                severity: Severity::Warning,
                detail: format!("Commit message appears to be a pure version bump: {msg:?}"),
                location: Some(msg.clone()),
            })
            .collect()
    }
}

/// ABF-002 — Whitespace-only large PR.
///
/// Fires when more than 50 files are changed but the diff is ≥90 % whitespace.
struct RuleWhitespaceOnlyDiff;

impl BusyworkRule for RuleWhitespaceOnlyDiff {
    fn id(&self) -> &'static str {
        "ABF-002"
    }
    fn name(&self) -> &'static str {
        "whitespace-only-diff"
    }
    fn check(&self, input: &ScanInput) -> Vec<Finding> {
        if input.changed_files.len() <= 50 {
            return vec![];
        }
        let (whitespace, total) = diff_whitespace_ratio(&input.diff_lines);
        if total == 0 {
            return vec![];
        }
        let ratio = whitespace as f64 / total as f64;
        if ratio >= 0.90 {
            vec![Finding {
                rule_id: self.id().to_string(),
                rule_name: self.name().to_string(),
                severity: Severity::Warning,
                detail: format!(
                    "PR touches {} files but {:.0}% of diff lines are whitespace-only",
                    input.changed_files.len(),
                    ratio * 100.0
                ),
                location: None,
            }]
        } else {
            vec![]
        }
    }
}

/// ABF-003 — No-op rename (file renamed, content unchanged).
///
/// Detects `rename ... => ...` in diff output where the body is empty.
struct RuleNoOpRename;

impl BusyworkRule for RuleNoOpRename {
    fn id(&self) -> &'static str {
        "ABF-003"
    }
    fn name(&self) -> &'static str {
        "no-op-rename"
    }
    fn check(&self, input: &ScanInput) -> Vec<Finding> {
        // Look for lines like "rename src/{a => b}/file.rs" with similarity 100%
        input
            .diff_lines
            .iter()
            .filter(|l| l.contains("similarity index 100%"))
            .map(|_| Finding {
                rule_id: self.id().to_string(),
                rule_name: self.name().to_string(),
                severity: Severity::Info,
                detail: "File renamed with no content changes (100% similarity)".to_string(),
                location: None,
            })
            .collect()
    }
}

/// ABF-004 — Changelog-only PR.
///
/// Fires when every changed file is a changelog / release-notes file.
struct RuleChangelogOnly;

impl BusyworkRule for RuleChangelogOnly {
    fn id(&self) -> &'static str {
        "ABF-004"
    }
    fn name(&self) -> &'static str {
        "changelog-only"
    }
    fn check(&self, input: &ScanInput) -> Vec<Finding> {
        if input.changed_files.is_empty() {
            return vec![];
        }
        let all_changelog = input.changed_files.iter().all(|f| {
            let lower = f.to_lowercase();
            lower.contains("changelog")
                || lower.contains("changes")
                || lower.contains("release_notes")
                || lower.contains("release-notes")
                || lower.ends_with(".changeset")
        });
        if all_changelog {
            vec![Finding {
                rule_id: self.id().to_string(),
                rule_name: self.name().to_string(),
                severity: Severity::Info,
                detail: format!(
                    "All {} changed files appear to be changelog/release-notes files",
                    input.changed_files.len()
                ),
                location: None,
            }]
        } else {
            vec![]
        }
    }
}

/// ABF-005 — Auto-generated noise commit.
///
/// Fires when a commit message clearly indicates automated bulk generation
/// (e.g. "Update generated files", "Auto-update", "[skip ci]", "chore(bot)").
struct RuleAutoGeneratedNoise;

impl BusyworkRule for RuleAutoGeneratedNoise {
    fn id(&self) -> &'static str {
        "ABF-005"
    }
    fn name(&self) -> &'static str {
        "auto-generated-noise"
    }
    fn check(&self, input: &ScanInput) -> Vec<Finding> {
        let patterns = [
            "auto-update",
            "auto update",
            "update generated",
            "generated files",
            "bot:",
            "chore(bot)",
            "[bot]",
            "automated commit",
            "auto-generated",
        ];
        input
            .commit_messages
            .iter()
            .filter(|msg| {
                let lower = msg.to_lowercase();
                patterns.iter().any(|p| lower.contains(p))
            })
            .map(|msg| Finding {
                rule_id: self.id().to_string(),
                rule_name: self.name().to_string(),
                severity: Severity::Info,
                detail: format!("Commit message suggests auto-generated content: {msg:?}"),
                location: Some(msg.clone()),
            })
            .collect()
    }
}

// ─── Registry ─────────────────────────────────────────────────────────────────

/// Return the default v0 starter ruleset.
pub fn default_rules() -> Vec<Box<dyn BusyworkRule>> {
    vec![
        Box::new(RulePureVersionBump),
        Box::new(RuleWhitespaceOnlyDiff),
        Box::new(RuleNoOpRename),
        Box::new(RuleChangelogOnly),
        Box::new(RuleAutoGeneratedNoise),
    ]
}

// ─── Scanner ──────────────────────────────────────────────────────────────────

/// Run all rules against `input` and return the findings.
pub fn scan(input: &ScanInput, rules: &[Box<dyn BusyworkRule>]) -> Vec<Finding> {
    rules.iter().flat_map(|r| r.check(input)).collect()
}

/// Build a `ScanInput` by reading commit messages from a git repository at
/// `dir`, plus optional extra context.
///
/// Falls back gracefully if `git` is not available or the directory is not
/// a git repo.
pub fn build_scan_input_from_dir(dir: &Path) -> ScanInput {
    let commit_messages = read_git_log(dir);
    let diff_lines = read_git_diff(dir);
    let changed_files = extract_changed_files(&diff_lines);
    ScanInput {
        commit_messages,
        changed_files,
        diff_lines,
    }
}

// ─── CLI plumbing ─────────────────────────────────────────────────────────────

/// Build the top-level `lint` command.
pub fn build_lint_command() -> Command {
    Command::new("lint")
        .about("Code and workflow quality linters")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(build_busywork_command())
}

/// Build the `lint busywork` subcommand.
fn build_busywork_command() -> Command {
    Command::new("busywork")
        .about("Scan for AntiBusyworkFactor (ABF) patterns — work without value (v0 ruleset)")
        .arg(
            Arg::new("dir")
                .long("dir")
                .short('d')
                .help("Directory to scan (default: current working directory)")
                .value_name("PATH"),
        )
        .arg(
            Arg::new("format")
                .long("format")
                .short('f')
                .help("Output format: json (default) or table")
                .value_name("FORMAT")
                .default_value("json"),
        )
        .arg(
            Arg::new("strict")
                .long("strict")
                .help("Exit with code 1 if any warnings or errors are found")
                .action(clap::ArgAction::SetTrue),
        )
        .after_help(
            "\
ABF RULES (v0):
  ABF-001  pure-version-bump    Commit messages that are nothing but a version bump
  ABF-002  whitespace-only-diff Large PRs (>50 files) with ≥90% whitespace changes
  ABF-003  no-op-rename         Files renamed with no content changes
  ABF-004  changelog-only       PR that only touches changelog/release-notes files
  ABF-005  auto-generated-noise Commits whose messages indicate automated bulk generation

EXAMPLES:
  uws lint busywork
  uws lint busywork --dir ./my-repo
  uws lint busywork --format table
  uws lint busywork --strict
",
        )
}

/// Handle the `lint` command tree.
pub async fn handle_lint_command(args: &[String]) -> Result<(), GwsError> {
    let cmd = build_lint_command();
    let matches = cmd
        .try_get_matches_from(std::iter::once("lint".to_string()).chain(args.iter().cloned()))
        .map_err(|e| {
            if e.kind() == clap::error::ErrorKind::DisplayHelp
                || e.kind() == clap::error::ErrorKind::DisplayVersion
            {
                print!("{e}");
                std::process::exit(0);
            }
            GwsError::Validation(e.to_string())
        })?;

    match matches.subcommand() {
        Some(("busywork", sub)) => handle_busywork(sub).await,
        _ => Err(GwsError::Validation(
            "Unknown lint subcommand. Run `uws lint --help` for usage.".to_string(),
        )),
    }
}

async fn handle_busywork(matches: &ArgMatches) -> Result<(), GwsError> {
    let dir: PathBuf = matches
        .get_one::<String>("dir")
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    let format = matches
        .get_one::<String>("format")
        .map(|s| s.as_str())
        .unwrap_or("json");

    let strict = matches.get_flag("strict");

    let input = build_scan_input_from_dir(&dir);
    let rules = default_rules();
    let findings = scan(&input, &rules);

    let has_issue = findings
        .iter()
        .any(|f| f.severity == Severity::Warning || f.severity == Severity::Error);

    match format {
        "table" => print_table(&findings),
        _ => print_json(&findings, &dir),
    }

    if strict && has_issue {
        std::process::exit(1);
    }

    Ok(())
}

// ─── Output formatters ────────────────────────────────────────────────────────

fn print_json(findings: &[Finding], dir: &Path) {
    let items: Vec<_> = findings.iter().map(|f| f.to_json()).collect();
    let report = json!({
        "ruleset_version": "v0",
        "scanned_dir": dir.display().to_string(),
        "total_findings": findings.len(),
        "findings": items,
    });
    println!("{}", serde_json::to_string_pretty(&report).unwrap());
}

fn print_table(findings: &[Finding]) {
    if findings.is_empty() {
        println!("No busywork findings.");
        return;
    }
    println!(
        "{:<10} {:<30} {:<10} {}",
        "Rule", "Name", "Severity", "Detail"
    );
    println!("{}", "-".repeat(80));
    for f in findings {
        println!(
            "{:<10} {:<30} {:<10} {}",
            f.rule_id,
            f.rule_name,
            f.severity.as_str(),
            f.detail
        );
    }
}

// ─── Git helpers ──────────────────────────────────────────────────────────────

fn read_git_log(dir: &Path) -> Vec<String> {
    let output = std::process::Command::new("git")
        .args(["log", "--pretty=format:%s", "-50"])
        .current_dir(dir)
        .output();
    match output {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout)
            .lines()
            .map(|l| l.to_string())
            .collect(),
        _ => vec![],
    }
}

fn read_git_diff(dir: &Path) -> Vec<String> {
    let output = std::process::Command::new("git")
        .args(["diff", "HEAD~1..HEAD", "--stat"])
        .current_dir(dir)
        .output();
    match output {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout)
            .lines()
            .map(|l| l.to_string())
            .collect(),
        _ => vec![],
    }
}

fn extract_changed_files(diff_lines: &[String]) -> Vec<String> {
    // git diff --stat lines look like: " path/to/file.rs | 5 ++"
    diff_lines
        .iter()
        .filter_map(|l| {
            let parts: Vec<&str> = l.splitn(2, '|').collect();
            if parts.len() == 2 {
                Some(parts[0].trim().to_string())
            } else {
                None
            }
        })
        .filter(|s| !s.is_empty())
        .collect()
}

/// Count whitespace-only lines vs total changed lines in a diff.
///
/// A "whitespace-only" line is one that starts with `+` or `-` (a changed
/// line) and whose remainder is empty or all whitespace.
fn diff_whitespace_ratio(diff_lines: &[String]) -> (usize, usize) {
    let changed: Vec<&String> = diff_lines
        .iter()
        .filter(|l| l.starts_with('+') || l.starts_with('-'))
        .filter(|l| !l.starts_with("+++") && !l.starts_with("---"))
        .collect();
    let total = changed.len();
    let whitespace = changed
        .iter()
        .filter(|l| l[1..].trim().is_empty())
        .count();
    (whitespace, total)
}

/// Heuristic: is this commit message purely a version bump?
fn is_pure_version_bump(msg: &str) -> bool {
    let lower = msg.trim().to_lowercase();

    // Patterns: "v1.2.3", "1.2.3", "bump to 1.2.3", "release 1.2.3",
    //           "bump version", "version bump", "chore: bump", "chore(release)"
    let semver_only = lower
        .trim_start_matches('v')
        .split('.')
        .count()
        == 3
        && lower
            .trim_start_matches('v')
            .replace('.', "")
            .chars()
            .all(|c| c.is_ascii_digit());

    let bump_keywords = [
        "bump version",
        "version bump",
        "bump to",
        "release ",
        "chore: bump",
        "chore(release)",
        "chore: release",
        "prepare release",
        "cut release",
    ];

    semver_only || bump_keywords.iter().any(|kw| lower.contains(kw))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_input(
        commits: Vec<&str>,
        files: Vec<&str>,
        diff: Vec<&str>,
    ) -> ScanInput {
        ScanInput {
            commit_messages: commits.into_iter().map(|s| s.to_string()).collect(),
            changed_files: files.into_iter().map(|s| s.to_string()).collect(),
            diff_lines: diff.into_iter().map(|s| s.to_string()).collect(),
        }
    }

    // ── ABF-001 ─────────────────────────────────────────────────────────────

    #[test]
    fn abf001_fires_on_pure_semver() {
        let input = make_input(vec!["1.2.3"], vec![], vec![]);
        let rule = RulePureVersionBump;
        let findings = rule.check(&input);
        assert!(!findings.is_empty(), "ABF-001 should fire on '1.2.3'");
        assert_eq!(findings[0].rule_id, "ABF-001");
    }

    #[test]
    fn abf001_fires_on_bump_keyword() {
        let input = make_input(vec!["chore: bump version to 2.0.0"], vec![], vec![]);
        let findings = RulePureVersionBump.check(&input);
        assert!(!findings.is_empty(), "ABF-001 should fire on 'bump version'");
    }

    #[test]
    fn abf001_silent_on_real_commit() {
        let input = make_input(vec!["feat: add GoldenTrace provenance trailer"], vec![], vec![]);
        let findings = RulePureVersionBump.check(&input);
        assert!(findings.is_empty(), "ABF-001 must not fire on real commits");
    }

    // ── ABF-002 ─────────────────────────────────────────────────────────────

    #[test]
    fn abf002_fires_on_large_whitespace_only_pr() {
        let files: Vec<&str> = (0..55).map(|_| "src/file.rs").collect();
        // 100 diff lines that are all whitespace
        let diff: Vec<&str> = (0..100).map(|_| "+   ").collect();
        let input = make_input(vec![], files, diff);
        let findings = RuleWhitespaceOnlyDiff.check(&input);
        assert!(!findings.is_empty(), "ABF-002 should fire on large whitespace-only PR");
    }

    #[test]
    fn abf002_silent_on_small_pr() {
        let files: Vec<&str> = (0..10).map(|_| "src/file.rs").collect();
        let input = make_input(vec![], files, vec![]);
        let findings = RuleWhitespaceOnlyDiff.check(&input);
        assert!(findings.is_empty(), "ABF-002 must not fire on small PR");
    }

    #[test]
    fn abf002_silent_on_real_changes() {
        let files: Vec<&str> = (0..55).map(|_| "src/file.rs").collect();
        // Mix of real changes and whitespace
        let mut diff = vec!["+   "; 20]; // 20 whitespace
        diff.extend(vec!["+ let x = 1;"; 80]); // 80 real lines
        let input = make_input(vec![], files, diff);
        let findings = RuleWhitespaceOnlyDiff.check(&input);
        assert!(findings.is_empty(), "ABF-002 must not fire if < 90% whitespace");
    }

    // ── ABF-004 ─────────────────────────────────────────────────────────────

    #[test]
    fn abf004_fires_on_changelog_only_pr() {
        let input = make_input(vec![], vec!["CHANGELOG.md", "CHANGES.md"], vec![]);
        let findings = RuleChangelogOnly.check(&input);
        assert!(!findings.is_empty(), "ABF-004 should fire on changelog-only PR");
    }

    #[test]
    fn abf004_silent_on_mixed_pr() {
        let input = make_input(
            vec![],
            vec!["CHANGELOG.md", "src/main.rs"],
            vec![],
        );
        let findings = RuleChangelogOnly.check(&input);
        assert!(findings.is_empty(), "ABF-004 must not fire on mixed PR");
    }

    // ── ABF-005 ─────────────────────────────────────────────────────────────

    #[test]
    fn abf005_fires_on_auto_generated_commit() {
        let input = make_input(vec!["auto-update generated files"], vec![], vec![]);
        let findings = RuleAutoGeneratedNoise.check(&input);
        assert!(!findings.is_empty(), "ABF-005 should fire on auto-generated commit");
    }

    #[test]
    fn abf005_silent_on_normal_commit() {
        let input = make_input(vec!["feat: implement lint busywork"], vec![], vec![]);
        let findings = RuleAutoGeneratedNoise.check(&input);
        assert!(findings.is_empty(), "ABF-005 must not fire on normal commits");
    }

    // ── Full scan ────────────────────────────────────────────────────────────

    #[test]
    fn full_scan_aggregates_findings() {
        let input = make_input(
            vec!["1.2.3", "feat: real work", "auto-update generated files"],
            vec![],
            vec![],
        );
        let rules = default_rules();
        let findings = scan(&input, &rules);
        // ABF-001 fires on "1.2.3", ABF-005 fires on "auto-update..."
        assert!(findings.len() >= 2, "should have at least 2 findings");
    }

    // ── CLI ──────────────────────────────────────────────────────────────────

    #[test]
    fn test_build_lint_command_has_busywork() {
        let cmd = build_lint_command();
        let subs: Vec<_> = cmd.get_subcommands().map(|s| s.get_name()).collect();
        assert!(subs.contains(&"busywork"), "lint must have a 'busywork' subcommand");
    }

    #[tokio::test]
    async fn test_handle_lint_busywork_runs() {
        let dir = std::env::temp_dir();
        let dir_str = dir.to_string_lossy().to_string();
        let args = vec!["busywork".to_string(), "--dir".to_string(), dir_str];
        // Should not panic even with no git context
        let result = handle_lint_command(&args).await;
        assert!(result.is_ok(), "lint busywork must not fail on a non-git dir");
    }
}
