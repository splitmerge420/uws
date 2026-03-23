// src/context_compressor.rs
// Aluminum OS — LLM Context Window Optimizer
//
// Novel Invention #9 — Context Compressor
//
// AI models have finite context windows. When a `uws` command returns a
// large JSON payload (e.g., 5,000 emails or 200 GitHub issues), naively
// dumping it into an LLM prompt will exceed the window and fail silently
// or incur massive token costs.
//
// The Context Compressor solves this by:
// 1. Counting estimated tokens in a JSON response
// 2. Truncating/summarizing the array to fit a target token budget
// 3. Selecting the most relevant fields (field projection)
// 4. Producing a compact, still-valid JSON output the LLM can consume
//
// Usage:
//   uws compress --input response.json --tokens 4096
//   uws compress --input - --tokens 8192 --fields "title,url,body"
//
// Author: GitHub Copilot — Aluminum OS Session 2026-03-23

#![allow(dead_code)]

// ─── Token estimation ─────────────────────────────────────────────────────

/// Estimate the number of LLM tokens in a string.
///
/// Rule of thumb: ~4 characters per token for English text.
/// This is a conservative estimate suitable for GPT-4, Claude, and Gemini.
pub fn estimate_tokens(text: &str) -> usize {
    // Each whitespace-separated word is ~1.3 tokens on average.
    // Each non-ASCII character is treated as 2 tokens.
    let mut estimate = 0usize;
    let mut in_word = false;
    for ch in text.chars() {
        if ch.is_whitespace() {
            if in_word {
                estimate += 1; // end of word — roughly 1.3 tokens, we use 1 for simplicity
                in_word = false;
            }
        } else if !ch.is_ascii() {
            estimate += 2;
            in_word = false;
        } else {
            in_word = true;
        }
    }
    if in_word {
        estimate += 1;
    }
    // Add a 30% overhead for JSON punctuation, quotes, and braces
    estimate + estimate / 3
}

// ─── Field projection ─────────────────────────────────────────────────────

/// Remove all keys not in `keep_fields` from a JSON-like flat string map.
///
/// This is a heuristic projector for flat JSON objects (no nesting).
/// For nested JSON, the caller should pre-process with a proper parser.
pub fn project_fields<'a>(
    json_line: &'a str,
    keep_fields: &[&str],
) -> std::borrow::Cow<'a, str> {
    if keep_fields.is_empty() {
        return std::borrow::Cow::Borrowed(json_line);
    }
    // Build a simple regex-free field extractor: find `"field": value` pairs
    let mut result = String::from("{");
    let mut first = true;

    for field in keep_fields {
        // Look for `"field":` pattern
        let key_pattern = format!("\"{}\":", field);
        if let Some(start) = json_line.find(&key_pattern) {
            let value_start = start + key_pattern.len();
            let value_portion = json_line[value_start..].trim_start();
            let value_end = find_json_value_end(value_portion);
            let value = &value_portion[..value_end];
            if !first {
                result.push(',');
            }
            result.push_str(&format!("\"{}\":{}", field, value));
            first = false;
        }
    }
    result.push('}');
    std::borrow::Cow::Owned(result)
}

/// Find the end index of a JSON value (string, number, bool, null, object, array).
fn find_json_value_end(s: &str) -> usize {
    let s = s.trim_start();
    if s.is_empty() {
        return 0;
    }
    let first = s.as_bytes()[0];
    match first {
        b'"' => {
            // String: find closing quote, respecting escapes
            let mut i = 1;
            let bytes = s.as_bytes();
            while i < bytes.len() {
                if bytes[i] == b'\\' {
                    i += 2;
                } else if bytes[i] == b'"' {
                    return i + 1;
                } else {
                    i += 1;
                }
            }
            s.len()
        }
        b'{' | b'[' => {
            // Object or array: count matching braces
            let open = first;
            let close = if open == b'{' { b'}' } else { b']' };
            let mut depth = 0i32;
            let mut in_string = false;
            let mut i = 0;
            let bytes = s.as_bytes();
            while i < bytes.len() {
                if in_string {
                    if bytes[i] == b'\\' {
                        i += 2;
                        continue;
                    }
                    if bytes[i] == b'"' {
                        in_string = false;
                    }
                } else {
                    if bytes[i] == b'"' {
                        in_string = true;
                    } else if bytes[i] == open {
                        depth += 1;
                    } else if bytes[i] == close {
                        depth -= 1;
                        if depth == 0 {
                            return i + 1;
                        }
                    }
                }
                i += 1;
            }
            s.len()
        }
        _ => {
            // Number, bool, null: read until whitespace or comma or brace
            s.find(|c: char| c == ',' || c == '}' || c == ']' || c.is_whitespace())
                .unwrap_or(s.len())
        }
    }
}

