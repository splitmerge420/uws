// src/universal/ai_cli.rs
// Universal AI CLI Adapter Layer — Aluminum OS
//
// Defines a provider-agnostic trait for AI CLI interactions and concrete adapters
// for GitHub Copilot, Claude (Anthropic), Gemini (Google), and OpenAI.
//
// All adapters produce a uniform `AiCliResponse` so the ModelRouter can treat
// every provider identically, independent of the underlying API surface.
//
// Design: no I/O or external crates — adapters build request/response structs
// that are serialised by the caller.  Actual HTTP execution happens upstream.
//
// Enforces: INV-6 (Provider Abstraction), INV-7 (Vendor Balance)

use std::collections::BTreeMap;

// ─── Canonical provider enum ──────────────────────────────────

/// All supported AI CLI providers.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AiProvider {
    /// `gh copilot` — GitHub Copilot CLI
    GithubCopilot,
    /// Anthropic Claude API / Claude CLI wrappers
    Claude,
    /// Google Gemini CLI / Vertex AI
    Gemini,
    /// OpenAI GPT API
    OpenAi,
}

impl AiProvider {
    /// Human-readable display name.
    pub fn display_name(&self) -> &'static str {
        match self {
            AiProvider::GithubCopilot => "GitHub Copilot",
            AiProvider::Claude => "Anthropic Claude",
            AiProvider::Gemini => "Google Gemini",
            AiProvider::OpenAi => "OpenAI",
        }
    }

    /// Environment variable that carries the bearer token for this provider.
    pub fn token_env_var(&self) -> &'static str {
        match self {
            AiProvider::GithubCopilot => "GITHUB_TOKEN",
            AiProvider::Claude => "ANTHROPIC_API_KEY",
            AiProvider::Gemini => "GEMINI_API_KEY",
            AiProvider::OpenAi => "OPENAI_API_KEY",
        }
    }

    /// Parse a provider from a string alias (case-insensitive).
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "github" | "github-copilot" | "gh" | "copilot" => Some(AiProvider::GithubCopilot),
            "claude" | "anthropic" => Some(AiProvider::Claude),
            "gemini" | "google" | "vertex" => Some(AiProvider::Gemini),
            "openai" | "gpt" | "chatgpt" => Some(AiProvider::OpenAi),
            _ => None,
        }
    }
}

// ─── Uniform request ──────────────────────────────────────────

/// A provider-agnostic AI CLI request.
#[derive(Debug, Clone)]
pub struct AiCliRequest {
    /// Which provider to route to.
    pub provider: AiProvider,
    /// The user's natural-language prompt.
    pub prompt: String,
    /// Optional system instruction / role context.
    pub system: Option<String>,
    /// Model identifier (e.g. "claude-opus-4", "gpt-4o", "gemini-2.0-flash").
    /// `None` lets each adapter choose a sensible default.
    pub model: Option<String>,
    /// Caller-supplied metadata threaded through for audit/provenance.
    pub metadata: BTreeMap<String, String>,
}

impl AiCliRequest {
    /// Convenience constructor with the minimum required fields.
    pub fn new(provider: AiProvider, prompt: impl Into<String>) -> Self {
        AiCliRequest {
            provider,
            prompt: prompt.into(),
            system: None,
            model: None,
            metadata: BTreeMap::new(),
        }
    }
}

// ─── Uniform response ─────────────────────────────────────────

/// Outcome of an AI CLI call, in a provider-agnostic shape.
#[derive(Debug, Clone)]
pub struct AiCliResponse {
    /// Which provider generated this response.
    pub provider: AiProvider,
    /// Model that was actually used (as reported by the provider).
    pub model_used: String,
    /// The raw text response from the AI.
    pub content: String,
    /// Provider-specific fields preserved verbatim (for pass-through fidelity).
    pub raw_fields: BTreeMap<String, String>,
    /// Wall-clock latency in milliseconds (0 if not measured).
    pub latency_ms: u64,
    /// Whether the response was truncated due to token limits.
    pub truncated: bool,
}

