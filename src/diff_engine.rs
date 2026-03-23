// src/diff_engine.rs
// Aluminum OS — Cross-Provider Document Diff Engine
//
// Novel Invention #12 — Cross-Provider Document Diffing
//
// When you have a Google Doc and an equivalent Notion page, or a GitHub
// README and its OneDrive copy, this engine computes a structured diff
// between the two, without needing to understand the native format of
// either provider.
//
// The diff engine works on normalized `UniversalDocument` (plain text/Markdown)
// representations. It produces a line-level diff annotated with change type
// (Added, Removed, Unchanged) and provider attribution.
//
// Commands:
//   uws diff --left-provider google-drive --left-id "fileId1" \
//            --right-provider notion      --right-id "pageId1"
//
// Author: GitHub Copilot — Aluminum OS Session 2026-03-23

#![allow(dead_code)]

// ─── Change type ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChangeKind {
    Unchanged,
    Added,
    Removed,
    Modified,
}

impl ChangeKind {
    pub fn as_symbol(&self) -> &str {
        match self {
            ChangeKind::Unchanged => " ",
            ChangeKind::Added => "+",
            ChangeKind::Removed => "-",
            ChangeKind::Modified => "~",
        }
    }
}

// ─── Diff hunk ────────────────────────────────────────────────────────────

/// A single line in a diff output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffLine {
    pub kind: ChangeKind,
    /// Line number in the left document (None for added lines).
    pub left_line: Option<usize>,
    /// Line number in the right document (None for removed lines).
    pub right_line: Option<usize>,
    pub content: String,
}

impl DiffLine {
    pub fn unchanged(left: usize, right: usize, content: impl Into<String>) -> Self {
        DiffLine {
            kind: ChangeKind::Unchanged,
            left_line: Some(left),
            right_line: Some(right),
            content: content.into(),
        }
    }

    pub fn added(right: usize, content: impl Into<String>) -> Self {
        DiffLine {
            kind: ChangeKind::Added,
            left_line: None,
            right_line: Some(right),
            content: content.into(),
        }
    }

    pub fn removed(left: usize, content: impl Into<String>) -> Self {
        DiffLine {
            kind: ChangeKind::Removed,
            left_line: Some(left),
            right_line: None,
            content: content.into(),
        }
    }
}

// ─── Diff result ──────────────────────────────────────────────────────────

/// The result of diffing two documents.
#[derive(Debug, Clone)]
pub struct DiffResult {
    pub left_provider: String,
    pub right_provider: String,
    pub left_title: String,
    pub right_title: String,
    pub lines: Vec<DiffLine>,
    pub stats: DiffStats,
}

#[derive(Debug, Clone, Default)]
pub struct DiffStats {
    pub lines_added: usize,
    pub lines_removed: usize,
    pub lines_unchanged: usize,
    pub similarity_percent: u8,
}

impl DiffStats {
    fn compute(lines: &[DiffLine]) -> DiffStats {
        let added = lines.iter().filter(|l| l.kind == ChangeKind::Added).count();
        let removed = lines.iter().filter(|l| l.kind == ChangeKind::Removed).count();
        let unchanged = lines.iter().filter(|l| l.kind == ChangeKind::Unchanged).count();
        let total = added + removed + unchanged;
        let similarity = if total == 0 {
            100
        } else {
            ((unchanged * 100) / total).min(100) as u8
        };
        DiffStats {
            lines_added: added,
            lines_removed: removed,
            lines_unchanged: unchanged,
            similarity_percent: similarity,
        }
    }
}

// ─── Diff algorithm ───────────────────────────────────────────────────────

/// Compute the diff between two text documents using the Longest Common
/// Subsequence (LCS) algorithm.
///
/// This is the classic Myers diff algorithm simplified to line granularity.
/// Input: two slices of lines (pre-split by caller).
pub fn diff_lines<'a>(left: &'a [&str], right: &'a [&str]) -> Vec<DiffLine> {
    let lcs = lcs_matrix(left, right);
    backtrack_diff(&lcs, left, right, left.len(), right.len())
}