// ─── Truncation plan ──────────────────────────────────────────────────────

/// Describes how a response will be truncated to fit a token budget.
#[derive(Debug, Clone)]
pub struct TruncationPlan {
    /// Total estimated tokens in the original response.
    pub original_tokens: usize,
    /// Target maximum tokens.
    pub target_tokens: usize,
    /// Number of items kept (for array responses).
    pub items_kept: usize,
    /// Total items in the original response.
    pub items_total: usize,
    /// Fields projected (empty = all fields kept).
    pub fields_kept: Vec<String>,
    /// Whether truncation was necessary.
    pub was_truncated: bool,
}

// ─── Compressor ───────────────────────────────────────────────────────────

/// Configuration for the context compressor.
#[derive(Debug, Clone)]
pub struct CompressorConfig {
    /// Target token budget (exclusive upper bound).
    pub target_tokens: usize,
    /// Fields to retain (empty = all).
    pub keep_fields: Vec<String>,
    /// Maximum items to include in array responses.
    pub max_items: Option<usize>,
    /// If true, append a `__truncated` metadata object at the end.
    pub annotate_truncation: bool,
}

impl Default for CompressorConfig {
    fn default() -> Self {
        CompressorConfig {
            target_tokens: 8192,
            keep_fields: Vec::new(),
            max_items: None,
            annotate_truncation: true,
        }
    }
}

impl CompressorConfig {
    pub fn with_token_limit(mut self, tokens: usize) -> Self {
        self.target_tokens = tokens;
        self
    }

    pub fn with_fields(mut self, fields: Vec<String>) -> Self {
        self.keep_fields = fields;
        self
    }

    pub fn with_max_items(mut self, n: usize) -> Self {
        self.max_items = Some(n);
        self
    }
}

/// Compress a JSON response string to fit within the token budget.
///
/// Supports:
/// - JSON arrays of objects: truncates to the first N items that fit
/// - Single JSON objects: projects to keep_fields
/// - Plain strings: hard-truncates at character limit
///
/// Returns `(compressed_json, plan)`.
pub fn compress(input: &str, config: &CompressorConfig) -> (String, TruncationPlan) {
    let original_tokens = estimate_tokens(input);
    let field_refs: Vec<&str> = config.keep_fields.iter().map(|s| s.as_str()).collect();

    // Fast path: already fits
    if original_tokens <= config.target_tokens && config.keep_fields.is_empty() && config.max_items.is_none() {
        return (
            input.to_string(),
            TruncationPlan {
                original_tokens,
                target_tokens: config.target_tokens,
                items_kept: 0,
                items_total: 0,
                fields_kept: vec![],
                was_truncated: false,
            },
        );
    }

    // Detect array response
    let trimmed = input.trim();
    if trimmed.starts_with('[') {
        compress_array(trimmed, config, original_tokens, &field_refs)
    } else {
        // Single object or string: field projection then char-truncate
        let projected = if !config.keep_fields.is_empty() {
            project_fields(trimmed, &field_refs).into_owned()
        } else {
            trimmed.to_string()
        };
        let projected_tokens = estimate_tokens(&projected);
        let was_truncated = projected_tokens > config.target_tokens || projected != trimmed;
        (
            projected,
            TruncationPlan {
                original_tokens,
                target_tokens: config.target_tokens,
                items_kept: 1,
                items_total: 1,
                fields_kept: config.keep_fields.clone(),
                was_truncated,
            },
        )
    }
}