impl AiCliResponse {
    /// Serialise to a deterministic JSON string (no external serde dep required).
    pub fn to_json(&self) -> String {
        let raw: String = self
            .raw_fields
            .iter()
            .map(|(k, v)| format!("\"{}\":\"{}\"", escape_json(k), escape_json(v)))
            .collect::<Vec<_>>()
            .join(",");

        format!(
            "{{\"provider\":\"{}\",\"model_used\":\"{}\",\"content\":{},\
             \"latency_ms\":{},\"truncated\":{},\"raw_fields\":{{{}}}}}",
            escape_json(self.provider.display_name()),
            escape_json(&self.model_used),
            json_string(&self.content),
            self.latency_ms,
            self.truncated,
            raw,
        )
    }
}

// ─── Adapter trait ────────────────────────────────────────────

/// Implemented by each AI provider adapter.
///
/// Adapters are intentionally pure value-transformers — they build the HTTP
/// request body and parse the response body without performing any I/O.
/// This keeps them testable in isolation and OS-agnostic.
pub trait AiCliAdapter {
    /// The provider this adapter handles.
    fn provider(&self) -> AiProvider;

    /// Build the JSON request body for this provider's API.
    fn build_request_body(&self, request: &AiCliRequest) -> String;

    /// Parse a raw JSON response body into the uniform `AiCliResponse`.
    /// Returns `Err` if the response cannot be parsed.
    fn parse_response(&self, request: &AiCliRequest, raw: &str) -> Result<AiCliResponse, String>;

    /// The HTTPS endpoint URL for this provider's completions API.
    fn endpoint_url(&self) -> &'static str;
}

// ─── GitHub Copilot adapter ───────────────────────────────────

/// Adapter for `gh copilot` / GitHub Copilot API.
pub struct GithubCopilotAdapter;

impl AiCliAdapter for GithubCopilotAdapter {
    fn provider(&self) -> AiProvider {
        AiProvider::GithubCopilot
    }

    fn build_request_body(&self, request: &AiCliRequest) -> String {
        let model = request
            .model
            .as_deref()
            .unwrap_or("gpt-4o");
        let system = request.system.as_deref().unwrap_or(
            "You are GitHub Copilot, an AI programming assistant.",
        );
        format!(
            "{{\"model\":\"{}\",\"messages\":[\
             {{\"role\":\"system\",\"content\":{}}},\
             {{\"role\":\"user\",\"content\":{}}}]}}",
            escape_json(model),
            json_string(system),
            json_string(&request.prompt),
        )
    }

    fn parse_response(&self, request: &AiCliRequest, raw: &str) -> Result<AiCliResponse, String> {
        let content = extract_json_field(raw, "content")
            .or_else(|| extract_nested_content(raw))
            .unwrap_or_else(|| raw.to_string());
        let model_used = extract_json_field(raw, "model")
            .unwrap_or_else(|| request.model.clone().unwrap_or_else(|| "gpt-4o".to_string()));
        Ok(AiCliResponse {
            provider: AiProvider::GithubCopilot,
            model_used,
            content,
            raw_fields: BTreeMap::new(),
            latency_ms: 0,
            truncated: false,
        })
    }

    fn endpoint_url(&self) -> &'static str {
        "https://api.githubcopilot.com/chat/completions"
    }
}

// ─── Claude (Anthropic) adapter ───────────────────────────────

/// Adapter for Anthropic Claude API.
pub struct ClaudeAdapter;

impl AiCliAdapter for ClaudeAdapter {
    fn provider(&self) -> AiProvider {
        AiProvider::Claude
    }

    fn build_request_body(&self, request: &AiCliRequest) -> String {
        let model = request
            .model
            .as_deref()
            .unwrap_or("claude-opus-4-5");
        let system_block = match &request.system {
            Some(s) => format!(",\"system\":{}", json_string(s)),
            None => String::new(),
        };
        format!(
            "{{\"model\":\"{}\",\"max_tokens\":8192{},\
             \"messages\":[{{\"role\":\"user\",\"content\":{}}}]}}",
            escape_json(model),
            system_block,
            json_string(&request.prompt),
        )
    }