/// Compute the LCS (Longest Common Subsequence) matrix.
fn lcs_matrix(left: &[&str], right: &[&str]) -> Vec<Vec<usize>> {
    let m = left.len();
    let n = right.len();
    let mut dp = vec![vec![0usize; n + 1]; m + 1];

    for i in 1..=m {
        for j in 1..=n {
            if left[i - 1] == right[j - 1] {
                dp[i][j] = dp[i - 1][j - 1] + 1;
            } else {
                dp[i][j] = dp[i - 1][j].max(dp[i][j - 1]);
            }
        }
    }
    dp
}

/// Backtrack through the LCS matrix to produce diff lines.
fn backtrack_diff(
    dp: &[Vec<usize>],
    left: &[&str],
    right: &[&str],
    i: usize,
    j: usize,
) -> Vec<DiffLine> {
    if i == 0 && j == 0 {
        return Vec::new();
    }
    if i == 0 {
        let mut result = backtrack_diff(dp, left, right, i, j - 1);
        result.push(DiffLine::added(j, right[j - 1]));
        return result;
    }
    if j == 0 {
        let mut result = backtrack_diff(dp, left, right, i - 1, j);
        result.push(DiffLine::removed(i, left[i - 1]));
        return result;
    }
    if left[i - 1] == right[j - 1] {
        let mut result = backtrack_diff(dp, left, right, i - 1, j - 1);
        result.push(DiffLine::unchanged(i, j, left[i - 1]));
        return result;
    }
    if dp[i - 1][j] >= dp[i][j - 1] {
        let mut result = backtrack_diff(dp, left, right, i - 1, j);
        result.push(DiffLine::removed(i, left[i - 1]));
        result
    } else {
        let mut result = backtrack_diff(dp, left, right, i, j - 1);
        result.push(DiffLine::added(j, right[j - 1]));
        result
    }
}

// ─── Document diff entry point ───────────────────────────────────────────

/// A document descriptor for the diff engine.
#[derive(Debug, Clone)]
pub struct DocDescriptor {
    pub provider: String,
    pub title: String,
    pub content: String,
}

impl DocDescriptor {
    pub fn new(provider: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        DocDescriptor {
            provider: provider.into(),
            title: title.into(),
            content: content.into(),
        }
    }
}

/// Compute a diff between two provider documents.
pub fn diff_documents(left: &DocDescriptor, right: &DocDescriptor) -> DiffResult {
    let left_lines: Vec<&str> = left.content.lines().collect();
    let right_lines: Vec<&str> = right.content.lines().collect();

    let lines = diff_lines(&left_lines, &right_lines);
    let stats = DiffStats::compute(&lines);

    DiffResult {
        left_provider: left.provider.clone(),
        right_provider: right.provider.clone(),
        left_title: left.title.clone(),
        right_title: right.title.clone(),
        lines,
        stats,
    }
}

// ─── Unified diff formatter ───────────────────────────────────────────────

/// Format a `DiffResult` as a unified diff string.
pub fn format_unified_diff(result: &DiffResult) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "--- {}/{}\n+++ {}/{}\n",
        result.left_provider, result.left_title,
        result.right_provider, result.right_title,
    ));
    out.push_str(&format!(
        "  {} lines added, {} lines removed, {}% similarity\n",
        result.stats.lines_added,
        result.stats.lines_removed,
        result.stats.similarity_percent,
    ));
    for line in &result.lines {
        out.push_str(&format!("{} {}\n", line.kind.as_symbol(), line.content));
    }
    out
}