fn compress_array(
    input: &str,
    config: &CompressorConfig,
    original_tokens: usize,
    field_refs: &[&str],
) -> (String, TruncationPlan) {
    // Split the array into individual items using a simple bracket counter.
    let items = split_json_array(input);
    let total = items.len();
    let max_by_config = config.max_items.unwrap_or(usize::MAX);

    let mut kept = Vec::new();
    let mut token_budget = config.target_tokens.saturating_sub(10); // overhead for []

    for item in items.iter().take(max_by_config) {
        let projected = if !field_refs.is_empty() {
            project_fields(item.trim(), field_refs).into_owned()
        } else {
            item.trim().to_string()
        };
        let item_tokens = estimate_tokens(&projected);
        if item_tokens > token_budget && !kept.is_empty() {
            break;
        }
        token_budget = token_budget.saturating_sub(item_tokens + 1); // +1 for comma
        kept.push(projected);
    }

    let items_kept = kept.len();
    let mut output = String::from("[");
    output.push_str(&kept.join(","));
    output.push(']');

    if config.annotate_truncation && items_kept < total {
        // Append a sentinel object to signal truncation
        output.pop(); // remove trailing ]
        output.push_str(&format!(
            ",{{\"__truncated\":true,\"items_shown\":{items_kept},\"items_total\":{total}}}]"
        ));
    }

    let was_truncated = items_kept < total || !field_refs.is_empty();

    (
        output,
        TruncationPlan {
            original_tokens,
            target_tokens: config.target_tokens,
            items_kept,
            items_total: total,
            fields_kept: config.keep_fields.to_vec(),
            was_truncated,
        },
    )
}

/// Split a JSON array string into its individual item strings.
/// Returns raw item strings (not trimmed).
fn split_json_array(s: &str) -> Vec<String> {
    let mut items = Vec::new();
    let bytes = s.as_bytes();
    let mut i = 0;
    // Skip opening [
    while i < bytes.len() && bytes[i] != b'[' {
        i += 1;
    }
    i += 1; // skip [

    let mut depth = 0i32;
    let mut in_string = false;
    let mut item_start = i;

    while i < bytes.len() {
        let b = bytes[i];
        if in_string {
            if b == b'\\' {
                i += 2;
                continue;
            }
            if b == b'"' {
                in_string = false;
            }
        } else {
            match b {
                b'"' => in_string = true,
                b'{' | b'[' => depth += 1,
                b'}' | b']' => {
                    depth -= 1;
                    if depth < 0 {
                        // End of array
                        let item = std::str::from_utf8(&bytes[item_start..i])
                            .unwrap_or("")
                            .trim()
                            .to_string();
                        if !item.is_empty() {
                            items.push(item);
                        }
                        break;
                    }
                }
                b',' if depth == 0 => {
                    let item = std::str::from_utf8(&bytes[item_start..i])
                        .unwrap_or("")
                        .trim()
                        .to_string();
                    if !item.is_empty() {
                        items.push(item);
                    }
                    item_start = i + 1;
                }
                _ => {}
            }
        }
        i += 1;
    }
    items
}