    fn parse_response(&self, request: &AiCliRequest, raw: &str) -> Result<AiCliResponse, String> {
        // Claude returns: {"content":[{"type":"text","text":"..."}],"model":"..."}
        let content = extract_nested_content(raw)
            .or_else(|| extract_json_field(raw, "text"))
            .unwrap_or_else(|| raw.to_string());
        let model_used = extract_json_field(raw, "model")
            .unwrap_or_else(|| request.model.clone().unwrap_or_else(|| "claude-opus-4-5".to_string()));
        let truncated = extract_json_field(raw, "stop_reason")
            .map(|r| r == "max_tokens")
            .unwrap_or(false);
        Ok(AiCliResponse {
            provider: AiProvider::Claude,
            model_used,
            content,
            raw_fields: BTreeMap::new(),
            latency_ms: 0,
            truncated,
        })
    }

    fn endpoint_url(&self) -> &'static str {
        "https://api.anthropic.com/v1/messages"
    }
}

// ─── Gemini adapter ───────────────────────────────────────────

/// Adapter for Google Gemini / Vertex AI.
pub struct GeminiAdapter;

impl AiCliAdapter for GeminiAdapter {
    fn provider(&self) -> AiProvider {
        AiProvider::Gemini
    }

    fn build_request_body(&self, request: &AiCliRequest) -> String {
        let system_block = match &request.system {
            Some(s) => format!("\"systemInstruction\":{{\"parts\":[{{\"text\":{}}}]}},", json_string(s)),
            None => String::new(),
        };
        format!(
            "{{{}\
             \"contents\":[{{\"role\":\"user\",\"parts\":[{{\"text\":{}}}]}}]}}",
            system_block,
            json_string(&request.prompt),
        )
    }

    fn parse_response(&self, request: &AiCliRequest, raw: &str) -> Result<AiCliResponse, String> {
        // Gemini: {"candidates":[{"content":{"parts":[{"text":"..."}]}}],"modelVersion":"..."}
        let content = extract_gemini_text(raw)
            .unwrap_or_else(|| raw.to_string());
        let model_used = extract_json_field(raw, "modelVersion")
            .unwrap_or_else(|| request.model.clone().unwrap_or_else(|| "gemini-2.0-flash".to_string()));
        Ok(AiCliResponse {
            provider: AiProvider::Gemini,
            model_used,
            content,
            raw_fields: BTreeMap::new(),
            latency_ms: 0,
            truncated: false,
        })
    }

    fn endpoint_url(&self) -> &'static str {
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.0-flash:generateContent"
    }
}

// ─── OpenAI adapter ───────────────────────────────────────────

/// Adapter for OpenAI Chat Completions API.
pub struct OpenAiAdapter;

impl AiCliAdapter for OpenAiAdapter {
    fn provider(&self) -> AiProvider {
        AiProvider::OpenAi
    }

    fn build_request_body(&self, request: &AiCliRequest) -> String {
        let model = request
            .model
            .as_deref()
            .unwrap_or("gpt-4o");
        let system_msg = match &request.system {
            Some(s) => format!(
                "{{\"role\":\"system\",\"content\":{}}},",
                json_string(s)
            ),
            None => String::new(),
        };
        format!(
            "{{\"model\":\"{}\",\"messages\":[{}{{\"role\":\"user\",\"content\":{}}}]}}",
            escape_json(model),
            system_msg,
            json_string(&request.prompt),
        )
    }

    fn parse_response(&self, request: &AiCliRequest, raw: &str) -> Result<AiCliResponse, String> {
        // OpenAI: {"choices":[{"message":{"content":"..."},"finish_reason":"stop"}],"model":"..."}
        let content = extract_openai_content(raw)
            .or_else(|| extract_nested_content(raw))
            .unwrap_or_else(|| raw.to_string());
        let model_used = extract_json_field(raw, "model")
            .unwrap_or_else(|| request.model.clone().unwrap_or_else(|| "gpt-4o".to_string()));
        let truncated = extract_openai_finish_reason(raw)
            .map(|r| r == "length")
            .unwrap_or(false);
        Ok(AiCliResponse {
            provider: AiProvider::OpenAi,
            model_used,
            content,
            raw_fields: BTreeMap::new(),
            latency_ms: 0,
            truncated,
        })
    }

    fn endpoint_url(&self) -> &'static str {
        "https://api.openai.com/v1/chat/completions"
    }
}

// ─── Factory ──────────────────────────────────────────────────

/// Return the adapter for a given provider.
pub fn adapter_for(provider: &AiProvider) -> Box<dyn AiCliAdapter> {
    match provider {
        AiProvider::GithubCopilot => Box::new(GithubCopilotAdapter),
        AiProvider::Claude => Box::new(ClaudeAdapter),
        AiProvider::Gemini => Box::new(GeminiAdapter),
        AiProvider::OpenAi => Box::new(OpenAiAdapter),
    }
}