// ─── Unit tests ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diff_identical_documents() {
        let left = ["line1", "line2", "line3"];
        let right = ["line1", "line2", "line3"];
        let diff = diff_lines(&left, &right);
        assert!(diff.iter().all(|l| l.kind == ChangeKind::Unchanged));
        assert_eq!(diff.len(), 3);
    }

    #[test]
    fn test_diff_single_addition() {
        let left = ["line1", "line3"];
        let right = ["line1", "line2", "line3"];
        let diff = diff_lines(&left, &right);
        let added: Vec<_> = diff.iter().filter(|l| l.kind == ChangeKind::Added).collect();
        assert_eq!(added.len(), 1);
        assert_eq!(added[0].content, "line2");
    }

    #[test]
    fn test_diff_single_removal() {
        let left = ["line1", "line2", "line3"];
        let right = ["line1", "line3"];
        let diff = diff_lines(&left, &right);
        let removed: Vec<_> = diff.iter().filter(|l| l.kind == ChangeKind::Removed).collect();
        assert_eq!(removed.len(), 1);
        assert_eq!(removed[0].content, "line2");
    }

    #[test]
    fn test_diff_completely_different() {
        let left = ["aaa", "bbb"];
        let right = ["ccc", "ddd"];
        let diff = diff_lines(&left, &right);
        let removed: Vec<_> = diff.iter().filter(|l| l.kind == ChangeKind::Removed).collect();
        let added: Vec<_> = diff.iter().filter(|l| l.kind == ChangeKind::Added).collect();
        assert_eq!(removed.len(), 2);
        assert_eq!(added.len(), 2);
    }

    #[test]
    fn test_diff_empty_left() {
        let left: [&str; 0] = [];
        let right = ["line1", "line2"];
        let diff = diff_lines(&left, &right);
        assert_eq!(diff.len(), 2);
        assert!(diff.iter().all(|l| l.kind == ChangeKind::Added));
    }

    #[test]
    fn test_diff_empty_right() {
        let left = ["line1", "line2"];
        let right: [&str; 0] = [];
        let diff = diff_lines(&left, &right);
        assert_eq!(diff.len(), 2);
        assert!(diff.iter().all(|l| l.kind == ChangeKind::Removed));
    }

    #[test]
    fn test_diff_stats_similarity_100_percent_identical() {
        let left = ["a", "b", "c"];
        let right = ["a", "b", "c"];
        let diff = diff_lines(&left, &right);
        let stats = DiffStats::compute(&diff);
        assert_eq!(stats.similarity_percent, 100);
        assert_eq!(stats.lines_added, 0);
        assert_eq!(stats.lines_removed, 0);
    }

    #[test]
    fn test_diff_stats_similarity_0_percent_completely_different() {
        let left = ["a", "b"];
        let right = ["c", "d"];
        let diff = diff_lines(&left, &right);
        let stats = DiffStats::compute(&diff);
        assert_eq!(stats.similarity_percent, 0);
    }

    #[test]
    fn test_diff_documents_end_to_end() {
        let left = DocDescriptor::new("google-drive", "budget.docx", "Revenue: $1M\nExpenses: $800K\n");
        let right = DocDescriptor::new("notion", "Budget Page", "Revenue: $1M\nExpenses: $900K\nProfit: $100K\n");
        let result = diff_documents(&left, &right);
        assert_eq!(result.left_provider, "google-drive");
        assert_eq!(result.right_provider, "notion");
        assert!(result.stats.lines_removed > 0 || result.stats.lines_added > 0);
    }

    #[test]
    fn test_format_unified_diff() {
        let left = DocDescriptor::new("drive", "doc.md", "Hello\nWorld\n");
        let right = DocDescriptor::new("notion", "page", "Hello\nUniverse\n");
        let result = diff_documents(&left, &right);
        let formatted = format_unified_diff(&result);
        assert!(formatted.contains("---"));
        assert!(formatted.contains("+++"));
        assert!(formatted.contains("drive"));
        assert!(formatted.contains("notion"));
    }

    #[test]
    fn test_change_kind_symbols() {
        assert_eq!(ChangeKind::Added.as_symbol(), "+");
        assert_eq!(ChangeKind::Removed.as_symbol(), "-");
        assert_eq!(ChangeKind::Unchanged.as_symbol(), " ");
    }

    #[test]
    fn test_diff_line_constructors() {
        let added = DiffLine::added(5, "new line");
        assert!(added.left_line.is_none());
        assert_eq!(added.right_line, Some(5));

        let removed = DiffLine::removed(3, "old line");
        assert_eq!(removed.left_line, Some(3));
        assert!(removed.right_line.is_none());

        let unchanged = DiffLine::unchanged(2, 2, "same line");
        assert_eq!(unchanged.left_line, Some(2));
        assert_eq!(unchanged.right_line, Some(2));
    }
}