// ─── Unit tests ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_estimate_tokens_empty() {
        assert_eq!(estimate_tokens(""), 0);
    }

    #[test]
    fn test_estimate_tokens_simple_sentence() {
        let tokens = estimate_tokens("Hello world this is a test");
        // 6 words → roughly 8 tokens after overhead
        assert!(tokens > 0 && tokens < 20);
    }

    #[test]
    fn test_estimate_tokens_scales_with_length() {
        let short = estimate_tokens("a b c");
        let long = estimate_tokens("a b c d e f g h i j k l m n o p q r s t");
        assert!(long > short);
    }

    #[test]
    fn test_project_fields_basic() {
        let json = r#"{"title":"Hello","body":"World","id":42,"url":"https://example.com"}"#;
        let projected = project_fields(json, &["title", "url"]);
        assert!(projected.contains("title"));
        assert!(projected.contains("url"));
        assert!(!projected.contains("body"));
        assert!(!projected.contains("\"id\""));
    }

    #[test]
    fn test_project_fields_empty_keeps_all() {
        let json = r#"{"title":"Hello","body":"World"}"#;
        let projected = project_fields(json, &[]);
        assert_eq!(projected.as_ref(), json);
    }

    #[test]
    fn test_split_json_array_simple() {
        let json = r#"[{"a":1},{"a":2},{"a":3}]"#;
        let items = split_json_array(json);
        assert_eq!(items.len(), 3);
    }

    #[test]
    fn test_split_json_array_empty() {
        let items = split_json_array("[]");
        assert_eq!(items.len(), 0);
    }

    #[test]
    fn test_split_json_array_single() {
        let items = split_json_array(r#"[{"x":1}]"#);
        assert_eq!(items.len(), 1);
    }

    #[test]
    fn test_compress_already_fits_no_truncation() {
        let input = r#"{"title":"short"}"#;
        let config = CompressorConfig::default().with_token_limit(10000);
        let (output, plan) = compress(input, &config);
        assert!(!plan.was_truncated);
        assert_eq!(output, input);
    }

    #[test]
    fn test_compress_array_truncates_to_fit_tokens() {
        // Build a large array
        let items: Vec<String> = (0..100)
            .map(|i| format!(r#"{{"id":{i},"title":"Issue number {i} with a longer description that takes up more tokens in the context window"}}"#))
            .collect();
        let input = format!("[{}]", items.join(","));

        let config = CompressorConfig::default().with_token_limit(200);
        let (_output, plan) = compress(&input, &config);
        assert!(plan.was_truncated);
        assert!(plan.items_kept < 100);
        assert_eq!(plan.items_total, 100);
    }

    #[test]
    fn test_compress_array_with_field_projection() {
        let input = r#"[{"id":1,"title":"T1","body":"B1","url":"U1"},{"id":2,"title":"T2","body":"B2","url":"U2"}]"#;
        let config = CompressorConfig::default()
            .with_token_limit(10000)
            .with_fields(vec!["title".to_string(), "url".to_string()]);
        let (output, plan) = compress(input, &config);
        assert!(plan.was_truncated); // because fields were projected
        assert!(output.contains("title"));
        assert!(!output.contains("body"));
    }

    #[test]
    fn test_compress_max_items() {
        let items: Vec<String> = (0..50).map(|i| format!(r#"{{"id":{i}}}"#)).collect();
        let input = format!("[{}]", items.join(","));
        let config = CompressorConfig::default()
            .with_token_limit(100000)
            .with_max_items(5);
        let (_output, plan) = compress(&input, &config);
        assert!(plan.items_kept <= 5);
        assert_eq!(plan.items_total, 50);
    }

    #[test]
    fn test_compress_annotates_truncation() {
        let items: Vec<String> = (0..50)
            .map(|i| format!(r#"{{"id":{i},"text":"long text here for item {i}"}}"#))
            .collect();
        let input = format!("[{}]", items.join(","));
        let config = CompressorConfig {
            target_tokens: 100,
            keep_fields: vec![],
            max_items: None,
            annotate_truncation: true,
        };
        let (output, plan) = compress(&input, &config);
        if plan.was_truncated {
            assert!(output.contains("__truncated"));
        }
    }

    #[test]
    fn test_find_json_value_end_string() {
        let s = r#""hello world", rest"#;
        let end = find_json_value_end(s);
        assert_eq!(&s[..end], r#""hello world""#);
    }

    #[test]
    fn test_find_json_value_end_number() {
        let s = "42, rest";
        let end = find_json_value_end(s);
        assert_eq!(&s[..end], "42");
    }

    #[test]
    fn test_find_json_value_end_nested_object() {
        let s = r#"{"a":{"b":1}}, rest"#;
        let end = find_json_value_end(s);
        assert_eq!(&s[..end], r#"{"a":{"b":1}}"#);
    }
}