// ─── Minimal JSON helpers (no external deps) ──────────────────

/// Wrap a string as a JSON string literal with proper escaping.
pub(crate) fn json_string(s: &str) -> String {
    format!("\"{}\"", escape_json(s))
}

/// Escape a string for embedding inside a JSON string literal.
pub(crate) fn escape_json(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => out.push(c),
        }
    }
    out
}

/// Naively extract a top-level string field from JSON by key name.
/// Returns `None` if not found.  Not a full parser — used only for
/// well-known fields in controlled API responses.
pub(crate) fn extract_json_field(json: &str, key: &str) -> Option<String> {
    let needle = format!("\"{}\":", key);
    let start = json.find(&needle)? + needle.len();
    let rest = json[start..].trim_start();
    if let Some(inner) = rest.strip_prefix('"') {
        let mut value = String::new();
        let mut escaped = false;
        for c in inner.chars() {
            if escaped {
                match c {
                    '"' => value.push('"'),
                    '\\' => value.push('\\'),
                    'n' => value.push('\n'),
                    'r' => value.push('\r'),
                    't' => value.push('\t'),
                    _ => {
                        value.push('\\');
                        value.push(c);
                    }
                }
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                break;
            } else {
                value.push(c);
            }
        }
        Some(value)
    } else {
        None
    }
}

/// Extract text from Anthropic/OpenAI nested content array.
/// Handles: `"content":[{"type":"text","text":"..."}]`
fn extract_nested_content(json: &str) -> Option<String> {
    // Look for "text": after "content"
    let content_pos = json.find("\"content\"")?;
    let after = &json[content_pos..];
    extract_json_field(after, "text")
}

/// Extract text from Gemini candidates array.
/// `"candidates":[{"content":{"parts":[{"text":"..."}]}}]`
fn extract_gemini_text(json: &str) -> Option<String> {
    let parts_pos = json.find("\"parts\"")?;
    let after = &json[parts_pos..];
    extract_json_field(after, "text")
}

/// Extract content text from OpenAI choices[0].message.content
fn extract_openai_content(json: &str) -> Option<String> {
    let choices_pos = json.find("\"choices\"")?;
    let after = &json[choices_pos..];
    let msg_pos = after.find("\"message\"")?;
    let after_msg = &after[msg_pos..];
    extract_json_field(after_msg, "content")
}

/// Extract finish_reason from OpenAI response.
fn extract_openai_finish_reason(json: &str) -> Option<String> {
    extract_json_field(json, "finish_reason")
}

// ─── Tests ────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_display_names() {
        assert_eq!(AiProvider::GithubCopilot.display_name(), "GitHub Copilot");
        assert_eq!(AiProvider::Claude.display_name(), "Anthropic Claude");
        assert_eq!(AiProvider::Gemini.display_name(), "Google Gemini");
        assert_eq!(AiProvider::OpenAi.display_name(), "OpenAI");
    }

    #[test]
    fn test_provider_from_str() {
        assert_eq!(AiProvider::parse("gh"), Some(AiProvider::GithubCopilot));
        assert_eq!(AiProvider::parse("claude"), Some(AiProvider::Claude));
        assert_eq!(AiProvider::parse("gemini"), Some(AiProvider::Gemini));
        assert_eq!(AiProvider::parse("openai"), Some(AiProvider::OpenAi));
        assert_eq!(AiProvider::parse("unknown"), None);
    }

    #[test]
    fn test_provider_token_env_vars() {
        assert_eq!(AiProvider::GithubCopilot.token_env_var(), "GITHUB_TOKEN");
        assert_eq!(AiProvider::Claude.token_env_var(), "ANTHROPIC_API_KEY");
        assert_eq!(AiProvider::Gemini.token_env_var(), "GEMINI_API_KEY");
        assert_eq!(AiProvider::OpenAi.token_env_var(), "OPENAI_API_KEY");
    }

    #[test]
    fn test_escape_json() {
        assert_eq!(escape_json("hello"), "hello");
        assert_eq!(escape_json("say \"hi\""), "say \\\"hi\\\"");
        assert_eq!(escape_json("line1\nline2"), "line1\\nline2");
        assert_eq!(escape_json("tab\there"), "tab\\there");
    }

    #[test]
    fn test_json_string() {
        assert_eq!(json_string("hello"), "\"hello\"");
        assert_eq!(json_string("a\"b"), "\"a\\\"b\"");
    }

    #[test]
    fn test_extract_json_field_simple() {
        let json = r#"{"model":"gpt-4o","content":"hello world"}"#;
        assert_eq!(extract_json_field(json, "model"), Some("gpt-4o".to_string()));
        assert_eq!(extract_json_field(json, "content"), Some("hello world".to_string()));
        assert_eq!(extract_json_field(json, "missing"), None);
    }

    #[test]
    fn test_github_copilot_request_body() {
        let req = AiCliRequest::new(AiProvider::GithubCopilot, "explain recursion");
        let adapter = GithubCopilotAdapter;
        let body = adapter.build_request_body(&req);
        assert!(body.contains("\"model\":\"gpt-4o\""));
        assert!(body.contains("explain recursion"));
        assert!(body.contains("user"));
    }

    #[test]
    fn test_claude_request_body() {
        let req = AiCliRequest::new(AiProvider::Claude, "write a test");
        let adapter = ClaudeAdapter;
        let body = adapter.build_request_body(&req);
        assert!(body.contains("claude-opus-4-5"));
        assert!(body.contains("max_tokens"));
        assert!(body.contains("write a test"));
    }

    #[test]
    fn test_claude_request_body_with_system() {
        let mut req = AiCliRequest::new(AiProvider::Claude, "summarize");
        req.system = Some("You are a helpful assistant.".to_string());
        let adapter = ClaudeAdapter;
        let body = adapter.build_request_body(&req);
        assert!(body.contains("systemInstruction") == false);
        assert!(body.contains("\"system\""));
        assert!(body.contains("You are a helpful assistant."));
    }

    #[test]
    fn test_gemini_request_body() {
        let req = AiCliRequest::new(AiProvider::Gemini, "translate to French");
        let adapter = GeminiAdapter;
        let body = adapter.build_request_body(&req);
        assert!(body.contains("contents"));
        assert!(body.contains("translate to French"));
    }

    #[test]
    fn test_openai_request_body() {
        let req = AiCliRequest::new(AiProvider::OpenAi, "fix this bug");
        let adapter = OpenAiAdapter;
        let body = adapter.build_request_body(&req);
        assert!(body.contains("\"model\":\"gpt-4o\""));
        assert!(body.contains("fix this bug"));
    }

    #[test]
    fn test_claude_parse_response() {
        let raw = r#"{"id":"msg_01","type":"message","role":"assistant","content":[{"type":"text","text":"Hello there!"}],"model":"claude-opus-4-5","stop_reason":"end_turn"}"#;
        let req = AiCliRequest::new(AiProvider::Claude, "say hi");
        let adapter = ClaudeAdapter;
        let resp = adapter.parse_response(&req, raw).unwrap();
        assert_eq!(resp.content, "Hello there!");
        assert_eq!(resp.model_used, "claude-opus-4-5");
        assert!(!resp.truncated);
    }

    #[test]
    fn test_ai_cli_response_to_json() {
        let resp = AiCliResponse {
            provider: AiProvider::OpenAi,
            model_used: "gpt-4o".to_string(),
            content: "hello".to_string(),
            raw_fields: BTreeMap::new(),
            latency_ms: 42,
            truncated: false,
        };
        let json = resp.to_json();
        assert!(json.contains("\"provider\":\"OpenAI\""));
        assert!(json.contains("\"model_used\":\"gpt-4o\""));
        assert!(json.contains("\"content\":\"hello\""));
        assert!(json.contains("\"latency_ms\":42"));
    }

    #[test]
    fn test_adapter_factory() {
        assert_eq!(adapter_for(&AiProvider::GithubCopilot).provider(), AiProvider::GithubCopilot);
        assert_eq!(adapter_for(&AiProvider::Claude).provider(), AiProvider::Claude);
        assert_eq!(adapter_for(&AiProvider::Gemini).provider(), AiProvider::Gemini);
        assert_eq!(adapter_for(&AiProvider::OpenAi).provider(), AiProvider::OpenAi);
    }
}
